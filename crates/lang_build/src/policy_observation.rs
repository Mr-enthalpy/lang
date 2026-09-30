use crate::policy_pair::{
    ObservationHorizon, PatternComponentPolicy, PolicyMode, PolicyResultEntry, ValueComponentPolicy,
};

/// A symbol is resolved by identity/path before any horizon visibility is
/// considered. A successful result can consequently expose no readable facet
/// in the current horizon without becoming an "unresolved symbol".
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolEntry<I, V, P> {
    pub identity: I,
    pub path: String,
    pub entries: Vec<PolicyResultEntry<V, P>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolResolutionError {
    Unresolved,
}

pub fn resolve_explicit_path<'a, I, V, P>(
    symbols: &'a [SymbolEntry<I, V, P>],
    path: &str,
) -> Result<&'a SymbolEntry<I, V, P>, SymbolResolutionError> {
    symbols
        .iter()
        .find(|symbol| symbol.path == path)
        .ok_or(SymbolResolutionError::Unresolved)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FacetView<T> {
    Exposed(T),
    HiddenAtHorizon,
    Absent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExposedPolicyEntry<V, P> {
    pub value: FacetView<V>,
    pub value_policy: ValueComponentPolicy,
    pub pattern: FacetView<P>,
    pub pattern_policy: PatternComponentPolicy,
    pub mode: PolicyMode,
}

pub fn expose_policy_slice<V: Clone, P: Clone>(
    entry: &PolicyResultEntry<V, P>,
    horizon: ObservationHorizon,
) -> ExposedPolicyEntry<V, P> {
    let value = match &entry.value {
        None => FacetView::Absent,
        Some(value)
            if entry
                .view
                .pair
                .value
                .stage()
                .is_some_and(|s| s.visible_at(horizon)) =>
        {
            FacetView::Exposed(value.clone())
        }
        Some(_) => FacetView::HiddenAtHorizon,
    };
    let pattern = if entry.view.pair.pattern.stage.visible_at(horizon) {
        FacetView::Exposed(entry.pattern.clone())
    } else {
        FacetView::HiddenAtHorizon
    };
    ExposedPolicyEntry {
        value,
        value_policy: entry.view.pair.value,
        pattern,
        pattern_policy: entry.view.pair.pattern,
        mode: entry.view.mode,
    }
}

pub fn read_value<V, P>(entry: &ExposedPolicyEntry<V, P>) -> Option<&V> {
    match &entry.value {
        FacetView::Exposed(value) => Some(value),
        FacetView::HiddenAtHorizon | FacetView::Absent => None,
    }
}

pub fn read_pattern<V, P>(entry: &ExposedPolicyEntry<V, P>) -> Option<&P> {
    match &entry.pattern {
        FacetView::Exposed(pattern) => Some(pattern),
        FacetView::HiddenAtHorizon | FacetView::Absent => None,
    }
}

pub fn enumerate_value_facet<V, P>(
    entries: &[ExposedPolicyEntry<V, P>],
) -> impl Iterator<Item = &V> {
    entries.iter().filter_map(read_value)
}
