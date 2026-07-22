//! Formal meta invocation boundary.
//!
//! Consumes a `PreparedCallableCandidate` and dispatches to the appropriate
//! primitive invocation. This step is graph-installation-free and binding-free:
//! it produces a `MetaInvocationValue` but does **not** install
//! `NamespaceDelta`, bind declared symbols, or mutate the namespace graph. It
//! may allocate or attach registry-backed materialization state.
//!
//! ## Separation of concerns
//!
//! ```text
//! CandidatePrepResult::ApplicablePlaceholder
//!   → MetaInvocationInput
//!   → invoke_meta_callable
//!   → MetaInvocationValue  (no graph installation or binding)
//!
//! MetaInvocationValue
//!   → bind_meta_invocation_value_result (meta.rs)
//!   → MetaExpansionResult  (declaration binding, with NamespaceDelta)
//! ```
//!
//! `invoke_meta_callable_with_materialization_state` may populate
//! `TypeMaterializationState` for values whose semantic identity is
//! registry-backed, such as `GeneratedTypeDefinitionValue`. This is still
//! graph-installation-free with respect to the namespace graph. The cache stores
//! only replayable value material and strips concrete registry-backed
//! `PatternHeadId`s before insertion; cache hits rematerialize heads in the
//! caller's current state.
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
//! `MetaInvocationInput::placeholder_invocation_frame` records this boundary
//! without replacing the shortcut with full call-entry resolution.

use std::collections::BTreeSet;

use lang_syntax::{NormExpr, NormProductElem};

use crate::{
    extraction_view::{
        EvalResultNormalForm, ExposedExtractionInterface, ValuePointKind, ValuePointShape,
    },
    invocation_frame::{
        InvocationCallableRef, InvocationExecutionEnv, InvocationFrame, InvocationLookupEnv,
        SelfPosition,
    },
    meta_cache::MetaInstanceCache,
    meta_candidate::{CanonicalArgProductShapeMaterial, PreparedCallableCandidate},
    meta_key::{compute_meta_instance_key, MetaInstanceKey},
    model::{Diagnostic, Provenance, SymbolId},
    pattern_head::{
        PatternFieldMaterialization, PatternHeadId, PatternMaterializationContext,
        TypeMaterializationState,
    },
    pattern_space::{derive_sum_pattern_space, SumPatternSpaceShape, TypePatternExprShape},
    product_shape::{NonValueArgKind, ProductAtom, RawArgValueClass},
    struct_decoder::DecodedStructPattern,
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

    pub fn compute_key(&self) -> MetaInstanceKey {
        compute_meta_instance_key(&self.candidate)
    }

    /// Build the current placeholder invocation frame for substrate continuity.
    ///
    /// This does not change meta invocation behavior. Under the v0.8
    /// `temporary_direct_callable_shortcut`, the selected candidate symbol is
    /// used only as placeholder material for the invocation frame. Full target
    /// value → target type → `()` call-entry resolution is deferred. This does
    /// not claim that the callee symbol is the final self-object identity. This
    /// frame records the correct self/product boundary in the meantime.
    pub fn placeholder_invocation_frame(&self) -> Result<InvocationFrame, Diagnostic> {
        InvocationFrame::new(
            InvocationCallableRef::Symbol(self.candidate.callee_symbol_id),
            SelfPosition::placeholder_from_callable_symbol(
                self.candidate.callee_symbol_id,
                self.candidate.provenance.clone(),
            ),
            self.candidate.arg_product_shape.clone(),
            InvocationLookupEnv::new(self.candidate.policy_planes.lookup_env),
            InvocationExecutionEnv::new(self.candidate.policy_planes.demanded_execution),
            self.provenance.clone(),
        )
    }
}

impl MetaInvocationValue {
    /// Semantic equality: compares identity material without provenance.
    ///
    /// Two invocation values with the same semantic identity but
    /// different provenance compare equal here. This is distinct
    /// from `PartialEq` which includes provenance.
    pub fn semantic_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                MetaInvocationValue::ForwardedValue(lhs),
                MetaInvocationValue::ForwardedValue(rhs),
            ) => lhs.target == rhs.target && lhs.return_view == rhs.return_view,
            (
                MetaInvocationValue::GeneratedConstructionValue(lhs),
                MetaInvocationValue::GeneratedConstructionValue(rhs),
            ) => {
                lhs.construction_instance_id == rhs.construction_instance_id
                    && lhs.identity_material == rhs.identity_material
                    && lhs.return_view == rhs.return_view
            }
            (
                MetaInvocationValue::GeneratedTypeDefinitionValue(lhs),
                MetaInvocationValue::GeneratedTypeDefinitionValue(rhs),
            ) => {
                lhs.type_definition_id == rhs.type_definition_id
                    && lhs.identity_material == rhs.identity_material
                    && lhs.fields.len() == rhs.fields.len()
                    && lhs
                        .fields
                        .iter()
                        .zip(rhs.fields.iter())
                        .all(|(a, b)| a.semantic_eq(b))
                    && lhs.pattern_heads == rhs.pattern_heads
                    && lhs.return_view == rhs.return_view
                    && type_pattern_expr_semantic_eq(&lhs.type_pattern_expr, &rhs.type_pattern_expr)
                    && sum_pattern_space_semantic_eq(&lhs.sum_pattern_space, &rhs.sum_pattern_space)
            }
            _ => false,
        }
    }
}

fn type_pattern_expr_semantic_eq(
    lhs: &Option<crate::pattern_space::TypePatternExprShape>,
    rhs: &Option<crate::pattern_space::TypePatternExprShape>,
) -> bool {
    match (lhs, rhs) {
        (Some(l), Some(r)) => l.semantic_eq(r),
        (None, None) => true,
        _ => false,
    }
}

fn sum_pattern_space_semantic_eq(
    lhs: &Option<crate::pattern_space::SumPatternSpaceShape>,
    rhs: &Option<crate::pattern_space::SumPatternSpaceShape>,
) -> bool {
    match (lhs, rhs) {
        (Some(l), Some(r)) => l.semantic_eq(r),
        (None, None) => true,
        _ => false,
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
/// `ForwardedValue` is produced by the restricted evaluator's legacy
/// `IdentityType` forwarding proof. It is transitional transport, not the final
/// formal meta-return model. `GeneratedConstructionValue` is produced by
/// `UnaryConstructionPrototype` from argument-derived construction material.
/// `GeneratedTypeDefinitionValue` is produced by `struct` and is materialized
/// only by the binding layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetaInvocationValue {
    ForwardedValue(ForwardedValue),
    GeneratedConstructionValue(GeneratedConstructionValue),
    GeneratedTypeDefinitionValue(GeneratedTypeDefinitionValue),
}

impl MetaInvocationValue {
    pub fn return_normal_form_shape(&self) -> EvalResultNormalForm {
        match self {
            MetaInvocationValue::ForwardedValue(value) => {
                EvalResultNormalForm::ValuePoint(ValuePointShape {
                    value_kind: ValuePointKind::Forwarded {
                        target: value.target,
                    },
                    extraction_interface: ExposedExtractionInterface::Leaf,
                    provenance: value.provenance.clone(),
                })
            }
            MetaInvocationValue::GeneratedConstructionValue(value) => {
                EvalResultNormalForm::ValuePoint(ValuePointShape {
                    value_kind: ValuePointKind::GeneratedConstruction {
                        construction_instance_id: value.construction_instance_id,
                    },
                    extraction_interface: ExposedExtractionInterface::Leaf,
                    provenance: value.provenance.clone(),
                })
            }
            MetaInvocationValue::GeneratedTypeDefinitionValue(value) => {
                EvalResultNormalForm::ValuePoint(ValuePointShape {
                    value_kind: ValuePointKind::GeneratedTypeDefinition {
                        type_definition_id: value.type_definition_id,
                    },
                    extraction_interface: ExposedExtractionInterface::Leaf,
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
/// identity. Reserved for future generative type constructors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedConstructionValue {
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
pub struct GeneratedTypeDefinitionValue {
    pub type_definition_id: TypeDefinitionInstanceId,
    pub identity_material: TypeDefinitionIdentityMaterial,
    pub fields: Vec<GeneratedFieldDefinition>,
    pub pattern_heads: Option<TypeDefinitionPatternHeads>,
    pub return_view: ReturnViewShape,
    /// The decoded type-pattern expression shape, if the struct argument
    /// was successfully decoded by the struct-local decoder.
    pub type_pattern_expr: Option<TypePatternExprShape>,
    /// The sum pattern space derived from the type-pattern expression,
    /// if the expression contains a sum.
    pub sum_pattern_space: Option<SumPatternSpaceShape>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeDefinitionPatternHeads {
    pub owner_head: PatternHeadId,
    pub field_heads: Vec<GeneratedFieldPatternHead>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedFieldPatternHead {
    pub field_name: String,
    pub field_head: PatternHeadId,
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
    pub pattern_head: Option<PatternHeadId>,
    pub provenance: Provenance,
}

impl GeneratedFieldDefinition {
    /// Semantic equality: compares field identity material without provenance.
    pub fn semantic_eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.type_symbol_id == other.type_symbol_id
            && self.index == other.index
            && self.pattern_head == other.pattern_head
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
pub fn invoke_meta_callable(input: MetaInvocationInput) -> MetaInvocationResult {
    // Standalone compatibility entry point. It is suitable for one-off formal
    // invocation tests but does not preserve PatternHeadRegistry continuity
    // across calls. Callers that need registry-backed identity continuity must
    // use `invoke_meta_callable_with_materialization_state`.
    let mut materialization_state = TypeMaterializationState::default();
    invoke_meta_callable_with_materialization_state(input, &mut materialization_state)
}

pub fn invoke_meta_callable_with_materialization_state(
    input: MetaInvocationInput,
    materialization_state: &mut TypeMaterializationState,
) -> MetaInvocationResult {
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
        crate::model::CoreMetaFunction::Struct => {
            invoke_struct_type_definition(&input, materialization_state)
        }
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
    // Standalone compatibility entry point. Cache hits for registry-backed
    // values are rehydrated into this temporary state only; callers that need
    // to query the registry later must use the `_with_materialization_state`
    // variant.
    let mut materialization_state = TypeMaterializationState::default();
    invoke_meta_callable_cached_with_materialization_state(input, cache, &mut materialization_state)
}

pub fn invoke_meta_callable_cached_with_materialization_state(
    input: MetaInvocationInput,
    cache: &mut MetaInstanceCache,
    materialization_state: &mut TypeMaterializationState,
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
        return cached_value_for_current_materialization_state(
            cached.result.clone(),
            materialization_state,
            input.provenance,
        );
    }
    let result = invoke_meta_callable_with_materialization_state(input, materialization_state);
    if let MetaInvocationResult::Value(ref val) = result {
        cache.insert(
            key,
            cacheable_invocation_value(val.clone()),
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

fn invoke_struct_type_definition(
    input: &MetaInvocationInput,
    materialization_state: &mut TypeMaterializationState,
) -> MetaInvocationResult {
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
            pattern_head: None,
            provenance: field.provenance.clone(),
        })
        .collect();
    let value = GeneratedTypeDefinitionValue {
        type_definition_id,
        identity_material,
        fields,
        pattern_heads: None,
        return_view: ReturnViewShape::Leaf,
        type_pattern_expr: input
            .struct_decoded_pattern
            .as_ref()
            .map(|p| p.type_pattern_expr.clone()),
        sum_pattern_space: input
            .struct_decoded_pattern
            .as_ref()
            .and_then(|p| derive_sum_pattern_space(&p.type_pattern_expr)),
        provenance: input.provenance.clone(),
    };
    match attach_type_definition_pattern_heads(
        value,
        materialization_state,
        input.provenance.clone(),
    ) {
        Ok(value) => {
            MetaInvocationResult::Value(MetaInvocationValue::GeneratedTypeDefinitionValue(value))
        }
        Err(diagnostic) => MetaInvocationResult::Diagnostic(diagnostic),
    }
}

fn cached_value_for_current_materialization_state(
    value: MetaInvocationValue,
    materialization_state: &mut TypeMaterializationState,
    provenance: Provenance,
) -> MetaInvocationResult {
    match value {
        MetaInvocationValue::GeneratedTypeDefinitionValue(value) => {
            match attach_type_definition_pattern_heads(value, materialization_state, provenance) {
                Ok(value) => MetaInvocationResult::Value(
                    MetaInvocationValue::GeneratedTypeDefinitionValue(value),
                ),
                Err(diagnostic) => MetaInvocationResult::Diagnostic(diagnostic),
            }
        }
        other => MetaInvocationResult::Value(other),
    }
}

fn cacheable_invocation_value(value: MetaInvocationValue) -> MetaInvocationValue {
    match value {
        MetaInvocationValue::GeneratedTypeDefinitionValue(mut value) => {
            value.pattern_heads = None;
            for field in &mut value.fields {
                field.pattern_head = None;
            }
            MetaInvocationValue::GeneratedTypeDefinitionValue(value)
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
pub fn attach_type_definition_pattern_heads(
    value: GeneratedTypeDefinitionValue,
    materialization_state: &mut TypeMaterializationState,
    provenance: Provenance,
) -> Result<GeneratedTypeDefinitionValue, Diagnostic> {
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
    attach_type_definition_pattern_heads_with_context(
        value,
        materialization_state,
        PatternMaterializationContext::GeneratedTypeDefinition { type_definition_id },
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
/// The display name is diagnostic material only. The owner `PatternHeadId`
/// identity comes from `context`; callers must not derive identity from the
/// bare source spelling.
#[doc(hidden)]
pub fn attach_type_definition_pattern_heads_with_context(
    mut value: GeneratedTypeDefinitionValue,
    materialization_state: &mut TypeMaterializationState,
    context: PatternMaterializationContext,
    owner_display_name: impl Into<String>,
    provenance: Provenance,
) -> Result<GeneratedTypeDefinitionValue, Diagnostic> {
    let pattern_fields = value
        .identity_material
        .field_signature_material
        .iter()
        .map(|field| PatternFieldMaterialization {
            field_name: field.field_name.clone(),
            field_type_symbol_id: field.field_type_symbol_id,
            projection: crate::model::FieldProjection::Value,
            provenance: field.provenance.clone(),
        });
    let pattern_materialization = materialization_state
        .pattern_heads
        .materialize_struct_pattern_heads(
            context,
            owner_display_name.into(),
            pattern_fields,
            provenance,
        )?;
    let pattern_head_by_name = pattern_materialization
        .field_heads
        .iter()
        .cloned()
        .collect::<std::collections::BTreeMap<_, _>>();
    for field in &mut value.fields {
        field.pattern_head = pattern_head_by_name.get(&field.name).copied();
    }
    value.pattern_heads = Some(TypeDefinitionPatternHeads {
        owner_head: pattern_materialization.owner_head,
        field_heads: pattern_materialization
            .field_heads
            .into_iter()
            .map(|(field_name, field_head)| GeneratedFieldPatternHead {
                field_name,
                field_head,
            })
            .collect(),
    });
    Ok(value)
}

fn owner_display_name_from_type_pattern_expr(expr: &TypePatternExprShape) -> Option<String> {
    match expr {
        TypePatternExprShape::Named { pattern_name, .. } => Some(pattern_name.clone()),
        _ => None,
    }
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
