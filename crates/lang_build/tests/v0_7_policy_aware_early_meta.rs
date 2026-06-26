mod support;
use support::*;

use lang_build::{
    callable_body_allows_execution, policy_set_runtime, CompilationWorld, CoreMetaFunction,
    ExecutionEnv, MetaFunctionObject, PolicyEnv, PolicyFlag, PolicyMetadata, Provenance,
    ResolveExpectation, ResolverCode, SourceCategory, SymbolKind, SymbolObject, SymbolPayload,
};

#[test]
fn type_named_struct_does_not_shadow_core_meta_function() {
    let world = build_single_fixture_world("type_named_struct", "app");
    // The local `struct` is a Type symbol (type-annotated → meta+runtime).
    // The resolver rejects it via ResolveExpectation::MetaFunction (kind
    // filtering), not policy filtering. Core's `struct` (MetaFunction,
    // export+meta) is then found through the default core mount.
    let type_symbol = world.resolve("T").expect("resolve generated type");
    assert_eq!(type_symbol.kind, SymbolKind::Type);
    let SymbolPayload::Type(type_object) = &type_symbol.payload else {
        panic!("expected type payload");
    };
    assert_eq!(type_object.field_names, ["a"]);
}

#[test]
fn runtime_only_call_target_is_soft_miss_under_meta() {
    let world = build_single_fixture_world("non_meta_target", "app");
    // Soft miss: T is harvested as a type-annotated placeholder.
    let symbol = world.resolve("T").expect("T resolved as type placeholder");
    assert_eq!(symbol.kind, SymbolKind::Type);
    // No field functions exist (not a struct expansion).
    assert!(world.resolve("a::T").is_err());
}

#[test]
fn uint8_resolves_under_meta_policy() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let capability = world.snapshot().capability();
    let context = world.package_context();

    let symbol = capability
        .resolve_type_object_with_policy("uint8", &context, PolicyEnv::Meta)
        .expect("uint8 should resolve under Meta policy (export+meta+runtime)");
    assert_eq!(symbol.kind, SymbolKind::Type);
    assert_eq!(symbol.name, "uint8");
}

#[test]
fn runtime_only_value_not_usable_as_struct_field_type() {
    let project = TempProject::new("runtime_field_type");
    project.write(
        "src/main.lang",
        "let MyType = 1; let T: type = (MyType a) |> struct",
    );
    let error = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect_err("runtime-only value should not resolve as field type under Meta policy");
    assert!(
        error
            .diagnostics
            .iter()
            .any(|d| d.message.contains("unknown struct field type")),
        "expected 'unknown struct field type' diagnostic"
    );
}

#[test]
fn core_symbols_have_expected_policy_flags() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");

    // Core meta-functions: export + meta
    for name in ["struct", "assert"] {
        let symbol = world.resolve(name).expect("resolve core meta-function");
        let ps = &symbol.policy_metadata.policy_set;
        assert!(ps.contains(PolicyFlag::Export), "{name} missing Export");
        assert!(ps.contains(PolicyFlag::Meta), "{name} missing Meta");
        assert!(
            !ps.contains(PolicyFlag::Runtime),
            "{name} should not have Runtime"
        );
    }

    // Core types: export + meta + runtime
    for name in [
        "uint8",
        "type",
        "namespace",
        "ref",
        "share",
        "uint16",
        "uint32",
        "float32",
    ] {
        let symbol = world.resolve(name).expect("resolve core type");
        let ps = &symbol.policy_metadata.policy_set;
        assert!(ps.contains(PolicyFlag::Export), "{name} missing Export");
        assert!(ps.contains(PolicyFlag::Meta), "{name} missing Meta");
        assert!(ps.contains(PolicyFlag::Runtime), "{name} missing Runtime");
    }
}

#[test]
fn core_meta_function_payload_policy_slots_are_non_empty() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");

    for (name, primitive, return_has_runtime) in [
        ("struct", CoreMetaFunction::Struct, true),
        ("assert", CoreMetaFunction::Assert, false),
    ] {
        let symbol = world.resolve(name).expect("resolve core meta-function");
        let symbol_policy = &symbol.policy_metadata.policy_set;
        assert!(symbol_policy.contains(PolicyFlag::Export));
        assert!(symbol_policy.contains(PolicyFlag::Meta));
        assert!(!symbol_policy.contains(PolicyFlag::Runtime));

        let SymbolPayload::MetaFunction(meta_function) = &symbol.payload else {
            panic!("expected {name} meta-function payload");
        };
        assert_eq!(meta_function.primitive, primitive);

        let function_policy = &meta_function.function_policy.policy_set;
        assert!(function_policy.contains(PolicyFlag::Export));
        assert!(function_policy.contains(PolicyFlag::Meta));
        assert!(!function_policy.contains(PolicyFlag::Runtime));

        let body_policy = &meta_function.body_entry_policy.policy_set;
        assert!(body_policy.contains(PolicyFlag::Meta));
        assert!(!body_policy.contains(PolicyFlag::Runtime));
        assert!(callable_body_allows_execution(
            &meta_function.body_entry_policy,
            ExecutionEnv::Meta
        ));
        assert!(!callable_body_allows_execution(
            &meta_function.body_entry_policy,
            ExecutionEnv::Runtime
        ));

        let return_policy = &meta_function.return_object_policy.policy_set;
        assert!(return_policy.contains(PolicyFlag::Meta));
        assert_eq!(
            return_policy.contains(PolicyFlag::Runtime),
            return_has_runtime
        );
    }
}

#[test]
fn user_declared_values_are_runtime_only() {
    let world = build_single_fixture_world("user_runtime_values", "app");

    let x = world.resolve("x").expect("resolve x");
    let ps = &x.policy_metadata.policy_set;
    assert!(!ps.contains(PolicyFlag::Export), "x should not have Export");
    assert!(!ps.contains(PolicyFlag::Meta), "x should not have Meta");
    assert!(ps.contains(PolicyFlag::Runtime), "x should have Runtime");

    let y = world
        .resolve_with_expectation("y", ResolveExpectation::TypeObject)
        .expect("resolve y as type");
    let ps = &y.policy_metadata.policy_set;
    assert!(!ps.contains(PolicyFlag::Export), "y should not have Export");
    assert!(
        ps.contains(PolicyFlag::Meta),
        "y should have Meta (type-annotated)"
    );
    assert!(ps.contains(PolicyFlag::Runtime), "y should have Runtime");
}

#[test]
fn runtime_only_value_is_invisible_under_meta_lookup() {
    let world = build_single_fixture_world("user_runtime_values", "app");
    let context = world.package_context();

    let diagnostic = world
        .snapshot()
        .capability()
        .resolve_with_policy(
            &["x".to_string()],
            &context,
            ResolveExpectation::Object,
            PolicyEnv::Meta,
        )
        .expect_err("runtime-only value should be filtered out under Meta lookup");
    assert_eq!(diagnostic.code, Some(ResolverCode::Unresolved));
}

#[test]
fn struct_generated_type_has_meta_runtime_policy() {
    let world = build_single_fixture_world("struct_single_field", "app");
    let symbol = world.resolve("T").expect("resolve T");
    let ps = &symbol.policy_metadata.policy_set;
    assert!(
        !ps.contains(PolicyFlag::Export),
        "generated T should not have Export"
    );
    assert!(
        ps.contains(PolicyFlag::Meta),
        "generated T should have Meta"
    );
    assert!(
        ps.contains(PolicyFlag::Runtime),
        "generated T should have Runtime"
    );

    let context = world.package_context();
    let meta_resolved = world
        .snapshot()
        .capability()
        .resolve_type_object_with_policy("T", &context, PolicyEnv::Meta)
        .expect("generated T should resolve under Meta lookup");
    assert_eq!(meta_resolved.kind, SymbolKind::Type);
}

#[test]
fn runtime_only_meta_function_is_filtered_under_meta_policy() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let mut delta = world.snapshot().empty_delta();

    let local_struct_id = delta.allocate_symbol_id();
    let mut local_struct = SymbolObject::placeholder(
        local_struct_id,
        "struct",
        SymbolKind::MetaFunction,
        SourceCategory::DeclaredSymbol,
        Some(world.package_root_node()),
        Provenance::new("local runtime-only struct"),
    );
    local_struct.policy_metadata.policy_set = policy_set_runtime();
    local_struct.payload = SymbolPayload::MetaFunction(MetaFunctionObject {
        function_symbol_id: local_struct_id,
        primitive: CoreMetaFunction::Assert,
        function_policy: PolicyMetadata::default(),
        body_entry_policy: PolicyMetadata::default(),
        return_object_policy: PolicyMetadata::default(),
    });
    delta.insert_symbol(world.package_root_node(), local_struct);

    let snapshot = world
        .snapshot()
        .install_delta(delta)
        .expect("install delta");
    let context = world.package_context();

    let result = snapshot.capability().resolve_meta_function_with_policy(
        "struct",
        &context,
        PolicyEnv::Meta,
    );
    assert!(
        result.is_ok(),
        "core struct should resolve under Meta despite local runtime-only struct"
    );
    let symbol = result.unwrap();
    assert_eq!(symbol.name, "struct");
    assert!(
        symbol.provenance.description.contains("core"),
        "should resolve to core's struct, not the local runtime-only one"
    );
}

#[test]
fn explicit_path_meta_function_resolves_under_meta() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let context = world.root_context();

    let symbol = world
        .snapshot()
        .capability()
        .resolve_meta_function_with_policy("struct::core", &context, PolicyEnv::Meta)
        .expect("struct::core should resolve under Meta with root context");
    assert_eq!(symbol.kind, SymbolKind::MetaFunction);
    assert_eq!(symbol.name, "struct");
    assert!(
        symbol.provenance.description.contains("core"),
        "should be core's struct"
    );
}

#[test]
fn explicit_path_type_object_resolves_under_meta() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let context = world.root_context();

    let symbol = world
        .snapshot()
        .capability()
        .resolve_type_object_with_policy("uint8::core", &context, PolicyEnv::Meta)
        .expect("uint8::core should resolve under Meta with root context");
    assert_eq!(symbol.kind, SymbolKind::Type);
    assert_eq!(symbol.name, "uint8");
    assert!(
        symbol.provenance.description.contains("core"),
        "should be core's uint8"
    );
}

#[test]
fn meta_policy_can_traverse_physical_namespace_path_to_type() {
    let world = build_single_fixture_world("physical_subns", "app");
    let context = world.package_context();

    let symbol = world
        .snapshot()
        .capability()
        .resolve_type_object_with_policy("T::subns", &context, PolicyEnv::Meta)
        .expect("T::subns should resolve under Meta via physical namespace");
    assert_eq!(symbol.kind, SymbolKind::Type);
    assert_eq!(symbol.name, "T");
}

#[test]
fn meta_policy_can_traverse_type_projection_namespace() {
    let world = build_single_fixture_world("struct_single_field", "app");
    let context = world.package_context();

    let symbol = world
        .snapshot()
        .capability()
        .resolve_with_policy(
            &["ref".to_string(), "T".to_string()],
            &context,
            ResolveExpectation::NamespaceSubspace,
            PolicyEnv::Meta,
        )
        .expect("ref::T projection namespace should resolve under Meta");
    assert_eq!(symbol.kind, SymbolKind::Namespace);
    assert_eq!(symbol.name, "ref");
}

#[test]
fn generated_field_function_is_visible_under_meta_policy() {
    let world = build_single_fixture_world("struct_single_field", "app");
    let context = world.package_context();

    for (path, expected_projection) in [
        (
            vec!["a".to_string(), "T".to_string()],
            lang_build::FieldProjection::Value,
        ),
        (
            vec!["a".to_string(), "ref".to_string(), "T".to_string()],
            lang_build::FieldProjection::Ref,
        ),
        (
            vec!["a".to_string(), "share".to_string(), "T".to_string()],
            lang_build::FieldProjection::Share,
        ),
    ] {
        let symbol = world
            .snapshot()
            .capability()
            .resolve_with_policy(
                &path,
                &context,
                ResolveExpectation::FieldFunction,
                PolicyEnv::Meta,
            )
            .expect("field function should resolve under Meta lookup");
        assert_eq!(symbol.kind, SymbolKind::FieldFunction);
        assert_eq!(symbol.name, "a");

        let ps = &symbol.policy_metadata.policy_set;
        assert!(
            !ps.contains(PolicyFlag::Export),
            "field function should not have Export"
        );
        assert!(
            ps.contains(PolicyFlag::Meta),
            "field function should have Meta"
        );
        assert!(
            ps.contains(PolicyFlag::Runtime),
            "field function should have Runtime"
        );

        let SymbolPayload::FieldFunction(field_object) = &symbol.payload else {
            panic!("expected field-function payload");
        };
        assert_eq!(field_object.projection, expected_projection);

        let body_policy = &field_object.callable_policy.body_entry_policy;
        assert!(
            body_policy.policy_set.contains(PolicyFlag::Runtime),
            "field body should have Runtime"
        );
        assert!(
            !body_policy.policy_set.contains(PolicyFlag::Meta),
            "field body should not have Meta"
        );

        let return_policy = &field_object.callable_policy.return_object_policy;
        assert!(
            return_policy.policy_set.contains(PolicyFlag::Runtime),
            "field return object should have Runtime"
        );
        assert!(
            !return_policy.policy_set.contains(PolicyFlag::Meta),
            "field return object should not have Meta"
        );

        assert!(
            !callable_body_allows_execution(body_policy, ExecutionEnv::Meta),
            "Meta lookup visibility must not imply Meta body execution"
        );
        assert!(
            callable_body_allows_execution(body_policy, ExecutionEnv::Runtime),
            "field body should allow Runtime execution"
        );
    }
}
