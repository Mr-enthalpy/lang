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

#[test]
fn toolchain_global_closure_declaration_requires_tau_formation() {
    let mut manifest = BuildManifest::new("app", vec!["app".to_string()]);
    manifest.global_implementation_roots.push(global_bundle());
    let error = CompilationWorld::from_manifest(&manifest)
        .expect_err("global source has no privileged function-value binding fallback");
    assert!(error
        .diagnostics
        .iter()
        .any(|d| d.message.contains("closure-to-tau formation consumer")));
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
fn source_closure_frontier_precedes_binding_demand() {
    let error = CompilationWorld::from_manifest(&BuildManifest::single_source_root(
        "app",
        vec!["app".to_string()],
        fixture_source_root("binding_result_demand", "app"),
    ))
    .expect_err("source closure formation is unavailable");
    assert!(error
        .diagnostics
        .iter()
        .any(|d| d.message.contains("closure-to-tau formation consumer")));
}

#[test]
fn result_mode_preference_is_sealed_independently_of_callable_mode() {
    let mut world = support::AssociatedFamily::new(&[
        "close let first = (const let self, const let t: type): const + compile -> let r: type => { t; };",
        "close let second = (const let self, const let t: type): mut + compile -> let r: type => { t; };",
    ]);
    let call = extract_single_call_site(&initializer_from_source("let x = uint8 choose;"))
        .expect("normalized call");
    for (result_mode, callable_mode) in [
        (PolicyMode::Mut, PolicyMode::Const),
        (PolicyMode::Const, PolicyMode::Const),
    ] {
        let failure = world
            .invoke_ordinary_call(
                world.package_root_node(),
                &call,
                OrdinaryInvocationContext::open_static(&[PolicyMode::Const])
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
fn named_transport_source_requires_tau_formation() {
    let mut manifest = BuildManifest::new("app", vec!["app".into()]);
    manifest
        .global_implementation_roots
        .push(transport_bundle());
    let error = CompilationWorld::from_manifest(&manifest).expect_err("no sibling fallback");
    assert!(error
        .diagnostics
        .iter()
        .any(|d| d.message.contains("closure-to-tau formation consumer")));
}

#[test]
fn literals_form_abstract_values_before_concrete_construction() {
    let manifest = BuildManifest::single_source_root(
        "app",
        vec!["app".to_string()],
        fixture_source_root("abstract_literal_pipeline", "app"),
    );
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
