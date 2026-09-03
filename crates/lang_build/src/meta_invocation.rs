//! Formal meta invocation boundary.
//!
//! Consumes a `PreparedCallableCandidate` and dispatches to the appropriate
//! primitive invocation. This step is graph-installation-free and binding-free:
//! it produces a `MetaExecutionMaterial` but does **not** install
//! `NamespaceDelta`, bind declared symbols, or mutate the namespace graph. It
//! does not allocate graph or Pattern-relation state.
//!
//! ## Separation of concerns
//!
//! ```text
//! CandidatePrepResult::Applicable
//!   → MetaInvocationInput
//!   → invoke_meta_callable
//!   → MetaPrimitiveExecution::Material(MetaExecutionMaterial)
//!     (no semantic result, graph installation, or binding)
//!
//! MetaExecutionMaterial
//!   → bind_meta_invocation_value_result (meta.rs)
//!   → namespace installation material
//! ```
//!
//! Production invocation reaches this primitive executor only after ordinary
//! value → complete type → associated `()` resolution has selected a call-entry
//! semantic value. The implicit `self` belongs to that invocation frame, never
//! to `ProductObject` / `ArgProductShape` / `RawArgShape`.

use std::collections::{BTreeMap, BTreeSet};

use lang_syntax::{NormExpr, NormProductElem};

use crate::{
    meta_candidate::{CanonicalArgProductShapeMaterial, PreparedCallableCandidate},
    model::{Diagnostic, Provenance, SymbolId},
    product_shape::{NonValueArgKind, ProductAtom, RawArgValueClass},
    struct_decoder::DecodedStructPattern,
    struct_pattern_material::{StructPatternSyntaxMaterial, StructuralMemberVisibility},
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

impl StructConstructionMaterial {
    /// The struct construction material's structural Pattern normal form.
    ///
    /// Construction callable identity, build identity, export metadata,
    /// provenance, and the FNV-derived construction material id are deliberately
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
        canonical_struct_body_pattern(self.type_pattern_expr.as_ref(), &self.fields)
    }

    /// Mirror a successful incremental pure-P contribution on the returned
    /// construction artifact.  The SemanticWorld PatternValue remains the
    /// authority; this copy ensures later binding/materialization observes
    /// the same completed Pattern rather than the original one-shot body.
    pub fn set_canonical_pattern_value(&mut self, value: crate::CanonicalPatternValue) {
        self.canonical_pattern_override = Some(value);
    }
}

fn canonical_struct_body_pattern(
    pattern: Option<&StructPatternSyntaxMaterial>,
    fields: &[StructFieldConstructionMaterial],
) -> crate::CanonicalPatternValue {
    if let Some(pattern) = pattern {
        let field_types = fields
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
        fields
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

/// Replayable execution material produced behind the unified invocation
/// result boundary.
///
/// `IdentityTypeMaterial` records an `IdentityType` proof for later result
/// formation.
/// `StructConstructionMaterial` is the replayable construction material
/// produced while evaluating `struct`; it is not the semantic result of that
/// callable.  The world-connected invocation path installs the material and
/// returns a complete type value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetaExecutionMaterial {
    IdentityType(IdentityTypeMaterial),
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
    pub type_observation: crate::CanonicalTypeObservation,
    pub provenance: Provenance,
}

/// Replayable normalized body material produced by `struct` execution.
///
/// This is graph-installation-free and binding-free. The declared type Symbol,
/// meta-instance root, associated namespace, and field projections are formed
/// at their respective semantic boundaries and never enter body identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructConstructionMaterial {
    /// Normalized struct body identity.  This is body material only:
    /// it never carries a canonical root by itself.  Two different meta
    /// functions with identical bodies share this id while their canonical
    /// TypeValue roots stay distinct.
    pub material_id: StructConstructionMaterialId,
    pub identity_material: StructConstructionIdentityMaterial,
    pub fields: Vec<StructFieldConstructionMaterial>,
    /// The decoded type-pattern expression shape, if the struct argument
    /// was successfully decoded by the struct-local decoder.
    pub type_pattern_expr: Option<StructPatternSyntaxMaterial>,
    /// Canonical semantic TypeValue root assigned at meta-instance
    /// registration: `TypeValue = (OuterMetaInstanceRoot,
    /// NormalizedStructBody)`.  `None` until the invocation owner
    /// registers the member. Parent-neutral cached body material never
    /// retains another meta-instance root.
    pub canonical_type: Option<crate::TypeValueId>,
    /// Updated canonical Pattern material after incremental pure-P
    /// contributions. `None` means the normal form is derived directly from
    /// the decoded `struct` body.
    pub canonical_pattern_override: Option<crate::CanonicalPatternValue>,
    pub provenance: Provenance,
}

/// Build-local identifier for replayable struct construction material.
///
/// This is neither complete-type identity nor meta-instance identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StructConstructionMaterialId(pub u64);

impl StructConstructionMaterialId {
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

/// Material that determines replayable struct-construction identity.
///
/// `provenance` is diagnostic material and is excluded from equality and
/// identity computation.
#[derive(Clone, Debug)]
pub struct StructConstructionIdentityMaterial {
    pub canonical_pattern: crate::CanonicalPatternValue,
    pub field_signature_material: Vec<FieldSignatureMaterial>,
    pub provenance: Provenance,
}

impl PartialEq for StructConstructionIdentityMaterial {
    fn eq(&self, other: &Self) -> bool {
        self.canonical_pattern == other.canonical_pattern
            && self.field_signature_material == other.field_signature_material
    }
}

impl Eq for StructConstructionIdentityMaterial {}

#[derive(Clone, Debug)]
pub struct FieldSignatureMaterial {
    pub field_name: String,
    /// First-order field type projection.  Transport/registry material only:
    /// canonical equality and instance identity consume
    /// `field_type_observation`, never this bare projection and never the
    /// carrier name used in source.
    pub field_type_value: crate::TypeValueId,
    /// The field type's observation identity — `Addr(Norm_type)` including
    /// the recursive Val2 read at the argument's carrier place.
    pub field_type_observation: crate::CanonicalTypeObservation,
    /// Graph projection carrier used when installing the field namespace.
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
pub struct StructFieldConstructionMaterial {
    pub name: String,
    pub type_value: crate::TypeValueId,
    /// The field type's observation identity; semantic equality consumes
    /// this, never the bare `type_value` projection.
    pub type_observation: crate::CanonicalTypeObservation,
    pub type_carrier_symbol: SymbolId,
    pub index: usize,
    pub visibility: StructuralMemberVisibility,
    pub provenance: Provenance,
}

pub fn compute_struct_construction_material_id(
    material: &StructConstructionIdentityMaterial,
) -> StructConstructionMaterialId {
    use crate::fingerprint::Fnv1a64;
    let mut h = Fnv1a64::new();
    h.write_str_field("struct-construction-material");
    hash_canonical_pattern(&mut h, &material.canonical_pattern);
    h.write_field(&(material.field_signature_material.len() as u64).to_le_bytes());
    for field in &material.field_signature_material {
        h.write_str_field(&field.field_name);
        let crate::CanonicalTypeObservation::Observed(addr) = field.field_type_observation;
        h.write_field(&addr.0.to_le_bytes());
        h.write_field(&(field.field_index as u64).to_le_bytes());
        h.write_field(&[match field.visibility {
            StructuralMemberVisibility::Default => 0,
            StructuralMemberVisibility::Public => 1,
            StructuralMemberVisibility::Private => 2,
        }]);
    }
    let raw = u64::from_str_radix(&h.finish_hex(), 16)
        .expect("Fnv1a64::finish_hex must produce a valid u64 hex string");
    StructConstructionMaterialId(if raw == 0 { 1 } else { raw })
}

fn hash_navigation(
    h: &mut crate::fingerprint::Fnv1a64,
    navigation: &crate::CanonicalFullNavigation,
) {
    h.write_field(&(navigation.components().len() as u64).to_le_bytes());
    for component in navigation.components() {
        h.write_str_field(component);
    }
}

fn hash_canonical_pattern(
    h: &mut crate::fingerprint::Fnv1a64,
    pattern: &crate::CanonicalPatternValue,
) {
    use crate::{CanonicalPatternAtom, CanonicalPatternValue, CanonicalTypeObservation};
    match pattern {
        CanonicalPatternValue::Atom(CanonicalPatternAtom::Type(
            CanonicalTypeObservation::Observed(addr),
        )) => {
            h.write_field(&[0]);
            h.write_field(&addr.0.to_le_bytes());
        }
        CanonicalPatternValue::Atom(CanonicalPatternAtom::Unit) => h.write_field(&[1]),
        CanonicalPatternValue::NamedPattern { navigation, body } => {
            h.write_field(&[2]);
            hash_navigation(h, navigation);
            hash_canonical_pattern(h, body);
        }
        CanonicalPatternValue::OrderedLayer(entries) => {
            h.write_field(&[3]);
            h.write_field(&(entries.len() as u64).to_le_bytes());
            for entry in entries {
                match &entry.navigation {
                    Some(navigation) => {
                        h.write_field(&[1]);
                        hash_navigation(h, navigation);
                    }
                    None => h.write_field(&[0]),
                }
                hash_canonical_pattern(h, &entry.value);
            }
        }
        CanonicalPatternValue::UnorderedLayer(entries) => {
            h.write_field(&[4]);
            h.write_field(&(entries.len() as u64).to_le_bytes());
            for (navigation, value) in entries {
                hash_navigation(h, navigation);
                hash_canonical_pattern(h, value);
            }
        }
        CanonicalPatternValue::Sum(alternatives) => {
            h.write_field(&[5]);
            h.write_field(&(alternatives.len() as u64).to_le_bytes());
            for alternative in alternatives {
                hash_canonical_pattern(h, alternative);
            }
        }
        CanonicalPatternValue::Hole(hole) => {
            h.write_field(&[6]);
            h.write_field(&hole.to_le_bytes());
        }
    }
}

pub(crate) fn invoke_meta_callable(input: MetaInvocationInput) -> MetaPrimitiveExecution {
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
        crate::model::CoreMetaFunction::Struct => invoke_struct_construction(&input),
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
                    "IdentityType: argument is not a classified complete type value",
                    Some(input.provenance.clone()),
                )
                .with_symbol_context(candidate.callee_symbol_id),
            );
        }
    };
    let Some(type_observation) = candidate
        .arg_product_shape
        .raw_args
        .first()
        .and_then(|raw| {
            raw.known_complete_type_observation
                .map(crate::CanonicalTypeObservation::Observed)
                .or_else(|| raw.type_observation())
        })
    else {
        return MetaPrimitiveExecution::Diagnostic(
            Diagnostic::hard_error(
                "IdentityType requires an exact canonical type observation",
                Some(input.provenance.clone()),
            )
            .with_symbol_context(candidate.callee_symbol_id),
        );
    };

    MetaPrimitiveExecution::Material(MetaExecutionMaterial::IdentityType(IdentityTypeMaterial {
        type_value,
        type_observation,
        provenance: input.provenance.clone(),
    }))
}

fn invoke_struct_construction(input: &MetaInvocationInput) -> MetaPrimitiveExecution {
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

    let fields = field_signature_material
        .iter()
        .map(|field| StructFieldConstructionMaterial {
            name: field.field_name.clone(),
            type_value: field.field_type_value,
            type_observation: field.field_type_observation,
            type_carrier_symbol: field.field_type_carrier_symbol,
            index: field.field_index,
            visibility: field.visibility,
            provenance: field.provenance.clone(),
        })
        .collect::<Vec<_>>();
    let type_pattern_expr = input
        .struct_decoded_pattern
        .as_ref()
        .map(|pattern| pattern.type_pattern_expr.clone());
    let identity_material = StructConstructionIdentityMaterial {
        canonical_pattern: canonical_struct_body_pattern(type_pattern_expr.as_ref(), &fields),
        field_signature_material: field_signature_material.clone(),
        provenance: input.provenance.clone(),
    };
    let material_id = compute_struct_construction_material_id(&identity_material);
    let value = StructConstructionMaterial {
        material_id,
        identity_material,
        fields,
        type_pattern_expr,
        canonical_type: None,
        canonical_pattern_override: None,
        provenance: input.provenance.clone(),
    };
    MetaPrimitiveExecution::Material(MetaExecutionMaterial::StructConstructionMaterial(value))
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
        let Some(field_type_observation) = raw_arg.type_observation() else {
            return Err(Diagnostic::hard_error(
                format!("struct field `{field_name}` requires an exact canonical type observation"),
                Some(field_provenance),
            )
            .with_symbol_context(candidate.callee_symbol_id));
        };
        fields.push(FieldSignatureMaterial {
            field_name,
            field_type_value: type_value,
            field_type_observation,
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
            "invalid struct syntax: nested product fields are not supported by the struct decoder",
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
                "invalid struct syntax: nested product fields are not supported by the struct decoder",
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
