use crate::{AtomAst, AtomKind, DiagnosticCode, NameAst, Symbol, TokenKind};

use super::form::Parser;

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
