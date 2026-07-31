use std::path::Path;

use lang_syntax::{
    norm::NormNavComponent, NormAliasBinder, NormAnnotation, NormClosure, NormDecl, NormExpr,
    NormForm, NormOrigin, NormPattern, NormPolicySpec, NormProgram,
};

use crate::{
    core::{core_declared_pair, install_core_bootstrap},
    discovery::{DiscoveredSourceUnit, SourceDiscoveryConfig, SourceDiscoveryReport},
    initializer_eval::EvalMode,
    manifest::{BuildManifest, NamespaceMount},
    meta::bind_meta_invocation_value_result_with_materialization_state,
    model::{
        Diagnostic, DiagnosticSeverity, MetaFunctionObject, NamespaceNode, NamespaceNodeId,
        NamespaceNodeKind, PolicyFlag, PolicySet, Provenance, ResolverCode, SemanticNameDelta,
        SourceCallableObject, SourceCategory, SymbolKind, SymbolObject, SymbolPayload, TypeObject,
    },
    pattern_head::TypeMaterializationState,
    policy_expr::{elaborate_declaration_policy_expr, legacy_policy_set_from_pair},
    policy_metadata,
    policy_pair::{
        derive_function_object_p1, elaborate_namespace_declaration_policy,
        function_object_declaration_policy, normalize_p2_policy, ExplicitP1Selection,
        NamespaceDeclarationPolicy, NamespaceDeclarationPosition,
    },
    policy_pair::{PatternComponentPolicy, PolicyPair, PolicyStage, ValuePresence},
    policy_set_meta_runtime, policy_set_runtime,
    return_target::{
        elaborate_return_targets_in_program, elaborate_return_targets_in_returnable_closure,
        ReturnFrameOwner,
    },
    semantic_name_index::{
        namespace_symbol, BuildError, ResolveExpectation, ResolverContext, SemanticNameIndex,
    },
    semantic_world::{SemanticDeclarationEntry, SemanticNamespaceDelta, SemanticWorld},
    source::SourceFragment,
    verify::evaluate_source_verifications as evaluate_verify_forms,
};

/// One resolved call target together with the COMPLETE host chain it was
/// reached through.
///
/// Explicit navigation `g::f::T` selects the terminal Symbol *through* every
/// intermediate host type member, and exposure composes over the whole chain:
/// `Expose(g::f::T, φ) = Expose(T, φ) ∧ Expose(f, φ) ∧ Expose(g_member, φ)`.
/// Keeping only the innermost host would silently drop the outer navigability
/// factors, so the navigator's full `host_chain` travels with the target and
/// the invocation gates on every layer.
#[derive(Clone, Debug)]
pub struct ResolvedCallTarget {
    pub host_chain: Vec<crate::PatternHostMember>,
    pub symbol: crate::SemanticSymbolIdentity,
}

#[derive(Clone, Debug)]
enum ConnectedInitializerOutcome {
    Ordinary(crate::InvocationOutcome),
    Existing(Vec<crate::PolicyResultEntry<crate::SemanticValueRef, crate::PatternValueId>>),
    Residual {
        reason: crate::ResidualReason,
        provenance: Provenance,
    },
    Diagnostic(Diagnostic),
}

/// Build/namespace world object for the v0.6 vertical slice.
///
/// This is the canonical holder for source fragments, the default core mount,
/// and one connected [`SemanticWorld`].  Namespace topology is owned by that
/// semantic world; there is no separately committed namespace snapshot.
#[derive(Clone, Debug)]
pub struct CompilationWorld {
    package_root_node: NamespaceNodeId,
    core_node: NamespaceNodeId,
    semantic_world: SemanticWorld,
    type_materialization_state: TypeMaterializationState,
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
        if manifest.namespace_root.is_empty() {
            return Err(BuildError::single(Diagnostic::hard_error(
                "build manifest error: an ordinary project requires a non-empty namespace install prefix; only toolchain global construction owns `::`",
                Some(Provenance::new("ordinary project construction authority")),
            )));
        }
        if manifest
            .source_roots
            .iter()
            .any(|root| root.namespace_root.is_empty())
        {
            return Err(BuildError::single(Diagnostic::hard_error(
                "build manifest error: an ordinary source root cannot install directly into `::`; only ToolchainGlobalSourceRoot carries global construction authority",
                Some(Provenance::new("ordinary project construction authority")),
            )));
        }

        let snapshot = SemanticNameIndex::new();
        let (mut snapshot, core_node, core_callables, core_types) =
            install_core_bootstrap(&snapshot)?;

        let mut semantic_world = SemanticWorld::new(manifest.package_name.clone());
        semantic_world.bind_toolchain_root(snapshot.root_node());
        if core_node != snapshot.root_node() {
            let core = snapshot
                .node(core_node)
                .expect("core bootstrap returned an installed namespace");
            semantic_world
                .register_namespace(
                    core_node,
                    core.parent.unwrap_or_else(|| snapshot.root_node()),
                    core.name.clone(),
                )
                .expect("root semantic owner is installed");
        }
        let package_root_node =
            ensure_declared_namespace_path(&mut snapshot, &manifest.namespace_root)?;
        let mut owner_check = Some(package_root_node);
        while let Some(node) = owner_check {
            if node == snapshot.root_node() {
                break;
            }
            if semantic_world.namespace_owner(node).is_some() {
                return Err(BuildError::single(Diagnostic::hard_error(
                    "build manifest error: ordinary package boundary overlaps a toolchain-owned namespace",
                    Some(Provenance::new(
                        "ordinary project construction authority",
                    )),
                )));
            }
            owner_check = snapshot.node(node).and_then(|node| node.parent);
        }
        semantic_world.bind_package_namespace(package_root_node);
        install_dependency_mounts(&mut snapshot, &manifest.dependency_mounts)?;
        semantic_world.replace_namespace_index(snapshot);

        let mut world = Self {
            package_root_node,
            core_node,
            semantic_world,
            type_materialization_state: TypeMaterializationState::default(),
            source_fragments: Vec::new(),
            diagnostics: Vec::new(),
        };
        // Core type carriers enter the semantic world
        // straight from the bootstrap's declared registration roster; the
        // graph is never rescanned through a flat legacy PolicySet
        // re-projection (`sync_semantic_type_values` is deleted).
        let type_rank = core_types
            .iter()
            .find(|registration| registration.name == "type")
            .map(|registration| registration.represented_type)
            .expect("core bootstrap declares the canonical `type` carrier first");
        for registration in core_types {
            world
                .semantic_world
                .register_type_symbol(
                    registration.namespace,
                    &registration.name,
                    registration.binding,
                    registration.represented_type,
                    type_rank,
                    Some(registration.associated_namespace),
                    registration.policy,
                    registration.provenance,
                )
                .expect("core namespace has a semantic owner");
        }
        // Core callables enter the semantic world straight
        // from the bootstrap's declared registration roster; there is no
        // graph-payload scan or legacy PolicySet re-projection step.
        for registration in core_callables {
            world.semantic_world.register_core_callable(
                registration.namespace,
                &registration.name,
                registration.backing,
                registration.primitive,
                Some(ExplicitP1Selection::from_complete_pair(
                    &registration.function_policy,
                )),
                registration.return_shape,
                registration.function_policy,
                registration.result_policy,
                registration.visibility,
                registration.provenance,
            )?;
        }

        let global_roots = manifest
            .global_implementation_roots
            .iter()
            .map(|root| crate::SourceRoot {
                path: root.path.clone(),
                namespace_root: root.install_prefix.clone(),
            })
            .collect::<Vec<_>>();
        let global_report = SourceDiscoveryConfig::from_source_roots(&global_roots).discover();
        if global_report.has_hard_errors() {
            return Err(BuildError {
                diagnostics: global_report.diagnostics,
            });
        }
        world
            .diagnostics
            .extend(global_report.diagnostics.iter().cloned());
        world.consume_global_implementation_discovery(&global_report)?;

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

    /// Read-only compatibility projection for diagnostics and historical
    /// boundary tests. Namespace allocation, installation, and invocation
    /// authority remain inside the connected SemanticWorld.
    pub fn namespace_projection(&self) -> &SemanticNameIndex {
        self.semantic_world.namespace_index()
    }

    pub fn package_root_node(&self) -> NamespaceNodeId {
        self.package_root_node
    }

    pub fn core_node(&self) -> NamespaceNodeId {
        self.core_node
    }

    pub fn semantic_world(&self) -> &SemanticWorld {
        &self.semantic_world
    }

    /// `Addr(Norm_type(type_value, place))` — passthrough to the semantic
    /// world's type-observation interning, for callers (tests, expectation
    /// material) that need the observation address of an already-resolved
    /// type value.  Interning is content-idempotent, so replaying an
    /// observation already read at an invocation boundary returns the same
    /// address.
    pub fn canonical_type_observation_address(
        &mut self,
        type_value: crate::TypeValueId,
        place: Option<crate::ObjectPlaceId>,
    ) -> Result<crate::CanonicalValueAddr, crate::Diagnostic> {
        self.semantic_world
            .canonical_type_observation_address(type_value, place)
    }

    /// Test-support passthrough for an ordinary Val2 injection
    /// (`let name::target = symbol;`): records `name -> symbol` in the given
    /// object place.  Resident-observation tests use it to change one type
    /// object's observed Val2 between two invocations without a second build.
    pub fn associate_existing_symbol_in_place(
        &mut self,
        place: crate::ObjectPlaceId,
        name: &str,
        symbol: crate::SemanticSymbolIdentity,
    ) -> Option<()> {
        self.semantic_world
            .associate_existing_symbol_in_place(place, name, symbol)
    }

    /// Install one already-evaluated semantic value without inventing a
    /// binding Symbol or rerooting its canonical PatternValue.
    pub fn install_semantic_value(
        &mut self,
        type_value: crate::TypeValueId,
        policy: PolicyPair,
        provenance: Provenance,
    ) -> Option<crate::SemanticValueId> {
        self.semantic_world
            .install_plain_value(type_value, policy, provenance)
    }

    /// Invoke the language-authorized atomic runtime migration through the
    /// same Pattern-associated ordinary-call trunk used by source callables.
    pub fn invoke_atomic_runtime_migration(
        &mut self,
        request: &crate::PolicyTransitionRequest,
    ) -> Result<crate::AtomicRuntimeMigrationResult, crate::OrdinaryInvocationFailure> {
        let resolver_context = self.root_context();
        crate::invoke_atomic_runtime_migration(
            &mut self.semantic_world,
            &mut self.type_materialization_state,
            request,
            &resolver_context,
        )
    }

    /// Invoke one already-authorized Pattern-associated operation without
    /// fabricating a source path or a migration-specific resolver.
    pub fn invoke_pattern_associated_operation(
        &mut self,
        pattern: crate::PatternValueId,
        operation_name: &str,
        receiver: crate::SemanticValueId,
        explicit_args: crate::ArgProductShape,
        context: crate::OrdinaryInvocationContext<'_>,
        provenance: Provenance,
    ) -> Result<crate::InvocationOutcome, crate::OrdinaryInvocationFailure> {
        let resolver_context = self.root_context();
        if operation_name == "()" {
            crate::invoke_pattern_associated_ordinary(
                &mut self.semantic_world,
                &mut self.type_materialization_state,
                pattern,
                operation_name,
                receiver,
                explicit_args,
                &resolver_context,
                context,
                provenance,
            )
        } else {
            let mut explicit_mutability =
                Vec::with_capacity(1 + context.explicit_argument_mutability.len());
            explicit_mutability.push(crate::ValueMutability::Const);
            explicit_mutability.extend_from_slice(context.explicit_argument_mutability);
            let named_context = crate::OrdinaryInvocationContext {
                explicit_argument_mutability: &explicit_mutability,
                ..context
            };
            crate::invoke_pattern_associated_value_ordinary(
                &mut self.semantic_world,
                &mut self.type_materialization_state,
                pattern,
                operation_name,
                receiver,
                explicit_args,
                &resolver_context,
                named_context,
                provenance,
            )
        }
    }

    pub fn source_fragments(&self) -> &[SourceFragment] {
        &self.source_fragments
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn type_materialization_state(&self) -> &TypeMaterializationState {
        &self.type_materialization_state
    }

    /// Every installation point that
    /// carries a `TypeObject` registers its semantic type binding through the
    /// atomic [`SemanticNamespaceDelta`] path with the declared canonical
    /// `PolicyPair`; the graph is never rescanned through a flat legacy
    /// PolicySet re-projection, and the type-associated namespace is created
    /// by the semantic world itself instead of being read back from the
    /// graph.
    fn register_installed_type_carrier(
        &mut self,
        namespace: NamespaceNodeId,
        name: &str,
        binding: crate::SymbolId,
        represented_type: crate::TypeValueId,
        associated_namespace: Option<NamespaceNodeId>,
        policy: PolicyPair,
        provenance: Provenance,
    ) -> Result<(), BuildError> {
        self.semantic_world
            .install_namespace_delta(SemanticNamespaceDelta {
                namespace,
                entries: vec![SemanticDeclarationEntry::TypeCarrier {
                    name: name.to_string(),
                    binding,
                    represented_type,
                    associated_namespace: associated_namespace
                        .map(|node| (node, format!("{name}<type-associated>"))),
                    policy,
                    provenance,
                }],
            })
    }

    /// Registers the semantic type binding for a meta-expansion replacement
    /// object when (and only when) it carries a `TypeObject` payload.
    fn register_expansion_type_carrier(
        &mut self,
        replacement_object: &SymbolObject,
        namespace: NamespaceNodeId,
        policy: PolicyPair,
    ) -> Result<(), BuildError> {
        if let SymbolPayload::Type(type_object) = &replacement_object.payload {
            self.register_installed_type_carrier(
                namespace,
                &replacement_object.name,
                replacement_object.id,
                type_object.represented_type,
                type_object.type_associated_namespace,
                policy,
                replacement_object.provenance.clone(),
            )?;
        }
        Ok(())
    }

    pub fn package_context(&self) -> ResolverContext {
        ResolverContext::with_mounts(
            self.package_root_node,
            vec![self.semantic_world.namespace_index().root_node()],
            vec![self.core_node],
        )
    }

    pub fn root_context(&self) -> ResolverContext {
        ResolverContext::with_mounts(
            self.semantic_world.namespace_index().root_node(),
            vec![self.semantic_world.namespace_index().root_node()],
            vec![self.core_node],
        )
    }

    pub fn resolve(&self, source_order_path: &str) -> Result<SymbolObject, Diagnostic> {
        self.resolve_with_expectation(source_order_path, ResolveExpectation::AnyUnique)
    }

    pub fn resolve_with_expectation(
        &self,
        source_order_path: &str,
        expectation: ResolveExpectation,
    ) -> Result<SymbolObject, Diagnostic> {
        let components = source_order_path
            .split("::")
            .filter(|component| !component.is_empty())
            .map(str::trim)
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        let identity = self.semantic_world.resolve_symbol_path(
            &components,
            self.package_root_node,
            &[self.semantic_world.namespace_index().root_node()],
            &[self.core_node],
        )?;
        let object = self
            .semantic_world
            .projected_symbol_object(identity)
            .cloned()
            .ok_or_else(|| {
                Diagnostic::hard_error(
                    format!(
                        "resolver error: semantic symbol `{source_order_path}` has no declaration projection"
                    ),
                    None,
                )
                .with_code(ResolverCode::Unresolved)
            })?;
        if projection_matches_expectation(&object, expectation) {
            Ok(object)
        } else {
            Err(Diagnostic::hard_error(
                format!(
                    "resolver error: unresolved symbol `{source_order_path}` for expectation {expectation:?}"
                ),
                None,
            )
            .with_code(ResolverCode::Unresolved))
        }
    }

    /// Resolve a source-order type path through semantic Symbol/Pattern
    /// facts, without reading a declaration payload from the namespace-name
    /// index.
    pub fn resolve_type_value(
        &self,
        source_order_path: &str,
    ) -> Result<crate::TypeValueId, Diagnostic> {
        let components = source_order_path
            .split("::")
            .filter(|component| !component.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        let identity = self.semantic_world.resolve_symbol_path(
            &components,
            self.package_root_node,
            &[self.semantic_world.namespace_index().root_node()],
            &[self.core_node],
        )?;
        let pattern = self
            .semantic_world
            .symbol(identity)
            .and_then(|symbol| symbol.pure_p_pattern())
            .ok_or_else(|| {
                Diagnostic::hard_error(
                    format!("resolver error: `{source_order_path}` is not a semantic type carrier"),
                    None,
                )
            })?;
        self.semantic_world
            .type_for_pattern(pattern)
            .ok_or_else(|| {
                Diagnostic::hard_error(
                    format!("resolver error: `{source_order_path}` has no represented TypeValue"),
                    None,
                )
            })
    }

    /// Resolve a normalized source call through the semantic Symbol/value/type
    /// associated-`()` spine and invoke the unique ordinary candidate.
    ///
    /// This is the source-facing integration entry.  It does not use
    /// `call_target`'s legacy direct-callable shortcut.
    pub fn invoke_ordinary_call(
        &mut self,
        namespace: NamespaceNodeId,
        call_site: &crate::NormalizedCallSite,
        context: crate::OrdinaryInvocationContext<'_>,
        provenance: Provenance,
    ) -> Result<crate::InvocationOutcome, crate::OrdinaryInvocationFailure> {
        let candidates = self.resolve_semantic_call_target_chain(namespace, &call_site.target);
        if candidates.is_empty() {
            return Err(crate::OrdinaryInvocationFailure::NoTargetValues {
                trace: crate::OrdinaryPipelineTrace::default(),
            });
        }
        let resolver_context = ResolverContext::with_mounts(
            namespace,
            vec![self.semantic_world.namespace_index().root_node()],
            vec![self.core_node],
        );
        let caller_package = self
            .semantic_world
            .namespace_owner(namespace)
            .map(|owner| self.semantic_world.owners().package_of(owner));
        let mut last_failure = None;
        for candidate in candidates {
            let symbol = candidate.symbol;
            let mut attempt_context = context;
            let target_package = self.semantic_world.symbol(symbol).map(|symbol| {
                self.semantic_world
                    .owners()
                    .package_of(symbol.declaration_owner)
            });
            if caller_package.is_some()
                && target_package.is_some()
                && caller_package != target_package
            {
                attempt_context.visibility = crate::VisibilityView::External;
            }
            match crate::invoke_host_member_symbol_ordinary(
                &mut self.semantic_world,
                &mut self.type_materialization_state,
                &candidate.host_chain,
                symbol,
                call_site,
                &resolver_context,
                attempt_context,
                provenance.clone(),
            ) {
                Ok(outcome) => return Ok(outcome),
                // No-shadow candidate search: a nearer same-name Symbol
                // whose member views expose no admissible callable does not
                // shadow an outer callable Symbol; the search falls through
                // to the next scope link.  Any other failure is a real
                // selection/execution failure of this scope's candidate set.
                Err(
                    failure @ (crate::OrdinaryInvocationFailure::NoTargetValues { .. }
                    | crate::OrdinaryInvocationFailure::NoFullyAdmissibleCandidate {
                        ..
                    }),
                ) => {
                    last_failure = Some(failure);
                }
                Err(failure) => return Err(failure),
            }
        }
        Err(last_failure.expect("non-empty candidate chain records a failure"))
    }

    /// Scope-ordered same-name call-target Symbols (`near → outer → core`).
    /// Navigation targets resolve to exactly one Symbol.
    fn resolve_semantic_call_target_chain(
        &self,
        namespace: NamespaceNodeId,
        target: &NormExpr,
    ) -> Vec<ResolvedCallTarget> {
        let name = match target {
            NormExpr::Name { text, .. } => Some(text.as_str()),
            NormExpr::OperatorTarget { spelling, .. } => Some(spelling.as_str()),
            _ => None,
        };
        let Some(name) = name else {
            return self
                .resolve_semantic_call_target(namespace, target)
                .into_iter()
                .collect();
        };
        let mut chain: Vec<ResolvedCallTarget> = Vec::new();
        for scope in self
            .semantic_world
            .bare_name_scope_chain(namespace, &[self.core_node])
        {
            if let Some(symbol) = self.semantic_world.symbol_in_namespace(scope, name) {
                if !chain
                    .iter()
                    .any(|candidate| candidate.symbol == symbol.identity)
                {
                    // A bare name is reached without navigating a host layer,
                    // so only the member factor of the exposure conjunction
                    // applies.
                    chain.push(ResolvedCallTarget {
                        host_chain: Vec::new(),
                        symbol: symbol.identity,
                    });
                }
            }
        }
        chain
    }

    fn resolve_semantic_call_target(
        &self,
        namespace: NamespaceNodeId,
        target: &NormExpr,
    ) -> Option<ResolvedCallTarget> {
        if let NormExpr::Name { text, .. } = target {
            return self
                .semantic_world
                .bare_name_scope_chain(namespace, &[self.core_node])
                .into_iter()
                .find_map(|scope| {
                    self.semantic_world
                        .symbol_in_namespace(scope, text)
                        .map(|symbol| ResolvedCallTarget {
                            host_chain: Vec::new(),
                            symbol: symbol.identity,
                        })
                });
        }
        if let NormExpr::OperatorTarget { spelling, .. } = target {
            return self
                .semantic_world
                .bare_name_scope_chain(namespace, &[self.core_node])
                .into_iter()
                .find_map(|scope| {
                    self.semantic_world
                        .symbol_in_namespace(scope, spelling)
                        .map(|symbol| ResolvedCallTarget {
                            host_chain: Vec::new(),
                            symbol: symbol.identity,
                        })
                });
        }
        let NormExpr::Nav {
            components,
            explicit_terminated,
            ..
        } = target
        else {
            return None;
        };
        // Explicit navigation resolves through the one shared recursive Symbol
        // navigator, which returns the COMPLETE host chain it stepped through.
        // The call target keeps every traversed host so the invocation can
        // compose the full per-layer exposure conjunction
        // `Expose(T, φ) ∧ Expose(f, φ) ∧ …`; collapsing the chain to the
        // innermost host alone would silently drop the outer navigability
        // constraints, e.g. `g::f::T` must be unreachable when `T` is hidden
        // even if `f` and `g` are visible.  The navigator also reads each
        // host's own Val2 place, so lookup never leaks a same-Pattern
        // carrier's members.
        let path = components
            .iter()
            .map(|component| match component {
                NormNavComponent::Name { name, .. } => Some(name.clone()),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?;
        let root = self.semantic_world.namespace_index().root_node();
        let lookup_start = if *explicit_terminated {
            root
        } else {
            namespace
        };
        self.semantic_world
            .navigate_semantic_path(
                &path,
                lookup_start,
                &[root, self.core_node],
                &[self.core_node],
            )
            .ok()
            .map(|navigation| ResolvedCallTarget {
                host_chain: navigation.host_chain,
                symbol: navigation.terminal_symbol,
            })
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

            let root_namespace = self.semantic_world.ensure_namespace_path(
                self.semantic_world.namespace_index().root_node(),
                &root.namespace_root,
                NamespaceNodeKind::Declared,
                SourceCategory::DeclaredSymbol,
                "declared namespace mount",
            )?;
            let directory = unit
                .canonical_path
                .parent()
                .unwrap_or(unit.canonical_path.as_path());
            let unit_namespace = self.semantic_world.ensure_namespace_path(
                root_namespace,
                &unit.namespace_dir,
                NamespaceNodeKind::Physical,
                SourceCategory::PhysicalDirectory,
                &format!("physical directory `{}`", directory.display()),
            )?;
            self.consume_source_unit(unit, unit_namespace)?;
        }

        self.evaluate_source_verifications()?;

        Ok(())
    }

    fn consume_global_implementation_discovery(
        &mut self,
        report: &SourceDiscoveryReport,
    ) -> Result<(), BuildError> {
        let global_root = self.semantic_world.namespace_index().root_node();
        for unit in &report.units {
            let root = report
                .roots
                .iter()
                .find(|root| root.root_index == unit.source_root_index)
                .ok_or_else(|| {
                    BuildError::single(Diagnostic::hard_error(
                        "source discovery error: global unit references unknown source root",
                        Some(unit.provenance.clone()),
                    ))
                })?;
            let install_root = self.semantic_world.ensure_namespace_path(
                global_root,
                &root.namespace_root,
                NamespaceNodeKind::Declared,
                SourceCategory::DeclaredSymbol,
                "declared namespace mount",
            )?;
            let directory = unit
                .canonical_path
                .parent()
                .unwrap_or(unit.canonical_path.as_path());
            let unit_namespace = self.semantic_world.ensure_namespace_path(
                if root.namespace_root.is_empty() {
                    global_root
                } else {
                    install_root
                },
                &unit.namespace_dir,
                NamespaceNodeKind::Physical,
                SourceCategory::PhysicalDirectory,
                &format!("physical directory `{}`", directory.display()),
            )?;
            if !self
                .semantic_world
                .namespace_is_toolchain_owned(unit_namespace)
            {
                return Err(BuildError::single(Diagnostic::hard_error(
                    "global implementation source cannot contribute through a package-owned namespace boundary",
                    Some(unit.provenance.clone()),
                )));
            }
            self.consume_source_unit(unit, unit_namespace)?;
        }
        Ok(())
    }

    fn consume_source_unit(
        &mut self,
        unit: &DiscoveredSourceUnit,
        namespace: NamespaceNodeId,
    ) -> Result<(), BuildError> {
        let parsed = lang_syntax::parse(&unit.content);
        let provenance = unit.provenance.clone();
        let mut diagnostics = parsed
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
        let normalized = lang_syntax::normalize_and_validate_patterns(&parsed.program);
        if let Err(invalid) = &normalized {
            diagnostics.extend(invalid.pattern_errors.iter().map(|error| {
                let message = match error {
                    lang_syntax::PatternValidationError::MultiplePacks(error) => format!(
                        "Pattern contains {} pack nodes at one normalized structural level",
                        error.pack_count
                    ),
                    lang_syntax::PatternValidationError::NonCanonicalPackOperand { .. } => {
                        "Pack operand is a bare Product without a stable top Pattern; write a whole-remainder binder/discard or an explicitly headed structured Pattern"
                            .to_string()
                    }
                    lang_syntax::PatternValidationError::DuplicateHole { name, .. } => format!(
                        "DeduceList hole `{name}` duplicates a declaration in the same PatternRoot; a new PatternRoot may shadow"
                    ),
                };
                Diagnostic::hard_error(
                    message,
                    Some(Provenance::from_norm_origin(
                        "global normalized Pattern validation",
                        error.origin(),
                    )),
                )
            }));
        }
        self.diagnostics.extend(diagnostics.clone());

        let normalized = match normalized {
            Ok(validated) => {
                self.harvest_program(namespace, validated.as_program(), &unit.canonical_path)?;
                validated.into_program()
            }
            Err(invalid) => invalid.program,
        };

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
                vec![self.semantic_world.namespace_index().root_node()],
                vec![self.core_node],
            );
            diagnostics.extend(evaluate_verify_forms(
                self.semantic_world.namespace_index(),
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
        let return_target_report = elaborate_return_targets_in_program(normalized);
        if !return_target_report.diagnostics.is_empty() {
            return Err(BuildError {
                diagnostics: return_target_report.diagnostics,
            });
        }

        for form in &normalized.forms {
            // One source declaration is one transaction over the complete
            // semantic world, including namespace topology and the temporary
            // declaration-name index.  A later conflict or projection error
            // discards the staged clone, so no Symbol, member, namespace,
            // owner, or name-record prefix survives.
            let mut staged = self.clone();
            match form {
                NormForm::Let(decl) => staged.harvest_let(namespace, decl, file)?,
                NormForm::Alias(decl) => staged.harvest_alias(namespace, decl, file)?,
                NormForm::Expr(_) | NormForm::TailValue(_) => {}
                NormForm::ReturnEvent(return_ev) => {
                    return Err(BuildError::single(Diagnostic::hard_error(
                        "source contribution error: unbound return event reached declaration harvesting after return target binding",
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
            *self = staged;
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

        if matches!(
            &slot.value_pattern,
            NormPattern::Product { elements, .. } if elements.is_empty()
        ) {
            let Some(NormExpr::Closure(closure)) = slot.initializer.as_deref() else {
                return Err(BuildError::single(Diagnostic::hard_error(
                    "source contribution error: associated `let ()` requires a callable initializer",
                    Some(Provenance::from_norm_origin(
                        "associated call entry",
                        origin,
                    )),
                )));
            };
            let Some(pattern) = self
                .semantic_world
                .pattern_for_associated_namespace(namespace)
            else {
                return Err(BuildError::single(Diagnostic::hard_error(
                    "source contribution error: `let ()` is valid only in a namespace owned by an existing PatternValue",
                    Some(Provenance::from_norm_origin(
                        "associated call entry",
                        origin,
                    )),
                )));
            };
            let declaration_provenance =
                Provenance::from_norm_origin("associated call entry `()`", origin);
            let callable = source_callable_delta(
                self.semantic_world.namespace_index(),
                namespace,
                "()",
                slot.policy.as_ref(),
                closure,
                declaration_provenance.clone(),
            )?;
            // The semantic declaration is authoritative;
            // its compatibility rendering is installed afterward within the
            // same staged CompilationWorld transaction.
            self.semantic_world
                .install_namespace_delta(SemanticNamespaceDelta {
                    namespace,
                    entries: vec![SemanticDeclarationEntry::AssociatedCallEntry {
                        pattern,
                        backing_declaration: callable.symbol_id,
                        closure: closure.clone(),
                        outer_p1_explicit: callable.outer_p1_explicit.clone(),
                        callable_value_policy: callable.function_policy,
                        complete_result_policy: callable.result_policy,
                        namespace_visibility: callable.namespace_visibility,
                        candidate_role: crate::OrdinaryCandidateRole::Ordinary,
                        return_shape: callable.return_shape,
                        provenance: declaration_provenance,
                    }],
                })?;
            self.semantic_world
                .install_namespace_name_delta(callable.delta)?;
            return Ok(());
        }

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
        if let Some(NormExpr::Closure(closure)) = slot.initializer.as_deref() {
            if closure.head.is_some() {
                let callable = source_callable_delta(
                    self.semantic_world.namespace_index(),
                    namespace,
                    &binder_name,
                    slot.policy.as_ref(),
                    closure,
                    declaration_provenance.clone(),
                )?;

                // Explicit ContributeSiblingVal target.  A
                // declaration contributes as a cluster sibling val ONLY when
                // its binder name matches an existing cluster Symbol (a
                // Symbol whose `pure_p` is set) in the same namespace.  This
                // is the explicit boundary: `const let uint8 = (self, ...) =>
                // {...}` contributes to the `uint8` cluster, but `let identity
                // = ...` does NOT match any cluster Symbol and is registered as
                // an ordinary source callable (Val2[name]).  The previous
                // namespace-membership heuristic that contributed every
                // callable in an associated namespace as a sibling is deleted.
                let cluster_symbol = self
                    .semantic_world
                    .symbol_in_namespace(namespace, &binder_name)
                    .filter(|cell| cell.pure_p.is_some())
                    .map(|cell| cell.identity)
                    .or_else(|| {
                        // A declaration mounted inside a type-associated
                        // namespace (e.g. a global implementation root at
                        // `core::uint8`) contributes to the owning cluster
                        // Symbol when the binder name matches the cluster
                        // Symbol's own name.
                        self.semantic_world
                            .pattern_for_associated_namespace(namespace)
                            .and_then(|pattern| self.semantic_world.owner_cluster(pattern))
                            .and_then(|owner| owner.installed())
                            .filter(|identity| {
                                self.semantic_world.symbol(*identity).is_some_and(|cell| {
                                    cell.name == binder_name && cell.pure_p.is_some()
                                })
                            })
                    });

                // The semantic declaration is authoritative;
                // its compatibility rendering is installed afterward within
                // the same staged CompilationWorld transaction.
                let entry = if let Some(cluster_symbol) = cluster_symbol {
                    SemanticDeclarationEntry::ClusterContribution {
                        cluster_symbol,
                        backing_declaration: callable.symbol_id,
                        closure: closure.clone(),
                        outer_p1_explicit: callable.outer_p1_explicit.clone(),
                        function_policy: callable.function_policy,
                        complete_result_policy: callable.result_policy,
                        namespace_visibility: callable.namespace_visibility,
                        return_shape: callable.return_shape,
                        provenance: declaration_provenance,
                    }
                } else {
                    SemanticDeclarationEntry::SourceCallable {
                        name: binder_name.clone(),
                        backing_declaration: callable.symbol_id,
                        closure: closure.clone(),
                        outer_p1_explicit: callable.outer_p1_explicit.clone(),
                        function_policy: callable.function_policy,
                        complete_result_policy: callable.result_policy,
                        namespace_visibility: callable.namespace_visibility,
                        return_shape: callable.return_shape,
                        provenance: declaration_provenance,
                    }
                };
                self.semantic_world
                    .install_namespace_delta(SemanticNamespaceDelta {
                        namespace,
                        entries: vec![entry],
                    })?;
                self.semantic_world
                    .install_namespace_name_delta(callable.delta)?;
                return Ok(());
            }
        }

        let namespace_declaration = elaborate_namespace_declaration_policy(
            slot.policy.as_ref(),
            NamespaceDeclarationPosition::DirectTopLevel,
            declaration_provenance.clone(),
        )
        .map_err(BuildError::single)?;
        let explicit_policy = slot.policy.as_ref().map(|_| {
            crate::policy_expr::legacy_policy_set_from_namespace_declaration(&namespace_declaration)
        });
        let mut residual_binding_policy = None;

        if let Some(initializer) = slot.initializer.as_deref() {
            match self.evaluate_initializer_best_effort_connected(
                namespace,
                initializer,
                EvalMode::MetaPartial,
                declaration_provenance.clone(),
            ) {
                ConnectedInitializerOutcome::Ordinary(result) => {
                    return self.bind_connected_ordinary_result(
                        namespace,
                        &binder_name,
                        slot,
                        &namespace_declaration,
                        result,
                        declaration_provenance,
                    );
                }
                ConnectedInitializerOutcome::Existing(result) => {
                    return self.bind_connected_existing_result(
                        namespace,
                        &binder_name,
                        slot,
                        &namespace_declaration,
                        result,
                        declaration_provenance,
                    );
                }
                ConnectedInitializerOutcome::Residual { reason, provenance } => {
                    verify_residual_policy_compatible(
                        explicit_policy.as_ref(),
                        &reason,
                        provenance.clone(),
                    )?;
                    if let Some(explicit_policy) = explicit_policy.as_ref() {
                        residual_binding_policy =
                            policy_projection(explicit_policy, &policy_set_runtime());
                    }
                    if is_type_annotation(slot.annotation.as_ref()) {
                        return Err(BuildError::single(Diagnostic::hard_error(
                            "UnsupportedDeferredTypeAssertion: `: type` assertion is deferred for a residual initializer, and deferred type assertions are not implemented in the restricted v0.8 initializer evaluator",
                            Some(provenance),
                        )
                        .with_code(ResolverCode::UnsupportedDeferredTypeAssertion)));
                    }
                }
                ConnectedInitializerOutcome::Diagnostic(diagnostic) => {
                    return Err(BuildError::single(diagnostic));
                }
            }
        }

        let mut declared_type_carrier = None;
        let mut delta = if is_type_annotation(slot.annotation.as_ref()) {
            let carrier = declared_type_placeholder_delta(
                self.semantic_world.namespace_index(),
                namespace,
                &binder_name,
                declaration_provenance.clone(),
            );
            declared_type_carrier = Some((
                carrier.symbol_id,
                carrier.represented_type,
                carrier.associated_namespace,
            ));
            carrier.delta
        } else {
            self.semantic_world.namespace_index().capability().declare(
                namespace,
                binder_name.clone(),
                SymbolKind::Placeholder,
                SourceCategory::DeclaredSymbol,
                Provenance::file("declared source symbol", file),
            )
        };
        {
            let policy_set = residual_binding_policy
                .clone()
                .or_else(|| explicit_policy.clone())
                .unwrap_or_else(|| {
                    if is_type_annotation(slot.annotation.as_ref()) {
                        policy_set_meta_runtime()
                    } else {
                        policy_set_runtime()
                    }
                });
            for symbol in delta.symbols.values_mut() {
                if symbol.name == binder_name {
                    symbol.policy_metadata.policy_set = policy_set.clone();
                    symbol.visibility_metadata.namespace_visibility =
                        namespace_declaration.visibility;
                    symbol.visibility_metadata.export_root = namespace_declaration.export_root;
                }
            }
        }
        // Install the authoritative semantic carrier before
        // its compatibility rendering, in one staged world transaction.
        let semantic_entry = if let Some((symbol_id, represented_type, associated_namespace)) =
            declared_type_carrier
        {
            SemanticDeclarationEntry::TypeCarrier {
                name: binder_name.clone(),
                binding: symbol_id,
                represented_type,
                associated_namespace: Some((
                    associated_namespace,
                    format!("{binder_name}<type-associated>"),
                )),
                policy: declared_type_binding_pair(&namespace_declaration),
                provenance: declaration_provenance,
            }
        } else {
            let backing_declaration = delta
                .symbols
                .values()
                .find(|symbol| symbol.name == binder_name)
                .map(|symbol| symbol.id)
                .expect("residual binding delta contains its declaration projection");
            SemanticDeclarationEntry::ProjectionOnly {
                name: binder_name.clone(),
                backing_declaration,
                provenance: declaration_provenance,
            }
        };
        self.semantic_world
            .install_namespace_delta(SemanticNamespaceDelta {
                namespace,
                entries: vec![semantic_entry],
            })?;
        self.semantic_world.install_namespace_name_delta(delta)?;
        Ok(())
    }

    fn bind_connected_ordinary_result(
        &mut self,
        namespace: NamespaceNodeId,
        binder_name: &str,
        slot: &lang_syntax::NormBindingSlot,
        namespace_declaration: &NamespaceDeclarationPolicy,
        result: crate::InvocationOutcome,
        provenance: Provenance,
    ) -> Result<(), BuildError> {
        let result = match result {
            crate::InvocationOutcome::SingleMember(result) => result,
            crate::InvocationOutcome::ClusterSymbol(meta) => {
                return self.bind_connected_meta_construction_result(
                    namespace,
                    binder_name,
                    slot,
                    namespace_declaration,
                    meta,
                    provenance,
                );
            }
            crate::InvocationOutcome::Unit(_) => {
                // The invocation layer already reports Unit execution as
                // future work; no binding path exists yet.
                return Err(BuildError::single(Diagnostic::hard_error(
                    "binding a Unit invocation result is future work",
                    Some(provenance),
                )));
            }
        };
        // The ordinary binding path consumes the
        // exposure layer, never the raw complete result:
        //
        //   CompleteResultDomain(P2) -> expose under callable P1
        //                            -> outer binding P1
        //
        // The callable's canonical P1 is a real window here: material
        // outside it is invisible to the binder, and a fully invisible
        // result is a hard error before any outer projection runs.
        let exposed = result.exposed();
        if exposed.material.is_empty() {
            return Err(BuildError::single(Diagnostic::hard_error(
                "invocation result is not visible outside the callable: the complete \
                 result P2 domain has no overlap with the callable's canonical P1 \
                 exposure window",
                Some(provenance),
            )));
        }
        assert_semantic_result_satisfies_annotation(
            &self.semantic_world,
            slot.annotation.as_ref(),
            &exposed.material,
            provenance.clone(),
        )?;

        let explicit_p1 = slot
            .policy
            .as_ref()
            .map(|_| &namespace_declaration.projection);
        let mut exposed_material = exposed.material.clone();
        if let Some(projection) = explicit_p1 {
            if projection_requests_runtime_value(projection)
                && exposed_material.iter().all(|entry| entry.value.is_none())
            {
                // A pure-P (forwarded type) result carries no Val1.  A runtime
                // value demand first materializes the static source value of
                // the result type, then enters the same ordinary
                // atomic-migration trunk as any value-bearing result.
                for entry in &mut exposed_material {
                    let Some(type_value) = self.semantic_world.type_for_pattern(entry.pattern)
                    else {
                        continue;
                    };
                    let source_value_policy = crate::ValueComponentPolicy {
                        stages: entry.value_policy.stages.static_stages(),
                        mutability: entry.value_policy.mutability.clone(),
                        presence: ValuePresence::Present,
                    };
                    let source_policy = PolicyPair {
                        value: source_value_policy.clone(),
                        pattern: entry.pattern_policy.clone(),
                    };
                    let Some(source) = self.semantic_world.install_plain_value(
                        type_value,
                        source_policy,
                        provenance.clone(),
                    ) else {
                        continue;
                    };
                    entry.value = Some(crate::SemanticValueRef {
                        id: source,
                        type_value,
                    });
                    entry.value_policy = source_value_policy;
                }
            }
        }
        let elaborated =
            crate::elaborate_value_binding_p1(&exposed_material, explicit_p1, provenance.clone())
                .map_err(|failure| {
            BuildError::single(
                Diagnostic::hard_error(
                    format!(
                        "ExplicitPolicyProjectionFailed: ordinary result cannot satisfy binding P1 ({failure:?})"
                    ),
                    Some(provenance.clone()),
                )
                .with_code(ResolverCode::ExplicitPolicyVerificationFailed),
            )
        })?;

        match elaborated {
            crate::P1Elaboration::Projected { selected, .. } => match result.returned {
                crate::OrdinaryReturnedValue::Meta(crate::MetaInvocationValue::ForwardedValue(
                    _,
                ))
                | crate::OrdinaryReturnedValue::ForwardedSemanticValue(_) => self
                    .install_connected_semantic_binding(
                        namespace,
                        binder_name,
                        namespace_declaration,
                        &selected,
                        provenance,
                    )
                    .map(|_| ()),
                crate::OrdinaryReturnedValue::Meta(value) => {
                    let result_policy = legacy_policy_set_from_result_entries(&selected);
                    let mut expansion =
                        bind_meta_invocation_value_result_with_materialization_state(
                            value,
                            self.semantic_world.namespace_index(),
                            namespace,
                            binder_name,
                            provenance.clone(),
                            &mut self.type_materialization_state,
                        )?;
                    override_delta_binding_policy(
                        &mut expansion.namespace_delta,
                        binder_name,
                        result_policy.clone(),
                    );
                    override_delta_binding_visibility(
                        &mut expansion.namespace_delta,
                        binder_name,
                        namespace_declaration,
                    );
                    expansion.replacement_object.policy_metadata.policy_set = result_policy;
                    expansion
                        .replacement_object
                        .visibility_metadata
                        .namespace_visibility = namespace_declaration.visibility;
                    expansion.replacement_object.visibility_metadata.export_root =
                        namespace_declaration.export_root;
                    self.semantic_world
                        .bind_ordinary_new(namespace, binder_name, &selected, provenance.clone())
                        .map_err(|conflict| {
                            bind_conflict_error(conflict, binder_name, &provenance)
                        })?;
                    // Semantic type and projection Symbols
                    // are installed before their compatibility rendering.
                    if let Some(entry) = selected.first() {
                        self.register_expansion_type_carrier(
                            &expansion.replacement_object,
                            namespace,
                            declared_pair_from_result_entry(entry, namespace_declaration),
                        )?;
                    }
                    self.semantic_world
                        .register_generated_projection_symbols(&expansion.namespace_delta)?;
                    self.semantic_world
                        .install_namespace_name_delta(expansion.namespace_delta)?;
                    self.diagnostics.extend(expansion.diagnostics);
                    Ok(())
                }
            },
            crate::P1Elaboration::AtomicRuntimeMigration { demands, .. } => {
                let demanded_views = self.invoke_binding_migration_demands(demands, &provenance)?;
                self.install_connected_semantic_binding(
                    namespace,
                    binder_name,
                    namespace_declaration,
                    &demanded_views,
                    provenance,
                )
                .map(|_| ())
            }
        }
    }

    /// S6 — connect `InvocationOutcome::ClusterSymbol` to the ordinary let
    /// binding path.  The finalized cluster construction's member views are
    /// the canonical result facts; they flow through the same annotation check,
    /// P1 elaboration, and installation as any other connected result.
    /// Installation creates a fresh destination Symbol; patterns generated
    /// by this construction flip from `Open(cluster)` to `Installed`, while
    /// forwarded patterns keep their original owner (no reroot, no alias of
    /// the callee or any result source Symbol).
    fn bind_connected_meta_construction_result(
        &mut self,
        namespace: NamespaceNodeId,
        binder_name: &str,
        slot: &lang_syntax::NormBindingSlot,
        namespace_declaration: &NamespaceDeclarationPolicy,
        meta: crate::ClusterSymbolResult,
        provenance: Provenance,
    ) -> Result<(), BuildError> {
        let construction = meta.construction;
        let generated_types = meta.generated_types;
        let result = construction
            .member_views
            .iter()
            .map(|entry| crate::PolicyResultEntry {
                value: entry.value.map(|id| {
                    let value = self
                        .semantic_world
                        .value(id)
                        .expect("cluster construction member view references an installed value");
                    crate::SemanticValueRef {
                        id,
                        type_value: value.type_value,
                    }
                }),
                value_policy: entry.value_policy.clone(),
                pattern: entry.pattern,
                pattern_policy: entry.pattern_policy.clone(),
            })
            .collect::<Vec<_>>();
        assert_semantic_result_satisfies_annotation(
            &self.semantic_world,
            slot.annotation.as_ref(),
            &result,
            provenance.clone(),
        )?;

        let explicit_p1 = slot
            .policy
            .as_ref()
            .map(|_| &namespace_declaration.projection);
        let elaborated =
            crate::elaborate_value_binding_p1(&result, explicit_p1, provenance.clone()).map_err(
                |failure| {
                    BuildError::single(
                        Diagnostic::hard_error(
                            format!(
                                "ExplicitPolicyProjectionFailed: meta construction result cannot satisfy binding P1 ({failure:?})"
                            ),
                            Some(provenance.clone()),
                        )
                        .with_code(ResolverCode::ExplicitPolicyVerificationFailed),
                    )
                },
            )?;
        let selected = match elaborated {
            crate::P1Elaboration::Projected { selected, .. } => selected,
            crate::P1Elaboration::AtomicRuntimeMigration { demands, .. } => {
                self.invoke_binding_migration_demands(demands, &provenance)?
            }
        };
        // A construction whose sole member is backed by a generated type
        // definition expands the full namespace projection (field-function
        // layer, ref/share projection namespaces, extraction interface).
        // Everything else installs the plain semantic binding carrier.
        let destination =
            if generated_types.len() == 1 && selected.iter().all(|entry| entry.value.is_none()) {
                let generated = generated_types
                    .into_iter()
                    .next()
                    .expect("generated_types holds exactly one entry");
                // Diagnostic-only binder record: an ambient struct collision
                // at this level later points at this source-visible binding.
                // The binder never feeds type identity.
                let canonical_type = generated.canonical_type;
                let destination = self.install_connected_generated_type_binding(
                    namespace,
                    binder_name,
                    namespace_declaration,
                    &selected,
                    generated,
                    provenance,
                )?;
                if let Some(canonical_type) = canonical_type {
                    self.semantic_world.record_ambient_type_binder(
                        canonical_type,
                        crate::AmbientTypeBinder::WholeSymbol(binder_name.to_string()),
                    );
                }
                destination
            } else {
                self.install_connected_semantic_binding(
                    namespace,
                    binder_name,
                    namespace_declaration,
                    &selected,
                    provenance,
                )?
            };
        // Patterns generated by this construction are still registered to
        // the open cluster id; flip them to the fresh destination Symbol.
        // A forwarded-only construction has no `Open` pattern and yields
        // `None` here, which is the correct outcome: the forwarded pattern
        // keeps its original owner.
        let _ = self
            .semantic_world
            .upgrade_cluster_owner(construction.identity, destination);
        Ok(())
    }

    fn bind_connected_existing_result(
        &mut self,
        namespace: NamespaceNodeId,
        binder_name: &str,
        slot: &lang_syntax::NormBindingSlot,
        namespace_declaration: &NamespaceDeclarationPolicy,
        result: Vec<crate::PolicyResultEntry<crate::SemanticValueRef, crate::PatternValueId>>,
        provenance: Provenance,
    ) -> Result<(), BuildError> {
        assert_semantic_result_satisfies_annotation(
            &self.semantic_world,
            slot.annotation.as_ref(),
            &result,
            provenance.clone(),
        )?;

        let explicit_p1 = slot
            .policy
            .as_ref()
            .map(|_| &namespace_declaration.projection);
        let elaborated =
            crate::elaborate_value_binding_p1(&result, explicit_p1, provenance.clone()).map_err(
                |failure| {
                    BuildError::single(
                        Diagnostic::hard_error(
                            format!(
                                "ExplicitPolicyProjectionFailed: existing semantic value cannot satisfy binding P1 ({failure:?})"
                            ),
                            Some(provenance.clone()),
                        )
                        .with_code(ResolverCode::ExplicitPolicyVerificationFailed),
                    )
                },
            )?;
        let selected = match elaborated {
            crate::P1Elaboration::Projected { selected, .. } => selected,
            crate::P1Elaboration::AtomicRuntimeMigration { demands, .. } => {
                self.invoke_binding_migration_demands(demands, &provenance)?
            }
        };
        self.install_connected_semantic_binding(
            namespace,
            binder_name,
            namespace_declaration,
            &selected,
            provenance,
        )
        .map(|_| ())
    }

    /// Run binding-level atomic runtime migration demands and wrap every
    /// demanded entry into a fresh `InvocationResult` value.
    ///
    /// The raw `demanded_view` of `invoke_atomic_runtime_migration` keeps the
    /// identity semantics of the selected forwarding transport: its Val1 is
    /// the existing static source value.  A `let` binding, however, binds the
    /// migration *result* — a fresh runtime value recording the selected call
    /// entry and the migration source — never the static source value itself.
    fn invoke_binding_migration_demands(
        &mut self,
        demands: Vec<crate::PolicyTransitionDemand>,
        provenance: &Provenance,
    ) -> Result<
        Vec<crate::PolicyResultEntry<crate::SemanticValueRef, crate::PatternValueId>>,
        BuildError,
    > {
        let mut demanded_views = Vec::new();
        for demand in demands {
            let migration = self
                .invoke_atomic_runtime_migration(&demand.request)
                .map_err(|failure| {
                    BuildError::single(ordinary_invocation_failure_diagnostic(
                        failure,
                        provenance.clone(),
                    ))
                })?;
            let selected_call_entry = migration.invocation.selected.call_entry_value;
            for mut entry in migration.demanded_view {
                if let Some(source) = entry.value {
                    let result_policy = PolicyPair {
                        value: entry.value_policy.clone(),
                        pattern: entry.pattern_policy.clone(),
                    };
                    let result_value = self.semantic_world.install_invocation_result(
                        selected_call_entry,
                        Some(source.id),
                        source.type_value,
                        entry.pattern,
                        result_policy,
                        provenance.clone(),
                    );
                    entry.value = Some(crate::SemanticValueRef {
                        id: result_value,
                        type_value: source.type_value,
                    });
                }
                demanded_views.push(entry);
            }
        }
        Ok(demanded_views)
    }

    /// Installs the selected connected result views under a fresh
    /// destination Symbol and returns that Symbol's identity.
    fn install_connected_semantic_binding(
        &mut self,
        namespace: NamespaceNodeId,
        binder_name: &str,
        namespace_declaration: &NamespaceDeclarationPolicy,
        selected: &[crate::PolicyResultEntry<crate::SemanticValueRef, crate::PatternValueId>],
        provenance: Provenance,
    ) -> Result<crate::SemanticSymbolIdentity, BuildError> {
        let Some(first_pattern) = selected.first().map(|entry| entry.pattern) else {
            return Err(BuildError::single(Diagnostic::hard_error(
                "ordinary semantic binding produced no result entries",
                Some(provenance),
            )));
        };
        if selected.iter().any(|entry| entry.pattern != first_pattern) {
            return Err(BuildError::single(Diagnostic::hard_error(
                "ordinary semantic binding cannot install result entries with different PatternValue identities under one Symbol",
                Some(provenance),
            )));
        }
        let policy = legacy_policy_set_from_result_entries(selected);
        let mut declared_type_carrier = None;
        if selected.iter().all(|entry| entry.value.is_none()) {
            let pattern = first_pattern;
            let represented_type = self
                .semantic_world
                .type_for_pattern(pattern)
                .ok_or_else(|| {
                    BuildError::single(Diagnostic::hard_error(
                        "ordinary semantic binding: pure-P result uses an unregistered PatternValue",
                        Some(provenance.clone()),
                    ))
                })?;
            let mut delta = {
                let carrier = declared_bound_type_value_delta(
                    self.semantic_world.namespace_index(),
                    namespace,
                    binder_name,
                    represented_type,
                    provenance.clone(),
                );
                declared_type_carrier = Some((
                    carrier.symbol_id,
                    represented_type,
                    carrier.associated_namespace,
                ));
                carrier.delta
            };
            override_delta_binding_policy(&mut delta, binder_name, policy);
            override_delta_binding_visibility(&mut delta, binder_name, namespace_declaration);
            let pure_selected: Vec<_> = selected
                .iter()
                .map(|entry| crate::PolicyResultEntry {
                    value: None,
                    value_policy: entry.value_policy.clone(),
                    pattern: entry.pattern,
                    pattern_policy: entry.pattern_policy.clone(),
                })
                .collect();
            let destination = self
                .semantic_world
                .bind_ordinary_new(namespace, binder_name, &pure_selected, provenance.clone())
                .map_err(|conflict| bind_conflict_error(conflict, binder_name, &provenance))?;
            // Install the authoritative semantic carrier
            // before its compatibility rendering.
            if let Some((symbol_id, represented_type, associated_namespace)) = declared_type_carrier
            {
                self.register_installed_type_carrier(
                    namespace,
                    binder_name,
                    symbol_id,
                    represented_type,
                    Some(associated_namespace),
                    declared_pair_from_result_entry(&selected[0], namespace_declaration),
                    provenance,
                )?;
            }
            self.semantic_world.install_namespace_name_delta(delta)?;
            return Ok(destination);
        }

        // Extracting represented_type from the TypeObject
        // carrier is a legitimate type-system boundary: `let name: type = T`
        // must read the underlying SemanticTypeValue to install the binding.
        // This is NOT ordinary-semantic-algorithm leakage.
        let represented_type = selected
            .iter()
            .filter_map(|entry| entry.value)
            .map(|value| {
                self.semantic_world
                    .value(value.id)
                    .and_then(|value| match value.payload {
                        crate::SemanticValuePayload::TypeObject {
                            represented_type, ..
                        } => Some(represented_type),
                        _ => None,
                    })
            })
            .collect::<Option<Vec<_>>>()
            .and_then(|values| {
                let first = values.first().copied()?;
                values.iter().all(|value| *value == first).then_some(first)
            });
        let mut delta = if let Some(represented_type) = represented_type {
            let carrier = declared_bound_type_value_delta(
                self.semantic_world.namespace_index(),
                namespace,
                binder_name,
                represented_type,
                provenance.clone(),
            );
            declared_type_carrier = Some((
                carrier.symbol_id,
                represented_type,
                carrier.associated_namespace,
            ));
            carrier.delta
        } else {
            self.semantic_world.namespace_index().capability().declare(
                namespace,
                binder_name.to_string(),
                SymbolKind::Placeholder,
                SourceCategory::DeclaredSymbol,
                provenance.clone(),
            )
        };
        override_delta_binding_policy(&mut delta, binder_name, policy);
        override_delta_binding_visibility(&mut delta, binder_name, namespace_declaration);
        let destination = self
            .semantic_world
            .bind_ordinary_new(namespace, binder_name, selected, provenance.clone())
            .map_err(|conflict| bind_conflict_error(conflict, binder_name, &provenance))?;
        // Install the authoritative semantic carrier before
        // its compatibility rendering.
        if let Some((symbol_id, represented_type, associated_namespace)) = declared_type_carrier {
            self.register_installed_type_carrier(
                namespace,
                binder_name,
                symbol_id,
                represented_type,
                Some(associated_namespace),
                declared_pair_from_result_entry(&selected[0], namespace_declaration),
                provenance,
            )?;
        }
        self.semantic_world.install_namespace_name_delta(delta)?;
        Ok(destination)
    }

    /// Installs a connected meta construction result whose unique type
    /// member is backed by a generated type definition.  The namespace side
    /// reuses the full generated-type expansion (TypeObject with fields,
    /// field-function projection layer, ref/share projection namespaces),
    /// while the semantic side binds the construction's member views under
    /// a fresh destination Symbol — the same canonical facts as the plain
    /// carrier path, plus the namespace projection the plain carrier lacks.
    fn install_connected_generated_type_binding(
        &mut self,
        namespace: NamespaceNodeId,
        binder_name: &str,
        namespace_declaration: &NamespaceDeclarationPolicy,
        selected: &[crate::PolicyResultEntry<crate::SemanticValueRef, crate::PatternValueId>],
        generated: crate::GeneratedTypeDefinitionValue,
        provenance: Provenance,
    ) -> Result<crate::SemanticSymbolIdentity, BuildError> {
        let result_policy = legacy_policy_set_from_result_entries(selected);
        let mut expansion = bind_meta_invocation_value_result_with_materialization_state(
            crate::MetaInvocationValue::GeneratedTypeDefinitionValue(generated),
            self.semantic_world.namespace_index(),
            namespace,
            binder_name,
            provenance.clone(),
            &mut self.type_materialization_state,
        )?;
        override_delta_binding_policy(
            &mut expansion.namespace_delta,
            binder_name,
            result_policy.clone(),
        );
        override_delta_binding_visibility(
            &mut expansion.namespace_delta,
            binder_name,
            namespace_declaration,
        );
        expansion.replacement_object.policy_metadata.policy_set = result_policy;
        expansion
            .replacement_object
            .visibility_metadata
            .namespace_visibility = namespace_declaration.visibility;
        expansion.replacement_object.visibility_metadata.export_root =
            namespace_declaration.export_root;
        let destination = self
            .semantic_world
            .bind_ordinary_new(namespace, binder_name, selected, provenance.clone())
            .map_err(|conflict| bind_conflict_error(conflict, binder_name, &provenance))?;
        // Semantic type and projection Symbols are
        // installed before their compatibility rendering.
        if let Some(entry) = selected.first() {
            self.register_expansion_type_carrier(
                &expansion.replacement_object,
                namespace,
                declared_pair_from_result_entry(entry, namespace_declaration),
            )?;
        }
        self.semantic_world
            .register_generated_projection_symbols(&expansion.namespace_delta)?;
        self.semantic_world
            .install_namespace_name_delta(expansion.namespace_delta)?;
        self.diagnostics.extend(expansion.diagnostics);
        Ok(destination)
    }

    fn evaluate_initializer_best_effort_connected(
        &mut self,
        namespace: NamespaceNodeId,
        initializer: &NormExpr,
        mode: EvalMode,
        provenance: Provenance,
    ) -> ConnectedInitializerOutcome {
        if let Ok(call_site) = crate::extract_single_call_site(initializer) {
            if self
                .resolve_semantic_call_target(namespace, &call_site.target)
                .is_some()
            {
                let explicit_mutability =
                    vec![crate::ValueMutability::Const; call_site.source_product.elements.len()];
                // B8: a world-level connected declaration's environment is the
                // namespace level itself (no enclosing callable), so the
                // ambient construction owner is supplied explicitly here.  A
                // future callable-body evaluator must supply the enclosing
                // anonymous function object's Self scope owner instead.
                let mut context =
                    crate::OrdinaryInvocationContext::open_static(&explicit_mutability);
                context.ambient_construction_owner = self.semantic_world.namespace_owner(namespace);
                return match self.invoke_ordinary_call(
                    namespace,
                    &call_site,
                    context,
                    provenance.clone(),
                ) {
                    Ok(result) => ConnectedInitializerOutcome::Ordinary(result),
                    // Meta-partial residualization: a resolvable target whose
                    // candidate set exposes nothing admissible at the static
                    // phase (e.g. a runtime-only body entry) defers the
                    // binding to runtime instead of hard-failing the build.
                    // P1 never expands such a callable into meta visibility.
                    // A candidate that was reached but failed to assemble or
                    // execute (`first_diagnostic: Some`) is a real error and
                    // is never residualized.
                    Err(
                        crate::OrdinaryInvocationFailure::NoTargetValues { .. }
                        | crate::OrdinaryInvocationFailure::NoFullyAdmissibleCandidate {
                            first_diagnostic: None,
                            ..
                        },
                    ) if mode == EvalMode::MetaPartial => ConnectedInitializerOutcome::Residual {
                        reason: crate::ResidualReason::NoMetaVisibleCandidate,
                        provenance,
                    },
                    Err(failure) => ConnectedInitializerOutcome::Diagnostic(
                        ordinary_invocation_failure_diagnostic(failure, provenance),
                    ),
                };
            }
        }

        if let Some(existing) = self.existing_semantic_result(namespace, initializer) {
            return ConnectedInitializerOutcome::Existing(existing);
        }

        // No second semantic machine.  An initializer
        // whose call target does not resolve to a semantic cluster Symbol
        // and which names no existing semantic material is residualized as
        // unsupported; no second evaluator is reachable from the connected
        // world.
        ConnectedInitializerOutcome::Residual {
            reason: crate::ResidualReason::UnsupportedExpression,
            provenance,
        }
    }

    fn existing_semantic_result(
        &self,
        namespace: NamespaceNodeId,
        initializer: &NormExpr,
    ) -> Option<Vec<crate::PolicyResultEntry<crate::SemanticValueRef, crate::PatternValueId>>> {
        let symbol = self
            .resolve_semantic_call_target(namespace, initializer)?
            .symbol;
        let symbol = self.semantic_world.symbol(symbol)?;
        let entries = symbol
            .member_views
            .iter()
            .map(|entry| crate::PolicyResultEntry {
                value: entry
                    .value
                    .map(|id| {
                        let value = self
                            .semantic_world
                            .value(id)
                            .expect("semantic Symbol view references an installed value");
                        Some(crate::SemanticValueRef {
                            id,
                            type_value: value.type_value,
                        })
                    })
                    .unwrap_or_default(),
                value_policy: entry.value_policy.clone(),
                pattern: entry.pattern,
                pattern_policy: entry.pattern_policy.clone(),
            })
            .collect::<Vec<_>>();
        if entries.is_empty() {
            return None;
        }
        Some(entries)
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
        // The alias target resolves through the semantic
        // world's own path material and the forwarding entry installs there
        // first; the graph alias delta below is a demoted name mirror, not
        // the resolution or installation authority.
        let target_identity = self
            .semantic_world
            .resolve_symbol_path(
                &target_path,
                namespace,
                &[self.semantic_world.namespace_index().root_node()],
                &[self.core_node],
            )
            .map_err(BuildError::single)?;
        self.semantic_world
            .bind_alias_symbol(namespace, &name, target_identity)
            .ok_or_else(|| {
                BuildError::single(Diagnostic::hard_error(
                    format!(
                        "alias declaration error: `{name}` conflicts with an existing member in this namespace"
                    ),
                    Some(Provenance::from_norm_origin("alias declaration", origin)),
                ))
            })?;
        // Compatibility declaration rendering; semantic selection has already
        // resolved the target.
        let context = ResolverContext::with_mounts(
            namespace,
            vec![self.semantic_world.namespace_index().root_node()],
            vec![self.core_node],
        );
        let target_symbol = self
            .semantic_world
            .namespace_index()
            .capability()
            .resolve(&target_path, &context)
            .map_err(BuildError::single)?;
        let mut delta = self.semantic_world.namespace_index().capability().alias(
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
        self.semantic_world.install_namespace_name_delta(delta)?;
        Ok(())
    }
}

/// True when the binding's explicit P1 can only be satisfied by a present
/// runtime value (possibly via atomic migration): the demanded value stage
/// set is runtime-only.  A union P1 such as `meta || runtime` keeps a static
/// slice and is projected from the result directly, never force-materialized.
fn projection_requests_runtime_value(projection: &crate::P1Projection) -> bool {
    let value = match projection {
        crate::P1Projection::ValueDominant { value } => value,
        crate::P1Projection::Pair(pair) => &pair.value,
        crate::P1Projection::Infer => return false,
    };
    value.presence != ValuePresence::Absent
        && value.stages.contains(crate::PolicyStage::Runtime)
        && value.stages.static_stages().is_empty()
}

fn ensure_declared_namespace_path(
    snapshot: &mut SemanticNameIndex,
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
    snapshot: &mut SemanticNameIndex,
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

fn ensure_namespace_path(
    snapshot: &mut SemanticNameIndex,
    root: NamespaceNodeId,
    components: &[String],
    node_kind: NamespaceNodeKind,
    source_category: SourceCategory,
    provenance_description: &str,
) -> Result<NamespaceNodeId, BuildError> {
    let mut current = root;
    for component in components {
        // Reuse any existing namespace-capable symbol for this component:
        // either a declared namespace-subspace symbol or an object symbol
        // carrying an associated namespace node (e.g. a type symbol's
        // type-associated namespace).  Mount paths like `core::uint8` must
        // land inside `uint8`'s associated namespace instead of declaring a
        // conflicting cross-role namespace symbol.
        if let Ok(existing) = snapshot.child_symbol_with_expectation(
            current,
            component,
            ResolveExpectation::NamespaceCapableParent,
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
    snapshot: &SemanticNameIndex,
    parent: NamespaceNodeId,
    name: &str,
    provenance: Provenance,
) -> DeclaredTypeCarrierDelta {
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
    let represented_type = crate::type_value_projection_from_type_symbol(type_symbol_id);
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
        carrier_symbol_id: type_symbol_id,
        represented_type,
        owner_pattern_head: None,
        fields: Vec::new(),
        field_names: Vec::new(),
        field_type_values: Vec::new(),
        field_type_symbol_ids: Vec::new(),
        type_associated_namespace: Some(type_namespace_id),
        extraction_interface: None,
        provenance,
        generation_origin: None,
        layout_slot: None,
        abi_slot: None,
    });
    delta.insert_symbol(parent, symbol);
    DeclaredTypeCarrierDelta {
        delta,
        symbol_id: type_symbol_id,
        represented_type,
        associated_namespace: type_namespace_id,
    }
}

/// One declared type-carrier installation: its compatibility projection plus
/// the carrier facts (`SymbolId`, represented `TypeValue`, associated
/// namespace) installed directly into SemanticWorld.
struct DeclaredTypeCarrierDelta {
    delta: SemanticNameDelta,
    symbol_id: crate::SymbolId,
    represented_type: crate::TypeValueId,
    associated_namespace: NamespaceNodeId,
}

/// The semantic world is the declaration-conflict
/// authority: an ordinary binding that lands on an occupied semantic Symbol
/// is a hard declaration conflict, reported before any graph mirror runs.
fn bind_conflict_error(
    conflict: crate::semantic_world::BindConflict,
    binder_name: &str,
    provenance: &Provenance,
) -> BuildError {
    let message = match conflict {
        crate::semantic_world::BindConflict::AlreadyBound { name, .. } => format!(
            "declaration conflict: `{name}` is already bound in this namespace"
        ),
        other => format!(
            "declaration binding error: binding `{binder_name}` failed in the semantic world ({other:?})"
        ),
    };
    BuildError::single(Diagnostic::hard_error(message, Some(provenance.clone())))
}

fn declared_bound_type_value_delta(
    snapshot: &SemanticNameIndex,
    parent: NamespaceNodeId,
    name: &str,
    represented_type: crate::TypeValueId,
    provenance: Provenance,
) -> DeclaredTypeCarrierDelta {
    let mut carrier = declared_type_placeholder_delta(snapshot, parent, name, provenance);
    let symbol = carrier
        .delta
        .symbols
        .values_mut()
        .find(|symbol| symbol.name == name)
        .expect("declared type-value delta contains its carrier");
    let SymbolPayload::Type(type_object) = &mut symbol.payload else {
        unreachable!("declared type-value carrier is a Type object");
    };
    type_object.represented_type = represented_type;
    type_object.generation_origin = Some("ordinary evaluated TypeValue binding".to_string());
    carrier.represented_type = represented_type;
    carrier
}

/// The declared canonical `PolicyPair` for a type-carrier binding: the
/// explicit declaration projection when the user wrote one, otherwise the
/// natural `meta runtime` type-carrier pair. Namespace attributes remain on
/// the declaration object and never enter this `Pv:Pp` value.
fn declared_type_binding_pair(namespace_declaration: &NamespaceDeclarationPolicy) -> PolicyPair {
    match &namespace_declaration.projection {
        crate::P1Projection::Pair(pair) => pair.clone(),
        crate::P1Projection::ValueDominant { value } => PolicyPair {
            value: value.clone(),
            pattern: PatternComponentPolicy {
                stages: value.stages.static_stages(),
            },
        },
        crate::P1Projection::Infer => {
            core_declared_pair(&[PolicyStage::Meta, PolicyStage::Runtime], false)
        }
    }
}

/// The declared canonical `PolicyPair` carried by one connected result entry.
/// Binding visibility/export remain separate declaration attributes.
fn declared_pair_from_result_entry<V, P>(
    entry: &crate::PolicyResultEntry<V, P>,
    _namespace_declaration: &NamespaceDeclarationPolicy,
) -> PolicyPair {
    PolicyPair {
        value: entry.value_policy.clone(),
        pattern: entry.pattern_policy.clone(),
    }
}

struct SourceCallableDelta {
    delta: SemanticNameDelta,
    symbol_id: crate::SymbolId,
    /// User-written outer P1 (the `let name: policy = ...` policy), or None
    /// if the user wrote no explicit policy. This is kept separate from
    /// `function_policy` (which is the mechanically derived P1 =
    /// `derive_function_object_p1(&p2, ...)`) so the canonicalizer can
    /// distinguish "outer explicit" from "outer derived" and apply the
    /// canonical-P1 conflict rule.
    outer_p1_explicit: Option<ExplicitP1Selection>,
    function_policy: PolicyPair,
    result_policy: PolicyPair,
    namespace_visibility: Option<crate::NamespaceVisibility>,
    /// Independent declared return-shape coordinate, elaborated once from
    /// the return-slot annotation (`declared_return_shape_from_closure`)
    /// and validated against the result P2 (`validate_return_shape`).
    /// Registration sites mirror it onto the call entry verbatim.
    return_shape: crate::ReturnShape,
}

fn source_callable_delta(
    snapshot: &SemanticNameIndex,
    parent: NamespaceNodeId,
    name: &str,
    policy_expr: Option<&NormPolicySpec>,
    closure: &NormClosure,
    provenance: Provenance,
) -> Result<SourceCallableDelta, BuildError> {
    let result_p2 =
        result_policy_from_closure(closure, provenance.clone()).map_err(BuildError::single)?;
    // Semantic invariant — a runtime-only result P2 normalizes to
    // `runtime:compile` (`N2(runtime) = runtime:compile`), whose value stage
    // is disjoint from its Pattern stage. A pure-P return slot
    // (`let r: type`) declares a member with no value dimension, so the
    // declared runtime value slice could never be filled: the declaration
    // itself is a hard error at this elaboration boundary.
    ensure_runtime_result_slice_has_value_dimension(closure, &result_p2, provenance.clone())
        .map_err(BuildError::single)?;
    // Elaborate the declared return shape once at this boundary from the
    // return-slot annotation: `-> r: symbol` declares a ClusterSymbol
    // return, `-> r: type` a single pure-P type, `_: unit` the value-less
    // pure shape; any other slot declares a single value.  The body form
    // family is never scanned (the shape is a declared fact, not an
    // inference from the implementation), and the Policy stage is never
    // consulted — `: meta ->` governs visibility and execution timing
    // only.  `Validate(P2, ReturnShape)` is then the legality relation
    // between the two independent coordinates, not a derivation in either
    // direction: the core criterion is that meta-legal returns occupy
    // exactly one position.
    let return_shape = crate::overload_set::declared_return_shape_from_closure(closure)
        .map_err(BuildError::single)?;
    crate::policy_pair::validate_return_shape(return_shape, &result_p2, &provenance)
        .map_err(BuildError::single)?;
    let namespace_declaration = elaborate_namespace_declaration_policy(
        policy_expr,
        NamespaceDeclarationPosition::DirectTopLevel,
        provenance.clone(),
    )
    .map_err(BuildError::single)?;
    let declaration_policy = function_object_declaration_policy(&namespace_declaration);
    let derived_function_p1 = derive_function_object_p1(&result_p2, &declaration_policy);
    let derived_symbol_policy = legacy_policy_set_from_pair(&derived_function_p1);
    let explicit_symbol_policy = policy_expr
        .map(|policy| elaborate_declaration_policy_expr(Some(policy), provenance.clone()))
        .transpose()
        .map_err(BuildError::single)?;
    verify_explicit_policy_compatible(
        explicit_symbol_policy.as_ref(),
        &derived_symbol_policy,
        provenance.clone(),
    )?;
    let symbol_policy =
        final_binding_policy(explicit_symbol_policy.as_ref(), &derived_symbol_policy);
    let body_entry_policy = legacy_policy_set_from_pair(&result_p2);
    ensure_return_policy_supported(closure, provenance.clone()).map_err(BuildError::single)?;

    // Preserve explicitness information at the
    // declaration elaboration boundary. `outer_p1_explicit` is `Some` iff
    // the user wrote P1-relevant material in the `let name: policy = ...`
    // prefix. It is kept separate from `derived_function_p1` (the
    // mechanically derived P1 = `derive_function_object_p1(&p2, ...)`) so
    // the canonicalizer can apply the per-dimension conflict rule:
    //   outer explicit + self explicit => must agree per dimension
    //   outer explicit only            => canonical = outer selection
    //   self explicit only             => canonical = self selection
    //   neither                        => canonical = derived
    //
    // The explicit P1 is the COMPLETE `Pv:Pp` selection: stage atoms in
    // the prefix are an explicit value-stage selection (they double as
    // declaration policy, already validated against the derived symbol
    // policy above via `verify_explicit_policy_compatible`), while
    // `public/private/export` remain namespace declaration attributes and
    // never enter the P1.
    let outer_p1_explicit: Option<ExplicitP1Selection> = crate::policy_pair::elaborate_explicit_p1(
        policy_expr,
        &result_p2,
        crate::policy_pair::ExplicitP1Position::OuterBinding,
        provenance.clone(),
    )
    .map_err(BuildError::single)?;

    let mut delta = snapshot.empty_delta();
    let symbol_id = delta.allocate_symbol_id();
    // Current v0.9 integration is validation-only at source harvesting time.
    // Bound return events are not stored in SourceCallableObject yet; later
    // evaluators may re-run the return-target binder when they need the bound
    // event stream for completion/result semantics.
    let return_target_report = elaborate_return_targets_in_returnable_closure(
        closure,
        ReturnFrameOwner::SourceCallable {
            symbol_id: Some(symbol_id),
            name: Some(name.to_string()),
        },
    );
    if !return_target_report.diagnostics.is_empty() {
        return Err(BuildError {
            diagnostics: return_target_report.diagnostics,
        });
    }

    let mut symbol = SymbolObject::placeholder(
        symbol_id,
        name,
        SymbolKind::MetaFunction,
        SourceCategory::DeclaredSymbol,
        Some(parent),
        provenance.clone(),
    );
    symbol.policy_metadata.policy_set = symbol_policy.clone();
    symbol.visibility_metadata.namespace_visibility = namespace_declaration.visibility;
    symbol.visibility_metadata.export_root = namespace_declaration.export_root;
    symbol.payload = SymbolPayload::MetaFunction(MetaFunctionObject {
        function_symbol_id: symbol_id,
        primitive: None,
        source_callable: Some(SourceCallableObject {
            closure: closure.clone(),
            provenance: provenance.clone(),
        }),
        function_policy: policy_metadata(symbol_policy.clone()),
        body_entry_policy: policy_metadata(body_entry_policy.clone()),
        return_object_policy: policy_metadata(body_entry_policy),
        return_shape,
        privilege: crate::CallablePrivilege::OrdinarySource,
    });
    delta.insert_symbol(parent, symbol);
    Ok(SourceCallableDelta {
        delta,
        symbol_id,
        outer_p1_explicit,
        function_policy: derived_function_p1,
        result_policy: result_p2,
        namespace_visibility: namespace_declaration.visibility,
        return_shape,
    })
}

fn legacy_policy_set_from_result_entries<V, P>(
    entries: &[crate::PolicyResultEntry<V, P>],
) -> PolicySet {
    let mut policy = PolicySet::new();
    for entry in entries {
        let projected = legacy_policy_set_from_pair(&PolicyPair {
            value: entry.value_policy.clone(),
            pattern: entry.pattern_policy.clone(),
        });
        policy.flags.extend(projected.flags);
    }
    policy
}

fn ordinary_invocation_failure_diagnostic(
    failure: crate::OrdinaryInvocationFailure,
    provenance: Provenance,
) -> Diagnostic {
    match failure {
        crate::OrdinaryInvocationFailure::SelectedDelete { diagnostic, .. }
        | crate::OrdinaryInvocationFailure::SelectedCoreBody { diagnostic, .. }
        | crate::OrdinaryInvocationFailure::CyclicVal2 { diagnostic, .. }
        | crate::OrdinaryInvocationFailure::MetaReturnTypeRootMismatch { diagnostic, .. }
        | crate::OrdinaryInvocationFailure::SelectedBody {
            failure: crate::RestrictedOverloadFailure { diagnostic, .. },
            ..
        } => diagnostic,
        crate::OrdinaryInvocationFailure::NoFullyAdmissibleCandidate {
            first_diagnostic: Some(diagnostic),
            ..
        } => diagnostic,
        crate::OrdinaryInvocationFailure::NoTargetValues { .. } => Diagnostic::hard_error(
            "ordinary invocation found no semantic target values",
            Some(provenance),
        )
        .with_code(ResolverCode::NoMetaVisibleCandidate),
        crate::OrdinaryInvocationFailure::NoFullyAdmissibleCandidate { .. } => {
            Diagnostic::hard_error(
                "ordinary invocation found no fully admissible candidate",
                Some(provenance),
            )
            .with_code(ResolverCode::NoMetaVisibleCandidate)
        }
        crate::OrdinaryInvocationFailure::Ambiguous { .. } => Diagnostic::hard_error(
            "ordinary invocation has multiple maximal candidates",
            Some(provenance),
        )
        .with_code(ResolverCode::AmbiguousMetaCandidate),
        crate::OrdinaryInvocationFailure::ResultTypeHasNoPattern { type_value, .. } => {
            Diagnostic::hard_error(
                format!(
                    "ordinary invocation result TypeValue {:?} has no installed PatternValue",
                    type_value
                ),
                Some(provenance),
            )
        }
        crate::OrdinaryInvocationFailure::MigrationResultTypeChanged { source, result, .. } => {
            Diagnostic::hard_error(
                format!(
                    "atomic Policy migration cannot change TypeValue: source {:?}, result {:?}",
                    source, result
                ),
                Some(provenance),
            )
        }
        crate::OrdinaryInvocationFailure::MigrationOutputProjectionFailed { .. } => {
            Diagnostic::hard_error(
                "ordinary migration result does not expose the demanded runtime view",
                Some(provenance),
            )
        }
    }
}

fn result_policy_from_closure(
    closure: &NormClosure,
    provenance: Provenance,
) -> Result<crate::PolicyPair, Diagnostic> {
    let Some(head) = &closure.head else {
        return Err(Diagnostic::hard_error(
            "source callable declaration requires an explicit closure head",
            Some(provenance),
        ));
    };
    let Some(annotation) = &head.call_policy else {
        return Err(Diagnostic::hard_error(
            "source callable declaration requires a P2 annotation such as `: meta ->`",
            Some(provenance),
        ));
    };
    normalize_p2_policy(annotation, provenance)
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

/// A runtime-only result P2 (all value stages == `runtime`) paired with a
/// pure-P return slot (`let r: type`) declares a runtime value slice that
/// carries no value dimension. `N2(runtime) = runtime:compile` makes Pv
/// disjoint from Pp, so the declared value slice can never be filled by a
/// pure-P member; the declaration is rejected here. Static single policies
/// keep Pv == Pp (`N2(P) = P:(P - runtime)`), so pure-P return slots under
/// `meta`/`compile`/`seal` remain legal.
fn ensure_runtime_result_slice_has_value_dimension(
    closure: &NormClosure,
    result_p2: &PolicyPair,
    provenance: Provenance,
) -> Result<(), Diagnostic> {
    let runtime_only = result_p2.value.stages.contains(PolicyStage::Runtime)
        && result_p2.value.stages.static_stages().is_empty();
    if !runtime_only {
        return Ok(());
    }
    let pure_p_return_slot = closure
        .head
        .as_ref()
        .and_then(|head| head.returns.as_ref())
        .is_some_and(|returns| is_type_annotation(returns.annotation.as_ref()));
    if pure_p_return_slot {
        return Err(Diagnostic::hard_error(
            "runtime-only result P2 (`: runtime ->` = `runtime:compile`) declares a runtime \
             value slice, but the pure-P return slot (`let r: type`) carries no value \
             dimension; the declaration is illegal",
            Some(provenance),
        )
        .with_code(ResolverCode::RuntimeSliceWithoutValueDimension));
    }
    Ok(())
}

fn assert_semantic_result_satisfies_annotation(
    semantic_world: &crate::SemanticWorld,
    annotation: Option<&NormAnnotation>,
    result: &[crate::PolicyResultEntry<crate::SemanticValueRef, crate::PatternValueId>],
    provenance: Provenance,
) -> Result<(), BuildError> {
    if !is_type_annotation(annotation) {
        return Ok(());
    }
    // Validating that `: type` annotation results carry
    // a TypeObject payload is a legitimate type-system boundary: the
    // annotation asserts the value IS a type.  This is type checking, not
    // ordinary-semantic-algorithm leakage.
    let value_bearing = result.iter().filter_map(|entry| entry.value);
    for value in value_bearing {
        if !matches!(
            semantic_world.value(value.id).map(|value| &value.payload),
            Some(crate::SemanticValuePayload::TypeObject { .. })
        ) {
            return Err(BuildError::single(
                Diagnostic::hard_error(
                    "AnnotationAssertionFailed: `: type` expects the evaluated ordinary result value to be a TypeValue",
                    Some(provenance),
                )
                .with_code(ResolverCode::AnnotationAssertionFailed),
            ));
        }
    }
    Ok(())
}

fn verify_explicit_policy_compatible(
    explicit_policy: Option<&PolicySet>,
    result_policy: &PolicySet,
    provenance: Provenance,
) -> Result<(), BuildError> {
    let Some(explicit_policy) = explicit_policy else {
        return Ok(());
    };
    if policy_projection(explicit_policy, result_policy).is_some() {
        Ok(())
    } else {
        Err(BuildError::single(Diagnostic::hard_error(
            "ExplicitPolicyProjectionFailed: explicit binding policy selects an empty RHS slice",
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
    let runtime = policy_set_runtime();
    if policy_projection(explicit_policy, &runtime).is_some() {
        return Ok(());
    }
    Err(BuildError::single(Diagnostic::hard_error(
        format!(
            "ExplicitPolicyProjectionFailed: RHS residualized to runtime ({reason:?}) and the requested binding policy selects no runtime value slice"
        ),
        Some(provenance),
    )
    .with_code(ResolverCode::ExplicitPolicyVerificationFailed)))
}

fn is_stage_flag(flag: PolicyFlag) -> bool {
    matches!(
        flag,
        PolicyFlag::Meta | PolicyFlag::Compile | PolicyFlag::Seal | PolicyFlag::Runtime
    )
}

fn policy_projection(requested: &PolicySet, available: &PolicySet) -> Option<PolicySet> {
    let requested_stages = requested
        .flags
        .iter()
        .copied()
        .filter(|flag| is_stage_flag(*flag))
        .collect::<Vec<_>>();
    let mut selected = PolicySet::new();
    if requested_stages.is_empty() {
        selected.flags.extend(
            available
                .flags
                .iter()
                .copied()
                .filter(|flag| is_stage_flag(*flag)),
        );
    } else {
        selected.flags.extend(
            requested_stages
                .into_iter()
                .filter(|flag| available.contains(*flag)),
        );
        if selected.flags.is_empty() {
            return None;
        }
    }
    if requested.contains(PolicyFlag::Export) {
        selected.insert(PolicyFlag::Export);
    }
    Some(selected)
}

fn final_binding_policy(
    explicit_policy: Option<&PolicySet>,
    result_policy: &PolicySet,
) -> PolicySet {
    if let Some(explicit_policy) = explicit_policy {
        return policy_projection(explicit_policy, result_policy)
            .expect("explicit policy was verified before final binding projection");
    }
    let mut inferred = result_policy.clone();
    inferred.flags.remove(&PolicyFlag::Export);
    inferred
}

fn projection_matches_expectation(object: &SymbolObject, expectation: ResolveExpectation) -> bool {
    match expectation {
        ResolveExpectation::AnyUnique | ResolveExpectation::Object => {
            object.kind != SymbolKind::Namespace
        }
        ResolveExpectation::NamespaceSubspace => object.kind == SymbolKind::Namespace,
        ResolveExpectation::NamespaceCapableParent => object.namespace_node().is_some(),
        ResolveExpectation::TypeObject => object.kind == SymbolKind::Type,
        ResolveExpectation::MetaFunction => object.kind == SymbolKind::MetaFunction,
        ResolveExpectation::FieldFunction => object.kind == SymbolKind::FieldFunction,
    }
}

/// Rewrites the flat `policy_metadata.policy_set` on declaration-projection
/// records matched by name.  The flat PolicySet is compatibility metadata,
/// not canonical member visibility authority, and must never be read back to
/// derive or overwrite `SemanticSymbolCell.member_views`.
fn override_delta_binding_policy(
    delta: &mut SemanticNameDelta,
    binding_name: &str,
    policy: PolicySet,
) {
    for symbol in delta.symbols.values_mut() {
        if symbol.name == binding_name {
            symbol.policy_metadata.policy_set = policy.clone();
        }
    }
}

fn override_delta_binding_visibility(
    delta: &mut SemanticNameDelta,
    binding_name: &str,
    declaration: &NamespaceDeclarationPolicy,
) {
    for symbol in delta.symbols.values_mut() {
        if symbol.name == binding_name {
            symbol.visibility_metadata.namespace_visibility = declaration.visibility;
            symbol.visibility_metadata.export_root = declaration.export_root;
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
        | NormPattern::Pack { origin, .. }
        | NormPattern::Unit { origin }
        | NormPattern::HoleRef { origin, .. }
        | NormPattern::AnonymousHole { origin }
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
