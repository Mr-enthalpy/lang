mod support;

use std::convert::Infallible;

use lang_build::{
    assemble_value_binding_slices, compare_policy_transition_candidates, default_p1,
    elaborate_pure_type_binding_p1, elaborate_value_binding_p1, invoke_resolved_policy_bridge,
    materialize_literal_value, policy_bridge_is_available, resolve_policy_bridge,
    select_policy_overload, type_value_projection_from_type_symbol, validate_runtime_transition,
    AtomicBuiltinFamily, CompilationWorld, LiteralMaterializationFailure, LiteralTypeSelection,
    MutabilityActualFrame, MutabilityFormalFrame, MutabilityPattern, NumericFamily, NumericTypeKey,
    NumericTypeRegistry, OrdinaryCallableTypeInput, OrdinaryCallableTypeOutput,
    P1ElaborationFailure, P1Origin, PatternComponentPolicy, Phase, PhaseOverloadCandidate,
    PolicyBridgeBody, PolicyBridgeResolution, PolicyOverloadCandidate, PolicyOverloadSelection,
    PolicyPair, PolicyPartialOrdering, PolicyResultEntry, PolicyStage, PolicyTransitionCallable,
    PolicyTransitionFailure, PolicyTransitionRequest, Provenance, SemanticValueId,
    SemanticValueRef, StageSet, TransitionTypeExpectation, TypeValueId, ValueComponentPolicy,
    ValueMutability, ValuePresence,
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
fn omitted_p1_retains_defaulted_entries_without_demands() {
    let result = vec![value_entry(
        1,
        10,
        &[PolicyStage::Runtime],
        &[PolicyStage::Compile],
    )];
    let elaboration =
        elaborate_value_binding_p1(&result, None, Provenance::new("omitted P1")).unwrap();
    assert_eq!(elaboration.origin, P1Origin::Inferred);
    assert_eq!(elaboration.requested, None);
    assert!(elaboration.missing_value_demands.is_empty());
    assert_eq!(
        elaboration.existing_slices[0].value_policy.stages,
        compile_runtime_pair().value.stages
    );
    assert_eq!(
        default_p1(&pair(&[PolicyStage::Runtime], &[PolicyStage::Compile], &[])),
        compile_runtime_pair()
    );
}

#[test]
fn explicit_identical_p1_is_an_existing_identity_slice() {
    let result = vec![value_entry(
        1,
        10,
        &[PolicyStage::Compile],
        &[PolicyStage::Compile],
    )];
    let elaboration = elaborate_value_binding_p1(
        &result,
        Some(&compile_pair()),
        Provenance::new("explicit identity"),
    )
    .unwrap();
    assert_eq!(elaboration.origin, P1Origin::Explicit);
    assert_eq!(elaboration.existing_slices.len(), 1);
    assert!(elaboration.missing_value_demands.is_empty());
    assert_eq!(
        elaboration.existing_slices[0].value.unwrap().id,
        SemanticValueId(1)
    );
}

#[test]
fn explicit_existing_runtime_slice_preserves_semantic_value_identity() {
    let result = vec![value_entry(
        1,
        10,
        &[PolicyStage::Compile, PolicyStage::Runtime],
        &[PolicyStage::Compile],
    )];
    let elaboration = elaborate_value_binding_p1(
        &result,
        Some(&runtime_pair()),
        Provenance::new("existing runtime slice"),
    )
    .unwrap();
    assert_eq!(elaboration.existing_slices.len(), 1);
    assert!(elaboration.missing_value_demands.is_empty());
    assert_eq!(
        elaboration.existing_slices[0].value.unwrap().id,
        SemanticValueId(1)
    );
    assert_eq!(
        elaboration.existing_slices[0].value_policy.stages,
        runtime_pair().value.stages
    );
}

#[test]
fn existing_compile_slice_and_missing_runtime_demand_are_combined() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("bootstrap world");
    let numeric_types = NumericTypeRegistry::from_core_world(&world).expect("numeric registry");
    let int32 = NumericTypeKey::new(NumericFamily::Int, 32);
    let int32_type = numeric_types.get(int32).expect("canonical core int32");
    let expr = initializer_from_source("let x = 42");
    let literal = materialize_literal_value(
        &expr,
        &numeric_types,
        LiteralTypeSelection::Numeric(int32),
        SemanticValueId(20),
        Provenance::new("42"),
    )
    .expect("context selected concrete int32");
    let result = vec![PolicyResultEntry {
        value: Some(SemanticValueRef {
            id: literal.id,
            type_value: literal.type_value,
        }),
        value_policy: literal.policy.value.clone(),
        pattern: "literal-pattern",
        pattern_policy: literal.policy.pattern.clone(),
    }];

    let elaboration = elaborate_value_binding_p1(
        &result,
        Some(&compile_runtime_pair()),
        Provenance::new("(compile || runtime) let x = 42"),
    )
    .expect("P1 decomposes into existing and missing slices");
    assert_eq!(elaboration.existing_slices.len(), 1);
    assert_eq!(
        elaboration.existing_slices[0].value.unwrap().id,
        literal.id,
        "the compile slice keeps the original semantic value"
    );
    assert_eq!(elaboration.missing_value_demands.len(), 1);
    let demand = &elaboration.missing_value_demands[0].request;
    assert_eq!(demand.source_policy, compile_pair());
    assert_eq!(demand.target_policy, runtime_pair());
    validate_runtime_transition(&demand.source_policy, &demand.target_policy)
        .expect("only the missing runtime slice is validated");

    let candidate = callable(
        "builtin-copy",
        OrdinaryCallableTypeInput::Exact(int32_type),
        OrdinaryCallableTypeOutput::SameAsInput,
        compile_pair(),
        runtime_pair(),
        false,
        PolicyBridgeBody::BuiltinValueCopy,
    );
    let PolicyBridgeResolution::Selected(selected) =
        resolve_policy_bridge(demand, &[candidate], TransitionTypeExpectation::default())
    else {
        panic!("copy candidate must be selected");
    };
    let produced = invoke_resolved_policy_bridge(&selected, demand, SemanticValueId(21))
        .expect("copy prototype")
        .value;
    let combined = assemble_value_binding_slices(&elaboration, &[produced]).unwrap();
    assert_eq!(combined.len(), 2);
    assert_eq!(combined[0].value.unwrap().id, SemanticValueId(20));
    assert_eq!(combined[1].value.unwrap().id, SemanticValueId(21));
    assert_eq!(
        combined[0]
            .value_policy
            .stages
            .union(&combined[1].value_policy.stages),
        compile_runtime_pair().value.stages
    );
}

#[test]
fn multiple_missing_value_stages_produce_distinct_direct_demands() {
    let result = vec![value_entry(
        20,
        10,
        &[PolicyStage::Compile],
        &[PolicyStage::Compile],
    )];
    let target = pair(
        &[
            PolicyStage::Meta,
            PolicyStage::Compile,
            PolicyStage::Runtime,
        ],
        &[PolicyStage::Compile],
        &[],
    );
    let elaboration = elaborate_value_binding_p1(
        &result,
        Some(&target),
        Provenance::new("multi-demand target"),
    )
    .unwrap();
    assert_eq!(elaboration.existing_slices.len(), 1);
    assert_eq!(elaboration.missing_value_demands.len(), 2);
    assert_eq!(
        elaboration.missing_value_demands[0]
            .request
            .target_policy
            .value
            .stages,
        StageSet::from([PolicyStage::Meta])
    );
    assert_eq!(
        elaboration.missing_value_demands[1]
            .request
            .target_policy
            .value
            .stages,
        StageSet::from([PolicyStage::Runtime])
    );
}

#[test]
fn multi_entry_elaboration_preserves_each_identity_and_derives_each_demand() {
    let result = vec![
        value_entry(20, 10, &[PolicyStage::Compile], &[PolicyStage::Compile]),
        value_entry(30, 11, &[PolicyStage::Compile], &[PolicyStage::Compile]),
    ];
    let elaboration = elaborate_value_binding_p1(
        &result,
        Some(&compile_runtime_pair()),
        Provenance::new("multi-entry target"),
    )
    .unwrap();
    assert_eq!(
        elaboration
            .existing_slices
            .iter()
            .map(|entry| entry.value.unwrap().id)
            .collect::<Vec<_>>(),
        vec![SemanticValueId(20), SemanticValueId(30)]
    );
    assert_eq!(
        elaboration
            .missing_value_demands
            .iter()
            .map(|demand| demand.request.source_value)
            .collect::<Vec<_>>(),
        vec![SemanticValueId(20), SemanticValueId(30)]
    );
    assert!(elaboration
        .missing_value_demands
        .iter()
        .all(|demand| demand.request.target_policy == runtime_pair()));
}

#[test]
fn pure_type_can_project_pattern_slice_without_value_identity_or_transition_api() {
    let result = vec![pure_entry(&[PolicyStage::Meta, PolicyStage::Compile])];
    let target = absent_pair(&[PolicyStage::Compile]);
    let elaboration =
        elaborate_pure_type_binding_p1(&result, Some(&target)).expect("pure compile slice");
    assert_eq!(elaboration.existing_slices.len(), 1);
    assert_eq!(elaboration.existing_slices[0].value, None);
    assert_eq!(
        elaboration.existing_slices[0].pattern_policy.stages,
        StageSet::from([PolicyStage::Compile])
    );
}

#[test]
fn pure_type_unavailable_pattern_slice_is_projection_failure() {
    let result = vec![pure_entry(&[PolicyStage::Compile])];
    let target = absent_pair(&[PolicyStage::Seal]);
    assert!(matches!(
        elaborate_pure_type_binding_p1(&result, Some(&target)),
        Err(P1ElaborationFailure::RequestedPatternSliceUnavailable { .. })
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
    assert_eq!(
        elaborate_value_binding_p1(
            &result,
            Some(&runtime_pair()),
            Provenance::new("invalid value-bearing input")
        ),
        Err(P1ElaborationFailure::ValueBearingInputContainsAbsentValue)
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
        (NumericTypeKey::new(NumericFamily::Uint, 64), "uint64"),
        (NumericTypeKey::new(NumericFamily::Int, 8), "int8"),
        (NumericTypeKey::new(NumericFamily::Int, 16), "int16"),
        (NumericTypeKey::new(NumericFamily::Int, 32), "int32"),
        (NumericTypeKey::new(NumericFamily::Int, 64), "int64"),
        (NumericTypeKey::new(NumericFamily::Float, 32), "float32"),
        (NumericTypeKey::new(NumericFamily::Float, 64), "float64"),
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
    let int32 = NumericTypeKey::new(NumericFamily::Int, 32);
    let expected = registry.get(int32).expect("canonical core int32");
    let expr = initializer_from_source("let x = 42");
    let literal = materialize_literal_value(
        &expr,
        &registry,
        LiteralTypeSelection::Numeric(int32),
        SemanticValueId(30),
        Provenance::new("42"),
    )
    .expect("concrete Tnum selected");
    assert_eq!(literal.literal_family, AtomicBuiltinFamily::Int);
    assert_eq!(literal.numeric_type, Some(int32));
    assert_eq!(literal.type_value, expected);
    assert_eq!(literal.policy, compile_pair());
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
            LiteralMaterializationFailure::NumericLiteralRequiresConcreteNumericKey {
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
    assert_eq!(literal.literal_family, AtomicBuiltinFamily::Str);
    assert_eq!(literal.numeric_type, None);
    assert_eq!(literal.type_value, TypeValueId(5));
    assert_ne!(literal.type_value, TypeValueId(900), "dependent str ref");
    assert_eq!(literal.policy, compile_pair());
}

#[test]
fn const_ref_bridge_selects_unique_non_delete_candidate() {
    let target = runtime_pair_with(ValueMutability::Const);
    let mut request = request(compile_pair(), target.clone());
    request.source_type = TypeValueId(5);
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
            &request.source_policy,
            &request.target_policy,
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
