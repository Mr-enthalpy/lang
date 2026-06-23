use crate::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticCode {
    InvalidToken,
    UnclosedString,
    UnclosedComment,
    UnexpectedToken,
    ExpectedName,
    ExpectedColon,
    ExpectedBindingAnnotation,
    ExpectedEqual,
    EmptyPipeSegment,
    ExpectedNameAfterDot,
    ExpectedNameAfterDoubleDot,
    ExpectedProductAfterDoubleDotName,
    UnclosedParen,
    UnclosedBracket,
    UnclosedBrace,
    InvalidDeduceList,
    InvalidCanonicalSkeleton,
    InvalidClosureHead,
    InvalidOperatorExpression,
    ChainedNonAssociativeOperator,
    InvalidNavComponent,
    TopLevelComma,
    UnusedClosureAst,
    ExpectedAliasTarget,
    InvalidAliasBinder,
    InvalidEntityRef,
    UnexpectedAliasRhsExpression,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub message: String,
    pub span: Span,
}

impl Diagnostic {
    pub fn new(code: DiagnosticCode, message: impl Into<String>, span: Span) -> Self {
        Self {
            code,
            message: message.into(),
            span,
        }
    }
}
