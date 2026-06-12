use crate::{
    normalize_source_text, Diagnostic, DiagnosticCode, Span, Symbol, Token, TokenKind, TriviaKind,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LexOutput {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn lex(source: &str) -> LexOutput {
    let normalized = normalize_source_text(source);
    Lexer::new(&normalized).lex()
}

struct Lexer<'src> {
    source: &'src str,
    byte: usize,
    line: usize,
    column: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
}

impl<'src> Lexer<'src> {
    fn new(source: &'src str) -> Self {
        Self {
            source,
            byte: 0,
            line: 1,
            column: 1,
            tokens: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn lex(mut self) -> LexOutput {
        while !self.is_eof() {
            self.lex_token();
        }

        let eof_span = Span::at(self.byte, self.line, self.column);
        self.tokens.push(Token::new(TokenKind::Eof, eof_span, ""));

        LexOutput {
            tokens: self.tokens,
            diagnostics: self.diagnostics,
        }
    }

    fn lex_token(&mut self) {
        if self.peek_is_ascii_ident_start() {
            self.lex_name();
        } else if self.peek_is_ascii_digit() {
            self.lex_int_literal();
        } else if self.starts_with("\"") {
            self.lex_string_literal();
        } else if self.peek_is_whitespace() {
            self.lex_whitespace();
        } else if self.starts_with("//") {
            self.lex_line_comment();
        } else if self.starts_with("/*") {
            self.lex_block_comment();
        } else if self.lex_symbol() {
            // Symbol consumed.
        } else {
            self.lex_invalid_token();
        }
    }

    fn lex_name(&mut self) {
        let start = self.mark();
        self.advance_char();

        while self.peek_is_ascii_ident_continue() {
            self.advance_char();
        }

        self.push_token(TokenKind::Name, start);
    }

    fn lex_int_literal(&mut self) {
        let start = self.mark();

        while self.peek_is_ascii_digit() {
            self.advance_char();
        }

        self.push_token(TokenKind::IntLiteral, start);
    }

    fn lex_string_literal(&mut self) {
        let start = self.mark();
        self.advance_char();
        let mut closed = false;

        while let Some(ch) = self.peek_char() {
            match ch {
                '"' => {
                    self.advance_char();
                    closed = true;
                    break;
                }
                '\n' | '\r' => break,
                '\\' => {
                    self.advance_char();
                    match self.peek_char() {
                        Some('\n' | '\r') | None => break,
                        Some(_) => {
                            self.advance_char();
                        }
                    }
                }
                _ => {
                    self.advance_char();
                }
            }
        }

        let span = self.span_from(start);
        if !closed {
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::UnclosedString,
                "unclosed string literal",
                span,
            ));
        }

        self.push_token_with_span(TokenKind::StringLiteral, span);
    }

    fn lex_whitespace(&mut self) {
        let start = self.mark();

        while self.peek_is_whitespace() {
            self.advance_char();
        }

        self.push_token(TokenKind::Trivia(TriviaKind::Whitespace), start);
    }

    fn lex_line_comment(&mut self) {
        let start = self.mark();
        self.advance_char();
        self.advance_char();

        while let Some(ch) = self.peek_char() {
            if ch == '\n' || ch == '\r' {
                break;
            }
            self.advance_char();
        }

        self.push_token(TokenKind::Trivia(TriviaKind::LineComment), start);
    }

    fn lex_block_comment(&mut self) {
        let start = self.mark();
        self.advance_char();
        self.advance_char();
        let mut closed = false;

        while !self.is_eof() {
            if self.starts_with("*/") {
                self.advance_char();
                self.advance_char();
                closed = true;
                break;
            }
            self.advance_char();
        }

        let span = self.span_from(start);
        if !closed {
            self.diagnostics.push(Diagnostic::new(
                DiagnosticCode::UnclosedComment,
                "unclosed block comment",
                span,
            ));
        }

        self.push_token_with_span(TokenKind::Trivia(TriviaKind::BlockComment), span);
    }

    fn lex_symbol(&mut self) -> bool {
        let symbols = [
            ("..", Symbol::DotDot),
            ("::", Symbol::ColonColon),
            ("|>", Symbol::PipeGreater),
            ("=>", Symbol::FatArrow),
            ("->", Symbol::ThinArrow),
            ("(", Symbol::LParen),
            (")", Symbol::RParen),
            ("[", Symbol::LBracket),
            ("]", Symbol::RBracket),
            ("{", Symbol::LBrace),
            ("}", Symbol::RBrace),
            (",", Symbol::Comma),
            (":", Symbol::Colon),
            ("=", Symbol::Equal),
            (".", Symbol::Dot),
            ("<", Symbol::Less),
            (">", Symbol::Greater),
            (";", Symbol::Semicolon),
        ];

        for (text, symbol) in symbols {
            if self.starts_with(text) {
                let start = self.mark();
                for _ in text.chars() {
                    self.advance_char();
                }
                self.push_token(TokenKind::Symbol(symbol), start);
                return true;
            }
        }

        false
    }

    fn lex_invalid_token(&mut self) {
        let start = self.mark();
        self.advance_char();
        let span = self.span_from(start);

        self.diagnostics.push(Diagnostic::new(
            DiagnosticCode::InvalidToken,
            "invalid token",
            span,
        ));
        self.push_token_with_span(TokenKind::Invalid, span);
    }

    fn push_token(&mut self, kind: TokenKind, start: Mark) {
        let span = self.span_from(start);
        self.push_token_with_span(kind, span);
    }

    fn push_token_with_span(&mut self, kind: TokenKind, span: Span) {
        let text = self.source[span.byte_start..span.byte_end].to_string();
        self.tokens.push(Token::new(kind, span, text));
    }

    fn span_from(&self, start: Mark) -> Span {
        Span::new(start.byte, self.byte, start.line, start.column)
    }

    fn mark(&self) -> Mark {
        Mark {
            byte: self.byte,
            line: self.line,
            column: self.column,
        }
    }

    fn is_eof(&self) -> bool {
        self.byte >= self.source.len()
    }

    fn starts_with(&self, text: &str) -> bool {
        self.source[self.byte..].starts_with(text)
    }

    fn peek_char(&self) -> Option<char> {
        self.source[self.byte..].chars().next()
    }

    fn advance_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.byte += ch.len_utf8();

        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += ch.len_utf8();
        }

        Some(ch)
    }

    fn peek_is_ascii_ident_start(&self) -> bool {
        matches!(self.peek_char(), Some('a'..='z' | 'A'..='Z' | '_'))
    }

    fn peek_is_ascii_ident_continue(&self) -> bool {
        matches!(
            self.peek_char(),
            Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_')
        )
    }

    fn peek_is_ascii_digit(&self) -> bool {
        matches!(self.peek_char(), Some('0'..='9'))
    }

    fn peek_is_whitespace(&self) -> bool {
        matches!(self.peek_char(), Some(' ' | '\t' | '\r' | '\n'))
    }
}

#[derive(Clone, Copy)]
struct Mark {
    byte: usize,
    line: usize,
    column: usize,
}
