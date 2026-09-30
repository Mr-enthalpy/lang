mod support;
use support::*;

use lang_build::{
    declared_policy_view, policy_view_visible_at, CompilationWorld, ObservationHorizon, PolicyEnv,
    PolicyMode, Provenance, ResolveExpectation, SourceCategory, Stage, SymbolKind, SymbolObject,
};

#[test]
fn core_type_is_visible_in_open_static_horizon() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let symbol = world
        .namespace_projection()
        .capability()
        .resolve_complete_type_projection_with_policy(
            "uint8",
            &world.package_context(),
            PolicyEnv::OpenStatic,
        )
        .expect("uint8 should be visible in the open-static horizon");
    assert_eq!(symbol.kind, SymbolKind::CompleteTypeProjection);
    assert_eq!(symbol.name, "uint8");
}

#[test]
fn policy_view_stage_controls_visibility_without_changing_mode() {
    let meta = declared_policy_view(Stage::Meta, PolicyMode::Plain);
    let runtime = declared_policy_view(Stage::Runtime, PolicyMode::Plain);

    assert!(policy_view_visible_at(
        &meta,
        ObservationHorizon::OpenStatic
    ));
    assert!(!policy_view_visible_at(&meta, ObservationHorizon::Runtime));
    assert!(!policy_view_visible_at(
        &runtime,
        ObservationHorizon::OpenStatic
    ));
    assert!(policy_view_visible_at(
        &runtime,
        ObservationHorizon::Runtime
    ));
    assert_eq!(meta.mode, PolicyMode::Plain);
    assert_eq!(runtime.mode, PolicyMode::Plain);
}

#[test]
fn horizon_projection_does_not_define_symbol_existence() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let mut delta = world.namespace_projection().empty_delta();
    let mut symbol = SymbolObject::new(
        delta.allocate_symbol_id(),
        "x",
        SymbolKind::Object,
        SourceCategory::DeclaredSymbol,
        Some(world.package_root_node()),
        Provenance::new("runtime observation fixture"),
    );
    symbol.policy_view = Some(declared_policy_view(Stage::Runtime, PolicyMode::Plain));
    delta.insert_symbol(world.package_root_node(), symbol);
    let snapshot = world
        .namespace_projection()
        .install_delta(delta)
        .expect("install fixture");
    let context = world.package_context();
    let capability = snapshot.capability();

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
fn seal_horizon_projection_reads_concrete_policy_views() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let mut delta = world.namespace_projection().empty_delta();
    for (name, stage) in [
        ("meta_only", Stage::Meta),
        ("compile_only", Stage::Compile),
        ("seal_only", Stage::Seal),
    ] {
        let symbol_id = delta.allocate_symbol_id();
        let mut symbol = SymbolObject::new(
            symbol_id,
            name,
            SymbolKind::Object,
            SourceCategory::DeclaredSymbol,
            Some(world.package_root_node()),
            Provenance::new(name),
        );
        symbol.policy_view = Some(declared_policy_view(stage, PolicyMode::Plain));
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
