//! v0.6 namespace graph world model bootstrap.
//!
//! This crate intentionally sits after `lang_syntax`: it consumes parsed and
//! normalized source fragments, but does not add parser or normalizer rules.

pub mod build;
pub mod call_target;
pub mod core;
pub mod discovery;
pub mod fingerprint;
pub mod graph;
pub mod identity;
pub mod manifest;
pub mod meta;
pub mod meta_candidate;
pub mod model;
pub mod normalized_call;
pub mod product_shape;
pub mod source;
pub mod verify;
pub mod world;

pub use build::{
    BuildCache, BuildCacheStats, BuildResult, BuildSession, BuildWorkspace, CacheStatus,
    DependencyBuildMetadata, ExplicitMountBuildMetadata, PackageBuildArtifact,
    PackageBuildMetadata, PackageBuildSpec, SourceRootMetadata, SourceUnitBuildMetadata,
    StaticDependencySpec, SyntheticSymbolBuildMetadata,
};
pub use call_target::{resolve_call_target, ResolvedCallTarget};
pub use discovery::{
    DiscoveredSourceRoot, DiscoveredSourceUnit, SourceDiscoveryConfig, SourceDiscoveryReport,
    SourceRootRequest,
};
pub use fingerprint::{fnv1a64_hex, Fnv1a64};
pub use graph::{
    BuildError, NamespaceGraphCapability, NamespaceGraphSnapshot, NamespaceInstallError,
    ResolveExpectation, ResolverContext,
};
pub use identity::{
    AliasChain, AliasCycleDetectionState, AliasQueryDisposition, AliasQueryMode, AliasQueryRequest,
    AliasQueryResult, AliasWritableBoundary, PlaceId, TypeValueBindingPlaceholder, TypeValueId,
};
pub use manifest::{BuildManifest, NamespaceMount, SourceRoot};
pub use meta::MetaExpansionResult;
pub use meta_candidate::{
    prepare_meta_callable_candidate, prepare_meta_callable_candidate_from_input,
    CallableCandidateKind, CandidateBuildIdentityPlaceholder, CandidatePolicyPlanes,
    CandidatePrepDeferredReason, CandidatePrepResult, CandidatePreparationContext,
    CandidatePreparationInput, CanonicalArgAtomKind, CanonicalArgProductShapeMaterial,
    CanonicalMetaInstanceKeySeed, ParameterArgRequirement, ParameterShape,
    PreparedCallableCandidate,
};
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
pub use product_shape::{
    ArgProductShape, ExplicitPassMode, FlattenedProductInvariant, FlattenedProductObject,
    NonValueArgKind, ProductAtom, ProductMaterialRole, ProductObject, RawArgShape,
    RawArgValueClass,
};
pub use source::SourceFragment;
pub use verify::evaluate_source_verifications;
pub use world::CompilationWorld;
