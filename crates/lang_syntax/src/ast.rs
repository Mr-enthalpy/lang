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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LetBinderAst {
    Simple {
        name: NameAst,
        annotation: DeclAnnotationAst,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExprAst {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExprKind {
    Segment(Vec<AtomAst>),
    Error(ErrorAst),
}

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
    Path {
        base: Box<AtomAst>,
        names: Vec<NameAst>,
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
