use crate::{
    AtomAst, AtomKind, DiagnosticCode, NameAst, NumericNameAst, OperatorExprAst, OperatorExprKind,
    SelectorAst, Span, Symbol, TokenKind,
};

use super::{argpack::parse_argpack, form::Parser, pipe::parse_pipe_expr};

// Current-phase operator-expr wrapper. In the future operator parser phase,
// this function will also parse binary/postfix/prefix operator expressions.
// For now it wraps every atom in OperatorExprAst.
pub fn parse_operator_expr_current_phase(parser: &mut Parser<'_>) -> Option<OperatorExprAst> {
    let atom = parse_atom(parser)?;
    let span = atom.span;
    Some(OperatorExprAst {
        kind: OperatorExprKind::Atom(atom),
        span,
    })
}

pub fn parse_atom(parser: &mut Parser<'_>) -> Option<AtomAst> {
    if parser.is_form_boundary() {
        return None;
    }
    let mut atom = parse_atom_base(parser)?;

    loop {
        if parser.at_top_level_newline() {
            break;
        }
        if parser.is_form_boundary() {
            break;
        }
        if parser.cursor.at_symbol(Symbol::ColonColon) {
            parser.cursor.bump_non_trivia();
            if let Some(selector) = parse_selector(parser) {
                atom = extend_or_create_path(atom, selector);
            } else {
                let span = parser.cursor.current_span();
                parser.error(
                    DiagnosticCode::ExpectedName,
                    "expected name after `::`",
                    span,
                );
                break;
            }
        } else if parser.cursor.at_symbol(Symbol::Dot) {
            let dot_token = parser.cursor.bump_non_trivia();
            if let Some(selector) = parse_selector(parser) {
                let span = atom.span.join(selector_span(&selector));
                atom = AtomAst {
                    kind: AtomKind::MemberSugar {
                        object: Box::new(atom),
                        selector,
                    },
                    span,
                };
            } else {
                parser.error(
                    DiagnosticCode::ExpectedNameAfterDot,
                    "expected name after `.`",
                    dot_token.span,
                );
                break;
            }
        } else if parser.cursor.at_symbol(Symbol::DotDot) {
            let dotdot_token = parser.cursor.bump_non_trivia();
            if let Some(selector) = parse_selector(parser) {
                if parser.cursor.at_symbol(Symbol::LParen) {
                    let argpack = parse_argpack(parser);
                    let span = atom.span.join(argpack.span);
                    atom = AtomAst {
                        kind: AtomKind::DoubleDotSugar {
                            object: Box::new(atom),
                            selector,
                            args: argpack,
                        },
                        span,
                    };
                } else {
                    parser.error(
                        DiagnosticCode::ExpectedArgPackAfterDoubleDotName,
                        "expected argument pack after `.. Selector`",
                        selector_span(&selector),
                    );
                    break;
                }
            } else {
                parser.error(
                    DiagnosticCode::ExpectedNameAfterDoubleDot,
                    "expected name after `..`",
                    dotdot_token.span,
                );
                break;
            }
        } else {
            break;
        }
    }

    Some(atom)
}

fn parse_atom_base(parser: &mut Parser<'_>) -> Option<AtomAst> {
    let token = parser.cursor.peek_non_trivia();

    match &token.kind {
        TokenKind::Name => {
            let token = parser.cursor.bump_non_trivia();
            Some(AtomAst {
                kind: AtomKind::Name(NameAst {
                    text: token.text.clone(),
                    span: token.span,
                }),
                span: token.span,
            })
        }
        TokenKind::IntLiteral => {
            let token = parser.cursor.bump_non_trivia();
            Some(AtomAst {
                kind: AtomKind::IntLiteral(token.text.clone()),
                span: token.span,
            })
        }
        TokenKind::StringLiteral => {
            let token = parser.cursor.bump_non_trivia();
            Some(AtomAst {
                kind: AtomKind::StringLiteral(token.text.clone()),
                span: token.span,
            })
        }
        TokenKind::Symbol(Symbol::LParen) => parse_group(parser),
        _ => None,
    }
}

fn parse_selector(parser: &mut Parser<'_>) -> Option<SelectorAst> {
    let token = parser.cursor.peek_non_trivia();
    match token.kind {
        TokenKind::Name => {
            let token = parser.cursor.bump_non_trivia();
            Some(SelectorAst::Text(NameAst {
                text: token.text.clone(),
                span: token.span,
            }))
        }
        TokenKind::IntLiteral => {
            let token = parser.cursor.bump_non_trivia();
            Some(SelectorAst::Numeric(NumericNameAst {
                text: token.text.clone(),
                span: token.span,
            }))
        }
        _ => None,
    }
}

fn selector_span(selector: &SelectorAst) -> Span {
    match selector {
        SelectorAst::Text(name) => name.span,
        SelectorAst::Numeric(num) => num.span,
    }
}

fn extend_or_create_path(atom: AtomAst, selector: SelectorAst) -> AtomAst {
    let sel_span = selector_span(&selector);
    match atom.kind {
        AtomKind::Path { base, mut names } => {
            let span = atom.span.join(sel_span);
            names.push(selector);
            AtomAst {
                kind: AtomKind::Path { base, names },
                span,
            }
        }
        _ => {
            let span = atom.span.join(sel_span);
            AtomAst {
                kind: AtomKind::Path {
                    base: Box::new(atom),
                    names: vec![selector],
                },
                span,
            }
        }
    }
}

fn parse_group(parser: &mut Parser<'_>) -> Option<AtomAst> {
    let lparen = parser
        .cursor
        .consume_symbol(Symbol::LParen)
        .expect("parse_group called at `(`");

    parser.enter_nesting();

    let expr = parse_pipe_expr(parser, |p| {
        p.cursor.at_symbol(Symbol::Comma) || p.cursor.at_symbol(Symbol::RParen)
    });

    if parser.cursor.at_symbol(Symbol::Comma) {
        let comma_span = parser.cursor.current_span();
        parser.error(
            DiagnosticCode::TopLevelComma,
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
    parser.leave_nesting();
    Some(AtomAst {
        kind: AtomKind::Group(Box::new(expr)),
        span,
    })
}
