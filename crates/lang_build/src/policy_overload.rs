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
    pub parameter_policies: Vec<MutabilityPattern>,
    pub result_policy: Option<MutabilityPattern>,
    pub is_delete: bool,
}

impl<I> PolicyOverloadCandidate<I> {
    /// Build the externally comparable candidate policy from elaborated
    /// formal parameters. The const/mut slice is not body-local metadata: it
    /// is copied into the product-order positions used to select this
    /// candidate from its overload set.
    pub fn from_formal_patterns(
        id: I,
        parameters: &[FormalPolicyPattern],
        result_policy: Option<MutabilityPattern>,
        is_delete: bool,
    ) -> Self {
        Self {
            id,
            parameter_policies: parameters
                .iter()
                .map(|formal| match formal.mutability {
                    Some(ValueMutability::Const) => MutabilityPattern::Const,
                    Some(ValueMutability::Mut) => MutabilityPattern::Mut,
                    None => MutabilityPattern::Unspecified,
                })
                .collect(),
            result_policy,
            is_delete,
        }
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
    arguments: &[ValueMutability],
    target_result: Option<ValueMutability>,
) -> PolicyOverloadSelection<I> {
    let admissible = candidates
        .iter()
        .filter(|candidate| candidate.parameter_policies.len() == arguments.len())
        .collect::<Vec<_>>();
    if admissible.is_empty() {
        return PolicyOverloadSelection::NoCandidate;
    }

    let maximal = admissible
        .iter()
        .enumerate()
        .filter(|(candidate_index, candidate)| {
            !admissible.iter().enumerate().any(|(other_index, other)| {
                *candidate_index != other_index
                    && dominates(other, candidate, arguments, target_result)
            })
        })
        .map(|(_, candidate)| *candidate)
        .collect::<Vec<_>>();

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
    arguments: &[ValueMutability],
    target_result: Option<ValueMutability>,
    phase: Phase,
) -> PolicyOverloadSelection<I> {
    let admissible = candidates
        .iter()
        .filter(|candidate| {
            candidate.fully_admissible
                && candidate.stage.visible_at(phase)
                && candidate.candidate.parameter_policies.len() == arguments.len()
        })
        .collect::<Vec<_>>();
    if admissible.is_empty() {
        return PolicyOverloadSelection::NoCandidate;
    }

    let maximal = admissible
        .iter()
        .enumerate()
        .filter(|(candidate_index, candidate)| {
            !admissible.iter().enumerate().any(|(other_index, other)| {
                *candidate_index != other_index
                    && phase_dominates(other, candidate, arguments, target_result, phase)
            })
        })
        .map(|(_, candidate)| *candidate)
        .collect::<Vec<_>>();

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
    arguments: &[ValueMutability],
    target_result: Option<ValueMutability>,
    phase: Phase,
) -> bool {
    let mut strictly_better = false;
    for ((better_policy, worse_policy), argument) in better
        .candidate
        .parameter_policies
        .iter()
        .zip(&worse.candidate.parameter_policies)
        .zip(arguments)
    {
        match compare_position(*better_policy, *worse_policy, *argument) {
            PositionPreference::Worse => return false,
            PositionPreference::Better => strictly_better = true,
            PositionPreference::Equal => {}
        }
    }

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
    arguments: &[ValueMutability],
    target_result: Option<ValueMutability>,
) -> bool {
    let mut strictly_better = false;
    for ((better_policy, worse_policy), argument) in better
        .parameter_policies
        .iter()
        .zip(&worse.parameter_policies)
        .zip(arguments)
    {
        match compare_position(*better_policy, *worse_policy, *argument) {
            PositionPreference::Worse => return false,
            PositionPreference::Better => strictly_better = true,
            PositionPreference::Equal => {}
        }
    }

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
