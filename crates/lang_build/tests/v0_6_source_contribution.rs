mod support;
use support::*;

use lang_build::{
    CompilationWorld, DiagnosticSeverity, ResolveExpectation, SourceCategory, SymbolPayload,
};

#[test]
fn source_level_import_use_include_module_remain_ordinary_expressions() {
    // `import` / `use` / `include` / `module` are ordinary expression atoms, not
    // declarations: building the committed fixture must not create any of them as
    // symbols.
    let world = build_single_fixture_world("no_import_syntax", "app");
    assert!(
        world.resolve("import").is_err(),
        "source-level import syntax must not create an import symbol"
    );
    assert!(world.resolve("use").is_err());
    assert!(world.resolve("include").is_err());
    assert!(world.resolve("module").is_err());
}

#[test]
fn conflict_is_hard_error_and_blocks_installation() {
    // Namespace-graph conflict boundary: a physical directory `T` collides with a
    // declared symbol `T`. Kept synthetic because the test checks the conflict
    // diagnostic, not an ordinary successful build.
    let project = TempProject::new("conflict");
    project.write("src/T/placeholder.lang", "");
    project.write("src/main.lang", "let T: type = uint8");

    let error = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect_err("physical directory and declared symbol must conflict");
    assert!(error.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == DiagnosticSeverity::HardError
            && diagnostic.message.contains("conflict")
    }));
}

#[test]
fn ordinary_parent_to_descendant_injection_is_rejected() {
    // Rejected-contribution boundary: `let a::T = uint8` is intentionally an
    // unsupported descendant injection; the test checks the rejection diagnostic.
    let project = TempProject::new("descendant_injection");
    project.write("src/main.lang", "let a::T = uint8");

    let error = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect_err("ordinary file contribution cannot inject descendants");
    assert!(error.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("ordinary parent-to-descendant injection")));
}

#[test]
fn ordinary_source_contribution_rejects_deep_and_pattern_binders() {
    // Rejected-contribution boundary: each source intentionally uses an
    // unsupported top-level binder and the test checks the error diagnostic. The
    // ordinary direct-child success path is covered by the committed
    // `single_package_type_binding` fixture.
    for (case_name, source, expected) in [
        (
            "deep_descendant",
            "let a::b::T = uint8",
            "ordinary parent-to-descendant injection",
        ),
        (
            "product_binder",
            "let (a, b) = uint8",
            "unsupported top-level declaration binder",
        ),
        (
            "discard_binder",
            "let _ = uint8",
            "ordinary parent-to-descendant injection",
        ),
    ] {
        let project = TempProject::new(case_name);
        project.write("src/main.lang", source);
        let error = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
            .expect_err("unsupported contribution should fail");
        assert!(
            error
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "missing expected diagnostic {expected:?}: {:#?}",
            error.diagnostics
        );
    }
}

#[test]
fn type_value_binding_placeholder_keeps_fresh_symbol_place_in_v0_6() {
    let world = build_single_fixture_world("single_package_type_binding", "app");
    let symbol = world
        .resolve_with_expectation("T", ResolveExpectation::TypeObject)
        .expect("resolve v0.6 placeholder type binding");
    assert_eq!(symbol.name, "T");
    assert_eq!(symbol.source_category, SourceCategory::DeclaredSymbol);
    assert_eq!(symbol.parent, Some(world.package_root_node()));
    let symbol_id = symbol.id;

    // GUARD: This test must NOT be read as final fresh nominal type generation.
    // The placeholder `TypeObject` exists only because type-value evaluation,
    // TypeValueId, and writable-place checking are not yet implemented.
    // When those features land, this test must be replaced with a proper
    // type-value binding test (fresh symbol/place `T` bound to existing type
    // value `uint8`, place(T) != place(uint8)).
    //
    // v0.6 placeholder behavior only: this is not final type-value semantics.
    // Long-term, `let T: type = uint8` binds fresh symbol/place `T` to the
    // existing type value `uint8`; injection through `T` targets place(T), not
    // place(uint8).
    let SymbolPayload::Type(type_object) = symbol.payload else {
        panic!("expected placeholder Type payload");
    };
    assert_eq!(type_object.type_symbol_id, symbol_id);
    assert!(type_object.type_associated_namespace.is_some());
}

#[test]
fn alias_injection_writability_is_future_not_current_contribution_rule() {
    // Rejected-contribution boundary: descendant injection through an alias is
    // intentionally rejected in v0.6; the test checks the rejection diagnostic.
    let project = TempProject::new("alias_external_injection_future");
    project.write("src/main.lang", "let t === uint8; let f::t = uint8");

    let error = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect_err("v0.6 rejects descendant contribution before alias-place writability");

    // Future semantics must reject this because `t` forwards to the external
    // stable built-in place(uint8), which is readable/aliasable but not writable
    // from the current lexical level. The v0.6 vertical slice has no alias
    // forwarding or writable-place checker yet, so the current diagnostic is
    // the existing source-contribution boundary.
    assert!(error.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("ordinary parent-to-descendant injection")));
}

#[test]
fn diagnostic_source_contribution_prefix() {
    // Error-diagnostic boundary: checks the stable `source contribution error:`
    // prefix on a rejected contribution.
    let project = TempProject::new("diag_sc_prefix");
    project.write("src/main.lang", "let a::T = uint8");
    let error = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect_err("descendant injection");
    assert!(
        error
            .diagnostics
            .iter()
            .any(|d| d.message.contains("source contribution error:")),
        "prefix must be stable: {error:#?}"
    );
}
