use std::collections::BTreeSet;

use lang_build::{
    derive_function_object_p1, elaborate_p1_projection, normalize_p2_policy, project_p1,
    select_by_mutability_product, FunctionMember, FunctionMemberKind, FunctionObject,
    FunctionObjectDeclarationPolicy, FunctionSliceStage, MutabilityPattern, NamespaceVisibility,
    P1Projection, PatternComponentPolicy, PolicyLookupStage, PolicyOverloadCandidate,
    PolicyOverloadSelection, PolicyResultEntry, PolicyStage, Provenance, SealWorldSnapshot,
    StageSet, ValueComponentPolicy, ValueMutability, ValuePresence,
};
use lang_syntax::{NormDecl, NormForm, NormOrigin, NormPolicySpec, NormValuePolicyPattern, Span};

fn policy_spec(source: &str) -> NormPolicySpec {
    let parsed = lang_syntax::parse(&format!("{source} let x = value;"));
    assert!(
        parsed.diagnostics.is_empty(),
        "unexpected parser diagnostics: {}",
        lang_syntax::dump_diagnostics(&parsed.diagnostics)
    );
    let normalized = lang_syntax::normalize_program(&parsed.program);
    match normalized.forms.as_slice() {
        [NormForm::Let(NormDecl::Let { slot, .. })] => slot.policy.clone().expect("policy prefix"),
        other => panic!("expected one let declaration, got {other:#?}"),
    }
}

fn stages(expected: &[PolicyStage]) -> StageSet {
    let mut stages = StageSet::new();
    for stage in expected {
        stages.insert(*stage);
    }
    stages
}

fn mutability(expected: &[ValueMutability]) -> BTreeSet<ValueMutability> {
    expected.iter().copied().collect()
}

#[test]
fn p2_single_policy_normalization_uses_last_static_for_runtime() {
    let cases = [
        ("meta", vec![PolicyStage::Meta], PolicyStage::Meta),
        ("compile", vec![PolicyStage::Compile], PolicyStage::Compile),
        ("seal", vec![PolicyStage::Seal], PolicyStage::Seal),
        ("runtime", vec![PolicyStage::Runtime], PolicyStage::Seal),
        (
            "runtime|compile",
            vec![PolicyStage::Compile, PolicyStage::Runtime],
            PolicyStage::Compile,
        ),
        (
            "runtime|seal",
            vec![PolicyStage::Seal, PolicyStage::Runtime],
            PolicyStage::Seal,
        ),
    ];

    for (source, value_stages, pattern_stage) in cases {
        let pair =
            normalize_p2_policy(&policy_spec(source), Provenance::new(source)).expect("valid P2");
        assert_eq!(pair.value.stages, stages(&value_stages));
        assert_eq!(pair.pattern.stages, stages(&[pattern_stage]));
    }
}

#[test]
fn p2_explicit_pairs_validate_component_boundaries() {
    for source in [
        "runtime:compile",
        "runtime:seal",
        "(runtime|compile):compile",
        "(runtime|seal):seal",
        "const+(compile|runtime):compile",
    ] {
        normalize_p2_policy(&policy_spec(source), Provenance::new(source))
            .expect("valid explicit P2 pair");
    }

    for source in [
        "runtime:runtime",
        "compile:seal",
        "meta:compile",
        "public:private",
        "mut+export:compile",
    ] {
        assert!(
            normalize_p2_policy(&policy_spec(source), Provenance::new(source)).is_err(),
            "{source} must be rejected as P2"
        );
    }
}

#[test]
fn p1_single_policy_is_value_dominant_not_p_colon_p() {
    let projection =
        elaborate_p1_projection(Some(&policy_spec("runtime")), Provenance::new("runtime P1"))
            .expect("valid P1");
    assert!(matches!(projection, P1Projection::ValueDominant { .. }));

    let result = vec![
        PolicyResultEntry {
            value: Some("runtime-value"),
            value_policy: ValueComponentPolicy {
                stages: stages(&[PolicyStage::Runtime]),
                mutability: BTreeSet::new(),
                presence: ValuePresence::Present,
            },
            pattern: "seal-type",
            pattern_policy: PatternComponentPolicy {
                stages: stages(&[PolicyStage::Seal]),
            },
        },
        PolicyResultEntry {
            value: Some("compile-value"),
            value_policy: ValueComponentPolicy {
                stages: stages(&[PolicyStage::Compile]),
                mutability: BTreeSet::new(),
                presence: ValuePresence::Present,
            },
            pattern: "compile-type",
            pattern_policy: PatternComponentPolicy {
                stages: stages(&[PolicyStage::Compile]),
            },
        },
    ];
    let selected = project_p1(&projection, &result);
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].value, Some("runtime-value"));
    assert_eq!(selected[0].pattern, "seal-type");
}

#[test]
fn omitted_p1_infers_and_normalized_ast_reserves_absent_value() {
    assert_eq!(
        elaborate_p1_projection(None, Provenance::new("inferred P1")).expect("omitted P1 is valid"),
        P1Projection::Infer
    );
    let absent = NormValuePolicyPattern::Absent {
        origin: NormOrigin::Source(Span::at(0, 1, 1)),
    };
    assert!(matches!(absent, NormValuePolicyPattern::Absent { .. }));
}

#[test]
fn p1_pair_filters_value_and_pattern_components() {
    let projection = elaborate_p1_projection(
        Some(&policy_spec("runtime:compile")),
        Provenance::new("pair P1"),
    )
    .expect("valid P1 pair");
    assert!(matches!(projection, P1Projection::Pair(_)));

    let result = vec![
        PolicyResultEntry {
            value: Some(1),
            value_policy: ValueComponentPolicy {
                stages: stages(&[PolicyStage::Runtime]),
                mutability: BTreeSet::new(),
                presence: ValuePresence::Present,
            },
            pattern: "compile",
            pattern_policy: PatternComponentPolicy {
                stages: stages(&[PolicyStage::Compile]),
            },
        },
        PolicyResultEntry {
            value: Some(2),
            value_policy: ValueComponentPolicy {
                stages: stages(&[PolicyStage::Runtime]),
                mutability: BTreeSet::new(),
                presence: ValuePresence::Present,
            },
            pattern: "seal",
            pattern_policy: PatternComponentPolicy {
                stages: stages(&[PolicyStage::Seal]),
            },
        },
    ];
    let selected = project_p1(&projection, &result);
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].value, Some(1));
}

#[test]
fn optional_value_pattern_accepts_runtime_value_or_absent_value() {
    let projection = P1Projection::Pair(lang_build::PolicyPair {
        value: ValueComponentPolicy {
            stages: stages(&[PolicyStage::Runtime]),
            mutability: BTreeSet::new(),
            presence: ValuePresence::Optional,
        },
        pattern: PatternComponentPolicy {
            stages: stages(&[PolicyStage::Compile]),
        },
        namespace_visibility: None,
    });
    let result = vec![
        PolicyResultEntry {
            value: Some("runtime-value"),
            value_policy: ValueComponentPolicy {
                stages: stages(&[PolicyStage::Runtime]),
                mutability: BTreeSet::new(),
                presence: ValuePresence::Present,
            },
            pattern: "runtime-value-type",
            pattern_policy: PatternComponentPolicy {
                stages: stages(&[PolicyStage::Compile]),
            },
        },
        PolicyResultEntry {
            value: None,
            value_policy: ValueComponentPolicy {
                stages: StageSet::new(),
                mutability: BTreeSet::new(),
                presence: ValuePresence::Absent,
            },
            pattern: "pure-type",
            pattern_policy: PatternComponentPolicy {
                stages: stages(&[PolicyStage::Compile]),
            },
        },
        PolicyResultEntry {
            value: Some("compile-value"),
            value_policy: ValueComponentPolicy {
                stages: stages(&[PolicyStage::Compile]),
                mutability: BTreeSet::new(),
                presence: ValuePresence::Present,
            },
            pattern: "compile-value-type",
            pattern_policy: PatternComponentPolicy {
                stages: stages(&[PolicyStage::Compile]),
            },
        },
    ];

    let selected = project_p1(&projection, &result);
    assert_eq!(selected.len(), 2);
    assert_eq!(selected[0].value, Some("runtime-value"));
    assert_eq!(selected[1].value, None);
}

#[test]
fn function_object_p1_lifts_only_result_stages() {
    let result = normalize_p2_policy(
        &policy_spec("const+runtime:seal"),
        Provenance::new("runtime result"),
    )
    .expect("valid result policy");
    let object = derive_function_object_p1(
        &result,
        &FunctionObjectDeclarationPolicy {
            mutability: mutability(&[ValueMutability::Mut]),
            namespace_visibility: Some(NamespaceVisibility::Private),
        },
    );
    assert_eq!(
        object.value.stages,
        stages(&[PolicyStage::Seal, PolicyStage::Runtime])
    );
    assert_eq!(object.pattern.stages, stages(&[PolicyStage::Seal]));
    assert_eq!(object.value.mutability, mutability(&[ValueMutability::Mut]));
    assert_eq!(
        object.namespace_visibility,
        Some(NamespaceVisibility::Private)
    );
    assert!(!object.value.mutability.contains(&ValueMutability::Const));

    let compile_result = normalize_p2_policy(
        &policy_spec("(runtime|compile):compile"),
        Provenance::new("compile result"),
    )
    .expect("valid result policy");
    let compile_object =
        derive_function_object_p1(&compile_result, &FunctionObjectDeclarationPolicy::default());
    assert_eq!(
        compile_object.value.stages,
        stages(&[PolicyStage::Compile, PolicyStage::Runtime])
    );
    assert_eq!(
        compile_object.pattern.stages,
        stages(&[PolicyStage::Compile])
    );
}

#[test]
fn p1_namespace_rules_share_visibility_and_reject_mut_export() {
    let left = elaborate_p1_projection(
        Some(&policy_spec("public:compile")),
        Provenance::new("left namespace"),
    )
    .expect("public on value side");
    let right = elaborate_p1_projection(
        Some(&policy_spec("compile:public")),
        Provenance::new("right namespace"),
    )
    .expect("public on pattern side");
    assert_eq!(left, right);

    for source in ["public:private", "mut+export"] {
        assert!(
            elaborate_p1_projection(Some(&policy_spec(source)), Provenance::new(source)).is_err()
        );
    }
}

#[test]
fn function_slices_preserve_identity_and_bound_member_sets() {
    let object = FunctionObject {
        symbol_identity: 10_u32,
        anonymous_type_identity: 20_u32,
        members: vec![
            FunctionMember {
                id: 1,
                kind: FunctionMemberKind::Concrete,
            },
            FunctionMember {
                id: 2,
                kind: FunctionMemberKind::MaterializedInstance,
            },
            FunctionMember {
                id: 3,
                kind: FunctionMemberKind::GenericTemplate,
            },
        ],
    };
    let runtime = object.slice(FunctionSliceStage::Runtime);
    let seal = object.slice(FunctionSliceStage::Seal);
    assert_eq!(
        (runtime.symbol_identity, runtime.anonymous_type_identity),
        (10, 20)
    );
    assert_eq!(runtime.member_ids, vec![1]);
    assert_eq!(seal.member_ids, vec![1, 2]);
}

#[test]
fn seal_visibility_and_scan_snapshot_are_bounded() {
    assert!(!PolicyStage::Meta.visible_at(PolicyLookupStage::Seal));
    assert!(PolicyStage::Meta.visible_at(PolicyLookupStage::Compile));
    assert!(PolicyStage::Compile.visible_at(PolicyLookupStage::Seal));
    assert!(PolicyStage::Seal.visible_at(PolicyLookupStage::Compile));
    assert!(PolicyStage::Seal.visible_at(PolicyLookupStage::PostSealCompile));
    assert!(!PolicyStage::Seal.visible_at(PolicyLookupStage::OpenMeta));

    let mut world = SealWorldSnapshot::new(vec!["pre-a", "pre-b"]);
    world.push_seal_generated("seal-a");
    assert_eq!(world.scan_domain(), ["pre-a", "pre-b"]);
    assert_eq!(
        world.final_world().copied().collect::<Vec<_>>(),
        vec!["pre-a", "pre-b", "seal-a"]
    );
}

fn candidate(
    id: &'static str,
    parameter_policies: Vec<MutabilityPattern>,
    is_delete: bool,
) -> PolicyOverloadCandidate<&'static str> {
    PolicyOverloadCandidate {
        id,
        parameter_policies,
        result_policy: None,
        is_delete,
    }
}

#[test]
fn const_mut_selection_uses_product_partial_order() {
    let single = vec![
        candidate("const", vec![MutabilityPattern::Const], false),
        candidate("wide", vec![MutabilityPattern::Unspecified], false),
        candidate("mut", vec![MutabilityPattern::Mut], false),
    ];
    assert_eq!(
        select_by_mutability_product(&single, &[ValueMutability::Const], None),
        PolicyOverloadSelection::Selected("const")
    );
    assert_eq!(
        select_by_mutability_product(&single, &[ValueMutability::Mut], None),
        PolicyOverloadSelection::Selected("mut")
    );

    let crossed = vec![
        candidate(
            "left",
            vec![MutabilityPattern::Const, MutabilityPattern::Unspecified],
            false,
        ),
        candidate(
            "right",
            vec![MutabilityPattern::Unspecified, MutabilityPattern::Const],
            false,
        ),
    ];
    assert!(matches!(
        select_by_mutability_product(
            &crossed,
            &[ValueMutability::Const, ValueMutability::Const],
            None
        ),
        PolicyOverloadSelection::Ambiguous(_)
    ));
}

#[test]
fn delete_member_participates_and_reports_specific_rejection() {
    let candidates = vec![
        candidate("const-delete", vec![MutabilityPattern::Const], true),
        candidate("wide", vec![MutabilityPattern::Unspecified], false),
    ];
    assert_eq!(
        select_by_mutability_product(&candidates, &[ValueMutability::Const], None),
        PolicyOverloadSelection::RejectedByDelete("const-delete")
    );
}

#[test]
fn result_mutability_only_orders_candidates_with_a_target_constraint() {
    let candidates = vec![
        PolicyOverloadCandidate {
            id: "const-result",
            parameter_policies: vec![MutabilityPattern::Unspecified],
            result_policy: Some(MutabilityPattern::Const),
            is_delete: false,
        },
        PolicyOverloadCandidate {
            id: "mut-result",
            parameter_policies: vec![MutabilityPattern::Unspecified],
            result_policy: Some(MutabilityPattern::Mut),
            is_delete: false,
        },
    ];
    assert!(matches!(
        select_by_mutability_product(&candidates, &[ValueMutability::Const], None),
        PolicyOverloadSelection::Ambiguous(_)
    ));
    assert_eq!(
        select_by_mutability_product(
            &candidates,
            &[ValueMutability::Const],
            Some(ValueMutability::Const)
        ),
        PolicyOverloadSelection::Selected("const-result")
    );

    let with_unspecified = vec![
        PolicyOverloadCandidate {
            id: "const-result",
            parameter_policies: vec![MutabilityPattern::Unspecified],
            result_policy: Some(MutabilityPattern::Const),
            is_delete: false,
        },
        PolicyOverloadCandidate {
            id: "unspecified-result",
            parameter_policies: vec![MutabilityPattern::Unspecified],
            result_policy: None,
            is_delete: false,
        },
    ];
    assert_eq!(
        select_by_mutability_product(
            &with_unspecified,
            &[ValueMutability::Const],
            Some(ValueMutability::Const)
        ),
        PolicyOverloadSelection::Selected("const-result")
    );
}
