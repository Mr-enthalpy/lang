use crate::{
    callable_body_allows_execution,
    identity::TypeValueId,
    model::{
        Diagnostic, ExecutionEnv, FieldObject, MetaFunctionObject, PolicyEnv, PolicyMetadata,
        Provenance, SymbolId, SymbolKind, SymbolObject, SymbolPayload,
    },
    product_shape::{ArgProductShape, NonValueArgKind, RawArgValueClass},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParameterShape {
    pub expected_arity: Option<usize>,
    pub provenance: Provenance,
}

impl ParameterShape {
    pub fn deferred(provenance: Provenance) -> Self {
        Self {
            expected_arity: None,
            provenance,
        }
    }

    pub fn exact_arity(expected_arity: usize, provenance: Provenance) -> Self {
        Self {
            expected_arity: Some(expected_arity),
            provenance,
        }
    }
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
    pub argument_product_shape_fingerprint_fragment: Option<String>,
    pub unit_positions: Vec<usize>,
    pub argument_arity: usize,
    pub argument_type_values: Vec<Option<TypeValueId>>,
    pub package_identity_fragment: Option<String>,
    pub mount_identity_fragment: Option<String>,
    pub build_config_fingerprint_fragment: Option<String>,
    pub policy_export_fingerprint_fragment: Option<String>,
    pub provenance: Provenance,
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
    let candidate = PreparedCallableCandidate {
        callee_symbol_id: callee.id,
        callee_name: callee.name.clone(),
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
