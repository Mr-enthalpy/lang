//! Call-target resolution boundary.
//!
//! Resolves a `NormalizedCallSite.target` through the namespace graph to a
//! callable `SymbolObject`.
//!
//! ## Temporary shortcut (v0.8)
//!
//! `resolve_call_target` treats the resolved target symbol as the callable
//! entry directly. The full pipeline is:
//!
//! ```text
//! target expression → target value → target type →
//!     type-associated namespace → `()` call entry
//! ```
//!
//! This shortcut will be replaced when function-object types and associated
//! call-entry insertion are implemented. Until then, **no** parser, normalizer,
//! or candidate-prep logic may special-case target names by string.
//!
//! The current implementation boundary lives in `lang_build::product_shape`,
//! `lang_build::identity`, `lang_build::meta_candidate`, and
//! `lang_build::call_target`. These are substrate boundaries, not full
//! implementations of the future systems.

use lang_syntax::{NormExpr, NormNavComponent};

use crate::{
    graph::{NamespaceGraphCapability, ResolveExpectation, ResolverContext},
    model::{Diagnostic, PolicyEnv, Provenance, ResolverCode, SymbolObject},
};

#[derive(Clone, Debug)]
pub struct ResolvedCallTarget {
    pub callee: SymbolObject,
    /// Always `true` in v0.8 — target symbol is used directly as the callable
    /// entry. Future: set to `false` when `target → type → associated namespace
    /// → ()` lookup is implemented.
    pub temporary_direct_callable_shortcut: bool,
    pub provenance: Provenance,
}

/// Resolve the target of a `NormalizedCallSite` to a callable `SymbolObject`.
///
/// The target must be a `NormExpr::Name` or `NormExpr::Nav` whose components are
/// all names. It is resolved through the namespace graph as a policy-visible
/// meta-function first, then as a policy-visible object. Unknown / unresolved
/// targets return `None` (not an error).
/// Ambiguous or conflicting resolutions return a diagnostic.
pub fn resolve_call_target(
    target: &NormExpr,
    capability: &NamespaceGraphCapability<'_>,
    context: &ResolverContext,
    policy_env: PolicyEnv,
) -> Result<Option<ResolvedCallTarget>, Diagnostic> {
    let target_path = match expr_to_target_path(target) {
        Some(path) => path,
        None => return Ok(None),
    };

    let meta_symbol = match capability.resolve_with_policy(
        &target_path,
        context,
        ResolveExpectation::MetaFunction,
        policy_env,
    ) {
        Ok(symbol) => Some(symbol),
        Err(diagnostic) => match diagnostic.code {
            Some(ResolverCode::Unresolved) | None => None,
            _ => return Err(diagnostic),
        },
    };
    let target_symbol = match meta_symbol {
        Some(symbol) => symbol,
        None => match capability.resolve_with_policy(
            &target_path,
            context,
            ResolveExpectation::Object,
            policy_env,
        ) {
            Ok(symbol) => symbol,
            Err(diagnostic) => match diagnostic.code {
                Some(ResolverCode::Unresolved) | None => return Ok(None),
                _ => return Err(diagnostic),
            },
        },
    };

    let provenance =
        Provenance::from_norm_origin("ResolvedCallTarget (v0.8 shortcut)", expr_origin(target));
    Ok(Some(ResolvedCallTarget {
        callee: target_symbol,
        temporary_direct_callable_shortcut: true,
        provenance,
    }))
}

fn expr_to_target_path(expr: &NormExpr) -> Option<Vec<String>> {
    match expr {
        NormExpr::Name { text, .. } => Some(vec![text.clone()]),
        NormExpr::Nav { components, .. } => {
            let mut path = Vec::new();
            for component in components {
                match component {
                    NormNavComponent::Name { name, .. } => path.push(name.clone()),
                    _ => return None,
                }
            }
            Some(path)
        }
        _ => None,
    }
}

fn expr_origin(expr: &NormExpr) -> &lang_syntax::NormOrigin {
    match expr {
        NormExpr::Call { origin, .. }
        | NormExpr::Name { origin, .. }
        | NormExpr::Literal { origin, .. }
        | NormExpr::Nav { origin, .. }
        | NormExpr::OperatorTarget { origin, .. }
        | NormExpr::Unsupported { origin, .. } => origin,
        NormExpr::Product(product) => &product.origin,
        NormExpr::Closure(closure) => &closure.origin,
        NormExpr::Error(error) => &error.origin,
    }
}
