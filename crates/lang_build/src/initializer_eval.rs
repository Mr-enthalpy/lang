use lang_syntax::NormBindingSlot;

use crate::model::{Diagnostic, DiagnosticSeverity, Provenance};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvalMode {
    MetaPartial,
    MetaStrict,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResidualReason {
    UnsupportedExpression,
    NoMetaVisibleCandidate,
    BodyEntryPolicyMismatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnnotationContext {
    Assertion,
    /// Reserved for future rank-pattern declaration grammar. The v0.8
    /// initializer evaluator does not classify any current binding annotation
    /// as rank-pattern material.
    RankPattern,
}

pub fn binding_assertion_annotation_context(slot: &NormBindingSlot) -> Option<AnnotationContext> {
    slot.annotation
        .as_ref()
        .map(|_| AnnotationContext::Assertion)
}

pub fn residual_diagnostic(reason: &ResidualReason, provenance: Provenance) -> Diagnostic {
    Diagnostic::new(
        DiagnosticSeverity::Error,
        format!("initializer residualized to runtime: {reason:?}"),
        Some(provenance),
    )
}
