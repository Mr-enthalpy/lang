use crate::{
    token::operator_spelling_in_expr_context, AliasBinderAst, AnnotationTermAst, BinderNameAst,
    BindingAnnotationAst, BindingPatternAst, BindingSlotAst, CanonicalSkeletonAst, DeduceListAst,
    DiagnosticCode, EntityRefAst, ErrorAst, ExprAst, ExprKind, FormAst, LetAliasAst, LetAst,
    NameAst, NavComponentAst, NumericNameAst, OperatorNameAst, Span, Symbol, TokenKind,
    WithClauseAst, WithClauseKind,
};

use super::{
    atom::parse_nav_group_component, canonical::parse_canonical_skeleton,
    deduce::parse_deduce_list, expr::parse_expr_until, form::Parser,
};

#[derive(Clone, Copy)]
pub enum BindingSlotContext {
    Let,
    Param,
    Return,
}

pub fn parse_let_form(parser: &mut Parser<'_>) -> FormAst {
    let let_token = parser
        .cursor
        .consume_name("let")
        .expect("parse_let_form called at let");

    let token = parser.cursor.peek_non_trivia();
    if is_valid_alias_binder(&token.kind) {
        let next = parser.cursor.peek_next_non_trivia();
        if matches!(next.kind, TokenKind::Symbol(Symbol::TripleEqual)) {
            return FormAst::AliasLet(parse_alias_let_body(parser, let_token.span));
        }
    }

    let mut slot = parse_binding_slot(parser, BindingSlotContext::Let, None, true);
    slot.has_let = true;
    let span = let_token.span.join(slot.span);
    FormAst::Let(LetAst { slot, span })
}

pub fn parse_binding_slot(
    parser: &mut Parser<'_>,
    context: BindingSlotContext,
    inherited_deduce: Option<&DeduceListAst>,
    require_initializer: bool,
) -> BindingSlotAst {
    let start = parser.cursor.current_span();
    let has_let = if matches!(
        context,
        BindingSlotContext::Param | BindingSlotContext::Return
    ) {
        parser.cursor.consume_name("let").is_some()
    } else {
        false
    };

    let deduce = if parser.cursor.at_symbol(Symbol::Less) {
        Some(parse_deduce_list(parser))
    } else {
        None
    };

    if matches!(context, BindingSlotContext::Let) {
        if let Some(deduce) = &deduce {
            if deduce.binders.is_empty() {
                parser.error(
                    DiagnosticCode::InvalidDeduceList,
                    "empty deduce list",
                    deduce.span,
                );
            }
        }
    }

    let pattern = parse_binding_pattern(parser, context, deduce.as_ref(), inherited_deduce);
    let mut end = binding_pattern_span(&pattern);

    let annotation = parse_binding_annotation(parser, context);
    if let Some(annotation) = &annotation {
        end = binding_annotation_span(annotation);
    }

    let with_clause = if parser.cursor.at_name("with") {
        let with_clause = parse_with_clause(parser);
        if matches!(context, BindingSlotContext::Return) {
            let span = with_clause
                .as_ref()
                .map_or(parser.cursor.current_span(), |w| w.span);
            parser.error(
                DiagnosticCode::InvalidClosureHead,
                "with clause is not allowed in return slot",
                span,
            );
        }
        if let Some(with_clause) = &with_clause {
            end = with_clause.span;
        }
        with_clause
    } else {
        None
    };

    let initializer = if matches!(context, BindingSlotContext::Let) {
        Some(parse_let_value(parser, require_initializer))
    } else {
        None
    };
    if let Some(initializer) = &initializer {
        end = initializer.span;
    }

    BindingSlotAst {
        has_let,
        deduce,
        pattern,
        annotation,
        with_clause,
        initializer,
        span: start.join(end),
    }
}

fn parse_binding_pattern(
    parser: &mut Parser<'_>,
    context: BindingSlotContext,
    local_deduce: Option<&DeduceListAst>,
    inherited_deduce: Option<&DeduceListAst>,
) -> BindingPatternAst {
    let token = parser.cursor.peek_non_trivia();

    if at_binding_pattern_boundary(parser, context) {
        let message = match context {
            BindingSlotContext::Let => "expected binding pattern after `let`",
            BindingSlotContext::Param => "expected parameter binding pattern",
            BindingSlotContext::Return => "expected return binding pattern after `->`",
        };
        parser.error(DiagnosticCode::ExpectedName, message, token.span);
        return BindingPatternAst::Error(parser.error_ast(message, token.span));
    }

    if parser.cursor.at_symbol(Symbol::LParen)
        || parser.cursor.at_symbol(Symbol::Less)
        || starts_skeleton_name(parser, context)
        || matches!(token.kind, TokenKind::Name if token.text == "_")
    {
        let empty_deduce;
        let deduce_ref = match local_deduce.or(inherited_deduce) {
            Some(deduce) => deduce,
            None => {
                empty_deduce = DeduceListAst {
                    binders: vec![],
                    span: parser.cursor.current_span(),
                };
                &empty_deduce
            }
        };
        return BindingPatternAst::Skeleton(parse_canonical_skeleton(parser, deduce_ref));
    }

    if matches!(token.kind, TokenKind::Name) {
        let token = parser.cursor.bump_non_trivia();
        return BindingPatternAst::Binder(BinderNameAst::Text(NameAst {
            text: token.text.clone(),
            span: token.span,
        }));
    }

    if let Some(spelling) = operator_spelling_in_expr_context(&token.kind) {
        let token = parser.cursor.bump_non_trivia();
        return BindingPatternAst::Binder(BinderNameAst::Operator(OperatorNameAst {
            spelling: spelling.as_source_text().to_string(),
            span: token.span,
        }));
    }

    let message = match context {
        BindingSlotContext::Let => "expected binding pattern after `let`",
        BindingSlotContext::Param => "expected parameter binding pattern",
        BindingSlotContext::Return => "expected return binding pattern after `->`",
    };
    parser.error(DiagnosticCode::ExpectedName, message, token.span);
    BindingPatternAst::Error(parser.error_ast(message, token.span))
}

fn starts_skeleton_name(parser: &mut Parser<'_>, context: BindingSlotContext) -> bool {
    let token = parser.cursor.peek_non_trivia();
    if !matches!(token.kind, TokenKind::Name) {
        return false;
    }
    let next = parser.cursor.peek_next_non_trivia();
    if matches!(next.kind, TokenKind::Name) && next.text == "with" {
        return false;
    }
    !is_binding_pattern_stop_kind(&next.kind, context)
}

fn at_binding_pattern_boundary(parser: &mut Parser<'_>, context: BindingSlotContext) -> bool {
    is_binding_pattern_stop_kind(&parser.cursor.peek_non_trivia().kind, context)
        || parser.is_form_boundary()
}

fn is_binding_pattern_stop_kind(kind: &TokenKind, context: BindingSlotContext) -> bool {
    match kind {
        TokenKind::Eof => true,
        TokenKind::Symbol(Symbol::Colon | Symbol::Comma | Symbol::RParen) => true,
        TokenKind::Symbol(Symbol::Equal) if matches!(context, BindingSlotContext::Let) => true,
        TokenKind::Symbol(Symbol::FatArrow | Symbol::LBrace)
            if matches!(context, BindingSlotContext::Return) =>
        {
            true
        }
        TokenKind::Name => false,
        _ => false,
    }
}

fn parse_binding_annotation(
    parser: &mut Parser<'_>,
    context: BindingSlotContext,
) -> Option<BindingAnnotationAst> {
    parser.cursor.consume_symbol(Symbol::Colon)?;

    let start = parser.cursor.current_span();
    if annotation_stop(parser, context) {
        parser.error(
            DiagnosticCode::ExpectedBindingAnnotation,
            "expected binding annotation",
            start,
        );
        return Some(BindingAnnotationAst::Error(
            parser.error_ast("expected binding annotation", start),
        ));
    }

    if parser.cursor.at_name("_")
        && matches!(
            parser.cursor.peek_next_non_trivia().kind,
            TokenKind::Symbol(Symbol::Colon)
        )
    {
        let hole = parser.cursor.bump_non_trivia();
        parser.cursor.consume_symbol(Symbol::Colon);
        let right = parse_expr_until(parser, |p| annotation_stop(p, context));
        let span = hole.span.join(right.span);
        return Some(BindingAnnotationAst::Compound {
            left: AnnotationTermAst::Hole { span: hole.span },
            right,
            span,
        });
    }

    let left_or_expr = parse_expr_until(parser, |p| {
        p.cursor.at_symbol(Symbol::Colon) || annotation_stop(p, context)
    });

    if parser.cursor.consume_symbol(Symbol::Colon).is_some() {
        let right = parse_expr_until(parser, |p| annotation_stop(p, context));
        let span = left_or_expr.span.join(right.span);
        Some(BindingAnnotationAst::Compound {
            left: AnnotationTermAst::Expr(left_or_expr),
            right,
            span,
        })
    } else {
        Some(BindingAnnotationAst::Expr(left_or_expr))
    }
}

fn annotation_stop(parser: &mut Parser<'_>, context: BindingSlotContext) -> bool {
    parser.cursor.at_name("with")
        || parser.cursor.at_symbol(Symbol::Comma)
        || parser.cursor.at_symbol(Symbol::RParen)
        || parser.cursor.at_symbol(Symbol::FatArrow)
        || parser.cursor.at_symbol(Symbol::LBrace)
        || (matches!(context, BindingSlotContext::Let) && parser.cursor.at_symbol(Symbol::Equal))
        || parser.is_form_boundary()
}

fn parse_let_value(parser: &mut Parser<'_>, require_initializer: bool) -> ExprAst {
    if parser.cursor.consume_symbol(Symbol::Equal).is_some() {
        parse_expr_until(parser, |parser| parser.is_form_boundary())
    } else {
        let span = parser.cursor.current_span();
        if require_initializer {
            parser.error(DiagnosticCode::ExpectedEqual, "expected `=` in let", span);
            parser.recover_to_form_boundary();
        }
        error_expr(parser, "expected `=` in let", span)
    }
}

fn parse_alias_let_body(parser: &mut Parser<'_>, let_span_start: Span) -> LetAliasAst {
    let name_token = parser.cursor.bump_non_trivia();
    let binder = binder_name_to_alias_binder(name_token);

    parser.cursor.consume_symbol(Symbol::TripleEqual);

    let target = parse_entity_ref(parser);
    let span = let_span_start.join(target.span);
    LetAliasAst {
        binder,
        target,
        span,
    }
}

fn binder_name_to_alias_binder(token: &crate::Token) -> AliasBinderAst {
    match &token.kind {
        TokenKind::Name => AliasBinderAst::Name(NameAst {
            text: token.text.clone(),
            span: token.span,
        }),
        _ => {
            if let Some(spelling) = operator_spelling_in_expr_context(&token.kind) {
                AliasBinderAst::Operator(OperatorNameAst {
                    spelling: spelling.as_source_text().to_string(),
                    span: token.span,
                })
            } else {
                AliasBinderAst::Error(ErrorAst {
                    message: "invalid alias binder".to_string(),
                    span: token.span,
                })
            }
        }
    }
}

fn is_valid_alias_binder(kind: &TokenKind) -> bool {
    matches!(kind, TokenKind::Name) || operator_spelling_in_expr_context(kind).is_some()
}

fn parse_entity_ref(parser: &mut Parser<'_>) -> EntityRefAst {
    let start = parser.cursor.current_raw_span();
    let mut components: Vec<NavComponentAst> = Vec::new();

    if is_entity_ref_boundary(parser) {
        parser.error(
            DiagnosticCode::ExpectedAliasTarget,
            "expected entity reference after `===`",
            start,
        );
        return EntityRefAst {
            components: vec![NavComponentAst::Error(
                parser.error_ast("expected entity reference", start),
            )],
            span: start,
        };
    }

    let Some(first) = parse_entity_inner_component(parser) else {
        let span = parser.cursor.current_span();
        let (code, message, node_message) = if parser.cursor.at_symbol(Symbol::LParen) {
            (
                DiagnosticCode::InvalidEntityRef,
                "grouped expression cannot be an innermost navigation component",
                "grouped expression cannot be an innermost navigation component",
            )
        } else {
            (
                DiagnosticCode::ExpectedAliasTarget,
                "expected entity reference after `===`",
                "expected entity reference",
            )
        };
        parser.error(code, message, span);
        parser.cursor.bump_non_trivia();
        parser.recover_to_form_boundary();
        return EntityRefAst {
            components: vec![NavComponentAst::Error(parser.error_ast(node_message, span))],
            span: start.join(span),
        };
    };

    let mut span = start.join(nav_component_span(&first));
    components.push(first);

    while !is_entity_ref_boundary(parser)
        && parser.cursor.consume_symbol(Symbol::ColonColon).is_some()
    {
        if is_entity_ref_boundary(parser) {
            let error_span = parser.cursor.current_span();
            parser.error(
                DiagnosticCode::ExpectedAliasTarget,
                "expected navigation component after `::`",
                error_span,
            );
            span = span.join(error_span);
            components.push(NavComponentAst::Error(
                parser.error_ast("expected navigation component", error_span),
            ));
            break;
        }

        let Some(component) = parse_entity_outer_component(parser) else {
            let error_span = parser.cursor.current_span();
            parser.error(
                DiagnosticCode::InvalidEntityRef,
                "expected navigation component after `::`",
                error_span,
            );
            parser.cursor.bump_non_trivia();
            parser.recover_to_form_boundary();
            span = span.join(error_span);
            components.push(NavComponentAst::Error(
                parser.error_ast("expected navigation component", error_span),
            ));
            break;
        };

        span = span.join(nav_component_span(&component));
        components.push(component);
    }

    finish_entity_ref(parser, components, span)
}

fn finish_entity_ref(
    parser: &mut Parser<'_>,
    components: Vec<NavComponentAst>,
    span: Span,
) -> EntityRefAst {
    if parser.is_alias_rhs_boundary() {
        return EntityRefAst { components, span };
    }

    let next = parser.cursor.peek_non_trivia();
    parser.error(
        DiagnosticCode::UnexpectedAliasRhsExpression,
        format!("unexpected token `{}` after entity reference", next.text),
        next.span,
    );
    parser.recover_to_form_boundary();
    EntityRefAst { components, span }
}

fn parse_entity_inner_component(parser: &mut Parser<'_>) -> Option<NavComponentAst> {
    let token = parser.cursor.peek_non_trivia();
    match token.kind {
        TokenKind::Name => {
            let token = parser.cursor.bump_non_trivia();
            Some(NavComponentAst::Text(NameAst {
                text: token.text.clone(),
                span: token.span,
            }))
        }
        TokenKind::IntLiteral => {
            let token = parser.cursor.bump_non_trivia();
            Some(NavComponentAst::Numeric(NumericNameAst {
                text: token.text.clone(),
                span: token.span,
            }))
        }
        _ => {
            let spelling = operator_spelling_in_expr_context(&token.kind)?;
            let token = parser.cursor.bump_non_trivia();
            Some(NavComponentAst::Operator(OperatorNameAst {
                spelling: spelling.as_source_text().to_string(),
                span: token.span,
            }))
        }
    }
}

fn parse_entity_outer_component(parser: &mut Parser<'_>) -> Option<NavComponentAst> {
    let token = parser.cursor.peek_non_trivia();
    match token.kind {
        TokenKind::Name => {
            let token = parser.cursor.bump_non_trivia();
            Some(NavComponentAst::Text(NameAst {
                text: token.text.clone(),
                span: token.span,
            }))
        }
        TokenKind::IntLiteral => {
            let token = parser.cursor.bump_non_trivia();
            Some(NavComponentAst::Numeric(NumericNameAst {
                text: token.text.clone(),
                span: token.span,
            }))
        }
        TokenKind::Symbol(Symbol::LParen) => parse_nav_group_component(parser),
        _ if token.kind.is_operator_spelling() => {
            let token = parser.cursor.bump_non_trivia();
            parser.error(
                DiagnosticCode::InvalidEntityRef,
                "operator cannot be an outer navigation component",
                token.span,
            );
            Some(NavComponentAst::Error(parser.error_ast(
                "operator cannot be an outer navigation component",
                token.span,
            )))
        }
        _ => None,
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

fn is_entity_ref_boundary(parser: &mut Parser<'_>) -> bool {
    let raw = parser.cursor.peek();
    if matches!(
        raw.kind,
        TokenKind::Eof | TokenKind::Symbol(Symbol::Semicolon | Symbol::RBrace)
    ) {
        return true;
    }
    if parser.is_alias_rhs_newline_boundary() {
        return true;
    }
    false
}

fn parse_with_clause(parser: &mut Parser<'_>) -> Option<WithClauseAst> {
    let Some(with_token) = parser.cursor.consume_name("with") else {
        return None;
    };

    let Some(lbrace) = parser.cursor.consume_symbol(Symbol::LBrace) else {
        let span = parser.cursor.current_span();
        parser.error(
            DiagnosticCode::UnexpectedToken,
            "expected `{` after `with`",
            span,
        );
        recover_to_initializer(parser);
        let error_span = with_token.span.join(span);
        return Some(WithClauseAst {
            kind: WithClauseKind::Error(parser.error_ast("invalid with clause", error_span)),
            span: error_span,
        });
    };

    let mut items = Vec::new();
    let mut invalid_span: Option<Span> = None;

    if let Some(rbrace) = parser.cursor.consume_symbol(Symbol::RBrace) {
        return Some(WithClauseAst {
            kind: WithClauseKind::Empty,
            span: with_token.span.join(rbrace.span),
        });
    }

    loop {
        let token = parser.cursor.peek_non_trivia();
        if !matches!(token.kind, TokenKind::Name) {
            parser.error(
                DiagnosticCode::ExpectedName,
                "expected name in with clause",
                token.span,
            );
            invalid_span = Some(token.span);
            recover_to_with_block_end(parser);
            break;
        }

        let token = parser.cursor.bump_non_trivia();
        items.push(NameAst {
            text: token.text.clone(),
            span: token.span,
        });

        if parser.cursor.consume_symbol(Symbol::Comma).is_none() {
            break;
        }

        if parser.cursor.at_symbol(Symbol::RBrace) {
            let span = parser.cursor.current_span();
            parser.error(
                DiagnosticCode::ExpectedName,
                "expected name after `,` in with clause",
                span,
            );
            invalid_span = Some(span);
            break;
        }
    }

    let end = if let Some(rbrace) = parser.cursor.consume_symbol(Symbol::RBrace) {
        rbrace.span
    } else {
        parser.error(
            DiagnosticCode::UnclosedBrace,
            "unclosed with block, expected `}`",
            lbrace.span,
        );
        let span = parser.cursor.current_span();
        invalid_span = Some(invalid_span.map_or(lbrace.span, |invalid| invalid.join(span)));
        span
    };

    let span = with_token.span.join(end);
    if let Some(invalid_span) = invalid_span {
        return Some(WithClauseAst {
            kind: WithClauseKind::Error(
                parser.error_ast("invalid with clause", with_token.span.join(invalid_span)),
            ),
            span,
        });
    }

    Some(WithClauseAst {
        kind: if items.is_empty() {
            WithClauseKind::Empty
        } else {
            WithClauseKind::Items { items }
        },
        span,
    })
}

fn recover_to_with_block_end(parser: &mut Parser<'_>) {
    while !parser.is_form_boundary()
        && !parser.cursor.at_symbol(Symbol::Equal)
        && !parser.cursor.at_symbol(Symbol::RBrace)
    {
        parser.cursor.bump_non_trivia();
    }
}

fn recover_to_initializer(parser: &mut Parser<'_>) {
    while !parser.is_form_boundary() && !parser.cursor.at_symbol(Symbol::Equal) {
        parser.cursor.bump_non_trivia();
    }
}

fn binding_pattern_span(pattern: &BindingPatternAst) -> Span {
    match pattern {
        BindingPatternAst::Binder(name) => binder_name_span(name),
        BindingPatternAst::Skeleton(skeleton) => skeleton_span(skeleton),
        BindingPatternAst::Error(error) => error.span,
    }
}

fn binder_name_span(name: &BinderNameAst) -> Span {
    match name {
        BinderNameAst::Text(name) => name.span,
        BinderNameAst::Operator(name) => name.span,
    }
}

fn binding_annotation_span(annotation: &BindingAnnotationAst) -> Span {
    match annotation {
        BindingAnnotationAst::Expr(expr) => expr.span,
        BindingAnnotationAst::Compound { span, .. } => *span,
        BindingAnnotationAst::Error(error) => error.span,
    }
}

fn skeleton_span(skeleton: &CanonicalSkeletonAst) -> Span {
    match skeleton {
        CanonicalSkeletonAst::Segment { span, .. } => *span,
        CanonicalSkeletonAst::ArgPack { span, .. } => *span,
        CanonicalSkeletonAst::Wildcard { span } => *span,
        CanonicalSkeletonAst::Name { span, .. } => *span,
        CanonicalSkeletonAst::NavPath { span, .. } => *span,
        CanonicalSkeletonAst::Literal { span, .. } => *span,
        CanonicalSkeletonAst::Error(error) => error.span,
    }
}

fn error_expr(parser: &Parser<'_>, message: &str, span: Span) -> ExprAst {
    ExprAst {
        kind: ExprKind::Error(parser.error_ast(message, span)),
        span,
    }
}
