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
//! `CandidatePreparationInput` is the preferred pipeline carrier. It is not
//! formal invocation. `CanonicalArgProductShapeMaterial` records structural
//! input material. It is not final hash.
//!
//! The current implementation boundary lives in `lang_build::product_shape`,
//! `lang_build::identity`, and `lang_build::meta_candidate`. These are substrate
//! boundaries, not full implementations of the future systems.

use crate::{
    callable_body_allows_execution,
    identity::TypeValueId,
    model::{
        CoreMetaFunction, Diagnostic, ExecutionEnv, FieldObject, MetaFunctionObject, PolicyEnv,
        PolicyMetadata, Provenance, SymbolId, SymbolKind, SymbolObject, SymbolPayload,
    },
    product_shape::{ArgProductShape, NonValueArgKind, RawArgValueClass},
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

    /// Single-parameter signature requiring a type object argument.
    pub fn type_parameter_signature(provenance: Provenance) -> Self {
        Self {
            expected_arity: Some(1),
            expected_arg_kinds: vec![ParameterArgRequirement::TypeObject],
            provenance,
        }
    }
}

/// Per-argument kind requirement for parameter shape validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParameterArgRequirement {
    TypeObject,
    Deferred,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidatePreparationContext {
    pub lookup_env: PolicyEnv,
    pub demanded_execution: ExecutionEnv,
    pub build_identity: CandidateBuildIdentityPlaceholder,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CandidateBuildIdentityPlaceholder {
    pub package_identity_fragment: Option<String>,
    pub mount_identity_fragment: Option<String>,
    pub build_config_fingerprint_fragment: Option<String>,
    pub policy_export_fingerprint_fragment: Option<String>,
}

/// Aggregated input for candidate preparation.
///
/// The callee must already be graph-resolved; the arg product shape must already
/// be extracted from a normalized call site. This struct exists to make the
/// pipeline pass explicit data between stages without letting each stage invent
/// its own partial extraction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidatePreparationInput {
    pub callee: SymbolObject,
    pub arg_product_shape: ArgProductShape,
    pub parameter_shape: ParameterShape,
    pub context: CandidatePreparationContext,
}

impl CandidatePreparationInput {
    pub fn new(
        callee: SymbolObject,
        arg_product_shape: ArgProductShape,
        parameter_shape: ParameterShape,
        context: CandidatePreparationContext,
    ) -> Self {
        Self {
            callee,
            arg_product_shape,
            parameter_shape,
            context,
        }
    }

    pub fn into_parts(
        self,
    ) -> (
        SymbolObject,
        ArgProductShape,
        ParameterShape,
        CandidatePreparationContext,
    ) {
        (
            self.callee,
            self.arg_product_shape,
            self.parameter_shape,
            self.context,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidatePolicyPlanes {
    pub lookup_env: PolicyEnv,
    pub symbol_visibility_policy: PolicyMetadata,
    pub demanded_execution: ExecutionEnv,
    pub body_entry_policy: PolicyMetadata,
    pub return_object_policy: PolicyMetadata,
}

impl CandidatePolicyPlanes {
    pub fn body_entry_allows_demanded_execution(&self) -> bool {
        callable_body_allows_execution(&self.body_entry_policy, self.demanded_execution)
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
    pub canonical_key_seed: CanonicalMetaInstanceKeySeed,
    pub provenance: Provenance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallableCandidateKind {
    MetaFunction,
    FieldFunction,
}

/// Placeholder input contract for the final canonical meta instance key.
///
/// The final key must be derived from the resolved callee `SymbolId`,
/// canonical argument product shape, Expression barrier structure, Unit
/// positions, arity, first-order `TypeValueId` values where known, package
/// identity, mount identity, build/config fingerprint, policy/export-relevant
/// metadata, and provenance/cache key fragments as needed.
///
/// It must not be reduced to source text, a normalized dump, callee name plus
/// arity, or callee name plus `TypeValueId` list only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalMetaInstanceKeySeed {
    pub callee_function_symbol_id: SymbolId,
    /// Remains `None` — reserved for a future serialized-key format.
    /// The authoritative canonical key is computed by
    /// `compute_meta_instance_key(&PreparedCallableCandidate)`.
    pub argument_product_shape_fingerprint_fragment: Option<String>,
    pub argument_product_shape_material: CanonicalArgProductShapeMaterial,
    pub unit_positions: Vec<usize>,
    pub argument_arity: usize,
    pub argument_type_values: Vec<Option<TypeValueId>>,
    pub package_identity_fragment: Option<String>,
    pub mount_identity_fragment: Option<String>,
    pub build_config_fingerprint_fragment: Option<String>,
    pub policy_export_fingerprint_fragment: Option<String>,
    pub provenance: Provenance,
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
                    RawArgValueClass::NonValue(NonValueArgKind::TypeObject) => {
                        CanonicalArgAtomKind::TypeObject
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
/// it does **not** encode type values, resolve lookup, or decide semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanonicalArgAtomKind {
    /// Opaque Expression barrier — target not yet resolved.
    ExpressionBarrier,
    /// Positively classified as a value argument.
    ResolvedValue,
    /// Classified as a type object argument.
    TypeObject,
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
/// `ApplicablePlaceholder` means the candidate passed the current placeholder
/// arity and body-entry checks. It is not a completed invocation result and it
/// does not produce a `MetaReductionResult`, `MetaExpansionResult`, or
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
    ApplicablePlaceholder(Box<PreparedCallableCandidate>),
    Diagnostic(Diagnostic),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CandidatePrepDeferredReason {
    ParameterShapeCompatibilityDeferred,
    BodyEntryPolicyMismatch,
}

pub fn prepare_meta_callable_candidate(
    callee: &SymbolObject,
    arg_product_shape: ArgProductShape,
    parameter_shape: ParameterShape,
    context: CandidatePreparationContext,
) -> CandidatePrepResult {
    let Some((callable_kind, body_entry_policy, return_object_policy)) =
        callable_policy_from_symbol(callee)
    else {
        return CandidatePrepResult::Diagnostic(
            Diagnostic::hard_error(
                "candidate preparation requires a graph-resolved callable SymbolObject",
                Some(callee.provenance.clone()),
            )
            .with_symbol_context(callee.id),
        );
    };

    let policy_planes = CandidatePolicyPlanes {
        lookup_env: context.lookup_env,
        symbol_visibility_policy: callee.policy_metadata.clone(),
        demanded_execution: context.demanded_execution,
        body_entry_policy,
        return_object_policy,
    };
    let unit_positions = arg_product_shape
        .raw_args
        .iter()
        .filter_map(|raw_arg| match raw_arg.value_class {
            RawArgValueClass::NonValue(NonValueArgKind::ProductUnit) => Some(raw_arg.index),
            _ => None,
        })
        .collect();
    let canonical_key_seed = CanonicalMetaInstanceKeySeed {
        callee_function_symbol_id: callee.id,
        argument_product_shape_fingerprint_fragment: None,
        argument_product_shape_material: CanonicalArgProductShapeMaterial::from_arg_product_shape(
            &arg_product_shape,
        ),
        unit_positions,
        argument_arity: arg_product_shape.arity,
        argument_type_values: arg_product_shape
            .raw_args
            .iter()
            .map(|raw_arg| raw_arg.known_first_order_type_value)
            .collect(),
        package_identity_fragment: context.build_identity.package_identity_fragment.clone(),
        mount_identity_fragment: context.build_identity.mount_identity_fragment.clone(),
        build_config_fingerprint_fragment: context
            .build_identity
            .build_config_fingerprint_fragment
            .clone(),
        policy_export_fingerprint_fragment: context
            .build_identity
            .policy_export_fingerprint_fragment
            .clone(),
        provenance: context.provenance.clone(),
    };
    let callee_primitive = match &callee.payload {
        SymbolPayload::MetaFunction(mf) => Some(mf.primitive),
        _ => None,
    };
    let candidate = PreparedCallableCandidate {
        callee_symbol_id: callee.id,
        callee_name: callee.name.clone(),
        callee_primitive,
        callable_kind,
        arg_product_shape,
        parameter_shape,
        policy_planes,
        canonical_key_seed,
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
            ParameterArgRequirement::TypeObject => {
                if !matches!(
                    raw_arg.value_class,
                    RawArgValueClass::NonValue(NonValueArgKind::TypeObject)
                ) {
                    let got = format!("{:?}", raw_arg.value_class);
                    return CandidatePrepResult::Diagnostic(
                        Diagnostic::hard_error(
                            format!(
                                "candidate preparation argument kind mismatch at position {index}: expected TypeObject argument, got {got}"
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

    CandidatePrepResult::ApplicablePlaceholder(Box::new(candidate))
}

/// Wrapper that accepts a `CandidatePreparationInput` and delegates to the
/// existing `prepare_meta_callable_candidate` logic.
///
/// This is the preferred pipeline entry point. Future stages should construct
/// a `CandidatePreparationInput` rather than assembling scattered parameters.
/// It does **not** execute meta functions or install `NamespaceDelta`.
pub fn prepare_meta_callable_candidate_from_input(
    input: CandidatePreparationInput,
) -> CandidatePrepResult {
    let (callee, arg_product_shape, parameter_shape, context) = input.into_parts();
    prepare_meta_callable_candidate(&callee, arg_product_shape, parameter_shape, context)
}

fn callable_policy_from_symbol(
    callee: &SymbolObject,
) -> Option<(CallableCandidateKind, PolicyMetadata, PolicyMetadata)> {
    match &callee.payload {
        SymbolPayload::MetaFunction(MetaFunctionObject {
            body_entry_policy,
            return_object_policy,
            ..
        }) if callee.kind == SymbolKind::MetaFunction => Some((
            CallableCandidateKind::MetaFunction,
            body_entry_policy.clone(),
            return_object_policy.clone(),
        )),
        SymbolPayload::FieldFunction(FieldObject {
            callable_policy, ..
        }) if callee.kind == SymbolKind::FieldFunction => Some((
            CallableCandidateKind::FieldFunction,
            callable_policy.body_entry_policy.clone(),
            callable_policy.return_object_policy.clone(),
        )),
        _ => None,
    }
}
