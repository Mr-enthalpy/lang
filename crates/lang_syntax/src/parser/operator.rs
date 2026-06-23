use crate::{
    token::operator_spelling_in_expr_context, DiagnosticCode, NavComponentAst, OperatorExprAst,
    OperatorExprKind, OperatorFixity, OperatorNameAst, OperatorSpelling, Span, Symbol, TokenKind,
};

use super::{
    atom::{parse_atom, parse_member_selector, parse_nav_outer_component, selector_span},
    closure::token_index_starts_head_clause,
    cursor::ParenClassification,
    form::Parser,
    product::{parse_bracket_product_expr, parse_product_expr},
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
        if is_operator_expr_boundary(parser, stop) {
            break;
        }

        let Some(current) = current_operator(parser) else {
            break;
        };

        if matches!(
            parser.cursor.peek_next_non_trivia().kind,
            TokenKind::Symbol(Symbol::ColonColon)
        ) {
            break;
        }

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
                return Some(parse_operator_nav_path(parser, current));
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

fn parse_operand(
    parser: &mut Parser<'_>,
    stop: &mut impl FnMut(&mut Parser<'_>) -> bool,
) -> Option<OperatorExprAst> {
    if is_operator_expr_boundary(parser, stop) {
        return None;
    }

    if parser.cursor.at_symbol(Symbol::LParen) {
        let (class, after_idx) = parser.cursor.classify_paren_at_segment_position();
        if matches!(class, ParenClassification::Product) {
            if let Some(idx) = after_idx {
                let (_, after) = parser.cursor.peek_at_skip_trivia(idx);
                if matches!(
                    after.kind,
                    TokenKind::Symbol(Symbol::FatArrow | Symbol::LBrace)
                ) || token_index_starts_head_clause(parser, idx)
                {
                    let atom = parse_atom(parser)?;
                    return Some(OperatorExprAst {
                        span: atom.span,
                        kind: OperatorExprKind::Atom(atom),
                    });
                }
            }
            let product = parse_product_expr(parser);
            return Some(OperatorExprAst {
                span: product.span,
                kind: OperatorExprKind::Product(product),
            });
        }
    }

    let atom = parse_atom(parser)?;
    Some(OperatorExprAst {
        span: atom.span,
        kind: OperatorExprKind::Atom(atom),
    })
}

fn parse_postfix_expr(
    parser: &mut Parser<'_>,
    stop: &mut impl FnMut(&mut Parser<'_>) -> bool,
) -> Option<OperatorExprAst> {
    let mut expr = parse_operand(parser, stop)?;

    loop {
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
            parser.cursor.bump_non_trivia();
            if let Some(component) = parse_nav_outer_component(parser) {
                expr = extend_operator_nav_path(expr, component);
            } else {
                let span = parser.cursor.current_span();
                parser.error(
                    DiagnosticCode::ExpectedName,
                    "expected navigation component after `::`",
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
                    let args = parse_product_expr(parser);
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
                        DiagnosticCode::ExpectedProductAfterDoubleDotName,
                        "expected product after `.. Selector`",
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
        } else if parser.cursor.at_symbol(Symbol::LBracket) {
            let args = parse_bracket_product_expr(parser);
            let operator = OperatorNameAst {
                spelling: OperatorSpelling::BracketCall.as_source_text().to_string(),
                span: args.span,
            };
            let span = expr.span.join(args.span);
            expr = OperatorExprAst {
                kind: OperatorExprKind::BracketCallSugar {
                    object: Box::new(expr),
                    operator,
                    args,
                    span,
                },
                span,
            };
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

fn parse_operator_nav_path(parser: &mut Parser<'_>, current: CurrentOperator) -> OperatorExprAst {
    let operator = bump_operator(parser, current);
    let mut components = vec![NavComponentAst::Operator(OperatorNameAst {
        spelling: operator.spelling.clone(),
        span: operator.span,
    })];
    let mut span = operator.span;

    while parser.cursor.consume_symbol(Symbol::ColonColon).is_some() {
        if let Some(component) = parse_nav_outer_component(parser) {
            let component_span = nav_component_span(&component);
            span = span.join(component_span);
            components.push(component);
        } else {
            let error_span = parser.cursor.current_span();
            parser.error(
                DiagnosticCode::ExpectedName,
                "expected navigation component after `::`",
                error_span,
            );
            span = span.join(error_span);
            break;
        }
    }

    OperatorExprAst {
        kind: OperatorExprKind::NavPath { components, span },
        span,
    }
}

fn nav_component_span(component: &NavComponentAst) -> Span {
    match component {
        NavComponentAst::Text(name) => name.span,
        NavComponentAst::Numeric(num) => num.span,
        NavComponentAst::Operator(operator) => operator.span,
        NavComponentAst::Group(expr) => expr.span,
        NavComponentAst::Error(error) => error.span,
    }
}

fn extend_operator_nav_path(expr: OperatorExprAst, component: NavComponentAst) -> OperatorExprAst {
    let component_span = nav_component_span(&component);
    match expr.kind {
        OperatorExprKind::NavPath {
            mut components,
            span,
        } => {
            let span = span.join(component_span);
            components.push(component);
            OperatorExprAst {
                kind: OperatorExprKind::NavPath { components, span },
                span,
            }
        }
        _ => {
            let span = expr.span.join(component_span);
            OperatorExprAst {
                kind: OperatorExprKind::NavPath {
                    components: vec![
                        NavComponentAst::Error(crate::ErrorAst {
                            message: "invalid navigation component".to_string(),
                            span: expr.span,
                        }),
                        component,
                    ],
                    span,
                },
                span,
            }
        }
    }
}

fn is_operator_expr_boundary(
    parser: &mut Parser<'_>,
    stop: &mut impl FnMut(&mut Parser<'_>) -> bool,
) -> bool {
    let saved_index = parser.cursor.current_index();
    if stop(parser) {
        return true;
    }
    parser.cursor.set_index(saved_index);

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
            precedence: 10,
            associativity: Associativity::Left,
        },
        Plus | Minus => BinaryInfo {
            precedence: 9,
            associativity: Associativity::Left,
        },
        LessLess | GreaterGreater => BinaryInfo {
            precedence: 8,
            associativity: Associativity::Left,
        },
        Less | LessEqual | Greater | GreaterEqual => BinaryInfo {
            precedence: 7,
            associativity: Associativity::NonAssociative,
        },
        EqualEqual | BangEqual => BinaryInfo {
            precedence: 6,
            associativity: Associativity::NonAssociative,
        },
        Amp => BinaryInfo {
            precedence: 5,
            associativity: Associativity::Left,
        },
        Pipe => BinaryInfo {
            precedence: 4,
            associativity: Associativity::Left,
        },
        AmpAmp => BinaryInfo {
            precedence: 3,
            associativity: Associativity::Left,
        },
        PipePipe => BinaryInfo {
            precedence: 2,
            associativity: Associativity::Left,
        },
        PlusEqual | MinusEqual | StarEqual | SlashEqual | AmpEqual | PipeEqual | LessLessEqual
        | GreaterGreaterEqual => BinaryInfo {
            precedence: 1,
            associativity: Associativity::NonAssociative,
        },
        Bang | At | Tilde | Caret | Dollar | PlusPlus | MinusMinus | Question | BracketCall => {
            return None;
        }
    };
    Some(info)
}

fn is_postfix_operator(spelling: OperatorSpelling) -> bool {
    matches!(
        spelling,
        OperatorSpelling::Bang
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
