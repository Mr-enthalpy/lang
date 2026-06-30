use crate::{
    AnnotationTermAst, BinderDeclAst, DeduceListAst, DiagnosticCode, NameAst, Span, Symbol,
    TokenKind,
};

use super::form::Parser;

pub fn parse_deduce_list(parser: &mut Parser<'_>) -> DeduceListAst {
    let less = parser
        .cursor
        .consume_symbol(Symbol::Less)
        .expect("parse_deduce_list called at `<`");

    parser.enter_nesting();
    let mut binders = Vec::new();

    loop {
        if parser.cursor.at_eof() || parser.cursor.at_symbol(Symbol::Greater) {
            break;
        }
        if parser.is_form_boundary() {
            break;
        }

        let token = parser.cursor.peek_non_trivia();
        if !matches!(token.kind, TokenKind::Name) {
            parser.error(
                DiagnosticCode::InvalidDeduceList,
                "expected binder name in deduce list",
                token.span,
            );
            recover_to_greater(parser);
            break;
        }

        let name_token = parser.cursor.bump_non_trivia();
        let name = NameAst {
            text: name_token.text.clone(),
            span: name_token.span,
        };
        let start_span = name.span;

        let annotation = if parser.cursor.consume_symbol(Symbol::Colon).is_some() {
            let annotation = parse_annotation_term_in_deduce(parser);
            Some(annotation)
        } else {
            None
        };

        let end_span = annotation
            .as_ref()
            .map(|a| annotation_term_span(a))
            .unwrap_or(name.span);

        binders.push(BinderDeclAst {
            name,
            annotation,
            span: start_span.join(end_span),
        });

        if parser.cursor.consume_symbol(Symbol::Comma).is_none() {
            break;
        }

        if parser.cursor.at_symbol(Symbol::Greater)
            || parser.cursor.at_eof()
            || parser.is_form_boundary()
        {
            let span = parser.cursor.current_span();
            parser.error(
                DiagnosticCode::InvalidDeduceList,
                "trailing comma in deduce list",
                span,
            );
            break;
        }
    }

    let end = if let Some(greater) = parser.cursor.consume_symbol(Symbol::Greater) {
        greater.span
    } else {
        let span = parser.cursor.current_span();
        parser.error(
            DiagnosticCode::InvalidDeduceList,
            "unclosed deduce list, expected `>`",
            less.span,
        );
        recover_to_binder_end(parser);
        span
    };

    parser.leave_nesting();
    DeduceListAst {
        binders,
        span: less.span.join(end),
    }
}

fn parse_annotation_term_in_deduce(parser: &mut Parser<'_>) -> AnnotationTermAst {
    if parser.cursor.at_name("_") {
        let next = parser.cursor.peek_next_non_trivia();
        if matches!(
            next.kind,
            TokenKind::Symbol(Symbol::Comma | Symbol::Greater)
        ) || matches!(next.kind, TokenKind::Eof)
        {
            let hole = parser.cursor.bump_non_trivia();
            return AnnotationTermAst::Hole { span: hole.span };
        }
    }

    if parser.cursor.at_symbol(Symbol::Comma)
        || parser.cursor.at_symbol(Symbol::Greater)
        || parser.cursor.at_eof()
        || parser.is_form_boundary()
    {
        let span = parser.cursor.current_span();
        parser.error(
            DiagnosticCode::InvalidDeduceList,
            "expected deduce annotation after `:`",
            span,
        );
        return AnnotationTermAst::Expr(super::product::error_expr(
            parser,
            "missing deduce annotation",
            span,
        ));
    }

    let expr = super::expr::parse_expr_until(parser, |p| {
        p.cursor.at_symbol(Symbol::Comma)
            || p.cursor.at_symbol(Symbol::Greater)
            || p.cursor.at_symbol(Symbol::Equal)
    });
    if super::form::expression_contains_name(&expr, "return") {
        parser.error(
            crate::DiagnosticCode::ReturnExpressionNotAllowed,
            "return is only allowed as a block terminal form",
            expr.span,
        );
    }
    AnnotationTermAst::Expr(expr)
}

fn annotation_term_span(annotation: &AnnotationTermAst) -> Span {
    match annotation {
        AnnotationTermAst::Expr(expr) => expr.span,
        AnnotationTermAst::Hole { span } => *span,
    }
}

fn recover_to_binder_end(parser: &mut Parser<'_>) {
    while !parser.cursor.at_eof()
        && !parser.cursor.at_symbol(Symbol::Equal)
        && !parser.is_form_boundary()
    {
        parser.cursor.bump_non_trivia();
    }
}

fn recover_to_greater(parser: &mut Parser<'_>) {
    while !parser.cursor.at_eof()
        && !parser.cursor.at_symbol(Symbol::Greater)
        && !parser.is_form_boundary()
    {
        parser.cursor.bump_non_trivia();
    }
    // Do not consume `>` here; let the main loop handle it.
}
