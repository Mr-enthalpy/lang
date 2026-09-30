use lang_syntax::NormBindingSlot;

use crate::model::{Diagnostic, Provenance, ResolverCode};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResidualReason {
    UnsupportedExpression,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnnotationContext {
    Assertion,
    /// Reserved for rank-pattern declaration grammar. Initializer evaluation
    /// does not classify any current binding annotation as rank-pattern
    /// material.
    RankPattern,
}

pub fn binding_assertion_annotation_context(slot: &NormBindingSlot) -> Option<AnnotationContext> {
    slot.annotation
        .as_ref()
        .map(|_| AnnotationContext::Assertion)
}

pub fn residual_diagnostic(reason: &ResidualReason, provenance: Provenance) -> Diagnostic {
    Diagnostic::hard_error(
        format!("initializer continuation preservation consumer is not connected: {reason:?}; incomplete evaluation establishes no result Policy or binding"),
        Some(provenance),
    )
    .with_code(ResolverCode::UnsupportedInitializerContinuation)
}
