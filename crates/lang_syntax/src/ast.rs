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
    AliasLet(LetAliasAst),
    Expr(ExprAst),
    Error(ErrorAst),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LetAst {
    pub slot: BindingSlotAst,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WithClauseAst {
    pub kind: WithClauseKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WithClauseKind {
    Empty,
    Items { items: Vec<NameAst> },
    Error(ErrorAst),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingSlotAst {
    // Optional policy expression written before `let`. `None` means the policy
    // was not written (implicit / to be inferred later), not "no policy". The
    // parser preserves the expression shape only; it performs no validation.
    pub policy: Option<ExprAst>,
    pub has_let: bool,
    pub deduce: Option<DeduceListAst>,
    pub pattern: BindingPatternAst,
    pub annotation: Option<BindingAnnotationAst>,
    pub with_clause: Option<WithClauseAst>,
    pub initializer: Option<ExprAst>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindingPatternAst {
    Binder(BinderNameAst),
    Product(ProductExtractAst),
    Skeleton(CanonicalSkeletonAst),
    Error(ErrorAst),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BinderNameAst {
    Text(NameAst),
    Operator(OperatorNameAst),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindingAnnotationAst {
    Expr(ExprAst),
    Compound {
        left: AnnotationTermAst,
        right: ExprAst,
        span: Span,
    },
    Error(ErrorAst),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnnotationTermAst {
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
    pub annotation: Option<AnnotationTermAst>,
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
    ProductExtract {
        elements: Vec<CanonicalProductElementAst>,
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
    NavPath {
        names: Vec<NameAst>,
        span: Span,
    },
    Literal {
        text: String,
        span: Span,
    },
    Error(ErrorAst),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CanonicalProductElementAst {
    Skeleton(CanonicalSkeletonAst),
    Unit { span: Span },
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
    Product(ProductExprAst),
    Error(ErrorAst),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductExprAst {
    pub elements: Vec<ProductElementAst>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProductElementAst {
    Expr(ExprAst),
    Unit { span: Span },
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
    Product(ProductExprAst),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperatorExprAst {
    pub kind: OperatorExprKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OperatorExprKind {
    Atom(AtomAst),
    Product(ProductExprAst),
    OperatorSugar {
        operator: OperatorNameAst,
        fixity: OperatorFixity,
        args: Vec<OperatorExprAst>,
        span: Span,
    },
    NavPath {
        components: Vec<NavComponentAst>,
        span: Span,
    },
    MemberSugar {
        object: Box<OperatorExprAst>,
        selector: SelectorAst,
        span: Span,
    },
    DoubleDotSugar {
        object: Box<OperatorExprAst>,
        selector: SelectorAst,
        args: ProductExprAst,
        span: Span,
    },
    // `obj[args...]` bracket-call sugar for the operator spelling `[]`.
    // Source-preserving; not indexing/slicing/container access.
    BracketCallSugar {
        object: Box<OperatorExprAst>,
        operator: OperatorNameAst,
        args: ProductExprAst,
        span: Span,
    },
    Error(ErrorAst),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperatorNameAst {
    pub spelling: String,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperatorFixity {
    Prefix,
    Postfix,
    Binary,
}

// --- Selectors ---

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectorAst {
    Text(NameAst),
    Numeric(NumericNameAst),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NavComponentAst {
    Text(NameAst),
    Numeric(NumericNameAst),
    Operator(OperatorNameAst),
    Group(Box<ExprAst>),
    Error(ErrorAst),
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
    NavPath {
        components: Vec<NavComponentAst>,
    },
    MemberSugar {
        object: Box<AtomAst>,
        selector: SelectorAst,
    },
    DoubleDotSugar {
        object: Box<AtomAst>,
        selector: SelectorAst,
        args: ProductExprAst,
    },
    // `obj[args...]` bracket-call sugar for the operator spelling `[]`.
    BracketCallSugar {
        object: Box<AtomAst>,
        operator: OperatorNameAst,
        args: ProductExprAst,
    },
    Closure(ClosureAst),
    Error(ErrorAst),
}

// --- Closure AST ---

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClosureAst {
    InPlace(InPlaceClosureAst),
    Explicit(ExplicitClosureAst),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InPlaceClosureAst {
    pub body: BodyBlockAst,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExplicitClosureAst {
    pub head: FnHeadPrefixAst,
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
    pub clauses: Vec<HeadClauseAst>,
    pub span: Span,
}

// Source-preserving closure/function head clauses. Each clause holds exactly
// one raw expression slot. The parser does not decide whether the expression
// is a valid contract, lifetime condition, resource condition, type-level
// object, rank-level object, or semantic predicate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HeadClauseAst {
    Require { expr: ExprAst, span: Span },
    Pre { expr: ExprAst, span: Span },
    Post { expr: ExprAst, span: Span },
    LifetimePre { expr: ExprAst, span: Span },
    LifetimePost { expr: ExprAst, span: Span },
    Error(ErrorAst),
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
    pub extract: ProductExtractAst,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductExtractAst {
    pub elements: Vec<ProductExtractElementAst>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProductExtractElementAst {
    Slot(BindingSlotAst),
    Unit { span: Span },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReturnClauseAst {
    pub slot: BindingSlotAst,
    pub span: Span,
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

// --- Alias binding ---

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LetAliasAst {
    // Optional policy expression written before `let` (see `BindingSlotAst`).
    pub policy: Option<ExprAst>,
    pub binder: AliasBinderAst,
    pub target: EntityRefAst,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AliasBinderAst {
    Name(NameAst),
    Operator(OperatorNameAst),
    Error(ErrorAst),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityRefAst {
    pub components: Vec<NavComponentAst>,
    pub span: Span,
}
