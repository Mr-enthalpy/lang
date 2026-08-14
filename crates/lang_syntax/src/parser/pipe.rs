use crate::{
    AtomAst, AtomKind, BinderNameAst, BindingPatternAst, BindingSlotAst, BodyBlockAst, ClosureAst,
    ClosureBodyAst, ClosurePlacementAst, DeduceListAst, DiagnosticCode, ExprAst, ExprKind,
    FnHeadPrefixAst, NameAst, OperatorExprAst, OperatorExprKind, ParamClauseAst, PipeExprAst,
    ProductExtractAst, ProductExtractElementAst, SegmentAst, SegmentElementAst, Span, Symbol,
    TokenKind,
};

use super::{
    closure::parse_body_block, cursor::ParenClassification, form::Parser,
    operator::parse_operator_expr, product::parse_product_expr,
};

pub fn parse_pipe_expr(
    parser: &mut Parser<'_>,
    mut stop: impl FnMut(&mut Parser<'_>) -> bool,
) -> ExprAst {
    let start = parser.cursor.current_span();
    let mut segments = Vec::new();

    if parser.cursor.is_at_pipe_element() {
        let span = parser.cursor.current_span();
        parser.error(DiagnosticCode::EmptyPipeSegment, "empty pipe segment", span);
        parser.cursor.bump_non_trivia();
        segments.push(empty_error_segment(
            parser,
            "unexpected `|>` at expression start",
            span,
        ));
    }

    let seg = parse_segment(
        parser,
        |p| p.is_form_boundary() || p.cursor.is_at_pipe_element() || stop(p),
        false,
    );
    segments.push(seg);

    loop {
        if stop(parser) || parser.is_form_boundary() {
            break;
        }

        if !parser.cursor.consume_symbol(Symbol::PipeGreater).is_some() {
            break;
        }

        if stop(parser) || parser.is_form_boundary() {
            let span = parser.cursor.current_span();
            parser.error(
                DiagnosticCode::EmptyPipeSegment,
                "empty pipe segment after `|>`",
                span,
            );
            segments.push(empty_error_segment(parser, "empty pipe segment", span));
            break;
        }

        if parser.cursor.is_at_pipe_element() {
            let pipe_span = parser.cursor.current_span();
            parser.error(
                DiagnosticCode::EmptyPipeSegment,
                "empty pipe segment after `|>`",
                pipe_span,
            );
            parser.cursor.bump_non_trivia();
            segments.push(empty_error_segment(parser, "empty pipe segment", pipe_span));
            if stop(parser) || parser.is_form_boundary() || parser.cursor.is_at_pipe_element() {
                continue;
            }
            let seg = parse_segment(
                parser,
                |p| p.is_form_boundary() || p.cursor.is_at_pipe_element() || stop(p),
                true,
            );
            segments.push(seg);
            continue;
        }

        let seg = parse_segment(
            parser,
            |p| p.is_form_boundary() || p.cursor.is_at_pipe_element() || stop(p),
            true,
        );
        segments.push(seg);
    }

    for (i, segment) in segments.iter_mut().enumerate() {
        segment.has_incoming = i > 0;
    }

    let span = segments
        .first()
        .map(|s| s.span)
        .unwrap_or(start)
        .join(segments.last().map(|s| s.span).unwrap_or(start));

    if segments.len() == 1 && segments[0].elements.len() == 1 {
        if let SegmentElementAst::Product(product) = &segments[0].elements[0] {
            return ExprAst {
                kind: ExprKind::Product(product.clone()),
                span: product.span,
            };
        }
    }

    ExprAst {
        kind: ExprKind::Pipe(PipeExprAst { segments, span }),
        span,
    }
}

fn parse_segment(
    parser: &mut Parser<'_>,
    mut stop: impl FnMut(&mut Parser<'_>) -> bool,
    allow_pipe_branch_sugar: bool,
) -> SegmentAst {
    let start = parser.cursor.current_span();
    let mut elements = Vec::new();
    let mut has_product_head = false;
    let mut current_headless_closure_diagnosed = false;

    while !stop(parser) {
        if parser.cursor.at_eof() {
            break;
        }

        if allow_pipe_branch_sugar && elements.is_empty() {
            if let Some(branch_elements) = try_parse_pipe_branch_sugar(parser) {
                has_product_head = branch_elements
                    .iter()
                    .any(|element| matches!(element, SegmentElementAst::Product(_)));
                elements.extend(branch_elements);
                continue;
            }
            if let Some(branch_elements) = try_parse_incoming_product_branch(parser) {
                has_product_head = branch_elements
                    .iter()
                    .any(|element| matches!(element, SegmentElementAst::Product(_)));
                elements.extend(branch_elements);
                continue;
            }
            if parser.cursor.at_symbol(Symbol::LBrace) {
                let span = parser.cursor.current_span();
                parser.error(
                    DiagnosticCode::InvalidClosureHead,
                    "pipe branch body requires an explicit extraction head",
                    span,
                );
                current_headless_closure_diagnosed = true;
            }
        }

        if let Some(element) = parse_segment_element(parser, &mut stop) {
            if allow_pipe_branch_sugar
                && !has_product_head
                && is_headless_in_place_closure_segment_element(&element)
            {
                if !current_headless_closure_diagnosed {
                    diagnose_unheaded_incoming_closure(parser, &element);
                }
                current_headless_closure_diagnosed = false;
            }
            if matches!(element, SegmentElementAst::Product(_)) {
                has_product_head = true;
            }
            elements.push(element);
        } else if stop(parser) {
            break;
        } else {
            current_headless_closure_diagnosed = false;
            let token = parser.cursor.peek_non_trivia();
            if matches!(token.kind, TokenKind::Eof) {
                break;
            }
            if matches!(token.kind, TokenKind::Symbol(Symbol::Comma)) {
                let span = token.span;
                parser.cursor.bump_non_trivia();
                parser.error(
                    DiagnosticCode::TopLevelComma,
                    "unexpected top-level comma",
                    span,
                );
            } else {
                parser.unexpected_current();
            }
        }
    }

    let span = elements
        .first()
        .map(|e| element_span(e))
        .unwrap_or(start)
        .join(elements.last().map(|e| element_span(e)).unwrap_or(start));

    SegmentAst {
        elements,
        has_incoming: false,
        span: if span.byte_end >= start.byte_start {
            span
        } else {
            start
        },
    }
}

fn try_parse_pipe_branch_sugar(parser: &mut Parser<'_>) -> Option<Vec<SegmentElementAst>> {
    try_parse_bare_name_pipe_branch_sugar(parser)
}

fn try_parse_incoming_product_branch(parser: &mut Parser<'_>) -> Option<Vec<SegmentElementAst>> {
    let start_index = parser.cursor.current_index();
    let (_, open) = parser.cursor.peek_at_skip_trivia(start_index);
    if !matches!(open.kind, TokenKind::Symbol(Symbol::LParen)) {
        return None;
    }

    let (_, after_idx) = parser.cursor.classify_paren_at_segment_position();
    let after_idx = after_idx?;
    let (_, body_start) = parser.cursor.peek_at_skip_trivia(after_idx);
    if !matches!(body_start.kind, TokenKind::Symbol(Symbol::LBrace)) {
        return None;
    }

    parser.cursor.set_index(start_index);
    let product = parse_product_expr(parser);
    let closure = pipe_branch_body(parser);
    Some(vec![SegmentElementAst::Product(product), closure])
}

fn try_parse_bare_name_pipe_branch_sugar(
    parser: &mut Parser<'_>,
) -> Option<Vec<SegmentElementAst>> {
    let start_index = parser.cursor.current_index();
    let (_, name_token) = parser.cursor.peek_at_skip_trivia(start_index);
    if !matches!(name_token.kind, TokenKind::Name) {
        return None;
    }

    let (after_name_index, after_name) = parser.cursor.peek_at_skip_trivia(start_index + 1);
    if !matches!(after_name.kind, TokenKind::Symbol(Symbol::LBrace)) {
        return None;
    }

    parser.cursor.set_index(start_index);
    let name = parser.cursor.bump_non_trivia();
    parser.cursor.set_index(after_name_index);
    let body = parse_body_block(parser);
    Some(vec![pipe_branch_headed_closure(
        name.text.clone(),
        name.span,
        body,
    )])
}

fn pipe_branch_body(parser: &mut Parser<'_>) -> SegmentElementAst {
    let body = parse_body_block(parser);
    let span = body.span;
    SegmentElementAst::OperatorExpr(OperatorExprAst {
        kind: OperatorExprKind::Atom(AtomAst {
            kind: AtomKind::Closure(ClosureAst {
                placement: ClosurePlacementAst::InPlace,
                head: None,
                body: ClosureBodyAst::Block(body),
                span,
            }),
            span,
        }),
        span,
    })
}

/// Bare pipe branch-name shorthand `|> name { ... }` desugars to a single
/// closure element whose head carries a slot-level empty deduce list plus the
/// binding name — `|> (<> name) { ... }` / `|> (let <> name) { ... }`. The
/// closure is `Ordinary` so it matches the established explicit param-closure
/// shape (`pipe_explicit_param_closure`), keeping the v0.2 closure projection
/// contract intact.
fn pipe_branch_headed_closure(
    name_text: String,
    name_span: Span,
    body: BodyBlockAst,
) -> SegmentElementAst {
    let body_span = body.span;
    let slot = BindingSlotAst {
        policy: None,
        has_let: false,
        deduce: Some(DeduceListAst {
            binders: Vec::new(),
            span: name_span,
        }),
        pattern: BindingPatternAst::Binder(BinderNameAst::Text(NameAst {
            text: name_text,
            span: name_span,
        })),
        annotation: None,
        with_clause: None,
        initializer: None,
        span: name_span,
    };
    let head = FnHeadPrefixAst {
        deduce: None,
        captures: None,
        params: Some(ParamClauseAst {
            extract: ProductExtractAst {
                elements: vec![ProductExtractElementAst::Slot(slot)],
                span: name_span,
            },
            span: name_span,
        }),
        call_policy: None,
        returns: None,
        clauses: Vec::new(),
        span: name_span,
    };
    let closure = ClosureAst {
        placement: ClosurePlacementAst::Ordinary,
        head: Some(head),
        body: ClosureBodyAst::Block(body),
        span: name_span.join(body_span),
    };
    let span = closure.span;
    SegmentElementAst::OperatorExpr(OperatorExprAst {
        kind: OperatorExprKind::Atom(AtomAst {
            kind: AtomKind::Closure(closure),
            span,
        }),
        span,
    })
}

fn parse_segment_element(
    parser: &mut Parser<'_>,
    stop: &mut impl FnMut(&mut Parser<'_>) -> bool,
) -> Option<SegmentElementAst> {
    let (class, after_idx) = parser.cursor.classify_paren_at_segment_position();

    match class {
        ParenClassification::Product => {
            // Check if this is a closure param clause before parsing as Product.
            // A `(...)` followed by any centralized closure-head continuation
            // is a parameter clause rather than an ordinary Product.
            if let Some(idx) = after_idx {
                if super::closure::token_index_starts_closure_head_continuation(parser, idx) {
                    let op_expr = parse_operator_expr(parser, stop)?;
                    return Some(SegmentElementAst::OperatorExpr(op_expr));
                }
                let (_, after) = parser.cursor.peek_at_skip_trivia(idx);
                if matches!(after.kind, TokenKind::Symbol(Symbol::LBracket)) {
                    // A Product followed by `[...]` remains in the ordinary
                    // postfix-expression path. This preserves cases such as
                    // `()[[capture] => { ... }]`; only a complete
                    // `[[Name]] {` continuation above proves a closure head.
                    let op_expr = parse_operator_expr(parser, stop)?;
                    return Some(SegmentElementAst::OperatorExpr(op_expr));
                }
            }
            let product = parse_product_expr(parser);
            Some(SegmentElementAst::Product(product))
        }
        ParenClassification::Group => {
            let op_expr = parse_operator_expr(parser, stop)?;
            Some(SegmentElementAst::OperatorExpr(op_expr))
        }
        ParenClassification::Unclosed => {
            let product = parse_product_expr(parser);
            Some(SegmentElementAst::Product(product))
        }
        ParenClassification::NotParen => {
            parse_operator_expr(parser, stop).map(SegmentElementAst::OperatorExpr)
        }
    }
}

fn empty_error_segment(parser: &Parser<'_>, message: &str, span: Span) -> SegmentAst {
    SegmentAst {
        elements: vec![SegmentElementAst::OperatorExpr(OperatorExprAst {
            kind: OperatorExprKind::Error(parser.error_ast(message, span)),
            span,
        })],
        has_incoming: false,
        span,
    }
}

fn is_headless_in_place_closure_segment_element(element: &SegmentElementAst) -> bool {
    match element {
        SegmentElementAst::OperatorExpr(op_expr) => {
            operator_expr_contains_headless_in_place_closure(op_expr)
        }
        SegmentElementAst::Product(_) => false,
    }
}

fn operator_expr_contains_headless_in_place_closure(op_expr: &OperatorExprAst) -> bool {
    match &op_expr.kind {
        OperatorExprKind::Atom(atom) => matches!(
            atom.kind,
            AtomKind::Closure(ClosureAst {
                placement: ClosurePlacementAst::InPlace,
                head: None,
                ..
            })
        ),
        OperatorExprKind::Product(_) => false,
        OperatorExprKind::OperatorSugar { args, .. } => args
            .iter()
            .any(operator_expr_contains_headless_in_place_closure),
        OperatorExprKind::MemberSugar { object, .. }
        | OperatorExprKind::DoubleDotSugar { object, .. }
        | OperatorExprKind::BracketCallSugar { object, .. } => {
            operator_expr_contains_headless_in_place_closure(object)
        }
        OperatorExprKind::NavPath { .. } | OperatorExprKind::Error(_) => false,
    }
}

fn diagnose_unheaded_incoming_closure(parser: &mut Parser<'_>, element: &SegmentElementAst) {
    parser.error(
        DiagnosticCode::InvalidClosureHead,
        "pipe branch body requires an explicit extraction head",
        element_span(element),
    );
}

fn element_span(element: &SegmentElementAst) -> Span {
    match element {
        SegmentElementAst::OperatorExpr(op_expr) => op_expr.span,
        SegmentElementAst::Product(product) => product.span,
    }
}
