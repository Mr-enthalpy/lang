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
