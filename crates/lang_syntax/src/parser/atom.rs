use crate::{AtomAst, AtomKind, DiagnosticCode, NameAst, Symbol, TokenKind};

use super::{form::Parser, pipe::parse_pipe_expr};

pub fn parse_atom(parser: &mut Parser<'_>) -> Option<AtomAst> {
    let token = parser.cursor.peek_non_trivia();

    let base = match &token.kind {
        TokenKind::Name => {
            let token = parser.cursor.bump_non_trivia();
            AtomAst {
                kind: AtomKind::Name(NameAst {
                    text: token.text.clone(),
                    span: token.span,
                }),
                span: token.span,
            }
        }
        TokenKind::IntLiteral => {
            let token = parser.cursor.bump_non_trivia();
            AtomAst {
                kind: AtomKind::IntLiteral(token.text.clone()),
                span: token.span,
            }
        }
        TokenKind::StringLiteral => {
            let token = parser.cursor.bump_non_trivia();
            AtomAst {
                kind: AtomKind::StringLiteral(token.text.clone()),
                span: token.span,
            }
        }
        TokenKind::Symbol(Symbol::LParen) => return parse_group(parser),
        _ => return None,
    };

    let mut names = Vec::new();
    let mut span = base.span;

    while parser.cursor.consume_symbol(Symbol::ColonColon).is_some() {
        let next = parser.cursor.peek_non_trivia();
        if !matches!(next.kind, TokenKind::Name) {
            parser.error(
                DiagnosticCode::ExpectedName,
                "expected name after `::`",
                next.span,
            );
            break;
        }

        let token = parser.cursor.bump_non_trivia();
        span = span.join(token.span);
        names.push(NameAst {
            text: token.text.clone(),
            span: token.span,
        });
    }

    if names.is_empty() {
        Some(base)
    } else {
        Some(AtomAst {
            span,
            kind: AtomKind::Path {
                base: Box::new(base),
                names,
            },
        })
    }
}

fn parse_group(parser: &mut Parser<'_>) -> Option<AtomAst> {
    let lparen = parser
        .cursor
        .consume_symbol(Symbol::LParen)
        .expect("parse_group called at `(`");

    let expr = parse_pipe_expr(parser, |p| {
        p.cursor.at_symbol(Symbol::Comma) || p.cursor.at_symbol(Symbol::RParen)
    });

    if parser.cursor.at_symbol(Symbol::Comma) {
        let comma_span = parser.cursor.current_span();
        parser.error(
            DiagnosticCode::UnexpectedToken,
            "unexpected top-level comma",
            comma_span,
        );
        parser.cursor.bump_non_trivia();
        parser.recover_to_paren_close();
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
    Some(AtomAst {
        kind: AtomKind::Group(Box::new(expr)),
        span,
    })
}
