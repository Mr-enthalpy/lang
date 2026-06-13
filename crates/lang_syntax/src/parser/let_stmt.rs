use crate::{
    DeclAnnotationAst, DiagnosticCode, ExprAst, ExprKind, LetAst, LetAttrAst, LetBinderAst,
    NameAst, Span, Symbol, TokenKind, TypeObjectAnnotationAst,
};

use super::{expr::parse_expr_until, form::Parser};

pub fn parse_let(parser: &mut Parser<'_>) -> LetAst {
    let let_token = parser
        .cursor
        .consume_name("let")
        .expect("parse_let called at let");
    let mut attrs = Vec::new();

    while parser.cursor.consume_name("guard").is_some() {
        attrs.push(LetAttrAst::Guard);
    }

    let binder = parse_simple_binder(parser);
    let with_deps = parse_with_clause(parser);

    let value = if parser.cursor.consume_symbol(Symbol::Equal).is_some() {
        parse_expr_until(parser, |parser| parser.cursor.is_form_boundary())
    } else {
        let span = parser.cursor.current_span();
        parser.error(DiagnosticCode::ExpectedEqual, "expected `=` in let", span);
        parser.recover_to_form_boundary();
        error_expr(parser, "expected `=` in let", span)
    };

    let span = let_token.span.join(value.span);
    LetAst {
        attrs,
        binder,
        with_deps,
        value,
        span,
    }
}

// Future operator parser phase: this function must also accept operator binder
// names, so `let +: _: operator = expr` becomes a valid let form. Currently
// only Name tokens are accepted as binder names.
fn parse_simple_binder(parser: &mut Parser<'_>) -> LetBinderAst {
    let name_token = parser.cursor.peek_non_trivia();
    if !matches!(name_token.kind, TokenKind::Name) {
        let span = name_token.span;
        parser.error(
            DiagnosticCode::ExpectedName,
            "expected name after `let`",
            span,
        );
        return LetBinderAst::Error(parser.error_ast("expected name after `let`", span));
    }

    let name_token = parser.cursor.bump_non_trivia();
    let name = NameAst {
        text: name_token.text.clone(),
        span: name_token.span,
    };

    if parser.cursor.consume_symbol(Symbol::Colon).is_none() {
        let span = parser.cursor.current_span();
        parser.error(
            DiagnosticCode::ExpectedColon,
            "expected `:` after let binder name",
            span,
        );
        recover_to_equal(parser);
        let error = parser.error_ast("expected declaration annotation", span);
        return LetBinderAst::Simple {
            name,
            annotation: DeclAnnotationAst::Error(error),
            span: name_token.span.join(span),
        };
    }

    let annotation = parse_decl_annotation(parser);
    let end_span = decl_annotation_span(&annotation);

    LetBinderAst::Simple {
        name,
        annotation,
        span: name_token.span.join(end_span),
    }
}

fn parse_decl_annotation(parser: &mut Parser<'_>) -> DeclAnnotationAst {
    let start = parser.cursor.current_span();

    if parser.cursor.at_symbol(Symbol::Equal) || parser.cursor.at_name("with") {
        parser.error(
            DiagnosticCode::ExpectedDeclAnnotation,
            "expected declaration annotation",
            start,
        );
        return DeclAnnotationAst::Error(
            parser.error_ast("expected declaration annotation", start),
        );
    }

    if parser.cursor.at_name("_")
        && matches!(
            parser.cursor.peek_next_non_trivia().kind,
            TokenKind::Symbol(Symbol::Colon)
        )
    {
        let hole = parser.cursor.bump_non_trivia();
        parser.cursor.consume_symbol(Symbol::Colon);
        let rank_annotation = parse_expr_until(parser, annotation_stop);
        let span = hole.span.join(rank_annotation.span);
        return DeclAnnotationAst::TypeObjectWithRank {
            type_object_annotation: TypeObjectAnnotationAst::Hole { span: hole.span },
            rank_annotation,
            span,
        };
    }

    let type_or_bare = parse_expr_until(parser, |parser| {
        parser.cursor.at_symbol(Symbol::Colon) || annotation_stop(parser)
    });

    if parser.cursor.consume_symbol(Symbol::Colon).is_some() {
        let rank_annotation = parse_expr_until(parser, annotation_stop);
        let span = type_or_bare.span.join(rank_annotation.span);
        DeclAnnotationAst::TypeObjectWithRank {
            type_object_annotation: TypeObjectAnnotationAst::Expr(type_or_bare),
            rank_annotation,
            span,
        }
    } else {
        DeclAnnotationAst::Bare(type_or_bare)
    }
}

fn annotation_stop(parser: &mut Parser<'_>) -> bool {
    parser.cursor.at_symbol(Symbol::Equal) || parser.cursor.at_name("with")
}

fn parse_with_clause(parser: &mut Parser<'_>) -> Vec<NameAst> {
    let mut deps = Vec::new();
    if parser.cursor.consume_name("with").is_none() {
        return deps;
    }

    loop {
        let token = parser.cursor.peek_non_trivia();
        if !matches!(token.kind, TokenKind::Name) {
            parser.error(
                DiagnosticCode::ExpectedName,
                "expected name in with clause",
                token.span,
            );
            break;
        }

        let token = parser.cursor.bump_non_trivia();
        deps.push(NameAst {
            text: token.text.clone(),
            span: token.span,
        });

        if parser.cursor.consume_symbol(Symbol::Comma).is_none() {
            break;
        }
    }

    deps
}

fn recover_to_equal(parser: &mut Parser<'_>) {
    while !parser.cursor.is_form_boundary() && !parser.cursor.at_symbol(Symbol::Equal) {
        parser.cursor.bump_non_trivia();
    }
}

fn decl_annotation_span(annotation: &DeclAnnotationAst) -> Span {
    match annotation {
        DeclAnnotationAst::Bare(expr) => expr.span,
        DeclAnnotationAst::TypeObjectWithRank { span, .. } => *span,
        DeclAnnotationAst::Error(error) => error.span,
    }
}

fn error_expr(parser: &Parser<'_>, message: &str, span: Span) -> ExprAst {
    ExprAst {
        kind: ExprKind::Error(parser.error_ast(message, span)),
        span,
    }
}
