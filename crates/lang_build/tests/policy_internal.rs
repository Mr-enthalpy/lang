mod support;
use support::*;

use lang_build::{
    callable_body_allows_execution, policy_metadata, policy_set_meta, policy_set_runtime,
    CompilationWorld, CoreMetaFunction, ExecutionEnv, MetaFunctionObject, PolicyEnv,
    PolicyMetadata, Provenance, ResolveExpectation, ResolverCode, SourceCategory, SymbolKind,
    SymbolObject, SymbolPayload,
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
fn callable_body_execution_helper_uses_policy_flags() {
    // Compiler-internal helper truth table; source verification covers ordinary
    // callable policy facts for core and generated symbols.
    let meta_policy = policy_metadata(policy_set_meta());
    let runtime_policy = policy_metadata(policy_set_runtime());

    assert!(callable_body_allows_execution(
        &meta_policy,
        ExecutionEnv::Meta
    ));
    assert!(!callable_body_allows_execution(
        &meta_policy,
        ExecutionEnv::Runtime
    ));
    assert!(!callable_body_allows_execution(
        &runtime_policy,
        ExecutionEnv::Meta
    ));
    assert!(callable_body_allows_execution(
        &runtime_policy,
        ExecutionEnv::Runtime
    ));
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
