# Type Values, Places, and Alias Forwarding

**Status: Non-normative design note, partially realized. The identity core (recursive type-object normal form, canonical observation `Addr(Norm_type)`, per-carrier `Val2` places) is current `lang_build` behavior; alias forwarding, writable-place checking, injection-place checking, and type checker behavior remain future design.**

This document specifies the future semantic boundary between *type values*,
*symbol identity*, *writable places*, *alias forwarding*, and *namespace
injection targets*. It is a design note, not a parser or normalizer rule, and
not current public language behavior; §10 records which parts are already
current `lang_build` behavior and which remain future design.

The document is self-contained. It does not require the reader to assemble its
meaning from `type-associated-function-objects-and-access-trees.md`,
`early-meta-functions-and-namespace-graph.md`, or `entity-alias-design.md`. Those
documents are background or adjacent design only; the model here stands on its
own and is the canonical authority for the type-value / place / symbol / alias
distinction.

The broader symbol-first facet, `PatternValue`, `compile` / `meta`, pattern
scope, `struct`, and functional `inject` model is canonicalized in
`spec/design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md`.
That document composes with this identity/alias model rather than replacing it.

## 1. Purpose

The language must distinguish three things that look similar in source text but
are semantically different: the identity of a name, the identity of a writable
location, and the identity of a type value. Conflating them produces subtle
errors — for example, injecting a declaration into a built-in type because its
type value happens to equal a freshly bound symbol's type value.

The invariants this document protects:

```text
type-value equality must not collapse symbol-place identity
alias forwarding must not silently create writable places
namespace injection must target writable places, not values
```

This document does **not** define:

- a full type checker,
- a full alias implementation,
- access-tree construction,
- a full lifetime checker,
- package import / export,
- runtime lookup.

Two phrasings are explicitly rejected throughout. `let T: type = uint8` is
**not** fresh nominal type generation. Alias forwarding is **not** textual
substitution. And type-value equality is **not** writable-place equality.

## 2. Semantic identities

Three distinct symbol/type identities participate in this model, alongside
canonical pattern-value identity:

```text
SymbolId
PlaceId
TypeValueId
PatternValue identity
```

- `SymbolId` is the identity of a symbol object in the name graph.
- `PlaceId` is the identity of a location that can be bound, updated, injected
  into, or opened for a namespace delta.
- `TypeValueId` is the stable first-order type root — a registry projection
  of a type value, not the full type-object semantic identity. The complete
  identity of a type object at an observation moment is the canonical
  observation defined in §2.1:

  ```text
  TypeObservation(x, p) = Addr(Norm_type(x, p))
  ```

  One `TypeValueId` observed under different `Val2` states is two distinct
  type observations, so bare `TypeValueId` comparison is legitimate only as a
  first-order projection check — never as canonical type-value equality, and
  never as the identity consumed by pattern/overload matching, field
  signatures, or canonical argument keys.
- `PatternValue identity` is the canonical identity of any compile-time pattern
  value, including ordinary compile-time values, type values, and structured
  pattern values. The type-rank projection consumed when a parameter or
  expectation has `type` rank is the canonical observation of the evaluated
  type object (`Addr(Norm_type)`); `TypeValueId` is only its first-order root
  component.

These identities are independent. None implies another:

```text
SymbolId equality does not imply TypeValueId equality.
TypeValueId equality does not imply PlaceId equality.
PatternValue equality does not imply SymbolId or PlaceId equality.
Alias forwarding may relate symbols, values, and places, but does not erase the distinction.
```

A type expression cares about the *value*. A namespace injection target or a
declaration-extension site cares about the *place*. Alias forwarding relates a
*symbol* to a forwarding chain. The three concerns must not be folded into one
another.

In the symbol-first model, a path initially resolves to one symbol cell and the
use site then projects namespace, type, or heterogeneous value facets. Facet
projection does not collapse these identities and is not a cast.

### 2.1 Type-object identity is the recursive object normal form

A type object is an object like any other: `null × P × Val2`. Its canonical
identity therefore has to carry both live components, not the Pattern alone:

```text
Norm_type(x)    = ⟨ Norm_P(P_x), Norm_Val2(Val2_x) ⟩
Norm_Val2(V)    = Map_name( Norm_Cluster(V[name]) )
Norm_Cluster(C) = ⟨ Norm_pureP(C.pureP)?, Multiset{ Norm_val(v) } ⟩
Norm_pureP(x)   = ⟨ Norm_P(P_x), Norm_Val2(Val2_x) ⟩
```

The recursion is **well-founded finite recursion**: every traversed `Val2`
child edge must descend toward a leaf. The leaf boundary is not one special
case but the general condition

```text
L = { x | Children_V(x) = ∅ }
```

where `Children_V(x)` is the set of object children that `Val2` normalization
may continue descending into. `()` is the standard leaf:

```text
Val2(()) = ∅
Norm(()) = ⟨ Norm_P(P_FunctionItem), ∅ ⟩
```

Other typical leaves are terminal built-in type objects and associated pure-P
objects whose concrete object carries no further `Val2` expansion — an
associated type is not a special recursion rule, it is an ordinary pure P that
happens to have run out of children. Future `ref` / `share` / `alias` values
are also leaves, not back references:

```text
Children_V(t ref)   = ∅
Children_V(t share) = ∅
Children_V(t alias) = ∅
PatternOf(t ref)    = t ref        (extraction still matches the form)
```

The `t` in `(t ref)` is pattern material of the built-in meta function that
produced the value, not a vertical object edge inside the produced value.
ref/share/alias extraction is **horizontal, not vertical**: pattern
decomposition never creates a `Val2` child edge, so `extractable` does not
imply `recursively traversable`, and `t → (t ref) → t` never exists as an
object cycle.

`PlaceId` is **not** identity material. A place is only the coordinate from
which an object's `Val2` is observed:

```text
place(x) ⟼ Val2_x
```

so identity follows the observed content in both directions:

```text
P_x = P_y ∧ Norm_Val2(Val2_x) = Norm_Val2(Val2_y) ⇒ Norm_type(x) = Norm_type(y)
P_x = P_y ∧ Norm_Val2(Val2_x) ≠ Norm_Val2(Val2_y) ⇒ Norm_type(x) ≠ Norm_type(y)
```

The first line holds even when `place(x) ≠ place(y)`; the second holds even
when the two observations are of one object through one place at two different
times. A list of allocated value ids under each name is not a normal form:
allocation order is not semantic content, so the walk must resolve each name to
its cluster symbol and normalize that symbol's own members.

This is what makes an open construction observable at all. Given

```lang
let fn = (...): meta -> _ :symbol = {
    let t = (() t) |> struct;

    let f::t = X;
    let A = t |> meta_fn;

    let g::t = Y;
    let B = t |> meta_fn;
    t;
};
```

the two observations of `t` are different type objects:

```text
t_1 = ⟨ P_t, {f} ⟩
t_2 = ⟨ P_t, {f, g} ⟩
```

so `Norm_type(t_1) ≠ Norm_type(t_2)` and therefore
`MetaKey(meta_fn, t_1) ≠ MetaKey(meta_fn, t_2)` — both observations invoke the
SAME callable, so only the recursive `Val2` separates the keys. Reading the
shared Pattern's canonical object instead of the observing carrier's own
object would merge the two meta instances.

Memoizing FINISHED cycle-free subtrees is permitted (a shared acyclic diamond
is DAG reuse, not a cycle), but no `PlaceId` or memo node number may appear in
the resulting normal form, and no `SemanticValueId` may enter the
recursively-normalizable type-object structure (`Norm_P × Norm_Val2`).
A Val1 payload that has no content normal form yet is the one permitted
exception: it keeps an identity-stable opaque leaf (`OpaqueValue`), so two
references to one value share an address while two content-equal but distinct
values stay distinct. This is a safe under-merge, never a claim of a stronger
equivalence than the implementation actually decides. A cyclic `Val2`
(`let loop::t = t;`) has **no normal form**: re-entering an object still on
the active recursion stack proves the well-foundedness violation and is a
hard semantic error. Whether cyclic type objects are ever admitted is a
separate, explicit future language decision — it does not follow from the
normalizer's ability to detect the cycle.

## 3. Value judgment versus place judgment

The model uses two distinct judgments. One evaluates an expression to a value;
the other resolves a name to a writable place.

Value evaluation:

```text
Γ ⊢ x ⇓ v
```

means an expression / rank / type expression evaluates to value `v`.

Place resolution:

```text
Γ ⊢ x ⇐ p
```

means a declaration extension / namespace injection / assignment-like operation
resolves `x` to a writable place `p`.

These are not interchangeable. `let f::T = ...` uses the **place** judgment on
`T`, not the value judgment: it targets the place that `T` owns, not the value
that `T` evaluates to.

### 3.1 General value binding resolves symbols first

The ordinary rule:

```lang
let r = expr;
```

is:

```text
value(symbol(r)) := evaluate(expr)
```

When `expr` is a source path, value evaluation is not direct value naming:

```text
source path
  -> resolve Symbol
  -> read the selected value / PatternValue from that Symbol
  -> bind the value to the destination Symbol/Place
```

Thus:

```lang
let a = b;
```

binds the exact value read through `symbol(b)` into the fresh destination
`symbol(a)`. It does not alias the symbols or merge their places.

Formally:

```text
resolve(b) = s_b
read(s_b)  = v
fresh SymbolId s_a
fresh PlaceId p_a
--------------------------------
bind(a, v)
```

The source carrier `s_b` is not stored as part of `v` after evaluation.
Provenance may mention it; semantic value identity does not. Consequently no
ordinary binding path may recover associated operations by mapping
`TypeValueId` back to an “original defining Symbol”. The forward semantic path
is `Symbol -> value -> PatternValue -> Pattern owner`.

The same separation applies inside derived semantic material. A struct field,
callable signature, canonical argument key, or extraction view that denotes a
type consumes the canonical observation of the evaluated type object
(`Addr(Norm_type)`; the bare `TypeValueId` is only its first-order root):

```text
field source path
  -> carrier Symbol
  -> read TypeValue v
  -> record the observation of v as field-type identity
```

An implementation may temporarily retain the carrier Symbol for graph
navigation or provenance, but it is not part of field-type equality,
Pattern-head identity, or generated type-definition identity. Consequently
`(uint8 field) struct` and `(T field) struct` have the same field-type material
after `let T: type = uint8`; a reverse `TypeValueId -> original Symbol` lookup
would incorrectly make ordinary binding observable.

Extraction interfaces follow the same split. Their semantic owner/type
coordinates are the owner `TypeValue` and Pattern identity. A graph carrier may
still be present to reach installed field projection Symbols, but
`semantic_eq` cannot distinguish two extraction shapes merely because the same
type value is carried by different bindings.

Ordinary Pattern applicability follows the same rule. A written Pattern name is
resolved forward to its `PatternValue`; the actual argument contributes the
`PatternValue` reached through its evaluated type/value. Matching compares
those identities, not the carrier spellings. Hence a formal `_ uint8` accepts a
type value read through `T` after `let T: type = uint8`; comparing the strings
`"uint8"` and `"T"` would be name-category-first resolution in disguise.

The same rule applies to an externally owned pattern value:

```lang
let t1::t = bool;
```

resolves `symbol(bool)`, reads its `PatternValue`, and binds that value to the
destination symbol/place `t1::t`. It does not reroot the pattern, rewrite its
navigation, or make the destination symbol identical to the pattern owner.

Literal syntax is the explicit exception only to source-path resolution. It
still evaluates to a value and uses the same binding rule. In
`let a = 'a';`, the left `a` is a symbol name while the right `'a'` is a
character literal; matching textual content does not make them the same object.
Pattern values have no analogous standalone literal syntax, so same-spelled
symbol paths and pattern diagnostic names must be kept especially distinct.

### 3.2 One navigator, many projections

Symbol-first resolution is a single ordered pipeline:

```text
Path -> ⟨HostChain, TerminalSymbol⟩ -> ContextDirectedProjection
```

Which symbol a path denotes is decided by the path alone. It is **not** decided
by whether the result is subsequently used as a call target, a type, a value,
an injection target, or an extraction subject. One navigation algorithm serves
every context:

```text
resolve the first component in lexical scope     -> Symbol
for each following component:
    select the current Symbol's object facet
    push that object as a host layer onto HostChain
    enter that object's OWN Val2 place
    look up the next associated Symbol
-> ⟨HostChain, terminal Symbol⟩
```

Only the final step is context-directed, and it projects a facet of the already
chosen terminal symbol:

| context | projection |
| --- | --- |
| call target | callable sibling vals |
| type | pure-P member |
| value | sibling vals |
| injection target | writable host object / place |
| extraction | Pattern facet |

Consequently `f::T` denotes `Val2(T)[f]` in all of

```lang
let A: type = f::T;
let B = (f::T) meta_fn;
let g::U = f::T;
(…) |> f::T;
g::f::T
```

and differs only in the facet each site reads. Resolving the same spelling as
an object-level `Val2` path in one context and as a namespace path in another
would make path meaning depend on its consumer, which is name-category-first
resolution in disguise. Namespace children remain reachable: a step consults
the current symbol's object facet and its associated namespace, so ordinary
namespace paths keep resolving unchanged.

The host layers traversed on the way are retained as an ordered `HostChain`,
because per-layer exposure is a conjunction over every layer
(`Expose(g::f::T, φ) = Expose(T_t, φ) ∧ Expose(C_f, φ) ∧ …`) rather than a
property of the terminal symbol alone. Consumers do not re-derive this chain:
ordinary invocation reads the whole navigation and refuses the target unless
**every** host layer is exposed at the current phase, so a hidden outer layer
cannot be bypassed by a visible terminal reached through it. Cross-root
resolution likewise deduplicates on the full `⟨HostChain, TerminalSymbol⟩`; two
roots that reach one terminal through different host chains are a navigation
ambiguity, not a silently-merged result.

## 4. Ordinary type-value binding

Type-value binding is the general value-binding rule under a `type`
expectation, not a separate assignment mechanism. The form:

```text
let T: type = uint8
```

means:

```text
symbol(T) = fresh symbol
place(T) = fresh writable place at current lexical level
value(T) = value(uint8)
type_value(T) = type_value(uint8)
pattern_value(T) = pattern_value(uint8)
```

This must be read precisely:

```text
T is not a fresh nominal type.
T is not a symbol alias.
T has fresh place identity.
T may evaluate to an existing type value.
```

`T` is a new symbol with its own fresh, current-level writable place. Its *type
value* is the value read through `uint8`, while its *place* is its own. Binding to an existing
type value does not generate a new type, and it does not forward to `uint8`'s
symbol or place.

This ordinary declaration rule does not license a meta return symbol to use an
external type value as its type root. A canonical meta instance has an
additional self-root invariant: if its return symbol has a `TypeFacet`, the
facet's outer pattern root must be the `MetaInstanceScope`. Thus ordinary
`let T: type = uint8` remains legal while direct `r = uint8` as a meta return
type construction is rejected.

Consequently, injection through `T`:

```text
let f::T = ...
```

executes:

```text
place(T) += { f ↦ ... }
```

and not:

```text
place(uint8) += { f ↦ ... }
```

Injection is closer to `+=` on a place than to pure expression evaluation. The
right-hand use of a name is a value; the injection target is a place being
extended.

### 4.1 Atomic builtin types, concrete numeric types, and literal typing

The literal spelling family, atomic builtin type, and concrete numeric type are
distinct:

```text
LiteralFamily
  = Integer | Float | String

AtomicBuiltinType T
  = Uint | Int | Float | Buffer | Str

NumericTypeKey Tnum
  = NumericFamily x width
```

A literal family records normalized syntax and is not a type identity. Each
member of `AtomicBuiltinType` denotes an actual atomic builtin type whose
identity, once installed, comes from its Type symbol; it is not merely a
classifier invented by literal materialization. The Rust enum is a lookup key,
not itself a `TypeValueId`.

A concrete numeric key selects a type object such as `uint16` or `float32`.
Current Rust code carries the first-order `TypeValueId` projection derived from
the installed canonical core Type symbol. That projection is transitional
material and does not claim final canonical type-value equality:

```text
literal spelling
  -> LiteralFamily
  -> contextual/default concrete Tnum selection
  -> resolve canonical Type Symbol
  -> project TypeValueId
  -> materialize semantic value
```

The lexical frontend continues to preserve spelling only; it does not choose
width, signedness, precision, or overflow behavior. The semantic selection
step extends that result without changing lexer meaning. An unsuffixed default
and range/context rules remain separate decisions.

Requiring a concrete `Tnum` for numeric literals does not imply that
`uint`/`int`/`float` cease to be Type values. It means only that the numeric
literal's final type is the selected concrete numeric Type rather than the
atomic numeric family Type.

The design target for a string literal is a compile-stage `str` value, not
`str ref`, implicit storage, or a lifetime extension. That statement requires
a `str` Type symbol and its first-order projection in the semantic world. The
current core bootstrap installs `uint8`, `uint16`, `uint32`, and `float32`, but
not `str`; the current helper can materialize a string only when its
`AtomicBuiltinTypeRegistry` contains a resolved `str` projection. It must not
accept an arbitrary numeric identifier as an implemented core `str` carrier.

## 5. Alias forwarding

The alias form is different from ordinary type-value binding:

```text
let T === uint8
```

means symbol forwarding, not a fresh binding:

```text
alias(T) = uint8
value(T) = value(final_target(uint8))
place(T) = place(final_target(uint8))
```

Crucially, the *writability* of the forwarded place depends on the final target,
not on the alias. An alias does not create a fresh writable place; it points at
whatever place the final target owns, with that target's writability.

To reason about forwarding, the model introduces an `AliasChain` concept. It is a
semantic design object, not an implemented structure:

```text
source symbol
forwarded target
final symbol
final value
final place
provenance chain
writable boundary
cycle detection
```

The `AliasChain` records the path from the source alias symbol through any
intermediate forwarding to the final symbol, the final value and place, the
provenance of each hop, where the writable boundary lies, and whether the chain
contains a cycle. Cycle detection is part of the design because forwarding chains
must terminate.

Canonical summary:

```text
alias does not affect type-value equality;
alias still affects symbol forwarding, place forwarding,
namespace injection target, writability, and provenance.
```

This ordinary declaration-layer alias meaning is not removed by the formal meta
return correction. `let a === b` remains valid design syntax. Inside a meta
body, the same alias mechanism applied to the return slot
(`let r === path;`) adds an alias member to the return cluster; only the
obsolete reading of bare `r === ...` as a special formal meta-return category
is removed.

## 6. Writable-place checking

A future writable-place checker decides whether a place may be written or
injected into from the current context. A place is writable only when it
satisfies the current stage and the current lexical/context boundary.

```text
Γ ⊢ place p writable_at current_context
```

At minimum, the following are **not** writable from an ordinary current-level
injection:

```text
core built-in stable object
external package stable object
closed generated object
alias whose final target is not writable
place from an inner lexical level escaping into a longer-lived injection target
place whose namespace delta is sealed/frozen
place whose policy does not admit the current injection action
```

Type-value equality grants no write permission. Even when:

```text
value(T) == value(uint8)
```

it does not follow that:

```text
place(T) == place(uint8)
```

and it certainly does not follow that:

```text
place(uint8) is writable
```

This is the concrete reason `let T === uint8; let f::T = ...` must be rejected:
the chain forwards to `uint8`, whose place is an external stable object and is
not writable from the current level. Alias forwarding cannot turn a non-writable
place into a writable one.

## 7. Namespace injection target

Namespace injection is a *place* operation, not a type-value operation. The
injection target is not determined by ordinary expression evaluation of the
target path.

The intended flow:

```text
parse / normalize injection target path
resolve path as injection-place target
follow alias chain only if alias semantics requires it
check final place writability
install NamespaceDelta under that place
```

The resolver here is asking "which writable place does this path name?", not
"what value does this path evaluate to?". An injection that resolves to a value
rather than a writable place is ill-formed.

Writability alone does not grant construction ownership. Under the current
future construction contract, another source file cannot reopen a namespace,
type, pattern, ordinary value-member, or overload subtree created by a parallel
`SourceConstructionUnit`, even to add a previously absent child. Physical
directory authority and construction-unit ownership are specified in
`symbol-construction-units-and-namespace-origin.md`.

An ordinary type facet is installed once. Repeating:

```lang
let T = A;
let T = B;
```

as two type-facet definitions is a conflict, not implicit `A | B`. Child
construction and sum construction require explicit APIs and remain distinct
from repeated ordinary binding.

> **Open question — `let` versus `=` for namespace injection targets.**
>
> The current implementation conflates fresh binding (`let f::T = expr`) and
> existing-target write under one `let ... = ...` form. This is a
> conservative compromise: the `=` operator is not yet supported, so all
> writes use `let`.
>
> The intended long-term separation:
>
> ```text
> let f::T = expr   — creates a new associated member (fresh symbol)
> f::T = expr       — writes to an already existing target (requires = operator)
> ```
>
> Under this separation, namespace injection target resolution (§7 above)
> applies to both forms, but the *judgment* differs: `let` uses
> creation-place resolution (fresh child symbol), `=` uses write-place
> resolution (existing place, writability check). The §6 rules are not
> suspended for `let`, because two distinct judgments are involved:
>
> ```text
> Fresh(child)            — the created member symbol is fresh
> CanExtend(parent_place) — the host place admits this extension
> ```
>
> `let f::T = expr` creates a fresh child (`Fresh(f)` holds trivially), but
> that creation still extends `T`'s `Val2` object/place, so the host must
> independently satisfy `CanExtend(place(T))`: construction
> authority / open-window state, lexical lifetime, and external-stability
> conditions on the parent place all still apply. Freshness of the child
> never implies extension eligibility of the parent place.
>
> This does not cancel `let f::T = expr` as a valid Val2 injection form.
> It clarifies that `let` creates (fresh symbol/member) whereas `=`
> overwrites (existing target, type value semantics). The `=` operator is
> required for future `inject` support (`inject` provides inward navigation
> resolution; `=` provides the overwrite that makes the result observable)
> and for cluster-symbol synthesis, but its absence does not invalidate
> the current `let`-based Val2 injection path.

## 8. Type values in overload and pattern matching

First-order type matching for overload and pattern compatibility uses
`TypeValueId`, not source symbol names. (The candidate-preparation layer that
consumes type values is specified in
`pattern-normalization-and-first-order-overload.md`; this document defines what
a type value identity is.) This first-order layer is one of the remaining
bare-`TypeValueId` comparison consumers scheduled to migrate to full by-value
comparison (`Addr(Norm_type)`, §2); until that migration it is a first-order
projection check only, never canonical type-value equality.

For example:

```text
let T: type = uint8
```

In first-order type matching, `T` and `uint8` may carry the same `TypeValueId`.
But this says nothing about their places:

```text
T and uint8 may have the same TypeValueId but different PlaceId.
```

The same separation applies to normalized pattern layers. If the layer is the
body of a Pattern and every direct element has a complete top-pattern
navigation name, it is
`Map<CanonicalFullNavigation, CanonicalPatternValue>`. A naked Product remains
positional regardless of whether its elements are named. `SymbolId` and
`PlaceId` identify carriers/locations; they are neither map keys nor resident
values. Extraction resolves a source symbol, reads its `PatternValue`, and
looks up that value by complete navigation and normalized resident. A symbol
path may share the value's navigation spelling or differ from it without
changing this sequence. Source/provenance classification does not participate
in `PatternValue` identity.

Pass mode is **not** part of `TypeValueId`. A construct such as `T move` does not
change the type value, and type-value comparison is invariant under
`move` / `copy` / `ref` / `share`. The detailed treatment of `T move == T` as a
move fixed point belongs to a future mechanical argument-passing / move design
and is only referenced here, not expanded.

## 9. Alias forwarding and policy

Alias forwarding redirects lookup; it does not grant capabilities. It must
operate within the existing policy, visibility, and writable-place restrictions.

```text
Alias may redirect lookup.
Alias may expose a forwarded value.
Alias must not manufacture permission.
Alias must not make non-writable places writable.
Alias must not bypass policy filtering.
```

If an alias target is not visible or not executable under the current
`PolicyEnv`, the alias does not make it visible or executable. A re-export or
wrapper semantics that intentionally re-exposes a forwarded target under
different policy is a separate, later design and is **not** defined here.

## 10. Relation to current implementation

The `lang_build` semantic spine now implements the identity core of this
document: the recursive type-object normal form `Norm_type`, the canonical
observation identity `Addr(Norm_type)` consumed by struct residents,
canonical pattern atoms, and meta instance keys, per-carrier `Val2` places
for ordinary type bindings (`Pattern(T) = Pattern(U)` coexisting with
`Place(T) ≠ Place(U)`), and meta return self-root validation. The
`TypeObject` adapter survives only as a per-TypeValue transport reference
inside an object place, never as a binding-level policy authority.

Still future work, tracked in `spec/planning/open-questions.md`: explicit
alias forwarding through an `AliasChain`, writable-place checking,
alias-forwarded injection places, source-level injection through installed
rebinding carriers, migration of the remaining first-order `TypeValueId`
comparison consumers to full by-value comparison, and construction-unit
ownership.

## 11. Non-goals

```text
No parser syntax change.
No Rust implementation change in this PR.
No test fixture change.
No full type checker.
No full alias resolver implementation.
No full lifetime/access-tree checker.
No runtime lookup implementation.
No package re-export semantics.
No permission escalation through aliasing.
No current public behavior change.
```

## 12. Relationship to other documents

The documents below are adjacent or background design. They do not define the
distinctions specified here, and this document does not depend on them for its
meaning.

- `symbol-first-meta-construction-and-pattern-injection.md` — canonical
  symbol-first facet resolution, `PatternValue`, `compile` / `meta`, pattern
  scopes, `struct`, functional `inject`, and binding/install boundary. It uses
  this document's `SymbolId` / `PlaceId` / `TypeValueId` and alias judgments.
- `type-associated-function-objects-and-access-trees.md` — field functions,
  projection namespaces, role-aware lookup, and access-tree work. It references
  this document for the canonical type-value / place / alias-forwarding
  distinction rather than restating it.
- `early-meta-functions-and-namespace-graph.md` — the build / namespace graph and
  early-meta slice, including the v0.6 placeholder `TypeObject` representation
  this document supersedes as the long-term semantics.
- `symbol-construction-units-and-namespace-origin.md` — canonical
  `NamespaceOrigin`, construction-unit ownership, physical contribution
  authority, type/namespace facet inclusion, and cross-file closure rules.
- `entity-alias-design.md` — the surface/parser alias syntax (`let binder ===
  EntityRef`) and frozen parser preservation. This document defines the
  *semantic* alias forwarding model (value/place forwarding, `AliasChain`,
  writable-place effect) that surface design will later target.
- `pattern-normalization-and-first-order-overload.md` — the pattern/type
  candidate-preparation layer that uses `TypeValueId` for first-order type
  matching.
