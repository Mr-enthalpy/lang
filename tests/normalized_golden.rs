use std::fs;
use std::path::PathBuf;

use lang_syntax::norm::{
    NormBindingSlot, NormClosure, NormClosureBody, NormClosureKind, NormDecl, NormExpr, NormForm,
    NormNavComponent, NormOperatorFixity, NormOrigin, NormPattern, NormPatternElem, NormProduct,
    NormProductElem, NormProgram, NormRule,
};

fn case_path(name: &str, extension: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("cases")
        .join("norm")
        .join(format!("{name}.{extension}"))
}

fn assert_norm_case(name: &str, expect_diagnostics: bool) {
    let source = fs::read_to_string(case_path(name, "lang")).expect("read source fixture");
    let expected_norm = lang_syntax::normalize_source_text(
        &fs::read_to_string(case_path(name, "norm")).expect("read normalized fixture"),
    );
    let output = lang_syntax::parse(&source);
    let normalized = lang_syntax::normalize_program(&output.program);

    assert_eq!(lang_syntax::dump_norm_program(&normalized), expected_norm);

    if expect_diagnostics {
        assert!(
            !output.diagnostics.is_empty(),
            "expected diagnostics for {name}"
        );
    } else {
        assert!(
            output.diagnostics.is_empty(),
            "expected no diagnostics, got:\n{}",
            lang_syntax::dump_diagnostics(&output.diagnostics)
        );
    }
}

fn norm_dump_from_source(source: &str) -> String {
    let output = lang_syntax::parse(source);
    let normalized = lang_syntax::normalize_program(&output.program);
    lang_syntax::dump_norm_program(&normalized)
}

fn norm_program_from_source(source: &str) -> NormProgram {
    let output = lang_syntax::parse(source);
    lang_syntax::normalize_program(&output.program)
}

fn single_expr_from_source(source: &str) -> NormExpr {
    let normalized = norm_program_from_source(source);
    match normalized.forms.as_slice() {
        [NormForm::Expr(expr)] => expr.clone(),
        other => panic!("expected single expression form, got {other:#?}"),
    }
}

fn single_let_slot(source: &str) -> NormBindingSlot {
    let normalized = norm_program_from_source(source);
    match normalized.forms.as_slice() {
        [NormForm::Let(NormDecl::Let { slot, .. })] => slot.clone(),
        other => panic!("expected single let declaration, got {other:#?}"),
    }
}

fn expect_call(expr: &NormExpr) -> (&NormProduct, &NormExpr, &NormOrigin) {
    match expr {
        NormExpr::Call {
            source,
            target,
            origin,
        } => (source, target.as_ref(), origin),
        other => panic!("expected call, got {other:#?}"),
    }
}

fn expect_generated(origin: &NormOrigin, rule: NormRule) {
    match origin {
        NormOrigin::Generated { rule: actual, .. } if *actual == rule => {}
        other => panic!("expected generated {rule:?}, got {other:#?}"),
    }
}

fn expect_derived(origin: &NormOrigin, rule: NormRule, summary: &str) {
    match origin {
        NormOrigin::Derived {
            rule: actual,
            summary: actual_summary,
            ..
        } if *actual == rule && actual_summary.contains(summary) => {}
        other => panic!("expected derived {rule:?} containing {summary:?}, got {other:#?}"),
    }
}

fn expect_expr_name(expr: &NormExpr, expected: &str) {
    match expr {
        NormExpr::Name { text, .. } if text == expected => {}
        other => panic!("expected name {expected:?}, got {other:#?}"),
    }
}

fn expect_product_elem_expr(product: &NormProduct, index: usize) -> &NormExpr {
    match product.elements.get(index) {
        Some(NormProductElem::Expr(expr)) => expr,
        other => panic!("expected expr product element {index}, got {other:#?}"),
    }
}

fn expect_product_elem_name(product: &NormProduct, index: usize, expected: &str) {
    expect_expr_name(expect_product_elem_expr(product, index), expected);
}

fn expect_product_elem_call(product: &NormProduct, index: usize) -> &NormExpr {
    let expr = expect_product_elem_expr(product, index);
    match expr {
        NormExpr::Call { .. } => expr,
        other => panic!("expected call product element {index}, got {other:#?}"),
    }
}

fn expect_nav_names(expr: &NormExpr, expected: &[&str]) {
    let NormExpr::Nav { components, .. } = expr else {
        panic!("expected nav target, got {expr:#?}");
    };
    let actual = components
        .iter()
        .map(|component| match component {
            NormNavComponent::Name { name, .. } => name.as_str(),
            other => panic!("expected nav name component, got {other:#?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

fn expect_generated_closure(expr: &NormExpr, rule: NormRule) -> &NormClosure {
    let (_, target, origin) = expect_call(expr);
    expect_generated(origin, rule);
    let NormExpr::Closure(closure) = target else {
        panic!("expected generated closure target, got {target:#?}");
    };
    match closure.kind {
        NormClosureKind::Generated { rule: actual } if actual == rule => closure,
        _ => panic!("expected generated closure {rule:?}, got {closure:#?}"),
    }
}

fn expect_generated_receiver_head(closure: &NormClosure, rule: NormRule) {
    let head = closure.head.as_ref().expect("generated closure head");
    assert_eq!(head.deduce.len(), 1);
    assert_eq!(head.deduce[0].name, "T");
    expect_generated(&head.deduce[0].origin, rule);
    assert!(matches!(
        head.deduce[0].annotation.as_ref().map(|annotation| &annotation.pattern),
        Some(NormPattern::Name { name, .. }) if name == "type"
    ));

    assert_eq!(
        head.params.len(),
        1,
        "expected one generated receiver param, got {:#?}",
        head.params
    );
    if let NormPatternElem::BindingSlot(slot) = &head.params[0] {
        assert!(matches!(
            &slot.value_pattern,
            NormPattern::Binder { name, .. } if name == "val"
        ));
        assert!(matches!(
            slot.annotation.as_ref().map(|annotation| &annotation.pattern),
            Some(NormPattern::HoleRef { name, .. }) if name == "T"
        ));
    } else {
        panic!(
            "expected generated receiver binding slot, got {:#?}",
            head.params[0]
        );
    }
}

fn expect_generated_body_call(closure: &NormClosure) -> (&NormProduct, &NormExpr, &NormOrigin) {
    let prog = match &closure.body {
        NormClosureBody::Block(prog) => prog,
        NormClosureBody::Delete(_) => panic!("expected block body, got delete"),
    };
    match prog.forms.as_slice() {
        [NormForm::Expr(expr)] => expect_call(expr),
        other => panic!("expected generated closure body expression, got {other:#?}"),
    }
}

#[test]
fn basic_pipe() {
    assert_norm_case("01_basic_pipe", false);
}

#[test]
fn source_product_continuation() {
    assert_norm_case("02_source_product_continuation", false);
}

#[test]
fn continuation_then_residual() {
    assert_norm_case("03_continuation_then_residual", false);
}

#[test]
fn target_growth() {
    assert_norm_case("04_target_growth", false);
}

#[test]
fn second_repair() {
    assert_norm_case("05_second_repair", false);
}

#[test]
fn repair_does_not_override_continuation() {
    assert_norm_case("06_repair_does_not_override_continuation", false);
}

#[test]
fn products_and_unit() {
    assert_norm_case("07_products_and_unit", false);
}

#[test]
fn operator_lowering() {
    assert_norm_case("08_operator_lowering", false);
}

#[test]
fn prefix_negative() {
    assert_norm_case("09_prefix_negative", false);
}

#[test]
fn bracket_call() {
    assert_norm_case("10_bracket_call", false);
}

#[test]
fn member_and_double_dot() {
    assert_norm_case("11_member_and_double_dot", false);
}

#[test]
fn alias_preservation() {
    assert_norm_case("12_alias_preservation", false);
}

#[test]
fn annotation_pattern() {
    assert_norm_case("13_annotation_pattern", false);
}

#[test]
fn closure_head() {
    assert_norm_case("14_closure_head", false);
}

#[test]
fn error_recovery_min() {
    assert_norm_case("15_error_recovery_min", true);
}

#[test]
fn call_skeleton_hardening() {
    assert_norm_case("16_call_skeleton_hardening", false);
}

#[test]
fn second_repair_hardening() {
    assert_norm_case("17_second_repair_hardening", false);
}

#[test]
fn group_product_lift_hardening() {
    assert_norm_case("18_group_product_lift_hardening", false);
}

#[test]
fn suffix_lowering_hardening() {
    assert_norm_case("19_suffix_lowering_hardening", false);
}

#[test]
fn operator_hardening() {
    assert_norm_case("20_operator_hardening", true);
}

#[test]
fn alias_hardening() {
    assert_norm_case("21_alias_hardening", false);
}

#[test]
fn pattern_annotation_hardening() {
    assert_norm_case("22_pattern_annotation_hardening", false);
}

#[test]
fn closure_hardening() {
    assert_norm_case("23_closure_hardening", false);
}

#[test]
fn extraction_skeleton_with_hardening() {
    assert_norm_case("24_extraction_skeleton_with_hardening", false);
}

#[test]
fn error_recovery_hardening() {
    assert_norm_case("25_error_recovery_hardening", true);
}

#[test]
fn unsupported_audit() {
    assert_norm_case("26_unsupported_audit", false);
}

#[test]
fn closure_delete_body_normalizes_as_delete_not_call() {
    assert_norm_case("closure_delete_body", false);
}

#[test]
fn annotation_patterns_are_structural_pattern_material() {
    let slot = single_let_slot("let <T> x: T = y");
    let annotation = slot.annotation.as_ref().expect("annotation");

    assert!(matches!(
        &slot.value_pattern,
        NormPattern::Binder { name, .. } if name == "x"
    ));
    assert!(matches!(
        &annotation.pattern,
        NormPattern::HoleRef { name, .. } if name == "T"
    ));

    let slot = single_let_slot("let <T> x: U = y");
    let annotation = slot.annotation.as_ref().expect("annotation");
    assert!(matches!(
        &annotation.pattern,
        NormPattern::Name { name, .. } if name == "U"
    ));
}

#[test]
fn pattern_and_value_names_have_distinct_dump_labels() {
    let dump = norm_dump_from_source("P; let <T> x: P = y");

    assert!(dump.contains("Name \"P\" origin=Source"));
    assert!(dump.contains("PatternName \"P\" origin=Source"));
    assert!(!dump.contains("AnnotationPattern\n              Name \"P\""));
}

#[test]
fn alias_target_remains_entity_ref_not_expression() {
    let output = lang_syntax::parse("let A === B::C");
    let normalized = lang_syntax::normalize_program(&output.program);

    match normalized.forms.as_slice() {
        [lang_syntax::NormForm::Alias(lang_syntax::NormDecl::Alias { target, .. })] => {
            assert_eq!(target.components.len(), 2);
        }
        other => panic!("expected alias declaration, got {other:#?}"),
    }
}

#[test]
fn raw_group_does_not_persist_as_normalized_expression_node() {
    let expr = single_expr_from_source("((x))");
    expect_expr_name(&expr, "x");
}

#[test]
fn source_product_continuation_has_structural_shape() {
    let expr = single_expr_from_source("x |> f (a) g");
    let (outer_source, outer_target, outer_origin) = expect_call(&expr);
    expect_derived(
        outer_origin,
        NormRule::PipeFallback,
        "ordinary expression-chain growth",
    );
    expect_expr_name(outer_target, "g");
    assert_eq!(outer_source.elements.len(), 1);

    let inner = expect_product_elem_call(outer_source, 0);
    let (inner_source, inner_target, inner_origin) = expect_call(inner);
    expect_derived(
        inner_origin,
        NormRule::ProductMerge,
        "source-product continuation",
    );
    expect_expr_name(inner_target, "f");
    assert_eq!(inner_source.elements.len(), 2);
    expect_product_elem_name(inner_source, 0, "x");
    expect_product_elem_name(inner_source, 1, "a");
}

#[test]
fn source_product_continuation_consumes_only_first_following_product() {
    let expr = single_expr_from_source("x |> f (a) (b)");
    let (outer_source, outer_target, outer_origin) = expect_call(&expr);
    expect_derived(
        outer_origin,
        NormRule::PipeFallback,
        "ordinary expression-chain growth",
    );
    expect_expr_name(outer_target, "b");

    let inner = expect_product_elem_call(outer_source, 0);
    let (inner_source, inner_target, inner_origin) = expect_call(inner);
    expect_derived(
        inner_origin,
        NormRule::ProductMerge,
        "source-product continuation",
    );
    expect_expr_name(inner_target, "f");
    assert_eq!(inner_source.elements.len(), 2);
    expect_product_elem_name(inner_source, 0, "x");
    expect_product_elem_name(inner_source, 1, "a");
}

#[test]
fn second_repair_has_structural_shape_without_incoming_source() {
    let expr = single_expr_from_source("f (a) g");
    let (outer_source, outer_target, outer_origin) = expect_call(&expr);
    expect_derived(
        outer_origin,
        NormRule::SecondLegalityRepair,
        "repaired product target in expression chain",
    );
    expect_product_elem_name(outer_source, 0, "f");

    let (repair_source, repair_target, repair_origin) = expect_call(outer_target);
    expect_derived(
        repair_origin,
        NormRule::SecondLegalityRepair,
        "repaired product-before-target",
    );
    expect_product_elem_name(repair_source, 0, "a");
    expect_expr_name(repair_target, "g");
}

#[test]
fn prefix_negative_generated_closure_has_expected_shape() {
    let expr = single_expr_from_source("-x");
    let (source, _, origin) = expect_call(&expr);
    expect_generated(origin, NormRule::PrefixNegativeLowering);
    expect_product_elem_name(source, 0, "x");

    let closure = expect_generated_closure(&expr, NormRule::PrefixNegativeLowering);
    expect_generated_receiver_head(closure, NormRule::PrefixNegativeLowering);
    let (body_source, body_target, body_origin) = expect_generated_body_call(closure);
    expect_generated(body_origin, NormRule::PrefixNegativeLowering);
    expect_nav_names(expect_product_elem_expr(body_source, 0), &["zero", "T"]);
    expect_product_elem_name(body_source, 1, "val");
    assert!(matches!(
        body_target,
        NormExpr::OperatorTarget {
            spelling,
            fixity: NormOperatorFixity::Binary,
            arity: 2,
            ..
        } if spelling == "-"
    ));
}

#[test]
fn member_sugar_generated_closure_has_unresolved_nav_target() {
    let expr = single_expr_from_source("obj.field");
    let (source, _, origin) = expect_call(&expr);
    expect_generated(origin, NormRule::MemberLowering);
    expect_product_elem_name(source, 0, "obj");

    let closure = expect_generated_closure(&expr, NormRule::MemberLowering);
    expect_generated_receiver_head(closure, NormRule::MemberLowering);
    let (body_source, body_target, body_origin) = expect_generated_body_call(closure);
    expect_generated(body_origin, NormRule::MemberLowering);
    expect_product_elem_name(body_source, 0, "val");
    expect_nav_names(body_target, &["field", "T"]);
}

#[test]
fn double_dot_generated_closure_has_unresolved_nav_target() {
    let expr = single_expr_from_source("obj..method(a)");
    let (source, _, origin) = expect_call(&expr);
    expect_generated(origin, NormRule::DoubleDotLowering);
    expect_product_elem_name(source, 0, "obj");

    let closure = expect_generated_closure(&expr, NormRule::DoubleDotLowering);
    expect_generated_receiver_head(closure, NormRule::DoubleDotLowering);
    let (body_source, body_target, body_origin) = expect_generated_body_call(closure);
    expect_generated(body_origin, NormRule::DoubleDotLowering);
    assert_eq!(body_source.elements.len(), 2);
    expect_product_elem_name(body_source, 0, "val");
    expect_product_elem_name(body_source, 1, "a");
    expect_nav_names(body_target, &["method", "T"]);
}

#[test]
fn obj_empty_bracket_has_no_unit_argument() {
    let output = lang_syntax::parse("obj[]");
    let normalized = lang_syntax::normalize_program(&output.program);

    let [lang_syntax::NormForm::Expr(lang_syntax::NormExpr::Call { source, .. })] =
        normalized.forms.as_slice()
    else {
        panic!(
            "expected bracket-call expression, got {:#?}",
            normalized.forms
        );
    };

    assert_eq!(source.elements.len(), 1);
    assert!(matches!(
        &source.elements[0],
        lang_syntax::NormProductElem::Expr(lang_syntax::NormExpr::Name { text, .. })
            if text == "obj"
    ));
}

#[test]
fn bracket_call_preserves_explicit_unit_and_nested_products() {
    let expr = single_expr_from_source("obj[()]");
    let (source, target, _) = expect_call(&expr);
    assert_eq!(source.elements.len(), 2);
    expect_product_elem_name(source, 0, "obj");
    match expect_product_elem_expr(source, 1) {
        NormExpr::Product(product) => {
            assert!(matches!(
                product.elements.as_slice(),
                [NormProductElem::Unit { .. }]
            ));
        }
        other => panic!("expected explicit unit product argument, got {other:#?}"),
    }
    assert!(matches!(
        target,
        NormExpr::OperatorTarget {
            spelling,
            fixity: NormOperatorFixity::BracketCall,
            arity: 2,
            ..
        } if spelling == "[]"
    ));

    let expr = single_expr_from_source("obj[(a)]");
    let (source, _, _) = expect_call(&expr);
    assert_eq!(source.elements.len(), 2);
    expect_product_elem_name(source, 0, "obj");
    expect_product_elem_name(source, 1, "a");

    let expr = single_expr_from_source("obj[(a, b)]");
    let (source, _, _) = expect_call(&expr);
    assert_eq!(source.elements.len(), 2);
    match expect_product_elem_expr(source, 1) {
        NormExpr::Product(product) => {
            assert_eq!(product.elements.len(), 2);
            expect_product_elem_name(product, 0, "a");
            expect_product_elem_name(product, 1, "b");
        }
        other => panic!("expected nested product argument, got {other:#?}"),
    }

    let expr = single_expr_from_source("obj[a, (b, c)]");
    let (source, _, _) = expect_call(&expr);
    assert_eq!(source.elements.len(), 3);
    expect_product_elem_name(source, 0, "obj");
    expect_product_elem_name(source, 1, "a");
    match expect_product_elem_expr(source, 2) {
        NormExpr::Product(product) => {
            assert_eq!(product.elements.len(), 2);
            expect_product_elem_name(product, 0, "b");
            expect_product_elem_name(product, 1, "c");
        }
        other => panic!("expected nested product argument, got {other:#?}"),
    }
}

#[test]
fn recovered_errors_stay_in_family_local_normalized_nodes() {
    let normalized =
        norm_program_from_source("let = y; x |>; x::+; let A === ::B; let x with A = y");

    let [NormForm::Let(NormDecl::Let { slot, .. }), expr_form, nav_form, alias_form, with_form] =
        normalized.forms.as_slice()
    else {
        panic!("expected five recovered forms, got {:#?}", normalized.forms);
    };
    assert!(matches!(slot.value_pattern, NormPattern::Error(_)));

    let NormForm::Expr(expr) = expr_form else {
        panic!("expected expression error form, got {expr_form:#?}");
    };
    let (_, target, _) = expect_call(expr);
    assert!(matches!(target, NormExpr::Error(_)));

    let NormForm::Expr(NormExpr::Nav { components, .. }) = nav_form else {
        panic!("expected recovered nav expression, got {nav_form:#?}");
    };
    assert!(components
        .iter()
        .any(|component| matches!(component, NormNavComponent::Error(_))));

    let NormForm::Alias(NormDecl::Alias { target, .. }) = alias_form else {
        panic!("expected recovered alias form, got {alias_form:#?}");
    };
    assert!(target
        .components
        .iter()
        .any(|component| matches!(component, NormNavComponent::Error(_))));

    let NormForm::Let(NormDecl::Let { slot, .. }) = with_form else {
        panic!("expected recovered with let form, got {with_form:#?}");
    };
    assert!(slot
        .with_clause
        .as_ref()
        .and_then(|with_clause| with_clause.error.as_ref())
        .is_some());
}
