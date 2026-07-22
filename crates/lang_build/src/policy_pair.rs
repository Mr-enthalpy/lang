use std::collections::BTreeSet;

use lang_syntax::{NormExpr, NormPolicySpec, NormProductElem, NormValuePolicyPattern};

use crate::{Diagnostic, Provenance};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PolicyStage {
    Meta,
    Compile,
    Seal,
    Runtime,
}

impl PolicyStage {
    pub fn is_static(self) -> bool {
        !matches!(self, Self::Runtime)
    }

    pub fn visible_at(self, lookup: PolicyLookupStage) -> bool {
        match self {
            Self::Meta => matches!(
                lookup,
                PolicyLookupStage::OpenMeta | PolicyLookupStage::Compile
            ),
            Self::Compile => matches!(
                lookup,
                PolicyLookupStage::OpenMeta
                    | PolicyLookupStage::Compile
                    | PolicyLookupStage::Seal
                    | PolicyLookupStage::PostSealCompile
            ),
            Self::Seal => matches!(
                lookup,
                PolicyLookupStage::Compile
                    | PolicyLookupStage::Seal
                    | PolicyLookupStage::PostSealCompile
            ),
            Self::Runtime => matches!(lookup, PolicyLookupStage::Runtime),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StageSet(BTreeSet<PolicyStage>);

impl StageSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, stage: PolicyStage) {
        self.0.insert(stage);
    }

    pub fn contains(&self, stage: PolicyStage) -> bool {
        self.0.contains(&stage)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = PolicyStage> + '_ {
        self.0.iter().copied()
    }

    pub fn static_stages(&self) -> Self {
        Self(
            self.0
                .iter()
                .copied()
                .filter(|stage| stage.is_static())
                .collect(),
        )
    }

    pub fn union(&self, other: &Self) -> Self {
        Self(self.0.union(&other.0).copied().collect())
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.0.iter().any(|stage| other.contains(*stage))
    }

    pub fn visible_at(&self, lookup: PolicyLookupStage) -> bool {
        self.0.iter().any(|stage| stage.visible_at(lookup))
    }
}

impl<const N: usize> From<[PolicyStage; N]> for StageSet {
    fn from(stages: [PolicyStage; N]) -> Self {
        Self(stages.into_iter().collect())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueMutability {
    Const,
    Mut,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NamespaceVisibility {
    Public,
    Private,
    Export,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValuePresence {
    Present,
    Optional,
    Absent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValueComponentPolicy {
    pub stages: StageSet,
    pub mutability: BTreeSet<ValueMutability>,
    pub presence: ValuePresence,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternComponentPolicy {
    pub stages: StageSet,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyPair {
    pub value: ValueComponentPolicy,
    pub pattern: PatternComponentPolicy,
    pub namespace_visibility: Option<NamespaceVisibility>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum P1Projection {
    Infer,
    ValueDominant {
        value: ValueComponentPolicy,
        namespace_visibility: Option<NamespaceVisibility>,
    },
    Pair(PolicyPair),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyLookupStage {
    OpenMeta,
    Compile,
    Seal,
    PostSealCompile,
    Runtime,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionObjectDeclarationPolicy {
    pub mutability: BTreeSet<ValueMutability>,
    pub namespace_visibility: Option<NamespaceVisibility>,
}

impl Default for FunctionObjectDeclarationPolicy {
    fn default() -> Self {
        Self {
            mutability: BTreeSet::new(),
            namespace_visibility: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyResultEntry<V, P> {
    pub value: Option<V>,
    pub value_policy: ValueComponentPolicy,
    pub pattern: P,
    pub pattern_policy: PatternComponentPolicy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FunctionMemberKind {
    Concrete,
    MaterializedInstance,
    GenericTemplate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionMember<I> {
    pub id: I,
    pub kind: FunctionMemberKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionObject<I> {
    pub symbol_identity: I,
    pub anonymous_type_identity: I,
    pub members: Vec<FunctionMember<I>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FunctionSliceStage {
    Runtime,
    Seal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionObjectView<I> {
    pub symbol_identity: I,
    pub anonymous_type_identity: I,
    pub member_ids: Vec<I>,
}

impl<I: Clone> FunctionObject<I> {
    pub fn slice(&self, stage: FunctionSliceStage) -> FunctionObjectView<I> {
        let member_ids = self
            .members
            .iter()
            .filter(|member| match stage {
                FunctionSliceStage::Runtime => {
                    matches!(member.kind, FunctionMemberKind::Concrete)
                }
                FunctionSliceStage::Seal => matches!(
                    member.kind,
                    FunctionMemberKind::Concrete | FunctionMemberKind::MaterializedInstance
                ),
            })
            .map(|member| member.id.clone())
            .collect();
        FunctionObjectView {
            symbol_identity: self.symbol_identity.clone(),
            anonymous_type_identity: self.anonymous_type_identity.clone(),
            member_ids,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SealWorldSnapshot<T> {
    pre_seal: Vec<T>,
    seal_generated: Vec<T>,
}

impl<T> SealWorldSnapshot<T> {
    pub fn new(pre_seal: Vec<T>) -> Self {
        Self {
            pre_seal,
            seal_generated: Vec::new(),
        }
    }

    pub fn scan_domain(&self) -> &[T] {
        &self.pre_seal
    }

    pub fn push_seal_generated(&mut self, value: T) {
        self.seal_generated.push(value);
    }

    pub fn seal_generated(&self) -> &[T] {
        &self.seal_generated
    }

    pub fn final_world(&self) -> impl Iterator<Item = &T> {
        self.pre_seal.iter().chain(self.seal_generated.iter())
    }
}

#[derive(Default)]
struct ComponentAtoms {
    stages: StageSet,
    mutability: BTreeSet<ValueMutability>,
    namespace: BTreeSet<NamespaceVisibility>,
}

pub fn normalize_p2_policy(
    policy: &NormPolicySpec,
    provenance: Provenance,
) -> Result<PolicyPair, Diagnostic> {
    match (&policy.value_policy, &policy.type_policy) {
        (NormValuePolicyPattern::Expr(value), None) => {
            let atoms = parse_component(value, provenance.clone())?;
            reject_namespace_in_p2(&atoms, provenance.clone())?;
            let static_stages = atoms.stages.static_stages();
            if static_stages.len() > 1 {
                return Err(policy_error(
                    "P2 single-policy form contains more than one static stage",
                    provenance,
                ));
            }
            let pattern_stages = if static_stages.is_empty() {
                if !atoms.stages.contains(PolicyStage::Runtime) {
                    return Err(policy_error(
                        "P2 single-policy form requires a stage",
                        provenance,
                    ));
                }
                StageSet::from([PolicyStage::Seal])
            } else {
                static_stages
            };
            validate_p2_pair(
                PolicyPair {
                    value: ValueComponentPolicy {
                        stages: atoms.stages,
                        mutability: atoms.mutability,
                        presence: ValuePresence::Present,
                    },
                    pattern: PatternComponentPolicy {
                        stages: pattern_stages,
                    },
                    namespace_visibility: None,
                },
                provenance,
            )
        }
        (value_pattern, Some(pattern)) => {
            let value_atoms = match value_pattern {
                NormValuePolicyPattern::Expr(value) => parse_component(value, provenance.clone())?,
                NormValuePolicyPattern::Absent { .. } => ComponentAtoms::default(),
            };
            let pattern_atoms = parse_component(pattern, provenance.clone())?;
            reject_namespace_in_p2(&value_atoms, provenance.clone())?;
            reject_namespace_in_p2(&pattern_atoms, provenance.clone())?;
            if !pattern_atoms.mutability.is_empty() {
                return Err(policy_error(
                    "const/mut policy is valid only in the P2 value component",
                    provenance,
                ));
            }
            validate_p2_pair(
                PolicyPair {
                    value: ValueComponentPolicy {
                        stages: value_atoms.stages,
                        mutability: value_atoms.mutability,
                        presence: if matches!(value_pattern, NormValuePolicyPattern::Absent { .. })
                        {
                            ValuePresence::Absent
                        } else {
                            ValuePresence::Present
                        },
                    },
                    pattern: PatternComponentPolicy {
                        stages: pattern_atoms.stages,
                    },
                    namespace_visibility: None,
                },
                provenance,
            )
        }
        (NormValuePolicyPattern::Absent { .. }, None) => Err(policy_error(
            "an absent P2 value component requires an explicit type component",
            provenance,
        )),
    }
}

pub fn elaborate_p1_projection(
    policy: Option<&NormPolicySpec>,
    provenance: Provenance,
) -> Result<P1Projection, Diagnostic> {
    let Some(policy) = policy else {
        return Ok(P1Projection::Infer);
    };
    match (&policy.value_policy, &policy.type_policy) {
        (NormValuePolicyPattern::Expr(value), None) => {
            let atoms = parse_component(value, provenance.clone())?;
            let namespace_visibility = one_namespace(&atoms.namespace, provenance.clone())?;
            reject_mut_export(&atoms.mutability, namespace_visibility, provenance)?;
            Ok(P1Projection::ValueDominant {
                value: ValueComponentPolicy {
                    stages: atoms.stages,
                    mutability: atoms.mutability,
                    presence: ValuePresence::Present,
                },
                namespace_visibility,
            })
        }
        (value_pattern, Some(pattern)) => {
            let mut value_atoms = match value_pattern {
                NormValuePolicyPattern::Expr(value) => parse_component(value, provenance.clone())?,
                NormValuePolicyPattern::Absent { .. } => ComponentAtoms::default(),
            };
            let mut pattern_atoms = parse_component(pattern, provenance.clone())?;
            if !pattern_atoms.mutability.is_empty() {
                return Err(policy_error(
                    "const/mut policy is valid only in the P1 value component",
                    provenance,
                ));
            }
            let namespace_visibility = merge_namespace(
                &value_atoms.namespace,
                &pattern_atoms.namespace,
                provenance.clone(),
            )?;
            reject_mut_export(
                &value_atoms.mutability,
                namespace_visibility,
                provenance.clone(),
            )?;
            if value_atoms.stages.is_empty() && !pattern_atoms.stages.is_empty() {
                value_atoms.stages = pattern_atoms.stages.clone();
            } else if pattern_atoms.stages.is_empty() && !value_atoms.stages.is_empty() {
                pattern_atoms.stages = value_atoms.stages.clone();
            }
            if pattern_atoms.stages.contains(PolicyStage::Runtime) {
                return Err(policy_error(
                    "the P1 type component cannot contain runtime",
                    provenance,
                ));
            }
            Ok(P1Projection::Pair(PolicyPair {
                value: ValueComponentPolicy {
                    stages: value_atoms.stages,
                    mutability: value_atoms.mutability,
                    presence: if matches!(value_pattern, NormValuePolicyPattern::Absent { .. }) {
                        ValuePresence::Absent
                    } else {
                        ValuePresence::Present
                    },
                },
                pattern: PatternComponentPolicy {
                    stages: pattern_atoms.stages,
                },
                namespace_visibility,
            }))
        }
        (NormValuePolicyPattern::Absent { .. }, None) => Err(policy_error(
            "an absent P1 value pattern requires an explicit type component",
            provenance,
        )),
    }
}

pub fn derive_function_object_p1(
    result_p2: &PolicyPair,
    declaration: &FunctionObjectDeclarationPolicy,
) -> PolicyPair {
    PolicyPair {
        value: ValueComponentPolicy {
            stages: result_p2.value.stages.union(&result_p2.pattern.stages),
            mutability: declaration.mutability.clone(),
            presence: ValuePresence::Present,
        },
        pattern: PatternComponentPolicy {
            stages: result_p2.pattern.stages.clone(),
        },
        namespace_visibility: declaration.namespace_visibility,
    }
}

pub fn project_p1<'a, V, P>(
    projection: &P1Projection,
    result: &'a [PolicyResultEntry<V, P>],
) -> Vec<&'a PolicyResultEntry<V, P>> {
    result
        .iter()
        .filter(|entry| match projection {
            P1Projection::Infer => true,
            P1Projection::ValueDominant { value, .. } => value_matches(value, entry),
            P1Projection::Pair(pair) => {
                value_matches(&pair.value, entry)
                    && stages_match(&pair.pattern.stages, &entry.pattern_policy.stages)
            }
        })
        .collect()
}

fn value_matches<V, P>(query: &ValueComponentPolicy, entry: &PolicyResultEntry<V, P>) -> bool {
    match query.presence {
        ValuePresence::Absent => return entry.value.is_none(),
        ValuePresence::Present if entry.value.is_none() => return false,
        ValuePresence::Optional if entry.value.is_none() => return true,
        ValuePresence::Present | ValuePresence::Optional => {}
    }
    stages_match(&query.stages, &entry.value_policy.stages)
        && (query.mutability.is_empty()
            || query
                .mutability
                .iter()
                .any(|mutability| entry.value_policy.mutability.contains(mutability)))
}

fn stages_match(query: &StageSet, available: &StageSet) -> bool {
    query.is_empty() || query.intersects(available)
}

fn validate_p2_pair(pair: PolicyPair, provenance: Provenance) -> Result<PolicyPair, Diagnostic> {
    if pair.pattern.stages.contains(PolicyStage::Runtime) {
        return Err(policy_error(
            "P2 type component cannot contain runtime",
            provenance,
        ));
    }
    if pair.pattern.stages.len() != 1 {
        return Err(policy_error(
            "P2 type component must name exactly one static stage",
            provenance,
        ));
    }
    if pair.value.presence != ValuePresence::Absent && pair.value.stages.is_empty() {
        return Err(policy_error(
            "P2 value component requires a stage",
            provenance,
        ));
    }
    let value_static = pair.value.stages.static_stages();
    if !value_static.is_empty() && value_static != pair.pattern.stages {
        return Err(policy_error(
            "P2 value and type components use different static stages",
            provenance,
        ));
    }
    Ok(pair)
}

fn reject_namespace_in_p2(
    atoms: &ComponentAtoms,
    provenance: Provenance,
) -> Result<(), Diagnostic> {
    if atoms.namespace.is_empty() {
        Ok(())
    } else {
        Err(policy_error(
            "namespace visibility is valid only in P1 declaration position",
            provenance,
        ))
    }
}

fn reject_mut_export(
    mutability: &BTreeSet<ValueMutability>,
    namespace: Option<NamespaceVisibility>,
    provenance: Provenance,
) -> Result<(), Diagnostic> {
    if mutability.contains(&ValueMutability::Mut) && namespace == Some(NamespaceVisibility::Export)
    {
        Err(policy_error(
            "mut+export is invalid: a globally exported mutable value is not permitted",
            provenance,
        ))
    } else {
        Ok(())
    }
}

fn merge_namespace(
    value: &BTreeSet<NamespaceVisibility>,
    pattern: &BTreeSet<NamespaceVisibility>,
    provenance: Provenance,
) -> Result<Option<NamespaceVisibility>, Diagnostic> {
    let mut combined = value.clone();
    combined.extend(pattern.iter().copied());
    one_namespace(&combined, provenance)
}

fn one_namespace(
    namespace: &BTreeSet<NamespaceVisibility>,
    provenance: Provenance,
) -> Result<Option<NamespaceVisibility>, Diagnostic> {
    if namespace.len() > 1 {
        return Err(policy_error(
            "conflicting namespace visibility policies",
            provenance,
        ));
    }
    Ok(namespace.iter().next().copied())
}

fn parse_component(expr: &NormExpr, provenance: Provenance) -> Result<ComponentAtoms, Diagnostic> {
    let mut atoms = ComponentAtoms::default();
    collect_atoms(expr, &mut atoms, provenance)?;
    Ok(atoms)
}

fn collect_atoms(
    expr: &NormExpr,
    atoms: &mut ComponentAtoms,
    provenance: Provenance,
) -> Result<(), Diagnostic> {
    match expr {
        NormExpr::Name { text, .. } => {
            match text.as_str() {
                "meta" => atoms.stages.insert(PolicyStage::Meta),
                "compile" => atoms.stages.insert(PolicyStage::Compile),
                "seal" => atoms.stages.insert(PolicyStage::Seal),
                "runtime" => atoms.stages.insert(PolicyStage::Runtime),
                "const" => {
                    atoms.mutability.insert(ValueMutability::Const);
                }
                "mut" => {
                    atoms.mutability.insert(ValueMutability::Mut);
                }
                "public" => {
                    atoms.namespace.insert(NamespaceVisibility::Public);
                }
                "private" => {
                    atoms.namespace.insert(NamespaceVisibility::Private);
                }
                "export" => {
                    atoms.namespace.insert(NamespaceVisibility::Export);
                }
                other => {
                    return Err(policy_error(
                        format!("unknown policy atom `{other}`"),
                        provenance,
                    ));
                }
            }
            Ok(())
        }
        NormExpr::Call { source, target, .. } => {
            let NormExpr::OperatorTarget { spelling, .. } = target.as_ref() else {
                return Err(policy_error(
                    "policy expression requires `+` or `|`",
                    provenance,
                ));
            };
            if spelling != "+" && spelling != "|" {
                return Err(policy_error(
                    format!("policy expression cannot use operator `{spelling}`"),
                    provenance,
                ));
            }
            for element in &source.elements {
                let NormProductElem::Expr(expr) = element else {
                    return Err(policy_error(
                        "policy operators require expression operands",
                        provenance,
                    ));
                };
                collect_atoms(expr, atoms, provenance.clone())?;
            }
            Ok(())
        }
        _ => Err(policy_error(
            "invalid policy component expression",
            provenance,
        )),
    }
}

fn policy_error(message: impl Into<String>, provenance: Provenance) -> Diagnostic {
    Diagnostic::hard_error(message, Some(provenance))
}
