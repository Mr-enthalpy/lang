//! Formal meta invocation boundary.
//!
//! Consumes a `PreparedCallableCandidate` and dispatches to the appropriate
//! primitive invocation. This is a **pure reduction** step — it produces a
//! `MetaReductionResult` but does **not** install `NamespaceDelta`, bind
//! declared symbols, or mutate the namespace graph.
//!
//! ## Separation of concerns
//!
//! ```text
//! CandidatePrepResult::ApplicablePlaceholder
//!   → MetaInvocationInput
//!   → invoke_meta_callable
//!   → MetaReductionResult  (pure, no graph mutation)
//!
//! MetaReductionResult
//!   → bind_meta_type_value_result (meta.rs)
//!   → MetaExpansionResult  (declaration binding, with NamespaceDelta)
//! ```
//!
//! ## Relation to v0.8 shortcut
//!
//! Under the current v0.8 `temporary_direct_callable_shortcut`, the candidate's
//! callee is treated as the callable entry directly. Future:
//!
//! ```text
//! target value → target type → `()` call entry → implicit self + explicit Product
//! ```
//!
//! The implicit `self` belongs to the invocation frame, **not** to
//! `ProductObject` / `ArgProductShape` / `RawArgShape`.

use crate::{
    identity::TypeValueId,
    meta_candidate::PreparedCallableCandidate,
    model::{CoreMetaFunction, Diagnostic, Provenance},
};

/// Input for formal meta invocation.
///
/// The candidate must already have passed `prepare_meta_callable_candidate`.
/// The `callee_primitive` identifies which primitive implementation to invoke.
///
/// Future: `MetaInvocationInput` will also carry an `InvocationFrame` with
/// implicit `self` when the `()` call-entry model is implemented.
#[derive(Clone, Debug)]
pub struct MetaInvocationInput {
    pub candidate: PreparedCallableCandidate,
    pub callee_primitive: CoreMetaFunction,
    pub provenance: Provenance,
}

impl MetaInvocationInput {
    pub fn new(
        candidate: PreparedCallableCandidate,
        callee_primitive: CoreMetaFunction,
        provenance: Provenance,
    ) -> Self {
        Self {
            candidate,
            callee_primitive,
            provenance,
        }
    }
}

/// Pure reduction result of formal meta invocation.
///
/// Does **not** carry `NamespaceDelta`, declared symbols, or generated objects.
/// This is the minimal output of a primitive meta callable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetaReductionResult {
    /// The invocation returned a type value (e.g. `IdentityType(uint8)`).
    TypeValue(TypeValueId),
}

/// Result of formal meta invocation.
#[derive(Clone, Debug)]
pub enum MetaInvocationResult {
    Reduction(MetaReductionResult),
    Diagnostic(Diagnostic),
}

/// Invoke a prepared callable candidate through the formal meta invocation
/// boundary.
///
/// Dispatches based on `callee_primitive`. Currently only `IdentityType`
/// is supported. The reduction is pure — no graph mutation, no
/// `NamespaceDelta` installation, no declared symbol creation.
pub fn invoke_meta_callable(input: MetaInvocationInput) -> MetaInvocationResult {
    match input.callee_primitive {
        CoreMetaFunction::IdentityType => invoke_identity_type(&input),
        CoreMetaFunction::Struct => MetaInvocationResult::Diagnostic(
            Diagnostic::hard_error(
                "meta invocation: struct is not yet callable through formal invocation",
                Some(input.provenance),
            )
            .with_symbol_context(input.candidate.callee_symbol_id),
        ),
        _ => MetaInvocationResult::Diagnostic(
            Diagnostic::hard_error(
                format!(
                    "meta invocation: primitive {:?} is not callable through formal invocation",
                    input.callee_primitive
                ),
                Some(input.provenance),
            )
            .with_symbol_context(input.candidate.callee_symbol_id),
        ),
    }
}

fn invoke_identity_type(input: &MetaInvocationInput) -> MetaInvocationResult {
    let candidate = &input.candidate;
    let mat = &candidate.canonical_key_seed.argument_product_shape_material;

    if mat.arity != 1 {
        return MetaInvocationResult::Diagnostic(
            Diagnostic::hard_error(
                format!(
                    "IdentityType: expected exactly 1 type argument, got {}",
                    mat.arity
                ),
                Some(input.provenance.clone()),
            )
            .with_symbol_context(candidate.callee_symbol_id),
        );
    }

    let type_value_id = match mat.known_type_values.get(0).and_then(|tv| *tv) {
        Some(tv) => tv,
        None => {
            return MetaInvocationResult::Diagnostic(
                Diagnostic::hard_error(
                    "IdentityType: argument is not a classified type object with a TypeValueId",
                    Some(input.provenance.clone()),
                )
                .with_symbol_context(candidate.callee_symbol_id),
            );
        }
    };

    MetaInvocationResult::Reduction(MetaReductionResult::TypeValue(type_value_id))
}
