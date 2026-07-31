mod support;

use std::collections::BTreeSet;

use lang_build::{
    extract_single_call_site, ArgProductShape, BuildManifest, CompilationWorld,
    FlattenedProductInvariant, FlattenedProductObject, InvocationOutcome,
    OrdinaryInvocationContext, OrdinaryInvocationFailure, PatternComponentPolicy, PolicyPair,
    PolicyStage, PolicyTransitionRequest, Provenance, ResolveExpectation, SemanticValuePayload,
    SourceRoot, StageSet, SymbolPayload, ToolchainGlobalSourceRoot, ValueComponentPolicy,
    ValueMutability, ValuePresence,
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
    mutability: &[ValueMutability],
) -> PolicyPair {
    PolicyPair {
        value: ValueComponentPolicy {
            stages: stages(value_stages),
            mutability: mutability.iter().copied().collect(),
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
        .resolve_with_expectation("Result", ResolveExpectation::TypeObject)
        .expect("user initializer invoked explicit global implementation");
    assert_eq!(
        result.generation_origin, None,
        "ordinary result binding is not an alias/forwarding declaration"
    );
    let result_semantic = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "Result")
        .expect("Result semantic Symbol");
    // The fixture computes `Result` from `local global_identity::` where
    // `local` is bound to core `uint8`; the forwarding body preserves that
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
    let actual_mutability = [ValueMutability::Const];
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
fn source_backed_four_member_transport_uses_pattern_owner_and_ordinary_spine() {
    let mut manifest = BuildManifest::new("app", vec!["app".to_string()]);
    manifest
        .global_implementation_roots
        .push(transport_bundle());
    let mut world =
        CompilationWorld::from_manifest(&manifest).expect("source-backed transport bundle builds");

    let uint8 = world
        .resolve_with_expectation("uint8", ResolveExpectation::TypeObject)
        .expect("core uint8 type");
    let SymbolPayload::Type(uint8_type) = uint8.payload else {
        panic!("uint8 resolves as a Type object");
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
    // Exactly 4 sibling vals (the 4 transports named `uint8`).
    // `identity` and `type_identity` must NOT be cluster siblings; they are
    // registered as ordinary source callables under their own Val2 names.
    assert_eq!(
        uint8_cluster.sibling_vals.len(),
        4,
        "exactly 4 transports named `uint8` are cluster sibling vals: got {} siblings",
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
        &[ValueMutability::Const],
    );
    let target_policy = pair(
        &[PolicyStage::Runtime],
        &[PolicyStage::Compile],
        &[ValueMutability::Mut],
    );
    let source = world
        .install_semantic_value(
            type_value,
            source_policy.clone(),
            Provenance::new("compile uint8 fixture value"),
        )
        .expect("installed value reuses uint8 PatternValue");
    let request = PolicyTransitionRequest::new(
        source_policy,
        target_policy.clone(),
        type_value,
        source,
        Provenance::new("const compile -> mut runtime demand"),
    )
    .expect("legal atomic migration request");

    let migration = world
        .invoke_atomic_runtime_migration(&request)
        .expect("ordinary source-backed transport is selected and invoked");
    assert_eq!(migration.invocation.trace.a_fully_admissible.len(), 4);
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
        migration.invocation.complete_result[0].value_policy.stages,
        stages(&[PolicyStage::Compile, PolicyStage::Runtime]),
        "ordinary invocation retains its complete P2 before Project_out"
    );
    assert_eq!(migration.demanded_view.len(), 1);
    assert_eq!(migration.demanded_view[0].value_policy, target_policy.value);
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
    let InvocationOutcome::SingleMember(named) = named else {
        panic!("named associated value returns ordinary result");
    };
    assert!(matches!(
        named.returned,
        lang_build::OrdinaryReturnedValue::ForwardedSemanticValue(value)
            if value == source
    ));
    assert_eq!(
        named.complete_result[0]
            .value
            .expect("identity result carries Val1")
            .id,
        source,
        "an ordinary forwarding body returns the existing value; invocation does not invent a wrapper value"
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
        existing_view.value_policy.stages,
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
        .type_object_value_for_symbol(existing.identity)
        .expect("existing type object value");
    let uint8_type = world
        .semantic_world()
        .type_object_value_for_symbol(uint8.identity)
        .expect("uint8 type object value");
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
        .resolve_with_expectation("forwarded", ResolveExpectation::TypeObject)
        .expect("`: type` checks the evaluated ordinary result value");
    let SymbolPayload::Type(forwarded_type) = forwarded_graph.payload else {
        panic!("forwarded type value receives a fresh LHS graph carrier");
    };
    assert_eq!(
        forwarded_type.represented_type,
        world
            .semantic_world()
            .type_object_value_for_symbol(existing.identity)
            .and_then(|id| {
                let value = world.semantic_world().value(id)?;
                match value.payload {
                    SemanticValuePayload::TypeObject {
                        represented_type, ..
                    } => Some(represented_type),
                    _ => None,
                }
            })
            .expect("existing type object value"),
        "forwarded type binding preserves the represented type"
    );
    assert_ne!(
        forwarded_type.carrier_symbol_id,
        world
            .resolve_with_expectation("existing", ResolveExpectation::TypeObject)
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
    assert_eq!(view.value_policy.stages, stages(&[PolicyStage::Runtime]));
    assert_eq!(
        view.value_policy.mutability,
        BTreeSet::from([ValueMutability::Mut])
    );
    assert_eq!(view.pattern_policy.stages, stages(&[PolicyStage::Compile]));

    let result_id = view.value.expect("runtime binding carries Val1");
    let result = world
        .semantic_world()
        .value(result_id)
        .expect("bound invocation result exists");
    let SemanticValuePayload::InvocationResult {
        selected_call_entry,
        source_value: Some(source_value),
    } = result.payload
    else {
        panic!("runtime binding must expose the fresh ordinary migration result");
    };
    let source = world
        .semantic_world()
        .value(source_value)
        .expect("migration source ordinary result exists");
    assert_eq!(source.type_value, result.type_value);
    assert_eq!(source.pattern, result.pattern);

    let selected = world
        .semantic_world()
        .value(selected_call_entry)
        .expect("selected transport call entry exists");
    let SemanticValuePayload::CallEntry(entry) = &selected.payload else {
        panic!("migration winner must be an ordinary associated call entry");
    };
    assert_eq!(
        entry.callable_value_policy.value.mutability,
        BTreeSet::from([ValueMutability::Mut]),
        "Project_out mutability is owned by the ordinary member endpoint"
    );
    assert_eq!(
        entry.complete_result_policy.value.stages,
        stages(&[PolicyStage::Compile, PolicyStage::Runtime]),
        "the selected member retains complete ordinary P2 before Project_out"
    );

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
}
