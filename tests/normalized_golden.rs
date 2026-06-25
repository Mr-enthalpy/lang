use std::fs;
use std::path::PathBuf;

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
