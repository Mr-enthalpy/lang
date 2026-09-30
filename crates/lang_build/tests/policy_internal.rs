mod support;
use support::*;

use lang_build::{
    declared_policy_view, policy_view_visible_at, CompilationWorld, ObservationHorizon, PolicyMode,
    Provenance, ResolveExpectation, ResolverCode, ResolverContext, SourceCategory, Stage,
    SymbolKind, SymbolObject,
};

#[test]
fn core_type_is_visible_in_open_static_horizon() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let symbol = world
        .namespace_projection()
        .capability()
        .resolve_complete_type_projection("uint8", &world.package_context())
        .expect("resolve uint8 independently of its exposure");
    assert_eq!(symbol.kind, SymbolKind::CompleteTypeProjection);
    assert_eq!(symbol.name, "uint8");
    assert!(policy_view_visible_at(
        symbol.policy_view.as_ref().unwrap(),
        ObservationHorizon::OpenStatic,
    ));
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

    let view = symbol.policy_view.as_ref().unwrap();
    for horizon in [
        ObservationHorizon::OpenStatic,
        ObservationHorizon::SealStatic,
        ObservationHorizon::Runtime,
    ] {
        let entry = lang_build::PolicyResultEntry {
            value: Some(symbol.id),
            pattern: 11,
            view: view.clone(),
        };
        let observed = lang_build::expose_policy_slice(&entry, horizon);
        assert_eq!(
            lang_build::read_value(&observed).copied(),
            (horizon == ObservationHorizon::Runtime).then_some(symbol.id),
        );
        assert_eq!(
            lang_build::read_pattern(&observed).is_some(),
            horizon != ObservationHorizon::Runtime,
        );
        assert_eq!(
            capability.resolve(&["x".to_string()], &context).unwrap().id,
            symbol.id
        );
        assert_eq!(observed.value_policy, view.pair.value);
    }
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
    let capability = snapshot.capability();
    let meta = capability.resolve_str("meta_only", &context).unwrap();
    let compile = capability.resolve_str("compile_only", &context).unwrap();
    let seal = capability.resolve_str("seal_only", &context).unwrap();
    assert!(!policy_view_visible_at(
        meta.policy_view.as_ref().unwrap(),
        ObservationHorizon::SealStatic
    ));
    assert!(policy_view_visible_at(
        compile.policy_view.as_ref().unwrap(),
        ObservationHorizon::SealStatic
    ));
    assert!(policy_view_visible_at(
        seal.policy_view.as_ref().unwrap(),
        ObservationHorizon::SealStatic
    ));
    assert!(!policy_view_visible_at(
        seal.policy_view.as_ref().unwrap(),
        ObservationHorizon::OpenStatic
    ));
}

#[test]
fn hidden_observation_cannot_suppress_a_search_root_conflict() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).unwrap();
    let mut delta = world.namespace_projection().empty_delta();
    for (root, stage) in [
        (world.package_root_node(), Stage::Runtime),
        (world.core_node(), Stage::Compile),
    ] {
        let mut symbol = SymbolObject::new(
            delta.allocate_symbol_id(),
            "x",
            SymbolKind::Object,
            SourceCategory::DeclaredSymbol,
            Some(root),
            Provenance::new("distinct root binding"),
        );
        symbol.policy_view = Some(declared_policy_view(stage, PolicyMode::Plain));
        delta.insert_symbol(root, symbol);
    }
    let snapshot = world.namespace_projection().install_delta(delta).unwrap();
    let capability = snapshot.capability();
    let path = ["x".to_string()];
    let local = capability
        .resolve(&path, &ResolverContext::new(world.package_root_node()))
        .unwrap();
    let outer = capability
        .resolve(&path, &ResolverContext::new(world.core_node()))
        .unwrap();
    assert_ne!(local.id, outer.id);
    let mounted =
        ResolverContext::with_default_mounts(world.package_root_node(), vec![world.core_node()]);
    for horizon in [
        ObservationHorizon::OpenStatic,
        ObservationHorizon::SealStatic,
        ObservationHorizon::Runtime,
    ] {
        // Exactly one resident value is visible, but both bindings still exist.
        assert_ne!(
            policy_view_visible_at(local.policy_view.as_ref().unwrap(), horizon),
            policy_view_visible_at(outer.policy_view.as_ref().unwrap(), horizon),
        );
        let failure = capability
            .resolve_with_expectation(&path, &mounted, ResolveExpectation::Object)
            .unwrap_err();
        assert_eq!(failure.code, Some(ResolverCode::Conflict));
    }
}
