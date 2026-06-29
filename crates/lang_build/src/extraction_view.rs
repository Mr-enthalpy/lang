//! Return normal form and extraction-view substrate.
//!
//! This module is shape-only. It does not evaluate full expressions, perform
//! destructuring, or install namespace graph material. Product normal form `P`
//! is represented directly; it is not wrapped in a non-product call value.

use crate::{
    meta_invocation::{ConstructionInstanceId, MetaValueTarget, TypeDefinitionInstanceId},
    model::{Diagnostic, FieldProjection, Provenance, SymbolId},
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
        owner_type_symbol_id: Option<SymbolId>,
    },
    Forwarded {
        target: MetaValueTarget,
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
        owner_type_symbol_id: Option<SymbolId>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductNormalFormElem {
    pub label: Option<String>,
    pub value_shape: Box<EvalResultNormalForm>,
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
    pub owner_type_symbol_id: SymbolId,
    pub fields: Vec<NamedExtractionField>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedExtractionField {
    pub label: String,
    pub field_type_symbol_id: SymbolId,
    pub field_index: usize,
    pub projection: FieldProjection,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeExtractionInterface {
    pub owner_type_symbol_id: SymbolId,
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
                type_symbol_id: Some(field.field_type_symbol_id),
                provenance: field.provenance.clone(),
            })
            .collect(),
        product_kind: ProductNormalFormKind::Named {
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
