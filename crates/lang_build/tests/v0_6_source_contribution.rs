mod support;
use support::*;

use lang_build::{DiagnosticSeverity, ResolveExpectation, SourceCategory, SymbolPayload};

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
    // Committed fixture that intentionally fails: a physical directory `T`
    // collides with a declared symbol `T`.
    let error = build_fixture_error("source_conflict_physical_dir_symbol", "app");
    assert!(error.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == DiagnosticSeverity::HardError
            && diagnostic.message.contains("conflict")
    }));
}

#[test]
fn ordinary_parent_to_descendant_injection_is_rejected() {
    // Committed fixture that intentionally fails: `let a::T = uint8` is an
    // unsupported descendant injection.
    let error = build_fixture_error("descendant_injection", "app");
    assert!(error.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("ordinary parent-to-descendant injection")));
}

#[test]
fn ordinary_source_contribution_rejects_deep_and_pattern_binders() {
    // Committed fixtures that intentionally fail: each uses an unsupported
    // top-level binder. The ordinary direct-child success path is covered by the
    // `single_package_type_binding` fixture.
    for (fixture, expected) in [
        (
            "deep_descendant_injection",
            "ordinary parent-to-descendant injection",
        ),
        (
            "product_binder_rejected",
            "unsupported top-level declaration binder",
        ),
        (
            "discard_binder_rejected",
            "ordinary parent-to-descendant injection",
        ),
    ] {
        let error = build_fixture_error(fixture, "app");
        assert!(
            error
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "missing expected diagnostic {expected:?} for `{fixture}`: {:#?}",
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
    // Committed fixture that intentionally fails: descendant injection through an
    // alias is rejected in v0.6.
    let error = build_fixture_error("alias_external_injection_future", "app");

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
    // Committed fixture that intentionally fails: checks the stable
    // `source contribution error:` prefix on a rejected contribution.
    let error = build_fixture_error("diagnostic_source_contribution_prefix", "app");
    assert!(
        error
            .diagnostics
            .iter()
            .any(|d| d.message.contains("source contribution error:")),
        "prefix must be stable: {error:#?}"
    );
}
