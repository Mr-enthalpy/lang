use std::collections::{BTreeMap, BTreeSet};

use lang_build::{
    compute_export_retention_closure, compute_wpre,
    derive_function_object_view as derive_function_object_p1, elaborate_binding_result_demand,
    elaborate_formal_policy_pattern, elaborate_namespace_declaration_policy,
    elaborate_return_policy_pattern, expose_policy_slice, externally_visible,
    function_object_declaration_policy, normalize_p2_policy, project_export_overload_sets,
    project_p1, publicly_reachable, read_pattern, read_value, select_by_policy_product,
    CapabilityRealization, CapabilityRealizationCell, ExportAdmission,
    FunctionObjectDeclarationPolicy, NamespaceDeclarationPosition, NamespaceExportNode,
    NamespaceVisibility, ObjectPlaceId, ObservationHorizon, OutputModeDemand, P1Projection,
    PatternComponentPolicy, PolicyActualFrame, PolicyFormalFrame, PolicyMode,
    PolicyOverloadCandidate, PolicyOverloadSelection, PolicyPair, PolicyResultEntry, PolicyView,
    Provenance, ResolvedCandidatePolicy, Stage, ValueComponentPolicy, WpreRoots, WritableContext,
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

fn elaborate_binding_p1_projection(
    policy: Option<&NormPolicySpec>,
    provenance: Provenance,
) -> Result<P1Projection, lang_build::Diagnostic> {
    elaborate_binding_result_demand(policy, provenance).map(|demand| demand.pair_query)
}

fn result_entry<V, P>(
    value: Option<V>,
    value_stage: Stage,
    pattern: P,
    pattern_stage: Stage,
) -> PolicyResultEntry<V, P> {
    PolicyResultEntry {
        value,
        pattern,
        view: PolicyView {
            pair: PolicyPair {
                value: ValueComponentPolicy::Present(value_stage),
                pattern: PatternComponentPolicy {
                    stage: pattern_stage,
                },
            },
            mode: PolicyMode::Plain,
        },
    }
}

#[test]
fn policy_pair_and_whole_slot_mode_are_orthogonal_facts() {
    let pair = PolicyPair {
        value: ValueComponentPolicy::Present(Stage::Compile),
        pattern: PatternComponentPolicy {
            stage: Stage::Compile,
        },
    };
    let plain = PolicyView {
        pair: pair.clone(),
        mode: PolicyMode::Plain,
    };
    let constant = PolicyView {
        pair: pair.clone(),
        mode: PolicyMode::Const,
    };
    assert_ne!(plain, constant, "same pair does not erase whole-slot mode");

    let mut different_pair = pair;
    different_pair.value = ValueComponentPolicy::Present(Stage::Runtime);
    assert_ne!(
        PolicyView {
            pair: different_pair,
            mode: PolicyMode::Plain,
        },
        plain,
        "same mode does not erase pair coordinates"
    );

    let omitted = elaborate_binding_result_demand(None, Provenance::new("omitted mode"))
        .expect("omitted demand is total");
    assert_eq!(omitted.pair_query, P1Projection::Infer);
    assert_eq!(omitted.mode, PolicyMode::Plain);
}

#[test]
fn public_policy_constraints_preserve_conjunction_and_reject_pair_choice() {
    let view = normalize_p2_policy(
        &policy_spec("const + runtime"),
        Provenance::new("conjunction"),
    )
    .unwrap();
    assert_eq!(view.mode, PolicyMode::Const);
    for source in [
        "runtime:compile",
        "runtime:seal",
        "runtime || compile",
        "const || mut",
        "S : compile",
    ] {
        let parsed = lang_syntax::parse(&format!("{source} let x = value;"));
        assert!(!parsed.diagnostics.is_empty(), "{source}");
    }
    assert!(
        lang_syntax::parse("let bool = ((if | else) bool) |> struct;")
            .diagnostics
            .is_empty()
    );
}

#[test]
fn policy_algebra_rejects_same_dimension_conjunction() {
    for source in [
        "const + mut",
        "public + private",
        "meta + compile",
        "compile + seal",
        "runtime + compile",
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
        ("meta", Stage::Meta, Stage::Meta),
        ("compile", Stage::Compile, Stage::Compile),
        ("seal", Stage::Seal, Stage::Seal),
        ("runtime", Stage::Runtime, Stage::Compile),
    ];

    for (source, value_stage, pattern_stage) in cases {
        let view =
            normalize_p2_policy(&policy_spec(source), Provenance::new(source)).expect("valid P2");
        assert_eq!(view.pair.value.stage(), Some(value_stage));
        assert_eq!(view.pair.pattern.stage, pattern_stage);
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
        Stage::Runtime,
        "same-pattern",
        Stage::Compile,
    )];
    let selected = project_p1(&projection, &result);
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].value, Some("same-symbol"));
    assert_eq!(selected[0].pattern, "same-pattern");
    assert_eq!(selected[0].view.pair.value.stage(), Some(Stage::Runtime));
    assert_eq!(selected[0].view.pair.pattern.stage, Stage::Compile);

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
        &policy_spec("runtime"),
        Provenance::new("formal inherited P2"),
    )
    .expect("valid inherited P2");
    let plain =
        elaborate_formal_policy_pattern(None, &inherited_p2, Provenance::new("plain formal"))
            .expect("omitted formal policy inherits P2");
    assert_eq!(plain.effective_pair, inherited_p2.pair);
    assert_eq!(plain.mode, PolicyMode::Plain);

    let formal = elaborate_formal_policy_pattern(
        Some(&policy_spec("const")),
        &inherited_p2,
        Provenance::new("formal"),
    )
    .expect("const formal pattern");
    assert_eq!(formal.mode, PolicyMode::Const);
    assert_eq!(
        formal.effective_pair.value.stage(),
        inherited_p2.pair.value.stage()
    );
    assert_eq!(
        formal.effective_pair.pattern, inherited_p2.pair.pattern,
        "formal const/mut syntax must not change the inherited Pattern policy"
    );
    assert_eq!(
        formal.effective_pair.value.presence(),
        inherited_p2.pair.value.presence()
    );
    assert_eq!(formal.mode, PolicyMode::Const);

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
    let const_only_p2 = normalize_p2_policy(
        &policy_spec("const + runtime"),
        Provenance::new("const-only inherited P2"),
    )
    .expect("valid const-only P2");
    let mut_formal = elaborate_formal_policy_pattern(
        Some(&policy_spec("mut")),
        &const_only_p2,
        Provenance::new("expanding mut formal"),
    )
    .expect("formal whole-slot mode is independent of inherited pair");
    assert_eq!(mut_formal.mode, PolicyMode::Mut);
    assert_eq!(mut_formal.effective_pair, const_only_p2.pair);

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
    assert_eq!(value.stage, Some(Stage::Runtime));
    assert_eq!(declaration.mode, PolicyMode::Plain);
    let Some(P1Projection::ValueDominant {
        value: external_value,
    }) = &declaration.external_projection
    else {
        panic!("export root must carry an external value view");
    };
    assert_eq!(external_value, value);
    let function_declaration = function_object_declaration_policy(&declaration);
    assert_eq!(function_declaration.mode, PolicyMode::Plain);

    let explicit_const = elaborate_namespace_declaration_policy(
        Some(&policy_spec("export + const + runtime")),
        NamespaceDeclarationPosition::DirectTopLevel,
        Provenance::new("explicit const export"),
    )
    .expect("export + const is valid");
    let P1Projection::ValueDominant { .. } = explicit_const.projection else {
        panic!("single namespace policy must elaborate as value-dominant P1");
    };
    assert_eq!(explicit_const.mode, PolicyMode::Const);
    assert!(matches!(
        explicit_const.external_projection,
        Some(P1Projection::ValueDominant { .. })
    ));

    let mut_only_export = elaborate_namespace_declaration_policy(
        Some(&policy_spec("export + mut + runtime")),
        NamespaceDeclarationPosition::DirectTopLevel,
        Provenance::new("mut-only export"),
    )
    .expect("export admission does not const-crop or reject a mut Policy view");
    assert_eq!(mut_only_export.mode, PolicyMode::Mut);
    assert!(matches!(
        mut_only_export.external_projection,
        Some(P1Projection::ValueDominant { .. })
    ));

    assert!(elaborate_namespace_declaration_policy(
        Some(&policy_spec("export + runtime")),
        NamespaceDeclarationPosition::Local,
        Provenance::new("local export"),
    )
    .is_err());
}

#[test]
fn explicit_pin_stages_report_unconnected_input_admissibility() {
    for inherited in ["meta", "compile", "seal", "runtime"] {
        let p2 = normalize_p2_policy(&policy_spec(inherited), Provenance::new(inherited)).unwrap();
        let original = p2.clone();
        for source in ["meta", "runtime", "compile", "seal", "const + runtime"] {
            let provenance = Provenance::new(source);
            let diagnostic = elaborate_formal_policy_pattern(
                Some(&policy_spec(source)),
                &p2,
                provenance.clone(),
            )
            .expect_err("explicit Pin stage needs the InputAdmissible consumer");
            assert_eq!(
                diagnostic.code,
                Some(lang_build::ResolverCode::UnsupportedInputAdmissibleStage)
            );
            assert!(diagnostic.message.contains("is canonical"));
            assert!(diagnostic.message.contains("consumer is not connected"));
            assert_eq!(diagnostic.provenance, Some(provenance));
            assert_eq!(
                p2, original,
                "a Pin constraint never rewrites the callable P2"
            );
        }
    }
}

#[test]
fn omitted_pin_inherits_p2_and_pout_inherits_p1_stage() {
    let inherited_p2 = normalize_p2_policy(
        &policy_spec("mut + runtime"),
        Provenance::new("position inherited P2"),
    )
    .expect("valid inherited P2");
    let omitted_formal = elaborate_formal_policy_pattern(
        None,
        &inherited_p2,
        Provenance::new("omitted formal inherits P2 mode"),
    )
    .expect("omitted formal position");
    assert_eq!(omitted_formal.effective_pair, inherited_p2.pair);
    assert_eq!(omitted_formal.mode, PolicyMode::Mut);

    let inherited_p1 = derive_function_object_p1(
        &inherited_p2,
        &FunctionObjectDeclarationPolicy {
            mode: PolicyMode::Const,
        },
    );
    let omitted_return = elaborate_return_policy_pattern(
        None,
        &inherited_p1,
        Provenance::new("omitted return inherits P1"),
    )
    .expect("omitted return position");
    assert_eq!(omitted_return.effective_view, inherited_p1);

    let mut_return = elaborate_return_policy_pattern(
        Some(&policy_spec("mut")),
        &inherited_p1,
        Provenance::new("return mode overlay"),
    )
    .expect("return mode may override inherited P1 mode");
    assert_eq!(mut_return.effective_view.pair, inherited_p1.pair);
    assert_eq!(mut_return.effective_view.mode, PolicyMode::Mut);

    for forbidden in ["meta", "compile", "runtime", "seal", "mut + runtime"] {
        assert!(
            elaborate_return_policy_pattern(
                Some(&policy_spec(forbidden)),
                &inherited_p1,
                Provenance::new(format!("forbidden return overlay {forbidden}")),
            )
            .is_err(),
            "return position `{forbidden}` must not rewrite inherited stage"
        );
    }

    let writable = WritableContext::default();
    assert!(
        !writable.place_is_writable(ObjectPlaceId(77)),
        "a mut return-position mode is an overload/view coordinate and grants no Writable fact"
    );
}

#[test]
fn export_overload_set_is_a_projection_of_the_full_set_not_a_second_world() {
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Candidate {
        identity: u32,
        export_root: bool,
        internal_policy: PolicyPair,
        mode: PolicyMode,
    }

    let runtime_value = || PolicyPair {
        value: ValueComponentPolicy::Present(Stage::Runtime),
        pattern: PatternComponentPolicy {
            stage: Stage::Compile,
        },
    };
    let type_only = || PolicyPair {
        value: ValueComponentPolicy::Absent,
        pattern: PatternComponentPolicy {
            stage: Stage::Compile,
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
                    internal_policy: runtime_value(),
                    mode: PolicyMode::Plain,
                },
                Candidate {
                    identity: 2,
                    export_root: false,
                    internal_policy: runtime_value(),
                    mode: PolicyMode::Mut,
                },
            ],
        ),
        (
            "exported_child",
            vec![Candidate {
                identity: 4,
                export_root: false,
                internal_policy: runtime_value(),
                mode: PolicyMode::Plain,
            }],
        ),
        (
            "private_child",
            vec![Candidate {
                identity: 3,
                export_root: false,
                internal_policy: runtime_value(),
                mode: PolicyMode::Plain,
            }],
        ),
        (
            "public_behind_private",
            vec![Candidate {
                identity: 7,
                export_root: false,
                internal_policy: runtime_value(),
                mode: PolicyMode::Plain,
            }],
        ),
        (
            "exported_type",
            vec![Candidate {
                identity: 5,
                export_root: false,
                internal_policy: type_only(),
                mode: PolicyMode::Plain,
            }],
        ),
        (
            "private_dependency",
            vec![Candidate {
                identity: 8,
                export_root: false,
                internal_policy: runtime_value(),
                mode: PolicyMode::Plain,
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
                    mode: candidate.mode,
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
        "Pv=absent has no value-policy_mode projection obligation"
    );

    assert!(views.resolve_internal(&"private_dependency").is_some());
    assert!(views.resolve_external(&"private_dependency").is_none());

    let wpre = compute_wpre(
        WpreRoots {
            exported_symbols: vec!["f"],
            materialized_results_of_exported_callables: vec![],
            parameter_dependencies_of_exported_callables: vec![],
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
            internal_policy: runtime_value(),
            mode: PolicyMode::Mut,
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
            .internal_candidate
            .mode,
        PolicyMode::Mut,
        "export must preserve, not crop, the mut Policy view"
    );
}

#[test]
fn omitted_p1_completes_to_one_p2_stage() {
    let result = PolicyView {
        pair: PolicyPair {
            value: ValueComponentPolicy::Present(Stage::Runtime),
            pattern: PatternComponentPolicy { stage: Stage::Seal },
        },
        mode: PolicyMode::Const,
    };
    let object = derive_function_object_p1(
        &result,
        &FunctionObjectDeclarationPolicy {
            mode: PolicyMode::Const,
        },
    );
    assert_eq!(object.pair.value.stage(), Some(Stage::Runtime));
    assert_eq!(object.pair.pattern.stage, Stage::Seal);
    assert_eq!(object.mode, PolicyMode::Const);

    let compile = normalize_p2_policy(&policy_spec("runtime"), Provenance::new("compile result"))
        .expect("valid result policy");
    let object = derive_function_object_p1(&compile, &FunctionObjectDeclarationPolicy::default());
    assert_eq!(object.pair.value.stage(), Some(Stage::Runtime));
    assert_eq!(object.pair.pattern.stage, Stage::Compile);
    assert_eq!(object.mode, PolicyMode::Plain);
    let const_projection = elaborate_binding_p1_projection(
        Some(&policy_spec("const")),
        Provenance::new("const function-object P1"),
    )
    .expect("const P1 projection");
    let object_entry = PolicyResultEntry {
        value: Some("function-object"),
        pattern: "function-pattern",
        view: object.clone(),
    };
    let selected = project_p1(&const_projection, &[object_entry]);
    assert_eq!(selected.len(), 1);
    assert_eq!(
        selected[0].view.mode,
        PolicyMode::Plain,
        "a pair projection does not manufacture or rewrite whole-slot mode"
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
fn horizon_visibility_uses_visibility_domains_not_atom_intersection() {
    assert!(Stage::Meta.visible_at(ObservationHorizon::OpenStatic));
    assert!(!Stage::Meta.visible_at(ObservationHorizon::SealStatic));
    assert!(Stage::Compile.visible_at(ObservationHorizon::OpenStatic));
    assert!(Stage::Compile.visible_at(ObservationHorizon::SealStatic));
    assert!(!Stage::Compile.visible_at(ObservationHorizon::Runtime));
    assert!(!Stage::Seal.visible_at(ObservationHorizon::OpenStatic));
    assert!(Stage::Seal.visible_at(ObservationHorizon::SealStatic));
    assert!(Stage::Runtime.visible_at(ObservationHorizon::Runtime));
}

#[test]
fn fixed_runtime_value_observation_exposes_only_static_pattern() {
    let entry = result_entry(
        Some("runtime computation"),
        Stage::Runtime,
        "compile Pattern",
        Stage::Compile,
    );
    let exposed = expose_policy_slice(&entry, ObservationHorizon::OpenStatic);
    assert!(read_value(&exposed).is_none());
    assert_eq!(read_pattern(&exposed), Some(&"compile Pattern"));
    assert_eq!(
        entry.view.pair.value.stage(),
        Some(Stage::Runtime),
        "static projection must not consume the runtime computation"
    );
}

#[test]
fn wpre_is_the_least_semantic_dependency_closure_of_export_roots() {
    let roots = WpreRoots {
        exported_symbols: vec!["api"],
        materialized_results_of_exported_callables: vec!["made"],
        parameter_dependencies_of_exported_callables: vec!["parameter"],
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

fn candidate(
    id: &'static str,
    frame_patterns: Vec<PolicyMode>,
    is_delete: bool,
) -> PolicyOverloadCandidate<&'static str> {
    let mut frame_patterns = frame_patterns.into_iter();
    PolicyOverloadCandidate {
        id,
        formal_frame: PolicyFormalFrame {
            self_mode: frame_patterns.next().unwrap_or(PolicyMode::Plain),
            explicit_parameter_modes: frame_patterns.collect(),
        },
        result_policy: PolicyMode::Plain,
        is_delete,
    }
}

fn actual_frame(
    caller_value: PolicyMode,
    explicit_arguments: Vec<PolicyMode>,
) -> PolicyActualFrame {
    PolicyActualFrame {
        caller_value,
        explicit_arguments,
    }
}

#[test]
fn const_mut_selection_uses_product_partial_order_and_delete_is_normal() {
    let single = vec![
        candidate("const", vec![PolicyMode::Const], false),
        candidate("plain", vec![PolicyMode::Plain], false),
        candidate("mut", vec![PolicyMode::Mut], false),
    ];
    assert_eq!(
        select_by_policy_product(
            &single,
            &actual_frame(PolicyMode::Const, vec![]),
            OutputModeDemand::default(),
        ),
        PolicyOverloadSelection::Selected("const")
    );
    assert_eq!(
        select_by_policy_product(
            &single,
            &actual_frame(PolicyMode::Mut, vec![]),
            OutputModeDemand::default(),
        ),
        PolicyOverloadSelection::Selected("mut")
    );

    let crossed = vec![
        candidate("left", vec![PolicyMode::Const, PolicyMode::Plain], false),
        candidate("right", vec![PolicyMode::Plain, PolicyMode::Const], false),
    ];
    assert!(matches!(
        select_by_policy_product(
            &crossed,
            &actual_frame(PolicyMode::Const, vec![PolicyMode::Const]),
            OutputModeDemand::default()
        ),
        PolicyOverloadSelection::Ambiguous(_)
    ));

    let delete = vec![
        candidate("const-delete", vec![PolicyMode::Const], true),
        candidate("plain", vec![PolicyMode::Plain], false),
    ];
    assert_eq!(
        select_by_policy_product(
            &delete,
            &actual_frame(PolicyMode::Const, vec![]),
            OutputModeDemand::default(),
        ),
        PolicyOverloadSelection::RejectedByDelete("const-delete")
    );
}

#[test]
fn formal_p2_policy_mode_slice_is_exported_to_the_overload_product_order() {
    let inherited_p2 = normalize_p2_policy(
        &policy_spec("runtime"),
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
        PolicyFormalFrame {
            self_mode: PolicyMode::Const,
            explicit_parameter_modes: vec![PolicyMode::Mut],
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
        select_by_policy_product(
            &candidates,
            &actual_frame(PolicyMode::Const, vec![]),
            OutputModeDemand::default()
        ),
        PolicyOverloadSelection::Selected("const")
    );
    assert_eq!(
        select_by_policy_product(
            &candidates,
            &actual_frame(PolicyMode::Mut, vec![]),
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
            formal_frame: PolicyFormalFrame {
                self_mode: PolicyMode::Plain,
                explicit_parameter_modes: vec![],
            },
            result_policy: PolicyMode::Const,
            is_delete: false,
        },
        PolicyOverloadCandidate {
            id: "mut-result",
            formal_frame: PolicyFormalFrame {
                self_mode: PolicyMode::Plain,
                explicit_parameter_modes: vec![],
            },
            result_policy: PolicyMode::Mut,
            is_delete: false,
        },
    ];
    assert!(matches!(
        select_by_policy_product(
            &candidates,
            &actual_frame(PolicyMode::Const, vec![]),
            OutputModeDemand::default()
        ),
        PolicyOverloadSelection::Ambiguous(_)
    ));
    assert_eq!(
        select_by_policy_product(
            &candidates,
            &actual_frame(PolicyMode::Const, vec![]),
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
            formal_frame: PolicyFormalFrame {
                self_mode: PolicyMode::Plain,
                explicit_parameter_modes: vec![],
            },
            result_policy: mode,
            is_delete: false,
        })
        .collect::<Vec<_>>();
    let actual = actual_frame(PolicyMode::Plain, vec![]);

    for demand in [PolicyMode::Const, PolicyMode::Plain, PolicyMode::Mut] {
        assert_eq!(
            select_by_policy_product(&candidates, &actual, OutputModeDemand(demand),),
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
        select_by_policy_product(
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
