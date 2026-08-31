mod support;

use lang_build::{
    extract_single_call_site, ArgProductShape, BuildManifest, CompilationWorld,
    FlattenedProductInvariant, FlattenedProductObject, OrdinaryInvocationContext,
    OrdinaryInvocationFailure, P1Projection, PatternComponentPolicy, PolicyMigrationRequest,
    PolicyMode, PolicyPair, PolicyStage, PolicyView, Provenance, ResolveExpectation,
    ResultPolicyDemand, SemanticValuePayload, SourceRoot, StageSet, SymbolPayload,
    ToolchainGlobalSourceRoot, ValueComponentPolicy, ValuePresence,
};

use support::{fixture_root, fixture_source_root, initializer_from_source};

fn global_bundle() -> ToolchainGlobalSourceRoot {
    ToolchainGlobalSourceRoot::new(fixture_root().join("global_implementation").join("basic"))
}

fn transport_bundle() -> ToolchainGlobalSourceRoot {
    ToolchainGlobalSourceRoot::under(
        fixture_root()
            .join("global_implementation")
            .join("uint8_transport"),
        vec!["core".to_string(), "uint8".to_string()],
    )
}

fn compile_identity_bundle() -> ToolchainGlobalSourceRoot {
    ToolchainGlobalSourceRoot::new(
        fixture_root()
            .join("global_implementation")
            .join("compile_identity"),
    )
}

fn type_transport_bundle() -> ToolchainGlobalSourceRoot {
    ToolchainGlobalSourceRoot::under(
        fixture_root()
            .join("global_implementation")
            .join("type_transport"),
        vec!["core".to_string(), "type".to_string()],
    )
}

fn wrong_type_transport_bundle() -> ToolchainGlobalSourceRoot {
    ToolchainGlobalSourceRoot::under(
        fixture_root()
            .join("global_implementation")
            .join("wrong_type_transport"),
        vec!["core".to_string(), "uint8".to_string()],
    )
}

fn stages(items: &[PolicyStage]) -> StageSet {
    let mut stages = StageSet::new();
    for stage in items {
        stages.insert(*stage);
    }
    stages
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

#[test]
fn toolchain_global_source_is_parsed_installed_and_invoked_through_ordinary_spine() {
    let mut manifest = BuildManifest::single_source_root(
        "app",
        vec!["app".to_string()],
        fixture_source_root("gsrc_ordinary_call", "app"),
    );
    manifest.global_implementation_roots.push(global_bundle());

    let mut world = CompilationWorld::from_manifest(&manifest)
        .expect("toolchain global source and user source build");
    let root = world.namespace_projection().root_node();
    let global = world
        .semantic_world()
        .symbol_in_namespace(root, "global_identity")
        .expect("global implementation is a real semantic Symbol at `::`");
    assert_eq!(
        global.declaration_owner,
        world.semantic_world().toolchain_owner()
    );
    assert_eq!(global.sibling_vals.len(), 1);

    let result = world
        .resolve_with_expectation("Result", ResolveExpectation::CoreTypeProjection)
        .expect("user initializer invoked explicit global implementation");
    assert_eq!(
        result.generation_origin, None,
        "ordinary result binding has no graph-generation origin"
    );
    let result_semantic = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "Result")
        .expect("Result semantic Symbol");
    // The fixture computes `Result` from `local global_identity::` where
    // `local` is bound to core `uint8`; the identity body preserves that
    // TypeValue identity (core has no `int` bootstrap type).
    let uint8_semantic = world
        .semantic_world()
        .symbol_in_namespace(world.core_node(), "uint8")
        .expect("core uint8 semantic Symbol");
    assert_eq!(
        result_semantic.pure_p_pattern(),
        uint8_semantic.pure_p_pattern(),
        "ordinary forwarding plus let binding preserves the returned TypeValue identity"
    );

    let bare_initializer = initializer_from_source("let x = local global_identity;");
    let bare_call = extract_single_call_site(&bare_initializer).expect("normalized bare-name call");
    let actual_mutability = [PolicyMode::Const];
    assert!(matches!(
        world.invoke_ordinary_call(
            world.package_root_node(),
            &bare_call,
            OrdinaryInvocationContext::open_static(&actual_mutability),
            Provenance::new("Gsrc is not a prelude"),
        ),
        Err(OrdinaryInvocationFailure::NoTargetValues { .. })
    ));
}

#[test]
fn ordinary_source_root_cannot_gain_global_construction_authority_from_empty_prefix() {
    // The manifest itself carries a legal non-empty install prefix; only the
    // source root tries to claim `::` directly, isolating the source-root
    // authority check from the manifest-prefix check.
    let mut manifest = BuildManifest::new("app", vec!["app".to_string()]);
    manifest.source_roots.push(SourceRoot {
        path: fixture_source_root("gsrc_ordinary_call", "app"),
        namespace_root: Vec::new(),
    });
    let error = CompilationWorld::from_manifest(&manifest)
        .expect_err("ordinary project cannot install direct members into `::`");
    assert!(error.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("only ToolchainGlobalSourceRoot carries global construction authority")));
}

#[test]
fn sourceless_ordinary_manifest_still_cannot_claim_the_global_root_owner() {
    let manifest = BuildManifest::new("app", Vec::new());
    let error = CompilationWorld::from_manifest(&manifest)
        .expect_err("absence of package source does not grant ownership of `::`");
    assert!(error.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("ordinary project requires a non-empty namespace install prefix")));
}

#[test]
fn ordinary_package_boundary_cannot_overlap_toolchain_namespace_owner() {
    let manifest = BuildManifest::new("malicious", vec!["core".to_string()]);
    let error = CompilationWorld::from_manifest(&manifest)
        .expect_err("ordinary package cannot claim the toolchain-owned core namespace");
    assert!(error.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("ordinary package boundary overlaps a toolchain-owned namespace")));
}

#[test]
fn global_source_cannot_enter_a_package_owned_namespace_boundary() {
    let mut manifest = BuildManifest::new("app", vec!["app".to_string()]);
    manifest
        .global_implementation_roots
        .push(ToolchainGlobalSourceRoot::under(
            fixture_root().join("global_implementation").join("basic"),
            vec!["app".to_string()],
        ));
    let error = CompilationWorld::from_manifest(&manifest)
        .expect_err("global construction input cannot borrow package ownership");
    assert!(error.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("cannot contribute through a package-owned namespace boundary")));
}

#[test]
fn source_backed_transport_family_uses_pattern_owner_and_ordinary_spine() {
    let mut manifest = BuildManifest::new("app", vec!["app".to_string()]);
    manifest
        .global_implementation_roots
        .push(transport_bundle());
    let mut world =
        CompilationWorld::from_manifest(&manifest).expect("source-backed transport bundle builds");

    let uint8 = world
        .resolve_with_expectation("uint8", ResolveExpectation::CoreTypeProjection)
        .expect("core uint8 type");
    let SymbolPayload::CompleteTypeProjection(uint8_type) = uint8.payload else {
        panic!("uint8 resolves as a CompleteType projection");
    };
    let type_value = uint8_type.represented_type;
    let pattern = world
        .semantic_world()
        .type_value(type_value)
        .expect("uint8 has a semantic TypeValue")
        .pattern;
    let uint8_cluster = world
        .semantic_world()
        .symbol_in_namespace(world.core_node(), "uint8")
        .expect("core uint8 cluster symbol");
    // Exactly 5 sibling vals (const/mut transport endpoints plus the real
    // plain-input member).
    // `identity` and `type_identity` must NOT be cluster siblings; they are
    // registered as ordinary source callables under their own Val2 names.
    assert_eq!(
        uint8_cluster.sibling_vals.len(),
        5,
        "exactly 5 transports named `uint8` are cluster sibling vals: got {} siblings",
        uint8_cluster.sibling_vals.len()
    );
    // Transports must NOT live in uint8's associated Val2["()"].
    assert!(
        world
            .semantic_world()
            .associated_values_for_pattern(pattern, "()")
            .map(|vals| vals.is_empty())
            .unwrap_or(true),
        "pure-P.Val2['()'] must not contain transport function objects"
    );

    // `identity` and `type_identity` must NOT be cluster siblings — they
    // are registered as ordinary source callables with their own Val2[name]
    // entries.  This verifies the explicit ContributeSiblingVal boundary:
    // only declarations whose binder name matches an existing cluster Symbol
    // contribute as siblings.
    let identity_pattern = world
        .semantic_world()
        .pattern_for_associated_namespace(world.core_node());
    if let Some(id_pattern) = identity_pattern {
        let identity_vals = world
            .semantic_world()
            .associated_values_for_pattern(id_pattern, "identity")
            .map(|vals| vals.len())
            .unwrap_or(0);
        assert_eq!(
            identity_vals, 1,
            "`identity` is an ordinary source callable in Val2[\"identity\"], not a cluster sibling"
        );
        let type_identity_vals = world
            .semantic_world()
            .associated_values_for_pattern(id_pattern, "type_identity")
            .map(|vals| vals.len())
            .unwrap_or(0);
        assert_eq!(
            type_identity_vals, 1,
            "`type_identity` is an ordinary source callable in Val2[\"type_identity\"], not a cluster sibling"
        );
    }

    // Test D — each transport FunctionObject has its own Val2["()"]
    // containing a terminal call entry.  This verifies the transport
    // hierarchy: transports are independent FunctionObjects, each with
    // its own call entry, not shared entries on the cluster.
    for sibling in &uint8_cluster.sibling_vals {
        let transport_obj = world
            .semantic_world()
            .value(*sibling)
            .expect("transport sibling value exists");
        assert!(
            matches!(
                transport_obj.payload,
                SemanticValuePayload::FunctionObject { .. }
            ),
            "Test D: each transport is a FunctionObject"
        );
        let transport_entries = world
            .semantic_world()
            .associated_values_for_value(*sibling, "()")
            .map(|vals| vals.len())
            .unwrap_or(0);
        assert_eq!(
            transport_entries, 1,
            "Test D: each transport has exactly one Val2['()'] call entry"
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
    let source = world
        .install_semantic_value(
            type_value,
            source_policy.clone(),
            Provenance::new("compile uint8 fixture value"),
        )
        .expect("installed value reuses uint8 PatternValue");
    let request = PolicyMigrationRequest::new(
        PolicyView {
            pair: source_policy,
            mode: PolicyMode::Const,
        },
        ResultPolicyDemand {
            pair_query: P1Projection::Pair(target_policy.clone()),
            mode: PolicyMode::Mut,
        },
        type_value,
        source,
        Provenance::new("const compile -> mut runtime demand"),
    )
    .expect("legal migration request");

    let migration = world
        .invoke_policy_migration(&request)
        .expect("ordinary source-backed transport is selected and invoked");
    assert_eq!(migration.invocation.trace.a_fully_admissible.len(), 5);
    assert_eq!(migration.invocation.trace.bp_prime.len(), 1);
    // Test F — A and Bp' consume the same prepared migration endpoint
    // coordinates.  The selected candidate stores endpoints computed
    // once at A-stage; Bp' reads them via bp_prime_dominates, not by
    // re-deriving from function_object_p1.
    assert!(
        migration
            .invocation
            .selected
            .migration_input_endpoint
            .is_some(),
        "Test F: selected candidate stores input endpoint from A-stage"
    );
    assert!(
        migration
            .invocation
            .selected
            .migration_output_endpoint
            .is_some(),
        "Test F: selected candidate stores output endpoint from A-stage"
    );
    assert_eq!(migration.invocation.complete_result.len(), 1);
    assert_eq!(
        migration.invocation.complete_result[0]
            .view
            .pair
            .value
            .stages,
        stages(&[PolicyStage::Compile, PolicyStage::Runtime]),
        "ordinary invocation retains its complete P2 before Project_out"
    );
    assert_eq!(migration.demanded_view.len(), 1);
    assert_eq!(
        migration.demanded_view[0].view.pair.value,
        target_policy.value
    );
    assert_eq!(
        migration.demanded_view[0]
            .value
            .expect("runtime result value")
            .id,
        source,
        "migration preserves the identity semantics of the selected ordinary operation; a forwarding transport body returns the existing source value"
    );

    let no_mutability = [];
    let named = world
        .invoke_pattern_associated_operation(
            pattern,
            "identity",
            source,
            ArgProductShape::from_flattened(FlattenedProductObject {
                atoms: Vec::new(),
                provenance: Provenance::new("named associated identity args"),
                invariant: FlattenedProductInvariant {
                    no_direct_product_atom_remains: true,
                },
            }),
            OrdinaryInvocationContext::open_static(&no_mutability),
            Provenance::new("named associated identity"),
        )
        .expect("named associated Val2 function uses the ordinary function-object trunk");
    let lang_build::InvocationResult::SemanticResult {
        value: lang_build::ProjectedInvocationOutcome::SingleMember(named),
        ..
    } = named
    else {
        panic!("named associated value returns ordinary result");
    };
    assert!(matches!(
        named.returned,
        lang_build::ReturnedSemanticEntity::OrdinaryValue(value)
            if value == source
    ));
    assert_eq!(
        named.complete_result[0]
            .value
            .expect("identity result carries Val1")
            .id,
        source,
        "an ordinary identity body returns the existing value without inventing a wrapper value"
    );
    assert_eq!(named.complete_result[0].pattern, pattern);
    assert_eq!(named.trace.c0_target_values.len(), 1);
    assert_eq!(named.trace.c3_call_entries.len(), 1);
}

#[test]
fn source_binding_p1_uses_existing_projection_then_connected_ordinary_migration() {
    let mut manifest = BuildManifest::single_source_root(
        "app",
        vec!["app".to_string()],
        fixture_source_root("gsrc_binding_migration", "app"),
    );
    manifest
        .global_implementation_roots
        .push(compile_identity_bundle());
    manifest
        .global_implementation_roots
        .push(type_transport_bundle());
    manifest
        .global_implementation_roots
        .push(transport_bundle());

    let world = CompilationWorld::from_manifest(&manifest)
        .expect("source P1 should use the connected ordinary migration trunk");
    let existing = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "existing")
        .expect("existing compile slice is installed");
    assert_eq!(existing.member_views.len(), 1);
    let existing_view = &existing.member_views[0];
    assert_eq!(
        existing_view.view.pair.value.stages,
        stages(&[PolicyStage::Compile])
    );
    assert!(
        existing_view.value.is_none(),
        "compile type binding has value=None (pure-P result)"
    );
    let uint8 = world
        .semantic_world()
        .symbol_in_namespace(world.core_node(), "uint8")
        .expect("core uint8 type value");
    let existing_type = world
        .semantic_world()
        .core_type_projection_value_for_symbol(existing.identity)
        .expect("existing pure type Object value");
    let uint8_type = world
        .semantic_world()
        .core_type_projection_value_for_symbol(uint8.identity)
        .expect("uint8 pure type Object value");
    assert_eq!(
        existing_type, uint8_type,
        "the forwarding call returns the existing TypeValue and let binds that value"
    );

    let forwarded = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "forwarded")
        .expect("ordinary value argument call result is installed");
    let forwarded_view = forwarded
        .member_views
        .first()
        .expect("forwarded binding has one view");
    assert!(
        forwarded_view.value.is_none(),
        "forwarded type binding has value=None"
    );
    assert_eq!(
        forwarded.pure_p_pattern(),
        existing.pure_p_pattern(),
        "ordinary body forwarding preserves its semantic type identity"
    );
    assert_eq!(forwarded_view.pattern, existing_view.pattern);
    let forwarded_graph = world
        .resolve_with_expectation("forwarded", ResolveExpectation::CoreTypeProjection)
        .expect("`: type` checks the evaluated ordinary result value");
    let SymbolPayload::CompleteTypeProjection(forwarded_type) = forwarded_graph.payload else {
        panic!("forwarded type value receives a fresh LHS graph carrier");
    };
    assert_eq!(
        forwarded_type.represented_type,
        world
            .semantic_world()
            .core_type_projection_value_for_symbol(existing.identity)
            .and_then(|id| {
                let value = world.semantic_world().value(id)?;
                match value.payload {
                    SemanticValuePayload::CoreTypeProjection {
                        represented_type, ..
                    } => Some(represented_type),
                    _ => None,
                }
            })
            .expect("existing pure type Object value"),
        "forwarded type binding preserves the represented type"
    );
    assert_ne!(
        forwarded_type.carrier_symbol_id,
        world
            .resolve_with_expectation("existing", ResolveExpectation::CoreTypeProjection)
            .expect("existing carrier")
            .id,
        "type annotation does not turn ordinary `=` into Symbol forwarding"
    );

    let via_associated = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "via_associated")
        .expect("associated source navigation through the bound PatternValue succeeds");
    assert_eq!(
        via_associated.pure_p_pattern(), existing.pure_p_pattern(),
        "`type_identity::existing` follows existing -> TypeValue/PatternValue -> Pattern owner -> associated Symbol and returns the same value"
    );

    let binding = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "materialized")
        .expect("runtime binding is installed as a semantic Symbol");
    assert_eq!(binding.sibling_vals.len(), 1);
    assert_eq!(binding.member_views.len(), 1);
    let view = &binding.member_views[0];
    assert_eq!(view.view.pair.value.stages, stages(&[PolicyStage::Runtime]));
    assert_eq!(view.view.mode, PolicyMode::Mut);
    assert_eq!(
        view.view.pair.pattern.stages,
        stages(&[PolicyStage::Compile])
    );

    let result_id = view.value.expect("runtime binding carries Val1");
    let result = world
        .semantic_world()
        .value(result_id)
        .expect("bound migrated value exists");
    let migration_source = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "migration_source")
        .and_then(|symbol| symbol.member_views.first())
        .and_then(|view| view.value)
        .expect("compile migration source carries Val1");
    assert_eq!(
        result_id, migration_source,
        "migration exposes the selected body's coherent realization directly instead of allocating an InvocationResult wrapper"
    );
    assert!(matches!(
        result.payload,
        SemanticValuePayload::ConstructedLiteral { .. }
    ));
    assert_eq!(
        world
            .semantic_world()
            .value(migration_source)
            .expect("migration source exists")
            .pattern,
        result.pattern
    );

    let canonical_source = world
        .semantic_world()
        .value(result_id)
        .expect("migration result is an ordinary canonicalizable value");
    assert!(!matches!(
        canonical_source.payload,
        SemanticValuePayload::CallEntry(_)
    ));

    let rebound = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "rebound")
        .expect("ordinary name binding is installed");
    assert_eq!(
        rebound.sibling_vals, binding.sibling_vals,
        "`let LHS = RHS` associates the RHS value with the LHS Symbol without allocating a binding value"
    );
    assert_eq!(
        rebound.member_views, binding.member_views,
        "the identity-preserving binding also retains the selected Policy/Pattern views"
    );
    let binding_place = binding
        .sibling_place(result_id)
        .expect("the first ordinary binding has a destination Place");
    let rebound_place = rebound
        .sibling_place(result_id)
        .expect("rebinding the value establishes another destination Place");
    assert_ne!(
        binding_place, rebound_place,
        "equal resident values do not collapse distinct ordinary binding Places"
    );
    let mut writable = lang_build::WritableContext::default();
    writable.grant_place(binding_place);
    assert!(writable.place_is_writable(binding_place));
    assert!(
        !writable.place_is_writable(rebound_place),
        "Writable authority for one binding cannot leak through shared SemanticValue identity"
    );
    let binding_resident = world
        .semantic_world()
        .resident_generation(binding_place)
        .expect("binding resident exists");
    let rebound_resident = world
        .semantic_world()
        .resident_generation(rebound_place)
        .expect("rebound resident exists");
    assert_ne!(
        binding_resident, rebound_resident,
        "fresh destinations own independent ProjectionSlot/borrow generations"
    );
}

#[test]
fn binding_demand_reaches_rhs_maxima_and_output_preference_reads_result_p2_mode() {
    let mut world = CompilationWorld::from_manifest(&BuildManifest::single_source_root(
        "app",
        vec!["app".to_string()],
        fixture_source_root("binding_result_demand", "app"),
    ))
    .expect("the written mut binding demand must disambiguate its RHS producer before selection");
    assert!(
        world
            .semantic_world()
            .symbol_in_namespace(world.package_root_node(), "Selected")
            .is_some(),
        "without pre-maxima binding demand the crossed const/mut producers are ambiguous"
    );

    let initializer = initializer_from_source("let x = uint8 choose;");
    let call = extract_single_call_site(&initializer).expect("fixture call normalizes");
    let actual_modes = [PolicyMode::Plain];
    let invocation = world
        .invoke_ordinary_call(
            world.package_root_node(),
            &call,
            OrdinaryInvocationContext::open_static(&actual_modes).with_result_policy_demand(
                ResultPolicyDemand {
                    pair_query: P1Projection::Infer,
                    mode: PolicyMode::Mut,
                },
            ),
            Provenance::new("function-object and result mode coordinates"),
        )
        .expect("mut result demand chooses one producer");
    let lang_build::InvocationResult::SemanticResult {
        value: lang_build::ProjectedInvocationOutcome::SingleMember(invocation),
        ..
    } = invocation
    else {
        panic!("ordinary value call has one member result");
    };
    assert_eq!(
        invocation.selected.complete_result_view.mode,
        PolicyMode::Mut,
        "Bp compares the producer's concrete result P2 mode"
    );
    assert_eq!(
        invocation.selected.function_object_view.mode,
        PolicyMode::Const,
        "the selected producer deliberately has the opposite P1 mode; P1 cannot create output preference"
    );
}

#[test]
fn type_changing_migration_candidate_is_excluded_in_a_before_preference() {
    let mut manifest = BuildManifest::new("app", vec!["app".to_string()]);
    manifest
        .global_implementation_roots
        .push(wrong_type_transport_bundle());
    let mut world = CompilationWorld::from_manifest(&manifest)
        .expect("dedicated migration family is installed");
    let uint8_type = world.resolve_type_value("uint8").expect("uint8 Type");
    let source_view = PolicyView {
        pair: pair(&[PolicyStage::Compile], &[PolicyStage::Compile], &[]),
        mode: PolicyMode::Const,
    };
    let source = world
        .install_semantic_value(
            uint8_type,
            source_view.pair.clone(),
            Provenance::new("same-Type A-stage source"),
        )
        .expect("source value");
    let request = PolicyMigrationRequest::new(
        source_view,
        ResultPolicyDemand {
            pair_query: P1Projection::Pair(pair(
                &[PolicyStage::Runtime],
                &[PolicyStage::Compile],
                &[],
            )),
            mode: PolicyMode::Mut,
        },
        uint8_type,
        source,
        Provenance::new("wrong declared result Type must not enter A"),
    )
    .expect("migration request");
    let migration = world
        .invoke_policy_migration(&request)
        .expect("the less preferred same-Type candidate remains selectable");
    assert_eq!(
        migration.invocation.trace.a_fully_admissible.len(),
        1,
        "the structurally preferred uint16-result candidate is removed before Bp/maxima"
    );
    let selected = world
        .semantic_world()
        .value(migration.invocation.selected.call_entry_value)
        .expect("selected call entry");
    let SemanticValuePayload::CallEntry(entry) = &selected.payload else {
        panic!("migration selected an ordinary call entry");
    };
    let return_type_name = entry
        .closure
        .as_ref()
        .and_then(|closure| closure.head.as_ref())
        .and_then(|head| head.returns.as_ref())
        .and_then(|returns| returns.annotation.as_ref())
        .and_then(|annotation| match &annotation.pattern {
            lang_syntax::NormPattern::Name { name, .. } => Some(name.as_str()),
            _ => None,
        });
    assert_eq!(return_type_name, Some("uint8"));
}

#[test]
fn pure_p_policy_let_never_fabricates_a_val1_for_migration() {
    let mut manifest = BuildManifest::single_source_root(
        "app",
        vec!["app".to_string()],
        fixture_source_root("pure_p_policy_let_migration", "app"),
    );
    manifest
        .global_implementation_roots
        .push(compile_identity_bundle());
    manifest
        .global_implementation_roots
        .push(transport_bundle());
    let error = CompilationWorld::from_manifest(&manifest)
        .expect_err("absent Val1 is outside same-Type Policy migration");
    assert!(error.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("cannot migrate a pure-P result")
            && diagnostic
                .message
                .contains("authorized constructor/materializer")
    }));
}

#[test]
fn policy_let_forms_inward_mode_before_selection_and_returns_a_completed_view() {
    let mut manifest = BuildManifest::single_source_root(
        "app",
        vec!["app".to_string()],
        fixture_source_root("policy_let_boundary", "app"),
    );
    manifest
        .global_implementation_roots
        .push(compile_identity_bundle());
    manifest
        .global_implementation_roots
        .push(transport_bundle());

    let world = CompilationWorld::from_manifest(&manifest)
        .expect("PolicyLet resolves its operand once and completes the requested outward view");
    let completed = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "completed")
        .expect("completed PolicyLet result is bound as ordinary semantic material");
    let [view] = completed.member_views.as_slice() else {
        panic!("PolicyLet exposes exactly one completed result view");
    };
    assert_eq!(
        view.view.mode,
        PolicyMode::Mut,
        "the explicit inward/outward demand is the real mut Policy point"
    );
    let seed = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "seed")
        .and_then(|symbol| symbol.member_views.first())
        .and_then(|view| view.value)
        .expect("PolicyLet seed is a real value");
    let result = world
        .semantic_world()
        .value(view.value.expect("PolicyLet completed view carries Val1"))
        .expect("completed result is installed");
    assert_eq!(
        result.id, seed,
        "PolicyLet outward satisfaction exposes the selected body's coherent value realization"
    );
    assert!(matches!(
        result.payload,
        SemanticValuePayload::ConstructedLiteral { .. }
    ));

    let outer = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "outer")
        .expect("outer binding consumes the already completed PolicyLet view");
    let [outer_view] = outer.member_views.as_slice() else {
        panic!("outer binding has one completed view");
    };
    assert_eq!(outer_view.view.mode, PolicyMode::Const);
    assert_eq!(
        outer_view.value,
        Some(seed),
        "the outer consumer changes only its completed view and cannot replace or wrap the sealed inner realization"
    );
}

#[test]
fn literals_form_abstract_values_before_construction_and_same_type_migration() {
    let mut manifest = BuildManifest::single_source_root(
        "app",
        vec!["app".to_string()],
        fixture_source_root("abstract_literal_pipeline", "app"),
    );
    manifest
        .global_implementation_roots
        .push(transport_bundle());
    let world = CompilationWorld::from_manifest(&manifest)
        .expect("abstract literal, concrete construction, and migration remain separate");

    let integer_type = world
        .resolve_type_value("integer")
        .expect("abstract integer Type");
    let real_type = world
        .resolve_type_value("real")
        .expect("abstract real Type");
    let uint16_type = world
        .resolve_type_value("uint16")
        .expect("concrete uint16 Type");
    let uint8_type = world
        .resolve_type_value("uint8")
        .expect("concrete uint8 Type");

    let bound_value = |name: &str| {
        let symbol = world
            .semantic_world()
            .symbol_in_namespace(world.package_root_node(), name)
            .unwrap_or_else(|| panic!("binding `{name}` exists"));
        let id = symbol.member_views[0]
            .value
            .unwrap_or_else(|| panic!("binding `{name}` carries Val1"));
        world
            .semantic_world()
            .value(id)
            .expect("bound value exists")
    };

    let abstract_value = bound_value("abstract_value");
    assert_eq!(abstract_value.type_value, integer_type);
    assert!(matches!(
        abstract_value.payload,
        SemanticValuePayload::AbstractLiteral {
            family: lang_build::AbstractLiteralFamily::Integer,
            ..
        }
    ));
    let exact_real = bound_value("exact_real");
    assert_eq!(exact_real.type_value, real_type);

    let concrete = bound_value("concrete_value");
    assert_eq!(concrete.type_value, uint16_type);
    let SemanticValuePayload::ConstructedLiteral {
        source_abstract,
        target_complete_type,
        ..
    } = concrete.payload
    else {
        panic!("concrete annotation runs a later construction operation");
    };
    let original = world
        .semantic_world()
        .value(source_abstract)
        .expect("construction retains its abstract source value");
    assert_eq!(
        original.type_value, integer_type,
        "expected uint16 never rewrites the literal's initial integer Type"
    );
    assert_eq!(
        world
            .semantic_world()
            .complete_type_by_whole_observation(target_complete_type)
            .expect("the concrete result carries a registered complete Type")
            .lookup_key(),
        uint16_type
    );

    let runtime = bound_value("runtime_value");
    assert_eq!(runtime.type_value, uint8_type);
    assert!(matches!(
        runtime.payload,
        SemanticValuePayload::ConstructedLiteral { .. }
    ), "same-Type materialization preserves the concrete constructor's ordinary value realization instead of wrapping it");
}
