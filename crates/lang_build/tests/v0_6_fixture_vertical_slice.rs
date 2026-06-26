mod support;
use support::*;

use lang_build::{BuildSession, PolicyFlag, SymbolKind, SymbolPayload};

// High-level vertical-slice proofs built from committed repository fixtures
// through BuildSession:
//
//   real repo directory
//     -> build session
//     -> physical source discovery
//     -> lexer / parser / normalizer
//     -> namespace graph assembly
//     -> early struct-meta object generation
//     -> policy-plane metadata

// Early struct-meta expansion through the full build, asserting the generated
// type and every generated field function exist.
#[test]
fn early_struct_meta_full_path() {
    let world = build_single_fixture_world("early_struct_meta", "app");

    let type_symbol = world.resolve("T").expect("resolve generated type");
    assert_eq!(type_symbol.kind, SymbolKind::Type);
    let SymbolPayload::Type(type_object) = &type_symbol.payload else {
        panic!("expected type payload, got {type_symbol:#?}");
    };
    assert_eq!(type_object.field_names, ["a", "b"]);
    assert_eq!(type_object.fields.len(), 2);
    assert!(type_object.type_associated_namespace.is_some());

    for path in [
        "a::T",
        "a::ref::T",
        "a::share::T",
        "b::T",
        "b::ref::T",
        "b::share::T",
    ] {
        let symbol = world
            .resolve(path)
            .expect("resolve generated field function");
        assert_eq!(symbol.kind, SymbolKind::FieldFunction, "{path}");
        assert!(matches!(symbol.payload, SymbolPayload::FieldFunction(_)));
    }
}

// Policy-plane boundary through the full build: an ordinary runtime value stays
// runtime-only, struct meta still expands, the generated field function is
// meta+runtime visible, and its body remains runtime-entry only.
#[test]
fn policy_aware_early_meta_full_path() {
    let world = build_single_fixture_world("policy_aware_early_meta", "app");

    let not_meta = world.resolve("not_meta").expect("resolve runtime value");
    let runtime_policy = &not_meta.policy_metadata.policy_set;
    assert!(runtime_policy.contains(PolicyFlag::Runtime));
    assert!(!runtime_policy.contains(PolicyFlag::Meta));
    assert!(!runtime_policy.contains(PolicyFlag::Export));

    let type_symbol = world.resolve("T").expect("resolve generated type");
    assert_eq!(type_symbol.kind, SymbolKind::Type);
    let SymbolPayload::Type(type_object) = &type_symbol.payload else {
        panic!("expected type payload");
    };
    assert_eq!(type_object.field_names, ["a"]);

    let field = world
        .resolve("a::T")
        .expect("resolve generated field function");
    assert_eq!(field.kind, SymbolKind::FieldFunction);
    let field_policy = &field.policy_metadata.policy_set;
    assert!(field_policy.contains(PolicyFlag::Meta));
    assert!(field_policy.contains(PolicyFlag::Runtime));
    assert!(!field_policy.contains(PolicyFlag::Export));

    let SymbolPayload::FieldFunction(field_object) = &field.payload else {
        panic!("expected field-function payload");
    };
    let body_policy = &field_object.callable_policy.body_entry_policy.policy_set;
    assert!(body_policy.contains(PolicyFlag::Runtime));
    assert!(!body_policy.contains(PolicyFlag::Meta));
}

// Full vertical slice from a multi-file committed fixture through BuildSession:
// discovery metadata, source fragments, clean diagnostics, struct-meta object
// generation, field-function policy planes, and nested physical-namespace
// discovery.
#[test]
fn vertical_slice_full_pipeline() {
    let mut session = BuildSession::new();
    let result = session
        .build_workspace(&single_package_fixture("vertical_slice", "app"))
        .expect("build workspace");

    assert_eq!(result.artifacts.len(), 1);
    let artifact = &result.artifacts[0];
    assert_eq!(artifact.package_name, "app");

    // Discovery metadata includes the committed fixture files.
    assert_eq!(artifact.metadata.source_units.len(), 2);
    let fragment_names: Vec<&str> = artifact
        .metadata
        .source_units
        .iter()
        .map(|unit| unit.fragment_name.as_str())
        .collect();
    assert!(fragment_names.contains(&"main.lang"));
    assert!(fragment_names.contains(&"user.lang"));

    let world = &artifact.world;
    assert_eq!(world.source_fragments().len(), 2);
    assert!(
        world.diagnostics().is_empty(),
        "lexer/parser diagnostics must be empty: {:#?}",
        world.diagnostics()
    );

    // `User` is a top-level struct-meta generated type object.
    let user = world.resolve("User").expect("User resolves");
    assert_eq!(user.kind, SymbolKind::Type);
    assert!(matches!(user.payload, SymbolPayload::Type(_)));

    // Field functions are generated and carry policy-plane metadata.
    let id_field = world.resolve("id::User").expect("id::User resolves");
    assert_eq!(id_field.kind, SymbolKind::FieldFunction);
    let id_ref_field = world
        .resolve("id::ref::User")
        .expect("id::ref::User resolves");
    assert_eq!(id_ref_field.kind, SymbolKind::FieldFunction);

    let field_policy = &id_field.policy_metadata.policy_set;
    assert!(field_policy.contains(PolicyFlag::Meta));
    assert!(field_policy.contains(PolicyFlag::Runtime));
    let SymbolPayload::FieldFunction(field_object) = &id_field.payload else {
        panic!("expected field-function payload");
    };
    let body_policy = &field_object.callable_policy.body_entry_policy.policy_set;
    assert!(body_policy.contains(PolicyFlag::Runtime));
    assert!(!body_policy.contains(PolicyFlag::Meta));

    // The `types/` directory contributes a nested physical namespace.
    assert!(
        world.resolve("Helper::types").is_ok(),
        "the types/ directory must contribute a physical namespace"
    );
}
