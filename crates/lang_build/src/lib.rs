//! v0.6 namespace graph world model bootstrap.
//!
//! This crate intentionally sits after `lang_syntax`: it consumes parsed and
//! normalized source fragments, but does not add parser or normalizer rules.

pub mod build;
pub mod core;
pub mod discovery;
pub mod fingerprint;
pub mod graph;
pub mod manifest;
pub mod meta;
pub mod model;
pub mod source;
pub mod verify;
pub mod world;

pub use build::{
    BuildCache, BuildCacheStats, BuildResult, BuildSession, BuildWorkspace, CacheStatus,
    DependencyBuildMetadata, ExplicitMountBuildMetadata, PackageBuildArtifact,
    PackageBuildMetadata, PackageBuildSpec, SourceRootMetadata, SourceUnitBuildMetadata,
    StaticDependencySpec, SyntheticSymbolBuildMetadata,
};
pub use discovery::{
    DiscoveredSourceRoot, DiscoveredSourceUnit, SourceDiscoveryConfig, SourceDiscoveryReport,
    SourceRootRequest,
};
pub use fingerprint::{fnv1a64_hex, Fnv1a64};
pub use graph::{
    BuildError, NamespaceGraphCapability, NamespaceGraphSnapshot, NamespaceInstallError,
    ResolveExpectation, ResolverContext,
};
pub use manifest::{BuildManifest, NamespaceMount, SourceRoot};
pub use meta::MetaExpansionResult;
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
pub use source::SourceFragment;
pub use verify::evaluate_source_verifications;
pub use world::CompilationWorld;
