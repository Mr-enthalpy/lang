mod support;
use support::*;

use lang_build::meta::try_expand_early_meta_initializer;
use lang_build::{CompilationWorld, Provenance, SourceCategory, SymbolKind};

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
        .any(|diagnostic| diagnostic.message.contains("duplicate struct field")));
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
