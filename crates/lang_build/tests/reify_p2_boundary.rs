mod support;

use lang_build::{
    canonical_function_object_view, declared_policy_view, direct_callable_body_policy,
    elaborate_binding_result_demand, elaborate_callable_p2, elaborate_formal_policy_pattern,
    elaborate_return_policy_pattern, CallableP2Policy, CompilationWorld,
    NamespaceDeclarationPosition, ObservationHorizon, P1Projection, PatternComponentPolicy,
    PolicyMigrationRequest, PolicyMigrationRequestFailure, PolicyMode, PolicyPair, PolicyView,
    Provenance, ResultPolicyDemand, SemanticValueId, Stage, ValueComponentPolicy,
};
use lang_syntax::{NormDecl, NormForm, NormPolicySpec};

fn policy(text: &str) -> NormPolicySpec {
    let parsed = lang_syntax::parse(&format!("{text} let x = value;"));
    assert!(parsed.diagnostics.is_empty());
    let norm = lang_syntax::normalize_program(&parsed.program);
    let NormForm::Let(NormDecl::Let { slot, .. }) = &norm.forms[0] else {
        panic!("let")
    };
    slot.policy.clone().unwrap()
}

#[test]
fn reify_is_only_callable_p2_with_outward_runtime_policy() {
    let p = Provenance::new("P2 qualification");
    let ordinary = elaborate_callable_p2(&policy("runtime + mut"), p.clone()).unwrap();
    let reify = elaborate_callable_p2(&policy("reify + mut"), p.clone()).unwrap();
    assert!(matches!(ordinary, CallableP2Policy::Direct(_)));
    assert!(matches!(reify, CallableP2Policy::Reify { .. }));
    assert_eq!(ordinary.outward_view(), reify.outward_view());
    assert_eq!(
        reify.outward_view().pair.value.stage(),
        Some(Stage::Runtime)
    );
    assert!(reify
        .require_connected_body(p.clone())
        .unwrap_err()
        .message
        .contains("closed-return"));
    for text in [
        "reify + compile",
        "reify + runtime",
        "reify + reify",
        "open + reify",
    ] {
        assert!(elaborate_callable_p2(&policy(text), p.clone()).is_err());
    }
    assert!(elaborate_binding_result_demand(Some(&policy("reify")), p.clone()).is_err());
    let runtime = declared_policy_view(Stage::Runtime, PolicyMode::Const);
    assert!(elaborate_formal_policy_pattern(Some(&policy("reify")), &runtime, p.clone()).is_err());
    assert!(elaborate_return_policy_pattern(Some(&policy("reify")), &runtime, p.clone()).is_err());
    assert!(lang_build::elaborate_namespace_declaration_policy(
        Some(&policy("reify")),
        NamespaceDeclarationPosition::DirectTopLevel,
        p
    )
    .is_err());
}

#[test]
fn runtime_p2_preserves_an_explicit_compile_producer() {
    let p = Provenance::new("runtime closure");
    let runtime = declared_policy_view(Stage::Runtime, PolicyMode::Const);
    let compile = declared_policy_view(Stage::Compile, PolicyMode::Const);
    let explicit = lang_build::ExplicitP1Selection::from_complete_view(&compile);
    assert_eq!(
        canonical_function_object_view(Some(&explicit), &runtime, &runtime, None, &p).unwrap(),
        compile
    );
    assert_eq!(
        canonical_function_object_view(None, &runtime, &runtime, None, &p).unwrap(),
        runtime
    );
    let base = CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
    let mut world = base.semantic_world().clone();
    let registered = world
        .register_core_callable(
            base.package_root_node(),
            "runtime_with_compile_result",
            lang_build::SymbolId(900010),
            lang_build::BuiltinCallableImpl::IdentityType,
            Some(explicit),
            lang_build::DeclaredResultClass::CompleteType,
            compile.clone(),
            runtime.clone(),
            compile,
            None,
            p,
        )
        .unwrap();
    let lang_build::SemanticValuePayload::CallEntry(entry) =
        &world.value(registered.call_entry).unwrap().payload
    else {
        panic!("call entry");
    };
    assert_eq!(
        entry.callable_view.pair.leading_stage(),
        Some(Stage::Compile)
    );
    assert_eq!(
        entry.body_entry_view.pair.leading_stage(),
        Some(Stage::Runtime)
    );
    assert_eq!(
        entry.complete_result_view.pair.leading_stage(),
        Some(Stage::Compile)
    );
}

#[test]
fn reverse_endpoints_cannot_forge_an_active_reify_boundary() {
    let cc = declared_policy_view(Stage::Compile, PolicyMode::Const);
    let rc = declared_policy_view(Stage::Runtime, PolicyMode::Const);
    let rr = PolicyView {
        pair: PolicyPair {
            value: ValueComponentPolicy::Present(Stage::Runtime),
            pattern: PatternComponentPolicy {
                stage: Some(Stage::Runtime),
            },
        },
        mode: PolicyMode::Const,
    };
    for (source, target) in [
        (rc.clone(), cc.clone()),
        (rr.clone(), rc),
        (rr.clone(), cc.clone()),
    ] {
        let error = PolicyMigrationRequest::new(
            source,
            ResultPolicyDemand {
                pair_query: P1Projection::Pair(target.pair),
                mode: target.mode,
            },
            support::type_lookup_fixture("reify/type"),
            SemanticValueId(1),
            Provenance::new("no active continuation witness"),
        )
        .unwrap_err();
        assert_eq!(
            error,
            PolicyMigrationRequestFailure::ReifyBoundaryUnavailable
        );
    }
    assert!(PolicyMigrationRequest::new(
        cc,
        ResultPolicyDemand {
            pair_query: P1Projection::Pair(rr.pair),
            mode: rr.mode
        },
        support::type_lookup_fixture("reify/type"),
        SemanticValueId(1),
        Provenance::new("forward endpoints")
    )
    .is_ok());
}

#[test]
fn runtime_policy_type_observation_does_not_supply_compile_type_operations() {
    let entry = lang_build::PolicyResultEntry::<(), _> {
        value: None,
        pattern: 7,
        view: PolicyView {
            pair: PolicyPair {
                value: ValueComponentPolicy::Present(Stage::Runtime),
                pattern: PatternComponentPolicy {
                    stage: Some(Stage::Runtime),
                },
            },
            mode: PolicyMode::Const,
        },
    };
    assert_eq!(
        lang_build::read_pattern(&lang_build::expose_policy_slice(
            &entry,
            ObservationHorizon::Runtime
        )),
        Some(&7)
    );
    assert_eq!(
        lang_build::read_pattern(&lang_build::expose_policy_slice(
            &entry,
            ObservationHorizon::OpenStatic
        )),
        None
    );
}

#[test]
fn runtime_type_residency_cannot_execute_a_compile_type_builtin() {
    let base = CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
    let mut world = base.semantic_world().clone();
    let source = world
        .symbol_in_namespace(base.core_node(), "uint8")
        .unwrap()
        .type_resident()
        .unwrap();
    let complete = world
        .complete_type_by_whole_observation(source.complete_type.unwrap())
        .unwrap()
        .clone();
    let runtime = PolicyView {
        pair: PolicyPair {
            value: ValueComponentPolicy::Present(Stage::Runtime),
            pattern: PatternComponentPolicy {
                stage: Some(Stage::Runtime),
            },
        },
        mode: PolicyMode::Const,
    };
    world
        .bind_ordinary_new(
            base.package_root_node(),
            "RuntimeType",
            &[lang_build::PolicyResultEntry {
                value: None,
                pattern: source.pattern,
                view: runtime,
            }],
            Some(&complete),
            Provenance::new("runtime type residency"),
        )
        .unwrap();
    let binding = world
        .symbol_in_namespace(base.core_node(), "IdentityType")
        .unwrap()
        .identity;
    let site = lang_build::extract_single_call_site(&support::initializer_from_source(
        "let result=RuntimeType IdentityType::core;",
    ))
    .unwrap();
    let before = format!("{world:?}");
    let error = lang_build::invoke_resolved_binding_ordinary(
        &mut world,
        &[],
        binding,
        &site,
        &base.package_context(),
        lang_build::OrdinaryInvocationContext::open_static(&[]),
        Provenance::new("runtime type consumer"),
    )
    .unwrap_err();
    let lang_build::OrdinaryInvocationFailure::ApplicabilityUnsupported { diagnostic, trace } =
        error
    else {
        panic!("{error:?}")
    };
    assert!(diagnostic.message.contains("reify reverse-migration"));
    assert!(trace.selected.is_none());
    assert!(trace.compile_instance.is_none());
    assert_eq!(format!("{world:?}"), before);
    let wrong_arity = lang_build::extract_single_call_site(&support::initializer_from_source(
        "let result=(RuntimeType,uint8) IdentityType::core;",
    ))
    .unwrap();
    let failure = lang_build::invoke_resolved_binding_ordinary(
        &mut world,
        &[],
        binding,
        &wrong_arity,
        &base.package_context(),
        lang_build::OrdinaryInvocationContext::open_static(&[]),
        Provenance::new("decided arity"),
    )
    .unwrap_err();
    assert!(
        matches!(
            failure,
            lang_build::OrdinaryInvocationFailure::NoFullyAdmissibleCandidate {
                first_diagnostic: Some(_),
                ..
            }
        ),
        "{failure:?}"
    );
    assert_eq!(format!("{world:?}"), before);
}

#[test]
fn reify_execution_and_its_bootstrap_name_are_unavailable() {
    let world = CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
    assert!(world
        .semantic_world()
        .symbol_in_namespace(world.core_node(), "reify")
        .is_none());
    let err =
        direct_callable_body_policy(&policy("reify"), Provenance::new("unconnected boundary"))
            .unwrap_err();
    assert!(err.message.contains("consumer is unavailable"));
    let error =
        support::AssociatedFamily::try_new(&["let f=(self,x):reify -> _:bool:: => { x; };"])
            .err()
            .expect("no boundary consumer");
    assert!(format!("{error:?}").contains("closed-return"));
}
