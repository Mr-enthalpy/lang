use crate::TokenKind;
use crate::{
    AtomAst, AtomKind, BodyBlockAst, CanonicalSkeletonAst, CaptureClauseAst, CaptureItemAst,
    ClosureAst, DiagnosticCode, ExplicitClosureAst, FnHeadPrefixAst, InlineClosureAst, NameAst,
    ParamClauseAst, ParamItemAst, ReturnBinderAst, ReturnClauseAst, Span, Symbol,
    TypeObjectAnnotationAst,
};

use super::{
    canonical::parse_canonical_skeleton, deduce::parse_deduce_list, expr::parse_expr_until,
    form::Parser,
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
    // Bare inline closure: { ... }
    if parser.cursor.at_symbol(Symbol::LBrace) {
        let body = parse_body_block(parser);
        let span = body.span;
        return Some(AtomAst {
            kind: AtomKind::Closure(ClosureAst::Inline(InlineClosureAst {
                head: None,
                body,
                span,
            })),
            span,
        });
    }

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
        if parser.cursor.at_symbol(Symbol::LBrace) {
            parser.ungate_keep_diagnostics();
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
    } else if parser.cursor.at_symbol(Symbol::LBrace) {
        parser.ungate_keep_diagnostics();
        let body = parse_body_block(parser);
        let span = head.span.join(body.span);
        return Some(AtomAst {
            kind: AtomKind::Closure(ClosureAst::Inline(InlineClosureAst {
                head: Some(head),
                body,
                span,
            })),
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
        Some(parse_param_clause(parser))
    } else {
        None
    };

    let fn_item_trait = if params.is_some() && parser.cursor.consume_symbol(Symbol::Colon).is_some()
    {
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

fn parse_param_clause(parser: &mut Parser<'_>) -> ParamClauseAst {
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

        let param = parse_param_item(parser);
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

fn parse_param_item(parser: &mut Parser<'_>) -> ParamItemAst {
    let token = parser.cursor.peek_non_trivia();

    // Extract param: starts with < or _
    if parser.cursor.at_symbol(Symbol::Less) {
        let deduce = Some(parse_deduce_list(parser));
        let skeleton = parse_canonical_skeleton(
            parser,
            deduce
                .as_ref()
                .unwrap_or_else(|| unreachable!("deduce list just parsed")),
        );
        let annotation = parse_param_annotation(parser);
        let span = deduce
            .as_ref()
            .map(|d| d.span)
            .unwrap_or(skeleton_span(&skeleton));
        let end = annotation
            .as_ref()
            .map(|a| type_object_span(a))
            .unwrap_or(span);
        return ParamItemAst::ExtractParam {
            deduce,
            skeleton,
            annotation,
            span: span.join(end),
        };
    }

    if matches!(token.kind, TokenKind::Name if token.text == "_") && is_at_param_stop(parser, true)
    {
        let wildcard_span = parser.cursor.current_span();
        parser.cursor.bump_non_trivia();
        let skeleton = CanonicalSkeletonAst::Wildcard {
            span: wildcard_span,
        };
        let annotation = parse_param_annotation(parser);
        let span = wildcard_span;
        let end = annotation
            .as_ref()
            .map(|a| type_object_span(a))
            .unwrap_or(span);
        return ParamItemAst::ExtractParam {
            deduce: None,
            skeleton,
            annotation,
            span: span.join(end),
        };
    }

    // Name param
    if matches!(token.kind, TokenKind::Name) {
        let name_token = parser.cursor.bump_non_trivia();
        let name_span = name_token.span;
        let name = NameAst {
            text: name_token.text.clone(),
            span: name_span,
        };
        let annotation = parse_param_annotation(parser);
        let end = annotation
            .as_ref()
            .map(|a| type_object_span(a))
            .unwrap_or(name_span);
        return ParamItemAst::NameParam {
            span: name_span.join(end),
            name,
            annotation,
        };
    }

    let span = token.span;
    parser.error(
        DiagnosticCode::InvalidClosureHead,
        "expected parameter",
        span,
    );
    ParamItemAst::Error(parser.error_ast("expected parameter", span))
}

fn parse_param_annotation(parser: &mut Parser<'_>) -> Option<TypeObjectAnnotationAst> {
    if !parser.cursor.consume_symbol(Symbol::Colon).is_some() {
        return None;
    }
    let token = parser.cursor.peek_non_trivia();
    if matches!(token.kind, TokenKind::Name if token.text == "_") && is_at_param_stop(parser, false)
    {
        let hole = parser.cursor.bump_non_trivia();
        return Some(TypeObjectAnnotationAst::Hole { span: hole.span });
    }
    let expr = parse_expr_until(parser, |p| {
        p.cursor.at_symbol(Symbol::Comma) || p.cursor.at_symbol(Symbol::RParen)
    });
    Some(TypeObjectAnnotationAst::Expr(expr))
}

fn is_at_param_stop(parser: &mut Parser<'_>, check_next: bool) -> bool {
    let look = if check_next {
        parser.cursor.peek_next_non_trivia()
    } else {
        parser.cursor.peek_non_trivia()
    };
    matches!(look.kind, TokenKind::Symbol(Symbol::Comma | Symbol::RParen))
        || matches!(look.kind, TokenKind::Eof)
}

// -- Return clause --

fn parse_return_clause(parser: &mut Parser<'_>) -> ReturnClauseAst {
    let start = parser.cursor.current_span();

    let binder = if parser.cursor.at_symbol(Symbol::Less) {
        let deduce = parse_deduce_list(parser);
        let skeleton = parse_canonical_skeleton(parser, &deduce);
        let span = deduce.span.join(skeleton_span(&skeleton));
        ReturnBinderAst::ExtractType {
            deduce,
            skeleton,
            span,
        }
    } else {
        let expr = parse_expr_until(parser, |p| {
            p.cursor.at_symbol(Symbol::Colon)
                || p.cursor.at_symbol(Symbol::FatArrow)
                || p.cursor.at_symbol(Symbol::LBrace)
                || p.cursor.is_form_boundary()
        });
        ReturnBinderAst::TypeExpr(expr)
    };

    let constraint = if parser.cursor.consume_symbol(Symbol::Colon).is_some() {
        let expr = parse_expr_until(parser, |p| {
            p.cursor.at_symbol(Symbol::FatArrow)
                || p.cursor.at_symbol(Symbol::LBrace)
                || p.is_form_boundary()
        });
        Some(expr)
    } else {
        None
    };

    let end = constraint
        .as_ref()
        .map(|c| c.span)
        .unwrap_or(return_binder_span(&binder));
    ReturnClauseAst {
        binder,
        constraint,
        span: start.join(end),
    }
}

fn return_binder_span(binder: &ReturnBinderAst) -> Span {
    match binder {
        ReturnBinderAst::TypeExpr(expr) => expr.span,
        ReturnBinderAst::ExtractType { span, .. } => *span,
        ReturnBinderAst::Error(error) => error.span,
    }
}

fn skeleton_span(skeleton: &CanonicalSkeletonAst) -> Span {
    match skeleton {
        CanonicalSkeletonAst::Segment { span, .. } => *span,
        CanonicalSkeletonAst::ArgPack { span, .. } => *span,
        CanonicalSkeletonAst::Wildcard { span } => *span,
        CanonicalSkeletonAst::Name { span, .. } => *span,
        CanonicalSkeletonAst::Path { span, .. } => *span,
        CanonicalSkeletonAst::Literal { span, .. } => *span,
        CanonicalSkeletonAst::Error(error) => error.span,
    }
}

fn type_object_span(annotation: &TypeObjectAnnotationAst) -> Span {
    match annotation {
        TypeObjectAnnotationAst::Expr(expr) => expr.span,
        TypeObjectAnnotationAst::Hole { span } => *span,
    }
}
