mod support;
use support::*;

use lang_build::{ResolveExpectation, SourceCategory, SymbolPayload};

#[test]
fn type_value_binding_placeholder_keeps_fresh_symbol_place_until_type_values_exist() {
    // Compiler-internal placeholder invariant: source verification can observe
    // that `T` is a type, but not the temporary TypeObject payload used until
    // TypeValueId and writable-place checking exist.
    // Missing capability: verify.type_payload_identity.
    let world = build_single_fixture_world("single_package_type_binding", "app");
    let symbol = world
        .resolve_with_expectation("T", ResolveExpectation::TypeObject)
        .expect("resolve v0.6 placeholder type binding");
    assert_eq!(symbol.name, "T");
    assert_eq!(symbol.source_category, SourceCategory::DeclaredSymbol);
    assert_eq!(symbol.parent, Some(world.package_root_node()));
    let symbol_id = symbol.id;

    // GUARD: This test must NOT be read as final fresh nominal type generation.
    // The placeholder `TypeObject` exists only because type-value evaluation,
    // TypeValueId, and writable-place checking are not yet implemented.
    // When those features land, this test must be replaced with a proper
    // type-value binding test (fresh symbol/place `T` bound to existing type
    // value `uint8`, place(T) != place(uint8)).
    //
    // v0.6 placeholder behavior only: this is not final type-value semantics.
    // Long-term, `let T: type = uint8` binds fresh symbol/place `T` to the
    // existing type value `uint8`; injection through `T` targets place(T), not
    // place(uint8).
    let SymbolPayload::Type(type_object) = symbol.payload else {
        panic!("expected placeholder Type payload");
    };
    assert_eq!(type_object.type_symbol_id, symbol_id);
    assert!(type_object.type_associated_namespace.is_some());
}
