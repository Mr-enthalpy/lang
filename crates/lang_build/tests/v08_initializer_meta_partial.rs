mod support;

use std::path::Path;

use lang_build::{PolicyStage, ResolverCode, SymbolPayload};
use support::{build_fixture_error, build_single_fixture_world};

fn has_code(error: &lang_build::BuildError, code: ResolverCode) -> bool {
    error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == Some(code))
}

fn assert_symbol_stage(symbol: &lang_build::SymbolObject, stage: PolicyStage) {
    assert!(symbol
        .policy_view
        .as_ref()
        .expect("Symbol Policy view")
        .pair
        .value
        .stages
        .contains(stage));
}

fn assert_symbol_not_stage(symbol: &lang_build::SymbolObject, stage: PolicyStage) {
    assert!(!symbol
        .policy_view
        .as_ref()
        .expect("Symbol Policy view")
        .pair
        .value
        .stages
        .contains(stage));
}

#[test]
fn let_type_annotation_is_post_rhs_assertion_not_meta_trigger() {
    let err = build_fixture_error("v08_initializer_annotation_not_trigger", "app");
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
    let world = build_single_fixture_world("v08_initializer_omitted_policy_residual", "app");
    let symbol = world
        .resolve_with_expectation("runtime_residual", lang_build::ResolveExpectation::Object)
        .expect("runtime residual symbol");
    assert_symbol_stage(&symbol, PolicyStage::Runtime);
    assert_symbol_not_stage(&symbol, PolicyStage::Meta);
}

#[test]
fn missing_meta_visible_candidate_residualizes_under_meta_partial() {
    let world = build_single_fixture_world("v08_initializer_missing_candidate_residual", "app");
    let symbol = world
        .resolve_with_expectation("x", lang_build::ResolveExpectation::Object)
        .expect("runtime residual symbol");
    assert_symbol_stage(&symbol, PolicyStage::Runtime);
    assert_symbol_not_stage(&symbol, PolicyStage::Meta);
}

#[test]
fn explicit_p1_projects_runtime_slice_from_residual_initializer() {
    let world = build_single_fixture_world("v08_initializer_explicit_policy_fail", "app");
    let symbol = world
        .resolve_with_expectation("x", lang_build::ResolveExpectation::Object)
        .expect("runtime P1 slice");
    assert_symbol_stage(&symbol, PolicyStage::Runtime);
    assert_symbol_not_stage(&symbol, PolicyStage::Meta);
}

#[test]
fn explicit_p1_projects_selected_callable_result_slice() {
    let world = build_single_fixture_world("v08_initializer_return_policy_verification", "app");
    let symbol = world
        .resolve_with_expectation("X", lang_build::ResolveExpectation::TypeObject)
        .expect("meta result slice");
    assert_symbol_stage(&symbol, PolicyStage::Meta);
    assert_symbol_not_stage(&symbol, PolicyStage::Runtime);
}

#[test]
fn omitted_policy_infers_selected_callable_return_policy() {
    let world =
        build_single_fixture_world("v08_initializer_omitted_return_policy_inference", "app");
    let symbol = world
        .resolve_with_expectation("X", lang_build::ResolveExpectation::TypeObject)
        .expect("X type");
    assert_symbol_stage(&symbol, PolicyStage::Meta);
    assert_symbol_not_stage(&symbol, PolicyStage::Runtime);
}

#[test]
fn residual_type_name_annotation_is_deferred_not_placeholder() {
    let err = build_fixture_error("v08_initializer_residual_type_name", "app");
    assert!(has_code(
        &err,
        ResolverCode::UnsupportedDeferredTypeAssertion
    ));
}

#[test]
fn runtime_body_declaration_may_contain_local_meta_shaped_initializer() {
    let world = build_single_fixture_world("v08_initializer_runtime_body_local_meta", "app");
    let runtime_body = world
        .resolve_with_expectation("runtime_body", lang_build::ResolveExpectation::MetaFunction)
        .expect("runtime_body callable");
    let SymbolPayload::MetaFunction(meta_function) = &runtime_body.payload else {
        panic!("runtime_body must be meta function object");
    };
    assert!(meta_function
        .body_entry_policy
        .pair
        .value
        .stages
        .contains(PolicyStage::Runtime));
    assert!(!meta_function
        .body_entry_policy
        .pair
        .value
        .stages
        .contains(PolicyStage::Meta));
}

#[test]
fn ambiguity_does_not_residualize_under_meta_partial() {
    let err = build_fixture_error("v08_initializer_ambiguous", "app");
    assert!(has_code(&err, ResolverCode::AmbiguousMetaCandidate));
}

// A runtime-only result P2 (`: runtime ->` = `runtime:compile`) claims a
// runtime value slice whose stage is disjoint from its Pattern stage
// (`N2(runtime) = runtime:compile`). A pure-P return slot (`let r: type`)
// carries no value dimension, so the declared runtime slice could never be
// filled: the declaration itself is rejected at elaboration. Static single
// policies keep Pv == Pp (`N2(P) = P:(P - runtime)`) and stay legal for
// pure-P return slots.
#[test]
fn runtime_only_pure_p_return_slot_declaration_is_hard_error() {
    let err = build_fixture_error("v09_runtime_slice_no_value_dimension", "app");
    assert!(has_code(
        &err,
        ResolverCode::RuntimeSliceWithoutValueDimension
    ));
    assert!(err
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("no value dimension")));
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
