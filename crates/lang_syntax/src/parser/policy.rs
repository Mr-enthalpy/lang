use crate::{
    DiagnosticCode, ErrorAst, NameAst, OperatorSpelling, PolicyAtomAst, PolicyChoiceAst,
    PolicyConjunctionAst, PolicySpecAst, Span, Symbol, TokenKind, ValuePolicyPatternAst,
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

/// Parse the strong-context policy grammar.
///
/// `||` is represented only by `PolicyChoiceAst`, `+` only by
/// `PolicyConjunctionAst`, and `:` only by `PolicySpecAst`. Ordinary expression
/// parsing is deliberately not used here, so a Pattern `|` can never be
/// reinterpreted as policy choice.
pub fn parse_policy_spec_until(
    parser: &mut Parser<'_>,
    mut boundary: impl FnMut(&mut Parser<'_>) -> bool,
) -> PolicySpecAst {
    let value = {
        let mut value_boundary =
            |p: &mut Parser<'_>| p.cursor.at_symbol(Symbol::Colon) || boundary(p);
        parse_policy_conjunction(parser, &mut value_boundary)
    };
    let value_span = value.span;

    let pattern_policy = if parser.cursor.consume_symbol(Symbol::Colon).is_some() {
        Some(parse_policy_conjunction(parser, &mut boundary))
    } else {
        None
    };

    if !boundary(parser) {
        let unexpected = parser.cursor.peek_non_trivia().clone();
        let message = match unexpected.kind {
            TokenKind::Operator(OperatorSpelling::Pipe) => {
                "single `|` is a Pattern alternative; policy choice must use `||`"
            }
            _ => "unexpected token in policy specification",
        };
        parser.error(DiagnosticCode::UnexpectedToken, message, unexpected.span);
        while !boundary(parser) && !parser.cursor.at_eof() {
            parser.cursor.bump_non_trivia();
        }
    }

    let span = pattern_policy
        .as_ref()
        .map_or(value_span, |pattern| value_span.join(pattern.span));

    PolicySpecAst {
        value_policy: ValuePolicyPatternAst::Conjunction(value),
        pattern_policy,
        span,
    }
}

fn parse_policy_conjunction(
    parser: &mut Parser<'_>,
    boundary: &mut dyn FnMut(&mut Parser<'_>) -> bool,
) -> PolicyConjunctionAst {
    let first = parse_policy_choice(parser, boundary);
    let mut span = first.span;
    let mut choices = vec![first];

    while consume_operator(parser, OperatorSpelling::Plus).is_some() {
        let choice = parse_policy_choice(parser, boundary);
        span = span.join(choice.span);
        choices.push(choice);
    }

    PolicyConjunctionAst { choices, span }
}

fn parse_policy_choice(
    parser: &mut Parser<'_>,
    boundary: &mut dyn FnMut(&mut Parser<'_>) -> bool,
) -> PolicyChoiceAst {
    let first = parse_policy_atom(parser, boundary);
    let mut span = policy_atom_span(&first);
    let mut atoms = vec![first];

    while consume_operator(parser, OperatorSpelling::PipePipe).is_some() {
        let atom = parse_policy_atom(parser, boundary);
        span = span.join(policy_atom_span(&atom));
        atoms.push(atom);
    }

    PolicyChoiceAst { atoms, span }
}

fn parse_policy_atom(
    parser: &mut Parser<'_>,
    boundary: &mut dyn FnMut(&mut Parser<'_>) -> bool,
) -> PolicyAtomAst {
    let token = parser.cursor.peek_non_trivia().clone();
    match token.kind {
        TokenKind::Name => {
            parser.cursor.bump_non_trivia();
            if token.text == "S" {
                PolicyAtomAst::AbsentValuePattern { span: token.span }
            } else {
                PolicyAtomAst::Name(NameAst {
                    text: token.text,
                    span: token.span,
                })
            }
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
            if !boundary(parser) {
                parser.error(
                    DiagnosticCode::UnexpectedToken,
                    "expected policy atom",
                    span,
                );
            }
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
        PolicyAtomAst::Group { span, .. }
        | PolicyAtomAst::AbsentValuePattern { span }
        | PolicyAtomAst::Error(ErrorAst { span, .. }) => *span,
    }
}
