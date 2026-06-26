mod support;
use support::*;

use lang_build::{CompilationWorld, SourceDiscoveryConfig, SymbolKind};

// A. Discovers `.lang` files from a real source root, through both the direct
// discovery layer and the full `from_manifest` pipeline.
#[test]
fn discovers_lang_files_from_real_source_root() {
    let project = TempProject::new("discovery_basic");
    project.write("src/main.lang", "let A: type = uint8");
    project.write("src/a/b/foo.lang", "let B: type = uint8");
    let manifest = app_manifest(&project.path().join("src"));

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

// B. Directory path contributes namespace; the file name does not.
#[test]
fn directory_contributes_namespace_file_name_does_not() {
    let project = TempProject::new("discovery_namespace_dir");
    project.write("src/a/b/foo.lang", "let T: type = uint8");
    let manifest = app_manifest(&project.path().join("src"));
    let world = CompilationWorld::from_manifest(&manifest).expect("build world");

    let capability = world.snapshot().capability();
    let context = world.root_context();

    // Physical directories `a` and `b` contribute namespace segments; `T` is a
    // direct child of `b` (reversed inner-to-outer resolver path order).
    assert!(
        capability.resolve_str("T::b::a::app", &context).is_ok(),
        "directory components contribute the physical namespace path"
    );

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
    let project = TempProject::new("discovery_same_namespace");
    project.write("src/math/vector.lang", "let Vec: type = uint8");
    project.write("src/math/matrix.lang", "let Mat: type = uint8");
    let manifest = app_manifest(&project.path().join("src"));
    let world = CompilationWorld::from_manifest(&manifest).expect("build world");

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

// D. Deterministic traversal: files created in unsorted order are discovered in
// stable lexical order over normalized source-root-relative path components.
#[test]
fn discovery_order_is_stable_and_sorted() {
    let project = TempProject::new("discovery_order");
    // Intentionally created out of lexical order.
    project.write("src/z.lang", "");
    project.write("src/a.lang", "");
    project.write("src/m/lang.lang", "");
    let manifest = app_manifest(&project.path().join("src"));

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
    let project = TempProject::new("discovery_ignore_non_lang");
    project.write("src/readme.md", "# readme");
    project.write("src/data.txt", "data");
    project.write("src/main.lang", "let M: type = uint8");
    let manifest = app_manifest(&project.path().join("src"));

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

// F. A missing source root is diagnosed without panic.
#[test]
fn missing_source_root_is_a_build_error() {
    let project = TempProject::new("discovery_missing_root");
    let missing = project.path().join("does_not_exist");
    let manifest = app_manifest(&missing);

    let report = SourceDiscoveryConfig::from_source_roots(&manifest.source_roots).discover();
    assert!(report.has_hard_errors());
    assert!(report.units.is_empty());

    let error = CompilationWorld::from_manifest(&manifest).expect_err("missing source root");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("does not exist")));
}

// G. A source root that is a file (not a directory) is diagnosed.
#[test]
fn source_root_that_is_a_file_is_a_build_error() {
    let project = TempProject::new("discovery_root_is_file");
    project.write("rootfile", "not a directory");
    let manifest = app_manifest(&project.path().join("rootfile"));

    let error = CompilationWorld::from_manifest(&manifest).expect_err("source root is a file");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("is not a directory")));
}

// H. A non-UTF-8 `.lang` file is diagnosed without panic and without lossy decode.
#[test]
fn non_utf8_lang_file_is_a_build_error() {
    let project = TempProject::new("discovery_non_utf8");
    project.write_bytes("src/main.lang", &[0xff, 0xfe, 0x00, 0x9f]);
    let manifest = app_manifest(&project.path().join("src"));

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
    let project = TempProject::new("discovery_duplicate_decl");
    project.write("src/math/vector.lang", "let Dup: type = uint8");
    project.write("src/math/matrix.lang", "let Dup: type = uint8");
    let manifest = app_manifest(&project.path().join("src"));

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
    let project = TempProject::new("discovery_fragment_identity");
    project.write("src/foo.lang", "let T: type = uint8");
    let manifest = app_manifest(&project.path().join("src"));
    let world = CompilationWorld::from_manifest(&manifest).expect("build world");

    let symbol = world
        .resolve("T")
        .expect("`T` resolves under the package root");
    assert_eq!(symbol.name, "T");
    assert_eq!(symbol.kind, SymbolKind::Type);
    assert_eq!(symbol.parent, Some(world.package_root_node()));

    assert!(
        world.resolve("foo").is_err(),
        "the file stem `foo` must not exist as a namespace"
    );

    let capability = world.snapshot().capability();
    let context = world.root_context();
    assert!(capability.resolve_str("T::app", &context).is_ok());
    assert!(
        capability.resolve_str("T::foo::app", &context).is_err(),
        "the file stem must not introduce a namespace segment"
    );
}
