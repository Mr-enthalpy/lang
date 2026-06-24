use std::fs;
use std::path::PathBuf;

fn case_path(name: &str, extension: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("cases")
        .join("lexer")
        .join(format!("{name}.{extension}"))
}

fn assert_lexer_case(name: &str, expect_diagnostics: bool) {
    let source = fs::read_to_string(case_path(name, "lang")).expect("read source fixture");
    let expected_tokens = lang_syntax::normalize_source_text(
        &fs::read_to_string(case_path(name, "tokens")).expect("read token fixture"),
    );
    let output = lang_syntax::lex(&source);

    assert_eq!(lang_syntax::dump_tokens(&output.tokens), expected_tokens);

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
fn names() {
    assert_lexer_case("names", false);
}

#[test]
fn int_dot_int() {
    // Locks that `1.2` lexes as a single `FloatLiteral` token.
    assert_lexer_case("int_dot_int", false);
}

#[test]
fn float_with_name() {
    // Locks that `1.2ms` lexes as `FloatLiteral` `Name`, not unit-suffixed.
    assert_lexer_case("float_with_name", false);
}

#[test]
fn symbols() {
    assert_lexer_case("symbols", false);
}

#[test]
fn trivia() {
    assert_lexer_case("trivia", false);
}

#[test]
fn literals() {
    assert_lexer_case("literals", false);
}

#[test]
fn invalid() {
    assert_lexer_case("invalid", true);
}

#[test]
fn unclosed_string() {
    assert_lexer_case("unclosed_string", true);
}

#[test]
fn unclosed_comment() {
    assert_lexer_case("unclosed_comment", true);
}

#[test]
fn operators() {
    assert_lexer_case("operators", false);
}

#[test]
fn structural_with_operators() {
    assert_lexer_case("structural_with_operators", false);
}

#[test]
fn triple_equal_token() {
    assert_lexer_case("triple_equal_token", false);
}

#[test]
fn triple_equal_plus_equal_tail() {
    assert_lexer_case("triple_equal_plus_equal_tail", false);
}

#[test]
fn nested_block_comment() {
    assert_lexer_case("nested_block_comment", false);
}

#[test]
fn block_comment_with_slashes() {
    assert_lexer_case("block_comment_with_slashes", false);
}

#[test]
fn line_comment_with_block_delims() {
    assert_lexer_case("line_comment_with_block_delims", false);
}

#[test]
fn unclosed_nested_block_comment() {
    assert_lexer_case("unclosed_nested_block_comment", true);
}
