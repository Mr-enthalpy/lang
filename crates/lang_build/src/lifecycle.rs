//! Continuation-relative lifecycle semantics.
//!
//! The carriers in this module intentionally do not encode a CFG, arena, or
//! closed Color universe.  They establish the canonical relations consumed by
//! evaluation: one `SemanticContinuation`, finite lifetime observations,
//! half-open generations, cleanup-before-observation, Pre-before-action/Post-
//! after-commit, and extensible Color/access snapshots.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    CommittedSemanticAction, ContinuationIdentity, Diagnostic, Provenance, SemanticActionIdentity,
    SemanticContinuation, SemanticPosition, SemanticValueId,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LifeName(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Region {
    pub start: SemanticPosition,
    pub end: Option<SemanticPosition>,
    pub generation: u64,
}

impl Region {
    pub fn contains(self, position: SemanticPosition) -> bool {
        self.start <= position && self.end.is_none_or(|end| position < end)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NameView<T> {
    pub name: LifeName,
    pub value: T,
    pub origin: Option<LifeName>,
    pub region: Region,
}

/// Ordinary first-class observation produced by `@`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LifetimeValue {
    pub name: LifeName,
    pub observed_at: SemanticPosition,
    /// One finite origin observation. Following `.origin` performs another
    /// provider query; construction never eagerly unfolds the chain.
    pub origin: Option<LifeName>,
    pub region: Region,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LifecycleEventKind {
    Use,
    Move {
        destination: LifeName,
        effect: MoveEffect,
    },
    Drop,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LifecycleEvent {
    pub action: SemanticActionIdentity,
    pub name: LifeName,
    pub at: SemanticPosition,
    pub kind: LifecycleEventKind,
}

/// Open/extensible Color identity. Adding a Color never changes a closed Rust
/// enum because no such enum defines the vocabulary.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColorId(pub String);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ColorAlgebra {
    colors: BTreeSet<ColorId>,
    compatible: BTreeSet<(ColorId, ColorId)>,
    exclusive: BTreeSet<(ColorId, ColorId)>,
    exchangeable: BTreeSet<(ColorId, ColorId)>,
}

impl ColorAlgebra {
    pub fn register(&mut self, color: ColorId) {
        self.colors.insert(color);
    }

    pub fn declare_compatible(&mut self, left: ColorId, right: ColorId) {
        self.register(left.clone());
        self.register(right.clone());
        self.compatible.insert((left, right));
    }

    pub fn declare_exclusive(&mut self, left: ColorId, right: ColorId) {
        self.register(left.clone());
        self.register(right.clone());
        self.exclusive.insert((left, right));
    }

    pub fn declare_exchangeable(&mut self, left: ColorId, right: ColorId) {
        self.register(left.clone());
        self.register(right.clone());
        self.exchangeable.insert((left, right));
    }

    pub fn contains(&self, color: &ColorId) -> bool {
        self.colors.contains(color)
    }

    pub fn compatible(&self, left: &ColorId, right: &ColorId) -> bool {
        self.compatible.contains(&(left.clone(), right.clone()))
    }

    pub fn exclusive(&self, left: &ColorId, right: &ColorId) -> bool {
        self.exclusive.contains(&(left.clone(), right.clone()))
    }

    pub fn exchangeable(&self, left: &ColorId, right: &ColorId) -> bool {
        self.exchangeable.contains(&(left.clone(), right.clone()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccessPath(pub Vec<String>);

/// Extension interface for the still-open access-tree construction
/// algorithm. Validation depends only on this relation, never on one chosen
/// tree representation.
pub trait AccessRelationProvider {
    fn permits(&self, name: LifeName, path: &AccessPath) -> bool;
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AccessSnapshot {
    permitted: BTreeSet<(LifeName, AccessPath)>,
}

impl AccessSnapshot {
    pub fn permit(&mut self, name: LifeName, path: AccessPath) {
        self.permitted.insert((name, path));
    }
}

impl AccessRelationProvider for AccessSnapshot {
    fn permits(&self, name: LifeName, path: &AccessPath) -> bool {
        self.permitted.contains(&(name, path.clone()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LifecyclePrecondition {
    Alive(LifeName),
    ColorCompatible(ColorId, ColorId),
    ColorNotExclusive(ColorId, ColorId),
    AccessAllowed(LifeName, AccessPath),
    Reject(String),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LifecycleSnapshot {
    pub live: BTreeSet<LifeName>,
    /// Supplied instance/frontier facts, not inferred from Type or liveness.
    pub killable: BTreeSet<LifeName>,
    pub movable: BTreeSet<(LifeName, SemanticValueId)>,
    /// The selected Preserve realization's narrow ordinary proof.
    pub preserving_moves: BTreeSet<(LifeName, SemanticValueId)>,
    pub colors: ColorAlgebra,
    pub access: AccessSnapshot,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LifecycleValidationContext {
    pub snapshot: LifecycleSnapshot,
    pub preconditions: Vec<LifecyclePrecondition>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LifecycleValidationProof {
    pub checked: Vec<LifecyclePrecondition>,
}

impl LifecycleValidationContext {
    pub fn validate_pre(
        &self,
        provenance: &Provenance,
    ) -> Result<LifecycleValidationProof, Diagnostic> {
        for condition in &self.preconditions {
            let valid = match condition {
                LifecyclePrecondition::Alive(name) => self.snapshot.live.contains(name),
                LifecyclePrecondition::ColorCompatible(left, right) => {
                    self.snapshot.colors.compatible(left, right)
                }
                LifecyclePrecondition::ColorNotExclusive(left, right) => {
                    !self.snapshot.colors.exclusive(left, right)
                }
                LifecyclePrecondition::AccessAllowed(name, path) => {
                    self.snapshot.access.permits(*name, path)
                }
                LifecyclePrecondition::Reject(_) => false,
            };
            if !valid {
                return Err(Diagnostic::hard_error(
                    format!("lifecycle Pre validation failed: {condition:?}"),
                    Some(provenance.clone()),
                ));
            }
        }
        Ok(LifecycleValidationProof {
            checked: self.preconditions.clone(),
        })
    }
}

/// Fixed by the selected ordinary movement relation before any projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveEffect {
    Kill,
    Preserve,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LifecycleAction {
    Use(LifeName),
    Move {
        source: LifeName,
        destination: SemanticValueId,
        effect: MoveEffect,
    },
    Drop(LifeName),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LifecyclePost {
    pub event: LifecycleEvent,
    pub closed_region: Option<NameView<()>>,
    pub destination: Option<LifeName>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LifecycleFailure {
    CleanupScheduleNotFrozen,
    ForeignContinuation,
    UnknownValue(SemanticValueId),
    ValueAlreadyRegistered(SemanticValueId),
    DestinationAlreadyBound(SemanticValueId),
    DeadName(LifeName),
    MoveNotAuthorized,
    KillNotAuthorized(LifeName),
    PreserveNotProved,
    CleanupPointMismatch,
    CleanupPrecedencePending,
    IdentityExhausted,
    StaleOrForeignProof,
    PreRejected(String),
}

/// Read-only Pre evidence tied to this state, continuation snapshot, action and cut.
/// Fields are private; there is no fabricated or independently committing proof.
pub struct LifecyclePreProof {
    expected_state: LifecycleState,
    expected_continuation: SemanticContinuation,
    action: LifecycleAction,
    identity: SemanticActionIdentity,
    at: SemanticPosition,
    validation: LifecycleValidationProof,
}

impl LifecyclePreProof {
    pub fn validation(&self) -> &LifecycleValidationProof {
        &self.validation
    }
}

/// Lifetime projection storage. It owns neither a continuation nor a commit
/// entry point, and cannot select a movement effect or run cleanup scheduling.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LifecycleState {
    continuation: ContinuationIdentity,
    next_name: u64,
    values: BTreeMap<SemanticValueId, LifeName>,
    active: BTreeMap<LifeName, Region>,
    origins: BTreeMap<LifeName, Option<LifeName>>,
    colors: BTreeMap<LifeName, BTreeSet<ColorId>>,
    closed: Vec<NameView<()>>,
    events: Vec<LifecycleEvent>,
}

impl LifecycleState {
    pub fn new(continuation: &SemanticContinuation) -> Self {
        Self {
            continuation: continuation.identity().clone(),
            next_name: 0,
            values: BTreeMap::new(),
            active: BTreeMap::new(),
            origins: BTreeMap::new(),
            colors: BTreeMap::new(),
            closed: Vec::new(),
            events: Vec::new(),
        }
    }

    fn require_continuation(
        &self,
        continuation: &SemanticContinuation,
    ) -> Result<(), LifecycleFailure> {
        if self.continuation != *continuation.identity() {
            return Err(LifecycleFailure::ForeignContinuation);
        }
        Ok(())
    }

    /// Registration of an already established value at the shared cut. Source
    /// construction/ExtractCommit wiring remains a separate consumer gate.
    pub fn register_value(
        &mut self,
        continuation: &SemanticContinuation,
        value: SemanticValueId,
        origin: Option<LifeName>,
    ) -> Result<LifeName, LifecycleFailure> {
        self.require_continuation(continuation)?;
        if self.values.contains_key(&value) {
            return Err(LifecycleFailure::ValueAlreadyRegistered(value));
        }
        let next = self
            .next_name
            .checked_add(1)
            .ok_or(LifecycleFailure::IdentityExhausted)?;
        let name = LifeName(self.next_name);
        self.next_name = next;
        self.values.insert(value, name);
        self.origins.insert(name, origin);
        self.active.insert(
            name,
            Region {
                start: continuation.position(),
                end: None,
                generation: 0,
            },
        );
        Ok(name)
    }

    pub fn ensure_value(
        &mut self,
        continuation: &SemanticContinuation,
        value: SemanticValueId,
    ) -> Result<LifeName, LifecycleFailure> {
        self.require_continuation(continuation)?;
        match self.values.get(&value).copied() {
            Some(name) => Ok(name),
            None => self.register_value(continuation, value, None),
        }
    }

    pub fn name_of(&self, value: SemanticValueId) -> Option<LifeName> {
        self.values.get(&value).copied()
    }

    pub fn events(&self) -> &[LifecycleEvent] {
        &self.events
    }

    pub fn assign_color(&mut self, name: LifeName, color: ColorId) {
        self.colors.entry(name).or_default().insert(color);
    }

    /// Finite observation; cyclic/coinductive origin material stops at the
    /// first repeated name rather than unfolding an infinite chain.
    pub fn observed_colors(&self, name: LifeName) -> BTreeSet<ColorId> {
        let mut result = BTreeSet::new();
        let mut seen = BTreeSet::new();
        let mut cursor = Some(name);
        while let Some(current) = cursor {
            if !seen.insert(current) {
                break;
            }
            if let Some(colors) = self.colors.get(&current) {
                result.extend(colors.iter().cloned());
            }
            cursor = self.origins.get(&current).copied().flatten();
        }
        result
    }

    /// ReifyLife(NameOf(E), Pos(K)); no Place is required and @ changes no cut.
    pub fn reify_value(
        &self,
        continuation: &SemanticContinuation,
        value: SemanticValueId,
    ) -> Result<LifetimeValue, LifecycleFailure> {
        self.require_continuation(continuation)?;
        if !continuation.cleanup_is_frozen() {
            return Err(LifecycleFailure::CleanupScheduleNotFrozen);
        }
        let name = self
            .values
            .get(&value)
            .copied()
            .ok_or(LifecycleFailure::UnknownValue(value))?;
        let region = self
            .active
            .get(&name)
            .copied()
            .ok_or(LifecycleFailure::DeadName(name))?;
        Ok(LifetimeValue {
            name,
            observed_at: continuation.position(),
            origin: self.origins.get(&name).copied().flatten(),
            region,
        })
    }

    pub fn snapshot(&self, colors: ColorAlgebra, access: AccessSnapshot) -> LifecycleSnapshot {
        LifecycleSnapshot {
            live: self.active.keys().copied().collect(),
            colors,
            access,
            ..LifecycleSnapshot::default()
        }
    }

    /// Check selected action facts without advancing K or mutating any state.
    /// Killable, Movable and Preserve evidence are supplied independently by
    /// the ordinary instance/frontier consumer; Alive implies none of them.
    pub fn check_pre(
        &self,
        continuation: &SemanticContinuation,
        at: SemanticPosition,
        action: &LifecycleAction,
        validation: &LifecycleValidationContext,
        provenance: &Provenance,
    ) -> Result<LifecyclePreProof, LifecycleFailure> {
        self.require_continuation(continuation)?;
        if !continuation.cleanup_is_frozen() {
            return Err(LifecycleFailure::CleanupScheduleNotFrozen);
        }
        let proof = validation
            .validate_pre(provenance)
            .map_err(|diagnostic| LifecycleFailure::PreRejected(diagnostic.message))?;
        let name = match *action {
            LifecycleAction::Use(name) | LifecycleAction::Drop(name) => name,
            LifecycleAction::Move { source, .. } => source,
        };
        let current = self
            .active
            .get(&name)
            .ok_or(LifecycleFailure::DeadName(name))?;
        match *action {
            LifecycleAction::Move {
                source,
                destination,
                effect,
            } => {
                if self.values.contains_key(&destination) {
                    return Err(LifecycleFailure::DestinationAlreadyBound(destination));
                }
                if !validation.snapshot.movable.contains(&(source, destination)) {
                    return Err(LifecycleFailure::MoveNotAuthorized);
                }
                match effect {
                    MoveEffect::Kill => {
                        if !validation.snapshot.killable.contains(&source) {
                            return Err(LifecycleFailure::KillNotAuthorized(source));
                        }
                        self.next_name
                            .checked_add(1)
                            .ok_or(LifecycleFailure::IdentityExhausted)?;
                        current
                            .generation
                            .checked_add(1)
                            .ok_or(LifecycleFailure::IdentityExhausted)?;
                    }
                    MoveEffect::Preserve => {
                        if !validation
                            .snapshot
                            .preserving_moves
                            .contains(&(source, destination))
                        {
                            return Err(LifecycleFailure::PreserveNotProved);
                        }
                    }
                }
            }
            LifecycleAction::Drop(name) => {
                if let Some(index) = continuation
                    .cleanup()
                    .iter()
                    .position(|entry| entry.name == name)
                {
                    if continuation.cleanup()[index].at != at {
                        return Err(LifecycleFailure::CleanupPointMismatch);
                    }
                    if continuation.cleanup()[..index]
                        .iter()
                        .any(|entry| !self.cleanup_obligation_discharged(entry.name))
                    {
                        return Err(LifecycleFailure::CleanupPrecedencePending);
                    }
                }
                if !validation.snapshot.killable.contains(&name) {
                    return Err(LifecycleFailure::KillNotAuthorized(name));
                }
            }
            LifecycleAction::Use(_) => {}
        }
        Ok(LifecyclePreProof {
            expected_state: self.clone(),
            expected_continuation: continuation.clone(),
            action: action.clone(),
            identity: continuation.next_action_identity(),
            at,
            validation: proof,
        })
    }

    /// Consume one common committed-action witness. No position is allocated
    /// here. The common transaction stages every projection's Post and publishes
    /// only their joint success, so a stale/foreign proof cannot partially commit.
    pub fn apply_post(
        &mut self,
        committed: &CommittedSemanticAction<LifecycleAction>,
        proof: LifecyclePreProof,
    ) -> Result<LifecyclePost, LifecycleFailure> {
        if *self != proof.expected_state
            || *committed.continuation_before() != proof.expected_continuation
            || *committed.action() != proof.action
            || *committed.identity() != proof.identity
            || committed.position() != proof.at
        {
            return Err(LifecycleFailure::StaleOrForeignProof);
        }
        let at = committed.position();
        let name = match *committed.action() {
            LifecycleAction::Use(name) | LifecycleAction::Drop(name) => name,
            LifecycleAction::Move { source, .. } => source,
        };
        let current = self.active[&name];
        let mut closed_region = None;
        let mut destination_name = None;
        let kind = match *committed.action() {
            LifecycleAction::Use(_) => LifecycleEventKind::Use,
            LifecycleAction::Move {
                source,
                destination,
                effect,
            } => {
                let moved_name = match effect {
                    MoveEffect::Kill => {
                        let inherited_colors = self.observed_colors(source);
                        let new_name = LifeName(self.next_name);
                        self.next_name += 1;
                        let view = self.close_region(source, current, at);
                        closed_region = Some(view);
                        // Preserve deeper origin, not a new edge to the old name.
                        self.origins.insert(new_name, self.origins[&source]);
                        self.colors.insert(new_name, inherited_colors);
                        self.active.insert(
                            new_name,
                            Region {
                                start: at,
                                end: None,
                                generation: current.generation + 1,
                            },
                        );
                        new_name
                    }
                    // Transport the same surviving subject. No clone, fresh
                    // object, new generation or origin layer is manufactured.
                    MoveEffect::Preserve => source,
                };
                self.values.insert(destination, moved_name);
                destination_name = Some(moved_name);
                LifecycleEventKind::Move {
                    destination: moved_name,
                    effect,
                }
            }
            LifecycleAction::Drop(_) => {
                closed_region = Some(self.close_region(name, current, at));
                LifecycleEventKind::Drop
            }
        };
        let event = LifecycleEvent {
            action: committed.identity().clone(),
            name,
            at,
            kind,
        };
        self.events.push(event.clone());
        Ok(LifecyclePost {
            event,
            closed_region,
            destination: destination_name,
        })
    }

    fn close_region(
        &mut self,
        name: LifeName,
        current: Region,
        at: SemanticPosition,
    ) -> NameView<()> {
        self.active.remove(&name);
        let view = NameView {
            name,
            value: (),
            origin: self.origins.get(&name).copied().flatten(),
            region: Region {
                end: Some(at),
                ..current
            },
        };
        self.closed.push(view.clone());
        view
    }

    /// An already committed killing move or Drop discharges this generation's
    /// scheduled Drop. The scheduler consumes this fact; it must not add another
    /// destructor for the old generation.
    pub fn cleanup_obligation_discharged(&self, name: LifeName) -> bool {
        self.events.iter().any(|event| {
            event.name == name
                && matches!(
                    event.kind,
                    LifecycleEventKind::Drop
                        | LifecycleEventKind::Move {
                            effect: MoveEffect::Kill,
                            ..
                        }
                )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (SemanticContinuation, LifecycleState, LifeName) {
        let mut continuation = SemanticContinuation::default();
        let mut state = LifecycleState::new(&continuation);
        let name = state
            .register_value(&continuation, SemanticValueId(7), None)
            .unwrap();
        continuation.freeze_cleanup_schedule().unwrap();
        (continuation, state, name)
    }

    fn validation(state: &LifecycleState, action: &LifecycleAction) -> LifecycleValidationContext {
        let mut snapshot = state.snapshot(ColorAlgebra::default(), AccessSnapshot::default());
        match *action {
            LifecycleAction::Use(_) => {}
            LifecycleAction::Drop(name) => {
                snapshot.killable.insert(name);
            }
            LifecycleAction::Move {
                source,
                destination,
                effect,
            } => {
                snapshot.movable.insert((source, destination));
                match effect {
                    MoveEffect::Kill => {
                        snapshot.killable.insert(source);
                    }
                    MoveEffect::Preserve => {
                        snapshot.preserving_moves.insert((source, destination));
                    }
                }
            }
        }
        LifecycleValidationContext {
            snapshot,
            preconditions: Vec::new(),
        }
    }

    fn commit(
        continuation: &mut SemanticContinuation,
        state: &mut LifecycleState,
        at: u64,
        action: LifecycleAction,
    ) -> Result<LifecyclePost, crate::SemanticCommitFailure<LifecycleFailure>> {
        continuation.commit_action(
            SemanticPosition(at),
            state,
            action,
            |state, k, at, action| {
                state.check_pre(
                    k,
                    at,
                    action,
                    &validation(state, action),
                    &Provenance::new("selected action"),
                )
            },
            |state, committed, proof| state.apply_post(committed, proof),
        )
    }

    #[test]
    fn cleanup_precedes_observation_and_killing_move_shares_one_common_cut() {
        let mut k = SemanticContinuation::default();
        let mut state = LifecycleState::new(&k);
        let source_value = SemanticValueId(7);
        let source = state.register_value(&k, source_value, None).unwrap();
        assert_eq!(
            state.reify_value(&k, source_value),
            Err(LifecycleFailure::CleanupScheduleNotFrozen)
        );
        k.freeze_cleanup_schedule().unwrap();
        assert_eq!(state.reify_value(&k, source_value).unwrap().name, source);
        let destination = SemanticValueId(8);
        let post = commit(
            &mut k,
            &mut state,
            3,
            LifecycleAction::Move {
                source,
                destination,
                effect: MoveEffect::Kill,
            },
        )
        .unwrap();
        let new_name = post.destination.unwrap();
        assert_ne!(new_name, source);
        assert_eq!(
            post.closed_region.unwrap().region.end,
            Some(SemanticPosition(3))
        );
        assert_eq!(state.active[&new_name].start, SemanticPosition(3));
        assert_eq!(state.reify_value(&k, destination).unwrap().name, new_name);
        assert_eq!(
            state.reify_value(&k, source_value),
            Err(LifecycleFailure::DeadName(source))
        );
        assert_eq!(post.event.action.continuation, *k.identity());
        assert_eq!(post.event.at, k.position());
        assert!(state.cleanup_obligation_discharged(source));
        assert_eq!(
            state.events().len(),
            1,
            "a killing move adds no destructor event"
        );
        let before = state.clone();
        assert!(commit(&mut k, &mut state, 9, LifecycleAction::Drop(source)).is_err());
        assert_eq!(
            state, before,
            "the consumed generation cannot be dropped twice"
        );
        assert_eq!(k.position(), SemanticPosition(3));
    }

    #[test]
    fn preserve_move_transports_the_same_subject_without_clone_or_generation_change() {
        let (mut k, mut state, source) = fixture();
        let before_name_count = state.next_name;
        let region = state.active[&source];
        let color = ColorId("stable-subject".into());
        state.assign_color(source, color.clone());
        let destination = SemanticValueId(8);
        let post = commit(
            &mut k,
            &mut state,
            2,
            LifecycleAction::Move {
                source,
                destination,
                effect: MoveEffect::Preserve,
            },
        )
        .unwrap();
        assert_eq!(post.destination, Some(source));
        assert!(post.closed_region.is_none());
        assert_eq!(state.name_of(SemanticValueId(7)), Some(source));
        assert_eq!(state.name_of(destination), Some(source));
        assert_eq!(state.active[&source], region);
        assert_eq!(
            state.next_name, before_name_count,
            "Preserve creates no fresh lifecycle subject"
        );
        assert_eq!(state.observed_colors(source), BTreeSet::from([color]));
        assert!(!state.cleanup_obligation_discharged(source));
        assert!(matches!(
            post.event.kind,
            LifecycleEventKind::Move {
                effect: MoveEffect::Preserve,
                ..
            }
        ));
    }

    #[test]
    fn movable_killable_and_preserve_evidence_are_independent_and_effect_never_falls_back() {
        let (mut k, mut state, source) = fixture();
        let before = state.clone();
        let action = LifecycleAction::Move {
            source,
            destination: SemanticValueId(8),
            effect: MoveEffect::Kill,
        };
        let mut facts = LifecycleValidationContext::default();
        facts.snapshot.killable.insert(source);
        assert!(matches!(
            state.check_pre(
                &k,
                SemanticPosition(1),
                &action,
                &facts,
                &Provenance::new("killable alone")
            ),
            Err(LifecycleFailure::MoveNotAuthorized)
        ));
        facts.snapshot.movable.insert((source, SemanticValueId(8)));
        facts.snapshot.killable.clear();
        facts
            .snapshot
            .preserving_moves
            .insert((source, SemanticValueId(8)));
        let failed = k.commit_action(
            SemanticPosition(1),
            &mut state,
            action.clone(),
            |state, k, at, action| {
                state.check_pre(k, at, action, &facts, &Provenance::new("fixed Kill"))
            },
            |state, action, proof| state.apply_post(action, proof),
        );
        assert!(
            matches!(failed, Err(crate::SemanticCommitFailure::Pre(LifecycleFailure::KillNotAuthorized(n))) if n == source)
        );
        assert_eq!(state, before);
        assert_eq!(k.position(), SemanticPosition(0));
        let preserve = LifecycleAction::Move {
            source,
            destination: SemanticValueId(8),
            effect: MoveEffect::Preserve,
        };
        facts.snapshot.preserving_moves.clear();
        facts.snapshot.killable.insert(source);
        assert!(matches!(
            state.check_pre(
                &k,
                SemanticPosition(1),
                &preserve,
                &facts,
                &Provenance::new("Kill proof is not Preserve proof")
            ),
            Err(LifecycleFailure::PreserveNotProved)
        ));
    }

    #[test]
    fn killing_move_preserves_direct_and_inherited_colors_and_deeper_origin() {
        let (mut k, mut state, ancestor) = fixture();
        let source = state
            .register_value(&k, SemanticValueId(71), Some(ancestor))
            .unwrap();
        let ancestor_color = ColorId("ancestor".into());
        let direct_color = ColorId("direct".into());
        state.assign_color(ancestor, ancestor_color.clone());
        state.assign_color(source, direct_color.clone());
        let before = state.observed_colors(source);
        let post = commit(
            &mut k,
            &mut state,
            1,
            LifecycleAction::Move {
                source,
                destination: SemanticValueId(72),
                effect: MoveEffect::Kill,
            },
        )
        .unwrap();
        let destination = post.destination.unwrap();
        assert_eq!(state.origins[&destination], Some(ancestor));
        assert_eq!(state.observed_colors(destination), before);
        assert_eq!(before, BTreeSet::from([ancestor_color, direct_color]));
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct ProjectionState {
        life: LifecycleState,
        type_fact: u64,
        type_position: SemanticPosition,
        type_action: Option<SemanticActionIdentity>,
    }

    #[test]
    fn every_projection_pre_precedes_one_shared_commit_and_posts() {
        let (mut k, life, name) = fixture();
        let mut state = ProjectionState {
            life,
            type_fact: 0,
            type_position: SemanticPosition(0),
            type_action: None,
        };
        let before = state.clone();
        let failed = k.commit_action(
            SemanticPosition(1),
            &mut state,
            LifecycleAction::Use(name),
            |state, k, at, action| {
                let _life_proof = state.life.check_pre(
                    k,
                    at,
                    action,
                    &validation(&state.life, action),
                    &Provenance::new("life passes"),
                )?;
                Err::<LifecyclePreProof, _>(LifecycleFailure::PreRejected("type Pre fails".into()))
            },
            |_: &mut ProjectionState, _, _| -> Result<(), LifecycleFailure> {
                panic!("Post cannot run after another projection's Pre failure")
            },
        );
        assert!(matches!(failed, Err(crate::SemanticCommitFailure::Pre(_))));
        assert_eq!(state, before);
        assert_eq!(k.position(), SemanticPosition(0));
        let post = k
            .commit_action(
                SemanticPosition(4),
                &mut state,
                LifecycleAction::Use(name),
                |state, k, at, action| {
                    state.life.check_pre(
                        k,
                        at,
                        action,
                        &validation(&state.life, action),
                        &Provenance::new("joint Pre"),
                    )
                },
                |state, action, proof| {
                    state.type_fact += 1;
                    state.type_position = action.position();
                    state.type_action = Some(action.identity().clone());
                    state.life.apply_post(action, proof)
                },
            )
            .unwrap();
        assert_eq!(state.type_position, post.event.at);
        assert_eq!(state.type_action.as_ref(), Some(&post.event.action));
        assert_eq!(k.position(), SemanticPosition(4));
        assert_eq!(state.life.events().len(), 1);
    }

    #[test]
    fn stale_foreign_and_mismatched_proofs_publish_no_projection_state() {
        let (mut k, mut state, name) = fixture();
        let action = LifecycleAction::Use(name);
        let proof = state
            .check_pre(
                &k,
                SemanticPosition(1),
                &action,
                &validation(&state, &action),
                &Provenance::new("old proof"),
            )
            .unwrap();
        commit(&mut k, &mut state, 1, action.clone()).unwrap();
        let before = state.clone();
        let failed = k.commit_action(
            SemanticPosition(1),
            &mut state,
            action.clone(),
            |_, _, _, _| Ok::<_, LifecycleFailure>(proof),
            |state, action, proof| state.apply_post(action, proof),
        );
        assert!(matches!(
            failed,
            Err(crate::SemanticCommitFailure::Projection(
                LifecycleFailure::StaleOrForeignProof
            ))
        ));
        assert_eq!(state, before);
        let mut foreign_k = SemanticContinuation::default();
        foreign_k.freeze_cleanup_schedule().unwrap();
        assert!(matches!(
            state.check_pre(
                &foreign_k,
                SemanticPosition(1),
                &action,
                &validation(&state, &action),
                &Provenance::new("other K")
            ),
            Err(LifecycleFailure::ForeignContinuation)
        ));
        let proof = state
            .check_pre(
                &k,
                SemanticPosition(2),
                &action,
                &validation(&state, &action),
                &Provenance::new("Use proof"),
            )
            .unwrap();
        let failed = k.commit_action(
            SemanticPosition(2),
            &mut state,
            LifecycleAction::Drop(name),
            |_, _, _, _| Ok::<_, LifecycleFailure>(proof),
            |state, action, proof| state.apply_post(action, proof),
        );
        assert!(matches!(
            failed,
            Err(crate::SemanticCommitFailure::Projection(_))
        ));
        assert_eq!(state, before);
        assert_eq!(k.position(), SemanticPosition(1));
    }

    #[test]
    fn proof_cannot_cross_different_frozen_cleanup_snapshots_of_the_same_continuation() {
        let mut k = SemanticContinuation::default();
        let mut state = LifecycleState::new(&k);
        let a = state.register_value(&k, SemanticValueId(1), None).unwrap();
        let b = state.register_value(&k, SemanticValueId(2), None).unwrap();
        let mut other_snapshot = k.clone();
        for (continuation, declarations) in [
            (&mut k, [(a, 1), (b, 2)]),
            (&mut other_snapshot, [(a, 2), (b, 1)]),
        ] {
            for (name, declaration_order) in declarations {
                continuation
                    .place_cleanup(crate::CleanupPlacement {
                        name,
                        at: SemanticPosition(3),
                        declaration_order,
                    })
                    .unwrap();
            }
            continuation.freeze_cleanup_schedule().unwrap();
        }
        let action = LifecycleAction::Drop(b);
        let facts = validation(&state, &action);
        let proof = state
            .check_pre(
                &k,
                SemanticPosition(3),
                &action,
                &facts,
                &Provenance::new("b precedes a in this frozen snapshot"),
            )
            .unwrap();
        let before_state = state.clone();
        let before_continuation = other_snapshot.clone();
        let result = other_snapshot.commit_action(
            SemanticPosition(3),
            &mut state,
            action,
            |_, _, _, _| Ok::<_, LifecycleFailure>(proof),
            |state, action, proof| state.apply_post(action, proof),
        );
        assert!(matches!(
            result,
            Err(crate::SemanticCommitFailure::Projection(
                LifecycleFailure::StaleOrForeignProof
            ))
        ));
        assert_eq!(state, before_state);
        assert_eq!(other_snapshot, before_continuation);
        assert_eq!(other_snapshot.cleanup()[0].name, a);
    }

    #[test]
    fn rejected_lifecycle_pre_keeps_state_and_color_vocabulary_extensible() {
        let (mut k, mut state, name) = fixture();
        let before = state.clone();
        let mut facts = validation(&state, &LifecycleAction::Drop(name));
        facts
            .preconditions
            .push(LifecyclePrecondition::Reject("no authority".into()));
        let failed = k.commit_action(
            SemanticPosition(1),
            &mut state,
            LifecycleAction::Drop(name),
            |state, k, at, action| {
                state.check_pre(k, at, action, &facts, &Provenance::new("rejected Drop"))
            },
            |state, action, proof| state.apply_post(action, proof),
        );
        assert!(matches!(failed, Err(crate::SemanticCommitFailure::Pre(_))));
        assert_eq!(state, before);
        assert_eq!(k.position(), SemanticPosition(0));
        let mut colors = ColorAlgebra::default();
        let future = ColorId("project-defined/future-color".into());
        colors.register(future.clone());
        assert!(colors.contains(&future));
    }

    #[test]
    fn fixed_cleanup_sequence_commits_only_drops_at_the_fixed_cut() {
        let mut k = SemanticContinuation::default();
        let mut state = LifecycleState::new(&k);
        let a = state.register_value(&k, SemanticValueId(1), None).unwrap();
        let b = state.register_value(&k, SemanticValueId(2), None).unwrap();
        let c = state.register_value(&k, SemanticValueId(3), None).unwrap();
        for (name, declaration_order) in [(b, 2), (a, 1), (c, 3)] {
            k.place_cleanup(crate::CleanupPlacement {
                name,
                declaration_order,
                at: SemanticPosition(9),
            })
            .unwrap();
        }
        k.freeze_cleanup_schedule().unwrap();
        assert_eq!(
            k.cleanup()
                .iter()
                .map(|entry| entry.name)
                .collect::<Vec<_>>(),
            vec![c, b, a]
        );
        let before = state.clone();
        assert!(matches!(
            commit(&mut k, &mut state, 8, LifecycleAction::Drop(c)),
            Err(crate::SemanticCommitFailure::Pre(
                LifecycleFailure::CleanupPointMismatch
            ))
        ));
        assert!(matches!(
            commit(&mut k, &mut state, 9, LifecycleAction::Drop(a)),
            Err(crate::SemanticCommitFailure::Pre(
                LifecycleFailure::CleanupPrecedencePending
            ))
        ));
        assert_eq!(state, before);
        let mut identities = Vec::new();
        for name in [c, b, a] {
            let post = commit(&mut k, &mut state, 9, LifecycleAction::Drop(name)).unwrap();
            assert_eq!(post.event.kind, LifecycleEventKind::Drop);
            assert_eq!(post.event.at, SemanticPosition(9));
            identities.push(post.event.action);
        }
        assert_ne!(identities[0], identities[1]);
        assert_ne!(identities[1], identities[2]);
        assert_eq!(k.cleanup()[0].at, SemanticPosition(9));
    }

    #[test]
    fn equal_material_registration_does_not_merge_subjects_and_foreign_observation_fails() {
        let (k, mut state, first) = fixture();
        let second = state.register_value(&k, SemanticValueId(8), None).unwrap();
        assert_ne!(first, second);
        assert_eq!(state.ensure_value(&k, SemanticValueId(7)).unwrap(), first);
        assert_eq!(
            state.register_value(&k, SemanticValueId(7), None),
            Err(LifecycleFailure::ValueAlreadyRegistered(SemanticValueId(7)))
        );
        let foreign = SemanticContinuation::default();
        assert_eq!(
            state.reify_value(&foreign, SemanticValueId(7)),
            Err(LifecycleFailure::ForeignContinuation)
        );
    }

    #[test]
    fn killing_move_discharges_scheduled_drop_without_a_second_destructor() {
        let mut k = SemanticContinuation::default();
        let mut state = LifecycleState::new(&k);
        let source = state.register_value(&k, SemanticValueId(1), None).unwrap();
        let other = state.register_value(&k, SemanticValueId(2), None).unwrap();
        for (name, declaration_order) in [(source, 2), (other, 1)] {
            k.place_cleanup(crate::CleanupPlacement {
                name,
                declaration_order,
                at: SemanticPosition(9),
            })
            .unwrap();
        }
        k.freeze_cleanup_schedule().unwrap();
        commit(
            &mut k,
            &mut state,
            1,
            LifecycleAction::Move {
                source,
                destination: SemanticValueId(3),
                effect: MoveEffect::Kill,
            },
        )
        .unwrap();
        assert!(state.cleanup_obligation_discharged(source));
        // The scheduler transports the fixed sequence, consuming discharge
        // facts instead of fabricating a separate Cleanup action.
        for placement in k.cleanup().to_vec() {
            if !state.cleanup_obligation_discharged(placement.name) {
                commit(
                    &mut k,
                    &mut state,
                    placement.at.0,
                    LifecycleAction::Drop(placement.name),
                )
                .unwrap();
            }
        }
        assert_eq!(state.events().len(), 2);
        assert_eq!(state.events()[1].name, other);
        assert_eq!(state.events()[1].kind, LifecycleEventKind::Drop);
        assert!(
            state.reify_value(&k, SemanticValueId(3)).is_ok(),
            "the transferred generation remains independent"
        );
    }

    #[test]
    fn proof_requires_current_state_and_cut_and_preserves_origins_finitely() {
        let (mut k, mut state, name) = fixture();
        let action = LifecycleAction::Use(name);
        let proof = state
            .check_pre(
                &k,
                SemanticPosition(2),
                &action,
                &validation(&state, &action),
                &Provenance::new("at two"),
            )
            .unwrap();
        state.assign_color(name, ColorId("new-fact".into()));
        let before = state.clone();
        let failed = k.commit_action(
            SemanticPosition(2),
            &mut state,
            action.clone(),
            |_, _, _, _| Ok::<_, LifecycleFailure>(proof),
            |state, action, proof| state.apply_post(action, proof),
        );
        assert!(matches!(
            failed,
            Err(crate::SemanticCommitFailure::Projection(_))
        ));
        assert_eq!(state, before);
        let proof = state
            .check_pre(
                &k,
                SemanticPosition(2),
                &action,
                &validation(&state, &action),
                &Provenance::new("different cut"),
            )
            .unwrap();
        let failed = k.commit_action(
            SemanticPosition(3),
            &mut state,
            action,
            |_, _, _, _| Ok::<_, LifecycleFailure>(proof),
            |state, action, proof| state.apply_post(action, proof),
        );
        assert!(matches!(
            failed,
            Err(crate::SemanticCommitFailure::Projection(_))
        ));
        assert_eq!(state, before);
        assert_eq!(k.position(), SemanticPosition(0));
        state.origins.insert(name, Some(name));
        assert_eq!(
            state.observed_colors(name),
            BTreeSet::from([ColorId("new-fact".into())])
        );
    }

    #[test]
    fn color_rows_are_directed_explicit_and_relation_local() {
        let a = ColorId("a".into());
        let b = ColorId("b".into());
        let mut colors = ColorAlgebra::default();

        colors.declare_compatible(a.clone(), b.clone());
        assert!(colors.compatible(&a, &b));
        assert!(!colors.compatible(&b, &a));
        assert!(!colors.compatible(&a, &a));
        assert!(!colors.exclusive(&a, &b));
        assert!(!colors.exchangeable(&a, &b));

        colors.declare_compatible(a.clone(), a.clone());
        assert!(colors.compatible(&a, &a), "an explicit self row is valid");

        colors.declare_exclusive(b.clone(), a.clone());
        assert!(colors.exclusive(&b, &a));
        assert!(!colors.exclusive(&a, &b));
        assert!(colors.compatible(&a, &b));
        assert!(!colors.exchangeable(&b, &a));

        colors.declare_exchangeable(a.clone(), b.clone());
        assert!(colors.exchangeable(&a, &b));
        assert!(!colors.exchangeable(&b, &a));
        assert!(!colors.exclusive(&a, &b));
    }

    #[test]
    fn lifecycle_pre_consumes_color_rows_in_the_written_direction() {
        let a = ColorId("a".into());
        let b = ColorId("b".into());
        let mut colors = ColorAlgebra::default();
        colors.declare_compatible(a.clone(), b.clone());
        colors.declare_exclusive(b.clone(), a.clone());

        let forward = LifecycleValidationContext {
            snapshot: LifecycleSnapshot {
                colors: colors.clone(),
                ..LifecycleSnapshot::default()
            },
            preconditions: vec![
                LifecyclePrecondition::ColorCompatible(a.clone(), b.clone()),
                LifecyclePrecondition::ColorNotExclusive(a.clone(), b.clone()),
            ],
        };
        assert!(forward.validate_pre(&Provenance::new("a to b")).is_ok());

        let reverse_compatible = LifecycleValidationContext {
            snapshot: LifecycleSnapshot {
                colors: colors.clone(),
                ..LifecycleSnapshot::default()
            },
            preconditions: vec![LifecyclePrecondition::ColorCompatible(b.clone(), a.clone())],
        };
        assert!(reverse_compatible
            .validate_pre(&Provenance::new("b to a compatibility"))
            .is_err());

        let reverse_not_exclusive = LifecycleValidationContext {
            snapshot: LifecycleSnapshot {
                colors,
                ..LifecycleSnapshot::default()
            },
            preconditions: vec![LifecyclePrecondition::ColorNotExclusive(b, a)],
        };
        assert!(reverse_not_exclusive
            .validate_pre(&Provenance::new("b to a exclusion"))
            .is_err());
    }
}
