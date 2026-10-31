mod support;
use support::*;

use lang_build::{ResolveExpectation, SourceCategory, Stage, SymbolPayload};

#[test]
fn inherited_declaration_destination_requires_the_ordinary_consumer() {
    let error = build_fixture_error("inherited_navigation", "app");
    assert!(format!("{error:?}").contains("inherited navigation requires the qualified NameExpr"));
}

#[test]
fn empty_nested_source_does_not_create_a_namespace_beside_a_type_binding() {
    let world = build_single_fixture_world("empty_navigation_beside_binding", "app");
    let semantic = world.semantic_world();
    let root = world.package_root_node();
    assert!(semantic
        .symbol_in_namespace(root, "T")
        .unwrap()
        .type_resident()
        .is_some());
    assert!(semantic.child_namespace(root, "T").is_none());
    assert!(world
        .namespace_projection()
        .child_symbol_with_expectation(root, "T", ResolveExpectation::NamespaceSubspace)
        .is_err());
    assert!(world
        .source_fragments()
        .iter()
        .any(|f| f.inherited_navigation == ["T"]));
}

#[test]
fn type_value_binding_reuses_value_and_keeps_fresh_binding_place() {
    let world = build_single_fixture_world("single_package_type_binding", "app");
    let symbol = world
        .resolve_with_expectation("T", ResolveExpectation::CoreTypeProjection)
        .expect("resolve ordinary type-value binding");
    let core_uint8 = world
        .resolve_with_expectation("uint8::core", ResolveExpectation::CoreTypeProjection)
        .expect("resolve core uint8 type");

    assert_eq!(symbol.name, "T");
    assert_eq!(symbol.source_category, SourceCategory::DeclaredSymbol);
    assert_eq!(symbol.parent, Some(world.package_root_node()));
    let view = symbol
        .policy_view
        .as_ref()
        .expect("type binding Policy view");
    assert!(view.pair.value.stage() == Some(Stage::Compile));
    assert!(view.pair.value.stage() != Some(Stage::Runtime));
    let symbol_id = symbol.id;

    let SymbolPayload::CompleteTypeProjection(type_projection) = symbol.payload else {
        panic!("expected bound Type payload");
    };

    let SymbolPayload::CompleteTypeProjection(core_type) = core_uint8.payload else {
        panic!("core uint8 is a CompleteType projection");
    };
    assert_eq!(type_projection.carrier_symbol_id, symbol_id);
    assert_eq!(type_projection.represented_type, core_type.represented_type);
    assert_ne!(symbol_id, core_uint8.id);
    assert!(type_projection.type_associated_namespace.is_some());
}
