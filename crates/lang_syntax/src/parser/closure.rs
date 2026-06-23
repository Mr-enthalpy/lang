use crate::{
    AtomAst, AtomKind, BodyBlockAst, CaptureClauseAst, CaptureItemAst, ClosureAst, DiagnosticCode,
    ExplicitClosureAst, FnHeadPrefixAst, InlineClosureAst, ParamClauseAst, ReturnClauseAst, Symbol,
};

use super::{
    deduce::parse_deduce_list,
    expr::parse_expr_until,
    form::Parser,
    let_stmt::{parse_binding_slot, BindingSlotContext},
};

// -- Body block --

pub fn parse_body_block(parser: &mut Parser<'_>) -> BodyBlockAst {
    let lbrace = parser
        .cursor
        .consume_symbol(Symbol::LBrace)
        .expect("parse_body_block at `{`");

    parser.enter_nesting();
    let mut forms = Vec::new();

    loop {
        if parser.cursor.at_eof() || parser.cursor.at_symbol(Symbol::RBrace) {
            break;
        }
        if parser.cursor.consume_symbol(Symbol::Semicolon).is_some() {
            continue;
        }
        forms.push(parser.parse_form());
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

// -- Closure entry from atom parser --

pub fn try_parse_closure(parser: &mut Parser<'_>) -> Option<AtomAst> {
    // Attempt FnHeadPrefix lookahead
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

    // Check for closure continuation
    if parser.cursor.consume_symbol(Symbol::FatArrow).is_some() {
        parser.ungate_keep_diagnostics();
        if parser.cursor.at_symbol(Symbol::LBrace) {
            let body = parse_body_block(parser);
            let span = head.span.join(body.span);
            return Some(AtomAst {
                kind: AtomKind::Closure(ClosureAst::Explicit(ExplicitClosureAst {
                    head: Some(head),
                    body,
                    span,
                })),
                span,
            });
        }
        // => consumed but no { follows — malformed explicit closure
        let body_start = parser.cursor.current_span();
        parser.error(
            DiagnosticCode::InvalidClosureHead,
            "expected `{` after `=>`",
            body_start,
        );
        let body = BodyBlockAst {
            forms: Vec::new(),
            span: body_start,
        };
        let span = head.span.join(body.span);
        return Some(AtomAst {
            kind: AtomKind::Closure(ClosureAst::Explicit(ExplicitClosureAst {
                head: Some(head),
                body,
                span,
            })),
            span,
        });
    } else if parser.cursor.at_symbol(Symbol::LBrace) {
        parser.ungate_keep_diagnostics();
        let body = parse_body_block(parser);
        let span = head.span.join(body.span);
        return Some(AtomAst {
            kind: AtomKind::Closure(ClosureAst::Inline(InlineClosureAst { head, body, span })),
            span,
        });
    }

    // Head parsed but no valid closure continuation — restore and retry
    parser.cursor.set_index(saved);
    parser.ungate_drop_diagnostics();
    None
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

    let end = parser.cursor.current_span();
    let span = start.join(end);

    if deduce.is_none() && captures.is_none() && params.is_none() {
        return None;
    }

    Some(FnHeadPrefixAst {
        deduce,
        captures,
        params,
        fn_item_trait,
        returns,
        span,
    })
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
    let lparen = parser
        .cursor
        .consume_symbol(Symbol::LParen)
        .expect("parse_param_clause at `(`");

    parser.enter_nesting();
    let mut params = Vec::new();

    loop {
        if parser.cursor.at_eof()
            || parser.cursor.at_symbol(Symbol::RParen)
            || parser.is_form_boundary()
        {
            break;
        }

        let param = parse_binding_slot(parser, BindingSlotContext::Param, head_deduce, false);
        params.push(param);

        if parser.cursor.consume_symbol(Symbol::Comma).is_none() {
            break;
        }

        if parser.cursor.at_symbol(Symbol::RParen)
            || parser.cursor.at_eof()
            || parser.is_form_boundary()
        {
            let span = parser.cursor.current_span();
            parser.error(
                DiagnosticCode::InvalidClosureHead,
                "trailing comma in parameter list",
                span,
            );
            break;
        }
    }

    let end = if let Some(rparen) = parser.cursor.consume_symbol(Symbol::RParen) {
        rparen.span
    } else {
        let span = parser.cursor.current_span();
        parser.error(
            DiagnosticCode::UnclosedParen,
            "unclosed parameter clause, expected `)`",
            lparen.span,
        );
        span
    };

    parser.leave_nesting();
    ParamClauseAst {
        params,
        span: lparen.span.join(end),
    }
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
