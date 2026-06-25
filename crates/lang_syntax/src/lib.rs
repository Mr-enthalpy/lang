//! v0.2 frontend library.
//!
//! The completed Raw AST frontend: source text -> tokens -> AST + diagnostics.

pub mod ast;
pub mod diagnostic;
pub mod dump;
pub mod lexer;
pub mod norm;
pub mod parser;
pub mod source;
pub mod span;
pub mod token;

pub use ast::{
    AliasBinderAst, AnnotationTermAst, AtomAst, AtomKind, BinderDeclAst, BinderNameAst,
    BindingAnnotationAst, BindingPatternAst, BindingSlotAst, BodyBlockAst, CanonicalNameRole,
    CanonicalProductElementAst, CanonicalSkeletonAst, CaptureClauseAst, CaptureItemAst, ClosureAst,
    DeduceListAst, EntityRefAst, ErrorAst, ExplicitClosureAst, ExprAst, ExprKind, FnHeadPrefixAst,
    FormAst, HeadClauseAst, InPlaceClosureAst, LetAliasAst, LetAst, NameAst, NavComponentAst,
    OperatorExprAst, OperatorExprKind, OperatorFixity, OperatorNameAst, ParamClauseAst,
    PipeExprAst, ProductElementAst, ProductExprAst, ProductExtractAst, ProductExtractElementAst,
    ProgramAst, ReturnClauseAst, SegmentAst, SegmentElementAst, SelectorAst, WithClauseAst,
    WithClauseKind,
};
pub use diagnostic::{Diagnostic, DiagnosticCode};
pub use dump::{dump_ast, dump_diagnostics, dump_tokens};
pub use lexer::{lex, LexOutput};
pub use norm::{
    dump_norm_program, normalize_program, NormAliasBinder, NormAnnotation, NormBindingSlot,
    NormCanonicalNameRole, NormClosure, NormClosureHead, NormClosureKind, NormDecl, NormEntityRef,
    NormError, NormExpr, NormForm, NormHeadClause, NormHoleDecl, NormLiteralKind, NormNavComponent,
    NormOperatorFixity, NormOrigin, NormPattern, NormPatternElem, NormProduct, NormProductElem,
    NormProgram, NormRule, NormSkeleton, NormSkeletonElem, NormWithClause,
};
pub use parser::{parse, ParseOutput};
pub use source::normalize_source_text;
pub use span::Span;
pub use token::{OperatorSpelling, Symbol, Token, TokenKind, TriviaKind};

pub const VERSION: &str = "0.2.0";
