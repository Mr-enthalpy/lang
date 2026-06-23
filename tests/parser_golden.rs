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
fn expr_nav_path() {
    assert_parser_case("expr_nav_path", false);
}

#[test]
fn nav_group_scope() {
    assert_parser_case("nav_group_scope", false);
}

#[test]
fn nav_ungrouped_scope() {
    assert_parser_case("nav_ungrouped_scope", false);
}

#[test]
fn nav_two_elements() {
    assert_parser_case("nav_two_elements", false);
}

#[test]
fn let_simple_type() {
    assert_parser_case("let_simple_type", false);
}

#[test]
fn let_rank_annotation_fn() {
    assert_parser_case("let_rank_annotation_fn", false);
}

#[test]
fn let_type_object_rank() {
    assert_parser_case("let_type_object_rank", false);
}

#[test]
fn let_unannotated() {
    assert_parser_case("let_unannotated", false);
}

#[test]
fn let_annotation_neutral() {
    assert_parser_case("let_annotation_neutral", false);
}

#[test]
fn let_compound_binding_skeleton() {
    assert_parser_case("let_compound_binding_skeleton", false);
}

#[test]
fn let_with_lexical() {
    assert_parser_case("let_with_lexical", false);
}

#[test]
fn let_with_semantic_one() {
    assert_parser_case("let_with_semantic_one", false);
}

#[test]
fn let_with_semantic_many() {
    assert_parser_case("let_with_semantic_many", false);
}

#[test]
fn let_value_nav() {
    assert_parser_case("let_value_nav", false);
}

#[test]
fn invalid_missing_colon() {
    assert_parser_case("invalid_missing_colon", false);
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
fn nav_numeric_inner() {
    assert_parser_case("nav_numeric_inner", false);
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
fn let_extract_nav() {
    assert_parser_case("let_extract_nav", false);
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
fn invalid_closure_missing_body() {
    assert_parser_case("invalid_closure_missing_body", true);
}

#[test]
fn invalid_closure_unclosed_body() {
    assert_parser_case("invalid_closure_unclosed_body", true);
}

#[test]
fn invalid_closure_unclosed_params() {
    assert_parser_case("invalid_closure_unclosed_params", true);
}

#[test]
fn invalid_closure_unclosed_capture() {
    assert_parser_case("invalid_closure_unclosed_capture", true);
}

#[test]
fn invalid_closure_missing_param_after_comma() {
    assert_parser_case("invalid_closure_missing_param_after_comma", true);
}

#[test]
fn invalid_closure_bad_head_recovery() {
    assert_parser_case("invalid_closure_bad_head_recovery", true);
}

#[test]
fn invalid_closure_trailing_param_comma() {
    assert_parser_case("invalid_closure_trailing_param_comma", true);
}

#[test]
fn invalid_closure_missing_trait_after_colon() {
    assert_parser_case("invalid_closure_missing_trait_after_colon", true);
}

#[test]
fn invalid_closure_missing_return_after_arrow() {
    assert_parser_case("invalid_closure_missing_return_after_arrow", true);
}

#[test]
fn invalid_closure_capture_bad_head_recovery() {
    assert_parser_case("invalid_closure_capture_bad_head_recovery", true);
}

#[test]
fn closure_explicit_empty_params() {
    assert_parser_case("closure_explicit_empty_params", false);
}

#[test]
fn closure_explicit_name_param() {
    assert_parser_case("closure_explicit_name_param", false);
}

#[test]
fn closure_explicit_typed_param() {
    assert_parser_case("closure_explicit_typed_param", false);
}

#[test]
fn closure_explicit_deduce_param() {
    assert_parser_case("closure_explicit_deduce_param", false);
}

#[test]
fn closure_explicit_trait_clause() {
    assert_parser_case("closure_explicit_trait_clause", false);
}

#[test]
fn closure_explicit_return_type() {
    assert_parser_case("closure_explicit_return_type", false);
}

#[test]
fn closure_explicit_return_extract() {
    assert_parser_case("closure_explicit_return_extract", false);
}

#[test]
fn closure_explicit_full_head() {
    assert_parser_case("closure_explicit_full_head", false);
}

#[test]
fn closure_prefixed_inline() {
    assert_parser_case("closure_prefixed_inline", false);
}

#[test]
fn closure_prefixed_inline_params() {
    assert_parser_case("closure_prefixed_inline_params", false);
}

#[test]
fn closure_explicit_deduce_head() {
    assert_parser_case("closure_explicit_deduce_head", false);
}

#[test]
fn closure_explicit_multi_param() {
    assert_parser_case("closure_explicit_multi_param", false);
}

#[test]
fn closure_explicit_multi_typed_param() {
    assert_parser_case("closure_explicit_multi_typed_param", false);
}

#[test]
fn binding_param_unannotated() {
    assert_parser_case("binding_param_unannotated", false);
}

#[test]
fn binding_param_annotation() {
    assert_parser_case("binding_param_annotation", false);
}

#[test]
fn binding_param_let_with_empty() {
    assert_parser_case("binding_param_let_with_empty", false);
}

#[test]
fn binding_param_deduce_pattern_with_empty() {
    assert_parser_case("binding_param_deduce_pattern_with_empty", false);
}

#[test]
fn binding_param_with_items() {
    assert_parser_case("binding_param_with_items", false);
}

#[test]
fn closure_in_argpack_match_style() {
    assert_parser_case("closure_in_argpack_match_style", false);
}

#[test]
fn closure_group_not_head() {
    assert_parser_case("closure_group_not_head", false);
}

#[test]
fn closure_argpack_not_head() {
    assert_parser_case("closure_argpack_not_head", false);
}

#[test]
fn closure_bare_name_not_head() {
    assert_parser_case("closure_bare_name_not_head", true);
}

#[test]
fn invalid_closure_where_not_parsed() {
    assert_parser_case("invalid_closure_where_not_parsed", true);
}

#[test]
fn invalid_closure_acquire_not_parsed() {
    assert_parser_case("invalid_closure_acquire_not_parsed", true);
}

#[test]
fn invalid_bare_closure_empty() {
    assert_parser_case("invalid_bare_closure_empty", true);
}

#[test]
fn invalid_bare_closure_body() {
    assert_parser_case("invalid_bare_closure_body", true);
}

#[test]
fn closure_deduce_where_not_clause() {
    assert_parser_case("closure_deduce_where_not_clause", true);
}

#[test]
fn closure_deduce_acquire_not_clause() {
    assert_parser_case("closure_deduce_acquire_not_clause", true);
}

#[test]
fn closure_body_multi_form() {
    assert_parser_case("closure_body_multi_form", false);
}

#[test]
fn closure_body_newline_single_form() {
    assert_parser_case("closure_body_newline_single_form", false);
}

#[test]
fn closure_body_semicolon_two_forms() {
    assert_parser_case("closure_body_semicolon_two_forms", false);
}

#[test]
fn closure_capture_simple() {
    assert_parser_case("closure_capture_simple", false);
}

#[test]
fn closure_capture_multiple() {
    assert_parser_case("closure_capture_multiple", false);
}

#[test]
fn closure_return_type() {
    assert_parser_case("closure_return_type", false);
}

#[test]
fn closure_return_constraint() {
    assert_parser_case("closure_return_constraint", false);
}

#[test]
fn closure_return_extract() {
    assert_parser_case("closure_return_extract", false);
}

#[test]
fn closure_return_extract_constraint() {
    assert_parser_case("closure_return_extract_constraint", false);
}

#[test]
fn binding_return_wildcard() {
    assert_parser_case("binding_return_wildcard", false);
}

#[test]
fn binding_return_wildcard_annotation() {
    assert_parser_case("binding_return_wildcard_annotation", false);
}

#[test]
fn binding_return_named() {
    assert_parser_case("binding_return_named", false);
}

#[test]
fn binding_return_named_annotation() {
    assert_parser_case("binding_return_named_annotation", false);
}

#[test]
fn binding_return_let_named_annotation() {
    assert_parser_case("binding_return_let_named_annotation", false);
}

#[test]
fn invalid_binding_return_with() {
    assert_parser_case("invalid_binding_return_with", true);
}

#[test]
fn invalid_deduce_unclosed() {
    assert_parser_case("invalid_deduce_unclosed", true);
}

#[test]
fn invalid_deduce_missing_annotation() {
    assert_parser_case("invalid_deduce_missing_annotation", true);
}

#[test]
fn invalid_deduce_missing_annotation_comma() {
    assert_parser_case("invalid_deduce_missing_annotation_comma", true);
}

#[test]
fn invalid_deduce_missing_name() {
    assert_parser_case("invalid_deduce_missing_name", true);
}

#[test]
fn invalid_canonical_comma_before_equal() {
    assert_parser_case("invalid_canonical_comma_before_equal", true);
}

#[test]
fn invalid_canonical_bracket_before_equal() {
    assert_parser_case("invalid_canonical_bracket_before_equal", true);
}

#[test]
fn invalid_deduce_missing_greater_before_equal() {
    assert_parser_case("invalid_deduce_missing_greater_before_equal", true);
}

#[test]
fn invalid_deduce_hole_before_equal() {
    assert_parser_case("invalid_deduce_hole_before_equal", true);
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

#[test]
fn operator_binary_add() {
    assert_parser_case("operator_binary_add", false);
}

#[test]
fn operator_binary_multiply() {
    assert_parser_case("operator_binary_multiply", false);
}

#[test]
fn operator_precedence_multiply_before_add() {
    assert_parser_case("operator_precedence_multiply_before_add", false);
}

#[test]
fn operator_left_assoc_add() {
    assert_parser_case("operator_left_assoc_add", false);
}

#[test]
fn operator_left_assoc_shift() {
    assert_parser_case("operator_left_assoc_shift", false);
}

#[test]
fn operator_comparison() {
    assert_parser_case("operator_comparison", false);
}

#[test]
fn operator_equality() {
    assert_parser_case("operator_equality", false);
}

#[test]
fn operator_compound_looking() {
    assert_parser_case("operator_compound_looking", false);
}

#[test]
fn operator_prefix_minus_name() {
    assert_parser_case("operator_prefix_minus_name", false);
}

#[test]
fn operator_prefix_minus_literal() {
    assert_parser_case("operator_prefix_minus_literal", false);
}

#[test]
fn operator_postfix_bang() {
    assert_parser_case("operator_postfix_bang", false);
}

#[test]
fn operator_postfix_question() {
    assert_parser_case("operator_postfix_question", false);
}

#[test]
fn operator_postfix_chain() {
    assert_parser_case("operator_postfix_chain", false);
}

#[test]
fn operator_suffix_then_postfix() {
    assert_parser_case("operator_suffix_then_postfix", false);
}

#[test]
fn operator_postfix_then_member() {
    assert_parser_case("operator_postfix_then_member", false);
}

#[test]
fn operator_doubledot_then_postfix() {
    assert_parser_case("operator_doubledot_then_postfix", false);
}

#[test]
fn operator_pipe_precedence() {
    assert_parser_case("operator_pipe_precedence", false);
}

#[test]
fn operator_segment_local() {
    assert_parser_case("operator_segment_local", false);
}

#[test]
fn operator_angle_less_greater() {
    assert_parser_case("operator_angle_less_greater", false);
}

#[test]
fn operator_grouped_nonassoc_left() {
    assert_parser_case("operator_grouped_nonassoc_left", false);
}

#[test]
fn operator_grouped_nonassoc_right() {
    assert_parser_case("operator_grouped_nonassoc_right", false);
}

#[test]
fn operator_inside_argpack() {
    assert_parser_case("operator_inside_argpack", false);
}

#[test]
fn operator_inside_closure_body() {
    assert_parser_case("operator_inside_closure_body", false);
}

#[test]
fn invalid_operator_missing_rhs_add() {
    assert_parser_case("invalid_operator_missing_rhs_add", true);
}

#[test]
fn invalid_operator_missing_lhs_add() {
    assert_parser_case("invalid_operator_missing_lhs_add", true);
}

#[test]
fn invalid_operator_missing_rhs_multiply() {
    assert_parser_case("invalid_operator_missing_rhs_multiply", true);
}

#[test]
fn invalid_operator_chained_comparison() {
    assert_parser_case("invalid_operator_chained_comparison", true);
}

#[test]
fn invalid_operator_chained_equality() {
    assert_parser_case("invalid_operator_chained_equality", true);
}

#[test]
fn invalid_operator_chained_compound() {
    assert_parser_case("invalid_operator_chained_compound", true);
}

#[test]
fn invalid_operator_unsupported_prefix_bang() {
    assert_parser_case("invalid_operator_unsupported_prefix_bang", true);
}

#[test]
fn invalid_operator_unsupported_prefix_star() {
    assert_parser_case("invalid_operator_unsupported_prefix_star", true);
}

#[test]
fn invalid_operator_unsupported_prefix_increment() {
    assert_parser_case("invalid_operator_unsupported_prefix_increment", true);
}

#[test]
fn operator_binder_plus() {
    assert_parser_case("operator_binder_plus", false);
}

#[test]
fn operator_binder_shift() {
    assert_parser_case("operator_binder_shift", false);
}

#[test]
fn operator_binder_postfix_bang() {
    assert_parser_case("operator_binder_postfix_bang", false);
}

#[test]
fn operator_nav_inner_plus() {
    assert_parser_case("operator_nav_inner_plus", false);
}

#[test]
fn operator_nav_inner_shift() {
    assert_parser_case("operator_nav_inner_shift", false);
}

#[test]
fn nav_numeric_inner_chain() {
    assert_parser_case("nav_numeric_inner_chain", false);
}

#[test]
fn operator_nav_in_expression() {
    assert_parser_case("operator_nav_in_expression", false);
}

#[test]
fn operator_nav_then_binary_operator() {
    assert_parser_case("operator_nav_then_binary_operator", false);
}

#[test]
fn invalid_operator_binder_missing_colon() {
    assert_parser_case("invalid_operator_binder_missing_colon", false);
}

#[test]
fn invalid_nav_outer_operator() {
    assert_parser_case("invalid_nav_outer_operator", true);
}

#[test]
fn invalid_nav_outer_operator_after_scope() {
    assert_parser_case("invalid_nav_outer_operator_after_scope", true);
}

#[test]
fn invalid_nav_operator_outer_after_inner() {
    assert_parser_case("invalid_nav_operator_outer_after_inner", true);
}

#[test]
fn invalid_operator_member_selector() {
    assert_parser_case("invalid_operator_member_selector", true);
}

#[test]
fn invalid_operator_doubledot_selector() {
    assert_parser_case("invalid_operator_doubledot_selector", true);
}

#[test]
fn let_alias_operator_plus() {
    assert_parser_case("let_alias_operator_plus", false);
}

#[test]
fn let_alias_name_simple() {
    assert_parser_case("let_alias_name_simple", false);
}

#[test]
fn let_alias_name_path() {
    assert_parser_case("let_alias_name_path", false);
}

#[test]
fn let_alias_operator_shift() {
    assert_parser_case("let_alias_operator_shift", false);
}

#[test]
fn let_alias_operator_plus_multiline() {
    assert_parser_case("let_alias_operator_plus_multiline", false);
}

#[test]
fn let_alias_operator_shift_multiline() {
    assert_parser_case("let_alias_operator_shift_multiline", false);
}

#[test]
fn let_alias_single_leaf() {
    assert_parser_case("let_alias_single_leaf", false);
}

#[test]
fn let_alias_operator_unqualified() {
    assert_parser_case("let_alias_operator_unqualified", false);
}

#[test]
fn invalid_alias_missing_target() {
    assert_parser_case("invalid_alias_missing_target", true);
}

#[test]
fn invalid_alias_rhs_pipe() {
    assert_parser_case("invalid_alias_rhs_pipe", true);
}

#[test]
fn invalid_alias_rhs_closure() {
    assert_parser_case("invalid_alias_rhs_closure", true);
}

#[test]
fn invalid_alias_rhs_argpack() {
    assert_parser_case("invalid_alias_rhs_argpack", true);
}

#[test]
fn invalid_alias_rhs_operator_expr() {
    assert_parser_case("invalid_alias_rhs_operator_expr", true);
}

#[test]
fn invalid_alias_operator_intermediate_segment() {
    assert_parser_case("invalid_alias_operator_intermediate_segment", true);
}

#[test]
fn let_alias_grouped_outer_scope() {
    assert_parser_case("let_alias_grouped_outer_scope", false);
}

#[test]
fn let_alias_operator_inner_qualified() {
    assert_parser_case("let_alias_operator_inner_qualified", false);
}

#[test]
fn invalid_alias_grouped_innermost() {
    assert_parser_case("invalid_alias_grouped_innermost", true);
}

#[test]
fn invalid_nav_grouped_innermost() {
    assert_parser_case("invalid_nav_grouped_innermost", true);
}

#[test]
fn invalid_alias_outer_operator() {
    assert_parser_case("invalid_alias_outer_operator", true);
}

#[test]
fn invalid_alias_outer_operator_after_scope() {
    assert_parser_case("invalid_alias_outer_operator_after_scope", true);
}

#[test]
fn invalid_alias_bad_binder() {
    assert_parser_case("invalid_alias_bad_binder", true);
}

#[test]
fn invalid_alias_extract_let_not_alias() {
    assert_parser_case("invalid_alias_extract_let_not_alias", true);
}

#[test]
fn let_alias_following_form() {
    assert_parser_case("let_alias_following_form", false);
}

#[test]
fn let_alias_semicolon_next_form() {
    assert_parser_case("let_alias_semicolon_next_form", false);
}

#[test]
fn invalid_alias_missing_target_recovery() {
    assert_parser_case("invalid_alias_missing_target_recovery", true);
}

#[test]
fn invalid_alias_rhs_member() {
    assert_parser_case("invalid_alias_rhs_member", true);
}

#[test]
fn invalid_alias_rhs_doubledot() {
    assert_parser_case("invalid_alias_rhs_doubledot", true);
}

#[test]
fn invalid_alias_target_trailing_coloncolon() {
    assert_parser_case("invalid_alias_target_trailing_coloncolon", true);
}

#[test]
fn invalid_alias_target_leading_coloncolon() {
    assert_parser_case("invalid_alias_target_leading_coloncolon", true);
}

#[test]
fn invalid_alias_guard_not_alias() {
    assert_parser_case("invalid_alias_guard_not_alias", true);
}

#[test]
fn let_guard_not_attr() {
    assert_parser_case("let_guard_not_attr", false);
}

#[test]
fn invalid_alias_annotation_not_alias() {
    assert_parser_case("invalid_alias_annotation_not_alias", true);
}

#[test]
fn invalid_alias_with_not_alias() {
    assert_parser_case("invalid_alias_with_not_alias", true);
}

#[test]
fn invalid_with_namelist() {
    assert_parser_case("invalid_with_namelist", true);
}

#[test]
fn invalid_with_missing_block() {
    assert_parser_case("invalid_with_missing_block", true);
}

#[test]
fn invalid_with_unclosed_block() {
    assert_parser_case("invalid_with_unclosed_block", true);
}

#[test]
fn invalid_with_trailing_comma() {
    assert_parser_case("invalid_with_trailing_comma", true);
}

#[test]
fn invalid_alias_expression_position() {
    assert_parser_case("invalid_alias_expression_position", true);
}
