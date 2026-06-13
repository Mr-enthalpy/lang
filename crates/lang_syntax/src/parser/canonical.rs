use crate::{
    CanonicalNameRole, CanonicalSkeletonAst, DeduceListAst, DiagnosticCode, ErrorAst, NameAst,
    Symbol, TokenKind,
};

use super::form::Parser;

pub fn parse_canonical_skeleton(
    parser: &mut Parser<'_>,
    deduce: &DeduceListAst,
) -> CanonicalSkeletonAst {
    let start = parser.cursor.current_span();
    let mut elements = Vec::new();

    while !parser.cursor.at_eof()
        && !parser.cursor.at_symbol(Symbol::Equal)
        && !parser.cursor.at_name("with")
        && !parser.is_form_boundary()
    {
        if let Some(element) = parse_canonical_element(parser, deduce) {
            elements.push(element);
        } else {
            break;
        }
    }

    let end = parser.cursor.current_span();
    let span = start.join(end);

    if elements.is_empty() {
        let error = ErrorAst {
            message: "expected canonical skeleton element".to_string(),
            span,
        };
        parser.error(
            DiagnosticCode::InvalidCanonicalSkeleton,
            "expected canonical skeleton element",
            span,
        );
        recover_to_canonical_boundary(parser);
        return CanonicalSkeletonAst::Error(error);
    }

    if elements.len() == 1 {
        let mut elem = elements.into_iter().next().unwrap();
        if let CanonicalSkeletonAst::Segment { span: s, .. } = &mut elem {
            *s = span;
        }
        elem
    } else {
        CanonicalSkeletonAst::Segment { elements, span }
    }
}

fn parse_canonical_element(
    parser: &mut Parser<'_>,
    deduce: &DeduceListAst,
) -> Option<CanonicalSkeletonAst> {
    let token = parser.cursor.peek_non_trivia();

    match &token.kind {
        TokenKind::Symbol(Symbol::LParen) => Some(parse_canonical_argpack(parser, deduce)),
        TokenKind::Name if token.text == "_" => {
            let token = parser.cursor.bump_non_trivia();
            Some(CanonicalSkeletonAst::Wildcard { span: token.span })
        }
        TokenKind::Name => Some(parse_canonical_name_or_path(parser, deduce)),
        TokenKind::IntLiteral | TokenKind::StringLiteral => {
            let token = parser.cursor.bump_non_trivia();
            Some(CanonicalSkeletonAst::Literal {
                text: token.text.clone(),
                span: token.span,
            })
        }
        _ => None,
    }
}

fn parse_canonical_argpack(
    parser: &mut Parser<'_>,
    deduce: &DeduceListAst,
) -> CanonicalSkeletonAst {
    let lparen = parser
        .cursor
        .consume_symbol(Symbol::LParen)
        .expect("parse_canonical_argpack at `(`");

    parser.enter_nesting();
    let mut elements = Vec::new();

    loop {
        if parser.cursor.at_eof()
            || parser.cursor.at_symbol(Symbol::RParen)
            || parser.is_form_boundary()
        {
            break;
        }

        let element = parse_canonical_skeleton(parser, deduce);
        elements.push(element);

        if parser.cursor.consume_symbol(Symbol::Comma).is_none() {
            break;
        }

        if parser.cursor.at_symbol(Symbol::RParen)
            || parser.cursor.at_eof()
            || parser.is_form_boundary()
        {
            let span = parser.cursor.current_span();
            parser.error(
                DiagnosticCode::InvalidCanonicalSkeleton,
                "trailing comma in canonical argument pack",
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
            DiagnosticCode::InvalidCanonicalSkeleton,
            "unclosed canonical argument pack, expected `)`",
            lparen.span,
        );
        span
    };

    parser.leave_nesting();
    CanonicalSkeletonAst::ArgPack {
        elements,
        span: lparen.span.join(end),
    }
}

fn parse_canonical_name_or_path(
    parser: &mut Parser<'_>,
    deduce: &DeduceListAst,
) -> CanonicalSkeletonAst {
    let token = parser.cursor.bump_non_trivia();
    let name = NameAst {
        text: token.text.clone(),
        span: token.span,
    };

    let role = if is_in_deduce(deduce, &name.text) {
        CanonicalNameRole::Hole
    } else {
        CanonicalNameRole::NodeName
    };

    let mut path_names = Vec::new();
    let mut span = name.span;

    while parser.cursor.consume_symbol(Symbol::ColonColon).is_some() {
        let next = parser.cursor.peek_non_trivia();
        if !matches!(next.kind, TokenKind::Name) {
            parser.error(
                DiagnosticCode::ExpectedName,
                "expected name after `::` in canonical path",
                next.span,
            );
            break;
        }

        let next_token = parser.cursor.bump_non_trivia();
        span = span.join(next_token.span);
        path_names.push(NameAst {
            text: next_token.text.clone(),
            span: next_token.span,
        });
    }

    if path_names.is_empty() {
        CanonicalSkeletonAst::Name { name, role, span }
    } else {
        path_names.insert(
            0,
            NameAst {
                text: name.text.clone(),
                span: name.span,
            },
        );
        span = name.span.join(span);
        CanonicalSkeletonAst::Path {
            names: path_names,
            span,
        }
    }
}

fn is_in_deduce(deduce: &DeduceListAst, name: &str) -> bool {
    deduce.binders.iter().any(|b| b.name.text == name)
}

fn recover_to_canonical_boundary(parser: &mut Parser<'_>) {
    while !parser.cursor.at_eof()
        && !parser.cursor.at_symbol(Symbol::Equal)
        && !parser.cursor.at_name("with")
        && !parser.is_form_boundary()
    {
        parser.cursor.bump_non_trivia();
    }
}
