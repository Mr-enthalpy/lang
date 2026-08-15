use crate::{
    token::operator_spelling_in_expr_context, AliasBinderAst, AnnotationTermAst, BinderNameAst,
    BindingAnnotationAst, BindingPatternAst, BindingSlotAst, CanonicalSkeletonAst, DeduceListAst,
    DiagnosticCode, EntityRefAst, ErrorAst, ExprAst, ExprKind, FormAst, LetAliasAst, LetAst,
    NameAst, NavComponentAst, OperatorNameAst, PolicySpecAst, ProductExtractAst,
    ProductExtractElementAst, Span, Symbol, TokenKind, WithClauseAst, WithClauseKind,
};

use super::{
    atom::parse_nav_group_component, canonical::parse_canonical_skeleton,
    deduce::parse_deduce_list, expr::parse_expr_until, form::Parser,
    policy::try_parse_policy_spec_before_let,
};

#[derive(Clone, Copy)]
pub enum BindingSlotContext {
    Let,
    Capture,
    Param,
    Return,
}

pub fn parse_let_form(parser: &mut Parser<'_>, policy: Option<PolicySpecAst>) -> FormAst {
    let start = policy
        .as_ref()
        .map(|policy| policy.span)
        .unwrap_or_else(|| parser.cursor.current_span());
    parser
        .cursor
        .consume_name("let")
        .expect("parse_let_form called at let");

    if alias_binder_followed_by_triple_equal(parser) {
        return FormAst::AliasLet(parse_alias_let_body(parser, start, policy));
    }

    let inherited_deduce = parser.active_deduce_list();
    let mut slot = parse_binding_slot(
        parser,
        BindingSlotContext::Let,
        inherited_deduce.as_ref(),
        true,
    );
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

    if has_let
        && !matches!(context, BindingSlotContext::Let)
        && looks_like_alias_binding_start(parser)
    {
        let span = parser.cursor.current_span();
        parser.error(
            DiagnosticCode::InvalidAliasPosition,
            "alias binding must appear as a standalone form",
            span,
        );
        if matches!(context, BindingSlotContext::Capture) {
            while !parser.cursor.at_eof()
                && !parser.cursor.at_symbol(Symbol::Comma)
                && !parser.cursor.at_symbol(Symbol::RBracket)
            {
                parser.cursor.bump_non_trivia();
            }
        } else {
            parser.recover_to_form_boundary();
        }
        let end = parser.cursor.current_span();
        return BindingSlotAst {
            policy: None,
            has_let: false,
            deduce: None,
            pattern: BindingPatternAst::Error(
                parser.error_ast("alias binding must appear as a standalone form", span),
            ),
            annotation: None,
            with_clause: None,
            initializer: None,
            span: start.join(end),
        };
    }

    let has_deduce = starts_binding_deduce_list(parser);
    let binderless = has_deduce
        && matches!(
            parser.cursor.peek_next_non_trivia().kind,
            TokenKind::Symbol(Symbol::Greater)
        );
    let deduce = if has_deduce {
        Some(parse_deduce_list(parser))
    } else {
        None
    };

    parse_binding_slot_after_prefix(
        parser,
        context,
        inherited_deduce,
        require_initializer,
        start,
        policy,
        has_let,
        binderless,
        deduce,
    )
}

/// Parse the atomic Pattern of pipe branch shorthand through the same
/// BindingSlot path as an explicitly written `(<> P)` parameter.
///
/// The empty DeduceList is semantic, not decorative: it selects a binderless
/// Pattern position. The caller is responsible for proving that the current
/// token is the one atomic Pattern admitted by branch shorthand.
pub(super) fn parse_synthesized_empty_deduce_binding_slot(
    parser: &mut Parser<'_>,
    context: BindingSlotContext,
) -> BindingSlotAst {
    let start = parser.cursor.current_span();
    let inherited_deduce = parser.active_deduce_list();
    let deduce = Some(DeduceListAst {
        binders: Vec::new(),
        span: start,
    });

    parse_binding_slot_after_prefix(
        parser,
        context,
        inherited_deduce.as_ref(),
        false,
        start,
        None,
        false,
        true,
        deduce,
    )
}

#[allow(clippy::too_many_arguments)]
fn parse_binding_slot_after_prefix(
    parser: &mut Parser<'_>,
    context: BindingSlotContext,
    inherited_deduce: Option<&DeduceListAst>,
    require_initializer: bool,
    start: Span,
    policy: Option<PolicySpecAst>,
    has_let: bool,
    binderless: bool,
    deduce: Option<DeduceListAst>,
) -> BindingSlotAst {
    let local_deduce_scope_mark = parser.push_deduce_scope(deduce.as_ref());

    let active_deduce = merge_active_deduce(inherited_deduce, deduce.as_ref(), start);
    let pattern = parse_binding_pattern(parser, context, has_let, binderless, Some(&active_deduce));
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

    let initializer = match context {
        BindingSlotContext::Let => Some(parse_let_value(parser, require_initializer)),
        BindingSlotContext::Capture => Some(parse_capture_value(parser, require_initializer)),
        BindingSlotContext::Param | BindingSlotContext::Return => None,
    };
    if let Some(initializer) = &initializer {
        end = initializer.span;
    }

    let slot = BindingSlotAst {
        policy,
        has_let,
        deduce,
        pattern,
        annotation,
        with_clause,
        initializer,
        span: start.join(end),
    };
    parser.restore_deduce_scope(local_deduce_scope_mark);
    slot
}

pub fn parse_product_extract(
    parser: &mut Parser<'_>,
    element_context: BindingSlotContext,
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
            if parser.cursor.at_symbol(Symbol::RParen)
                || parser.cursor.at_eof()
                || parser.is_form_boundary()
            {
                elements.push(ProductExtractElementAst::Unit { span: comma.span });
                break;
            }
            continue;
        }

        let element = parse_binding_slot(parser, element_context, inherited_deduce, false);
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
) -> (Option<PolicySpecAst>, bool) {
    if matches!(context, BindingSlotContext::Let) {
        return (None, false);
    }

    if parser.cursor.at_name("let") {
        parser.cursor.bump_non_trivia();
        return (None, true);
    }

    if let Some(policy) =
        try_parse_policy_spec_before_let(parser, |p| slot_policy_boundary(p, context))
    {
        parser.cursor.bump_non_trivia();
        (Some(policy), true)
    } else {
        (None, false)
    }
}

fn slot_policy_boundary(parser: &mut Parser<'_>, context: BindingSlotContext) -> bool {
    if parser.is_form_boundary() {
        return true;
    }
    match context {
        BindingSlotContext::Param => {
            parser.cursor.at_symbol(Symbol::Comma) || parser.cursor.at_symbol(Symbol::RParen)
        }
        BindingSlotContext::Return => {
            super::closure::at_callable_implementation_tail(parser) || parser.cursor.at_name("with")
        }
        BindingSlotContext::Capture => {
            parser.cursor.at_symbol(Symbol::Equal)
                || parser.cursor.at_symbol(Symbol::Comma)
                || parser.cursor.at_symbol(Symbol::RBracket)
        }
        BindingSlotContext::Let => true,
    }
}

fn parse_binding_pattern(
    parser: &mut Parser<'_>,
    context: BindingSlotContext,
    _has_let: bool,
    binderless: bool,
    active_deduce: Option<&DeduceListAst>,
) -> BindingPatternAst {
    let token = parser.cursor.peek_non_trivia();

    if parser.cursor.at_symbol(Symbol::Ellipsis) {
        let empty_deduce;
        let deduce_ref = match active_deduce {
            Some(deduce) => deduce,
            None => {
                empty_deduce = DeduceListAst {
                    binders: vec![],
                    span: parser.cursor.current_span(),
                };
                &empty_deduce
            }
        };
        let skeleton = parse_canonical_skeleton(parser, deduce_ref);
        return canonical_pack_or_sequence_binding_pattern(skeleton);
    }

    if at_binding_pattern_boundary(parser, context) {
        let message = match context {
            BindingSlotContext::Let => "expected binding pattern after `let`",
            BindingSlotContext::Capture => "expected capture binding pattern",
            BindingSlotContext::Param => "expected parameter binding pattern",
            BindingSlotContext::Return => "expected return binding pattern after `->`",
        };
        parser.error(DiagnosticCode::ExpectedName, message, token.span);
        return BindingPatternAst::Error(parser.error_ast(message, token.span));
    }

    if parser.cursor.at_symbol(Symbol::LParen) {
        let element_context = match context {
            BindingSlotContext::Return => BindingSlotContext::Return,
            BindingSlotContext::Let | BindingSlotContext::Capture | BindingSlotContext::Param => {
                BindingSlotContext::Param
            }
        };
        return BindingPatternAst::Product(parse_product_extract(
            parser,
            element_context,
            active_deduce,
        ));
    }

    if binderless
        || starts_skeleton_name(parser, context)
        || matches!(token.kind, TokenKind::Name if token.text == "_")
        || matches!(token.kind, TokenKind::IntLiteral | TokenKind::StringLiteral)
    {
        let empty_deduce;
        let deduce_ref = match active_deduce {
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
        BindingSlotContext::Capture => "expected capture binding pattern",
        BindingSlotContext::Param => "expected parameter binding pattern",
        BindingSlotContext::Return => "expected return binding pattern after `->`",
    };
    parser.error(DiagnosticCode::ExpectedName, message, token.span);
    BindingPatternAst::Error(parser.error_ast(message, token.span))
}

fn merge_active_deduce(
    inherited: Option<&DeduceListAst>,
    local: Option<&DeduceListAst>,
    fallback_span: Span,
) -> DeduceListAst {
    let mut binders = Vec::new();
    if let Some(inherited) = inherited {
        binders.extend(inherited.binders.iter().cloned());
    }
    if let Some(local) = local {
        binders.extend(local.binders.iter().cloned());
    }
    let span = match (inherited, local) {
        (Some(inherited), Some(local)) => inherited.span.join(local.span),
        (Some(inherited), None) => inherited.span,
        (None, Some(local)) => local.span,
        (None, None) => fallback_span,
    };
    DeduceListAst { binders, span }
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
    ) {
        let (current_index, _) = parser
            .cursor
            .peek_at_skip_trivia(parser.cursor.current_index());
        if super::closure::token_index_starts_closure_head_continuation(parser, current_index + 1) {
            return false;
        }
    }
    !is_binding_pattern_stop_kind(&next.kind, context)
}

fn at_binding_pattern_boundary(parser: &mut Parser<'_>, context: BindingSlotContext) -> bool {
    if matches!(context, BindingSlotContext::Return)
        && super::closure::at_callable_implementation_tail(parser)
    {
        return true;
    }
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
        TokenKind::Symbol(Symbol::RBracket) if matches!(context, BindingSlotContext::Capture) => {
            true
        }
        TokenKind::Symbol(Symbol::Equal)
            if matches!(
                context,
                BindingSlotContext::Let | BindingSlotContext::Capture
            ) =>
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
        if super::form::expression_contains_name(&right, "return") {
            parser.error(
                DiagnosticCode::ReturnExpressionNotAllowed,
                "return is only allowed as a block terminal form",
                right.span,
            );
        }
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
    if super::form::expression_contains_name(&left_or_expr, "return") {
        parser.error(
            DiagnosticCode::ReturnExpressionNotAllowed,
            "return is only allowed as a block terminal form",
            left_or_expr.span,
        );
    }

    if parser.cursor.consume_symbol(Symbol::Colon).is_some() {
        let right = parse_expr_until(parser, |p| annotation_stop(p, context));
        if super::form::expression_contains_name(&right, "return") {
            parser.error(
                DiagnosticCode::ReturnExpressionNotAllowed,
                "return is only allowed as a block terminal form",
                right.span,
            );
        }
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
        || (matches!(context, BindingSlotContext::Capture)
            && parser.cursor.at_symbol(Symbol::RBracket))
        || parser.cursor.at_symbol(Symbol::FatArrow)
        || parser.cursor.at_symbol(Symbol::LBrace)
        || (matches!(context, BindingSlotContext::Return)
            && super::closure::at_callable_implementation_tail(parser))
        || (matches!(
            context,
            BindingSlotContext::Let | BindingSlotContext::Capture
        ) && parser.cursor.at_symbol(Symbol::Equal))
        || (matches!(
            context,
            BindingSlotContext::Param | BindingSlotContext::Return
        ) && super::closure::at_head_clause_keyword(parser))
        || parser.is_form_boundary()
}

fn parse_let_value(parser: &mut Parser<'_>, require_initializer: bool) -> ExprAst {
    if parser.cursor.consume_symbol(Symbol::Equal).is_some() {
        let expr = parse_expr_until(parser, |parser| parser.is_form_boundary());
        if super::form::expression_contains_name(&expr, "return") {
            parser.error(
                DiagnosticCode::ReturnExpressionNotAllowed,
                "return is only allowed as a block terminal form",
                expr.span,
            );
        }
        expr
    } else {
        let span = parser.cursor.current_span();
        if require_initializer {
            parser.error(DiagnosticCode::ExpectedEqual, "expected `=` in let", span);
            parser.recover_to_form_boundary();
        }
        error_expr(parser, "expected `=` in let", span)
    }
}

fn parse_capture_value(parser: &mut Parser<'_>, require_initializer: bool) -> ExprAst {
    if parser.cursor.consume_symbol(Symbol::Equal).is_some() {
        let expr = parse_expr_until(parser, |parser| {
            parser.cursor.at_symbol(Symbol::Comma) || parser.cursor.at_symbol(Symbol::RBracket)
        });
        if super::form::expression_contains_name(&expr, "return") {
            parser.error(
                DiagnosticCode::ReturnExpressionNotAllowed,
                "return is only allowed as a block terminal form",
                expr.span,
            );
        }
        expr
    } else {
        let span = parser.cursor.current_span();
        if require_initializer {
            parser.error(
                DiagnosticCode::ExpectedEqual,
                "expected `=` in capture binding",
                span,
            );
            while !parser.cursor.at_eof()
                && !parser.cursor.at_symbol(Symbol::Comma)
                && !parser.cursor.at_symbol(Symbol::RBracket)
            {
                parser.cursor.bump_non_trivia();
            }
        }
        error_expr(parser, "expected `=` in capture binding", span)
    }
}

fn parse_alias_let_body(
    parser: &mut Parser<'_>,
    span_start: Span,
    policy: Option<PolicySpecAst>,
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

pub(super) fn looks_like_alias_binding_start(parser: &mut Parser<'_>) -> bool {
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
        BindingPatternAst::Product(product) => product.span,
        BindingPatternAst::Pack { span, .. } => *span,
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
        CanonicalSkeletonAst::Pack { span, .. } => *span,
        CanonicalSkeletonAst::ProductExtract { span, .. } => *span,
        CanonicalSkeletonAst::Wildcard { span } => *span,
        CanonicalSkeletonAst::Name { span, .. } => *span,
        CanonicalSkeletonAst::NavPath { span, .. } => *span,
        CanonicalSkeletonAst::Literal { span, .. } => *span,
        CanonicalSkeletonAst::Error(error) => error.span,
    }
}

fn canonical_pack_or_sequence_binding_pattern(skeleton: CanonicalSkeletonAst) -> BindingPatternAst {
    match skeleton {
        CanonicalSkeletonAst::Pack { inner, span } => BindingPatternAst::Pack {
            inner: Box::new(canonical_pack_inner_binding_pattern(*inner)),
            span,
        },
        other => BindingPatternAst::Skeleton(other),
    }
}

fn canonical_pack_inner_binding_pattern(skeleton: CanonicalSkeletonAst) -> BindingPatternAst {
    match skeleton {
        CanonicalSkeletonAst::Pack { inner, span } => BindingPatternAst::Pack {
            inner: Box::new(canonical_pack_inner_binding_pattern(*inner)),
            span,
        },
        CanonicalSkeletonAst::Name { name, role, .. } if role != crate::CanonicalNameRole::Hole => {
            BindingPatternAst::Binder(BinderNameAst::Text(name))
        }
        other => BindingPatternAst::Skeleton(other),
    }
}

fn error_expr(parser: &Parser<'_>, message: &str, span: Span) -> ExprAst {
    ExprAst {
        kind: ExprKind::Error(parser.error_ast(message, span)),
        span,
    }
}
