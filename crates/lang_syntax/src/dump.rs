use crate::{Diagnostic, DiagnosticCode, Symbol, Token, TokenKind, TriviaKind};

pub fn dump_tokens(tokens: &[Token]) -> String {
    let mut output = String::new();

    for token in tokens {
        output.push_str(&format!(
            "{} \"{}\" @ {}:{} [{}..{}]\n",
            token_kind_label(&token.kind),
            escape_text(&token.text),
            token.span.line,
            token.span.column,
            token.span.byte_start,
            token.span.byte_end
        ));
    }

    output
}

pub fn dump_diagnostics(diagnostics: &[Diagnostic]) -> String {
    let mut output = String::new();

    for diagnostic in diagnostics {
        output.push_str(&format!(
            "error {} \"{}\" @ {}:{} [{}..{}]\n",
            diagnostic_code_label(diagnostic.code),
            escape_text(&diagnostic.message),
            diagnostic.span.line,
            diagnostic.span.column,
            diagnostic.span.byte_start,
            diagnostic.span.byte_end
        ));
    }

    output
}

fn token_kind_label(kind: &TokenKind) -> String {
    match kind {
        TokenKind::Name => "Name".to_string(),
        TokenKind::IntLiteral => "IntLiteral".to_string(),
        TokenKind::StringLiteral => "StringLiteral".to_string(),
        TokenKind::Symbol(symbol) => format!("Symbol.{}", symbol_label(*symbol)),
        TokenKind::Trivia(trivia) => format!("Trivia.{}", trivia_label(*trivia)),
        TokenKind::Invalid => "Invalid".to_string(),
        TokenKind::Eof => "Eof".to_string(),
    }
}

fn symbol_label(symbol: Symbol) -> &'static str {
    match symbol {
        Symbol::LParen => "LParen",
        Symbol::RParen => "RParen",
        Symbol::LBracket => "LBracket",
        Symbol::RBracket => "RBracket",
        Symbol::LBrace => "LBrace",
        Symbol::RBrace => "RBrace",
        Symbol::Comma => "Comma",
        Symbol::Colon => "Colon",
        Symbol::Equal => "Equal",
        Symbol::Dot => "Dot",
        Symbol::DotDot => "DotDot",
        Symbol::ColonColon => "ColonColon",
        Symbol::PipeGreater => "PipeGreater",
        Symbol::FatArrow => "FatArrow",
        Symbol::ThinArrow => "ThinArrow",
        Symbol::Less => "Less",
        Symbol::Greater => "Greater",
        Symbol::Semicolon => "Semicolon",
    }
}

fn trivia_label(trivia: TriviaKind) -> &'static str {
    match trivia {
        TriviaKind::Whitespace => "Whitespace",
        TriviaKind::LineComment => "LineComment",
        TriviaKind::BlockComment => "BlockComment",
    }
}

fn diagnostic_code_label(code: DiagnosticCode) -> &'static str {
    match code {
        DiagnosticCode::InvalidToken => "InvalidToken",
        DiagnosticCode::UnclosedString => "UnclosedString",
        DiagnosticCode::UnclosedComment => "UnclosedComment",
    }
}

fn escape_text(text: &str) -> String {
    let mut escaped = String::new();

    for ch in text.chars() {
        match ch {
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            _ => escaped.push(ch),
        }
    }

    escaped
}
