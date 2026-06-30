mod support;
use support::*;

use lang_build::{PolicyFlag, ResolveExpectation, SourceCategory, SymbolPayload};

#[test]
fn type_value_binding_forwards_type_value_and_keeps_fresh_binding_place() {
    let world = build_single_fixture_world("single_package_type_binding", "app");
    let symbol = world
        .resolve_with_expectation("T", ResolveExpectation::TypeObject)
        .expect("resolve forwarded type binding");
    let core_uint8 = world
        .resolve_with_expectation("uint8::core", ResolveExpectation::TypeObject)
        .expect("resolve core uint8 type");

    assert_eq!(symbol.name, "T");
    assert_eq!(symbol.source_category, SourceCategory::DeclaredSymbol);
    assert_eq!(symbol.parent, Some(world.package_root_node()));
    assert!(symbol.policy_metadata.policy_set.contains(PolicyFlag::Meta));
    assert!(symbol
        .policy_metadata
        .policy_set
        .contains(PolicyFlag::Runtime));
    let symbol_id = symbol.id;

    let SymbolPayload::Type(type_object) = symbol.payload else {
        panic!("expected forwarded Type payload");
    };

    assert_eq!(type_object.type_symbol_id, core_uint8.id);
    assert_ne!(symbol_id, core_uint8.id);
    assert!(type_object.type_associated_namespace.is_some());
}
