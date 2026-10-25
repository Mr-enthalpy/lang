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
        project_p1, P1Projection, PolicyPair, PolicyResultEntry, PolicyView, ResultPolicyDemand,
        ValuePresence,
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
    TargetValueAbsent,
    TargetStageOmitted,
    /// Active reify boundary, independent return classifier and continuation
    /// witnesses are required before reverse execution can be formed.
    ReifyBoundaryUnavailable,
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
            .ok_or(PolicyMigrationRequestFailure::TargetStageOmitted)?;
        if reverse_stage_endpoint(&source_view.pair, target_pair) {
            return Err(PolicyMigrationRequestFailure::ReifyBoundaryUnavailable);
        }
        if source_view.pair.value.presence() == ValuePresence::Absent {
            return Err(PolicyMigrationRequestFailure::SourceValueAbsent);
        }
        if target_pair.value.presence() == ValuePresence::Absent {
            return Err(PolicyMigrationRequestFailure::TargetValueAbsent);
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

    pub(crate) fn validate_connected_execution(&self) -> Result<(), crate::Diagnostic> {
        if reverse_stage_endpoint(self.source_policy(), self.target_pair()) {
            return Err(crate::Diagnostic::hard_error(
                "runtime-to-compile migration requires the unavailable active reify P2 / closed-return / dependent-continuation consumer",
                Some(self.provenance.clone()),
            ));
        }
        Ok(())
    }
}

fn reverse_stage_endpoint(source: &PolicyPair, target: &PolicyPair) -> bool {
    (source.value.stage() == Some(crate::Stage::Runtime)
        && target.value.stage() == Some(crate::Stage::Compile))
        || (source.pattern.stage == crate::Stage::Runtime
            && target.pattern.stage == crate::Stage::Compile)
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
        P1Projection::Pair(pair) => {
            if let Some(stage) = pair.value.stage() {
                pair.value = crate::ValueComponentPolicy::Optional(stage);
            }
        }
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
    let presence = intersect_presence(query.value.presence(), available.value.presence())?;
    if query.pattern.stage != available.pattern.stage
        || (presence != ValuePresence::Absent && query.value.stage() != available.value.stage())
    {
        return None;
    }
    Some(PolicyPair {
        value: available
            .value
            .stage()
            .map(|s| match presence {
                ValuePresence::Present => crate::ValueComponentPolicy::Present(s),
                ValuePresence::Optional => crate::ValueComponentPolicy::Optional(s),
                ValuePresence::Absent => crate::ValueComponentPolicy::Absent,
            })
            .unwrap_or(crate::ValueComponentPolicy::Absent),
        pattern: available.pattern,
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
        compare_stage_atoms(left.value.stage(), right.value.stage()),
        compare_presence_domains(left.value.presence(), right.value.presence()),
        compare_stage_atoms(Some(left.pattern.stage), Some(right.pattern.stage)),
    ])
}

fn compare_stage_atoms(
    left: Option<crate::Stage>,
    right: Option<crate::Stage>,
) -> PolicyPartialOrdering {
    if left == right {
        PolicyPartialOrdering::Equal
    } else {
        PolicyPartialOrdering::Incomparable
    }
}

#[cfg(test)]
mod stage_endpoint_tests {
    use super::project_migration_endpoint_hard_coordinates;
    use crate::{PatternComponentPolicy, PolicyPair, Stage, ValueComponentPolicy};

    #[test]
    fn reverse_request_cannot_bypass_execution_authority_validation() {
        let base = crate::CompilationWorld::from_manifest(&crate::BuildManifest::new(
            "app",
            vec!["app".into()],
        ))
        .unwrap();
        let mut world = base.semantic_world().clone();
        let source = crate::declared_policy_view(Stage::Runtime, crate::PolicyMode::Const);
        let target = crate::declared_policy_view(Stage::Compile, crate::PolicyMode::Const);
        let request = super::PolicyMigrationRequest {
            source_view: source,
            target_demand: crate::ResultPolicyDemand {
                pair_query: crate::P1Projection::Pair(target.pair),
                mode: target.mode,
            },
            source_type: world.type_rank().unwrap(),
            source_value: crate::SemanticValueId(999999),
            provenance: crate::Provenance::new("unwitnessed internal request"),
        };
        let before = format!("{world:?}");
        let result = crate::invoke_policy_migration(&mut world, &request, &base.package_context());
        assert!(
            matches!(result,Err(crate::OrdinaryInvocationFailure::ApplicabilityUnsupported {diagnostic,..}) if diagnostic.message.contains("active reify P2"))
        );
        assert_eq!(format!("{world:?}"), before);
        let binding = world
            .symbol_in_namespace(base.core_node(), "IdentityType")
            .unwrap()
            .identity;
        let parsed = lang_syntax::parse("let result=uint8 IdentityType::core;");
        let norm = lang_syntax::normalize_program(&parsed.program);
        let lang_syntax::NormForm::Let(lang_syntax::NormDecl::Let { slot, .. }) = &norm.forms[0]
        else {
            panic!("let")
        };
        let site = crate::extract_single_call_site(slot.initializer.as_deref().unwrap()).unwrap();
        let mut context = crate::OrdinaryInvocationContext::open_static(&[]);
        context.migration = Some(crate::MigrationInvocationContext {
            request: &request,
            source_value: request.source_value,
        });
        let bypass = crate::invoke_resolved_binding_ordinary(
            &mut world,
            &[],
            binding,
            &site,
            &base.package_context(),
            context,
            crate::Provenance::new("ordinary invocation entrance"),
        );
        assert!(
            matches!(bypass,Err(crate::OrdinaryInvocationFailure::ApplicabilityUnsupported {diagnostic,..}) if diagnostic.message.contains("active reify P2"))
        );
        assert_eq!(format!("{world:?}"), before);
    }

    fn endpoint(value: ValueComponentPolicy) -> PolicyPair {
        PolicyPair {
            value,
            pattern: PatternComponentPolicy {
                stage: Stage::Compile,
            },
        }
    }

    #[test]
    fn absent_presence_intersection_has_no_value_stage_to_compare() {
        let optional = endpoint(ValueComponentPolicy::Optional(Stage::Runtime));
        let absent = endpoint(ValueComponentPolicy::Absent);
        for (query, actual) in [(&optional, &absent), (&absent, &optional)] {
            let result = project_migration_endpoint_hard_coordinates(query, actual).unwrap();
            assert_eq!(result.value, ValueComponentPolicy::Absent);
            assert_eq!(result.pattern, absent.pattern);
        }
        assert!(project_migration_endpoint_hard_coordinates(
            &endpoint(ValueComponentPolicy::Present(Stage::Runtime)),
            &absent,
        )
        .is_none());
    }
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
