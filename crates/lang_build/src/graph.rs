use std::collections::{BTreeMap, BTreeSet};

use crate::model::{
    ChildBucket, ChildLink, ChildNameRole, Diagnostic, DiagnosticSeverity, NamespaceDelta,
    NamespaceNode, NamespaceNodeId, NamespaceNodeKind, PolicyEnv, PolicyFlag, PolicyMetadata,
    Provenance, ResolverCode, SourceCategory, SymbolId, SymbolKind, SymbolObject, SymbolPayload,
};

/// Immutable namespace graph world snapshot.
///
/// All resolver, declaration harvesting, core bootstrap, and early-meta logic
/// reads from this canonical graph model.
#[derive(Clone, Debug)]
pub struct NamespaceGraphSnapshot {
    snapshot_id: u64,
    root_node: NamespaceNodeId,
    nodes: BTreeMap<NamespaceNodeId, NamespaceNode>,
    symbols: BTreeMap<SymbolId, SymbolObject>,
    diagnostics: Vec<Diagnostic>,
    next_node_id: u64,
    next_symbol_id: u64,
}

impl NamespaceGraphSnapshot {
    pub fn new() -> Self {
        let root_node = NamespaceNodeId(0);
        let root = NamespaceNode::new(
            root_node,
            "<root>",
            NamespaceNodeKind::Virtual,
            SourceCategory::CoreBootstrap,
            None,
            Provenance::new("namespace graph root"),
        );
        let mut nodes = BTreeMap::new();
        nodes.insert(root_node, root);

        Self {
            snapshot_id: 0,
            root_node,
            nodes,
            symbols: BTreeMap::new(),
            diagnostics: Vec::new(),
            next_node_id: 1,
            next_symbol_id: 1,
        }
    }

    pub fn snapshot_id(&self) -> u64 {
        self.snapshot_id
    }

    pub fn root_node(&self) -> NamespaceNodeId {
        self.root_node
    }

    pub fn empty_delta(&self) -> NamespaceDelta {
        NamespaceDelta::new(self.snapshot_id, self.next_node_id, self.next_symbol_id)
    }

    pub fn capability(&self) -> NamespaceGraphCapability<'_> {
        NamespaceGraphCapability { snapshot: self }
    }

    pub fn node(&self, id: NamespaceNodeId) -> Option<&NamespaceNode> {
        self.nodes.get(&id)
    }

    pub fn symbol(&self, id: SymbolId) -> Option<&SymbolObject> {
        self.symbols.get(&id)
    }

    pub fn symbols(&self) -> &BTreeMap<SymbolId, SymbolObject> {
        &self.symbols
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn child_symbol(&self, parent: NamespaceNodeId, name: &str) -> Option<&SymbolObject> {
        self.child_symbol_with_expectation(parent, name, ResolveExpectation::AnyUnique)
            .ok()
    }

    pub fn child_symbol_with_expectation(
        &self,
        parent: NamespaceNodeId,
        name: &str,
        expectation: ResolveExpectation,
    ) -> Result<&SymbolObject, Diagnostic> {
        let bucket = self
            .nodes
            .get(&parent)
            .and_then(|node| node.children.get(name))
            .ok_or_else(|| {
                Diagnostic::hard_error(format!("resolver error: unresolved symbol `{name}`"), None)
                    .with_node_context(parent)
                    .with_code(ResolverCode::Unresolved)
            })?;
        select_symbol_from_bucket(&self.symbols, bucket, name, expectation).ok_or_else(|| {
            Diagnostic::hard_error(
                format!(
                    "resolver error: unresolved symbol `{name}` for expectation {expectation:?}"
                ),
                None,
            )
            .with_node_context(parent)
            .with_code(ResolverCode::Unresolved)
        })?
    }

    pub fn install_delta(
        &self,
        delta: NamespaceDelta,
    ) -> Result<NamespaceGraphSnapshot, NamespaceInstallError> {
        self.validate_delta(&delta)?;

        let next_node_id = delta.next_node_id();
        let next_symbol_id = delta.next_symbol_id();
        let mut next = self.clone();
        for (id, node) in delta.nodes {
            next.nodes.insert(id, node);
        }
        for (id, symbol) in delta.symbols {
            next.symbols.insert(id, symbol);
        }
        for link in delta.child_links {
            let parent = next
                .nodes
                .get_mut(&link.parent)
                .expect("delta validation ensures parent exists");
            parent
                .children
                .entry(link.name)
                .or_default()
                .set(link.role, link.symbol);
        }
        next.diagnostics.extend(delta.diagnostics);
        next.next_node_id = next_node_id;
        next.next_symbol_id = next_symbol_id;
        next.snapshot_id += 1;
        Ok(next)
    }

    fn validate_delta(&self, delta: &NamespaceDelta) -> Result<(), NamespaceInstallError> {
        if delta.base_snapshot_id != self.snapshot_id {
            return Err(NamespaceInstallError::single(Diagnostic::hard_error(
                "delta install conflict: base snapshot id does not match current snapshot",
                None,
            )));
        }

        let mut diagnostics = Vec::new();
        let mut pending_links = BTreeSet::new();
        let mut pending_buckets: BTreeMap<(NamespaceNodeId, String), ChildBucket> = BTreeMap::new();

        for id in delta.nodes.keys() {
            if self.nodes.contains_key(id) {
                diagnostics.push(Diagnostic::hard_error(
                    format!(
                        "delta install conflict: duplicate namespace node id {}",
                        id.as_u64()
                    ),
                    None,
                ));
            }
        }

        for id in delta.symbols.keys() {
            if self.symbols.contains_key(id) {
                diagnostics.push(Diagnostic::hard_error(
                    format!(
                        "delta install conflict: duplicate symbol id {}",
                        id.as_u64()
                    ),
                    None,
                ));
            }
        }

        for ChildLink {
            parent,
            name,
            symbol,
            role,
            provenance,
        } in &delta.child_links
        {
            let parent_exists = self.nodes.contains_key(parent) || delta.nodes.contains_key(parent);
            if !parent_exists {
                diagnostics.push(Diagnostic::hard_error(
                    format!(
                        "delta install conflict: parent namespace node {} does not exist",
                        parent.as_u64()
                    ),
                    Some(provenance.clone()),
                ));
                continue;
            }

            if !(self.symbols.contains_key(symbol) || delta.symbols.contains_key(symbol)) {
                diagnostics.push(Diagnostic::hard_error(
                    format!(
                        "delta install conflict: symbol {} does not exist",
                        symbol.as_u64()
                    ),
                    Some(provenance.clone()),
                ));
            }

            let linked_symbol = self
                .symbols
                .get(symbol)
                .or_else(|| delta.symbols.get(symbol));

            if let Some(parent_node) = self.nodes.get(parent) {
                if let Some(bucket) = parent_node.children.get(name) {
                    match role {
                        ChildNameRole::Object => {
                            for existing_symbol in bucket.object_symbols() {
                                let existing = self.symbols.get(existing_symbol);
                                if !object_symbols_are_overload_compatible(existing, linked_symbol)
                                {
                                    diagnostics.push(
                                        Diagnostic::hard_error(
                                            format!(
                                                "delta install conflict: symbol `{name}` already exists for role {role:?}"
                                            ),
                                            Some(provenance.clone()),
                                        )
                                        .with_node_context(*parent)
                                        .with_symbol_context(*existing_symbol),
                                    );
                                }
                            }
                        }
                        ChildNameRole::NamespaceSubspace => {
                            if let Some(existing_symbol) = bucket.namespace_subspace {
                                diagnostics.push(
                                    Diagnostic::hard_error(
                                        format!(
                                            "delta install conflict: symbol `{name}` already exists for role {role:?}"
                                        ),
                                        Some(provenance.clone()),
                                    )
                                    .with_node_context(*parent)
                                    .with_symbol_context(existing_symbol),
                                );
                            }
                        }
                    }
                }

                if let (Some(bucket), Some(symbol_object)) =
                    (parent_node.children.get(name), linked_symbol)
                {
                    let opposite_conflict = match role {
                        ChildNameRole::Object => cross_role_namespace_capable_conflict(
                            *role,
                            symbol_object,
                            bucket
                                .namespace_subspace
                                .and_then(|id| self.symbols.get(&id)),
                        ),
                        ChildNameRole::NamespaceSubspace => bucket
                            .object_symbols()
                            .iter()
                            .filter_map(|id| self.symbols.get(id))
                            .any(|opposite| {
                                cross_role_namespace_capable_conflict(
                                    *role,
                                    symbol_object,
                                    Some(opposite),
                                )
                            }),
                    };
                    if opposite_conflict {
                        diagnostics.push(
                            Diagnostic::hard_error(
                                format!(
                                    "delta install conflict: namespace-capable symbol `{name}` conflicts with existing cross-role child"
                                ),
                                Some(provenance.clone()),
                            )
                            .with_node_context(*parent)
                            .with_symbol_context(symbol_object.id),
                        );
                    }
                }
            }

            if !pending_links.insert((*parent, name.clone(), *role, *symbol)) {
                diagnostics.push(
                    Diagnostic::hard_error(
                        format!(
                            "delta install conflict: duplicate symbol link `{name}` in same namespace for role {role:?}"
                        ),
                        Some(provenance.clone()),
                    )
                    .with_node_context(*parent),
                );
            }

            let pending_bucket = pending_buckets.entry((*parent, name.clone())).or_default();
            if let Some(symbol_object) = linked_symbol {
                if *role == ChildNameRole::Object {
                    for existing_symbol in pending_bucket.object_symbols() {
                        let existing = self
                            .symbols
                            .get(existing_symbol)
                            .or_else(|| delta.symbols.get(existing_symbol));
                        if !object_symbols_are_overload_compatible(existing, linked_symbol) {
                            diagnostics.push(
                                Diagnostic::hard_error(
                                    format!(
                                        "delta install conflict: duplicate symbol `{name}` in same namespace for role {role:?}"
                                    ),
                                    Some(provenance.clone()),
                                )
                                .with_node_context(*parent)
                                .with_symbol_context(*existing_symbol),
                            );
                        }
                    }
                } else if let Some(existing_symbol) = pending_bucket.namespace_subspace {
                    diagnostics.push(
                        Diagnostic::hard_error(
                            format!(
                                "delta install conflict: duplicate symbol `{name}` in same namespace for role {role:?}"
                            ),
                            Some(provenance.clone()),
                        )
                        .with_node_context(*parent)
                        .with_symbol_context(existing_symbol),
                    );
                }

                let opposite_conflict = match role {
                    ChildNameRole::Object => {
                        let opposite_symbol = pending_bucket
                            .namespace_subspace
                            .and_then(|id| self.symbols.get(&id))
                            .or_else(|| {
                                pending_bucket
                                    .namespace_subspace
                                    .and_then(|id| delta.symbols.get(&id))
                            });
                        cross_role_namespace_capable_conflict(*role, symbol_object, opposite_symbol)
                    }
                    ChildNameRole::NamespaceSubspace => pending_bucket
                        .object_symbols()
                        .iter()
                        .filter_map(|id| self.symbols.get(id).or_else(|| delta.symbols.get(id)))
                        .any(|opposite| {
                            cross_role_namespace_capable_conflict(
                                *role,
                                symbol_object,
                                Some(opposite),
                            )
                        }),
                };
                if opposite_conflict {
                    diagnostics.push(
                        Diagnostic::hard_error(
                            format!(
                                "delta install conflict: namespace-capable symbol `{name}` conflicts with pending cross-role child"
                            ),
                            Some(provenance.clone()),
                        )
                        .with_node_context(*parent)
                        .with_symbol_context(symbol_object.id),
                    );
                }
            }
            pending_bucket.set(*role, *symbol);
        }

        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(NamespaceInstallError { diagnostics })
        }
    }
}

impl Default for NamespaceGraphSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolver search context for a graph query.
///
/// `current_namespace` is searched first, `explicit_mount_roots` support
/// explicit paths such as `uint8::core`, and `default_mounts`
/// support short-name lookup such as `uint8`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolverContext {
    pub current_namespace: NamespaceNodeId,
    pub explicit_mount_roots: Vec<NamespaceNodeId>,
    pub default_mounts: Vec<NamespaceNodeId>,
    pub current_policy: PolicyMetadata,
}

impl ResolverContext {
    pub fn new(current_namespace: NamespaceNodeId) -> Self {
        Self {
            current_namespace,
            explicit_mount_roots: Vec::new(),
            default_mounts: Vec::new(),
            current_policy: PolicyMetadata::default(),
        }
    }

    pub fn with_default_mounts(
        current_namespace: NamespaceNodeId,
        default_mounts: Vec<NamespaceNodeId>,
    ) -> Self {
        Self {
            current_namespace,
            explicit_mount_roots: Vec::new(),
            default_mounts,
            current_policy: PolicyMetadata::default(),
        }
    }

    pub fn with_mounts(
        current_namespace: NamespaceNodeId,
        explicit_mount_roots: Vec<NamespaceNodeId>,
        default_mounts: Vec<NamespaceNodeId>,
    ) -> Self {
        Self {
            current_namespace,
            explicit_mount_roots,
            default_mounts,
            current_policy: PolicyMetadata::default(),
        }
    }
}

/// Expected result shape for resolver lookup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolveExpectation {
    AnyUnique,
    Object,
    NamespaceSubspace,
    NamespaceCapableParent,
    TypeObject,
    MetaFunction,
    FieldFunction,
}

/// Shared capability API for graph reads and delta construction.
pub struct NamespaceGraphCapability<'snapshot> {
    pub(crate) snapshot: &'snapshot NamespaceGraphSnapshot,
}

impl<'snapshot> NamespaceGraphCapability<'snapshot> {
    /// Resolve a namespace path to a globally unique symbol under the current
    /// resolver context.
    ///
    /// Uses [`ResolveExpectation::AnyUnique`] — the terminal component must be
    /// unique across both object and namespace-subspace roles. If both roles
    /// exist for the same name the lookup fails with an ambiguity diagnostic.
    ///
    /// Most semantic passes should use [`resolve_with_expectation`] or one of
    /// the typed helpers (`resolve_type_object`, `resolve_meta_function`, …)
    /// instead of plain `resolve`, because plain `resolve` rejects role
    /// coexistence that may be semantically valid.
    pub fn resolve(
        &self,
        source_order_path: &[String],
        context: &ResolverContext,
    ) -> Result<SymbolObject, Diagnostic> {
        self.resolve_with_expectation(source_order_path, context, ResolveExpectation::AnyUnique)
    }

    /// Resolve a namespace path with an explicit terminal role expectation.
    ///
    /// The `terminal_expectation` discriminates which child-name role the
    /// final path component must satisfy:
    ///
    /// | Expectation | Resolves |
    /// |---|---|
    /// | `AnyUnique` | Terminal must be unique across both roles (ambiguity otherwise) |
    /// | `Object` | Terminal in the object/function role |
    /// | `NamespaceSubspace` | Terminal in the namespace-subspace role |
    /// | `NamespaceCapableParent` | Terminal must be a namespace-capable symbol (object with `namespace_node` or namespace-subspace) |
    /// | `TypeObject` | Terminal with kind `Type` |
    /// | `MetaFunction` | Terminal with kind `MetaFunction` |
    /// | `FieldFunction` | Terminal with kind `FieldFunction` |
    ///
    /// Intermediate path components always resolve as
    /// `NamespaceCapableParent` regardless of the terminal expectation.
    pub fn resolve_with_expectation(
        &self,
        source_order_path: &[String],
        context: &ResolverContext,
        terminal_expectation: ResolveExpectation,
    ) -> Result<SymbolObject, Diagnostic> {
        self.resolve_search_roots(source_order_path, context, terminal_expectation, None)
    }

    /// Resolve a namespace path with an explicit terminal role expectation and
    /// policy environment filter.
    ///
    /// Symbols that do not satisfy `policy_env` are treated as if they do not
    /// exist in the search root. Policy filtering happens before cross-root
    /// conflict reporting.
    pub fn resolve_with_policy(
        &self,
        source_order_path: &[String],
        context: &ResolverContext,
        terminal_expectation: ResolveExpectation,
        policy_env: PolicyEnv,
    ) -> Result<SymbolObject, Diagnostic> {
        self.resolve_search_roots(
            source_order_path,
            context,
            terminal_expectation,
            Some(policy_env),
        )
    }

    /// Shared internal search-root loop with optional policy filtering.
    fn resolve_search_roots(
        &self,
        source_order_path: &[String],
        context: &ResolverContext,
        terminal_expectation: ResolveExpectation,
        policy_env: Option<PolicyEnv>,
    ) -> Result<SymbolObject, Diagnostic> {
        if source_order_path.is_empty() {
            return Err(self
                .hard_error(None, "unresolved empty namespace path")
                .with_code(ResolverCode::Unresolved));
        }

        let mut hits = Vec::new();
        let mut errors = Vec::new();
        match self.resolve_from_internal(
            source_order_path,
            context.current_namespace,
            terminal_expectation,
            policy_env,
        ) {
            Ok(symbol) => hits.push(symbol),
            Err(diagnostic) => errors.push(diagnostic),
        }

        for mount_root in &context.explicit_mount_roots {
            match self.resolve_from_internal(
                source_order_path,
                *mount_root,
                terminal_expectation,
                policy_env,
            ) {
                Ok(symbol) => hits.push(symbol),
                Err(diagnostic) => errors.push(diagnostic),
            }
        }

        if source_order_path.len() == 1 {
            for mount in &context.default_mounts {
                match self.resolve_from_internal(
                    source_order_path,
                    *mount,
                    terminal_expectation,
                    policy_env,
                ) {
                    Ok(symbol) => hits.push(symbol),
                    Err(diagnostic) => errors.push(diagnostic),
                }
            }
        }

        hits.sort_by_key(|symbol| symbol.id);
        hits.dedup_by_key(|symbol| symbol.id);

        match hits.as_slice() {
            [symbol] => Ok(symbol.clone()),
            [] => Err(errors
                .into_iter()
                .find(|diagnostic| diagnostic.code == Some(ResolverCode::Ambiguous))
                .unwrap_or_else(|| {
                    self.hard_error(
                        None,
                        format!(
                            "resolver error: unresolved symbol `{}`",
                            source_order_path.join("::")
                        ),
                    )
                    .with_code(ResolverCode::Unresolved)
                })),
            _ => Err(self
                .hard_error(
                    None,
                    format!(
                        "resolver error: conflicting symbol `{}` across resolver search roots",
                        source_order_path.join("::")
                    ),
                )
                .with_code(ResolverCode::Conflict)),
        }
    }

    /// String convenience wrapper around [`resolve`](Self::resolve).
    ///
    /// Splits `source_order_path` on `::`, trims each component, and delegates
    /// to [`resolve`](Self::resolve). The terminal component must be globally
    /// unique; see [`resolve`](Self::resolve) for details.
    pub fn resolve_str(
        &self,
        source_order_path: &str,
        context: &ResolverContext,
    ) -> Result<SymbolObject, Diagnostic> {
        self.resolve_str_with_expectation(source_order_path, context, ResolveExpectation::AnyUnique)
    }

    /// String convenience wrapper around
    /// [`resolve_with_expectation`](Self::resolve_with_expectation).
    ///
    /// Splits `source_order_path` on `::`, trims each component, and delegates
    /// to [`resolve_with_expectation`](Self::resolve_with_expectation).
    pub fn resolve_str_with_expectation(
        &self,
        source_order_path: &str,
        context: &ResolverContext,
        terminal_expectation: ResolveExpectation,
    ) -> Result<SymbolObject, Diagnostic> {
        let components = source_order_path
            .split("::")
            .filter(|component| !component.is_empty())
            .map(str::trim)
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        self.resolve_with_expectation(&components, context, terminal_expectation)
    }

    /// String convenience wrapper around
    /// [`resolve_with_policy`](Self::resolve_with_policy).
    pub fn resolve_str_with_policy(
        &self,
        source_order_path: &str,
        context: &ResolverContext,
        terminal_expectation: ResolveExpectation,
        policy_env: PolicyEnv,
    ) -> Result<SymbolObject, Diagnostic> {
        let components = source_order_path
            .split("::")
            .filter(|component| !component.is_empty())
            .map(str::trim)
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        self.resolve_with_policy(&components, context, terminal_expectation, policy_env)
    }

    /// Resolve a terminal symbol whose kind is `Type`.
    ///
    /// Shortcut for `resolve_str_with_expectation(…, ResolveExpectation::TypeObject)`.
    pub fn resolve_type_object(
        &self,
        source_order_path: &str,
        context: &ResolverContext,
    ) -> Result<SymbolObject, Diagnostic> {
        self.resolve_str_with_expectation(
            source_order_path,
            context,
            ResolveExpectation::TypeObject,
        )
    }

    /// Policy-aware variant of [`resolve_type_object`](Self::resolve_type_object).
    pub fn resolve_type_object_with_policy(
        &self,
        source_order_path: &str,
        context: &ResolverContext,
        policy_env: PolicyEnv,
    ) -> Result<SymbolObject, Diagnostic> {
        self.resolve_str_with_policy(
            source_order_path,
            context,
            ResolveExpectation::TypeObject,
            policy_env,
        )
    }

    /// Resolve a terminal symbol whose kind is `MetaFunction`.
    ///
    /// Shortcut for `resolve_str_with_expectation(…, ResolveExpectation::MetaFunction)`.
    pub fn resolve_meta_function(
        &self,
        source_order_path: &str,
        context: &ResolverContext,
    ) -> Result<SymbolObject, Diagnostic> {
        self.resolve_str_with_expectation(
            source_order_path,
            context,
            ResolveExpectation::MetaFunction,
        )
    }

    /// Policy-aware variant of [`resolve_meta_function`](Self::resolve_meta_function).
    pub fn resolve_meta_function_with_policy(
        &self,
        source_order_path: &str,
        context: &ResolverContext,
        policy_env: PolicyEnv,
    ) -> Result<SymbolObject, Diagnostic> {
        self.resolve_str_with_policy(
            source_order_path,
            context,
            ResolveExpectation::MetaFunction,
            policy_env,
        )
    }

    /// Resolve a terminal symbol whose kind is `FieldFunction`.
    ///
    /// Shortcut for `resolve_str_with_expectation(…, ResolveExpectation::FieldFunction)`.
    pub fn resolve_field_function(
        &self,
        source_order_path: &str,
        context: &ResolverContext,
    ) -> Result<SymbolObject, Diagnostic> {
        self.resolve_str_with_expectation(
            source_order_path,
            context,
            ResolveExpectation::FieldFunction,
        )
    }

    /// Resolve a terminal symbol in the namespace-subspace role.
    ///
    /// Shortcut for `resolve_str_with_expectation(…, ResolveExpectation::NamespaceSubspace)`.
    pub fn resolve_namespace_subspace(
        &self,
        source_order_path: &str,
        context: &ResolverContext,
    ) -> Result<SymbolObject, Diagnostic> {
        self.resolve_str_with_expectation(
            source_order_path,
            context,
            ResolveExpectation::NamespaceSubspace,
        )
    }

    #[allow(dead_code)]
    fn resolve_from(
        &self,
        source_order_path: &[String],
        start: NamespaceNodeId,
        terminal_expectation: ResolveExpectation,
    ) -> Result<SymbolObject, Diagnostic> {
        self.resolve_from_internal(source_order_path, start, terminal_expectation, None)
    }

    /// Internal path-resolution with optional policy filtering at each step.
    fn resolve_from_internal(
        &self,
        source_order_path: &[String],
        start: NamespaceNodeId,
        terminal_expectation: ResolveExpectation,
        policy_env: Option<PolicyEnv>,
    ) -> Result<SymbolObject, Diagnostic> {
        let mut current_node = start;
        let mut current_symbol = None;

        let component_count = source_order_path.len();
        for (resolved_count, component) in source_order_path.iter().rev().enumerate() {
            let expectation = if resolved_count + 1 == component_count {
                terminal_expectation
            } else {
                ResolveExpectation::NamespaceCapableParent
            };
            let symbol = self.snapshot.child_symbol_with_expectation(
                current_node,
                component,
                expectation,
            )?;

            if !self.symbol_satisfies_policy(&symbol, policy_env) {
                return Err(self
                    .hard_error(
                        None,
                        format!("resolver error: unresolved symbol `{component}`"),
                    )
                    .with_code(ResolverCode::Unresolved));
            }

            current_symbol = Some(symbol.clone());

            if resolved_count + 1 != component_count {
                current_node = symbol.namespace_node().ok_or_else(|| {
                    self.hard_error(
                        Some(symbol.provenance.clone()),
                        format!("symbol `{}` is not a namespace parent", symbol.name),
                    )
                })?;
            }
        }

        current_symbol.ok_or_else(|| {
            self.hard_error(None, "unresolved empty namespace path")
                .with_code(ResolverCode::Unresolved)
        })
    }

    fn symbol_satisfies_policy(
        &self,
        symbol: &SymbolObject,
        policy_env: Option<PolicyEnv>,
    ) -> bool {
        match policy_env {
            None => true,
            Some(env) => match env {
                PolicyEnv::Meta => symbol.policy_metadata.policy_set.contains(PolicyFlag::Meta),
                PolicyEnv::Runtime => symbol
                    .policy_metadata
                    .policy_set
                    .contains(PolicyFlag::Runtime),
            },
        }
    }

    pub fn declare(
        &self,
        parent: NamespaceNodeId,
        name: impl Into<String>,
        kind: SymbolKind,
        source_category: SourceCategory,
        provenance: Provenance,
    ) -> NamespaceDelta {
        let mut delta = self.snapshot.empty_delta();
        let id = delta.allocate_symbol_id();
        let symbol =
            SymbolObject::placeholder(id, name, kind, source_category, Some(parent), provenance);
        delta.insert_symbol(parent, symbol);
        delta
    }

    pub fn inject_child(
        &self,
        parent: NamespaceNodeId,
        object: SymbolObject,
        _provenance: Provenance,
    ) -> NamespaceDelta {
        let mut delta = self.snapshot.empty_delta();
        delta.insert_symbol(parent, object);
        delta
    }

    pub fn alias(
        &self,
        parent: NamespaceNodeId,
        name: impl Into<String>,
        target: SymbolId,
        provenance: Provenance,
    ) -> NamespaceDelta {
        let mut delta = self.snapshot.empty_delta();
        let id = delta.allocate_symbol_id();
        let mut symbol = SymbolObject::placeholder(
            id,
            name,
            SymbolKind::Alias,
            SourceCategory::Alias,
            Some(parent),
            provenance,
        );
        symbol.payload = SymbolPayload::Alias { target };
        delta.insert_symbol(parent, symbol);
        delta
    }

    pub fn open_virtual_node(
        &self,
        parent: NamespaceNodeId,
        key: impl Into<String>,
        source_category: SourceCategory,
        provenance: Provenance,
    ) -> (NamespaceNodeId, NamespaceDelta) {
        let mut delta = self.snapshot.empty_delta();
        let key = key.into();
        let node_id = delta.allocate_node_id();
        let symbol_id = delta.allocate_symbol_id();
        let node = NamespaceNode::new(
            node_id,
            key.clone(),
            NamespaceNodeKind::Virtual,
            source_category,
            Some(parent),
            provenance.clone(),
        );
        let mut symbol = SymbolObject::namespace(
            symbol_id,
            key,
            node_id,
            NamespaceNodeKind::Virtual,
            source_category,
            Some(parent),
            provenance,
        );
        symbol.policy_metadata.policy_set = crate::policy_set_meta_runtime();
        delta.insert_node(node);
        delta.insert_symbol(parent, symbol);
        (node_id, delta)
    }

    pub fn install_delta(
        &self,
        delta: NamespaceDelta,
    ) -> Result<NamespaceGraphSnapshot, NamespaceInstallError> {
        self.snapshot.install_delta(delta)
    }

    pub fn diagnostic(
        &self,
        provenance: Option<Provenance>,
        message: impl Into<String>,
        severity: DiagnosticSeverity,
    ) -> Diagnostic {
        Diagnostic::new(severity, message, provenance)
    }

    pub fn hard_error(
        &self,
        provenance: Option<Provenance>,
        message: impl Into<String>,
    ) -> Diagnostic {
        Diagnostic::hard_error(message, provenance)
    }
}

/// Error returned when a namespace delta cannot be atomically installed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamespaceInstallError {
    pub diagnostics: Vec<Diagnostic>,
}

impl NamespaceInstallError {
    pub fn single(diagnostic: Diagnostic) -> Self {
        Self {
            diagnostics: vec![diagnostic],
        }
    }
}

/// Build/world construction error carrying diagnostics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildError {
    pub diagnostics: Vec<Diagnostic>,
}

impl BuildError {
    pub fn single(diagnostic: Diagnostic) -> Self {
        Self {
            diagnostics: vec![diagnostic],
        }
    }

    pub fn extend(mut self, diagnostics: Vec<Diagnostic>) -> Self {
        self.diagnostics.extend(diagnostics);
        self
    }
}

impl From<NamespaceInstallError> for BuildError {
    fn from(error: NamespaceInstallError) -> Self {
        Self {
            diagnostics: error.diagnostics,
        }
    }
}

pub(crate) fn namespace_symbol(
    delta: &mut NamespaceDelta,
    parent: NamespaceNodeId,
    name: impl Into<String>,
    node_kind: NamespaceNodeKind,
    source_category: SourceCategory,
    provenance: Provenance,
) -> NamespaceNodeId {
    let name = name.into();
    let node_id = delta.allocate_node_id();
    let symbol_id = delta.allocate_symbol_id();
    let node = NamespaceNode::new(
        node_id,
        name.clone(),
        node_kind,
        source_category,
        Some(parent),
        provenance.clone(),
    );
    let mut symbol = SymbolObject::namespace(
        symbol_id,
        name,
        node_id,
        node_kind,
        source_category,
        Some(parent),
        provenance,
    );
    symbol.policy_metadata.policy_set = crate::policy_set_meta_runtime();
    delta.insert_node(node);
    delta.insert_symbol(parent, symbol);
    node_id
}

fn select_symbol_from_bucket<'symbols>(
    symbols: &'symbols BTreeMap<SymbolId, SymbolObject>,
    bucket: &ChildBucket,
    name: &str,
    expectation: ResolveExpectation,
) -> Option<Result<&'symbols SymbolObject, Diagnostic>> {
    let symbol = |id: SymbolId| symbols.get(&id);
    match expectation {
        ResolveExpectation::AnyUnique => {
            let mut ids = bucket.object_symbols().to_vec();
            ids.extend(bucket.namespace_subspace);
            ids.sort();
            ids.dedup();
            match ids.as_slice() {
                [id] => symbol(*id).map(Ok),
                [] => None,
                _ => Some(Err(Diagnostic::hard_error(
                    format!(
                        "resolver error: ambiguous terminal symbol `{name}` across object and namespace-subspace roles"
                    ),
                    None,
                )
                .with_code(ResolverCode::Ambiguous))),
            }
        }
        ResolveExpectation::Object => select_unique_object_symbol(symbols, bucket, name, |_| true),
        ResolveExpectation::NamespaceSubspace => bucket.namespace_subspace.and_then(symbol).map(Ok),
        ResolveExpectation::NamespaceCapableParent => {
            let mut candidates = Vec::new();
            if let Some(namespace_symbol) = bucket.namespace_subspace.and_then(symbol) {
                candidates.push(namespace_symbol);
            }
            for object_symbol in bucket.object_symbols().iter().filter_map(|id| symbol(*id)) {
                if object_symbol.namespace_node().is_some() {
                    candidates.push(object_symbol);
                }
            }
            match candidates.as_slice() {
                [symbol] => Some(Ok(*symbol)),
                [] => None,
                _ => Some(Err(Diagnostic::hard_error(
                    format!("resolver error: ambiguous namespace-capable parent `{name}`"),
                    None,
                )
                .with_code(ResolverCode::Ambiguous))),
            }
        }
        ResolveExpectation::TypeObject => {
            select_unique_object_symbol(symbols, bucket, name, |symbol| {
                symbol.kind == SymbolKind::Type
            })
        }
        ResolveExpectation::MetaFunction => {
            select_unique_object_symbol(symbols, bucket, name, |symbol| {
                symbol.kind == SymbolKind::MetaFunction
            })
        }
        ResolveExpectation::FieldFunction => {
            select_unique_object_symbol(symbols, bucket, name, |symbol| {
                symbol.kind == SymbolKind::FieldFunction
            })
        }
    }
}

fn select_unique_object_symbol<'symbols>(
    symbols: &'symbols BTreeMap<SymbolId, SymbolObject>,
    bucket: &ChildBucket,
    name: &str,
    predicate: impl Fn(&SymbolObject) -> bool,
) -> Option<Result<&'symbols SymbolObject, Diagnostic>> {
    let mut candidates = bucket
        .object_symbols()
        .iter()
        .filter_map(|id| symbols.get(id))
        .filter(|symbol| predicate(symbol))
        .collect::<Vec<_>>();
    candidates.sort_by_key(|symbol| symbol.id);
    match candidates.as_slice() {
        [symbol] => Some(Ok(*symbol)),
        [] => None,
        _ => Some(Err(
            Diagnostic::hard_error(
                format!(
                    "resolver error: non-call lookup of same-name overload set `{name}` requires overload context"
                ),
                None,
            )
            .with_code(ResolverCode::Ambiguous),
        )),
    }
}

fn object_symbols_are_overload_compatible(
    existing: Option<&SymbolObject>,
    incoming: Option<&SymbolObject>,
) -> bool {
    matches!(
        (existing, incoming),
        (Some(left), Some(right))
            if left.kind == SymbolKind::MetaFunction && right.kind == SymbolKind::MetaFunction
    )
}

fn cross_role_namespace_capable_conflict(
    role: ChildNameRole,
    symbol: &SymbolObject,
    opposite: Option<&SymbolObject>,
) -> bool {
    let Some(opposite) = opposite else {
        return false;
    };
    match role {
        ChildNameRole::Object => symbol.namespace_node().is_some(),
        ChildNameRole::NamespaceSubspace => opposite.namespace_node().is_some(),
    }
}
