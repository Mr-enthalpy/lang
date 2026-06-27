mod support;
use support::*;

use lang_build::{
    CompilationWorld, NamespaceNodeKind, SourceDiscoveryConfig, SourceRoot, SymbolKind,
};

// A. Discovers `.lang` files from a committed fixture source root, through both
// the direct discovery layer and the full `from_manifest` pipeline.
#[test]
fn discovers_lang_files_from_real_source_root() {
    let manifest = fixture_manifest("nested_physical_namespace", "app");

    let report = SourceDiscoveryConfig::from_source_roots(&manifest.source_roots).discover();
    assert!(
        !report.has_hard_errors(),
        "unexpected diagnostics: {report:#?}"
    );
    assert_eq!(report.units.len(), 2);
    assert!(report
        .units
        .iter()
        .all(|unit| !unit.content_hash.is_empty()));

    let world = CompilationWorld::from_manifest(&manifest).expect("build world");
    assert_eq!(world.source_fragments().len(), 2);
}

// B. Directory path contributes namespace; the file name does not. Intermediate
// directory components become physical namespace nodes.
#[test]
fn directory_contributes_namespace_file_name_does_not() {
    let world =
        CompilationWorld::from_manifest(&fixture_manifest("nested_physical_namespace", "app"))
            .expect("build world");

    let capability = world.snapshot().capability();
    let context = world.root_context();

    // Physical directories `a` and `b` contribute namespace segments; `T` is a
    // direct child of `b` (reversed inner-to-outer resolver path order).
    assert!(
        capability.resolve_str("T::b::a::app", &context).is_ok(),
        "directory components contribute the physical namespace path"
    );

    // The intermediate directory component is a real physical namespace node.
    let intermediate = capability
        .resolve_str("b::a::app", &context)
        .expect("intermediate physical namespace resolves");
    assert_eq!(intermediate.kind, SymbolKind::Namespace);
    assert_eq!(intermediate.node_kind, Some(NamespaceNodeKind::Physical));

    // The file stem `foo` is fragment identity only, never a namespace segment.
    assert!(
        capability
            .resolve_str("T::foo::b::a::app", &context)
            .is_err(),
        "file name must not contribute a namespace segment"
    );
    assert!(
        capability.resolve_str("foo::b::a::app", &context).is_err(),
        "no `foo` namespace exists unless a real `foo/` directory exists"
    );
}

// C. Multiple files in the same namespace directory are allowed.
#[test]
fn multiple_files_in_same_namespace_directory_are_allowed() {
    let world =
        CompilationWorld::from_manifest(&fixture_manifest("multi_file_same_namespace", "app"))
            .expect("build world");

    assert_eq!(world.source_fragments().len(), 2);
    assert_eq!(
        world.source_fragments()[0].namespace,
        world.source_fragments()[1].namespace,
        "both fragments attach to the same physical namespace node"
    );

    let capability = world.snapshot().capability();
    let context = world.root_context();
    assert!(capability.resolve_str("Vec::math::app", &context).is_ok());
    assert!(capability.resolve_str("Mat::math::app", &context).is_ok());
}

// D. Deterministic traversal: a fixture whose files are not in lexical order on
// disk is still discovered in stable lexical order over normalized relative path
// components.
#[test]
fn discovery_order_is_stable_and_sorted() {
    let manifest = fixture_manifest("discovery_order", "app");

    let report = SourceDiscoveryConfig::from_source_roots(&manifest.source_roots).discover();
    assert!(
        !report.has_hard_errors(),
        "unexpected diagnostics: {report:#?}"
    );

    let observed: Vec<(Vec<String>, String)> = report
        .units
        .iter()
        .map(|unit| (unit.namespace_dir.clone(), unit.fragment_name.clone()))
        .collect();
    assert_eq!(
        observed,
        vec![
            (vec![], "a.lang".to_string()),
            (vec!["m".to_string()], "lang.lang".to_string()),
            (vec![], "z.lang".to_string()),
        ],
        "discovered units must be lexically sorted by relative path components"
    );
}

// E. Only `.lang` files are discovered; the extension decides inclusion.
#[test]
fn non_lang_files_are_ignored() {
    let manifest = fixture_manifest("non_lang_files_ignored", "app");

    let report = SourceDiscoveryConfig::from_source_roots(&manifest.source_roots).discover();
    assert!(
        !report.has_hard_errors(),
        "unexpected diagnostics: {report:#?}"
    );
    assert_eq!(report.units.len(), 1);
    assert_eq!(report.units[0].fragment_name, "main.lang");

    let world = CompilationWorld::from_manifest(&manifest).expect("build world");
    assert_eq!(world.source_fragments().len(), 1);
}

// F. A missing source root is diagnosed without panic (invalid filesystem
// configuration, kept synthetic).
#[test]
fn missing_source_root_is_a_build_error() {
    let project = TempProject::new("discovery_missing_root");
    let missing = project.path().join("does_not_exist");
    let manifest = boundary_app_manifest(&missing);

    let report = SourceDiscoveryConfig::from_source_roots(&manifest.source_roots).discover();
    assert!(report.has_hard_errors());
    assert!(report.units.is_empty());

    let error = CompilationWorld::from_manifest(&manifest).expect_err("missing source root");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("does not exist")));
}

// G. A source root that is a file (not a directory) is diagnosed (invalid
// filesystem configuration, kept synthetic).
#[test]
fn source_root_that_is_a_file_is_a_build_error() {
    let project = TempProject::new("discovery_root_is_file");
    project.write_boundary_source("rootfile", "not a directory");
    let manifest = boundary_app_manifest(&project.path().join("rootfile"));

    let error = CompilationWorld::from_manifest(&manifest).expect_err("source root is a file");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("is not a directory")));
}

// H. A non-UTF-8 `.lang` file is diagnosed without panic and without lossy
// decode. Kept synthetic via raw bytes (we do not commit invalid UTF-8).
#[test]
fn non_utf8_lang_file_is_a_build_error() {
    let project = TempProject::new("discovery_non_utf8");
    project.write_bytes("src/main.lang", &[0xff, 0xfe, 0x00, 0x9f]);
    let manifest = boundary_app_manifest(&project.path().join("src"));

    let error = CompilationWorld::from_manifest(&manifest).expect_err("non-UTF-8 source file");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("not valid UTF-8")));
}

// I. Duplicate declarations remain a graph/declaration conflict, not a discovery
// conflict. Discovery succeeds; the world build fails at namespace-graph install.
#[test]
fn duplicate_declarations_conflict_at_graph_level_not_discovery() {
    let manifest = fixture_manifest("duplicate_declaration", "app");

    let report = SourceDiscoveryConfig::from_source_roots(&manifest.source_roots).discover();
    assert!(
        !report.has_hard_errors(),
        "discovery must not reject two files declaring the same name"
    );
    assert_eq!(report.units.len(), 2);

    let error = CompilationWorld::from_manifest(&manifest)
        .expect_err("duplicate declaration is a namespace-graph conflict");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("conflict")));
}

// J. File name is fragment identity only: a top-level file declaration lands
// under the package root, not under a namespace named after the file stem.
#[test]
fn file_name_is_fragment_identity_only() {
    let world =
        CompilationWorld::from_manifest(&fixture_manifest("single_package_type_binding", "app"))
            .expect("build world");

    let symbol = world
        .resolve("T")
        .expect("`T` resolves under the package root");
    assert_eq!(symbol.name, "T");
    assert_eq!(symbol.kind, SymbolKind::Type);
    assert_eq!(symbol.parent, Some(world.package_root_node()));

    assert!(
        world.resolve("main").is_err(),
        "the file stem `main` must not exist as a namespace"
    );

    let capability = world.snapshot().capability();
    let context = world.root_context();
    assert!(capability.resolve_str("T::app", &context).is_ok());
    assert!(
        capability.resolve_str("T::main::app", &context).is_err(),
        "the file stem must not introduce a namespace segment"
    );
}

// Duplicate physical identity: configuring the same source root path twice
// surfaces the same canonical `.lang` file more than once. Kept synthetic because
// it tests invalid source-root configuration, not language source structure.
#[test]
fn duplicate_physical_source_identity_is_a_hard_diagnostic() {
    let project = TempProject::new("discovery_duplicate_identity");
    project.write_boundary_source("src/main.lang", "let T: type = uint8");
    let src = project.path().join("src");

    let mut manifest = empty_app_manifest();
    manifest.source_roots.push(SourceRoot {
        path: src.clone(),
        namespace_root: vec!["app".to_string()],
    });
    manifest.source_roots.push(SourceRoot {
        path: src,
        namespace_root: vec!["app".to_string()],
    });

    let report = SourceDiscoveryConfig::from_source_roots(&manifest.source_roots).discover();
    assert!(report.has_hard_errors());
    assert!(
        report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("duplicate physical source identity")),
        "expected a duplicate physical source identity diagnostic: {report:#?}"
    );
}
