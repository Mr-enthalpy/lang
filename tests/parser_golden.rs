use std::fs;
use std::path::PathBuf;

fn case_path(name: &str, extension: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("cases")
        .join("parser")
        .join(format!("{name}.{extension}"))
}

fn assert_parser_case(name: &str, expect_diagnostics: bool) {
    let source = fs::read_to_string(case_path(name, "lang")).expect("read source fixture");
    let expected_ast = lang_syntax::normalize_source_text(
        &fs::read_to_string(case_path(name, "ast")).expect("read AST fixture"),
    );
    let output = lang_syntax::parse(&source);

    assert_eq!(lang_syntax::dump_ast(&output.program), expected_ast);

    if expect_diagnostics {
        let expected_diagnostics = lang_syntax::normalize_source_text(
            &fs::read_to_string(case_path(name, "diag")).expect("read diagnostic fixture"),
        );
        assert_eq!(
            lang_syntax::dump_diagnostics(&output.diagnostics),
            expected_diagnostics
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
fn expr_name() {
    assert_parser_case("expr_name", false);
}

#[test]
fn expr_path() {
    assert_parser_case("expr_path", false);
}

#[test]
fn let_simple_type() {
    assert_parser_case("let_simple_type", false);
}

#[test]
fn let_bare_annotation_fn() {
    assert_parser_case("let_bare_annotation_fn", false);
}

#[test]
fn let_type_object_rank() {
    assert_parser_case("let_type_object_rank", false);
}

#[test]
fn let_guard_with() {
    assert_parser_case("let_guard_with", false);
}

#[test]
fn let_value_path() {
    assert_parser_case("let_value_path", false);
}

#[test]
fn invalid_missing_colon() {
    assert_parser_case("invalid_missing_colon", true);
}

#[test]
fn invalid_unexpected_symbol() {
    assert_parser_case("invalid_unexpected_symbol", true);
}

#[test]
fn pipe_basic() {
    assert_parser_case("pipe_basic", false);
}

#[test]
fn argpack_source() {
    assert_parser_case("argpack_source", false);
}

#[test]
fn argpack_right_target() {
    assert_parser_case("argpack_right_target", false);
}

#[test]
fn argpack_insert() {
    assert_parser_case("argpack_insert", false);
}

#[test]
fn argpack_multiple() {
    assert_parser_case("argpack_multiple", false);
}

#[test]
fn group_basic() {
    assert_parser_case("group_basic", false);
}

#[test]
fn let_pipe_value() {
    assert_parser_case("let_pipe_value", false);
}

#[test]
fn let_argpack_value() {
    assert_parser_case("let_argpack_value", false);
}

#[test]
fn invalid_empty_pipe_segment() {
    assert_parser_case("invalid_empty_pipe_segment", true);
}

#[test]
fn invalid_top_level_comma() {
    assert_parser_case("invalid_top_level_comma", true);
}

#[test]
fn unclosed_paren() {
    assert_parser_case("unclosed_paren", true);
}
