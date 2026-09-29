mod support;

use lang_build::{
    extract_single_call_site, BuildManifest, CompilationWorld, OrdinaryInvocationContext,
    OrdinaryInvocationFailure, PolicyMode, Provenance, SemanticValuePayload, SourceRoot,
    ToolchainGlobalSourceRoot,
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

#[test]
fn toolchain_global_source_is_parsed_installed_and_invoked_through_ordinary_spine() {
    let mut manifest = BuildManifest::new("app", vec!["app".to_string()]);
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

    let initializer = initializer_from_source("let x = uint8 global_identity::;");
    let call = extract_single_call_site(&initializer).expect("normalized global call");
    let failure = world
        .invoke_ordinary_call(
            world.package_root_node(),
            &call,
            OrdinaryInvocationContext::open_static(&[PolicyMode::Const]),
            Provenance::new("source completion frontier"),
        )
        .expect_err("serial expression completion is not executable");
    let OrdinaryInvocationFailure::SelectedBody { failure, trace } = failure else {
        panic!("expected a sealed selected-body failure");
    };
    assert_eq!(
        failure.diagnostic.code,
        Some(lang_build::ResolverCode::UnsupportedSelectedSourceBody)
    );
    assert!(failure
        .diagnostic
        .message
        .contains("serial expression completion requires the shared continuation consumer"));
    assert!(
        trace.selected.is_some(),
        "completion failure occurs after unique selection"
    );

    let bare_initializer = initializer_from_source("let x = uint8 global_identity;");
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
fn binding_demand_selects_before_reporting_unavailable_completion() {
    let error = CompilationWorld::from_manifest(&BuildManifest::single_source_root(
        "app",
        vec!["app".to_string()],
        fixture_source_root("binding_result_demand", "app"),
    ))
    .expect_err("the uniquely selected source body still requires completion");
    assert!(error.diagnostics.iter().any(|d| d
        .message
        .contains("serial expression completion requires the shared continuation consumer")));
}

#[test]
fn result_mode_preference_is_sealed_independently_of_callable_mode() {
    let mut world = CompilationWorld::from_manifest(&BuildManifest::single_source_root(
        "app",
        vec!["app".to_string()],
        fixture_source_root("result_mode_candidates", "app"),
    ))
    .expect("declarations need no body execution");
    let call = extract_single_call_site(&initializer_from_source("let x = uint8 choose;"))
        .expect("normalized call");
    for (result_mode, callable_mode) in [
        (PolicyMode::Mut, PolicyMode::Const),
        (PolicyMode::Const, PolicyMode::Mut),
    ] {
        let failure = world
            .invoke_ordinary_call(
                world.package_root_node(),
                &call,
                OrdinaryInvocationContext::open_static(&[PolicyMode::Plain])
                    .with_result_policy_demand(lang_build::ResultPolicyDemand {
                        pair_query: lang_build::P1Projection::Infer,
                        mode: result_mode,
                    }),
                Provenance::new("result mode before body completion"),
            )
            .expect_err("selected source completion is unavailable");
        let OrdinaryInvocationFailure::SelectedBody { failure, trace } = failure else {
            panic!("expected a sealed body failure, got {failure:?}");
        };
        assert_eq!(
            failure.diagnostic.code,
            Some(lang_build::ResolverCode::UnsupportedSelectedSourceBody)
        );
        assert_eq!(trace.a_fully_admissible.len(), 2);
        assert_eq!(trace.bp_prime.len(), 1);
        let selected = trace.selected.expect("selection seals before execution");
        let SemanticValuePayload::CallEntry(entry) =
            &world.semantic_world().value(selected).unwrap().payload
        else {
            panic!("selected implementation entry");
        };
        assert_eq!(entry.complete_result_view.mode, result_mode);
        assert_eq!(entry.callable_view.mode, callable_mode);
    }
}

#[test]
fn type_changing_migration_candidate_is_excluded_before_preference() {
    let mut manifest = BuildManifest::new("app", vec!["app".to_string()]);
    manifest
        .global_implementation_roots
        .push(ToolchainGlobalSourceRoot::under(
            fixture_root()
                .join("global_implementation")
                .join("wrong_type_transport"),
            vec!["core".to_string(), "uint8".to_string()],
        ));
    let mut world = CompilationWorld::from_manifest(&manifest).expect("single-stage candidates");
    let ty = world.resolve_type_value("uint8").unwrap();
    let source_view =
        lang_build::declared_policy_view(&[lang_build::PolicyStage::Compile], PolicyMode::Const);
    let source = world
        .install_semantic_value(
            ty,
            source_view.pair.clone(),
            Provenance::new("migration source"),
        )
        .unwrap();
    let request = lang_build::PolicyMigrationRequest::new(
        source_view.clone(),
        lang_build::ResultPolicyDemand {
            pair_query: lang_build::P1Projection::Pair(source_view.pair),
            mode: PolicyMode::Mut,
        },
        ty,
        source,
        Provenance::new("same-Type candidate filter"),
    )
    .unwrap();
    let failure = world
        .invoke_policy_migration(&request)
        .expect_err("source completion is unavailable");
    let OrdinaryInvocationFailure::SelectedBody { failure, trace } = failure else {
        panic!("expected a selected body failure, got {failure:?}");
    };
    assert_eq!(
        failure.diagnostic.code,
        Some(lang_build::ResolverCode::UnsupportedSelectedSourceBody)
    );
    assert_eq!(trace.a_fully_admissible.len(), 1);
    assert!(
        !trace.c0_target_values.contains(&source),
        "explicit source does not become callable self"
    );
    let selected = trace.selected.expect("unique same-Type implementation");
    assert_eq!(trace.a_fully_admissible, vec![selected]);
    let SemanticValuePayload::CallEntry(entry) =
        &world.semantic_world().value(selected).unwrap().payload
    else {
        panic!("call entry")
    };
    let return_pattern = &entry
        .closure
        .as_ref()
        .unwrap()
        .head
        .as_ref()
        .unwrap()
        .returns
        .as_ref()
        .unwrap()
        .annotation
        .as_ref()
        .unwrap()
        .pattern;
    assert!(
        matches!(return_pattern, lang_syntax::NormPattern::Name { name, .. } if name == "uint8")
    );
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
fn policy_let_without_admissible_transport_reports_failure() {
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

    let error = CompilationWorld::from_manifest(&manifest)
        .expect_err("a single-stage transport is not a stage-union execution path");
    assert!(
        error
            .diagnostics
            .iter()
            .any(|d| d.message == "ordinary invocation found no fully admissible candidate"),
        "{:?}",
        error.diagnostics
    );
}

#[test]
fn literals_form_abstract_values_before_concrete_construction() {
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
}
