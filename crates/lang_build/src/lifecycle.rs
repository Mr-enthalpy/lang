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

/// Formation-origin material. Pending is an unconnected producer, not the
/// semantic proposition that the origin chain terminates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifecycleOrigin {
    Pending,
    ExplicitNone,
    Name(LifeName),
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
    CleanupPrefixNotFixed,
    ForeignContinuation,
    UnknownValue(SemanticValueId),
    ValueAlreadyRegistered(SemanticValueId),
    FormationPending(LifeName),
    FormationAfterFrontier,
    OriginPending(LifeName),
    OriginAlreadyEstablished(LifeName),
    LifecycleContinuationPending,
    DestinationAlreadyBound(SemanticValueId),
    DeadName(LifeName),
    MoveNotAuthorized,
    KillNotAuthorized(LifeName),
    PreserveNotProved,
    CleanupPointMismatch,
    CleanupPrecedencePending,
    CleanupBoundaryPending(LifeName),
    NonEndingEventAtBoundary(LifeName),
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
    pending_formations: BTreeSet<LifeName>,
    origins: BTreeMap<LifeName, LifecycleOrigin>,
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
            pending_formations: BTreeSet::new(),
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

    /// Seed a test fixture with a supplied formation cut, preserving a
    /// discovered name. This does not execute or witness a producer action.
    #[cfg(test)]
    fn admit_committed_formation_fact(
        &mut self,
        continuation: &SemanticContinuation,
        value: SemanticValueId,
        formed_at: SemanticPosition,
        origin: LifecycleOrigin,
    ) -> Result<LifeName, LifecycleFailure> {
        self.require_continuation(continuation)?;
        if formed_at > continuation.position() {
            return Err(LifecycleFailure::FormationAfterFrontier);
        }
        if let Some(name) = self.values.get(&value) {
            if !self.pending_formations.contains(name) {
                return Err(LifecycleFailure::ValueAlreadyRegistered(value));
            }
        }
        let name = self.discover_value(continuation, value)?;
        self.pending_formations.remove(&name);
        self.origins.insert(name, origin);
        self.active.insert(
            name,
            Region {
                start: formed_at,
                end: None,
                generation: 0,
            },
        );
        Ok(name)
    }

    /// Establish only the stable name map. No Alive, Region or origin fact
    /// follows from discovering a SemanticWorld value.
    pub fn discover_value(
        &mut self,
        continuation: &SemanticContinuation,
        value: SemanticValueId,
    ) -> Result<LifeName, LifecycleFailure> {
        self.require_continuation(continuation)?;
        match self.values.get(&value).copied() {
            Some(name) => Ok(name),
            None => {
                let next = self
                    .next_name
                    .checked_add(1)
                    .ok_or(LifecycleFailure::IdentityExhausted)?;
                let name = LifeName(self.next_name);
                self.next_name = next;
                self.values.insert(value, name);
                self.pending_formations.insert(name);
                self.origins.insert(name, LifecycleOrigin::Pending);
                Ok(name)
            }
        }
    }

    /// Complete a test fixture's origin fact once.
    /// None here is intentionally written termination, never an omission.
    #[cfg(test)]
    fn admit_committed_origin_fact(
        &mut self,
        continuation: &SemanticContinuation,
        name: LifeName,
        origin: Option<LifeName>,
    ) -> Result<(), LifecycleFailure> {
        self.require_continuation(continuation)?;
        if self.pending_formations.contains(&name) {
            return Err(LifecycleFailure::FormationPending(name));
        }
        match self.origins.get(&name) {
            Some(LifecycleOrigin::Pending) => {}
            Some(_) => return Err(LifecycleFailure::OriginAlreadyEstablished(name)),
            None => return Err(LifecycleFailure::OriginPending(name)),
        }
        self.origins.insert(
            name,
            origin.map_or(LifecycleOrigin::ExplicitNone, LifecycleOrigin::Name),
        );
        Ok(())
    }

    fn resolved_origin(&self, name: LifeName) -> Result<Option<LifeName>, LifecycleFailure> {
        match self.origins.get(&name) {
            Some(LifecycleOrigin::ExplicitNone) => Ok(None),
            Some(LifecycleOrigin::Name(origin)) => Ok(Some(*origin)),
            Some(LifecycleOrigin::Pending) | None => Err(LifecycleFailure::OriginPending(name)),
        }
    }

    pub fn name_of(&self, value: SemanticValueId) -> Option<LifeName> {
        self.values.get(&value).copied()
    }

    pub fn events(&self) -> &[LifecycleEvent] {
        &self.events
    }

    /// Seed a test fixture's Color fact for an active generation in K.
    /// Ended, unknown and formation-pending subjects cannot acquire facts here.
    #[cfg(test)]
    fn admit_committed_color_fact(
        &mut self,
        continuation: &SemanticContinuation,
        name: LifeName,
        color: ColorId,
    ) -> Result<(), LifecycleFailure> {
        self.require_continuation(continuation)?;
        if self.pending_formations.contains(&name) {
            return Err(LifecycleFailure::FormationPending(name));
        }
        let region = self
            .active
            .get(&name)
            .ok_or(LifecycleFailure::DeadName(name))?;
        // An old clone of K must not import facts into a projection that has
        // already observed later committed actions or this subject's birth.
        if region.start > continuation.position()
            || self.events.last().is_some_and(|event| {
                event.at > continuation.position()
                    || event.action.ordinal >= continuation.next_action_identity().ordinal
            })
        {
            return Err(LifecycleFailure::StaleOrForeignProof);
        }
        self.colors.entry(name).or_default().insert(color);
        Ok(())
    }

    /// Finite observation; cyclic/coinductive origin material stops at the
    /// first repeated name rather than unfolding an infinite chain.
    pub fn observed_colors(&self, name: LifeName) -> Result<BTreeSet<ColorId>, LifecycleFailure> {
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
            cursor = self.resolved_origin(current)?;
        }
        Ok(result)
    }

    /// ReifyLife requires the fully fixed generation continuation, including
    /// its endpoint, not just a frozen cleanup table. That consumer is pending;
    /// do not expose the operational active Region as a complete LifetimeValue.
    pub fn reify_value(
        &self,
        continuation: &SemanticContinuation,
        value: SemanticValueId,
    ) -> Result<LifetimeValue, LifecycleFailure> {
        self.require_continuation(continuation)?;
        if !continuation.cleanup_is_fixed_through(continuation.position()) {
            return Err(LifecycleFailure::CleanupPrefixNotFixed);
        }
        let name = self
            .values
            .get(&value)
            .copied()
            .ok_or(LifecycleFailure::UnknownValue(value))?;
        if self.pending_formations.contains(&name) {
            return Err(LifecycleFailure::FormationPending(name));
        }
        self.active
            .get(&name)
            .ok_or(LifecycleFailure::DeadName(name))?;
        self.resolved_origin(name)?;
        Err(LifecycleFailure::LifecycleContinuationPending)
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
        if !continuation.cleanup_is_fixed_through(at) {
            return Err(LifecycleFailure::CleanupPrefixNotFixed);
        }
        if let Some(placement) = continuation
            .cleanup()
            .iter()
            .find(|entry| entry.at < at && !self.cleanup_obligation_discharged(entry.name))
        {
            return Err(LifecycleFailure::CleanupBoundaryPending(placement.name));
        }
        let proof = validation
            .validate_pre(provenance)
            .map_err(|diagnostic| LifecycleFailure::PreRejected(diagnostic.message))?;
        let name = match *action {
            LifecycleAction::Use(name) | LifecycleAction::Drop(name) => name,
            LifecycleAction::Move { source, .. } => source,
        };
        if self.pending_formations.contains(&name) {
            return Err(LifecycleFailure::FormationPending(name));
        }
        let current = self
            .active
            .get(&name)
            .ok_or(LifecycleFailure::DeadName(name))?;
        self.resolved_origin(name)?;
        let ending = matches!(
            action,
            LifecycleAction::Drop(_)
                | LifecycleAction::Move {
                    effect: MoveEffect::Kill,
                    ..
                }
        );
        if let Some(index) = continuation
            .cleanup()
            .iter()
            .position(|entry| entry.name == name)
        {
            let placement = &continuation.cleanup()[index];
            if matches!(action, LifecycleAction::Drop(_)) && placement.at != at {
                return Err(LifecycleFailure::CleanupPointMismatch);
            }
            if placement.at == at {
                if !ending {
                    return Err(LifecycleFailure::CleanupBoundaryPending(name));
                }
                if continuation.cleanup()[..index]
                    .iter()
                    .any(|entry| !self.cleanup_obligation_discharged(entry.name))
                {
                    return Err(LifecycleFailure::CleanupPrecedencePending);
                }
            }
        }
        // A scalar half-open Region cannot later end at a cut that already
        // contains a Use or Preserve of that generation. Action ordinal does
        // not change Region membership, even for an unscheduled ending action.
        if ending
            && self.events.iter().any(|event| {
                event.name == name
                    && event.at == at
                    && matches!(
                        event.kind,
                        LifecycleEventKind::Use
                            | LifecycleEventKind::Move {
                                effect: MoveEffect::Preserve,
                                ..
                            }
                    )
            })
        {
            return Err(LifecycleFailure::NonEndingEventAtBoundary(name));
        }
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
                        self.observed_colors(source)?;
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
                if continuation
                    .pending_cleanup()
                    .iter()
                    .any(|entry| entry.name == name)
                {
                    return Err(LifecycleFailure::CleanupPointMismatch);
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
                        let inherited_colors = self
                            .observed_colors(source)
                            .expect("Pre established the finite origin/Color observation");
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
            origin: self
                .resolved_origin(name)
                .expect("Pre established the origin fact"),
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
            .admit_committed_formation_fact(
                &continuation,
                SemanticValueId(7),
                SemanticPosition(0),
                LifecycleOrigin::ExplicitNone,
            )
            .unwrap();
        continuation
            .freeze_cleanup_through(SemanticPosition(100))
            .unwrap();
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
    fn frozen_cleanup_alone_does_not_produce_an_incomplete_lifetime_value() {
        let mut k = SemanticContinuation::default();
        let mut state = LifecycleState::new(&k);
        let name = state
            .admit_committed_formation_fact(
                &k,
                SemanticValueId(1),
                SemanticPosition(0),
                LifecycleOrigin::ExplicitNone,
            )
            .unwrap();
        k.place_cleanup(crate::CleanupPlacement {
            name,
            at: SemanticPosition(9),
            declaration_order: 1,
        })
        .unwrap();
        k.freeze_cleanup_through(SemanticPosition(100)).unwrap();
        let before = state.clone();
        assert_eq!(
            state.reify_value(&k, SemanticValueId(1)),
            Err(LifecycleFailure::LifecycleContinuationPending),
            "a full generation continuation, not a frozen boolean, must establish [0,9)"
        );
        assert_eq!(state, before);
        assert_eq!(k.position(), SemanticPosition(0));
        assert_eq!(k.cleanup()[0].at, SemanticPosition(9));
    }

    #[test]
    fn killing_move_destination_can_acquire_its_own_future_cleanup() {
        let mut k = SemanticContinuation::default();
        let mut state = LifecycleState::new(&k);
        let source = state
            .admit_committed_formation_fact(
                &k,
                SemanticValueId(1),
                SemanticPosition(0),
                LifecycleOrigin::ExplicitNone,
            )
            .unwrap();
        k.place_cleanup(crate::CleanupPlacement {
            name: source,
            at: SemanticPosition(1),
            declaration_order: 1,
        })
        .unwrap();
        k.freeze_cleanup_through(SemanticPosition(1)).unwrap();
        let destination = commit(
            &mut k,
            &mut state,
            1,
            LifecycleAction::Move {
                source,
                destination: SemanticValueId(2),
                effect: MoveEffect::Kill,
            },
        )
        .unwrap()
        .destination
        .unwrap();
        assert!(state.cleanup_obligation_discharged(source));
        assert!(!state.cleanup_obligation_discharged(destination));
        k.place_cleanup(crate::CleanupPlacement {
            name: destination,
            at: SemanticPosition(5),
            declaration_order: 2,
        })
        .unwrap();
        let before = (state.clone(), k.clone());
        assert_eq!(
            commit(&mut k, &mut state, 2, LifecycleAction::Use(destination)),
            Err(crate::SemanticCommitFailure::Pre(
                LifecycleFailure::CleanupPrefixNotFixed
            ))
        );
        assert_eq!((state.clone(), k.clone()), before);
        k.freeze_cleanup_through(SemanticPosition(2)).unwrap();
        assert_eq!(
            commit(&mut k, &mut state, 2, LifecycleAction::Drop(destination)),
            Err(crate::SemanticCommitFailure::Pre(
                LifecycleFailure::CleanupPointMismatch
            ))
        );
        commit(&mut k, &mut state, 2, LifecycleAction::Use(destination)).unwrap();
        k.freeze_cleanup_through(SemanticPosition(5)).unwrap();
        let post = commit(&mut k, &mut state, 5, LifecycleAction::Drop(destination)).unwrap();
        assert_eq!(
            post.closed_region.unwrap().region.end,
            Some(SemanticPosition(5))
        );
        assert!(state.cleanup_obligation_discharged(destination));
        assert_eq!(
            state
                .events
                .iter()
                .filter(
                    |event| event.name == source && matches!(event.kind, LifecycleEventKind::Drop)
                )
                .count(),
            0
        );
    }

    #[test]
    fn late_formation_has_a_future_cleanup_without_reopening_the_prefix() {
        let mut k = SemanticContinuation::default();
        let mut state = LifecycleState::new(&k);
        k.freeze_cleanup_through(SemanticPosition(4)).unwrap();
        k.commit_action(
            SemanticPosition(4),
            &mut (),
            (),
            |_, _, _, _| Ok::<_, ()>(()),
            |_, _, _| Ok::<_, ()>(()),
        )
        .unwrap();
        let value = SemanticValueId(1);
        let name = state.discover_value(&k, value).unwrap();
        state
            .admit_committed_formation_fact(
                &k,
                value,
                SemanticPosition(1),
                LifecycleOrigin::ExplicitNone,
            )
            .unwrap();
        k.place_cleanup(crate::CleanupPlacement {
            name,
            at: SemanticPosition(6),
            declaration_order: 1,
        })
        .unwrap();
        k.freeze_cleanup_through(SemanticPosition(6)).unwrap();
        let post = commit(&mut k, &mut state, 6, LifecycleAction::Drop(name)).unwrap();
        let region = post.closed_region.unwrap().region;
        assert_eq!(region.start, SemanticPosition(1));
        assert_eq!(region.end, Some(SemanticPosition(6)));
    }

    #[test]
    fn use_and_preserve_cannot_occupy_their_scheduled_half_open_endpoint() {
        for effect in [None, Some(MoveEffect::Preserve)] {
            let mut k = SemanticContinuation::default();
            let mut state = LifecycleState::new(&k);
            let name = state
                .admit_committed_formation_fact(
                    &k,
                    SemanticValueId(1),
                    SemanticPosition(0),
                    LifecycleOrigin::ExplicitNone,
                )
                .unwrap();
            k.place_cleanup(crate::CleanupPlacement {
                name,
                at: SemanticPosition(5),
                declaration_order: 1,
            })
            .unwrap();
            k.freeze_cleanup_through(SemanticPosition(5)).unwrap();
            let action = match effect {
                None => LifecycleAction::Use(name),
                Some(effect) => LifecycleAction::Move {
                    source: name,
                    destination: SemanticValueId(2),
                    effect,
                },
            };
            let before = (state.clone(), k.clone());
            assert_eq!(
                commit(&mut k, &mut state, 5, action),
                Err(crate::SemanticCommitFailure::Pre(
                    LifecycleFailure::CleanupBoundaryPending(name)
                ))
            );
            assert_eq!((state.clone(), k.clone()), before);
            let post = commit(&mut k, &mut state, 5, LifecycleAction::Drop(name)).unwrap();
            assert_eq!(
                post.closed_region.unwrap().region.end,
                Some(SemanticPosition(5))
            );
        }
    }

    #[test]
    fn boundary_kill_obeys_the_same_fixed_order_as_scheduled_drop() {
        let mut k = SemanticContinuation::default();
        let mut state = LifecycleState::new(&k);
        let a = state
            .admit_committed_formation_fact(
                &k,
                SemanticValueId(1),
                SemanticPosition(0),
                LifecycleOrigin::ExplicitNone,
            )
            .unwrap();
        let b = state
            .admit_committed_formation_fact(
                &k,
                SemanticValueId(2),
                SemanticPosition(0),
                LifecycleOrigin::ExplicitNone,
            )
            .unwrap();
        for (name, declaration_order) in [(a, 1), (b, 2)] {
            k.place_cleanup(crate::CleanupPlacement {
                name,
                at: SemanticPosition(5),
                declaration_order,
            })
            .unwrap();
        }
        k.freeze_cleanup_through(SemanticPosition(5)).unwrap();
        let action = LifecycleAction::Move {
            source: a,
            destination: SemanticValueId(3),
            effect: MoveEffect::Kill,
        };
        let before = (state.clone(), k.clone());
        assert_eq!(
            commit(&mut k, &mut state, 5, action.clone()),
            Err(crate::SemanticCommitFailure::Pre(
                LifecycleFailure::CleanupPrecedencePending
            ))
        );
        assert_eq!((state.clone(), k.clone()), before);
        commit(&mut k, &mut state, 5, LifecycleAction::Drop(b)).unwrap();
        let destination = commit(&mut k, &mut state, 5, action)
            .unwrap()
            .destination
            .unwrap();
        assert!(state.cleanup_obligation_discharged(a));
        // The endpoint is specific to the old subject, not a ban on the cut.
        commit(&mut k, &mut state, 5, LifecycleAction::Use(destination)).unwrap();
    }

    #[test]
    fn unscheduled_ending_cannot_put_an_existing_event_outside_its_region() {
        for nonending in [false, true] {
            for kill in [false, true] {
                let (mut k, mut state, name) = fixture();
                let action = if nonending {
                    LifecycleAction::Move {
                        source: name,
                        destination: SemanticValueId(8),
                        effect: MoveEffect::Preserve,
                    }
                } else {
                    LifecycleAction::Use(name)
                };
                commit(&mut k, &mut state, 3, action).unwrap();
                let ending = if kill {
                    LifecycleAction::Move {
                        source: name,
                        destination: SemanticValueId(9),
                        effect: MoveEffect::Kill,
                    }
                } else {
                    LifecycleAction::Drop(name)
                };
                let before = (state.clone(), k.clone());
                assert_eq!(
                    commit(&mut k, &mut state, 3, ending),
                    Err(crate::SemanticCommitFailure::Pre(
                        LifecycleFailure::NonEndingEventAtBoundary(name)
                    ))
                );
                assert_eq!((state.clone(), k.clone()), before);
                commit(&mut k, &mut state, 4, LifecycleAction::Drop(name)).unwrap();
            }
        }
    }

    #[test]
    fn prefix_or_suffix_changes_invalidate_old_pre_evidence() {
        for extend_prefix in [false, true] {
            let mut k = SemanticContinuation::default();
            let mut state = LifecycleState::new(&k);
            let name = state
                .admit_committed_formation_fact(
                    &k,
                    SemanticValueId(1),
                    SemanticPosition(0),
                    LifecycleOrigin::ExplicitNone,
                )
                .unwrap();
            k.freeze_cleanup_through(SemanticPosition(1)).unwrap();
            let action = LifecycleAction::Use(name);
            let proof = state
                .check_pre(
                    &k,
                    SemanticPosition(1),
                    &action,
                    &validation(&state, &action),
                    &Provenance::new("fixed cut"),
                )
                .unwrap();
            if extend_prefix {
                k.freeze_cleanup_through(SemanticPosition(2)).unwrap();
            } else {
                k.place_cleanup(crate::CleanupPlacement {
                    name,
                    at: SemanticPosition(5),
                    declaration_order: 1,
                })
                .unwrap();
            }
            let before = (state.clone(), k.clone());
            let result = k.commit_action(
                SemanticPosition(1),
                &mut state,
                action,
                |_, _, _, _| Ok::<_, LifecycleFailure>(proof),
                |state, committed, proof| state.apply_post(committed, proof),
            );
            assert_eq!(
                result,
                Err(crate::SemanticCommitFailure::Projection(
                    LifecycleFailure::StaleOrForeignProof
                ))
            );
            assert_eq!((state.clone(), k.clone()), before);
        }
    }

    #[test]
    fn use_and_both_move_effects_cannot_cross_an_outstanding_fixed_cleanup() {
        let mut k = SemanticContinuation::default();
        let mut state = LifecycleState::new(&k);
        let source = state
            .admit_committed_formation_fact(
                &k,
                SemanticValueId(1),
                SemanticPosition(0),
                LifecycleOrigin::ExplicitNone,
            )
            .unwrap();
        let other = state
            .admit_committed_formation_fact(
                &k,
                SemanticValueId(2),
                SemanticPosition(0),
                LifecycleOrigin::ExplicitNone,
            )
            .unwrap();
        k.place_cleanup(crate::CleanupPlacement {
            name: source,
            at: SemanticPosition(5),
            declaration_order: 1,
        })
        .unwrap();
        k.freeze_cleanup_through(SemanticPosition(100)).unwrap();
        let before_state = state.clone();
        let before_k = k.clone();
        for action in [
            LifecycleAction::Use(source),
            LifecycleAction::Use(other),
            LifecycleAction::Move {
                source,
                destination: SemanticValueId(3),
                effect: MoveEffect::Kill,
            },
            LifecycleAction::Move {
                source,
                destination: SemanticValueId(3),
                effect: MoveEffect::Preserve,
            },
            LifecycleAction::Drop(other),
        ] {
            assert_eq!(
                commit(&mut k, &mut state, 10, action),
                Err(crate::SemanticCommitFailure::Pre(
                    LifecycleFailure::CleanupBoundaryPending(source)
                ))
            );
            assert_eq!(state, before_state);
            assert_eq!(
                k, before_k,
                "no event, cut, destination or ordinal was published"
            );
        }
        let drop = commit(&mut k, &mut state, 5, LifecycleAction::Drop(source)).unwrap();
        assert_eq!(drop.event.action.ordinal, 0);
        commit(&mut k, &mut state, 10, LifecycleAction::Use(other)).unwrap();
        assert_eq!(k.position(), SemanticPosition(10));
    }

    #[test]
    fn kill_discharge_allows_later_actions_but_preserve_cannot_erase_cleanup() {
        for effect in [MoveEffect::Kill, MoveEffect::Preserve] {
            let mut k = SemanticContinuation::default();
            let mut state = LifecycleState::new(&k);
            let source = state
                .admit_committed_formation_fact(
                    &k,
                    SemanticValueId(1),
                    SemanticPosition(0),
                    LifecycleOrigin::ExplicitNone,
                )
                .unwrap();
            k.place_cleanup(crate::CleanupPlacement {
                name: source,
                at: SemanticPosition(5),
                declaration_order: 1,
            })
            .unwrap();
            k.freeze_cleanup_through(SemanticPosition(100)).unwrap();
            let post = commit(
                &mut k,
                &mut state,
                1,
                LifecycleAction::Move {
                    source,
                    destination: SemanticValueId(2),
                    effect,
                },
            )
            .unwrap();
            let destination = post.destination.unwrap();
            let before_state = state.clone();
            let before_k = k.clone();
            let result = commit(&mut k, &mut state, 10, LifecycleAction::Use(destination));
            if effect == MoveEffect::Kill {
                assert!(result.is_ok());
                assert_eq!(
                    state.events().len(),
                    2,
                    "no old-generation Drop is introduced"
                );
            } else {
                assert_eq!(
                    result,
                    Err(crate::SemanticCommitFailure::Pre(
                        LifecycleFailure::CleanupBoundaryPending(source)
                    ))
                );
                assert_eq!(state, before_state);
                assert_eq!(k, before_k);
            }
        }
    }

    #[test]
    fn late_discovery_cannot_supply_formation_or_default_origin_facts() {
        let mut k = SemanticContinuation::default();
        k.freeze_cleanup_through(SemanticPosition(100)).unwrap();
        let mut state = LifecycleState::new(&k);
        k.commit_action(
            SemanticPosition(4),
            &mut state,
            (),
            |_, _, _, _| Ok::<_, LifecycleFailure>(()),
            |_, _, _| Ok::<_, LifecycleFailure>(()),
        )
        .unwrap();
        let value = SemanticValueId(1);
        let name = state.discover_value(&k, value).unwrap();
        assert!(!state.active.contains_key(&name));
        assert_eq!(state.origins[&name], LifecycleOrigin::Pending);
        assert_eq!(
            state.reify_value(&k, value),
            Err(LifecycleFailure::FormationPending(name))
        );
        let before_state = state.clone();
        let before_k = k.clone();
        assert_eq!(
            commit(&mut k, &mut state, 4, LifecycleAction::Use(name)),
            Err(crate::SemanticCommitFailure::Pre(
                LifecycleFailure::FormationPending(name)
            ))
        );
        assert_eq!(state, before_state);
        assert_eq!(k, before_k);
        assert_eq!(
            state
                .admit_committed_formation_fact(
                    &k,
                    value,
                    SemanticPosition(1),
                    LifecycleOrigin::Pending
                )
                .unwrap(),
            name
        );
        assert_eq!(
            state.active[&name].start,
            SemanticPosition(1),
            "formation occurred before discovery at 4"
        );
        assert_eq!(
            state.reify_value(&k, value),
            Err(LifecycleFailure::OriginPending(name))
        );
        assert_eq!(
            state.observed_colors(name),
            Err(LifecycleFailure::OriginPending(name))
        );
        state.admit_committed_origin_fact(&k, name, None).unwrap();
        assert_eq!(state.origins[&name], LifecycleOrigin::ExplicitNone);
        assert_eq!(state.observed_colors(name), Ok(BTreeSet::new()));
        assert_eq!(
            state.reify_value(&k, value),
            Err(LifecycleFailure::LifecycleContinuationPending)
        );
        assert_eq!(
            state.admit_committed_origin_fact(&k, name, Some(name)),
            Err(LifecycleFailure::OriginAlreadyEstablished(name))
        );
    }

    #[test]
    fn missing_ancestor_origin_is_not_a_terminal_color_fact_or_move_post() {
        let (mut k, mut state, _) = fixture();
        let ancestor = state.discover_value(&k, SemanticValueId(8)).unwrap();
        let source = state
            .admit_committed_formation_fact(
                &k,
                SemanticValueId(9),
                SemanticPosition(0),
                LifecycleOrigin::Name(ancestor),
            )
            .unwrap();
        let before = state.clone();
        assert_eq!(
            state.observed_colors(source),
            Err(LifecycleFailure::OriginPending(ancestor))
        );
        assert_eq!(
            commit(
                &mut k,
                &mut state,
                1,
                LifecycleAction::Move {
                    source,
                    destination: SemanticValueId(10),
                    effect: MoveEffect::Kill
                }
            ),
            Err(crate::SemanticCommitFailure::Pre(
                LifecycleFailure::OriginPending(ancestor)
            ))
        );
        assert_eq!(state, before);
        assert_eq!(k.position(), SemanticPosition(0));
    }

    #[test]
    fn supplied_formation_rejects_future_or_replacement_facts_before_mutation() {
        let (k, mut state, _) = fixture();
        let before = state.clone();
        assert_eq!(
            state.admit_committed_formation_fact(
                &k,
                SemanticValueId(8),
                SemanticPosition(1),
                LifecycleOrigin::ExplicitNone
            ),
            Err(LifecycleFailure::FormationAfterFrontier)
        );
        assert_eq!(state, before);
        assert_eq!(
            state.discover_value(&k, SemanticValueId(7)).unwrap(),
            state.name_of(SemanticValueId(7)).unwrap()
        );
        assert_eq!(
            state.admit_committed_formation_fact(
                &k,
                SemanticValueId(7),
                SemanticPosition(0),
                LifecycleOrigin::Pending
            ),
            Err(LifecycleFailure::ValueAlreadyRegistered(SemanticValueId(7)))
        );
        assert_eq!(state, before);
    }

    #[test]
    fn cleanup_precedes_observation_and_killing_move_shares_one_common_cut() {
        let mut k = SemanticContinuation::default();
        let mut state = LifecycleState::new(&k);
        let source_value = SemanticValueId(7);
        let source = state
            .admit_committed_formation_fact(
                &k,
                source_value,
                SemanticPosition(0),
                LifecycleOrigin::ExplicitNone,
            )
            .unwrap();
        assert_eq!(
            state.reify_value(&k, source_value),
            Err(LifecycleFailure::CleanupPrefixNotFixed)
        );
        k.freeze_cleanup_through(SemanticPosition(100)).unwrap();
        assert_eq!(
            state.reify_value(&k, source_value),
            Err(LifecycleFailure::LifecycleContinuationPending)
        );
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
        assert_eq!(state.name_of(destination), Some(new_name));
        assert_eq!(
            state.reify_value(&k, destination),
            Err(LifecycleFailure::LifecycleContinuationPending)
        );
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
        state
            .admit_committed_color_fact(&k, source, color.clone())
            .unwrap();
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
        assert_eq!(state.observed_colors(source), Ok(BTreeSet::from([color])));
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
    fn color_admission_requires_its_continuation_and_a_formed_active_subject() {
        let (k, mut state, name) = fixture();
        let (foreign_k, mut foreign_state, foreign_name) = fixture();
        assert_eq!(name, foreign_name, "numeric names may coincide across K");
        let color = ColorId("red".into());
        let before = (state.clone(), k.clone());
        assert_eq!(
            state.admit_committed_color_fact(&foreign_k, foreign_name, color.clone()),
            Err(LifecycleFailure::ForeignContinuation)
        );
        assert_eq!((state.clone(), k.clone()), before);

        let foreign_only = foreign_state
            .admit_committed_formation_fact(
                &foreign_k,
                SemanticValueId(8),
                SemanticPosition(0),
                LifecycleOrigin::ExplicitNone,
            )
            .unwrap();
        assert_eq!(
            state.admit_committed_color_fact(&k, foreign_only, color.clone()),
            Err(LifecycleFailure::DeadName(foreign_only))
        );
        assert_eq!((state.clone(), k.clone()), before);
        let pending = state.discover_value(&k, SemanticValueId(8)).unwrap();
        let before = state.clone();
        assert_eq!(
            state.admit_committed_color_fact(&k, pending, color.clone()),
            Err(LifecycleFailure::FormationPending(pending))
        );
        assert_eq!(state, before);

        let unallocated = LifeName(state.next_name);
        let before = state.clone();
        assert_eq!(
            state.admit_committed_color_fact(&k, unallocated, color.clone()),
            Err(LifecycleFailure::DeadName(unallocated))
        );
        assert_eq!(state, before);
        let later = state
            .admit_committed_formation_fact(
                &k,
                SemanticValueId(9),
                SemanticPosition(0),
                LifecycleOrigin::ExplicitNone,
            )
            .unwrap();
        assert_eq!(later, unallocated);
        assert_eq!(
            state.observed_colors(later),
            Ok(BTreeSet::new()),
            "rejected Color cannot pollute a future allocated subject"
        );

        let before_k = k.clone();
        state
            .admit_committed_color_fact(&k, name, color.clone())
            .unwrap();
        let once = state.clone();
        state
            .admit_committed_color_fact(&k, name, color.clone())
            .unwrap();
        assert_eq!(state, once, "admission is monotone and idempotent");
        assert_eq!(state.observed_colors(name), Ok(BTreeSet::from([color])));
        assert_eq!(k, before_k, "fact import allocates no action or cut");
    }

    #[test]
    fn closed_generations_cannot_receive_late_color_or_break_kill_inheritance() {
        let (mut k, mut state, source) = fixture();
        let inherited = ColorId("inherited".into());
        state
            .admit_committed_color_fact(&k, source, inherited.clone())
            .unwrap();
        let destination = commit(
            &mut k,
            &mut state,
            2,
            LifecycleAction::Move {
                source,
                destination: SemanticValueId(8),
                effect: MoveEffect::Kill,
            },
        )
        .unwrap()
        .destination
        .unwrap();
        let before = (state.clone(), k.clone());
        assert_eq!(
            state.admit_committed_color_fact(&k, source, ColorId("late".into())),
            Err(LifecycleFailure::DeadName(source))
        );
        assert_eq!((state.clone(), k.clone()), before);
        assert_eq!(
            state.observed_colors(source),
            Ok(BTreeSet::from([inherited.clone()]))
        );
        assert_eq!(
            state.observed_colors(destination),
            Ok(BTreeSet::from([inherited.clone()]))
        );
        assert_eq!(state.origins[&destination], state.origins[&source]);

        let added = ColorId("destination".into());
        state
            .admit_committed_color_fact(&k, destination, added.clone())
            .unwrap();
        let expected = BTreeSet::from([inherited, added]);
        commit(&mut k, &mut state, 3, LifecycleAction::Drop(destination)).unwrap();
        let before = (state.clone(), k.clone());
        assert_eq!(
            state.admit_committed_color_fact(&k, destination, ColorId("after-drop".into())),
            Err(LifecycleFailure::DeadName(destination))
        );
        assert_eq!((state.clone(), k.clone()), before);
        assert_eq!(state.observed_colors(destination), Ok(expected));
    }

    #[test]
    fn color_admission_rejects_an_old_continuation_before_projection_frontier() {
        let (mut k, mut state, name) = fixture();
        let old_k = k.clone();
        // Same cut, different ordinal: position comparison alone is insufficient.
        commit(&mut k, &mut state, 0, LifecycleAction::Use(name)).unwrap();
        let before = (state.clone(), k.clone());
        assert_eq!(
            state.admit_committed_color_fact(&old_k, name, ColorId("stale".into())),
            Err(LifecycleFailure::StaleOrForeignProof)
        );
        assert_eq!((state.clone(), k.clone()), before);
        state
            .admit_committed_color_fact(&k, name, ColorId("current".into()))
            .unwrap();

        let before_birth = k.clone();
        let destination = commit(
            &mut k,
            &mut state,
            2,
            LifecycleAction::Move {
                source: name,
                destination: SemanticValueId(8),
                effect: MoveEffect::Kill,
            },
        )
        .unwrap()
        .destination
        .unwrap();
        let before = (state.clone(), k.clone());
        assert_eq!(
            state.admit_committed_color_fact(
                &before_birth,
                destination,
                ColorId("before-birth".into())
            ),
            Err(LifecycleFailure::StaleOrForeignProof)
        );
        assert_eq!((state.clone(), k.clone()), before);
        assert_eq!(
            state.observed_colors(destination),
            Ok(BTreeSet::from([ColorId("current".into())]))
        );
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
            .admit_committed_formation_fact(
                &k,
                SemanticValueId(71),
                SemanticPosition(0),
                LifecycleOrigin::Name(ancestor),
            )
            .unwrap();
        let ancestor_color = ColorId("ancestor".into());
        let direct_color = ColorId("direct".into());
        state
            .admit_committed_color_fact(&k, ancestor, ancestor_color.clone())
            .unwrap();
        state
            .admit_committed_color_fact(&k, source, direct_color.clone())
            .unwrap();
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
        assert_eq!(state.origins[&destination], LifecycleOrigin::Name(ancestor));
        assert_eq!(state.observed_colors(destination), before);
        assert_eq!(before, Ok(BTreeSet::from([ancestor_color, direct_color])));
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
        foreign_k
            .freeze_cleanup_through(SemanticPosition(100))
            .unwrap();
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
        let a = state
            .admit_committed_formation_fact(
                &k,
                SemanticValueId(1),
                SemanticPosition(0),
                LifecycleOrigin::ExplicitNone,
            )
            .unwrap();
        let b = state
            .admit_committed_formation_fact(
                &k,
                SemanticValueId(2),
                SemanticPosition(0),
                LifecycleOrigin::ExplicitNone,
            )
            .unwrap();
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
            continuation
                .freeze_cleanup_through(SemanticPosition(100))
                .unwrap();
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
        let a = state
            .admit_committed_formation_fact(
                &k,
                SemanticValueId(1),
                SemanticPosition(0),
                LifecycleOrigin::ExplicitNone,
            )
            .unwrap();
        let b = state
            .admit_committed_formation_fact(
                &k,
                SemanticValueId(2),
                SemanticPosition(0),
                LifecycleOrigin::ExplicitNone,
            )
            .unwrap();
        let c = state
            .admit_committed_formation_fact(
                &k,
                SemanticValueId(3),
                SemanticPosition(0),
                LifecycleOrigin::ExplicitNone,
            )
            .unwrap();
        for (name, declaration_order) in [(b, 2), (a, 1), (c, 3)] {
            k.place_cleanup(crate::CleanupPlacement {
                name,
                declaration_order,
                at: SemanticPosition(9),
            })
            .unwrap();
        }
        k.freeze_cleanup_through(SemanticPosition(100)).unwrap();
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
        let second = state
            .admit_committed_formation_fact(
                &k,
                SemanticValueId(8),
                SemanticPosition(0),
                LifecycleOrigin::ExplicitNone,
            )
            .unwrap();
        assert_ne!(first, second);
        assert_eq!(state.discover_value(&k, SemanticValueId(7)).unwrap(), first);
        assert_eq!(
            state.admit_committed_formation_fact(
                &k,
                SemanticValueId(7),
                SemanticPosition(0),
                LifecycleOrigin::ExplicitNone
            ),
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
        let source = state
            .admit_committed_formation_fact(
                &k,
                SemanticValueId(1),
                SemanticPosition(0),
                LifecycleOrigin::ExplicitNone,
            )
            .unwrap();
        let other = state
            .admit_committed_formation_fact(
                &k,
                SemanticValueId(2),
                SemanticPosition(0),
                LifecycleOrigin::ExplicitNone,
            )
            .unwrap();
        for (name, declaration_order) in [(source, 2), (other, 1)] {
            k.place_cleanup(crate::CleanupPlacement {
                name,
                declaration_order,
                at: SemanticPosition(9),
            })
            .unwrap();
        }
        k.freeze_cleanup_through(SemanticPosition(100)).unwrap();
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
        let destination = state.name_of(SemanticValueId(3)).unwrap();
        assert!(
            state.active.contains_key(&destination),
            "the transferred generation remains independent"
        );
        assert_eq!(
            state.reify_value(&k, SemanticValueId(3)),
            Err(LifecycleFailure::LifecycleContinuationPending)
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
        state
            .admit_committed_color_fact(&k, name, ColorId("new-fact".into()))
            .unwrap();
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
        state.origins.insert(name, LifecycleOrigin::Name(name));
        assert_eq!(
            state.observed_colors(name),
            Ok(BTreeSet::from([ColorId("new-fact".into())]))
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
