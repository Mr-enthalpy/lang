//! Facet observations of already resolved material. Name resolution belongs to
//! the name resolver and does not depend on the observation horizon.

use crate::policy_pair::{
    ObservationHorizon, PatternComponentPolicy, PolicyMode, PolicyResultEntry, ValueComponentPolicy,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FacetView<T> {
    Exposed(T),
    /// Retain malformed observation material without granting any read.
    Unavailable {
        retained: T,
        diagnostic: crate::Diagnostic,
    },
    /// Retained resolved material; retention does not authorize a facet read.
    HiddenAtHorizon(T),
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
    if let Err(diagnostic) = entry
        .view
        .pair
        .validate_rank_chain(crate::Provenance::new("Policy observation"))
    {
        return ExposedPolicyEntry {
            value: match &entry.value {
                Some(value) => FacetView::Unavailable {
                    retained: value.clone(),
                    diagnostic: diagnostic.clone(),
                },
                None => FacetView::Absent,
            },
            value_policy: entry.view.pair.value,
            pattern: FacetView::Unavailable {
                retained: entry.pattern.clone(),
                diagnostic,
            },
            pattern_policy: entry.view.pair.pattern,
            mode: entry.view.mode,
        };
    }
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
        Some(value) => FacetView::HiddenAtHorizon(value.clone()),
    };
    let pattern = if entry.view.pair.pattern.stage.visible_at(horizon) {
        FacetView::Exposed(entry.pattern.clone())
    } else {
        FacetView::HiddenAtHorizon(entry.pattern.clone())
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
        FacetView::HiddenAtHorizon(_) | FacetView::Unavailable { .. } | FacetView::Absent => None,
    }
}

pub fn read_pattern<V, P>(entry: &ExposedPolicyEntry<V, P>) -> Option<&P> {
    match &entry.pattern {
        FacetView::Exposed(pattern) => Some(pattern),
        FacetView::HiddenAtHorizon(_) | FacetView::Unavailable { .. } | FacetView::Absent => None,
    }
}

pub fn enumerate_value_facet<V, P>(
    entries: &[ExposedPolicyEntry<V, P>],
) -> impl Iterator<Item = &V> {
    entries.iter().filter_map(read_value)
}
