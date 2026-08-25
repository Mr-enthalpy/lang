//! Existing-view-first same-Type Policy-migration substrate.
//!
//! Ordinary Policy projection always runs first.  An unmet, already-total
//! target demand may prepare one direct authorized migration family.  The
//! migration is selected by the ordinary overload pipeline; it never performs
//! graph search, transitive chaining, or Type-changing conversion.  The
//! bounded compile-to-runtime request remains only a compatibility constructor
//! and the first connected family implementation.
//!
//! The candidate/result carriers below remain prototype algebra fixtures. The
//! connected path lives in `semantic_world` + `ordinary_invocation`: it
//! enumerates ordinary semantic values, follows TypeValue to Pattern owner and
//! associated `()`, constructs an `InvocationFrame`, and returns ordinary
//! result entries. These fixtures intentionally do not delegate to or compete
//! with that resolver, and establish no final result Pattern coherence.
//!
//! Ordinary binding semantics stay in `policy_pair::project_p1`: omitted P1
//! preserves the complete RHS and any non-empty explicit projection completes
//! binding elaboration. This module prepares a transition request only after an
//! explicit query projects no entry.

use std::{collections::BTreeSet, convert::Infallible};

use crate::{
    identity::{SemanticValueId, TypeValueId},
    model::{Provenance, SymbolId},
    policy_overload::{maximal_candidates, mutability_preference_rank},
    policy_pair::{
        project_p1, P1Projection, PatternComponentPolicy, PolicyMode, PolicyPair,
        PolicyResultEntry, PolicyStage, PolicyView, ResultPolicyDemand, StageSet, ValuePresence,
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
pub struct PolicyTransitionDemand {
    pub request: PolicyTransitionRequest,
}

/// Conservative P1 elaboration over an arbitrary multi-entry result.
///
/// Ordinary P1 remains a projection query. A non-empty projection completes
/// binding elaboration. Transition preparation is considered only when the
/// complete query projects no existing entry, and then only value-bearing
/// entries whose Pattern side can satisfy the query may form demands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum P1Elaboration<P> {
    Projected {
        origin: P1Origin,
        requested: Option<P1Projection>,
        selected: Vec<PolicyResultEntry<SemanticValueRef, P>>,
    },
    AtomicRuntimeMigration {
        requested: P1Projection,
        demands: Vec<PolicyTransitionDemand>,
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
    ProjectionUnavailableWithoutValue {
        requested: P1Projection,
    },
    PatternPolicyStageSliceUnavailableForMigration {
        requested: P1Projection,
    },
    ProjectionUnavailableOutsideAtomicRuntimeMigration {
        requested: P1Projection,
    },
    InvalidTransitionSource {
        entry_index: usize,
        failure: PolicyTransitionRequestFailure,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// One checked atomic-migration request after demand decomposition.
///
/// `source_policy` is the selected pure-static `Project_in` endpoint, not the
/// complete source result. `target_query` is the runtime-only branch extracted
/// from the complete consumer query after that complete query projected no
/// existing view.
pub struct PolicyTransitionRequest {
    source_view: PolicyView,
    target_demand: ResultPolicyDemand,
    source_type: TypeValueId,
    source_value: SemanticValueId,
    provenance: Provenance,
}

/// Consumer-neutral request for one direct same-Type Policy migration.
///
/// This carrier is the production semantic authority.  It does not encode a
/// compile-to-runtime edge, perform graph search, or permit a result Type
/// change.  Legacy `PolicyTransitionRequest` values are one bounded way to
/// construct it while old binding P1 callers migrate.
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

    pub fn from_atomic_runtime(request: &PolicyTransitionRequest) -> Self {
        Self {
            source_view: request.source_view.clone(),
            target_demand: request.target_demand.clone(),
            source_type: request.source_type,
            source_value: request.source_value,
            provenance: request.provenance.clone(),
        }
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyTransitionRequestFailure {
    SourceValueAbsent,
    SourceStaticValueStageDomainEmpty,
    SelectedInputContainsRuntime,
    SourceStaticPatternStageMismatch {
        source_static: StageSet,
        source_pattern: StageSet,
    },
    TargetDoesNotRequireRuntimeOnly {
        target_value_stages: StageSet,
        target_value_presence: ValuePresence,
    },
    TargetPatternPolicyUnavailable {
        source: PatternComponentPolicy,
        target: PatternComponentPolicy,
    },
}

impl PolicyTransitionRequest {
    pub fn new(
        source_view: PolicyView,
        target_demand: ResultPolicyDemand,
        source_type: TypeValueId,
        source_value: SemanticValueId,
        provenance: Provenance,
    ) -> Result<Self, PolicyTransitionRequestFailure> {
        let target_query = concrete_target_pair(&target_demand)
            .expect("atomic runtime transition requires a concrete pair query");
        let source_policy = &source_view.pair;
        if source_policy.value.presence == ValuePresence::Absent {
            return Err(PolicyTransitionRequestFailure::SourceValueAbsent);
        }
        if source_policy.value.stages.contains(PolicyStage::Runtime) {
            return Err(PolicyTransitionRequestFailure::SelectedInputContainsRuntime);
        }
        let source_static = source_policy.value.stages.static_stages();
        if source_static.is_empty() {
            return Err(PolicyTransitionRequestFailure::SourceStaticValueStageDomainEmpty);
        }
        if source_static != source_policy.pattern.stages {
            return Err(
                PolicyTransitionRequestFailure::SourceStaticPatternStageMismatch {
                    source_static,
                    source_pattern: source_policy.pattern.stages.clone(),
                },
            );
        }
        let target_is_runtime_only = target_query.value.presence == ValuePresence::Present
            && target_query.value.stages.len() == 1
            && target_query.value.stages.contains(PolicyStage::Runtime);
        if !target_is_runtime_only {
            return Err(
                PolicyTransitionRequestFailure::TargetDoesNotRequireRuntimeOnly {
                    target_value_stages: target_query.value.stages.clone(),
                    target_value_presence: target_query.value.presence,
                },
            );
        }
        if target_query.pattern.stages.is_empty()
            || !target_query
                .pattern
                .stages
                .is_subset(&source_policy.pattern.stages)
        {
            return Err(
                PolicyTransitionRequestFailure::TargetPatternPolicyUnavailable {
                    source: source_policy.pattern.clone(),
                    target: target_query.pattern.clone(),
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
            .expect("validated atomic migration carries a concrete pair query")
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

/// Elaborate P1 over arbitrary RHS result entries.
///
/// Omitted P1 preserves the complete RHS entries exactly. An explicit P1 first
/// runs the canonical `project_p1` query over the complete result. Transition
/// preparation occurs only when that projection is empty and the complete
/// query accepts a runtime value branch. The derived request target is the
/// runtime-only branch; other query alternatives are not manufactured. Absent
/// entries are not an error: they simply cannot form an atomic migration
/// demand.
pub fn elaborate_value_binding_p1<P: Clone>(
    result: &[PolicyResultEntry<SemanticValueRef, P>],
    explicit_p1: Option<&P1Projection>,
    provenance: Provenance,
) -> Result<P1Elaboration<P>, P1ElaborationFailure> {
    if result.is_empty() {
        return Err(P1ElaborationFailure::EmptyResult);
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

    let mut saw_value_bearing_entry = false;
    let mut saw_runtime_migration_shape = false;
    let mut demands = Vec::with_capacity(result.len());
    for (entry_index, entry) in result.iter().enumerate() {
        let Some(source) = entry.value else {
            continue;
        };
        saw_value_bearing_entry = true;
        if projection_accepts_runtime_branch(projection) {
            saw_runtime_migration_shape = true;
        }
        let Some((source_view, target_demand)) =
            atomic_runtime_migration_endpoints(projection, entry)
        else {
            continue;
        };
        let request = PolicyTransitionRequest::new(
            source_view,
            target_demand,
            source.type_value,
            source.id,
            provenance.clone(),
        )
        .map_err(|failure| P1ElaborationFailure::InvalidTransitionSource {
            entry_index,
            failure,
        })?;
        demands.push(PolicyTransitionDemand { request });
    }

    if !saw_value_bearing_entry {
        // Every entry is pure-P (no Val1).  Pure types never enter transition
        // machinery; a stage P1 on such a binding (`compile let T = <type
        // result>;`) demands a visible stage slice, not a value identity, so
        // the value-presence requirement is relaxed to Optional before the
        // slice is retried.
        let relaxed = relax_projection_presence(projection);
        let selected = project_p1(&relaxed, result);
        if !selected.is_empty() {
            return Ok(P1Elaboration::Projected {
                origin: P1Origin::Explicit,
                requested: Some(projection.clone()),
                selected,
            });
        }
        return Err(P1ElaborationFailure::ProjectionUnavailableWithoutValue {
            requested: projection.clone(),
        });
    }
    if !saw_runtime_migration_shape {
        return Err(
            P1ElaborationFailure::ProjectionUnavailableOutsideAtomicRuntimeMigration {
                requested: projection.clone(),
            },
        );
    }
    if demands.is_empty() {
        return Err(
            P1ElaborationFailure::PatternPolicyStageSliceUnavailableForMigration {
                requested: projection.clone(),
            },
        );
    }

    Ok(P1Elaboration::AtomicRuntimeMigration {
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

/// Relax a P1 request's value-presence to `Optional` for pure-P results.
///
/// Stage and mutability constraints are preserved; only the presence gate is
/// dropped, because a pure-P result has no Val1 by construction.
fn relax_projection_presence(projection: &P1Projection) -> P1Projection {
    let mut relaxed = projection.clone();
    match &mut relaxed {
        P1Projection::Infer => {}
        P1Projection::ValueDominant { value } => value.presence = ValuePresence::Optional,
        P1Projection::Pair(pair) => pair.value.presence = ValuePresence::Optional,
    }
    relaxed
}

/// Derive `Project_in` and the runtime-only output branch for one binding
/// entry. The complete projection has already failed before this helper is
/// called. Returning `None` means either that the original demand accepts no
/// runtime value branch or that this entry cannot supply the required
/// Pattern-policy/static input slice.
fn atomic_runtime_migration_endpoints<V, P>(
    projection: &P1Projection,
    entry: &PolicyResultEntry<V, P>,
) -> Option<(PolicyView, ResultPolicyDemand)> {
    let accepted_value = match projection {
        P1Projection::Pair(pair) => pair.value.clone(),
        P1Projection::ValueDominant { value } => value.clone(),
        P1Projection::Infer => return None,
    };
    if accepted_value.presence == ValuePresence::Absent
        || !accepted_value.stages.contains(PolicyStage::Runtime)
    {
        return None;
    }
    let target_value = crate::policy_pair::ValueComponentPolicy {
        stages: StageSet::from([PolicyStage::Runtime]),
        presence: ValuePresence::Present,
    };

    let selected_pattern_stages = match projection {
        P1Projection::Pair(pair) => {
            if pair.pattern.stages.is_empty() {
                entry.view.pair.pattern.stages.clone()
            } else {
                let selected = pair
                    .pattern
                    .stages
                    .intersection(&entry.view.pair.pattern.stages);
                if selected.is_empty() {
                    return None;
                }
                selected
            }
        }
        P1Projection::ValueDominant { .. } => entry.view.pair.pattern.stages.clone(),
        P1Projection::Infer => return None,
    };

    let source_static = entry.view.pair.value.stages.static_stages();
    if selected_pattern_stages.is_empty()
        || !selected_pattern_stages.is_subset(&source_static)
        || source_static != entry.view.pair.pattern.stages
    {
        return None;
    }

    let selected_pattern = PatternComponentPolicy {
        stages: selected_pattern_stages.clone(),
    };
    let source_view = PolicyView {
        pair: PolicyPair {
            value: crate::policy_pair::ValueComponentPolicy {
                stages: selected_pattern_stages,
                presence: ValuePresence::Present,
            },
            pattern: selected_pattern.clone(),
        },
        mode: entry.view.mode,
    };
    let target_query = PolicyPair {
        value: target_value,
        pattern: selected_pattern,
    };
    Some((
        source_view,
        ResultPolicyDemand {
            pair_query: P1Projection::Pair(target_query),
            mode: entry.view.mode,
        },
    ))
}

fn projection_accepts_runtime_branch(projection: &P1Projection) -> bool {
    let value = match projection {
        P1Projection::Pair(pair) => &pair.value,
        P1Projection::ValueDominant { value } => value,
        P1Projection::Infer => return false,
    };
    value.presence != ValuePresence::Absent && value.stages.contains(PolicyStage::Runtime)
}

/// Intersect a pair-shaped transition query with one available Policy domain.
///
/// Unlike `project_p1`, this operates on policy domains rather than on one
/// concrete `PolicyResultEntry`. It therefore preserves present/optional/
/// absent alternatives without fabricating a `Some(value)` solely to borrow
/// the ordinary result projector. This general capability-projection helper is
/// not migration-candidate admissibility. Whole-slot mode is intentionally
/// absent and is compared only by the ordinary Bp product.
pub fn project_transition_policy_domain(
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

/// Project the hard pair-shaped coordinates of a migration input endpoint.
/// Whole-slot mode is consumed independently by ordinary Bp preference.
pub(crate) fn project_migration_input_endpoint(
    candidate: &PolicyPair,
    actual: &PolicyPair,
) -> Option<PolicyPair> {
    project_migration_endpoint_hard_coordinates(candidate, actual)
}

/// Project the hard pair-shaped coordinates of a migration output endpoint.
/// Whole-slot mode remains an independent concrete coordinate.
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyTransitionFailure {
    SourceValueAbsent,
    SelectedInputContainsRuntime,
    SourceStaticValueStageDomainEmpty,
    SourceStaticPatternStageMismatch {
        source_static: StageSet,
        source_pattern: StageSet,
    },
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

/// Validate one selected atomic runtime migration endpoint.
///
/// The input is an already selected static Policy view. The compiler-mandated
/// edge changes Val1 staging to runtime and keeps Pattern-policy stage
/// capability unchanged. Value mutability is deliberately not required to be
/// equal: input/output mutability belongs to the selected ordinary callable.
/// This is intentionally not a validator for arbitrary conversion or for
/// meta/compile/seal migration.
pub fn validate_runtime_transition(
    source: &PolicyPair,
    target: &PolicyPair,
) -> Result<(), PolicyTransitionFailure> {
    if source.value.presence == ValuePresence::Absent {
        return Err(PolicyTransitionFailure::SourceValueAbsent);
    }
    if source.value.stages.contains(PolicyStage::Runtime) {
        return Err(PolicyTransitionFailure::SelectedInputContainsRuntime);
    }
    let source_static = source.value.stages.static_stages();
    if source_static.is_empty() {
        return Err(PolicyTransitionFailure::SourceStaticValueStageDomainEmpty);
    }
    if source_static != source.pattern.stages {
        return Err(PolicyTransitionFailure::SourceStaticPatternStageMismatch {
            source_static,
            source_pattern: source.pattern.stages.clone(),
        });
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
    pub input_mode: PolicyMode,
    pub output_mode: PolicyMode,
    /// Hard conditions owned by ordinary candidate preparation (shape,
    /// require/concept checks, body availability, and similar facts).
    pub ordinary_fully_admissible: bool,
    /// Fixture-only marker for the future pre-Bp fallback strategy. Current
    /// source cannot construct this role; surface syntax and final ordinary
    /// candidate storage remain unfrozen.
    pub prototype_is_fallback: bool,
    /// Test-only stand-in for ordinary B3 extraction specificity. It is
    /// deliberately applied after the endpoint-Policy product fixture.
    pub prototype_pattern_specificity: u32,
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
    /// Complete ordinary result Policy declared by the selected callable.
    pub complete_result_view: PolicyView,
    /// Runtime-stage endpoint used by prototype validation and Bp comparison.
    /// Final ordinary `Project_out` still occurs after invocation and may fail
    /// without reopening selection.
    pub validated_output_endpoint: PolicyPair,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyBridgeResolution<I> {
    Selected(ResolvedPolicyBridge<I>),
    RejectedByDelete(I),
    Ambiguous(Vec<I>),
    NoCandidate,
}

/// Typed qualification consumed by an outer candidate's fully-admissible
/// check. Only `Available` admits the outer candidate; delete, ambiguity, and
/// absence retain distinct diagnostic causes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BridgeQualification<I> {
    Available(ResolvedPolicyBridge<I>),
    RejectedByDelete(I),
    Missing,
    Ambiguous(Vec<I>),
}

/// Compare only the input/output Policy dimensions of two transition
/// candidates. `Greater` means `a` dominates `b`.
///
/// Input and output are composed as one Pareto/product order. Crossed
/// advantages are therefore `Incomparable`.
pub fn compare_policy_transition_candidates<I>(
    required_source: &PolicyView,
    target_demand: &ResultPolicyDemand,
    a: &PolicyTransitionCallable<I>,
    b: &PolicyTransitionCallable<I>,
) -> PolicyPartialOrdering {
    let target_query = concrete_target_pair(target_demand)
        .expect("migration comparison requires a concrete target pair query");
    compose_orders([
        compare_input_policy_fit(&required_source.pair, &a.input_policy, &b.input_policy),
        compare_output_policy_fit(target_query, &a.output_policy, &b.output_policy),
        compare_mode_fit(required_source.mode, a.input_mode, b.input_mode),
        compare_mode_fit(target_demand.mode, a.output_mode, b.output_mode),
    ])
}

/// Select from a caller-supplied transitional candidate family.
///
/// Endpoint Policy fitness models only the endpoint-coordinate portion of
/// future Bp' and runs before the prototype's later preference stand-ins. It
/// is not sequentially composable with an ordinary-Bp maxima pass. This
/// resolver first applies the canonical post-admissibility, pre-Bp fallback
/// suppression rule. It does not perform global Symbol lookup or ordinary
/// function-object invocation. It never feeds a candidate result back as
/// another request and therefore cannot perform transitive search.
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

    let fallback_survivors = suppress_fallback_candidates(&admissible);
    let policy_survivors = prototype_endpoint_policy_maxima(request, &fallback_survivors);
    let entry_survivors = maximal_candidates(&policy_survivors, |better, worse| {
        ordinary_candidate_dominates(better, worse)
    });
    let maximal = maximal_candidates(&entry_survivors, |better, worse| {
        better.prototype_pattern_specificity > worse.prototype_pattern_specificity
    });

    match maximal.as_slice() {
        [] => PolicyBridgeResolution::NoCandidate,
        [candidate] if candidate.is_delete => {
            PolicyBridgeResolution::RejectedByDelete(candidate.id.clone())
        }
        [candidate] => PolicyBridgeResolution::Selected(ResolvedPolicyBridge {
            callable: (*candidate).clone(),
            result_type: candidate.output_type.resolve(request.source_type()),
            complete_result_view: PolicyView {
                pair: candidate.output_policy.clone(),
                mode: candidate.output_mode,
            },
            validated_output_endpoint: project_migration_output_endpoint(
                request.target_pair(),
                &candidate.output_policy,
            )
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

/// Prototype the semantics fixed for a possible future fallback strategy.
///
/// Current source has no fallback role, so this helper is identity there. If
/// future metadata is present, suppression happens after full admissibility
/// and before Bp'. Any admissible non-fallback candidate, including `delete`,
/// permanently removes every fallback candidate. Later failure never reopens
/// this set.
fn suppress_fallback_candidates<'a, I>(
    fully_admissible: &[&'a PolicyTransitionCallable<I>],
) -> Vec<&'a PolicyTransitionCallable<I>> {
    if fully_admissible
        .iter()
        .any(|candidate| !candidate.prototype_is_fallback)
    {
        fully_admissible
            .iter()
            .copied()
            .filter(|candidate| !candidate.prototype_is_fallback)
            .collect()
    } else {
        fully_admissible.to_vec()
    }
}

/// Compute maxima for only the prototype migration-endpoint Policy coordinates.
///
/// This is private because endpoint-only maxima cannot be sequentially
/// composed with ordinary Bp maxima. Final Bp' must compare ordinary Bp,
/// migration-input, and migration-output coordinates as one product before
/// taking maxima. The prototype has no ordinary-Bp carrier and proves only the
/// endpoint coordinate relation plus its placement before the B3 stand-in.
fn prototype_endpoint_policy_maxima<'a, I>(
    request: &PolicyTransitionRequest,
    fully_admissible: &[&'a PolicyTransitionCallable<I>],
) -> Vec<&'a PolicyTransitionCallable<I>> {
    maximal_candidates(fully_admissible, |better, worse| {
        matches!(
            compare_policy_transition_candidates(
                request.source_view(),
                request.target_demand(),
                better,
                worse,
            ),
            PolicyPartialOrdering::Greater
        )
    })
}

pub fn qualify_policy_bridge<I: Clone>(
    request: &PolicyTransitionRequest,
    candidates: &[PolicyTransitionCallable<I>],
    expectation: TransitionTypeExpectation,
) -> BridgeQualification<I> {
    match resolve_policy_bridge(request, candidates, expectation) {
        PolicyBridgeResolution::Selected(selected) => BridgeQualification::Available(selected),
        PolicyBridgeResolution::RejectedByDelete(id) => BridgeQualification::RejectedByDelete(id),
        PolicyBridgeResolution::Ambiguous(ids) => BridgeQualification::Ambiguous(ids),
        PolicyBridgeResolution::NoCandidate => BridgeQualification::Missing,
    }
}

/// Fixture-only result carrier for the prototype migration adapter.
///
/// Carrying a complete entry proves that the adapter does not synthesize a
/// Pattern by copying the source demand. It does not prove that the supplied
/// TypeValue, PatternValue, Pattern owner, or constructor/extractor relations
/// form a coherent final ordinary invocation result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrototypeTransitionResultCarrier<P> {
    pub entry: PolicyResultEntry<SemanticValueRef, P>,
    pub source_value: SemanticValueId,
    pub provenance: Provenance,
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
    demands: &[PolicyTransitionDemand],
    produced: &[PrototypeTransitionResultCarrier<P>],
) -> Result<Vec<PolicyResultEntry<SemanticValueRef, P>>, P1AssemblyFailure> {
    if demands.len() != produced.len() {
        return Err(P1AssemblyFailure::ProducedValueCountMismatch {
            expected: demands.len(),
            actual: produced.len(),
        });
    }

    let mut results = Vec::with_capacity(produced.len());
    for (demand_index, (demand, produced_result)) in demands.iter().zip(produced).enumerate() {
        if produced_result.source_value != demand.request.source_value() {
            return Err(P1AssemblyFailure::ProducedValueDoesNotMatchDemand { demand_index });
        }
        let projected = project_p1(
            &demand.request.target_demand().pair_query,
            std::slice::from_ref(&produced_result.entry),
        );
        if projected.is_empty() {
            return Err(P1AssemblyFailure::ProducedValueDoesNotMatchDemand { demand_index });
        }
        results.extend(projected);
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
pub struct PolicyBridgeInvocationResult<I, P> {
    pub callable_id: I,
    pub result: PrototypeTransitionResultCarrier<P>,
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
/// selection or choose a former second-place candidate. `result_pattern` is
/// explicit fixture material; accepting it does not establish final ordinary
/// result Type/Pattern/owner coherence. The produced entry carries the
/// callable's complete ordinary result Policy; `assemble_transition_results`
/// performs the separate demanded `Project_out`.
pub fn invoke_resolved_policy_bridge<I: Clone, P>(
    selected: &ResolvedPolicyBridge<I>,
    request: &PolicyTransitionRequest,
    result_value: SemanticValueId,
    result_pattern: P,
) -> Result<PolicyBridgeInvocationResult<I, P>, PolicyBridgeInvocationFailure<I>> {
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
        result: PrototypeTransitionResultCarrier {
            entry: PolicyResultEntry {
                value: Some(SemanticValueRef {
                    id: result_value,
                    type_value: selected.result_type,
                }),
                pattern: result_pattern,
                view: selected.complete_result_view.clone(),
            },
            source_value: request.source_value(),
            provenance: request.provenance().clone(),
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
    {
        return false;
    }
    let Some(input_view) =
        project_migration_input_endpoint(&candidate.input_policy, &request.source_view().pair)
    else {
        return false;
    };
    let Some(result_policy) =
        project_migration_output_endpoint(request.target_pair(), &candidate.output_policy)
    else {
        return false;
    };
    if validate_runtime_transition(&input_view, &result_policy).is_err() {
        return false;
    }
    let result_type = candidate.output_type.resolve(request.source_type());
    if result_type != request.source_type() {
        return false;
    }
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

fn ordinary_candidate_dominates<I>(
    better: &PolicyTransitionCallable<I>,
    worse: &PolicyTransitionCallable<I>,
) -> bool {
    matches!(
        compare_input_type_fit(better.input_type, worse.input_type),
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
    if project_migration_input_endpoint(left, required).is_none()
        || project_migration_input_endpoint(right, required).is_none()
    {
        return PolicyPartialOrdering::Incomparable;
    }
    compare_policy_endpoint_fit(required, left, right)
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

    compare_policy_endpoint_fit(required, left, right)
}

fn compare_mode_fit(
    demand: PolicyMode,
    left: PolicyMode,
    right: PolicyMode,
) -> PolicyPartialOrdering {
    match mutability_preference_rank(left, demand).cmp(&mutability_preference_rank(right, demand)) {
        std::cmp::Ordering::Less => PolicyPartialOrdering::Less,
        std::cmp::Ordering::Equal => PolicyPartialOrdering::Equal,
        std::cmp::Ordering::Greater => PolicyPartialOrdering::Greater,
    }
}

/// Endpoint-coordinate comparison used by the connected ordinary Bp' carrier.
///
/// This returns only the migration input/output portion of the product.  The
/// caller must compose it with ordinary Bp coordinates before taking maxima;
/// it must never run as a sequential filter.
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

fn compare_policy_endpoint_fit(
    _required: &PolicyPair,
    left: &PolicyPair,
    right: &PolicyPair,
) -> PolicyPartialOrdering {
    compose_orders([
        compare_stage_domains(&left.value.stages, &right.value.stages),
        // Presence ordering is a coordinate of this prototype's endpoint Bp
        // extension. It is not asserted as a general ordinary-call order.
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
