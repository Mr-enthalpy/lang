mod support;
use support::*;

use lang_build::{
    callable_body_allows_execution, policy_metadata, policy_set_compile, policy_set_meta,
    policy_set_runtime, policy_set_seal, CompilationWorld, CoreMetaFunction, ExecutionEnv,
    MetaFunctionObject, PolicyEnv, PolicyMetadata, Provenance, ResolveExpectation, ResolverCode,
    SourceCategory, SymbolKind, SymbolObject, SymbolPayload,
};

#[test]
fn uint8_resolves_under_open_static_compatibility_view() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let capability = world.namespace_projection().capability();
    let context = world.package_context();

    let symbol = capability
        .resolve_type_object_with_policy("uint8", &context, PolicyEnv::OpenStatic)
        .expect("uint8 should resolve under OpenStatic compatibility view");
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
        ExecutionEnv::OpenStatic
    ));
    assert!(!callable_body_allows_execution(
        &meta_policy,
        ExecutionEnv::Runtime
    ));
    assert!(!callable_body_allows_execution(
        &runtime_policy,
        ExecutionEnv::OpenStatic
    ));
    assert!(callable_body_allows_execution(
        &runtime_policy,
        ExecutionEnv::Runtime
    ));
}

#[test]
fn runtime_only_value_compatibility_filter_does_not_define_symbol_existence() {
    // The unfiltered resolver establishes symbol identity first. The legacy
    // PolicyEnv adapter can still return a filtered diagnostic, but canonical
    // phase exposure must not reinterpret that adapter result as nonexistence.
    let world = build_single_fixture_world("user_runtime_values", "app");
    let context = world.package_context();
    let capability = world.namespace_projection().capability();

    let symbol = capability
        .resolve(&["x".to_string()], &context)
        .expect("runtime symbol exists independently of phase exposure");
    assert_eq!(symbol.name, "x");

    let diagnostic = capability
        .resolve_with_policy(
            &["x".to_string()],
            &context,
            ResolveExpectation::Object,
            PolicyEnv::OpenStatic,
        )
        .expect_err("compatibility OpenStatic adapter filters the runtime value flag");
    assert_eq!(diagnostic.code, Some(ResolverCode::Unresolved));
}

#[test]
fn runtime_only_meta_function_is_filtered_by_legacy_open_static_adapter() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let mut delta = world.namespace_projection().empty_delta();

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
        primitive: Some(CoreMetaFunction::Assert),
        source_callable: None,
        function_policy: PolicyMetadata::default(),
        body_entry_policy: PolicyMetadata::default(),
        return_object_policy: PolicyMetadata::default(),
        return_shape: lang_build::ReturnShape::SingleVal(
            lang_build::PatternConstraint::Unconstrained,
        ),
        privilege: lang_build::CallablePrivilege::BuiltinPrivileged,
    });
    delta.insert_symbol(world.package_root_node(), local_struct);

    let snapshot = world
        .namespace_projection()
        .install_delta(delta)
        .expect("install delta");
    let context = world.package_context();

    let result = snapshot.capability().resolve_meta_function_with_policy(
        "struct",
        &context,
        PolicyEnv::OpenStatic,
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
fn seal_lookup_uses_visibility_domains_without_granting_execution() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let mut delta = world.namespace_projection().empty_delta();
    for (name, policy_set) in [
        ("meta_only", policy_set_meta()),
        ("compile_only", policy_set_compile()),
        ("seal_only", policy_set_seal()),
    ] {
        let symbol_id = delta.allocate_symbol_id();
        let mut symbol = SymbolObject::placeholder(
            symbol_id,
            name,
            SymbolKind::Placeholder,
            SourceCategory::DeclaredSymbol,
            Some(world.package_root_node()),
            Provenance::new(name),
        );
        symbol.policy_metadata.policy_set = policy_set;
        delta.insert_symbol(world.package_root_node(), symbol);
    }
    let snapshot = world
        .namespace_projection()
        .install_delta(delta)
        .expect("install policy fixtures");
    let context = world.package_context();
    let resolve = |name: &str, env| {
        snapshot.capability().resolve_with_policy(
            &[name.to_string()],
            &context,
            ResolveExpectation::Object,
            env,
        )
    };

    assert!(resolve("meta_only", PolicyEnv::SealStatic).is_err());
    assert!(resolve("compile_only", PolicyEnv::SealStatic).is_ok());
    assert!(resolve("seal_only", PolicyEnv::SealStatic).is_ok());
    assert!(resolve("seal_only", PolicyEnv::OpenStatic).is_err());
}
