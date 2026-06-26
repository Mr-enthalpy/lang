//! Physical source discovery layer for the v0.6 build slice.
//!
//! This module is the lowest physical input layer below namespace graph
//! assembly. It finds `.lang` source files on the filesystem and records their
//! physical identity, relative namespace directory, raw UTF-8 content, a stable
//! content fingerprint, and diagnostic provenance.
//!
//! Discovery is intentionally non-semantic. It does **not** parse, normalize,
//! resolve names, check types, check policy, expand meta-functions, forward
//! aliases, handle imports, or solve package dependencies. Those concerns live
//! in later layers:
//!
//! ```text
//! physical discovery   (this module)
//!   -> namespace assembly       (world.rs: opens physical namespace nodes)
//!   -> declaration harvesting   (world.rs: installs symbols / deltas)
//!   -> resolver / policy / meta  (consume the namespace graph)
//! ```
//!
//! Semantic rule preserved here: physical directory structure contributes
//! namespace skeleton components; file names never contribute namespace
//! segments. A `.lang` file is a source fragment located inside a physical
//! namespace directory, not a module.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::manifest::SourceRoot;
use crate::model::{Diagnostic, DiagnosticSeverity, Provenance};

/// Stable diagnostic prefix for discovery-originated hard errors.
const DISCOVERY_ERROR_PREFIX: &str = "source discovery error:";

/// Configuration for the physical source discovery layer.
///
/// Source roots remain API-level for v0.6: there is intentionally no manifest
/// file parser. Discovery consumes already-structured source-root requests and
/// reads the filesystem.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SourceDiscoveryConfig {
    pub roots: Vec<SourceRootRequest>,
}

/// A single configured physical source root to scan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceRootRequest {
    pub root_index: usize,
    pub declared_path: PathBuf,
    pub namespace_root: Vec<String>,
}

/// A source root that was successfully canonicalized and confirmed to be a
/// readable directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveredSourceRoot {
    pub root_index: usize,
    pub declared_path: PathBuf,
    pub canonical_path: PathBuf,
    pub namespace_root: Vec<String>,
    pub provenance: Provenance,
}

/// A single discovered `.lang` source file.
///
/// Each unit records all physical-identity concepts distinctly so later layers
/// never have to re-derive them from a path string:
///
/// ```text
/// source_root_index   -- which configured root produced this unit
/// canonical_path      -- canonical physical identity (duplicate detection)
/// root_relative_path  -- path relative to the canonical source root
/// namespace_dir       -- relative directory namespace components (no file name)
/// fragment_name       -- file/fragment identity only (never a namespace segment)
/// content_hash        -- deterministic physical content fingerprint
/// content             -- validated UTF-8 source text
/// provenance          -- diagnostic provenance pointing at the physical file
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveredSourceUnit {
    pub source_root_index: usize,
    pub canonical_path: PathBuf,
    pub root_relative_path: PathBuf,
    pub namespace_dir: Vec<String>,
    pub fragment_name: String,
    pub content_hash: String,
    pub content: String,
    pub provenance: Provenance,
}

/// Result of a discovery pass.
///
/// Discovery accumulates diagnostics instead of failing fast, so it can be
/// tested directly. Callers (e.g. `CompilationWorld::from_manifest`) convert any
/// hard diagnostic into a build error and must not continue into partial
/// namespace assembly when [`SourceDiscoveryReport::has_hard_errors`] is true.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SourceDiscoveryReport {
    pub roots: Vec<DiscoveredSourceRoot>,
    pub units: Vec<DiscoveredSourceUnit>,
    pub diagnostics: Vec<Diagnostic>,
}

impl SourceDiscoveryReport {
    /// Whether any accumulated diagnostic is a hard error.
    pub fn has_hard_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::HardError)
    }
}

impl SourceDiscoveryConfig {
    /// Build a discovery configuration from API-level source roots, assigning a
    /// stable `root_index` by declaration order.
    pub fn from_source_roots(roots: &[SourceRoot]) -> Self {
        Self {
            roots: roots
                .iter()
                .enumerate()
                .map(|(root_index, root)| SourceRootRequest {
                    root_index,
                    declared_path: root.path.clone(),
                    namespace_root: root.namespace_root.clone(),
                })
                .collect(),
        }
    }

    /// Run physical source discovery over all configured roots.
    pub fn discover(&self) -> SourceDiscoveryReport {
        let mut report = SourceDiscoveryReport::default();

        for request in &self.roots {
            if let Some(root) = discover_root(request, &mut report.diagnostics) {
                walk_directory(
                    &root.canonical_path,
                    &root,
                    &mut report.units,
                    &mut report.diagnostics,
                );
                report.roots.push(root);
            }
        }

        // Deterministic order independent of filesystem iteration order: stable
        // lexical ordering over (source root index, normalized relative path
        // components). This drives fragment order, diagnostic order, namespace
        // skeleton construction order, and future cache-key stability.
        report.units.sort_by(|left, right| {
            left.source_root_index
                .cmp(&right.source_root_index)
                .then_with(|| {
                    relative_sort_key(&left.root_relative_path)
                        .cmp(&relative_sort_key(&right.root_relative_path))
                })
        });

        detect_duplicate_identity(&report.units, &mut report.diagnostics);

        report
    }
}

/// Canonicalize and validate a single source root.
fn discover_root(
    request: &SourceRootRequest,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<DiscoveredSourceRoot> {
    let declared = &request.declared_path;

    if !declared.exists() {
        diagnostics.push(Diagnostic::hard_error(
            format!(
                "{DISCOVERY_ERROR_PREFIX} source root `{}` does not exist",
                declared.display()
            ),
            Some(Provenance::file("source root", declared)),
        ));
        return None;
    }

    let canonical = match fs::canonicalize(declared) {
        Ok(canonical) => canonical,
        Err(error) => {
            diagnostics.push(Diagnostic::hard_error(
                format!(
                    "{DISCOVERY_ERROR_PREFIX} source root `{}` cannot be read: {error}",
                    declared.display()
                ),
                Some(Provenance::file("source root", declared)),
            ));
            return None;
        }
    };

    if !canonical.is_dir() {
        diagnostics.push(Diagnostic::hard_error(
            format!(
                "{DISCOVERY_ERROR_PREFIX} source root `{}` is not a directory",
                declared.display()
            ),
            Some(Provenance::file("source root", declared)),
        ));
        return None;
    }

    Some(DiscoveredSourceRoot {
        root_index: request.root_index,
        declared_path: declared.clone(),
        canonical_path: canonical,
        namespace_root: request.namespace_root.clone(),
        provenance: Provenance::file("source root", declared),
    })
}

/// Recursively walk a physical directory, collecting `.lang` source units.
///
/// Symlink policy (conservative v0.6 rule):
///
/// ```text
/// directory symlinks: not followed
/// file symlinks: accepted only if their canonical target remains under the
///                canonical source root (see `collect_lang_file`)
/// ```
///
/// Directory symlinks report `file_type().is_dir() == false`, so the directory
/// recursion branch below naturally skips them.
fn walk_directory(
    directory: &Path,
    root: &DiscoveredSourceRoot,
    units: &mut Vec<DiscoveredSourceUnit>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) => {
            diagnostics.push(Diagnostic::hard_error(
                format!(
                    "{DISCOVERY_ERROR_PREFIX} failed to read source directory `{}`: {error}",
                    directory.display()
                ),
                Some(Provenance::file("source directory", directory)),
            ));
            return;
        }
    };

    // Collect then sort locally so recursion is deterministic even before the
    // final global sort in `discover`. Per-entry read errors are surfaced as
    // hard diagnostics rather than silently dropped.
    let mut children: Vec<PathBuf> = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry) => children.push(entry.path()),
            Err(error) => diagnostics.push(Diagnostic::hard_error(
                format!(
                    "{DISCOVERY_ERROR_PREFIX} failed to read source directory entry in `{}`: {error}",
                    directory.display()
                ),
                Some(Provenance::file("source directory", directory)),
            )),
        }
    }
    children.sort();

    for path in children {
        let file_type = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata.file_type(),
            Err(error) => {
                diagnostics.push(Diagnostic::hard_error(
                    format!(
                        "{DISCOVERY_ERROR_PREFIX} failed to inspect source entry `{}`: {error}",
                        path.display()
                    ),
                    Some(Provenance::file("source entry", &path)),
                ));
                continue;
            }
        };

        if file_type.is_dir() {
            // Real directory only; directory symlinks are intentionally skipped.
            walk_directory(&path, root, units, diagnostics);
        } else if is_lang_file(&path) {
            collect_lang_file(&path, root, units, diagnostics);
        }
        // All other entries (non-`.lang` files, directory symlinks, etc.) are
        // ignored. The file extension decides inclusion.
    }
}

/// Read, validate, and record a single `.lang` file.
fn collect_lang_file(
    path: &Path,
    root: &DiscoveredSourceRoot,
    units: &mut Vec<DiscoveredSourceUnit>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    // Canonicalize for stable physical identity and to enforce root containment.
    // Canonicalization also resolves file symlinks to their real target.
    let canonical = match fs::canonicalize(path) {
        Ok(canonical) => canonical,
        Err(error) => {
            diagnostics.push(Diagnostic::hard_error(
                format!(
                    "{DISCOVERY_ERROR_PREFIX} failed to canonicalize source file `{}`: {error}",
                    path.display()
                ),
                Some(Provenance::file("source file", path)),
            ));
            return;
        }
    };

    // Root containment: a discovered file must remain under its canonical root.
    // For file symlinks this rejects targets that escape the source root.
    let root_relative_path = match canonical.strip_prefix(&root.canonical_path) {
        Ok(relative) => relative.to_path_buf(),
        Err(_) => {
            diagnostics.push(Diagnostic::hard_error(
                format!(
                    "{DISCOVERY_ERROR_PREFIX} source file `{}` escapes its source root `{}`",
                    canonical.display(),
                    root.canonical_path.display()
                ),
                Some(Provenance::file("source file", &canonical)),
            ));
            return;
        }
    };

    let bytes = match fs::read(&canonical) {
        Ok(bytes) => bytes,
        Err(error) => {
            diagnostics.push(Diagnostic::hard_error(
                format!(
                    "{DISCOVERY_ERROR_PREFIX} failed to read source file `{}`: {error}",
                    canonical.display()
                ),
                Some(Provenance::file("source file", &canonical)),
            ));
            return;
        }
    };

    // `.lang` files must be valid UTF-8. Never lossy-decode and never panic.
    let content = match String::from_utf8(bytes) {
        Ok(content) => content,
        Err(_) => {
            diagnostics.push(Diagnostic::hard_error(
                format!(
                    "{DISCOVERY_ERROR_PREFIX} source file `{}` is not valid UTF-8",
                    canonical.display()
                ),
                Some(Provenance::file("source file", &canonical)),
            ));
            return;
        }
    };

    let fragment_name = match canonical.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => {
            diagnostics.push(Diagnostic::hard_error(
                format!(
                    "{DISCOVERY_ERROR_PREFIX} source file `{}` has no file name",
                    canonical.display()
                ),
                Some(Provenance::file("source file", &canonical)),
            ));
            return;
        }
    };

    // Directory components contribute namespace path; the file name never does.
    let mut namespace_dir = Vec::new();
    if let Some(parent) = root_relative_path.parent() {
        for component in parent.components() {
            let raw = component.as_os_str().to_string_lossy();
            if raw.is_empty() {
                continue;
            }
            match namespace_component_from_dir_name(&raw, &canonical) {
                Ok(name) => namespace_dir.push(name),
                Err(diagnostic) => {
                    diagnostics.push(diagnostic);
                    return;
                }
            }
        }
    }

    let content_hash = content_fingerprint(content.as_bytes());
    let provenance = Provenance::file("source fragment", &canonical);

    units.push(DiscoveredSourceUnit {
        source_root_index: root.root_index,
        canonical_path: canonical,
        root_relative_path,
        namespace_dir,
        fragment_name,
        content_hash,
        content,
        provenance,
    });
}

/// Detect identical canonical paths discovered more than once.
///
/// Two configured source roots, or file symlinks pointing at the same target,
/// can surface the same canonical `.lang` file twice. This is rejected rather
/// than silently duplicated or silently de-duplicated, because future
/// incremental caching depends on unambiguous physical identity.
fn detect_duplicate_identity(units: &[DiscoveredSourceUnit], diagnostics: &mut Vec<Diagnostic>) {
    let mut seen: BTreeMap<&Path, ()> = BTreeMap::new();
    for unit in units {
        if seen.insert(unit.canonical_path.as_path(), ()).is_some() {
            diagnostics.push(Diagnostic::hard_error(
                format!(
                    "{DISCOVERY_ERROR_PREFIX} duplicate physical source identity `{}` discovered more than once",
                    unit.canonical_path.display()
                ),
                Some(unit.provenance.clone()),
            ));
        }
    }
}

/// Provisional v0.6 namespace component validation.
///
/// Directory names that contribute namespace path components must be valid
/// ordinary name components. Full lexical reuse from the parser is not wired in
/// yet, so this uses a narrow, provisional ASCII rule: the first character must
/// be an ASCII letter or `_`, and the rest ASCII alphanumeric or `_`. This is
/// deliberately conservative and is not the final name rule. It must not invent
/// escaping syntax and must not change parser/lexer behavior.
fn namespace_component_from_dir_name(name: &str, file: &Path) -> Result<String, Diagnostic> {
    let valid = !name.is_empty()
        && name.chars().enumerate().all(|(index, character)| {
            if index == 0 {
                character.is_ascii_alphabetic() || character == '_'
            } else {
                character.is_ascii_alphanumeric() || character == '_'
            }
        });

    if valid {
        Ok(name.to_string())
    } else {
        Err(Diagnostic::hard_error(
            format!(
                "{DISCOVERY_ERROR_PREFIX} physical directory component `{name}` is not a valid namespace name component"
            ),
            Some(Provenance::file("physical directory component", file)),
        ))
    }
}

/// Whether a path is a `.lang` source file by extension.
///
/// The extension alone decides inclusion. Hidden files are not treated
/// specially: a file is included iff it ends exactly in `.lang`.
fn is_lang_file(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension == "lang")
}

/// Normalized lexical sort key over relative path components.
fn relative_sort_key(path: &Path) -> Vec<String> {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .filter(|component| !component.is_empty())
        .collect()
}

/// Deterministic physical content fingerprint (FNV-1a, 64-bit).
///
/// This is a stable, dependency-free fingerprint of the raw source bytes. It is
/// **not** a cryptographic hash and **not** a future cache-validity proof. It is
/// also not a semantic hash of normalized AST. It is recorded for physical
/// identity/provenance only and is not used for cache invalidation yet.
///
/// The digest is delegated to the shared [`crate::fingerprint`] helper; the
/// `fnv1a64:` prefix and output format are preserved exactly.
fn content_fingerprint(bytes: &[u8]) -> String {
    format!("fnv1a64:{}", crate::fingerprint::fnv1a64_hex(bytes))
}
