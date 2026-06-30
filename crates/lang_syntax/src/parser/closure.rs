use crate::{
    AtomAst, AtomKind, BodyBlockAst, CaptureClauseAst, CaptureItemAst, ClosureAst, ClosureBodyAst,
    DeleteBodyAst, DiagnosticCode, ExplicitClosureAst, ExprAst, FnHeadPrefixAst, FormAst,
    HeadClauseAst, InPlaceClosureAst, NameAst, ParamClauseAst, ReturnClauseAst, Span, Symbol,
    TokenKind,
};

use super::{
    deduce::parse_deduce_list,
    expr::parse_expr_until,
    form::Parser,
    let_stmt::{parse_binding_slot, parse_product_extract, BindingSlotContext},
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
            if seen_terminal {
                let span = parser.cursor.current_span();
                parser.error(
                    DiagnosticCode::StatementAfterBlockTailValue,
                    "statement after block tail value",
                    span,
                );
            }
            continue;
        }
        if seen_terminal {
            let form = parser.parse_form();
            let span = form_span(&form);
            parser.error(
                DiagnosticCode::StatementAfterReturnEvent,
                "statement after terminal block form",
                span,
            );
            forms.push(FormAst::Error(
                parser.error_ast("statement after terminal block form", span),
            ));
            continue;
        }
        let form = parser.parse_form();
        if matches!(&form, FormAst::ReturnEvent(_)) {
            seen_terminal = true;
        }
        forms.push(form);
    }

    if !forms.is_empty() && !seen_terminal {
        if let Some(last) = forms.last() {
            if matches!(
                last,
                FormAst::Expr(_) | FormAst::Let(_) | FormAst::AliasLet(_) | FormAst::Error(_)
            ) {
                // The last form is implicitly a tail value;
                // nothing else may follow.
            }
        }
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
        return Some(AtomAst {
            kind: AtomKind::Closure(ClosureAst::InPlace(InPlaceClosureAst { body, span })),
            span,
        });
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
            return Some(AtomAst {
                kind: AtomKind::Closure(ClosureAst::Explicit(ExplicitClosureAst {
                    head,
                    body,
                    span,
                })),
                span,
            });
        }
        if parser.cursor.at_symbol(Symbol::LParen) {
            match parse_delete_body(parser) {
                Some(delete_body) => {
                    let span = head.span.join(delete_body.span);
                    return Some(AtomAst {
                        kind: AtomKind::Closure(ClosureAst::Explicit(ExplicitClosureAst {
                            head,
                            body: ClosureBodyAst::Delete(delete_body),
                            span,
                        })),
                        span,
                    });
                }
                None => {
                    parser.recover_to_form_boundary();
                    let body_start = parser.cursor.current_span();
                    parser.error(
                        DiagnosticCode::InvalidClosureHead,
                        "expected `)` then `delete` after `=>`",
                        body_start,
                    );
                    let body = ClosureBodyAst::Block(BodyBlockAst {
                        forms: Vec::new(),
                        span: body_start,
                    });
                    let span = head.span.join(body_start);
                    return Some(AtomAst {
                        kind: AtomKind::Closure(ClosureAst::Explicit(ExplicitClosureAst {
                            head,
                            body,
                            span,
                        })),
                        span,
                    });
                }
            }
        }
        parser.recover_to_form_boundary();
        let body_start = parser.cursor.current_span();
        parser.error(
            DiagnosticCode::InvalidClosureHead,
            "expected `{` or `(message) delete` after `=>`",
            body_start,
        );
        let body = ClosureBodyAst::Block(BodyBlockAst {
            forms: Vec::new(),
            span: body_start,
        });
        let span = head.span.join(body_start);
        return Some(AtomAst {
            kind: AtomKind::Closure(ClosureAst::Explicit(ExplicitClosureAst {
                head,
                body,
                span,
            })),
            span,
        });
    }

    if parser.cursor.at_symbol(Symbol::LBrace) {
        parser.ungate_keep_diagnostics();
        parser.error(
            DiagnosticCode::InvalidClosureHead,
            "closure head before `{` requires `=>`; in-place closure cannot have captures or parameters",
            head.span,
        );
        let body = parse_body_block(parser);
        let span = head.span.join(body.span);
        return Some(AtomAst {
            kind: AtomKind::Error(parser.error_ast("invalid headed closure without `=>`", span)),
            span,
        });
    }

    parser.cursor.set_index(saved);
    parser.ungate_drop_diagnostics();
    None
}

// -- Delete body --

/// Parse `(message_expr) delete` after `=>`.
///
/// Caller has already verified that the cursor is at `(`.
/// Returns `None` if the grouped expression or the `delete` name
/// cannot be parsed (diagnostics are emitted by the caller).
fn parse_delete_body(parser: &mut Parser<'_>) -> Option<DeleteBodyAst> {
    let lparen = parser.cursor.consume_symbol(Symbol::LParen)?;

    // Parse the message expression, terminated by `)`
    let message = parse_expr_until(parser, |p| p.cursor.at_symbol(Symbol::RParen));

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
        message,
        delete_name: NameAst {
            text: "delete".to_string(),
            span: delete_token.span,
        },
        span,
    })
}

// -- FnHeadPrefix --

fn parse_fn_head_prefix(parser: &mut Parser<'_>) -> Option<FnHeadPrefixAst> {
    let start = parser.cursor.current_span();

    let deduce = if parser.cursor.at_symbol(Symbol::Less) {
        Some(parse_deduce_list(parser))
    } else {
        None
    };

    let captures = if parser.cursor.at_symbol(Symbol::LBracket) {
        Some(parse_capture_clause(parser))
    } else {
        None
    };

    let params = if parser.cursor.at_symbol(Symbol::LParen) {
        Some(parse_param_clause(parser, deduce.as_ref()))
    } else {
        None
    };

    let fn_item_trait = if params.is_some() && parser.cursor.consume_symbol(Symbol::Colon).is_some()
    {
        if at_fn_item_trait_boundary(parser) {
            let span = parser.cursor.current_span();
            parser.error(
                DiagnosticCode::InvalidClosureHead,
                "expected function item trait after `:`",
                span,
            );
        }
        let expr = parse_expr_until(parser, |p| {
            p.cursor.at_symbol(Symbol::ThinArrow)
                || p.cursor.at_symbol(Symbol::FatArrow)
                || p.cursor.at_symbol(Symbol::LBrace)
                || p.is_form_boundary()
        });
        Some(expr)
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
        fn_item_trait,
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
    parser.cursor.at_symbol(Symbol::FatArrow)
        || parser.cursor.at_symbol(Symbol::LBrace)
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
            parse_expr_until(parser, clause_expr_boundary)
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

fn at_fn_item_trait_boundary(parser: &mut Parser<'_>) -> bool {
    parser.cursor.at_symbol(Symbol::ThinArrow)
        || parser.cursor.at_symbol(Symbol::FatArrow)
        || parser.cursor.at_symbol(Symbol::LBrace)
        || parser.is_form_boundary()
}
