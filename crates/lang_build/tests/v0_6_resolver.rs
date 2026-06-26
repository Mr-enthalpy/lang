mod support;
use support::*;

use lang_build::{
    ChildLink, ChildNameRole, CompilationWorld, DiagnosticSeverity, NamespaceGraphSnapshot,
    NamespaceMount, NamespaceNodeKind, Provenance, ResolverContext, SourceCategory, SymbolKind,
    SymbolPayload,
};

#[test]
fn core_bootstrap_installs_resolvable_symbol_objects() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let context = world.package_context();
    let capability = world.snapshot().capability();

    for name in ["struct", "assert", "uint8"] {
        let symbol = capability
            .resolve_str(name, &context)
            .expect("resolve core symbol through default mount");
        assert_eq!(symbol.name, name);
        assert!(symbol.id.as_u64() > 0);
    }

    let struct_symbol = capability.resolve_str("struct", &context).unwrap();
    assert_eq!(struct_symbol.kind, SymbolKind::MetaFunction);
    assert!(matches!(
        struct_symbol.payload,
        SymbolPayload::MetaFunction(_)
    ));

    let full_path = capability
        .resolve_str("struct::core", &world.root_context())
        .expect("resolve core path through graph");
    assert_eq!(full_path.id, struct_symbol.id);
}

#[test]
fn resolver_handles_short_and_explicit_mounted_core_paths() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let context = world.package_context();
    let capability = world.snapshot().capability();

    let uint8_short = capability.resolve_str("uint8", &context).unwrap();
    let uint8_explicit = capability.resolve_str("uint8::core", &context).unwrap();
    assert_eq!(uint8_short.id, uint8_explicit.id);
    assert_eq!(uint8_explicit.kind, SymbolKind::Type);

    let struct_short = capability.resolve_str("struct", &context).unwrap();
    let struct_explicit = capability.resolve_str("struct::core", &context).unwrap();
    assert_eq!(struct_short.id, struct_explicit.id);
    assert_eq!(struct_explicit.kind, SymbolKind::MetaFunction);

    let diagnostic = capability
        .resolve_str("Missing::core", &context)
        .expect_err("explicit mounted path should fail when target is absent");
    assert!(diagnostic.message.contains("Missing::core"));
}

#[test]
fn resolver_reports_current_namespace_conflict_with_default_mount() {
    // Resolver-conflict boundary: `let uint8 = uint8` deliberately collides a
    // local short name with the core short name. The build succeeds but the
    // resolve is a hard conflict. Kept synthetic because the test checks the
    // conflict diagnostic, not an ordinary successful resolution.
    let project = TempProject::new("core_conflict");
    project.write("src/main.lang", "let uint8 = uint8");
    let world = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect("build world");

    let diagnostic = world
        .resolve("uint8")
        .expect_err("local short name colliding with core short name is a hard conflict");
    assert!(diagnostic.message.contains("conflicting symbol `uint8`"));
    assert_eq!(diagnostic.severity, DiagnosticSeverity::HardError);
}

#[test]
fn dependency_mount_placeholders_are_visible_as_explicit_paths() {
    let mut manifest = empty_app_manifest();
    manifest.dependency_mounts.push(
        NamespaceMount::synthetic_root("std", vec!["std".to_string()])
            .with_symbol("Vec", SymbolKind::Placeholder),
    );

    let world = CompilationWorld::from_manifest(&manifest).expect("build world with mount");
    let std_symbol = world
        .snapshot()
        .capability()
        .resolve_str("std", &world.package_context())
        .expect("mounted root visible");
    assert_eq!(std_symbol.kind, SymbolKind::Namespace);
    assert_eq!(std_symbol.source_category, SourceCategory::DependencyMount);

    let vec_symbol = world
        .snapshot()
        .capability()
        .resolve_str("Vec::std", &world.package_context())
        .expect("synthetic mounted child visible through explicit path");
    assert_eq!(vec_symbol.name, "Vec");
    assert_eq!(vec_symbol.source_category, SourceCategory::DependencyMount);

    assert!(world
        .snapshot()
        .capability()
        .resolve_str("Vec::mylib", &world.package_context())
        .is_err());
}

#[test]
fn symbols_with_same_name_in_different_namespaces_have_distinct_ids() {
    let world = build_single_fixture_world("same_name_distinct_namespaces", "app");
    let left = world
        .snapshot()
        .capability()
        .resolve_str("T::left::app", &world.root_context())
        .expect("left T");
    let right = world
        .snapshot()
        .capability()
        .resolve_str("T::right::app", &world.root_context())
        .expect("right T");
    assert_eq!(left.name, right.name);
    assert_ne!(left.id, right.id);
    assert!(left.diagnostic_label().contains("symbol#"));
    assert!(left.diagnostic_label().contains("T"));
}

#[test]
fn resolve_type_object_uint8() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let capability = world.snapshot().capability();
    let context = world.package_context();

    let symbol = capability
        .resolve_type_object("uint8", &context)
        .expect("uint8 is a type object");
    assert_eq!(symbol.kind, SymbolKind::Type);
    assert_eq!(symbol.name, "uint8");
}

#[test]
fn resolve_meta_function_struct() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let capability = world.snapshot().capability();
    let context = world.package_context();

    let symbol = capability
        .resolve_meta_function("struct", &context)
        .expect("struct is a meta function");
    assert_eq!(symbol.kind, SymbolKind::MetaFunction);
    assert_eq!(symbol.name, "struct");
}

#[test]
fn resolve_type_object_non_type_fails() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let capability = world.snapshot().capability();
    let context = world.package_context();

    let error = capability
        .resolve_type_object("struct", &context)
        .expect_err("struct is a MetaFunction, not a Type");
    assert!(error.message.contains("resolver error"));
}

#[test]
fn resolve_field_function_ref_field() {
    let world = build_single_fixture_world("early_struct_meta", "app");
    let capability = world.snapshot().capability();
    let context = world.package_context();

    let symbol = capability
        .resolve_field_function("a::T", &context)
        .expect("a::T is a field function");
    assert_eq!(symbol.kind, SymbolKind::FieldFunction);
    assert!(matches!(symbol.payload, SymbolPayload::FieldFunction(_)));
}

#[test]
fn resolve_namespace_subspace_ref() {
    let world = build_single_fixture_world("early_struct_meta", "app");
    let capability = world.snapshot().capability();
    let context = world.package_context();

    let symbol = capability
        .resolve_namespace_subspace("ref::T", &context)
        .expect("ref::T is a namespace subspace");
    assert_eq!(symbol.kind, SymbolKind::Namespace);
}

#[test]
fn diagnostic_resolver_ambiguity_prefix() {
    let snapshot = NamespaceGraphSnapshot::new();
    let root = snapshot.root_node();

    let mut delta = snapshot.empty_delta();
    let ns_node = delta.allocate_node_id();
    delta.nodes.insert(
        ns_node,
        lang_build::NamespaceNode::new(
            ns_node,
            "ambig<namespace>",
            NamespaceNodeKind::Virtual,
            SourceCategory::DeclaredSymbol,
            Some(root),
            Provenance::new("ambig namespace"),
        ),
    );

    let object_id = delta.allocate_symbol_id();
    let namespace_id = delta.allocate_symbol_id();
    delta.symbols.insert(
        object_id,
        placeholder_symbol(object_id, root, "ambig", "object-role ambig"),
    );
    delta.symbols.insert(
        namespace_id,
        namespace_symbol(
            namespace_id,
            root,
            "ambig",
            ns_node,
            "namespace-subspace ambig",
        ),
    );
    delta.child_links.push(ChildLink {
        parent: root,
        name: "ambig".into(),
        symbol: object_id,
        role: ChildNameRole::Object,
        provenance: Provenance::new("object ambig"),
    });
    delta.child_links.push(ChildLink {
        parent: root,
        name: "ambig".into(),
        symbol: namespace_id,
        role: ChildNameRole::NamespaceSubspace,
        provenance: Provenance::new("namespace ambig"),
    });

    let snapshot = snapshot.install_delta(delta).expect("base delta");
    let err = snapshot
        .capability()
        .resolve_str("ambig", &ResolverContext::new(root))
        .expect_err("ambiguity expected");
    assert!(
        err.message.contains("resolver error: ambiguous"),
        "prefix must be stable: {err:?}"
    );
}
