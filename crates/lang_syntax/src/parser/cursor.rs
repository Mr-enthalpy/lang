use crate::{Span, Symbol, Token, TokenKind, TriviaKind};

pub struct Cursor<'tokens> {
    tokens: &'tokens [Token],
    index: usize,
}

impl<'tokens> Cursor<'tokens> {
    pub fn new(tokens: &'tokens [Token]) -> Self {
        Self { tokens, index: 0 }
    }

    pub fn peek(&self) -> &'tokens Token {
        &self.tokens[self.index]
    }

    pub fn peek_non_trivia(&mut self) -> &'tokens Token {
        self.skip_trivia();
        self.peek()
    }

    pub fn peek_next_non_trivia(&self) -> &'tokens Token {
        let mut index = self.index + 1;
        while matches!(self.tokens[index].kind, TokenKind::Trivia(_)) {
            index += 1;
        }
        &self.tokens[index]
    }

    pub fn bump(&mut self) -> &'tokens Token {
        let token = self.peek();
        if !matches!(token.kind, TokenKind::Eof) {
            self.index += 1;
        }
        token
    }

    pub fn bump_non_trivia(&mut self) -> &'tokens Token {
        self.skip_trivia();
        self.bump()
    }

    pub fn skip_trivia(&mut self) {
        while matches!(self.peek().kind, TokenKind::Trivia(_)) {
            self.index += 1;
        }
    }

    pub fn at_eof(&mut self) -> bool {
        matches!(self.peek_non_trivia().kind, TokenKind::Eof)
    }

    pub fn at_symbol(&mut self, symbol: Symbol) -> bool {
        matches!(self.peek_non_trivia().kind, TokenKind::Symbol(current) if current == symbol)
    }

    pub fn at_name(&mut self, text: &str) -> bool {
        let token = self.peek_non_trivia();
        matches!(token.kind, TokenKind::Name) && token.text == text
    }

    pub fn consume_symbol(&mut self, symbol: Symbol) -> Option<&'tokens Token> {
        if self.at_symbol(symbol) {
            Some(self.bump_non_trivia())
        } else {
            None
        }
    }

    pub fn consume_name(&mut self, text: &str) -> Option<&'tokens Token> {
        if self.at_name(text) {
            Some(self.bump_non_trivia())
        } else {
            None
        }
    }

    pub fn current_span(&mut self) -> Span {
        self.peek_non_trivia().span
    }

    // TODO: v0.1 full spec lists `;`, top-level newline, `}`, and EOF as
    // form boundaries. Currently only semicolon and EOF are implemented;
    // newline-sensitive parsing and closing-brace boundary detection are
    // pending parser refinement.
    pub fn is_form_boundary(&mut self) -> bool {
        matches!(
            self.peek_non_trivia().kind,
            TokenKind::Eof | TokenKind::Symbol(Symbol::Semicolon)
        )
    }

    pub fn consume_form_boundary(&mut self) {
        self.skip_trivia();
        if matches!(self.peek().kind, TokenKind::Symbol(Symbol::Semicolon)) {
            self.bump();
        }
    }

    #[allow(dead_code)]
    pub fn at_trivia_newline(&self) -> bool {
        matches!(
            self.peek().kind,
            TokenKind::Trivia(TriviaKind::Whitespace) if self.peek().text.contains('\n')
        )
    }
}
