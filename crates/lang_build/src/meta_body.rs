//! Build/check semantics for `delete` closure bodies.
//!
//! `delete` is a contextual explicit-closure body terminator introduced by
//! the `=> ("msg") delete` syntax. This module provides the minimal
//! build/check substrate:
//!
//! 1. Legality check: `Delete` bodies are only valid in statically executed
//!    closures; runtime-only closures reject them.
//! 2. Selected-body evaluation: when a selected static closure body is a
//!    `Delete` body, it produces a hard static diagnostic.
//!
//! `delete` is not a primitive callable, not a value, not `assert`, and
//! not `panic`. It remains `NormClosureBody::Delete` through normalization.

use lang_syntax::{NormClosureBody, NormDeleteBody};

use crate::model::{Diagnostic, DiagnosticSeverity, Provenance};

// ---------------------------------------------------------------------------
// Execution environment
// ---------------------------------------------------------------------------

/// The execution environment a closure body is demanded under.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClosureBodyExecutionEnv {
    OpenStatic,
    SealStatic,
    Runtime,
}

// ---------------------------------------------------------------------------
// Legality check
// ---------------------------------------------------------------------------

/// Check whether a closure body is legal in the given execution environment.
///
/// - `Block` bodies are legal in every phase.
/// - `Delete` bodies are legal only in a static phase.
///
/// If illegal, returns a `Diagnostic` describing the violation.
pub fn check_closure_body_delete_legality(
    body: &NormClosureBody,
    env: ClosureBodyExecutionEnv,
    fallback_provenance: Provenance,
) -> Result<(), Diagnostic> {
    match body {
        NormClosureBody::Block(_)
        | NormClosureBody::NamedBlock { .. }
        | NormClosureBody::Defaulted { .. } => Ok(()),
        NormClosureBody::Delete(del) => match env {
            ClosureBodyExecutionEnv::OpenStatic | ClosureBodyExecutionEnv::SealStatic => Ok(()),
            ClosureBodyExecutionEnv::Runtime => Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                "delete closure body is only valid in static bodies".to_string(),
                Some(del.origin_reprovenance(&fallback_provenance)),
            )),
        },
    }
}

// ---------------------------------------------------------------------------
// Selected meta delete evaluation
// ---------------------------------------------------------------------------

/// Strip the outer double-quote characters from a normalized string
/// literal text.  For a normalized representation like `"\"msg\""` this
/// yields `msg`.
fn strip_string_literal_payload(quoted: &str) -> String {
    let mut result = String::with_capacity(quoted.len());
    let chars: Vec<char> = quoted.chars().collect();

    if chars.len() < 2 {
        return quoted.to_string();
    }

    // Skip opening quote
    let mut i = 1;
    while i < chars.len() - 1 {
        if chars[i] == '\\' && i + 1 < chars.len() - 1 {
            // Simple escape sequence — skip the backslash and emit the
            // next character. This handles `\"` and `\\` correctly.
            i += 1;
            result.push(chars[i]);
        } else {
            result.push(chars[i]);
        }
        i += 1;
    }

    result
}

/// Build a hard static diagnostic from a selected meta `Delete` body.
///
/// The diagnostic message carries the string payload with a `meta delete:`
/// prefix. Non-string messages cannot reach this typed normalized node.
pub fn selected_meta_delete_diagnostic(
    delete: &NormDeleteBody,
    fallback_provenance: Provenance,
) -> Diagnostic {
    let provenance = delete.origin_reprovenance(&fallback_provenance);
    let Some(message) = delete.message.as_deref() else {
        return Diagnostic::new(
            DiagnosticSeverity::Error,
            "meta delete: selected callable is deleted".to_string(),
            Some(provenance),
        );
    };
    let message = strip_string_literal_payload(message);
    Diagnostic::new(
        DiagnosticSeverity::Error,
        format!("meta delete: {message}"),
        Some(provenance),
    )
}

// ---------------------------------------------------------------------------
// Selected meta body evaluation
// ---------------------------------------------------------------------------

/// Outcome of evaluating a selected meta closure body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectedMetaBodyEvaluation {
    /// The body is a `Block` — full meta evaluation is deferred.
    DeferredBlock,
    /// Compiler default implementation generation is deferred to the
    /// callable's default rule.
    Defaulted,
    /// The body is a `Delete` — evaluation produces a static diagnostic.
    DeleteDiagnostic(Diagnostic),
}

/// Evaluate a selected meta closure body.
///
/// - `Block` → `DeferredBlock` (full meta evaluation not yet implemented).
/// - `Delete` → `DeleteDiagnostic` carrying the delete message.
pub fn evaluate_selected_meta_closure_body(
    body: &NormClosureBody,
    fallback_provenance: Provenance,
) -> SelectedMetaBodyEvaluation {
    match body {
        NormClosureBody::Block(_) | NormClosureBody::NamedBlock { .. } => {
            SelectedMetaBodyEvaluation::DeferredBlock
        }
        NormClosureBody::Defaulted { .. } => SelectedMetaBodyEvaluation::Defaulted,
        NormClosureBody::Delete(del) => SelectedMetaBodyEvaluation::DeleteDiagnostic(
            selected_meta_delete_diagnostic(del, fallback_provenance),
        ),
    }
}

// ---------------------------------------------------------------------------
// Provenance helper for NormDeleteBody
// ---------------------------------------------------------------------------

/// Extract the origin of a `NormDeleteBody` and re-provenance it
/// with a fallback if no origin span is available.
trait DeleteOrigin {
    fn origin_reprovenance(&self, fallback: &Provenance) -> Provenance;
}

impl DeleteOrigin for NormDeleteBody {
    fn origin_reprovenance(&self, fallback: &Provenance) -> Provenance {
        match &self.origin {
            lang_syntax::NormOrigin::Source(span) => Provenance {
                description: format!("delete body at {}:{}", span.line, span.column),
                file: None,
                span: Some(*span),
            },
            _ => fallback.clone(),
        }
    }
}
