mod support;
use support::*;

use lang_build::{
    declared_policy_view, policy_view_allows_execution, CompilationWorld, ExecutionEnv, PolicyEnv,
    PolicyMode, PolicyStage, Provenance, ResolveExpectation, SourceCategory, SymbolKind,
    SymbolObject,
};

#[test]
fn core_type_is_visible_in_open_static_phase() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let symbol = world
        .namespace_projection()
        .capability()
        .resolve_complete_type_projection_with_policy(
            "uint8",
            &world.package_context(),
            PolicyEnv::OpenStatic,
        )
        .expect("uint8 should be visible in the open-static phase");
    assert_eq!(symbol.kind, SymbolKind::CompleteTypeProjection);
    assert_eq!(symbol.name, "uint8");
}

#[test]
fn policy_view_stage_controls_execution_without_changing_mode() {
    let meta = declared_policy_view(&[PolicyStage::Meta], PolicyMode::Plain);
    let runtime = declared_policy_view(&[PolicyStage::Runtime], PolicyMode::Plain);

    assert!(policy_view_allows_execution(
        &meta,
        ExecutionEnv::OpenStatic
    ));
    assert!(!policy_view_allows_execution(&meta, ExecutionEnv::Runtime));
    assert!(!policy_view_allows_execution(
        &runtime,
        ExecutionEnv::OpenStatic
    ));
    assert!(policy_view_allows_execution(
        &runtime,
        ExecutionEnv::Runtime
    ));
    assert_eq!(meta.mode, PolicyMode::Plain);
    assert_eq!(runtime.mode, PolicyMode::Plain);
}

#[test]
fn phase_projection_does_not_define_symbol_existence() {
    let world = build_single_fixture_world_with_uint8_transport("user_runtime_values", "app");
    let context = world.package_context();
    let capability = world.namespace_projection().capability();

    let symbol = capability
        .resolve(&["x".to_string()], &context)
        .expect("name resolution establishes the Symbol first");
    assert_eq!(symbol.name, "x");

    assert!(capability
        .resolve_with_policy(
            &["x".to_string()],
            &context,
            ResolveExpectation::Object,
            PolicyEnv::OpenStatic,
        )
        .is_err());
}

#[test]
fn seal_phase_projection_reads_concrete_policy_views() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let mut delta = world.namespace_projection().empty_delta();
    for (name, stage) in [
        ("meta_only", PolicyStage::Meta),
        ("compile_only", PolicyStage::Compile),
        ("seal_only", PolicyStage::Seal),
    ] {
        let symbol_id = delta.allocate_symbol_id();
        let mut symbol = SymbolObject::placeholder(
            symbol_id,
            name,
            SymbolKind::Placeholder,
            SourceCategory::DeclaredSymbol,
            Some(world.package_root_node()),
            Provenance::new(name),
        );
        symbol.policy_view = Some(declared_policy_view(&[stage], PolicyMode::Plain));
        delta.insert_symbol(world.package_root_node(), symbol);
    }
    let snapshot = world
        .namespace_projection()
        .install_delta(delta)
        .expect("install policy fixtures");
    let context = world.package_context();
    let resolve = |name: &str, env| {
        snapshot.capability().resolve_with_policy(
            &[name.to_string()],
            &context,
            ResolveExpectation::Object,
            env,
        )
    };

    assert!(resolve("meta_only", PolicyEnv::SealStatic).is_err());
    assert!(resolve("compile_only", PolicyEnv::SealStatic).is_ok());
    assert!(resolve("seal_only", PolicyEnv::SealStatic).is_ok());
    assert!(resolve("seal_only", PolicyEnv::OpenStatic).is_err());
}
