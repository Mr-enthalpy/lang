//! v0.1 frontend library.
//!
//! The current implementation includes the lexer loop and the first
//! parser skeleton: source text -> tokens -> AST + diagnostics.

pub mod ast;
pub mod diagnostic;
pub mod dump;
pub mod lexer;
pub mod parser;
pub mod source;
pub mod span;
pub mod token;

pub use ast::{
    AliasBinderAst, AnnotationTermAst, AtomAst, AtomKind, BinderDeclAst, BinderNameAst,
    BindingAnnotationAst, BindingPatternAst, BindingSlotAst, BodyBlockAst, CanonicalNameRole,
    CanonicalSkeletonAst, CaptureClauseAst, CaptureItemAst, ClosureAst, DeduceListAst,
    EntityRefAst, ErrorAst, ExplicitClosureAst, ExprAst, ExprKind, FnHeadPrefixAst, FormAst,
    HeadClauseAst, InPlaceClosureAst, LetAliasAst, LetAst, NameAst, NavComponentAst,
    NumericNameAst, OperatorExprAst, OperatorExprKind, OperatorFixity, OperatorNameAst,
    ParamClauseAst, PipeExprAst, ProductExprAst, ProductExtractAst, ProgramAst, ReturnClauseAst,
    SegmentAst, SegmentElementAst, SelectorAst, WithClauseAst, WithClauseKind,
};
pub use diagnostic::{Diagnostic, DiagnosticCode};
pub use dump::{dump_ast, dump_diagnostics, dump_tokens};
pub use lexer::{lex, LexOutput};
pub use parser::{parse, ParseOutput};
pub use source::normalize_source_text;
pub use span::Span;
pub use token::{OperatorSpelling, Symbol, Token, TokenKind, TriviaKind};

pub const VERSION: &str = "0.1.0";
