//! v0.6 namespace graph world model bootstrap.
//!
//! This crate intentionally sits after `lang_syntax`: it consumes parsed and
//! normalized source fragments, but does not add parser or normalizer rules.

pub mod build;
pub mod call_target;
pub mod control_flow_meta;
pub mod core;
pub mod discovery;
pub mod extraction_view;
pub mod fingerprint;
pub mod graph;
pub mod identity;
pub mod manifest;
pub mod meta;
pub mod meta_cache;
pub mod meta_candidate;
pub mod meta_invocation;
pub mod meta_key;
pub mod model;
pub mod normalized_call;
pub mod pattern_space;
pub mod product_shape;
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
pub use call_target::{resolve_call_target, ResolvedCallTarget};
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
pub use graph::{
    BuildError, NamespaceGraphCapability, NamespaceGraphSnapshot, NamespaceInstallError,
    ResolveExpectation, ResolverContext,
};
pub use identity::{
    type_value_projection_from_type_symbol, AliasChain, AliasCycleDetectionState,
    AliasQueryDisposition, AliasQueryMode, AliasQueryRequest, AliasQueryResult,
    AliasWritableBoundary, PlaceId, TypeValueBindingPlaceholder, TypeValueId,
};
pub use manifest::{BuildManifest, NamespaceMount, SourceRoot};
pub use meta::{
    bind_meta_invocation_value_result, expand_meta_initializer_via_invocation, MetaExpansionResult,
};
pub use meta_cache::{CachedMetaInstance, MetaInstanceCache};
pub use meta_candidate::{
    prepare_meta_callable_candidate, prepare_meta_callable_candidate_from_input,
    CallableCandidateKind, CandidateBuildIdentityPlaceholder, CandidatePolicyPlanes,
    CandidatePrepDeferredReason, CandidatePrepResult, CandidatePreparationContext,
    CandidatePreparationInput, CanonicalArgAtomKind, CanonicalArgProductShapeMaterial,
    CanonicalMetaInstanceKeySeed, ParameterArgRequirement, ParameterShape,
    PreparedCallableCandidate,
};
pub use meta_invocation::{
    compute_construction_instance_id, compute_type_definition_instance_id, invoke_meta_callable,
    invoke_meta_callable_cached, ConstructionIdentityMaterial, ConstructionInstanceId,
    FieldSignatureMaterial, ForwardedValue, GeneratedConstructionValue, GeneratedFieldDefinition,
    GeneratedTypeDefinitionValue, MetaInvocationInput, MetaInvocationResult, MetaInvocationValue,
    MetaValueTarget, ReturnSlotSemantics, ReturnViewShape, TypeDefinitionIdentityMaterial,
    TypeDefinitionInstanceId,
};
pub use meta_key::{compute_meta_instance_key, CanonicalFingerprint, MetaInstanceKey};
pub use model::{
    callable_body_allows_execution, policy_metadata, policy_set_allows_execution,
    policy_set_export_meta, policy_set_export_meta_runtime, policy_set_meta,
    policy_set_meta_runtime, policy_set_runtime, CallablePolicyMetadata, ChildBucket, ChildLink,
    ChildNameRole, CoreMetaFunction, Diagnostic, DiagnosticSeverity, ExecutionEnv, FieldObject,
    FieldProjection, MetaFunctionObject, NamespaceDelta, NamespaceNode, NamespaceNodeId,
    NamespaceNodeKind, PolicyEnv, PolicyFlag, PolicyMetadata, PolicySet, Provenance, ResolverCode,
    SourceCategory, SymbolId, SymbolKind, SymbolObject, SymbolPayload, SyntaxObject,
    SyntaxObjectKind, TypeField, TypeObject, VerificationPrimitive, VisibilityMetadata,
};
pub use normalized_call::{extract_single_call_site, NormalizedCallSite};
pub use pattern_space::{
    bool_branch_space_for_tests, derive_sum_pattern_space, SelectedSumPattern,
    StructLeafTypeExprShape, SumPatternAlternative, SumPatternPayloadShape, SumPatternSpaceShape,
    SymbolPathShape, TypePatternExprShape,
};
pub use product_shape::{
    ArgProductShape, ExplicitPassMode, FlattenedProductInvariant, FlattenedProductObject,
    NonValueArgKind, ProductAtom, ProductMaterialRole, ProductObject, RawArgShape,
    RawArgValueClass,
};
pub use source::SourceFragment;
pub use struct_decoder::{decode_struct_type_pattern_expr, DecodedStructPattern};
pub use type_argument::{
    classify_type_arguments, classify_type_arguments_with_report, TypeArgumentClassificationReport,
};
pub use verify::evaluate_source_verifications;
pub use world::CompilationWorld;
