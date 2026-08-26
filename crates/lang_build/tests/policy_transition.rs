mod support;

use std::convert::Infallible;

use lang_build::{
    assemble_transition_results, compare_policy_transition_candidates,
    elaborate_pure_type_binding_p1, elaborate_value_binding_p1, expose_policy_slice,
    invoke_resolved_policy_bridge, project_transition_policy_domain, qualify_policy_bridge,
    read_pattern, read_value, resolve_policy_bridge, select_policy_overload,
    validate_runtime_transition, AtomicBuiltinType, AtomicBuiltinTypeRegistry,
    AtomicBuiltinTypeRegistryFailure, BridgeQualification, CompilationWorld, MutabilityActualFrame,
    MutabilityFormalFrame, MutabilityPattern, NumericFamily, NumericTypeKey, NumericTypeRegistry,
    OrdinaryCallableTypeInput, OrdinaryCallableTypeOutput, OutputModeDemand, P1Elaboration,
    P1ElaborationFailure, P1Origin, P1Projection, PatternComponentPolicy, Phase,
    PhaseOverloadCandidate, PolicyBridgeBody, PolicyBridgeResolution, PolicyMode,
    PolicyOverloadCandidate, PolicyOverloadSelection, PolicyPair, PolicyPartialOrdering,
    PolicyResultEntry, PolicyStage, PolicyTransitionCallable, PolicyTransitionFailure,
    PolicyTransitionRequest, PolicyTransitionRequestFailure, PolicyView, Provenance,
    ResultPolicyDemand, SemanticValueId, SemanticValueRef, StageSet, SymbolPayload,
    TransitionTypeExpectation, TypeValueId, ValueComponentPolicy, ValuePresence,
};
use support::empty_app_manifest;

fn stages(items: &[PolicyStage]) -> StageSet {
    let mut stages = StageSet::new();
    for stage in items {
        stages.insert(*stage);
    }
    stages
}

fn pair(
    value_stages: &[PolicyStage],
    pattern_stages: &[PolicyStage],
    _mode: &[PolicyMode],
) -> PolicyPair {
    PolicyPair {
        value: ValueComponentPolicy {
            stages: stages(value_stages),
            presence: ValuePresence::Present,
        },
        pattern: PatternComponentPolicy {
            stages: stages(pattern_stages),
        },
    }
}

fn absent_pair(pattern_stages: &[PolicyStage]) -> PolicyPair {
    PolicyPair {
        value: ValueComponentPolicy {
            stages: StageSet::new(),
            presence: ValuePresence::Absent,
        },
        pattern: PatternComponentPolicy {
            stages: stages(pattern_stages),
        },
    }
}

fn compile_pair() -> PolicyPair {
    pair(&[PolicyStage::Compile], &[PolicyStage::Compile], &[])
}

fn optional_compile_pair() -> PolicyPair {
    let mut policy = compile_pair();
    policy.value.presence = ValuePresence::Optional;
    policy
}

fn meta_pair() -> PolicyPair {
    pair(&[PolicyStage::Meta], &[PolicyStage::Meta], &[])
}

fn runtime_pair() -> PolicyPair {
    pair(&[PolicyStage::Runtime], &[PolicyStage::Compile], &[])
}

fn runtime_meta_pair() -> PolicyPair {
    pair(&[PolicyStage::Runtime], &[PolicyStage::Meta], &[])
}

fn compile_runtime_pair() -> PolicyPair {
    pair(
        &[PolicyStage::Compile, PolicyStage::Runtime],
        &[PolicyStage::Compile],
        &[],
    )
}

fn runtime_pair_with(mutability: PolicyMode) -> PolicyPair {
    pair(
        &[PolicyStage::Runtime],
        &[PolicyStage::Compile],
        &[mutability],
    )
}

fn compile_pair_with(mutability: PolicyMode) -> PolicyPair {
    pair(
        &[PolicyStage::Compile],
        &[PolicyStage::Compile],
        &[mutability],
    )
}

fn compile_runtime_pair_with(mutability: PolicyMode) -> PolicyPair {
    pair(
        &[PolicyStage::Compile, PolicyStage::Runtime],
        &[PolicyStage::Compile],
        &[mutability],
    )
}

fn broad_static_pair() -> PolicyPair {
    pair(
        &[PolicyStage::Meta, PolicyStage::Compile],
        &[PolicyStage::Meta, PolicyStage::Compile],
        &[],
    )
}

fn broad_runtime_output() -> PolicyPair {
    compile_runtime_pair()
}

fn runtime_with_broad_static_pattern() -> PolicyPair {
    pair(
        &[PolicyStage::Runtime],
        &[PolicyStage::Meta, PolicyStage::Compile],
        &[],
    )
}

fn value_entry(
    id: u64,
    type_value: u64,
    value_stages: &[PolicyStage],
    pattern_stages: &[PolicyStage],
) -> PolicyResultEntry<SemanticValueRef, &'static str> {
    let policy = pair(value_stages, pattern_stages, &[]);
    PolicyResultEntry {
        value: Some(SemanticValueRef {
            id: SemanticValueId(id),
            type_value: TypeValueId(type_value),
        }),
        pattern: "pattern",
        view: PolicyView {
            pair: policy,
            mode: PolicyMode::Plain,
        },
    }
}

fn pure_entry(pattern_stages: &[PolicyStage]) -> PolicyResultEntry<Infallible, &'static str> {
    let policy = absent_pair(pattern_stages);
    PolicyResultEntry {
        value: None,
        pattern: "type-pattern",
        view: PolicyView {
            pair: policy,
            mode: PolicyMode::Plain,
        },
    }
}

fn request(source_policy: PolicyPair, target_policy: PolicyPair) -> PolicyTransitionRequest {
    request_with_modes(
        source_policy,
        PolicyMode::Plain,
        target_policy,
        PolicyMode::Plain,
    )
}

fn request_with_modes(
    source_policy: PolicyPair,
    source_mode: PolicyMode,
    target_policy: PolicyPair,
    target_mode: PolicyMode,
) -> PolicyTransitionRequest {
    PolicyTransitionRequest::new(
        PolicyView {
            pair: source_policy,
            mode: source_mode,
        },
        ResultPolicyDemand {
            pair_query: P1Projection::Pair(target_policy),
            mode: target_mode,
        },
        TypeValueId(10),
        SemanticValueId(20),
        Provenance::new("transition request"),
    )
    .expect("value-bearing transition request")
}

fn callable(
    id: &'static str,
    input_type: OrdinaryCallableTypeInput,
    output_type: OrdinaryCallableTypeOutput,
    input_policy: PolicyPair,
    output_policy: PolicyPair,
    is_delete: bool,
    body: PolicyBridgeBody,
) -> PolicyTransitionCallable<&'static str> {
    callable_with_modes(
        id,
        input_type,
        output_type,
        input_policy,
        PolicyMode::Plain,
        output_policy,
        PolicyMode::Plain,
        is_delete,
        body,
    )
}

#[allow(clippy::too_many_arguments)]
fn callable_with_modes(
    id: &'static str,
    input_type: OrdinaryCallableTypeInput,
    output_type: OrdinaryCallableTypeOutput,
    input_policy: PolicyPair,
    input_mode: PolicyMode,
    output_policy: PolicyPair,
    output_mode: PolicyMode,
    is_delete: bool,
    body: PolicyBridgeBody,
) -> PolicyTransitionCallable<&'static str> {
    PolicyTransitionCallable {
        id,
        input_type,
        output_type,
        input_policy,
        output_policy,
        input_mode,
        output_mode,
        ordinary_fully_admissible: true,
        prototype_is_fallback: false,
        prototype_pattern_specificity: 0,
        is_delete,
        body,
    }
}

fn exact_copy(id: &'static str) -> PolicyTransitionCallable<&'static str> {
    callable(
        id,
        OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
        OrdinaryCallableTypeOutput::SameAsInput,
        compile_pair(),
        runtime_pair(),
        false,
        PolicyBridgeBody::BuiltinValueCopy,
    )
}

#[test]
fn omitted_ordinary_p1_preserves_complete_rhs_without_stage_lift() {
    let result = vec![value_entry(
        1,
        10,
        &[PolicyStage::Runtime],
        &[PolicyStage::Compile],
    )];
    let elaboration =
        elaborate_value_binding_p1(&result, None, Provenance::new("omitted P1")).unwrap();
    let P1Elaboration::Projected {
        origin,
        requested,
        selected,
    } = elaboration
    else {
        panic!("omitted ordinary P1 must project the complete RHS");
    };
    assert_eq!(origin, P1Origin::Inferred);
    assert_eq!(requested, None);
    assert_eq!(selected, result);
    assert_eq!(
        selected[0].view.pair.value.stages,
        StageSet::from([PolicyStage::Runtime]),
        "ordinary binding must not copy the function-object P1 stage lift"
    );
}

#[test]
fn omitted_p1_preserves_a_mixed_result_collection_exactly() {
    let result = vec![
        value_entry(1, 10, &[PolicyStage::Runtime], &[PolicyStage::Compile]),
        PolicyResultEntry {
            value: None,
            pattern: "type-pattern",
            view: PolicyView {
                pair: absent_pair(&[PolicyStage::Compile]),
                mode: PolicyMode::Plain,
            },
        },
    ];
    let P1Elaboration::Projected { selected, .. } =
        elaborate_value_binding_p1(&result, None, Provenance::new("omitted mixed P1"))
            .expect("omitted P1 is exact identity over the collection")
    else {
        panic!("omitted P1 cannot enter transition");
    };
    assert_eq!(selected, result);
}

#[test]
fn explicit_identical_p1_is_an_existing_projection() {
    let result = vec![value_entry(
        1,
        10,
        &[PolicyStage::Compile],
        &[PolicyStage::Compile],
    )];
    let projection = P1Projection::Pair(compile_pair());
    let elaboration = elaborate_value_binding_p1(
        &result,
        Some(&projection),
        Provenance::new("explicit identity"),
    )
    .unwrap();
    let P1Elaboration::Projected {
        origin, selected, ..
    } = elaboration
    else {
        panic!("identical P1 must remain projection");
    };
    assert_eq!(origin, P1Origin::Explicit);
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].value.unwrap().id, SemanticValueId(1));
}

#[test]
fn meta_or_runtime_query_over_meta_only_source_selects_meta_without_transition() {
    let result = vec![value_entry(
        1,
        10,
        &[PolicyStage::Meta],
        &[PolicyStage::Meta],
    )];
    let query = P1Projection::ValueDominant {
        value: pair(
            &[PolicyStage::Meta, PolicyStage::Runtime],
            &[PolicyStage::Meta],
            &[],
        )
        .value,
    };
    let elaboration = elaborate_value_binding_p1(
        &result,
        Some(&query),
        Provenance::new("meta || runtime query"),
    )
    .unwrap();
    let P1Elaboration::Projected { selected, .. } = elaboration else {
        panic!("a non-empty meta projection must not manufacture runtime");
    };
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].value.unwrap().id, SemanticValueId(1));
    assert_eq!(
        selected[0].view.pair.value.stages,
        StageSet::from([PolicyStage::Meta])
    );
}

#[test]
fn compile_or_runtime_query_over_compile_source_selects_compile_without_transition() {
    let result = vec![value_entry(
        20,
        10,
        &[PolicyStage::Compile],
        &[PolicyStage::Compile],
    )];
    let query = P1Projection::Pair(compile_runtime_pair());
    let elaboration = elaborate_value_binding_p1(
        &result,
        Some(&query),
        Provenance::new("compile || runtime query"),
    )
    .expect("the available compile branch satisfies the query");
    let P1Elaboration::Projected { selected, .. } = elaboration else {
        panic!("a non-empty compile projection must not request runtime");
    };
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].value.unwrap().id, SemanticValueId(20));
    assert_eq!(
        selected[0].view.pair.value.stages,
        StageSet::from([PolicyStage::Compile])
    );
}

#[test]
fn existing_runtime_slice_dominates_migration_and_preserves_identity() {
    let result = vec![value_entry(
        20,
        10,
        &[PolicyStage::Compile, PolicyStage::Runtime],
        &[PolicyStage::Compile],
    )];
    let query = P1Projection::ValueDominant {
        value: runtime_pair().value,
    };
    let P1Elaboration::Projected { selected, .. } = elaborate_value_binding_p1(
        &result,
        Some(&query),
        Provenance::new("existing runtime slice"),
    )
    .expect("the existing runtime branch satisfies the demand") else {
        panic!("migration preparation is unreachable after a non-empty projection");
    };
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].value.unwrap().id, SemanticValueId(20));
    assert_eq!(
        selected[0].view.pair.value.stages,
        StageSet::from([PolicyStage::Runtime])
    );

    let open_static_view = expose_policy_slice(&selected[0], Phase::OpenStatic);
    assert!(
        read_value(&open_static_view).is_none(),
        "an extant runtime Policy slice is not a statically readable runtime value"
    );
    assert_eq!(read_pattern(&open_static_view), Some(&"pattern"));
}

#[test]
fn empty_runtime_projection_prepares_one_direct_transition_demand() {
    let result = vec![value_entry(
        20,
        10,
        &[PolicyStage::Compile],
        &[PolicyStage::Compile],
    )];
    let query = P1Projection::Pair(runtime_pair());
    let elaboration =
        elaborate_value_binding_p1(&result, Some(&query), Provenance::new("runtime query"))
            .unwrap();
    let P1Elaboration::AtomicRuntimeMigration { requested, demands } = elaboration else {
        panic!("empty value projection may enter transition preparation");
    };
    assert_eq!(requested, query);
    assert_eq!(demands.len(), 1);
    let demand = &demands[0].request;
    assert_eq!(demand.source_policy(), &compile_pair());
    assert_eq!(demand.target_query(), &runtime_pair());
    assert_eq!(demand.source_value(), SemanticValueId(20));
}

#[test]
fn mixed_choice_extracts_runtime_branch_only_after_complete_projection_fails() {
    let result = vec![value_entry(
        20,
        10,
        &[PolicyStage::Compile],
        &[PolicyStage::Compile],
    )];
    let broad_query_pair = pair(
        &[PolicyStage::Meta, PolicyStage::Runtime],
        &[PolicyStage::Compile],
        &[],
    );
    let query = P1Projection::Pair(broad_query_pair.clone());
    let P1Elaboration::AtomicRuntimeMigration { requested, demands } = elaborate_value_binding_p1(
        &result,
        Some(&query),
        Provenance::new("meta || runtime query over compile"),
    )
    .expect("runtime is a constructible accepted branch") else {
        panic!("the complete query is empty, so its runtime branch may be constructed");
    };
    assert_eq!(requested, query);
    assert_eq!(demands.len(), 1);
    assert_eq!(demands[0].request.source_policy(), &compile_pair());
    assert_eq!(
        demands[0].request.target_query(),
        &runtime_pair(),
        "the internal migration request targets only the extracted runtime branch"
    );
}

#[test]
fn empty_query_without_runtime_branch_does_not_authorize_migration() {
    let result = vec![value_entry(
        20,
        10,
        &[PolicyStage::Compile],
        &[PolicyStage::Compile],
    )];
    let query = P1Projection::Pair(pair(&[PolicyStage::Meta], &[PolicyStage::Compile], &[]));
    assert_eq!(
        elaborate_value_binding_p1(
            &result,
            Some(&query),
            Provenance::new("meta-only query over compile"),
        ),
        Err(
            P1ElaborationFailure::ProjectionUnavailableOutsideAtomicRuntimeMigration {
                requested: query
            }
        )
    );
}

#[test]
fn multi_entry_query_stops_after_any_existing_projection() {
    let result = vec![
        value_entry(20, 10, &[PolicyStage::Compile], &[PolicyStage::Compile]),
        value_entry(30, 11, &[PolicyStage::Runtime], &[PolicyStage::Compile]),
    ];
    let query = P1Projection::Pair(runtime_pair());
    let elaboration = elaborate_value_binding_p1(
        &result,
        Some(&query),
        Provenance::new("multi-entry runtime query"),
    )
    .unwrap();
    let P1Elaboration::Projected { selected, .. } = elaboration else {
        panic!("an existing runtime entry satisfies the complete P1 query");
    };
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].value.unwrap().id, SemanticValueId(30));
}

#[test]
fn value_dominant_transition_query_preserves_the_source_pattern_component() {
    let result = vec![value_entry(
        20,
        10,
        &[PolicyStage::Compile],
        &[PolicyStage::Compile],
    )];
    let query = P1Projection::ValueDominant {
        value: runtime_pair().value,
    };
    let P1Elaboration::AtomicRuntimeMigration { demands, .. } = elaborate_value_binding_p1(
        &result,
        Some(&query),
        Provenance::new("value-dominant runtime query"),
    )
    .unwrap() else {
        panic!("runtime is not an existing value slice");
    };
    assert_eq!(demands[0].request.target_query(), &runtime_pair());
}

#[test]
fn pair_transition_query_crops_pattern_before_preparing_value_demand() {
    let result = vec![value_entry(
        20,
        10,
        &[PolicyStage::Meta, PolicyStage::Compile],
        &[PolicyStage::Meta, PolicyStage::Compile],
    )];
    let query = P1Projection::Pair(runtime_pair());
    let P1Elaboration::AtomicRuntimeMigration { demands, .. } = elaborate_value_binding_p1(
        &result,
        Some(&query),
        Provenance::new("runtime value plus compile Pattern slice"),
    )
    .expect("the Pattern side has an existing compile slice") else {
        panic!("runtime is not an existing value slice");
    };
    assert_eq!(demands.len(), 1);
    assert_eq!(demands[0].request.source_policy(), &compile_pair());
    assert_eq!(demands[0].request.target_query(), &runtime_pair());
}

#[test]
fn value_transition_cannot_manufacture_an_unavailable_pattern_slice() {
    let result = vec![value_entry(
        20,
        10,
        &[PolicyStage::Compile],
        &[PolicyStage::Compile],
    )];
    let query = P1Projection::Pair(pair(&[PolicyStage::Runtime], &[PolicyStage::Seal], &[]));
    assert!(matches!(
        elaborate_value_binding_p1(
            &result,
            Some(&query),
            Provenance::new("unavailable Pattern slice")
        ),
        Err(P1ElaborationFailure::PatternPolicyStageSliceUnavailableForMigration { .. })
    ));
}

#[test]
fn pure_type_can_project_pattern_slice_without_value_identity_or_transition_api() {
    let result = vec![pure_entry(&[PolicyStage::Meta, PolicyStage::Compile])];
    let target = P1Projection::Pair(absent_pair(&[PolicyStage::Compile]));
    let elaboration =
        elaborate_pure_type_binding_p1(&result, Some(&target)).expect("pure compile slice");
    assert_eq!(elaboration.selected.len(), 1);
    assert_eq!(elaboration.selected[0].value, None);
    assert_eq!(
        elaboration.selected[0].view.pair.pattern.stages,
        StageSet::from([PolicyStage::Compile])
    );
}

#[test]
fn pure_type_unavailable_pattern_slice_is_projection_failure() {
    let result = vec![pure_entry(&[PolicyStage::Compile])];
    let target = P1Projection::Pair(absent_pair(&[PolicyStage::Seal]));
    assert!(matches!(
        elaborate_pure_type_binding_p1(&result, Some(&target)),
        Err(P1ElaborationFailure::ProjectionUnavailableWithoutValue { .. })
    ));
}

#[test]
fn mixed_result_preserves_old_projection_before_absent_entries_are_considered() {
    let result = vec![
        value_entry(30, 10, &[PolicyStage::Runtime], &[PolicyStage::Compile]),
        PolicyResultEntry {
            value: None,
            pattern: "type-pattern",
            view: PolicyView {
                pair: absent_pair(&[PolicyStage::Compile]),
                mode: PolicyMode::Plain,
            },
        },
    ];
    let target = P1Projection::ValueDominant {
        value: runtime_pair().value,
    };
    let P1Elaboration::Projected { selected, .. } = elaborate_value_binding_p1(
        &result,
        Some(&target),
        Provenance::new("mixed result old projection"),
    )
    .expect("an absent sibling must not invalidate an existing value projection") else {
        panic!("old non-empty projection must remain authoritative");
    };
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].value.unwrap().id, SemanticValueId(30));
}

#[test]
fn all_absent_general_result_fails_only_after_projection_is_empty() {
    let result = vec![PolicyResultEntry {
        value: None,
        pattern: "type-pattern",
        view: PolicyView {
            pair: absent_pair(&[PolicyStage::Compile]),
            mode: PolicyMode::Plain,
        },
    }];
    let target = P1Projection::Pair(runtime_pair());
    assert!(matches!(
        elaborate_value_binding_p1(&result, Some(&target), Provenance::new("all-absent result")),
        Err(P1ElaborationFailure::ProjectionUnavailableWithoutValue { .. })
    ));
}

#[test]
fn absent_source_cannot_construct_or_validate_a_transition() {
    assert_eq!(
        PolicyTransitionRequest::new(
            PolicyView {
                pair: absent_pair(&[PolicyStage::Compile]),
                mode: PolicyMode::Plain,
            },
            ResultPolicyDemand {
                pair_query: P1Projection::Pair(runtime_pair()),
                mode: PolicyMode::Plain,
            },
            TypeValueId(10),
            SemanticValueId(20),
            Provenance::new("absent source"),
        ),
        Err(PolicyTransitionRequestFailure::SourceValueAbsent)
    );
    assert_eq!(
        validate_runtime_transition(&absent_pair(&[PolicyStage::Compile]), &runtime_pair()),
        Err(PolicyTransitionFailure::SourceValueAbsent)
    );
}

#[test]
fn transition_policy_domain_projection_preserves_presence_intersection() {
    let present = project_transition_policy_domain(&compile_pair(), &optional_compile_pair())
        .expect("present intersects optional");
    assert_eq!(present.value.presence, ValuePresence::Present);
    assert_eq!(present.value.stages, StageSet::from([PolicyStage::Compile]));

    let absent_query = absent_pair(&[PolicyStage::Compile]);
    let absent = project_transition_policy_domain(&absent_query, &optional_compile_pair())
        .expect("absent intersects optional");
    assert_eq!(absent.value.presence, ValuePresence::Absent);
    assert!(absent.value.stages.is_empty());

    assert!(
        project_transition_policy_domain(&compile_pair(), &absent_pair(&[PolicyStage::Compile]))
            .is_none(),
        "present and absent domains do not intersect"
    );
}

#[test]
fn legal_runtime_value_transition_preserves_available_pattern_capability() {
    assert_eq!(
        validate_runtime_transition(&compile_pair(), &runtime_pair()),
        Ok(())
    );
}

#[test]
fn selected_atomic_runtime_migration_keeps_pattern_policy_unchanged() {
    let source = broad_static_pair();
    assert_eq!(
        validate_runtime_transition(&source, &runtime_with_broad_static_pattern()),
        Ok(())
    );
    assert!(matches!(
        validate_runtime_transition(&source, &runtime_pair()),
        Err(PolicyTransitionFailure::PatternPolicyChanged { .. })
    ));
}

#[test]
fn atomic_runtime_migration_rejects_runtime_in_the_selected_input_endpoint() {
    assert_eq!(
        PolicyTransitionRequest::new(
            PolicyView {
                pair: compile_runtime_pair(),
                mode: PolicyMode::Plain,
            },
            ResultPolicyDemand {
                pair_query: P1Projection::Pair(runtime_pair()),
                mode: PolicyMode::Plain,
            },
            TypeValueId(10),
            SemanticValueId(20),
            Provenance::new("selected input contains runtime"),
        ),
        Err(PolicyTransitionRequestFailure::SelectedInputContainsRuntime)
    );
}

#[test]
fn const_compile_can_materialize_a_fresh_mut_runtime_value() {
    let source = compile_pair_with(PolicyMode::Const);
    let target = runtime_pair_with(PolicyMode::Mut);
    assert_eq!(validate_runtime_transition(&source, &target), Ok(()));
    let request = request_with_modes(
        source.clone(),
        PolicyMode::Const,
        target.clone(),
        PolicyMode::Mut,
    );
    let candidate = callable_with_modes(
        "const-compile-to-mut-runtime",
        OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
        OrdinaryCallableTypeOutput::SameAsInput,
        source,
        PolicyMode::Const,
        target,
        PolicyMode::Mut,
        false,
        PolicyBridgeBody::BuiltinValueCopy,
    );
    let PolicyBridgeResolution::Selected(selected) =
        resolve_policy_bridge(&request, &[candidate], TransitionTypeExpectation::default())
    else {
        panic!("callable-owned mutability endpoints should be admissible");
    };
    assert_eq!(selected.callable.id, "const-compile-to-mut-runtime");
    assert_eq!(selected.complete_result_view.mode, PolicyMode::Mut);
}

fn mutability_transport_candidates() -> Vec<PolicyTransitionCallable<&'static str>> {
    [
        ("const<-const", PolicyMode::Const, PolicyMode::Const),
        ("const<-mut", PolicyMode::Mut, PolicyMode::Const),
        ("mut<-const", PolicyMode::Const, PolicyMode::Mut),
        ("mut<-mut", PolicyMode::Mut, PolicyMode::Mut),
    ]
    .into_iter()
    .map(|(id, input, output)| {
        callable_with_modes(
            id,
            OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
            OrdinaryCallableTypeOutput::SameAsInput,
            compile_runtime_pair(),
            input,
            compile_runtime_pair(),
            output,
            false,
            PolicyBridgeBody::BuiltinValueCopy,
        )
    })
    .collect()
}

#[test]
fn four_member_mutability_transport_uses_ordinary_actual_relative_preference() {
    let candidates = mutability_transport_candidates();
    for (source, target, expected) in [
        (PolicyMode::Const, PolicyMode::Const, "const<-const"),
        (PolicyMode::Const, PolicyMode::Mut, "mut<-const"),
        (PolicyMode::Mut, PolicyMode::Const, "const<-mut"),
        (PolicyMode::Mut, PolicyMode::Mut, "mut<-mut"),
    ] {
        let request = request_with_modes(compile_pair(), source, runtime_pair(), target);
        let PolicyBridgeResolution::Selected(selected) =
            resolve_policy_bridge(&request, &candidates, TransitionTypeExpectation::default())
        else {
            panic!("the exact input/output mutability member must be selected");
        };
        assert_eq!(selected.callable.id, expected);
        assert_eq!(
            selected.callable.input_policy,
            compile_runtime_pair(),
            "ordinary formal keeps its complete compile||runtime P2 while Project_in selects compile"
        );
        assert_eq!(
            selected.complete_result_view.pair,
            compile_runtime_pair(),
            "ordinary transport declares its complete compile||runtime P2"
        );
        assert_eq!(
            selected.validated_output_endpoint,
            runtime_pair(),
            "the demand selects the runtime Project_out view"
        );

        let produced = invoke_resolved_policy_bridge(
            &selected,
            &request,
            SemanticValueId(21),
            "ordinary-result-pattern",
        )
        .expect("ordinary transport fixture")
        .result;
        assert_eq!(
            produced.entry.view.pair.value.stages,
            compile_runtime_pair().value.stages,
            "the invocation fixture must preserve the complete ordinary result"
        );
        let assembled = assemble_transition_results(
            &[lang_build::PolicyTransitionDemand {
                request: request.clone(),
            }],
            &[produced],
        )
        .expect("Project_out must expose the demanded runtime view");
        assert_eq!(assembled[0].view.pair.value, runtime_pair().value);
        assert_eq!(assembled[0].view.mode, target);
    }
}

#[test]
fn opposite_mutability_endpoints_are_not_hard_inadmissible() {
    let request = request_with_modes(
        compile_pair(),
        PolicyMode::Const,
        runtime_pair(),
        PolicyMode::Const,
    );
    let opposite = callable_with_modes(
        "opposite-both-endpoints",
        OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
        OrdinaryCallableTypeOutput::SameAsInput,
        compile_pair(),
        PolicyMode::Mut,
        runtime_pair(),
        PolicyMode::Mut,
        false,
        PolicyBridgeBody::BuiltinValueCopy,
    );
    let PolicyBridgeResolution::Selected(selected) =
        resolve_policy_bridge(&request, &[opposite], TransitionTypeExpectation::default())
    else {
        panic!("opposite mutability Patterns belong to Bp preference, not hard applicability");
    };
    assert_eq!(selected.callable.id, "opposite-both-endpoints");
    assert_eq!(selected.complete_result_view.mode, PolicyMode::Mut);
}

#[test]
fn endpoint_mutability_orders_exact_then_unspecified_then_opposite() {
    let request = request_with_modes(
        compile_pair(),
        PolicyMode::Const,
        runtime_pair(),
        PolicyMode::Const,
    );
    let exact = callable_with_modes(
        "exact",
        OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
        OrdinaryCallableTypeOutput::SameAsInput,
        compile_pair(),
        PolicyMode::Const,
        runtime_pair(),
        PolicyMode::Const,
        false,
        PolicyBridgeBody::BuiltinValueCopy,
    );
    let unspecified = callable_with_modes(
        "unspecified",
        OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
        OrdinaryCallableTypeOutput::SameAsInput,
        compile_pair(),
        PolicyMode::Plain,
        runtime_pair(),
        PolicyMode::Plain,
        false,
        PolicyBridgeBody::BuiltinValueCopy,
    );
    let opposite = callable_with_modes(
        "opposite",
        OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
        OrdinaryCallableTypeOutput::SameAsInput,
        compile_pair(),
        PolicyMode::Mut,
        runtime_pair(),
        PolicyMode::Mut,
        false,
        PolicyBridgeBody::BuiltinValueCopy,
    );

    assert_eq!(
        compare_policy_transition_candidates(
            request.source_view(),
            request.target_demand(),
            &exact,
            &unspecified,
        ),
        PolicyPartialOrdering::Greater
    );
    assert_eq!(
        compare_policy_transition_candidates(
            request.source_view(),
            request.target_demand(),
            &unspecified,
            &opposite,
        ),
        PolicyPartialOrdering::Greater
    );
}

#[test]
fn explicit_mode_mismatch_requires_a_selected_migration() {
    let request = request_with_modes(
        compile_pair(),
        PolicyMode::Const,
        runtime_pair(),
        PolicyMode::Mut,
    );
    let candidate = callable_with_modes(
        "fresh-mut-runtime",
        OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
        OrdinaryCallableTypeOutput::SameAsInput,
        compile_pair(),
        PolicyMode::Const,
        runtime_pair(),
        PolicyMode::Mut,
        false,
        PolicyBridgeBody::BuiltinValueCopy,
    );
    assert!(matches!(
        resolve_policy_bridge(&request, &[candidate], TransitionTypeExpectation::default(),),
        PolicyBridgeResolution::Selected(_)
    ));
}

#[test]
fn runtime_transition_reports_non_runtime_target() {
    assert!(matches!(
        validate_runtime_transition(&meta_pair(), &compile_pair()),
        Err(PolicyTransitionFailure::TargetValueNotRuntime { .. })
    ));
}

#[test]
fn runtime_transition_requires_a_present_runtime_value_component() {
    let mut target = runtime_pair();
    target.value.presence = ValuePresence::Optional;
    assert!(matches!(
        validate_runtime_transition(&compile_pair(), &target),
        Err(PolicyTransitionFailure::TargetValueNotRuntime {
            target_value_presence: ValuePresence::Optional,
            ..
        })
    ));
}

#[test]
fn runtime_transition_rejects_unavailable_pattern_capability() {
    let target = pair(&[PolicyStage::Runtime], &[PolicyStage::Seal], &[]);
    assert!(matches!(
        validate_runtime_transition(&compile_pair(), &target),
        Err(PolicyTransitionFailure::PatternPolicyChanged { .. })
    ));
}

#[test]
fn runtime_transition_reports_value_pattern_overlap() {
    let target = pair(&[PolicyStage::Runtime], &[PolicyStage::Runtime], &[]);
    assert!(matches!(
        validate_runtime_transition(&compile_pair(), &target),
        Err(PolicyTransitionFailure::ValuePatternStageOverlap { .. })
    ));
}

#[test]
fn core_numeric_registry_uses_explicit_type_values_carried_by_installed_symbols() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("bootstrap world");
    let registry = NumericTypeRegistry::from_core_world(&world).expect("core numeric registry");
    for (key, name) in [
        (NumericTypeKey::new(NumericFamily::Uint, 8), "uint8"),
        (NumericTypeKey::new(NumericFamily::Uint, 16), "uint16"),
        (NumericTypeKey::new(NumericFamily::Uint, 32), "uint32"),
        (NumericTypeKey::new(NumericFamily::Float, 32), "float32"),
    ] {
        let symbol = world.resolve(name).expect("core numeric Type symbol");
        let SymbolPayload::Type(type_object) = &symbol.payload else {
            panic!("core numeric binding carries a type projection");
        };
        assert_eq!(registry.get(key), Some(type_object.represented_type));
    }
}

#[test]
fn atomic_registry_rejects_a_different_type_symbol_as_str() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("bootstrap world");
    assert!(
        world.resolve("str").is_err(),
        "current core bootstrap does not yet install str"
    );
    let uint8 = world.resolve("uint8").expect("installed uint8 Type symbol");
    let mut atomic_types = AtomicBuiltinTypeRegistry::new();
    assert_eq!(
        atomic_types.insert_resolved_type_symbol(AtomicBuiltinType::Str, &uint8),
        Err(AtomicBuiltinTypeRegistryFailure::SymbolNameMismatch {
            key: AtomicBuiltinType::Str,
            actual_name: "uint8".to_string()
        })
    );
}

#[test]
fn policy_migration_cannot_repair_type_pattern_structural_failure() {
    let request = request(compile_pair(), runtime_pair());
    let type_changing_candidate = callable(
        "implicit-ref-is-forbidden",
        OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
        OrdinaryCallableTypeOutput::Exact(TypeValueId(90)),
        compile_pair(),
        runtime_pair(),
        false,
        PolicyBridgeBody::IntrinsicStub("would construct ref::T".to_string()),
    );
    assert_eq!(
        resolve_policy_bridge(
            &request,
            &[type_changing_candidate],
            TransitionTypeExpectation {
                required_output_type: Some(TypeValueId(90)),
            },
        ),
        PolicyBridgeResolution::NoCandidate,
        "an atomic Policy migration cannot turn T into ref::T to repair applicability"
    );
}

#[test]
fn explicit_ref_mechanical_operation_remains_a_separate_structure_change_fixture() {
    let source = SemanticValueRef {
        id: SemanticValueId(20),
        type_value: TypeValueId(10),
    };
    let explicit_ref_result = PolicyResultEntry {
        value: Some(SemanticValueRef {
            id: SemanticValueId(21),
            type_value: TypeValueId(90),
        }),
        pattern: "ordinary-ref-result-pattern",
        view: PolicyView {
            pair: runtime_pair(),
            mode: PolicyMode::Const,
        },
    };
    assert_ne!(
        explicit_ref_result.value.unwrap().type_value,
        source.type_value,
        "an explicitly selected ordinary ref operation may return ref::T"
    );
    assert_eq!(explicit_ref_result.pattern, "ordinary-ref-result-pattern");
}

#[test]
fn prototype_migration_result_carrier_preserves_the_supplied_complete_entry() {
    let request = request(compile_pair(), runtime_pair());
    let PolicyBridgeResolution::Selected(selected) = resolve_policy_bridge(
        &request,
        &[exact_copy("same-type-runtime-copy")],
        TransitionTypeExpectation::default(),
    ) else {
        panic!("same-type atomic runtime migration should select");
    };
    let produced = invoke_resolved_policy_bridge(
        &selected,
        &request,
        SemanticValueId(21),
        "fixture-result-pattern",
    )
    .expect("prototype body")
    .result;
    assert_eq!(
        produced.entry.value.unwrap().type_value,
        request.source_type()
    );
    assert_eq!(produced.entry.pattern, "fixture-result-pattern");

    let demand = lang_build::PolicyTransitionDemand {
        request: request.clone(),
    };
    let assembled =
        assemble_transition_results(&[demand], &[produced]).expect("project output view");
    assert_eq!(
        assembled[0].view.pair.value.stages,
        runtime_pair().value.stages
    );
}

#[test]
fn atomic_mut_runtime_migration_keeps_delete_as_selected_rejection() {
    let target = runtime_pair_with(PolicyMode::Mut);
    let request = request(compile_pair(), target.clone());
    let candidates = vec![callable(
        "mut-runtime-delete",
        OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
        OrdinaryCallableTypeOutput::SameAsInput,
        compile_pair(),
        target,
        true,
        PolicyBridgeBody::IntrinsicStub("deleted mut runtime migration".to_string()),
    )];
    assert_eq!(
        resolve_policy_bridge(&request, &candidates, TransitionTypeExpectation::default(),),
        PolicyBridgeResolution::RejectedByDelete("mut-runtime-delete")
    );
}

#[test]
fn ref_specific_delete_uses_b3_after_equal_policy_endpoints() {
    let source = compile_pair_with(PolicyMode::Const);
    let target = runtime_pair_with(PolicyMode::Mut);
    let request = PolicyTransitionRequest::new(
        PolicyView {
            pair: source.clone(),
            mode: PolicyMode::Const,
        },
        ResultPolicyDemand {
            pair_query: P1Projection::Pair(target.clone()),
            mode: PolicyMode::Mut,
        },
        TypeValueId(90),
        SemanticValueId(20),
        Provenance::new("ref materialization safety overload"),
    )
    .unwrap();
    let generic = callable(
        "generic-materialize",
        OrdinaryCallableTypeInput::Exact(TypeValueId(90)),
        OrdinaryCallableTypeOutput::SameAsInput,
        source.clone(),
        target.clone(),
        false,
        PolicyBridgeBody::BuiltinValueCopy,
    );
    let mut ref_specific_delete = callable(
        "ref-specific-delete",
        OrdinaryCallableTypeInput::Exact(TypeValueId(90)),
        OrdinaryCallableTypeOutput::SameAsInput,
        source,
        target,
        true,
        PolicyBridgeBody::IntrinsicStub("deleted ref materialization".to_string()),
    );
    ref_specific_delete.prototype_pattern_specificity = 100;

    assert_eq!(
        resolve_policy_bridge(
            &request,
            &[generic, ref_specific_delete],
            TransitionTypeExpectation::default(),
        ),
        PolicyBridgeResolution::RejectedByDelete("ref-specific-delete"),
        "equal endpoint Policy leaves both candidates for the B3 stand-in"
    );
}

#[test]
fn fallback_survives_only_when_no_admissible_non_fallback_exists() {
    let request = request(
        compile_pair_with(PolicyMode::Const),
        runtime_pair_with(PolicyMode::Mut),
    );
    let mut fallback = callable(
        "default-transport",
        OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
        OrdinaryCallableTypeOutput::SameAsInput,
        compile_runtime_pair_with(PolicyMode::Const),
        compile_runtime_pair_with(PolicyMode::Mut),
        false,
        PolicyBridgeBody::BuiltinValueCopy,
    );
    fallback.prototype_is_fallback = true;

    let PolicyBridgeResolution::Selected(selected) =
        resolve_policy_bridge(&request, &[fallback], TransitionTypeExpectation::default())
    else {
        panic!("fallback candidates remain when no admissible non-fallback exists");
    };
    assert_eq!(selected.callable.id, "default-transport");
}

#[test]
fn admissible_delete_suppresses_fallback_before_endpoint_policy_order() {
    let request = request(
        compile_pair_with(PolicyMode::Const),
        runtime_pair_with(PolicyMode::Mut),
    );
    let mut better_policy_fallback = callable(
        "better-policy-fallback",
        OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
        OrdinaryCallableTypeOutput::SameAsInput,
        compile_runtime_pair_with(PolicyMode::Const),
        compile_runtime_pair_with(PolicyMode::Mut),
        false,
        PolicyBridgeBody::BuiltinValueCopy,
    );
    better_policy_fallback.prototype_is_fallback = true;

    let worse_policy_delete = callable(
        "ordinary-delete",
        OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
        OrdinaryCallableTypeOutput::SameAsInput,
        compile_runtime_pair_with(PolicyMode::Mut),
        compile_runtime_pair_with(PolicyMode::Const),
        true,
        PolicyBridgeBody::IntrinsicStub("delete suppresses fallback".to_string()),
    );

    assert_eq!(
        resolve_policy_bridge(
            &request,
            &[better_policy_fallback, worse_policy_delete],
            TransitionTypeExpectation::default(),
        ),
        PolicyBridgeResolution::RejectedByDelete("ordinary-delete"),
        "an admissible non-fallback delete removes fallback before Bp' and later rejection cannot reopen it"
    );
}

#[test]
fn fine_delete_refines_one_cell_of_the_fallback_transport_quartet() {
    for (source, target, expected_default) in [
        (PolicyMode::Const, PolicyMode::Const, "const<-const"),
        (PolicyMode::Const, PolicyMode::Mut, "mut<-const"),
        (PolicyMode::Mut, PolicyMode::Const, "const<-mut"),
        (PolicyMode::Mut, PolicyMode::Mut, "mut<-mut"),
    ] {
        let mut candidates = mutability_transport_candidates();
        for candidate in &mut candidates {
            candidate.prototype_is_fallback = true;
        }

        let fine_delete_applies = source == PolicyMode::Const && target == PolicyMode::Mut;
        let mut fine_delete = callable_with_modes(
            "fine-const-to-mut-delete",
            OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
            OrdinaryCallableTypeOutput::SameAsInput,
            compile_runtime_pair(),
            PolicyMode::Const,
            compile_runtime_pair(),
            PolicyMode::Mut,
            true,
            PolicyBridgeBody::IntrinsicStub(
                "fine structural region rejects const-to-mut materialization".to_string(),
            ),
        );
        // This fixture stands in for an additional structural/require
        // applicability condition. Endpoint mutability remains preference,
        // rather than becoming hard compatibility merely to isolate one cell.
        fine_delete.ordinary_fully_admissible = fine_delete_applies;
        candidates.push(fine_delete);

        let request = request_with_modes(compile_pair(), source, runtime_pair(), target);
        let resolution =
            resolve_policy_bridge(&request, &candidates, TransitionTypeExpectation::default());

        if fine_delete_applies {
            assert_eq!(
                resolution,
                PolicyBridgeResolution::RejectedByDelete("fine-const-to-mut-delete"),
                "an admissible fine non-fallback delete suppresses the default member for its cell"
            );
        } else {
            let PolicyBridgeResolution::Selected(selected) = resolution else {
                panic!("the fallback quartet must continue to fill uncovered cells");
            };
            assert_eq!(
                selected.callable.id, expected_default,
                "a fine member outside A must not suppress fallback for another source/target cell"
            );
        }
    }
}

#[test]
fn bridge_resolution_does_not_search_transitive_paths() {
    let request = request(meta_pair(), runtime_meta_pair());
    let candidates = vec![
        callable(
            "meta-to-compile",
            OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
            OrdinaryCallableTypeOutput::SameAsInput,
            meta_pair(),
            compile_pair(),
            false,
            PolicyBridgeBody::UserCallable(lang_build::SymbolId(1)),
        ),
        callable(
            "compile-to-runtime",
            OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
            OrdinaryCallableTypeOutput::SameAsInput,
            compile_pair(),
            runtime_meta_pair(),
            false,
            PolicyBridgeBody::UserCallable(lang_build::SymbolId(2)),
        ),
    ];
    assert_eq!(
        resolve_policy_bridge(&request, &candidates, TransitionTypeExpectation::default()),
        PolicyBridgeResolution::NoCandidate
    );
}

#[test]
fn candidate_input_can_project_existing_slice_for_a_transition_demand() {
    let request = request(broad_static_pair(), runtime_with_broad_static_pattern());
    let candidates = vec![
        callable(
            "broad-input",
            OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
            OrdinaryCallableTypeOutput::SameAsInput,
            broad_static_pair(),
            runtime_with_broad_static_pattern(),
            false,
            PolicyBridgeBody::BuiltinValueCopy,
        ),
        exact_copy("compile-slice-to-runtime"),
    ];
    let PolicyBridgeResolution::Selected(selected) =
        resolve_policy_bridge(&request, &candidates, TransitionTypeExpectation::default())
    else {
        panic!("compile input slice must be preferred");
    };
    assert_eq!(selected.callable.id, "compile-slice-to-runtime");
}

#[test]
fn output_policy_participates_in_transition_preference() {
    let request = request(compile_pair(), runtime_pair());
    let exact = exact_copy("exact-output");
    let wide = callable(
        "wide-output",
        OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
        OrdinaryCallableTypeOutput::SameAsInput,
        compile_pair(),
        broad_runtime_output(),
        false,
        PolicyBridgeBody::BuiltinValueCopy,
    );
    assert_eq!(
        compare_policy_transition_candidates(
            request.source_view(),
            request.target_demand(),
            &exact,
            &wide,
        ),
        PolicyPartialOrdering::Greater
    );
    let PolicyBridgeResolution::Selected(selected) = resolve_policy_bridge(
        &request,
        &[wide, exact],
        TransitionTypeExpectation::default(),
    ) else {
        panic!("exact output Policy must win");
    };
    assert_eq!(selected.callable.id, "exact-output");

    let input_only = vec![
        PolicyOverloadCandidate {
            id: "wide-output",
            formal_frame: MutabilityFormalFrame {
                self_pattern: MutabilityPattern::Plain,
                explicit_parameter_patterns: vec![],
            },
            result_policy: PolicyMode::Plain,
            is_delete: false,
        },
        PolicyOverloadCandidate {
            id: "exact-output",
            formal_frame: MutabilityFormalFrame {
                self_pattern: MutabilityPattern::Plain,
                explicit_parameter_patterns: vec![],
            },
            result_policy: PolicyMode::Plain,
            is_delete: false,
        },
    ];
    assert!(matches!(
        lang_build::select_by_mutability_product(
            &input_only,
            &MutabilityActualFrame {
                caller_value: PolicyMode::Const,
                explicit_arguments: vec![],
            },
            OutputModeDemand::default(),
        ),
        PolicyOverloadSelection::Ambiguous(_)
    ));
}

#[test]
fn prototype_endpoint_policy_order_precedes_pattern_specificity_stand_in() {
    let request = request(compile_pair(), runtime_pair());
    let mut better_policy_generic_pattern = callable(
        "better-policy-generic-pattern",
        OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
        OrdinaryCallableTypeOutput::SameAsInput,
        compile_pair(),
        runtime_pair(),
        false,
        PolicyBridgeBody::BuiltinValueCopy,
    );
    better_policy_generic_pattern.prototype_pattern_specificity = 0;
    let mut worse_policy_specific_pattern = callable(
        "worse-policy-specific-pattern",
        OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
        OrdinaryCallableTypeOutput::SameAsInput,
        broad_static_pair(),
        broad_runtime_output(),
        false,
        PolicyBridgeBody::BuiltinValueCopy,
    );
    worse_policy_specific_pattern.prototype_pattern_specificity = 100;

    let candidates = vec![worse_policy_specific_pattern, better_policy_generic_pattern];
    let PolicyBridgeResolution::Selected(selected) =
        resolve_policy_bridge(&request, &candidates, TransitionTypeExpectation::default())
    else {
        panic!("the endpoint-only prototype must run before its B3 stand-in");
    };
    assert_eq!(selected.callable.id, "better-policy-generic-pattern");
}

fn crossed_policy_candidates() -> Vec<PolicyTransitionCallable<&'static str>> {
    vec![
        callable(
            "better-input",
            OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
            OrdinaryCallableTypeOutput::SameAsInput,
            compile_pair(),
            broad_runtime_output(),
            false,
            PolicyBridgeBody::BuiltinValueCopy,
        ),
        callable(
            "better-output",
            OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
            OrdinaryCallableTypeOutput::SameAsInput,
            broad_static_pair(),
            runtime_pair(),
            false,
            PolicyBridgeBody::BuiltinValueCopy,
        ),
    ]
}

#[test]
fn input_output_policy_tradeoff_is_incomparable_ambiguity() {
    let request = request(compile_pair(), runtime_pair());
    let candidates = crossed_policy_candidates();
    assert_eq!(
        compare_policy_transition_candidates(
            request.source_view(),
            request.target_demand(),
            &candidates[0],
            &candidates[1],
        ),
        PolicyPartialOrdering::Incomparable
    );
    assert!(matches!(
        resolve_policy_bridge(&request, &candidates, TransitionTypeExpectation::default()),
        PolicyBridgeResolution::Ambiguous(_)
    ));
}

#[test]
fn transition_ambiguity_is_declaration_order_invariant() {
    let request = request(compile_pair(), runtime_pair());
    let mut candidates = crossed_policy_candidates();
    let first = resolve_policy_bridge(&request, &candidates, TransitionTypeExpectation::default());
    candidates.reverse();
    let second = resolve_policy_bridge(&request, &candidates, TransitionTypeExpectation::default());
    assert!(matches!(first, PolicyBridgeResolution::Ambiguous(_)));
    assert!(matches!(second, PolicyBridgeResolution::Ambiguous(_)));
}

#[test]
fn output_type_does_not_drive_ordinary_type_preference() {
    let request = request(compile_pair(), runtime_pair());
    let candidates = vec![
        callable(
            "returns-a",
            OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
            OrdinaryCallableTypeOutput::SameAsInput,
            compile_pair(),
            runtime_pair(),
            false,
            PolicyBridgeBody::BuiltinValueCopy,
        ),
        callable(
            "returns-b",
            OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
            OrdinaryCallableTypeOutput::Exact(TypeValueId(10)),
            compile_pair(),
            runtime_pair(),
            false,
            PolicyBridgeBody::BuiltinValueCopy,
        ),
    ];
    assert!(matches!(
        resolve_policy_bridge(&request, &candidates, TransitionTypeExpectation::default()),
        PolicyBridgeResolution::Ambiguous(_)
    ));
}

fn outer_candidate(
    id: &'static str,
    fully_admissible: bool,
) -> PhaseOverloadCandidate<&'static str> {
    PhaseOverloadCandidate {
        candidate: PolicyOverloadCandidate {
            id,
            formal_frame: MutabilityFormalFrame {
                self_pattern: MutabilityPattern::Plain,
                explicit_parameter_patterns: vec![],
            },
            result_policy: PolicyMode::Plain,
            is_delete: false,
        },
        stage: PolicyStage::Compile,
        fully_admissible,
    }
}

#[test]
fn bridge_existence_is_checked_before_outer_winner_and_failure_cannot_backtrack() {
    let request = request(compile_pair(), runtime_pair());
    let available = vec![exact_copy("selected-bridge")];
    let missing: Vec<PolicyTransitionCallable<&str>> = Vec::new();
    let missing_qualification =
        qualify_policy_bridge(&request, &missing, TransitionTypeExpectation::default());
    let available_qualification =
        qualify_policy_bridge(&request, &available, TransitionTypeExpectation::default());
    let outer = vec![
        outer_candidate(
            "needs-missing-bridge",
            matches!(missing_qualification, BridgeQualification::Available(_)),
        ),
        outer_candidate(
            "has-bridge",
            matches!(available_qualification, BridgeQualification::Available(_)),
        ),
    ];
    assert_eq!(
        select_policy_overload(
            &outer,
            &MutabilityActualFrame {
                caller_value: PolicyMode::Const,
                explicit_arguments: vec![],
            },
            OutputModeDemand::default(),
            Phase::OpenStatic,
        ),
        PolicyOverloadSelection::Selected("has-bridge")
    );

    let failing = vec![
        callable(
            "winner",
            OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
            OrdinaryCallableTypeOutput::SameAsInput,
            compile_pair(),
            runtime_pair(),
            false,
            PolicyBridgeBody::FailAfterSelection("lowering failed".to_string()),
        ),
        callable(
            "former-runner-up",
            OrdinaryCallableTypeInput::Any,
            OrdinaryCallableTypeOutput::SameAsInput,
            broad_static_pair(),
            broad_runtime_output(),
            false,
            PolicyBridgeBody::BuiltinValueCopy,
        ),
    ];
    let PolicyBridgeResolution::Selected(selected) =
        resolve_policy_bridge(&request, &failing, TransitionTypeExpectation::default())
    else {
        panic!("exact winner must be selected once");
    };
    let failure =
        invoke_resolved_policy_bridge(&selected, &request, SemanticValueId(21), "failed-pattern")
            .expect_err("selected prototype failure");
    assert_eq!(failure.selected_callable_id, "winner");
}

#[test]
fn selected_delete_bridge_rejects_outer_candidate_instead_of_qualifying_it() {
    let request = request(compile_pair(), runtime_pair_with(PolicyMode::Mut));
    let deleted = vec![callable(
        "deleted-transition",
        OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
        OrdinaryCallableTypeOutput::SameAsInput,
        compile_pair(),
        runtime_pair_with(PolicyMode::Mut),
        true,
        PolicyBridgeBody::IntrinsicStub("deleted".to_string()),
    )];
    let available = vec![callable(
        "available-transition",
        OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
        OrdinaryCallableTypeOutput::SameAsInput,
        compile_pair(),
        runtime_pair_with(PolicyMode::Mut),
        false,
        PolicyBridgeBody::BuiltinValueCopy,
    )];

    let rejected = qualify_policy_bridge(&request, &deleted, TransitionTypeExpectation::default());
    let qualified =
        qualify_policy_bridge(&request, &available, TransitionTypeExpectation::default());
    assert_eq!(
        rejected,
        BridgeQualification::RejectedByDelete("deleted-transition")
    );
    assert!(matches!(&qualified, BridgeQualification::Available(_)));

    let outer = vec![
        outer_candidate(
            "requires-deleted-transition",
            matches!(&rejected, BridgeQualification::Available(_)),
        ),
        outer_candidate(
            "requires-available-transition",
            matches!(&qualified, BridgeQualification::Available(_)),
        ),
    ];
    assert_eq!(
        select_policy_overload(
            &outer,
            &MutabilityActualFrame {
                caller_value: PolicyMode::Const,
                explicit_arguments: vec![],
            },
            OutputModeDemand::default(),
            Phase::OpenStatic,
        ),
        PolicyOverloadSelection::Selected("requires-available-transition")
    );
}
