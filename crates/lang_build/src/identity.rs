//! Distinct identity coordinates used by the semantic world.
//!
//! Distinct lookup, value, callable, and residency identities used by the
//! semantic model. No identity in this module can be reconstructed from
//! another coordinate.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeValueId(pub u64);

impl TypeValueId {
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

/// Snapshot-local identity of a semantic value.
///
/// This is deliberately distinct from [`TypeValueId`], [`PlaceId`], and a
/// graph Symbol identity.
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
