//! Formal meta invocation boundary.
//!
//! Consumes a `PreparedCallableCandidate` and dispatches to the appropriate
//! primitive invocation. This is a **pure** step — it produces an
//! `MetaInvocationValue` but does **not** install `NamespaceDelta`, bind
//! declared symbols, or mutate the namespace graph.
//!
//! ## Separation of concerns
//!
//! ```text
//! CandidatePrepResult::ApplicablePlaceholder
//!   → MetaInvocationInput
//!   → invoke_meta_callable
//!   → MetaInvocationValue  (pure, no graph mutation)
//!
//! MetaInvocationValue
//!   → bind_meta_invocation_value_result (meta.rs)
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

/// Result of formal meta invocation.
#[derive(Clone, Debug)]
pub enum MetaInvocationResult {
    Value(MetaInvocationValue),
    Diagnostic(Diagnostic),
}

/// Target of a forwarded invocation value.
///
/// `TypeValueProjection` is the current legacy path — forwarded values only
/// carry a type-value identity. Future variants will carry `SymbolId`,
/// `ValueObject`, and `ConstructionInstance` targets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetaValueTarget {
    TypeValueProjection(TypeValueId),
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
    pub target: MetaValueTarget,
    pub return_view: ReturnViewShape,
    pub provenance: Provenance,
}

/// Generated construction value — the call returns a new construction value
/// whose external identity is shielded by callee + canonical args + build
/// identity (`r = t`). Reserved for future generative type constructors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedConstructionValue {
    pub construction_instance_id: ConstructionInstanceId,
    pub identity_material: ConstructionIdentityMaterial,
    pub return_view: ReturnViewShape,
    pub provenance: Provenance,
}

/// Stable identity for a generated construction value.
///
/// Produced by `compute_construction_instance_id`. Distinct from `SymbolId`
/// and `TypeValueId` — two different symbols may carry the same construction
/// instance identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConstructionInstanceId(pub u64);

impl ConstructionInstanceId {
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

/// Return-slot semantics for the meta callable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReturnSlotSemantics {
    /// `r === arg` — forwarded existing value.
    Forward,
    /// `r = arg` — generated construction value.
    Generate,
}

/// Material that determines a generated construction value's identity.
///
/// Same callee + same canonical args + same return-slot semantics + same
/// build/policy identity → same `ConstructionInstanceId`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstructionIdentityMaterial {
    pub callee_symbol_id: SymbolId,
    pub canonical_args: CanonicalArgProductShapeMaterial,
    pub return_slot_semantics: ReturnSlotSemantics,
    pub build_identity_fragment: Option<String>,
    pub policy_export_fingerprint_fragment: Option<String>,
    pub provenance: Provenance,
}

/// Compute a stable `ConstructionInstanceId` from identity material.
///
/// Uses a placeholder FNV-1a hash. Must be replaced with a stable
/// construction-instance key derivation when cross-build identity is
/// implemented.
pub fn compute_construction_instance_id(
    material: &ConstructionIdentityMaterial,
) -> ConstructionInstanceId {
    use crate::fingerprint::Fnv1a64;
    let mut h = Fnv1a64::new();
    h.write_str_field("v08:construction");
    h.write_field(&material.callee_symbol_id.0.to_le_bytes());
    h.write_field(&(material.canonical_args.arity as u64).to_le_bytes());
    h.write_field(&(material.canonical_args.unit_positions.len() as u64).to_le_bytes());
    for pos in &material.canonical_args.unit_positions {
        h.write_field(&(*pos as u64).to_le_bytes());
    }
    for kind in &material.canonical_args.atom_kinds {
        h.write_field(&[crate::meta_key::atom_kind_discriminant(kind)]);
    }
    for tv in &material.canonical_args.known_type_values {
        match tv {
            None => h.write_field(&[0u8]),
            Some(tv) => {
                h.write_field(&[1u8]);
                h.write_field(&tv.0.to_le_bytes());
            }
        }
    }
    let sem = match material.return_slot_semantics {
        ReturnSlotSemantics::Forward => 0u8,
        ReturnSlotSemantics::Generate => 1u8,
    };
    h.write_field(&[sem]);
    if let Some(ref s) = material.build_identity_fragment {
        h.write_str_field(s);
    }
    if let Some(ref s) = material.policy_export_fingerprint_fragment {
        h.write_str_field(s);
    }
    let raw = u64::from_str_radix(&h.finish_hex(), 16)
        .expect("Fnv1a64::finish_hex must produce a valid u64 hex string");
    ConstructionInstanceId(raw)
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
        crate::model::CoreMetaFunction::UnaryConstructionPrototype => {
            invoke_unary_construction_prototype(&input)
        }
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
/// invocation value. The cache stores only `MetaInvocationValue` — no
/// `NamespaceDelta`.
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
        target: MetaValueTarget::TypeValueProjection(type_value_id),
        return_view: ReturnViewShape::Leaf,
        provenance: input.provenance.clone(),
    }))
}

fn invoke_unary_construction_prototype(input: &MetaInvocationInput) -> MetaInvocationResult {
    let candidate = &input.candidate;
    let mat = &candidate.canonical_key_seed.argument_product_shape_material;

    if mat.arity != 1 {
        return MetaInvocationResult::Diagnostic(
            Diagnostic::hard_error(
                format!(
                    "UnaryConstructionPrototype: expected exactly 1 type argument, got {}",
                    mat.arity
                ),
                Some(input.provenance.clone()),
            )
            .with_symbol_context(candidate.callee_symbol_id),
        );
    }

    let _type_value_id = match mat.known_type_values.get(0).and_then(|tv| *tv) {
        Some(tv) => tv,
        None => {
            return MetaInvocationResult::Diagnostic(
                Diagnostic::hard_error(
                    "UnaryConstructionPrototype: argument is not a classified type object with a TypeValueId",
                    Some(input.provenance.clone()),
                )
                .with_symbol_context(candidate.callee_symbol_id),
            );
        }
    };

    let identity_material = ConstructionIdentityMaterial {
        callee_symbol_id: candidate.callee_symbol_id,
        canonical_args: mat.clone(),
        return_slot_semantics: ReturnSlotSemantics::Generate,
        build_identity_fragment: candidate
            .canonical_key_seed
            .package_identity_fragment
            .clone(),
        policy_export_fingerprint_fragment: candidate
            .canonical_key_seed
            .policy_export_fingerprint_fragment
            .clone(),
        provenance: input.provenance.clone(),
    };
    let construction_instance_id = compute_construction_instance_id(&identity_material);

    MetaInvocationResult::Value(MetaInvocationValue::GeneratedConstructionValue(
        GeneratedConstructionValue {
            construction_instance_id,
            identity_material,
            return_view: ReturnViewShape::Leaf,
            provenance: input.provenance.clone(),
        },
    ))
}
