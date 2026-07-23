use crate::{
    AtomAst, AtomKind, BodyBlockAst, CaptureClauseAst, CaptureItemAst, ClosureAst, ClosureBodyAst,
    ClosurePlacementAst, DeleteBodyAst, DiagnosticCode, ExprAst, FnHeadPrefixAst, FormAst,
    HeadClauseAst, NameAst, ParamClauseAst, ReturnClauseAst, Span, Symbol, TokenKind,
};

use super::{
    deduce::parse_deduce_list,
    expr::parse_expr_until,
    form::Parser,
    let_stmt::{parse_binding_slot, parse_product_extract, BindingSlotContext},
    policy::parse_policy_spec_until,
    product::error_expr,
};

// -- Body block --

pub fn parse_body_block(parser: &mut Parser<'_>) -> BodyBlockAst {
    let lbrace = parser
        .cursor
        .consume_symbol(Symbol::LBrace)
        .expect("parse_body_block at `{`");

    parser.enter_nesting();
    let mut forms = Vec::new();
    let mut seen_terminal = false;

    loop {
        if parser.cursor.at_eof() || parser.cursor.at_symbol(Symbol::RBrace) {
            break;
        }
        if parser.cursor.consume_symbol(Symbol::Semicolon).is_some() {
            continue;
        }
        if seen_terminal {
            let form = parser.parse_form();
            let span = form_span(&form);
            parser.error(
                DiagnosticCode::StatementAfterTerminalBlockForm,
                "statement after terminal block form",
                span,
            );
            forms.push(FormAst::Error(
                parser.error_ast("statement after terminal block form", span),
            ));
            continue;
        }
        let form = parser.parse_form();
        if matches!(&form, FormAst::ReturnEvent(_) | FormAst::Expr(_)) {
            seen_terminal = true;
        }
        forms.push(form);
    }

    let end = if let Some(rbrace) = parser.cursor.consume_symbol(Symbol::RBrace) {
        rbrace.span
    } else {
        let span = parser.cursor.current_span();
        parser.error(
            DiagnosticCode::UnclosedBrace,
            "unclosed body block, expected `}`",
            lbrace.span,
        );
        span
    };

    parser.leave_nesting();
    BodyBlockAst {
        forms,
        span: lbrace.span.join(end),
    }
}

fn form_span(form: &FormAst) -> Span {
    match form {
        FormAst::Let(l) => l.span,
        FormAst::AliasLet(a) => a.span,
        FormAst::Expr(e) => e.span,
        FormAst::ReturnEvent(r) => r.span,
        FormAst::Error(e) => e.span,
    }
}

// -- Closure entry from atom parser --

pub fn try_parse_closure(parser: &mut Parser<'_>) -> Option<AtomAst> {
    if parser.cursor.at_symbol(Symbol::LBrace) {
        let body = parse_body_block(parser);
        let span = body.span;
        return Some(closure_atom(
            ClosurePlacementAst::InPlace,
            None,
            ClosureBodyAst::Block(body),
            span,
        ));
    }

    let saved = parser.cursor.current_index();
    parser.gate_diagnostics();
    let head = match parse_fn_head_prefix(parser) {
        Some(h) => h,
        None => {
            parser.cursor.set_index(saved);
            parser.ungate_drop_diagnostics();
            return None;
        }
    };

    if parser.cursor.consume_symbol(Symbol::FatArrow).is_some() {
        parser.ungate_keep_diagnostics();
        if parser.cursor.at_symbol(Symbol::LBrace) {
            let block = parse_body_block(parser);
            let span = head.span.join(block.span);
            let body = ClosureBodyAst::Block(block);
            return Some(closure_atom(
                ClosurePlacementAst::Ordinary,
                Some(head),
                body,
                span,
            ));
        }
        if parser.cursor.at_name("delete") {
            let delete_body = parse_bare_delete_body(parser);
            let span = head.span.join(delete_body.span);
            return Some(closure_atom(
                ClosurePlacementAst::Ordinary,
                Some(head),
                ClosureBodyAst::Delete(delete_body),
                span,
            ));
        }
        if parser.cursor.at_name("default") {
            let token = parser.cursor.bump_non_trivia();
            let span = head.span.join(token.span);
            return Some(closure_atom(
                ClosurePlacementAst::Ordinary,
                Some(head),
                ClosureBodyAst::Defaulted {
                    default_name: NameAst {
                        text: token.text.clone(),
                        span: token.span,
                    },
                    span: token.span,
                },
                span,
            ));
        }
        if parser.cursor.at_symbol(Symbol::LParen) {
            match parse_delete_body(parser) {
                Some(delete_body) => {
                    let span = head.span.join(delete_body.span);
                    return Some(closure_atom(
                        ClosurePlacementAst::Ordinary,
                        Some(head),
                        ClosureBodyAst::Delete(delete_body),
                        span,
                    ));
                }
                None => {
                    parser.recover_to_form_boundary();
                    let error_end = parser.cursor.current_span();
                    parser.error(
                        DiagnosticCode::InvalidClosureHead,
                        "expected `)` then `delete` after `=>`",
                        error_end,
                    );
                    let span = head.span.join(error_end);
                    return Some(AtomAst {
                        kind: AtomKind::Error(
                            parser.error_ast("invalid parenthesized callable tail", span),
                        ),
                        span,
                    });
                }
            }
        }
        if matches!(parser.cursor.peek_non_trivia().kind, TokenKind::Name) {
            let token = parser.cursor.bump_non_trivia();
            let strategy = NameAst {
                text: token.text.clone(),
                span: token.span,
            };
            if parser.cursor.at_symbol(Symbol::LBrace) {
                let block = parse_body_block(parser);
                let body_span = strategy.span.join(block.span);
                let span = head.span.join(block.span);
                return Some(closure_atom(
                    ClosurePlacementAst::Ordinary,
                    Some(head),
                    ClosureBodyAst::NamedBlock {
                        strategy,
                        block,
                        span: body_span,
                    },
                    span,
                ));
            }
            parser.error(
                DiagnosticCode::InvalidClosureHead,
                "expected `{` after named overload strategy in callable tail",
                strategy.span,
            );
            parser.recover_to_form_boundary();
            let error_span = head.span.join(strategy.span);
            return Some(AtomAst {
                kind: AtomKind::Error(parser.error_ast(
                    "invalid named-strategy callable tail without body",
                    error_span,
                )),
                span: error_span,
            });
        }
        parser.recover_to_form_boundary();
        let body_start = parser.cursor.current_span();
        parser.error(
            DiagnosticCode::InvalidClosureHead,
            "expected `{`, `delete`, `(string) delete`, `default`, or `strategy { ... }` after `=>`",
            body_start,
        );
        let span = head.span.join(body_start);
        return Some(AtomAst {
            kind: AtomKind::Error(parser.error_ast("invalid callable implementation tail", span)),
            span,
        });
    }

    if at_overload_strategy_annotation(parser) {
        parser.ungate_keep_diagnostics();
        let Some(strategy) = parse_overload_strategy_annotation(parser) else {
            let end = if parser.cursor.at_symbol(Symbol::LBrace) {
                parse_body_block(parser).span
            } else {
                parser.recover_to_form_boundary();
                parser.cursor.current_span()
            };
            let error_span = head.span.join(end);
            return Some(AtomAst {
                kind: AtomKind::Error(
                    parser.error_ast("invalid `[[strategy]]` callable tail", error_span),
                ),
                span: error_span,
            });
        };
        if !parser.cursor.at_symbol(Symbol::LBrace) {
            let span = parser.cursor.current_span();
            parser.error(
                DiagnosticCode::InvalidClosureHead,
                "expected `{` after `[[strategy]]` callable tail annotation",
                span,
            );
            parser.recover_to_form_boundary();
            let error_span = head.span.join(span);
            return Some(AtomAst {
                kind: AtomKind::Error(parser.error_ast(
                    "invalid `[[strategy]]` callable tail without body",
                    error_span,
                )),
                span: error_span,
            });
        }
        let body = parse_body_block(parser);
        let span = head.span.join(body.span);
        if let Some(error) = reject_in_place_capture(parser, &head, span) {
            return Some(error);
        }
        return Some(closure_atom(
            ClosurePlacementAst::InPlace,
            Some(head),
            ClosureBodyAst::NamedBlock {
                strategy,
                block: body,
                span,
            },
            span,
        ));
    }

    if parser.cursor.at_symbol(Symbol::LBrace) {
        parser.ungate_keep_diagnostics();
        let body = parse_body_block(parser);
        let span = head.span.join(body.span);
        if let Some(error) = reject_in_place_capture(parser, &head, span) {
            return Some(error);
        }
        return Some(closure_atom(
            ClosurePlacementAst::InPlace,
            Some(head),
            ClosureBodyAst::Block(body),
            span,
        ));
    }

    parser.cursor.set_index(saved);
    parser.ungate_drop_diagnostics();
    None
}

fn closure_atom(
    placement: ClosurePlacementAst,
    head: Option<FnHeadPrefixAst>,
    body: ClosureBodyAst,
    span: Span,
) -> AtomAst {
    AtomAst {
        kind: AtomKind::Closure(ClosureAst {
            placement,
            head,
            body,
            span,
        }),
        span,
    }
}

fn reject_in_place_capture(
    parser: &mut Parser<'_>,
    head: &FnHeadPrefixAst,
    closure_span: Span,
) -> Option<AtomAst> {
    let capture = head.captures.as_ref()?;
    parser.error(
        DiagnosticCode::InvalidClosureHead,
        "in-place closure cannot have a capture list; add `=>` for an ordinary closure",
        capture.span,
    );
    Some(AtomAst {
        kind: AtomKind::Error(parser.error_ast(
            "capture list is not allowed on an in-place closure",
            closure_span,
        )),
        span: closure_span,
    })
}

// -- Delete body --

/// Parse `(string_literal) delete` after `=>`.
///
/// Caller has already verified that the cursor is at `(`.
/// Returns `None` if the grouped expression or the `delete` name
/// cannot be parsed (diagnostics are emitted by the caller).
fn parse_delete_body(parser: &mut Parser<'_>) -> Option<DeleteBodyAst> {
    let lparen = parser.cursor.consume_symbol(Symbol::LParen)?;

    let message_token = parser.cursor.peek_non_trivia();
    if !matches!(message_token.kind, TokenKind::StringLiteral) {
        parser.error(
            DiagnosticCode::InvalidClosureHead,
            "delete message must be a string literal",
            message_token.span,
        );
        return None;
    }
    let message = parser.cursor.bump_non_trivia().text.clone();

    let _rparen = match parser.cursor.consume_symbol(Symbol::RParen) {
        Some(tok) => tok.span,
        None => {
            let span = parser.cursor.current_span();
            parser.error(
                DiagnosticCode::InvalidClosureHead,
                "expected `)` after delete message",
                span,
            );
            return None;
        }
    };

    let delete_token = match parser.cursor.consume_name("delete") {
        Some(tok) => tok,
        None => {
            let span = parser.cursor.current_span();
            parser.error(
                DiagnosticCode::InvalidClosureHead,
                "expected `delete` after `)` in `=> (...message...) delete`",
                span,
            );
            return None;
        }
    };

    let span = lparen.span.join(delete_token.span);
    Some(DeleteBodyAst {
        message: Some(message),
        delete_name: NameAst {
            text: "delete".to_string(),
            span: delete_token.span,
        },
        span,
    })
}

fn parse_bare_delete_body(parser: &mut Parser<'_>) -> DeleteBodyAst {
    let token = parser
        .cursor
        .consume_name("delete")
        .expect("parse_bare_delete_body at `delete`");
    DeleteBodyAst {
        message: None,
        delete_name: NameAst {
            text: token.text.clone(),
            span: token.span,
        },
        span: token.span,
    }
}

pub(super) fn at_overload_strategy_annotation(parser: &Parser<'_>) -> bool {
    token_index_starts_overload_strategy_annotation(parser, parser.cursor.current_index())
}

pub(super) fn token_index_starts_overload_strategy_annotation(
    parser: &Parser<'_>,
    from: usize,
) -> bool {
    let (first_index, first) = parser.cursor.peek_at_skip_trivia(from);
    if !matches!(first.kind, TokenKind::Symbol(Symbol::LBracket)) {
        return false;
    }
    let (_, second) = parser.cursor.peek_at_skip_trivia(first_index + 1);
    matches!(second.kind, TokenKind::Symbol(Symbol::LBracket))
}

/// The single strong-context lookahead used when a parenthesized form could be
/// either an ordinary Product or the parameter clause of a closure head.
///
/// Keep every continuation that proves `FnHeadPrefix` here. Callers in the
/// segment and operator parsers must not maintain their own approximations.
pub(super) fn token_index_starts_closure_head_continuation(
    parser: &Parser<'_>,
    from: usize,
) -> bool {
    let (_, token) = parser.cursor.peek_at_skip_trivia(from);
    matches!(
        token.kind,
        TokenKind::Symbol(Symbol::Colon | Symbol::ThinArrow | Symbol::FatArrow | Symbol::LBrace)
    ) || token_index_starts_head_clause(parser, from)
        || token_index_starts_overload_strategy_annotation(parser, from)
}

pub(super) fn at_callable_implementation_tail(parser: &mut Parser<'_>) -> bool {
    parser.cursor.at_symbol(Symbol::FatArrow)
        || parser.cursor.at_symbol(Symbol::LBrace)
        || at_overload_strategy_annotation(parser)
}

fn parse_overload_strategy_annotation(parser: &mut Parser<'_>) -> Option<NameAst> {
    let first = parser.cursor.consume_symbol(Symbol::LBracket)?;
    parser.cursor.consume_symbol(Symbol::LBracket)?;
    let token = parser.cursor.peek_non_trivia();
    let name = if matches!(token.kind, TokenKind::Name) {
        let token = parser.cursor.bump_non_trivia();
        NameAst {
            text: token.text.clone(),
            span: token.span,
        }
    } else {
        parser.error(
            DiagnosticCode::ExpectedName,
            "expected overload strategy name inside `[[...]]`",
            token.span,
        );
        return None;
    };
    if parser.cursor.consume_symbol(Symbol::RBracket).is_none()
        || parser.cursor.consume_symbol(Symbol::RBracket).is_none()
    {
        parser.error(
            DiagnosticCode::UnclosedBracket,
            "expected closing `]]` after overload strategy name",
            first.span.join(name.span),
        );
        return None;
    }
    Some(name)
}

// -- FnHeadPrefix --

fn parse_fn_head_prefix(parser: &mut Parser<'_>) -> Option<FnHeadPrefixAst> {
    let start = parser.cursor.current_span();

    let deduce = if parser.cursor.at_symbol(Symbol::Less) {
        Some(parse_deduce_list(parser))
    } else {
        None
    };

    let captures =
        if parser.cursor.at_symbol(Symbol::LBracket) && !at_overload_strategy_annotation(parser) {
            Some(parse_capture_clause(parser))
        } else {
            None
        };

    let params = if parser.cursor.at_symbol(Symbol::LParen) {
        Some(parse_param_clause(parser, deduce.as_ref()))
    } else {
        None
    };

    let call_policy = if params.is_some() && parser.cursor.consume_symbol(Symbol::Colon).is_some() {
        if at_call_policy_boundary(parser) {
            let span = parser.cursor.current_span();
            parser.error(
                DiagnosticCode::InvalidClosureHead,
                "expected call-result policy after `:`",
                span,
            );
        }
        let policy = parse_policy_spec_until(parser, |p| {
            p.cursor.at_symbol(Symbol::ThinArrow)
                || at_callable_implementation_tail(p)
                || p.is_form_boundary()
        });
        Some(policy)
    } else {
        None
    };

    let returns = if parser.cursor.consume_symbol(Symbol::ThinArrow).is_some() {
        Some(parse_return_clause(parser))
    } else {
        None
    };

    let clauses = parse_head_clauses(parser);

    let end = parser.cursor.current_span();
    let span = start.join(end);

    if deduce.is_none() && captures.is_none() && params.is_none() && clauses.is_empty() {
        return None;
    }

    Some(FnHeadPrefixAst {
        deduce,
        captures,
        params,
        call_policy,
        returns,
        clauses,
        span,
    })
}

// -- Head clauses (require / pre / post / lifetime pre / lifetime post) --

#[derive(Clone, Copy)]
enum HeadClauseKind {
    Require,
    Pre,
    Post,
    LifetimePre,
    LifetimePost,
}

impl HeadClauseKind {
    fn keyword_text(self) -> &'static str {
        match self {
            HeadClauseKind::Require => "require",
            HeadClauseKind::Pre => "pre",
            HeadClauseKind::Post => "post",
            HeadClauseKind::LifetimePre => "lifetime pre",
            HeadClauseKind::LifetimePost => "lifetime post",
        }
    }

    fn into_clause(self, expr: ExprAst, span: Span) -> HeadClauseAst {
        match self {
            HeadClauseKind::Require => HeadClauseAst::Require { expr, span },
            HeadClauseKind::Pre => HeadClauseAst::Pre { expr, span },
            HeadClauseKind::Post => HeadClauseAst::Post { expr, span },
            HeadClauseKind::LifetimePre => HeadClauseAst::LifetimePre { expr, span },
            HeadClauseKind::LifetimePost => HeadClauseAst::LifetimePost { expr, span },
        }
    }
}

// A head clause keyword starts at `from` (skipping trivia) when the token is
// `require`/`pre`/`post`, or `lifetime` immediately followed by `pre`/`post`.
pub(super) fn token_index_starts_head_clause(parser: &Parser<'_>, from: usize) -> bool {
    let (first_index, first) = parser.cursor.peek_at_skip_trivia(from);
    if !matches!(first.kind, TokenKind::Name) {
        return false;
    }
    match first.text.as_str() {
        "require" | "pre" | "post" => true,
        "lifetime" => {
            let (_, second) = parser.cursor.peek_at_skip_trivia(first_index + 1);
            matches!(second.kind, TokenKind::Name)
                && (second.text == "pre" || second.text == "post")
        }
        _ => false,
    }
}

pub(super) fn at_head_clause_keyword(parser: &Parser<'_>) -> bool {
    token_index_starts_head_clause(parser, parser.cursor.current_index())
}

fn clause_expr_boundary(parser: &mut Parser<'_>) -> bool {
    at_callable_implementation_tail(parser)
        || parser.is_form_boundary()
        || at_head_clause_keyword(parser)
}

fn consume_head_clause_keyword(parser: &mut Parser<'_>) -> Option<(HeadClauseKind, Span)> {
    if !at_head_clause_keyword(parser) {
        return None;
    }
    let first = parser.cursor.bump_non_trivia();
    let start = first.span;
    let kind = match first.text.as_str() {
        "require" => HeadClauseKind::Require,
        "pre" => HeadClauseKind::Pre,
        "post" => HeadClauseKind::Post,
        "lifetime" => {
            let second = parser.cursor.bump_non_trivia();
            if second.text == "post" {
                return Some((HeadClauseKind::LifetimePost, start.join(second.span)));
            }
            return Some((HeadClauseKind::LifetimePre, start.join(second.span)));
        }
        _ => return None,
    };
    Some((kind, start))
}

fn parse_head_clauses(parser: &mut Parser<'_>) -> Vec<HeadClauseAst> {
    let mut clauses = Vec::new();

    while at_head_clause_keyword(parser) {
        let Some((kind, header_span)) = consume_head_clause_keyword(parser) else {
            break;
        };

        let expr = if clause_expr_boundary(parser) {
            let span = parser.cursor.current_span();
            parser.error(
                DiagnosticCode::InvalidClosureHead,
                format!("expected expression after `{}`", kind.keyword_text()),
                span,
            );
            error_expr(parser, "missing head clause expression", span)
        } else {
            let expr = parse_expr_until(parser, clause_expr_boundary);
            if super::form::expression_contains_name(&expr, "return") {
                parser.error(
                    DiagnosticCode::ReturnExpressionNotAllowed,
                    "return is only allowed as a block terminal form",
                    expr.span,
                );
            }
            expr
        };

        let span = header_span.join(expr.span);
        clauses.push(kind.into_clause(expr, span));
    }

    clauses
}

// -- Capture clause --

fn parse_capture_clause(parser: &mut Parser<'_>) -> CaptureClauseAst {
    let lbracket = parser
        .cursor
        .consume_symbol(Symbol::LBracket)
        .expect("parse_capture_clause at `[`");

    parser.enter_nesting();
    let mut items = Vec::new();

    loop {
        if parser.cursor.at_eof()
            || parser.cursor.at_symbol(Symbol::RBracket)
            || parser.is_form_boundary()
        {
            break;
        }

        let expr = parse_expr_until(parser, |p| {
            p.cursor.at_symbol(Symbol::Comma) || p.cursor.at_symbol(Symbol::RBracket)
        });
        if super::form::expression_contains_name(&expr, "return") {
            parser.error(
                DiagnosticCode::ReturnExpressionNotAllowed,
                "return is only allowed as a block terminal form",
                expr.span,
            );
        }
        let span = expr.span;
        items.push(CaptureItemAst { expr, span });

        if parser.cursor.consume_symbol(Symbol::Comma).is_none() {
            break;
        }
    }

    let end = if let Some(rbracket) = parser.cursor.consume_symbol(Symbol::RBracket) {
        rbracket.span
    } else {
        let span = parser.cursor.current_span();
        parser.error(
            DiagnosticCode::UnclosedBracket,
            "unclosed capture clause, expected `]`",
            lbracket.span,
        );
        span
    };

    parser.leave_nesting();
    CaptureClauseAst {
        items,
        span: lbracket.span.join(end),
    }
}

// -- Param clause --

fn parse_param_clause(
    parser: &mut Parser<'_>,
    head_deduce: Option<&crate::DeduceListAst>,
) -> ParamClauseAst {
    let extract = parse_product_extract(parser, BindingSlotContext::Param, head_deduce);
    let span = extract.span;
    ParamClauseAst { extract, span }
}

// -- Return clause --

fn parse_return_clause(parser: &mut Parser<'_>) -> ReturnClauseAst {
    let start = parser.cursor.current_span();
    let slot = parse_binding_slot(parser, BindingSlotContext::Return, None, false);
    let end = slot.span;
    ReturnClauseAst {
        slot,
        span: start.join(end),
    }
}

fn at_call_policy_boundary(parser: &mut Parser<'_>) -> bool {
    parser.cursor.at_symbol(Symbol::ThinArrow)
        || at_callable_implementation_tail(parser)
        || parser.is_form_boundary()
}
