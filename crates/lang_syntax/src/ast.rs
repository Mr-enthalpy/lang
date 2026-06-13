use crate::Span;

// First parser skeleton AST. This intentionally covers only a narrow subset:
// flat name/literal/path expressions and simple let forms. The full v0.1 raw
// AST will expand in later parser phases.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramAst {
    pub forms: Vec<FormAst>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormAst {
    Let(LetAst),
    Expr(ExprAst),
    Error(ErrorAst),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LetAst {
    pub attrs: Vec<LetAttrAst>,
    pub binder: LetBinderAst,
    pub with_deps: Vec<NameAst>,
    pub value: ExprAst,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LetAttrAst {
    Guard,
}

// Future operator parser phase: binder leaves will allow operator names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LetBinderAst {
    Simple {
        name: NameAst,
        annotation: DeclAnnotationAst,
        span: Span,
    },
    Extract {
        deduce: DeduceListAst,
        skeleton: CanonicalSkeletonAst,
        span: Span,
    },
    Error(ErrorAst),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeclAnnotationAst {
    Bare(ExprAst),
    TypeObjectWithRank {
        type_object_annotation: TypeObjectAnnotationAst,
        rank_annotation: ExprAst,
        span: Span,
    },
    Error(ErrorAst),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeObjectAnnotationAst {
    Expr(ExprAst),
    Hole { span: Span },
}

// --- Deduce lists ---

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeduceListAst {
    pub binders: Vec<BinderDeclAst>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BinderDeclAst {
    pub name: NameAst,
    pub annotation: Option<TypeObjectAnnotationAst>,
    pub span: Span,
}

// --- Canonical skeleton ---

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanonicalNameRole {
    Hole,
    NodeName,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CanonicalSkeletonAst {
    Segment {
        elements: Vec<CanonicalSkeletonAst>,
        span: Span,
    },
    ArgPack {
        elements: Vec<CanonicalSkeletonAst>,
        span: Span,
    },
    Wildcard {
        span: Span,
    },
    Name {
        name: NameAst,
        role: CanonicalNameRole,
        span: Span,
    },
    Path {
        names: Vec<NameAst>,
        span: Span,
    },
    Literal {
        text: String,
        span: Span,
    },
    Error(ErrorAst),
}

// --- Expression skeleton ---

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExprAst {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExprKind {
    Pipe(PipeExprAst),
    Error(ErrorAst),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PipeExprAst {
    pub segments: Vec<SegmentAst>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SegmentAst {
    pub elements: Vec<SegmentElementAst>,
    pub has_incoming: bool,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SegmentElementAst {
    OperatorExpr(OperatorExprAst),
    ArgPack(ArgPackAst),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperatorExprAst {
    pub kind: OperatorExprKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OperatorExprKind {
    Atom(AtomAst),
    Error(ErrorAst),
    // Future operator parser phase:
    // Binary { ... }
    // Postfix { ... }
    // Prefix { ... }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArgPackAst {
    pub args: Vec<ExprAst>,
    pub role: ArgPackRole,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgPackRole {
    SourcePack,
    InsertPack,
    RightTargetSubsegment,
    Unknown,
}

// --- Selectors ---

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectorAst {
    Text(NameAst),
    Numeric(NumericNameAst),
    // Future operator parser phase: Operator(OperatorSpelling) for operator selectors.
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumericNameAst {
    pub text: String,
    pub span: Span,
}

// --- Atoms ---

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtomAst {
    pub kind: AtomKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AtomKind {
    Name(NameAst),
    IntLiteral(String),
    StringLiteral(String),
    Group(Box<ExprAst>),
    Path {
        base: Box<AtomAst>,
        names: Vec<SelectorAst>,
    },
    MemberSugar {
        object: Box<AtomAst>,
        selector: SelectorAst,
    },
    DoubleDotSugar {
        object: Box<AtomAst>,
        selector: SelectorAst,
        args: ArgPackAst,
    },
    Closure(ClosureAst),
    Error(ErrorAst),
}

// --- Closure AST ---

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClosureAst {
    Inline(InlineClosureAst),
    Explicit(ExplicitClosureAst),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InlineClosureAst {
    pub head: Option<FnHeadPrefixAst>,
    pub body: BodyBlockAst,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExplicitClosureAst {
    pub head: Option<FnHeadPrefixAst>,
    pub body: BodyBlockAst,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BodyBlockAst {
    pub forms: Vec<FormAst>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FnHeadPrefixAst {
    pub deduce: Option<DeduceListAst>,
    pub captures: Option<CaptureClauseAst>,
    pub params: Option<ParamClauseAst>,
    pub fn_item_trait: Option<ExprAst>,
    pub returns: Option<ReturnClauseAst>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureClauseAst {
    pub items: Vec<CaptureItemAst>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureItemAst {
    pub expr: ExprAst,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParamClauseAst {
    pub params: Vec<ParamItemAst>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParamItemAst {
    NameParam {
        name: NameAst,
        annotation: Option<TypeObjectAnnotationAst>,
        span: Span,
    },
    ExtractParam {
        deduce: Option<DeduceListAst>,
        skeleton: CanonicalSkeletonAst,
        annotation: Option<TypeObjectAnnotationAst>,
        span: Span,
    },
    Error(ErrorAst),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReturnClauseAst {
    pub binder: ReturnBinderAst,
    pub constraint: Option<ExprAst>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReturnBinderAst {
    TypeExpr(ExprAst),
    ExtractType {
        deduce: DeduceListAst,
        skeleton: CanonicalSkeletonAst,
        span: Span,
    },
    Error(ErrorAst),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NameAst {
    pub text: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ErrorAst {
    pub message: String,
    pub span: Span,
}
