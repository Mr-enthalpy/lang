mod support;

use lang_build::{
    extract_single_call_site, BuildManifest, CapabilityRealization, CapabilityRealizationCell,
    CompilationWorld, LifecyclePrecondition, LifecycleValidationContext, OrdinaryInvocationContext,
    PatternComponentPolicy, PolicyMigrationRequest, PolicyMode, PolicyPair, PolicyStage,
    Provenance, ResolveExpectation, SemanticOwnerKind, SemanticValuePayload, StageSet,
    SymbolPayload, ToolchainGlobalSourceRoot, ValueComponentPolicy, ValuePresence, WritableContext,
};

use support::{
    build_fixture_error, build_single_fixture_world, fixture_root, initializer_from_source,
};

fn transport_bundle() -> ToolchainGlobalSourceRoot {
    ToolchainGlobalSourceRoot::under(
        fixture_root()
            .join("global_implementation")
            .join("uint8_transport"),
        vec!["core".to_string(), "uint8".to_string()],
    )
}

fn build_transport_world() -> CompilationWorld {
    let mut manifest = BuildManifest::new("app", vec!["app".to_string()]);
    manifest
        .global_implementation_roots
        .push(transport_bundle());
    CompilationWorld::from_manifest(&manifest).expect("transport bundle builds")
}

fn stages(items: &[PolicyStage]) -> StageSet {
    let mut s = StageSet::new();
    for stage in items {
        s.insert(*stage);
    }
    s
}

fn pair(
    value_stages: &[PolicyStage],
    pattern_stages: &[PolicyStage],
    _mode: &[PolicyMode],
) -> PolicyPair {
    PolicyPair {
        value: ValueComponentPolicy {
            stages: stages(value_stages),
            presence: ValuePresence::Present,
        },
        pattern: PatternComponentPolicy {
            stages: stages(pattern_stages),
        },
    }
}

// ---------------------------------------------------------------------------
// Invariant tests: I1-I12 from the semantic model closure
// ---------------------------------------------------------------------------

#[test]
fn i1_let_parens_never_changes_sibling_vals() {
    // I1 — sibling_vals only changes when a declaration's binder name matches
    // an existing cluster Symbol.  Before any transport fixture is loaded,
    // sibling_vals is empty.  After loading the transport fixture (4
    // declarations named `uint8`), sibling_vals has exactly 5 entries,
    // including the canonical plain-input transport.
    // Named methods like `identity` and `type_identity` do NOT match the
    // `uint8` cluster Symbol and must appear in Val2[name], not sibling_vals.

    // Before: no transport fixture, sibling_vals is empty.
    let before_world = build_single_fixture_world("single_package_type_binding", "app");
    let before_uint8 = before_world
        .semantic_world()
        .symbol_in_namespace(before_world.core_node(), "uint8")
        .expect("core uint8");
    assert_eq!(
        before_uint8.sibling_vals.len(),
        0,
        "I1 before: no transports, sibling_vals is empty"
    );

    // After: transport fixture loaded, sibling_vals has exactly 5.
    let after_world = build_transport_world();
    let after_uint8 = after_world
        .semantic_world()
        .symbol_in_namespace(after_world.core_node(), "uint8")
        .expect("core uint8 with transports");
    assert_eq!(
        after_uint8.sibling_vals.len(),
        5,
        "I1 after: exactly 5 transports named `uint8` are cluster sibling vals"
    );

    // `identity` and `type_identity` must NOT be cluster siblings — they are
    // registered as ordinary source callables in Val2[name].
    let pattern = after_world
        .semantic_world()
        .pattern_for_associated_namespace(after_world.core_node());
    if let Some(pat) = pattern {
        let identity_vals = after_world
            .semantic_world()
            .associated_values_for_pattern(pat, "identity")
            .map(|vals| vals.len())
            .unwrap_or(0);
        assert_eq!(
            identity_vals, 1,
            "I1: `identity` is in Val2[\"identity\"], not sibling_vals"
        );
        let type_identity_vals = after_world
            .semantic_world()
            .associated_values_for_pattern(pat, "type_identity")
            .map(|vals| vals.len())
            .unwrap_or(0);
        assert_eq!(
            type_identity_vals, 1,
            "I1: `type_identity` is in Val2[\"type_identity\"], not sibling_vals"
        );
    }
}

#[test]
fn i2_every_callable_sibling_is_function_object() {
    let world = build_single_fixture_world("s10_cluster_exposure", "app");
    let pick = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "pick")
        .expect("pick symbol");
    for v in &pick.sibling_vals {
        let obj = world.semantic_world().value(*v).unwrap();
        assert!(
            matches!(obj.payload, SemanticValuePayload::FunctionObject { .. }),
            "I2: every callable sibling is a FunctionObject"
        );
    }
}

#[test]
fn i8_call_entry_is_terminal_function_item() {
    let world = build_single_fixture_world("s10_cluster_exposure", "app");
    let pick = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "pick")
        .expect("pick symbol");
    for v in &pick.sibling_vals {
        let entries = world
            .semantic_world()
            .associated_values_for_value(*v, "()")
            .unwrap_or(&[]);
        for entry in entries.iter().copied() {
            let call = world.semantic_world().value(entry).unwrap();
            assert!(
                matches!(call.payload, SemanticValuePayload::CallEntry(_)),
                "I8: () entry is terminal FunctionItem"
            );
            assert!(
                world
                    .semantic_world()
                    .associated_values_for_pattern(call.pattern, "()")
                    .is_none(),
                "I8: call entry Val2 is empty (terminal)"
            );
            // Test A — function object type != call-entry FunctionItem type,
            // function object pattern != call-entry pattern.  Each terminal
            // call entry has an independent FunctionItem type and pattern
            // allocated by allocate_terminal_call_entry.
            let func_obj = world.semantic_world().value(*v).unwrap();
            assert_ne!(
                func_obj.type_value, call.type_value,
                "Test A: function object type != call-entry FunctionItem type"
            );
            assert_ne!(
                func_obj.pattern, call.pattern,
                "Test A: function object pattern != call-entry pattern"
            );
        }
    }
}

#[test]
fn i11_sibling_vals_different_from_pure_p_val2() {
    // Cluster sibling vals and pure-P.Val2 are different structural layers.
    let world = build_single_fixture_world("single_package_type_binding", "app");
    let uint8 = world
        .semantic_world()
        .symbol_in_namespace(world.core_node(), "uint8")
        .expect("core uint8");
    assert!(uint8.pure_p_pattern().is_some());
    // sibling_vals contains no TypeObject adapter
    assert!(uint8.sibling_vals.is_empty());
    // pure-P.Val2["()"] may contain call entries from let () declarations,
    // which are NOT sibling vals. This structural separation is invariant I11.
}

#[test]
fn i12_type_object_not_in_sibling_vals() {
    let world = build_single_fixture_world("single_package_type_binding", "app");
    let uint8 = world
        .semantic_world()
        .symbol_in_namespace(world.core_node(), "uint8")
        .expect("core uint8");
    assert!(uint8.sibling_vals.is_empty());
    assert!(uint8.pure_p_pattern().is_some());
    // TypeObject adapter is accessible through type_object_value_for_symbol,
    // never through sibling_vals.
    let type_obj = world
        .semantic_world()
        .type_object_value_for_symbol(uint8.identity)
        .expect("type object adapter exists");
    let val = world.semantic_world().value(type_obj).unwrap();
    assert!(
        matches!(val.payload, SemanticValuePayload::TypeObject { .. }),
        "I12: TypeObject is compatibility adapter, not semantic Val1"
    );
}

#[test]
fn struct_binding_carries_exact_tau_independently_of_typeobject_projection() {
    let world = build_single_fixture_world("struct_single_field", "app");
    let binding = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "T")
        .expect("struct result is bound as T");
    let member = binding
        .pure_p
        .expect("T carries the returned pure type object");
    let whole = member
        .complete_type
        .expect("the binding stores the returned exact complete tau snapshot");
    let complete = world
        .semantic_world()
        .complete_type_by_whole_observation(whole)
        .expect("the exact complete tau remains interned");
    assert_eq!(complete.whole, whole);
    assert_eq!(
        world.semantic_world().type_for_pattern(member.pattern),
        Some(complete.lookup_key),
        "the Core lookup projection agrees with tau without defining its whole identity"
    );
}

#[test]
fn i9_slot0_is_selected_callable_function_object() {
    // I9 — slot 0 in a transport invocation is the selected transport function
    // object (a cluster sibling val), NOT the migration source.  The migration
    // source is passed as slot 1 (the first explicit argument / Source formal).
    // This is verified by performing an actual migration invocation and
    // checking that the invocation frame's c0_target_values are the cluster
    // sibling vals, while the source appears in the explicit argument product.
    let mut world = build_transport_world();
    let uint8 = world
        .semantic_world()
        .symbol_in_namespace(world.core_node(), "uint8")
        .expect("core uint8 with transports");
    let siblings = uint8.sibling_vals.clone();
    assert_eq!(
        siblings.len(),
        5,
        "I9: transport cluster has 5 sibling candidates for c0 enumeration"
    );

    // Verify that every sibling is a FunctionObject — these are the c0
    // target values (slot 0 candidates), not the migration source.
    for v in &siblings {
        let obj = world.semantic_world().value(*v).unwrap();
        assert!(
            matches!(obj.payload, SemanticValuePayload::FunctionObject { .. }),
            "I9: slot-0 candidate (sibling) is a FunctionObject"
        );
    }

    let source_policy = pair(
        &[PolicyStage::Compile],
        &[PolicyStage::Compile],
        &[PolicyMode::Const],
    );
    let target_policy = pair(
        &[PolicyStage::Runtime],
        &[PolicyStage::Compile],
        &[PolicyMode::Mut],
    );
    let uint8_type = match &world
        .resolve_with_expectation("uint8", ResolveExpectation::TypeObject)
        .expect("core uint8 type")
        .payload
    {
        SymbolPayload::Type(t) => t.represented_type,
        _ => panic!("uint8 resolves as a Type object"),
    };
    let source = world
        .install_semantic_value(
            uint8_type,
            source_policy.clone(),
            Provenance::new("I9 compile uint8 fixture value"),
        )
        .expect("installed source value");
    let request = PolicyMigrationRequest::new(
        lang_build::PolicyView {
            pair: source_policy,
            mode: PolicyMode::Const,
        },
        lang_build::ResultPolicyDemand {
            pair_query: lang_build::P1Projection::Pair(target_policy),
            mode: PolicyMode::Mut,
        },
        uint8_type,
        source,
        Provenance::new("I9 const compile -> mut runtime demand"),
    )
    .expect("legal migration request");

    let migration = world
        .invoke_policy_migration(&request)
        .expect("I9: migration invocation succeeds");

    // c0_target_values must be the cluster sibling vals (slot-0 candidates),
    // NOT the migration source.  The source appears in the explicit argument
    // product (slot 1).
    assert_eq!(
        migration.invocation.trace.c0_target_values.len(),
        siblings.len(),
        "I9: c0_target_values are cluster siblings (slot-0 candidates)"
    );
    assert!(
        !migration
            .invocation
            .trace
            .c0_target_values
            .contains(&source),
        "I9: migration source is NOT in c0_target_values (it is slot 1, not slot 0)"
    );
    assert_eq!(
        migration
            .invocation
            .selected
            .frame
            .explicit_arg_product
            .arity,
        1,
        "I9: migration source is in explicit_arg_product (slot 1)"
    );
}

#[test]
fn i14_finalize_construction_separate_from_install() {
    // I14 — the cluster construction lifecycle is: begin → contribute →
    // finalize → install.  Each phase is a distinct step:
    //   begin:    begin_cluster_construction creates an Open cluster
    //   contribute: contribute_cluster_pure_p sets pure_p on the open cluster
    //   finalize: finalize_type_cluster produces a
    //             SymbolConstructionValue (removes from open_clusters)
    //   install:  the construction value is installed as a cluster Symbol
    //             (upgrade_cluster_owner sets PatternClusterOwner::Installed)
    //
    // A cluster Symbol comes from a construction-family meta invocation
    // (`(uint8 field) struct`).  Ordinary callable declarations (e.g. the
    // s10 `pick` overloads) contribute sibling vals to their name Symbol
    // and never open a cluster themselves.
    let world = build_single_fixture_world("single_package_type_binding", "app");
    let direct = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "Direct")
        .expect("struct-generated Direct symbol");

    // contribute phase: pure_p was set during the struct meta invocation.
    assert!(
        direct.pure_p_pattern().is_some(),
        "I14 contribute: pure_p was set during meta invocation"
    );

    // finalize phase: the construction was finalized (not left Open).
    // After finalization, the cluster is removed from open_clusters.
    let pattern = direct.pure_p_pattern().unwrap();
    let owner = world.semantic_world().owner_cluster(pattern);
    assert!(
        owner.is_some(),
        "I14 finalize: pattern has a cluster owner (was finalized)"
    );

    // install phase: the cluster owner is Installed (a Symbol), not Open
    // (a ClusterConstructionId).  This verifies the final install step.
    use lang_build::semantic_world::PatternClusterOwner;
    let owner = owner.unwrap();
    assert!(
        matches!(owner, PatternClusterOwner::Installed(_)),
        "I14 install: cluster owner is Installed (a Symbol), not Open (a ClusterConstructionId)"
    );

    // sibling_vals accrue on an installed cluster Symbol through later
    // ContributeSiblingVal declarations (transport fixture), separate from
    // the install step itself.
    let transport_world = build_transport_world();
    let uint8 = transport_world
        .semantic_world()
        .symbol_in_namespace(transport_world.core_node(), "uint8")
        .expect("core uint8 cluster with transports");
    let uint8_owner = transport_world
        .semantic_world()
        .owner_cluster(uint8.pure_p_pattern().expect("core uint8 pure P"))
        .expect("core uint8 cluster owner");
    assert!(
        matches!(uint8_owner, PatternClusterOwner::Installed(_)),
        "I14: transported cluster stays Installed"
    );
    assert_eq!(
        uint8.sibling_vals.len(),
        5,
        "I14 contribute-after-install: cluster gains sibling_vals from transports"
    );
}

#[test]
fn i15_source_ordinary_call_begins_from_cluster_sibling_enumeration() {
    // Source ordinary call begins from ClusterSymbol sibling enumeration,
    // not from Pattern.Val2["()"].  This is structurally enforced by
    // the ordinary invocation trunk, which reads target_values from
    // the cluster's sibling_vals, not from associated_val2.
    let world = build_single_fixture_world("s10_cluster_exposure", "app");
    let pick = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "pick")
        .expect("pick symbol");
    // The sibling_vals are the callable candidates that the invocation
    // pipeline enumerates.  They are FunctionObjects, not Val2["()"] entries.
    assert!(
        !pick.sibling_vals.is_empty(),
        "I15: cluster has sibling vals for ordinary call enumeration"
    );
    for v in &pick.sibling_vals {
        let obj = world.semantic_world().value(*v).unwrap();
        assert!(
            matches!(obj.payload, SemanticValuePayload::FunctionObject { .. }),
            "I15: enumerated siblings are FunctionObjects (callable candidates)"
        );
    }
}

#[test]
fn dynamic_legality_runs_after_unique_selection_and_never_reopens_the_family() {
    let mut world = build_single_fixture_world("s10_cluster_exposure", "app");
    let initializer = initializer_from_source("let R: type = uint8 pick;");
    let call_site = extract_single_call_site(&initializer).expect("normalized overloaded call");
    let actual = [PolicyMode::Const];
    let failure = world
        .invoke_ordinary_call(
            world.package_root_node(),
            &call_site,
            OrdinaryInvocationContext::open_static(&actual)
                .with_capability_demand(PolicyMode::Const, PolicyMode::Mut),
            Provenance::new("post-selection capability death test"),
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
    let mut world = build_single_fixture_world("s10_cluster_exposure", "app");
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
            Provenance::new("post-selection lifecycle death test"),
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
    let mut world = build_single_fixture_world("s4_return_ontology", "app");
    let keep = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "keep")
        .expect("single compile callable Symbol");
    let entries = keep
        .sibling_vals
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
    let mut world = build_single_fixture_world("s10_cluster_exposure", "app");
    let initializer = initializer_from_source("let R: type = uint8 pick;");
    let call_site = extract_single_call_site(&initializer).expect("normalized overloaded call");
    let actual = [PolicyMode::Const];
    let writable = WritableContext::default();
    let mut context =
        OrdinaryInvocationContext::open_static(&actual).requiring_target_writable(&writable);
    context.caller_mode = PolicyMode::Mut;
    let failure = world
        .invoke_ordinary_call(
            world.package_root_node(),
            &call_site,
            context,
            Provenance::new("mut is not Writable death test"),
        )
        .expect_err("mut Policy alone cannot authorize a Place write");
    assert!(matches!(
        failure,
        lang_build::OrdinaryInvocationFailure::DynamicLegality { ref diagnostic, .. }
            if diagnostic.message.contains("not Writable")
    ));
}

#[test]
fn source_position_policy_inherits_stage_and_overlays_result_mode() {
    let world = build_single_fixture_world("position_policy", "app");
    let function = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "f")
        .expect("source callable f");
    let function_value = *function
        .sibling_vals
        .first()
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
    assert_ne!(
        entry.body_entry_view.pair, entry.return_position_view.pair,
        "declaration-local P2 remains distinct from callable-internal P_out"
    );
}

#[test]
fn return_position_cannot_override_inherited_stage() {
    let error = build_fixture_error("position_policy_invalid_stage", "app");
    assert!(
        error
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("inherits evaluation stages")),
        "return-stage rewrite is rejected during declaration Policy formation: {:?}",
        error.diagnostics
    );
}

// ---------------------------------------------------------------------------

#[test]
fn one_semantic_symbol_preserves_distinct_function_object_identities() {
    let world = build_single_fixture_world("s10_cluster_exposure", "app");
    let pick = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "pick")
        .expect("semantic `pick` Symbol");
    let mut ids = pick.sibling_vals.clone();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), pick.sibling_vals.len());

    let mut function_types = pick
        .sibling_vals
        .iter()
        .map(|value| {
            world
                .semantic_world()
                .value(*value)
                .expect("function value")
                .type_value
        })
        .collect::<Vec<_>>();
    function_types.sort();
    function_types.dedup();
    assert_eq!(
        function_types.len(),
        pick.sibling_vals.len(),
        "each source function object owns a distinct anonymous TypeValue"
    );
}

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
        .type_object_value_for_symbol(bound.identity)
        .expect("T type object value");
    let bound_value = world
        .semantic_world()
        .value(bound_type)
        .expect("T value facet");
    let core_type = world
        .semantic_world()
        .type_object_value_for_symbol(core.identity)
        .expect("uint8 type object value");
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
        .type_object_value_for_symbol(rebound.identity)
        .expect("U type object value");
    assert_eq!(
        rebound_type, bound_type,
        "`let U: type = T` reads the value carried by T and binds that same value; the RHS carrier is not identity"
    );
    let SemanticValuePayload::TypeObject {
        represented_type,
        represented_pattern,
        ..
    } = core_value.payload
    else {
        panic!("core uint8 carries a TypeObject adapter");
    };
    assert_eq!(bound_value.type_value, core_value.type_value);
    assert_eq!(bound_value.pattern, core_value.pattern);
    assert_ne!(
        core_value.type_value, represented_type,
        "the TypeObject adapter has rank `type`; it is not an instance of represented uint8"
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
        .resolve_with_expectation("T", lang_build::ResolveExpectation::TypeObject)
        .expect("graph-level forwarding type binding");
    let lang_build::SymbolPayload::Type(bound_type) = bound_graph_symbol.payload else {
        panic!("T has the graph carrier required for source navigation/place semantics");
    };
    let core_graph_symbol = world
        .resolve_with_expectation("uint8", lang_build::ResolveExpectation::TypeObject)
        .expect("core uint8 graph carrier");
    let lang_build::SymbolPayload::Type(core_type) = core_graph_symbol.payload else {
        panic!("uint8 is a graph Type object");
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
        .resolve_with_expectation("U", lang_build::ResolveExpectation::TypeObject)
        .expect("U graph carrier");
    let lang_build::SymbolPayload::Type(rebound_type) = rebound_graph_symbol.payload else {
        panic!("U is a graph Type object");
    };
    assert_ne!(rebound_type.carrier_symbol_id, bound_type.carrier_symbol_id);
    assert_eq!(rebound_type.represented_type, bound_type.represented_type);
    let companion = bound_type
        .type_associated_namespace
        .expect("legacy graph transport installs a companion place for T");
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
fn rebound_type_value_is_canonical_struct_field_material() {
    let world = build_single_fixture_world("single_package_type_binding", "app");
    let direct = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "Direct")
        .expect("struct whose field spells uint8");
    let rebound = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "Rebound")
        .expect("carrier rebinding of Direct");

    assert_eq!(
        direct.pure_p_pattern(), rebound.pure_p_pattern(),
        "a carrier rebinding refers to the same generated TypeValue, so the rebound carrier name cannot change type identity"
    );

    let direct_graph = world
        .resolve_with_expectation("Direct", lang_build::ResolveExpectation::TypeObject)
        .expect("Direct graph carrier");
    let rebound_graph = world
        .resolve_with_expectation("Rebound", lang_build::ResolveExpectation::TypeObject)
        .expect("Rebound graph carrier");
    let (
        lang_build::SymbolPayload::Type(direct_type),
        lang_build::SymbolPayload::Type(rebound_type),
    ) = (&direct_graph.payload, &rebound_graph.payload)
    else {
        panic!("both source declarations bind type values");
    };
    assert_ne!(
        direct_type.carrier_symbol_id,
        rebound_type.carrier_symbol_id
    );
    assert_eq!(direct_type.represented_type, rebound_type.represented_type);
    // Field material lives on the generated type value's own carrier; a
    // carrier rebinding is a bare reference and carries no field material
    // of its own.
    assert!(!direct_type.field_type_values.is_empty());
    assert!(rebound_type.field_type_values.is_empty());
}

#[test]
fn owner_cluster_preserved_across_carrier_rebinding() {
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

    let bound_cluster = world
        .semantic_world()
        .owner_cluster(bound_pure)
        .and_then(|owner| owner.installed())
        .expect("T PatternValue has owning cluster");
    let core_cluster = world
        .semantic_world()
        .owner_cluster(core_pure)
        .and_then(|owner| owner.installed())
        .expect("uint8 PatternValue has owning cluster");
    let rebound_cluster = world
        .semantic_world()
        .owner_cluster(rebound_pure)
        .and_then(|owner| owner.installed())
        .expect("U PatternValue has owning cluster");

    assert_eq!(
        bound_cluster, core_cluster,
        "carrier rebinding must not change the canonical owning cluster"
    );
    assert_eq!(
        rebound_cluster, core_cluster,
        "two-level carrier rebinding must still refer to the original owning cluster"
    );
    assert_ne!(
        bound.identity, core.identity,
        "carrier Symbol identity is distinct from the owning cluster identity"
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
/// pure-type object instead of to the PatternValue that `U` and `uint8`
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

    // The Pattern's canonical type object belongs to the cluster that
    // declared the Pattern; neither rebinding writes there.
    let canonical = semantic
        .pattern_place(pattern)
        .expect("the Pattern has a canonical type object");
    assert_ne!(bound_place, canonical);
    assert_ne!(rebound_place, canonical);
}

#[test]
fn cluster_pure_p_not_in_sibling_vals() {
    let world = build_single_fixture_world("single_package_type_binding", "app");
    let bound = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "T")
        .expect("source binding T");
    assert!(
        bound.pure_p_pattern().is_some(),
        "type binding T has a pure P PatternValue"
    );
    assert!(
        bound.sibling_vals.is_empty(),
        "TypeObject adapter value does not appear in sibling_vals"
    );
    let core = world
        .semantic_world()
        .symbol_in_namespace(world.core_node(), "uint8")
        .expect("core uint8");
    assert!(core.pure_p_pattern().is_some());
    assert!(
        core.sibling_vals.is_empty(),
        "core type without transport fixture has no sibling vals (TypeObject is not a sibling val)"
    );

    // Ordinary callable declarations cluster by name into a Symbol whose
    // pure_p is absent — the `pick` overloads live in the s10 fixture.
    let pick_world = build_single_fixture_world("s10_cluster_exposure", "app");
    let pick = pick_world
        .semantic_world()
        .symbol_in_namespace(pick_world.package_root_node(), "pick")
        .expect("callable `pick` symbol");
    assert!(
        pick.pure_p_pattern().is_none(),
        "callable symbol has no pure P, only sibling vals"
    );
    assert!(
        pick.sibling_vals.len() > 1,
        "callable sibling vals form an overload set"
    );
}

#[test]
fn callable_sibling_has_own_type_and_terminal_call_entry() {
    let world = build_single_fixture_world("s10_cluster_exposure", "app");
    let pick = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "pick")
        .expect("semantic `pick` Symbol");
    for value in &pick.sibling_vals {
        let value_obj = world
            .semantic_world()
            .value(*value)
            .expect("sibling val exists");
        assert!(
            matches!(
                value_obj.payload,
                SemanticValuePayload::FunctionObject { .. }
            ),
            "I2: every callable sibling is already a complete function object"
        );
        let entries = world
            .semantic_world()
            .associated_values_for_value(*value, "()")
            .expect("function object owns () via its type");
        for entry in entries {
            let call_obj = world
                .semantic_world()
                .value(*entry)
                .expect("call entry value exists");
            assert!(
                matches!(call_obj.payload, SemanticValuePayload::CallEntry(_)),
                "I8: call entry is terminal FunctionItem"
            );
            assert!(
                world
                    .semantic_world()
                    .associated_values_for_pattern(call_obj.pattern, "()")
                    .is_none(),
                "I8: call entry Val2 is empty — terminal"
            );
        }
    }
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
            Provenance::new("core IdentityType ordinary-spine regression"),
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
    assert_eq!(result.trace.c0_target_values, identity.sibling_vals);
    assert_eq!(result.trace.c3_call_entries.len(), 1);

    let uint8 = world
        .semantic_world()
        .symbol_in_namespace(world.core_node(), "uint8")
        .expect("core uint8 TypeValue");
    let uint8_type = world
        .semantic_world()
        .type_object_value_for_symbol(uint8.identity)
        .expect("uint8 type object value");
    assert!(
        result.complete_result[0].value.is_none(),
        "IdentityType returns a type result (value=None)"
    );
    assert_eq!(
        result.complete_result[0].pattern,
        world
            .semantic_world()
            .value(uint8_type)
            .expect("uint8 type object")
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
        .expect("production invocation registers the callable value");
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
            Provenance::new("type-value-not-carrier regression"),
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
    let lang_build::OrdinaryReturnedValue::CompleteType(value) = result.returned else {
        panic!("IdentityType returns the evaluated complete type value");
    };
    let uint8_type = world
        .semantic_world()
        .type_object_value_for_symbol(uint8.identity)
        .expect("uint8 type object value");
    let represented = value.complete_type.lookup_key;
    let SemanticValuePayload::TypeObject {
        represented_type, ..
    } = world
        .semantic_world()
        .value(uint8_type)
        .expect("uint8 TypeObject value")
        .payload
    else {
        panic!("uint8 carries a TypeObject");
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
        inner.sibling_vals.is_empty(),
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
fn bare_call_target_does_not_fall_through_after_a_rejects_nearest_symbol() {
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
        .symbol_in_namespace(inner_namespace, "g")
        .expect("inner runtime-only callable g")
        .identity;

    let initializer = initializer_from_source("let result = uint8 g;");
    let call_site = extract_single_call_site(&initializer).expect("bare g call");
    assert_eq!(
        world.resolve_source_terminal_symbol(inner_namespace, &call_site.target),
        Some(inner),
        "lexical resolution seals inner.g before call projection"
    );
    let failure = world
        .invoke_ordinary_call(
            inner_namespace,
            &call_site,
            OrdinaryInvocationContext::open_static(&[PolicyMode::Plain]),
            Provenance::new("nearest callable fails A without outward retry"),
        )
        .expect_err("runtime-only inner.g is inadmissible at OpenStatic");
    assert!(
        matches!(
            failure,
            lang_build::OrdinaryInvocationFailure::NoFullyAdmissibleCandidate { .. }
        ),
        "A-stage failure belongs to inner.g and never retries outer.g: {failure:?}"
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
fn privileged_struct_enters_ordinary_overload_and_returns_complete_tau() {
    let mut world =
        CompilationWorld::from_manifest(&BuildManifest::new("app", vec!["app".to_string()]))
            .expect("core semantic world builds");
    let initializer = initializer_from_source("let T: type = (uint8 a) struct;");
    let call_site = extract_single_call_site(&initializer).expect("normalized struct call");
    let actual_mutability = [PolicyMode::Const];
    let result = world
        .invoke_ordinary_call(
            world.package_root_node(),
            &call_site,
            OrdinaryInvocationContext::open_static(&actual_mutability),
            Provenance::new("core struct ordinary-spine regression"),
        )
        .expect("privileged AST decoding is an ordinary call-entry body capability");
    let lang_build::InvocationResult::SemanticResult {
        value: lang_build::ProjectedInvocationOutcome::SingleMember(result),
        ..
    } = result
    else {
        panic!("struct has the unified CompleteType result class");
    };

    // No overload bypass: the caller package differs from the toolchain
    // package, so this call goes through the external member view, and
    // `struct` is still selected through the normal
    // C0 -> C1 -> C2 -> Cc -> C3 -> A -> Bp' -> B3 spine.
    assert_eq!(result.trace.c0_target_values.len(), 1);
    assert_eq!(
        result.trace.c1_visible_values, result.trace.c0_target_values,
        "core struct is a declaration-boundary public export; External C1 keeps it"
    );
    assert_eq!(result.trace.c3_call_entries.len(), 1);
    assert!(
        result.trace.selected.is_some(),
        "privilege applies only to the selected body, after ordinary selection"
    );

    // The selected result is a complete tau value.  Replayable construction
    // material may remain attached for namespace projection, but it is not
    // the semantic result authority.  `struct` is a
    // builtin privileged meta function: it never creates a
    // `MetaInstance(struct, arguments)` scope of its own, so the generated
    // Pattern owner is the ambient declaration environment (the caller's
    // package root), not a MetaInstance.
    let lang_build::OrdinaryReturnedValue::CompleteType(returned) = &result.returned else {
        panic!("world-connected struct must return complete tau, not private meta material");
    };
    assert_eq!(
        result.complete_type.as_ref(),
        Some(&returned.complete_type),
        "every CompleteType semantic success carries its exact whole tau explicitly"
    );
    let view = &result.complete_result[0];
    assert!(
        view.value.is_some(),
        "complete tau is an ordinary first-class semantic value"
    );
    let owner = world
        .semantic_world()
        .pattern_owner(returned.pattern)
        .expect("generated type Pattern has a resolved owner")
        .owner;
    let ambient_owner = world
        .semantic_world()
        .namespace_owner(world.package_root_node())
        .expect("package root has a semantic owner");
    assert_eq!(
        owner, ambient_owner,
        "direct `struct` attaches its generated type to the ambient declaration environment"
    );
    assert_eq!(
        returned.complete_type.lookup_key,
        world
            .semantic_world()
            .type_for_pattern(returned.pattern)
            .expect("returned tau core has a registered Pattern"),
    );
    assert_eq!(
        world
            .semantic_world()
            .complete_type_by_whole_observation(returned.complete_type.whole),
        Some(&returned.complete_type),
        "the ordinary result carries the interned whole-snapshot observation"
    );
    assert!(matches!(
        world
            .semantic_world()
            .owners()
            .node(owner)
            .expect("generated Pattern owner exists")
            .kind,
        SemanticOwnerKind::PackageRoot { .. }
    ));
}

#[test]
fn return_shape_is_a_declaration_boundary_fact_shared_by_core_and_source() {
    // S4 — `CallableSemantics = P1 × P2 × ReturnShape ×
    // Privilege`.  The return shape and the privilege are two independent
    // declared coordinates: source callables spell the shape on the
    // return-slot annotation (`-> r: symbol` declares a ClusterSymbol
    // return), built-ins state both per primitive declaration.  Each is
    // elaborated once at the declaration boundary
    // (`declared_return_shape_from_closure`) and stored in the
    // MetaFunctionObject payload; no coordinate is projected from another.
    // The `s4_return_ontology` fixture declares `-> r: symbol` and
    // ordinary-slot source callables and performs no build-time struct
    // invocation, so it builds before the S6 meta-binding hookup lands.
    let world = build_single_fixture_world("s4_return_ontology", "app");

    let coordinates_of = |name: &str| {
        let symbol = world.resolve(name).expect("declared callable resolves");
        let SymbolPayload::MetaFunction(function) = symbol.payload else {
            panic!("`{name}` is a callable declaration");
        };
        (function.return_shape, function.privilege)
    };

    // Source-declared `-> r: symbol` return slot => ClusterSymbol shape;
    // a constrained ordinary return slot (`-> let result: uint8`) =>
    // SingleVal(Constrained).  Neither the body form family nor the
    // Policy stage is consulted, and the source surface can never spell
    // the privileged coordinate.
    assert_eq!(
        coordinates_of("make_type"),
        (
            lang_build::ReturnShape::ClusterSymbol,
            lang_build::CallablePrivilege::OrdinarySource,
        ),
        "source-defined `-> r: symbol` callable declares a ClusterSymbol return"
    );
    assert_eq!(
        coordinates_of("keep"),
        (
            lang_build::ReturnShape::SingleVal(lang_build::PatternConstraint::Constrained),
            lang_build::CallablePrivilege::OrdinarySource,
        ),
        "constrained-slot source callable declares a single-value return"
    );

    // Built-ins use the same ontology, declared per primitive at the core
    // declaration boundary: `struct` returns one complete type; `assert`
    // returns a single ordinary value even though it executes at meta
    // stage — privilege and shape are independent coordinates, so a
    // privileged built-in is NOT forced into any particular shape.
    assert_eq!(
        coordinates_of("struct"),
        (
            lang_build::ReturnShape::SingleType,
            lang_build::CallablePrivilege::BuiltinPrivileged,
        ),
        "core struct declares a privileged complete-type return"
    );
    assert_eq!(
        coordinates_of("assert"),
        (
            lang_build::ReturnShape::SingleVal(lang_build::PatternConstraint::Unconstrained),
            lang_build::CallablePrivilege::BuiltinPrivileged,
        ),
        "core assert declares a privileged single-value return"
    );
}

#[test]
fn return_slot_annotation_declares_shape_independent_of_body_form() {
    // The return shape is a declared fact, never an inference from the
    // body: a `-> r: symbol` callable with zero member events is still a
    // cluster construction, and a body full of member-event-shaped forms
    // cannot flip a `-> let r: type` slot away from SingleType.
    let world = build_single_fixture_world("s4_return_ontology", "app");

    let shape_of = |name: &str| {
        let symbol = world.resolve(name).expect("declared callable resolves");
        let SymbolPayload::MetaFunction(function) = symbol.payload else {
            panic!("`{name}` is a callable declaration");
        };
        function.return_shape
    };

    // Zero member events, `-> r: symbol`: still a cluster construction.
    assert_eq!(
        shape_of("empty_cluster"),
        lang_build::ReturnShape::ClusterSymbol,
        "a `-> r: symbol` callable with an effect-free body is still a cluster construction"
    );
    // Same body shape as the old `forward_type` (`let r = t; r;`) but a
    // `-> let r: type` slot: body refactoring never changes the shape,
    // and a meta P2 with a SingleType shape is a legal declaration
    // (`Validate(P2, Shape)` accepts it — one position, pure-P type).
    assert_eq!(
        shape_of("refactor_kept"),
        lang_build::ReturnShape::SingleType,
        "member-event-shaped body forms cannot flip a `-> let r: type` slot away from SingleType"
    );
}

#[test]
fn alias_expression_spelling_cannot_restore_retired_forwarding_semantics() {
    // Alias syntax remains normalized input, but neither expression nor
    // declaration spelling may create a semantic forwarding identity.
    let mut world = build_single_fixture_world("s4_return_ontology", "app");
    for (spelling, body_path) in [
        ("let R: type = uint8 bare_alias;", "ordinary body"),
        (
            "let R2: type = uint8 bare_alias_meta;",
            "meta construction body",
        ),
    ] {
        let initializer = initializer_from_source(spelling);
        let call_site = extract_single_call_site(&initializer).expect("normalized call");
        let result = world.invoke_ordinary_call(
            world.package_root_node(),
            &call_site,
            OrdinaryInvocationContext::open_static(&[PolicyMode::Const]),
            Provenance::new("bare alias convergence regression"),
        );
        let Err(lang_build::OrdinaryInvocationFailure::SelectedBody { failure, .. }) = result
        else {
            panic!("{body_path}: bare `r === X;` must be a hard body error, got: {result:?}");
        };
        assert!(
            failure
                .diagnostic
                .message
                .contains("lexical alias resolution is not implemented")
                && failure
                    .diagnostic
                    .message
                    .contains("must not create or forward a semantic entity"),
            "{body_path}: the diagnostic must preserve the lexical-only boundary without restoring forwarding, got: {}",
            failure.diagnostic.message,
        );
        assert_eq!(
            failure.diagnostic.code,
            Some(lang_build::ResolverCode::UnsupportedLexicalAlias)
        );
    }
}

#[test]
fn source_meta_body_contribution_stream_returns_cluster_construction() {
    // S5 — clustered return construction.  A source-defined meta body is
    // not a "single value wrapper": the body evaluator yields a stream of
    // construction effects and the invocation pipeline contributes each
    // one to an open cluster, then finalizes one SymbolConstructionValue.
    // The construction-effect family is distinct: `let r = expr;` adds a
    // fresh member, `r = expr;` writes to an existing target (currently a
    // placeholder overwrite scaffold), and the bare `r;` terminal delivers
    // the cluster — it is not a member event. Alias declarations never enter
    // this effect stream.
    //
    // v0.9 pattern head identity: the cluster's unique type member is
    // navigated as the meta function itself plus its input arguments, so
    // the member carries a fresh MetaInstance-owned PatternValue built by
    // the body's own self-rooted `struct` construction; the argument
    // uint8 keeps its own PatternValue and owner (no reroot).
    let callable = "make_type";
    let mut world = build_single_fixture_world("s4_return_ontology", "app");
    let initializer = initializer_from_source("let R: type = uint8 make_type;");
    let call_site = extract_single_call_site(&initializer).expect("normalized meta call");
    let actual_mutability = [PolicyMode::Const];
    let result = world
        .invoke_ordinary_call(
            world.package_root_node(),
            &call_site,
            OrdinaryInvocationContext::open_static(&actual_mutability),
            Provenance::new("S5 contribution stream regression"),
        )
        .unwrap_or_else(|failure| {
            panic!("{callable}: source meta callable is selected through the ordinary spine: {failure:?}")
        });
    let lang_build::InvocationResult::SemanticResult {
        declared_result_class: lang_build::DeclaredResultClass::ClusterSymbol,
        value: lang_build::ProjectedInvocationOutcome::ClusterSymbol(meta),
    } = result
    else {
        panic!("{callable}: meta-declared source callable returns a cluster construction");
    };

    assert_eq!(
        meta.construction.member_views.len(),
        1,
        "{callable}: one construction effect produces one member view"
    );
    let view = &meta.construction.member_views[0];
    assert!(
        view.value.is_none(),
        "{callable}: the constructed type contribution is a pure-P member (value=None)"
    );
    let uint8 = world
        .semantic_world()
        .symbol_in_namespace(world.core_node(), "uint8")
        .expect("core uint8");
    let uint8_pattern = uint8.pure_p_pattern().expect("uint8 pure-P");
    assert_ne!(
        view.pattern, uint8_pattern,
        "{callable}: the cluster's type member is a fresh meta-instance PatternValue, not the argument's uint8 pattern"
    );

    // The type member's top pattern is navigated as the meta function
    // itself plus its input arguments: its owner is a MetaInstance.
    let owner = world
        .semantic_world()
        .pattern_owner(view.pattern)
        .expect("member pattern has a resolved owner")
        .owner;
    assert!(
        matches!(
            world
                .semantic_world()
                .owners()
                .node(owner)
                .expect("member pattern owner exists")
                .kind,
            SemanticOwnerKind::MetaInstance { .. }
        ),
        "{callable}: the type member's top pattern is owned by the meta instance (meta function + input arguments)"
    );

    // No Pattern reroot: the argument uint8 keeps its original owner.
    let uint8_owner = world
        .semantic_world()
        .pattern_owner(uint8_pattern)
        .expect("uint8 Pattern keeps a resolved owner")
        .owner;
    assert!(
        !matches!(
            world
                .semantic_world()
                .owners()
                .node(uint8_owner)
                .expect("uint8 Pattern owner exists")
                .kind,
            SemanticOwnerKind::MetaInstance { .. }
        ),
        "{callable}: construction must not reroot uint8's PatternValue under a MetaInstance"
    );
}
