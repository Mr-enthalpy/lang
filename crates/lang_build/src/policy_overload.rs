use crate::policy_pair::{FormalPolicyPattern, Phase, PolicyStage, ValueMutability};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MutabilityPattern {
    Const,
    Unspecified,
    Mut,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyOverloadCandidate<I> {
    pub id: I,
    pub formal_frame: MutabilityFormalFrame,
    pub result_policy: Option<MutabilityPattern>,
    pub is_delete: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MutabilityFormalFrame {
    /// Policy Pattern of callable-frame slot 0. The first source-written
    /// formal occupies this position. If no formal is written, the implicit
    /// self-position remains and uses the unspecified Pattern.
    pub self_pattern: MutabilityPattern,
    /// Policy Patterns for source-written positions after the first one.
    /// These consume the explicit call-site Product.
    pub explicit_parameter_patterns: Vec<MutabilityPattern>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MutabilityActualFrame {
    /// Mutability view of the caller object injected into invocation-frame
    /// slot 0. For a standalone function this is the function object; for an
    /// associated `()` entry it is the object whose type supplied that entry.
    pub caller_value: ValueMutability,
    /// Mutability views supplied by the explicit call-site Product.
    pub explicit_arguments: Vec<ValueMutability>,
}

impl<I> PolicyOverloadCandidate<I> {
    /// Build the externally comparable candidate policy from source-order
    /// elaborated formals. The first written formal is the explicitly declared
    /// Pattern for the implicitly passed self-position; only later formals
    /// consume the call-site Product.
    pub fn from_formal_patterns(
        id: I,
        parameters: &[FormalPolicyPattern],
        result_policy: Option<MutabilityPattern>,
        is_delete: bool,
    ) -> Self {
        let mut patterns = parameters.iter().map(formal_mutability_pattern);
        let self_pattern = patterns.next().unwrap_or(MutabilityPattern::Unspecified);
        Self {
            id,
            formal_frame: MutabilityFormalFrame {
                self_pattern,
                explicit_parameter_patterns: patterns.collect(),
            },
            result_policy,
            is_delete,
        }
    }
}

fn formal_mutability_pattern(formal: &FormalPolicyPattern) -> MutabilityPattern {
    match formal.mutability {
        Some(ValueMutability::Const) => MutabilityPattern::Const,
        Some(ValueMutability::Mut) => MutabilityPattern::Mut,
        None => MutabilityPattern::Unspecified,
    }
}

/// A candidate after heterogeneous entry enumeration. The phase-aware selector
/// first removes candidates that are not fully admissible or whose stage is not
/// exposed, then uses one product partial order across mutability positions and
/// phase-local stage specificity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhaseOverloadCandidate<I> {
    pub candidate: PolicyOverloadCandidate<I>,
    pub stage: PolicyStage,
    pub fully_admissible: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyOverloadSelection<I> {
    Selected(I),
    RejectedByDelete(I),
    Ambiguous(Vec<I>),
    NoCandidate,
}

pub fn select_by_mutability_product<I: Clone>(
    candidates: &[PolicyOverloadCandidate<I>],
    actual_frame: &MutabilityActualFrame,
    target_result: Option<ValueMutability>,
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

pub fn select_policy_overload<I: Clone>(
    candidates: &[PhaseOverloadCandidate<I>],
    actual_frame: &MutabilityActualFrame,
    target_result: Option<ValueMutability>,
    phase: Phase,
) -> PolicyOverloadSelection<I> {
    let admissible = candidates
        .iter()
        .filter(|candidate| {
            candidate.fully_admissible
                && candidate.stage.visible_at(phase)
                && frame_arity_matches(&candidate.candidate.formal_frame, actual_frame)
        })
        .collect::<Vec<_>>();
    if admissible.is_empty() {
        return PolicyOverloadSelection::NoCandidate;
    }

    let maximal = maximal_candidates(&admissible, |better, worse| {
        phase_dominates(better, worse, actual_frame, target_result, phase)
    });

    match maximal.as_slice() {
        [] => PolicyOverloadSelection::NoCandidate,
        [candidate] if candidate.candidate.is_delete => {
            PolicyOverloadSelection::RejectedByDelete(candidate.candidate.id.clone())
        }
        [candidate] => PolicyOverloadSelection::Selected(candidate.candidate.id.clone()),
        candidates => PolicyOverloadSelection::Ambiguous(
            candidates
                .iter()
                .map(|candidate| candidate.candidate.id.clone())
                .collect(),
        ),
    }
}

fn phase_dominates<I>(
    better: &PhaseOverloadCandidate<I>,
    worse: &PhaseOverloadCandidate<I>,
    actual_frame: &MutabilityActualFrame,
    target_result: Option<ValueMutability>,
    phase: Phase,
) -> bool {
    let Some(mut strictly_better) = compare_frames(
        &better.candidate.formal_frame,
        &worse.candidate.formal_frame,
        actual_frame,
    ) else {
        return false;
    };

    if let Some(target_result) = target_result {
        let better_result = better
            .candidate
            .result_policy
            .unwrap_or(MutabilityPattern::Unspecified);
        let worse_result = worse
            .candidate
            .result_policy
            .unwrap_or(MutabilityPattern::Unspecified);
        match compare_position(better_result, worse_result, target_result) {
            PositionPreference::Worse => return false,
            PositionPreference::Better => strictly_better = true,
            PositionPreference::Equal => {}
        }
    }

    match compare_stage_specificity(better.stage, worse.stage, phase) {
        PositionPreference::Worse => return false,
        PositionPreference::Better => strictly_better = true,
        PositionPreference::Equal => {}
    }

    strictly_better
}

fn compare_stage_specificity(
    left: PolicyStage,
    right: PolicyStage,
    phase: Phase,
) -> PositionPreference {
    let rank = |stage| match (phase, stage) {
        (Phase::OpenStatic, PolicyStage::Meta) => 2,
        (Phase::OpenStatic, PolicyStage::Compile) => 1,
        (Phase::SealStatic, PolicyStage::Seal) => 2,
        (Phase::SealStatic, PolicyStage::Compile) => 1,
        (Phase::Runtime, PolicyStage::Runtime) => 1,
        _ => 0,
    };
    match rank(left).cmp(&rank(right)) {
        std::cmp::Ordering::Greater => PositionPreference::Better,
        std::cmp::Ordering::Equal => PositionPreference::Equal,
        std::cmp::Ordering::Less => PositionPreference::Worse,
    }
}

fn dominates<I>(
    better: &PolicyOverloadCandidate<I>,
    worse: &PolicyOverloadCandidate<I>,
    actual_frame: &MutabilityActualFrame,
    target_result: Option<ValueMutability>,
) -> bool {
    let Some(mut strictly_better) =
        compare_frames(&better.formal_frame, &worse.formal_frame, actual_frame)
    else {
        return false;
    };

    if let Some(target_result) = target_result {
        let better_result = better
            .result_policy
            .unwrap_or(MutabilityPattern::Unspecified);
        let worse_result = worse
            .result_policy
            .unwrap_or(MutabilityPattern::Unspecified);
        match compare_position(better_result, worse_result, target_result) {
            PositionPreference::Worse => return false,
            PositionPreference::Better => strictly_better = true,
            PositionPreference::Equal => {}
        }
    }

    strictly_better
}

fn frame_arity_matches(formal: &MutabilityFormalFrame, actual: &MutabilityActualFrame) -> bool {
    formal.explicit_parameter_patterns.len() == actual.explicit_arguments.len()
}

/// Compare the complete callable frame. The self-position participates in the
/// same product partial order as every explicit argument, but it is supplied
/// from `actual.caller_value`, never from the call-site Product.
fn compare_frames(
    better: &MutabilityFormalFrame,
    worse: &MutabilityFormalFrame,
    actual: &MutabilityActualFrame,
) -> Option<bool> {
    let mut strictly_better = false;
    match compare_position(better.self_pattern, worse.self_pattern, actual.caller_value) {
        PositionPreference::Worse => return None,
        PositionPreference::Better => strictly_better = true,
        PositionPreference::Equal => {}
    }

    for ((better_policy, worse_policy), argument) in better
        .explicit_parameter_patterns
        .iter()
        .zip(&worse.explicit_parameter_patterns)
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

fn compare_position(
    left: MutabilityPattern,
    right: MutabilityPattern,
    actual: ValueMutability,
) -> PositionPreference {
    let left_rank = position_rank(left, actual);
    let right_rank = position_rank(right, actual);
    match left_rank.cmp(&right_rank) {
        std::cmp::Ordering::Greater => PositionPreference::Better,
        std::cmp::Ordering::Equal => PositionPreference::Equal,
        std::cmp::Ordering::Less => PositionPreference::Worse,
    }
}

fn position_rank(pattern: MutabilityPattern, actual: ValueMutability) -> u8 {
    match (pattern, actual) {
        (MutabilityPattern::Const, ValueMutability::Const)
        | (MutabilityPattern::Mut, ValueMutability::Mut) => 2,
        (MutabilityPattern::Unspecified, _) => 1,
        (MutabilityPattern::Const, ValueMutability::Mut)
        | (MutabilityPattern::Mut, ValueMutability::Const) => 0,
    }
}

/// Shared maximal-element selection for ordinary typed partial orders.
///
/// Candidate adapters own admissibility and comparison dimensions; this
/// function owns the common "retain every non-dominated maximum" rule. It
/// intentionally has no declaration-order fallback.
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
