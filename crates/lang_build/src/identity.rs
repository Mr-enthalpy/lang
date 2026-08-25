//! Placeholder identity types for the v0.8 construction contract.
//!
//! This module provides provisional lookup and residency identities.  The
//! former alias-query placeholders were removed when declaration aliases lost
//! semantic authority; alias syntax remains a frontend-preserved shape only.
//!
//! The current implementation boundary lives in `lang_build::identity`,
//! `lang_build::product_shape`, and `lang_build::meta_candidate`. These are
//! substrate boundaries, not full implementations of the future systems.

use crate::model::{Provenance, SymbolId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeValueId(pub u64);

impl TypeValueId {
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

/// Snapshot-local identity of a semantic value.
///
/// This is deliberately distinct from [`TypeValueId`], [`PlaceId`], and
/// `SymbolId`.  The current transition slice uses it to preserve the identity
/// of the source value across candidate preparation without pretending that a
/// value is its type, binding place, or declaring symbol.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemanticValueId(pub u64);

impl SemanticValueId {
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

/// Identity of the *selected callable* behind one meta invocation.
///
/// Meta instance roots are keyed by the selected callable **value** plus the
/// selected `()` call entry, never by the carrier Symbol that hosts the
/// overload cluster: two distinct meta function values under one Symbol must
/// produce distinct instance roots, and — because the object model allows
/// one function object Pattern to expose several `()` entries — two distinct
/// call entries under one function value are two distinct meta callables.
/// Formal binder names, source paths, body material, and provenance never
/// participate in this identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MetaCallableIdentity {
    pub selected_function_value: SemanticValueId,
    pub selected_call_entry: SemanticValueId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlaceId(pub u64);

impl PlaceId {
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeValueBindingPlaceholder {
    pub symbol: SymbolId,
    pub place: PlaceId,
    pub type_value: TypeValueId,
    pub provenance: Provenance,
}

impl TypeValueBindingPlaceholder {
    pub fn new(
        symbol: SymbolId,
        place: PlaceId,
        type_value: TypeValueId,
        provenance: Provenance,
    ) -> Self {
        Self {
            symbol,
            place,
            type_value,
            provenance,
        }
    }
}
