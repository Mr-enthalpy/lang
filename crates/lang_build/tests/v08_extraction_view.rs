mod support;

use support::*;

use lang_build::{
    observe_content_projection, ContentObservationInterface, FieldProjection,
    ObservedArgumentContent, ObservedAtomContent, ObservedAtomKind, ObservedContentProjection,
    ObservedProductContent, ObservedProductElement, ObservedProductKind, Provenance,
    ResolveExpectation, SymbolPayload,
};

fn leaf(description: &str) -> ObservedArgumentContent {
    ObservedArgumentContent::ValuePoint(ObservedAtomContent {
        value_kind: ObservedAtomKind::Leaf,
        extraction_interface: ContentObservationInterface::Leaf,
        provenance: Provenance::new(description),
    })
}

fn bare_product(arity: usize, description: &str) -> ObservedProductContent {
    ObservedProductContent {
        elements: (0..arity)
            .map(|index| ObservedProductElement {
                label: None,
                value_shape: Box::new(leaf(format!("{description} element {index}").as_str())),
                type_value: None,
                type_observation: None,
                type_symbol_id: None,
                provenance: Provenance::new(format!("{description} element {index}")),
            })
            .collect(),
        product_kind: ObservedProductKind::Bare,
        provenance: Provenance::new(description),
    }
}

fn value_exposing_product(product: ObservedProductContent) -> ObservedArgumentContent {
    ObservedArgumentContent::ValuePoint(ObservedAtomContent {
        value_kind: ObservedAtomKind::Constructed {
            owner_type_value: None,
            owner_type_symbol_id: None,
        },
        extraction_interface: ContentObservationInterface::Product(product),
        provenance: Provenance::new("non-leaf value exposing product"),
    })
}

#[test]
fn product_return_is_product_normal_form() {
    let product = ObservedArgumentContent::Product(bare_product(2, "product normal form"));

    assert!(matches!(product, ObservedArgumentContent::Product(_)));
    assert_eq!(
        observe_content_projection(&product),
        ObservedContentProjection::NormalForm(product)
    );
}

#[test]
fn question_mark_is_idempotent_on_product_normal_form() {
    let product = ObservedArgumentContent::Product(bare_product(2, "idempotent product"));

    assert_eq!(
        observe_content_projection(&product),
        ObservedContentProjection::NormalForm(product)
    );
}

#[test]
fn question_mark_is_idempotent_on_leaf_value_point() {
    let value = leaf("leaf value point");

    assert_eq!(
        observe_content_projection(&value),
        ObservedContentProjection::NormalForm(value)
    );
}

#[test]
fn question_mark_enters_non_leaf_exposed_product_view() {
    let product = bare_product(2, "exposed product");
    let value = value_exposing_product(product.clone());

    assert_eq!(
        observe_content_projection(&value),
        ObservedContentProjection::NormalForm(ObservedArgumentContent::Product(product))
    );
}

#[test]
fn struct_type_materialization_records_named_field_extraction_interface() {
    let world = build_single_fixture_world("v08_struct_uint8", "app");
    let result = world
        .resolve_with_expectation("T", ResolveExpectation::CoreTypeProjection)
        .expect("source build installs the generated type");
    let uint8 = world
        .resolve_with_expectation("uint8", ResolveExpectation::CoreTypeProjection)
        .expect("uint8 type resolves");

    let SymbolPayload::CompleteTypeProjection(type_object) = &result.payload else {
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
    let SymbolPayload::CompleteTypeProjection(uint8_type) = &uint8.payload else {
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
    let product_normal_form = ObservedArgumentContent::Product(product.clone());
    let non_leaf = value_exposing_product(product);

    assert_ne!(product_normal_form, non_leaf);
    assert_eq!(
        observe_content_projection(&non_leaf),
        ObservedContentProjection::NormalForm(product_normal_form)
    );
}

#[test]
fn extraction_semantic_identity_ignores_type_carrier_symbol() {
    let left = ObservedArgumentContent::ValuePoint(ObservedAtomContent {
        value_kind: ObservedAtomKind::Constructed {
            owner_type_value: Some(lang_build::TypeValueId(7)),
            owner_type_symbol_id: Some(lang_build::SymbolId(70)),
        },
        extraction_interface: ContentObservationInterface::Leaf,
        provenance: Provenance::new("left carrier"),
    });
    let right = ObservedArgumentContent::ValuePoint(ObservedAtomContent {
        value_kind: ObservedAtomKind::Constructed {
            owner_type_value: Some(lang_build::TypeValueId(7)),
            owner_type_symbol_id: Some(lang_build::SymbolId(71)),
        },
        extraction_interface: ContentObservationInterface::Leaf,
        provenance: Provenance::new("right carrier"),
    });

    assert_ne!(left, right, "exact fixture material retains graph carriers");
    assert!(
        left.observationally_equal(&right),
        "content observation consumes the TypeValue, not the carrier Symbol"
    );
}
