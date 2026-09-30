mod support;

use std::path::Path;

use lang_build::{ResolverCode, Stage};
use support::{build_fixture_error, build_single_fixture_world};

fn has_code(error: &lang_build::BuildError, code: ResolverCode) -> bool {
    error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == Some(code))
}

fn assert_symbol_stage(symbol: &lang_build::SymbolObject, stage: Stage) {
    assert!(
        symbol
            .policy_view
            .as_ref()
            .expect("Symbol Policy view")
            .pair
            .value
            .stage()
            == Some(stage)
    );
}

fn assert_symbol_not_stage(symbol: &lang_build::SymbolObject, stage: Stage) {
    assert!(
        symbol
            .policy_view
            .as_ref()
            .expect("Symbol Policy view")
            .pair
            .value
            .stage()
            != Some(stage)
    );
}

#[test]
fn let_type_annotation_is_post_rhs_assertion_not_meta_trigger() {
    let err = build_fixture_error("initializer_annotation_non_trigger", "app");
    assert!(has_code(
        &err,
        ResolverCode::UnsupportedDeferredTypeAssertion
    ));
    assert!(err.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("deferred for a residual initializer")));
}

#[test]
fn omitted_policy_is_inferred_runtime_for_residual_initializer() {
    let world = build_single_fixture_world("initializer_default_policy_residual", "app");
    let symbol = world
        .resolve_with_expectation("runtime_residual", lang_build::ResolveExpectation::Object)
        .expect("runtime residual symbol");
    assert_symbol_stage(&symbol, Stage::Runtime);
    assert_symbol_not_stage(&symbol, Stage::Meta);
}

#[test]
fn unsupported_expression_remains_residual_at_initializer_boundary() {
    let world = build_single_fixture_world("initializer_missing_candidate_residual", "app");
    let symbol = world
        .resolve_with_expectation("x", lang_build::ResolveExpectation::Object)
        .expect("runtime residual symbol");
    assert_symbol_stage(&symbol, Stage::Runtime);
    assert_symbol_not_stage(&symbol, Stage::Meta);
}

#[test]
fn explicit_p1_projects_runtime_slice_from_residual_initializer() {
    let world = build_single_fixture_world("initializer_explicit_policy_failure", "app");
    let symbol = world
        .resolve_with_expectation("x", lang_build::ResolveExpectation::Object)
        .expect("runtime P1 slice");
    assert_symbol_stage(&symbol, Stage::Runtime);
    assert_symbol_not_stage(&symbol, Stage::Meta);
}

#[test]
fn residual_type_name_annotation_remains_deferred() {
    let err = build_fixture_error("initializer_residual_type_name", "app");
    assert!(has_code(
        &err,
        ResolverCode::UnsupportedDeferredTypeAssertion
    ));
}

#[test]
fn runtime_closure_declaration_requires_tau_formation() {
    let error = build_fixture_error("initializer_runtime_body_local_meta", "app");
    assert!(error
        .diagnostics
        .iter()
        .any(|d| d.message.contains("closure-to-tau formation consumer")));
}

#[test]
fn unavailable_source_contribution_is_a_hard_formation_failure() {
    let err = build_fixture_error("initializer_ambiguous", "app");
    assert!(err
        .diagnostics
        .iter()
        .any(|d| d.message.contains("closure-to-tau formation consumer")));
}

#[test]
fn initializer_routing_does_not_depend_on_diagnostic_message_text() {
    let src = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/initializer_eval.rs"),
    )
    .expect("read initializer evaluator source");
    assert!(!src.contains("diagnostic.message.contains"));
    assert!(!src.contains(".message.contains(\"ambiguous overload candidate\")"));
    assert!(!src.contains(".message.contains(\"no matching overload candidate\")"));
    assert!(!src.contains(".message.contains(\"not visible to MetaAction\")"));
    assert!(
        !src.contains(".message.contains(\"body-entry policy does not admit demanded execution\")")
    );
}
