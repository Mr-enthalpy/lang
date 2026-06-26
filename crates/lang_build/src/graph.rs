use std::collections::{BTreeMap, BTreeSet};

use crate::model::{
    ChildBucket, ChildLink, ChildNameRole, Diagnostic, DiagnosticSeverity, NamespaceDelta,
    NamespaceNode, NamespaceNodeId, NamespaceNodeKind, PolicyMetadata, Provenance, SourceCategory,
    SymbolId, SymbolKind, SymbolObject, SymbolPayload,
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
                Diagnostic::hard_error(format!("unresolved symbol `{name}`"), None)
                    .with_node_context(parent)
            })?;
        select_symbol_from_bucket(&self.symbols, bucket, name, expectation).ok_or_else(|| {
            Diagnostic::hard_error(
                format!("unresolved symbol `{name}` for resolver expectation {expectation:?}"),
                None,
            )
            .with_node_context(parent)
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
                if let Some(existing_symbol) = parent_node
                    .children
                    .get(name)
                    .and_then(|bucket| bucket.get(*role))
                {
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

                if let (Some(bucket), Some(symbol_object)) =
                    (parent_node.children.get(name), linked_symbol)
                {
                    let opposite = match role {
                        ChildNameRole::Object => bucket.namespace_subspace,
                        ChildNameRole::NamespaceSubspace => bucket.object,
                    };
                    if cross_role_namespace_capable_conflict(
                        *role,
                        symbol_object,
                        opposite.and_then(|id| self.symbols.get(&id)),
                    ) {
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

            if !pending_links.insert((*parent, name.clone(), *role)) {
                diagnostics.push(
                    Diagnostic::hard_error(
                        format!(
                            "delta install conflict: duplicate symbol `{name}` in same namespace for role {role:?}"
                        ),
                        Some(provenance.clone()),
                    )
                    .with_node_context(*parent),
                );
            }

            let pending_bucket = pending_buckets.entry((*parent, name.clone())).or_default();
            if let Some(symbol_object) = linked_symbol {
                let opposite = match role {
                    ChildNameRole::Object => pending_bucket.namespace_subspace,
                    ChildNameRole::NamespaceSubspace => pending_bucket.object,
                };
                let opposite_symbol = opposite
                    .and_then(|id| self.symbols.get(&id))
                    .or_else(|| opposite.and_then(|id| delta.symbols.get(&id)));
                if cross_role_namespace_capable_conflict(*role, symbol_object, opposite_symbol) {
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
/// source-order explicit paths such as `uint8::core`, and `default_mounts`
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
    pub fn resolve(
        &self,
        source_order_path: &[String],
        context: &ResolverContext,
    ) -> Result<SymbolObject, Diagnostic> {
        self.resolve_with_expectation(source_order_path, context, ResolveExpectation::AnyUnique)
    }

    pub fn resolve_with_expectation(
        &self,
        source_order_path: &[String],
        context: &ResolverContext,
        terminal_expectation: ResolveExpectation,
    ) -> Result<SymbolObject, Diagnostic> {
        if source_order_path.is_empty() {
            return Err(self.hard_error(None, "unresolved empty namespace path"));
        }

        let mut hits = Vec::new();
        let mut errors = Vec::new();
        match self.resolve_from(
            source_order_path,
            context.current_namespace,
            terminal_expectation,
        ) {
            Ok(symbol) => hits.push(symbol),
            Err(diagnostic) => errors.push(diagnostic),
        }

        for mount_root in &context.explicit_mount_roots {
            match self.resolve_from(source_order_path, *mount_root, terminal_expectation) {
                Ok(symbol) => hits.push(symbol),
                Err(diagnostic) => errors.push(diagnostic),
            }
        }

        if source_order_path.len() == 1 {
            for mount in &context.default_mounts {
                match self.resolve_from(source_order_path, *mount, terminal_expectation) {
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
                .find(|diagnostic| diagnostic.message.contains("ambiguous"))
                .unwrap_or_else(|| {
                    self.hard_error(
                        None,
                        format!("unresolved symbol `{}`", source_order_path.join("::")),
                    )
                })),
            _ => Err(self.hard_error(
                None,
                format!(
                    "conflicting symbol `{}` found across resolver search roots",
                    source_order_path.join("::")
                ),
            )),
        }
    }

    pub fn resolve_str(
        &self,
        source_order_path: &str,
        context: &ResolverContext,
    ) -> Result<SymbolObject, Diagnostic> {
        self.resolve_str_with_expectation(source_order_path, context, ResolveExpectation::AnyUnique)
    }

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

    fn resolve_from(
        &self,
        source_order_path: &[String],
        start: NamespaceNodeId,
        terminal_expectation: ResolveExpectation,
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

        current_symbol.ok_or_else(|| self.hard_error(None, "unresolved empty namespace path"))
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
        let symbol = SymbolObject::namespace(
            symbol_id,
            key,
            node_id,
            NamespaceNodeKind::Virtual,
            source_category,
            Some(parent),
            provenance,
        );
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
    let symbol = SymbolObject::namespace(
        symbol_id,
        name,
        node_id,
        node_kind,
        source_category,
        Some(parent),
        provenance,
    );
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
            let mut ids = [bucket.object, bucket.namespace_subspace]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            ids.sort();
            ids.dedup();
            match ids.as_slice() {
                [id] => symbol(*id).map(Ok),
                [] => None,
                _ => Some(Err(Diagnostic::hard_error(
                    format!(
                        "ambiguous terminal symbol `{name}` across object and namespace-subspace roles"
                    ),
                    None,
                ))),
            }
        }
        ResolveExpectation::Object => bucket.object.and_then(symbol).map(Ok),
        ResolveExpectation::NamespaceSubspace => bucket.namespace_subspace.and_then(symbol).map(Ok),
        ResolveExpectation::NamespaceCapableParent => {
            let mut candidates = Vec::new();
            if let Some(namespace_symbol) = bucket.namespace_subspace.and_then(symbol) {
                candidates.push(namespace_symbol);
            }
            if let Some(object_symbol) = bucket.object.and_then(symbol) {
                if object_symbol.namespace_node().is_some() {
                    candidates.push(object_symbol);
                }
            }
            match candidates.as_slice() {
                [symbol] => Some(Ok(*symbol)),
                [] => None,
                _ => Some(Err(Diagnostic::hard_error(
                    format!("ambiguous namespace-capable parent `{name}`"),
                    None,
                ))),
            }
        }
        ResolveExpectation::TypeObject => bucket
            .object
            .and_then(symbol)
            .and_then(|symbol| (symbol.kind == SymbolKind::Type).then_some(Ok(symbol))),
        ResolveExpectation::MetaFunction => bucket
            .object
            .and_then(symbol)
            .and_then(|symbol| (symbol.kind == SymbolKind::MetaFunction).then_some(Ok(symbol))),
        ResolveExpectation::FieldFunction => bucket
            .object
            .and_then(symbol)
            .and_then(|symbol| (symbol.kind == SymbolKind::FieldFunction).then_some(Ok(symbol))),
    }
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
