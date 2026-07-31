//! Return normal form and extraction-view substrate.
//!
//! This module is shape-only. It does not evaluate full expressions, perform
//! destructuring, or install namespace graph material. Product normal form `P`
//! is represented directly; it is not wrapped in a non-product call value.

use crate::{
    identity::TypeValueId,
    meta_invocation::{ConstructionInstanceId, TypeDefinitionInstanceId},
    model::{Diagnostic, FieldProjection, Provenance, SymbolId},
    pattern_head::PatternHeadId,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EvalResultNormalForm {
    ValuePoint(ValuePointShape),
    Product(ProductNormalFormShape),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValuePointShape {
    pub value_kind: ValuePointKind,
    pub extraction_interface: ExposedExtractionInterface,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValuePointKind {
    Leaf,
    Constructed {
        owner_type_value: Option<TypeValueId>,
        /// Compatibility graph carrier only.
        owner_type_symbol_id: Option<SymbolId>,
    },
    Forwarded {
        type_value: TypeValueId,
    },
    GeneratedConstruction {
        construction_instance_id: ConstructionInstanceId,
    },
    GeneratedTypeDefinition {
        type_definition_id: TypeDefinitionInstanceId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductNormalFormShape {
    pub elements: Vec<ProductNormalFormElem>,
    pub product_kind: ProductNormalFormKind,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProductNormalFormKind {
    Bare,
    Named {
        owner_type_value: Option<TypeValueId>,
        /// Compatibility graph carrier only.
        owner_type_symbol_id: Option<SymbolId>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductNormalFormElem {
    pub label: Option<String>,
    pub value_shape: Box<EvalResultNormalForm>,
    /// Evaluated first-order type projection of this element.
    /// Transport/navigation material only: semantic equality consumes
    /// `type_observation`.
    pub type_value: Option<TypeValueId>,
    /// The element type's observation identity — `Addr(Norm_type)` when the
    /// producing boundary was world-connected, otherwise the `Detached`
    /// projection.  Semantic equality consumes this, never the bare
    /// `type_value`.
    pub type_observation: Option<crate::CanonicalTypeObservation>,
    /// Compatibility graph carrier for navigation/provenance only.
    pub type_symbol_id: Option<SymbolId>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExposedExtractionInterface {
    Leaf,
    Product(ProductNormalFormShape),
    NamedProduct(NamedProductExtractionShape),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedProductExtractionShape {
    pub owner_type_value: TypeValueId,
    /// Compatibility graph carrier used to navigate installed projections.
    pub owner_type_symbol_id: SymbolId,
    pub owner_pattern_head: Option<PatternHeadId>,
    pub fields: Vec<NamedExtractionField>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedExtractionField {
    pub label: String,
    /// Evaluated first-order field type projection. Transport/registry
    /// material only: extraction-shape semantic equality consumes
    /// `field_type_observation`, not this projection and not the source
    /// carrier Symbol.
    pub field_type_value: TypeValueId,
    /// The field type's observation identity — `Addr(Norm_type)` including
    /// the recursive Val2 read at the classifying boundary, otherwise the
    /// `Detached` projection.  Semantic equality consumes this.
    pub field_type_observation: crate::CanonicalTypeObservation,
    /// Compatibility graph carrier for current namespace projection.
    pub field_type_symbol_id: SymbolId,
    pub field_pattern_head: Option<PatternHeadId>,
    pub field_index: usize,
    pub projection: FieldProjection,
    pub visibility: crate::pattern_space::StructuralMemberVisibility,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeExtractionInterface {
    pub owner_type_value: TypeValueId,
    pub owner_type_symbol_id: SymbolId,
    pub owner_pattern_head: Option<PatternHeadId>,
    pub exposed_view: NamedProductExtractionShape,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExtractionViewResult {
    NormalForm(EvalResultNormalForm),
    Diagnostic(Diagnostic),
}

pub fn question_view(shape: &EvalResultNormalForm) -> ExtractionViewResult {
    match shape {
        EvalResultNormalForm::Product(product) => {
            ExtractionViewResult::NormalForm(EvalResultNormalForm::Product(product.clone()))
        }
        EvalResultNormalForm::ValuePoint(value) => match &value.extraction_interface {
            ExposedExtractionInterface::Leaf => {
                ExtractionViewResult::NormalForm(EvalResultNormalForm::ValuePoint(value.clone()))
            }
            ExposedExtractionInterface::Product(product) => {
                ExtractionViewResult::NormalForm(EvalResultNormalForm::Product(product.clone()))
            }
            ExposedExtractionInterface::NamedProduct(named) => ExtractionViewResult::NormalForm(
                EvalResultNormalForm::Product(named_product_to_product_normal_form(named)),
            ),
        },
    }
}

pub fn named_product_to_product_normal_form(
    named: &NamedProductExtractionShape,
) -> ProductNormalFormShape {
    ProductNormalFormShape {
        elements: named
            .fields
            .iter()
            .map(|field| ProductNormalFormElem {
                label: Some(field.label.clone()),
                value_shape: Box::new(EvalResultNormalForm::ValuePoint(ValuePointShape {
                    value_kind: ValuePointKind::Leaf,
                    extraction_interface: ExposedExtractionInterface::Leaf,
                    provenance: field.provenance.clone(),
                })),
                type_value: Some(field.field_type_value),
                type_observation: Some(field.field_type_observation),
                type_symbol_id: Some(field.field_type_symbol_id),
                provenance: field.provenance.clone(),
            })
            .collect(),
        product_kind: ProductNormalFormKind::Named {
            owner_type_value: Some(named.owner_type_value),
            owner_type_symbol_id: Some(named.owner_type_symbol_id),
        },
        provenance: named.provenance.clone(),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindingPatternShape {
    Binder,
    Product { arity: usize, named: bool },
    NamedProduct { labels: Vec<String> },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindingShapeMatchResult {
    Direct,
    AfterExtraction,
    Mismatch,
}

pub fn match_binding_pattern_shape(
    pattern: &BindingPatternShape,
    value: &EvalResultNormalForm,
) -> BindingShapeMatchResult {
    if matches!(pattern, BindingPatternShape::Binder) {
        return BindingShapeMatchResult::Direct;
    }

    if let EvalResultNormalForm::Product(product) = value {
        return if product_matches_pattern(pattern, product) {
            BindingShapeMatchResult::Direct
        } else {
            BindingShapeMatchResult::Mismatch
        };
    }

    let EvalResultNormalForm::ValuePoint(value_point) = value else {
        unreachable!("all EvalResultNormalForm variants handled above");
    };
    match &value_point.extraction_interface {
        ExposedExtractionInterface::Leaf => BindingShapeMatchResult::Mismatch,
        ExposedExtractionInterface::Product(product) => {
            if product_matches_pattern(pattern, product) {
                BindingShapeMatchResult::AfterExtraction
            } else {
                BindingShapeMatchResult::Mismatch
            }
        }
        ExposedExtractionInterface::NamedProduct(named) => {
            let product = named_product_to_product_normal_form(named);
            if product_matches_pattern(pattern, &product) {
                BindingShapeMatchResult::AfterExtraction
            } else {
                BindingShapeMatchResult::Mismatch
            }
        }
    }
}

fn product_matches_pattern(
    pattern: &BindingPatternShape,
    product: &ProductNormalFormShape,
) -> bool {
    match pattern {
        BindingPatternShape::Binder => true,
        BindingPatternShape::Product { arity, named } => {
            product.elements.len() == *arity
                && if *named {
                    product.elements.iter().all(|elem| elem.label.is_some())
                } else {
                    matches!(product.product_kind, ProductNormalFormKind::Bare)
                }
        }
        BindingPatternShape::NamedProduct { labels } => {
            product.elements.len() == labels.len()
                && product
                    .elements
                    .iter()
                    .zip(labels)
                    .all(|(elem, label)| elem.label.as_deref() == Some(label.as_str()))
        }
    }
}

// ---------------------------------------------------------------------------
// Semantic equality helpers — compare shape identity without provenance
// ---------------------------------------------------------------------------

impl EvalResultNormalForm {
    pub fn semantic_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (EvalResultNormalForm::ValuePoint(v1), EvalResultNormalForm::ValuePoint(v2)) => {
                v1.semantic_eq(v2)
            }
            (EvalResultNormalForm::Product(p1), EvalResultNormalForm::Product(p2)) => {
                p1.semantic_eq(p2)
            }
            _ => false,
        }
    }
}

impl ValuePointShape {
    pub fn semantic_eq(&self, other: &Self) -> bool {
        value_point_kind_semantic_eq(&self.value_kind, &other.value_kind)
            && self
                .extraction_interface
                .semantic_eq(&other.extraction_interface)
    }
}

fn value_point_kind_semantic_eq(left: &ValuePointKind, right: &ValuePointKind) -> bool {
    match (left, right) {
        (ValuePointKind::Leaf, ValuePointKind::Leaf) => true,
        (
            ValuePointKind::Constructed {
                owner_type_value: left,
                ..
            },
            ValuePointKind::Constructed {
                owner_type_value: right,
                ..
            },
        ) => left == right,
        (
            ValuePointKind::Forwarded { type_value: left },
            ValuePointKind::Forwarded { type_value: right },
        ) => left == right,
        (
            ValuePointKind::GeneratedConstruction {
                construction_instance_id: left,
            },
            ValuePointKind::GeneratedConstruction {
                construction_instance_id: right,
            },
        ) => left == right,
        (
            ValuePointKind::GeneratedTypeDefinition {
                type_definition_id: left,
            },
            ValuePointKind::GeneratedTypeDefinition {
                type_definition_id: right,
            },
        ) => left == right,
        _ => false,
    }
}

impl ExposedExtractionInterface {
    pub fn semantic_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ExposedExtractionInterface::Leaf, ExposedExtractionInterface::Leaf) => true,
            (ExposedExtractionInterface::Product(p1), ExposedExtractionInterface::Product(p2)) => {
                p1.semantic_eq(p2)
            }
            (
                ExposedExtractionInterface::NamedProduct(n1),
                ExposedExtractionInterface::NamedProduct(n2),
            ) => n1.semantic_eq(n2),
            _ => false,
        }
    }
}

impl ProductNormalFormShape {
    pub fn semantic_eq(&self, other: &Self) -> bool {
        product_kind_semantic_eq(&self.product_kind, &other.product_kind)
            && self.elements.len() == other.elements.len()
            && self
                .elements
                .iter()
                .zip(other.elements.iter())
                .all(|(a, b)| a.semantic_eq(b))
    }
}

fn product_kind_semantic_eq(left: &ProductNormalFormKind, right: &ProductNormalFormKind) -> bool {
    match (left, right) {
        (ProductNormalFormKind::Bare, ProductNormalFormKind::Bare) => true,
        (
            ProductNormalFormKind::Named {
                owner_type_value: left,
                ..
            },
            ProductNormalFormKind::Named {
                owner_type_value: right,
                ..
            },
        ) => left == right,
        _ => false,
    }
}

impl ProductNormalFormElem {
    pub fn semantic_eq(&self, other: &Self) -> bool {
        self.label == other.label
            && self.value_shape.semantic_eq(&other.value_shape)
            && self.type_observation == other.type_observation
    }
}

impl NamedProductExtractionShape {
    pub fn semantic_eq(&self, other: &Self) -> bool {
        self.owner_type_value == other.owner_type_value
            && self.owner_pattern_head == other.owner_pattern_head
            && self.fields.len() == other.fields.len()
            && self
                .fields
                .iter()
                .zip(other.fields.iter())
                .all(|(a, b)| a.semantic_eq(b))
    }
}

impl NamedExtractionField {
    pub fn semantic_eq(&self, other: &Self) -> bool {
        self.label == other.label
            && self.field_type_observation == other.field_type_observation
            && self.field_pattern_head == other.field_pattern_head
            && self.field_index == other.field_index
            && self.projection == other.projection
            && self.visibility == other.visibility
    }
}
