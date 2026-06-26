mod support;
use support::*;

use lang_build::{
    policy_set_runtime, CompilationWorld, CoreMetaFunction, MetaFunctionObject, PolicyEnv,
    PolicyFlag, PolicyMetadata, Provenance, ResolveExpectation, SourceCategory, SymbolKind,
    SymbolObject, SymbolPayload,
};

#[test]
fn struct_expands_under_meta_policy() {
    let project = TempProject::new("meta_policy_struct");
    project.write(
        "src/main.lang",
        "let T: type = (uint8 a, uint8 b) |> struct",
    );
    let world = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect("build world");
    let type_symbol = world.resolve("T").expect("resolve generated type");
    assert_eq!(type_symbol.kind, SymbolKind::Type);
    let SymbolPayload::Type(type_object) = &type_symbol.payload else {
        panic!("expected type payload");
    };
    assert_eq!(type_object.field_names, ["a", "b"]);
}

#[test]
fn type_named_struct_does_not_shadow_core_meta_function() {
    let project = TempProject::new("type_named_struct");
    project.write(
        "src/main.lang",
        "let struct: type = uint8; let T: type = (uint8 a) |> struct",
    );
    let world = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect("struct expansion should succeed");
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
    let project = TempProject::new("runtime_call_target");
    project.write(
        "src/main.lang",
        "let not_meta = 1; let T: type = (uint8 a) |> not_meta",
    );
    let world = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect("runtime-only non-meta target should be soft miss");
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
fn user_declared_values_are_runtime_only() {
    let project = TempProject::new("user_runtime");
    project.write("src/main.lang", "let x = 1; let y: type = uint8");
    let world = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect("build world");

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
fn struct_generated_type_has_meta_runtime_policy() {
    let project = TempProject::new("generated_policy");
    project.write("src/main.lang", "let T: type = (uint8 a) |> struct");
    let world = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect("build world");
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
    let project = TempProject::new("ns_traversal");
    project.write("src/subns/main.lang", "let T: type = uint8");
    let world = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect("build world");
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
    let project = TempProject::new("ns_projection");
    project.write("src/main.lang", "let S: type = (uint8 a) |> struct");
    let world = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect("build world");
    let context = world.package_context();

    let symbol = world
        .snapshot()
        .capability()
        .resolve_with_policy(
            &["ref".to_string(), "S".to_string()],
            &context,
            ResolveExpectation::NamespaceSubspace,
            PolicyEnv::Meta,
        )
        .expect("ref::S projection namespace should resolve under Meta");
    assert_eq!(symbol.kind, SymbolKind::Namespace);
    assert_eq!(symbol.name, "ref");
}
