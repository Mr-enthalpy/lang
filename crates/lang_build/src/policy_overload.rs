use crate::policy_pair::{FormalPolicyPattern, OutputModeDemand, PolicyMode};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyOverloadCandidate<I> {
    pub id: I,
    pub formal_frame: PolicyFormalFrame,
    pub result_policy: PolicyMode,
    pub is_delete: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyFormalFrame {
    /// Policy Pattern of callable-frame slot 0. The first source-written
    /// formal occupies this position. If no formal is written, the implicit
    /// self-position remains and uses the primitive `plain` mode.
    pub self_mode: PolicyMode,
    /// Policy Patterns for source-written positions after the first one.
    /// These consume the explicit call-site Product.
    pub explicit_parameter_modes: Vec<PolicyMode>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyActualFrame {
    /// Whole-slot Policy mode of the caller object injected into invocation-frame
    /// slot 0. For a standalone function this is the function object; for an
    /// associated `()` entry it is the object whose type supplied that entry.
    pub caller_value: PolicyMode,
    /// Whole-slot Policy modes supplied by the explicit call-site Product.
    pub explicit_arguments: Vec<PolicyMode>,
}

impl<I> PolicyOverloadCandidate<I> {
    /// Build the externally comparable candidate policy from source-order
    /// elaborated formals. Slot zero is the written or generated
    /// Pattern for the implicitly passed self-position; only later formals
    /// consume the call-site Product.
    pub fn from_formal_patterns(
        id: I,
        parameters: &[FormalPolicyPattern],
        result_policy: PolicyMode,
        is_delete: bool,
    ) -> Self {
        let mut modes = parameters.iter().map(formal_policy_mode);
        let self_mode = modes.next().unwrap_or(PolicyMode::Plain);
        Self {
            id,
            formal_frame: PolicyFormalFrame {
                self_mode,
                explicit_parameter_modes: modes.collect(),
            },
            result_policy,
            is_delete,
        }
    }
}

fn formal_policy_mode(formal: &FormalPolicyPattern) -> PolicyMode {
    formal.mode
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyOverloadSelection<I> {
    Selected(I),
    RejectedByDelete(I),
    Ambiguous(Vec<I>),
    NoCandidate,
}

pub fn select_by_policy_product<I: Clone>(
    candidates: &[PolicyOverloadCandidate<I>],
    actual_frame: &PolicyActualFrame,
    target_result: OutputModeDemand,
) -> PolicyOverloadSelection<I> {
    let admissible = candidates
        .iter()
        .filter(|candidate| frame_arity_matches(&candidate.formal_frame, actual_frame))
        .collect::<Vec<_>>();
    if admissible.is_empty() {
        return PolicyOverloadSelection::NoCandidate;
    }

    let maximal = maximal_candidates(&admissible, |better, worse| {
        dominates(better, worse, actual_frame, target_result)
    });

    match maximal.as_slice() {
        [] => PolicyOverloadSelection::NoCandidate,
        [candidate] if candidate.is_delete => {
            PolicyOverloadSelection::RejectedByDelete(candidate.id.clone())
        }
        [candidate] => PolicyOverloadSelection::Selected(candidate.id.clone()),
        candidates => PolicyOverloadSelection::Ambiguous(
            candidates
                .iter()
                .map(|candidate| candidate.id.clone())
                .collect(),
        ),
    }
}

fn dominates<I>(
    better: &PolicyOverloadCandidate<I>,
    worse: &PolicyOverloadCandidate<I>,
    actual_frame: &PolicyActualFrame,
    target_result: OutputModeDemand,
) -> bool {
    let Some(mut strictly_better) =
        compare_frames(&better.formal_frame, &worse.formal_frame, actual_frame)
    else {
        return false;
    };

    match compare_position(
        better.result_policy,
        worse.result_policy,
        target_result.mode(),
    ) {
        PositionPreference::Worse => return false,
        PositionPreference::Better => strictly_better = true,
        PositionPreference::Equal => {}
    }

    strictly_better
}

fn frame_arity_matches(formal: &PolicyFormalFrame, actual: &PolicyActualFrame) -> bool {
    formal.explicit_parameter_modes.len() == actual.explicit_arguments.len()
}

/// Compare the complete callable frame. The self-position participates in the
/// same product partial order as every explicit argument, but it is supplied
/// from `actual.caller_value`, never from the call-site Product.
fn compare_frames(
    better: &PolicyFormalFrame,
    worse: &PolicyFormalFrame,
    actual: &PolicyActualFrame,
) -> Option<bool> {
    let mut strictly_better = false;
    match compare_position(better.self_mode, worse.self_mode, actual.caller_value) {
        PositionPreference::Worse => return None,
        PositionPreference::Better => strictly_better = true,
        PositionPreference::Equal => {}
    }

    for ((better_policy, worse_policy), argument) in better
        .explicit_parameter_modes
        .iter()
        .zip(&worse.explicit_parameter_modes)
        .zip(&actual.explicit_arguments)
    {
        match compare_position(*better_policy, *worse_policy, *argument) {
            PositionPreference::Worse => return None,
            PositionPreference::Better => strictly_better = true,
            PositionPreference::Equal => {}
        }
    }
    Some(strictly_better)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PositionPreference {
    Better,
    Equal,
    Worse,
}

fn compare_position(left: PolicyMode, right: PolicyMode, actual: PolicyMode) -> PositionPreference {
    let left_rank = policy_mode_preference_rank(left, actual);
    let right_rank = policy_mode_preference_rank(right, actual);
    match left_rank.cmp(&right_rank) {
        std::cmp::Ordering::Greater => PositionPreference::Better,
        std::cmp::Ordering::Equal => PositionPreference::Equal,
        std::cmp::Ordering::Less => PositionPreference::Worse,
    }
}

/// Ordinary actual-relative Policy-mode preference used by Bp.
///
/// Migration endpoint projections must reuse this relation rather than treating
/// opposite const/mut Patterns as hard-incompatible Policy domains.
pub(crate) fn policy_mode_preference_rank(candidate: PolicyMode, demand: PolicyMode) -> u8 {
    match (candidate, demand) {
        (PolicyMode::Const, PolicyMode::Const)
        | (PolicyMode::Plain, PolicyMode::Plain)
        | (PolicyMode::Mut, PolicyMode::Mut) => 2,
        (PolicyMode::Plain, PolicyMode::Const | PolicyMode::Mut)
        | (PolicyMode::Const | PolicyMode::Mut, PolicyMode::Plain) => 1,
        (PolicyMode::Const, PolicyMode::Mut) | (PolicyMode::Mut, PolicyMode::Const) => 0,
    }
}

/// Shared maximal-element selection for ordinary typed partial orders.
/// Candidate projections own admissibility and comparison dimensions;
/// every non-dominated maximum remains, with no declaration-order fallback.
pub(crate) fn maximal_candidates<'a, T, F>(candidates: &[&'a T], mut dominates: F) -> Vec<&'a T>
where
    F: FnMut(&T, &T) -> bool,
{
    candidates
        .iter()
        .enumerate()
        .filter(|(candidate_index, candidate)| {
            !candidates.iter().enumerate().any(|(other_index, other)| {
                *candidate_index != other_index && dominates(other, candidate)
            })
        })
        .map(|(_, candidate)| *candidate)
        .collect()
}
