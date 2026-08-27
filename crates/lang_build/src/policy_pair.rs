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

    pub fn is_subset(&self, other: &Self) -> bool {
        self.0.is_subset(&other.0)
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

/// Concrete overload-visible Policy point. `Plain` is neither omission nor an
/// unconstrained set; every evaluated object/call context carries one point.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PolicyMode {
    Const,
    #[default]
    Plain,
    Mut,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OutputModeDemand(pub PolicyMode);

impl OutputModeDemand {
    pub const fn mode(self) -> PolicyMode {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapabilityRealizationCell {
    Absent,
    Default,
    Delete,
    Custom,
}

/// Candidate-local, Policy-orthogonal input-mode x output-mode realization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityRealization {
    cells: BTreeMap<(PolicyMode, PolicyMode), CapabilityRealizationCell>,
}

impl Default for CapabilityRealization {
    fn default() -> Self {
        let mut cells = BTreeMap::new();
        for input in [PolicyMode::Const, PolicyMode::Plain, PolicyMode::Mut] {
            for output in [PolicyMode::Const, PolicyMode::Plain, PolicyMode::Mut] {
                cells.insert((input, output), CapabilityRealizationCell::Absent);
            }
        }
        Self { cells }
    }
}

impl CapabilityRealization {
    pub fn set(&mut self, input: PolicyMode, output: PolicyMode, cell: CapabilityRealizationCell) {
        self.cells.insert((input, output), cell);
    }

    pub fn cell(&self, input: PolicyMode, output: PolicyMode) -> CapabilityRealizationCell {
        self.cells[&(input, output)]
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = ((PolicyMode, PolicyMode), CapabilityRealizationCell)> + '_ {
        self.cells
            .iter()
            .map(|(coordinate, cell)| (*coordinate, *cell))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NamespaceVisibility {
    Public,
    Private,
}

/// The aggregate shape of a callable's invocation result.
///
/// `CallableSemantics = P1 × P2 × ReturnShape × Privilege`.
/// The return shape is elaborated once from the return-slot annotation
/// (`declared_return_shape_from_closure`); it is never derived from the
/// Policy stage, and no stage is ever derived from it.  The only relation
/// between P2 and the shape is the legality check
/// [`validate_return_shape`], which is a validation, not a derivation in
/// either direction.
///
/// `ClusterSymbol` — plural values under ONE name at ONE position (a
/// Symbol cluster: at most one pure-P member plus arbitrary val
/// members).  Spelled `-> r: symbol` (a single binder carries the
/// cluster).
///
/// There is deliberately NO parallel "multiple bare positions" return
/// ontology: a future product-shaped result is one ordinary value whose
/// Val1 is a Product (`⟨Val1_Product, P_Product, Val2⟩`) — still a single
/// `SingleVal` result, never a fifth return shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReturnShape {
    /// Value-less pure shape.  Required spelling: `_: unit` — the `_`
    /// occupies the leftmost slot so `unit` cannot be misread as the
    /// leftmost to-be-extracted name of an extraction shorthand
    /// (compare `_ unit`, which matches and discards the leaf node).
    Unit,
    /// A single pure-P type result (`-> r: type`).
    SingleType,
    /// A single ordinary value result.  Carries the presence fact of the
    /// return-slot value-pattern constraint; the full annotation pattern
    /// stays on the closure head return slot.
    SingleVal(PatternConstraint),
    /// Plural values under one name at one position — a Symbol cluster
    /// construction (`-> r: symbol`).
    ClusterSymbol,
}

/// Presence fact of the return-slot value-pattern constraint carried by
/// [`ReturnShape::SingleVal`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatternConstraint {
    Unconstrained,
    Constrained,
}

/// Independent capability axis of `CallableSemantics`.
///
/// Privilege states what special operations a callable may perform (for
/// example consuming raw/meta AST material).  It implies nothing about
/// the return shape and nothing about the Policy stage: `struct` is a
/// privileged built-in whose shape is `ClusterSymbol`, while `assert` /
/// `verify` / `identity_type` are privileged built-ins with ordinary
/// single-position shapes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallablePrivilege {
    /// Ordinary source-declared callable; the source surface can never
    /// spell a privilege.
    OrdinarySource,
    /// Compiler-provided built-in with privileged capabilities.
    BuiltinPrivileged,
}

/// `Validate(P2, ReturnShape)` — the legality relation between the result
/// Policy coordinate and the declared return shape.  A validation, never
/// a derivation: neither coordinate is ever computed from the other.
///
/// The core criterion for meta-legal returns is a SINGLE position:
///
/// * `ClusterSymbol` (one position, plural values) requires `P2 = meta`.
/// * `SingleType` / `SingleVal` / `Unit` are legal under both; root
///   constraints (self-rooting of meta type results) are enforced by the
///   invocation/installation layer, not here.
pub fn validate_return_shape(
    shape: ReturnShape,
    p2: &PolicyPair,
    provenance: &crate::Provenance,
) -> Result<(), crate::Diagnostic> {
    let stages = p2.value.stages.union(&p2.pattern.stages);
    let includes_meta = stages.contains(PolicyStage::Meta);
    let cluster_construction_authorized = stages.len() == 1 && includes_meta;
    match shape {
        ReturnShape::ClusterSymbol if !cluster_construction_authorized => {
            Err(crate::Diagnostic::hard_error(
                "a ClusterSymbol return (`-> r: symbol`) requires a pure meta result P2: \
                 a Symbol cluster cannot be constructed by a mixed meta/compile or \
                 meta/runtime result domain",
                Some(provenance.clone()),
            ))
        }
        _ => Ok(()),
    }
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
}

/// One complete observation edge. The pair and the whole-slot mode are
/// orthogonal semantic facts: neither may be reconstructed from the other.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyView {
    pub pair: PolicyPair,
    pub mode: PolicyMode,
}

/// Policy disjunction: the least Policy admitting everything either
/// operand admits.
///
/// This is the algebraic base of the derived cluster Policy law
/// `P_cluster = P_member_1 || … || P_member_n`
/// (`derived_cluster_policy` in `semantic_world`).  The result is a
/// derived fact only — it never becomes a storage or exposure
/// authority; exposure keeps filtering per member
/// (`Expose(cluster, phase) = { member_i | Expose(P_i, phase) }`).
///
/// EXCLUSIVITY: the member → whole-function-object P1 disjunction holds
/// between the members of one ClusterSymbol and nowhere else;
/// `derived_cluster_policy` is this function's only legitimate caller.
/// A Val2 name is itself a recursive ClusterSymbol (`Val2(T_t)[f] = C_f`), so
/// the same law applies one layer down: `P(C_f)` is the disjunction of `C_f`'s
/// OWN members.  What never happens is absorption across layers — a host
/// type/cluster does not disjoin its associated Symbols' Policies into its own,
/// layered exposure composes conjunctively (`∧`) at lookup, and a written `||`
/// inside one Policy spelling is elaborated within that single spelling only.
///
/// Component rules:
/// * stages — set union on both the value and pattern components;
/// * presence — `Present || Present = Present`,
///   `Absent || Absent = Absent`, any mix is `Optional`.
/// Whole-slot PolicyMode is deliberately absent: callers combine complete
/// [`PolicyView`] values without inventing a mode disjunction.
pub fn policy_or(a: &PolicyPair, b: &PolicyPair) -> PolicyPair {
    let presence = match (a.value.presence, b.value.presence) {
        (ValuePresence::Present, ValuePresence::Present) => ValuePresence::Present,
        (ValuePresence::Absent, ValuePresence::Absent) => ValuePresence::Absent,
        _ => ValuePresence::Optional,
    };
    PolicyPair {
        value: ValueComponentPolicy {
            stages: a.value.stages.union(&b.value.stages),
            presence,
        },
        pattern: PatternComponentPolicy {
            stages: a.pattern.stages.union(&b.pattern.stages),
        },
    }
}

/// Namespace declaration attributes adjacent to, but never part of, a
/// callable's canonical `Pv:Pp` pair.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DeclarationVisibility {
    pub namespace_visibility: Option<NamespaceVisibility>,
    pub export_root: bool,
}

/// Body-entry admissibility judged directly on the
/// callable's complete result P2 (`PolicyPair`).
///
/// This replaces the declaration-projection
/// `SymbolPayload::MetaFunction.body_entry_policy`
/// read on the invocation spine.  The body-entry domain is the value stage
/// set when present, otherwise the pattern stage set — the graph body-entry
/// PolicySet was installed as exactly this projection
/// (`legacy_policy_set_from_pair(&result_p2)`), so the judgement is
/// equivalent, only sourced from the semantic call entry's own P2.
pub fn body_entry_allows_execution(p2: &PolicyPair, env: crate::model::ExecutionEnv) -> bool {
    use crate::model::ExecutionEnv;
    let stages = if p2.value.stages.is_empty() {
        &p2.pattern.stages
    } else {
        &p2.value.stages
    };
    match env {
        ExecutionEnv::OpenStatic => {
            stages.contains(PolicyStage::Meta) || stages.contains(PolicyStage::Compile)
        }
        ExecutionEnv::SealStatic => {
            stages.contains(PolicyStage::Seal) || stages.contains(PolicyStage::Compile)
        }
        ExecutionEnv::Runtime => stages.contains(PolicyStage::Runtime),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum P1Projection {
    Infer,
    ValueDominant { value: ValueComponentPolicy },
    Pair(PolicyPair),
}

/// Candidate-independent result demand formed before root-call maxima.
/// `Infer` is the candidate-local default pair query; `mode` is always one
/// concrete point and is never inferred from the pair.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResultPolicyDemand {
    pub pair_query: P1Projection,
    pub mode: PolicyMode,
}

impl Default for ResultPolicyDemand {
    fn default() -> Self {
        Self {
            pair_query: P1Projection::Infer,
            mode: PolicyMode::Plain,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormalPolicyPattern {
    /// The parameter policy after inheriting its callable P2 and applying the
    /// optional const/mut-only formal slice.
    pub effective_pair: PolicyPair,
    /// Total overload-preference point. Omitted syntax forms concrete
    /// `PolicyMode::Plain`; it is never represented by `None`.
    pub mode: PolicyMode,
}

/// Effective policy of a declared return position.  Its pair/stage is
/// inherited from the callable P1 and cannot be rewritten at the position;
/// only the orthogonal whole-slot mode may be explicitly overridden.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReturnPolicyPattern {
    pub effective_view: PolicyView,
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
    /// Whole-slot declaration mode, factored before `projection` is formed.
    pub mode: PolicyMode,
    /// Root-local external projection derived when this declaration directly
    /// writes `export`. This is an early validation/preview only:
    /// `None` does not prove that the declaration is absent from the eventual
    /// export view, because `ExportRetentionClosure(root)` may admit ancestors
    /// and descendants. Namespace graph integration must combine retention
    /// membership with public path reachability before projecting candidates
    /// through `project_export_overload_sets`.
    pub external_projection: Option<P1Projection>,
    pub visibility: Option<NamespaceVisibility>,
    pub export_root: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionObjectDeclarationPolicy {
    /// Concrete whole-slot mode. Omitted source syntax forms `plain`.
    pub mode: PolicyMode,
}

impl Default for FunctionObjectDeclarationPolicy {
    fn default() -> Self {
        Self {
            mode: PolicyMode::Plain,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyResultEntry<V, P> {
    pub value: Option<V>,
    pub pattern: P,
    pub view: PolicyView,
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
/// identity. Export admission does not rewrite the candidate's Policy mode or
/// capability realization; external resolution consumes the same stable facts
/// that were fixed for the internal candidate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportCandidateView<I, C> {
    pub identity: I,
    pub internal_candidate: C,
    pub external_policy: PolicyPair,
    pub mode: PolicyMode,
    pub capability_realization: CapabilityRealization,
}

/// Resolved internal candidate view after its declaration-side `P1Projection`
/// has already been applied to the actual RHS/result entries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedCandidatePolicy {
    pub pair: PolicyPair,
    pub mode: PolicyMode,
    pub capability_realization: CapabilityRealization,
    pub provenance: Provenance,
}

/// Namespace-level facts required before a symbol may contribute candidate
/// views to `Sigma_export`.
///
/// Export-closure membership alone is not sufficient: every component of the
/// externally navigated path must also pass public/private reachability.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExportAdmission {
    pub in_export_retention_closure: bool,
    pub publicly_reachable: bool,
}

impl ExportAdmission {
    pub fn is_externally_exposed(self) -> bool {
        self.in_export_retention_closure && self.publicly_reachable
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
/// `ExportAdmission` combines export-retention-closure membership with public
/// path reachability. Only externally exposed symbols are considered.
/// Candidate Policy validation is then a separate operation over each resolved
/// internal `PolicyPair`. Export is an admission/view boundary, never a
/// const-cropping operation.
pub fn project_export_overload_sets<N: Clone + Ord, I, C: Clone>(
    full: BTreeMap<N, Vec<C>>,
    mut external_admission: impl FnMut(&N) -> ExportAdmission,
    mut resolve_candidate: impl FnMut(&C) -> (I, ResolvedCandidatePolicy),
) -> Result<NamespaceOverloadSets<N, I, C>, Diagnostic> {
    let mut exported = BTreeMap::new();
    for (name, candidates) in &full {
        if !external_admission(name).is_externally_exposed() {
            continue;
        }
        let mut projected = Vec::new();
        for candidate in candidates {
            let (identity, internal_policy) = resolve_candidate(candidate);
            let external_policy = project_resolved_export_view(&internal_policy)?;
            projected.push(ExportCandidateView {
                identity,
                internal_candidate: candidate.clone(),
                external_policy,
                mode: internal_policy.mode,
                capability_realization: internal_policy.capability_realization,
            });
        }
        if !projected.is_empty() {
            exported.insert(name.clone(), projected);
        }
    }
    Ok(NamespaceOverloadSets { full, exported })
}

/// Derive the externally readable pair from an already resolved internal
/// candidate view.
///
/// This function never accepts `P1Projection`: declaration projection has
/// already happened. Every component is preserved exactly; external admission
/// is orthogonal to Policy preference and capability realization.
pub fn project_resolved_export_view(
    internal_policy: &ResolvedCandidatePolicy,
) -> Result<PolicyPair, Diagnostic> {
    let projected = internal_policy.pair.clone();
    validate_value_component_invariant(
        &projected.value,
        "resolved export candidate",
        internal_policy.provenance.clone(),
    )?;
    Ok(projected)
}

/// Compute `PathAncestors(root) ∪ Subtree(root)` for every export root.
/// This is a retention/admission-input closure, not the externally exported
/// symbol set. Descendants cannot opt out, while siblings are included only
/// when they are themselves an ancestor/descendant of another root.
pub fn compute_export_retention_closure<I: Clone + Ord>(
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
    export_retention_closure: &BTreeSet<I>,
    nodes: &BTreeMap<I, NamespaceExportNode<I>>,
    path: impl IntoIterator<Item = I>,
) -> bool {
    export_retention_closure.contains(symbol) && publicly_reachable(nodes, path)
}

#[derive(Clone, Debug, Default)]
struct ComponentAtoms {
    stages: StageSet,
    mode_atoms: BTreeSet<PolicyMode>,
    namespace: BTreeSet<NamespaceVisibility>,
    export_root: bool,
    absent_value: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum PolicyDimension {
    Stage,
    Mode,
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
        if !self.mode_atoms.is_empty() {
            result.insert(PolicyDimension::Mode);
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
) -> Result<PolicyView, Diagnostic> {
    match (&policy.value_policy, &policy.pattern_policy) {
        (NormValuePolicyPattern::Conjunction(value), None) => {
            let atoms = parse_component(value, true, provenance.clone())?;
            reject_namespace_attributes(&atoms, "P2", provenance.clone())?;
            let mode = concrete_mode_atom(&atoms, "P2", provenance.clone())?;
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
            let pair = validate_p2_pair(
                PolicyPair {
                    value: ValueComponentPolicy {
                        stages: atoms.stages,
                        presence,
                    },
                    pattern: PatternComponentPolicy {
                        stages: pattern_stages,
                    },
                },
                provenance,
            )?;
            Ok(PolicyView { pair, mode })
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
            if !pattern_atoms.mode_atoms.is_empty() {
                return Err(policy_error(
                    "PolicyMode is a whole-slot coordinate and may not appear in Pp",
                    provenance,
                ));
            }
            let mode = concrete_mode_atom(&value_atoms, "P2", provenance.clone())?;
            let pair = validate_p2_pair(
                PolicyPair {
                    value: ValueComponentPolicy {
                        stages: value_atoms.stages,
                        presence: value_presence,
                    },
                    pattern: PatternComponentPolicy {
                        stages: pattern_atoms.stages,
                    },
                },
                provenance,
            )?;
            Ok(PolicyView { pair, mode })
        }
        (NormValuePolicyPattern::Absent { .. }, None) => Err(policy_error(
            "an absent P2 value component requires an explicit Pattern component",
            provenance,
        )),
    }
}

pub fn elaborate_binding_result_demand(
    policy: Option<&NormPolicySpec>,
    provenance: Provenance,
) -> Result<ResultPolicyDemand, Diagnostic> {
    let Some(policy) = policy else {
        return Ok(ResultPolicyDemand::default());
    };
    let (pair_query, mode, namespace, export_root) =
        elaborate_p1_components(policy, provenance.clone())?;
    if !namespace.is_empty() || export_root {
        return Err(policy_error(
            "public/private/export are valid only on namespace declarations",
            provenance,
        ));
    }
    Ok(ResultPolicyDemand { pair_query, mode })
}

pub fn elaborate_formal_policy_pattern(
    policy: Option<&NormPolicySpec>,
    inherited_p2: &PolicyView,
    provenance: Provenance,
) -> Result<FormalPolicyPattern, Diagnostic> {
    validate_value_component_invariant(
        &inherited_p2.pair.value,
        "formal inherited P2",
        provenance.clone(),
    )?;
    let Some(policy) = policy else {
        return Ok(FormalPolicyPattern {
            effective_pair: inherited_p2.pair.clone(),
            mode: inherited_p2.mode,
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
    let selected = explicit_mode_atom(&atoms, "formal parameter", provenance)?;
    Ok(FormalPolicyPattern {
        effective_pair: inherited_p2.pair.clone(),
        mode: selected,
    })
}

pub fn elaborate_return_policy_pattern(
    policy: Option<&NormPolicySpec>,
    inherited_p1: &PolicyView,
    provenance: Provenance,
) -> Result<ReturnPolicyPattern, Diagnostic> {
    validate_value_component_invariant(
        &inherited_p1.pair.value,
        "return inherited P1",
        provenance.clone(),
    )?;
    let Some(policy) = policy else {
        return Ok(ReturnPolicyPattern {
            effective_view: inherited_p1.clone(),
        });
    };
    if policy.pattern_policy.is_some() {
        return Err(policy_error(
            "return position policy may override only its whole-slot PolicyMode",
            provenance,
        ));
    }
    let atoms = match &policy.value_policy {
        NormValuePolicyPattern::Conjunction(value) => {
            parse_component(value, false, provenance.clone())?
        }
        NormValuePolicyPattern::Absent { .. } => {
            return Err(policy_error(
                "return position policy cannot rewrite inherited value presence",
                provenance,
            ));
        }
    };
    reject_namespace_attributes(&atoms, "return position", provenance.clone())?;
    if !atoms.stages.is_empty() || atoms.absent_value {
        return Err(policy_error(
            "return position policy inherits evaluation stages and may override only PolicyMode",
            provenance,
        ));
    }
    let mode = explicit_mode_atom(&atoms, "return position", provenance)?;
    Ok(ReturnPolicyPattern {
        effective_view: PolicyView {
            pair: inherited_p1.pair.clone(),
            mode,
        },
    })
}

/// Where an explicit P1 spelling appears.  The outer binding prefix
/// (`compile let f = ...`) doubles as declaration policy, so namespace
/// visibility/export atoms are ignored there (they are validated against
/// the derived symbol policy separately); the written-self slot policy is
/// pure P1 material and rejects them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExplicitP1Position {
    OuterBinding,
    WrittenSelf,
}

/// The per-dimension explicit P1 selection extracted from one spelling
/// site (outer binding prefix or written-self slot policy).
///
/// The explicit selection keeps the complete `Pv:Pp` coordinates and its
/// orthogonal whole-slot mode separate. Value stage, value presence, Pattern
/// stage, and mode are independently selectable. A
/// dimension that was not written stays `None` and falls back to
/// `Derive(P2)` in `canonical_function_object_p1`; a dimension written at
/// BOTH spelling sites must agree there or the canonicalizer hard-errors.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExplicitP1Selection {
    pub value_stages: Option<StageSet>,
    pub presence: Option<ValuePresence>,
    pub pattern_stages: Option<StageSet>,
    pub mode: Option<PolicyMode>,
}

impl ExplicitP1Selection {
    pub fn is_empty(&self) -> bool {
        self.value_stages.is_none()
            && self.presence.is_none()
            && self.pattern_stages.is_none()
            && self.mode.is_none()
    }

    /// A fully explicit selection carrying every dimension of `pair`.
    /// Used by core-callable registration, whose declared function policy
    /// is explicit by construction (there is no source spelling to parse).
    pub fn from_complete_view(view: &PolicyView) -> Self {
        Self {
            value_stages: Some(view.pair.value.stages.clone()),
            presence: Some(view.pair.value.presence),
            pattern_stages: Some(view.pair.pattern.stages.clone()),
            mode: Some(view.mode),
        }
    }
}

/// Elaborate an explicit P1 spelling into its per-dimension selection.
///
/// Returns `Ok(None)` when nothing P1-relevant was written (no policy, or
/// an outer prefix carrying only namespace visibility/export atoms).
/// Stage atoms ARE an explicit P1 stage selection — a stage-only outer
/// policy must never be treated as "no explicit P1".
pub fn elaborate_explicit_p1(
    policy: Option<&NormPolicySpec>,
    _inherited_p2: &PolicyPair,
    position: ExplicitP1Position,
    provenance: Provenance,
) -> Result<Option<ExplicitP1Selection>, Diagnostic> {
    let Some(policy) = policy else {
        return Ok(None);
    };
    let mut selection = ExplicitP1Selection::default();

    // Pattern component of an explicit `Pv:Pp` pair projection.
    if let Some(pattern) = &policy.pattern_policy {
        let pattern_atoms = parse_component(pattern, false, provenance.clone())?;
        reject_namespace_attributes(
            &pattern_atoms,
            "explicit P1 Pattern component",
            provenance.clone(),
        )?;
        if !pattern_atoms.mode_atoms.is_empty() {
            return Err(policy_error(
                "PolicyMode is a whole-slot coordinate and may not appear in Pp",
                provenance,
            ));
        }
        if pattern_atoms.stages.is_empty() {
            return Err(policy_error(
                "an explicit P1 Pattern component requires at least one stage",
                provenance,
            ));
        }
        selection.pattern_stages = Some(pattern_atoms.stages);
    }

    let value_atoms = match &policy.value_policy {
        NormValuePolicyPattern::Conjunction(value) => {
            parse_component(value, true, provenance.clone())?
        }
        NormValuePolicyPattern::Absent { .. } => {
            if policy.pattern_policy.is_none() {
                return Err(policy_error(
                    "an absent explicit P1 value pattern requires an explicit Pattern component",
                    provenance,
                ));
            }
            selection.presence = Some(ValuePresence::Absent);
            return Ok(Some(selection));
        }
    };
    match position {
        // Visibility/export atoms in the outer prefix are namespace
        // declaration attributes, separate from the function-object P1.
        ExplicitP1Position::OuterBinding => {}
        ExplicitP1Position::WrittenSelf => {
            reject_namespace_attributes(
                &value_atoms,
                "written-self explicit P1",
                provenance.clone(),
            )?;
        }
    }
    if !value_atoms.stages.is_empty() {
        selection.value_stages = Some(value_atoms.stages.clone());
    }
    if value_atoms.absent_value {
        selection.presence = Some(value_atoms.presence());
    }
    if !value_atoms.mode_atoms.is_empty() {
        selection.mode = Some(explicit_mode_atom(&value_atoms, "explicit P1", provenance)?);
    }
    if selection.is_empty() {
        Ok(None)
    } else {
        Ok(Some(selection))
    }
}

pub fn elaborate_namespace_declaration_policy(
    policy: Option<&NormPolicySpec>,
    position: NamespaceDeclarationPosition,
    provenance: Provenance,
) -> Result<NamespaceDeclarationPolicy, Diagnostic> {
    let Some(policy) = policy else {
        return Ok(NamespaceDeclarationPolicy {
            projection: P1Projection::Infer,
            mode: PolicyMode::Plain,
            external_projection: None,
            visibility: None,
            export_root: false,
        });
    };
    let (projection, mode, namespace, export_root) =
        elaborate_p1_components(policy, provenance.clone())?;
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
        mode,
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
    let projected = projection.clone();
    let value = match &projected {
        P1Projection::ValueDominant { value } => value,
        P1Projection::Pair(pair) => &pair.value,
        P1Projection::Infer => {
            return Err(policy_error(
                "an export root requires an explicit namespace declaration policy",
                provenance,
            ));
        }
    };

    validate_value_component_invariant(value, "export-root P1", provenance.clone())?;
    Ok(projected)
}

fn elaborate_p1_components(
    policy: &NormPolicySpec,
    provenance: Provenance,
) -> Result<
    (
        P1Projection,
        PolicyMode,
        BTreeSet<NamespaceVisibility>,
        bool,
    ),
    Diagnostic,
> {
    match (&policy.value_policy, &policy.pattern_policy) {
        (NormValuePolicyPattern::Conjunction(value), None) => {
            let atoms = parse_component(value, true, provenance.clone())?;
            let mode = concrete_mode_atom(&atoms, "P1", provenance.clone())?;
            let value = ValueComponentPolicy {
                stages: atoms.stages.clone(),
                presence: atoms.presence(),
            };
            validate_value_component_invariant(&value, "P1 value component", provenance)?;
            let projection = P1Projection::ValueDominant { value };
            Ok((projection, mode, atoms.namespace, atoms.export_root))
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
            if !pattern_atoms.mode_atoms.is_empty() {
                return Err(policy_error(
                    "PolicyMode is a whole-slot coordinate and may not appear in Pp",
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
            let mode = concrete_mode_atom(&value_atoms, "P1", provenance.clone())?;
            one_namespace(&namespace, provenance.clone())?;
            let value_presence = value_atoms.presence();
            let value = ValueComponentPolicy {
                stages: value_atoms.stages,
                presence: value_presence,
            };
            validate_value_component_invariant(&value, "P1 value component", provenance.clone())?;
            Ok((
                P1Projection::Pair(PolicyPair {
                    value,
                    pattern: PatternComponentPolicy {
                        stages: pattern_atoms.stages,
                    },
                }),
                mode,
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
    FunctionObjectDeclarationPolicy {
        mode: declaration.mode,
    }
}

pub fn derive_function_object_view(
    result_p2: &PolicyView,
    declaration: &FunctionObjectDeclarationPolicy,
) -> PolicyView {
    PolicyView {
        pair: PolicyPair {
            value: ValueComponentPolicy {
                stages: result_p2
                    .pair
                    .value
                    .stages
                    .union(&result_p2.pair.pattern.stages),
                presence: ValuePresence::Present,
            },
            pattern: PatternComponentPolicy {
                stages: result_p2.pair.pattern.stages.clone(),
            },
        },
        mode: declaration.mode,
    }
}

/// Apply a P1 projection as a real slice restriction. The returned entries are
/// owned views whose pair stage/presence coordinates are cropped; whole-slot
/// mode and associated value/Pattern identities are cloned unchanged.
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
                    pattern: entry.pattern.clone(),
                    view: PolicyView {
                        pair: PolicyPair {
                            value: value_policy,
                            pattern: entry.view.pair.pattern.clone(),
                        },
                        mode: entry.view.mode,
                    },
                })
            }
            P1Projection::Pair(pair) => {
                let value_policy = restrict_value_policy(&pair.value, entry)?;
                let pattern_stages =
                    restrict_stages(&pair.pattern.stages, &entry.view.pair.pattern.stages)?;
                Some(PolicyResultEntry {
                    value: entry.value.clone(),
                    pattern: entry.pattern.clone(),
                    view: PolicyView {
                        pair: PolicyPair {
                            value: value_policy,
                            pattern: PatternComponentPolicy {
                                stages: pattern_stages,
                            },
                        },
                        mode: entry.view.mode,
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
        // A pure-P entry still answers a stage slice: the visible policy is
        // the requested restriction of the entry policy (P1 is the visible
        // policy authority), never the entry policy verbatim.
        let stages = restrict_stages(&query.stages, &entry.view.pair.value.stages)?;
        return Some(ValueComponentPolicy {
            stages,
            presence: entry.view.pair.value.presence,
        });
    }
    let stages = restrict_stages(&query.stages, &entry.view.pair.value.stages)?;
    Some(ValueComponentPolicy {
        stages,
        presence: entry.view.pair.value.presence,
    })
}

pub(crate) fn restrict_stages(query: &StageSet, available: &StageSet) -> Option<StageSet> {
    if query.is_empty() {
        return Some(available.clone());
    }
    let selected = query.intersection(available);
    (!selected.is_empty()).then_some(selected)
}

fn validate_p2_pair(pair: PolicyPair, provenance: Provenance) -> Result<PolicyPair, Diagnostic> {
    validate_value_component_invariant(&pair.value, "P2 value component", provenance.clone())?;
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

fn validate_value_component_invariant(
    value: &ValueComponentPolicy,
    context: &str,
    provenance: Provenance,
) -> Result<(), Diagnostic> {
    if value.presence == ValuePresence::Absent && !value.stages.is_empty() {
        return Err(policy_error(
            format!("{context}: `Pv = absent` cannot carry value stages"),
            provenance,
        ));
    }
    Ok(())
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
                atoms.mode_atoms.insert(PolicyMode::Const);
            }
            "plain" => {
                atoms.mode_atoms.insert(PolicyMode::Plain);
            }
            "mut" => {
                atoms.mode_atoms.insert(PolicyMode::Mut);
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
    result.mode_atoms.extend(next.mode_atoms);
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
        PolicyDimension::Mode => result.mode_atoms.extend(next.mode_atoms),
        PolicyDimension::NamespaceVisibility => result.namespace.extend(next.namespace),
        PolicyDimension::ExportRoot => result.export_root |= next.export_root,
        PolicyDimension::ValuePresence => result.absent_value |= next.absent_value,
    }
}

fn concrete_mode_atom(
    atoms: &ComponentAtoms,
    context: &str,
    provenance: Provenance,
) -> Result<PolicyMode, Diagnostic> {
    match atoms.mode_atoms.len() {
        0 => Ok(PolicyMode::Plain),
        1 => Ok(*atoms.mode_atoms.iter().next().expect("one mode atom")),
        _ => Err(policy_error(
            format!(
                "{context} must select exactly one whole-slot PolicyMode; mode choices are not a PolicyPair domain"
            ),
            provenance,
        )),
    }
}

fn explicit_mode_atom(
    atoms: &ComponentAtoms,
    context: &str,
    provenance: Provenance,
) -> Result<PolicyMode, Diagnostic> {
    if atoms.mode_atoms.is_empty() {
        return Err(policy_error(
            format!("{context} must select one of const, plain, or mut"),
            provenance,
        ));
    }
    concrete_mode_atom(atoms, context, provenance)
}

fn policy_error(message: impl Into<String>, provenance: Provenance) -> Diagnostic {
    Diagnostic::hard_error(message, Some(provenance))
}
