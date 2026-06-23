use lang_syntax;
use std::fs;
use std::path::PathBuf;

fn case_path(name: &str, extension: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("cases")
        .join("diagnostics")
        .join(format!("{name}.{extension}"))
}

fn assert_diagnostics_case(name: &str) {
    let source = fs::read_to_string(case_path(name, "lang")).expect("read source fixture");
    let expected_diagnostics = lang_syntax::normalize_source_text(
        &fs::read_to_string(case_path(name, "diag")).expect("read diagnostic fixture"),
    );
    let output = lang_syntax::parse(&source);

    assert_eq!(
        lang_syntax::dump_diagnostics(&output.diagnostics),
        expected_diagnostics
    );
}

#[test]
fn invalid_token() {
    assert_diagnostics_case("invalid_token");
}

#[test]
fn unclosed_string() {
    assert_diagnostics_case("unclosed_string");
}

#[test]
fn unclosed_comment() {
    assert_diagnostics_case("unclosed_comment");
}

#[test]
fn missing_binding_pattern() {
    assert_diagnostics_case("missing_binding_pattern");
}

#[test]
fn missing_decl_annotation() {
    assert_diagnostics_case("missing_decl_annotation");
}

#[test]
fn missing_equal() {
    assert_diagnostics_case("missing_equal");
}

#[test]
fn empty_pipe_segment() {
    assert_diagnostics_case("empty_pipe_segment");
}

#[test]
fn top_level_comma() {
    assert_diagnostics_case("top_level_comma");
}

#[test]
fn unclosed_paren() {
    assert_diagnostics_case("unclosed_paren");
}

#[test]
fn expected_name_after_dot_paren() {
    assert_diagnostics_case("expected_name_after_dot_paren");
}

#[test]
fn expected_name_after_dot_string() {
    assert_diagnostics_case("expected_name_after_dot_string");
}

#[test]
fn expected_name_after_dot_eof() {
    assert_diagnostics_case("expected_name_after_dot_eof");
}

#[test]
fn expected_name_after_dot_operator() {
    assert_diagnostics_case("expected_name_after_dot_operator");
}

#[test]
fn expected_name_after_doubledot_paren() {
    assert_diagnostics_case("expected_name_after_doubledot_paren");
}

#[test]
fn expected_name_after_doubledot_string() {
    assert_diagnostics_case("expected_name_after_doubledot_string");
}

#[test]
fn expected_name_after_doubledot_eof() {
    assert_diagnostics_case("expected_name_after_doubledot_eof");
}

#[test]
fn expected_name_after_doubledot_operator() {
    assert_diagnostics_case("expected_name_after_doubledot_operator");
}

#[test]
fn expected_argpack_after_doubledot_text() {
    assert_diagnostics_case("expected_argpack_after_doubledot_text");
}

#[test]
fn expected_argpack_after_doubledot_text_next() {
    assert_diagnostics_case("expected_argpack_after_doubledot_text_next");
}

#[test]
fn expected_argpack_after_doubledot_text_field() {
    assert_diagnostics_case("expected_argpack_after_doubledot_text_field");
}

#[test]
fn expected_argpack_after_doubledot_numeric() {
    assert_diagnostics_case("expected_argpack_after_doubledot_numeric");
}

#[test]
fn expected_argpack_after_doubledot_numeric_next() {
    assert_diagnostics_case("expected_argpack_after_doubledot_numeric_next");
}

#[test]
fn expected_argpack_after_doubledot_numeric_field() {
    assert_diagnostics_case("expected_argpack_after_doubledot_numeric_field");
}

#[test]
fn empty_pipe_segment_newline() {
    assert_diagnostics_case("empty_pipe_segment_newline");
}

#[test]
fn top_level_comma_newline() {
    assert_diagnostics_case("top_level_comma_newline");
}

#[test]
fn missing_equal_before_newline() {
    assert_diagnostics_case("missing_equal_before_newline");
}

#[test]
fn unexpected_token_before_newline() {
    assert_diagnostics_case("unexpected_token_before_newline");
}

#[test]
fn invalid_return_with() {
    assert_diagnostics_case("invalid_return_with");
}

#[test]
fn invalid_with_namelist() {
    assert_diagnostics_case("invalid_with_namelist");
}

#[test]
fn invalid_bare_closure_empty() {
    assert_diagnostics_case("invalid_bare_closure_empty");
}

#[test]
fn invalid_with_missing_block() {
    assert_diagnostics_case("invalid_with_missing_block");
}

#[test]
fn invalid_with_unclosed_block() {
    assert_diagnostics_case("invalid_with_unclosed_block");
}

#[test]
fn invalid_head_clause_comma() {
    assert_diagnostics_case("invalid_head_clause_comma");
}

#[test]
fn invalid_head_clause_missing_expr() {
    assert_diagnostics_case("invalid_head_clause_missing_expr");
}
