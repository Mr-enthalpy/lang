mod support;

use lang_build::*;

fn pair(value: ValueComponentPolicy, classifier: Stage) -> PolicyPair {
    PolicyPair {
        value,
        pattern: PatternComponentPolicy { stage: classifier },
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
    assert_eq!(project_p1(&runtime.pair_query, &entries), entries[..2]);
    assert_eq!(entries[0].view.pair.pattern.stage, Stage::Compile);
    assert_eq!(entries[1].value, None);
}

#[test]
fn inverted_rank_chain_never_yields_projection_export_observation_or_migration() {
    let invalid = pair(
        ValueComponentPolicy::Present(Stage::Compile),
        Stage::Runtime,
    );
    assert!(!invalid.is_rank_coherent());
    let entry = PolicyResultEntry {
        value: Some("resident"),
        pattern: "classifier",
        view: PolicyView {
            pair: invalid.clone(),
            mode: PolicyMode::Const,
        },
    };
    assert!(project_p1(&P1Projection::Infer, &[entry.clone()]).is_empty());
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
