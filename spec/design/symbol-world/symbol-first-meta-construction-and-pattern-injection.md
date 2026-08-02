# Symbol-First Meta Construction and Pattern Injection

**Status: Canonical future-design direction. Not current public language
behavior and not fully implemented.** This document is the canonical design
note for symbol-first resolution, symbol facets, `compile` / `meta` result
boundaries, meta return type self-root identity, resolved pattern scopes,
`struct`, functional `inject`, and the binding/install boundary.

The current implementation is a transitional substrate described in §13. In
particular, the current `PatternHeadId` attachment path must not be read as the
final owner-resolution rule.

This document builds on, without replacing:

- `spec/design/symbol-world/type-values-places-and-borrow-views.md` for
  `SymbolId` / `PlaceId` / `TypeValueId`, the borrow views `ref` / `share` / `@`,
  and writability / extension-eligibility judgments;
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
  `Val1 x Pattern x Val2`, `Pv:Pp`, binding `P1`, result `P2`, compile-flow
  projection, derived compile companions, match staging, and automatic require.

## 1. Canonical Boundaries

The design has five load-bearing boundaries:

```text
name/path resolution:
  path/name -> Symbol -> context-directed facet projection

ordinary value binding:
  let destination = source
    -> resolve source Symbol -> read value -> bind destination Symbol/Place

compile-time value computation:
  compile -> any ordinary PatternValue
    subject to the root-conservation law of §4.2

meta symbol construction:
  meta -> MetaInstance(F, Norm(args)) -> sealed global symbol root

graph mutation:
  let binding/injection -> NamespaceDelta installation
```

Consequences:

1. A name does not initially resolve as a type, value, namespace, function, or
   category. It resolves as a first-class symbol.
2. Ordinary `=` reads a value through the source symbol and binds it to a
   distinct destination symbol/place. It does not forward, reroot, or merge
   identities. There is no declaration form that makes two symbols name one
   place.
3. `compile` computes values. It may accept and return **any** ordinary
   `PatternValue`, including a type value, a symbol value, and a `type ref`.
   What it may not do is create a new global root: it registers no global
   Symbol, produces no nominal type lacking a normal global root, and never
   promotes a local temporary pattern value into a global type (§4.2).
4. `meta` is the only construction that establishes a global symbol root. Entry
   into a meta invocation creates a globally live but unsealed `MetaInstance`
   root; the return stage seals it (§4.3).
5. `struct` and `inject` are pure functions producing uninstalled values.
   Neither operation installs a graph delta and neither mutates its input.
6. A `let` binding or injection path chooses the installation place. It does
   not retroactively choose or reroot the pattern owner carried by the value.

## 2. Symbol-First Resolution and Facets

### 2.1 Conceptual SymbolCell

The specification model is:

```text
SymbolCell {
    SymbolId
    PlaceId

    namespace_facet: optional
    type_facet: optional
    value_facet: zero or more heterogeneous value entries
}
```

This is a semantic model, not a requirement that this PR refactor the current
Rust `SymbolObject` into a structure with these exact fields.

Resolution is always:

```text
path/name
  -> Symbol
  -> context-directed facet projection
```

The following are facet projections:

```lang
symbol |> type
symbol |> val
symbol |> namespace
```

They are not traditional casts or conversions. Projection selects a facet of
the same symbol under the expectation of the use site.

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

### 2.2 Facets may coexist

One symbol may simultaneously provide:

- a namespace facet;
- a type facet;
- an ordinary value;
- a callable value;
- multiple heterogeneous value entries forming an overload candidate set.

The symbol remains one symbol. Facet coexistence does not imply that namespace,
type, and value identity collapse into one identity.

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

Operator-name binding is a separate, narrow mechanism with its own rules and
must not be generalized into an ordinary alias facility. Nothing in this section
licenses reading an operator-name declaration as evidence that ordinary aliases
exist.

## 3. Value Facets and Calls

### 3.1 A value entry is not necessarily a function

The value facet may contain any value:

```lang
let f = expr;
```

If `expr` produces a value, the declaration may contribute a value entry to
the symbol `f`. The entry need not originate from closure syntax and need not
be callable.

Multiple entries under the same symbol may have heterogeneous types. A same-name
value facet is therefore not equivalent to a traditional same-signature
function-overload bucket.

### 3.2 Call candidate preparation

A call position performs the following conceptual flow:

```text
resolve symbol
  -> project value facet
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

Orthogonal to all three dimensions, a callable's complete semantics is the
product of independent coordinates — there is no separate `CallableKind`
axis that decides return ontology:

```text
CallableSemantics
    = P1 × P2 × ReturnShape × Privilege

ReturnShape ::= Unit | SingleType | SingleVal | SymbolValue

Privilege   ::= Ordinary | BuiltinPrivileged   -- bounded AST access
```

The declared `ReturnShape` decides the shape of the returned value. The one
authorization law is about *rooting*, not about shape:

```text
SealsAMetaInstanceRoot(F) => P2(F) = meta

P2(F) = meta  !=>  F returns a SymbolValue
```

Only a pure meta result P2 may seal a `MetaInstance` and thereby install a new
global symbol root. The converse never holds: a meta callable may legally return
`SingleType`, `SingleVal`, or `Unit`, and every `ReturnShape` — including
`SymbolValue` — is legal under `compile`, because returning a symbol *value* is
not the same act as rooting a new global Symbol (§4.2).

An implementation must not derive return ontology from whether a result policy
pair contains the `Meta` stage, and must not maintain a callable-kind table as a
second ontology authority parallel to `ReturnShape`. Where prose still
distinguishes "meta functions" from ordinary functions, the distinction means
exactly "is this callable authorized to seal a new global root", never a
hardwired return shape. Policy stages (`meta` / `compile` / `seal` / `runtime`
in `P1` / `P2`) decide only visibility and execution timing.

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

The returned view is subject to the ordinary formation condition of
[`type-values-places-and-borrow-views.md`](type-values-places-and-borrow-views.md)
§5.5, evaluated at the receiving position. If the return stays inside the same
Open window, `out : type ref` holds and the call is well-formed. If it crosses
the closing boundary, the failure is not "a later `inject` was rejected" but

```text
Γ_receiver ⊬ out : type ref
```

— the returned value cannot satisfy the formation/survival condition of its own
view type. That rejection belongs to the escape check of
[`../lifetime/lifetime-policy-and-overload-boundary.md`](../lifetime/lifetime-policy-and-overload-boundary.md)
§3, or the body must weaken before returning:

```lang
r share;
```

#### 4.2.2 The two contexts of a compile evaluation

A `compile` evaluation reads two independent contexts:

```text
EvalCompile(F, args; ConstructionContext_caller)

DefinitionLexicalContext(F)
  — local Self space, anonymous closure type ownership,
    lexically declared identity

CallerConstructionContext
  — the ambient Open facts that a by-value argument does not carry itself
```

The definition context decides what names and owners the body sees. The caller
context supplies only the one thing a by-value pattern argument cannot supply:
whether its construction root is still `Open` (§8.2.3). Neither substitutes for
the other: a body whose definition context is intact can still be unable to
`inject` a by-value `type` because the caller's construction context has frozen
that target.

The sensitivity is narrow. It arises only from Open-sensitive operations on
by-value `type` arguments:

```lang
let extend =
    (self, t: type): compile -> out: type => {
        t |> inject(...);
    };
```

`t` carries no Open witness, so the body's legality is a property of the call
site, recordable as a function summary and verified there:

```text
Requires(extend) = Open( ConstructionRoot(t) )
Γ_caller ⊨ Open( ConstructionRoot(t) )
```

A `type ref` parameter proves the same fact by itself:

```lang
let extend_ref =
    (self, t: type ref): compile -> out: type => {
        t |> inject(...);
    };
```

```text
t : type ref  =>  Open_Γ( Target(t) )
RequiresAmbientOpen(extend_ref) = ∅
```

Such a body consumes only the capability its argument carries; `inject` never
asks which in-place closure the `compile` call came from. Hence:

```text
a compile evaluation depends on the caller's Open window
  only through Open-sensitive operations on by-value `type` arguments
```

not as a general property of every `compile` call.

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
type facet of its local `Self` to a named receiver such as `ref::T`.

Nested paths print in source order, current/innermost callable-local `Self`
first and outermost `Self` last, but identity is the parent-linked owner graph.
No `__inner_space` or `__inner_namespace` node participates in canonical
ownership. This owner is not a meta-instance owner such as
`MetaInstanceOwner(meta_function, canonical_arguments)`.

### 4.3 Ordinary `meta`

`meta` is symbol-level staging. A meta invocation is the only construction that
establishes a new global symbol root.

```text
M = MetaInstance(F, Norm(args))
```

Entering the invocation immediately creates `M` as a **globally live but unsealed
root**. `M` exists in the global world from the first statement of the body; what
the return stage adds is not existence but *seal*.

When the declared `ReturnShape` is `SymbolValue`, the returned value is the
symbol value of `M`:

```text
meta (ReturnShape = SymbolValue):
  accepted parameters
  -> the symbol value of M
```

A meta callable may accept a `symbol` parameter, or constrain a parameter to a
narrower `type` or ordinary pattern-value rank. That does not change its declared
return shape. A meta callable whose `ReturnShape` is `SingleType`, `SingleVal`,
or `Unit` returns exactly that shape — `meta` staging never coerces the returned
value into a symbol (§4.1).

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

#### 4.3.2 Seal happens only at the return stage

The only construction-closing event of a meta invocation is its final return
stage, and it runs in a fixed order. The returned symbol has the ordinary Symbol
shape — at most one type member, any number of val members:

```text
Σ = ⟨ T?, V ⟩

1. validate that there is at most one type member
2. if T exists, promote OwnedClosure(T) into M
3. validate the remaining returned members
4. seal M
```

Step 1 is a cardinality bound, **not** a requirement that a type member be
present. Nothing in the Symbol ontology or in the self-root constraint promotes
`|T| <= 1` to `|T| = 1`: the self-root rule says that *if* a type facet exists it
must be rooted at `M`, which is vacuous when there is none. So a namespace-only,
val-only, or type-less mixed return is well-formed, and step 2 is simply skipped:

```text
T absent  ->  step 2 is a no-op; steps 3 and 4 proceed unchanged
```

Step 2 promotes the **owned** closure only. Horizontal borrow edges are not
ownership and are never dragged into the promotion:

```text
OwnedClosure(x) excludes every ref / share edge reachable from x
```

A member reachable only through a borrow view is therefore not promoted, and its
presence does not extend `M`'s owned material. After step 4, `M` is sealed and
nothing may reopen it.

#### 4.3.3 `M` as a navigable layer

Every ordinary canonical meta-function invocation establishes a virtual
symbol-construction scope:

```text
M = MetaInstanceScope(callee_symbol, canonical_arguments)
```

For:

```lang
let f = (self, t: type): meta -> r: symbol => { ... };
```

the diagnostic navigation projection of `M` is:

```text
(t f)
```

This is not merely a folder analogy. `M` is a symbol/namespace layer that
participates in default pattern navigation and name shadowing, may carry
namespace, type, and value facets, anchors cache/incremental identity, and owns
the return construction transaction. An ordinary meta invocation must therefore
establish its own symbol layer rather than act as a value-level forwarding
function.

The externally navigable result symbol is `M` itself. The declared return slot
is only a lexical construction handle to that symbol:

```text
symbol_of_result(invoke_meta(callee, canonical_arguments)) = M
return_slot(r) = lexical_handle(M)
```

The slot name `r` does not add another component to the final navigation path.
Material written through `r` contributes facets or children to `M`; it does not
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

### 4.4 Ordinary meta return type self-root invariant

If the return symbol of an ordinary canonical meta invocation has a type facet,
the outermost pattern root of that facet must be the invocation's own `M`:

```text
type_facet(r) = tau
  => root_pattern_scope(tau) = M
```

This is identity equality between a pattern root and the meta-instance symbol
scope. It is not equality of rendered strings. The root identity is:

```text
MetaTypeRoot = MetaFunctionIdentity + Normalize(Arguments)
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
Neither value may directly replace the return symbol's required type root.
The failure is the hard diagnostic `MetaReturnTypeRootMismatch`. An
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

External `PatternValue`s may be members of the self-rooted type; they may not
replace the root. For example:

```lang
let fn = (self, t: type): meta -> r: symbol => {
    let t1::r = bool;
    r;
};
```

keeps `(t fn)` as the return symbol's root and includes the externally owned
`bool::` value as a member beneath that root. It must not be summarized as
`type_facet(r) = bool::`.

The self-root check is conditional on a type facet. A return symbol with only a
namespace facet, ordinary value facet, or both does not acquire a synthetic
type facet merely to satisfy this rule.

### 4.5 Formal return material

The return slot name `r` denotes the open symbol under construction. It shares
one substrate with namespace-level same-name synthesis: accumulating members on
`r` inside a meta body and accumulating same-named contributions under a
namespace are the same mechanism — member events on one open Symbol.

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
                         declaration context (let r may then shadow the
                         return slot)
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
  replaces type facets / val siblings by RHS shape is registered
  implementation debt in §13.
- `r;` is the TailValue terminal. It delivers the constructed symbol to the
  directly enclosing layer. It is not a member contribution, and a meta body
  with member events but no terminal does not implicitly deliver anything.

The terminal family follows the general control-flow end model: `expr;`
delivers to the directly enclosing layer, `expr return;` returns to the
outermost function layer, and `expr (T return);` returns to the layer selected
by the function-object type `T`.

Add-fresh-member and write-to-existing-target are two distinct construction
effects. They must not be collapsed into one injection event, and neither is a
return. Whether contributed material references an existing `PatternValue`,
computes new material, or projects a symbol facet is represented inside the
construction value; any resulting type facet must pass the self-root invariant in
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

Assignment checks exactly three conditions:

```text
the left side names a writable place
the right value conforms to the target's Pattern
lifetime and capability conditions of the target place hold
```

It does not inspect how the right value was produced. Assignment requires no
provenance, no construction witness, and no transition proof from any particular
producer; a value that conforms to the target Pattern is acceptable regardless of
which operation built it.

This distinction does not cancel `let f::t = expr` (ordinary `Val2` member
contribution) and does not change the `r;` terminal semantics. The current
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
    assigned_facets_or_values,
    optional_child_contributions,
    provenance,
}

assigned non-root value/member may equal an already existing PatternValue
```

Value equality remains independent of source name and navigation path and does
not merge symbol or place identity. However, that general identity separation
does not waive the type self-root invariant: `r = uint8` as a direct meta return
type installation is rejected after symbol resolution/value read, rather than
being reinterpreted as forwarding or accepted as an identity meta type.

### 4.7 A symbol is an ordinary PatternValue

A symbol value is not a separate ontological rank. It is an object with the same
three components as every other object:

```text
SymbolValue = ⟨ MemberContainer, P_symbol, Val2_symbol ⟩
```

Its member content is ordinary object content:

```text
at most one type member
any number of val members
```

Because the member set is the mutable part, it lives in `Val1`:

```text
Val1(Symbol) = Member * omega
```

The consequence is that symbol-level operations are `Val1` transformations and
leave the symbol's own pattern untouched:

```text
s = new_symbol                   -> replaces Val1(s)
s += contribution                -> extends Val1(s)
s -= associated_type_family       -> removes members from Val1(s)

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
    result_rank,
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
remain graph-installation-free and binding-free;
return SymbolConstructionValue or an owned construction handle;
leave graph installation to an outer binding.
```

Unlike an `OrdinaryMetaFunction`, an individual built-in may define a special
scope/owner rule and need not create an independently navigable
`MetaInstanceScope M`. Users may call compiler-provided members but cannot
define new privileged AST meta functions. Privilege is member-specific: one
built-in's accepted carrier and bounded transformation do not imply a general
macro system or arbitrary AST rewriting.

`struct` and `inject` are the first specified members. Future candidates may
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
open a namespace facet
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
  PatternSyntax / normalized pattern material
  -> SymbolConstructionValue : symbol
```

An implementation may carry AST or Normalized AST as a private structured
carrier. The public result rank is not AST and this capability does not expose a
general macro system.

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

left-side let binding/injection path:
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
  -> obtain an uninstalled SymbolConstructionValue with an already resolved owner
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
```

It does not install a `NamespaceDelta`. The current implementation may allocate
or attach registry-backed pattern material while forming the invocation value;
that allocation means the invocation must not be described unconditionally as
pure. Graph installation remains outside formal invocation.

### 7.2 Structural leaves and pure Pattern nodes

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

### 7.3 Internal construction and later injection normalize equally

An element written inside the original `struct` input and an equal element
added later through the owner's navigated injection path differ only in how
their full navigation is obtained. For example:

```lang
let t = ((bool inner)t) |> struct;
```

and the construction sequence using `inject` (privileged structural
registration):

```lang
let t = (()t) |> struct;
let t_ref = t@;
t_ref = t_ref |> inject(bool inner);
```

produce the same normalized PatternValue, provided both operations are under
the same still-open construction authority:

```text
NamedPattern(
  name = t,
  child = inner::t -> Norm(bool::)
)
```

The first form inherits/completes `inner` under `t`; the second supplies the
same complete navigation through the `inject` privileged operation, installed
by the ordinary `=` overwrite of the carrier slot reached by `t@` — a pure
pattern slot is writable only through that `type ref` (§8.2.2). After
completion, normalization retains only the complete navigation and normalized
resident value. It erases whether the element was internal or injected, and
whether its navigation was inherited or explicit.

> **Correction:** Ordinary navigated
> `let inner::t = bool::;` does **not** produce the same PatternValue.
> It only installs `bool::` as an associated type (Val2 member) named
> `inner` under `t`'s scope, without registering `inner` into `t`'s
> Pattern canonical structure. Registering a member into the Pattern
> structure is a privilege held exclusively by `struct` inline construction
> and the `inject` built-in meta function. See §12.1 for the full
> privilege boundary.

## 8. `inject`

### 8.1 Privileged built-in

`inject` is a future `BuiltinPrivilegedAstMetaFunction`, parallel to `struct` in
trust boundary. It does not create an ordinary externally navigable
`MetaInstanceScope M`:

- it accepts normalized pattern syntax or an equivalent internal AST carrier;
- its public successful return rank is `type`;
- it does not re-enter the parser;
- it does not concatenate arbitrary tokens;
- it does not expose unrestricted AST-consuming capability to user functions;
- it performs only bounded pattern-child construction.

The source examples in this section are semantic sketches. They do not change
the frozen parser or introduce traditional `f(args)` call syntax.

### 8.2 `inject` is a pure function on type values

`inject` takes an ordinary pattern value and child pattern material, and returns
a new pattern value:

```text
inject : ( type | type ref ) x ChildPatternMaterial ⇀ type

Inject(old, Δ) ⇓ new
```

There is no construction-handle rank. The input is an ordinary value of rank
`type`, or a `type ref` view of a place holding one. Nothing about `inject`
requires a capability-bearing carrier, an uninstalled construction object, or a
privileged handle type.

The function is total in its effects in the following sense:

```text
Inject does not modify old
Inject does not install a namespace delta
Inject does not perform an assignment
```

`old` is an input value and is left exactly as it was. `new` is a distinct
resulting value. Discarding `new` produces no symbol-world side effect, because
there was never a side effect to discard.

#### 8.2.1 Failure is total

```text
failure => no partial result, no write, no rollback
```

Because `inject` writes nothing, a failed `inject` has nothing to undo. There is
no half-extended pattern, no compensating action, and no rollback protocol. A
failed call simply produces no value.

#### 8.2.2 Observing the result requires a separate write through a `type ref`

When the caller wants the extension to become the content of an existing place,
the write is a separate, ordinary step. Through a `type ref` the full sequence is
three distinct operations:

```text
old = Read(t_ref)
new = Inject(old, Δ)
Write(t_ref, new)
```

`Read` and `Write` are the ordinary borrow-view operations of
`type-values-places-and-borrow-views.md`. `inject` occupies only the middle step.

The left side of that write must be a `type ref`. This is not a stylistic
preference; it is the same rule that makes `@` necessary in the first place. For a
pure pattern slot — one with no `Val1` payload — an ordinary use of the name reads
out a `P x Val2` value and does **not** reach the carrier slot, so it cannot be
the target of a write:

```lang
let t_ref = t@;                            // the carrier slot, as a type ref
t_ref = t |> inject(bool inner);           // write back through the ref
```

or, reading and writing through the same view:

```lang
t_ref = t_ref |> inject(bool inner);
```

The **input** side is the permissive one — it accepts either rank:

```text
right side of the write  :  type | type ref
left  side of the write  :  type ref
```

What is excluded is the shorthand `t = t |> inject(...)` on a pure pattern slot.
The language cannot hold both of these at once:

```text
an ordinary read of t does not reach the slot, which is why t@ exists
t = ... nevertheless writes that slot
```

So the shorthand is not a convenience spelling for the three-step sequence; on a
pure pattern slot it has no writable left side at all. `t = ...` remains an
ordinary write wherever `t` genuinely names a writable place on its own.

The write itself is still the ordinary `=` write governed by §4.5.1 — the left
side must name a writable place, the right value must conform to the target's
Pattern, and lifetime/capability conditions of the target place must hold.
Assignment does not inspect how the right value was produced, so it asks for no
construction witness and no proof that the value came from `inject`.

A navigated `let child::target = result;` is **not** a structural installer:
like every ordinary navigated `let`, it only installs a Val2 associated
member under `target` and never writes the construction back into a Pattern
canonical structure (see the §7.3 correction).

#### 8.2.3 Input validity: two overloads, two places the Open fact comes from

`inject` has two input overloads. They do not differ in how much permission they
demand; they differ in **where** the openness fact is obtained:

```text
Injectable_Γ(x : type)        = Open_Γ( ConstructionRoot(x) )
Injectable_Γ(x : type ref)    = true
Injectable_Γ(x : type share)  = false
```

**By-value `type` — ask the evaluation context.**

```text
Γ ⊢ t : type      Open_Γ( ConstructionRoot(t) )
------------------------------------------------
Γ ⊢ t |> inject(Δ) : type
```

A plain `type` value carries no construction capability. The same pure pattern
value may sit inside an open window in one context and be a frozen but globally
live value in another, and nothing in the value itself distinguishes the two.
This overload therefore has to consult the current evaluation context.

**`type ref` — the argument already proved it.**

```text
Γ ⊢ r : type ref
---------------------------
Γ ⊢ r |> inject(Δ) : type
```

The `true` above is not an exemption. It is discharged by the formation invariant
of the view (`type-values-places-and-borrow-views.md` §5.5):

```text
Γ ⊢ r : type ref   =>   Open_Γ( Target(r) )
```

Re-querying the environment would re-ask a question the premise has already
answered. Evaluation stays purely functional:

```text
r |> inject(Δ)  =  Inject( Read(Target(r)), Δ )
```

It returns an ordinary `type` and does not modify the target slot. The write-back
remains the separate three-step sequence of §8.2.2.

A `type share` view is not injectable input: `share` carries no write path and no
extension eligibility, so a call attempting it has **no applicable overload**
rather than a special "cannot inject through share" error.

`Open_Γ` is the ordinary construction-anchor condition written in context form,
not a separate inject-only permission system:

```text
EffectiveOpen(x, c) = StateOpen(x)
                    ∧ ConstructionAnchorCompatible(owner(x), c)
```

This is why an already-sealed root cannot be extended: not because `inject`
checks handle ownership, but because a sealed object is not `StateOpen`, so no
valid `type ref` to it can be formed and the by-value overload finds no ambient
`Open` fact either.

### 8.3 Navigation direction

The distinction between `struct` and `inject` is navigation direction, not
ownership authority:

```text
struct:  resolves OUTWARD
  resolve owner by ordinary input navigation + ambient scope
  (always looks up for the top-pattern navigation name)

inject:  resolves INWARD
  takes the input pattern value as the navigation anchor;
  children inherit that pattern's path
  (never looks outward for a top-level scope)
```

This is the whole reason `inject` needs an existing pattern value as input: it
needs a pattern whose navigation path is already resolved, so that the new
children can be linked beneath that path.

Example:

```lang
t1::r = t1::r
    |> inject(t first)
    |> inject(u second);
```

Each `inject` reads a value and produces a value; the single `=` at the end
performs the one write. The resulting pattern is:

```text
(
    t first::t1::r,
    u second::t1::r
)::t1::r
```

`inject` determines the child set of the resulting pattern value. It does not
change owner identity, and it does not reopen anything: whether `t1::r` may be
written at all is decided by §8.2.2 and §8.2.3, before `inject` is reached.

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

`inject` extends the input pattern by **direct children only**:

```text
Inject(old, Δ) may add children directly beneath P(old)
Inject(old, Δ) may not reach into a grandchild layer
```

Extending a deeper layer is expressed by composing the operation at that layer —
read the child value, inject into it, and write it back — not by giving `inject`
a deep path.

Within that scope, `inject`:

- adds direct children to the resulting pattern;
- preserves the owner identity carried by the input pattern.

It does not:

- replace the owner;
- overwrite an existing type facet;
- delete an existing child;
- implicitly reroot an arbitrary external pattern value;
- mutate the input value or the installed namespace graph;
- extend a value that is not `EffectiveOpen` in the calling context;
- grant a general macro or arbitrary AST-rewrite capability.

The last two entries are not extra guards inside `inject`. Failing them means
there is **no applicable overload** — the call is rejected by ordinary
input-validity and write checking (§8.2.3), in the same way any other call with
unsuitable arguments is rejected.

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
different-name injections commute;
same-navigation/different-value conflicts are rejected before map formation.
```

For example:

```lang
t1::r
|> inject(t first)
|> inject(u second)
```

and:

```lang
t1::r
|> inject(u second)
|> inject(t first)
```

produce the same pattern value because both direct children have top-pattern
names.

Once normalized, the map does not classify elements as “internal patterns” or
“external patterns.” Parent-scope inheritance, explicit `::`, ordinary symbol
binding, and `inject` explain how a `PatternValue` was resolved or produced
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

“Inject once” applies to a complete child navigation path, not to the owner as a
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
|> inject(t first)
|> inject(u second)
```

is valid, while:

```lang
|> inject(t first)
|> inject(u first)
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
callables from the heterogeneous value facet.

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
    PatternChild           (PRIVILEGED: struct inline / inject only)
  | NamespaceValueMember   (ordinary navigated let)
```

> **Privilege boundary:**
>
> `PatternChild` is a **privileged** expectation available only to:
> - `struct` inline construction (elements in the struct body)
> - `inject` built-in meta function (future)
>
> Ordinary navigated `let f::t = expr` is **always** interpreted under
> `NamespaceValueMember`, regardless of whether `expr` is `null × P × Val2`
> (a pure type object) or `Val1 × P × Val2` (a complete value). The
> expectation is never guessed from the RHS shape.
>
> ```text
> ordinary navigated let  -> NamespaceValueMember (always)
> struct inline / inject  -> PatternChild (privileged)
> ```

Under `PatternChild`, the source path is resolved to a symbol and projected to
its type/pattern value. The resulting `PatternValue` is installed as a child of
the owner's type construction and participates in normalization and extraction:

```text
resolve source Symbol
  -> project/read PatternValue
  -> contribute to owner TypeFacet(PatternValue)
```

This expectation is exercised by `struct` inline construction elements and
(future) `inject`.  It requires the enclosing construction to be Open and
owned by the current authority.

Under `NamespaceValueMember`, the source is projected through its ordinary
value facet and a namespace value symbol is constructed. This changes only the
namespace graph/value facet; it does not enter or change the owner's
`PatternValue`:

```text
resolve source Symbol
  -> project/read value (including pure type objects)
  -> install as associated Val2 member
  -> does NOT modify target Pattern canonical structure
```

This is the expectation of:
- Ordinary navigated `let f::t = expr` (always, regardless of RHS shape)
- An ordinary let-shaped declaration consumed inside `struct` construction:

```lang
let name = expr
```

It contributes one associated member to the current Pattern owner's
`Val2`/namespace value facet:

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
struct / inject -> can produce the third and fourth, with privilege
```

This distinction supersedes the previous text which described Pattern-value
injection as a possible outcome of `let f::t = expr`:

```text
Privileged structural registration (struct inline / inject ONLY):
  null × P × Val2
  -> registers pure Pattern material into target P canonical structure
  -> the member becomes a structural child with extraction/construction capability

Ordinary Val2 installation (let f::t = expr, always):
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

Resolving `let f::t = x` selects the target cluster's unique type member
`T_t`, obtains *that member object's own* `ObjectPlace`, interns the
associated Symbol `C_f` under the name `f` in that place, and installs `x`
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

`let f::T` therefore writes `T`'s own pure-type object, and `U::f` /
`uint8::f` do not see it. Reads fall back from the carrier's own place to
the Pattern's canonical type object, which is where construction-time and
toolchain-installed type members live, so inherited type members stay
visible through every carrier while a per-carrier injection stays local.
The carrier that declared the Pattern keeps writing the canonical object,
because construction-time members were installed there before any
rebinding carrier existed. There is no second, place-forwarding declaration
form: every carrier allocates its own place (§2.6), so a per-carrier extension
is always local to that carrier. Where one place must be reached through
another name, the value held is a borrow view, and the extension follows the
view's own eligibility — `Eligible(view of p) ≤ Eligible(p)`, as specified in
`type-values-places-and-borrow-views.md`.

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
target cluster's member ledger, the type member's own Policy, the derived
cluster Policy, the Pattern canonical norm, and the Val2 of the cluster's
same-named ordinary value members. Navigation and invocation always take
the Symbol route:

```text
target Symbol -> unique type member -> its Val2 Symbol -> facet projection
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
Pattern; source-level `let f::U` against an already installed rebinding
carrier, extension through a `type ref` view, and writability checking of the
selected place remain future work.

The two operations may target the same still-open construction, but one source
value is not simultaneously interpreted under both judgments.

#### 12.1.1 Open is relative to the construction context

Openness is not a global flag on the object. It is a judgment about an object in
a context:

```text
EffectiveOpen(x, c) = StateOpen(x)
                    ∧ ConstructionAnchorCompatible(owner(x), c)
```

`StateOpen(x)` is the object's own construction state. `ConstructionAnchorCompatible`
asks whether the construction that owns `x` is the construction currently being
evaluated in `c`. Both factors are required: material may be structurally unsealed
and still not be extendable from an unrelated context, and a context may own a
construction whose material has already been frozen.

The state transition is one-way:

```text
Open -> Frozen
Frozen -/> Open
```

Nothing reopens frozen material. `inject` does not reopen it (§8.2.3), a borrow
view does not reopen it, and re-navigating to the same object from a new context
does not reopen it.

#### 12.1.2 Freezing events of an ordinary construction

In an **ordinary, non-meta** construction context, the following events freeze the
material being built:

```text
UseForVal1(x)                                  -> Frozen
x used as a meta argument                      -> Frozen
x entering a global normalized structure       -> Frozen
any non-meta control-flow branch / join /
  loop boundary crossed by x                   -> Frozen
leaving the construction interval of the
  in-place closure that owns x                 -> Frozen
```

Observation is not a freezing event: reading `P` or `Val2`, extending a child
pattern, and contributing an ordinary Val2 member of another type all leave the
material open.

The control-flow entries are the reason an ordinary construction is narrow: an
ordinary context has no static guarantee about which branch executed, so material
that crosses a branch, join, or loop edge can no longer be treated as a single
known construction and is frozen at that edge.

#### 12.1.3 A meta body freezes none of this

The list in §12.1.2 is scoped to ordinary constructions. Inside a meta body the
same events do **not** freeze the material, because the construction anchor is the
meta instance itself (§4.3.1):

```text
inside M:  UseForVal1(x) does not freeze x
           static control flow does not freeze x
           entering an in-place closure written by M does not freeze x
```

The only construction-closing event of a meta invocation is its return-stage seal
(§4.3.2). Consequently `UseForVal1 -> Frozen` must not be read as a universal
invariant of the construction model, and must not be applied to a meta body to
justify rejecting a meta-local computation.

#### 12.1.4 The apparent self-typed intersection

With §12.1.2 in force, the ordinary case that looked like an intersection resolves
without a special rule. Suppose an RHS is a complete `Val1 x P x Val2` whose own
`P x Val2` is the very type being extended, and the extension is attempted from an
ordinary context:

```text
construct RHS value of target type
  -> UseForVal1(target)
  -> target is no longer EffectiveOpen
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
value-member name. Inside construction of `T`, `let () = impl` contributes
`()` below `T` only. A separate `()::ref::T` or `()::share::T` requires a
separate authorized contribution. The body of an associated `()` entry still
has its own `CallableOwner`, while invocation-frame slot 0 receives the object
whose type supplied that entry.

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

### 12.2 Same-symbol facet rules

The future symbol-facet direction is:

```text
namespace facet:
  establish the facet from exactly one NamespaceOrigin;
  add children only under the owning construction/authority rules

type facet:
  install once by ordinary definition

value facet:
  admit multiple heterogeneous value entries;
  form candidates only in a call position;
  do not infer cross-construction-unit merge authority
```

When `struct` establishes a type/pattern facet inside an already resolved owner
pattern scope, an existing incompatible facet is a hard conflict. Same-origin,
same-material cache replay may reuse the existing facet.

In particular, an ordinary symbol place receives its type facet at most once:

```lang
let T = A;
let T = B;
```

If both declarations attempt to install `T`'s type facet, the second is a hard
conflict. It is never interpreted as:

```text
A | B
```

Three operations must remain distinct:

```text
first type-facet installation
  -> ordinary type installation

add a direct child under an owned, still-open construction
  -> inject or another explicit child-construction API

construct or extend a sum
  -> explicit sum-construction / sum-extension API
```

The final spelling of the sum API remains open. Duplicate ordinary definitions
do not provide that API, and `inject` must not convert an existing type or an
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
  -> PatternValue or SymbolConstructionValue
  -> for a source path: resolve Symbol -> read its value/facets
  -> let binding/injection judgment binds that value/construction
  -> resolve writable install PlaceId
  -> form NamespaceDelta
  -> validate facet/child conflicts
  -> install atomically or install nothing
```

Neither `struct` nor functional `inject` directly mutates the namespace graph.
Graph installation always occurs in the outer declaration/binding layer.

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
- meta return type self-root validation;
- the canonical meta-invocation navigation atom;
- `SymbolCell` facets;
- the `compile` / `meta` capability split specified here;
- `SymbolConstruction` as the public meta result model;
- functional `inject`;
- pure-functional `inject` input validity based on `EffectiveOpen`;
- contribution-expectation-driven pattern-child versus namespace-value facet
  selection;
- an explicit sum construction/extension API;
- the final owner-resolution rule for `struct`;
- fully named
  `Map<CanonicalFullNavigation, CanonicalPatternValue>` versus ordered
  pattern-layer representation;
- namespace-origin uniqueness or source/meta construction-unit ownership;
- physical-directory contribution authority or cross-file reopening checks;
- the structural `TypeFacet`-implies-`NamespaceFacet` model;
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
return type's root. In final semantics, the meta instance's own symbol scope
anchors that root.

Formal `struct` invocation currently may allocate or attach registry material
under `GeneratedTypeDefinition`. It remains graph-installation-free and
binding-free, but it is not unconditionally pure.

## 14. Non-Goals of This Note

This document does not:

- change the parser, Raw AST, or Normalized AST;
- introduce traditional call syntax;
- implement `inject`;
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
SymbolCell / facet-aware resolution
  -> PatternValue identity and rank-directed canonical arguments
  -> SymbolConstruction
  -> ResolvedPatternScope / PatternScopeId / MetaInstanceScopeId
  -> namespace origin and construction-unit ownership
  -> meta return type self-root validation
  -> struct owner resolution independent of binding place
  -> = operator (distinct from let =)
  -> functional child-only inject (depends on = operator)
  -> explicit sum construction/extension
  -> fully named canonical navigation map / ordered-layer representation
  -> writable let binding and Pattern extension
  -> NamespaceDelta atomic installation
```

### 15.1 Registered implementation debt for `inject`

The semantics of `inject` are settled by §8; what is missing is implementation.
The ordering dependency is a build-order fact, not a semantic condition:

```text
inject is a pure function            -- settled (§8.2)
inject needs no write capability     -- settled (§8.2)
observing its result needs `=`       -- ordinary write, specified in §4.5.1
`=` is not yet implemented           -- implementation debt
```

The consequences are:

- `inject` is implementable and testable without `=`, because it writes nothing;
  a call whose result is only read back is complete behavior, not a partial one;
- `=` is independently required by several unrelated features — writing an
  existing member, writing an explicit return slot, and updating an ordinary
  value — so it is not an `inject`-specific prerequisite;
- the current `let`-only substrate is a transitional state. Documentation and
  implementation must not treat `let`-only behavior as the target rule, and must
  not restate the missing `=` as a semantic restriction on `inject`.

Remaining engineering questions in this area are about representation, not about
meaning: how a real `=` adds or replaces type facets and val siblings by RHS
shape (§13), and how `EffectiveOpen` is tracked efficiently across nested
construction anchors.

Until those objects exist, the current attachment registry is useful substrate,
but documentation must keep the substrate/final-semantics gap explicit.
