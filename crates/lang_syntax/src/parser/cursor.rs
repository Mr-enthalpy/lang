use crate::{Span, Symbol, Token, TokenKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParenClassification {
    NotParen,
    Group,
    Product,
    Unclosed,
}

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

    pub fn current_raw_span(&self) -> Span {
        self.peek().span
    }

    pub fn current_index(&self) -> usize {
        self.index
    }

    pub fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    pub fn peek_at(&self, index: usize) -> &'tokens Token {
        &self.tokens[index]
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn peek_at_skip_trivia(&self, mut index: usize) -> (usize, &'tokens Token) {
        while index < self.tokens.len() && matches!(self.tokens[index].kind, TokenKind::Trivia(_)) {
            index += 1;
        }
        if index < self.tokens.len() {
            (index, &self.tokens[index])
        } else {
            (index, &self.tokens[self.tokens.len() - 1])
        }
    }

    pub fn can_start_segment_element(token: &Token) -> bool {
        matches!(
            token.kind,
            TokenKind::Name
                | TokenKind::IntLiteral
                | TokenKind::StringLiteral
                | TokenKind::Symbol(Symbol::LParen)
        )
    }

    pub fn classify_paren_at_segment_position(&self) -> (ParenClassification, Option<usize>) {
        let (mut i, first) = self.peek_at_skip_trivia(self.index);
        if !matches!(first.kind, TokenKind::Symbol(Symbol::LParen)) {
            return (ParenClassification::NotParen, None);
        }

        let _start = i;
        let mut depth: usize = 0;
        let mut has_comma = false;

        loop {
            if i >= self.tokens.len() {
                return (ParenClassification::Unclosed, None);
            }
            let token = &self.tokens[i];

            if matches!(token.kind, TokenKind::Trivia(_)) {
                i += 1;
                continue;
            }

            match &token.kind {
                TokenKind::Symbol(Symbol::LParen)
                | TokenKind::Symbol(Symbol::LBracket)
                | TokenKind::Symbol(Symbol::LBrace) => {
                    depth += 1;
                }
                TokenKind::Symbol(Symbol::RParen)
                | TokenKind::Symbol(Symbol::RBracket)
                | TokenKind::Symbol(Symbol::RBrace) => {
                    if depth == 1 {
                        if matches!(token.kind, TokenKind::Symbol(Symbol::RParen)) {
                            i += 1;
                            break;
                        }
                        break;
                    }
                    depth -= 1;
                }
                TokenKind::Symbol(Symbol::Comma) => {
                    if depth == 1 {
                        has_comma = true;
                    }
                }
                TokenKind::Eof => {
                    return (ParenClassification::Unclosed, None);
                }
                _ => {}
            }

            if depth == 0 {
                return (ParenClassification::Unclosed, None);
            }

            i += 1;
        }

        if has_comma {
            (ParenClassification::Product, Some(i))
        } else {
            (ParenClassification::Group, Some(i))
        }
    }

    // This is used only by parenthesis classification. Operator-expression
    // starts are handled by the operator parser.
    pub fn at_angle_left_for_deduce(&mut self) -> bool {
        matches!(self.peek_non_trivia().kind, TokenKind::Symbol(Symbol::Less))
    }

    pub fn at_angle_right_for_deduce(&mut self) -> bool {
        matches!(
            self.peek_non_trivia().kind,
            TokenKind::Symbol(Symbol::Greater)
        )
    }

    pub fn is_at_segment_element_start(&self) -> bool {
        let (_, token) = self.peek_at_skip_trivia(self.index);
        Self::can_start_segment_element(token)
    }

    pub fn is_at_pipe_element(&mut self) -> bool {
        matches!(
            self.peek_non_trivia().kind,
            TokenKind::Symbol(Symbol::PipeGreater)
        )
    }

    // Hard form boundaries: `;`, `}`, EOF.
    pub fn is_form_boundary(&mut self) -> bool {
        matches!(
            self.peek_non_trivia().kind,
            TokenKind::Eof
                | TokenKind::Symbol(Symbol::Semicolon)
                | TokenKind::Symbol(Symbol::RBrace)
        )
    }

    pub fn consume_form_boundary(&mut self) {
        self.skip_trivia();
        if matches!(
            self.peek().kind,
            TokenKind::Symbol(Symbol::Semicolon) | TokenKind::Symbol(Symbol::RBrace)
        ) {
            self.bump();
        }
    }
}
