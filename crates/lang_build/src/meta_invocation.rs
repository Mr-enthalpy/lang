//! Formal meta invocation boundary.
//!
//! Consumes a `PreparedCallableCandidate` and dispatches to the appropriate
//! primitive invocation. This step is graph-installation-free and binding-free:
//! it produces a `MetaExecutionMaterial` but does **not** install
//! `NamespaceDelta`, bind declared symbols, or mutate the namespace graph. It
//! may allocate or attach registry-backed materialization state.
//!
//! ## Separation of concerns
//!
//! ```text
//! CandidatePrepResult::ApplicablePlaceholder
//!   → MetaInvocationInput
//!   → invoke_meta_callable
//!   → MetaPrimitiveExecution::Material(MetaExecutionMaterial)
//!     (no semantic result, graph installation, or binding)
//!
//! MetaExecutionMaterial
//!   → bind_meta_invocation_value_result (meta.rs)
//!   → MetaExpansionResult  (declaration binding, with NamespaceDelta)
//! ```
//!
//! `invoke_meta_callable_with_materialization_state` may populate
//! `StructMaterializationState` for values whose semantic identity is
//! registry-backed, such as `StructConstructionMaterial`. This is still
//! graph-installation-free with respect to the namespace graph. The cache stores
//! only replayable value material and strips concrete registry-backed
//! `StructPatternMaterialId`s before insertion; cache hits rematerialize heads in the
//! caller's current state.
//!
//! Production invocation reaches this primitive executor only after ordinary
//! value → complete type → associated `()` resolution has selected a call-entry
//! semantic value. The implicit `self` belongs to that invocation frame, never
//! to `ProductObject` / `ArgProductShape` / `RawArgShape`.

use std::collections::{BTreeMap, BTreeSet};

use lang_syntax::{NormExpr, NormProductElem};

use crate::{
    extraction_view::{
        ContentObservationInterface, ObservedArgumentContent, ObservedAtomContent, ObservedAtomKind,
    },
    meta_cache::MetaInstanceCache,
    meta_candidate::{CanonicalArgProductShapeMaterial, PreparedCallableCandidate},
    meta_key::MetaInvocationMaterialKey,
    model::{Diagnostic, Provenance, SymbolId},
    product_shape::{NonValueArgKind, ProductAtom, RawArgValueClass},
    struct_decoder::DecodedStructPattern,
    struct_pattern_material::{
        derive_struct_sum_material, StructPatternSyntaxMaterial, StructSumSyntaxMaterial,
        StructuralMemberVisibility,
    },
    struct_pattern_registry::{
        StructFieldPatternMaterial, StructMaterializationState, StructPatternMaterialContext,
        StructPatternMaterialId,
    },
};

/// Input for formal meta invocation.
///
/// The candidate must already have passed candidate preparation
/// (`prepare_meta_callable_candidate_with_declared_planes`).
/// The primitive is read from `candidate.callee_primitive` — callers do not
/// pass it separately, preventing primitive-vs-candidate mismatch.
#[derive(Clone, Debug)]
pub struct MetaInvocationInput {
    pub candidate: PreparedCallableCandidate,
    pub provenance: Provenance,
    /// Pre-decoded struct type-pattern shape, if this is a struct invocation
    /// and the decoder was able to interpret the argument.
    pub struct_decoded_pattern: Option<DecodedStructPattern>,
}

impl MetaInvocationInput {
    pub fn new(candidate: PreparedCallableCandidate, provenance: Provenance) -> Self {
        Self {
            candidate,
            provenance,
            struct_decoded_pattern: None,
        }
    }
}

impl MetaExecutionMaterial {
    /// Semantic equality: compares identity material without provenance.
    ///
    /// Two invocation values with the same semantic identity but
    /// different provenance compare equal here. This is distinct
    /// from `PartialEq` which includes provenance.
    pub fn semantic_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                MetaExecutionMaterial::ForwardedResultMaterial(lhs),
                MetaExecutionMaterial::ForwardedResultMaterial(rhs),
            ) => lhs.type_observation == rhs.type_observation && lhs.return_view == rhs.return_view,
            (
                MetaExecutionMaterial::UnaryConstructionMaterial(lhs),
                MetaExecutionMaterial::UnaryConstructionMaterial(rhs),
            ) => {
                lhs.construction_instance_id == rhs.construction_instance_id
                    && lhs.identity_material == rhs.identity_material
                    && lhs.return_view == rhs.return_view
            }
            (
                MetaExecutionMaterial::StructConstructionMaterial(lhs),
                MetaExecutionMaterial::StructConstructionMaterial(rhs),
            ) => {
                lhs.type_definition_id == rhs.type_definition_id
                    && lhs.identity_material == rhs.identity_material
                    && lhs.fields.len() == rhs.fields.len()
                    && lhs
                        .fields
                        .iter()
                        .zip(rhs.fields.iter())
                        .all(|(a, b)| a.semantic_eq(b))
                    && lhs.pattern_materials == rhs.pattern_materials
                    && lhs.return_view == rhs.return_view
                    && type_pattern_expr_semantic_eq(&lhs.type_pattern_expr, &rhs.type_pattern_expr)
                    && sum_struct_pattern_material_semantic_eq(
                        &lhs.sum_struct_pattern_material,
                        &rhs.sum_struct_pattern_material,
                    )
                    && lhs.canonical_pattern_override == rhs.canonical_pattern_override
            }
            _ => false,
        }
    }
}

impl StructConstructionMaterial {
    /// The generated definition's structural Pattern normal form.
    ///
    /// Construction callable identity, build identity, export metadata,
    /// provenance, and the FNV-derived `type_definition_id` are deliberately
    /// absent.  A naked struct Product remains an ordered layer even when all
    /// fields are named.  Only a fully named Product used as the body of a
    /// named Pattern becomes an unordered map keyed by each field's complete
    /// navigation.  A decoded top Pattern name is Pattern-internal
    /// identity—not the eventual carrier Symbol name—and is appended as the
    /// outer component of every field navigation.
    pub fn canonical_pattern_value(&self) -> crate::CanonicalPatternValue {
        if let Some(value) = &self.canonical_pattern_override {
            return value.clone();
        }
        if let Some(pattern) = self.type_pattern_expr.as_ref() {
            let field_types = self
                .fields
                .iter()
                .map(|field| (field.name.as_str(), field.type_observation))
                .collect::<BTreeMap<_, _>>();
            return canonicalize_struct_pattern(
                pattern.transparent_singleton(),
                &crate::CanonicalFullNavigation::new(std::iter::empty::<String>()),
                &field_types,
                crate::PatternLayerContext::NakedProduct,
            );
        }
        crate::CanonicalPatternValue::OrderedLayer(
            self.fields
                .iter()
                .map(|field| crate::CanonicalOrderedPatternEntry {
                    navigation: Some(crate::CanonicalFullNavigation::from_component(
                        field.name.clone(),
                    )),
                    value: crate::CanonicalPatternValue::Atom(crate::CanonicalPatternAtom::Type(
                        field.type_observation,
                    )),
                })
                .collect(),
        )
    }

    /// Mirror a successful incremental pure-P contribution on the returned
    /// construction artifact.  The SemanticWorld PatternValue remains the
    /// authority; this copy ensures later binding/materialization observes
    /// the same completed Pattern rather than the original one-shot body.
    pub fn set_canonical_pattern_value(&mut self, value: crate::CanonicalPatternValue) {
        self.canonical_pattern_override = Some(value);
    }
}

fn canonicalize_struct_pattern(
    pattern: &StructPatternSyntaxMaterial,
    enclosing: &crate::CanonicalFullNavigation,
    field_types: &BTreeMap<&str, crate::CanonicalTypeObservation>,
    layer_context: crate::PatternLayerContext,
) -> crate::CanonicalPatternValue {
    match pattern {
        StructPatternSyntaxMaterial::Leaf {
            local_pattern_name, ..
        } => crate::CanonicalPatternValue::Atom(crate::CanonicalPatternAtom::Type(
            *field_types
                .get(local_pattern_name.as_str())
                .expect("every decoded struct leaf has one classified TypeValue"),
        )),
        StructPatternSyntaxMaterial::Named {
            child,
            pattern_name,
            ..
        } => {
            let navigation = complete_pattern_navigation(pattern_name, enclosing);
            crate::CanonicalPatternValue::NamedPattern {
                navigation: navigation.clone(),
                body: Box::new(match child.as_ref() {
                    StructPatternSyntaxMaterial::Leaf { .. }
                    | StructPatternSyntaxMaterial::Named { .. } => {
                        let child_navigation = complete_pattern_navigation(
                            direct_pattern_navigation(child)
                                .expect("leaf and named child carry navigation"),
                            &navigation,
                        );
                        crate::CanonicalPatternValue::UnorderedLayer(BTreeMap::from([(
                            child_navigation,
                            canonicalize_struct_pattern(
                                child,
                                &navigation,
                                field_types,
                                crate::PatternLayerContext::NakedProduct,
                            ),
                        )]))
                    }
                    StructPatternSyntaxMaterial::Product { .. }
                    | StructPatternSyntaxMaterial::Sum { .. } => canonicalize_struct_pattern(
                        child,
                        &navigation,
                        field_types,
                        crate::PatternLayerContext::NamedPatternBody,
                    ),
                }),
            }
        }
        StructPatternSyntaxMaterial::Product { elements, .. } => {
            let mut unordered = BTreeMap::new();
            let mut ordered = Vec::with_capacity(elements.len());
            let mut fully_named = true;
            for element in elements {
                let navigation = direct_pattern_navigation(element)
                    .map(|name| complete_pattern_navigation(name, enclosing));
                let value = canonicalize_struct_pattern(
                    element,
                    enclosing,
                    field_types,
                    crate::PatternLayerContext::NakedProduct,
                );
                if let Some(navigation) = &navigation {
                    assert!(
                        !unordered.contains_key(navigation),
                        "decoded struct elements have unique complete navigation names"
                    );
                    unordered.insert(navigation.clone(), value.clone());
                } else {
                    fully_named = false;
                }
                ordered.push(crate::CanonicalOrderedPatternEntry { navigation, value });
            }
            if layer_context == crate::PatternLayerContext::NamedPatternBody && fully_named {
                crate::CanonicalPatternValue::UnorderedLayer(unordered)
            } else {
                crate::CanonicalPatternValue::OrderedLayer(ordered)
            }
        }
        StructPatternSyntaxMaterial::Sum { alternatives, .. } => crate::CanonicalPatternValue::Sum(
            alternatives
                .iter()
                .map(|alternative| {
                    canonicalize_struct_pattern(
                        alternative,
                        enclosing,
                        field_types,
                        crate::PatternLayerContext::NakedProduct,
                    )
                })
                .collect(),
        ),
    }
}

fn direct_pattern_navigation(pattern: &StructPatternSyntaxMaterial) -> Option<&str> {
    match pattern {
        StructPatternSyntaxMaterial::Leaf {
            local_pattern_name, ..
        } => Some(local_pattern_name),
        StructPatternSyntaxMaterial::Named { pattern_name, .. } => Some(pattern_name),
        StructPatternSyntaxMaterial::Product { .. } | StructPatternSyntaxMaterial::Sum { .. } => {
            None
        }
    }
}

fn complete_pattern_navigation(
    inner: &str,
    enclosing: &crate::CanonicalFullNavigation,
) -> crate::CanonicalFullNavigation {
    crate::CanonicalFullNavigation::new(
        std::iter::once(inner.to_string()).chain(enclosing.components().iter().cloned()),
    )
}

fn type_pattern_expr_semantic_eq(
    lhs: &Option<crate::struct_pattern_material::StructPatternSyntaxMaterial>,
    rhs: &Option<crate::struct_pattern_material::StructPatternSyntaxMaterial>,
) -> bool {
    match (lhs, rhs) {
        (Some(l), Some(r)) => l.materially_equal(r),
        (None, None) => true,
        _ => false,
    }
}

fn sum_struct_pattern_material_semantic_eq(
    lhs: &Option<crate::struct_pattern_material::StructSumSyntaxMaterial>,
    rhs: &Option<crate::struct_pattern_material::StructSumSyntaxMaterial>,
) -> bool {
    match (lhs, rhs) {
        (Some(l), Some(r)) => l.materially_equal(r),
        (None, None) => true,
        _ => false,
    }
}

/// Replayable execution material produced behind the unified invocation
/// result boundary.
///
/// `ForwardedResultMaterial` is produced by the restricted evaluator's legacy
/// `IdentityType` forwarding proof. It is transitional transport, not the final
/// formal meta-return model. `UnaryConstructionMaterial` is produced by
/// `UnaryConstructionPrototype` from argument-derived construction material.
/// `StructConstructionMaterial` is the replayable construction material
/// produced while evaluating `struct`; it is not the semantic result of that
/// callable.  The world-connected invocation path installs the material and
/// returns a complete type value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetaExecutionMaterial {
    ForwardedResultMaterial(ForwardedResultMaterial),
    UnaryConstructionMaterial(UnaryConstructionMaterial),
    StructConstructionMaterial(StructConstructionMaterial),
}

/// Result of executing a compiler primitive before semantic result
/// materialization.
///
/// `Material` is replayable implementation material, not a value in any
/// [`crate::DeclaredResultClass`].  In particular, a
/// [`StructConstructionMaterial`] must be installed and observed as a
/// [`crate::CompleteTypeValue`] before a
/// [`crate::InvocationResult::SemanticResult`] with class
/// [`crate::DeclaredResultClass::CompleteType`] may be formed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetaPrimitiveExecution {
    Material(MetaExecutionMaterial),
    Residual(crate::InvocationResidual),
    Diagnostic(Diagnostic),
}

impl MetaExecutionMaterial {
    pub fn return_normal_form_shape(&self) -> ObservedArgumentContent {
        match self {
            MetaExecutionMaterial::ForwardedResultMaterial(value) => {
                ObservedArgumentContent::ValuePoint(ObservedAtomContent {
                    value_kind: ObservedAtomKind::Forwarded {
                        type_value: value.type_value,
                    },
                    extraction_interface: ContentObservationInterface::Leaf,
                    provenance: value.provenance.clone(),
                })
            }
            MetaExecutionMaterial::UnaryConstructionMaterial(value) => {
                ObservedArgumentContent::ValuePoint(ObservedAtomContent {
                    value_kind: ObservedAtomKind::GeneratedConstruction {
                        construction_instance_id: value.construction_instance_id,
                    },
                    extraction_interface: ContentObservationInterface::Leaf,
                    provenance: value.provenance.clone(),
                })
            }
            MetaExecutionMaterial::StructConstructionMaterial(value) => {
                ObservedArgumentContent::ValuePoint(ObservedAtomContent {
                    value_kind: ObservedAtomKind::GeneratedTypeDefinition {
                        type_definition_id: value.type_definition_id,
                    },
                    extraction_interface: ContentObservationInterface::Leaf,
                    provenance: value.provenance.clone(),
                })
            }
        }
    }
}

/// Forwarded existing value used by the restricted evaluator's `IdentityType`
/// proof path. The final formal meta-return model does not expose this as a
/// separate source-level forwarding category.
///
/// The target carries the forwarded TypeValue directly. Reaching that value
/// through a graph Symbol does not make the carrier Symbol part of the
/// invocation result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForwardedResultMaterial {
    /// The represented value itself. Even when evaluation reached it through a
    /// source name, the carrier Symbol is not part of this result identity.
    /// Symbol/place forwarding belongs to the separate `===` mechanism.
    pub type_value: crate::TypeValueId,
    /// The type OBSERVATION forwarded through this result — `Addr(Norm_type)`
    /// when the invocation boundary was world-connected, otherwise the
    /// `Detached` projection.  Semantic equality consumes this, never the bare
    /// `type_value` projection.
    pub type_observation: crate::CanonicalTypeObservation,
    pub return_view: ReturnViewShape,
    pub provenance: Provenance,
}

/// Generated construction value — the call returns a new construction value
/// whose external identity is shielded by callee + canonical args + build
/// identity. Reserved for future generative type constructors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnaryConstructionMaterial {
    pub construction_instance_id: ConstructionInstanceId,
    pub identity_material: ConstructionIdentityMaterial,
    pub return_view: ReturnViewShape,
    pub provenance: Provenance,
}

/// Generated type-definition value produced by formal `struct` invocation.
///
/// This is graph-installation-free and binding-free invocation output. Registry
/// material may already be attached; the declared type symbol, associated
/// namespace, and field projections are binding materialization artifacts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructConstructionMaterial {
    /// Normalized struct body identity.  This is body material only:
    /// it never carries a canonical root by itself.  Two different meta
    /// functions with identical bodies share this id while their canonical
    /// TypeValue roots stay distinct.
    pub type_definition_id: TypeDefinitionInstanceId,
    pub identity_material: TypeDefinitionIdentityMaterial,
    pub fields: Vec<GeneratedFieldDefinition>,
    pub pattern_materials: Option<TypeDefinitionStructPatternMaterials>,
    pub return_view: ReturnViewShape,
    /// The decoded type-pattern expression shape, if the struct argument
    /// was successfully decoded by the struct-local decoder.
    pub type_pattern_expr: Option<StructPatternSyntaxMaterial>,
    /// The sum pattern space derived from the type-pattern expression,
    /// if the expression contains a sum.
    pub sum_struct_pattern_material: Option<StructSumSyntaxMaterial>,
    /// Canonical semantic TypeValue root assigned at meta-instance
    /// registration: `TypeValue = (OuterMetaInstanceRoot,
    /// NormalizedStructBody)`.  `None` until the invocation owner
    /// registers the member; stripped by the invocation cache like
    /// pattern heads (a cached body must never leak another instance's
    /// root).
    pub canonical_type: Option<crate::TypeValueId>,
    /// Updated canonical Pattern material after incremental pure-P
    /// contributions. `None` means the normal form is derived directly from
    /// the decoded `struct` body.
    pub canonical_pattern_override: Option<crate::CanonicalPatternValue>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeDefinitionStructPatternMaterials {
    pub owner_head: StructPatternMaterialId,
    pub field_heads: Vec<GeneratedFieldStructPatternMaterial>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedFieldStructPatternMaterial {
    pub field_name: String,
    pub field_head: StructPatternMaterialId,
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
    /// First-order field type projection.  Transport/registry material only:
    /// canonical equality and instance identity consume
    /// `field_type_observation`, never this bare projection and never the
    /// carrier name used in source.
    pub field_type_value: crate::TypeValueId,
    /// The field type's observation identity — `Addr(Norm_type)` including
    /// the recursive Val2 read at the argument's carrier place when the
    /// invocation boundary was world-connected, otherwise the `Detached`
    /// projection (which never equals an observed address).
    pub field_type_observation: crate::CanonicalTypeObservation,
    /// Compatibility graph carrier retained for current StructPatternMaterial/field
    /// installation only. It is non-identity material.
    pub field_type_carrier_symbol: SymbolId,
    pub field_index: usize,
    pub visibility: StructuralMemberVisibility,
    pub provenance: Provenance,
}

impl PartialEq for FieldSignatureMaterial {
    fn eq(&self, other: &Self) -> bool {
        self.field_name == other.field_name
            && self.field_type_observation == other.field_type_observation
            && self.field_index == other.field_index
            && self.visibility == other.visibility
    }
}

impl Eq for FieldSignatureMaterial {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedFieldDefinition {
    pub name: String,
    pub type_value: crate::TypeValueId,
    /// The field type's observation identity; semantic equality consumes
    /// this, never the bare `type_value` projection.
    pub type_observation: crate::CanonicalTypeObservation,
    pub type_carrier_symbol: SymbolId,
    pub index: usize,
    pub visibility: StructuralMemberVisibility,
    pub struct_pattern_registry: Option<StructPatternMaterialId>,
    pub provenance: Provenance,
}

impl GeneratedFieldDefinition {
    /// Semantic equality: compares field identity material without provenance.
    pub fn semantic_eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.type_observation == other.type_observation
            && self.visibility == other.visibility
            && self.index == other.index
            && self.struct_pattern_registry == other.struct_pattern_registry
    }
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
    for type_value in &material.canonical_args.known_type_values {
        match type_value {
            None => h.write_field(&[0u8]),
            Some(type_value) => {
                h.write_field(&[1u8]);
                h.write_field(&type_value.0.to_le_bytes());
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
    h.write_field(&(material.canonical_args.known_type_values.len() as u64).to_le_bytes());
    for type_value in &material.canonical_args.known_type_values {
        match type_value {
            None => h.write_field(&[0u8]),
            Some(type_value) => {
                h.write_field(&[1u8]);
                h.write_field(&type_value.0.to_le_bytes());
            }
        }
    }
    h.write_field(&(material.field_signature_material.len() as u64).to_le_bytes());
    for field in &material.field_signature_material {
        h.write_str_field(&field.field_name);
        match field.field_type_observation {
            crate::CanonicalTypeObservation::Observed(addr) => {
                h.write_field(&[1u8]);
                h.write_field(&addr.0.to_le_bytes());
            }
            crate::CanonicalTypeObservation::Detached(type_value) => {
                h.write_field(&[0u8]);
                h.write_field(&type_value.0.to_le_bytes());
            }
        }
        h.write_field(&(field.field_index as u64).to_le_bytes());
        h.write_field(&[match field.visibility {
            StructuralMemberVisibility::Default => 0,
            StructuralMemberVisibility::Public => 1,
            StructuralMemberVisibility::Private => 2,
        }]);
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

/// Return value shape marker.
///
/// `Leaf` marks a returned normal form that is a non-product value point. If
/// the value point has no exposed extraction interface, `?` is idempotent.
/// `Product` marks product normal form `P`: `P? = P`, and product pattern
/// matching consumes `P` directly.
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
pub fn invoke_meta_callable(input: MetaInvocationInput) -> MetaPrimitiveExecution {
    // Standalone compatibility entry point. It is suitable for one-off formal
    // invocation tests but does not preserve StructPatternMaterialRegistry continuity
    // across calls. Callers that need registry-backed identity continuity must
    // use `invoke_meta_callable_with_materialization_state`.
    let mut materialization_state = StructMaterializationState::default();
    invoke_meta_callable_with_materialization_state(input, &mut materialization_state)
}

pub fn invoke_meta_callable_with_materialization_state(
    input: MetaInvocationInput,
    materialization_state: &mut StructMaterializationState,
) -> MetaPrimitiveExecution {
    let Some(primitive) = input.candidate.callee_primitive else {
        return MetaPrimitiveExecution::Diagnostic(
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
        crate::model::CoreMetaFunction::Struct => {
            invoke_struct_type_definition(&input, materialization_state)
        }
        _ => MetaPrimitiveExecution::Diagnostic(
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
/// invocation value. The cache stores only `MetaExecutionMaterial` — no
/// `NamespaceDelta`.
pub fn invoke_meta_callable_cached(
    input: MetaInvocationInput,
    key: MetaInvocationMaterialKey,
    cache: &mut MetaInstanceCache,
) -> MetaPrimitiveExecution {
    // Standalone compatibility entry point. Cache hits for registry-backed
    // values are rehydrated into this temporary state only; callers that need
    // to query the registry later must use the `_with_materialization_state`
    // variant.
    let mut materialization_state = StructMaterializationState::default();
    invoke_meta_callable_cached_with_materialization_state(
        input,
        key,
        cache,
        &mut materialization_state,
    )
}

pub fn invoke_meta_callable_cached_with_materialization_state(
    input: MetaInvocationInput,
    key: MetaInvocationMaterialKey,
    cache: &mut MetaInstanceCache,
    materialization_state: &mut StructMaterializationState,
) -> MetaPrimitiveExecution {
    // Validate primitive before cache lookup — prevents a manually-inserted
    // cache entry for a no-primitive candidate from bypassing validation.
    if input.candidate.callee_primitive.is_none() {
        return MetaPrimitiveExecution::Diagnostic(
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
    if let Some(cached) = cache.lookup(&key) {
        return cached_value_for_current_materialization_state(
            cached.result.clone(),
            materialization_state,
            input.provenance,
        );
    }
    let result = invoke_meta_callable_with_materialization_state(input, materialization_state);
    if let MetaPrimitiveExecution::Material(val) = &result {
        cache.insert(
            key,
            cacheable_invocation_value(val.clone()),
            Provenance::new("cached meta invocation result"),
        );
    }
    result
}

fn invoke_identity_type(input: &MetaInvocationInput) -> MetaPrimitiveExecution {
    let candidate = &input.candidate;
    let mat =
        CanonicalArgProductShapeMaterial::from_arg_product_shape(&candidate.arg_product_shape);

    if mat.arity != 1 {
        return MetaPrimitiveExecution::Diagnostic(
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
            return MetaPrimitiveExecution::Diagnostic(
                Diagnostic::hard_error(
                    "IdentityType: argument is not a classified type object with a TypeValue",
                    Some(input.provenance.clone()),
                )
                .with_symbol_context(candidate.callee_symbol_id),
            );
        }
    };
    let type_observation = candidate
        .arg_product_shape
        .raw_args
        .first()
        .and_then(|raw| {
            raw.known_complete_type_observation
                .map(crate::CanonicalTypeObservation::Observed)
                .or_else(|| raw.type_observation())
        })
        .unwrap_or(crate::CanonicalTypeObservation::Detached(type_value));

    MetaPrimitiveExecution::Material(MetaExecutionMaterial::ForwardedResultMaterial(
        ForwardedResultMaterial {
            type_value,
            type_observation,
            return_view: ReturnViewShape::Leaf,
            provenance: input.provenance.clone(),
        },
    ))
}

fn invoke_unary_construction_prototype(input: &MetaInvocationInput) -> MetaPrimitiveExecution {
    let candidate = &input.candidate;
    let mat =
        CanonicalArgProductShapeMaterial::from_arg_product_shape(&candidate.arg_product_shape);

    if mat.arity != 1 {
        return MetaPrimitiveExecution::Diagnostic(
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

    let _type_value = match mat.known_type_values.first().and_then(|value| *value) {
        Some(value) => value,
        None => {
            return MetaPrimitiveExecution::Diagnostic(
                Diagnostic::hard_error(
                    "UnaryConstructionPrototype: argument is not a classified type object with a TypeValue",
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
        build_identity_fragment: candidate.build_identity.package_identity_fragment.clone(),
        policy_export_fingerprint_fragment: candidate
            .build_identity
            .policy_export_fingerprint_fragment
            .clone(),
        provenance: input.provenance.clone(),
    };
    let construction_instance_id = compute_construction_instance_id(&identity_material);

    MetaPrimitiveExecution::Material(MetaExecutionMaterial::UnaryConstructionMaterial(
        UnaryConstructionMaterial {
            construction_instance_id,
            identity_material,
            return_view: ReturnViewShape::Leaf,
            provenance: input.provenance.clone(),
        },
    ))
}

fn invoke_struct_type_definition(
    input: &MetaInvocationInput,
    materialization_state: &mut StructMaterializationState,
) -> MetaPrimitiveExecution {
    let candidate = &input.candidate;
    let mat =
        CanonicalArgProductShapeMaterial::from_arg_product_shape(&candidate.arg_product_shape);

    let pure_pattern_without_value = input
        .struct_decoded_pattern
        .as_ref()
        .is_some_and(|decoded| decoded.type_pattern_expr.is_pure_pattern_without_value());
    if mat.arity == 0 && !pure_pattern_without_value {
        return MetaPrimitiveExecution::Diagnostic(
            Diagnostic::hard_error(
                "struct: expected at least one `Expr name` field or a pure no-value Pattern such as `(() t)` or `if | else`",
                Some(input.provenance.clone()),
            )
            .with_symbol_context(candidate.callee_symbol_id),
        );
    }

    let field_signature_material = match field_signature_material_from_candidate(
        candidate,
        input.struct_decoded_pattern.as_ref(),
        &input.provenance,
    ) {
        Ok(fields) => fields,
        Err(diagnostic) => return MetaPrimitiveExecution::Diagnostic(diagnostic),
    };

    let identity_material = TypeDefinitionIdentityMaterial {
        callee_symbol_id: candidate.callee_symbol_id,
        canonical_args: mat.clone(),
        field_signature_material: field_signature_material.clone(),
        return_slot_semantics: ReturnSlotSemantics::Generate,
        build_identity_fragment: candidate.build_identity.package_identity_fragment.clone(),
        policy_export_fingerprint_fragment: candidate
            .build_identity
            .policy_export_fingerprint_fragment
            .clone(),
        provenance: input.provenance.clone(),
    };
    let type_definition_id = compute_type_definition_instance_id(&identity_material);
    let fields = field_signature_material
        .iter()
        .map(|field| GeneratedFieldDefinition {
            name: field.field_name.clone(),
            type_value: field.field_type_value,
            type_observation: field.field_type_observation,
            type_carrier_symbol: field.field_type_carrier_symbol,
            index: field.field_index,
            visibility: field.visibility,
            struct_pattern_registry: None,
            provenance: field.provenance.clone(),
        })
        .collect();
    let value = StructConstructionMaterial {
        type_definition_id,
        identity_material,
        fields,
        pattern_materials: None,
        return_view: ReturnViewShape::Leaf,
        type_pattern_expr: input
            .struct_decoded_pattern
            .as_ref()
            .map(|p| p.type_pattern_expr.clone()),
        sum_struct_pattern_material: input
            .struct_decoded_pattern
            .as_ref()
            .and_then(|p| derive_struct_sum_material(&p.type_pattern_expr)),
        canonical_type: None,
        canonical_pattern_override: None,
        provenance: input.provenance.clone(),
    };
    match attach_type_definition_pattern_materials(
        value,
        materialization_state,
        input.provenance.clone(),
    ) {
        Ok(value) => MetaPrimitiveExecution::Material(
            MetaExecutionMaterial::StructConstructionMaterial(value),
        ),
        Err(diagnostic) => MetaPrimitiveExecution::Diagnostic(diagnostic),
    }
}

fn cached_value_for_current_materialization_state(
    value: MetaExecutionMaterial,
    materialization_state: &mut StructMaterializationState,
    provenance: Provenance,
) -> MetaPrimitiveExecution {
    match value {
        MetaExecutionMaterial::StructConstructionMaterial(value) => {
            match attach_type_definition_pattern_materials(value, materialization_state, provenance)
            {
                Ok(value) => MetaPrimitiveExecution::Material(
                    MetaExecutionMaterial::StructConstructionMaterial(value),
                ),
                Err(diagnostic) => MetaPrimitiveExecution::Diagnostic(diagnostic),
            }
        }
        MetaExecutionMaterial::ForwardedResultMaterial(value) => {
            MetaPrimitiveExecution::Material(MetaExecutionMaterial::ForwardedResultMaterial(value))
        }
        MetaExecutionMaterial::UnaryConstructionMaterial(value) => {
            MetaPrimitiveExecution::Material(MetaExecutionMaterial::UnaryConstructionMaterial(
                value,
            ))
        }
    }
}

fn cacheable_invocation_value(value: MetaExecutionMaterial) -> MetaExecutionMaterial {
    match value {
        MetaExecutionMaterial::StructConstructionMaterial(mut value) => {
            value.pattern_materials = None;
            value.canonical_type = None;
            for field in &mut value.fields {
                field.struct_pattern_registry = None;
            }
            MetaExecutionMaterial::StructConstructionMaterial(value)
        }
        other => other,
    }
}

/// Attach pattern heads for a generated type definition under its anonymous
/// generated fallback context.
///
/// Formal `struct` invocation is graph-installation-free and binding-free. It
/// may allocate registry-backed material through this fallback before final
/// resolved pattern-scope semantics are available.
pub fn attach_type_definition_pattern_materials(
    value: StructConstructionMaterial,
    materialization_state: &mut StructMaterializationState,
    provenance: Provenance,
) -> Result<StructConstructionMaterial, Diagnostic> {
    let type_definition_id = value.type_definition_id;
    let owner_display_name = value
        .type_pattern_expr
        .as_ref()
        .and_then(owner_display_name_from_type_pattern_expr)
        .unwrap_or_else(|| {
            format!(
                "generated-type-definition-{}",
                value.type_definition_id.as_u64()
            )
        });
    attach_type_definition_pattern_materials_with_context(
        value,
        materialization_state,
        StructPatternMaterialContext::StructDefinition { type_definition_id },
        owner_display_name,
        provenance,
    )
}

/// Attach pattern heads for a generated type definition under an explicit
/// materialization context.
///
/// This is a doc-hidden transitional test-support API. It remains public only
/// so integration tests can exercise categorical registry identity. It is not
/// a stable pattern-owner construction capability, and production semantic
/// callers must not treat `Global`, `Namespace`, `Local`, or `Generated`
/// contexts as final `ResolvedPatternScope` identities.
///
/// The display name is diagnostic material only. The owner `StructPatternMaterialId`
/// identity comes from `context`; callers must not derive identity from the
/// bare source spelling.
#[doc(hidden)]
pub fn attach_type_definition_pattern_materials_with_context(
    mut value: StructConstructionMaterial,
    materialization_state: &mut StructMaterializationState,
    context: StructPatternMaterialContext,
    owner_display_name: impl Into<String>,
    provenance: Provenance,
) -> Result<StructConstructionMaterial, Diagnostic> {
    let pattern_fields = value
        .identity_material
        .field_signature_material
        .iter()
        .map(|field| StructFieldPatternMaterial {
            field_name: field.field_name.clone(),
            field_type_value: field.field_type_value,
            projection: crate::model::FieldProjection::Value,
            provenance: field.provenance.clone(),
        });
    let pattern_materialization = materialization_state
        .pattern_materials
        .materialize_struct_pattern(
            context,
            owner_display_name.into(),
            pattern_fields,
            provenance,
        )?;
    let struct_pattern_registry_by_name = pattern_materialization
        .field_heads
        .iter()
        .cloned()
        .collect::<std::collections::BTreeMap<_, _>>();
    for field in &mut value.fields {
        field.struct_pattern_registry = struct_pattern_registry_by_name.get(&field.name).copied();
    }
    value.pattern_materials = Some(TypeDefinitionStructPatternMaterials {
        owner_head: pattern_materialization.owner_head,
        field_heads: pattern_materialization
            .field_heads
            .into_iter()
            .map(
                |(field_name, field_head)| GeneratedFieldStructPatternMaterial {
                    field_name,
                    field_head,
                },
            )
            .collect(),
    });
    Ok(value)
}

fn owner_display_name_from_type_pattern_expr(expr: &StructPatternSyntaxMaterial) -> Option<String> {
    match expr {
        StructPatternSyntaxMaterial::Named { pattern_name, .. } => Some(pattern_name.clone()),
        _ => None,
    }
}

fn field_signature_material_from_candidate(
    candidate: &PreparedCallableCandidate,
    decoded: Option<&crate::struct_decoder::DecodedStructPattern>,
    provenance: &Provenance,
) -> Result<Vec<FieldSignatureMaterial>, Diagnostic> {
    let decoded_fields = decoded.map(|decoded| {
        let mut fields = Vec::new();
        collect_decoded_struct_fields(&decoded.type_pattern_expr, &mut fields);
        fields
    });
    if let Some(decoded_fields) = &decoded_fields {
        if decoded_fields.len() != candidate.arg_product_shape.raw_args.len() {
            return Err(Diagnostic::hard_error(
                "struct decoded field count does not match classified argument count",
                Some(provenance.clone()),
            )
            .with_symbol_context(candidate.callee_symbol_id));
        }
    }
    let mut fields = Vec::new();
    let mut seen_names = BTreeSet::new();

    for raw_arg in &candidate.arg_product_shape.raw_args {
        if !matches!(
            raw_arg.value_class,
            RawArgValueClass::NonValue(NonValueArgKind::CoreTypeProjection)
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
        let Some(type_value) = raw_arg.known_first_order_type_value else {
            return Err(Diagnostic::hard_error(
                "struct field type did not carry an evaluated TypeValue",
                Some(raw_arg.provenance.clone()),
            )
            .with_symbol_context(candidate.callee_symbol_id));
        };
        let (field_name, visibility, field_provenance) = if let Some(decoded_fields) =
            &decoded_fields
        {
            decoded_fields[raw_arg.index].clone()
        } else {
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
            let (name, provenance) = struct_field_name_from_atom(atom, candidate.callee_symbol_id)?;
            (name, StructuralMemberVisibility::Default, provenance)
        };
        if !seen_names.insert(field_name.clone()) {
            return Err(Diagnostic::hard_error(
                format!("duplicate struct field `{field_name}`"),
                Some(field_provenance),
            )
            .with_symbol_context(candidate.callee_symbol_id));
        }
        fields.push(FieldSignatureMaterial {
            field_name,
            field_type_value: type_value,
            field_type_observation: raw_arg
                .type_observation()
                .unwrap_or(crate::CanonicalTypeObservation::Detached(type_value)),
            field_type_carrier_symbol: type_symbol_id,
            field_index: raw_arg.index,
            visibility,
            provenance: field_provenance,
        });
    }

    Ok(fields)
}

fn collect_decoded_struct_fields(
    pattern: &StructPatternSyntaxMaterial,
    output: &mut Vec<(String, StructuralMemberVisibility, Provenance)>,
) {
    match pattern {
        StructPatternSyntaxMaterial::Leaf {
            local_pattern_name,
            visibility,
            provenance,
            ..
        } => {
            output.push((local_pattern_name.clone(), *visibility, provenance.clone()));
        }
        StructPatternSyntaxMaterial::Product { elements, .. } => {
            for element in elements {
                collect_decoded_struct_fields(element, output);
            }
        }
        StructPatternSyntaxMaterial::Sum { alternatives, .. } => {
            for alternative in alternatives {
                collect_decoded_struct_fields(alternative, output);
            }
        }
        StructPatternSyntaxMaterial::Named { child, .. } => {
            collect_decoded_struct_fields(child, output);
        }
    }
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
        target: _,
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

    let field_name = struct_field_name_from_expr(expr, atom.provenance(), callee_symbol_id)?;

    Ok((
        field_name,
        Provenance::from_norm_origin("struct field", origin),
    ))
}

fn struct_field_name_from_expr(
    expr: &NormExpr,
    provenance: &Provenance,
    callee_symbol_id: SymbolId,
) -> Result<String, Diagnostic> {
    let NormExpr::Call { source, target, .. } = expr else {
        return Err(Diagnostic::hard_error(
            "invalid struct syntax: expected a field form like `uint8 a`",
            Some(provenance.clone()),
        )
        .with_symbol_context(callee_symbol_id));
    };
    match target.as_ref() {
        NormExpr::Name { text, .. } => Ok(text.clone()),
        NormExpr::Call {
            source: annotation_source,
            target: annotation_target,
            ..
        } if is_member_view_target(annotation_target) => {
            let annotated =
                one_struct_member_expr(annotation_source, provenance, callee_symbol_id)?;
            match annotated {
                NormExpr::Name { text, .. } => Ok(text.clone()),
                other => struct_field_name_from_expr(other, provenance, callee_symbol_id),
            }
        }
        target if is_member_view_target(target) => {
            let annotated = one_struct_member_expr(source, provenance, callee_symbol_id)?;
            match annotated {
                NormExpr::Name { text, .. } => Ok(text.clone()),
                other => struct_field_name_from_expr(other, provenance, callee_symbol_id),
            }
        }
        other => Err(Diagnostic::hard_error(
            format!(
                "invalid struct syntax: expected a field binder name, found normalized target {other:#?}"
            ),
            Some(provenance.clone()),
        )
        .with_symbol_context(callee_symbol_id)),
    }
}

fn one_struct_member_expr<'a>(
    source: &'a lang_syntax::NormProduct,
    provenance: &Provenance,
    callee_symbol_id: SymbolId,
) -> Result<&'a NormExpr, Diagnostic> {
    let mut elements = source.elements.iter().filter_map(|element| match element {
        NormProductElem::Expr(expr) => Some(expr),
        NormProductElem::Unit { .. } => None,
    });
    let Some(expr) = elements.next() else {
        return Err(Diagnostic::hard_error(
            "invalid struct syntax: member visibility must annotate one field",
            Some(provenance.clone()),
        )
        .with_symbol_context(callee_symbol_id));
    };
    if elements.next().is_some() {
        return Err(Diagnostic::hard_error(
            "invalid struct syntax: member visibility annotated more than one field",
            Some(provenance.clone()),
        )
        .with_symbol_context(callee_symbol_id));
    }
    Ok(expr)
}

fn is_member_view_target(target: &NormExpr) -> bool {
    matches!(
        target,
        NormExpr::OperatorTarget { spelling, .. }
            if spelling == "[[public]]" || spelling == "[[private]]"
    )
}
