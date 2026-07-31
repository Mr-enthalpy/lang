//! Minimal in-memory meta instance cache.
//!
//! Stores replayable `MetaInvocationValue` material keyed by a compatibility
//! candidate digest (`CanonicalFingerprint`). The digest is an opaque cache
//! key and defines no
//! semantic identity — the canonical structural `MetaInstanceKey` never
//! enters this cache.
//! Does **not** store `NamespaceDelta`, `MetaExpansionResult`, declared
//! symbols, binding names, or concrete registry-backed `PatternHeadId`
//! material.
//!
//! ## Separation of concerns
//!
//! The cache stores only pure invocation material. Declaration binding
//! (`bind_meta_invocation_value_result`) remains outside the cache — duplicate
//! invocation material can be reused, but each distinct binding still installs
//! its own declared symbol via `NamespaceDelta`. Values that require
//! `TypeMaterializationState` are rehydrated in the caller's current state on
//! cache hit.

use std::collections::BTreeMap;

use crate::{
    meta_invocation::MetaInvocationValue, meta_key::CanonicalFingerprint, model::Provenance,
};

/// Cached meta invocation entry.
#[derive(Clone, Debug)]
pub struct CachedMetaInstance {
    pub key: CanonicalFingerprint,
    pub result: MetaInvocationValue,
    pub provenance: Provenance,
}

/// In-memory cache of meta invocation results.
///
/// The cache is an explicit object — it is **not** a global singleton.
/// Callers that want caching must pass a `&mut MetaInstanceCache`.
#[derive(Clone, Debug, Default)]
pub struct MetaInstanceCache {
    entries: BTreeMap<CanonicalFingerprint, CachedMetaInstance>,
}

impl MetaInstanceCache {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Look up a cached invocation value by key.
    pub fn lookup(&self, key: &CanonicalFingerprint) -> Option<&CachedMetaInstance> {
        self.entries.get(key)
    }

    /// Insert an invocation value into the cache.
    pub fn insert(
        &mut self,
        key: CanonicalFingerprint,
        result: MetaInvocationValue,
        provenance: Provenance,
    ) {
        self.entries.insert(
            key.clone(),
            CachedMetaInstance {
                key,
                result,
                provenance,
            },
        );
    }

    /// Number of cached entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
