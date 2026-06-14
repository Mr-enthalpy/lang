use crate::{
    token::operator_spelling_in_expr_context, DiagnosticCode, OperatorExprAst, OperatorExprKind,
    OperatorFixity, OperatorNameAst, OperatorSpelling, SelectorAst, Span, Symbol, TokenKind,
};

use super::{
    argpack::parse_argpack,
    atom::{
        parse_atom, parse_member_selector, parse_path_selector, selector_is_operator, selector_span,
    },
    form::Parser,
};

pub fn parse_operator_expr(
    parser: &mut Parser<'_>,
    stop: &mut impl FnMut(&mut Parser<'_>) -> bool,
) -> Option<OperatorExprAst> {
    parse_binary_expr(parser, stop, 1)
}

fn parse_binary_expr(
    parser: &mut Parser<'_>,
    stop: &mut impl FnMut(&mut Parser<'_>) -> bool,
    min_precedence: u8,
) -> Option<OperatorExprAst> {
    let mut lhs = parse_prefix_expr(parser, stop)?;
    let mut seen_nonassoc = false;

    loop {
        if parser.can_promote_newline_after_segment_element() {
            break;
        }

        if is_operator_expr_boundary(parser, stop) {
            break;
        }

        let Some(current) = current_operator(parser) else {
            break;
        };

        let Some(binary) = binary_info(current.spelling) else {
            break;
        };

        if binary.precedence < min_precedence {
            break;
        }

        let operator = bump_operator(parser, current);

        if binary.associativity == Associativity::NonAssociative {
            if seen_nonassoc {
                parser.error(
                    DiagnosticCode::ChainedNonAssociativeOperator,
                    "chained non-associative operator requires explicit grouping",
                    operator.span,
                );
            }
            seen_nonassoc = true;
        }

        let rhs = match parse_binary_expr(parser, stop, binary.precedence + 1) {
            Some(rhs) => rhs,
            None => {
                parser.error(
                    DiagnosticCode::InvalidOperatorExpression,
                    format!(
                        "expected right-hand side after operator `{}`",
                        operator.spelling
                    ),
                    operator.span,
                );
                error_operator_expr(parser, "expected operator right-hand side", operator.span)
            }
        };

        let span = lhs.span.join(rhs.span);
        lhs = OperatorExprAst {
            kind: OperatorExprKind::OperatorSugar {
                operator,
                fixity: OperatorFixity::Binary,
                args: vec![lhs, rhs],
                span,
            },
            span,
        };
    }

    Some(lhs)
}

fn parse_prefix_expr(
    parser: &mut Parser<'_>,
    stop: &mut impl FnMut(&mut Parser<'_>) -> bool,
) -> Option<OperatorExprAst> {
    if is_operator_expr_boundary(parser, stop) {
        return None;
    }

    if let Some(current) = current_operator(parser) {
        if current.spelling == OperatorSpelling::Less {
            if let Some(expr) = parse_postfix_expr(parser, stop) {
                return Some(expr);
            }
        }

        if current.spelling == OperatorSpelling::Minus {
            let operator = bump_operator(parser, current);
            let arg = match parse_prefix_expr(parser, stop) {
                Some(arg) => arg,
                None => {
                    parser.error(
                        DiagnosticCode::InvalidOperatorExpression,
                        "expected expression after prefix operator `-`",
                        operator.span,
                    );
                    error_operator_expr(parser, "expected prefix operator argument", operator.span)
                }
            };
            let span = operator.span.join(arg.span);
            return Some(OperatorExprAst {
                kind: OperatorExprKind::OperatorSugar {
                    operator,
                    fixity: OperatorFixity::Prefix,
                    args: vec![arg],
                    span,
                },
                span,
            });
        }

        if is_operator_token_at_expr_start(current.spelling) {
            if matches!(
                parser.cursor.peek_next_non_trivia().kind,
                TokenKind::Symbol(Symbol::ColonColon)
            ) {
                let operator = bump_operator(parser, current);
                parser.error(
                    DiagnosticCode::OperatorPathLeafNotFinal,
                    "operator path leaf must follow a path head and be final",
                    operator.span,
                );
                parser.cursor.consume_symbol(Symbol::ColonColon);
                let _ = parse_path_selector(parser);
                return Some(error_operator_expr(
                    parser,
                    "operator path leaf cannot start a path",
                    operator.span,
                ));
            }

            let operator = bump_operator(parser, current);
            parser.error(
                DiagnosticCode::InvalidOperatorExpression,
                format!("unsupported prefix operator `{}`", operator.spelling),
                operator.span,
            );
            return Some(error_operator_expr(
                parser,
                "unsupported prefix operator",
                operator.span,
            ));
        }
    }

    parse_postfix_expr(parser, stop)
}

fn parse_postfix_expr(
    parser: &mut Parser<'_>,
    stop: &mut impl FnMut(&mut Parser<'_>) -> bool,
) -> Option<OperatorExprAst> {
    let atom = parse_atom(parser)?;
    let mut expr = OperatorExprAst {
        span: atom.span,
        kind: OperatorExprKind::Atom(atom),
    };

    loop {
        if parser.can_promote_newline_after_segment_element() {
            break;
        }

        if is_operator_expr_boundary(parser, stop) {
            break;
        }

        if let Some(current) = current_operator(parser) {
            if is_postfix_operator(current.spelling) {
                let operator = bump_operator(parser, current);
                let span = expr.span.join(operator.span);
                expr = OperatorExprAst {
                    kind: OperatorExprKind::OperatorSugar {
                        operator,
                        fixity: OperatorFixity::Postfix,
                        args: vec![expr],
                        span,
                    },
                    span,
                };
                continue;
            }
        }

        if matches!(expr.kind, OperatorExprKind::Atom(_)) {
            break;
        }

        if parser.cursor.at_symbol(Symbol::ColonColon) {
            let coloncolon = parser.cursor.bump_non_trivia();
            if operator_path_ends_with_operator_leaf(&expr) {
                parser.error(
                    DiagnosticCode::OperatorPathLeafNotFinal,
                    "operator path leaf cannot be followed by `::`",
                    coloncolon.span,
                );
                let _ = parse_path_selector(parser);
                break;
            }
            if let Some(selector) = parse_path_selector(parser) {
                expr = extend_or_create_operator_path(expr, selector);
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
            if let Some(selector) = parse_member_selector(parser) {
                let span = expr.span.join(selector_span(&selector));
                expr = OperatorExprAst {
                    kind: OperatorExprKind::MemberSugar {
                        object: Box::new(expr),
                        selector,
                        span,
                    },
                    span,
                };
            } else {
                consume_invalid_operator_selector(parser);
                parser.error(
                    DiagnosticCode::ExpectedNameAfterDot,
                    "expected name after `.`",
                    dot_token.span,
                );
                break;
            }
        } else if parser.cursor.at_symbol(Symbol::DotDot) {
            let dotdot_token = parser.cursor.bump_non_trivia();
            if let Some(selector) = parse_member_selector(parser) {
                if parser.cursor.at_symbol(Symbol::LParen) {
                    let args = parse_argpack(parser);
                    let span = expr.span.join(args.span);
                    expr = OperatorExprAst {
                        kind: OperatorExprKind::DoubleDotSugar {
                            object: Box::new(expr),
                            selector,
                            args,
                            span,
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
                consume_invalid_operator_selector(parser);
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

    Some(expr)
}

fn consume_invalid_operator_selector(parser: &mut Parser<'_>) {
    if parser.cursor.peek_non_trivia().kind.is_operator_spelling() {
        parser.cursor.bump_non_trivia();
    }
}

fn extend_or_create_operator_path(expr: OperatorExprAst, selector: SelectorAst) -> OperatorExprAst {
    let selector_end = selector_span(&selector);
    match expr.kind {
        OperatorExprKind::Path {
            base,
            mut names,
            span,
        } => {
            let span = span.join(selector_end);
            names.push(selector);
            OperatorExprAst {
                kind: OperatorExprKind::Path { base, names, span },
                span,
            }
        }
        _ => {
            let span = expr.span.join(selector_end);
            OperatorExprAst {
                kind: OperatorExprKind::Path {
                    base: Box::new(expr),
                    names: vec![selector],
                    span,
                },
                span,
            }
        }
    }
}

fn operator_path_ends_with_operator_leaf(expr: &OperatorExprAst) -> bool {
    match &expr.kind {
        OperatorExprKind::Path { names, .. } => names.last().is_some_and(selector_is_operator),
        _ => false,
    }
}

fn is_operator_expr_boundary(
    parser: &mut Parser<'_>,
    stop: &mut impl FnMut(&mut Parser<'_>) -> bool,
) -> bool {
    let saved_index = parser.cursor.current_index();
    if !parser.at_top_level_newline() {
        if stop(parser) {
            return true;
        }
        parser.cursor.set_index(saved_index);
    }

    let (_, token) = parser
        .cursor
        .peek_at_skip_trivia(parser.cursor.current_index());

    matches!(
        token.kind,
        TokenKind::Eof
            | TokenKind::Symbol(
                Symbol::RParen
                    | Symbol::RBracket
                    | Symbol::RBrace
                    | Symbol::Comma
                    | Symbol::Semicolon
                    | Symbol::PipeGreater
                    | Symbol::FatArrow
                    | Symbol::ThinArrow
                    | Symbol::Equal
                    | Symbol::Colon
                    | Symbol::TripleEqual
            )
    )
}

#[derive(Clone, Copy)]
struct CurrentOperator {
    spelling: OperatorSpelling,
    span: Span,
}

fn current_operator(parser: &mut Parser<'_>) -> Option<CurrentOperator> {
    let token = parser.cursor.peek_non_trivia();
    let spelling = operator_spelling_in_expr_context(&token.kind)?;
    Some(CurrentOperator {
        spelling,
        span: token.span,
    })
}

fn bump_operator(parser: &mut Parser<'_>, current: CurrentOperator) -> OperatorNameAst {
    parser.cursor.bump_non_trivia();
    OperatorNameAst {
        spelling: current.spelling.as_source_text().to_string(),
        span: current.span,
    }
}

fn error_operator_expr(parser: &Parser<'_>, message: &str, span: Span) -> OperatorExprAst {
    OperatorExprAst {
        kind: OperatorExprKind::Error(parser.error_ast(message, span)),
        span,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Associativity {
    Left,
    NonAssociative,
}

#[derive(Clone, Copy)]
struct BinaryInfo {
    precedence: u8,
    associativity: Associativity,
}

fn binary_info(spelling: OperatorSpelling) -> Option<BinaryInfo> {
    use OperatorSpelling::*;
    let info = match spelling {
        Star | Slash => BinaryInfo {
            precedence: 6,
            associativity: Associativity::Left,
        },
        Plus | Minus => BinaryInfo {
            precedence: 5,
            associativity: Associativity::Left,
        },
        LessLess | GreaterGreater => BinaryInfo {
            precedence: 4,
            associativity: Associativity::Left,
        },
        Less | LessEqual | Greater | GreaterEqual => BinaryInfo {
            precedence: 3,
            associativity: Associativity::NonAssociative,
        },
        EqualEqual | BangEqual => BinaryInfo {
            precedence: 2,
            associativity: Associativity::NonAssociative,
        },
        PlusEqual | MinusEqual | StarEqual | SlashEqual | LessLessEqual | GreaterGreaterEqual => {
            BinaryInfo {
                precedence: 1,
                associativity: Associativity::NonAssociative,
            }
        }
        Bang | Amp | At | Tilde | Caret | Dollar | PlusPlus | MinusMinus | Question => {
            return None;
        }
    };
    Some(info)
}

fn is_postfix_operator(spelling: OperatorSpelling) -> bool {
    matches!(
        spelling,
        OperatorSpelling::Bang
            | OperatorSpelling::Amp
            | OperatorSpelling::At
            | OperatorSpelling::Tilde
            | OperatorSpelling::Caret
            | OperatorSpelling::Dollar
            | OperatorSpelling::PlusPlus
            | OperatorSpelling::MinusMinus
            | OperatorSpelling::Question
    )
}

fn is_operator_token_at_expr_start(_spelling: OperatorSpelling) -> bool {
    true
}
