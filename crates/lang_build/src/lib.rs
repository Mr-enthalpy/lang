//! Connected build graph and canonical semantic substrate.
//!
//! This crate intentionally sits after `lang_syntax`: it consumes parsed and
//! normalized source fragments, but does not add parser or normalizer rules.

pub mod build;
pub mod canonical_value;
mod content_observation;
pub mod control_flow_end;
pub mod core;
pub mod discovery;
pub mod fingerprint;
pub mod identity;
pub mod initializer_eval;
pub mod invocation_frame;
pub mod invocation_result;
pub mod lifecycle;
pub mod literal_semantics;
pub mod manifest;
mod meta;
pub mod meta_body;
mod meta_candidate;
mod meta_invocation;
mod meta_key;
pub mod model;
pub mod normalized_call;
pub mod ordinary_invocation;
mod overload_pattern;
mod overload_set;
pub mod owner_namespace;
pub mod pattern_relation;
pub mod phase_flow;
pub mod policy_migration;
pub mod policy_overload;
pub mod policy_pair;
pub mod product_shape;
pub mod return_target;
pub mod semantic_name_index;
pub mod semantic_owner;
pub mod semantic_world;
pub mod source;
mod struct_decoder;
mod struct_pattern_material;
mod struct_pattern_registry;
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
pub use content_observation::{
    observe_content_projection, ContentObservationInterface, NamedObservedField,
    NamedObservedProduct, ObservedArgumentContent, ObservedAtomContent, ObservedAtomKind,
    ObservedContentProjection, ObservedProductContent, ObservedProductElement, ObservedProductKind,
    TypeContentObservation,
};
pub use control_flow_end::{
    compute_control_flow_end_report, ControlFlowEndDiagnostic, ControlFlowEndReport,
    ControlFlowTerminal,
};
pub use discovery::{
    DiscoveredSourceRoot, DiscoveredSourceUnit, SourceDiscoveryConfig, SourceDiscoveryReport,
    SourceRootRequest,
};
pub use fingerprint::{fnv1a64_hex, Fnv1a64};
pub use identity::{
    MetaCallableIdentity, PlaceId, SemanticValueId, TypeLookupIndexAllocator, TypeValueId,
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
    AbstractLiteralValue, BuiltinNumericConstructorSpec, ConstructionFamily, ConstructionRequest,
    NumericFamily, NumericTypeKey, NumericTypeRegistry,
};
pub use manifest::{BuildManifest, NamespaceMount, SourceRoot, ToolchainGlobalSourceRoot};
pub use meta_body::{
    check_closure_body_delete_legality, evaluate_selected_meta_closure_body,
    selected_meta_delete_diagnostic, ClosureBodyExecutionEnv, SelectedMetaBodyEvaluation,
};
pub use meta_candidate::{
    prepare_meta_callable_candidate_with_declared_planes, CallableCandidateKind,
    CandidatePolicyPlanes, CandidatePrepDeferredReason, CandidatePrepResult,
    CandidatePreparationContext, CanonicalArgAtomKind, CanonicalArgProductShapeMaterial,
    ParameterArgRequirement, ParameterShape, PreparedCallableCandidate,
};
pub(crate) use meta_invocation::{
    IdentityTypeMaterial, MetaExecutionMaterial, MetaInvocationInput, MetaPrimitiveExecution,
    ReturnViewShape,
};
pub use meta_invocation::{StructConstructionMaterial, StructConstructionMaterialId};
pub use meta_key::{compute_meta_invocation_material_key, MetaInvocationMaterialKey};
pub use model::{
    policy_view_allows_execution, CallablePolicyViews, ChildBucket, ChildLink, ChildNameRole,
    CoreMetaFunction, CoreTypeProjection, Diagnostic, DiagnosticSeverity, ExecutionEnv,
    FieldObject, FieldProjection, MetaFunctionObject, NamespaceNode, NamespaceNodeId,
    NamespaceNodeKind, PolicyEnv, Provenance, ResolverCode, SemanticNameDelta,
    SourceCallableObject, SourceCategory, SymbolId, SymbolKind, SymbolObject, SymbolPayload,
    SyntaxObject, SyntaxObjectKind, TypeField, VerificationPrimitive, VisibilityMetadata,
};
pub use normalized_call::{extract_single_call_site, NormalizedCallSite};
pub use ordinary_invocation::{
    invoke_host_member_symbol_ordinary, invoke_pattern_associated_ordinary,
    invoke_pattern_associated_value_ordinary, invoke_policy_migration, invoke_symbol_ordinary,
    CallableTarget, ClusterSymbolResult, DynamicLegalityDemand, DynamicLegalityProof,
    ExposedInvocationResult, InvocationOutcome, MigrationInvocationContext,
    OrdinaryCandidateOrigin, OrdinaryInvocationContext, OrdinaryInvocationFailure,
    OrdinaryPipelineTrace, PolicyMigrationResult, PreparedCallCandidate,
    ProjectedInvocationOutcome, ReturnedCompleteType, ReturnedSemanticEntity,
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
// The connected ordinary pipeline owns candidate selection. This surface
// exposes only its shared failure taxonomy and sealed-candidate material.
pub use overload_set::{
    declared_return_shape_from_closure, LookupPhase, RestrictedOverloadFailure,
    RestrictedOverloadFailureKind, SelectedOverloadCandidate, VisibilityView,
};
pub use owner_namespace::{
    ExtractionMemberVisibility, NamespaceLookupFailure, NamespaceLookupResult, NamespaceNameView,
    NamespaceSymbolEntry, OwnerNamespaceGraph, OwnerNamespaceNode, OwnerNamespaceNodeId,
};
pub use pattern_relation::{
    direct_pattern_child_from_canonical_value, solve_parameter_product_relation,
    DirectPatternChildEvidence, ExtractedTypeObservation, NamedPatternObservation,
    PatternApplicabilityProof, PatternLocalBinding, PatternPackBinding, PatternRelationContext,
    PatternRelationDerivation, PatternRelationFailure, PatternSelector, ResolvedPatternBinderId,
    StructuralDefault,
};
pub use phase_flow::{
    classify_static_task, enumerate_value_facet, expose_policy_slice, project_complete_symbol_flow,
    read_pattern, read_value, resolve_explicit_path, CompleteFlowNode, CompleteSymbolFlow,
    ExposedPolicyEntry, FacetView, ProjectedCompileFlow, RuntimeResidualFlow, StaticFlow,
    StaticTaskDisposition, SymbolEntry, SymbolResolutionError,
};
pub use policy_migration::{
    elaborate_pure_type_binding_p1, elaborate_value_binding_p1, P1Elaboration,
    P1ElaborationFailure, P1Origin, PolicyMigrationRequest, PolicyMigrationRequestFailure,
    PolicyPartialOrdering, PureTypeP1Elaboration, SemanticValueRef,
};
pub use policy_overload::{
    select_by_policy_product, select_policy_overload, PhaseOverloadCandidate, PolicyActualFrame,
    PolicyFormalFrame, PolicyOverloadCandidate, PolicyOverloadSelection,
};
pub use policy_pair::{
    body_entry_allows_execution, compute_export_retention_closure, compute_wpre,
    declared_policy_view, derive_function_object_view, elaborate_binding_result_demand,
    elaborate_explicit_p1, elaborate_formal_policy_pattern, elaborate_namespace_declaration_policy,
    elaborate_return_policy_pattern, externally_visible, function_object_declaration_policy,
    normalize_p2_policy, policy_or, project_export_overload_sets, project_export_root_preview,
    project_p1, project_resolved_export_view, publicly_reachable, validate_return_shape,
    BuiltinPrivilegedSealFunction, CallablePrivilege, CapabilityRealization,
    CapabilityRealizationCell, DeclarationVisibility, ExplicitP1Position, ExplicitP1Selection,
    ExportAdmission, ExportCandidateView, FormalPolicyPattern, FunctionMember, FunctionMemberKind,
    FunctionObject, FunctionObjectDeclarationPolicy, FunctionObjectView, FunctionSliceStage,
    NamespaceCandidateSetRef, NamespaceDeclarationPolicy, NamespaceDeclarationPosition,
    NamespaceExportNode, NamespaceOverloadSets, NamespaceResolveAuthority, NamespaceVisibility,
    OutputModeDemand, P1Projection, PatternComponentPolicy, PatternConstraint, Phase, PolicyMode,
    PolicyPair, PolicyResultEntry, PolicyStage, PolicyView, ResolvedCandidatePolicy,
    ResultPolicyDemand, ReturnPolicyPattern, ReturnShape, SealWorldSnapshot, StageSet,
    ValueComponentPolicy, ValuePresence, WpreRoots,
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
    ClusterConstructionId, ClusterConstructionMaterial, CompleteTypeValue, ConstructionAuthority,
    ConstructionEvaluationContext, ConstructionWindow, ImmutableTypeCallSpace,
    InjectedValueIdentity, MemberCreationProof, MetaInstanceRoot, MetaInstanceRootKey, ObjectPlace,
    ObjectPlaceId, OpenClusterConstruction, OpenHereFailure, OpenHereProof, OrdinaryCallEntry,
    OrdinaryCandidateRole, OrdinaryOpenWindow, OwnerStrategy, PatternClusterOwner,
    PatternHostMember, PatternValueId, PlaceMutationFailure, ProjectionSelector, ProjectionSlot,
    ProjectionSlotContents, ProjectionSlotIdentity, PurePMember, RegisteredCallable,
    ResidentGeneration, ResidentIdentity, ResidualRuntimeEpoch, ResolvedExtractionTarget,
    ResolvedPatternScope, ResolvedPatternScopeId, ResolvedSemanticNavigation, SemanticObjectId,
    SemanticPatternValue, SemanticSymbolCell, SemanticTypeValue, SemanticVal2Snapshot,
    SemanticValueObject, SemanticValuePayload, SemanticWorld, StableBorrowTarget, TypeMemberFacet,
    TypeMemberSnapshotEntry, WritableContext,
};
pub use source::SourceFragment;
pub use struct_decoder::{
    decode_struct_associated_val2_let, decode_struct_type_pattern_expr, DecodedStructPattern,
    StructAssociatedVal2Contribution,
};
pub use struct_pattern_material::{
    bool_struct_aliases_for_tests, bool_struct_sum_material_for_tests, derive_struct_sum_material,
    SelectedStructAlternative, StructLeafSyntaxMaterial, StructPatternAlias,
    StructPatternSyntaxMaterial, StructSumAlternative, StructSumPayloadMaterial,
    StructSumSyntaxMaterial, StructSymbolPathMaterial, StructuralMemberVisibility,
};
pub use struct_pattern_registry::{
    nav_component_name, LocalPatternPlaceId, StructFieldPatternMaterial,
    StructMaterializationState, StructPatternLookupExpectation, StructPatternLookupInput,
    StructPatternMaterial, StructPatternMaterialContext, StructPatternMaterialId,
    StructPatternMaterialKind, StructPatternMaterialOrigin, StructPatternMaterialRegistry,
    StructPatternMaterialization,
};
pub use type_argument::{
    classify_type_arguments_env_with_report, BodyLocalInitializerCheck, NamedTypeResolution,
    SemanticTypeEnv, TypeArgumentClassificationReport, TypeResolutionEnv,
};
pub use verify::evaluate_source_verifications;
pub use world::CompilationWorld;
