use crate::{
    AtomAst, AtomKind, DiagnosticCode, ErrorAst, NameAst, NavComponentAst, OperatorNameAst,
    OperatorSpelling, SelectorAst, Span, Symbol, TokenKind,
};

use super::{
    closure::try_parse_closure,
    form::Parser,
    let_stmt::looks_like_alias_binding_start,
    pipe::parse_pipe_expr,
    product::{parse_bracket_product_expr, parse_product_expr},
};

pub fn parse_atom(parser: &mut Parser<'_>) -> Option<AtomAst> {
    if parser.is_form_boundary() {
        return None;
    }
    let mut atom = parse_atom_base(parser)?;

    loop {
        if parser.is_form_boundary() {
            break;
        }
        if parser.cursor.at_symbol(Symbol::ColonColon) {
            parser.cursor.bump_non_trivia();
            if let Some(component) = parse_nav_outer_component(parser) {
                atom = extend_or_create_nav_path(parser, atom, component);
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
                let span = atom.span.join(selector_span(&selector));
                atom = AtomAst {
                    kind: AtomKind::MemberSugar {
                        object: Box::new(atom),
                        selector,
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
                    let product = parse_product_expr(parser);
                    let span = atom.span.join(product.span);
                    atom = AtomAst {
                        kind: AtomKind::DoubleDotSugar {
                            object: Box::new(atom),
                            selector,
                            args: product,
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
            let span = atom.span.join(args.span);
            atom = AtomAst {
                kind: AtomKind::BracketCallSugar {
                    object: Box::new(atom),
                    operator,
                    args,
                },
                span,
            };
        } else {
            break;
        }
    }

    Some(atom)
}

fn parse_atom_base(parser: &mut Parser<'_>) -> Option<AtomAst> {
    // Try closure first (handles `{` and FnHeadPrefix lookahead)
    if let Some(closure_atom) = try_parse_closure(parser) {
        return Some(closure_atom);
    }

    let token = parser.cursor.peek_non_trivia();

    match &token.kind {
        TokenKind::Name => {
            let name_text = token.text.clone();
            let name_span = token.span;

            if name_text == "let" {
                let saved = parser.cursor.current_index();
                parser.cursor.bump_non_trivia();
                let is_alias = looks_like_alias_binding_start(parser);
                if is_alias {
                    parser.error(
                        DiagnosticCode::InvalidAliasPosition,
                        "alias binding must appear as a standalone form",
                        name_span,
                    );
                    parser.recover_to_form_boundary();
                    return Some(AtomAst {
                        kind: AtomKind::Error(parser.error_ast(
                            "alias binding must appear as a standalone form",
                            name_span,
                        )),
                        span: name_span,
                    });
                }
                parser.cursor.set_index(saved);
            }

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
        TokenKind::FloatLiteral => {
            let token = parser.cursor.bump_non_trivia();
            Some(AtomAst {
                kind: AtomKind::FloatLiteral(token.text.clone()),
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

fn consume_invalid_operator_selector(parser: &mut Parser<'_>) {
    if parser.cursor.peek_non_trivia().kind.is_operator_spelling() {
        parser.cursor.bump_non_trivia();
    }
}

pub fn parse_member_selector(parser: &mut Parser<'_>) -> Option<SelectorAst> {
    let token = parser.cursor.peek_non_trivia();
    match token.kind {
        TokenKind::Name => {
            let token = parser.cursor.bump_non_trivia();
            Some(SelectorAst::Text(NameAst {
                text: token.text.clone(),
                span: token.span,
            }))
        }
        _ => None,
    }
}

pub fn parse_nav_outer_component(parser: &mut Parser<'_>) -> Option<NavComponentAst> {
    let token = parser.cursor.peek_non_trivia();
    match token.kind {
        TokenKind::Name => {
            let token = parser.cursor.bump_non_trivia();
            Some(NavComponentAst::Text(NameAst {
                text: token.text.clone(),
                span: token.span,
            }))
        }
        TokenKind::Symbol(Symbol::LParen) => parse_nav_group_component(parser),
        _ if token.kind.is_operator_spelling() => {
            let operator = parser.cursor.bump_non_trivia();
            parser.error(
                DiagnosticCode::InvalidNavComponent,
                "operator cannot be an outer navigation component",
                operator.span,
            );
            Some(NavComponentAst::Error(parser.error_ast(
                "operator cannot be an outer navigation component",
                operator.span,
            )))
        }
        _ => None,
    }
}

pub fn parse_nav_group_component(parser: &mut Parser<'_>) -> Option<NavComponentAst> {
    parse_group(parser).map(|group| match group.kind {
        AtomKind::Group(expr) => NavComponentAst::Group(expr),
        AtomKind::Error(error) => NavComponentAst::Error(error),
        _ => NavComponentAst::Error(ErrorAst {
            message: "invalid navigation group".to_string(),
            span: group.span,
        }),
    })
}

pub fn selector_span(selector: &SelectorAst) -> Span {
    match selector {
        SelectorAst::Text(name) => name.span,
    }
}

fn nav_component_span(component: &NavComponentAst) -> Span {
    match component {
        NavComponentAst::Text(name) => name.span,
        NavComponentAst::Operator(operator) => operator.span,
        NavComponentAst::Group(expr) => expr.span,
        NavComponentAst::Error(error) => error.span,
    }
}

pub fn nav_component_is_operator(component: &NavComponentAst) -> bool {
    matches!(component, NavComponentAst::Operator(_))
}

fn atom_to_nav_component(parser: &mut Parser<'_>, atom: AtomAst) -> NavComponentAst {
    match atom.kind {
        AtomKind::Name(name) => NavComponentAst::Text(name),
        AtomKind::Group(_) => {
            parser.error(
                DiagnosticCode::InvalidNavComponent,
                "grouped expression cannot be an innermost navigation component",
                atom.span,
            );
            NavComponentAst::Error(parser.error_ast(
                "grouped expression cannot be an innermost navigation component",
                atom.span,
            ))
        }
        AtomKind::Error(error) => NavComponentAst::Error(error),
        _ => NavComponentAst::Error(parser.error_ast("invalid navigation component", atom.span)),
    }
}

pub fn extend_nav_components(
    span: Span,
    mut components: Vec<NavComponentAst>,
    component: NavComponentAst,
) -> (Vec<NavComponentAst>, Span) {
    let component_span = nav_component_span(&component);
    components.push(component);
    (components, span.join(component_span))
}

fn extend_or_create_nav_path(
    parser: &mut Parser<'_>,
    atom: AtomAst,
    component: NavComponentAst,
) -> AtomAst {
    let component_span = nav_component_span(&component);
    match atom.kind {
        AtomKind::NavPath { mut components } => {
            let span = atom.span.join(component_span);
            components.push(component);
            AtomAst {
                kind: AtomKind::NavPath { components },
                span,
            }
        }
        _ => {
            let span = atom.span.join(component_span);
            AtomAst {
                kind: AtomKind::NavPath {
                    components: vec![atom_to_nav_component(parser, atom), component],
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
