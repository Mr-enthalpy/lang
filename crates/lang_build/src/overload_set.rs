use std::collections::{BTreeMap, BTreeSet};

use lang_syntax::{
    validate_pack_pattern_element_level, validate_pack_pattern_layers, NormClosure,
    NormClosureBody, NormExpr, NormForm, NormOverloadStrategy, NormPattern, NormPatternElem,
    NormProductElem,
};

use crate::{
    meta_body::selected_meta_delete_diagnostic,
    meta_invocation::MetaExecutionMaterial,
    model::{
        Diagnostic, DiagnosticSeverity, ExecutionEnv, Provenance, ResolverCode,
        SourceCallableObject, SymbolObject,
    },
    overload_pattern::{OverloadArgShape, SpecificityTuple},
    pattern_relation::{
        solve_parameter_product_relation, NamedPatternObservation, PatternApplicabilityProof,
        PatternRelationContext, PatternRelationFailure,
    },
    semantic_name_index::ResolverContext,
    semantic_owner::SemanticOwnerId,
    type_argument::{BodyLocalInitializerCheck, TypeResolutionEnv},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisibilityView {
    Internal,
    External,
}

/// Source-body input formed after unique selection and DynamicLegality.
#[derive(Clone, Debug)]
pub(crate) struct SelectedSourceBody {
    pub(crate) symbol: SymbolObject,
    pub(crate) source_callable: SourceCallableObject,
    pub(crate) bindings: BTreeMap<String, OverloadArgShape>,
    pub(crate) pack_bindings: BTreeMap<String, Vec<OverloadArgShape>>,
}

#[derive(Clone, Debug)]
pub struct SourceBodyEvaluationFailure {
    pub diagnostic: Diagnostic,
}

#[derive(Clone, Debug)]
pub(crate) struct ApplicableCandidate {
    pub(crate) symbol: SymbolObject,
    pub(crate) source_callable: SourceCallableObject,
    pub(crate) bindings: BTreeMap<String, OverloadArgShape>,
    pub(crate) pack_bindings: BTreeMap<String, Vec<OverloadArgShape>>,
    pub(crate) specificity: SpecificityTuple,
    /// Proof-relevant result of the canonical Pattern relation. The name-keyed
    /// maps above are one-way body-evaluator transport derived from
    /// this proof and never participate in applicability.
    pub(crate) pattern_proof: PatternApplicabilityProof,
    pub(crate) overload_strategy: NormOverloadStrategy,
}

pub(crate) enum CandidateApplicabilityFailure {
    Inapplicable(Diagnostic),
    Unsupported(Diagnostic),
}

/// Canonical A-stage entry point.
///
/// The candidate is shaped from the `OrdinaryCallEntry`'s own closure
/// handle; no graph payload is read.
pub(crate) fn applicable_candidate_from_closure(
    symbol: &SymbolObject,
    closure: &NormClosure,
    provenance: &Provenance,
    args: &[OverloadArgShape],
    demanded_execution: ExecutionEnv,
    callable_owner: SemanticOwnerId,
    resolve_named_pattern: Option<&dyn Fn(&str) -> Option<NamedPatternObservation>>,
) -> Result<ApplicableCandidate, CandidateApplicabilityFailure> {
    applicable_candidate_from_source_callable(
        symbol,
        SourceCallableObject {
            closure: closure.clone(),
            provenance: provenance.clone(),
        },
        args,
        demanded_execution,
        callable_owner,
        resolve_named_pattern,
    )
}

fn applicable_candidate_from_source_callable(
    symbol: &SymbolObject,
    source_callable: SourceCallableObject,
    args: &[OverloadArgShape],
    _demanded_execution: ExecutionEnv,
    callable_owner: SemanticOwnerId,
    resolve_named_pattern: Option<&dyn Fn(&str) -> Option<NamedPatternObservation>>,
) -> Result<ApplicableCandidate, CandidateApplicabilityFailure> {
    let head = source_callable.closure.head.as_ref().ok_or_else(|| {
        CandidateApplicabilityFailure::Unsupported(Diagnostic::hard_error(
            "overload candidate lacks explicit closure head",
            Some(source_callable.provenance.clone()),
        ))
    })?;
    let formal_frame = head.formal_frame();
    let explicit_params = formal_frame.explicit_parameters;
    validate_parameter_pack_levels(explicit_params)
        .map_err(CandidateApplicabilityFailure::Unsupported)?;
    if !parameter_arity_matches(explicit_params, args.len()) {
        return Err(CandidateApplicabilityFailure::Inapplicable(
            Diagnostic::hard_error(
                format!(
                    "overload candidate arity mismatch: parameter pattern cannot consume {} explicit args",
                    args.len()
                ),
                Some(source_callable.provenance.clone()),
            ),
        ));
    }

    let relation_context = PatternRelationContext::for_source_callable(
        &source_callable.closure,
        callable_owner,
        resolve_named_pattern,
    )
    .map_err(|failure| match failure {
        PatternRelationFailure::Inapplicable(diagnostic) => {
            CandidateApplicabilityFailure::Inapplicable(diagnostic)
        }
        PatternRelationFailure::Unsupported(diagnostic) => {
            CandidateApplicabilityFailure::Unsupported(diagnostic)
        }
    })?;
    let pattern_proof = solve_parameter_product_relation(explicit_params, args, &relation_context)
        .map_err(|failure| match failure {
            PatternRelationFailure::Inapplicable(diagnostic) => {
                CandidateApplicabilityFailure::Inapplicable(diagnostic)
            }
            PatternRelationFailure::Unsupported(diagnostic) => {
                CandidateApplicabilityFailure::Unsupported(diagnostic)
            }
        })?;
    let specificity = pattern_proof.specificity;
    let bindings = pattern_proof.named_bindings();
    let pack_bindings = pattern_proof.named_pack_bindings();

    let overload_strategy = source_callable.closure.body.overload_strategy();
    Ok(ApplicableCandidate {
        symbol: symbol.clone(),
        source_callable,
        bindings,
        pack_bindings,
        specificity,
        pattern_proof,
        overload_strategy,
    })
}

fn validate_parameter_pack_levels(params: &[NormPatternElem]) -> Result<(), Diagnostic> {
    if let Err(error) = validate_pack_pattern_element_level(params) {
        return Err(Diagnostic::hard_error(
            format!(
                "parameter Pattern contains {} pack nodes at one normalized structural level",
                error.pack_count
            ),
            Some(Provenance::from_norm_origin(
                "duplicate parameter pack level",
                &error.origin,
            )),
        ));
    }

    for param in params {
        let pattern = match param {
            NormPatternElem::Pattern(pattern) => pattern,
            NormPatternElem::BindingSlot(slot) => &slot.value_pattern,
            NormPatternElem::Unit { .. } => continue,
        };
        if let Err(error) = validate_pack_pattern_layers(pattern) {
            return Err(Diagnostic::hard_error(
                format!(
                    "parameter Pattern contains {} pack nodes at one normalized structural level",
                    error.pack_count
                ),
                Some(Provenance::from_norm_origin(
                    "duplicate nested parameter pack level",
                    &error.origin,
                )),
            ));
        }
    }
    Ok(())
}

fn param_is_pack(element: &NormPatternElem) -> bool {
    matches!(
        element,
        NormPatternElem::BindingSlot(slot)
            if matches!(&slot.value_pattern, NormPattern::Pack { .. })
    )
}

fn parameter_arity_matches(params: &[NormPatternElem], explicit_arity: usize) -> bool {
    let pack_count = params.iter().filter(|param| param_is_pack(param)).count();
    match pack_count {
        0 => params.len() == explicit_arity,
        1 => explicit_arity >= params.len().saturating_sub(1),
        _ => false,
    }
}

/// Declaration-boundary result-class elaboration.
///
/// The result class is spelled on the return slot and implies nothing about
/// Policy or privilege. The body is not inspected. The complete return
/// Pattern remains on the closure return slot and never determines the class.
///
/// Mapping:
///
/// * `-> r: type`   → `CompleteType`;
/// * `-> _: unit`   → `Unit` — the value-less result REQUIRES the `_`
///   binder (`_: unit` matches and discards the value, exactly as `_ unit`
///   in extraction matches and discards the leaf; a named binder for a
///   value-less result is a spelling error);
/// * any other annotation or no annotation → `OrdinaryValue`.
///
/// The complete return Pattern remains on the closure return slot and is
/// interpreted independently by the Pattern relation.
///
/// A future product-shaped result is one ordinary value whose Val1 is a
/// Product — still one `OrdinaryValue`: the return slot is restricted to a
/// single binder.
pub fn declared_result_class_from_closure(
    closure: &NormClosure,
) -> Result<crate::DeclaredResultClass, Diagnostic> {
    use crate::DeclaredResultClass;
    let returns = closure.head.as_ref().and_then(|head| head.returns.as_ref());
    let annotation = returns.and_then(|returns| returns.annotation.as_ref());
    let annotation_name = match annotation.map(|annotation| &annotation.pattern) {
        Some(NormPattern::Name { name, .. }) => Some(name.as_str()),
        Some(_) => return Ok(DeclaredResultClass::OrdinaryValue),
        None => None,
    };
    match annotation_name {
        Some("type") => Ok(DeclaredResultClass::CompleteType),
        Some("unit") => {
            // `_` in binder position normalizes to a wildcard skeleton
            // pattern (not a `Binder` named `_`).
            let is_wildcard_binder = returns.is_some_and(|returns| {
                matches!(
                    &returns.value_pattern,
                    NormPattern::Skeleton {
                        skeleton: lang_syntax::NormSkeleton::Wildcard { .. },
                        ..
                    }
                )
            });
            if is_wildcard_binder {
                Ok(DeclaredResultClass::Unit)
            } else {
                Err(Diagnostic::hard_error(
                    "a unit return is value-less and must be spelled `_: unit` \
                     (`_` occupies the leftmost slot so `unit` cannot be misread as the \
                     leftmost to-be-extracted name of an extraction shorthand)",
                    Some(Provenance::from_norm_origin(
                        "return slot",
                        returns
                            .map(|returns| &returns.origin)
                            .unwrap_or(&closure.origin),
                    )),
                ))
            }
        }
        Some(_) | None => Ok(DeclaredResultClass::OrdinaryValue),
    }
}

pub(crate) fn evaluate_selected_source_body(
    type_env: &dyn TypeResolutionEnv,
    resolver_context: &ResolverContext,
    selected: &SelectedSourceBody,
) -> Result<MetaExecutionMaterial, SourceBodyEvaluationFailure> {
    match &selected.source_callable.closure.body {
        NormClosureBody::Delete(delete) => {
            let diagnostic = selected_meta_delete_diagnostic(
                delete,
                selected.source_callable.provenance.clone(),
            )
            .with_code(ResolverCode::UnsupportedSelectedSourceBody);
            Err(SourceBodyEvaluationFailure { diagnostic })
        }
        NormClosureBody::Block(program) | NormClosureBody::NamedBlock { body: program, .. } => {
            evaluate_block_body(type_env, resolver_context, selected, program)
        }
        NormClosureBody::Defaulted { .. } => Err(selected_body_failure(
            selected,
            ResolverCode::UnsupportedSelectedSourceBody,
            "selected defaulted callable requires compiler default-implementation materialization",
        )),
    }
}

fn evaluate_body_local_let(
    type_env: &dyn TypeResolutionEnv,
    resolver_context: &ResolverContext,
    selected: &SelectedSourceBody,
    local_names: &BTreeSet<String>,
    slot: &lang_syntax::NormBindingSlot,
) -> Result<(), SourceBodyEvaluationFailure> {
    // Execution gap — a body-local `let x:symbol = ...` outside the
    // return-slot position has no defined meaning yet: symbol-rank
    // local construction is an undefined future construct, so it is
    // rejected explicitly instead of being accepted as a
    // checked-then-discarded dead local.  The future pass that defines
    // it must be the first to give the form positive semantics.
    if let Some(annotation) = &slot.annotation {
        if matches!(
            &annotation.pattern,
            NormPattern::Name { name, .. } if name == "symbol"
        ) {
            return Err(selected_body_failure(
                selected,
                ResolverCode::UnsupportedSelectedSourceBody,
                "symbol-rank local binding (`let ...:symbol = ...`) outside the return slot has no defined source-body execution",
            ));
        }
    }
    if let Some(initializer) = slot.initializer.as_deref() {
        if expr_refs_selected_or_local_binding(initializer, selected, local_names) {
            return Err(selected_body_failure(
                selected,
                ResolverCode::UnsupportedSelectedSourceBodyLocalBinding,
                "selected source-body local bindings are not connected to execution",
            ));
        }
        match type_env.check_body_local_initializer(
            selected.symbol.parent,
            initializer,
            resolver_context,
            Provenance::from_norm_origin("selected source-body local let", &slot.origin),
        ) {
            BodyLocalInitializerCheck::Accepted => {}
            BodyLocalInitializerCheck::Residual { reason, provenance } => {
                return Err(SourceBodyEvaluationFailure {
                    diagnostic: Diagnostic::hard_error(
                        format!(
                            "ResidualNotAllowedInMetaStrict: runtime-only dependency in MetaStrict context ({reason})"
                        ),
                        Some(provenance),
                    )
                    .with_code(ResolverCode::ResidualNotAllowedInMetaStrict),
                });
            }
            BodyLocalInitializerCheck::Rejected(diagnostic) => {
                return Err(SourceBodyEvaluationFailure { diagnostic });
            }
        }
    }
    Ok(())
}

fn evaluate_block_body(
    type_env: &dyn TypeResolutionEnv,
    resolver_context: &ResolverContext,
    selected: &SelectedSourceBody,
    program: &lang_syntax::NormProgram,
) -> Result<MetaExecutionMaterial, SourceBodyEvaluationFailure> {
    // Validate connected local forms without inventing expression completion.
    // Shared continuation execution must supply UnitDiscard and tail inference.
    let mut local_names = BTreeSet::new();

    for form in &program.forms {
        match form {
            NormForm::Let(lang_syntax::NormDecl::Let { slot, .. }) => {
                evaluate_body_local_let(type_env, resolver_context, selected, &local_names, slot)?;
                if let Some(name) = binding_slot_name(slot) {
                    local_names.insert(name);
                }
            }
            NormForm::ReturnEvent(_) => break,
            NormForm::Expr(expr) => {
                if lexical_alias_operator_shape(expr) {
                    return Err(bare_alias_spelling_failure(selected));
                }
                return Err(unsupported_body(
                    selected,
                    ResolverCode::UnsupportedSelectedSourceBody,
                    "serial expression completion requires the shared continuation consumer; UnitDiscard and implicit ReturnEvent are not yet executable",
                ));
            }
            NormForm::Let(lang_syntax::NormDecl::Alias { .. })
            | NormForm::Alias(lang_syntax::NormDecl::Alias { .. }) => {
                return Err(unsupported_lexical_alias_failure(selected));
            }
            NormForm::Let(lang_syntax::NormDecl::Error(_))
            | NormForm::Alias(lang_syntax::NormDecl::Let { .. })
            | NormForm::Alias(lang_syntax::NormDecl::Error(_))
            | NormForm::Error(_) => {
                return Err(unsupported_body(
                    selected,
                    ResolverCode::UnsupportedSelectedSourceBody,
                    "selected source body contains an unsupported non-terminal form before its terminal",
                ));
            }
        }
    }

    let report = crate::control_flow_end::compute_control_flow_end_report(program);

    if !report.diagnostics.is_empty() {
        return Err(unsupported_body(
            selected,
            ResolverCode::UnsupportedSelectedSourceBody,
            "statement after terminal block form in selected source body",
        ));
    }

    Err(unsupported_body(
        selected,
        ResolverCode::UnsupportedSelectedSourceBody,
        match report.terminal {
            Some(crate::control_flow_end::ControlFlowTerminal::ReturnEvent(event)) => {
                return_event_execution_gap_message(&event)
            }
            None => "serial block completion is not yet executable",
        },
    ))
}

/// Shape test for an illegal expression use of the lexical-alias delimiter.
fn lexical_alias_operator_shape(expr: &NormExpr) -> bool {
    let NormExpr::Call { source, target, .. } = expr else {
        return false;
    };
    let NormExpr::OperatorTarget { spelling, .. } = target.as_ref() else {
        return false;
    };
    if spelling != "===" || source.elements.len() != 2 {
        return false;
    }
    matches!(&source.elements[0], NormProductElem::Expr(_))
}

/// Expression spellings cannot become a back door to the lexical-alias
/// declaration mechanism.
fn bare_alias_spelling_failure(selected: &SelectedSourceBody) -> SourceBodyEvaluationFailure {
    unsupported_lexical_alias_failure(selected)
}

fn unsupported_lexical_alias_failure(selected: &SelectedSourceBody) -> SourceBodyEvaluationFailure {
    selected_body_failure(
        selected,
        ResolverCode::UnsupportedLexicalAlias,
        "block-local lexical alias resolution is not implemented; `===` must not create or forward a semantic entity",
    )
}

fn unsupported_body(
    selected: &SelectedSourceBody,
    code: ResolverCode,
    message: impl Into<String>,
) -> SourceBodyEvaluationFailure {
    selected_body_failure(selected, code, message)
}

/// Return target binding is available; shared completion execution is not.
fn return_event_execution_gap_message(event: &lang_syntax::NormReturnEvent) -> &'static str {
    match event.target {
        lang_syntax::NormReturnTargetSyntax::Omitted => {
            "return to the outermost enclosing function layer is not yet executable"
        }
        lang_syntax::NormReturnTargetSyntax::Explicit(_) => {
            "return to the explicitly selected active frame is not yet executable"
        }
    }
}

fn selected_body_failure(
    selected: &SelectedSourceBody,
    code: ResolverCode,
    message: impl Into<String>,
) -> SourceBodyEvaluationFailure {
    let diagnostic = Diagnostic::new(
        DiagnosticSeverity::Error,
        message,
        Some(selected.source_callable.provenance.clone()),
    )
    .with_symbol_context(selected.symbol.id)
    .with_code(code);
    SourceBodyEvaluationFailure { diagnostic }
}

fn binding_slot_name(slot: &lang_syntax::NormBindingSlot) -> Option<String> {
    match &slot.value_pattern {
        NormPattern::Binder { name, .. } => name.clone(),
        _ => None,
    }
}

fn expr_refs_selected_or_local_binding(
    expr: &NormExpr,
    selected: &SelectedSourceBody,
    local_names: &BTreeSet<String>,
) -> bool {
    match expr {
        NormExpr::PolicyLet { operand, .. } => {
            expr_refs_selected_or_local_binding(operand, selected, local_names)
        }
        NormExpr::Name { text, .. } => {
            selected.bindings.contains_key(text)
                || selected.pack_bindings.contains_key(text)
                || local_names.contains(text)
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
