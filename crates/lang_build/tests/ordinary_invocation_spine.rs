mod support;

use lang_build::{
    extract_single_call_site, BuildManifest, CapabilityRealization, CapabilityRealizationCell,
    CompilationWorld, LifecyclePrecondition, LifecycleValidationContext, OrdinaryInvocationContext,
    PolicyMode, Provenance, SemanticValuePayload, WritableContext,
};

use support::{build_single_fixture_world, initializer_from_source};

#[test]
fn type_projection_is_not_an_ordinary_resident() {
    let world = build_single_fixture_world("single_package_type_binding", "app");
    let uint8 = world
        .semantic_world()
        .symbol_in_namespace(world.core_node(), "uint8")
        .expect("core uint8");
    assert!(uint8.ordinary_value().is_none());
    assert!(uint8.pure_p_pattern().is_some());
    // CoreTypeProjection graph value is accessible through core_type_projection_value_for_symbol,
    // never through an ordinary resident.
    let type_obj = world
        .semantic_world()
        .core_type_projection_value_for_symbol(uint8.identity)
        .expect("pure type Object projection exists");
    let val = world.semantic_world().value(type_obj).unwrap();
    assert!(
        matches!(val.payload, SemanticValuePayload::CoreTypeProjection { .. }),
        "I12: CoreTypeProjection is graph projection, not semantic Val1"
    );
}

#[test]
fn forwarded_type_binding_carries_exact_tau_independently_of_core_projection() {
    let world = build_single_fixture_world("single_package_type_binding", "app");
    let binding = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "PatternResult")
        .expect("forwarded type result is bound");
    let member = binding
        .pure_p()
        .expect("T carries the returned pure type Object");
    let whole = member
        .complete_type
        .expect("the binding stores the returned exact complete tau snapshot");
    let complete = world
        .semantic_world()
        .complete_type_by_whole_observation(whole)
        .expect("the exact complete tau remains interned");
    assert_eq!(complete.whole(), whole);
    assert_eq!(
        world.semantic_world().type_for_pattern(member.pattern),
        Some(complete.lookup_key()),
        "the Core lookup projection agrees with tau without defining its whole identity"
    );
}

#[test]
fn source_body_frontiers_never_produce_builtin_material_or_reopen_selection() {
    for (tail, message, deleted) in [
        (
            "{ (uint8 field) struct; }",
            "shared continuation consumer",
            false,
        ),
        (
            "named { (uint8 field) struct; }",
            "shared continuation consumer",
            false,
        ),
        ("default", "default-implementation materialization", false),
        (
            "(\"selected frontier\") delete",
            "selected delete: selected frontier",
            true,
        ),
    ] {
        let selected_source =
            format!("let chosen = (self, _ uint8: type): compile -> let r: type => {tail};");
        let mut world = support::AssociatedFamily::new(&[
            "let fallback = (self, t: type): compile -> let r: type => (\"must not reopen\") delete;",
            &selected_source,
        ]);
        let before = format!("{:?}", world.semantic_world());
        let call =
            extract_single_call_site(&initializer_from_source("let R = uint8 pick;")).unwrap();
        let failure = world
            .invoke_ordinary_call(
                world.package_root_node(),
                &call,
                OrdinaryInvocationContext::open_static(&[PolicyMode::Const]),
                Provenance::new("source private-material boundary"),
            )
            .expect_err("source bodies have only a diagnostic frontier");
        let (diagnostic, trace) = match failure {
            lang_build::OrdinaryInvocationFailure::SelectedBody { failure, trace } if !deleted => {
                (failure.diagnostic, trace)
            }
            lang_build::OrdinaryInvocationFailure::SelectedDelete {
                diagnostic, trace, ..
            } if deleted => (diagnostic, trace),
            other => panic!("expected the uniquely selected source frontier for {tail}: {other:?}"),
        };
        assert!(diagnostic.message.contains(message), "{diagnostic:?}");
        assert_eq!(trace.a_fully_admissible.len(), 2);
        assert_eq!(trace.b3_pattern_specific.len(), 1);
        assert!(trace.selected.is_some());
        assert!(trace.dynamic_legality.is_some());
        assert!(
            trace.compile_instance.is_some(),
            "identity admission precedes selected body failure"
        );
        assert_eq!(
            format!("{:?}", world.semantic_world()),
            before,
            "source frontier must not execute its inner struct or publish a result"
        );
    }
}

#[test]
fn dynamic_legality_runs_after_unique_selection_and_never_reopens_the_family() {
    let mut world = support::AssociatedFamily::new(&[
        "let first = (self, t: type): compile -> let r: type => { t; };",
        "let second = (self, _ uint8: type): compile -> let r: type => { self; };",
    ]);
    let initializer = initializer_from_source("let R: type = uint8 pick;");
    let call_site = extract_single_call_site(&initializer).expect("normalized overloaded call");
    let actual = [PolicyMode::Const];
    let failure = world
        .invoke_ordinary_call(
            world.package_root_node(),
            &call_site,
            OrdinaryInvocationContext::open_static(&actual)
                .with_capability_demand(PolicyMode::Const, PolicyMode::Mut),
            Provenance::new("selected candidate capability proof failure is terminal"),
        )
        .expect_err("the selected entry has an absent capability cell");
    let lang_build::OrdinaryInvocationFailure::DynamicLegality {
        selected,
        diagnostic,
        trace,
    } = failure
    else {
        panic!("capability failure must occur at DynamicLegality: {failure:?}");
    };
    assert_eq!(trace.selected, Some(selected));
    assert!(
        trace.c3_call_entries.len() > 1,
        "fixture supplies a runner-up"
    );
    assert!(diagnostic.message.contains("no capability realization"));
}

#[test]
fn lifecycle_pre_failure_is_post_selection_and_never_reopens_the_family() {
    let mut world = support::AssociatedFamily::new(&[
        "let first = (self, t: type): compile -> let r: type => { t; };",
        "let second = (self, _ uint8: type): compile -> let r: type => { self; };",
    ]);
    let initializer = initializer_from_source("let R: type = uint8 pick;");
    let call_site = extract_single_call_site(&initializer).expect("normalized overloaded call");
    let actual = [PolicyMode::Const];
    let lifecycle = LifecycleValidationContext {
        preconditions: vec![LifecyclePrecondition::Reject(
            "selected invocation cannot outlive this continuation".into(),
        )],
        ..LifecycleValidationContext::default()
    };
    let failure = world
        .invoke_ordinary_call(
            world.package_root_node(),
            &call_site,
            OrdinaryInvocationContext::open_static(&actual)
                .with_lifecycle_preconditions(&lifecycle),
            Provenance::new("selected invocation lifecycle precondition failure is terminal"),
        )
        .expect_err("the selected entry must fail lifecycle Pre validation");
    let lang_build::OrdinaryInvocationFailure::DynamicLegality {
        selected,
        diagnostic,
        trace,
    } = failure
    else {
        panic!("lifecycle failure must occur at DynamicLegality: {failure:?}");
    };
    assert_eq!(trace.selected, Some(selected));
    assert!(
        trace.c3_call_entries.len() > 1,
        "fixture supplies a runner-up"
    );
    assert!(diagnostic
        .message
        .contains("lifecycle Pre validation failed"));
}

#[test]
fn configured_capability_cell_is_proof_material_not_policy_preference() {
    let mut world = support::AssociatedFamily::new(&[
        "let member = (self, _ uint8:type):compile -> let r:uint8 => { r; };",
    ]);
    let keep = world.target_binding();
    let entries = keep
        .ordinary_value()
        .iter()
        .flat_map(|value| {
            world
                .semantic_world()
                .associated_values_for_value(*value, "()")
                .unwrap_or(&[])
                .iter()
                .copied()
        })
        .collect::<Vec<_>>();
    let mut realization = CapabilityRealization::default();
    realization.set(
        PolicyMode::Const,
        PolicyMode::Mut,
        CapabilityRealizationCell::Default,
    );
    for entry in entries {
        world
            .semantic_world_mut()
            .configure_call_entry_capability_realization(entry, realization.clone())
            .expect("terminal call entry accepts candidate-local realization");
    }

    let initializer = initializer_from_source("let R: type = uint8 keep;");
    let call_site = extract_single_call_site(&initializer).expect("normalized ordinary call");
    let actual = [PolicyMode::Const];
    let failure = world
        .invoke_ordinary_call(
            world.package_root_node(),
            &call_site,
            OrdinaryInvocationContext::open_static(&actual)
                .with_capability_demand(PolicyMode::Const, PolicyMode::Mut),
            Provenance::new("positive DynamicLegality capability proof"),
        )
        .expect_err("fixture body is unsupported after DynamicLegality succeeds");
    let lang_build::OrdinaryInvocationFailure::SelectedBody { trace, .. } = failure else {
        panic!("configured capability must pass legality before the body failure: {failure:?}");
    };
    assert_eq!(
        trace
            .dynamic_legality
            .expect("successful post-selection validation leaves proof material")
            .capability_cell,
        Some(CapabilityRealizationCell::Default)
    );
}

#[test]
fn mut_policy_mode_does_not_grant_writable() {
    let mut world = support::AssociatedFamily::new(&[
        "mut let first = (self, t: type): compile -> let r: type => { t; };",
        "mut let second = (self, _ uint8: type): compile -> let r: type => { self; };",
    ]);
    let initializer = initializer_from_source("let R: type = uint8 pick;");
    let call_site = extract_single_call_site(&initializer).expect("normalized overloaded call");
    let actual = [PolicyMode::Const];
    let writable = WritableContext::default();
    let context =
        OrdinaryInvocationContext::open_static(&actual).requiring_target_writable(&writable);
    let failure = world
        .invoke_ordinary_call(
            world.package_root_node(),
            &call_site,
            context,
            Provenance::new("PolicyMode::Mut does not imply Writable authority"),
        )
        .expect_err("mut Policy alone cannot authorize a Place write");
    assert!(matches!(
        failure,
        lang_build::OrdinaryInvocationFailure::DynamicLegality { ref diagnostic, .. }
            if diagnostic.message.contains("actual target Place")
                || diagnostic.message.contains("not Writable")
    ));
}

#[test]
fn actual_callable_binding_place_authorizes_target_sensitive_legality() {
    let mut world = support::AssociatedFamily::new(&[
        "let member = (self, t:type):compile -> let r:type => { t; };",
    ]);
    let binding = world.target_binding();
    let value = binding.ordinary_value().unwrap();
    let place = world
        .semantic_world()
        .binding_place(binding.identity, value)
        .unwrap();
    let mut writable = WritableContext::default();
    writable.grant_place(place);
    let call =
        extract_single_call_site(&initializer_from_source("let result = uint8 member;")).unwrap();
    let failure = world
        .invoke_ordinary_call(
            world.package_root_node(),
            &call,
            OrdinaryInvocationContext::open_static(&[PolicyMode::Const])
                .requiring_target_writable(&writable),
            Provenance::new("actual binding Place"),
        )
        .expect_err("selected user body completion remains unavailable");
    let lang_build::OrdinaryInvocationFailure::SelectedBody { trace, .. } = failure else {
        panic!("actual Place grant must pass legality: {failure:?}");
    };
    assert!(trace.dynamic_legality.is_some());
    assert!(trace.selected.is_some());
}

#[test]
fn callable_material_position_policy_inherits_stage_and_overlays_result_mode() {
    let world = support::AssociatedFamily::from_fixture("position_policy");
    let function = world.target_binding();
    let function_value = *function
        .ordinary_value()
        .as_ref()
        .expect("f has one function object");
    let call_entry = *world
        .semantic_world()
        .associated_values_for_value(function_value, "()")
        .and_then(|entries| entries.first())
        .expect("f owns a terminal call entry");
    let SemanticValuePayload::CallEntry(entry) = &world
        .semantic_world()
        .value(call_entry)
        .expect("call entry value")
        .payload
    else {
        panic!("associated () value is a call entry");
    };

    assert_eq!(entry.body_entry_view.mode, PolicyMode::Mut);
    assert_eq!(entry.callable_view.mode, PolicyMode::Const);
    assert_eq!(entry.complete_result_view, entry.body_entry_view);
    assert_eq!(entry.return_position_view.mode, PolicyMode::Mut);
    assert_eq!(
        entry.return_position_view.pair, entry.callable_view.pair,
        "P_out inherits the canonical P1 pair/stage byte-for-byte"
    );
    assert_eq!(
        entry.body_entry_view.pair, entry.return_position_view.pair,
        "omitted P1 completes to the single P2 stage; the authorities remain independent"
    );
}

#[test]
fn return_position_cannot_override_inherited_stage() {
    let error = support::AssociatedFamily::try_new(&[include_str!(
        "fixtures/workspaces/position_policy_invalid_stage/app/src/main.lang"
    )])
    .err()
    .expect("invalid return stage");
    assert!(
        error
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("inherits the P1 stage")),
        "return-stage rewrite is rejected during declaration Policy formation: {:?}",
        error.diagnostics
    );
}

// ---------------------------------------------------------------------------

#[test]
fn ordinary_type_binding_reuses_type_and_pattern_without_rerooting() {
    let world = build_single_fixture_world("single_package_type_binding", "app");
    let bound = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "T")
        .expect("source binding T");
    let core = world
        .semantic_world()
        .symbol_in_namespace(world.core_node(), "uint8")
        .expect("core uint8");
    let rebound = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "U")
        .expect("source binding U");
    assert_ne!(bound.identity, core.identity);
    assert_ne!(rebound.identity, bound.identity);

    let bound_type = world
        .semantic_world()
        .core_type_projection_value_for_symbol(bound.identity)
        .expect("T pure type Object value");
    let bound_value = world
        .semantic_world()
        .value(bound_type)
        .expect("T value facet");
    let core_type = world
        .semantic_world()
        .core_type_projection_value_for_symbol(core.identity)
        .expect("uint8 pure type Object value");
    let core_value = world
        .semantic_world()
        .value(core_type)
        .expect("uint8 value facet");
    assert_eq!(
        bound_type, core_type,
        "`let T: type = uint8` binds the existing type pattern; it does not allocate a second type pattern"
    );
    let rebound_type = world
        .semantic_world()
        .core_type_projection_value_for_symbol(rebound.identity)
        .expect("U pure type Object value");
    assert_eq!(
        rebound_type, bound_type,
        "`let U: type = T` reads the value carried by T and binds that same value; the RHS carrier is not identity"
    );
    let SemanticValuePayload::CoreTypeProjection {
        represented_type,
        represented_pattern,
        ..
    } = core_value.payload
    else {
        panic!("core uint8 carries a CoreTypeProjection graph value");
    };
    assert_eq!(bound_value.type_value, core_value.type_value);
    assert_eq!(bound_value.pattern, core_value.pattern);
    assert_ne!(
        core_value.type_value, represented_type,
        "the CoreTypeProjection graph value has rank `type`; it is not an instance of represented uint8"
    );
    assert_eq!(represented_pattern, core_value.pattern);
    assert_eq!(
        world
            .semantic_world()
            .pattern_owner(bound_value.pattern)
            .expect("shared Pattern owner")
            .owner,
        world
            .semantic_world()
            .pattern_owner(core_value.pattern)
            .expect("core Pattern owner")
            .owner,
        "binding installation must not reroot PatternValue ownership"
    );

    let bound_graph_symbol = world
        .resolve_with_expectation("T", lang_build::ResolveExpectation::CoreTypeProjection)
        .expect("graph-level forwarding type binding");
    let lang_build::SymbolPayload::CompleteTypeProjection(bound_type) = bound_graph_symbol.payload
    else {
        panic!("T has the graph carrier required for source navigation/place semantics");
    };
    let core_graph_symbol = world
        .resolve_with_expectation("uint8", lang_build::ResolveExpectation::CoreTypeProjection)
        .expect("core uint8 graph carrier");
    let lang_build::SymbolPayload::CompleteTypeProjection(core_type) = core_graph_symbol.payload
    else {
        panic!("uint8 is a graph CompleteType projection");
    };
    assert_ne!(
        bound_type.carrier_symbol_id, core_type.carrier_symbol_id,
        "ordinary `=` creates a fresh LHS carrier rather than forwarding the RHS Symbol"
    );
    assert_eq!(
        bound_type.carrier_symbol_id, bound_graph_symbol.id,
        "the graph Type carrier belongs to the fresh LHS Symbol"
    );
    assert_eq!(
        bound_type.represented_type, core_type.represented_type,
        "both carrier Symbols expose the same evaluated TypeValue"
    );
    let rebound_graph_symbol = world
        .resolve_with_expectation("U", lang_build::ResolveExpectation::CoreTypeProjection)
        .expect("U graph carrier");
    let lang_build::SymbolPayload::CompleteTypeProjection(rebound_type) =
        rebound_graph_symbol.payload
    else {
        panic!("U is a graph CompleteType projection");
    };
    assert_ne!(rebound_type.carrier_symbol_id, bound_type.carrier_symbol_id);
    assert_eq!(rebound_type.represented_type, bound_type.represented_type);
    let companion = bound_type
        .type_associated_namespace
        .expect("graph projection installs a companion place for T");
    assert_eq!(
        world
            .semantic_world()
            .pattern_for_associated_namespace(companion),
        Some(bound_value.pattern),
        "navigation through the fresh carrier place still obtains the bound value's existing PatternValue"
    );
    assert_ne!(
        world
            .semantic_world()
            .namespace_owner(companion)
            .expect("T companion place has its own namespace owner"),
        world
            .semantic_world()
            .pattern_owner(bound_value.pattern)
            .expect("bound PatternValue keeps its owner")
            .owner,
        "routing a carrier place to the existing PatternValue must not reroot its Pattern owner"
    );
}

#[test]
fn rebound_type_value_keeps_complete_identity_and_distinct_carriers() {
    let world = build_single_fixture_world("single_package_type_binding", "app");
    let direct = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "Direct")
        .expect("Direct type binding");
    let rebound = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "Rebound")
        .expect("carrier rebinding of Direct");

    assert_eq!(
        direct.pure_p_pattern(), rebound.pure_p_pattern(),
        "a carrier rebinding refers to the same complete type, so the rebound carrier name cannot change type identity"
    );

    let direct_graph = world
        .resolve_with_expectation("Direct", lang_build::ResolveExpectation::CoreTypeProjection)
        .expect("Direct graph carrier");
    let rebound_graph = world
        .resolve_with_expectation(
            "Rebound",
            lang_build::ResolveExpectation::CoreTypeProjection,
        )
        .expect("Rebound graph carrier");
    let (
        lang_build::SymbolPayload::CompleteTypeProjection(direct_type),
        lang_build::SymbolPayload::CompleteTypeProjection(rebound_type),
    ) = (&direct_graph.payload, &rebound_graph.payload)
    else {
        panic!("both source declarations bind type values");
    };
    assert_ne!(
        direct_type.carrier_symbol_id,
        rebound_type.carrier_symbol_id
    );
    assert_eq!(direct_type.represented_type, rebound_type.represented_type);
    let direct_snapshot = direct.pure_p().unwrap().complete_type.unwrap();
    let rebound_snapshot = rebound.pure_p().unwrap().complete_type.unwrap();
    assert_eq!(direct_snapshot, rebound_snapshot);
    assert_ne!(direct.pure_p_place(), rebound.pure_p_place());
}

#[test]
fn original_declaration_is_preserved_across_carrier_rebinding() {
    let world = build_single_fixture_world("single_package_type_binding", "app");
    let bound = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "T")
        .expect("source binding T");
    let core = world
        .semantic_world()
        .symbol_in_namespace(world.core_node(), "uint8")
        .expect("core uint8");
    let rebound = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "U")
        .expect("source binding U");

    let bound_pure = bound.pure_p_pattern().expect("T has pure_p");
    let core_pure = core.pure_p_pattern().expect("uint8 has pure_p");
    let rebound_pure = rebound.pure_p_pattern().expect("U has pure_p");

    assert_eq!(bound_pure, core_pure);
    assert_eq!(rebound_pure, core_pure);

    let bound_declaration = world
        .semantic_world()
        .pattern_declaration(bound_pure)
        .expect("T PatternValue has original declaration");
    let core_declaration = world
        .semantic_world()
        .pattern_declaration(core_pure)
        .expect("uint8 PatternValue has original declaration");
    let rebound_declaration = world
        .semantic_world()
        .pattern_declaration(rebound_pure)
        .expect("U PatternValue has original declaration");

    assert_eq!(
        bound_declaration, core_declaration,
        "carrier rebinding must not change the canonical original declaration"
    );
    assert_eq!(
        rebound_declaration, core_declaration,
        "two-level carrier rebinding must still refer to the original original declaration"
    );
    assert_ne!(
        bound.identity, core.identity,
        "carrier Symbol identity is distinct from the original declaration identity"
    );
}

/// `let T: type = uint8; let U: type = T;` — shared Pattern identity, three
/// separate objects:
///
/// ```text
/// Pattern(T)   = Pattern(U)   = Pattern(uint8)
/// TypeValue(T) = TypeValue(U) = TypeValue(uint8)
/// Symbol(T)   != Symbol(U)   != Symbol(uint8)
/// Place(T)    != Place(U)    != Place(uint8)
/// ```
///
/// The place inequality is what makes `let f::T = ...` a write to `T`'s own
/// pure-pure type Object instead of to the PatternValue that `U` and `uint8`
/// share, so it is the structural difference between `let =` and `let ===`.
#[test]
fn ordinary_type_bindings_own_distinct_val2_places() {
    let world = build_single_fixture_world("single_package_type_binding", "app");
    let semantic = world.semantic_world();
    let bound = semantic
        .symbol_in_namespace(world.package_root_node(), "T")
        .expect("source binding T");
    let rebound = semantic
        .symbol_in_namespace(world.package_root_node(), "U")
        .expect("source binding U");
    let core = semantic
        .symbol_in_namespace(world.core_node(), "uint8")
        .expect("core uint8");

    let pattern = core.pure_p_pattern().expect("uint8 has pure_p");
    assert_eq!(bound.pure_p_pattern(), Some(pattern));
    assert_eq!(rebound.pure_p_pattern(), Some(pattern));
    let type_value = semantic
        .type_for_pattern(pattern)
        .expect("the shared Pattern denotes one TypeValue");
    assert_eq!(
        semantic.type_for_pattern(bound.pure_p_pattern().expect("T pure_p")),
        Some(type_value),
        "a carrier rebinding never mints a new TypeValue"
    );

    assert_ne!(bound.identity, core.identity);
    assert_ne!(rebound.identity, core.identity);
    assert_ne!(bound.identity, rebound.identity);

    let bound_place = bound.pure_p_place().expect("T's pure P is a real object");
    let rebound_place = rebound.pure_p_place().expect("U's pure P is a real object");
    let core_place = core
        .pure_p_place()
        .expect("uint8's pure P is a real object");
    assert_ne!(
        bound_place, core_place,
        "`let T: type = uint8` binds a new object, so T owns a fresh writable place"
    );
    assert_ne!(
        rebound_place, core_place,
        "`let U: type = T` binds a new object too"
    );
    assert_ne!(
        bound_place, rebound_place,
        "two ordinary bindings of one Pattern never share one Val2 place"
    );

    // The Pattern's canonical pure type Object belongs to the declaration that
    // declared the Pattern; neither rebinding writes there.
    let canonical = semantic
        .pattern_place(pattern)
        .expect("the Pattern has a canonical pure type Object");
    assert_ne!(bound_place, canonical);
    assert_ne!(rebound_place, canonical);
}

#[test]
fn named_pattern_applicability_consumes_pattern_value_not_carrier_name() {
    let world = build_single_fixture_world("single_package_type_binding", "app");
    let rebound = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "T")
        .expect("ordinary type-value binding T");
    let result = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "PatternResult")
        .expect("source call selected `_ uint8` with T as the actual");
    let core = world
        .semantic_world()
        .symbol_in_namespace(world.core_node(), "uint8")
        .expect("core uint8");

    assert_eq!(rebound.pure_p_pattern(), core.pure_p_pattern());
    assert_eq!(
        result.pure_p_pattern(), core.pure_p_pattern(),
        "named Pattern applicability must compare the resolved PatternValue reached through T, not the spelling `T`"
    );
}

#[test]
fn core_identity_is_a_function_object_on_the_ordinary_spine() {
    let mut world =
        CompilationWorld::from_manifest(&BuildManifest::new("app", vec!["app".to_string()]))
            .expect("core semantic world builds");
    let initializer = initializer_from_source("let result = uint8 IdentityType;");
    let call_site = extract_single_call_site(&initializer).expect("normalized core call");
    let actual_mutability = [PolicyMode::Const];
    let result = world
        .invoke_ordinary_call(
            world.package_root_node(),
            &call_site,
            OrdinaryInvocationContext::open_static(&actual_mutability),
            Provenance::new("core IdentityType ordinary invocation"),
        )
        .expect("core primitive uses the ordinary function-object trunk");
    let lang_build::InvocationResult::SemanticResult {
        declared_result_class: lang_build::DeclaredResultClass::CompleteType,
        value: lang_build::ProjectedInvocationOutcome::SingleMember(result),
    } = result
    else {
        panic!("declared CompleteType is the sole result-class authority");
    };

    let identity = world
        .semantic_world()
        .symbol_in_namespace(world.core_node(), "IdentityType")
        .expect("core primitive has a semantic Symbol/value facet");
    assert_eq!(
        result.trace.c0_target_values,
        identity.ordinary_value().into_iter().collect::<Vec<_>>()
    );
    assert_eq!(result.trace.c3_call_entries.len(), 1);
    let instance = result
        .trace
        .compile_instance
        .expect("external type result has an instance");
    let state = world.semantic_world().compile_instance(instance).unwrap();
    assert_eq!(
        state.root.parent_owner,
        world
            .semantic_world()
            .namespace_owner(world.package_root_node())
            .unwrap()
    );
    assert!(
        world
            .semantic_world()
            .symbol(state.invoke_name())
            .unwrap()
            .pure_p()
            .is_none(),
        "returning an existing type does not install it as the instance self resident"
    );
    assert_eq!(
        state.delivered_result().unwrap().complete_type,
        result.complete_type.as_ref().map(|ty| ty.whole())
    );

    let uint8 = world
        .semantic_world()
        .symbol_in_namespace(world.core_node(), "uint8")
        .expect("core uint8 TypeValue");
    let uint8_type = world
        .semantic_world()
        .core_type_projection_value_for_symbol(uint8.identity)
        .expect("uint8 pure type Object value");
    assert!(
        result.complete_result[0].value.is_none(),
        "IdentityType returns a type result (value=None)"
    );
    assert_eq!(
        result.complete_result[0].pattern,
        world
            .semantic_world()
            .value(uint8_type)
            .expect("uint8 pure type Object")
            .pattern,
        "the core identity implementation returns the uint8 PatternValue"
    );
}

#[test]
fn production_world_owns_one_lifecycle_name_map_across_invocations() {
    let mut world =
        CompilationWorld::from_manifest(&BuildManifest::new("app", vec!["app".to_string()]))
            .expect("core semantic world builds");
    let initializer = initializer_from_source("let result = uint8 IdentityType;");
    let call_site = extract_single_call_site(&initializer).expect("normalized core call");
    let first = world
        .invoke_ordinary_call(
            world.package_root_node(),
            &call_site,
            OrdinaryInvocationContext::open_static(&[PolicyMode::Const]),
            Provenance::new("first lifecycle-owned call"),
        )
        .expect("first call");
    let lang_build::InvocationResult::SemanticResult {
        value: lang_build::ProjectedInvocationOutcome::SingleMember(first),
        ..
    } = first
    else {
        panic!("identity returns one member");
    };
    let target = first.selected.target_value;
    let first_name = world
        .lifecycle()
        .name_of(target)
        .expect("production invocation discovers the callable's stable name");
    let _ = world
        .invoke_ordinary_call(
            world.package_root_node(),
            &call_site,
            OrdinaryInvocationContext::open_static(&[PolicyMode::Const]),
            Provenance::new("second lifecycle-owned call"),
        )
        .expect("second call");
    assert_eq!(
        world.lifecycle().name_of(target),
        Some(first_name),
        "one CompilationWorld keeps one stable LifeName map"
    );
    let snapshot = world.lifecycle().snapshot(
        lang_build::ColorAlgebra::default(),
        lang_build::AccessSnapshot::default(),
    );
    assert!(
        !snapshot.live.contains(&first_name),
        "roster discovery does not establish Alive"
    );
    let mut k = world.continuation().clone();
    k.freeze_cleanup_through(k.position()).unwrap();
    assert_eq!(
        world.lifecycle().reify_value(&k, target),
        Err(lang_build::LifecycleFailure::FormationPending(first_name)),
        "sync supplies neither the formation cut nor an origin termination fact"
    );
}

#[test]
fn core_identity_consumes_type_value_not_rhs_carrier_symbol() {
    let mut world = build_single_fixture_world("single_package_type_binding", "app");
    let initializer = initializer_from_source("let result = U IdentityType;");
    let call_site = extract_single_call_site(&initializer).expect("normalized core call");
    let result = world
        .invoke_ordinary_call(
            world.package_root_node(),
            &call_site,
            OrdinaryInvocationContext::open_static(&[PolicyMode::Const]),
            Provenance::new("complete-type value authority"),
        )
        .expect("IdentityType accepts the value read through U");
    let lang_build::InvocationResult::SemanticResult {
        declared_result_class,
        value: lang_build::ProjectedInvocationOutcome::SingleMember(result),
    } = result
    else {
        panic!("expected ordinary outcome");
    };
    assert_eq!(
        declared_result_class,
        lang_build::DeclaredResultClass::CompleteType
    );

    let uint8 = world
        .semantic_world()
        .symbol_in_namespace(world.core_node(), "uint8")
        .expect("core uint8 TypeValue");
    assert!(result.complete_result[0].value.is_none());
    assert_eq!(
        result.complete_result[0].pattern,
        uint8.pure_p_pattern().unwrap()
    );
    let lang_build::ReturnedSemanticEntity::CompleteType(value) = result.returned else {
        panic!("IdentityType returns the evaluated complete type value");
    };
    let uint8_type = world
        .semantic_world()
        .core_type_projection_value_for_symbol(uint8.identity)
        .expect("uint8 pure type Object value");
    let represented = value.complete_type.lookup_key();
    let SemanticValuePayload::CoreTypeProjection {
        represented_type, ..
    } = world
        .semantic_world()
        .value(uint8_type)
        .expect("uint8 CoreTypeProjection value")
        .payload
    else {
        panic!("uint8 carries a CoreTypeProjection");
    };
    assert_eq!(represented, represented_type);
}

#[test]
fn bare_call_target_resolves_nearest_symbol_once_even_if_non_callable() {
    let mut world = build_single_fixture_world("bare_scope_chain", "app");
    let package = world.package_root_node();
    let outer_namespace = world
        .semantic_world()
        .child_namespace(package, "outer")
        .expect("outer physical namespace");
    let inner_namespace = world
        .semantic_world()
        .child_namespace(outer_namespace, "inner")
        .expect("inner physical namespace");
    let inner = world
        .semantic_world()
        .symbol_in_namespace(inner_namespace, "f")
        .expect("inner non-callable f")
        .clone();
    assert!(
        inner.ordinary_value().is_none(),
        "near f is deliberately non-callable"
    );

    let initializer = initializer_from_source("let result = uint8 f;");
    let call_site = extract_single_call_site(&initializer).expect("bare f call");
    let failure = world
        .invoke_ordinary_call(
            inner_namespace,
            &call_site,
            OrdinaryInvocationContext::open_static(&[PolicyMode::Const]),
            Provenance::new("near outer core bare-name chain"),
        )
        .expect_err("the nearest non-callable Symbol shadows outer callable Symbols");
    assert!(
        matches!(
            failure,
            lang_build::OrdinaryInvocationFailure::NoTargetValues { .. }
                | lang_build::OrdinaryInvocationFailure::NoFullyAdmissibleCandidate { .. }
        ),
        "call projection fails on inner.f and never re-resolves the name: {failure:?}"
    );
}

#[test]
fn same_bare_path_has_one_terminal_symbol_before_value_type_and_call_projection() {
    let world = build_single_fixture_world("bare_scope_chain", "app");
    let package = world.package_root_node();
    let outer_namespace = world
        .semantic_world()
        .child_namespace(package, "outer")
        .expect("outer physical namespace");
    let inner_namespace = world
        .semantic_world()
        .child_namespace(outer_namespace, "inner")
        .expect("inner physical namespace");
    let inner = world
        .semantic_world()
        .symbol_in_namespace(inner_namespace, "f")
        .expect("inner type-valued f")
        .identity;
    let initializer = initializer_from_source("let result = uint8 f;");
    let call_site = extract_single_call_site(&initializer).expect("bare f call");

    let neutral = world
        .resolve_source_terminal_symbol(inner_namespace, &call_site.target)
        .expect("neutral source resolution");
    let by_symbol_path = world
        .semantic_world()
        .resolve_symbol_path(
            &["f".to_string()],
            inner_namespace,
            &[world.semantic_world().namespace_index().root_node()],
            &[world.core_node()],
        )
        .expect("context-independent Symbol resolution");
    assert_eq!(neutral, inner);
    assert_eq!(by_symbol_path, inner);
    assert!(
        world
            .semantic_world()
            .symbol(inner)
            .and_then(|symbol| symbol.pure_p_pattern())
            .is_some(),
        "type projection observes the already resolved inner Symbol"
    );
}

#[test]
fn explicit_call_target_is_one_symbol_and_never_falls_back() {
    let mut world = build_single_fixture_world("bare_scope_chain", "app");
    let package = world.package_root_node();
    let outer_namespace = world
        .semantic_world()
        .child_namespace(package, "outer")
        .expect("outer physical namespace");
    let initializer = initializer_from_source("let result = uint8 f::inner;");
    let call_site = extract_single_call_site(&initializer).expect("explicit inner::f call");
    let failure = world
        .invoke_ordinary_call(
            outer_namespace,
            &call_site,
            OrdinaryInvocationContext::open_static(&[PolicyMode::Const]),
            Provenance::new("explicit target no-fallback"),
        )
        .expect_err("explicit inner f is non-callable and must fail");
    assert!(
        matches!(
            failure,
            lang_build::OrdinaryInvocationFailure::NoTargetValues { .. }
                | lang_build::OrdinaryInvocationFailure::NoFullyAdmissibleCandidate { .. }
        ),
        "explicit target fails on that Symbol instead of trying outer/core: {failure:?}"
    );
}

#[test]
fn selected_type_forwarder_delivers_exact_external_snapshot() {
    let mut world =
        CompilationWorld::from_manifest(&BuildManifest::new("app", vec!["app".to_string()]))
            .unwrap();
    let input = world
        .semantic_world()
        .symbol_in_namespace(world.core_node(), "uint8")
        .unwrap()
        .pure_p()
        .unwrap();
    let source = initializer_from_source("let T = uint8 IdentityType::core;");
    let call = extract_single_call_site(&source).unwrap();
    let result = world
        .invoke_ordinary_call(
            world.package_root_node(),
            &call,
            OrdinaryInvocationContext::open_static(&[PolicyMode::Const]),
            Provenance::new("external type forwarding"),
        )
        .unwrap();
    let lang_build::InvocationResult::SemanticResult {
        value: lang_build::ProjectedInvocationOutcome::SingleMember(result),
        ..
    } = result
    else {
        panic!("ordinary complete result");
    };
    assert_eq!(result.trace.c0_target_values.len(), 1);
    assert_eq!(
        result.trace.c1_visible_values,
        result.trace.c0_target_values
    );
    assert_eq!(result.trace.c3_call_entries.len(), 1);
    assert!(result.trace.selected.is_some());
    let instance = result.trace.compile_instance.unwrap();
    let lang_build::ReturnedSemanticEntity::CompleteType(returned) = &result.returned else {
        panic!("exact tau");
    };
    assert_eq!(result.complete_type.as_ref(), Some(&returned.complete_type));
    assert_eq!(returned.pattern, input.pattern);
    assert_eq!(
        world
            .semantic_world()
            .complete_type_by_whole_observation(returned.complete_type.whole()),
        Some(&returned.complete_type)
    );
    assert_ne!(
        world
            .semantic_world()
            .pattern_owner(returned.pattern)
            .unwrap()
            .owner,
        instance.owner()
    );
    let self_name = world
        .semantic_world()
        .compile_instance(instance)
        .unwrap()
        .invoke_name();
    assert!(world
        .semantic_world()
        .symbol(self_name)
        .unwrap()
        .pure_p()
        .is_none());
}

#[test]
fn unsupported_pattern_query_terminates_before_candidate_selection() {
    let mut world = support::AssociatedFamily::new(&[
        "let first = (self, if | else: type): compile -> let r: uint8 => { self; };",
        "let second = (self, t: type): compile -> let r: uint8 => { self; };",
    ]);
    let initializer = initializer_from_source("let result = uint8 choose;");
    let call_site = extract_single_call_site(&initializer).expect("normalized ordinary call");
    let failure = world
        .invoke_ordinary_call(
            world.package_root_node(),
            &call_site,
            OrdinaryInvocationContext::open_static(&[PolicyMode::Const]),
            Provenance::new("canonical Pattern query frontier"),
        )
        .expect_err("an unanswered Pattern query makes the candidate set incomplete");

    let lang_build::OrdinaryInvocationFailure::ApplicabilityUnsupported { diagnostic, trace } =
        failure
    else {
        panic!("unsupported applicability must be terminal before maxima: {failure:?}");
    };
    assert!(
        diagnostic.message.contains("relational alternative query"),
        "the terminal diagnostic comes from the unanswered canonical relation: {diagnostic:?}"
    );
    assert_eq!(
        trace.c3_call_entries.len(),
        2,
        "the callable projection contains both the unsupported query and an otherwise applicable candidate"
    );
    assert!(
        trace.selected.is_none(),
        "no candidate may be selected from an incomplete A-stage"
    );
}

#[test]
fn wildcard_unit_return_pattern_reaches_selection_before_execution_frontier() {
    let mut world = support::AssociatedFamily::from_fixture("unit_result_selection");
    let initializer = initializer_from_source("let result = uint8 unit_pick;");
    let call_site = extract_single_call_site(&initializer).expect("normalized ordinary call");
    let failure = world
        .invoke_ordinary_call(
            world.package_root_node(),
            &call_site,
            OrdinaryInvocationContext::open_static(&[PolicyMode::Const]),
            Provenance::new("unit result selection"),
        )
        .expect_err("Unit execution is an explicit implementation frontier");

    let lang_build::OrdinaryInvocationFailure::SelectedImplementation { diagnostic, trace } =
        failure
    else {
        panic!("return Pattern shape must not remove the candidate during A: {failure:?}");
    };
    assert!(diagnostic.message.contains("Unit result"));
    assert_eq!(trace.a_fully_admissible.len(), 1);
    assert_eq!(trace.selected, trace.a_fully_admissible.first().copied());
}
