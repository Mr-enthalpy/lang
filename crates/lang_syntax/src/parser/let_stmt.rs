use crate::{
    token::operator_spelling_in_expr_context, AliasBinderAst, BinderNameAst, DeclAnnotationAst,
    DiagnosticCode, EntityPathLeafAst, EntityPathSegmentAst, EntityRefAst, ErrorAst, ExprAst,
    ExprKind, FormAst, LetAliasAst, LetAst, LetBinderAst, NameAst, OperatorNameAst, Span, Symbol,
    TokenKind, TypeObjectAnnotationAst, WithClauseAst, WithClauseKind,
};

use super::{
    canonical::parse_canonical_skeleton, deduce::parse_deduce_list, expr::parse_expr_until,
    form::Parser,
};

pub fn parse_let_form(parser: &mut Parser<'_>) -> FormAst {
    let let_token = parser
        .cursor
        .consume_name("let")
        .expect("parse_let_form called at let");

    if parser.cursor.at_symbol(Symbol::Less) {
        let binder = parse_extract_binder(parser);
        let with_clause = parse_with_clause(parser);
        let value = parse_let_value(parser);
        let span = let_token.span.join(value.span);
        return FormAst::Let(LetAst {
            binder,
            with_clause,
            value,
            span,
        });
    }

    let token = parser.cursor.peek_non_trivia();
    if is_valid_alias_binder(&token.kind) {
        let next = parser.cursor.peek_next_non_trivia();
        if matches!(next.kind, TokenKind::Symbol(Symbol::TripleEqual)) {
            return FormAst::AliasLet(parse_alias_let_body(parser, let_token.span));
        }
    }

    let binder = parse_let_binder(parser);
    let with_clause = parse_with_clause(parser);
    let value = parse_let_value(parser);
    let span = let_token.span.join(value.span);
    FormAst::Let(LetAst {
        binder,
        with_clause,
        value,
        span,
    })
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

fn parse_let_value(parser: &mut Parser<'_>) -> ExprAst {
    if parser.cursor.consume_symbol(Symbol::Equal).is_some() {
        parse_expr_until(parser, |parser| parser.is_form_boundary())
    } else {
        let span = parser.cursor.current_span();
        parser.error(DiagnosticCode::ExpectedEqual, "expected `=` in let", span);
        parser.recover_to_form_boundary();
        error_expr(parser, "expected `=` in let", span)
    }
}

fn is_valid_alias_binder(kind: &TokenKind) -> bool {
    matches!(kind, TokenKind::Name) || operator_spelling_in_expr_context(kind).is_some()
}

fn parse_entity_ref(parser: &mut Parser<'_>) -> EntityRefAst {
    let start = parser.cursor.current_raw_span();
    let mut path: Vec<EntityPathSegmentAst> = Vec::new();

    if is_entity_ref_boundary(parser) {
        parser.error(
            DiagnosticCode::ExpectedAliasTarget,
            "expected entity reference after `===`",
            start,
        );
        let leaf = EntityPathLeafAst::Error(parser.error_ast("expected entity reference", start));
        return EntityRefAst {
            path,
            leaf,
            span: start,
        };
    }

    loop {
        let token = parser.cursor.peek_non_trivia();

        match &token.kind {
            TokenKind::Name => {
                let name_token = parser.cursor.bump_non_trivia();
                let name = NameAst {
                    text: name_token.text.clone(),
                    span: name_token.span,
                };

                if coloncolon_follows(parser) {
                    parser.cursor.bump_non_trivia();
                    path.push(EntityPathSegmentAst {
                        name,
                        span: name_token.span,
                    });
                    continue;
                }

                let leaf = EntityPathLeafAst::Name(name);
                let end_span = name_token.span;
                let span = start.join(end_span);
                return finish_entity_ref(parser, path, leaf, span);
            }
            _ => {
                if let Some(spelling) = operator_spelling_in_expr_context(&token.kind) {
                    let op_token = parser.cursor.bump_non_trivia();
                    let operator = OperatorNameAst {
                        spelling: spelling.as_source_text().to_string(),
                        span: op_token.span,
                    };

                    if coloncolon_follows(parser) {
                        let cc_span = parser.cursor.bump_non_trivia().span;
                        parser.error(
                            DiagnosticCode::InvalidEntityRef,
                            "operator cannot appear as intermediate entity path segment",
                            cc_span,
                        );
                        parser.recover_to_form_boundary();
                        let leaf = EntityPathLeafAst::Error(
                            parser.error_ast("operator path leaf not final", cc_span),
                        );
                        let span = start.join(cc_span);
                        return EntityRefAst { path, leaf, span };
                    }

                    let leaf = EntityPathLeafAst::Operator(operator);
                    let end_span = op_token.span;
                    let span = start.join(end_span);
                    return finish_entity_ref(parser, path, leaf, span);
                }

                let span = token.span;
                let code = if path.is_empty() {
                    DiagnosticCode::ExpectedAliasTarget
                } else {
                    DiagnosticCode::InvalidEntityRef
                };
                parser.error(code, "expected entity reference after `===`", span);
                parser.cursor.bump_non_trivia();
                parser.recover_to_form_boundary();
                let leaf =
                    EntityPathLeafAst::Error(parser.error_ast("expected entity reference", span));
                return EntityRefAst {
                    path,
                    leaf,
                    span: start.join(span),
                };
            }
        }
    }
}

fn finish_entity_ref(
    parser: &mut Parser<'_>,
    path: Vec<EntityPathSegmentAst>,
    leaf: EntityPathLeafAst,
    span: Span,
) -> EntityRefAst {
    if parser.is_alias_rhs_boundary() {
        return EntityRefAst { path, leaf, span };
    }

    let next = parser.cursor.peek_non_trivia();
    parser.error(
        DiagnosticCode::UnexpectedAliasRhsExpression,
        format!("unexpected token `{}` after entity reference", next.text),
        next.span,
    );
    parser.recover_to_form_boundary();
    EntityRefAst { path, leaf, span }
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

fn coloncolon_follows(parser: &Parser<'_>) -> bool {
    let idx = parser.cursor.current_index();
    let (_, token) = parser.cursor.peek_at_skip_trivia(idx);
    matches!(token.kind, TokenKind::Symbol(Symbol::ColonColon))
}

fn parse_let_binder(parser: &mut Parser<'_>) -> LetBinderAst {
    if parser.cursor.at_symbol(Symbol::Less) {
        return parse_extract_binder(parser);
    }
    parse_simple_binder(parser)
}

fn parse_extract_binder(parser: &mut Parser<'_>) -> LetBinderAst {
    let start = parser.cursor.current_span();
    let deduce = parse_deduce_list(parser);

    if deduce.binders.is_empty() {
        parser.error(
            DiagnosticCode::InvalidDeduceList,
            "empty deduce list",
            deduce.span,
        );
    }

    let skeleton = parse_canonical_skeleton(parser, &deduce);
    let end_span = skeleton_span(&skeleton);
    let span = start.join(end_span);

    LetBinderAst::Extract {
        deduce,
        skeleton,
        span,
    }
}

fn skeleton_span(skeleton: &crate::CanonicalSkeletonAst) -> Span {
    match skeleton {
        crate::CanonicalSkeletonAst::Segment { span, .. } => *span,
        crate::CanonicalSkeletonAst::ArgPack { span, .. } => *span,
        crate::CanonicalSkeletonAst::Wildcard { span } => *span,
        crate::CanonicalSkeletonAst::Name { span, .. } => *span,
        crate::CanonicalSkeletonAst::Path { span, .. } => *span,
        crate::CanonicalSkeletonAst::Literal { span, .. } => *span,
        crate::CanonicalSkeletonAst::Error(error) => error.span,
    }
}

fn parse_simple_binder(parser: &mut Parser<'_>) -> LetBinderAst {
    let name_token = parser.cursor.peek_non_trivia();
    let name = if matches!(name_token.kind, TokenKind::Name) {
        let name_token = parser.cursor.bump_non_trivia();
        BinderNameAst::Text(NameAst {
            text: name_token.text.clone(),
            span: name_token.span,
        })
    } else if let Some(spelling) = operator_spelling_in_expr_context(&name_token.kind) {
        let name_token = parser.cursor.bump_non_trivia();
        BinderNameAst::Operator(OperatorNameAst {
            spelling: spelling.as_source_text().to_string(),
            span: name_token.span,
        })
    } else {
        let span = name_token.span;
        parser.error(
            DiagnosticCode::ExpectedName,
            "expected name after `let`",
            span,
        );
        return LetBinderAst::Error(parser.error_ast("expected name after `let`", span));
    };
    let name_span = binder_name_span(&name);

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
            span: name_span.join(span),
        };
    }

    let annotation = parse_decl_annotation(parser);
    let end_span = decl_annotation_span(&annotation);

    LetBinderAst::Simple {
        name,
        annotation,
        span: name_span.join(end_span),
    }
}

fn binder_name_span(name: &BinderNameAst) -> Span {
    match name {
        BinderNameAst::Text(name) => name.span,
        BinderNameAst::Operator(name) => name.span,
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
        recover_to_equal(parser);
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
            kind: WithClauseKind::Lexical,
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
            WithClauseKind::Lexical
        } else {
            WithClauseKind::Semantic { items }
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

fn recover_to_equal(parser: &mut Parser<'_>) {
    while !parser.is_form_boundary() && !parser.cursor.at_symbol(Symbol::Equal) {
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
