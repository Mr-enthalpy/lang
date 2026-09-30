//! Shared action positions and atomic projection publication.
//!
//! This is a transaction substrate, not E saturation or a source evaluator.
//! The caller supplies an already determined action/cut, checks every affected
//! projection, and publishes their postconditions together. Cleanup points are
//! supplied by the ordinary control/liveness consumer; only their final
//! same-point linearization is implemented here.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::LifeName;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemanticPosition(pub u64);

/// Opaque in-memory identity; persistent continuation encoding remains open.
#[derive(Clone, Debug)]
pub struct ContinuationIdentity(Arc<()>);

impl PartialEq for ContinuationIdentity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ContinuationIdentity {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticActionIdentity {
    pub continuation: ContinuationIdentity,
    pub ordinal: u64,
}

/// Only the common transaction can construct this committed-action witness.
pub struct CommittedSemanticAction<A> {
    identity: SemanticActionIdentity,
    at: SemanticPosition,
    action: A,
    pre_continuation: SemanticContinuation,
}

impl<A> CommittedSemanticAction<A> {
    pub fn identity(&self) -> &SemanticActionIdentity {
        &self.identity
    }

    pub fn position(&self) -> SemanticPosition {
        self.at
    }

    pub fn action(&self) -> &A {
        &self.action
    }

    pub(crate) fn continuation_before(&self) -> &SemanticContinuation {
        &self.pre_continuation
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CleanupPlacement {
    pub name: LifeName,
    pub at: SemanticPosition,
    /// Established declaration order, independent of schedule insertion order.
    pub declaration_order: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContinuationFailure {
    CleanupScheduleAlreadyFrozen,
    DuplicateCleanup(LifeName),
    UnknownCleanup(LifeName),
    DuplicateDeclarationOrder,
    CleanupPrecedenceContradictsPoints,
    CleanupPrecedenceCycle,
    PositionBeforeFrontier,
    ActionIdentityExhausted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SemanticCommitFailure<E> {
    Continuation(ContinuationFailure),
    Pre(E),
    Projection(E),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticContinuation {
    identity: ContinuationIdentity,
    position: SemanticPosition,
    next_action: u64,
    cleanup: Vec<CleanupPlacement>,
    cleanup_precedence: BTreeSet<(LifeName, LifeName)>,
    cleanup_frozen: bool,
}

impl Default for SemanticContinuation {
    fn default() -> Self {
        Self {
            identity: ContinuationIdentity(Arc::new(())),
            position: SemanticPosition::default(),
            next_action: 0,
            cleanup: Vec::new(),
            cleanup_precedence: BTreeSet::new(),
            cleanup_frozen: false,
        }
    }
}

impl SemanticContinuation {
    pub fn identity(&self) -> &ContinuationIdentity {
        &self.identity
    }

    pub fn position(&self) -> SemanticPosition {
        self.position
    }

    pub(crate) fn next_action_identity(&self) -> SemanticActionIdentity {
        SemanticActionIdentity {
            continuation: self.identity.clone(),
            ordinal: self.next_action,
        }
    }

    /// All Pre callbacks read the original state. Post projections publish as
    /// one transaction; even an invalid/stale projection witness publishes
    /// neither partial state nor a continuation position. No projection owns
    /// position allocation. Multiple ordered actions may occupy one fixed cut.
    pub fn commit_action<S: Clone, A, P, R, E>(
        &mut self,
        at: SemanticPosition,
        state: &mut S,
        action: A,
        check_all_pre: impl FnOnce(&S, &Self, SemanticPosition, &A) -> Result<P, E>,
        apply_all_post: impl FnOnce(&mut S, &CommittedSemanticAction<A>, P) -> Result<R, E>,
    ) -> Result<R, SemanticCommitFailure<E>> {
        if at < self.position {
            return Err(SemanticCommitFailure::Continuation(
                ContinuationFailure::PositionBeforeFrontier,
            ));
        }
        let next_action =
            self.next_action
                .checked_add(1)
                .ok_or(SemanticCommitFailure::Continuation(
                    ContinuationFailure::ActionIdentityExhausted,
                ))?;
        let proofs = check_all_pre(state, self, at, &action).map_err(SemanticCommitFailure::Pre)?;
        let committed = CommittedSemanticAction {
            identity: SemanticActionIdentity {
                continuation: self.identity.clone(),
                ordinal: self.next_action,
            },
            at,
            action,
            pre_continuation: self.clone(),
        };
        // Scratch state is publication storage, never a second evaluator.
        let mut post_state = state.clone();
        let result = apply_all_post(&mut post_state, &committed, proofs)
            .map_err(SemanticCommitFailure::Projection)?;
        *state = post_state;
        self.position = at;
        self.next_action = next_action;
        Ok(result)
    }

    pub fn place_cleanup(
        &mut self,
        placement: CleanupPlacement,
    ) -> Result<(), ContinuationFailure> {
        if self.cleanup_frozen {
            return Err(ContinuationFailure::CleanupScheduleAlreadyFrozen);
        }
        if self
            .cleanup
            .iter()
            .any(|entry| entry.name == placement.name)
        {
            return Err(ContinuationFailure::DuplicateCleanup(placement.name));
        }
        self.cleanup.push(placement);
        Ok(())
    }

    pub fn order_cleanup_before(
        &mut self,
        before: LifeName,
        after: LifeName,
    ) -> Result<(), ContinuationFailure> {
        if self.cleanup_frozen {
            return Err(ContinuationFailure::CleanupScheduleAlreadyFrozen);
        }
        self.cleanup_precedence.insert((before, after));
        Ok(())
    }

    /// Respect all supplied precedence, then prefer reverse declaration order
    /// among available same-point events. No point is moved to make this work.
    pub fn freeze_cleanup_schedule(&mut self) -> Result<(), ContinuationFailure> {
        if self.cleanup_frozen {
            return Err(ContinuationFailure::CleanupScheduleAlreadyFrozen);
        }
        let by_name = self
            .cleanup
            .iter()
            .map(|entry| (entry.name, entry))
            .collect::<BTreeMap<_, _>>();
        for &(before, after) in &self.cleanup_precedence {
            let before = by_name
                .get(&before)
                .ok_or(ContinuationFailure::UnknownCleanup(before))?;
            let after = by_name
                .get(&after)
                .ok_or(ContinuationFailure::UnknownCleanup(after))?;
            if before.at > after.at {
                return Err(ContinuationFailure::CleanupPrecedenceContradictsPoints);
            }
        }
        let mut points = BTreeMap::<SemanticPosition, Vec<&CleanupPlacement>>::new();
        for entry in &self.cleanup {
            points.entry(entry.at).or_default().push(entry);
        }
        let mut ordered = Vec::new();
        for entries in points.values() {
            let mut remaining = entries
                .iter()
                .map(|entry| entry.name)
                .collect::<BTreeSet<_>>();
            let orders = entries
                .iter()
                .map(|entry| entry.declaration_order)
                .collect::<BTreeSet<_>>();
            if orders.len() != entries.len() {
                return Err(ContinuationFailure::DuplicateDeclarationOrder);
            }
            while !remaining.is_empty() {
                let selected = entries
                    .iter()
                    .filter(|entry| remaining.contains(&entry.name))
                    .filter(|entry| {
                        !self.cleanup_precedence.iter().any(|(before, after)| {
                            *after == entry.name && remaining.contains(before)
                        })
                    })
                    .max_by_key(|entry| entry.declaration_order)
                    .ok_or(ContinuationFailure::CleanupPrecedenceCycle)?;
                ordered.push((*selected).clone());
                remaining.remove(&selected.name);
            }
        }
        self.cleanup = ordered;
        self.cleanup_frozen = true;
        Ok(())
    }

    pub fn cleanup_is_frozen(&self) -> bool {
        self.cleanup_frozen
    }

    pub fn cleanup(&self) -> &[CleanupPlacement] {
        &self.cleanup
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placement(name: u64, at: u64, declaration_order: u64) -> CleanupPlacement {
        CleanupPlacement {
            name: LifeName(name),
            at: SemanticPosition(at),
            declaration_order,
        }
    }

    #[test]
    fn cleanup_order_uses_explicit_precedence_before_reverse_declaration_priority() {
        let mut k = SemanticContinuation::default();
        for entry in [placement(2, 9, 2), placement(3, 9, 3), placement(1, 9, 1)] {
            k.place_cleanup(entry).unwrap();
        }
        // a must precede c; unrelated b is the first available reverse-order event.
        k.order_cleanup_before(LifeName(1), LifeName(3)).unwrap();
        k.freeze_cleanup_schedule().unwrap();
        assert_eq!(
            k.cleanup()
                .iter()
                .map(|entry| entry.name)
                .collect::<Vec<_>>(),
            vec![LifeName(2), LifeName(1), LifeName(3)]
        );
        assert!(k
            .cleanup()
            .iter()
            .all(|entry| entry.at == SemanticPosition(9)));
        assert_eq!(
            k.place_cleanup(placement(4, 9, 4)),
            Err(ContinuationFailure::CleanupScheduleAlreadyFrozen)
        );
        assert_eq!(
            k.order_cleanup_before(LifeName(3), LifeName(2)),
            Err(ContinuationFailure::CleanupScheduleAlreadyFrozen)
        );
    }

    #[test]
    fn declaration_order_never_moves_fixed_cleanup_points() {
        let mut k = SemanticContinuation::default();
        k.place_cleanup(placement(1, 2, 1)).unwrap();
        k.place_cleanup(placement(2, 1, 2)).unwrap();
        k.freeze_cleanup_schedule().unwrap();
        assert_eq!(k.cleanup(), &[placement(2, 1, 2), placement(1, 2, 1)]);

        let mut invalid = SemanticContinuation::default();
        invalid.place_cleanup(placement(1, 2, 1)).unwrap();
        invalid.place_cleanup(placement(2, 1, 2)).unwrap();
        invalid
            .order_cleanup_before(LifeName(1), LifeName(2))
            .unwrap();
        let before = invalid.clone();
        assert_eq!(
            invalid.freeze_cleanup_schedule(),
            Err(ContinuationFailure::CleanupPrecedenceContradictsPoints)
        );
        assert_eq!(
            invalid, before,
            "a contradiction diagnoses instead of relocating cleanup"
        );
    }

    #[test]
    fn invalid_cleanup_relations_fail_without_freezing_or_reordering() {
        let mut k = SemanticContinuation::default();
        k.place_cleanup(placement(1, 2, 1)).unwrap();
        assert_eq!(
            k.place_cleanup(placement(1, 2, 2)),
            Err(ContinuationFailure::DuplicateCleanup(LifeName(1)))
        );
        k.place_cleanup(placement(2, 2, 2)).unwrap();
        k.order_cleanup_before(LifeName(1), LifeName(2)).unwrap();
        k.order_cleanup_before(LifeName(2), LifeName(1)).unwrap();
        let before = k.clone();
        assert_eq!(
            k.freeze_cleanup_schedule(),
            Err(ContinuationFailure::CleanupPrecedenceCycle)
        );
        assert_eq!(k, before);

        let mut unknown = SemanticContinuation::default();
        unknown.place_cleanup(placement(1, 2, 1)).unwrap();
        unknown
            .order_cleanup_before(LifeName(1), LifeName(3))
            .unwrap();
        assert_eq!(
            unknown.freeze_cleanup_schedule(),
            Err(ContinuationFailure::UnknownCleanup(LifeName(3)))
        );
        let mut ambiguous = SemanticContinuation::default();
        ambiguous.place_cleanup(placement(1, 2, 1)).unwrap();
        ambiguous.place_cleanup(placement(2, 2, 1)).unwrap();
        assert_eq!(
            ambiguous.freeze_cleanup_schedule(),
            Err(ContinuationFailure::DuplicateDeclarationOrder)
        );
    }

    #[test]
    fn failed_projection_publication_changes_neither_other_projection_nor_cut() {
        let mut k = SemanticContinuation::default();
        let before = k.clone();
        let mut state = (0u64, 0u64);
        let result = k.commit_action(
            SemanticPosition(2),
            &mut state,
            "ordinary action",
            |_, _, _, _| Ok::<_, &str>(()),
            |state, _, _| {
                state.0 = 5;
                Err::<(), _>("second projection rejects a stale witness")
            },
        );
        assert_eq!(
            result,
            Err(SemanticCommitFailure::Projection(
                "second projection rejects a stale witness"
            ))
        );
        assert_eq!(state, (0, 0));
        assert_eq!(k, before);
        let identity = k
            .commit_action(
                SemanticPosition(2),
                &mut state,
                "ordinary action",
                |_, _, _, _| Ok::<_, &str>(()),
                |state, action, _| {
                    state.1 = 6;
                    Ok::<_, &str>(action.identity().clone())
                },
            )
            .unwrap();
        assert_eq!(
            identity.ordinal, 0,
            "failed publication did not allocate a semantic action"
        );
        assert_eq!(state, (0, 6));
        assert_eq!(k.position(), SemanticPosition(2));
        let before = k.clone();
        let invalid = k.commit_action(
            SemanticPosition(1),
            &mut state,
            (),
            |_, _, _, _| -> Result<(), &str> { panic!("old cut must fail before Pre") },
            |_, _, _| Ok::<_, &str>(()),
        );
        assert_eq!(
            invalid,
            Err(SemanticCommitFailure::Continuation(
                ContinuationFailure::PositionBeforeFrontier
            ))
        );
        assert_eq!(k, before);
    }
}
