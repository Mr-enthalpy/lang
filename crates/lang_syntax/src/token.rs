use crate::Span;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Name,
    IntLiteral,
    StringLiteral,
    Symbol(Symbol),
    Trivia(TriviaKind),
    Invalid,
    Eof,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Symbol {
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Equal,
    Dot,
    DotDot,
    ColonColon,
    PipeGreater,
    FatArrow,
    ThinArrow,
    Less,
    Greater,
    Semicolon,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriviaKind {
    Whitespace,
    LineComment,
    BlockComment,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub text: String,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, text: impl Into<String>) -> Self {
        Self {
            kind,
            span,
            text: text.into(),
        }
    }
}
