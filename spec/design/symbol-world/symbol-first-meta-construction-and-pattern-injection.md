# Symbol-First Meta Construction and Pattern Injection

**Status: canonical semantic authority.** This document owns symbol-first
resolution, Symbol role/member projections, `compile` / `meta` result
boundaries, meta return self-root identity (§4.4), resolved pattern scopes,
`struct`, pure `extend`, place-level `inject`, and the binding/install boundary.

Owner resolution is expressed through resolved Pattern scopes and stable
semantic owners; registry allocation details do not participate in semantic
identity. Implementation coverage is recorded in `spec/planning/roadmap.md`.

This document builds on, without replacing:

- `spec/design/symbol-world/type-values-places-and-borrow-views.md` for
  `SymbolId` / `PlaceId` / `TypeValueId`, the borrow views `ref` / `share` and
  continuation-relative lifetime name reification `@`,
  and independent writability / construction-authority (`OpenHere_Σ` /
  `WindowLive_Σ`) judgments;
- `spec/design/lifetime/lifetime-policy-and-overload-boundary.md` for the
  positive overloads of `@`, escape checking, and the lifetime-rule boundary;
- `spec/contracts/pattern-root-identity-and-explicit-navigation.md` for
  the preserved bare-name versus explicit-`::` distinction and the current
  registry-backed substrate;
- `spec/design/patterns-overload/pattern-values-relational-semantics-and-extraction.md`
  for the canonical Pattern relation, direct structural incidence,
  binderless Patterns, observation/extraction, and Pattern normalization;
- `spec/design/patterns-overload/static-pattern-spaces-and-extraction-chains.md`
  for later residual, `Done`, and control-pattern semantics;
- `spec/design/meta-invocation/meta-object-invocation-and-policy-reduction.md`
  for candidate selection, evaluation demand, policy, and residualization;
- `spec/design/build-package/build-system-design.md` for transactional
  namespace-graph assembly and physical source contributions;
- `spec/design/symbol-world/symbol-construction-units-and-namespace-origin.md`
  for namespace-facet origin, source/meta construction ownership, physical
  authority, and cross-file closure;
- `spec/design/symbol-world/symbol-policy-and-compile-flow-projection.md` for
  the Object flow `⟨Val1?, P, Val2⟩`, `Pv:Pp`, binding `P1`, result `P2`,
  compile-flow projection, derived compile companions, match staging, and
  automatic require.

## 1. Canonical Boundaries

**Terminology (frozen).** "Symbol" (capitalized) is the semantic constructor
`S = ⟨τ?, V_S?⟩` — a value of type `symbol` (glossary: Symbol terminology).
"A symbol" in natural language below means that constructor value (the
name-graph node); "binding name" / "symbolic name" / "named binding" name the
`NameBinding` concept; "the `symbol` type" / "`x : symbol`" are the type and its
classification. `NameBinding(a)` never implies `a : symbol`.

The design has five load-bearing boundaries:

```text
name/path resolution:
  path/name -> Symbol -> context-directed role/member projection

ordinary value binding:
  let destination = source
    -> resolve source Symbol -> read value -> bind destination Symbol/Place

compile-time value computation:
  compile -> any declared ordinary semantic value across result classes (§4.1)
    subject to the root-conservation law of §4.2

ordinary meta symbol construction:
  meta -> require every canonical argument GlobalKeyable
       -> MetaInstance(F, Norm(args))
       -> sealed navigable MetaInstanceRoot

graph mutation:
  let binding / namespace contribution -> NamespaceDelta installation
  ordinary = / inject                 -> write an already existing place
```

Consequences:

1. A name does not initially resolve as a type, value, namespace, function, or
   category. It resolves as a first-class `Symbol` constructor value (a value
   of type `symbol`).
2. Ordinary `let` reads a value through the source Symbol and creates a distinct
   destination Symbol/place. Ordinary `=` only writes an already existing
   place. Neither operation forwards, reroots, or merges identities, and bare
   `=` never creates a missing member.
3. `compile` computes values. It may accept and return **any** ordinary
   semantic value across result classes (§4.1): an ordinary `PatternValue` (including a
   `Symbol` constructor value), a complete type value `tau` (a rank-indexed closure, not an
   ordinary PatternValue), and a `type ref` / `type share` borrow instance. What it may not
   do is create a new global root: it registers no global
   Symbol, produces no nominal type lacking a normal global root, and never
   promotes a local temporary pattern value into a global type (§4.2).
4. For an ordinary meta callable, `meta` is static evaluation **plus** the
   authority to establish one navigable `MetaInstanceRoot`. Every such
   invocation establishes that globally identified but unsealed root on entry,
   without externally installing it, and no
   other ordinary callable coordinate may establish or seal that *kind* of
   root. It returns the default complete type value `τ` of that instance; the
   return stage
   promotes only the MetaInstance-owned stable result graph — the default
   result's `OwnedResultClosure(τ_M)`: `OwnedClosure(Core(τ_M))` plus its
   owned callspace closure `OwnedCallSpaceClosure(CallSpace(τ_M))` — and
   seals the instance
   (§4.1, §4.3). Privileged built-ins retain member-specific owner rules (§4.8).
5. `struct` forms a complete type value `tau` directly and
   `extend` is the primitive referentially pure value transformation. `inject`
   is the explicit read--extend--write wrapper over an existing `type ref`.
   None installs a new global root; only `inject` mutates an existing slot. A
   `let` binding or installation path that later carries `struct`'s result is
   what creates a Symbol; it does not retroactively make `struct` a
   symbol-producing generator. Registry allocation is implementation
   bookkeeping and is not an observable semantic effect.
6. A `let` binding or installation path chooses the installation place. It does
   not retroactively choose or reroot the pattern owner carried by the value.

## 2. Symbol-First Resolution and Role Projections

### 2.1 Conceptual `Symbol` constructor value

The specification model is:

```text
SymbolValue {
    SymbolId
    PlaceId

    tau: zero or one well-formed type value (WellFormedTau, type-values §2.2)
    V_S: zero or more ordinary sibling value members in typed buckets
}
```

A `Symbol` constructor value carries an optional well-formed type value `tau` and an optional
sibling space `V_S`. All four shapes are well-formed:

```text
WellFormedSymbol(<None, None>)
WellFormedSymbol(<tau,  None>)
WellFormedSymbol(<None, V_S>)
WellFormedSymbol(<tau,  V_S>)
```

A `Symbol` constructor value with `<None, None>` has nothing to project; it remains a valid
name/candidate-bearing node. The Symbol never forms a type from its own
contents: `tau` is formed before installation and carried as a whole value.

The canonical namespace projection of a `Symbol` constructor value is the core
of its installed type value:

```text
NamespaceCoreProjection(S) =
    Some(Core(tau))   if tau is present
    None              otherwise

NamespaceProjection(S) = NamespaceCoreProjection(S)
TypeProjection(S)      = tau  iff tau present and TypeValueRole(tau)
                               (undefined otherwise)
```

When `tau` is present, `NamespaceProjection(S) = Core(tau) = Q` whether or not
`TypeRole(Q)` holds: namespace projection exists whenever the Symbol carries a
well-formed `tau` (type-values §2.2). Type projection requires the type-role
refinement: `TypeProjection(S) = tau` iff `TypeValueRole(tau)` (equivalently
`CompleteType(tau)`), i.e. `TypeRole(Core(tau))`. A `Symbol` constructor value with `<None, V_S>`
has no core: ordinary sibling members never become one by
count — `NamespaceOnly(tau)` binds to the formal `NamespaceOnly(Q)` judgment
(`NamespaceRole(Q)` and not `HasRegisteredSelfConstruction(Q)`; type-values
§2.2, pattern-values §13), never `count(pure members in V_S)`.

The complete type value is the closure:

```text
tau = bind alpha. <Q, V_τ[alpha]>

Core(tau)      = Q
CallSpace(tau) = V_τ
```

The closure and its description material are one semantic entity with two
equivalent views, not two objects (normative:
`spec/design/patterns-overload/pattern-values-relational-semantics-and-extraction.md`
§15; `SameEntityTypeInvariant` in
`spec/design/symbol-world/type-values-places-and-borrow-views.md` §2.2):

```text
DescriptionView(X) = ⟨P, Val2⟩
TypeClosureView(X) = τ = ⟨Q, V_τ⟩
τ ≡ DescriptionClosure(P, Val2)
```

Constraints on `⟨P, Val2⟩` and on `⟨Q, V_τ⟩` constrain the same entity;
neither view may be extended independently of the other.

`Q` is the ordinary pure Object core. `V_τ` is the callspace captured when
`tau` was formed — the direct TypeMember members that belong to this type
snapshot. `V_S` is the Symbol's own ordinary sibling candidate space,
independent of `V_τ`.

TypeMember membership is decided when a member is created, by its direct
classifier home; it is never a post-hoc partition of a shared Symbol space:

```text
TypeMember_Q(F)
  iff Anonymous(F)
  and DirectClassifierHome(F) = TypeMemberScope(Q)

CreateClassifier_Gamma(
  F,
  DirectClassifierHome = TypeMemberScope(Q)
)
  => CurrentAuthority_Γ(Q)
```

`DirectClassifierHome` is fixed when the classifier is created, but formation
itself is privileged: only a process holding current construction authority
for `Q` may create an anonymous classifier directly in `TypeMemberScope(Q)`.
Type/`struct` construction and `extend` over the current snapshot may hold that
authority; `inject` reaches it only by invoking `extend`. Ordinary callable
creation, navigated `let`, copying, rebinding, writing, and namespace
installation may neither nominate this direct home at creation nor change it
afterward. An anonymous classifier nested inside a direct TypeMember
classifier is a descendant, not a direct member:

```text
DirectClassifierHome(F) = TypeMemberScope(Q)
  => TypeMember_Q(F)

AncestorClassifierHome(G) = TypeMemberScope(Q)
and DirectClassifierHome(G) != TypeMemberScope(Q)
  => not TypeMember_Q(G)
```

`TypeMemberScope(Q)` itself is not a function of the whole `Q` snapshot. It is
derived from a stable, self-observable anchor inside `Q` — its canonical
self-pattern root — so that ordinary core replacement in a type-valued slot
does not silently invalidate every member's home:

```text
CoreAnchor(Q) = CanonicalSelfPatternRoot(Q)

TypeMemberScope(Q) = MemberScope(CoreAnchor(Q))

CoreAnchor(Core(τ)) = Root(τ)      -- for a complete type value τ;
                                    -- Root(τ) is the closure's canonical
                                    -- self-pattern root (see `extend`)
```

`MemberScope` is a derived classifier-home scope over the canonical Pattern
root; it introduces no new namespace/type identity ontology. The stability
theorem this anchors is:

```text
TypeMemberScopeStability:
  CoreAnchor(Q') = CoreAnchor(Q)
  ⇒ TypeMemberScope(Q') = TypeMemberScope(Q)
```

The converse is deliberately not promised: a replacement that changes the
core anchor (`CoreAnchor(Q') ≠ CoreAnchor(Q)`) may change the scope, and the
replaced closure `<Q', V_τ>` may then simply fail `WellFormedTau` — that is
the correct, history-free outcome, not a contradiction (type-values §2.2).

Home eligibility and snapshot membership are two different judgments:

```text
HomeEligible_Q(F)
  iff Anonymous(F)
  and DirectClassifierHome(F) = TypeMemberScope(Q)
      -- answers: which stable scope may this classifier belong to?
      -- TypeMember_Q(F) is the established name of this judgment

TypeMember_τ(F)
  iff F ∈ ClassifierDomain(V_τ)
  and HomeEligible_{Core(τ)}(F)
      -- answers: is this classifier actually carried by this snapshot?
      -- F ∈ MemberDomain(τ) ⇔ F ∈ ClassifierDomain(V_τ)
```

A concrete `τ`'s `V_τ` is fixed at formation and never grows: classifiers
created later under the same scope (e.g. by an `extend` that preserves
`CoreAnchor`) enter only the new snapshot `V_τ'` and never retroactively enter
an older `V_τ`. `DirectClassifierHome(F)` is immutable after creation, so a
surviving member's home eligibility is checked by comparing two current
values (`DirectClassifierHome(F) = TypeMemberScope(Q')`) — never by asking
whether `F` was originally created for an old `Q`.

A direct corollary is the injection invariant:

```text
NoForeignTypeMemberInjection:

F ∈ V_τ1
∧ TypeMemberScope(Core(τ1))
    !=
  TypeMemberScope(Core(τ2))
--------------------------------
F cannot become a member of V_τ2
merely because τ2 is structurally derived from τ1
```

A derived type value (`T ref`, `T share`, and any future derived construction)
has its own `V_τ` with its own direct-home obligation; it never inherits the
original type's callable objects. When a derived type must expose an associated
operation of its base, it creates a fresh, direct-home forwarder of its own
(`ForwardAssoc`, canonical in
`type-associated-function-objects-and-access-trees.md` — "Derived-Type
Associated Forwarding"); it never transports the base member `F` itself. This
invariant governs every derived type uniformly — `ref`, `share`, sequences, and
any future derived construction — and is not an exception to `WellFormedTau`.

Projections over the Symbol value are:

```text
NamespaceProjection(S) = NamespaceCoreProjection(S)   (§2.1)
TypeProjection(S)       = tau_S   iff tau_S present and TypeValueRole(tau_S)

TypeProjection(S) defined => NamespaceProjection(S) defined
(the converse does not hold: a Symbol may carry a namespace-only tau with
NamespaceRole(Core(tau)) and no TypeProjection)

CallableProjection(S)
  = DedupCandidateIdentity(V_S ⊎ V_τ)

V_S ⊎ V_τ
  -- source-annotated union: every candidate carries its source path
     (Symbol sibling space vs embedded closure callspace); the same
     object reachable through both paths appears here once per source

DedupCandidateIdentity(X)
  -- folds X by candidate/declaration identity: entries that are the
     same candidate (same declaration identity) collapse to one
     candidate; two different callables with identical signatures
     remain two candidates

Case analysis (a missing source contributes nothing to ⊎):
  = DedupCandidateIdentity(V_S ⊎ V_τ)   when both present
  = V_S                                 when tau_S absent
  = V_τ                                 when V_S absent
  = ∅                                   when <None, None>
```

Definition levels: `⊎` records provenance, `DedupCandidateIdentity` is the
candidate-identity quotient. Once duplicates are folded, the result is the
ordinary set union of the two sources — documents may write
`CallableProjection(S) = V_S ∪ V_τ` as shorthand for that quotient, but the
normative form is `DedupCandidateIdentity(V_S ⊎ V_τ)` and there is exactly one
canonical formula.

`CallableProjection` forms the candidate set in one step: there is no priority,
fallback, or reopening between `V_S` and `V_τ`. The same candidate reachable
through both paths is deduplicated by `DedupCandidateIdentity`; two different
callables with identical signatures remain two candidates. After the set is
formed, the ordinary overload pipeline runs once (hard admissibility → policy
preference → unique selection); failure does not reopen lookup.

`V_τ` is an intrinsic property of the embedded closure `τ`, not a function
of the Symbol `S`; `CallableProjection(S) = DedupCandidateIdentity(V_S ⊎ V_τ)`
is the Symbol call interface exposing the embedded closure callspace in one
step, and does not break the closure's independence. Ordinary sibling
operations only modify `V_S`; they cannot reach into the `V_τ` already
encapsulated in `τ`.

`tau` is not another Object and does not add a fourth Object coordinate. `Q`
and every ordinary member in `V_τ` remain ordinary Objects governed by the
existing `<Val1?,P,Val2>` ontology. The closure preserves their type-specific
pairing so a copied or extracted type carries its own callspace. `@` is the
continuation-relative name-reification operation that yields a lifetime value and never
a `type ref` (canonical owner
`../lifetime/lifetime-policy-and-overload-boundary.md` §1–§2); reaching the
type-level place explicitly uses `t |> (type ref)` or `(S ref).type`. The
closure is not normalized as a fourth kind of Object.

References from members in `V_τ` to the current type use the canonical binder:

```text
Norm_type^alpha(Self_τ) = BoundRef(alpha)
BoundRef(alpha) notin Children_owned
```

`Self_τ` establishes a `SymbolicReferenceEdge` to the enclosing closure —
symbolic anchoring, not an ownership edge and not an evaluation-flow edge, so
it never establishes an `ActiveEvaluation` or `OpenEvalReentry_κ`. Meta and
nonmeta closures share the same `bind alpha` / `Self_τ` representation; the
difference lives in the symbolic anchoring relation `SelfResolve`
(meta: root-relative/deferred; nonmeta: finite same-stratum static
backreference). The edge taxonomy and reentry criteria are canonical in
`type-values-places-and-borrow-views.md` §2.1.1.

After those authorized references are erased, the owned graph must satisfy
`WellFounded_kappa` (`type-values-places-and-borrow-views.md` §2.1): finite
under static-eval generation (covering both compile and meta) and acyclic once
materialized at runtime. This is
not a general recursive-Object rule; the complete normalization contract is
owned by `type-values-places-and-borrow-views.md` §2.1–§2.2.

An implementation may cache role projections in separate buckets, but storage
partitioning never creates additional semantic Objects.

Resolution is always:

```text
path/name
  -> Symbol
  -> context-directed role/member projection
```

The following are derived projections:

```lang
symbol |> type
symbol |> val
symbol |> namespace
```

They are not traditional casts or conversions. Projection selects an ordinary
member/role view of the same symbol under the expectation of the use site.

The complete type projection is `AsType`, not `TypeOf`:

```text
AsType(E) = E |> type
AsType(E) != TypeOf(E)
```

`AsType` neither raises universe rank nor manufactures a carrier place. Only
explicit type-of extraction may obtain the next classifier. `@` is the
continuation-relative name-reification operation and yields a lifetime value; it never
supplies `AsType` implicitly and never forms a borrow. A Symbol's `.type`
family is applicable exactly when the Symbol carries `τ` and
`TypeValueRole(τ)` holds (equivalently `TypeRole(Core(τ))`):
`S.type` reads the complete type snapshot by value, `(S ref).type` projects
`type ref`, and `(S share).type` projects `type share`. Reaching the
type-level place of an already-pure type slot uses `t |> (type ref)`, not `@`.

Each cluster member carries its own complete Policy view; the cluster itself
stores no flat Symbol-level Policy. The cluster-level Policy exists only as a
derived disjunction over the member views:

```text
cluster_policy(cluster)
    = fold(policy_or, cluster.member_views.map(member_policy))

P_cluster = P_member_1 || ... || P_member_n
```

This derived aggregate is a queryable fact of the ClusterSymbol's visible
domain, never a storage or exposure authority. Query and exposure always
filter per member:

```text
Expose(cluster, phase) = { member_i | Expose(P_i, phase) }
```

A phase admitted by the disjunction exposes only the members whose own view
admits that phase; no member Policy coordinate is ever re-derived from the
aggregate.

The disjunction law is exclusive: the members of one ClusterSymbol are the
only place in the model where a whole-function-object P1 is a disjunction
over per-object Policies — `P1 = P1_member_1 || ... || P1_member_n` holds
there and nowhere else. There is no second disjunction site to look for:

- a Val2 name is itself a ClusterSymbol (`Val2(T_t)[f] = C_f`), so
  `P(C_f) = P(P_x) || P(w_1) || ... || P(w_m)` is that same law applied
  one level down, not a second law: the host `C_t` and the host type member
  `T_t` never absorb `P(C_f)`;
- layered exposure (`t::inner`) composes conjunctively at lookup
  (`Expose(T_t, φ) ∧ Expose(x, φ)`), never disjunctively;
- a single object's P2 → P1 derivation unions its own value/pattern facets
  — an intra-object completion, not a cross-member disjunction;
- no namespace, owner, or overload-selection layer forms a Policy
  disjunction.

### 2.1.1 V_τ closure materialization: derived semantics

The invariants already stated — `τ = ⟨Q, V_τ⟩`, `TypeMember_Q(F)` home
eligibility, `P × Val2` as the description axis, `SymbolicReferenceEdge` ≠ owned
edge, and the stable `MetaInstanceRoot` — entail four derived theorems about
what `V_τ` contains and how its members may refer back to the enclosing type.
These are not optional explanatory text; they are semantic consequences that
close the abstraction boundary. Without them, two implementations could both
satisfy the surface formulas yet differ on what may enter `V_τ`, where the
`()` call-entry leaf lives, what a closure's structural formation is, and what
an enclosing-type reference resolves to.

**Theorem 1 — V_τ member closure ownership.**

For every callable member `F ∈ V_τ` there exists an anonymous type `A_F`
directly under the `τ` layer:

```text
F ∈ V_τ
  ⇒ exists A_F.
      AnonymousType(A_F)
      and DirectClassifierHome(A_F) = TypeMemberScope(Core(τ))
      and the callable's actual entry is the associated () Val2 leaf:
          Val2(A_F)[()] = call-entry(F)
```

`F` is not an arbitrary callable dropped into a set; it is materialized
through an anonymous type directly under `τ`. Capture, visibility, and the
`()` leaf placement constraint — the leaf must inhabit the associated scope of
the first parameter's type — jointly make arbitrary insertion impossible: only
a classifier whose direct home is `TypeMemberScope(Core(τ))`, created under
current construction authority for the core, may appear.

**Theorem 2 — Closure structural lowering.**

Closure materialization has the same structural formation semantics as an
anonymous `struct` construction that produces an anonymous type carrying an
associated `()` Val2 leaf:

```text
MaterializeClosure_τ(C)
  = anonymous type A directly below τ
  + associated Val2(A)[()] = call-entry(C)
```

This is a formation-semantics equation: the two paths share one structural
formation rule. It does not require source code to desugar into `struct`; it
requires that the canonical structural semantics of closure materialization and
anonymous-`struct` construction coincide. The anonymous type must sit directly
below the `τ` layer, just as a `struct` element sits directly below its owner —
the same layering constraint `struct` applies when it locates its top pattern.

**Theorem 3 — Enclosing-reference theorem.**

`V_τ` follows the `P × Val2` model: a `Val2` leaf inhabits a structural
position *below* the `P`/type layer it describes. An upward reference from a
`V_τ` descendant to its enclosing `τ` is therefore the same structural problem
as a `Val2` referring to its enclosing `P` layer — a static descriptive
reference, not an owned edge:

```text
upper P/type layer (τ)
    ↓
anonymous type A_F
    ↓
Val2[()] callable entry
    ↖ static reference to enclosing layer
```

The reference is a `BoundRef(alpha)` / `SymbolicReferenceEdge` (§2.1),
authorized and static; it is not in `Children_owned`, so it does not form an
owned cycle `τ → A_F → () → τ`. The existing invariant
`BackRefsOnlyInStaticPV2Region(τ)` (type-values §2.2, `WellFounded_static`) is
the well-foundedness projection of this theorem — it records the consequence,
not the theorem itself, and needs no separate recursive `V_τ` loop rule.

**Theorem 4 — Meta anchoring theorem.**

In a meta construction context, the upward enclosing reference resolves to the
stable `MetaInstanceRoot` determined at invocation entry, never to a
meta-local `r` or another ephemeral PatternValue:

```text
M = MetaInstanceRoot(MetaInstance(F, args))

HostAnchor(A_F) = M                -- always the stable invocation root

Forbidden:
  HostAnchor(A_F) = r_local        -- even when Value(r_local) = Value(installed result)
```

This is not a new prohibitive rule. It is the inevitable closure of three
existing invariant families:

```text
visibility
+ lifetime / global-survivability
+ capture / classifier-home
----------------------------------------
=> ephemeral return-local PatternValue
cannot become a V_τ enclosing anchor
```

While `int Vec::std` is computing, its body may hold a local construction
result `r`. In ordinary name resolution `r` can be only one of three things: a
capture-list entry, a local definition, or a global symbolic name. It cannot
be context-sensitively remembered as "the `r` from the return position": a
local definition or global lookup finds a *different* `r` (or none), and the
only candidate that could denote the meta-local value is the capture list. But
meta-local PatternValues whose lifetime is governed by the open/construction
window have non-global lifetimes that do not extend by simple copy — whether
interpreted by value or by borrow — so the capture-list path is closed as
well. The returned PatternValue and its **dependency closure** must both
satisfy global survivability; `EscapeDeps` checks this at seal (§4.3.2).

Even when `Value(r) = Value(installed result)`, value equality does not
retroactively imply `Identity(r) = MetaInstanceRoot`. Permitting such
retroactive promotion would reintroduce "future promotion can ratify past
capture" — exactly the model the meta-key / global-stability boundary has
always prohibited.

Closure construction and TypeMember injection are orthogonal operations:

```text
ConstructClosure(f) independent-of Inject(f, r)

Owner(AnonymousTypeOf(f)) = current stable MetaInstanceRoot
Inject(f, r)              = a later, explicit contribution to the open τ
```

An in-place closure therefore acquires its anonymous classifier owner from the
ambient meta environment before any return-local construction handle is
consulted. A source spelling or implementation shortcut may sequence closure
construction immediately before injection, but it must not use `r:type` as the
owner anchor, add `HomeSymbol(τ)`, or merge the two semantic operations. Nested
an unavailable in-place closure-anchoring consumer cannot recover the eventual
result binding as a substitute owner anchor.

### 2.2 Role and value projections coexist

One symbol may simultaneously provide:

- the complete immutable type closure `tau = <Q,V_τ>`, optionally written
  `bind alpha.<Q,V_τ[alpha]>`, stored at installation and returned by type
  projection; its pure core `Q = Core(tau)` also serves namespace projection;
- an ordinary value;
- a callable value;
- multiple heterogeneous value entries forming an overload candidate set.

The Symbol remains one Symbol. Namespace and type are not independently stored
Objects, and coexistence does not collapse role, value, Symbol, or place
identity.

### 2.3 Identity separation

The model preserves distinct identities:

```text
SymbolId
PlaceId
TypeValueId
PatternValue identity
ResolvedPatternScope / PatternScopeId
```

Their roles are:

```text
SymbolId:
  identity of the resolved symbol cell

PlaceId:
  identity of the bindable/openable installation location

TypeValueId:
  stable first-order root projection of Core(tau); implementation/index
  key only, not semantic equality. Ordinary type equality and keying
  observe Core(tau) = Q by default (minimal-change rule, type-values §2).
  Addr(Norm_type(tau)) is the whole-snapshot identity, used to tell
  shared-root snapshots apart in transport and in positions the language
  has independently frozen to whole-snapshot semantics

PatternValue identity:
  canonical identity of an ordinary compile-time value or structured pattern
  value — an ordinary Object / PatternValue. A complete type value `tau` is
  not itself an ordinary PatternValue: Pattern-facing observation goes
  through `Core(tau)`, and whole type identity goes through
  `Addr(Norm_type(tau))` (type-values §2.2)

PatternScopeId:
  identity of a navigable pattern-owner layer
```

No equality implication is automatic between these identities.

### 2.4 Program text names bindings before values

Except for literal syntax and other explicitly specified immediate values,
program text does not directly name a value. A source path first names a
Symbol, and value use then reads a facet/value from that Symbol:

```text
source path
  -> resolve Symbol
  -> read value / PatternValue from that Symbol
```

This applies to ordinary values, type values, pattern values, callable values,
and values later used as meta construction material.

Pattern navigation follows the same rule. A normalized pattern navigation name
may happen to render exactly like the source symbol path that carries it, but
matching diagnostic text does not merge their identities:

```text
source navigation path names a Symbol
PatternValue navigation name is a diagnostic/canonical projection
same spelling does not imply same semantic object
```

For a schematic future character spelling (the frozen lexer does not currently
accept `CharLiteral`):

```lang
let a = 'a';
```

The left `a` is a binding name. The right `'a'` is a character literal. Their
textual content happens to match, but they are not one semantic object.
Pattern values have no comparable standalone literal syntax, which makes a
same-spelled source path and pattern diagnostic projection especially easy to
confuse. The language still resolves the source path as a Symbol first.

### 2.5 General `let` value binding

The ordinary binding rule is uniform. Its optional policy prefix is P1:

```lang
P1 let r = expr;
```

Evaluation first produces policy-indexed value/pattern entries:

```text
Gamma |- expr : (tau, Pv:Pp)
Gamma |- ProjectP1(P1, result(expr)) = selected
selected is non-empty
------------------------------------------------
Gamma |- P1 let r = expr
```

A single P1 `Q` selects RHS value entries visible under Q and follows each
selected value's associated pattern/type component. A pair P1 `Qv:Qp` filters
both components. Single P1 is not `Q:Q`. There is no general
`binding_policy != runtime` condition, so a normal runtime binding is legal:

```lang
runtime let x = runtime_value;
```

Bare `let` first forms output selection preference `PolicyMode=plain`, before
RHS call selection; that preference participates with input Policy coordinates
in the ordinary product order. After unique RHS selection, omitted P1 retains
and infers the complete RHS pair view, while the selected producer retains its
declared concrete `ResultPolicyMode`. The destination remains independently
plain, and ordinary move/copy transfer between the two slots does not rewrite
the producer mode. See the canonical binding judgment in
`symbol-policy-and-compile-flow-projection.md` §3.1. The destination does not
inherit the RHS mode or make runtime the only way to obtain a runtime binding.

Policy migration does not reinterpret a P1 query as an exact target. Any
non-empty `ProjectP1` result completes the binding and makes
migration unreachable. Only after the complete query projects nothing may an
accepted runtime branch be extracted and paired with an eligible static input
view for one language-authorized atomic migration. The compiler mandates the
static-to-runtime stage edge; candidate-declared endpoint `PolicyMode` belongs to
ordinary overload. Empty queries with no runtime alternative fail, and no
Policy failure searches structure-changing operations. See
`../../contracts/policy-migration.md`.

The unannotated form:

```lang
let r = expr;
```

means:

```text
Gamma |- expr ⇓ v
fresh SymbolId s
fresh PlaceId p
--------------------------------
Gamma |- let r = expr
          where value(s) := v
```

If the right-hand expression is a source path, evaluation expands to:

```text
source path
  -> resolve source Symbol
  -> read its value / selected facet
  -> bind that value to the destination Symbol/Place
```

For example:

```lang
let a = b;
```

reads the value carried by `symbol(b)` and binds that value to `symbol(a)`.
It does not rename `a` to `b`, make their `SymbolId`s equal, or merge their
`PlaceId`s.

The bound object is the exact evaluated semantic value, not a new
binding-shaped copy:

```text
resolve(b) = s_b
read(s_b)  = v
bind(a, v)
```

Once `read(s_b)` has produced `v`, `s_b` is no longer part of `v`'s semantic
identity. It may remain in diagnostic provenance only. Ordinary `=` therefore
never requires or creates a `value -> original carrier Symbol` inverse map.
No declaration form forwards Symbol/place lookup (§2.6); shared observation of
another object is expressed only by a borrow view.

The rule does not change merely because the value is a type value, structured
pattern value, or symbol-construction result. In particular:

```lang
let t1::t = bool;
```

means:

```text
resolve symbol(bool)
  -> read the PatternValue carried by symbol(bool)
  -> bind that PatternValue to destination symbol/place t1::t
```

It does not reroot the `PatternValue`, rewrite its internal navigation, rename
its top pattern to `t1`, or identify `symbol(t1::t)` with the pattern owner.

Likewise:

```lang
let T: type = uint8;
let U: type = T;
```

has:

```text
Symbol(T) != Symbol(uint8)
Symbol(U) != Symbol(T)
Place(T)  != Place(uint8)
Place(U)  != Place(T)

TypeValue(uint8) = tau_uint8 = <Q_uint8,V_uint8>
TypeValue(T) = TypeValue(U) = Copy(tau_uint8)
Eval(T) = Eval(U) = tau_uint8
CoreView(tau_uint8) = Q_uint8
PatternView(T) = PatternView(U) = Q_uint8
CallSpace(tau_uint8) = V_uint8
```

The hole-free annotation `type` is the ordinary result-as transformation
applied while evaluating the RHS. It does not select a second “type binding”
judgment or a Boolean compatibility check.

Canonical summary:

```text
Program text normally cannot name values directly. It names a Symbol, then
obtains a value through that Symbol.

Name navigation is a way to obtain a value, not part of ordinary value
identity.

Pattern navigation paths are likewise Symbol navigation first. Even when a
PatternValue's canonical navigation name matches the Symbol carrying it, the
matching spelling does not establish identity.

A normalized fully named body of a named Pattern contains
complete-navigation to PatternValue entries, not Symbols. A naked Product
remains positional even when all of its children are named. Extraction
resolves a source Symbol, reads its PatternValue, and looks up its canonical
navigation/value entry in the normalized map.

let destination = source
uniformly reads source's value and binds it to destination. It does not reroot
patterns, perform symbol aliasing, or merge place identity.
```

Any separate rule that requires a compile-determined projection source to have
non-runtime policy constrains that rule's `Psrc` only. It does not constrain
the P1 binding destination. In particular, an
implementation must not reject a binding merely because
`binding_policy == runtime`.

### 2.6 There is no alias declaration form

Binding a name to an existing value is always an ordinary copy into a fresh
symbol and a fresh place:

```lang
let T = uint8;
```

```text
SymbolId(T) ≠ SymbolId(uint8)
PlaceId(T)  ≠ PlaceId(uint8)
Value(T)    =  Value(uint8)
```

The language defines **no** ordinary symbol-alias or place-forwarding
declaration. There is no form that makes a second name resolve to another
symbol's place, inherit its writability, or serve as a second entry point for
namespace extension. Shared observation of another object is expressed only by
the borrow views `ref` and `share` and continuation-relative lifetime name
reification `@`, specified in `type-values-places-and-borrow-views.md`.

The canonical conclusion is:

```text
value equality does not imply symbol equality;
value equality does not imply place equality;
no declaration converts value equality into place sharing.
```

Therefore several bindings may expose the same `TypeValueId` or pattern value
while each retaining its own distinct symbol and place.

No operator-name binding exception survives. The closed direction treats
`operator` as an ordinary global type and the nearest lexical
`operator : operator` as an ordinary value mapping `OperatorIdentity` to Symbols
or `None`, where `OperatorIdentity = spelling + fixity + arity`. Local
environments use value copy, shadowing, and Symbol `+=`/`-=`; complete selector
algebra remains deferred.

## 3. Value Members and Calls

### 3.1 A value entry is not necessarily a function

The typed `V` member buckets may contain any value:

```lang
let f = expr;
```

If `expr` produces a value, the declaration may contribute a value entry to
the Symbol `f`. The entry need not originate from closure syntax and need not
be callable.

Multiple entries under the same symbol may have heterogeneous types. A same-name
value-member family is therefore not equivalent to a traditional same-signature
function-overload bucket.

### 3.2 Call candidate preparation

A call position performs the following conceptual flow:

```text
resolve symbol
  -> form CallableProjection(S)
  -> enumerate heterogeneous values
  -> observe each Val2 object's Pv:Pp view for the current lookup stage
  -> obtain each value's type
  -> resolve the type-associated `()` call entry
  -> discard non-callable or non-applicable entries
  -> form fully admissible set A using structure, Pattern/type/result checks,
     receiver/parameter policy-pair compatibility, P2 target-result
     compatibility when constrained, stage legality, and concept/require legality
  -> retain phase-specificity/const-mut product-maximal candidates
  -> apply the remaining fixed-order preference filters
  -> enforce must-select consistency and require one final candidate
```

An uncallable value is valid value-facet material. It is discarded only while
preparing candidates for a call position. Its presence does not make the Symbol
invalid and does not turn it into a function overload.

Candidate identity and applicability belong to the candidate/invocation model;
symbol-first resolution only establishes where the heterogeneous values come
from. Derived compile companions are complete first-class `Val2` function
objects whose existence is derived under the compile transform
(`CompilePartner(F) = C(F)`, function-object-call-model §8), not post-failure
fallback entries; their policy and overload
obligations are defined in
`symbol-policy-and-compile-flow-projection.md`.

## 4. `compile`, `meta`, and Evaluation Demand

### 4.1 Orthogonal dimensions

The model has three independent dimensions:

```text
execution capability:
    meta / compile / seal / runtime

evaluation demand:
    partial / strict

result class:
    ordinary PatternValue
    | complete type value τ
    | type ref / type share borrow instance
    | runtime value
```

This is the current result-class set. Invocation results are driven by each
callable's declared result class — `Result(F)` follows
`DeclaredResultClass(F)` — and consumers must not maintain separate narrow
hand-written enumerations of what `compile` or `meta` can return.

A value of type `symbol` is an ordinary `PatternValue` (§4.7), so a declared
`symbol` result is a statement about which Pattern value is returned, not about
a separate ontological class.

`MetaPartial` / `MetaStrict` describe evaluation demand. They do not define the
meaning of `compile` or `meta`, and they do not determine the successful result
class.

Callable semantics still use ordinary PatternValue result declarations; there
is no private construction result class:

```text
CallableSemantics
    = P1 × P2 × DeclaredResultPattern × Privilege

Privilege   ::= Ordinary | BuiltinPrivileged   -- bounded AST access
```

`compile` may return any declared ordinary semantic value across result
classes (§4.1): an ordinary `PatternValue` (including a `Symbol` constructor
value), a complete type value `tau`, or a `type ref` / `type share` borrow
instance; a returned `tau` participates in Pattern
observation through `Core(tau)` and is not itself an ordinary
PatternValue/Object.
Ordinary-meta callable kind, call legality, and successful-call effects are
separate judgments inside the ordinary value/policy model:

```text
F in OrdinaryMetaFunction
  => P2(F) = meta
  and DefaultMetaResult(F) = τ

WellFormedMetaCall_Gamma(F, args)
  <=> F in OrdinaryMetaFunction
   and Admissible_Gamma(F, args)
   and forall a in Canonicalize(args): GlobalKeyable_Gamma(a)
   and forall a in Canonicalize(args): MetaArgumentAdmissible(a)

WellFormedMetaCall_Gamma(F, args)
  => K = MetaInstanceKey(F, Canonicalize(args))
   and M = MetaInstanceRoot(ParentSemanticOwner_Gamma(F), K)
   and RootIdentityExists(M)
   and ConstructionNavigationAvailable_Gamma(M)
```

The parent owner is an identity coordinate of the root, not diagnostic
placement metadata:

```text
Identity(M)
  = <ParentSemanticOwner(M),
     SelectedCallableIdentity(M),
     Addr(Product(Canonicalize(args)))>
```

The callable/argument pair may remain a reusable `MetaInstanceKey`, but a root
cache must scope that key by `ParentSemanticOwner`; equal callable and argument
material under distinct stable owners denotes distinct roots.

Root consistency is a positive invariant of meta-root formation:

```text
MetaInstanceRootAlwaysPlain:
  MetaInstanceRoot(M) => PolicyMode(M) = plain

MetaInstanceRoot(M) => StableSemanticOwner(M)
PolicyMode(M) = plain =/> Writable(M)
```

This `plain` coordinate belongs to root identity/formation and is not a
contextual default. Parameter/return position overlays and caller demands may
refine views produced under the root; they cannot change the root itself to
`const` or `mut`.

Equivalently, and without overloading “return shape”:

```text
DefaultMetaResult(F) = τ
MetaInstance(M) -> τ_M
ShapeOfTypeSymbol(v) = Σ = ⟨ τ?, V_S ⟩  -- shape of a `symbol`-typed value
```

`DefaultMetaResult = τ` is a default, not a constraint
(`OnlyMetaResult = τ` is false): an explicit `f : … -> symbol` is legal
because `symbol : type` is a first-class type. `τ` is complete independently
of any Symbol. `let t = meta_expr;` merely binds `τ_M` to a name; binding does
not retroactively prove the meta expression returns a Symbol.
`struct(P) → τ_P` follows the same default-result principle — `struct` is a
special built-in meta constructor.

The default result is fixed for every ordinary meta callable. `τ`, when present,
is the stored complete type closure; `V_S` may contain any ordinary sibling
values (when the result is explicitly `symbol`-typed). These are content facts
about the result value, not type/val/namespace result categories. Namespace projection selects `Core(tau)=Q` when `tau` is
present; type projection returns the stored `tau` — it never re-partitions
members to form `tau` post-hoc. The optional binder-aware form of `tau` is
`bind alpha.<Q,V_τ[alpha]>` when its members refer to `Self_τ`.

Callable kind fixes `P2` and `DefaultMetaResult`; `GlobalKeyable` belongs to a
particular call's well-formedness, never to the callable type itself. A
successful call establishes a globally stable root identity and makes it
navigable to the construction, while sealing remains the return-stage effect.
No `compile` callable may establish or seal this root kind.

This exclusivity does not claim that every stable owner/root in the language is
a `MetaInstanceRoot`. Lexical declarations and privileged built-ins may
establish, select, or preserve other root kinds only through their separately
specified owner rules (§4.8). They cannot use those rules to manufacture an
ordinary navigable `M`.

This is not a new result class. The default meta result is the complete type
value `τ` itself, which is not an ordinary `PatternValue`; an explicitly
declared `symbol` result returns a `symbol`-typed `Symbol` value rather than
turning `τ` into a `PatternValue`. Root authority governs the
open-window state and global lifetime of the default result's
`OwnedResultClosure(τ_M)` — `OwnedClosure(Core(τ_M))` plus
`OwnedCallSpaceClosure(CallSpace(τ_M))`, where `Core(τ_M)` is the first
projection of the default result and hence always present. An
implementation may retain a carrier to accumulate those members,
but may not expose that carrier as a callable result ontology.

A cluster-shaped invocation outcome transports multi-member construction
material after semantic result-class formation. It is not an ontological result
category. The following three roles remain distinct:

```text
Symbol value ontology          — an ordinary PatternValue (§4.7)
Meta return construction role  — the members a meta body accumulates before seal
Namespace same-name synthesis  — merging same-named contributions in a namespace
World installation role        — what a sealed root becomes in the global graph
```

A rule stated for one role does not transfer to another.

### 4.2 `compile`

`compile` is value-level staging. It performs compile-time computation without
creating a symbol-construction root:

```text
compile:
  input / output  any declared ordinary semantic value across result classes
                  (subject to root conservation, §4.2.1)
```

`compile` may pass and return:

- ordinary compile-time values (ordinary PatternValues);
- complete type values `tau` — they participate in Pattern observation through
  `Core(tau)` and are not ordinary PatternValues/Objects;
- `Symbol` constructor values (ordinary PatternValues);
- `type ref` and `type share` views;
- structured pattern values.

All of these may be passed to and returned from a `compile` callable. A computed
type value is still a value: it is not thereby an installed type symbol, a
namespace node, or an extendable place.

#### 4.2.1 Root conservation

The positive restriction on `compile` is that it conserves roots:

```text
Roots(Output)
  ⊆ Roots(Arguments)
  ∪ Roots(GlobalConstants)
  ∪ LexicallyDeclaredStableRoots
```

Every root reachable from a `compile` result must already have been rooted
somewhere the caller can name: in an argument, in a global constant, or in a
lexically declared stable declaration. Consequently `compile`:

```text
registers no global Symbol
produces no nominal type that lacks a normal global root
never promotes a local temporary pattern value into a global type
```

This is a conservation law, not a shape restriction. Returning a `Symbol` constructor value or
a `type ref` whose root already exists is legal; manufacturing a rootless
nominal type is not. `compile` is therefore not a rootless meta-type generator,
and "compile may return a type" and "compile may not invent a type root" are
both true.

Returning a `type ref` from `compile` is **not** prohibited:

```lang
let identity = (self, r: type ref): compile -> out: type ref => {
    r;
};
```

The returned view is subject to the ordinary lifetime/capability condition of
[`type-values-places-and-borrow-views.md`](type-values-places-and-borrow-views.md)
§5.5, evaluated at the receiving position. Its validity is independent of
whether the then-current pointee is Open. A return is rejected only when the
ordinary borrow escape check fails; a valid returned ref may later be unable to
`inject` because `OpenHere_Σ(Read(ref))` is false. Escape checking belongs to
[`../lifetime/lifetime-policy-and-overload-boundary.md`](../lifetime/lifetime-policy-and-overload-boundary.md)
§3. The body may weaken before returning when write capability is unnecessary:

```lang
r share;
```

#### 4.2.2 Compile is construction-transparent and root-non-generative

A `compile` evaluation reads two independent contexts:

```text
EvalCompile(F, args; ConstructionContext_caller)

DefinitionLexicalContext(F)
  — local Self space, anonymous closure type ownership,
    lexically declared identity

CallerConstructionContext
  — the current evaluation stack used with each value's `Anchor` and
    current window state
```

The definition context decides names and lexical owners. The caller context is
used only by operations that query `OpenHere_Σ(v)`: they combine the value's
`Anchor` with the current window state and an authority-frame resolution over the
caller's stack (§12.1.1). Neither context substitutes for the other.

Passing through a `compile` call, cloning, selecting, or composing a value
preserves its canonical value and `Anchor`/`GenerationRegime` while discarding source
place identity. A compile frame is transparent to the Open-authority stack walk, so an
OpenHere value remains OpenHere through any number of compile/transparent-intrinsic
frames unless another semantic boundary closes its construction interval:

```text
Anchor(Clone(Read(q))) = Anchor(Read(q))
OpenHere_{Σ + compile-frame}(v) = OpenHere_Σ(v)
```

The formal-parameter case is ordinary value transport:

```lang
let extend =
    (self, t: type): compile -> out: type => {
        (t, ...) |> extend;
    };
```

The call is applicable only when the transported value is open in the caller's
stack:

```text
Requires(extend) = OpenHere_Σ(t)
  -- OpenHere_Σ combines the live window state with the authority-frame
  -- judgment of §12.1.1 (non-meta: AuthorityFrame_Σ(t) exists;
  --          meta: Anchor = CurrentEvaluationCoordinate_meta)
```

A `type ref` parameter proves no such fact. A body that performs place-level
`inject` must read the pointee and check both independent premises:

```lang
let extend_ref =
    (self, t: type ref): compile -> out: type => {
        (t, ...) |> inject;
        t clone;
    };
```

```text
Requires(extend_ref) = OpenHere_Σ(Read(t)) ∧ Writable_Γ(Target(t))
```

Hence compile context sensitivity is construction-authority sensitivity, never
a hidden capability on `type ref`:

```text
a compile evaluation depends on the caller's Open window
  exactly for operations that query OpenHere_Σ on a transported PatternValue
```

not as a general property of every `compile` call, and not decided by whether a
`type` value happens to be a formal parameter. Caches and `Requires` summaries
track `Anchor` and the open-window state separately from canonical value
identity and recheck applicability in the caller stack.

`compile` does **not** create a `MetaInstanceScope`, does not introduce a
meta-style virtual symbolic-navigation layer for name shadowing, and does not
impose a self-root requirement on a returned type value. It may freely return an
already existing value:

```lang
let identity = (self, t: type): compile -> r: type => {
    let r = t;
    r;
};
```

The opposite boundary: `compile` has no responsibility to establish a new
globally stable `MetaInstance` anchor, so it may transport local or open
PatternValues as ordinary values:

```text
compile computation   may transport open/local PatternValues
meta invocation       requires globally survivable inputs (§4.3.3)

transport of an open PatternValue
  ≠
evaluation reentry of that PatternValue
```

Transporting an open PatternValue through `compile` is subject to
`NoOpenEvaluationReentry` (`OpenEvalReentry_κ`, type-values §2.1.1):
the value may be passed, but no active evaluation edge may be re-entered into
it. This is the complement of §4.3.3's argument boundary.

When a `compile` body uses a local `struct`, ordinary function-object scope
rules apply. Its ambient lexical/Pattern owner is the current
`CallableOwner` and that owner's callable-local `Self` space. This statement
does not determine the invocation receiver type. Standalone function-object
materialization defaults to an anonymous callable type derived from the owner;
an associated `()` implementation may instead bind invocation slot 0 and the
receiver-type projection of its local `Self` to a named receiver such as `T ref`.

Nested paths print in source order, current/innermost callable-local `Self`
first and outermost `Self` last, but identity is the parent-linked owner graph.
No `__inner_space` or `__inner_namespace` node participates in canonical
ownership. This owner is not a meta-instance owner such as
`MetaInstanceOwner(meta_function, canonical_arguments)`.

### 4.3 Ordinary `meta`

`meta` is symbol-level staging. An ordinary meta invocation is the only
construction that establishes a new navigable `MetaInstanceRoot`, and by §4.1
every ordinary meta invocation does so:

```text
WellFormedMetaCall_Gamma(F, args)
  => M = MetaInstance(F, Canonicalize(args))
   and RootIdentityExists(M)
   and ConstructionNavigationAvailable_Gamma(M)

RootIdentityExists(M) != ExternallyInstalled(M)
ConstructionNavigationAvailable_Gamma(M) != ExternallyInstalled(M)
```

Entering the invocation creates `M` as a **globally identified but unsealed
root** available to its construction. This does not publish a partially built
namespace delta. `ExternallyInstalled(M)` becomes true only after the returned
result crosses an ordinary outer binding/namespace-installation boundary and
that delta commits atomically (§12.4). The returned value is the default result
`τ_M` of `M`:

```text
meta:
  accepted parameters
  -> the default result τ_M of M
```

A meta callable may accept a `symbol` parameter, or constrain a parameter to a
narrower `type` or ordinary PatternValue. That does not introduce another result
class: successful ordinary meta invocation still defaults to `τ`. `M` exists in
the global world from body entry; the return stage runs the default-branch seal
`Seal(DefaultTau(τ_M))` of §4.3.2 —
well-formedness of `τ_M`, promotion of `OwnedResultClosure(τ_M)`, escape check —
and seals the result.

Failure never publishes construction material:

```text
FailedMetaCall(M) => not ExternallyObservablePartialInstallation(M)
```

Whether an implementation retains the failed canonical root identity for cache
or diagnostics is non-semantic. No partial namespace delta becomes externally
visible.

Meta functions are divided into two privilege classes:

```text
MetaFunction
  |- OrdinaryMetaFunction
  `- BuiltinPrivilegedAstMetaFunction
```

#### 4.3.1 The body is fully transparent to construction

Everything an ordinary meta body does to its own construction material is
permitted, and none of it closes the construction. The following are all legal
inside a meta body and none of them ends the open state of the values being
built:

```text
generating local pattern values
generating the same struct shape repeatedly
locally modifying material the body itself produced
using a value for Val1
passing material through static control flow
calling compile callables
entering an in-place closure that the body itself writes
referring recursively to M
```

This is the meta-closure transparency rule. The construction anchor of an
in-place closure written inside `M` is `M` itself:

```text
ConstructionAnchor( in-place closure inside M ) = M
```

so material owned by `M` remains open across that closure boundary. Anchor
transparency is not identity erasure: the closure still has its own anonymous
callable type identity,

```text
ClosureType = M::Site
```

and that identity keeps its own owner and lexical `Self` space. Transparency
concerns *who owns the construction*, not *which type the closure is*.

Construction transparency is not lifetime promotion. A fresh PatternValue
created inside an ordinary meta invocation has the invocation-local lifetime:

```text
Life(LocalPatternValue(M)) = MetaInvocation(M)
```

It may be copied through local binders, static control, `compile` calls, and
transparent construction intrinsics without freezing. Those operations do not
form a new global key. It may not, however, become a dependency of another
ordinary `MetaInstance` unless it has independently become `GlobalKeyable`.
Thus:

```text
No freezing inside M
  !=
arbitrary meta-local PatternValues implicitly become global
```

An anonymous closure type such as `M::Site` is globally stable only when every
PatternValue dependency in its signature is global-keyable. A signature may not
capture the identity of an ephemeral local PatternValue merely because the
closure type itself has a stable site name.

#### 4.3.2 Seal happens only at the return stage

The only construction-ending disposition of a meta invocation is its final
return stage, and it runs in a fixed order. The returned result is either the
default complete type value `τ_M = ⟨ Q, V_τ ⟩`, or the value of an explicitly
declared result type such as `Σ = ⟨ τ?, V_S ⟩` when that type is `symbol`.
These shapes have different seal obligations, so the seal judgment branches on
the result shape instead of sharing one optional-core criterion:

```text
Seal(DefaultTau(τ_M)):
    WellFormedTau(τ_M)
    Q := Core(τ_M)          // first projection of the construction; always present
    Pure(Q)
    Root(Q) = M
    promote OwnedResultClosure(τ_M) into M   (call it P)
    EscapeDeps(τ_M) subset AlreadyGlobalStable union P
    seal M

Seal(ExplicitSymbol(Σ)),  ShapeOfTypeSymbol(Σ) = ⟨ τ?, V_S ⟩:
    τ present ->
        Pure(Core(τ)) ∧ Root(Core(τ)) = M
        promote OwnedResultClosure(τ)          (call it P_Σ)
        EscapeDeps(Σ) subset AlreadyGlobalStable union P_Σ
    τ absent  -> EscapeDeps(Σ) subset AlreadyGlobalStable
    seal M

Seal(ExplicitOther(T)):
    the result-type-specific seal obligations of T
```

For the default branch, `Core` is a total projection on complete type
values, so `τ_M` always has a defined core projection:

```text
τ_M = ⟨Q, V_τ⟩
--------------------------------
Core(τ_M) = Q
```

This is a pair projection (an elimination rule), not a cardinality count:
there is no "core collection" to size, no `τ`-absent case to guard, and no
optional installed-core slot. `Q` is the first projection of `τ_M`. The
self-root rule is unconditional there: `Root(Core(τ_M)) = M` holds for every
well-formed default result. A namespace-only core — `NamespaceRole(Core(τ_M))`
and `not HasRegisteredSelfConstruction(Core(τ_M))` — is
therefore a valid promotion anchor even when `TypeRole(Core(τ_M))` is false;
type-role requirements are refinements, not generic result constraints.

The `τ?` slot belongs to an explicitly `symbol`-typed result value
(`ShapeOfTypeSymbol`, glossary), and the conditional form above applies only to
that branch. An explicitly declared `symbol` result with `τ` absent skips
promotion and requires its entire escape dependency set to be already globally
stable:

```text
τ present -> EscapeDeps(Σ)
               subset AlreadyGlobalStable union P_Σ
τ absent  -> EscapeDeps(Σ)
               subset AlreadyGlobalStable
```

`EscapeDeps(τ)` traverses the whole returned result at the τ level:
`Core(τ) union CallSpace(τ)` plus every horizontal `ref` / `share` / `rebind`
dependency target. At the Object level this still runs through
`Children_Val1 union Children_Val2`, including nested products, Sequences,
callables, and navigable `Val2` structures; the τ-level entry is what makes
`V_τ` — its closures, their anonymous types, and their captures — part of the
escape check rather than an implementation guess. Thus no returned branch can
smuggle unrelated meta-local material out of the invocation, and no `V_τ`
member can escape the closure check by being reachable only through the
callspace.

Promotion is likewise defined at the τ level:

```text
OwnedResultClosure(τ)
    = OwnedClosure(Core(τ))
      union OwnedCallSpaceClosure(CallSpace(τ))

OwnedCallSpaceClosure(CallSpace(τ))
    = least closure of the CallSpace(τ) members — including the V_τ closure
      anonymous types A_F and their () leaves, per the §2.1 V_τ member
      closure-ownership theorem — under the owned navigation relation of τ
```

Horizontal borrow edges are not ownership and are never dragged into either
component:

```text
OwnedClosure(x) excludes every ref / share / rebind edge reachable from x
```

Edge classification is explicit:

```text
BoundRef / stable enclosing-root reference
    = dependency / backreference, not an owned promotion edge

ref / share / rebind target
    = escape dependency, not an owned promotion edge

external stable dependency
    = dependency leaf, not recursively promoted
```

For this promotion, “owned closure” is not arbitrary graph reachability. Let
`OwnedNavigation_Q(x, y)` hold only when `y` is a genuine direct child owned by
`x` in Q's construction tree; the callspace component uses the isomorphic
relation over `CallSpace(τ)`. Then `OwnedClosure(Q)` is the least closure under
that relation, subject to all of these invariants, applied component-wise:

```text
direct child only:       every step is parent -> direct child
no jump:                 a parent cannot inherit a deeper descendant directly
bare termination:        Bare(x) stops expansion for the component
external termination:    ExternalTo(component, x) is an opaque dependency leaf
no external re-entry:    expansion never leaves the component, enters an
                         external subtree, and later re-enters owned material
no cycle:                 x not-in OwnedNavigation_component+(x)

OwnedNavigation_Q(x, y) => DirectOwnedChild(x, y)
Bare(x) | ExternalTo(Q, x) => no y: OwnedNavigation_Q(x, y)
ExternalTo(Q, q_i) => no j > i: Owner(q_j) = Owner(Q)
```

Borrow edges remain excluded from both components of `OwnedResultClosure(τ)`
and are never promoted merely because they are referenced.

External leaves may retain their own independently owned trees, but those trees
are not promoted through `τ`; their dependencies must already be globally
stable. The ordinary recursive Object normal form still traverses
`Children_Val1 union Children_Val2`; this construction judgment only determines
which fresh-owned part may acquire M's global lifetime.

A member reachable only through a borrow view is therefore not promoted, and its
presence does not extend `M`'s owned material. Its target must already satisfy
the escape condition. After the seal step, `M` is sealed and nothing may reopen
it.

#### 4.3.3 `M` as a navigable layer

Every ordinary canonical meta-function invocation establishes a virtual
symbolic-navigation and construction-authority scope:

```text
M = MetaInstanceScope(callee_symbol, canonical_arguments)
```

`M` is the `MetaInstanceRoot` of §2.1 — the symbolic-navigation, stable-identity,
and construction-authority anchor of the invocation. It is **not** itself the
result value: the default result is `τ_M` with `Root(τ_M) = M`; an explicitly
`symbol`-typed result is a `Symbol` value `Σ : symbol` (`ShapeOfTypeSymbol`).
A `NameBinding` or installation is a separate outer-graph binding/assembly
operation and does not constitute the result ontology.

Formation additionally requires:

```text
for every canonical argument a:
  GlobalKeyable(a) ∧ MetaArgumentAdmissible(a)

OwnedDependency(a) != GlobalKeyDependency(a)

Borrow(q) in a
  => Target(q) in GlobalKeyDependency(a)

GlobalKeyable_Γ(a)
  <=> every d in GlobalKeyDependency(a) is, at key-creation time,
        AlreadyGlobalStable_Γ(d)
      | AlreadyPromoted_Γ(d)
```

A meta invocation is a new stable MetaInstance construction boundary, so its
arguments must carry no PatternValue dependency that cannot survive globally:

```text
MetaArgumentAdmissible(a)
  => GlobalSurvivable(a)

GlobalSurvivable(a)
  <=> every dependency d reachable from a is globally survivable:
       direct PatternValue dependency
     | PatternValue held inside a carried type (τ)
     | dependency reachable through a type ref / type share target
     | nested dependency in Val1 / Val2
     | other escaping semantic dependency

GlobalSurvivable(a) ≠ GloballyVisible(a)
```

A value may survive globally without being name-visible everywhere, and a
PatternValue visible in the current lexical scope whose lifetime ends with the
current meta invocation is **not** admissible as an argument of a deeper meta
invocation.

A binder local to a meta invocation is not rejected merely for being local: if
it holds a canonical value whose dependencies are already global-keyable, that
value may enter the key. What is rejected is a fresh ephemeral PatternValue
dependency or a borrow of a meta-local place entering a new `MetaInstance` key.
A closure that might be promoted only when an enclosing meta invocation later
seals is not `AlreadyPromoted` for an inner key created now. `compile` and transparent
construction intrinsics impose no such boundary because they establish no
`MetaInstance` key and no new root.

For:

```lang
let f = (self, t: type): meta -> r: symbol => { ... };
```

the diagnostic navigation projection of `M` is:

```text
(t f)
```

This is not merely a folder analogy. `M` is a symbolic-navigation layer that
participates in default pattern navigation and name shadowing; the stored
complete type closure and typed value members belong to `τ_M`'s `Core(τ_M)` and
`V_τ` (not to `M` as a Symbol). `M` anchors cache/incremental identity and owns
the return construction transaction.

The default result is `τ_M` rooted at `M`; an explicit `symbol`-typed result is
a `Symbol` value `Σ : symbol` governed by `ShapeOfTypeSymbol`. The declared
return slot is a lexical name for the result value, not a transferable
construction class:

```text
ResultValue = τ_M,  Root(τ_M) = M        (default)
ResultValue = Σ : symbol                (explicit symbol)
return_slot(r) = NameBinding of τ_M / Σ (lexical name, not a result class)
```

The slot name `r` does not add another component to the final navigation path.
Material written through `r` contributes role/value members or children to
`τ_M` rooted at `M`; it does not
create `r::M` or place an extra symbol named `r` beneath `M`. For example, a
pattern-child contribution written as `let t1::r = bool;` inside the invocation
targets `t1`'s `M`-rooted `τ_M` under the applicable pattern-construction
expectation, not `t1::r::M`.

Canonical argument identity follows parameter rank:

```text
symbol parameter -> SymbolId / symbol-place identity
type parameter   -> default Core(tau) = Q observation; `TypeValueId` is only
                    the implementation/index projection, not semantic equality;
                    whole-snapshot Addr(Norm_type(tau)) identity applies only
                    where the language has independently frozen it
value parameter  -> PatternValue identity
```

The exact inclusion of `PlaceId` in a symbol-parameter key depends on whether
the callable observes the Symbol's installation place. A key must not silently
replace Symbol identity with type-value equality.

### 4.4 Ordinary meta return self-root invariant

If the return value of an ordinary canonical meta invocation carries a
complete type value `τ`, its installed type core `Core(τ)` — the structural
material that anchors the returned role root — must have its outermost
pattern root at the invocation's own `M`:

```text
τ present
  => Pure(Core(τ))
   and root_pattern_scope(Core(τ)) = M
```

This is identity equality between a pattern root and the meta-instance symbol
scope. It is not equality of rendered strings. The root identity is:

```text
MetaRoleRoot = MetaFunctionIdentity
             + Normalize(Arguments where every argument is GlobalKeyable)
```

Nodes beneath the root compare by normalized value: same root and same
normalized value imply the same pattern node. Source spelling, source symbol
names, and provenance do not participate in node equality.

Consequently, both of these meta bodies are invalid:

```lang
let f = (self, t: type): meta -> r: symbol => {
    let r = t;
    r;
};

let fn = (self, t: type): meta -> r: symbol => {
    let r = uint8;
    r;
};
```

The right sides are valid external type values, but their `PatternValue` roots
belong to external scopes. Resolving `symbol(t)` or `symbol(uint8)` and reading
its value does not make that external root identical to `(t f)` or `(t fn)`.
Neither value may directly replace the returned result's required role root.
The failure is the hard diagnostic `MetaReturnRoleRootMismatch`. An
implementation must not silently repair the mismatch by wrapping the external
value in a synthetic self-rooted node; check failure is failure.

A legal meta construction builds under its own scope:

```lang
let f = (self, t: type): meta -> r: symbol => {
    let r = (t inner) |> struct;
    r;
};
```

Its complete pattern is:

```text
(t inner::(t f))::(t f)
```

External `PatternValue`s may be members of the self-rooted core; they may not
replace the root. For example:

```lang
let fn = (self, t: type): meta -> r: symbol => {
    let t1::r = bool;
    r;
};
```

keeps `(t fn)` as the returned result's root and includes the externally owned
`bool::` value as a member beneath that root. It must not be summarized as
`NamespaceCoreProjection(r) = bool::`.

The self-root check is conditional on the installed type core `Core(τ) = Q`, not
on `TypeRole(Q)`. A namespace-only `Q` — `NamespaceRole(Q)` and
`not HasRegisteredSelfConstruction(Q)` — is self-rooted and may own fresh
invocation-local material. A returned result with no installed type core
does not acquire a synthetic core merely to satisfy this rule. When
`TypeRole(Q)` does hold, it is the additional type
refinement (imported judgment); namespace-only `Q` is not required to define Val1.

### 4.5 Formal return material

Target semantics do not give the spelling of a return slot a special creation
meaning. A meta body computes its result value (`τ` by default); `let` creates its local
members, `=` writes existing places, and the return event transfers that value.
The explicit return-slot spelling `r` denotes the declared return position; it
does not create a construction-value ontology.

Formal meta return material is a family of distinct construction-effect forms,
not one spelling-insensitive binding. Create, write, and deliver are distinct
events that never collapse:

```text
    let x = expr;     -> creates a fresh Symbol/member according to the
                         declaration context
    target = expr;    -> Write(existing target, expr)
    return event      -> control transfer only
```

- `target = expr;` writes to an existing target; a write is not append, and a
  construction model that only supports appending cannot express
  `let x = first; x = second; return x` by treating both operations as
  contributions.
- A return event delivers its value to the selected enclosing layer. It is not
  a member contribution and does not give the return-slot spelling special
  binding semantics.

Source wiring for expression-level write and general construction effects is
pending. An unavailable source operation does not acquire a spelling-directed
substitute.

The terminal family follows the general control-flow end model: `expr;`
delivers to the directly enclosing layer, `expr return;` returns to the
outermost function layer, and `expr (T return);` returns to the layer selected
by the function-object type `T`.

Add-fresh-member and write-to-existing-target are two distinct construction
effects. They must not be collapsed into one injection event, and neither is a
return. Whether contributed material references an existing `PatternValue`,
computes new material, or projects a Symbol member is represented inside the
construction value; any resulting type core `Core(τ)` must pass the self-root invariant in
§4.4.

There is no fourth "alias member" event. A member is created by `let`, written by
`=`, and nothing forwards an external symbol's `Val2` material into a member.
Where shared observation of an external object is wanted, the member holds a
borrow view (`ref` / `share`), which is an ordinary value and is subject to the
ordinary member rules — including the rule that a borrow edge is not owned
material and is therefore not promoted at seal (§4.3.2).

#### 4.5.1 `let` creates, `=` writes, the return event transfers control

The three rules are orthogonal:

```text
let   — only creates a new Symbol/member (never writes existing targets)
=     — only writes to an already existing target
return event — produces control return, independent of whether a return
        value was written
```

Consequently:

- `let r` may shadow the return value, because `let` creates and `=` writes;
- explicit return uses the return event mechanism — return depends on the
  event, not on whether a return value binding was written;
- even if an explicit return value exists and has not been shadowed, return
  still requires a return event to produce control flow;
- writing to an explicit return value after which control does not return is
  analogous to dead code — not erroneous, because intermediate computation may
  have side effects.

Assignment is itself an associated operation. The source spelling `=` selects an
ordinary assignment candidate; only the selected candidate's default
implementation performs the universal write judgment below. There is no
compiler primitive `Write` behind the source spelling, and no assignment
candidate exists merely because a checker could prove the place writable —
write capability is exposed by the selected associated callable, not invented
by `mut` policy.

The default `=` entrance forwards through the operator/ADL path, not through
special compiler logic that inspects the LHS and searches for an assignment
family:

```text
operator[=]   -> .=
.=            ≡ =::adl
```

Required source behavior:

```text
object : T        object ref = value   -- form ref, then .=
object : T ref    object = value        -- direct .= on the ref's target
```

`NoImplicitBorrowFormation` remains absolute: an ordinary `T`-valued LHS
never secretly forms `Ref(CarrierPlace(lhs))` (`AssignmentReceiverFromPlace`
is forbidden). When the receiver is already `T ref`, assignment writes
`Target(receiver)`, not the place storing the ref handle. Custom Val2 may
define setter candidates through `.=`; setter participation does not make
anything a P structural field.

The `=` family for `T ref` is:

```text
AssignmentFamily(T):

  =
  (self,
   mut let object : T ref,
   other : T)
  -> unit
  => default

  =
  (self,
   const let object : T ref,
   other : T)
  => delete

  =
  (self,
   let object : T ref,
   other : T)
  => delete
```

Only the selected `default` performs the universal write judgment below. The
three layers are thereby fully separated:

```text
policy
    controls which = candidate wins

selected = candidate
    exposes the write operation

Write default
    validates the actual place
```

`T share` provides no `=` family at all: `share-value = other` yields **no
applicable overload** in the candidate domain, never a selected assignment that
then fails `Writable`. `AssignmentFamily` here is the universal `T ref × T`
family. Field-specific write candidates (`FieldWriteFamily(T, name, A) ⊆
Candidates(=::adl)`) are a distinct ordinary associated family for every `A`
— including `A = T`: shape coincidence (both `T ref × T -> unit`) is never
family identity, because the field family's target operation is
`field(receiver, name)` while the universal family's is `Target(receiver)`;
selector entry and family identity are normative in
`type-associated-function-objects-and-access-trees.md`. Assignment carries no `extend`-specific validation, but
that is not the same as carrying no validation. A pure `extend` in the right
side already discharged `Open ∧ ParentToChild ∧ NoPatternConflict`. The
place-level `inject` wrapper performs that check before its own write.
Everything else that applies to any write still applies. After the assignment
candidate is selected, the write `lhs = rhs` is checked in four independent
layers:

```text
1. RHS operation legality
     Evaluate(rhs) ⇓ v
     -- an extend inside rhs checks its own Open here, not at the write

2. universal write applicability
     Writable(lhs)
     Compatible( P(lhs), v )
     ValidCapability(lhs)
     Contents(lhs) = Some(old)
     -- a type share is not a write target; bare = never creates None

3. result-object invariants
     WellFounded_kappa(v)
     Canonicalizable(v)
     NoForbiddenCycle(v)
     -- a write forming a non-normalizable Val2 cycle fails, even when it comes
        from an ordinary assignment

4. semantic-boundary constraints of the enclosing region
     meta return self-root; ref / pattern-value lifetimes;
     mutability limits on global type-bearing values; seal / global-promotion
     rules; the single-τ-installation bound on a returned result — the
     installed type value slot is optional (`τ?`), so a result installs at
     most one τ by shape, never by counting cores
     -- these may run at write time, normalization time, return time, or
        install time, but they all remain in force
```

Assignment RHS semantics are explicitly value semantics:

```text
AssignmentRHSIsValueSemantic:

object : T ref
other  : T
```

`other` is a genuine `T` value. There is no implicit dereference
(`T ref -> T`), no implicit clone (`T share -> T`), and no reading of a borrow
handle's referent bytes as if they were the value (`T ref -> T ref` memcpy).
In assignment candidate adaptation:

```text
T ref/share =/=> T
```

must hold. If `T ref |> cloneable == true` or `T share |> cloneable == true`,
the legal path is an explicit/independently selected clone producing a `T`
value, then `=`; assignment never secretly clones or dereferences. This is
orthogonal to `NoImplicitBorrowFormation`:

```text
NoImplicitBorrowFormation     forbids T -> T ref/share
AssignmentRHSIsValueSemantic  forbids T ref/share -> T
```

both directions are closed.

The two validation families are therefore distinct and must not be conflated:

```text
ExtendSpecificValidation             -- discharged during RHS evaluation, once
UniversalObjectAndBoundaryValidation -- always applies to the write result
```

Assignment does not inspect how the right value was produced: it asks for no
provenance, no construction witness, and no transition proof from any particular
producer, and a value that conforms to the target Pattern is acceptable
regardless of which operation built it. That freedom covers layer 1 only. It
does **not** exempt the result from layers 2–4 — the write result must still
satisfy every ordinary type, capability, lifetime, normal-form, and boundary
invariant.

This distinction does not cancel `let f::(t |> (type ref)) = expr` for an
already-pure type slot, or `let f::((S ref).type) = expr` for a Symbol whose
`Q` satisfies `TypeRole` (ordinary `Val2` member creation at an explicit type
place), and does not change the `r;` terminal semantics.

A successful construction returns the semantic entity declared by the selected
callable's result class. A fresh returned Symbol has its own `SymbolId` and, once
bound, a fresh destination `PlaceId`; its ordinary value or member material may
reuse existing Pattern values. Construction effects and replay provenance are
execution material, not a second value ontology.

Value equality remains independent of source name and navigation path and does
not merge symbol or place identity. However, that general identity separation
does not waive the meta return self-root invariant (§4.4): `r = uint8` as a direct meta
return core installation is rejected after symbol resolution/value read, rather than
being reinterpreted as forwarding or accepted as an identity meta type.

### 4.7 A `Symbol` constructor value is an ordinary PatternValue

A `Symbol` constructor value is not a separate ontological rank. It is an object with the same
three components as every other object:

```text
SymbolValue = ⟨ Σ, P_symbol, Val2_symbol ⟩

Σ = ⟨ tau?, V_S ⟩
V_S = ⨄_{T_c} V_S[T_c]
V_S[T_c] : T_c * omega
```

Its member content is ordinary object content:

```text
optional complete type value tau
any number of ordinary sibling val members
```

Because the member content is the mutable part, it lives in `Val1`:

```text
Val1(Symbol) = Σ = ⟨ tau?, ⨄_{T_c} V_S[T_c] ⟩
```

`Σ` is a logical view over ordinary Object containers, not a
specification-private record carrier. Using the constructor lemmas in
`type-values-places-and-borrow-views.md`:

```text
TypeOption(absent) = BareProduct()
TypeOption(tau)    = BareProduct(LowerTypeClosure(tau))
                     where WellFormedTau(tau)

LowerTypeClosure : WellFormedTau -> Object
-- lowering/representation only: used when an implementation stores tau in
   an Object-position carrier (e.g. Σ_Object); NOT derived from
   ¬Object(τ), NOT a precondition for ordinary semantic operations on τ
DecodeTypeClosure(LowerTypeClosure(tau)) = tau

Fidelity (representation faithfulness):
  Norm(LowerTypeClosure(tau_1)) = Norm(LowerTypeClosure(tau_2))
    iff Norm_type(tau_1) = Norm_type(tau_2)
  -- LowerTypeClosure is injective up to Norm_type: two closures lower to
     the same normalized Object exactly when their normalized type values
     are equal (Norm_type as defined in
     type-values-places-and-borrow-views.md §2.1); the lowering introduces
     no extra observable distinction beyond the tau API

BucketEntry(T_c)     = ProductValue(T_c, V_S[T_c]) : product
BucketCarrier(V_S)   = Seq_omega(product; BucketEntry(T_c) for each occupied T_c)

Σ_Object(tau?, V_S)  = BareProduct(TypeOption(tau?), BucketCarrier(V_S)) ∈ Object
Val1(Symbol)         = Σ_Object(tau?, V_S)
```

The notation `⟨tau?, V_S⟩` merely projects the two ordinal positions of this bare
Product Object. `tau` itself is a semantic package; when an implementation must
store it in an Object-position carrier, the Symbol's
Val1 stores its lowering `LowerTypeClosure(tau)`. Every
`V_S[T_c]` is itself the ordinary `T_c * omega` Sequence
Object, and every bucket entry is classified by the global `product` type so
the bucket carrier remains genuinely homogeneous. Symbol normalization applies
its unordered quotient to this ordinary carrier; neither `Σ` nor its buckets
introduce a compiler-private semantic collection.

The lowering is representation-opaque: ordinary Pattern, Object navigation,
and Val1/Val2 inspection semantics must not observe any extra distinction
beyond what the `tau` API defines through `LowerTypeClosure`. The lowering is
the single canonical representation inside the Object ontology; it does not
form a second observable identity system.

Each `V_S[T_c]` contains ordinary member/candidate objects of their actual type
`T_c`. Those objects preserve stable declaration/candidate identity, their
complete value or callable body, and every annotation that affects semantics
through their own ordinary recursive identity. Symbol is not a set of erased
callable bodies, and there is no universal `SemanticMember` wrapper type.
Symbol mutability is not mutation of `P_symbol × Val2_symbol`.

The typed buckets use the ordinary built-in finite-sequence PatternValue family.
The minimum public container kernel is:

```text
T * N      =  T^N              -- exactly N objects of T, N a compile-time count
T * omega  =  ⨄_{n in ℕ} T^n     -- some finite T^n; n is not type identity
```

These are formal language type constructors, not specification metavariables.
The global privileged type-forming builtin `*` supplies the reconstruction:

```text
*(T, N)     -> T * N          where N is a compile-time natural number
*(T, omega) -> T * omega

rank(T * N)     = rank(T)
rank(T * omega) = rank(T)
```

`*` establishes no ordinary `MetaInstanceRoot`; its member-declared privileged
owner rule derives the result from the element type and shape argument. Like the
borrow modalities, the container modality does not climb the type universe.

Both are finite, homogeneous, anonymous, and ordered. `N` enters the type
identity of `T * N`; the concrete length of a `T * omega` value remains in its
`Val1` but does not enter the outer type identity. There is a canonical
shape-erasing conversion:

```text
T * N -> T * omega
```

Neither family promises a machine array, contiguous layout, capacity,
`push_back`, or any growth API. Their mechanically generated `[]` associated
Val2 is bounded indexed observation over the same ordinal `ProjectionSlot`
mechanism as named fields:

```text
Index : T * N         x number ->? T
Index : (T * N) ref   x number ->? T ref
Index : (T * N) share x number ->? T share

Index : T * omega         x number ->? T
Index : (T * omega) ref   x number ->? T ref
Index : (T * omega) share x number ->? T share

Dom(Index) = { (s, i) | 0 <= i < Length(s) }
ElementBase(s) = Val1(s) = BareProduct(v_0, ..., v_(Length(s)-1))
IndexSlot(s, i) = ProjectionSlot(Resident(ElementBase(s)), pos_i)

Index(s, i)       = Read(IndexSlot(s, i))  within Dom(Index)
Index(s ref, i)   = Ref(IndexSlot(s, i))   within Dom(Index)
Index(s share, i) = Share(IndexSlot(s, i)) within Dom(Index)
CanCreateMember(sequence, pos_i) = false
```

The value/ref/share candidates have the same bounds domain; view kind changes
only observation capability. Out-of-domain behavior (diagnostic, trap, proof,
or error representation) remains deferred, so `Index` is not total.

The heterogeneous counterparts are bare Product and the global built-in
`product` type. A bare Product has a fixed concrete arity/type vector. A value
classified by `product` retains any finite bare Product in `Val1`, while that
arity/type vector is erased only from the **outer classifier**:

```text
let p: product = (a, b, c);
Val1(p) = (a, b, c)
```

No element information is erased from `Val1`. General runtime `product[]`
remains undefined because a sound result needs dependent/existential result
material or a type witness. The four ordered-container cases are:

| element shape | fixed concrete outer shape | erased outer shape |
| --- | --- | --- |
| homogeneous | `T * N` | `T * omega` |
| heterogeneous | bare Product | `product` |

The Symbol Pattern applies an unordered identity quotient to each typed bucket:

```text
DecodeSymbolPayload(Σ_Object) = ⟨ τ?, V_S ⟩

Norm_Val1?^P_symbol(Σ_Object)
  = ⟨ Norm_type(τ)? ,
      { Norm(T_c) ↦ Set{ Norm(v) | v ∈ V_S[T_c] } } ⟩
```

If distinct `T_c` keys normalize equally, their buckets are combined under that
normalized key before the set quotient. Carrier position, insertion order, and
replayed contribution of the same stable member do not enter Symbol identity:
`Σ + Σ = Σ`. Duplicate declarations, conflicting definitions, and same-root
conflicts are diagnosed in construction/well-formedness before normalization;
they are not remembered as value multiplicity. Distinct stable member objects
remain distinct even when their callable bodies normalize alike. In particular,
`s += a; s += b;` and `s += b; s += a;` normalize equally exactly when their
stored `tau` (if any) and every typed member set are equal.

Callable val members project the formal overload set directly from this value:

```text
OverloadSet(Σ, q)
  = ⨄_{T_c} { v ∈ V_S[T_c] | Callable(v) ∧ q(v) }
```

This is an ordinary projection from the typed Symbol buckets, not a
resolver-private multiset. An implementation may use heterogeneous registries or
Rust vectors to transport these objects, but such storage is not a language
container and never contributes value identity.

The global `symbol` type demonstrates the same generated-field mechanism. Its
ordinary associated Symbol named `type` contains:

```text
type : (object: symbol)       -> type
type : (object: symbol ref)   -> type ref
type : (object: symbol share) -> type share

Applicable(type candidate, Σ) <=> TypeSlot(Σ) = Some(τ) and TypeValueRole(τ)
```

Thus `S.type` agrees by value with `AsType(S)` and returns the complete stored
`τ`, while `(S ref).type` and `(S share).type` return a borrow observation of
the type-valued slot when `TypeSlot(S) = Some(τ)`.
This is ordinary field/candidate selection, not a resolver primitive that
projects a value and then recovers its provenance.

The consequence is that Symbol-level operations are `Val1` transformations and
leave the Symbol's own pattern untouched:

```text
s = new_symbol                   -> replaces Val1(s)
s += contribution                -> extends Val1(s)
s -= contribution_family          -> removes a typed contribution family from Val1(s)

in every case:  P_symbol unchanged
```

These equations are value-algebra shorthand describing how `Val1(s)`
transforms. They are not source-level elaborations that implicitly form
borrow edges or adapt `s` into `s ref`. An explicit source operation
would spell `s ref += contribution` and satisfy the ordinary borrow
formation boundary (§9).

A `Symbol` constructor value is therefore an ordinary value that can be computed, passed, and
returned like any other — including by `compile`, subject only to root
conservation (§4.2.1). The four roles listed in §4.1 (value ontology, meta return
construction, namespace same-name synthesis, world installation) are separate
concerns that happen to involve bindings; none of them is the Symbol's ontology.

### 4.8 Built-in privileged AST meta functions

A compiler-defined privileged family uses the general function-object and meta
invocation framework without becoming user-definable macro capability:

```text
BuiltinPrivilegedAstMetaFunction {
    compiler_known_identity,
    accepted_normalized_ast_or_pattern_rank,
    required_ambient_construction_capability,
    declared_result_pattern,
    special_scope_rule,
    special_owner_rule,
    bounded_privileged_behavior,
}
```

These objects:

```text
participate in ordinary symbol-first lookup;
have function-object, type, and associated () identity;
use the ordinary invocation frame, including implicit self;
may accept a bounded Normalized-AST or pattern carrier;
establish no ordinary MetaInstance root;
return ordinary PatternValues or complete type values rather than a construction class;
declare explicitly whether they are pure or write an existing place.
```

Privilege buys a bounded AST carrier and a special scope/owner rule — it buys no
result ontology. There is no shared "construction handle" return family and no
third result class (§4.1):

```text
extend  : type × StructLikeMaterial -> type
inject  : type ref × StructLikeMaterial -> type ref
struct  : StructLikePattern -> tau
*       : type × (CompileNatural | omega) -> type
```

Unlike an `OrdinaryMetaFunction`, an individual built-in defines a
member-specific scope/owner rule and does not create an independently navigable
`MetaInstanceScope M`. Users may call compiler-provided members but cannot
define new privileged AST meta functions. Privilege is member-specific: one
built-in's accepted carrier and bounded transformation do not imply a general
macro system or arbitrary AST rewriting.

The ordinary-meta root-establishment rule in §4.1 governs only a navigable
`MetaInstanceRoot`; it does not claim authority over every stable owner/root in
the language. Built-in root behavior is therefore split by member rather than
inferred from the privilege class:

```text
ordinary meta:
  require GlobalKeyable(Norm(args))
  establish NavigableMetaInstanceRoot(MetaInstance(F, Norm(args)))

struct:
  establish or select StructLexicalRoot(input_navigation, ambient_scope)
  according to §7.2; establish no navigable M

extend:
  establish no root
  Root(output) = Root(input)

inject:
  establish no root
  read the target, call extend, and write the result to that same target

*:
  establish no navigable MetaInstance root
  derive T*N or T*omega from the normalized element type and shape argument
  preserve rank(T)

other privileged built-in:
  must declare its own special_owner_rule and special_scope_rule
  before it can produce rooted material
```

A special owner rule cannot be used as an alternate route to an ordinary
navigable `M`. Liveness, visibility, borrowability, and installation of a
built-in result follow the particular member rule and ordinary outer binding;
the privilege class supplies no generic conclusion that every result is rooted
under the call-site `Self` chain, has global root identity, or is externally installed.

`struct`, `extend`, and `inject` are the first specified members. Future candidates may
include explicit sum construction/extension, bounded AST injection, or a
facet-construction primitive, but each must receive its own capability boundary.

## 5. Physical Namespace Contributions and Meta Construction

Physical source contributions and meta-produced construction values use the
same symbol-world capability substrate.

For example:

```text
ns/
  impl.lang
  export.lang
```

Both implementation files may create distinct same-level children in namespace
`ns`.
The corresponding meta-shaped construction can be sketched as:

```lang
let ns = (): meta => {
    let r = ...;
    let r = r |> impl;
    let r = r |> export;
    r;
};
```

The example is semantic design notation. It does not introduce a new parser
special form or promise that these exact bodies execute in the current
implementation.

Both origins share capabilities for:

```text
declare symbol/facet material
inject a direct child into a construction
extend the navigable structure of the current `Core(τ)`
form a replayable contribution/delta
install a delta transactionally at the outer assembly/binding layer
```

Sharing a capability substrate does not give physical files an implicit meta
pipeline execution order:

```text
physical source fragments
  -> independently derived contribution/delta values
  -> transactional assembly of distinct direct-child deltas
```

The contribution set is not evaluated as `impl.lang |> export.lang` according
to filename, discovery, or source order. Each file is nevertheless a distinct,
closed `SourceConstructionUnit`: it may create and fully construct its own new
child subtree, but it may not reopen a child subtree created by the other file.
Distinct direct-child contributions can be installed transactionally;
same-child reopening, duplicate names, or incompatible facets are conflicts. No
partial merge is installed after failure.

The canonical namespace-origin, construction-unit ownership, physical-directory
authority, and cross-file merge rules are specified in
`symbol-construction-units-and-namespace-origin.md`.

## 6. Resolved Pattern Scopes

### 6.1 One uniform scope model

The canonical object is:

```text
ResolvedPatternScope
```

or, when emphasizing ownership:

```text
ResolvedOwnerPatternScope
```

A meta-function instance is itself a navigable pattern scope. The design does
not split construction into separate special cases based on whether source
syntax contains a distinguished outer pattern name.

Example:

```lang
let f = (self, t: symbol): meta -> r: symbol {
    let r = (t first, t second) |> struct;
};
```

The current meta instance may have this diagnostic projection:

```text
(t f)
```

The fully resolved pattern is:

```text
(
    t first::(t f),
    t second::(t f)
)::(t f)
```

The single-field form uses the same rule:

```lang
let f = (self, t: symbol): meta -> r: symbol {
    let r = (t first) |> struct;
};
```

Its fully resolved pattern is:

```text
(t first::(t f))::(t f)
```

The two examples do not represent “no top pattern” versus “a top pattern.” They
are both:

```text
explicit relative pattern components
  + ambient navigable pattern scope
  -> fully resolved pattern path
```

The explicit relative component may be empty. The ambient scope still exists
and still owns the resolved pattern layer.

### 6.2 Scope identity is not rendering

Forms such as `(t f)`, `first::(t f)`, or `first::t1::t` are diagnostic
projections. `ResolvedPatternScope` identity is not raw string concatenation.
Implementations may eventually represent it with a `PatternScopeId` plus
structured owner/child relations.

### 6.3 An ordinary meta invocation is one navigation atom

When an ordinary meta callee has an outer namespace path, the complete
invocation remains
one navigable symbol atom. If `Vec` is found under `std` and the argument is
`int`, the canonical form is:

```text
(int Vec::std)
```

Resolution proceeds as:

```text
resolve callee path Vec::std
  -> resolve argument int
  -> form canonical meta invocation
  -> treat the complete invocation as one navigable symbol atom
```

A child of the resulting instance is written:

```text
child::(int Vec::std)
```

These are not equivalent forms:

```text
(int Vec)::std   // invalid: invocation boundary cuts off the callee path
int Vec::std     // invalid: missing invocation-atom parentheses
```

The future semantic grammar may name this unit:

```text
MetaInstanceNavigationAtom :=
    '(' ArgumentProduct MetaCalleePath ')'
```

This semantic/navigation rule is not part of the current lexer, parser, Raw
AST, or Normalized AST surface.

## 7. `struct`

### 7.1 Public boundary

`struct` is a `BuiltinPrivilegedAstMetaFunction`, not an ordinary user-definable
meta function. It uses the general function-object/meta call framework but does
not create its own ordinary externally navigable `MetaInstanceScope M`.

The public semantic boundary is:

```text
struct:
  StructLikePattern
  -> tau
```

An implementation may carry AST or Normalized AST as a private structured
carrier. The public result is a complete type value `tau`, not AST, not an
ordinary Symbol PatternValue, and not a separate construction class (§4.1,
§4.7–§4.8). The formation event is:

```text
struct(P)
  = tau_P
  = bind alpha.<Q_P[alpha], V_τ[alpha]>
```

where the core `Q_struct = Core(tau_struct)` is produced
during the formation event, satisfying `TypeRole(Q_struct)`, and the
direct TypeMembers generated during that formation event enter `V_τ`
immediately; there is no intermediate Symbol from which `Q_struct` or `V_τ`
is later projected. Section 7.5 closes the mechanically generated
field/access/ref/share/assignment partners in that complete type snapshot and
exposes corresponding associated views. Other direct-home TypeMembers, when
present, are likewise part of that snapshot's `V_τ`; type-as-callee never
recovers a defining Symbol. This bounded capability does not expose a general
macro system.

In the complete-type notation this producer-specific guarantee is:

```text
Core(struct(material)) = Q_struct
Pure(Q_struct)
TypeRole(Q_struct)
CallSpace(tau_struct) = V_τ
```

Thus general Symbol and ordinary-meta ontology use optional pure `Q`; `struct`
specifically guarantees that its core `Q_struct` exists and is type-capable.
The two-step reading is preserved: `struct(P) -> tau_P` is a formation event,
while a subsequent `let t = P |> struct` is the binding that creates the Symbol
`S_t = <tau_P, V_St?>` (§6). These are consecutive but distinct semantic steps;
the type closure is formed by `struct` alone, before any Symbol is installed.

### 7.2 Owner resolution

`struct` resolves its pattern owner from:

```text
the input pattern's explicit navigation
+ the ambient ResolvedPatternScope
```

It does not inspect the eventual left-side binding target.

The invariant is:

```text
struct pattern owner:
  determined by input pattern material and ambient pattern scope

left-side let binding/installation path:
  determines only the Place where the construction is installed
```

**In-place closures are transparent to `struct` navigation in meta context.**
When `struct` resolves the ambient `ResolvedPatternScope`, it sees through any
in-place (inline-called) closures within the meta body until it reaches the
meta function call entry point. These intermediate closures do not contribute
navigation components to `struct`'s owner resolution:

```text
meta body:
  in-place closure invocation  <-- struct sees through this
    in-place closure body
      ... |> struct            <-- resolves owner at the meta entry scope,
                                   NOT at the in-place closure scope
```

Only non-meta contexts observe these in-place closures as affecting navigation
names. The rationale is: in-place closures within a meta body are control-flow
mechanisms (combinators, continuations, local abstractions) that do not
represent semantic ownership boundaries. The meta function call entry is the
canonical ownership boundary; closures called within it are internal structure.

Therefore:

```lang
let t1::t = (...) |> struct;
```

does not reroot the right-hand pattern into the internal pattern scope of
`t1::t`. Its effect is:

```text
evaluate the right-hand struct invocation
  -> obtain an uninstalled pattern value with an already resolved owner
resolve the destination symbol/place t1::t
  -> bind/install the construction result there without changing that owner
```

Every construction value must therefore distinguish:

```text
install_place(V)
pattern_owner(V)
```

The two identities may differ.

### 7.3 Formal invocation boundary

Formal `struct` invocation is:

```text
graph-installation-free
binding-free
referentially pure
```

Purity means that `struct` does not install a Symbol or mutate an
input place. It may establish the result type's declared `StructLexicalRoot`
under its privileged owner rule, but outer `let` remains the only operation that
creates the destination Symbol/member in the surrounding graph.

It does not install a `NamespaceDelta`. Registry-backed pattern material is an
implementation record: it may affect cache/storage mechanics but is not
observable in `Norm`, does not mutate language-visible input, and does not
weaken referential purity. Graph installation remains outside formal
invocation.

### 7.4 Structural leaves and pure Pattern nodes

`struct` recognizes the shape inside each leaf parentheses. The value-bearing
leaf form is:

```text
Expr name
```

`Expr` supplies the leaf value/type material and `name` supplies that leaf's
Pattern name. This resembles the token order of a C-style field declaration
only as a surface mnemonic; it imports no C type, layout, object, or field
semantics.

A single `name` with no preceding `Expr` is instead a pure Pattern node:

```text
name
  -> null x P(name) x Val2(name)
```

It has no Val1. This is the basis on which no-value alternatives such as
`if | else` remain visible Pattern material rather than being rejected as
missing fields.

A named empty Pattern is valid:

```lang
let t = (()t) |> struct;
```

Here `()` supplies an empty child layer and `t` supplies the Pattern name. The
result is not a value-bearing field. This rule does not by itself assign a
meaning to an anonymous bare `() |> struct`; that is a separate boundary.

### 7.5 Generated field and companion members

For a structural field `f : A` produced during the `struct` formation event, let
`tau_struct = struct(material)` and
`T = tau_struct`. The core `Q_struct = Core(tau_struct)` is produced during that
formation event; there is no intermediate `S_struct` from which it is projected.
`struct` uses one general field rule. It does not introduce a separate semantic
category for “type fields”. All observations are candidates of one same-name associated
Symbol `f`; receiver and result observation kinds distinguish the overloads.
The `struct` generator produces the full `GeneratedFieldFamily(T, name, A)` —
the by-value accessor plus the `ref`/`share` policy triples with their exact
`default` / `delete` cells (canonical schema in
`type-associated-function-objects-and-access-trees.md`). Erasing policy detail,
the family presents as:

```text
f : (object: T)       -> A
f : (object: T ref)   -> A ref
f : (object: T share) -> A share
```

`ref` and `share` are not generated navigation subspaces. The same-name family
is stored once as ordinary callable/member Objects. Its direct anonymous
classifier home is `TypeMemberScope(Q_struct)`, so it belongs to `V_τ`; `const
let` / `let` / `mut let` policy and the formal object type determine its
candidates.

The `ref` / `share` type constructions do not copy that family. With respect
to inherited associated names, each derived type value `T ref` / `T share`
generates fresh direct-home forwarding entries
(`ForwardAssoc`, §2.1 `NoForeignTypeMemberInjection`): `f::(T ref) ->
f::T` and `f::(T share) -> f::T` are fresh derived-type members homed in the
derived type's own `V_τ`, whose bodies perform a new ordinary invocation of the
base family. The model is therefore:

```text
struct
    generates the real field family under T

ref/share type construction
    for inherited associated names:
        generates fresh direct-home forwarding entries
    derived τ still owns its intrinsic
        ref/share formation, borrow formation,
        fixed-point/weakening, and other native callspace members
```

no foreign callable object ever enters a derived `V_τ`.
Their selection uses the ordinary context-indexed preference relations. In a
plain context `succ_plain: plain > const = mut`; if no `plain` candidate is
admissible, a surviving `const` and `mut` pair remains ambiguous rather than
being resolved by generation order.

Where the field policy permits mutation, the same generator also contributes
field write candidates shaped `T ref × A`. They form a field-specific setter
family `FieldWriteFamily(T, name, A) ⊆ Candidates(=::adl)` — an ordinary
associated family reachable through the same `.=` entrance, and **never** the
universal `AssignmentFamily(T)` (whose domain is `T ref × T`): for every `A`,
`FieldWriteFamily(T, name, A) ≠ AssignmentFamily(T)`; at `A = T` the two have
coincident formal shape only, never coincident family identity (canonical
field-side rules: `type-associated-function-objects-and-access-trees.md`).
Field write, accessor, and policy cells are all registered under the stable
call-site family identity `StructuralFamily(T, name, A)` =
`StableFamilyId(CoreAnchor(Q_T), name, StructuralDefault)` that P-internal
extraction filters on; the identity key is the stable core anchor
(§2.1), not the whole `Q` snapshot. Family registration and the stability
theorem are normative in
`type-associated-function-objects-and-access-trees.md`. Assignment still uses
the general existing-place write rule and never creates the field. Written
`const let` / unqualified `let` / `mut let` field policy selects the admitted
value, shared, mutable, and assignment cells of this ordinary overload
family. The exact machine body and access-tree representation are implementation
debt, not additional semantics.

Accessor stage follows one structural predicate rather than a coarse “type” or
“PatternValue field” category:

```text
RuntimeField(f)
  <=> Val1_f != absent
    and Materializable_0(Val1_f)
    and not RequiresStaticPattern(f)

Stage(accessor(f))
  = runtime || compile   if RuntimeField(f)
  = compile              otherwise
```

`Materializable_0` means that the current first-order runtime object model can
materialize the complete selected `Val1` without a static-only witness;
`RequiresStaticPattern(f)` means that selecting or constructing the field
observation intrinsically depends on PatternValue material unavailable at
runtime. Both are structural judgments over the field object, not nominal type
lists.

A type-valued field is compile-only only because it fails this predicate in the
current runtime model; it is not a special field category. Ordinary runtime
values remain PatternValues and are not excluded by that fact. The mechanically
generated `[]` observations of `T*N` and `T*omega` use this predicate for the
selected element but retain all ordinary call dependencies:

```text
Dependencies(Index(s, i)) = { container observation s,
                              index observation i,
                              selected element observation }
Stage(Index(s, i)) = meet { Stage(d) | d in Dependencies(Index(s, i)) }
```

`RuntimeField(selected element)` is one local condition inside that meet. No
Sequence-specific stage rule exists.

The generated partner candidates are ordinary members whose classifiers
satisfy `TypeMember_Q_struct`; they enter `V_τ` during the `struct` formation
event, and `Core(tau_struct) = Q_struct` exposes them as its associated members.
Any navigable associated
view is a projection of those same members, not a second owned copy in
`Q_struct` or its `Val2`. The partners are ordinary typed member objects: user
construction may remove them, replace them, or add a more specific declaration
subject to the ordinary duplicate, fallback, and overload rules. They are not
hidden compiler metadata.

The closed structural generator contract stops at the same-name field
value/ref/share observations and the corresponding assignment/write partners
described above:

```text
struct closure = field + access + ref/share observation + assignment/write partners
```

Type-as-callee is now closed without any defining-Symbol recovery:

```text
TypeValue(t) = tau = <Q,V_τ>
CallSpace(tau) = V_τ
```

A copied or extracted type value retains the `V_τ` of that immutable `tau`
snapshot. A complete type has no home Symbol and no reverse carrier, source-place,
or `AsType` identity route.

Open authority does not propagate along owned field relations. Each
PatternValue's open authority is determined independently by stack-relative
authority-frame resolution:

```text
OpenHere_Σ(v)
  iff WindowLive_Σ(v)
  ∧ AuthorityMatches(v, Σ)
```

No parent-to-child or child-to-parent implication holds; a terminal event that
closes multiple windows in one structural region does so because each value
independently fails `WindowLive_Σ` or `AuthorityMatches`, not because a
neighboring value closed. Borrow edges are horizontal and do not participate.
Mutability is independent:

```text
mut(child) does not imply mut(parent)
mut(parent) does not imply mut(child)
```

This same rule makes a typeclass-like object an ordinary struct; its fields are
compile-only exactly when they fail `RuntimeField`, not because they inhabit a
separate “type/PatternValue field” category.

### 7.6 Internal construction and later extension normalize equally

An element written inside the original `struct` input and an equal element
added later through the owner's navigated structural-extension path differ only
in **how their full navigation is obtained**. They do not differ in the Pattern
**entity identity** of the child. For example:

```lang
let t = ((bool inner)t) |> struct;
```

and the construction sequence using place-level `inject`:

```lang
let s = (()t) |> struct;
let t_ref = s |> (type ref);
(t_ref, bool inner) |> inject;
```

produce type values whose core `Q_struct` members both satisfy `TypeRole`,
provided the read value is Open and the destination slot is writable. Both
paths install exactly one canonical Pattern child under `t`:

```text
exists exactly one C.
  C = inner::t
  and DirectPatternChild(t, inner, C)
  and LeafSource(C, bool)
  and Norm(leaf value of C) = Norm(bool::)
```

The canonical entry is the child entity `C = inner::t` carrying its leaf value;
the same structural theorem is stated in
`../patterns-overload/pattern-values-relational-semantics-and-extraction.md` §12.
The two construction paths differ only in formation/navigation provenance:

```text
SameChildPattern(C₁, C₂) ∧ DifferentNavigationFormation(path₁, path₂)
  ⇒ SameCanonicalEntry(C₁, C₂)

-- the converse does not hold: erasing formation provenance never erases the
   child entity itself, and never equates distinct pattern children
```

The first form inherits/completes `inner` under `t`; the second supplies the
same child material through pure `extend`, then writes it back through the
type-level carrier slot reached by `s |> (type ref)`. After
completion, normalization retains the child entity `inner::t` and its
normalized leaf value. It erases only how the child's navigation was obtained
(inherited versus explicit) and how the child was formed (internal versus
extended) — never the Pattern entity identity of `inner`.

Ordinary navigated `let inner::(s |> (type ref)) = bool::;` installs `bool::`
as an associated type (Val2 member) named `inner` under `t`'s scope. It does not
register `inner` in `t`'s Pattern structure. Pattern-member registration is a
privilege of `struct` inline construction and the `extend` primitive (directly
or through `inject`). See §12.1 for the full privilege boundary.

## 8. `extend` and `inject`

### 8.1 Privileged built-in

`extend` and `inject` are future bounded privileged operations, parallel to
`struct` in trust boundary. Neither creates an ordinary externally navigable
`MetaInstanceScope M`:

- it accepts normalized pattern syntax or an equivalent internal AST carrier;
- `extend` returns `type`; `inject` returns the input `type ref`;
- it does not re-enter the parser;
- it does not concatenate arbitrary tokens;
- it does not expose unrestricted AST-consuming capability to user functions;
- they perform only bounded pattern-child construction.

The source examples in this section are semantic sketches. They do not change
the frozen parser or introduce traditional `f(args)` call syntax.

### 8.2 `extend` is the primitive pure value transformation

`extend` takes one complete ordinary type snapshot and struct-like child
material, and returns a new complete type snapshot:

```text
extend : type × StructLikeMaterial ⇀ type

old = bind alpha. <Q_old, V_old[alpha]>

Extend_Gamma(old, Delta)
  => new = bind beta. <Q_new, V_new[beta]>
```

`extend` establishes no root and preserves the root already carried by its
input:

```text
Root(new) = Root(old)
```

Root preservation is not snapshot equality and never redirects older copies to
a current mutable Symbol:

```text
new != old                 when the extension contributes semantic material
V_new != V_old             when generated/direct TypeMembers change
CallSpace(old) = V_old
CallSpace(new) = V_new
```

The structural contribution first changes `Q_new` under the canonical Pattern
relation. Any generated classifier whose
`DirectClassifierHome = TypeMemberScope(Q_new)` contributes its ordinary
members to `V_new`. Both components belong to the returned snapshot.

There is no construction-handle rank. The input is an ordinary value of rank
`type`; `type ref` and `type share` are not accepted inputs. A caller may first
clone/read through a view to obtain the ordinary value, but the view contributes
no construction permission.

The function is total in its effects in the following sense:

```text
Extend does not modify old
Extend does not install a namespace delta
Extend does not perform an assignment
```

`old` is an input value and is left exactly as it was, including its `V_old`
callspace. `new` is a distinct resulting value. Discarding `new` produces no
symbol-world side effect, because there was never a side effect to discard.

#### 8.2.1 Failure is total

```text
failure => no partial result, no write, no rollback
```

Because `extend` writes nothing, a failed `extend` has nothing to undo. There is
no half-extended pattern, no compensating action, and no rollback protocol. A
failed call simply produces no value.

#### 8.2.2 `extend` applicability is a construction-authority judgment

The primitive checks the old value in the current evaluation context:

```text
Γ ⊢ old : type
OpenHere_Σ(old)
ParentToChild(old, Δ)
NoPatternConflict(old, Δ)
Canonicalizable(result)
--------------------------------
Γ ⊢ (old, Δ) |> extend : type
  and WellFormedTau(result)     -- independently checked on the result structure
```

`OpenHere_Σ(old)` is derived from `Anchor(old)` and the authority-frame
resolution of §12.1.1 (non-meta: `AuthorityFrame_Σ(Core(old))` exists; meta:
coordinate equality against `CurrentEvaluationCoordinate_meta`), not from a
carrier place. Because `old` is a complete type value `τ` rather than an
ordinary `PatternValue`, the horizontal attributes resolve by Core projection (§12.1.2): `OpenHere_Σ(old) = OpenHere_Σ(Core(old))`. Clone/read
preserves the anchor:

```text
Anchor(Clone(old)) = Anchor(old)
```

Consequently an `OpenHere` value with no writable carrier may be extended and
bound elsewhere, while a closed-window value read through a writable
`type ref` is rejected. There are deliberately no `type ref` or `type share`
overloads for `extend`.

A navigated `let child::target = result;` is **not** a structural installer:
ordinary navigated `let` creates a Val2 associated member and never substitutes
for `extend` or for the write-back performed by `inject`.

#### 8.2.3 `inject` is the read--extend--write wrapper

`inject` accepts exactly a writable type-slot view and struct-like material:

```text
inject : type ref × StructLikeMaterial ⇀ type ref

Inject_Σ(r, Δ):
  require Writable_Γ(Target(r))
  old := Clone(Read(r))
  new := Extend_Σ(old, Δ)       -- independently requires OpenHere_Σ(old)
  Write(Target(r), new)           -- ordinary slot replacement, not construction
  return r
```

The two requirements are deliberately independent:

```text
CanInject_Σ(r, Δ)
  = Writable_Γ(Target(r))
  ∧ CanExtend_Σ(Clone(Read(r)), Δ)
```

`inject` is the composition `clone/read old τ → Extend → ordinary Write back`.
The step that depends on construction authority is `Extend`; the final
`Write` is an ordinary slot replacement (`slot := x'`) that needs only
`Writable_Γ(p)` and the slot's local constraints. Ordinary slot replacement
is **not** a `τ -> τ'` construction transformation: it does not require
formation history, and it does not automatically acquire `extend` semantics
just because the carrier is a type value.

`r : type ref` proves target/lifetime/capability only. It never proves the
current pointee satisfies `OpenHere_Σ`. A closed-window pointee may
therefore be replaced wholesale by ordinary assignment through a writable
ref, while `inject(r, Δ)` fails before the write because its `extend` step is
inadmissible.

Failure before `Write` leaves the target unchanged. `type share` has no
`inject` candidate because it is not writable; by-value `type` has no `inject`
candidate because it supplies no destination place. Both may still participate
in pure value computation where their ordinary value is accepted.

Canonical source makes the type slot explicit:

```lang
let r = (S ref).type;
(r, delta) |> inject;
```

The `.type` field preserves the explicit Symbol borrow observation and performs
no provenance recovery. The result is the same ref `r`, now observing the
successfully written value.

### 8.3 Navigation direction

The distinction between `struct` and `extend` is navigation direction, not
ownership authority; `inject` delegates its middle step to `extend`:

```text
struct:  resolves OUTWARD
  resolve owner by ordinary input navigation + ambient scope
  (always looks up for the top-pattern navigation name)

extend:  resolves INWARD
  takes the input pattern value as the navigation anchor;
  children inherit that pattern's path
  (never looks outward for a top-level scope)

inject: read target -> extend inward -> write the same target
```

This is the whole reason `extend` needs an existing pattern value as input: it
needs a pattern whose navigation path is already resolved, so that the new
children can be linked beneath that path.

Example. `t1::r` is an ordinary pure-pattern path, so it is not a legal
assignment left side (§8.2.2); the carrier slot has to be taken first:

```lang
let r_ref = (t1::r ref).type;
(r_ref, (t first, u second)) |> inject;
```

Pure value construction is separate and performs no write:

```lang
let old = t1::r |> type;
let next = (old, t first) |> extend;
let final = (next, u second) |> extend;
```

The first form performs one read--extend--write transaction through the ref; the
second produces values only. The resulting type Pattern is:

```text
(
    t first::t1::r,
    u second::t1::r
)::t1::r
```

`extend` determines the child set of the resulting pattern value. It does not
change owner identity or reopen a closed-window value. `inject` additionally requires
the target to be writable; formation of `r_ref` alone proves neither premise.

As with `struct`, the lowest-level leaf reduction has the form:

```text
E name
```

At that leaf:

- `name` is the leaf's pattern name;
- `E` is value-bearing material that must be resolved through its external
  symbol binding and then evaluated;
- different leaves do not require the same `E`.

Consequently:

```text
t first
u second
```

means:

```text
first is the pattern name; the leaf value is read through symbol t
second is the pattern name; the leaf value is read through symbol u
```

Pattern-name identity and leaf-value origin are independent. Using `t` for both
leaves would obscure this distinction.

### 8.4 Child-only restriction

`extend` extends the input pattern by **direct children only**; `inject` inherits
the same restriction:

```text
Extend(old, Δ) may add children directly beneath P(old)
Extend(old, Δ) may not reach into a grandchild layer
```

Extending a deeper layer is expressed by composing the operation at that layer —
read the child value, extend it, and write it back where independently
authorized — not by giving either primitive a deep path.

Within that scope, `extend`:

- adds direct children to the resulting pattern;
- preserves the owner identity carried by the input pattern.

It does not:

- replace the owner;
- overwrite an existing core `Core(τ)`;
- delete an existing child;
- implicitly reroot an arbitrary external pattern value;
- mutate the input value or the installed namespace graph;
- extend a value that is not `OpenHere_Σ` in the calling context;
- grant a general macro or arbitrary AST-rewrite capability.

`inject` adds only the ordinary write to an already existing target; it does not
relax any `extend` restriction. Failing Open or write applicability produces no
partial write.

## 9. Pattern-Layer Ordering

This section applies the canonical named-versus-positional and structural-child
rules from
`../patterns-overload/pattern-values-relational-semantics-and-extraction.md` to
Symbol construction. It is not an independent definition of Pattern identity
or relational equivalence.

Let the direct children of one pattern layer be:

```text
p1, p2, ..., pn
```

The ordering rule is decided at the level as a whole. Order-insensitivity
requires both:

```text
the sibling level is wrapped by a Pattern;
every direct child has a top-pattern navigation layer.
```

A naked Product never satisfies the first condition. Therefore:

```text
(a, b)c == (b, a)c
(a, b)  != (b, a)
```

Naming both Product elements does not by itself erase their positions.

The normalizer must therefore preserve two distinct node kinds until this
decision has been made:

```text
ProductNode(children)
PatternLayerNode(name, body)
```

It must not flatten both into one undifferentiated children list and then infer
the node kind from whether every child has a complete navigation. Complete
navigation is necessary for an unordered Pattern body, but it is not
sufficient.

### 9.1 Fully named body of a Pattern

If a sibling layer is the body of a Pattern and every direct child has a
top-pattern navigation layer:

```text
normalize layer
  -> Map<CanonicalFullNavigation, CanonicalPatternValue>
```

For example:

```text
{
    bool::,
    t1::t,
    t2::t
}
```

Every entry contains an already completed Pattern navigation and its normalized
resident value. Neither coordinate is a source `Symbol`, source path, or symbol
reference. The complete navigation is the canonical map key; the resident is
the canonical value at that navigation.

Consequences:

```text
the whole layer is order-insensitive;
layer equality is canonical map equality;
different-name extensions commute;
same-navigation/different-value conflicts are rejected before map formation.
```

For example:

```lang
t1::r
|> extend(t first)
|> extend(u second)
```

and:

```lang
t1::r
|> extend(u second)
|> extend(t first)
```

produce the same pattern value because both direct children have top-pattern
names.

Once normalized, the map does not classify elements as “internal patterns” or
“external patterns.” Parent-scope inheritance, explicit `::`, ordinary symbol
binding, and `extend` explain how a `PatternValue` was resolved or produced
before normalization. After its navigation name is fully qualified, source
category and construction route do not participate in `PatternValue` identity,
map equality, or extraction semantics.

An implementation may retain source symbol, inherited/explicit navigation,
binding origin, or injection origin as provenance for diagnostics and replay.
That provenance must not affect `PatternValue` equality.

Insertion of an equal `(complete navigation, normalized resident)` entry is
idempotent. Distinct source symbols may remain distinct extraction entry paths
while contributing only one canonical map entry:

```lang
let a::t = bool;
let b::t = bool;
```

```text
value(symbol(a::t)) = bool::
value(symbol(b::t)) = bool::

{
  FullNav(bool::) -> Norm(bool)
}
```

Both `a::t` and `b::t` may be used as source navigation paths. After symbol
resolution and value read, both look up the single `bool::` entry. The layer
is neither a multiset nor a relation keyed by the carrier Symbol's source name.
It is keyed by canonical complete Pattern navigation.

Symbol paths and `PatternValue` navigation names may coincide or differ. For
example, the same spelling may describe:

```text
symbol navigation path:                 t1::t
PatternValue navigation carried there:  t1::t
```

The `t1::t` key in a normalized map is still canonical Pattern navigation; its
spelling does not turn it into a Symbol reference. Conversely:

```lang
let t3::t = bool;
```

may establish:

```text
symbol navigation path:                 t3::t
PatternValue navigation carried there:  bool::
```

The symbol path and value path are then visibly different. Both cases use the
same symbol-resolution/value-read semantics.

### 9.2 Naked Product or Pattern body containing a bare value

The layer is order-sensitive if either:

```text
it is a naked Product; or
it is a Pattern body with at least one bare direct child.
```

In either case:

```text
the entire current layer is order-sensitive;
positions participate in identity;
the layer cannot be replaced by a name map.
```

The rule is not “only the bare child is ordered.” The presence of one bare
value makes the complete sibling layer positional.

### 9.3 Representation guidance

An implementation may distinguish:

```text
Fully named body of a Pattern:
  representation =
    Map<CanonicalFullNavigation, CanonicalPatternValue>
  membership/equality use the complete navigation and normalized resident
  order-insensitive

OrderedPatternLayer:
  position-preserving, order-sensitive
  used for every naked Product
  also used for a Pattern body containing any bare direct child
```

A canonical serializer may sort a fully named map by canonical complete
navigation encoding. Sorting is only a stable representation of map semantics;
it must not be presented as preserved source-order meaning. An ordered layer
must preserve positions.

### 9.4 Navigation, ordering, and optional peeling are orthogonal

These mechanisms answer different questions:

```text
navigation completeness:
  determined by OwnNavigation and Pattern-parent anchor traversal

ordering:
  determined by ProductNode versus PatternLayerNode

optional top peel:
  erases one top Pattern identity while retaining an anonymous
  PatternLayerNode boundary and that layer's ordering
```

The future default `?` operation must therefore use:

```text
PatternLayer(c, B, O)
  ?-> PatternLayer(NameAbsent, B, O)
```

not:

```text
PatternLayer(c, B, O)
  ?-> Product(B)
```

If no top Pattern is peelable, `OptionalPeel(x) = x`; this is a fixed point,
not failure and not `none`. The retained layer boundary must guarantee:

```text
PeelView(Norm(x)) = Norm(PeelView(x))
```

This is a recorded future extraction invariant. It does not claim that the
current evaluator implements `?`.

## 10. Child Uniqueness and Replay

“Extend once” applies to a complete child navigation path, not to the owner as a
whole.

For named direct children, the conceptual uniqueness key is:

```text
(owner PatternScopeId, child top-pattern identity)
```

This is a construction-time path-conflict key, not the representation of the
normalized layer. After successful validation/evaluation, the child contributes
its complete-navigation/normalized-value entry to the canonical unordered map.

Therefore:

```lang
|> extend(t first)
|> extend(u second)
```

is valid, while:

```lang
|> extend(t first)
|> extend(u first)
```

is a conflict because both attempt to create:

```text
first::owner
```

Cache replay remains idempotent only for the same origin and material:

```text
same owner + same child + same construction origin/material
  -> reuse / idempotent replay

same owner + same child + different material
  -> hard conflict
```

Replay origin controls whether a construction action may be reused; it does not
become part of the resulting `PatternValue` identity.

An ordered layer still preserves positional identity; a symbol-keyed or
name-keyed map must not replace either the ordered layer or the normalized
map keyed by canonical complete Pattern navigation.

## 11. Extraction and Explicit Navigation

This section applies the canonical navigation-formation and child-identity
rules from
`../patterns-overload/pattern-values-relational-semantics-and-extraction.md` to
symbol-first lookup. Formation provenance may be retained for diagnostics but
does not define a competing Pattern normal form.

### 11.1 Navigation always reaches a Symbol before a value

Both inherited and explicit pattern navigation use the same final two steps:

```text
symbol resolution
  -> value read
```

They differ only in how the symbol path is formed.

Each Pattern layer has one own-navigation state:

```text
OwnNavigation(layer) =
    Explicit(path)
  | ImplicitGlobal
  | Absent
```

`Absent` is valid only for a non-root layer and means that completion continues
through the semantic Pattern-parent link. A root Pattern whose navigation is
omitted has `ImplicitGlobal`, never `Absent`. Therefore the anchor is total:

```text
Anchor(x) =
  nearest ancestor a of x
  where OwnNavigation(a) != Absent
```

A bare name is completed by walking its already existing Pattern-parent chain
from nearest to farthest:

```text
name
  -> append the nearest parent's local navigation
  -> continue through every parent whose OwnNavigation is Absent
  -> stop at the nearest Explicit(path) or ImplicitGlobal anchor
  -> resolve that completed Symbol path
  -> read the PatternValue carried by that Symbol
```

Equivalently:

```text
FullNav(x) =
  LocalSegments(x -> Anchor(x))
  :: Navigation(Anchor(x))
```

This walk does not classify either the subject or any parent as internal or
external. Those source/construction categories are irrelevant to navigation
completion. The only question at each parent is whether that parent explicitly
specified its own navigation level.

The top Pattern is always the final anchor. If its own navigation was omitted,
the omission means an exact global lookup—implicit `::`. It does **not** mean
“treat the top name as an ordinary bare name and search
`near -> outer -> core`.” That ordinary scope chain belongs to value/call
target resolution, not extraction navigation completion.

Navigation completion never infers a missing parent by reversing or guessing
from the resident's spelling. It follows only semantic Pattern-parent links
that already exist.

An explicitly navigated extraction subject does not inherit the Pattern-parent
chain:

```text
::external
  -> begin at the explicitly selected external Symbol layer
  -> resolve that Symbol path
  -> read the PatternValue carried by that Symbol
```

In the current inner-to-outer surface notation, an explicitly terminated
external component is written as `external::` where a grouping boundary is
needed. The conceptual `::external` description above emphasizes that the
external layer is selected rather than parent-completed; it does not reverse the
frozen source navigation order.

Default inheritance is therefore not “indirect value access” while explicit
navigation is “direct value access.” Neither form directly touches a pattern
value. Both first produce one exact symbol path, resolve it, and then read its
value.

The pattern expectation permits only a `PatternValue`/pattern interface exposed
by that symbol. It does not fall back to invoking arbitrary ordinary values or
callables from the heterogeneous typed `V` members.

### 11.2 Binding a fully qualified PatternValue through another symbol

Consider a globally defined symbol construction:

```lang
let bool = ((if | else) bool) |> struct;
```

Two semantic objects may share the diagnostic spelling `bool`:

```text
symbol(bool)
pattern head bool
```

They are not one identity. The first is the source-resolved symbol. The second
is the owner/head projection inside the `PatternValue` carried by that symbol.

Now:

```lang
let t1::t = bool;
```

uses the general value-binding rule:

```text
resolve symbol(bool)
  -> read its PatternValue, whose fully qualified navigation is bool::
resolve destination symbol/place t1::t
  -> bind that same PatternValue to t1::t
```

For normalized-pattern explanation, the relation may be written:

```text
(bool::)t
```

This does not reroot the value, rewrite its navigation, change its top name to
`t1`, identify `symbol(t1::t)` with the `bool` pattern head, or create an
internal `bool` pattern under `t1::t`.

The accurate normalized statement is:

```text
symbol t1::t is bound to a PatternValue whose fully qualified navigation is bool::
```

The source binding route may be retained as provenance, but “external” versus
“internal” is not a category in normalized `PatternValue` identity.

### 11.3 Inherited and explicit extraction are equivalent here

With the binding above, the extraction shorthand:

```lang
let P t1 t = t;
```

and the explicit form:

```lang
let <P> ((P)bool::)t = t;
```

denote the same extraction.

For the shorthand, resolving bare `t1` starts at its nearest Pattern parent
`t`. Here `t` is also the nearest navigation anchor, producing the symbol
path:

```text
t1::t
```

The evaluator then resolves `symbol(t1::t)` and reads its bound
`PatternValue`. That value reveals its fully qualified pattern navigation:

```text
bool::
```

For the explicit form, `bool::` explicitly terminates the external symbol path
(the conceptual `::bool` choice) and blocks completion under the current parent
`t`. The evaluator resolves `symbol(bool)` and then reads the `PatternValue`
carried by that symbol.

Both paths therefore reach:

```text
P = if::bool | else::bool
```

The distinction is solely:

```text
inherited form:
  follow Pattern parents through Absent layers to the nearest
  Explicit(path) or ImplicitGlobal anchor,
  then resolve exact Symbol path -> read PatternValue

explicit form:
  select an external symbol path, then resolve Symbol -> read PatternValue
```

It is never a distinction between an indirect pattern value and a directly
named pattern value. Source navigation names bindings first. A pattern's
canonical/diagnostic navigation may match a source Symbol spelling without
becoming the same identity.

### 11.4 Extraction looks up PatternValue in the canonical map

For a fully named sibling layer that is the body of a Pattern, normalization
produces:

```text
M: Map<CanonicalFullNavigation, CanonicalPatternValue>
```

Extraction is therefore value lookup, not symbol lookup. The normative process
is:

```text
1. Complete the source navigation path by walking Pattern parents through
   `OwnNavigation = Absent` to the nearest `Explicit(path)` or
   `ImplicitGlobal` anchor. Honor an explicit subject navigation without
   parent completion.
2. Resolve the completed path to a Symbol.
3. Read the PatternValue bound to that Symbol.
4. Split that normalized PatternValue into its complete navigation and
   normalized resident, then look up the equal entry in M.
5. If present, continue extraction through the matched PatternValue.
```

Formally:

```text
extract(path, M)
  = lookup(canonical_entry(value(resolve_symbol(path))), M)
```

not:

```text
lookup(resolve_symbol(path), M)
```

because `M` contains evaluated canonical navigation/value entries, not
name-graph nodes or Symbol references.

For example:

```lang
let bool = ((if | else) bool) |> struct;
let t3::t = bool;
```

and:

```text
M = {
  FullNav(bool::) -> Norm(bool),
  FullNav(t1::t)  -> Norm(t1),
  FullNav(t2::t)  -> Norm(t2)
}
```

the extraction path:

```text
t3 t
```

first inherits parent navigation and forms symbol path:

```text
t3::t
```

Then:

```text
resolve_symbol(t3::t) = symbol(t3::t)
value(symbol(t3::t)) = bool::
canonical_entry(bool::) ∈ M
```

Thus `t3 t` matches `bool::`, not `t3::t`.

By contrast, if:

```text
value(symbol(t1::t)) = t1::t
```

then the source symbol path and resulting `PatternValue` navigation happen to
share a spelling. The extraction still performs symbol resolution and value
read before set lookup; the shared spelling does not permit either step to be
omitted.

## 12. Facet Conflicts and Installation

### 12.1 Contribution expectation selects the facet

A navigated child binder does not determine its contribution facet from the
runtime shape of the right side. The enclosing semantic position supplies a
construction expectation, optionally made explicit by a rank/facet annotation:

```text
ContributionExpectation =
    PatternChild           (PRIVILEGED: struct inline / extend only)
  | NamespaceValueMember   (ordinary navigated let)
```

> **Privilege boundary:**
>
> `PatternChild` is a **privileged** expectation available only to:
> - `struct` inline construction (elements in the struct body)
> - `extend` primitive (directly or through `inject`)
>
> Ordinary navigated member creation is interpreted under
> `NamespaceValueMember`, regardless of whether `expr` is type-valued
> (`TypeValue(expr)=tau`, with ordinary observation `Core(tau)=Q`) or an
> ordinary value-bearing Object. The
> expectation is never guessed from the RHS shape.
>
> ```text
> let f::(t |> (type ref))   -> NamespaceValueMember (always)
> struct inline / extend  -> PatternChild (privileged)
> ```

Under `PatternChild`, the source path is resolved to a Symbol and projected to
its type/pattern value. The resulting `PatternValue` is installed as a child of
the owner's type construction and participates in normalization and extraction:

```text
resolve source Symbol
  -> project/read PatternValue
  -> contribute to the owner Object's Pattern/type-role construction
```

This expectation is exercised by `struct` inline construction elements and
`extend`. It requires the input PatternValue to satisfy `OpenHere_Σ`; `inject` reaches
the same rule only by reading its ref and invoking `extend`.

Under the current `NamespaceValueMember` implementation expectation, the source
is projected through its ordinary `V` members and a namespace value Symbol is
constructed. This changes only the namespace graph/value members; it does not
enter or change the owner's
`PatternValue`:

```text
resolve source Symbol
  -> project/read value (including a complete type closure when type-demanded)
  -> install as associated Val2 member
  -> does NOT modify target Pattern canonical structure
```

This is the expectation of:
- Explicit-place navigated `let f::(t |> (type ref)) = expr`
- An ordinary let-shaped declaration consumed inside `struct` construction:

```lang
let name = expr
```

It contributes one associated member to the current Pattern owner's
`Val2` value-member structure:

```text
target pure-P contribution = none
installed contribution      = the complete expr value
```

The initializer is not restricted to type/Pattern material or to `Pv=absent`.
It may contribute any ordinary heterogeneous value entry, including a callable
function Object or a type-valued entry. A type-valued entry preserves its whole
`tau=<Q,V_τ>` snapshot in the slot while ordinary Pattern/namespace observation
sees `Q`; an ordinary Object preserves its own recursive coordinates. Neither
form is spliced into the target owner's pure Pattern. The construction does not
mutate the namespace graph during `struct` evaluation.

The four-way classification of installed members:

```text
Associated member     : Val2 中存在
Associated type       : Val2 slot 中存在完整 tau，普通观察为 Core(tau)=Q
Structural child      : Val2 成员已登记到父 P 正规结构
Bare structural value : 登记到正规结构但局部模式为 ε

ordinary let -> produces the first two only
struct / extend -> can produce the third and fourth, with privilege
```

```text
Privileged structural registration (struct inline / extend ONLY):
  Core(tau) = Q or other admitted pure Pattern material
  -> registers that material into target P canonical structure
  -> the member becomes a structural child with extraction/construction capability

Ordinary Val2 installation (let f::(type_ref) = expr, always):
  TypeValue(expr)=tau -> installs complete tau in the associated slot (Val2 only)
  ordinary Object     -> installs that Object as associated value (Val2 only)
  Neither modifies the target Pattern canonical structure.
```

The associated-type judgment is scoped to the TARGET cluster; `Val2` is not
a name → raw value list map, it stays a recursive Symbol world:

```text
Val2(T_t)[f] = C_f
C_f          = ⟨P_x, w_1, ..., w_m⟩

AssociatedType ⊄ target ClusterMember
AssociatedType  = pure-P member of its associated Val2 Symbol
AssociatedType ⊄ PatternStructuralChild
```

so, writing `C_t = ⟨T_t, v_1, ..., v_n⟩` for the target cluster:

```text
x ∉ Members(C_t)
x  = PureP(C_f),  C_f ∈ Val2(T_t)
```

Resolving `let f::(t |> (type ref)) = x` borrows the type-level carrier place of the
target type-valued binding as `type ref`, derives the
stable prospective `ProjectionSlot(ObjectPlace(T_t), f)`, interns the associated
Symbol `C_f` there, and installs `x`
as `C_f.pure_p` with the binding-level member view in `C_f.member_views`.
Same-named associated vals join that very same `C_f` as its sibling vals
`w_i`, so `C_f` obeys the ordinary cluster Policy disjunction:

```text
P(C_f) = P(P_x) || P(w_1) || ... || P(w_m)
```

`P(T_t)` and `P(C_t)` never absorb `P(C_f)`. The binding-level Policy of
the associated type is the member view of `C_f` — the RHS complete pure-P
view already restricted by the binding's written P1, exactly as on the
ordinary value path; a type does not get a second P1 discipline for lacking
a Val1. The target type-valued slot preserves its own whole
`tau=<Q,V_τ>` snapshot. Core lookup projections are derived from that snapshot
and never recover or define the complete type value.

A pure P is a real object, so the place is per carrier, never per
PatternValue:

```text
let T: type = uint8;
let U: type = T;

Core(TypeValue(T)) = Core(TypeValue(U)) = Q_uint8
Place(T)  != Place(U)  != Place(uint8)
```

`let f::(T |> (type ref))` therefore creates beneath `T`'s own pure-type place, and
`U::f` / `uint8::f` do not see it. Bare `let f::T` performs no implicit
Symbol-to-type projection and is not this operation. The ordinary associated
installation is ordinary **slot replacement** (type-values §2.2, §7.1): it
replaces the carrier's stored snapshot with a fresh `tau' = <Q', V_τ>` and
updates only `T`'s carrier-local `Val2` observation; it neither changes the
copied snapshot in `U`, changes `V_τ`, nor registers a structural child. The
fresh snapshot `tau' = <Q', V_τ>` is checked independently: `WellFormedTau(tau')`
is a structural judgment (`Q'` is a well-formed pure Object obtained by the
permitted slot update, and `V_τ` is unchanged), so `CompleteType(tau')` is
derivable without any formation-history reasoning. Complete type observation includes the resulting core observation when
identity is demanded. Generated
construction-time TypeMembers are already closed into `V_τ`; they are never
recovered through fallback to a mutable defining Symbol or canonical root.
There is no second, place-forwarding declaration form: every carrier allocates
its own place (§2.6), so a per-carrier installation is local to that carrier.
Where one place must be reached through another name, the value held is a borrow
view. Member creation still requires a prospective ProjectionSlot plus `let`;
later writes require an existing place and `Writable(place)`. Neither obtains
structural `Open` from the view, as specified in
`type-values-places-and-borrow-views.md`.

Exposure of `t::f` composes `Expose(T_t, φ) ∧ Expose(C_f, φ)` at lookup
time, and a deeper path `g::f::T` composes the whole chain
`Expose(T_t, φ) ∧ Expose(C_f, φ) ∧ …` — installation never merges, disjoins,
or writes `P(x)` back into `P(T_t)`. The conjunction is a phase predicate
applied per layer, not a stage-set intersection: a `meta` host legitimately
carries `compile` members, and it is each host's own binding-level view — not
the Pattern — that decides that layer's factor. Explicit navigation therefore
carries the resolved host chain (each
layer's carrier Symbol, its object place, its member view) along with the
selected `C_f`, so the invocation pipeline applies every host factor before
member selection and refuses the target when any layer is hidden; a bare name
reaches its target with an empty host chain and composes only the member
factor. Two carriers of one TypeValue with
different written P1 expose the same `C_f` differently, which a
`PatternValueId` alone cannot express. Everything else is invariant: the
target cluster's member ledger, the selected type-capable `Q`'s own Policy, the derived
cluster Policy, the Pattern canonical norm, and the Val2 of the cluster's
same-named ordinary value members. Navigation and invocation always take
the Symbol route:

```text
target Symbol -> Q where TypeRole(Q) -> Q.Val2 Symbol -> member projection
```

A raw `PatternValueId → Vec<SemanticValueId>` read is transport material
for compiler-installed entries that never allocated a scope-local Symbol
(for example the `()` call entries of a materialized type); it is never the
authoritative route for a source-visible associated name.

Implementation state of this section: the per-carrier `ObjectPlace`, the
per-object source-visible Val2 name ledger, the recursive `C_f` with its
own member views, and the layered exposure conjunction on explicitly
navigated targets are implemented in `crates/lang_build`. Still open debt:
the associated-extension entry point is reached only through a still-open
construction, so it resolves the target object from the constructed
Pattern; source-level `let f::(U |> (type ref))` against an already installed
pure-type rebinding carrier, navigation through that explicit `type ref` view,
and writability checking of the selected place remain future implementation
work. Bare `let f::U` is not shorthand for obtaining the type-level place.

The two operations may target the same still-open construction, but one source
value is not simultaneously interpreted under both judgments.

#### 12.1.1 Open authority is stack-relative

Every constructed PatternValue carries a structural anchor and an immutable
birth regime. Whether it may be structurally modified in the current evaluation
context is a separate, dynamic judgment that combines the value's static anchor
with the evaluation stack and the current open-window state:

```text
Anchor(v) = ⟨PatternRoot(v), Navigation(v)⟩

GenerationRegime(v) ∈ { MetaGenerated, NonMetaGenerated }
                     -- immutable birth classification (value attribute)

WindowLive_Σ(v)       -- construction window still open at current program point
                       -- evaluation/window state, not a value attribute
Visible_Σ(v)          -- current frame can obtain v

OpenHere_Σ(v)
  iff WindowLive_Σ(v)
  ∧ AuthorityMatches(v, Σ)

AuthorityMatches(v, Σ)
  iff AuthorityFrame_Σ(v) exists

Anchor(v) ∉ Norm(v)
CarrierPlace(v) ∉ Anchor(v)
GenerationRegime(v) ∉ Norm(v)
```

`GenerationRegime(v)` is fixed at creation. `WindowLive_Σ(v)` is a property of
the current evaluation state: the construction window has not been permanently
closed at the current program point. `OpenHere_Σ` adds the contextual question:
does the current evaluation stack still contain the frame that owns this value's
anchor, and is the window still live there? `Visible_Σ` adds a third state: the
value exists and the window may still be live, but the current frame cannot
obtain it (for example, it is shadowed by a deeper meta invocation — see below).

Clone, value copy, and compile transport preserve the anchor and regime; they
do not preserve or manufacture source-place identity, and they do not create a
fresh window state:

```text
Anchor(Clone(v))    = Anchor(v)
Anchor(let-copy(v)) = Anchor(v)
```

Construction authority is resolved **per value** against the evaluation
stack. The PatternValue supplies the static anchor; the stack supplies each
level's current evaluation position; authority then belongs to the frame that
still owns that anchor — not unconditionally to the stack-top callable:

```text
Frame = ⟨ CallableRoot, MetaPartnerRoot?, ActiveInlineClosurePath ⟩

EvaluationCoordinate(f)
  = ⟨RootCoordinate(Callable(f)), ActiveInlineClosurePath(f)⟩

RootCoordinate(F)
  = MetaPartnerRoot(F, GenericArgs)   if Generic(F)
    CallableRoot(F)                   otherwise

AuthorityFrame_Σ(v)
  -- the nearest still-active frame owning Anchor(v),
     resolved per regime (below)

CurrentAuthority_Γ     -- typing-context form of the same judgment
```

For a **meta** context, walk the compile-time stack in reverse, skipping
`compile` and transparent construction-intrinsic frames. Let `M` be the first
ordinary meta invocation frame found; `NearestMetaRoot(Σ)` is its MetaInstance
root. In-place closure navigation is transparent for authority purposes
(`VisibleInlinePath_meta(path) = ε`), so the meta authority frame degenerates
to the nearest meta root:

```text
CurrentEvaluationCoordinate_meta(Σ)
  = ⟨NearestMetaRoot(Σ), ε⟩

AuthorityFrame_Σ(v)                  -- meta context
  = the nearest meta invocation frame M such that
      EvaluationCoordinate(M) = Anchor(v)
  -- equivalent to Anchor(v) = CurrentEvaluationCoordinate_meta(Σ)

OpenHere_Σ(v)
  iff WindowLive_Σ(v)
  ∧ AuthorityMatches_meta(v, Σ)
  where AuthorityMatches_meta(v, Σ)
          iff Anchor(v) = CurrentEvaluationCoordinate_meta(Σ)
```

The original spelling `RootOf(Anchor(v)) = NearestMetaRoot(Σ)` is the
simplified form of this unified rule under the meta transparent-navigation
quotient.

Meta invocation is naturally masking. If `M₀ └─ M₁` and the current context
is `M₁`, a value anchored on `M₀` satisfies:

```text
WindowLive_Σ(v) = true   -- window still open
Visible_Σ(v)    = false  -- not obtainable in M₁'s frame
OpenHere_Σ(v)   = false  -- AuthorityFrame_Σ(v) undefined: M₁ is the
                             nearest meta frame and does not own the
                             anchor; the resolution does not look past a
                             masking meta boundary
```

The value may persist in `M₀`'s suspended frame. It cannot be accessed or
passed as an argument in `M₁`. When the stack returns to `M₀`, the value
becomes visible and `OpenHere` again — this is **not** a reopen. True close is
the permanent, irreversible transition:

```text
WindowLive_Σ(v) := false   -- the only real close; never retracted
```

Nothing reopens closed material. `extend`/`inject` do not reopen it (§8.2), a
borrow view does not reopen it, and re-navigating to the same object from a
new context does not reopen it. Cloning/copying/transporting a closed-window
value carries its closed state with it: the clone is not a reopening.

For a **non-meta** context, authority is **not** a fixed function of the
stack-top callable. `AuthorityFrame_Σ(v)` is the nearest still-active frame
whose evaluation coordinate owns `Anchor(v)`, searched outward from the
current frame:

```text
AuthorityFrame_Σ(v)                        -- non-meta context
  = the nearest still-active frame f such that
      EvaluationCoordinate(f) = Anchor(v),
    searched outward from the current frame,
    skipping compile and transparent construction-intrinsic frames,
    and stopping at any meta invocation frame:
    a meta boundary between the current frame and f masks v and
    leaves AuthorityFrame_Σ(v) undefined

AuthorityMatches_nonmeta(v, Σ)
  iff AuthorityFrame_Σ(v) exists
```

The owning frame's coordinate contributes the `CallableRoot`, the
`MetaPartnerRoot` when the callable is generic (providing the stable symbolic
anchoring for the generic arguments), and that frame's own
`ActiveInlineClosurePath` — its navigation level within the in-place closure.
The path entering the comparison is always the owning frame's path, never
unconditionally the stack-top frame's path. Meta and non-meta resolutions are
different authority computations over the same `OpenHere_Σ` judgment, not
different notions of place capability.

Passing an open value into a deeper ordinary call frame therefore does not
destroy authority: the caller's frame remains still-active on the stack and
continues to own the anchor:

```text
F calls G (ordinary), v anchored at F:
  AuthorityFrame_Σ(v) = F's still-active frame while G executes
  OpenHere_Σ(v) holds inside G while the window is live
  -- G operates on v under the §12.1.2 disposition rules: at
     coordinates below the anchor the terminal actions are Reject,
     not Terminate
```

`AuthorityMatches` is therefore not an open ontology decision: it is the
coordinate equality between the value's static anchor and the owning frame's
evaluation coordinate.

The equality is opaque navigation-coordinate equality, **not** arbitrary
prefix matching. A prefix match would let an outer PatternValue automatically
acquire authority over every deeper inline closure, destroying the property
that non-meta in-place closure levels are opaque to authority.

The bare name `AuthorityMatches(v, Σ)` is the regime-dispatched form of the
same judgment: `AuthorityMatches_nonmeta` when the current context is non-meta,
`AuthorityMatches_meta` when the current context is meta.

The meta case is the same coordinate model under the transparent-navigation
quotient (above): `CurrentEvaluationCoordinate_meta(Σ) = ⟨NearestMetaRoot(Σ), ε⟩`.

The canonical principle is:

```text
PatternValue     records static PatternRoot + Navigation
evaluation stack records current dynamic evaluation position
PatternValue does not record dynamic call history
```

The `MetaPartnerRoot` answers where generic symbolic anchoring lives for a
generic callable `F`, and is required exactly when `F` is generic:

```text
Generic(F) => MetaPartnerRoot(F, GenericArgs)
```

It is **not** conditioned on whether `F` also has a `CompilePartner(F)`. The
compile partner `CompilePartner(F) = C(F)` (function-object-call-model §8)
answers how the compile-time realization of `F` is produced; the meta partner
`MetaPartner(F) = M(F)` (meta-object-invocation §4) answers at which level the
callable's generic symbolic identity is anchored. The two partners are
orthogonal: a runtime generic `F` has both `C(F)` and `M(F)`; a compile generic
`F` has no distinct compile partner but still has `M(F)`; a meta `F` has
neither. `CurrentAuthority(Σ)` therefore uses `MetaPartnerRoot(F, GenericArgs)`
for generic symbolic anchoring, independent of any `CompilePartner(F)`
consideration.

The required independence is explicit:

```text
Writable_Γ(q)            does not imply OpenHere_Σ(Read(q))
OpenHere_Σ(v)            does not imply Writable_Γ(Carrier(v))
Γ ⊢ r : type ref         does not imply OpenHere_Σ(Read(r))
WindowLive_Σ(v)          does not imply Visible_Σ(v)
Visible_Σ(v)             does not imply OpenHere_Σ(v)
```

The state transition of the open window is one-way:

```text
WindowLive_Σ(v) := false   -- irreversible
```

Nothing reopens closed material. `extend`/`inject` do not reopen it (§8.2), a
borrow view does not reopen it, and re-navigating to the same object from a
new context does not reopen it.

#### 12.1.2 GenerationRegime and open dispositions

Every `PatternValue` carries a small immutable horizontal attribute:

```text
GenerationRegime(v) ∈ { MetaGenerated, NonMetaGenerated }
```

`GenerationRegime(v)` is **not** part of the Object structure
`Object = ⟨Val1?, P, Val2⟩`, is not part of `Norm(v)`, and does not
participate in canonical Pattern identity or τ normalization. It is an
implementation attribute used only to decide how the open window may be
closed.

Although `GenerationRegime` and `Anchor` are defined on
ordinary `PatternValue`s, and `WindowLive_Σ` is defined on the evaluation
state, `extend` operates on the complete type value
`τ = <Q, V_τ>`, which is not itself an ordinary `PatternValue`. The bridge is
by Core projection, consistent with the minimal-change observation rule (§2.2:
ordinary type-rank equality observes `Core(τ) = Q`):

```text
GenerationRegime(τ) := GenerationRegime(Core(τ))
WindowLive_Σ(τ)   := WindowLive_Σ(Core(τ))
Anchor(τ)           := Anchor(Core(τ))

OpenHere_Σ(τ)
  = WindowLive_Σ(τ) ∧ AuthorityMatches(τ, Σ)
  = OpenHere_Σ(Core(τ))
  -- AuthorityMatches as defined in §12.1.1: per-value authority-frame
  -- resolution (non-meta: AuthorityFrame_Σ exists;
  -- meta: coordinate equality against CurrentEvaluationCoordinate_meta)
```

`GenerationRegime(τ)` does not participate in `WellFormedTau(τ)` or in Pattern
identity; it is consulted only by the contextual capability rules above. No new
`ConstructionSubject` ontology is introduced: the horizontal attributes of a
complete type value are those of its core `PatternValue`.

- **MetaGenerated.** A value produced inside a meta body has no birthright
  global lifetime. It can be used freely within the same meta computation, and
  it may be promoted into a stable result only when the MetaInstance seals and
  owns/copies the material it owns. The original local value is not magically
  prolonged: persistence happens by promoting the MetaInstance's stable value,
  never by extending the local value's lifetime.

- **NonMetaGenerated.** A value produced in an ordinary (non-meta) construction
  context is born globally survivable with a live open window:
  `GlobalSurvivable(v) ∧ WindowLive_Σ(v)` hold from creation. Its open window
  is a linear evaluation flow, not a flat event list. The disposition of an
  action on `v` is one of three outcomes:

```text
OpenDisposition_κ(p, action, Σ)
  ∈ { Continue, Terminate, Reject }
```

The owning in-place closure's evaluation segment is only the natural upper
bound of the open window; the window may end earlier. In particular:

```text
EffectiveOpenSegment(p)
  ⊆ OwningInlineClosureEvaluationSegment(p)
```

At the value's own outermost open coordinate
(`CurrentCoordinate = OpenRootCoordinate(p)`), the legal terminal actions end
the open window (`Terminate`); they are not forbidden, but they close the
window:

```text
CurrentCoordinate = OpenRootCoordinate(p)      -- outermost open coordinate

UseForVal1(p)        ->  Terminate   -- legal action; ends the open window
UseAsMetaArgument(p) ->  Terminate   -- legal action; ends the open window
ControlFlowSplit(p) / ControlFlowMerge(p)
  at generation level                  ->  Terminate
  -- the window requires a single, non-forking, non-merging linear
     evaluation stream; a static join/loop-carried state or a
     residual-runtime fork at the generation level violates that
     requirement exactly at that point
```

Inside an opaque non-meta inline closure (the evaluation has already moved
below the value's own open coordinate), `UseForVal1` and `UseAsMetaArgument`
are **forbidden** (`Reject`) at any depth, because performing the construction
effect would already have crossed the value's legal linear open flow.
`ControlFlowSplit` / `ControlFlowMerge` are **generation-coordinate** events:
they terminate the window only at the value's own generation level, and at a
deeper ordinary coordinate they are **irrelevant to the outer window** —
neither `Reject` nor `Terminate`:

```text
CurrentCoordinate ≻opaque OpenRootCoordinate(p)
  -- evaluation is inside an opaque non-meta inline closure below the
     PatternValue's own open coordinate

UseForVal1(p)        ->  Reject
UseAsMetaArgument(p) ->  Reject
ControlFlowSplit(p) / ControlFlowMerge(p)
  at the generation coordinate    ->  Terminate
  at a deeper ordinary coordinate ->  Continue (irrelevant to outer window)
```

The judgment reversal therefore applies only to `UseForVal1` and
`UseAsMetaArgument`: at the outermost coordinate the action is a legal
terminal action; in a nested opaque non-meta level the same action is a
forbidden one. It cannot be explained as "first allow `UseForVal1`, then
close": by the time the construction effect happens, the value's legal linear
open flow has already been crossed. Control-flow split/merge are not "close
after the fact" events: they are scoped to the value's own generation
coordinate, so a split or merge inside a deeper ordinary frame does not reach
back and close an open value generated at an outer level.

`UseForVal1` and `UseAsMetaArgument` reject/terminate independent of
call-frame depth at the relevant coordinate: a meta boundary cannot be
escaped by performing the meta call inside a deeper ordinary frame, and
installing the value as `Val1` is likewise unconditional. `ControlFlowMerge`
and `ControlFlowSplit` apply only at the value's own generation level; a merge
or split inside a deeper ordinary call frame does not reach back and close an
open value generated at an outer level. Passing the value into a deeper
ordinary call frame does **not** itself end or forbid the window. Likewise, a
value's visibility (`Visible_Σ`) may be lost because of stack masking without
its open window being touched.

In an ordinary, non-meta construction context the concrete dispositions are:

```text
UseForVal1(x)                                    -> Terminate at OpenRootCoordinate(x)
                                                     Reject inside an opaque non-meta
                                                     inline closure below it
x used as a meta argument                        -> Terminate / Reject (same rule)
x entering a global normalized structure         -> Terminate (at OpenRootCoordinate)
x in Dependencies(c), for NonMetaStaticControl(c) -> Terminate
                                                     (at generation level)
x in LiveAcross(c), for ResidualRuntimeFork(c)    -> Terminate
                                                     (at generation level)
leaving the construction interval of the
  in-place closure that owns x                   -> Terminate
                                                     (owner's interval)
```

Observation is not a terminating action: reading `P` or `Val2`, extending a
child pattern, and contributing an ordinary Val2 member of another type all
leave the material open (`Continue`).

For static control, dependency and liveness are different facts:

```text
Dependencies(c) != LiveAcross(c)

NonMetaStaticControl(c)
  => OpenDisposition_κ(d, UseInControlFlow, Σ) = Terminate
     for each d ∈ Dependencies(c) at the generation level
```

`Dependencies(c)` contains the open Pattern values actually read by the
predicate or structural selection, branch/iteration versions whose identity
must be unified at a join, and loop-carried construction state that feeds a
later static decision. A value that is merely live across an unrelated static
branch, join, or loop is not terminated. In contrast, a residual-runtime fork
loses the single known static construction path, so open values carried across
that fork are terminated even when they did not determine its predicate.
Leaving the ordinary owner interval remains an independent terminating
disposition.

#### 12.1.3 Meta construction is transparent but meta-local lifetime is not global

The open dispositions of §12.1.2 are scoped to `NonMetaGenerated` values.
Inside a meta body, material is `MetaGenerated`, and the same actions do
**not** terminate its open window, because the construction anchor is the
meta instance itself (§4.3.1). Meta navigation is transparent for authority:
`ActiveInlineClosurePath_meta` is quotient/erased (`VisibleInlinePath_meta(path)
= ε`), so meta evaluation never produces the opaque nested state that triggers
`Reject` for non-meta inline closures. The meta space is governed by
`NearestMetaRoot`, `MetaArgumentAdmissible`, `GlobalSurvivable`,
`NoOpenEvaluationReentry`, and seal/promotion rules instead:

```text
inside M (MetaGenerated material):
  UseForVal1(x)                     Continue -- does not end the window
  using x as a meta argument        Continue -- presupposes meta argument
                                      admissibility (§4.3.1–§4.3.3):
                                      MetaArgumentAdmissible(a) =>
                                        GlobalSurvivable(a), and a
                                        non-GlobalSurvivable MetaGenerated
                                        local cannot enter another meta
                                        invocation at all
  entering global-normalization     Continue -- does not end the window
  static control flow               Continue -- does not end the window
  entering an in-place closure of M Continue -- transparent navigation;
                                      ActiveInlineClosurePath_meta is erased
```

The only capability-ending event for material owned by the meta construction
is its return-stage seal (§4.3.2). A fresh meta-local PatternValue nevertheless
has `Life = MetaInvocation(M)`. Attempting to pass it to another ordinary meta
does not close or promote it; candidate formation rejects the call when the
canonical argument is not `GlobalKeyable` (§4.3.1–§4.3.3). The rejection is
total: the argument never enters the deeper invocation, so meta invocations
cannot smuggle meta-local open material into the closed world and re-open it
when the stack unwinds. `compile` and
transparent construction intrinsics may consume it because they create no new
MetaInstance key.

At seal, only `OwnedResultClosure(τ)` is promoted: for the default result `τ_M`
that is `OwnedClosure(Core(τ_M))` plus `OwnedCallSpaceClosure(CallSpace(τ_M))`
(§4.3.2); an explicitly `symbol`-typed result promotes the carried `τ`'s owned
result closure only when that `τ` is present. Other local
PatternValues expire with the invocation. Consequently the open-disposition rule for
`UseForVal1` (§12.1.2) must not be read as a universal invariant, while “meta body is
transparent” must not be read as implicit global promotion.

#### 12.1.4 The apparent self-typed intersection

With §12.1.2 in force, the ordinary case that looked like an intersection
resolves without a special rule. Suppose an RHS is an ordinary value-bearing
Object whose Pattern core is the `Q` of the type closure being extended, and the
extension is attempted from an ordinary context:

```text
construct RHS value of target type
  -> UseForVal1(target) at OpenRootCoordinate(target)
  -> Terminate -- legal action, but it ends target's open window
  -> target is no longer OpenHere_Σ
  -> attempt to extend target
  -> no applicable overload
```

So in an ordinary context there is no legal situation in which one operation both
extends the target's Pattern and contributes a complete self-typed val to the same
still-open target. A complete value of some *other* type may still be contributed
as ordinary Val2 while the target is open; its own Pattern and Val2 remain
attached to that value.

In a meta body the same sequence is simply legal: the first step is
`Continue` (the material is `MetaGenerated`), so the open window survives the
`UseForVal1` and the subsequent extension is admissible.

The empty destination `()` is the special call-entry leaf rather than a normal
value-member name. Inside construction of `T`, `let () = impl` contributes one
candidate to the same associated `()` Symbol. Candidates for receivers `T`,
`T ref`, and `T share` are distinguished by their formal object Pattern, not by
`ref`/`share` navigation subspaces; a borrowed-receiver candidate still requires
its own authorized contribution. The body of an associated `()` entry has its
own `CallableOwner`, while invocation-frame slot 0 receives the object matched
by the selected candidate.

Under equal owner/construction authority, an inner contribution and a later
inner-to-outer navigated declaration denote the same pending namespace delta:

```text
struct-local contribution under owner name1::T
  ==
later installation at name::name1::T
```

Neither spelling forwards a place or reroots the initializer's Pattern.

The language must select the expectation from semantic context or an explicit
rank/facet annotation. It must not guess `PatternChild` merely because the
right side happens to carry a type or `PatternValue`. Both paths still obey the
general symbol-resolution-then-facet-projection rule.

### 12.2 Same-symbol role/member rules

The canonical Symbol is a pair `S = ⟨τ?, V_S?⟩` where `τ` is an optional complete
type value and `V_S` is an optional candidate space. Namespace consumers read
`Core(τ)` (when `τ` is present); call consumers read the deduplicated candidate
space `CallableProjection(S) = DedupCandidateIdentity(V_S ⊎ V_τ)` (symbol-first
§2.1). Symbol role/member rules are therefore:

```text
S = ⟨τ?, V_S?⟩

install τ at most once by ordinary definition;
require WellFormedTau(τ) and Pure(Core(τ));
derive NamespaceProjection(S) from Core(τ) when τ is present;
derive TypeProjection(S) from τ only when TypeValueRole(τ);
add children only under the owning construction/authority rules;
seal/promotion uses OwnedResultClosure(τ) — OwnedClosure(Core(τ))
    plus OwnedCallSpaceClosure(CallSpace(τ)) — not a unique pure member

value members V:
  admit multiple heterogeneous value entries;
  form candidates only in a call position;
  do not infer cross-construction-unit merge authority
```

When `struct` establishes a type-role `Q` inside an already resolved owner
Pattern scope, an existing incompatible Core is a hard conflict.
Same-origin, same-material cache replay may reuse the existing core.

In particular, an ordinary symbol place receives its type core at most once:

```lang
let T = A;
let T = B;
```

If both declarations attempt to install `T`'s core, the second is a hard
conflict. It is never interpreted as:

```text
A | B
```

Three operations must remain distinct:

```text
first type-core installation
  -> ordinary core installation

add a direct child under an owned, still-open construction
  -> extend (directly or through inject)

construct or extend a sum
  -> explicit sum-construction / sum-extension API
```

The final spelling of the sum API remains open. Duplicate ordinary definitions
do not provide that API, and `extend`/`inject` must not convert an existing type or an
existing child into an implicit sum.

An explicit read-transform-bind form such as:

```lang
let T = T |> some_explicit_transform(...);
```

conceptually reads the existing value, applies a named structural
transformation, and asks the outer binding/update judgment to install the new
value. Whether that writeback spelling is permitted is reserved for later
place/update rules. It does not make two unrelated ordinary definitions
mergeable.

### 12.3 Value identity does not multiply with names

Do not infer three type values from:

```lang
let Bool = value;
let bool = value;
let t = bool;
```

If the bindings expose the same pattern/type value, the value identity is the
same. Their `SymbolId` and `PlaceId` nevertheless remain distinct and separately
observable; provenance is diagnostic material and is not part of the value's
normal form.

### 12.4 Installation is always outer-layer work

The installation flow is:

```text
compile/meta invocation
  -> compile: an ordinary PatternValue or complete type value of its declared result Pattern
  -> meta: the default result τ_M (an ordinary `Symbol` PatternValue only for an explicitly declared `symbol` result)
  -> for a source path: resolve Symbol -> read its value/facets
  -> let creates a destination or ordinary =/inject writes an existing place
  -> resolve writable install PlaceId
  -> form NamespaceDelta
  -> validate facet/child conflicts
  -> install atomically or install nothing
```

`struct` and `extend` do not mutate the namespace graph. `inject` writes one
already existing type slot through ordinary place semantics, but creates no
Symbol/member and establishes no root. New graph installation remains the work
of outer `let`/namespace contribution.

Future compile-to-runtime materialization preserves the same separation:

```text
materialization_place(result)
pattern_owner(result)
```

The first may be a newly allocated runtime owner/place or compiler-generated
`[[global]]` storage. It does not imply that the result Pattern is rerooted to
that place. Pattern owner/root/scope continue to come from ordinary result
construction semantics. Likewise, generated storage placement is not
source-visible `NamespaceGraph` symbol installation.

## 13. Non-Goals and Open Representation Boundaries

This document does not change Raw or Normalized AST syntax, introduce a general
macro system, expose unrestricted AST rewriting, or choose the final storage
representation for Pattern space, owner persistence, access trees, or
continuations.

The following semantic distinctions are fixed even while those representations
remain open:

```text
complete type != Symbol != Place
construction material != semantic result
ordinary member != TypeMember != structural field
extend = pure transformation
inject = read + extend + write
OpenHere != Writable != PolicyMode
```
