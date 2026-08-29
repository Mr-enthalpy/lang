//! Candidate preparation boundary before formal meta invocation.
//!
//! This module holds the candidate-preparation pipeline that sits between
//! product/argument shaping and formal meta invocation. It checks arity and
//! body-entry policy compatibility but does **not** execute meta functions,
//! resolve overloads, or perform type inference.
//!
//! Three-segment separation:
//! - `ProductObject` does **not** resolve the call target.
//! - Candidate preparation does **not** parse source.
//! - The resolver does **not** flatten products.
//!
//! `CanonicalArgProductShapeMaterial` records structural input material
//! derived from the argument product shape.

use crate::{
    identity::TypeValueId,
    model::policy_view_allows_execution,
    model::{
        CoreMetaFunction, Diagnostic, ExecutionEnv, PolicyEnv, Provenance, SymbolId, SymbolObject,
    },
    product_shape::{ArgProductShape, NonValueArgKind, RawArgValueClass},
    PolicyView,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParameterShape {
    pub expected_arity: Option<usize>,
    pub expected_arg_kinds: Vec<ParameterArgRequirement>,
    pub provenance: Provenance,
}

impl ParameterShape {
    pub fn deferred(provenance: Provenance) -> Self {
        Self {
            expected_arity: None,
            expected_arg_kinds: Vec::new(),
            provenance,
        }
    }

    pub fn exact_arity(expected_arity: usize, provenance: Provenance) -> Self {
        Self {
            expected_arity: Some(expected_arity),
            expected_arg_kinds: Vec::new(),
            provenance,
        }
    }

    /// Single-parameter signature requiring a pure type Object argument.
    pub fn type_parameter_signature(provenance: Provenance) -> Self {
        Self {
            expected_arity: Some(1),
            expected_arg_kinds: vec![ParameterArgRequirement::CoreTypeProjection],
            provenance,
        }
    }

    pub fn type_parameter_sequence(expected_arity: usize, provenance: Provenance) -> Self {
        Self {
            expected_arity: Some(expected_arity),
            expected_arg_kinds: vec![ParameterArgRequirement::CoreTypeProjection; expected_arity],
            provenance,
        }
    }
}

/// Per-argument kind requirement for parameter shape validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParameterArgRequirement {
    CoreTypeProjection,
    Deferred,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidatePreparationContext {
    pub lookup_env: PolicyEnv,
    pub demanded_execution: ExecutionEnv,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidatePolicyPlanes {
    pub lookup_env: PolicyEnv,
    pub symbol_policy_view: Option<PolicyView>,
    pub demanded_execution: ExecutionEnv,
    pub body_entry_policy: PolicyView,
    pub return_object_policy: PolicyView,
}

impl CandidatePolicyPlanes {
    pub fn body_entry_allows_demanded_execution(&self) -> bool {
        policy_view_allows_execution(&self.body_entry_policy, self.demanded_execution)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedCallableCandidate {
    pub callee_symbol_id: SymbolId,
    pub callee_name: String,
    pub callee_primitive: Option<CoreMetaFunction>,
    pub callable_kind: CallableCandidateKind,
    pub arg_product_shape: ArgProductShape,
    pub parameter_shape: ParameterShape,
    pub policy_planes: CandidatePolicyPlanes,
    pub provenance: Provenance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallableCandidateKind {
    MetaFunction,
    FieldFunction,
}

/// Fingerprint input material for the canonical meta instance key.
///
/// Captures the structural argument product shape at candidate-preparation
/// time. Contains **no** source text, **no** normalized dump, and **no**
/// hash. This is input material only — the final key derivation is future work.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalArgProductShapeMaterial {
    pub arity: usize,
    pub unit_positions: Vec<usize>,
    pub atom_kinds: Vec<CanonicalArgAtomKind>,
    pub known_type_values: Vec<Option<TypeValueId>>,
}

impl CanonicalArgProductShapeMaterial {
    pub fn from_arg_product_shape(shape: &ArgProductShape) -> Self {
        Self {
            arity: shape.arity,
            unit_positions: shape
                .raw_args
                .iter()
                .filter_map(|raw_arg| match raw_arg.value_class {
                    RawArgValueClass::NonValue(NonValueArgKind::ProductUnit) => Some(raw_arg.index),
                    _ => None,
                })
                .collect(),
            atom_kinds: shape
                .raw_args
                .iter()
                .map(|raw_arg| match &raw_arg.value_class {
                    RawArgValueClass::UnknownExpression => CanonicalArgAtomKind::ExpressionBarrier,
                    RawArgValueClass::Value => CanonicalArgAtomKind::ResolvedValue,
                    RawArgValueClass::NonValue(NonValueArgKind::CoreTypeProjection) => {
                        CanonicalArgAtomKind::CoreTypeProjection
                    }
                    RawArgValueClass::NonValue(NonValueArgKind::RankObject) => {
                        CanonicalArgAtomKind::RankObject
                    }
                    RawArgValueClass::NonValue(NonValueArgKind::NamespaceObject) => {
                        CanonicalArgAtomKind::NamespaceObject
                    }
                    RawArgValueClass::NonValue(NonValueArgKind::MetaObject) => {
                        CanonicalArgAtomKind::MetaObject
                    }
                    RawArgValueClass::NonValue(NonValueArgKind::PatternObject) => {
                        CanonicalArgAtomKind::PatternObject
                    }
                    RawArgValueClass::NonValue(NonValueArgKind::ProductUnit) => {
                        CanonicalArgAtomKind::ProductUnit
                    }
                    RawArgValueClass::Unsupported { .. } => CanonicalArgAtomKind::Unsupported,
                })
                .collect(),
            known_type_values: shape
                .raw_args
                .iter()
                .map(|raw_arg| raw_arg.known_first_order_type_value)
                .collect(),
        }
    }
}

/// Structural kind of an argument atom at the canonical key boundary.
///
/// Records whether an argument position carries an Expression barrier, a
/// positively classified value, a specific non-value object kind, a Product
/// Unit, or unsupported material. This is structural classification only —
/// it does **not** encode first-order projection values, resolve lookup, or decide semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanonicalArgAtomKind {
    /// Opaque Expression barrier — target not yet resolved.
    ExpressionBarrier,
    /// Positively classified as a value argument.
    ResolvedValue,
    /// Classified as a pure type Object argument.
    CoreTypeProjection,
    /// Classified as a rank object argument.
    RankObject,
    /// Classified as a namespace object argument.
    NamespaceObject,
    /// Classified as a meta object argument.
    MetaObject,
    /// Classified as a pattern object argument.
    PatternObject,
    /// Product Unit (non-value structural position).
    ProductUnit,
    /// Unsupported or unclassifiable material.
    Unsupported,
}

/// Candidate preparation result before formal meta invocation.
///
/// `Applicable` means the candidate passed arity and body-entry checks. It is
/// not a completed invocation result and it
/// does not produce an `InvocationResult`, `MetaExpansionResult`, or
/// `NamespaceDelta`.
///
/// `Deferred` means later pattern/type/policy/meta-invocation machinery must
/// decide. It is not silent success and it does not residualize runtime
/// expressions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CandidatePrepResult {
    Deferred {
        candidate: Box<PreparedCallableCandidate>,
        reason: CandidatePrepDeferredReason,
    },
    Applicable(Box<PreparedCallableCandidate>),
    Diagnostic(Diagnostic),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CandidatePrepDeferredReason {
    ParameterShapeCompatibilityDeferred,
    BodyEntryPolicyMismatch,
}

/// Candidate preparation with declared policy planes.
///
/// This is the only candidate-preparation entry. The canonical spine supplies
/// the callable kind, primitive identity, and body-entry/return-object planes
/// from its own declared facts (the core bootstrap roster or the semantic call
/// entry); no `SymbolPayload` is read here. The callee `SymbolObject` remains
/// identity/visibility material.
#[allow(clippy::too_many_arguments)]
pub fn prepare_meta_callable_candidate_with_declared_planes(
    callee: &SymbolObject,
    callable_kind: CallableCandidateKind,
    callee_primitive: Option<CoreMetaFunction>,
    body_entry_policy: PolicyView,
    return_object_policy: PolicyView,
    arg_product_shape: ArgProductShape,
    parameter_shape: ParameterShape,
    context: CandidatePreparationContext,
) -> CandidatePrepResult {
    let policy_planes = CandidatePolicyPlanes {
        lookup_env: context.lookup_env,
        symbol_policy_view: callee.policy_view.clone(),
        demanded_execution: context.demanded_execution,
        body_entry_policy,
        return_object_policy,
    };
    let candidate = PreparedCallableCandidate {
        callee_symbol_id: callee.id,
        callee_name: callee.name.clone(),
        callee_primitive,
        callable_kind,
        arg_product_shape,
        parameter_shape,
        policy_planes,
        provenance: context.provenance,
    };

    let Some(expected_arity) = candidate.parameter_shape.expected_arity else {
        return CandidatePrepResult::Deferred {
            candidate: Box::new(candidate),
            reason: CandidatePrepDeferredReason::ParameterShapeCompatibilityDeferred,
        };
    };
    if expected_arity != candidate.arg_product_shape.arity {
        return CandidatePrepResult::Diagnostic(
            Diagnostic::hard_error(
                format!(
                    "candidate preparation arity mismatch: expected {expected_arity}, got {}",
                    candidate.arg_product_shape.arity
                ),
                Some(candidate.parameter_shape.provenance.clone()),
            )
            .with_symbol_context(candidate.callee_symbol_id),
        );
    }
    for (index, requirement) in candidate
        .parameter_shape
        .expected_arg_kinds
        .iter()
        .enumerate()
    {
        let raw_arg = match candidate.arg_product_shape.raw_args.get(index) {
            Some(arg) => arg,
            None => break,
        };
        match requirement {
            ParameterArgRequirement::CoreTypeProjection => {
                if !matches!(
                    raw_arg.value_class,
                    RawArgValueClass::NonValue(NonValueArgKind::CoreTypeProjection)
                ) {
                    let got = format!("{:?}", raw_arg.value_class);
                    return CandidatePrepResult::Diagnostic(
                        Diagnostic::hard_error(
                            format!(
                                "candidate preparation argument kind mismatch at position {index}: expected CoreTypeProjection argument, got {got}"
                            ),
                            Some(raw_arg.provenance.clone()),
                        )
                        .with_symbol_context(candidate.callee_symbol_id),
                    );
                }
            }
            ParameterArgRequirement::Deferred => {}
        }
    }
    if !candidate
        .policy_planes
        .body_entry_allows_demanded_execution()
    {
        return CandidatePrepResult::Deferred {
            candidate: Box::new(candidate),
            reason: CandidatePrepDeferredReason::BodyEntryPolicyMismatch,
        };
    }

    CandidatePrepResult::Applicable(Box::new(candidate))
}
