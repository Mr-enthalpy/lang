//! Cross-policy value-transition substrate.
//!
//! This module contains a transition-demand model plus a transitional
//! candidate-algebra prototype. The prototype shares the ordinary
//! maximal-element rule, but it does not yet resolve a distinguished global
//! Symbol, enumerate heterogeneous Val2, construct an `InvocationFrame`, or
//! invoke the repository's ordinary function-object pipeline. It must not
//! become a second permanent callable representation.

use std::{collections::BTreeSet, convert::Infallible};

use crate::{
    identity::{SemanticValueId, TypeValueId},
    model::{Provenance, SymbolId},
    policy_overload::maximal_candidates,
    policy_pair::{
        project_p1, P1Projection, PatternComponentPolicy, PolicyPair, PolicyResultEntry,
        PolicyStage, StageSet, ValueComponentPolicy, ValueMutability, ValuePresence,
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
/// Existing slices and missing value demands are both collections because one
/// explicit P1 can retain identity-preserving slices and request newly
/// materialized slices at the same time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct P1Elaboration<P> {
    pub origin: P1Origin,
    pub requested: Option<PolicyPair>,
    pub existing_slices: Vec<PolicyResultEntry<SemanticValueRef, P>>,
    pub missing_value_demands: Vec<PolicyTransitionDemand<P>>,
}

/// Projection-only P1 elaboration for a pure type/Pattern result.
///
/// `Infallible` makes `Some(value)` unconstructible and this carrier exposes no
/// transition field. A pure type therefore cannot enter transition machinery.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PureTypeP1Elaboration<P> {
    pub origin: P1Origin,
    pub requested: Option<PolicyPair>,
    pub existing_slices: Vec<PolicyResultEntry<Infallible, P>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum P1ElaborationFailure {
    EmptyResult,
    ValueBearingInputContainsAbsentValue,
    RequestedPatternSliceUnavailable {
        requested: StageSet,
        available: StageSet,
    },
    RequestedValueSliceUnavailable {
        requested: ValueComponentPolicy,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyTransitionRequest {
    pub source_policy: PolicyPair,
    pub target_policy: PolicyPair,
    pub source_type: TypeValueId,
    pub source_value: SemanticValueId,
    pub provenance: Provenance,
}

/// Derive the natural P1 pair of a value-bearing P2 result.
///
/// This is the canonical stage lift `(P2v || P2p):P2p`. Other typed
/// dimensions remain those of P2. An absent value stays absent and therefore
/// cannot acquire a value-stage domain merely from its Pattern component.
pub fn default_p1(p2: &PolicyPair) -> PolicyPair {
    let value_stages = if p2.value.presence == ValuePresence::Absent {
        StageSet::new()
    } else {
        p2.value.stages.union(&p2.pattern.stages)
    };
    PolicyPair {
        value: ValueComponentPolicy {
            stages: value_stages,
            mutability: p2.value.mutability.clone(),
            presence: p2.value.presence,
        },
        pattern: p2.pattern.clone(),
        namespace_visibility: p2.namespace_visibility,
        export_root: p2.export_root,
    }
}

/// Elaborate P1 over value-bearing RHS result entries.
///
/// Existing views are selected through the same `project_p1` operation used by
/// the older multi-entry path. Missing Val1 stages become independent direct
/// demands; the full explicit target is never sent to a runtime-only
/// transition validator.
pub fn elaborate_value_binding_p1<P: Clone>(
    result: &[PolicyResultEntry<SemanticValueRef, P>],
    explicit_p1: Option<&PolicyPair>,
    provenance: Provenance,
) -> Result<P1Elaboration<P>, P1ElaborationFailure> {
    if result.is_empty() {
        return Err(P1ElaborationFailure::EmptyResult);
    }
    if result.iter().any(|entry| entry.value.is_none()) {
        return Err(P1ElaborationFailure::ValueBearingInputContainsAbsentValue);
    }

    let available = result.iter().map(default_result_entry).collect::<Vec<_>>();
    let Some(target_policy) = explicit_p1 else {
        return Ok(P1Elaboration {
            origin: P1Origin::Inferred,
            requested: None,
            existing_slices: available,
            missing_value_demands: Vec::new(),
        });
    };

    validate_requested_pattern_slice(&available, target_policy)?;
    let projection = P1Projection::Pair(target_policy.clone());
    let existing_slices = project_p1(&projection, &available);
    let mut missing_value_demands = Vec::new();

    for entry in &available {
        let source = entry
            .value
            .expect("value-bearing input was checked before elaboration");
        let selected_pattern_stages = target_policy
            .pattern
            .stages
            .intersection(&entry.pattern_policy.stages);
        if selected_pattern_stages.is_empty() {
            continue;
        }

        for stage in target_policy.value.stages.iter() {
            let mut requested_slice = target_policy.clone();
            requested_slice.value.stages = StageSet::from([stage]);
            requested_slice.pattern.stages = selected_pattern_stages.clone();
            let already_available = !project_p1(
                &P1Projection::Pair(requested_slice.clone()),
                std::slice::from_ref(entry),
            )
            .is_empty();
            if already_available {
                continue;
            }
            missing_value_demands.push(PolicyTransitionDemand {
                request: PolicyTransitionRequest {
                    source_policy: policy_pair_from_entry(entry),
                    target_policy: requested_slice,
                    source_type: source.type_value,
                    source_value: source.id,
                    provenance: provenance.clone(),
                },
                pattern: entry.pattern.clone(),
            });
        }
    }

    if existing_slices.is_empty() && missing_value_demands.is_empty() {
        return Err(P1ElaborationFailure::RequestedValueSliceUnavailable {
            requested: target_policy.value.clone(),
        });
    }

    Ok(P1Elaboration {
        origin: P1Origin::Explicit,
        requested: Some(target_policy.clone()),
        existing_slices,
        missing_value_demands,
    })
}

/// Elaborate P1 over a pure type/Pattern result.
///
/// This API requires `PolicyResultEntry<Infallible, P>` and returns a carrier
/// with no transition demand field. An unavailable Pattern slice is a
/// projection failure.
pub fn elaborate_pure_type_binding_p1<P: Clone>(
    result: &[PolicyResultEntry<Infallible, P>],
    explicit_p1: Option<&PolicyPair>,
) -> Result<PureTypeP1Elaboration<P>, P1ElaborationFailure> {
    if result.is_empty() {
        return Err(P1ElaborationFailure::EmptyResult);
    }
    let available = result.iter().map(default_result_entry).collect::<Vec<_>>();
    let Some(target_policy) = explicit_p1 else {
        return Ok(PureTypeP1Elaboration {
            origin: P1Origin::Inferred,
            requested: None,
            existing_slices: available,
        });
    };

    validate_requested_pattern_slice(&available, target_policy)?;
    let existing_slices = project_p1(&P1Projection::Pair(target_policy.clone()), &available);
    if existing_slices.is_empty() {
        return Err(P1ElaborationFailure::RequestedValueSliceUnavailable {
            requested: target_policy.value.clone(),
        });
    }
    Ok(PureTypeP1Elaboration {
        origin: P1Origin::Explicit,
        requested: Some(target_policy.clone()),
        existing_slices,
    })
}

fn default_result_entry<V: Clone, P: Clone>(
    entry: &PolicyResultEntry<V, P>,
) -> PolicyResultEntry<V, P> {
    let pair = default_p1(&policy_pair_from_entry(entry));
    PolicyResultEntry {
        value: entry.value.clone(),
        value_policy: pair.value,
        pattern: entry.pattern.clone(),
        pattern_policy: pair.pattern,
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

fn validate_requested_pattern_slice<V, P>(
    result: &[PolicyResultEntry<V, P>],
    target: &PolicyPair,
) -> Result<(), P1ElaborationFailure> {
    let available = result.iter().fold(StageSet::new(), |stages, entry| {
        stages.union(&entry.pattern_policy.stages)
    });
    if target.pattern.stages.is_subset(&available) {
        Ok(())
    } else {
        Err(P1ElaborationFailure::RequestedPatternSliceUnavailable {
            requested: target.pattern.stages.clone(),
            available,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyTransitionFailure {
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
    required_target: &PolicyPair,
    a: &PolicyTransitionCallable<I>,
    b: &PolicyTransitionCallable<I>,
) -> PolicyPartialOrdering {
    compose_orders([
        compare_input_policy_fit(required_source, &a.input_policy, &b.input_policy),
        compare_output_policy_fit(required_target, &a.output_policy, &b.output_policy),
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
            result_type: candidate.output_type.resolve(request.source_type),
            result_policy: request.target_policy.clone(),
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

/// Combine identity-preserving existing views with newly produced transition
/// values. Produced values are positional results of
/// `missing_value_demands`; they never replace the retained entries.
pub fn assemble_value_binding_slices<P: Clone>(
    elaboration: &P1Elaboration<P>,
    produced: &[TransitionedValue],
) -> Result<Vec<PolicyResultEntry<SemanticValueRef, P>>, P1AssemblyFailure> {
    if elaboration.missing_value_demands.len() != produced.len() {
        return Err(P1AssemblyFailure::ProducedValueCountMismatch {
            expected: elaboration.missing_value_demands.len(),
            actual: produced.len(),
        });
    }

    let mut combined = elaboration.existing_slices.clone();
    for (demand_index, (demand, value)) in elaboration
        .missing_value_demands
        .iter()
        .zip(produced)
        .enumerate()
    {
        if value.source_value != demand.request.source_value
            || value.policy != demand.request.target_policy
        {
            return Err(P1AssemblyFailure::ProducedValueDoesNotMatchDemand { demand_index });
        }
        combined.push(PolicyResultEntry {
            value: Some(SemanticValueRef {
                id: value.id,
                type_value: value.type_value,
            }),
            value_policy: value.policy.value.clone(),
            pattern: demand.pattern.clone(),
            pattern_policy: value.policy.pattern.clone(),
        });
    }
    Ok(combined)
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
            source_value: request.source_value,
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
        || !input_type_accepts(candidate.input_type, request.source_type)
        || !policy_domains_overlap(&candidate.input_policy, &request.source_policy)
        || !policy_covers(&candidate.output_policy, &request.target_policy)
    {
        return false;
    }
    let result_type = candidate.output_type.resolve(request.source_type);
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
        &request.source_policy,
        &request.target_policy,
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
    if !policy_domains_overlap(left, required) || !policy_domains_overlap(right, required) {
        return PolicyPartialOrdering::Incomparable;
    }
    compare_policy_domain_specificity(left, right)
}

fn compare_output_policy_fit(
    required: &PolicyPair,
    left: &PolicyPair,
    right: &PolicyPair,
) -> PolicyPartialOrdering {
    if !policy_covers(left, required) || !policy_covers(right, required) {
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

fn policy_covers(candidate: &PolicyPair, required: &PolicyPair) -> bool {
    required.value.stages.is_subset(&candidate.value.stages)
        && mutability_domain(&required.value.mutability)
            .is_subset(&mutability_domain(&candidate.value.mutability))
        && presence_domain(required.value.presence)
            .is_subset(&presence_domain(candidate.value.presence))
        && required.pattern.stages.is_subset(&candidate.pattern.stages)
        && candidate.namespace_visibility == required.namespace_visibility
        && candidate.export_root == required.export_root
}

/// Whether a callable input can consume at least one identity-preserving slice
/// of the source. This is deliberately not the output rule: an output must
/// cover the complete requested target, while an input may first select an
/// existing source slice.
fn policy_domains_overlap(left: &PolicyPair, right: &PolicyPair) -> bool {
    let left_mutability = mutability_domain(&left.value.mutability);
    let right_mutability = mutability_domain(&right.value.mutability);
    let left_presence = presence_domain(left.value.presence);
    let right_presence = presence_domain(right.value.presence);
    !left
        .value
        .stages
        .intersection(&right.value.stages)
        .is_empty()
        && left_mutability
            .intersection(&right_mutability)
            .next()
            .is_some()
        && left_presence.intersection(&right_presence).next().is_some()
        && !left
            .pattern
            .stages
            .intersection(&right.pattern.stages)
            .is_empty()
        && left.namespace_visibility == right.namespace_visibility
        && left.export_root == right.export_root
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
