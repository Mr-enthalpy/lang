# Type Values, Places, and Borrow Views

**Status: canonical target semantics for object identity, place identity, and
borrow views. The identity core (recursive object normal form, canonical
observation `Addr(Norm)`, per-carrier `Val2` places) is current `lang_build`
behavior; the borrow-view operators (`ref`, `share`, `@`, `rebind`), the
extension-eligibility judgment, and the type checker remain unimplemented
target semantics. §10 registers the implementation debt.**

This document specifies the semantic boundary between *object values*, *symbol
identity*, *places*, *borrow views*, and *namespace extension targets*. It
defines what an object is, what a place is, which borrow operators exist, which
overloads they have, and when each overload is callable. It is a semantic
authority, not a parser or normalizer rule.

The document is self-contained. It does not require the reader to assemble its
meaning from `type-associated-function-objects-and-access-trees.md` or
`early-meta-functions-and-namespace-graph.md`. Those documents are background
or adjacent design only; the model here stands on its own and is the canonical
authority for the value / place / symbol / borrow-view distinction.

There is no ordinary symbol-alias or place-forwarding declaration form in this
language. `let a = b;` copies a value into a fresh symbol with a fresh place.
Sharing an observation of another object is expressed by the borrow operators
defined in §5, never by a declaration that makes two symbols name one place.

The broader symbol-first facet, `PatternValue`, `compile` / `meta`, pattern
scope, `struct`, and functional `inject` model is canonicalized in
`spec/design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md`.
That document composes with this identity/place model rather than replacing it.

## 1. Purpose

The language must distinguish three things that look similar in source text but
are semantically different: the identity of a name, the identity of a writable
location, and the identity of a type value. Conflating them produces subtle
errors — for example, injecting a declaration into a built-in type because its
type value happens to equal a freshly bound symbol's type value.

The invariants this document protects:

```text
value equality must not collapse symbol-place identity
a borrow view must not manufacture a place that its source does not own
namespace extension must target extendable places, not values
```

This document does **not** define:

- a full type checker,
- access-tree construction,
- the lifetime checking algorithm (see `../lifetime/lifetime-policy-and-overload-boundary.md`),
- package import / export,
- runtime lookup.

Three phrasings are explicitly rejected throughout. `let T: type = uint8` is
**not** fresh nominal type generation. A borrow view is **not** textual
substitution and **not** a second name for a symbol. And value equality is
**not** place equality.

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
A borrow view names one place from one origin; it relates values and places without erasing the distinction.
```

A type expression cares about the *value*. A namespace extension target or a
declaration-extension site cares about the *place*. A borrow view is itself a
value that carries a place coordinate. The three concerns must not be folded
into one another.

In the symbol-first model, a path initially resolves to one symbol cell and the
use site then projects namespace, type, or heterogeneous value facets. Facet
projection does not collapse these identities and is not a cast.

### 2.1 Object identity is the recursive three-component normal form

Every object in the language has the same shape:

```text
Object x  = ⟨ Val1?(x), P(x), Val2(x) ⟩
Val1?(x) ∈ 1 + Object
```

`Val1?(x) = null` states exactly one fact: this object carries no internal
`Val1` payload. It does not mean the object is untyped, unobservable,
value-less at the observation edge, or a different kind of entity.

Two distinct notions must not be collapsed here:

| notion | condition | what it is |
| --- | --- | --- |
| pure type object (type seed) | `Val1?(x) = null` | an object with no payload; the operand of `@` in §5.2 |
| type-rank use of an object | positional | what a type-expected position asks for (§5.6) |

A pure type object is simply an object whose `Val1?` is `null`. It is **not** a
definition of "type rank": an object that carries a `Val1` payload may still be
used where a type is expected, and the type-expected position supplies the
projection (§5.6). Conversely, `Val1?(x) = null` does not by itself make `x` a
type for every purpose — it makes `x` payload-less, which is what §5.2 depends on.
Keeping these apart is what prevents `Val1?` from being re-read as an implicit
kind classifier.

The canonical identity of an object is the recursive normal form over **all
three** components:

```text
Norm(x)         = ⟨ Norm_Val1?(Val1?(x)), Norm_P(P(x)), Norm_Val2(Val2(x)) ⟩
Norm_Val1?(null)= null
Norm_Val1?(v)   = Norm(v)
Norm_Val2(V)    = Map_name( Norm_Cluster(V[name]) )
Norm_Cluster(C) = ⟨ Norm_pureP(C.pureP)?, Multiset{ Norm_val(v) } ⟩
Norm_pureP(x)   = Norm(x)
```

There is no case split in which one component is ignored. Earlier revisions
normalized `Val1? = null` objects as `⟨P, Val2⟩` and `Val1? ≠ null` objects as
`⟨Val1, P⟩` with `Val2` discarded; that bifurcation is retired. A value-bearing
object whose `Val2` differs is a different object, and a type object whose
`Val1?` is `null` still normalizes its `P` and `Val2` fully. `Norm_type(x)` is
retained only as the name of `Norm(x)` restricted to objects with
`Val1?(x) = null`, and its two-component spelling is shorthand for the
three-component form with a `null` first slot.

Carrier coordinates never enter the normal form:

```text
ObjectPlaceId  ∉ Norm(x)
SymbolId       ∉ Norm(x)
allocation order ∉ Norm(x)
provenance     ∉ Norm(x)
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
Norm(()) = ⟨ null, Norm_P(P_FunctionItem), ∅ ⟩
```

Other typical leaves are terminal built-in type objects and associated pure-P
objects whose concrete object carries no further `Val2` expansion — an
associated type is not a special recursion rule, it is an ordinary pure P that
happens to have run out of children. Borrow views (`ref` / `share`) are also
leaves, not back references:

```text
Children_V(t ref)   = ∅
Children_V(t share) = ∅
PatternOf(t ref)    = t ref        (extraction still matches the form)
```

The `t` in `(t ref)` is pattern material of the built-in operation that
produced the value, not a vertical object edge inside the produced value.
Borrow-view extraction is **horizontal, not vertical**: pattern decomposition
never creates a `Val2` child edge, so `extractable` does not imply
`recursively traversable`, and `t → (t ref) → t` never exists as an object
cycle.

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
recursively-normalizable object structure.
A `Val1` payload that has no content normal form yet is the one permitted
exception: it keeps an identity-stable opaque leaf (`OpaqueValue`), so two
references to one value share an address while two content-equal but distinct
values stay distinct. This is a safe under-merge, never a claim of a stronger
equivalence than the implementation actually decides, and never a licence to
treat `Val1` as excluded from the normal form: the target rule is that `Val1?`
normalizes recursively like every other component, and the opaque leaf is a
placeholder for content normalization that is not yet implemented. A cyclic
`Val2` (`let loop::t = t;`) has **no normal form**: re-entering an object still
on the active recursion stack proves the well-foundedness violation and is a
hard semantic error. Whether cyclic objects are ever admitted is a separate,
explicit future language decision — it does not follow from the normalizer's
ability to detect the cycle.

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
Path -> SelectedHead -> ⟨HostChain, TerminalSymbol⟩ -> ContextDirectedProjection
```

The stability claim applies to the **tail**, not to the head. Once the head
symbol is selected, the remaining navigation is decided by the path alone: it is
**not** decided by whether the result is subsequently used as a call target, a
type, a value, an injection target, or an extraction subject. Head selection is a
separate, earlier step with its own rule.

#### 3.2.1 Head selection: bare versus explicitly anchored

The two forms do not use the same rule, and the difference is confined to this
step:

```text
ResolveBare_q(name)
  = the nearest enclosing Symbol spelled `name` that carries the required
    coarse facet q

ResolveExplicit(path)
  = the uniquely designated anchor, taken as written
```

A bare head may look outward, and the coarse facet `q` demanded by the use site
participates in that search. An explicitly anchored path may not look outward at
all. The search discipline is:

```text
bare head    : walk outward; stop at the first same-spelled Symbol carrying q
explicit head: no outward walk; the written anchor is the head or resolution fails
```

The outward walk is bounded to exactly one decision. Once a Symbol carrying `q`
is found, that Symbol is the head, permanently:

```text
overload resolution failing inside the selected head
  -> the program is ill-formed
  -> NOT a reason to resume the outward walk
```

This is what keeps a local type-only Symbol from silently shadowing an outer
callable Symbol of the same spelling: at a call site the coarse facet demanded is
callability, so a local Symbol that carries no callable facet is simply not a
candidate head. It is equally what stops the search from degenerating into
"retry outward until something type-checks" — the facet is coarse, and it is
consulted once.

`q` is coarse in the strict sense: it distinguishes facet presence, never
signatures, argument types, arity, or specificity. Head selection therefore never
becomes overload resolution in disguise.

#### 3.2.2 Tail navigation is context-independent

After the head is fixed, one navigation algorithm serves every context:

```text
SelectedHead                                     -> Symbol
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
| extension target | extendable host object / place |
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

The coarse facet of §3.2.1 is not an exception to this. It participates only in
selecting the head, once, and it distinguishes facet presence rather than
meaning; the tail steps and the final projection remain as above.

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

## 5. Borrow views

There is no declaration form that makes two symbols share one symbol identity
or one place. Shared observation is expressed by three operators with distinct
input judgments.

### 5.1 `ref` and `share` are value operations

`ref` and `share` apply to the **value** of their operand expression. That value
is whatever the ordinary read of §3.1 produced. `Read` always yields a complete
three-component object; the `Val1` dimension of the slot decides only whether the
first component is populated:

```text
Read(Σ) = ⟨ Val1(Σ), P(Σ), Val2(Σ) ⟩    when Val1(Σ) ≠ ⊥
Read(Σ) = ⟨ ⊥,       P(Σ), Val2(Σ) ⟩    when Val1(Σ) = ⊥

E ref   = Ref( Read(E) )
E share = Share( Read(E) )
```

`Val1` is the object's internal payload, **not** the read result. `Read` never
projects an object down to its payload, so the result keeps its own `P` and
`Val2` and is named by its own Pattern. The only difference the two cases make is
whether the read value carries a payload at all — and that is what §5.2 depends
on.

Both are ordinary meta-function calls on that result. Neither asks which symbol
slot the value came out of, and neither consults, captures, or exports it.
Therefore:

```lang
let t = uint8;
let r = t ref;
```

binds `r` to `uint8 ref` — a borrow view of the type object `uint8` — and not to
a reference to the symbol slot `t`. Rebinding `t` afterwards does not change
`r`. The slot itself is reached only by `t@` (§5.2).

`share` differs from `ref` in the capability it grants, not in the judgment it
uses: a `share` view admits reading and passing but is not an assignable place
and cannot be an `inject` target (§5.5).

#### 5.1.1 A `Val1` payload makes `ref` sufficient on its own

When the operand slot has a `Val1` payload, `Read` yields the complete object
that carries it, and `ref` borrows that object. Nothing further is required, and
nothing is elaborated in front of the operator:

```lang
let s: symbol = ...;
let r = s ref;              // Read(s) : symbol, so r : symbol ref
```

A symbol value is value-bearing (`Val1(Symbol) = Member * ω`), so `s ref` is the
ordinary "form a borrow of this value" operation. Because `Read` does not descend
into `Val1`, `r` is a `symbol ref` and **not** a reference to the member array
held inside the symbol. What `r` borrows is the symbol value that `s` holds, not
the binding slot that holds `s`, and not the payload inside that value.

The rule is about the presence of the `Val1` dimension, not about type-rank. An
object that happens to sit at type rank and still carries a payload takes the
same path, and is likewise named by its own host Pattern:

```text
x        = ⟨ v, P_val_has_type_field, Val2 ⟩
Read(x)  = ⟨ v, P_val_has_type_field, Val2 ⟩
x ref    : val_has_type_field ref
```

Reaching `v` itself is an ordinary member/projection operation on the read
result, not something `Read` or `ref` performs implicitly.

No implicit projection or conversion participates in an operand position. `s ref`
is never elaborated into `s |> type ref`, or into any other facet projection,
because an operand or argument position performs no implicit type conversion.
Facet projection stays explicit there (`|> type`, `|> val`, `|> namespace`), and
an explicit `symbol |> type` is itself well-formed whenever the operand really
carries a `Val1` dimension — what is excluded is supplying it on the writer's
behalf. A language-designated type-expected position is the separate case where
the projection *is* supplied; see §5.6.

### 5.2 `@` is the carrier-slot operation

`@` is the only borrow operator that consults where its operand came from. In its
borrow-producing group that is:

```text
E@ = RefCarrierSlot( CarrierPlace(E) )
```

`@` is an ordinary overloaded operation, not a syntax-only marker and not
excluded from overload resolution. Its overload groups are specified in
`../lifetime/lifetime-policy-and-overload-boundary.md`, which is the canonical
owner of `@`. This document states only the facts that belong to the place/value
model:

```text
CarrierPlace(E) is required input to @ and is not required input to ref / share
E@ is undefined when E has no carrier place (a freshly computed temporary)
@ is not a general PlaceOf(E) available on every expression
```

The last line is a domain restriction, not a style rule: the borrow-producing
group of `@` exists for pure pattern slots (§5.2.1), and a value-bearing operand
has no borrow-producing `@` candidate at all, because `ref` already expresses
that borrow. On a complete `⟨ Val1, P, Val2 ⟩` object, `@` is the lifetime
observation — a different operation with a different result, and one this
document does not narrow.

#### 5.2.1 `@` exists because a pure pattern read hides the carrier slot

`@` is not a stylistic alternative to `ref`, and it is not the borrow operator
of types. It fills exactly one gap: when `Val1(Σ) = ⊥`, the ordinary read has
already selected the pure pattern facet, so the carrier slot is no longer
reachable from the value:

```lang
let t: type = uint8;

t ref   // Ref(Read(t))            = uint8 ref
t@      // RefCarrierSlot(t)       = type ref, pointing at the slot t
```

`t ref` is not a mistake to be corrected; it is the correct borrow of the value
that was read. A value-directed meta-function has no business guessing that the
writer actually meant the slot underneath. `@` is the explicit divider between
the two readings.

So the operator choice is decided by the `Val1` dimension of what is read, never
by type-rank:

| what the expression reads | `E ref` | `@` needed |
| --- | --- | --- |
| ordinary value with `Val1` | borrow of that `Val1` value | no |
| symbol value with `Val1` | `symbol ref` | no |
| type-rank value with `Val1` | `ref` of that value's type | no |
| pure pattern value | `ref` of that pattern value | only to reach the carrier slot |
| pure `type` slot | `ref` of the concrete type | `E@` is what yields `type ref` |

Consequently the compile stage offers no borrow-meaning `@` candidate for an
operand that has a `Val1` payload:

```lang
s ref   // borrows the Val1 value of s
s@      // not an ordinary way to obtain a borrow
```

`@` on such an operand still carries its established meaning: on a complete
object shape `⟨ Val1, P, Val2 ⟩`, `@` takes that object's lifetime. That group is
separate from the compile-stage pure-pattern-slot group and is not a way to
obtain a borrow, and the narrowing above does not weaken it — the two groups have
disjoint premises and neither is a fallback for the other.

### 5.3 Borrow views are non-stacking because the overlapping overloads exist

Applying a borrow operator to something that is already a borrow view is
**well-formed**. There is a candidate for it, and that candidate is what makes
borrowing behave idempotently instead of building a second layer:

```text
Borrow_k( Borrow_j(q) )  =  Coerce_{j->k}( Borrow_j(q) )

Target( Coerce_{j->k}(v) )  =  Target(v)
```

The result is never a view of a view. The target is preserved and only the
capability index changes, so the whole family collapses to one layer:

| composition | result | why |
| --- | --- | --- |
| `ref ref` | the same `ref` view | `Coerce` at equal capability is the identity |
| `share share` | the same `share` view | same |
| `ref share` | a `share` view of the same target | legal weakening |
| `share ref` | **no candidate** | illegal strengthening |
| `@@` | the same view | no retarget occurs |

So the two statements that used to sit next to each other are now one statement.
Idempotence is the *consequence* of providing the equal-capability overload, not a
rule that contradicts it:

```text
Borrow_j( Borrow_j(q) ) = Borrow_j(q)          idempotence, from Coerce_{j->j} = id
ref  -> share  is a capability weakening       admitted
share -> ref   is a capability strengthening   no candidate
```

This is what makes the weakening used throughout this document well-formed:

```lang
let r = t@;                 // r : type ref
let s = r share;            // ref share: s : type share, same target
```

`r share` is exactly the `ref share` composition. It is admitted, it does not
nest, and it does not retarget.

Only `share ref` is rejected, and it is rejected at selection time as "no
applicable overload" rather than being evaluated and then diagnosed: a `share`
view never carries the write/extension capability that `ref` would have to
produce. Capability can be surrendered, never regained.

No overlapping composition changes what is observed:

```text
retargeting is available only through rebind (§5.4)
```

### 5.4 Writing through a reference versus retargeting a reference

A reference value is itself held in a place. The two operations are distinct and
both are ordinary assignments — they differ in **which** place is the target:

```lang
r_ref = value;              // writes value into the referent
r_ref rebind = expression;  // retargets r_ref itself at a new referent
```

```text
r_ref = v          ->  Write( Referent(r_ref), v )
r_ref rebind = E   ->  Target( Value( HolderPlace(r_ref) ) ) := NewTarget(E)
```

`rebind` is a **retargeting** operation, not a value borrow. It does not evaluate
`E ref`, because `Ref(Read(E))` would reintroduce exactly the ambiguity that §5.2
removed: for a pure `type` slot `t`, `Ref(Read(t))` is `uint8 ref`, not a
reference to the slot `t`. So the new target is taken from a place-bearing right
side:

```text
NewTarget(E) = Target(E)          when E is already a borrow view
NewTarget(E) = CarrierPlace(E)    when E supplies a carrier place
NewTarget(E) is undefined         otherwise
```

An `E` that supplies neither — a freshly computed temporary — gives `rebind` no
applicable candidate. The obligations a `rebind` must discharge are:

```text
E supplies an origin/place
Pattern( NewTarget(E) ) conforms to the Pattern the view is declared over
Capability( result ) ≤ Capability( E )        no strengthening
lifetime / escape check on the new target
```

The last obligation is the escape check of
[`../lifetime/lifetime-policy-and-overload-boundary.md`](../lifetime/lifetime-policy-and-overload-boundary.md)
§3.

Without `rebind`, an assignment whose left side is a reference always reaches
through to the referent. `rebind` is what selects the borrow-holder place as the
assignment target. There is no context in which the same spelling means both.

### 5.5 `type`, `type ref`, and `type share`

A `type` value has no place of its own; consuming one can only produce a new
value. It also carries no construction capability: the same pure pattern value
may be inside an open window in one context and merely frozen-but-globally-live
in another, and nothing in the value distinguishes the two.

A `type ref` is different in kind. It can only be formed inside the open window
of its target's carrier:

```text
Carrier(t) = q      Open_Γ(q)
---------------------------------
Γ ⊢ t@ : type ref
```

So a well-formed `type ref` is not merely `⟨Place, type⟩`. It is
capability-equivalent to

```text
⟨ Place, type, OpenWitness ⟩
```

and therefore carries the invariant

```text
Γ ⊢ r : type ref   =>   Open_Γ( Target(r) )
```

`OpenWitness` is not required to exist as a runtime field. It is required to be
an unforgeable fact of the static judgment.

The consequence is a holdable interval, not a permission re-check:

```text
holdable interval of a type ref  =  the Open window
holdable interval of a type ref  ≠  Lifetime(Target) has not ended yet
```

Once the window closes, that `type ref` cannot continue to exist as a legal
usable value in any later context. The way to carry something across the
boundary is to weaken it *before* the boundary:

```lang
r share    // type share: still observable, no structural extension eligibility
```

This is also why a `type ref` satisfies the capability requirement of `inject`
directly, with no ambient query — see
`symbol-first-meta-construction-and-pattern-injection.md` §8.2.3.

Reachability alone still forms no view:

```text
GlobalLifetime(x) does not imply Open_Γ(x)
```

`EffectiveOpen` — the context-relative form of `Open_Γ` — is defined in
`symbol-first-meta-construction-and-pattern-injection.md` §12.

`type share` is the deliberately weaker view. It may cross an open-capability
boundary and be stored or passed where a `type ref` may not, precisely because
it is not assignable and not an `inject` target:

```text
type share crosses an Open boundary
type share is not a valid assignment left side
type share is not a valid inject target
```

The last two lines are domain facts. A `type share` in an assignment-target or
`inject`-target position produces "no applicable overload", never a permission
error discovered after the operation has begun.

#### 5.5.1 Three separate responsibilities

Because the witness travels with the view, the three obligations never collapse
into one check:

```text
inject on a type            ->  ask the evaluation context for ambient Open
inject on a type ref        ->  the view already proves Open; ask nothing
returning / storing a ref   ->  escape check: does the Open capability escape?
```

Returning a `type ref` from a `compile` callable is therefore not forbidden. A
return that stays inside the same open window is well-formed; a return that
crosses the closing boundary fails at the third obligation — the receiving
context cannot derive `out : type ref` at all — rather than surfacing later as a
failed `inject`.

### 5.6 Type-expected positions do elaborate `|> type`

§5.1.1 excludes implicit projection in *operand* positions. That exclusion is
about operand positions only, and it must not be read as "the language performs
no type-context projection anywhere". The two rules are complements, not one
rule:

```text
OperandPosition       =/=>  ImplicitTypeProjection
TypeExpectedPosition   ==>  ImplicitTypeProjection
```

In a language-designated type-expected position the elaboration is supplied:

```text
Elab_Type(E)  =  E |> type
```

The designated positions are:

| position | example |
| --- | --- |
| declaration annotation | `let x: E` |
| a path component that demands the type facet | the type-facet step of §3.2 |
| type argument position | a parameter declared to receive a type |
| `t: type` | a parameter or binder at type rank |
| `t: type ref` | the borrow-view form of the same |
| type-rank return position | a callable whose return is declared at type rank |

So `E` supplying a `Val1` dimension in one of these positions is projected to its
type facet without the author writing `|> type`, while the very same `E` under
`ref` is not:

```lang
let x: s = ...;             // type-expected: elaborates to s |> type
let r = s ref;              // operand: no projection; r : symbol ref
```

The distinction is positional and fixed by the language, never inferred from the
operand's shape. An operand position never acquires a projection because a
projection would make the program check.

## 6. Writability and extension eligibility

A future checker decides whether a place may be written or extended from the
current context. A place is eligible only when it satisfies the current stage,
the current construction anchor, and the current lexical boundary.

```text
Γ ⊢ place p writable_at current_context
Γ ⊢ place p extendable_at current_context
```

At minimum, the following are **not** eligible for an ordinary current-level
extension:

```text
core built-in stable object
external package stable object
closed generated object
place reached through a share view
place whose target is no longer EffectiveOpen at this context
place from an inner lexical level escaping into a longer-lived extension target
place whose namespace delta is sealed/frozen
place whose policy does not admit the current action
```

Value equality grants no write permission. Even when:

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

This is the concrete reason `let f::uint8 = ...` must be rejected while
`let T = (() t) |> struct; let f::T = ...;` is accepted: extension eligibility
is a property of the target place, and no binding or borrow view can promote an
externally stable place into an extendable one. A borrow view never widens the
eligibility of the place it observes:

```text
Eligible(view of p) ≤ Eligible(p)
```

## 7. Namespace extension target

Namespace extension is a *place* operation, not a value operation. The target is
not determined by ordinary expression evaluation of the target path.

The intended flow:

```text
parse / normalize the target path
resolve the path as an extension-place target
check the final place's extension eligibility
install NamespaceDelta under that place
```

There is no forwarding-chain step: a path resolves to exactly one place, and a
borrow view interposed on that path either denotes the same place (`ref`) or
removes eligibility entirely (`share`).

The resolver here is asking "which eligible place does this path name?", not
"what value does this path evaluate to?". An extension that resolves to a value
rather than a place is ill-formed.

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

### 7.1 `let` creates a member; `=` writes an existing target

The two forms are distinct operations on the same resolved target place:

```text
let f::T = expr   — creates a new associated member (fresh child symbol)
f::T = expr       — writes an already existing target
```

Both use the extension/write-place resolution of §7, but they discharge
different obligations:

```text
Fresh(child)            — the created member symbol is fresh
CanExtend(parent_place) — the host place admits this extension
```

`let f::T = expr` satisfies `Fresh(f)` trivially, but the creation still extends
`T`'s `Val2` object/place, so the host must independently satisfy
`CanExtend(place(T))`: construction authority, `EffectiveOpen` state, lexical
lifetime, and external-stability conditions on the parent place all still apply.
Freshness of the child never implies extension eligibility of the parent.

The two forms also differ in what they change about the host:

```text
let f::T = expr   ->  Val2(T)[f] += expr        (P(T) unchanged)
T |> inject(Δ)    ->  P(T) + child pattern, Val2 + interpretation
```

An ordinary member declaration adds a `Val2` entry under an existing pattern
name; it does not widen `P(T)`. Widening the host pattern with a new child
pattern is exactly what `inject` does, and it is specified in
`symbol-first-meta-construction-and-pattern-injection.md` §8. Both are limited to
extending the current parent pattern with a *direct* child; neither reaches into
a grandchild pattern.

Assignment checks exactly three things about its right side and nothing else:

```text
the left side names a writable place
the right value conforms to the target's Pattern
lifetime and capability conditions of the target place hold
```

There is no check of how the right value was constructed. Assignment does not
require a construction witness, a transition proof, or provenance from any
particular producer.

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
`move` / `copy`. Borrow views are different: `T`, `T ref`, and `T share` are
three distinct values with distinct patterns, because a borrow view is a value
produced by an operation (§5), not a passing annotation. The detailed treatment
of `T move == T` as a move fixed point belongs to the mechanical
argument-passing / move design and is only referenced here, not expanded.

## 9. Borrow views and policy

A borrow view observes; it does not grant capabilities. It must operate within
the existing policy, visibility, and place-eligibility restrictions.

```text
A borrow view may expose an observation of its source value.
A borrow view must not manufacture permission.
A borrow view must not make an ineligible place eligible.
A borrow view must not bypass policy filtering.
```

If the observed object is not visible or not usable under the current
`PolicyEnv`, taking a `ref` or `share` of it does not make it visible or usable.
Re-export or wrapper semantics that intentionally re-expose a target under a
different policy is a separate, later design and is **not** defined here.

## 10. Relation to current implementation

The `lang_build` semantic spine implements the identity core of this
document: the recursive object normal form, the canonical
observation identity consumed by struct residents,
canonical pattern atoms, and meta instance keys, per-carrier `Val2` places
for ordinary type bindings (`Pattern(T) = Pattern(U)` coexisting with
`Place(T) ≠ Place(U)`), and meta return self-root validation. The
`TypeObject` adapter survives only as a per-TypeValue transport reference
inside an object place, never as a binding-level policy authority.

Registered implementation debt — semantics closed here, not yet built:

```text
full three-component Norm(x) including recursive Norm_Val1?
  (current normalizer keeps an opaque Val1 leaf)
ref / share / @ / rebind operations and their overloads
type ref and type share values, and ValidContext for them
the writability and extension-eligibility judgments of §6
the = assignment operator and its three-condition check
migration of remaining first-order TypeValueId comparison consumers
  to full by-value comparison
construction-unit ownership enforcement
```

The retired alias-forwarding model (`AliasChain`, symbol/place forwarding,
alias-forwarded extension places, installed rebinding carriers) is not
implementation debt. It is removed from the target semantics and must not be
revived as future work.

## 11. Non-goals

```text
No parser syntax change.
No full type checker.
No full lifetime/access-tree checker.
No runtime lookup implementation.
No package re-export semantics.
No permission escalation through borrow views.
No revival of symbol-alias or place-forwarding declaration forms.
```

## 12. Relationship to other documents

The documents below are adjacent or background design. They do not define the
distinctions specified here, and this document does not depend on them for its
meaning.

- `symbol-first-meta-construction-and-pattern-injection.md` — canonical
  symbol-first facet resolution, `PatternValue`, `compile` / `meta`, pattern
  scopes, `struct`, functional `inject`, `EffectiveOpen`, and the
  binding/install boundary. It uses this document's `SymbolId` / `PlaceId` /
  `TypeValueId` and place judgments.
- `../lifetime/lifetime-policy-and-overload-boundary.md` — canonical owner of
  `@`, its overload groups, and escape checking. This document supplies only the
  `Origin`/`Value` split that `@` consumes.
- `type-associated-function-objects-and-access-trees.md` — field functions,
  projection namespaces, role-aware lookup, and access-tree work. It references
  this document for the canonical value / place / borrow-view distinction rather
  than restating it.
- `early-meta-functions-and-namespace-graph.md` — the build / namespace graph and
  early-meta slice, including the v0.6 placeholder `TypeObject` representation
  this document supersedes as the long-term semantics.
- `symbol-construction-units-and-namespace-origin.md` — canonical
  `NamespaceOrigin`, construction-unit ownership, physical contribution
  authority, type/namespace facet inclusion, and cross-file closure rules.
- `pattern-normalization-and-first-order-overload.md` — the pattern/type
  candidate-preparation layer that uses `TypeValueId` for first-order type
  matching.
