use std::{
    fs,
    path::{Path, PathBuf},
};

use lang_syntax::{
    norm::NormNavComponent, NormAliasBinder, NormAnnotation, NormDecl, NormForm, NormOrigin,
    NormPattern, NormProgram,
};

use crate::{
    core::install_core_bootstrap,
    graph::{
        namespace_symbol, BuildError, NamespaceGraphSnapshot, ResolveExpectation, ResolverContext,
    },
    manifest::{BuildManifest, NamespaceMount, SourceRoot},
    meta::try_expand_early_meta_initializer,
    model::{
        Diagnostic, DiagnosticSeverity, NamespaceDelta, NamespaceNode, NamespaceNodeId,
        NamespaceNodeKind, Provenance, SourceCategory, SymbolKind, SymbolObject, SymbolPayload,
        TypeObject,
    },
    source::SourceFragment,
};

/// Build/namespace world object for the v0.6 vertical slice.
///
/// This is the canonical holder for source fragments, default core mount, and
/// the namespace graph snapshot used by resolver and early meta.
#[derive(Clone, Debug)]
pub struct CompilationWorld {
    snapshot: NamespaceGraphSnapshot,
    package_root_node: NamespaceNodeId,
    core_node: NamespaceNodeId,
    source_fragments: Vec<SourceFragment>,
    diagnostics: Vec<Diagnostic>,
}

impl CompilationWorld {
    pub fn from_manifest(manifest: &BuildManifest) -> Result<Self, BuildError> {
        if !manifest.default_core_mount {
            return Err(BuildError::single(Diagnostic::hard_error(
                "build manifest error: default core mount is required for v0.6 bootstrap",
                Some(Provenance::new("build manifest")),
            )));
        }

        let snapshot = NamespaceGraphSnapshot::new();
        let (mut snapshot, core_node) = install_core_bootstrap(&snapshot)?;
        let package_root_node =
            ensure_declared_namespace_path(&mut snapshot, &manifest.namespace_root)?;
        install_dependency_mounts(&mut snapshot, &manifest.dependency_mounts)?;

        let mut world = Self {
            snapshot,
            package_root_node,
            core_node,
            source_fragments: Vec::new(),
            diagnostics: Vec::new(),
        };

        for source_root in &manifest.source_roots {
            world.collect_source_root(source_root)?;
        }

        Ok(world)
    }

    pub fn snapshot(&self) -> &NamespaceGraphSnapshot {
        &self.snapshot
    }

    pub fn package_root_node(&self) -> NamespaceNodeId {
        self.package_root_node
    }

    pub fn core_node(&self) -> NamespaceNodeId {
        self.core_node
    }

    pub fn source_fragments(&self) -> &[SourceFragment] {
        &self.source_fragments
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn package_context(&self) -> ResolverContext {
        ResolverContext::with_mounts(
            self.package_root_node,
            vec![self.snapshot.root_node()],
            vec![self.core_node],
        )
    }

    pub fn root_context(&self) -> ResolverContext {
        ResolverContext::with_mounts(
            self.snapshot.root_node(),
            vec![self.snapshot.root_node()],
            vec![self.core_node],
        )
    }

    pub fn resolve(&self, source_order_path: &str) -> Result<SymbolObject, Diagnostic> {
        self.snapshot
            .capability()
            .resolve_str(source_order_path, &self.package_context())
    }

    pub fn resolve_with_expectation(
        &self,
        source_order_path: &str,
        expectation: ResolveExpectation,
    ) -> Result<SymbolObject, Diagnostic> {
        self.snapshot.capability().resolve_str_with_expectation(
            source_order_path,
            &self.package_context(),
            expectation,
        )
    }

    fn collect_source_root(&mut self, source_root: &SourceRoot) -> Result<(), BuildError> {
        let root_namespace =
            ensure_declared_namespace_path(&mut self.snapshot, &source_root.namespace_root)?;
        let directories = collect_directories(&source_root.path)?;

        for directory in directories {
            let relative_components = relative_components(&source_root.path, &directory)?;
            let directory_namespace = ensure_physical_namespace_path(
                &mut self.snapshot,
                root_namespace,
                &relative_components,
                &directory,
            )?;
            self.collect_source_files_in_directory(&directory, directory_namespace)?;
        }

        Ok(())
    }

    fn collect_source_files_in_directory(
        &mut self,
        directory: &Path,
        namespace: NamespaceNodeId,
    ) -> Result<(), BuildError> {
        let mut files = fs::read_dir(directory)
            .map_err(|error| {
                BuildError::single(Diagnostic::hard_error(
                    format!(
                        "failed to read source directory `{}`: {error}",
                        directory.display()
                    ),
                    Some(Provenance::file("source directory", directory)),
                ))
            })?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "lang"))
            .collect::<Vec<_>>();
        files.sort();

        for file in files {
            self.collect_source_file(file, namespace)?;
        }

        Ok(())
    }

    fn collect_source_file(
        &mut self,
        file: PathBuf,
        namespace: NamespaceNodeId,
    ) -> Result<(), BuildError> {
        let source = fs::read_to_string(&file).map_err(|error| {
            BuildError::single(Diagnostic::hard_error(
                format!("failed to read source file `{}`: {error}", file.display()),
                Some(Provenance::file("source file", &file)),
            ))
        })?;
        let parsed = lang_syntax::parse(&source);
        let normalized = lang_syntax::normalize_program(&parsed.program);
        let provenance = Provenance::file("source fragment", &file);
        let diagnostics = parsed
            .diagnostics
            .iter()
            .map(|diagnostic| {
                Diagnostic::new(
                    DiagnosticSeverity::Error,
                    format!(
                        "syntax diagnostic {:?}: {}",
                        diagnostic.code, diagnostic.message
                    ),
                    Some(provenance.clone().with_span(diagnostic.span)),
                )
            })
            .collect::<Vec<_>>();
        self.diagnostics.extend(diagnostics.clone());

        self.harvest_program(namespace, &normalized, &file)?;
        self.source_fragments.push(SourceFragment {
            path: file,
            namespace,
            normalized,
            diagnostics,
            provenance,
        });

        Ok(())
    }

    fn harvest_program(
        &mut self,
        namespace: NamespaceNodeId,
        normalized: &NormProgram,
        file: &Path,
    ) -> Result<(), BuildError> {
        for form in &normalized.forms {
            match form {
                NormForm::Let(decl) => self.harvest_let(namespace, decl, file)?,
                NormForm::Alias(decl) => self.harvest_alias(namespace, decl, file)?,
                NormForm::Expr(_) => {}
                NormForm::Error(error) => {
                    return Err(BuildError::single(Diagnostic::hard_error(
                        "source contribution error: cannot harvest declaration from normalized error form",
                        Some(Provenance::from_norm_origin(
                            "normalized error",
                            &error.origin,
                        )),
                    )));
                }
            }
        }
        Ok(())
    }

    fn harvest_let(
        &mut self,
        namespace: NamespaceNodeId,
        decl: &NormDecl,
        file: &Path,
    ) -> Result<(), BuildError> {
        let NormDecl::Let { slot, origin } = decl else {
            return Ok(());
        };

        let binder_name = match &slot.value_pattern {
            NormPattern::Binder { name, .. } => name.clone(),
            NormPattern::Nav { .. }
            | NormPattern::Sequence { .. }
            | NormPattern::Skeleton { .. } => {
                return Err(BuildError::single(Diagnostic::hard_error(
                    "source contribution error: ordinary parent-to-descendant injection is rejected in file contribution context",
                    Some(Provenance::from_norm_origin(
                        "top-level declaration binder",
                        pattern_origin(&slot.value_pattern),
                    )),
                )));
            }
            _ => {
                return Err(BuildError::single(Diagnostic::hard_error(
                    "source contribution error: unsupported top-level declaration binder in v0.6 vertical slice",
                    Some(Provenance::from_norm_origin(
                        "top-level declaration binder",
                        pattern_origin(&slot.value_pattern),
                    )),
                )));
            }
        };

        let declaration_provenance =
            Provenance::from_norm_origin(format!("declaration `{binder_name}`"), origin);
        let context = ResolverContext::with_mounts(
            namespace,
            vec![self.snapshot.root_node()],
            vec![self.core_node],
        );

        if let Some(initializer) = slot.initializer.as_deref() {
            if let Some(expansion) = try_expand_early_meta_initializer(
                &self.snapshot,
                namespace,
                &binder_name,
                initializer,
                &context,
                declaration_provenance.clone(),
            )? {
                self.snapshot = self
                    .snapshot
                    .install_delta(expansion.namespace_delta)
                    .map_err(BuildError::from)?;
                self.diagnostics.extend(expansion.diagnostics);
                return Ok(());
            }
        }

        let delta = if is_type_annotation(slot.annotation.as_ref()) {
            declared_type_placeholder_delta(
                &self.snapshot,
                namespace,
                &binder_name,
                declaration_provenance,
            )
        } else {
            self.snapshot.capability().declare(
                namespace,
                binder_name,
                SymbolKind::Placeholder,
                SourceCategory::DeclaredSymbol,
                Provenance::file("declared source symbol", file),
            )
        };
        self.snapshot = self
            .snapshot
            .install_delta(delta)
            .map_err(BuildError::from)?;
        Ok(())
    }

    fn harvest_alias(
        &mut self,
        namespace: NamespaceNodeId,
        decl: &NormDecl,
        _file: &Path,
    ) -> Result<(), BuildError> {
        let NormDecl::Alias {
            binder,
            target,
            origin,
            ..
        } = decl
        else {
            return Ok(());
        };

        let name = match binder {
            NormAliasBinder::Name { name, .. } => name.clone(),
            _ => {
                return Err(BuildError::single(Diagnostic::hard_error(
                    "source contribution error: unsupported alias binder in v0.6 vertical slice",
                    Some(Provenance::from_norm_origin("alias binder", origin)),
                )));
            }
        };
        let target_path = target
            .components
            .iter()
            .map(|component| match component {
                NormNavComponent::Name { name, .. } => Ok(name.clone()),
                _ => Err(BuildError::single(Diagnostic::hard_error(
                    "source contribution error: unsupported alias target in v0.6 vertical slice",
                    Some(Provenance::from_norm_origin("alias target", &target.origin)),
                ))),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let context = ResolverContext::with_mounts(
            namespace,
            vec![self.snapshot.root_node()],
            vec![self.core_node],
        );
        let target_symbol = self
            .snapshot
            .capability()
            .resolve(&target_path, &context)
            .map_err(BuildError::single)?;
        let delta = self.snapshot.capability().alias(
            namespace,
            name,
            target_symbol.id,
            Provenance::from_norm_origin("alias declaration", origin),
        );
        self.snapshot = self
            .snapshot
            .install_delta(delta)
            .map_err(BuildError::from)?;
        Ok(())
    }
}

fn ensure_declared_namespace_path(
    snapshot: &mut NamespaceGraphSnapshot,
    components: &[String],
) -> Result<NamespaceNodeId, BuildError> {
    ensure_namespace_path(
        snapshot,
        snapshot.root_node(),
        components,
        NamespaceNodeKind::Declared,
        SourceCategory::DeclaredSymbol,
        "declared namespace mount",
    )
}

fn install_dependency_mounts(
    snapshot: &mut NamespaceGraphSnapshot,
    mounts: &[NamespaceMount],
) -> Result<(), BuildError> {
    for mount in mounts {
        if mount.mount_path.is_empty() {
            return Err(BuildError::single(Diagnostic::hard_error(
                "build manifest error: dependency mount path must not be empty",
                Some(Provenance::new(format!(
                    "dependency mount from `{}`",
                    mount.from_package
                ))),
            )));
        }

        if snapshot
            .capability()
            .resolve_with_expectation(
                &mount.mount_path,
                &ResolverContext::new(snapshot.root_node()),
                ResolveExpectation::NamespaceSubspace,
            )
            .is_ok()
        {
            return Err(BuildError::single(Diagnostic::hard_error(
                format!(
                    "build manifest error: duplicate mount root `{}`",
                    mount.mount_path.join("::")
                ),
                Some(Provenance::new(format!(
                    "dependency mount from `{}`",
                    mount.from_package
                ))),
            )));
        }

        let mount_node = ensure_namespace_path(
            snapshot,
            snapshot.root_node(),
            &mount.mount_path,
            NamespaceNodeKind::Declared,
            SourceCategory::DependencyMount,
            &format!("dependency mount from `{}`", mount.from_package),
        )?;

        for synthetic in &mount.synthetic_symbols {
            let delta = snapshot.capability().declare(
                mount_node,
                &synthetic.name,
                synthetic.kind,
                SourceCategory::DependencyMount,
                Provenance::new(format!(
                    "synthetic symbol `{}` from dependency mount `{}`",
                    synthetic.name, mount.from_package
                )),
            );
            *snapshot = snapshot.install_delta(delta).map_err(BuildError::from)?;
        }
    }
    Ok(())
}

fn ensure_physical_namespace_path(
    snapshot: &mut NamespaceGraphSnapshot,
    root: NamespaceNodeId,
    components: &[String],
    path: &Path,
) -> Result<NamespaceNodeId, BuildError> {
    if components.is_empty() {
        return Ok(root);
    }
    ensure_namespace_path(
        snapshot,
        root,
        components,
        NamespaceNodeKind::Physical,
        SourceCategory::PhysicalDirectory,
        &format!("physical directory `{}`", path.display()),
    )
}

fn ensure_namespace_path(
    snapshot: &mut NamespaceGraphSnapshot,
    root: NamespaceNodeId,
    components: &[String],
    node_kind: NamespaceNodeKind,
    source_category: SourceCategory,
    provenance_description: &str,
) -> Result<NamespaceNodeId, BuildError> {
    let mut current = root;
    for component in components {
        if let Ok(existing) = snapshot.child_symbol_with_expectation(
            current,
            component,
            ResolveExpectation::NamespaceSubspace,
        ) {
            current = existing.namespace_node().ok_or_else(|| {
                BuildError::single(Diagnostic::hard_error(
                    format!("namespace symbol `{component}` has no namespace node"),
                    Some(existing.provenance.clone()),
                ))
            })?;
            continue;
        }

        let mut delta = snapshot.empty_delta();
        let next = namespace_symbol(
            &mut delta,
            current,
            component,
            node_kind,
            source_category,
            Provenance::new(provenance_description),
        );
        *snapshot = snapshot.install_delta(delta).map_err(BuildError::from)?;
        current = next;
    }
    Ok(current)
}

fn declared_type_placeholder_delta(
    snapshot: &NamespaceGraphSnapshot,
    parent: NamespaceNodeId,
    name: &str,
    provenance: Provenance,
) -> NamespaceDelta {
    // v0.6 placeholder: this represents a type-annotated declaration before
    // type-value evaluation exists. Long-term, `let t: type = uint8` is an
    // ordinary binding of symbol/place `t` to the existing type value `uint8`,
    // not fresh type generation and not symbol aliasing. Namespace injection
    // through `t` must target place(t), not place(uint8), once writable-place
    // checking exists.
    //
    // This PR (v0.6.1) does not implement TypeValueId, canonical type-value
    // equality, alias forwarding evaluation, or writable-place checking.
    // The placeholder representation remains until those features land.
    let mut delta = snapshot.empty_delta();
    let type_symbol_id = delta.allocate_symbol_id();
    let type_namespace_id = delta.allocate_node_id();
    delta.insert_node(NamespaceNode::new(
        type_namespace_id,
        format!("{name}<type-associated>"),
        NamespaceNodeKind::Virtual,
        SourceCategory::TypeAssociatedNamespace,
        Some(parent),
        provenance.clone(),
    ));

    let mut symbol = SymbolObject::placeholder(
        type_symbol_id,
        name,
        SymbolKind::Type,
        SourceCategory::DeclaredSymbol,
        Some(parent),
        provenance.clone(),
    );
    symbol.node_kind = Some(NamespaceNodeKind::Virtual);
    symbol.payload = SymbolPayload::Type(TypeObject {
        type_symbol_id,
        fields: Vec::new(),
        field_names: Vec::new(),
        field_type_symbol_ids: Vec::new(),
        type_associated_namespace: Some(type_namespace_id),
        provenance,
        generation_origin: None,
        layout_slot: None,
        abi_slot: None,
    });
    delta.insert_symbol(parent, symbol);
    delta
}

fn collect_directories(root: &Path) -> Result<Vec<PathBuf>, BuildError> {
    let mut directories = Vec::new();
    collect_directories_rec(root, &mut directories)?;
    directories.sort();
    Ok(directories)
}

fn collect_directories_rec(path: &Path, directories: &mut Vec<PathBuf>) -> Result<(), BuildError> {
    if !path.is_dir() {
        return Err(BuildError::single(Diagnostic::hard_error(
            format!("source root `{}` is not a directory", path.display()),
            Some(Provenance::file("source root", path)),
        )));
    }
    directories.push(path.to_path_buf());
    let mut children = fs::read_dir(path)
        .map_err(|error| {
            BuildError::single(Diagnostic::hard_error(
                format!(
                    "failed to read source directory `{}`: {error}",
                    path.display()
                ),
                Some(Provenance::file("source directory", path)),
            ))
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    children.sort();
    for child in children {
        collect_directories_rec(&child, directories)?;
    }
    Ok(())
}

fn relative_components(root: &Path, directory: &Path) -> Result<Vec<String>, BuildError> {
    let relative = directory.strip_prefix(root).map_err(|error| {
        BuildError::single(Diagnostic::hard_error(
            format!(
                "failed to compute source-root relative path `{}` from `{}`: {error}",
                directory.display(),
                root.display()
            ),
            Some(Provenance::file("source directory", directory)),
        ))
    })?;
    Ok(relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .filter(|component| !component.is_empty())
        .collect())
}

fn is_type_annotation(annotation: Option<&NormAnnotation>) -> bool {
    matches!(
        annotation.map(|annotation| &annotation.pattern),
        Some(NormPattern::Name { name, .. }) if name == "type"
    )
}

fn pattern_origin(pattern: &NormPattern) -> &NormOrigin {
    match pattern {
        NormPattern::Binder { origin, .. }
        | NormPattern::OperatorBinder { origin, .. }
        | NormPattern::Product { origin, .. }
        | NormPattern::Unit { origin }
        | NormPattern::HoleRef { origin, .. }
        | NormPattern::Name { origin, .. }
        | NormPattern::Literal { origin, .. }
        | NormPattern::Nav { origin, .. }
        | NormPattern::Sequence { origin, .. }
        | NormPattern::Skeleton { origin, .. }
        | NormPattern::BindingSlot { origin, .. }
        | NormPattern::Unsupported { origin, .. } => origin,
        NormPattern::Error(error) => &error.origin,
    }
}
