//! Existing-view-first, direct same-Type Policy migration.
//!
//! A total consumer demand first attempts ordinary view projection.  If no
//! existing view satisfies it, exactly one authorized ordinary migration
//! family may be enumerated and selected by the shared invocation pipeline.
//! The request is same-Type and direct: it enumerates one authorized family,
//! selects once, and never retries.

use std::{collections::BTreeSet, convert::Infallible};

use crate::{
    identity::{SemanticValueId, TypeValueId},
    model::Provenance,
    policy_pair::{
        project_p1, P1Projection, PatternComponentPolicy, PolicyPair, PolicyResultEntry,
        PolicyView, ResultPolicyDemand, StageSet, ValuePresence,
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

/// Result of applying the binding's pair-shaped P1 query to an existing result.
/// Migration is deliberately not represented here; it is requested only by
/// the caller after this projection fails.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct P1Elaboration<P> {
    pub origin: P1Origin,
    pub requested: Option<P1Projection>,
    pub selected: Vec<PolicyResultEntry<SemanticValueRef, P>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PureTypeP1Elaboration<P> {
    pub origin: P1Origin,
    pub requested: Option<P1Projection>,
    pub selected: Vec<PolicyResultEntry<Infallible, P>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum P1ElaborationFailure {
    EmptyResult,
    ProjectionUnavailable { requested: P1Projection },
    ProjectionUnavailableWithoutValue { requested: P1Projection },
}

/// One direct, same-Type Policy migration request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyMigrationRequest {
    source_view: PolicyView,
    target_demand: ResultPolicyDemand,
    source_type: TypeValueId,
    source_value: SemanticValueId,
    provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyMigrationRequestFailure {
    SourceValueAbsent,
    SourceValueStageDomainEmpty,
    TargetValueAbsent,
    TargetValueStageDomainEmpty,
    TargetPatternPolicyUnavailable {
        source: PatternComponentPolicy,
        target: PatternComponentPolicy,
    },
}

impl PolicyMigrationRequest {
    pub fn new(
        source_view: PolicyView,
        target_demand: ResultPolicyDemand,
        source_type: TypeValueId,
        source_value: SemanticValueId,
        provenance: Provenance,
    ) -> Result<Self, PolicyMigrationRequestFailure> {
        let target_pair = concrete_target_pair(&target_demand)
            .ok_or(PolicyMigrationRequestFailure::TargetValueStageDomainEmpty)?;
        if source_view.pair.value.presence == ValuePresence::Absent {
            return Err(PolicyMigrationRequestFailure::SourceValueAbsent);
        }
        if source_view.pair.value.stages.is_empty() {
            return Err(PolicyMigrationRequestFailure::SourceValueStageDomainEmpty);
        }
        if target_pair.value.presence == ValuePresence::Absent {
            return Err(PolicyMigrationRequestFailure::TargetValueAbsent);
        }
        if target_pair.value.stages.is_empty() {
            return Err(PolicyMigrationRequestFailure::TargetValueStageDomainEmpty);
        }
        if target_pair.pattern.stages.is_empty()
            || !target_pair
                .pattern
                .stages
                .is_subset(&source_view.pair.pattern.stages)
        {
            return Err(
                PolicyMigrationRequestFailure::TargetPatternPolicyUnavailable {
                    source: source_view.pair.pattern.clone(),
                    target: target_pair.pattern.clone(),
                },
            );
        }
        Ok(Self {
            source_view,
            target_demand,
            source_type,
            source_value,
            provenance,
        })
    }

    pub fn source_view(&self) -> &PolicyView {
        &self.source_view
    }

    pub fn source_policy(&self) -> &PolicyPair {
        &self.source_view.pair
    }

    pub fn target_demand(&self) -> &ResultPolicyDemand {
        &self.target_demand
    }

    pub fn target_pair(&self) -> &PolicyPair {
        concrete_target_pair(&self.target_demand)
            .expect("validated migration requests carry a concrete pair query")
    }

    pub fn target_query(&self) -> &PolicyPair {
        self.target_pair()
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

fn concrete_target_pair(demand: &ResultPolicyDemand) -> Option<&PolicyPair> {
    match &demand.pair_query {
        P1Projection::Pair(pair) => Some(pair),
        P1Projection::Infer | P1Projection::ValueDominant { .. } => None,
    }
}

/// Apply a binding's pair-shaped demand to already-produced result entries.
/// If this returns `ProjectionUnavailable`, the caller may form one ordinary
/// `PolicyMigrationRequest`; this function never manufactures a special edge.
pub fn elaborate_value_binding_p1<P: Clone>(
    result: &[PolicyResultEntry<SemanticValueRef, P>],
    explicit_p1: Option<&P1Projection>,
    _provenance: Provenance,
) -> Result<P1Elaboration<P>, P1ElaborationFailure> {
    if result.is_empty() {
        return Err(P1ElaborationFailure::EmptyResult);
    }
    let Some(projection) = explicit_p1 else {
        return Ok(P1Elaboration {
            origin: P1Origin::Inferred,
            requested: None,
            selected: result.to_vec(),
        });
    };
    let selected = project_p1(projection, result);
    if !selected.is_empty() {
        return Ok(P1Elaboration {
            origin: P1Origin::Explicit,
            requested: Some(projection.clone()),
            selected,
        });
    }
    if result.iter().all(|entry| entry.value.is_none()) {
        let relaxed = relax_projection_presence(projection);
        let selected = project_p1(&relaxed, result);
        if !selected.is_empty() {
            return Ok(P1Elaboration {
                origin: P1Origin::Explicit,
                requested: Some(projection.clone()),
                selected,
            });
        }
        return Err(P1ElaborationFailure::ProjectionUnavailableWithoutValue {
            requested: projection.clone(),
        });
    }
    Err(P1ElaborationFailure::ProjectionUnavailable {
        requested: projection.clone(),
    })
}

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

fn relax_projection_presence(projection: &P1Projection) -> P1Projection {
    let mut relaxed = projection.clone();
    match &mut relaxed {
        P1Projection::Infer => {}
        P1Projection::ValueDominant { value } => value.presence = ValuePresence::Optional,
        P1Projection::Pair(pair) => pair.value.presence = ValuePresence::Optional,
    }
    relaxed
}

pub(crate) fn project_migration_input_endpoint(
    candidate: &PolicyPair,
    actual: &PolicyPair,
) -> Option<PolicyPair> {
    project_migration_endpoint_hard_coordinates(candidate, actual)
}

pub(crate) fn project_migration_output_endpoint(
    required: &PolicyPair,
    candidate: &PolicyPair,
) -> Option<PolicyPair> {
    project_migration_endpoint_hard_coordinates(required, candidate)
}

fn project_migration_endpoint_hard_coordinates(
    query: &PolicyPair,
    available: &PolicyPair,
) -> Option<PolicyPair> {
    let presence = intersect_presence(query.value.presence, available.value.presence)?;
    let value_stages = if presence == ValuePresence::Absent {
        StageSet::new()
    } else {
        project_non_empty_stages(&query.value.stages, &available.value.stages)?
    };
    let pattern_stages =
        project_non_empty_stages(&query.pattern.stages, &available.pattern.stages)?;
    Some(PolicyPair {
        value: crate::policy_pair::ValueComponentPolicy {
            stages: value_stages,
            presence,
        },
        pattern: PatternComponentPolicy {
            stages: pattern_stages,
        },
    })
}

fn intersect_presence(query: ValuePresence, available: ValuePresence) -> Option<ValuePresence> {
    match (query, available) {
        (ValuePresence::Optional, selected) | (selected, ValuePresence::Optional) => Some(selected),
        (ValuePresence::Present, ValuePresence::Present) => Some(ValuePresence::Present),
        (ValuePresence::Absent, ValuePresence::Absent) => Some(ValuePresence::Absent),
        (ValuePresence::Present, ValuePresence::Absent)
        | (ValuePresence::Absent, ValuePresence::Present) => None,
    }
}

fn project_non_empty_stages(query: &StageSet, available: &StageSet) -> Option<StageSet> {
    let selected = if query.is_empty() {
        available.clone()
    } else {
        query.intersection(available)
    };
    (!selected.is_empty()).then_some(selected)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyPartialOrdering {
    Less,
    Equal,
    Greater,
    Incomparable,
}

pub(crate) fn compare_migration_endpoint_coordinates(
    required_source: &PolicyPair,
    required_target: &PolicyPair,
    left_input: &PolicyPair,
    left_output: &PolicyPair,
    right_input: &PolicyPair,
    right_output: &PolicyPair,
) -> PolicyPartialOrdering {
    compose_orders([
        compare_input_policy_fit(required_source, left_input, right_input),
        compare_output_policy_fit(required_target, left_output, right_output),
    ])
}

fn compare_input_policy_fit(
    required: &PolicyPair,
    left: &PolicyPair,
    right: &PolicyPair,
) -> PolicyPartialOrdering {
    if project_migration_input_endpoint(left, required).is_none()
        || project_migration_input_endpoint(right, required).is_none()
    {
        return PolicyPartialOrdering::Incomparable;
    }
    compare_policy_endpoint_fit(left, right)
}

fn compare_output_policy_fit(
    required: &PolicyPair,
    left: &PolicyPair,
    right: &PolicyPair,
) -> PolicyPartialOrdering {
    if project_migration_output_endpoint(required, left).is_none()
        || project_migration_output_endpoint(required, right).is_none()
    {
        return PolicyPartialOrdering::Incomparable;
    }
    compare_policy_endpoint_fit(left, right)
}

fn compare_policy_endpoint_fit(left: &PolicyPair, right: &PolicyPair) -> PolicyPartialOrdering {
    compose_orders([
        compare_stage_domains(&left.value.stages, &right.value.stages),
        compare_presence_domains(left.value.presence, right.value.presence),
        compare_stage_domains(&left.pattern.stages, &right.pattern.stages),
    ])
}

fn compare_stage_domains(left: &StageSet, right: &StageSet) -> PolicyPartialOrdering {
    compare_subsets(left.is_subset(right), right.is_subset(left))
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
