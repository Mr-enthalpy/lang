use std::path::PathBuf;

use lang_syntax::NormProgram;

use crate::model::{Diagnostic, NamespaceNodeId, Provenance};

/// Parsed and normalized source file fragment attached to a namespace node.
#[derive(Clone, Debug)]
pub struct SourceFragment {
    pub path: PathBuf,
    pub namespace: NamespaceNodeId,
    pub normalized: NormProgram,
    pub diagnostics: Vec<Diagnostic>,
    pub provenance: Provenance,
}
