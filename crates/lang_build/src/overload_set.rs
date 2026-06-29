use std::collections::BTreeMap;

use lang_syntax::{
    NormClosure, NormClosureBody, NormExpr, NormForm, NormPattern, NormPatternElem, NormProductElem,
};

use crate::{
    callable_body_allows_execution,
    graph::{NamespaceGraphSnapshot, ResolverContext},
    meta_body::selected_meta_delete_diagnostic,
    meta_invocation::{
        ForwardedValue, MetaInvocationResult, MetaInvocationValue, MetaValueTarget, ReturnViewShape,
    },
    model::{
        Diagnostic, DiagnosticSeverity, ExecutionEnv, NamespaceNodeId, PolicyFlag, Provenance,
        SourceCallableObject, SymbolId, SymbolKind, SymbolObject, SymbolPayload,
    },
    overload_pattern::{
        decode_param_pattern, match_param_pattern, overload_args_from_classified_shape,
        OverloadArgShape, SpecificityTuple,
    },
    product_shape::ProductMaterialRole,
    type_argument::classify_type_arguments_with_report,
    NormalizedCallSite,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisibilityView {
    Internal,
    External,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LookupPhase {
    MetaAction,
    RuntimeBinding,
}

impl LookupPhase {
    fn policy_flag(self) -> PolicyFlag {
        match self {
            Self::MetaAction => PolicyFlag::Meta,
            Self::RuntimeBinding => PolicyFlag::Runtime,
        }
    }
}

#[derive(Clone, Debug)]
pub struct OverloadSelectionInput<'a> {
    pub snapshot: &'a NamespaceGraphSnapshot,
    pub namespace: NamespaceNodeId,
    pub callable_name: String,
    pub arg_product_shape: crate::ArgProductShape,
    pub lookup_phase: LookupPhase,
    pub demanded_execution: ExecutionEnv,
    pub visibility: VisibilityView,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverloadCandidateSet {
    pub c0_symbol_ids: Vec<SymbolId>,
}

#[derive(Clone, Debug)]
pub struct SelectedOverloadCandidate {
    pub symbol: SymbolObject,
    pub source_callable: SourceCallableObject,
    pub bindings: BTreeMap<String, OverloadArgShape>,
    pub specificity: SpecificityTuple,
    pub return_slot_name: String,
}

#[derive(Clone, Debug)]
struct ApplicableCandidate {
    symbol: SymbolObject,
    source_callable: SourceCallableObject,
    bindings: BTreeMap<String, OverloadArgShape>,
    specificity: SpecificityTuple,
    return_slot_name: String,
}

pub fn invoke_restricted_meta_overload(
    snapshot: &NamespaceGraphSnapshot,
    namespace: NamespaceNodeId,
    call_site: &NormalizedCallSite,
    resolver_context: &ResolverContext,
    lookup_phase: LookupPhase,
    demanded_execution: ExecutionEnv,
    visibility: VisibilityView,
    provenance: Provenance,
) -> MetaInvocationResult {
    let Some(callable_name) = callable_name_from_target(&call_site.target) else {
        return MetaInvocationResult::Diagnostic(Diagnostic::hard_error(
            "restricted overload selection requires a name or operator target",
            Some(call_site.provenance.clone()),
        ));
    };
    let arg_shape = call_site.to_arg_product_shape(ProductMaterialRole::CallableArgumentProduct);
    let classified =
        classify_type_arguments_with_report(&arg_shape, &snapshot.capability(), resolver_context);
    let selected = match select_restricted_meta_overload(OverloadSelectionInput {
        snapshot,
        namespace,
        callable_name,
        arg_product_shape: classified.classified_shape,
        lookup_phase,
        demanded_execution,
        visibility,
        provenance,
    }) {
        Ok(selected) => selected,
        Err(diagnostic) => return MetaInvocationResult::Diagnostic(diagnostic),
    };
    evaluate_selected_source_meta_body(snapshot, resolver_context, &selected)
}

pub fn select_restricted_meta_overload(
    input: OverloadSelectionInput<'_>,
) -> Result<SelectedOverloadCandidate, Diagnostic> {
    if input.visibility == VisibilityView::External {
        return Err(Diagnostic::hard_error(
            "External visibility is not implemented in the restricted v0.8 overload selector",
            Some(input.provenance),
        ));
    }
    let candidate_set = construct_c0(&input);
    if candidate_set.c0_symbol_ids.is_empty() {
        return Err(Diagnostic::hard_error(
            format!(
                "no matching overload candidate: no source-declared callable `{}` in selected namespace view",
                input.callable_name
            ),
            Some(input.provenance),
        ));
    }

    let args = overload_args_from_classified_shape(&input.arg_product_shape, |symbol_id| {
        input
            .snapshot
            .symbol(symbol_id)
            .map(|symbol| symbol.name.clone())
    });

    let mut policy_visible = Vec::new();
    for symbol_id in &candidate_set.c0_symbol_ids {
        let Some(symbol) = input.snapshot.symbol(*symbol_id).cloned() else {
            continue;
        };
        if symbol
            .policy_metadata
            .policy_set
            .contains(input.lookup_phase.policy_flag())
        {
            policy_visible.push(symbol);
        }
    }
    if policy_visible.is_empty() {
        return Err(Diagnostic::hard_error(
            format!(
                "no matching overload candidate: `{}` is not visible to {:?}",
                input.callable_name, input.lookup_phase
            ),
            Some(input.provenance),
        ));
    }

    let mut shape_matches = Vec::new();
    let mut last_pattern_error = None;
    for symbol in policy_visible {
        match applicable_candidate_from_symbol(&symbol, &args, input.demanded_execution) {
            Ok(candidate) => shape_matches.push(candidate),
            Err(diagnostic) => last_pattern_error = Some(diagnostic),
        }
    }
    if shape_matches.is_empty() {
        return Err(last_pattern_error.unwrap_or_else(|| {
            Diagnostic::hard_error(
                format!(
                    "no matching overload candidate for `{}`",
                    input.callable_name
                ),
                Some(input.provenance.clone()),
            )
        }));
    }

    let body_entry_matches = shape_matches
        .into_iter()
        .filter(|candidate| {
            callable_body_allows_execution(
                match &candidate.symbol.payload {
                    SymbolPayload::MetaFunction(meta_function) => &meta_function.body_entry_policy,
                    _ => return false,
                },
                input.demanded_execution,
            )
        })
        .collect::<Vec<_>>();
    if body_entry_matches.is_empty() {
        return Err(Diagnostic::hard_error(
            "candidate body-entry policy does not admit demanded execution",
            Some(input.provenance),
        ));
    }

    let max_specificity = body_entry_matches
        .iter()
        .map(|candidate| candidate.specificity)
        .max()
        .expect("non-empty body-entry candidate list");
    let maximal = body_entry_matches
        .into_iter()
        .filter(|candidate| candidate.specificity == max_specificity)
        .collect::<Vec<_>>();
    match maximal.as_slice() {
        [candidate] => Ok(SelectedOverloadCandidate {
            symbol: candidate.symbol.clone(),
            source_callable: candidate.source_callable.clone(),
            bindings: candidate.bindings.clone(),
            specificity: candidate.specificity,
            return_slot_name: candidate.return_slot_name.clone(),
        }),
        [] => unreachable!("maximal candidates are built from a non-empty list"),
        _ => Err(Diagnostic::hard_error(
            format!(
                "ambiguous overload candidate: duplicate overload declaration is ambiguous only when selected (specificity {:?})",
                max_specificity
            ),
            Some(input.provenance),
        )),
    }
}

pub fn construct_c0(input: &OverloadSelectionInput<'_>) -> OverloadCandidateSet {
    let mut c0_symbol_ids = Vec::new();
    let Some(node) = input.snapshot.node(input.namespace) else {
        return OverloadCandidateSet { c0_symbol_ids };
    };
    let Some(bucket) = node.children.get(&input.callable_name) else {
        return OverloadCandidateSet { c0_symbol_ids };
    };

    for symbol_id in bucket.object_symbols() {
        let Some(symbol) = input.snapshot.symbol(*symbol_id) else {
            continue;
        };
        if symbol.kind != SymbolKind::MetaFunction {
            continue;
        }
        let SymbolPayload::MetaFunction(meta_function) = &symbol.payload else {
            continue;
        };
        let Some(source_callable) = &meta_function.source_callable else {
            continue;
        };
        if callable_shape_compatible(&source_callable.closure, input.arg_product_shape.arity) {
            c0_symbol_ids.push(*symbol_id);
        }
    }
    c0_symbol_ids.sort();
    OverloadCandidateSet { c0_symbol_ids }
}

fn applicable_candidate_from_symbol(
    symbol: &SymbolObject,
    args: &[OverloadArgShape],
    _demanded_execution: ExecutionEnv,
) -> Result<ApplicableCandidate, Diagnostic> {
    let SymbolPayload::MetaFunction(meta_function) = &symbol.payload else {
        return Err(Diagnostic::hard_error(
            "overload candidate is not a meta-function payload",
            Some(symbol.provenance.clone()),
        ));
    };
    let source_callable = meta_function.source_callable.clone().ok_or_else(|| {
        Diagnostic::hard_error(
            "overload candidate is not source-declared callable material",
            Some(symbol.provenance.clone()),
        )
    })?;
    let head = source_callable.closure.head.as_ref().ok_or_else(|| {
        Diagnostic::hard_error(
            "overload candidate lacks explicit closure head",
            Some(source_callable.provenance.clone()),
        )
    })?;
    if head.params.len() != args.len() + 1 {
        return Err(Diagnostic::hard_error(
            format!(
                "overload candidate arity mismatch: expected {} explicit args, got {}",
                head.params.len().saturating_sub(1),
                args.len()
            ),
            Some(source_callable.provenance.clone()),
        ));
    }

    let return_slot_name = return_slot_name(&source_callable.closure)?;
    let mut specificity = SpecificityTuple::default();
    let mut bindings = BTreeMap::new();
    for (param, arg) in head.params.iter().skip(1).zip(args) {
        let pattern = decode_param_pattern(param);
        let outcome = match_param_pattern(&pattern, arg)?;
        specificity = specificity.add(outcome.specificity);
        bindings.extend(outcome.bindings);
    }

    Ok(ApplicableCandidate {
        symbol: symbol.clone(),
        source_callable,
        bindings,
        specificity,
        return_slot_name,
    })
}

fn callable_shape_compatible(closure: &NormClosure, explicit_arity: usize) -> bool {
    let Some(head) = &closure.head else {
        return false;
    };
    if head.params.len() != explicit_arity + 1 {
        return false;
    }
    matches!(head.params.first(), Some(NormPatternElem::BindingSlot(slot))
        if matches!(&slot.value_pattern, NormPattern::Binder { name, .. } if name == "self"))
}

fn return_slot_name(closure: &NormClosure) -> Result<String, Diagnostic> {
    let Some(head) = &closure.head else {
        return Err(Diagnostic::hard_error(
            "source callable has no explicit closure head",
            Some(Provenance::from_norm_origin(
                "source callable",
                &closure.origin,
            )),
        ));
    };
    let Some(returns) = &head.returns else {
        return Err(Diagnostic::hard_error(
            "source callable has no return slot",
            Some(Provenance::from_norm_origin(
                "source callable",
                &head.origin,
            )),
        ));
    };
    match &returns.value_pattern {
        NormPattern::Binder { name, .. } => Ok(name.clone()),
        _ => Err(Diagnostic::hard_error(
            "restricted source callable return slot must be a binder",
            Some(Provenance::from_norm_origin("return slot", &returns.origin)),
        )),
    }
}

fn evaluate_selected_source_meta_body(
    snapshot: &NamespaceGraphSnapshot,
    resolver_context: &ResolverContext,
    selected: &SelectedOverloadCandidate,
) -> MetaInvocationResult {
    match &selected.source_callable.closure.body {
        NormClosureBody::Delete(delete) => MetaInvocationResult::Diagnostic(
            selected_meta_delete_diagnostic(delete, selected.source_callable.provenance.clone()),
        ),
        NormClosureBody::Block(program) => {
            evaluate_block_body(snapshot, resolver_context, selected, program)
        }
    }
}

fn evaluate_block_body(
    snapshot: &NamespaceGraphSnapshot,
    resolver_context: &ResolverContext,
    selected: &SelectedOverloadCandidate,
    program: &lang_syntax::NormProgram,
) -> MetaInvocationResult {
    let [NormForm::Expr(expr)] = program.forms.as_slice() else {
        return unsupported_body(
            selected,
            "selected meta body form is outside the restricted v0.8 meta-overload evaluator",
        );
    };
    let Some(rhs_name) = forwarding_equality_rhs(expr, &selected.return_slot_name) else {
        return unsupported_body(
            selected,
            "selected meta body form is outside the restricted v0.8 meta-overload evaluator",
        );
    };
    if let Some(bound) = selected.bindings.get(&rhs_name) {
        return forwarded_type_value(selected, bound.type_symbol_id);
    }
    let resolved = snapshot.capability().resolve_type_object_with_policy(
        &rhs_name,
        resolver_context,
        crate::PolicyEnv::Meta,
    );
    match resolved {
        Ok(symbol) => forwarded_type_value(selected, Some(symbol.id)),
        Err(_) => unsupported_body(
            selected,
            "selected meta body form is outside the restricted v0.8 meta-overload evaluator",
        ),
    }
}

fn forwarding_equality_rhs(expr: &NormExpr, return_slot_name: &str) -> Option<String> {
    let NormExpr::Call { source, target, .. } = expr else {
        return None;
    };
    let NormExpr::OperatorTarget { spelling, .. } = target.as_ref() else {
        return None;
    };
    if spelling != "===" || source.elements.len() != 2 {
        return None;
    }
    let lhs = match &source.elements[0] {
        NormProductElem::Expr(NormExpr::Name { text, .. }) => text,
        _ => return None,
    };
    if lhs != return_slot_name {
        return None;
    }
    match &source.elements[1] {
        NormProductElem::Expr(NormExpr::Name { text, .. }) => Some(text.clone()),
        NormProductElem::Expr(NormExpr::Call { target, .. }) => match target.as_ref() {
            NormExpr::Name { text, .. } => Some(text.clone()),
            _ => None,
        },
        _ => None,
    }
}

fn forwarded_type_value(
    selected: &SelectedOverloadCandidate,
    type_symbol_id: Option<SymbolId>,
) -> MetaInvocationResult {
    let Some(type_symbol_id) = type_symbol_id else {
        return unsupported_body(
            selected,
            "selected simple forwarding body requires a graph-resolved TypeSymbol value",
        );
    };
    MetaInvocationResult::Value(MetaInvocationValue::ForwardedValue(ForwardedValue {
        target: MetaValueTarget::TypeSymbol(type_symbol_id),
        return_view: ReturnViewShape::Leaf,
        provenance: selected.source_callable.provenance.clone(),
    }))
}

fn unsupported_body(
    selected: &SelectedOverloadCandidate,
    message: impl Into<String>,
) -> MetaInvocationResult {
    MetaInvocationResult::Diagnostic(
        Diagnostic::new(
            DiagnosticSeverity::Error,
            message,
            Some(selected.source_callable.provenance.clone()),
        )
        .with_symbol_context(selected.symbol.id),
    )
}

fn callable_name_from_target(target: &NormExpr) -> Option<String> {
    match target {
        NormExpr::Name { text, .. } => Some(text.clone()),
        NormExpr::OperatorTarget { spelling, .. } => Some(spelling.clone()),
        _ => None,
    }
}
