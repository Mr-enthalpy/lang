mod support;
use lang_build::*;

#[test]
fn migration_selects_against_frozen_source_core_and_failure_publishes_nothing() {
    let mut family = support::AssociatedFamily::new(&[
        "mut let m=(mut let self,x):compile -> mut let r:Old:: => (\"held source\") delete;",
        "mut let m=(mut let self,x):compile -> mut let r:New:: => (\"successor source\") delete;",
    ]);
    let base = CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
    let core = base.core_node();
    let pattern = family.target_binding().type_resident_pattern().unwrap();
    let world = family.semantic_world_mut();
    let ty = world.type_for_pattern(pattern).unwrap();
    let rank = world.type_rank().unwrap();
    let view = declared_policy_view(Stage::Compile, PolicyMode::Const);
    let source = world
        .install_plain_value(ty, view.pair.clone(), Provenance::new("held source object"))
        .unwrap();
    let held = world.complete_type_for_value(source).unwrap().clone();
    world
        .bind_ordinary_new(
            core,
            "Old",
            &[PolicyResultEntry {
                value: None,
                pattern,
                view: view.clone(),
            }],
            Some(&held),
            Provenance::new("old classifier"),
        )
        .unwrap();
    let extra_type = world
        .type_for_pattern(
            world
                .symbol_in_namespace(core, "uint8")
                .unwrap()
                .type_resident_pattern()
                .unwrap(),
        )
        .unwrap();
    let extra = world
        .install_plain_value(
            extra_type,
            view.pair.clone(),
            Provenance::new("new Core member"),
        )
        .unwrap();
    world
        .associate_existing_value(pattern, "extra", extra)
        .unwrap();
    let successor = world.observe_complete_type(ty, None).unwrap();
    assert_ne!(held.core(), successor.core());
    world
        .register_type_symbol_with_complete_type(
            core,
            "New",
            SymbolId(880030),
            ty,
            Some(successor.whole()),
            rank,
            None,
            view.pair.clone(),
            Provenance::new("new classifier"),
        )
        .unwrap();
    let request = PolicyMigrationRequest::new(
        view.clone(),
        ResultPolicyDemand {
            pair_query: P1Projection::Pair(view.pair),
            mode: PolicyMode::Mut,
        },
        ty,
        source,
        Provenance::new("direct same-Type request"),
    )
    .unwrap();
    let before = format!("{world:?}");
    let failure = invoke_policy_migration(world, &request, &base.root_context()).unwrap_err();
    let OrdinaryInvocationFailure::SelectedDelete {
        diagnostic, trace, ..
    } = failure
    else {
        panic!("{failure:?}")
    };
    assert!(diagnostic.message.contains("held source"), "{diagnostic:?}");
    assert_eq!(trace.c3_call_entries.len(), 2);
    assert_eq!(trace.a_fully_admissible.len(), 1);
    assert!(trace.selected.is_some());
    assert_eq!(format!("{world:?}"), before);
    assert!(world.compile_instances().next().is_none());
}

#[test]
fn sole_admissible_migration_with_wrong_output_mode_fails_after_selection_without_publication() {
    let mut family = support::AssociatedFamily::new(&[
        "let sole=(self,x):compile -> let r:Requested:: => { x; };",
        "mut let other=(mut let self,x):compile -> mut let r:uint8:: => { x; };",
    ]);
    let base = CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
    let pattern = family.target_binding().type_resident_pattern().unwrap();
    let world = family.semantic_world_mut();
    let ty = world.type_for_pattern(pattern).unwrap();
    let view = declared_policy_view(Stage::Compile, PolicyMode::Const);
    let source = world
        .install_plain_value(ty, view.pair.clone(), Provenance::new("actual source"))
        .unwrap();
    let held = world.complete_type_for_value(source).unwrap().clone();
    world
        .bind_ordinary_new(
            base.core_node(),
            "Requested",
            &[PolicyResultEntry {
                value: None,
                pattern,
                view: view.clone(),
            }],
            Some(&held),
            Provenance::new("same-Type result classifier"),
        )
        .unwrap();
    let request = PolicyMigrationRequest::new(
        view.clone(),
        ResultPolicyDemand {
            pair_query: P1Projection::Pair(view.pair),
            mode: PolicyMode::Mut,
        },
        ty,
        source,
        Provenance::new("mut target with sole const producer"),
    )
    .unwrap();
    let before = format!("{world:?}");
    let failure = invoke_policy_migration(world, &request, &base.root_context()).unwrap_err();
    let OrdinaryInvocationFailure::SelectedImplementation { diagnostic, trace } = failure else {
        panic!("{failure:?}")
    };
    assert!(
        diagnostic
            .message
            .contains("declaration Mode does not satisfy target Mode"),
        "{diagnostic:?}"
    );
    assert_eq!(trace.c3_call_entries.len(), 2);
    assert_eq!(trace.a_fully_admissible.len(), 1);
    assert_eq!(trace.bp_prime.len(), 1);
    assert_eq!(trace.b3_pattern_specific.len(), 1);
    assert_eq!(trace.selected, Some(trace.a_fully_admissible[0]));
    assert!(trace.compile_instance.is_none());
    assert_eq!(format!("{world:?}"), before);
    assert!(world.compile_instances().next().is_none());
}
