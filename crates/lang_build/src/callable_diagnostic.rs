//! Selected closure-body diagnostic substrate.
//!
//! `delete` remains `NormClosureBody::Delete` through normalization.
//! Horizon visibility is not an execution-legality proof; this module does
//! not infer body legality from the observation horizon.

use lang_syntax::NormDeleteBody;

use crate::model::{Diagnostic, DiagnosticSeverity, Provenance};
// ---------------------------------------------------------------------------
// Selected callable delete diagnostic
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

/// Build the diagnostic for an ordinary selected `Delete` body.
///
/// The diagnostic message carries the string payload with a `selected delete:`
/// prefix. Non-string messages cannot reach this typed normalized node.
pub fn selected_callable_delete_diagnostic(
    delete: &NormDeleteBody,
    fallback_provenance: Provenance,
) -> Diagnostic {
    let provenance = delete.origin_reprovenance(&fallback_provenance);
    let Some(message) = delete.message.as_deref() else {
        return Diagnostic::new(
            DiagnosticSeverity::Error,
            "selected delete: selected callable is deleted".to_string(),
            Some(provenance),
        );
    };
    let message = strip_string_literal_payload(message);
    Diagnostic::new(
        DiagnosticSeverity::Error,
        format!("selected delete: {message}"),
        Some(provenance),
    )
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
                file: fallback.file.clone(),
                span: Some(*span),
            },
            _ => fallback.clone(),
        }
    }
}
