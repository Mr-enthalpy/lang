//! Unified semantic invocation result universe.
//!
//! Evaluation stage (`meta`, `compile`, or `runtime`) does not define a
//! separate value ontology.  Every selected callable reports the result class
//! declared by that callable and then produces exactly one of:
//!
//! * a semantic value in that declared class;
//! * an opaque residual owned by a later evaluator boundary; or
//! * a diagnostic.
//!
//! The payload is generic because the current vertical slice still has more
//! than one storage carrier behind the common semantic boundary.  The generic
//! parameter is carrier compatibility only; it never changes the result
//! universe or the declared result class.

use crate::{Diagnostic, Provenance};

/// Semantic class declared for one callable result.
///
/// `Extension` is an explicit future-work gate.  It lets later canonical
/// result classes cross this boundary without treating this provisional Rust
/// representation as a closed language ontology.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeclaredResultClass {
    Unit,
    OrdinaryValue,
    CompleteType,
    ClusterSymbol,
    Extension(String),
}

/// Opaque residual crossing the unified invocation boundary.
///
/// The evaluator that owns `class` also owns the payload interpretation.  This
/// carrier deliberately makes no claim about the still-open residual IR/ABI.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvocationResidual {
    pub class: String,
    pub provenance: Provenance,
}

/// Result of invoking a uniquely selected callable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InvocationResult<T> {
    SemanticResult {
        declared_result_class: DeclaredResultClass,
        value: T,
    },
    Residual(InvocationResidual),
    Diagnostic(Diagnostic),
}

impl<T> InvocationResult<T> {
    pub fn semantic(declared_result_class: DeclaredResultClass, value: T) -> Self {
        Self::SemanticResult {
            declared_result_class,
            value,
        }
    }

    pub fn map_semantic<U>(self, map: impl FnOnce(T) -> U) -> InvocationResult<U> {
        match self {
            Self::SemanticResult {
                declared_result_class,
                value,
            } => InvocationResult::SemanticResult {
                declared_result_class,
                value: map(value),
            },
            Self::Residual(residual) => InvocationResult::Residual(residual),
            Self::Diagnostic(diagnostic) => InvocationResult::Diagnostic(diagnostic),
        }
    }
}
