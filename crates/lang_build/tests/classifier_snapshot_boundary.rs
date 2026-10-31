mod support;

use lang_build::*;

#[test]
fn shared_pattern_root_does_not_merge_frozen_classifier_cores_in_selection() {
    let mut family = support::AssociatedFamily::new(&[
        "let old_candidate=(self,x:Old::):compile -> let r:uint8:: => (\"old\") delete;",
        "let new_candidate=(self,x:New::):compile -> let r:uint8:: => (\"new\") delete;",
    ]);
    let base = CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
    let core = base.core_node();
    let root = base.package_root_node();
    let world = family.semantic_world_mut();
    let rank = world.type_rank().unwrap();
    let ty = support::type_lookup_fixture("frozen-classifier");
    let policy = declared_policy_view(Stage::Compile, PolicyMode::Const);
    let (_, _, pattern) = world
        .register_type_symbol(
            core,
            "RootClassifier",
            SymbolId(880001),
            ty,
            rank,
            None,
            policy.pair.clone(),
            Provenance::new("classifier formation"),
        )
        .unwrap();
    let old = world
        .install_plain_value(ty, policy.pair.clone(), Provenance::new("old object"))
        .unwrap();
    let old_type = world.complete_type_for_value(old).unwrap().clone();
    assert_eq!(
        old_type.lookup_key(),
        ty,
        "formation captures intended classifier"
    );
    world
        .bind_ordinary_new(
            core,
            "Old",
            &[PolicyResultEntry {
                value: None,
                pattern,
                view: policy.clone(),
            }],
            Some(&old_type),
            Provenance::new("held old classifier"),
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
            policy.pair.clone(),
            Provenance::new("actual added Val2"),
        )
        .unwrap();
    world
        .associate_existing_value(pattern, "extra", extra)
        .unwrap();
    let new = world
        .install_plain_value(ty, policy.pair.clone(), Provenance::new("successor object"))
        .unwrap();
    let new_type = world.complete_type_for_value(new).unwrap().clone();
    assert_ne!(old_type.core(), new_type.core());
    assert_eq!(old_type.lookup_key(), new_type.lookup_key());
    world
        .register_type_symbol_with_complete_type(
            core,
            "New",
            SymbolId(880003),
            ty,
            Some(new_type.whole()),
            rank,
            None,
            policy.pair.clone(),
            Provenance::new("held successor classifier"),
        )
        .unwrap();
    for (name, value) in [("old", old), ("new", new)] {
        world
            .bind_ordinary_new(
                base.root_context().current_namespace,
                name,
                &[PolicyResultEntry {
                    value: Some(SemanticValueRef {
                        id: value,
                        type_value: ty,
                    }),
                    pattern,
                    view: policy.clone(),
                }],
                None,
                Provenance::new("actual value binding"),
            )
            .unwrap();
    }
    for name in ["old", "new"] {
        let call = extract_single_call_site(&support::initializer_from_source(&format!(
            "let r={name} choose;"
        )))
        .unwrap();
        let before = format!("{:?}", family.semantic_world());
        let failure = family
            .invoke_ordinary_call(
                root,
                &call,
                OrdinaryInvocationContext::open_static(&[]),
                Provenance::new("frozen classifier first round"),
            )
            .unwrap_err();
        let OrdinaryInvocationFailure::SelectedDelete {
            diagnostic, trace, ..
        } = failure
        else {
            panic!("{failure:?}")
        };
        assert!(diagnostic.message.contains(name), "{diagnostic:?}");
        assert_eq!(trace.c3_call_entries.len(), 2);
        assert_eq!(trace.a_fully_admissible.len(), 1);
        assert!(trace.selected.is_some());
        assert_eq!(format!("{:?}", family.semantic_world()), before);
    }
}

#[test]
fn saved_structural_value_is_one_argument_and_runtime_view_exposes_no_shape() {
    for stage in [Stage::Compile, Stage::Runtime] {
        let mut family = support::AssociatedFamily::new(&[
            "let one=(self,_):compile -> let r:uint8:: => (\"one saved object\") delete;",
            "let two=(self,_,_):compile -> let r:uint8:: => (\"two entries\") delete;",
        ]);
        let base = CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
        let namespace = base.root_context().current_namespace;
        let world = family.semantic_world_mut();
        let rank = world.type_rank().unwrap();
        let ty = support::type_lookup_fixture("saved-layer");
        let policy = declared_policy_view(Stage::Compile, PolicyMode::Const);
        let (_, _, pattern) = world
            .register_type_symbol(
                namespace,
                "SavedLayer",
                SymbolId(880020),
                ty,
                rank,
                None,
                policy.pair.clone(),
                Provenance::new("saved nonstandard layer"),
            )
            .unwrap();
        let value = world
            .install_plain_value(ty, policy.pair, Provenance::new("saved complete object"))
            .unwrap();
        world
            .bind_ordinary_new(
                namespace,
                "saved",
                &[PolicyResultEntry {
                    value: Some(SemanticValueRef {
                        id: value,
                        type_value: ty,
                    }),
                    pattern,
                    view: declared_policy_view(stage, PolicyMode::Const),
                }],
                None,
                Provenance::new("observation edge"),
            )
            .unwrap();
        let call =
            extract_single_call_site(&support::initializer_from_source("let r=saved choose;"))
                .unwrap();
        assert_eq!(call.to_arg_product_shape().arity, 1);
        let before = format!("{:?}", family.semantic_world());
        let failure = family
            .invoke_ordinary_call(
                namespace,
                &call,
                OrdinaryInvocationContext::open_static(&[]),
                Provenance::new("saved layer boundary"),
            )
            .unwrap_err();
        match (stage, failure) {
            (
                Stage::Compile,
                OrdinaryInvocationFailure::SelectedDelete {
                    diagnostic, trace, ..
                },
            ) => {
                assert!(diagnostic.message.contains("one saved object"));
                assert_eq!(trace.a_fully_admissible.len(), 1);
            }
            (Stage::Runtime, OrdinaryInvocationFailure::ObservationUnavailable { trace, .. }) => {
                assert!(trace.selected.is_none());
                assert!(trace.a_fully_admissible.is_empty());
            }
            (_, other) => panic!("{other:?}"),
        }
        assert_eq!(format!("{:?}", family.semantic_world()), before);
    }
}
