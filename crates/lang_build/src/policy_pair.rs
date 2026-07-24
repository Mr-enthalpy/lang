use std::collections::{BTreeMap, BTreeSet, VecDeque};

use lang_syntax::{
    NormPolicyAtom, NormPolicyChoice, NormPolicyConjunction, NormPolicySpec, NormValuePolicyPattern,
};

use crate::{Diagnostic, Provenance};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PolicyStage {
    Meta,
    Compile,
    Seal,
    Runtime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Phase {
    OpenStatic,
    SealStatic,
    Runtime,
}

impl PolicyStage {
    pub fn is_static(self) -> bool {
        !matches!(self, Self::Runtime)
    }

    pub fn visible_at(self, phase: Phase) -> bool {
        match self {
            Self::Meta => phase == Phase::OpenStatic,
            Self::Compile => matches!(phase, Phase::OpenStatic | Phase::SealStatic),
            Self::Seal => phase == Phase::SealStatic,
            Self::Runtime => phase == Phase::Runtime,
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

    pub fn intersection(&self, other: &Self) -> Self {
        Self(self.0.intersection(&other.0).copied().collect())
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.0.iter().any(|stage| other.contains(*stage))
    }

    pub fn visible_at(&self, phase: Phase) -> bool {
        self.0.iter().any(|stage| stage.visible_at(phase))
    }

    pub fn exposed_at(&self, phase: Phase) -> Self {
        Self(
            self.0
                .iter()
                .copied()
                .filter(|stage| stage.visible_at(phase))
                .collect(),
        )
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
    /// An empty domain is the unconstrained `const || mut` domain. A
    /// singleton is an explicit restriction to that mutability view.
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
    pub export_root: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum P1Projection {
    Infer,
    ValueDominant { value: ValueComponentPolicy },
    Pair(PolicyPair),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormalPolicyPattern {
    /// The parameter policy after inheriting its callable P2 and applying the
    /// optional const/mut-only formal slice.
    pub effective_pair: PolicyPair,
    /// The written overload-preference qualifier. `None` means unspecified;
    /// it does not erase or rebuild any inherited P2 dimension. Candidate
    /// formation must copy this field into the external policy product order.
    pub mutability: Option<ValueMutability>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NamespaceDeclarationPosition {
    DirectTopLevel,
    Local,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamespaceDeclarationPolicy {
    /// The complete namespace-internal declaration view. Export never crops
    /// this projection.
    pub projection: P1Projection,
    /// Root-local external projection derived when this declaration directly
    /// writes `export`. This is an early validation/preview only:
    /// `None` does not prove that the declaration is absent from the eventual
    /// export view, because `ExportClosure(root)` may admit ancestors and
    /// descendants. Namespace graph integration must combine closure
    /// membership with public path reachability before projecting candidates
    /// through `project_export_overload_sets`.
    pub external_projection: Option<P1Projection>,
    pub visibility: Option<NamespaceVisibility>,
    pub export_root: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionObjectDeclarationPolicy {
    /// Empty is the default unrestricted `const || mut` function-object
    /// domain. This is the complete namespace-internal declaration view;
    /// export const-projection is represented separately and never crops it.
    pub mutability: BTreeSet<ValueMutability>,
    pub namespace_visibility: Option<NamespaceVisibility>,
    pub export_root: bool,
}

impl Default for FunctionObjectDeclarationPolicy {
    fn default() -> Self {
        Self {
            mutability: BTreeSet::new(),
            namespace_visibility: None,
            export_root: false,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuiltinPrivilegedSealFunction {
    ExportWorldMaterializer,
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

    pub fn scan_domain_for_builtin(&self, _builtin: BuiltinPrivilegedSealFunction) -> &[T] {
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

    pub fn resolve_explicit(&self, mut predicate: impl FnMut(&T) -> bool) -> Option<&T> {
        self.final_world().find(|value| predicate(value))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WpreRoots<T> {
    pub exported_symbols: Vec<T>,
    pub materialized_results_of_exported_meta_functions: Vec<T>,
    pub parameter_dependencies_of_exported_meta_functions: Vec<T>,
}

pub fn compute_wpre<T: Clone + Ord>(
    roots: WpreRoots<T>,
    mut semantic_dependencies: impl FnMut(&T) -> Vec<T>,
) -> BTreeSet<T> {
    let mut closure = BTreeSet::new();
    let mut queue = VecDeque::new();
    queue.extend(roots.exported_symbols);
    queue.extend(roots.materialized_results_of_exported_meta_functions);
    queue.extend(roots.parameter_dependencies_of_exported_meta_functions);

    while let Some(symbol) = queue.pop_front() {
        if !closure.insert(symbol.clone()) {
            continue;
        }
        queue.extend(semantic_dependencies(&symbol));
    }
    closure
}

/// Namespace facts used to derive the export graph and ordinary path
/// visibility. Export closure and public reachability intentionally remain
/// independent computations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamespaceExportNode<I> {
    pub parent: Option<I>,
    pub visibility: NamespaceVisibility,
}

/// One externally exposed view of an existing internal candidate.
///
/// `identity` and `internal_candidate` preserve the candidate's symbol-world
/// identity. `external_policy` is the const-projected namespace interface
/// policy; external resolution must consume this field rather than the
/// candidate's complete internal policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportCandidateView<I, C> {
    pub identity: I,
    pub internal_candidate: C,
    pub external_policy: PolicyPair,
}

/// Resolved internal candidate view after its declaration-side `P1Projection`
/// has already been applied to the actual RHS/result entries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedCandidatePolicy {
    pub pair: PolicyPair,
}

/// Namespace-level facts required before a symbol may contribute candidate
/// views to `Sigma_export`.
///
/// Export-closure membership alone is not sufficient: every component of the
/// externally navigated path must also pass public/private reachability.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExportAdmission {
    pub in_export_closure: bool,
    pub publicly_reachable: bool,
}

impl ExportAdmission {
    pub fn is_externally_exposed(self) -> bool {
        self.in_export_closure && self.publicly_reachable
    }
}

/// The complete namespace overload set and its externally exposed candidate
/// views. Export views retain internal candidate identity but carry a distinct
/// policy projection.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NamespaceOverloadSets<N, I, C> {
    pub full: BTreeMap<N, Vec<C>>,
    pub exported: BTreeMap<N, Vec<ExportCandidateView<I, C>>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NamespaceResolveAuthority {
    Internal,
    External,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NamespaceCandidateSetRef<'a, I, C> {
    Internal(&'a [C]),
    External(&'a [ExportCandidateView<I, C>]),
}

impl<N: Ord, I, C> NamespaceOverloadSets<N, I, C> {
    pub fn resolve(
        &self,
        name: &N,
        authority: NamespaceResolveAuthority,
    ) -> Option<NamespaceCandidateSetRef<'_, I, C>> {
        match authority {
            NamespaceResolveAuthority::Internal => self
                .full
                .get(name)
                .map(|candidates| NamespaceCandidateSetRef::Internal(candidates)),
            NamespaceResolveAuthority::External => self
                .exported
                .get(name)
                .map(|candidates| NamespaceCandidateSetRef::External(candidates)),
        }
    }

    pub fn resolve_internal(&self, name: &N) -> Option<&[C]> {
        self.full.get(name).map(Vec::as_slice)
    }

    pub fn resolve_external(&self, name: &N) -> Option<&[ExportCandidateView<I, C>]> {
        self.exported.get(name).map(Vec::as_slice)
    }
}

/// Project external overload views from the complete namespace sets.
///
/// `ExportAdmission` combines export-closure membership with public path
/// reachability. Only externally exposed symbols are considered. Candidate
/// policy eligibility is then a separate operation over each resolved internal
/// `PolicyPair`: candidates without a const value slice remain in `full` but
/// are omitted from `exported`. Direct source declarations that explicitly
/// write `export + mut` are rejected earlier by `project_export_root_preview`.
pub fn project_export_overload_sets<N: Clone + Ord, I, C: Clone>(
    full: BTreeMap<N, Vec<C>>,
    mut external_admission: impl FnMut(&N) -> ExportAdmission,
    mut resolve_candidate: impl FnMut(&C) -> (I, ResolvedCandidatePolicy),
) -> NamespaceOverloadSets<N, I, C> {
    let mut exported = BTreeMap::new();
    for (name, candidates) in &full {
        if !external_admission(name).is_externally_exposed() {
            continue;
        }
        let mut projected = Vec::new();
        for candidate in candidates {
            let (identity, internal_policy) = resolve_candidate(candidate);
            let Some(external_policy) = project_resolved_export_view(&internal_policy) else {
                continue;
            };
            projected.push(ExportCandidateView {
                identity,
                internal_candidate: candidate.clone(),
                external_policy,
            });
        }
        if !projected.is_empty() {
            exported.insert(name.clone(), projected);
        }
    }
    NamespaceOverloadSets { full, exported }
}

/// Derive the externally readable pair from an already resolved internal
/// candidate view.
///
/// This function never accepts `P1Projection`: declaration projection has
/// already happened. The Pattern component is preserved exactly. A
/// value-bearing candidate without a const slice is not externally eligible.
pub fn project_resolved_export_view(
    internal_policy: &ResolvedCandidatePolicy,
) -> Option<PolicyPair> {
    let mut projected = internal_policy.pair.clone();
    if projected.value.presence == ValuePresence::Absent {
        return Some(projected);
    }
    if !projected.value.mutability.is_empty()
        && !projected.value.mutability.contains(&ValueMutability::Const)
    {
        return None;
    }
    projected.value.mutability = BTreeSet::from([ValueMutability::Const]);
    Some(projected)
}

/// Compute `PathAncestors(root) ∪ Subtree(root)` for every export root.
/// Descendants cannot opt out, while siblings are included only when they are
/// themselves an ancestor/descendant of another root.
pub fn compute_export_closure<I: Clone + Ord>(
    nodes: &BTreeMap<I, NamespaceExportNode<I>>,
    export_roots: impl IntoIterator<Item = I>,
) -> BTreeSet<I> {
    let mut exported = BTreeSet::new();
    let mut children = BTreeMap::<I, Vec<I>>::new();
    for (id, node) in nodes {
        if let Some(parent) = &node.parent {
            children.entry(parent.clone()).or_default().push(id.clone());
        }
    }

    for root in export_roots {
        let mut current = Some(root.clone());
        let mut visited_ancestors = BTreeSet::new();
        while let Some(id) = current {
            if !visited_ancestors.insert(id.clone()) {
                break;
            }
            exported.insert(id.clone());
            current = nodes.get(&id).and_then(|node| node.parent.clone());
        }

        let mut queue = VecDeque::from([root]);
        while let Some(id) = queue.pop_front() {
            if exported.insert(id.clone()) || children.contains_key(&id) {
                if let Some(direct_children) = children.get(&id) {
                    queue.extend(direct_children.iter().cloned());
                }
            }
        }
    }
    exported
}

pub fn publicly_reachable<I: Ord>(
    nodes: &BTreeMap<I, NamespaceExportNode<I>>,
    path: impl IntoIterator<Item = I>,
) -> bool {
    path.into_iter().all(|id| {
        nodes
            .get(&id)
            .is_some_and(|node| node.visibility == NamespaceVisibility::Public)
    })
}

pub fn externally_visible<I: Ord>(
    symbol: &I,
    export_closure: &BTreeSet<I>,
    nodes: &BTreeMap<I, NamespaceExportNode<I>>,
    path: impl IntoIterator<Item = I>,
) -> bool {
    export_closure.contains(symbol) && publicly_reachable(nodes, path)
}

#[derive(Clone, Debug, Default)]
struct ComponentAtoms {
    stages: StageSet,
    mutability: BTreeSet<ValueMutability>,
    namespace: BTreeSet<NamespaceVisibility>,
    export_root: bool,
    absent_value: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum PolicyDimension {
    Stage,
    Mutability,
    NamespaceVisibility,
    ExportRoot,
    ValuePresence,
}

impl ComponentAtoms {
    fn dimensions(&self) -> BTreeSet<PolicyDimension> {
        let mut result = BTreeSet::new();
        if !self.stages.is_empty() {
            result.insert(PolicyDimension::Stage);
        }
        if !self.mutability.is_empty() {
            result.insert(PolicyDimension::Mutability);
        }
        if !self.namespace.is_empty() {
            result.insert(PolicyDimension::NamespaceVisibility);
        }
        if self.export_root {
            result.insert(PolicyDimension::ExportRoot);
        }
        if self.absent_value {
            result.insert(PolicyDimension::ValuePresence);
        }
        result
    }

    fn presence(&self) -> ValuePresence {
        match (self.absent_value, self.stages.is_empty()) {
            (true, true) => ValuePresence::Absent,
            (true, false) => ValuePresence::Optional,
            (false, _) => ValuePresence::Present,
        }
    }
}

pub fn normalize_p2_policy(
    policy: &NormPolicySpec,
    provenance: Provenance,
) -> Result<PolicyPair, Diagnostic> {
    match (&policy.value_policy, &policy.pattern_policy) {
        (NormValuePolicyPattern::Conjunction(value), None) => {
            let atoms = parse_component(value, true, provenance.clone())?;
            reject_namespace_attributes(&atoms, "P2", provenance.clone())?;
            let presence = atoms.presence();
            let static_stages = atoms.stages.static_stages();
            let pattern_stages = if static_stages.is_empty() {
                if !atoms.stages.contains(PolicyStage::Runtime) {
                    return Err(policy_error(
                        "P2 single-policy form requires a stage",
                        provenance,
                    ));
                }
                StageSet::from([PolicyStage::Compile])
            } else {
                static_stages
            };
            validate_p2_pair(
                PolicyPair {
                    value: ValueComponentPolicy {
                        stages: atoms.stages,
                        mutability: atoms.mutability,
                        presence,
                    },
                    pattern: PatternComponentPolicy {
                        stages: pattern_stages,
                    },
                    namespace_visibility: None,
                    export_root: false,
                },
                provenance,
            )
        }
        (value_pattern, Some(pattern)) => {
            let value_atoms = match value_pattern {
                NormValuePolicyPattern::Conjunction(value) => {
                    parse_component(value, true, provenance.clone())?
                }
                NormValuePolicyPattern::Absent { .. } => ComponentAtoms {
                    absent_value: true,
                    ..ComponentAtoms::default()
                },
            };
            let pattern_atoms = parse_component(pattern, false, provenance.clone())?;
            reject_namespace_attributes(&value_atoms, "P2", provenance.clone())?;
            reject_namespace_attributes(&pattern_atoms, "P2", provenance.clone())?;
            let value_presence = value_atoms.presence();
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
                        presence: value_presence,
                    },
                    pattern: PatternComponentPolicy {
                        stages: pattern_atoms.stages,
                    },
                    namespace_visibility: None,
                    export_root: false,
                },
                provenance,
            )
        }
        (NormValuePolicyPattern::Absent { .. }, None) => Err(policy_error(
            "an absent P2 value component requires an explicit Pattern component",
            provenance,
        )),
    }
}

pub fn elaborate_binding_p1_projection(
    policy: Option<&NormPolicySpec>,
    provenance: Provenance,
) -> Result<P1Projection, Diagnostic> {
    let Some(policy) = policy else {
        return Ok(P1Projection::Infer);
    };
    let (projection, namespace, export_root) = elaborate_p1_components(policy, provenance.clone())?;
    if !namespace.is_empty() || export_root {
        return Err(policy_error(
            "public/private/export are valid only on namespace declarations",
            provenance,
        ));
    }
    Ok(projection)
}

pub fn elaborate_formal_policy_pattern(
    policy: Option<&NormPolicySpec>,
    inherited_p2: &PolicyPair,
    provenance: Provenance,
) -> Result<FormalPolicyPattern, Diagnostic> {
    let Some(policy) = policy else {
        return Ok(FormalPolicyPattern {
            effective_pair: inherited_p2.clone(),
            mutability: None,
        });
    };
    if policy.pattern_policy.is_some() {
        return Err(policy_error(
            "formal parameter policy uses a value policy pattern, not a P1 pair projection",
            provenance,
        ));
    }
    let atoms = match &policy.value_policy {
        NormValuePolicyPattern::Conjunction(value) => {
            parse_component(value, false, provenance.clone())?
        }
        NormValuePolicyPattern::Absent { .. } => {
            return Err(policy_error(
                "formal parameter policy cannot use an absent value pattern",
                provenance,
            ));
        }
    };
    reject_namespace_attributes(&atoms, "formal parameter", provenance.clone())?;
    if !atoms.stages.is_empty() || atoms.absent_value {
        return Err(policy_error(
            "formal parameter policy may restrict only the const/mut axis inherited from P2",
            provenance,
        ));
    }
    if atoms.mutability.len() != 1 {
        return Err(policy_error(
            "an explicit formal parameter policy must select exactly one of const or mut",
            provenance,
        ));
    }
    let selected = atoms
        .mutability
        .iter()
        .next()
        .copied()
        .expect("one formal mutability after validation");
    if !inherited_p2.value.mutability.is_empty()
        && !inherited_p2.value.mutability.contains(&selected)
    {
        return Err(policy_error(
            "formal parameter const/mut slice is outside the mutability domain inherited from P2",
            provenance,
        ));
    }
    let mut effective_pair = inherited_p2.clone();
    effective_pair.value.mutability = BTreeSet::from([selected]);
    Ok(FormalPolicyPattern {
        effective_pair,
        mutability: Some(selected),
    })
}

pub fn elaborate_namespace_declaration_policy(
    policy: Option<&NormPolicySpec>,
    position: NamespaceDeclarationPosition,
    provenance: Provenance,
) -> Result<NamespaceDeclarationPolicy, Diagnostic> {
    let Some(policy) = policy else {
        return Ok(NamespaceDeclarationPolicy {
            projection: P1Projection::Infer,
            external_projection: None,
            visibility: None,
            export_root: false,
        });
    };
    let (projection, namespace, export_root) = elaborate_p1_components(policy, provenance.clone())?;
    let visibility = one_namespace(&namespace, provenance.clone())?;
    if export_root && position != NamespaceDeclarationPosition::DirectTopLevel {
        return Err(policy_error(
            "export is allowed only on a direct top-level declaration of a namespace construction level",
            provenance,
        ));
    }
    let external_projection = export_root
        .then(|| project_export_root_preview(&projection, provenance.clone()))
        .transpose()?;
    Ok(NamespaceDeclarationPolicy {
        projection,
        external_projection,
        visibility,
        export_root,
    })
}

/// Validate and preview a direct export-root declaration without modifying its
/// complete internal P1 request.
///
/// This is not the final candidate view: `P1Projection::ValueDominant` still
/// lacks the associated resolved Pattern component. Final external views are
/// produced only from `ResolvedCandidatePolicy` by
/// `project_resolved_export_view`.
pub fn project_export_root_preview(
    projection: &P1Projection,
    provenance: Provenance,
) -> Result<P1Projection, Diagnostic> {
    let mut projected = projection.clone();
    let value = match &mut projected {
        P1Projection::ValueDominant { value } => value,
        P1Projection::Pair(pair) => &mut pair.value,
        P1Projection::Infer => {
            return Err(policy_error(
                "an export root requires an explicit namespace declaration policy",
                provenance,
            ));
        }
    };

    if value.presence == ValuePresence::Absent {
        return Ok(projected);
    }
    if !value.mutability.is_empty() && !value.mutability.contains(&ValueMutability::Const) {
        return Err(policy_error(
            "a value-bearing export requires a non-empty const projection",
            provenance,
        ));
    }
    value.mutability = BTreeSet::from([ValueMutability::Const]);
    Ok(projected)
}

fn elaborate_p1_components(
    policy: &NormPolicySpec,
    provenance: Provenance,
) -> Result<(P1Projection, BTreeSet<NamespaceVisibility>, bool), Diagnostic> {
    match (&policy.value_policy, &policy.pattern_policy) {
        (NormValuePolicyPattern::Conjunction(value), None) => {
            let atoms = parse_component(value, true, provenance)?;
            let projection = P1Projection::ValueDominant {
                value: ValueComponentPolicy {
                    stages: atoms.stages.clone(),
                    mutability: atoms.mutability.clone(),
                    presence: atoms.presence(),
                },
            };
            Ok((projection, atoms.namespace, atoms.export_root))
        }
        (value_pattern, Some(pattern)) => {
            let value_atoms = match value_pattern {
                NormValuePolicyPattern::Conjunction(value) => {
                    parse_component(value, true, provenance.clone())?
                }
                NormValuePolicyPattern::Absent { .. } => ComponentAtoms {
                    absent_value: true,
                    ..ComponentAtoms::default()
                },
            };
            let pattern_atoms = parse_component(pattern, false, provenance.clone())?;
            if !pattern_atoms.mutability.is_empty() {
                return Err(policy_error(
                    "const/mut policy is valid only in the P1 value component",
                    provenance,
                ));
            }
            if pattern_atoms.stages.contains(PolicyStage::Runtime) {
                return Err(policy_error(
                    "the P1 Pattern component cannot contain runtime",
                    provenance,
                ));
            }
            let mut namespace = value_atoms.namespace.clone();
            namespace.extend(pattern_atoms.namespace.iter().copied());
            let export_root = value_atoms.export_root || pattern_atoms.export_root;
            let visibility = one_namespace(&namespace, provenance.clone())?;
            let value_presence = value_atoms.presence();
            Ok((
                P1Projection::Pair(PolicyPair {
                    value: ValueComponentPolicy {
                        stages: value_atoms.stages,
                        mutability: value_atoms.mutability,
                        presence: value_presence,
                    },
                    pattern: PatternComponentPolicy {
                        stages: pattern_atoms.stages,
                    },
                    namespace_visibility: visibility,
                    export_root,
                }),
                namespace,
                export_root,
            ))
        }
        (NormValuePolicyPattern::Absent { .. }, None) => Err(policy_error(
            "an absent P1 value pattern requires an explicit Pattern component",
            provenance,
        )),
    }
}

pub fn function_object_declaration_policy(
    declaration: &NamespaceDeclarationPolicy,
) -> FunctionObjectDeclarationPolicy {
    let mutability = match &declaration.projection {
        P1Projection::Infer => BTreeSet::new(),
        P1Projection::ValueDominant { value } => value.mutability.clone(),
        P1Projection::Pair(pair) => pair.value.mutability.clone(),
    };
    FunctionObjectDeclarationPolicy {
        mutability,
        namespace_visibility: declaration.visibility,
        export_root: declaration.export_root,
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
        export_root: declaration.export_root,
    }
}

/// Apply a P1 projection as a real slice restriction. The returned entries are
/// owned views whose stage/mutability sets are cropped; associated value and
/// Pattern identities are cloned unchanged.
pub fn project_p1<V: Clone, P: Clone>(
    projection: &P1Projection,
    result: &[PolicyResultEntry<V, P>],
) -> Vec<PolicyResultEntry<V, P>> {
    result
        .iter()
        .filter_map(|entry| match projection {
            P1Projection::Infer => Some((*entry).clone()),
            P1Projection::ValueDominant { value } => {
                let value_policy = restrict_value_policy(value, entry)?;
                Some(PolicyResultEntry {
                    value: entry.value.clone(),
                    value_policy,
                    pattern: entry.pattern.clone(),
                    pattern_policy: entry.pattern_policy.clone(),
                })
            }
            P1Projection::Pair(pair) => {
                let value_policy = restrict_value_policy(&pair.value, entry)?;
                let pattern_stages =
                    restrict_stages(&pair.pattern.stages, &entry.pattern_policy.stages)?;
                Some(PolicyResultEntry {
                    value: entry.value.clone(),
                    value_policy,
                    pattern: entry.pattern.clone(),
                    pattern_policy: PatternComponentPolicy {
                        stages: pattern_stages,
                    },
                })
            }
        })
        .collect()
}

fn restrict_value_policy<V, P>(
    query: &ValueComponentPolicy,
    entry: &PolicyResultEntry<V, P>,
) -> Option<ValueComponentPolicy> {
    match query.presence {
        ValuePresence::Absent if entry.value.is_some() => return None,
        ValuePresence::Present if entry.value.is_none() => return None,
        ValuePresence::Optional | ValuePresence::Present | ValuePresence::Absent => {}
    }
    if entry.value.is_none() {
        return Some(entry.value_policy.clone());
    }
    let stages = restrict_stages(&query.stages, &entry.value_policy.stages)?;
    let mutability = if query.mutability.is_empty() {
        entry.value_policy.mutability.clone()
    } else if entry.value_policy.mutability.is_empty() {
        // Empty is the unconstrained `const || mut` domain, so an explicit
        // query crops it to the requested singleton/domain.
        query.mutability.clone()
    } else {
        let selected = query
            .mutability
            .intersection(&entry.value_policy.mutability)
            .copied()
            .collect::<BTreeSet<_>>();
        if selected.is_empty() {
            return None;
        }
        selected
    };
    Some(ValueComponentPolicy {
        stages,
        mutability,
        presence: entry.value_policy.presence,
    })
}

fn restrict_stages(query: &StageSet, available: &StageSet) -> Option<StageSet> {
    if query.is_empty() {
        return Some(available.clone());
    }
    let selected = query.intersection(available);
    (!selected.is_empty()).then_some(selected)
}

fn validate_p2_pair(pair: PolicyPair, provenance: Provenance) -> Result<PolicyPair, Diagnostic> {
    if pair.pattern.stages.contains(PolicyStage::Runtime) {
        return Err(policy_error(
            "P2 Pattern component cannot contain runtime",
            provenance,
        ));
    }
    if pair.pattern.stages.is_empty() {
        return Err(policy_error(
            "P2 Pattern component requires at least one static stage",
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
            "P2 value and Pattern components use different static stages",
            provenance,
        ));
    }
    Ok(pair)
}

fn reject_namespace_attributes(
    atoms: &ComponentAtoms,
    context: &str,
    provenance: Provenance,
) -> Result<(), Diagnostic> {
    if atoms.namespace.is_empty() && !atoms.export_root {
        Ok(())
    } else {
        Err(policy_error(
            format!("public/private/export are not valid in {context} policy"),
            provenance,
        ))
    }
}

fn one_namespace(
    namespace: &BTreeSet<NamespaceVisibility>,
    provenance: Provenance,
) -> Result<Option<NamespaceVisibility>, Diagnostic> {
    if namespace.len() > 1 {
        return Err(policy_error(
            "a namespace declaration must choose exactly one of public or private",
            provenance,
        ));
    }
    Ok(namespace.iter().next().copied())
}

fn parse_component(
    conjunction: &NormPolicyConjunction,
    allow_absent: bool,
    provenance: Provenance,
) -> Result<ComponentAtoms, Diagnostic> {
    let mut result = ComponentAtoms::default();
    for choice in &conjunction.choices {
        let next = parse_choice(choice, allow_absent, provenance.clone())?;
        merge_conjunction(&mut result, next, provenance.clone())?;
    }
    Ok(result)
}

fn parse_choice(
    choice: &NormPolicyChoice,
    allow_absent: bool,
    provenance: Provenance,
) -> Result<ComponentAtoms, Diagnostic> {
    let mut alternatives = Vec::new();
    for atom in &choice.atoms {
        alternatives.push(parse_atom(atom, allow_absent, provenance.clone())?);
    }
    if alternatives.len() == 1 {
        return Ok(alternatives.pop().expect("one alternative"));
    }

    let dimensions = alternatives
        .iter()
        .map(ComponentAtoms::dimensions)
        .collect::<Vec<_>>();
    let same_single_dimension = dimensions
        .first()
        .and_then(|first| (first.len() == 1).then(|| first.iter().next().copied().unwrap()))
        .filter(|dimension| {
            dimensions
                .iter()
                .all(|current| current.len() == 1 && current.contains(dimension))
        });

    if let Some(dimension) = same_single_dimension {
        let mut result = ComponentAtoms::default();
        for alternative in alternatives {
            merge_same_dimension(&mut result, alternative, dimension);
        }
        return Ok(result);
    }

    let stage_or_absent = dimensions.iter().all(|current| {
        !current.is_empty()
            && current.iter().all(|dimension| {
                matches!(
                    dimension,
                    PolicyDimension::Stage | PolicyDimension::ValuePresence
                )
            })
    });
    if stage_or_absent {
        let mut result = ComponentAtoms::default();
        for alternative in alternatives {
            result.stages = result.stages.union(&alternative.stages);
            result.absent_value |= alternative.absent_value;
        }
        return Ok(result);
    }

    Err(policy_error(
        "policy `||` may choose alternatives only within one dimension; clause-level disjunction is not supported",
        provenance,
    ))
}

fn parse_atom(
    atom: &NormPolicyAtom,
    allow_absent: bool,
    provenance: Provenance,
) -> Result<ComponentAtoms, Diagnostic> {
    let mut atoms = ComponentAtoms::default();
    match atom {
        NormPolicyAtom::Name { text, .. } => match text.as_str() {
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
            "export" => atoms.export_root = true,
            other => {
                return Err(policy_error(
                    format!("unknown policy atom `{other}`"),
                    provenance,
                ));
            }
        },
        NormPolicyAtom::HoleRef { text, .. } => {
            return Err(policy_error(
                format!("DeduceList hole `{text}` is not yet a concrete typed policy atom"),
                provenance,
            ));
        }
        NormPolicyAtom::Group { conjunction, .. } => {
            return parse_component(conjunction, allow_absent, provenance);
        }
        NormPolicyAtom::AbsentValuePattern { .. } => {
            if !allow_absent {
                return Err(policy_error(
                    "absent-value pattern is valid only in the value component",
                    provenance,
                ));
            }
            atoms.absent_value = true;
        }
        NormPolicyAtom::Error(_) => {
            return Err(policy_error("invalid policy AST", provenance));
        }
    }
    Ok(atoms)
}

fn merge_conjunction(
    result: &mut ComponentAtoms,
    next: ComponentAtoms,
    provenance: Provenance,
) -> Result<(), Diagnostic> {
    let overlap = result
        .dimensions()
        .intersection(&next.dimensions())
        .copied()
        .collect::<Vec<_>>();
    if !overlap.is_empty() {
        return Err(policy_error(
            format!("policy `+` cannot conjoin two values of the same dimension ({overlap:?})"),
            provenance,
        ));
    }
    if (result.absent_value && !next.stages.is_empty())
        || (next.absent_value && !result.stages.is_empty())
    {
        return Err(policy_error(
            "value absence is an alternative (`||`), not a stage conjunction (`+`)",
            provenance,
        ));
    }
    result.stages = result.stages.union(&next.stages);
    result.mutability.extend(next.mutability);
    result.namespace.extend(next.namespace);
    result.export_root |= next.export_root;
    result.absent_value |= next.absent_value;
    Ok(())
}

fn merge_same_dimension(
    result: &mut ComponentAtoms,
    next: ComponentAtoms,
    dimension: PolicyDimension,
) {
    match dimension {
        PolicyDimension::Stage => result.stages = result.stages.union(&next.stages),
        PolicyDimension::Mutability => result.mutability.extend(next.mutability),
        PolicyDimension::NamespaceVisibility => result.namespace.extend(next.namespace),
        PolicyDimension::ExportRoot => result.export_root |= next.export_root,
        PolicyDimension::ValuePresence => result.absent_value |= next.absent_value,
    }
}

fn policy_error(message: impl Into<String>, provenance: Provenance) -> Diagnostic {
    Diagnostic::hard_error(message, Some(provenance))
}
