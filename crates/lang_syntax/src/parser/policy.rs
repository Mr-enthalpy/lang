use crate::{
    DiagnosticCode, ErrorAst, NameAst, OperatorSpelling, PolicyAtomAst, PolicyConjunctionAst,
    PolicySpecAst, Span, Symbol, TokenKind,
};

use super::form::Parser;

pub fn try_parse_policy_spec_before_let(
    parser: &mut Parser<'_>,
    mut boundary: impl FnMut(&mut Parser<'_>) -> bool,
) -> Option<PolicySpecAst> {
    if parser.cursor.at_name("let") {
        return None;
    }
    if !matches!(
        parser.cursor.peek_non_trivia().kind,
        TokenKind::Name | TokenKind::Symbol(Symbol::LParen)
    ) {
        return None;
    }

    let saved = parser.cursor.current_index();
    parser.gate_diagnostics();
    let policy = parse_policy_spec_until(parser, |p| p.cursor.at_name("let") || boundary(p));

    if parser.cursor.at_name("let") && parser.current_diagnostic_gate_is_empty() {
        parser.ungate_keep_diagnostics();
        Some(policy)
    } else {
        parser.cursor.set_index(saved);
        parser.ungate_drop_diagnostics();
        None
    }
}

/// Preserve ordinary atoms and orthogonal conjunction in a strong Policy context.
/// Internal value/type observations have no public pair or choice grammar.
pub fn parse_policy_spec_until(
    parser: &mut Parser<'_>,
    mut boundary: impl FnMut(&mut Parser<'_>) -> bool,
) -> PolicySpecAst {
    let mut constraint = parse_policy_conjunction(parser, &mut boundary);
    if !boundary(parser) {
        let unexpected = parser.cursor.peek_non_trivia().clone();
        parser.error(
            DiagnosticCode::UnexpectedToken,
            "unexpected token in policy constraint",
            unexpected.span,
        );
        constraint.atoms = vec![PolicyAtomAst::Error(ErrorAst {
            message: "invalid policy constraint".to_string(),
            span: unexpected.span,
        })];
        while !boundary(parser) && !parser.cursor.at_eof() {
            parser.cursor.bump_non_trivia();
        }
    }
    PolicySpecAst {
        span: constraint.span,
        constraint,
    }
}

fn parse_policy_conjunction(
    parser: &mut Parser<'_>,
    boundary: &mut dyn FnMut(&mut Parser<'_>) -> bool,
) -> PolicyConjunctionAst {
    let first = parse_policy_atom(parser, boundary);
    let mut span = policy_atom_span(&first);
    let mut atoms = vec![first];
    while consume_operator(parser, OperatorSpelling::Plus).is_some() {
        let atom = parse_policy_atom(parser, boundary);
        span = span.join(policy_atom_span(&atom));
        atoms.push(atom);
    }
    PolicyConjunctionAst { atoms, span }
}

fn parse_policy_atom(
    parser: &mut Parser<'_>,
    boundary: &mut dyn FnMut(&mut Parser<'_>) -> bool,
) -> PolicyAtomAst {
    let token = parser.cursor.peek_non_trivia().clone();
    match token.kind {
        TokenKind::Name => {
            parser.cursor.bump_non_trivia();
            PolicyAtomAst::Name(NameAst {
                text: token.text,
                span: token.span,
            })
        }
        TokenKind::Symbol(Symbol::LParen) => {
            let lparen = parser.cursor.bump_non_trivia().span;
            let mut group_boundary = |p: &mut Parser<'_>| p.cursor.at_symbol(Symbol::RParen);
            let conjunction = parse_policy_conjunction(parser, &mut group_boundary);
            let end = if let Some(rparen) = parser.cursor.consume_symbol(Symbol::RParen) {
                rparen.span
            } else {
                parser.error(
                    DiagnosticCode::UnclosedParen,
                    "unclosed policy group, expected `)`",
                    lparen,
                );
                conjunction.span
            };
            PolicyAtomAst::Group {
                conjunction: Box::new(conjunction),
                span: lparen.join(end),
            }
        }
        _ => {
            let span = token.span;
            parser.error(
                DiagnosticCode::UnexpectedToken,
                "expected policy atom",
                span,
            );
            if !boundary(parser) && !parser.cursor.at_eof() {
                parser.cursor.bump_non_trivia();
            }
            PolicyAtomAst::Error(ErrorAst {
                message: "expected policy atom".to_string(),
                span,
            })
        }
    }
}

fn consume_operator(parser: &mut Parser<'_>, expected: OperatorSpelling) -> Option<Span> {
    let token = parser.cursor.peek_non_trivia();
    if matches!(token.kind, TokenKind::Operator(actual) if actual == expected) {
        Some(parser.cursor.bump_non_trivia().span)
    } else {
        None
    }
}

fn policy_atom_span(atom: &PolicyAtomAst) -> Span {
    match atom {
        PolicyAtomAst::Name(name) => name.span,
        PolicyAtomAst::Group { span, .. } | PolicyAtomAst::Error(ErrorAst { span, .. }) => *span,
    }
}
