mod support;

use lang_build::*;

#[test]
fn arbitrary_rank_prefixes_have_exactly_the_ordered_block_shape() {
    use PolicyRank::{Absent as A, Compile as C, Runtime as R};
    for absent in [0, 1, 3, 40] {
        for runtime in [0, 1, 4, 32] {
            for compile in [0, 1, 5, 64] {
                let ranks = [vec![A; absent], vec![R; runtime], vec![C; compile]].concat();
                let prefix = PolicyRankPrefix::new(ranks.clone()).unwrap();
                assert_eq!(prefix.ranks(), ranks);
                let expected = if runtime > 0 {
                    Some(false)
                } else if compile > 0 {
                    Some(true)
                } else {
                    None
                };
                assert_eq!(prefix.all_compile_query(), expected);
                assert_eq!(
                    prefix.compile_target().ranks(),
                    [vec![A; absent], vec![C; runtime + compile]].concat()
                );
            }
        }
    }
    for bad in [
        vec![R, A],
        vec![C, A],
        vec![C, R],
        vec![A, R, C, R],
        vec![A, A, R, R, A],
        vec![A, R, R, C, C, A],
    ] {
        assert!(PolicyRankPrefix::new(bad).is_none());
    }
}

#[test]
fn a_pair_window_cannot_equate_or_complete_distinct_higher_rank_prefixes() {
    use PolicyRank::{Absent as A, Compile as C, Runtime as R};
    let compile = PolicyRankPrefix::new(vec![A, A, C, C]).unwrap();
    let runtime = PolicyRankPrefix::new(vec![A, A, R, C]).unwrap();
    assert_eq!(compile.pair(), runtime.pair());
    assert_ne!(compile, runtime);
    assert_eq!(compile.all_compile_query(), Some(true));
    assert_eq!(runtime.all_compile_query(), Some(false));
    let pair = compile.pair().unwrap();
    assert_eq!(pair.leading_stage(), None);
    let entry = PolicyResultEntry {
        value: Some("retained resident"),
        pattern: "retained classifier",
        view: PolicyView {
            pair,
            mode: PolicyMode::Const,
        },
    };
    for horizon in [ObservationHorizon::OpenStatic, ObservationHorizon::Runtime] {
        let exposed = expose_policy_slice(&entry, horizon);
        assert!(read_value(&exposed).is_none());
        assert!(read_pattern(&exposed).is_none());
    }
    assert!(project_p1(&P1Projection::AllCompile, &[entry]).is_err());
}

#[test]
fn source_compile_demand_is_whole_chain_and_keeps_reverse_migration_unavailable() {
    let parsed = lang_syntax::parse("compile let result = input;");
    assert!(parsed.diagnostics.is_empty());
    let norm = lang_syntax::normalize_program(&parsed.program);
    let lang_syntax::NormForm::Let(lang_syntax::NormDecl::Let { slot, .. }) = &norm.forms[0] else {
        panic!("let");
    };
    let demand =
        elaborate_binding_result_demand(slot.policy.as_ref(), Provenance::new("compile demand"))
            .unwrap();
    assert_eq!(demand.pair_query, P1Projection::AllCompile);
    let rr = PolicyView {
        pair: pair(
            ValueComponentPolicy::Present(Stage::Runtime),
            Stage::Runtime,
        ),
        mode: PolicyMode::Const,
    };
    let target = rr.pair.compile_target();
    assert_eq!(
        target,
        declared_policy_view(Stage::Compile, PolicyMode::Const).pair
    );
    assert_eq!(
        PolicyMigrationRequest::new(
            rr,
            ResultPolicyDemand {
                pair_query: P1Projection::Pair(target),
                mode: PolicyMode::Const
            },
            support::type_lookup_fixture("whole rank compile"),
            SemanticValueId(1),
            Provenance::new("closed E unavailable")
        )
        .unwrap_err(),
        PolicyMigrationRequestFailure::ReifyBoundaryUnavailable
    );
}

#[test]
fn absent_classifier_cannot_follow_a_present_rank() {
    for stage in [Stage::Runtime, Stage::Compile] {
        let pair = PolicyPair {
            value: ValueComponentPolicy::Present(stage),
            pattern: PatternComponentPolicy { stage: None },
        };
        assert!(!pair.is_rank_prefix_coherent());
        let entry = PolicyResultEntry {
            value: Some("resident"),
            pattern: "classifier",
            view: PolicyView {
                pair,
                mode: PolicyMode::Const,
            },
        };
        assert!(project_p1(&P1Projection::Infer, &[entry.clone()]).is_err());
        let observed = expose_policy_slice(&entry, ObservationHorizon::OpenStatic);
        assert!(matches!(observed.pattern, FacetView::Unavailable { .. }));
        assert!(read_value(&observed).is_none());
    }
}

#[test]
fn ordinary_p1_qualifiers_do_not_rewrite_an_absent_prefix() {
    let rr = PolicyView {
        pair: pair(
            ValueComponentPolicy::Present(Stage::Runtime),
            Stage::Runtime,
        ),
        mode: PolicyMode::Const,
    };
    let aa = PolicyView {
        pair: PolicyPair {
            value: ValueComponentPolicy::Absent,
            pattern: PatternComponentPolicy { stage: None },
        },
        mode: PolicyMode::Const,
    };
    for spelling in ["compile", "runtime"] {
        let parsed = lang_syntax::parse(&format!("{spelling} let value=input;"));
        assert!(parsed.diagnostics.is_empty());
        let norm = lang_syntax::normalize_program(&parsed.program);
        let lang_syntax::NormForm::Let(lang_syntax::NormDecl::Let { slot, .. }) = &norm.forms[0]
        else {
            panic!("let");
        };
        let p = Provenance::new("source P1 scope");
        let selection = elaborate_explicit_p1(
            slot.policy.as_ref(),
            &rr.pair,
            ExplicitP1Position::OuterBinding,
            p.clone(),
        )
        .unwrap()
        .unwrap();
        let result = canonical_function_object_view(Some(&selection), &rr, &rr, None, &p).unwrap();
        assert_eq!(
            result.pair,
            if spelling == "compile" {
                rr.pair.compile_target()
            } else {
                rr.pair.clone()
            }
        );
        assert!(
            canonical_function_object_view(Some(&selection), &aa, &rr, None, &p)
                .unwrap_err()
                .message
                .contains("higher Policy rank")
        );
    }
}

#[test]
fn incomplete_rank_projection_does_not_return_a_partially_classified_result() {
    let value = SemanticValueRef {
        id: SemanticValueId(1),
        type_value: support::type_lookup_fixture("projection"),
    };
    let unknown = PolicyResultEntry {
        value: Some(value),
        pattern: PatternCoordinateId(1),
        view: PolicyView {
            pair: PolicyPair {
                value: ValueComponentPolicy::Absent,
                pattern: PatternComponentPolicy { stage: None },
            },
            mode: PolicyMode::Const,
        },
    };
    let known = PolicyResultEntry {
        value: Some(value),
        pattern: PatternCoordinateId(1),
        view: declared_policy_view(Stage::Compile, PolicyMode::Const),
    };
    let entries = [unknown, known];
    assert!(project_p1(&P1Projection::AllCompile, &entries)
        .unwrap_err()
        .message
        .contains("higher Policy rank"));
    assert!(matches!(
        elaborate_value_binding_p1(
            &entries,
            Some(&P1Projection::AllCompile),
            Provenance::new("binding projection")
        ),
        Err(P1ElaborationFailure::RankObservationUnavailable(_))
    ));
}

#[test]
fn an_unknown_body_horizon_is_unavailable_before_selection() {
    let base = CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
    let mut world = base.semantic_world().clone();
    let compile = declared_policy_view(Stage::Compile, PolicyMode::Const);
    let unknown = PolicyView {
        pair: PolicyPair {
            value: ValueComponentPolicy::Absent,
            pattern: PatternComponentPolicy { stage: None },
        },
        mode: PolicyMode::Const,
    };
    let registered = world
        .register_core_callable(
            base.package_root_node(),
            "unknown_body",
            SymbolId(900030),
            BuiltinCallableImpl::IdentityType,
            None,
            DeclaredResultClass::CompleteType,
            compile.clone(),
            unknown,
            compile,
            None,
            Provenance::new("unknown P2 prefix"),
        )
        .unwrap();
    let site = extract_single_call_site(&support::initializer_from_source(
        "let result=uint8 unknown_body;",
    ))
    .unwrap();
    let before = format!("{world:?}");
    let failure = invoke_resolved_binding_ordinary(
        &mut world,
        &[],
        registered.symbol,
        &site,
        &base.package_context(),
        OrdinaryInvocationContext::open_static(&[]),
        Provenance::new("body rank demand"),
    )
    .unwrap_err();
    let OrdinaryInvocationFailure::ApplicabilityUnsupported { diagnostic, trace } = failure else {
        panic!("{failure:?}");
    };
    assert!(diagnostic.message.contains("higher Policy rank"));
    assert!(trace.selected.is_none());
    assert_eq!(format!("{world:?}"), before);
}

fn pair(value: ValueComponentPolicy, classifier: Stage) -> PolicyPair {
    PolicyPair {
        value,
        pattern: PatternComponentPolicy {
            stage: Some(classifier),
        },
    }
}

#[test]
fn leading_stage_requests_preserve_absence_and_the_classifier_tail() {
    let cc = declared_policy_view(Stage::Compile, PolicyMode::Const).pair;
    let rc = declared_policy_view(Stage::Runtime, PolicyMode::Const).pair;
    let rr = pair(
        ValueComponentPolicy::Present(Stage::Runtime),
        Stage::Runtime,
    );
    assert_eq!(cc.with_leading_stage(Stage::Runtime), Some(rc.clone()));
    assert_eq!(rc.with_leading_stage(Stage::Runtime), Some(rc));
    assert_eq!(rr.with_leading_stage(Stage::Runtime), Some(rr.clone()));
    assert_eq!(rr.with_leading_stage(Stage::Compile), None);
    let hidden = pair(ValueComponentPolicy::Absent, Stage::Compile);
    assert_eq!(
        hidden.with_leading_stage(Stage::Runtime),
        Some(pair(ValueComponentPolicy::Absent, Stage::Runtime))
    );
    let optional = pair(
        ValueComponentPolicy::Optional(Stage::Compile),
        Stage::Compile,
    );
    assert_eq!(
        optional.with_leading_stage(Stage::Runtime),
        Some(pair(
            ValueComponentPolicy::Optional(Stage::Runtime),
            Stage::Compile
        ))
    );
}

#[test]
fn leading_projection_selects_existing_rank_facts_without_recoloring_or_identity_loss() {
    let runtime = ResultPolicyDemand {
        pair_query: P1Projection::Leading {
            stage: Some(Stage::Runtime),
        },
        mode: PolicyMode::Const,
    };
    let entries = [
        PolicyResultEntry {
            value: Some("object"),
            pattern: "its classifier",
            view: declared_policy_view(Stage::Runtime, PolicyMode::Const),
        },
        PolicyResultEntry {
            value: None,
            pattern: "higher classifier",
            view: PolicyView {
                pair: pair(ValueComponentPolicy::Absent, Stage::Runtime),
                mode: PolicyMode::Mut,
            },
        },
        PolicyResultEntry {
            value: Some("compile object"),
            pattern: "compile classifier",
            view: declared_policy_view(Stage::Compile, PolicyMode::Const),
        },
    ];
    assert_eq!(
        project_p1(&runtime.pair_query, &entries).unwrap(),
        entries[..2]
    );
    assert_eq!(entries[0].view.pair.pattern.stage, Some(Stage::Compile));
    assert_eq!(entries[1].value, None);
}

#[test]
fn inverted_rank_chain_never_yields_projection_export_observation_or_migration() {
    let invalid = pair(
        ValueComponentPolicy::Present(Stage::Compile),
        Stage::Runtime,
    );
    assert!(!invalid.is_rank_prefix_coherent());
    let entry = PolicyResultEntry {
        value: Some("resident"),
        pattern: "classifier",
        view: PolicyView {
            pair: invalid.clone(),
            mode: PolicyMode::Const,
        },
    };
    assert!(project_p1(&P1Projection::Infer, &[entry.clone()]).is_err());
    for horizon in [ObservationHorizon::OpenStatic, ObservationHorizon::Runtime] {
        let exposed = expose_policy_slice(&entry, horizon);
        assert!(read_value(&exposed).is_none());
        assert!(read_pattern(&exposed).is_none());
        assert!(matches!(exposed.pattern, FacetView::Unavailable { .. }));
    }
    assert!(project_resolved_export_view(&ResolvedCandidatePolicy {
        pair: invalid.clone(),
        mode: PolicyMode::Const,
        capability_realization: Default::default(),
        provenance: Provenance::new("invalid export"),
    })
    .is_err());
    let valid = declared_policy_view(Stage::Compile, PolicyMode::Const);
    for (source, target) in [(invalid.clone(), valid.pair.clone()), (valid.pair, invalid)] {
        assert_eq!(
            PolicyMigrationRequest::new(
                PolicyView {
                    pair: source,
                    mode: PolicyMode::Const
                },
                ResultPolicyDemand {
                    pair_query: P1Projection::Pair(target),
                    mode: PolicyMode::Const
                },
                support::type_lookup_fixture("incoherent policy"),
                SemanticValueId(1),
                Provenance::new("invalid migration"),
            )
            .unwrap_err(),
            PolicyMigrationRequestFailure::IncoherentRankChain
        );
    }
}

#[test]
fn inverted_rank_import_or_binding_replacement_publishes_nothing() {
    let base = CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
    let mut world = base.semantic_world().clone();
    let resident = world
        .symbol_in_namespace(base.core_node(), "uint8")
        .unwrap()
        .type_resident()
        .unwrap();
    let type_value = world.type_for_pattern(resident.pattern).unwrap();
    let valid = declared_policy_view(Stage::Runtime, PolicyMode::Const);
    let value = world
        .install_ordinary_value(type_value, valid.clone(), Provenance::new("valid object"))
        .unwrap();
    let view = PolicyResultEntry {
        value: Some(SemanticValueRef {
            id: value,
            type_value,
        }),
        pattern: resident.pattern,
        view: valid,
    };
    world
        .bind_ordinary_new(
            base.package_root_node(),
            "resident",
            &[view.clone()],
            None,
            Provenance::new("bind"),
        )
        .unwrap();
    let invalid = PolicyView {
        pair: pair(
            ValueComponentPolicy::Present(Stage::Compile),
            Stage::Runtime,
        ),
        mode: PolicyMode::Const,
    };
    let before = format!("{world:?}");
    assert!(world
        .install_ordinary_value(
            type_value,
            invalid.clone(),
            Provenance::new("invalid import")
        )
        .is_none());
    let bad = PolicyResultEntry {
        view: invalid.clone(),
        ..view
    };
    assert!(matches!(
        world.bind_ordinary_new(
            base.package_root_node(),
            "invalid",
            &[bad.clone()],
            None,
            Provenance::new("invalid bind")
        ),
        Err(BindConflict::IncoherentPolicyRankChain)
    ));
    assert!(world
        .replace_binding_projection(
            base.package_root_node(),
            "resident",
            &[bad],
            Provenance::new("invalid replacement")
        )
        .is_none());
    assert!(world
        .register_core_callable(
            base.package_root_node(),
            "invalid_callable",
            SymbolId(900020),
            BuiltinCallableImpl::IdentityType,
            None,
            DeclaredResultClass::CompleteType,
            declared_policy_view(Stage::Compile, PolicyMode::Const),
            declared_policy_view(Stage::Runtime, PolicyMode::Const),
            invalid,
            None,
            Provenance::new("invalid result plane"),
        )
        .is_err());
    assert_eq!(format!("{world:?}"), before);
}
