# Function Objects and Call Projection

Status: canonical call semantics. Consumer gaps are tracked in the roadmap.

## 1. Basic thesis

A callable is an ordinary complete function object. A name denotes a named type
whose V_tau is synthesized by its named contributions. Explicit OverloadGroups
aggregate type candidates using the singleton embedding eta(T).

    Type(callee) = Type(first self)
    CallCandidates(T) = CallCandidates(V_tau(T))
    CallCandidates(G) = disjoint_union over T in G of CallCandidates(T)

The [name/type algebra](names-and-overload-groups.md) owns these projections
and the distinct update algebras. There is one ordinary overload pipeline;
neither a second binding facet nor a defining-name lookup supplements a
complete type's immutable callspace. Non-callable members contribute no call
candidates, and an empty projection never restarts name lookup.

Direct function expressions produce a complete anonymous type and a function
object with its associated () entry. Its own object occupies first self.
Captures, parameters, result demands and Policy participate through the
existing ordinary rules.

## 2. Pipeline call form

    Product |> expression
      -> resolved value / named type / explicit candidate group
      -> ordinary call projection
      -> exact captured type and associated ()
      -> ordinary admissibility and preference
      -> unique sealed invocation
      -> DynamicLegality and execution

The explicit Product excludes implicit self. Distinct contributions are not
erased by content interning; bucket aggregation and ordinary candidate identity
follow their own rules. A selected failure never reopens selection.

### 2.1 Compiler-inserted atomic runtime migration call

The language-authorized static-value-to-runtime-value migration is a
compiler-inserted use of this same call trunk, not a second callable kind:

```text
consumer demand
  -> project the complete accepted Policy choice over existing views
  -> if successful, use it and stop
  -> otherwise, if the query accepts runtime, extract its runtime branch
  -> select an existing static source Policy view
  -> from the held source PatternValue, enter its resolved Pattern owner
  -> enumerate the language-authorized associated Val2 family
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
its overload declaration. In particular, input/output `PolicyMode` need not be
equal: `const compile -> mut runtime` may construct a fresh runtime result
whose output slot has `PolicyMode = mut` when such a candidate is the unique
ordinary winner. This does not imply `Writable(result)`. The compiler
authorizes the stage edge but does not synthesize the candidate's `mut`
capability. Opposite const/mut endpoint Patterns remain fully admissible and
participate in the same actual-relative ordinary Bp order as explicit
parameters/results; mode is not tested by Policy-domain intersection.
Stage, presence, Pp capability, Type, and structural applicability remain hard
conditions.

As an explanatory model rather than frozen surface syntax, one type name binding may
carry the pure Pattern member `:t` plus ordinary value members over all nine
`output PolicyMode <- input PolicyMode` coordinates. Every coordinate is
expressible, but no coordinate is required to exist: each may be absent or
realized by `default`, `delete`, or `custom`. More specific Pattern members may
refine or delete regions of that capability relation. This 3×3 relation is not
the three-point Policy preference order.

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

Policy migration selects views around an ordinary call; it does not rewrite
the callable's complete P2 into a migration edge.

Migration still cannot turn `T` into `T ref`, repair a failed Pattern/Type
match, or search an arbitrary operation graph. `ref` and `share` remain
independently selected ordinary mechanical operations. When one of those
operations is explicitly required, its ordinary result may change Type and
Pattern; that is not Policy-demand repair.

Any successful existing-view satisfaction terminates before migration
candidate enumeration. In the currently implemented binding case, a non-empty
ordinary P1 projection makes this call unreachable. An absent-Val1 entry
cannot be passed as migration input. Failure after the unique ordinary winner
is selected cannot reopen the candidate set.

The model does not freeze a special global `transition` name binding or a new
callable ontology. It freezes complete-choice existing projection followed,
only when that projection is empty and the choice accepts runtime, by one
ordinary function-object call toward the extracted runtime branch. The
connected atomic slice begins from the source value's existing PatternValue
owner and its associated `()` Val2; independently authorized mechanical
operations may select other ordinary associated families without changing
this demand rule.

The connected `lang_build` slice now implements this routing for source
function objects and toolchain-source associated `()` entries:

```text
Semantic name binding / Pattern owner
  -> CallCandidates(NamedType(S))
  -> TypeValue
  -> PatternValue / ResolvedPatternScope
  -> associated ()
  -> PreparedCallCandidate
  -> InvocationFrame
  -> ordinary result entries
```

The source-name binding and already-held-Pattern entrances merge at the same
candidate pipeline. The latter carries an explicit semantic receiver and does
not fabricate a source path or require migration metadata.

Name-based source calls and compiler-authorized operations therefore have
different candidate entrances but one ordinary call trunk. Neither entrance
requires `TypeValue -> original carrier name binding`: source navigation resolves a
name binding and reads its values, while an already-held complete type value carries
its own callspace:

```text
TypeValue(t) = tau = <Q, V_τ>
Core(tau) = Q
CallSpace(tau) = V_τ

Candidates(args |> t) = CallSpace(TypeValue(t)) = V_τ
```

Copied/extracted type-as-callee lookup selects candidates from that immutable
`V_τ` snapshot. Complete type values require no defining-name binding
recovery, most-recent carrier, or reverse `AsType` candidate entrance.

Associated source navigation obeys the same forward-only rule. If:

```lang
let T: type = uint8;
```

then a target selected through `T` resolves T to one terminal NameBinding and reads the complete
type snapshot. Type-as-callee candidates come from its `V_τ`; an ordinary
navigated member may additionally be selected through `Q`/`Val2` under the
normal navigation rules. Neither path inspects provenance or searches for
uint8's defining binding.

Migration callables obtain their result Pattern and complete type observations
from registered semantic entities. Constructor/extractor materialization and
place lowering consume those observations through the ordinary call boundary.

For source-backed transport members, the callable/member value Policy,
first-formal Policy Pattern, and complete result P2 remain distinct. The member
Policy supplies the migration output endpoint coordinate, the formal inherits
the complete P2 and supplies input fit, and ordinary invocation preserves the
complete P2 until the later demanded `Project_out`.

### 2.2 Derived associated forwarding is an ordinary call

A derived associated forwarder is an ordinary callable. After it is uniquely
selected, its body may invoke another associated family. That inner invocation
is a new ordinary call and does not reopen the candidate set of the outer
call:

```text
resolve name::D(T)
-> select forwarder uniquely
-> execute forwarder body
-> body performs a new ordinary invocation of name::T
```

This covers `field::(T ref) -> field::T` and `field::(T share) -> field::T`
(canonical `ForwardAssoc` in
`type-associated-function-objects-and-access-trees.md`), as well as any future
derived-type forwarding. It is not fallback, not candidate reopening, and not
late adaptation.

### 2.3 Compiler-authorized stage migration vs explicit `const`/`mut` reconstruction

The compiler-authorized stage migration of §2.1 (static-value-to-runtime-value)
and the explicit `const` / `mut` reconstruction of
`symbol-policy-and-compile-flow-projection.md` §1.2 are distinct:

```text
compiler-authorized stage migration
≠
explicit const/mut reconstruction
```

Both may reuse `T`'s ordinary construction/call family, but their triggers
differ. Stage migration is inserted by the compiler when an existing-view
projection is empty and the demanded stage accepts runtime; explicit
`const`/`mut` is a user-visible reconstruction demand. Neither turns `T` into
`T ref`/`T share`, neither reopens a candidate set after selection, and both
obtain their conversion capability from the complete `τ`'s callspace
(`CallSpace(τ) = V_τ`), never from defining-name binding or carrier-provenance
recovery.

## 3. `()` is not an operator

`()` is not an operator. An operator is a callable value with special binding and parsing behavior. Since values are not namespace/type parents, an operator cannot serve as an intermediate navigation node.

`()` is a special type/namespace call entry. It is not itself a callable operator value. It cannot become the parent of another call lookup. It can only appear as a navigation leaf.

## 4. Direct function object call method

For `let f = (self) => {};`, the generated anonymous function-object type `F` has
one associated call name binding `()`. Receiver observation is expressed by candidate
formals, not by generated `ref::F`, `share::F`, or `move::F` namespaces.

A directly defined function object call receives that function object as its
caller/self. Ownership is not written by the user — it is part of the generated
function-object call method.

## 5. User-defined callable objects

For a value x of type T, invocation uses T's captured callspace and supplies x
as first self. A ref/share decorated value has its own exact type and therefore
its own matching () entry. There is no coercion of T into T ref/share to repair
a first-self mismatch, and those distinct callee types are not one receiver
exception under T's call entry.

CallableOwner still owns local names, Pattern roots, nested callables and code
identity. It is not inferred from a parameter's spelling. Member contribution
requires the closure type to belong to the destination core; eligible closure
expressions can create a new anchored instance under
[replication](closure-anchored-replication.md), without changing the original.

A receiving construction can call ordinary compile logic from A[t] with its
own mutable type reference. That reference is an explicit argument, after the
selected function object's self. Ordinary field forwarding already uses this
same distinction and needs no special open-world contribution mechanism.

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
for an associated call entry it may be a `T`, `T ref`, `T share`, or another
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
With no formal prefix, the pair component inherits P2 exactly while the
unwritten whole-slot mode elaborates to the concrete `plain` point. `const let`
or `mut let` changes only that whole-slot `PolicyMode`; every stage, presence,
and Pattern-side dimension stays equal to P2. That qualifier remains an
overload-order Pattern, so it must not be implemented by running ordinary
binding P1 projection over the actual and deleting the oppositely qualified
candidate early.

Candidate preparation also carries that qualifier outward as the parameter's
three-point product-order position. It therefore affects selection between
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

The callable-local `Self` frame may therefore combine these local projections:

```text
namespace role          = callable-local semantic space
receiver-type projection = ReceiverType(C)
caller value slot       = invocation slot 0
```

These are callable-frame labels, not independently stored name binding facets in the
target Object ontology.

This does not inject callable-local declarations into the named receiver
type's namespace. Nested owner paths use source navigation order:
current/innermost `Self` first and outermost `Self` last. This spelling is not
identity and does not assert that each receiver type is anonymous. Canonical
ownership contains no synthetic `__inner_space` or `__inner_namespace`
component.

A local `struct` evaluated by an ordinary or `compile` callable uses the
current callable owner as its ambient Pattern owner. A `compile` invocation
does not manufacture a meta-style canonical-arguments owner.

An ordinary canonical `meta` invocation is different: symbol construction is
anchored by a parent-linked
`MetaInstanceOwner(callee_symbol, canonical_arguments)`.
Ordinary meta callables still use the implicit-self mechanics described above,
but their returned type construction is rooted in the meta-instance scope.

A compiler-provided `BuiltinPrivilegedAstMetaFunction`, such as `struct`,
`extend`, or `inject`, also has a function object, type, associated `()`, and implicit self,
but may use its specified special owner/scope rule instead of creating an
ordinary externally navigable `MetaInstanceScope`.

## 7. ZST function objects

A function object with no stored environment is normally zero-sized. ZST values
are not move-killed, so a zero-sized function object can naturally be called
multiple times. Reusability follows from the general ZST movement rule.

A capture requirement does not by itself imply a stored field or non-ZST
layout. If representation selection chooses stored state, the resulting object
may be non-ZST and follows ordinary value-passing and ownership rules.

### 7.1 Function-object PolicyMode default

The binding created by `let fn = () => { ... }` has an unwritten mode spelling,
which elaborates to the real `plain` point, not a `const || mut` choice and not
an inference variable. The mode is not copied from P2. An
explicit declaration P1 may select another mode. The
namespace-declaration spelling `export let fn = ...` does not change this
complete internal view. Export elaboration derives a stable, identity-preserving
`Σ_export` from export retention and public path reachability; it neither
filters candidates by a future consumer demand nor projects the mode to
`const`.

### 7.2 Ordinary closure capture requirements

Ordinary closures combine source-explicit capture bindings with resolved-stage
automatic eligible capture:

```text
source [let x = E] / [x = E] -> Explicit capture
source [E] shorthand          -> ExplicitInferredBinder capture
unreplaced resolved free ref  -> ImplicitEligible capture
```

`[x]` is the explicit shorthand `[let x = x]`. Because its capture mode is
unwritten, it is `plain`; capture does not silently replace it with `const`.
Write capability remains a separate family-specific capability and is not
implied merely by selecting `mut`.

External explicit navigation resolves through the stable namespace export view;
the resulting capture requirement later enters ordinary capability-family and
Policy checks.
Internal explicit navigation resolves through the complete namespace view and
does not prove export membership. Eligibility and visibility do not alter the
selected slot's `PolicyMode`; ordinary external calls therefore do not acquire
an automatic const dependency merely by crossing the boundary.

Automatic capture and call resolution occupy adjoining problem domains: both
reason about an external symbol identity and its stable external view. This
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
  required_access_capability,
  origin
}
```

Namespace lookup supplies the stable source candidate set; it does not consume
these request fields. The later ordinary capture-legality consumer checks both
the requested Policy demand and required access capability.

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
and storage-or-link category. This is only a handoff obligation. Automatic
mechanical move-vs-copy selection, concrete borrow/copy representation, Region
IR construction, escape-check implementation, and ABI remain open; entry
origin defaults, the exact move-origin/Region boundary, and the selected
share/rebind-plus-clone realization lifecycle-post boundary are closed by the
lifetime owner; `CopyConstruct` adds no default origin equation.

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
  DeferredEmbeddingLookup(name) -> Resolve(name, L)
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
WriteSet(C) intersect OuterBindings(C) = empty
```

It may still mutate its own locals, call effectful functions, and use ordinary
capabilities. The prohibition is specifically direct outer-place mutation.
Because an in-place closure has neither a capture list nor an automatic capture
set, there is no syntax or materialization step that can grant an exception.
The resolved embedding check owns this rule. The Normalized AST only preserves
`InPlace` and, for ordinary closures, elaborates the let-shaped capture
syntax. Its free non-call-name inference is shape-directed; it performs no
lookup, capture-environment layout, or capture admissibility analysis.

## 8. Call lookup pipeline

```text
Product |> Expr

1. Shape explicit Product: ProductObject → ArgProductShape → RawArgShape*
2. Resolve a name/path to name binding `S`; form `C0 := CallCandidates(NamedType(S))`
   and enumerate that candidate set (one step, no priority, no fallback, no
   reopening; the same candidate reachable through both paths is deduplicated)
3. Expose each Val2 object's policy-pair view for the current `Phase`; for
   ordinary result evaluation, derive the candidate-local P1 stage view from
   its P2 under that phase
4. For each surviving value entry, obtain its type / TypeValueId
5. Find call entry: type(value).associated_namespace → lookup `()`
6. Discard non-callable/non-applicable entries
   while retaining visible derived companion objects
7. Determine receiver binding: caller type `F` / `T ref` / `T share` and
   selected associated `()`
8. Build invocation frame: implicit caller/self + explicit shaped product args
9. Form fully admissible set A using all hard checks, including receiver and
   parameter pair compatibility, phase legality of the P1-stage-follow-P2
   default, P2 result compatibility with any explicit target
   pair/type/rank/facet expectation actually supplied, and require legality
10. Export every elaborated formal PolicyMode Pattern to its candidate position,
    add the always-present `OutputModeDemand(call)` (`plain` when no
    candidate-independent immediate-consumer demand exists), apply PolicyMode
    product-maximal filtering and the remaining fixed-order
    preference filters, including in-place over non-in-place after the
    first-order-over-instantiated filter, then named strategy rules and the
    must-select check
11. Enter the unique selected invocation or defer according to demand
```

Every nested producer actual is closed under the canonical
`CallLocalPolicyClosure` before an unresolved candidate of the current outer
call can influence it. A nested call uses an already-formed,
candidate-independent immediate-consumer output demand when one exists;
otherwise it uses local `plain`. Its selected concrete result mode is then an
ordinary actual fact for this pipeline. Outer ambiguity or failure never
reopens the nested producer.

The evaluation phase is a separate, already-known input. In the absence of an
explicit target-result pair/stage constraint, each candidate's default
evaluation P1 stage view follows its P2 through the canonical stage lift and is
checked against the current phase. Therefore `compile`/`runtime` evaluation is
not gated on the presence of `PolicyLet`. This default does not derive
PolicyMode: an unwritten output mode remains `plain`, while an explicit
`const`/`mut` result context is a manual demand.

`PolicyLet(P, e)` is the explicit expression boundary that may provide such a
candidate-independent demand. It is optional for the phase-derived default:
`compile let e` or `runtime let e` explicitly delimits/narrows the stage
context, while `const let e` or `mut let e` explicitly replaces the default
plain Mode demand. Its complete operand pipe is resolved once under `P`, then
`SourcePolicy(result) -> P` enters the ordinary Policy migration candidate
preparation and unique Policy-overload selection. The selected migration
jointly produces the concrete Policy projection and value realization in the
node's ordinary expression-result slot. That slot has its own mode but is not a
NameBinding, name binding, declaration, or independently addressable Place. A later
outer candidate cannot propagate a formal-mode preference through the
preserved `PolicyLet` node. The node is not an ordinary Val2 call or a hidden
binding.

A derived compile companion is a complete `Val2` function object with stable
origin, its own type, and its own associated static `()`. For origin result
`runtime:Qstatic`, that result pair is `Qstatic:Qstatic`. It is not a
lookup-failure fallback. If its prepared candidate enters fully admissible set
`A`, its must-select strategy requires it to be the final unique candidate.
Compile projection preserves an ordinary projected call; normal compile
evaluation later enumerates and selects objects.

The semantic source of a compile companion is derivation, not name binding
injection. The compile realization is defined only for the stage that admits
one — a runtime generic callable — and is undefined for the other stages:

```text
CompileRealization(F)
  = C(F)        if Stage(F) = runtime
  = F           if Stage(F) = compile
  = undefined   if Stage(F) = meta
    -- a partner operation is undefined for meta callables

DistinctCompilePartner(F)
  iff Stage(F) = runtime
  -- equivalently: CompileRealization(F) = C(F) != F

CompilePartner(F) = C(F)   -- defined exactly when DistinctCompilePartner(F)

C(n) = n  with produced-runtime-Val1 := absent
         if ManufacturesRuntimeVal1(n)
C(n) = n  otherwise
C(F) = Resolve(CompileTransform(body(F)))
```

`CompileTransform(body(F))` rewrites the callable body's result
classification so that a runtime-value-producing body instead produces its
static result (absent runtime `Val1`), leaving the callable structure,
receiver, and associated static `()` intact. The compile companion's existence
is a fact about `F` under the compile transform, and only about a runtime
callable: a compile generic `F` has no distinct compile partner, and a meta `F`
has none either (its realization is `F` itself in both cases). This matches the
partner classification in meta-object-invocation's Meta-instance identity section: runtime generic `F`
has `C(F)` plus `M(F)`; compile generic `F` has only `M(F)`; meta `F` has
neither. A host name binding's symbol-facet
entry for the companion (overload-resolution §3.3) is a lowering/implementation
cache, not the semantic cause: removing the cache entry does not remove
`CompilePartner(F)`, and `C(F)` never becomes a candidate by virtue of that
entry alone.

## 9. Normalized call-site handoff

`NormalizedCallSite.target` is the callable object expression, not the `()`
member. The full pipeline is:

```text
target expression → target value → target type →
  type-associated namespace → `()` call entry
```

If the target expression is a `NormClosure`, the target position is an
explicit materialization consumer. Likewise, a declaration initializer is a
materialization consumer when it binds that carrier. Normalization itself
creates only a closure carrier; an arbitrary surrounding expression does not
eagerly turn the carrier into a value or allocate its environment.

## 10. Invariants

- Function object is a value.
- Every function object has a type.
- A directly defined function object has an anonymous function-object type.
- The call entry `()` for a directly defined function object lives under that anonymous type.
- Each value/ref/share callee type supplies an associated () with matching first self.
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
- Each selected call entry obeys exact callee/first-self type equality under
  ordinary invocation checking.
- ZST function objects are reusable because ZST values are not move-killed.
- Non-ZST function objects obey ordinary ownership and passing rules.
- An unwritten function-object mode is `plain`; an explicit declaration P1 may
  select another mode. Export preserves the complete namespace-internal mode
  and filters external candidates through independent capability/visibility
  eligibility rather than a universal const projection.
- Written formal parameters inherit P2 exactly outside the optional whole-slot
  PolicyMode axis.
- Ordinary closures distinguish explicit, explicit-inferred-binder, and
  implicit-eligible capture requirements; those requirements do not define
  `self` fields or physical layout.
- In-place closures may be overload candidates, have no capture clause or
  automatic capture set, defer unresolved outer reads to their embedding
  layer, and are forbidden from writing outer symbols/places.
- Ordinary and built-in privileged meta functions follow the same
  function-object and implicit-self call model.
- Ordinary/compile local pattern construction uses the function-object internal
  Self frame; compile does not create a MetaInstanceScope.
- Ordinary meta construction is anchored by the canonical MetaInstance anchor
  `M` (a symbolic-navigation layer); its default result is `τ_M` rooted at `M`;
  built-in privileged AST meta functions may instead use their declared special
  scope/owner rule.
- `.name` is a first-class closure expression whose normalization produces a
  generated in-place `NormClosure` carrier. Binding or explicit call context
  may materialize that carrier; normalization does not. `E.name` uses the same
  carrier, while `..name` remains direct member-call sugar.
- Callable-tail named strategy metadata operates only on fully admissible
  candidates and cannot reopen ordinary overload enumeration.
