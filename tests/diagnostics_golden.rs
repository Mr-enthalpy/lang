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
fn missing_colon() {
    assert_diagnostics_case("missing_colon");
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
