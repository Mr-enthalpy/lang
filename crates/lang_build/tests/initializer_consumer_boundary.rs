mod support;

use std::path::Path;

use lang_build::ResolverCode;
use support::build_fixture_error;

fn has_code(error: &lang_build::BuildError, code: ResolverCode) -> bool {
    error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == Some(code))
}

#[test]
fn let_type_annotation_is_post_rhs_assertion_not_compile_trigger() {
    let err = build_fixture_error("initializer_annotation_non_trigger", "app");
    assert!(has_code(
        &err,
        ResolverCode::UnsupportedInitializerExpression
    ));
    assert!(err.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("initializer expression consumer is unavailable")));
}

#[test]
fn omitted_policy_cannot_derive_runtime_from_incomplete_evaluation() {
    let error = build_fixture_error("initializer_default_policy_frontier", "app");
    assert!(has_code(
        &error,
        ResolverCode::UnsupportedInitializerExpression
    ));
}

#[test]
fn unsupported_expression_cannot_install_a_residual_binding() {
    let error = build_fixture_error("initializer_unconnected_expression", "app");
    assert!(has_code(
        &error,
        ResolverCode::UnsupportedInitializerExpression
    ));
}

#[test]
fn explicit_runtime_demand_cannot_fabricate_a_residual_producer() {
    let error = build_fixture_error("initializer_explicit_policy_failure", "app");
    assert!(has_code(
        &error,
        ResolverCode::UnsupportedInitializerExpression
    ));
}

#[test]
fn missing_type_name_annotation_does_not_supply_a_remaining_continuation() {
    let err = build_fixture_error("initializer_unresolved_type_name", "app");
    assert!(has_code(
        &err,
        ResolverCode::UnsupportedInitializerExpression
    ));
}

#[test]
fn runtime_closure_declaration_requires_tau_formation() {
    let error = build_fixture_error("initializer_runtime_body_local_compile", "app");
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
    assert!(!src.contains(".message.contains(\"not visible to StaticAction\")"));
    assert!(
        !src.contains(".message.contains(\"body-entry policy does not admit demanded execution\")")
    );
}
