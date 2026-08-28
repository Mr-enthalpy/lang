//! The single connected semantic world used by namespace installation,
//! resolution, construction, and ordinary invocation.
//!
//! Namespace topology and its temporary declaration-record index are owned by
//! this object.  `CompilationWorld` never keeps a separately committed graph
//! snapshot: semantic declarations and their name index are staged by cloning
//! this complete world and committed together.
//!
//! ```text
//! one Symbol
//!   -> heterogeneous Val2 values
//!   -> each value's TypeValue
//!   -> that type's PatternValue
//!   -> resolved Pattern owner
//!   -> associated Symbols / Val2, including `()`
//! ```
//!
//! None of these identities are reconstructed from a `TypeValueId`.  A
//! `TypeValueId` is only a forward lookup key into `types`; there is no reverse
//! mapping to a defining namespace Symbol.

use std::collections::{BTreeMap, BTreeSet};

use lang_syntax::{
    NormClosure, NormClosurePlacement, NormExpr, NormLiteralKind, NormPatternElem, NormPolicySpec,
};

use crate::{
    canonical_value::{
        canonical_literal_norm, expand_extraction_navigation, CanonicalCompleteTypeNorm,
        CanonicalFullNavigation, CanonicalLiteralFamily, CanonicalNormForm, CanonicalObjectNorm,
        CanonicalPatternNorm, CanonicalPatternValue, CanonicalProductConstructor,
        CanonicalTypeCallSpaceNorm, CanonicalVal1Norm, CanonicalValueAddr, ExtractionPatternParent,
    },
    identity::{MetaCallableIdentity, SemanticValueId, TypeValueId},
    meta_invocation::TypeDefinitionInstanceId,
    meta_key::MetaInvocationMaterialKey,
    model::{
        CoreMetaFunction, NamespaceNodeId, Provenance, SemanticNameDelta, SymbolId, SymbolKind,
    },
    owner_namespace::{
        ExtractionMemberVisibility, NamespaceSymbolEntry, OwnerNamespaceGraph, OwnerNamespaceNodeId,
    },
    policy_pair::{
        elaborate_return_policy_pattern, CapabilityRealization, ExplicitP1Selection,
        NamespaceVisibility, PolicyMode, PolicyPair, PolicyResultEntry, PolicyView,
    },
    product_shape::{NonValueArgKind, ProductAtom, RawArgShape, RawArgValueClass},
    semantic_name_index::{BuildError, SemanticNameIndex},
    semantic_owner::{
        CallableOwnerPlacement, LocalCallableIdentity, LocalSymbolIdentity, PackageId,
        ResolvedPatternRootId, SemanticOwnerGraph, SemanticOwnerId, SemanticOwnerKind,
        SemanticSymbolIdentity,
    },
};

/// Canonical registration key for the type member generated inside a meta
/// invocation.
///
/// Storage key for one complete meta-instance root. Root identity is scoped by
/// the stable parent owner in addition to the selected callable and canonical
/// whole argument Product; body material never participates.
/// The normalized struct body is content *under* the root: replaying the
/// same root key with an equal body is an idempotent reuse, while a
/// different body under one root key is a construction conflict — never a
/// second root.  Two meta functions `f` and `g` whose bodies produce the
/// same normalized body from the same arguments still get distinct keys
/// (distinct roots) whose body material compares equal.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MetaInstanceRootKey {
    pub parent_owner: SemanticOwnerId,
    pub material: MetaInvocationMaterialKey,
}

/// Placement + identity bundle for one meta instance root.
///
/// `meta_callable` is the selected function object **value** identity.
/// `placement_parent` is the stable semantic owner under which this instance
/// is established. Together with the selected callable and canonical argument
/// Product held by `MetaInvocationMaterialKey`, both coordinates participate in root identity. Neither a
/// graph declaration Symbol nor a result binding/Place participates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MetaInstanceRoot {
    pub meta_callable: MetaCallableIdentity,
    pub placement_parent: SemanticOwnerId,
}

impl MetaInstanceRoot {
    /// Root-level Policy is an identity invariant, not a contextual default
    /// and not a position overlay. Stable ownership supplies global
    /// consistency; `plain` must not be replaced by `const` or interpreted as
    /// a Writable grant.
    pub const fn policy_mode(&self) -> PolicyMode {
        PolicyMode::Plain
    }
}

/// Owner of a PatternValue: either an open cluster construction or an
/// already-installed ClusterSymbol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatternClusterOwner {
    Open(ClusterConstructionId),
    Installed(SemanticSymbolIdentity),
}

impl PatternClusterOwner {
    pub fn installed(&self) -> Option<SemanticSymbolIdentity> {
        match *self {
            PatternClusterOwner::Installed(identity) => Some(identity),
            PatternClusterOwner::Open(_) => None,
        }
    }
}

/// Snapshot-local identity of one semantic Pattern value.
///
/// This is deliberately distinct from a Pattern root/scope, a TypeValue,
/// Symbol, value, and install place.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PatternValueId(pub u64);

/// Snapshot-local handle of one resolved Pattern scope.
///
/// The semantic owner and root identity remain explicit in the scope object;
/// this handle is only a map key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResolvedPatternScopeId(pub u64);

/// Snapshot-local handle of one per-object Val2 place.
///
/// Each semantic object (including `null × P × Val2` pure type Objects) owns
/// exactly one `ObjectPlace`.  The place holds the object's recursive
/// associated Val2 members — the values navigable from that specific object,
/// independent of other objects sharing the same Pattern.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectPlaceId(pub u64);

/// Identity of one resident Object occupying a Place.  A wholesale place
/// replacement allocates a new resident identity; it never reuses the old
/// projection-slot family merely because selectors are spelled the same.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResidentIdentity(pub u64);

/// Formation-time resident generation observed by projections and borrows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResidentGeneration {
    pub resident: ResidentIdentity,
    pub generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProjectionSelector {
    Named(String),
    Positional(usize),
}

/// Stable identity of one prospective or occupied projection slot.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProjectionSlotIdentity {
    pub parent: ResidentGeneration,
    pub selector: ProjectionSelector,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionSlotContents {
    Missing,
    Occupied,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectionSlot {
    pub identity: ProjectionSlotIdentity,
    pub contents: ProjectionSlotContents,
}

/// Context-indexed write grants.  No Policy carrier appears in this type:
/// Policy preference cannot manufacture a write capability.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WritableContext {
    places: BTreeSet<ObjectPlaceId>,
    slots: BTreeSet<ProjectionSlotIdentity>,
}

impl WritableContext {
    pub fn grant_place(&mut self, place: ObjectPlaceId) {
        self.places.insert(place);
    }

    pub fn grant_slot(&mut self, slot: ProjectionSlotIdentity) {
        self.slots.insert(slot);
    }

    pub fn place_is_writable(&self, place: ObjectPlaceId) -> bool {
        self.places.contains(&place)
    }

    pub fn slot_is_writable(&self, place: ObjectPlaceId, slot: &ProjectionSlotIdentity) -> bool {
        self.places.contains(&place) || self.slots.contains(slot)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BorrowViewId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BorrowKind {
    Ref,
    Share,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StableBorrowTarget {
    Place {
        place: ObjectPlaceId,
        resident: ResidentGeneration,
    },
    Projection(ProjectionSlotIdentity),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BorrowView {
    pub id: BorrowViewId,
    pub kind: BorrowKind,
    pub target: StableBorrowTarget,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BorrowOperand {
    Actual(StableBorrowTarget),
    Borrow(BorrowViewId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BorrowFormationFailure {
    UnknownBorrow(BorrowViewId),
    NoCandidateForStrengthening,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlaceMutationFailure {
    UnknownPlace(ObjectPlaceId),
    NotWritable,
    SlotAlreadyOccupied(ProjectionSlotIdentity),
    SlotMissing(ProjectionSlotIdentity),
}

fn projection_storage_key(selector: &ProjectionSelector) -> String {
    match selector {
        ProjectionSelector::Named(name) => name.clone(),
        ProjectionSelector::Positional(index) => format!("<position:{index}>"),
    }
}

/// Per-object Val2 container.
///
/// Each semantic object (including `null × P × Val2` pure type Objects) owns
/// exactly one `ObjectPlace`.  The place holds the object's recursive
/// associated Val2 members — the values navigable from that specific object,
/// independent of other objects sharing the same Pattern.
///
/// Val2 has exactly one authority per channel and both channels live here,
/// on the object:
///
/// * `associated_symbols` is the authority for every **source-visible**
///   Val2 name: `Val2(T_t)[f] = C_f` names one recursive ClusterSymbol, and
///   that Symbol's own member ledger carries the binding Policy.
/// * `associated_val2` is transport material: the value ids reachable from
///   this object, including compiler-installed anonymous entries such as
///   `()` call entries that never allocate a scope-local Symbol.
///
/// A source-visible name may appear in both channels during the current
/// transition (the Symbol is the authority, the value vector is the
/// navigable transport reference); the value vector must never become a
/// parallel member world with its own Policy facts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectPlace {
    pub id: ObjectPlaceId,
    /// Current resident generation.  This coordinate is horizontal residency
    /// state and never enters ordinary Object normalization.
    pub resident: ResidentGeneration,
    /// Source-navigable Val2 names of this specific object.  Two carriers of
    /// one Pattern (`let T: type = uint8; let U: type = T;`) own separate
    /// places, so `let f::T` adds `f` to `T`'s object only.
    pub associated_symbols: BTreeMap<String, SemanticSymbolIdentity>,
    /// Heterogeneous associated values indexed by ordinary member spelling.
    /// This is the per-object Val2 transport: `()` entries make the object
    /// callable, named entries are the navigable references of this
    /// object's members.
    pub associated_val2: BTreeMap<String, Vec<SemanticValueId>>,
}

/// Frozen semantic content of the owned `Val2(x)` coordinate.
///
/// This is intentionally not a navigation view and not a type callspace.
/// Lookup-visible Pattern fallback remains in the navigation APIs, while
/// `Norm_Val2` consumes only this snapshot.  Physical sharing may populate a
/// snapshot explicitly, but a normalizer never invents inheritance by reading
/// another object's place.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SemanticVal2Snapshot {
    clusters: BTreeMap<String, SemanticVal2ClusterSnapshot>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct SemanticVal2ClusterSnapshot {
    pure_p: Option<PurePMember>,
    values: Vec<SemanticValueId>,
}

/// Recursion state of one top-level `Norm_type` / `Norm_Val2` walk.
///
/// `frames` is the ACTIVE stack of objects currently being normalized, keyed
/// by observation coordinate plus Pattern.  Val2 normalization is
/// well-founded finite recursion, so re-entering an object still on this
/// stack proves an illegal cyclic Val2 and aborts the walk with a semantic
/// error — a cycle has no normal form.  `memo` caches only FINISHED subtrees
/// (shared acyclic material, e.g. a diamond) for the duration of one
/// top-level walk: canonical addresses are content-derived, but the
/// *content* of an open pure type Object changes as members are injected, so a
/// memo entry must never outlive the observation it was taken in.  Neither
/// the frame keys nor the memo keys reach any produced normal form.
#[derive(Clone, Debug, Default)]
struct Val2NormState {
    frames: Vec<(Option<ObjectPlaceId>, PatternValueId)>,
    memo: BTreeMap<(Option<ObjectPlaceId>, PatternValueId), CanonicalValueAddr>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticPatternValue {
    pub id: PatternValueId,
    pub root: ResolvedPatternRootId,
    pub scope: ResolvedPatternScopeId,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedPatternScope {
    pub id: ResolvedPatternScopeId,
    pub owner: SemanticOwnerId,
    pub root: ResolvedPatternRootId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticTypeValue {
    /// Opaque first-order lookup index for the core.  It is neither semantic
    /// type equality nor a whole complete-type snapshot identity.
    pub id: TypeValueId,
    pub pattern: PatternValueId,
    pub provenance: Provenance,
}

/// Facet of one direct TypeMember in an immutable `V_tau` snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypeMemberFacet {
    PureP,
    Value,
}

/// One ordinary Object captured as a direct TypeMember.
///
/// `direct_home` is fixed when the member is created.  A member may enter a
/// snapshot only when this root equals the current core's TypeMember scope;
/// this is the implementation boundary for `NoForeignTypeMemberInjection`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeMemberSnapshotEntry {
    pub direct_home: ResolvedPatternRootId,
    pub facet: TypeMemberFacet,
    pub value: SemanticValueId,
}

/// Immutable `V_tau` material captured by one complete type value.
pub type ImmutableTypeCallSpace = BTreeMap<String, Vec<TypeMemberSnapshotEntry>>;

/// Complete first-class type closure:
/// `tau = bind alpha.<Core(tau), CallSpace(tau)>`.
///
/// `lookup_key`, `core`, and `whole` are deliberately distinct.  The first is
/// a registry index, the second is ordinary semantic type equality, and the
/// third observes the immutable callspace snapshot as well.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompleteTypeValue {
    pub lookup_key: TypeValueId,
    pub core: CanonicalValueAddr,
    pub call_space: ImmutableTypeCallSpace,
    pub whole: CanonicalValueAddr,
}

/// Scope material selected by the outer components of one explicit
/// navigation: the host type member reached by the innermost
/// outer component and/or the namespace child of the same spelling.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticOuterScope {
    /// The complete host layer, not just its Pattern: the carrier Symbol,
    /// its own object place, and its own pure-P member view.  Navigation
    /// must not collapse this to a bare `PatternValueId` — the layered
    /// exposure conjunction `Expose(T_t, φ) ∧ Expose(C_f, φ)` needs the
    /// host's binding view, and per-carrier Val2 needs the host's place.
    pub host: Option<PatternHostMember>,
    pub namespace: Option<NamespaceNodeId>,
}

impl SemanticOuterScope {
    pub fn pattern(&self) -> Option<PatternValueId> {
        self.host.as_ref().map(|host| host.pattern)
    }
}

/// The result of the single recursive Symbol navigation shared by every use
/// context.
///
/// ```text
/// Path -> Symbol -> ContextDirectedProjection
/// ```
///
/// Which Symbol a path denotes must not depend on whether the path is later
/// used as a call target, a type, a value, an injection target, or an
/// extraction subject.  The navigation therefore always produces the same
/// `terminal_symbol` plus the `host_chain` it was reached through, and each
/// context afterwards projects only the facet it needs:
///
/// * call context: the terminal Symbol's callable sibling vals;
/// * type context: its pure-P member (and that member's own place);
/// * value context: its sibling vals;
/// * injection-target context: the writable host object/place;
/// * extraction context: the Pattern facet.
///
/// The `host_chain` is the layered exposure material of the navigation:
/// `Expose(t::f, φ) = Expose(T_t, φ) ∧ Expose(C_f, φ)` needs each traversed
/// host's own binding view, and per-carrier Val2 needs each host's own place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedSemanticNavigation {
    pub host_chain: Vec<PatternHostMember>,
    pub terminal_symbol: SemanticSymbolIdentity,
}

impl ResolvedSemanticNavigation {
    /// The innermost host layer the terminal Symbol was reached through, if
    /// the path navigated through an object at all.
    pub fn terminal_host(&self) -> Option<&PatternHostMember> {
        self.host_chain.last()
    }
}

/// One resolved host layer of a Val2 navigation.
///
/// A host is the pure-P member (the `null × P × Val2` pure type Object) that the
/// navigation stepped through.  `symbol` is the carrier that named it when
/// the step came from source navigation; compiler-internal Pattern-level
/// hosts have no carrier Symbol.  `place` is that object's own Val2 place,
/// and `view` is its own binding-level pure-P member view — the host factor
/// of the layered exposure conjunction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternHostMember {
    pub symbol: Option<SemanticSymbolIdentity>,
    pub pattern: PatternValueId,
    pub place: ObjectPlaceId,
    /// Whole-snapshot observation of the complete type carried by this host.
    /// `None` is reserved for compiler-internal Pattern hosts whose current
    /// direct TypeMember callspace is observed through the Pattern registry.
    pub complete_type: Option<CanonicalValueAddr>,
    pub view: Option<PolicyResultEntry<SemanticValueId, PatternValueId>>,
}

impl PatternHostMember {
    /// The host factor of `Expose(t::f, φ) = Expose(T_t, φ) ∧ Expose(f, φ)`.
    ///
    /// The host member's own Pattern stages are the navigability coordinate
    /// of everything reached through its Val2, so when that layer is not
    /// visible at `phase` nothing under the name is reachable.  This is a
    /// phase predicate, never a stage-set intersection: a `meta` host
    /// legitimately carries `compile` members.  A host with no recorded
    /// Pattern stage carries no exposure fact to compose.
    pub fn exposed_at(&self, phase: crate::Phase) -> bool {
        match &self.view {
            Some(view) if !view.view.pair.pattern.stages.is_empty() => {
                view.view.pair.pattern.stages.visible_at(phase)
            }
            _ => true,
        }
    }
}

/// One atomic semantic declaration-installation unit.
///
/// All entries stage against one declaration namespace and commit
/// all-or-nothing: application runs on a scratch copy of the semantic
/// world and the copy replaces the live world only after every entry
/// succeeded, so a failing entry leaves no partial semantic residue.
/// Any graph declaration projection is committed only as part of
/// the owning `CompilationWorld` transaction after this unit succeeds.
#[derive(Clone, Debug)]
pub struct SemanticNamespaceDelta {
    pub namespace: NamespaceNodeId,
    pub entries: Vec<SemanticDeclarationEntry>,
}

/// One staged semantic declaration installation.
#[derive(Clone, Debug)]
pub enum SemanticDeclarationEntry {
    /// `let () = closure` inside a Pattern-owned associated namespace.
    AssociatedCallEntry {
        pattern: PatternValueId,
        backing_declaration: SymbolId,
        closure: NormClosure,
        outer_p1_explicit: Option<ExplicitP1Selection>,
        callable_view: PolicyView,
        body_entry_view: PolicyView,
        namespace_visibility: Option<NamespaceVisibility>,
        candidate_role: OrdinaryCandidateRole,
        return_shape: ReturnShape,
        provenance: Provenance,
    },
    /// An ordinary named source callable declaration.
    SourceCallable {
        name: String,
        backing_declaration: SymbolId,
        closure: NormClosure,
        outer_p1_explicit: Option<ExplicitP1Selection>,
        function_view: PolicyView,
        body_entry_view: PolicyView,
        namespace_visibility: Option<NamespaceVisibility>,
        return_shape: ReturnShape,
        provenance: Provenance,
    },
    /// A declaration whose binder matches an existing cluster Symbol and
    /// contributes a sibling function object to that cluster.
    ClusterContribution {
        cluster_symbol: SemanticSymbolIdentity,
        backing_declaration: SymbolId,
        closure: NormClosure,
        outer_p1_explicit: Option<ExplicitP1Selection>,
        function_view: PolicyView,
        body_entry_view: PolicyView,
        namespace_visibility: Option<NamespaceVisibility>,
        return_shape: ReturnShape,
        provenance: Provenance,
    },
    /// A declared type carrier (`let t: type`), including the semantic
    /// registration of its associated namespace node.
    TypeCarrier {
        name: String,
        binding: SymbolId,
        represented_type: TypeValueId,
        /// Exact immutable complete-type snapshot carried by the semantic
        /// result/binding, when one is already known. `represented_type` is
        /// only its Core lookup projection and must not be used to rebuild a
        /// newer callspace snapshot.
        complete_type: Option<CanonicalValueAddr>,
        /// Associated namespace node together with its local spelling.
        associated_namespace: Option<(NamespaceNodeId, String)>,
        policy: PolicyPair,
        provenance: Provenance,
    },
    /// A declared name whose value is intentionally residual/unsupported in
    /// the current evaluator. It still receives a semantic Symbol identity so
    /// lexical lookup never depends on the declaration projection index. It
    /// still shadows outer same-spelled Symbols in every use context;
    /// projection failure never reopens name resolution.
    ProjectionOnly {
        name: String,
        backing_declaration: SymbolId,
        provenance: Provenance,
    },
}

/// The pure-P / type member of one ClusterSymbol.
///
/// A pure P is a real object (`Val1 = ∅`, `P`, `Val2`), so it owns its own
/// `ObjectPlace`.  The Pattern is shared identity material — `let T: type =
/// uint8` keeps `Pattern(T) = Pattern(uint8)` and `TypeValue(T) =
/// TypeValue(uint8)` — while the place is per object:
///
/// ```text
/// Pattern(T)   = Pattern(U)   = Pattern(uint8)
/// TypeValue(T) = TypeValue(U) = TypeValue(uint8)
/// Symbol(T)   ≠ Symbol(U)    ≠ Symbol(uint8)
/// Place(T)    ≠ Place(U)     ≠ Place(uint8)
/// ```
///
/// so a later `let f::T = expr;` writes `T`'s Val2 only.  Ordinary `let =`
/// binds a new object and therefore a fresh writable place. A future lexical
/// `let ===` pass installs no cell at all; it maps a local spelling to an
/// already-resolved terminal Symbol in a separate lexical environment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PurePMember {
    pub pattern: PatternValueId,
    pub place: ObjectPlaceId,
    /// The complete immutable type value read into this binding.  Two
    /// carriers can share `pattern`/Core while preserving different V_tau
    /// snapshots; later construction of a successor snapshot never rewrites
    /// this coordinate on an ordinary copied binding.
    pub complete_type: Option<CanonicalValueAddr>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticSymbolCell {
    pub identity: SemanticSymbolIdentity,
    pub name: String,
    pub declaration_owner: SemanticOwnerId,
    /// Graph namespace node the symbol was declared under.  `None` for
    /// Pattern-scope-local cluster symbols (meta-injected members), which
    /// have no graph namespace at all.
    pub namespace_node: Option<NamespaceNodeId>,
    /// The pure-P / type member of this cluster symbol, with its own place.
    ///
    /// A cluster symbol carries at most one pure P (Val1 = ∅).
    /// This must never store a `SemanticValueId`: pure P has no Val1.
    pub pure_p: Option<PurePMember>,
    /// Sibling vals (Val1 ≠ ∅) of this cluster symbol.
    ///
    /// These are not the Val2 of `pure_p`. Each sibling val has its own
    /// recursive Val1 × P × Val2 structure.
    ///
    /// A `CoreTypeProjection` adapter value must never appear in this list.
    pub sibling_vals: Vec<SemanticValueId>,
    /// Destination residency of each ordinary value member carried by this
    /// Symbol.  The semantic value may be shared with another binding, but
    /// `let` always establishes a fresh horizontal Place coordinate.
    ///
    /// This map is deliberately separate from `SemanticValueObject::place`:
    /// that field is the formation/storage place of the value object, while
    /// this field records where this binding currently carries the value.
    pub sibling_places: BTreeMap<SemanticValueId, ObjectPlaceId>,
    /// Per-binding Policy views over the cluster members.
    ///
    /// A pure-P-only view may use `value = None`. A sibling val view uses
    /// `value = Some(v)`. A P1 slice changes this association, not the
    /// SemanticValue identity.
    pub member_views: Vec<PolicyResultEntry<SemanticValueId, PatternValueId>>,
    pub provenance: Provenance,
}

impl SemanticSymbolCell {
    /// The PatternValue of this cluster symbol's pure-P member, without its
    /// place.  Type identity questions use this; anything that writes or
    /// reads this object's own Val2 must use the member's `place`.
    pub fn pure_p_pattern(&self) -> Option<PatternValueId> {
        self.pure_p.map(|member| member.pattern)
    }

    /// The own Val2 place of this cluster symbol's pure-P member.
    pub fn pure_p_place(&self) -> Option<ObjectPlaceId> {
        self.pure_p.map(|member| member.place)
    }

    /// This cluster symbol's own pure-P member view (`value = None`), the
    /// binding-level Policy authority of the pure P.  A globally reused
    /// CoreTypeProjection adapter is transport material and never a substitute.
    pub fn pure_p_view(&self) -> Option<&PolicyResultEntry<SemanticValueId, PatternValueId>> {
        let pattern = self.pure_p_pattern()?;
        self.member_views
            .iter()
            .find(|view| view.value.is_none() && view.pattern == pattern)
    }

    /// All sibling value ids (`Val1 ≠ ∅` members). A pure-P member never
    /// contributes an id here: its CoreTypeProjection adapter is Val2 transport
    /// material only and must not enter `sibling_vals`.
    #[allow(dead_code)]
    pub fn all_value_ids(&self) -> Vec<SemanticValueId> {
        self.sibling_vals.clone()
    }

    pub fn sibling_place(&self, value: SemanticValueId) -> Option<ObjectPlaceId> {
        self.sibling_places.get(&value).copied()
    }

    /// Derived cluster Policy disjunction over the installed member
    /// views; see [`derived_cluster_policy`].
    pub fn cluster_policy(&self) -> Option<PolicyPair> {
        derived_cluster_policy(&self.member_views)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticValueObject {
    pub id: SemanticValueId,
    pub type_value: TypeValueId,
    pub pattern: PatternValueId,
    /// Formation/storage Place of this value object. Ordinary bindings may
    /// carry this same semantic value in distinct destination Places; those
    /// horizontal residencies live on the binding Symbol and are never
    /// recovered from this field.
    pub place: ObjectPlaceId,
    pub policy: PolicyPair,
    pub mode: PolicyMode,
    pub namespace_visibility: Option<NamespaceVisibility>,
    pub payload: SemanticValuePayload,
    pub provenance: Provenance,
}

impl SemanticValueObject {
    pub fn policy_view(&self) -> PolicyView {
        PolicyView {
            pair: self.policy.clone(),
            mode: self.mode,
        }
    }
}

/// Declaration-event identity of a meta-injected value.
///
/// An injected member (`let f::t = fn_expr;` inside a meta body) is a
/// *local* callable of the canonical meta instance, never a re-exposure
/// of the outer meta function's declaration.  Its identity coordinates
/// are the canonical meta instance's structural coordinates (enclosing
/// meta callable × canonical argument-product address) plus the source
/// declaration event that performed the injection.  Neither the member
/// name nor the body participates: two distinct declaration events that
/// both write `f` are two sibling vals of one ClusterSymbol `f`, and
/// replaying the same canonical instance re-finds each event's value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct InjectedValueIdentity {
    pub enclosing_meta: MetaCallableIdentity,
    pub canonical_arguments: CanonicalValueAddr,
    pub construction_event: u32,
}

/// Declaration material recorded for one injected value identity.
/// Name/body material never participates in the identity itself; it only
/// drives the idempotence/conflict split under that identity.
#[derive(Clone, Debug)]
struct InjectedMemberRecord {
    value: SemanticValueId,
    member_name: String,
    declaration: NormClosure,
    canonical_view: PolicyView,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SemanticValuePayload {
    /// Ordinary non-callable value material installed by semantic evaluation.
    PlainValue,
    /// Simple literal value material carrying its canonical content normal
    /// form: `Norm(<Val1, P, Val2>)` re-derives from this payload, so equal-content
    /// materialized literals merge to one canonical address instead of
    /// staying identity-opaque.
    SimpleLiteral {
        family: CanonicalLiteralFamily,
        normalized: String,
    },
    /// Exact compile-time abstract literal (`integer`, `real`, or
    /// `character`).  Its TypeValue is the canonical abstract semantic Type;
    /// no concrete expected type participated in formation.
    AbstractLiteral {
        family: crate::AbstractLiteralFamily,
        canonical_family: CanonicalLiteralFamily,
        normalized: String,
    },
    /// Result of a later abstract-to-concrete construction.  The source
    /// abstract value is provenance/realization material, not part of Object
    /// normalization; exact literal content and the concrete Pattern remain
    /// the semantic value coordinates.
    ConstructedLiteral {
        source_abstract: SemanticValueId,
        /// Whole-snapshot identity of the exact complete target Type selected
        /// by ordinary construction.  The lookup key alone is not enough:
        /// two complete Types may share a Core while carrying different
        /// immutable TypeMember callspaces.
        target_complete_type: CanonicalValueAddr,
        canonical_family: CanonicalLiteralFamily,
        normalized: String,
    },
    /// Ordinary first-class result of continuation-relative `@`
    /// reification. The value records no Place coordinate.
    LifetimeValue(crate::LifetimeValue),
    /// A normal value member installed under a source-visible Symbol.
    FunctionObject { backing_declaration: SymbolId },
    /// A function object injected by a source meta body into the
    /// constructed type member's associated Val2 scope, as a sibling val
    /// of the member-name ClusterSymbol.  Its identity is the local
    /// declaration-event identity, not the outer meta function's backing
    /// declaration and not the member name.
    InjectedFunctionObject { identity: InjectedValueIdentity },
    /// An ordinary callable entry found through a type/Pattern owner's
    /// associated `()` Val2.
    CallEntry(OrdinaryCallEntry),
    /// Existing first-order type-value material.
    CoreTypeProjection {
        represented_type: TypeValueId,
        represented_pattern: PatternValueId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrdinaryCallEntry {
    pub backing_declaration: SymbolId,
    /// Declared spelling of the callable.  Together with
    /// `declaration_namespace` this is the call entry's own declaration
    /// environment: the A-stage never looks the backing declaration up in the
    /// read name index to recover identity or namespace material.
    pub declaration_name: String,
    /// Namespace the callable was declared in.  `None` for callables with
    /// no graph declaration site (meta-injected members).
    pub declaration_namespace: Option<NamespaceNodeId>,
    pub callable_owner: SemanticOwnerId,
    pub receiver_type: TypeValueId,
    /// Source body shape when this ordinary call entry was declared in
    /// language source. Core primitives and authorized ordinary intrinsics
    /// use their respective body coordinates instead; all three remain
    /// implementation bodies behind the same call-entry candidate.
    pub closure: Option<NormClosure>,
    pub core_primitive: Option<CoreMetaFunction>,
    pub(crate) intrinsic_body: Option<OrdinaryIntrinsicBody>,
    /// Callable P2 inherited by parameter positions and used for body-entry
    /// stage admissibility. It is declaration-local and never receives a
    /// caller's contextual result demand.
    pub body_entry_view: PolicyView,
    /// This call entry is a terminal FunctionItem: `Type(c) = FunctionItem(Self, Args...) -> R`
    /// and `c.Val2 = ∅`.  There is no recursive callable lookup from a
    /// CallEntry — the invocation spine terminates here.
    /// Producer result P2 exposed across the call boundary. This is the
    /// output-mode preference coordinate and remains distinct from the
    /// callable-internal return position.
    pub complete_result_view: PolicyView,
    /// Callable-internal return position: canonical P1 pair/stage plus the
    /// optional mode-only return annotation. Caller demand never propagates
    /// backward into this declaration-local view.
    pub return_position_view: PolicyView,
    /// Policy view declared by the ordinary callable value/member itself.
    ///
    /// For an authorized migration call this supplies the candidate's output
    /// endpoint coordinate. It is deliberately distinct from both the
    /// declaration-local P2 and the produced-result position view.
    pub callable_view: PolicyView,
    /// Orthogonal 3x3 realization facts. Policy comparison never derives or
    /// modifies these cells.
    pub capability_realization: CapabilityRealization,
    /// Current source construction always installs `Ordinary`.  The
    /// `Fallback` role is an internal carrier for the already-fixed future
    /// `A -> SuppressFallback -> Bp'` semantics; no surface spelling is
    /// inferred here.
    pub candidate_role: OrdinaryCandidateRole,
    /// Declared return-shape coordinate of the callable, mirrored at
    /// registration from the declaration payload: the aggregate shape of
    /// the invocation result (single value, single type, Symbol cluster,
    /// …).  Never re-derived from the Policy stage at call time.
    pub return_shape: ReturnShape,
    /// Independent declared privilege coordinate, owned by the call entry
    /// so the invocation spine never consults the graph payload: only
    /// compiler built-ins carry `BuiltinPrivileged`; the source surface
    /// can never spell it.
    pub privilege: CallablePrivilege,
    pub provenance: Provenance,
}

/// Compiler-authorized implementation body behind an ordinary call entry.
///
/// This is body data only. Candidate enumeration, A-stage applicability,
/// Policy preference, unique selection, DynamicLegality, and the no-reopen
/// boundary remain owned by the ordinary invocation pipeline.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum OrdinaryIntrinsicBody {
    AbstractLiteralConstruct(crate::BuiltinNumericConstructorSpec),
    /// Explicit deleted realization used by builtin declarations and
    /// authority-death tests. Selection of this candidate is terminal.
    Delete,
    /// Testable selected-body failure. It exists to prove that a runnable
    /// lower-ranked intrinsic is never retried after selection is sealed.
    FailSelected,
}

/// Reconcile the outer function-object P1 with the written-self P1 from the
/// closure head into a single canonical function-object P1.
///
/// P1(function object) = P1(outer let()) = P1(written self = slot0).
///
/// The canonical P1 is the COMPLETE `Pv:Pp` coordinate:
/// value stage / value mutability / value presence / Pattern stage. Each
/// spelling is completed independently against `Derive(P2)`. When both
/// spelling sites are present, the two complete pairs must be equal; their
/// explicitly written dimensions are never cross-assembled into a third P1.
///
/// `outer_explicit` is `Some` only when the user wrote P1-relevant
/// material in the declaration prefix (`compile let f = ...`,
/// `mut let f = ...`).  A stage-only prefix IS an explicit value-stage
/// selection — it no longer silently degrades to "no explicit P1".
/// `public/private/export` remain namespace declaration attributes and
/// never enter the P1; visibility/export of the canonical pair always
/// come from `outer_derived`.
pub fn canonical_function_object_view(
    outer_explicit: Option<&ExplicitP1Selection>,
    outer_derived: &crate::PolicyView,
    p2: &crate::PolicyView,
    closure: Option<&NormClosure>,
    provenance: &Provenance,
) -> Result<crate::PolicyView, crate::Diagnostic> {
    // Extract the raw written-self policy spec from the closure head, if any.
    let written_self_spec: Option<&NormPolicySpec> = closure
        .and_then(|c| c.head.as_ref())
        .and_then(|head| head.formal_frame().written_self)
        .and_then(|element| match element {
            NormPatternElem::BindingSlot(slot) => slot.policy.as_ref(),
            _ => None,
        });

    // Elaborate the written-self policy spec against P2 if present.
    // Propagate elaboration failures — do NOT swallow them.
    let self_explicit: Option<ExplicitP1Selection> = match written_self_spec {
        Some(spec) => crate::policy_pair::elaborate_explicit_p1(
            Some(spec),
            &p2.pair,
            crate::policy_pair::ExplicitP1Position::WrittenSelf,
            provenance.clone(),
        )?,
        None => None,
    };

    fn complete(selection: &ExplicitP1Selection, derived: &crate::PolicyView) -> crate::PolicyView {
        let mut complete = derived.clone();
        if let Some(stages) = &selection.value_stages {
            complete.pair.value.stages = stages.clone();
        }
        if let Some(presence) = selection.presence {
            complete.pair.value.presence = presence;
        }
        if let Some(stages) = &selection.pattern_stages {
            complete.pair.pattern.stages = stages.clone();
        }
        if let Some(mode) = selection.mode {
            complete.mode = mode;
        }
        complete
    }

    let canonical = match (outer_explicit, self_explicit.as_ref()) {
        (Some(outer), Some(written_self)) => {
            let complete_outer = complete(outer, outer_derived);
            let complete_self = complete(written_self, outer_derived);
            if complete_outer != complete_self {
                return Err(crate::Diagnostic::hard_error(
                    format!(
                        "canonical P1 mismatch: completed outer P1 {:?} != completed self P1 {:?}",
                        complete_outer, complete_self
                    ),
                    Some(provenance.clone()),
                ));
            }
            complete_outer
        }
        (Some(outer), None) => complete(outer, outer_derived),
        (None, Some(written_self)) => complete(written_self, outer_derived),
        (None, None) => outer_derived.clone(),
    };
    // Cross-site dimension fallback must not assemble an inconsistent
    // value component: `Pv = absent` carries neither stages nor const/mut.
    if canonical.pair.value.presence == crate::policy_pair::ValuePresence::Absent
        && !canonical.pair.value.stages.is_empty()
    {
        return Err(crate::Diagnostic::hard_error(
            "canonical P1: `Pv = absent` cannot carry value stages",
            Some(provenance.clone()),
        ));
    }
    Ok(canonical)
}

/// Declared return-shape / privilege coordinates of `CallableSemantics`.
/// Elaborated once at the declaration boundary and mirrored onto
/// `OrdinaryCallEntry`; the invocation pipeline reads
/// `selected.return_shape` directly and does NOT re-derive it from
/// `policy.stages.contains(Meta)` at call time.
pub use crate::policy_pair::{CallablePrivilege, PatternConstraint, ReturnShape};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrdinaryCandidateRole {
    Ordinary,
    Fallback,
}

/// Identity of one open cluster construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClusterConstructionId(pub u64);

/// Where a constructed result's Pattern owner comes from.  The owner
/// strategy is a fact of the selected callable and the call context; it is
/// never derived from the callable's return category (`MetaClusterConstruction
/// => create callee MetaInstanceScope` is exactly the collapsed rule this
/// separation forbids).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnerStrategy {
    /// Ordinary functions: results live in the callable's own result scope
    /// (the ordinary, non-cluster invocation path).
    OrdinaryCallableSelfScope,
    /// Ordinary (source-declared) meta functions: constructed type members
    /// are rooted at `MetaInstance(meta function, normalized arguments)`.
    OrdinaryMetaInstanceScope,
    /// The builtin privileged `struct` called directly in an ordinary
    /// declaration context: the generated type attaches to the ambient
    /// declaration environment as Self.  `struct` never creates its own
    /// externally navigable `MetaInstance(struct, arguments)` scope.
    AmbientStructScope,
    /// A privileged call whose owner is injected by an explicit rule:
    /// nested `struct` inside a meta body roots at the *outer* meta
    /// instance, and explicit input-pattern navigation may override the
    /// constructed owner (navigation override is future work).
    ExplicitPrivilegedOwnerRule,
}

/// Authority that owns an open cluster construction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConstructionAuthority {
    BuildRoot,
    MetaInvocation {
        meta_callable: MetaCallableIdentity,
        canonical_key: crate::meta_key::MetaInvocationMaterialKey,
    },
    /// An ambient-scope construction (`AmbientStructScope`): the owning
    /// authority is the declaration environment itself, not a meta
    /// instance of the invoked builtin.
    AmbientScope {
        owner: SemanticOwnerId,
    },
}

/// Dynamic construction-authority frames visible at one evaluation point.
///
/// The vector is ordered nearest-first and contains only authority-bearing
/// frames; transparent compile/intrinsic frames have already been erased by
/// the evaluator.  Keeping this coordinate outside `ConstructionState` is
/// essential: a live window does not by itself prove that the current
/// continuation owns the value's construction anchor.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConstructionEvaluationContext {
    frames_nearest_first: Vec<ConstructionAuthority>,
}

impl ConstructionEvaluationContext {
    pub fn current(authority: ConstructionAuthority) -> Self {
        Self {
            frames_nearest_first: vec![authority],
        }
    }

    pub fn from_frames(
        frames_nearest_first: impl IntoIterator<Item = ConstructionAuthority>,
    ) -> Self {
        Self {
            frames_nearest_first: frames_nearest_first.into_iter().collect(),
        }
    }

    pub fn frames_nearest_first(&self) -> &[ConstructionAuthority] {
        &self.frames_nearest_first
    }
}

/// Evidence for the contextual `OpenHere_Sigma(value)` judgment.
///
/// This proof contains no write grant.  It may authorize pure `extend` even
/// when the value has no writable carrier.  Mutation boundaries revalidate
/// it so a proof cannot revive a window that closed after observation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenHereProof {
    construction: ClusterConstructionId,
    target_pattern: PatternValueId,
    authority: ConstructionAuthority,
}

impl OpenHereProof {
    pub fn construction(&self) -> ClusterConstructionId {
        self.construction
    }

    pub fn target_pattern(&self) -> PatternValueId {
        self.target_pattern
    }
}

/// Separate evidence for member creation.  It deliberately does not grant
/// ordinary slot writability and is not interchangeable with `OpenHereProof`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemberCreationProof {
    open_here: OpenHereProof,
}

impl MemberCreationProof {
    pub fn open_here(&self) -> &OpenHereProof {
        &self.open_here
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OpenHereFailure {
    UnknownPattern(PatternValueId),
    NoLiveConstruction(PatternValueId),
    WindowClosed(ClusterConstructionId),
    AuthorityMismatch(ClusterConstructionId),
}

fn authority_matches_context(
    target: &ConstructionAuthority,
    context: &ConstructionEvaluationContext,
) -> bool {
    match target {
        // A meta invocation masks all outer construction authorities.  Only
        // the nearest meta frame may own a meta-generated anchor.
        ConstructionAuthority::MetaInvocation { .. } => {
            context
                .frames_nearest_first()
                .iter()
                .find(|frame| matches!(frame, ConstructionAuthority::MetaInvocation { .. }))
                == Some(target)
        }
        // Non-meta authority is searched outward, but never through a meta
        // boundary.  This models a still-active ordinary owner without
        // equating authority with the stack-top frame.
        ConstructionAuthority::AmbientScope { .. } | ConstructionAuthority::BuildRoot => {
            for frame in context.frames_nearest_first() {
                if frame == target {
                    return true;
                }
                if matches!(frame, ConstructionAuthority::MetaInvocation { .. }) {
                    return false;
                }
            }
            false
        }
    }
}

/// How the value of an ambient struct generation was bound at its
/// declaration site.  The binder NEVER participates in type identity —
/// ambient uniqueness is keyed by (level, normalized navigation shape)
/// alone; the binder is recorded purely so a later collision diagnostic
/// can point at the existing, source-visible binding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AmbientTypeBinder {
    /// `let X = ... struct;` — the whole result is bound to one symbol.
    WholeSymbol(String),
    /// An extraction binding has no whole-result symbol; only the
    /// extracted member symbols are visible.  (Recording this shape is
    /// future work: the connected binding path today records only whole
    /// `let` binders.)
    ExtractionMembers(Vec<String>),
    /// `((a inner) |> struct) |> f` — the generated value is bound to a
    /// callable parameter.  Meta evaluation is strictly left-to-right, so
    /// by the time a collision can occur the earlier binding is already
    /// known.  The parameter lives one level below the ambient declaration
    /// environment, so no symbol at this level references the value; the
    /// collision guidance asks the user to first bind the temporary value
    /// to a symbol at this level.  (Recording this shape is future work.)
    CallableParameter(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstructionState {
    Open,
    /// The construction window closed before boundary delivery.  A frozen
    /// construction rejects further member contribution and Val2
    /// injection, while boundary delivery (`finalize_type_cluster`) stays
    /// legal.  How a window closes depends on [`ConstructionWindow`]:
    /// a meta window freezes only on `UseForVal1`; an ordinary window
    /// freezes on first semantic use and on any residual-runtime
    /// fork/end boundary.
    Frozen,
    /// Boundary delivery happened: the construction left its formal
    /// construction boundary and was handed to the outer layer.
    Finalized,
}

/// Monotone coordinate of the residual runtime serial flow.
///
/// After `compile` stripping, the residual runtime flow is a serial
/// stream; every fork of that stream and every end of a serial segment
/// advances the epoch.  Purely static (compile-evaluated) branching
/// never advances it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResidualRuntimeEpoch(pub u64);

/// Open-window discipline of one cluster construction.
///
/// The freeze rules are NOT one uniform state machine.  The conservative
/// `Open --UseForVal1--> Frozen` family is the *meta construction
/// window* only; an ambient ordinary construction lives in an *ordinary
/// window* with its own closing coordinates:
///
/// ```text
/// MetaInvocation window:
///     Observe(P) / Transform(Val2)      keep open
///     UseForVal1                        freeze
///     static/compile-only branching     transparent
///
/// Ambient ordinary window:
///     FirstUse                          freeze
///     residual runtime fork / end       freeze
///     compile-only branching            transparent (never freezes)
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstructionWindow {
    /// Meta construction window: a meta-invocation or build-root
    /// construction transaction.  Observation and Val2 transformation
    /// keep it open across static control flow; only producing a Val1
    /// of the constructed type freezes it.
    Meta,
    /// Ambient ordinary construction window (`AmbientScope` authority).
    Ordinary(OrdinaryOpenWindow),
}

/// Window coordinates of an ambient ordinary construction.
///
/// The construction stays open from its creation flow segment until its
/// first semantic use, and never survives past the end or fork of the
/// residual runtime serial flow it was created in.  The coordinates and
/// transitions are the settled contract; deriving the closing events
/// from real source-level control-flow analysis is future work (see the
/// registered unclosed items in `spec/planning/open-questions.md`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OrdinaryOpenWindow {
    /// The residual-runtime flow segment the construction was created in.
    pub creation_flow_segment: ResidualRuntimeEpoch,
    pub first_use_seen: bool,
    pub closed_by_fork_or_end: bool,
}

/// Tracking of how a cluster construction has been used or observed.
///
/// In an ordinary window, the first semantic use freezes the
/// construction.  In a meta window, `ObserveOrTransform(P,Val2)` never
/// freezes; only `UseForVal1` does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UseObservationKind {
    pub has_been_used_for_val1: bool,
    pub has_been_observed_or_transformed: bool,
}

/// An open cluster construction that has not been installed yet.
#[derive(Clone, Debug)]
pub struct OpenClusterConstruction {
    pub id: ClusterConstructionId,
    pub owner: SemanticOwnerId,
    pub authority: ConstructionAuthority,
    /// Canonical member facts.  Every contribution records the complete
    /// Policy view (value slot, value Policy, Pattern, Pattern Policy).
    /// A pure-P member uses `value = None`; an ordinary sibling member
    /// uses `value = Some(v)`.  `pure_p()`/`sibling_vals()` are derived
    /// projections of this list; no parallel field can diverge from it,
    /// and no per-member Policy coordinate is unioned across members.
    pub member_views: Vec<PolicyResultEntry<SemanticValueId, PatternValueId>>,
    pub state: ConstructionState,
    /// The open-window discipline governing when this construction
    /// freezes; derived from `authority` at `begin_cluster_construction`.
    pub window: ConstructionWindow,
    pub use_observation: UseObservationKind,
    pub provenance: Provenance,
}

impl OpenClusterConstruction {
    /// Derived pure-P projection over the canonical member views.
    pub fn pure_p(&self) -> Option<PatternValueId> {
        derived_pure_p(&self.member_views)
    }

    /// Derived sibling-val projection over the canonical member views.
    pub fn sibling_vals(&self) -> Vec<SemanticValueId> {
        derived_sibling_vals(&self.member_views)
    }

    /// Derived cluster Policy disjunction over the canonical member
    /// views; see [`derived_cluster_policy`].
    pub fn cluster_policy(&self) -> Option<PolicyPair> {
        derived_cluster_policy(&self.member_views)
    }
}

/// Result of finalizing a cluster construction.
#[derive(Clone, Debug)]
pub struct SymbolConstructionValue {
    pub identity: ClusterConstructionId,
    /// Canonical member facts carried over unchanged from the open
    /// construction.  Installation must preserve these views verbatim; it
    /// must not re-derive member Policy from any cluster-level aggregate.
    pub member_views: Vec<PolicyResultEntry<SemanticValueId, PatternValueId>>,
    pub owner: SemanticOwnerId,
    pub provenance: Provenance,
}

impl SymbolConstructionValue {
    /// Derived pure-P projection over the canonical member views.
    pub fn pure_p(&self) -> Option<PatternValueId> {
        derived_pure_p(&self.member_views)
    }

    /// Derived sibling-val projection over the canonical member views.
    pub fn sibling_vals(&self) -> Vec<SemanticValueId> {
        derived_sibling_vals(&self.member_views)
    }

    /// Derived cluster Policy disjunction over the canonical member
    /// views; see [`derived_cluster_policy`].
    pub fn cluster_policy(&self) -> Option<PolicyPair> {
        derived_cluster_policy(&self.member_views)
    }
}

/// The single pattern of the `value = None` member views (a cluster
/// carries at most one pure P; contribution enforces this invariant).
fn derived_pure_p(
    views: &[PolicyResultEntry<SemanticValueId, PatternValueId>],
) -> Option<PatternValueId> {
    views
        .iter()
        .find(|view| view.value.is_none())
        .map(|view| view.pattern)
}

/// Sibling vals (Val1 ≠ ∅ members) in contribution order, deduplicated
/// by value identity (one value may carry several Policy views).
fn derived_sibling_vals(
    views: &[PolicyResultEntry<SemanticValueId, PatternValueId>],
) -> Vec<SemanticValueId> {
    let mut vals = Vec::new();
    for view in views {
        if let Some(value) = view.value {
            if !vals.contains(&value) {
                vals.push(value);
            }
        }
    }
    vals
}

/// Derived cluster Policy disjunction over the canonical member views:
///
/// ```text
/// cluster_policy(cluster)
///     = fold(policy_or, cluster.member_views.map(member_policy))
///
/// P_cluster = P_member_1 || ... || P_member_n
/// ```
///
/// EXCLUSIVITY LAW: this member → whole-function-object P1 disjunction
/// holds between the members of one ClusterSymbol and NOWHERE ELSE in the
/// model.  A Val2 name is itself a recursive ClusterSymbol
/// (`Val2(T_t)[f] = C_f`), so this same law applies unchanged one layer down
/// — `P(C_f)` is the disjunction of `C_f`'s own members.  What never happens
/// is absorption or aggregation ACROSS layers:
///
/// * a host type/cluster never disjoins its associated Symbols' Policies into
///   its own; injecting `t::f` leaves `P(T_t)` unchanged;
/// * layered exposure (`t::inner`) composes conjunctively at lookup
///   (`Expose(T_t, φ) ∧ Expose(x, φ)`), never disjunctively;
/// * a single object's P2 → P1 derivation unions its own value/pattern
///   facets — an intra-object completion, not a cross-member disjunction;
/// * no namespace, owner, or overload-selection layer forms a Policy
///   disjunction.
///
/// This is a pure derivation, never a storage authority: no cluster-level
/// aggregate is ever installed, and no per-member Policy coordinate is
/// re-derived from it.  Query and exposure keep filtering per member:
///
/// ```text
/// Expose(cluster, phase) = { member_i | Expose(P_i, phase) }
/// ```
///
/// A phase admitted by the disjunction exposes only the members whose own
/// view admits that phase.  Returns `None` for an empty member ledger.
pub fn derived_cluster_policy(
    views: &[PolicyResultEntry<SemanticValueId, PatternValueId>],
) -> Option<PolicyPair> {
    let mut folded: Option<PolicyPair> = None;
    for view in views {
        let member = view.view.pair.clone();
        folded = Some(match folded {
            None => member,
            Some(current) => crate::policy_pair::policy_or(&current, &member),
        });
    }
    folded
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindConflict {
    NoNamespaceOwner,
    ValueNotInstalled,
    AlreadyBound {
        name: String,
        identity: SemanticSymbolIdentity,
    },
}

/// Semantic facts materialized for one source callable declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredCallable {
    pub symbol: SemanticSymbolIdentity,
    pub function_value: SemanticValueId,
    pub function_type: TypeValueId,
    pub function_pattern: PatternValueId,
    pub pattern_scope: ResolvedPatternScopeId,
    pub call_entry: SemanticValueId,
}

/// One snapshot-local connected semantic world.
#[derive(Clone, Debug)]
pub struct SemanticWorld {
    /// Read projection of graph names embedded in the one semantic world.
    ///
    /// This is not an independently committed world.  It is retained while
    /// the remaining projection-oriented resolver adapters migrate to direct
    /// semantic objects; every mutation is committed with the surrounding
    /// `SemanticWorld` clone.
    namespace_index: SemanticNameIndex,
    owners: SemanticOwnerGraph,
    /// Canonical namespace topology and Symbol admission graph.
    ///
    /// `namespace_index` is a declaration-record projection only. Semantic
    /// path traversal, child lookup, and name-to-Symbol identity lookup read
    /// this typed graph and never reconstruct identity from that projection.
    owner_namespaces_graph: OwnerNamespaceGraph,
    owner_namespace_nodes: BTreeMap<NamespaceNodeId, OwnerNamespaceNodeId>,
    projected_namespace_nodes: BTreeMap<OwnerNamespaceNodeId, NamespaceNodeId>,
    toolchain_owner: SemanticOwnerId,
    package_owner: SemanticOwnerId,
    namespace_owners: BTreeMap<NamespaceNodeId, SemanticOwnerId>,
    owner_namespaces: BTreeMap<SemanticOwnerId, NamespaceNodeId>,
    local_symbol_counters: BTreeMap<SemanticOwnerId, u64>,
    local_pattern_root_counters: BTreeMap<SemanticOwnerId, u32>,
    symbols: BTreeMap<SemanticSymbolIdentity, SemanticSymbolCell>,
    values: BTreeMap<SemanticValueId, SemanticValueObject>,
    /// Exact immutable complete-Type snapshot captured when each ordinary
    /// Val1 is formed. This is `Type(v) = tau_v`; it is never reconstructed
    /// from the lookup key after later TypeMember contributions.
    value_complete_types: BTreeMap<SemanticValueId, CanonicalValueAddr>,
    types: BTreeMap<TypeValueId, SemanticTypeValue>,
    /// Direct TypeMembers admitted under each core lookup key.  This mutable
    /// table is construction substrate only: observing a complete type clones
    /// the admitted entries into an immutable `CompleteTypeValue` snapshot.
    /// Later contributions can produce a new snapshot but never mutate an
    /// existing one.
    direct_type_members: BTreeMap<TypeValueId, ImmutableTypeCallSpace>,
    /// Interned complete type closures keyed by whole-snapshot observation.
    complete_types: BTreeMap<CanonicalValueAddr, CompleteTypeValue>,
    /// Projection carrying a Core lookup identity for graph transport. This
    /// is NOT a semantic Val1
    /// (pure-P types have Val1 = ∅).  It never appears in sibling_vals.
    core_type_projection_values: BTreeMap<TypeValueId, SemanticValueId>,
    pattern_types: BTreeMap<PatternValueId, TypeValueId>,
    /// Canonical meta-type roots: `MetaRootKey = parent SemanticOwner + meta
    /// function + normalized arguments`. The stored `TypeDefinitionInstanceId` is body material
    /// used only for the idempotence/conflict split under one root (equal
    /// body ⇒ reuse, different body ⇒ construction conflict).  Two meta
    /// functions whose bodies produce the same normalized struct body
    /// material share the body, never the root:
    /// `Root(f(args)) != Root(g(args))` while `Body(f(args)) = Body(g(args))`.
    meta_type_roots: BTreeMap<MetaInstanceRootKey, (TypeValueId, TypeDefinitionInstanceId)>,
    /// Ambient struct generations: one generated type per (declaration
    /// level, normalized navigation shape).  A second direct `struct`
    /// generation with the same key at the same level is a hard error, not
    /// a silent reuse.
    ambient_struct_types: BTreeMap<(SemanticOwnerId, TypeDefinitionInstanceId), TypeValueId>,
    /// Diagnostic-only binder records for ambient struct generations.  The
    /// binder never participates in type identity; it exists so a
    /// collision can point at the existing source-visible binding.
    ambient_type_binders: BTreeMap<TypeValueId, AmbientTypeBinder>,
    patterns: BTreeMap<PatternValueId, SemanticPatternValue>,
    scopes: BTreeMap<ResolvedPatternScopeId, ResolvedPatternScope>,
    /// Object/residency Places. Semantic values have a formation Place;
    /// ordinary bindings may additionally carry the same value in fresh
    /// destination Places. Pattern-level lookups use `pattern_places` to find
    /// the canonical pure type Object's place.
    places: BTreeMap<ObjectPlaceId, ObjectPlace>,
    /// Owned Val2 semantic snapshots. Navigation and lookup may expose more
    /// material through `places`; ordinary Object normalization may not.
    semantic_val2_snapshots: BTreeMap<ObjectPlaceId, SemanticVal2Snapshot>,
    borrows: BTreeMap<BorrowViewId, BorrowView>,
    /// Forward map: PatternValue (as pure-P type) -> its pure type Object's place.
    /// This is the canonical type-level Val2 for this pattern.  Every
    /// pattern allocated via `allocate_pattern_and_scope` receives an entry.
    pattern_places: BTreeMap<PatternValueId, ObjectPlaceId>,
    /// Forward mapping: PatternValue → canonical owning ClusterSymbol.
    ///
    /// Recorded at first creation of a new owning PatternValue. Rebinding a
    /// type value to a new carrier Symbol does not rewrite this entry.
    pattern_clusters: BTreeMap<PatternValueId, PatternClusterOwner>,
    associated_namespace_patterns: BTreeMap<NamespaceNodeId, PatternValueId>,
    backing_to_function_value: BTreeMap<SymbolId, SemanticValueId>,
    /// Projection-only link from a semantic Symbol to its source/core
    /// declaration record.  Resolution chooses the semantic Symbol first;
    /// this link is consulted only when an older API asks for a
    /// `SymbolObject` rendering.
    symbol_backing_declarations: BTreeMap<SemanticSymbolIdentity, SymbolId>,
    registered_type_bindings: BTreeSet<SymbolId>,
    open_clusters: BTreeMap<ClusterConstructionId, OpenClusterConstruction>,
    /// Replay registry for meta-injected local callables.
    /// One injected callable identity (enclosing meta callable × canonical
    /// instance × member name) maps to its installed value plus the
    /// declaration material that produced it: replaying with equal
    /// material is an idempotent reuse, replaying with different material
    /// is a construction conflict.
    injected_members: BTreeMap<InjectedValueIdentity, InjectedMemberRecord>,
    /// Recorded structural normal-form material per PatternValue:
    /// meta-generated struct patterns normalize by their
    /// normalized structural body, so two separately allocated
    /// PatternValues with equal bodies share one `Norm_P(P)`.  Nominal
    /// declaration patterns never enter this table — their declaration root
    /// IS their normal form.
    pattern_structural_norms: BTreeMap<PatternValueId, CanonicalPatternValue>,
    /// Interning table for `Addr(v) = Intern(Norm(v))`: equal canonical
    /// normal forms share one snapshot-local address.  Opaque fresh
    /// addresses (material without a stable normal form) are allocated from
    /// the same counter but never enter this table, so they never merge.
    canonical_value_addrs: BTreeMap<CanonicalNormForm, CanonicalValueAddr>,
    opaque_val1_ids: BTreeMap<SemanticValueId, crate::OpaqueVal1Id>,
    type_rank: Option<TypeValueId>,
    symbol_rank: Option<TypeValueId>,
    /// Current residual-runtime flow segment.  Ambient ordinary
    /// construction windows record it at creation and never survive a
    /// later segment (`note_residual_runtime_fork_or_end`).
    residual_runtime_epoch: ResidualRuntimeEpoch,
    next_cluster: u64,
    next_callable: u64,
    next_value: u64,
    next_anonymous_type: u64,
    next_pattern: u64,
    next_scope: u64,
    next_place: u64,
    next_resident: u64,
    next_borrow: u64,
    next_canonical_value_addr: u64,
    next_opaque_val1: u64,
}

/// The resolved target of one source-level extraction.
///
/// A completed extraction navigation resolves to exactly one Symbol; the
/// extraction then reads that Symbol's PatternValue and compares canonical
/// pattern norms.  No step of this chain re-enters bare-name lookup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedExtractionTarget {
    pub symbol: SemanticSymbolIdentity,
    pub pattern: PatternValueId,
    pub norm: CanonicalPatternNorm,
}

impl SemanticWorld {
    pub fn new(package_name: impl Into<String>) -> Self {
        let mut owners = SemanticOwnerGraph::new();
        let toolchain_owner = owners.package_root(PackageId(0), "<toolchain-global>");
        let package_owner = owners.package_root(PackageId(1), package_name);
        Self {
            namespace_index: SemanticNameIndex::new(),
            owners,
            owner_namespaces_graph: OwnerNamespaceGraph::new(),
            owner_namespace_nodes: BTreeMap::new(),
            projected_namespace_nodes: BTreeMap::new(),
            toolchain_owner,
            package_owner,
            namespace_owners: BTreeMap::new(),
            owner_namespaces: BTreeMap::new(),
            local_symbol_counters: BTreeMap::new(),
            local_pattern_root_counters: BTreeMap::new(),
            symbols: BTreeMap::new(),
            values: BTreeMap::new(),
            value_complete_types: BTreeMap::new(),
            types: BTreeMap::new(),
            direct_type_members: BTreeMap::new(),
            complete_types: BTreeMap::new(),
            core_type_projection_values: BTreeMap::new(),
            pattern_types: BTreeMap::new(),
            meta_type_roots: BTreeMap::new(),
            ambient_struct_types: BTreeMap::new(),
            ambient_type_binders: BTreeMap::new(),
            patterns: BTreeMap::new(),
            scopes: BTreeMap::new(),
            places: BTreeMap::new(),
            semantic_val2_snapshots: BTreeMap::new(),
            borrows: BTreeMap::new(),
            pattern_places: BTreeMap::new(),
            pattern_clusters: BTreeMap::new(),
            associated_namespace_patterns: BTreeMap::new(),
            backing_to_function_value: BTreeMap::new(),
            symbol_backing_declarations: BTreeMap::new(),
            registered_type_bindings: BTreeSet::new(),
            open_clusters: BTreeMap::new(),
            injected_members: BTreeMap::new(),
            pattern_structural_norms: BTreeMap::new(),
            canonical_value_addrs: BTreeMap::new(),
            opaque_val1_ids: BTreeMap::new(),
            type_rank: None,
            symbol_rank: None,
            residual_runtime_epoch: ResidualRuntimeEpoch::default(),
            next_cluster: 0,
            next_callable: 0,
            next_value: 0,
            // Anonymous type values live in a disjoint provisional allocation
            // range.  This is representation only; semantic equality remains
            // the TypeValueId itself and never a SymbolId conversion.
            next_anonymous_type: 1u64 << 63,
            next_pattern: 0,
            next_scope: 0,
            next_place: 0,
            next_resident: 0,
            next_borrow: 0,
            next_canonical_value_addr: 0,
            next_opaque_val1: 0,
        }
    }

    pub(crate) fn replace_namespace_index(&mut self, index: SemanticNameIndex) {
        self.namespace_index = index;
        // Adopting the bootstrap name index is an atomic
        // SemanticWorld installation: every namespace node in the adopted
        // tree receives its semantic owner here, parents first.  After
        // adoption no outer layer ever back-fills owners from the index
        // (`sync_semantic_namespace_chain` is deleted).
        let nodes = self
            .namespace_index
            .namespace_nodes()
            .map(|node| (node.id, node.parent, node.name.clone()))
            .collect::<Vec<_>>();
        self.register_namespace_owner_closure(nodes)
            .expect("adopted namespace index forms a rooted tree");

        // Namespace Symbols are semantic Symbols too.  Earlier bootstrap
        // adoption registered only namespace topology, leaving compiler-owned
        // Admit bootstrap namespace binding identities into the typed graph;
        // their SymbolObjects remain read projections of that graph.
        let namespace_symbols = self
            .namespace_index
            .symbols()
            .values()
            .filter(|symbol| symbol.kind == crate::SymbolKind::Namespace)
            .filter_map(|symbol| {
                Some((
                    symbol.id,
                    symbol.parent?,
                    symbol.name.clone(),
                    symbol.provenance.clone(),
                ))
            })
            .collect::<Vec<_>>();
        for (backing, namespace, name, provenance) in namespace_symbols {
            let owner = self
                .namespace_owner(namespace)
                .expect("adopted namespace Symbol parent has a semantic owner");
            let identity = self.intern_symbol(namespace, owner, &name, provenance);
            self.symbol_backing_declarations.insert(identity, backing);
        }
    }

    /// Registers semantic owners for the given `(node, parent, name)` facts,
    /// parents first.  Nodes that already carry an owner are kept as-is
    /// (`register_namespace` is idempotent on owned nodes), so pre-bound
    /// roots such as the toolchain root and the package namespace survive
    /// adoption unchanged.
    fn register_namespace_owner_closure(
        &mut self,
        nodes: Vec<(NamespaceNodeId, Option<NamespaceNodeId>, String)>,
    ) -> Result<(), BuildError> {
        let mut pending = nodes;
        while !pending.is_empty() {
            let before = pending.len();
            pending.retain(|(id, parent, name)| {
                if self.owner_namespace_nodes.contains_key(id) {
                    return false;
                }
                let Some(parent) = parent else {
                    // A pre-bound root may not yet have been present in the
                    // adopted declaration projection. Materialize its typed
                    // node now; an unbound root remains invalid input.
                    self.ensure_owner_namespace_node(*id, None, name.clone());
                    return false;
                };
                if !self.owner_namespace_nodes.contains_key(parent) {
                    return true;
                }
                self.register_namespace(*id, *parent, name.clone())
                    .expect("checked semantic namespace parent");
                false
            });
            if pending.len() == before {
                return Err(BuildError::single(crate::Diagnostic::hard_error(
                    "namespace delta contains topology whose parent is not installed",
                    None,
                )));
            }
        }
        Ok(())
    }

    pub fn namespace_index(&self) -> &SemanticNameIndex {
        &self.namespace_index
    }

    pub(crate) fn install_namespace_name_delta(
        &mut self,
        delta: crate::SemanticNameDelta,
    ) -> Result<(), BuildError> {
        // One atomic installation: the name-index snapshot
        // advances and every namespace node introduced by the delta receives
        // its semantic owner in the same operation.  There is no separate
        // "sync the other representation" step anywhere above this method.
        let new_nodes = delta
            .nodes
            .values()
            .map(|node| (node.id, node.parent, node.name.clone()))
            .collect::<Vec<_>>();
        let projection_symbols = delta
            .symbols
            .values()
            .filter_map(|symbol| {
                Some((
                    symbol.id,
                    symbol.parent?,
                    symbol.name.clone(),
                    symbol.kind,
                    symbol.provenance.clone(),
                ))
            })
            .collect::<Vec<_>>();
        let mut staged = self.clone();
        staged.namespace_index = staged
            .namespace_index
            .install_delta(delta)
            .map_err(BuildError::from)?;
        staged.register_namespace_owner_closure(new_nodes)?;
        for (backing, namespace, name, kind, provenance) in projection_symbols {
            let identity = staged
                .symbol_in_namespace(namespace, &name)
                .map(|symbol| symbol.identity)
                .or_else(|| {
                    (kind == SymbolKind::Namespace).then(|| {
                        let owner = staged
                            .namespace_owner(namespace)
                            .expect("new namespace Symbol parent has a semantic owner");
                        staged.intern_symbol(namespace, owner, &name, provenance)
                    })
                });
            if let Some(identity) = identity {
                staged
                    .symbol_backing_declarations
                    .entry(identity)
                    .or_insert(backing);
            }
        }
        *self = staged;
        Ok(())
    }

    /// Atomically ensures a namespace path below `root`.  Existing
    /// namespace-capable symbols are reused; each missing component is
    /// installed as one name delta whose namespace node receives its
    /// semantic owner inside the same [`Self::install_namespace_name_delta`]
    /// operation — there is no separate owner back-fill step.
    pub(crate) fn ensure_namespace_path(
        &mut self,
        root: NamespaceNodeId,
        components: &[String],
        node_kind: crate::NamespaceNodeKind,
        source_category: crate::SourceCategory,
        provenance_description: &str,
    ) -> Result<NamespaceNodeId, BuildError> {
        let mut current = root;
        for component in components {
            // Reuse any existing namespace-capable symbol for this component:
            // either a declared namespace-subspace symbol or an object symbol
            // carrying an associated namespace node (e.g. a type symbol's
            // type-associated namespace).
            if let Some(existing) = self.namespace_capable_child(current, component) {
                current = existing;
                continue;
            }
            let mut delta = self.namespace_index.empty_delta();
            let next = crate::semantic_name_index::namespace_symbol(
                &mut delta,
                current,
                component,
                node_kind,
                source_category,
                crate::Provenance::new(provenance_description),
            );
            self.install_namespace_name_delta(delta)?;
            current = next;
        }
        Ok(current)
    }

    /// Register semantic namespace topology and source-navigable projection
    /// Symbols carried by a generated declaration projection.
    ///
    /// Generated field functions do not yet have an executable semantic body,
    /// but they still require a Semantic Symbol identity: path resolution and
    /// Pattern association may not discover them by querying the graph
    /// name index. The whole registration is staged on a clone, so malformed
    /// topology leaves no partial semantic residue.
    pub(crate) fn register_generated_projection_symbols(
        &mut self,
        delta: &SemanticNameDelta,
    ) -> Result<(), BuildError> {
        let mut staged = self.clone();
        let mut pending = delta.nodes.values().collect::<Vec<_>>();
        while !pending.is_empty() {
            let before = pending.len();
            pending.retain(|node| {
                if staged.namespace_owner(node.id).is_some() {
                    return false;
                }
                let Some(parent) = node.parent else {
                    return false;
                };
                if staged.namespace_owner(parent).is_none() {
                    return true;
                }
                staged
                    .register_namespace(node.id, parent, node.name.clone())
                    .expect("checked semantic namespace parent");
                false
            });
            if pending.len() == before {
                return Err(BuildError::single(crate::Diagnostic::hard_error(
                    "generated projection contains namespace topology whose parent is not installed",
                    pending.first().map(|node| node.provenance.clone()),
                )));
            }
        }

        // Bind every newly installed graph declaration record to the
        // already-authoritative typed Symbol selected by namespace+name.
        // Ordinary bindings create the typed Symbol before committing this
        // delta, so the projection must never be used to reconstruct it.
        for object in delta.symbols.values() {
            let Some(parent) = object.parent else {
                continue;
            };
            if let Some(identity) = staged
                .symbol_in_namespace(parent, &object.name)
                .map(|symbol| symbol.identity)
            {
                staged
                    .symbol_backing_declarations
                    .entry(identity)
                    .or_insert(object.id);
            }
        }

        for object in delta
            .symbols
            .values()
            .filter(|object| object.kind == SymbolKind::FieldFunction)
        {
            let parent = object.parent.ok_or_else(|| {
                BuildError::single(crate::Diagnostic::hard_error(
                    "generated field projection has no owning namespace",
                    Some(object.provenance.clone()),
                ))
            })?;
            let owner = staged.namespace_owner(parent).ok_or_else(|| {
                BuildError::single(crate::Diagnostic::hard_error(
                    "generated field projection namespace has no semantic owner",
                    Some(object.provenance.clone()),
                ))
            })?;
            let symbol =
                staged.intern_symbol(parent, owner, &object.name, object.provenance.clone());
            staged
                .symbol_backing_declarations
                .entry(symbol)
                .or_insert(object.id);
            if let Some(pattern) = staged.pattern_for_associated_namespace(parent) {
                staged
                    .associate_existing_symbol(pattern, &object.name, symbol)
                    .expect("generated projection Symbol was just installed");
            }
        }
        *self = staged;
        Ok(())
    }

    /// `Addr(v) = Intern(Norm(v))`: equal normal forms share one address.
    pub fn intern_canonical_value(&mut self, norm: CanonicalNormForm) -> CanonicalValueAddr {
        if let Some(existing) = self.canonical_value_addrs.get(&norm) {
            return *existing;
        }
        let addr = CanonicalValueAddr(self.next_canonical_value_addr);
        self.next_canonical_value_addr += 1;
        self.canonical_value_addrs.insert(norm, addr);
        addr
    }

    /// Diagnostic/proof inspection of an interned normal form.  This reverse
    /// query never recovers a semantic value, Symbol, Place, or type lookup
    /// key from an address.
    pub fn canonical_normal_form(&self, addr: CanonicalValueAddr) -> Option<&CanonicalNormForm> {
        self.canonical_value_addrs
            .iter()
            .find_map(|(form, candidate)| (*candidate == addr).then_some(form))
    }

    fn opaque_val1_id(&mut self, value: SemanticValueId) -> crate::OpaqueVal1Id {
        if let Some(existing) = self.opaque_val1_ids.get(&value) {
            return *existing;
        }
        let opaque = crate::OpaqueVal1Id(self.next_opaque_val1);
        self.next_opaque_val1 = self
            .next_opaque_val1
            .checked_add(1)
            .expect("opaque Val1 identity exhausted");
        self.opaque_val1_ids.insert(value, opaque);
        opaque
    }

    /// `Norm_Val2(Val2(x))` of one frozen owned semantic snapshot.
    ///
    /// Lookup inheritance is intentionally absent.  Generated/common members
    /// that are physically shared must already have been contributed to this
    /// snapshot at object formation; the normalizer cannot manufacture an
    /// `EffectiveVal2` by reading the Pattern's canonical place.
    fn canonical_val2_norm(
        &mut self,
        snapshot: &SemanticVal2Snapshot,
        state: &mut Val2NormState,
    ) -> Result<crate::canonical_value::CanonicalVal2Norm, crate::Diagnostic> {
        let mut val2 = crate::canonical_value::CanonicalVal2Norm::new();
        for (name, cluster) in &snapshot.clusters {
            let norm = self.canonical_cluster_norm(cluster, state)?;
            if !norm.is_empty() {
                val2.insert(name.clone(), norm);
            }
        }
        Ok(val2)
    }

    /// `Norm_Cluster(C) = ⟨Norm_pureP(C.pureP)?, Multiset{Norm_val(v)}⟩` for
    /// one Val2 name.
    ///
    /// `Val2(T_t)[f] = C_f`, so the name resolves to its ClusterSymbol first
    /// and that Symbol's own members are the normalized material.  Only
    /// compiler-installed transport entries without a scope-local Symbol (the
    /// `()` call entries of a materialized type) fall back to the place's
    /// transport value vector.
    fn canonical_cluster_norm(
        &mut self,
        cluster: &SemanticVal2ClusterSnapshot,
        state: &mut Val2NormState,
    ) -> Result<crate::canonical_value::CanonicalClusterNorm, crate::Diagnostic> {
        let pure_p = match cluster.pure_p {
            Some(member) => {
                Some(self.canonical_pure_type_address(Some(member.place), member.pattern, state)?)
            }
            None => None,
        };
        let vals = cluster
            .values
            .iter()
            .copied()
            .map(|value| self.canonical_member_value_address(value, state))
            .collect::<Result<_, _>>()?;
        Ok(crate::canonical_value::CanonicalClusterNorm::new(
            pure_p, vals,
        ))
    }

    /// `Norm_pureP(x) = ⟨Norm_P(P_x), Norm_Val2(Val2_x)⟩` — the recursive
    /// normal form of one pure type Object, observed through `place`.
    ///
    /// Val2 normalization is well-founded finite recursion: every traversed
    /// object edge must descend toward a leaf (`Children_V(x) = ∅`).
    /// Re-entering an object still on the ACTIVE recursion stack therefore
    /// proves an illegal cyclic Val2 (`let f::t = t;`) and is a hard semantic
    /// error — a cycle has no normal form.  Shared acyclic subtrees (a
    /// diamond) stay legal: FINISHED subtrees are memoized for the duration
    /// of one top-level canonicalization only, so a later injection must be
    /// observed by the next canonicalization.
    fn canonical_pure_type_address(
        &mut self,
        place: Option<ObjectPlaceId>,
        pattern: PatternValueId,
        state: &mut Val2NormState,
    ) -> Result<CanonicalValueAddr, crate::Diagnostic> {
        self.canonical_object_address(place, pattern, None, state)
    }

    /// Apply the one ordinary Object rule:
    /// `Norm(x)=<Norm_Val1?,Norm_P,Norm_Val2>`.
    fn canonical_object_address(
        &mut self,
        place: Option<ObjectPlaceId>,
        pattern: PatternValueId,
        val1: Option<CanonicalVal1Norm>,
        state: &mut Val2NormState,
    ) -> Result<CanonicalValueAddr, crate::Diagnostic> {
        let key = (place, pattern);
        if state.frames.contains(&key) {
            return Err(crate::Diagnostic::hard_error(
                "cyclic Val2: this object is reached again while its own Val2 is still \
                 being normalized; Val2 normalization is well-founded finite recursion, \
                 so a cyclic Val2 has no normal form and is rejected",
                None,
            ));
        }
        if let Some(addr) = state.memo.get(&key) {
            return Ok(*addr);
        }
        let pattern_norm = self.canonical_pattern_norm(pattern).ok_or_else(|| {
            crate::Diagnostic::hard_error(
                "ordinary Object normalization requires an observable Pattern normal form",
                None,
            )
        })?;
        let snapshot = place
            .and_then(|place| self.semantic_val2_snapshots.get(&place).cloned())
            .unwrap_or_default();
        state.frames.push(key);
        let val2 = self.canonical_val2_norm(&snapshot, state);
        state.frames.pop();
        let addr = self.intern_canonical_value(CanonicalNormForm::Object(CanonicalObjectNorm {
            val1,
            pattern: pattern_norm,
            val2: val2?,
        }));
        state.memo.insert(key, addr);
        Ok(addr)
    }

    /// `Norm_val(v)` of one Val2 member value.
    ///
    /// Literal content and type-object material normalize by content; a
    /// materialized call entry normalizes as the object
    /// `⟨Norm_P(P_FunctionItem), Norm_Val2(∅)⟩`, which is where the recursion
    /// bottoms out. Payloads without a content normalizer use a stable opaque
    /// Val1 leaf, which safely under-merges rather than inventing equality.
    fn canonical_member_value_address(
        &mut self,
        value: SemanticValueId,
        state: &mut Val2NormState,
    ) -> Result<CanonicalValueAddr, crate::Diagnostic> {
        let object = self.values.get(&value).cloned().ok_or_else(|| {
            crate::Diagnostic::hard_error(
                "ordinary Object normalization received an unknown semantic value",
                None,
            )
        })?;
        let place = object.place;
        let pattern = object.pattern;
        match object.payload {
            SemanticValuePayload::CoreTypeProjection {
                represented_pattern,
                ..
            } => self.canonical_pure_type_address(Some(place), represented_pattern, state),
            SemanticValuePayload::SimpleLiteral { family, normalized } => self
                .canonical_object_address(
                    Some(place),
                    pattern,
                    Some(CanonicalVal1Norm::Literal { family, normalized }),
                    state,
                ),
            SemanticValuePayload::AbstractLiteral {
                canonical_family: family,
                normalized,
                ..
            }
            | SemanticValuePayload::ConstructedLiteral {
                canonical_family: family,
                normalized,
                ..
            } => self.canonical_object_address(
                Some(place),
                pattern,
                Some(CanonicalVal1Norm::Literal { family, normalized }),
                state,
            ),
            SemanticValuePayload::LifetimeValue(lifetime) => self.canonical_object_address(
                Some(place),
                pattern,
                Some(CanonicalVal1Norm::Lifetime(lifetime)),
                state,
            ),
            SemanticValuePayload::FunctionObject { .. }
            | SemanticValuePayload::InjectedFunctionObject { .. } => self.canonical_object_address(
                Some(place),
                pattern,
                Some(CanonicalVal1Norm::FunctionObject),
                state,
            ),
            SemanticValuePayload::CallEntry(_) => self.canonical_object_address(
                Some(place),
                pattern,
                Some(CanonicalVal1Norm::CallEntry),
                state,
            ),
            SemanticValuePayload::PlainValue => {
                let opaque = self.opaque_val1_id(value);
                self.canonical_object_address(
                    Some(place),
                    pattern,
                    Some(CanonicalVal1Norm::Opaque(opaque)),
                    state,
                )
            }
        }
    }

    /// Normalize one call-argument position into its canonical interning
    /// address.
    ///
    /// Simple closed material — resolved pure type Objects (pure-P), product
    /// units, and literal spellings — receives content-normalized addresses.
    /// Material with an implemented content normalizer uses it; otherwise an
    /// identity-stable opaque Val1 leaf preserves a normal form without
    /// claiming content equality.
    ///
    /// A type argument normalizes through
    /// `Norm_type(x) = ⟨none, Norm_P(P_x), Norm_Val2(Val2_x)⟩`, where the Val2 is
    /// read from the argument's own carrier place when the resolution carried
    /// one.  Two carriers of one Pattern with equal recursive Val2 therefore
    /// share one address even though their places differ, and one open type
    /// observed before and after an injection does not.
    pub fn canonical_argument_address(
        &mut self,
        raw: &RawArgShape,
        atom: &ProductAtom,
    ) -> Result<CanonicalValueAddr, crate::Diagnostic> {
        let mut state = Val2NormState::default();
        if let Some(value) = raw.known_semantic_value {
            return self.canonical_member_value_address(value, &mut state);
        }
        match &raw.value_class {
            RawArgValueClass::NonValue(NonValueArgKind::CoreTypeProjection) => {
                if let Some(type_value) = raw.known_first_order_type_value {
                    if let Some(whole) = raw.known_complete_type_observation {
                        return Ok(whole);
                    }
                    let place = raw.known_type_carrier_place;
                    return Ok(self.observe_complete_type(type_value, place)?.whole);
                }
            }
            RawArgValueClass::NonValue(NonValueArgKind::ProductUnit) => {
                return Ok(self.intern_canonical_value(CanonicalNormForm::Object(
                    CanonicalObjectNorm {
                        val1: Some(CanonicalVal1Norm::ProductUnit),
                        pattern: CanonicalPatternNorm::ProductUnit,
                        val2: Default::default(),
                    },
                )));
            }
            _ => {}
        }
        if let ProductAtom::Expression { expr, .. } = atom {
            if let NormExpr::Literal { kind, text, .. } = expr {
                return Ok(self.intern_canonical_value(canonical_literal_norm(*kind, text)));
            }
        }
        Err(crate::Diagnostic::hard_error(
            "ordinary Object normalization requires observable Val1, Pattern, and Val2 material",
            None,
        ))
    }

    /// `Addr(Norm_type(type_value, place))` — the interned observation
    /// address for one type value read at one observation place.
    ///
    /// This is the same normal form `canonical_argument_address` computes for
    /// a type argument, exposed for callers that need a type observation
    /// outside an argument tuple (tests, expectation material).  The place is
    /// observation coordinate only: it selects which Val2 is read and never
    /// enters the normal form itself.
    pub fn canonical_complete_type_observation_address(
        &mut self,
        type_value: TypeValueId,
        place: Option<ObjectPlaceId>,
    ) -> Result<CanonicalValueAddr, crate::Diagnostic> {
        Ok(self.observe_complete_type(type_value, place)?.whole)
    }

    /// Default semantic equality observation of a type: `Core(tau) = Q`.
    /// This is ordinary Object normalization of the pure core and therefore
    /// remains distinct from both the opaque lookup index and the whole
    /// complete-type snapshot.
    pub fn canonical_type_core_observation_address(
        &mut self,
        type_value: TypeValueId,
        place: Option<ObjectPlaceId>,
    ) -> Result<CanonicalValueAddr, crate::Diagnostic> {
        let mut state = Val2NormState::default();
        let pattern = self
            .types
            .get(&type_value)
            .map(|t| t.pattern)
            .ok_or_else(|| {
                crate::Diagnostic::hard_error(
                "type observation has no registered Pattern and cannot form complete Object Norm",
                None,
            )
            })?;
        self.canonical_pure_type_address(place, pattern, &mut state)
    }

    /// Observe the registered Core of a type lookup key for ordinary Type
    /// equality. The lookup key locates the core; equality is the resulting
    /// canonical address, never the key itself.
    pub fn canonical_registered_type_core_observation_address(
        &mut self,
        type_value: TypeValueId,
    ) -> Result<CanonicalValueAddr, crate::Diagnostic> {
        let pattern = self
            .types
            .get(&type_value)
            .map(|ty| ty.pattern)
            .ok_or_else(|| {
                crate::Diagnostic::hard_error(
                    "type equality observation has no registered Pattern",
                    None,
                )
            })?;
        let place = self.pattern_places.get(&pattern).copied();
        self.canonical_type_core_observation_address(type_value, place)
    }

    /// Observe and intern the complete immutable closure
    /// `tau = bind alpha.<Q,V_tau>` at one type-valued read.
    ///
    /// `place` only selects the current core Object value.  It never enters
    /// identity.  The direct-TypeMember table is cloned before normalization,
    /// so later authorized contributions can only form another interned
    /// snapshot; they cannot mutate this returned value.
    pub fn observe_complete_type(
        &mut self,
        type_value: TypeValueId,
        place: Option<ObjectPlaceId>,
    ) -> Result<CompleteTypeValue, crate::Diagnostic> {
        let core = self.canonical_type_core_observation_address(type_value, place)?;
        let call_space = self
            .direct_type_members
            .get(&type_value)
            .cloned()
            .unwrap_or_default();
        let expected_home = self
            .types
            .get(&type_value)
            .and_then(|ty| self.patterns.get(&ty.pattern))
            .map(|pattern| pattern.root)
            .ok_or_else(|| {
                crate::Diagnostic::hard_error(
                    "complete type observation requires a registered core Pattern root",
                    None,
                )
            })?;
        let mut normalized: CanonicalTypeCallSpaceNorm = BTreeMap::new();
        for (selector, entries) in &call_space {
            let mut pure_p = None;
            let mut vals = Vec::new();
            for entry in entries {
                if entry.direct_home != expected_home {
                    return Err(crate::Diagnostic::hard_error(
                        "NoForeignTypeMemberInjection: a direct TypeMember's home does not match the observed core TypeMember scope",
                        None,
                    ));
                }
                let mut state = Val2NormState::default();
                let addr = self.canonical_member_value_address(entry.value, &mut state)?;
                match entry.facet {
                    TypeMemberFacet::PureP => {
                        if pure_p
                            .replace(addr)
                            .is_some_and(|existing| existing != addr)
                        {
                            return Err(crate::Diagnostic::hard_error(
                                "complete type callspace contains two different pure-P facets under one selector",
                                None,
                            ));
                        }
                    }
                    TypeMemberFacet::Value => vals.push(addr),
                }
            }
            let cluster = crate::CanonicalClusterNorm::new(pure_p, vals);
            if !cluster.is_empty() {
                normalized.insert(selector.clone(), cluster);
            }
        }
        let whole = self.intern_canonical_value(CanonicalNormForm::CompleteType(
            CanonicalCompleteTypeNorm {
                core,
                call_space: normalized,
            },
        ));
        let complete = CompleteTypeValue {
            lookup_key: type_value,
            core,
            call_space,
            whole,
        };
        self.complete_types
            .entry(whole)
            .or_insert_with(|| complete.clone());
        Ok(complete)
    }

    pub fn complete_type_by_whole_observation(
        &self,
        whole: CanonicalValueAddr,
    ) -> Option<&CompleteTypeValue> {
        self.complete_types.get(&whole)
    }

    /// Attach `Addr(Norm_type)` observations to the type arguments of an
    /// invocation at a world-connected boundary.
    ///
    /// Only `NonValue(CoreTypeProjection)` arguments receive an observation; other
    /// argument classes keep `known_type_observation = None` so their
    /// projections stay `Detached` (under-merge only).  Failure surfaces the
    /// same cyclic-Val2 diagnostics as canonical argument normalization.
    pub fn attach_canonical_type_observations(
        &mut self,
        raw_args: &mut [RawArgShape],
        _atoms: &[ProductAtom],
    ) -> Result<(), crate::Diagnostic> {
        for raw in raw_args.iter_mut() {
            if matches!(
                raw.value_class,
                RawArgValueClass::NonValue(NonValueArgKind::CoreTypeProjection)
            ) {
                let addr = self.canonical_type_core_observation_address(
                    raw.known_first_order_type_value.ok_or_else(|| {
                        crate::Diagnostic::hard_error(
                            "classified type argument has no core lookup key",
                            None,
                        )
                    })?,
                    raw.known_type_carrier_place,
                )?;
                raw.known_type_observation = Some(addr);
            }
        }
        Ok(())
    }

    /// Normalize a whole call-argument tuple into one canonical address.
    ///
    /// The invocation parentheses ARE a Product value, so the arguments of
    /// a meta invocation normalize through the ordinary Product normal
    /// form: `Addr(Product(a1..an))` with ordered member addresses.
    /// Top-level argument equivalence is position-sensitive by
    /// construction — it inherits the Product's positional identity
    /// instead of using an ad-hoc sequence encoding.
    pub fn canonical_arguments_product_address(
        &mut self,
        raw_args: &[RawArgShape],
        atoms: &[ProductAtom],
    ) -> Result<CanonicalValueAddr, crate::Diagnostic> {
        let members = raw_args
            .iter()
            .zip(atoms.iter())
            .map(|(raw, atom)| self.canonical_argument_address(raw, atom))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(
            self.intern_canonical_value(CanonicalNormForm::Object(CanonicalObjectNorm {
                val1: Some(CanonicalVal1Norm::Product { members }),
                pattern: CanonicalPatternNorm::Product {
                    constructor: CanonicalProductConstructor::CallParentheses,
                },
                val2: Default::default(),
            })),
        )
    }

    /// `Norm_P(P)` — the canonical normal form of one PatternValue.  A
    /// Pattern with recorded structural material normalizes
    /// by its normalized structural body (equal bodies allocated separately
    /// share one norm); a nominal declaration Pattern normalizes by its
    /// declaration root coordinate, which IS its normal form under the
    /// existing pattern-equivalence rules.
    pub fn canonical_pattern_norm(&self, pattern: PatternValueId) -> Option<CanonicalPatternNorm> {
        if let Some(value) = self.pattern_structural_norms.get(&pattern).cloned() {
            return Some(CanonicalPatternNorm::Structural { value });
        }
        self.patterns
            .get(&pattern)
            .map(|p| CanonicalPatternNorm::Nominal { root: p.root })
    }

    /// Proof of direct structural incidence derived exclusively from the
    /// Pattern's registered canonical structure.  Ordinary Val2 membership is
    /// deliberately not consulted, so a navigable helper cannot become a
    /// real field by accident.
    pub fn direct_pattern_child(
        &self,
        pattern: PatternValueId,
        selector: &crate::PatternSelector,
    ) -> Option<crate::DirectPatternChildEvidence> {
        crate::direct_pattern_child_from_canonical_value(
            pattern,
            self.pattern_structural_norms.get(&pattern)?,
            selector,
        )
    }

    /// Declaration-environment owner of one resolved call entry.
    ///
    /// Meta construction placement comes from the
    /// selected call entry's callable owner, never from a graph
    /// declaration Symbol: a `Callable` owner node yields its parent (the
    /// namespace or enclosing Self scope that declared it), while an
    /// entry rooted directly at a pattern owner (associated call entries)
    /// yields that owner itself.
    pub fn callable_declaration_environment(
        &self,
        call_entry: SemanticValueId,
    ) -> Option<SemanticOwnerId> {
        let SemanticValuePayload::CallEntry(entry) = &self.values.get(&call_entry)?.payload else {
            return None;
        };
        let owner = entry.callable_owner;
        match &self.owners.node(owner)?.kind {
            SemanticOwnerKind::Callable { .. } => Some(self.owners.parent(owner).unwrap_or(owner)),
            _ => Some(owner),
        }
    }

    pub fn owners(&self) -> &SemanticOwnerGraph {
        &self.owners
    }

    pub fn owner_namespace_graph(&self) -> &OwnerNamespaceGraph {
        &self.owner_namespaces_graph
    }

    pub fn package_owner(&self) -> SemanticOwnerId {
        self.package_owner
    }

    pub fn toolchain_owner(&self) -> SemanticOwnerId {
        self.toolchain_owner
    }

    pub fn bind_toolchain_root(&mut self, node: NamespaceNodeId) {
        self.namespace_owners.insert(node, self.toolchain_owner);
        self.owner_namespaces.insert(self.toolchain_owner, node);
        self.ensure_owner_namespace_node(node, None, "<root>");
    }

    pub fn bind_package_namespace(&mut self, node: NamespaceNodeId) {
        self.namespace_owners.insert(node, self.package_owner);
        self.owner_namespaces.insert(self.package_owner, node);
    }

    pub fn register_namespace(
        &mut self,
        node: NamespaceNodeId,
        parent: NamespaceNodeId,
        local_name: impl Into<String>,
    ) -> Option<SemanticOwnerId> {
        let local_name = local_name.into();
        if let Some(existing) = self.namespace_owners.get(&node).copied() {
            self.ensure_owner_namespace_node(node, Some(parent), local_name);
            return Some(existing);
        }
        let parent_owner = self.namespace_owners.get(&parent).copied()?;
        let owner = self.owners.namespace(parent_owner, local_name.clone());
        self.namespace_owners.insert(node, owner);
        self.owner_namespaces.insert(owner, node);
        self.ensure_owner_namespace_node(node, Some(parent), local_name);
        Some(owner)
    }

    fn ensure_owner_namespace_node(
        &mut self,
        node: NamespaceNodeId,
        parent: Option<NamespaceNodeId>,
        local_name: impl Into<String>,
    ) -> Option<OwnerNamespaceNodeId> {
        if let Some(existing) = self.owner_namespace_nodes.get(&node).copied() {
            return Some(existing);
        }
        let owner = self.namespace_owner(node)?;
        let parent_typed = match parent {
            Some(parent) => Some(*self.owner_namespace_nodes.get(&parent)?),
            None => None,
        };
        let package = self.owners.package_of(owner);
        let parent_package = parent.and_then(|parent| {
            self.namespace_owner(parent)
                .map(|parent_owner| self.owners.package_of(parent_owner))
        });
        let package_boundary =
            (parent.is_none() || parent_package != Some(package)).then_some(package);
        let typed = self.owner_namespaces_graph.add_node(
            owner,
            parent_typed,
            local_name,
            package_boundary,
            NamespaceVisibility::Public,
        );
        self.owner_namespace_nodes.insert(node, typed);
        self.projected_namespace_nodes.insert(typed, node);
        Some(typed)
    }

    /// Immediate enclosing namespace in the semantic owner tree.
    pub fn namespace_parent(&self, node: NamespaceNodeId) -> Option<NamespaceNodeId> {
        let typed = self.owner_namespace_nodes.get(&node)?;
        let parent = self.owner_namespaces_graph.node(*typed)?.parent?;
        self.projected_namespace_nodes.get(&parent).copied()
    }

    /// Bare-name lookup order: current namespace, every semantic enclosing
    /// namespace, then default mounts such as core. Duplicates are removed
    /// without changing nearest-first order.
    pub fn bare_name_scope_chain(
        &self,
        start: NamespaceNodeId,
        default_mounts: &[NamespaceNodeId],
    ) -> Vec<NamespaceNodeId> {
        let mut scopes = Vec::new();
        let mut current = Some(start);
        while let Some(scope) = current {
            if scopes.contains(&scope) {
                break;
            }
            scopes.push(scope);
            let is_package_boundary = self
                .owner_namespace_nodes
                .get(&scope)
                .and_then(|node| self.owner_namespaces_graph.node(*node))
                .is_some_and(|node| node.package_boundary.is_some());
            if is_package_boundary {
                break;
            }
            current = self.namespace_parent(scope);
        }
        for mount in default_mounts {
            if !scopes.contains(mount) {
                scopes.push(*mount);
            }
        }
        scopes
    }

    pub fn child_namespace(&self, parent: NamespaceNodeId, name: &str) -> Option<NamespaceNodeId> {
        let typed_parent = self.owner_namespace_nodes.get(&parent).copied()?;
        let child = self.owner_namespaces_graph.child(typed_parent, name)?;
        self.projected_namespace_nodes.get(&child).copied()
    }

    /// Resolve an inner-to-outer path to its namespace facet through typed
    /// namespace topology and namespace-capable semantic Symbols.  This is
    /// the namespace-role companion of `resolve_symbol_path`; graph identity
    /// selection is completed before any projection is rendered.
    pub fn resolve_namespace_path(
        &self,
        path: &[String],
        start: NamespaceNodeId,
        explicit_roots: &[NamespaceNodeId],
        default_mounts: &[NamespaceNodeId],
    ) -> Result<NamespaceNodeId, crate::Diagnostic> {
        if path.is_empty() {
            return Err(crate::Diagnostic::hard_error(
                "unresolved empty namespace path",
                None,
            ));
        }
        let mut roots = vec![start];
        for root in explicit_roots.iter().chain(default_mounts.iter()) {
            if !roots.contains(root) {
                roots.push(*root);
            }
        }
        let mut hits = Vec::new();
        for root in roots {
            let mut cursor = Some(root);
            for component in path.iter().rev() {
                cursor = cursor.and_then(|node| self.namespace_capable_child(node, component));
            }
            if let Some(hit) = cursor {
                if !hits.contains(&hit) {
                    hits.push(hit);
                }
            }
        }
        match hits.as_slice() {
            [one] => Ok(*one),
            [] => Err(crate::Diagnostic::hard_error(
                format!("resolver error: unresolved namespace `{}`", path.join("::")),
                None,
            )),
            _ => Err(crate::Diagnostic::hard_error(
                format!("resolver error: ambiguous namespace `{}`", path.join("::")),
                None,
            )),
        }
    }

    /// One namespace-capable navigation step: a semantic namespace child
    /// wins first; otherwise a type carrier whose Pattern owns an
    /// associated namespace is namespace-capable.
    fn namespace_capable_child(
        &self,
        parent: NamespaceNodeId,
        name: &str,
    ) -> Option<NamespaceNodeId> {
        if let Some(child) = self.child_namespace(parent, name) {
            return Some(child);
        }
        let pattern = self.symbol_in_namespace(parent, name)?.pure_p_pattern()?;
        self.associated_namespace_for_pattern(pattern)
    }

    /// The associated namespace node owned by one Pattern, if it has one.
    pub fn associated_namespace_for_pattern(
        &self,
        pattern: PatternValueId,
    ) -> Option<NamespaceNodeId> {
        self.associated_namespace_patterns
            .iter()
            .find(|(_, owner)| **owner == pattern)
            .map(|(namespace, _)| *namespace)
    }

    /// The Symbol named by `name` at one navigation cursor.
    ///
    /// Symbol-first, object-before-namespace: when the cursor stands on an
    /// object, that object's own Val2 answers the name (`Val2(T_t)[f] = C_f`,
    /// read through the carrier's own place with per-name inheritance from the
    /// Pattern's canonical pure type Object).  The namespace side of the cursor is
    /// the fallback for names that live in a declaration namespace instead of
    /// an object's Val2.
    fn cursor_symbol(
        &self,
        cursor: &SemanticOuterScope,
        name: &str,
    ) -> Option<SemanticSymbolIdentity> {
        if let Some(host) = &cursor.host {
            if let Some(symbol) = self.associated_symbol_for_host(host, name) {
                return Some(symbol);
            }
        }
        self.symbol_in_namespace(cursor.namespace?, name)
            .map(|cell| cell.identity)
    }

    /// One step of the single recursive Symbol navigation.
    ///
    /// ```text
    /// Path -> Symbol -> ContextDirectedProjection
    /// ```
    ///
    /// A step resolves `component` to a Symbol at the current cursor, selects
    /// that Symbol's object facet, and carries the object's own Val2 place
    /// forward as the next cursor's host.  The namespace side travels along
    /// unchanged so pure declaration navigation (`Vec::std`) keeps working
    /// through the same algorithm.  Which facet the *terminal* Symbol is read
    /// through is decided by the use context afterwards, never here.
    fn navigate_step(
        &self,
        cursor: &SemanticOuterScope,
        component: &str,
    ) -> Option<SemanticOuterScope> {
        let host = self
            .cursor_symbol(cursor, component)
            .and_then(|symbol| self.host_member_of(symbol));
        let namespace = cursor
            .namespace
            .and_then(|namespace| self.namespace_capable_child(namespace, component))
            .or_else(|| {
                host.as_ref()
                    .and_then(|host| self.associated_namespace_for_pattern(host.pattern))
            });
        if host.is_none() && namespace.is_none() {
            return None;
        }
        Some(SemanticOuterScope { host, namespace })
    }

    /// Walk one complete inner-to-outer path from a single root.
    ///
    /// The outer components are navigation steps; the innermost component is
    /// the terminal Symbol query.
    fn navigate_path_from(
        &self,
        path: &[String],
        start: NamespaceNodeId,
    ) -> Option<ResolvedSemanticNavigation> {
        let (inner, outers) = path.split_first()?;
        let mut cursor = SemanticOuterScope {
            host: None,
            namespace: Some(start),
        };
        let mut host_chain = Vec::new();
        for component in outers.iter().rev() {
            cursor = self.navigate_step(&cursor, component)?;
            if let Some(host) = &cursor.host {
                host_chain.push(host.clone());
            }
        }
        Some(ResolvedSemanticNavigation {
            host_chain,
            terminal_symbol: self.cursor_symbol(&cursor, inner)?,
        })
    }

    fn symbol_path_from(
        &self,
        path: &[String],
        start: NamespaceNodeId,
    ) -> Option<SemanticSymbolIdentity> {
        self.navigate_path_from(path, start)
            .map(|navigation| navigation.terminal_symbol)
    }

    /// Cell-level exact path walk shared by identity resolution and
    /// extraction-target resolution (which also needs the cell's pure P).
    fn symbol_cell_path_from(
        &self,
        path: &[String],
        start: NamespaceNodeId,
    ) -> Option<&SemanticSymbolCell> {
        self.symbol(self.symbol_path_from(path, start)?)
    }

    /// Resolve one already-completed navigation from exactly one namespace
    /// root.  No bare-name scope chain or default mount fallback is applied.
    pub fn resolve_symbol_path_exact(
        &self,
        path: &[String],
        root: NamespaceNodeId,
    ) -> Option<SemanticSymbolIdentity> {
        self.symbol_path_from(path, root)
    }

    pub fn global_namespace(&self) -> Option<NamespaceNodeId> {
        self.owner_namespaces.get(&self.toolchain_owner).copied()
    }

    /// Resolve one extraction subject to its target
    /// Pattern through the real navigation-completion + exact-resolution
    /// chain:
    ///
    /// ```text
    /// extraction tree position
    /// -> expand_extraction_navigation (nearest explicit anchor /
    ///    implicit global top)
    /// -> completed full Symbol path
    /// -> exact resolution from the global namespace root
    /// -> the Symbol's PatternValue
    /// -> canonical pattern norm
    /// ```
    ///
    /// The completed path is exact: it never re-enters the bare-name scope
    /// chain or default-mount fallback, so a symbol that would be visible
    /// by bare-name lookup is NOT a hit unless its full global path equals
    /// the completed navigation.
    pub fn resolve_extraction_target(
        &self,
        subject_local_navigation: &CanonicalFullNavigation,
        explicit_navigation: Option<&CanonicalFullNavigation>,
        parents_nearest_first: &[ExtractionPatternParent],
        provenance: Provenance,
    ) -> Result<ResolvedExtractionTarget, crate::Diagnostic> {
        let completed = expand_extraction_navigation(
            subject_local_navigation,
            explicit_navigation,
            parents_nearest_first,
        )
        .map_err(|_| {
            crate::Diagnostic::hard_error(
                "extraction navigation has no anchor: no enclosing pattern layer \
                 carries an explicit or implicit-global own navigation",
                Some(provenance.clone()),
            )
        })?;
        let root = self.global_namespace().ok_or_else(|| {
            crate::Diagnostic::hard_error(
                "extraction resolution requires the global namespace root, which is missing",
                Some(provenance.clone()),
            )
        })?;
        let cell = self
            .symbol_cell_path_from(completed.components(), root)
            .ok_or_else(|| {
                crate::Diagnostic::hard_error(
                    format!(
                        "unresolved extraction path `{}`: a completed extraction navigation \
                         is exact and never falls back to bare-name lookup",
                        completed.components().join("::"),
                    ),
                    Some(provenance.clone()),
                )
            })?;
        let pattern = cell.pure_p_pattern().ok_or_else(|| {
            crate::Diagnostic::hard_error(
                format!(
                    "extraction path `{}` resolved to a symbol without a pure-P \
                     PatternValue; extraction targets must denote a Pattern",
                    completed.components().join("::"),
                ),
                Some(provenance.clone()),
            )
        })?;
        let norm = self.canonical_pattern_norm(pattern).ok_or_else(|| {
            crate::Diagnostic::hard_error(
                format!(
                    "extraction path `{}` resolved to a Pattern without a canonical norm",
                    completed.components().join("::"),
                ),
                Some(provenance),
            )
        })?;
        Ok(ResolvedExtractionTarget {
            symbol: cell.identity,
            pattern,
            norm,
        })
    }

    /// Canonical pattern comparison used by the
    /// extraction matcher: two Patterns match exactly when their canonical
    /// pattern norms are equal.  `None` when either Pattern is unknown.
    pub fn extraction_pattern_matches(
        &self,
        target: PatternValueId,
        subject: PatternValueId,
    ) -> Option<bool> {
        Some(self.canonical_pattern_norm(target)? == self.canonical_pattern_norm(subject)?)
    }

    /// Semantic path→Symbol resolution.
    ///
    /// This is the terminal-Symbol projection of
    /// [`Self::navigate_semantic_path`]: every use context resolves through
    /// the one recursive Symbol navigation and only then projects the facet it
    /// needs.
    pub fn resolve_symbol_path(
        &self,
        path: &[String],
        start: NamespaceNodeId,
        explicit_roots: &[NamespaceNodeId],
        default_mounts: &[NamespaceNodeId],
    ) -> Result<SemanticSymbolIdentity, crate::Diagnostic> {
        self.navigate_semantic_path(path, start, explicit_roots, default_mounts)
            .map(|navigation| navigation.terminal_symbol)
    }

    /// The single recursive Symbol navigation, shared by every use context.
    ///
    /// A bare name follows the semantic scope chain in nearest-first order:
    /// current namespace, every enclosing namespace, then default mounts such
    /// as core.  The first hit wins.  A navigated path denotes one explicit
    /// Symbol query; its candidate roots are checked for one distinct hit and
    /// are never reinterpreted as a sequence of bare-name fallbacks.
    ///
    /// Each outer component steps through the named Symbol's own object facet
    /// and that object's own Val2 place, so `f::T` denotes `Val2(T)[f]`
    /// regardless of whether the result is later used as a call target, a
    /// type, a value, or an injection RHS.
    pub fn navigate_semantic_path(
        &self,
        path: &[String],
        start: NamespaceNodeId,
        explicit_roots: &[NamespaceNodeId],
        default_mounts: &[NamespaceNodeId],
    ) -> Result<ResolvedSemanticNavigation, crate::Diagnostic> {
        if path.is_empty() {
            return Err(crate::Diagnostic::hard_error(
                "unresolved empty namespace path",
                None,
            ));
        }
        if path.len() == 1 {
            let name = &path[0];
            for scope in self.bare_name_scope_chain(start, default_mounts) {
                if let Some(symbol) = self.symbol_in_namespace(scope, name) {
                    return Ok(ResolvedSemanticNavigation {
                        host_chain: Vec::new(),
                        terminal_symbol: symbol.identity,
                    });
                }
            }
            return Err(crate::Diagnostic::hard_error(
                format!("resolver error: unresolved symbol `{name}`"),
                None,
            ));
        }
        let mut roots = vec![start];
        for root in explicit_roots {
            if !roots.contains(root) {
                roots.push(*root);
            }
        }
        let mut hits: Vec<ResolvedSemanticNavigation> = Vec::new();
        for root in roots {
            if let Some(hit) = self.navigate_path_from(path, root) {
                // Dedup on the WHOLE navigation, not just the terminal Symbol:
                // `ResolvedNavigation = ⟨HostChain, TerminalSymbol⟩` and the
                // host chain participates in exposure, so two roots that reach
                // the same terminal through different host chains are NOT the
                // same navigation.  Collapsing them by terminal alone would
                // make the surviving path depend on search-root order; keeping
                // both makes the disagreement a reported ambiguity below.
                if !hits.iter().any(|existing| existing == &hit) {
                    hits.push(hit);
                }
            }
        }
        match hits.as_slice() {
            [one] => Ok(one.clone()),
            [] => Err(crate::Diagnostic::hard_error(
                format!("resolver error: unresolved symbol `{}`", path.join("::")),
                None,
            )),
            _ => Err(crate::Diagnostic::hard_error(
                format!(
                    "resolver error: ambiguous navigation `{}`: distinct host chains reach it across resolver search roots",
                    path.join("::")
                ),
                None,
            )),
        }
    }

    /// Resolve the outer components of an explicit
    /// navigation to the scope material they select: the type Pattern
    /// owning the innermost outer component (associated-member scope)
    /// and/or the namespace child of the same spelling.
    pub fn resolve_outer_scope(
        &self,
        outer_path: &[String],
        start: NamespaceNodeId,
        explicit_roots: &[NamespaceNodeId],
        default_mounts: &[NamespaceNodeId],
    ) -> Option<SemanticOuterScope> {
        let mut roots = vec![start];
        for root in explicit_roots {
            if !roots.contains(root) {
                roots.push(*root);
            }
        }
        if outer_path.len() == 1 {
            for mount in default_mounts {
                if !roots.contains(mount) {
                    roots.push(*mount);
                }
            }
        }
        let mut hits: Vec<SemanticOuterScope> = Vec::new();
        for root in roots {
            if let Some(hit) = self.outer_scope_from(outer_path, root) {
                if !hits.contains(&hit) {
                    hits.push(hit);
                }
            }
        }
        match hits.as_slice() {
            [one] => Some(one.clone()),
            _ => None,
        }
    }

    fn outer_scope_from(
        &self,
        outer_path: &[String],
        start: NamespaceNodeId,
    ) -> Option<SemanticOuterScope> {
        let mut cursor = SemanticOuterScope {
            host: None,
            namespace: Some(start),
        };
        for component in outer_path.iter().rev() {
            cursor = self.navigate_step(&cursor, component)?;
        }
        Some(cursor)
    }

    /// The host layer named by one carrier Symbol.
    ///
    /// The carrier's own pure-P member is the host object: its place is that
    /// object's Val2 container and its member view is the binding-level
    /// Policy authority.  Two carriers of one Pattern (`let T: type = uint8;
    /// let U: type = T;`) therefore produce different hosts even though
    /// `Pattern(T) = Pattern(U)`.
    fn host_member_for_symbol(&self, cell: &SemanticSymbolCell) -> Option<PatternHostMember> {
        let member = cell.pure_p?;
        Some(PatternHostMember {
            symbol: Some(cell.identity),
            pattern: member.pattern,
            place: member.place,
            complete_type: member.complete_type,
            view: cell.pure_p_view().cloned(),
        })
    }

    /// [`Self::host_member_for_symbol`] addressed by Symbol identity.
    pub fn host_member_of(&self, symbol: SemanticSymbolIdentity) -> Option<PatternHostMember> {
        self.host_member_for_symbol(self.symbols.get(&symbol)?)
    }

    /// The compiler-internal host layer of a bare Pattern.
    ///
    /// No carrier Symbol named this step, so there is no binding-level view
    /// to compose: the host factor of the exposure conjunction is vacuous and
    /// Val2 lookup lands on the Pattern's canonical type-object place.
    pub fn host_member_for_pattern(&self, pattern: PatternValueId) -> Option<PatternHostMember> {
        Some(PatternHostMember {
            symbol: None,
            pattern,
            place: self.pattern_place(pattern)?,
            complete_type: None,
            view: None,
        })
    }

    pub fn namespace_owner(&self, node: NamespaceNodeId) -> Option<SemanticOwnerId> {
        self.namespace_owners.get(&node).copied()
    }

    pub fn namespace_is_toolchain_owned(&self, node: NamespaceNodeId) -> bool {
        self.namespace_owner(node).is_some_and(|owner| {
            self.owners.package_of(owner) == self.owners.package_of(self.toolchain_owner)
        })
    }

    pub fn symbol(&self, identity: SemanticSymbolIdentity) -> Option<&SemanticSymbolCell> {
        self.symbols.get(&identity)
    }

    /// Binding-local destination Place for one value member.
    ///
    /// A value can be resident in multiple bindings.  Consequently this
    /// relation is keyed by Symbol and value; it is not recoverable from the
    /// value's formation place.
    pub fn binding_place(
        &self,
        symbol: SemanticSymbolIdentity,
        value: SemanticValueId,
    ) -> Option<ObjectPlaceId> {
        self.symbol(symbol)?.sibling_place(value)
    }

    pub fn binding_places(
        &self,
        symbol: SemanticSymbolIdentity,
    ) -> BTreeMap<SemanticValueId, ObjectPlaceId> {
        self.symbol(symbol)
            .map(|cell| cell.sibling_places.clone())
            .unwrap_or_default()
    }

    pub fn symbol_in_namespace(
        &self,
        namespace: NamespaceNodeId,
        name: &str,
    ) -> Option<&SemanticSymbolCell> {
        let node = self.owner_namespace_nodes.get(&namespace).copied()?;
        let entries = self.owner_namespaces_graph.symbol_entries(node, name)?;
        let [entry] = entries else {
            return None;
        };
        let identity = entry.identity;
        self.symbol(identity)
    }

    pub fn sibling_value(
        &self,
        symbol: SemanticSymbolIdentity,
        index: usize,
    ) -> Option<SemanticValueId> {
        self.symbol(symbol)
            .and_then(|cell| cell.sibling_vals.get(index).copied())
    }

    pub fn core_type_projection_value_for_symbol(
        &self,
        identity: SemanticSymbolIdentity,
    ) -> Option<SemanticValueId> {
        let cell = self.symbol(identity)?;
        let pattern = cell.pure_p_pattern()?;
        let type_value = self.type_for_pattern(pattern)?;
        self.core_type_projection_value(type_value)
    }

    pub fn value(&self, id: SemanticValueId) -> Option<&SemanticValueObject> {
        self.values.get(&id)
    }

    /// Install candidate-local capability realization metadata on an already
    /// materialized terminal call entry. Policy mode is deliberately absent
    /// from the authorization rule: the table is an independent 3x3 fact.
    pub fn configure_call_entry_capability_realization(
        &mut self,
        id: SemanticValueId,
        realization: crate::CapabilityRealization,
    ) -> Result<(), crate::Diagnostic> {
        let value = self.values.get_mut(&id).ok_or_else(|| {
            crate::Diagnostic::hard_error(
                "capability realization target is not an installed semantic value",
                None,
            )
        })?;
        let SemanticValuePayload::CallEntry(entry) = &mut value.payload else {
            return Err(crate::Diagnostic::hard_error(
                "capability realization may be configured only on a terminal call entry",
                Some(value.provenance.clone()),
            ));
        };
        entry.capability_realization = realization;
        Ok(())
    }

    pub fn values(&self) -> impl Iterator<Item = &SemanticValueObject> {
        self.values.values()
    }

    /// Install one real `Val1 × P × Val2` object.
    ///
    /// This is the single write boundary for semantic values whose Val1 is
    /// present.  If the value's type Pattern is still owned by an open
    /// construction, materializing the Val1 performs `UseForVal1` before the
    /// value becomes observable.  Type-object adapters deliberately bypass
    /// this helper: they transport pure `null × P × Val2` material and are
    /// not semantic Val1 residents.
    fn materialize_val1_object(&mut self, mut object: SemanticValueObject) -> SemanticValueId {
        // Each materialized value gets its own per-object Val2 place.
        object.place = self.allocate_object_place();
        let id = object.id;
        if let Some(type_pattern) = self.types.get(&object.type_value).map(|ty| ty.pattern) {
            debug_assert_eq!(
                type_pattern, object.pattern,
                "a materialized Val1 must carry the Pattern of its TypeValue"
            );
            if let Some(PatternClusterOwner::Open(cluster)) =
                self.pattern_clusters.get(&type_pattern).copied()
            {
                let construction = self
                    .open_clusters
                    .get_mut(&cluster)
                    .expect("an open Pattern owner must name a live construction");
                assert_ne!(
                    construction.state,
                    ConstructionState::Finalized,
                    "a finalized construction cannot materialize a new Val1"
                );
                construction.use_observation.has_been_used_for_val1 = true;
                construction.state = ConstructionState::Frozen;
            }
        }
        if let Ok(complete) = self.observe_complete_type(object.type_value, Some(object.place)) {
            self.value_complete_types.insert(id, complete.whole);
        }
        let replaced = self.values.insert(id, object);
        debug_assert!(
            replaced.is_none(),
            "semantic value ids are single-assignment"
        );
        id
    }

    /// `CallSpace(Type(v))["()"]` from the exact snapshot captured when `v`
    /// was formed. No Object.Val2 fallback and no lookup-key refresh is
    /// permitted here.
    pub fn callable_entries_for_value(&self, value: SemanticValueId) -> Vec<SemanticValueId> {
        self.value_complete_types
            .get(&value)
            .and_then(|whole| self.complete_types.get(whole))
            .and_then(|complete| complete.call_space.get("()"))
            .map(|entries| entries.iter().map(|entry| entry.value).collect())
            .unwrap_or_default()
    }

    /// Complete formation of a value whose Type callspace was assembled after
    /// the Val1 carrier itself was allocated. Ordinary later TypeMember
    /// contributions never call this, so they cannot retarget an existing
    /// value to a successor tau snapshot.
    fn freeze_value_complete_type(&mut self, value: SemanticValueId) {
        let Some(object) = self.values.get(&value) else {
            return;
        };
        let type_value = object.type_value;
        let place = object.place;
        if let Ok(complete) = self.observe_complete_type(type_value, Some(place)) {
            self.value_complete_types.insert(value, complete.whole);
        }
    }

    pub fn type_value(&self, id: TypeValueId) -> Option<&SemanticTypeValue> {
        self.types.get(&id)
    }

    pub fn type_rank(&self) -> Option<TypeValueId> {
        self.type_rank
    }

    pub fn symbol_rank(&self) -> Option<TypeValueId> {
        self.symbol_rank
    }

    pub fn core_type_projection_value(
        &self,
        represented_type: TypeValueId,
    ) -> Option<SemanticValueId> {
        self.core_type_projection_values
            .get(&represented_type)
            .copied()
    }

    /// The transport-default pure-P member view of a Pattern.
    ///
    /// This is NOT a binding-level Policy authority. The Policy components
    /// come from the Pattern's shared CoreTypeProjection adapter, which is interned
    /// per `TypeValueId` and therefore identical for every carrier of the same
    /// type: `P_a let T: type = X;` and `P_b let U: type = X;` share one
    /// adapter but have different pure-P member views. A caller that has a
    /// carrier Symbol or a bound formal parameter must read that binding's own
    /// member view; this helper only supplies the ontological default when no
    /// binding view exists at all (compiler-internal transport of a Pattern
    /// with no naming carrier). When the adapter records no Policy the view
    /// carries the empty (absent-value) Policy.
    pub fn transport_pure_p_view(
        &self,
        pattern: PatternValueId,
    ) -> PolicyResultEntry<SemanticValueId, PatternValueId> {
        let recorded = self
            .pattern_types
            .get(&pattern)
            .and_then(|type_value| self.core_type_projection_values.get(type_value))
            .and_then(|value| self.values.get(value))
            .map(|object| object.policy.clone());
        match recorded {
            Some(policy) => PolicyResultEntry {
                value: None,
                pattern,
                view: PolicyView {
                    pair: policy,
                    mode: PolicyMode::Plain,
                },
            },
            None => PolicyResultEntry {
                value: None,
                pattern,
                view: PolicyView {
                    pair: PolicyPair {
                        value: crate::ValueComponentPolicy {
                            stages: crate::StageSet::new(),
                            presence: crate::ValuePresence::Absent,
                        },
                        pattern: crate::PatternComponentPolicy {
                            stages: crate::StageSet::new(),
                        },
                    },
                    mode: PolicyMode::Plain,
                },
            },
        }
    }

    pub fn type_for_pattern(&self, pattern: PatternValueId) -> Option<TypeValueId> {
        self.pattern_types.get(&pattern).copied()
    }

    /// Admit one direct TypeMember under an explicitly supplied classifier
    /// home.  The home equality check is the semantic gate; callers cannot
    /// copy a foreign member into another type's `V_tau` and cannot change a
    /// member's home after creation.
    pub fn admit_direct_type_member(
        &mut self,
        target_pattern: PatternValueId,
        direct_home_pattern: PatternValueId,
        selector: impl Into<String>,
        facet: TypeMemberFacet,
        value: SemanticValueId,
    ) -> Result<(), crate::Diagnostic> {
        let target = self.pattern(target_pattern).ok_or_else(|| {
            crate::Diagnostic::hard_error(
                "direct TypeMember target Pattern is not registered",
                None,
            )
        })?;
        let expected_home = target.root;
        let supplied_home = self
            .pattern(direct_home_pattern)
            .map(|pattern| pattern.root)
            .ok_or_else(|| {
                crate::Diagnostic::hard_error(
                    "direct TypeMember classifier home Pattern is not registered",
                    None,
                )
            })?;
        if expected_home != supplied_home {
            return Err(crate::Diagnostic::hard_error(
                "NoForeignTypeMemberInjection: classifier direct home differs from the target TypeMember scope",
                None,
            ));
        }
        if !self.values.contains_key(&value) {
            return Err(crate::Diagnostic::hard_error(
                "direct TypeMember value is not installed",
                None,
            ));
        }
        let type_value = self.type_for_pattern(target_pattern).ok_or_else(|| {
            crate::Diagnostic::hard_error(
                "direct TypeMember target Pattern has no core lookup entry",
                None,
            )
        })?;
        let entry = TypeMemberSnapshotEntry {
            direct_home: supplied_home,
            facet,
            value,
        };
        let entries = self
            .direct_type_members
            .entry(type_value)
            .or_default()
            .entry(selector.into())
            .or_default();
        if !entries.contains(&entry) {
            if facet == TypeMemberFacet::PureP
                && entries
                    .iter()
                    .any(|entry| entry.facet == TypeMemberFacet::PureP && entry.value != value)
            {
                return Err(crate::Diagnostic::hard_error(
                    "direct TypeMember selector already carries a different pure-P facet",
                    None,
                ));
            }
            entries.push(entry);
            entries.sort();
        }
        self.refresh_declaring_type_carrier_snapshot(target_pattern)?;
        Ok(())
    }

    /// A direct TypeMember contribution during formation produces a successor
    /// complete type value for the declaring carrier only. Ordinary copied
    /// carriers are deliberately not scanned or rewritten.
    fn refresh_declaring_type_carrier_snapshot(
        &mut self,
        target_pattern: PatternValueId,
    ) -> Result<(), crate::Diagnostic> {
        let Some(PatternClusterOwner::Installed(owner)) = self.owner_cluster(target_pattern) else {
            return Ok(());
        };
        let Some(member) = self.symbol(owner).and_then(|cell| cell.pure_p) else {
            return Ok(());
        };
        if member.pattern != target_pattern {
            return Ok(());
        }
        let type_value = self.type_for_pattern(target_pattern).ok_or_else(|| {
            crate::Diagnostic::hard_error(
                "declaring type carrier Pattern has no core lookup entry",
                None,
            )
        })?;
        let whole = self
            .observe_complete_type(type_value, Some(member.place))?
            .whole;
        self.symbols
            .get_mut(&owner)
            .expect("declaring carrier still exists")
            .pure_p
            .as_mut()
            .expect("declaring carrier still has its pure-P member")
            .complete_type = Some(whole);
        Ok(())
    }

    pub fn direct_type_members(&self, type_value: TypeValueId) -> Option<&ImmutableTypeCallSpace> {
        self.direct_type_members.get(&type_value)
    }

    pub fn contains_type_value(&self, id: TypeValueId) -> bool {
        self.types.contains_key(&id)
    }

    pub fn contains_type_binding(&self, symbol: SymbolId) -> bool {
        self.registered_type_bindings.contains(&symbol)
    }

    pub fn pattern(&self, id: PatternValueId) -> Option<&SemanticPatternValue> {
        self.patterns.get(&id)
    }

    pub fn pattern_scope(&self, id: ResolvedPatternScopeId) -> Option<&ResolvedPatternScope> {
        self.scopes.get(&id)
    }

    /// Follow a PatternValue to its canonical owning ClusterSymbol.
    ///
    /// This is a forward-only mapping recorded when the PatternValue is first
    /// created as an owning pure-P member. Carrier rebinding (`let T: type = X`)
    /// does not rewrite this entry.
    pub fn owner_cluster(&self, pattern: PatternValueId) -> Option<PatternClusterOwner> {
        self.pattern_clusters.get(&pattern).copied()
    }

    pub fn pattern_owner(&self, pattern: PatternValueId) -> Option<&ResolvedPatternScope> {
        let scope = self.pattern(pattern)?.scope;
        self.pattern_scope(scope)
    }

    /// Access the per-object Val2 place by its handle.
    pub fn object_place(&self, id: ObjectPlaceId) -> Option<&ObjectPlace> {
        self.places.get(&id)
    }

    /// Mutable access to the per-object Val2 place by its handle.
    pub fn object_place_mut(&mut self, id: ObjectPlaceId) -> Option<&mut ObjectPlace> {
        self.places.get_mut(&id)
    }

    /// Read the frozen owned Val2 snapshot used by ordinary Object Norm.
    /// This is deliberately separate from `object_place`, whose maps are
    /// navigation/storage substrate and may participate in inherited lookup.
    pub fn semantic_val2_snapshot(&self, id: ObjectPlaceId) -> Option<&SemanticVal2Snapshot> {
        self.semantic_val2_snapshots.get(&id)
    }

    /// Commit the current *owned* place material as one semantic Val2
    /// snapshot.  No Pattern place or lookup fallback is consulted here.
    fn refresh_semantic_val2_snapshot(&mut self, id: ObjectPlaceId) -> Option<()> {
        let place = self.places.get(&id)?.clone();
        let mut names = BTreeSet::new();
        names.extend(place.associated_symbols.keys().cloned());
        names.extend(place.associated_val2.keys().cloned());
        let mut snapshot = self
            .semantic_val2_snapshots
            .get(&id)
            .cloned()
            .unwrap_or_default();
        for name in names {
            let cluster = place
                .associated_symbols
                .get(&name)
                .and_then(|symbol| self.symbols.get(symbol))
                .map(|cell| SemanticVal2ClusterSnapshot {
                    pure_p: cell.pure_p,
                    values: cell.sibling_vals.clone(),
                })
                .unwrap_or_else(|| SemanticVal2ClusterSnapshot {
                    pure_p: None,
                    values: place
                        .associated_val2
                        .get(&name)
                        .cloned()
                        .unwrap_or_default(),
                });
            if cluster.pure_p.is_some() || !cluster.values.is_empty() {
                snapshot.clusters.insert(name, cluster);
            }
        }
        self.semantic_val2_snapshots.insert(id, snapshot);
        Some(())
    }

    /// Get the canonical type-object place for a Pattern.
    pub fn pattern_place(&self, pattern: PatternValueId) -> Option<ObjectPlaceId> {
        self.pattern_places.get(&pattern).copied()
    }

    pub fn pattern_for_associated_namespace(
        &self,
        namespace: NamespaceNodeId,
    ) -> Option<PatternValueId> {
        self.associated_namespace_patterns.get(&namespace).copied()
    }

    /// Follow the canonical forward path from a semantic value's own
    /// per-object Val2 place. Each value has an independent place even if
    /// it shares a Pattern with other values.
    pub fn associated_values_for_value(
        &self,
        value: SemanticValueId,
        name: &str,
    ) -> Option<&[SemanticValueId]> {
        let value = self.value(value)?;
        // First check the value's own per-object place.
        if let Some(entries) = self
            .places
            .get(&value.place)
            .and_then(|p| p.associated_val2.get(name))
        {
            if !entries.is_empty() {
                return Some(entries.as_slice());
            }
        }
        // Fall back to the pattern's canonical type-level place.
        let place_id = self.pattern_places.get(&value.pattern)?;
        self.places
            .get(place_id)?
            .associated_val2
            .get(name)
            .map(Vec::as_slice)
    }

    /// Look up Val2 for the canonical pure type Object of a Pattern.
    ///
    /// This is the type-level Val2: the values navigable from the type itself
    /// (the `null × P × Val2` object that owns this Pattern).  For per-value
    /// Val2, use [`Self::associated_values_for_value`] instead; for the Val2 of
    /// one specific type carrier, use [`Self::associated_values_in_place`] with
    /// that carrier's own place.
    pub fn associated_values_for_pattern(
        &self,
        pattern: PatternValueId,
        name: &str,
    ) -> Option<&[SemanticValueId]> {
        let place_id = self.pattern_places.get(&pattern)?;
        self.places
            .get(place_id)?
            .associated_val2
            .get(name)
            .map(Vec::as_slice)
    }

    /// Transport Val2 entries of one specific object place.
    pub fn associated_values_in_place(
        &self,
        place: ObjectPlaceId,
        name: &str,
    ) -> Option<&[SemanticValueId]> {
        self.places
            .get(&place)?
            .associated_val2
            .get(name)
            .map(Vec::as_slice)
    }

    pub fn resident_generation(&self, place: ObjectPlaceId) -> Option<ResidentGeneration> {
        self.places.get(&place).map(|place| place.resident)
    }

    /// Resolve a reusable logical navigation coordinate against the current
    /// resident.  The final slot may be missing; that prospective slot still
    /// has stable formation-time identity.
    pub fn projection_slot(
        &self,
        place: ObjectPlaceId,
        selector: ProjectionSelector,
    ) -> Option<ProjectionSlot> {
        let resident = self.places.get(&place)?;
        let key = projection_storage_key(&selector);
        let occupied = resident.associated_symbols.contains_key(&key)
            || resident.associated_val2.contains_key(&key)
            || self
                .semantic_val2_snapshots
                .get(&place)
                .is_some_and(|snapshot| snapshot.clusters.contains_key(&key));
        Some(ProjectionSlot {
            identity: ProjectionSlotIdentity {
                parent: resident.resident,
                selector,
            },
            contents: if occupied {
                ProjectionSlotContents::Occupied
            } else {
                ProjectionSlotContents::Missing
            },
        })
    }

    pub fn stable_place_target(&self, place: ObjectPlaceId) -> Option<StableBorrowTarget> {
        Some(StableBorrowTarget::Place {
            place,
            resident: self.resident_generation(place)?,
        })
    }

    pub fn stable_projection_target(
        &self,
        place: ObjectPlaceId,
        selector: ProjectionSelector,
    ) -> Option<StableBorrowTarget> {
        Some(StableBorrowTarget::Projection(
            self.projection_slot(place, selector)?.identity,
        ))
    }

    pub fn borrow_view(&self, borrow: BorrowViewId) -> Option<&BorrowView> {
        self.borrows.get(&borrow)
    }

    /// Explicit `ref` formation.  This is the only entry point that creates a
    /// ref view; candidate adaptation and Policy projection have no access to
    /// it (`NoImplicitBorrowFormation`).
    pub fn form_ref(
        &mut self,
        operand: BorrowOperand,
    ) -> Result<BorrowViewId, BorrowFormationFailure> {
        match operand {
            BorrowOperand::Actual(target) => Ok(self.allocate_borrow(BorrowKind::Ref, target)),
            BorrowOperand::Borrow(existing) => {
                let view = self
                    .borrows
                    .get(&existing)
                    .ok_or(BorrowFormationFailure::UnknownBorrow(existing))?;
                match view.kind {
                    BorrowKind::Ref => Ok(existing),
                    BorrowKind::Share => Err(BorrowFormationFailure::NoCandidateForStrengthening),
                }
            }
        }
    }

    /// Explicit `share` formation.  `share(share(q))` is a fixed point and
    /// `share(ref(q))` is the one legal capability weakening.
    pub fn form_share(
        &mut self,
        operand: BorrowOperand,
    ) -> Result<BorrowViewId, BorrowFormationFailure> {
        match operand {
            BorrowOperand::Actual(target) => Ok(self.allocate_borrow(BorrowKind::Share, target)),
            BorrowOperand::Borrow(existing) => {
                let view = self
                    .borrows
                    .get(&existing)
                    .cloned()
                    .ok_or(BorrowFormationFailure::UnknownBorrow(existing))?;
                match view.kind {
                    BorrowKind::Share => Ok(existing),
                    BorrowKind::Ref => Ok(self.allocate_borrow(BorrowKind::Share, view.target)),
                }
            }
        }
    }

    /// Explicit retargeting of the borrow value held by `borrow`.  This never
    /// follows the old referent and never infers a target from a temporary.
    pub fn rebind_borrow(
        &mut self,
        borrow: BorrowViewId,
        new_target: StableBorrowTarget,
    ) -> Result<(), BorrowFormationFailure> {
        let view = self
            .borrows
            .get_mut(&borrow)
            .ok_or(BorrowFormationFailure::UnknownBorrow(borrow))?;
        view.target = new_target;
        Ok(())
    }

    pub fn borrow_target_is_valid(&self, borrow: BorrowViewId) -> bool {
        self.borrows
            .get(&borrow)
            .is_some_and(|view| self.stable_target_is_current(&view.target))
    }

    pub fn stable_target_is_current(&self, target: &StableBorrowTarget) -> bool {
        match target {
            StableBorrowTarget::Place { place, resident } => self
                .places
                .get(place)
                .is_some_and(|current| current.resident == *resident),
            StableBorrowTarget::Projection(slot) => self
                .places
                .values()
                .any(|place| place.resident == slot.parent),
        }
    }

    /// Whole-resident replacement.  Every old projection-slot family becomes
    /// invalid; same-spelled future navigation resolves under the new resident.
    pub fn replace_place_resident(
        &mut self,
        place: ObjectPlaceId,
        writable: &WritableContext,
    ) -> Result<ResidentGeneration, PlaceMutationFailure> {
        if !writable.place_is_writable(place) {
            return Err(PlaceMutationFailure::NotWritable);
        }
        let current = self
            .places
            .get_mut(&place)
            .ok_or(PlaceMutationFailure::UnknownPlace(place))?;
        let resident = ResidentIdentity(self.next_resident);
        self.next_resident = self
            .next_resident
            .checked_add(1)
            .expect("resident identity exhausted");
        current.resident = ResidentGeneration {
            resident,
            generation: current.resident.generation.saturating_add(1),
        };
        current.associated_symbols.clear();
        current.associated_val2.clear();
        let resident = current.resident;
        self.semantic_val2_snapshots
            .insert(place, SemanticVal2Snapshot::default());
        Ok(resident)
    }

    /// Missing-member creation.  Unlike ordinary `=`, this operation requires
    /// a missing prospective slot and establishes its first contents.
    pub fn create_projection_value(
        &mut self,
        place: ObjectPlaceId,
        selector: ProjectionSelector,
        value: SemanticValueId,
        writable: &WritableContext,
    ) -> Result<ProjectionSlotIdentity, PlaceMutationFailure> {
        let slot = self
            .projection_slot(place, selector.clone())
            .ok_or(PlaceMutationFailure::UnknownPlace(place))?;
        if !writable.slot_is_writable(place, &slot.identity) {
            return Err(PlaceMutationFailure::NotWritable);
        }
        if slot.contents == ProjectionSlotContents::Occupied {
            return Err(PlaceMutationFailure::SlotAlreadyOccupied(slot.identity));
        }
        self.places
            .get_mut(&place)
            .expect("place was checked")
            .associated_val2
            .insert(projection_storage_key(&selector), vec![value]);
        self.refresh_semantic_val2_snapshot(place)
            .expect("checked place has a semantic Val2 snapshot");
        Ok(slot.identity)
    }

    /// Existing-member write.  This operation cannot create a missing slot.
    pub fn write_projection_value(
        &mut self,
        place: ObjectPlaceId,
        selector: ProjectionSelector,
        value: SemanticValueId,
        writable: &WritableContext,
    ) -> Result<ProjectionSlotIdentity, PlaceMutationFailure> {
        let slot = self
            .projection_slot(place, selector.clone())
            .ok_or(PlaceMutationFailure::UnknownPlace(place))?;
        if !writable.slot_is_writable(place, &slot.identity) {
            return Err(PlaceMutationFailure::NotWritable);
        }
        if slot.contents == ProjectionSlotContents::Missing {
            return Err(PlaceMutationFailure::SlotMissing(slot.identity));
        }
        self.places
            .get_mut(&place)
            .expect("place was checked")
            .associated_val2
            .insert(projection_storage_key(&selector), vec![value]);
        self.refresh_semantic_val2_snapshot(place)
            .expect("checked place has a semantic Val2 snapshot");
        Ok(slot.identity)
    }

    /// The source-visible Val2 Symbol of one object place.
    ///
    /// `Val2(obj)[f] = C_f`: the place's `associated_symbols` is the single
    /// authority for source-visible names of that object.
    pub fn associated_symbol_in_place(
        &self,
        place: ObjectPlaceId,
        name: &str,
    ) -> Option<SemanticSymbolIdentity> {
        self.places
            .get(&place)?
            .associated_symbols
            .get(name)
            .copied()
    }

    /// The source-visible Val2 Symbol of a Pattern's canonical pure type Object.
    pub fn associated_symbol_for_pattern(
        &self,
        pattern: PatternValueId,
        name: &str,
    ) -> Option<SemanticSymbolIdentity> {
        self.associated_symbol_in_place(self.pattern_place(pattern)?, name)
    }

    /// The source-visible Val2 Symbol reached through one host layer.
    ///
    /// The host object's own place wins: `let T: type = uint8; let f::T = ...`
    /// installs `f` on `T`'s object only, so `uint8::f` and `U::f` must not
    /// see it.  A host whose own place carries nothing under the name falls
    /// back to the Pattern's canonical pure type Object, which is where
    /// construction-time and toolchain-installed type members live.
    pub fn associated_symbol_for_host(
        &self,
        host: &PatternHostMember,
        name: &str,
    ) -> Option<SemanticSymbolIdentity> {
        self.associated_symbol_in_place(host.place, name)
            .or_else(|| self.associated_symbol_for_pattern(host.pattern, name))
    }

    /// Record a source-visible Val2 name on a Pattern's canonical pure type Object.
    pub fn associate_existing_symbol(
        &mut self,
        pattern: PatternValueId,
        name: &str,
        symbol: SemanticSymbolIdentity,
    ) -> Option<()> {
        self.associate_existing_symbol_in_place(self.pattern_place(pattern)?, name, symbol)
    }

    /// Record a source-visible Val2 name on one specific object place.
    pub fn associate_existing_symbol_in_place(
        &mut self,
        place: ObjectPlaceId,
        name: &str,
        symbol: SemanticSymbolIdentity,
    ) -> Option<()> {
        if !self.symbols.contains_key(&symbol) {
            return None;
        }
        let previous = self
            .places
            .get_mut(&place)?
            .associated_symbols
            .insert(name.to_string(), symbol);
        debug_assert!(previous.is_none() || previous == Some(symbol));
        self.refresh_semantic_val2_snapshot(place)?;
        Some(())
    }

    pub fn associate_existing_value(
        &mut self,
        pattern: PatternValueId,
        name: &str,
        value: SemanticValueId,
    ) -> Option<()> {
        self.associate_existing_value_in_place(self.pattern_place(pattern)?, name, value)
    }

    pub fn associate_existing_value_in_place(
        &mut self,
        place: ObjectPlaceId,
        name: &str,
        value: SemanticValueId,
    ) -> Option<()> {
        if !self.values.contains_key(&value) {
            return None;
        }
        let values = self
            .places
            .get_mut(&place)?
            .associated_val2
            .entry(name.to_string())
            .or_default();
        if !values.contains(&value) {
            values.push(value);
        }
        self.refresh_semantic_val2_snapshot(place)?;
        Some(())
    }

    pub fn function_value_for_backing_symbol(&self, symbol: SymbolId) -> Option<SemanticValueId> {
        self.backing_to_function_value.get(&symbol).copied()
    }

    pub fn backing_declaration_for_symbol(
        &self,
        symbol: SemanticSymbolIdentity,
    ) -> Option<SymbolId> {
        self.symbol_backing_declarations.get(&symbol).copied()
    }

    /// Render an already-selected semantic Symbol through the graph projection
    /// declaration projection.
    ///
    /// The name index is deliberately not consulted for selection here:
    /// semantic path/scope resolution must choose `symbol` before an older API
    /// asks for its `SymbolObject` representation.
    pub fn projected_symbol_object(
        &self,
        symbol: SemanticSymbolIdentity,
    ) -> Option<&crate::SymbolObject> {
        let backing = self.backing_declaration_for_symbol(symbol)?;
        self.namespace_index.symbol(backing)
    }

    /// Synthesize member views for values reached outside a cluster Symbol
    /// (for example a Pattern owner's associated Val2 entries).
    ///
    /// Each value's own installed PolicyPair is its member-level Policy
    /// fact.  This is a per-value projection, never a flat Symbol-level
    /// aggregate, so no cross-member Policy union can occur here.
    pub fn member_views_for_values(
        &self,
        values: &[SemanticValueId],
    ) -> Vec<PolicyResultEntry<SemanticValueId, PatternValueId>> {
        values
            .iter()
            .filter_map(|id| {
                let value = self.values.get(id)?;
                Some(PolicyResultEntry {
                    value: Some(*id),
                    pattern: value.pattern,
                    view: PolicyView {
                        pair: value.policy.clone(),
                        mode: value.mode,
                    },
                })
            })
            .collect()
    }

    /// Canonical callable/member projection for one name reached through a
    /// host layer.
    ///
    /// `CallableProjection(S) = DedupCandidateIdentity(V_S ⊎ V_tau)`: local
    /// Symbol members and the immutable complete-type snapshot occupy one
    /// candidate space. Transport-only values are admitted as a one-way
    /// projection source, then deduplicated in that same space. There is no
    /// local-first / TypeMember-second fallback tier.
    ///
    /// Exposure composes per layer and per phase:
    ///
    /// ```text
    /// Expose(t::f, φ) = Expose(T_t, φ) ∧ Expose(f, φ)
    /// ```
    ///
    /// The host factor is decided here, from the host's own binding-level
    /// member view (see [`PatternHostMember::exposed_at`]) — never from the
    /// shared CoreTypeProjection adapter, which is transport material and carries no
    /// per-binding Policy. The member factor stays where it already lives —
    /// the per-member exposure stage of the invocation pipeline — so member
    /// views pass through unmodified here. The host Policy is only READ; it is
    /// never folded into, disjoined with, or written back to any member.
    pub fn associated_member_views_for_host(
        &self,
        host: &PatternHostMember,
        name: &str,
        phase: crate::Phase,
    ) -> Vec<PolicyResultEntry<SemanticValueId, PatternValueId>> {
        if !host.exposed_at(phase) {
            return Vec::new();
        }
        let mut projected = self
            .associated_symbol_for_host(host, name)
            .and_then(|symbol| self.symbols.get(&symbol))
            .map(|cell| cell.member_views.clone())
            .unwrap_or_default();

        let type_members = host
            .complete_type
            .and_then(|whole| self.complete_types.get(&whole))
            .map(|complete| &complete.call_space)
            .or_else(|| {
                self.type_for_pattern(host.pattern)
                    .and_then(|lookup| self.direct_type_members.get(&lookup))
            })
            .and_then(|call_space| call_space.get(name))
            .map(|entries| entries.iter().map(|entry| entry.value).collect::<Vec<_>>())
            .unwrap_or_default();
        projected.extend(self.member_views_for_values(&type_members));

        let transported = self
            .associated_values_in_place(host.place, name)
            .or_else(|| self.associated_values_for_pattern(host.pattern, name))
            .map(<[SemanticValueId]>::to_vec)
            .unwrap_or_default();
        projected.extend(self.member_views_for_values(&transported));

        let mut seen_values = BTreeSet::new();
        let mut seen_pure_patterns = BTreeSet::new();
        projected.retain(|view| match view.value {
            Some(value) => seen_values.insert(value),
            None => seen_pure_patterns.insert(view.pattern),
        });
        projected
    }

    /// Associated-member views of a bare Pattern host.
    ///
    /// This is the compiler-internal entry point: no carrier Symbol named the
    /// host step (an authorized receiver operation, for example), so there is
    /// no binding-level host view to compose and lookup lands on the Pattern's
    /// canonical pure type Object. Source navigation must use
    /// [`Self::associated_member_views_for_host`] with the carrier's own host
    /// layer instead, otherwise the host factor of the exposure conjunction is
    /// silently dropped.
    pub fn associated_member_views_for_pattern(
        &self,
        pattern: PatternValueId,
        name: &str,
        phase: crate::Phase,
    ) -> Vec<PolicyResultEntry<SemanticValueId, PatternValueId>> {
        match self.host_member_for_pattern(pattern) {
            Some(host) => self.associated_member_views_for_host(&host, name, phase),
            None => Vec::new(),
        }
    }

    /// The pure-P member one carrier installs for `pattern`.
    ///
    /// A pure P is a real object, so each carrier owns its own writable Val2
    /// place while the Pattern stays shared identity material.  The Pattern's
    /// canonical pure type Object belongs to the cluster that declared the Pattern:
    /// that carrier keeps writing there, because construction-time members
    /// were injected into the canonical place before the carrier existed.
    /// Every other carrier of the same Pattern — `let U: type = T` — binds a
    /// new object and therefore receives a fresh writable place, so a later
    /// `let f::U` cannot reach `T` or `uint8`.  Physically shared canonical
    /// members are copied into the fresh object's semantic Val2 snapshot at
    /// formation; later navigation fallback never changes that owned
    /// snapshot or its Object normal form.
    ///
    /// An existing pure-P member is never re-placed: rebinding a carrier is a
    /// new declaration, not a mutation of the previous object.
    fn pure_p_member_for_carrier(
        &mut self,
        symbol: SemanticSymbolIdentity,
        pattern: PatternValueId,
    ) -> PurePMember {
        if let Some(mut existing) = self.symbols.get(&symbol).and_then(|cell| cell.pure_p) {
            if existing.complete_type.is_none() {
                existing.complete_type = self.type_for_pattern(pattern).and_then(|type_value| {
                    self.observe_complete_type(type_value, Some(existing.place))
                        .ok()
                        .map(|complete| complete.whole)
                });
            }
            return existing;
        }
        let declares_pattern = match self.owner_cluster(pattern) {
            Some(PatternClusterOwner::Installed(owner)) => owner == symbol,
            _ => true,
        };
        let place = match self.pattern_place(pattern) {
            Some(canonical) if declares_pattern => canonical,
            canonical => {
                let place = self.allocate_object_place();
                if let Some(inherited) = canonical
                    .and_then(|canonical| self.semantic_val2_snapshots.get(&canonical))
                    .cloned()
                {
                    self.semantic_val2_snapshots.insert(place, inherited);
                }
                place
            }
        };
        let complete_type = self.type_for_pattern(pattern).and_then(|type_value| {
            self.observe_complete_type(type_value, Some(place))
                .ok()
                .map(|complete| complete.whole)
        });
        PurePMember {
            pattern,
            place,
            complete_type,
        }
    }

    pub fn register_type_symbol(
        &mut self,
        namespace: NamespaceNodeId,
        name: &str,
        binding_symbol: SymbolId,
        represented_type: TypeValueId,
        type_rank: TypeValueId,
        associated_namespace: Option<NamespaceNodeId>,
        policy: PolicyPair,
        provenance: Provenance,
    ) -> Option<(SemanticSymbolIdentity, SemanticValueId, PatternValueId)> {
        self.register_type_symbol_with_complete_type(
            namespace,
            name,
            binding_symbol,
            represented_type,
            None,
            type_rank,
            associated_namespace,
            policy,
            provenance,
        )
    }

    /// Register a type carrier while preserving an already-observed exact
    /// complete-type snapshot. This is the semantic-result binding entry;
    /// the lookup-only wrapper above is reserved for bootstrap callers.
    pub fn register_type_symbol_with_complete_type(
        &mut self,
        namespace: NamespaceNodeId,
        name: &str,
        binding_symbol: SymbolId,
        represented_type: TypeValueId,
        complete_type: Option<CanonicalValueAddr>,
        type_rank: TypeValueId,
        associated_namespace: Option<NamespaceNodeId>,
        policy: PolicyPair,
        provenance: Provenance,
    ) -> Option<(SemanticSymbolIdentity, SemanticValueId, PatternValueId)> {
        let owner = self.namespace_owner(namespace)?;
        let symbol = self.intern_symbol(namespace, owner, name, provenance.clone());
        self.symbol_backing_declarations
            .entry(symbol)
            .or_insert(binding_symbol);
        let represented_pattern = if let Some(existing) = self.types.get(&represented_type) {
            existing.pattern
        } else {
            let pattern_owner = associated_namespace
                .and_then(|namespace| self.namespace_owner(namespace))
                .unwrap_or(owner);
            let (pattern, _scope) = self.allocate_pattern(pattern_owner, provenance.clone());
            self.types.insert(
                represented_type,
                SemanticTypeValue {
                    id: represented_type,
                    pattern,
                    provenance: provenance.clone(),
                },
            );
            self.pattern_types.insert(pattern, represented_type);
            self.pattern_clusters
                .entry(pattern)
                .or_insert(PatternClusterOwner::Installed(symbol));
            pattern
        };
        debug_assert!(
            self.pattern_clusters.contains_key(&represented_pattern),
            "PatternValue {:?} must already have an owning cluster at carrier installation time",
            represented_pattern.0
        );
        if represented_type == type_rank {
            self.type_rank = Some(type_rank);
        }
        if name == "symbol" {
            self.symbol_rank = Some(represented_type);
        }
        let value_type_pattern = self.types.get(&type_rank)?.pattern;
        let value = if let Some(existing) = self
            .core_type_projection_values
            .get(&represented_type)
            .copied()
        {
            existing
        } else {
            let value = self.allocate_value_id();
            let place = self.allocate_object_place();
            self.values.insert(
                value,
                SemanticValueObject {
                    id: value,
                    type_value: type_rank,
                    pattern: represented_pattern,
                    place,
                    policy: policy.clone(),
                    mode: PolicyMode::Plain,
                    namespace_visibility: None,
                    payload: SemanticValuePayload::CoreTypeProjection {
                        represented_type,
                        represented_pattern,
                    },
                    provenance: provenance.clone(),
                },
            );
            self.core_type_projection_values
                .insert(represented_type, value);
            value
        };
        debug_assert!(matches!(
            self.values.get(&value),
            Some(SemanticValueObject {
                type_value,
                pattern,
                payload:
                    SemanticValuePayload::CoreTypeProjection {
                        represented_type: existing,
                        ..
                    },
                ..
            }) if *type_value == type_rank
                && *pattern == represented_pattern
                && *existing == represented_type
        ));
        let mut member = self.pure_p_member_for_carrier(symbol, represented_pattern);
        if let Some(whole) = complete_type {
            debug_assert!(self.complete_types.contains_key(&whole));
            member.complete_type = Some(whole);
        }
        let cell = self
            .symbols
            .get_mut(&symbol)
            .expect("interned semantic symbol exists");
        match &mut cell.pure_p {
            Some(existing) => {
                debug_assert_eq!(existing.pattern, member.pattern);
                if existing.complete_type.is_none() {
                    existing.complete_type = member.complete_type;
                }
            }
            slot @ None => *slot = Some(member),
        }
        let pure_p_view = PolicyResultEntry {
            value: None,
            pattern: represented_pattern,
            view: PolicyView {
                pair: policy,
                mode: PolicyMode::Plain,
            },
        };
        if !cell.member_views.contains(&pure_p_view) {
            cell.member_views.push(pure_p_view);
        }
        if let Some(namespace) = associated_namespace {
            self.associated_namespace_patterns
                .insert(namespace, represented_pattern);
        }
        self.registered_type_bindings.insert(binding_symbol);
        debug_assert_eq!(
            self.types.get(&type_rank).map(|value| value.pattern),
            Some(value_type_pattern)
        );
        Some((symbol, value, represented_pattern))
    }

    /// Install (or reuse) the canonical type member generated by a meta
    /// invocation body.
    ///
    /// `MetaRootKey = ParentSemanticOwner + MetaCallableIdentity +
    /// Normalize(Arguments)`: the
    /// canonical TypeValue root is keyed by [`MetaInstanceRootKey`], which never
    /// includes body material.  `normalized_body` is
    /// content under the root — replaying the same root key with an equal
    /// body is an idempotent reuse, while a different body under one root
    /// key is a construction conflict hard error, never a second root.
    /// Returns the type-object value, the canonical pattern, and the
    /// canonical TypeValue root; `Ok(None)` reports missing internal
    /// prerequisites (unknown declaring symbol or absent type rank).
    pub fn install_generated_type_value(
        &mut self,
        root: &MetaInstanceRoot,
        canonical_key: MetaInvocationMaterialKey,
        normalized_body: TypeDefinitionInstanceId,
        canonical_pattern: CanonicalPatternValue,
        policy: PolicyPair,
        provenance: Provenance,
    ) -> Result<Option<(SemanticValueId, PatternValueId, TypeValueId)>, crate::Diagnostic> {
        if canonical_key.callable != root.meta_callable {
            return Err(crate::Diagnostic::hard_error(
                "meta root callable disagrees with the invocation material key",
                Some(provenance),
            ));
        }
        let key = MetaInstanceRootKey {
            parent_owner: root.placement_parent,
            material: canonical_key.clone(),
        };
        let canonical_type = match self.meta_type_roots.get(&key) {
            Some((existing, existing_body)) => {
                if *existing_body != normalized_body {
                    // Same root, conflicting body: the root is never split.
                    return Err(crate::Diagnostic::hard_error(
                        "meta construction conflict: the same meta function and normalized \
                         arguments produced a different normalized body; a conflicting body \
                         never allocates a second root",
                        Some(provenance),
                    ));
                }
                *existing
            }
            None => {
                let owner = self
                    .owners
                    .meta_instance(root.placement_parent, canonical_key);
                let id = self.allocate_anonymous_type();
                let (pattern, _scope) = self.allocate_pattern(owner, provenance.clone());
                self.types.insert(
                    id,
                    SemanticTypeValue {
                        id,
                        pattern,
                        provenance: provenance.clone(),
                    },
                );
                self.pattern_types.insert(pattern, id);
                self.pattern_structural_norms
                    .insert(pattern, canonical_pattern);
                self.meta_type_roots.insert(key, (id, normalized_body));
                id
            }
        };
        let Some(pattern) = self.types.get(&canonical_type).map(|value| value.pattern) else {
            return Ok(None);
        };
        if let Some(value) = self
            .core_type_projection_values
            .get(&canonical_type)
            .copied()
        {
            return Ok(Some((value, pattern, canonical_type)));
        }
        let Some(type_rank) = self.type_rank else {
            return Ok(None);
        };
        let value = self.allocate_value_id();
        let place = self.allocate_object_place();
        self.values.insert(
            value,
            SemanticValueObject {
                id: value,
                type_value: type_rank,
                pattern,
                place,
                policy: policy.clone(),
                mode: root.policy_mode(),
                namespace_visibility: None,
                payload: SemanticValuePayload::CoreTypeProjection {
                    represented_type: canonical_type,
                    represented_pattern: pattern,
                },
                provenance,
            },
        );
        self.core_type_projection_values
            .insert(canonical_type, value);
        Ok(Some((value, pattern, canonical_type)))
    }

    /// An earlier ambient struct generation with the same normalized
    /// navigation shape at the same declaration level, if any, together
    /// with its recorded binder (diagnostic material only).
    ///
    /// Replaying the same shape at one level is currently a hard error even
    /// though the material is normalization-equal.  Precise idempotent replay
    /// requires absorbing every preceding `let f::t`-style injection into
    /// the struct's own internal `let` material first — the generated
    /// pattern's state in the current computation flow, not the moral
    /// snapshot at generation time.  That absorption is explicitly
    /// registered future work; once it lands, `same root + same absorbed
    /// material` becomes a legal idempotent reuse.
    pub fn ambient_struct_collision(
        &self,
        ambient_owner: SemanticOwnerId,
        normalized_body: TypeDefinitionInstanceId,
    ) -> Option<(TypeValueId, Option<&AmbientTypeBinder>)> {
        let existing = self
            .ambient_struct_types
            .get(&(ambient_owner, normalized_body))
            .copied()?;
        Some((existing, self.ambient_type_binders.get(&existing)))
    }

    /// Install the type generated by a *direct* `struct` call in an
    /// ordinary declaration context (`OwnerStrategy::AmbientStructScope`).
    ///
    /// The generated type attaches to the ambient declaration environment
    /// as Self; no `MetaInstance(struct, arguments)` scope is created.
    /// Callers must reject an [`Self::ambient_struct_collision`] hit before
    /// calling: one (level, normalized navigation shape) is generated at
    /// most once.
    pub fn install_ambient_struct_type_value(
        &mut self,
        ambient_owner: SemanticOwnerId,
        normalized_body: TypeDefinitionInstanceId,
        canonical_pattern: CanonicalPatternValue,
        policy: PolicyPair,
        provenance: Provenance,
    ) -> Option<(SemanticValueId, PatternValueId, TypeValueId)> {
        debug_assert!(
            !self
                .ambient_struct_types
                .contains_key(&(ambient_owner, normalized_body)),
            "ambient struct collision must be rejected before installation"
        );
        let canonical_type = self.allocate_anonymous_type();
        let (pattern, _scope) = self.allocate_pattern(ambient_owner, provenance.clone());
        self.types.insert(
            canonical_type,
            SemanticTypeValue {
                id: canonical_type,
                pattern,
                provenance: provenance.clone(),
            },
        );
        self.pattern_types.insert(pattern, canonical_type);
        self.pattern_structural_norms
            .insert(pattern, canonical_pattern);
        self.ambient_struct_types
            .insert((ambient_owner, normalized_body), canonical_type);
        let type_rank = self.type_rank?;
        let value = self.allocate_value_id();
        let place = self.allocate_object_place();
        self.values.insert(
            value,
            SemanticValueObject {
                id: value,
                type_value: type_rank,
                pattern,
                place,
                policy,
                mode: PolicyMode::Plain,
                namespace_visibility: None,
                payload: SemanticValuePayload::CoreTypeProjection {
                    represented_type: canonical_type,
                    represented_pattern: pattern,
                },
                provenance,
            },
        );
        self.core_type_projection_values
            .insert(canonical_type, value);
        Some((value, pattern, canonical_type))
    }

    /// Record how an ambient struct generation was bound at its declaration
    /// site.  Diagnostic material only — the binder never feeds type
    /// identity.  A no-op for types that are not ambient struct
    /// generations.
    pub fn record_ambient_type_binder(
        &mut self,
        type_value: TypeValueId,
        binder: AmbientTypeBinder,
    ) {
        if self
            .ambient_struct_types
            .values()
            .any(|installed| *installed == type_value)
        {
            self.ambient_type_binders.insert(type_value, binder);
        }
    }

    pub fn allocate_meta_result_pattern(
        &mut self,
        root: &MetaInstanceRoot,
        canonical_key: MetaInvocationMaterialKey,
        provenance: Provenance,
    ) -> Option<PatternValueId> {
        debug_assert_eq!(root.meta_callable, canonical_key.callable);
        let owner = self
            .owners
            .meta_instance(root.placement_parent, canonical_key);
        Some(self.allocate_pattern(owner, provenance).0)
    }

    /// Install the unique type member of a meta-instance cluster.
    ///
    /// v0.9 pattern head identity: the type member of a cluster returned by
    /// a meta invocation is navigated as the meta function itself plus its
    /// input arguments, so its PatternValue is allocated under the
    /// `MetaInstance` owner and paired with a fresh anonymous TypeValue.
    /// A type forwarded by the body keeps its own PatternValue and owner
    /// untouched; it never becomes the cluster's type member directly.
    pub fn install_meta_instance_type_value(
        &mut self,
        root: &MetaInstanceRoot,
        canonical_key: MetaInvocationMaterialKey,
        provenance: Provenance,
    ) -> Option<(TypeValueId, PatternValueId)> {
        debug_assert_eq!(root.meta_callable, canonical_key.callable);
        let owner = self
            .owners
            .meta_instance(root.placement_parent, canonical_key);
        let (pattern, _scope) = self.allocate_pattern(owner, provenance.clone());
        let id = self.allocate_anonymous_type();
        self.types.insert(
            id,
            SemanticTypeValue {
                id,
                pattern,
                provenance,
            },
        );
        self.pattern_types.insert(pattern, id);
        Some((id, pattern))
    }

    /// Allocate a terminal call entry with an independent FunctionItem type
    /// and pattern.
    ///
    /// The call entry's own scope is never populated — `Type(c) =
    /// FunctionItem(Self, Args...) -> Result` and `c.Val2 = ∅`.  The call
    /// entry is registered in `owner_pattern`'s immutable TypeMember
    /// callspace under `operation_selector`, not in its own FunctionItem
    /// scope. Source-visible callables additionally materialize the same
    /// entry in the owner's owned Val2; compiler-only operation families can
    /// remain V_tau-only so V_tau is never redefined as Object Val2.
    ///
    /// This is the single semantic construction primitive shared by all
    /// callable construction paths (associated call entries, source
    /// callables, core callables, and cluster-contributed function
    /// objects and compiler-authorized operation families). It guarantees
    /// that every call entry is terminal regardless of which entrance
    /// reached it.
    #[allow(clippy::too_many_arguments)]
    fn allocate_terminal_call_entry(
        &mut self,
        owner_pattern: PatternValueId,
        backing_declaration: SymbolId,
        declaration_name: &str,
        operation_selector: &str,
        materialize_owned_val2: bool,
        declaration_namespace: Option<NamespaceNodeId>,
        closure: Option<&NormClosure>,
        core_primitive: Option<CoreMetaFunction>,
        intrinsic_body: Option<OrdinaryIntrinsicBody>,
        callable_owner: SemanticOwnerId,
        receiver_type: TypeValueId,
        canonical_view: PolicyView,
        body_entry_view: PolicyView,
        complete_result_view: PolicyView,
        return_position_view: PolicyView,
        namespace_visibility: Option<NamespaceVisibility>,
        candidate_role: OrdinaryCandidateRole,
        return_shape: ReturnShape,
        privilege: CallablePrivilege,
        provenance: Provenance,
    ) -> Result<SemanticValueId, BuildError> {
        // Capture a clone of provenance for use in error closures below —
        // `provenance` is moved into SemanticValueObject later in this
        // function, and we cannot borrow it after the move.
        let err_provenance = provenance.clone();
        let owner_pattern_value = self
            .pattern(owner_pattern)
            .ok_or_else(|| {
                BuildError::single(crate::Diagnostic::hard_error(
                    "allocate_terminal_call_entry: owner pattern not registered",
                    Some(err_provenance.clone()),
                ))
            })?
            .clone();

        // Allocate an independent anonymous FunctionItem type and pattern for
        // the () call entry.  Type(c) = FunctionItem(Self, Args...) -> Result.
        // This type is distinct from the receiver/owner type; the call entry
        // is a terminal FunctionItem with Val2 = ∅.
        let function_item_type = self.allocate_anonymous_type();
        let (function_item_pattern, _function_item_scope) =
            self.allocate_pattern(owner_pattern_value.root.owner, provenance.clone());
        self.types.insert(
            function_item_type,
            SemanticTypeValue {
                id: function_item_type,
                pattern: function_item_pattern,
                provenance: provenance.clone(),
            },
        );
        self.pattern_types
            .insert(function_item_pattern, function_item_type);

        let call_entry = self.allocate_value_id();
        self.materialize_val1_object(SemanticValueObject {
            id: call_entry,
            type_value: function_item_type,
            pattern: function_item_pattern,
            place: ObjectPlaceId(0), // overwritten by materialize_val1_object
            policy: canonical_view.pair.clone(),
            mode: canonical_view.mode,
            namespace_visibility,
            payload: SemanticValuePayload::CallEntry(OrdinaryCallEntry {
                backing_declaration,
                declaration_name: declaration_name.to_string(),
                declaration_namespace,
                callable_owner,
                receiver_type,
                closure: closure.cloned(),
                core_primitive,
                intrinsic_body,
                body_entry_view,
                complete_result_view,
                return_position_view,
                callable_view: canonical_view,
                capability_realization: CapabilityRealization::default(),
                candidate_role,
                return_shape,
                privilege,
                provenance: provenance.clone(),
            }),
            provenance,
        });
        // The call entry always enters the owner's immutable TypeMember
        // callspace. Only source-visible families also own a corresponding
        // Object Val2 member; the FunctionItem scope itself remains terminal.
        let place_id = self
            .pattern_places
            .get(&owner_pattern)
            .copied()
            .ok_or_else(|| {
                BuildError::single(crate::Diagnostic::hard_error(
                    "allocate_terminal_call_entry: owner pattern has no allocated place",
                    Some(err_provenance.clone()),
                ))
            })?;
        if materialize_owned_val2 {
            self.associate_existing_value_in_place(place_id, operation_selector, call_entry)
                .expect("allocated pattern place and call entry exist");
        }
        self.admit_direct_type_member(
            owner_pattern,
            owner_pattern,
            operation_selector,
            TypeMemberFacet::Value,
            call_entry,
        )
        .map_err(BuildError::single)?;
        Ok(call_entry)
    }

    /// Register one compiler-authorized ordinary intrinsic directly in a
    /// target Type's operation family. The returned call entry is the
    /// candidate identity; `backing_declaration` is projection/provenance
    /// material and never participates in selection identity.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn register_intrinsic_type_operation(
        &mut self,
        target_type: TypeValueId,
        operation_selector: &str,
        backing_declaration: SymbolId,
        body: OrdinaryIntrinsicBody,
        callable_view: PolicyView,
        complete_result_view: PolicyView,
        provenance: Provenance,
    ) -> Result<SemanticValueId, BuildError> {
        let target_pattern = self
            .type_value(target_type)
            .map(|value| value.pattern)
            .ok_or_else(|| {
                BuildError::single(crate::Diagnostic::hard_error(
                    "intrinsic operation target Type is not installed",
                    Some(provenance.clone()),
                ))
            })?;
        let callable_owner = self.owners.callable(
            self.pattern(target_pattern)
                .expect("installed Type Pattern exists")
                .root
                .owner,
            LocalCallableIdentity(self.next_callable),
            CallableOwnerPlacement::Ordinary,
        );
        self.next_callable = self
            .next_callable
            .checked_add(1)
            .expect("semantic callable identity exhausted");
        let receiver_type = self.type_rank.ok_or_else(|| {
            BuildError::single(crate::Diagnostic::hard_error(
                "intrinsic Type operation requires the builtin `type` rank",
                Some(provenance.clone()),
            ))
        })?;
        self.allocate_terminal_call_entry(
            target_pattern,
            backing_declaration,
            operation_selector,
            operation_selector,
            false,
            None,
            None,
            None,
            Some(body),
            callable_owner,
            receiver_type,
            callable_view.clone(),
            complete_result_view.clone(),
            complete_result_view,
            callable_view,
            None,
            OrdinaryCandidateRole::Ordinary,
            ReturnShape::SingleVal(crate::PatternConstraint::Unconstrained),
            CallablePrivilege::BuiltinPrivileged,
            provenance,
        )
    }

    /// Install one [`SemanticNamespaceDelta`] atomically.
    ///
    /// Every entry applies to a scratch copy of the semantic world; the
    /// copy replaces the live world only when all entries succeeded, so a
    /// failing entry installs nothing. The semantic world is the declaration
    /// installation authority; graph projections never define the
    /// installed identities or member policies.
    pub fn install_namespace_delta(
        &mut self,
        delta: SemanticNamespaceDelta,
    ) -> Result<(), BuildError> {
        let mut staged = self.clone();
        for entry in delta.entries {
            match entry {
                SemanticDeclarationEntry::AssociatedCallEntry {
                    pattern,
                    backing_declaration,
                    closure,
                    outer_p1_explicit,
                    callable_view,
                    body_entry_view,
                    namespace_visibility,
                    candidate_role,
                    return_shape,
                    provenance,
                } => {
                    staged.register_associated_call_entry(
                        pattern,
                        delta.namespace,
                        backing_declaration,
                        &closure,
                        outer_p1_explicit,
                        callable_view,
                        body_entry_view,
                        namespace_visibility,
                        candidate_role,
                        return_shape,
                        provenance,
                    )?;
                }
                SemanticDeclarationEntry::SourceCallable {
                    name,
                    backing_declaration,
                    closure,
                    outer_p1_explicit,
                    function_view,
                    body_entry_view,
                    namespace_visibility,
                    return_shape,
                    provenance,
                } => {
                    let registered = staged.register_source_callable(
                        delta.namespace,
                        &name,
                        backing_declaration,
                        &closure,
                        outer_p1_explicit,
                        function_view,
                        body_entry_view,
                        namespace_visibility,
                        return_shape,
                        provenance,
                    )?;
                    if let Some(pattern) = staged.pattern_for_associated_namespace(delta.namespace)
                    {
                        staged
                            .associate_existing_symbol(pattern, &name, registered.symbol)
                            .expect("registered associated source Symbol exists");
                        staged
                            .associate_existing_value(pattern, &name, registered.function_value)
                            .expect("registered source callable value exists");
                    }
                }
                SemanticDeclarationEntry::ClusterContribution {
                    cluster_symbol,
                    backing_declaration,
                    closure,
                    outer_p1_explicit,
                    function_view,
                    body_entry_view,
                    namespace_visibility,
                    return_shape,
                    provenance,
                } => {
                    let binder_name = staged
                        .symbol(cluster_symbol)
                        .map(|cell| cell.name.clone())
                        .ok_or_else(|| {
                            BuildError::single(crate::Diagnostic::hard_error(
                                "cluster contribution target symbol not found",
                                Some(provenance.clone()),
                            ))
                        })?;
                    let (function_value, _call_entry) = staged
                        .contribute_function_object_to_cluster(
                            cluster_symbol,
                            delta.namespace,
                            backing_declaration,
                            &closure,
                            outer_p1_explicit,
                            function_view,
                            body_entry_view,
                            namespace_visibility,
                            return_shape,
                            provenance,
                        )?;
                    if let Some(pattern) = staged.pattern_for_associated_namespace(delta.namespace)
                    {
                        staged
                            .associate_existing_value(pattern, &binder_name, function_value)
                            .expect("contributed sibling value exists");
                    }
                }
                SemanticDeclarationEntry::TypeCarrier {
                    name,
                    binding,
                    represented_type,
                    complete_type,
                    associated_namespace,
                    policy,
                    provenance,
                } => {
                    let associated_node = match associated_namespace {
                        Some((node, local_name)) => {
                            staged
                                .register_namespace(node, delta.namespace, local_name)
                                .ok_or_else(|| {
                                    BuildError::single(crate::Diagnostic::hard_error(
                                        "type carrier namespace has no semantic owner",
                                        Some(provenance.clone()),
                                    ))
                                })?;
                            Some(node)
                        }
                        None => None,
                    };
                    let type_rank = staged.type_rank().ok_or_else(|| {
                        BuildError::single(crate::Diagnostic::hard_error(
                            "core bootstrap has not registered the canonical `type` TypeValue",
                            Some(provenance.clone()),
                        ))
                    })?;
                    staged
                        .register_type_symbol_with_complete_type(
                            delta.namespace,
                            &name,
                            binding,
                            represented_type,
                            complete_type,
                            type_rank,
                            associated_node,
                            policy,
                            provenance.clone(),
                        )
                        .ok_or_else(|| {
                            BuildError::single(crate::Diagnostic::hard_error(
                                "type carrier parent namespace has no semantic owner",
                                Some(provenance),
                            ))
                        })?;
                }
                SemanticDeclarationEntry::ProjectionOnly {
                    name,
                    backing_declaration,
                    provenance,
                } => {
                    let owner = staged.namespace_owner(delta.namespace).ok_or_else(|| {
                        BuildError::single(crate::Diagnostic::hard_error(
                            "projection-only declaration namespace has no semantic owner",
                            Some(provenance.clone()),
                        ))
                    })?;
                    let symbol = staged.intern_symbol(delta.namespace, owner, &name, provenance);
                    staged
                        .symbol_backing_declarations
                        .entry(symbol)
                        .or_insert(backing_declaration);
                    if let Some(pattern) = staged.pattern_for_associated_namespace(delta.namespace)
                    {
                        staged
                            .associate_existing_symbol(pattern, &name, symbol)
                            .expect("projection-only Symbol was just installed");
                    }
                }
            }
        }
        *self = staged;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn register_associated_call_entry(
        &mut self,
        pattern: PatternValueId,
        declaration_namespace: NamespaceNodeId,
        backing_declaration: SymbolId,
        closure: &NormClosure,
        outer_p1_explicit: Option<ExplicitP1Selection>,
        callable_view: PolicyView,
        body_entry_view: PolicyView,
        namespace_visibility: Option<NamespaceVisibility>,
        candidate_role: OrdinaryCandidateRole,
        return_shape: ReturnShape,
        provenance: Provenance,
    ) -> Result<SemanticValueId, BuildError> {
        let receiver_type = self.type_for_pattern(pattern).ok_or_else(|| {
            BuildError::single(crate::Diagnostic::hard_error(
                "associated Pattern owner has no TypeValue",
                Some(provenance.clone()),
            ))
        })?;

        // Canonical P1 normalization: reconcile the outer
        // let() P1 with the written-self P1 from the closure head. The
        // canonical P1 is the single authority — SemanticValueObject.policy,
        // OrdinaryCallEntry.callable_value_policy, and member_views.value_policy
        // all read the same canonical_p1. Mismatch between explicit outer
        // and explicit self is a hard diagnostic.
        let canonical_view = canonical_function_object_view(
            outer_p1_explicit.as_ref(),
            &callable_view,
            &body_entry_view,
            Some(closure),
            &provenance,
        )
        .map_err(BuildError::single)?;
        let return_position_view = elaborate_return_policy_pattern(
            closure
                .head
                .as_ref()
                .and_then(|head| head.returns.as_ref())
                .and_then(|slot| slot.policy.as_ref()),
            &canonical_view,
            provenance.clone(),
        )
        .map_err(BuildError::single)?
        .effective_view;

        Ok(self.allocate_terminal_call_entry(
            pattern,
            backing_declaration,
            "()",
            "()",
            true,
            Some(declaration_namespace),
            Some(closure),
            None,
            None,
            self.pattern(pattern)
                .ok_or_else(|| {
                    BuildError::single(crate::Diagnostic::hard_error(
                        "associated Pattern owner has no root owner",
                        Some(provenance.clone()),
                    ))
                })?
                .root
                .owner,
            receiver_type,
            canonical_view.clone(),
            body_entry_view.clone(),
            body_entry_view,
            return_position_view,
            namespace_visibility,
            candidate_role,
            return_shape,
            CallablePrivilege::OrdinarySource,
            provenance,
        )?)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn register_source_callable(
        &mut self,
        namespace: NamespaceNodeId,
        name: &str,
        backing_declaration: SymbolId,
        closure: &NormClosure,
        outer_p1_explicit: Option<ExplicitP1Selection>,
        function_view: PolicyView,
        body_entry_view: PolicyView,
        namespace_visibility: Option<NamespaceVisibility>,
        return_shape: ReturnShape,
        provenance: Provenance,
    ) -> Result<RegisteredCallable, BuildError> {
        let namespace_owner = self.namespace_owner(namespace).ok_or_else(|| {
            BuildError::single(crate::Diagnostic::hard_error(
                "source callable namespace has no semantic owner",
                Some(provenance.clone()),
            ))
        })?;
        let symbol = self.intern_symbol(namespace, namespace_owner, name, provenance.clone());
        self.symbol_backing_declarations
            .entry(symbol)
            .or_insert(backing_declaration);
        let placement = match closure.placement {
            NormClosurePlacement::InPlace => CallableOwnerPlacement::InPlace,
            NormClosurePlacement::Ordinary => CallableOwnerPlacement::Ordinary,
        };
        let callable_owner = self.owners.callable(
            namespace_owner,
            LocalCallableIdentity(self.next_callable),
            placement,
        );
        self.next_callable = self
            .next_callable
            .checked_add(1)
            .expect("semantic callable identity exhausted");

        // Canonical P1 normalization. The canonical P1 is
        // the single authority — see register_associated_call_entry doc.
        let canonical_view = canonical_function_object_view(
            outer_p1_explicit.as_ref(),
            &function_view,
            &body_entry_view,
            Some(closure),
            &provenance,
        )
        .map_err(BuildError::single)?;
        let return_position_view = elaborate_return_policy_pattern(
            closure
                .head
                .as_ref()
                .and_then(|head| head.returns.as_ref())
                .and_then(|slot| slot.policy.as_ref()),
            &canonical_view,
            provenance.clone(),
        )
        .map_err(BuildError::single)?
        .effective_view;

        let function_type = self.allocate_anonymous_type();
        let (function_pattern, pattern_scope) =
            self.allocate_pattern(callable_owner, provenance.clone());
        self.types.insert(
            function_type,
            SemanticTypeValue {
                id: function_type,
                pattern: function_pattern,
                provenance: provenance.clone(),
            },
        );
        self.pattern_types.insert(function_pattern, function_type);

        let function_value = self.allocate_value_id();
        self.materialize_val1_object(SemanticValueObject {
            id: function_value,
            type_value: function_type,
            pattern: function_pattern,
            place: ObjectPlaceId(0), // overwritten by materialize_val1_object
            policy: canonical_view.pair.clone(),
            mode: canonical_view.mode,
            namespace_visibility,
            payload: SemanticValuePayload::FunctionObject {
                backing_declaration,
            },
            provenance: provenance.clone(),
        });

        // The call entry is allocated via the unified terminal primitive so
        // it gets an independent FunctionItem type/pattern.  The call entry
        // is registered under the function object's pattern scope, not its
        // own.  This makes the call entry terminal (c.Val2 = ∅).
        let call_entry_value = self.allocate_terminal_call_entry(
            function_pattern,
            backing_declaration,
            name,
            "()",
            true,
            Some(namespace),
            Some(closure),
            None,
            None,
            callable_owner,
            function_type,
            canonical_view.clone(),
            body_entry_view.clone(),
            body_entry_view,
            return_position_view,
            namespace_visibility,
            OrdinaryCandidateRole::Ordinary,
            return_shape,
            CallablePrivilege::OrdinarySource,
            provenance.clone(),
        )?;
        let function_place = self
            .values
            .get(&function_value)
            .expect("materialized function object exists")
            .place;
        self.associate_existing_value_in_place(function_place, "()", call_entry_value)
            .expect("function object explicitly owns its terminal call entry");
        self.freeze_value_complete_type(function_value);
        self.symbols
            .get_mut(&symbol)
            .expect("interned semantic symbol exists")
            .sibling_vals
            .push(function_value);
        // Member_views.value_policy/pattern_policy must read
        // the same canonical P1 as SemanticValueObject.policy and
        // OrdinaryCallEntry.callable_value_policy. Previously this used
        // function_policy (the un-canonicalized outer P1), creating a split
        // between "object policy = canonical P1" and "member view = outer P1".
        self.symbols
            .get_mut(&symbol)
            .expect("interned semantic symbol exists")
            .member_views
            .push(PolicyResultEntry {
                value: Some(function_value),
                pattern: function_pattern,
                view: canonical_view,
            });
        self.backing_to_function_value
            .insert(backing_declaration, function_value);

        Ok(RegisteredCallable {
            symbol,
            function_value,
            function_type,
            function_pattern,
            pattern_scope,
            call_entry: call_entry_value,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn register_core_callable(
        &mut self,
        namespace: NamespaceNodeId,
        name: &str,
        backing_declaration: SymbolId,
        primitive: CoreMetaFunction,
        outer_p1_explicit: Option<ExplicitP1Selection>,
        return_shape: ReturnShape,
        function_view: PolicyView,
        body_entry_view: PolicyView,
        complete_result_view: PolicyView,
        namespace_visibility: Option<NamespaceVisibility>,
        provenance: Provenance,
    ) -> Result<RegisteredCallable, BuildError> {
        let namespace_owner = self.namespace_owner(namespace).ok_or_else(|| {
            BuildError::single(crate::Diagnostic::hard_error(
                "core callable namespace has no semantic owner",
                Some(provenance.clone()),
            ))
        })?;
        let symbol = self.intern_symbol(namespace, namespace_owner, name, provenance.clone());
        self.symbol_backing_declarations
            .entry(symbol)
            .or_insert(backing_declaration);
        let callable_owner = self.owners.callable(
            namespace_owner,
            LocalCallableIdentity(self.next_callable),
            CallableOwnerPlacement::Ordinary,
        );
        self.next_callable = self
            .next_callable
            .checked_add(1)
            .expect("semantic callable identity exhausted");

        // Canonical P1 normalization. Core callables have no
        // closure, so self_explicit is None. The caller passes
        // outer_p1_explicit=Some(&function_policy) since core callables
        // always have an explicit outer P1; the canonical P1 is therefore
        // the outer P1. Errors from the canonicalizer propagate rather
        // than being swallowed.
        let canonical_view = canonical_function_object_view(
            outer_p1_explicit.as_ref(),
            &function_view,
            &complete_result_view,
            None,
            &provenance,
        )
        .map_err(BuildError::single)?;

        let function_type = self.allocate_anonymous_type();
        let (function_pattern, pattern_scope) =
            self.allocate_pattern(callable_owner, provenance.clone());
        self.types.insert(
            function_type,
            SemanticTypeValue {
                id: function_type,
                pattern: function_pattern,
                provenance: provenance.clone(),
            },
        );
        self.pattern_types.insert(function_pattern, function_type);

        let function_value = self.allocate_value_id();
        self.materialize_val1_object(SemanticValueObject {
            id: function_value,
            type_value: function_type,
            pattern: function_pattern,
            place: ObjectPlaceId(0), // overwritten by materialize_val1_object
            policy: canonical_view.pair.clone(),
            mode: canonical_view.mode,
            namespace_visibility,
            payload: SemanticValuePayload::FunctionObject {
                backing_declaration,
            },
            provenance: provenance.clone(),
        });

        // The call entry is allocated via the unified terminal primitive so
        // it gets an independent FunctionItem type/pattern.  The call entry
        // is registered under the function object's pattern scope, not its
        // own.  This makes the call entry terminal (c.Val2 = ∅).
        let call_entry_value = self.allocate_terminal_call_entry(
            function_pattern,
            backing_declaration,
            name,
            "()",
            true,
            Some(namespace),
            None,
            Some(primitive),
            None,
            callable_owner,
            function_type,
            canonical_view.clone(),
            body_entry_view,
            complete_result_view,
            canonical_view.clone(),
            namespace_visibility,
            OrdinaryCandidateRole::Ordinary,
            return_shape,
            CallablePrivilege::BuiltinPrivileged,
            provenance.clone(),
        )?;
        let function_place = self
            .values
            .get(&function_value)
            .expect("materialized function object exists")
            .place;
        self.associate_existing_value_in_place(function_place, "()", call_entry_value)
            .expect("function object explicitly owns its terminal call entry");
        self.freeze_value_complete_type(function_value);
        let cell = self
            .symbols
            .get_mut(&symbol)
            .expect("interned semantic symbol exists");
        cell.sibling_vals.push(function_value);
        // Member_views read the same canonical P1 as
        // SemanticValueObject.policy and OrdinaryCallEntry.callable_value_policy.
        cell.member_views.push(PolicyResultEntry {
            value: Some(function_value),
            pattern: function_pattern,
            view: canonical_view,
        });
        self.backing_to_function_value
            .insert(backing_declaration, function_value);

        Ok(RegisteredCallable {
            symbol,
            function_value,
            function_type,
            function_pattern,
            pattern_scope,
            call_entry: call_entry_value,
        })
    }

    /// Contribute a function object as a sibling val of an existing
    /// cluster symbol.
    ///
    /// Creates an anonymous function type with its own Pattern scope.
    /// The function object's own type receives the associated Val2["()"]
    /// call entry — `()` is a terminal FunctionItem, not a recursive
    /// callable object. The function value is appended to the cluster's
    /// [`SemanticSymbolCell::sibling_vals`].
    ///
    /// This is NOT the path for `let ()` declarations. `let ()` writes
    /// to the current Pattern owner's Val2. This method constructs a
    /// standalone function object and adds it as a cluster sibling.
    #[allow(clippy::too_many_arguments)]
    pub fn contribute_function_object_to_cluster(
        &mut self,
        cluster_symbol: SemanticSymbolIdentity,
        declaration_namespace: NamespaceNodeId,
        backing_declaration: SymbolId,
        closure: &NormClosure,
        outer_p1_explicit: Option<ExplicitP1Selection>,
        function_view: PolicyView,
        body_entry_view: PolicyView,
        namespace_visibility: Option<NamespaceVisibility>,
        return_shape: ReturnShape,
        provenance: Provenance,
    ) -> Result<(SemanticValueId, SemanticValueId), BuildError> {
        let cell = self.symbols.get(&cluster_symbol).ok_or_else(|| {
            BuildError::single(crate::Diagnostic::hard_error(
                "contribute_function_object_to_cluster: cluster symbol not found",
                Some(provenance.clone()),
            ))
        })?;
        let namespace_owner = cell.declaration_owner;
        let declaration_name = cell.name.clone();
        let callable_owner = self.owners.callable(
            namespace_owner,
            LocalCallableIdentity(self.next_callable),
            CallableOwnerPlacement::Ordinary,
        );
        self.next_callable = self
            .next_callable
            .checked_add(1)
            .expect("semantic callable identity exhausted");

        // Canonical P1 normalization. The canonical P1 is the
        // single authority — see register_associated_call_entry doc.
        let canonical_view = canonical_function_object_view(
            outer_p1_explicit.as_ref(),
            &function_view,
            &body_entry_view,
            Some(closure),
            &provenance,
        )
        .map_err(BuildError::single)?;
        let return_position_view = elaborate_return_policy_pattern(
            closure
                .head
                .as_ref()
                .and_then(|head| head.returns.as_ref())
                .and_then(|slot| slot.policy.as_ref()),
            &canonical_view,
            provenance.clone(),
        )
        .map_err(BuildError::single)?
        .effective_view;

        let function_type = self.allocate_anonymous_type();
        let (function_pattern, _pattern_scope) =
            self.allocate_pattern(callable_owner, provenance.clone());
        self.types.insert(
            function_type,
            SemanticTypeValue {
                id: function_type,
                pattern: function_pattern,
                provenance: provenance.clone(),
            },
        );
        self.pattern_types.insert(function_pattern, function_type);

        let function_value = self.allocate_value_id();
        self.materialize_val1_object(SemanticValueObject {
            id: function_value,
            type_value: function_type,
            pattern: function_pattern,
            place: ObjectPlaceId(0), // overwritten by materialize_val1_object
            policy: canonical_view.pair.clone(),
            mode: canonical_view.mode,
            namespace_visibility,
            payload: SemanticValuePayload::FunctionObject {
                backing_declaration,
            },
            provenance: provenance.clone(),
        });

        // The call entry is allocated via the unified terminal primitive so
        // it gets an independent FunctionItem type/pattern.  The call entry
        // is registered under the function object's pattern scope, not its
        // own.  This makes the call entry terminal (c.Val2 = ∅).
        let call_entry_value = self.allocate_terminal_call_entry(
            function_pattern,
            backing_declaration,
            &declaration_name,
            "()",
            true,
            Some(declaration_namespace),
            Some(closure),
            None,
            None,
            callable_owner,
            function_type,
            canonical_view.clone(),
            body_entry_view.clone(),
            body_entry_view,
            return_position_view,
            namespace_visibility,
            OrdinaryCandidateRole::Ordinary,
            return_shape,
            CallablePrivilege::OrdinarySource,
            provenance.clone(),
        )?;
        let function_place = self
            .values
            .get(&function_value)
            .expect("materialized function object exists")
            .place;
        self.associate_existing_value_in_place(function_place, "()", call_entry_value)
            .expect("function object explicitly owns its terminal call entry");
        self.freeze_value_complete_type(function_value);

        let cell = self
            .symbols
            .get_mut(&cluster_symbol)
            .expect("cluster symbol exists");
        cell.sibling_vals.push(function_value);
        // Member_views read the same canonical P1 as
        // SemanticValueObject.policy and OrdinaryCallEntry.callable_value_policy.
        cell.member_views.push(PolicyResultEntry {
            value: Some(function_value),
            pattern: function_pattern,
            view: canonical_view,
        });
        self.backing_to_function_value
            .insert(backing_declaration, function_value);

        Ok((function_value, call_entry_value))
    }

    pub fn install_plain_value(
        &mut self,
        type_value: TypeValueId,
        policy: PolicyPair,
        provenance: Provenance,
    ) -> Option<SemanticValueId> {
        let pattern = self.type_value(type_value)?.pattern;
        let id = self.allocate_value_id();
        self.materialize_val1_object(SemanticValueObject {
            id,
            type_value,
            pattern,
            place: ObjectPlaceId(0), // overwritten by materialize_val1_object
            policy,
            mode: PolicyMode::Plain,
            namespace_visibility: None,
            payload: SemanticValuePayload::PlainValue,
            provenance,
        });
        Some(id)
    }

    /// Install a simple literal semantic value carrying its canonical
    /// content normal form.
    ///
    /// Argument normalization re-derives `Norm(<Val1, P, Val2>)` from the stored
    /// family + normalized content, so two materialized literal values with
    /// equal content merge to one canonical address instead of staying
    /// identity-opaque like [`Self::install_plain_value`] material.
    pub fn install_simple_literal_value(
        &mut self,
        type_value: TypeValueId,
        policy: PolicyPair,
        kind: NormLiteralKind,
        text: &str,
        provenance: Provenance,
    ) -> Option<SemanticValueId> {
        let pattern = self.type_value(type_value)?.pattern;
        let CanonicalNormForm::Object(CanonicalObjectNorm {
            val1: Some(CanonicalVal1Norm::Literal { family, normalized }),
            ..
        }) = canonical_literal_norm(kind, text)
        else {
            return None;
        };
        let id = self.allocate_value_id();
        self.materialize_val1_object(SemanticValueObject {
            id,
            type_value,
            pattern,
            place: ObjectPlaceId(0), // overwritten by materialize_val1_object
            policy,
            mode: PolicyMode::Plain,
            namespace_visibility: None,
            payload: SemanticValuePayload::SimpleLiteral { family, normalized },
            provenance,
        });
        Some(id)
    }

    /// Install an exact abstract semantic literal under `integer`, `real`, or
    /// `character`.  Concrete target Types are intentionally absent.
    pub fn install_abstract_literal_value(
        &mut self,
        family: crate::AbstractLiteralFamily,
        exact: crate::AbstractLiteralExactValue,
        type_value: TypeValueId,
        policy: PolicyPair,
        provenance: Provenance,
    ) -> Option<SemanticValueId> {
        let pattern = self.type_value(type_value)?.pattern;
        let (canonical_family, normalized) = match (&family, exact) {
            (
                crate::AbstractLiteralFamily::Integer,
                crate::AbstractLiteralExactValue::Integer(normalized),
            ) => (CanonicalLiteralFamily::Int, normalized),
            (
                crate::AbstractLiteralFamily::Real,
                crate::AbstractLiteralExactValue::Real(normalized),
            ) => (CanonicalLiteralFamily::Float, normalized),
            (
                crate::AbstractLiteralFamily::Character,
                crate::AbstractLiteralExactValue::Character(value),
            ) => (CanonicalLiteralFamily::String, value.to_string()),
            _ => return None,
        };
        let id = self.allocate_value_id();
        self.materialize_val1_object(SemanticValueObject {
            id,
            type_value,
            pattern,
            place: ObjectPlaceId(0),
            policy,
            mode: PolicyMode::Plain,
            namespace_visibility: None,
            payload: SemanticValuePayload::AbstractLiteral {
                family,
                canonical_family,
                normalized,
            },
            provenance,
        });
        Some(id)
    }

    /// Install one continuation-relative lifetime observation as an
    /// ordinary semantic value. The object's Place is only its current
    /// carrier residency; it is absent from the observed lifetime content
    /// and from `@` formation.
    pub fn install_lifetime_value(
        &mut self,
        lifetime: crate::LifetimeValue,
        type_value: TypeValueId,
        policy: PolicyPair,
        provenance: Provenance,
    ) -> Option<SemanticValueId> {
        let pattern = self.type_value(type_value)?.pattern;
        let id = self.allocate_value_id();
        self.materialize_val1_object(SemanticValueObject {
            id,
            type_value,
            pattern,
            place: ObjectPlaceId(0),
            policy,
            mode: PolicyMode::Plain,
            namespace_visibility: None,
            payload: SemanticValuePayload::LifetimeValue(lifetime),
            provenance,
        });
        Some(id)
    }

    /// Execute the value-realization half of one explicit
    /// abstract-to-concrete literal construction.  The caller has already
    /// selected the concrete constructor/target; this operation never
    /// changes the source abstract value or performs Policy migration.
    pub(crate) fn construct_abstract_literal_value(
        &mut self,
        source_abstract: SemanticValueId,
        target: &CompleteTypeValue,
        view: PolicyView,
        provenance: Provenance,
    ) -> Option<SemanticValueId> {
        let source = self.value(source_abstract)?.clone();
        let SemanticValuePayload::AbstractLiteral {
            canonical_family,
            normalized,
            ..
        } = source.payload
        else {
            return None;
        };
        let pattern = self.type_value(target.lookup_key)?.pattern;
        let id = self.allocate_value_id();
        self.materialize_val1_object(SemanticValueObject {
            id,
            type_value: target.lookup_key,
            pattern,
            place: ObjectPlaceId(0),
            policy: view.pair,
            mode: view.mode,
            namespace_visibility: None,
            payload: SemanticValuePayload::ConstructedLiteral {
                source_abstract,
                target_complete_type: target.whole,
                canonical_family,
                normalized,
            },
            provenance,
        });
        Some(id)
    }

    /// Bind semantic values under a source Symbol for the first time.
    ///
    /// An ordinary `let LHS = RHS` calls this exactly once.  Subsequent
    /// ordinary bindings to the same name are a conflict — ordinary `let`
    /// is not assignment and does not silently replace.
    ///
    /// Meta/build cluster construction uses [`contribute_cluster_member`]
    /// instead.
    pub fn bind_ordinary_new(
        &mut self,
        namespace: NamespaceNodeId,
        name: &str,
        views: &[PolicyResultEntry<crate::SemanticValueRef, PatternValueId>],
        provenance: Provenance,
    ) -> Result<SemanticSymbolIdentity, BindConflict> {
        let owner = self
            .namespace_owner(namespace)
            .ok_or(BindConflict::NoNamespaceOwner)?;
        if views
            .iter()
            .filter_map(|view| view.value)
            .any(|value| !self.values.contains_key(&value.id))
        {
            return Err(BindConflict::ValueNotInstalled);
        }
        let symbol = self.intern_symbol(namespace, owner, name, provenance);
        if let Some(cell) = self.symbols.get(&symbol) {
            if !cell.sibling_vals.is_empty()
                || !cell.member_views.is_empty()
                || cell.pure_p.is_some()
            {
                return Err(BindConflict::AlreadyBound {
                    name: name.to_string(),
                    identity: cell.identity,
                });
            }
        }
        // Ordinary `let =` binds a NEW object, so a pure-P view gives this
        // carrier its own writable Val2 place: `let U: type = T` must not be
        // able to write `T`'s members. `let ===` never reaches this path: the
        // not-yet-implemented lexical alias pass creates no carrier cell.
        let pure_p_member = views
            .iter()
            .find(|view| view.value.is_none())
            .map(|view| view.pattern)
            .map(|pattern| self.pure_p_member_for_carrier(symbol, pattern));
        // A binding copies the resident Object into a fresh destination
        // Place without allocating a new semantic value.  Place is a
        // horizontal coordinate and therefore does not participate in
        // Object equality or SemanticValue identity.
        let mut destination_places = BTreeMap::new();
        for value in views
            .iter()
            .filter_map(|view| view.value.map(|value| value.id))
        {
            if destination_places.contains_key(&value) {
                continue;
            }
            let Some(place) = self.allocate_binding_destination(value) else {
                return Err(BindConflict::ValueNotInstalled);
            };
            destination_places.insert(value, place);
        }
        let cell = self
            .symbols
            .get_mut(&symbol)
            .expect("interned semantic symbol exists");

        for view in views {
            let value = view.value.map(|value| value.id);
            if let Some(value) = value {
                if !cell.sibling_vals.contains(&value) {
                    cell.sibling_vals.push(value);
                }
                cell.sibling_places.insert(
                    value,
                    *destination_places
                        .get(&value)
                        .expect("every ordinary value receives a destination Place"),
                );
            } else {
                // Pure-P view (value=None, pattern=P):
                // set pure_p directly so SemanticWorld has the complete
                // fact after binding returns; declaration projections are
                // never rescanned (`sync_semantic_type_values` is deleted).
                // §8.2: projection records are not the semantic truth source.
                if cell.pure_p.is_none() {
                    cell.pure_p = pure_p_member;
                }
            }
            // Every bound semantic value carries a Pattern, including a
            // first-class complete type value.  Register its owning cluster
            // without rerooting an already-owned Pattern. Restricting this to
            // pure-P (`value=None`) views made `struct -> tau` impossible: the
            // later graph projection
            // observed an ownerless Pattern.
            self.pattern_clusters
                .entry(view.pattern)
                .or_insert(PatternClusterOwner::Installed(symbol));
            let binding_view = PolicyResultEntry {
                value,
                pattern: view.pattern,
                view: view.view.clone(),
            };
            if !cell.member_views.contains(&binding_view) {
                cell.member_views.push(binding_view);
            }
        }
        Ok(symbol)
    }

    /// Contribute member values to an open cluster construction.
    ///
    /// Each incoming view is recorded verbatim as a canonical member view
    /// via [`SemanticWorld::contribute_cluster_member_view`]: the complete
    /// value Policy and Pattern Policy travel with the member.  Nothing is
    /// silently dropped and no Policy is degraded to a bare id.
    pub fn contribute_cluster_member(
        &mut self,
        cluster: ClusterConstructionId,
        views: &[PolicyResultEntry<crate::SemanticValueRef, PatternValueId>],
    ) -> Option<()> {
        for view in views {
            self.contribute_cluster_member_view(
                cluster,
                PolicyResultEntry {
                    value: view.value.map(|value| value.id),
                    pattern: view.pattern,
                    view: view.view.clone(),
                },
            )?;
        }
        Some(())
    }

    /// Eagerly register a pattern's cluster ownership for a pure-P member
    /// that will be contributed later. This is needed when injection effects
    /// are processed before `contribute_cluster_member_view` runs, so the
    /// injection ownership check passes. Uses `or_insert` semantics: if the
    /// pattern already has an owner, this is a no-op.
    pub fn ensure_pattern_cluster_ownership(
        &mut self,
        pattern: PatternValueId,
        cluster: ClusterConstructionId,
    ) {
        self.pattern_clusters
            .entry(pattern)
            .or_insert(PatternClusterOwner::Open(cluster));
    }

    /// Force-set a pattern's cluster ownership to `Open(cluster)`,
    /// overriding any prior registration.  Intended for test harnesses
    /// that create a pattern via `register_type_symbol` (which marks it
    /// `Installed`) and then want to exercise injection into it as if the
    /// construction owned it.
    pub fn force_pattern_cluster_ownership(
        &mut self,
        pattern: PatternValueId,
        cluster: ClusterConstructionId,
    ) {
        self.pattern_clusters
            .insert(pattern, PatternClusterOwner::Open(cluster));
    }

    /// Record one complete member view on an open cluster construction.
    ///
    /// This is the single write entry for cluster construction content.
    /// Pure-P views (`value = None`) enforce the at-most-one-pure-P
    /// invariant and register the pattern's owning cluster when the
    /// pattern is newly generated; forwarded pre-existing patterns keep
    /// their original owner.  Identical views are deduplicated; distinct
    /// Policy views over the same member are kept as separate member views.
    pub fn contribute_cluster_member_view(
        &mut self,
        cluster: ClusterConstructionId,
        view: PolicyResultEntry<SemanticValueId, PatternValueId>,
    ) -> Option<()> {
        let construction = self.open_clusters.get_mut(&cluster)?;
        if construction.state != ConstructionState::Open {
            return None;
        }
        if view.value.is_none() {
            if let Some(existing) = derived_pure_p(&construction.member_views) {
                if existing != view.pattern {
                    return None;
                }
            }
        }
        if !construction.member_views.contains(&view) {
            construction.member_views.push(view.clone());
        }
        if view.value.is_none() {
            // Register the pattern's owning cluster only when the pattern
            // has no owner yet (it was generated by this construction).
            // A forwarded pre-existing PatternValue (e.g. a body forwarding
            // `uint8`) keeps its original cluster/Symbol owner: contributing
            // it as a member view must never reroot or re-own it.
            self.pattern_clusters
                .entry(view.pattern)
                .or_insert(PatternClusterOwner::Open(cluster));
        }
        Some(())
    }

    /// Replace all binding projections for an already-installed Symbol.
    ///
    /// This is an internal operation (not the source `let` semantics).
    /// Ordinary source bindings use [`bind_ordinary_new`]; cluster
    /// construction uses [`contribute_cluster_member`].
    pub fn replace_binding_projection(
        &mut self,
        namespace: NamespaceNodeId,
        name: &str,
        views: &[PolicyResultEntry<crate::SemanticValueRef, PatternValueId>],
        provenance: Provenance,
    ) -> Option<SemanticSymbolIdentity> {
        let owner = self.namespace_owner(namespace)?;
        if views
            .iter()
            .filter_map(|view| view.value)
            .any(|value| !self.values.contains_key(&value.id))
        {
            return None;
        }
        let symbol = self.intern_symbol(namespace, owner, name, provenance);
        let pure_p_member = views
            .iter()
            .find(|view| view.value.is_none())
            .map(|view| view.pattern)
            .map(|pattern| self.pure_p_member_for_carrier(symbol, pattern));
        let cell = self
            .symbols
            .get_mut(&symbol)
            .expect("interned semantic symbol exists");
        cell.member_views.clear();
        cell.sibling_vals.clear();
        cell.sibling_places.clear();
        cell.pure_p = None;
        let mut pure_p_patterns = Vec::new();
        for view in views {
            let value = view.value.map(|value| value.id);
            if let Some(value) = value {
                if !cell.sibling_vals.contains(&value) {
                    cell.sibling_vals.push(value);
                }
            } else {
                // Keep the derived caches strictly in sync with the
                // canonical member views: pure_p mirrors the first
                // value=None view, and the pattern-cluster owner is
                // registered exactly like `bind_ordinary_new`.
                if cell.pure_p.is_none() {
                    cell.pure_p = pure_p_member;
                }
                pure_p_patterns.push(view.pattern);
            }
            let binding_view = PolicyResultEntry {
                value,
                pattern: view.pattern,
                view: view.view.clone(),
            };
            if !cell.member_views.contains(&binding_view) {
                cell.member_views.push(binding_view);
            }
        }
        for pattern in pure_p_patterns {
            self.pattern_clusters
                .entry(pattern)
                .or_insert(PatternClusterOwner::Installed(symbol));
        }
        Some(symbol)
    }

    pub fn begin_cluster_construction(
        &mut self,
        authority: ConstructionAuthority,
        owner: SemanticOwnerId,
        provenance: Provenance,
    ) -> ClusterConstructionId {
        let id = ClusterConstructionId(self.next_cluster);
        self.next_cluster = self
            .next_cluster
            .checked_add(1)
            .expect("cluster construction id exhausted");
        // The open-window discipline is derived from the owning authority:
        // an ambient-scope construction lives in an ordinary window with
        // flow-segment coordinates; build-root and meta-invocation
        // constructions live in the conservative meta window.
        let window = match &authority {
            ConstructionAuthority::AmbientScope { .. } => {
                ConstructionWindow::Ordinary(OrdinaryOpenWindow {
                    creation_flow_segment: self.residual_runtime_epoch,
                    first_use_seen: false,
                    closed_by_fork_or_end: false,
                })
            }
            ConstructionAuthority::BuildRoot | ConstructionAuthority::MetaInvocation { .. } => {
                ConstructionWindow::Meta
            }
        };
        self.open_clusters.insert(
            id,
            OpenClusterConstruction {
                id,
                owner,
                authority,
                member_views: Vec::new(),
                state: ConstructionState::Open,
                window,
                use_observation: UseObservationKind::default(),
                provenance,
            },
        );
        id
    }

    /// Evaluate the contextual construction-authority judgment for a
    /// Pattern value.  `ConstructionState::Open` contributes only the
    /// `WindowLive` half; authority is resolved independently from the
    /// current evaluation frames.
    pub fn open_here(
        &self,
        target_pattern: PatternValueId,
        context: &ConstructionEvaluationContext,
    ) -> Result<OpenHereProof, OpenHereFailure> {
        if !self.patterns.contains_key(&target_pattern) {
            return Err(OpenHereFailure::UnknownPattern(target_pattern));
        }
        let Some(PatternClusterOwner::Open(construction_id)) =
            self.pattern_clusters.get(&target_pattern).copied()
        else {
            return Err(OpenHereFailure::NoLiveConstruction(target_pattern));
        };
        let construction = self
            .open_clusters
            .get(&construction_id)
            .ok_or(OpenHereFailure::NoLiveConstruction(target_pattern))?;
        if !self.construction_window_is_live(construction) {
            return Err(OpenHereFailure::WindowClosed(construction_id));
        }
        if !authority_matches_context(&construction.authority, context) {
            return Err(OpenHereFailure::AuthorityMismatch(construction_id));
        }
        Ok(OpenHereProof {
            construction: construction_id,
            target_pattern,
            authority: construction.authority.clone(),
        })
    }

    /// Establish the distinct member-creation judgment.  Freshness and
    /// operation-specific conflicts remain the responsibility of the
    /// selected creation operation; this proof only establishes that the
    /// current construction unit may attempt that operation here.
    pub fn can_create_member_here(
        &self,
        target_pattern: PatternValueId,
        context: &ConstructionEvaluationContext,
    ) -> Result<MemberCreationProof, OpenHereFailure> {
        self.open_here(target_pattern, context)
            .map(|open_here| MemberCreationProof { open_here })
    }

    fn construction_window_is_live(&self, construction: &OpenClusterConstruction) -> bool {
        if construction.state != ConstructionState::Open {
            return false;
        }
        match construction.window {
            ConstructionWindow::Meta => true,
            ConstructionWindow::Ordinary(window) => {
                !window.first_use_seen
                    && !window.closed_by_fork_or_end
                    && window.creation_flow_segment == self.residual_runtime_epoch
            }
        }
    }

    fn revalidate_open_here(&self, proof: &OpenHereProof) -> Result<(), OpenHereFailure> {
        let construction = self
            .open_clusters
            .get(&proof.construction)
            .ok_or(OpenHereFailure::NoLiveConstruction(proof.target_pattern))?;
        if self.pattern_clusters.get(&proof.target_pattern)
            != Some(&PatternClusterOwner::Open(proof.construction))
        {
            return Err(OpenHereFailure::NoLiveConstruction(proof.target_pattern));
        }
        if !self.construction_window_is_live(construction) {
            return Err(OpenHereFailure::WindowClosed(proof.construction));
        }
        if construction.authority != proof.authority {
            return Err(OpenHereFailure::AuthorityMismatch(proof.construction));
        }
        Ok(())
    }

    pub(crate) fn finalize_cluster_construction(
        &mut self,
        cluster: ClusterConstructionId,
    ) -> Option<SymbolConstructionValue> {
        let construction = self.open_clusters.remove(&cluster)?;
        Some(SymbolConstructionValue {
            identity: construction.id,
            member_views: construction.member_views,
            owner: construction.owner,
            provenance: construction.provenance,
        })
    }

    /// Finalize a type cluster construction at the construction boundary.
    ///
    /// S9 — finalization only closes the open construction and yields the
    /// accumulated member views as one `SymbolConstructionValue`.  Transport
    /// members (e.g. mut↔const transports) are ordinary sibling-member
    /// contributions with normal contribution semantics; they are injected
    /// through the contribution stream like any other member and are
    /// orthogonal to finalization.  Finalization never synthesizes members.
    ///
    /// This is the explicit result delivery / construction-boundary
    /// transition: observation, transformation, and injection never call
    /// it.  Both `Open` and `Frozen` (post-`UseForVal1`) constructions can
    /// be delivered; only an already-finalized construction returns `None`.
    pub fn finalize_type_cluster(
        &mut self,
        cluster: ClusterConstructionId,
    ) -> Option<SymbolConstructionValue> {
        let construction = self.open_clusters.get_mut(&cluster)?;
        if construction.state == ConstructionState::Finalized {
            return None;
        }
        construction.state = ConstructionState::Finalized;
        self.finalize_cluster_construction(cluster)
    }

    /// Observe the construction's Pattern: `OpenMeta --Observe(P)-->
    /// OpenMeta`.
    ///
    /// P and Val2 are exactly what a meta context observes, computes, and
    /// generates; observing them marks the construction as observed but
    /// never changes its state.  Returns the derived pure-P Pattern when
    /// the type member already exists.
    pub fn observe_cluster_pattern(
        &mut self,
        cluster: ClusterConstructionId,
    ) -> Option<PatternValueId> {
        let construction = self.open_clusters.get_mut(&cluster)?;
        construction
            .use_observation
            .has_been_observed_or_transformed = true;
        derived_pure_p(&construction.member_views)
    }

    /// Use the constructed type to generate a Val1: `OpenMeta
    /// --UseForVal1--> Frozen`.
    ///
    /// In the meta window this is the only transition that freezes an
    /// open construction (ordinary windows additionally freeze on first
    /// semantic use and residual-runtime fork/end; see
    /// [`ConstructionWindow`]).  After it, member contribution and Val2
    /// injection are rejected; boundary delivery
    /// (`finalize_type_cluster`) stays legal.
    ///
    /// This also makes Pattern injection and ordinary value injection
    /// disjoint. If an injected `Val1 × P × Val2` has this constructed type
    /// as its own `P × Val2`, producing that Val1 necessarily performs
    /// `UseForVal1` first. The type is therefore Frozen before injection can
    /// be attempted; it cannot simultaneously receive the value as a new
    /// Pattern contribution.
    pub fn use_cluster_for_val1(&mut self, cluster: ClusterConstructionId) -> Option<()> {
        let construction = self.open_clusters.get_mut(&cluster)?;
        if construction.state == ConstructionState::Finalized {
            return None;
        }
        construction.use_observation.has_been_used_for_val1 = true;
        construction.state = ConstructionState::Frozen;
        Some(())
    }

    /// Current residual-runtime flow segment coordinate.
    pub fn residual_runtime_epoch(&self) -> ResidualRuntimeEpoch {
        self.residual_runtime_epoch
    }

    /// The residual runtime serial flow forked or a serial segment ended.
    ///
    /// Advances the flow-segment coordinate and freezes every still-open
    /// ordinary-window construction created in an earlier segment: an
    /// ambient ordinary construction never survives past the end or fork
    /// of the residual runtime flow it was created in.  Meta windows are
    /// untouched — a meta construction transaction spans static control
    /// flow freely.
    ///
    /// Wiring this event to real source-level control-flow analysis is
    /// future work (registered in `spec/planning/open-questions.md`);
    /// today the event is raised explicitly by the evaluation driver.
    pub fn note_residual_runtime_fork_or_end(&mut self) {
        self.residual_runtime_epoch = ResidualRuntimeEpoch(
            self.residual_runtime_epoch
                .0
                .checked_add(1)
                .expect("residual runtime epoch exhausted"),
        );
        let boundary = self.residual_runtime_epoch;
        for construction in self.open_clusters.values_mut() {
            if construction.state != ConstructionState::Open {
                continue;
            }
            if let ConstructionWindow::Ordinary(window) = &mut construction.window {
                if window.creation_flow_segment < boundary {
                    window.closed_by_fork_or_end = true;
                    construction.state = ConstructionState::Frozen;
                }
            }
        }
    }

    /// A purely static (compile-evaluated) branch was taken.
    ///
    /// Deliberately a no-op: compile-only branching is transparent to
    /// both window kinds.  It neither advances the residual-runtime
    /// coordinate nor closes any window.
    pub fn note_compile_only_branch(&mut self) {}

    /// First semantic use of the constructed type outside its own
    /// construction stream.
    ///
    /// Ordinary window: `Open --FirstUse--> Frozen`.  Meta window: a use
    /// that does not produce a Val1 is an observation and keeps the
    /// window open (`use_cluster_for_val1` is the meta freeze).  Returns
    /// `None` when the construction does not exist or was already
    /// delivered.
    pub fn note_first_semantic_use(&mut self, cluster: ClusterConstructionId) -> Option<()> {
        let construction = self.open_clusters.get_mut(&cluster)?;
        if construction.state == ConstructionState::Finalized {
            return None;
        }
        match &mut construction.window {
            ConstructionWindow::Ordinary(window) => {
                window.first_use_seen = true;
                construction.state = ConstructionState::Frozen;
            }
            ConstructionWindow::Meta => {
                construction
                    .use_observation
                    .has_been_observed_or_transformed = true;
            }
        }
        Some(())
    }

    /// Contribute one evaluated pure Pattern resident
    /// (`null × P × Val2`) to a still-open named Pattern layer.
    ///
    /// # Privilege boundary
    ///
    /// **PRIVILEGE**: Only callable from `struct` inline construction path
    /// and (future) `inject` built-in meta function. Ordinary navigated
    /// `let f::t = expr` MUST NOT reach this method — it must use
    /// [`inject_associated_type_member`] instead, which installs the type
    /// as a Val2 member without entering the target Pattern.
    ///
    /// Registering a Val2 member into the Pattern canonical structure is a
    /// privileged operation that changes the type's structural identity.
    /// Only `struct` and `inject` hold the authority to establish the
    /// "P normal-form node — Val2 capability" registration edge.
    ///
    /// `local_navigation` is completed against the target Pattern's own
    /// complete root navigation.  The stored normal form contains only that
    /// completed name and the resident value: inline versus later injection
    /// and inherited versus explicit spelling are erased.  Replaying an
    /// equal contribution is idempotent; a different resident at the same
    /// complete navigation is a construction conflict.
    pub fn extend_pattern_value(
        &self,
        open_here: &OpenHereProof,
        local_navigation: crate::CanonicalFullNavigation,
        resident: CanonicalPatternValue,
        provenance: Provenance,
    ) -> Result<CanonicalPatternValue, crate::Diagnostic> {
        self.revalidate_open_here(open_here).map_err(|failure| {
            crate::Diagnostic::hard_error(
                format!("Pattern extend requires OpenHere in the current context: {failure:?}"),
                Some(provenance.clone()),
            )
        })?;
        let target_pattern = open_here.target_pattern;
        let mut normalized = self
            .pattern_structural_norms
            .get(&target_pattern)
            .cloned()
            .ok_or_else(|| {
                crate::Diagnostic::hard_error(
                    "Pattern extend target has no structural Pattern normal form",
                    Some(provenance.clone()),
                )
            })?;
        let CanonicalPatternValue::NamedPattern { navigation, body } = &mut normalized else {
            return Err(crate::Diagnostic::hard_error(
                "Pattern extend requires a named target Pattern layer",
                Some(provenance),
            ));
        };
        let complete_navigation = crate::CanonicalFullNavigation::new(
            local_navigation
                .components()
                .iter()
                .cloned()
                .chain(navigation.components().iter().cloned()),
        );
        let CanonicalPatternValue::UnorderedLayer(entries) = body.as_mut() else {
            return Err(crate::Diagnostic::hard_error(
                "Pattern extend requires an order-insensitive named Pattern body",
                Some(provenance),
            ));
        };
        if let Some(existing) = entries.get(&complete_navigation) {
            if existing == &resident {
                return Ok(normalized);
            }
            return Err(crate::Diagnostic::hard_error(
                format!(
                    "Pattern extend conflict: `{}` already carries a different Pattern resident",
                    complete_navigation.components().join("::")
                ),
                Some(provenance),
            ));
        }
        entries.insert(complete_navigation, resident);
        Ok(normalized)
    }

    /// Place-level structural injection: read the current Pattern value,
    /// perform the pure `extend`, then commit one explicit writable update.
    /// Open construction authority and writability are independent inputs.
    pub fn inject_extended_pattern_value(
        &mut self,
        open_here: &OpenHereProof,
        local_navigation: crate::CanonicalFullNavigation,
        resident: CanonicalPatternValue,
        writable: &WritableContext,
        provenance: Provenance,
    ) -> Result<CanonicalPatternValue, crate::Diagnostic> {
        let target_pattern = open_here.target_pattern;
        let place = self.pattern_place(target_pattern).ok_or_else(|| {
            crate::Diagnostic::hard_error(
                "Pattern inject target has no carrier place",
                Some(provenance.clone()),
            )
        })?;
        if !writable.place_is_writable(place) {
            return Err(crate::Diagnostic::hard_error(
                "Pattern inject requires an independent Writable grant for the target place",
                Some(provenance),
            ));
        }
        let extended =
            self.extend_pattern_value(open_here, local_navigation, resident, provenance.clone())?;
        self.pattern_structural_norms
            .insert(target_pattern, extended.clone());
        let construction = self
            .open_clusters
            .get_mut(&open_here.construction)
            .expect("OpenHere proof was revalidated before commit");
        construction
            .use_observation
            .has_been_observed_or_transformed = true;
        Ok(extended)
    }

    /// Install a `null × P × Val2` pure type Object as an **associated type** in
    /// the target type member's object-level Val2, without modifying the
    /// target's canonical Pattern structure.
    ///
    /// ## Privilege boundary
    ///
    /// This is the ordinary navigated `let f::t = expr` path when `expr` is a
    /// pure type Object. It does **not** register `f` into `t`'s Pattern
    /// canonical norm — that privilege belongs exclusively to `struct` inline
    /// construction (which calls [`inject_pattern_value_member`]) and the
    /// future `inject` built-in meta function.
    ///
    /// ## Recursive Val2 Symbol ontology
    ///
    /// Val2 is not a name → raw value list map; it stays a recursive Symbol
    /// world: `Val2(T_t)[f] = C_f`. The injected pure type Object
    /// `x = ⟨Val1 = ∅, P_x, Val2_x⟩` becomes the single pure-P member of
    /// that associated Symbol:
    ///
    /// ```text
    /// x ∉ Members(C_t)      — never a member of the HOST cluster
    /// x  = PureP(C_f)       — the pure-P member of its own Val2 Symbol
    /// C_f ∈ Val2(T_t)       — reached through the host type member's place
    ///
    /// P(C_f) = P(P_x) || P(w_1) || ... || P(w_m)
    /// ```
    ///
    /// so `C_f` obeys the ordinary cluster Policy disjunction over its own
    /// members — same-named associated vals are its sibling vals `w_i`.
    /// `AssociatedType ⊄ target ClusterMember`, never the unqualified
    /// `AssociatedType ⊄ ClusterMember`. The host stays invariant:
    ///
    /// ```text
    /// Δ host cluster pure_p / sibling_vals / member_views    = ∅
    /// Δ host type member Policy                             = ∅
    /// Δ host derived cluster Policy                         = ∅
    /// Δ host Pattern canonical norm                         = ∅
    /// Δ Val2 of the host's ordinary same-named value members = ∅
    /// ```
    ///
    /// `view` is the binding-level pure-P member view (`view.value` must be
    /// `None`): the RHS complete view already restricted by the binding's
    /// written P1, exactly as on the ordinary value path — a type does not
    /// get a second P1 discipline for lacking a Val1. It is installed as
    /// `C_f`'s pure-P member view and is the Policy authority for this
    /// binding. The ObjectPlace entry carries only the CoreTypeProjection transport
    /// reference; that globally reused adapter is never a binding-Policy
    /// carrier. Exposure of `t::f` composes per layer at lookup
    /// (`Expose(T_t, φ) ∧ Expose(C_f member, φ)`; see
    /// [`Self::associated_member_views_for_host`]), never at installation.
    ///
    /// One Symbol carries at most one pure P, so a same-named different
    /// associated type is a construction conflict; the equal contribution
    /// replays idempotently.
    pub fn associated_type_member_is_replay(
        &self,
        target_pattern: PatternValueId,
        member_name: &str,
        view: &PolicyResultEntry<SemanticValueId, PatternValueId>,
        member_type_value: TypeValueId,
    ) -> bool {
        if view.value.is_some() || self.type_for_pattern(view.pattern) != Some(member_type_value) {
            return false;
        }
        let Some(symbol) = self.associated_symbol_for_pattern(target_pattern, member_name) else {
            return false;
        };
        self.symbol(symbol).is_some_and(|cell| {
            cell.pure_p_pattern() == Some(view.pattern) && cell.member_views.contains(view)
        })
    }

    pub fn create_associated_type_member(
        &mut self,
        creation: &MemberCreationProof,
        member_name: &str,
        view: PolicyResultEntry<SemanticValueId, PatternValueId>,
        member_type_value: TypeValueId,
        provenance: Provenance,
    ) -> Result<(), crate::Diagnostic> {
        if view.value.is_some() {
            return Err(crate::Diagnostic::hard_error(
                "associated-type injection requires a pure-P member view (Val1 = ∅)",
                Some(provenance),
            ));
        }
        self.revalidate_open_here(creation.open_here()).map_err(|failure| {
            crate::Diagnostic::hard_error(
                format!("associated-type member creation requires current construction authority: {failure:?}"),
                Some(provenance.clone()),
            )
        })?;
        let cluster = creation.open_here.construction;
        let target_pattern = creation.open_here.target_pattern;
        let member_pattern = view.pattern;
        let construction = self
            .open_clusters
            .get_mut(&cluster)
            .expect("member-creation proof names a live construction");
        construction
            .use_observation
            .has_been_observed_or_transformed = true;

        let scope_id = self
            .pattern(target_pattern)
            .ok_or_else(|| {
                crate::Diagnostic::hard_error(
                    "meta associated-type injection target Pattern is not registered",
                    Some(provenance.clone()),
                )
            })?
            .scope;
        let place_id = *self
            .pattern_places
            .get(&target_pattern)
            .expect("injection target pattern has an allocated place");

        // `Val2(T_t)[f] = C_f`: the member name first resolves to one
        // recursive ClusterSymbol on the target object's own place. The first
        // contributing event allocates it; same-named ordinary vals join the
        // very same Symbol as sibling vals.
        let scope_owner = self
            .scopes
            .get(&scope_id)
            .expect("injection target Pattern scope exists")
            .owner;
        let associated_symbol = match self.associated_symbol_in_place(place_id, member_name) {
            Some(existing) => existing,
            None => {
                let fresh =
                    self.allocate_scope_local_symbol(scope_owner, member_name, provenance.clone());
                self.associate_existing_symbol_in_place(place_id, member_name, fresh)
                    .expect("scope-local Symbol was just allocated");
                fresh
            }
        };

        // One Symbol carries at most one pure P: equal material is an
        // idempotent replay, a different associated type is a construction
        // conflict. Same-named sibling vals of `C_f` are untouched.
        let installed_pure_p = self
            .symbols
            .get(&associated_symbol)
            .expect("associated Val2 ClusterSymbol exists")
            .pure_p_pattern();
        if let Some(installed) = installed_pure_p {
            if installed != member_pattern {
                return Err(crate::Diagnostic::hard_error(
                    format!(
                        "associated-type construction conflict: `{member_name}` already carries a different associated type"
                    ),
                    Some(provenance),
                ));
            }
        }

        // Transport reference only: the object-level Val2 container indexes
        // by `SemanticValueId`, so a pure type Object needs a CoreTypeProjection
        // adapter to be navigable from the place. The adapter is globally
        // reused per TypeValue and is NEVER the binding-Policy carrier — the
        // member view installed below is.
        let type_value_id = self.find_or_install_core_type_projection_value(
            member_type_value,
            member_pattern,
            view.view.pair.clone(),
            provenance.clone(),
        );

        // The semantic fact: `x = PureP(C_f)` with this binding's own member
        // view as its Policy authority.  `C_f` is a distinct object, so it
        // receives its own writable Val2 place unless it declares the Pattern
        // itself.
        let member = self.pure_p_member_for_carrier(associated_symbol, member_pattern);
        let cell = self
            .symbols
            .get_mut(&associated_symbol)
            .expect("associated Val2 ClusterSymbol exists");
        cell.pure_p = Some(member);
        if !cell.member_views.contains(&view) {
            cell.member_views.push(view);
        }

        // Object-level navigation entry on the host type member's place.
        self.associate_existing_value_in_place(place_id, member_name, type_value_id)
            .expect("associated type target and transport value exist");
        self.admit_direct_type_member(
            target_pattern,
            target_pattern,
            member_name,
            TypeMemberFacet::PureP,
            type_value_id,
        )
    }
    /// Install an already-evaluated ordinary value into one associated Val2
    /// ClusterSymbol.  The value keeps its own `Val1 × P × Val2` identity and
    /// its binding view; its Pattern is never merged into the target Pattern.
    pub fn associated_value_member_is_replay(
        &self,
        target_pattern: PatternValueId,
        member_name: &str,
        view: &PolicyResultEntry<SemanticValueId, PatternValueId>,
    ) -> bool {
        let Some(value) = view.value else {
            return false;
        };
        let Some(symbol) = self.associated_symbol_for_pattern(target_pattern, member_name) else {
            return false;
        };
        self.symbol(symbol).is_some_and(|cell| {
            cell.sibling_vals.contains(&value) && cell.member_views.contains(view)
        })
    }

    pub fn create_associated_existing_value_member(
        &mut self,
        creation: &MemberCreationProof,
        member_name: &str,
        view: PolicyResultEntry<SemanticValueId, PatternValueId>,
        provenance: Provenance,
    ) -> Result<(), crate::Diagnostic> {
        self.revalidate_open_here(creation.open_here()).map_err(|failure| {
            crate::Diagnostic::hard_error(
                format!("associated-value member creation requires current construction authority: {failure:?}"),
                Some(provenance.clone()),
            )
        })?;
        let cluster = creation.open_here.construction;
        let target_pattern = creation.open_here.target_pattern;
        let Some(value) = view.value else {
            return Err(crate::Diagnostic::hard_error(
                "ordinary associated-Val2 injection requires an evaluated Val1",
                Some(provenance),
            ));
        };
        if !self.values.contains_key(&value) {
            return Err(crate::Diagnostic::hard_error(
                "ordinary associated-Val2 injection value is not installed",
                Some(provenance),
            ));
        }
        let construction = self
            .open_clusters
            .get_mut(&cluster)
            .expect("member-creation proof names a live construction");
        construction
            .use_observation
            .has_been_observed_or_transformed = true;

        let scope_id = self
            .pattern(target_pattern)
            .ok_or_else(|| {
                crate::Diagnostic::hard_error(
                    "meta Val2 injection target Pattern is not registered",
                    Some(provenance.clone()),
                )
            })?
            .scope;
        let scope_owner = self
            .scopes
            .get(&scope_id)
            .expect("injection target Pattern scope exists")
            .owner;
        let place_id = *self
            .pattern_places
            .get(&target_pattern)
            .expect("injection target pattern has an allocated place");
        let cluster_symbol = match self.associated_symbol_in_place(place_id, member_name) {
            Some(existing) => existing,
            None => {
                let fresh =
                    self.allocate_scope_local_symbol(scope_owner, member_name, provenance.clone());
                self.associate_existing_symbol_in_place(place_id, member_name, fresh)
                    .expect("scope-local Symbol was just allocated");
                fresh
            }
        };
        let cell = self
            .symbols
            .get_mut(&cluster_symbol)
            .expect("associated value ClusterSymbol exists");
        if !cell.sibling_vals.contains(&value) {
            cell.sibling_vals.push(value);
        }
        if !cell.member_views.contains(&view) {
            cell.member_views.push(view);
        }
        self.associate_existing_value_in_place(place_id, member_name, value)
            .expect("associated value target and member value exist");
        self.admit_direct_type_member(
            target_pattern,
            target_pattern,
            member_name,
            TypeMemberFacet::Value,
            value,
        )
    }

    /// Inject a function-object member into the constructed type's
    /// associated scope: `OpenMeta --Inject--> OpenMeta`.
    ///
    /// `let f::t = fn_expr;` in a meta body, when the RHS evaluates to an
    /// ordinary value, contributes the full `Val1 × P × Val2` value through
    /// the recursive
    /// ClusterSymbol substrate: the target object's place-level
    /// `associated_symbols[member_name]` names one ClusterSymbol
    /// `f`, and each injecting declaration event adds one fresh sibling
    /// function-object value to that symbol. This is distinct from injecting
    /// a Pattern value (`null × P × Val2`), whose P participates in Pattern
    /// normalization. If the ordinary value's own `P × Val2` is the target
    /// type, its Val1 construction has already frozen that target and this
    /// operation rejects it; only a value of another type can remain an
    /// ordinary associated-Val2 injection while the target stays Open.
    /// Ordinary value injection transforms Val2
    /// without freezing: the construction stays open until boundary
    /// delivery.
    ///
    /// The injected value's identity is the declaration event, never the
    /// member name: replaying the same canonical meta instance re-finds
    /// each event's value (equal declaration material is an idempotent
    /// reuse, different material is a construction conflict), while two
    /// distinct declaration events that both write `f` are two sibling
    /// vals of the one ClusterSymbol `f`.  A frozen (`UseForVal1`) or
    /// delivered construction rejects injection.
    ///
    /// `backing_declaration` is NOT identity material.  It is the outer
    /// meta function's declaration Symbol, carried only as the A-stage
    /// declaration-environment transport on the terminal call entry until
    /// the semantic name index fully replaces the graph-backed lookup.
    #[allow(clippy::too_many_arguments)]
    pub fn replay_associated_function_member(
        &self,
        cluster: ClusterConstructionId,
        member_name: &str,
        construction_event: u32,
        closure: &NormClosure,
        outer_p1_explicit: Option<&ExplicitP1Selection>,
        function_view: &PolicyView,
        complete_result_view: &PolicyView,
        provenance: Provenance,
    ) -> Result<Option<SemanticValueId>, crate::Diagnostic> {
        let Some(construction) = self.open_clusters.get(&cluster) else {
            return Ok(None);
        };
        let ConstructionAuthority::MetaInvocation { canonical_key, .. } = &construction.authority
        else {
            return Ok(None);
        };
        let identity = InjectedValueIdentity {
            enclosing_meta: canonical_key.callable,
            canonical_arguments: canonical_key.arguments,
            construction_event,
        };
        let Some(record) = self.injected_members.get(&identity) else {
            return Ok(None);
        };
        let canonical_view = canonical_function_object_view(
            outer_p1_explicit,
            function_view,
            complete_result_view,
            Some(closure),
            &provenance,
        )?;
        if record.member_name == member_name
            && record.declaration == *closure
            && record.canonical_view == canonical_view
        {
            Ok(Some(record.value))
        } else {
            Err(crate::Diagnostic::hard_error(
                format!(
                    "meta construction conflict: the canonical meta instance replays injected member `{member_name}` with different declaration material"
                ),
                Some(provenance),
            ))
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_associated_function_member(
        &mut self,
        creation: &MemberCreationProof,
        member_name: &str,
        construction_event: u32,
        backing_declaration: SymbolId,
        closure: &NormClosure,
        outer_p1_explicit: Option<&ExplicitP1Selection>,
        function_view: &PolicyView,
        body_entry_view: PolicyView,
        return_shape: ReturnShape,
        provenance: Provenance,
    ) -> Result<SemanticValueId, crate::Diagnostic> {
        self.revalidate_open_here(creation.open_here()).map_err(|failure| {
            crate::Diagnostic::hard_error(
                format!("associated function member creation requires current construction authority: {failure:?}"),
                Some(provenance.clone()),
            )
        })?;
        let cluster = creation.open_here.construction;
        let target_pattern = creation.open_here.target_pattern;
        let construction = self
            .open_clusters
            .get_mut(&cluster)
            .expect("member-creation proof names a live construction");
        construction
            .use_observation
            .has_been_observed_or_transformed = true;
        let construction_owner = construction.owner;
        // B3: the injected value's identity is the canonical meta
        // instance's structural coordinates plus the source declaration
        // event — never the member name and never a digest.
        let identity = match &construction.authority {
            ConstructionAuthority::MetaInvocation { canonical_key, .. } => InjectedValueIdentity {
                enclosing_meta: canonical_key.callable,
                canonical_arguments: canonical_key.arguments,
                construction_event,
            },
            _ => {
                return Err(crate::Diagnostic::hard_error(
                    "meta Val2 injection requires a canonical meta invocation construction authority",
                    Some(provenance.clone()),
                ));
            }
        };

        // Canonical P1 normalization — the injected member's
        // binding P1 and its written-self P1 reconcile into one canonical
        // P1, exactly like a namespace-level function object declaration.
        // Computed before the replay check so the replay comparison covers
        // the complete declaration material.
        let canonical_view = canonical_function_object_view(
            outer_p1_explicit,
            function_view,
            &body_entry_view,
            Some(closure),
            &provenance,
        )?;
        let return_position_view = elaborate_return_policy_pattern(
            closure
                .head
                .as_ref()
                .and_then(|head| head.returns.as_ref())
                .and_then(|slot| slot.policy.as_ref()),
            &canonical_view,
            provenance.clone(),
        )?
        .effective_view;

        // Canonical meta instance replay: same
        // declaration-event identity + same declaration material →
        // idempotent reuse; same identity + different material →
        // construction conflict — never a stacked duplicate or a silent
        // overwrite.  Two distinct events with the same member name never
        // collide here: they are distinct identities and become two
        // sibling vals of one ClusterSymbol.
        if let Some(record) = self.injected_members.get(&identity) {
            if record.member_name == member_name
                && record.declaration == *closure
                && record.canonical_view == canonical_view
            {
                return Ok(record.value);
            }
            return Err(crate::Diagnostic::hard_error(
                format!(
                    "meta construction conflict: the canonical meta instance replays injected member `{member_name}` with a different member name, function body, or binding policy"
                ),
                Some(provenance.clone()),
            ));
        }

        let scope_id = self
            .pattern(target_pattern)
            .ok_or_else(|| {
                crate::Diagnostic::hard_error(
                    "meta Val2 injection target Pattern is not registered",
                    Some(provenance.clone()),
                )
            })?
            .scope;
        // B3: the member name names one recursive ClusterSymbol on the target
        // object's own place.  The first injecting event allocates it;
        // later events (and distinct same-name events) reuse it and add
        // sibling vals.  The symbol is Pattern-scope-local: it never
        // enters the (namespace, name) symbol index.
        let scope_owner = self
            .scopes
            .get(&scope_id)
            .expect("injection target Pattern scope exists")
            .owner;
        let place_id = *self
            .pattern_places
            .get(&target_pattern)
            .expect("injection target pattern has an allocated place");
        let cluster_symbol = match self.associated_symbol_in_place(place_id, member_name) {
            Some(existing) => existing,
            None => {
                let fresh =
                    self.allocate_scope_local_symbol(scope_owner, member_name, provenance.clone());
                self.associate_existing_symbol_in_place(place_id, member_name, fresh)
                    .expect("scope-local Symbol was just allocated");
                fresh
            }
        };

        let callable_owner = self.owners.callable(
            construction_owner,
            LocalCallableIdentity(self.next_callable),
            CallableOwnerPlacement::Ordinary,
        );
        self.next_callable = self
            .next_callable
            .checked_add(1)
            .expect("semantic callable identity exhausted");

        // Canonical P1 was already normalized above, before
        // the replay check.

        let function_type = self.allocate_anonymous_type();
        let (function_pattern, _pattern_scope) =
            self.allocate_pattern(callable_owner, provenance.clone());
        self.types.insert(
            function_type,
            SemanticTypeValue {
                id: function_type,
                pattern: function_pattern,
                provenance: provenance.clone(),
            },
        );
        self.pattern_types.insert(function_pattern, function_type);

        let function_value = self.allocate_value_id();
        self.materialize_val1_object(SemanticValueObject {
            id: function_value,
            type_value: function_type,
            pattern: function_pattern,
            place: ObjectPlaceId(0), // overwritten by materialize_val1_object
            policy: canonical_view.pair.clone(),
            mode: canonical_view.mode,
            namespace_visibility: None,
            payload: SemanticValuePayload::InjectedFunctionObject { identity },
            provenance: provenance.clone(),
        });
        let record_view = canonical_view.clone();
        let call_entry_value = self
            .allocate_terminal_call_entry(
                function_pattern,
                backing_declaration,
                member_name,
                "()",
                true,
                None,
                Some(closure),
                None,
                None,
                callable_owner,
                function_type,
                canonical_view,
                body_entry_view.clone(),
                body_entry_view,
                return_position_view,
                None,
                OrdinaryCandidateRole::Ordinary,
                return_shape,
                CallablePrivilege::OrdinarySource,
                provenance.clone(),
            )
            .map_err(|error| {
                error.diagnostics.into_iter().next().unwrap_or_else(|| {
                    crate::Diagnostic::hard_error(
                        "meta Val2 injection call entry allocation failed",
                        Some(provenance.clone()),
                    )
                })
            })?;
        let function_place = self
            .values
            .get(&function_value)
            .expect("materialized injected function object exists")
            .place;
        self.associate_existing_value_in_place(function_place, "()", call_entry_value)
            .expect("injected function object explicitly owns its terminal call entry");
        self.freeze_value_complete_type(function_value);

        // B3: the injected value is a fresh sibling val of the member-name
        // ClusterSymbol, with its Policy view read from the same canonical
        // P1 as the value object itself.  The scope's associated Val2
        // bucket stays the invoke-side read surface over the same ids.
        let cell = self
            .symbols
            .get_mut(&cluster_symbol)
            .expect("injected cluster symbol exists");
        cell.sibling_vals.push(function_value);
        cell.member_views.push(PolicyResultEntry {
            value: Some(function_value),
            pattern: function_pattern,
            view: record_view.clone(),
        });
        self.associate_existing_value_in_place(place_id, member_name, function_value)
            .expect("injection target and function value exist");
        self.injected_members.insert(
            identity,
            InjectedMemberRecord {
                value: function_value,
                member_name: member_name.to_string(),
                declaration: closure.clone(),
                canonical_view: record_view,
            },
        );
        self.admit_direct_type_member(
            target_pattern,
            target_pattern,
            member_name,
            TypeMemberFacet::Value,
            function_value,
        )?;
        Ok(function_value)
    }

    pub fn upgrade_cluster_owner(
        &mut self,
        cluster: ClusterConstructionId,
        symbol: SemanticSymbolIdentity,
    ) -> Option<()> {
        for (_pattern, owner) in self.pattern_clusters.iter_mut() {
            if *owner == PatternClusterOwner::Open(cluster) {
                *owner = PatternClusterOwner::Installed(symbol);
                return Some(());
            }
        }
        None
    }

    pub fn open_cluster(&self, cluster: ClusterConstructionId) -> Option<&OpenClusterConstruction> {
        self.open_clusters.get(&cluster)
    }

    fn intern_symbol(
        &mut self,
        namespace: NamespaceNodeId,
        owner: SemanticOwnerId,
        name: &str,
        provenance: Provenance,
    ) -> SemanticSymbolIdentity {
        if !self.owner_namespace_nodes.contains_key(&namespace) {
            self.ensure_owner_namespace_node(
                namespace,
                None,
                format!("<namespace:{}>", namespace.as_u64()),
            )
            .expect("semantic namespace owner has a typed namespace node");
        }
        if let Some(existing) = self.symbol_in_namespace(namespace, name) {
            return existing.identity;
        }
        let next = self.local_symbol_counters.entry(owner).or_default();
        let identity = SemanticSymbolIdentity {
            owner,
            local: LocalSymbolIdentity(*next),
        };
        *next = next
            .checked_add(1)
            .expect("semantic symbol identity exhausted");
        self.symbols.insert(
            identity,
            SemanticSymbolCell {
                identity,
                name: name.to_string(),
                declaration_owner: owner,
                namespace_node: Some(namespace),
                pure_p: None,
                sibling_vals: Vec::new(),
                sibling_places: BTreeMap::new(),
                member_views: Vec::new(),
                provenance,
            },
        );
        let node = self
            .owner_namespace_nodes
            .get(&namespace)
            .copied()
            .expect("semantic namespace has a typed owner-namespace node");
        self.owner_namespaces_graph.add_symbol(
            node,
            name,
            NamespaceSymbolEntry {
                identity,
                declaration_owner: owner,
                namespace_visibility: NamespaceVisibility::Public,
                in_export_retention_closure: true,
                has_external_candidate_view: true,
                extraction_visibility: ExtractionMemberVisibility::Default,
            },
        );
        identity
    }

    /// Allocate a Pattern-scope-local cluster symbol.
    ///
    /// Meta-injected member names live in one constructed Pattern scope,
    /// not in a graph namespace: two different Patterns may each own a
    /// ClusterSymbol `f`.  The symbol therefore never enters the global
    /// `(namespace, name)` symbol index and carries no namespace node.
    fn allocate_scope_local_symbol(
        &mut self,
        owner: SemanticOwnerId,
        name: &str,
        provenance: Provenance,
    ) -> SemanticSymbolIdentity {
        let next = self.local_symbol_counters.entry(owner).or_default();
        let identity = SemanticSymbolIdentity {
            owner,
            local: LocalSymbolIdentity(*next),
        };
        *next = next
            .checked_add(1)
            .expect("semantic symbol identity exhausted");
        self.symbols.insert(
            identity,
            SemanticSymbolCell {
                identity,
                name: name.to_string(),
                declaration_owner: owner,
                namespace_node: None,
                pure_p: None,
                sibling_vals: Vec::new(),
                sibling_places: BTreeMap::new(),
                member_views: Vec::new(),
                provenance,
            },
        );
        identity
    }

    fn allocate_pattern(
        &mut self,
        owner: SemanticOwnerId,
        provenance: Provenance,
    ) -> (PatternValueId, ResolvedPatternScopeId) {
        let pattern = PatternValueId(self.next_pattern);
        self.next_pattern = self
            .next_pattern
            .checked_add(1)
            .expect("PatternValue identity exhausted");
        let scope = ResolvedPatternScopeId(self.next_scope);
        self.next_scope = self
            .next_scope
            .checked_add(1)
            .expect("Pattern scope identity exhausted");
        let local_root = self.local_pattern_root_counters.entry(owner).or_default();
        let root = ResolvedPatternRootId {
            owner,
            local_root: *local_root,
        };
        *local_root = local_root
            .checked_add(1)
            .expect("Pattern root identity exhausted");
        self.patterns.insert(
            pattern,
            SemanticPatternValue {
                id: pattern,
                root,
                scope,
                provenance,
            },
        );
        self.scopes.insert(
            scope,
            ResolvedPatternScope {
                id: scope,
                owner,
                root,
            },
        );
        // Every pattern receives a canonical type-level Val2 place.
        let place = self.allocate_object_place();
        self.pattern_places.insert(pattern, place);
        (pattern, scope)
    }

    fn allocate_value_id(&mut self) -> SemanticValueId {
        let id = SemanticValueId(self.next_value);
        self.next_value = self
            .next_value
            .checked_add(1)
            .expect("semantic value identity exhausted");
        id
    }

    fn allocate_object_place(&mut self) -> ObjectPlaceId {
        let id = ObjectPlaceId(self.next_place);
        self.next_place = self
            .next_place
            .checked_add(1)
            .expect("object place identity exhausted");
        let resident = ResidentIdentity(self.next_resident);
        self.next_resident = self
            .next_resident
            .checked_add(1)
            .expect("resident identity exhausted");
        self.places.insert(
            id,
            ObjectPlace {
                id,
                resident: ResidentGeneration {
                    resident,
                    generation: 0,
                },
                associated_symbols: BTreeMap::new(),
                associated_val2: BTreeMap::new(),
            },
        );
        self.semantic_val2_snapshots
            .insert(id, SemanticVal2Snapshot::default());
        id
    }

    /// Establish a fresh binding destination carrying an equal resident
    /// Object. Storage maps are copied as an implementation realization of
    /// that resident; the new resident identity prevents writes, borrows and
    /// projection generations from aliasing the source binding.
    fn allocate_binding_destination(&mut self, value: SemanticValueId) -> Option<ObjectPlaceId> {
        let source_place = self.values.get(&value)?.place;
        let source_storage = self.places.get(&source_place)?.clone();
        let source_snapshot = self.semantic_val2_snapshots.get(&source_place)?.clone();
        let destination = self.allocate_object_place();
        let destination_storage = self
            .places
            .get_mut(&destination)
            .expect("fresh binding destination exists");
        destination_storage.associated_symbols = source_storage.associated_symbols;
        destination_storage.associated_val2 = source_storage.associated_val2;
        self.semantic_val2_snapshots
            .insert(destination, source_snapshot);
        Some(destination)
    }

    fn allocate_borrow(&mut self, kind: BorrowKind, target: StableBorrowTarget) -> BorrowViewId {
        let id = BorrowViewId(self.next_borrow);
        self.next_borrow = self
            .next_borrow
            .checked_add(1)
            .expect("borrow view identity exhausted");
        self.borrows.insert(id, BorrowView { id, kind, target });
        id
    }

    pub(crate) fn allocate_type_lookup_index(&mut self) -> TypeValueId {
        let id = TypeValueId(self.next_anonymous_type);
        self.next_anonymous_type = self
            .next_anonymous_type
            .checked_add(1)
            .expect("anonymous TypeValue identity exhausted");
        id
    }

    fn allocate_anonymous_type(&mut self) -> TypeValueId {
        self.allocate_type_lookup_index()
    }

    /// Return the existing `SemanticValueId` adapter for a given TypeValue, or
    /// create a fresh one. This is used to store a pure-P Object in Val2
    /// scopes which index by `SemanticValueId`.
    ///
    /// `transport_policy` is construction metadata for a freshly created
    /// adapter, NOT a Policy authority: the adapter is globally reused per
    /// TypeValue, so a later binding of the same type reuses the first
    /// adapter and its recorded Policy verbatim. Binding-level Policy for an
    /// associated type lives in the associated Symbol's member view
    /// (`C_f.member_views`), which is where lookup reads it from; two
    /// distinct bindings of one type therefore keep two distinct views.
    fn find_or_install_core_type_projection_value(
        &mut self,
        represented_type: TypeValueId,
        represented_pattern: PatternValueId,
        transport_policy: PolicyPair,
        provenance: Provenance,
    ) -> SemanticValueId {
        if let Some(existing) = self
            .core_type_projection_values
            .get(&represented_type)
            .copied()
        {
            return existing;
        }
        let type_rank = self.type_rank.unwrap_or(represented_type);
        let value = self.allocate_value_id();
        let place = self.allocate_object_place();
        self.values.insert(
            value,
            SemanticValueObject {
                id: value,
                type_value: type_rank,
                pattern: represented_pattern,
                place,
                policy: transport_policy,
                mode: PolicyMode::Plain,
                namespace_visibility: None,
                payload: SemanticValuePayload::CoreTypeProjection {
                    represented_type,
                    represented_pattern,
                },
                provenance,
            },
        );
        self.core_type_projection_values
            .insert(represented_type, value);
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical_value::{
        canonical_literal_content, canonical_literal_norm, CanonicalFullNavigation,
        CanonicalLiteralFamily, CanonicalPatternAtom, CanonicalPatternValue,
    };
    use lang_syntax::{NormDecl, NormForm, NormLiteralKind};

    fn test_source_closure() -> NormClosure {
        let parsed =
            lang_syntax::parse("let f = (self): compile -> let result: uint8 => { result; };");
        assert!(parsed.diagnostics.is_empty());
        let normalized = lang_syntax::normalize_program(&parsed.program);
        let [NormForm::Let(NormDecl::Let { slot, .. })] = normalized.forms.as_slice() else {
            panic!("one normalized closure declaration");
        };
        let Some(NormExpr::Closure(closure)) = slot.initializer.as_deref() else {
            panic!("initializer is a closure");
        };
        closure.clone()
    }

    #[test]
    fn typed_owner_namespace_graph_is_the_name_to_symbol_authority() {
        let mut world = SemanticWorld::new("app");
        let root = NamespaceNodeId(0);
        world.bind_toolchain_root(root);
        let identity = world.intern_symbol(
            root,
            world.toolchain_owner(),
            "typed",
            Provenance::new("typed semantic symbol"),
        );

        assert_eq!(
            world
                .symbol_in_namespace(root, "typed")
                .map(|cell| cell.identity),
            Some(identity)
        );
        let typed_root = world
            .owner_namespace_nodes
            .get(&root)
            .copied()
            .expect("toolchain root has a typed namespace node");
        assert_eq!(
            world
                .owner_namespaces_graph
                .symbol_entries(typed_root, "typed")
                .expect("typed graph records the symbol")
                .iter()
                .map(|entry| entry.identity)
                .collect::<Vec<_>>(),
            vec![identity]
        );

        // Corrupt only the graph projection. Semantic lookup must not
        // discover that declaration because the typed graph did not admit it.
        let projection_only = world.namespace_index.capability().declare(
            root,
            "projection_only",
            SymbolKind::Placeholder,
            crate::SourceCategory::DeclaredSymbol,
            Provenance::new("projection-only declaration"),
        );
        world.namespace_index = world
            .namespace_index
            .install_delta(projection_only)
            .expect("projection fixture is internally valid");
        assert!(world
            .namespace_index
            .child_symbol(root, "projection_only")
            .is_some());
        assert!(world.symbol_in_namespace(root, "projection_only").is_none());
    }

    #[test]
    fn namespace_delta_owner_failure_does_not_advance_projection_or_typed_graph() {
        let mut world = SemanticWorld::new("app");
        let root = NamespaceNodeId(0);
        world.bind_toolchain_root(root);

        // Install a parent only into the read projection so a later delta
        // passes projection validation but cannot acquire a semantic owner.
        let mut parent_delta = world.namespace_index.empty_delta();
        let projection_parent = crate::semantic_name_index::namespace_symbol(
            &mut parent_delta,
            root,
            "projection_parent",
            crate::NamespaceNodeKind::Declared,
            crate::SourceCategory::DeclaredSymbol,
            Provenance::new("projection-only parent"),
        );
        world.namespace_index = world
            .namespace_index
            .install_delta(parent_delta)
            .expect("projection-only parent is valid in the read index");

        let before_snapshot = world.namespace_index.snapshot_id();
        let before_typed_nodes = world.owner_namespace_nodes.clone();
        let mut child_delta = world.namespace_index.empty_delta();
        let child = crate::semantic_name_index::namespace_symbol(
            &mut child_delta,
            projection_parent,
            "child",
            crate::NamespaceNodeKind::Declared,
            crate::SourceCategory::DeclaredSymbol,
            Provenance::new("child below an unowned projection parent"),
        );

        assert!(world.install_namespace_name_delta(child_delta).is_err());
        assert_eq!(world.namespace_index.snapshot_id(), before_snapshot);
        assert_eq!(world.owner_namespace_nodes, before_typed_nodes);
        assert!(world.namespace_index.node(child).is_none());
    }

    // `Val1 × P` normalization asserted at the
    // interned ADDRESS level: `Addr(v) = Intern(Norm(v))` on one world.

    /// `same literal content + different P → different address`.
    #[test]
    fn same_literal_content_under_different_p_keeps_different_addresses() {
        let mut world = SemanticWorld::new("app");
        let intrinsic =
            world.intern_canonical_value(canonical_literal_norm(NormLiteralKind::Int, "1"));
        let under_structural_type =
            world.intern_canonical_value(CanonicalNormForm::Object(CanonicalObjectNorm {
                val1: Some(CanonicalVal1Norm::Literal {
                    family: CanonicalLiteralFamily::Int,
                    normalized: canonical_literal_content(NormLiteralKind::Int, "1"),
                }),
                pattern: CanonicalPatternNorm::Structural {
                    value: CanonicalPatternValue::Atom(CanonicalPatternAtom::Unit),
                },
                val2: Default::default(),
            }));
        assert_ne!(
            intrinsic, under_structural_type,
            "Norm_VP(Val1, P) is a pair: equal Val1 content under different Ps never merges"
        );
    }

    /// `distinct Pattern allocations + equivalent normalized P → same
    /// address` — and the nominal counter-case stays allocation-distinct.
    #[test]
    fn distinct_pattern_allocations_with_equivalent_normalized_p_share_one_address() {
        let mut world = SemanticWorld::new("app");
        let owner = world.package_owner();
        let (p1, _) = world.allocate_pattern(owner, Provenance::new("structural alloc 1"));
        let (p2, _) = world.allocate_pattern(owner, Provenance::new("structural alloc 2"));
        assert_ne!(p1, p2, "two allocations, two PatternValue identities");
        let structure = CanonicalPatternValue::unordered([(
            CanonicalFullNavigation::from_component("field"),
            CanonicalPatternValue::Atom(CanonicalPatternAtom::Type(
                crate::CanonicalTypeObservation::Detached(TypeValueId(41)),
            )),
        )])
        .expect("one unique field navigation");
        world.pattern_structural_norms.insert(p1, structure.clone());
        world.pattern_structural_norms.insert(p2, structure);

        let n1 = world.canonical_pattern_norm(p1).expect("p1 norm");
        let n2 = world.canonical_pattern_norm(p2).expect("p2 norm");
        assert_eq!(
            n1, n2,
            "equal normalized structural bodies share one Norm_P(P) regardless of allocation"
        );
        let a1 = world.intern_canonical_value(CanonicalNormForm::Object(CanonicalObjectNorm {
            val1: None,
            pattern: n1,
            val2: BTreeMap::new(),
        }));
        let a2 = world.intern_canonical_value(CanonicalNormForm::Object(CanonicalObjectNorm {
            val1: None,
            pattern: n2,
            val2: BTreeMap::new(),
        }));
        assert_eq!(a1, a2, "equivalent normalized P interns to one address");

        // Counter-case: nominal declaration patterns (no recorded structural
        // material) normalize by their declaration root — distinct
        // allocations keep distinct addresses.
        let (p3, _) = world.allocate_pattern(owner, Provenance::new("nominal alloc 1"));
        let (p4, _) = world.allocate_pattern(owner, Provenance::new("nominal alloc 2"));
        let n3 = world.canonical_pattern_norm(p3).expect("p3 norm");
        let n4 = world.canonical_pattern_norm(p4).expect("p4 norm");
        assert_ne!(n3, n4, "nominal patterns are equivalent only to themselves");
        let a3 = world.intern_canonical_value(CanonicalNormForm::Object(CanonicalObjectNorm {
            val1: None,
            pattern: n3,
            val2: BTreeMap::new(),
        }));
        let a4 = world.intern_canonical_value(CanonicalNormForm::Object(CanonicalObjectNorm {
            val1: None,
            pattern: n4,
            val2: BTreeMap::new(),
        }));
        assert_ne!(a3, a4);
    }

    #[test]
    fn ordinary_val2_members_never_create_direct_pattern_children() {
        let mut world = SemanticWorld::new("app");
        let owner = world.package_owner();
        let (pattern, _) = world.allocate_pattern(owner, Provenance::new("structural parent"));
        let canonical = CanonicalPatternValue::NamedPattern {
            navigation: CanonicalFullNavigation::from_component("Parent"),
            body: Box::new(
                CanonicalPatternValue::unordered([(
                    CanonicalFullNavigation::new(["real", "Parent"]),
                    CanonicalPatternValue::Atom(CanonicalPatternAtom::Unit),
                )])
                .expect("one structural child"),
            ),
        };
        world.pattern_structural_norms.insert(pattern, canonical);

        let real = world
            .direct_pattern_child(pattern, &crate::PatternSelector::Named("real".into()))
            .expect("registered structural entry is a direct child");
        assert_eq!(real.extraction_family, crate::StructuralDefault);

        let place = world
            .pattern_place(pattern)
            .expect("Pattern has a Val2 place");
        world
            .places
            .get_mut(&place)
            .expect("place exists")
            .associated_val2
            .insert("virtual".into(), vec![SemanticValueId(999)]);
        assert!(world
            .associated_values_for_pattern(pattern, "virtual")
            .is_some());
        assert!(
            world
                .direct_pattern_child(pattern, &crate::PatternSelector::Named("virtual".into()))
                .is_none(),
            "ordinary navigable Val2 membership is not structural incidence"
        );
    }

    #[test]
    fn prospective_projection_creation_and_existing_write_are_distinct() {
        let mut world = SemanticWorld::new("app");
        let place = world.allocate_object_place();
        let selector = ProjectionSelector::Named("field".into());
        let prospective = world
            .projection_slot(place, selector.clone())
            .expect("place exists");
        assert_eq!(prospective.contents, ProjectionSlotContents::Missing);

        let no_grant = WritableContext::default();
        assert_eq!(
            world.create_projection_value(place, selector.clone(), SemanticValueId(1), &no_grant),
            Err(PlaceMutationFailure::NotWritable)
        );

        let mut writable = WritableContext::default();
        writable.grant_place(place);
        let created = world
            .create_projection_value(place, selector.clone(), SemanticValueId(1), &writable)
            .expect("let-like creation instantiates the missing slot");
        assert_eq!(created, prospective.identity);
        assert_eq!(
            world
                .projection_slot(place, selector.clone())
                .expect("slot remains addressable")
                .contents,
            ProjectionSlotContents::Occupied
        );
        assert!(matches!(
            world.create_projection_value(place, selector.clone(), SemanticValueId(2), &writable),
            Err(PlaceMutationFailure::SlotAlreadyOccupied(_))
        ));
        assert!(matches!(
            world.write_projection_value(
                place,
                ProjectionSelector::Named("missing".into()),
                SemanticValueId(2),
                &writable
            ),
            Err(PlaceMutationFailure::SlotMissing(_))
        ));
        let written = world
            .write_projection_value(place, selector, SemanticValueId(2), &writable)
            .expect("ordinary assignment writes only an existing slot");
        assert_eq!(written, created);
    }

    #[test]
    fn borrow_algebra_is_explicit_and_parent_replacement_invalidates_old_slots() {
        let mut world = SemanticWorld::new("app");
        let place = world.allocate_object_place();
        let selector = ProjectionSelector::Named("field".into());
        let target = world
            .stable_projection_target(place, selector.clone())
            .expect("prospective slots can be borrowed explicitly");
        let reference = world
            .form_ref(BorrowOperand::Actual(target.clone()))
            .expect("explicit ref formation");
        assert_eq!(
            world.form_ref(BorrowOperand::Borrow(reference)),
            Ok(reference),
            "ref(ref(q)) is a fixed point"
        );
        let shared = world
            .form_share(BorrowOperand::Borrow(reference))
            .expect("share(ref(q)) is legal weakening");
        assert_eq!(
            world.borrow_view(shared).expect("share exists").kind,
            BorrowKind::Share
        );
        assert_eq!(
            world.form_share(BorrowOperand::Borrow(shared)),
            Ok(shared),
            "share(share(q)) is a fixed point"
        );
        assert_eq!(
            world.form_ref(BorrowOperand::Borrow(shared)),
            Err(BorrowFormationFailure::NoCandidateForStrengthening)
        );
        assert!(world.borrow_target_is_valid(reference));

        let old_slot = world
            .projection_slot(place, selector.clone())
            .expect("old slot")
            .identity;
        let mut writable = WritableContext::default();
        writable.grant_place(place);
        world
            .replace_place_resident(place, &writable)
            .expect("explicit resident replacement");
        let new_slot = world
            .projection_slot(place, selector)
            .expect("new resident has a prospective same-name slot")
            .identity;
        assert_ne!(old_slot, new_slot);
        assert!(!world.borrow_target_is_valid(reference));
        world
            .rebind_borrow(reference, StableBorrowTarget::Projection(new_slot))
            .expect("only explicit rebind acquires the replacement target");
        assert!(world.borrow_target_is_valid(reference));
    }

    #[test]
    fn open_here_extend_and_inject_keep_authority_writability_and_effects_distinct() {
        let mut world = SemanticWorld::new("app");
        let owner = world.package_owner();
        let (pattern, _) = world.allocate_pattern(owner, Provenance::new("open type"));
        let original = CanonicalPatternValue::NamedPattern {
            navigation: CanonicalFullNavigation::from_component("OpenType"),
            body: Box::new(CanonicalPatternValue::UnorderedLayer(BTreeMap::new())),
        };
        world
            .pattern_structural_norms
            .insert(pattern, original.clone());
        let authority = ConstructionAuthority::BuildRoot;
        let cluster = world.begin_cluster_construction(
            authority.clone(),
            owner,
            Provenance::new("open construction"),
        );
        world.ensure_pattern_cluster_ownership(pattern, cluster);

        let masked = ConstructionEvaluationContext::from_frames([
            ConstructionAuthority::MetaInvocation {
                meta_callable: MetaCallableIdentity {
                    selected_function_value: SemanticValueId(99),
                    selected_call_entry: SemanticValueId(100),
                },
                canonical_key: crate::MetaInvocationMaterialKey {
                    callable: MetaCallableIdentity {
                        selected_function_value: SemanticValueId(99),
                        selected_call_entry: SemanticValueId(100),
                    },
                    arguments: CanonicalValueAddr(99),
                    provenance: Provenance::new("masking meta frame"),
                },
            },
            authority.clone(),
        ]);
        assert_eq!(
            world.open_here(pattern, &masked),
            Err(OpenHereFailure::AuthorityMismatch(cluster)),
            "a live window does not bypass a masking meta authority frame"
        );

        let context = ConstructionEvaluationContext::current(authority);
        let open_here = world
            .open_here(pattern, &context)
            .expect("matching dynamic authority plus a live window establishes OpenHere");
        let extended = world
            .extend_pattern_value(
                &open_here,
                CanonicalFullNavigation::from_component("field"),
                CanonicalPatternValue::Atom(CanonicalPatternAtom::Unit),
                Provenance::new("pure extension"),
            )
            .expect("OpenHere value can be extended without a carrier write grant");
        assert_ne!(extended, original);
        assert_eq!(
            world.pattern_structural_norms.get(&pattern),
            Some(&original),
            "extend is a pure transform and leaves the old value unchanged"
        );

        let no_write = WritableContext::default();
        assert!(world
            .inject_extended_pattern_value(
                &open_here,
                CanonicalFullNavigation::from_component("field"),
                CanonicalPatternValue::Atom(CanonicalPatternAtom::Unit),
                &no_write,
                Provenance::new("unwritable injection"),
            )
            .is_err());
        assert_eq!(
            world.pattern_structural_norms.get(&pattern),
            Some(&original)
        );

        let mut writable = WritableContext::default();
        writable.grant_place(world.pattern_place(pattern).expect("Pattern carrier place"));
        let injected = world
            .inject_extended_pattern_value(
                &open_here,
                CanonicalFullNavigation::from_component("field"),
                CanonicalPatternValue::Atom(CanonicalPatternAtom::Unit),
                &writable,
                Provenance::new("writable injection"),
            )
            .expect("inject performs the committed write after pure extend");
        assert_eq!(
            world.pattern_structural_norms.get(&pattern),
            Some(&injected)
        );

        world
            .use_cluster_for_val1(cluster)
            .expect("using the construction closes the window");
        assert!(matches!(
            world.extend_pattern_value(
                &open_here,
                CanonicalFullNavigation::from_component("late"),
                CanonicalPatternValue::Atom(CanonicalPatternAtom::Unit),
                Provenance::new("late extension"),
            ),
            Err(_)
        ));
    }

    /// `same Val1 + equivalent P → same address`.
    #[test]
    fn same_val1_under_equivalent_p_shares_one_address() {
        let mut world = SemanticWorld::new("app");
        // Different spellings of one Val1 under the literal family's own P.
        let hex =
            world.intern_canonical_value(canonical_literal_norm(NormLiteralKind::Int, "0x10"));
        let dec = world.intern_canonical_value(canonical_literal_norm(NormLiteralKind::Int, "16"));
        assert_eq!(
            hex, dec,
            "one exact integer Val1, one intrinsic P, one address"
        );
        // Replay of the identical normal form is idempotent.
        let replay =
            world.intern_canonical_value(canonical_literal_norm(NormLiteralKind::Int, "16"));
        assert_eq!(dec, replay);
        // Exact-rational float spellings under one intrinsic P.
        let trailing =
            world.intern_canonical_value(canonical_literal_norm(NormLiteralKind::Float, "1.50"));
        let scientific =
            world.intern_canonical_value(canonical_literal_norm(NormLiteralKind::Float, "15e-1"));
        assert_eq!(trailing, scientific);
    }

    /// Pattern fallback and owned Val2 are different relations.
    ///
    /// A later member on the Pattern's canonical lookup place does not change
    /// an existing CoreTypeProjection's normal form.  An explicit contribution to the
    /// object's own SemanticVal2Snapshot does.
    #[test]
    fn pure_p_with_different_val2_keeps_different_addresses() {
        let mut world = SemanticWorld::new("app");
        let owner = world.package_owner();
        let (pattern, _) = world.allocate_pattern(owner, Provenance::new("val2 test pattern"));

        // Give the pattern a structural norm so canonical_pattern_norm succeeds.
        let structure = CanonicalPatternValue::Atom(CanonicalPatternAtom::Type(
            crate::CanonicalTypeObservation::Detached(TypeValueId(100)),
        ));
        world.pattern_structural_norms.insert(pattern, structure);

        // The Pattern's canonical place is a navigation fallback coordinate,
        // not this value's owned Val2 snapshot.
        let place_id = world.allocate_object_place();
        world.pattern_places.insert(pattern, place_id);

        // Register a CoreTypeProjection value pointing to this pattern.
        let represented_type = TypeValueId(200);
        let type_rank = world.type_rank.unwrap_or(represented_type);
        let value_id = world.allocate_value_id();
        let value_place = world.allocate_object_place();
        let policy = PolicyPair {
            value: crate::ValueComponentPolicy {
                stages: crate::StageSet::new(),
                presence: crate::ValuePresence::Absent,
            },
            pattern: crate::PatternComponentPolicy {
                stages: crate::StageSet::new(),
            },
        };
        world.values.insert(
            value_id,
            SemanticValueObject {
                id: value_id,
                type_value: type_rank,
                pattern,
                place: value_place,
                policy: policy.clone(),
                mode: PolicyMode::Plain,
                namespace_visibility: None,
                payload: SemanticValuePayload::CoreTypeProjection {
                    represented_type,
                    represented_pattern: pattern,
                },
                provenance: Provenance::new("val2 test pure type Object"),
            },
        );
        world
            .core_type_projection_values
            .insert(represented_type, value_id);

        // Build the RawArgShape that identifies this value.
        let raw = crate::product_shape::RawArgShape {
            index: 0,
            value_class: crate::product_shape::RawArgValueClass::Value,
            explicit_pass_mode: None,
            known_type_symbol_id: None,
            known_type_pattern_name: None,
            known_first_order_type_value: Some(represented_type),
            known_type_member_view: None,
            known_type_carrier_place: None,
            known_complete_type_observation: None,
            known_type_observation: None,
            known_semantic_value: Some(value_id),
            known_value_mode: None,
            provenance: Provenance::new("val2 test arg"),
        };
        let atom = lang_syntax::NormExpr::Literal {
            kind: NormLiteralKind::Int,
            text: "0".to_string(),
            origin: lang_syntax::NormOrigin::Source(lang_syntax::Span::new(0, 0, 0, 0)),
        };
        let product_atom = crate::product_shape::ProductAtom::Expression {
            expr: atom,
            provenance: Provenance::new("val2 test atom"),
        };

        // First address: empty Val2.
        let addr_empty = world
            .canonical_argument_address(&raw, &product_atom)
            .expect("acyclic Val2 normalizes");

        // Inject a fully normalizable value into the Pattern place (simulates
        // Val2 injection without relying on an allocation-only opaque leaf).
        let (leaf_pattern, _) =
            world.allocate_pattern(owner, Provenance::new("normalizable injected leaf pattern"));
        let injected_value = world.allocate_value_id();
        world.materialize_val1_object(SemanticValueObject {
            id: injected_value,
            type_value: TypeValueId(9999),
            pattern: leaf_pattern,
            place: ObjectPlaceId(0),
            policy: policy.clone(),
            mode: PolicyMode::Plain,
            namespace_visibility: None,
            payload: SemanticValuePayload::SimpleLiteral {
                family: CanonicalLiteralFamily::Int,
                normalized: "1".to_string(),
            },
            provenance: Provenance::new("normalizable injected leaf"),
        });
        world
            .associate_existing_value_in_place(place_id, "member", injected_value)
            .expect("semantic owned Val2 contribution succeeds");

        // The lookup fallback changed, but the existing object's owned Val2
        // did not.
        let addr_after_fallback = world
            .canonical_argument_address(&raw, &product_atom)
            .expect("acyclic Val2 normalizes");
        assert_eq!(
            addr_empty, addr_after_fallback,
            "lookup-visible inherited members are not owned Val2"
        );

        world
            .associate_existing_value_in_place(value_place, "member", injected_value)
            .expect("the CoreTypeProjection receives one owned Val2 contribution");
        let addr_with_owned_val2 = world
            .canonical_argument_address(&raw, &product_atom)
            .expect("acyclic owned Val2 normalizes");
        assert_ne!(addr_empty, addr_with_owned_val2);

        // Verify idempotence: same snapshot → same address.
        let addr_with_val2_replay = world
            .canonical_argument_address(&raw, &product_atom)
            .expect("acyclic Val2 normalizes");
        assert_eq!(
            addr_with_owned_val2, addr_with_val2_replay,
            "identical observed Val2 replays to the same interned address"
        );
    }

    /// Death test for the pre-cut-over rule that ignored Val2 on ordinary
    /// values: equal Val1 and Pattern diverge when their owned Val2 differs.
    #[test]
    fn ordinary_value_norm_observes_val1_pattern_and_val2() {
        let mut world = SemanticWorld::new("app");
        let owner = world.package_owner();
        let (pattern, _) = world.allocate_pattern(owner, Provenance::new("ordinary value Pattern"));
        let type_value = TypeValueId(500);
        world.types.insert(
            type_value,
            SemanticTypeValue {
                id: type_value,
                pattern,
                provenance: Provenance::new("ordinary value Type"),
            },
        );
        world.pattern_types.insert(pattern, type_value);
        let policy = PolicyPair {
            value: crate::ValueComponentPolicy {
                stages: crate::StageSet::new(),
                presence: crate::ValuePresence::Present,
            },
            pattern: crate::PatternComponentPolicy {
                stages: crate::StageSet::new(),
            },
        };
        let install_equal_literal = |world: &mut SemanticWorld| {
            let id = world.allocate_value_id();
            world.materialize_val1_object(SemanticValueObject {
                id,
                type_value,
                pattern,
                place: ObjectPlaceId(0),
                policy: policy.clone(),
                mode: PolicyMode::Plain,
                namespace_visibility: None,
                payload: SemanticValuePayload::SimpleLiteral {
                    family: CanonicalLiteralFamily::Int,
                    normalized: "7".to_string(),
                },
                provenance: Provenance::new("equal ordinary literal"),
            });
            id
        };
        let a = install_equal_literal(&mut world);
        let b = install_equal_literal(&mut world);
        let a_empty = world
            .canonical_member_value_address(a, &mut Val2NormState::default())
            .expect("ordinary Object normalizes");
        let b_empty = world
            .canonical_member_value_address(b, &mut Val2NormState::default())
            .expect("ordinary Object normalizes");
        assert_eq!(a_empty, b_empty, "equal Val1/P/Val2 must merge");

        let (leaf_pattern, _) = world.allocate_pattern(owner, Provenance::new("Val2 leaf Pattern"));
        let leaf = world.allocate_value_id();
        world.materialize_val1_object(SemanticValueObject {
            id: leaf,
            type_value: TypeValueId(501),
            pattern: leaf_pattern,
            place: ObjectPlaceId(0),
            policy,
            mode: PolicyMode::Plain,
            namespace_visibility: None,
            payload: SemanticValuePayload::SimpleLiteral {
                family: CanonicalLiteralFamily::Int,
                normalized: "1".to_string(),
            },
            provenance: Provenance::new("Val2 leaf"),
        });
        let b_place = world.value(b).expect("b exists").place;
        world
            .associate_existing_value_in_place(b_place, "only_b", leaf)
            .expect("semantic owned Val2 contribution succeeds");

        let a_after = world
            .canonical_member_value_address(a, &mut Val2NormState::default())
            .expect("ordinary Object normalizes");
        let b_after = world
            .canonical_member_value_address(b, &mut Val2NormState::default())
            .expect("ordinary Object normalizes");
        assert_eq!(a_empty, a_after);
        assert_ne!(
            a_after, b_after,
            "ordinary value canonicalization must not discard Val2"
        );
    }

    #[test]
    fn lifetime_observation_is_ordinary_val1_content_not_a_place_axis() {
        let mut world = SemanticWorld::new("app");
        let owner = world.package_owner();
        let (pattern, _) = world.allocate_pattern(owner, Provenance::new("lifetime value Pattern"));
        let type_value = TypeValueId(502);
        world.types.insert(
            type_value,
            SemanticTypeValue {
                id: type_value,
                pattern,
                provenance: Provenance::new("lifetime semantic Type"),
            },
        );
        world.pattern_types.insert(pattern, type_value);
        let lifetime = crate::LifetimeValue {
            name: crate::LifeName(17),
            observed_at: crate::SemanticPosition(4),
            origin: Some(crate::LifeName(3)),
            region: crate::Region {
                start: crate::SemanticPosition(1),
                end: Some(crate::SemanticPosition(9)),
                generation: 2,
            },
        };
        let policy = crate::compile_literal_policy();
        let a = world
            .install_lifetime_value(
                lifetime.clone(),
                type_value,
                policy.clone(),
                Provenance::new("first lifetime carrier"),
            )
            .expect("lifetime value installs");
        let b = world
            .install_lifetime_value(
                lifetime,
                type_value,
                policy,
                Provenance::new("second lifetime carrier"),
            )
            .expect("lifetime value installs again");
        assert_ne!(
            world.value(a).expect("a").place,
            world.value(b).expect("b").place,
            "the ordinary values occupy independent carrier Places"
        );
        let a_norm = world
            .canonical_member_value_address(a, &mut Val2NormState::default())
            .expect("first-class lifetime value normalizes");
        let b_norm = world
            .canonical_member_value_address(b, &mut Val2NormState::default())
            .expect("first-class lifetime value normalizes");
        assert_eq!(
            a_norm, b_norm,
            "carrier Place cannot become a fourth Object identity axis"
        );
    }

    #[test]
    fn semantic_declaration_delta_failure_leaves_no_partial_state() {
        let namespace = NamespaceNodeId(77);
        let mut world = SemanticWorld::new("app");
        world.bind_package_namespace(namespace);
        let before_symbols = world.symbols.len();
        let before_values = world.values.len();
        let before_types = world.types.len();
        let before_patterns = world.patterns.len();
        let before_scopes = world.scopes.len();
        let before_next_value = world.next_value;

        let closure = test_source_closure();
        let result_p2 = crate::policy_pair::normalize_p2_policy(
            closure
                .head
                .as_ref()
                .and_then(|head| head.call_policy.as_ref())
                .expect("test closure has P2"),
            Provenance::new("atomic semantic delta P2"),
        )
        .expect("test P2 normalizes");
        let function_view = crate::policy_pair::derive_function_object_view(
            &result_p2,
            &crate::policy_pair::FunctionObjectDeclarationPolicy {
                mode: PolicyMode::Plain,
            },
        );
        let missing_cluster = SemanticSymbolIdentity {
            owner: world.package_owner(),
            local: LocalSymbolIdentity(u64::MAX),
        };
        let delta = SemanticNamespaceDelta {
            namespace,
            entries: vec![
                SemanticDeclarationEntry::SourceCallable {
                    name: "would_be_partial".to_string(),
                    backing_declaration: SymbolId(900),
                    closure: closure.clone(),
                    outer_p1_explicit: None,
                    function_view: function_view.clone(),
                    body_entry_view: result_p2.clone(),
                    namespace_visibility: None,
                    return_shape: crate::policy_pair::ReturnShape::SingleVal(
                        crate::policy_pair::PatternConstraint::Unconstrained,
                    ),
                    provenance: Provenance::new("valid first staged entry"),
                },
                SemanticDeclarationEntry::ClusterContribution {
                    cluster_symbol: missing_cluster,
                    backing_declaration: SymbolId(901),
                    closure,
                    outer_p1_explicit: None,
                    function_view,
                    body_entry_view: result_p2,
                    namespace_visibility: None,
                    return_shape: crate::policy_pair::ReturnShape::SingleVal(
                        crate::policy_pair::PatternConstraint::Unconstrained,
                    ),
                    provenance: Provenance::new("failing second staged entry"),
                },
            ],
        };

        let error = world
            .install_namespace_delta(delta)
            .expect_err("missing cluster rejects the whole semantic transaction");
        assert!(error
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("target symbol not found")));
        assert_eq!(world.symbols.len(), before_symbols);
        assert_eq!(world.values.len(), before_values);
        assert_eq!(world.types.len(), before_types);
        assert_eq!(world.patterns.len(), before_patterns);
        assert_eq!(world.scopes.len(), before_scopes);
        assert_eq!(world.next_value, before_next_value);
        assert!(
            world
                .symbol_in_namespace(namespace, "would_be_partial")
                .is_none(),
            "the valid first entry was staged only and never committed"
        );
    }

    /// Two resolver search roots that reach the SAME terminal Symbol through
    /// DIFFERENT host chains are NOT one navigation.
    ///
    /// `ResolvedNavigation = ⟨HostChain, TerminalSymbol⟩` and the host chain
    /// participates in exposure, so deduping on the terminal Symbol alone
    /// would keep an order-dependent single path and silently erase the
    /// disagreement.  `navigate_semantic_path` dedups on the WHOLE navigation
    /// and reports the remaining disagreement as an ambiguity.
    #[test]
    fn navigation_across_roots_with_distinct_host_chains_is_ambiguous_not_terminal_deduped() {
        fn any_stage_policy() -> PolicyPair {
            PolicyPair {
                value: crate::ValueComponentPolicy {
                    stages: crate::StageSet::new(),
                    presence: crate::ValuePresence::Present,
                },
                pattern: crate::PatternComponentPolicy {
                    stages: crate::StageSet::new(),
                },
            }
        }

        let mut world = SemanticWorld::new("unit");
        // Two namespace roots under two owners so a `T` in each is a DISTINCT
        // carrier Symbol (the symbol index keys on `(namespace, name)`).
        let root_a = NamespaceNodeId(0);
        let root_b = NamespaceNodeId(1);
        world.bind_package_namespace(root_a);
        world.bind_toolchain_root(root_b);
        let provenance = Provenance::new("cross-root navigation ambiguity");
        let register = |world: &mut SemanticWorld, ns, name: &str, binding, represented| {
            world
                .register_type_symbol(
                    ns,
                    name,
                    SymbolId(binding),
                    TypeValueId(represented),
                    TypeValueId(0),
                    None,
                    any_stage_policy(),
                    provenance.clone(),
                )
                .expect("type-rank symbol registers")
        };

        // Seed the `type` rank once (types are global, not per-namespace).
        let _ = register(&mut world, root_a, "type_root", 0, 0);
        // One shared terminal Symbol `g`, reachable from both roots.
        let (g, _, _) = register(&mut world, root_a, "g", 1, 1);
        // Two DISTINCT `T` carriers, one per root.
        let (t_a, _, _) = register(&mut world, root_a, "T", 2, 2);
        let (t_b, _, _) = register(&mut world, root_b, "T", 3, 3);
        assert_ne!(t_a, t_b, "each root owns a distinct `T` carrier");
        let t_a_place = world.symbol(t_a).unwrap().pure_p_place().unwrap();
        let t_b_place = world.symbol(t_b).unwrap().pure_p_place().unwrap();
        assert_ne!(t_a_place, t_b_place);
        // Both carriers name the SAME terminal `g` under `f`.
        world
            .associate_existing_symbol_in_place(t_a_place, "f", g)
            .expect("`let f::T = g` in root A");
        world
            .associate_existing_symbol_in_place(t_b_place, "f", g)
            .expect("`let f::T = g` in root B");

        let path = ["f".to_string(), "T".to_string()];

        // Each root on its own resolves to the same terminal through its own
        // single-host chain.
        let from_a = world
            .navigate_path_from(&path, root_a)
            .expect("root A resolves `f::T`");
        let from_b = world
            .navigate_path_from(&path, root_b)
            .expect("root B resolves `f::T`");
        assert_eq!(from_a.terminal_symbol, g);
        assert_eq!(from_b.terminal_symbol, g);
        assert_ne!(
            from_a.host_chain, from_b.host_chain,
            "same terminal, but the host chains differ by carrier"
        );

        // Searching both roots at once is therefore ambiguous, not a silent
        // terminal-deduped pick.
        let ambiguous = world.navigate_semantic_path(&path, root_a, &[root_b], &[]);
        let error = ambiguous.expect_err("distinct host chains to one terminal are ambiguous");
        assert!(
            error.message.contains("ambiguous navigation"),
            "the disagreement is reported, not resolved by search order: {}",
            error.message
        );
    }

    #[test]
    fn complete_type_snapshots_keep_core_and_callspace_observations_distinct() {
        let mut world = SemanticWorld::new("unit");
        let owner = world.package_owner();
        let (core_pattern, _) =
            world.allocate_pattern(owner, Provenance::new("complete type core"));
        let core_lookup = TypeValueId(700);
        world.types.insert(
            core_lookup,
            SemanticTypeValue {
                id: core_lookup,
                pattern: core_pattern,
                provenance: Provenance::new("complete type core"),
            },
        );
        world.pattern_types.insert(core_pattern, core_lookup);
        let core_place = world.pattern_place(core_pattern);

        let before = world
            .observe_complete_type(core_lookup, core_place)
            .expect("empty complete type snapshot is well formed");
        assert!(before.call_space.is_empty());

        let (member_pattern, _) =
            world.allocate_pattern(owner, Provenance::new("direct member value pattern"));
        let member_type = TypeValueId(701);
        world.types.insert(
            member_type,
            SemanticTypeValue {
                id: member_type,
                pattern: member_pattern,
                provenance: Provenance::new("direct member value type"),
            },
        );
        world.pattern_types.insert(member_pattern, member_type);
        let empty_policy = PolicyPair {
            value: crate::ValueComponentPolicy {
                stages: crate::StageSet::new(),
                presence: crate::ValuePresence::Present,
            },
            pattern: crate::PatternComponentPolicy {
                stages: crate::StageSet::new(),
            },
        };
        let member = world
            .install_simple_literal_value(
                member_type,
                empty_policy,
                NormLiteralKind::Int,
                "1",
                Provenance::new("direct member value"),
            )
            .expect("member value installs");
        world
            .admit_direct_type_member(
                core_pattern,
                core_pattern,
                "member",
                TypeMemberFacet::Value,
                member,
            )
            .expect("same-home direct TypeMember is admitted");

        let after = world
            .observe_complete_type(core_lookup, core_place)
            .expect("extended complete type snapshot is well formed");
        assert_eq!(
            before.core, after.core,
            "ordinary type equality observes Core(tau), not V_tau"
        );
        assert_ne!(
            before.whole, after.whole,
            "whole snapshot identity observes immutable V_tau"
        );
        assert!(before.call_space.is_empty(), "old V_tau stays immutable");
        assert_eq!(
            world
                .complete_type_by_whole_observation(before.whole)
                .expect("old snapshot remains interned")
                .call_space,
            before.call_space
        );
    }

    #[test]
    fn callable_projection_unions_symbol_local_and_complete_type_candidates_once() {
        let mut world = SemanticWorld::new("unit");
        let namespace = NamespaceNodeId(0);
        world.bind_package_namespace(namespace);
        let owner = world.package_owner();
        let (pattern, _) = world.allocate_pattern(owner, Provenance::new("callable core"));
        let lookup = TypeValueId(705);
        world.types.insert(
            lookup,
            SemanticTypeValue {
                id: lookup,
                pattern,
                provenance: Provenance::new("callable core"),
            },
        );
        world.pattern_types.insert(pattern, lookup);
        let policy = PolicyPair {
            value: crate::ValueComponentPolicy {
                stages: crate::StageSet::new(),
                presence: crate::ValuePresence::Present,
            },
            pattern: crate::PatternComponentPolicy {
                stages: crate::StageSet::new(),
            },
        };
        let local = world
            .install_simple_literal_value(
                lookup,
                policy.clone(),
                NormLiteralKind::Int,
                "1",
                Provenance::new("symbol-local candidate"),
            )
            .expect("local candidate installs");
        let type_member = world
            .install_simple_literal_value(
                lookup,
                policy.clone(),
                NormLiteralKind::Int,
                "2",
                Provenance::new("TypeMember candidate"),
            )
            .expect("TypeMember candidate installs");

        let local_cluster = world.intern_symbol(
            namespace,
            owner,
            "callable-cluster",
            Provenance::new("symbol-local candidate cluster"),
        );
        let cell = world
            .symbols
            .get_mut(&local_cluster)
            .expect("local cluster was interned");
        cell.sibling_vals.push(local);
        cell.member_views.push(PolicyResultEntry {
            value: Some(local),
            pattern,
            view: PolicyView {
                pair: policy.clone(),
                mode: PolicyMode::Plain,
            },
        });
        world
            .associate_existing_symbol(pattern, "()", local_cluster)
            .expect("symbol-local call candidate is associated");
        world
            .admit_direct_type_member(pattern, pattern, "()", TypeMemberFacet::Value, type_member)
            .expect("direct TypeMember call candidate is admitted");

        let host = world
            .host_member_for_pattern(pattern)
            .expect("compiler Pattern host exists");
        let projected =
            world.associated_member_views_for_host(&host, "()", crate::Phase::OpenStatic);
        assert_eq!(
            projected
                .iter()
                .filter_map(|view| view.value)
                .collect::<Vec<_>>(),
            vec![local, type_member],
            "Symbol-local and V_tau candidates share one projection instead of local-first fallback"
        );
    }

    #[test]
    fn value_callability_uses_formed_tau_snapshot_not_object_val2_or_successor() {
        let mut world = SemanticWorld::new("unit");
        let owner = world.package_owner();
        let (pattern, _) = world.allocate_pattern(owner, Provenance::new("callable snapshot"));
        let lookup = TypeValueId(706);
        world.types.insert(
            lookup,
            SemanticTypeValue {
                id: lookup,
                pattern,
                provenance: Provenance::new("callable snapshot"),
            },
        );
        world.pattern_types.insert(pattern, lookup);
        let policy = PolicyPair {
            value: crate::ValueComponentPolicy {
                stages: crate::StageSet::new(),
                presence: crate::ValuePresence::Present,
            },
            pattern: crate::PatternComponentPolicy {
                stages: crate::StageSet::new(),
            },
        };
        let old_value = world
            .install_simple_literal_value(
                lookup,
                policy.clone(),
                NormLiteralKind::Int,
                "1",
                Provenance::new("old tau value"),
            )
            .expect("old value forms before the TypeMember");
        let call_member = world
            .install_simple_literal_value(
                lookup,
                policy.clone(),
                NormLiteralKind::Int,
                "2",
                Provenance::new("V_tau-only call member"),
            )
            .expect("member value");
        world
            .admit_direct_type_member(pattern, pattern, "()", TypeMemberFacet::Value, call_member)
            .expect("V_tau-only member admitted");

        assert!(
            world.callable_entries_for_value(old_value).is_empty(),
            "a later successor tau must not retarget a value formed with tau_old"
        );
        let old_place = world.value(old_value).expect("old value").place;
        world
            .associate_existing_value_in_place(old_place, "()", call_member)
            .expect("lookup Object.Val2 can contain the same spelling");
        assert!(
            world.callable_entries_for_value(old_value).is_empty(),
            "Object.Val2 is never callability authority"
        );

        let new_value = world
            .install_simple_literal_value(
                lookup,
                policy,
                NormLiteralKind::Int,
                "3",
                Provenance::new("new tau value"),
            )
            .expect("new value forms after the TypeMember");
        assert_eq!(
            world.callable_entries_for_value(new_value),
            vec![call_member],
            "a value formed with tau_new observes its immutable V_tau-only call family"
        );
    }

    #[test]
    fn foreign_direct_type_member_home_is_rejected() {
        let mut world = SemanticWorld::new("unit");
        let owner = world.package_owner();
        let (target, _) = world.allocate_pattern(owner, Provenance::new("target core"));
        let (foreign, _) = world.allocate_pattern(owner, Provenance::new("foreign core"));
        let target_type = TypeValueId(710);
        let foreign_type = TypeValueId(711);
        for (id, pattern) in [(target_type, target), (foreign_type, foreign)] {
            world.types.insert(
                id,
                SemanticTypeValue {
                    id,
                    pattern,
                    provenance: Provenance::new("foreign-home death test"),
                },
            );
            world.pattern_types.insert(pattern, id);
        }
        let value = world
            .install_simple_literal_value(
                foreign_type,
                PolicyPair {
                    value: crate::ValueComponentPolicy {
                        stages: crate::StageSet::new(),
                        presence: crate::ValuePresence::Present,
                    },
                    pattern: crate::PatternComponentPolicy {
                        stages: crate::StageSet::new(),
                    },
                },
                NormLiteralKind::Int,
                "2",
                Provenance::new("foreign member"),
            )
            .expect("foreign member value installs");
        let failure = world
            .admit_direct_type_member(target, foreign, "foreign", TypeMemberFacet::Value, value)
            .expect_err("foreign direct home cannot enter target V_tau");
        assert!(failure.message.contains("NoForeignTypeMemberInjection"));
    }
}
