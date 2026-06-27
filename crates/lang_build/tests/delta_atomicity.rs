mod support;
use support::*;

use lang_build::meta::try_expand_early_meta_initializer;
use lang_build::{
    ChildLink, ChildNameRole, CompilationWorld, NamespaceGraphSnapshot, NamespaceNodeId,
    Provenance, ResolverContext, SourceCategory, SymbolKind, SymbolObject,
};

#[test]
fn delta_transaction_installs_all_or_nothing_and_retains_diagnostics() {
    let snapshot = NamespaceGraphSnapshot::new();
    let root = snapshot.root_node();
    let mut delta = snapshot.empty_delta();
    let a = delta.allocate_symbol_id();
    let b = delta.allocate_symbol_id();
    delta.insert_symbol(
        root,
        SymbolObject::placeholder(
            a,
            "A",
            SymbolKind::Placeholder,
            SourceCategory::DeclaredSymbol,
            Some(root),
            Provenance::new("test A"),
        ),
    );
    delta.insert_symbol(
        root,
        SymbolObject::placeholder(
            b,
            "B",
            SymbolKind::Placeholder,
            SourceCategory::DeclaredSymbol,
            Some(root),
            Provenance::new("test B"),
        ),
    );
    let snapshot = snapshot.install_delta(delta).expect("install full delta");
    let context = ResolverContext::new(root);
    assert!(snapshot.capability().resolve_str("A", &context).is_ok());
    assert!(snapshot.capability().resolve_str("B", &context).is_ok());

    let mut conflict = snapshot.empty_delta();
    let x1 = conflict.allocate_symbol_id();
    let x2 = conflict.allocate_symbol_id();
    conflict.insert_symbol(
        root,
        SymbolObject::placeholder(
            x1,
            "X",
            SymbolKind::Placeholder,
            SourceCategory::DeclaredSymbol,
            Some(root),
            Provenance::new("test X1"),
        ),
    );
    conflict.insert_symbol(
        root,
        SymbolObject::placeholder(
            x2,
            "X",
            SymbolKind::Placeholder,
            SourceCategory::DeclaredSymbol,
            Some(root),
            Provenance::new("test X2"),
        ),
    );
    let error = snapshot
        .install_delta(conflict)
        .expect_err("conflicting delta must fail");
    assert!(!error.diagnostics.is_empty());
    assert!(
        snapshot.capability().resolve_str("X", &context).is_err(),
        "failed delta must not install partial symbols"
    );
}

#[test]
fn conflicting_delta_with_valid_symbol_installs_nothing() {
    let snapshot = NamespaceGraphSnapshot::new();
    let root = snapshot.root_node();
    let mut initial = snapshot.empty_delta();
    let existing_b = initial.allocate_symbol_id();
    initial.insert_symbol(
        root,
        placeholder_symbol(existing_b, root, "B", "existing B"),
    );
    let snapshot = snapshot.install_delta(initial).expect("install existing B");

    let mut delta = snapshot.empty_delta();
    let a = delta.allocate_symbol_id();
    let conflicting_b = delta.allocate_symbol_id();
    delta.insert_symbol(root, placeholder_symbol(a, root, "A", "valid A"));
    delta.insert_symbol(
        root,
        placeholder_symbol(conflicting_b, root, "B", "conflicting B"),
    );

    let error = snapshot
        .install_delta(delta)
        .expect_err("conflicting B rejects whole delta");
    assert!(error.diagnostics.iter().any(|diagnostic| {
        diagnostic.message.contains("B")
            && diagnostic
                .provenance
                .as_ref()
                .is_some_and(|provenance| provenance.description.contains("conflicting B"))
    }));
    let context = ResolverContext::new(root);
    assert!(snapshot.capability().resolve_str("A", &context).is_err());
    assert_eq!(
        snapshot.capability().resolve_str("B", &context).unwrap().id,
        existing_b
    );
}

#[test]
fn delta_with_missing_parent_or_duplicate_link_installs_nothing() {
    let snapshot = NamespaceGraphSnapshot::new();
    let root = snapshot.root_node();

    let mut missing_parent = snapshot.empty_delta();
    let orphan = missing_parent.allocate_symbol_id();
    missing_parent.insert_symbol(
        NamespaceNodeId(99_999),
        placeholder_symbol(orphan, NamespaceNodeId(99_999), "orphan", "missing parent"),
    );
    let error = snapshot
        .install_delta(missing_parent)
        .expect_err("missing parent rejects delta");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("parent namespace node")));
    assert!(snapshot
        .capability()
        .resolve_str("orphan", &ResolverContext::new(root))
        .is_err());

    let mut duplicate_link = snapshot.empty_delta();
    let symbol_id = duplicate_link.allocate_symbol_id();
    let symbol = placeholder_symbol(symbol_id, root, "dup", "duplicate link");
    duplicate_link.symbols.insert(symbol_id, symbol);
    duplicate_link.child_links.push(ChildLink {
        parent: root,
        name: "dup".to_string(),
        symbol: symbol_id,
        role: ChildNameRole::Object,
        provenance: Provenance::new("first duplicate link"),
    });
    duplicate_link.child_links.push(ChildLink {
        parent: root,
        name: "dup".to_string(),
        symbol: symbol_id,
        role: ChildNameRole::Object,
        provenance: Provenance::new("second duplicate link"),
    });
    let error = snapshot
        .install_delta(duplicate_link)
        .expect_err("duplicate child link rejects delta");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("duplicate symbol `dup`")));
    assert!(snapshot
        .capability()
        .resolve_str("dup", &ResolverContext::new(root))
        .is_err());
}

#[test]
fn generated_type_delta_conflict_installs_no_generated_fields() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let mut initial = world.snapshot().empty_delta();
    let existing_t = initial.allocate_symbol_id();
    initial.insert_symbol(
        world.package_root_node(),
        placeholder_symbol(
            existing_t,
            world.package_root_node(),
            "T",
            "preexisting T conflict",
        ),
    );
    let snapshot = world
        .snapshot()
        .install_delta(initial)
        .expect("install existing T");

    let initializer = initializer_from_source("let T: type = (uint8 a) |> struct");
    let expansion = try_expand_early_meta_initializer(
        &snapshot,
        world.package_root_node(),
        "T",
        &initializer,
        &world.package_context(),
        Provenance::new("generated T conflict"),
    )
    .expect("meta expansion result")
    .expect("struct expansion");
    let error = snapshot
        .install_delta(expansion.namespace_delta)
        .expect_err("generated type collides with existing T");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("T")));
    assert!(snapshot
        .capability()
        .resolve_str("a::T", &world.package_context())
        .is_err());
}

#[test]
fn diagnostic_delta_duplicate_child_prefix() {
    let snapshot = NamespaceGraphSnapshot::new();
    let root = snapshot.root_node();

    let mut delta = snapshot.empty_delta();
    let dup_ns = delta.allocate_node_id();
    delta.nodes.insert(
        dup_ns,
        lang_build::NamespaceNode::new(
            dup_ns,
            "dup_ns",
            lang_build::NamespaceNodeKind::Virtual,
            SourceCategory::DeclaredSymbol,
            Some(root),
            Provenance::new("dup namespace"),
        ),
    );

    let sym = delta.allocate_symbol_id();
    delta.symbols.insert(
        sym,
        placeholder_symbol(sym, root, "existing", "existing symbol"),
    );

    let snapshot = snapshot.install_delta(delta).expect("base");

    let mut conflict = snapshot.empty_delta();
    let s1 = conflict.allocate_symbol_id();
    let s2 = conflict.allocate_symbol_id();
    conflict.symbols.insert(
        s1,
        placeholder_symbol(s1, root, "existing", "first conflict"),
    );
    conflict.symbols.insert(
        s2,
        placeholder_symbol(s2, root, "existing", "second conflict"),
    );
    conflict.child_links.push(ChildLink {
        parent: root,
        name: "existing".into(),
        symbol: s1,
        role: ChildNameRole::Object,
        provenance: Provenance::new("first link"),
    });
    conflict.child_links.push(ChildLink {
        parent: root,
        name: "existing".into(),
        symbol: s2,
        role: ChildNameRole::Object,
        provenance: Provenance::new("second link"),
    });

    let error = snapshot
        .install_delta(conflict)
        .expect_err("duplicate child link expected");
    assert!(
        error
            .diagnostics
            .iter()
            .any(|d| d.message.contains("delta install conflict:")),
        "prefix must be stable: {error:#?}"
    );
}
