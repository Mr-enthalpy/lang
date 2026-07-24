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
    ClosureBodyAst, ClosurePlacementAst, DeduceListAst, DeleteBodyAst, EntityRefAst, ErrorAst,
    ExprAst, ExprKind, FnHeadPrefixAst, FormAst, HeadClauseAst, LetAliasAst, LetAst,
    MemberVisibilityAst, NameAst, NavComponentAst, OperatorExprAst, OperatorExprKind,
    OperatorFixity, OperatorNameAst, ParamClauseAst, PipeExprAst, PolicyAtomAst, PolicyChoiceAst,
    PolicyConjunctionAst, PolicySpecAst, ProductElementAst, ProductExprAst, ProductExtractAst,
    ProductExtractElementAst, ProgramAst, ReturnClauseAst, ReturnEventAst, ReturnTargetAst,
    SegmentAst, SegmentElementAst, SelectorAst, ValuePolicyPatternAst, WithClauseAst,
    WithClauseKind,
};
pub use diagnostic::{Diagnostic, DiagnosticCode};
pub use dump::{dump_ast, dump_diagnostics, dump_tokens};
pub use lexer::{lex, LexOutput};
pub use norm::{
    dump_norm_program, normalize_and_validate_patterns, normalize_program,
    validate_normalized_pattern_layers, validate_normalized_patterns,
    validate_pack_pattern_element_level, validate_pack_pattern_layers, AlphaOwnerId,
    GeneratedHoleKey, HoleBinderId, NormAliasBinder, NormAnnotation, NormBindingSlot,
    NormCallableOwner, NormCanonicalNameRole, NormCapture, NormClosure, NormClosureBody,
    NormClosureHead, NormClosurePlacement, NormDecl, NormDeleteBody, NormEntityRef, NormError,
    NormExpr, NormForm, NormHeadClause, NormHoleDecl, NormLiteralKind, NormNavComponent,
    NormOperatorFixity, NormOrigin, NormOverloadStrategy, NormPattern, NormPatternElem,
    NormPolicyAtom, NormPolicyChoice, NormPolicyConjunction, NormPolicySpec, NormProduct,
    NormProductElem, NormProgram, NormReturnEvent, NormReturnTargetSyntax, NormRule,
    NormSemanticOwnerId, NormSkeleton, NormSkeletonElem, NormValuePolicyPattern, NormWithClause,
    PackPatternLayerError, PatternInvalidNormProgram, PatternRootId, PatternValidatedNormProgram,
    PatternValidationError,
};
pub use parser::{parse, ParseOutput};
pub use source::normalize_source_text;
pub use span::Span;
pub use token::{OperatorSpelling, Symbol, Token, TokenKind, TriviaKind};

pub const VERSION: &str = "0.5.0";
