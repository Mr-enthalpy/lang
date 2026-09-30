use std::collections::{BTreeMap, BTreeSet, VecDeque};

use lang_syntax::{NormPolicyAtom, NormPolicyConjunction, NormPolicySpec};

use crate::{Diagnostic, Provenance};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stage {
    Meta,
    Compile,
    Seal,
    Runtime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Visibility domain for an observation. This is neither an evaluator phase,
/// a readiness proof, nor a scheduling queue.
pub enum ObservationHorizon {
    OpenStatic,
    SealStatic,
    Runtime,
}

impl Stage {
    pub fn visible_at(self, horizon: ObservationHorizon) -> bool {
        match self {
            Self::Meta => horizon == ObservationHorizon::OpenStatic,
            Self::Compile => matches!(
                horizon,
                ObservationHorizon::OpenStatic | ObservationHorizon::SealStatic
            ),
            Self::Seal => horizon == ObservationHorizon::SealStatic,
            Self::Runtime => horizon == ObservationHorizon::Runtime,
        }
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

/// One resolved value observation and its independently formed Pattern stage.
pub fn declared_policy_view(stage: Stage, mode: PolicyMode) -> PolicyView {
    PolicyView {
        pair: PolicyPair {
            value: ValueComponentPolicy::Present(stage),
            pattern: PatternComponentPolicy {
                stage: if stage == Stage::Runtime {
                    Stage::Compile
                } else {
                    stage
                },
            },
        },
        mode,
    }
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

/// Independent capability axis of `CallableSemantics`.
///
/// Privilege states what special operations a callable may perform (for
/// example consuming raw/meta AST material).  It implies nothing about
/// the declared result class and nothing about the Policy stage: `struct` is
/// a privileged built-in whose result class is `CompleteType`, while `assert`
/// and `verify` declare ordinary-value results.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallablePrivilege {
    /// Ordinary source-declared callable; the source surface can never
    /// spell a privilege.
    OrdinarySource,
    /// Compiler-provided built-in with privileged capabilities.
    BuiltinPrivileged,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValuePresence {
    Present,
    Optional,
    Absent,
}

/// A resolved value observation carries one atom, or no value observation.
/// Absence cannot carry a stage; it is distinct from an unhidden pure Object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueComponentPolicy {
    Present(Stage),
    Optional(Stage),
    Absent,
}

impl ValueComponentPolicy {
    pub fn stage(self) -> Option<Stage> {
        match self {
            Self::Present(s) | Self::Optional(s) => Some(s),
            Self::Absent => None,
        }
    }
    pub fn presence(self) -> ValuePresence {
        match self {
            Self::Present(_) => ValuePresence::Present,
            Self::Optional(_) => ValuePresence::Optional,
            Self::Absent => ValuePresence::Absent,
        }
    }
}

/// Uncompleted query material; omission imposes no stage constraint.
/// This is not a resolved Policy and contains no stage union.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValuePolicyQuery {
    pub stage: Option<Stage>,
    pub presence: ValuePresence,
}
impl From<ValueComponentPolicy> for ValuePolicyQuery {
    fn from(value: ValueComponentPolicy) -> Self {
        Self {
            stage: value.stage(),
            presence: value.presence(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PatternComponentPolicy {
    pub stage: Stage,
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

/// Namespace declaration attributes adjacent to, but never part of, a
/// callable's canonical `Pv:Pp` pair.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DeclarationVisibility {
    pub namespace_visibility: Option<NamespaceVisibility>,
    pub export_root: bool,
}

/// Horizon visibility only. Ready and active-frame dominance are separate judgments.
pub fn body_entry_visible_at(p2: &PolicyPair, horizon: ObservationHorizon) -> bool {
    p2.value
        .stage()
        .unwrap_or(p2.pattern.stage)
        .visible_at(horizon)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum P1Projection {
    Infer,
    ValueDominant { value: ValuePolicyQuery },
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
    stage: Option<Stage>,
    mode_atoms: BTreeSet<PolicyMode>,
    namespace: BTreeSet<NamespaceVisibility>,
    export_root: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum PolicyDimension {
    Stage,
    Mode,
    NamespaceVisibility,
    ExportRoot,
}

impl ComponentAtoms {
    fn dimensions(&self) -> BTreeSet<PolicyDimension> {
        let mut result = BTreeSet::new();
        if self.stage.is_some() {
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
        result
    }
}

pub fn normalize_p2_policy(
    policy: &NormPolicySpec,
    provenance: Provenance,
) -> Result<PolicyView, Diagnostic> {
    let atoms = parse_component(&policy.constraint, provenance.clone())?;
    reject_namespace_attributes(&atoms, "P2", provenance.clone())?;
    let mode = concrete_mode_atom(&atoms, "P2", provenance.clone())?;
    let stage = atoms
        .stage
        .ok_or_else(|| policy_error("P2 requires one concrete stage", provenance.clone()))?;
    let pair = validate_p2_pair(declared_policy_view(stage, mode).pair, provenance)?;
    Ok(PolicyView { pair, mode })
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
    let Some(policy) = policy else {
        return Ok(FormalPolicyPattern {
            effective_pair: inherited_p2.pair.clone(),
            mode: inherited_p2.mode,
        });
    };
    let atoms = parse_component(&policy.constraint, provenance.clone())?;
    reject_namespace_attributes(&atoms, "formal parameter", provenance.clone())?;
    if atoms.stage.is_some() {
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
    let Some(policy) = policy else {
        return Ok(ReturnPolicyPattern {
            effective_view: inherited_p1.clone(),
        });
    };
    let atoms = parse_component(&policy.constraint, provenance.clone())?;
    reject_namespace_attributes(&atoms, "return position", provenance.clone())?;
    if atoms.stage.is_some() {
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
/// the derived symbol policy separately); the self-slot policy is
/// pure P1 material and rejects them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExplicitP1Position {
    OuterBinding,
    WrittenSelf,
}

/// The per-dimension explicit P1 selection extracted from one spelling
/// site (outer binding prefix or self-slot policy).
///
/// The explicit selection keeps the complete `Pv:Pp` coordinates and its
/// orthogonal whole-slot mode separate. Value stage, value presence, Pattern
/// stage, and mode are independently selectable. A
/// dimension that was not written stays `None` and falls back to
/// `Derive(P2)` in `canonical_function_object_p1`; a dimension written at
/// BOTH spelling sites must agree there or the canonicalizer hard-errors.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExplicitP1Selection {
    pub value_stage: Option<Stage>,
    pub presence: Option<ValuePresence>,
    pub pattern_stage: Option<Stage>,
    pub mode: Option<PolicyMode>,
}

impl ExplicitP1Selection {
    pub fn is_empty(&self) -> bool {
        self.value_stage.is_none()
            && self.presence.is_none()
            && self.pattern_stage.is_none()
            && self.mode.is_none()
    }

    /// A fully explicit selection carrying every dimension of `pair`.
    /// Used by core-callable registration, whose declared function policy
    /// is explicit by construction (there is no source spelling to parse).
    pub fn from_complete_view(view: &PolicyView) -> Self {
        Self {
            value_stage: view.pair.value.stage(),
            presence: Some(view.pair.value.presence()),
            pattern_stage: Some(view.pair.pattern.stage),
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

    let value_atoms = parse_component(&policy.constraint, provenance.clone())?;
    match position {
        // Visibility/export atoms in the outer prefix are namespace
        // declaration attributes, separate from the function-object P1.
        ExplicitP1Position::OuterBinding => {}
        ExplicitP1Position::WrittenSelf => {
            reject_namespace_attributes(&value_atoms, "self-slot explicit P1", provenance.clone())?;
        }
    }
    if value_atoms.stage.is_some() {
        selection.value_stage = value_atoms.stage;
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
    if matches!(projected, P1Projection::Infer) {
        return Err(policy_error(
            "an export root requires an explicit namespace declaration policy",
            provenance,
        ));
    }
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
    let atoms = parse_component(&policy.constraint, provenance.clone())?;
    let mode = concrete_mode_atom(&atoms, "P1", provenance.clone())?;
    let value = ValuePolicyQuery {
        stage: atoms.stage,
        presence: ValuePresence::Present,
    };
    let projection = P1Projection::ValueDominant { value };
    Ok((projection, mode, atoms.namespace, atoms.export_root))
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
    let mut view = result_p2.clone();
    view.mode = declaration.mode;
    view
}

/// Apply a P1 projection as a real slice restriction. The returned entries are
/// owned matching observations; no resolved atom is cropped or unioned.
/// Whole-slot mode and associated value/Pattern identities stay unchanged.
pub fn project_p1<V: Clone, P: Clone>(
    projection: &P1Projection,
    result: &[PolicyResultEntry<V, P>],
) -> Vec<PolicyResultEntry<V, P>> {
    result
        .iter()
        .filter(|entry| match projection {
            P1Projection::Infer => true,
            P1Projection::ValueDominant { value } => value_query_matches(*value, entry),
            P1Projection::Pair(pair) => {
                value_query_matches(pair.value.into(), entry)
                    && pair.pattern.stage == entry.view.pair.pattern.stage
            }
        })
        .cloned()
        .collect()
}

fn value_query_matches<V, P>(query: ValuePolicyQuery, entry: &PolicyResultEntry<V, P>) -> bool {
    let presence = query.presence == ValuePresence::Optional
        || entry.view.pair.value.presence() == ValuePresence::Optional
        || query.presence == entry.view.pair.value.presence();
    presence
        && query
            .stage
            .is_none_or(|s| entry.view.pair.value.stage() == Some(s))
}

fn validate_p2_pair(pair: PolicyPair, provenance: Provenance) -> Result<PolicyPair, Diagnostic> {
    if pair.pattern.stage == Stage::Runtime
        || pair
            .value
            .stage()
            .is_some_and(|s| s != Stage::Runtime && s != pair.pattern.stage)
    {
        return Err(policy_error(
            "P2 requires compatible value and static Pattern atoms",
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
    provenance: Provenance,
) -> Result<ComponentAtoms, Diagnostic> {
    let mut result = ComponentAtoms::default();
    for atom in &conjunction.atoms {
        let next = parse_atom(atom, provenance.clone())?;
        merge_conjunction(&mut result, next, provenance.clone())?;
    }
    Ok(result)
}

fn parse_atom(atom: &NormPolicyAtom, provenance: Provenance) -> Result<ComponentAtoms, Diagnostic> {
    let mut atoms = ComponentAtoms::default();
    match atom {
        NormPolicyAtom::Name { text, .. } => match text.as_str() {
            "meta" => atoms.stage = Some(Stage::Meta),
            "compile" => atoms.stage = Some(Stage::Compile),
            "seal" => atoms.stage = Some(Stage::Seal),
            "runtime" => atoms.stage = Some(Stage::Runtime),
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
            return parse_component(conjunction, provenance);
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
    result.stage = result.stage.or(next.stage);
    result.mode_atoms.extend(next.mode_atoms);
    result.namespace.extend(next.namespace);
    result.export_root |= next.export_root;
    Ok(())
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
