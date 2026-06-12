use crate::{DiagnosticCode, ErrorAst, ExprAst, ExprKind, TokenKind};

use super::{atom::parse_atom, form::Parser};

pub fn parse_expr_until(
    parser: &mut Parser<'_>,
    mut stop: impl FnMut(&mut Parser<'_>) -> bool,
) -> ExprAst {
    let start = parser.cursor.current_span();
    let mut atoms = Vec::new();

    while !stop(parser) {
        if let Some(atom) = parse_atom(parser) {
            atoms.push(atom);
        } else if stop(parser) {
            break;
        } else {
            let token = parser.cursor.peek_non_trivia();
            if matches!(token.kind, TokenKind::Eof) {
                break;
            }
            parser.unexpected_current();
        }
    }

    if atoms.is_empty() {
        let span = parser.cursor.current_span();
        parser.error(DiagnosticCode::UnexpectedToken, "expected expression", span);
        return ExprAst {
            kind: ExprKind::Error(ErrorAst {
                message: "expected expression".to_string(),
                span,
            }),
            span,
        };
    }

    let span = atoms
        .first()
        .expect("atoms is not empty")
        .span
        .join(atoms.last().expect("atoms is not empty").span);

    ExprAst {
        kind: ExprKind::Segment(atoms),
        span: if span.byte_end >= start.byte_start {
            span
        } else {
            start
        },
    }
}
