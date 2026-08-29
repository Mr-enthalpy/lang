//! Return normal form and extraction-view substrate.
//!
//! This module is shape-only. It does not evaluate full expressions, perform
//! destructuring, or install namespace graph material. Product normal form `P`
//! is represented directly; it is not wrapped in a non-product call value.

use crate::{
    identity::TypeValueId,
    meta_invocation::TypeDefinitionInstanceId,
    model::{Diagnostic, FieldProjection, Provenance, SymbolId},
    struct_pattern_registry::StructPatternMaterialId,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservedArgumentContent {
    ValuePoint(ObservedAtomContent),
    Product(ObservedProductContent),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedAtomContent {
    pub value_kind: ObservedAtomKind,
    pub extraction_interface: ContentObservationInterface,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservedAtomKind {
    Leaf,
    Constructed {
        owner_type_value: Option<TypeValueId>,
        /// Graph projection carrier only.
        owner_type_symbol_id: Option<SymbolId>,
    },
    Forwarded {
        type_value: TypeValueId,
    },
    StructConstruction {
        type_definition_id: TypeDefinitionInstanceId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedProductContent {
    pub elements: Vec<ObservedProductElement>,
    pub product_kind: ObservedProductKind,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservedProductKind {
    Bare,
    Named {
        owner_type_value: Option<TypeValueId>,
        /// Graph projection carrier only.
        owner_type_symbol_id: Option<SymbolId>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedProductElement {
    pub label: Option<String>,
    pub value_shape: Box<ObservedArgumentContent>,
    /// Evaluated first-order type projection of this element.
    /// Transport/navigation material only: semantic equality consumes
    /// `type_observation`.
    pub type_value: Option<TypeValueId>,
    /// The element type's observation identity. Semantic equality consumes
    /// this, never the bare `type_value`.
    pub type_observation: Option<crate::CanonicalTypeObservation>,
    /// Graph projection carrier for navigation/provenance only.
    pub type_symbol_id: Option<SymbolId>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContentObservationInterface {
    Leaf,
    Product(ObservedProductContent),
    NamedProduct(NamedObservedProduct),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedObservedProduct {
    pub owner_type_value: TypeValueId,
    /// Graph projection carrier used to navigate installed projections.
    pub owner_type_symbol_id: SymbolId,
    pub owner_struct_pattern_registry: Option<StructPatternMaterialId>,
    pub fields: Vec<NamedObservedField>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedObservedField {
    pub label: String,
    /// Evaluated first-order field type projection. Transport/registry
    /// material only: extraction-shape semantic equality consumes
    /// `field_type_observation`, not this projection and not the source
    /// carrier Symbol.
    pub field_type_value: TypeValueId,
    /// The field type's observation identity — `Addr(Norm_type)` including
    /// the recursive Val2 read at the classifying boundary.
    pub field_type_observation: crate::CanonicalTypeObservation,
    /// Graph projection carrier for current namespace projection.
    pub field_type_symbol_id: SymbolId,
    pub field_struct_pattern_registry: Option<StructPatternMaterialId>,
    pub field_index: usize,
    pub projection: FieldProjection,
    pub visibility: crate::struct_pattern_material::StructuralMemberVisibility,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeContentObservation {
    pub owner_type_value: TypeValueId,
    pub owner_type_symbol_id: SymbolId,
    pub owner_struct_pattern_registry: Option<StructPatternMaterialId>,
    pub exposed_view: NamedObservedProduct,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservedContentProjection {
    NormalForm(ObservedArgumentContent),
    Diagnostic(Diagnostic),
}

pub fn observe_content_projection(shape: &ObservedArgumentContent) -> ObservedContentProjection {
    match shape {
        ObservedArgumentContent::Product(product) => {
            ObservedContentProjection::NormalForm(ObservedArgumentContent::Product(product.clone()))
        }
        ObservedArgumentContent::ValuePoint(value) => match &value.extraction_interface {
            ContentObservationInterface::Leaf => ObservedContentProjection::NormalForm(
                ObservedArgumentContent::ValuePoint(value.clone()),
            ),
            ContentObservationInterface::Product(product) => ObservedContentProjection::NormalForm(
                ObservedArgumentContent::Product(product.clone()),
            ),
            ContentObservationInterface::NamedProduct(named) => {
                ObservedContentProjection::NormalForm(ObservedArgumentContent::Product(
                    named_product_to_product_normal_form(named),
                ))
            }
        },
    }
}

pub fn named_product_to_product_normal_form(
    named: &NamedObservedProduct,
) -> ObservedProductContent {
    ObservedProductContent {
        elements: named
            .fields
            .iter()
            .map(|field| ObservedProductElement {
                label: Some(field.label.clone()),
                value_shape: Box::new(ObservedArgumentContent::ValuePoint(ObservedAtomContent {
                    value_kind: ObservedAtomKind::Leaf,
                    extraction_interface: ContentObservationInterface::Leaf,
                    provenance: field.provenance.clone(),
                })),
                type_value: Some(field.field_type_value),
                type_observation: Some(field.field_type_observation),
                type_symbol_id: Some(field.field_type_symbol_id),
                provenance: field.provenance.clone(),
            })
            .collect(),
        product_kind: ObservedProductKind::Named {
            owner_type_value: Some(named.owner_type_value),
            owner_type_symbol_id: Some(named.owner_type_symbol_id),
        },
        provenance: named.provenance.clone(),
    }
}

// Observation equality compares captured content without provenance. Pattern
// applicability and extraction are owned exclusively by `pattern_relation`.

impl ObservedArgumentContent {
    pub fn observationally_equal(&self, other: &Self) -> bool {
        match (self, other) {
            (ObservedArgumentContent::ValuePoint(v1), ObservedArgumentContent::ValuePoint(v2)) => {
                v1.observationally_equal(v2)
            }
            (ObservedArgumentContent::Product(p1), ObservedArgumentContent::Product(p2)) => {
                p1.observationally_equal(p2)
            }
            _ => false,
        }
    }
}

impl ObservedAtomContent {
    pub fn observationally_equal(&self, other: &Self) -> bool {
        value_point_kind_observationally_equal(&self.value_kind, &other.value_kind)
            && self
                .extraction_interface
                .observationally_equal(&other.extraction_interface)
    }
}

fn value_point_kind_observationally_equal(
    left: &ObservedAtomKind,
    right: &ObservedAtomKind,
) -> bool {
    match (left, right) {
        (ObservedAtomKind::Leaf, ObservedAtomKind::Leaf) => true,
        (
            ObservedAtomKind::Constructed {
                owner_type_value: left,
                ..
            },
            ObservedAtomKind::Constructed {
                owner_type_value: right,
                ..
            },
        ) => left == right,
        (
            ObservedAtomKind::Forwarded { type_value: left },
            ObservedAtomKind::Forwarded { type_value: right },
        ) => left == right,
        (
            ObservedAtomKind::StructConstruction {
                type_definition_id: left,
            },
            ObservedAtomKind::StructConstruction {
                type_definition_id: right,
            },
        ) => left == right,
        _ => false,
    }
}

impl ContentObservationInterface {
    pub fn observationally_equal(&self, other: &Self) -> bool {
        match (self, other) {
            (ContentObservationInterface::Leaf, ContentObservationInterface::Leaf) => true,
            (
                ContentObservationInterface::Product(p1),
                ContentObservationInterface::Product(p2),
            ) => p1.observationally_equal(p2),
            (
                ContentObservationInterface::NamedProduct(n1),
                ContentObservationInterface::NamedProduct(n2),
            ) => n1.observationally_equal(n2),
            _ => false,
        }
    }
}

impl ObservedProductContent {
    pub fn observationally_equal(&self, other: &Self) -> bool {
        product_kind_observationally_equal(&self.product_kind, &other.product_kind)
            && self.elements.len() == other.elements.len()
            && self
                .elements
                .iter()
                .zip(other.elements.iter())
                .all(|(a, b)| a.observationally_equal(b))
    }
}

fn product_kind_observationally_equal(
    left: &ObservedProductKind,
    right: &ObservedProductKind,
) -> bool {
    match (left, right) {
        (ObservedProductKind::Bare, ObservedProductKind::Bare) => true,
        (
            ObservedProductKind::Named {
                owner_type_value: left,
                ..
            },
            ObservedProductKind::Named {
                owner_type_value: right,
                ..
            },
        ) => left == right,
        _ => false,
    }
}

impl ObservedProductElement {
    pub fn observationally_equal(&self, other: &Self) -> bool {
        self.label == other.label
            && self.value_shape.observationally_equal(&other.value_shape)
            && self.type_observation == other.type_observation
    }
}

impl NamedObservedProduct {
    pub fn observationally_equal(&self, other: &Self) -> bool {
        self.owner_type_value == other.owner_type_value
            && self.owner_struct_pattern_registry == other.owner_struct_pattern_registry
            && self.fields.len() == other.fields.len()
            && self
                .fields
                .iter()
                .zip(other.fields.iter())
                .all(|(a, b)| a.observationally_equal(b))
    }
}

impl NamedObservedField {
    pub fn observationally_equal(&self, other: &Self) -> bool {
        self.label == other.label
            && self.field_type_observation == other.field_type_observation
            && self.field_struct_pattern_registry == other.field_struct_pattern_registry
            && self.field_index == other.field_index
            && self.projection == other.projection
            && self.visibility == other.visibility
    }
}
