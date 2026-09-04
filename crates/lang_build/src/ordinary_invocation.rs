//! Connected ordinary function-object invocation trunk.
//!
//! This module is the shared path for source calls and compiler-authorized
//! operations.  Their lookup entrances differ, but both supply semantic target
//! values and then use exactly this sequence:
//!
//! ```text
//! Cluster Symbol
//!   -> sibling vals of the owning cluster                  (C0)
//!   -> target value visibility                             (C1)
//!   -> target value phase view                             (C2)
//!   -> Callable filtering: CallSpace(Type(v)) contains ()  (Cc)
//!   -> resolve associated call entries from the exact tau  (C3)
//!   -> hard applicability                                  (A)
//!   -> optional fallback suppression                       (Af)
//!   -> one Bp' product comparison
//!   -> B1/B2
//!   -> Pattern extraction specificity                      (B3)
//!   -> B4/B5/B6
//!   -> unique candidate
//!   -> DynamicLegality (capability / Place / lifecycle Pre)
//!   -> InvocationFrame
//!   -> ordinary body
//!   -> complete ordinary result
//! ```
//!
//! The currently implemented B1/B2/B4/B5/B6 dimensions are identities.  They
//! are kept as explicit trace boundaries rather than invented ranking rules.

use std::collections::BTreeMap;

use lang_syntax::{
    NormClosureBody, NormForm, NormOverloadStrategy, NormPattern, NormPatternElem, NormPolicySpec,
};

use crate::{
    body_entry_allows_execution,
    identity::{SemanticValueId, TypeValueId},
    invocation_frame::{
        InvocationCallableRef, InvocationExecutionEnv, InvocationFrame, InvocationLookupEnv,
        SelfPosition,
    },
    meta_invocation::{MetaExecutionMaterial, MetaInvocationInput},
    model::{
        Diagnostic, ExecutionEnv, PolicyEnv, Provenance, ResolverCode, SourceCategory, SymbolId,
        SymbolKind, SymbolObject,
    },
    overload_pattern::{overload_args_from_classified_shape, SpecificityTuple},
    overload_set::{
        applicable_candidate_from_closure, evaluate_selected_source_body, ApplicableCandidate,
        CandidateApplicabilityFailure, SelectedSourceBody, SourceBodyEvaluationFailure,
        VisibilityView,
    },
    policy_migration::{
        compare_migration_endpoint_coordinates, project_migration_input_endpoint,
        project_migration_output_endpoint, PolicyMigrationRequest, PolicyPartialOrdering,
        SemanticValueRef,
    },
    policy_overload::{
        maximal_candidates, policy_mode_preference_rank, PolicyActualFrame, PolicyFormalFrame,
    },
    policy_pair::{
        elaborate_explicit_p1, elaborate_formal_policy_pattern, project_p1, CapabilityRealization,
        ExplicitP1Position, OutputModeDemand, P1Projection, PatternComponentPolicy, Phase,
        PolicyMode, PolicyPair, PolicyResultEntry, PolicyStage, PolicyView, ResultPolicyDemand,
        ValueComponentPolicy,
    },
    product_shape::{
        ArgProductShape, FlattenedProductInvariant, FlattenedProductObject, ProductAtom,
        ProductMaterialRole, RawArgValueClass,
    },
    semantic_name_index::ResolverContext,
    semantic_owner::{SemanticOwnerId, SemanticSymbolIdentity},
    semantic_world::{
        ObjectPlaceId, OrdinaryCallEntry, OrdinaryCandidateRole, PatternValueId,
        SemanticValuePayload, SemanticWorld, WritableContext,
    },
    type_argument::{classify_type_arguments_env_with_report, SemanticTypeEnv, TypeResolutionEnv},
    DeclaredResultClass, InvocationResidual, NormalizedCallSite,
};

#[derive(Clone, Copy, Debug)]
pub struct MigrationInvocationContext<'a> {
    pub request: &'a PolicyMigrationRequest,
    /// The source value is bound as the first explicit argument (slot 1) in
    /// the invocation frame.  Slot 0 is the receiver (`self`), which is bound
    /// separately by the associated-call machinery — see `semantic_owner.rs`.
    pub source_value: SemanticValueId,
}

#[derive(Clone, Debug)]
pub struct OrdinaryInvocationContext<'a> {
    pub policy_env: PolicyEnv,
    pub execution_env: ExecutionEnv,
    pub phase: Phase,
    pub caller_mode: PolicyMode,
    pub explicit_argument_modes: &'a [PolicyMode],
    /// Total before candidate maxima. Pair/stage coordinates are hard
    /// admissibility; the concrete mode coordinate participates in Bp.
    pub result_policy_demand: ResultPolicyDemand,
    pub visibility: VisibilityView,
    pub migration: Option<MigrationInvocationContext<'a>>,
    /// Exact complete target Type for an authorized type-changing
    /// construction. This supplies execution material only after the target
    /// snapshot itself has enumerated the candidate family.
    pub construction_target: Option<&'a crate::CompleteTypeValue>,
    /// Post-selection capability/place demand. This coordinate never
    /// participates in candidate ordering and therefore cannot reopen maxima.
    pub dynamic_legality: DynamicLegalityDemand<'a>,
    /// Semantic owner of the declaration environment that constructed
    /// results attach to.  When the declaration sits
    /// inside a callable body this must be the innermost enclosing
    /// anonymous function object's Self scope owner — owner identity is
    /// parent-linked, so this single node carries the whole Self chain.
    /// Only a top-level declaration supplies its namespace-level owner.
    pub ambient_construction_owner: Option<SemanticOwnerId>,
}

impl<'a> OrdinaryInvocationContext<'a> {
    pub fn open_static(explicit_argument_modes: &'a [PolicyMode]) -> Self {
        Self {
            policy_env: PolicyEnv::OpenStatic,
            execution_env: ExecutionEnv::OpenStatic,
            phase: Phase::OpenStatic,
            caller_mode: PolicyMode::Plain,
            explicit_argument_modes,
            result_policy_demand: ResultPolicyDemand::default(),
            visibility: VisibilityView::Internal,
            migration: None,
            construction_target: None,
            dynamic_legality: DynamicLegalityDemand::default(),
            ambient_construction_owner: None,
        }
    }

    /// Supply the declaration-environment owner (Self scope chain node or
    /// namespace-level owner) that ambient constructions root under.
    pub fn with_ambient_construction_owner(mut self, owner: SemanticOwnerId) -> Self {
        self.ambient_construction_owner = Some(owner);
        self
    }

    pub fn with_result_policy_demand(mut self, demand: ResultPolicyDemand) -> Self {
        self.result_policy_demand = demand;
        self
    }

    pub fn with_construction_target(mut self, target: &'a crate::CompleteTypeValue) -> Self {
        self.construction_target = Some(target);
        self
    }

    pub fn with_capability_demand(mut self, input: PolicyMode, output: PolicyMode) -> Self {
        self.dynamic_legality.capability = Some((input, output));
        self
    }

    pub fn requiring_target_writable(mut self, writable: &'a WritableContext) -> Self {
        self.dynamic_legality.require_target_writable = true;
        self.dynamic_legality.writable = Some(writable);
        self
    }

    /// Add continuation-relative lifecycle Pre facts. They are deliberately
    /// attached to DynamicLegality, after unique selection, so failure seals
    /// the selected invocation instead of reopening overload resolution.
    pub fn with_lifecycle_preconditions(
        mut self,
        lifecycle: &'a crate::LifecycleValidationContext,
    ) -> Self {
        self.dynamic_legality.lifecycle = Some(lifecycle);
        self
    }
}

/// Context-indexed facts checked only after a unique candidate has been
/// selected. Absence of a demand means the current operation does not require
/// that capability; it is not an implicit grant.
#[derive(Clone, Copy, Debug, Default)]
pub struct DynamicLegalityDemand<'a> {
    pub capability: Option<(PolicyMode, PolicyMode)>,
    pub require_target_writable: bool,
    pub writable: Option<&'a WritableContext>,
    pub lifecycle: Option<&'a crate::LifecycleValidationContext>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrdinaryCandidateOrigin {
    SourceSymbol(SemanticSymbolIdentity),
    PatternAssociatedCallEntry(PatternValueId),
    PatternAssociatedValue(PatternValueId),
}

#[derive(Clone, Debug)]
pub struct PreparedCallCandidate {
    pub origin: OrdinaryCandidateOrigin,
    pub target_value: SemanticValueId,
    /// Horizontal residency supplied by this invocation context. Pure value
    /// projection does not require one; Place-sensitive legality does.
    pub target_place: Option<ObjectPlaceId>,
    pub call_entry_value: SemanticValueId,
    pub backing_declaration: SymbolId,
    pub frame: InvocationFrame,
    /// Declaration-local P2 used for body entry and inherited parameter
    /// position policy. Caller demand never mutates this view.
    pub body_entry_view: PolicyView,
    /// Producer result P2 exposed across the call boundary and used by
    /// `ResultPolicyDemand`. The declaration-local P_out is stored separately
    /// on the call entry and never replaces this producer coordinate.
    pub complete_result_view: PolicyView,
    /// P1 of the function object — the single policy degree of freedom.
    /// P1(function object) = P1(slot0/self) = P1(let ()).
    /// If omitted, P1 is derived from P2.
    pub function_object_view: PolicyView,
    pub capability_realization: CapabilityRealization,
    pub formal_policy_frame: PolicyFormalFrame,
    pub(crate) source_shape: Option<ApplicableCandidate>,
    pub(crate) core_invocation: Option<MetaInvocationInput>,
    pub(crate) intrinsic_body: Option<crate::semantic_world::OrdinaryIntrinsicBody>,
    pub declared_result_class: DeclaredResultClass,
    pub candidate_role: OrdinaryCandidateRole,
    pub overload_strategy: NormOverloadStrategy,
    /// Migration input endpoint projected from the first explicit Product
    /// formal after slot 0. It is computed once at A-stage so admissibility and
    /// Bp' preference observe the same source-formal coordinate.
    pub migration_input_endpoint: Option<PolicyPair>,
    /// Migration output endpoint projected from the selected callable's
    /// canonical P1 (`callable_value_policy`). It is computed once at A-stage
    /// alongside the input endpoint.
    pub migration_output_endpoint: Option<PolicyPair>,
}

impl PreparedCallCandidate {
    pub fn specificity(&self) -> SpecificityTuple {
        self.source_shape
            .as_ref()
            .map(|source| source.specificity)
            .unwrap_or_default()
    }

    pub fn is_delete(&self) -> bool {
        self.source_shape.as_ref().is_some_and(|source| {
            matches!(
                source.source_callable.closure.body,
                lang_syntax::NormClosureBody::Delete(_)
            )
        }) || matches!(
            self.intrinsic_body,
            Some(crate::semantic_world::OrdinaryIntrinsicBody::Delete)
        )
    }

    /// Proof-relevant Pattern applicability result for source candidates.
    /// Core/compiler candidates have no source Pattern query in this slice.
    pub fn pattern_applicability(&self) -> Option<&crate::PatternApplicabilityProof> {
        self.source_shape
            .as_ref()
            .map(|source| &source.pattern_proof)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DynamicLegalityProof {
    pub selected_call_entry: SemanticValueId,
    pub phase: Phase,
    pub execution_env: ExecutionEnv,
    pub capability_cell: Option<crate::CapabilityRealizationCell>,
    pub writable_place: Option<ObjectPlaceId>,
    pub lifecycle: Option<crate::LifecycleValidationProof>,
}

/// Unique selection together with post-selection legality evidence. Execution
/// receives this sealed carrier, never the discarded candidate family.
#[derive(Clone, Debug)]
pub struct SealedSelectedInvocation {
    pub candidate: PreparedCallCandidate,
    pub legality: DynamicLegalityProof,
}

impl std::ops::Deref for SealedSelectedInvocation {
    type Target = PreparedCallCandidate;

    fn deref(&self) -> &Self::Target {
        &self.candidate
    }
}

fn validate_dynamic_legality(
    _semantic_world: &SemanticWorld,
    selected: &PreparedCallCandidate,
    context: &OrdinaryInvocationContext<'_>,
    provenance: &Provenance,
) -> Result<DynamicLegalityProof, Diagnostic> {
    let capability_cell = context
        .dynamic_legality
        .capability
        .map(|(input, output)| selected.capability_realization.cell(input, output));
    if let Some(cell) = capability_cell {
        match cell {
            crate::CapabilityRealizationCell::Default
            | crate::CapabilityRealizationCell::Custom => {}
            crate::CapabilityRealizationCell::Absent => {
                return Err(Diagnostic::hard_error(
                    "selected invocation has no capability realization for the demanded input/output Policy cell",
                    Some(provenance.clone()),
                ));
            }
            crate::CapabilityRealizationCell::Delete => {
                return Err(Diagnostic::hard_error(
                    "selected invocation deletes the demanded input/output capability realization",
                    Some(provenance.clone()),
                ));
            }
        }
    }

    let writable_place = if context.dynamic_legality.require_target_writable {
        let place = selected.target_place.ok_or_else(|| {
            Diagnostic::hard_error(
                "selected invocation requires an actual target Place",
                Some(provenance.clone()),
            )
        })?;
        let writable = context.dynamic_legality.writable.ok_or_else(|| {
            Diagnostic::hard_error(
                "selected invocation requires Writable but the evaluation context supplies no write authority",
                Some(provenance.clone()),
            )
        })?;
        if !writable.place_is_writable(place) {
            return Err(Diagnostic::hard_error(
                "selected invocation target Place is not Writable in this evaluation context",
                Some(provenance.clone()),
            ));
        }
        Some(place)
    } else {
        None
    };

    let lifecycle = context
        .dynamic_legality
        .lifecycle
        .map(|lifecycle| lifecycle.validate_pre(provenance))
        .transpose()?;

    Ok(DynamicLegalityProof {
        selected_call_entry: selected.call_entry_value,
        phase: context.phase,
        execution_env: context.execution_env,
        capability_cell,
        writable_place,
        lifecycle,
    })
}

/// A sibling val that has been verified callable by the Cc stage.
///
/// Keeps the sibling value and its resolved call entries distinct so that
/// overload enumeration and call-entry resolution are visibly separate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallableTarget {
    pub sibling_value: SemanticValueId,
    pub call_entries: Vec<SemanticValueId>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OrdinaryPipelineTrace {
    pub c0_target_values: Vec<SemanticValueId>,
    pub c1_visible_values: Vec<SemanticValueId>,
    pub c2_phase_values: Vec<SemanticValueId>,
    /// Explicit Callable filter result: only values v for which
    /// (v |> type).Val2 contains `()`.
    pub callable_values: Vec<SemanticValueId>,
    pub c3_call_entries: Vec<SemanticValueId>,
    pub a_fully_admissible: Vec<SemanticValueId>,
    pub af_after_fallback: Vec<SemanticValueId>,
    pub bp_prime: Vec<SemanticValueId>,
    pub b3_pattern_specific: Vec<SemanticValueId>,
    pub selected: Option<SemanticValueId>,
    pub dynamic_legality: Option<DynamicLegalityProof>,
}

#[derive(Clone, Debug)]
pub struct SingleMemberResult {
    pub selected: SealedSelectedInvocation,
    pub returned: ReturnedSemanticEntity,
    /// Exact semantic complete-type result, present iff the selected
    /// declaration's result class is `CompleteType`.  Binding consumers use
    /// this field directly; they never infer a type result or recover tau from
    /// a `CoreTypeProjection` graph payload.
    pub complete_type: Option<crate::CompleteTypeValue>,
    /// Complete result view returned by the ordinary callable, including its
    /// P2 type/Pattern observations, before any consumer-specific
    /// `Project_out`. This is
    /// NOT the outward exposure policy of the invocation result: outward
    /// visibility is the canonical P1 layer (`exposed()`).
    pub complete_result: Vec<PolicyResultEntry<SemanticValueRef, PatternValueId>>,
    pub trace: OrdinaryPipelineTrace,
}

impl SingleMemberResult {
    /// The outward exposure layer of this invocation result.  Derived, not
    /// stored: `outward_policy` is always the canonical P1 of the selected
    /// callable (`function_object_p1`), so the two layers can never drift
    /// apart and no third output authority can hide in between.
    ///
    /// This is the semantic boundary of the ordinary
    /// binding path, not a bypass query:
    ///
    /// ```text
    /// CompleteResultView(P2) -> expose under callable P1 -> outer binding P1
    /// ```
    pub fn exposed(&self) -> ExposedInvocationResult {
        ExposedInvocationResult::expose(
            self.selected.function_object_view.pair.clone(),
            &self.complete_result,
        )
    }
}

/// Outward exposure layer of an ordinary invocation result.
///
/// The result is split into two semantic layers instead
/// of letting one `PolicyResultEntry` field mean three things at once:
///
/// ```text
/// CompleteResultView      = P2 type/Pattern observations
/// ExposedInvocationResult = outward_policy (canonical P1)
///                           + material (the completed result entries)
/// ```
///
/// The outward visibility of an invocation result is the canonical P1 —
/// the same single output authority as the migration output endpoint —
/// while P2 keeps only input/result compatibility and the complete result
/// type/pattern domain.  Cluster member views carry each member's own
/// Policy and are a third, separate coordinate.
#[derive(Clone, Debug)]
pub struct ExposedInvocationResult {
    /// Canonical P1 of the selected callable: the invocation result's outward
    /// visibility policy.
    pub outward_policy: PolicyPair,
    /// The completed result material (the P2-domain entries), exposed
    /// under the callable P1 window.
    pub material: Vec<PolicyResultEntry<SemanticValueRef, PatternValueId>>,
}

impl ExposedInvocationResult {
    /// `CompleteResultView(P2) -> expose under callable P1`.
    ///
    /// Every entry's stage / Policy-mode window is intersected with the
    /// callable's canonical P1 before any consumer sees it; entries whose
    /// exposed window vanishes are not part of the outward result at all.
    /// When the canonical P1 is the P2 derivation (no explicit P1 written
    /// anywhere), the window is a superset of the material and exposure is
    /// an identity.
    pub fn expose(
        outward_policy: PolicyPair,
        complete_result: &[PolicyResultEntry<SemanticValueRef, PatternValueId>],
    ) -> Self {
        let material = complete_result
            .iter()
            .filter_map(|entry| expose_result_entry(&outward_policy, entry))
            .collect();
        Self {
            outward_policy,
            material,
        }
    }
}

/// Expose one complete-result entry under the callable P1 window.
///
/// Window rules mirror `project_p1`'s slice restriction (`restrict_stages`
/// is shared): stage sets are intersected, an empty window facet stays
/// unconstrained, and a facet whose non-empty intersection vanishes hides
/// the entry. Whole-slot mode remains the independent concrete coordinate
/// already stored on `PolicyView`; it is never inferred from this pair.
fn expose_result_entry(
    outward: &PolicyPair,
    entry: &PolicyResultEntry<SemanticValueRef, PatternValueId>,
) -> Option<PolicyResultEntry<SemanticValueRef, PatternValueId>> {
    let pattern_stages = crate::policy_pair::restrict_stages(
        &outward.pattern.stages,
        &entry.view.pair.pattern.stages,
    )?;
    let value_policy = if entry.value.is_some() {
        let stages = crate::policy_pair::restrict_stages(
            &outward.value.stages,
            &entry.view.pair.value.stages,
        )?;
        ValueComponentPolicy {
            stages,
            presence: entry.view.pair.value.presence,
        }
    } else {
        // Pure-P entry: the recorded static source stages are still
        // clipped to the window so a later materialization cannot exceed
        // the exposed view, but a pure-P entry is carried by its Pattern
        // facet and is not hidden by an empty value window.
        ValueComponentPolicy {
            stages: crate::policy_pair::restrict_stages(
                &outward.value.stages,
                &entry.view.pair.value.stages,
            )
            .unwrap_or_default(),
            presence: entry.view.pair.value.presence,
        }
    };
    Some(PolicyResultEntry {
        value: entry.value.clone(),
        pattern: entry.pattern,
        view: PolicyView {
            pair: PolicyPair {
                value: value_policy,
                pattern: PatternComponentPolicy {
                    stages: pattern_stages,
                },
            },
            mode: entry.view.mode,
        },
    })
}

/// Result of an invocation declaring `ClusterSymbol`: a completed Symbol
/// cluster construction (plural values under one name at one position).
#[derive(Clone, Debug)]
pub struct ClusterSymbolResult {
    pub construction: crate::ClusterConstructionMaterial,
    /// Struct construction materials backing the construction's self-rooted
    /// type members, in member order. The binding side uses these to
    /// expand the field-function and ref/share projection namespaces instead
    /// of a bare bound-type-value carrier. Forwarded members contribute no
    /// entry here.
    pub struct_materials: Vec<crate::StructConstructionMaterial>,
    /// The complete result P2 of the selected callable.  Carried per
    /// result class. This field keeps the independent Policy coordinate
    /// alongside it.
    pub result_p2: PolicyPair,
    pub trace: OrdinaryPipelineTrace,
}

/// Result of an invocation declaring `Unit` (`_: unit`). This is a value-less
/// result. Reserved carrier — no executable producer exists yet; the
/// declaration level validates the class and invocation reports the
/// execution gap explicitly.
#[derive(Clone, Debug)]
pub struct UnitInvocationResult {
    pub result_p2: PolicyPair,
    pub trace: OrdinaryPipelineTrace,
}

/// Projection transport carried inside the unified [`InvocationResult`]
/// success branch.
///
/// These variants preserve class-specific installation data; they do not
/// decide the semantic result class. That authority belongs exclusively to
/// `InvocationResult::SemanticResult.declared_result_class`, derived once
/// from the selected callable's declaration.
#[derive(Clone, Debug)]
pub enum ProjectedInvocationOutcome {
    Unit(UnitInvocationResult),
    SingleMember(SingleMemberResult),
    ClusterSymbol(ClusterSymbolResult),
}

/// Unified ordinary invocation boundary. Selection/admissibility failures
/// remain the outer `OrdinaryInvocationFailure` channel because no callable
/// `F` exists yet; once `F` is selected, its result crosses this envelope.
pub type InvocationOutcome = crate::InvocationResult<ProjectedInvocationOutcome>;

fn semantic_invocation_outcome(
    declared_result_class: DeclaredResultClass,
    projection: ProjectedInvocationOutcome,
) -> InvocationOutcome {
    crate::InvocationResult::semantic(declared_result_class, projection)
}

/// Complete type value returned by a world-connected invocation.
///
/// `construction_material` is replay/install material for graph replay
/// binding and namespace projection.  Semantic consumers use
/// `complete_type`; the material never defines type identity or equality.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReturnedCompleteType {
    pub complete_type: crate::CompleteTypeValue,
    pub carrier_value: SemanticValueId,
    pub pattern: PatternValueId,
    pub construction_material: Option<crate::StructConstructionMaterial>,
}

#[derive(Clone, Debug)]
pub struct PolicyMigrationResult {
    pub invocation: SingleMemberResult,
    pub demanded_view: Vec<PolicyResultEntry<SemanticValueRef, PatternValueId>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReturnedSemanticEntity {
    CompleteType(ReturnedCompleteType),
    OrdinaryValue(SemanticValueId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SelectedBodyOutput {
    Material(MetaExecutionMaterial),
    OrdinaryValue(SemanticValueId),
}

#[derive(Clone, Debug)]
pub enum OrdinaryInvocationFailure {
    NoTargetValues {
        trace: OrdinaryPipelineTrace,
    },
    NoFullyAdmissibleCandidate {
        first_diagnostic: Option<Diagnostic>,
        trace: OrdinaryPipelineTrace,
    },
    /// The canonical A-stage relation could not answer a query. Candidate
    /// enumeration is incomplete, so selection terminates before maxima.
    ApplicabilityUnsupported {
        diagnostic: Diagnostic,
        trace: OrdinaryPipelineTrace,
    },
    Ambiguous {
        candidates: Vec<SemanticValueId>,
        trace: OrdinaryPipelineTrace,
    },
    DynamicLegality {
        selected: SemanticValueId,
        diagnostic: Diagnostic,
        trace: OrdinaryPipelineTrace,
    },
    Residual {
        residual: InvocationResidual,
        trace: OrdinaryPipelineTrace,
    },
    /// Argument normalization hit an illegal cyclic Val2: Val2 normalization
    /// is well-founded finite recursion, so an object reached again while its
    /// own Val2 is still being normalized has no normal form and the
    /// invocation is rejected before any instance key exists.
    CyclicVal2 {
        diagnostic: Diagnostic,
        trace: OrdinaryPipelineTrace,
    },
    SelectedDelete {
        selected: SemanticValueId,
        diagnostic: Diagnostic,
        trace: OrdinaryPipelineTrace,
    },
    SelectedBody {
        failure: SourceBodyEvaluationFailure,
        trace: OrdinaryPipelineTrace,
    },
    SelectedCoreBody {
        diagnostic: Diagnostic,
        trace: OrdinaryPipelineTrace,
    },
    /// Meta-return self-root enforcement.  The unique type
    /// member of a meta invocation's cluster must be rooted at the meta
    /// function itself plus its normalized input arguments
    /// (`MetaTypeRoot = MetaFunctionIdentity + Normalize(Arguments)`).
    /// Forwarding an existing type root out of the body is a hard
    /// diagnostic; no automatic re-rooting or wrapper construction is
    /// performed.
    MetaReturnTypeRootMismatch {
        diagnostic: Diagnostic,
        trace: OrdinaryPipelineTrace,
    },
    ResultTypeHasNoPattern {
        type_value: TypeValueId,
        trace: OrdinaryPipelineTrace,
    },
    MigrationResultTypeChanged {
        source: TypeValueId,
        result: TypeValueId,
        trace: OrdinaryPipelineTrace,
    },
    MigrationOutputProjectionFailed {
        trace: OrdinaryPipelineTrace,
    },
}

/// Hard-error message for a second direct `struct` generation of the same
/// normalized navigation shape at the same declaration level.  The recorded
/// binder never feeds type identity; it only makes the guidance ergonomic:
/// the first generation is source-visible at this level, so the collision
/// points at its existing binding.
fn ambient_struct_collision_message(binder: Option<&crate::AmbientTypeBinder>) -> String {
    let base = "ambient struct collision: a type with the same navigation name and top pattern \
                was already generated by `struct` at this level";
    match binder {
        Some(crate::AmbientTypeBinder::WholeSymbol(name)) => format!(
            "{base}; its value is bound to symbol `{name}` here — use the existing `{name}` to \
             continue the type computation"
        ),
        Some(crate::AmbientTypeBinder::ExtractionMembers(names)) => {
            let members = names
                .iter()
                .map(|name| format!("`{name}`"))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "{base}; its extraction binding exposes the member symbols {members} — use \
                 these extracted symbols to continue the type computation"
            )
        }
        Some(crate::AmbientTypeBinder::CallableParameter(name)) => format!(
            "{base}; its value was only bound to the callable parameter `{name}`, which lives \
             one level below this declaration environment and is not visible here — first bind \
             the temporary value to a symbol at this level (e.g. `let t = ... struct;`), then \
             use that symbol to continue the type computation"
        ),
        None => format!("{base}; use its existing binding to continue the type computation"),
    }
}

/// Attach `Addr(Norm_type)` observations to the candidate's classified type
/// arguments before crossing the formal invocation boundary.
///
/// Candidate preparation runs behind an immutable `SemanticTypeEnv` and
/// cannot intern observation addresses; the world-connected invoke sites are
/// the earliest point with a `&mut SemanticWorld` channel.  A cyclic Val2
/// surfaces the same rejection as canonical instance-key normalization.
fn attach_candidate_type_observations(
    semantic_world: &mut SemanticWorld,
    input: &mut crate::MetaInvocationInput,
    trace: &OrdinaryPipelineTrace,
) -> Result<(), OrdinaryInvocationFailure> {
    let shape = &mut input.candidate.arg_product_shape;
    semantic_world
        .attach_canonical_type_observations(&mut shape.raw_args, &shape.flattened.atoms)
        .map_err(|diagnostic| OrdinaryInvocationFailure::CyclicVal2 {
            diagnostic,
            trace: trace.clone(),
        })
}

/// Form an ordinary MetaInstance key only for an invocation whose declared
/// result/owner rule actually establishes a MetaInstance.  Ordinary value
/// forwarding, same-Type migration, and privileged `struct` do not acquire a
/// hidden meta identity merely because they reuse the invocation trunk.
fn canonical_meta_instance_key_for_selected(
    semantic_world: &mut SemanticWorld,
    shape: &ArgProductShape,
    callable: crate::MetaCallableIdentity,
    provenance: &Provenance,
    trace: &OrdinaryPipelineTrace,
) -> Result<crate::MetaInvocationMaterialKey, OrdinaryInvocationFailure> {
    let arguments_product_addr = semantic_world
        .canonical_arguments_product_address(&shape.raw_args, &shape.flattened.atoms)
        .map_err(|diagnostic| OrdinaryInvocationFailure::CyclicVal2 {
            diagnostic,
            trace: trace.clone(),
        })?;
    Ok(crate::compute_meta_invocation_material_key(
        callable,
        arguments_product_addr,
        provenance.clone(),
    ))
}

pub fn invoke_policy_migration(
    semantic_world: &mut SemanticWorld,
    request: &PolicyMigrationRequest,
    resolver_context: &ResolverContext,
) -> Result<PolicyMigrationResult, OrdinaryInvocationFailure> {
    let Some(source) = semantic_world.value(request.source_value()).cloned() else {
        return Err(OrdinaryInvocationFailure::NoTargetValues {
            trace: OrdinaryPipelineTrace::default(),
        });
    };
    if source.type_value != request.source_type() {
        return Err(OrdinaryInvocationFailure::MigrationResultTypeChanged {
            source: request.source_type(),
            result: source.type_value,
            trace: OrdinaryPipelineTrace::default(),
        });
    }
    let migration_args = ArgProductShape::from_flattened(FlattenedProductObject {
        atoms: vec![ProductAtom::SemanticValue {
            value: request.source_value(),
            type_value: request.source_type(),
            mode: request.source_view().mode,
            provenance: request.provenance().clone(),
        }],
        provenance: request.provenance().clone(),
        invariant: FlattenedProductInvariant {
            no_direct_product_atom_remains: true,
        },
    });

    // Follow PatternValue → OwnerCluster → member views, not
    // Pattern → P.Val2["()"].  Transport members are sibling vals
    // of the owning cluster, not associated Val2 of the pure P.
    let cluster_owner = semantic_world
        .owner_cluster(source.pattern)
        .ok_or_else(|| OrdinaryInvocationFailure::NoTargetValues {
            trace: OrdinaryPipelineTrace::default(),
        })?;
    let cluster =
        cluster_owner
            .installed()
            .ok_or_else(|| OrdinaryInvocationFailure::NoTargetValues {
                trace: OrdinaryPipelineTrace::default(),
            })?;
    let target_members = semantic_world
        .symbol(cluster)
        .map(|cell| cell.member_views.clone())
        .unwrap_or_default();
    let target_places = semantic_world.binding_places(cluster);

    let no_explicit_modes = [];
    let trace = OrdinaryPipelineTrace {
        c0_target_values: target_members
            .iter()
            .filter_map(|view| view.value)
            .collect(),
        ..OrdinaryPipelineTrace::default()
    };
    if target_members.iter().all(|view| view.value.is_none()) {
        return Err(OrdinaryInvocationFailure::NoTargetValues { trace });
    }

    let invocation = invoke_target_values(
        semantic_world,
        OrdinaryCandidateOrigin::SourceSymbol(cluster),
        target_members,
        target_places,
        None,
        None,
        migration_args,
        resolver_context,
        OrdinaryInvocationContext {
            policy_env: PolicyEnv::OpenStatic,
            execution_env: ExecutionEnv::OpenStatic,
            phase: Phase::OpenStatic,
            caller_mode: PolicyMode::Plain,
            explicit_argument_modes: &no_explicit_modes,
            result_policy_demand: request.target_demand().clone(),
            visibility: VisibilityView::Internal,
            migration: Some(MigrationInvocationContext {
                request,
                source_value: source.id,
            }),
            construction_target: None,
            dynamic_legality: DynamicLegalityDemand::default(),
            // Migration transport never performs an ambient struct
            // construction; no declaration-environment owner applies.
            ambient_construction_owner: None,
        },
        request.provenance().clone(),
    )?;
    let crate::InvocationResult::SemanticResult {
        declared_result_class: crate::DeclaredResultClass::OrdinaryValue,
        value: ProjectedInvocationOutcome::SingleMember(invocation),
    } = invocation
    else {
        return Err(OrdinaryInvocationFailure::NoFullyAdmissibleCandidate {
            first_diagnostic: Some(Diagnostic::hard_error(
                "migration selected a non-ordinary result class",
                Some(request.provenance().clone()),
            )),
            trace: OrdinaryPipelineTrace::default(),
        });
    };
    let mut demanded_view = project_p1(
        &request.target_demand().pair_query,
        &invocation.complete_result,
    );
    if demanded_view.is_empty() {
        return Err(OrdinaryInvocationFailure::MigrationOutputProjectionFailed {
            trace: invocation.trace,
        });
    }
    let realized_mode = invocation.selected.complete_result_view.mode;
    for entry in &mut demanded_view {
        entry.view.mode = realized_mode;
    }
    Ok(PolicyMigrationResult {
        invocation,
        demanded_view,
    })
}

/// Invoke all value members of one resolved semantic Symbol.
///
/// C0 is the resolved ClusterSymbol's canonical member views — not a flat
/// value-id list.  Pure-P members (value = None) stay legal cluster members
/// but are not invocation candidates; exposure and callability are decided
/// per member view downstream.
pub fn invoke_symbol_ordinary(
    semantic_world: &mut SemanticWorld,
    symbol: SemanticSymbolIdentity,
    call_site: &NormalizedCallSite,
    resolver_context: &ResolverContext,
    context: OrdinaryInvocationContext<'_>,
    provenance: Provenance,
) -> Result<InvocationOutcome, OrdinaryInvocationFailure> {
    invoke_host_member_symbol_ordinary(
        semantic_world,
        &[],
        symbol,
        call_site,
        resolver_context,
        context,
        provenance,
    )
}

/// Invoke one resolved semantic Symbol reached through an explicit host chain.
///
/// Exposure of a navigated target composes per layer and per phase over the
/// WHOLE chain the navigator stepped through:
///
/// ```text
/// Expose(g::f::T, φ) = Expose(T, φ) ∧ Expose(f, φ) ∧ Expose(g_member, φ)
/// ```
///
/// The member factor is the per-member exposure stage decided downstream in
/// this pipeline; the host factors are decided here, each from that host
/// carrier's own binding-level pure-P member view.  A single host anywhere in
/// the chain that is not navigable at this phase hides everything reached
/// through it, so the whole chain must be exposed; the failure is reported as
/// `NoTargetValues` for the already resolved Symbol. Name resolution is sealed
/// before this projection, so the failure never resumes an outward scope walk.
/// A bare-name target has an empty host chain and composes only the member
/// factor.
#[allow(clippy::too_many_arguments)]
pub fn invoke_host_member_symbol_ordinary(
    semantic_world: &mut SemanticWorld,
    hosts: &[crate::PatternHostMember],
    symbol: SemanticSymbolIdentity,
    call_site: &NormalizedCallSite,
    resolver_context: &ResolverContext,
    context: OrdinaryInvocationContext<'_>,
    provenance: Provenance,
) -> Result<InvocationOutcome, OrdinaryInvocationFailure> {
    if hosts.iter().any(|host| !host.exposed_at(context.phase)) {
        return Err(OrdinaryInvocationFailure::NoTargetValues {
            trace: OrdinaryPipelineTrace::default(),
        });
    }
    let target_members = semantic_world
        .symbol(symbol)
        .map(|symbol| symbol.member_views.clone())
        .unwrap_or_default();
    let target_places = semantic_world.binding_places(symbol);
    invoke_target_values(
        semantic_world,
        OrdinaryCandidateOrigin::SourceSymbol(symbol),
        target_members,
        target_places,
        None,
        Some(call_site),
        call_site.to_arg_product_shape(ProductMaterialRole::CallableArgumentProduct),
        resolver_context,
        context,
        provenance,
    )
}

/// Invoke an authorized operation family reached from an existing
/// PatternValue's owner.  No source path or TypeValue reverse lookup is
/// fabricated.
pub fn invoke_pattern_associated_ordinary(
    semantic_world: &mut SemanticWorld,
    pattern: PatternValueId,
    operation_name: &str,
    receiver_value: SemanticValueId,
    explicit_arg_product: ArgProductShape,
    resolver_context: &ResolverContext,
    context: OrdinaryInvocationContext<'_>,
    provenance: Provenance,
) -> Result<InvocationOutcome, OrdinaryInvocationFailure> {
    let target_members =
        semantic_world.associated_member_views_for_pattern(pattern, operation_name, context.phase);
    invoke_target_values(
        semantic_world,
        OrdinaryCandidateOrigin::PatternAssociatedCallEntry(pattern),
        target_members,
        BTreeMap::new(),
        Some(receiver_value),
        None,
        explicit_arg_product,
        resolver_context,
        context,
        provenance,
    )
}

/// Invoke a named ordinary value member obtained from a Pattern owner's
/// associated Val2. The authorized receiver is inserted as the first explicit
/// argument; slot 0 remains the selected function object, exactly as for a
/// source-visible named function.
#[allow(clippy::too_many_arguments)]
pub fn invoke_pattern_associated_value_ordinary(
    semantic_world: &mut SemanticWorld,
    pattern: PatternValueId,
    operation_name: &str,
    receiver_value: SemanticValueId,
    mut explicit_arg_product: ArgProductShape,
    resolver_context: &ResolverContext,
    context: OrdinaryInvocationContext<'_>,
    provenance: Provenance,
) -> Result<InvocationOutcome, OrdinaryInvocationFailure> {
    let Some(receiver) = semantic_world.value(receiver_value) else {
        return Err(OrdinaryInvocationFailure::NoTargetValues {
            trace: OrdinaryPipelineTrace::default(),
        });
    };
    let receiver_mode = receiver.mode;
    let mut atoms = Vec::with_capacity(1 + explicit_arg_product.flattened.atoms.len());
    atoms.push(ProductAtom::SemanticValue {
        value: receiver_value,
        type_value: receiver.type_value,
        mode: receiver_mode,
        provenance: provenance.clone(),
    });
    atoms.append(&mut explicit_arg_product.flattened.atoms);
    let explicit_arg_product = ArgProductShape::from_flattened(FlattenedProductObject {
        atoms,
        provenance: provenance.clone(),
        invariant: FlattenedProductInvariant {
            no_direct_product_atom_remains: true,
        },
    });
    let target_members =
        semantic_world.associated_member_views_for_pattern(pattern, operation_name, context.phase);
    invoke_target_values(
        semantic_world,
        OrdinaryCandidateOrigin::PatternAssociatedValue(pattern),
        target_members,
        BTreeMap::new(),
        None,
        None,
        explicit_arg_product,
        resolver_context,
        context,
        provenance,
    )
}

/// Cc stage: filter sibling vals that are callable.
///
/// Callable(v) iff the immutable callspace of the exact complete Type captured
/// when `v` was formed contains `()`. Object.Val2 is not callable authority,
/// and the Type lookup key must not be refreshed to a later snapshot.

fn filter_callable(
    semantic_world: &SemanticWorld,
    values: &[SemanticValueId],
) -> Vec<CallableTarget> {
    values
        .iter()
        .filter_map(|value| {
            let entries = semantic_world.callable_entries_for_value(*value);
            let call_entries: Vec<SemanticValueId> = entries
                .into_iter()
                .filter(|entry| {
                    matches!(
                        semantic_world.value(*entry).map(|v| &v.payload),
                        Some(SemanticValuePayload::CallEntry(_))
                    )
                })
                .collect();
            if call_entries.is_empty() {
                return None;
            }
            Some(CallableTarget {
                sibling_value: *value,
                call_entries,
            })
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn invoke_target_values(
    semantic_world: &mut SemanticWorld,
    origin: OrdinaryCandidateOrigin,
    target_members: Vec<PolicyResultEntry<SemanticValueId, PatternValueId>>,
    target_places: BTreeMap<SemanticValueId, ObjectPlaceId>,
    associated_receiver: Option<SemanticValueId>,
    source_call_site: Option<&NormalizedCallSite>,
    mut arg_shape: ArgProductShape,
    resolver_context: &ResolverContext,
    context: OrdinaryInvocationContext<'_>,
    provenance: Provenance,
) -> Result<InvocationOutcome, OrdinaryInvocationFailure> {
    // C0: canonical member views of the resolved target.  Pure-P member
    // views (value = None) are legal cluster members but never invocation
    // candidates; only value-bearing views continue.  All subsequent
    // exposure decisions read the per-member view Policy — never a flat
    // Symbol/cluster aggregate.
    let value_views = target_members
        .iter()
        .filter(|view| view.value.is_some())
        .cloned()
        .collect::<Vec<_>>();
    let mut trace = OrdinaryPipelineTrace {
        c0_target_values: value_views.iter().filter_map(|view| view.value).collect(),
        ..OrdinaryPipelineTrace::default()
    };
    if target_members.is_empty() {
        return Err(OrdinaryInvocationFailure::NoTargetValues { trace });
    }

    let direct_pattern_entries = matches!(
        origin,
        OrdinaryCandidateOrigin::PatternAssociatedCallEntry(_)
    );

    // C1: member-level visibility exposure.  Internal callers see every
    // member view; external callers see the member views whose value is
    // publicly reachable.  This keeps/drops individual member views, never
    // the whole Symbol.
    let c1_views = value_views
        .into_iter()
        .filter(|view| {
            view.value.is_some_and(|id| {
                semantic_world.value(id).is_some_and(|value| {
                    context.visibility == VisibilityView::Internal
                        || value.namespace_visibility == Some(crate::NamespaceVisibility::Public)
                })
            })
        })
        .collect::<Vec<_>>();
    trace.c1_visible_values = c1_views.iter().filter_map(|view| view.value).collect();

    // C2: expose the member views whose own value Policy is visible at the
    // call phase; do not confuse exposure with ReadValue.  The projection
    // reads the member view's value_policy — not the value object's flat
    // PolicyPair and not any cluster-level union.
    let c2_views = c1_views
        .into_iter()
        .filter(|view| view.view.pair.value.stages.visible_at(context.phase))
        .collect::<Vec<_>>();
    let mut c2 = Vec::new();
    for view in &c2_views {
        if let Some(id) = view.value {
            if !c2.contains(&id) {
                c2.push(id);
            }
        }
    }
    trace.c2_phase_values = c2.clone();

    // Cc: filter sibling vals that are callable.  A value v is callable iff
    // (v |> type).Val2 contains an associated `()` call entry.
    let callable_targets = filter_callable(semantic_world, &c2);
    trace.callable_values = callable_targets
        .iter()
        .map(|target| target.sibling_value)
        .collect();

    // C3: for each callable sibling val, resolve its call entries.
    // A compiler-authorized Pattern entrance has already reached associated
    // Val2 and therefore feeds those call-entry values directly into C3.
    let mut c3 = Vec::new();
    let mut call_entry_receivers = BTreeMap::new();
    if direct_pattern_entries {
        let Some(receiver_value) = associated_receiver else {
            return Err(OrdinaryInvocationFailure::NoFullyAdmissibleCandidate {
                first_diagnostic: Some(Diagnostic::hard_error(
                    "Pattern-associated ordinary call entries require an explicit semantic receiver",
                    Some(provenance.clone()),
                )),
                trace,
            });
        };
        for entry in c2 {
            if matches!(
                semantic_world.value(entry).map(|value| &value.payload),
                Some(SemanticValuePayload::CallEntry(_))
            ) {
                c3.push(entry);
                call_entry_receivers.insert(entry, receiver_value);
            }
        }
    } else {
        for target in &callable_targets {
            for entry in &target.call_entries {
                c3.push(*entry);
                call_entry_receivers.insert(*entry, target.sibling_value);
            }
        }
    }
    c3.sort();
    c3.dedup();
    trace.c3_call_entries = c3.clone();

    classify_semantic_value_arguments(
        &mut arg_shape,
        semantic_world,
        resolver_context,
        context.phase,
    );
    let classified = classify_type_arguments_env_with_report(
        &arg_shape,
        &SemanticTypeEnv::new(semantic_world),
        resolver_context,
    );
    let args = overload_args_from_classified_shape(
        &classified.classified_shape,
        |_| None,
        |type_value| {
            semantic_world
                .type_value(type_value)
                .map(|value| value.pattern)
        },
    );

    // A: hard structural and phase/body-entry applicability.
    let mut prepared = Vec::new();
    let mut first_diagnostic = None;
    for call_entry_value in c3 {
        let Some(entry_value) = semantic_world.value(call_entry_value) else {
            continue;
        };
        let SemanticValuePayload::CallEntry(entry) = &entry_value.payload else {
            continue;
        };
        // Candidate preparation may intern Type Core observations during A;
        // own the call-entry facts so no immutable world borrow can become a
        // hidden constraint on that semantic observation.
        let entry = entry.clone();
        // C2 result-demand hard projection. Every pair/stage coordinate that
        // can affect producer selection is present before maxima; whole-slot
        // mode remains the independent Bp coordinate below.
        if !result_pair_demand_admits(
            &entry.complete_result_view.pair,
            &context.result_policy_demand.pair_query,
        ) {
            continue;
        }
        // The call entry carries its own declaration
        // environment (`declaration_name` / `declaration_namespace`); the
        // A-stage never looks the backing declaration up in the name index.
        // Body-entry admissibility is judged on the call entry's own
        // declaration-local P2; the declaration identity below is rebuilt
        // from the entry's declared facts for the shared candidate and
        // body-evaluator carriers.
        if !body_entry_allows_execution(&entry.body_entry_view.pair, context.execution_env) {
            continue;
        }
        let declaration_identity = SymbolObject::new(
            entry.backing_declaration,
            entry.declaration_name.clone(),
            SymbolKind::Object,
            SourceCategory::DeclaredSymbol,
            entry.declaration_namespace,
            entry.provenance.clone(),
        );

        let target_value = call_entry_receivers
            .get(&call_entry_value)
            .copied()
            .expect("every C3 entry retains its receiver");
        let target = semantic_world
            .value(target_value)
            .cloned()
            .expect("C1 retained existing target values");
        let target_place = target_places.get(&target_value).copied();
        if entry.receiver_type != target.type_value {
            continue;
        }
        let (
            source_shape,
            core_invocation,
            intrinsic_body,
            formal_policy_frame,
            self_policy,
            overload_strategy,
            frame_args,
        ) = if let Some(entry_closure) = &entry.closure {
            let declaration_pattern_context = ResolverContext {
                current_namespace: entry.declaration_namespace.unwrap_or_else(|| {
                    resolver_context
                        .explicit_mount_roots
                        .first()
                        .copied()
                        .unwrap_or(resolver_context.current_namespace)
                }),
                explicit_mount_roots: resolver_context.explicit_mount_roots.clone(),
                default_mounts: resolver_context.default_mounts.clone(),
            };
            if let Some(migration) = context.migration {
                // same-Type is hard migration applicability. A source
                // candidate whose declared result Type cannot be observed,
                // or whose Core differs from the source Core, never enters A
                // and therefore cannot win only to fail after execution.
                let declared_result_type = match declared_value_result_type(
                    entry_closure,
                    semantic_world,
                    &declaration_pattern_context,
                    provenance.clone(),
                ) {
                    Ok(Some(result_type)) => result_type,
                    Ok(None) => continue,
                    Err(diagnostic) => {
                        return Err(OrdinaryInvocationFailure::ApplicabilityUnsupported {
                            diagnostic,
                            trace,
                        });
                    }
                };
                match same_type_core(
                    semantic_world,
                    migration.request.source_type(),
                    declared_result_type,
                ) {
                    Ok(true) => {}
                    Ok(false) => continue,
                    Err(diagnostic) => {
                        return Err(OrdinaryInvocationFailure::ApplicabilityUnsupported {
                            diagnostic,
                            trace,
                        });
                    }
                }
            }
            let world_view: &SemanticWorld = semantic_world;
            let resolve_named_pattern = |name: &str| {
                SemanticTypeEnv::new(world_view)
                    .resolve_type_name(name, &declaration_pattern_context)
                    .and_then(|resolution| {
                        let pattern = world_view.type_value(resolution.represented_type)?.pattern;
                        let core = resolution.complete_type_observation.and_then(|whole| {
                            world_view
                                .complete_type_by_whole_observation(whole)
                                .map(|complete| complete.core())
                        });
                        Some(crate::NamedPatternObservation { pattern, core })
                    })
            };
            let mut source_shape = match applicable_candidate_from_closure(
                &declaration_identity,
                entry_closure,
                &entry.provenance,
                &args,
                context.execution_env,
                entry.callable_owner,
                Some(&resolve_named_pattern),
            ) {
                Ok(candidate) => candidate,
                Err(CandidateApplicabilityFailure::Inapplicable(diagnostic)) => {
                    first_diagnostic.get_or_insert(diagnostic);
                    continue;
                }
                Err(CandidateApplicabilityFailure::Unsupported(diagnostic)) => {
                    return Err(OrdinaryInvocationFailure::ApplicabilityUnsupported {
                        diagnostic,
                        trace,
                    });
                }
            };
            if let Err(failure) = apply_written_self_structure(
                &mut source_shape,
                &entry,
                &target,
                semantic_world,
                resolver_context,
                provenance.clone(),
            ) {
                match failure {
                    CandidateApplicabilityFailure::Inapplicable(diagnostic) => {
                        first_diagnostic.get_or_insert(diagnostic);
                        continue;
                    }
                    CandidateApplicabilityFailure::Unsupported(diagnostic) => {
                        return Err(OrdinaryInvocationFailure::ApplicabilityUnsupported {
                            diagnostic,
                            trace,
                        });
                    }
                }
            }
            if let Err(failure) = validate_explicit_value_type_annotations(
                &entry,
                &classified.classified_shape,
                semantic_world,
                resolver_context,
                provenance.clone(),
            ) {
                match failure {
                    CandidateApplicabilityFailure::Inapplicable(diagnostic) => {
                        first_diagnostic.get_or_insert(diagnostic);
                        continue;
                    }
                    CandidateApplicabilityFailure::Unsupported(diagnostic) => {
                        return Err(OrdinaryInvocationFailure::ApplicabilityUnsupported {
                            diagnostic,
                            trace,
                        });
                    }
                }
            }
            let formal_policy_frame = match formal_policy_frame(&entry, provenance.clone()) {
                Ok(frame) => frame,
                Err(CandidateApplicabilityFailure::Inapplicable(diagnostic)) => {
                    first_diagnostic.get_or_insert(diagnostic);
                    continue;
                }
                Err(CandidateApplicabilityFailure::Unsupported(diagnostic)) => {
                    return Err(OrdinaryInvocationFailure::ApplicabilityUnsupported {
                        diagnostic,
                        trace,
                    });
                }
            };
            // The canonical P1 was already normalized at the declaration
            // boundary by `canonical_function_object_p1`.  P1(function
            // object) = P1(slot0/self) = P1(let ()).  Do not re-derive from
            // the closure AST at invocation time.
            let self_policy = entry.callable_view.pair.clone();
            let strategy = source_shape.overload_strategy.clone();
            (
                Some(source_shape),
                None,
                None,
                formal_policy_frame,
                self_policy,
                strategy,
                classified.classified_shape.clone(),
            )
        } else if let Some(primitive) = entry.core_primitive {
            if context.migration.is_some() {
                // The connected slice has no declared ordinary result-Type
                // observation for core primitives. They cannot be admitted
                // as same-Type migration candidates by assumption.
                continue;
            }
            let Some(call_site) = source_call_site else {
                first_diagnostic.get_or_insert_with(|| {
                    Diagnostic::hard_error(
                        "core ordinary call entry requires its already-normalized source call site",
                        Some(provenance.clone()),
                    )
                });
                continue;
            };
            let core_invocation = match crate::meta::prepare_resolved_core_meta_call_with_primitive(
                &declaration_identity,
                primitive,
                call_site,
                &SemanticTypeEnv::new(&*semantic_world),
                resolver_context,
                context.policy_env,
                context.execution_env,
                provenance.clone(),
            ) {
                Ok(candidate) => candidate,
                Err(error) => {
                    if let Some(diagnostic) = error.diagnostics.into_iter().next() {
                        first_diagnostic.get_or_insert(diagnostic);
                    }
                    continue;
                }
            };
            let frame_args = core_invocation.candidate.arg_product_shape.clone();
            (
                None,
                Some(core_invocation),
                None,
                PolicyFormalFrame {
                    self_mode: PolicyMode::Plain,
                    explicit_parameter_modes: vec![PolicyMode::Plain; frame_args.arity],
                },
                // Core and source candidates use the same P1 coordinate. The
                // core candidate's function-object view is its declared
                // canonical P1 (`callable_value_policy`).
                entry.callable_view.pair.clone(),
                NormOverloadStrategy::Ordinary,
                frame_args,
            )
        } else if let Some(intrinsic) = &entry.intrinsic_body {
            if context.migration.is_some() {
                // Construction intrinsics are type-changing operations, not
                // same-Type Policy migration candidates.
                continue;
            }
            let Some(target_snapshot) = context.construction_target else {
                first_diagnostic.get_or_insert_with(|| {
                    Diagnostic::hard_error(
                        "ordinary construction intrinsic requires an exact complete target Type",
                        Some(provenance.clone()),
                    )
                });
                continue;
            };
            let SemanticValuePayload::CoreTypeProjection {
                represented_type: receiver_target,
                ..
            } = &target.payload
            else {
                continue;
            };
            let [ProductAtom::SemanticValue {
                value: source_value,
                ..
            }] = classified.classified_shape.flattened.atoms.as_slice()
            else {
                continue;
            };
            let Some(source) = semantic_world.value(*source_value) else {
                continue;
            };
            let SemanticValuePayload::AbstractLiteral { family, .. } = &source.payload else {
                continue;
            };
            let applicable = match intrinsic {
                crate::semantic_world::OrdinaryIntrinsicBody::AbstractLiteralConstruct(spec) => {
                    spec.source_family == *family
                        && spec.target_type == *receiver_target
                        && target_snapshot.lookup_key() == *receiver_target
                }
                crate::semantic_world::OrdinaryIntrinsicBody::Delete
                | crate::semantic_world::OrdinaryIntrinsicBody::FailSelected => {
                    target_snapshot.lookup_key() == *receiver_target
                }
            };
            if !applicable {
                continue;
            }
            (
                None,
                None,
                Some(intrinsic.clone()),
                PolicyFormalFrame {
                    self_mode: PolicyMode::Plain,
                    explicit_parameter_modes: vec![PolicyMode::Plain],
                },
                entry.callable_view.pair.clone(),
                NormOverloadStrategy::Ordinary,
                classified.classified_shape.clone(),
            )
        } else {
            continue;
        };
        // Compute stable migration endpoint coordinates once at
        // A-stage.  A only checks admissibility (is None?); Bp' later does
        // preference product comparison on the SAME coordinates.  No
        // re-interpretation of the candidate downstream.
        //
        // §4.1 input endpoint = Source formal = first explicit Product formal
        //     after slot0.
        // §4.2 output endpoint = canonical P1 / `callable_value_policy`.
        let (migration_input_endpoint, migration_output_endpoint) = match context.migration {
            Some(migration) => {
                let source_formal_p1 = entry
                    .closure
                    .as_ref()
                    .and_then(|c| c.head.as_ref())
                    .and_then(|head| head.formal_frame().explicit_parameters.first())
                    .and_then(|elem| match elem {
                        NormPatternElem::BindingSlot(slot) => slot.policy.clone(),
                        _ => None,
                    })
                    .map(|spec| {
                        elaborate_formal_policy_pattern(
                            Some(&spec),
                            &entry.body_entry_view,
                            provenance.clone(),
                        )
                        .ok()
                        .map(|elab| elab.effective_pair)
                    })
                    .flatten()
                    .unwrap_or_else(|| entry.body_entry_view.pair.clone());
                let input = project_migration_input_endpoint(
                    &source_formal_p1,
                    &migration.request.source_view().pair,
                );
                let output = project_migration_output_endpoint(
                    migration.request.target_pair(),
                    &entry.callable_view.pair,
                );
                if input.is_none() || output.is_none() {
                    continue;
                }
                (input, output)
            }
            None => (None, None),
        };

        let self_position = SelfPosition::from_semantic_associated_call_entry(
            target_value,
            target.type_value,
            provenance.clone(),
        );
        let frame = match InvocationFrame::new(
            InvocationCallableRef::SemanticValue(call_entry_value),
            self_position,
            frame_args,
            InvocationLookupEnv::new(context.policy_env),
            InvocationExecutionEnv::new(context.execution_env),
            provenance.clone(),
        ) {
            Ok(frame) => frame,
            Err(diagnostic) => {
                first_diagnostic.get_or_insert(diagnostic);
                continue;
            }
        };
        prepared.push(PreparedCallCandidate {
            origin: origin.clone(),
            target_value,
            target_place,
            call_entry_value,
            backing_declaration: entry.backing_declaration,
            frame,
            body_entry_view: entry.body_entry_view.clone(),
            complete_result_view: entry.complete_result_view.clone(),
            function_object_view: PolicyView {
                pair: self_policy,
                mode: entry.callable_view.mode,
            },
            capability_realization: entry.capability_realization.clone(),
            formal_policy_frame,
            candidate_role: entry.candidate_role,
            declared_result_class: entry.declared_result_class.clone(),
            overload_strategy,
            source_shape,
            core_invocation,
            intrinsic_body,
            migration_input_endpoint,
            migration_output_endpoint,
        });
    }
    trace.a_fully_admissible = prepared
        .iter()
        .map(|candidate| candidate.call_entry_value)
        .collect();
    if prepared.is_empty() {
        return Err(OrdinaryInvocationFailure::NoFullyAdmissibleCandidate {
            first_diagnostic,
            trace,
        });
    }

    // Af: source currently constructs only Ordinary candidates.  The role is
    // nevertheless carried by the real candidate so future toolchain/source
    // metadata cannot accidentally place suppression at B6.
    let has_non_fallback = prepared
        .iter()
        .any(|candidate| candidate.candidate_role == OrdinaryCandidateRole::Ordinary);
    let af = prepared
        .iter()
        .filter(|candidate| {
            !has_non_fallback || candidate.candidate_role == OrdinaryCandidateRole::Ordinary
        })
        .collect::<Vec<_>>();
    trace.af_after_fallback = af
        .iter()
        .map(|candidate| candidate.call_entry_value)
        .collect();

    let actual_frame = PolicyActualFrame {
        caller_value: context.caller_mode,
        explicit_arguments: classified
            .classified_shape
            .raw_args
            .iter()
            .enumerate()
            .map(|(index, argument)| {
                argument.known_value_mode.unwrap_or_else(|| {
                    context
                        .explicit_argument_modes
                        .get(index)
                        .copied()
                        .unwrap_or(PolicyMode::Plain)
                })
            })
            .collect(),
    };

    // Bp': ordinary coordinates and optional migration endpoint coordinates
    // are compared in one product.  No maxima pass runs in between them.
    let bp = maximal_candidates(&af, |better, worse| {
        bp_prime_dominates(
            better,
            worse,
            &actual_frame,
            context.phase,
            OutputModeDemand(context.result_policy_demand.mode),
            context.migration,
        )
    });
    trace.bp_prime = bp
        .iter()
        .map(|candidate| candidate.call_entry_value)
        .collect();

    // B1/B2 are identity in the current connected slice.

    // B3: Pattern extraction specificity.  This is deliberately later than
    // the complete Bp' product.
    let b3 = maximal_candidates(&bp, |better, worse| {
        better.specificity() > worse.specificity()
    });
    trace.b3_pattern_specific = b3
        .iter()
        .map(|candidate| candidate.call_entry_value)
        .collect();

    // B4/B5/B6 are identity for the currently implemented source candidate
    // metadata.  Named strategy strings are preserved, not interpreted.
    let selected_candidate = match b3.as_slice() {
        [candidate] => (*candidate).clone(),
        candidates => {
            return Err(OrdinaryInvocationFailure::Ambiguous {
                candidates: candidates
                    .iter()
                    .map(|candidate| candidate.call_entry_value)
                    .collect(),
                trace,
            });
        }
    };
    trace.selected = Some(selected_candidate.call_entry_value);
    let legality =
        validate_dynamic_legality(semantic_world, &selected_candidate, &context, &provenance)
            .map_err(|diagnostic| OrdinaryInvocationFailure::DynamicLegality {
                selected: selected_candidate.call_entry_value,
                diagnostic,
                trace: trace.clone(),
            })?;
    trace.dynamic_legality = Some(legality.clone());
    let selected = SealedSelectedInvocation {
        candidate: selected_candidate,
        legality,
    };

    let canonical_callable_identity = crate::MetaCallableIdentity {
        selected_function_value: selected.target_value,
        selected_call_entry: selected.call_entry_value,
    };
    let is_ambient_struct = matches!(
        selected
            .core_invocation
            .as_ref()
            .and_then(|core| core.candidate.callee_primitive),
        Some(crate::CoreMetaFunction::Struct)
    );
    let ambient_construction_owner = context
        .ambient_construction_owner
        .or_else(|| semantic_world.namespace_owner(resolver_context.current_namespace));

    // Ordinary meta invocation identity is computed once for the selected
    // candidate and shared by every ordinary meta construction path.  The
    // privileged `struct` builtin is intentionally excluded: its canonical
    // owner rule establishes no MetaInstance root, so forcing its private AST
    // carrier through an ordinary meta material key would invent semantic
    // identity that the language does not have.
    let mut canonical_instance_key = if selected.declared_result_class
        != DeclaredResultClass::ClusterSymbol
        || is_ambient_struct
    {
        None
    } else {
        Some(canonical_meta_instance_key_for_selected(
            semantic_world,
            &classified.classified_shape,
            canonical_callable_identity,
            &provenance,
            &trace,
        )?)
    };

    // A declared `Unit` result is validated at the declaration boundary but
    // has no executable producer yet: report the execution gap explicitly
    // instead of silently misrouting the result into a single-member or
    // cluster carrier.
    if selected.declared_result_class == DeclaredResultClass::Unit {
        return Err(OrdinaryInvocationFailure::SelectedCoreBody {
            diagnostic: Diagnostic::hard_error(
                "invocation of a Unit result is future work: \
                 the declaration is validated, but no executable producer exists yet",
                Some(provenance.clone()),
            ),
            trace,
        });
    }

    let meta_construction_result = if selected.declared_result_class
        == DeclaredResultClass::ClusterSymbol
    {
        // The meta instance root binds the selected function object VALUE
        // identity (never the carrier Symbol hosting the overload cluster);
        // owner-forest placement comes from the selected call entry's
        // declaration environment — a SourceSymbol
        // origin is NOT required to return a meta construction cluster.
        let Some(placement_parent) =
            semantic_world.callable_declaration_environment(selected.call_entry_value)
        else {
            return Err(OrdinaryInvocationFailure::NoFullyAdmissibleCandidate {
                first_diagnostic: Some(Diagnostic::hard_error(
                    "meta construction requires a resolved call entry with a \
                     declaration-environment owner",
                    Some(provenance.clone()),
                )),
                trace,
            });
        };
        let meta_root = crate::MetaInstanceRoot {
            meta_callable: canonical_callable_identity,
            placement_parent,
        };
        // Begin an open cluster construction for this meta invocation.
        // The owner strategy is a fact of the selected callable and the
        // call context, never of the return category: a source meta
        // function roots its contributions at `MetaInstance(meta callable,
        // normalized arguments)`, while the builtin privileged `struct`
        // called directly attaches its complete type result to the ambient
        // declaration environment and never creates a
        // `MetaInstance(struct, arguments)` scope of its own.
        let owner_strategy = if is_ambient_struct {
            crate::OwnerStrategy::AmbientStructScope
        } else {
            crate::OwnerStrategy::OrdinaryMetaInstanceScope
        };
        // B8: the ambient construction owner is a fact of the declaration
        // environment supplied by the caller.  A declaration inside a
        // callable body supplies its innermost anonymous function object's
        // Self scope owner, so two ordinary functions in one namespace never
        // share an ambient struct root; the resolver's namespace node is
        // only the top-level declaration case.
        let ambient_owner = ambient_construction_owner;
        let (authority, owner) = match owner_strategy {
            crate::OwnerStrategy::AmbientStructScope => {
                let Some(ambient_owner) = ambient_owner else {
                    return Err(OrdinaryInvocationFailure::SelectedCoreBody {
                        diagnostic: Diagnostic::hard_error(
                            "ambient struct construction requires a declaration environment with a semantic owner",
                            Some(provenance.clone()),
                        ),
                        trace,
                    });
                };
                (
                    crate::ConstructionAuthority::AmbientScope {
                        owner: ambient_owner,
                    },
                    ambient_owner,
                )
            }
            _ => {
                let instance_key = canonical_instance_key
                    .as_ref()
                    .expect("ordinary meta construction has a canonical instance key")
                    .clone();
                (
                    crate::ConstructionAuthority::MetaInvocation {
                        meta_callable: meta_root.meta_callable,
                        canonical_key: instance_key,
                    },
                    meta_root.placement_parent,
                )
            }
        };
        let cid = semantic_world.begin_cluster_construction(authority, owner, provenance.clone());

        // Struct construction materials harvested for the binding side's
        // namespace projection expansion (field layer, ref/share views).
        let mut struct_materials: Vec<crate::StructConstructionMaterial> = Vec::new();

        // Each member contribution carries the member's own value Policy and
        // Pattern Policy together with its Pattern identity.
        let pure_p_member_view = |pattern| PolicyResultEntry {
            value: None,
            pattern,
            view: selected.complete_result_view.clone(),
        };

        if let Some(core) = &selected.core_invocation {
            // Core primitive bodies produce exactly one local result
            // carrier; the carrier drives one member contribution.
            let mut core_input = core.clone();
            attach_candidate_type_observations(semantic_world, &mut core_input, &trace)?;
            let value = match crate::meta_invocation::invoke_meta_callable(core_input) {
                crate::MetaPrimitiveExecution::Material(value) => value,
                crate::MetaPrimitiveExecution::Diagnostic(diagnostic) => {
                    return Err(OrdinaryInvocationFailure::SelectedCoreBody { diagnostic, trace });
                }
            };
            match &value {
                MetaExecutionMaterial::IdentityType(value) => {
                    // Core identity-forwarding primitives (builtin
                    // privileged contract, e.g. `IdentityType`): the
                    // cluster's unique type member is still navigated as
                    // the meta function plus its input arguments.  The
                    // forwarded type's own PatternValue keeps its
                    // original owner and is never rerooted.
                    let Some(created) = semantic_world.install_meta_instance_type_value(
                        &meta_root,
                        canonical_instance_key
                            .as_ref()
                            .expect("ordinary forwarded meta result has an instance key")
                            .clone(),
                        value.provenance.clone(),
                    ) else {
                        return Err(OrdinaryInvocationFailure::NoFullyAdmissibleCandidate {
                            first_diagnostic: Some(Diagnostic::hard_error(
                                "meta instance type member installation failed",
                                Some(provenance.clone()),
                            )),
                            trace: trace.clone(),
                        });
                    };
                    semantic_world
                        .contribute_cluster_member_view(cid, pure_p_member_view(created.1));
                }
                MetaExecutionMaterial::StructConstructionMaterial(value) => {
                    let installed = match owner_strategy {
                        crate::OwnerStrategy::AmbientStructScope => {
                            let ambient_owner = ambient_owner
                                .expect("AmbientStructScope construction carries an ambient owner");
                            if let Some((_existing, binder)) = semantic_world
                                .ambient_struct_collision(ambient_owner, value.material_id)
                            {
                                return Err(OrdinaryInvocationFailure::SelectedCoreBody {
                                    diagnostic: Diagnostic::hard_error(
                                        ambient_struct_collision_message(binder),
                                        Some(provenance.clone()),
                                    ),
                                    trace,
                                });
                            }
                            semantic_world.install_ambient_struct_type_value(
                                ambient_owner,
                                value.material_id,
                                value.canonical_pattern_value(),
                                selected.complete_result_view.pair.clone(),
                                value.provenance.clone(),
                            )
                        }
                        _ => match semantic_world.install_meta_struct_complete_type(
                            &meta_root,
                            canonical_instance_key
                                .as_ref()
                                .expect("ordinary meta struct result has an instance key")
                                .clone(),
                            value.material_id,
                            value.canonical_pattern_value(),
                            selected.complete_result_view.pair.clone(),
                            value.provenance.clone(),
                        ) {
                            Ok(installed) => installed,
                            Err(diagnostic) => {
                                return Err(OrdinaryInvocationFailure::SelectedCoreBody {
                                    diagnostic,
                                    trace,
                                });
                            }
                        },
                    };
                    if let Some((_value_id, pattern, complete_type)) = installed {
                        semantic_world
                            .contribute_cluster_member_view(cid, pure_p_member_view(pattern));
                        let mut value = value.clone();
                        value.canonical_type = Some(complete_type.lookup_key());
                        struct_materials.push(value);
                    }
                }
            }
        } else if selected.source_shape.is_some() {
            return Err(OrdinaryInvocationFailure::SelectedCoreBody {
                diagnostic: Diagnostic::hard_error(
                    "source meta construction is not connected to the canonical member-creation operations",
                    Some(provenance.clone()),
                )
                .with_code(ResolverCode::UnsupportedSelectedSourceBody),
                trace,
            });
        } else {
            unreachable!("a prepared meta candidate has exactly one implementation body")
        }

        let construction = semantic_world.finalize_type_cluster(cid).ok_or_else(|| {
            OrdinaryInvocationFailure::NoFullyAdmissibleCandidate {
                first_diagnostic: Some(Diagnostic::hard_error(
                    "meta cluster construction finalization failed",
                    Some(provenance.clone()),
                )),
                trace: trace.clone(),
            }
        })?;

        Some(ClusterSymbolResult {
            construction,
            struct_materials,
            result_p2: selected.body_entry_view.pair.clone(),
            trace: trace.clone(),
        })
    } else {
        None
    };

    if let Some(meta_result) = meta_construction_result {
        return Ok(semantic_invocation_outcome(
            selected.declared_result_class.clone(),
            ProjectedInvocationOutcome::ClusterSymbol(meta_result),
        ));
    }

    let returned = if let Some(source_shape) = &selected.source_shape {
        // This arm constructs the selected source body's execution carrier.
        let selected_body_input = SelectedSourceBody {
            symbol: source_shape.symbol.clone(),
            source_callable: source_shape.source_callable.clone(),
            bindings: source_shape.bindings.clone(),
            pack_bindings: source_shape.pack_bindings.clone(),
        };
        if !selected.is_delete() {
            if let Some(value) = forwarded_semantic_body_value(&selected) {
                SelectedBodyOutput::OrdinaryValue(value)
            } else {
                match evaluate_selected_source_body(
                    &SemanticTypeEnv::new(&*semantic_world),
                    resolver_context,
                    &selected_body_input,
                ) {
                    Ok(value) => SelectedBodyOutput::Material(value),
                    Err(failure) => {
                        return Err(OrdinaryInvocationFailure::SelectedBody { failure, trace });
                    }
                }
            }
        } else {
            match evaluate_selected_source_body(
                &SemanticTypeEnv::new(&*semantic_world),
                resolver_context,
                &selected_body_input,
            ) {
                Ok(value) => SelectedBodyOutput::Material(value),
                Err(failure) => {
                    return Err(OrdinaryInvocationFailure::SelectedDelete {
                        selected: selected.call_entry_value,
                        diagnostic: failure.diagnostic,
                        trace,
                    });
                }
            }
        }
    } else if let Some(core) = &selected.core_invocation {
        let mut core_input = core.clone();
        attach_candidate_type_observations(semantic_world, &mut core_input, &trace)?;
        match crate::meta_invocation::invoke_meta_callable(core_input) {
            crate::MetaPrimitiveExecution::Material(value) => SelectedBodyOutput::Material(value),
            crate::MetaPrimitiveExecution::Diagnostic(diagnostic) => {
                return Err(OrdinaryInvocationFailure::SelectedCoreBody { diagnostic, trace });
            }
        }
    } else if let Some(intrinsic) = &selected.intrinsic_body {
        match intrinsic {
            crate::semantic_world::OrdinaryIntrinsicBody::AbstractLiteralConstruct(_) => {
                let Some(target) = context.construction_target else {
                    return Err(OrdinaryInvocationFailure::SelectedCoreBody {
                        diagnostic: Diagnostic::hard_error(
                            "selected literal constructor lost its exact complete target Type",
                            Some(provenance.clone()),
                        ),
                        trace,
                    });
                };
                let [ProductAtom::SemanticValue {
                    value: source_value,
                    ..
                }] = selected
                    .frame
                    .explicit_arg_product
                    .flattened
                    .atoms
                    .as_slice()
                else {
                    return Err(OrdinaryInvocationFailure::SelectedCoreBody {
                        diagnostic: Diagnostic::hard_error(
                            "selected literal constructor requires exactly one abstract source value",
                            Some(provenance.clone()),
                        ),
                        trace,
                    });
                };
                let Some(constructed) = semantic_world.construct_abstract_literal_value(
                    *source_value,
                    target,
                    selected.complete_result_view.clone(),
                    provenance.clone(),
                ) else {
                    return Err(OrdinaryInvocationFailure::SelectedCoreBody {
                        diagnostic: Diagnostic::hard_error(
                            "selected literal constructor failed to realize its result",
                            Some(provenance.clone()),
                        ),
                        trace,
                    });
                };
                SelectedBodyOutput::OrdinaryValue(constructed)
            }
            crate::semantic_world::OrdinaryIntrinsicBody::Delete => {
                return Err(OrdinaryInvocationFailure::SelectedDelete {
                    selected: selected.call_entry_value,
                    diagnostic: Diagnostic::hard_error(
                        "selected literal construction candidate is deleted",
                        Some(provenance.clone()),
                    ),
                    trace,
                });
            }
            crate::semantic_world::OrdinaryIntrinsicBody::FailSelected => {
                return Err(OrdinaryInvocationFailure::SelectedCoreBody {
                    diagnostic: Diagnostic::hard_error(
                        "selected literal construction candidate failed to realize its result",
                        Some(provenance.clone()),
                    ),
                    trace,
                });
            }
        }
    } else {
        unreachable!("a prepared candidate has exactly one ordinary implementation body")
    };

    if canonical_instance_key.is_none()
        && !is_ambient_struct
        && matches!(
            returned,
            SelectedBodyOutput::Material(MetaExecutionMaterial::StructConstructionMaterial(_))
        )
    {
        canonical_instance_key = Some(canonical_meta_instance_key_for_selected(
            semantic_world,
            &classified.classified_shape,
            canonical_callable_identity,
            &provenance,
            &trace,
        )?);
    }
    let identity = ordinary_result_identity(
        semantic_world,
        &selected,
        canonical_instance_key.as_ref(),
        is_ambient_struct
            .then_some(ambient_construction_owner)
            .flatten(),
        returned,
    )
    .map_err(|diagnostic| OrdinaryInvocationFailure::SelectedCoreBody {
        diagnostic,
        trace: trace.clone(),
    })?;
    let Some((result_type, pattern, returned_value, returned)) = identity else {
        return Err(OrdinaryInvocationFailure::SelectedCoreBody {
            diagnostic: Diagnostic::hard_error(
                "selected callable did not form a canonical semantic result",
                Some(provenance),
            ),
            trace,
        });
    };
    if let Some(migration) = context.migration {
        let source = migration.request.source_type();
        let same_type =
            same_type_core(semantic_world, source, result_type).map_err(|diagnostic| {
                OrdinaryInvocationFailure::SelectedCoreBody {
                    diagnostic,
                    trace: trace.clone(),
                }
            })?;
        if !same_type {
            return Err(OrdinaryInvocationFailure::MigrationResultTypeChanged {
                source,
                result: result_type,
                trace,
            });
        }
        // The candidate stores its migration output endpoint after projecting
        // the canonical P1 (`callable_value_policy`) at A-stage. Execution
        // consumes that stored coordinate unchanged.
        if selected.migration_output_endpoint.is_none() {
            return Err(OrdinaryInvocationFailure::MigrationOutputProjectionFailed { trace });
        }
    }

    // CompleteResultView — these entries carry the result P2 type/Pattern
    // observations only. The outward
    // visibility of the invocation result is NOT this P2: it is the
    // canonical P1 layer, derived on demand by
    // `SingleMemberResult::exposed()`.
    let semantic_complete_type = match &returned {
        ReturnedSemanticEntity::CompleteType(value) => Some(value.complete_type.clone()),
        _ => None,
    };
    if (selected.declared_result_class == DeclaredResultClass::CompleteType)
        != semantic_complete_type.is_some()
    {
        return Err(OrdinaryInvocationFailure::SelectedCoreBody {
            diagnostic: Diagnostic::hard_error(
                "declared CompleteType result did not materialize an exact complete tau",
                Some(provenance.clone()),
            ),
            trace,
        });
    }

    let complete_result = vec![PolicyResultEntry {
        value: returned_value.map(|id| SemanticValueRef {
            id,
            type_value: result_type,
        }),
        pattern,
        view: selected.complete_result_view.clone(),
    }];

    let declared_result_class = selected.declared_result_class.clone();
    Ok(semantic_invocation_outcome(
        declared_result_class,
        ProjectedInvocationOutcome::SingleMember(SingleMemberResult {
            selected,
            returned,
            complete_type: semantic_complete_type,
            complete_result,
            trace,
        }),
    ))
}

fn forwarded_semantic_body_value(selected: &PreparedCallCandidate) -> Option<SemanticValueId> {
    let closure = &selected.source_shape.as_ref()?.source_callable.closure;
    let Some(head) = &closure.head else {
        return None;
    };
    let frame = head.formal_frame();
    let tail_name = match &closure.body {
        NormClosureBody::Block(program) | NormClosureBody::NamedBlock { body: program, .. } => {
            match program.forms.as_slice() {
                [NormForm::TailValue(lang_syntax::NormExpr::Name { text, .. })] => text,
                _ => return None,
            }
        }
        NormClosureBody::Defaulted { .. } | NormClosureBody::Delete(_) => return None,
    };

    if let Some(written_self) = frame.written_self {
        let self_name = match written_self {
            NormPatternElem::BindingSlot(slot) => match &slot.value_pattern {
                NormPattern::Binder { name, .. } => Some(name.clone()),
                _ => None,
            },
            NormPatternElem::Pattern(NormPattern::Binder { name, .. }) => Some(name.clone()),
            _ => None,
        };
        if self_name.as_ref().is_some_and(|name| name == tail_name) {
            return Some(selected.target_value);
        }
    }

    frame
        .explicit_parameters
        .iter()
        .zip(&selected.frame.explicit_arg_product.raw_args)
        .find_map(|(formal, actual)| {
            let NormPatternElem::BindingSlot(slot) = formal else {
                return None;
            };
            match &slot.value_pattern {
                NormPattern::Binder { name, .. } if name == tail_name => {
                    actual.known_semantic_value
                }
                _ => None,
            }
        })
}

fn classify_semantic_value_arguments(
    shape: &mut ArgProductShape,
    semantic_world: &SemanticWorld,
    resolver_context: &ResolverContext,
    phase: Phase,
) {
    for raw_arg in &mut shape.raw_args {
        if !matches!(raw_arg.value_class, RawArgValueClass::UnknownExpression) {
            continue;
        }
        let Some(ProductAtom::Expression {
            expr: lang_syntax::NormExpr::Name { text, .. },
            ..
        }) = shape.flattened.atoms.get(raw_arg.index)
        else {
            continue;
        };

        let mut namespaces = Vec::with_capacity(1 + resolver_context.default_mounts.len());
        namespaces.push(resolver_context.current_namespace);
        namespaces.extend(resolver_context.default_mounts.iter().copied());
        let Some(symbol) = namespaces
            .into_iter()
            .find_map(|namespace| semantic_world.symbol_in_namespace(namespace, text))
        else {
            continue;
        };

        let mut readable = symbol
            .member_views
            .iter()
            .filter_map(|view| {
                let value = view.value?;
                let object = semantic_world.value(value)?;
                // CoreTypeProjection is projection material, not an ordinary
                // value argument. Pure-P/type views carry `value=None`; this
                // check also excludes projection material stored in a
                // value-bearing transport view.
                if matches!(
                    object.payload,
                    SemanticValuePayload::CoreTypeProjection { .. }
                ) || !view
                    .view
                    .pair
                    .value
                    .stages
                    .iter()
                    .any(|stage| stage.visible_at(phase))
                {
                    return None;
                }
                Some((value, object.type_value, view.view.mode))
            })
            .collect::<Vec<_>>();
        readable.sort_by_key(|(value, _, _)| *value);
        readable.dedup_by_key(|(value, _, _)| *value);
        let [(value, type_value, mode)] = readable.as_slice() else {
            continue;
        };
        *raw_arg = raw_arg
            .clone()
            .as_resolved_semantic_value(*value, *type_value, *mode);
    }
}

fn formal_policy_frame(
    entry: &OrdinaryCallEntry,
    provenance: Provenance,
) -> Result<PolicyFormalFrame, CandidateApplicabilityFailure> {
    let head = entry
        .closure
        .as_ref()
        .and_then(|closure| closure.head.as_ref())
        .ok_or_else(|| {
            CandidateApplicabilityFailure::Unsupported(Diagnostic::hard_error(
                "ordinary call entry has no explicit closure head",
                Some(provenance.clone()),
            ))
        })?;
    let frame = head.formal_frame();
    let self_mode = match frame.written_self {
        // The written-self slot policy is explicit P1 material: stage /
        // presence / Pattern atoms are legal there and are
        // reconciled by `canonical_function_object_p1` at registration.
        // The Bₚ' Policy-mode frame only consumes the PolicyMode coordinate.
        Some(element) => match elaborate_explicit_p1(
            element_policy(element),
            &entry.callable_view.pair,
            ExplicitP1Position::WrittenSelf,
            provenance.clone(),
        )
        .map_err(CandidateApplicabilityFailure::Unsupported)?
        .and_then(|selection| selection.mode)
        {
            Some(PolicyMode::Const) => PolicyMode::Const,
            Some(PolicyMode::Mut) => PolicyMode::Mut,
            _ => PolicyMode::Plain,
        },
        None => PolicyMode::Plain,
    };
    let explicit_parameter_modes = frame
        .explicit_parameters
        .iter()
        .map(|element| {
            formal_policy_mode(
                element_policy(element),
                &entry.body_entry_view,
                provenance.clone(),
            )
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(CandidateApplicabilityFailure::Unsupported)?;
    Ok(PolicyFormalFrame {
        self_mode,
        explicit_parameter_modes,
    })
}

fn apply_written_self_structure(
    candidate: &mut ApplicableCandidate,
    entry: &OrdinaryCallEntry,
    actual: &crate::semantic_world::SemanticValueObject,
    semantic_world: &SemanticWorld,
    resolver_context: &ResolverContext,
    provenance: Provenance,
) -> Result<(), CandidateApplicabilityFailure> {
    let Some(head) = entry
        .closure
        .as_ref()
        .and_then(|closure| closure.head.as_ref())
    else {
        return Err(CandidateApplicabilityFailure::Unsupported(
            Diagnostic::hard_error(
                "ordinary call entry has no explicit closure head",
                Some(provenance),
            ),
        ));
    };
    let Some(written_self) = head.formal_frame().written_self else {
        return Ok(());
    };
    let NormPatternElem::BindingSlot(slot) = written_self else {
        return Err(CandidateApplicabilityFailure::Unsupported(
            Diagnostic::hard_error(
                "ordinary written self Pattern is not a binding slot",
                Some(provenance),
            ),
        ));
    };

    let mut self_specificity = match &slot.value_pattern {
        NormPattern::Binder { .. } => SpecificityTuple {
            max_depth: 1,
            sum_depth: 1,
            non_discard_explicit_node_count: 1,
            ..SpecificityTuple::default()
        },
        NormPattern::Skeleton {
            skeleton: lang_syntax::NormSkeleton::Wildcard { .. },
            ..
        } => SpecificityTuple {
            max_depth: 1,
            sum_depth: 1,
            explicit_discard_count: 1,
            ..SpecificityTuple::default()
        },
        _ => {
            return Err(CandidateApplicabilityFailure::Unsupported(
                Diagnostic::hard_error(
                    "ordinary written self structural Pattern is not yet supported by the Pattern relation consumer",
                    Some(Provenance::from_norm_origin(
                        "ordinary written self Pattern",
                        &slot.origin,
                    )),
                ),
            ));
        }
    };

    if let Some(annotation) = &slot.annotation {
        let expected = resolve_type_annotation_value(
            &annotation.pattern,
            semantic_world,
            resolver_context,
            provenance.clone(),
        )
        .map_err(CandidateApplicabilityFailure::Unsupported)?;
        if expected != actual.type_value {
            return Err(CandidateApplicabilityFailure::Inapplicable(
                Diagnostic::hard_error(
                    format!(
                        "ordinary written self type applicability failed: expected {:?}, got {:?}",
                        expected, actual.type_value
                    ),
                    Some(provenance),
                ),
            ));
        }
        self_specificity = self_specificity.add(SpecificityTuple {
            max_depth: 1,
            sum_depth: 1,
            non_discard_explicit_node_count: 1,
            ..SpecificityTuple::default()
        });
    }

    candidate.specificity = candidate.specificity.add(self_specificity);
    Ok(())
}

fn resolve_type_annotation_value(
    pattern: &NormPattern,
    semantic_world: &SemanticWorld,
    resolver_context: &ResolverContext,
    provenance: Provenance,
) -> Result<TypeValueId, Diagnostic> {
    let name = match pattern {
        NormPattern::Name { name, .. } => name,
        _ => {
            return Err(Diagnostic::hard_error(
                "ordinary written self type annotation requires a resolved type-name Pattern",
                Some(provenance),
            ));
        }
    };
    SemanticTypeEnv::new(semantic_world)
        .resolve_type_name(name, resolver_context)
        .map(|resolution| resolution.represented_type)
        .ok_or_else(|| {
            Diagnostic::hard_error(
                format!("ordinary written self annotation `{name}` is not a resolved type value"),
                Some(provenance.clone()),
            )
        })
}

/// Declared ordinary value result Type used by same-Type migration A.
///
/// An absent annotation, or a non-value result class (`type`, `symbol`,
/// `unit`), supplies no same-Type proof and is therefore not admissible as a
/// migration producer in the connected slice.
fn declared_value_result_type(
    closure: &lang_syntax::NormClosure,
    semantic_world: &SemanticWorld,
    resolver_context: &ResolverContext,
    provenance: Provenance,
) -> Result<Option<TypeValueId>, Diagnostic> {
    let Some(annotation) = closure
        .head
        .as_ref()
        .and_then(|head| head.returns.as_ref())
        .and_then(|returns| returns.annotation.as_ref())
    else {
        return Ok(None);
    };
    if matches!(
        &annotation.pattern,
        NormPattern::Name { name, .. } if matches!(name.as_str(), "type" | "symbol" | "unit")
    ) {
        return Ok(None);
    }
    resolve_type_annotation_value(
        &annotation.pattern,
        semantic_world,
        resolver_context,
        provenance,
    )
    .map(Some)
}

fn same_type_core(
    semantic_world: &mut SemanticWorld,
    left: TypeValueId,
    right: TypeValueId,
) -> Result<bool, Diagnostic> {
    let left = semantic_world.canonical_registered_type_core_observation_address(left)?;
    let right = semantic_world.canonical_registered_type_core_observation_address(right)?;
    Ok(left == right)
}

fn validate_explicit_value_type_annotations(
    entry: &OrdinaryCallEntry,
    actuals: &ArgProductShape,
    semantic_world: &SemanticWorld,
    resolver_context: &ResolverContext,
    provenance: Provenance,
) -> Result<(), CandidateApplicabilityFailure> {
    let Some(head) = entry
        .closure
        .as_ref()
        .and_then(|closure| closure.head.as_ref())
    else {
        return Ok(());
    };
    for (formal, actual) in head
        .formal_frame()
        .explicit_parameters
        .iter()
        .zip(&actuals.raw_args)
    {
        let NormPatternElem::BindingSlot(slot) = formal else {
            continue;
        };
        let Some(annotation) = &slot.annotation else {
            continue;
        };
        let is_type_rank_annotation = matches!(
            &annotation.pattern,
            NormPattern::Name { name, .. } if name == "type"
        );
        if is_type_rank_annotation
            && matches!(
                actual.value_class,
                crate::RawArgValueClass::NonValue(crate::NonValueArgKind::CoreTypeProjection)
            )
        {
            continue;
        }
        let expected = resolve_type_annotation_value(
            &annotation.pattern,
            semantic_world,
            resolver_context,
            provenance.clone(),
        )
        .map_err(CandidateApplicabilityFailure::Unsupported)?;
        let Some(actual) = actual.known_first_order_type_value else {
            return Err(CandidateApplicabilityFailure::Unsupported(
                Diagnostic::hard_error(
                    "ordinary value parameter requires an evaluated argument TypeValue",
                    Some(provenance),
                ),
            ));
        };
        if expected != actual {
            return Err(CandidateApplicabilityFailure::Inapplicable(
                Diagnostic::hard_error(
                    format!(
                    "ordinary value parameter type applicability failed: expected {:?}, got {:?}",
                    expected, actual
                ),
                    Some(provenance),
                ),
            ));
        }
    }
    Ok(())
}

fn element_policy(element: &NormPatternElem) -> Option<&NormPolicySpec> {
    match element {
        NormPatternElem::BindingSlot(slot) => slot.policy.as_ref(),
        NormPatternElem::Pattern(_) | NormPatternElem::Unit { .. } => None,
    }
}

fn formal_policy_mode(
    policy: Option<&NormPolicySpec>,
    inherited: &PolicyView,
    provenance: Provenance,
) -> Result<PolicyMode, Diagnostic> {
    Ok(elaborate_formal_policy_pattern(policy, inherited, provenance)?.mode)
}

fn bp_prime_dominates(
    better: &PreparedCallCandidate,
    worse: &PreparedCallCandidate,
    actual: &PolicyActualFrame,
    phase: Phase,
    output_demand: OutputModeDemand,
    migration: Option<MigrationInvocationContext<'_>>,
) -> bool {
    let mut strictly_better = false;
    match compare_policy_frames(
        &better.formal_policy_frame,
        &worse.formal_policy_frame,
        actual,
    ) {
        PolicyPartialOrdering::Less | PolicyPartialOrdering::Incomparable => return false,
        PolicyPartialOrdering::Greater => strictly_better = true,
        PolicyPartialOrdering::Equal => {}
    }

    match compare_phase_view(
        &better.function_object_view.pair,
        &worse.function_object_view.pair,
        phase,
    ) {
        PolicyPartialOrdering::Less | PolicyPartialOrdering::Incomparable => return false,
        PolicyPartialOrdering::Greater => strictly_better = true,
        PolicyPartialOrdering::Equal => {}
    }

    match policy_mode_preference_rank(better.complete_result_view.mode, output_demand.mode()).cmp(
        &policy_mode_preference_rank(worse.complete_result_view.mode, output_demand.mode()),
    ) {
        std::cmp::Ordering::Less => return false,
        std::cmp::Ordering::Greater => strictly_better = true,
        std::cmp::Ordering::Equal => {}
    }

    if let Some(migration) = migration {
        // A and Bp' use the SAME endpoint facts stored on the
        // PreparedCandidate.  Input = Source formal (first explicit Product
        // formal after slot0); output = canonical P1 / callable_value_policy.
        // Do not re-derive from `function_object_p1` here.
        let better_input = better
            .migration_input_endpoint
            .as_ref()
            .expect("migration candidates store input endpoint at A-stage");
        let better_output = better
            .migration_output_endpoint
            .as_ref()
            .expect("migration candidates store output endpoint at A-stage");
        let worse_input = worse
            .migration_input_endpoint
            .as_ref()
            .expect("migration candidates store input endpoint at A-stage");
        let worse_output = worse
            .migration_output_endpoint
            .as_ref()
            .expect("migration candidates store output endpoint at A-stage");
        match compare_migration_endpoint_coordinates(
            &migration.request.source_view().pair,
            migration.request.target_pair(),
            better_input,
            better_output,
            worse_input,
            worse_output,
        ) {
            PolicyPartialOrdering::Less | PolicyPartialOrdering::Incomparable => return false,
            PolicyPartialOrdering::Greater => strictly_better = true,
            PolicyPartialOrdering::Equal => {}
        }
    }
    strictly_better
}

fn compare_policy_frames(
    left: &PolicyFormalFrame,
    right: &PolicyFormalFrame,
    actual: &PolicyActualFrame,
) -> PolicyPartialOrdering {
    if left.explicit_parameter_modes.len() != actual.explicit_arguments.len()
        || right.explicit_parameter_modes.len() != actual.explicit_arguments.len()
    {
        return PolicyPartialOrdering::Incomparable;
    }
    let mut left_better = false;
    let mut right_better = false;
    compare_policy_mode_position(
        left.self_mode,
        right.self_mode,
        actual.caller_value,
        &mut left_better,
        &mut right_better,
    );
    for ((left, right), actual) in left
        .explicit_parameter_modes
        .iter()
        .zip(&right.explicit_parameter_modes)
        .zip(&actual.explicit_arguments)
    {
        compare_policy_mode_position(*left, *right, *actual, &mut left_better, &mut right_better);
    }
    ordering_from_advantages(left_better, right_better)
}

fn compare_policy_mode_position(
    left: PolicyMode,
    right: PolicyMode,
    actual: PolicyMode,
    left_better: &mut bool,
    right_better: &mut bool,
) {
    match policy_mode_preference_rank(left, actual).cmp(&policy_mode_preference_rank(right, actual))
    {
        std::cmp::Ordering::Greater => *left_better = true,
        std::cmp::Ordering::Less => *right_better = true,
        std::cmp::Ordering::Equal => {}
    }
}

fn compare_phase_view(
    left: &PolicyPair,
    right: &PolicyPair,
    phase: Phase,
) -> PolicyPartialOrdering {
    match best_stage_rank(left, phase).cmp(&best_stage_rank(right, phase)) {
        std::cmp::Ordering::Greater => PolicyPartialOrdering::Greater,
        std::cmp::Ordering::Equal => PolicyPartialOrdering::Equal,
        std::cmp::Ordering::Less => PolicyPartialOrdering::Less,
    }
}

fn result_pair_demand_admits(candidate: &PolicyPair, demand: &P1Projection) -> bool {
    let value_admits = |required: &ValueComponentPolicy| {
        let presence = matches!(required.presence, crate::ValuePresence::Optional)
            || matches!(candidate.value.presence, crate::ValuePresence::Optional)
            || required.presence == candidate.value.presence;
        let stages = required.stages.is_empty()
            || (required.presence == crate::ValuePresence::Absent
                && candidate.value.presence == crate::ValuePresence::Absent)
            || required.stages.intersects(&candidate.value.stages);
        presence && stages
    };
    match demand {
        P1Projection::Infer => true,
        P1Projection::ValueDominant { value } => value_admits(value),
        P1Projection::Pair(required) => {
            value_admits(&required.value)
                && (required.pattern.stages.is_empty()
                    || required
                        .pattern
                        .stages
                        .intersects(&candidate.pattern.stages))
        }
    }
}

fn best_stage_rank(policy: &PolicyPair, phase: Phase) -> u8 {
    policy
        .value
        .stages
        .iter()
        .map(|stage| match (phase, stage) {
            (Phase::OpenStatic, PolicyStage::Meta) => 2,
            (Phase::OpenStatic, PolicyStage::Compile) => 1,
            (Phase::SealStatic, PolicyStage::Seal) => 2,
            (Phase::SealStatic, PolicyStage::Compile) => 1,
            (Phase::Runtime, PolicyStage::Runtime) => 1,
            _ => 0,
        })
        .max()
        .unwrap_or(0)
}

fn ordering_from_advantages(left: bool, right: bool) -> PolicyPartialOrdering {
    match (left, right) {
        (true, false) => PolicyPartialOrdering::Greater,
        (false, true) => PolicyPartialOrdering::Less,
        (false, false) => PolicyPartialOrdering::Equal,
        (true, true) => PolicyPartialOrdering::Incomparable,
    }
}

fn ordinary_result_identity(
    semantic_world: &mut SemanticWorld,
    selected: &PreparedCallCandidate,
    canonical_key: Option<&crate::MetaInvocationMaterialKey>,
    ambient_struct_owner: Option<SemanticOwnerId>,
    returned: SelectedBodyOutput,
) -> Result<
    Option<(
        TypeValueId,
        PatternValueId,
        Option<SemanticValueId>,
        ReturnedSemanticEntity,
    )>,
    Diagnostic,
> {
    match returned {
        SelectedBodyOutput::Material(MetaExecutionMaterial::IdentityType(value)) => {
            let represented = value.type_value;
            let Some(pattern) = semantic_world.type_value(represented).map(|t| t.pattern) else {
                return Ok(None);
            };
            let crate::CanonicalTypeObservation::Observed(whole) = value.type_observation;
            let complete_type = semantic_world
                .complete_type_by_whole_observation(whole)
                .cloned()
                .ok_or_else(|| {
                    Diagnostic::hard_error(
                        "forwarded CompleteType observation is not interned in this semantic world",
                        Some(value.provenance.clone()),
                    )
                })?;
            let carrier_value = semantic_world
                .core_type_projection_value(represented)
                .ok_or_else(|| {
                    Diagnostic::hard_error(
                        "forwarded CompleteType has no graph projection value",
                        Some(value.provenance.clone()),
                    )
                })?;
            Ok(Some((
                represented,
                pattern,
                None,
                ReturnedSemanticEntity::CompleteType(ReturnedCompleteType {
                    complete_type,
                    carrier_value,
                    pattern,
                    construction_material: None,
                }),
            )))
        }
        SelectedBodyOutput::Material(MetaExecutionMaterial::StructConstructionMaterial(
            mut value,
        )) => {
            let installed = if let Some(ambient_owner) = ambient_struct_owner {
                if let Some((_existing, binder)) =
                    semantic_world.ambient_struct_collision(ambient_owner, value.material_id)
                {
                    return Err(Diagnostic::hard_error(
                        ambient_struct_collision_message(binder),
                        Some(value.provenance.clone()),
                    ));
                }
                semantic_world.install_ambient_struct_type_value(
                    ambient_owner,
                    value.material_id,
                    value.canonical_pattern_value(),
                    selected.complete_result_view.pair.clone(),
                    value.provenance.clone(),
                )
            } else {
                let canonical_key = canonical_key
                    .expect("meta struct result requires a canonical MetaInstance key");
                let Some(placement_parent) =
                    semantic_world.callable_declaration_environment(selected.call_entry_value)
                else {
                    return Ok(None);
                };
                let meta_root = crate::MetaInstanceRoot {
                    meta_callable: canonical_key.callable,
                    placement_parent,
                };
                semantic_world.install_meta_struct_complete_type(
                    &meta_root,
                    canonical_key.clone(),
                    value.material_id,
                    value.canonical_pattern_value(),
                    selected.complete_result_view.pair.clone(),
                    value.provenance.clone(),
                )?
            };
            let Some((carrier_value, pattern, complete_type)) = installed else {
                return Err(Diagnostic::hard_error(
                    "struct result installation could not form its complete type",
                    Some(value.provenance.clone()),
                ));
            };
            value.canonical_type = Some(complete_type.lookup_key());
            Ok(Some((
                complete_type.lookup_key(),
                pattern,
                Some(carrier_value),
                ReturnedSemanticEntity::CompleteType(ReturnedCompleteType {
                    complete_type,
                    carrier_value,
                    pattern,
                    construction_material: Some(value),
                }),
            )))
        }
        SelectedBodyOutput::OrdinaryValue(value_id) => {
            Ok(semantic_world.value(value_id).map(|value| {
                (
                    value.type_value,
                    value.pattern,
                    Some(value.id),
                    ReturnedSemanticEntity::OrdinaryValue(value.id),
                )
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::result_pair_demand_admits;
    use crate::{
        P1Projection, PatternComponentPolicy, PolicyMode, PolicyPair, PolicyStage, PolicyView,
        StageSet, ValueComponentPolicy, ValuePresence,
    };

    fn view(
        value_stages: impl IntoIterator<Item = PolicyStage>,
        pattern_stages: impl IntoIterator<Item = PolicyStage>,
        mode: PolicyMode,
    ) -> PolicyView {
        let mut value_stage_set = StageSet::new();
        for stage in value_stages {
            value_stage_set.insert(stage);
        }
        let mut pattern_stage_set = StageSet::new();
        for stage in pattern_stages {
            pattern_stage_set.insert(stage);
        }
        PolicyView {
            pair: PolicyPair {
                value: ValueComponentPolicy {
                    stages: value_stage_set,
                    presence: ValuePresence::Present,
                },
                pattern: PatternComponentPolicy {
                    stages: pattern_stage_set,
                },
            },
            mode,
        }
    }

    #[test]
    fn result_pair_demand_is_a_pre_maxima_hard_coordinate() {
        let runtime = view(
            [PolicyStage::Runtime],
            [PolicyStage::Compile],
            PolicyMode::Const,
        );
        let compile = view(
            [PolicyStage::Compile],
            [PolicyStage::Compile],
            PolicyMode::Mut,
        );
        let runtime_demand = P1Projection::ValueDominant {
            value: runtime.pair.value.clone(),
        };
        assert!(result_pair_demand_admits(&runtime.pair, &runtime_demand));
        assert!(!result_pair_demand_admits(&compile.pair, &runtime_demand));

        let pair_demand = P1Projection::Pair(PolicyPair {
            value: runtime.pair.value.clone(),
            pattern: PatternComponentPolicy {
                stages: StageSet::from([PolicyStage::Seal]),
            },
        });
        assert!(!result_pair_demand_admits(&runtime.pair, &pair_demand));
        assert_ne!(
            runtime.mode, compile.mode,
            "mode remains a separate Bp coordinate"
        );
    }
}
