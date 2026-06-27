mod support;
use support::*;

use lang_build::meta::try_expand_early_meta_initializer;
use lang_build::{
    CompilationWorld, FieldProjection, Provenance, ResolveExpectation, SourceCategory, SymbolKind,
    SymbolPayload,
};

#[test]
fn type_associated_namespace_paths_have_expected_payloads() {
    let world = build_single_fixture_world("early_struct_meta", "app");
    let type_symbol = world.resolve("T").expect("resolve T");
    let SymbolPayload::Type(type_object) = &type_symbol.payload else {
        panic!("expected type object");
    };
    let type_namespace = type_object
        .type_associated_namespace
        .expect("type-associated namespace");

    let value_field = world.resolve("a::T").expect("resolve value field");
    let SymbolPayload::FieldFunction(value_field_object) = &value_field.payload else {
        panic!("expected value field payload");
    };
    assert_eq!(value_field_object.owner_type_symbol_id, type_symbol.id);
    assert_eq!(
        value_field_object.field_type_symbol_id,
        type_object.fields[0].type_symbol_id
    );
    assert_eq!(value_field_object.projection, FieldProjection::Value);

    let ref_namespace = world
        .resolve_with_expectation("ref::T", ResolveExpectation::NamespaceSubspace)
        .expect("resolve ref namespace");
    assert_eq!(ref_namespace.kind, SymbolKind::Namespace);
    assert_eq!(ref_namespace.parent, Some(type_namespace));

    let ref_field = world
        .resolve_with_expectation("a::ref::T", ResolveExpectation::FieldFunction)
        .expect("resolve ref field");
    let SymbolPayload::FieldFunction(ref_field_object) = &ref_field.payload else {
        panic!("expected ref field payload");
    };
    assert_eq!(ref_field_object.owner_type_symbol_id, type_symbol.id);
    assert_eq!(ref_field_object.projection, FieldProjection::Ref);

    let share_field = world
        .resolve_with_expectation("a::share::T", ResolveExpectation::FieldFunction)
        .expect("resolve share field");
    let SymbolPayload::FieldFunction(share_field_object) = &share_field.payload else {
        panic!("expected share field payload");
    };
    assert_eq!(share_field_object.owner_type_symbol_id, type_symbol.id);
    assert_eq!(share_field_object.projection, FieldProjection::Share);
}

#[test]
fn fields_named_ref_and_share_coexist_with_projection_subspaces() {
    for (fixture, field_name) in [("field_named_ref", "ref"), ("field_named_share", "share")] {
        let world = build_single_fixture_world(fixture, "app");
        let type_symbol = world
            .resolve_with_expectation("T", ResolveExpectation::TypeObject)
            .expect("resolve generated type");
        let SymbolPayload::Type(type_object) = &type_symbol.payload else {
            panic!("expected type object");
        };
        assert!(type_object
            .field_names
            .iter()
            .any(|name| name == field_name));

        let value_field = world
            .resolve_with_expectation(
                &format!("{field_name}::T"),
                ResolveExpectation::FieldFunction,
            )
            .expect("resolve field function sharing projection name");
        let SymbolPayload::FieldFunction(value_field_object) = &value_field.payload else {
            panic!("expected value field payload");
        };
        assert_eq!(value_field_object.field_name, field_name);
        assert_eq!(value_field_object.projection, FieldProjection::Value);

        let projection_namespace = world
            .resolve_with_expectation(
                &format!("{field_name}::T"),
                ResolveExpectation::NamespaceSubspace,
            )
            .expect("resolve projection namespace sharing field name");
        assert_eq!(projection_namespace.kind, SymbolKind::Namespace);

        let ambiguous = world.resolve(&format!("{field_name}::T")).expect_err(
            "terminal AnyUnique lookup is ambiguous across field object and projection namespace",
        );
        assert!(ambiguous.message.contains("ambiguous"));

        let ref_projection = world
            .resolve_with_expectation(
                &format!("{field_name}::ref::T"),
                ResolveExpectation::FieldFunction,
            )
            .expect("resolve ref projection field");
        let SymbolPayload::FieldFunction(ref_projection_object) = &ref_projection.payload else {
            panic!("expected ref projection field payload");
        };
        assert_eq!(ref_projection_object.field_name, field_name);
        assert_eq!(ref_projection_object.projection, FieldProjection::Ref);

        let share_projection = world
            .resolve_with_expectation(
                &format!("{field_name}::share::T"),
                ResolveExpectation::FieldFunction,
            )
            .expect("resolve share projection field");
        let SymbolPayload::FieldFunction(share_projection_object) = &share_projection.payload
        else {
            panic!("expected share projection field payload");
        };
        assert_eq!(share_projection_object.field_name, field_name);
        assert_eq!(share_projection_object.projection, FieldProjection::Share);

        let a_ref = world
            .resolve_with_expectation("a::ref::T", ResolveExpectation::FieldFunction)
            .expect("intermediate ref resolves through namespace subspace");
        let SymbolPayload::FieldFunction(a_ref_object) = &a_ref.payload else {
            panic!("expected ordinary ref projection field payload");
        };
        assert_eq!(a_ref_object.field_name, "a");
        assert_eq!(a_ref_object.projection, FieldProjection::Ref);
    }

    // Committed fixture that intentionally fails: duplicate same-role
    // projection-named field.
    let error = build_fixture_error("struct_duplicate_field", "app");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("duplicate struct field `ref`")));
}

#[test]
fn struct_checker_accepts_single_and_two_field_forms() {
    for (fixture, expected_fields) in [
        ("struct_single_field", vec!["a"]),
        ("early_struct_meta", vec!["a", "b"]),
    ] {
        let world = build_single_fixture_world(fixture, "app");
        let symbol = world.resolve("T").expect("resolve T");
        let SymbolPayload::Type(type_object) = symbol.payload else {
            panic!("expected Type payload");
        };
        assert_eq!(type_object.field_names, expected_fields);
    }
}

#[test]
fn struct_checker_rejects_non_type_nested_unit_and_target_errors() {
    // Committed fixtures that intentionally fail: each holds invalid struct input
    // and the test checks the rejection diagnostic.
    for (fixture, expected) in [
        ("struct_non_type_field", "unknown struct field type"),
        ("struct_nested_product", "invalid struct syntax"),
        ("struct_unit_field", "unit field or trailing unit"),
        ("struct_target_not_name", "expected a field binder name"),
        ("struct_operator_private_syntax", "invalid struct syntax"),
    ] {
        let error = build_fixture_error(fixture, "app");
        assert!(
            error
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "missing {expected:?} for `{fixture}` in {:#?}",
            error.diagnostics
        );
    }
}

#[test]
fn unknown_and_invalid_struct_inputs_are_hard_errors() {
    // Committed fixtures that intentionally fail: unknown field type and invalid
    // struct syntax.
    let error = build_fixture_error("struct_unknown_field_type", "app");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("unknown struct field type")));

    let error = build_fixture_error("struct_invalid_field_syntax", "app");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("invalid struct syntax")));
}

#[test]
fn failed_struct_meta_leaves_no_partial_generated_subtree() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let initializer = initializer_from_source("let T: type = (uint8 a, uint8 a) |> struct");
    let snapshot = world.snapshot().clone();
    let error = try_expand_early_meta_initializer(
        &snapshot,
        world.package_root_node(),
        "T",
        &initializer,
        &world.package_context(),
        Provenance::new("test duplicate fields"),
    )
    .expect_err("duplicate fields must fail");

    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("duplicate struct field")));
    assert!(
        snapshot
            .child_symbol(world.package_root_node(), "T")
            .is_none(),
        "failed meta expansion must not leave a generated type"
    );
}

#[test]
fn unresolved_meta_target_returns_none() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let initializer = initializer_from_source("let T: type = (uint8 a) |> missing_meta");
    let expansion = try_expand_early_meta_initializer(
        world.snapshot(),
        world.package_root_node(),
        "T",
        &initializer,
        &world.package_context(),
        Provenance::new("unresolved meta target"),
    )
    .expect("unresolved target is not treated as parser/normalizer special case");
    assert!(expansion.is_none());
}

#[test]
fn meta_function_kind_without_payload_is_hard_error() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let mut delta = world.snapshot().empty_delta();
    let bad_meta = delta.allocate_symbol_id();
    let mut bad_symbol = lang_build::SymbolObject::placeholder(
        bad_meta,
        "bad_meta",
        SymbolKind::MetaFunction,
        SourceCategory::DeclaredSymbol,
        Some(world.package_root_node()),
        Provenance::new("bad meta without payload"),
    );
    bad_symbol
        .policy_metadata
        .policy_set
        .insert(lang_build::PolicyFlag::Meta);
    delta.insert_symbol(world.package_root_node(), bad_symbol);
    let snapshot = world
        .snapshot()
        .install_delta(delta)
        .expect("install bad meta");
    let initializer = initializer_from_source("let T: type = (uint8 a) |> bad_meta");
    let error = try_expand_early_meta_initializer(
        &snapshot,
        world.package_root_node(),
        "T",
        &initializer,
        &lang_build::ResolverContext::with_mounts(
            world.package_root_node(),
            vec![snapshot.root_node()],
            vec![world.core_node()],
        ),
        Provenance::new("bad meta call"),
    )
    .expect_err("MetaFunction kind without payload is hard error");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("no meta-function payload")));
}
