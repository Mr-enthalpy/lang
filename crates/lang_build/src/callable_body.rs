//! Builtin implementation leaves selected by ordinary invocation.
//!
//! Consumes a fixed builtin implementation and its prepared argument material.
//! This step is graph-installation-free and binding-free:
//! it produces a `BuiltinBodyMaterial` but does **not** install
//! `NamespaceDelta`, bind declared symbols, or mutate the namespace graph. It
//! does not allocate graph or Pattern-relation state.
//!
//! ## Separation of concerns
//!
//! ```text
//! CandidatePrepResult::Applicable
//!   → BuiltinBodyInput
//!   → invoke_selected_builtin_body
//!   → BuiltinBodyResult::Material(BuiltinBodyMaterial)
//!     (no semantic result, graph installation, or binding)
//!
//! BuiltinBodyMaterial
//!   → ordinary invocation's declared-result consumer
//! ```
//!
//! Production invocation reaches this builtin leaf only after ordinary
//! value → complete type → associated `()` resolution has selected a call-entry
//! semantic value. The implicit `self` belongs to that invocation frame, never
//! to `ProductSyntaxMaterial` / `ArgProductShape` / `RawArgShape`.

use crate::{
    candidate_preparation::{CanonicalArgProductShapeMaterial, PreparedCallableCandidate},
    model::{Diagnostic, Provenance},
};

/// Input for a selected builtin implementation leaf.
///
/// The candidate must already have passed candidate preparation
/// (`prepare_callable_candidate_with_declared_planes`).
/// The implementation is the leaf retained by ordinary candidate preparation;
/// this carrier supplies neither Ready nor a common transaction witness.
#[derive(Clone, Debug)]
pub struct BuiltinBodyInput {
    pub candidate: PreparedCallableCandidate,
    pub implementation: crate::BuiltinCallableImpl,
    pub provenance: Provenance,
}

impl BuiltinBodyInput {
    pub fn new(
        candidate: PreparedCallableCandidate,
        implementation: crate::BuiltinCallableImpl,
        provenance: Provenance,
    ) -> Self {
        Self {
            candidate,
            implementation,
            provenance,
        }
    }
}

/// Private material produced only by selected builtin implementation leaves.
/// Source bodies have no route to this carrier; their future common E consumer
/// must deliver ordinary semantic completion.
///
/// Builtin leaves return proof-relevant material to the ordinary result
/// consumer. Structural interpretation forms complete types independently.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum BuiltinBodyMaterial {
    IdentityType(IdentityTypeMaterial),
    StructHelpers(IdentityTypeMaterial),
}

/// Implementation material handed to the ordinary declared-result consumer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuiltinBodyResult {
    Material(BuiltinBodyMaterial),
    Diagnostic(Diagnostic),
}

/// Existing type value and observation proven by the `IdentityType` primitive.
///
/// The target carries the forwarded TypeValue directly. Reaching that value
/// through a graph Symbol does not make the carrier Symbol part of the
/// invocation result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdentityTypeMaterial {
    /// The represented value itself. Even when evaluation reached it through a
    /// source name, the carrier Symbol is not part of this result identity.
    pub type_value: crate::TypeValueId,
    /// The type observation carried by this result. Semantic equality
    /// consumes this, never the bare `type_value` projection.
    pub complete_type_observation: crate::CanonicalValueAddr,
    pub provenance: Provenance,
}

pub(crate) fn invoke_selected_builtin_body(input: BuiltinBodyInput) -> BuiltinBodyResult {
    match input.implementation {
        crate::model::BuiltinCallableImpl::IdentityType => invoke_identity_type(&input),
        crate::model::BuiltinCallableImpl::Struct => invoke_struct_helpers(&input),
        primitive => BuiltinBodyResult::Diagnostic(
            Diagnostic::hard_error(
                format!(
                    "selected builtin implementation {:?} has no connected body consumer",
                    primitive
                ),
                Some(input.provenance),
            )
            .with_symbol_context(input.candidate.callee_symbol_id),
        ),
    }
}

fn invoke_identity_type(input: &BuiltinBodyInput) -> BuiltinBodyResult {
    let candidate = &input.candidate;
    let mat =
        CanonicalArgProductShapeMaterial::from_arg_product_shape(&candidate.arg_product_shape);

    if mat.arity != 1 {
        return BuiltinBodyResult::Diagnostic(
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

    let type_value = match mat.known_type_values.first().and_then(|value| *value) {
        Some(value) => value,
        None => {
            return BuiltinBodyResult::Diagnostic(
                Diagnostic::hard_error(
                    "IdentityType: argument is not a classified complete type value",
                    Some(input.provenance.clone()),
                )
                .with_symbol_context(candidate.callee_symbol_id),
            );
        }
    };
    let Some(complete_type_observation) = candidate
        .arg_product_shape
        .raw_args
        .first()
        .and_then(|raw| raw.known_complete_type_observation)
    else {
        return BuiltinBodyResult::Diagnostic(
            Diagnostic::hard_error(
                "IdentityType requires an exact complete type snapshot",
                Some(input.provenance.clone()),
            )
            .with_symbol_context(candidate.callee_symbol_id),
        );
    };

    BuiltinBodyResult::Material(BuiltinBodyMaterial::IdentityType(IdentityTypeMaterial {
        type_value,
        complete_type_observation,
        provenance: input.provenance.clone(),
    }))
}

fn invoke_struct_helpers(input: &BuiltinBodyInput) -> BuiltinBodyResult {
    match invoke_identity_type(input) {
        BuiltinBodyResult::Material(BuiltinBodyMaterial::IdentityType(value)) => {
            BuiltinBodyResult::Material(BuiltinBodyMaterial::StructHelpers(value))
        }
        other => other,
    }
}
