use lang_syntax::{NormExpr, NormProductElem};

use crate::{
    model::{Diagnostic, PolicyFlag, PolicySet, Provenance},
    policy_set_runtime,
};

/// Elaborate a declaration-policy expression into a symbol self-policy set.
///
/// This is intentionally restricted to declaration-policy context. `|` means
/// policy-set union here; no pattern-space or expression-level operator
/// semantics are invoked.
pub fn elaborate_declaration_policy_expr(
    policy: Option<&NormExpr>,
    fallback_provenance: Provenance,
) -> Result<PolicySet, Diagnostic> {
    let Some(policy) = policy else {
        return Ok(policy_set_runtime());
    };

    let mut set = PolicySet::new();
    collect_policy_flags(policy, &mut set, fallback_provenance)?;
    Ok(set)
}

fn collect_policy_flags(
    expr: &NormExpr,
    set: &mut PolicySet,
    fallback_provenance: Provenance,
) -> Result<(), Diagnostic> {
    match expr {
        NormExpr::Name { text, .. } => match text.as_str() {
            "meta" => {
                set.insert(PolicyFlag::Meta);
                Ok(())
            }
            "runtime" => {
                set.insert(PolicyFlag::Runtime);
                Ok(())
            }
            "export" => {
                set.insert(PolicyFlag::Export);
                Ok(())
            }
            other => Err(Diagnostic::hard_error(
                format!(
                    "invalid policy expression in declaration prefix: unknown policy `{other}`"
                ),
                Some(fallback_provenance),
            )),
        },
        NormExpr::Call { source, target, .. } => {
            let NormExpr::OperatorTarget { spelling, .. } = target.as_ref() else {
                return Err(Diagnostic::hard_error(
                    "invalid policy expression in declaration prefix: expected policy union `|`",
                    Some(fallback_provenance),
                ));
            };
            if spelling != "|" {
                return Err(Diagnostic::hard_error(
                    format!(
                        "policy expression attempted to use pattern-space operator semantics `{spelling}`; declaration policy union uses `|`"
                    ),
                    Some(fallback_provenance),
                ));
            }
            for element in &source.elements {
                let NormProductElem::Expr(expr) = element else {
                    return Err(Diagnostic::hard_error(
                        "invalid policy expression in declaration prefix: policy union operands must be names",
                        Some(fallback_provenance),
                    ));
                };
                collect_policy_flags(expr, set, fallback_provenance.clone())?;
            }
            Ok(())
        }
        _ => Err(Diagnostic::hard_error(
            "invalid policy expression in declaration prefix",
            Some(fallback_provenance),
        )),
    }
}
