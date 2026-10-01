//! Distinct identity coordinates used by the semantic world.
//!
//! Distinct lookup, value, callable, and residency identities used by the
//! semantic model. No identity in this module can be reconstructed from
//! another coordinate.

use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TYPE_LOOKUP_INDEX: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeValueId(u64);

/// Allocates opaque Core lookup indices.
///
/// The allocator exposes no numeric representation and accepts no identity
/// coordinate as input. A returned index acquires semantic meaning only when
/// its owning type registry installs the corresponding Core observation.
#[derive(Clone, Copy, Debug, Default)]
pub struct TypeLookupIndexAllocator;

impl TypeLookupIndexAllocator {
    pub const fn new() -> Self {
        Self
    }

    pub fn allocate(&mut self) -> TypeValueId {
        let index = NEXT_TYPE_LOOKUP_INDEX
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .expect("type lookup index space exhausted");
        assert_ne!(index, 0, "type lookup index zero is reserved");
        TypeValueId(index)
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

/// Identity of the selected ordinary callable and implementation pair.
///
/// Meta instance roots are keyed by the selected callable **value** plus the
/// selected `()` call entry, independently of a source name binding. Distinct
/// callable values produce distinct instance roots, and because the model allows
/// one function object Pattern to expose several `()` entries — two distinct
/// call entries under one function value are distinct selected implementations.
/// Formal binder names, source paths, body material, and provenance never
/// participate in this identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SelectedCallableIdentity {
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
