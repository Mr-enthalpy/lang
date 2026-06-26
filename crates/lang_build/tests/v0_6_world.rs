mod support;
use support::*;

use lang_build::{CompilationWorld, DiagnosticSeverity, NamespaceMount};

#[test]
fn missing_core_mount_is_a_build_error() {
    let mut manifest = empty_app_manifest();
    manifest.default_core_mount = false;

    let error = CompilationWorld::from_manifest(&manifest).expect_err("missing core mount");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("build manifest error")));
}

#[test]
fn duplicate_dependency_mount_root_is_hard_error() {
    let mut manifest = empty_app_manifest();
    manifest
        .dependency_mounts
        .push(NamespaceMount::synthetic_root(
            "dup",
            vec!["core".to_string()],
        ));

    let error = CompilationWorld::from_manifest(&manifest)
        .expect_err("dependency mount colliding with core root");
    assert!(error.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == DiagnosticSeverity::HardError
            && diagnostic.message.contains("duplicate mount root `core`")
    }));
}

#[test]
fn policy_metadata_slots_are_preserved_without_policy_checking() {
    use lang_build::{
        NamespaceGraphSnapshot, Provenance, ResolverContext, SourceCategory, SymbolKind,
    };

    let snapshot = NamespaceGraphSnapshot::new();
    let root = snapshot.root_node();
    let mut delta = snapshot.capability().declare(
        root,
        "policy_symbol",
        SymbolKind::Placeholder,
        SourceCategory::DeclaredSymbol,
        Provenance::new("policy test"),
    );
    let symbol = delta
        .symbols
        .values_mut()
        .next()
        .expect("declared symbol in delta");
    symbol
        .policy_metadata
        .slots
        .insert("entry".to_string(), "compile".to_string());

    let snapshot = snapshot
        .install_delta(delta)
        .expect("install policy symbol");
    let symbol = snapshot
        .capability()
        .resolve_str("policy_symbol", &ResolverContext::new(root))
        .expect("resolve policy symbol");
    assert_eq!(
        symbol
            .policy_metadata
            .slots
            .get("entry")
            .map(String::as_str),
        Some("compile")
    );
    assert!(symbol.visibility_metadata.slots.is_empty());
}

#[test]
fn representative_diagnostics_contain_useful_text_and_provenance() {
    // Diagnostic/provenance boundary: kept synthetic because each source is an
    // intentional conflict or rejected contribution and the test checks the
    // diagnostic text and provenance, not an ordinary successful build.
    let project = TempProject::new("diagnostic_conflict");
    project.write("src/T/placeholder.lang", "");
    project.write("src/main.lang", "let T: type = uint8");
    let error = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect_err("conflict");
    assert!(error.diagnostics.iter().any(|diagnostic| {
        diagnostic.message.contains("conflict")
            && diagnostic.provenance.as_ref().is_some_and(|provenance| {
                !provenance.description.is_empty()
                    || provenance.file.is_some()
                    || provenance.span.is_some()
            })
    }));

    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let unresolved = world
        .resolve("Nope::core")
        .expect_err("unresolved explicit path");
    assert!(unresolved.message.contains("Nope::core"));

    let project = TempProject::new("diagnostic_descendant");
    project.write("src/main.lang", "let a::T = uint8");
    let error = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect_err("descendant injection");
    assert!(error.diagnostics.iter().any(|diagnostic| {
        diagnostic.message.contains("parent-to-descendant")
            && diagnostic
                .provenance
                .as_ref()
                .is_some_and(|provenance| provenance.span.is_some())
    }));
}
