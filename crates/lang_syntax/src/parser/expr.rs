use crate::{DiagnosticCode, ExprAst, ExprKind, PolicyLetAst, PolicySpecAst};

use super::{form::Parser, pipe::parse_pipe_expr, policy::try_parse_policy_spec_before_let};

pub fn parse_expr_until(
    parser: &mut Parser<'_>,
    mut stop: impl FnMut(&mut Parser<'_>) -> bool,
) -> ExprAst {
    if let Some(policy) = try_parse_policy_spec_before_let(parser, |p| stop(p)) {
        return parse_policy_let_after_policy(parser, policy, stop);
    }
    parse_pipe_expr(parser, stop)
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
