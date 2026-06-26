//! v0.6 namespace graph world model bootstrap.
//!
//! This crate intentionally sits after `lang_syntax`: it consumes parsed and
//! normalized source fragments, but does not add parser or normalizer rules.

pub mod core;
pub mod discovery;
pub mod graph;
pub mod manifest;
pub mod meta;
pub mod model;
pub mod source;
pub mod world;

pub use discovery::{
    DiscoveredSourceRoot, DiscoveredSourceUnit, SourceDiscoveryConfig, SourceDiscoveryReport,
    SourceRootRequest,
};
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
    SyntaxObjectKind, TypeField, TypeObject, VisibilityMetadata,
};
pub use source::SourceFragment;
pub use world::CompilationWorld;
