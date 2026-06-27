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
