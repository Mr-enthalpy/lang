# Function Objects and Call Projection

Status: canonical call semantics. Consumer gaps are tracked in the roadmap.

## 1. Basic thesis

Ordinary value invocation and complete-type invocation have different projection
entrances into the same overload and execution relations.

```text
Val1?(x) != absent
T = Type(x)
CallCandidates_ordinary(x)
    = Entries(AssociatedNamespace(T).Val2[()], actual_self=x)
CallOrdinary(x)
    -> Type(x) -> AssociatedNamespace(Type(x)).Val2[()] -> selected Impl
self = x

tau = bind alpha.<Q, V_tau[alpha]>
CallCandidates_type(tau) = CallCandidates(V_tau)
CallType(tau)
    -> select c in V_tau
    -> CallOrdinary(c)
    -> Type(c) -> AssociatedNamespace(Type(c)).Val2[()] -> selected Impl
self = c

CallCandidates(G)
    = disjoint_union over tau in G of CallCandidates_type(tau)
Type(actual callee) = Type(first self)
```

Entries denotes the implementation family offered to candidate preparation,
not an additional selection stage. Type projection retains c and its associated implementation
entries in the same candidate pipeline; it never executes speculative bodies
or chooses a runner-up after selected failure.

`V_tau != Val2` is an operational distinction. An ordinary x enters through
its type's associated Val2 call entry; it does not first project Type(x).V_tau.
A type callee uses its own V_tau to supply actual callable values, then those
values use the ordinary entrance. Neither entrance requires a defining-name
lookup or a self-construction witness. An empty projection does not restart
name resolution.

The [name/type algebra](names-and-overload-groups.md) owns type/group aggregation
and their distinct update algebras. V_tau registration, Val2 residency, Pattern
registration and ConstructEdge are independent judgments. ConstructEdge and
TypeRole describe structural construction roles, not type-callee projection.

Every closure expression's legal completed evaluation returns a complete tau_C,
whether ordinary, in-place, generated, file-level or local. Formation is ordinary
struct construction of head/body/dependency material; equivalent material can
participate in extend/inject under their own premises. See
[construction](symbol-first-meta-construction-and-pattern-injection.md#211-v_τ-closure-materialization-derived-semantics).
This is not an extra conversion of a type into a value, nor a promise of readiness
or lifetime permission in every context.

```text
Eval(ClosureExpr_C) = tau_C
c_C in V_tau_C
A_C = Type(c_C)
Home(A_C) = TypeMemberScope(tau_C)
AssociatedNamespace(A_C).Val2[()] = Impl_C
Impl_C = existing terminal implementation leaf

tau_C -> V_tau_C -> c_C -> A_C -> Val2[()] -> Impl_C
```

The first c_C is formed in that same construction, not by recursively creating
another closure or later bootstrapping an arbitrary x:tau_C. TypeRole, named
residency, callability and Pattern registration retain their distinct judgments.
Call projection selects c_C and supplies its exact self; it does not replace
all self types with type. User-defined x:T with direct () still receives x.

Ordinary let binds the completed tau without a wrapper. In-place syntax uses
automatic dependency formation (§7.3). Its result supports ordinary value
operations, subject to actual dependency, capability and lifetime checks.
General [dependencies](dependency-observation-and-realization.md) precede capture
and layout. Closure lifetime propagation is explicitly handed to the lifetime
owner, not inferred from tau, Core equality, ZST or placement.

## 2. Pipeline call form

    Product |> expression
      -> resolved value / named type / explicit candidate group
      -> ordinary call projection
      -> ordinary value: Type(x).associated Val2[()], self=x
         type: V_tau supplies c, then Type(c).associated Val2[()], self=c
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

Those ordinary transport members have concrete declared input/output endpoints
or ordinary stage holes solved to concrete endpoints. A selected compile:compile
input and runtime:compile output remains an ordinary callable declaration;
no completed formal or result has a runtime/compile stage union.

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

For `let f = (self) => {};`, the expression returns tau_C and f binds it.
Call projection selects the ordinary c_C formed within that tau; its associated
() leaf receives c_C as self with Type(c_C)=A_C. Ownership is fixed by the
construction context, never inferred back from f. The implementation leaf
terminates this formation rather than recursively allocating another closure.

## 5. User-defined callable objects

For an ordinary value x of type T, invocation selects the applicable () entry
in AssociatedNamespace(T).Val2 and supplies x as first self. It does not
project V_T. T may provide an applicable () even when V_T is empty. A ref/share
decorated value has its own exact type and therefore
its own matching () entry. There is no coercion of T into T ref/share to repair
a first-self mismatch, and those distinct callee types are not one receiver
exception under T's call entry.

CallableOwner still owns local names, Pattern roots, nested callables and code
identity. It is not inferred from a parameter's spelling. Member contribution
requires Home(TypeOf(v)) = TypeMemberScope(T) for the complete destination T; eligible closure
expressions can create a new anchored instance under
[replication](closure-anchored-replication.md), without changing the original.

A receiving construction can call ordinary compile logic read from the derived
instance t |> A through its ordinary Val2 group member, with its own mutable
type reference. That reference is an explicit argument, after the
selected function object's self. Ordinary field forwarding already uses this
same distinction and needs no special open-world contribution mechanism.

### 5.1 Ordinary ADL field forwarding

`.name` denotes `name::adl`; `E.name` uses that same entry through the
ordinary Product/call binding spine. The [generative declaration owner](../patterns-overload/operator-patterns-and-generative-declarations.md)
defines its ordinary requested-name forwarder. The receiver object remains an
explicit argument after the selected callable's own self.

The normalizer preserves the dot/name/path source role; it has no semantic
authority to manufacture the forwarding implementation. Its current
DotClosureLowering carrier is pending replacement. Generated provenance never
changes suffix/pipe/Product association or first-product-only binding.
`..name` retains its distinct direct-call surface.

Once F is selected, its established P1 and P2 jointly constrain its terminal
inner call G. The ReturnPattern/Pout directly supplies G's immediate result
demand before maxima; no ordinary temp binding is inserted. H(G(...)) still
closes G independently of unresolved H formals. Registered structural extraction
retains StructuralDefault; custom ordinary ADL cannot redefine real fields.

### 5.2 Associated Val2 functions are ordinary function objects

A struct/type construction may contribute ordinary values to the current
Pattern owner's Val2 namespace:

```lang
let fun = (self_fun, object: T, ...args) => { ... };
```

This does not create a special method kind. Invocation of `fun` has:

```text
fun binds the completed tau_fun; ordinary selection reaches c_fun
slot 0 = the actual selected c_fun callable, of A_fun = Type(c_fun)
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
  slot 1..n = explicit argument value views

every slot must satisfy its selected associated () entry policy pattern
```

The formal/call-site alignment is:

```text
written formal 0       <- implicitly injected callable object
written formal 1..n    <- explicit call-site Product positions 0..n-1
```

A head with no written formal still has invocation-frame slot 0, but it does
not bind that object to a source Pattern.

No separate self-policy plane is required. P1/P2 are independent. For
omitted ordinary P1 stage, runtime P2 defaults to runtime, seal to seal, and
compile to compile; meta qualification follows its own owner. Explicit P1
is never overwritten. The completed view must supply legal self material.

Pin = ElabIn(P2, Delta_in). Omission inherits the applicable base; explicit
plain/const/mut refines mode, and explicit stage atoms/holes constrain stage.
Thus a runtime callable may have heterogeneous compile and runtime Pins.
Pout = ElabOut(P1, Delta_out), with Pout.stage = P1.stage. A mode qualifier
remains an overload-order Pattern, not ordinary binding P1 projection that
deletes an oppositely qualified actual before ranking.

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

An ordinary meta invocation constructs an ordinary result name under
MetaInstanceRoot(parent, callee, CanonicalizeInvocationInputs(In)). Its direct
result is the instance type tau_M rooted at M; arbitrary values and borrows may
be ordinary Val2 payloads. P1 meta retains the instance under OpenHere; plain let
completes and closes it. Member value/ref observation and migration are ordinary. The
[invocation owner](../meta-invocation/meta-object-invocation-and-policy-reduction.md)
defines result-name identity, dependency-derived openness and residency/cache
reuse. All of these use the implicit-self mechanics above.

A compiler-provided `BuiltinPrivilegedAstMetaFunction`, such as `struct`,
`extend`, or `inject`, also has a function object, type, associated `()`, and implicit self,
but may use its specified special owner/scope rule instead of creating an
ordinary externally navigable `MetaInstanceScope`.

## 7. ZST function objects

A function object with no stored environment is normally zero-sized.
Representation alone proves neither Killable nor Movable nor Copyable.
Reusable movement requires the instance-level Preserve proof and ordinary
frontier legality defined by the lifetime owner. Stored capture layout and
type equality do not replace those judgments.

### 7.1 Function-object PolicyMode default

The binding created by `let fn = () => { ... }` has no written mode override.
Inherited/contextual constraints are considered before a separate applicable
DefaultModeCompletion can choose plain. Omission is neither an explicit plain
constraint nor a deduction hole; P1 mode is not generally copied from P2. An
explicit declaration P1 supplies its written constraint. The
namespace-declaration spelling `export let fn = ...` does not change this
complete internal view. Export elaboration derives a stable, identity-preserving
`Σ_export` from export retention and public path reachability; it neither
filters candidates by a future consumer demand nor projects the mode to
`const`.

### 7.2 Ordinary closures consume general dependencies

The [dependency owner](dependency-observation-and-realization.md) starts from
Needs(action,source,observation,Gamma,Sigma), distinguishes requirement,
semantic realization and physical representation, and owns projection and
once-per-formation laws.

`[let x=E]`, `[x=E]` and legal `[E]` shorthand are explicit let-shaped
dependency formation. `[x]` implies no const, borrow or write grant. Resolved
free external observations may use an eligible implicit realization under
ordinary lookup, Policy/access and lifetime checks. Capture is one surface
consumer, not the ontology of every function/type/host/Path dependency.

Initializers share the pre-capture name environment, not concurrent effects.
They execute only at the reached formation occurrence, never once per body
invocation or projection. Snapshot versus live-place behavior is decided by
semantic realization before layout. Actual owned state participates in ordinary
structure and identity; refs retain targets/generations. Neither hidden
semantic side tables nor mandatory public self.Val2 fields are allowed.

Explicit and implicit declarations remain distinct even for the same source.
A layout optimization may coalesce equivalent storage only preserving semantic
binder, view and source obligations. Copy/replication preserves formed
dependencies without rerunning initializers, relookup or extending lifetimes.
Runtime formation residue is retained when not ready.

### 7.3 In-place syntax uses automatic dependency formation

`NormClosure.placement = InPlace` records a source formation form. It is not
a non-transferable value category or a post-formation permission flag.
Head presence and generated provenance remain separate syntax facts: bare
blocks and headed bodies without => use this form; => selects ordinary syntax.
The parser does not infer placement from head presence.

```text
FreeExternalObservation(C, x)
    => Needs(Form(C), x, observation, Gamma, Sigma)
DependencyRequirement -> DependencyRealization
ClosureFormation(C) = Struct(Head_C, Body_C, DependencyMaterial_C)

DependencyMaterial_C:
    explicit clause -> ExplicitFormation(C)
    in-place form   -> AutomaticFormation(C)

Eval(InPlaceClosure_C) = tau_C
```

Free external observations are handled at formation through ordinary resolution
and dependency realization. The chosen realization may retain owned material,
ref/share, a stable link or another existing legal dependency relation. Each
requires its own admissibility, access, capability, readiness and lifetime facts.
Automatic formation alone creates none of those permissions.

After formation, invocation consumes the established dependencies. It does not
resolve external names again by spelling or create a separate embedding
environment. Deferred formation retains its actual obligations; deferral is
not permission to recapture at a later invocation site.

Bind, Move, Copy, Return, Store and Pass are ordinary value operations on the
result. Source placement alone neither forbids them nor grants their concrete
legality. In particular, let f = { ... }; is not rejected merely because its
initializer used in-place syntax. Wrapping the result in Product, group or tau
preserves actual dependencies and lifecycle obligations, not a hidden source
restriction.

For a result f retaining reference r, return checks Pre(Return,f),
LifetimeLegal(Dependencies(f), destination), EscapeLegal(f,destination) and
ValidRegion(r) as applicable. A dependency-free result is not unreturnable due
to its syntax; it still obeys all otherwise applicable ordinary checks.

Outer writes likewise depend on the selected dependency/access/capability and
lifetime judgments. AutomaticDependencyFormation does not imply WriteAuthority.
If the actual realization legally supplies write capability, an InPlace tag
cannot veto it again. There is no blanket empty outer WriteSet rule.

Explicit capture syntax remains distinct from automatic dependency formation.
The frontend preserves syntax and performs no semantic lookup or environment
allocation. The existing independent overload preference for in-place candidates
is unchanged; it does not establish a binding, transfer or write prohibition.

## 8. Call lookup pipeline

```text
shape explicit Product; retain implicit self in invocation frame
resolve target once -> ordinary Val2[()] entrance / type V_tau projection / group
retain actual callable x or c -> its type's associated Val2[()]
apply pre-C0 family filter -> enumerate C0
R_vis(c,Omega,sigma) -> candidate plus visibility/input/projection evidence
prepare ordinary C_sigma(c) where required
FullyAdmissible A including total output demand and require
fallback suppression D
Policy product maxima (including migration endpoints when applicable)
Pattern specificity / first-order / in-place / named filters
unique Selected=(c*,sigma*,frame)
DynamicLegality -> Ready execution or retained continuation
```

The two rounds are owned by
[Policy §12](symbol-policy-and-compile-flow-projection.md#12-unified-binding-and-overload-selection).
Round one executes no speculative body or migration. Round two is ordinary
overload selection. Unresolved projection evidence retains a continuation.
Each nested producer seals under candidate-independent immediate-consumer
demand before an unresolved outer candidate can influence it. Delete,
selected extraction, migration, lifetime and execution failure are terminal.

PolicyLet is an optional explicit result-demand boundary. Its complete operand
is selected once; existing-first same-Type migration then provides a coherent
projection and realization. Its result slot is not a hidden NameBinding or
independently addressable Place. Knowing all runtime inputs during static
evaluation does not relabel the producer's Pout.

Compile realization is a family C(c)={C_sigma(c)} with an ordinary callable
structure and correspondence to the same source invocation. It may be lazy;
one global C(F) cannot represent every admissible input/output configuration.
It hides unreadable runtime Val1 without erasing an argument, its Pattern,
Val2 or semantic identity. Every projection and runtime residue retains the
sealed (c*,sigma*,frame); runtime does not reselect.

Generic meta partner M(F) has a separate symbolic anchor and invocation
identity. It is not C_sigma(F). A cached companion entry records a derivation;
it neither creates semantic callability nor grants a second dispatch route.
Existing ordinary source-call and associated-entry carriers implement only
the connected slice recorded in the roadmap; they do not establish the full
R_vis/C_sigma consumer.

## 9. Normalized call-site handoff

`NormalizedCallSite.target` is the callable object expression, not the `()`
member. The full pipeline is:

```text
target expression -> ordinary x -> Type(x).associated Val2[()] -> Impl
                  -> type tau -> V_tau -> c -> Type(c).associated Val2[()] -> Impl
```

A NormClosure is non-semantic source material. Its connected evaluator must
use ordinary complete-tau construction wherever a closure expression legally
completes, with the actual placement, dependency and readiness obligations.
Normalization neither constructs values nor allocates environments.
Current source carriers do not implement this full consumer.

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
- Reusability requires instance MoveEffect=Preserve and frontier Movable; ZST alone proves neither.
- Non-ZST function objects obey ordinary ownership and passing rules.
- An unwritten function-object mode is no override; inherited/contextual
  constraints and applicable default completion determine it. Export preserves the complete namespace-internal mode
  and filters external candidates through independent capability/visibility
  eligibility rather than a universal const projection.
- Pin inherits P2 with explicit stage/mode constraints or holes; Pout stage inherits P1.
- Ordinary closures distinguish explicit, explicit-inferred-binder, and
  implicit-eligible capture requirements; those requirements do not define
  `self` fields or physical layout.
- In-place syntax forms dependencies automatically and returns ordinary tau.
  Invocation uses those dependencies without recapture. Binding, transfer and
  outer writes depend on actual access/capability/lifetime, not source placement.
- Ordinary and built-in privileged meta functions follow the same
  function-object and implicit-self call model.
- Ordinary/compile local pattern construction uses the function-object internal
  Self frame; compile does not create a MetaInstanceScope.
- Ordinary meta construction is anchored by the canonical MetaInstance anchor
  `M` (a symbolic-navigation layer); its direct result is `τ_M` rooted at `M`;
  built-in privileged AST meta functions may instead use their declared special
  scope/owner rule.
- `.name` uses ordinary name::adl generation/forwarding; E.name shares that
  entrance, while ..name retains its direct-call surface. Private normalized
  forwarding-body generation is a legacy carrier, not canonical authority.
- Callable-tail named strategy metadata operates only on fully admissible
  candidates and cannot reopen ordinary overload enumeration.
