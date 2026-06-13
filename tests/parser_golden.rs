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

#[test]
fn member_basic() {
    assert_parser_case("member_basic", false);
}

#[test]
fn member_numeric() {
    assert_parser_case("member_numeric", false);
}

#[test]
fn member_numeric2() {
    assert_parser_case("member_numeric2", false);
}

#[test]
fn member_numeric_base() {
    assert_parser_case("member_numeric_base", false);
}

#[test]
fn member_nested() {
    assert_parser_case("member_nested", false);
}

#[test]
fn path_numeric_leaf() {
    assert_parser_case("path_numeric_leaf", false);
}

#[test]
fn doubledot_basic() {
    assert_parser_case("doubledot_basic", false);
}

#[test]
fn doubledot_numeric() {
    assert_parser_case("doubledot_numeric", false);
}

#[test]
fn mixed_suffixes() {
    assert_parser_case("mixed_suffixes", false);
}

#[test]
fn mixed_numeric() {
    assert_parser_case("mixed_numeric", false);
}

#[test]
fn doubledot_next_element() {
    assert_parser_case("doubledot_next_element", false);
}

#[test]
fn invalid_doubledot_missing_argpack_field() {
    assert_parser_case("invalid_doubledot_missing_argpack_field", true);
}

#[test]
fn newline_two_expr_forms() {
    assert_parser_case("newline_two_expr_forms", false);
}

#[test]
fn newline_two_let_forms() {
    assert_parser_case("newline_two_let_forms", false);
}

#[test]
fn newline_after_pipe_form() {
    assert_parser_case("newline_after_pipe_form", false);
}

#[test]
fn newline_inside_group_not_boundary() {
    assert_parser_case("newline_inside_group_not_boundary", false);
}

#[test]
fn newline_inside_argpack_not_boundary() {
    assert_parser_case("newline_inside_argpack_not_boundary", false);
}

#[test]
fn semicolon_still_boundary() {
    assert_parser_case("semicolon_still_boundary", false);
}

#[test]
fn newline_before_pipe() {
    assert_parser_case("newline_before_pipe", false);
}

#[test]
fn newline_between_dot_and_field() {
    assert_parser_case("newline_between_dot_and_field", false);
}

#[test]
fn let_extract_pair() {
    assert_parser_case("let_extract_pair", false);
}

#[test]
fn let_extract_wildcard() {
    assert_parser_case("let_extract_wildcard", false);
}

#[test]
fn let_extract_path() {
    assert_parser_case("let_extract_path", false);
}

#[test]
fn let_extract_literal() {
    assert_parser_case("let_extract_literal", false);
}

#[test]
fn let_extract_nested_argpack() {
    assert_parser_case("let_extract_nested_argpack", false);
}

#[test]
fn let_extract_annotated_deduce() {
    assert_parser_case("let_extract_annotated_deduce", false);
}

#[test]
fn let_extract_name_hole() {
    assert_parser_case("let_extract_name_hole", false);
}

#[test]
fn let_extract_empty_deduce() {
    assert_parser_case("let_extract_empty_deduce", true);
}

#[test]
fn let_extract_with_clause() {
    assert_parser_case("let_extract_with_clause", false);
}

#[test]
fn let_extract_hole_annotation() {
    assert_parser_case("let_extract_hole_annotation", false);
}

#[test]
fn invalid_deduce_trailing_comma() {
    assert_parser_case("invalid_deduce_trailing_comma", true);
}

#[test]
fn invalid_deduce_missing_name() {
    assert_parser_case("invalid_deduce_missing_name", true);
}

#[test]
fn invalid_deduce_unclosed() {
    assert_parser_case("invalid_deduce_unclosed", true);
}

#[test]
fn invalid_canonical_missing_skeleton() {
    assert_parser_case("invalid_canonical_missing_skeleton", true);
}

#[test]
fn invalid_canonical_unclosed_argpack() {
    assert_parser_case("invalid_canonical_unclosed_argpack", true);
}

#[test]
fn invalid_canonical_trailing_comma() {
    assert_parser_case("invalid_canonical_trailing_comma", true);
}

#[test]
fn newline_between_coloncolon_and_leaf() {
    assert_parser_case("newline_between_coloncolon_and_leaf", false);
}

#[test]
fn newline_after_pipe_before_segment() {
    assert_parser_case("newline_after_pipe_before_segment", false);
}

#[test]
fn newline_before_dot() {
    assert_parser_case("newline_before_dot", false);
}

#[test]
fn newline_before_coloncolon() {
    assert_parser_case("newline_before_coloncolon", false);
}
