use lang_syntax::NormBindingSlot;

use crate::model::{Diagnostic, Provenance, ResolverCode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnnotationContext {
    JointExtraction,
}

pub fn binding_extraction_annotation_context(slot: &NormBindingSlot) -> Option<AnnotationContext> {
    slot.annotation
        .as_ref()
        .map(|_| AnnotationContext::JointExtraction)
}

pub(crate) fn unavailable_initializer_diagnostic(provenance: Provenance) -> Diagnostic {
    Diagnostic::hard_error(
        "initializer expression consumer is unavailable; no remaining continuation, result Policy or binding has been established",
        Some(provenance),
    )
    .with_code(ResolverCode::UnsupportedInitializerExpression)
}
