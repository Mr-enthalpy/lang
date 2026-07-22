use crate::{PolicySpecAst, Symbol, ValuePolicyPatternAst};

use super::{expr::parse_expr_until, form::Parser};

pub fn try_parse_policy_spec_before_let(
    parser: &mut Parser<'_>,
    mut boundary: impl FnMut(&mut Parser<'_>) -> bool,
) -> Option<PolicySpecAst> {
    if parser.cursor.at_name("let") {
        return None;
    }

    let saved = parser.cursor.current_index();
    parser.gate_diagnostics();
    let policy = parse_policy_spec_until(parser, |p| p.cursor.at_name("let") || boundary(p));

    if parser.cursor.at_name("let") {
        parser.ungate_keep_diagnostics();
        Some(policy)
    } else {
        parser.cursor.set_index(saved);
        parser.ungate_drop_diagnostics();
        None
    }
}

pub fn parse_policy_spec_until(
    parser: &mut Parser<'_>,
    mut boundary: impl FnMut(&mut Parser<'_>) -> bool,
) -> PolicySpecAst {
    let value = parse_expr_until(parser, |p| p.cursor.at_symbol(Symbol::Colon) || boundary(p));
    let value_span = value.span;
    let type_policy = if parser.cursor.consume_symbol(Symbol::Colon).is_some() {
        Some(Box::new(parse_expr_until(parser, |p| boundary(p))))
    } else {
        None
    };
    let span = type_policy
        .as_ref()
        .map_or(value_span, |type_policy| value_span.join(type_policy.span));

    PolicySpecAst {
        value_policy: ValuePolicyPatternAst::Expr(Box::new(value)),
        type_policy,
        span,
    }
}
