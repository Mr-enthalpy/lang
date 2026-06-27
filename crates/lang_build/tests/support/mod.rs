#![allow(dead_code)]

use std::{
    fs,
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

use lang_build::{
    BuildError, BuildManifest, BuildSession, BuildWorkspace, CompilationWorld, NamespaceNodeId,
    PackageBuildSpec, Provenance, SourceCategory, SourceRoot, StaticDependencySpec, SymbolKind,
    SymbolObject, SymbolPayload, TypeObject,
};
use lang_syntax::{NormDecl, NormExpr, NormForm};

/// Temporary on-disk source tree for boundary-only tests.
///
/// `TempProject` is boundary-only. Ordinary successful build/discovery/early-meta
/// tests must use committed fixtures under `tests/fixtures/workspaces/`. Use
/// `TempProject` only for mutation/cache-invalidation, invalid-filesystem,
/// invalid-bytes, malformed-source, or graph/model boundary tests.
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

    /// Writes a temp source file for boundary/mutation tests only. Do not use for
    /// ordinary build success-path or static malformed-source tests; use committed
    /// fixtures under `tests/fixtures/workspaces/`.
    pub fn write_boundary_source(&self, relative: &str, source: &str) {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("fixture parent")).expect("create fixture dirs");
        fs::write(path, source).expect("write fixture");
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
    let workspace = "dependency_mount_no_import";
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
