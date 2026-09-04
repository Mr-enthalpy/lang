use crate::Span;

// The Raw AST covers forms, let/alias-let
// bindings, expression skeleton (pipe/segment/product), operator sugar,
// navigation/member/double-dot/bracket-call suffix sugar, closure AST,
// canonical skeletons, and deduce lists.

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
    ReturnEvent(ReturnEventAst),
    Error(ErrorAst),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReturnEventAst {
    pub value: Box<ExprAst>,
    pub target: ReturnTargetAst,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReturnTargetAst {
    ImplicitNearest { span: Span },
    Explicit { target: Box<ExprAst>, span: Span },
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
    // Optional policy specification written before `let`. `None` means the
    // policy was not written (implicit / to be inferred later), not "no
    // policy". The parser preserves component shape only; semantic validation
    // belongs to later elaboration.
    pub policy: Option<PolicySpecAst>,
    pub has_let: bool,
    pub deduce: Option<DeduceListAst>,
    pub pattern: BindingPatternAst,
    pub annotation: Option<BindingAnnotationAst>,
    pub with_clause: Option<WithClauseAst>,
    pub initializer: Option<ExprAst>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicySpecAst {
    pub value_policy: ValuePolicyPatternAst,
    pub pattern_policy: Option<PolicyConjunctionAst>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValuePolicyPatternAst {
    Conjunction(PolicyConjunctionAst),
    // Reserved semantic shape for a missing value component. No source token
    // spelling is frozen for this variant.
    Absent { span: Span },
}

/// Conjunction across orthogonal policy dimensions. Each item is a
/// same-dimension choice; the AST intentionally keeps `+` distinct from `||`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyConjunctionAst {
    pub choices: Vec<PolicyChoiceAst>,
    pub span: Span,
}

/// Alternatives within one policy dimension. `runtime || S` is the one
/// special value-presence form that combines a stage alternative with the
/// absent-value pattern whose public spelling remains Open.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyChoiceAst {
    pub atoms: Vec<PolicyAtomAst>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyAtomAst {
    Name(NameAst),
    Group {
        conjunction: Box<PolicyConjunctionAst>,
        span: Span,
    },
    /// Current strong-parser spelling `S` for the absent-value pattern. The
    /// canonical public spelling remains Open.
    AbsentValuePattern {
        span: Span,
    },
    Error(ErrorAst),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindingPatternAst {
    Binder(BinderNameAst),
    Product(ProductExtractAst),
    /// Match the remaining normalized nodes at this structural level. The
    /// ellipsis is pattern-side syntax only; it does not construct a pack
    /// value or introduce a right-value unpack operator.
    Pack {
        inner: Box<BindingPatternAst>,
        span: Span,
    },
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
    /// Pattern-side remainder constructor inside a canonical Sequence. It
    /// binds only the immediately following canonical primary.
    Pack {
        inner: Box<CanonicalSkeletonAst>,
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
    PolicyLet(PolicyLetAst),
    Pipe(PipeExprAst),
    Product(ProductExprAst),
    Error(ErrorAst),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyLetAst {
    pub policy: PolicySpecAst,
    pub operand: Box<ExprAst>,
    pub span: Span,
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
        explicit_terminated: bool,
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NavComponentAst {
    Text(NameAst),
    Operator(OperatorNameAst),
    Group(Box<ExprAst>),
    Error(ErrorAst),
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
    FloatLiteral(String),
    StringLiteral(String),
    Group(Box<ExprAst>),
    NavPath {
        components: Vec<NavComponentAst>,
        explicit_terminated: bool,
    },
    /// First-class field-function closure. Unlike `MemberSugar`, this node
    /// does not capture a receiver; its first explicit call-site argument
    /// determines `T` after invocation injects the generated self formal.
    DotClosure {
        selector: SelectorAst,
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
    /// Narrow postfix structural-member view annotation.
    ///
    /// This remains generic Raw AST shape. Only the struct decoder interprets
    /// it as member visibility; it is not an arbitrary PolicySpec.
    MemberViewAnnotation {
        object: Box<AtomAst>,
        visibility: MemberVisibilityAst,
    },
    Closure(ClosureAst),
    Error(ErrorAst),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemberVisibilityAst {
    Public,
    Private,
}

// --- Closure AST ---

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClosureAst {
    pub placement: ClosurePlacementAst,
    pub head: Option<FnHeadPrefixAst>,
    pub body: ClosureBodyAst,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClosurePlacementAst {
    InPlace,
    Ordinary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClosureBodyAst {
    Block(BodyBlockAst),
    NamedBlock {
        strategy: NameAst,
        block: BodyBlockAst,
        span: Span,
    },
    Defaulted {
        default_name: NameAst,
        span: Span,
    },
    Delete(DeleteBodyAst),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeleteBodyAst {
    /// Source spelling of the optional string literal, including quotes.
    pub message: Option<String>,
    pub delete_name: NameAst,
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
    pub call_policy: Option<PolicySpecAst>,
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
pub enum CaptureItemAst {
    /// A full let-shaped binding. `let` may be omitted when no policy prefix
    /// needs it as an anchor.
    Explicit {
        slot: BindingSlotAst,
        initializer: ExprAst,
        span: Span,
    },
    /// Source-preserving capture shorthand. Normalization must infer exactly
    /// one free non-call bare name and elaborate this into a binding.
    Inferred { initializer: ExprAst, span: Span },
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
    // Optional policy specification written before `let` (see
    // `BindingSlotAst`).
    pub policy: Option<PolicySpecAst>,
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
