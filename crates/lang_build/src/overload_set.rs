use std::collections::{BTreeMap, BTreeSet};

use lang_syntax::{
    NormClosure, NormClosureBody, NormExpr, NormForm, NormPattern, NormPatternElem, NormProductElem,
};

use crate::{
    callable_body_allows_execution,
    graph::{NamespaceGraphSnapshot, ResolverContext},
    initializer_eval::{evaluate_initializer_best_effort, EvalMode, EvalOutcome},
    meta_body::selected_meta_delete_diagnostic,
    meta_invocation::{
        ForwardedValue, MetaInvocationResult, MetaInvocationValue, MetaValueTarget, ReturnViewShape,
    },
    model::{
        Diagnostic, DiagnosticSeverity, ExecutionEnv, NamespaceNodeId, PolicyFlag, PolicySet,
        Provenance, ResolverCode, SourceCallableObject, SymbolId, SymbolKind, SymbolObject,
        SymbolPayload,
    },
    overload_pattern::{
        decode_param_pattern, match_param_pattern, overload_args_from_classified_shape,
        OverloadArgShape, RestrictedParamPattern, SpecificityTuple,
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
pub enum RestrictedMetaInvocationOutcome {
    Value {
        value: MetaInvocationValue,
        result_policy: PolicySet,
    },
    Diagnostic {
        diagnostic: Diagnostic,
        failure_kind: RestrictedOverloadFailureKind,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RestrictedOverloadFailureKind {
    InvalidTarget,
    NoSourceDeclaredCallable {
        callable_name: String,
    },
    NotVisibleToLookupPhase {
        callable_name: String,
        lookup_phase: LookupPhase,
    },
    NoApplicableCandidate {
        callable_name: String,
    },
    BodyEntryPolicyMismatch {
        demanded_execution: ExecutionEnv,
    },
    AmbiguousCandidate {
        specificity: SpecificityTuple,
    },
    UnsupportedExternalVisibility,
    UnsupportedCandidateShape,
    UnsupportedParameterPattern,
    UnsupportedCanonicalSumPatternValue,
    UnsupportedSelectedMetaBody,
    UnsupportedSelectedMetaBodyLocalBinding,
    SelectedDeleteBodyDiagnostic,
}

impl RestrictedOverloadFailureKind {
    pub fn diagnostic_code(&self) -> ResolverCode {
        match self {
            Self::InvalidTarget => ResolverCode::UnsupportedOverloadTarget,
            Self::NoSourceDeclaredCallable { .. }
            | Self::NotVisibleToLookupPhase { .. }
            | Self::NoApplicableCandidate { .. } => ResolverCode::NoMetaVisibleCandidate,
            Self::BodyEntryPolicyMismatch { .. } => ResolverCode::BodyEntryPolicyMismatch,
            Self::AmbiguousCandidate { .. } => ResolverCode::AmbiguousMetaCandidate,
            Self::UnsupportedExternalVisibility => ResolverCode::UnsupportedExternalVisibility,
            Self::UnsupportedCandidateShape => ResolverCode::UnsupportedCandidateShape,
            Self::UnsupportedParameterPattern => ResolverCode::UnsupportedParameterPattern,
            Self::UnsupportedCanonicalSumPatternValue => {
                ResolverCode::UnsupportedCanonicalSumPatternValue
            }
            Self::UnsupportedSelectedMetaBody => ResolverCode::UnsupportedSelectedMetaBody,
            Self::UnsupportedSelectedMetaBodyLocalBinding => {
                ResolverCode::UnsupportedSelectedMetaBodyLocalBinding
            }
            Self::SelectedDeleteBodyDiagnostic => ResolverCode::UnsupportedSelectedMetaBody,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RestrictedOverloadFailure {
    pub diagnostic: Diagnostic,
    pub kind: RestrictedOverloadFailureKind,
}

#[derive(Clone, Debug)]
struct ApplicableCandidate {
    symbol: SymbolObject,
    source_callable: SourceCallableObject,
    bindings: BTreeMap<String, OverloadArgShape>,
    specificity: SpecificityTuple,
    return_slot_name: String,
}

enum CandidateApplicabilityFailure {
    Inapplicable(Diagnostic),
    UnsupportedParameterPattern(Diagnostic),
    UnsupportedCandidateShape(Diagnostic),
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
    match invoke_restricted_meta_overload_with_policy(
        snapshot,
        namespace,
        call_site,
        resolver_context,
        lookup_phase,
        demanded_execution,
        visibility,
        provenance,
    ) {
        RestrictedMetaInvocationOutcome::Value { value, .. } => MetaInvocationResult::Value(value),
        RestrictedMetaInvocationOutcome::Diagnostic { diagnostic, .. } => {
            MetaInvocationResult::Diagnostic(diagnostic)
        }
    }
}

pub fn invoke_restricted_meta_overload_with_policy(
    snapshot: &NamespaceGraphSnapshot,
    namespace: NamespaceNodeId,
    call_site: &NormalizedCallSite,
    resolver_context: &ResolverContext,
    lookup_phase: LookupPhase,
    demanded_execution: ExecutionEnv,
    visibility: VisibilityView,
    provenance: Provenance,
) -> RestrictedMetaInvocationOutcome {
    let Some(callable_name) = callable_name_from_target(&call_site.target) else {
        return diagnostic_outcome(
            RestrictedOverloadFailureKind::InvalidTarget,
            "restricted overload selection requires a name or operator target",
            call_site.provenance.clone(),
        );
    };
    let arg_shape = call_site.to_arg_product_shape(ProductMaterialRole::CallableArgumentProduct);
    let classified =
        classify_type_arguments_with_report(&arg_shape, &snapshot.capability(), resolver_context);
    let selected = match select_restricted_meta_overload_structured(OverloadSelectionInput {
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
        Err(failure) => {
            return RestrictedMetaInvocationOutcome::Diagnostic {
                diagnostic: failure.diagnostic,
                failure_kind: failure.kind,
            };
        }
    };
    let result_policy = selected_return_object_policy(&selected);
    match evaluate_selected_source_meta_body(snapshot, resolver_context, &selected) {
        Ok(value) => RestrictedMetaInvocationOutcome::Value {
            value,
            result_policy,
        },
        Err(failure) => RestrictedMetaInvocationOutcome::Diagnostic {
            diagnostic: failure.diagnostic,
            failure_kind: failure.kind,
        },
    }
}

fn diagnostic_outcome(
    kind: RestrictedOverloadFailureKind,
    message: impl Into<String>,
    provenance: Provenance,
) -> RestrictedMetaInvocationOutcome {
    let diagnostic =
        Diagnostic::hard_error(message, Some(provenance)).with_code(kind.diagnostic_code());
    RestrictedMetaInvocationOutcome::Diagnostic {
        diagnostic,
        failure_kind: kind,
    }
}

fn selected_return_object_policy(selected: &SelectedOverloadCandidate) -> PolicySet {
    match &selected.symbol.payload {
        SymbolPayload::MetaFunction(meta_function) => {
            meta_function.return_object_policy.policy_set.clone()
        }
        _ => selected.symbol.policy_metadata.policy_set.clone(),
    }
}

pub fn select_restricted_meta_overload(
    input: OverloadSelectionInput<'_>,
) -> Result<SelectedOverloadCandidate, Diagnostic> {
    select_restricted_meta_overload_structured(input).map_err(|failure| failure.diagnostic)
}

pub fn select_restricted_meta_overload_structured(
    input: OverloadSelectionInput<'_>,
) -> Result<SelectedOverloadCandidate, RestrictedOverloadFailure> {
    if input.visibility == VisibilityView::External {
        return Err(overload_failure(
            RestrictedOverloadFailureKind::UnsupportedExternalVisibility,
            "External visibility is not implemented in the restricted v0.8 overload selector",
            input.provenance,
        ));
    }
    let candidate_set = construct_c0(&input);
    if candidate_set.c0_symbol_ids.is_empty() {
        return Err(overload_failure(
            RestrictedOverloadFailureKind::NoSourceDeclaredCallable {
                callable_name: input.callable_name.clone(),
            },
            format!(
                "no matching overload candidate: no source-declared callable `{}` in selected namespace view",
                input.callable_name
            ),
            input.provenance,
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
        return Err(overload_failure(
            RestrictedOverloadFailureKind::NotVisibleToLookupPhase {
                callable_name: input.callable_name.clone(),
                lookup_phase: input.lookup_phase,
            },
            format!(
                "no matching overload candidate: `{}` is not visible to {:?}",
                input.callable_name, input.lookup_phase
            ),
            input.provenance,
        ));
    }

    let mut shape_matches = Vec::new();
    let mut first_unsupported_shape = None;
    let mut first_unsupported_pattern = None;
    let mut last_inapplicable = None;
    for symbol in policy_visible {
        match applicable_candidate_from_symbol(&symbol, &args, input.demanded_execution) {
            Ok(candidate) => shape_matches.push(candidate),
            Err(CandidateApplicabilityFailure::UnsupportedCandidateShape(d)) => {
                if first_unsupported_shape.is_none() {
                    first_unsupported_shape = Some(d);
                }
            }
            Err(CandidateApplicabilityFailure::UnsupportedParameterPattern(d)) => {
                if first_unsupported_pattern.is_none() {
                    first_unsupported_pattern = Some(d);
                }
            }
            Err(CandidateApplicabilityFailure::Inapplicable(d)) => {
                last_inapplicable = Some(d);
            }
        }
    }
    if shape_matches.is_empty() {
        if let Some(diagnostic) = first_unsupported_shape {
            return Err(RestrictedOverloadFailure {
                diagnostic: diagnostic.with_code(
                    RestrictedOverloadFailureKind::UnsupportedCandidateShape.diagnostic_code(),
                ),
                kind: RestrictedOverloadFailureKind::UnsupportedCandidateShape,
            });
        }
        if let Some(diagnostic) = first_unsupported_pattern {
            return Err(RestrictedOverloadFailure {
                diagnostic: diagnostic.with_code(
                    RestrictedOverloadFailureKind::UnsupportedParameterPattern.diagnostic_code(),
                ),
                kind: RestrictedOverloadFailureKind::UnsupportedParameterPattern,
            });
        }
        let kind = RestrictedOverloadFailureKind::NoApplicableCandidate {
            callable_name: input.callable_name.clone(),
        };
        if let Some(diagnostic) = last_inapplicable {
            return Err(RestrictedOverloadFailure {
                diagnostic: diagnostic.with_code(kind.diagnostic_code()),
                kind,
            });
        }
        return Err(overload_failure(
            kind,
            format!(
                "no matching overload candidate for `{}`",
                input.callable_name
            ),
            input.provenance.clone(),
        ));
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
        return Err(overload_failure(
            RestrictedOverloadFailureKind::BodyEntryPolicyMismatch {
                demanded_execution: input.demanded_execution,
            },
            "candidate body-entry policy does not admit demanded execution",
            input.provenance,
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
        _ => Err(overload_failure(
            RestrictedOverloadFailureKind::AmbiguousCandidate {
                specificity: max_specificity,
            },
            format!(
                "ambiguous overload candidate: duplicate overload declaration is ambiguous only when selected (specificity {:?})",
                max_specificity
            ),
            input.provenance,
        )),
    }
}

fn overload_failure(
    kind: RestrictedOverloadFailureKind,
    message: impl Into<String>,
    provenance: Provenance,
) -> RestrictedOverloadFailure {
    let diagnostic =
        Diagnostic::hard_error(message, Some(provenance)).with_code(kind.diagnostic_code());
    RestrictedOverloadFailure { diagnostic, kind }
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
) -> Result<ApplicableCandidate, CandidateApplicabilityFailure> {
    let SymbolPayload::MetaFunction(meta_function) = &symbol.payload else {
        return Err(CandidateApplicabilityFailure::UnsupportedCandidateShape(
            Diagnostic::hard_error(
                "overload candidate is not a meta-function payload",
                Some(symbol.provenance.clone()),
            ),
        ));
    };
    let source_callable = meta_function.source_callable.clone().ok_or_else(|| {
        CandidateApplicabilityFailure::UnsupportedCandidateShape(Diagnostic::hard_error(
            "overload candidate is not source-declared callable material",
            Some(symbol.provenance.clone()),
        ))
    })?;
    let head = source_callable.closure.head.as_ref().ok_or_else(|| {
        CandidateApplicabilityFailure::UnsupportedCandidateShape(Diagnostic::hard_error(
            "overload candidate lacks explicit closure head",
            Some(source_callable.provenance.clone()),
        ))
    })?;
    if head.params.len() != args.len() + 1 {
        return Err(CandidateApplicabilityFailure::UnsupportedCandidateShape(
            Diagnostic::hard_error(
                format!(
                    "overload candidate arity mismatch: expected {} explicit args, got {}",
                    head.params.len().saturating_sub(1),
                    args.len()
                ),
                Some(source_callable.provenance.clone()),
            ),
        ));
    }

    let return_slot_name = return_slot_name(&source_callable.closure)
        .map_err(CandidateApplicabilityFailure::UnsupportedCandidateShape)?;
    let mut specificity = SpecificityTuple::default();
    let mut bindings = BTreeMap::new();
    for (param, arg) in head.params.iter().skip(1).zip(args) {
        let pattern = decode_param_pattern(param);
        if let RestrictedParamPattern::Unsupported { reason, provenance } = &pattern {
            return Err(CandidateApplicabilityFailure::UnsupportedParameterPattern(
                Diagnostic::hard_error(
                    format!("unsupported parameter extraction pattern: {reason}"),
                    Some(provenance.clone()),
                ),
            ));
        }
        let outcome = match_param_pattern(&pattern, arg)
            .map_err(CandidateApplicabilityFailure::Inapplicable)?;
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
) -> Result<MetaInvocationValue, RestrictedOverloadFailure> {
    match &selected.source_callable.closure.body {
        NormClosureBody::Delete(delete) => {
            let diagnostic = selected_meta_delete_diagnostic(
                delete,
                selected.source_callable.provenance.clone(),
            )
            .with_code(ResolverCode::UnsupportedSelectedMetaBody);
            Err(RestrictedOverloadFailure {
                diagnostic,
                kind: RestrictedOverloadFailureKind::SelectedDeleteBodyDiagnostic,
            })
        }
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
) -> Result<MetaInvocationValue, RestrictedOverloadFailure> {
    let mut final_expr = None;
    let mut local_names = BTreeSet::new();
    for form in &program.forms {
        match form {
            NormForm::Let(lang_syntax::NormDecl::Let { slot, .. }) => {
                if let Some(initializer) = slot.initializer.as_deref() {
                    if expr_refs_selected_or_local_binding(initializer, selected, &local_names) {
                        return Err(selected_body_failure(
                            selected,
                            RestrictedOverloadFailureKind::UnsupportedSelectedMetaBodyLocalBinding,
                            "UnsupportedSelectedMetaBodyLocalBinding: selected meta body local binding environment is not implemented in the restricted v0.8 evaluator",
                        ));
                    }
                    match evaluate_initializer_best_effort(
                        snapshot,
                        selected
                            .symbol
                            .parent
                            .unwrap_or_else(|| snapshot.root_node()),
                        initializer,
                        resolver_context,
                        EvalMode::MetaStrict,
                        Provenance::from_norm_origin("selected meta body local let", &slot.origin),
                    ) {
                        EvalOutcome::Value { .. } => {}
                        EvalOutcome::Residual {
                            reason, provenance, ..
                        } => {
                            return Err(RestrictedOverloadFailure {
                                diagnostic: Diagnostic::hard_error(
                                format!(
                                    "ResidualNotAllowedInMetaStrict: runtime-only dependency in MetaStrict context ({reason:?})"
                                ),
                                Some(provenance),
                                )
                                .with_code(ResolverCode::ResidualNotAllowedInMetaStrict),
                                kind: RestrictedOverloadFailureKind::UnsupportedSelectedMetaBody,
                            });
                        }
                        EvalOutcome::Diagnostic(diagnostic) => {
                            return Err(RestrictedOverloadFailure {
                                diagnostic,
                                kind: RestrictedOverloadFailureKind::UnsupportedSelectedMetaBody,
                            });
                        }
                    }
                }
                if let Some(name) = binding_slot_name(slot) {
                    local_names.insert(name);
                }
            }
            NormForm::Expr(expr) if final_expr.is_none() => final_expr = Some(expr),
            _ => {
                return Err(unsupported_body(
                    selected,
                    RestrictedOverloadFailureKind::UnsupportedSelectedMetaBody,
                    "selected meta body form is outside the restricted v0.8 meta-overload evaluator",
                ));
            }
        }
    }
    let Some(expr) = final_expr else {
        return Err(unsupported_body(
            selected,
            RestrictedOverloadFailureKind::UnsupportedSelectedMetaBody,
            "selected meta body form is outside the restricted v0.8 meta-overload evaluator",
        ));
    };
    if forwarding_expr_is_canonical_sum(expr, &selected.return_slot_name) {
        return Err(unsupported_body(
            selected,
            RestrictedOverloadFailureKind::UnsupportedCanonicalSumPatternValue,
            "UnsupportedCanonicalSumPatternValue: unsupported canonical sum-pattern value in restricted v0.8 initializer meta evaluation",
        ));
    }
    let Some(rhs_name) = forwarding_equality_rhs(expr, &selected.return_slot_name) else {
        if forwarding_rhs_is_canonical_sum(expr, &selected.return_slot_name) {
            return Err(unsupported_body(
                selected,
                RestrictedOverloadFailureKind::UnsupportedCanonicalSumPatternValue,
                "UnsupportedCanonicalSumPatternValue: unsupported canonical sum-pattern value in restricted v0.8 initializer meta evaluation",
            ));
        }
        return Err(unsupported_body(
            selected,
            RestrictedOverloadFailureKind::UnsupportedSelectedMetaBody,
            "selected meta body form is outside the restricted v0.8 meta-overload evaluator",
        ));
    };
    if local_names.contains(&rhs_name) {
        return Err(selected_body_failure(
            selected,
            RestrictedOverloadFailureKind::UnsupportedSelectedMetaBodyLocalBinding,
            "UnsupportedSelectedMetaBodyLocalBinding: selected meta body local binding environment is not implemented in the restricted v0.8 evaluator",
        ));
    }
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
        Err(_) => Err(unsupported_body(
            selected,
            RestrictedOverloadFailureKind::UnsupportedSelectedMetaBody,
            "selected meta body form is outside the restricted v0.8 meta-overload evaluator",
        )),
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

fn forwarding_rhs_is_canonical_sum(expr: &NormExpr, return_slot_name: &str) -> bool {
    let NormExpr::Call { source, target, .. } = expr else {
        return false;
    };
    let NormExpr::OperatorTarget { spelling, .. } = target.as_ref() else {
        return false;
    };
    if spelling != "===" || source.elements.len() != 2 {
        return false;
    }
    let lhs = match &source.elements[0] {
        NormProductElem::Expr(NormExpr::Name { text, .. }) => text,
        _ => return false,
    };
    if lhs != return_slot_name {
        return false;
    }
    matches!(
        &source.elements[1],
        NormProductElem::Expr(NormExpr::Call { target, .. })
            if matches!(target.as_ref(), NormExpr::OperatorTarget { spelling, .. } if spelling == "|")
    )
}

fn forwarding_expr_is_canonical_sum(expr: &NormExpr, return_slot_name: &str) -> bool {
    let NormExpr::Call { source, target, .. } = expr else {
        return false;
    };
    let NormExpr::OperatorTarget { spelling, .. } = target.as_ref() else {
        return false;
    };
    if spelling != "|" || source.elements.is_empty() {
        return false;
    }
    source.elements.iter().any(|element| {
        matches!(
            element,
            NormProductElem::Expr(inner)
                if forwarding_equality_rhs(inner, return_slot_name).is_some()
                    || forwarding_expr_is_canonical_sum(inner, return_slot_name)
        )
    })
}

fn forwarded_type_value(
    selected: &SelectedOverloadCandidate,
    type_symbol_id: Option<SymbolId>,
) -> Result<MetaInvocationValue, RestrictedOverloadFailure> {
    let Some(type_symbol_id) = type_symbol_id else {
        return Err(unsupported_body(
            selected,
            RestrictedOverloadFailureKind::UnsupportedSelectedMetaBody,
            "selected simple forwarding body requires a graph-resolved TypeSymbol value",
        ));
    };
    Ok(MetaInvocationValue::ForwardedValue(ForwardedValue {
        target: MetaValueTarget::TypeSymbol(type_symbol_id),
        return_view: ReturnViewShape::Leaf,
        provenance: selected.source_callable.provenance.clone(),
    }))
}

fn unsupported_body(
    selected: &SelectedOverloadCandidate,
    kind: RestrictedOverloadFailureKind,
    message: impl Into<String>,
) -> RestrictedOverloadFailure {
    selected_body_failure(selected, kind, message)
}

fn selected_body_failure(
    selected: &SelectedOverloadCandidate,
    kind: RestrictedOverloadFailureKind,
    message: impl Into<String>,
) -> RestrictedOverloadFailure {
    let diagnostic = Diagnostic::new(
        DiagnosticSeverity::Error,
        message,
        Some(selected.source_callable.provenance.clone()),
    )
    .with_symbol_context(selected.symbol.id)
    .with_code(kind.diagnostic_code());
    RestrictedOverloadFailure { diagnostic, kind }
}

fn binding_slot_name(slot: &lang_syntax::NormBindingSlot) -> Option<String> {
    match &slot.value_pattern {
        NormPattern::Binder { name, .. } => Some(name.clone()),
        _ => None,
    }
}

fn expr_refs_selected_or_local_binding(
    expr: &NormExpr,
    selected: &SelectedOverloadCandidate,
    local_names: &BTreeSet<String>,
) -> bool {
    match expr {
        NormExpr::Name { text, .. } => {
            selected.bindings.contains_key(text) || local_names.contains(text)
        }
        NormExpr::Call { source, target, .. } => {
            expr_refs_selected_or_local_binding(target, selected, local_names)
                || source.elements.iter().any(|element| match element {
                    NormProductElem::Expr(expr) => {
                        expr_refs_selected_or_local_binding(expr, selected, local_names)
                    }
                    _ => false,
                })
        }
        _ => false,
    }
}

fn callable_name_from_target(target: &NormExpr) -> Option<String> {
    match target {
        NormExpr::Name { text, .. } => Some(text.clone()),
        NormExpr::OperatorTarget { spelling, .. } => Some(spelling.clone()),
        _ => None,
    }
}
