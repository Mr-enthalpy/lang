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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LookupPhase {
    MetaAction,
    RuntimeBinding,
}

/// Selected-candidate carrier for the shared body evaluators.
///
/// The connected ordinary pipeline is the only producer. It builds this
/// struct directly
/// from its own prepared candidate to reuse the body evaluators, so this is
/// a plain data carrier, not selector output.
#[derive(Clone, Debug)]
pub struct SelectedOverloadCandidate {
    pub symbol: SymbolObject,
    pub source_callable: SourceCallableObject,
    pub bindings: BTreeMap<String, OverloadArgShape>,
    pub pack_bindings: BTreeMap<String, Vec<OverloadArgShape>>,
    pub specificity: SpecificityTuple,
    /// Static metadata carried into the fully-admissible candidate set. The
    /// pipeline does not invent semantics for arbitrary names.
    pub overload_strategy: NormOverloadStrategy,
    pub return_slot_name: String,
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
    UnsupportedLexicalAlias,
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
            Self::UnsupportedLexicalAlias => ResolverCode::UnsupportedLexicalAlias,
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
    pub(crate) return_slot_name: String,
}

pub(crate) enum CandidateApplicabilityFailure {
    Inapplicable(Diagnostic),
    UnsupportedParameterPattern(Diagnostic),
    UnsupportedCandidateShape(Diagnostic),
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
        CandidateApplicabilityFailure::UnsupportedCandidateShape(Diagnostic::hard_error(
            "overload candidate lacks explicit closure head",
            Some(source_callable.provenance.clone()),
        ))
    })?;
    let formal_frame = head.formal_frame();
    let explicit_params = formal_frame.explicit_parameters;
    validate_parameter_pack_levels(explicit_params)
        .map_err(CandidateApplicabilityFailure::UnsupportedParameterPattern)?;
    if !parameter_arity_matches(explicit_params, args.len()) {
        return Err(CandidateApplicabilityFailure::UnsupportedCandidateShape(
            Diagnostic::hard_error(
                format!(
                    "overload candidate arity mismatch: parameter pattern cannot consume {} explicit args",
                    args.len()
                ),
                Some(source_callable.provenance.clone()),
            ),
        ));
    }

    let return_slot_name = return_slot_name(&source_callable.closure)
        .map_err(CandidateApplicabilityFailure::UnsupportedCandidateShape)?;
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
            CandidateApplicabilityFailure::UnsupportedParameterPattern(diagnostic)
        }
    })?;
    let pattern_proof = solve_parameter_product_relation(explicit_params, args, &relation_context)
        .map_err(|failure| match failure {
            PatternRelationFailure::Inapplicable(diagnostic) => {
                CandidateApplicabilityFailure::Inapplicable(diagnostic)
            }
            PatternRelationFailure::Unsupported(diagnostic) => {
                CandidateApplicabilityFailure::UnsupportedParameterPattern(diagnostic)
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
        return_slot_name,
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

/// Declaration-boundary result-class elaboration.
///
/// The result class is spelled on the return slot and implies nothing about
/// Policy or privilege. The body is not inspected. The complete return
/// Pattern remains on the closure return slot and never determines the class.
///
/// Mapping:
///
/// * `-> r: symbol` → `ClusterSymbol` (one position, plural values under
///   one name);
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
        Some("symbol") => Ok(DeclaredResultClass::ClusterSymbol),
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

pub(crate) fn evaluate_selected_source_meta_body(
    type_env: &dyn TypeResolutionEnv,
    resolver_context: &ResolverContext,
    selected: &SelectedOverloadCandidate,
) -> Result<MetaExecutionMaterial, RestrictedOverloadFailure> {
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
        NormClosureBody::Block(program) | NormClosureBody::NamedBlock { body: program, .. } => {
            evaluate_block_body(type_env, resolver_context, selected, program)
        }
        NormClosureBody::Defaulted { .. } => Err(selected_body_failure(
            selected,
            RestrictedOverloadFailureKind::UnsupportedSelectedMetaBody,
            "selected defaulted callable requires compiler default-implementation materialization",
        )),
    }
}

fn evaluate_body_local_let(
    type_env: &dyn TypeResolutionEnv,
    resolver_context: &ResolverContext,
    selected: &SelectedOverloadCandidate,
    local_names: &BTreeSet<String>,
    slot: &lang_syntax::NormBindingSlot,
) -> Result<(), RestrictedOverloadFailure> {
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
                RestrictedOverloadFailureKind::UnsupportedSelectedMetaBody,
                "UnsupportedSelectedMetaBody: symbol-rank local binding (`let ...:symbol = ...`) outside the return slot has no defined meaning at this stage and is rejected explicitly rather than silently accepted",
            ));
        }
    }
    if let Some(initializer) = slot.initializer.as_deref() {
        if expr_refs_selected_or_local_binding(initializer, selected, local_names) {
            return Err(selected_body_failure(
                selected,
                RestrictedOverloadFailureKind::UnsupportedSelectedMetaBodyLocalBinding,
                "UnsupportedSelectedMetaBodyLocalBinding: selected meta body local binding environment is not connected to the source meta evaluator",
            ));
        }
        match type_env.check_body_local_initializer(
            selected.symbol.parent,
            initializer,
            resolver_context,
            Provenance::from_norm_origin("selected meta body local let", &slot.origin),
        ) {
            BodyLocalInitializerCheck::Accepted => {}
            BodyLocalInitializerCheck::Residual { reason, provenance } => {
                return Err(RestrictedOverloadFailure {
                    diagnostic: Diagnostic::hard_error(
                        format!(
                            "ResidualNotAllowedInMetaStrict: runtime-only dependency in MetaStrict context ({reason})"
                        ),
                        Some(provenance),
                    )
                    .with_code(ResolverCode::ResidualNotAllowedInMetaStrict),
                    kind: RestrictedOverloadFailureKind::UnsupportedSelectedMetaBody,
                });
            }
            BodyLocalInitializerCheck::Rejected(diagnostic) => {
                return Err(RestrictedOverloadFailure {
                    diagnostic,
                    kind: RestrictedOverloadFailureKind::UnsupportedSelectedMetaBody,
                });
            }
        }
    }
    Ok(())
}

/// Reject a non-name terminal expression of an ordinary body with the most
/// specific diagnostic. `===` is legal only in the lexical-alias declaration
/// form and never as an expression operator.
fn evaluate_contribution_expr(
    selected: &SelectedOverloadCandidate,
    expr: &NormExpr,
) -> RestrictedOverloadFailure {
    if lexical_alias_operator_shape(expr, &selected.return_slot_name) {
        return bare_alias_spelling_failure(selected);
    }
    unsupported_body(
        selected,
        RestrictedOverloadFailureKind::UnsupportedSelectedMetaBody,
        "selected meta body form is not supported by the source meta evaluator",
    )
}

/// Evaluate a direct-delivery terminal name to an identity-type result.
fn evaluate_contribution_rhs_name(
    type_env: &dyn TypeResolutionEnv,
    resolver_context: &ResolverContext,
    selected: &SelectedOverloadCandidate,
    local_names: &BTreeSet<String>,
    rhs_name: &str,
) -> Result<MetaExecutionMaterial, RestrictedOverloadFailure> {
    if local_names.contains(rhs_name) {
        return Err(selected_body_failure(
            selected,
            RestrictedOverloadFailureKind::UnsupportedSelectedMetaBodyLocalBinding,
            "UnsupportedSelectedMetaBodyLocalBinding: selected meta body local binding environment is not connected to the source meta evaluator",
        ));
    }
    if let Some(bound) = selected.bindings.get(rhs_name) {
        return identity_type_material(selected, bound.value_type, bound.complete_type_observation);
    }
    if selected.pack_bindings.contains_key(rhs_name) {
        return Err(unsupported_body(
            selected,
            RestrictedOverloadFailureKind::UnsupportedSelectedMetaBody,
            "selected meta body pack delivery relies on ordinary product normalization and is outside the restricted type-only evaluator",
        ));
    }
    match type_env.resolve_type_name(rhs_name, resolver_context) {
        Some(resolution) => identity_type_material(
            selected,
            Some(resolution.represented_type),
            resolution.complete_type_observation,
        ),
        None => Err(unsupported_body(
            selected,
            RestrictedOverloadFailureKind::UnsupportedSelectedMetaBody,
            "selected meta body form is not supported by the source meta evaluator",
        )),
    }
}

fn evaluate_block_body(
    type_env: &dyn TypeResolutionEnv,
    resolver_context: &ResolverContext,
    selected: &SelectedOverloadCandidate,
    program: &lang_syntax::NormProgram,
) -> Result<MetaExecutionMaterial, RestrictedOverloadFailure> {
    // Single-value body evaluation for ordinary (non-meta-construction)
    // callables: exactly one terminal contribution.  Shares the per-form
    // helpers with the clustered contributions evaluator so both read the
    // same body-shape rules.
    let mut local_names = BTreeSet::new();

    for form in &program.forms {
        match form {
            NormForm::Let(lang_syntax::NormDecl::Let { slot, .. }) => {
                evaluate_body_local_let(type_env, resolver_context, selected, &local_names, slot)?;
                if let Some(name) = binding_slot_name(slot) {
                    local_names.insert(name);
                }
            }
            NormForm::TailValue(_) | NormForm::ReturnEvent(_) => break,
            NormForm::Expr(expr) => {
                if lexical_alias_operator_shape(expr, &selected.return_slot_name) {
                    return Err(bare_alias_spelling_failure(selected));
                }
                return Err(unsupported_body(
                    selected,
                    RestrictedOverloadFailureKind::UnsupportedSelectedMetaBody,
                    "selected meta body contains ordinary expression form; expected explicit TailValue terminal",
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
                    RestrictedOverloadFailureKind::UnsupportedSelectedMetaBody,
                    "selected meta body contains unsupported non-terminal form before terminal",
                ));
            }
        }
    }

    let report = crate::control_flow_end::compute_control_flow_end_report(program);

    if !report.diagnostics.is_empty() {
        return Err(unsupported_body(
            selected,
            RestrictedOverloadFailureKind::UnsupportedSelectedMetaBody,
            "statement after terminal block form in selected meta body",
        ));
    }

    let expr = match report.terminal {
        Some(crate::control_flow_end::ControlFlowTerminal::TailValue(expr)) => expr,
        Some(crate::control_flow_end::ControlFlowTerminal::ReturnEvent(event)) => {
            return Err(unsupported_body(
                selected,
                RestrictedOverloadFailureKind::UnsupportedSelectedMetaBody,
                return_event_execution_gap_message(&event),
            ));
        }
        None => {
            return Err(unsupported_body(
                selected,
                RestrictedOverloadFailureKind::UnsupportedSelectedMetaBody,
                "selected meta body has no terminal form",
            ));
        }
    };

    // `X;` — the direct delivery terminal: the named value is delivered to
    // the direct outer layer.  This is the ordinary `expr;` terminal form,
    // not a member event.
    if let NormExpr::Name { text, .. } = &expr {
        let terminal_name = text.clone();
        if local_names.contains(&terminal_name) {
            return Err(selected_body_failure(
                selected,
                RestrictedOverloadFailureKind::UnsupportedSelectedMetaBodyLocalBinding,
                "UnsupportedSelectedMetaBodyLocalBinding: selected meta body local binding environment is not connected to the source meta evaluator",
            ));
        }
        return evaluate_contribution_rhs_name(
            type_env,
            resolver_context,
            selected,
            &local_names,
            &terminal_name,
        );
    }

    Err(evaluate_contribution_expr(selected, &expr))
}

/// Shape test for an illegal expression use of the lexical-alias delimiter.
fn lexical_alias_operator_shape(expr: &NormExpr, return_slot_name: &str) -> bool {
    let NormExpr::Call { source, target, .. } = expr else {
        return false;
    };
    let NormExpr::OperatorTarget { spelling, .. } = target.as_ref() else {
        return false;
    };
    if spelling != "===" || source.elements.len() != 2 {
        return false;
    }
    matches!(
        &source.elements[0],
        NormProductElem::Expr(NormExpr::Name { text, .. }) if text == return_slot_name
    )
}

/// Expression spellings cannot become a back door to the lexical-alias
/// declaration mechanism.
fn bare_alias_spelling_failure(selected: &SelectedOverloadCandidate) -> RestrictedOverloadFailure {
    unsupported_lexical_alias_failure(selected)
}

fn unsupported_lexical_alias_failure(
    selected: &SelectedOverloadCandidate,
) -> RestrictedOverloadFailure {
    selected_body_failure(
        selected,
        RestrictedOverloadFailureKind::UnsupportedLexicalAlias,
        "block-local lexical alias resolution is not implemented; `===` must not create or forward a semantic entity",
    )
}

fn identity_type_material(
    selected: &SelectedOverloadCandidate,
    represented_type: Option<crate::TypeValueId>,
    complete_type_observation: Option<crate::CanonicalValueAddr>,
) -> Result<MetaExecutionMaterial, RestrictedOverloadFailure> {
    let (Some(represented_type), Some(complete_type_observation)) =
        (represented_type, complete_type_observation)
    else {
        return Err(unsupported_body(
            selected,
            RestrictedOverloadFailureKind::UnsupportedSelectedMetaBody,
            "selected identity-type body requires a world-connected canonical type observation",
        ));
    };
    Ok(MetaExecutionMaterial::IdentityType(
        crate::IdentityTypeMaterial {
            type_value: represented_type,
            type_observation: crate::CanonicalTypeObservation::Observed(complete_type_observation),
            return_view: crate::ReturnViewShape::Leaf,
            provenance: selected.source_callable.provenance.clone(),
        },
    ))
}

fn unsupported_body(
    selected: &SelectedOverloadCandidate,
    kind: RestrictedOverloadFailureKind,
    message: impl Into<String>,
) -> RestrictedOverloadFailure {
    selected_body_failure(selected, kind, message)
}

/// B9 — explicit execution-gap record for the three control-flow
/// end events.  The syntax/normalizer contract distinguishes them
/// (`spec/contracts/control-flow-end-events.md`):
///
/// ```text
/// expr;              deliver to the directly enclosing layer
/// expr return;       return to the outermost function layer
/// expr (T return);   return to the layer selected by function-object type T
/// ```
///
/// The restricted meta body evaluator executes only the first form (the
/// `expr;` tail delivery).  Both return-event forms are contract-complete
/// but not yet executable; each is reported under its own documented
/// semantics rather than one blanket message, so the gap is an explicit
/// per-form record instead of a silent collapse.
fn return_event_execution_gap_message(event: &lang_syntax::NormReturnEvent) -> &'static str {
    match event.target {
        lang_syntax::NormReturnTargetSyntax::ImplicitNearest => {
            "control-flow end `expr return;` (return to the outermost function layer) is not yet executable in the restricted meta body evaluator; only the `expr;` delivery to the directly enclosing layer executes"
        }
        lang_syntax::NormReturnTargetSyntax::Explicit(_) => {
            "control-flow end `expr (T return);` (return to the layer selected by the function-object type) is not yet executable in the restricted meta body evaluator; only the `expr;` delivery to the directly enclosing layer executes"
        }
    }
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
