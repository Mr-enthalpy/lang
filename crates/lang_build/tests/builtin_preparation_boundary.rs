mod support;

use lang_build::{
    declared_policy_view, extract_single_call_site, invoke_resolved_binding_ordinary,
    BuiltinCallableImpl, CompilationWorld, DeclaredResultClass, ObservationHorizon,
    OrdinaryCandidateRole, OrdinaryInvocationContext, OrdinaryInvocationFailure, PolicyMode,
    Provenance, SemanticValuePayload, Stage, SymbolId,
};

#[test]
fn unknown_builtin_applicability_cannot_remove_a_candidate_and_select_a_runner_up() {
    for (target_path, input, expected) in [
        (
            "assert::core",
            "uint8",
            "applicability relation consumer is not connected",
        ),
        (
            "exists::verify::core",
            "uint8",
            "applicability relation consumer is not connected",
        ),
        (
            "struct::core",
            "(uint8 field)",
            "parameter shape compatibility is not established",
        ),
        (
            "IdentityType::core",
            "(uint8 field)",
            "parameter shape compatibility is not established",
        ),
    ] {
        let base = CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
        let declaration = base
            .namespace_projection()
            .capability()
            .resolve_callable(target_path, &base.package_context())
            .unwrap();
        let namespace = declaration.parent.unwrap();
        let mut world = base.semantic_world().clone();
        let binding = world
            .symbol_in_namespace(namespace, &declaration.name)
            .unwrap();
        let target = binding.identity;
        let receiver = binding.ordinary_value().unwrap();
        let pattern = world.value(receiver).unwrap().pattern;
        let lang_syntax::NormExpr::Closure(fallback) = support::initializer_from_source(
            "let fallback = (self, t:type):compile -> let r:type => (\"must not run\") delete;",
        ) else {
            panic!("fixture closure");
        };
        let view = declared_policy_view(Stage::Compile, PolicyMode::Const);
        world
            .register_associated_call_entry(
                pattern,
                namespace,
                SymbolId(900_001),
                &fallback,
                None,
                view.clone(),
                view,
                None,
                OrdinaryCandidateRole::Ordinary,
                DeclaredResultClass::CompleteType,
                Provenance::new("known candidate relation"),
            )
            .unwrap();
        assert_eq!(world.callable_entries_for_value(receiver).len(), 2);
        let before = format!("{world:?}");
        let site = extract_single_call_site(&support::initializer_from_source(&format!(
            "let result = {input} {target_path};"
        )))
        .unwrap();
        let error = invoke_resolved_binding_ordinary(
            &mut world,
            &[],
            target,
            &site,
            &base.package_context(),
            OrdinaryInvocationContext::open_static(&[]),
            Provenance::new("incomplete family"),
        )
        .unwrap_err();
        let OrdinaryInvocationFailure::ApplicabilityUnsupported { diagnostic, trace } = error
        else {
            panic!("unknown applicability must stop the family: {error:?}");
        };
        assert!(diagnostic.message.contains(expected));
        assert_eq!(trace.c3_call_entries.len(), 2);
        assert!(trace.bp_prime.is_empty());
        assert!(trace.selected.is_none());
        assert!(trace.dynamic_legality.is_none());
        assert_eq!(format!("{world:?}"), before);
    }
}

#[test]
fn builtin_selected_result_preserves_call_entry_declared_policy() {
    let base = CompilationWorld::from_manifest(&support::empty_app_manifest()).unwrap();
    let mut world = base.semantic_world().clone();
    let callable = declared_policy_view(Stage::Seal, PolicyMode::Const);
    let body = declared_policy_view(Stage::Seal, PolicyMode::Mut);
    let result = declared_policy_view(Stage::Seal, PolicyMode::Const);
    let installed = world
        .register_core_callable(
            base.package_root_node(),
            "custom_identity",
            SymbolId(900_002),
            BuiltinCallableImpl::IdentityType,
            None,
            DeclaredResultClass::CompleteType,
            callable.clone(),
            body.clone(),
            result.clone(),
            None,
            Provenance::new("declared planes"),
        )
        .unwrap();
    let site = extract_single_call_site(&support::initializer_from_source(
        "let result = uint8 custom_identity;",
    ))
    .unwrap();
    let mut context = OrdinaryInvocationContext::open_static(&[]);
    context.horizon = ObservationHorizon::SealStatic;
    let outcome = invoke_resolved_binding_ordinary(
        &mut world,
        &[],
        installed.symbol,
        &site,
        &base.package_context(),
        context.clone(),
        Provenance::new("declared Policy invocation"),
    )
    .unwrap();
    let lang_build::InvocationResult::SemanticResult {
        value: lang_build::ProjectedInvocationOutcome::SingleMember(outcome),
        ..
    } = outcome
    else {
        panic!("complete ordinary result");
    };
    assert_eq!(outcome.selected.function_object_view, callable);
    assert_eq!(outcome.selected.body_entry_view, body);
    assert_eq!(outcome.selected.complete_result_view, result);
    assert_eq!(outcome.complete_result[0].view, result);
    assert_eq!(
        outcome.selected.frame.horizon,
        ObservationHorizon::SealStatic
    );
    let SemanticValuePayload::CallEntry(entry) =
        &world.value(installed.call_entry).unwrap().payload
    else {
        panic!("terminal call entry");
    };
    assert_eq!(entry.complete_result_view, result);
    let instance = outcome
        .trace
        .compile_instance
        .expect("seal body has a compile instance");
    let state = world.compile_instance(instance).unwrap();
    assert_eq!(
        state.root.parent_owner,
        world.namespace_owner(base.package_root_node()).unwrap()
    );
    assert!(!state.evaluation_active());
    let delivered = state
        .delivered_result()
        .expect("ordinary result was delivered");
    assert_eq!(
        delivered.complete_type,
        outcome.complete_type.as_ref().map(|ty| ty.whole())
    );
    let repeated = invoke_resolved_binding_ordinary(
        &mut world,
        &[],
        installed.symbol,
        &site,
        &base.package_context(),
        context,
        Provenance::new("same inputs under seal"),
    )
    .unwrap();
    let lang_build::InvocationResult::SemanticResult {
        value: lang_build::ProjectedInvocationOutcome::SingleMember(repeated),
        ..
    } = repeated
    else {
        panic!("repeated ordinary result");
    };
    assert_eq!(repeated.trace.compile_instance, Some(instance));
}
