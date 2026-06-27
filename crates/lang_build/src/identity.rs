//! Placeholder identity types for the v0.8 construction contract.
//!
//! This module provides `TypeValueId`, `PlaceId`, `TypeValueBindingPlaceholder`,
//! and `AliasChain` as object-boundary placeholders. It does **not** implement
//! full alias resolution or type-value tracking.
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AliasChain {
    pub source_symbol: SymbolId,
    pub forwarded_target: SymbolId,
    /// When `cycle_detection_state` is `NotChecked`, `final_symbol` may only
    /// be the direct forwarded target placeholder. It must **not** be
    /// interpreted as a fully resolved final alias target.
    pub final_symbol: Option<SymbolId>,
    pub final_value: Option<TypeValueId>,
    pub final_place: Option<PlaceId>,
    pub provenance_chain: Vec<Provenance>,
    pub writable_boundary: AliasWritableBoundary,
    pub cycle_detection_state: AliasCycleDetectionState,
}

impl AliasChain {
    pub fn new(
        source_symbol: SymbolId,
        forwarded_target: SymbolId,
        provenance: Provenance,
    ) -> Self {
        Self {
            source_symbol,
            forwarded_target,
            final_symbol: Some(forwarded_target),
            final_value: None,
            final_place: None,
            provenance_chain: vec![provenance],
            writable_boundary: AliasWritableBoundary::Unknown,
            cycle_detection_state: AliasCycleDetectionState::NotChecked,
        }
    }

    pub fn query_disposition(&self, mode: AliasQueryMode) -> AliasQueryDisposition {
        match mode {
            AliasQueryMode::TypeValueEvaluation => AliasQueryDisposition::FollowValueChain,
            AliasQueryMode::CallableLookup => AliasQueryDisposition::PolicyAwareSymbolResolution,
            AliasQueryMode::InjectionPlaceTarget => AliasQueryDisposition::FollowPlaceWithBoundary,
        }
    }

    pub fn creates_fresh_writable_place(&self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AliasQueryMode {
    TypeValueEvaluation,
    CallableLookup,
    InjectionPlaceTarget,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AliasQueryDisposition {
    FollowValueChain,
    PolicyAwareSymbolResolution,
    FollowPlaceWithBoundary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AliasWritableBoundary {
    Unknown,
    ForwardTargetBoundary,
    ReadOnlyBoundary,
    WritableTargetBoundary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AliasCycleDetectionState {
    NotChecked,
    Visiting,
    Acyclic,
    CycleDetected,
}

/// Resolver-facing query surface for alias chain resolution.
///
/// Three query modes replace bare enum dispatch: type-value evaluation,
/// callable lookup, and injection-place target resolution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AliasQueryRequest {
    pub mode: AliasQueryMode,
    pub source_symbol: SymbolId,
    pub provenance: Provenance,
}

impl AliasQueryRequest {
    pub fn new(mode: AliasQueryMode, source_symbol: SymbolId, provenance: Provenance) -> Self {
        Self {
            mode,
            source_symbol,
            provenance,
        }
    }
}

/// Result of an alias chain query.
///
/// Contains a resolved disposition, optional terminal symbol/value/place, and
/// metadata about write boundaries and cycle detection. This is a placeholder
/// result object — the final resolver does not yet consume it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AliasQueryResult {
    pub disposition: AliasQueryDisposition,
    pub final_symbol: Option<SymbolId>,
    pub final_value: Option<TypeValueId>,
    pub final_place: Option<PlaceId>,
    pub writable_boundary: AliasWritableBoundary,
    pub cycle_detection_state: AliasCycleDetectionState,
}

impl AliasQueryResult {
    pub fn from_chain(chain: &AliasChain, mode: AliasQueryMode) -> Self {
        Self {
            disposition: chain.query_disposition(mode),
            final_symbol: chain.final_symbol,
            final_value: chain.final_value,
            final_place: chain.final_place,
            writable_boundary: chain.writable_boundary,
            cycle_detection_state: chain.cycle_detection_state,
        }
    }
}
