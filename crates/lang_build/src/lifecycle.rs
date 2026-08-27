//! Continuation-relative lifecycle semantics.
//!
//! The carriers in this module intentionally do not encode a CFG, arena, or
//! closed Color universe.  They establish the canonical relations consumed by
//! evaluation: one `SemanticContinuation`, finite lifetime observations,
//! half-open generations, cleanup-before-observation, Pre-before-action/Post-
//! after-commit, and extensible Color/access snapshots.

use std::collections::{BTreeMap, BTreeSet};

use crate::{Diagnostic, Provenance, SemanticValueId};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemanticPosition(pub u64);

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
    Move { replacement: LifeName },
    Drop,
    Cleanup,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LifecycleEvent {
    pub name: LifeName,
    pub at: SemanticPosition,
    pub kind: LifecycleEventKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CleanupPlacement {
    pub name: LifeName,
    pub at: SemanticPosition,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SemanticContinuation {
    position: SemanticPosition,
    cleanup: Vec<CleanupPlacement>,
    cleanup_frozen: bool,
    events: Vec<LifecycleEvent>,
}

impl SemanticContinuation {
    pub fn position(&self) -> SemanticPosition {
        self.position
    }

    pub fn place_cleanup(&mut self, placement: CleanupPlacement) -> Result<(), LifecycleFailure> {
        if self.cleanup_frozen {
            return Err(LifecycleFailure::CleanupScheduleAlreadyFrozen);
        }
        self.cleanup.push(placement);
        Ok(())
    }

    pub fn freeze_cleanup_schedule(&mut self) {
        self.cleanup_frozen = true;
    }

    pub fn cleanup_is_frozen(&self) -> bool {
        self.cleanup_frozen
    }

    pub fn cleanup(&self) -> &[CleanupPlacement] {
        &self.cleanup
    }

    pub fn events(&self) -> &[LifecycleEvent] {
        &self.events
    }

    fn commit(&mut self, name: LifeName, kind: LifecycleEventKind) -> LifecycleEvent {
        self.position.0 = self.position.0.saturating_add(1);
        let event = LifecycleEvent {
            name,
            at: self.position,
            kind,
        };
        self.events.push(event.clone());
        event
    }
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LifecycleAction {
    Use(LifeName),
    Move(LifeName),
    Drop(LifeName),
    Cleanup(LifeName),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LifecyclePost {
    pub event: LifecycleEvent,
    pub closed_region: Option<NameView<()>>,
    pub replacement: Option<LifeName>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LifecycleFailure {
    CleanupScheduleNotFrozen,
    CleanupScheduleAlreadyFrozen,
    UnknownValue(SemanticValueId),
    DeadName(LifeName),
    CleanupNotPlaced(LifeName),
    PreRejected(String),
}

#[derive(Clone, Debug, Default)]
pub struct LifecycleMachine {
    continuation: SemanticContinuation,
    next_name: u64,
    values: BTreeMap<SemanticValueId, LifeName>,
    active: BTreeMap<LifeName, Region>,
    origins: BTreeMap<LifeName, Option<LifeName>>,
    colors: BTreeMap<LifeName, BTreeSet<ColorId>>,
    closed: Vec<NameView<()>>,
}

impl LifecycleMachine {
    pub fn continuation(&self) -> &SemanticContinuation {
        &self.continuation
    }

    pub fn continuation_mut(&mut self) -> &mut SemanticContinuation {
        &mut self.continuation
    }

    pub fn register_value(&mut self, value: SemanticValueId, origin: Option<LifeName>) -> LifeName {
        let name = LifeName(self.next_name);
        self.next_name = self.next_name.saturating_add(1);
        self.values.insert(value, name);
        self.origins.insert(name, origin);
        self.active.insert(
            name,
            Region {
                start: self.continuation.position(),
                end: None,
                generation: 0,
            },
        );
        name
    }

    pub fn ensure_value(&mut self, value: SemanticValueId) -> LifeName {
        self.values
            .get(&value)
            .copied()
            .unwrap_or_else(|| self.register_value(value, None))
    }

    pub fn name_of(&self, value: SemanticValueId) -> Option<LifeName> {
        self.values.get(&value).copied()
    }

    pub fn assign_color(&mut self, name: LifeName, color: ColorId) {
        self.colors.entry(name).or_default().insert(color);
    }

    /// Finite observation of inherited Color facts. Cyclic/coinductive origin
    /// material is observed only until the first repeated name.
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

    /// `@ = ReifyLife(NameOf(E), Pos(K))`. The operand is a semantic value;
    /// no Place coordinate is read or required.
    pub fn reify_value(&self, value: SemanticValueId) -> Result<LifetimeValue, LifecycleFailure> {
        if !self.continuation.cleanup_is_frozen() {
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
            observed_at: self.continuation.position(),
            origin: self.origins.get(&name).copied().flatten(),
            region,
        })
    }

    pub fn snapshot(&self, colors: ColorAlgebra, access: AccessSnapshot) -> LifecycleSnapshot {
        LifecycleSnapshot {
            live: self.active.keys().copied().collect(),
            colors,
            access,
        }
    }

    /// Validate all Pre facts before allocating a continuation cut or
    /// mutating lifecycle state. Post is returned only after commit.
    pub fn perform(
        &mut self,
        action: LifecycleAction,
        validation: &LifecycleValidationContext,
        provenance: Provenance,
    ) -> Result<LifecyclePost, LifecycleFailure> {
        validation
            .validate_pre(&provenance)
            .map_err(|diagnostic| LifecycleFailure::PreRejected(diagnostic.message))?;
        let name = match &action {
            LifecycleAction::Use(name)
            | LifecycleAction::Move(name)
            | LifecycleAction::Drop(name)
            | LifecycleAction::Cleanup(name) => *name,
        };
        let current = self
            .active
            .get(&name)
            .copied()
            .ok_or(LifecycleFailure::DeadName(name))?;
        if matches!(action, LifecycleAction::Cleanup(_))
            && !self
                .continuation
                .cleanup()
                .iter()
                .any(|placement| placement.name == name)
        {
            return Err(LifecycleFailure::CleanupNotPlaced(name));
        }

        match action {
            LifecycleAction::Use(name) => {
                let event = self.continuation.commit(name, LifecycleEventKind::Use);
                Ok(LifecyclePost {
                    event,
                    closed_region: None,
                    replacement: None,
                })
            }
            LifecycleAction::Move(name) => {
                // Freeze the finite Color observation of the old generation
                // before committing the move cut. The replacement preserves
                // the deeper origin (not the moved-from name), so this
                // explicit generation snapshot is what enforces
                // `ObservedColors(new) >= ObservedColors(old)` without
                // inventing a new origin edge or closing the future storage
                // representation.
                let inherited_colors = self.observed_colors(name);
                let replacement = LifeName(self.next_name);
                self.next_name = self.next_name.saturating_add(1);
                let event = self
                    .continuation
                    .commit(name, LifecycleEventKind::Move { replacement });
                let closed_region = Region {
                    end: Some(event.at),
                    ..current
                };
                self.active.remove(&name);
                let view = NameView {
                    name,
                    value: (),
                    origin: self.origins.get(&name).copied().flatten(),
                    region: closed_region,
                };
                self.closed.push(view.clone());
                // Move preserves the deeper origin exactly; it does not add
                // the moved-from name as a new ancestry layer.
                let deeper_origin = self.origins.get(&name).copied().flatten();
                self.origins.insert(replacement, deeper_origin);
                self.colors.insert(replacement, inherited_colors);
                self.active.insert(
                    replacement,
                    Region {
                        start: event.at,
                        end: None,
                        generation: current.generation.saturating_add(1),
                    },
                );
                Ok(LifecyclePost {
                    event,
                    closed_region: Some(view),
                    replacement: Some(replacement),
                })
            }
            LifecycleAction::Drop(name) => {
                let event = self.continuation.commit(name, LifecycleEventKind::Drop);
                let closed_region = Region {
                    end: Some(event.at),
                    ..current
                };
                self.active.remove(&name);
                let view = NameView {
                    name,
                    value: (),
                    origin: self.origins.get(&name).copied().flatten(),
                    region: closed_region,
                };
                self.closed.push(view.clone());
                Ok(LifecyclePost {
                    event,
                    closed_region: Some(view),
                    replacement: None,
                })
            }
            LifecycleAction::Cleanup(name) => {
                let kind = LifecycleEventKind::Cleanup;
                let event = self.continuation.commit(name, kind);
                let closed_region = Region {
                    end: Some(event.at),
                    ..current
                };
                self.active.remove(&name);
                let view = NameView {
                    name,
                    value: (),
                    origin: self.origins.get(&name).copied().flatten(),
                    region: closed_region,
                };
                self.closed.push(view.clone());
                Ok(LifecyclePost {
                    event,
                    closed_region: Some(view),
                    replacement: None,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleanup_precedes_observation_and_move_regions_share_one_cut() {
        let mut machine = LifecycleMachine::default();
        let value = SemanticValueId(7);
        let name = machine.register_value(value, None);
        assert_eq!(
            machine.reify_value(value),
            Err(LifecycleFailure::CleanupScheduleNotFrozen)
        );
        machine
            .continuation_mut()
            .place_cleanup(CleanupPlacement {
                name,
                at: SemanticPosition(9),
            })
            .expect("cleanup placement is fixed first");
        machine.continuation_mut().freeze_cleanup_schedule();
        assert_eq!(
            machine.reify_value(value).expect("@ needs no Place").name,
            name
        );

        let validation = LifecycleValidationContext {
            snapshot: machine.snapshot(ColorAlgebra::default(), AccessSnapshot::default()),
            preconditions: vec![LifecyclePrecondition::Alive(name)],
        };
        let moved = machine
            .perform(
                LifecycleAction::Move(name),
                &validation,
                Provenance::new("move"),
            )
            .expect("valid Pre commits move");
        let replacement = moved.replacement.expect("move opens next generation");
        assert_eq!(
            moved
                .closed_region
                .expect("old generation closes")
                .region
                .end,
            Some(moved.event.at)
        );
        assert_eq!(
            machine
                .active
                .get(&replacement)
                .expect("new generation")
                .start,
            moved.event.at,
            "old end and new start are the same continuation cut"
        );
    }

    #[test]
    fn move_preserves_direct_and_inherited_colors_without_changing_deeper_origin() {
        let mut machine = LifecycleMachine::default();
        let ancestor = machine.register_value(SemanticValueId(70), None);
        let old = machine.register_value(SemanticValueId(71), Some(ancestor));
        let ancestor_color = ColorId("ancestor".into());
        let direct_color = ColorId("direct-old-generation".into());
        machine.assign_color(ancestor, ancestor_color.clone());
        machine.assign_color(old, direct_color.clone());
        let before = machine.observed_colors(old);
        assert_eq!(
            before,
            BTreeSet::from([ancestor_color.clone(), direct_color.clone()])
        );

        let validation = LifecycleValidationContext {
            snapshot: machine.snapshot(ColorAlgebra::default(), AccessSnapshot::default()),
            preconditions: vec![LifecyclePrecondition::Alive(old)],
        };
        let moved = machine
            .perform(
                LifecycleAction::Move(old),
                &validation,
                Provenance::new("move keeps every observed Color"),
            )
            .expect("move commits");
        let replacement = moved.replacement.expect("replacement generation");

        assert_eq!(machine.origins.get(&replacement), Some(&Some(ancestor)));
        assert_eq!(
            machine.observed_colors(replacement),
            before,
            "a generation cut may slice Color regions but never remove an already observed Color"
        );
        assert!(machine.observed_colors(replacement).contains(&direct_color));
    }

    #[test]
    fn pre_failure_has_no_effect_and_color_vocabulary_is_extensible() {
        let mut machine = LifecycleMachine::default();
        let name = machine.register_value(SemanticValueId(1), None);
        machine.continuation_mut().freeze_cleanup_schedule();
        let before = machine.clone();
        let rejected = LifecycleValidationContext {
            snapshot: machine.snapshot(ColorAlgebra::default(), AccessSnapshot::default()),
            preconditions: vec![LifecyclePrecondition::Reject("no authority".into())],
        };
        assert!(machine
            .perform(
                LifecycleAction::Drop(name),
                &rejected,
                Provenance::new("rejected drop"),
            )
            .is_err());
        assert_eq!(machine.continuation, before.continuation);
        assert_eq!(machine.active, before.active);

        let mut colors = ColorAlgebra::default();
        let future = ColorId("project-defined/future-color".into());
        colors.register(future.clone());
        assert!(colors.contains(&future));
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
