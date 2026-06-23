use crate::{ArgPackAst, ArgPackRole, DiagnosticCode, ExprAst, ExprKind, Span, Symbol};

use super::{form::Parser, pipe::parse_pipe_expr};

pub fn parse_argpack(parser: &mut Parser<'_>) -> ArgPackAst {
    parse_delimited_argpack(
        parser,
        Symbol::LParen,
        Symbol::RParen,
        DiagnosticCode::UnclosedParen,
        "unclosed parentheses",
    )
}

pub fn parse_bracket_argpack(parser: &mut Parser<'_>) -> ArgPackAst {
    parse_delimited_argpack(
        parser,
        Symbol::LBracket,
        Symbol::RBracket,
        DiagnosticCode::UnclosedBracket,
        "unclosed brackets",
    )
}

// Parse a delimited argument pack. Commas create argument slots; an empty slot
// (leading, double, or trailing comma) is preserved as `unit`. Zero slots
// without any comma is an empty argpack. The parser does not validate whether
// unit arguments are meaningful for a given call/operator.
fn parse_delimited_argpack(
    parser: &mut Parser<'_>,
    open: Symbol,
    close: Symbol,
    unclosed: DiagnosticCode,
    unclosed_message: &str,
) -> ArgPackAst {
    let open_token = parser
        .cursor
        .consume_symbol(open)
        .expect("parse_delimited_argpack called at opening delimiter");

    parser.enter_nesting();

    let mut args = Vec::new();

    // Zero slots without commas is an empty argpack (`()`, `obj[]`).
    if !at_argpack_end(parser, close) {
        loop {
            if parser.cursor.at_symbol(Symbol::Comma) || at_argpack_end(parser, close) {
                // Empty slot -> unit.
                let span = parser.cursor.current_span();
                args.push(unit_expr(span));
            } else {
                let expr = parse_pipe_expr(parser, |p| {
                    p.cursor.at_symbol(Symbol::Comma) || p.cursor.at_symbol(close)
                });
                args.push(expr);
            }

            if parser.cursor.consume_symbol(Symbol::Comma).is_some() {
                continue;
            }

            break;
        }
    }

    let end = if let Some(close_token) = parser.cursor.consume_symbol(close) {
        close_token.span
    } else {
        let span = parser.cursor.current_span();
        parser.error(unclosed, unclosed_message, open_token.span);
        span
    };

    let span = open_token.span.join(end);
    parser.leave_nesting();
    ArgPackAst {
        args,
        role: ArgPackRole::Unknown,
        span,
    }
}

fn at_argpack_end(parser: &mut Parser<'_>, close: Symbol) -> bool {
    parser.cursor.at_eof() || parser.cursor.at_symbol(close) || parser.is_form_boundary()
}

fn unit_expr(span: Span) -> ExprAst {
    ExprAst {
        kind: ExprKind::Unit,
        span,
    }
}

pub fn error_expr(parser: &Parser<'_>, message: &str, span: Span) -> ExprAst {
    ExprAst {
        kind: ExprKind::Error(parser.error_ast(message, span)),
        span,
    }
}
