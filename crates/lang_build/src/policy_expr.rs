use lang_syntax::NormPolicySpec;

use crate::{
    elaborate_p1_projection,
    model::{Diagnostic, PolicyFlag, PolicySet, Provenance},
    NamespaceVisibility, P1Projection, PolicyPair, PolicyStage, StageSet,
};

/// Elaborate a declaration-prefix policy as a P1 projection and expose its
/// stage/legacy-export portion to the current flat resolver substrate.
///
/// The canonical result is [`P1Projection`]. This adapter must not be used to
/// reconstruct value mutability, value presence, or the pattern component.
pub fn elaborate_declaration_policy_expr(
    policy: Option<&NormPolicySpec>,
    fallback_provenance: Provenance,
) -> Result<PolicySet, Diagnostic> {
    let projection = elaborate_p1_projection(policy, fallback_provenance)?;
    Ok(legacy_policy_set_from_p1(&projection))
}

pub fn legacy_policy_set_from_p1(projection: &P1Projection) -> PolicySet {
    match projection {
        P1Projection::Infer => PolicySet::new(),
        P1Projection::ValueDominant {
            value,
            namespace_visibility,
        } => legacy_policy_set(&value.stages, *namespace_visibility),
        P1Projection::Pair(pair) => legacy_policy_set_from_pair(pair),
    }
}

pub fn legacy_policy_set_from_pair(pair: &PolicyPair) -> PolicySet {
    let stages = if pair.value.stages.is_empty() {
        &pair.pattern.stages
    } else {
        &pair.value.stages
    };
    legacy_policy_set(stages, pair.namespace_visibility)
}

fn legacy_policy_set(
    stages: &StageSet,
    namespace_visibility: Option<NamespaceVisibility>,
) -> PolicySet {
    let mut set = PolicySet::new();
    for stage in stages.iter() {
        set.insert(match stage {
            PolicyStage::Meta => PolicyFlag::Meta,
            PolicyStage::Compile => PolicyFlag::Compile,
            PolicyStage::Seal => PolicyFlag::Seal,
            PolicyStage::Runtime => PolicyFlag::Runtime,
        });
    }
    if namespace_visibility == Some(NamespaceVisibility::Export) {
        set.insert(PolicyFlag::Export);
    }
    set
}
