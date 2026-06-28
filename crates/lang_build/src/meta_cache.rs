//! Minimal in-memory meta instance cache.
//!
//! Stores `MetaInvocationValue` keyed by `MetaInstanceKey`. Does **not** store
//! `NamespaceDelta`, `MetaExpansionResult`, declared symbols, or binding names.
//!
//! ## Separation of concerns
//!
//! The cache stores only pure invocation results. Declaration binding
//! (`bind_meta_invocation_value_result`) remains outside the cache — duplicate
//! invocation material can be reused, but each distinct binding still installs
//! its own declared symbol via `NamespaceDelta`.

use std::collections::BTreeMap;

use crate::{meta_invocation::MetaInvocationValue, meta_key::MetaInstanceKey, model::Provenance};

/// Cached meta invocation entry.
#[derive(Clone, Debug)]
pub struct CachedMetaInstance {
    pub key: MetaInstanceKey,
    pub result: MetaInvocationValue,
    pub provenance: Provenance,
}

/// In-memory cache of meta invocation results.
///
/// The cache is an explicit object — it is **not** a global singleton.
/// Callers that want caching must pass a `&mut MetaInstanceCache`.
#[derive(Clone, Debug, Default)]
pub struct MetaInstanceCache {
    entries: BTreeMap<MetaInstanceKey, CachedMetaInstance>,
}

impl MetaInstanceCache {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Look up a cached invocation value by key.
    pub fn lookup(&self, key: &MetaInstanceKey) -> Option<&CachedMetaInstance> {
        self.entries.get(key)
    }

    /// Insert an invocation value into the cache.
    pub fn insert(
        &mut self,
        key: MetaInstanceKey,
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
