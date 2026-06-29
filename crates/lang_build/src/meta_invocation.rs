//! Formal meta invocation boundary.
//!
//! Consumes a `PreparedCallableCandidate` and dispatches to the appropriate
//! primitive invocation. This is a **pure** step — it produces a
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

use std::collections::BTreeSet;

use lang_syntax::{NormExpr, NormProductElem};

use crate::{
    meta_cache::MetaInstanceCache,
    meta_candidate::{CanonicalArgProductShapeMaterial, PreparedCallableCandidate},
    meta_key::{compute_meta_instance_key, MetaInstanceKey},
    model::{Diagnostic, Provenance, SymbolId},
    product_shape::{NonValueArgKind, ProductAtom, RawArgValueClass},
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
/// `TypeSymbol` carries the forwarded type's `SymbolId` as its primary
/// identity. `TypeValueId` projection is derived from the symbol identity
/// (via `type_value_projection_from_type_symbol`), never used as
/// a binding lookup source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetaValueTarget {
    TypeSymbol(SymbolId),
}

/// Invocation value produced by formal meta invocation.
///
/// `ForwardedValue` is produced by `IdentityType` (`r === arg`).
/// `GeneratedConstructionValue` is produced by `UnaryConstructionPrototype`
/// (`r = t`, where `t` is computed from the argument object).
/// `GeneratedTypeDefinitionValue` is produced by `struct` and is materialized
/// only by the binding layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetaInvocationValue {
    ForwardedValue(ForwardedValue),
    GeneratedConstructionValue(GeneratedConstructionValue),
    GeneratedTypeDefinitionValue(GeneratedTypeDefinitionValue),
}

/// Forwarded existing value — the call returns the same value that was passed
/// as argument (`r === arg`). Used by `IdentityType` as forwarding proof.
///
/// The `target` carries the forwarded type's `SymbolId`. `TypeValueId`
/// projection is implicitly derived from the symbol identity.
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

/// Generated type-definition value produced by formal `struct` invocation.
///
/// This is pure invocation output. The declared type symbol, associated
/// namespace, and field projections are binding materialization artifacts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedTypeDefinitionValue {
    pub type_definition_id: TypeDefinitionInstanceId,
    pub identity_material: TypeDefinitionIdentityMaterial,
    pub fields: Vec<GeneratedFieldDefinition>,
    pub return_view: ReturnViewShape,
    pub provenance: Provenance,
}

/// Deterministic build-local construction identity placeholder.
///
/// Produced by `compute_construction_instance_id`. Distinct from `SymbolId`
/// and the type-value projection — two different symbols may carry the same
/// construction instance identity. This is a placeholder; a stable
/// will use a different key derivation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConstructionInstanceId(pub u64);

impl ConstructionInstanceId {
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

/// Deterministic build-local generated type-definition identity placeholder.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeDefinitionInstanceId(pub u64);

impl TypeDefinitionInstanceId {
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
///
/// `provenance` is non-identity diagnostic material. It does not participate
/// in `compute_construction_instance_id` and must not be treated as part of
/// construction identity equality.
#[derive(Clone, Debug)]
pub struct ConstructionIdentityMaterial {
    pub callee_symbol_id: SymbolId,
    pub canonical_args: CanonicalArgProductShapeMaterial,
    pub return_slot_semantics: ReturnSlotSemantics,
    pub build_identity_fragment: Option<String>,
    pub policy_export_fingerprint_fragment: Option<String>,
    pub provenance: Provenance,
}

impl PartialEq for ConstructionIdentityMaterial {
    fn eq(&self, other: &Self) -> bool {
        self.callee_symbol_id == other.callee_symbol_id
            && self.canonical_args == other.canonical_args
            && self.return_slot_semantics == other.return_slot_semantics
            && self.build_identity_fragment == other.build_identity_fragment
            && self.policy_export_fingerprint_fragment == other.policy_export_fingerprint_fragment
    }
}

impl Eq for ConstructionIdentityMaterial {}

/// Material that determines a generated type definition's identity.
///
/// `provenance` is diagnostic material and is excluded from equality and
/// identity computation.
#[derive(Clone, Debug)]
pub struct TypeDefinitionIdentityMaterial {
    pub callee_symbol_id: SymbolId,
    pub canonical_args: CanonicalArgProductShapeMaterial,
    pub field_signature_material: Vec<FieldSignatureMaterial>,
    pub return_slot_semantics: ReturnSlotSemantics,
    pub build_identity_fragment: Option<String>,
    pub policy_export_fingerprint_fragment: Option<String>,
    pub provenance: Provenance,
}

impl PartialEq for TypeDefinitionIdentityMaterial {
    fn eq(&self, other: &Self) -> bool {
        self.callee_symbol_id == other.callee_symbol_id
            && self.canonical_args == other.canonical_args
            && self.field_signature_material == other.field_signature_material
            && self.return_slot_semantics == other.return_slot_semantics
            && self.build_identity_fragment == other.build_identity_fragment
            && self.policy_export_fingerprint_fragment == other.policy_export_fingerprint_fragment
    }
}

impl Eq for TypeDefinitionIdentityMaterial {}

#[derive(Clone, Debug)]
pub struct FieldSignatureMaterial {
    pub field_name: String,
    pub field_type_symbol_id: SymbolId,
    pub field_index: usize,
    pub provenance: Provenance,
}

impl PartialEq for FieldSignatureMaterial {
    fn eq(&self, other: &Self) -> bool {
        self.field_name == other.field_name
            && self.field_type_symbol_id == other.field_type_symbol_id
            && self.field_index == other.field_index
    }
}

impl Eq for FieldSignatureMaterial {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedFieldDefinition {
    pub name: String,
    pub type_symbol_id: SymbolId,
    pub index: usize,
    pub provenance: Provenance,
}

/// Compute a deterministic build-local `ConstructionInstanceId` from identity
/// material.
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
    for sym in &material.canonical_args.known_type_symbols {
        match sym {
            None => h.write_field(&[0u8]),
            Some(s) => {
                h.write_field(&[1u8]);
                h.write_field(&s.0.to_le_bytes());
            }
        }
    }
    let sem = match material.return_slot_semantics {
        ReturnSlotSemantics::Forward => 0u8,
        ReturnSlotSemantics::Generate => 1u8,
    };
    h.write_field(&[sem]);
    match &material.build_identity_fragment {
        None => h.write_field(&[0u8]),
        Some(s) => {
            h.write_field(&[1u8]);
            h.write_str_field(s);
        }
    }
    match &material.policy_export_fingerprint_fragment {
        None => h.write_field(&[0u8]),
        Some(s) => {
            h.write_field(&[1u8]);
            h.write_str_field(s);
        }
    }
    let raw = u64::from_str_radix(&h.finish_hex(), 16)
        .expect("Fnv1a64::finish_hex must produce a valid u64 hex string");
    // Non-zero invariant: 0 is reserved as an invalid sentinel.
    ConstructionInstanceId(if raw == 0 { 1 } else { raw })
}

pub fn compute_type_definition_instance_id(
    material: &TypeDefinitionIdentityMaterial,
) -> TypeDefinitionInstanceId {
    use crate::fingerprint::Fnv1a64;
    let mut h = Fnv1a64::new();
    h.write_str_field("v08:type-definition");
    h.write_field(&material.callee_symbol_id.0.to_le_bytes());
    h.write_field(&(material.canonical_args.arity as u64).to_le_bytes());
    h.write_field(&(material.canonical_args.unit_positions.len() as u64).to_le_bytes());
    for pos in &material.canonical_args.unit_positions {
        h.write_field(&(*pos as u64).to_le_bytes());
    }
    h.write_field(&(material.canonical_args.atom_kinds.len() as u64).to_le_bytes());
    for kind in &material.canonical_args.atom_kinds {
        h.write_field(&[crate::meta_key::atom_kind_discriminant(kind)]);
    }
    h.write_field(&(material.canonical_args.known_type_symbols.len() as u64).to_le_bytes());
    for sym in &material.canonical_args.known_type_symbols {
        match sym {
            None => h.write_field(&[0u8]),
            Some(s) => {
                h.write_field(&[1u8]);
                h.write_field(&s.0.to_le_bytes());
            }
        }
    }
    h.write_field(&(material.field_signature_material.len() as u64).to_le_bytes());
    for field in &material.field_signature_material {
        h.write_str_field(&field.field_name);
        h.write_field(&field.field_type_symbol_id.0.to_le_bytes());
        h.write_field(&(field.field_index as u64).to_le_bytes());
    }
    let sem = match material.return_slot_semantics {
        ReturnSlotSemantics::Forward => 0u8,
        ReturnSlotSemantics::Generate => 1u8,
    };
    h.write_field(&[sem]);
    match &material.build_identity_fragment {
        None => h.write_field(&[0u8]),
        Some(s) => {
            h.write_field(&[1u8]);
            h.write_str_field(s);
        }
    }
    match &material.policy_export_fingerprint_fragment {
        None => h.write_field(&[0u8]),
        Some(s) => {
            h.write_field(&[1u8]);
            h.write_str_field(s);
        }
    }
    let raw = u64::from_str_radix(&h.finish_hex(), 16)
        .expect("Fnv1a64::finish_hex must produce a valid u64 hex string");
    TypeDefinitionInstanceId(if raw == 0 { 1 } else { raw })
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
/// Reads `callee_primitive` from the candidate itself. Invocation is pure
/// — no graph mutation, no `NamespaceDelta` installation.
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
        crate::model::CoreMetaFunction::Struct => invoke_struct_type_definition(&input),
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

    let type_symbol_id = match mat.known_type_symbols.get(0).and_then(|s| *s) {
        Some(s) => s,
        None => {
            return MetaInvocationResult::Diagnostic(
                Diagnostic::hard_error(
                    "IdentityType: argument is not a classified type object with a TypeSymbol",
                    Some(input.provenance.clone()),
                )
                .with_symbol_context(candidate.callee_symbol_id),
            );
        }
    };

    MetaInvocationResult::Value(MetaInvocationValue::ForwardedValue(ForwardedValue {
        target: MetaValueTarget::TypeSymbol(type_symbol_id),
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

    let _type_symbol_id = match mat.known_type_symbols.get(0).and_then(|s| *s) {
        Some(s) => s,
        None => {
            return MetaInvocationResult::Diagnostic(
                Diagnostic::hard_error(
                    "UnaryConstructionPrototype: argument is not a classified type object with a TypeSymbol",
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

fn invoke_struct_type_definition(input: &MetaInvocationInput) -> MetaInvocationResult {
    let candidate = &input.candidate;
    let mat = &candidate.canonical_key_seed.argument_product_shape_material;

    if mat.arity == 0 {
        return MetaInvocationResult::Diagnostic(
            Diagnostic::hard_error(
                "struct: expected at least one classified field argument",
                Some(input.provenance.clone()),
            )
            .with_symbol_context(candidate.callee_symbol_id),
        );
    }

    let field_signature_material =
        match field_signature_material_from_candidate(candidate, &input.provenance) {
            Ok(fields) => fields,
            Err(diagnostic) => return MetaInvocationResult::Diagnostic(diagnostic),
        };

    let identity_material = TypeDefinitionIdentityMaterial {
        callee_symbol_id: candidate.callee_symbol_id,
        canonical_args: mat.clone(),
        field_signature_material: field_signature_material.clone(),
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
    let type_definition_id = compute_type_definition_instance_id(&identity_material);
    let fields = field_signature_material
        .iter()
        .map(|field| GeneratedFieldDefinition {
            name: field.field_name.clone(),
            type_symbol_id: field.field_type_symbol_id,
            index: field.field_index,
            provenance: field.provenance.clone(),
        })
        .collect();

    MetaInvocationResult::Value(MetaInvocationValue::GeneratedTypeDefinitionValue(
        GeneratedTypeDefinitionValue {
            type_definition_id,
            identity_material,
            fields,
            return_view: ReturnViewShape::Leaf,
            provenance: input.provenance.clone(),
        },
    ))
}

fn field_signature_material_from_candidate(
    candidate: &PreparedCallableCandidate,
    provenance: &Provenance,
) -> Result<Vec<FieldSignatureMaterial>, Diagnostic> {
    let mut fields = Vec::new();
    let mut seen_names = BTreeSet::new();

    for raw_arg in &candidate.arg_product_shape.raw_args {
        if !matches!(
            raw_arg.value_class,
            RawArgValueClass::NonValue(NonValueArgKind::TypeObject)
        ) {
            return Err(Diagnostic::hard_error(
                "struct field type did not resolve as TypeSymbol",
                Some(raw_arg.provenance.clone()),
            )
            .with_symbol_context(candidate.callee_symbol_id));
        }
        let Some(type_symbol_id) = raw_arg.known_type_symbol_id else {
            return Err(Diagnostic::hard_error(
                "struct field type did not resolve as TypeSymbol",
                Some(raw_arg.provenance.clone()),
            )
            .with_symbol_context(candidate.callee_symbol_id));
        };
        let atom = candidate
            .arg_product_shape
            .flattened
            .atoms
            .get(raw_arg.index)
            .ok_or_else(|| {
                Diagnostic::hard_error(
                    "struct argument product shape is missing field atom material",
                    Some(provenance.clone()),
                )
                .with_symbol_context(candidate.callee_symbol_id)
            })?;
        let (field_name, field_provenance) =
            struct_field_name_from_atom(atom, candidate.callee_symbol_id)?;
        if !seen_names.insert(field_name.clone()) {
            return Err(Diagnostic::hard_error(
                format!("duplicate struct field `{field_name}`"),
                Some(field_provenance),
            )
            .with_symbol_context(candidate.callee_symbol_id));
        }
        fields.push(FieldSignatureMaterial {
            field_name,
            field_type_symbol_id: type_symbol_id,
            field_index: raw_arg.index,
            provenance: field_provenance,
        });
    }

    Ok(fields)
}

fn struct_field_name_from_atom(
    atom: &ProductAtom,
    callee_symbol_id: SymbolId,
) -> Result<(String, Provenance), Diagnostic> {
    let ProductAtom::Expression { expr, .. } = atom else {
        return Err(Diagnostic::hard_error(
            "invalid struct syntax: unit field or trailing unit is not supported",
            Some(atom.provenance().clone()),
        )
        .with_symbol_context(callee_symbol_id));
    };
    let NormExpr::Call {
        source,
        target,
        origin,
    } = expr
    else {
        return Err(Diagnostic::hard_error(
            "invalid struct syntax: expected a field form like `uint8 a`",
            Some(atom.provenance().clone()),
        )
        .with_symbol_context(callee_symbol_id));
    };
    if source.elements.len() != 1 {
        return Err(Diagnostic::hard_error(
            "invalid struct syntax: nested product fields are not supported in v0.8",
            Some(Provenance::from_norm_origin(
                "struct field source",
                &source.origin,
            )),
        )
        .with_symbol_context(callee_symbol_id));
    }
    match &source.elements[0] {
        NormProductElem::Expr(NormExpr::Product(product)) => {
            return Err(Diagnostic::hard_error(
                "invalid struct syntax: nested product fields are not supported in v0.8",
                Some(Provenance::from_norm_origin(
                    "nested struct field product",
                    &product.origin,
                )),
            )
            .with_symbol_context(callee_symbol_id));
        }
        NormProductElem::Unit { origin } => {
            return Err(Diagnostic::hard_error(
                "invalid struct syntax: unit field type is not supported",
                Some(Provenance::from_norm_origin(
                    "unit struct field type",
                    origin,
                )),
            )
            .with_symbol_context(callee_symbol_id));
        }
        NormProductElem::Expr(_) => {}
    }

    let field_name = match target.as_ref() {
        NormExpr::Name { text, .. } => text.clone(),
        _ => {
            return Err(Diagnostic::hard_error(
                "invalid struct syntax: expected a field binder name",
                Some(atom.provenance().clone()),
            )
            .with_symbol_context(callee_symbol_id));
        }
    };

    Ok((
        field_name,
        Provenance::from_norm_origin("struct field", origin),
    ))
}
