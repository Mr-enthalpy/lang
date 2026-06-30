use lang_syntax::{NormBindingSlot, NormExpr};

use crate::{
    graph::{NamespaceGraphSnapshot, ResolverContext},
    meta_invocation::{ForwardedValue, MetaInvocationValue, MetaValueTarget, ReturnViewShape},
    model::{
        Diagnostic, DiagnosticSeverity, ExecutionEnv, NamespaceNodeId, PolicyEnv, PolicySet,
        Provenance, SymbolKind,
    },
    normalized_call::extract_single_call_site,
    overload_set::{
        invoke_restricted_meta_overload_with_policy, LookupPhase, RestrictedMetaInvocationOutcome,
        RestrictedOverloadFailureKind, VisibilityView,
    },
};

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

#[derive(Clone, Debug)]
pub enum EvalOutcome {
    Value {
        value: MetaInvocationValue,
        result_policy: PolicySet,
        provenance: Provenance,
    },
    Residual {
        expr: NormExpr,
        reason: ResidualReason,
        provenance: Provenance,
    },
    Diagnostic(Diagnostic),
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

pub fn evaluate_initializer_best_effort(
    snapshot: &NamespaceGraphSnapshot,
    namespace: NamespaceNodeId,
    initializer: &NormExpr,
    resolver_context: &ResolverContext,
    mode: EvalMode,
    provenance: Provenance,
) -> EvalOutcome {
    if let Some((value, result_policy)) =
        evaluate_type_name(snapshot, initializer, resolver_context)
    {
        return EvalOutcome::Value {
            value,
            result_policy,
            provenance,
        };
    }

    let call_site = match extract_single_call_site(initializer) {
        Ok(site) => site,
        Err(_) => {
            return residual_or_strict_diagnostic(
                initializer,
                ResidualReason::UnsupportedExpression,
                mode,
                provenance,
            );
        }
    };

    let outcome = invoke_restricted_meta_overload_with_policy(
        snapshot,
        namespace,
        &call_site,
        resolver_context,
        LookupPhase::MetaAction,
        ExecutionEnv::Meta,
        VisibilityView::Internal,
        provenance.clone(),
    );

    match outcome {
        RestrictedMetaInvocationOutcome::Value {
            value,
            result_policy,
        } => EvalOutcome::Value {
            value,
            result_policy,
            provenance,
        },
        RestrictedMetaInvocationOutcome::Diagnostic {
            diagnostic,
            failure_kind,
        } => match residual_reason_from_failure_kind(&failure_kind) {
            Some(reason) => residual_or_strict_diagnostic(initializer, reason, mode, provenance),
            None => EvalOutcome::Diagnostic(diagnostic),
        },
    }
}

fn evaluate_type_name(
    snapshot: &NamespaceGraphSnapshot,
    initializer: &NormExpr,
    resolver_context: &ResolverContext,
) -> Option<(MetaInvocationValue, PolicySet)> {
    let NormExpr::Name { text, .. } = initializer else {
        return None;
    };
    let symbol = snapshot
        .capability()
        .resolve_type_object_with_policy(text, resolver_context, PolicyEnv::Meta)
        .ok()?;
    (symbol.kind == SymbolKind::Type).then(|| {
        (
            MetaInvocationValue::ForwardedValue(ForwardedValue {
                target: MetaValueTarget::TypeSymbol(symbol.id),
                return_view: ReturnViewShape::Leaf,
                provenance: symbol.provenance.clone(),
            }),
            symbol.policy_metadata.policy_set,
        )
    })
}

fn residual_or_strict_diagnostic(
    initializer: &NormExpr,
    reason: ResidualReason,
    mode: EvalMode,
    provenance: Provenance,
) -> EvalOutcome {
    match mode {
        EvalMode::MetaPartial => EvalOutcome::Residual {
            expr: initializer.clone(),
            reason,
            provenance,
        },
        EvalMode::MetaStrict => EvalOutcome::Diagnostic(Diagnostic::hard_error(
            format!(
                "ResidualNotAllowedInMetaStrict: runtime-only dependency in MetaStrict context ({reason:?})"
            ),
            Some(provenance),
        )
        .with_code(crate::ResolverCode::ResidualNotAllowedInMetaStrict)),
    }
}

fn residual_reason_from_failure_kind(
    failure_kind: &RestrictedOverloadFailureKind,
) -> Option<ResidualReason> {
    match failure_kind {
        RestrictedOverloadFailureKind::NoSourceDeclaredCallable { .. }
        | RestrictedOverloadFailureKind::NotVisibleToLookupPhase { .. }
        | RestrictedOverloadFailureKind::NoApplicableCandidate { .. } => {
            Some(ResidualReason::NoMetaVisibleCandidate)
        }
        RestrictedOverloadFailureKind::BodyEntryPolicyMismatch { .. } => {
            Some(ResidualReason::BodyEntryPolicyMismatch)
        }
        RestrictedOverloadFailureKind::AmbiguousCandidate { .. }
        | RestrictedOverloadFailureKind::InvalidTarget
        | RestrictedOverloadFailureKind::UnsupportedExternalVisibility
        | RestrictedOverloadFailureKind::UnsupportedCandidateShape
        | RestrictedOverloadFailureKind::UnsupportedParameterPattern
        | RestrictedOverloadFailureKind::UnsupportedCanonicalSumPatternValue
        | RestrictedOverloadFailureKind::UnsupportedSelectedMetaBody
        | RestrictedOverloadFailureKind::UnsupportedSelectedMetaBodyLocalBinding
        | RestrictedOverloadFailureKind::SelectedDeleteBodyDiagnostic => None,
    }
}

pub fn residual_diagnostic(reason: &ResidualReason, provenance: Provenance) -> Diagnostic {
    Diagnostic::new(
        DiagnosticSeverity::Error,
        format!("initializer residualized to runtime: {reason:?}"),
        Some(provenance),
    )
}
