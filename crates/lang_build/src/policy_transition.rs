//! Cross-policy value-transition substrate.
//!
//! This module contains a transition-demand model plus a transitional
//! candidate-algebra prototype. The prototype shares the ordinary
//! maximal-element rule, but it does not yet resolve a distinguished global
//! Symbol, enumerate heterogeneous Val2, construct an `InvocationFrame`, or
//! invoke the repository's ordinary function-object pipeline. It must not
//! become a second permanent callable representation.
//!
//! Ordinary binding semantics stay in `policy_pair::project_p1`: omitted P1
//! preserves the complete RHS and any non-empty explicit projection completes
//! binding elaboration. This module prepares a transition request only after an
//! explicit query projects no value-bearing entry.

use std::{collections::BTreeSet, convert::Infallible};

use crate::{
    identity::{SemanticValueId, TypeValueId},
    model::{Provenance, SymbolId},
    policy_overload::maximal_candidates,
    policy_pair::{
        project_p1, P1Projection, PatternComponentPolicy, PolicyPair, PolicyResultEntry,
        PolicyStage, StageSet, ValueMutability, ValuePresence,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum P1Origin {
    Inferred,
    Explicit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemanticValueRef {
    pub id: SemanticValueId,
    pub type_value: TypeValueId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyTransitionDemand<P> {
    pub request: PolicyTransitionRequest,
    pub pattern: P,
}

/// P1 elaboration for a value-bearing multi-entry result.
///
/// Ordinary P1 remains a projection query. A non-empty projection completes
/// binding elaboration. Transition preparation is considered only when the
/// complete query projects no existing entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum P1Elaboration<P> {
    Projected {
        origin: P1Origin,
        requested: Option<P1Projection>,
        selected: Vec<PolicyResultEntry<SemanticValueRef, P>>,
    },
    Transition {
        requested: P1Projection,
        demands: Vec<PolicyTransitionDemand<P>>,
    },
}

/// Projection-only P1 elaboration for a pure type/Pattern result.
///
/// `Infallible` makes `Some(value)` unconstructible and this carrier exposes no
/// transition field. A pure type therefore cannot enter transition machinery.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PureTypeP1Elaboration<P> {
    pub origin: P1Origin,
    pub requested: Option<P1Projection>,
    pub selected: Vec<PolicyResultEntry<Infallible, P>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum P1ElaborationFailure {
    EmptyResult,
    ValueBearingInputContainsAbsentValue,
    ProjectionUnavailableWithoutValue {
        requested: P1Projection,
    },
    InvalidTransitionSource {
        entry_index: usize,
        failure: PolicyTransitionRequestFailure,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyTransitionRequest {
    source_policy: PolicyPair,
    target_query: PolicyPair,
    source_type: TypeValueId,
    source_value: SemanticValueId,
    provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyTransitionRequestFailure {
    SourceValueAbsent,
    SourceValueStageDomainEmpty,
}

impl PolicyTransitionRequest {
    pub fn new(
        source_policy: PolicyPair,
        target_query: PolicyPair,
        source_type: TypeValueId,
        source_value: SemanticValueId,
        provenance: Provenance,
    ) -> Result<Self, PolicyTransitionRequestFailure> {
        if source_policy.value.presence == ValuePresence::Absent {
            return Err(PolicyTransitionRequestFailure::SourceValueAbsent);
        }
        if source_policy.value.stages.is_empty() {
            return Err(PolicyTransitionRequestFailure::SourceValueStageDomainEmpty);
        }
        Ok(Self {
            source_policy,
            target_query,
            source_type,
            source_value,
            provenance,
        })
    }

    pub fn source_policy(&self) -> &PolicyPair {
        &self.source_policy
    }

    pub fn target_query(&self) -> &PolicyPair {
        &self.target_query
    }

    pub fn source_type(&self) -> TypeValueId {
        self.source_type
    }

    pub fn source_value(&self) -> SemanticValueId {
        self.source_value
    }

    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }
}

/// Elaborate P1 over value-bearing RHS result entries.
///
/// Omitted P1 preserves the complete RHS entries exactly. An explicit P1 first
/// runs the canonical `project_p1` query over the complete result. Transition
/// preparation occurs only when that projection is empty. Candidate input
/// policy can still project a slice of each demand's source policy.
pub fn elaborate_value_binding_p1<P: Clone>(
    result: &[PolicyResultEntry<SemanticValueRef, P>],
    explicit_p1: Option<&P1Projection>,
    provenance: Provenance,
) -> Result<P1Elaboration<P>, P1ElaborationFailure> {
    if result.is_empty() {
        return Err(P1ElaborationFailure::EmptyResult);
    }
    if result.iter().any(|entry| entry.value.is_none()) {
        return Err(P1ElaborationFailure::ValueBearingInputContainsAbsentValue);
    }

    let Some(projection) = explicit_p1 else {
        return Ok(P1Elaboration::Projected {
            origin: P1Origin::Inferred,
            requested: None,
            selected: result.to_vec(),
        });
    };

    let selected = project_p1(projection, result);
    if !selected.is_empty() {
        return Ok(P1Elaboration::Projected {
            origin: P1Origin::Explicit,
            requested: Some(projection.clone()),
            selected,
        });
    }

    let mut demands = Vec::with_capacity(result.len());
    for (entry_index, entry) in result.iter().enumerate() {
        let source = entry
            .value
            .expect("value-bearing input was checked before elaboration");
        let source_policy = policy_pair_from_entry(entry);
        let target_query = transition_target_query(projection, entry);
        let request = PolicyTransitionRequest::new(
            source_policy,
            target_query,
            source.type_value,
            source.id,
            provenance.clone(),
        )
        .map_err(|failure| P1ElaborationFailure::InvalidTransitionSource {
            entry_index,
            failure,
        })?;
        demands.push(PolicyTransitionDemand {
            request,
            pattern: entry.pattern.clone(),
        });
    }

    Ok(P1Elaboration::Transition {
        requested: projection.clone(),
        demands,
    })
}

/// Elaborate P1 over a pure type/Pattern result.
///
/// This API requires `PolicyResultEntry<Infallible, P>` and returns a carrier
/// with no transition demand field. An unavailable Pattern slice is a
/// projection failure.
pub fn elaborate_pure_type_binding_p1<P: Clone>(
    result: &[PolicyResultEntry<Infallible, P>],
    explicit_p1: Option<&P1Projection>,
) -> Result<PureTypeP1Elaboration<P>, P1ElaborationFailure> {
    if result.is_empty() {
        return Err(P1ElaborationFailure::EmptyResult);
    }
    let Some(projection) = explicit_p1 else {
        return Ok(PureTypeP1Elaboration {
            origin: P1Origin::Inferred,
            requested: None,
            selected: result.to_vec(),
        });
    };

    let selected = project_p1(projection, result);
    if selected.is_empty() {
        return Err(P1ElaborationFailure::ProjectionUnavailableWithoutValue {
            requested: projection.clone(),
        });
    }
    Ok(PureTypeP1Elaboration {
        origin: P1Origin::Explicit,
        requested: Some(projection.clone()),
        selected,
    })
}

fn transition_target_query<V, P>(
    projection: &P1Projection,
    entry: &PolicyResultEntry<V, P>,
) -> PolicyPair {
    match projection {
        P1Projection::Pair(pair) => pair.clone(),
        P1Projection::ValueDominant { value } => PolicyPair {
            value: value.clone(),
            pattern: entry.pattern_policy.clone(),
            namespace_visibility: None,
            export_root: false,
        },
        P1Projection::Infer => policy_pair_from_entry(entry),
    }
}

fn policy_pair_from_entry<V, P>(entry: &PolicyResultEntry<V, P>) -> PolicyPair {
    PolicyPair {
        value: entry.value_policy.clone(),
        pattern: entry.pattern_policy.clone(),
        namespace_visibility: None,
        export_root: false,
    }
}

/// Apply an ordinary pair-shaped P1 query to one available policy view.
///
/// This deliberately delegates to `project_p1` so transition candidate input
/// and output adaptation cannot grow a competing policy-slicing rule.
fn project_policy_query(query: &PolicyPair, available: &PolicyPair) -> Option<PolicyPair> {
    let entry = PolicyResultEntry {
        value: Some(()),
        value_policy: available.value.clone(),
        pattern: (),
        pattern_policy: available.pattern.clone(),
    };
    project_p1(&P1Projection::Pair(query.clone()), &[entry])
        .into_iter()
        .next()
        .map(|selected| policy_pair_from_entry(&selected))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyTransitionFailure {
    SourceValueAbsent,
    TargetValueNotRuntime {
        target_value_stages: StageSet,
        target_value_presence: ValuePresence,
    },
    ValuePatternStageOverlap {
        overlap: StageSet,
    },
    PatternPolicyChanged {
        source: PatternComponentPolicy,
        target: PatternComponentPolicy,
    },
}

/// Validate the frozen Runtime Val1 transition shape.
///
/// This function is intentionally not a validator for meta/compile/seal
/// transitions. Those transition shapes remain separate semantic work.
pub fn validate_runtime_transition(
    source: &PolicyPair,
    target: &PolicyPair,
) -> Result<(), PolicyTransitionFailure> {
    if source.value.presence == ValuePresence::Absent || source.value.stages.is_empty() {
        return Err(PolicyTransitionFailure::SourceValueAbsent);
    }
    let runtime_only = target.value.presence == ValuePresence::Present
        && target.value.stages.len() == 1
        && target.value.stages.contains(PolicyStage::Runtime);
    if !runtime_only {
        return Err(PolicyTransitionFailure::TargetValueNotRuntime {
            target_value_stages: target.value.stages.clone(),
            target_value_presence: target.value.presence,
        });
    }

    let overlap = target.value.stages.intersection(&target.pattern.stages);
    if !overlap.is_empty() {
        return Err(PolicyTransitionFailure::ValuePatternStageOverlap { overlap });
    }

    if target.pattern != source.pattern {
        return Err(PolicyTransitionFailure::PatternPolicyChanged {
            source: source.pattern.clone(),
            target: target.pattern.clone(),
        });
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrdinaryCallableTypeInput {
    Any,
    Exact(TypeValueId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrdinaryCallableTypeOutput {
    SameAsInput,
    Exact(TypeValueId),
}

impl OrdinaryCallableTypeOutput {
    fn resolve(self, source: TypeValueId) -> TypeValueId {
        match self {
            Self::SameAsInput => source,
            Self::Exact(output) => output,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyBridgeBody {
    /// Prototype fixture for atomic static-value to runtime-value copying.
    BuiltinValueCopy,
    /// Placeholder for the Symbol identity that a future ordinary invocation
    /// adapter must route through the global function-object resolver.
    UserCallable(SymbolId),
    /// Bounded fixture body for candidate-algebra tests.
    IntrinsicStub(String),
    /// Test/lowering carrier for a failure after a winner has already been
    /// selected. The failure has no access to the discarded candidate set.
    FailAfterSelection(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Transitional candidate representation for Policy preference experiments.
///
/// This is not a global function object and is not the final bridge
/// architecture. Future integration must derive these comparison dimensions
/// from ordinary prepared callable candidates.
pub struct PolicyTransitionCallable<I> {
    pub id: I,
    pub input_type: OrdinaryCallableTypeInput,
    pub output_type: OrdinaryCallableTypeOutput,
    pub input_policy: PolicyPair,
    pub output_policy: PolicyPair,
    /// Hard conditions owned by ordinary candidate preparation (shape,
    /// require/concept checks, body availability, and similar facts).
    pub ordinary_fully_admissible: bool,
    pub is_delete: bool,
    pub body: PolicyBridgeBody,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TransitionTypeExpectation {
    /// This is a hard applicability expectation only. It never contributes an
    /// output-type preference rank.
    pub required_output_type: Option<TypeValueId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyPartialOrdering {
    /// The left candidate is less preferred.
    Less,
    Equal,
    /// The left candidate is more preferred.
    Greater,
    Incomparable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedPolicyBridge<I> {
    pub callable: PolicyTransitionCallable<I>,
    pub result_type: TypeValueId,
    pub result_policy: PolicyPair,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyBridgeResolution<I> {
    Selected(ResolvedPolicyBridge<I>),
    RejectedByDelete(I),
    Ambiguous(Vec<I>),
    NoCandidate,
}

/// Compare only the input/output Policy dimensions of two transition
/// candidates. `Greater` means `a` dominates `b`.
///
/// Input and output are composed as one Pareto/product order. Crossed
/// advantages are therefore `Incomparable`.
pub fn compare_policy_transition_candidates<I>(
    required_source: &PolicyPair,
    target_query: &PolicyPair,
    a: &PolicyTransitionCallable<I>,
    b: &PolicyTransitionCallable<I>,
) -> PolicyPartialOrdering {
    compose_orders([
        compare_input_policy_fit(required_source, &a.input_policy, &b.input_policy),
        compare_output_policy_fit(target_query, &a.output_policy, &b.output_policy),
    ])
}

/// Select from a caller-supplied transitional candidate family.
///
/// This function proves the input/output Policy product order and direct-only
/// selection rule. It does not perform global Symbol lookup or ordinary
/// function-object invocation. The resolver never feeds a candidate result
/// back as another request and therefore cannot perform transitive search.
pub fn resolve_policy_bridge<I: Clone>(
    request: &PolicyTransitionRequest,
    candidates: &[PolicyTransitionCallable<I>],
    expectation: TransitionTypeExpectation,
) -> PolicyBridgeResolution<I> {
    let admissible = candidates
        .iter()
        .filter(|candidate| candidate_is_fully_admissible(request, candidate, expectation))
        .collect::<Vec<_>>();
    if admissible.is_empty() {
        return PolicyBridgeResolution::NoCandidate;
    }

    let maximal = maximal_candidates(&admissible, |better, worse| {
        candidate_dominates(request, better, worse)
    });

    match maximal.as_slice() {
        [] => PolicyBridgeResolution::NoCandidate,
        [candidate] if candidate.is_delete => {
            PolicyBridgeResolution::RejectedByDelete(candidate.id.clone())
        }
        [candidate] => PolicyBridgeResolution::Selected(ResolvedPolicyBridge {
            callable: (*candidate).clone(),
            result_type: candidate.output_type.resolve(request.source_type()),
            result_policy: project_policy_query(request.target_query(), &candidate.output_policy)
                .expect("selected candidate output was projected during admissibility"),
        }),
        candidates => PolicyBridgeResolution::Ambiguous(
            candidates
                .iter()
                .map(|candidate| candidate.id.clone())
                .collect(),
        ),
    }
}

pub fn policy_bridge_is_available<I: Clone>(
    request: &PolicyTransitionRequest,
    candidates: &[PolicyTransitionCallable<I>],
    expectation: TransitionTypeExpectation,
) -> bool {
    matches!(
        resolve_policy_bridge(request, candidates, expectation),
        PolicyBridgeResolution::Selected(_) | PolicyBridgeResolution::RejectedByDelete(_)
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransitionedValue {
    pub id: SemanticValueId,
    pub type_value: TypeValueId,
    pub policy: PolicyPair,
    pub source_value: SemanticValueId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum P1AssemblyFailure {
    ProducedValueCountMismatch { expected: usize, actual: usize },
    ProducedValueDoesNotMatchDemand { demand_index: usize },
}

/// Assemble the value entries produced for one transition elaboration.
///
/// Ordinary projection has already terminated before this function is
/// reachable, so no existing entries are implicitly combined with transition
/// results.
pub fn assemble_transition_results<P: Clone>(
    demands: &[PolicyTransitionDemand<P>],
    produced: &[TransitionedValue],
) -> Result<Vec<PolicyResultEntry<SemanticValueRef, P>>, P1AssemblyFailure> {
    if demands.len() != produced.len() {
        return Err(P1AssemblyFailure::ProducedValueCountMismatch {
            expected: demands.len(),
            actual: produced.len(),
        });
    }

    let mut results = Vec::with_capacity(produced.len());
    for (demand_index, (demand, value)) in demands.iter().zip(produced).enumerate() {
        if value.source_value != demand.request.source_value()
            || project_policy_query(demand.request.target_query(), &value.policy).is_none()
        {
            return Err(P1AssemblyFailure::ProducedValueDoesNotMatchDemand { demand_index });
        }
        results.push(PolicyResultEntry {
            value: Some(SemanticValueRef {
                id: value.id,
                type_value: value.type_value,
            }),
            value_policy: value.policy.value.clone(),
            pattern: demand.pattern.clone(),
            pattern_policy: value.policy.pattern.clone(),
        });
    }
    Ok(results)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyBridgeEffect {
    BuiltinValueCopy,
    UserCallable(SymbolId),
    IntrinsicStub(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyBridgeInvocationResult<I> {
    pub callable_id: I,
    pub value: TransitionedValue,
    pub effect: PolicyBridgeEffect,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyBridgeInvocationFailure<I> {
    pub selected_callable_id: I,
    pub message: String,
}

/// Evaluate one selected prototype body.
///
/// This API deliberately receives one `ResolvedPolicyBridge`, not the
/// candidate family. A lowering/body failure therefore cannot reopen overload
/// selection or choose a former second-place candidate. It is not the final
/// ordinary invocation route.
pub fn invoke_resolved_policy_bridge<I: Clone>(
    selected: &ResolvedPolicyBridge<I>,
    request: &PolicyTransitionRequest,
    result_value: SemanticValueId,
) -> Result<PolicyBridgeInvocationResult<I>, PolicyBridgeInvocationFailure<I>> {
    let effect = match &selected.callable.body {
        PolicyBridgeBody::BuiltinValueCopy => PolicyBridgeEffect::BuiltinValueCopy,
        PolicyBridgeBody::UserCallable(symbol) => PolicyBridgeEffect::UserCallable(*symbol),
        PolicyBridgeBody::IntrinsicStub(name) => PolicyBridgeEffect::IntrinsicStub(name.clone()),
        PolicyBridgeBody::FailAfterSelection(message) => {
            return Err(PolicyBridgeInvocationFailure {
                selected_callable_id: selected.callable.id.clone(),
                message: message.clone(),
            });
        }
    };

    Ok(PolicyBridgeInvocationResult {
        callable_id: selected.callable.id.clone(),
        value: TransitionedValue {
            id: result_value,
            type_value: selected.result_type,
            policy: selected.result_policy.clone(),
            source_value: request.source_value(),
        },
        effect,
    })
}

fn candidate_is_fully_admissible<I>(
    request: &PolicyTransitionRequest,
    candidate: &PolicyTransitionCallable<I>,
    expectation: TransitionTypeExpectation,
) -> bool {
    if !candidate.ordinary_fully_admissible
        || !input_type_accepts(candidate.input_type, request.source_type())
        || project_policy_query(&candidate.input_policy, request.source_policy()).is_none()
        || project_policy_query(request.target_query(), &candidate.output_policy).is_none()
    {
        return false;
    }
    let result_type = candidate.output_type.resolve(request.source_type());
    expectation
        .required_output_type
        .map(|required| required == result_type)
        .unwrap_or(true)
}

fn input_type_accepts(pattern: OrdinaryCallableTypeInput, actual: TypeValueId) -> bool {
    match pattern {
        OrdinaryCallableTypeInput::Any => true,
        OrdinaryCallableTypeInput::Exact(expected) => expected == actual,
    }
}

fn candidate_dominates<I>(
    request: &PolicyTransitionRequest,
    better: &PolicyTransitionCallable<I>,
    worse: &PolicyTransitionCallable<I>,
) -> bool {
    let type_order = compare_input_type_fit(better.input_type, worse.input_type);
    let policy_order = compare_policy_transition_candidates(
        request.source_policy(),
        request.target_query(),
        better,
        worse,
    );
    matches!(
        compose_orders([type_order, policy_order]),
        PolicyPartialOrdering::Greater
    )
}

fn compare_input_type_fit(
    left: OrdinaryCallableTypeInput,
    right: OrdinaryCallableTypeInput,
) -> PolicyPartialOrdering {
    match (left, right) {
        (OrdinaryCallableTypeInput::Exact(a), OrdinaryCallableTypeInput::Exact(b)) if a == b => {
            PolicyPartialOrdering::Equal
        }
        (OrdinaryCallableTypeInput::Exact(_), OrdinaryCallableTypeInput::Any) => {
            PolicyPartialOrdering::Greater
        }
        (OrdinaryCallableTypeInput::Any, OrdinaryCallableTypeInput::Exact(_)) => {
            PolicyPartialOrdering::Less
        }
        (OrdinaryCallableTypeInput::Any, OrdinaryCallableTypeInput::Any) => {
            PolicyPartialOrdering::Equal
        }
        (OrdinaryCallableTypeInput::Exact(_), OrdinaryCallableTypeInput::Exact(_)) => {
            PolicyPartialOrdering::Incomparable
        }
    }
}

fn compare_input_policy_fit(
    required: &PolicyPair,
    left: &PolicyPair,
    right: &PolicyPair,
) -> PolicyPartialOrdering {
    if project_policy_query(left, required).is_none()
        || project_policy_query(right, required).is_none()
    {
        return PolicyPartialOrdering::Incomparable;
    }
    compare_policy_domain_specificity(left, right)
}

fn compare_output_policy_fit(
    required: &PolicyPair,
    left: &PolicyPair,
    right: &PolicyPair,
) -> PolicyPartialOrdering {
    if project_policy_query(required, left).is_none()
        || project_policy_query(required, right).is_none()
    {
        return PolicyPartialOrdering::Incomparable;
    }

    compare_policy_domain_specificity(left, right)
}

fn compare_policy_domain_specificity(
    left: &PolicyPair,
    right: &PolicyPair,
) -> PolicyPartialOrdering {
    compose_orders([
        compare_stage_domains(&left.value.stages, &right.value.stages),
        compare_mutability_domains(&left.value.mutability, &right.value.mutability),
        compare_presence_domains(left.value.presence, right.value.presence),
        compare_stage_domains(&left.pattern.stages, &right.pattern.stages),
        compare_exact_dimension(left.namespace_visibility, right.namespace_visibility),
        compare_exact_dimension(left.export_root, right.export_root),
    ])
}

fn compare_stage_domains(left: &StageSet, right: &StageSet) -> PolicyPartialOrdering {
    compare_subsets(left.is_subset(right), right.is_subset(left))
}

fn mutability_domain(mutability: &BTreeSet<ValueMutability>) -> BTreeSet<ValueMutability> {
    if mutability.is_empty() {
        BTreeSet::from([ValueMutability::Const, ValueMutability::Mut])
    } else {
        mutability.clone()
    }
}

fn compare_mutability_domains(
    left: &BTreeSet<ValueMutability>,
    right: &BTreeSet<ValueMutability>,
) -> PolicyPartialOrdering {
    let left = mutability_domain(left);
    let right = mutability_domain(right);
    compare_subsets(left.is_subset(&right), right.is_subset(&left))
}

fn presence_domain(presence: ValuePresence) -> BTreeSet<bool> {
    match presence {
        ValuePresence::Present => BTreeSet::from([true]),
        ValuePresence::Absent => BTreeSet::from([false]),
        ValuePresence::Optional => BTreeSet::from([false, true]),
    }
}

fn compare_presence_domains(left: ValuePresence, right: ValuePresence) -> PolicyPartialOrdering {
    let left = presence_domain(left);
    let right = presence_domain(right);
    compare_subsets(left.is_subset(&right), right.is_subset(&left))
}

fn compare_exact_dimension<T: PartialEq>(left: T, right: T) -> PolicyPartialOrdering {
    if left == right {
        PolicyPartialOrdering::Equal
    } else {
        PolicyPartialOrdering::Incomparable
    }
}

fn compare_subsets(left_in_right: bool, right_in_left: bool) -> PolicyPartialOrdering {
    match (left_in_right, right_in_left) {
        (true, true) => PolicyPartialOrdering::Equal,
        (true, false) => PolicyPartialOrdering::Greater,
        (false, true) => PolicyPartialOrdering::Less,
        (false, false) => PolicyPartialOrdering::Incomparable,
    }
}

fn compose_orders<const N: usize>(dimensions: [PolicyPartialOrdering; N]) -> PolicyPartialOrdering {
    let mut saw_less = false;
    let mut saw_greater = false;
    for dimension in dimensions {
        match dimension {
            PolicyPartialOrdering::Less => saw_less = true,
            PolicyPartialOrdering::Greater => saw_greater = true,
            PolicyPartialOrdering::Equal => {}
            PolicyPartialOrdering::Incomparable => return PolicyPartialOrdering::Incomparable,
        }
        if saw_less && saw_greater {
            return PolicyPartialOrdering::Incomparable;
        }
    }
    match (saw_less, saw_greater) {
        (true, false) => PolicyPartialOrdering::Less,
        (false, true) => PolicyPartialOrdering::Greater,
        (false, false) => PolicyPartialOrdering::Equal,
        (true, true) => PolicyPartialOrdering::Incomparable,
    }
}
