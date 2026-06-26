//! Static build graph, package build metadata, and an exact-key in-memory cache.
//!
//! Note: this is the build-graph module, **not** a Cargo build script (build
//! scripts live at the crate root, not under `src/`).
//!
//! This layer sits above physical source discovery and below any future
//! semantic phases:
//!
//! ```text
//! static package graph
//!   -> deterministic dependency topo order
//!   -> physical source discovery per package
//!   -> package build metadata
//!   -> simple in-memory cache
//!   -> CompilationWorld artifacts
//! ```
//!
//! Scope boundaries for this slice:
//!
//! - Static dependencies are API-level build inputs, not source-language syntax.
//!   The language still has no `import` / `use` / `include` / `module` /
//!   dependency / package declaration syntax.
//! - The graph is explicit and closed: the caller supplies every package; every
//!   dependency edge names another supplied package. There is no registry, no
//!   version solving, and no package discovery from source text.
//! - Static dependencies affect build ordering, package metadata, dependency
//!   fingerprints, and dependency mount metadata. They do **not** import,
//!   export, copy, or remap the dependency package's namespace subtree, symbols,
//!   type-associated namespaces, or generated children into the dependent graph.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::discovery::SourceDiscoveryConfig;
use crate::fingerprint::Fnv1a64;
use crate::graph::BuildError;
use crate::manifest::{BuildManifest, NamespaceMount, SourceRoot};
use crate::model::{Diagnostic, Provenance};
use crate::world::CompilationWorld;

/// Cache format version salt. Bumping this invalidates all cache keys without
/// changing package fingerprints.
const CACHE_FORMAT_VERSION: &str = "lang-build-cache-v0.6-static-graph-1";

/// Stable diagnostic prefix for build-graph hard errors.
const BUILD_GRAPH_ERROR_PREFIX: &str = "build graph error:";

/// A closed set of package build specifications.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildWorkspace {
    pub packages: Vec<PackageBuildSpec>,
}

/// A single package build input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageBuildSpec {
    pub name: String,
    pub namespace_root: Vec<String>,
    pub source_roots: Vec<SourceRoot>,
    pub dependencies: Vec<StaticDependencySpec>,
    pub dependency_mounts: Vec<NamespaceMount>,
    pub default_core_mount: bool,
}

impl PackageBuildSpec {
    pub fn new(name: impl Into<String>, namespace_root: Vec<String>) -> Self {
        Self {
            name: name.into(),
            namespace_root,
            source_roots: Vec::new(),
            dependencies: Vec::new(),
            dependency_mounts: Vec::new(),
            default_core_mount: true,
        }
    }
}

/// A static dependency edge: package `package` mounted at `mount_path`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaticDependencySpec {
    pub package: String,
    pub mount_path: Vec<String>,
}

/// Whether a package build was served from cache.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheStatus {
    Miss,
    Hit,
}

/// Build output for one package.
#[derive(Clone, Debug)]
pub struct PackageBuildArtifact {
    pub package_name: String,
    pub world: CompilationWorld,
    pub metadata: PackageBuildMetadata,
    pub fingerprint: String,
}

/// Build-input / build-output metadata for one package.
///
/// This is build metadata, not semantic typing metadata. It does not record
/// type identities, export sets, ABI, layout, borrow facts, policy conformance,
/// or overload sets; those are later semantic artifacts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageBuildMetadata {
    pub package_name: String,
    pub namespace_root: Vec<String>,
    pub source_roots: Vec<SourceRootMetadata>,
    pub source_units: Vec<SourceUnitBuildMetadata>,
    pub dependencies: Vec<DependencyBuildMetadata>,
    pub cache_key: String,
    pub cache_status: CacheStatus,
    pub diagnostic_count: usize,
}

/// Declared source-root metadata for a package build.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceRootMetadata {
    pub declared_path: PathBuf,
    pub namespace_root: Vec<String>,
}

/// Per-source-unit build metadata derived from physical discovery.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceUnitBuildMetadata {
    pub canonical_path: PathBuf,
    pub root_relative_path: PathBuf,
    pub namespace_dir: Vec<String>,
    pub fragment_name: String,
    pub content_hash: String,
}

/// Per-dependency build metadata, including the dependency artifact fingerprint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DependencyBuildMetadata {
    pub package_name: String,
    pub mount_path: Vec<String>,
    pub dependency_fingerprint: String,
}

/// Result of building a whole workspace.
///
/// `artifacts` are returned in deterministic topological build order.
#[derive(Clone, Debug)]
pub struct BuildResult {
    pub artifacts: Vec<PackageBuildArtifact>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Exact cache-key hit/miss statistics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BuildCacheStats {
    pub hits: usize,
    pub misses: usize,
}

/// Simple in-memory, exact-cache-key package artifact cache.
///
/// There is no disk cache, no serialization, no lockfile, and no incremental
/// invalidation beyond exact cache-key hit/miss.
#[derive(Clone, Debug, Default)]
pub struct BuildCache {
    entries: BTreeMap<String, PackageBuildArtifact>,
    stats: BuildCacheStats,
}

impl BuildCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stats(&self) -> BuildCacheStats {
        self.stats
    }

    fn lookup(&self, cache_key: &str) -> Option<PackageBuildArtifact> {
        self.entries.get(cache_key).cloned()
    }

    fn store(&mut self, cache_key: String, artifact: PackageBuildArtifact) {
        self.entries.insert(cache_key, artifact);
    }

    fn record_hit(&mut self) {
        self.stats.hits += 1;
    }

    fn record_miss(&mut self) {
        self.stats.misses += 1;
    }
}

/// Drives validation, deterministic ordering, per-package builds, and caching.
#[derive(Debug, Default)]
pub struct BuildSession {
    cache: BuildCache,
}

impl BuildSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cache_stats(&self) -> BuildCacheStats {
        self.cache.stats()
    }

    /// Validate the static graph, compute deterministic topological order, and
    /// build every package in dependency order.
    ///
    /// Graph-level hard errors stop the build before any package is built.
    pub fn build_workspace(
        &mut self,
        workspace: &BuildWorkspace,
    ) -> Result<BuildResult, BuildError> {
        validate_workspace(workspace)?;
        let order = topological_order(workspace)?;

        let mut artifacts = Vec::with_capacity(order.len());
        let mut diagnostics = Vec::new();
        let mut fingerprints: BTreeMap<String, String> = BTreeMap::new();

        for &index in &order {
            let spec = &workspace.packages[index];
            let artifact = self.build_package(spec, &fingerprints)?;
            fingerprints.insert(spec.name.clone(), artifact.fingerprint.clone());
            diagnostics.extend(artifact.world.diagnostics().iter().cloned());
            artifacts.push(artifact);
        }

        Ok(BuildResult {
            artifacts,
            diagnostics,
        })
    }

    fn build_package(
        &mut self,
        spec: &PackageBuildSpec,
        built_fingerprints: &BTreeMap<String, String>,
    ) -> Result<PackageBuildArtifact, BuildError> {
        // Discovery runs first: it supplies the physical source-unit identity
        // and content hashes that feed the package key. A cache hit may avoid
        // parse/normalize/namespace assembly, but it must not avoid discovery.
        let report = SourceDiscoveryConfig::from_source_roots(&spec.source_roots).discover();
        if report.has_hard_errors() {
            return Err(BuildError {
                diagnostics: report.diagnostics,
            });
        }

        let source_units: Vec<SourceUnitBuildMetadata> = report
            .units
            .iter()
            .map(|unit| SourceUnitBuildMetadata {
                canonical_path: unit.canonical_path.clone(),
                root_relative_path: unit.root_relative_path.clone(),
                namespace_dir: unit.namespace_dir.clone(),
                fragment_name: unit.fragment_name.clone(),
                content_hash: unit.content_hash.clone(),
            })
            .collect();

        // Dependencies sorted deterministically so fingerprints/metadata are
        // independent of declaration order.
        let mut dependencies: Vec<DependencyBuildMetadata> = spec
            .dependencies
            .iter()
            .map(|dependency| DependencyBuildMetadata {
                package_name: dependency.package.clone(),
                mount_path: dependency.mount_path.clone(),
                dependency_fingerprint: built_fingerprints
                    .get(&dependency.package)
                    .cloned()
                    .unwrap_or_default(),
            })
            .collect();
        dependencies.sort_by(|left, right| {
            left.package_name
                .cmp(&right.package_name)
                .then_with(|| left.mount_path.cmp(&right.mount_path))
        });

        let fingerprint = compute_package_fingerprint(spec, &source_units, &dependencies);
        let cache_key = compute_cache_key(&fingerprint);

        // Cache hit: clone the cached artifact and update only the returned
        // metadata cache_status. The package fingerprint and cache key are
        // unchanged.
        if let Some(mut cached) = self.cache.lookup(&cache_key) {
            cached.metadata.cache_status = CacheStatus::Hit;
            self.cache.record_hit();
            return Ok(cached);
        }

        // Cache miss: actually build the package world.
        let manifest = build_manifest(spec);
        let world = CompilationWorld::from_manifest(&manifest)?;
        let diagnostic_count = world.diagnostics().len();

        let metadata = PackageBuildMetadata {
            package_name: spec.name.clone(),
            namespace_root: spec.namespace_root.clone(),
            source_roots: spec
                .source_roots
                .iter()
                .map(|root| SourceRootMetadata {
                    declared_path: root.path.clone(),
                    namespace_root: root.namespace_root.clone(),
                })
                .collect(),
            source_units,
            dependencies,
            cache_key: cache_key.clone(),
            cache_status: CacheStatus::Miss,
            diagnostic_count,
        };

        let artifact = PackageBuildArtifact {
            package_name: spec.name.clone(),
            world,
            metadata,
            fingerprint,
        };

        self.cache.store(cache_key, artifact.clone());
        self.cache.record_miss();
        Ok(artifact)
    }
}

/// Map a package spec to a lower-level [`BuildManifest`].
///
/// Conservative dependency mounts: for each static dependency we install only
/// the mount namespace marker via existing [`NamespaceMount`] plumbing (an empty
/// synthetic mount). We do not clone, remap, or expose dependency package
/// symbols.
fn build_manifest(spec: &PackageBuildSpec) -> BuildManifest {
    let mut manifest = BuildManifest::new(spec.name.clone(), spec.namespace_root.clone());
    manifest.source_roots = spec.source_roots.clone();
    manifest.default_core_mount = spec.default_core_mount;
    manifest.dependency_mounts = spec.dependency_mounts.clone();
    for dependency in &spec.dependencies {
        manifest
            .dependency_mounts
            .push(NamespaceMount::synthetic_root(
                dependency.package.clone(),
                dependency.mount_path.clone(),
            ));
    }
    manifest
}

fn compute_package_fingerprint(
    spec: &PackageBuildSpec,
    source_units: &[SourceUnitBuildMetadata],
    dependencies: &[DependencyBuildMetadata],
) -> String {
    let mut hasher = Fnv1a64::new();

    hasher.write_str_field("package");
    hasher.write_str_field(&spec.name);
    hasher.write_str_field("namespace_root");
    hasher.write_str_field(&spec.namespace_root.join("::"));
    hasher.write_str_field("default_core_mount");
    hasher.write_field(&[u8::from(spec.default_core_mount)]);

    hasher.write_str_field("source_roots");
    hasher.write_field(&(spec.source_roots.len() as u64).to_le_bytes());
    for root in &spec.source_roots {
        hasher.write_str_field(&root.path.to_string_lossy());
        hasher.write_str_field(&root.namespace_root.join("::"));
    }

    hasher.write_str_field("source_units");
    hasher.write_field(&(source_units.len() as u64).to_le_bytes());
    for unit in source_units {
        hasher.write_str_field(&relative_path_key(&unit.root_relative_path));
        hasher.write_str_field(&unit.namespace_dir.join("::"));
        hasher.write_str_field(&unit.fragment_name);
        hasher.write_str_field(&unit.content_hash);
    }

    hasher.write_str_field("dependencies");
    hasher.write_field(&(dependencies.len() as u64).to_le_bytes());
    for dependency in dependencies {
        hasher.write_str_field(&dependency.package_name);
        hasher.write_str_field(&dependency.mount_path.join("::"));
        hasher.write_str_field(&dependency.dependency_fingerprint);
    }

    hasher.finish_hex()
}

fn compute_cache_key(fingerprint: &str) -> String {
    let mut hasher = Fnv1a64::new();
    hasher.write_str_field(CACHE_FORMAT_VERSION);
    hasher.write_str_field(fingerprint);
    hasher.finish_hex()
}

/// Machine-independent `/`-joined relative path key.
fn relative_path_key(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .filter(|component| !component.is_empty())
        .collect::<Vec<_>>()
        .join("/")
}

fn build_graph_error(message: impl Into<String>) -> Diagnostic {
    Diagnostic::hard_error(message, Some(Provenance::new("static build graph")))
}

/// Validate the static dependency graph (everything except cycle detection).
fn validate_workspace(workspace: &BuildWorkspace) -> Result<(), BuildError> {
    let mut diagnostics = Vec::new();

    let mut known_names: BTreeSet<&str> = BTreeSet::new();
    for package in &workspace.packages {
        if !package.name.is_empty() {
            known_names.insert(package.name.as_str());
        }
    }

    let mut seen_names: BTreeSet<&str> = BTreeSet::new();
    for package in &workspace.packages {
        if package.name.is_empty() {
            diagnostics.push(build_graph_error(format!(
                "{BUILD_GRAPH_ERROR_PREFIX} package name must not be empty"
            )));
        } else if !seen_names.insert(package.name.as_str()) {
            diagnostics.push(build_graph_error(format!(
                "{BUILD_GRAPH_ERROR_PREFIX} duplicate package name `{}`",
                package.name
            )));
        }

        let mut seen_mount_paths: BTreeSet<&[String]> = BTreeSet::new();
        for dependency in &package.dependencies {
            if dependency.package.is_empty() {
                diagnostics.push(build_graph_error(format!(
                    "{BUILD_GRAPH_ERROR_PREFIX} package `{}` has a dependency with an empty package name",
                    package.name
                )));
                continue;
            }
            if dependency.package == package.name {
                diagnostics.push(build_graph_error(format!(
                    "{BUILD_GRAPH_ERROR_PREFIX} package `{}` cannot depend on itself",
                    package.name
                )));
            }
            if !known_names.contains(dependency.package.as_str()) {
                diagnostics.push(build_graph_error(format!(
                    "{BUILD_GRAPH_ERROR_PREFIX} package `{}` depends on unknown package `{}`",
                    package.name, dependency.package
                )));
            }
            if dependency.mount_path.is_empty() {
                diagnostics.push(build_graph_error(format!(
                    "{BUILD_GRAPH_ERROR_PREFIX} package `{}` dependency `{}` has an empty mount path",
                    package.name, dependency.package
                )));
            } else if !seen_mount_paths.insert(dependency.mount_path.as_slice()) {
                diagnostics.push(build_graph_error(format!(
                    "{BUILD_GRAPH_ERROR_PREFIX} package `{}` has duplicate dependency mount path `{}`",
                    package.name,
                    dependency.mount_path.join("::")
                )));
            }
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(BuildError { diagnostics })
    }
}

/// Deterministic topological order (Kahn's algorithm, lexical name tie-break).
///
/// Assumes [`validate_workspace`] already passed: names are unique/non-empty and
/// every dependency names a known package.
fn topological_order(workspace: &BuildWorkspace) -> Result<Vec<usize>, BuildError> {
    let count = workspace.packages.len();
    let mut index_by_name: BTreeMap<&str, usize> = BTreeMap::new();
    for (index, package) in workspace.packages.iter().enumerate() {
        index_by_name.insert(package.name.as_str(), index);
    }

    let mut indegree = vec![0usize; count];
    let mut dependents: Vec<Vec<usize>> = vec![Vec::new(); count];
    for (index, package) in workspace.packages.iter().enumerate() {
        // Multiple mounts of the same dependency count as one ordering edge.
        let mut dependency_indices: BTreeSet<usize> = BTreeSet::new();
        for dependency in &package.dependencies {
            if let Some(&dependency_index) = index_by_name.get(dependency.package.as_str()) {
                dependency_indices.insert(dependency_index);
            }
        }
        for dependency_index in dependency_indices {
            dependents[dependency_index].push(index);
            indegree[index] += 1;
        }
    }

    let mut ready: BTreeSet<(&str, usize)> = BTreeSet::new();
    for index in 0..count {
        if indegree[index] == 0 {
            ready.insert((workspace.packages[index].name.as_str(), index));
        }
    }

    let mut order = Vec::with_capacity(count);
    while let Some((name, index)) = ready.iter().next().copied() {
        ready.remove(&(name, index));
        order.push(index);

        let mut next: Vec<usize> = dependents[index].clone();
        next.sort_by(|&left, &right| {
            workspace.packages[left]
                .name
                .cmp(&workspace.packages[right].name)
        });
        for dependent in next {
            indegree[dependent] -= 1;
            if indegree[dependent] == 0 {
                ready.insert((workspace.packages[dependent].name.as_str(), dependent));
            }
        }
    }

    if order.len() == count {
        Ok(order)
    } else {
        let mut cyclic: Vec<&str> = (0..count)
            .filter(|&index| indegree[index] > 0)
            .map(|index| workspace.packages[index].name.as_str())
            .collect();
        cyclic.sort_unstable();
        Err(BuildError {
            diagnostics: vec![build_graph_error(format!(
                "{BUILD_GRAPH_ERROR_PREFIX} dependency cycle detected involving packages: {}",
                cyclic.join(", ")
            ))],
        })
    }
}
