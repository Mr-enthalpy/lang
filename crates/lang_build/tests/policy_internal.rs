mod support;
use support::*;

use lang_build::{
    callable_body_allows_execution, policy_set_runtime, CompilationWorld, CoreMetaFunction,
    ExecutionEnv, MetaFunctionObject, PolicyEnv, PolicyFlag, PolicyMetadata, Provenance,
    ResolveExpectation, ResolverCode, SourceCategory, SymbolKind, SymbolObject, SymbolPayload,
};

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
fn runtime_only_value_is_invisible_under_meta_lookup() {
    // Compiler-internal resolver invariant: source verification observes the
    // policy flags, while this test checks PolicyEnv filtering.
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
