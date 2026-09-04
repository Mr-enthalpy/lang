//! Semantic ownership identity substrate.
//!
//! Semantic identity is derived from an owner graph. Source files, byte
//! offsets, display paths, and provenance never participate in equality.
//! Callable nesting and canonical meta invocation nesting use the same parent
//! relation.

use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::meta_key::MetaInvocationMaterialKey;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemanticOwnerGraphId(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemanticOwnerId {
    pub graph: SemanticOwnerGraphId,
    pub local: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalSymbolIdentity(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalCallableIdentity(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalGenerationIdentity(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemanticSymbolIdentity {
    pub owner: SemanticOwnerId,
    pub local: LocalSymbolIdentity,
}

/// Anonymous type available when a callable is materialized as a standalone
/// function object.
///
/// This type is derived from lexical/code ownership, but it is not the
/// universal type of invocation-frame slot 0. An associated `()` entry may
/// instead bind slot 0 to an independently named receiver type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnonymousCallableTypeId {
    pub callable_owner: SemanticOwnerId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CallableReceiverTypeId {
    Anonymous(AnonymousCallableTypeId),
    Named(SemanticSymbolIdentity),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CallableReceiverBindingSource {
    StandaloneAnonymousDefault,
    AssociatedCallEntry,
}

/// Separates the callable body's lexical/code owner from the type of the
/// caller object injected into invocation-frame slot 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CallableReceiverBinding {
    pub callable_owner: SemanticOwnerId,
    pub receiver_type: CallableReceiverTypeId,
    pub source: CallableReceiverBindingSource,
}

/// Persistent build-side Pattern root identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResolvedPatternRootId {
    pub owner: SemanticOwnerId,
    pub local_root: u32,
}

/// Persistent build-side hole identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResolvedHoleBinderId {
    pub root: ResolvedPatternRootId,
    pub local_binder: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnerQualificationError {
    UnmappedFrontendOwner(lang_syntax::NormSemanticOwnerId),
    ConflictingMapping {
        frontend: lang_syntax::NormSemanticOwnerId,
        existing: SemanticOwnerId,
        replacement: SemanticOwnerId,
    },
}

/// Explicit handoff from frontend-local callable owners to build-world
/// semantic owners.
///
/// A `HoleBinderId` cannot enter multi-root semantic state through this API
/// until the exact `NormSemanticOwnerId` carried by its PatternRoot has a
/// mapping. Callers therefore cannot qualify an unrelated local ordinal merely
/// by supplying an arbitrary persistent owner.
///
/// This carrier does not yet prove that the complete frontend owner tree maps
/// homomorphically into the persistent owner tree:
///
/// ```text
/// Map(Parent(x)) = Parent(Map(x))
/// ```
///
/// Persistent owner harvesting must establish that relation before multiple
/// mapped roots are admitted to one build world. Exact presence and
/// non-conflicting remapping are the only invariants enforced here.
#[derive(Clone, Debug, Default)]
pub struct SemanticOwnerQualification {
    mappings: BTreeMap<lang_syntax::NormSemanticOwnerId, SemanticOwnerId>,
}

impl SemanticOwnerQualification {
    pub fn bind(
        &mut self,
        frontend: lang_syntax::NormSemanticOwnerId,
        resolved: SemanticOwnerId,
    ) -> Result<(), OwnerQualificationError> {
        if let Some(existing) = self.mappings.get(&frontend).copied() {
            return (existing == resolved).then_some(()).ok_or(
                OwnerQualificationError::ConflictingMapping {
                    frontend,
                    existing,
                    replacement: resolved,
                },
            );
        }
        self.mappings.insert(frontend, resolved);
        Ok(())
    }

    pub fn qualify_hole(
        &self,
        frontend: lang_syntax::HoleBinderId,
    ) -> Result<ResolvedHoleBinderId, OwnerQualificationError> {
        let frontend_root = frontend.pattern_root();
        let owner = self.mappings.get(&frontend_root.owner).copied().ok_or(
            OwnerQualificationError::UnmappedFrontendOwner(frontend_root.owner),
        )?;
        Ok(ResolvedHoleBinderId {
            root: ResolvedPatternRootId {
                owner,
                local_root: frontend_root.local_root,
            },
            local_binder: frontend.local_ordinal(),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallableOwnerPlacement {
    InPlace,
    Ordinary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SemanticOwnerKind {
    PackageRoot {
        package: PackageId,
        display_name: String,
    },
    Namespace {
        local_name: String,
    },
    Callable {
        local_callable: LocalCallableIdentity,
        placement: CallableOwnerPlacement,
    },
    MetaInstance {
        material_key: MetaInvocationMaterialKey,
    },
    Generated {
        local_generation: LocalGenerationIdentity,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticOwnerNode {
    pub id: SemanticOwnerId,
    pub parent: Option<SemanticOwnerId>,
    pub package: PackageId,
    pub kind: SemanticOwnerKind,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct MetaInstanceInternKey {
    parent: SemanticOwnerId,
    material_key: MetaInvocationMaterialKey,
}

/// One build-snapshot-local semantic owner forest.
///
/// IDs are stable for the lifetime of this graph. A persistent cache may later
/// replace the numeric allocator with stable graph fingerprints without
/// changing the parent-linked identity model.
#[derive(Clone, Debug)]
pub struct SemanticOwnerGraph {
    graph_id: SemanticOwnerGraphId,
    nodes: BTreeMap<SemanticOwnerId, SemanticOwnerNode>,
    package_roots: BTreeMap<PackageId, SemanticOwnerId>,
    namespaces: BTreeMap<(SemanticOwnerId, String), SemanticOwnerId>,
    callables: BTreeMap<(SemanticOwnerId, LocalCallableIdentity), SemanticOwnerId>,
    meta_instances: BTreeMap<MetaInstanceInternKey, SemanticOwnerId>,
    generated: BTreeMap<(SemanticOwnerId, LocalGenerationIdentity), SemanticOwnerId>,
    next_owner: u64,
}

static NEXT_OWNER_GRAPH: AtomicU64 = AtomicU64::new(1);

impl Default for SemanticOwnerGraph {
    fn default() -> Self {
        Self {
            graph_id: SemanticOwnerGraphId(NEXT_OWNER_GRAPH.fetch_add(1, Ordering::Relaxed)),
            nodes: BTreeMap::new(),
            package_roots: BTreeMap::new(),
            namespaces: BTreeMap::new(),
            callables: BTreeMap::new(),
            meta_instances: BTreeMap::new(),
            generated: BTreeMap::new(),
            next_owner: 0,
        }
    }
}

impl SemanticOwnerGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn package_root(
        &mut self,
        package: PackageId,
        display_name: impl Into<String>,
    ) -> SemanticOwnerId {
        if let Some(existing) = self.package_roots.get(&package) {
            return *existing;
        }
        let owner = self.allocate(
            None,
            package,
            SemanticOwnerKind::PackageRoot {
                package,
                display_name: display_name.into(),
            },
        );
        self.package_roots.insert(package, owner);
        owner
    }

    pub fn namespace(
        &mut self,
        parent: SemanticOwnerId,
        local_name: impl Into<String>,
    ) -> SemanticOwnerId {
        let local_name = local_name.into();
        let key = (parent, local_name.clone());
        if let Some(existing) = self.namespaces.get(&key) {
            return *existing;
        }
        let package = self.package_of(parent);
        let owner = self.allocate(
            Some(parent),
            package,
            SemanticOwnerKind::Namespace { local_name },
        );
        self.namespaces.insert(key, owner);
        owner
    }

    pub fn callable(
        &mut self,
        parent: SemanticOwnerId,
        local_callable: LocalCallableIdentity,
        placement: CallableOwnerPlacement,
    ) -> SemanticOwnerId {
        let key = (parent, local_callable);
        if let Some(existing) = self.callables.get(&key).copied() {
            let existing_placement = match &self
                .node(existing)
                .expect("interned callable owner exists")
                .kind
            {
                SemanticOwnerKind::Callable { placement, .. } => *placement,
                _ => unreachable!("callable interner points to non-callable owner"),
            };
            assert_eq!(
                existing_placement, placement,
                "one local callable identity cannot have two placements"
            );
            return existing;
        }
        let package = self.package_of(parent);
        let owner = self.allocate(
            Some(parent),
            package,
            SemanticOwnerKind::Callable {
                local_callable,
                placement,
            },
        );
        self.callables.insert(key, owner);
        owner
    }

    /// Intern a canonical meta invocation below `parent`.
    ///
    /// Repeating the same selected meta callable value and canonical argument
    /// key returns the same owner. Different canonical arguments — or a
    /// different selected function value under the same carrier Symbol —
    /// produce a distinct owner.
    pub fn meta_instance(
        &mut self,
        parent: SemanticOwnerId,
        material_key: MetaInvocationMaterialKey,
    ) -> SemanticOwnerId {
        let key = MetaInstanceInternKey {
            parent,
            material_key,
        };
        if let Some(existing) = self.meta_instances.get(&key) {
            return *existing;
        }
        let package = self.package_of(parent);
        let owner = self.allocate(
            Some(parent),
            package,
            SemanticOwnerKind::MetaInstance {
                material_key: key.material_key.clone(),
            },
        );
        self.meta_instances.insert(key, owner);
        owner
    }

    pub fn generated(
        &mut self,
        parent: SemanticOwnerId,
        local_generation: LocalGenerationIdentity,
    ) -> SemanticOwnerId {
        let key = (parent, local_generation);
        if let Some(existing) = self.generated.get(&key) {
            return *existing;
        }
        let package = self.package_of(parent);
        let owner = self.allocate(
            Some(parent),
            package,
            SemanticOwnerKind::Generated { local_generation },
        );
        self.generated.insert(key, owner);
        owner
    }

    pub fn node(&self, owner: SemanticOwnerId) -> Option<&SemanticOwnerNode> {
        self.nodes.get(&owner)
    }

    pub fn package_of(&self, owner: SemanticOwnerId) -> PackageId {
        self.nodes
            .get(&owner)
            .unwrap_or_else(|| panic!("unknown semantic owner {owner:?}"))
            .package
    }

    pub fn parent(&self, owner: SemanticOwnerId) -> Option<SemanticOwnerId> {
        self.node(owner).and_then(|node| node.parent)
    }

    pub fn is_ancestor_or_self(
        &self,
        ancestor: SemanticOwnerId,
        descendant: SemanticOwnerId,
    ) -> bool {
        let mut current = Some(descendant);
        while let Some(owner) = current {
            if owner == ancestor {
                return true;
            }
            current = self.parent(owner);
        }
        false
    }

    /// Anonymous receiver type used by default if this callable is
    /// materialized as a standalone function object.
    pub fn anonymous_callable_type(
        &self,
        owner: SemanticOwnerId,
    ) -> Option<AnonymousCallableTypeId> {
        matches!(
            self.node(owner).map(|node| &node.kind),
            Some(SemanticOwnerKind::Callable { .. })
        )
        .then_some(AnonymousCallableTypeId {
            callable_owner: owner,
        })
    }

    pub fn standalone_receiver_binding(
        &self,
        owner: SemanticOwnerId,
    ) -> Option<CallableReceiverBinding> {
        Some(CallableReceiverBinding {
            callable_owner: owner,
            receiver_type: CallableReceiverTypeId::Anonymous(self.anonymous_callable_type(owner)?),
            source: CallableReceiverBindingSource::StandaloneAnonymousDefault,
        })
    }

    pub fn associated_call_entry_receiver_binding(
        &self,
        owner: SemanticOwnerId,
        receiver_type: SemanticSymbolIdentity,
    ) -> Option<CallableReceiverBinding> {
        self.anonymous_callable_type(owner)?;
        (receiver_type.owner.graph == self.graph_id).then_some(CallableReceiverBinding {
            callable_owner: owner,
            receiver_type: CallableReceiverTypeId::Named(receiver_type),
            source: CallableReceiverBindingSource::AssociatedCallEntry,
        })
    }

    /// Source-order inner-to-outer callable-local `Self` owner path.
    ///
    /// This is diagnostic material only. Semantic equality uses owner IDs and
    /// parent edges, never this string. The type facet of each `Self` is
    /// supplied by its `CallableReceiverBinding`; it is not inferred from this
    /// printable path.
    pub fn printable_self_path(&self, owner: SemanticOwnerId) -> Option<Vec<&'static str>> {
        self.anonymous_callable_type(owner)?;
        let mut components = Vec::new();
        let mut current = Some(owner);
        while let Some(id) = current {
            if matches!(
                self.node(id).map(|node| &node.kind),
                Some(SemanticOwnerKind::Callable { .. })
            ) {
                components.push("Self");
            }
            current = self.parent(id);
        }
        Some(components)
    }

    /// Exact callable-owner path in source navigation order:
    /// current/innermost first, then each enclosing callable.
    pub fn callable_owner_path(&self, owner: SemanticOwnerId) -> Option<Vec<SemanticOwnerId>> {
        self.anonymous_callable_type(owner)?;
        let mut components = Vec::new();
        let mut current = Some(owner);
        while let Some(id) = current {
            if matches!(
                self.node(id).map(|node| &node.kind),
                Some(SemanticOwnerKind::Callable { .. })
            ) {
                components.push(id);
            }
            current = self.parent(id);
        }
        Some(components)
    }

    fn allocate(
        &mut self,
        parent: Option<SemanticOwnerId>,
        package: PackageId,
        kind: SemanticOwnerKind,
    ) -> SemanticOwnerId {
        let id = SemanticOwnerId {
            graph: self.graph_id,
            local: self.next_owner,
        };
        self.next_owner = self
            .next_owner
            .checked_add(1)
            .expect("semantic owner graph exhausted u64 identity space");
        self.nodes.insert(
            id,
            SemanticOwnerNode {
                id,
                parent,
                package,
                kind,
            },
        );
        id
    }
}
