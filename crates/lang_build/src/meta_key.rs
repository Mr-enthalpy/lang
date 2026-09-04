//! Parent-neutral meta invocation material key.
//!
//! `MetaInvocationMaterialKey = MetaCallableIdentity × CanonicalArgumentProductAddr`
//! stores its structural coordinates and defines equality/ordering directly on
//! them.

use crate::{
    canonical_value::CanonicalValueAddr, identity::MetaCallableIdentity, model::Provenance,
};

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
pub struct MetaInvocationMaterialKey {
    /// Selected meta callable: function object value + selected call entry.
    pub callable: MetaCallableIdentity,
    /// Canonical address of the whole argument Product,
    /// `Addr(Product(a1..an))`.
    pub arguments: CanonicalValueAddr,
    pub provenance: Provenance,
}

impl MetaInvocationMaterialKey {
    /// Structural identity coordinates participating in Eq/Ord.
    fn coords(&self) -> (MetaCallableIdentity, CanonicalValueAddr) {
        (self.callable, self.arguments)
    }
}

impl PartialEq for MetaInvocationMaterialKey {
    fn eq(&self, other: &Self) -> bool {
        self.coords() == other.coords()
    }
}

impl Eq for MetaInvocationMaterialKey {}

impl PartialOrd for MetaInvocationMaterialKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MetaInvocationMaterialKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.coords().cmp(&other.coords())
    }
}

/// Compute the parent-neutral material key of one meta invocation from the
/// selected meta callable identity and the canonical address of the whole
/// argument Product.
///
/// `MetaInvocationMaterialKey = MetaCallableIdentity × Addr(Product(a1..an))` — this
/// single key mechanism serves source-declared AND core meta callables.
/// The invocation parentheses are themselves a
/// Product value, so the arguments participate as one Product normal form
/// whose members are the per-position canonical addresses: top-level
/// argument equivalence is order-sensitive because Product identity is
/// positional, not because of any sequence encoding here.  Formal binder
/// names, source paths, body material, backing declaration SymbolIds, and
/// carrier Symbols never enter this key.  α-renaming a formal binder
/// cannot change the key; two distinct meta function values under one
/// carrier Symbol always produce distinct keys.
pub fn compute_meta_invocation_material_key(
    callable: MetaCallableIdentity,
    arguments_product_addr: CanonicalValueAddr,
    provenance: Provenance,
) -> MetaInvocationMaterialKey {
    MetaInvocationMaterialKey {
        callable,
        arguments: arguments_product_addr,
        provenance,
    }
}
