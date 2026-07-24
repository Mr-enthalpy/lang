//! Owner-aware namespace views, package boundaries, and mount redirects.
//!
//! This is the canonical typed substrate for the persistent namespace forest.
//! The older `NamespaceGraphSnapshot` remains a compatibility transport while
//! v0.6 consumers migrate to this model.

use std::collections::BTreeMap;

use crate::{
    policy_pair::NamespaceVisibility,
    semantic_owner::{PackageId, SemanticOwnerGraph, SemanticOwnerId, SemanticSymbolIdentity},
};

/// Graph-local traversal handle, not symbol or namespace semantic identity.
///
/// Long-lived equality is carried by the node's `SemanticOwnerId`; mount
/// resolution returns the target symbol identity rather than this handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OwnerNamespaceNodeId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NamespaceNameView {
    FullNameView,
    ExternalNameView,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtractionMemberVisibility {
    Default,
    Public,
    Private,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamespaceSymbolEntry {
    pub identity: SemanticSymbolIdentity,
    pub declaration_owner: SemanticOwnerId,
    pub namespace_visibility: NamespaceVisibility,
    pub in_export_retention_closure: bool,
    pub has_external_candidate_view: bool,
    pub extraction_visibility: ExtractionMemberVisibility,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnerNamespaceNode {
    pub id: OwnerNamespaceNodeId,
    pub owner: SemanticOwnerId,
    pub parent: Option<OwnerNamespaceNodeId>,
    pub local_name: String,
    /// An explicit package boundary. Descendants inherit the nearest boundary.
    pub package_boundary: Option<PackageId>,
    /// Resolution redirect. A mount is an edge to an existing target node; it
    /// never owns a copied candidate or symbol.
    pub mount_target: Option<OwnerNamespaceNodeId>,
    pub visibility: NamespaceVisibility,
    pub children: BTreeMap<String, OwnerNamespaceNodeId>,
    pub symbols: BTreeMap<String, Vec<NamespaceSymbolEntry>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NamespaceLookupFailure {
    Unresolved,
    NotInExportRetentionDomain,
    PrivatePath,
    NoExternallyEligibleCandidate,
    MountTargetMissing,
    PackageBoundaryViolation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamespaceLookupResult {
    /// Identity-preserving candidate entries. Path resolution never selects an
    /// overload; the ordinary admissibility/partial-order pipeline consumes
    /// this complete exposed set later.
    pub candidate_identities: Vec<SemanticSymbolIdentity>,
    pub view: NamespaceNameView,
    pub crossed_package_boundary: bool,
}

#[derive(Clone, Debug)]
pub struct OwnerNamespaceGraph {
    nodes: BTreeMap<OwnerNamespaceNodeId, OwnerNamespaceNode>,
    next_node: u64,
}

impl Default for OwnerNamespaceGraph {
    fn default() -> Self {
        Self {
            nodes: BTreeMap::new(),
            next_node: 0,
        }
    }
}

impl OwnerNamespaceGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(
        &mut self,
        owner: SemanticOwnerId,
        parent: Option<OwnerNamespaceNodeId>,
        local_name: impl Into<String>,
        package_boundary: Option<PackageId>,
        visibility: NamespaceVisibility,
    ) -> OwnerNamespaceNodeId {
        let id = OwnerNamespaceNodeId(self.next_node);
        self.next_node = self
            .next_node
            .checked_add(1)
            .expect("owner namespace graph exhausted u64 identity space");
        let local_name = local_name.into();
        self.nodes.insert(
            id,
            OwnerNamespaceNode {
                id,
                owner,
                parent,
                local_name: local_name.clone(),
                package_boundary,
                mount_target: None,
                visibility,
                children: BTreeMap::new(),
                symbols: BTreeMap::new(),
            },
        );
        if let Some(parent) = parent {
            self.nodes
                .get_mut(&parent)
                .unwrap_or_else(|| panic!("unknown namespace parent {parent:?}"))
                .children
                .insert(local_name, id);
        }
        id
    }

    pub fn add_mount(
        &mut self,
        owner: SemanticOwnerId,
        parent: OwnerNamespaceNodeId,
        local_name: impl Into<String>,
        target: OwnerNamespaceNodeId,
        visibility: NamespaceVisibility,
    ) -> OwnerNamespaceNodeId {
        let mount = self.add_node(owner, Some(parent), local_name, None, visibility);
        self.nodes
            .get_mut(&mount)
            .expect("newly allocated mount exists")
            .mount_target = Some(target);
        mount
    }

    pub fn add_symbol(
        &mut self,
        node: OwnerNamespaceNodeId,
        name: impl Into<String>,
        entry: NamespaceSymbolEntry,
    ) {
        self.nodes
            .get_mut(&node)
            .unwrap_or_else(|| panic!("unknown namespace node {node:?}"))
            .symbols
            .entry(name.into())
            .or_default()
            .push(entry);
    }

    pub fn node(&self, node: OwnerNamespaceNodeId) -> Option<&OwnerNamespaceNode> {
        self.nodes.get(&node)
    }

    pub fn package_of(&self, node: OwnerNamespaceNodeId) -> Option<PackageId> {
        let mut current = Some(node);
        while let Some(id) = current {
            let node = self.nodes.get(&id)?;
            if let Some(package) = node.package_boundary {
                return Some(package);
            }
            current = node.parent;
        }
        None
    }

    fn validated_package_of(
        &self,
        owners: &SemanticOwnerGraph,
        node: OwnerNamespaceNodeId,
    ) -> Result<PackageId, NamespaceLookupFailure> {
        let package = self
            .package_of(node)
            .ok_or(NamespaceLookupFailure::PackageBoundaryViolation)?;
        let node_owner = self
            .node(node)
            .ok_or(NamespaceLookupFailure::Unresolved)?
            .owner;
        (owners.node(node_owner).map(|node| node.package) == Some(package))
            .then_some(package)
            .ok_or(NamespaceLookupFailure::PackageBoundaryViolation)
    }

    pub fn can_lexically_see(
        &self,
        owners: &SemanticOwnerGraph,
        query_owner: SemanticOwnerId,
        entry: &NamespaceSymbolEntry,
    ) -> bool {
        let Some(query_package) = owners.node(query_owner).map(|node| node.package) else {
            return false;
        };
        let Some(declaration_package) = owners
            .node(entry.declaration_owner)
            .map(|node| node.package)
        else {
            return false;
        };
        query_package == declaration_package
            && owners.is_ancestor_or_self(entry.declaration_owner, query_owner)
    }

    /// Resolve one unqualified lexical name.
    ///
    /// This is intentionally distinct from explicit path traversal. A
    /// non-export declaration is inherited by descendant owners in the same
    /// package, but not by an unrelated sibling merely because the spelling
    /// appears in the same package's `FullNameView`.
    pub fn resolve_lexical_symbol(
        &self,
        owners: &SemanticOwnerGraph,
        query_owner: SemanticOwnerId,
        node: OwnerNamespaceNodeId,
        name: &str,
    ) -> Result<NamespaceLookupResult, NamespaceLookupFailure> {
        let candidates = self
            .node(node)
            .and_then(|node| node.symbols.get(name))
            .ok_or(NamespaceLookupFailure::Unresolved)?;
        let candidate_identities = candidates
            .iter()
            .filter(|entry| self.can_lexically_see(owners, query_owner, entry))
            .map(|entry| entry.identity)
            .collect::<Vec<_>>();
        if candidate_identities.is_empty() {
            return Err(NamespaceLookupFailure::Unresolved);
        }
        Ok(NamespaceLookupResult {
            candidate_identities,
            view: NamespaceNameView::FullNameView,
            crossed_package_boundary: false,
        })
    }

    /// Resolve language-source navigation components.
    ///
    /// Source components are stored inner-to-outer, unlike the containment
    /// graph traversal order. This entry point performs the one mechanical
    /// reversal before using the graph resolver.
    pub fn resolve_inner_to_outer(
        &self,
        owners: &SemanticOwnerGraph,
        query_owner: SemanticOwnerId,
        start: OwnerNamespaceNodeId,
        source_components: &[String],
    ) -> Result<NamespaceLookupResult, NamespaceLookupFailure> {
        let outer_to_inner = source_components.iter().rev().cloned().collect::<Vec<_>>();
        self.resolve_outer_to_inner(owners, query_owner, start, &outer_to_inner)
    }

    /// Resolve an outer-to-inner path.
    ///
    /// Language source navigation is stored inner-to-outer; callers should
    /// reverse those components before calling this graph-level routine.
    pub fn resolve_outer_to_inner(
        &self,
        owners: &SemanticOwnerGraph,
        query_owner: SemanticOwnerId,
        start: OwnerNamespaceNodeId,
        components: &[String],
    ) -> Result<NamespaceLookupResult, NamespaceLookupFailure> {
        if components.is_empty() {
            return Err(NamespaceLookupFailure::Unresolved);
        }
        let query_package = owners
            .node(query_owner)
            .map(|node| node.package)
            .ok_or(NamespaceLookupFailure::PackageBoundaryViolation)?;
        let mut current = start;
        let start_package = self.validated_package_of(owners, start)?;
        let mut crossed_package_boundary = start_package != query_package;
        let mut path_is_public = true;

        for (index, component) in components.iter().enumerate() {
            let terminal = index + 1 == components.len();
            if !terminal {
                let child = self
                    .node(current)
                    .and_then(|node| node.children.get(component))
                    .copied()
                    .ok_or(NamespaceLookupFailure::Unresolved)?;
                let child_node = self.node(child).ok_or(NamespaceLookupFailure::Unresolved)?;
                path_is_public &= child_node.visibility != NamespaceVisibility::Private;
                current = if let Some(target) = child_node.mount_target {
                    if self.node(target).is_none() {
                        return Err(NamespaceLookupFailure::MountTargetMissing);
                    }
                    target
                } else {
                    child
                };
                let current_package = self.validated_package_of(owners, current)?;
                crossed_package_boundary |= current_package != query_package;
                continue;
            }

            let candidates = self
                .node(current)
                .and_then(|node| node.symbols.get(component))
                .ok_or(NamespaceLookupFailure::Unresolved)?;
            if crossed_package_boundary {
                if !path_is_public {
                    return Err(NamespaceLookupFailure::PrivatePath);
                }
                let publicly_reachable = candidates
                    .iter()
                    .filter(|entry| entry.namespace_visibility != NamespaceVisibility::Private)
                    .collect::<Vec<_>>();
                if publicly_reachable.is_empty() {
                    return Err(NamespaceLookupFailure::PrivatePath);
                }
                if publicly_reachable
                    .iter()
                    .all(|entry| !entry.in_export_retention_closure)
                {
                    return Err(NamespaceLookupFailure::NotInExportRetentionDomain);
                }
                let candidate_identities = publicly_reachable
                    .into_iter()
                    .filter(|entry| {
                        entry.in_export_retention_closure && entry.has_external_candidate_view
                    })
                    .map(|entry| entry.identity)
                    .collect::<Vec<_>>();
                if candidate_identities.is_empty() {
                    return Err(NamespaceLookupFailure::NoExternallyEligibleCandidate);
                }
                return Ok(NamespaceLookupResult {
                    candidate_identities,
                    view: NamespaceNameView::ExternalNameView,
                    crossed_package_boundary,
                });
            }

            // An explicit same-package path consumes the complete internal
            // name view. Lexical inheritance is a separate query above and
            // must not be reused as an explicit-navigation admission rule.
            let candidate_identities = candidates
                .iter()
                .map(|entry| entry.identity)
                .collect::<Vec<_>>();
            if candidate_identities.is_empty() {
                return Err(NamespaceLookupFailure::Unresolved);
            }
            return Ok(NamespaceLookupResult {
                candidate_identities,
                view: NamespaceNameView::FullNameView,
                crossed_package_boundary,
            });
        }

        Err(NamespaceLookupFailure::Unresolved)
    }

    pub fn default_extraction_view(
        &self,
        node: OwnerNamespaceNodeId,
    ) -> BTreeMap<String, Vec<SemanticSymbolIdentity>> {
        self.node(node)
            .map(|node| {
                node.symbols
                    .iter()
                    .filter_map(|(name, entries)| {
                        let visible = entries
                            .iter()
                            .filter(|entry| {
                                entry.extraction_visibility != ExtractionMemberVisibility::Private
                            })
                            .map(|entry| entry.identity)
                            .collect::<Vec<_>>();
                        (!visible.is_empty()).then(|| (name.clone(), visible))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}
