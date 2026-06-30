use std::path::Path;

use lang_syntax::{
    norm::NormNavComponent, NormAliasBinder, NormAnnotation, NormClosure, NormDecl, NormExpr,
    NormForm, NormOrigin, NormPattern, NormProgram,
};

use crate::{
    core::install_core_bootstrap,
    discovery::{DiscoveredSourceUnit, SourceDiscoveryConfig, SourceDiscoveryReport},
    graph::{
        namespace_symbol, BuildError, NamespaceGraphSnapshot, ResolveExpectation, ResolverContext,
    },
    initializer_eval::{evaluate_initializer_best_effort, EvalMode, EvalOutcome},
    manifest::{BuildManifest, NamespaceMount},
    meta::{bind_meta_invocation_value_result, try_expand_early_meta_initializer},
    model::{
        Diagnostic, DiagnosticSeverity, MetaFunctionObject, NamespaceDelta, NamespaceNode,
        NamespaceNodeId, NamespaceNodeKind, PolicyFlag, PolicySet, Provenance, ResolverCode,
        SourceCallableObject, SourceCategory, SymbolKind, SymbolObject, SymbolPayload, TypeObject,
    },
    policy_expr::elaborate_declaration_policy_expr,
    policy_metadata, policy_set_meta, policy_set_meta_runtime, policy_set_runtime,
    source::SourceFragment,
    verify::evaluate_source_verifications as evaluate_verify_forms,
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

        // Physical source discovery is the explicit input layer below namespace
        // assembly. If discovery produced any hard diagnostic we must not
        // continue into partial namespace assembly.
        let report = SourceDiscoveryConfig::from_source_roots(&manifest.source_roots).discover();
        if report.has_hard_errors() {
            return Err(BuildError {
                diagnostics: report.diagnostics,
            });
        }
        world.diagnostics.extend(report.diagnostics.iter().cloned());
        world.consume_discovery(&report)?;

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

    /// Feed discovered physical source units into namespace assembly and
    /// declaration harvesting.
    ///
    /// Only directories containing discovered `.lang` source units contribute
    /// physical namespace nodes. Empty directories are ignored by v0.6 source
    /// discovery and do not create "empty namespace existence". If explicit
    /// empty-namespace nodes are ever required (e.g. package manifests or
    /// explicit namespace declarations) that must be a separate semantic
    /// decision, not a side effect of physical scanning.
    fn consume_discovery(&mut self, report: &SourceDiscoveryReport) -> Result<(), BuildError> {
        for unit in &report.units {
            let root = report
                .roots
                .iter()
                .find(|root| root.root_index == unit.source_root_index)
                .ok_or_else(|| {
                    BuildError::single(Diagnostic::hard_error(
                        "source discovery error: discovered unit references unknown source root",
                        Some(unit.provenance.clone()),
                    ))
                })?;

            let root_namespace =
                ensure_declared_namespace_path(&mut self.snapshot, &root.namespace_root)?;
            let directory = unit
                .canonical_path
                .parent()
                .unwrap_or(unit.canonical_path.as_path());
            let unit_namespace = ensure_physical_namespace_path(
                &mut self.snapshot,
                root_namespace,
                &unit.namespace_dir,
                directory,
            )?;
            self.consume_source_unit(unit, unit_namespace)?;
        }

        self.evaluate_source_verifications()?;

        Ok(())
    }

    fn consume_source_unit(
        &mut self,
        unit: &DiscoveredSourceUnit,
        namespace: NamespaceNodeId,
    ) -> Result<(), BuildError> {
        let parsed = lang_syntax::parse(&unit.content);
        let normalized = lang_syntax::normalize_program(&parsed.program);
        let provenance = unit.provenance.clone();
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

        self.harvest_program(namespace, &normalized, &unit.canonical_path)?;
        self.source_fragments.push(SourceFragment {
            path: unit.canonical_path.clone(),
            namespace,
            normalized,
            diagnostics,
            provenance,
        });

        Ok(())
    }

    fn evaluate_source_verifications(&mut self) -> Result<(), BuildError> {
        let mut diagnostics = Vec::new();
        for fragment in &self.source_fragments {
            let context = ResolverContext::with_mounts(
                fragment.namespace,
                vec![self.snapshot.root_node()],
                vec![self.core_node],
            );
            diagnostics.extend(evaluate_verify_forms(
                &self.snapshot,
                fragment.namespace,
                &fragment.normalized,
                &context,
            )?);
        }

        if diagnostics.is_empty() {
            Ok(())
        } else {
            self.diagnostics.extend(diagnostics.clone());
            Err(BuildError { diagnostics })
        }
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
                NormForm::Expr(_) | NormForm::TailValue(_) => {}
                NormForm::ReturnEvent(return_ev) => {
                    return Err(BuildError::single(Diagnostic::hard_error(
                        "source contribution error: return event is not allowed at the top level",
                        Some(Provenance::from_norm_origin(
                            "normalized return event",
                            &return_ev.origin,
                        )),
                    )));
                }
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
            NormPattern::OperatorBinder { spelling, .. } => spelling.clone(),
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

        if let Some(NormExpr::Closure(closure)) = slot.initializer.as_deref() {
            if closure.head.is_some() {
                let delta = source_callable_delta(
                    &self.snapshot,
                    namespace,
                    &binder_name,
                    slot.policy.as_deref(),
                    closure,
                    declaration_provenance.clone(),
                )?;
                self.snapshot = self
                    .snapshot
                    .install_delta(delta)
                    .map_err(BuildError::from)?;
                return Ok(());
            }
        }

        let explicit_policy = slot
            .policy
            .as_deref()
            .map(|policy_expr| {
                elaborate_declaration_policy_expr(Some(policy_expr), declaration_provenance.clone())
            })
            .transpose()
            .map_err(BuildError::single)?;

        if let Some(initializer) = slot.initializer.as_deref() {
            if let Some(mut expansion) = try_expand_early_meta_initializer(
                &self.snapshot,
                namespace,
                &binder_name,
                initializer,
                &context,
                declaration_provenance.clone(),
            )? {
                assert_expansion_satisfies_annotation(
                    slot.annotation.as_ref(),
                    &expansion.replacement_object,
                    declaration_provenance.clone(),
                )?;
                verify_explicit_policy_compatible(
                    explicit_policy.as_ref(),
                    &expansion.replacement_object.policy_metadata.policy_set,
                    declaration_provenance.clone(),
                )?;
                let final_binding_policy = final_binding_policy(
                    explicit_policy.as_ref(),
                    &expansion.replacement_object.policy_metadata.policy_set,
                );
                override_delta_binding_policy(
                    &mut expansion.namespace_delta,
                    &binder_name,
                    final_binding_policy.clone(),
                );
                expansion.replacement_object.policy_metadata.policy_set = final_binding_policy;
                self.snapshot = self
                    .snapshot
                    .install_delta(expansion.namespace_delta)
                    .map_err(BuildError::from)?;
                self.diagnostics.extend(expansion.diagnostics);
                return Ok(());
            }

            match evaluate_initializer_best_effort(
                &self.snapshot,
                namespace,
                initializer,
                &context,
                EvalMode::MetaPartial,
                declaration_provenance.clone(),
            ) {
                EvalOutcome::Value {
                    value,
                    result_policy,
                    provenance,
                } => {
                    assert_value_satisfies_annotation(
                        slot.annotation.as_ref(),
                        &value,
                        provenance,
                    )?;
                    verify_explicit_policy_compatible(
                        explicit_policy.as_ref(),
                        &result_policy,
                        declaration_provenance.clone(),
                    )?;
                    let final_binding_policy =
                        final_binding_policy(explicit_policy.as_ref(), &result_policy);
                    let mut expansion = bind_meta_invocation_value_result(
                        value,
                        &self.snapshot,
                        namespace,
                        &binder_name,
                        declaration_provenance.clone(),
                    )?;
                    override_delta_binding_policy(
                        &mut expansion.namespace_delta,
                        &binder_name,
                        final_binding_policy.clone(),
                    );
                    expansion.replacement_object.policy_metadata.policy_set = final_binding_policy;
                    self.snapshot = self
                        .snapshot
                        .install_delta(expansion.namespace_delta)
                        .map_err(BuildError::from)?;
                    self.diagnostics.extend(expansion.diagnostics);
                    return Ok(());
                }
                EvalOutcome::Residual {
                    reason, provenance, ..
                } => {
                    verify_residual_policy_compatible(
                        explicit_policy.as_ref(),
                        &reason,
                        provenance.clone(),
                    )?;
                    if is_type_annotation(slot.annotation.as_ref()) {
                        return Err(BuildError::single(Diagnostic::hard_error(
                            "UnsupportedDeferredTypeAssertion: `: type` assertion is deferred for a residual initializer, and deferred type assertions are not implemented in the restricted v0.8 initializer evaluator",
                            Some(provenance),
                        )
                        .with_code(ResolverCode::UnsupportedDeferredTypeAssertion)));
                    }
                }
                EvalOutcome::Diagnostic(diagnostic) => {
                    return Err(BuildError::single(diagnostic));
                }
            }
        }

        let mut delta = if is_type_annotation(slot.annotation.as_ref()) {
            declared_type_placeholder_delta(
                &self.snapshot,
                namespace,
                &binder_name,
                declaration_provenance,
            )
        } else {
            self.snapshot.capability().declare(
                namespace,
                binder_name.clone(),
                SymbolKind::Placeholder,
                SourceCategory::DeclaredSymbol,
                Provenance::file("declared source symbol", file),
            )
        };
        {
            let policy_set = explicit_policy.clone().unwrap_or_else(|| {
                if is_type_annotation(slot.annotation.as_ref()) {
                    policy_set_meta_runtime()
                } else {
                    policy_set_runtime()
                }
            });
            for symbol in delta.symbols.values_mut() {
                if symbol.name == binder_name {
                    symbol.policy_metadata.policy_set = policy_set.clone();
                }
            }
        }
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
        let mut delta = self.snapshot.capability().alias(
            namespace,
            name.clone(),
            target_symbol.id,
            Provenance::from_norm_origin("alias declaration", origin),
        );
        for symbol in delta.symbols.values_mut() {
            if symbol.name == name {
                symbol.policy_metadata.policy_set = policy_set_runtime();
            }
        }
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
    // type-object binding evaluation exists. Long-term, `let t: type = uint8` is an
    // ordinary binding of symbol/place `t` to the existing type object `uint8`,
    // not fresh type generation and not symbol aliasing. Namespace injection
    // through `t` must target place(t), not place(uint8), once writable-place
    // checking exists.
    //
    // This PR (v0.6.1) does not implement canonical first-order projection
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
        extraction_interface: None,
        provenance,
        generation_origin: None,
        layout_slot: None,
        abi_slot: None,
    });
    delta.insert_symbol(parent, symbol);
    delta
}

fn source_callable_delta(
    snapshot: &NamespaceGraphSnapshot,
    parent: NamespaceNodeId,
    name: &str,
    policy_expr: Option<&NormExpr>,
    closure: &NormClosure,
    provenance: Provenance,
) -> Result<NamespaceDelta, BuildError> {
    let symbol_policy = elaborate_declaration_policy_expr(policy_expr, provenance.clone())
        .map_err(BuildError::single)?;
    let body_entry_policy =
        body_entry_policy_from_closure(closure, provenance.clone()).map_err(BuildError::single)?;
    ensure_return_policy_supported(closure, provenance.clone()).map_err(BuildError::single)?;

    let mut delta = snapshot.empty_delta();
    let symbol_id = delta.allocate_symbol_id();
    let mut symbol = SymbolObject::placeholder(
        symbol_id,
        name,
        SymbolKind::MetaFunction,
        SourceCategory::DeclaredSymbol,
        Some(parent),
        provenance.clone(),
    );
    symbol.policy_metadata.policy_set = symbol_policy.clone();
    symbol.payload = SymbolPayload::MetaFunction(MetaFunctionObject {
        function_symbol_id: symbol_id,
        primitive: None,
        source_callable: Some(SourceCallableObject {
            closure: closure.clone(),
            provenance: provenance.clone(),
        }),
        function_policy: policy_metadata(symbol_policy.clone()),
        body_entry_policy: policy_metadata(body_entry_policy),
        return_object_policy: policy_metadata(symbol_policy),
    });
    delta.insert_symbol(parent, symbol);
    Ok(delta)
}

fn body_entry_policy_from_closure(
    closure: &NormClosure,
    provenance: Provenance,
) -> Result<PolicySet, Diagnostic> {
    let Some(head) = &closure.head else {
        return Err(Diagnostic::hard_error(
            "source callable declaration requires an explicit closure head",
            Some(provenance),
        ));
    };
    let Some(annotation) = &head.fn_item_trait else {
        return Err(Diagnostic::hard_error(
            "source callable declaration requires a body-entry annotation such as `: meta ->`",
            Some(provenance),
        ));
    };
    match &annotation.pattern {
        NormPattern::Name { name, .. } if name == "meta" => Ok(policy_set_meta()),
        NormPattern::Name { name, .. } if name == "runtime" => Ok(policy_set_runtime()),
        _ => Err(Diagnostic::hard_error(
            "source callable body-entry policy must currently be `meta` or `runtime`",
            Some(provenance),
        )),
    }
}

fn ensure_return_policy_supported(
    closure: &NormClosure,
    provenance: Provenance,
) -> Result<(), Diagnostic> {
    let Some(head) = &closure.head else {
        return Ok(());
    };
    let Some(returns) = &head.returns else {
        return Ok(());
    };
    if returns.policy.is_some() {
        return Err(Diagnostic::hard_error(
            "unsupported explicit return policy annotation in restricted v0.8 callable declaration",
            Some(provenance),
        ));
    }
    Ok(())
}

fn assert_expansion_satisfies_annotation(
    annotation: Option<&NormAnnotation>,
    replacement_object: &SymbolObject,
    provenance: Provenance,
) -> Result<(), BuildError> {
    if is_type_annotation(annotation) && replacement_object.kind != SymbolKind::Type {
        return Err(BuildError::single(
            Diagnostic::hard_error(
                "AnnotationAssertionFailed: `: type` assertion failed after RHS evaluation",
                Some(provenance),
            )
            .with_code(ResolverCode::AnnotationAssertionFailed),
        ));
    }
    Ok(())
}

fn assert_value_satisfies_annotation(
    annotation: Option<&NormAnnotation>,
    value: &crate::MetaInvocationValue,
    provenance: Provenance,
) -> Result<(), BuildError> {
    if !is_type_annotation(annotation) {
        return Ok(());
    }
    let is_type_level = matches!(
        value,
        crate::MetaInvocationValue::ForwardedValue(crate::ForwardedValue {
            target: crate::MetaValueTarget::TypeSymbol(_),
            ..
        }) | crate::MetaInvocationValue::GeneratedConstructionValue(_)
            | crate::MetaInvocationValue::GeneratedTypeDefinitionValue(_)
    );
    if is_type_level {
        Ok(())
    } else {
        Err(BuildError::single(
            Diagnostic::hard_error(
                "AnnotationAssertionFailed: `: type` assertion failed after RHS evaluation",
                Some(provenance),
            )
            .with_code(ResolverCode::AnnotationAssertionFailed),
        ))
    }
}

fn verify_explicit_policy_compatible(
    explicit_policy: Option<&PolicySet>,
    result_policy: &PolicySet,
    provenance: Provenance,
) -> Result<(), BuildError> {
    let Some(explicit_policy) = explicit_policy else {
        return Ok(());
    };
    if policy_subset(explicit_policy, result_policy) {
        Ok(())
    } else {
        Err(BuildError::single(Diagnostic::hard_error(
            "ExplicitPolicyVerificationFailed: explicit binding policy is not compatible with RHS result policy",
            Some(provenance),
        )
        .with_code(ResolverCode::ExplicitPolicyVerificationFailed)))
    }
}

fn verify_residual_policy_compatible(
    explicit_policy: Option<&PolicySet>,
    reason: &crate::ResidualReason,
    provenance: Provenance,
) -> Result<(), BuildError> {
    let Some(explicit_policy) = explicit_policy else {
        return Ok(());
    };
    if explicit_policy.contains(PolicyFlag::Meta) {
        Err(BuildError::single(Diagnostic::hard_error(
            format!(
                "ExplicitPolicyVerificationFailed: RHS residualized to runtime ({reason:?}) and cannot satisfy explicit meta-visible binding policy"
            ),
            Some(provenance),
        )
        .with_code(ResolverCode::ExplicitPolicyVerificationFailed)))
    } else {
        Ok(())
    }
}

fn policy_subset(requested: &PolicySet, available: &PolicySet) -> bool {
    requested.flags.iter().all(|flag| available.contains(*flag))
}

fn final_binding_policy(
    explicit_policy: Option<&PolicySet>,
    result_policy: &PolicySet,
) -> PolicySet {
    if let Some(explicit_policy) = explicit_policy {
        return explicit_policy.clone();
    }
    let mut inferred = result_policy.clone();
    inferred.flags.remove(&PolicyFlag::Export);
    inferred
}

fn override_delta_binding_policy(
    delta: &mut NamespaceDelta,
    binding_name: &str,
    policy: PolicySet,
) {
    for symbol in delta.symbols.values_mut() {
        if symbol.name == binding_name {
            symbol.policy_metadata.policy_set = policy.clone();
        }
    }
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
