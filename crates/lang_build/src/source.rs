use std::path::PathBuf;

use lang_syntax::NormProgram;

use crate::model::{Diagnostic, NamespaceNodeId, Provenance};

/// Parsed and normalized source material at an established install root.
#[derive(Clone, Debug)]
pub struct SourceFragment {
    pub path: PathBuf,
    pub namespace: NamespaceNodeId,
    /// Directory-provided navigation, without semantic namespace allocation.
    pub inherited_navigation: Vec<String>,
    pub normalized: NormProgram,
    pub diagnostics: Vec<Diagnostic>,
    pub provenance: Provenance,
}
