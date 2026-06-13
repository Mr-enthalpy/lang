use crate::{ArgPackAst, ArgPackRole, DiagnosticCode, ExprAst, ExprKind, Span, Symbol};

use super::{form::Parser, pipe::parse_pipe_expr};

pub fn parse_argpack(parser: &mut Parser<'_>) -> ArgPackAst {
    let lparen = parser
        .cursor
        .consume_symbol(Symbol::LParen)
        .expect("parse_argpack called at `(`");

    parser.enter_nesting();

    let mut args = Vec::new();

    loop {
        if parser.cursor.at_eof()
            || parser.cursor.at_symbol(Symbol::RParen)
            || parser.is_form_boundary()
        {
            break;
        }

        let expr = parse_pipe_expr(parser, |p| {
            p.cursor.at_symbol(Symbol::Comma) || p.cursor.at_symbol(Symbol::RParen)
        });

        args.push(expr);

        if parser.cursor.consume_symbol(Symbol::Comma).is_some() {
            continue;
        }

        break;
    }

    let end = if let Some(rparen) = parser.cursor.consume_symbol(Symbol::RParen) {
        rparen.span
    } else {
        let span = parser.cursor.current_span();
        parser.error(
            DiagnosticCode::UnclosedParen,
            "unclosed parentheses",
            lparen.span,
        );
        span
    };

    let span = lparen.span.join(end);
    parser.leave_nesting();
    ArgPackAst {
        args,
        role: ArgPackRole::Unknown,
        span,
    }
}

pub fn error_expr(parser: &Parser<'_>, message: &str, span: Span) -> ExprAst {
    ExprAst {
        kind: ExprKind::Error(parser.error_ast(message, span)),
        span,
    }
}
