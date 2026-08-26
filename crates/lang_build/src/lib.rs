//! v0.6 connected SemanticWorld bootstrap.
//!
//! This crate intentionally sits after `lang_syntax`: it consumes parsed and
//! normalized source fragments, but does not add parser or normalizer rules.

pub mod build;
pub mod canonical_value;
pub mod construction_value;
pub mod control_flow_end;
pub mod control_flow_meta;
pub mod core;
pub mod discovery;
pub mod extraction_view;
pub mod fingerprint;
pub mod identity;
pub mod initializer_eval;
pub mod invocation_frame;
pub mod invocation_result;
pub mod lifecycle;
pub mod literal_semantics;
pub mod manifest;
pub mod meta;
pub mod meta_body;
pub mod meta_cache;
pub mod meta_candidate;
pub mod meta_invocation;
pub mod meta_key;
pub mod model;
pub mod normalized_call;
pub mod ordinary_invocation;
pub mod overload_pattern;
pub mod overload_set;
pub mod owner_namespace;
pub mod pattern_head;
pub mod pattern_relation;
pub mod pattern_space;
pub mod phase_flow;
pub mod policy_expr;
pub mod policy_overload;
pub mod policy_pair;
pub mod policy_transition;
pub mod product_shape;
pub mod return_target;
pub mod semantic_name_index;
pub mod semantic_owner;
pub mod semantic_world;
pub mod source;
pub mod struct_decoder;
pub mod type_argument;
pub mod verify;
pub mod world;

pub use build::{
    BuildCache, BuildCacheStats, BuildResult, BuildSession, BuildWorkspace, CacheStatus,
    DependencyBuildMetadata, ExplicitMountBuildMetadata, PackageBuildArtifact,
    PackageBuildMetadata, PackageBuildSpec, SourceRootMetadata, SourceUnitBuildMetadata,
    StaticDependencySpec, SyntheticSymbolBuildMetadata,
};
pub use canonical_value::{
    canonical_literal_content, canonical_literal_norm, expand_extraction_navigation,
    CanonicalClusterNorm, CanonicalCompleteTypeNorm, CanonicalFullNavigation,
    CanonicalLiteralFamily, CanonicalNormForm, CanonicalObjectNorm, CanonicalOrderedPatternEntry,
    CanonicalPatternAtom, CanonicalPatternBuilder, CanonicalPatternNorm, CanonicalPatternValue,
    CanonicalProductConstructor, CanonicalTypeCallSpaceNorm, CanonicalTypeObservation,
    CanonicalVal1Norm, CanonicalVal2Norm, CanonicalValueAddr, DuplicatePatternNavigation,
    ExtractionPatternParent, MissingExtractionNavigationAnchor, OpaqueVal1Id, PatternChildInput,
    PatternLayerContext, PatternNavigationInput, PatternOwnNavigation,
};
pub use construction_value::{
    construct_field_value, construct_owner_value, constructed_question_view, leaf_value,
    placeholder_field_constructor_head, placeholder_owner_constructor_head, question_view_peels,
    ConstructedValue, ConstructorHead,
};
pub use control_flow_end::{
    compute_control_flow_end_report, ControlFlowEndDiagnostic, ControlFlowEndReport,
    ControlFlowTerminal,
};
pub use control_flow_meta::{
    check_simple_policy, check_simple_type_predicate, evaluate_guarded_branches,
    lookup_branch_local_symbol, select_branch_arm, validate_branch_arm_labels, BranchActionShape,
    BranchArmShape, BranchLocalBinding, BranchLocalLookupResult, BranchLocalSymbol,
    BranchLocalSymbolSpace, BranchSelectionResult, BranchTypeRequirement,
    ControlFlowLocalEvalResult, ControlFlowLocalMetaContext, EvaluatedBranchAction,
    GuardResidualReason, MetaInvocationPlanShape, SimpleCapability, SimplePolicyCheckResult,
    SimplePolicyFacts, SimplePolicyRequirement, SimpleTypeCheckResult, SimpleTypeFacts,
    SimpleTypePredicate, SimpleTypePredicateFact,
};
pub use discovery::{
    DiscoveredSourceRoot, DiscoveredSourceUnit, SourceDiscoveryConfig, SourceDiscoveryReport,
    SourceRootRequest,
};
pub use extraction_view::{
    match_binding_pattern_shape, question_view, BindingPatternShape, BindingShapeMatchResult,
    EvalResultNormalForm, ExposedExtractionInterface, ExtractionViewResult, NamedExtractionField,
    NamedProductExtractionShape, ProductNormalFormElem, ProductNormalFormKind,
    ProductNormalFormShape, TypeExtractionInterface, ValuePointKind, ValuePointShape,
};
pub use fingerprint::{fnv1a64_hex, Fnv1a64};
pub use identity::{
    MetaCallableIdentity, PlaceId, SemanticValueId, TypeValueBindingPlaceholder, TypeValueId,
};
pub use initializer_eval::{
    binding_assertion_annotation_context, residual_diagnostic, AnnotationContext, EvalMode,
    ResidualReason,
};
pub use invocation_frame::{
    CallableFrameShape, ExplicitParameterShape, InvocationCallableRef, InvocationExecutionEnv,
    InvocationFrame, InvocationLookupEnv, ReceiverTypeRef, ReturnTargetShape, SelfPosition,
    SelfPositionSource, SelfSlotKind, SelfSlotShape, SELF_SLOT_INDEX,
};
pub use invocation_result::{DeclaredResultClass, InvocationResidual, InvocationResult};
pub use lifecycle::{
    AccessPath, AccessRelationProvider, AccessSnapshot, CleanupPlacement, ColorAlgebra, ColorId,
    LifeName, LifecycleAction, LifecycleEvent, LifecycleEventKind, LifecycleFailure,
    LifecycleMachine, LifecyclePost, LifecyclePrecondition, LifecycleSnapshot,
    LifecycleValidationContext, LifecycleValidationProof, LifetimeValue, NameView, Region,
    SemanticContinuation, SemanticPosition,
};
pub use literal_semantics::{
    abstract_character_value, compile_literal_policy, form_abstract_literal_value,
    AbstractLiteralExactValue, AbstractLiteralFamily, AbstractLiteralFormationFailure,
    AbstractLiteralValue, AtomicBuiltinType, AtomicBuiltinTypeRegistry,
    AtomicBuiltinTypeRegistryFailure, BuiltinNumericConstructorSpec, ConstructionFamily,
    ConstructionRequest, NumericFamily, NumericTypeKey, NumericTypeRegistry,
};
pub use manifest::{BuildManifest, NamespaceMount, SourceRoot, ToolchainGlobalSourceRoot};
pub use meta::{
    bind_meta_invocation_value_result,
    bind_meta_invocation_value_result_with_materialization_state, MetaExpansionResult,
};
pub use meta_body::{
    check_closure_body_delete_legality, evaluate_selected_meta_closure_body,
    selected_meta_delete_diagnostic, ClosureBodyExecutionEnv, SelectedMetaBodyEvaluation,
};
pub use meta_cache::{CachedMetaInstance, MetaInstanceCache};
pub use meta_candidate::{
    prepare_meta_callable_candidate_with_declared_planes, CallableCandidateKind,
    CandidateBuildIdentityPlaceholder, CandidatePolicyPlanes, CandidatePrepDeferredReason,
    CandidatePrepResult, CandidatePreparationContext, CanonicalArgAtomKind,
    CanonicalArgProductShapeMaterial, ParameterArgRequirement, ParameterShape,
    PreparedCallableCandidate,
};
pub use meta_invocation::{
    attach_type_definition_pattern_heads, attach_type_definition_pattern_heads_with_context,
    compute_construction_instance_id, compute_type_definition_instance_id, invoke_meta_callable,
    invoke_meta_callable_cached, invoke_meta_callable_cached_with_materialization_state,
    invoke_meta_callable_with_materialization_state, ConstructionIdentityMaterial,
    ConstructionInstanceId, FieldSignatureMaterial, ForwardedValue, GeneratedConstructionValue,
    GeneratedFieldDefinition, GeneratedFieldPatternHead, GeneratedTypeDefinitionValue,
    MetaInvocationInput, MetaInvocationValue, ReturnSlotSemantics, ReturnViewShape,
    TypeDefinitionIdentityMaterial, TypeDefinitionInstanceId, TypeDefinitionPatternHeads,
};
pub use meta_key::{
    compute_canonical_meta_instance_key, compute_legacy_meta_instance_digest, CanonicalFingerprint,
    MetaInstanceKey,
};
pub use model::{
    callable_body_allows_execution, policy_metadata, policy_set_allows_execution,
    policy_set_compile, policy_set_export_meta, policy_set_export_meta_runtime, policy_set_meta,
    policy_set_meta_runtime, policy_set_runtime, policy_set_seal, CallablePolicyMetadata,
    ChildBucket, ChildLink, ChildNameRole, CoreMetaFunction, Diagnostic, DiagnosticSeverity,
    ExecutionEnv, FieldObject, FieldProjection, MetaFunctionObject, NamespaceNode, NamespaceNodeId,
    NamespaceNodeKind, PolicyEnv, PolicyFlag, PolicyMetadata, PolicySet, Provenance, ResolverCode,
    SemanticNameDelta, SourceCallableObject, SourceCategory, SymbolId, SymbolKind, SymbolObject,
    SymbolPayload, SyntaxObject, SyntaxObjectKind, TypeField, TypeObject, VerificationPrimitive,
    VisibilityMetadata,
};
pub use normalized_call::{extract_single_call_site, NormalizedCallSite};
pub use ordinary_invocation::{
    invoke_atomic_runtime_migration, invoke_host_member_symbol_ordinary,
    invoke_pattern_associated_ordinary, invoke_pattern_associated_value_ordinary,
    invoke_policy_migration, invoke_symbol_ordinary, AtomicRuntimeMigrationResult, CallableTarget,
    ClusterSymbolResult, DynamicLegalityDemand, DynamicLegalityProof, ExposedInvocationResult,
    InvocationOutcome, MigrationInvocationContext, OrdinaryCandidateOrigin,
    OrdinaryInvocationContext, OrdinaryInvocationFailure, OrdinaryPipelineTrace,
    OrdinaryReturnedValue, PolicyMigrationResult, PreparedCallCandidate, ReturnedCompleteType,
    SealedSelectedInvocation, SingleMemberResult, UnitInvocationResult,
};
pub use overload_pattern::{
    overload_args_from_classified_shape, pack_operand_is_admissible, OverloadArgShape,
    PackOperandClass, PatternLayerOrder, SpecificityTuple,
};
pub use semantic_name_index::{
    BuildError, ResolveExpectation, ResolverContext, SemanticNameIndex, SemanticNameInstallError,
    SemanticNameResolver,
};
// The legacy restricted selector/evaluator surface is
// deleted.  The canonical overload ontology is the connected ordinary
// pipeline exported from `ordinary_invocation` above; what remains here is
// the shared failure taxonomy and the selected-candidate carrier consumed
// by the shared body evaluators.
pub use overload_set::{
    declared_return_shape_from_closure, LookupPhase, RestrictedOverloadFailure,
    RestrictedOverloadFailureKind, SelectedOverloadCandidate, VisibilityView,
};
pub use owner_namespace::{
    ExtractionMemberVisibility, NamespaceLookupFailure, NamespaceLookupResult, NamespaceNameView,
    NamespaceSymbolEntry, OwnerNamespaceGraph, OwnerNamespaceNode, OwnerNamespaceNodeId,
};
pub use pattern_head::{
    nav_component_name, LocalPatternPlaceId, PatternExpectation, PatternFieldMaterialization,
    PatternHead, PatternHeadId, PatternHeadKind, PatternHeadMaterialization, PatternHeadOrigin,
    PatternHeadRegistry, PatternLookupInput, PatternMaterializationContext,
    TypeMaterializationState,
};
pub use pattern_relation::{
    direct_pattern_child_from_canonical_value, solve_parameter_product_relation,
    DirectPatternChildEvidence, ExtractedTypeObservation, NamedPatternObservation,
    PatternApplicabilityProof, PatternLocalBinding, PatternPackBinding, PatternRelationContext,
    PatternRelationDerivation, PatternRelationFailure, PatternSelector, ResolvedPatternBinderId,
    StructuralDefault,
};
pub use pattern_space::{
    bool_branch_space_for_tests, bool_pattern_aliases_for_tests, derive_sum_pattern_space,
    PatternSymbolAlias, SelectedSumPattern, StructLeafTypeExprShape, StructuralMemberVisibility,
    SumPatternAlternative, SumPatternPayloadShape, SumPatternSpaceShape, SymbolPathShape,
    TypePatternExprShape,
};
pub use phase_flow::{
    classify_static_task, enumerate_value_facet, expose_policy_slice, project_complete_symbol_flow,
    read_pattern, read_value, resolve_explicit_path, CompleteFlowNode, CompleteSymbolFlow,
    ExposedPolicyEntry, FacetView, ProjectedCompileFlow, RuntimeResidualFlow, StaticFlow,
    StaticTaskDisposition, SymbolEntry, SymbolResolutionError,
};
pub use policy_expr::elaborate_declaration_policy_expr;
pub use policy_overload::{
    select_by_mutability_product, select_policy_overload, MutabilityActualFrame,
    MutabilityFormalFrame, MutabilityPattern, PhaseOverloadCandidate, PolicyOverloadCandidate,
    PolicyOverloadSelection,
};
pub use policy_pair::{
    body_entry_allows_execution, compute_export_retention_closure, compute_wpre,
    derive_function_object_view, elaborate_binding_result_demand, elaborate_explicit_p1,
    elaborate_formal_policy_pattern, elaborate_namespace_declaration_policy, externally_visible,
    function_object_declaration_policy, normalize_p2_policy, policy_or,
    project_export_overload_sets, project_export_root_preview, project_p1,
    project_resolved_export_view, publicly_reachable, validate_return_shape,
    BuiltinPrivilegedSealFunction, CallablePrivilege, CapabilityRealization,
    CapabilityRealizationCell, DeclarationVisibility, ExplicitP1Position, ExplicitP1Selection,
    ExportAdmission, ExportCandidateView, FormalPolicyPattern, FunctionMember, FunctionMemberKind,
    FunctionObject, FunctionObjectDeclarationPolicy, FunctionObjectView, FunctionSliceStage,
    NamespaceCandidateSetRef, NamespaceDeclarationPolicy, NamespaceDeclarationPosition,
    NamespaceExportNode, NamespaceOverloadSets, NamespaceResolveAuthority, NamespaceVisibility,
    OutputModeDemand, P1Projection, PatternComponentPolicy, PatternConstraint, Phase, PolicyMode,
    PolicyPair, PolicyResultEntry, PolicyStage, PolicyView, ResolvedCandidatePolicy,
    ResultPolicyDemand, ReturnShape, SealWorldSnapshot, StageSet, ValueComponentPolicy,
    ValuePresence, WpreRoots,
};
pub use policy_transition::{
    assemble_transition_results, compare_policy_transition_candidates,
    elaborate_pure_type_binding_p1, elaborate_value_binding_p1, invoke_resolved_policy_bridge,
    project_transition_policy_domain, qualify_policy_bridge, resolve_policy_bridge,
    validate_runtime_transition, BridgeQualification, OrdinaryCallableTypeInput,
    OrdinaryCallableTypeOutput, P1AssemblyFailure, P1Elaboration, P1ElaborationFailure, P1Origin,
    PolicyBridgeBody, PolicyBridgeEffect, PolicyBridgeInvocationFailure,
    PolicyBridgeInvocationResult, PolicyBridgeResolution, PolicyMigrationRequest,
    PolicyMigrationRequestFailure, PolicyPartialOrdering, PolicyTransitionCallable,
    PolicyTransitionDemand, PolicyTransitionFailure, PolicyTransitionRequest,
    PolicyTransitionRequestFailure, PrototypeTransitionResultCarrier, PureTypeP1Elaboration,
    ResolvedPolicyBridge, SemanticValueRef, TransitionTypeExpectation,
};
pub use product_shape::{
    ArgProductShape, ExplicitPassMode, FlattenedProductInvariant, FlattenedProductObject,
    NonValueArgKind, ProductAtom, ProductMaterialRole, ProductObject, RawArgShape,
    RawArgValueClass,
};
pub use return_target::{
    elaborate_return_targets_in_program, elaborate_return_targets_in_returnable_closure,
    elaborate_return_targets_in_returnable_closure_with_resolver, BoundReturnEvent,
    ExplicitReturnTargetResolution, ExplicitReturnTargetResolver, PreservedReturnReason,
    PreservedUnboundReturnEvent, ResolvedReturnTarget, ReturnFrameId, ReturnFrameOwner,
    ReturnSelfIdentity, ReturnSlotIdentity, ReturnSlotRef, ReturnTargetBinder,
    ReturnTargetBindingReport, ReturnTargetFrame, ReturnTargetStack, UnboundReturnEvent,
    UnresolvedReturnTargetForm,
};
pub use semantic_owner::{
    AnonymousCallableTypeId, CallableOwnerPlacement, CallableReceiverBinding,
    CallableReceiverBindingSource, CallableReceiverTypeId, LocalCallableIdentity,
    LocalGenerationIdentity, LocalSymbolIdentity, OwnerQualificationError, PackageId,
    ResolvedHoleBinderId, ResolvedPatternRootId, SemanticOwnerGraph, SemanticOwnerGraphId,
    SemanticOwnerId, SemanticOwnerKind, SemanticOwnerNode, SemanticOwnerQualification,
    SemanticSymbolIdentity,
};
pub use semantic_world::{
    canonical_function_object_view, derived_cluster_policy, AmbientTypeBinder, BindConflict,
    BorrowFormationFailure, BorrowKind, BorrowOperand, BorrowView, BorrowViewId,
    ClusterConstructionId, CompleteTypeValue, ConstructionAuthority, ConstructionEvaluationContext,
    ConstructionState, ConstructionWindow, ImmutableTypeCallSpace, InjectedValueIdentity,
    MemberCreationProof, MetaInstanceRoot, MetaTypeKey, ObjectPlace, ObjectPlaceId,
    OpenClusterConstruction, OpenHereFailure, OpenHereProof, OrdinaryCallEntry,
    OrdinaryCandidateRole, OrdinaryOpenWindow, OwnerStrategy, PatternClusterOwner,
    PatternHostMember, PatternValueId, PlaceMutationFailure, ProjectionSelector, ProjectionSlot,
    ProjectionSlotContents, ProjectionSlotIdentity, PurePMember, RegisteredCallable,
    ResidentGeneration, ResidentIdentity, ResidualRuntimeEpoch, ResolvedExtractionTarget,
    ResolvedPatternScope, ResolvedPatternScopeId, ResolvedSemanticNavigation, SemanticPatternValue,
    SemanticSymbolCell, SemanticTypeValue, SemanticVal2Snapshot, SemanticValueObject,
    SemanticValuePayload, SemanticWorld, StableBorrowTarget, SymbolConstructionValue,
    TypeMemberFacet, TypeMemberSnapshotEntry, WritableContext,
};
pub use source::SourceFragment;
pub use struct_decoder::{
    decode_struct_associated_val2_let, decode_struct_type_pattern_expr, DecodedStructPattern,
    StructAssociatedVal2Contribution,
};
pub use type_argument::{
    classify_type_arguments, classify_type_arguments_env_with_report,
    classify_type_arguments_with_report, BodyLocalInitializerCheck, NamedTypeResolution,
    SemanticTypeEnv, TypeArgumentClassificationReport, TypeResolutionEnv,
};
pub use verify::evaluate_source_verifications;
pub use world::CompilationWorld;
