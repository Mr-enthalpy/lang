use crate::{
    token::operator_spelling_in_expr_context, AliasBinderAst, AnnotationTermAst, BinderNameAst,
    BindingAnnotationAst, BindingPatternAst, BindingSlotAst, CanonicalSkeletonAst, DeduceListAst,
    DiagnosticCode, EntityRefAst, ErrorAst, ExprAst, ExprKind, FormAst, LetAliasAst, LetAst,
    NameAst, NavComponentAst, NumericNameAst, OperatorNameAst, ProductExtractAst,
    ProductExtractElementAst, Span, Symbol, TokenKind, WithClauseAst, WithClauseKind,
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

pub fn parse_let_form(parser: &mut Parser<'_>, policy: Option<ExprAst>) -> FormAst {
    let start = policy
        .as_ref()
        .map(|expr| expr.span)
        .unwrap_or_else(|| parser.cursor.current_span());
    parser
        .cursor
        .consume_name("let")
        .expect("parse_let_form called at let");

    if alias_binder_followed_by_triple_equal(parser) {
        return FormAst::AliasLet(parse_alias_let_body(parser, start, policy));
    }

    let mut slot = parse_binding_slot(parser, BindingSlotContext::Let, None, true);
    slot.has_let = true;
    slot.policy = policy;
    let span = start.join(slot.span);
    FormAst::Let(LetAst { slot, span })
}

fn starts_binding_deduce_list(parser: &mut Parser<'_>) -> bool {
    if !parser.cursor.at_symbol(Symbol::Less) {
        return false;
    }
    let next = parser.cursor.peek_next_non_trivia();
    matches!(
        next.kind,
        TokenKind::Name | TokenKind::Symbol(Symbol::Greater | Symbol::Comma)
    )
}

pub fn parse_binding_slot(
    parser: &mut Parser<'_>,
    context: BindingSlotContext,
    inherited_deduce: Option<&DeduceListAst>,
    require_initializer: bool,
) -> BindingSlotAst {
    let start = parser.cursor.current_span();
    let (policy, has_let) = parse_slot_policy_and_let(parser, context);

    let deduce = if starts_binding_deduce_list(parser) {
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

    let pattern =
        parse_binding_pattern(parser, context, has_let, deduce.as_ref(), inherited_deduce);
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
        policy,
        has_let,
        deduce,
        pattern,
        annotation,
        with_clause,
        initializer,
        span: start.join(end),
    }
}

pub fn parse_product_extract(
    parser: &mut Parser<'_>,
    inherited_deduce: Option<&DeduceListAst>,
) -> ProductExtractAst {
    let lparen = parser
        .cursor
        .consume_symbol(Symbol::LParen)
        .expect("parse_product_extract at `(`");

    parser.enter_nesting();
    let mut elements = Vec::new();
    let mut expect_element = true;

    loop {
        if parser.cursor.at_eof()
            || parser.cursor.at_symbol(Symbol::RParen)
            || parser.is_form_boundary()
        {
            break;
        }

        if parser.cursor.at_symbol(Symbol::Comma) {
            let comma = parser.cursor.bump_non_trivia();
            if expect_element {
                elements.push(ProductExtractElementAst::Unit { span: comma.span });
            }
            expect_element = true;
            continue;
        }

        let element =
            parse_binding_slot(parser, BindingSlotContext::Param, inherited_deduce, false);
        elements.push(ProductExtractElementAst::Slot(element));

        if let Some(comma) = parser.cursor.consume_symbol(Symbol::Comma) {
            expect_element = true;
            if parser.cursor.at_symbol(Symbol::RParen)
                || parser.cursor.at_eof()
                || parser.is_form_boundary()
            {
                elements.push(ProductExtractElementAst::Unit { span: comma.span });
                break;
            }
        } else {
            break;
        }
    }

    let end = if let Some(rparen) = parser.cursor.consume_symbol(Symbol::RParen) {
        rparen.span
    } else {
        let span = parser.cursor.current_span();
        parser.error(
            DiagnosticCode::UnclosedParen,
            "unclosed product extraction, expected `)`",
            lparen.span,
        );
        span
    };

    parser.leave_nesting();
    ProductExtractAst {
        elements,
        span: lparen.span.join(end),
    }
}

// Detect an optional policy expression in `Param`/`Return` binding-slot prefix
// position. A policy is recognized only by the shape `Expr let`: the parser
// speculatively parses an expression that stops at a top-level `let`, and keeps
// it only if a `let` actually follows. Without the `let` anchor the tokens are
// restored for ordinary pattern / canonical-skeleton parsing. In `Let` context
// the policy and `let` are handled by `parse_let_form`.
fn parse_slot_policy_and_let(
    parser: &mut Parser<'_>,
    context: BindingSlotContext,
) -> (Option<ExprAst>, bool) {
    if !matches!(
        context,
        BindingSlotContext::Param | BindingSlotContext::Return
    ) {
        return (None, false);
    }

    if parser.cursor.at_name("let") {
        parser.cursor.bump_non_trivia();
        return (None, true);
    }

    let saved = parser.cursor.current_index();
    parser.gate_diagnostics();
    let expr = parse_expr_until(parser, |p| {
        p.cursor.at_name("let") || slot_policy_boundary(p, context)
    });

    if parser.cursor.at_name("let") {
        parser.ungate_keep_diagnostics();
        parser.cursor.bump_non_trivia();
        (Some(expr), true)
    } else {
        parser.cursor.set_index(saved);
        parser.ungate_drop_diagnostics();
        (None, false)
    }
}

fn slot_policy_boundary(parser: &mut Parser<'_>, context: BindingSlotContext) -> bool {
    if parser.is_form_boundary() {
        return true;
    }
    match context {
        BindingSlotContext::Param => {
            parser.cursor.at_symbol(Symbol::Colon)
                || parser.cursor.at_symbol(Symbol::Comma)
                || parser.cursor.at_symbol(Symbol::RParen)
        }
        BindingSlotContext::Return => {
            parser.cursor.at_symbol(Symbol::Colon)
                || parser.cursor.at_symbol(Symbol::FatArrow)
                || parser.cursor.at_symbol(Symbol::LBrace)
                || parser.cursor.at_name("with")
        }
        BindingSlotContext::Let => true,
    }
}

fn parse_binding_pattern(
    parser: &mut Parser<'_>,
    context: BindingSlotContext,
    has_let: bool,
    local_deduce: Option<&DeduceListAst>,
    inherited_deduce: Option<&DeduceListAst>,
) -> BindingPatternAst {
    let token = parser.cursor.peek_non_trivia();

    if at_binding_pattern_boundary(parser, context) {
        if has_let && parser.cursor.at_symbol(Symbol::Colon) {
            return BindingPatternAst::Implicit { span: token.span };
        }
        let message = match context {
            BindingSlotContext::Let => "expected binding pattern after `let`",
            BindingSlotContext::Param => "expected parameter binding pattern",
            BindingSlotContext::Return => "expected return binding pattern after `->`",
        };
        parser.error(DiagnosticCode::ExpectedName, message, token.span);
        return BindingPatternAst::Error(parser.error_ast(message, token.span));
    }

    if parser.cursor.at_symbol(Symbol::LParen) {
        return BindingPatternAst::Product(parse_product_extract(
            parser,
            local_deduce.or(inherited_deduce),
        ));
    }

    if starts_skeleton_name(parser, context)
        || matches!(token.kind, TokenKind::Name if token.text == "_")
        || matches!(token.kind, TokenKind::IntLiteral | TokenKind::StringLiteral)
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

    if let Some(operator) = try_consume_bracket_operator_name(parser) {
        return BindingPatternAst::Binder(BinderNameAst::Operator(operator));
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
    if matches!(
        context,
        BindingSlotContext::Param | BindingSlotContext::Return
    ) && next_token_starts_head_clause(parser)
    {
        return false;
    }
    !is_binding_pattern_stop_kind(&next.kind, context)
}

// Head clauses (`require`/`pre`/`post`/`lifetime pre`/`lifetime post`) only act
// as binding-slot boundaries inside closure-head parameter and return slots.
fn next_token_starts_head_clause(parser: &Parser<'_>) -> bool {
    let (current_index, _) = parser
        .cursor
        .peek_at_skip_trivia(parser.cursor.current_index());
    super::closure::token_index_starts_head_clause(parser, current_index + 1)
}

fn at_binding_pattern_boundary(parser: &mut Parser<'_>, context: BindingSlotContext) -> bool {
    if matches!(
        context,
        BindingSlotContext::Param | BindingSlotContext::Return
    ) && super::closure::at_head_clause_keyword(parser)
    {
        return true;
    }
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
        || (matches!(
            context,
            BindingSlotContext::Param | BindingSlotContext::Return
        ) && super::closure::at_head_clause_keyword(parser))
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

fn parse_alias_let_body(
    parser: &mut Parser<'_>,
    span_start: Span,
    policy: Option<ExprAst>,
) -> LetAliasAst {
    let binder = parse_alias_binder(parser);

    parser.cursor.consume_symbol(Symbol::TripleEqual);

    let target = parse_entity_ref(parser);
    let span = span_start.join(target.span);
    LetAliasAst {
        policy,
        binder,
        target,
        span,
    }
}

fn parse_alias_binder(parser: &mut Parser<'_>) -> AliasBinderAst {
    if let Some(operator) = try_consume_bracket_operator_name(parser) {
        return AliasBinderAst::Operator(operator);
    }
    let token = parser.cursor.bump_non_trivia();
    binder_name_to_alias_binder(token)
}

// True when the upcoming alias binder (a single-token name/operator, or the
// paired `[]` operator) is immediately followed by `===`.
fn alias_binder_followed_by_triple_equal(parser: &mut Parser<'_>) -> bool {
    let token = parser.cursor.peek_non_trivia();
    if is_valid_alias_binder(&token.kind) {
        return matches!(
            parser.cursor.peek_next_non_trivia().kind,
            TokenKind::Symbol(Symbol::TripleEqual)
        );
    }
    if matches!(token.kind, TokenKind::Symbol(Symbol::LBracket)) {
        let cursor_index = parser.cursor.current_index();
        let (rbracket_index, rbracket) = parser.cursor.peek_at_skip_trivia(cursor_index + 1);
        if matches!(rbracket.kind, TokenKind::Symbol(Symbol::RBracket)) {
            let (_, after) = parser.cursor.peek_at_skip_trivia(rbracket_index + 1);
            return matches!(after.kind, TokenKind::Symbol(Symbol::TripleEqual));
        }
    }
    false
}

// Recognize the paired empty brackets `[]` as the operator spelling `[]` in
// operator-name positions (binder, alias binder, entity-ref inner component).
// `[` followed by content is not the `[]` operator and is left untouched.
fn try_consume_bracket_operator_name(parser: &mut Parser<'_>) -> Option<OperatorNameAst> {
    if !parser.cursor.at_symbol(Symbol::LBracket) {
        return None;
    }
    let cursor_index = parser.cursor.current_index();
    let (_, rbracket) = parser.cursor.peek_at_skip_trivia(cursor_index + 1);
    if !matches!(rbracket.kind, TokenKind::Symbol(Symbol::RBracket)) {
        return None;
    }
    let lbracket_span = parser.cursor.bump_non_trivia().span;
    let rbracket_span = parser.cursor.bump_non_trivia().span;
    Some(OperatorNameAst {
        spelling: crate::OperatorSpelling::BracketCall
            .as_source_text()
            .to_string(),
        span: lbracket_span.join(rbracket_span),
    })
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
    if let Some(operator) = try_consume_bracket_operator_name(parser) {
        return Some(NavComponentAst::Operator(operator));
    }
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
    parser.cursor.is_form_boundary()
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
        BindingPatternAst::Implicit { span } => *span,
        BindingPatternAst::Product(product) => product.span,
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
        CanonicalSkeletonAst::ProductExtract { span, .. } => *span,
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
