mod support;

use std::path::Path;

use lang_build::{
    invoke_restricted_meta_overload_with_policy, ExecutionEnv, LookupPhase, PolicyFlag,
    ResolverCode, RestrictedMetaInvocationOutcome, RestrictedOverloadFailureKind, SymbolPayload,
    VisibilityView,
};
use lang_syntax::NormForm;
use support::{build_fixture_error, build_single_fixture_world};

fn type_forward_target_name(world: &lang_build::CompilationWorld, name: &str) -> String {
    let symbol = world
        .resolve_with_expectation(name, lang_build::ResolveExpectation::TypeObject)
        .expect("type symbol");
    let SymbolPayload::Type(type_object) = &symbol.payload else {
        panic!("{name} must resolve to TypeObject");
    };
    world
        .snapshot()
        .symbol(type_object.type_symbol_id)
        .expect("forward target")
        .name
        .clone()
}

fn has_code(error: &lang_build::BuildError, code: ResolverCode) -> bool {
    error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == Some(code))
}

fn call_site(source: &str) -> lang_build::NormalizedCallSite {
    let parsed = lang_syntax::parse(source);
    assert!(
        parsed.diagnostics.is_empty(),
        "unexpected parse diagnostics:\n{}",
        lang_syntax::dump_diagnostics(&parsed.diagnostics)
    );
    let normalized = lang_syntax::normalize_program(&parsed.program);
    let expr = match normalized.forms.as_slice() {
        [NormForm::Expr(expr)] => expr.clone(),
        other => panic!("expected one expression form, got {other:#?}"),
    };
    lang_build::extract_single_call_site(&expr).expect("test expression must be a call")
}

#[test]
fn initializer_evaluates_int_plus_unit_by_default_meta_partial_policy() {
    let world = build_single_fixture_world("v08_initializer_meta_partial", "app");

    assert_eq!(type_forward_target_name(&world, "X"), "int");
}

#[test]
fn initializer_evaluates_unit_plus_int_by_default_meta_partial_policy() {
    let world = build_single_fixture_world("v08_initializer_meta_partial", "app");

    assert_eq!(type_forward_target_name(&world, "Y"), "int");
}

#[test]
fn ordinary_initializer_uses_restricted_overload_selector_from_graph() {
    let world = build_single_fixture_world("v08_initializer_meta_partial", "app");
    let root = world
        .snapshot()
        .node(world.package_root_node())
        .expect("package root");
    let plus_bucket = root.children.get("+").expect("plus overload bucket");
    assert_eq!(plus_bucket.object_symbols().len(), 2);
    for symbol_id in plus_bucket.object_symbols() {
        let symbol = world.snapshot().symbol(*symbol_id).expect("plus symbol");
        let SymbolPayload::MetaFunction(meta_function) = &symbol.payload else {
            panic!("plus overload must be meta-function");
        };
        assert!(
            meta_function.source_callable.is_some(),
            "plus overload must be harvested from source callable declaration"
        );
    }

    let x = world
        .resolve_with_expectation("X", lang_build::ResolveExpectation::TypeObject)
        .expect("X type");
    assert_eq!(
        x.generation_origin.as_deref(),
        Some("ForwardedValue(TypeSymbol) binding")
    );
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
    let world = build_single_fixture_world("v08_initializer_meta_partial", "app");
    let symbol = world
        .resolve_with_expectation("runtime_residual", lang_build::ResolveExpectation::Object)
        .expect("runtime residual symbol");
    assert!(symbol
        .policy_metadata
        .policy_set
        .contains(PolicyFlag::Runtime));
    assert!(!symbol.policy_metadata.policy_set.contains(PolicyFlag::Meta));
}

#[test]
fn missing_meta_visible_candidate_residualizes_under_meta_partial() {
    let world = build_single_fixture_world("v08_initializer_missing_candidate_residual", "app");
    let symbol = world
        .resolve_with_expectation("x", lang_build::ResolveExpectation::Object)
        .expect("runtime residual symbol");
    assert!(symbol
        .policy_metadata
        .policy_set
        .contains(PolicyFlag::Runtime));
    assert!(!symbol.policy_metadata.policy_set.contains(PolicyFlag::Meta));
}

#[test]
fn explicit_policy_is_verification_for_residual_initializer() {
    let err = build_fixture_error("v08_initializer_explicit_policy_fail", "app");
    assert!(has_code(
        &err,
        ResolverCode::ExplicitPolicyVerificationFailed
    ));
}

#[test]
fn explicit_policy_verification_uses_selected_callable_return_policy() {
    let err = build_fixture_error("v08_initializer_return_policy_verification", "app");
    assert!(has_code(
        &err,
        ResolverCode::ExplicitPolicyVerificationFailed
    ));
}

#[test]
fn omitted_policy_infers_selected_callable_return_policy() {
    let world =
        build_single_fixture_world("v08_initializer_omitted_return_policy_inference", "app");
    let symbol = world
        .resolve_with_expectation("X", lang_build::ResolveExpectation::TypeObject)
        .expect("X type");
    assert!(symbol.policy_metadata.policy_set.contains(PolicyFlag::Meta));
    assert!(!symbol
        .policy_metadata
        .policy_set
        .contains(PolicyFlag::Runtime));
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
        .policy_set
        .contains(PolicyFlag::Runtime));
    assert!(!meta_function
        .body_entry_policy
        .policy_set
        .contains(PolicyFlag::Meta));
}

#[test]
fn meta_body_uses_meta_strict_for_local_initializers() {
    let err = build_fixture_error("v08_initializer_meta_strict_fail", "app");
    assert!(has_code(&err, ResolverCode::ResidualNotAllowedInMetaStrict));
}

#[test]
fn selected_meta_body_local_let_parameter_environment_is_unsupported_boundary() {
    let err = build_fixture_error("v08_initializer_meta_body_local_param_unsupported", "app");
    assert!(err
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code
            == Some(ResolverCode::UnsupportedSelectedMetaBodyLocalBinding)));
}

#[test]
fn generic_sum_pattern_value_is_explicitly_unsupported() {
    let err = build_fixture_error("v08_initializer_generic_sum_unsupported", "app");
    assert!(has_code(
        &err,
        ResolverCode::UnsupportedCanonicalSumPatternValue
    ));
}

#[test]
fn ambiguity_does_not_residualize_under_meta_partial() {
    let err = build_fixture_error("v08_initializer_ambiguous", "app");
    assert!(has_code(&err, ResolverCode::AmbiguousMetaCandidate));
}

#[test]
fn body_entry_mismatch_residualizes_from_structured_failure_kind() {
    let world = build_single_fixture_world("v08_initializer_body_entry_mismatch_residual", "app");
    let symbol = world
        .resolve_with_expectation("x", lang_build::ResolveExpectation::Object)
        .expect("runtime residual symbol");
    assert!(symbol
        .policy_metadata
        .policy_set
        .contains(PolicyFlag::Runtime));
    assert!(!symbol.policy_metadata.policy_set.contains(PolicyFlag::Meta));

    let site = call_site("int f");
    let outcome = invoke_restricted_meta_overload_with_policy(
        world.snapshot(),
        world.package_root_node(),
        &site,
        &world.package_context(),
        LookupPhase::MetaAction,
        ExecutionEnv::Meta,
        VisibilityView::Internal,
        lang_build::Provenance::new("body entry mismatch test"),
    );
    let RestrictedMetaInvocationOutcome::Diagnostic {
        diagnostic,
        failure_kind,
    } = outcome
    else {
        panic!("body-entry mismatch should not invoke under Meta");
    };
    assert_eq!(
        failure_kind,
        RestrictedOverloadFailureKind::BodyEntryPolicyMismatch {
            demanded_execution: ExecutionEnv::Meta
        }
    );
    assert_eq!(diagnostic.code, Some(ResolverCode::BodyEntryPolicyMismatch));
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
