#![allow(dead_code)]

use std::{
    fs,
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

use lang_build::{
    BuildManifest, NamespaceNodeId, Provenance, SourceCategory, SymbolKind, SymbolObject,
    SymbolPayload, TypeObject,
};
use lang_syntax::{NormDecl, NormExpr, NormForm};

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

    pub fn write(&self, relative: &str, source: &str) {
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

pub fn app_manifest(source_root: &Path) -> BuildManifest {
    BuildManifest::single_source_root("app", vec!["app".to_string()], source_root)
}

pub fn empty_app_manifest() -> BuildManifest {
    BuildManifest::new("app", vec!["app".to_string()])
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
