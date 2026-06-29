mod support;
use support::*;

use lang_build::meta::{expand_meta_initializer_via_invocation, try_expand_early_meta_initializer};
use lang_build::{
    policy_set_meta, CandidateBuildIdentityPlaceholder, CompilationWorld, ExecutionEnv, PolicyEnv,
    Provenance, SourceCategory, SymbolKind, SymbolObject,
};

#[test]
fn failed_struct_meta_leaves_no_partial_generated_subtree() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let initializer = initializer_from_source("let T: type = (uint8 a, uint8 a) |> struct");
    let snapshot = world.snapshot().clone();
    let error = try_expand_early_meta_initializer(
        &snapshot,
        world.package_root_node(),
        "T",
        &initializer,
        &world.package_context(),
        Provenance::new("test duplicate fields"),
    )
    .expect_err("duplicate fields must fail");

    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("duplicate field name")));
    assert!(
        snapshot
            .child_symbol(world.package_root_node(), "T")
            .is_none(),
        "failed meta expansion must not leave a generated type"
    );
}

#[test]
fn unresolved_meta_target_returns_none() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let initializer = initializer_from_source("let T: type = (uint8 a) |> missing_meta");
    let expansion = try_expand_early_meta_initializer(
        world.snapshot(),
        world.package_root_node(),
        "T",
        &initializer,
        &world.package_context(),
        Provenance::new("unresolved meta target"),
    )
    .expect("unresolved target is not treated as parser/normalizer special case");
    assert!(expansion.is_none());
}

#[test]
fn try_expand_early_meta_initializer_returns_none_for_non_meta_call() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let mut delta = world.snapshot().empty_delta();
    let object_id = delta.allocate_symbol_id();
    let mut object = SymbolObject::placeholder(
        object_id,
        "not_meta",
        SymbolKind::Placeholder,
        SourceCategory::DeclaredSymbol,
        Some(world.package_root_node()),
        Provenance::new("meta-visible non-meta object"),
    );
    object.policy_metadata.policy_set = policy_set_meta();
    delta.insert_symbol(world.package_root_node(), object);
    let snapshot = world
        .snapshot()
        .install_delta(delta)
        .expect("install non-meta object");
    let context = lang_build::ResolverContext::with_mounts(
        world.package_root_node(),
        vec![snapshot.root_node()],
        vec![world.core_node()],
    );
    let initializer = initializer_from_source("let T: type = (uint8 a) |> not_meta");

    let expansion = try_expand_early_meta_initializer(
        &snapshot,
        world.package_root_node(),
        "T",
        &initializer,
        &context,
        Provenance::new("non-meta probe"),
    )
    .expect("non-meta call is not a probing error");

    assert!(
        expansion.is_none(),
        "try expansion must ignore resolved non-meta call initializers"
    );
}

#[test]
fn explicit_meta_initializer_driver_rejects_non_meta_call() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let mut delta = world.snapshot().empty_delta();
    let object_id = delta.allocate_symbol_id();
    let mut object = SymbolObject::placeholder(
        object_id,
        "not_meta",
        SymbolKind::Placeholder,
        SourceCategory::DeclaredSymbol,
        Some(world.package_root_node()),
        Provenance::new("meta-visible non-meta object"),
    );
    object.policy_metadata.policy_set = policy_set_meta();
    delta.insert_symbol(world.package_root_node(), object);
    let snapshot = world
        .snapshot()
        .install_delta(delta)
        .expect("install non-meta object");
    let context = lang_build::ResolverContext::with_mounts(
        world.package_root_node(),
        vec![snapshot.root_node()],
        vec![world.core_node()],
    );
    let initializer = initializer_from_source("let T: type = (uint8 a) |> not_meta");

    let error = expand_meta_initializer_via_invocation(
        &initializer,
        &snapshot,
        world.package_root_node(),
        "T",
        &context,
        PolicyEnv::Meta,
        ExecutionEnv::Meta,
        CandidateBuildIdentityPlaceholder::default(),
        Provenance::new("explicit non-meta driver"),
        None,
    )
    .expect_err("explicit meta driver must reject non-meta targets");

    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("no meta-function payload")));
}

#[test]
fn meta_function_kind_without_payload_is_hard_error() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let mut delta = world.snapshot().empty_delta();
    let bad_meta = delta.allocate_symbol_id();
    let mut bad_symbol = lang_build::SymbolObject::placeholder(
        bad_meta,
        "bad_meta",
        SymbolKind::MetaFunction,
        SourceCategory::DeclaredSymbol,
        Some(world.package_root_node()),
        Provenance::new("bad meta without payload"),
    );
    bad_symbol
        .policy_metadata
        .policy_set
        .insert(lang_build::PolicyFlag::Meta);
    delta.insert_symbol(world.package_root_node(), bad_symbol);
    let snapshot = world
        .snapshot()
        .install_delta(delta)
        .expect("install bad meta");
    let initializer = initializer_from_source("let T: type = (uint8 a) |> bad_meta");
    let error = try_expand_early_meta_initializer(
        &snapshot,
        world.package_root_node(),
        "T",
        &initializer,
        &lang_build::ResolverContext::with_mounts(
            world.package_root_node(),
            vec![snapshot.root_node()],
            vec![world.core_node()],
        ),
        Provenance::new("bad meta call"),
    )
    .expect_err("MetaFunction kind without payload is hard error");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("no meta-function payload")));
}
