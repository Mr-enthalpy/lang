use crate::Span;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Name,
    IntLiteral,
    StringLiteral,
    Symbol(Symbol),
    Operator(OperatorSpelling),
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
    TripleEqual,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperatorSpelling {
    Plus,
    Minus,
    Star,
    Slash,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    EqualEqual,
    BangEqual,
    LessLess,
    GreaterGreater,
    Bang,
    Amp,
    At,
    Tilde,
    Caret,
    Dollar,
    PlusPlus,
    MinusMinus,
    Question,
    PlusEqual,
    MinusEqual,
    StarEqual,
    SlashEqual,
    LessLessEqual,
    GreaterGreaterEqual,
}

impl OperatorSpelling {
    pub fn label(&self) -> &'static str {
        match self {
            OperatorSpelling::Plus => "Plus",
            OperatorSpelling::Minus => "Minus",
            OperatorSpelling::Star => "Star",
            OperatorSpelling::Slash => "Slash",
            OperatorSpelling::Less => "Less",
            OperatorSpelling::Greater => "Greater",
            OperatorSpelling::LessEqual => "LessEqual",
            OperatorSpelling::GreaterEqual => "GreaterEqual",
            OperatorSpelling::EqualEqual => "EqualEqual",
            OperatorSpelling::BangEqual => "BangEqual",
            OperatorSpelling::LessLess => "LessLess",
            OperatorSpelling::GreaterGreater => "GreaterGreater",
            OperatorSpelling::Bang => "Bang",
            OperatorSpelling::Amp => "Amp",
            OperatorSpelling::At => "At",
            OperatorSpelling::Tilde => "Tilde",
            OperatorSpelling::Caret => "Caret",
            OperatorSpelling::Dollar => "Dollar",
            OperatorSpelling::PlusPlus => "PlusPlus",
            OperatorSpelling::MinusMinus => "MinusMinus",
            OperatorSpelling::Question => "Question",
            OperatorSpelling::PlusEqual => "PlusEqual",
            OperatorSpelling::MinusEqual => "MinusEqual",
            OperatorSpelling::StarEqual => "StarEqual",
            OperatorSpelling::SlashEqual => "SlashEqual",
            OperatorSpelling::LessLessEqual => "LessLessEqual",
            OperatorSpelling::GreaterGreaterEqual => "GreaterGreaterEqual",
        }
    }

    pub fn as_source_text(&self) -> &'static str {
        match self {
            OperatorSpelling::Plus => "+",
            OperatorSpelling::Minus => "-",
            OperatorSpelling::Star => "*",
            OperatorSpelling::Slash => "/",
            OperatorSpelling::Less => "<",
            OperatorSpelling::Greater => ">",
            OperatorSpelling::LessEqual => "<=",
            OperatorSpelling::GreaterEqual => ">=",
            OperatorSpelling::EqualEqual => "==",
            OperatorSpelling::BangEqual => "!=",
            OperatorSpelling::LessLess => "<<",
            OperatorSpelling::GreaterGreater => ">>",
            OperatorSpelling::Bang => "!",
            OperatorSpelling::Amp => "&",
            OperatorSpelling::At => "@",
            OperatorSpelling::Tilde => "~",
            OperatorSpelling::Caret => "^",
            OperatorSpelling::Dollar => "$",
            OperatorSpelling::PlusPlus => "++",
            OperatorSpelling::MinusMinus => "--",
            OperatorSpelling::Question => "?",
            OperatorSpelling::PlusEqual => "+=",
            OperatorSpelling::MinusEqual => "-=",
            OperatorSpelling::StarEqual => "*=",
            OperatorSpelling::SlashEqual => "/=",
            OperatorSpelling::LessLessEqual => "<<=",
            OperatorSpelling::GreaterGreaterEqual => ">>=",
        }
    }
}

impl TokenKind {
    pub fn is_operator_spelling(&self) -> bool {
        matches!(
            self,
            TokenKind::Operator(_)
                | TokenKind::Symbol(Symbol::Less)
                | TokenKind::Symbol(Symbol::Greater)
        )
    }
}

// In expression/operator context, bare `<` and `>` are operator spellings
// even though the lexer emits them as Symbol::Less / Symbol::Greater for
// deduce-list compatibility. This helper unifies the two representations.
pub fn operator_spelling_in_expr_context(kind: &TokenKind) -> Option<OperatorSpelling> {
    match kind {
        TokenKind::Operator(op) => Some(*op),
        TokenKind::Symbol(Symbol::Less) => Some(OperatorSpelling::Less),
        TokenKind::Symbol(Symbol::Greater) => Some(OperatorSpelling::Greater),
        _ => None,
    }
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
