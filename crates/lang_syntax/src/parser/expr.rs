use crate::{
    DiagnosticCode, ExprAst, ExprKind, PolicyLetAst, PolicySpecAst, ProductElementAst, Symbol,
};

use super::{form::Parser, pipe::parse_pipe_expr, policy::try_parse_policy_spec_before_let};

pub fn parse_expr_until(
    parser: &mut Parser<'_>,
    stop: impl FnMut(&mut Parser<'_>) -> bool,
) -> ExprAst {
    if parser.pattern_context {
        parse_ordinary_expr_until(parser, stop)
    } else {
        parse_colon_slots(parser, stop)
    }
}

fn parse_ordinary_expr_until(
    parser: &mut Parser<'_>,
    mut stop: impl FnMut(&mut Parser<'_>) -> bool,
) -> ExprAst {
    if let Some(policy) = try_parse_policy_spec_before_let(parser, |p| stop(p)) {
        parse_policy_let_after_policy(parser, policy, stop)
    } else {
        parse_pipe_expr(parser, stop)
    }
}

/// Syntactic Pattern interpretation, propagated through groups and products.
pub fn parse_pattern_expr_until(
    parser: &mut Parser<'_>,
    stop: impl FnMut(&mut Parser<'_>) -> bool,
) -> ExprAst {
    let previous = parser.pattern_context;
    parser.pattern_context = true;
    let expr = parse_expr_until(parser, stop);
    parser.pattern_context = previous;
    expr
}

fn parse_colon_slots(
    parser: &mut Parser<'_>,
    mut stop: impl FnMut(&mut Parser<'_>) -> bool,
) -> ExprAst {
    let start = parser.cursor.current_span();
    if !parser.cursor.at_symbol(Symbol::Colon) && (parser.is_form_boundary() || stop(parser)) {
        return parse_pipe_expr(parser, stop);
    }
    let mut slots = Vec::new();
    let mut has_colon = false;
    let mut empty_span = start;
    loop {
        if parser.cursor.at_symbol(Symbol::Colon) || parser.is_form_boundary() || stop(parser) {
            slots.push(ProductElementAst::Unit { span: empty_span });
        } else {
            slots.push(ProductElementAst::Expr(parse_ordinary_expr_until(
                parser,
                |p| p.cursor.at_symbol(Symbol::Colon) || stop(p),
            )));
        }
        if stop(parser) {
            break;
        }
        let Some(colon) = parser.cursor.consume_symbol(Symbol::Colon) else {
            break;
        };
        empty_span = colon.span;
        has_colon = true;
    }
    if !has_colon {
        return match slots.pop().expect("one structural slot") {
            ProductElementAst::Expr(expr) => expr,
            ProductElementAst::Unit { span } => ExprAst {
                kind: ExprKind::Error(parser.error_ast("expected expression", span)),
                span,
            },
        };
    }
    let end = match slots.last().expect("colon slots") {
        ProductElementAst::Expr(expr) => expr.span,
        ProductElementAst::Unit { span } => *span,
    };
    ExprAst {
        kind: ExprKind::Colon { slots },
        span: start.join(end),
    }
}

pub fn parse_policy_let_after_policy(
    parser: &mut Parser<'_>,
    policy: PolicySpecAst,
    mut stop: impl FnMut(&mut Parser<'_>) -> bool,
) -> ExprAst {
    let let_token = parser
        .cursor
        .consume_name("let")
        .expect("parse_policy_let_after_policy called at let");

    if parser.is_form_boundary() || stop(parser) {
        let operand_span = parser.cursor.current_span();
        parser.error(
            DiagnosticCode::ExpectedPolicyLetOperand,
            "expected expression after policy `let`",
            operand_span,
        );
        let span = policy.span.join(let_token.span);
        return ExprAst {
            kind: ExprKind::PolicyLet(PolicyLetAst {
                policy,
                operand: Box::new(ExprAst {
                    kind: ExprKind::Error(
                        parser.error_ast("expected policy-let operand", operand_span),
                    ),
                    span: operand_span,
                }),
                span,
            }),
            span,
        };
    }

    let operand = parse_pipe_expr(parser, stop);
    let span = policy.span.join(operand.span);
    ExprAst {
        kind: ExprKind::PolicyLet(PolicyLetAst {
            policy,
            operand: Box::new(operand),
            span,
        }),
        span,
    }
}
