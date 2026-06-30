mod support;

use lang_build::{
    construct_c0, invoke_restricted_meta_overload, invoke_restricted_meta_overload_with_policy,
    select_restricted_meta_overload, CompilationWorld, Diagnostic, ExecutionEnv, LookupPhase,
    MetaInvocationResult, MetaInvocationValue, MetaValueTarget, OverloadSelectionInput, PolicyFlag,
    ProductMaterialRole, Provenance, ResolveExpectation, ResolverCode,
    RestrictedMetaInvocationOutcome, RestrictedOverloadFailureKind, VisibilityView,
};
use lang_syntax::{NormExpr, NormForm};
use support::{build_fixture_error, build_single_fixture_world, fixture_source_root};

fn world() -> CompilationWorld {
    build_single_fixture_world("v08_meta_overload", "app")
}

fn ambiguous_world() -> CompilationWorld {
    build_single_fixture_world("v08_meta_overload_ambiguous", "app")
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

fn plus_selection(source: &str) -> Result<lang_build::SelectedOverloadCandidate, Diagnostic> {
    let world = world();
    let site = call_site(source);
    let shape = site.to_arg_product_shape(ProductMaterialRole::CallableArgumentProduct);
    let classified = lang_build::classify_type_arguments_with_report(
        &shape,
        &world.snapshot().capability(),
        &world.package_context(),
    );
    select_restricted_meta_overload(OverloadSelectionInput {
        snapshot: world.snapshot(),
        namespace: world.package_root_node(),
        callable_name: "+".to_string(),
        arg_product_shape: classified.classified_shape,
        lookup_phase: LookupPhase::MetaAction,
        demanded_execution: ExecutionEnv::Meta,
        visibility: VisibilityView::Internal,
        provenance: Provenance::new(source),
    })
}

fn invoke(source: &str) -> MetaInvocationResult {
    let world = world();
    let site = call_site(source);
    invoke_restricted_meta_overload(
        world.snapshot(),
        world.package_root_node(),
        &site,
        &world.package_context(),
        LookupPhase::MetaAction,
        ExecutionEnv::Meta,
        VisibilityView::Internal,
        Provenance::new(source),
    )
}

fn invoke_named(source: &str, name: &str) -> MetaInvocationResult {
    let world = world();
    let site = call_site(source);
    assert!(matches!(&site.target, NormExpr::Name { text, .. } if text == name));
    invoke_restricted_meta_overload(
        world.snapshot(),
        world.package_root_node(),
        &site,
        &world.package_context(),
        LookupPhase::MetaAction,
        ExecutionEnv::Meta,
        VisibilityView::Internal,
        Provenance::new(source),
    )
}

fn forwarded_type_name(result: MetaInvocationResult) -> String {
    let MetaInvocationResult::Value(MetaInvocationValue::ForwardedValue(value)) = result else {
        panic!("expected forwarded value, got {result:#?}");
    };
    let MetaValueTarget::TypeSymbol(symbol_id) = value.target;
    world()
        .snapshot()
        .symbol(symbol_id)
        .expect("forwarded symbol")
        .name
        .clone()
}

#[test]
fn source_declares_multiple_same_name_plus_overloads() {
    let world = world();
    let site = call_site("unit + unit");
    let shape = site.to_arg_product_shape(ProductMaterialRole::CallableArgumentProduct);
    let classified = lang_build::classify_type_arguments(
        &shape,
        &world.snapshot().capability(),
        &world.package_context(),
    );
    let input = OverloadSelectionInput {
        snapshot: world.snapshot(),
        namespace: world.package_root_node(),
        callable_name: "+".to_string(),
        arg_product_shape: classified,
        lookup_phase: LookupPhase::MetaAction,
        demanded_execution: ExecutionEnv::Meta,
        visibility: VisibilityView::Internal,
        provenance: Provenance::new("c0"),
    };
    let c0 = construct_c0(&input);
    assert_eq!(c0.c0_symbol_ids.len(), 8);
}

#[test]
fn same_name_plus_declarations_do_not_overwrite_each_other() {
    let selected = plus_selection("unit + int").expect("select overload");
    assert_eq!(selected.symbol.name, "+");
    assert_eq!(
        selected.bindings["t"].top_pattern_name.as_deref(),
        Some("int")
    );
    assert_eq!(
        plus_selection("int + unit").unwrap().bindings["t"]
            .top_pattern_name
            .as_deref(),
        Some("int")
    );
}

#[test]
fn plus_overload_set_is_built_from_namespace_graph_children() {
    let world = world();
    let site = call_site("unit + unit");
    let classified = lang_build::classify_type_arguments(
        &site.to_arg_product_shape(ProductMaterialRole::CallableArgumentProduct),
        &world.snapshot().capability(),
        &world.package_context(),
    );
    let c0 = construct_c0(&OverloadSelectionInput {
        snapshot: world.snapshot(),
        namespace: world.package_root_node(),
        callable_name: "+".to_string(),
        arg_product_shape: classified,
        lookup_phase: LookupPhase::MetaAction,
        demanded_execution: ExecutionEnv::Meta,
        visibility: VisibilityView::Internal,
        provenance: Provenance::new("namespace child c0"),
    });
    let root_node = world.snapshot().node(world.package_root_node()).unwrap();
    let bucket = root_node.children.get("+").expect("plus bucket");
    assert_eq!(bucket.object_symbols().len(), c0.c0_symbol_ids.len());
}

#[test]
fn plus_overload_set_is_not_hand_constructed_in_test() {
    let source_path = fixture_source_root("v08_meta_overload", "app").join("main.lang");
    let source = std::fs::read_to_string(source_path).expect("read fixture");
    assert!(source.contains("meta | runtime let +"));
    assert!(world()
        .snapshot()
        .node(world().package_root_node())
        .unwrap()
        .children
        .contains_key("+"));
}

#[test]
fn meta_or_runtime_policy_prefix_sets_symbol_policy_meta_runtime() {
    let world = world();
    let node = world.snapshot().node(world.package_root_node()).unwrap();
    for id in node.children.get("+").unwrap().object_symbols() {
        let symbol = world.snapshot().symbol(*id).unwrap();
        assert!(symbol.policy_metadata.policy_set.contains(PolicyFlag::Meta));
        assert!(symbol
            .policy_metadata
            .policy_set
            .contains(PolicyFlag::Runtime));
    }
}

#[test]
fn meta_or_runtime_policy_does_not_set_body_entry_runtime() {
    let selected = plus_selection("unit + unit").expect("select overload");
    let lang_build::SymbolPayload::MetaFunction(meta_function) = selected.symbol.payload else {
        panic!("expected meta payload");
    };
    assert!(meta_function
        .body_entry_policy
        .policy_set
        .contains(PolicyFlag::Meta));
    assert!(!meta_function
        .body_entry_policy
        .policy_set
        .contains(PolicyFlag::Runtime));
}

#[test]
fn return_policy_defaults_to_symbol_policy() {
    let selected = plus_selection("unit + int").expect("select overload");
    let lang_build::SymbolPayload::MetaFunction(meta_function) = &selected.symbol.payload else {
        panic!("expected meta payload");
    };
    assert_eq!(
        selected.symbol.policy_metadata.policy_set,
        meta_function.return_object_policy.policy_set
    );
}

#[test]
fn unsupported_explicit_return_policy_annotation_is_diagnostic() {
    let err = build_fixture_error("v08_meta_overload_bad_return_policy", "app");
    assert!(err.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("unsupported explicit return policy annotation")));
}

#[test]
fn non_call_lookup_of_overload_set_reports_ambiguity_or_requires_overload_context() {
    let world = world();
    let err = world
        .snapshot()
        .capability()
        .resolve_str_with_expectation(
            "+",
            &world.package_context(),
            ResolveExpectation::MetaFunction,
        )
        .expect_err("non-call lookup must not choose one overload");
    assert_eq!(err.code, Some(ResolverCode::Ambiguous));
    assert!(err.message.contains("overload context"));
}

#[test]
fn candidate_set_does_not_global_search_all_plus_symbols() {
    let world = world();
    let root_site = call_site("unit + unit");
    let root_shape = lang_build::classify_type_arguments(
        &root_site.to_arg_product_shape(ProductMaterialRole::CallableArgumentProduct),
        &world.snapshot().capability(),
        &world.package_context(),
    );
    let root_c0 = construct_c0(&OverloadSelectionInput {
        snapshot: world.snapshot(),
        namespace: world.package_root_node(),
        callable_name: "+".to_string(),
        arg_product_shape: root_shape,
        lookup_phase: LookupPhase::MetaAction,
        demanded_execution: ExecutionEnv::Meta,
        visibility: VisibilityView::Internal,
        provenance: Provenance::new("root c0"),
    });
    assert_eq!(root_c0.c0_symbol_ids.len(), 8);

    let other = world
        .snapshot()
        .capability()
        .resolve_namespace_subspace("other", &world.package_context())
        .expect("other namespace")
        .namespace_node()
        .unwrap();
    let other_site = call_site("unit + unit");
    let other_shape = lang_build::classify_type_arguments(
        &other_site.to_arg_product_shape(ProductMaterialRole::CallableArgumentProduct),
        &world.snapshot().capability(),
        &world.package_context(),
    );
    let other_c0 = construct_c0(&OverloadSelectionInput {
        snapshot: world.snapshot(),
        namespace: other,
        callable_name: "+".to_string(),
        arg_product_shape: other_shape,
        lookup_phase: LookupPhase::MetaAction,
        demanded_execution: ExecutionEnv::Meta,
        visibility: VisibilityView::Internal,
        provenance: Provenance::new("other c0"),
    });
    assert_eq!(other_c0.c0_symbol_ids.len(), 1);
}

#[test]
fn generic_type_binder_matches_type_argument() {
    let selected = plus_selection("int + int").expect("generic binder should match");
    assert_eq!(
        selected.bindings["t"].top_pattern_name.as_deref(),
        Some("int")
    );
    assert_eq!(
        selected.bindings["u"].top_pattern_name.as_deref(),
        Some("int")
    );
}

#[test]
fn unit_named_pattern_matches_unit_and_does_not_match_int() {
    assert_eq!(forwarded_type_name(invoke("unit + int")), "int");
    let selected = plus_selection("int + int").expect("generic selected");
    assert_eq!(selected.specificity.sum_depth, 2);
}

#[test]
fn if_else_named_and_or_patterns_match_only_their_top_pattern() {
    let if_result = invoke_named("(if, int) either", "either");
    assert_eq!(forwarded_type_name(if_result), "int");
    let else_result = invoke_named("(else, int) either", "either");
    assert_eq!(forwarded_type_name(else_result), "int");
    let unit_result = invoke_named("(unit, int) either", "either");
    let MetaInvocationResult::Diagnostic(diag) = unit_result else {
        panic!("unit must not match if|else");
    };
    assert!(diag.message.contains("expected one of"));
}

#[test]
fn unit_specific_overload_outranks_generic_binder_for_right_and_left_unit() {
    let right = plus_selection("int + unit").expect("select right unit");
    assert_eq!(right.bindings["t"].top_pattern_name.as_deref(), Some("int"));
    assert!(right.specificity.sum_depth > 2);

    let left = plus_selection("unit + int").expect("select left unit");
    assert_eq!(left.bindings["t"].top_pattern_name.as_deref(), Some("int"));
    assert!(left.specificity.sum_depth > 2);
}

#[test]
fn if_else_delete_overloads_outrank_generic_binder() {
    let MetaInvocationResult::Diagnostic(if_diag) = invoke("if + int") else {
        panic!("if + int must select delete overload");
    };
    assert!(if_diag.message.contains("cannot combine bare if"));

    let MetaInvocationResult::Diagnostic(else_diag) = invoke("else + int") else {
        panic!("else + int must select delete overload");
    };
    assert!(else_diag.message.contains("cannot combine bare else"));
}

#[test]
fn specificity_uses_lexicographic_tuple_not_declaration_order() {
    let selected = plus_selection("unit + unit").expect("unit exact selected");
    assert_eq!(selected.specificity.max_depth, 1);
    assert_eq!(selected.specificity.sum_depth, 4);
    assert_eq!(forwarded_type_name(invoke("unit + unit")), "unit");
}

#[test]
fn ambiguous_equal_specificity_candidates_are_diagnostic() {
    let world = ambiguous_world();
    let site = call_site("int + int");
    let shape = lang_build::classify_type_arguments(
        &site.to_arg_product_shape(ProductMaterialRole::CallableArgumentProduct),
        &world.snapshot().capability(),
        &world.package_context(),
    );
    let err = select_restricted_meta_overload(OverloadSelectionInput {
        snapshot: world.snapshot(),
        namespace: world.package_root_node(),
        callable_name: "+".to_string(),
        arg_product_shape: shape,
        lookup_phase: LookupPhase::MetaAction,
        demanded_execution: ExecutionEnv::Meta,
        visibility: VisibilityView::Internal,
        provenance: Provenance::new("ambiguous"),
    })
    .expect_err("equal specificity must be ambiguous");
    assert!(err.message.contains("ambiguous overload candidate"));
}

#[test]
fn selected_delete_overload_produces_diagnostic_not_value() {
    let result = invoke("if + int");
    assert!(matches!(result, MetaInvocationResult::Diagnostic(_)));
}

#[test]
fn selected_unit_and_forwarding_overloads_return_expected_types() {
    assert_eq!(forwarded_type_name(invoke("unit + unit")), "unit");
    assert_eq!(forwarded_type_name(invoke("int + unit")), "int");
    assert_eq!(forwarded_type_name(invoke("unit + int")), "int");
}

#[test]
fn unsupported_selected_meta_block_is_diagnostic() {
    let result = invoke_named("(int, unit) bad", "bad");
    let MetaInvocationResult::Diagnostic(diag) = result else {
        panic!("unsupported body must be diagnostic");
    };
    assert!(diag.message.contains("restricted v0.8"));
}

#[test]
fn runtime_execution_must_not_enter_meta_only_body() {
    let world = world();
    let site = call_site("unit + unit");
    let shape = lang_build::classify_type_arguments(
        &site.to_arg_product_shape(ProductMaterialRole::CallableArgumentProduct),
        &world.snapshot().capability(),
        &world.package_context(),
    );
    let err = select_restricted_meta_overload(OverloadSelectionInput {
        snapshot: world.snapshot(),
        namespace: world.package_root_node(),
        callable_name: "+".to_string(),
        arg_product_shape: shape,
        lookup_phase: LookupPhase::RuntimeBinding,
        demanded_execution: ExecutionEnv::Runtime,
        visibility: VisibilityView::Internal,
        provenance: Provenance::new("runtime execution"),
    })
    .expect_err("runtime execution cannot enter meta body");
    assert!(err.message.contains("body-entry policy"));
}

#[test]
fn runtime_binding_lookup_can_see_meta_or_runtime_symbol_metadata() {
    let world = world();
    let site = call_site("unit + unit");
    let shape = lang_build::classify_type_arguments(
        &site.to_arg_product_shape(ProductMaterialRole::CallableArgumentProduct),
        &world.snapshot().capability(),
        &world.package_context(),
    );
    let selected = select_restricted_meta_overload(OverloadSelectionInput {
        snapshot: world.snapshot(),
        namespace: world.package_root_node(),
        callable_name: "+".to_string(),
        arg_product_shape: shape,
        lookup_phase: LookupPhase::RuntimeBinding,
        demanded_execution: ExecutionEnv::Meta,
        visibility: VisibilityView::Internal,
        provenance: Provenance::new("runtime lookup metadata"),
    })
    .expect("runtime lookup phase sees runtime-visible symbol metadata");
    assert!(selected
        .symbol
        .policy_metadata
        .policy_set
        .contains(PolicyFlag::Runtime));
}

#[test]
fn symbol_visibility_policy_is_not_body_entry_permission() {
    let world = world();
    let symbol = world
        .snapshot()
        .capability()
        .resolve_str_with_expectation(
            "runtime_body",
            &world.package_context(),
            ResolveExpectation::MetaFunction,
        )
        .expect("runtime body symbol");
    assert!(symbol.policy_metadata.policy_set.contains(PolicyFlag::Meta));
    let lang_build::SymbolPayload::MetaFunction(meta_function) = symbol.payload else {
        panic!("meta payload");
    };
    assert!(!meta_function
        .body_entry_policy
        .policy_set
        .contains(PolicyFlag::Meta));
    assert!(meta_function
        .body_entry_policy
        .policy_set
        .contains(PolicyFlag::Runtime));
}

#[test]
fn generic_plus_control_flow_body_is_not_evaluated_in_this_pr() {
    let result = invoke_named("(int, unit) bad", "bad");
    let MetaInvocationResult::Diagnostic(diag) = result else {
        panic!("control-flow/generic body must be diagnostic");
    };
    assert!(diag.message.contains("restricted v0.8"));
}

#[test]
fn done_is_not_special_cased_inside_plus_selection() {
    let result = invoke("int + (else Done)");
    let MetaInvocationResult::Diagnostic(diag) = result else {
        panic!("unclassified Done expression should be diagnostic, not a value");
    };
    assert!(!diag.message.contains("cannot combine into bare else"));
    assert!(!diag.message.contains("Done"));
}

#[test]
fn policy_bar_is_not_pattern_sum_bar_and_pattern_bar_is_not_policy_bar() {
    assert!(plus_selection("unit + unit").is_ok());
    assert_eq!(
        forwarded_type_name(invoke_named("(if, int) either", "either")),
        "int"
    );
}

#[test]
fn plus_is_not_treated_as_bar_by_parser_or_normalizer() {
    let site = call_site("int + unit");
    assert!(matches!(
        site.target,
        NormExpr::OperatorTarget { ref spelling, .. } if spelling == "+"
    ));
    assert_eq!(forwarded_type_name(invoke("int + unit")), "int");
}

fn unsupported_param_world() -> CompilationWorld {
    build_single_fixture_world("v08_meta_overload_unsupported_param", "app")
}

fn non_binder_return_world() -> CompilationWorld {
    build_single_fixture_world("v08_meta_overload_no_return_slot", "app")
}

fn invoke_with_policy(world: &CompilationWorld, source: &str) -> RestrictedMetaInvocationOutcome {
    let site = call_site(source);
    invoke_restricted_meta_overload_with_policy(
        world.snapshot(),
        world.package_root_node(),
        &site,
        &world.package_context(),
        LookupPhase::MetaAction,
        ExecutionEnv::Meta,
        VisibilityView::Internal,
        Provenance::new(source),
    )
}

#[test]
fn unsupported_parameter_pattern_produces_hard_error_not_residual() {
    let outcome = invoke_with_policy(&unsupported_param_world(), "int magic");
    let RestrictedMetaInvocationOutcome::Diagnostic {
        diagnostic,
        failure_kind,
    } = outcome
    else {
        panic!("expected diagnostic outcome for unsupported parameter pattern");
    };
    assert_eq!(
        failure_kind,
        RestrictedOverloadFailureKind::UnsupportedParameterPattern
    );
    assert_eq!(
        diagnostic.code,
        Some(ResolverCode::UnsupportedParameterPattern)
    );
    assert!(
        diagnostic
            .message
            .contains("unsupported parameter extraction pattern"),
        "diagnostic message should mention unsupported parameter pattern, got: {}",
        diagnostic.message
    );
}

#[test]
fn unsupported_candidate_shape_non_binder_return_is_hard_error() {
    let outcome = invoke_with_policy(&non_binder_return_world(), "int magic");
    let RestrictedMetaInvocationOutcome::Diagnostic {
        diagnostic,
        failure_kind,
    } = outcome
    else {
        panic!("expected diagnostic outcome for missing return slot");
    };
    assert_eq!(
        failure_kind,
        RestrictedOverloadFailureKind::UnsupportedCandidateShape
    );
    assert_eq!(
        diagnostic.code,
        Some(ResolverCode::UnsupportedCandidateShape)
    );
    assert!(
        diagnostic.message.contains("return slot must be a binder"),
        "diagnostic message should mention return slot must be a binder, got: {}",
        diagnostic.message
    );
}
