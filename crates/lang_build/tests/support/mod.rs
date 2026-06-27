#![allow(dead_code)]

use std::{
    fs,
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

use lang_build::{
    BuildError, BuildManifest, BuildSession, BuildWorkspace, CompilationWorld, NamespaceNodeId,
    NormalizedCallSite, PackageBuildSpec, ProductMaterialRole, Provenance, SourceCategory,
    SourceRoot, StaticDependencySpec, SymbolKind, SymbolObject, SymbolPayload, TypeObject,
};
use lang_syntax::{NormDecl, NormExpr, NormForm};

/// Temporary on-disk tree for boundary-only tests.
///
/// `TempProject` is boundary-only. Ordinary successful build/discovery/early-meta
/// tests must use committed fixtures under `tests/fixtures/workspaces/`. Use
/// `TempProject` only for mutation/cache-invalidation via copied committed
/// fixtures, invalid-filesystem, invalid-bytes, or graph/model boundary tests.
pub struct TempProject {
    root: PathBuf,
}

impl TempProject {
    pub fn new(name: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("lang_build_{name}_{}_{}", process::id(), nanos));
        fs::create_dir_all(&root).expect("create temp project");
        Self { root }
    }

    pub fn path(&self) -> &Path {
        &self.root
    }

    pub fn write_bytes(&self, relative: &str, bytes: &[u8]) {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("fixture parent")).expect("create fixture dirs");
        fs::write(path, bytes).expect("write fixture bytes");
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Boundary-only manifest pointing at a temp source root. Use only for
/// invalid-filesystem boundary tests (missing root / root-is-file / non-UTF-8).
/// Ordinary and static malformed-source tests must use committed fixtures via
/// `fixture_manifest` / `build_fixture_error`.
pub fn boundary_app_manifest(source_root: &Path) -> BuildManifest {
    BuildManifest::single_source_root("app", vec!["app".to_string()], source_root)
}

pub fn empty_app_manifest() -> BuildManifest {
    BuildManifest::new("app", vec!["app".to_string()])
}

// ---------------------------------------------------------------------------
// Repository fixture helpers
//
// Build/discovery/early-meta integration tests point at committed physical
// source trees under `tests/fixtures/workspaces/`, not strings written at test
// time. These helpers still construct API-level `BuildManifest` / `BuildWorkspace`
// values in Rust (there is no manifest-file parser); only the source trees are
// real committed directories.
// ---------------------------------------------------------------------------

pub fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

pub fn fixture_workspace_path(name: &str) -> PathBuf {
    fixture_root().join("workspaces").join(name)
}

pub fn fixture_source_root(workspace: &str, package: &str) -> PathBuf {
    fixture_workspace_path(workspace).join(package).join("src")
}

pub fn fixture_package_spec(workspace: &str, package: &str) -> PackageBuildSpec {
    let mut spec = PackageBuildSpec::new(package, vec![package.to_string()]);
    spec.source_roots.push(SourceRoot {
        path: fixture_source_root(workspace, package),
        namespace_root: vec![package.to_string()],
    });
    spec
}

pub fn fixture_manifest(workspace: &str, package: &str) -> BuildManifest {
    BuildManifest::single_source_root(
        package,
        vec![package.to_string()],
        fixture_source_root(workspace, package),
    )
}

pub fn single_package_fixture(workspace: &str, package: &str) -> BuildWorkspace {
    BuildWorkspace {
        packages: vec![fixture_package_spec(workspace, package)],
    }
}

pub fn static_dependency_chain_fixture() -> BuildWorkspace {
    let workspace = "static_dependency_chain";
    let aaa = fixture_package_spec(workspace, "aaa");
    let corelib = fixture_package_spec(workspace, "corelib");
    let mut math = fixture_package_spec(workspace, "math");
    math.dependencies.push(StaticDependencySpec {
        package: "corelib".to_string(),
        mount_path: vec!["corelib".to_string()],
    });
    let mut app = fixture_package_spec(workspace, "app");
    app.dependencies.push(StaticDependencySpec {
        package: "math".to_string(),
        mount_path: vec!["math".to_string()],
    });
    app.dependencies.push(StaticDependencySpec {
        package: "corelib".to_string(),
        mount_path: vec!["corelib".to_string()],
    });
    // Provided out of topological order to prove the build order is computed.
    BuildWorkspace {
        packages: vec![app, math, corelib, aaa],
    }
}

pub fn dependency_mount_no_import_fixture() -> BuildWorkspace {
    dependency_mount_fixture("dependency_mount_no_import")
}

pub fn dependency_mount_no_import_dep_changed_fixture() -> BuildWorkspace {
    dependency_mount_fixture("dependency_mount_no_import_dep_changed")
}

fn dependency_mount_fixture(workspace: &str) -> BuildWorkspace {
    let dep = fixture_package_spec(workspace, "dep");
    let mut app = fixture_package_spec(workspace, "app");
    app.dependencies.push(StaticDependencySpec {
        package: "dep".to_string(),
        mount_path: vec!["dep".to_string()],
    });
    BuildWorkspace {
        packages: vec![dep, app],
    }
}

/// A package spec with no source roots, for validation-only build-graph tests
/// (graph validation runs before physical source discovery).
pub fn bare_package_spec(name: &str) -> PackageBuildSpec {
    PackageBuildSpec::new(name, vec![name.to_string()])
}

/// Build a single-package fixture workspace through `BuildSession` and return its
/// `CompilationWorld`. Used by early-meta / policy integration tests.
pub fn build_single_fixture_world(workspace: &str, package: &str) -> CompilationWorld {
    let mut session = BuildSession::new();
    let result = session
        .build_workspace(&single_package_fixture(workspace, package))
        .expect("build fixture workspace");
    result
        .artifacts
        .into_iter()
        .next()
        .expect("one fixture artifact")
        .world
}

/// Build a single-package committed fixture workspace that is expected to FAIL,
/// returning the resulting `BuildError`. Used by malformed-source /
/// diagnostic-boundary fixture tests whose source trees intentionally do not
/// build.
pub fn build_fixture_error(workspace: &str, package: &str) -> BuildError {
    CompilationWorld::from_manifest(&fixture_manifest(workspace, package))
        .expect_err("fixture workspace should fail to build")
}

/// Copy a committed fixture workspace into a fresh temp directory so mutation /
/// cache-invalidation tests can start from a real fixture tree and then mutate.
pub fn copy_fixture_workspace_to_temp(fixture: &str, temp_name: &str) -> TempProject {
    let temp = TempProject::new(temp_name);
    copy_dir_recursive(&fixture_workspace_path(fixture), temp.path());
    temp
}

/// Replace one file in a temp workspace with committed fixture bytes.
///
/// Mutation/cache tests use this to copy fixture variants into a temp workspace.
/// The helper does not parse or interpret source contents; no source program is
/// constructed in Rust.
pub fn replace_temp_file_from_fixture(
    temp: &TempProject,
    target_relative: &str,
    fixture: &str,
    fixture_relative: &str,
) {
    let src = fixture_workspace_path(fixture).join(fixture_relative);
    let target = temp.path().join(target_relative);
    fs::create_dir_all(target.parent().expect("target parent")).expect("create target parent");
    fs::copy(src, target).expect("copy fixture variant file");
}

fn copy_dir_recursive(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).expect("create temp copy dir");
    for entry in fs::read_dir(src).expect("read fixture dir") {
        let entry = entry.expect("fixture entry");
        let file_type = entry.file_type().expect("fixture entry type");
        let target = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).expect("copy fixture file");
        }
    }
}

pub fn placeholder_symbol(
    id: lang_build::SymbolId,
    parent: NamespaceNodeId,
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

pub fn namespace_symbol(
    id: lang_build::SymbolId,
    parent: NamespaceNodeId,
    name: &str,
    node_id: NamespaceNodeId,
    provenance: &str,
) -> SymbolObject {
    SymbolObject::namespace(
        id,
        name,
        node_id,
        lang_build::NamespaceNodeKind::Virtual,
        SourceCategory::DeclaredSymbol,
        Some(parent),
        Provenance::new(provenance),
    )
}

pub fn type_with_namespace(
    type_id: lang_build::SymbolId,
    name: &str,
    parent: NamespaceNodeId,
    type_namespace_id: NamespaceNodeId,
    provenance: &str,
) -> SymbolObject {
    let mut symbol = placeholder_symbol(type_id, parent, name, provenance);
    symbol.kind = SymbolKind::Type;
    symbol.node_kind = Some(lang_build::NamespaceNodeKind::Virtual);
    symbol.payload = SymbolPayload::Type(TypeObject {
        type_symbol_id: type_id,
        fields: Vec::new(),
        field_names: Vec::new(),
        field_type_symbol_ids: Vec::new(),
        type_associated_namespace: Some(type_namespace_id),
        provenance: Provenance::new(provenance),
        generation_origin: None,
        layout_slot: None,
        abi_slot: None,
    });
    symbol
}

pub fn initializer_from_source(source: &str) -> NormExpr {
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

// ---------------------------------------------------------------------------
// Normalized fixture analysis helpers
//
// These helpers read committed fixture files, parse and normalize them, then
// extract call sites and argument product shapes. They are the single point
// through which product-shape and candidate-preparation tests reach normalized
// source material. Tests should not manually match `NormForm` / `NormExpr`
// when these helpers provide the needed data.
// ---------------------------------------------------------------------------

/// Parse a committed fixture file and return the single normalized expression.
///
/// Panics if the fixture does not parse without diagnostics, or if the
/// normalized program is not exactly one expression form.
pub fn parse_and_normalize_fixture_expr(path: PathBuf) -> NormExpr {
    let source = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("read fixture {}: {e}", path.display());
    });
    let parsed = lang_syntax::parse(&source);
    assert!(
        parsed.diagnostics.is_empty(),
        "unexpected parse diagnostics:\n{}",
        lang_syntax::dump_diagnostics(&parsed.diagnostics)
    );
    let normalized = lang_syntax::normalize_program(&parsed.program);
    match normalized.forms.as_slice() {
        [NormForm::Expr(expr)] => expr.clone(),
        other => panic!(
            "expected one normalized expression in `{}`, got {other:#?}",
            path.display()
        ),
    }
}

/// Parse a committed fixture file and return the initializer of the single
/// let declaration.
///
/// Panics if the fixture does not parse without diagnostics, or if the
/// normalized program is not exactly one let form with an initializer.
pub fn parse_and_normalize_fixture_let_initializer(path: PathBuf) -> NormExpr {
    let source = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("read fixture {}: {e}", path.display());
    });
    let parsed = lang_syntax::parse(&source);
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
        other => panic!(
            "expected one let declaration in `{}`, got {other:#?}",
            path.display()
        ),
    }
}

/// Convenience path to the v0.8 product fixture directory.
pub fn v08_fixture_path(name: &str) -> PathBuf {
    fixture_root().join("v08").join(name)
}

/// Extract a `NormalizedCallSite` from a committed v0.8 product fixture.
///
/// The fixture is expected to normalize to a single call expression.
pub fn fixture_call_site(name: &str) -> NormalizedCallSite {
    let expr = parse_and_normalize_fixture_expr(v08_fixture_path(name));
    lang_build::extract_single_call_site(&expr)
        .unwrap_or_else(|diagnostic| panic!("fixture `{name}` is not a call: {diagnostic:?}"))
}

/// Produce an `ArgProductShape` from a committed v0.8 product fixture.
///
/// The fixture is expected to normalize to a single call expression.
/// The call site source product is wrapped in a `ProductObject` with the
/// given role and shaped.
pub fn fixture_arg_product_shape(
    name: &str,
    role: ProductMaterialRole,
) -> lang_build::ArgProductShape {
    let site = fixture_call_site(name);
    site.to_arg_product_shape(role)
}

/// Build the v0.8 candidate fixture world (`v08_candidate` / `app`).
pub fn v08_candidate_world() -> CompilationWorld {
    build_single_fixture_world("v08_candidate", "app")
}

/// Extract the single `NormalizedCallSite` from the v0.8 candidate fixture's
/// let initializer (`let T: type = (<product>) |> struct`).
pub fn v08_candidate_call_site() -> NormalizedCallSite {
    let expr = parse_and_normalize_fixture_let_initializer(
        fixture_source_root("v08_candidate", "app").join("main.lang"),
    );
    lang_build::extract_single_call_site(&expr).unwrap_or_else(|diagnostic| {
        panic!("v08 candidate initializer is not a call: {diagnostic:?}")
    })
}
