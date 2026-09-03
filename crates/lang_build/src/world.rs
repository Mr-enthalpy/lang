use std::path::Path;

use lang_syntax::{
    norm::NormNavComponent, NormAnnotation, NormClosure, NormDecl, NormExpr, NormForm, NormOrigin,
    NormPattern, NormPolicySpec, NormProgram,
};

use crate::{
    core::{core_declared_pair, install_core_bootstrap},
    discovery::{DiscoveredSourceUnit, SourceDiscoveryConfig, SourceDiscoveryReport},
    initializer_eval::EvalMode,
    manifest::{BuildManifest, NamespaceMount},
    meta::expand_struct_construction_material,
    model::{
        CoreTypeProjection, Diagnostic, DiagnosticSeverity, MetaFunctionObject, NamespaceNode,
        NamespaceNodeId, NamespaceNodeKind, Provenance, ResolverCode, SemanticNameDelta,
        SourceCallableObject, SourceCategory, SymbolKind, SymbolObject, SymbolPayload,
    },
    policy_pair::{
        declared_policy_view, derive_function_object_view, elaborate_binding_result_demand,
        elaborate_namespace_declaration_policy, elaborate_return_policy_pattern,
        function_object_declaration_policy, normalize_p2_policy, ExplicitP1Selection,
        NamespaceDeclarationPolicy, NamespaceDeclarationPosition, P1Projection, PolicyMode,
        PolicyView, ResultPolicyDemand, ValueComponentPolicy,
    },
    policy_pair::{PatternComponentPolicy, PolicyPair, PolicyStage, ValuePresence},
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

/// One resolved source target together with the COMPLETE host chain it was
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
    Existing(ConnectedExistingResult),
    Residual {
        reason: crate::ResidualReason,
        provenance: Provenance,
    },
    Diagnostic(Diagnostic),
}

#[derive(Clone, Debug)]
struct ConnectedExistingResult {
    material: Vec<crate::PolicyResultEntry<crate::SemanticValueRef, crate::PatternValueId>>,
    /// Exact complete tau carried by an existing type result. This coordinate
    /// comes from the semantic binding's immutable snapshot, never from a
    /// CoreTypeProjection graph payload.
    complete_type: Option<crate::CompleteTypeValue>,
}

const CONSTRUCT_OR_CONVERT_SELECTOR: &str = "<ConstructOrConvert>";

fn policy_let_target_demand(demand: &ResultPolicyDemand, source: &PolicyPair) -> PolicyView {
    let (value_query, pattern_query) = match &demand.pair_query {
        P1Projection::Infer => (None, None),
        P1Projection::ValueDominant { value } => (Some(value), None),
        P1Projection::Pair(pair) => (Some(&pair.value), Some(&pair.pattern)),
    };
    let value_stages = value_query
        .filter(|query| !query.stages.is_empty())
        .map(|query| query.stages.clone())
        .unwrap_or_else(|| source.value.stages.clone());
    let pattern_stages = pattern_query
        .filter(|query| !query.stages.is_empty())
        .map(|query| query.stages.clone())
        .unwrap_or_else(|| source.pattern.stages.clone());
    PolicyView {
        pair: PolicyPair {
            value: ValueComponentPolicy {
                stages: value_stages,
                presence: ValuePresence::Present,
            },
            pattern: PatternComponentPolicy {
                stages: pattern_stages,
            },
        },
        mode: demand.mode,
    }
}

/// Build and namespace world.
///
/// This is the canonical holder for source fragments, the default core mount,
/// and one connected [`SemanticWorld`].  Namespace topology is owned by that
/// semantic world; there is no separately committed namespace snapshot.
#[derive(Clone, Debug)]
pub struct CompilationWorld {
    package_root_node: NamespaceNodeId,
    core_node: NamespaceNodeId,
    semantic_world: SemanticWorld,
    /// One shared continuation/lifecycle state owned by the real evaluator.
    /// SemanticWorld stores object ontology; lifecycle remains an orthogonal
    /// evaluation-state judgment.
    lifecycle: crate::LifecycleMachine,
    source_fragments: Vec<SourceFragment>,
    diagnostics: Vec<Diagnostic>,
    /// Graph-projection declaration ids for compiler-internal call
    /// entries. Candidate identity is the semantic call-entry value; these
    /// ids never enter name lookup or selection.
    next_intrinsic_backing: u64,
}

impl CompilationWorld {
    pub fn from_manifest(manifest: &BuildManifest) -> Result<Self, BuildError> {
        if !manifest.default_core_mount {
            return Err(BuildError::single(Diagnostic::hard_error(
                "build manifest error: the default core mount is required",
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
            lifecycle: crate::LifecycleMachine::default(),
            source_fragments: Vec::new(),
            diagnostics: Vec::new(),
            next_intrinsic_backing: u64::MAX,
        };
        // Core type carriers enter the semantic world
        // straight from the bootstrap's declared registration roster; the
        // graph projection is not a semantic registration source.
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
        world.register_builtin_literal_constructors()?;
        // Core callables enter the semantic world straight
        // from the bootstrap's declared registration roster; there is no
        // graph-payload inference step.
        for registration in core_callables {
            world.semantic_world.register_core_callable(
                registration.namespace,
                &registration.name,
                registration.backing,
                registration.primitive,
                Some(ExplicitP1Selection::from_complete_view(
                    &registration.function_view,
                )),
                registration.declared_result_class,
                registration.function_view,
                registration.body_entry_view,
                registration.result_view,
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

    /// Read-only graph projection for diagnostics and graph-boundary tests.
    /// Namespace allocation, installation, and invocation
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

    pub fn lifecycle(&self) -> &crate::LifecycleMachine {
        &self.lifecycle
    }

    pub fn lifecycle_mut(&mut self) -> &mut crate::LifecycleMachine {
        &mut self.lifecycle
    }

    fn sync_lifecycle_values(&mut self) {
        let values = self
            .semantic_world
            .values()
            .map(|value| value.id)
            .collect::<Vec<_>>();
        for value in values {
            self.lifecycle.ensure_value(value);
        }
    }

    pub fn configure_call_entry_capability_realization(
        &mut self,
        entry: crate::SemanticValueId,
        realization: crate::CapabilityRealization,
    ) -> Result<(), crate::Diagnostic> {
        self.semantic_world
            .configure_call_entry_capability_realization(entry, realization)
    }

    /// `Addr(Norm_type(type_value, place))` — passthrough to the semantic
    /// world's type-observation interning, for callers (tests, expectation
    /// material) that need the observation address of an already-resolved
    /// type value.  Interning is content-idempotent, so replaying an
    /// observation already read at an invocation boundary returns the same
    /// address.
    pub fn canonical_complete_type_observation_address(
        &mut self,
        type_value: crate::TypeValueId,
        place: Option<crate::ObjectPlaceId>,
    ) -> Result<crate::CanonicalValueAddr, crate::Diagnostic> {
        self.semantic_world
            .canonical_complete_type_observation_address(type_value, place)
    }

    /// Ordinary semantic type-equality observation: `Addr(Norm(Core(tau)))`.
    pub fn canonical_type_core_observation_address(
        &mut self,
        type_value: crate::TypeValueId,
        place: Option<crate::ObjectPlaceId>,
    ) -> Result<crate::CanonicalValueAddr, crate::Diagnostic> {
        self.semantic_world
            .canonical_type_core_observation_address(type_value, place)
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

    /// Invoke one direct same-Type Policy migration through ordinary
    /// candidate enumeration and the sealed no-reopen invocation trunk.
    pub fn invoke_policy_migration(
        &mut self,
        request: &crate::PolicyMigrationRequest,
    ) -> Result<crate::PolicyMigrationResult, crate::OrdinaryInvocationFailure> {
        let resolver_context = self.root_context();
        crate::invoke_policy_migration(&mut self.semantic_world, request, &resolver_context)
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
                pattern,
                operation_name,
                receiver,
                explicit_args,
                &resolver_context,
                context,
                provenance,
            )
        } else {
            let mut explicit_modes = Vec::with_capacity(1 + context.explicit_argument_modes.len());
            explicit_modes.push(crate::PolicyMode::Plain);
            explicit_modes.extend_from_slice(context.explicit_argument_modes);
            let named_context = crate::OrdinaryInvocationContext {
                explicit_argument_modes: &explicit_modes,
                ..context
            };
            crate::invoke_pattern_associated_value_ordinary(
                &mut self.semantic_world,
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

    /// Every installation point that
    /// carries a `CoreTypeProjection` registers its semantic type binding through the
    /// atomic [`SemanticNamespaceDelta`] path with the declared canonical
    /// `PolicyPair`; the graph is not a registration source, and the
    /// type-associated namespace is created
    /// by the semantic world itself instead of being read back from the
    /// graph.
    fn register_installed_type_carrier(
        &mut self,
        namespace: NamespaceNodeId,
        name: &str,
        binding: crate::SymbolId,
        represented_type: crate::TypeValueId,
        complete_type: Option<crate::CanonicalValueAddr>,
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
                    complete_type,
                    associated_namespace: associated_namespace
                        .map(|node| (node, format!("{name}<type-associated>"))),
                    policy,
                    provenance,
                }],
            })
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

    /// Install core builtin implementations as ordinary target-callspace
    /// candidates.  `NumericTypeRegistry` supplies bootstrap lookup/data only;
    /// annotation evaluation never consults it for legality.
    fn register_builtin_literal_constructors(&mut self) -> Result<(), BuildError> {
        let registry =
            crate::NumericTypeRegistry::from_core_world(self).map_err(BuildError::single)?;
        for spec in registry.builtin_constructor_specs() {
            let provenance =
                Provenance::new(format!("builtin {:?} literal constructor", spec.target_key));
            let view = PolicyView {
                pair: crate::compile_literal_policy(),
                mode: PolicyMode::Plain,
            };
            let backing = crate::SymbolId(self.next_intrinsic_backing);
            self.next_intrinsic_backing = self
                .next_intrinsic_backing
                .checked_sub(1)
                .expect("compiler-internal declaration id exhausted");
            self.semantic_world.register_intrinsic_type_operation(
                spec.target_type,
                CONSTRUCT_OR_CONVERT_SELECTOR,
                backing,
                crate::semantic_world::OrdinaryIntrinsicBody::AbstractLiteralConstruct(spec),
                view.clone(),
                view,
                provenance,
            )?;
        }
        Ok(())
    }

    /// Resolve the annotation's exact complete tau snapshot.  The target
    /// carrier is used only to choose the owned Core observation; construction
    /// candidates are enumerated from the immutable callspace on this value.
    fn resolve_complete_annotation_type(
        &mut self,
        source_order_path: &str,
    ) -> Result<Option<crate::CompleteTypeValue>, BuildError> {
        let components = source_order_path
            .split("::")
            .filter(|component| !component.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        let identity = match self.semantic_world.resolve_symbol_path(
            &components,
            self.package_root_node,
            &[self.semantic_world.namespace_index().root_node()],
            &[self.core_node],
        ) {
            Ok(identity) => identity,
            Err(_) => return Ok(None),
        };
        let Some(member) = self
            .semantic_world
            .symbol(identity)
            .and_then(|symbol| symbol.pure_p)
        else {
            return Ok(None);
        };
        let Some(target_type) = self.semantic_world.type_for_pattern(member.pattern) else {
            return Ok(None);
        };
        self.semantic_world
            .observe_complete_type(target_type, Some(member.place))
            .map(Some)
            .map_err(BuildError::single)
    }

    /// Resolve a normalized source call through the semantic Symbol/value/type
    /// associated-`()` spine and invoke the unique ordinary candidate.
    ///
    /// This is the source-facing integration entry. Name resolution produces
    /// one Symbol, then call projection observes its exact complete type.
    pub fn invoke_ordinary_call(
        &mut self,
        namespace: NamespaceNodeId,
        call_site: &crate::NormalizedCallSite,
        context: crate::OrdinaryInvocationContext<'_>,
        provenance: Provenance,
    ) -> Result<crate::InvocationOutcome, crate::OrdinaryInvocationFailure> {
        self.sync_lifecycle_values();
        let Some(candidate) = self.resolve_semantic_source_target(namespace, &call_site.target)
        else {
            return Err(crate::OrdinaryInvocationFailure::NoTargetValues {
                trace: crate::OrdinaryPipelineTrace::default(),
            });
        };
        let resolver_context = ResolverContext::with_mounts(
            namespace,
            vec![self.semantic_world.namespace_index().root_node()],
            vec![self.core_node],
        );
        let caller_package = self
            .semantic_world
            .namespace_owner(namespace)
            .map(|owner| self.semantic_world.owners().package_of(owner));
        let symbol = candidate.symbol;
        let mut attempt_context = context;
        let target_package = self.semantic_world.symbol(symbol).map(|symbol| {
            self.semantic_world
                .owners()
                .package_of(symbol.declaration_owner)
        });
        if caller_package.is_some() && target_package.is_some() && caller_package != target_package
        {
            attempt_context.visibility = crate::VisibilityView::External;
        }
        crate::invoke_host_member_symbol_ordinary(
            &mut self.semantic_world,
            &candidate.host_chain,
            symbol,
            call_site,
            &resolver_context,
            attempt_context,
            provenance,
        )
    }

    /// Resolve source navigation exactly once, before any value/type/call
    /// projection.  A bare name is fixed by lexical shadowing alone; failure
    /// of a later projection, A-stage check, Policy comparison, legality
    /// check, or body execution can never resume the outward scope walk.
    fn resolve_semantic_source_target(
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

    /// Read-only terminal-Symbol observation of the same source resolver used
    /// by value, type, and call consumers.  This exposes identity for
    /// coherence tests and diagnostics; it performs no contextual projection.
    pub fn resolve_source_terminal_symbol(
        &self,
        namespace: NamespaceNodeId,
        target: &NormExpr,
    ) -> Option<crate::SemanticSymbolIdentity> {
        self.resolve_semantic_source_target(namespace, target)
            .map(|resolved| resolved.symbol)
    }

    /// Feed discovered physical source units into namespace assembly and
    /// declaration harvesting.
    ///
    /// Only directories containing discovered `.lang` source units contribute
    /// physical namespace nodes. Empty directories are ignored by source
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
                &self.semantic_world,
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
            // its graph rendering is installed afterward within the
            // same staged CompilationWorld transaction.
            self.semantic_world
                .install_namespace_delta(SemanticNamespaceDelta {
                    namespace,
                    entries: vec![SemanticDeclarationEntry::AssociatedCallEntry {
                        pattern,
                        backing_declaration: callable.symbol_id,
                        closure: closure.clone(),
                        outer_p1_explicit: callable.outer_p1_explicit.clone(),
                        callable_view: callable.function_view,
                        body_entry_view: callable.body_entry_view,
                        namespace_visibility: callable.namespace_visibility,
                        candidate_role: crate::OrdinaryCandidateRole::Ordinary,
                        declared_result_class: callable.declared_result_class,
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
                    "source contribution error: unsupported top-level declaration binder",
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

                // A declaration contributes as a cluster sibling exactly when
                // its binder names an existing cluster Symbol (a Symbol whose
                // `pure_p` is set) in the same namespace. For example,
                // `const let uint8 = (self, ...) => {...}` contributes to the
                // `uint8` cluster, while `let identity = ...` is installed as
                // an ordinary Val2 callable under its own name.
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
                // its graph rendering is installed afterward within
                // the same staged CompilationWorld transaction.
                let entry = if let Some(cluster_symbol) = cluster_symbol {
                    SemanticDeclarationEntry::ClusterContribution {
                        cluster_symbol,
                        backing_declaration: callable.symbol_id,
                        closure: closure.clone(),
                        outer_p1_explicit: callable.outer_p1_explicit.clone(),
                        function_view: callable.function_view,
                        body_entry_view: callable.body_entry_view,
                        namespace_visibility: callable.namespace_visibility,
                        declared_result_class: callable.declared_result_class,
                        provenance: declaration_provenance,
                    }
                } else {
                    SemanticDeclarationEntry::SourceCallable {
                        name: binder_name.clone(),
                        backing_declaration: callable.symbol_id,
                        closure: closure.clone(),
                        outer_p1_explicit: callable.outer_p1_explicit.clone(),
                        function_view: callable.function_view,
                        body_entry_view: callable.body_entry_view,
                        namespace_visibility: callable.namespace_visibility,
                        declared_result_class: callable.declared_result_class,
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
        // One complete binding result demand is formed before the RHS is
        // evaluated. The root producer consumes this same demand before
        // C2/A/Bp/maxima; binding projection/transfer may inspect it only
        // after that producer has been sealed.
        let result_policy_demand = binding_result_policy_demand(slot, &namespace_declaration);
        let mut residual_binding_view = None;

        if let Some(initializer) = slot.initializer.as_deref() {
            match self.evaluate_initializer_best_effort_connected(
                namespace,
                initializer,
                EvalMode::MetaPartial,
                result_policy_demand.clone(),
                declaration_provenance.clone(),
            ) {
                ConnectedInitializerOutcome::Ordinary(result) => {
                    return self.bind_connected_ordinary_result(
                        namespace,
                        &binder_name,
                        slot,
                        &namespace_declaration,
                        &result_policy_demand,
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
                        &result_policy_demand,
                        result,
                        declaration_provenance,
                    );
                }
                ConnectedInitializerOutcome::Residual { reason, provenance } => {
                    verify_residual_policy_compatible(
                        &result_policy_demand,
                        &reason,
                        provenance.clone(),
                    )?;
                    residual_binding_view = residual_policy_view(&result_policy_demand);
                    if is_type_annotation(slot.annotation.as_ref()) {
                        return Err(BuildError::single(Diagnostic::hard_error(
                            "UnsupportedDeferredTypeAssertion: `: type` assertion is deferred for a residual initializer, and deferred type assertions are not connected to the initializer evaluator",
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
            let represented_type = self.semantic_world.allocate_type_lookup_index();
            let carrier = declared_type_projection_delta(
                self.semantic_world.namespace_index(),
                namespace,
                &binder_name,
                represented_type,
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
                SymbolKind::Object,
                SourceCategory::DeclaredSymbol,
                Provenance::file("declared source symbol", file),
            )
        };
        {
            let policy_view = residual_binding_view.clone().unwrap_or_else(|| {
                if is_type_annotation(slot.annotation.as_ref()) {
                    declared_policy_view(
                        &[PolicyStage::Meta, PolicyStage::Runtime],
                        namespace_declaration.mode,
                    )
                } else {
                    declared_policy_view(&[PolicyStage::Runtime], namespace_declaration.mode)
                }
            });
            for symbol in delta.symbols.values_mut() {
                if symbol.name == binder_name {
                    symbol.policy_view = Some(policy_view.clone());
                    symbol.visibility_metadata.namespace_visibility =
                        namespace_declaration.visibility;
                    symbol.visibility_metadata.export_root = namespace_declaration.export_root;
                }
            }
        }
        // Install the authoritative semantic carrier before
        // its graph rendering, in one staged world transaction.
        let semantic_entry = if let Some((symbol_id, represented_type, associated_namespace)) =
            declared_type_carrier
        {
            SemanticDeclarationEntry::TypeCarrier {
                name: binder_name.clone(),
                binding: symbol_id,
                represented_type,
                complete_type: None,
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
        demand: &ResultPolicyDemand,
        result: crate::InvocationOutcome,
        provenance: Provenance,
    ) -> Result<(), BuildError> {
        let result = match result {
            crate::InvocationResult::SemanticResult {
                declared_result_class: crate::DeclaredResultClass::OrdinaryValue,
                value: crate::ProjectedInvocationOutcome::SingleMember(result),
            }
            | crate::InvocationResult::SemanticResult {
                declared_result_class: crate::DeclaredResultClass::CompleteType,
                value: crate::ProjectedInvocationOutcome::SingleMember(result),
            } => result,
            crate::InvocationResult::SemanticResult {
                declared_result_class: crate::DeclaredResultClass::ClusterSymbol,
                value: crate::ProjectedInvocationOutcome::ClusterSymbol(meta),
            } => {
                return self.bind_connected_meta_construction_result(
                    namespace,
                    binder_name,
                    slot,
                    namespace_declaration,
                    demand,
                    meta,
                    provenance,
                );
            }
            crate::InvocationResult::SemanticResult {
                declared_result_class: crate::DeclaredResultClass::Unit,
                value: crate::ProjectedInvocationOutcome::Unit(_),
            } => {
                // The invocation layer already reports Unit execution as
                // future work; no binding path exists yet.
                return Err(BuildError::single(Diagnostic::hard_error(
                    "binding a Unit invocation result is future work",
                    Some(provenance),
                )));
            }
            crate::InvocationResult::Residual(residual) => {
                return Err(BuildError::single(Diagnostic::hard_error(
                    format!(
                        "ordinary invocation residual `{}` cannot be bound here",
                        residual.class
                    ),
                    Some(residual.provenance),
                )));
            }
            crate::InvocationResult::Diagnostic(diagnostic) => {
                return Err(BuildError::single(diagnostic));
            }
            crate::InvocationResult::SemanticResult { .. } => {
                return Err(BuildError::single(Diagnostic::hard_error(
                    "declared invocation result class does not match its projection payload",
                    Some(provenance),
                )));
            }
        };
        // The ordinary binding path consumes the
        // exposure layer, never the raw complete result:
        //
        //   CompleteResultView(P2) -> expose under callable P1
        //                            -> outer binding P1
        //
        // The callable's canonical P1 is a real window here: material
        // outside it is invisible to the binder, and a fully invisible
        // result is a hard error before any outer projection runs.
        let complete_type_authority = result.complete_type.clone();
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
            slot.annotation.as_ref(),
            &exposed.material,
            complete_type_authority.as_ref(),
            provenance.clone(),
        )?;

        let explicit_p1 = slot
            .policy
            .as_ref()
            .map(|_| &namespace_declaration.projection);
        let exposed_material = exposed.material.clone();
        let pair_selected = match crate::elaborate_value_binding_p1(
            &exposed_material,
            explicit_p1,
            provenance.clone(),
        ) {
            Ok(elaboration) => elaboration.selected,
            Err(failure) => {
                return Err(BuildError::single(
                    Diagnostic::hard_error(
                        format!(
                            "ExplicitPolicyProjectionFailed: ordinary result cannot satisfy binding P1 ({failure:?})"
                        ),
                        Some(provenance),
                    )
                    .with_code(ResolverCode::ExplicitPolicyVerificationFailed),
                ));
            }
        };
        let selected = pair_selected
            .into_iter()
            .filter(|entry| entry.view.mode == demand.mode)
            .collect::<Vec<_>>();
        if !selected.is_empty() {
            match result.returned {
                crate::ReturnedSemanticEntity::OrdinaryValue(_) => self
                    .install_connected_semantic_binding(
                        namespace,
                        binder_name,
                        namespace_declaration,
                        &selected,
                        None,
                        provenance,
                    )
                    .map(|_| ()),
                crate::ReturnedSemanticEntity::CompleteType(value) => {
                    let complete_type = complete_type_authority.as_ref().ok_or_else(|| {
                        BuildError::single(Diagnostic::hard_error(
                            "CompleteType binding lost its exact semantic tau",
                            Some(provenance.clone()),
                        ))
                    })?;
                    if let Some(material) = value.construction_material {
                        self.bind_connected_meta_material_result(
                            namespace,
                            binder_name,
                            namespace_declaration,
                            &selected,
                            crate::MetaExecutionMaterial::StructConstructionMaterial(material),
                            Some(complete_type),
                            provenance,
                        )
                    } else {
                        self.install_connected_semantic_binding(
                            namespace,
                            binder_name,
                            namespace_declaration,
                            &selected,
                            Some(complete_type),
                            provenance,
                        )
                        .map(|_| ())
                    }
                }
            }
        } else {
            let demanded_views = self
                .invoke_general_binding_migration(&exposed_material, demand, &provenance)
                .map_err(|failure| {
                    BuildError::single(
                        Diagnostic::hard_error(
                            format!(
                                "ExplicitPolicyProjectionFailed: ordinary result cannot satisfy complete result demand ({failure})"
                            ),
                            Some(provenance.clone()),
                        )
                        .with_code(ResolverCode::ExplicitPolicyVerificationFailed),
                    )
                })?;
            self.install_connected_semantic_binding(
                namespace,
                binder_name,
                namespace_declaration,
                &demanded_views,
                complete_type_authority.as_ref(),
                provenance,
            )
            .map(|_| ())
        }
    }

    /// Installation of replayable meta construction material.
    ///
    /// The selected ordinary result (including a complete tau value) is the
    /// semantic authority.  This helper only expands graph/projection material
    /// required by the current namespace renderer.
    fn bind_connected_meta_material_result(
        &mut self,
        namespace: NamespaceNodeId,
        binder_name: &str,
        namespace_declaration: &NamespaceDeclarationPolicy,
        selected: &[crate::PolicyResultEntry<crate::SemanticValueRef, crate::PatternValueId>],
        value: crate::MetaExecutionMaterial,
        semantic_complete_type: Option<&crate::CompleteTypeValue>,
        provenance: Provenance,
    ) -> Result<(), BuildError> {
        let (material, complete_type) = match (value, semantic_complete_type) {
            (
                crate::MetaExecutionMaterial::StructConstructionMaterial(material),
                Some(complete_type),
            ) => (material, complete_type),
            (material, _) => {
                return Err(BuildError::single(Diagnostic::hard_error(
                    format!(
                        "meta execution material has no canonical binding projection: {material:?}"
                    ),
                    Some(provenance),
                )));
            }
        };
        let canonical_type = material.canonical_type;
        let result_view = uniform_result_policy_view(selected);
        let mut expansion = expand_struct_construction_material(
            material,
            complete_type,
            self.semantic_world.namespace_index(),
            namespace,
            binder_name,
            provenance.clone(),
        )?;
        override_delta_binding_policy_view(
            &mut expansion.namespace_delta,
            binder_name,
            result_view.clone(),
        );
        override_delta_binding_visibility(
            &mut expansion.namespace_delta,
            binder_name,
            namespace_declaration,
        );
        expansion.replacement_object.policy_view = result_view;
        expansion
            .replacement_object
            .visibility_metadata
            .namespace_visibility = namespace_declaration.visibility;
        expansion.replacement_object.visibility_metadata.export_root =
            namespace_declaration.export_root;
        self.semantic_world
            .bind_ordinary_new(namespace, binder_name, selected, provenance.clone())
            .map_err(|conflict| bind_conflict_error(conflict, binder_name, &provenance))?;
        // Semantic type and projection Symbols are installed before their
        // graph rendering.
        if let Some(entry) = selected.first() {
            let pair = declared_pair_from_result_entry(entry, namespace_declaration);
            let associated_namespace = match &expansion.replacement_object.payload {
                SymbolPayload::CompleteTypeProjection(projection) => {
                    projection.type_associated_namespace
                }
                _ => None,
            };
            self.register_installed_type_carrier(
                namespace,
                &expansion.replacement_object.name,
                expansion.replacement_object.id,
                complete_type.lookup_key(),
                Some(complete_type.whole()),
                associated_namespace,
                pair,
                expansion.replacement_object.provenance.clone(),
            )?;
        }
        self.semantic_world
            .register_generated_projection_symbols(&expansion.namespace_delta)?;
        self.semantic_world
            .install_namespace_name_delta(expansion.namespace_delta)?;
        self.diagnostics.extend(expansion.diagnostics);
        if let Some(canonical_type) = canonical_type {
            self.semantic_world.record_ambient_type_binder(
                canonical_type,
                crate::AmbientTypeBinder::WholeSymbol(binder_name.to_string()),
            );
        }
        Ok(())
    }

    /// Project a unified invocation success carrying ClusterSymbol material
    /// into the ordinary let-binding path. The finalized cluster construction's
    /// member views are the canonical result facts; they flow through the same
    /// annotation check, P1 elaboration, and installation as any other result.
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
        demand: &ResultPolicyDemand,
        meta: crate::ClusterSymbolResult,
        provenance: Provenance,
    ) -> Result<(), BuildError> {
        let construction = meta.construction;
        let struct_materials = meta.struct_materials;
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
                pattern: entry.pattern,
                view: entry.view.clone(),
            })
            .collect::<Vec<_>>();
        let semantic_complete_type = if struct_materials.len() == 1 {
            struct_materials
                .first()
                .and_then(|material| material.canonical_type)
                .and_then(|lookup| {
                    let pattern = self.semantic_world.type_value(lookup)?.pattern;
                    let place = self.semantic_world.pattern_place(pattern);
                    self.semantic_world
                        .observe_complete_type(lookup, place)
                        .ok()
                })
        } else {
            None
        };
        assert_semantic_result_satisfies_annotation(
            slot.annotation.as_ref(),
            &result,
            semantic_complete_type.as_ref(),
            provenance.clone(),
        )?;

        let selected = self
            .satisfy_binding_result_demand(&result, demand, &provenance)
            .map_err(|failure| {
                BuildError::single(
                    Diagnostic::hard_error(
                        format!(
                            "ExplicitPolicyProjectionFailed: meta construction result cannot satisfy complete result demand ({failure})"
                        ),
                        Some(provenance.clone()),
                    )
                    .with_code(ResolverCode::ExplicitPolicyVerificationFailed),
                )
            })?;
        // A construction whose sole member is backed by struct material
        // expands the field-function and ref/share projection namespaces.
        // Everything else installs the plain semantic binding carrier.
        let destination =
            if struct_materials.len() == 1 && selected.iter().all(|entry| entry.value.is_none()) {
                let struct_material = struct_materials
                    .into_iter()
                    .next()
                    .expect("struct_materials holds exactly one entry");
                // Diagnostic-only binder record: an ambient struct collision
                // at this level later points at this source-visible binding.
                // The binder never feeds type identity.
                let canonical_type = struct_material.canonical_type;
                let destination = self.install_connected_struct_result_binding(
                    namespace,
                    binder_name,
                    namespace_declaration,
                    &selected,
                    struct_material,
                    semantic_complete_type.as_ref().ok_or_else(|| {
                        BuildError::single(Diagnostic::hard_error(
                            "struct result material lost its exact complete tau",
                            Some(provenance.clone()),
                        ))
                    })?,
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
                    None,
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
        demand: &ResultPolicyDemand,
        result: ConnectedExistingResult,
        provenance: Provenance,
    ) -> Result<(), BuildError> {
        let complete_type = result.complete_type;
        let result = self.construct_abstract_literals_for_annotation(
            slot.annotation.as_ref(),
            result.material,
            demand,
            &provenance,
        )?;
        assert_semantic_result_satisfies_annotation(
            slot.annotation.as_ref(),
            &result,
            complete_type.as_ref(),
            provenance.clone(),
        )?;

        let selected = self
            .satisfy_binding_result_demand(&result, demand, &provenance)
            .map_err(|failure| {
                BuildError::single(
                    Diagnostic::hard_error(
                        format!(
                            "ExplicitPolicyProjectionFailed: existing semantic value cannot satisfy complete result demand ({failure})"
                        ),
                        Some(provenance.clone()),
                    )
                    .with_code(ResolverCode::ExplicitPolicyVerificationFailed),
                )
            })?;
        self.install_connected_semantic_binding(
            namespace,
            binder_name,
            namespace_declaration,
            &selected,
            complete_type.as_ref(),
            provenance,
        )
        .map(|_| ())
    }

    /// Select and execute one abstract-to-concrete constructor from the exact
    /// target tau callspace.  The target is slot 0 and the abstract source is
    /// the sole explicit argument (slot 1).  Selection is sealed before the
    /// builtin/custom body runs, so realization failure cannot retry.
    fn invoke_literal_construction_request(
        &mut self,
        request: crate::ConstructionRequest,
        provenance: &Provenance,
    ) -> Result<(crate::SemanticValueRef, crate::PatternValueId, PolicyView), BuildError> {
        if request.family != crate::ConstructionFamily::ConstructOrConvert {
            return Err(BuildError::single(Diagnostic::hard_error(
                "unsupported literal construction candidate family",
                Some(provenance.clone()),
            )));
        }
        let source_object = self
            .semantic_world
            .value(request.source.id)
            .cloned()
            .ok_or_else(|| {
                BuildError::single(Diagnostic::hard_error(
                    "literal construction source value is not installed",
                    Some(provenance.clone()),
                ))
            })?;
        let crate::SemanticValuePayload::AbstractLiteral { .. } = source_object.payload else {
            return Err(BuildError::single(Diagnostic::hard_error(
                "literal construction source is not an abstract semantic literal",
                Some(provenance.clone()),
            )));
        };
        let target_receiver = self
            .semantic_world
            .core_type_projection_value(request.target.lookup_key())
            .ok_or_else(|| {
                BuildError::single(Diagnostic::hard_error(
                    "literal construction target tau has no semantic receiver value",
                    Some(provenance.clone()),
                ))
            })?;
        let explicit_atom = crate::ProductAtom::SemanticValue {
            value: request.source.id,
            type_value: request.source.type_value,
            mode: source_object.mode,
            provenance: provenance.clone(),
        };
        let explicit_product =
            crate::ArgProductShape::from_flattened(crate::FlattenedProductObject {
                atoms: vec![explicit_atom],
                provenance: provenance.clone(),
                invariant: crate::FlattenedProductInvariant {
                    no_direct_product_atom_remains: true,
                },
            });
        let candidate_values = request
            .target
            .call_space()
            .get(CONSTRUCT_OR_CONVERT_SELECTOR)
            .into_iter()
            .flatten()
            .filter(|entry| entry.facet == crate::TypeMemberFacet::Value)
            .map(|entry| entry.value)
            .collect::<Vec<_>>();
        let target_members = self
            .semantic_world
            .member_views_for_values(&candidate_values);
        let target_pattern = self
            .semantic_world
            .type_value(request.target.lookup_key())
            .expect("complete target lookup key remains installed")
            .pattern;

        // Abstract-to-concrete construction itself produces a compile view.
        // A surrounding runtime demand is satisfied only afterwards by the
        // ordinary same-Type migration boundary. Whole-slot mode remains the
        // coordinate supplied by the original result demand.
        let construction_demand = ResultPolicyDemand {
            pair_query: P1Projection::Pair(crate::compile_literal_policy()),
            mode: request.result_demand.mode,
        };
        let explicit_modes = [source_object.mode];
        let context = crate::OrdinaryInvocationContext::open_static(&explicit_modes)
            .with_result_policy_demand(construction_demand)
            .with_construction_target(&request.target);
        let resolver_context = self.root_context();
        let outcome = crate::ordinary_invocation::invoke_target_values(
            &mut self.semantic_world,
            crate::OrdinaryCandidateOrigin::PatternAssociatedCallEntry(target_pattern),
            target_members,
            std::collections::BTreeMap::new(),
            Some(target_receiver),
            None,
            explicit_product,
            &resolver_context,
            context,
            provenance.clone(),
        )
        .map_err(|failure| {
            BuildError::single(ordinary_invocation_failure_diagnostic(
                failure,
                provenance.clone(),
            ))
        })?;
        let crate::InvocationResult::SemanticResult {
            declared_result_class: crate::DeclaredResultClass::OrdinaryValue,
            value: crate::ProjectedInvocationOutcome::SingleMember(selected),
        } = outcome
        else {
            return Err(BuildError::single(Diagnostic::hard_error(
                "literal construction selected a non-OrdinaryValue result class",
                Some(provenance.clone()),
            )));
        };
        let exposed = selected.exposed();
        let [entry] = exposed.material.as_slice() else {
            return Err(BuildError::single(Diagnostic::hard_error(
                "literal construction did not expose exactly one concrete value",
                Some(provenance.clone()),
            )));
        };
        let Some(result_ref) = entry.value else {
            return Err(BuildError::single(Diagnostic::hard_error(
                "literal construction returned a pure Pattern instead of a value",
                Some(provenance.clone()),
            )));
        };
        let result = self
            .semantic_world
            .value(result_ref.id)
            .expect("ordinary constructor installed its returned value");
        let exact_target = matches!(
            &result.payload,
            crate::SemanticValuePayload::ConstructedLiteral {
                target_complete_type,
                ..
            } if *target_complete_type == request.target.whole()
        );
        if result_ref.type_value != request.target.lookup_key() || !exact_target {
            return Err(BuildError::single(Diagnostic::hard_error(
                "selected literal constructor returned a value outside the demanded complete Type",
                Some(provenance.clone()),
            )));
        }
        Ok((result_ref, entry.pattern, entry.view.clone()))
    }

    /// Apply an explicit concrete annotation only after abstract literal
    /// formation.  The installed abstract source remains intact and the
    /// result records it as construction provenance, so an expected machine
    /// Type can never retroactively rewrite the literal's initial Type.
    fn construct_abstract_literals_for_annotation(
        &mut self,
        annotation: Option<&NormAnnotation>,
        mut result: Vec<crate::PolicyResultEntry<crate::SemanticValueRef, crate::PatternValueId>>,
        result_demand: &ResultPolicyDemand,
        provenance: &Provenance,
    ) -> Result<
        Vec<crate::PolicyResultEntry<crate::SemanticValueRef, crate::PatternValueId>>,
        BuildError,
    > {
        let Some(NormPattern::Name { name, .. }) = annotation.map(|annotation| &annotation.pattern)
        else {
            return Ok(result);
        };
        if matches!(name.as_str(), "type" | "integer" | "real" | "character") {
            return Ok(result);
        }
        let Some(target) = self.resolve_complete_annotation_type(name)? else {
            return Ok(result);
        };
        for entry in &mut result {
            let Some(source) = entry.value else {
                continue;
            };
            if !matches!(
                self.semantic_world
                    .value(source.id)
                    .map(|value| &value.payload),
                Some(crate::SemanticValuePayload::AbstractLiteral { .. })
            ) {
                continue;
            }
            let (constructed, pattern, view) = self.invoke_literal_construction_request(
                crate::ConstructionRequest {
                    source,
                    target: target.clone(),
                    result_demand: result_demand.clone(),
                    family: crate::ConstructionFamily::ConstructOrConvert,
                },
                provenance,
            )?;
            entry.value = Some(constructed);
            entry.pattern = pattern;
            entry.view = view;
        }
        Ok(result)
    }

    /// Direct general same-Type satisfaction used after an ordinary existing
    /// view projection has failed.  Each source produces at most one
    /// migration request; results are never fed back into candidate lookup.
    fn invoke_general_binding_migration(
        &mut self,
        result: &[crate::PolicyResultEntry<crate::SemanticValueRef, crate::PatternValueId>],
        demand: &ResultPolicyDemand,
        provenance: &Provenance,
    ) -> Result<Vec<crate::PolicyResultEntry<crate::SemanticValueRef, crate::PatternValueId>>, String>
    {
        let mut completed = Vec::new();
        for entry in result {
            let Some(source) = entry.value else {
                return Err("a pure-P entry has no value realization to migrate".into());
            };
            let source_view = entry.view.clone();
            let target_view = policy_let_target_demand(demand, &source_view.pair);
            let target_demand = ResultPolicyDemand {
                pair_query: P1Projection::Pair(target_view.pair),
                mode: target_view.mode,
            };
            let request = crate::PolicyMigrationRequest::new(
                source_view,
                target_demand,
                source.type_value,
                source.id,
                provenance.clone(),
            )
            .map_err(|failure| format!("request formation: {failure:?}"))?;
            let migration = self
                .invoke_policy_migration(&request)
                .map_err(|failure| format!("selection/execution: {failure:?}"))?;
            // The selected migration body already produced the coherent
            // ValueRealization carried by demanded_view.  Callable identity
            // remains in the invocation trace; it must not be reified as a
            // second ordinary Val1 wrapper.
            completed.extend(migration.demanded_view);
        }
        if completed.is_empty() {
            Err("migration produced no completed view".into())
        } else {
            Ok(completed)
        }
    }

    /// Satisfy a binding result demand existing-view-first. Identity is legal
    /// only when both the pair projection and the explicit whole-slot mode
    /// match; otherwise exactly one direct same-Type migration is selected.
    fn satisfy_binding_result_demand(
        &mut self,
        result: &[crate::PolicyResultEntry<crate::SemanticValueRef, crate::PatternValueId>],
        demand: &ResultPolicyDemand,
        provenance: &Provenance,
    ) -> Result<Vec<crate::PolicyResultEntry<crate::SemanticValueRef, crate::PatternValueId>>, String>
    {
        // Binding P1 has one deliberate pure-P rule: a value-presence
        // coordinate cannot require migration when the semantic result has
        // no Val1, but its requested stage slice still applies.  Reuse that
        // binding elaboration here; all non-identity completion below goes
        // through the one general same-Type migration entry.
        let explicit_pair = (!matches!(demand.pair_query, crate::P1Projection::Infer))
            .then_some(&demand.pair_query);
        let pair_projected =
            match crate::elaborate_value_binding_p1(result, explicit_pair, provenance.clone()) {
                Ok(elaboration) => elaboration.selected,
                Err(_) => Vec::new(),
            };
        let projected = pair_projected
            .into_iter()
            .filter(|entry| entry.view.mode == demand.mode)
            .collect::<Vec<_>>();
        if !projected.is_empty() {
            return Ok(projected);
        }
        self.invoke_general_binding_migration(result, demand, provenance)
    }

    /// Installs the selected connected result views under a fresh
    /// destination Symbol and returns that Symbol's identity.
    fn install_connected_semantic_binding(
        &mut self,
        namespace: NamespaceNodeId,
        binder_name: &str,
        namespace_declaration: &NamespaceDeclarationPolicy,
        selected: &[crate::PolicyResultEntry<crate::SemanticValueRef, crate::PatternValueId>],
        semantic_complete_type: Option<&crate::CompleteTypeValue>,
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
        let policy_view = uniform_result_policy_view(selected);
        let mut declared_type_carrier = None;
        if selected.iter().all(|entry| entry.value.is_none()) {
            let pattern = first_pattern;
            let represented_type = match semantic_complete_type {
                Some(complete) => complete.lookup_key(),
                None => self
                    .semantic_world
                    .type_for_pattern(pattern)
                    .ok_or_else(|| {
                        BuildError::single(Diagnostic::hard_error(
                            "ordinary semantic binding: pure-P result uses an unregistered PatternValue",
                            Some(provenance.clone()),
                        ))
                    })?,
            };
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
            override_delta_binding_policy_view(&mut delta, binder_name, policy_view.clone());
            override_delta_binding_visibility(&mut delta, binder_name, namespace_declaration);
            let pure_selected: Vec<_> = selected
                .iter()
                .map(|entry| crate::PolicyResultEntry {
                    value: None,
                    pattern: entry.pattern,
                    view: entry.view.clone(),
                })
                .collect();
            let destination = self
                .semantic_world
                .bind_ordinary_new(namespace, binder_name, &pure_selected, provenance.clone())
                .map_err(|conflict| bind_conflict_error(conflict, binder_name, &provenance))?;
            // Install the authoritative semantic carrier
            // before its graph rendering.
            if let Some((symbol_id, represented_type, associated_namespace)) = declared_type_carrier
            {
                self.register_installed_type_carrier(
                    namespace,
                    binder_name,
                    symbol_id,
                    represented_type,
                    semantic_complete_type.map(|complete| complete.whole()),
                    Some(associated_namespace),
                    declared_pair_from_result_entry(&selected[0], namespace_declaration),
                    provenance,
                )?;
            }
            self.semantic_world.install_namespace_name_delta(delta)?;
            return Ok(destination);
        }

        // A complete type result reaches this boundary as an already-observed
        // semantic entity.  The CoreTypeProjection carried by a projected value
        // is deliberately not inspected here: graph projection is a
        // one-way `tau -> CoreTypeProjection` rendering and can never recover or
        // decide type identity.
        let mut delta = if let Some(complete_type) = semantic_complete_type {
            let represented_type = complete_type.lookup_key();
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
                SymbolKind::Object,
                SourceCategory::DeclaredSymbol,
                provenance.clone(),
            )
        };
        override_delta_binding_policy_view(&mut delta, binder_name, policy_view);
        override_delta_binding_visibility(&mut delta, binder_name, namespace_declaration);
        let destination = self
            .semantic_world
            .bind_ordinary_new(namespace, binder_name, selected, provenance.clone())
            .map_err(|conflict| bind_conflict_error(conflict, binder_name, &provenance))?;
        // Install the authoritative semantic carrier before
        // its graph rendering.
        if let Some((symbol_id, represented_type, associated_namespace)) = declared_type_carrier {
            self.register_installed_type_carrier(
                namespace,
                binder_name,
                symbol_id,
                represented_type,
                semantic_complete_type.map(|complete| complete.whole()),
                Some(associated_namespace),
                declared_pair_from_result_entry(&selected[0], namespace_declaration),
                provenance,
            )?;
        }
        self.semantic_world.install_namespace_name_delta(delta)?;
        Ok(destination)
    }

    /// Installs a connected meta construction result whose unique complete
    /// type member is backed by struct construction material. The namespace
    /// side forms the full projection (CoreTypeProjection with fields,
    /// field-function projection layer, ref/share projection namespaces),
    /// while the semantic side binds the construction's member views under
    /// a fresh destination Symbol — the same canonical facts as the plain
    /// carrier path, plus the namespace projection the plain carrier lacks.
    fn install_connected_struct_result_binding(
        &mut self,
        namespace: NamespaceNodeId,
        binder_name: &str,
        namespace_declaration: &NamespaceDeclarationPolicy,
        selected: &[crate::PolicyResultEntry<crate::SemanticValueRef, crate::PatternValueId>],
        struct_material: crate::StructConstructionMaterial,
        semantic_complete_type: &crate::CompleteTypeValue,
        provenance: Provenance,
    ) -> Result<crate::SemanticSymbolIdentity, BuildError> {
        let result_view = uniform_result_policy_view(selected);
        let mut expansion = expand_struct_construction_material(
            struct_material,
            semantic_complete_type,
            self.semantic_world.namespace_index(),
            namespace,
            binder_name,
            provenance.clone(),
        )?;
        override_delta_binding_policy_view(
            &mut expansion.namespace_delta,
            binder_name,
            result_view.clone(),
        );
        override_delta_binding_visibility(
            &mut expansion.namespace_delta,
            binder_name,
            namespace_declaration,
        );
        expansion.replacement_object.policy_view = result_view;
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
        // installed before their graph rendering.
        if let Some(entry) = selected.first() {
            let associated_namespace = match &expansion.replacement_object.payload {
                SymbolPayload::CompleteTypeProjection(projection) => {
                    projection.type_associated_namespace
                }
                _ => None,
            };
            self.register_installed_type_carrier(
                namespace,
                &expansion.replacement_object.name,
                expansion.replacement_object.id,
                semantic_complete_type.lookup_key(),
                Some(semantic_complete_type.whole()),
                associated_namespace,
                declared_pair_from_result_entry(entry, namespace_declaration),
                expansion.replacement_object.provenance.clone(),
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
        result_policy_demand: ResultPolicyDemand,
        provenance: Provenance,
    ) -> ConnectedInitializerOutcome {
        if let NormExpr::PolicyLet {
            policy,
            operand,
            origin,
        } = initializer
        {
            return self.evaluate_policy_let_initializer(
                namespace,
                policy,
                operand,
                mode,
                Provenance::from_norm_origin("PolicyLet result boundary", origin),
            );
        }
        if matches!(initializer, NormExpr::Literal { .. }) {
            let abstract_literal = crate::form_abstract_literal_value(
                initializer,
                |family| self.resolve_type_value(family.type_name()).ok(),
                crate::SemanticValueId(0),
                provenance.clone(),
            );
            return match abstract_literal {
                Ok(literal) => {
                    let policy = literal.policy.clone();
                    let type_value = literal.type_value;
                    let Some(value) = self.semantic_world.install_abstract_literal_value(
                        literal.family,
                        literal.exact,
                        type_value,
                        policy.clone(),
                        provenance.clone(),
                    ) else {
                        return ConnectedInitializerOutcome::Diagnostic(Diagnostic::hard_error(
                            "abstract literal Type is not installed in the semantic world",
                            Some(provenance),
                        ));
                    };
                    let pattern = self
                        .semantic_world
                        .type_value(type_value)
                        .expect("abstract literal Type was resolved")
                        .pattern;
                    ConnectedInitializerOutcome::Existing(ConnectedExistingResult {
                        material: vec![crate::PolicyResultEntry {
                            value: Some(crate::SemanticValueRef {
                                id: value,
                                type_value,
                            }),
                            pattern,
                            view: PolicyView {
                                pair: policy,
                                mode: PolicyMode::Plain,
                            },
                        }],
                        complete_type: None,
                    })
                }
                Err(crate::AbstractLiteralFormationFailure::CharacterSpellingOpen) => {
                    ConnectedInitializerOutcome::Residual {
                        reason: crate::ResidualReason::UnsupportedExpression,
                        provenance,
                    }
                }
                Err(failure) => ConnectedInitializerOutcome::Diagnostic(Diagnostic::hard_error(
                    format!("abstract literal formation failed: {failure:?}"),
                    Some(provenance),
                )),
            };
        }
        if let Ok(call_site) = crate::extract_single_call_site(initializer) {
            if self
                .resolve_semantic_source_target(namespace, &call_site.target)
                .is_some()
            {
                // Unknown actuals have the primitive Plain view. A concrete
                // argument resolved by the ordinary classifier replaces this
                // default with its own PolicyView.mode; the world never
                // fabricates Const.
                let explicit_modes =
                    vec![crate::PolicyMode::Plain; call_site.source_product.elements.len()];
                // B8: a world-level connected declaration's environment is the
                // namespace level itself (no enclosing callable), so the
                // ambient construction owner is supplied explicitly here.  A
                // future callable-body evaluator must supply the enclosing
                // anonymous function object's Self scope owner instead.
                let mut context = crate::OrdinaryInvocationContext::open_static(&explicit_modes)
                    .with_result_policy_demand(result_policy_demand);
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

    /// Evaluate one explicit PolicyLet result boundary.
    ///
    /// The inward mode is installed before the operand root call forms Bp'
    /// maxima.  The operand is evaluated exactly once, converted to a local
    /// completed view, and then satisfied existing-view-first or by one
    /// direct same-Type migration.  No outer consumer receives an inner call
    /// site or candidate set that it could reopen.
    fn evaluate_policy_let_initializer(
        &mut self,
        namespace: NamespaceNodeId,
        policy: &NormPolicySpec,
        operand: &NormExpr,
        mode: EvalMode,
        provenance: Provenance,
    ) -> ConnectedInitializerOutcome {
        let demand = match elaborate_binding_result_demand(Some(policy), provenance.clone()) {
            Ok(demand) => demand,
            Err(diagnostic) => return ConnectedInitializerOutcome::Diagnostic(diagnostic),
        };

        let inner = if let Ok(call_site) = crate::extract_single_call_site(operand) {
            if self
                .resolve_semantic_source_target(namespace, &call_site.target)
                .is_some()
            {
                let explicit_modes =
                    vec![PolicyMode::Plain; call_site.source_product.elements.len()];
                let mut context = crate::OrdinaryInvocationContext::open_static(&explicit_modes)
                    .with_result_policy_demand(demand.clone());
                context.ambient_construction_owner = self.semantic_world.namespace_owner(namespace);
                match self.invoke_ordinary_call(namespace, &call_site, context, provenance.clone())
                {
                    Ok(result) => ConnectedInitializerOutcome::Ordinary(result),
                    Err(
                        crate::OrdinaryInvocationFailure::NoTargetValues { .. }
                        | crate::OrdinaryInvocationFailure::NoFullyAdmissibleCandidate {
                            first_diagnostic: None,
                            ..
                        },
                    ) if mode == EvalMode::MetaPartial => ConnectedInitializerOutcome::Residual {
                        reason: crate::ResidualReason::NoMetaVisibleCandidate,
                        provenance: provenance.clone(),
                    },
                    Err(failure) => ConnectedInitializerOutcome::Diagnostic(
                        ordinary_invocation_failure_diagnostic(failure, provenance.clone()),
                    ),
                }
            } else {
                ConnectedInitializerOutcome::Residual {
                    reason: crate::ResidualReason::UnsupportedExpression,
                    provenance: provenance.clone(),
                }
            }
        } else if matches!(operand, NormExpr::PolicyLet { .. }) {
            self.evaluate_initializer_best_effort_connected(
                namespace,
                operand,
                mode,
                ResultPolicyDemand::default(),
                provenance.clone(),
            )
        } else if let Some(existing) = self.existing_semantic_result(namespace, operand) {
            ConnectedInitializerOutcome::Existing(existing)
        } else {
            ConnectedInitializerOutcome::Residual {
                reason: crate::ResidualReason::UnsupportedExpression,
                provenance: provenance.clone(),
            }
        };

        let (material, complete_type) = match inner {
            ConnectedInitializerOutcome::Ordinary(crate::InvocationResult::SemanticResult {
                value: crate::ProjectedInvocationOutcome::SingleMember(result),
                ..
            }) => (result.exposed().material, result.complete_type),
            ConnectedInitializerOutcome::Ordinary(crate::InvocationResult::SemanticResult {
                declared_result_class: crate::DeclaredResultClass::ClusterSymbol,
                value: crate::ProjectedInvocationOutcome::ClusterSymbol(result),
            }) => (
                result
                    .construction
                    .member_views
                    .into_iter()
                    .map(|entry| crate::PolicyResultEntry {
                        value: entry.value.map(|id| {
                            let value = self
                                .semantic_world
                                .value(id)
                                .expect("construction result references installed value");
                            crate::SemanticValueRef {
                                id,
                                type_value: value.type_value,
                            }
                        }),
                        pattern: entry.pattern,
                        view: entry.view,
                    })
                    .collect(),
                None,
            ),
            ConnectedInitializerOutcome::Ordinary(crate::InvocationResult::SemanticResult {
                declared_result_class: crate::DeclaredResultClass::Unit,
                value: crate::ProjectedInvocationOutcome::Unit(_),
            }) => {
                return ConnectedInitializerOutcome::Diagnostic(Diagnostic::hard_error(
                    "PolicyLet cannot yet complete a Unit invocation result",
                    Some(provenance),
                ));
            }
            ConnectedInitializerOutcome::Ordinary(crate::InvocationResult::Residual(residual)) => {
                return ConnectedInitializerOutcome::Residual {
                    reason: crate::ResidualReason::UnsupportedExpression,
                    provenance: residual.provenance,
                };
            }
            ConnectedInitializerOutcome::Ordinary(crate::InvocationResult::Diagnostic(
                diagnostic,
            )) => return ConnectedInitializerOutcome::Diagnostic(diagnostic),
            ConnectedInitializerOutcome::Ordinary(crate::InvocationResult::SemanticResult {
                ..
            }) => {
                return ConnectedInitializerOutcome::Diagnostic(Diagnostic::hard_error(
                    "declared invocation result class does not match its projection payload",
                    Some(provenance),
                ));
            }
            ConnectedInitializerOutcome::Existing(existing) => {
                (existing.material, existing.complete_type)
            }
            ConnectedInitializerOutcome::Residual { reason, provenance } => {
                return ConnectedInitializerOutcome::Residual { reason, provenance };
            }
            ConnectedInitializerOutcome::Diagnostic(diagnostic) => {
                return ConnectedInitializerOutcome::Diagnostic(diagnostic);
            }
        };

        let projected = crate::project_p1(&demand.pair_query, &material)
            .into_iter()
            .filter(|entry| entry.view.mode == demand.mode)
            .collect::<Vec<_>>();
        if !projected.is_empty() {
            return ConnectedInitializerOutcome::Existing(ConnectedExistingResult {
                material: projected,
                complete_type,
            });
        }

        let mut migrated = Vec::new();
        for entry in material {
            let source_view = entry.view;
            let Some(source) = entry.value else {
                return ConnectedInitializerOutcome::Diagnostic(Diagnostic::hard_error(
                    "PolicyLet cannot migrate a pure-P result: absent Val1 is outside same-Type Policy migration; an authorized constructor/materializer must produce a value first",
                    Some(provenance),
                ));
            };
            let target_view = policy_let_target_demand(&demand, &source_view.pair);
            let target_demand = ResultPolicyDemand {
                pair_query: P1Projection::Pair(target_view.pair),
                mode: target_view.mode,
            };
            let request = match crate::PolicyMigrationRequest::new(
                source_view,
                target_demand,
                source.type_value,
                source.id,
                provenance.clone(),
            ) {
                Ok(request) => request,
                Err(failure) => {
                    return ConnectedInitializerOutcome::Diagnostic(Diagnostic::hard_error(
                        format!(
                            "PolicyLet cannot form its same-Type migration demand: {failure:?}"
                        ),
                        Some(provenance),
                    ));
                }
            };
            let migration = match self.invoke_policy_migration(&request) {
                Ok(migration) => migration,
                Err(failure) => {
                    return ConnectedInitializerOutcome::Diagnostic(
                        ordinary_invocation_failure_diagnostic(failure, provenance),
                    );
                }
            };
            migrated.extend(migration.demanded_view);
        }
        if migrated.is_empty() {
            ConnectedInitializerOutcome::Diagnostic(Diagnostic::hard_error(
                "PolicyLet migration produced no completed outward view",
                Some(provenance),
            ))
        } else {
            ConnectedInitializerOutcome::Existing(ConnectedExistingResult {
                material: migrated,
                complete_type,
            })
        }
    }

    fn existing_semantic_result(
        &self,
        namespace: NamespaceNodeId,
        initializer: &NormExpr,
    ) -> Option<ConnectedExistingResult> {
        let symbol = self
            .resolve_semantic_source_target(namespace, initializer)?
            .symbol;
        let symbol = self.semantic_world.symbol(symbol)?;
        let complete_type = symbol
            .pure_p
            .and_then(|member| member.complete_type)
            .and_then(|whole| {
                self.semantic_world
                    .complete_type_by_whole_observation(whole)
            })
            .cloned();
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
                pattern: entry.pattern,
                view: entry.view.clone(),
            })
            .collect::<Vec<_>>();
        if entries.is_empty() {
            return None;
        }
        Some(ConnectedExistingResult {
            material: entries,
            complete_type,
        })
    }

    fn harvest_alias(
        &mut self,
        _namespace: NamespaceNodeId,
        decl: &NormDecl,
        _file: &Path,
    ) -> Result<(), BuildError> {
        let NormDecl::Alias { origin, .. } = decl else {
            return Ok(());
        };
        Err(BuildError::single(
            Diagnostic::hard_error(
                "block-local lexical alias resolution is not implemented; `===` is preserved by the frontend and must not install or forward a semantic entity",
                Some(Provenance::from_norm_origin("alias declaration", origin)),
            )
            .with_code(ResolverCode::UnsupportedLexicalAlias),
        ))
    }
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

fn declared_type_projection_delta(
    snapshot: &SemanticNameIndex,
    parent: NamespaceNodeId,
    name: &str,
    represented_type: crate::TypeValueId,
    provenance: Provenance,
) -> DeclaredTypeCarrierDelta {
    // Graph projection for a declared type carrier. `let t: type =
    // uint8` is an ordinary fresh Symbol/Place binding, not type generation or
    // aliasing. Canonical Core/whole equality and Writable judgments live in
    // SemanticWorld; this graph object only renders the already-decided type
    // binding for namespace projection.
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

    let mut symbol = SymbolObject::new(
        type_symbol_id,
        name,
        SymbolKind::CompleteTypeProjection,
        SourceCategory::DeclaredSymbol,
        Some(parent),
        provenance.clone(),
    );
    symbol.node_kind = Some(NamespaceNodeKind::Virtual);
    symbol.payload = SymbolPayload::CompleteTypeProjection(CoreTypeProjection {
        carrier_symbol_id: type_symbol_id,
        represented_type,
        fields: Vec::new(),
        field_names: Vec::new(),
        field_type_values: Vec::new(),
        field_type_symbol_ids: Vec::new(),
        type_associated_namespace: Some(type_namespace_id),
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

/// One declared type-carrier installation: its graph projection plus
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
    let mut carrier =
        declared_type_projection_delta(snapshot, parent, name, represented_type, provenance);
    let symbol = carrier
        .delta
        .symbols
        .values_mut()
        .find(|symbol| symbol.name == name)
        .expect("declared type-value delta contains its carrier");
    let SymbolPayload::CompleteTypeProjection(type_projection) = &mut symbol.payload else {
        unreachable!("declared type-value carrier is a CompleteType projection");
    };
    type_projection.represented_type = represented_type;
    type_projection.generation_origin = Some("ordinary evaluated TypeValue binding".to_string());
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

/// The one complete result demand established by a binding spelling.
///
/// Namespace visibility/export remain declaration coordinates. Only the
/// binding's Policy projection and primitive whole-slot mode enter producer
/// resolution, and this same value crosses the later binding-satisfaction
/// boundary after the producer has been sealed.
fn binding_result_policy_demand(
    slot: &lang_syntax::NormBindingSlot,
    namespace_declaration: &NamespaceDeclarationPolicy,
) -> ResultPolicyDemand {
    if slot.policy.is_some() {
        ResultPolicyDemand {
            pair_query: namespace_declaration.projection.clone(),
            mode: namespace_declaration.mode,
        }
    } else {
        ResultPolicyDemand::default()
    }
}

/// The declared canonical `PolicyPair` carried by one connected result entry.
/// Binding visibility/export remain separate declaration attributes.
fn declared_pair_from_result_entry<V, P>(
    entry: &crate::PolicyResultEntry<V, P>,
    _namespace_declaration: &NamespaceDeclarationPolicy,
) -> PolicyPair {
    entry.view.pair.clone()
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
    function_view: PolicyView,
    body_entry_view: PolicyView,
    namespace_visibility: Option<crate::NamespaceVisibility>,
    /// Declared result class, elaborated once from the return slot and
    /// validated against result P2. The full Pattern remains in the closure.
    /// Registration sites mirror it onto the call entry verbatim.
    declared_result_class: crate::DeclaredResultClass,
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
    ensure_runtime_result_slice_has_value_dimension(closure, &result_p2.pair, provenance.clone())
        .map_err(BuildError::single)?;
    // Elaborate the declared result class once from the return slot. The body
    // is never inspected, Policy does not determine the class, and the full
    // return Pattern remains in the closure.
    let declared_result_class = crate::overload_set::declared_result_class_from_closure(closure)
        .map_err(BuildError::single)?;
    crate::policy_pair::validate_declared_result_class(
        declared_result_class.clone(),
        &result_p2.pair,
        &provenance,
    )
    .map_err(BuildError::single)?;
    let namespace_declaration = elaborate_namespace_declaration_policy(
        policy_expr,
        NamespaceDeclarationPosition::DirectTopLevel,
        provenance.clone(),
    )
    .map_err(BuildError::single)?;
    let declaration_policy = function_object_declaration_policy(&namespace_declaration);
    let derived_function_view = derive_function_object_view(&result_p2, &declaration_policy);
    let preliminary_return_view = elaborate_return_policy_pattern(
        closure
            .head
            .as_ref()
            .and_then(|head| head.returns.as_ref())
            .and_then(|slot| slot.policy.as_ref()),
        &derived_function_view,
        provenance.clone(),
    )
    .map_err(BuildError::single)?
    .effective_view;
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
    // the prefix are an explicit value-stage selection, while
    // `public/private/export` remain namespace declaration attributes and
    // never enter the P1.
    let outer_p1_explicit: Option<ExplicitP1Selection> = crate::policy_pair::elaborate_explicit_p1(
        policy_expr,
        &result_p2.pair,
        crate::policy_pair::ExplicitP1Position::OuterBinding,
        provenance.clone(),
    )
    .map_err(BuildError::single)?;

    let mut delta = snapshot.empty_delta();
    let symbol_id = delta.allocate_symbol_id();
    // Return targets are validated during source harvesting. Bound return
    // events are not stored in SourceCallableObject; execution wiring remains
    // outside this source-harvesting boundary.
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

    let mut symbol = SymbolObject::new(
        symbol_id,
        name,
        SymbolKind::MetaFunction,
        SourceCategory::DeclaredSymbol,
        Some(parent),
        provenance.clone(),
    );
    symbol.policy_view = Some(derived_function_view.clone());
    symbol.visibility_metadata.namespace_visibility = namespace_declaration.visibility;
    symbol.visibility_metadata.export_root = namespace_declaration.export_root;
    symbol.payload = SymbolPayload::MetaFunction(MetaFunctionObject {
        function_symbol_id: symbol_id,
        primitive: None,
        source_callable: Some(SourceCallableObject {
            closure: closure.clone(),
            provenance: provenance.clone(),
        }),
        function_policy: derived_function_view.clone(),
        body_entry_policy: result_p2.clone(),
        return_object_policy: preliminary_return_view,
        declared_result_class: declared_result_class.clone(),
        privilege: crate::CallablePrivilege::OrdinarySource,
    });
    delta.insert_symbol(parent, symbol);
    Ok(SourceCallableDelta {
        delta,
        symbol_id,
        outer_p1_explicit,
        function_view: derived_function_view,
        body_entry_view: result_p2,
        namespace_visibility: namespace_declaration.visibility,
        declared_result_class,
    })
}

fn uniform_result_policy_view<V, P>(
    entries: &[crate::PolicyResultEntry<V, P>],
) -> Option<PolicyView> {
    let first = entries.first()?.view.clone();
    entries
        .iter()
        .all(|entry| entry.view == first)
        .then_some(first)
}

fn ordinary_invocation_failure_diagnostic(
    failure: crate::OrdinaryInvocationFailure,
    provenance: Provenance,
) -> Diagnostic {
    match failure {
        crate::OrdinaryInvocationFailure::SelectedDelete { diagnostic, .. }
        | crate::OrdinaryInvocationFailure::SelectedCoreBody { diagnostic, .. }
        | crate::OrdinaryInvocationFailure::DynamicLegality { diagnostic, .. }
        | crate::OrdinaryInvocationFailure::CyclicVal2 { diagnostic, .. }
        | crate::OrdinaryInvocationFailure::MetaReturnTypeRootMismatch { diagnostic, .. }
        | crate::OrdinaryInvocationFailure::ApplicabilityUnsupported { diagnostic, .. }
        | crate::OrdinaryInvocationFailure::SelectedBody {
            failure: crate::SourceBodyEvaluationFailure { diagnostic, .. },
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
        crate::OrdinaryInvocationFailure::Residual { residual, .. } => Diagnostic::hard_error(
            format!(
                "invocation residual `{}` reached a binding boundary without an owning evaluator",
                residual.class
            ),
            Some(residual.provenance),
        ),
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
) -> Result<crate::PolicyView, Diagnostic> {
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
    annotation: Option<&NormAnnotation>,
    result: &[crate::PolicyResultEntry<crate::SemanticValueRef, crate::PatternValueId>],
    semantic_complete_type: Option<&crate::CompleteTypeValue>,
    provenance: Provenance,
) -> Result<(), BuildError> {
    if !is_type_annotation(annotation) {
        return Ok(());
    }
    if semantic_complete_type.is_none() {
        return Err(BuildError::single(
            Diagnostic::hard_error(
                "AnnotationAssertionFailed: `: type` expects an explicit complete type semantic result",
                Some(provenance),
            )
            .with_code(ResolverCode::AnnotationAssertionFailed),
        ));
    }
    debug_assert!(!result.is_empty());
    Ok(())
}

fn verify_residual_policy_compatible(
    demand: &ResultPolicyDemand,
    reason: &crate::ResidualReason,
    provenance: Provenance,
) -> Result<(), BuildError> {
    if residual_policy_view(demand).is_some() {
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

fn residual_policy_view(demand: &ResultPolicyDemand) -> Option<PolicyView> {
    let runtime = crate::PolicyResultEntry {
        value: Some(()),
        pattern: (),
        view: declared_policy_view(&[PolicyStage::Runtime], demand.mode),
    };
    crate::policy_pair::project_p1(&demand.pair_query, &[runtime])
        .into_iter()
        .next()
        .map(|entry| entry.view)
}

fn projection_matches_expectation(object: &SymbolObject, expectation: ResolveExpectation) -> bool {
    match expectation {
        ResolveExpectation::AnyUnique | ResolveExpectation::Object => {
            object.kind != SymbolKind::Namespace
        }
        ResolveExpectation::NamespaceSubspace => object.kind == SymbolKind::Namespace,
        ResolveExpectation::NamespaceCapableParent => object.namespace_node().is_some(),
        ResolveExpectation::CoreTypeProjection => object.kind == SymbolKind::CompleteTypeProjection,
        ResolveExpectation::MetaFunction => object.kind == SymbolKind::MetaFunction,
        ResolveExpectation::FieldFunction => object.kind == SymbolKind::FieldFunction,
    }
}

/// Mirrors one uniform result view onto declaration-projection records.
/// Heterogeneous Symbol clusters keep their per-member views in the semantic
/// Symbol and deliberately have no fabricated whole-Symbol Policy view.
fn override_delta_binding_policy_view(
    delta: &mut SemanticNameDelta,
    binding_name: &str,
    policy_view: Option<PolicyView>,
) {
    for symbol in delta.symbols.values_mut() {
        if symbol.name == binding_name {
            symbol.policy_view = policy_view.clone();
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

#[cfg(test)]
mod literal_construction_tests {
    use super::*;

    fn world() -> CompilationWorld {
        CompilationWorld::from_manifest(&BuildManifest::new("app", vec!["app".to_string()]))
            .expect("core world builds")
    }

    fn abstract_integer(world: &mut CompilationWorld) -> crate::SemanticValueRef {
        let integer = world
            .resolve_type_value("integer")
            .expect("abstract integer Type resolves");
        let id = world
            .semantic_world
            .install_abstract_literal_value(
                crate::AbstractLiteralFamily::Integer,
                crate::AbstractLiteralExactValue::Integer("42".to_string()),
                integer,
                crate::compile_literal_policy(),
                Provenance::new("literal construction test source"),
            )
            .expect("abstract integer installs");
        crate::SemanticValueRef {
            id,
            type_value: integer,
        }
    }

    fn add_test_candidate(
        world: &mut CompilationWorld,
        target_type: crate::TypeValueId,
        mode: PolicyMode,
        body: crate::semantic_world::OrdinaryIntrinsicBody,
    ) -> crate::SemanticValueId {
        let view = PolicyView {
            pair: crate::compile_literal_policy(),
            mode,
        };
        let backing = crate::SymbolId(world.next_intrinsic_backing);
        world.next_intrinsic_backing = world
            .next_intrinsic_backing
            .checked_sub(1)
            .expect("test intrinsic declaration ids remain available");
        world
            .semantic_world
            .register_intrinsic_type_operation(
                target_type,
                CONSTRUCT_OR_CONVERT_SELECTOR,
                backing,
                body,
                view.clone(),
                view,
                Provenance::new("test literal constructor candidate"),
            )
            .expect("test candidate enters target callspace")
    }

    fn request(
        world: &mut CompilationWorld,
        target_name: &str,
        mode: PolicyMode,
    ) -> crate::ConstructionRequest {
        let source = abstract_integer(world);
        let target = world
            .resolve_complete_annotation_type(target_name)
            .expect("target observation succeeds")
            .expect("target resolves to complete tau");
        crate::ConstructionRequest {
            source,
            target,
            result_demand: ResultPolicyDemand {
                pair_query: P1Projection::Infer,
                mode,
            },
            family: crate::ConstructionFamily::ConstructOrConvert,
        }
    }

    fn constructed_count(world: &CompilationWorld) -> usize {
        world
            .semantic_world
            .values()
            .filter(|value| {
                matches!(
                    value.payload,
                    crate::SemanticValuePayload::ConstructedLiteral { .. }
                )
            })
            .count()
    }

    #[test]
    fn complete_type_without_constructor_is_not_constructible() {
        let mut world = world();
        let request = request(&mut world, "type", PolicyMode::Plain);
        let before = constructed_count(&world);
        let error = world
            .invoke_literal_construction_request(
                request,
                &Provenance::new("missing constructor candidate"),
            )
            .expect_err("the numeric registry is not construction authority");
        assert!(error.diagnostics[0]
            .message
            .contains("no semantic target values"));
        assert_eq!(constructed_count(&world), before);
    }

    #[test]
    fn selected_constructor_failure_does_not_run_plain_runner_up() {
        let mut world = world();
        let target = world.resolve_type_value("uint16").expect("uint16 resolves");
        add_test_candidate(
            &mut world,
            target,
            PolicyMode::Const,
            crate::semantic_world::OrdinaryIntrinsicBody::FailSelected,
        );
        let request = request(&mut world, "uint16", PolicyMode::Const);
        let before = constructed_count(&world);
        let error = world
            .invoke_literal_construction_request(
                request,
                &Provenance::new("selected constructor failure"),
            )
            .expect_err("the preferred selected body fails terminally");
        assert!(error.diagnostics[0].message.contains("failed to realize"));
        assert_eq!(
            constructed_count(&world),
            before,
            "the runnable plain builtin was not retried"
        );
    }

    #[test]
    fn selected_delete_and_ambiguity_do_not_fabricate_literal_results() {
        let mut deleted = world();
        let target = deleted
            .resolve_type_value("uint16")
            .expect("uint16 resolves");
        add_test_candidate(
            &mut deleted,
            target,
            PolicyMode::Mut,
            crate::semantic_world::OrdinaryIntrinsicBody::Delete,
        );
        let deleted_request = request(&mut deleted, "uint16", PolicyMode::Mut);
        let before = constructed_count(&deleted);
        let error = deleted
            .invoke_literal_construction_request(
                deleted_request,
                &Provenance::new("deleted constructor"),
            )
            .expect_err("delete winner rejects");
        assert!(error.diagnostics[0].message.contains("deleted"));
        assert_eq!(constructed_count(&deleted), before);

        let mut ambiguous = world();
        let target = ambiguous
            .resolve_type_value("uint16")
            .expect("uint16 resolves");
        let builtin_spec = crate::BuiltinNumericConstructorSpec {
            source_family: crate::AbstractLiteralFamily::Integer,
            target_key: crate::NumericTypeKey::new(crate::NumericFamily::Uint, 16),
            target_type: target,
        };
        add_test_candidate(
            &mut ambiguous,
            target,
            PolicyMode::Plain,
            crate::semantic_world::OrdinaryIntrinsicBody::AbstractLiteralConstruct(builtin_spec),
        );
        let ambiguous_request = request(&mut ambiguous, "uint16", PolicyMode::Plain);
        let before = constructed_count(&ambiguous);
        let error = ambiguous
            .invoke_literal_construction_request(
                ambiguous_request,
                &Provenance::new("ambiguous constructors"),
            )
            .expect_err("equal maxima are ambiguous");
        assert!(error.diagnostics[0].message.contains("multiple maximal"));
        assert_eq!(constructed_count(&ambiguous), before);
    }
}
