use std::collections::{BTreeMap, BTreeSet};

use lang_build::{
    bool_branch_space_for_tests, bool_pattern_aliases_for_tests, classify_static_task,
    compute_export_retention_closure, compute_wpre, derive_function_object_p1,
    elaborate_binding_p1_projection, elaborate_formal_policy_pattern,
    elaborate_namespace_declaration_policy, expose_policy_slice, externally_visible,
    function_object_declaration_policy, normalize_p2_policy, project_complete_symbol_flow,
    project_export_overload_sets, project_p1, project_resolved_export_view, publicly_reachable,
    read_pattern, read_value, resolve_explicit_path, select_by_mutability_product,
    select_policy_overload, BuiltinPrivilegedSealFunction, CapabilityRealization,
    CapabilityRealizationCell, CompleteFlowNode, CompleteSymbolFlow, ExportAdmission,
    FunctionObjectDeclarationPolicy, MutabilityActualFrame, MutabilityFormalFrame,
    MutabilityPattern, NamespaceDeclarationPosition, NamespaceExportNode, NamespaceVisibility,
    OutputModeDemand, P1Projection, PatternComponentPolicy, Phase, PhaseOverloadCandidate,
    PolicyMode, PolicyOverloadCandidate, PolicyOverloadSelection, PolicyPair, PolicyResultEntry,
    PolicyStage, Provenance, ResolvedCandidatePolicy, SealWorldSnapshot, StageSet,
    StaticTaskDisposition, SymbolEntry, ValueComponentPolicy, ValueMutability, ValuePresence,
    WpreRoots,
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
fn absent_value_component_cannot_carry_stage_or_mutability_subdimensions() {
    let absent = normalize_p2_policy(
        &policy_spec("S : compile"),
        Provenance::new("valid absent P2"),
    )
    .expect("a structurally empty absent Pv with compile Pp remains valid");
    assert_eq!(absent.value.presence, ValuePresence::Absent);
    assert!(absent.value.stages.is_empty());
    assert!(absent.value.mutability.is_empty());

    for source in ["const + S : compile", "mut + S : compile"] {
        let policy = policy_spec(source);
        assert!(
            normalize_p2_policy(&policy, Provenance::new(format!("P2 {source}"))).is_err(),
            "`{source}` must not form a P2 with mutability attached to absent Pv"
        );
        assert!(
            elaborate_binding_p1_projection(Some(&policy), Provenance::new(format!("P1 {source}")))
                .is_err(),
            "`{source}` must not form a P1 with mutability attached to absent Pv"
        );
    }

    for source in ["export + const + S : compile", "export + mut + S : compile"] {
        assert!(
            elaborate_namespace_declaration_policy(
                Some(&policy_spec(source)),
                NamespaceDeclarationPosition::DirectTopLevel,
                Provenance::new(source),
            )
            .is_err(),
            "`{source}` must not attach mutability to an absent exported Pv"
        );
    }

    for (label, value_stages, value_mutability) in [
        (
            "absent with stage",
            stages(&[PolicyStage::Runtime]),
            BTreeSet::new(),
        ),
        (
            "absent with mutability",
            StageSet::new(),
            mutability(&[ValueMutability::Const]),
        ),
    ] {
        let invalid = ResolvedCandidatePolicy {
            pair: PolicyPair {
                value: ValueComponentPolicy {
                    stages: value_stages,
                    mutability: value_mutability,
                    presence: ValuePresence::Absent,
                },
                pattern: PatternComponentPolicy {
                    stages: stages(&[PolicyStage::Compile]),
                },
            },
            mode: PolicyMode::Plain,
            capability_realization: CapabilityRealization::default(),
            provenance: Provenance::new(label),
        };
        assert!(
            project_resolved_export_view(&invalid).is_err(),
            "resolved export projection must reject {label}"
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
    let inherited_p2 = normalize_p2_policy(
        &policy_spec("runtime:compile"),
        Provenance::new("formal inherited P2"),
    )
    .expect("valid inherited P2");
    let plain =
        elaborate_formal_policy_pattern(None, &inherited_p2, Provenance::new("plain formal"))
            .expect("omitted formal policy inherits P2");
    assert_eq!(plain.effective_pair, inherited_p2);
    assert_eq!(plain.mode, PolicyMode::Plain);

    let formal = elaborate_formal_policy_pattern(
        Some(&policy_spec("const")),
        &inherited_p2,
        Provenance::new("formal"),
    )
    .expect("const formal pattern");
    assert_eq!(formal.mode, PolicyMode::Const);
    assert_eq!(
        formal.effective_pair.value.stages,
        inherited_p2.value.stages
    );
    assert_eq!(
        formal.effective_pair.pattern, inherited_p2.pattern,
        "formal const/mut syntax must not change the inherited Pattern policy"
    );
    assert_eq!(
        formal.effective_pair.value.presence,
        inherited_p2.value.presence
    );
    assert_eq!(
        formal.effective_pair.value.mutability,
        mutability(&[ValueMutability::Const])
    );

    for source in ["public", "private", "export"] {
        assert!(elaborate_binding_p1_projection(
            Some(&policy_spec(source)),
            Provenance::new(source)
        )
        .is_err());
        assert!(elaborate_formal_policy_pattern(
            Some(&policy_spec(source)),
            &inherited_p2,
            Provenance::new(source)
        )
        .is_err());
    }
    for source in ["runtime", "compile", "seal", "const + runtime"] {
        assert!(
            elaborate_formal_policy_pattern(
                Some(&policy_spec(source)),
                &inherited_p2,
                Provenance::new(source)
            )
            .is_err(),
            "formal `{source}` must not replace inherited P2 dimensions"
        );
    }

    let const_only_p2 = normalize_p2_policy(
        &policy_spec("const + runtime:compile"),
        Provenance::new("const-only inherited P2"),
    )
    .expect("valid const-only P2");
    assert!(elaborate_formal_policy_pattern(
        Some(&policy_spec("mut")),
        &const_only_p2,
        Provenance::new("expanding mut formal")
    )
    .is_err());

    let declaration = elaborate_namespace_declaration_policy(
        Some(&policy_spec("export + public + runtime")),
        NamespaceDeclarationPosition::DirectTopLevel,
        Provenance::new("namespace top-level"),
    )
    .expect("export preserves its internal Policy view independently of visibility");
    assert!(declaration.export_root);
    assert_eq!(declaration.visibility, Some(NamespaceVisibility::Public));
    let P1Projection::ValueDominant { value } = &declaration.projection else {
        panic!("single namespace policy must elaborate as value-dominant P1");
    };
    assert!(
        value.mutability.is_empty(),
        "export must not crop the complete namespace-internal declaration view"
    );
    let Some(P1Projection::ValueDominant {
        value: external_value,
    }) = &declaration.external_projection
    else {
        panic!("export root must carry an external value view");
    };
    assert!(
        external_value.mutability.is_empty(),
        "bare export preserves its plain declaration projection"
    );
    let function_declaration = function_object_declaration_policy(&declaration);
    assert!(
        function_declaration.mutability.is_empty(),
        "function-object formation consumes the complete internal declaration view"
    );

    let explicit_const = elaborate_namespace_declaration_policy(
        Some(&policy_spec("export + const + runtime")),
        NamespaceDeclarationPosition::DirectTopLevel,
        Provenance::new("explicit const export"),
    )
    .expect("export + const is valid");
    let P1Projection::ValueDominant { value } = explicit_const.projection else {
        panic!("single namespace policy must elaborate as value-dominant P1");
    };
    assert_eq!(value.mutability, mutability(&[ValueMutability::Const]));
    assert!(matches!(
        explicit_const.external_projection,
        Some(P1Projection::ValueDominant { ref value })
            if value.mutability == mutability(&[ValueMutability::Const])
    ));

    let broad_internal = elaborate_namespace_declaration_policy(
        Some(&policy_spec("export + (const || mut) + runtime")),
        NamespaceDeclarationPosition::DirectTopLevel,
        Provenance::new("broad internal export"),
    )
    .expect("a full const-or-mut internal view remains stable across export");
    assert!(matches!(
        broad_internal.projection,
        P1Projection::ValueDominant { ref value }
            if value.mutability
                == mutability(&[ValueMutability::Const, ValueMutability::Mut])
    ));
    assert!(matches!(
        broad_internal.external_projection,
        Some(P1Projection::ValueDominant { ref value })
            if value.mutability
                == mutability(&[ValueMutability::Const, ValueMutability::Mut])
    ));

    let mut_only_export = elaborate_namespace_declaration_policy(
        Some(&policy_spec("export + mut + runtime")),
        NamespaceDeclarationPosition::DirectTopLevel,
        Provenance::new("mut-only export"),
    )
    .expect("export admission does not const-crop or reject a mut Policy view");
    assert!(matches!(
        mut_only_export.external_projection,
        Some(P1Projection::ValueDominant { ref value })
            if value.mutability == mutability(&[ValueMutability::Mut])
    ));

    let type_only = elaborate_namespace_declaration_policy(
        Some(&policy_spec("export + S : compile")),
        NamespaceDeclarationPosition::DirectTopLevel,
        Provenance::new("type-only export"),
    )
    .expect("a pure Pattern/type export has no value-mutability obligation");
    assert!(matches!(
        type_only.external_projection,
        Some(P1Projection::Pair(ref pair))
            if pair.value.presence == ValuePresence::Absent
                && pair.value.mutability.is_empty()
    ));

    assert!(elaborate_namespace_declaration_policy(
        Some(&policy_spec("export + runtime")),
        NamespaceDeclarationPosition::Local,
        Provenance::new("local export"),
    )
    .is_err());
}

#[test]
fn export_overload_set_is_a_projection_of_the_full_set_not_a_second_world() {
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Candidate {
        identity: u32,
        export_root: bool,
        internal_policy: PolicyPair,
    }

    let runtime_value = |mutability: BTreeSet<ValueMutability>| PolicyPair {
        value: ValueComponentPolicy {
            stages: stages(&[PolicyStage::Runtime]),
            mutability,
            presence: ValuePresence::Present,
        },
        pattern: PatternComponentPolicy {
            stages: stages(&[PolicyStage::Compile]),
        },
    };
    let type_only = || PolicyPair {
        value: ValueComponentPolicy {
            stages: StageSet::new(),
            mutability: BTreeSet::new(),
            presence: ValuePresence::Absent,
        },
        pattern: PatternComponentPolicy {
            stages: stages(&[PolicyStage::Compile]),
        },
    };
    fn namespace_path<'a>(
        nodes: &BTreeMap<&'a str, NamespaceExportNode<&'a str>>,
        symbol: &'a str,
    ) -> Vec<&'a str> {
        let mut reversed = Vec::new();
        let mut current = Some(symbol);
        while let Some(id) = current {
            reversed.push(id);
            current = nodes.get(id).and_then(|node| node.parent);
        }
        reversed.reverse();
        reversed
    }

    let nodes = BTreeMap::from([
        (
            "f",
            NamespaceExportNode {
                parent: None,
                visibility: NamespaceVisibility::Public,
            },
        ),
        (
            "exported_child",
            NamespaceExportNode {
                parent: Some("f"),
                visibility: NamespaceVisibility::Public,
            },
        ),
        (
            "private_child",
            NamespaceExportNode {
                parent: Some("f"),
                visibility: NamespaceVisibility::Private,
            },
        ),
        (
            "public_behind_private",
            NamespaceExportNode {
                parent: Some("private_child"),
                visibility: NamespaceVisibility::Public,
            },
        ),
        (
            "exported_type",
            NamespaceExportNode {
                parent: Some("f"),
                visibility: NamespaceVisibility::Public,
            },
        ),
        (
            "private_dependency",
            NamespaceExportNode {
                parent: None,
                visibility: NamespaceVisibility::Private,
            },
        ),
    ]);
    let export_retention_closure = compute_export_retention_closure(&nodes, ["f"]);
    assert!(export_retention_closure.contains("private_child"));
    assert!(export_retention_closure.contains("public_behind_private"));

    let full = BTreeMap::from([
        (
            "f",
            vec![
                Candidate {
                    identity: 1,
                    export_root: true,
                    internal_policy: runtime_value(BTreeSet::new()),
                },
                Candidate {
                    identity: 2,
                    export_root: false,
                    internal_policy: runtime_value(mutability(&[ValueMutability::Mut])),
                },
            ],
        ),
        (
            "exported_child",
            vec![Candidate {
                identity: 4,
                export_root: false,
                internal_policy: runtime_value(mutability(&[
                    ValueMutability::Const,
                    ValueMutability::Mut,
                ])),
            }],
        ),
        (
            "private_child",
            vec![Candidate {
                identity: 3,
                export_root: false,
                internal_policy: runtime_value(BTreeSet::new()),
            }],
        ),
        (
            "public_behind_private",
            vec![Candidate {
                identity: 7,
                export_root: false,
                internal_policy: runtime_value(BTreeSet::new()),
            }],
        ),
        (
            "exported_type",
            vec![Candidate {
                identity: 5,
                export_root: false,
                internal_policy: type_only(),
            }],
        ),
        (
            "private_dependency",
            vec![Candidate {
                identity: 8,
                export_root: false,
                internal_policy: runtime_value(BTreeSet::new()),
            }],
        ),
    ]);
    let views = project_export_overload_sets(
        full,
        |name| ExportAdmission {
            in_export_retention_closure: export_retention_closure.contains(*name),
            publicly_reachable: publicly_reachable(&nodes, namespace_path(&nodes, *name)),
        },
        |candidate| {
            (
                candidate.identity,
                ResolvedCandidatePolicy {
                    pair: candidate.internal_policy.clone(),
                    mode: lang_build::concrete_policy_mode(&candidate.internal_policy.value),
                    capability_realization: CapabilityRealization::default(),
                    provenance: Provenance::new(format!(
                        "resolved candidate {}",
                        candidate.identity
                    )),
                },
            )
        },
    )
    .expect("all resolved candidates satisfy the value-component invariant");

    assert_eq!(
        views
            .resolve_internal(&"f")
            .expect("full overload set")
            .iter()
            .map(|candidate| candidate.identity)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    let external_f = views.resolve_external(&"f").expect("export candidate view");
    assert_eq!(
        external_f
            .iter()
            .map(|candidate| candidate.identity)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert_eq!(external_f[0].internal_candidate.identity, 1);
    assert_eq!(
        external_f[0].external_policy, external_f[0].internal_candidate.internal_policy,
        "external admission preserves the complete internal Policy pair"
    );
    assert_eq!(
        external_f[0].external_policy.pattern,
        external_f[0].internal_candidate.internal_policy.pattern,
        "external projection preserves the resolved associated Pp"
    );

    let external_child = views
        .resolve_external(&"exported_child")
        .expect("public retention-closure descendant receives an external candidate view");
    assert!(!external_child[0].internal_candidate.export_root);
    assert_eq!(
        external_child[0].external_policy, external_child[0].internal_candidate.internal_policy,
        "export does not crop a broad internal Policy domain"
    );
    assert_eq!(
        external_child[0].external_policy.pattern,
        external_child[0].internal_candidate.internal_policy.pattern
    );

    assert!(views.resolve_internal(&"private_child").is_some());
    assert!(
        views.resolve_external(&"private_child").is_none(),
        "a private export-retention-closure member is not externally exposed"
    );
    assert!(views.resolve_internal(&"public_behind_private").is_some());
    assert!(
        views.resolve_external(&"public_behind_private").is_none(),
        "a public descendant behind a private path is not externally exposed"
    );

    let external_type = views
        .resolve_external(&"exported_type")
        .expect("pure Pattern/type candidate remains externally visible");
    assert_eq!(
        external_type[0].external_policy, external_type[0].internal_candidate.internal_policy,
        "Pv=absent has no value-mutability projection obligation"
    );

    assert!(views.resolve_internal(&"private_dependency").is_some());
    assert!(views.resolve_external(&"private_dependency").is_none());

    let wpre = compute_wpre(
        WpreRoots {
            exported_symbols: vec!["f"],
            materialized_results_of_exported_meta_functions: vec![],
            parameter_dependencies_of_exported_meta_functions: vec![],
        },
        |symbol| {
            if *symbol == "f" {
                vec![
                    "private_child",
                    "public_behind_private",
                    "private_dependency",
                ]
            } else {
                vec![]
            }
        },
    );
    assert!(
        wpre.contains("private_dependency"),
        "Wpre may retain a private semantic dependency"
    );
    assert!(wpre.contains("private_child"));
    assert!(wpre.contains("public_behind_private"));
    assert!(
        views.resolve_external(&"private_dependency").is_none(),
        "world membership must not install an external export view"
    );

    let mut_only = BTreeMap::from([(
        "mut_only_member",
        vec![Candidate {
            identity: 6,
            export_root: false,
            internal_policy: runtime_value(mutability(&[ValueMutability::Mut])),
        }],
    )]);
    let mut_only_views = project_export_overload_sets(
        mut_only,
        |_| ExportAdmission {
            in_export_retention_closure: true,
            publicly_reachable: true,
        },
        |candidate| {
            (
                candidate.identity,
                ResolvedCandidatePolicy {
                    pair: candidate.internal_policy.clone(),
                    mode: PolicyMode::Mut,
                    capability_realization: CapabilityRealization::default(),
                    provenance: Provenance::new(format!(
                        "resolved candidate {}",
                        candidate.identity
                    )),
                },
            )
        },
    )
    .expect("mut-only is a stable externally admitted Policy view");
    assert!(
        mut_only_views
            .resolve_internal(&"mut_only_member")
            .is_some(),
        "a mut-only overload remains in Sigma_full"
    );
    assert_eq!(
        mut_only_views
            .resolve_external(&"mut_only_member")
            .expect("mut-only candidate remains in Sigma_export")[0]
            .external_policy
            .value
            .mutability,
        mutability(&[ValueMutability::Mut]),
        "export must preserve, not crop, the mut Policy view"
    );
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
            mutability: mutability(&[ValueMutability::Const]),
        },
    );
    assert_eq!(
        object.value.stages,
        stages(&[PolicyStage::Seal, PolicyStage::Runtime])
    );
    assert_eq!(object.pattern.stages, stages(&[PolicyStage::Seal]));
    assert_eq!(
        object.value.mutability,
        mutability(&[ValueMutability::Const])
    );
    assert!(!object.value.mutability.contains(&ValueMutability::Mut));

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
    assert!(
        object.value.mutability.is_empty(),
        "empty function-object mutability is the unconstrained const || mut domain"
    );
    let const_projection = elaborate_binding_p1_projection(
        Some(&policy_spec("const")),
        Provenance::new("const function-object P1"),
    )
    .expect("const P1 projection");
    let object_entry = PolicyResultEntry {
        value: Some("function-object"),
        value_policy: object.value.clone(),
        pattern: "function-pattern",
        pattern_policy: object.pattern.clone(),
    };
    let selected = project_p1(&const_projection, &[object_entry]);
    assert_eq!(selected.len(), 1);
    assert_eq!(
        selected[0].value_policy.mutability,
        mutability(&[ValueMutability::Const]),
        "an explicit declaration/P1 restriction crops the unconstrained domain"
    );
}

#[test]
fn namespace_attributes_never_change_the_canonical_function_object_pair() {
    let result = normalize_p2_policy(
        &policy_spec("meta"),
        Provenance::new("meta result for declaration-attribute separation"),
    )
    .expect("valid result policy");
    let elaborate = |source: &str| {
        elaborate_namespace_declaration_policy(
            Some(&policy_spec(source)),
            NamespaceDeclarationPosition::DirectTopLevel,
            Provenance::new(source),
        )
        .expect("valid namespace declaration")
    };
    let public = elaborate("public + meta");
    let private = elaborate("private + meta");
    let export = elaborate("export + public + meta");

    let public_p1 =
        derive_function_object_p1(&result, &function_object_declaration_policy(&public));
    let private_p1 =
        derive_function_object_p1(&result, &function_object_declaration_policy(&private));
    let export_p1 =
        derive_function_object_p1(&result, &function_object_declaration_policy(&export));
    assert_eq!(public_p1, private_p1);
    assert_eq!(public_p1, export_p1);
    assert_ne!(public.visibility, private.visibility);
    assert!(export.export_root);
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
fn export_retention_closure_and_public_path_reachability_are_independent() {
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
    let export_retention_closure = compute_export_retention_closure(&nodes, [1]);
    assert_eq!(export_retention_closure, BTreeSet::from([0, 1, 3, 4]));
    assert!(
        !export_retention_closure.contains(&2),
        "siblings do not enter the export-retention closure"
    );
    assert!(publicly_reachable(&nodes, [0, 1]));
    assert!(!publicly_reachable(&nodes, [0, 1, 3]));
    assert!(
        export_retention_closure.contains(&4),
        "private descendants remain export-retention-closure members"
    );
    assert!(!externally_visible(
        &4,
        &export_retention_closure,
        &nodes,
        [0, 1, 3, 4]
    ));
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
    frame_patterns: Vec<MutabilityPattern>,
    is_delete: bool,
) -> PolicyOverloadCandidate<&'static str> {
    let mut frame_patterns = frame_patterns.into_iter();
    PolicyOverloadCandidate {
        id,
        formal_frame: MutabilityFormalFrame {
            self_pattern: frame_patterns.next().unwrap_or(MutabilityPattern::Plain),
            explicit_parameter_patterns: frame_patterns.collect(),
        },
        result_policy: PolicyMode::Plain,
        is_delete,
    }
}

fn actual_frame(
    caller_value: ValueMutability,
    explicit_arguments: Vec<ValueMutability>,
) -> MutabilityActualFrame {
    MutabilityActualFrame {
        caller_value,
        explicit_arguments,
    }
}

#[test]
fn const_mut_selection_uses_product_partial_order_and_delete_is_normal() {
    let single = vec![
        candidate("const", vec![MutabilityPattern::Const], false),
        candidate("plain", vec![MutabilityPattern::Plain], false),
        candidate("mut", vec![MutabilityPattern::Mut], false),
    ];
    assert_eq!(
        select_by_mutability_product(
            &single,
            &actual_frame(ValueMutability::Const, vec![]),
            OutputModeDemand::default(),
        ),
        PolicyOverloadSelection::Selected("const")
    );
    assert_eq!(
        select_by_mutability_product(
            &single,
            &actual_frame(ValueMutability::Mut, vec![]),
            OutputModeDemand::default(),
        ),
        PolicyOverloadSelection::Selected("mut")
    );

    let crossed = vec![
        candidate(
            "left",
            vec![MutabilityPattern::Const, MutabilityPattern::Plain],
            false,
        ),
        candidate(
            "right",
            vec![MutabilityPattern::Plain, MutabilityPattern::Const],
            false,
        ),
    ];
    assert!(matches!(
        select_by_mutability_product(
            &crossed,
            &actual_frame(ValueMutability::Const, vec![ValueMutability::Const]),
            OutputModeDemand::default()
        ),
        PolicyOverloadSelection::Ambiguous(_)
    ));

    let delete = vec![
        candidate("const-delete", vec![MutabilityPattern::Const], true),
        candidate("plain", vec![MutabilityPattern::Plain], false),
    ];
    assert_eq!(
        select_by_mutability_product(
            &delete,
            &actual_frame(ValueMutability::Const, vec![]),
            OutputModeDemand::default(),
        ),
        PolicyOverloadSelection::RejectedByDelete("const-delete")
    );
}

#[test]
fn formal_p2_mutability_slice_is_exported_to_the_overload_product_order() {
    let inherited_p2 = normalize_p2_policy(
        &policy_spec("runtime:compile"),
        Provenance::new("overload formal P2"),
    )
    .expect("valid inherited P2");
    let const_formal = elaborate_formal_policy_pattern(
        Some(&policy_spec("const")),
        &inherited_p2,
        Provenance::new("const formal"),
    )
    .expect("const formal");
    let plain_formal =
        elaborate_formal_policy_pattern(None, &inherited_p2, Provenance::new("plain formal"))
            .expect("plain formal");
    let mut_formal = elaborate_formal_policy_pattern(
        Some(&policy_spec("mut")),
        &inherited_p2,
        Provenance::new("mut formal"),
    )
    .expect("mut formal");

    let split = PolicyOverloadCandidate::from_formal_patterns(
        "split",
        &[const_formal.clone(), mut_formal.clone()],
        PolicyMode::Plain,
        false,
    );
    assert_eq!(
        split.formal_frame,
        MutabilityFormalFrame {
            self_pattern: MutabilityPattern::Const,
            explicit_parameter_patterns: vec![MutabilityPattern::Mut],
        },
        "the first written formal is the self policy position; only later formals consume explicit arguments"
    );

    let candidates = vec![
        PolicyOverloadCandidate::from_formal_patterns(
            "const",
            &[const_formal],
            PolicyMode::Plain,
            false,
        ),
        PolicyOverloadCandidate::from_formal_patterns(
            "plain",
            &[plain_formal],
            PolicyMode::Plain,
            false,
        ),
        PolicyOverloadCandidate::from_formal_patterns(
            "mut",
            &[mut_formal],
            PolicyMode::Plain,
            false,
        ),
    ];

    assert_eq!(
        select_by_mutability_product(
            &candidates,
            &actual_frame(ValueMutability::Const, vec![]),
            OutputModeDemand::default()
        ),
        PolicyOverloadSelection::Selected("const")
    );
    assert_eq!(
        select_by_mutability_product(
            &candidates,
            &actual_frame(ValueMutability::Mut, vec![]),
            OutputModeDemand::default()
        ),
        PolicyOverloadSelection::Selected("mut")
    );
}

#[test]
fn total_output_mode_demand_orders_candidate_results() {
    let candidates = vec![
        PolicyOverloadCandidate {
            id: "const-result",
            formal_frame: MutabilityFormalFrame {
                self_pattern: MutabilityPattern::Plain,
                explicit_parameter_patterns: vec![],
            },
            result_policy: PolicyMode::Const,
            is_delete: false,
        },
        PolicyOverloadCandidate {
            id: "mut-result",
            formal_frame: MutabilityFormalFrame {
                self_pattern: MutabilityPattern::Plain,
                explicit_parameter_patterns: vec![],
            },
            result_policy: PolicyMode::Mut,
            is_delete: false,
        },
    ];
    assert!(matches!(
        select_by_mutability_product(
            &candidates,
            &actual_frame(ValueMutability::Const, vec![]),
            OutputModeDemand::default()
        ),
        PolicyOverloadSelection::Ambiguous(_)
    ));
    assert_eq!(
        select_by_mutability_product(
            &candidates,
            &actual_frame(ValueMutability::Const, vec![]),
            OutputModeDemand(PolicyMode::Const)
        ),
        PolicyOverloadSelection::Selected("const-result")
    );
}

#[test]
fn policy_mode_is_a_real_three_point_preference_and_plain_is_not_a_wildcard() {
    let candidates = [PolicyMode::Const, PolicyMode::Plain, PolicyMode::Mut]
        .into_iter()
        .map(|mode| PolicyOverloadCandidate {
            id: mode,
            formal_frame: MutabilityFormalFrame {
                self_pattern: PolicyMode::Plain,
                explicit_parameter_patterns: vec![],
            },
            result_policy: mode,
            is_delete: false,
        })
        .collect::<Vec<_>>();
    let actual = actual_frame(PolicyMode::Plain, vec![]);

    for demand in [PolicyMode::Const, PolicyMode::Plain, PolicyMode::Mut] {
        assert_eq!(
            select_by_mutability_product(&candidates, &actual, OutputModeDemand(demand),),
            PolicyOverloadSelection::Selected(demand),
            "the exact point must win for every total output demand"
        );
    }

    let endpoints_only = candidates
        .iter()
        .filter(|candidate| candidate.result_policy != PolicyMode::Plain)
        .cloned()
        .collect::<Vec<_>>();
    assert!(matches!(
        select_by_mutability_product(
            &endpoints_only,
            &actual,
            OutputModeDemand(PolicyMode::Plain),
        ),
        PolicyOverloadSelection::Ambiguous(ref ids)
            if ids.contains(&PolicyMode::Const) && ids.contains(&PolicyMode::Mut)
    ));
}

#[test]
fn capability_realization_is_a_complete_policy_orthogonal_three_by_three_grid() {
    let mut realization = CapabilityRealization::default();
    assert_eq!(realization.iter().count(), 9);
    assert!(realization
        .iter()
        .all(|(_, cell)| cell == CapabilityRealizationCell::Absent));

    realization.set(
        PolicyMode::Const,
        PolicyMode::Mut,
        CapabilityRealizationCell::Delete,
    );
    realization.set(
        PolicyMode::Mut,
        PolicyMode::Const,
        CapabilityRealizationCell::Custom,
    );
    realization.set(
        PolicyMode::Plain,
        PolicyMode::Plain,
        CapabilityRealizationCell::Default,
    );

    assert_eq!(
        realization.cell(PolicyMode::Const, PolicyMode::Mut),
        CapabilityRealizationCell::Delete
    );
    assert_eq!(
        realization.cell(PolicyMode::Mut, PolicyMode::Const),
        CapabilityRealizationCell::Custom
    );
    assert_eq!(
        realization.cell(PolicyMode::Plain, PolicyMode::Plain),
        CapabilityRealizationCell::Default
    );
    assert_eq!(
        realization.cell(PolicyMode::Mut, PolicyMode::Mut),
        CapabilityRealizationCell::Absent,
        "Policy preference cannot synthesize an unconfigured capability cell"
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
        select_policy_overload(
            &open,
            &actual_frame(ValueMutability::Const, vec![]),
            OutputModeDemand::default(),
            Phase::OpenStatic
        ),
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
        select_policy_overload(
            &seal,
            &actual_frame(ValueMutability::Mut, vec![]),
            OutputModeDemand::default(),
            Phase::SealStatic
        ),
        PolicyOverloadSelection::Selected("seal")
    );

    let crossed = vec![
        PhaseOverloadCandidate {
            candidate: candidate("meta-plain", vec![MutabilityPattern::Plain], false),
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
        select_policy_overload(
            &crossed,
            &actual_frame(ValueMutability::Const, vec![]),
            OutputModeDemand::default(),
            Phase::OpenStatic
        ),
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
