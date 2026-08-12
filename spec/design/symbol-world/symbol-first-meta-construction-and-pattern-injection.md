# Symbol-First Meta Construction and Pattern Injection

**Status: Canonical future-design direction. Not current public language
behavior and not fully implemented.** This document is the canonical design
note for symbol-first resolution, Symbol role/member projections, `compile` / `meta` result
boundaries, meta return pure-role self-root identity, resolved pattern scopes,
`struct`, pure `extend`, place-level `inject`, and the binding/install boundary.

The current implementation is a transitional substrate described in §13. In
particular, the current `PatternHeadId` attachment path must not be read as the
final owner-resolution rule.

This document builds on, without replacing:

- `spec/design/symbol-world/type-values-places-and-borrow-views.md` for
  `SymbolId` / `PlaceId` / `TypeValueId`, the borrow views `ref` / `share` / `@`,
  and independent writability / construction-lineage Open judgments;
- `spec/design/lifetime/lifetime-policy-and-overload-boundary.md` for the
  positive overloads of `@`, escape checking, and the lifetime-rule boundary;
- `spec/contracts/v0.9-pattern-head-identity-and-explicit-navigation.md` for
  the preserved bare-name versus explicit-`::` distinction and the current
  registry-backed substrate;
- `spec/design/patterns-overload/static-pattern-spaces-and-extraction-chains.md`
  for static pattern spaces, bounded extraction, and extraction-chain
  semantics;
- `spec/design/meta-invocation/meta-object-invocation-and-policy-reduction.md`
  for candidate selection, evaluation demand, policy, and residualization;
- `spec/design/build-package/build-system-design.md` for transactional
  namespace-graph assembly and physical source contributions;
- `spec/design/symbol-world/symbol-construction-units-and-namespace-origin.md`
  for namespace-facet origin, source/meta construction ownership, physical
  authority, and cross-file closure;
- `spec/design/symbol-world/symbol-policy-and-compile-flow-projection.md` for
  `Val1? x Pattern x Val2`, `Pv:Pp`, binding `P1`, result `P2`, compile-flow
  projection, derived compile companions, match staging, and automatic require.

## 1. Canonical Boundaries

The design has five load-bearing boundaries:

```text
name/path resolution:
  path/name -> Symbol -> context-directed role/member projection

ordinary value binding:
  let destination = source
    -> resolve source Symbol -> read value -> bind destination Symbol/Place

compile-time value computation:
  compile -> any ordinary PatternValue
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
   category. It resolves as a first-class symbol.
2. Ordinary `let` reads a value through the source symbol and creates a distinct
   destination symbol/place. Ordinary `=` only writes an already existing
   place. Neither operation forwards, reroots, or merges identities, and bare
   `=` never creates a missing member.
3. `compile` computes values. It may accept and return **any** ordinary
   `PatternValue`, including a type value, a symbol value, and a `type ref`.
   What it may not do is create a new global root: it registers no global
   Symbol, produces no nominal type lacking a normal global root, and never
   promotes a local temporary pattern value into a global type (§4.2).
4. For an ordinary meta callable, `meta` is static evaluation **plus** the
   authority to establish one navigable `MetaInstanceRoot`. Every such
   invocation establishes that globally identified but unsealed root on entry,
   without externally installing it, and no
   other ordinary callable coordinate may establish or seal that *kind* of
   root. It returns the ordinary Symbol value of that instance; the return stage
   promotes only the unique pure-role-member-owned closure and seals the instance
   (§4.1, §4.3). Privileged built-ins retain member-specific owner rules (§4.8).
5. In target semantics, `struct` is a symbol-producing structural generator and
   `extend` is the primitive referentially pure value transformation. `inject`
   is the explicit read--extend--write wrapper over an existing `type ref`.
   None installs a new global root; only `inject` mutates an existing slot. A
   current registry allocation used to represent a result is non-semantic
   substrate bookkeeping, not an observable effect.
6. A `let` binding or installation path chooses the installation place. It does
   not retroactively choose or reroot the pattern owner carried by the value.

## 2. Symbol-First Resolution and Role Projections

### 2.1 Conceptual Symbol value

The specification model is:

```text
SymbolValue {
    SymbolId
    PlaceId

    Q: zero or one pure Object role member
    V: zero or more heterogeneous value members in typed buckets
}
```

The target ontology stores neither an independent namespace facet nor an
independent type facet. `Q`, when present, is pure. Namespace and type
projections are derived judgments over that same Object:

```text
NamespaceProjection(S) = Q
  iff Val1(S) = <Q, V>

TypeProjection(S) = Q
  iff Val1(S) = <Q, V> and TypeRole(Q)

TypeProjection(S) defined => NamespaceProjection(S) defined
```

An implementation may cache role projections in separate buckets, but those
caches are transitional substrate, not two semantic Objects. This is not a
requirement that this PR refactor the current Rust `SymbolObject` into these
exact fields.

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

The type projection is `AsType`, not `TypeOf`:

```text
AsType(E) = E |> type
AsType(E) != TypeOf(E)
```

`AsType` neither raises universe rank nor manufactures a carrier place. Only
explicit type-of extraction may obtain the next classifier. `@` never supplies
`AsType` implicitly. A Symbol's `.type` family is applicable exactly when its
unique `Q` satisfies `TypeRole`: `S.type` reads `Q` by value, `(S ref).type`
projects `type ref`, and
`(S share).type` projects `type share`. Only an already-pure type slot uses
direct `t@`.

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

### 2.2 Role and value projections coexist

One symbol may simultaneously provide:

- one optional pure role member `Q` and its namespace projection;
- the type projection of that same `Q` when `TypeRole(Q)`;
- an ordinary value;
- a callable value;
- multiple heterogeneous value entries forming an overload candidate set.

The symbol remains one symbol. Namespace and type are not independently stored
Objects, and coexistence does not collapse role, value, symbol, or place
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
  stable first-order type root (registry projection); the full
  type-object semantic identity at an observation moment is the
  canonical observation Addr(Norm_type), not the bare TypeValueId

PatternValue identity:
  canonical identity of an ordinary compile-time value, type value, or
  structured pattern value

PatternScopeId:
  identity of a navigable pattern-owner layer
```

No equality implication is automatic between these identities.

### 2.4 Program text names symbols before values

Except for literal syntax and other explicitly specified immediate values,
program text does not directly name a value. A source path first names a
symbol, and value use then reads a facet/value from that symbol:

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

For example:

```lang
let a = 'a';
```

The left `a` is a symbol name. The right `'a'` is a character literal. Their
textual content happens to match, but they are not one semantic object.
Pattern values have no comparable standalone literal syntax, which makes a
same-spelled source path and pattern diagnostic projection especially easy to
confuse. The language still resolves the source path as a symbol first.

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

An omitted P1 retains and infers the complete RHS result; it does not make
runtime the only way to obtain a runtime binding.

The bounded migration prototype does not reinterpret a P1 query as an exact
target. Any non-empty `ProjectP1` result completes the binding and makes
migration unreachable. Only after the complete query projects nothing may an
accepted runtime branch be extracted and paired with an eligible static input
view for one language-authorized atomic migration. The compiler mandates the
static-to-runtime stage edge; candidate-declared endpoint mutability belongs to
ordinary overload. Empty queries with no runtime alternative fail, and no
Policy failure searches structure-changing operations. See
`../../contracts/v0.6-cross-policy-value-transition.md`.

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

value(U) = value(T) = value(uint8)
TypeValue(U) = TypeValue(T) = TypeValue(uint8)
PatternValue(U) = PatternValue(T) = PatternValue(uint8)
```

`type` is an expected rank/facet assertion applied while evaluating the RHS.
It does not select a second “type binding” judgment.

Canonical summary:

```text
Program text normally cannot name values directly. It names a symbol, then
obtains a value through that symbol.

Name navigation is a way to obtain a value, not part of ordinary value
identity.

Pattern navigation paths are likewise symbol navigation first. Even when a
PatternValue's canonical navigation name matches the symbol carrying it, the
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
the borrow views `ref`, `share`, and `@`, specified in
`type-values-places-and-borrow-views.md`.

The canonical conclusion is:

```text
value equality does not imply symbol equality;
value equality does not imply place equality;
no declaration converts value equality into place sharing.
```

Therefore several symbols may expose the same `TypeValueId` or pattern value
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
the symbol `f`. The entry need not originate from closure syntax and need not
be callable.

Multiple entries under the same symbol may have heterogeneous types. A same-name
value-member family is therefore not equivalent to a traditional same-signature
function-overload bucket.

### 3.2 Call candidate preparation

A call position performs the following conceptual flow:

```text
resolve symbol
  -> project typed V members
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
preparing candidates for a call position. Its presence does not make the symbol
invalid and does not turn it into a function overload.

Candidate identity and applicability belong to the candidate/invocation model;
symbol-first resolution only establishes where the heterogeneous values come
from. Derived compile companions are complete first-class `Val2` function
objects, not post-failure fallback entries; their policy and overload
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

result rank:
    PatternValue / runtime value
```

There is no third result rank. A symbol value is an ordinary `PatternValue`
(§4.7), so "returns a symbol" is a statement about which pattern value is
returned, not about a separate ontological rank.

`MetaPartial` / `MetaStrict` describe evaluation demand. They do not define the
meaning of `compile` or `meta`, and they do not determine the successful result
rank.

Callable semantics still use ordinary PatternValue result declarations; there
is no private construction result rank:

```text
CallableSemantics
    = P1 × P2 × DeclaredResultPattern × Privilege

Privilege   ::= Ordinary | BuiltinPrivileged   -- bounded AST access
```

`compile` may return any declared ordinary PatternValue, including a Symbol.
Ordinary-meta callable kind, call legality, and successful-call effects are
separate judgments inside the ordinary value/policy model:

```text
F in OrdinaryMetaFunction
  => P2(F) = meta
  and ResultPattern(F) = symbol

WellFormedMetaCall_Gamma(F, args)
  <=> F in OrdinaryMetaFunction
   and Admissible_Gamma(F, args)
   and forall a in Canonicalize(args): GlobalKeyable_Gamma(a)

WellFormedMetaCall_Gamma(F, args)
  => M = MetaInstance(F, Canonicalize(args))
   and RootIdentityExists(M)
   and ConstructionNavigationAvailable_Gamma(M)
```

Equivalently, and without overloading “return shape”:

```text
ReturnClassifier(F) = symbol
ReturnShapeWithinSymbol(F(args)) = Σ = ⟨ Q?, V ⟩
```

The classifier is fixed for every ordinary meta callable. `Q`, when present, is
the unique pure role member and may or may not satisfy `TypeRole`; `V` may
contain any ordinary sibling values. These are content facts about one Symbol
Object, not type/val/namespace result categories. Namespace projection selects
`Q`; type projection selects the same `Q` only when `TypeRole(Q)`.

Callable kind fixes `P2` and the result classifier; `GlobalKeyable` belongs to a
particular call's well-formedness, never to the callable type itself. A
successful call establishes a globally stable root identity and makes it
navigable to the construction, while sealing remains the return-stage effect.
No `compile` callable may establish or seal this root kind.

This exclusivity does not claim that every stable owner/root in the language is
a `MetaInstanceRoot`. Lexical declarations and privileged built-ins may
establish, select, or preserve other root kinds only through their separately
specified owner rules (§4.8). They cannot use those rules to manufacture an
ordinary navigable `M`.

This is not a new SymbolConstruction rank. The returned Symbol is an ordinary
PatternValue whose mutable member content is `Val1`; root authority governs the
construction lineage and global lifetime of its unique pure-role-member-owned
closure. An implementation may retain a carrier to accumulate those members,
but may not expose that carrier as a callable result ontology.

`ClusterSymbol` and `ReturnShape::ClusterSymbol` survive in the current
implementation as a **transitional carrier** for the multi-member return of a
meta construction. They are not an ontological category of the target semantics
and must not be treated as one. Three roles that the single name `ClusterSymbol`
has been used to conflate are distinct:

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
  input  ordinary PatternValue
  -> output ordinary PatternValue
```

`PatternValue` includes:

- ordinary compile-time values;
- type values;
- symbol values;
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

This is a conservation law, not a shape restriction. Returning a symbol value or
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
`inject` because `Open_Γ(Read(ref))` is false. Escape checking belongs to
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
  — the current compile-time stack used with each value's ConstructionLineage
```

The definition context decides names and lexical owners. The caller context is
used only by operations that query `Open_Γ(v)`: they compare the value's
`ConstructionLineage` with the current compile-time stack (§12.1.1). Neither
context substitutes for the other.

Passing through a `compile` call, cloning, selecting, or composing a value
preserves its canonical value and `ConstructionLineage` while discarding source
place identity. A compile frame is transparent to the Open stack walk, so an
Open value remains Open through any number of compile/transparent-intrinsic
frames unless another semantic boundary closes its construction interval:

```text
Lineage(Clone(Read(q))) = Lineage(Read(q))
Open_{Γ + compile-frame}(v) = Open_Γ(v)
```

The formal-parameter case is ordinary value transport:

```lang
let extend =
    (self, t: type): compile -> out: type => {
        (t, ...) |> extend;
    };
```

The call is applicable only when the transported value is Open in the caller's
stack:

```text
Requires(extend) = Open_Γ(t)
Γ_caller ⊨ Open(Lineage(t), CompileTimeStack_caller)
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
Requires(extend_ref) = Open_Γ(Read(t)) ∧ Writable_Γ(Target(t))
```

Hence compile context sensitivity is value-lineage sensitivity, never a hidden
capability on `type ref`:

```text
a compile evaluation depends on the caller's Open window
  exactly for operations that query Open_Γ on a transported PatternValue
```

not as a general property of every `compile` call, and not decided by whether a
`type` value happens to be a formal parameter. Caches and `Requires` summaries
track `ConstructionLineage` separately from canonical value identity and recheck
applicability in the caller stack.

`compile` does **not** create a `MetaInstanceScope`, does not introduce a
meta-style virtual symbol layer for name shadowing, and does not impose a
self-root requirement on a returned type value. It may freely return an
already existing value:

```lang
let identity = (self, t: type): compile -> r: type => {
    let r = t;
    r;
};
```

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
Symbol crosses an ordinary outer binding/namespace-installation boundary and
that delta commits atomically (§12.4). The returned value is the ordinary Symbol
value of `M`:

```text
meta:
  accepted parameters
  -> the symbol value of M
```

A meta callable may accept a `symbol` parameter, or constrain a parameter to a
narrower `type` or ordinary PatternValue. That does not introduce another result
rank: successful ordinary meta invocation still yields `symbol`. `M` exists in
the global world from body entry; the return stage validates the at-most-one pure
role member constraint, promotes only that member's owned PatternValue closure, and
seals the result.

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

The only construction-closing event of a meta invocation is its final return
stage, and it runs in a fixed order. The returned symbol has the ordinary Symbol
shape — at most one pure role member, any number of val members:

```text
Σ = ⟨ Q?, V ⟩

1. validate that there is at most one pure role member and that Q is Pure
2. if Q exists, promote OwnedClosure(Q) into M and call it P_Q
3. validate the escape dependencies of the entire returned Symbol Object
4. seal M
```

Step 1 is a cardinality bound, **not** a requirement that a role member be
present. Nothing in the Symbol ontology or in the self-root constraint promotes
`|Q| <= 1` to `|Q| = 1`: the self-root rule says that *if* `Q` exists it must be
rooted at `M`, which is vacuous when there is none. A namespace-only `Q` is
therefore a valid promotion anchor even when `TypeRole(Q)` is false; type-role
requirements are refinements, not generic Symbol constraints. Step 2 is simply
skipped when `Q` is absent:

```text
Q present -> EscapeDeps(ReturnSymbol)
               subset AlreadyGlobalStable union P_Q
Q absent  -> EscapeDeps(ReturnSymbol)
               subset AlreadyGlobalStable
```

`EscapeDeps(ReturnSymbol)` traverses the entire returned Symbol Object through
`Children_Val1 union Children_Val2`, including nested products, Sequences,
callables, and navigable `Val2` structures. It additionally includes every
target reached through a horizontal `ref` / `share` / `rebind` view. Thus no
returned branch can smuggle unrelated meta-local material out of the invocation.
Borrow edges remain excluded from `OwnedClosure(Q)` and are never promoted
merely because they are referenced.

Step 2 promotes the **owned** closure only. Horizontal borrow edges are not
ownership and are never dragged into the promotion:

```text
OwnedClosure(x) excludes every ref / share / rebind edge reachable from x
```

For this promotion, “owned closure” is not arbitrary graph reachability. Let
`OwnedNavigation_Q(x, y)` hold only when `y` is a genuine direct child owned by
`x` in Q's construction tree. Then `OwnedClosure(Q)` is the least closure under
that relation, subject to all of these invariants:

```text
direct child only:       every step is parent -> direct child
no jump:                 a parent cannot inherit a deeper descendant directly
bare termination:        Bare(x) stops expansion for Q
external termination:    ExternalTo(Q, x) is an opaque dependency leaf
no external re-entry:    expansion never leaves Q, enters an external subtree,
                         and later re-enters Q-owned material
no cycle:                 x not-in OwnedNavigation_Q+(x)

OwnedNavigation_Q(x, y) => DirectOwnedChild(x, y)
Bare(x) | ExternalTo(Q, x) => no y: OwnedNavigation_Q(x, y)
ExternalTo(Q, q_i) => no j > i: Owner(q_j) = Owner(Q)
```

External leaves may retain their own independently owned trees, but those trees
are not promoted through `Q`; their dependencies must already be globally
stable. The ordinary recursive Object normal form still traverses
`Children_Val1 union Children_Val2`; this construction judgment only determines
which fresh-owned part may acquire M's global lifetime.

A member reachable only through a borrow view is therefore not promoted, and its
presence does not extend `M`'s owned material. Its target must already satisfy
the step-3 escape condition. After step 4, `M` is sealed and nothing may reopen
it.

#### 4.3.3 `M` as a navigable layer

Every ordinary canonical meta-function invocation establishes a virtual
symbol-construction scope:

```text
M = MetaInstanceScope(callee_symbol, canonical_arguments)
```

Formation additionally requires:

```text
for every canonical argument a:
  GlobalKeyable(a)

OwnedDependency(a) != GlobalKeyDependency(a)

Borrow(q) in a
  => Target(q) in GlobalKeyDependency(a)

GlobalKeyable_Γ(a)
  <=> every d in GlobalKeyDependency(a) is, at key-creation time,
        AlreadyGlobalStable_Γ(d)
      | AlreadyPromoted_Γ(d)
```

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

This is not merely a folder analogy. `M` is a symbol/namespace layer that
participates in default pattern navigation and name shadowing, may carry the
optional pure role member and ordinary value members, anchors cache/incremental identity, and owns
the return construction transaction. An ordinary meta invocation must therefore
establish its own symbol layer rather than act as a value-level forwarding
function.

The externally navigable result symbol is `M` itself. The declared return slot
is only a lexical name for that symbol, not a transferable construction rank:

```text
symbol_of_result(invoke_meta(callee, canonical_arguments)) = M
return_slot(r) = lexical_name(M)
```

The slot name `r` does not add another component to the final navigation path.
Material written through `r` contributes role/value members or children to `M`; it does not
create `r::M` or place an extra symbol named `r` beneath `M`. For example, a
pattern-child contribution written as `let t1::r = bool;` inside the invocation
targets `t1::M` under the applicable pattern-construction expectation, not
`t1::r::M`.

Canonical argument identity follows parameter rank:

```text
symbol parameter -> SymbolId / symbol-place identity
type parameter   -> canonical observation of the evaluated type object
                    = Addr(Norm_type)   (TypeValueId is only the
                      first-order root, never the argument identity)
value parameter  -> PatternValue identity
```

The exact inclusion of `PlaceId` in a symbol-parameter key depends on whether
the callable observes the symbol's installation place. A key must not silently
replace symbol identity with type-value equality.

### 4.4 Ordinary meta return pure-role self-root invariant

If the return symbol of an ordinary canonical meta invocation has a pure role
member `Q`, its outermost pattern root must be the invocation's own `M`:

```text
RoleMember(r) = Q
  => Pure(Q)
   and root_pattern_scope(Q) = M
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
Neither value may directly replace the return symbol's required role root.
The failure is the hard diagnostic `MetaReturnRoleRootMismatch` (the current
implementation may retain `MetaReturnTypeRootMismatch` as a transitional code). An
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

External `PatternValue`s may be members of the self-rooted `Q`; they may not
replace the root. For example:

```lang
let fn = (self, t: type): meta -> r: symbol => {
    let t1::r = bool;
    r;
};
```

keeps `(t fn)` as the return symbol's root and includes the externally owned
`bool::` value as a member beneath that root. It must not be summarized as
`RoleMember(r) = bool::`.

The self-root check is conditional on `Q`, not on `TypeRole(Q)`. A namespace-only
`Q` is self-rooted and may own fresh invocation-local material. A return Symbol
with no `Q` does not acquire a synthetic role member merely to satisfy this
rule. When `TypeRole(Q)` does hold, `DefinesVal1(P(Q))` is the additional type
refinement; namespace-only `Q` is not required to define Val1.

### 4.5 Formal return material

Target semantics do not give the spelling of a return slot a special creation
meaning. A meta body computes an ordinary Symbol value; `let` creates its local
members, `=` writes existing places, and the return event transfers that value.
The current execution substrate still maps the explicit return-slot spelling
`r` onto an open construction carrier. That mapping is transitional
compatibility encoding only.

Formal meta return material is a family of distinct construction-effect forms,
not one spelling-insensitive binding. The *family split* — create / write /
deliver are three distinct events that never collapse — is fixed. The
*spellings* below live on two different layers and must not be read as one rule:

```text
Current execution encoding (this stage, while expression-level `=` does
not exist):

    let r = expr;     -> AddMember — return-slot compatibility encoding:
                         one fresh member event on the return symbol;
                         ordinary locals may not shadow the explicit
                         return slot
    r = expr;         -> PlaceholderOverwrite — placeholder write to an
                         existing target
    r;                -> terminal: deliver the construction (not a member
                         event)

Target orthogonal semantics (future, once `=` is semantic):

    let x = expr;     -> creates a fresh Symbol/member according to the
                         declaration context; a binder spelled r is ordinary
    target = expr;    -> Write(existing target, expr)
    return event      -> control transfer only
```

- `let r = expr;` contributes a fresh member binding under the current
  encoding. Repeated `let r = ...` forms do not shadow one binder; each adds
  one more member event on the same open Symbol. This
  spelling-directed reading — and the no-shadow restriction that protects
  it — is the return-slot compatibility encoding removed when
  expression-level `=` becomes semantic (§4.5.1); it is
  not a permanent rule that `let` on a return-slot name means member
  contribution.
- `r = expr;` writes to an existing target; a write is not append, and a
  construction model that only supports appending cannot express
  `let r = first; r = second; r;`. The current implementation of this
  effect (internally `PlaceholderOverwrite`) is a placeholder scaffold
  while expression-level `=` does not exist: it replaces the unique
  existing member of the written facet, purely to validate
  existing-target addressing. That unique-member replacement rule is not
  the final write algebra for a multi-member symbol — how a real `=` adds or
  replaces the pure role member / val siblings by RHS shape is registered
  implementation debt in §13.
- In the current compatibility encoding, `r;` is the TailValue terminal. It
  delivers the constructed symbol to the directly enclosing layer. It is not a
  member contribution, and a meta body with member events but no terminal does
  not implicitly deliver anything. The target return event is only control
  transfer and does not give the spelling `r` special binding semantics.

The terminal family follows the general control-flow end model: `expr;`
delivers to the directly enclosing layer, `expr return;` returns to the
outermost function layer, and `expr (T return);` returns to the layer selected
by the function-object type `T`.

Add-fresh-member and write-to-existing-target are two distinct construction
effects. They must not be collapsed into one injection event, and neither is a
return. Whether contributed material references an existing `PatternValue`,
computes new material, or projects a symbol member is represented inside the
construction value; any resulting pure role member must pass the self-root invariant in
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

Assignment carries no `extend`-specific validation, but that is not the same as
carrying no validation. A pure `extend` in the right side already discharged
`Open ∧ ParentToChild ∧ NoPatternConflict`. The place-level `inject` wrapper
performs that check before its own write. Everything else that applies to any
write still applies. A write `lhs = rhs` is checked in four independent layers:

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
     WellFounded(v)
     Canonicalizable(v)
     NoForbiddenCycle(v)
     -- a write forming a non-normalizable Val2 cycle fails, even when it comes
        from an ordinary assignment

4. semantic-boundary constraints of the enclosing region
     meta return self-root; ref / pattern-value lifetimes;
     mutability limits on global type-bearing values; seal / global-promotion
     rules; the single-pure-role-member bound (`|Q| ≤ 1`) on a returned Symbol
     -- these may run at write time, normalization time, return time, or
        install time, but they all remain in force
```

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

This distinction does not cancel `let f::(t@) = expr` for an already-pure type
slot, or `let f::((S ref).type) = expr` for a Symbol whose `Q` satisfies `TypeRole`
(ordinary `Val2` member creation at an explicit type place), and does not change the `r;`
terminal semantics. The current
`let r =` binding-to-return-value with no-shadow is a transitional encoding, not
the target rule.

A symbol construction value is not restricted to newly generated structure
definitions. It may describe a fresh return symbol with its own `SymbolId` and,
once bound, a potentially independent `PlaceId`; it may also reuse existing
values as ordinary value-facet material or as members of a newly self-rooted
type construction:

```text
SymbolConstruction {
    return_symbol_identity,
    assigned_role_or_value_members,
    optional_child_contributions,
    provenance,
}

assigned non-root value/member may equal an already existing PatternValue
```

Value equality remains independent of source name and navigation path and does
not merge symbol or place identity. However, that general identity separation
does not waive the pure-role self-root invariant: `r = uint8` as a direct meta
return role installation is rejected after symbol resolution/value read, rather than
being reinterpreted as forwarding or accepted as an identity meta type.

### 4.7 A symbol is an ordinary PatternValue

A symbol value is not a separate ontological rank. It is an object with the same
three components as every other object:

```text
SymbolValue = ⟨ Σ, P_symbol, Val2_symbol ⟩

Σ = ⟨ Q?, V ⟩
V = ⨄_{T_c} V[T_c]
V[T_c] : T_c * omega
```

Its member content is ordinary object content:

```text
at most one pure role member Q
any number of val members
```

Because the member content is the mutable part, it lives in `Val1`:

```text
Val1(Symbol) = Σ = ⟨ Q?, ⨄_{T_c} V[T_c] ⟩
```

`Σ` is a logical view over ordinary Object containers, not a
specification-private record carrier. Using the constructor lemmas in
`type-values-places-and-borrow-views.md`:

```text
RoleOption(absent) = BareProduct()
RoleOption(Q)      = BareProduct(Q) where Pure(Q)

BucketEntry(T_c)  = ProductValue(T_c, V[T_c]) : product
BucketCarrier(V)  = Seq_omega(product; BucketEntry(T_c) for each occupied T_c)

Σ_Object(Q?, V)   = BareProduct(RoleOption(Q?), BucketCarrier(V)) ∈ Object
Val1(Symbol)       = Σ_Object(Q?, V)
```

The notation `⟨Q?, V⟩` merely projects the two ordinal positions of this bare
Product Object. Every `V[T_c]` is itself the ordinary `T_c * omega` Sequence
Object, and every bucket entry is classified by the global `product` type so
the bucket carrier remains genuinely homogeneous. Symbol normalization applies
its unordered quotient to this ordinary carrier; neither `Σ` nor its buckets
introduce a compiler-private semantic collection.

Each `V[T_c]` contains ordinary member/candidate objects of their actual type
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

No element information is erased from `Val1`. This PR defines no general
runtime `product[]`; a sound result needs dependent/existential result material
or a type witness and remains deferred. The four ordered-container cases are:

| element shape | fixed concrete outer shape | erased outer shape |
| --- | --- | --- |
| homogeneous | `T * N` | `T * omega` |
| heterogeneous | bare Product | `product` |

The Symbol Pattern applies an unordered identity quotient to each typed bucket:

```text
DecodeSymbolPayload(Σ_Object) = ⟨ Q?, V ⟩

Norm_Val1?^P_symbol(Σ_Object)
  = ⟨ Norm(Q)? ,
      { Norm(T_c) ↦ Set{ Norm(v) | v ∈ V[T_c] } } ⟩
```

If distinct `T_c` keys normalize equally, their buckets are combined under that
normalized key before the set quotient. Carrier position, insertion order, and
replayed contribution of the same stable member do not enter Symbol identity:
`Σ + Σ = Σ`. Duplicate declarations, conflicting definitions, and same-root
conflicts are diagnosed in construction/well-formedness before normalization;
they are not remembered as value multiplicity. Distinct stable member objects
remain distinct even when their callable bodies normalize alike. In particular,
`s += a; s += b;` and `s += b; s += a;` normalize equally exactly when their
optional pure role member and every typed member set are equal.

Callable val members project the formal overload set directly from this value:

```text
OverloadSet(Σ, q)
  = ⨄_{T_c} { v ∈ V[T_c] | Callable(v) ∧ q(v) }
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

Applicable(type candidate, Σ) <=> Σ = <Q, V> and TypeRole(Q)
```

Thus `S.type` agrees by value with `AsType(S)`, while `(S ref).type` and
`(S share).type` preserve the borrow observation of the `Q` slot when
`TypeRole(Q)`.
This is ordinary field/candidate selection, not a resolver primitive that
projects a value and then recovers its provenance.

The consequence is that symbol-level operations are `Val1` transformations and
leave the symbol's own pattern untouched:

```text
s = new_symbol                   -> replaces Val1(s)
s += contribution                -> extends Val1(s)
s -= contribution_family          -> removes a typed contribution family from Val1(s)

in every case:  P_symbol unchanged
```

A symbol is therefore an ordinary value that can be computed, passed, and
returned like any other — including by `compile`, subject only to root
conservation (§4.2.1). The four roles listed in §4.1 (value ontology, meta return
construction, namespace same-name synthesis, world installation) are separate
concerns that happen to involve symbols; none of them is the symbol's ontology.

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
return ordinary PatternValues rather than a construction rank;
declare explicitly whether they are pure or write an existing place.
```

Privilege buys a bounded AST carrier and a special scope/owner rule — it buys no
result ontology. There is no shared "construction handle" return family and no
third result rank (§4.1):

```text
extend  : type × StructLikeMaterial -> type
inject  : type ref × StructLikeMaterial -> type ref
struct  : StructLikePattern -> symbol
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
extend the current pure role member's navigable structure
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

This is a future semantic/navigation rule. It does not change the current lexer,
parser, Raw AST, or Normalized AST in this PR.

## 7. `struct`

### 7.1 Public boundary

`struct` is a `BuiltinPrivilegedAstMetaFunction`, not an ordinary user-definable
meta function. It uses the general function-object/meta call framework but does
not create its own ordinary externally navigable `MetaInstanceScope M`.

The public semantic boundary is:

```text
struct:
  StructLikePattern
  -> symbol
```

An implementation may carry AST or Normalized AST as a private structured
carrier. The public result is an ordinary Symbol PatternValue, not AST and not a
separate construction rank (§4.1, §4.7–§4.8). Its `Val1` contains exactly one
pure-role member `Q_struct` satisfying `TypeRole(Q_struct)`, plus any ordinary
sibling values explicitly contributed by the construction. Section 7.5 closes
the mechanically generated field/access/ref/share/assignment partners in
`Q_struct`'s associated
`Val2`; it does not imply a closed defining-Symbol recovery path for other
type-as-callee sibling families. This bounded capability does not expose a
general macro system.

In the general Symbol notation this producer-specific guarantee is:

```text
Val1(struct(material)) = <Q_struct, V>
Pure(Q_struct)
TypeRole(Q_struct)
```

Thus general Symbol and ordinary-meta ontology use optional pure `Q`; `struct`
specifically guarantees that its `Q_struct` exists and is type-capable.

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

Formal `struct` invocation is, in target semantics:

```text
graph-installation-free
binding-free
referentially pure
```

Purity means that `struct` does not install the returned Symbol or mutate an
input place. It may establish the result type's declared `StructLexicalRoot`
under its privileged owner rule, but outer `let` remains the only operation that
creates the destination Symbol/member in the surrounding graph.

It does not install a `NamespaceDelta`. The current implementation may allocate
or attach registry-backed pattern material while forming the invocation value.
That is a non-semantic implementation record: it may affect cache/storage
mechanics but is not observable in `Norm`, does not mutate language-visible
input, and does not weaken the target claim of referential purity. Graph
installation remains outside formal invocation.

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

For a structural field `f : A` owned by `Q_struct`, let
`T = AsType(Q_struct)`. `struct`
uses one general field rule. It does not introduce a separate semantic category
for “type fields”. All observations are candidates of one same-name associated
Symbol `f`; receiver and result observation kinds distinguish the overloads:

```text
f : (object: T)       -> A
f : (object: T ref)   -> A ref
f : (object: T share) -> A share
```

`ref` and `share` are not generated navigation subspaces. The associated Symbol
is installed once beneath `Q_struct`'s place; `const let` / `let` /
`mut let` policy and the formal object type determine its candidates.
Their selection uses the ordinary context-indexed preference relations. In a
plain context `succ_plain: let > const = mut`; if no plain `let` candidate is
admissible, a surviving `const` and `mut` pair remains ambiguous rather than
being resolved by generation order.

Where the field policy permits mutation, the same generator also contributes
the corresponding assignment/write candidate over `T ref × A`; assignment
still uses the general existing-place write rule and never creates the field.
Written `const let` / unqualified `let` / `mut let` field policy selects the
admitted value, shared, mutable, and assignment cells of this ordinary overload
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

The generated partner candidates live in same-name associated Symbols in
`Q_struct`'s `Val2`. The returned Symbol's `Val1` contains `Q_struct` and any
ordinary sibling values explicitly contributed by the
construction; this section does not duplicate the generated accessors into a
second sibling universe. The partners are ordinary typed member objects: user
construction may remove them, replace them, or add a more specific declaration
subject to the ordinary duplicate, fallback, and overload rules. They are not
hidden compiler metadata.

The closure claim of this PR stops at the same-name field value/ref/share
observations and the corresponding assignment/write partners described above:

```text
#99 closes = field + access + ref/share observation + assignment/write partners
```

It does not yet define `HomeSymbol(TypeValue)` (or an equivalent recovery from a
canonical type root), nor how a copied or extracted type used as a callee finds
constructor or policy-transform siblings of its defining Symbol. Those are
explicitly deferred. Any future solution must be a semantic property or
recoverable relation of the canonical type root; it may not use the most recent
binding carrier, source place, or reverse provenance from `AsType`.

Construction state propagates only along owned field relations:

```text
Open_Γ(child)    => Open_Γ(parent)
Frozen(parent)   => Frozen(child)
```

Borrow edges are horizontal and do not participate. Mutability is independent:

```text
mut(child) does not imply mut(parent)
mut(parent) does not imply mut(child)
```

This same rule makes a typeclass-like object an ordinary struct; its fields are
compile-only exactly when they fail `RuntimeField`, not because they inhabit a
separate “type/PatternValue field” category.

### 7.6 Internal construction and later extension normalize equally

An element written inside the original `struct` input and an equal element
added later through the owner's navigated structural-extension path differ only in how
their full navigation is obtained. For example:

```lang
let t = ((bool inner)t) |> struct;
```

and the construction sequence using place-level `inject`:

```lang
let s = (()t) |> struct;
let t_ref = (s ref).type;
(t_ref, bool inner) |> inject;
```

produce Symbols whose `Q_struct` members both satisfy `TypeRole` and have the
same normalized PatternValue, provided the read value is Open and the destination slot is
writable:

```text
NamedPattern(
  name = t,
  child = inner::t -> Norm(bool::)
)
```

The first form inherits/completes `inner` under `t`; the second supplies the
same complete navigation through pure `extend`, then writes it back through the
type-member slot reached by `(s ref).type`. After
completion, normalization retains only the complete navigation and normalized
resident value. It erases whether the element was internal or extended, and
whether its navigation was inherited or explicit.

> **Correction:** Ordinary navigated
> `let inner::((s ref).type) = bool::;` does **not** produce the same PatternValue.
> It only installs `bool::` as an associated type (Val2 member) named
> `inner` under `t`'s scope, without registering `inner` into `t`'s
> Pattern canonical structure. Registering a member into the Pattern
> structure is a privilege held exclusively by `struct` inline construction and
> the `extend` primitive (directly or through `inject`). See §12.1 for the full
> privilege boundary.

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

`extend` takes an ordinary type PatternValue and struct-like child material, and
returns a new type PatternValue:

```text
extend : type × StructLikeMaterial ⇀ type

Extend_Γ(old, Δ) ⇓ new
```

`extend` establishes no root and preserves the root already carried by its
input:

```text
Root(new) = Root(old)
```

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

`old` is an input value and is left exactly as it was. `new` is a distinct
resulting value. Discarding `new` produces no symbol-world side effect, because
there was never a side effect to discard.

#### 8.2.1 Failure is total

```text
failure => no partial result, no write, no rollback
```

Because `extend` writes nothing, a failed `extend` has nothing to undo. There is
no half-extended pattern, no compensating action, and no rollback protocol. A
failed call simply produces no value.

#### 8.2.2 `extend` applicability is a value-lineage judgment

The primitive checks the old value in the current compile-time context:

```text
Γ ⊢ old : type
Open_Γ(old)
ParentToChild(old, Δ)
NoPatternConflict(old, Δ)
Canonicalizable(result)
--------------------------------
Γ ⊢ (old, Δ) |> extend : type
```

`Open_Γ(old)` is derived from `ConstructionLineage(old)` and the current
compile-time stack (§12.1.1), not from a carrier place. Clone/read preserves
lineage:

```text
Lineage(Clone(old)) = Lineage(old)
```

Consequently an Open value with no writable carrier may be extended and bound
elsewhere, while a frozen value read through a writable `type ref` is rejected.
There are deliberately no `type ref` or `type share` overloads for `extend`.

A navigated `let child::target = result;` is **not** a structural installer:
ordinary navigated `let` creates a Val2 associated member and never substitutes
for `extend` or for the write-back performed by `inject`.

#### 8.2.3 `inject` is the read--extend--write wrapper

`inject` accepts exactly a writable type-slot view and struct-like material:

```text
inject : type ref × StructLikeMaterial ⇀ type ref

Inject_Γ(r, Δ):
  require Writable_Γ(Target(r))
  old := Clone(Read(r))
  new := Extend_Γ(old, Δ)       -- independently requires Open_Γ(old)
  Write(Target(r), new)
  return r
```

The two requirements are deliberately independent:

```text
CanInject_Γ(r, Δ)
  = Writable_Γ(Target(r))
  ∧ CanExtend_Γ(Clone(Read(r)), Δ)
```

`r : type ref` proves target/lifetime/capability only. It never proves the
current pointee Open. A frozen pointee may therefore be replaced wholesale by
ordinary assignment through a writable ref, while `inject(r, Δ)` fails before
the write because its `extend` step is inadmissible.

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
change owner identity or reopen a frozen value. `inject` additionally requires
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
- overwrite an existing pure role member;
- delete an existing child;
- implicitly reroot an arbitrary external pattern value;
- mutate the input value or the installed namespace graph;
- extend a value that is not `Open_Γ` in the calling context;
- grant a general macro or arbitrary AST-rewrite capability.

`inject` adds only the ordinary write to an already existing target; it does not
relax any `extend` restriction. Failing Open or write applicability produces no
partial write.

## 9. Pattern-Layer Ordering

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
spelling does not turn it into a symbol reference. Conversely:

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
  ?-> PatternLayer(_, B, O)
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

### 11.1 Navigation always reaches a symbol before a value

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
named pattern value. Source navigation names symbols first. A pattern's
canonical/diagnostic navigation may match a source symbol spelling without
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

because `M` contains evaluated canonical navigation/value entries, not symbols
or symbol references.

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
> `NamespaceValueMember`, regardless of whether `expr` is `null × P × Val2`
> (a pure type object) or `Val1 × P × Val2` (a complete value). The
> expectation is never guessed from the RHS shape.
>
> ```text
> let f::((t ref).type)   -> NamespaceValueMember (always)
> struct inline / extend  -> PatternChild (privileged)
> ```

Under `PatternChild`, the source path is resolved to a symbol and projected to
its type/pattern value. The resulting `PatternValue` is installed as a child of
the owner's type construction and participates in normalization and extraction:

```text
resolve source Symbol
  -> project/read PatternValue
  -> contribute to the owner Object's Pattern/type-role construction
```

This expectation is exercised by `struct` inline construction elements and
`extend`. It requires the input PatternValue to be `Open_Γ`; `inject` reaches
the same rule only by reading its ref and invoking `extend`.

Under the current `NamespaceValueMember` implementation expectation, the source
is projected through its ordinary `V` members and a namespace value Symbol is
constructed. This changes only the namespace graph/value members; it does not
enter or change the owner's
`PatternValue`:

```text
resolve source Symbol
  -> project/read value (including pure type objects)
  -> install as associated Val2 member
  -> does NOT modify target Pattern canonical structure
```

This is the expectation of:
- Explicit-place navigated `let f::((t ref).type) = expr`
- An ordinary let-shaped declaration consumed inside `struct` construction:

```lang
let name = expr
```

It contributes one associated member to the current Pattern owner's
`Val2` value-member structure:

```text
target pure-P contribution = none
injected member             = the complete expr object (Val1 × P × Val2 or null × P × Val2)
```

The initializer is not restricted to type/Pattern material or to `Pv=absent`.
It may contribute any ordinary heterogeneous value entry, including a callable
function object or a pure type object. Its `P(expr) × Val2(expr)` remains the
recursive structure of that installed member; it is not spliced into the target
owner's pure Pattern. The construction stores the complete member as an
associated value contribution; it does not mutate the namespace graph during
`struct` evaluation.

The four-way classification of installed members:

```text
Associated member     : Val2 中存在
Associated type       : Val2 中存在 null × P × Val2 成员
Structural child      : Val2 成员已登记到父 P 正规结构
Bare structural value : 登记到正规结构但局部模式为 ε

ordinary let -> produces the first two only
struct / extend -> can produce the third and fourth, with privilege
```

This distinction supersedes the previous text which described Pattern-value
injection as a possible outcome of associated-member `let`:

```text
Privileged structural registration (struct inline / extend ONLY):
  null × P × Val2
  -> registers pure Pattern material into target P canonical structure
  -> the member becomes a structural child with extraction/construction capability

Ordinary Val2 installation (let f::(type_ref) = expr, always):
  null × P × Val2  -> installs as associated type (Val2 only)
  Val1 × P × Val2  -> installs as associated value (Val2 only)
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

Resolving `let f::((t ref).type) = x` borrows the target Symbol and projects its
ordinary same-name type-member field as `type ref`, derives the
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
a Val1. The `ObjectPlace` entry carries only the TypeObject transport
reference needed to index a pure type object by value id; that adapter is
globally reused per TypeValue and is never a binding-Policy carrier.

A pure P is a real object, so the place is per carrier, never per
PatternValue:

```text
let T: type = uint8;
let U: type = T;

Pattern(T) = Pattern(U) = Pattern(uint8)
Place(T)  != Place(U)  != Place(uint8)
```

`let f::(T@)` therefore creates beneath `T`'s own pure-type place, and
`U::f` / `uint8::f` do not see it. Bare `let f::T` performs no implicit
Symbol-to-type projection and is not this operation. Reads fall back from the carrier's own place to
the Pattern's canonical type object, which is where construction-time and
toolchain-installed type members live, so inherited type members stay
visible through every carrier while a per-carrier member installation stays local.
The carrier that declared the Pattern keeps writing the canonical object,
because construction-time members were installed there before any
rebinding carrier existed. There is no second, place-forwarding declaration
form: every carrier allocates its own place (§2.6), so a per-carrier extension
is always local to that carrier. Where one place must be reached through
another name, the value held is a borrow view. Member creation still requires a
prospective ProjectionSlot plus `let`; later writes require an existing place and
`Writable(place)`. Neither obtains structural `Open` from the view, as specified
in `type-values-places-and-borrow-views.md`.

Exposure of `t::f` composes `Expose(T_t, φ) ∧ Expose(C_f, φ)` at lookup
time, and a deeper path `g::f::T` composes the whole chain
`Expose(T_t, φ) ∧ Expose(C_f, φ) ∧ …` — installation never merges, disjoins,
or writes `P(x)` back into `P(T_t)`. The conjunction is a phase predicate
applied per layer, not a stage-set intersection: a `meta` host legitimately
carries `compile` members, and it is each host's own binding-level view — not
the shared TypeObject adapter and not the Pattern — that decides that layer's
factor. Explicit navigation therefore carries the resolved host chain (each
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
Pattern; source-level `let f::(U@)` against an already installed pure-type
rebinding carrier, navigation through that explicit `type ref` view, and
writability checking of the selected place remain future implementation work.
Bare `let f::U` is not shorthand for obtaining the carrier place.

The two operations may target the same still-open construction, but one source
value is not simultaneously interpreted under both judgments.

#### 12.1.1 Open is construction lineage relative to the compile-time stack

Every constructed PatternValue has a `ConstructionLineage` separate from its
canonical contents and from every place that may carry it:

```text
Open_Γ(v)
  = Open(ConstructionLineage(v), CompileTimeStack_Γ)

ConstructionLineage(v) ∉ Norm(v)
CarrierPlace(v)         ∉ ConstructionLineage(v)
```

Lineage records the construction owner/interval under which the value was
formed and whether that owned line has sealed. Clone, value copy, and compile
transport preserve it; they do not preserve or manufacture source place
identity:

```text
Lineage(Clone(v)) = Lineage(v)
Lineage(let-copy(v)) = Lineage(v)
```

When checking in a meta context, walk down the current compile-time stack while
ignoring `compile` and transparent construction-intrinsic frames. Let `M` be the
first ordinary meta invocation frame found:

```text
Open_Γ(v) <=> DominatedBy(Lineage(v), M) ∧ not Sealed(Lineage(v))
```

In a non-meta context, the corresponding walk follows the stable lexical owner
chain and requires the originating construction interval still to be active and
unfrozen. These are different closing disciplines over the same relation, not
different notions of place capability.

The required independence is explicit:

```text
Writable_Γ(q) does not imply Open_Γ(Read(q))
Open_Γ(v)     does not imply Writable_Γ(Carrier(v))
Γ ⊢ r : type ref does not imply Open_Γ(Read(r))
```

The state transition is one-way:

```text
Open -> Frozen
Frozen -/> Open
```

Nothing reopens frozen material. `extend`/`inject` do not reopen it (§8.2), a borrow
view does not reopen it, and re-navigating to the same object from a new context
does not reopen it.

#### 12.1.2 Freezing events of an ordinary construction

In an **ordinary, non-meta** construction context, the following events freeze the
material being built:

```text
UseForVal1(x)                                    -> Frozen
x used as a meta argument                        -> Frozen
x entering a global normalized structure         -> Frozen
x in Dependencies(c), for NonMetaStaticControl(c) -> Frozen
x in LiveAcross(c), for ResidualRuntimeFork(c)    -> Frozen
leaving the construction interval of the
  in-place closure that owns x                   -> Frozen
```

Observation is not a freezing event: reading `P` or `Val2`, extending a child
pattern, and contributing an ordinary Val2 member of another type all leave the
material open.

For static control, dependency and liveness are different facts:

```text
Dependencies(c) != LiveAcross(c)

NonMetaStaticControl(c)
  => Freeze*(Dependencies(c))
```

`Dependencies(c)` contains the open Pattern values actually read by the
predicate or structural selection, branch/iteration versions whose identity
must be unified at a join, and loop-carried construction state that feeds a
later static decision. A value that is merely live across an unrelated static
branch, join, or loop is not frozen. In contrast, a residual-runtime fork loses
the single known static construction path, so open values carried across that
fork are frozen even when they did not determine its predicate. Leaving the
ordinary owner interval remains an independent closing event.

#### 12.1.3 Meta construction is transparent but meta-local lifetime is not global

The list in §12.1.2 is scoped to ordinary constructions. Inside a meta body the
same events do **not** freeze the material, because the construction anchor is the
meta instance itself (§4.3.1):

```text
inside M:  UseForVal1(x) does not freeze x
           using x as an attempted meta argument does not freeze x
           entering global-normalization machinery does not freeze x
           static control flow does not freeze x
           entering an in-place closure written by M does not freeze x
```

The only construction-closing event for material owned by the meta construction
is its return-stage seal (§4.3.2). A fresh meta-local PatternValue nevertheless
has `Life = MetaInvocation(M)`. Attempting to pass it to another ordinary meta
does not freeze or promote it; candidate formation rejects the call when the
canonical argument is not `GlobalKeyable` (§4.3.1–§4.3.3). `compile` and
transparent construction intrinsics may consume it because they create no new
MetaInstance key.

At seal, only the `OwnedClosure` of the returned Symbol's unique pure role member, if
present, is promoted. Other local PatternValues expire with the invocation. Consequently
`UseForVal1 -> Frozen` must not be read as a universal invariant, while “meta
body is transparent” must not be read as implicit global promotion.

#### 12.1.4 The apparent self-typed intersection

With §12.1.2 in force, the ordinary case that looked like an intersection resolves
without a special rule. Suppose an RHS is a complete `Val1? x P x Val2` whose own
`P x Val2` is the very type being extended, and the extension is attempted from an
ordinary context:

```text
construct RHS value of target type
  -> UseForVal1(target)
  -> target is no longer Open_Γ
  -> attempt to extend target
  -> no applicable overload
```

So in an ordinary context there is no legal situation in which one operation both
extends the target's Pattern and contributes a complete self-typed val to the same
still-open target. A complete value of some *other* type may still be contributed
as ordinary Val2 while the target is open; its own Pattern and Val2 remain
attached to that value.

In a meta body the same sequence is simply legal, because the first step did not
freeze anything.

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

The future Symbol direction is:

```text
pure role member Q:
  install at most once by ordinary definition;
  require Pure(Q);
  derive NamespaceProjection from Q;
  derive TypeProjection only when TypeRole(Q);
  add children only under the owning construction/authority rules

value members V:
  admit multiple heterogeneous value entries;
  form candidates only in a call position;
  do not infer cross-construction-unit merge authority
```

When `struct` establishes a type-role `Q` inside an already resolved owner
pattern scope, an existing incompatible role member is a hard conflict.
Same-origin, same-material cache replay may reuse the existing member.

In particular, an ordinary symbol place receives its pure role member at most once:

```lang
let T = A;
let T = B;
```

If both declarations attempt to install `T`'s role member, the second is a hard
conflict. It is never interpreted as:

```text
A | B
```

Three operations must remain distinct:

```text
first pure-role-member installation
  -> ordinary role-member installation

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
  -> compile: an ordinary PatternValue of its declared result Pattern
  -> meta: an ordinary symbol PatternValue
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

## 13. Current Implementation Substrate

The PR #94 implementation remains a neutral transitional
identity/materialization substrate. It currently provides:

- a doc-hidden explicit context attachment helper for generated type-definition
  pattern heads, retained publicly only for integration-test support;
- categorical `Generated`, `GeneratedTypeDefinition`, `Global`, `Namespace`,
  and `Local` materialization contexts as low-level registry test/materialization
  categories, not final language owner scopes;
- `GeneratedTypeDefinition` as the formal-invocation and binding-time fallback
  for cache-safe anonymous reattachment;
- binding that preserves already attached provisional material and does not
  derive owner identity from the destination global/namespace path;
- registry-backed owner/field `PatternHeadId` allocation and bounded child
  lookup.

This substrate does **not** implement:

- `PatternScopeId` or `ResolvedPatternScope`;
- `MetaInstanceScopeId` or a meta-instance pattern scope such as `(t f)`;
- meta return pure-role self-root validation;
- the canonical meta-invocation navigation atom;
- Symbol `Q` role projection and any corresponding implementation caches;
- the `compile` / `meta` capability split specified here;
- ordinary Symbol as the public meta result (the current
  `SymbolConstruction` carrier is transitional);
- pure value `extend`;
- place-level `inject` with independent `Open_Γ(Read(ref))` and writability;
- contribution-expectation-driven pattern-child versus namespace-value cache
  selection;
- an explicit sum construction/extension API;
- the final owner-resolution rule for `struct`;
- fully named
  `Map<CanonicalFullNavigation, CanonicalPatternValue>` versus ordered
  pattern-layer representation;
- namespace-origin uniqueness or source/meta construction-unit ownership;
- physical-directory contribution authority or cross-file reopening checks;
- the structural `Pure`-implies-`NamespaceRole` and `TypeRole` refinement judgments and their
  implementation enforcement;
- the distinction between ordinary namespace value members and
  pattern-material leaves as implemented facets;
- full place / writability / borrow-view checking;
- graph installation from the construction model in this document.

The categorical global/namespace/local contexts remain available only to the
doc-hidden low-level attachment helper and registry tests. They are not a
stable external owner-construction capability. The ordinary binding path does
not select among them: it preserves attached provisional owner
material, or restores stripped material under the anonymous
`GeneratedTypeDefinition(type_definition_id)` fallback. It must not be
described as determining or rerooting `struct` pattern-owner identity or a meta
return role member's root. In final semantics, the meta instance's own symbol scope
anchors that root.

Formal `struct` invocation currently may allocate or attach registry material
under `GeneratedTypeDefinition`. It remains graph-installation-free and
binding-free. The target operation is referentially pure; this allocation is a
non-semantic implementation record rather than mutation of language-visible
input or installation of a graph delta.

## 14. Non-Goals of This Note

This document does not:

- change the parser, Raw AST, or Normalized AST;
- introduce traditional call syntax;
- implement `extend` or `inject`;
- define a general macro system;
- allow users to define new `BuiltinPrivilegedAstMetaFunction` members;
- expose arbitrary AST or token rewriting;
- implement type checking, name resolution, overload resolution, pattern-space
  execution, extraction execution, D/Done, ownership, runtime evaluation, or
  code generation;
- require the current Rust `SymbolObject`, `PatternHeadId`, or meta invocation
  enums to implement the future objects defined here. PR #94 only neutralizes
  destination-derived owner attachment in the existing substrate.

## 15. Required Direction for Later Implementation

Future implementation should converge in this order:

```text
Symbol Q-role / typed-value-member resolution
  -> PatternValue identity and rank-directed canonical arguments
  -> ordinary Symbol result (current carrier: transitional SymbolConstruction)
  -> ResolvedPatternScope / PatternScopeId / MetaInstanceScopeId
  -> namespace origin and construction-unit ownership
  -> meta return pure-role self-root validation
  -> struct owner resolution independent of binding place
  -> = operator (distinct from let =)
  -> pure child-only extend and read--extend--write inject
  -> explicit sum construction/extension
  -> fully named canonical navigation map / ordered-layer representation
  -> writable let binding and Pattern extension
  -> NamespaceDelta atomic installation
```

### 15.1 Registered implementation debt for `extend` and `inject`

The semantics of `extend` and `inject` are settled by §8; what is missing is implementation.
The ordering dependency is a build-order fact, not a semantic condition:

```text
extend is a pure value function      -- settled (§8.2)
extend needs Open_Γ(old), not a place -- settled (§8.2.2)
inject needs a writable type ref     -- settled (§8.2.3)
inject = read + extend + write       -- settled (§8.2.3)
ordinary `=` is not yet implemented  -- shared implementation debt
```

The consequences are:

- `extend` is implementable and testable without `=`, because it writes nothing;
- `inject` depends on the ordinary write machinery but never on member creation;
- `=` is independently required by several unrelated features — writing an
  existing member, writing an explicit return slot, and updating an ordinary
  value — so that machinery is not an `inject`-specific ontology;
- the current `let`-only substrate is a transitional state. Documentation and
  implementation must not treat `let`-only behavior as the target rule, and must
  not restate the missing `=` as a semantic restriction on `extend`.

Remaining engineering questions in this area are about representation, not about
meaning: the exact ordinary write algebra for the optional Q member and val siblings
(§13), and how ConstructionLineage/stack applicability is tracked efficiently
without entering canonical value identity.

Until those objects exist, the current attachment registry is useful substrate,
but documentation must keep the substrate/final-semantics gap explicit.
