# Function Object Call Model

Design consolidation note. Not a user-visible syntax document.

The canonical symbol-first/facet boundary is
`symbol-first-meta-construction-and-pattern-injection.md`. This document owns
the type-associated `()` call mechanism; it does not redefine how a name first
resolves to a symbol and heterogeneous value facet.
Policy pairs, binding P1, result P2, compile companions, and must-select consistency
are canonical in `symbol-policy-and-compile-flow-projection.md`.

## 1. Basic thesis

A function is a value.

```text
let f = (self) => {};
```

`f` is a value of an anonymous function-object type `F`. `F` is usually not nameable in source syntax (obtainable as `f |> type`).

The converse is not true: a value entry need not be a function. A symbol's
value facet may contain ordinary uncallable values and multiple heterogeneous
values of unrelated types. Callability is tested only in a call position by
resolving each value's type-associated `()` entry.

Under the associated namespace of `F` there is a call entry `() :: F`. This `()` is the call method of the function object.

Function call is not primitive textual application. It is resolved through the callable object's type-associated namespace.

## 2. Pipeline call form

```text
Product |> Expr
```

`Expr` is the would-be callable object. The call process:

1. Evaluate/resolve `Expr` as a value object
2. Obtain `type(Expr)`
3. Inspect the associated namespace of `type(Expr)`
4. Find the call entry `()`
5. Invoke that entry with implicit caller/self + explicit Product

The target expression is not itself the call method. The target is a value whose type-associated namespace contains the call method.

When `Expr` is a name/path, resolution first produces a symbol and projects its
heterogeneous value facet. Candidate preparation observes each enumerated
object's `Pv:Pp` view for the current lookup domain before type-associated call
lookup. The remaining steps run independently for each surviving value entry;
entries without an applicable `()` call entry are discarded.

### 2.1 Compiler-inserted atomic runtime migration call

The language-authorized static-value-to-runtime-value migration is a
compiler-inserted use of this same call trunk, not a second callable kind:

```text
consumer demand
  -> project the complete accepted Policy choice over existing views
  -> if successful, use it and stop
  -> otherwise, if the query accepts runtime, extract its runtime branch
  -> select an existing static source Policy view
  -> resolve the ordinary atomic-migration callable family to Symbol
  -> enumerate its heterogeneous Val2
  -> obtain each candidate value's TypeValue
  -> resolve associated ()
  -> build InvocationFrame
       slot 0 = selected function object
       explicit input = source value view
  -> ordinary structural/Type/Pattern applicability
  -> if future fallback metadata is present, suppress it when any admissible
     non-fallback candidate exists
  -> Bp extended by input/output endpoint Policy fit
  -> ordinary B1..B6 filters
  -> unique ordinary invocation
  -> ordinary result entries
  -> project the demanded runtime output view
```

This implicit operation preserves Type while constructing a new runtime value
object; it does not preserve value/place identity. Its compiler-mandated
skeleton is:

```text
input:  Type=T, value stage=S,       Pp=S
output: Type=T, value stage=runtime, Pp=S, presence=present
```

Other legal endpoint Policy coordinates belong to the ordinary callable and
its overload declaration. In particular, input/output mutability need not be
equal: `const compile -> mut runtime` may construct a fresh mutable runtime
object when such a candidate is the unique ordinary winner. The compiler
authorizes the stage edge but does not synthesize the candidate's `mut`
capability. Opposite const/mut endpoint Patterns remain fully admissible and
participate in the same actual-relative ordinary Bp order as explicit
parameters/results; mutability is not tested by Policy-domain intersection.
Stage, presence, Pp capability, Type, and structural applicability remain hard
conditions.

As an explanatory model rather than frozen surface syntax, one type Symbol may
carry the pure Pattern member `:t` plus ordinary value members for the four
default transports `const <- const`, `const <- mut`, `mut <- const`, and
`mut <- mut`. More specific Pattern members may refine or delete regions of
that default ordinary relation.

Those ordinary transport members expose complete callable Policies, not
special `compile -> runtime` signatures:

```text
candidate formal P2:
  (compile || runtime):compile

candidate complete result P2:
  (compile || runtime):compile

selected static source
  -> Project_in(complete formal)
  -> ordinary invocation
  -> complete ordinary result
  -> Project_out(runtime demand)
```

The migration adapter selects views around an ordinary call; it does not
rewrite the callable's complete P2 into a migration edge.

Migration still cannot turn `T` into `T ref`, repair a failed Pattern/Type
match, or search an arbitrary operation graph. `ref`, `share`, and `alias`
remain independently selected ordinary mechanical operations. When one of
those operations is explicitly required, its ordinary result may change Type
and Pattern; that is not Policy-demand repair.

Any successful existing-view satisfaction terminates before migration
candidate enumeration. In the currently implemented binding case, a non-empty
ordinary P1 projection makes this call unreachable. An absent-Val1 entry
cannot be passed as migration input. Failure after the unique ordinary winner
is selected cannot reopen the candidate set.

The model does not freeze a special global `transition` Symbol or a new
callable ontology. It freezes complete-choice existing projection followed,
only when that projection is empty and the choice accepts runtime, by one
ordinary function-object call toward the extracted runtime branch. The
operation-to-Symbol mapping remains an implementation/design handoff.

The current `PolicyTransitionCallable` Rust carrier does not implement this
pipeline; it is bounded candidate/result fixture material and must be removed
or reduced to an adapter when ordinary routing is wired. In particular, its
caller-supplied result Pattern proves only that the carrier can transport
fixture data. It does not establish canonical TypeValue/PatternValue/owner/
constructor coherence for an ordinary invocation result. The fixture does,
however, retain the selected callable's complete result Policy separately from
the demanded `Project_out` view, and constructs its provisional ordinary
result before applying that output projection.

## 3. `()` is not an operator

`()` is not an operator. An operator is a callable value with special binding and parsing behavior. Since values are not namespace/type parents, an operator cannot serve as an intermediate navigation node.

`()` is a special type/namespace call entry. It is not itself a callable operator value. It cannot become the parent of another call lookup. It can only appear as a navigation leaf.

## 4. Direct function object call method

For `let f = (self) => {};`, the generated anonymous function-object type `F` has the call method under `F` itself: `() :: F`. Not under `ref::F`, `share::F`, or `move::F`.

A directly defined function object call receives that function object as its
caller/self. Ownership is not written by the user — it is part of the generated
function-object call method.

## 5. User-defined callable objects

```text
Product |> object
  → object value
  → type T = object |> type
  → associated namespace search for `()`
```

User-defined call entries are commonly installed under borrowed associated namespaces (e.g. `() :: ref::T`). The user writes `ref` explicitly; the expression's type becomes `ref::T`, and lookup follows from there. The language does not automatically jump from `T` to `ref::T`.

Direct function objects are not merely sugar for user-defined `ref::T` callables. They have their call method directly under their anonymous function-object type.

The implementation body installed under `()::ref::T` does not thereby acquire
`ref::T` as its lexical owner. Its `CallableOwner` still owns local symbols,
Pattern roots, nested callables, and code identity. `ref::T` is instead the
receiver type of invocation slot 0.

Source navigation is inner-to-outer. `ref::T` therefore means the `ref` child
owned below `T`. A construction currently authorized to add children of `T`
may contribute `ref::T`; the reversed spelling `T::ref` would require modifying
the unrelated outer owner `ref` and is not equivalent.

Likewise, `let ()` inside construction of `T` contributes only `()::T`.
`()` entries below `ref::T` and `share::T` are independent injections. `move`
does not require another call namespace because it is the type fixed point
`T move == T`; borrowing constructs distinct `ref::T` and `share::T` object
types.

### 5.1 First-class field-function closures

`.name` is itself a function-object expression. It normalizes without a
receiver to:

```lang
(self, val: T, ...args) {
    (val, args) |> name::T
}
```

`self` is the generated first formal for the implicitly passed field-function
object. The first explicit call-site argument binds `val` and supplies `T`;
`...args` is a Pattern remainder binding, not a pack type. Consequently
`.name` can be stored and passed independently.
After `.name` becomes that ordinary function-object expression, its origin
grants no call-binding privilege. In particular, for `let d = .name`:

```text
BindingShape(P1 |> .name P2)
  == BindingShape(P1 |> d P2)
```

The surrounding ordinary expression/pipe/product rules alone determine
whether `P2` is a source-product continuation, later target, or legality
repair. The normalizer must not inspect `DotClosureLowering` provenance to
override those rules.

Raw AST may preserve `E.name` as member sugar, but normalization routes it
mechanically through the same `DotClosure(name)` core as `E |> .name`, then
returns the resulting ordinary expression to the existing suffix/space
environment. `E..name(product)` remains separate direct member-call sugar; it
is not removed by the more general `.name` form. No lookup or dispatch occurs
during this normalization.

### 5.2 Associated Val2 functions are ordinary function objects

A struct/type construction may contribute ordinary values to the current
Pattern owner's Val2 namespace:

```lang
let fun = (self_fun, object: T, ...args) => { ... };
```

This does not create a special method kind. Invocation of `fun` has:

```text
slot 0 = the `fun` function object
slot 1 = object
slot 2.. = remaining explicit arguments
```

A virtual-field-like function is the same shape with only `object` after
`self_fun`. Direct member-call sugar such as `object..fun(args...)` and the
first-class `.fun` form ultimately call this ordinary associated value; they do
not turn `object` into slot 0 of `fun`.

The special target `let () = implementation` is different. It installs the
call entry of the current Pattern owner. Invoking an object of that owner type
places the object itself in slot 0. No anonymous wrapper closure value becomes
the caller, and the implementation carrier is not prematurely materialized as
a standalone value.

## 6. Implicit `self`

Every callable, including ordinary, in-place, meta, and compiler-generated
closures, has an implicit first parameter position for the caller object.
When the source writes any formal position, the first written formal explicitly
declares the Pattern/binder for that position. Its spelling is unrestricted;
`self` is conventional rather than reserved.

The selected call entry `()` always receives the value being invoked as
implicit `self`. For a standalone function this value is the function object;
for an associated call entry it may be a `T`, `ref::T`, `share::T`, or another
ordinary callable object. The user cannot manually pass this slot.

The source product contains only the explicit user arguments. `ProductObject`, `ArgProductShape`, and `RawArgShape` represent only the explicit product supplied by the user. They do not contain the implicit `self`.

The implicit `self` belongs to the callable-entry invocation frame, not to the source product.

Declared receiver/parameter policy-pair compatibility applies uniformly to
that complete frame:

```text
InvocationFrame:
  slot 0 = selected caller-object self view
  slot 1..n = explicit argument symbol views

every slot must satisfy its selected associated () entry policy pattern
```

The formal/call-site alignment is:

```text
written formal 0       <- implicitly injected callable object
written formal 1..n    <- explicit call-site Product positions 0..n-1
```

A head with no written formal still has invocation-frame slot 0, but it does
not bind that object to a source Pattern.

No separate self-policy plane is required. Independently, the function
object's available stage view is derived from its result P2:

```text
Stage(P1p) = Stage(P2p)
Stage(P1v) = Stage(P2v) union Stage(P2p)
```

Thus the selected object has the static/runtime view required to supply self;
an optional written P1 prefix merely projects that derived view.

Each written formal parameter takes the callable P2 as its base policy pair.
No formal prefix means exact inheritance. `const let` or `mut let` changes only
the inherited value-mutability Pattern; every stage, presence, and Pattern-side
dimension stays equal to P2. That qualifier remains an overload-order Pattern,
so it must not be implemented by running ordinary binding P1 projection over
the actual and deleting the oppositely qualified candidate early.

Candidate preparation also carries that qualifier outward as the parameter's
const/mut product-order position. It therefore affects selection between
callable objects as well as the effective parameter pair seen after entry.

### 6.1 Callable owner, receiver type, and local pattern construction

Every callable, including an in-place closure, has a parent-linked
`CallableOwner`. This is lexical/code identity. It does not universally
determine the type of slot 0:

```text
CallableOwner(C) != ReceiverType(C) in general

standalone closure default:
  ReceiverType(C) = AnonymousType(CallableOwner(C))

associated () implementation:
  ReceiverType(C) = type carrying the selected call entry
```

The callable-local `Self` symbol may therefore combine:

```text
namespace facet = callable-local semantic space
type facet      = ReceiverType(C)
value facet     = invocation slot 0
```

This does not inject callable-local declarations into the named receiver
type's namespace. Nested owner paths use source navigation order:
current/innermost `Self` first and outermost `Self` last. This spelling is not
identity and does not assert that each receiver type is anonymous. The former
synthetic `__inner_space` / `__inner_namespace` component is removed from
canonical ownership.

A local `struct` evaluated by an ordinary or `compile` callable uses the
current callable owner as its ambient Pattern owner. A `compile` invocation
does not manufacture a meta-style canonical-arguments owner.

An ordinary canonical `meta` invocation is different: symbol construction is
anchored by a parent-linked
`MetaInstanceOwner(callee_symbol, canonical_arguments)`.
Ordinary meta callables still use the implicit-self mechanics described above,
but their returned type construction is rooted in the meta-instance scope.

A compiler-provided `BuiltinPrivilegedAstMetaFunction`, such as `struct` or
`inject`, also has a function object, type, associated `()`, and implicit self,
but may use its specified special owner/scope rule instead of creating an
ordinary externally navigable `MetaInstanceScope`.

## 7. ZST function objects

A function object with no stored environment is normally zero-sized. ZST values
are not move-killed, so a zero-sized function object can naturally be called
multiple times. Reusability follows from the general ZST movement rule.

A capture requirement does not by itself imply a stored field or non-ZST
layout. If representation selection chooses stored state, the resulting object
may be non-ZST and follows ordinary value-passing and ownership rules.

### 7.1 Function-object mutability default

The binding created by `let fn = () => { ... }` has no written mutability
restriction. Its empty typed mutability domain denotes `const || mut`. This is
the neutral, fully available function-object view; it is not copied from P2.
An explicit declaration P1 may restrict that domain to one view. The
namespace-declaration spelling `export let fn = ...` does not change this
complete internal view. Export elaboration separately derives the externally
visible const projection. A written `const || mut` internal view is therefore
valid when its const projection is non-empty; a `mut`-only value export is not.

### 7.2 Ordinary closure capture requirements

Ordinary closures combine source-explicit capture bindings with resolved-stage
automatic const capture:

```text
source [let x = E] / [x = E] -> Explicit capture
source [E] shorthand          -> ExplicitInferredBinder capture
unreplaced resolved free ref  -> ImplicitConst capture
```

`[x]` is the explicit shorthand `[let x = x]`. Because its capture policy is
unwritten, its mutability domain is the neutral `const || mut`; it is not
automatic const capture. A write to an outer source requires an explicit
capture projected to a `mut` view. Automatic capture never grants mutability.

External explicit navigation resolves through the namespace export view.
Internal explicit navigation resolves through the complete namespace view and
does not prove export membership. Exported value views are const-projected, so
an externally navigated callable or value normally satisfies the
`ImplicitConst` capture requirement. Ordinary external calls are therefore
normally backed by automatic const dependencies, not by a source capture list.

Automatic capture and call resolution occupy adjoining problem domains: both
reason about an external symbol identity and its readable const view. This
does not require either mechanism to consume the other's output, share a pass,
or run in a prescribed order. Automatic capture does not skip admissibility or
select a candidate.

Explicit and automatic capture may resolve to the same source symbol but remain
distinct dependency declarations. Explicit capture can rename, project policy,
use a complex initializer, request `mut`, and carry its own diagnostic
provenance. No frontend or capture-discovery step erases it as redundant.
Equivalent storage/link requirements may be coalesced only by a later layout
pass while preserving binder identity, policy, and provenance.

Resolved capture analysis produces abstract dependencies:

```text
ResolvedCaptureRequirement {
  local_binder,
  source,
  requested_policy,
  origin
}
```

This object is not a `self` field list and does not determine receiver mode,
copy/reference representation, field ordering, ZST status, or ABI layout.
Static symbol links, constant embedding, zero-layout dependencies, stack
environments, and stored checked references are possible later lowerings.

For example:

```lang
mut let internal_state = ...;

export let exported_fn =
    [internal_state]() => {
        use internal_state;
    };
```

The explicit dependency may lower to an internal static link. It neither
exports `internal_state` nor requires an address field in every
`exported_fn` object.

Before materialization, each requirement must lower to a lifetime-checkable
form naming its source place, requested access view, origin/region relation,
and storage-or-link category. This is only a handoff obligation; copy/move/
borrow defaults, region construction, escape rules, and ABI remain unfrozen.

### 7.3 In-place closures are embedded callable candidates

An in-place closure is distinguished by
`NormClosure.placement = NormClosurePlacement::InPlace`. Generated provenance
is carried independently by `NormClosure.origin`; it is never a placement
variant. Its
semantic object remains embedded in the control-flow layer at which it is
used; it is not converted into a freely escaping captured closure. It may
nevertheless contribute a normal callable candidate to an overload set.

Head presence is independent of that placement. Bare `{ ... }`,
`() -> r name { ... }`, and `() -> r [[strategy]] { ... }` are all in-place;
the latter two merely preserve a head and optional strategy metadata. `=>`
selects ordinary placement. The parser and normalizer must not infer
placement from `head.is_some()`.

An in-place closure has no capture clause and no capture environment. Reads of
outer symbols do not require `[]`. Instead, unresolved outer reads are carried
as lazy embedding lookups:

```text
definition/materialization:
  unresolved read name -> DeferredEmbeddingLookup(name)

candidate use at control-flow layer L:
  DeferredEmbeddingLookup(name) -> ResolveSymbol(name, L)
  missing at L -> diagnostic at that use
```

Failure to find the symbol at the syntactic closure site is therefore not yet
an error. The lookup becomes final only at the layer where that in-place
candidate is embedded and selected. This is lexical embedding, not textual
macro substitution: local declarations still shadow normally, symbol identity
is used after resolution, and each use is checked in its own embedding
environment.

An in-place closure may not write any symbol/place outside its closure-local
scope:

```text
WriteSet(C) intersect OuterSymbols(C) = empty
```

It may still mutate its own locals, call effectful functions, and use ordinary
capabilities. The prohibition is specifically direct outer-place mutation.
Because an in-place closure has neither a capture list nor an automatic capture
set, there is no syntax or materialization step that can grant an exception.
The resolved embedding check owns this rule. The Normalized AST only preserves
`InPlace` and, for ordinary closures, elaborates the v0.5-A let-shaped capture
syntax. Its free non-call-name inference is shape-directed; it performs no
lookup, capture-environment layout, or capture admissibility analysis.

## 8. Call lookup pipeline

```text
Product |> Expr

1. Shape explicit Product: ProductObject → ArgProductShape → RawArgShape*
2. Resolve a name/path to Symbol and project/enumerate its heterogeneous value facet
3. Expose each Val2 object's policy-pair view for the current `Phase`
4. For each surviving value entry, obtain its type / TypeValueId
5. Find call entry: type(value).associated_namespace → lookup `()`
6. Discard non-callable/non-applicable entries
   while retaining visible derived companion objects
7. Determine receiver binding: caller type `F` / `ref::T` / `share::T` and
   selected associated `()`
8. Build invocation frame: implicit caller/self + explicit shaped product args
9. Form fully admissible set A using all hard checks, including receiver and
   parameter pair compatibility, P2 result compatibility with any target
   expectation, stage legality, and require legality
10. Export every elaborated formal const/mut Pattern to its candidate position,
    apply const/mut product-maximal filtering and the remaining fixed-order
    preference filters, including in-place over non-in-place after the
    first-order-over-instantiated filter, then named strategy rules and the
    must-select check
11. Enter the unique selected invocation or defer according to demand
```

A derived compile companion is a complete `Val2` function object with stable
origin, its own type, and its own associated static `()`. For origin result
`runtime:Qstatic`, that result pair is `Qstatic:Qstatic`. It is not a
lookup-failure fallback. If its prepared candidate enters fully admissible set
`A`, its must-select strategy requires it to be the final unique candidate.
Compile projection preserves an ordinary projected call; normal compile
evaluation later enumerates and selects objects.

## 9. Relation to v0.8 substrate

For v0.8 construction substrate, `NormalizedCallSite.target` is not itself the `()` method — it is the callable object expression. The full pipeline is:

```text
target expression → target value → target type →
  type-associated namespace → `()` call entry
```

If the target expression is a `NormClosure`, the target position is an
explicit materialization consumer. Likewise, a declaration initializer is a
materialization consumer when it binds that carrier. Normalization itself
creates only a closure carrier; an arbitrary surrounding expression does not
eagerly turn the carrier into a value or allocate its environment.

The current implementation uses a documented shortcut (v0.8): the resolved target `SymbolObject` is treated as the callable entry directly, via `ResolvedCallTarget { temporary_direct_callable_shortcut: true }`. This shortcut will be replaced when function-object types and associated call-entry insertion are implemented.

## 10. Invariants

- Function object is a value.
- Every function object has a type.
- A directly defined function object has an anonymous function-object type.
- The call entry `()` for a directly defined function object lives under that anonymous type.
- User-defined callable objects may define `()` under `ref::T` / `share::T` / other associated namespaces.
- `CallableOwner` and receiver type are independent semantic facts.
- Implicit `self` is always the invoked caller object and is passed by the call
  mechanism.
- The self role is positional; the first written formal exposes it under an
  ordinary user-chosen binder/Pattern, and `self` is only a conventional
  spelling.
- Implicit `self` is not part of `ProductObject` / `ArgProductShape`.
- The user cannot manually pass implicit `self`.
- `()` is not an operator.
- Operator values cannot be namespace/navigation parents.
- `()` is a special type/namespace call entry and can only be a navigation leaf.
- Struct-associated named lets contribute ordinary Val2 entries; the special
  empty target `let ()` contributes the current owner's call entry.
- A call-entry owner/first-formal mismatch is handled by ordinary invocation
  type checking, not by a separate declaration validator.
- ZST function objects are reusable because ZST values are not move-killed.
- Non-ZST function objects obey ordinary ownership and passing rules.
- Empty function-object mutability means the unrestricted `const || mut`
  domain; an explicit declaration P1 may crop it. Export does not crop the
  complete namespace-internal domain. A value-bearing external candidate view
  projects that domain to const; `const || mut` is valid, while mut-only has no
  external candidate view.
- Written formal parameters inherit P2 exactly outside the optional const/mut
  Pattern axis.
- Ordinary closures distinguish explicit, explicit-inferred-binder, and
  implicit-const capture requirements; those requirements do not define
  `self` fields or physical layout.
- In-place closures may be overload candidates, have no capture clause or
  automatic capture set, defer unresolved outer reads to their embedding
  layer, and are forbidden from writing outer symbols/places.
- Ordinary and built-in privileged meta functions follow the same
  function-object and implicit-self call model.
- Ordinary/compile local pattern construction uses the function-object internal
  Self frame; compile does not create a MetaInstanceScope.
- Ordinary meta symbol construction is anchored by canonical MetaInstanceScope;
  built-in privileged AST meta functions may instead use their declared special
  scope/owner rule.
- `.name` is a first-class closure expression whose normalization produces a
  generated in-place `NormClosure` carrier. Binding or explicit call context
  may materialize that carrier; normalization does not. `E.name` uses the same
  carrier, while `..name` remains direct member-call sugar.
- Callable-tail named strategy metadata operates only on fully admissible
  candidates and cannot reopen ordinary overload enumeration.
