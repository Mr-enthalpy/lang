//! Norm(v) / Addr(v) — canonical static-value normalization and semantic
//! interning addresses.
//!
//! ```text
//! Norm(v):  Val1 = ∅  =>  Norm_type(x) = ⟨Norm_P(P_x), Norm_Val2(Val2_x)⟩
//!           Val1 ≠ ∅  =>  Norm_VP(Val1, P)   (value material)
//! Val2 participates in Norm(v) for type arguments to meta invocations
//! (where Val2 represents the current injection state of the type).
//! For ordinary values, Val2 is not part of identity.
//! Closure values in Val2 are implicitly always-distinct (unique
//! SemanticValueId per allocation).
//! Addr(v) = Intern(Norm(v))                  (semantic interning address)
//! ```
//!
//! A pure P is a real object (`null × P × Val2`), and two carriers of one
//! Pattern can hold different Val2, so the Pattern alone cannot be the type
//! object's identity:
//!
//! ```text
//! Norm_Val2(V)     = Map_name( Norm_Cluster(V[name]) )
//! Norm_Cluster(C)  = ⟨ Norm_pureP(C.pureP)?, Multiset{ Norm_val(v) } ⟩
//! Norm_pureP(x)    = ⟨ Norm_P(P_x), Norm_Val2(Val2_x) ⟩
//! ```
//!
//! The recursion descends into the associated Symbol of each source-visible
//! Val2 name and terminates at objects whose Val2 is empty (`Val2(()) = ∅`).
//! The observation coordinate — which carrier's [`crate::ObjectPlaceId`] was
//! read to obtain a Val2 — is NOT identity material: it only decides *which*
//! Val2 is observed.  Equal Pattern plus equal recursive Val2 means one type
//! normal form even from two distinct places, and unequal recursive Val2
//! means two normal forms even under one Pattern.
//!
//! The interning table lives on [`crate::SemanticWorld`]; equal normal forms
//! share one snapshot-local [`CanonicalValueAddr`].  Simple literal values,
//! pure-P types, structured PatternValues, and static Products are
//! canonicalized here.  Every value normal form carries its Pattern
//! coordinate explicitly: `Norm_VP(Val1, P)` is a PAIR — equal Val1 content
//! under different Ps never shares one address.  Complex compile-time
//! memory values (heap material,
//! mutable buffers, pointer provenance, cycles, allocator state) are
//! registered future work: they receive an identity-stable opaque form (or a
//! fresh, never-merged address for material without any stable identity), so
//! deferral can only under-merge — it never collides two distinct values
//! into one address.
//!
//! Formal binder names, source paths, body material, provenance, and carrier
//! Symbols never appear in any normal form.

use lang_syntax::NormLiteralKind;

use std::collections::BTreeMap;

use crate::{
    identity::{SemanticValueId, TypeValueId},
    semantic_owner::ResolvedPatternRootId,
};

/// `Norm_Val2(V) = Map_name(Norm_Cluster(V[name]))` — the recursive normal
/// form of one object's Val2 at canonicalization time.
///
/// Keys are the object's source-visible Val2 names (a `BTreeMap`, so name
/// order is not identity material).  Each entry is the recursive normal form
/// of that name's ClusterSymbol, never a raw allocation id list.
pub type CanonicalVal2Norm = BTreeMap<String, CanonicalClusterNorm>;

/// `Norm_Cluster(C) = ⟨Norm_pureP(C.pureP)?, Multiset{Norm_val(v)}⟩` — the
/// normal form of one Val2 name.
///
/// `Val2(T_t)[f] = C_f` is itself a ClusterSymbol, so a Val2 name normalizes
/// exactly like any other cluster: at most one pure-P facet plus its sibling
/// vals.  `vals` is a multiset (sorted; duplicates retained) because sibling
/// order is not cluster identity, and each element is a content-derived
/// interning address rather than a `SemanticValueId`.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonicalClusterNorm {
    /// `Norm_pureP(x) = ⟨Norm_P(P_x), Norm_Val2(Val2_x)⟩`, interned.  Absent
    /// when the name carries only vals.
    pub pure_p: Option<CanonicalValueAddr>,
    /// The sibling vals' interned normal forms, sorted as a multiset.
    pub vals: Vec<CanonicalValueAddr>,
}

impl CanonicalClusterNorm {
    /// Build one cluster normal form, sorting the val multiset.
    pub fn new(pure_p: Option<CanonicalValueAddr>, mut vals: Vec<CanonicalValueAddr>) -> Self {
        vals.sort();
        Self { pure_p, vals }
    }

    pub fn is_empty(&self) -> bool {
        self.pure_p.is_none() && self.vals.is_empty()
    }
}

/// Snapshot-local semantic interning address: `Addr(v) = Intern(Norm(v))`.
///
/// Distinct from [`SemanticValueId`] (a value identity) and from
/// [`TypeValueId`]: two distinct values with equal normal forms share one
/// address, and the address never round-trips back to a value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalValueAddr(pub u64);

impl CanonicalValueAddr {
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

/// Literal family of a canonical literal normal form.
///
/// This is spelling-family material for `Norm_VP` of literal Val1 content —
/// not a builtin type identity and not a numeric-type selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalLiteralFamily {
    Int,
    Float,
    String,
}

impl CanonicalLiteralFamily {
    pub fn from_norm_literal_kind(kind: NormLiteralKind) -> Self {
        match kind {
            NormLiteralKind::Int => Self::Int,
            NormLiteralKind::Float => Self::Float,
            NormLiteralKind::String => Self::String,
        }
    }
}

/// `Norm_P(P)` — the canonical normal form of one PatternValue.  This is
/// NOT the snapshot allocation identity: a Pattern
/// with recorded structural material normalizes by that structure, so two
/// separately allocated PatternValues with equal normalized structural
/// bodies share one Pattern normal form.  Only patterns WITHOUT structural
/// material (nominal declaration patterns) normalize by their declaration
/// root coordinate — for those, the declaration identity IS the existing
/// pattern-equivalence normal form.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CanonicalPatternNorm {
    /// A Pattern carrying a recorded structural normal form (meta-generated
    /// struct patterns): `Norm_P(P) = CanonicalPatternValue(P)` by the
    /// normalized structural body, independent of allocation.
    Structural { value: CanonicalPatternValue },
    /// A nominal declaration Pattern: its declaration root coordinate is its
    /// normal form under the existing pattern-equivalence rules (nominal
    /// patterns are equivalent only to themselves).
    Nominal { root: ResolvedPatternRootId },
    /// The intrinsic Pattern of an un-materialized literal spelling: the
    /// literal family's own P.  A literal value materialized under a named
    /// type carries that type's Pattern instead — the two coordinates never
    /// merge (`same Val1 + different P → different address`).
    LiteralIntrinsic { family: CanonicalLiteralFamily },
}

/// One complete, inner-to-outer navigation name in a normalized Pattern.
///
/// Whether these components were written explicitly or inherited from an
/// enclosing Pattern layer is deliberately absent.  That distinction is
/// normalization input, not PatternValue identity.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalFullNavigation(Vec<String>);

impl CanonicalFullNavigation {
    pub fn new(components: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self(components.into_iter().map(Into::into).collect())
    }

    pub fn from_component(component: impl Into<String>) -> Self {
        Self(vec![component.into()])
    }

    pub fn components(&self) -> &[String] {
        &self.0
    }

    /// Complete an inner-to-outer relative navigation by appending the
    /// enclosing layer's already-complete outer navigation.
    fn inherit_outer(&self, enclosing: &Self) -> Self {
        let mut components = self.0.clone();
        components.extend(enclosing.0.iter().cloned());
        Self(components)
    }
}

/// Navigation material accepted while normalizing one source Pattern child.
///
/// This enum is intentionally consumed by normalization.  It never appears
/// inside [`CanonicalPatternValue`], so explicit/inherited provenance cannot
/// affect Pattern equality.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatternNavigationInput {
    Explicit(CanonicalFullNavigation),
    InheritOuter(CanonicalFullNavigation),
}

impl PatternNavigationInput {
    fn complete(self, enclosing: &CanonicalFullNavigation) -> CanonicalFullNavigation {
        match self {
            Self::Explicit(navigation) => navigation,
            Self::InheritOuter(relative) => relative.inherit_outer(enclosing),
        }
    }
}

/// One source child accepted by direct-layer normalization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternChildInput {
    /// `None` is a genuinely bare child.  Navigation is necessary but not
    /// sufficient for order-insensitivity: the containing layer must also
    /// be the body of a named Pattern rather than a naked Product.
    pub navigation: Option<PatternNavigationInput>,
    pub value: CanonicalPatternValue,
}

/// The syntactic/semantic container of one direct Pattern child layer.
///
/// A naked Product is always positional.  A named Pattern body may erase
/// sibling order only when every direct child has a complete navigation
/// identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatternLayerContext {
    NakedProduct,
    NamedPatternBody,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DuplicatePatternNavigation {
    pub navigation: CanonicalFullNavigation,
}

/// One entry in an order-sensitive Pattern layer.
///
/// A named child still carries its completed navigation, while a genuinely
/// bare child has `None`.  Position always participates in equality.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonicalOrderedPatternEntry {
    pub navigation: Option<CanonicalFullNavigation>,
    pub value: CanonicalPatternValue,
}

/// Incremental normalizer for one fully named, order-insensitive Pattern
/// layer.
///
/// A one-shot `struct` body and a later **privileged** Pattern-value
/// injection (the future `t = t |> inject(bool inner)` built-in) both feed
/// this builder. Construction order and explicit/inherited spelling are
/// erased; only the completed navigation map survives in the PatternValue.
///
/// Ordinary navigated `let f::t = expr` never calls this builder — whether
/// the RHS is `Val1 × P × Val2` or a pure `null × P × Val2` type object, it
/// installs an associated Val2 member and cannot change the Pattern normal
/// form. Only `struct` and `inject` hold Pattern-injection privilege.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalPatternBuilder {
    root_navigation: CanonicalFullNavigation,
    entries: BTreeMap<CanonicalFullNavigation, CanonicalPatternValue>,
}

impl CanonicalPatternBuilder {
    pub fn named_root(root_navigation: CanonicalFullNavigation) -> Self {
        Self {
            root_navigation,
            entries: BTreeMap::new(),
        }
    }

    pub fn contribute_pattern_value(
        &mut self,
        navigation: PatternNavigationInput,
        value: CanonicalPatternValue,
    ) -> Result<(), DuplicatePatternNavigation> {
        let navigation = navigation.complete(&self.root_navigation);
        if self.entries.contains_key(&navigation) {
            return Err(DuplicatePatternNavigation { navigation });
        }
        self.entries.insert(navigation, value);
        Ok(())
    }

    pub fn finish(self) -> CanonicalPatternValue {
        CanonicalPatternValue::NamedPattern {
            navigation: self.root_navigation,
            body: Box::new(CanonicalPatternValue::UnorderedLayer(self.entries)),
        }
    }
}

/// One recursively comparable `PatternValue` normal form.
///
/// This is semantic value material, not a digest and not a construction
/// artifact id.  Hashing may accelerate an interning table, but derived
/// `Eq`/`Ord` over this tree defines equality.
///
/// The final value contains no `internal`/`external`, inherited/explicit,
/// source carrier Symbol, or source-path marker.  In an unordered layer the
/// complete navigation itself is part of element identity.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CanonicalPatternValue {
    /// A resolved leaf value used by a Pattern node.
    Atom(CanonicalPatternAtom),
    /// A named Pattern root/layer.  This is Pattern-internal navigation
    /// identity, not the name of whichever Symbol currently carries it.
    NamedPattern {
        navigation: CanonicalFullNavigation,
        body: Box<CanonicalPatternValue>,
    },
    /// A positional sibling layer.  Every naked Product uses this form, even
    /// when all children are named.  A named Pattern body also uses it when
    /// at least one direct child is bare.  Named entries retain their
    /// complete navigation in addition to their position.
    OrderedLayer(Vec<CanonicalOrderedPatternEntry>),
    /// A fully navigated direct-child layer inside a named Pattern body.
    /// Order is not semantic; the complete navigation name is the key and
    /// therefore part of equality.
    UnorderedLayer(BTreeMap<CanonicalFullNavigation, CanonicalPatternValue>),
    /// A canonical sum result.  Alternative order remains explicit until
    /// the complete sum-value algebra supplies a stronger equivalence rule.
    Sum(Vec<CanonicalPatternValue>),
    /// A normalized hole coordinate.
    Hole(u32),
}

impl CanonicalPatternValue {
    pub fn unordered(
        entries: impl IntoIterator<Item = (CanonicalFullNavigation, CanonicalPatternValue)>,
    ) -> Result<Self, DuplicatePatternNavigation> {
        let mut normalized = BTreeMap::new();
        for (navigation, value) in entries {
            if normalized.contains_key(&navigation) {
                return Err(DuplicatePatternNavigation { navigation });
            }
            normalized.insert(navigation, value);
        }
        Ok(Self::UnorderedLayer(normalized))
    }

    /// Complete child navigation and normalize one direct-child layer.
    ///
    /// Only a named Pattern body whose direct children are all navigated
    /// becomes a map keyed by complete navigation.  A naked Product is
    /// always ordered, including `(a, b)` when both `a` and `b` are named.
    /// Navigation-source provenance is erased in either result.
    pub fn direct_child_layer(
        context: PatternLayerContext,
        enclosing: &CanonicalFullNavigation,
        children: Vec<PatternChildInput>,
    ) -> Result<Self, DuplicatePatternNavigation> {
        if context == PatternLayerContext::NamedPatternBody
            && children.iter().all(|child| child.navigation.is_some())
        {
            Self::unordered(children.into_iter().map(|child| {
                (
                    child
                        .navigation
                        .expect("all child navigation was checked")
                        .complete(enclosing),
                    child.value,
                )
            }))
        } else {
            Ok(Self::OrderedLayer(
                children
                    .into_iter()
                    .map(|child| CanonicalOrderedPatternEntry {
                        navigation: child
                            .navigation
                            .map(|navigation| navigation.complete(enclosing)),
                        value: child.value,
                    })
                    .collect(),
            ))
        }
    }
}

/// Expand extraction navigation independently of PatternValue equality.
///
/// An explicit query navigation is already complete.  Otherwise, the
/// subject's local navigation follows its Pattern parents from nearest to
/// farthest.  A parent with [`PatternOwnNavigation::Absent`] contributes its
/// local components and traversal continues.  The nearest
/// [`PatternOwnNavigation::Explicit`] or
/// [`PatternOwnNavigation::ImplicitGlobal`] parent is the anchor; farther
/// parents are ignored.  A non-empty chain that ends without either anchor
/// is invalid because a root Pattern can never have `Absent` navigation.
///
/// The result is always an exact extraction path.  It must never be sent
/// through ordinary bare-name `near -> outer -> core` lookup, and it is
/// never cached into the carried [`CanonicalPatternValue`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatternOwnNavigation {
    /// This Pattern layer specifies its own complete navigation.  It is an
    /// anchor and no farther parent participates.
    Explicit(CanonicalFullNavigation),
    /// The outermost Pattern omitted a written navigation.  It is still an
    /// anchor: its local navigation is rooted at implicit global `::`.
    ImplicitGlobal,
    /// A non-root Pattern layer omitted its own navigation.  Its local
    /// components participate, then completion continues toward its parent.
    Absent,
}

/// One Pattern parent visited while completing extraction navigation.
///
/// The chain is ordered nearest-to-farthest.  It contains semantic Pattern
/// parent links only; it never guesses an outer layer from a resident name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtractionPatternParent {
    pub local_navigation: CanonicalFullNavigation,
    pub own_navigation: PatternOwnNavigation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MissingExtractionNavigationAnchor;

impl ExtractionPatternParent {
    pub fn new(
        local_navigation: CanonicalFullNavigation,
        own_navigation: PatternOwnNavigation,
    ) -> Self {
        Self {
            local_navigation,
            own_navigation,
        }
    }
}

pub fn expand_extraction_navigation(
    subject_local_navigation: &CanonicalFullNavigation,
    explicit_navigation: Option<&CanonicalFullNavigation>,
    parents_nearest_first: &[ExtractionPatternParent],
) -> Result<CanonicalFullNavigation, MissingExtractionNavigationAnchor> {
    if let Some(explicit_navigation) = explicit_navigation {
        return Ok(explicit_navigation.clone());
    }

    // An empty ancestry means the subject itself is the top Pattern.  Its
    // omitted navigation is `ImplicitGlobal`, never ordinary bare-name
    // lookup.
    if parents_nearest_first.is_empty() {
        return Ok(subject_local_navigation.clone());
    }

    let mut completed = subject_local_navigation.clone();
    for parent in parents_nearest_first {
        match &parent.own_navigation {
            PatternOwnNavigation::Absent => {
                completed = completed.inherit_outer(&parent.local_navigation);
            }
            PatternOwnNavigation::Explicit(navigation) => {
                completed = completed.inherit_outer(navigation);
                return Ok(completed);
            }
            PatternOwnNavigation::ImplicitGlobal => {
                completed = completed.inherit_outer(&parent.local_navigation);
                return Ok(completed);
            }
        }
    }

    Err(MissingExtractionNavigationAnchor)
}

/// One type observation consumed by a structural identity position (struct
/// Pattern leaves, field signatures, extraction views).
///
/// A bare `TypeValueId` is only the first-order projection: the same open
/// type observed before and after a Val2 injection keeps one `TypeValueId`
/// while its `Norm_type = ⟨Norm_P, Norm_Val2⟩` changes.  Identity positions
/// therefore consume the observation, never the bare projection.
///
/// - [`Self::Observed`] carries the interned `Addr(Norm_type(type_value,
///   place))` computed against the live [`SemanticWorld`] snapshot at the
///   invocation boundary — the authoritative observation identity.
/// - [`Self::Detached`] carries only the first-order projection, for
///   world-free standalone formal invocation where no observation channel
///   exists (and therefore no Val2 can be observed at all).
///
/// The two variants never compare equal, so a missing observation can only
/// under-merge — it never collapses two different Val2 observations.
///
/// [`SemanticWorld`]: crate::SemanticWorld
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalTypeObservation {
    Observed(CanonicalValueAddr),
    Detached(TypeValueId),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalPatternAtom {
    /// A resident type leaf, identified by its type OBSERVATION — the
    /// interned `Addr(Norm_type)` including the observed recursive Val2 —
    /// never by a bare `TypeValueId`.
    Type(CanonicalTypeObservation),
    Unit,
}

/// The Pattern constructor of a canonical static Product form.
///
/// The general static-Product `Val1 × P` normal form must state its own P
/// explicitly.  The only constructor currently
/// normalized is the fixed invocation-parentheses Product constructor; other
/// static Product Ps are registered future work and receive opaque
/// addresses instead of silently claiming this constructor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalProductConstructor {
    /// The fixed call-parentheses Product constructor P.
    CallParentheses,
}

/// Canonical normal form `Norm(v)` of one static argument value.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CanonicalNormForm {
    /// `Norm_type(x) = ⟨Norm_P(P_x), Norm_Val2(Val2_x)⟩` — pure-P material
    /// (Val1 = ∅) normalized by the PAIR of its Pattern's own canonical normal
    /// form and its own recursive Val2, never by a raw snapshot allocation
    /// index and never by the Pattern alone.  `val2` is the Val2 observed
    /// through this object's carrier place (with per-name fallback to the
    /// Pattern's canonical type object); the place itself never enters the
    /// form, so `Place(T) ≠ Place(U)` with equal recursive Val2 still shares
    /// one address.
    PureP {
        pattern: CanonicalPatternNorm,
        val2: CanonicalVal2Norm,
    },
    /// `Norm_val(v) = ⟨Norm_P(P_v), Norm_Val2(Val2_v)⟩` for a value object
    /// whose Val1 carries no further normalizable content of its own — a
    /// materialized call entry is the canonical case, and it is also the leaf
    /// where the Val2 recursion stops: `Val2(()) = ∅`, so
    /// `Norm(()) = ⟨Norm_P(P_FunctionItem), ∅⟩`.
    ///
    /// Kept apart from [`Self::PureP`] because a type object (`Val1 = ∅`) and
    /// a value object are different objects even under one Pattern normal
    /// form.
    ValueObject {
        pattern: CanonicalPatternNorm,
        val2: CanonicalVal2Norm,
    },
    /// Pure-P type material whose PatternValue is not resolved in this
    /// snapshot: normalized by the canonical TypeValue root itself.
    PurePType { type_value: TypeValueId },
    /// `Norm_VP(Val1, P)` of a simple literal: the canonicalized content
    /// (digit separators removed; integer radix spellings decoded to one
    /// canonical decimal form; float spellings normalized as exact decimal
    /// rationals; string spellings decoded from their ranked quote
    /// boundaries to content) PAIRED with the value's Pattern normal form.
    /// Equal content under different Ps keeps different addresses.
    Literal {
        family: CanonicalLiteralFamily,
        normalized: String,
        pattern: CanonicalPatternNorm,
    },
    /// `Norm_VP(Val1, P)` of a static Product: the ordered member addresses
    /// plus the Product's own constructor P.  Product structure participates
    /// positionally; member content participates only through each member's
    /// own interned address.
    ///
    /// The invocation argument parentheses are themselves a Product value,
    /// so a call's whole argument tuple normalizes through this form:
    /// `Norm(args) = Product⟨Addr(a1)..Addr(an)⟩` under the fixed
    /// call-parentheses constructor.  Top-level argument equivalence for
    /// meta instance keys is therefore order-sensitive by construction — it
    /// inherits the Product's positional identity rather than relying on an
    /// ad-hoc sequence encoding.
    Product {
        constructor: CanonicalProductConstructor,
        members: Vec<CanonicalValueAddr>,
    },
    /// A product Unit position.
    Unit,
    /// Deferred complex-value normalization: identity-stable but not
    /// content-normalized.  Two references to one semantic value share one
    /// address; two content-equal but distinct values keep distinct
    /// addresses (safe under-merge, registered future work).
    OpaqueValue { value: SemanticValueId },
}

/// Canonicalize an un-materialized literal spelling into its
/// `Norm_VP(Val1, P)` literal form under the literal family's intrinsic P.
///
/// - digit separators (`'`, per lexical spec §6.2) are removed for numeric
///   families;
/// - integer radix spellings (`0x` / `0o` / `0b`) are decoded into one
///   arbitrary-precision canonical decimal form;
/// - float spellings are normalized as EXACT decimal rationals
///   (`dec:{digits}e{exp}`) — never through a host `f64` round-trip, so no
///   two spellings merge unless they denote one exact rational (spellings
///   outside the plain decimal grammar, e.g. hexadecimal floats, keep a
///   deterministic separator-stripped lowercase fallback: safe under-merge,
///   exact decoding for those forms is registered future work);
/// - string spellings are decoded from their ranked quote boundaries
///   (`\\`^k `"` … `\\`^k `"`) to the raw content bytes — the v0.2 string
///   form has no escape decoding, so equal content across boundary ranks
///   shares one normal form.
///
/// A literal value already materialized under a named type normalizes with
/// that type's Pattern coordinate instead (see
/// [`crate::SemanticWorld::canonical_argument_address`]); the two P
/// coordinates never merge.
pub fn canonical_literal_norm(kind: NormLiteralKind, text: &str) -> CanonicalNormForm {
    let family = CanonicalLiteralFamily::from_norm_literal_kind(kind);
    let normalized = canonical_literal_content(kind, text);
    CanonicalNormForm::Literal {
        family,
        normalized,
        pattern: CanonicalPatternNorm::LiteralIntrinsic { family },
    }
}

/// Canonicalized content half of a literal `Norm_VP(Val1, P)` — the Val1
/// content normal form without any Pattern coordinate.
pub fn canonical_literal_content(kind: NormLiteralKind, text: &str) -> String {
    match CanonicalLiteralFamily::from_norm_literal_kind(kind) {
        CanonicalLiteralFamily::Int => canonical_int_spelling(text),
        CanonicalLiteralFamily::Float => canonical_float_value(text),
        CanonicalLiteralFamily::String => canonical_string_content(text),
    }
}

fn canonical_int_spelling(text: &str) -> String {
    let stripped: String = text.chars().filter(|c| *c != '\'').collect();
    let lower = stripped.to_ascii_lowercase();
    let (radix, digits) = if let Some(rest) = lower.strip_prefix("0x") {
        (16, rest)
    } else if let Some(rest) = lower.strip_prefix("0o") {
        (8, rest)
    } else if let Some(rest) = lower.strip_prefix("0b") {
        (2, rest)
    } else {
        (10, lower.as_str())
    };
    arbitrary_precision_radix_to_decimal(digits, radix).unwrap_or(lower)
}

/// Convert an unsigned integer digit string to decimal without imposing a
/// host-integer width.  Limbs are little-endian base 10^9; each source digit
/// performs the usual `value = value * radix + digit` update.
fn arbitrary_precision_radix_to_decimal(digits: &str, radix: u32) -> Option<String> {
    if digits.is_empty() {
        return None;
    }
    const LIMB_BASE: u64 = 1_000_000_000;
    let mut limbs = vec![0u32];
    for ch in digits.chars() {
        let digit = ch.to_digit(radix)? as u64;
        let mut carry = digit;
        for limb in &mut limbs {
            let next = (*limb as u64) * (radix as u64) + carry;
            *limb = (next % LIMB_BASE) as u32;
            carry = next / LIMB_BASE;
        }
        while carry != 0 {
            limbs.push((carry % LIMB_BASE) as u32);
            carry /= LIMB_BASE;
        }
    }
    while limbs.len() > 1 && limbs.last() == Some(&0) {
        limbs.pop();
    }
    let mut rendered = limbs
        .pop()
        .expect("the arbitrary-precision integer always has one limb")
        .to_string();
    for limb in limbs.iter().rev() {
        rendered.push_str(&format!("{limb:09}"));
    }
    Some(rendered)
}

/// Exact float normal form: decimal spellings that denote one exact
/// rational share one spelling (`dec:{digits}e{exp}`), computed purely on
/// the digit strings — the host `f64` NEVER participates, so the normal
/// form cannot over-merge distinct semantic inhabitants through binary64
/// rounding.  `1.5`, `1.50`, and `15e-1` all denote the
/// exact rational 15×10⁻¹ and merge; spellings outside the plain decimal
/// grammar (hexadecimal floats, exotic forms) keep the separator-stripped
/// lowercase spelling as an identity-stable atom: deterministic
/// under-merge, exact decoding for those forms is registered future work.
fn canonical_float_value(text: &str) -> String {
    let stripped: String = text.chars().filter(|c| *c != '\'').collect();
    let lower = stripped.to_ascii_lowercase();
    exact_decimal_float_norm(&lower).unwrap_or(lower)
}

/// Normalize a plain decimal float spelling (`digits[.digits][e[±]digits]`)
/// into its exact rational form `dec:{mantissa}e{exp}` with no leading or
/// trailing zero digits in the mantissa.  Returns `None` for spellings
/// outside this grammar.
fn exact_decimal_float_norm(lower: &str) -> Option<String> {
    let (mantissa_part, exp_part) = match lower.split_once('e') {
        Some((mantissa, exp)) => (mantissa, Some(exp)),
        None => (lower, None),
    };
    let (int_part, frac_part) = match mantissa_part.split_once('.') {
        Some((int, frac)) => (int, frac),
        None => (mantissa_part, ""),
    };
    if int_part.is_empty() && frac_part.is_empty() {
        return None;
    }
    if !int_part.bytes().all(|b| b.is_ascii_digit())
        || !frac_part.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let written_exp: i64 = match exp_part {
        Some(exp) => {
            let (negative, digits) = match exp.strip_prefix('-') {
                Some(rest) => (true, rest),
                None => (false, exp.strip_prefix('+').unwrap_or(exp)),
            };
            if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            let magnitude: i64 = digits.parse().ok()?;
            if negative {
                -magnitude
            } else {
                magnitude
            }
        }
        None => 0,
    };
    let mut digits: String = format!("{int_part}{frac_part}");
    let mut exp = written_exp.checked_sub(frac_part.len() as i64)?;
    let leading_zeros = digits.len() - digits.trim_start_matches('0').len();
    digits.drain(..leading_zeros);
    if digits.is_empty() {
        // Every all-zero spelling denotes the exact rational zero.
        return Some("dec:0".to_string());
    }
    while digits.ends_with('0') {
        digits.pop();
        exp = exp.checked_add(1)?;
    }
    Some(format!("dec:{digits}e{exp}"))
}

/// Decode a ranked quote-boundary string spelling to its content bytes.
///
/// The v0.2 string form is `\\`^k `"` content `\\`^k `"` with NO escape
/// decoding inside content (backslashes participate only in the boundary),
/// so the content slice `text[k+1 .. len-1-k]` IS the string value.  A
/// malformed spelling (unclosed string recovered by the lexer) keeps the
/// original spelling: deterministic, and malformed material never merges
/// with well-formed content.
fn canonical_string_content(text: &str) -> String {
    let bytes = text.as_bytes();
    let rank = bytes.iter().take_while(|b| **b == b'\\').count();
    let well_formed = bytes.len() >= 2 * rank + 2
        && bytes.get(rank) == Some(&b'"')
        && bytes.last() == Some(&b'"')
        && bytes[bytes.len() - 1 - rank..bytes.len() - 1]
            .iter()
            .all(|b| *b == b'\\');
    if well_formed {
        text[rank + 1..text.len() - 1 - rank].to_string()
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_radix_and_separator_spellings_share_one_normal_form() {
        let dec = canonical_literal_norm(NormLiteralKind::Int, "4096");
        let sep = canonical_literal_norm(NormLiteralKind::Int, "4'096");
        let hex = canonical_literal_norm(NormLiteralKind::Int, "0x1000");
        assert_eq!(dec, sep);
        assert_eq!(dec, hex);
    }

    #[test]
    fn integer_radix_normalization_is_not_limited_by_u128() {
        // 2^128, one greater than the largest width representable by u128.
        let dec = canonical_literal_norm(
            NormLiteralKind::Int,
            "340282366920938463463374607431768211456",
        );
        let hex =
            canonical_literal_norm(NormLiteralKind::Int, "0x100000000000000000000000000000000");
        assert_eq!(dec, hex);
    }

    #[test]
    fn float_spellings_with_equal_exact_rational_share_one_normal_form() {
        let plain = canonical_literal_norm(NormLiteralKind::Float, "1.5");
        let trailing = canonical_literal_norm(NormLiteralKind::Float, "1.50");
        let scientific = canonical_literal_norm(NormLiteralKind::Float, "15e-1");
        assert_eq!(plain, trailing);
        assert_eq!(plain, scientific);
        let other = canonical_literal_norm(NormLiteralKind::Float, "1.25");
        assert_ne!(plain, other);
    }

    #[test]
    fn float_normal_form_is_exact_and_never_a_host_f64_round_trip() {
        // These two spellings round to the SAME f64 (binary64 cannot
        // distinguish them) but denote DISTINCT exact decimal rationals:
        // a bit-pattern normal form would over-merge them.
        let a = canonical_literal_norm(NormLiteralKind::Float, "0.1");
        let b = canonical_literal_norm(
            NormLiteralKind::Float,
            "0.1000000000000000055511151231257827021181583404541015625",
        );
        assert_ne!(a, b, "exact rationals differ even when f64 rounds equal");
        // Zero spellings denote one exact rational.
        let z1 = canonical_literal_norm(NormLiteralKind::Float, "0.0");
        let z2 = canonical_literal_norm(NormLiteralKind::Float, "0e5");
        assert_eq!(z1, z2);
    }

    #[test]
    fn string_quote_boundary_ranks_share_one_content_normal_form() {
        let rank0 = canonical_literal_norm(NormLiteralKind::String, "\"ab\"");
        let rank1 = canonical_literal_norm(NormLiteralKind::String, "\\\"ab\\\"");
        assert_eq!(rank0, rank1);
        let distinct = canonical_literal_norm(NormLiteralKind::String, "\"ac\"");
        assert_ne!(rank0, distinct);
    }

    #[test]
    fn distinct_integer_values_keep_distinct_normal_forms() {
        let one = canonical_literal_norm(NormLiteralKind::Int, "1");
        let two = canonical_literal_norm(NormLiteralKind::Int, "2");
        assert_ne!(one, two);
    }

    #[test]
    fn literal_families_never_merge() {
        let int = canonical_literal_norm(NormLiteralKind::Int, "1");
        let float = canonical_literal_norm(NormLiteralKind::Float, "1");
        assert_ne!(int, float);
    }

    #[test]
    fn product_normal_form_is_order_sensitive_at_top_level() {
        // The invocation parentheses are a Product: swapping two distinct
        // member addresses changes the normal form, so argument-tuple
        // equivalence is positional, never bag/set equivalence.
        let a = CanonicalValueAddr(1);
        let b = CanonicalValueAddr(2);
        assert_ne!(
            CanonicalNormForm::Product {
                constructor: CanonicalProductConstructor::CallParentheses,
                members: vec![a, b]
            },
            CanonicalNormForm::Product {
                constructor: CanonicalProductConstructor::CallParentheses,
                members: vec![b, a]
            },
        );
    }

    #[test]
    fn equal_literal_content_under_different_patterns_never_merges() {
        // Norm_VP(Val1, P) is a PAIR: the intrinsic-spelling P and a
        // structural P keep equal content at distinct normal forms.
        let intrinsic = canonical_literal_norm(NormLiteralKind::Int, "1");
        let structural = CanonicalNormForm::Literal {
            family: CanonicalLiteralFamily::Int,
            normalized: canonical_literal_content(NormLiteralKind::Int, "1"),
            pattern: CanonicalPatternNorm::Structural {
                value: CanonicalPatternValue::Atom(CanonicalPatternAtom::Unit),
            },
        };
        assert_ne!(intrinsic, structural);
    }

    #[test]
    fn inherited_and_explicit_navigation_normalize_to_the_same_pattern_value() {
        let a = CanonicalPatternValue::Atom(CanonicalPatternAtom::Type(
            CanonicalTypeObservation::Detached(TypeValueId(1)),
        ));
        let enclosing = CanonicalFullNavigation::from_component("bool");
        let inherited = CanonicalPatternValue::direct_child_layer(
            PatternLayerContext::NamedPatternBody,
            &enclosing,
            vec![PatternChildInput {
                navigation: Some(PatternNavigationInput::InheritOuter(
                    CanonicalFullNavigation::from_component("t"),
                )),
                value: a.clone(),
            }],
        )
        .unwrap();
        let explicit = CanonicalPatternValue::direct_child_layer(
            PatternLayerContext::NamedPatternBody,
            &CanonicalFullNavigation::new(Vec::<String>::new()),
            vec![PatternChildInput {
                navigation: Some(PatternNavigationInput::Explicit(
                    CanonicalFullNavigation::new(["t", "bool"]),
                )),
                value: a,
            }],
        )
        .unwrap();
        assert_eq!(
            inherited, explicit,
            "navigation provenance is erased after full-name completion"
        );
    }

    #[test]
    fn unordered_pattern_identity_includes_the_complete_navigation_name() {
        let value = CanonicalPatternValue::Atom(CanonicalPatternAtom::Type(
            CanonicalTypeObservation::Detached(TypeValueId(1)),
        ));
        let left = CanonicalPatternValue::unordered([(
            CanonicalFullNavigation::new(["t", "bool"]),
            value.clone(),
        )])
        .unwrap();
        let right = CanonicalPatternValue::unordered([(
            CanonicalFullNavigation::new(["t", "truth"]),
            value,
        )])
        .unwrap();
        assert_ne!(
            left, right,
            "equal child values under distinct complete navigation names are distinct"
        );
    }

    #[test]
    fn named_pattern_body_is_unordered_only_when_every_child_is_named() {
        let a = CanonicalPatternValue::Atom(CanonicalPatternAtom::Type(
            CanonicalTypeObservation::Detached(TypeValueId(1)),
        ));
        let b = CanonicalPatternValue::Atom(CanonicalPatternAtom::Type(
            CanonicalTypeObservation::Detached(TypeValueId(2)),
        ));
        let no_enclosing = CanonicalFullNavigation::new(Vec::<String>::new());
        assert_eq!(
            CanonicalPatternValue::direct_child_layer(
                PatternLayerContext::NamedPatternBody,
                &no_enclosing,
                vec![
                    PatternChildInput {
                        navigation: Some(PatternNavigationInput::Explicit(
                            CanonicalFullNavigation::from_component("a"),
                        )),
                        value: a.clone(),
                    },
                    PatternChildInput {
                        navigation: Some(PatternNavigationInput::Explicit(
                            CanonicalFullNavigation::from_component("b"),
                        )),
                        value: b.clone(),
                    },
                ],
            )
            .unwrap(),
            CanonicalPatternValue::direct_child_layer(
                PatternLayerContext::NamedPatternBody,
                &no_enclosing,
                vec![
                    PatternChildInput {
                        navigation: Some(PatternNavigationInput::Explicit(
                            CanonicalFullNavigation::from_component("b"),
                        )),
                        value: b.clone(),
                    },
                    PatternChildInput {
                        navigation: Some(PatternNavigationInput::Explicit(
                            CanonicalFullNavigation::from_component("a"),
                        )),
                        value: a.clone(),
                    },
                ],
            )
            .unwrap(),
            "`(a, b)c == (b, a)c`: a fully navigated named Pattern body is an \
             unordered navigation map"
        );
        assert_ne!(
            CanonicalPatternValue::direct_child_layer(
                PatternLayerContext::NamedPatternBody,
                &no_enclosing,
                vec![
                    PatternChildInput {
                        navigation: None,
                        value: a.clone(),
                    },
                    PatternChildInput {
                        navigation: None,
                        value: b.clone(),
                    },
                ],
            )
            .unwrap(),
            CanonicalPatternValue::direct_child_layer(
                PatternLayerContext::NamedPatternBody,
                &no_enclosing,
                vec![
                    PatternChildInput {
                        navigation: None,
                        value: b,
                    },
                    PatternChildInput {
                        navigation: None,
                        value: a,
                    },
                ],
            )
            .unwrap(),
            "a named Pattern body containing bare children remains ordered"
        );
    }

    #[test]
    fn naked_product_remains_ordered_even_when_every_child_is_named() {
        let a = CanonicalPatternValue::Atom(CanonicalPatternAtom::Type(
            CanonicalTypeObservation::Detached(TypeValueId(1)),
        ));
        let b = CanonicalPatternValue::Atom(CanonicalPatternAtom::Type(
            CanonicalTypeObservation::Detached(TypeValueId(2)),
        ));
        let no_enclosing = CanonicalFullNavigation::new(Vec::<String>::new());
        assert_ne!(
            CanonicalPatternValue::direct_child_layer(
                PatternLayerContext::NakedProduct,
                &no_enclosing,
                vec![
                    PatternChildInput {
                        navigation: Some(PatternNavigationInput::Explicit(
                            CanonicalFullNavigation::from_component("a"),
                        )),
                        value: a.clone(),
                    },
                    PatternChildInput {
                        navigation: Some(PatternNavigationInput::Explicit(
                            CanonicalFullNavigation::from_component("b"),
                        )),
                        value: b.clone(),
                    },
                ],
            )
            .unwrap(),
            CanonicalPatternValue::direct_child_layer(
                PatternLayerContext::NakedProduct,
                &no_enclosing,
                vec![
                    PatternChildInput {
                        navigation: Some(PatternNavigationInput::Explicit(
                            CanonicalFullNavigation::from_component("b"),
                        )),
                        value: b,
                    },
                    PatternChildInput {
                        navigation: Some(PatternNavigationInput::Explicit(
                            CanonicalFullNavigation::from_component("a"),
                        )),
                        value: a,
                    },
                ],
            )
            .unwrap(),
            "`(a, b) != (b, a)`: naming Product elements never erases position \
             without a wrapping Pattern"
        );
    }

    #[test]
    fn extraction_navigation_stops_at_the_nearest_explicit_parent() {
        let completed = expand_extraction_navigation(
            &CanonicalFullNavigation::from_component("leaf"),
            None,
            &[
                ExtractionPatternParent::new(
                    CanonicalFullNavigation::from_component("middle"),
                    PatternOwnNavigation::Absent,
                ),
                ExtractionPatternParent::new(
                    CanonicalFullNavigation::from_component("anchor"),
                    PatternOwnNavigation::Explicit(CanonicalFullNavigation::new([
                        "anchor", "scope",
                    ])),
                ),
                ExtractionPatternParent::new(
                    CanonicalFullNavigation::from_component("farther"),
                    PatternOwnNavigation::Explicit(CanonicalFullNavigation::from_component(
                        "farther",
                    )),
                ),
            ],
        )
        .unwrap();
        assert_eq!(
            completed,
            CanonicalFullNavigation::new(["leaf", "middle", "anchor", "scope"]),
            "farther parents do not participate after the nearest explicit navigation anchor"
        );
    }

    #[test]
    fn extraction_navigation_reaching_an_implicit_top_is_an_exact_global_path() {
        let completed = expand_extraction_navigation(
            &CanonicalFullNavigation::from_component("leaf"),
            None,
            &[
                ExtractionPatternParent::new(
                    CanonicalFullNavigation::from_component("middle"),
                    PatternOwnNavigation::Absent,
                ),
                ExtractionPatternParent::new(
                    CanonicalFullNavigation::from_component("top"),
                    PatternOwnNavigation::ImplicitGlobal,
                ),
            ],
        )
        .unwrap();
        assert_eq!(
            completed,
            CanonicalFullNavigation::new(["leaf", "middle", "top"]),
            "the root Pattern's ImplicitGlobal anchor means exact `::`, not ordinary \
             near-to-outer lookup"
        );
    }

    #[test]
    fn nonempty_extraction_parent_chain_must_terminate_at_an_anchor() {
        assert_eq!(
            expand_extraction_navigation(
                &CanonicalFullNavigation::from_component("leaf"),
                None,
                &[ExtractionPatternParent::new(
                    CanonicalFullNavigation::from_component("not_a_root"),
                    PatternOwnNavigation::Absent,
                )],
            ),
            Err(MissingExtractionNavigationAnchor),
            "a truncated parent chain cannot silently pretend its last non-root layer is global"
        );
    }

    #[test]
    fn top_pattern_without_written_navigation_is_implicitly_global() {
        assert_eq!(
            expand_extraction_navigation(
                &CanonicalFullNavigation::from_component("top"),
                None,
                &[],
            )
            .unwrap(),
            CanonicalFullNavigation::from_component("top"),
            "an empty ancestry means the subject is the top Pattern and its omission is `::`"
        );
    }

    #[test]
    fn parent_chain_affects_shorthand_not_pattern_identity() {
        let pattern = CanonicalPatternValue::unordered([(
            CanonicalFullNavigation::new(["t", "bool"]),
            CanonicalPatternValue::Atom(CanonicalPatternAtom::Unit),
        )])
        .unwrap();
        let rebound_pattern = pattern.clone();
        assert_eq!(pattern, rebound_pattern);

        let local = CanonicalFullNavigation::from_component("t");
        assert_ne!(
            expand_extraction_navigation(
                &local,
                None,
                &[ExtractionPatternParent::new(
                    CanonicalFullNavigation::from_component("first_parent"),
                    PatternOwnNavigation::ImplicitGlobal,
                )],
            )
            .unwrap(),
            expand_extraction_navigation(
                &local,
                None,
                &[ExtractionPatternParent::new(
                    CanonicalFullNavigation::from_component("second_parent"),
                    PatternOwnNavigation::ImplicitGlobal,
                )],
            )
            .unwrap(),
            "bare extraction follows the Pattern parent chain without changing PatternValue identity"
        );

        let explicit = CanonicalFullNavigation::new(["t", "bool"]);
        assert_eq!(
            expand_extraction_navigation(
                &local,
                Some(&explicit),
                &[ExtractionPatternParent::new(
                    CanonicalFullNavigation::from_component("ignored_parent"),
                    PatternOwnNavigation::Absent,
                )],
            )
            .unwrap(),
            explicit,
            "an explicit extraction path is independent of the Pattern parent chain"
        );
    }

    #[test]
    fn one_shot_struct_and_privileged_incremental_contribution_normalize_equally() {
        let bool_pattern = CanonicalPatternValue::Atom(CanonicalPatternAtom::Type(
            CanonicalTypeObservation::Detached(TypeValueId(7)),
        ));

        // `let t = ((bool inner)t) |> struct;`
        let mut one_shot =
            CanonicalPatternBuilder::named_root(CanonicalFullNavigation::from_component("t"));
        one_shot
            .contribute_pattern_value(
                PatternNavigationInput::InheritOuter(CanonicalFullNavigation::from_component(
                    "inner",
                )),
                bool_pattern.clone(),
            )
            .unwrap();
        let one_shot = one_shot.finish();

        // First form Pattern `t`, then apply the future privileged
        // `t = t |> inject(bool inner)` semantic operation.
        //
        // This unit test covers only canonical PatternValue normalization —
        // not source-level `inject` evaluation, Val2 capability
        // installation, Symbol creation, or ObjectPlace updates.  Ordinary
        // `let inner::t = bool::` is associated-type installation and never
        // reaches this builder.
        let mut incrementally_built =
            CanonicalPatternBuilder::named_root(CanonicalFullNavigation::from_component("t"));
        incrementally_built
            .contribute_pattern_value(
                PatternNavigationInput::Explicit(CanonicalFullNavigation::new(["inner", "t"])),
                bool_pattern,
            )
            .unwrap();
        let incrementally_built = incrementally_built.finish();

        assert_eq!(one_shot, incrementally_built);
    }
}
