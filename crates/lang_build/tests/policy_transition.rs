mod support;

use lang_build::{
    compare_policy_transition_candidates, default_p1, elaborate_value_binding_p1,
    invoke_resolved_policy_bridge, materialize_literal_value, policy_bridge_is_available,
    resolve_policy_bridge, select_policy_overload, validate_runtime_transition, AtomicBuiltinType,
    AtomicBuiltinTypeIds, ExistingPolicySlice, MutabilityActualFrame, MutabilityFormalFrame,
    MutabilityPattern, OrdinaryCallableTypeInput, OrdinaryCallableTypeOutput, P1Elaboration,
    P1Origin, PatternComponentPolicy, Phase, PhaseOverloadCandidate, PolicyBridgeBody,
    PolicyBridgeEffect, PolicyBridgeResolution, PolicyOverloadCandidate, PolicyOverloadSelection,
    PolicyPair, PolicyPartialOrdering, PolicyStage, PolicyTransitionCallable,
    PolicyTransitionFailure, PolicyTransitionRequest, Provenance, SemanticValueId, StageSet,
    TransitionTypeExpectation, TypeValueId, ValueComponentPolicy, ValueMutability, ValuePresence,
};
use support::initializer_from_source;

fn pair(
    value_stages: &[PolicyStage],
    pattern_stages: &[PolicyStage],
    mutability: &[ValueMutability],
) -> PolicyPair {
    let mut value_stage_set = StageSet::new();
    for stage in value_stages {
        value_stage_set.insert(*stage);
    }
    PolicyPair {
        value: ValueComponentPolicy {
            stages: value_stage_set,
            mutability: mutability.iter().copied().collect(),
            presence: ValuePresence::Present,
        },
        pattern: PatternComponentPolicy {
            stages: {
                let mut stages = StageSet::new();
                for stage in pattern_stages {
                    stages.insert(*stage);
                }
                stages
            },
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
    pair(
        &[PolicyStage::Compile, PolicyStage::Runtime],
        &[PolicyStage::Compile],
        &[],
    )
}

fn request(source_policy: PolicyPair, target_policy: PolicyPair) -> PolicyTransitionRequest {
    PolicyTransitionRequest {
        source_policy,
        target_policy,
        source_type: TypeValueId(10),
        source_value: SemanticValueId(20),
        provenance: Provenance::new("transition request"),
    }
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
fn omitted_p1_uses_default_without_transition() {
    let p2 = pair(&[PolicyStage::Runtime], &[PolicyStage::Compile], &[]);
    let expected = pair(
        &[PolicyStage::Compile, PolicyStage::Runtime],
        &[PolicyStage::Compile],
        &[],
    );
    assert_eq!(default_p1(&p2), expected);
    assert_eq!(
        elaborate_value_binding_p1(
            &p2,
            None,
            TypeValueId(1),
            SemanticValueId(1),
            Provenance::new("omitted P1"),
        ),
        P1Elaboration {
            effective: expected,
            origin: P1Origin::Inferred,
            existing_slice: None,
            transition: None,
        }
    );
}

#[test]
fn explicit_identical_p1_preserves_explicit_provenance_without_transition() {
    let p2 = compile_pair();
    let default = default_p1(&p2);
    assert_eq!(
        elaborate_value_binding_p1(
            &p2,
            Some(&default),
            TypeValueId(1),
            SemanticValueId(1),
            Provenance::new("explicit identity"),
        ),
        P1Elaboration {
            effective: default,
            origin: P1Origin::Explicit,
            existing_slice: None,
            transition: None,
        }
    );
}

#[test]
fn explicit_existing_slice_is_projection_not_transition() {
    let p2 = pair(
        &[PolicyStage::Compile, PolicyStage::Runtime],
        &[PolicyStage::Compile],
        &[],
    );
    let source = default_p1(&p2);
    let target = runtime_pair();
    assert_eq!(
        elaborate_value_binding_p1(
            &p2,
            Some(&target),
            TypeValueId(1),
            SemanticValueId(1),
            Provenance::new("existing runtime slice"),
        ),
        P1Elaboration {
            effective: target,
            origin: P1Origin::Explicit,
            existing_slice: Some(ExistingPolicySlice {
                source,
                selected: runtime_pair(),
            }),
            transition: None,
        }
    );
}

#[test]
fn explicit_unavailable_p1_forms_transition_request() {
    let p2 = compile_pair();
    let target = runtime_pair();
    let elaboration = elaborate_value_binding_p1(
        &p2,
        Some(&target),
        TypeValueId(7),
        SemanticValueId(9),
        Provenance::new("compile to runtime"),
    );
    assert!(elaboration.existing_slice.is_none());
    let request = elaboration
        .transition
        .expect("unavailable runtime target must form a transition");
    assert_eq!(request.source_policy, compile_pair());
    assert_eq!(request.target_policy, target);
    assert_eq!(request.source_type, TypeValueId(7));
    assert_eq!(request.source_value, SemanticValueId(9));
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
fn compile_literal_reaches_runtime_only_through_selected_copy_callable() {
    let types = AtomicBuiltinTypeIds {
        uint: TypeValueId(1),
        int: TypeValueId(10),
        float: TypeValueId(3),
        buffer: TypeValueId(4),
        str_: TypeValueId(5),
    };
    let expr = initializer_from_source("let x = 42");
    let literal =
        materialize_literal_value(&expr, &types, SemanticValueId(20), Provenance::new("42"))
            .expect("integer literal");
    assert_eq!(literal.builtin_type, AtomicBuiltinType::Int);
    assert_eq!(literal.policy, compile_pair());

    let elaboration = elaborate_value_binding_p1(
        &literal.policy,
        Some(&runtime_pair()),
        literal.type_value,
        literal.id,
        Provenance::new("runtime let x = 42"),
    );
    assert!(elaboration.existing_slice.is_none());
    let request = elaboration
        .transition
        .expect("compile literal must request runtime transition");
    validate_runtime_transition(&request.source_policy, &request.target_policy)
        .expect("legal runtime transition");

    let candidates = vec![exact_copy("builtin-copy")];
    let PolicyBridgeResolution::Selected(selected) =
        resolve_policy_bridge(&request, &candidates, TransitionTypeExpectation::default())
    else {
        panic!("copy callable must be selected");
    };
    let result = invoke_resolved_policy_bridge(&selected, &request, SemanticValueId(21))
        .expect("copy invocation");
    assert_eq!(result.effect, PolicyBridgeEffect::BuiltinValueCopy);
    assert_eq!(result.value.type_value, literal.type_value);
    assert_eq!(result.value.source_value, literal.id);
    assert_eq!(result.value.policy, runtime_pair());
    assert_eq!(
        result.value.policy.pattern, literal.policy.pattern,
        "Pattern policy is unchanged"
    );
}

#[test]
fn atomic_builtins_exist_and_string_literal_is_compile_str_value_not_ref() {
    let types = AtomicBuiltinTypeIds {
        uint: TypeValueId(1),
        int: TypeValueId(2),
        float: TypeValueId(3),
        buffer: TypeValueId(4),
        str_: TypeValueId(5),
    };
    assert_eq!(types.get(AtomicBuiltinType::Uint), TypeValueId(1));
    assert_eq!(types.get(AtomicBuiltinType::Buffer), TypeValueId(4));
    // A dependent `str ref` identity is produced from `str` by a later type
    // constructor. It is intentionally not one of the atomic registry ids.
    let dependent_str_ref = TypeValueId(900);

    let expr = initializer_from_source("let s = \"abc\"");
    let literal = materialize_literal_value(
        &expr,
        &types,
        SemanticValueId(30),
        Provenance::new("\"abc\""),
    )
    .expect("string literal");
    assert_eq!(literal.builtin_type, AtomicBuiltinType::Str);
    assert_eq!(literal.type_value, types.str_);
    assert_ne!(literal.type_value, dependent_str_ref);
    assert_eq!(literal.policy, compile_pair());
}

#[test]
fn const_ref_bridge_selects_unique_non_delete_callable() {
    let target = runtime_pair_with(ValueMutability::Const);
    let mut request = request(compile_pair(), target.clone());
    request.source_type = TypeValueId(5); // compile `str` value
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
        panic!("const ref bridge must select");
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
        resolve_policy_bridge(&request, &candidates, TransitionTypeExpectation::default(),),
        PolicyBridgeResolution::NoCandidate
    );
}

#[test]
fn bridge_input_can_project_an_existing_slice_while_transition_is_required() {
    let request = request(multi_slice_static_value_pair(), runtime_pair());
    validate_runtime_transition(&request.source_policy, &request.target_policy)
        .expect("Pattern policy remains compile");
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
        panic!("the callable may select the compile slice of a multi-slice source");
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
            &request.source_policy,
            &request.target_policy,
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
        panic!("exact output policy must win");
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
            &request.source_policy,
            &request.target_policy,
            &candidates[0],
            &candidates[1],
        ),
        PolicyPartialOrdering::Incomparable
    );
    assert!(matches!(
        resolve_policy_bridge(&request, &candidates, TransitionTypeExpectation::default(),),
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
        resolve_policy_bridge(&request, &candidates, TransitionTypeExpectation::default(),),
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
    assert_eq!(selected.callable.id, "winner");
    let failure = invoke_resolved_policy_bridge(&selected, &request, SemanticValueId(21))
        .expect_err("selected lowering failure");
    assert_eq!(failure.selected_callable_id, "winner");
    assert_eq!(failure.message, "lowering failed");
}
