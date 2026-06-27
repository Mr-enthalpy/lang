mod support;
use support::*;

use lang_build::{
    ChildLink, ChildNameRole, NamespaceGraphSnapshot, NamespaceNodeKind, Provenance,
    ResolveExpectation, ResolverContext, SourceCategory, SymbolKind,
};

#[test]
fn same_parent_same_role_object_conflict() {
    let snapshot = NamespaceGraphSnapshot::new();
    let root = snapshot.root_node();

    let mut delta = snapshot.empty_delta();
    let a1 = delta.allocate_symbol_id();
    let a2 = delta.allocate_symbol_id();
    delta
        .symbols
        .insert(a1, placeholder_symbol(a1, root, "X", "first X"));
    delta
        .symbols
        .insert(a2, placeholder_symbol(a2, root, "X", "second X"));
    delta.child_links.push(ChildLink {
        parent: root,
        name: "X".into(),
        symbol: a1,
        role: ChildNameRole::Object,
        provenance: Provenance::new("first X"),
    });
    delta.child_links.push(ChildLink {
        parent: root,
        name: "X".into(),
        symbol: a2,
        role: ChildNameRole::Object,
        provenance: Provenance::new("second X"),
    });

    let error = snapshot
        .install_delta(delta)
        .expect_err("same parent + same name + same role must be hard error");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("already exists")
            || diagnostic.message.contains("duplicate")));
}

#[test]
fn cross_role_coexistence_non_namespace_capable() {
    let snapshot = NamespaceGraphSnapshot::new();
    let root = snapshot.root_node();

    let mut delta = snapshot.empty_delta();
    let namespace_node_id = delta.allocate_node_id();
    delta.nodes.insert(
        namespace_node_id,
        lang_build::NamespaceNode::new(
            namespace_node_id,
            "T<namespace>",
            NamespaceNodeKind::Virtual,
            SourceCategory::DeclaredSymbol,
            Some(root),
            Provenance::new("namespace node"),
        ),
    );

    let object_id = delta.allocate_symbol_id();
    let namespace_id = delta.allocate_symbol_id();
    delta.symbols.insert(
        object_id,
        placeholder_symbol(object_id, root, "T", "object role"),
    );
    delta.symbols.insert(
        namespace_id,
        namespace_symbol(namespace_id, root, "T", namespace_node_id, "namespace role"),
    );
    delta.child_links.push(ChildLink {
        parent: root,
        name: "T".into(),
        symbol: object_id,
        role: ChildNameRole::Object,
        provenance: Provenance::new("object link"),
    });
    delta.child_links.push(ChildLink {
        parent: root,
        name: "T".into(),
        symbol: namespace_id,
        role: ChildNameRole::NamespaceSubspace,
        provenance: Provenance::new("namespace link"),
    });

    let snapshot = snapshot
        .install_delta(delta)
        .expect("non-namespace-capable object + namespace-subspace must coexist");

    let context = ResolverContext::new(root);

    let by_object = snapshot
        .capability()
        .resolve_str_with_expectation("T", &context, ResolveExpectation::Object)
        .expect("Object expectation resolves object-role symbol");
    assert_eq!(by_object.kind, SymbolKind::Placeholder);

    let by_namespace = snapshot
        .capability()
        .resolve_str_with_expectation("T", &context, ResolveExpectation::NamespaceSubspace)
        .expect("NamespaceSubspace expectation resolves namespace-role symbol");
    assert_eq!(by_namespace.kind, SymbolKind::Namespace);

    let any_unique = snapshot
        .capability()
        .resolve_str("T", &context)
        .expect_err("AnyUnique must be ambiguous with both roles present");
    assert!(any_unique.message.contains("ambiguous"));
}

#[test]
fn namespace_capable_object_cross_role_rejected() {
    let snapshot = NamespaceGraphSnapshot::new();
    let root = snapshot.root_node();

    let mut delta = snapshot.empty_delta();
    let type_namespace_id = delta.allocate_node_id();
    delta.nodes.insert(
        type_namespace_id,
        lang_build::NamespaceNode::new(
            type_namespace_id,
            "T<type-associated>",
            NamespaceNodeKind::Virtual,
            SourceCategory::TypeAssociatedNamespace,
            Some(root),
            Provenance::new("type-associated namespace"),
        ),
    );

    let subject_namespace_id = delta.allocate_node_id();
    delta.nodes.insert(
        subject_namespace_id,
        lang_build::NamespaceNode::new(
            subject_namespace_id,
            "T<subject-namespace>",
            NamespaceNodeKind::Virtual,
            SourceCategory::DeclaredSymbol,
            Some(root),
            Provenance::new("subject namespace node"),
        ),
    );

    let type_id = delta.allocate_symbol_id();
    let namespace_id = delta.allocate_symbol_id();
    delta.symbols.insert(
        type_id,
        type_with_namespace(type_id, "T", root, type_namespace_id, "type object"),
    );
    delta.symbols.insert(
        namespace_id,
        namespace_symbol(
            namespace_id,
            root,
            "T",
            subject_namespace_id,
            "namespace role",
        ),
    );
    delta.child_links.push(ChildLink {
        parent: root,
        name: "T".into(),
        symbol: type_id,
        role: ChildNameRole::Object,
        provenance: Provenance::new("type link"),
    });
    delta.child_links.push(ChildLink {
        parent: root,
        name: "T".into(),
        symbol: namespace_id,
        role: ChildNameRole::NamespaceSubspace,
        provenance: Provenance::new("namespace link"),
    });

    let error = snapshot
        .install_delta(delta)
        .expect_err("namespace-capable type + namespace-subspace with same name must be rejected");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("namespace-capable")));
}

#[test]
fn terminal_any_unique_ambiguous_with_both_roles() {
    let snapshot = NamespaceGraphSnapshot::new();
    let root = snapshot.root_node();

    let mut delta = snapshot.empty_delta();
    let ns_node = delta.allocate_node_id();
    delta.nodes.insert(
        ns_node,
        lang_build::NamespaceNode::new(
            ns_node,
            "ref<namespace>",
            NamespaceNodeKind::Virtual,
            SourceCategory::DeclaredSymbol,
            Some(root),
            Provenance::new("ref namespace"),
        ),
    );

    let object_id = delta.allocate_symbol_id();
    let namespace_id = delta.allocate_symbol_id();
    delta.symbols.insert(
        object_id,
        placeholder_symbol(object_id, root, "ref", "object-role ref"),
    );
    delta.symbols.insert(
        namespace_id,
        namespace_symbol(namespace_id, root, "ref", ns_node, "namespace-subspace ref"),
    );
    delta.child_links.push(ChildLink {
        parent: root,
        name: "ref".into(),
        symbol: object_id,
        role: ChildNameRole::Object,
        provenance: Provenance::new("object ref"),
    });
    delta.child_links.push(ChildLink {
        parent: root,
        name: "ref".into(),
        symbol: namespace_id,
        role: ChildNameRole::NamespaceSubspace,
        provenance: Provenance::new("namespace ref"),
    });

    let snapshot = snapshot
        .install_delta(delta)
        .expect("non-namespace-capable object + namespace-subspace coexists");

    let context = ResolverContext::new(root);
    let err = snapshot
        .capability()
        .resolve_str("ref", &context)
        .expect_err("AnyUnique must fail with both roles present");
    assert!(err.message.contains("ambiguous"));
}

#[test]
fn expectation_field_function_resolves_object_role() {
    let snapshot = NamespaceGraphSnapshot::new();
    let root = snapshot.root_node();

    let mut delta = snapshot.empty_delta();
    let ns_node = delta.allocate_node_id();
    delta.nodes.insert(
        ns_node,
        lang_build::NamespaceNode::new(
            ns_node,
            "ref<namespace>",
            NamespaceNodeKind::Virtual,
            SourceCategory::DeclaredSymbol,
            Some(root),
            Provenance::new("ref namespace"),
        ),
    );

    let field_id = delta.allocate_symbol_id();
    let namespace_id = delta.allocate_symbol_id();
    let mut field_symbol = placeholder_symbol(field_id, root, "ref", "field-function ref");
    field_symbol.kind = SymbolKind::FieldFunction;

    delta.symbols.insert(field_id, field_symbol);
    delta.symbols.insert(
        namespace_id,
        namespace_symbol(namespace_id, root, "ref", ns_node, "namespace-subspace ref"),
    );
    delta.child_links.push(ChildLink {
        parent: root,
        name: "ref".into(),
        symbol: field_id,
        role: ChildNameRole::Object,
        provenance: Provenance::new("field ref"),
    });
    delta.child_links.push(ChildLink {
        parent: root,
        name: "ref".into(),
        symbol: namespace_id,
        role: ChildNameRole::NamespaceSubspace,
        provenance: Provenance::new("namespace ref"),
    });

    let snapshot = snapshot
        .install_delta(delta)
        .expect("field function + namespace-subspace coexists");

    let context = ResolverContext::new(root);
    let symbol = snapshot
        .capability()
        .resolve_str_with_expectation("ref", &context, ResolveExpectation::FieldFunction)
        .expect("FieldFunction expectation resolves field function");
    assert_eq!(symbol.kind, SymbolKind::FieldFunction);
}

#[test]
fn expectation_namespace_subspace_resolves_namespace_role() {
    let snapshot = NamespaceGraphSnapshot::new();
    let root = snapshot.root_node();

    let mut delta = snapshot.empty_delta();
    let ns_node = delta.allocate_node_id();
    delta.nodes.insert(
        ns_node,
        lang_build::NamespaceNode::new(
            ns_node,
            "Namespc",
            NamespaceNodeKind::Virtual,
            SourceCategory::DeclaredSymbol,
            Some(root),
            Provenance::new("Namespc node"),
        ),
    );

    let obj_id = delta.allocate_symbol_id();
    let ns_id = delta.allocate_symbol_id();
    delta.symbols.insert(
        obj_id,
        placeholder_symbol(obj_id, root, "Namespc", "object-role"),
    );
    delta.symbols.insert(
        ns_id,
        namespace_symbol(ns_id, root, "Namespc", ns_node, "namespace-subspace"),
    );
    delta.child_links.push(ChildLink {
        parent: root,
        name: "Namespc".into(),
        symbol: obj_id,
        role: ChildNameRole::Object,
        provenance: Provenance::new("object link"),
    });
    delta.child_links.push(ChildLink {
        parent: root,
        name: "Namespc".into(),
        symbol: ns_id,
        role: ChildNameRole::NamespaceSubspace,
        provenance: Provenance::new("namespace link"),
    });

    let snapshot = snapshot
        .install_delta(delta)
        .expect("non-namespace-capable object + namespace-subspace coexists");

    let context = ResolverContext::new(root);
    let symbol = snapshot
        .capability()
        .resolve_str_with_expectation("Namespc", &context, ResolveExpectation::NamespaceSubspace)
        .expect("NamespaceSubspace expectation resolves namespace-role symbol");
    assert_eq!(symbol.kind, SymbolKind::Namespace);
}

#[test]
fn intermediate_component_uses_namespace_capable_parent() {
    let snapshot = NamespaceGraphSnapshot::new();
    let root = snapshot.root_node();

    let mut delta = snapshot.empty_delta();
    let ns_node = delta.allocate_node_id();
    delta.nodes.insert(
        ns_node,
        lang_build::NamespaceNode::new(
            ns_node,
            "ref<namespace>",
            NamespaceNodeKind::Virtual,
            SourceCategory::DeclaredSymbol,
            Some(root),
            Provenance::new("ref namespace"),
        ),
    );

    let object_id = delta.allocate_symbol_id();
    let namespace_id = delta.allocate_symbol_id();
    let deep_child_id = delta.allocate_symbol_id();

    delta.symbols.insert(
        object_id,
        placeholder_symbol(object_id, root, "ref", "object-role ref"),
    );
    delta.symbols.insert(
        namespace_id,
        namespace_symbol(namespace_id, root, "ref", ns_node, "namespace-subspace ref"),
    );
    let mut deep_child = placeholder_symbol(
        deep_child_id,
        ns_node,
        "DeepChild",
        "deep child of ref namespace",
    );
    deep_child.parent = Some(ns_node);
    delta.symbols.insert(deep_child_id, deep_child);

    delta.child_links.push(ChildLink {
        parent: root,
        name: "ref".into(),
        symbol: object_id,
        role: ChildNameRole::Object,
        provenance: Provenance::new("object ref"),
    });
    delta.child_links.push(ChildLink {
        parent: root,
        name: "ref".into(),
        symbol: namespace_id,
        role: ChildNameRole::NamespaceSubspace,
        provenance: Provenance::new("namespace ref"),
    });
    delta.child_links.push(ChildLink {
        parent: ns_node,
        name: "DeepChild".into(),
        symbol: deep_child_id,
        role: ChildNameRole::Object,
        provenance: Provenance::new("deep child"),
    });

    let snapshot = snapshot.install_delta(delta).expect("base delta installs");

    let context = ResolverContext::new(root);
    let deep = snapshot
        .capability()
        .resolve_str("DeepChild::ref", &context)
        .expect("intermediate ref resolves as NamespaceCapableParent for DeepChild");
    assert_eq!(deep.name, "DeepChild");
}

#[test]
fn same_parent_same_role_namespace_subspace_conflict() {
    let snapshot = NamespaceGraphSnapshot::new();
    let root = snapshot.root_node();

    let mut delta = snapshot.empty_delta();
    let ns1 = delta.allocate_node_id();
    let ns2 = delta.allocate_node_id();
    delta.nodes.insert(
        ns1,
        lang_build::NamespaceNode::new(
            ns1,
            "Y<ns1>",
            NamespaceNodeKind::Virtual,
            SourceCategory::DeclaredSymbol,
            Some(root),
            Provenance::new("first Y namespace"),
        ),
    );
    delta.nodes.insert(
        ns2,
        lang_build::NamespaceNode::new(
            ns2,
            "Y<ns2>",
            NamespaceNodeKind::Virtual,
            SourceCategory::DeclaredSymbol,
            Some(root),
            Provenance::new("second Y namespace"),
        ),
    );

    let ns1_id = delta.allocate_symbol_id();
    let ns2_id = delta.allocate_symbol_id();
    delta.symbols.insert(
        ns1_id,
        namespace_symbol(ns1_id, root, "Y", ns1, "first Y namespace"),
    );
    delta.symbols.insert(
        ns2_id,
        namespace_symbol(ns2_id, root, "Y", ns2, "second Y namespace"),
    );
    delta.child_links.push(ChildLink {
        parent: root,
        name: "Y".into(),
        symbol: ns1_id,
        role: ChildNameRole::NamespaceSubspace,
        provenance: Provenance::new("first Y link"),
    });
    delta.child_links.push(ChildLink {
        parent: root,
        name: "Y".into(),
        symbol: ns2_id,
        role: ChildNameRole::NamespaceSubspace,
        provenance: Provenance::new("second Y link"),
    });

    let error = snapshot
        .install_delta(delta)
        .expect_err("same parent + same name + same NamespaceSubspace role must be hard error");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("already exists")
            || diagnostic.message.contains("duplicate")));
}
