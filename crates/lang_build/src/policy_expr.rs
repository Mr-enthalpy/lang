use lang_syntax::NormPolicySpec;

use crate::{
    elaborate_namespace_declaration_policy,
    model::{Diagnostic, PolicyFlag, PolicySet, Provenance},
    NamespaceDeclarationPolicy, NamespaceDeclarationPosition, P1Projection, PolicyPair,
    PolicyStage, StageSet,
};

/// Elaborate a declaration-prefix policy as a P1 projection and expose its
/// stage/legacy-export portion to the current flat resolver substrate.
///
/// The canonical result is [`P1Projection`]. This adapter must not be used to
/// reconstruct value mutability, value presence, the pattern component, or the
/// separate const-projected external export view.
pub fn elaborate_declaration_policy_expr(
    policy: Option<&NormPolicySpec>,
    fallback_provenance: Provenance,
) -> Result<PolicySet, Diagnostic> {
    let declaration = elaborate_namespace_declaration_policy(
        policy,
        NamespaceDeclarationPosition::DirectTopLevel,
        fallback_provenance,
    )?;
    Ok(legacy_policy_set_from_namespace_declaration(&declaration))
}

pub fn legacy_policy_set_from_p1(projection: &P1Projection) -> PolicySet {
    match projection {
        P1Projection::Infer => PolicySet::new(),
        P1Projection::ValueDominant { value } => legacy_policy_set(&value.stages, false),
        P1Projection::Pair(pair) => legacy_policy_set_from_pair(pair),
    }
}

pub fn legacy_policy_set_from_namespace_declaration(
    declaration: &NamespaceDeclarationPolicy,
) -> PolicySet {
    let mut set = legacy_policy_set_from_p1(&declaration.projection);
    if declaration.export_root {
        set.insert(PolicyFlag::Export);
    }
    set
}

pub fn legacy_policy_set_from_pair(pair: &PolicyPair) -> PolicySet {
    let stages = if pair.value.stages.is_empty() {
        &pair.pattern.stages
    } else {
        &pair.value.stages
    };
    legacy_policy_set(stages, false)
}

fn legacy_policy_set(stages: &StageSet, export_root: bool) -> PolicySet {
    let mut set = PolicySet::new();
    for stage in stages.iter() {
        set.insert(match stage {
            PolicyStage::Meta => PolicyFlag::Meta,
            PolicyStage::Compile => PolicyFlag::Compile,
            PolicyStage::Seal => PolicyFlag::Seal,
            PolicyStage::Runtime => PolicyFlag::Runtime,
        });
    }
    if export_root {
        set.insert(PolicyFlag::Export);
    }
    set
}
