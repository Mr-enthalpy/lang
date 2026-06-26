use crate::model::SymbolKind;
use std::path::PathBuf;

/// API-level manifest used by the v0.6 vertical slice.
///
/// There is intentionally no manifest file parser yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildManifest {
    pub package_name: String,
    pub source_roots: Vec<SourceRoot>,
    pub namespace_root: Vec<String>,
    pub dependency_mounts: Vec<NamespaceMount>,
    pub default_core_mount: bool,
}

impl BuildManifest {
    pub fn new(package_name: impl Into<String>, namespace_root: Vec<String>) -> Self {
        Self {
            package_name: package_name.into(),
            source_roots: Vec::new(),
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

/// Filesystem source root mounted into a namespace root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceRoot {
    pub path: PathBuf,
    pub namespace_root: Vec<String>,
}

/// API-level dependency mount placeholder.
///
/// v0.6 can create synthetic mounted roots for tests without package solving.
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

/// Synthetic symbol installed under a dependency mount placeholder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntheticMountSymbol {
    pub name: String,
    pub kind: SymbolKind,
}
