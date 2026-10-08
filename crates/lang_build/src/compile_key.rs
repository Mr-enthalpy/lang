//! Parent-neutral compile invocation material key.
//!
//! `CompileInvocationMaterialKey = CompilePartner × CanonicalArgumentProductAddr`
//! stores its structural coordinates and defines equality/ordering directly on
//! them.

use crate::{
    canonical_value::CanonicalValueAddr, identity::CompilePartner, model::Provenance,
    semantic_owner::SemanticOwnerId,
};

/// Complete instance identity: stable parent owner, selected callable, and
/// canonical whole argument Product. Result/body material is not an axis.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompileInstanceKey {
    pub parent_owner: SemanticOwnerId,
    pub material: CompileInvocationMaterialKey,
}

/// Parent-neutral structural key for replayable compile invocation material.
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
pub struct CompileInvocationMaterialKey {
    /// Selected callable: receiver value + selected call entry.
    pub callable: CompilePartner,
    /// Canonical address of the whole argument Product,
    /// `Addr(Product(a1..an))`.
    pub arguments: CanonicalValueAddr,
    pub provenance: Provenance,
}

impl CompileInvocationMaterialKey {
    /// Structural identity coordinates participating in Eq/Ord.
    fn coords(&self) -> (CompilePartner, CanonicalValueAddr) {
        (self.callable, self.arguments)
    }
}

impl PartialEq for CompileInvocationMaterialKey {
    fn eq(&self, other: &Self) -> bool {
        self.coords() == other.coords()
    }
}

impl Eq for CompileInvocationMaterialKey {}

impl PartialOrd for CompileInvocationMaterialKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CompileInvocationMaterialKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.coords().cmp(&other.coords())
    }
}

/// Compute the parent-neutral material key of one compile invocation from the
/// selected callable identity and the canonical address of the whole
/// argument Product.
///
/// `CompileInvocationMaterialKey = CompilePartner × Addr(Product(a1..an))` — this
/// single key mechanism is independent of source or builtin implementation.
/// Arguments participate as one Product normal form. A bare argument layer
/// is positional; an all-named layer is unordered. Nested layers determine
/// their own order independently. Formal binder
/// names, source paths, body material, backing declaration SymbolIds, and
/// carrier Symbols never enter this key.  α-renaming a formal binder
/// cannot change the key; two distinct callable values under one
/// carrier Symbol always produce distinct keys.
pub fn compute_compile_instance_material_key(
    callable: CompilePartner,
    arguments_product_addr: CanonicalValueAddr,
    provenance: Provenance,
) -> CompileInvocationMaterialKey {
    CompileInvocationMaterialKey {
        callable,
        arguments: arguments_product_addr,
        provenance,
    }
}
