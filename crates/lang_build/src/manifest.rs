use crate::model::SymbolKind;
use std::path::PathBuf;

/// API-level manifest used by the v0.6 vertical slice.
///
/// There is intentionally no manifest file parser yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildManifest {
    pub package_name: String,
    pub source_roots: Vec<SourceRoot>,
    /// Toolchain-owned source inputs installed directly in the source-visible
    /// global implementation space `::`.
    ///
    /// These roots are a typed build authority, not ordinary package roots and
    /// not a prelude/import mechanism.
    pub global_implementation_roots: Vec<ToolchainGlobalSourceRoot>,
    pub namespace_root: Vec<String>,
    pub dependency_mounts: Vec<NamespaceMount>,
    pub default_core_mount: bool,
}

impl BuildManifest {
    pub fn new(package_name: impl Into<String>, namespace_root: Vec<String>) -> Self {
        Self {
            package_name: package_name.into(),
            source_roots: Vec::new(),
            global_implementation_roots: Vec::new(),
            namespace_root,
            dependency_mounts: Vec::new(),
            default_core_mount: true,
        }
    }

    pub fn single_source_root(
        package_name: impl Into<String>,
        namespace_root: Vec<String>,
        path: impl Into<PathBuf>,
    ) -> Self {
        let mut manifest = Self::new(package_name, namespace_root.clone());
        manifest.source_roots.push(SourceRoot {
            path: path.into(),
            namespace_root,
        });
        manifest
    }
}

/// Physical source bundle carrying toolchain global-construction authority.
///
/// Its files still pass through lexer, parser, normalization, and semantic
/// construction.  The type is what authorizes the empty/root install prefix;
/// no empty string/path convention grants that authority to ordinary source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolchainGlobalSourceRoot {
    pub path: PathBuf,
    /// Source-order namespace path under `::`. Empty means direct global
    /// members and is legal only because this carrier owns toolchain
    /// authority.
    pub install_prefix: Vec<String>,
}

impl ToolchainGlobalSourceRoot {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            install_prefix: Vec::new(),
        }
    }

    pub fn under(path: impl Into<PathBuf>, install_prefix: Vec<String>) -> Self {
        Self {
            path: path.into(),
            install_prefix,
        }
    }
}

/// Filesystem source root mounted into a namespace root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceRoot {
    pub path: PathBuf,
    pub namespace_root: Vec<String>,
}

/// Explicit namespace mount supplied by the build manifest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamespaceMount {
    pub from_package: String,
    pub mount_path: Vec<String>,
    pub synthetic_symbols: Vec<SyntheticMountSymbol>,
}

impl NamespaceMount {
    pub fn synthetic_root(from_package: impl Into<String>, mount_path: Vec<String>) -> Self {
        Self {
            from_package: from_package.into(),
            mount_path,
            synthetic_symbols: Vec::new(),
        }
    }

    pub fn with_symbol(mut self, name: impl Into<String>, kind: SymbolKind) -> Self {
        self.synthetic_symbols.push(SyntheticMountSymbol {
            name: name.into(),
            kind,
        });
        self
    }
}

/// Synthetic symbol installed under an explicit namespace mount.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntheticMountSymbol {
    pub name: String,
    pub kind: SymbolKind,
}
