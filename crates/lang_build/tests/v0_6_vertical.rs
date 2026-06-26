use std::{
    fs,
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

use lang_build::{
    meta::try_expand_early_meta_initializer, BuildManifest, ChildLink, CompilationWorld,
    DiagnosticSeverity, FieldProjection, NamespaceGraphSnapshot, NamespaceMount, NamespaceNodeId,
    NamespaceNodeKind, Provenance, ResolverContext, SourceCategory, SymbolKind, SymbolObject,
    SymbolPayload,
};
use lang_syntax::{NormDecl, NormExpr, NormForm};

struct TempProject {
    root: PathBuf,
}

impl TempProject {
    fn new(name: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("lang_build_{name}_{}_{}", process::id(), nanos));
        fs::create_dir_all(&root).expect("create temp project");
        Self { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }

    fn write(&self, relative: &str, source: &str) {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("fixture parent")).expect("create fixture dirs");
        fs::write(path, source).expect("write fixture");
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn app_manifest(source_root: &Path) -> BuildManifest {
    BuildManifest::single_source_root("app", vec!["app".to_string()], source_root)
}

fn empty_app_manifest() -> BuildManifest {
    BuildManifest::new("app", vec!["app".to_string()])
}

fn placeholder_symbol(
    id: lang_build::SymbolId,
    parent: lang_build::NamespaceNodeId,
    name: &str,
    provenance: &str,
) -> SymbolObject {
    SymbolObject::placeholder(
        id,
        name,
        SymbolKind::Placeholder,
        SourceCategory::DeclaredSymbol,
        Some(parent),
        Provenance::new(provenance),
    )
}

fn initializer_from_source(source: &str) -> NormExpr {
    let parsed = lang_syntax::parse(source);
    assert!(
        parsed.diagnostics.is_empty(),
        "unexpected parse diagnostics:\n{}",
        lang_syntax::dump_diagnostics(&parsed.diagnostics)
    );
    let normalized = lang_syntax::normalize_program(&parsed.program);
    match normalized.forms.as_slice() {
        [NormForm::Let(NormDecl::Let { slot, .. })] => {
            slot.initializer.as_deref().expect("initializer").clone()
        }
        other => panic!("expected one let declaration, got {other:#?}"),
    }
}

#[test]
fn core_bootstrap_installs_resolvable_symbol_objects() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let context = world.package_context();
    let capability = world.snapshot().capability();

    for name in ["struct", "assert", "uint8"] {
        let symbol = capability
            .resolve_str(name, &context)
            .expect("resolve core symbol through default mount");
        assert_eq!(symbol.name, name);
        assert!(symbol.id.as_u64() > 0);
    }

    let struct_symbol = capability.resolve_str("struct", &context).unwrap();
    assert_eq!(struct_symbol.kind, SymbolKind::MetaFunction);
    assert!(matches!(
        struct_symbol.payload,
        SymbolPayload::MetaFunction(_)
    ));

    let full_path = capability
        .resolve_str("struct::core", &world.root_context())
        .expect("resolve core path through graph");
    assert_eq!(full_path.id, struct_symbol.id);
}

#[test]
fn resolver_handles_short_and_explicit_mounted_core_paths() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let context = world.package_context();
    let capability = world.snapshot().capability();

    let uint8_short = capability.resolve_str("uint8", &context).unwrap();
    let uint8_explicit = capability.resolve_str("uint8::core", &context).unwrap();
    assert_eq!(uint8_short.id, uint8_explicit.id);
    assert_eq!(uint8_explicit.kind, SymbolKind::Type);

    let struct_short = capability.resolve_str("struct", &context).unwrap();
    let struct_explicit = capability.resolve_str("struct::core", &context).unwrap();
    assert_eq!(struct_short.id, struct_explicit.id);
    assert_eq!(struct_explicit.kind, SymbolKind::MetaFunction);

    let diagnostic = capability
        .resolve_str("Missing::core", &context)
        .expect_err("explicit mounted path should fail when target is absent");
    assert!(diagnostic.message.contains("Missing::core"));
}

#[test]
fn resolver_reports_current_namespace_conflict_with_default_mount() {
    let project = TempProject::new("core_conflict");
    project.write("src/main.lang", "let uint8 = uint8");
    let world = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect("build world");

    let diagnostic = world
        .resolve("uint8")
        .expect_err("local short name colliding with core short name is a hard conflict");
    assert!(diagnostic.message.contains("conflicting symbol `uint8`"));
    assert_eq!(diagnostic.severity, DiagnosticSeverity::HardError);
}

#[test]
fn dependency_mount_placeholders_are_visible_as_explicit_paths() {
    let mut manifest = empty_app_manifest();
    manifest.dependency_mounts.push(
        NamespaceMount::synthetic_root("std", vec!["std".to_string()])
            .with_symbol("Vec", SymbolKind::Placeholder),
    );

    let world = CompilationWorld::from_manifest(&manifest).expect("build world with mount");
    let std_symbol = world
        .snapshot()
        .capability()
        .resolve_str("std", &world.package_context())
        .expect("mounted root visible");
    assert_eq!(std_symbol.kind, SymbolKind::Namespace);
    assert_eq!(std_symbol.source_category, SourceCategory::DependencyMount);

    let vec_symbol = world
        .snapshot()
        .capability()
        .resolve_str("Vec::std", &world.package_context())
        .expect("synthetic mounted child visible through explicit path");
    assert_eq!(vec_symbol.name, "Vec");
    assert_eq!(vec_symbol.source_category, SourceCategory::DependencyMount);

    assert!(world
        .snapshot()
        .capability()
        .resolve_str("Vec::mylib", &world.package_context())
        .is_err());
}

#[test]
fn duplicate_dependency_mount_root_is_hard_error() {
    let mut manifest = empty_app_manifest();
    manifest
        .dependency_mounts
        .push(NamespaceMount::synthetic_root(
            "dup",
            vec!["core".to_string()],
        ));

    let error = CompilationWorld::from_manifest(&manifest)
        .expect_err("dependency mount colliding with core root");
    assert!(error.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == DiagnosticSeverity::HardError
            && diagnostic.message.contains("duplicate mount root `core`")
    }));
}

#[test]
fn source_collection_uses_directories_not_file_names() {
    let project = TempProject::new("source_collection");
    project.write(
        "src/math/vector/a.lang",
        "let T: type = (uint8 a, uint8 b) |> struct",
    );
    project.write("src/math/vector/b.lang", "let U: type = uint8");

    let world = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect("build world");
    assert_eq!(world.source_fragments().len(), 2);
    assert_eq!(
        world.source_fragments()[0].namespace,
        world.source_fragments()[1].namespace
    );

    let root_context = world.root_context();
    let capability = world.snapshot().capability();
    let vector = capability
        .resolve_str("vector::math::app", &root_context)
        .expect("directory path contributes namespace skeleton");
    assert_eq!(vector.kind, SymbolKind::Namespace);
    assert_eq!(vector.node_kind, Some(NamespaceNodeKind::Physical));

    assert!(
        capability
            .resolve_str("a::vector::math::app", &root_context)
            .is_err(),
        "implementation file name must not contribute a namespace segment"
    );
    assert!(
        capability
            .resolve_str("T::vector::math::app", &root_context)
            .is_ok(),
        "source fragment declarations contribute direct children"
    );
}

#[test]
fn source_level_import_use_include_module_remain_ordinary_expressions() {
    let project = TempProject::new("no_import_syntax");
    project.write("src/main.lang", "import x; use y; include z; module w");

    let world = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect("build world");
    assert!(
        world.resolve("import").is_err(),
        "source-level import syntax must not create an import symbol"
    );
    assert!(world.resolve("use").is_err());
    assert!(world.resolve("include").is_err());
    assert!(world.resolve("module").is_err());
}

#[test]
fn struct_meta_creates_type_object_and_field_namespaces() {
    let project = TempProject::new("struct_meta");
    project.write(
        "src/main.lang",
        "let T: type = (uint8 a, uint8 b) |> struct",
    );

    let world = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect("build world");
    let type_symbol = world.resolve("T").expect("resolve generated type");
    assert_eq!(type_symbol.kind, SymbolKind::Type);
    let SymbolPayload::Type(type_object) = &type_symbol.payload else {
        panic!("expected type payload, got {type_symbol:#?}");
    };
    assert_eq!(type_object.field_names, ["a", "b"]);
    assert_eq!(type_object.fields.len(), 2);
    assert!(type_object.type_associated_namespace.is_some());

    for path in [
        "a::T",
        "a::ref::T",
        "a::share::T",
        "b::T",
        "b::ref::T",
        "b::share::T",
    ] {
        let symbol = world
            .resolve(path)
            .expect("resolve generated field function");
        assert_eq!(symbol.kind, SymbolKind::FieldFunction, "{path}");
        assert!(matches!(symbol.payload, SymbolPayload::FieldFunction(_)));
    }
}

#[test]
fn type_associated_namespace_paths_have_expected_payloads() {
    let project = TempProject::new("type_associated_paths");
    project.write(
        "src/main.lang",
        "let T: type = (uint8 a, uint8 b) |> struct",
    );

    let world = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect("build world");
    let type_symbol = world.resolve("T").expect("resolve T");
    let SymbolPayload::Type(type_object) = &type_symbol.payload else {
        panic!("expected type object");
    };
    let type_namespace = type_object
        .type_associated_namespace
        .expect("type-associated namespace");

    let value_field = world.resolve("a::T").expect("resolve value field");
    let SymbolPayload::FieldFunction(value_field_object) = &value_field.payload else {
        panic!("expected value field payload");
    };
    assert_eq!(value_field_object.owner_type_symbol_id, type_symbol.id);
    assert_eq!(
        value_field_object.field_type_symbol_id,
        type_object.fields[0].type_symbol_id
    );
    assert_eq!(value_field_object.projection, FieldProjection::Value);

    let ref_namespace = world.resolve("ref::T").expect("resolve ref namespace");
    assert_eq!(ref_namespace.kind, SymbolKind::Namespace);
    assert_eq!(ref_namespace.parent, Some(type_namespace));

    let ref_field = world.resolve("a::ref::T").expect("resolve ref field");
    let SymbolPayload::FieldFunction(ref_field_object) = &ref_field.payload else {
        panic!("expected ref field payload");
    };
    assert_eq!(ref_field_object.owner_type_symbol_id, type_symbol.id);
    assert_eq!(ref_field_object.projection, FieldProjection::Ref);

    let share_field = world.resolve("a::share::T").expect("resolve share field");
    let SymbolPayload::FieldFunction(share_field_object) = &share_field.payload else {
        panic!("expected share field payload");
    };
    assert_eq!(share_field_object.owner_type_symbol_id, type_symbol.id);
    assert_eq!(share_field_object.projection, FieldProjection::Share);
}

#[test]
fn conflict_is_hard_error_and_blocks_installation() {
    let project = TempProject::new("conflict");
    project.write("src/T/placeholder.lang", "");
    project.write("src/main.lang", "let T: type = uint8");

    let error = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect_err("physical directory and declared symbol must conflict");
    assert!(error.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == DiagnosticSeverity::HardError
            && diagnostic.message.contains("conflict")
    }));
}

#[test]
fn ordinary_parent_to_descendant_injection_is_rejected() {
    let project = TempProject::new("descendant_injection");
    project.write("src/main.lang", "let a::T = uint8");

    let error = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect_err("ordinary file contribution cannot inject descendants");
    assert!(error.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("ordinary parent-to-descendant injection")));
}

#[test]
fn ordinary_source_contribution_rejects_deep_and_pattern_binders() {
    for (case_name, source, expected) in [
        (
            "deep_descendant",
            "let a::b::T = uint8",
            "ordinary parent-to-descendant injection",
        ),
        (
            "product_binder",
            "let (a, b) = uint8",
            "unsupported top-level declaration binder",
        ),
        (
            "discard_binder",
            "let _ = uint8",
            "ordinary parent-to-descendant injection",
        ),
    ] {
        let project = TempProject::new(case_name);
        project.write("src/main.lang", source);
        let error = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
            .expect_err("unsupported contribution should fail");
        assert!(
            error
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "missing expected diagnostic {expected:?}: {:#?}",
            error.diagnostics
        );
    }

    let project = TempProject::new("direct_child");
    project.write("src/main.lang", "let T: type = uint8");
    let world = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect("direct child contribution works");
    let symbol = world.resolve("T").expect("resolve direct child type");
    assert_eq!(symbol.kind, SymbolKind::Type);
}

#[test]
fn missing_core_mount_is_a_build_error() {
    let mut manifest = empty_app_manifest();
    manifest.default_core_mount = false;

    let error = CompilationWorld::from_manifest(&manifest).expect_err("missing core mount");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("missing mount")));
}

#[test]
fn delta_transaction_installs_all_or_nothing_and_retains_diagnostics() {
    let snapshot = NamespaceGraphSnapshot::new();
    let root = snapshot.root_node();
    let mut delta = snapshot.empty_delta();
    let a = delta.allocate_symbol_id();
    let b = delta.allocate_symbol_id();
    delta.insert_symbol(
        root,
        SymbolObject::placeholder(
            a,
            "A",
            SymbolKind::Placeholder,
            SourceCategory::DeclaredSymbol,
            Some(root),
            Provenance::new("test A"),
        ),
    );
    delta.insert_symbol(
        root,
        SymbolObject::placeholder(
            b,
            "B",
            SymbolKind::Placeholder,
            SourceCategory::DeclaredSymbol,
            Some(root),
            Provenance::new("test B"),
        ),
    );
    let snapshot = snapshot.install_delta(delta).expect("install full delta");
    let context = ResolverContext::new(root);
    assert!(snapshot.capability().resolve_str("A", &context).is_ok());
    assert!(snapshot.capability().resolve_str("B", &context).is_ok());

    let mut conflict = snapshot.empty_delta();
    let x1 = conflict.allocate_symbol_id();
    let x2 = conflict.allocate_symbol_id();
    conflict.insert_symbol(
        root,
        SymbolObject::placeholder(
            x1,
            "X",
            SymbolKind::Placeholder,
            SourceCategory::DeclaredSymbol,
            Some(root),
            Provenance::new("test X1"),
        ),
    );
    conflict.insert_symbol(
        root,
        SymbolObject::placeholder(
            x2,
            "X",
            SymbolKind::Placeholder,
            SourceCategory::DeclaredSymbol,
            Some(root),
            Provenance::new("test X2"),
        ),
    );
    let error = snapshot
        .install_delta(conflict)
        .expect_err("conflicting delta must fail");
    assert!(!error.diagnostics.is_empty());
    assert!(
        snapshot.capability().resolve_str("X", &context).is_err(),
        "failed delta must not install partial symbols"
    );
}

#[test]
fn conflicting_delta_with_valid_symbol_installs_nothing() {
    let snapshot = NamespaceGraphSnapshot::new();
    let root = snapshot.root_node();
    let mut initial = snapshot.empty_delta();
    let existing_b = initial.allocate_symbol_id();
    initial.insert_symbol(
        root,
        placeholder_symbol(existing_b, root, "B", "existing B"),
    );
    let snapshot = snapshot.install_delta(initial).expect("install existing B");

    let mut delta = snapshot.empty_delta();
    let a = delta.allocate_symbol_id();
    let conflicting_b = delta.allocate_symbol_id();
    delta.insert_symbol(root, placeholder_symbol(a, root, "A", "valid A"));
    delta.insert_symbol(
        root,
        placeholder_symbol(conflicting_b, root, "B", "conflicting B"),
    );

    let error = snapshot
        .install_delta(delta)
        .expect_err("conflicting B rejects whole delta");
    assert!(error.diagnostics.iter().any(|diagnostic| {
        diagnostic.message.contains("B")
            && diagnostic
                .provenance
                .as_ref()
                .is_some_and(|provenance| provenance.description.contains("conflicting B"))
    }));
    let context = ResolverContext::new(root);
    assert!(snapshot.capability().resolve_str("A", &context).is_err());
    assert_eq!(
        snapshot.capability().resolve_str("B", &context).unwrap().id,
        existing_b
    );
}

#[test]
fn delta_with_missing_parent_or_duplicate_link_installs_nothing() {
    let snapshot = NamespaceGraphSnapshot::new();
    let root = snapshot.root_node();

    let mut missing_parent = snapshot.empty_delta();
    let orphan = missing_parent.allocate_symbol_id();
    missing_parent.insert_symbol(
        NamespaceNodeId(99_999),
        placeholder_symbol(orphan, NamespaceNodeId(99_999), "orphan", "missing parent"),
    );
    let error = snapshot
        .install_delta(missing_parent)
        .expect_err("missing parent rejects delta");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("parent namespace node")));
    assert!(snapshot
        .capability()
        .resolve_str("orphan", &ResolverContext::new(root))
        .is_err());

    let mut duplicate_link = snapshot.empty_delta();
    let symbol_id = duplicate_link.allocate_symbol_id();
    let symbol = placeholder_symbol(symbol_id, root, "dup", "duplicate link");
    duplicate_link.symbols.insert(symbol_id, symbol);
    duplicate_link.child_links.push(ChildLink {
        parent: root,
        name: "dup".to_string(),
        symbol: symbol_id,
        provenance: Provenance::new("first duplicate link"),
    });
    duplicate_link.child_links.push(ChildLink {
        parent: root,
        name: "dup".to_string(),
        symbol: symbol_id,
        provenance: Provenance::new("second duplicate link"),
    });
    let error = snapshot
        .install_delta(duplicate_link)
        .expect_err("duplicate child link rejects delta");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("duplicate symbol `dup`")));
    assert!(snapshot
        .capability()
        .resolve_str("dup", &ResolverContext::new(root))
        .is_err());
}

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
fn generated_type_delta_conflict_installs_no_generated_fields() {
    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let mut initial = world.snapshot().empty_delta();
    let existing_t = initial.allocate_symbol_id();
    initial.insert_symbol(
        world.package_root_node(),
        placeholder_symbol(
            existing_t,
            world.package_root_node(),
            "T",
            "preexisting T conflict",
        ),
    );
    let snapshot = world
        .snapshot()
        .install_delta(initial)
        .expect("install existing T");

    let initializer = initializer_from_source("let T: type = (uint8 a) |> struct");
    let expansion = try_expand_early_meta_initializer(
        &snapshot,
        world.package_root_node(),
        "T",
        &initializer,
        &world.package_context(),
        Provenance::new("generated T conflict"),
    )
    .expect("meta expansion result")
    .expect("struct expansion");
    let error = snapshot
        .install_delta(expansion.namespace_delta)
        .expect_err("generated type collides with existing T");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("T")));
    assert!(snapshot
        .capability()
        .resolve_str("a::T", &world.package_context())
        .is_err());
}

#[test]
fn early_meta_only_fires_for_meta_function_payloads() {
    let project = TempProject::new("not_meta");
    project.write(
        "src/main.lang",
        "let not_meta = uint8; let T: type = (uint8 a) |> not_meta",
    );
    let world = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect("non-meta target should not panic or expand");
    let symbol = world
        .resolve("T")
        .expect("T is harvested as placeholder type");
    assert_eq!(symbol.kind, SymbolKind::Type);
    assert!(world.resolve("a::T").is_err());

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
    delta.insert_symbol(
        world.package_root_node(),
        SymbolObject::placeholder(
            bad_meta,
            "bad_meta",
            SymbolKind::MetaFunction,
            SourceCategory::DeclaredSymbol,
            Some(world.package_root_node()),
            Provenance::new("bad meta without payload"),
        ),
    );
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
        &ResolverContext::with_mounts(
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

#[test]
fn unknown_and_invalid_struct_inputs_are_hard_errors() {
    let project = TempProject::new("struct_errors");
    project.write("src/unknown.lang", "let T: type = (Nope a) |> struct");
    let error = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect_err("unknown field type");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("unknown struct field type")));

    let project = TempProject::new("struct_invalid");
    project.write("src/invalid.lang", "let T: type = (uint8) |> struct");
    let error = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect_err("invalid field syntax");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("invalid struct syntax")));
}

#[test]
fn struct_checker_accepts_single_and_two_field_forms() {
    for (case_name, source, expected_fields) in [
        (
            "single_field",
            "let T: type = (uint8 a) |> struct",
            vec!["a"],
        ),
        (
            "two_fields",
            "let T: type = (uint8 a, uint8 b) |> struct",
            vec!["a", "b"],
        ),
    ] {
        let project = TempProject::new(case_name);
        project.write("src/main.lang", source);
        let world = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
            .expect("struct form accepted");
        let symbol = world.resolve("T").expect("resolve T");
        let SymbolPayload::Type(type_object) = symbol.payload else {
            panic!("expected Type payload");
        };
        assert_eq!(type_object.field_names, expected_fields);
    }
}

#[test]
fn struct_checker_rejects_non_type_nested_unit_target_and_projection_collisions() {
    for (case_name, source, expected) in [
        (
            "non_type_field",
            "let not_type = uint8; let T: type = (not_type a) |> struct",
            "resolved symbol is not a type",
        ),
        (
            "nested_product",
            "let T: type = ((uint8 a, uint8 b), uint8 c) |> struct",
            "invalid struct syntax",
        ),
        (
            "unit_field",
            "let T: type = (uint8 a,) |> struct",
            "unit field or trailing unit",
        ),
        (
            "target_not_name",
            "let T: type = (uint8 (a, b)) |> struct",
            "expected a field binder name",
        ),
        (
            "field_ref",
            "let T: type = (uint8 ref) |> struct",
            "conflicts with v0.6 projection namespace",
        ),
        (
            "field_share",
            "let T: type = (uint8 share) |> struct",
            "conflicts with v0.6 projection namespace",
        ),
        (
            "operator_private_syntax",
            "let T: type = (uint8 a * uint8 b + uint8 c) |> struct",
            "invalid struct syntax",
        ),
    ] {
        let project = TempProject::new(case_name);
        project.write("src/main.lang", source);
        let error = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
            .expect_err("invalid struct input rejected");
        assert!(
            error
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "missing {expected:?} in {:#?}",
            error.diagnostics
        );
    }
}

#[test]
fn symbols_with_same_name_in_different_namespaces_have_distinct_ids() {
    let project = TempProject::new("identity");
    project.write("src/left/main.lang", "let T: type = uint8");
    project.write("src/right/main.lang", "let T: type = uint8");
    let world = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect("build world");
    let left = world
        .snapshot()
        .capability()
        .resolve_str("T::left::app", &world.root_context())
        .expect("left T");
    let right = world
        .snapshot()
        .capability()
        .resolve_str("T::right::app", &world.root_context())
        .expect("right T");
    assert_eq!(left.name, right.name);
    assert_ne!(left.id, right.id);
    assert!(left.diagnostic_label().contains("symbol#"));
    assert!(left.diagnostic_label().contains("T"));
}

#[test]
fn representative_diagnostics_contain_useful_text_and_provenance() {
    let project = TempProject::new("diagnostic_conflict");
    project.write("src/T/placeholder.lang", "");
    project.write("src/main.lang", "let T: type = uint8");
    let error = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect_err("conflict");
    assert!(error.diagnostics.iter().any(|diagnostic| {
        diagnostic.message.contains("conflict")
            && diagnostic.provenance.as_ref().is_some_and(|provenance| {
                !provenance.description.is_empty()
                    || provenance.file.is_some()
                    || provenance.span.is_some()
            })
    }));

    let world = CompilationWorld::from_manifest(&empty_app_manifest()).expect("build world");
    let unresolved = world
        .resolve("Nope::core")
        .expect_err("unresolved explicit path");
    assert!(unresolved.message.contains("Nope::core"));

    let project = TempProject::new("diagnostic_descendant");
    project.write("src/main.lang", "let a::T = uint8");
    let error = CompilationWorld::from_manifest(&app_manifest(&project.path().join("src")))
        .expect_err("descendant injection");
    assert!(error.diagnostics.iter().any(|diagnostic| {
        diagnostic.message.contains("parent-to-descendant")
            && diagnostic
                .provenance
                .as_ref()
                .is_some_and(|provenance| provenance.span.is_some())
    }));
}

#[test]
fn policy_metadata_slots_are_preserved_without_policy_checking() {
    let snapshot = NamespaceGraphSnapshot::new();
    let root = snapshot.root_node();
    let mut delta = snapshot.capability().declare(
        root,
        "policy_symbol",
        SymbolKind::Placeholder,
        SourceCategory::DeclaredSymbol,
        Provenance::new("policy test"),
    );
    let symbol = delta
        .symbols
        .values_mut()
        .next()
        .expect("declared symbol in delta");
    symbol
        .policy_metadata
        .slots
        .insert("entry".to_string(), "compile".to_string());

    let snapshot = snapshot
        .install_delta(delta)
        .expect("install policy symbol");
    let symbol = snapshot
        .capability()
        .resolve_str("policy_symbol", &ResolverContext::new(root))
        .expect("resolve policy symbol");
    assert_eq!(
        symbol
            .policy_metadata
            .slots
            .get("entry")
            .map(String::as_str),
        Some("compile")
    );
    assert!(symbol.visibility_metadata.slots.is_empty());
}
