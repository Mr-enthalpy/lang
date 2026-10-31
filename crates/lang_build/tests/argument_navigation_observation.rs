mod support;
use lang_build::*;

fn choices() -> support::AssociatedFamily {
    support::AssociatedFamily::new(&[
        "let c=(self,const let x):compile -> let r:uint8:: => (\"const chosen\") delete;",
        "let m=(self,mut let x):compile -> let r:uint8:: => (\"mut chosen\") delete;",
    ])
}

fn bind_value(
    world: &mut SemanticWorld,
    core: NamespaceNodeId,
    ns: NamespaceNodeId,
    name: &str,
    views: &[PolicyView],
) {
    let pattern = world
        .symbol_in_namespace(core, "uint8")
        .unwrap()
        .type_resident_pattern()
        .unwrap();
    let ty = world.type_for_pattern(pattern).unwrap();
    let id = world
        .install_plain_value(
            ty,
            declared_policy_view(Stage::Compile, PolicyMode::Const).pair,
            Provenance::new("actual resident"),
        )
        .unwrap();
    let entries = views
        .iter()
        .map(|view| PolicyResultEntry {
            value: Some(SemanticValueRef { id, type_value: ty }),
            pattern,
            view: view.clone(),
        })
        .collect::<Vec<_>>();
    world
        .bind_ordinary_new(
            ns,
            name,
            &entries,
            None,
            Provenance::new("actual observation edges"),
        )
        .unwrap();
}

#[test]
fn argument_uses_complete_lexical_chain_and_hidden_outer_never_falls_back_to_mount() {
    for stage in [Stage::Compile, Stage::Runtime] {
        let mut family = choices();
        let target = family.target_binding().identity;
        let base = CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
        let root = base.package_root_node();
        let core = base.core_node();
        let outer = NamespaceNodeId(890001);
        let inner = NamespaceNodeId(890002);
        let world = family.semantic_world_mut();
        world.register_namespace(outer, root, "outer").unwrap();
        world.register_namespace(inner, outer, "inner").unwrap();
        bind_value(
            world,
            core,
            outer,
            "arg",
            &[declared_policy_view(stage, PolicyMode::Mut)],
        );
        bind_value(
            world,
            core,
            core,
            "arg",
            &[declared_policy_view(Stage::Compile, PolicyMode::Const)],
        );
        let context = ResolverContext::with_mounts(inner, vec![root], vec![core]);
        let navigation = world
            .navigate_semantic_path(&["arg".into()], inner, &[root], &[core])
            .unwrap();
        assert_eq!(
            navigation.terminal_symbol,
            world.symbol_in_namespace(outer, "arg").unwrap().identity
        );
        let call = extract_single_call_site(&support::initializer_from_source("let r=arg choose;"))
            .unwrap();
        let before = format!("{world:?}");
        let failure = invoke_resolved_binding_ordinary(
            world,
            &[],
            target,
            &call,
            &context,
            OrdinaryInvocationContext::open_static(&[]),
            Provenance::new("shared argument lookup"),
        )
        .unwrap_err();
        match (stage, failure) {
            (
                Stage::Compile,
                OrdinaryInvocationFailure::SelectedDelete {
                    diagnostic, trace, ..
                },
            ) => {
                assert!(diagnostic.message.contains("mut chosen"), "{diagnostic:?}");
                assert_eq!(trace.a_fully_admissible.len(), 2);
            }
            (Stage::Runtime, OrdinaryInvocationFailure::ObservationUnavailable { trace, .. }) => {
                assert!(trace.selected.is_none());
                assert!(trace.a_fully_admissible.is_empty());
            }
            (_, other) => panic!("{other:?}"),
        }
        assert_eq!(format!("{world:?}"), before);
    }
}

#[test]
fn argument_policy_edges_do_not_collapse_by_resident_id_or_registration_order() {
    for type_argument in [false, true] {
        for reverse in [false, true] {
            for hint in [PolicyMode::Const, PolicyMode::Mut] {
                let mut family = choices();
                let target = family.target_binding().identity;
                let base = CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
                let ns = base.root_context().current_namespace;
                let core = base.core_node();
                let mut views = vec![
                    declared_policy_view(Stage::Compile, PolicyMode::Const),
                    declared_policy_view(Stage::Compile, PolicyMode::Mut),
                ];
                if reverse {
                    views.reverse()
                }
                let world = family.semantic_world_mut();
                if type_argument {
                    let member = world
                        .symbol_in_namespace(core, "uint8")
                        .unwrap()
                        .type_resident()
                        .unwrap();
                    let complete = world
                        .complete_type_by_whole_observation(member.complete_type.unwrap())
                        .unwrap()
                        .clone();
                    let entries = views
                        .iter()
                        .map(|view| PolicyResultEntry {
                            value: None,
                            pattern: member.pattern,
                            view: view.clone(),
                        })
                        .collect::<Vec<_>>();
                    world
                        .bind_ordinary_new(
                            ns,
                            "arg",
                            &entries,
                            Some(&complete),
                            Provenance::new("type observation edges"),
                        )
                        .unwrap();
                } else {
                    bind_value(world, core, ns, "arg", &views)
                }
                assert_eq!(
                    world
                        .symbol_in_namespace(ns, "arg")
                        .unwrap()
                        .member_views
                        .len(),
                    2
                );
                let call = extract_single_call_site(&support::initializer_from_source(
                    "let r=arg choose;",
                ))
                .unwrap();
                let before = format!("{world:?}");
                let failure = invoke_resolved_binding_ordinary(
                    world,
                    &[],
                    target,
                    &call,
                    &base.root_context(),
                    OrdinaryInvocationContext::open_static(&[hint]),
                    Provenance::new("multiview argument"),
                )
                .unwrap_err();
                let OrdinaryInvocationFailure::ObservationUnavailable { trace, .. } = failure
                else {
                    panic!("{failure:?}")
                };
                assert!(trace.selected.is_none());
                assert!(trace.a_fully_admissible.is_empty());
                assert!(trace.compile_instance.is_none());
                assert_eq!(format!("{world:?}"), before);
            }
        }
    }
}

#[test]
fn identical_argument_edges_have_one_observation_without_inventing_mode() {
    let mut family = choices();
    let target = family.target_binding().identity;
    let base = CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
    let view = declared_policy_view(Stage::Compile, PolicyMode::Mut);
    let world = family.semantic_world_mut();
    bind_value(
        world,
        base.core_node(),
        base.root_context().current_namespace,
        "arg",
        &[view.clone(), view],
    );
    let call =
        extract_single_call_site(&support::initializer_from_source("let r=arg choose;")).unwrap();
    let failure = invoke_resolved_binding_ordinary(
        world,
        &[],
        target,
        &call,
        &base.root_context(),
        OrdinaryInvocationContext::open_static(&[]),
        Provenance::new("identical edge control"),
    )
    .unwrap_err();
    assert!(
        matches!(failure,OrdinaryInvocationFailure::SelectedDelete {diagnostic,..} if diagnostic.message.contains("mut chosen"))
    );
}

#[test]
fn classifier_only_annotation_does_not_require_a_unique_argument_mode() {
    for reverse in [false, true] {
        let mut family=support::AssociatedFamily::new(&[
            "let typed=(self,x:Classifier::):compile -> let r:uint8:: => (\"classifier matched\") delete;",
            "let generic=(self,_):compile -> let r:uint8:: => (\"generic\") delete;",
        ]);
        let target = family.target_binding().identity;
        let base = CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
        let core = base.core_node();
        let world = family.semantic_world_mut();
        let resident = world
            .symbol_in_namespace(core, "uint8")
            .unwrap()
            .type_resident()
            .unwrap();
        let complete = world
            .complete_type_by_whole_observation(resident.complete_type.unwrap())
            .unwrap()
            .clone();
        let mut edges = vec![
            PolicyResultEntry {
                value: None,
                pattern: resident.pattern,
                view: declared_policy_view(Stage::Compile, PolicyMode::Const),
            },
            PolicyResultEntry {
                value: None,
                pattern: resident.pattern,
                view: declared_policy_view(Stage::Compile, PolicyMode::Mut),
            },
        ];
        if reverse {
            edges.reverse()
        }
        world
            .bind_ordinary_new(
                core,
                "Classifier",
                &edges,
                Some(&complete),
                Provenance::new("classifier-only multi-edge observation"),
            )
            .unwrap();
        let resolution = SemanticTypeEnv::new(world)
            .resolve_type_name("Classifier", &base.root_context())
            .unwrap();
        assert!(resolution
            .observe_classifier_at(ObservationHorizon::OpenStatic)
            .is_ok());
        assert!(resolution
            .observe_argument_at(ObservationHorizon::OpenStatic)
            .is_err());
        bind_value(
            world,
            core,
            base.root_context().current_namespace,
            "arg",
            &[declared_policy_view(Stage::Compile, PolicyMode::Const)],
        );
        let call = extract_single_call_site(&support::initializer_from_source("let r=arg choose;"))
            .unwrap();
        let before = format!("{world:?}");
        let failure = invoke_resolved_binding_ordinary(
            world,
            &[],
            target,
            &call,
            &base.root_context(),
            OrdinaryInvocationContext::open_static(&[]),
            Provenance::new("Pattern/Core facet only"),
        )
        .unwrap_err();
        let OrdinaryInvocationFailure::SelectedDelete {
            diagnostic, trace, ..
        } = failure
        else {
            panic!("{failure:?}")
        };
        assert!(
            diagnostic.message.contains("classifier matched"),
            "{diagnostic:?}"
        );
        assert_eq!(trace.a_fully_admissible.len(), 2);
        assert!(trace.selected.is_some());
        assert_eq!(format!("{world:?}"), before);
    }
}

#[test]
fn type_argument_retains_every_host_factor_with_visible_terminal_control() {
    for host_stages in [
        vec![Stage::Runtime],
        vec![Stage::Runtime, Stage::Compile],
        vec![Stage::Compile, Stage::Runtime],
    ] {
        let base = CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
        let mut world = base.semantic_world().clone();
        let ns = base.root_context().current_namespace;
        let core = base.core_node();
        let rank = world.type_rank().unwrap();
        let terminal = world.symbol_in_namespace(core, "uint8").unwrap().identity;
        let target = world
            .symbol_in_namespace(core, "IdentityType")
            .unwrap()
            .identity;
        let seed = world
            .register_type_symbol(
                ns,
                "SeedHost",
                SymbolId(890010),
                support::type_lookup_fixture("navigation host"),
                rank,
                None,
                declared_policy_view(Stage::Compile, PolicyMode::Const).pair,
                Provenance::new("host formation"),
            )
            .unwrap()
            .0;
        let resident = world.symbol(seed).unwrap().type_resident().unwrap();
        let complete = world
            .complete_type_by_whole_observation(resident.complete_type.unwrap())
            .unwrap()
            .clone();
        let edges = host_stages
            .iter()
            .map(|stage| PolicyResultEntry {
                value: None,
                pattern: resident.pattern,
                view: PolicyView {
                    pair: PolicyPair {
                        value: ValueComponentPolicy::Absent,
                        pattern: PatternComponentPolicy {
                            stage: Some(*stage),
                        },
                    },
                    mode: PolicyMode::Const,
                },
            })
            .collect::<Vec<_>>();
        let host = world
            .bind_ordinary_new(
                ns,
                "Host",
                &edges,
                Some(&complete),
                Provenance::new("host edges"),
            )
            .unwrap();
        let place = world.symbol(host).unwrap().type_resident_place().unwrap();
        world
            .associate_existing_symbol_in_place(place, "leaf", terminal)
            .unwrap();
        let path = vec!["leaf".into(), "Host".into()];
        let resolved = SemanticTypeEnv::new(&world)
            .resolve_type_path(&path, &base.root_context())
            .unwrap();
        assert_eq!(resolved.navigation.terminal_symbol, terminal);
        assert_eq!(resolved.navigation.host_chain.len(), 1);
        assert_eq!(
            resolved.navigation.host_chain[0].views.len(),
            host_stages.len()
        );
        let control = extract_single_call_site(&support::initializer_from_source(
            "let r=uint8 IdentityType;",
        ))
        .unwrap();
        assert!(invoke_resolved_binding_ordinary(
            &mut world,
            &[],
            target,
            &control,
            &base.root_context(),
            OrdinaryInvocationContext::open_static(&[]),
            Provenance::new("visible terminal control")
        )
        .is_ok());
        let call = extract_single_call_site(&support::initializer_from_source(
            "let r=(leaf::Host) IdentityType;",
        ))
        .unwrap();
        let type_report = classify_type_arguments_env_with_report(
            &call.to_arg_product_shape(),
            &SemanticTypeEnv::new(&world),
            &base.root_context(),
            ObservationHorizon::OpenStatic,
        );
        assert_eq!(type_report.is_ok(), host_stages.contains(&Stage::Compile));
        let before = format!("{world:?}");
        let outcome = invoke_resolved_binding_ordinary(
            &mut world,
            &[],
            target,
            &call,
            &base.root_context(),
            OrdinaryInvocationContext::open_static(&[]),
            Provenance::new("argument host conjunction"),
        );
        if host_stages.contains(&Stage::Compile) {
            assert!(outcome.is_ok(), "{outcome:?}");
        } else {
            let OrdinaryInvocationFailure::ObservationUnavailable { obstruction, trace } =
                outcome.unwrap_err()
            else {
                panic!("hidden host must block before selection")
            };
            assert_eq!(obstruction.class, "hidden-navigation-host-observation");
            assert!(trace.selected.is_none());
            assert!(trace.a_fully_admissible.is_empty());
            assert_eq!(format!("{world:?}"), before);
        }
    }
}
