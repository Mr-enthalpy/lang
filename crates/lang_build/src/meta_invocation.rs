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
    meta_cache::MetaInstanceCache,
    meta_candidate::{CanonicalArgProductShapeMaterial, PreparedCallableCandidate},
    meta_key::{compute_meta_instance_key, MetaInstanceKey},
    model::{Diagnostic, Provenance, SymbolId},
};

/// Input for formal meta invocation.
///
/// The candidate must already have passed `prepare_meta_callable_candidate`.
/// The primitive is read from `candidate.callee_primitive` — callers do not
/// pass it separately, preventing primitive-vs-candidate mismatch.
#[derive(Clone, Debug)]
pub struct MetaInvocationInput {
    pub candidate: PreparedCallableCandidate,
    pub provenance: Provenance,
}

impl MetaInvocationInput {
    pub fn new(candidate: PreparedCallableCandidate, provenance: Provenance) -> Self {
        Self {
            candidate,
            provenance,
        }
    }

    pub fn compute_key(&self) -> MetaInstanceKey {
        compute_meta_instance_key(&self.candidate)
    }
}

/// Legacy placeholder reduction result. Use `MetaInvocationValue::ForwardedValue`
/// for forwarding proofs. `MetaInvocationValue::GeneratedConstructionValue` is
/// the future replacement for generative type-to-type meta construction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetaReductionResult {
    TypeValue(TypeValueId),
}

/// Result of formal meta invocation.
#[derive(Clone, Debug)]
pub enum MetaInvocationResult {
    Value(MetaInvocationValue),
    Diagnostic(Diagnostic),
}

/// Invocation value produced by formal meta invocation.
///
/// `ForwardedValue` represents `r === arg` semantics (forwarding proof).
/// `GeneratedConstructionValue` represents `r = t` semantics (generative
/// construction). Both are future slots — currently only `ForwardedValue`
/// is produced for `IdentityType`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetaInvocationValue {
    ForwardedValue(ForwardedValue),
    GeneratedConstructionValue(GeneratedConstructionValue),
}

/// Forwarded existing value — the call returns the same value that was passed
/// as argument (`r === arg`). Used by `IdentityType` as forwarding proof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForwardedValue {
    pub target: TypeValueId,
    pub return_view: ReturnViewShape,
    pub provenance: Provenance,
}

/// Generated construction value — the call returns a new construction value
/// whose external identity is shielded by callee + canonical args + build
/// identity (`r = t`). Reserved for future generative type constructors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedConstructionValue {
    pub callee_symbol_id: SymbolId,
    pub canonical_args: CanonicalArgProductShapeMaterial,
    pub return_view: ReturnViewShape,
    pub provenance: Provenance,
}

/// Return value shape — whether the invocation value exposes a leaf or product
/// extraction view. `Leaf` means `v? == v`; `Product` means `v?` splits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReturnViewShape {
    Leaf,
    Product { arity: usize },
}

/// Invoke a prepared callable candidate through the formal meta invocation
/// boundary.
///
/// Reads `callee_primitive` from the candidate itself. The reduction is pure.
pub fn invoke_meta_callable(input: MetaInvocationInput) -> MetaInvocationResult {
    let Some(primitive) = input.candidate.callee_primitive else {
        return MetaInvocationResult::Diagnostic(
            Diagnostic::hard_error(
                format!(
                    "meta invocation: candidate `{}` has no callee primitive",
                    input.candidate.callee_name
                ),
                Some(input.provenance),
            )
            .with_symbol_context(input.candidate.callee_symbol_id),
        );
    };

    match primitive {
        crate::model::CoreMetaFunction::IdentityType => invoke_identity_type(&input),
        crate::model::CoreMetaFunction::Struct => MetaInvocationResult::Diagnostic(
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
                    primitive
                ),
                Some(input.provenance),
            )
            .with_symbol_context(input.candidate.callee_symbol_id),
        ),
    }
}

/// Cached variant: look up the key in the cache before invoking.
///
/// On cache miss, invokes and inserts the result. On hit, returns the cached
/// reduction. The cache stores only `MetaReductionResult` — no `NamespaceDelta`.
pub fn invoke_meta_callable_cached(
    input: MetaInvocationInput,
    cache: &mut MetaInstanceCache,
) -> MetaInvocationResult {
    // Validate primitive before cache lookup — prevents a manually-inserted
    // cache entry for a no-primitive candidate from bypassing validation.
    if input.candidate.callee_primitive.is_none() {
        return MetaInvocationResult::Diagnostic(
            Diagnostic::hard_error(
                format!(
                    "meta invocation (cached): candidate `{}` has no callee primitive",
                    input.candidate.callee_name
                ),
                Some(input.provenance),
            )
            .with_symbol_context(input.candidate.callee_symbol_id),
        );
    }
    let key = input.compute_key();
    if let Some(cached) = cache.lookup(&key) {
        return MetaInvocationResult::Value(cached.result.clone());
    }
    let result = invoke_meta_callable(input);
    if let MetaInvocationResult::Value(ref val) = result {
        cache.insert(
            key,
            val.clone(),
            Provenance::new("cached meta invocation result"),
        );
    }
    result
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

    MetaInvocationResult::Value(MetaInvocationValue::ForwardedValue(ForwardedValue {
        target: type_value_id,
        return_view: ReturnViewShape::Leaf,
        provenance: input.provenance.clone(),
    }))
}
