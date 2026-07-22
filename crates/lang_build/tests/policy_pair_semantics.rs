use std::collections::{BTreeMap, BTreeSet};

use lang_build::{
    bool_branch_space_for_tests, bool_pattern_aliases_for_tests, classify_static_task,
    compute_export_closure, compute_wpre, derive_function_object_p1,
    elaborate_binding_p1_projection, elaborate_formal_policy_pattern,
    elaborate_namespace_declaration_policy, expose_policy_slice, externally_visible,
    normalize_p2_policy, project_complete_symbol_flow, project_p1, publicly_reachable,
    read_pattern, read_value, resolve_explicit_path, select_by_mutability_product,
    select_policy_overload, BuiltinPrivilegedSealFunction, CompleteFlowNode, CompleteSymbolFlow,
    FunctionObjectDeclarationPolicy, MutabilityPattern, NamespaceDeclarationPosition,
    NamespaceExportNode, NamespaceVisibility, P1Projection, PatternComponentPolicy, Phase,
    PhaseOverloadCandidate, PolicyOverloadCandidate, PolicyOverloadSelection, PolicyResultEntry,
    PolicyStage, Provenance, SealWorldSnapshot, StageSet, StaticTaskDisposition, SymbolEntry,
    ValueComponentPolicy, ValueMutability, ValuePresence, WpreRoots,
};
use lang_syntax::{NormDecl, NormForm, NormPolicySpec};

fn policy_spec(source: &str) -> NormPolicySpec {
    let parsed = lang_syntax::parse(&format!("{source} let x = value;"));
    assert!(
        parsed.diagnostics.is_empty(),
        "unexpected parser diagnostics for `{source}`: {}",
        lang_syntax::dump_diagnostics(&parsed.diagnostics)
    );
    let normalized = lang_syntax::normalize_program(&parsed.program);
    match normalized.forms.as_slice() {
        [NormForm::Let(NormDecl::Let { slot, .. })] => slot.policy.clone().expect("policy prefix"),
        other => panic!("expected one let declaration, got {other:#?}"),
    }
}

fn stages(expected: &[PolicyStage]) -> StageSet {
    let mut result = StageSet::new();
    for stage in expected {
        result.insert(*stage);
    }
    result
}

fn mutability(expected: &[ValueMutability]) -> BTreeSet<ValueMutability> {
    expected.iter().copied().collect()
}

fn result_entry<V, P>(
    value: Option<V>,
    value_stages: &[PolicyStage],
    pattern: P,
    pattern_stages: &[PolicyStage],
) -> PolicyResultEntry<V, P> {
    PolicyResultEntry {
        value,
        value_policy: ValueComponentPolicy {
            stages: stages(value_stages),
            mutability: BTreeSet::new(),
            presence: if value_stages.is_empty() {
                ValuePresence::Absent
            } else {
                ValuePresence::Present
            },
        },
        pattern,
        pattern_policy: PatternComponentPolicy {
            stages: stages(pattern_stages),
        },
    }
}

#[test]
fn policy_parser_keeps_choice_conjunction_and_pair_precedence_distinct() {
    let pair = normalize_p2_policy(
        &policy_spec("const + runtime || compile : compile"),
        Provenance::new("precedence"),
    )
    .expect("`||` binds tighter than `+`, which binds tighter than `:`");
    assert_eq!(
        pair.value.stages,
        stages(&[PolicyStage::Compile, PolicyStage::Runtime])
    );
    assert_eq!(pair.pattern.stages, stages(&[PolicyStage::Compile]));
    assert_eq!(pair.value.mutability, mutability(&[ValueMutability::Const]));

    for source in [
        "runtime:compile",
        "runtime:seal",
        "(runtime || compile):compile",
        "(runtime || seal):seal",
        "mut + (runtime || compile):compile",
        "runtime || S : compile",
    ] {
        normalize_p2_policy(&policy_spec(source), Provenance::new(source))
            .unwrap_or_else(|diagnostic| panic!("`{source}` must parse: {diagnostic:?}"));
    }

    let parsed = lang_syntax::parse("runtime | compile let x = value;");
    assert!(
        !parsed.diagnostics.is_empty(),
        "single `|` must not become policy choice"
    );
    let pattern = lang_syntax::parse("let bool = ((if | else) bool) |> struct;");
    assert!(pattern.diagnostics.is_empty(), "Pattern `|` remains valid");
}

#[test]
fn policy_algebra_rejects_cross_dimension_choice_and_same_dimension_conjunction() {
    for source in [
        "runtime || const",
        "compile || public",
        "mut || export",
        "const + mut",
        "public + private",
    ] {
        assert!(
            normalize_p2_policy(&policy_spec(source), Provenance::new(source)).is_err(),
            "`{source}` must be rejected"
        );
    }
}

#[test]
fn p2_single_policy_normalization_uses_compile_for_runtime_only() {
    let cases = [
        ("meta", vec![PolicyStage::Meta], vec![PolicyStage::Meta]),
        (
            "compile",
            vec![PolicyStage::Compile],
            vec![PolicyStage::Compile],
        ),
        ("seal", vec![PolicyStage::Seal], vec![PolicyStage::Seal]),
        (
            "runtime",
            vec![PolicyStage::Runtime],
            vec![PolicyStage::Compile],
        ),
        (
            "runtime || compile",
            vec![PolicyStage::Compile, PolicyStage::Runtime],
            vec![PolicyStage::Compile],
        ),
        (
            "runtime || seal",
            vec![PolicyStage::Seal, PolicyStage::Runtime],
            vec![PolicyStage::Seal],
        ),
    ];

    for (source, value_stages, pattern_stages) in cases {
        let pair =
            normalize_p2_policy(&policy_spec(source), Provenance::new(source)).expect("valid P2");
        assert_eq!(pair.value.stages, stages(&value_stages));
        assert_eq!(pair.pattern.stages, stages(&pattern_stages));
    }
}

#[test]
fn p2_explicit_pairs_validate_component_boundaries() {
    for source in [
        "runtime:compile",
        "runtime:seal",
        "(runtime || compile):compile",
        "(runtime || seal):seal",
        "const + (compile || runtime):compile",
    ] {
        normalize_p2_policy(&policy_spec(source), Provenance::new(source))
            .expect("valid explicit P2 pair");
    }

    for source in [
        "runtime:runtime",
        "compile:seal",
        "meta:compile",
        "export:compile",
    ] {
        assert!(
            normalize_p2_policy(&policy_spec(source), Provenance::new(source)).is_err(),
            "`{source}` must be rejected as P2"
        );
    }
}

#[test]
fn p1_value_dominant_projection_restricts_the_actual_slice() {
    let projection = elaborate_binding_p1_projection(
        Some(&policy_spec("runtime")),
        Provenance::new("runtime P1"),
    )
    .expect("valid P1");
    assert!(matches!(projection, P1Projection::ValueDominant { .. }));

    let result = vec![result_entry(
        Some("same-symbol"),
        &[PolicyStage::Compile, PolicyStage::Runtime],
        "same-pattern",
        &[PolicyStage::Compile],
    )];
    let selected = project_p1(&projection, &result);
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].value, Some("same-symbol"));
    assert_eq!(selected[0].pattern, "same-pattern");
    assert_eq!(
        selected[0].value_policy.stages,
        stages(&[PolicyStage::Runtime])
    );
    assert_eq!(
        selected[0].pattern_policy.stages,
        stages(&[PolicyStage::Compile])
    );

    assert_eq!(
        project_p1(
            &elaborate_binding_p1_projection(None, Provenance::new("inferred"))
                .expect("omitted P1"),
            &result,
        ),
        result
    );
}

#[test]
fn formal_and_namespace_policy_contexts_are_not_binding_queries() {
    let formal =
        elaborate_formal_policy_pattern(Some(&policy_spec("const")), Provenance::new("formal"))
            .expect("const formal pattern");
    assert_eq!(formal.mutability, Some(ValueMutability::Const));

    for source in ["public", "private", "export"] {
        assert!(elaborate_binding_p1_projection(
            Some(&policy_spec(source)),
            Provenance::new(source)
        )
        .is_err());
        assert!(elaborate_formal_policy_pattern(
            Some(&policy_spec(source)),
            Provenance::new(source)
        )
        .is_err());
    }

    let declaration = elaborate_namespace_declaration_policy(
        Some(&policy_spec("export + public + runtime")),
        NamespaceDeclarationPosition::DirectTopLevel,
        Provenance::new("namespace top-level"),
    )
    .expect("independent export and visibility attributes");
    assert!(declaration.export_root);
    assert_eq!(declaration.visibility, Some(NamespaceVisibility::Public));
    assert!(elaborate_namespace_declaration_policy(
        Some(&policy_spec("export + runtime")),
        NamespaceDeclarationPosition::Local,
        Provenance::new("local export"),
    )
    .is_err());
}

#[test]
fn function_object_p1_lifts_only_p2_stage_dimensions() {
    let result = normalize_p2_policy(
        &policy_spec("const + runtime:seal"),
        Provenance::new("runtime result"),
    )
    .expect("valid result policy");
    let object = derive_function_object_p1(
        &result,
        &FunctionObjectDeclarationPolicy {
            mutability: mutability(&[ValueMutability::Mut]),
            namespace_visibility: Some(NamespaceVisibility::Private),
            export_root: true,
        },
    );
    assert_eq!(
        object.value.stages,
        stages(&[PolicyStage::Seal, PolicyStage::Runtime])
    );
    assert_eq!(object.pattern.stages, stages(&[PolicyStage::Seal]));
    assert_eq!(object.value.mutability, mutability(&[ValueMutability::Mut]));
    assert!(!object.value.mutability.contains(&ValueMutability::Const));
    assert_eq!(
        object.namespace_visibility,
        Some(NamespaceVisibility::Private)
    );
    assert!(object.export_root);

    let compile = normalize_p2_policy(
        &policy_spec("runtime:compile"),
        Provenance::new("compile result"),
    )
    .expect("valid result policy");
    let object = derive_function_object_p1(&compile, &FunctionObjectDeclarationPolicy::default());
    assert_eq!(
        object.value.stages,
        stages(&[PolicyStage::Compile, PolicyStage::Runtime])
    );
    assert_eq!(object.pattern.stages, stages(&[PolicyStage::Compile]));
    assert!(!object.export_root);
}

#[test]
fn phase_visibility_uses_visibility_domains_not_atom_intersection() {
    assert!(PolicyStage::Meta.visible_at(Phase::OpenStatic));
    assert!(!PolicyStage::Meta.visible_at(Phase::SealStatic));
    assert!(PolicyStage::Compile.visible_at(Phase::OpenStatic));
    assert!(PolicyStage::Compile.visible_at(Phase::SealStatic));
    assert!(!PolicyStage::Compile.visible_at(Phase::Runtime));
    assert!(!PolicyStage::Seal.visible_at(Phase::OpenStatic));
    assert!(PolicyStage::Seal.visible_at(Phase::SealStatic));
    assert!(PolicyStage::Runtime.visible_at(Phase::Runtime));
}

#[test]
fn runtime_value_symbol_resolves_while_only_static_pattern_is_exposed() {
    let symbols = vec![SymbolEntry {
        identity: 7_u32,
        path: "pkg::runtime_value".to_string(),
        entries: vec![result_entry(
            Some("runtime computation"),
            &[PolicyStage::Runtime],
            "compile Pattern",
            &[PolicyStage::Compile],
        )],
    }];
    let symbol = resolve_explicit_path(&symbols, "pkg::runtime_value").expect("symbol resolves");
    let exposed = expose_policy_slice(&symbol.entries[0], Phase::OpenStatic);
    assert_eq!(symbol.identity, 7);
    assert!(read_value(&exposed).is_none());
    assert_eq!(read_pattern(&exposed), Some(&"compile Pattern"));
    assert!(exposed.derived_compile_companion);
    assert_eq!(
        symbol.entries[0].value_policy.stages,
        stages(&[PolicyStage::Runtime]),
        "static projection must not consume the runtime computation"
    );
}

#[test]
fn seal_explicit_lookup_is_distinct_from_privileged_scan() {
    let mut world = SealWorldSnapshot::new(vec!["pre-a", "pre-b"]);
    world.push_seal_generated("seal-a");
    assert_eq!(
        world.scan_domain_for_builtin(BuiltinPrivilegedSealFunction::ExportWorldMaterializer),
        ["pre-a", "pre-b"]
    );
    assert_eq!(
        world.resolve_explicit(|name| *name == "seal-a"),
        Some(&"seal-a")
    );
    assert_eq!(
        world.final_world().copied().collect::<Vec<_>>(),
        vec!["pre-a", "pre-b", "seal-a"]
    );

    let seal_entry = result_entry(
        Some("seal value"),
        &[PolicyStage::Seal],
        "seal Pattern",
        &[PolicyStage::Seal],
    );
    assert!(read_value(&expose_policy_slice(&seal_entry, Phase::OpenStatic)).is_none());
    assert_eq!(
        read_value(&expose_policy_slice(&seal_entry, Phase::SealStatic)),
        Some(&"seal value")
    );
}

#[test]
fn wpre_is_the_least_semantic_dependency_closure_of_export_roots() {
    let roots = WpreRoots {
        exported_symbols: vec!["api"],
        materialized_results_of_exported_meta_functions: vec!["made"],
        parameter_dependencies_of_exported_meta_functions: vec!["parameter"],
    };
    let closure = compute_wpre(roots, |symbol| match *symbol {
        "api" => vec!["private-type"],
        "made" => vec!["made-dependency"],
        "private-type" => vec!["leaf"],
        _ => Vec::new(),
    });
    assert_eq!(
        closure,
        BTreeSet::from([
            "api",
            "leaf",
            "made",
            "made-dependency",
            "parameter",
            "private-type",
        ])
    );
}

#[test]
fn export_closure_and_public_path_reachability_are_independent() {
    let nodes = BTreeMap::from([
        (
            0,
            NamespaceExportNode {
                parent: None,
                visibility: NamespaceVisibility::Public,
            },
        ),
        (
            1,
            NamespaceExportNode {
                parent: Some(0),
                visibility: NamespaceVisibility::Public,
            },
        ),
        (
            2,
            NamespaceExportNode {
                parent: Some(0),
                visibility: NamespaceVisibility::Public,
            },
        ),
        (
            3,
            NamespaceExportNode {
                parent: Some(1),
                visibility: NamespaceVisibility::Private,
            },
        ),
        (
            4,
            NamespaceExportNode {
                parent: Some(3),
                visibility: NamespaceVisibility::Public,
            },
        ),
    ]);
    let exported = compute_export_closure(&nodes, [1]);
    assert_eq!(exported, BTreeSet::from([0, 1, 3, 4]));
    assert!(!exported.contains(&2), "siblings do not inherit export");
    assert!(publicly_reachable(&nodes, [0, 1]));
    assert!(!publicly_reachable(&nodes, [0, 1, 3]));
    assert!(exported.contains(&4), "private descendants remain exported");
    assert!(!externally_visible(&4, &exported, &nodes, [0, 1, 3, 4]));
}

#[test]
fn compile_projection_is_mechanical_and_preserves_control_structure() {
    let flow = CompleteSymbolFlow {
        nodes: vec![
            CompleteFlowNode::PatternType("type"),
            CompleteFlowNode::StaticCall("meta/compile/seal call"),
            CompleteFlowNode::DerivedCompileCompanion("companion"),
            CompleteFlowNode::DeferredSealTask("deferred"),
            CompleteFlowNode::RuntimeValueComputation("runtime value"),
            CompleteFlowNode::RuntimeBody("runtime body"),
            CompleteFlowNode::RuntimeEffect("effect"),
            CompleteFlowNode::RuntimeSymbolBinding("binding"),
            CompleteFlowNode::ControlFlow("branch"),
            CompleteFlowNode::Done("done"),
        ],
    };
    let projected = project_complete_symbol_flow(&flow);
    assert_eq!(projected.static_flow.nodes.len(), 6);
    assert_eq!(projected.runtime_residual_flow.nodes.len(), 6);
    assert!(matches!(
        classify_static_task(&stages(&[PolicyStage::Seal]), Phase::OpenStatic),
        StaticTaskDisposition::DeferredToSealStatic
    ));
    assert!(matches!(
        classify_static_task(&stages(&[PolicyStage::Runtime]), Phase::SealStatic),
        StaticTaskDisposition::FinalStaticError
    ));
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
fn const_mut_selection_uses_product_partial_order_and_delete_is_normal() {
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

    let delete = vec![
        candidate("const-delete", vec![MutabilityPattern::Const], true),
        candidate("wide", vec![MutabilityPattern::Unspecified], false),
    ];
    assert_eq!(
        select_by_mutability_product(&delete, &[ValueMutability::Const], None),
        PolicyOverloadSelection::RejectedByDelete("const-delete")
    );
}

#[test]
fn target_result_only_orders_candidates_when_the_context_supplies_it() {
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
}

#[test]
fn phase_stage_preference_is_part_of_the_partial_order() {
    let open = vec![
        PhaseOverloadCandidate {
            candidate: candidate("meta", vec![MutabilityPattern::Const], false),
            stage: PolicyStage::Meta,
            fully_admissible: true,
        },
        PhaseOverloadCandidate {
            candidate: candidate("compile", vec![MutabilityPattern::Const], false),
            stage: PolicyStage::Compile,
            fully_admissible: true,
        },
    ];
    assert_eq!(
        select_policy_overload(&open, &[ValueMutability::Const], None, Phase::OpenStatic),
        PolicyOverloadSelection::Selected("meta")
    );

    let seal = vec![
        PhaseOverloadCandidate {
            candidate: candidate("seal", vec![MutabilityPattern::Mut], false),
            stage: PolicyStage::Seal,
            fully_admissible: true,
        },
        PhaseOverloadCandidate {
            candidate: candidate("compile", vec![MutabilityPattern::Mut], false),
            stage: PolicyStage::Compile,
            fully_admissible: true,
        },
    ];
    assert_eq!(
        select_policy_overload(&seal, &[ValueMutability::Mut], None, Phase::SealStatic),
        PolicyOverloadSelection::Selected("seal")
    );

    let crossed = vec![
        PhaseOverloadCandidate {
            candidate: candidate("meta-wide", vec![MutabilityPattern::Unspecified], false),
            stage: PolicyStage::Meta,
            fully_admissible: true,
        },
        PhaseOverloadCandidate {
            candidate: candidate("compile-const", vec![MutabilityPattern::Const], false),
            stage: PolicyStage::Compile,
            fully_admissible: true,
        },
    ];
    assert!(matches!(
        select_policy_overload(&crossed, &[ValueMutability::Const], None, Phase::OpenStatic),
        PolicyOverloadSelection::Ambiguous(_)
    ));
}

#[test]
fn bool_has_one_pattern_alternative_space_and_true_false_are_aliases() {
    let space = bool_branch_space_for_tests(Provenance::new("bool"));
    assert_eq!(
        space
            .alternatives
            .iter()
            .map(|alternative| alternative.label.as_str())
            .collect::<Vec<_>>(),
        vec!["if", "else"]
    );
    let aliases = bool_pattern_aliases_for_tests();
    assert_eq!(aliases.len(), 2);
    assert_eq!(aliases[0].alias, "true");
    assert_eq!(
        aliases[0].target.segments,
        vec!["if".to_string(), "bool".to_string()]
    );
    assert_eq!(aliases[1].alias, "false");
    assert_eq!(
        aliases[1].target.segments,
        vec!["else".to_string(), "bool".to_string()]
    );
}
