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
CallCandidates_type(tau)
    = disjoint_union over c in V_tau of CallCandidates_ordinary(c)
CallType(tau)
    -> project every c in V_tau
    -> project each AssociatedNamespace(Type(c)).Val2[()] with actual_self=c
    -> one candidate family
    -> one ordinary applicability/preference/unique-selection
    -> sealed (c*, Impl*, projection*, frame), self=c*

CallCandidates(G)
    = disjoint_union over tau in G of CallCandidates_type(tau)
CallOrdinary(x): CallTarget(I)=x, ActualSelf(I)=x
CallType(tau): CallTarget(I)=tau, ActualSelf(I)=c*
```

Entries denotes the implementation family offered to candidate preparation,
not an additional selection stage. Each entry retains its actual callable c
and implementation Impl. All entries from all c compete together; there is no
per-c winner or preliminary choice of c. The existing sealed candidate records
this pair together with projection and frame, rather than adding a second
selection. It never executes speculative bodies or chooses a runner-up after
selected failure.

AssociatedNamespace(T) = MemberScope(Core(T)); its Val2 is Core(T)'s Val2,
not /tau(T), V_T or an independent companion. AssociatedName(T,s) is
NameCoord(AssociatedNamespace(T),s). The
[type-value owner](type-values-places-and-borrow-views.md#associated-namespace-is-the-core-member-scope)
defines these coordinates and their Place boundary.

`V_tau != Val2` is an operational distinction. An ordinary x enters through
its type's associated Val2 call entry; it does not first project Type(x).V_tau.
A type callee uses its own V_tau to supply actual callable values, then those
values use the ordinary entrance. Neither entrance requires a defining-name
lookup or a self-construction witness. An empty projection does not restart
name resolution.

The [name/type algebra](names-and-overload-groups.md) owns type/group aggregation
and their distinct update algebras. V_tau registration, Val2 residency, Pattern
registration and ConstructEdge are independent judgments. TypeRole follows
absent Val1; ConstructEdge witnesses structural construction, not type identity
or type-callee projection.

Every closure expression's legal completed evaluation returns a complete tau_C,
whether ordinary, in-place, generated, file-level or local. Formation atomically constructs the complete type and its structural witnesses
from head/body/dependency material. Complete types can participate in ordinary
* / *= under their own premises. See
[construction](structural-type-formation-and-composition.md#7-closures-and-anchored-contributions).
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
This diagram describes a closure without default formals. With trailing defaults,
§12 expands its formation material to several ordinary callable members of the
same complete tau_C; c_n retains the source implementation leaf. It does not
turn one member into an optionally applicable callable.
One selection fixes c_C together with its implementation and exact self; it
does not replace all self types with type. User-defined x:T with direct () still receives x.

Ordinary let binds the completed tau without a wrapper. In-place syntax uses
automatic dependency formation (§7.3). Its result supports ordinary value
operations, subject to actual dependency, capability and lifetime checks.
General [dependencies](dependency-observation-and-realization.md) precede capture
and layout. Closure lifetime propagation is explicitly handed to the lifetime
owner, not inferred from tau, Core equality, ZST or placement.

### 1.1 Two orthogonal construction axes

Ordinary-value callability is ordinary Val2 member semantics at the special
terminal selector (). P let ()::path:t forms its typed Place; borrow,
initialize and replace follow the ordinary name/Place rules. A legal
AssociatedNamespace(T).Val2[()]=k supplies x:T's implementation entries.
There is no additional ordinary-callable registration.

TypeAdd(T,v) is the other axis: it adds only v'=AnchorFor(v,T) to V_T.
TypeMember_T(v') requires OrdinaryCallableValue(v'), including present Val1,
the complete-type classifier home and non-generative RegisteredCallability_T.

    TypeAdd(T,v) does not imply AssociatedNamespace(T).Val2[()] = v
    AssociatedNamespace(T).Val2[()] = k does not imply k in V_T

The candidate union above therefore ranges over ordinary callable values,
each entering through its own classifier's associated ().

Ordinary let f=C installs tau_C:type. Only an already established same-name
contribution role consumes CallabilityContribution(T_f,C)=c_C^f instead.
Sibling materials jointly form T_f and all its members against a common
snapshot; no first sibling supplies a standalone tau_C as the initial resident.
Neither TypeAdd(T_f,tau_C) nor automatic bulk import of V_tau_C is that relation.

## 2. Pipeline call form

The source skeleton is P |> E P2 or P |> E. Omitting the pipe deletes only
that token; it never exchanges argument and callee:

```text
P |> E == P E
P E != E P
```

```lang
()f;
x f;
(x, y) f;
```

Traditional callee-first f() / f(x) are not source call spellings.
Call(...), Invoke(...), Entries(...) and nested H(G(...)) in semantic
equations are metanotation, not exceptions to this direction.

A name-headed callable shorthand preserves (_, let _: (name)$) as its semantic
head. It must not lose slot zero by expanding to (<> name) alone; see the
Pattern owner §9. This retains the selected ordinary x or type-projected c
all the way into the actual invocation.



    Product |> expression
      -> resolved value / named type / explicit candidate group
      -> ordinary call projection
      -> ordinary value: Type(x).associated Val2[()], self=x
         type: union of every c's Type(c).associated Val2[()] entries, self=c
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
  -> from the held source P coordinate, enter its resolved Pattern owner
  -> enumerate the language-authorized associated Val2 family
  -> obtain each candidate value's TypeValue
  -> resolve associated ()
  -> build InvocationFrame
       ActualSelf = selected function object (contextual receiver, not expanded Param[0])
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
carry the type resident `:t` plus ordinary value members over the four current
`output PolicyMode <- input PolicyMode` coordinates. Every coordinate is
expressible, but no coordinate is required to exist: each may be absent or
realized by `default`, `delete`, or `custom`. More specific Pattern members may
refine or delete regions of that capability relation. This derived 2×2 view is not
the two-point Policy preference order.

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
connected atomic slice begins from the source value's existing P-coordinate
owner and its associated `()` Val2; independently authorized mechanical
operations may select other ordinary associated families without changing
this demand rule.

The connected `lang_build` slice now implements this routing for source
function objects and toolchain-source associated `()` entries:

```text
Semantic name binding
  -> ordinary resident x
  -> Type(x)
  -> associated Val2[()]
  -> PreparedCallCandidate
  -> InvocationFrame
  -> ordinary result entries
```

The source-name binding and held-complete-type operation entrances merge at
the same candidate pipeline. The latter consumes actual resident witnesses
from the captured Core and carries an explicit semantic receiver.

Name-based source calls and compiler-authorized operations therefore have
different candidate entrances but one ordinary call trunk. Neither entrance
requires `TypeValue -> original carrier name binding`: source navigation resolves a
name binding and reads its values, while an already-held complete type value carries
its own callspace:

```text
TypeValue(t) = tau = <Q, V_τ>
Core(tau) = Q
CallSpace(tau) = V_τ

Candidates(args |> t)
  = disjoint_union over c in CallSpace(TypeValue(t)) of CallCandidates_ordinary(c)
```

Copied/extracted type-as-callee lookup selects candidates from that immutable
`V_τ` snapshot. Complete type values require no defining-name binding
recovery, most-recent carrier, or reverse `AsType` candidate entrance.

Associated source navigation obeys the same forward-only rule. If:

```lang
let T:type:: = uint8;
```

then a target selected through `T` resolves T to one terminal NameBinding and reads the complete
type snapshot. Type-as-callee candidates come from its `V_τ`; an ordinary
navigated member may additionally be selected through `Q`/`Val2` under the
normal navigation rules. Neither path inspects provenance or searches for
uint8's defining binding.

Associated operation candidates observed through a complete type value come
from the owned Val2 material captured by that exact `Core(tau)`. Every admitted
candidate requires a resident witness in that snapshot. A Pattern coordinate
may index an explicitly registered Core Object; it supplies no member lookup
authority by itself. Later changes to another Object with the same Pattern
cannot add candidates to a held snapshot, and navigation through a resident
Object observes that Object's own Val2.

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
`policy-and-static-flow-projection.md` §1.2 are distinct:

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
in AssociatedNamespace(T).Val2, retains ActualSelf=x and checks its ordinary
receiver contract. It does not project V_T. T may provide an applicable () even when V_T is empty. A ref/share
decorated value has its own exact type and therefore
its own matching () entry. There is no coercion of T into T ref/share to repair
an actual-receiver mismatch, and those distinct callee types are not one receiver
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

`.name` denotes `name::adl`; `E.name == E |> .name == E |> name::adl`.
This is an ordinary expression call, not NameExpr, direct field lookup or an
atomic Place operation. `move x.field` cannot be read as removing a subfield
and leaving a partial aggregate. Structural extraction has its own registered
Pattern relation. The [generative declaration owner](../patterns-overload/operator-patterns-and-generative-declarations.md)
defines its ordinary requested-name forwarder. The receiver object remains an
explicit argument after the selected callable's own self.

The normalizer preserves the dot/name/path source role; it has no semantic
authority to manufacture the forwarding implementation. Raw DotName now
normalizes to ordinary name::adl navigation. Generated provenance never
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
ActualSelf = the selected c_fun callable, of A_fun = Type(c_fun)
source formal 0 = contextual self_fun binder
explicit Product = (object, ...args)
InternalInput = merge(SelfProjection(I,Param), explicit Product)
```

A virtual-field-like function is the same shape with only `object` after
`self_fun`. Direct member-call sugar such as `object..fun(args...)` and the
first-class `.fun` form ultimately call this ordinary associated value; they do
not turn `object` into ActualSelf of `fun` or prepend a singleton to its captures.

The special target `let () = implementation` is different. It installs the
call entry of the current Pattern owner. Invoking an object of that owner type
retains the object itself as ActualSelf and the ordinary singleton receiver Param. No anonymous wrapper closure value becomes
the caller, and the implementation carrier is not prematurely materialized as
a standalone value.

## 6. Implicit `self`

Every callable, including ordinary, in-place, and compiler-generated
closures, has a contextual implicit self-position in its source formal frame.
When the source writes any formal position, the first written formal explicitly
declares the Pattern/binder for that position. Its spelling is unrestricted;
`self` is conventional rather than reserved.

The selected call entry `()` retains the value being invoked as ActualSelf.
For a standalone function this value is the function object;
for an associated call entry it may be a `T`, `T ref`, `T share`, or another
ordinary callable object. The user cannot manually pass this slot.

The source product contains only the explicit user arguments. `ProductSyntaxMaterial`, `ArgProductShape`, and `RawArgShape` represent only the explicit product supplied by the user. They do not contain the implicit `self`.

The contextual `self` belongs to the callable-entry invocation frame, not to
the source product. This source formal position does not assert one Object
slot in the complete internal input or in Function Pattern extraction.

Declared receiver/parameter policy-pair compatibility applies uniformly to
that complete frame:

```text
SourceFormalFrame:
  position 0 = contextual self Pattern/binder
  position 1..n = explicit argument Patterns

InternalInput = Merge_K(SelfProjection(I,Param), B_K(explicit_args))
every actual position satisfies its selected entry's ordinary requirements
```

The formal/call-site alignment is:

```text
written formal 0       <- selected callable's contextual self binding
written formal 1..n    <- explicit call-site Product positions 0..n-1
```

A head with no written formal still has contextual self, without a written
binder. Its implicit input Product may have zero, one or many value positions.

No separate self-policy plane is required. P1/P2 are independent. For
omitted ordinary P1 stage, runtime P2 defaults to runtime, and
compile to compile; open qualification follows its own owner. Explicit P1
is never overwritten. The completed view must supply legal self material.

Pin = ElabIn(HorizonPolicy(P2), Delta_in). Reify projects to the ordinary runtime
horizon for this input view. Omission inherits the applicable base; explicit
const/mut refines mode, and explicit stage atoms/holes constrain stage.
Thus a runtime callable may have heterogeneous compile and runtime Pins.
Pout = ElabOut(P1, Delta_out), with Pout.stage = P1.stage. A mode qualifier
remains an overload-order Pattern, not ordinary binding P1 projection that
deletes an oppositely qualified actual before ranking.

Candidate preparation also carries that qualifier outward as the parameter's
two-point product-order position. It therefore affects selection between
callable objects as well as the effective parameter pair seen after entry.

### 6.1 Callable owner, receiver type, and local pattern construction

Every callable, including an in-place closure, has a parent-linked
`CallableOwner`. This is lexical/code identity. It does not universally
determine the actual receiver type (not an expanded input index):

```text
CallableOwner(C) != ReceiverType(C) in general

standalone closure default:
  ReceiverType(C) = AnonymousType(CallableOwner(C))

associated () implementation:
  ReceiverType(C) = type carrying the selected call entry
```

The callable-local Self observation preserves namespace, receiver and activation
relations without exposing a public frame/capture tuple. Norm_Name is independent
of structural unfolding of self. ActualSelf, CallTarget and active return frame
are distinct observations of the same call.

This does not inject callable-local declarations into the named receiver
type's namespace. Nested owner paths use source navigation order:
current/innermost `Self` first and outermost `Self` last. This spelling is not
identity and does not assert that each receiver type is anonymous. Canonical
ownership contains no synthetic `__inner_space` or `__inner_namespace`
component.

Every compile call forms a CompileInstance from the current parent semantic
owner, selected CompilePartner and canonical inputs before EnterBody. The
result may be any declared ordinary value; a single direct instance-open
self-root type provides a computed NameExpr. The [invocation owner](../static-evaluation/compile-instance-invocation-and-result-delivery.md)
owns result reachability, delivery, current storage and open/close completion.

struct is an ordinary type-to-type helper call. Structural formation belongs
to S interpretation, with actual Val2 role witnesses published atomically.
Every ordinary and in-place closure keeps its own lexical/Self/navigation layer.
Selected semantic call entries supply implementation, Policy and privilege;
graph declarations remain rendering projections.

### 6.2 Consumer-directed self projections

This owner defines the projections of one selected invocation:

```text
I = <SelectedCallable c*, SelectedImpl Impl*, sigma*, frame*, K>
CallValue(x): ActualSelf(I)=x;  CallTarget(I)=x
CallType(tau): ActualSelf(I)=c*; CallTarget(I)=tau, c* in V_tau

SelfProjection(I,CallAnchor) = selected callable anchor
SelfProjection(I,Param) = implicit dependency Bare Product of this entry
SelfProjection(I,Return) = (CallTarget(I),Activation(I))
ReturnFrame(I) = SelfProjection(I,Return)

ordinary user-defined x:T associated () call: SelfProjection(I,Param)=B_K(x)
value-capture closure C: SelfProjection(I,Param)=CaptureValueProduct(C,I,K)
InternalInput(I,explicit_args)
  = Merge_K(SelfProjection(I,Param),B_K(explicit_args))
```

CaptureValueProduct is defined only by the [dependency owner](dependency-observation-and-realization.md#41-ordered-value-capture-material).
Compile structural dependencies do not become value parameter positions.
The ordinary user receiver is the cardinality-one case; closure captures can
have cardinality zero, one or many. The contextual first formal can observe
the caller/context without requiring an extra universal environment Object
among these positions. CallableOwner, ReceiverType, ActualSelf, CallTarget
and ReturnFrame remain distinct; binder spelling and self type select no
return activation.

Before selection, a candidate's formed implicit material and declared required
capture availability are local, read-only evidence for ordinary applicability and its implementation's input
requirements. It executes no transfer, initializer or candidate body. Candidates
need no common capture prefix, shape or count, and capture shape adds no
preference. Required missing evidence remains Unavailable. Only unique sealed
selection supplies I and authorizes actual implicit/explicit input formation
and movement for c*. This is the ordinary pipeline, not another selector.

The callable retains its entry/source anchor, so `(args) |> self` enters an
ordinary new call through CallAnchor. It creates a new activation and selects
at that call's actual frontier; no old sealed invocation is reopened. Reentry,
state transfer and borrows require the current Movable and Pre/Post facts.

Self, initialized captures, required captures bound in the actual call
context, and explicit arguments are equal-status call/lifecycle material:

```text
CallInputs(I,K) = InternalInput(I,ExplicitArgs(I))
               = Merge_K(SelfProjection(I,Param),B_K(ExplicitArgs(I)))
```

The closure Param contains its initialized and call-bound required capture
positions exactly once; an ordinary user receiver uses B_K(x) as above. These
roles supply no extra self position or duplicate capture transfer.
Semantic name origin is determined by the actual selected source operation,
never by an environment field offset. A capture need not be a field child owned
by the ActualSelf Object. Real Object fields retain ordinary field semantics.

### 6.3 One coherent passing transaction

```text
Pass(CalleeSelf(I)) ~life Pass(SelfProjection(I,Param))
SelectedCall -> exact implicit + explicit input material
  -> ordinary pass normalization, all terminal Move
  -> all affected Move/borrow/access/lifetime Pre
  -> atomic commit at existing action boundaries -> joint declared Post
```

Here ~life requires compatible observable consumption effects on actual
dependency/parameter positions, not equal Object identities or two transfers.
The implementation cannot Kill whole-self and then Kill each projected capture
again. Ref/share positions transport their handles with existing target,
generation and origin; they do not transfer ownership of the referent.
Kill/Preserve is fixed independently of Movable. Preserve creates no copy or
clone; a killed old generation cannot be reused. These obligations consume
the existing mechanical and lifetime judgments and introduce no new transaction
or lifecycle evaluator.

The [lifetime contract protocol](../lifetime/lifetime-policy-and-overload-boundary.md#216-call-boundary-summaries)
uses this same invocation correspondence: caller Pre.check, body Pre.admit,
body Post.check, caller Post.admit. Actual parameters, result names, captures
and activation determine the mapping; the protocol adds no second transfer or
new self. Caller Pre must hold before body admission, and adopting Post does
not replay the body. Implemented functions form automatic Post from already
formed semantic guarded body/operation descriptions; actual exits separately
validate the mapped summary under the [two-boundary lifecycle rule](../lifetime/lifetime-policy-and-overload-boundary.md#101-contract-formation-structural-transformation).
No future branch execution supplies that advance summary. Handwritten Function Patterns still default
omitted Post to NoDelta and cannot recover implementation structure from known
Val1. [LC4–LC6](../../contracts/lifetime-program-consumers.md#lc4-contract-expressions-and-four-way-boundary-consumption)
record the pending source consumers. Each generated default-forwarding body
must summarize its own real actions under this rule.

## 7. ZST function objects

A function object with no stored environment is normally zero-sized.
Representation alone proves neither Killable nor Movable nor Copyable.
Reusable movement requires the instance-level Preserve proof and ordinary
frontier legality defined by the lifetime owner. Stored capture layout and
type equality do not replace those judgments.

### 7.1 Function-object PolicyMode default

The binding created by `let fn = () => { ... }` has no written mode override.
Inherited/contextual constraints are considered before a separate applicable
DefaultModeCompletion chooses mut only under open + current OpenHere, const
otherwise. Required missing evidence is unavailable. Omission is neither an
explicit mode constraint nor a deduction hole; P1 mode is not generally copied from P2. An
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

`[let x=E]`, `[x=E]` and non-bare expression shorthand initialize dependencies
at formation. `[x]`, `[x:T]` and `[p let x]` declare RequiredCapture, fulfilled
by name in the actual admitted call context under the dependency owner's §3.
Neither kind implies const, borrow or write authority. Resolved
free external observations may use an eligible implicit realization under
ordinary lookup, Policy/access and lifetime checks. Capture is one surface
consumer, not the ontology of every function/type/host/Path dependency.

Initializers share the pre-capture name environment, not concurrent effects.
They execute only at the reached formation occurrence, never once per body
invocation or projection. Snapshot versus live-place behavior is decided by
semantic realization before layout. Actual owned state participates in ordinary
structure and identity; refs retain targets/generations. Neither hidden
semantic side tables nor mandatory public self.Val2 fields are allowed.

Value-capture modes are mutually exclusive for one closure; automatic compile
structural dependencies remain separate. Distinct admitted occurrences preserve
binder, view and source obligations even when layout shares storage.
Copy/replication preserves formed dependencies and required specifications
without rerunning initializers, rebinding formed sources or extending lifetimes.
Runtime formation residue is retained when not ready.

### 7.3 In-place syntax uses automatic dependency formation

`NormClosure.placement = InPlace` records a source formation form. It is not
a non-transferable value category or a post-formation permission flag.
Head presence and generated provenance remain separate syntax facts: bare
blocks and headed bodies without => use this form; => selects ordinary syntax.
The parser does not infer placement from head presence.

```text
DependencyMaterial(C) = D_struct_compile(C) union D_value_Mode(C)(C)
InPlace(C) => ValueCaptureMode(C)=Automatic
ClosureFormation(C) = FormCompleteType(Head_C, Body_C, DependencyMaterial(C))
Eval(InPlaceClosure_C) = tau_C
```

The [dependency owner](dependency-observation-and-realization.md#4-explicit-and-automatic-dependency-formation)
defines the closure-wide exclusive value modes and independent automatic compile
structural dependencies. An explicit clause cannot acquire an omitted external
value automatically; an ordinary => closure without that clause may use
Automatic mode. Resolved capture binders replace outer observations. CompilePartner
selection preserves these established dependencies and their ordinary identity.
Invocation preserves the actual lexical owner and admitted dependency coordinates.

Free external observations are handled at formation through ordinary resolution
and dependency realization fixed by the source occurrence's selected ordinary
action. Owned material, ref/share and stable links are possible results of
different legal actions, not interchangeable lowering choices. Each
requires its own admissibility, access, capability, readiness and lifetime facts.
Automatic formation alone creates none of those permissions.
For one selected action the semantic realization is unique up to observational
equivalence; distinct candidates use ordinary preference or report ambiguity.

After formation, invocation consumes the established dependencies. It does not
resolve external names again by spelling or create a separate embedding
environment. Deferred formation retains its actual obligations; deferral is
not permission to recapture at a later invocation site.

For InPlace automatic existing captures, omitted Pre inherits the formation
slice's preceding Post, as specified by the lifecycle owner's empty-slice law.
Only genuinely new entry parameter names use entry defaults. A real Move/repair
boundary supplies its validated successor state; Ordinary closure boundaries
are not empty slices.

Bind, Move, Copy, Return, Store and Pass are ordinary value operations on the
result. Source placement alone neither forbids them nor grants their concrete
legality. In particular, let f = { ... }; is not rejected merely because its
initializer used in-place syntax. Wrapping the result in Product, group or tau
preserves actual dependencies and lifecycle obligations, not a hidden source
restriction.

Independently, nonidentity compile-to-runtime same-Type migration must prove
retained internal structural Paths recoverable in its prospective destination
world at hard A, as defined by the
[Policy owner](policy-and-static-flow-projection.md#34-policy-migration-satisfaction-existing-first-unique-migration-second).
Proved failure rejects that candidate; unknown stays unavailable. This uses
actual type/dependency evidence, not placement, and is not Function Pattern
source-erasing projection. Existing-view reuse preserves established obligations.

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
allocation. After formation, placement and explicit/automatic dependency origin
participate in neither applicability, specificity nor preference:

```text
Form(C1) equivalent_to Form(C2)
and same ordinary applicability/preference evidence
    => Placement(C1) != Placement(C2) supplies no ordering
```

Distinct declaration/candidate identities remain distinct. If their ordinary
evidence ties, ordinary uniqueness reports ambiguity; source placement cannot
break the tie. Raw/Norm placement remains only formation syntax information.

## 8. Call lookup pipeline

```text
shape explicit Product; retain contextual self and candidate-local implicit evidence
resolve target once -> ordinary Val2[()] entrance / type V_tau projection / group
expand all actual callable/implementation pairs -> one candidate family
apply pre-C0 family filter -> enumerate C0
R_vis(c,Omega,sigma) -> candidate plus visibility/input/projection evidence
prepare ordinary C_sigma(c) where required
FullyAdmissible A including total output demand and require
fallback suppression D
Policy product maxima (including migration endpoints when applicable)
Pattern specificity / first-order / named filters
unique Selected=(candidate*,sigma*,frame), candidate* retains (c*,Impl*)
form selected SelfProjection(Param) + explicit input; normalize terminal Move
DynamicLegality -> Ready execution or retained continuation
```

The two rounds are owned by
[Policy §12](policy-and-static-flow-projection.md#12-unified-binding-and-overload-selection).
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

Each C_sigma(c) is a realization of the selected CompilePartner, retaining the
source receiver/call-entry pair and sealed projection/frame. It forms its
CompileInstance through the common invocation relation, independently of result
kind and P2. Cached correspondence supplies neither callability nor execution
permission; source and projection consumers obey the same ordinary relations.

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
- Each value/ref/share receiver type supplies its own associated () receiver contract.
- `CallableOwner` and receiver type are independent semantic facts.
- Contextual self retains the selected caller/context; CallAnchor, Param and
  Return are distinct consumer projections. Param is not universally one Object.
- The self role is positional; the first written formal exposes it under an
  ordinary user-chosen binder/Pattern, and `self` is only a conventional
  spelling.
- Implicit `self` is not part of `ProductSyntaxMaterial` / `ArgProductShape`.
- The user cannot manually pass implicit `self`.
- `()` is not an operator.
- Operator values cannot be namespace/navigation parents.
- `()` is a special type/namespace call entry and can only be a navigation leaf.
- Struct-associated named lets contribute ordinary Val2 entries; the special
  empty target `let ()` contributes the current owner's call entry.
- Each selected associated entry's ReceiverPattern/classifier requirements are
  checked against ActualSelf, independently of the cardinality or element types
  of expanded SelfProjection(Param). No universal first-capture equality holds.
- Reusability requires instance MoveEffect=Preserve and frontier Movable; ZST alone proves neither.
- Non-ZST function objects obey ordinary ownership and passing rules.
- An unwritten function-object mode is no override; inherited/contextual
  constraints and applicable default completion determine it. Export preserves the complete namespace-internal mode
  and filters external candidates through independent capability/visibility
  eligibility rather than a universal const projection.
- Pin inherits P2 with explicit stage/mode constraints or holes; Pout stage inherits P1.
- Ordinary closures distinguish RequiredCapture from InitializedCapture and
  automatically formed dependencies; those requirements do not define
  `self` fields or physical layout.
- In-place syntax forms dependencies automatically and returns ordinary tau.
  Automatic and initialized sources are retained; declared required entries
  bind at new calls. Binding, transfer and
  outer writes depend on actual access/capability/lifetime, not source placement.
- Every compile call forms a CompileInstance before body evaluation, independent
  of result kind and P2; selected call entries retain their ordinary self.
- Every closure keeps its lexical/Self/navigation/dependency layer.
- struct adds ordinary type helpers while preserving structural registration.
- .name enters name::adl; receiver and type-path candidates use ordinary
  applicability/specificity. e..name(args) equals e |> .name(args).


- Callable-tail named strategy metadata operates only on fully admissible
  candidates and cannot reopen ordinary overload enumeration.

## 11. Callable tag and activation observations

```text
CallType(tau):
  select (c*,Impl*,projection*,frame*) from V_tau
  ActualSelf(I)=c*
  CallTarget(I)=tau
CallValue(x): ActualSelf(I)=x; CallTarget(I)=x
ReturnTargetTag(I)=CallTarget(I)
```

The tag is the currently invoked callable Object, independent of the chosen
implementation receiver. Tag plus actual control-flow activation identifies
one active return frame. Recursive activations of the same tag remain distinct.
Type(tau_C)=type and Type(type)=(1 type); classifier extraction observes this
rank, independently of Type(c*). Self is Pattern-observable without transparent
capture-layout unfolding. Neither tag nor its representation affects selection.

## 12. Trailing default formals form ordinary overload members

### 12.1 Expansion and identity

A default is source formation material, not an optional position in a formed
callable. The expansion preserves the source consumer role: ordinary closure
expressions form callable members of tau_C; authorized same-name contributions
form members at their established target; direct current-owner
`let () = implementation` material forms ordinary implementation entries of
that owner's associated call entrance (§5). None changes the receiver role.
Let n count the explicit formals, excluding contextual self and capture
positions. Let exactly the final k explicit formals have defaults d_j:

```text
0 <= k <= n
required: a_1,...,a_(n-k)
defaulted: a_(n-k+1)=d_(n-k+1),...,a_n=d_n
ExpandDefaults(C) = {C_n,C_(n-1),...,C_(n-k)}
G_C = {c_n,c_(n-1),...,c_(n-k)}
V_tau_C = G_C                         // standalone closure formation
Home(Type(c_m)) = TypeMemberScope(tau_C)
```

G_C here denotes the set of callable members, not an extra OverloadGroup
Object around tau_C. The explicit contribution consumer may instead form
these members directly at its common target (§6.1 of the
[name owner](names-and-overload-groups.md#61-common-snapshot-closure-contribution-formation)).
Each c_m has exactly m explicit positions, its own callable/implementation
identity, and an ordinary associated Val2[()] entry. For k=0 this is the
existing singleton closure formation. c_n retains the source head with the
default clauses removed and the source implementation body exactly once.
All c_m enter the same ordinary candidate space; no preliminary family winner,
optional-argument applicability or preference for using fewer defaults exists.
Distinct otherwise tied user/generated candidates remain ordinarily ambiguous.
Removing a default clause preserves the formal's written Pattern/Policy.
An omitted annotation remains ordinary formal Pattern deduction, not lexical
let initializer inference: the default alone does not restrict every explicit
argument of c_n to the default result's type. A written classifier constrains
both supplied and default-produced material through ordinary extraction.

Expansion never drops the contextual self position or copies it into the
explicit source Product. In particular:

```text
source head:    (self, let i = 1uint8)
full head H_1:  (self, let i)
short head H_0: (self)
ExplicitFormals(H_0) = []             // separate, do not erase contextual self
ExplicitInputPattern(H_0) = () = BareProduct() = unit
```

Here `(self)` is a callable extraction head, not a value-side group and not a
singleton explicit self/Unit argument. After separating contextual self, it
matches the unit/empty explicit input. The full internal input may still include
ordinary self/capture material through SelfProjection(Param); it need not be
unit. The source self Pattern/binder/Policy is retained unchanged in every
expanded head. It cannot be defaulted, omitted by expansion, or manufactured
from a default expression. This restriction follows the first contextual
position, not the spelling `self`; another binder name there has the same rule.

Direct current-owner implementation material expands the same heads into
Impl_n,...,Impl_(n-k) in the existing associated Val2[()] implementation family.
For an invoked x:T, its ordinary candidates are (x,Impl_m), with ActualSelf=x
for every m, including zero explicit arity. These are distinct real ordinary
candidates, not optional arguments on one implementation. No anonymous c_m
replaces x, and no member is added to V_T by this expansion. Installation uses
only the original direct call-entry consumer's ordinary authority. Source body
self remains x in this role, versus c_n for a standalone full-arity member.

Defaults must be a contiguous trailing suffix of direct explicit formals.
Contextual self, captures, ReturnPattern and Pack/remainder positions cannot be
defaulted. A default applies to a whole direct formal, whose Pattern may itself
destructure; it is not a default on a nested extraction child. Defaults combined
with Pack/remainder in that head are outside this sugar. There is no named-argument
skipping, interior omission or explicit omission token: a supplied Unit or empty
Product is ordinary supplied material under the Product rules, not a request
to insert a default. Arity is observed after the ordinary available call-input
exposure, not by counting raw parentheses or including self.

### 12.2 Arity-growing ordinary forwarding

Each shorter entry computes just the next omitted formal, then makes a fresh
ordinary call with one more explicit position into its established call space:

```text
ForwardTarget(C) = formed tau_C                 // standalone closure
ForwardTarget(C) = formed target T_f            // authorized contribution
ForwardTarget(C,x) = x                         // direct associated () entry

for n-k <= m < n:
  entry_m(self; a_1,...,a_m):
    evaluate d_(m+1) once in its source parameter environment
    deliver (a_1,...,a_m,result(d_(m+1))) through an ordinary call to ForwardTarget
    explicitly return that call's result to this entry's actual activation

with only these generated entries applicable:
  entry_(n-k) -> entry_(n-k+1) -> ... -> entry_n
```

This is semantic pseudocode, not conventional source call syntax or a hidden
temporary rewrite. A closure/contribution forwarder retains the ordinary formed
complete-type target through an admitted dependency, not a public name to resolve
again. Its own self=c_m is not that family target. A direct associated forwarder
instead uses its existing receiver x as the ordinary callable target, preserving
that receiver's identity and ordinary current generation/availability premises.
It does not materialize an anonymous wrapper object or explicitly pass x in the
source Product. Callable lookup supplies implicit self in the ordinary way.

The target's entire ordinary candidate space participates at this new call.
Generated and user entries compete by normal applicability/preference/ambiguity;
there is no private filter selecting the nominal next generated entry and no
implementation-leaf shortcut. With only the generated fixed-arity entries,
adding one explicit position reaches the next entry; another ordinary winner
or ambiguity remains possible when other candidates are present. This is not
a second attempt at the outer call: the already selected entry stays sealed.
Lookup, R_Gamma/Policy preparation, selection, input Move, lifecycle checks and
InvocationResult delivery all occur at the new call's actual frontier. Missing
receiver/target dependency or access evidence is unavailable, not permission
to substitute another forwarding target or invoke an implementation directly.

For example, let the generated family be {c_2,c_1,c_0}, with defaults d_1,d_2,
and suppose the empty-input outer call has selected c_0. It evaluates d_1 once
and makes a one-explicit-argument ordinary call into the held target space:

- If a user candidate h is applicable but strictly loses to c_1 under ordinary
  preference, c_1 wins and evaluates d_2 once before its own forwarding call.
- If h strictly wins over c_1, c_0 forwards to h; c_1 and c_2 are not forced and
  d_2 does not execute. Any later work of h follows h's own ordinary semantics.
- If h and c_1 are tied distinct maxima, the inner call is ambiguous. Neither
  body nor d_2 executes; c_0's already selected outer invocation fails without
  trying another outer candidate. The reached d_1 effect remains governed by
  ordinary failure/cleanup rules.

Thus generated-member-to-next-member forwarding is a guarantee only when the
ordinary inner winner is that next member, not a private family chain law.

Only the selected entry executes. Selection never evaluates defaults, probes
their effects or executes forwarding to decide applicability. Applicability
may consume ordinary declared/inferred Pattern, Policy and contract evidence;
missing required evidence is unavailable, not proof of a viable default.
After a shorter entry is selected, a failing default, forwarding call, lifecycle check or
return delivery fails that invocation. Neither the outer selection nor any
already sealed inner selection can reopen. A full-arity call executes no
default, including when it supplies a value equal to a default's result.

Each wrapper exposes the prefix input contract and the source return demand,
with ordinary mapped/deduced Policy and lifecycle obligations. It does not
erase a required dependency or invent evidence about a later forwarding call.
Its actual Pre/Post contract must account for both default evaluation and the
forwarding call under the ordinary lifetime relations; copying the full entry's
contract or assuming NoDelta is not proof of the wrapper's contract. Unresolved
contract evidence stays unavailable and grants no SCC convergence assumption.
Default expressions obey that call's actual horizon and readiness: a runtime
default cannot supply a compile observation merely because its member was
generated. Compile calls have their ordinary instances; reify still requires
its independent closed-return and reverse-dependency premises.

### 12.2.1 Non-semantic forwarding warning

Whole-target forwarding is intentional, including a winner that skips the
nominal next generated member. If ordinary evidence proves all of the following,
the diagnostic layer may report that outcome:

```text
Selected(c_m)
Select(EstablishedTarget, forwarded_args, output_demand) = h
h != c_(m+1)

warning: default-argument forwarding selects a different overload member
```

Here c_(m+1) denotes the nominal next generated candidate entry, not a forced
callee. Compare ordinary candidate-entry identities, not result equality or
structural similarity; direct associated entries retain their original receiver
and implementation identity. The diagnostic observes an ordinarily proved
selection. It must not evaluate defaults or candidate bodies, re-resolve a target,
or force a future selection merely to determine whether to warn.

This Warning belongs to diagnostics, not E's evaluation semantics. Emitting or
omitting it changes no candidate set, applicability, specificity/preference,
selected entry, effects, Move/lifecycle behavior or return delivery. It never
makes a legal call illegal or upgrades the different winner into a default-specific
Error. Unknown winner/identity evidence proves no deviation and must not produce
this warning. Ordinary ambiguity, unavailable required evidence and other failures
retain their own rules; no warning substitutes for a missing winner or evidence.
Forwarding failure never reopens the sealed outer or inner selection.

Explicit declaration-side candidate controls and call-site family constraints
remain independent under the [ordinary overload pipeline](../patterns-overload/overload-resolution-design.md#3-pipeline).
Default expansion injects none of them. A user-written constraint is handled at
its ordinary stage; its violation is an ordinary failure, not a default-parameter
Error. This rule introduces no new control syntax or filter and does not settle
the remaining source-control spelling questions.

Warning emission and its executable regressions are pending, non-blocking
diagnostic work. Their absence does not invalidate the whole-target law or the
different-winner acceptance case DS22.

### 12.3 Default scope, dependency formation and passing

A default expression is delayed until its selected wrapper is entered. It is
not a capture initializer evaluated at declaration/formation. Along a completed
generated-entry chain, defaults proceed left to right, once for each omitted
formal. Only defaults of actually reached selected entries execute; an ordinary
alternative winner may end that chain. Required and previously supplied/defaulted parameter bindings are
available through their ordinary views; reading a current or later uninitialized
parameter is invalid and cannot fall back to an outer same-spelled binding.
Structural/classifier observations still require their independent ordinary
evidence. The result undergoes the formal's ordinary constrained production,
Pattern extraction and Policy checks, not an applicability-time conversion.

Defaults retain the source lexical environment and resolved external binder
identities. Their parameter occurrences are instantiated in the selected
entry's ordinary input frame. In the closure/contribution role contextual self
there is c_m, not the complete-type family target; in the direct associated role
it is the original receiver x, which is also the ordinary forwarding target.
Neither role defaults self or supplies it as a second explicit argument. The
original full implementation keeps its original CallableOwner, Self/navigation
layer and source return targets, with self=c_n or self=x as its role requires.
Generated entries have their own owners and actual activations, even when
several associated entries share the same receiver and callable tag x.

External uses in every default participate in the source implementation's
dependency requirements, under its established consumer role, as well as body
uses. Explicit capture mode must cover them;
Automatic mode discovers them at formation, not on each invocation. Source
capture initializers execute once per source formation, not once per generated
member or wrapper entry. Expansion does not clone captures, duplicate owned
resources or grant an implicit shared environment, borrow or lifetime extension.
Forwarding-target dependencies and each entry's access to the established source dependency
material must have ordinary legal realizations. The dependency owner's
uniqueness law fixes these from selected actions, not a backend storage choice.
If required retention, access or transport cannot be proved, formation or the
relevant selected use remains unavailable/illegal under that ordinary boundary.
The sugar adds neither a blanket ban on local captures nor an escape exemption.

Defaults may observe or consume earlier parameters only through ordinary
operations. Forwarding then transfers each still-required explicit position
through its own terminal Move at that edge. This is not a second transfer at
the same frontier: each wrapper has a fresh activation and current generations.
A default that kills a parameter still needed for forwarding makes that path
illegal; the sugar supplies no implicit Copy, clone, restoration or double Kill.
Default-created values and abandoned inputs follow normal cleanup on failure.

### 12.4 Return and formation authority

The wrapper forwards the source ReturnPattern/Pout demand into its immediate
call before that call's root maxima seal. It propagates the ordinary result
without an unconstrained intermediate lexical binding. Its generated terminal
return explicitly targets that wrapper's tag plus actual activation, even for
a unit result. Ordinary omitted-target inference would choose the outermost
enclosing function layer and is not a substitute; unit tail fallthrough is not
a forwarding return. User-written events inside defaults follow ordinary
source target resolution against the actually active frames, including failure
when no such target is active. They are not intercepted by the generated terminal
return, and no nonexistent full-entry activation is fabricated before its call. An
event that legally terminates the path prevents subsequent forwarding. No new
kind of control event or omitted-target inference rule is introduced.

Expansion is syntax-directed formation planning, not semantic execution in
Norm. Complete standalone tau_C formation or authorized common-target
contribution formation publishes all accepted members and registrations
atomically against its one snapshot. Direct associated implementation expansion
instead installs its complete ordinary entry family through the original Val2[()]
consumer, with the owner receiver unchanged and no implied V_T/Pattern
registration. Sharing callable tag x never merges distinct entry activations;
each generated terminal return targets its own actual activation. No repeated
lexical let aggregation,
first-sibling resident, partial Object, standalone-tau bulk import, implicit
registration from generated provenance, or new construction authority is
permitted. Ordinary dependency formation effects retain their sequence and
transaction rules; atomic publication does not promise rollback of earlier
external effects. See the [surface contract](../../public/normalized-surface-semantics.md#15-canonical-call-sugars-consumer-pending)
and [acceptance matrix](../../planning/canonical-semantic-conformance.md#empty-invocation-and-default-formal-sugar-acceptance)
for syntax/implementation boundaries and counterexamples.

## 13. Empty invocation sugar

`do E` supplies exactly the empty explicit Bare Product to the existing pipe
skeleton, as `() |> E`. It introduces no implicit self position. The target
occurrence is interpreted and formed once per reached evaluation; no binding,
closure, scope, stage, return target or control event is added by do. Lookup,
candidate selection, terminal Move, lifecycle and return delivery remain the
ordinary call relation.
The [surface contract](../../public/normalized-surface-semantics.md#15-canonical-call-sugars-consumer-pending)
owns binding/provenance; the [Product owner](../patterns-overload/product-repetition-and-unpacking.md#2-bare-product-and-context)
owns empty material versus a present Unit entry. Neither sugar implements
missing closure/call consumers by an alternate evaluator.
