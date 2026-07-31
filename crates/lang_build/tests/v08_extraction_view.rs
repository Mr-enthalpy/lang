mod support;

use support::*;

use lang_build::{
    match_binding_pattern_shape, question_view, BindingPatternShape, BindingShapeMatchResult,
    EvalResultNormalForm, ExposedExtractionInterface, ExtractionViewResult, FieldProjection,
    ProductNormalFormElem, ProductNormalFormKind, ProductNormalFormShape, Provenance,
    ResolveExpectation, SymbolPayload, ValuePointKind, ValuePointShape,
};

fn leaf(description: &str) -> EvalResultNormalForm {
    EvalResultNormalForm::ValuePoint(ValuePointShape {
        value_kind: ValuePointKind::Leaf,
        extraction_interface: ExposedExtractionInterface::Leaf,
        provenance: Provenance::new(description),
    })
}

fn bare_product(arity: usize, description: &str) -> ProductNormalFormShape {
    ProductNormalFormShape {
        elements: (0..arity)
            .map(|index| ProductNormalFormElem {
                label: None,
                value_shape: Box::new(leaf(format!("{description} element {index}").as_str())),
                type_value: None,
                type_observation: None,
                type_symbol_id: None,
                provenance: Provenance::new(format!("{description} element {index}")),
            })
            .collect(),
        product_kind: ProductNormalFormKind::Bare,
        provenance: Provenance::new(description),
    }
}

fn value_exposing_product(product: ProductNormalFormShape) -> EvalResultNormalForm {
    EvalResultNormalForm::ValuePoint(ValuePointShape {
        value_kind: ValuePointKind::Constructed {
            owner_type_value: None,
            owner_type_symbol_id: None,
        },
        extraction_interface: ExposedExtractionInterface::Product(product),
        provenance: Provenance::new("non-leaf value exposing product"),
    })
}

#[test]
fn product_return_is_product_normal_form() {
    let product = EvalResultNormalForm::Product(bare_product(2, "product normal form"));

    assert!(matches!(product, EvalResultNormalForm::Product(_)));
    assert_eq!(
        question_view(&product),
        ExtractionViewResult::NormalForm(product)
    );
}

#[test]
fn question_mark_is_idempotent_on_product_normal_form() {
    let product = EvalResultNormalForm::Product(bare_product(2, "idempotent product"));

    assert_eq!(
        question_view(&product),
        ExtractionViewResult::NormalForm(product)
    );
}

#[test]
fn question_mark_is_idempotent_on_leaf_value_point() {
    let value = leaf("leaf value point");

    assert_eq!(
        question_view(&value),
        ExtractionViewResult::NormalForm(value)
    );
}

#[test]
fn question_mark_enters_non_leaf_exposed_product_view() {
    let product = bare_product(2, "exposed product");
    let value = value_exposing_product(product.clone());

    assert_eq!(
        question_view(&value),
        ExtractionViewResult::NormalForm(EvalResultNormalForm::Product(product))
    );
}

#[test]
fn product_pattern_matches_product_normal_form_directly() {
    let product = EvalResultNormalForm::Product(bare_product(2, "direct product"));
    let pattern = BindingPatternShape::Product {
        arity: 2,
        named: false,
    };

    assert_eq!(
        match_binding_pattern_shape(&pattern, &product),
        BindingShapeMatchResult::Direct
    );
}

#[test]
fn product_pattern_matches_non_leaf_value_after_extraction() {
    let value = value_exposing_product(bare_product(2, "repair product"));
    let pattern = BindingPatternShape::Product {
        arity: 2,
        named: false,
    };

    assert_eq!(
        match_binding_pattern_shape(&pattern, &value),
        BindingShapeMatchResult::AfterExtraction
    );
}

#[test]
fn product_pattern_does_not_match_leaf_after_extraction() {
    let value = leaf("leaf mismatch");
    let pattern = BindingPatternShape::Product {
        arity: 2,
        named: false,
    };

    assert_eq!(
        match_binding_pattern_shape(&pattern, &value),
        BindingShapeMatchResult::Mismatch
    );
}

#[test]
fn struct_type_materialization_records_named_field_extraction_interface() {
    let world = build_single_fixture_world("v08_struct_uint8", "app");
    let result = world
        .resolve_with_expectation("T", ResolveExpectation::TypeObject)
        .expect("source build installs the generated type");
    let uint8 = world
        .resolve_with_expectation("uint8", ResolveExpectation::TypeObject)
        .expect("uint8 type resolves");

    let SymbolPayload::Type(type_object) = &result.payload else {
        panic!("struct expansion replacement must be a type object");
    };
    let extraction = type_object
        .extraction_interface
        .as_ref()
        .expect("generated struct type records instance extraction interface");

    assert_eq!(
        extraction.owner_type_symbol_id,
        type_object.carrier_symbol_id
    );
    assert_eq!(extraction.owner_type_value, type_object.represented_type);
    assert_eq!(
        extraction.exposed_view.owner_type_symbol_id,
        type_object.carrier_symbol_id
    );
    assert_eq!(
        extraction.exposed_view.owner_type_value,
        type_object.represented_type
    );
    assert_eq!(extraction.exposed_view.fields.len(), 1);
    let field = &extraction.exposed_view.fields[0];
    assert_eq!(field.label, "a");
    let SymbolPayload::Type(uint8_type) = &uint8.payload else {
        panic!("uint8 carries a type value");
    };
    assert_eq!(field.field_type_value, uint8_type.represented_type);
    assert_eq!(field.field_type_symbol_id, uint8.id);
    assert_eq!(field.field_index, 0);
    assert_eq!(field.projection, FieldProjection::Value);
}

#[test]
fn equality_shape_logic_has_no_extraction_repair_entry() {
    let product = bare_product(2, "equality product");
    let product_normal_form = EvalResultNormalForm::Product(product.clone());
    let non_leaf = value_exposing_product(product);

    assert_ne!(product_normal_form, non_leaf);
    assert_eq!(
        question_view(&non_leaf),
        ExtractionViewResult::NormalForm(product_normal_form)
    );
}

#[test]
fn extraction_semantic_identity_ignores_type_carrier_symbol() {
    let left = EvalResultNormalForm::ValuePoint(ValuePointShape {
        value_kind: ValuePointKind::Constructed {
            owner_type_value: Some(lang_build::TypeValueId(7)),
            owner_type_symbol_id: Some(lang_build::SymbolId(70)),
        },
        extraction_interface: ExposedExtractionInterface::Leaf,
        provenance: Provenance::new("left carrier"),
    });
    let right = EvalResultNormalForm::ValuePoint(ValuePointShape {
        value_kind: ValuePointKind::Constructed {
            owner_type_value: Some(lang_build::TypeValueId(7)),
            owner_type_symbol_id: Some(lang_build::SymbolId(71)),
        },
        extraction_interface: ExposedExtractionInterface::Leaf,
        provenance: Provenance::new("right carrier"),
    });

    assert_ne!(left, right, "exact fixture material retains graph carriers");
    assert!(
        left.semantic_eq(&right),
        "semantic extraction identity consumes the TypeValue, not the carrier Symbol"
    );
}
