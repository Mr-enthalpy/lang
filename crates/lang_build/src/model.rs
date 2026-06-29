use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    path::PathBuf,
};

use lang_syntax::{NormOrigin, NormProduct, Span};

use crate::extraction_view::TypeExtractionInterface;

/// Stable identity for a namespace node inside one graph snapshot.
///
/// v0.6 IDs are snapshot-local numeric identities. Cross-build stable IDs are
/// intentionally deferred until cache/fingerprint design exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NamespaceNodeId(pub u64);

impl NamespaceNodeId {
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl fmt::Display for NamespaceNodeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "node#{}", self.0)
    }
}

/// Stable identity for a symbol object inside one graph snapshot.
///
/// Symbols with the same display name in different namespaces must still have
/// different `SymbolId`s.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolId(pub u64);

impl SymbolId {
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl fmt::Display for SymbolId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "symbol#{}", self.0)
    }
}

/// The structural source of a namespace node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NamespaceNodeKind {
    Physical,
    Declared,
    Virtual,
}

/// The category of contribution that introduced a symbol or node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceCategory {
    PhysicalDirectory,
    DeclaredSymbol,
    TypeAssociatedNamespace,
    MetaInstantiationVirtualLayer,
    GeneratedChild,
    Alias,
    CoreBootstrap,
    DependencyMount,
}

/// Coarse symbol category used before full semantic analysis exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolKind {
    Namespace,
    Type,
    MetaFunction,
    FieldFunction,
    Alias,
    Placeholder,
}

/// Role used when a textual child name is installed under a namespace node.
///
/// Object/function symbols and pure namespace subspaces may intentionally share
/// the same textual name. Resolver callers must provide an expectation when
/// that terminal name is ambiguous.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChildNameRole {
    Object,
    NamespaceSubspace,
}

/// Role-aware child slot for a single textual name.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChildBucket {
    pub object: Option<SymbolId>,
    pub namespace_subspace: Option<SymbolId>,
}

impl ChildBucket {
    pub fn get(&self, role: ChildNameRole) -> Option<SymbolId> {
        match role {
            ChildNameRole::Object => self.object,
            ChildNameRole::NamespaceSubspace => self.namespace_subspace,
        }
    }

    pub fn set(&mut self, role: ChildNameRole, symbol: SymbolId) {
        match role {
            ChildNameRole::Object => self.object = Some(symbol),
            ChildNameRole::NamespaceSubspace => self.namespace_subspace = Some(symbol),
        }
    }
}

/// Early policy flag for v0.7 policy-aware resolution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PolicyFlag {
    Export,
    Meta,
    Runtime,
}

/// Set of policy flags carried by a symbol.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PolicySet {
    pub flags: BTreeSet<PolicyFlag>,
}

impl PolicySet {
    pub fn new() -> Self {
        Self {
            flags: BTreeSet::new(),
        }
    }

    pub fn contains(&self, flag: PolicyFlag) -> bool {
        self.flags.contains(&flag)
    }

    pub fn insert(&mut self, flag: PolicyFlag) {
        self.flags.insert(flag);
    }
}

/// Resolver lookup visibility environment.
///
/// This controls whether a symbol is visible to a resolver query. It does not
/// grant permission to enter or evaluate a callable body.
///
/// Only `Meta` lookup visibility is needed for v0.7-prep. Additional resolver
/// environments will be added as the policy lattice and lookup passes expand.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyEnv {
    Meta,
}

/// Callable body execution environment.
///
/// This is distinct from [`PolicyEnv`]: a resolver may see a callable symbol
/// whose body cannot be entered in the current execution environment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionEnv {
    Meta,
    Runtime,
}

// ---------------------------------------------------------------------------
// Policy-set helper constructors
// ---------------------------------------------------------------------------

pub fn policy_set_export_meta() -> PolicySet {
    let mut set = PolicySet::new();
    set.insert(PolicyFlag::Export);
    set.insert(PolicyFlag::Meta);
    set
}

pub fn policy_set_export_meta_runtime() -> PolicySet {
    let mut set = PolicySet::new();
    set.insert(PolicyFlag::Export);
    set.insert(PolicyFlag::Meta);
    set.insert(PolicyFlag::Runtime);
    set
}

pub fn policy_set_runtime() -> PolicySet {
    let mut set = PolicySet::new();
    set.insert(PolicyFlag::Runtime);
    set
}

pub fn policy_set_meta() -> PolicySet {
    let mut set = PolicySet::new();
    set.insert(PolicyFlag::Meta);
    set
}

pub fn policy_set_meta_runtime() -> PolicySet {
    let mut set = PolicySet::new();
    set.insert(PolicyFlag::Meta);
    set.insert(PolicyFlag::Runtime);
    set
}

pub fn policy_metadata(policy_set: PolicySet) -> PolicyMetadata {
    PolicyMetadata {
        slots: BTreeMap::new(),
        policy_set,
    }
}

pub fn callable_body_allows_execution(callable_policy: &PolicyMetadata, env: ExecutionEnv) -> bool {
    policy_set_allows_execution(&callable_policy.policy_set, env)
}

pub fn policy_set_allows_execution(policy_set: &PolicySet, env: ExecutionEnv) -> bool {
    match env {
        ExecutionEnv::Meta => policy_set.contains(PolicyFlag::Meta),
        ExecutionEnv::Runtime => policy_set.contains(PolicyFlag::Runtime),
    }
}

/// Reserved policy metadata slot.
///
/// v0.6 preserves this data but does not interpret it.
/// v0.7 adds `policy_set` for early policy-aware resolution.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PolicyMetadata {
    pub slots: BTreeMap<String, String>,
    pub policy_set: PolicySet,
}

/// Reserved visibility metadata slot.
///
/// v0.6 preserves this data but does not enforce visibility.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VisibilityMetadata {
    pub slots: BTreeMap<String, String>,
}

/// Human-readable origin information for diagnostics and future IDE/cache use.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Provenance {
    pub description: String,
    pub file: Option<PathBuf>,
    pub span: Option<Span>,
}

impl Provenance {
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            file: None,
            span: None,
        }
    }

    pub fn file(description: impl Into<String>, file: impl Into<PathBuf>) -> Self {
        Self {
            description: description.into(),
            file: Some(file.into()),
            span: None,
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn from_norm_origin(description: impl Into<String>, origin: &NormOrigin) -> Self {
        let span = match origin {
            NormOrigin::Source(span)
            | NormOrigin::Generated { span, .. }
            | NormOrigin::Derived { span, .. } => *span,
        };
        Self::new(description).with_span(span)
    }
}

/// Diagnostic severity used by build/graph/meta phases.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
    HardError,
}

/// Resolver diagnostic discriminator for callers that need to distinguish
/// genuine miss from ambiguity/conflict.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolverCode {
    /// Symbol genuinely not found, or filtered out by policy.
    Unresolved,
    /// Role ambiguity within a single namespace node.
    Ambiguous,
    /// Cross-root conflict — same symbol found in multiple search roots.
    Conflict,
}

/// Build/namespace diagnostic with optional provenance and graph context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub severity: DiagnosticSeverity,
    pub provenance: Option<Provenance>,
    pub symbol_context: Option<SymbolId>,
    pub node_context: Option<NamespaceNodeId>,
    pub code: Option<ResolverCode>,
}

impl Diagnostic {
    pub fn new(
        severity: DiagnosticSeverity,
        message: impl Into<String>,
        provenance: Option<Provenance>,
    ) -> Self {
        Self {
            message: message.into(),
            severity,
            provenance,
            symbol_context: None,
            node_context: None,
            code: None,
        }
    }

    pub fn hard_error(message: impl Into<String>, provenance: Option<Provenance>) -> Self {
        Self::new(DiagnosticSeverity::HardError, message, provenance)
    }

    pub fn error(message: impl Into<String>, provenance: Option<Provenance>) -> Self {
        Self::new(DiagnosticSeverity::Error, message, provenance)
    }

    pub fn with_node_context(mut self, node: NamespaceNodeId) -> Self {
        self.node_context = Some(node);
        self
    }

    pub fn with_symbol_context(mut self, symbol: SymbolId) -> Self {
        self.symbol_context = Some(symbol);
        self
    }

    pub fn with_code(mut self, code: ResolverCode) -> Self {
        self.code = Some(code);
        self
    }
}

/// Namespace graph node that owns child symbol links.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamespaceNode {
    pub id: NamespaceNodeId,
    pub name: String,
    pub kind: NamespaceNodeKind,
    pub source_category: SourceCategory,
    pub parent: Option<NamespaceNodeId>,
    pub children: BTreeMap<String, ChildBucket>,
    pub policy_metadata: PolicyMetadata,
    pub visibility_metadata: VisibilityMetadata,
    pub provenance: Provenance,
    pub diagnostics: Vec<Diagnostic>,
}

impl NamespaceNode {
    pub fn new(
        id: NamespaceNodeId,
        name: impl Into<String>,
        kind: NamespaceNodeKind,
        source_category: SourceCategory,
        parent: Option<NamespaceNodeId>,
        provenance: Provenance,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            kind,
            source_category,
            parent,
            children: BTreeMap::new(),
            policy_metadata: PolicyMetadata::default(),
            visibility_metadata: VisibilityMetadata::default(),
            provenance,
            diagnostics: Vec::new(),
        }
    }
}

/// Canonical graph object returned by the resolver.
///
/// Future compiler phases should consume `SymbolObject`s rather than reparsing
/// path strings or building side tables.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolObject {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub source_category: SourceCategory,
    pub node_kind: Option<NamespaceNodeKind>,
    pub parent: Option<NamespaceNodeId>,
    pub policy_metadata: PolicyMetadata,
    pub visibility_metadata: VisibilityMetadata,
    pub provenance: Provenance,
    pub diagnostics: Vec<Diagnostic>,
    pub generation_origin: Option<String>,
    pub cache_key_fragment: Option<String>,
    pub payload: SymbolPayload,
}

impl SymbolObject {
    pub fn placeholder(
        id: SymbolId,
        name: impl Into<String>,
        kind: SymbolKind,
        source_category: SourceCategory,
        parent: Option<NamespaceNodeId>,
        provenance: Provenance,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            kind,
            source_category,
            node_kind: None,
            parent,
            policy_metadata: PolicyMetadata::default(),
            visibility_metadata: VisibilityMetadata::default(),
            provenance,
            diagnostics: Vec::new(),
            generation_origin: None,
            cache_key_fragment: None,
            payload: SymbolPayload::Placeholder,
        }
    }

    pub fn namespace(
        id: SymbolId,
        name: impl Into<String>,
        node: NamespaceNodeId,
        node_kind: NamespaceNodeKind,
        source_category: SourceCategory,
        parent: Option<NamespaceNodeId>,
        provenance: Provenance,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            kind: SymbolKind::Namespace,
            source_category,
            node_kind: Some(node_kind),
            parent,
            policy_metadata: PolicyMetadata::default(),
            visibility_metadata: VisibilityMetadata::default(),
            provenance,
            diagnostics: Vec::new(),
            generation_origin: None,
            cache_key_fragment: None,
            payload: SymbolPayload::Namespace { node },
        }
    }

    pub fn namespace_node(&self) -> Option<NamespaceNodeId> {
        match &self.payload {
            SymbolPayload::Namespace { node } | SymbolPayload::VerificationNamespace { node } => {
                Some(*node)
            }
            SymbolPayload::Type(type_object) => type_object.type_associated_namespace,
            _ => None,
        }
    }

    pub fn child_name_role(&self) -> ChildNameRole {
        match self.kind {
            SymbolKind::Namespace => ChildNameRole::NamespaceSubspace,
            SymbolKind::Type
            | SymbolKind::MetaFunction
            | SymbolKind::FieldFunction
            | SymbolKind::Alias
            | SymbolKind::Placeholder => ChildNameRole::Object,
        }
    }

    pub fn diagnostic_label(&self) -> String {
        let provenance = self.provenance.description.as_str();
        format!(
            "{} `{}` ({:?}, {:?}, provenance={provenance})",
            self.id, self.name, self.kind, self.source_category
        )
    }
}

/// Optional payload carried by a `SymbolObject`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolPayload {
    Namespace { node: NamespaceNodeId },
    VerificationNamespace { node: NamespaceNodeId },
    Type(TypeObject),
    MetaFunction(MetaFunctionObject),
    FieldFunction(FieldObject),
    Alias { target: SymbolId },
    Placeholder,
}

/// Placeholder type payload created by the v0.6 struct meta slice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeObject {
    pub type_symbol_id: SymbolId,
    pub fields: Vec<TypeField>,
    pub field_names: Vec<String>,
    pub field_type_symbol_ids: Vec<SymbolId>,
    pub type_associated_namespace: Option<NamespaceNodeId>,
    pub extraction_interface: Option<TypeExtractionInterface>,
    pub provenance: Provenance,
    pub generation_origin: Option<String>,
    pub layout_slot: Option<String>,
    pub abi_slot: Option<String>,
}

/// Field entry recorded in a placeholder `TypeObject`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeField {
    pub name: String,
    pub type_symbol_id: SymbolId,
    pub provenance: Provenance,
}

/// Policy metadata carried by callable payloads.
///
/// `body_entry_policy` controls whether a callable body may be entered in an
/// execution environment. `return_object_policy` records the policy of the
/// object produced by the callable. Neither field controls resolver visibility;
/// that remains `SymbolObject.policy_metadata`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallablePolicyMetadata {
    pub body_entry_policy: PolicyMetadata,
    pub return_object_policy: PolicyMetadata,
}

/// Placeholder field-function payload generated under a type namespace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldObject {
    pub owner_type_symbol_id: SymbolId,
    pub field_name: String,
    pub field_type_symbol_id: SymbolId,
    pub projection: FieldProjection,
    pub callable_policy: CallablePolicyMetadata,
    pub provenance: Provenance,
}

/// Projection flavor for generated field functions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldProjection {
    Value,
    Ref,
    Share,
}

/// Core meta-function payload resolved through the namespace graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetaFunctionObject {
    pub function_symbol_id: SymbolId,
    pub primitive: CoreMetaFunction,
    pub function_policy: PolicyMetadata,
    pub body_entry_policy: PolicyMetadata,
    pub return_object_policy: PolicyMetadata,
}

/// Compiler-seeded core meta-function implementations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoreMetaFunction {
    Struct,
    Assert,
    Verify(VerificationPrimitive),
    IdentityType,
    UnaryConstructionPrototype,
}

/// Core source-verification primitive resolved through the namespace graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerificationPrimitive {
    Exists,
    NotExists,
    ResolvesAs,
    NotResolves,
    Kind,
    NamespaceKind,
    FieldNames,
    HasField,
    FieldProjection,
    FieldOwner,
    FieldType,
    Policy,
    NotPolicy,
    BodyEntryPolicy,
    NotBodyEntryPolicy,
    ReturnPolicy,
    NotReturnPolicy,
}

/// Transactional graph mutation.
///
/// A delta is installed atomically: either all links/nodes/symbols are applied
/// or none are.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamespaceDelta {
    pub base_snapshot_id: u64,
    pub nodes: BTreeMap<NamespaceNodeId, NamespaceNode>,
    pub symbols: BTreeMap<SymbolId, SymbolObject>,
    pub child_links: Vec<ChildLink>,
    pub diagnostics: Vec<Diagnostic>,
    next_node_id: u64,
    next_symbol_id: u64,
}

impl NamespaceDelta {
    pub fn new(base_snapshot_id: u64, next_node_id: u64, next_symbol_id: u64) -> Self {
        Self {
            base_snapshot_id,
            nodes: BTreeMap::new(),
            symbols: BTreeMap::new(),
            child_links: Vec::new(),
            diagnostics: Vec::new(),
            next_node_id,
            next_symbol_id,
        }
    }

    pub fn allocate_node_id(&mut self) -> NamespaceNodeId {
        let id = NamespaceNodeId(self.next_node_id);
        self.next_node_id += 1;
        id
    }

    pub fn allocate_symbol_id(&mut self) -> SymbolId {
        let id = SymbolId(self.next_symbol_id);
        self.next_symbol_id += 1;
        id
    }

    pub fn next_node_id(&self) -> u64 {
        self.next_node_id
    }

    pub fn next_symbol_id(&self) -> u64 {
        self.next_symbol_id
    }

    pub fn insert_node(&mut self, node: NamespaceNode) {
        self.nodes.insert(node.id, node);
    }

    pub fn insert_symbol(&mut self, parent: NamespaceNodeId, symbol: SymbolObject) {
        self.child_links.push(ChildLink {
            parent,
            name: symbol.name.clone(),
            symbol: symbol.id,
            role: symbol.child_name_role(),
            provenance: symbol.provenance.clone(),
        });
        self.symbols.insert(symbol.id, symbol);
    }

    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }
}

/// Pending parent-to-child symbol link inside a namespace delta.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChildLink {
    pub parent: NamespaceNodeId,
    pub name: String,
    pub symbol: SymbolId,
    pub role: ChildNameRole,
    pub provenance: Provenance,
}

/// Closed syntax object passed to early meta-functions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntaxObject {
    pub kind: SyntaxObjectKind,
    pub provenance: Provenance,
}

/// Supported closed syntax object forms for the current vertical slice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyntaxObjectKind {
    Product(NormProduct),
}
