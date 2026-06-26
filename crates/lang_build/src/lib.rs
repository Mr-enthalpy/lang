//! v0.6 namespace graph world model bootstrap.
//!
//! This crate intentionally sits after `lang_syntax`: it consumes parsed and
//! normalized source fragments, but does not add parser or normalizer rules.

pub mod core;
pub mod graph;
pub mod manifest;
pub mod meta;
pub mod model;
pub mod source;
pub mod world;

pub use graph::{
    BuildError, NamespaceGraphCapability, NamespaceGraphSnapshot, NamespaceInstallError,
    ResolveExpectation, ResolverContext,
};
pub use manifest::{BuildManifest, NamespaceMount, SourceRoot};
pub use meta::MetaExpansionResult;
pub use model::{
    ChildBucket, ChildLink, ChildNameRole, CoreMetaFunction, Diagnostic, DiagnosticSeverity,
    FieldObject, FieldProjection, MetaFunctionObject, NamespaceDelta, NamespaceNode,
    NamespaceNodeId, NamespaceNodeKind, PolicyMetadata, Provenance, SourceCategory, SymbolId,
    SymbolKind, SymbolObject, SymbolPayload, SyntaxObject, SyntaxObjectKind, TypeField, TypeObject,
    VisibilityMetadata,
};
pub use source::SourceFragment;
pub use world::CompilationWorld;
