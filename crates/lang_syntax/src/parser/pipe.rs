use crate::{
    DiagnosticCode, ExprAst, ExprKind, OperatorExprAst, OperatorExprKind, PipeExprAst, SegmentAst,
    SegmentElementAst, Span, Symbol, TokenKind,
};

use super::{
    cursor::ParenClassification, form::Parser, operator::parse_operator_expr,
    product::parse_product_expr,
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

    loop {
        let seg = parse_segment(parser, |p| {
            p.is_form_boundary() || p.cursor.is_at_pipe_element() || stop(p)
        });
        segments.push(seg);

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
            continue;
        }
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
) -> SegmentAst {
    let start = parser.cursor.current_span();
    let mut elements = Vec::new();

    while !stop(parser) {
        if parser.cursor.at_eof() {
            break;
        }

        if let Some(element) = parse_segment_element(parser, &mut stop) {
            elements.push(element);
        } else if stop(parser) {
            break;
        } else {
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

fn parse_segment_element(
    parser: &mut Parser<'_>,
    stop: &mut impl FnMut(&mut Parser<'_>) -> bool,
) -> Option<SegmentElementAst> {
    let (class, after_idx) = parser.cursor.classify_paren_at_segment_position();

    match class {
        ParenClassification::Product => {
            // Check if this is a closure param clause before parsing as Product.
            // A `(...)` followed by `=>`, `->`, `{`, or a head-clause keyword
            // (`require`/`pre`/`post`/`lifetime pre`/`lifetime post`) is a
            // closure-head parameter clause rather than a product expression.
            if let Some(idx) = after_idx {
                let (_, after) = parser.cursor.peek_at_skip_trivia(idx);
                if matches!(
                    after.kind,
                    TokenKind::Symbol(Symbol::FatArrow | Symbol::LBrace | Symbol::ThinArrow)
                ) || super::closure::token_index_starts_head_clause(parser, idx)
                {
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

fn element_span(element: &SegmentElementAst) -> Span {
    match element {
        SegmentElementAst::OperatorExpr(op_expr) => op_expr.span,
        SegmentElementAst::Product(product) => product.span,
    }
}
