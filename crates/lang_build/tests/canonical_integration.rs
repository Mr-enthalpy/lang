//! Canonical semantic-spine integration tests.
//!
//! These tests pin the canonical invariants end to end:
//! one resolved cluster Symbol per call target, member views as the canonical
//! fact (never a flat Symbol/Policy aggregate), declaration-time return
//! ontology shared by core and source, ordinary let-binding of meta outcomes
//! (bind the RHS value to the LHS symbol — types have value semantics too),
//! and the single canonical P1 authority chain.

mod support;

use lang_build::{
    extract_single_call_site, BuildManifest, CompilationWorld, ExecutionEnv, InvocationOutcome,
    OrdinaryInvocationContext, OrdinaryInvocationFailure, OrdinaryPipelineTrace,
    PatternClusterOwner, PatternComponentPolicy, Phase, PolicyEnv, PolicyMigrationRequest,
    PolicyMode, PolicyPair, PolicyStage, Provenance, ResolveExpectation, ResolverCode,
    SemanticOwnerKind, SemanticValuePayload, StageSet, SymbolPayload, ToolchainGlobalSourceRoot,
    ValueComponentPolicy, ValuePresence,
};

use support::{
    build_fixture_error, build_single_fixture_world, fixture_root, initializer_from_source,
};

/// Extract the pipeline trace from an invocation result, success or failure.
/// Exposure regressions are trace facts and must stay observable even when
/// body execution of the selected candidate is not (yet) supported.
fn trace_of<'a>(
    result: &'a Result<InvocationOutcome, OrdinaryInvocationFailure>,
) -> &'a OrdinaryPipelineTrace {
    match result {
        Ok(lang_build::InvocationResult::SemanticResult {
            value: lang_build::ProjectedInvocationOutcome::Unit(u),
            ..
        }) => &u.trace,
        Ok(lang_build::InvocationResult::SemanticResult {
            value: lang_build::ProjectedInvocationOutcome::SingleMember(r),
            ..
        }) => &r.trace,
        Ok(lang_build::InvocationResult::SemanticResult {
            value: lang_build::ProjectedInvocationOutcome::ClusterSymbol(c),
            ..
        }) => &c.trace,
        Ok(lang_build::InvocationResult::Residual(_))
        | Ok(lang_build::InvocationResult::Diagnostic(_)) => {
            panic!("ordinary invocation did not produce a semantic result")
        }
        Err(OrdinaryInvocationFailure::NoTargetValues { trace })
        | Err(OrdinaryInvocationFailure::NoFullyAdmissibleCandidate { trace, .. })
        | Err(OrdinaryInvocationFailure::Ambiguous { trace, .. })
        | Err(OrdinaryInvocationFailure::DynamicLegality { trace, .. })
        | Err(OrdinaryInvocationFailure::SelectedDelete { trace, .. })
        | Err(OrdinaryInvocationFailure::SelectedBody { trace, .. })
        | Err(OrdinaryInvocationFailure::SelectedCoreBody { trace, .. })
        | Err(OrdinaryInvocationFailure::MetaReturnTypeRootMismatch { trace, .. })
        | Err(OrdinaryInvocationFailure::ResultTypeHasNoPattern { trace, .. })
        | Err(OrdinaryInvocationFailure::MigrationResultTypeChanged { trace, .. })
        | Err(OrdinaryInvocationFailure::MigrationOutputProjectionFailed { trace })
        | Err(OrdinaryInvocationFailure::Residual { trace, .. })
        | Err(OrdinaryInvocationFailure::CyclicVal2 { trace, .. }) => trace,
    }
}

fn invoke(
    world: &mut CompilationWorld,
    spelling: &str,
    context: OrdinaryInvocationContext<'_>,
    provenance: &str,
) -> Result<InvocationOutcome, OrdinaryInvocationFailure> {
    let initializer = initializer_from_source(spelling);
    let call_site = extract_single_call_site(&initializer).expect("normalized call");
    world.invoke_ordinary_call(
        world.package_root_node(),
        &call_site,
        context,
        Provenance::new(provenance),
    )
}

#[test]
fn unknown_actual_uses_primitive_plain_and_never_world_fabricated_const() {
    let mut manifest = BuildManifest::new("app", vec!["app".to_string()]);
    manifest
        .global_implementation_roots
        .push(ToolchainGlobalSourceRoot::new(
            fixture_root()
                .join("global_implementation")
                .join("unknown_actual_default"),
        ));
    let mut world = CompilationWorld::from_manifest(&manifest).expect("probe family builds");
    let no_fabricated_modes = [];
    let result = invoke(
        &mut world,
        "let result = mystery probe::;",
        OrdinaryInvocationContext::open_static(&no_fabricated_modes),
        "unknown actual defaults to Plain",
    );
    let selected = trace_of(&result).selected.unwrap_or_else(|| {
        panic!("selection must seal before the unknown body result is diagnosed: {result:?}")
    });
    let selected = world
        .semantic_world()
        .value(selected)
        .expect("selected call entry");
    let SemanticValuePayload::CallEntry(entry) = &selected.payload else {
        panic!("probe selection is an ordinary call entry");
    };
    let formal = entry
        .closure
        .as_ref()
        .and_then(|closure| closure.head.as_ref())
        .and_then(|head| head.formal_frame().explicit_parameters.first())
        .expect("one explicit formal");
    let lang_syntax::NormPatternElem::BindingSlot(formal) = formal else {
        panic!("probe formal is a binding slot");
    };
    assert!(
        formal.policy.is_none(),
        "Plain formal must beat the const formal for an unknown actual; a fabricated const would select the other candidate"
    );
}

fn seal_static(explicit_argument_mutability: &[PolicyMode]) -> OrdinaryInvocationContext<'_> {
    let mut context = OrdinaryInvocationContext::open_static(explicit_argument_mutability);
    context.phase = Phase::SealStatic;
    context.policy_env = PolicyEnv::SealStatic;
    context.execution_env = ExecutionEnv::SealStatic;
    context
}

// ---------------------------------------------------------------------------
// Fixture build smoke: the committed semantic workspaces must build.
// ---------------------------------------------------------------------------

#[test]
fn fixture_type_binding_builds() {
    let _ = build_single_fixture_world("type_binding", "app");
}

#[test]
fn fixture_cluster_exposure_builds() {
    let _ = build_single_fixture_world("cluster_exposure", "app");
}

// ---------------------------------------------------------------------------
// ① `let T: type = uint8;` — ordinary let binding: the RHS type value is
// bound to the fresh destination Symbol `T`. No aliasing, no Pattern reroot.
// ---------------------------------------------------------------------------

#[test]
fn type_binding_is_fresh_symbol_no_alias_no_reroot() {
    let world = build_single_fixture_world("type_binding", "app");
    let t = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "T")
        .expect("destination symbol T installed");
    let uint8 = world
        .semantic_world()
        .symbol_in_namespace(world.core_node(), "uint8")
        .expect("core uint8");

    // Fresh destination Symbol: T is its own cluster Symbol, not an alias
    // facet of uint8.
    assert_ne!(
        t.identity, uint8.identity,
        "let binding creates a fresh destination Symbol, never an alias"
    );

    // Ordinary binding semantics: the RHS value (the uint8 type value) is
    // bound to the symbol T — T reads the same PatternValue.
    assert_eq!(
        t.pure_p_pattern(),
        uint8.pure_p_pattern(),
        "the bound type value is the RHS value itself"
    );

    // No reroot: carrier rebinding does not rewrite the Pattern's owning
    // cluster; uint8's PatternValue stays owned by uint8.
    let pattern = uint8.pure_p_pattern().expect("core uint8 pure-P");
    assert_eq!(
        world.semantic_world().owner_cluster(pattern),
        Some(PatternClusterOwner::Installed(uint8.identity)),
        "carrier rebinding must not reroot the RHS PatternValue"
    );
}

// ---------------------------------------------------------------------------
// ② Cluster with members of different Policy exposes only the member views
// whose own value Policy is visible at the call phase (C2 is per-member).
// ---------------------------------------------------------------------------

#[test]
fn cluster_exposure_filters_per_member_view_by_phase() {
    let mut world = build_single_fixture_world("cluster_exposure", "app");
    let muts = [PolicyMode::Const];

    // OpenStatic: both the meta-P2 member and the compile-P2 member are
    // visible, so both enter C2.
    let open = invoke(
        &mut world,
        "let R: type = uint8 pick;",
        OrdinaryInvocationContext::open_static(&muts),
        "open-static exposure",
    );
    let open_trace = trace_of(&open);
    assert_eq!(
        open_trace.c0_target_values.len(),
        2,
        "both members enter C0"
    );
    assert_eq!(
        open_trace.c1_visible_values, open_trace.c0_target_values,
        "internal caller sees every member view"
    );
    assert_eq!(
        open_trace.c2_phase_values.len(),
        2,
        "meta and compile member P1 stages are both visible at OpenStatic"
    );
    for value in &open_trace.callable_values {
        assert!(
            open_trace.c2_phase_values.contains(value),
            "Cc only filters within the C2-exposed member subset"
        );
    }

    // SealStatic: the meta member's P1 stages are not visible; only the
    // compile member view survives C2. The dropped member stays a legal
    // cluster member — C2 keeps/drops individual views, never the Symbol.
    let seal = invoke(
        &mut world,
        "let R: type = uint8 pick;",
        seal_static(&muts),
        "seal-static exposure",
    );
    let seal_trace = trace_of(&seal);
    assert_eq!(
        seal_trace.c0_target_values.len(),
        2,
        "C0 still carries both members — exposure happens at C2, not C0"
    );
    assert_eq!(
        seal_trace.c2_phase_values.len(),
        1,
        "only the phase-matching member view is exposed"
    );
    let pick = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "pick")
        .expect("pick cluster symbol");
    let surviving = seal_trace.c2_phase_values[0];
    let surviving_view = pick
        .member_views
        .iter()
        .find(|view| view.value == Some(surviving))
        .expect("surviving C2 value is a member view of the cluster");
    assert!(
        surviving_view
            .view
            .pair
            .value
            .stages
            .visible_at(Phase::SealStatic),
        "the surviving member is exactly the one whose own view Policy is visible"
    );
}

// ---------------------------------------------------------------------------
// ③ Crossed Policy coordinates are never unioned across members: each member
// view keeps its own value/pattern Policy, identical to its own value object.
// ---------------------------------------------------------------------------

#[test]
fn member_view_policies_do_not_union_across_members() {
    let world = build_single_fixture_world("cluster_exposure", "app");
    let pick = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "pick")
        .expect("pick cluster symbol");
    assert_eq!(pick.sibling_vals.len(), 2);
    assert_eq!(pick.member_views.len(), 2);

    let a = &pick.member_views[0];
    let b = &pick.member_views[1];
    assert_ne!(
        a.view.pair.value.stages, b.view.pair.value.stages,
        "fixture must keep two members with genuinely different P1 stages"
    );
    let union = a.view.pair.value.stages.union(&b.view.pair.value.stages);
    assert_ne!(a.view.pair.value.stages, union, "member A carries no union");
    assert_ne!(b.view.pair.value.stages, union, "member B carries no union");

    // Each view's coordinates are its own member's canonical P1 — the same
    // PolicyPair carried by the member's function-object value.
    for view in &pick.member_views {
        let value = view.value.expect("callable member view has a value");
        let object = world
            .semantic_world()
            .value(value)
            .expect("member value exists");
        assert_eq!(object.policy.value, view.view.pair.value);
        assert_eq!(object.policy.pattern, view.view.pair.pattern);
    }
}

// ---------------------------------------------------------------------------
// ④ A complete type is an ordinary first-class value but is not callable
// unless its complete callspace contains associated `()`.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// ⑤ A callable member owns its own function-object value with associated
// Val2["()"] and a terminal FunctionItem call entry. (The injected self
// slot 0 is exercised by ⑦: the call product carries explicit args only.)
// ---------------------------------------------------------------------------

#[test]
fn callable_member_owns_function_object_and_terminal_call_entry() {
    let world = build_single_fixture_world("declared_result", "app");
    let make_type = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "make_type")
        .expect("make_type cluster symbol");
    assert_eq!(make_type.sibling_vals.len(), 1);
    let function_value = make_type.sibling_vals[0];
    let function_obj = world
        .semantic_world()
        .value(function_value)
        .expect("function object value");
    assert!(matches!(
        function_obj.payload,
        SemanticValuePayload::FunctionObject { .. }
    ));

    let entries = world
        .semantic_world()
        .associated_values_for_value(function_value, "()")
        .unwrap_or(&[]);
    assert_eq!(entries.len(), 1, "one () call entry on the function object");
    let call_obj = world
        .semantic_world()
        .value(entries[0])
        .expect("call entry value");
    assert!(matches!(
        call_obj.payload,
        SemanticValuePayload::CallEntry(_)
    ));

    // Terminal FunctionItem: the call entry has its own type/pattern and an
    // empty Val2.
    assert_ne!(function_obj.pattern, call_obj.pattern);
    assert_ne!(function_obj.type_value, call_obj.type_value);
    assert!(
        world
            .semantic_world()
            .associated_values_for_pattern(call_obj.pattern, "()")
            .is_none(),
        "call entry is terminal: no further ()"
    );
}

// ---------------------------------------------------------------------------
// ⑥ Privileged `struct` goes through the normal overload path: privilege is
// a selected-body capability, never a resolution bypass.
// ---------------------------------------------------------------------------

#[test]
fn privileged_struct_uses_the_normal_overload_path() {
    let mut world =
        CompilationWorld::from_manifest(&BuildManifest::new("app", vec!["app".to_string()]))
            .expect("core semantic world builds");
    let result = invoke(
        &mut world,
        "let T: type = (uint8 a) struct;",
        OrdinaryInvocationContext::open_static(&[PolicyMode::Const]),
        "privileged struct",
    )
    .expect("struct is selected through the ordinary spine");
    let lang_build::InvocationResult::SemanticResult {
        value: lang_build::ProjectedInvocationOutcome::SingleMember(result),
        ..
    } = result
    else {
        panic!("struct declares one complete-type result");
    };
    assert_eq!(result.trace.c0_target_values.len(), 1);
    assert_eq!(
        result.trace.c1_visible_values,
        result.trace.c0_target_values
    );
    assert_eq!(result.trace.c3_call_entries.len(), 1);
    assert!(
        result.trace.selected.is_some(),
        "privilege applies only after ordinary selection"
    );
    let lang_build::ReturnedSemanticEntity::CompleteType(returned) = &result.returned else {
        panic!("struct semantic result is complete tau");
    };
    assert!(result.complete_result[0].value.is_some());
    let owner = world
        .semantic_world()
        .pattern_owner(returned.pattern)
        .expect("struct result Pattern owner")
        .owner;
    // Direct `struct` never creates a `MetaInstance(struct, arguments)`
    // scope of its own: the complete type attaches to the ambient
    // declaration environment.
    let ambient_owner = world
        .semantic_world()
        .namespace_owner(world.package_root_node())
        .expect("package root owner");
    assert_eq!(owner, ambient_owner);
    assert!(matches!(
        world
            .semantic_world()
            .owners()
            .node(owner)
            .expect("owner node")
            .kind,
        SemanticOwnerKind::PackageRoot { .. }
    ));
}

// ---------------------------------------------------------------------------
// ⑦ Source meta callables share the core return ontology.  Repeated
// contributions at the declaration layer produce multiple cluster members
// (the two `let pick` declarations — pinned in ③); inside one meta body, the
// legal type member is constructed self-rooted (`let r = (...) |> struct;`)
// and delivered by the `r;` terminal.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// ⑧ A Meta outcome is received by an ordinary let binding: build-time
// `let T: type = (uint8 a) struct;` and `let R: type = uint8 make_one;`
// install fresh destination cluster Symbols.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// ⑨ Runtime migration full chain — exercise the complete
// `install_semantic_value` → `invoke_policy_migration` chain
// (source value → transport member selection → migrated demanded view).
// ---------------------------------------------------------------------------

#[test]
fn runtime_migration_full_chain_through_source_backed_transport() {
    let mut manifest = BuildManifest::new("app", vec!["app".to_string()]);
    manifest
        .global_implementation_roots
        .push(ToolchainGlobalSourceRoot::under(
            fixture_root()
                .join("global_implementation")
                .join("uint8_transport"),
            vec!["core".to_string(), "uint8".to_string()],
        ));
    let mut world = CompilationWorld::from_manifest(&manifest)
        .expect("source-backed transport bundle mounts without conflict");

    let uint8 = world
        .resolve_with_expectation("uint8", ResolveExpectation::CoreTypeProjection)
        .expect("core uint8 type");
    let SymbolPayload::CompleteTypeProjection(uint8_type) = uint8.payload else {
        panic!("uint8 resolves as a CompleteType projection");
    };
    let type_value = uint8_type.represented_type;

    let stage_set = |items: &[PolicyStage]| {
        let mut set = StageSet::new();
        for stage in items {
            set.insert(*stage);
        }
        set
    };
    let policy = |value_stages: &[PolicyStage], _mode: PolicyMode| PolicyPair {
        value: ValueComponentPolicy {
            stages: stage_set(value_stages),
            presence: ValuePresence::Present,
        },
        pattern: PatternComponentPolicy {
            stages: stage_set(&[PolicyStage::Compile]),
        },
    };
    let source_policy = policy(&[PolicyStage::Compile], PolicyMode::Const);
    let target_policy = policy(&[PolicyStage::Runtime], PolicyMode::Mut);

    let source = world
        .install_semantic_value(
            type_value,
            source_policy.clone(),
            Provenance::new("compile uint8 migration source"),
        )
        .expect("installed value reuses the uint8 PatternValue");
    let request = PolicyMigrationRequest::new(
        lang_build::PolicyView {
            pair: source_policy,
            mode: PolicyMode::Const,
        },
        lang_build::ResultPolicyDemand {
            pair_query: lang_build::P1Projection::Pair(target_policy.clone()),
            mode: PolicyMode::Mut,
        },
        type_value,
        source,
        Provenance::new("const compile -> mut runtime demand"),
    )
    .expect("legal migration request");

    let migration = world
        .invoke_policy_migration(&request)
        .expect("source-backed transport member is selected and invoked");
    assert!(
        migration
            .invocation
            .selected
            .migration_output_endpoint
            .is_some(),
        "selected transport carries the single migration output authority"
    );
    assert_eq!(migration.demanded_view.len(), 1);
    assert_eq!(
        migration.demanded_view[0].view.pair.value,
        target_policy.value
    );
    assert_eq!(
        migration.demanded_view[0]
            .value
            .expect("migrated demanded view carries a runtime value")
            .id,
        source,
        "a forwarding transport keeps the identity of the existing source value"
    );
}

// ---------------------------------------------------------------------------
// ⑩ A flat symbol-level Policy aggregate cannot reproduce the canonical
// member-level result: the per-member C2 exposure decision differs from what
// any single unioned coordinate would produce.
// ---------------------------------------------------------------------------

#[test]
fn flat_symbol_policy_cannot_express_member_level_exposure() {
    let mut world = build_single_fixture_world("cluster_exposure", "app");
    let pick = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "pick")
        .expect("pick cluster symbol");
    let views = pick.member_views.clone();
    assert_eq!(views.len(), 2);

    let visible: Vec<_> = views
        .iter()
        .filter(|view| view.view.pair.value.stages.visible_at(Phase::SealStatic))
        .collect();
    let hidden: Vec<_> = views
        .iter()
        .filter(|view| !view.view.pair.value.stages.visible_at(Phase::SealStatic))
        .collect();
    assert_eq!(visible.len(), 1);
    assert_eq!(hidden.len(), 1);

    // The flat union coordinate WOULD be visible at SealStatic — a flat
    // symbol-level Policy cannot express the member-level distinction.
    let union = visible[0]
        .view
        .pair
        .value
        .stages
        .union(&hidden[0].view.pair.value.stages);
    assert!(union.visible_at(Phase::SealStatic));

    // The canonical pipeline reads the per-member view Policy: the hidden
    // member is dropped at C2 even though the flat union would keep it.
    let muts = [PolicyMode::Const];
    let seal = invoke(
        &mut world,
        "let R: type = uint8 pick;",
        seal_static(&muts),
        "member-level authority",
    );
    let trace = trace_of(&seal);
    assert!(trace
        .c2_phase_values
        .contains(&visible[0].value.expect("callable member value")));
    assert!(!trace
        .c2_phase_values
        .contains(&hidden[0].value.expect("callable member value")));
}

// ---------------------------------------------------------------------------
// ⑪ Direct forwarding of an external type value out of a meta body violates
// the self-root invariant: `{ let r = t; r; }` must fail with
// MetaReturnTypeRootMismatch. Root mismatch is terminal.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// ⑫ A meta body with construction effects but no terminal delivers nothing:
// `{ let r = (t inner) |> struct; }` (no trailing `r;`) must not succeed.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// ⑬ Alias syntax remains available to Raw/Normalized AST consumers, but it
// has no semantic forwarding authority.
// ---------------------------------------------------------------------------

#[test]
fn unwired_lexical_alias_creates_no_semantic_entity() {
    let error = build_fixture_error("lexical_alias_unwired", "app");
    assert_eq!(error.diagnostics.len(), 1);
    assert_eq!(
        error.diagnostics[0].code,
        Some(ResolverCode::UnsupportedLexicalAlias)
    );
    assert!(error.diagnostics[0]
        .message
        .contains("lexical alias resolution is not implemented"));
    assert!(error.diagnostics[0]
        .message
        .contains("must not install or forward a semantic entity"));
}
