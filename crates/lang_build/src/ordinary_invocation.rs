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
//!   -> Callable filtering: (v |> type).Val2 contains ()    (Cc)
//!   -> resolve associated call entries from each val's Val2(C3)
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
    NormClosure, NormClosureBody, NormExpr, NormForm, NormOverloadStrategy, NormPattern,
    NormPatternElem, NormPolicySpec,
};

use crate::{
    body_entry_allows_execution,
    identity::{SemanticValueId, TypeValueId},
    invocation_frame::{
        InvocationCallableRef, InvocationExecutionEnv, InvocationFrame, InvocationLookupEnv,
        SelfPosition,
    },
    meta_candidate::CandidateBuildIdentityPlaceholder,
    meta_invocation::{MetaInvocationInput, MetaInvocationValue},
    model::{
        Diagnostic, ExecutionEnv, PolicyEnv, Provenance, ResolverCode, SourceCategory, SymbolId,
        SymbolKind, SymbolObject,
    },
    overload_pattern::{overload_args_from_classified_shape, SpecificityTuple},
    overload_set::{
        applicable_candidate_from_closure, declared_return_shape_from_closure,
        evaluate_selected_source_meta_body, evaluate_selected_source_meta_body_execution,
        ApplicableCandidate, CandidateApplicabilityFailure, MetaConstructionEffect,
        RestrictedOverloadFailure, RestrictedOverloadFailureKind, SelectedOverloadCandidate,
        VisibilityView,
    },
    pattern_head::TypeMaterializationState,
    policy_overload::{
        maximal_candidates, mutability_preference_rank, MutabilityActualFrame,
        MutabilityFormalFrame, MutabilityPattern,
    },
    policy_pair::{
        derive_function_object_view, elaborate_binding_result_demand, elaborate_explicit_p1,
        elaborate_formal_policy_pattern, normalize_p2_policy, project_p1, CapabilityRealization,
        ExplicitP1Position, FunctionObjectDeclarationPolicy, OutputModeDemand, P1Projection,
        PatternComponentPolicy, Phase, PolicyMode, PolicyPair, PolicyResultEntry, PolicyStage,
        PolicyView, ResultPolicyDemand, ValueComponentPolicy,
    },
    policy_transition::{
        compare_migration_endpoint_coordinates, project_migration_input_endpoint,
        project_migration_output_endpoint, PolicyMigrationRequest, PolicyPartialOrdering,
        PolicyTransitionRequest, SemanticValueRef,
    },
    product_shape::{
        ArgProductShape, FlattenedProductInvariant, FlattenedProductObject, ProductAtom,
        ProductMaterialRole, RawArgValueClass,
    },
    semantic_name_index::ResolverContext,
    semantic_owner::{SemanticOwnerId, SemanticSymbolIdentity},
    semantic_world::{
        ObjectPlaceId, OrdinaryCallEntry, OrdinaryCandidateRole, PatternValueId, ReturnShape,
        SemanticValuePayload, SemanticWorld, WritableContext,
    },
    type_argument::{classify_type_arguments_env_with_report, SemanticTypeEnv, TypeResolutionEnv},
    InvocationResidual, InvocationResult, NormalizedCallSite,
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
    pub formal_policy_frame: MutabilityFormalFrame,
    pub(crate) source_shape: Option<ApplicableCandidate>,
    pub(crate) core_invocation: Option<MetaInvocationInput>,
    pub(crate) intrinsic_body: Option<crate::semantic_world::OrdinaryIntrinsicBody>,
    pub return_shape: ReturnShape,
    pub candidate_role: OrdinaryCandidateRole,
    pub overload_strategy: NormOverloadStrategy,
    /// Migration input endpoint — projected from the Source formal, i.e. the
    /// first explicit Product formal after slot0.  NOT `function_object_p1`
    /// and NOT the self policy.  Computed once at A-stage so that A
    /// (admissibility) and Bp' (preference product) share the same coordinate.
    pub migration_input_endpoint: Option<PolicyPair>,
    /// Migration output endpoint — projected from the canonical P1 /
    /// `callable_value_policy` of the selected callable.  NOT a fresh P3 or
    /// `return_policy3`/`output_visibility_policy`/`migration_output_policy`.
    /// Computed once at A-stage alongside the input endpoint.
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
    semantic_world: &SemanticWorld,
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
        let place = semantic_world
            .value(selected.target_value)
            .map(|value| value.place)
            .ok_or_else(|| {
                Diagnostic::hard_error(
                    "selected invocation target has no resident Place for Writable validation",
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
    pub returned: OrdinaryReturnedValue,
    /// CompleteResultDomain — the complete result P2 compatibility domain
    /// returned by the ordinary callable (type/pattern compatibility
    /// information), before any consumer-specific `Project_out`.  This is
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
    /// CompleteResultDomain(P2) -> expose under callable P1 -> outer binding P1
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
/// CompleteResultDomain    = P2 compatibility domain
///                           (type/pattern compatibility information)
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
    /// Canonical P1 of the selected callable — the invocation result's
    /// outward visibility policy.  Never the result P2, never a fresh P3.
    pub outward_policy: PolicyPair,
    /// The completed result material (the P2-domain entries), exposed
    /// under the callable P1 window.
    pub material: Vec<PolicyResultEntry<SemanticValueRef, PatternValueId>>,
}

impl ExposedInvocationResult {
    /// `CompleteResultDomain(P2) -> expose under callable P1`.
    ///
    /// Every entry's stage / mutability window is intersected with the
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

/// Result of a `ClusterSymbol`-shaped invocation: a completed Symbol
/// cluster construction (plural values under one name at one position).
#[derive(Clone, Debug)]
pub struct ClusterSymbolResult {
    pub construction: crate::SymbolConstructionValue,
    /// Generated type definitions backing the construction's self-rooted
    /// type members, in member order.  The binding side uses these to
    /// expand the full namespace projection (field-function layer,
    /// ref/share projection namespaces, extraction interface) instead of a
    /// bare bound-type-value carrier.  Forwarded members contribute no
    /// entry here.
    pub generated_types: Vec<crate::GeneratedTypeDefinitionValue>,
    /// The complete result P2 of the selected callable.  Carried per
    /// result shape: the shape variant states the aggregation form, this
    /// field keeps the independent Policy coordinate alongside it.
    pub result_p2: PolicyPair,
    pub trace: OrdinaryPipelineTrace,
}

/// Result of a `Unit`-shaped invocation (`_: unit`): a value-less pure
/// shape.  Reserved carrier — no executable producer exists yet; the
/// declaration level validates the shape and invocation reports the
/// execution gap explicitly.
#[derive(Clone, Debug)]
pub struct UnitInvocationResult {
    pub result_p2: PolicyPair,
    pub trace: OrdinaryPipelineTrace,
}

/// The outcome of invoking a selected callable, split by the declared
/// return SHAPE — never by the execution stage.  A meta-stage callable
/// returning a single value produces `SingleMember` exactly like a
/// compile-stage one; `ClusterSymbol` is a shape fact, not a stage fact.
#[derive(Clone, Debug)]
pub enum InvocationOutcome {
    Unit(UnitInvocationResult),
    SingleMember(SingleMemberResult),
    ClusterSymbol(ClusterSymbolResult),
}

/// Complete type value returned by a world-connected invocation.
///
/// `construction_material` is replay/install material for compatibility
/// binding and namespace projection.  Semantic consumers use
/// `complete_type`; the material never defines type identity or equality.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReturnedCompleteType {
    pub complete_type: crate::CompleteTypeValue,
    pub carrier_value: SemanticValueId,
    pub pattern: PatternValueId,
    pub construction_material: Option<crate::GeneratedTypeDefinitionValue>,
}

#[derive(Clone, Debug)]
pub struct PolicyMigrationResult {
    pub invocation: SingleMemberResult,
    pub demanded_view: Vec<PolicyResultEntry<SemanticValueRef, PatternValueId>>,
}

/// Compatibility name for the legacy compile-to-runtime caller.  The result
/// is produced by the general same-Type migration authority.
pub type AtomicRuntimeMigrationResult = PolicyMigrationResult;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrdinaryReturnedValue {
    Meta(MetaInvocationValue),
    CompleteType(ReturnedCompleteType),
    /// A source body returned an already-existing semantic value. Ordinary
    /// non-migration invocation preserves that value identity; atomic runtime
    /// migration uses it as the migration source input (slot 1 in the
    /// invocation frame), not as a freshly-constructed result.
    ForwardedSemanticValue(SemanticValueId),
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
        failure: RestrictedOverloadFailure,
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

/// Meta-return self-root enforcement failure.  The body tried to
/// deliver a type member whose root is not the meta function plus its input
/// arguments.  No automatic re-rooting is performed.
fn meta_return_type_root_mismatch(
    message: String,
    provenance: &Provenance,
    trace: &OrdinaryPipelineTrace,
) -> OrdinaryInvocationFailure {
    OrdinaryInvocationFailure::MetaReturnTypeRootMismatch {
        diagnostic: Diagnostic::hard_error(message, Some(provenance.clone())),
        trace: trace.clone(),
    }
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

fn unsupported_member_initializer(
    message: String,
    provenance: &Provenance,
    trace: &OrdinaryPipelineTrace,
) -> OrdinaryInvocationFailure {
    OrdinaryInvocationFailure::SelectedBody {
        failure: RestrictedOverloadFailure {
            diagnostic: Diagnostic::hard_error(message, Some(provenance.clone()))
                .with_code(ResolverCode::UnsupportedSelectedMetaBody),
            kind: RestrictedOverloadFailureKind::UnsupportedSelectedMetaBody,
        },
        trace: trace.clone(),
    }
}

/// Placeholder overwrite target selection (scaffold).
///
/// `facet_matches[i]` states whether pending member `i` carries the
/// overwritten facet.  The placeholder accepts exactly one match and
/// rejects zero or several matches — a deliberately conservative choice
/// so the scaffold never silently picks a target.  This 0/1/many rule is
/// NOT the final ClusterSymbol write algebra; how a real expression-level
/// `=` selects or extends targets on a cluster is future work.
///
/// The source spelling `r = expr;` is fixed by the construction-effect
/// family, but the frozen v0.2 grammar has no expression-level `=`
/// operator yet, so today this selection is reachable only through
/// harvested effects, never from parsed fixture source.
fn select_overwrite_target(facet_matches: &[bool]) -> Result<usize, &'static str> {
    let mut targets = facet_matches
        .iter()
        .enumerate()
        .filter(|(_, matches)| **matches)
        .map(|(index, _)| index);
    let Some(index) = targets.next() else {
        return Err(
            "member overwrite (`r = expr;`) requires an existing member of the overwritten facet to replace",
        );
    };
    if targets.next().is_some() {
        return Err(
            "member overwrite (`r = expr;`) is ambiguous: several members carry the overwritten facet, and overwrite never falls back to declaration order",
        );
    }
    Ok(index)
}

/// Semantic resolution of a nested member-initializer call target.
enum SemanticCorePrimitiveResolution {
    CorePrimitive {
        callee: SymbolObject,
        primitive: crate::model::CoreMetaFunction,
    },
    NotCorePrimitive {
        name: String,
    },
    Unresolved,
}

/// Evaluated semantic object accepted by `let member::<return-target> = RHS`.
///
/// The split is on Val1 presence after RHS evaluation, never on the source
/// syntax family. A closure is retained here only as deferred function-object
/// material for the same semantic allocation primitive used by declarations.
///
/// ## Privilege boundary
///
/// Ordinary navigated `let f::t = expr` installs Val2 members only. It never
/// registers a member into the target Pattern's canonical structure — that
/// privilege belongs exclusively to `struct` inline construction and (future)
/// `inject`. A `null × P × Val2` RHS is an **associated type**, not a
/// structural child.
enum EvaluatedMetaInjectionRhs {
    /// Associated type: a `null × P × Val2` object installed as a Val2 member
    /// without entering the target Pattern's canonical structure.
    ///
    /// `complete_view` is the RHS's own complete pure-P member view
    /// (`value` is always `None`): the RHS Pattern Policy flows into the
    /// installed member view through the binding-P1 projection instead of
    /// being replaced by a fabricated empty Policy.
    AssociatedType {
        complete_view: PolicyResultEntry<SemanticValueId, PatternValueId>,
        type_value: TypeValueId,
    },
    ExistingValue(SemanticValueId),
    FunctionObject(NormClosure),
}

fn evaluate_meta_injection_rhs(
    semantic_world: &SemanticWorld,
    resolver_context: &ResolverContext,
    source_shape: &ApplicableCandidate,
    initializer: &NormExpr,
) -> Result<EvaluatedMetaInjectionRhs, String> {
    if let NormExpr::Closure(closure) = initializer {
        if closure.head.is_none() {
            return Err(
                "meta injection function-object initializer requires an explicit closure head"
                    .to_string(),
            );
        }
        return Ok(EvaluatedMetaInjectionRhs::FunctionObject(closure.clone()));
    }

    if let NormExpr::Name { text, .. } = initializer {
        if let Some(bound) = source_shape.bindings.get(text) {
            if let Some(value) = bound.semantic_value {
                return Ok(EvaluatedMetaInjectionRhs::ExistingValue(value));
            }
            if let (Some(pattern), Some(type_value)) = (bound.pattern_value, bound.value_type) {
                // The argument carried its own binding view in; the globally
                // reused TypeObject adapter for this TypeValue is transport
                // material and is only a last resort for Patterns that were
                // never named by a carrier at all.
                let complete_view = bound
                    .effective_view
                    .clone()
                    .unwrap_or_else(|| semantic_world.transport_pure_p_view(pattern));
                return Ok(EvaluatedMetaInjectionRhs::AssociatedType {
                    complete_view,
                    type_value,
                });
            }
        }
    }

    let (path, explicit_terminated) = match initializer {
        NormExpr::Name { text, .. } => (vec![text.clone()], false),
        NormExpr::Nav {
            components,
            explicit_terminated,
            ..
        } => {
            let path = components
                .iter()
                .map(|component| match component {
                    lang_syntax::NormNavComponent::Name { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| {
                    "meta injection RHS navigation contains an unsupported component".to_string()
                })?;
            (path, *explicit_terminated)
        }
        _ => {
            return Err(
                "meta injection RHS did not evaluate to a supported Pattern or value object"
                    .to_string(),
            );
        }
    };

    let identity = if explicit_terminated {
        let root = semantic_world.global_namespace().ok_or_else(|| {
            "meta injection exact global RHS has no semantic global namespace".to_string()
        })?;
        semantic_world
            .resolve_symbol_path_exact(&path, root)
            .ok_or_else(|| {
                format!(
                    "meta injection exact RHS `{}` is unresolved",
                    path.join("::")
                )
            })?
    } else {
        semantic_world
            .resolve_symbol_path(
                &path,
                resolver_context.current_namespace,
                &resolver_context.explicit_mount_roots,
                &resolver_context.default_mounts,
            )
            .map_err(|diagnostic| diagnostic.message)?
    };
    let symbol = semantic_world
        .symbol(identity)
        .ok_or_else(|| "meta injection RHS Symbol is not installed".to_string())?;
    if let Some(pattern) = symbol.pure_p_pattern() {
        let type_value = semantic_world.type_for_pattern(pattern).ok_or_else(|| {
            "meta injection RHS pure Pattern has no represented TypeValue".to_string()
        })?;
        // The RHS symbol's own pure-P member view is the binding-level Policy
        // authority.  The transported adapter view is the fallback for a
        // Pattern with no naming carrier, never for a real binding.
        let complete_view = symbol
            .pure_p_view()
            .cloned()
            .unwrap_or_else(|| semantic_world.transport_pure_p_view(pattern));
        return Ok(EvaluatedMetaInjectionRhs::AssociatedType {
            complete_view,
            type_value,
        });
    }
    match symbol.sibling_vals.as_slice() {
        [value] => Ok(EvaluatedMetaInjectionRhs::ExistingValue(*value)),
        [] => Err("meta injection RHS Symbol has no semantic object value".to_string()),
        _ => {
            Err("meta injection RHS Symbol is ambiguous across several sibling values".to_string())
        }
    }
}

/// Extract the source path of a nested call target (`struct`, `f::ns`).
fn call_target_path(target: &lang_syntax::NormExpr) -> Option<Vec<String>> {
    match target {
        lang_syntax::NormExpr::Name { text, .. } => Some(vec![text.clone()]),
        lang_syntax::NormExpr::Nav { components, .. } => components
            .iter()
            .map(|component| match component {
                lang_syntax::NormNavComponent::Name { name, .. } => Some(name.clone()),
                _ => None,
            })
            .collect(),
        _ => None,
    }
}

/// Resolve a nested core call target through the semantic world's recursive
/// Symbol substrate: ClusterSymbol → sibling vals → Val2 `()` call entry →
/// `core_primitive`.  No graph Symbol payload is read.
fn resolve_semantic_core_primitive_entry(
    semantic_world: &SemanticWorld,
    path: &[String],
    context: &ResolverContext,
) -> SemanticCorePrimitiveResolution {
    let Ok(identity) = semantic_world.resolve_symbol_path(
        path,
        context.current_namespace,
        &context.explicit_mount_roots,
        &context.default_mounts,
    ) else {
        return SemanticCorePrimitiveResolution::Unresolved;
    };
    let Some(cell) = semantic_world.symbol(identity) else {
        return SemanticCorePrimitiveResolution::Unresolved;
    };
    for sibling in &cell.sibling_vals {
        let entries = semantic_world
            .associated_values_for_value(*sibling, "()")
            .map(<[SemanticValueId]>::to_vec)
            .unwrap_or_default();
        for entry_value in entries {
            let Some(value) = semantic_world.value(entry_value) else {
                continue;
            };
            let SemanticValuePayload::CallEntry(entry) = &value.payload else {
                continue;
            };
            let Some(primitive) = entry.core_primitive else {
                return SemanticCorePrimitiveResolution::NotCorePrimitive {
                    name: entry.declaration_name.clone(),
                };
            };
            let callee = SymbolObject::placeholder(
                entry.backing_declaration,
                entry.declaration_name.clone(),
                SymbolKind::Placeholder,
                SourceCategory::DeclaredSymbol,
                entry.declaration_namespace,
                entry.provenance.clone(),
            );
            return SemanticCorePrimitiveResolution::CorePrimitive { callee, primitive };
        }
    }
    SemanticCorePrimitiveResolution::Unresolved
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
) -> Result<crate::MetaInstanceKey, OrdinaryInvocationFailure> {
    let arguments_product_addr = semantic_world
        .canonical_arguments_product_address(&shape.raw_args, &shape.flattened.atoms)
        .map_err(|diagnostic| OrdinaryInvocationFailure::CyclicVal2 {
            diagnostic,
            trace: trace.clone(),
        })?;
    Ok(crate::compute_canonical_meta_instance_key(
        callable,
        arguments_product_addr,
        provenance.clone(),
    ))
}

/// Evaluate one member-creation initializer of a source meta body.
///
/// The only self-rooted construction the restricted evaluator implements is a
/// core primitive meta call (e.g. `(t inner) |> struct`): binding names are
/// substituted with their call-site type spellings, the call is prepared and
/// invoked through the core meta machinery, and the generated type definition
/// is installed with the OUTER invocation's canonical instance key — so the
/// member's type root is `MetaCallableIdentity + Normalize(Arguments)` of the
/// outer meta function, not of the nested core call.
///
/// A bare name that resolves to an existing type root is a self-root
/// violation (`MetaReturnTypeRootMismatch`), never silently re-rooted.
#[allow(clippy::too_many_arguments)]
fn evaluate_source_meta_member_initializer(
    semantic_world: &mut SemanticWorld,
    materialization_state: &mut TypeMaterializationState,
    resolver_context: &ResolverContext,
    source_shape: &ApplicableCandidate,
    meta_root: &crate::MetaInstanceRoot,
    instance_key: &crate::MetaInstanceKey,
    result_policy: &PolicyPair,
    initializer: &lang_syntax::NormExpr,
    provenance: &Provenance,
    trace: &OrdinaryPipelineTrace,
) -> Result<(PatternValueId, crate::GeneratedTypeDefinitionValue), OrdinaryInvocationFailure> {
    use lang_syntax::NormExpr;
    if let NormExpr::Name { text, .. } = initializer {
        let names_existing_type_root = source_shape
            .bindings
            .get(text)
            .map(|bound| bound.value_type.is_some())
            .unwrap_or(false)
            || SemanticTypeEnv::new(semantic_world)
                .resolve_type_name(text, resolver_context)
                .is_some();
        if names_existing_type_root {
            return Err(meta_return_type_root_mismatch(
                format!(
                    "meta return type member must be rooted at the meta function plus its input arguments; `{text}` forwards an existing type root (construct a self-rooted value, e.g. `({text} field) |> struct`)"
                ),
                provenance,
                trace,
            ));
        }
        return Err(unsupported_member_initializer(
            format!(
                "member initializer name `{text}` is not a parameter binding or resolvable type"
            ),
            provenance,
            trace,
        ));
    }
    let substituted = substitute_binding_type_names(initializer, source_shape)
        .map_err(|message| unsupported_member_initializer(message, provenance, trace))?;
    let site = match crate::extract_single_call_site(&substituted) {
        Ok(site) => site,
        Err(_) => {
            return Err(unsupported_member_initializer(
                "member initializer form is outside the restricted evaluator; expected a core meta call such as `(t field) |> struct`"
                    .to_string(),
                provenance,
                trace,
            ));
        }
    };
    // The body resolves in the meta function's declaration namespace, like
    // the declaration-pattern context of the A-stage.
    let body_context = ResolverContext {
        current_namespace: source_shape.symbol.parent.unwrap_or_else(|| {
            resolver_context
                .explicit_mount_roots
                .first()
                .copied()
                .unwrap_or(resolver_context.current_namespace)
        }),
        explicit_mount_roots: resolver_context.explicit_mount_roots.clone(),
        default_mounts: resolver_context.default_mounts.clone(),
        current_policy: resolver_context.current_policy.clone(),
    };
    let Some(target_path) = call_target_path(&site.target) else {
        return Err(unsupported_member_initializer(
            "member initializer call target did not resolve to a callable symbol".to_string(),
            provenance,
            trace,
        ));
    };
    let (callee, primitive) = match resolve_semantic_core_primitive_entry(
        semantic_world,
        &target_path,
        &body_context,
    ) {
        SemanticCorePrimitiveResolution::CorePrimitive { callee, primitive } => (callee, primitive),
        SemanticCorePrimitiveResolution::NotCorePrimitive { name } => {
            return Err(unsupported_member_initializer(
                    format!(
                        "member initializer target `{name}` is not a core primitive meta function; nested source meta construction is outside the restricted evaluator"
                    ),
                    provenance,
                    trace,
                ));
        }
        SemanticCorePrimitiveResolution::Unresolved => {
            return Err(unsupported_member_initializer(
                "member initializer call target did not resolve to a callable symbol".to_string(),
                provenance,
                trace,
            ));
        }
    };
    let input = match crate::meta::prepare_resolved_core_meta_call_with_primitive(
        &callee,
        primitive,
        &site,
        &SemanticTypeEnv::new(semantic_world),
        &body_context,
        PolicyEnv::OpenStatic,
        ExecutionEnv::OpenStatic,
        CandidateBuildIdentityPlaceholder::default(),
        provenance.clone(),
    ) {
        Ok(input) => input,
        Err(error) => {
            let diagnostic = error.diagnostics.into_iter().next().unwrap_or_else(|| {
                Diagnostic::hard_error(
                    "member initializer core call preparation failed",
                    Some(provenance.clone()),
                )
            });
            return Err(OrdinaryInvocationFailure::SelectedCoreBody {
                diagnostic,
                trace: trace.clone(),
            });
        }
    };
    let mut input = input;
    attach_candidate_type_observations(semantic_world, &mut input, trace)?;
    let value = match crate::invoke_meta_callable_with_materialization_state(
        input,
        materialization_state,
    ) {
        InvocationResult::SemanticResult { value, .. } => value,
        InvocationResult::Residual(residual) => {
            return Err(OrdinaryInvocationFailure::Residual {
                residual,
                trace: trace.clone(),
            });
        }
        InvocationResult::Diagnostic(diagnostic) => {
            return Err(OrdinaryInvocationFailure::SelectedCoreBody {
                diagnostic,
                trace: trace.clone(),
            });
        }
    };
    match value {
        MetaInvocationValue::GeneratedTypeDefinitionValue(mut value) => {
            // `OwnerStrategy::ExplicitPrivilegedOwnerRule`: a nested `struct`
            // inside a meta body never roots at `struct` itself — the outer
            // meta invocation injects its own MetaInstance root (meta
            // function + normalized arguments) as the constructed owner.
            //
            // FUTURE GUARD: when the restricted evaluator is extended to
            // support in-place closure invocations within meta bodies, this
            // property must be preserved: struct sees through intermediate
            // closures and resolves its owner at the meta entry scope, not
            // at the in-place closure's Self scope.  In-place closures are
            // control-flow mechanisms transparent to struct navigation in
            // meta context; only non-meta contexts observe them as
            // affecting navigation names (spec §7.2).
            let installed = semantic_world
                .install_generated_type_value(
                    meta_root,
                    instance_key.clone(),
                    value.type_definition_id,
                    value.canonical_pattern_value(),
                    result_policy.clone(),
                    value.provenance.clone(),
                )
                .map_err(|diagnostic| OrdinaryInvocationFailure::SelectedCoreBody {
                    diagnostic,
                    trace: trace.clone(),
                })?;
            let Some((_value_id, pattern, canonical_type)) = installed else {
                return Err(unsupported_member_initializer(
                    "generated type member installation failed".to_string(),
                    provenance,
                    trace,
                ));
            };
            value.canonical_type = Some(canonical_type);
            Ok((pattern, value))
        }
        _ => Err(unsupported_member_initializer(
            "member initializer core call did not produce a generated type definition".to_string(),
            provenance,
            trace,
        )),
    }
}

/// Substitute parameter binding names in a body initializer with the type
/// spelling bound at the call site, so the nested core call can resolve them
/// through the ordinary type-object path.  Pack bindings and bindings without
/// a substitutable spelling are rejected.
fn substitute_binding_type_names(
    expr: &lang_syntax::NormExpr,
    source_shape: &ApplicableCandidate,
) -> Result<lang_syntax::NormExpr, String> {
    use lang_syntax::{NormExpr, NormProduct, NormProductElem};
    match expr {
        NormExpr::PolicyLet {
            policy,
            operand,
            origin,
        } => Ok(NormExpr::PolicyLet {
            policy: policy.clone(),
            operand: Box::new(substitute_binding_type_names(operand, source_shape)?),
            origin: origin.clone(),
        }),
        NormExpr::Name { text, origin } => {
            if let Some(bound) = source_shape.bindings.get(text) {
                let Some(spelling) = bound.top_pattern_name.clone() else {
                    return Err(format!(
                        "bound argument `{text}` has no substitutable type spelling in the restricted evaluator"
                    ));
                };
                return Ok(NormExpr::Name {
                    text: spelling,
                    origin: origin.clone(),
                });
            }
            if source_shape.pack_bindings.contains_key(text) {
                return Err(format!(
                    "pack binding `{text}` forwarding is outside the restricted evaluator"
                ));
            }
            Ok(expr.clone())
        }
        NormExpr::Call {
            source,
            target,
            origin,
        } => Ok(NormExpr::Call {
            source: NormProduct {
                elements: source
                    .elements
                    .iter()
                    .map(|elem| match elem {
                        NormProductElem::Expr(inner) => {
                            substitute_binding_type_names(inner, source_shape)
                                .map(NormProductElem::Expr)
                        }
                        NormProductElem::Unit { origin } => Ok(NormProductElem::Unit {
                            origin: origin.clone(),
                        }),
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                origin: source.origin.clone(),
            },
            target: Box::new(substitute_binding_type_names(target, source_shape)?),
            origin: origin.clone(),
        }),
        NormExpr::Product(product) => Ok(NormExpr::Product(NormProduct {
            elements: product
                .elements
                .iter()
                .map(|elem| match elem {
                    NormProductElem::Expr(inner) => {
                        substitute_binding_type_names(inner, source_shape)
                            .map(NormProductElem::Expr)
                    }
                    NormProductElem::Unit { origin } => Ok(NormProductElem::Unit {
                        origin: origin.clone(),
                    }),
                })
                .collect::<Result<Vec<_>, _>>()?,
            origin: product.origin.clone(),
        })),
        _ => Ok(expr.clone()),
    }
}

pub fn invoke_policy_migration(
    semantic_world: &mut SemanticWorld,
    materialization_state: &mut TypeMaterializationState,
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
        materialization_state,
        OrdinaryCandidateOrigin::SourceSymbol(cluster),
        target_members,
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
    let InvocationOutcome::SingleMember(invocation) = invocation else {
        return Err(OrdinaryInvocationFailure::NoFullyAdmissibleCandidate {
            first_diagnostic: Some(Diagnostic::hard_error(
                "migration selected a non-single-member return shape, expected ordinary transport",
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

/// Migration-only adapter for the historical bounded static-to-runtime
/// request.  It supplies one canonical request and never owns selection.
pub fn invoke_atomic_runtime_migration(
    semantic_world: &mut SemanticWorld,
    materialization_state: &mut TypeMaterializationState,
    request: &PolicyTransitionRequest,
    resolver_context: &ResolverContext,
) -> Result<AtomicRuntimeMigrationResult, OrdinaryInvocationFailure> {
    let request = PolicyMigrationRequest::from_atomic_runtime(request);
    invoke_policy_migration(
        semantic_world,
        materialization_state,
        &request,
        resolver_context,
    )
}

/// Invoke all value members of one resolved semantic Symbol.
///
/// C0 is the resolved ClusterSymbol's canonical member views — not a flat
/// value-id list.  Pure-P members (value = None) stay legal cluster members
/// but are not invocation candidates; exposure and callability are decided
/// per member view downstream.
pub fn invoke_symbol_ordinary(
    semantic_world: &mut SemanticWorld,
    materialization_state: &mut TypeMaterializationState,
    symbol: SemanticSymbolIdentity,
    call_site: &NormalizedCallSite,
    resolver_context: &ResolverContext,
    context: OrdinaryInvocationContext<'_>,
    provenance: Provenance,
) -> Result<InvocationOutcome, OrdinaryInvocationFailure> {
    invoke_host_member_symbol_ordinary(
        semantic_world,
        materialization_state,
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
    materialization_state: &mut TypeMaterializationState,
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
    invoke_target_values(
        semantic_world,
        materialization_state,
        OrdinaryCandidateOrigin::SourceSymbol(symbol),
        target_members,
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
    materialization_state: &mut TypeMaterializationState,
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
        materialization_state,
        OrdinaryCandidateOrigin::PatternAssociatedCallEntry(pattern),
        target_members,
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
    materialization_state: &mut TypeMaterializationState,
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
        materialization_state,
        OrdinaryCandidateOrigin::PatternAssociatedValue(pattern),
        target_members,
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
/// Callable(v) iff (v |> type).Val2 contains `()` — i.e. following
/// the value's own recursive Val1×P×Val2 structure yields a call entry.
fn filter_callable(
    semantic_world: &SemanticWorld,
    values: &[SemanticValueId],
) -> Vec<CallableTarget> {
    values
        .iter()
        .filter_map(|value| {
            let entries = semantic_world
                .associated_values_for_value(*value, "()")
                .map(|e| e.to_vec())
                .unwrap_or_default();
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
    materialization_state: &mut TypeMaterializationState,
    origin: OrdinaryCandidateOrigin,
    target_members: Vec<PolicyResultEntry<SemanticValueId, PatternValueId>>,
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
        let declaration_identity = SymbolObject::placeholder(
            entry.backing_declaration,
            entry.declaration_name.clone(),
            SymbolKind::Placeholder,
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
                current_policy: resolver_context.current_policy.clone(),
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
                        first_diagnostic.get_or_insert(diagnostic);
                        continue;
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
                        first_diagnostic.get_or_insert(diagnostic);
                        continue;
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
                                .map(|complete| complete.core)
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
                Err(CandidateApplicabilityFailure::Inapplicable(diagnostic))
                | Err(CandidateApplicabilityFailure::UnsupportedParameterPattern(diagnostic))
                | Err(CandidateApplicabilityFailure::UnsupportedCandidateShape(diagnostic)) => {
                    first_diagnostic.get_or_insert(diagnostic);
                    continue;
                }
            };
            if let Err(diagnostic) = apply_written_self_structure(
                &mut source_shape,
                &entry,
                &target,
                semantic_world,
                resolver_context,
                provenance.clone(),
            ) {
                first_diagnostic.get_or_insert(diagnostic);
                continue;
            }
            if let Err(diagnostic) = validate_explicit_value_type_annotations(
                &entry,
                &classified.classified_shape,
                semantic_world,
                resolver_context,
                provenance.clone(),
            ) {
                first_diagnostic.get_or_insert(diagnostic);
                continue;
            }
            let formal_policy_frame = match formal_mutability_frame(&entry, provenance.clone()) {
                Ok(frame) => frame,
                Err(diagnostic) => {
                    first_diagnostic.get_or_insert(diagnostic);
                    continue;
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
                CandidateBuildIdentityPlaceholder::default(),
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
                MutabilityFormalFrame {
                    self_pattern: PolicyMode::Plain,
                    explicit_parameter_patterns: vec![PolicyMode::Plain; frame_args.arity],
                },
                // S7 — same canonical P1 authority as the source arm.  The
                // core candidate's function-object P1 is the declared
                // canonical P1 (`callable_value_policy`), NOT the result P2.
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
            let SemanticValuePayload::TypeObject {
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
                        && target_snapshot.lookup_key == *receiver_target
                }
                crate::semantic_world::OrdinaryIntrinsicBody::Delete
                | crate::semantic_world::OrdinaryIntrinsicBody::FailSelected => {
                    target_snapshot.lookup_key == *receiver_target
                }
            };
            if !applicable {
                continue;
            }
            (
                None,
                None,
                Some(intrinsic.clone()),
                MutabilityFormalFrame {
                    self_pattern: PolicyMode::Plain,
                    explicit_parameter_patterns: vec![PolicyMode::Plain],
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
        //     after slot0.  NOT `function_object_p1`, NOT self policy.
        // §4.2 output endpoint = canonical P1 / `callable_value_policy`.
        //     NOT a fresh P3 or return_policy3/output_visibility_policy.
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
            return_shape: entry.return_shape,
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

    let actual_frame = MutabilityActualFrame {
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
    // carrier through an ordinary MetaInstanceKey would invent semantic
    // identity that the language does not have.
    let mut canonical_instance_key =
        if selected.return_shape != ReturnShape::ClusterSymbol || is_ambient_struct {
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

    // A declared `Unit` shape is validated at the declaration boundary but
    // has no executable producer yet: report the execution gap explicitly
    // instead of silently misrouting the result into a single-member or
    // cluster carrier.
    if matches!(selected.return_shape, ReturnShape::Unit) {
        return Err(OrdinaryInvocationFailure::SelectedCoreBody {
            diagnostic: Diagnostic::hard_error(
                "invocation of a Unit return shape is future work: \
                 the declaration is validated, but no executable producer exists yet",
                Some(provenance.clone()),
            ),
            trace,
        });
    }

    let meta_construction_result = if selected.return_shape == ReturnShape::ClusterSymbol {
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
        // called directly attaches its generated type to the ambient
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
        // only the degenerate top-level fallback.
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
        let construction_context = crate::ConstructionEvaluationContext::current(authority.clone());
        let cid = semantic_world.begin_cluster_construction(authority, owner, provenance.clone());

        // Generated type definitions harvested for the binding side's
        // namespace projection expansion (field layer, ref/share views).
        let mut generated_types: Vec<crate::GeneratedTypeDefinitionValue> = Vec::new();

        // Each member contribution carries the member's own value Policy
        // and Pattern Policy (S1); nothing is degraded to a bare
        // PatternValueId.
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
            let value = match crate::invoke_meta_callable_with_materialization_state(
                core_input,
                materialization_state,
            ) {
                InvocationResult::SemanticResult { value, .. } => value,
                InvocationResult::Residual(residual) => {
                    return Err(OrdinaryInvocationFailure::Residual {
                        residual,
                        trace: trace.clone(),
                    });
                }
                InvocationResult::Diagnostic(diagnostic) => {
                    return Err(OrdinaryInvocationFailure::SelectedCoreBody { diagnostic, trace });
                }
            };
            match &value {
                MetaInvocationValue::ForwardedValue(value) => {
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
                MetaInvocationValue::GeneratedTypeDefinitionValue(value) => {
                    let installed = match owner_strategy {
                        crate::OwnerStrategy::AmbientStructScope => {
                            let ambient_owner = ambient_owner
                                .expect("AmbientStructScope construction carries an ambient owner");
                            if let Some((_existing, binder)) = semantic_world
                                .ambient_struct_collision(ambient_owner, value.type_definition_id)
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
                                value.type_definition_id,
                                value.canonical_pattern_value(),
                                selected.complete_result_view.pair.clone(),
                                value.provenance.clone(),
                            )
                        }
                        _ => match semantic_world.install_generated_type_value(
                            &meta_root,
                            canonical_instance_key
                                .as_ref()
                                .expect("ordinary generated meta type has an instance key")
                                .clone(),
                            value.type_definition_id,
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
                    if let Some((_value_id, pattern, canonical_type)) = installed {
                        semantic_world
                            .contribute_cluster_member_view(cid, pure_p_member_view(pattern));
                        let mut value = value.clone();
                        value.canonical_type = Some(canonical_type);
                        generated_types.push(value);
                    }
                }
                MetaInvocationValue::GeneratedConstructionValue(value) => {
                    if let Some(pattern) = semantic_world.allocate_meta_result_pattern(
                        &meta_root,
                        canonical_instance_key
                            .as_ref()
                            .expect("ordinary generated meta value has an instance key")
                            .clone(),
                        value.provenance.clone(),
                    ) {
                        semantic_world
                            .contribute_cluster_member_view(cid, pure_p_member_view(pattern));
                    }
                }
            }
        } else if let Some(source_shape) = &selected.source_shape {
            // Source meta body: harvest the construction effects and the
            // validated terminal from the body evaluator, then evaluate
            // each effect here — the construction owner has semantic
            // world and materialization access; the evaluator does not.
            //
            // S8 — `SelectedOverloadCandidate` here is a plain data
            // carrier for the shared body evaluator, built from the
            // canonical pipeline's own prepared candidate.  The legacy
            // restricted selector is not involved.
            let legacy_selected = SelectedOverloadCandidate {
                symbol: source_shape.symbol.clone(),
                source_callable: source_shape.source_callable.clone(),
                bindings: source_shape.bindings.clone(),
                pack_bindings: source_shape.pack_bindings.clone(),
                specificity: source_shape.specificity,
                overload_strategy: source_shape.overload_strategy.clone(),
                return_slot_name: source_shape.return_slot_name.clone(),
            };
            let execution = match evaluate_selected_source_meta_body_execution(
                &SemanticTypeEnv::new(&*semantic_world),
                resolver_context,
                &legacy_selected,
            ) {
                Ok(execution) => execution,
                Err(failure) => {
                    return Err(OrdinaryInvocationFailure::SelectedBody { failure, trace });
                }
            };
            // B5 — full member-view ledger: every pending member carries
            // its own projected member view (value, value Policy, Pattern,
            // Pattern Policy) plus the binding projection it was created
            // under, never just a bare type id.  The executable slice is
            // still narrow — self-rooted generated type members only; val
            // members and alias members remain explicit future work — but
            // the ledger itself is member-view shaped, so widening the
            // slice never reshapes the ledger.  The three effects are
            // distinct and never collapse into one "append" mechanism;
            // the ledger is contributed once at the end so an overwrite
            // really replaces instead of stacking.
            struct PendingClusterMember {
                /// The member's binding P1 projection; an overwrite
                /// re-projects the new value under this same binding.
                projection: P1Projection,
                view: PolicyResultEntry<SemanticValueId, PatternValueId>,
                generated: crate::GeneratedTypeDefinitionValue,
            }
            let mut pending_members: Vec<PendingClusterMember> = Vec::new();
            // B3 — the effect index is the declaration event: replaying the
            // same canonical instance replays the same effect list, so each
            // injecting event re-finds its own injected value by position.
            for (effect_index, effect) in execution.effects.iter().enumerate() {
                match effect {
                    MetaConstructionEffect::AddMember {
                        initializer,
                        binding_p1,
                    } => {
                        let (pattern, generated) = evaluate_source_meta_member_initializer(
                            semantic_world,
                            materialization_state,
                            resolver_context,
                            source_shape,
                            &meta_root,
                            canonical_instance_key
                                .as_ref()
                                .expect("source meta construction has an instance key"),
                            &selected.complete_result_view.pair,
                            initializer,
                            &provenance,
                            &trace,
                        )?;
                        // B6 — the member's own written P1 projects over
                        // the RHS complete member views; members never
                        // collapse onto the function P2.
                        let demand = elaborate_binding_result_demand(
                            binding_p1.as_ref(),
                            provenance.clone(),
                        )
                        .map_err(|diagnostic| {
                            OrdinaryInvocationFailure::SelectedCoreBody {
                                diagnostic,
                                trace: trace.clone(),
                            }
                        })?;
                        let projection = demand.pair_query.clone();
                        let Some(view) = project_p1(&projection, &[pure_p_member_view(pattern)])
                            .into_iter()
                            .next()
                        else {
                            return Err(unsupported_member_initializer(
                                "member binding P1 admits no view of its initializer's complete result"
                                    .to_string(),
                                &provenance,
                                &trace,
                            ));
                        };
                        pending_members.push(PendingClusterMember {
                            projection,
                            view,
                            generated,
                        });
                    }
                    MetaConstructionEffect::PlaceholderOverwrite { initializer } => {
                        let (pattern, generated) = evaluate_source_meta_member_initializer(
                            semantic_world,
                            materialization_state,
                            resolver_context,
                            source_shape,
                            &meta_root,
                            canonical_instance_key
                                .as_ref()
                                .expect("source meta construction has an instance key"),
                            &selected.complete_result_view.pair,
                            initializer,
                            &provenance,
                            &trace,
                        )?;
                        // Placeholder target selection (scaffold, not the
                        // final ClusterSymbol write algebra): while
                        // expression-level `=` does not exist, the write
                        // addresses the unique existing pure-P member so
                        // existing-target addressing itself is exercised.
                        let index = select_overwrite_target(
                            &pending_members
                                .iter()
                                .map(|member| member.view.value.is_none())
                                .collect::<Vec<_>>(),
                        )
                        .map_err(|message| {
                            unsupported_member_initializer(message.to_string(), &provenance, &trace)
                        })?;
                        // Scaffold behavior: the placeholder replaces the
                        // member's value under the member's own binding P1.
                        // Whether the final `=` on a ClusterSymbol replaces,
                        // adds facets, or rebinds is NOT decided here.
                        let projection = pending_members[index].projection.clone();
                        let Some(view) = project_p1(&projection, &[pure_p_member_view(pattern)])
                            .into_iter()
                            .next()
                        else {
                            return Err(unsupported_member_initializer(
                                "the overwritten member's binding P1 admits no view of the new value"
                                    .to_string(),
                                &provenance,
                                &trace,
                            ));
                        };
                        pending_members[index] = PendingClusterMember {
                            projection,
                            view,
                            generated,
                        };
                    }
                    MetaConstructionEffect::InjectMember {
                        member_name,
                        initializer,
                        binding_p1,
                    } => {
                        let Some(target_index) = pending_members
                            .iter()
                            .position(|member| member.view.value.is_none())
                        else {
                            return Err(unsupported_member_initializer(
                                "meta injection requires a constructed type member before `let member::<return-target> = RHS;`"
                                    .to_string(),
                                &provenance,
                                &trace,
                            ));
                        };
                        let target_pattern = pending_members[target_index].view.pattern;
                        // Eagerly register the target pattern's cluster
                        // ownership so the injection ownership check in
                        // inject_associated_* passes. The final
                        // contribute_cluster_member_view at the end is still
                        // needed for the full member-view ledger, but the
                        // pattern_clusters entry is required now.
                        semantic_world.ensure_pattern_cluster_ownership(target_pattern, cid);
                        let member_creation_failure = |failure| {
                            unsupported_member_initializer(
                                format!(
                                    "associated member creation is not authorized at this evaluation point: {failure:?}"
                                ),
                                &provenance,
                                &trace,
                            )
                        };
                        let selected_core_body =
                            |diagnostic: Diagnostic| OrdinaryInvocationFailure::SelectedCoreBody {
                                diagnostic,
                                trace: trace.clone(),
                            };
                        let evaluated = evaluate_meta_injection_rhs(
                            semantic_world,
                            resolver_context,
                            source_shape,
                            initializer,
                        )
                        .map_err(|message| {
                            unsupported_member_initializer(message, &provenance, &trace)
                        })?;
                        match evaluated {
                            EvaluatedMetaInjectionRhs::AssociatedType {
                                complete_view,
                                type_value,
                            } => {
                                debug_assert!(
                                    complete_view.value.is_none(),
                                    "an associated-type RHS view is a pure-P view"
                                );
                                debug_assert_eq!(
                                    semantic_world.type_for_pattern(complete_view.pattern),
                                    Some(type_value),
                                    "evaluated associated-type injection keeps its TypeValue/Pattern pair"
                                );
                                // The binding's written P1 restricts the RHS
                                // view exactly as on the ordinary value path:
                                // a type does not get a second P1 discipline
                                // for lacking a Val1. The projected pure-P
                                // view is what gets installed as the
                                // associated Symbol's member view.
                                let demand = elaborate_binding_result_demand(
                                    binding_p1.as_ref(),
                                    provenance.clone(),
                                )
                                .map_err(selected_core_body)?;
                                let projection = demand.pair_query;
                                let Some(view) =
                                    project_p1(&projection, std::slice::from_ref(&complete_view))
                                        .into_iter()
                                        .next()
                                else {
                                    return Err(unsupported_member_initializer(
                                        "meta associated-type binding P1 admits no view of the RHS pure Pattern"
                                            .to_string(),
                                        &provenance,
                                        &trace,
                                    ));
                                };
                                // PRIVILEGE BOUNDARY: ordinary navigated `let f::t = expr`
                                // installs the pure type object as the pure-P member of
                                // the associated Val2 Symbol `C_f` in the target type
                                // member's Val2 (`Val2(T_t)[f] = C_f`). It does NOT
                                // become a member of the HOST cluster and does NOT
                                // register into the target Pattern's canonical
                                // structure. Only `struct` inline construction and
                                // (future) `inject` hold structural registration
                                // privilege.
                                if !semantic_world.associated_type_member_is_replay(
                                    target_pattern,
                                    &member_name,
                                    &view,
                                    type_value,
                                ) {
                                    let creation = semantic_world
                                        .can_create_member_here(
                                            target_pattern,
                                            &construction_context,
                                        )
                                        .map_err(member_creation_failure)?;
                                    semantic_world
                                        .create_associated_type_member(
                                            &creation,
                                            &member_name,
                                            view,
                                            type_value,
                                            provenance.clone(),
                                        )
                                        .map_err(selected_core_body)?;
                                }
                            }
                            EvaluatedMetaInjectionRhs::ExistingValue(value) => {
                                let object = semantic_world
                                    .value(value)
                                    .expect("evaluated injection value is installed")
                                    .clone();
                                let demand = elaborate_binding_result_demand(
                                    binding_p1.as_ref(),
                                    provenance.clone(),
                                )
                                .map_err(selected_core_body)?;
                                let projection = demand.pair_query;
                                let complete = PolicyResultEntry {
                                    value: Some(value),
                                    pattern: object.pattern,
                                    view: object.policy_view(),
                                };
                                let Some(view) =
                                    project_p1(&projection, &[complete]).into_iter().next()
                                else {
                                    return Err(unsupported_member_initializer(
                                        "meta associated-value binding P1 admits no view of the evaluated RHS"
                                            .to_string(),
                                        &provenance,
                                        &trace,
                                    ));
                                };
                                if !semantic_world.associated_value_member_is_replay(
                                    target_pattern,
                                    member_name,
                                    &view,
                                ) {
                                    let creation = semantic_world
                                        .can_create_member_here(
                                            target_pattern,
                                            &construction_context,
                                        )
                                        .map_err(member_creation_failure)?;
                                    semantic_world
                                        .create_associated_existing_value_member(
                                            &creation,
                                            member_name,
                                            view,
                                            provenance.clone(),
                                        )
                                        .map_err(selected_core_body)?;
                                }
                            }
                            EvaluatedMetaInjectionRhs::FunctionObject(closure) => {
                                let head = closure.head.as_ref().expect(
                                    "evaluated function-object injection has an explicit head",
                                );
                                let Some(annotation) = &head.call_policy else {
                                    return Err(selected_core_body(Diagnostic::hard_error(
                                        "meta Val2 injection initializer requires a P2 annotation such as `: compile ->`",
                                        Some(provenance.clone()),
                                    )));
                                };
                                let result_p2 = normalize_p2_policy(annotation, provenance.clone())
                                    .map_err(selected_core_body)?;
                                let return_shape = declared_return_shape_from_closure(&closure)
                                    .map_err(selected_core_body)?;
                                let outer_p1_explicit = elaborate_explicit_p1(
                                    binding_p1.as_ref(),
                                    &result_p2.pair,
                                    ExplicitP1Position::OuterBinding,
                                    provenance.clone(),
                                )
                                .map_err(selected_core_body)?;
                                let function_view = derive_function_object_view(
                                    &result_p2,
                                    &FunctionObjectDeclarationPolicy::default(),
                                );
                                let construction_event = u32::try_from(effect_index)
                                    .expect("meta body effect count fits in u32");
                                let replay = semantic_world
                                    .replay_associated_function_member(
                                        cid,
                                        member_name,
                                        construction_event,
                                        &closure,
                                        outer_p1_explicit.as_ref(),
                                        &function_view,
                                        &result_p2,
                                        provenance.clone(),
                                    )
                                    .map_err(selected_core_body)?;
                                if replay.is_none() {
                                    let creation = semantic_world
                                        .can_create_member_here(
                                            target_pattern,
                                            &construction_context,
                                        )
                                        .map_err(member_creation_failure)?;
                                    semantic_world
                                        .create_associated_function_member(
                                            &creation,
                                            member_name,
                                            construction_event,
                                            selected.backing_declaration,
                                            &closure,
                                            outer_p1_explicit.as_ref(),
                                            &function_view,
                                            result_p2,
                                            return_shape,
                                            provenance.clone(),
                                        )
                                        .map_err(selected_core_body)?;
                                }
                            }
                        }
                    }
                }
            }
            for member in pending_members {
                semantic_world.contribute_cluster_member_view(cid, member.view);
                generated_types.push(member.generated);
            }
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
            generated_types,
            result_p2: selected.body_entry_view.pair.clone(),
            trace: trace.clone(),
        })
    } else {
        None
    };

    if let Some(meta_result) = meta_construction_result {
        return Ok(InvocationOutcome::ClusterSymbol(meta_result));
    }

    let mut returned = if let Some(source_shape) = &selected.source_shape {
        // S8 — carrier construction only; see the meta-construction arm
        // above.  Not legacy-selector output.
        let legacy_selected = SelectedOverloadCandidate {
            symbol: source_shape.symbol.clone(),
            source_callable: source_shape.source_callable.clone(),
            bindings: source_shape.bindings.clone(),
            pack_bindings: source_shape.pack_bindings.clone(),
            specificity: source_shape.specificity,
            overload_strategy: source_shape.overload_strategy.clone(),
            return_slot_name: source_shape.return_slot_name.clone(),
        };
        if !selected.is_delete() {
            if let Some(value) = forwarded_semantic_body_value(&selected) {
                OrdinaryReturnedValue::ForwardedSemanticValue(value)
            } else {
                match evaluate_selected_source_meta_body(
                    &SemanticTypeEnv::new(&*semantic_world),
                    resolver_context,
                    &legacy_selected,
                ) {
                    Ok(value) => OrdinaryReturnedValue::Meta(value),
                    Err(failure) => {
                        return Err(OrdinaryInvocationFailure::SelectedBody { failure, trace });
                    }
                }
            }
        } else {
            match evaluate_selected_source_meta_body(
                &SemanticTypeEnv::new(&*semantic_world),
                resolver_context,
                &legacy_selected,
            ) {
                Ok(value) => OrdinaryReturnedValue::Meta(value),
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
        match crate::invoke_meta_callable_with_materialization_state(
            core_input,
            materialization_state,
        ) {
            InvocationResult::SemanticResult { value, .. } => OrdinaryReturnedValue::Meta(value),
            InvocationResult::Residual(residual) => {
                return Err(OrdinaryInvocationFailure::Residual { residual, trace });
            }
            InvocationResult::Diagnostic(diagnostic) => {
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
                OrdinaryReturnedValue::ForwardedSemanticValue(constructed)
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
            OrdinaryReturnedValue::Meta(
                MetaInvocationValue::GeneratedTypeDefinitionValue(_)
                    | MetaInvocationValue::GeneratedConstructionValue(_)
            )
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
        &mut returned,
    )
    .map_err(|diagnostic| OrdinaryInvocationFailure::SelectedCoreBody {
        diagnostic,
        trace: trace.clone(),
    })?;
    let Some((result_type, pattern, returned_value)) = identity else {
        let result_type = match &returned {
            OrdinaryReturnedValue::Meta(value) => compatibility_meta_material_type(value),
            OrdinaryReturnedValue::CompleteType(value) => value.complete_type.lookup_key,
            OrdinaryReturnedValue::ForwardedSemanticValue(value) => {
                semantic_world
                    .value(*value)
                    .expect("forwarded receiver exists")
                    .type_value
            }
        };
        return Err(OrdinaryInvocationFailure::ResultTypeHasNoPattern {
            type_value: result_type,
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
        // The migration output endpoint has exactly one
        // authority: the coordinate projected from the canonical P1
        // (`callable_value_policy`) at A-stage and stored on the candidate.
        // Do not re-project from the result P2 here; that would be a second
        // output authority (a de-facto P3).
        if selected.migration_output_endpoint.is_none() {
            return Err(OrdinaryInvocationFailure::MigrationOutputProjectionFailed { trace });
        }
    }

    // CompleteResultDomain — these entries carry the result P2
    // (type/pattern compatibility information) only.  The outward
    // visibility of the invocation result is NOT this P2: it is the
    // canonical P1 layer, derived on demand by
    // `SingleMemberResult::exposed()`.
    let complete_result = vec![PolicyResultEntry {
        value: returned_value.map(|id| SemanticValueRef {
            id,
            type_value: result_type,
        }),
        pattern,
        view: selected.complete_result_view.clone(),
    }];

    Ok(InvocationOutcome::SingleMember(SingleMemberResult {
        selected,
        returned,
        complete_result,
        trace,
    }))
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
                // This TypeObject filter is a defensive
                // guard, NOT ontology leakage.  Pure-P/type views carry
                // `value=None` and are already skipped by `view.value?`
                // above.  This branch only catches legacy TypeObject carriers
                // that somehow ended up in a value-bearing view — ordinary
                // invocation must never treat a type adapter as a runtime
                // argument.  The filter proves TypeObject is NOT an
                // indispensable hidden Val1: ordinary algorithms explicitly
                // reject it rather than depend on it.
                if matches!(object.payload, SemanticValuePayload::TypeObject { .. })
                    || !view
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

fn formal_mutability_frame(
    entry: &OrdinaryCallEntry,
    provenance: Provenance,
) -> Result<MutabilityFormalFrame, Diagnostic> {
    let head = entry
        .closure
        .as_ref()
        .and_then(|closure| closure.head.as_ref())
        .ok_or_else(|| {
            Diagnostic::hard_error(
                "ordinary call entry has no explicit closure head",
                Some(provenance.clone()),
            )
        })?;
    let frame = head.formal_frame();
    let self_pattern = match frame.written_self {
        // The written-self slot policy is explicit P1 material: stage /
        // presence / Pattern atoms are legal there and are
        // reconciled by `canonical_function_object_p1` at registration.
        // The Bₚ' mutability frame only consumes the const/mut dimension.
        Some(element) => match elaborate_explicit_p1(
            element_policy(element),
            &entry.callable_view.pair,
            ExplicitP1Position::WrittenSelf,
            provenance.clone(),
        )?
        .and_then(|selection| selection.mode)
        {
            Some(PolicyMode::Const) => MutabilityPattern::Const,
            Some(PolicyMode::Mut) => MutabilityPattern::Mut,
            _ => PolicyMode::Plain,
        },
        None => PolicyMode::Plain,
    };
    let explicit_parameter_patterns = frame
        .explicit_parameters
        .iter()
        .map(|element| {
            formal_mutability_pattern(
                element_policy(element),
                &entry.body_entry_view,
                provenance.clone(),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(MutabilityFormalFrame {
        self_pattern,
        explicit_parameter_patterns,
    })
}

fn apply_written_self_structure(
    candidate: &mut ApplicableCandidate,
    entry: &OrdinaryCallEntry,
    actual: &crate::semantic_world::SemanticValueObject,
    semantic_world: &SemanticWorld,
    resolver_context: &ResolverContext,
    provenance: Provenance,
) -> Result<(), Diagnostic> {
    let Some(head) = entry
        .closure
        .as_ref()
        .and_then(|closure| closure.head.as_ref())
    else {
        return Err(Diagnostic::hard_error(
            "ordinary call entry has no explicit closure head",
            Some(provenance),
        ));
    };
    let Some(written_self) = head.formal_frame().written_self else {
        return Ok(());
    };
    let NormPatternElem::BindingSlot(slot) = written_self else {
        return Err(Diagnostic::hard_error(
            "ordinary written self Pattern is not a binding slot",
            Some(provenance),
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
            return Err(Diagnostic::hard_error(
                "ordinary written self structural Pattern is outside the currently connected Pattern matcher",
                Some(Provenance::from_norm_origin(
                    "ordinary written self Pattern",
                    &slot.origin,
                )),
            ));
        }
    };

    if let Some(annotation) = &slot.annotation {
        let expected = resolve_type_annotation_value(
            &annotation.pattern,
            semantic_world,
            resolver_context,
            provenance.clone(),
        )?;
        if expected != actual.type_value {
            return Err(Diagnostic::hard_error(
                format!(
                    "ordinary written self type applicability failed: expected {:?}, got {:?}",
                    expected, actual.type_value
                ),
                Some(provenance),
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
) -> Result<(), Diagnostic> {
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
                crate::RawArgValueClass::NonValue(crate::NonValueArgKind::TypeObject)
            )
        {
            continue;
        }
        let expected = resolve_type_annotation_value(
            &annotation.pattern,
            semantic_world,
            resolver_context,
            provenance.clone(),
        )?;
        let Some(actual) = actual.known_first_order_type_value else {
            return Err(Diagnostic::hard_error(
                "ordinary value parameter requires an evaluated argument TypeValue",
                Some(provenance),
            ));
        };
        if expected != actual {
            return Err(Diagnostic::hard_error(
                format!(
                    "ordinary value parameter type applicability failed: expected {:?}, got {:?}",
                    expected, actual
                ),
                Some(provenance),
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

fn formal_mutability_pattern(
    policy: Option<&NormPolicySpec>,
    inherited: &PolicyView,
    provenance: Provenance,
) -> Result<MutabilityPattern, Diagnostic> {
    Ok(elaborate_formal_policy_pattern(policy, inherited, provenance)?.mode)
}

fn bp_prime_dominates(
    better: &PreparedCallCandidate,
    worse: &PreparedCallCandidate,
    actual: &MutabilityActualFrame,
    phase: Phase,
    output_demand: OutputModeDemand,
    migration: Option<MigrationInvocationContext<'_>>,
) -> bool {
    let mut strictly_better = false;
    match compare_mutability_frames(
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

    match mutability_preference_rank(better.complete_result_view.mode, output_demand.mode()).cmp(
        &mutability_preference_rank(worse.complete_result_view.mode, output_demand.mode()),
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

fn compare_mutability_frames(
    left: &MutabilityFormalFrame,
    right: &MutabilityFormalFrame,
    actual: &MutabilityActualFrame,
) -> PolicyPartialOrdering {
    if left.explicit_parameter_patterns.len() != actual.explicit_arguments.len()
        || right.explicit_parameter_patterns.len() != actual.explicit_arguments.len()
    {
        return PolicyPartialOrdering::Incomparable;
    }
    let mut left_better = false;
    let mut right_better = false;
    compare_mutability_position(
        left.self_pattern,
        right.self_pattern,
        actual.caller_value,
        &mut left_better,
        &mut right_better,
    );
    for ((left, right), actual) in left
        .explicit_parameter_patterns
        .iter()
        .zip(&right.explicit_parameter_patterns)
        .zip(&actual.explicit_arguments)
    {
        compare_mutability_position(*left, *right, *actual, &mut left_better, &mut right_better);
    }
    ordering_from_advantages(left_better, right_better)
}

fn compare_mutability_position(
    left: MutabilityPattern,
    right: MutabilityPattern,
    actual: PolicyMode,
    left_better: &mut bool,
    right_better: &mut bool,
) {
    match mutability_preference_rank(left, actual).cmp(&mutability_preference_rank(right, actual)) {
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
    canonical_key: Option<&crate::MetaInstanceKey>,
    ambient_struct_owner: Option<SemanticOwnerId>,
    returned: &mut OrdinaryReturnedValue,
) -> Result<Option<(TypeValueId, PatternValueId, Option<SemanticValueId>)>, Diagnostic> {
    match returned {
        OrdinaryReturnedValue::Meta(MetaInvocationValue::ForwardedValue(value)) => {
            let represented = value.type_value;
            let Some(pattern) = semantic_world.type_value(represented).map(|t| t.pattern) else {
                return Ok(None);
            };
            Ok(Some((represented, pattern, None)))
        }
        OrdinaryReturnedValue::Meta(MetaInvocationValue::GeneratedTypeDefinitionValue(value)) => {
            // The generated definition id is normalized body material; the
            // result identity is either the ambient struct root or the
            // canonical meta-type root registered under the selected callable
            // plus its arguments.  Neither branch uses the body id as tau.
            let installed = if let Some(ambient_owner) = ambient_struct_owner {
                if let Some((_existing, binder)) =
                    semantic_world.ambient_struct_collision(ambient_owner, value.type_definition_id)
                {
                    return Err(Diagnostic::hard_error(
                        ambient_struct_collision_message(binder),
                        Some(value.provenance.clone()),
                    ));
                }
                semantic_world.install_ambient_struct_type_value(
                    ambient_owner,
                    value.type_definition_id,
                    value.canonical_pattern_value(),
                    selected.complete_result_view.pair.clone(),
                    value.provenance.clone(),
                )
            } else {
                let canonical_key = canonical_key
                    .expect("generated meta type identity requires a canonical MetaInstance key");
                let Some(placement_parent) =
                    semantic_world.callable_declaration_environment(selected.call_entry_value)
                else {
                    return Ok(None);
                };
                let meta_root = crate::MetaInstanceRoot {
                    meta_callable: canonical_key.callable,
                    placement_parent,
                };
                semantic_world.install_generated_type_value(
                    &meta_root,
                    canonical_key.clone(),
                    value.type_definition_id,
                    value.canonical_pattern_value(),
                    selected.complete_result_view.pair.clone(),
                    value.provenance.clone(),
                )?
            };
            let Some((carrier_value, pattern, canonical_type)) = installed else {
                return Err(Diagnostic::hard_error(
                    "generated type installation could not form its semantic carrier",
                    Some(value.provenance.clone()),
                ));
            };
            value.canonical_type = Some(canonical_type);
            let carrier_place = semantic_world
                .value(carrier_value)
                .map(|carrier| carrier.place)
                .ok_or_else(|| {
                    Diagnostic::hard_error(
                        "generated type installation lost its semantic carrier place",
                        Some(value.provenance.clone()),
                    )
                })?;
            let complete_type =
                semantic_world.observe_complete_type(canonical_type, Some(carrier_place))?;
            let construction_material = value.clone();
            *returned = OrdinaryReturnedValue::CompleteType(ReturnedCompleteType {
                complete_type: complete_type.clone(),
                carrier_value,
                pattern,
                construction_material: Some(construction_material),
            });
            Ok(Some((
                complete_type.lookup_key,
                pattern,
                Some(carrier_value),
            )))
        }
        OrdinaryReturnedValue::Meta(MetaInvocationValue::GeneratedConstructionValue(value)) => {
            let canonical_key = canonical_key
                .expect("generated meta value identity requires a canonical MetaInstance key");
            let Some(placement_parent) =
                semantic_world.callable_declaration_environment(selected.call_entry_value)
            else {
                return Ok(None);
            };
            let meta_root = crate::MetaInstanceRoot {
                meta_callable: canonical_key.callable,
                placement_parent,
            };
            let Some(pattern) = semantic_world.allocate_meta_result_pattern(
                &meta_root,
                canonical_key.clone(),
                value.provenance.clone(),
            ) else {
                return Ok(None);
            };
            Ok(semantic_world
                .symbol_rank()
                .map(|rank| (rank, pattern, None)))
        }
        OrdinaryReturnedValue::ForwardedSemanticValue(value) => Ok(semantic_world
            .value(*value)
            .map(|value| (value.type_value, value.pattern, Some(value.id)))),
        OrdinaryReturnedValue::CompleteType(value) => Ok(Some((
            value.complete_type.lookup_key,
            value.pattern,
            Some(value.carrier_value),
        ))),
    }
}

fn compatibility_meta_material_type(value: &MetaInvocationValue) -> TypeValueId {
    match value {
        MetaInvocationValue::ForwardedValue(value) => value.type_value,
        MetaInvocationValue::GeneratedConstructionValue(value) => {
            TypeValueId(value.construction_instance_id.0)
        }
        MetaInvocationValue::GeneratedTypeDefinitionValue(_) => unreachable!(
            "world-connected struct material must be installed and returned as complete tau"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{result_pair_demand_admits, select_overwrite_target};
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

    /// Placeholder scaffold pin — the overwrite target is the unique member
    /// carrying the written facet, by member resolution rather than
    /// position: the match may sit anywhere in the ledger.  This pins the
    /// scaffold's own conservative behavior, not a final write rule.
    #[test]
    fn unique_facet_match_is_selected_wherever_it_sits() {
        assert_eq!(select_overwrite_target(&[true]), Ok(0));
        assert_eq!(select_overwrite_target(&[false, true, false]), Ok(1));
        assert_eq!(select_overwrite_target(&[false, false, true]), Ok(2));
    }

    /// Placeholder scaffold pin — the placeholder never creates: with no
    /// member of the written facet the selection is a hard error.
    #[test]
    fn zero_facet_matches_is_a_hard_error() {
        for ledger in [&[] as &[bool], &[false], &[false, false]] {
            let message = select_overwrite_target(ledger).unwrap_err();
            assert!(
                message.contains("requires an existing member of the overwritten facet"),
                "zero-target overwrite names the missing-facet rule, got: {message}"
            );
        }
    }

    /// Placeholder scaffold pin — several facet matches make the placeholder
    /// write ambiguous; the scaffold rejects rather than falling back to
    /// declaration order.
    #[test]
    fn several_facet_matches_never_fall_back_to_declaration_order() {
        let message = select_overwrite_target(&[true, false, true]).unwrap_err();
        assert!(
            message.contains("is ambiguous"),
            "multi-target overwrite is reported as ambiguous, got: {message}"
        );
        assert!(
            message.contains("never falls back to declaration order"),
            "the diagnostic states the no-declaration-order rule, got: {message}"
        );
    }
}
