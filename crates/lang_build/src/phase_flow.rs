use crate::policy_pair::{
    PatternComponentPolicy, Phase, PolicyMode, PolicyResultEntry, PolicyStage, StageSet,
    ValueComponentPolicy,
};

/// A symbol is resolved by identity/path before any phase visibility is
/// considered. A successful result can consequently expose no readable facet
/// in the current phase without becoming an "unresolved symbol".
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
    HiddenInPhase,
    Absent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExposedPolicyEntry<V, P> {
    pub value: FacetView<V>,
    pub value_policy: ValueComponentPolicy,
    pub pattern: FacetView<P>,
    pub pattern_policy: PatternComponentPolicy,
    pub mode: PolicyMode,
    /// A runtime value with an exposed static Pattern/type view can supply its
    /// derived compile companion without consuming the runtime computation.
    pub derived_compile_companion: bool,
}

pub fn expose_policy_slice<V: Clone, P: Clone>(
    entry: &PolicyResultEntry<V, P>,
    phase: Phase,
) -> ExposedPolicyEntry<V, P> {
    let exposed_value_stages = entry.view.pair.value.stages.exposed_at(phase);
    let exposed_pattern_stages = entry.view.pair.pattern.stages.exposed_at(phase);
    let value = match (&entry.value, exposed_value_stages.is_empty()) {
        (None, _) => FacetView::Absent,
        (Some(_), true) => FacetView::HiddenInPhase,
        (Some(value), false) => FacetView::Exposed(value.clone()),
    };
    let pattern = if exposed_pattern_stages.is_empty() {
        FacetView::HiddenInPhase
    } else {
        FacetView::Exposed(entry.pattern.clone())
    };
    let derived_compile_companion = phase != Phase::Runtime
        && entry.view.pair.value.stages.contains(PolicyStage::Runtime)
        && matches!(pattern, FacetView::Exposed(_));

    ExposedPolicyEntry {
        value,
        value_policy: ValueComponentPolicy {
            stages: exposed_value_stages,
            presence: entry.view.pair.value.presence,
        },
        pattern,
        pattern_policy: PatternComponentPolicy {
            stages: exposed_pattern_stages,
        },
        mode: entry.view.mode,
        derived_compile_companion,
    }
}

pub fn read_value<V, P>(entry: &ExposedPolicyEntry<V, P>) -> Option<&V> {
    match &entry.value {
        FacetView::Exposed(value) => Some(value),
        FacetView::HiddenInPhase | FacetView::Absent => None,
    }
}

pub fn read_pattern<V, P>(entry: &ExposedPolicyEntry<V, P>) -> Option<&P> {
    match &entry.pattern {
        FacetView::Exposed(pattern) => Some(pattern),
        FacetView::HiddenInPhase | FacetView::Absent => None,
    }
}

pub fn enumerate_value_facet<V, P>(
    entries: &[ExposedPolicyEntry<V, P>],
) -> impl Iterator<Item = &V> {
    entries.iter().filter_map(read_value)
}

/// Node categories of the complete symbol flow. Projection is structural: it
/// neither enters callable bodies nor performs final overload selection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompleteFlowNode<T> {
    PatternType(T),
    StaticCall(T),
    DerivedCompileCompanion(T),
    StaticSymbolRelation(T),
    DeferredSealTask(T),
    RuntimeValueComputation(T),
    RuntimeBody(T),
    RuntimeBranchValueSelection(T),
    RuntimeEffect(T),
    RuntimeSymbolBinding(T),
    ControlFlow(T),
    Done(T),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CompleteSymbolFlow<T> {
    pub nodes: Vec<CompleteFlowNode<T>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StaticFlow<T> {
    pub nodes: Vec<CompleteFlowNode<T>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeResidualFlow<T> {
    pub nodes: Vec<CompleteFlowNode<T>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectedCompileFlow<T> {
    pub static_flow: StaticFlow<T>,
    pub runtime_residual_flow: RuntimeResidualFlow<T>,
}

pub fn project_complete_symbol_flow<T: Clone>(
    flow: &CompleteSymbolFlow<T>,
) -> ProjectedCompileFlow<T> {
    let mut projected = ProjectedCompileFlow {
        static_flow: StaticFlow { nodes: Vec::new() },
        runtime_residual_flow: RuntimeResidualFlow { nodes: Vec::new() },
    };
    for node in &flow.nodes {
        match node {
            CompleteFlowNode::PatternType(_)
            | CompleteFlowNode::StaticCall(_)
            | CompleteFlowNode::DerivedCompileCompanion(_)
            | CompleteFlowNode::StaticSymbolRelation(_)
            | CompleteFlowNode::DeferredSealTask(_) => {
                projected.static_flow.nodes.push(node.clone());
            }
            CompleteFlowNode::RuntimeValueComputation(_)
            | CompleteFlowNode::RuntimeBody(_)
            | CompleteFlowNode::RuntimeBranchValueSelection(_)
            | CompleteFlowNode::RuntimeEffect(_)
            | CompleteFlowNode::RuntimeSymbolBinding(_) => {
                projected.runtime_residual_flow.nodes.push(node.clone());
            }
            CompleteFlowNode::ControlFlow(_) | CompleteFlowNode::Done(_) => {
                projected.static_flow.nodes.push(node.clone());
                projected.runtime_residual_flow.nodes.push(node.clone());
            }
        }
    }
    projected
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StaticTaskDisposition {
    Ready,
    DeferredToSealStatic,
    FinalStaticError,
}

/// Classify a static task after symbol resolution. OpenStatic may defer only
/// dependencies whose required view exists at SealStatic. SealStatic is the
/// terminal static phase and therefore never creates another deferred stage.
pub fn classify_static_task(required_stages: &StageSet, phase: Phase) -> StaticTaskDisposition {
    if required_stages.visible_at(phase) {
        return StaticTaskDisposition::Ready;
    }
    match phase {
        Phase::OpenStatic if required_stages.visible_at(Phase::SealStatic) => {
            StaticTaskDisposition::DeferredToSealStatic
        }
        Phase::OpenStatic | Phase::SealStatic => StaticTaskDisposition::FinalStaticError,
        Phase::Runtime => StaticTaskDisposition::FinalStaticError,
    }
}
