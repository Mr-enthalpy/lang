mod support;
use support::*;

use lang_build::{
    ChildLink, ChildNameRole, CompilationWorld, NamespaceMount, NamespaceNodeKind, Provenance,
    ResolverContext, SemanticNameIndex, SourceCategory, SymbolKind, SymbolPayload,
};

#[test]
fn verification_namespace_is_ordinary_and_operations_have_associated_call_entries() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).unwrap();
    let capability = world.namespace_projection().capability();
    let namespace = capability
        .resolve_str("verify::core", &world.package_context())
        .unwrap();
    assert_eq!(namespace.kind, SymbolKind::Namespace);
    assert!(matches!(namespace.payload, SymbolPayload::Namespace { .. }));
    let operation = capability
        .resolve_callable("exists::verify::core", &world.package_context())
        .unwrap();
    assert_eq!(operation.kind, SymbolKind::Callable);
    let SymbolPayload::Callable(declaration) = &operation.payload else {
        panic!("ordinary callable declaration projection");
    };
    assert!(matches!(
        declaration.implementation,
        lang_build::CallableImplementation::Builtin(lang_build::BuiltinCallableImpl::Verify(_))
    ));
    let binding = world
        .semantic_world()
        .symbol_in_namespace(namespace.namespace_node().unwrap(), "exists")
        .unwrap();
    let value = binding.ordinary_value().unwrap();
    let entries = world.semantic_world().callable_entries_for_value(value);
    assert!(!entries.is_empty());
    for entry in entries {
        let lang_build::SemanticValuePayload::CallEntry(entry) =
            &world.semantic_world().value(entry).unwrap().payload
        else {
            panic!("associated () must contain terminal call entries");
        };
        assert!(entry.source_closure().is_none());
    }
}

#[test]
fn short_and_explicit_core_paths_share_symbol_identity() {
    // Short and explicit paths preserve the same resolved graph identity.
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let context = world.package_context();
    let capability = world.namespace_projection().capability();

    let uint8_short = capability.resolve_str("uint8", &context).unwrap();
    let uint8_explicit = capability.resolve_str("uint8::core", &context).unwrap();
    assert_eq!(uint8_short.id, uint8_explicit.id);

    let struct_short = capability.resolve_str("struct", &context).unwrap();
    let struct_explicit = capability.resolve_str("struct::core", &context).unwrap();
    assert_eq!(struct_short.id, struct_explicit.id);

    let diagnostic = capability
        .resolve_str("Missing::core", &context)
        .expect_err("explicit mounted path should fail when target is absent");
    assert!(diagnostic.message.contains("Missing::core"));
}

#[test]
fn explicit_dependency_mounts_are_visible_as_paths() {
    let mut manifest = empty_app_manifest();
    manifest.dependency_mounts.push(
        NamespaceMount::synthetic_root("std", vec!["std".to_string()])
            .with_symbol("Vec", SymbolKind::Object),
    );

    let world = CompilationWorld::from_manifest(&manifest).expect("build world with mount");
    let std_symbol = world
        .namespace_projection()
        .capability()
        .resolve_str("std", &world.package_context())
        .expect("mounted root visible");
    assert_eq!(std_symbol.kind, SymbolKind::Namespace);
    assert_eq!(std_symbol.source_category, SourceCategory::DependencyMount);

    let vec_symbol = world
        .namespace_projection()
        .capability()
        .resolve_str("Vec::std", &world.package_context())
        .expect("synthetic mounted child visible through explicit path");
    assert_eq!(vec_symbol.name, "Vec");
    assert_eq!(vec_symbol.source_category, SourceCategory::DependencyMount);

    assert!(world
        .namespace_projection()
        .capability()
        .resolve_str("Vec::mylib", &world.package_context())
        .is_err());
}

#[test]
fn symbols_with_same_name_in_different_namespaces_have_distinct_ids() {
    // Equal spelling in distinct namespaces does not merge graph identities.
    let world = build_single_fixture_world("same_name_distinct_namespaces", "app");
    let left = world
        .namespace_projection()
        .capability()
        .resolve_str("T::left::app", &world.root_context())
        .expect("left T");
    let right = world
        .namespace_projection()
        .capability()
        .resolve_str("T::right::app", &world.root_context())
        .expect("right T");
    assert_eq!(left.name, right.name);
    assert_ne!(left.id, right.id);
    assert!(left.diagnostic_label().contains("symbol#"));
    assert!(left.diagnostic_label().contains("T"));
}

#[test]
fn typed_resolver_helpers_select_expected_kind() {
    // Typed helpers check the role of the fixed resolved graph entry.
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let capability = world.namespace_projection().capability();
    let context = world.package_context();

    let type_symbol = capability
        .resolve_complete_type_projection("uint8", &context)
        .expect("uint8 is a pure type Object");
    assert_eq!(type_symbol.kind, SymbolKind::CompleteTypeProjection);

    let meta_symbol = capability
        .resolve_callable("struct", &context)
        .expect("struct is a builtin callable");
    assert_eq!(meta_symbol.kind, SymbolKind::Callable);

    let error = capability
        .resolve_complete_type_projection("struct", &context)
        .expect_err("struct is a Callable, not a Type");
    assert!(error.message.contains("resolver error"));
}

#[test]
fn diagnostic_resolver_ambiguity_prefix() {
    let snapshot = SemanticNameIndex::new();
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
        object_symbol(object_id, root, "ambig", "object-role ambig"),
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
