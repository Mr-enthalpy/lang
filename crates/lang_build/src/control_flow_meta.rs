//! Control-flow-local meta evaluation substrate.
//!
//! Provides the four cornerstones for branch-local guarded evaluation:
//! 1. simple type facts / predicates
//! 2. simple policy facts / requirements
//! 3. branch-local symbol-space construction and lookup
//! 4. branch-arm selection and guarded-branch driver
//!
//! The central invariant: **unselected branches have no symbol lookup, policy
//! check, invocation, or symbol-installation obligation**. Only the selected
//! branch is inspected; only the selected branch may construct local symbols,
//! perform lookup, require policy, or plan meta invocation.
//!
//! This is not a full meta interpreter. It does not execute arbitrary user
//! meta-function bodies, loops, full pattern matching, full type solving, or
//! backend lowering.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    extraction_view::EvalResultNormalForm,
    model::{Diagnostic, DiagnosticSeverity, Provenance, SymbolId, SymbolKind},
    pattern_space::SelectedSumPattern,
};

// ---------------------------------------------------------------------------
// Guard residual reason
// ---------------------------------------------------------------------------

/// Why a guarded-branch evaluation could not select a branch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuardResidualReason {
    UnknownSelector,
    NonSumSelector,
    AmbiguousAlternative,
}

// ---------------------------------------------------------------------------
// Simple type check substrate
// ---------------------------------------------------------------------------

/// Branch-local type facts for guarded-branch type-predicate evaluation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimpleTypeFacts {
    pub value_type_symbols: BTreeMap<String, SymbolId>,
    pub predicates: Vec<SimpleTypePredicateFact>,
}

/// One type predicate fact asserted for a given type symbol.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimpleTypePredicateFact {
    pub type_symbol_id: SymbolId,
    pub predicate: SimpleTypePredicate,
    pub value: bool,
    pub provenance: Provenance,
}

/// Recognised simple type predicates.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SimpleTypePredicate {
    HasError,
    HasPass,
    HasCustomQuestionView,
    IsValue,
}

impl SimpleTypeFacts {
    pub fn new() -> Self {
        Self {
            value_type_symbols: BTreeMap::new(),
            predicates: Vec::new(),
        }
    }
}

/// Check a simple type predicate against the supplied facts.
pub fn check_simple_type_predicate(
    facts: &SimpleTypeFacts,
    type_symbol_id: SymbolId,
    predicate: SimpleTypePredicate,
) -> SimpleTypeCheckResult {
    for fact in &facts.predicates {
        if fact.type_symbol_id == type_symbol_id && fact.predicate == predicate {
            if fact.value {
                return SimpleTypeCheckResult::KnownTrue;
            } else {
                return SimpleTypeCheckResult::KnownFalse;
            }
        }
    }
    SimpleTypeCheckResult::Unknown
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SimpleTypeCheckResult {
    KnownTrue,
    KnownFalse,
    Unknown,
}

// ---------------------------------------------------------------------------
// Simple policy check substrate
// ---------------------------------------------------------------------------

/// Branch-local policy facts for guarded-branch policy-requirement evaluation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimplePolicyFacts {
    pub visible_symbols: BTreeSet<SymbolId>,
    pub executable_symbols: BTreeSet<SymbolId>,
    pub available_capabilities: BTreeSet<SimpleCapability>,
}

/// A simple capability that may be required by a branch.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SimpleCapability {
    ReturnCapability,
    ErrorReturnCapability,
    MetaExecution,
}

/// A policy requirement that a branch may declare.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SimplePolicyRequirement {
    SymbolVisible(SymbolId),
    BodyExecutable(SymbolId),
    CapabilityAvailable(SimpleCapability),
}

impl SimplePolicyFacts {
    pub fn new() -> Self {
        Self {
            visible_symbols: BTreeSet::new(),
            executable_symbols: BTreeSet::new(),
            available_capabilities: BTreeSet::new(),
        }
    }
}

/// Check a simple policy requirement against the supplied facts.
pub fn check_simple_policy(
    facts: &SimplePolicyFacts,
    requirement: &SimplePolicyRequirement,
) -> SimplePolicyCheckResult {
    match requirement {
        SimplePolicyRequirement::SymbolVisible(symbol_id) => {
            if facts.visible_symbols.contains(symbol_id) {
                SimplePolicyCheckResult::Allowed
            } else {
                SimplePolicyCheckResult::Unknown
            }
        }
        SimplePolicyRequirement::BodyExecutable(symbol_id) => {
            if facts.executable_symbols.contains(symbol_id) {
                SimplePolicyCheckResult::Allowed
            } else {
                SimplePolicyCheckResult::Unknown
            }
        }
        SimplePolicyRequirement::CapabilityAvailable(cap) => {
            if facts.available_capabilities.contains(cap) {
                SimplePolicyCheckResult::Allowed
            } else {
                SimplePolicyCheckResult::Denied(Diagnostic::new(
                    DiagnosticSeverity::Error,
                    format!("required capability {:?} is not available", cap),
                    None,
                ))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SimplePolicyCheckResult {
    Allowed,
    Denied(Diagnostic),
    Unknown,
}

// ---------------------------------------------------------------------------
// Branch-local symbol space
// ---------------------------------------------------------------------------

/// A lightweight symbol scope constructed within a selected branch.
///
/// `BranchLocalSymbolSpace` is **not** a `NamespaceDelta`. Constructing it
/// does not install symbols into the namespace graph. It is a transient,
/// branch-local collection used only by the selected branch's body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BranchLocalSymbolSpace {
    pub parent_snapshot_marker: String,
    pub local_symbols: Vec<BranchLocalSymbol>,
    pub provenance: Provenance,
}

/// One symbol binding available inside a branch-local scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BranchLocalSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub symbol_id: Option<SymbolId>,
    pub provenance: Provenance,
}

impl BranchLocalSymbolSpace {
    pub fn new(parent_snapshot_marker: impl Into<String>, provenance: Provenance) -> Self {
        Self {
            parent_snapshot_marker: parent_snapshot_marker.into(),
            local_symbols: Vec::new(),
            provenance,
        }
    }

    pub fn add_symbol(&mut self, symbol: BranchLocalSymbol) {
        self.local_symbols.push(symbol);
    }
}

/// Look up a symbol by name in a branch-local symbol space.
pub fn lookup_branch_local_symbol(
    space: &BranchLocalSymbolSpace,
    name: &str,
) -> BranchLocalLookupResult {
    let matches: Vec<&BranchLocalSymbol> = space
        .local_symbols
        .iter()
        .filter(|s| s.name == name)
        .collect();
    match matches.len() {
        0 => BranchLocalLookupResult::Missing,
        1 => BranchLocalLookupResult::Found(matches[0].clone()),
        _ => BranchLocalLookupResult::Ambiguous(Diagnostic::new(
            DiagnosticSeverity::Error,
            format!("ambiguous branch-local symbol `{}`", name),
            Some(space.provenance.clone()),
        )),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BranchLocalLookupResult {
    Found(BranchLocalSymbol),
    Missing,
    Ambiguous(Diagnostic),
}

// ---------------------------------------------------------------------------
// Branch arm shapes
// ---------------------------------------------------------------------------

/// Shape-level description of one branch arm in a guarded expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BranchArmShape {
    pub label: String,
    pub local_bindings: Vec<BranchLocalBinding>,
    pub action: BranchActionShape,
    pub type_requirements: Vec<BranchTypeRequirement>,
    pub policy_requirements: Vec<SimplePolicyRequirement>,
    pub provenance: Provenance,
}

/// A type predicate that the selected branch requires to hold.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BranchTypeRequirement {
    pub type_symbol_id: SymbolId,
    pub predicate: SimpleTypePredicate,
    pub must_be_known_true: bool,
    pub provenance: Provenance,
}

/// One local binding introduced by a branch arm's pattern.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BranchLocalBinding {
    pub name: String,
    pub kind: SymbolKind,
    pub provenance: Provenance,
}

/// The action performed by a selected branch arm.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BranchActionShape {
    ReturnValue(EvalResultNormalForm),
    InvokeMeta(MetaInvocationPlanShape),
    BuildLocalSymbol(BranchLocalSymbol),
    Noop,
}

/// A planned meta invocation — bridges to the formal invocation API without
/// executing it directly in the control-flow-local substrate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetaInvocationPlanShape {
    pub description: String,
    pub provenance: Provenance,
}

// ---------------------------------------------------------------------------
// Branch selection
// ---------------------------------------------------------------------------

/// Result of branch-arm selection: exactly the selected arm, or a diagnostic.
#[derive(Clone, Debug)]
pub enum BranchSelectionResult<'a> {
    Selected(&'a BranchArmShape),
    MissingArm(Diagnostic),
    DuplicateArm(Diagnostic),
}

/// Select the branch arm matching the `selected.sum` label from the arm list.
///
/// Only the matching arm label is inspected. Unselected branch arms are not
/// traversed for symbol lookup, policy check, or invocation planning.
pub fn select_branch_arm<'a>(
    selected: &SelectedSumPattern,
    arms: &'a [BranchArmShape],
) -> BranchSelectionResult<'a> {
    let matching: Vec<&'a BranchArmShape> = arms
        .iter()
        .filter(|arm| arm.label == selected.selected_label)
        .collect();

    match matching.len() {
        0 => BranchSelectionResult::MissingArm(Diagnostic::new(
            DiagnosticSeverity::Error,
            format!(
                "no branch arm matches selected label `{}`",
                selected.selected_label
            ),
            Some(selected.provenance.clone()),
        )),
        1 => BranchSelectionResult::Selected(matching[0]),
        _ => BranchSelectionResult::DuplicateArm(Diagnostic::new(
            DiagnosticSeverity::Error,
            format!(
                "duplicate branch arms for label `{}`",
                selected.selected_label
            ),
            Some(selected.provenance.clone()),
        )),
    }
}

/// Validate that no duplicate arm labels exist in the arm list.
/// Returns `Ok(())` if all labels are unique, or a `DuplicateArm`
/// diagnostic if any label repeats.
pub fn validate_branch_arm_labels(arms: &[BranchArmShape]) -> Result<(), Diagnostic> {
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for arm in arms {
        if !seen.insert(&arm.label) {
            return Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                format!("duplicate branch arm label `{}`", arm.label),
                Some(arm.provenance.clone()),
            ));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Control-flow-local evaluation driver
// ---------------------------------------------------------------------------

/// Context for branch-local evaluation: type facts, policy facts, provenance.
#[derive(Clone, Debug)]
pub struct ControlFlowLocalMetaContext<'a> {
    pub type_facts: &'a SimpleTypeFacts,
    pub policy_facts: &'a SimplePolicyFacts,
    pub provenance: Provenance,
}

/// The result of evaluating a guarded branch expression locally.
#[derive(Clone, Debug)]
pub enum ControlFlowLocalEvalResult {
    Selected {
        selected_label: String,
        action: EvaluatedBranchAction,
        local_symbol_space: Option<BranchLocalSymbolSpace>,
        diagnostics: Vec<Diagnostic>,
    },
    Residual {
        reason: GuardResidualReason,
        diagnostics: Vec<Diagnostic>,
    },
    Diagnostic(Diagnostic),
}

/// The concrete action produced by evaluating a selected branch arm.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EvaluatedBranchAction {
    Value(EvalResultNormalForm),
    MetaInvocationPlanned(MetaInvocationPlanShape),
    Noop,
}

/// Evaluate guarded branches: select the matching branch arm, apply type and
/// policy checks **only for the selected branch**, construct a branch-local
/// symbol space if the arm declares local bindings, and return the result.
///
/// **Unselected branches are not traversed.** They contribute no symbol-lookup,
/// policy-check, invocation-planning, or symbol-installation obligations.
pub fn evaluate_guarded_branches(
    selector: &SelectedSumPattern,
    arms: &[BranchArmShape],
    context: &ControlFlowLocalMetaContext<'_>,
) -> ControlFlowLocalEvalResult {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    // Validate selector belongs to space
    if let Err(diag) = selector.validate() {
        return ControlFlowLocalEvalResult::Diagnostic(diag);
    }

    // Validate no duplicate arm labels (structural validation, not semantic)
    if let Err(diag) = validate_branch_arm_labels(arms) {
        return ControlFlowLocalEvalResult::Diagnostic(diag);
    }

    // Select the matching arm — only this arm is inspected
    let selected_arm = match select_branch_arm(selector, arms) {
        BranchSelectionResult::Selected(arm) => arm,
        BranchSelectionResult::MissingArm(diag) => {
            return ControlFlowLocalEvalResult::Residual {
                reason: GuardResidualReason::UnknownSelector,
                diagnostics: vec![diag],
            };
        }
        BranchSelectionResult::DuplicateArm(diag) => {
            return ControlFlowLocalEvalResult::Diagnostic(diag);
        }
    };

    // Check type requirements — only for the selected arm
    for req in &selected_arm.type_requirements {
        let result = check_simple_type_predicate(
            context.type_facts,
            req.type_symbol_id,
            req.predicate.clone(),
        );
        match result {
            SimpleTypeCheckResult::KnownTrue => {
                if !req.must_be_known_true {
                    diagnostics.push(Diagnostic::new(
                        DiagnosticSeverity::Error,
                        format!(
                            "type predicate {:?} is KnownTrue but branch expected false",
                            req.predicate
                        ),
                        Some(req.provenance.clone()),
                    ));
                }
            }
            SimpleTypeCheckResult::KnownFalse => {
                if req.must_be_known_true {
                    diagnostics.push(Diagnostic::new(
                        DiagnosticSeverity::Error,
                        format!(
                            "type predicate {:?} is KnownFalse but branch expected true",
                            req.predicate
                        ),
                        Some(req.provenance.clone()),
                    ));
                }
            }
            SimpleTypeCheckResult::Unknown => {
                return ControlFlowLocalEvalResult::Residual {
                    reason: GuardResidualReason::NonSumSelector,
                    diagnostics: vec![Diagnostic::new(
                        DiagnosticSeverity::Warning,
                        format!(
                            "type predicate {:?} is Unknown — guard residualizes",
                            req.predicate
                        ),
                        Some(req.provenance.clone()),
                    )],
                };
            }
        }
    }

    // Check policy requirements — only for the selected arm
    for req in &selected_arm.policy_requirements {
        match check_simple_policy(context.policy_facts, req) {
            SimplePolicyCheckResult::Allowed => {}
            SimplePolicyCheckResult::Denied(diag) => {
                return ControlFlowLocalEvalResult::Diagnostic(diag);
            }
            SimplePolicyCheckResult::Unknown => {
                diagnostics.push(Diagnostic::new(
                    DiagnosticSeverity::Warning,
                    format!("policy requirement {:?} is Unknown", req),
                    Some(selected_arm.provenance.clone()),
                ));
            }
        }
    }

    // Build branch-local symbol space from the selected arm's bindings AND
    // from any BuildLocalSymbol-encoded symbols.
    let mut local_symbol_count = selected_arm.local_bindings.len();
    if matches!(selected_arm.action, BranchActionShape::BuildLocalSymbol(_)) {
        local_symbol_count += 1;
    }
    let local_space = if local_symbol_count == 0 {
        None
    } else {
        let mut space = BranchLocalSymbolSpace::new("selected_branch", context.provenance.clone());
        for binding in &selected_arm.local_bindings {
            space.add_symbol(BranchLocalSymbol {
                name: binding.name.clone(),
                kind: binding.kind,
                symbol_id: None,
                provenance: binding.provenance.clone(),
            });
        }
        if let BranchActionShape::BuildLocalSymbol(sym) = &selected_arm.action {
            space.add_symbol(sym.clone());
        }
        Some(space)
    };

    // Resolve the selected arm's action
    let action = match &selected_arm.action {
        BranchActionShape::ReturnValue(v) => EvaluatedBranchAction::Value(v.clone()),
        BranchActionShape::InvokeMeta(plan) => {
            EvaluatedBranchAction::MetaInvocationPlanned(plan.clone())
        }
        BranchActionShape::BuildLocalSymbol(_) => EvaluatedBranchAction::Noop,
        BranchActionShape::Noop => EvaluatedBranchAction::Noop,
    };

    ControlFlowLocalEvalResult::Selected {
        selected_label: selector.selected_label.clone(),
        action,
        local_symbol_space: local_space,
        diagnostics,
    }
}
