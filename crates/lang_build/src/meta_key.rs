//! Parent-neutral meta invocation material key.
//!
//! `MetaInstanceMaterialKey = SelectedCallableIdentity × CanonicalArgumentProductAddr`
//! stores its structural coordinates and defines equality/ordering directly on
//! them.

use crate::{
    canonical_value::CanonicalValueAddr, identity::SelectedCallableIdentity, model::Provenance,
    semantic_owner::SemanticOwnerId,
};

/// Complete instance identity: stable parent owner, selected callable, and
/// canonical whole argument Product. Result/body material is not an axis.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MetaInstanceRootKey {
    pub parent_owner: SemanticOwnerId,
    pub material: MetaInstanceMaterialKey,
}

/// Parent-neutral structural key for replayable meta invocation material.
///
/// ## Equality and ordering
///
/// Equality and ordering are defined DIRECTLY on the structural coordinates
/// `(callable, arguments)` — never on a digest.
/// `provenance` is excluded: it is diagnostic context, not canonical
/// identity.  Graph declaration SymbolIds never enter the key: the
/// callable coordinate is the selected function object
/// VALUE identity plus its selected `()` call entry.
#[derive(Clone, Debug)]
pub struct MetaInstanceMaterialKey {
    /// Selected callable: receiver value + selected call entry.
    pub callable: SelectedCallableIdentity,
    /// Canonical address of the whole argument Product,
    /// `Addr(Product(a1..an))`.
    pub arguments: CanonicalValueAddr,
    pub provenance: Provenance,
}

impl MetaInstanceMaterialKey {
    /// Structural identity coordinates participating in Eq/Ord.
    fn coords(&self) -> (SelectedCallableIdentity, CanonicalValueAddr) {
        (self.callable, self.arguments)
    }
}

impl PartialEq for MetaInstanceMaterialKey {
    fn eq(&self, other: &Self) -> bool {
        self.coords() == other.coords()
    }
}

impl Eq for MetaInstanceMaterialKey {}

impl PartialOrd for MetaInstanceMaterialKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MetaInstanceMaterialKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.coords().cmp(&other.coords())
    }
}

/// Compute the parent-neutral material key of one meta invocation from the
/// selected callable identity and the canonical address of the whole
/// argument Product.
///
/// `MetaInstanceMaterialKey = SelectedCallableIdentity × Addr(Product(a1..an))` — this
/// single key mechanism is independent of source or builtin implementation.
/// The invocation parentheses are themselves a
/// Product value, so the arguments participate as one Product normal form
/// whose members are the per-position canonical addresses: top-level
/// argument equivalence is order-sensitive because Product identity is
/// positional, not because of any sequence encoding here.  Formal binder
/// names, source paths, body material, backing declaration SymbolIds, and
/// carrier Symbols never enter this key.  α-renaming a formal binder
/// cannot change the key; two distinct callable values under one
/// carrier Symbol always produce distinct keys.
pub fn compute_meta_instance_material_key(
    callable: SelectedCallableIdentity,
    arguments_product_addr: CanonicalValueAddr,
    provenance: Provenance,
) -> MetaInstanceMaterialKey {
    MetaInstanceMaterialKey {
        callable,
        arguments: arguments_product_addr,
        provenance,
    }
}
