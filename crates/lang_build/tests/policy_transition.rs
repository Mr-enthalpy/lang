mod support;

use std::convert::Infallible;

use lang_build::{
    assemble_transition_results, compare_policy_transition_candidates,
    elaborate_pure_type_binding_p1, elaborate_value_binding_p1, evaluate_initializer_best_effort,
    invoke_resolved_policy_bridge, materialize_literal_value, policy_bridge_is_available,
    resolve_policy_bridge, select_policy_overload, type_value_projection_from_type_symbol,
    validate_runtime_transition, AtomicBuiltinFamily, CompilationWorld, EvalMode, EvalOutcome,
    LiteralFamily, LiteralMaterializationFailure, LiteralTypeSelection, MutabilityActualFrame,
    MutabilityFormalFrame, MutabilityPattern, NumericFamily, NumericTypeKey, NumericTypeRegistry,
    OrdinaryCallableTypeInput, OrdinaryCallableTypeOutput, P1Elaboration, P1ElaborationFailure,
    P1Origin, P1Projection, PatternComponentPolicy, Phase, PhaseOverloadCandidate,
    PolicyBridgeBody, PolicyBridgeResolution, PolicyOverloadCandidate, PolicyOverloadSelection,
    PolicyPair, PolicyPartialOrdering, PolicyResultEntry, PolicyStage, PolicyTransitionCallable,
    PolicyTransitionFailure, PolicyTransitionRequest, PolicyTransitionRequestFailure, Provenance,
    ResidualReason, SemanticValueId, SemanticValueRef, StageSet, TransitionTypeExpectation,
    TypeValueId, ValueComponentPolicy, ValueMutability, ValuePresence,
};
use support::{empty_app_manifest, initializer_from_source};

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
    mutability: &[ValueMutability],
) -> PolicyPair {
    PolicyPair {
        value: ValueComponentPolicy {
            stages: stages(value_stages),
            mutability: mutability.iter().copied().collect(),
            presence: ValuePresence::Present,
        },
        pattern: PatternComponentPolicy {
            stages: stages(pattern_stages),
        },
        namespace_visibility: None,
        export_root: false,
    }
}

fn absent_pair(pattern_stages: &[PolicyStage]) -> PolicyPair {
    PolicyPair {
        value: ValueComponentPolicy {
            stages: StageSet::new(),
            mutability: Default::default(),
            presence: ValuePresence::Absent,
        },
        pattern: PatternComponentPolicy {
            stages: stages(pattern_stages),
        },
        namespace_visibility: None,
        export_root: false,
    }
}

fn compile_pair() -> PolicyPair {
    pair(&[PolicyStage::Compile], &[PolicyStage::Compile], &[])
}

fn meta_pair() -> PolicyPair {
    pair(&[PolicyStage::Meta], &[PolicyStage::Meta], &[])
}

fn runtime_pair() -> PolicyPair {
    pair(&[PolicyStage::Runtime], &[PolicyStage::Compile], &[])
}

fn compile_runtime_pair() -> PolicyPair {
    pair(
        &[PolicyStage::Compile, PolicyStage::Runtime],
        &[PolicyStage::Compile],
        &[],
    )
}

fn runtime_pair_with(mutability: ValueMutability) -> PolicyPair {
    pair(
        &[PolicyStage::Runtime],
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

fn multi_slice_static_value_pair() -> PolicyPair {
    pair(
        &[PolicyStage::Meta, PolicyStage::Compile],
        &[PolicyStage::Compile],
        &[],
    )
}

fn broad_runtime_output() -> PolicyPair {
    compile_runtime_pair()
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
        value_policy: policy.value,
        pattern: "pattern",
        pattern_policy: policy.pattern,
    }
}

fn pure_entry(pattern_stages: &[PolicyStage]) -> PolicyResultEntry<Infallible, &'static str> {
    let policy = absent_pair(pattern_stages);
    PolicyResultEntry {
        value: None,
        value_policy: policy.value,
        pattern: "type-pattern",
        pattern_policy: policy.pattern,
    }
}

fn request(source_policy: PolicyPair, target_policy: PolicyPair) -> PolicyTransitionRequest {
    PolicyTransitionRequest::new(
        source_policy,
        target_policy,
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
    PolicyTransitionCallable {
        id,
        input_type,
        output_type,
        input_policy,
        output_policy,
        ordinary_fully_admissible: true,
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
        selected[0].value_policy.stages,
        StageSet::from([PolicyStage::Runtime]),
        "ordinary binding must not copy the function-object P1 stage lift"
    );
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
        selected[0].value_policy.stages,
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
        selected[0].value_policy.stages,
        StageSet::from([PolicyStage::Compile])
    );
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
    let P1Elaboration::Transition { requested, demands } = elaboration else {
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
fn transition_output_satisfies_the_p1_query_by_non_empty_projection() {
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
    let P1Elaboration::Transition { demands, .. } = elaborate_value_binding_p1(
        &result,
        Some(&query),
        Provenance::new("meta || runtime query over compile"),
    )
    .unwrap() else {
        panic!("the compile source has no slice selected by this query");
    };

    let candidate = exact_copy("compile-to-runtime");
    let PolicyBridgeResolution::Selected(selected) = resolve_policy_bridge(
        &demands[0].request,
        &[candidate],
        TransitionTypeExpectation::default(),
    ) else {
        panic!("a runtime output is one valid answer to meta || runtime");
    };
    assert_eq!(selected.result_policy, runtime_pair());
    validate_runtime_transition(demands[0].request.source_policy(), &selected.result_policy)
        .expect("validate the selected output slice, not the broad P1 query");

    let produced =
        invoke_resolved_policy_bridge(&selected, &demands[0].request, SemanticValueId(21))
            .expect("prototype invocation")
            .value;
    let assembled = assemble_transition_results(&demands, &[produced]).unwrap();
    assert_eq!(assembled.len(), 1);
    assert_eq!(assembled[0].value.unwrap().id, SemanticValueId(21));
    assert_eq!(
        assembled[0].value_policy.stages,
        runtime_pair().value.stages
    );
    assert_ne!(
        assembled[0].value_policy.stages, broad_query_pair.value.stages,
        "the query alternatives are not manufactured as an exact output domain"
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
    let P1Elaboration::Transition { demands, .. } = elaborate_value_binding_p1(
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
fn pure_type_can_project_pattern_slice_without_value_identity_or_transition_api() {
    let result = vec![pure_entry(&[PolicyStage::Meta, PolicyStage::Compile])];
    let target = P1Projection::Pair(absent_pair(&[PolicyStage::Compile]));
    let elaboration =
        elaborate_pure_type_binding_p1(&result, Some(&target)).expect("pure compile slice");
    assert_eq!(elaboration.selected.len(), 1);
    assert_eq!(elaboration.selected[0].value, None);
    assert_eq!(
        elaboration.selected[0].pattern_policy.stages,
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
fn value_elaborator_rejects_absent_entries_before_transition_construction() {
    let result = vec![PolicyResultEntry {
        value: None,
        value_policy: absent_pair(&[PolicyStage::Compile]).value,
        pattern: "not-value-bearing",
        pattern_policy: absent_pair(&[PolicyStage::Compile]).pattern,
    }];
    let target = P1Projection::Pair(runtime_pair());
    assert_eq!(
        elaborate_value_binding_p1(
            &result,
            Some(&target),
            Provenance::new("invalid value-bearing input")
        ),
        Err(P1ElaborationFailure::ValueBearingInputContainsAbsentValue)
    );
}

#[test]
fn absent_source_cannot_construct_or_validate_a_transition() {
    assert_eq!(
        PolicyTransitionRequest::new(
            absent_pair(&[PolicyStage::Compile]),
            runtime_pair(),
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
fn legal_runtime_value_transition_preserves_pattern_policy() {
    assert_eq!(
        validate_runtime_transition(&compile_pair(), &runtime_pair()),
        Ok(())
    );
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
fn runtime_transition_reports_pattern_change() {
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
fn core_numeric_registry_uses_canonical_concrete_type_symbols() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("bootstrap world");
    let registry = NumericTypeRegistry::from_core_world(&world).expect("core numeric registry");
    for (key, name) in [
        (NumericTypeKey::new(NumericFamily::Uint, 8), "uint8"),
        (NumericTypeKey::new(NumericFamily::Uint, 16), "uint16"),
        (NumericTypeKey::new(NumericFamily::Uint, 32), "uint32"),
        (NumericTypeKey::new(NumericFamily::Float, 32), "float32"),
    ] {
        let symbol = world.resolve(name).expect("core numeric Type symbol");
        assert_eq!(
            registry.get(key),
            Some(type_value_projection_from_type_symbol(symbol.id))
        );
    }
}

#[test]
fn literal_family_is_distinct_from_selected_concrete_numeric_type() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("bootstrap world");
    let registry = NumericTypeRegistry::from_core_world(&world).expect("numeric registry");
    let uint16 = NumericTypeKey::new(NumericFamily::Uint, 16);
    let expected = registry.get(uint16).expect("canonical core uint16");
    let expr = initializer_from_source("let x = 42");
    let literal = materialize_literal_value(
        &expr,
        &registry,
        LiteralTypeSelection::Numeric(uint16),
        SemanticValueId(30),
        Provenance::new("42"),
    )
    .expect("concrete Tnum selected");
    assert_eq!(literal.literal_family, LiteralFamily::Integer);
    assert_eq!(literal.numeric_type, Some(uint16));
    assert_eq!(literal.type_value, expected);
    assert_eq!(literal.policy, compile_pair());
}

#[test]
fn literal_helper_is_not_yet_wired_into_the_initializer_evaluator() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("bootstrap world");
    let expr = initializer_from_source("let x = 42");
    assert!(matches!(
        evaluate_initializer_best_effort(
            world.snapshot(),
            world.package_root_node(),
            &expr,
            &world.package_context(),
            EvalMode::MetaPartial,
            Provenance::new("literal integration boundary"),
        ),
        EvalOutcome::Residual {
            reason: ResidualReason::UnsupportedExpression,
            ..
        }
    ));
}

#[test]
fn numeric_literal_cannot_use_a_family_as_its_concrete_type() {
    let expr = initializer_from_source("let x = 42");
    assert!(matches!(
        materialize_literal_value(
            &expr,
            &NumericTypeRegistry::new(),
            LiteralTypeSelection::Atomic {
                family: AtomicBuiltinFamily::Int,
                type_value: TypeValueId(700),
            },
            SemanticValueId(30),
            Provenance::new("42"),
        ),
        Err(
            LiteralMaterializationFailure::AtomicNumericFamilyIsNotConcrete {
                family: AtomicBuiltinFamily::Int
            }
        )
    ));
}

#[test]
fn string_literal_is_compile_str_value_not_ref() {
    let expr = initializer_from_source("let s = \"abc\"");
    let literal = materialize_literal_value(
        &expr,
        &NumericTypeRegistry::new(),
        LiteralTypeSelection::Atomic {
            family: AtomicBuiltinFamily::Str,
            type_value: TypeValueId(5),
        },
        SemanticValueId(30),
        Provenance::new("\"abc\""),
    )
    .expect("string literal");
    assert_eq!(literal.literal_family, LiteralFamily::String);
    assert_eq!(literal.numeric_type, None);
    assert_eq!(literal.type_value, TypeValueId(5));
    assert_ne!(literal.type_value, TypeValueId(900), "dependent str ref");
    assert_eq!(literal.policy, compile_pair());
}

#[test]
fn const_ref_bridge_selects_unique_non_delete_candidate() {
    let target = runtime_pair_with(ValueMutability::Const);
    let request = PolicyTransitionRequest::new(
        compile_pair(),
        target.clone(),
        TypeValueId(5),
        SemanticValueId(20),
        Provenance::new("const ref transition"),
    )
    .unwrap();
    let candidates = vec![
        callable(
            "const-ref",
            OrdinaryCallableTypeInput::Exact(TypeValueId(5)),
            OrdinaryCallableTypeOutput::Exact(TypeValueId(90)),
            compile_pair(),
            target,
            false,
            PolicyBridgeBody::IntrinsicStub("materialize const ref storage".to_string()),
        ),
        callable(
            "wide",
            OrdinaryCallableTypeInput::Any,
            OrdinaryCallableTypeOutput::Exact(TypeValueId(90)),
            broad_static_pair(),
            broad_runtime_output(),
            false,
            PolicyBridgeBody::IntrinsicStub("wide ref".to_string()),
        ),
    ];
    let PolicyBridgeResolution::Selected(selected) = resolve_policy_bridge(
        &request,
        &candidates,
        TransitionTypeExpectation {
            required_output_type: Some(TypeValueId(90)),
        },
    ) else {
        panic!("const ref candidate must select");
    };
    assert_eq!(selected.callable.id, "const-ref");
}

#[test]
fn mut_ref_bridge_keeps_delete_as_selected_rejection() {
    let target = runtime_pair_with(ValueMutability::Mut);
    let request = request(compile_pair(), target.clone());
    let candidates = vec![callable(
        "mut-ref-delete",
        OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
        OrdinaryCallableTypeOutput::Exact(TypeValueId(90)),
        compile_pair(),
        target,
        true,
        PolicyBridgeBody::IntrinsicStub("deleted mut ref".to_string()),
    )];
    assert_eq!(
        resolve_policy_bridge(
            &request,
            &candidates,
            TransitionTypeExpectation {
                required_output_type: Some(TypeValueId(90)),
            },
        ),
        PolicyBridgeResolution::RejectedByDelete("mut-ref-delete")
    );
}

#[test]
fn bridge_resolution_does_not_search_transitive_paths() {
    let request = request(meta_pair(), runtime_pair());
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
            runtime_pair(),
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
    let request = request(multi_slice_static_value_pair(), runtime_pair());
    let candidates = vec![
        callable(
            "broad-input",
            OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
            OrdinaryCallableTypeOutput::SameAsInput,
            multi_slice_static_value_pair(),
            runtime_pair(),
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
            request.source_policy(),
            request.target_query(),
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
                self_pattern: MutabilityPattern::Unspecified,
                explicit_parameter_patterns: vec![],
            },
            result_policy: None,
            is_delete: false,
        },
        PolicyOverloadCandidate {
            id: "exact-output",
            formal_frame: MutabilityFormalFrame {
                self_pattern: MutabilityPattern::Unspecified,
                explicit_parameter_patterns: vec![],
            },
            result_policy: None,
            is_delete: false,
        },
    ];
    assert!(matches!(
        lang_build::select_by_mutability_product(
            &input_only,
            &MutabilityActualFrame {
                caller_value: ValueMutability::Const,
                explicit_arguments: vec![],
            },
            None,
        ),
        PolicyOverloadSelection::Ambiguous(_)
    ));
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
            request.source_policy(),
            request.target_query(),
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
            OrdinaryCallableTypeOutput::Exact(TypeValueId(101)),
            compile_pair(),
            runtime_pair(),
            false,
            PolicyBridgeBody::BuiltinValueCopy,
        ),
        callable(
            "returns-b",
            OrdinaryCallableTypeInput::Exact(TypeValueId(10)),
            OrdinaryCallableTypeOutput::Exact(TypeValueId(102)),
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
                self_pattern: MutabilityPattern::Unspecified,
                explicit_parameter_patterns: vec![],
            },
            result_policy: None,
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
    let outer = vec![
        outer_candidate(
            "needs-missing-bridge",
            policy_bridge_is_available(&request, &missing, TransitionTypeExpectation::default()),
        ),
        outer_candidate(
            "has-bridge",
            policy_bridge_is_available(&request, &available, TransitionTypeExpectation::default()),
        ),
    ];
    assert_eq!(
        select_policy_overload(
            &outer,
            &MutabilityActualFrame {
                caller_value: ValueMutability::Const,
                explicit_arguments: vec![],
            },
            None,
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
    let failure = invoke_resolved_policy_bridge(&selected, &request, SemanticValueId(21))
        .expect_err("selected prototype failure");
    assert_eq!(failure.selected_callable_id, "winner");
}
