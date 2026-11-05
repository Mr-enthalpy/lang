# Lifetime Values, Bootstrap and Library Facilities

Status: canonical semantics for `@`, the lifetime value algebra, and the
lifetime/ordinary-overload boundary, ordinary contract programs, boundary
transforms and library protocol effects. [Walkthroughs](lifetime-algebra-walkthroughs.md)
apply these rules; [consumer contracts](../../contracts/lifetime-program-consumers.md)
record wiring, and [acceptance targets](../../planning/canonical-semantic-conformance.md#lifetime-value-computation-and-library-design-acceptance)
are specifications rather than executed coverage. `LifecycleState` stores LifeName, Region,
Color and lifecycle observations; it checks Pre and consumes the shared
`SemanticContinuation` transaction's committed-action witness for Post.
Linearization and Pre boundaries of already fixed cleanup points are implemented.
The common transaction checks earlier cleanup for every action through a
continuation-relative observation relation. Outstanding or unavailable facts
block commit even when the action has no lifecycle projection.
The substrate fixes continuation-relative cleanup prefixes, leaving future
generations' suffix placements available. Same-cut Use/Preserve cannot occupy
their own half-open endpoint; scheduled Drop and boundary Kill follow fixed order.
Roster discovery supplies stable names, not formation/origin facts. Full
generation-continuation reification, source operations, cleanup point derivation,
access-tree construction and the complete E consumer remain pending.

This document is the canonical owner of `@`. The object model, the value/place
split, and the `ref` / `share` / `rebind` operations are owned by
[`../symbol-world/type-values-places-and-borrow-views.md`](../symbol-world/type-values-places-and-borrow-views.md);
construction authority (`OpenHere_Σ(value)`) is owned by
[`../symbol-world/structural-type-formation-and-composition.md`](../symbol-world/structural-type-formation-and-composition.md).

## 1. `@` is continuation-relative name observation

N@ is a name iff N is a name. N@ is rank-0 ordinary lifecycle material with
its own type, fields and associated operations, independently of N's rank.
Origin and region are ordinary fields of lifecycle Objects; Color belongs to
region. They use Object = <Val1?, Pattern, Val2>, not a fourth business-Object
coordinate or a higher-rank constraint layer. Lifecycle computation knows
names and these relations, not special integer/container/business-type meaning.
Type punning's legality remains with the actual conversion's other consumers.

Reification preserves name interpretation; value observations and borrowed
lifecycle fields remain distinct. Safe code may observe origin/region/Color.
Ordinary boundary admit uses the checked boundary's limited world-update
authority (§9); constructing a contract grants no such authority. Raw writeback
of otherwise underivable external knowledge additionally requires
[unsafe admission](unsafe-semantic-admission.md), ordinary borrowing, Writable
and applicable construction premises. Neither route permits arbitrary history
replacement or a missing Pre. This does not make @ a borrow constructor.

`@` is a normal overloaded operation. It is not a syntax-only marker, not a
reserved lexical territory without semantics, and not exempt from overload
resolution:

```text
Apply(@, E):
  n := NameOf(E)
  k := Pos(SemanticContinuation)
  return ReifyLife(n, k)
```

`@` still applies to an expression syntactically, but its builtin action switches
from value interpretation to the semantic name observed at the current
continuation position. It neither exposes a user-callable `PlaceOf(E)` nor reads
hidden residency metadata from the value. Lifetime facts already follow from
the continuation; applying `@` reifies them and does not launch a second
analysis.

`@` is **not** a borrow constructor. It yields a lifetime value, never a borrow
view. Borrow formation belongs to `ref` and `share`, which may be privileged
actual-place builtins (`PrivilegedActualPlace(ref-family)`,
`PrivilegedActualPlace(share-family)`) when the overload needs the actual's
place (§2). A freshly computed temporary may receive a generated semantic
LifeName even when it has no writable or borrowable Place, so
`NoCarrierPlace(actual)` is not a general failure condition for `@`.

`@` is not a Stage atom. `@` is evaluated at a stage; it
does not name one.

## 2. Privileged place acquisition belongs to `ref` and `share`

`ref` and `share` are builtin applications that may, for some overloads, be
granted privileged access to the place of their actual argument:

```text
PrivilegedActualPlace(ref-family)
PrivilegedActualPlace(share-family)
```

This does not make them one callable family. `@` remains a normal overloaded
operator whose selected builtin performs name/continuation reification; `ref`
and `share` each have a type-forming overload (producing `type ref` /
`type share` as type values) and a borrow-forming overload (producing `t ref` /
`t share` as borrow instances). Place permission attaches to each borrow
family's builtin identity, not to ordinary function parameters. An ordinary
user function that spells the same formal head still cannot obtain the
actual's place.

The operators produce different results through different semantic premises:

```text
@      -> ReifyLife(NameOf(actual), Pos(K))
ref    -> borrow instance | TypeValue (type formation)
share  -> borrow instance | TypeValue (type formation)
```

Phase order for a place-sensitive `ref` / `share` member:

```text
ordinary candidate preparation
  (Pattern / type / Policy matching on the actual value)
-> ordinary overload selection
-> unique selected builtin/default
-> if SelectedBuiltinRequiresActualPlace:
       p := PrivilegedActualPlace(actual)
       if p absent:
           InvocationFailure(NoCarrierPlace(actual))
           -- post-selection precondition failure, not candidate
              removal / overload reopening / fallback
-> execute default
```

`ref` and `share` are privileged actual-place builtins
(`PrivilegedActualPlace(ref-family)`, `PrivilegedActualPlace(share-family)`).
There is no global `E ref = Ref(Read(E))` law: the selected overload determines
the result, and the builtin default may acquire `PrivilegedActualPlace(actual)`
(canonical owner [`../symbol-world/type-values-places-and-borrow-views.md`](../symbol-world/type-values-places-and-borrow-views.md)
§5.1). For `let t:type:: = uint8`, `t ref` is the **type-forming** overload: it
yields the TypeValue `uint8 ref` (the borrow type of `t`), not a borrow
instance. The borrow instance over the type-level place is produced by invoking
that borrow type:

```lang
t |> (type ref)    // invocation: borrow instance r : type ref, Target(r) = place(t)
t |> (type share)  // invocation: share instance over the type-level place
```

`type ref` is the ordinary type construction `type |> ref`, and `type share` is
`type |> share`; they are not special tokens and not produced by `@`. A
`type ref` is a type value; an instance `r : type ref` carries the borrow
content (target place, capability, lifetime relation) as a borrow instance, not
as the type value itself.

The builtin `ref` / `share` callables have two overload roles. The
type-forming member makes `type |> ref` / `type |> share` a well-formed
ordinary type construction through the ordinary type-as-callee / overload
machinery; no `RefType` primitive is introduced:

The `ref`/`share` family is not a single compile stage: the **type-forming**
member is a **compile** member (`T : U_n ⊢ T |> ref = RefTy(T)`, producing the
borrow TypeValue), while the **borrow-forming** member inside the formed
borrow type's callspace has ordinary concrete-stage builtin/default realizations and
is the only family member that may obtain `PrivilegedActualPlace`. The
declarations below are the type-forming members; the borrow-forming members
live inside each formed borrow type's callspace (canonical owner
`../symbol-world/type-values-places-and-borrow-views.md` §5.1).

```lang
let ref =
    <n>(self, t: n type):
    compile
    => default;

let share =
    <n>(self, t: n type):
    compile
    => default;
```

The general type-forming rule takes the **operand type** `T` itself as the
parameter to `RefTy` / `ShareTy` — not the universe `U_n` in which `T` resides:

```text
T : U_n
----------------------------
T |> ref   = RefTy(T)   : U_n
T |> share = ShareTy(T) : U_n
```

When the operand **is itself a universe object** `U_n`, the rule
mechanically specializes:

```text
U_0          = type
U_1          = type_1
U_n |> ref   = RefTy(U_n)        (n ≥ 0)
U_n |> share = ShareTy(U_n)      (n ≥ 0)
```

In particular:

```text
type |> ref   = RefTy(U_0) = type ref
type |> share = ShareTy(U_0) = type share
```

This is **not** `RefTy(U_1)`: `type` is `U_0`, and `U_1` is merely the
universe that classifies `type`. Confusing the operand with its classifier
would collapse distinct types (`uint8 ref`, `uint16 ref`) into one.

(`n type ref` / `n type share` are pure mathematical metavariable notation
for `RefTy(U_n)` / `ShareTy(U_n)`; they are **not** a legal source LHS and
**not** a source-parser `n type ref` / `n type share` expression.) The
universe-uniform routing — `type : type_1` being the instance that makes
`U_n |> ref/share` well-formed at every level `n ≥ 0` — is frozen, not
future work:

```text
rank(t ref)   = rank(t)
rank(t share) = rank(t)

CallSpace(RefTy(T))
  contains
    T -> RefTy(T)

CallSpace(ShareTy(T))
  contains
    T -> ShareTy(T)
```

Specialized to universe objects:

```text
CallSpace(RefTy(U_n))
  contains
    U_n -> RefTy(U_n)              (n ≥ 0)

CallSpace(ShareTy(U_n))
  contains
    U_n -> ShareTy(U_n)            (n ≥ 0)
```

The base case is immediate — no off-by-one recursion is needed:

```text
R_1 = RefTy(U_0) = type ref

CallSpace(type ref)
  contains
    type -> type ref
```

This is the instance needed for `t |> (type ref)` / `t |> (type share)`.

The type-forming members need no privileged actual-place access. Only the
selected borrow-forming defaults inside these borrow-type callspaces possess
actual-place privilege (`PrivilegedActualPlace(ref-family)`,
`PrivilegedActualPlace(share-family)`). Policy details remain schematic.

The borrow-forming member's source formal head is not ordinary move-in
parameter semantics:

```lang
mut let ref =
    (self, mut let object: t):
    s -> _: t ref
    => default;
```

Here s is schematic for one admitted concrete stage, not a resolved union;
projection and readiness obey the ordinary Policy owner.

The head's `object : t` displays the candidate head, Pattern, and policy so the
member participates in ordinary overload resolution. It must not be read as
`Read(actual) -> move/copy -> fresh ordinary parameter object`: at application
the builtin member may obtain the corresponding place from the actual. This is
the explicit formalization of the privileged actual-place semantics that the
generated/builtin `ref` / `share` callables already had, not a new arbitrary
place-reflection facility.

#### 2.0.1 Group and named-type observations

An ordinary OverloadGroup reference observes the actual group Place. It does
not implicitly borrow a candidate type or manufacture an OpenHere target.
A structural name denotes its named type, whose type reference and member
projections use ordinary Place, policy and capability rules.

Call projection uses the singleton type embedding into a candidate group.
Neither group callability nor an absent call candidate creates name freshness.
Type selection, borrowing and lifecycle observation retain their independent
relations; they do not recover a source Place from equal type values.

### 2.1 `@` yields a lifetime value, uniformly

```text
E@ : LifetimeValue
E@ = ReifyLife(NameOf(E), Pos(SemanticContinuation))
```

`@` is never a bridge from a hidden carrier slot to a `type ref`. A type-valued
binding evaluates to the complete closure `tau`; an ordinary/namespace consumer observes the projection `Core(tau)=Q`. Reaching the type-level
place is done explicitly with `t |> (type ref)`, including for a structural
named type, never by `@`. NameBinding is a binding relation and has no borrowable
wrapper or `.type` field. A by-value type projection followed by `@` does not
recover a Place.

`LifeName` is not a source spelling, NameBinding, Place, or ordinary
value identity. It is the semantic name whose lifecycle is observed. A bound
value normally has a stable LifeName; a temporary may receive a generated one.
This keeps place identity and lifecycle identity distinct.

#### First-class `LifetimeValue`

A value observation of `ReifyLife` produces an ordinary first-class semantic value, not an ephemeral
observation-only expression species:

```text
FirstClassLifetimeValuePrinciple:

ReifyLife(n, k) : LifetimeValue

LifetimeValue participates in the ordinary value formation machinery:
  binding
  storage in an ordinary value slot
  argument passing
  return
  Move or a clone-derived route when admitted by its ordinary Type/callspace
```

Each use remains subject to the same Type, Policy, capability, lifecycle, and
destination-legality judgments that apply to another value at that point.
First-class therefore does not mean unrestricted storage, implicit copying, or
automatic runtime availability. Binding or transporting `E@` carries the value
already reified at `Pos(SemanticContinuation)`; it does not silently re-run `@`
at the later destination and does not turn the destination into a hidden
LifeName, Place, or borrow view.

```text
StageDoesNotChangeLifetimeValueOntology:

TypeOf(ReifyLife(n, k)) = LifetimeValue
  independent of the stage at which the value is known

compile-known LifetimeValue
  =/= a distinct CompileLifetimeValue species
```

Whether a `LifetimeValue` can make a same-Type static-to-runtime transition is
therefore decided by the ordinary materialization theorem, not by first-class
status and not by a lifetime special case:

```text
RuntimeMaterializable(LifetimeValue)
iff exists an ordinary non-deleted callable realizing
    a legal same-Type static -> runtime transition of LifetimeValue
```

This is the general `RuntimeMaterializable(T)` rule owned by
[`../symbol-world/type-values-places-and-borrow-views.md`](../symbol-world/type-values-places-and-borrow-views.md)
§4.1. Applying `@` is still not an automatic mechanical pass. Once `@` has
produced a `LifetimeValue`, ordinary transport of that result uses the same
admitted Move and ordinary clone realization as another value.

#### 2.1.1 One semantic continuation

All compile, runtime and lifetime activity is projected from one
semantic continuation:

```text
SemanticContinuation K

pi_runtime(K)
pi_compile(K)
pi_life(K)

Life : LifeName × Pos(K) -> LifetimeValue
```

Compile-known and runtime-known values do not inhabit different lifetime
species. Stage says when a fact is known; it does not change the lifecycle
calculus or numeric/value identity.

K as a whole projects to lifecycle computation, forgetting the **business
Policy's Stage dimension**. It does not create compile and runtime lifecycle
timelines. The projection retains already selected operations and MoveEffect,
names, dependencies, activation, control, cleanup and delivery correspondence.
It never selects again after forgetting Stage. Compile evaluation is the
ordinary hosting of this value program, not another stage axis inside its world.
Reify resumes the same E/K and creates no separate lifecycle world.

Current established names and structural material can be computed before their
machine action runs; unknown business values remain unknown. E uses the actual
legal frontier and guarded continuation, without future-path enumeration.
Constructing guarded branch material does not commit both branches' effects
or permit lifecycle code to prune paths using unknown business values. A legal
O path rewrite revalidates affected projections at their new positions.

`lifetime pre/post` programs are projection-complete regions. Their ordinary
evaluation and check/admit boundaries remain, but transcription adds neither
another @ nor another companion layer to their lifecycle-side operations.
Established projected material must retain that completion provenance through
composition; it is not inferred by matching names or traversing source text.

#### 2.1.2 NameView, origin, and finite observation

For every semantic value operation `f_V : T -> U` that is name-projectable,
there exists a unique semantic name-level companion:

```text
NameView<T> {
  origin,
  region,
  name-observable field/operation companions
}

NameProjectable_T(f)
  => exists unique f_N : NameView<T> -> NameView<U>
```

Value and name projection must commute:

```text
@(f_V*(x))     = f_N*(x@)
@(field_V*(x)) = field_N*(x@)
(x.field*)@   = x@.field_lifetime*
```

The stars mark the original selected operation and its corresponding partner;
they are metanotation, not a source operator or second overload selection.
The name-level companion does not re-execute the runtime field operation.
Companion existence, uniqueness, and commutation are closed semantic
requirements. How companions are generated, stored, or represented in an IR or
registry remains implementation-open. Actual Pattern/structure formation must
synchronously establish sufficient companion material for this law. A missing
consumer is Unavailable, never permission to fabricate the partner or silently
waive the law. Dot still enters field::adl, and any forwarding is real ordinary
registration or Pattern extraction, not a private field resolver.

`origin` observes a LifeName link, explicit None, or an actual unknown boundary.
It records semantic provenance rather than
source text or an address. Origin ancestry is generated coinductively by the
default generator of the semantic construction that produced the name;
omission is not `None`. `None` is an explicit termination proposition. Each
`ReifyLife(n, k)` is nevertheless a finite structural observation. A later
`.origin` projection issues a new request; `@` never eagerly unfolds the whole
chain.

Unknown marks the information boundary of established material. A missing
producer/observation is instead Unavailable; it establishes neither a fresh
unknown root nor None. Boundary programs perform the necessary available
origin.origin traversal (§10); callers need not manually spell an arbitrary
number of links in from/live_with.

```text
OriginDefaultPrinciple:
  if no explicit origin(n) = None fact applies,
  origin(n) continues through the default origin generator
  of the semantic construction that produced n

  origin(n) = None is a terminating fact,
  not the interpretation of missing origin information
```

Construction, field projection, copy, move, and call-result formation may each
provide the applicable name companion or lifecycle post. The general default
therefore does not classify an ordinary non-borrow value as origin-free merely
because no borrow-root rule applies to it.

Function entry begins from a default universe and applies the written lifetime
precondition as a finitely described correction:

```text
U_entry = U_default ⊕ Delta_pre
|Delta_pre| < infinity

DefaultOriginUniverse:
  exclusive-write borrow roots e_i, e_j
    -> DefaultRoot(e_i) and DefaultRoot(e_j) are anonymous
    -> i != j => DefaultRoot(e_i) != DefaultRoot(e_j)

  shared-read borrow roots s_i, s_j
    -> G_shared is one anonymous LifeName
    -> DefaultRoot(s_i) = G_shared
    -> DefaultRoot(s_j) = G_shared

  therefore exclusive-write defaults are pairwise distinct while all
  shared-read defaults have the same root; a finite Delta_pre may override
  only the relations it states

  borrow name tree
    -> mirrors exactly the subname edges already admitted by
       the established access relation

  structural membership alone
    =/=> lifetime-name reachability
    =/=> access capability

  no alias-write/internal-mutability edge is synthesized here;
  when the independent access relation admits no such edge,
  the mirrored alias-write subtree is empty

  unmentioned origin ancestry
    -> continue the applicable semantic default generator
       unless an explicit origin = None terminates it

  function-entry borrow roots
    -> additionally receive the exclusive/shared default-root rules above

PreDoesNotConstructUniverseFromZero:
  Delta_pre is a finite patch over U_default,
  not the complete origin universe
```

Here `⊕` applies the finite declared corrections while retaining every
unmentioned default rule. Thus a missing Pre fact cannot silently truncate an
origin chain to `None`.

`|Delta_pre|` measures the finite **description material** delivered by one
completed formation/observation: its ordinary bound names, relation rows and
finite regular descriptor graph. It is not the cardinality of the descriptor's
path denotation. A finite descriptor using repetition may describe infinitely
many potential finite walks. It does not enumerate or grant simultaneous
operations on all of them. Each actual requested operation still requires its
real access-tree route, target/generation, Region and Color premises; descriptor
state reuse merges none of those identities.

Only the admitted regular relations and their justified algorithms may be used.
Finite representation proves neither decidability of an arbitrary infinite fact
set nor termination of an arbitrary Pre program or numeric-range query. A
nonterminating/unavailable computation supplies no completed Delta_pre. This
description-versus-denotation distinction also applies to §11's regular access
material and leaves the implementation/proof obligations of §14 intact.

Neither origin nor NameView is automatically added to a business Object's
Norm. Lifecycle Objects themselves have ordinary content normalization of
their fields and retained identities; equal business values do not merge names,
Places, lifecycle generations or credentials.

#### 2.1.3 Region and lifecycle events

The first-level region of one LifeName generation is a linear half-open
interval over the current semantic continuation K. NamePosition_K fixes the
region; future branches and their physical availability are not premises:

```text
Region_K(n) = R(NamePosition_K(n)) = [i, j)
events    = use | move | drop
```

`use` records an observation point. When MoveEffect=Kill, move rebuilds only the first-level
generation/Region and preserves the deeper origin exactly:

```text
MoveOriginPreservation (MoveEffect_K(old,m)=Kill):

old@ = { name = n_old, origin = o, region = r_old }
move(old -> new)
new@ = { name = n_new, origin = o, region = r_new }

new@.origin = old@.origin
k := Boundary(move)
k is a position in SemanticContinuation
end(Region_1(old))   = k
begin(Region_1(new)) = k

Region_1(old) = [i, k)
Region_1(new) = [k, j)
```

`Boundary(move)` is the single continuation cut occupied by the move event.
The old generation ends and the new generation begins at that same cut; no
unowned interval or implicit position lies between them. A future IR may split
one event into explicit `PrePos(move) < PostPos(move)` only if it uses those two
positions consistently for both interval endpoints; the one-position semantic
model above has no strict `>` edge.

The new generation therefore does not acquire the old generation itself as its
origin. Origin effects of copy-derived formation belong to the selected
ordinary share/rebind and clone lifecycle postconditions rather than to the
lifetime calculus as a universal copy theorem:

```text
ordinary source : T:
  shared_view := share(source)
  selected ordinary clone invocation on shared_view
    -> fresh complete result with the selected clone's lifecycle post
  Move(result)

source : T ref | T share:
  rebound_view := rebind(source)
  selected ordinary clone invocation on rebound_view
    -> fresh complete result with the selected clone's lifecycle post
  Move(result)
```

The selected clone-family realization owns the relation. A particular builtin,
default, or custom clone candidate may declare
`origin(result)=NameOf(source)`, `SomeInternalOrigin(source)`, a re-rooted
relation, or another legal lifecycle post. None is implied merely by the name
`copy`. The lifetime calculus consumes that selected post
through the ordinary Pre/Post boundary and adds no copy-origin equation.

Mechanical `copy(x)` evaluates ordinary share/rebind and the selected clone,
then transports the fresh complete result through terminal `Move(result)`.
The clone owns result formation/origin; terminal Move has its independently
fixed lifecycle effect. There is no opaque copy-producing action or compact
semantic alias. Candidate-specific origin effects do not authorize a pre-move
of `x`.

`drop` ends the outstanding lifecycle/cleanup obligation for the current
generation. Path-sensitive facts are represented by a region slice plus a
regular origin path, not by turning Region into an arbitrary CFG subgraph.

#### 2.1.3.1 Instance killability and move legality

```text
Killable_K(n)                  -- property of this lifecycle instance
MoveEffect_K(n,m) ∈ {Kill, Preserve}
Movable_K(n,m)                 -- legality at this action's frontier
```

These are independent judgments. Killable is not a Type trait; equal type
values, ZST layout, compile knowledge and compile provenance do not decide it.
Movable requires the ordinary selected action's access, borrow, capability,
origin and lifetime Pre. Movable_K(n,m) != Killable_K(n); Movable does not
imply Kill, and Move does not definitionally mean death. Nonkillability proves
neither Movable nor Copyable.

The effect is fixed before lifecycle observation. Kill ends the old generation
and begins the transferred generation at the same continuation cut, preserving
its deeper origin (§2.1.3). No separate destructor/drop is inserted for the
consumed old generation. Preserve requires the narrow proof that the subject
cannot legally die or that preserving it is observationally equivalent under
all admitted observations, including @, borrow/access capabilities, origin and
destructor effects. It is a legal Move effect, not clone, copy fallback, failed movement or
special non-owning syntax. Preserve does not invoke clone; a copy-derived
share/rebind -> clone -> Move(fresh result) path creates a fresh complete
object and is a different ordinary operation.

Alive/dead status alone introduces no observable branch. Death with no effect
under any admitted observation adds no observable semantics. If killing would
invalidate an established legal language fact, that Kill continuation is
illegal and must be rejected at Pre, before any mutation.

The ordinary realization is killing; Preserve is confined to that proved
exception. A stable compile root or global Val2 resident cannot be killed merely
by local transport of its observation. Moving a local instance into a legal
destination starts the destination generation; it never extends the old local
name's lifetime into a global one. Failed Pre cannot switch the fixed effect.

Compile, runtime, compile, type, Pattern and borrow instances all participate.
An ordinary ordinary type instance in the established domain follows its existing global-survival rule; this is not generalized to every dependency-bearing closure-generated tau (§8).
a compile-local type temporary can end. A stable compile result root, a local copy,
and a globally retained equal resident remain different lifecycle subjects.
Construction OpenHere neither extends a lifetime nor makes a subject killable.
The existing stable-instance rules include ordinary stable type values, compile
type values outside the current OpenHere domain, still-accessible Val2
residents of closed type structures and other stable/global residents.
Their local transport cannot kill those stable subjects; legal movement uses
Preserve while still checking Movable. These are instance/frontier facts,
not a type-wide Copyable or Killable trait.

Every expression occurrence, including a temporary without a Place, has its
continuation-relative LifeName. Lifecycle Pre/Post applies even when no source
@ is written. Value/name companions observe the same action positions; @
does not expose a compiler execution or scheduling trace.

#### 2.1.4 Cleanup placement precedes lifetime observation

```text
CleanupPlacementBeforeLifetimeObservation
```

Ordinary control-flow, ownership, and end-event semantics place cleanup/drop
events first, fix their points, and linearize remaining unordered events
at each same point in reverse declaration order. This last tie-break never
moves a point or overrides NLL/with/other precedence. Lifetime observation
then describes that fully fixed continuation. The NLL/lexical defaults and directed with constraints are defined in
[mechanical cleanup](../mechanical-lowering/mechanical-argument-passing-and-move-fixed-point.md#15-cleanup-placement-and-with).
It does not move cleanup to satisfy a constraint and does not participate in a
cleanup/lifetime fixed point.

#### 2.1.4.1 Atomic decomposition and destructor continuation

Construction and consuming extraction use the Pattern owner's atomic identity
boundary. A destructor receives complete children after parent decomposition:

```text
Pre(drop/extract)
  -> atomic ExtractCommit of the complete parent
  -> parent generation ends where the selected action so requires
  -> complete child identities become established
  -> ordinary destructor continuation
  -> children's ordinary NLL / with / use / move / drop
  -> Post / subsequent lifecycle observation
```

There is no partially destroyed parent, live-field bitmap or separate
destructor evaluator. Matching proofs and non-consuming views are not
ExtractCommit. An affine accessor likewise does not prove parent death.
Cleanup placement and same-point ordering are settled before @ observes this
continuation, and neither destructors nor @ re-solve placement.

#### 2.1.5 Pre-check and post-commit

Every fact-changing lifecycle action has the order:

```text
Pre(action, state)
  -> failure rejects before mutation
  -> perform action
  -> Post(action, committed_state)
```

The corresponding ordinary lifecycle program must reduce legally and deliver
the required result. This is the meaning of checking, rather than another
checker ontology. It does not tentatively publish facts and discover a failure
at function end. A failed precondition leaves prior state unchanged. Scratch
computation is not committed history, and earlier committed external effects
are not generally rolled back by a later failure.

#### 2.1.6 Call-boundary summaries

A contract clause hosts a complete ordinary compile expression:

```text
lifetime post a@ == b@ ref::lifetime::
```

@, == and ref::lifetime:: use ordinary interpretation and selected calls. Local
declarations, branches, recursion, calls and associated injection are permitted.
Intermediate values need not be NameExpr. The final delivery is a complete
single self-root type with the boundary's associated check/admit behavior (§9).
No special formula AST, separate logic evaluator, arbitrary implication solver
or user-written proof Object is required. The caller consumes that behavior
without expanding private internal history.

| Clause | Caller | Inside implementation |
|---|---|---|
| Pre | check | admit |
| Post | admit | check |

Ordinary calls map actual parameters, formed captures, result positions and
activation. Caller Pre check precedes the action; internal Pre admit cannot
repair its failure. Internal Post check establishes the delivered guarantee;
caller Post admit neither replays internal actions nor duplicates consumption.
Explicit Post constrains actual results/history; it cannot replace history.

An implementation body omitting Post enters the formation-time structural
summary transformation of §10; reached actual exits separately validate that
summary. It need not execute future branches to expose the contract. A handwritten Function
Pattern/function-pointer interface has already forgotten external structural
source authority; it cannot recover hidden implementation structure to fill in
a contract. The following defaults apply to that interface, not to body-based
automatic Post. An empty clause placeholder proves neither true nor NoDelta.

For a Function Pattern LHS interface F and actual callable contract A,
omission and compatibility have these meanings:

```text
Entry(F) = U_default op_plus Delta_pre
omitted pre  => Delta_pre = empty  // all U_default generators remain
omitted post => Post_F = NoDelta(S_before,S_after)
NoDelta = unchanged state on every lifecycle observation of the contract

LifetimeContractCompatibility(F,A):
  Pre_F implies Pre_A
  Post_A implies Post_F
```

These implications use the ordinary symbols, Region/Color/name mapping and
entry assumptions. Omitted Post is a strong guarantee: an actual postcondition
that cannot imply NoDelta rejects the interface assignment/extraction immediately,
before any invocation. Missing implication evidence remains unavailable.
This does not inspect require/body or enumerate overloads by lifetime.
The [Pattern owner](../patterns-overload/pattern-relations-observation-and-extraction.md#111-directional-function-pattern-consumers)
defines LHS extraction and RHS requests. RHS cannot add pre/post, but after
unique selection it still checks the callable's actual Pre, commits once under
the common transaction and publishes its Post. Dynamic failure never reopens.

The [call owner's self projection](../symbol-world/function-object-call-model.md#63-one-coherent-passing-transaction)
uses this same boundary: whole-self and projected capture input have compatible
observable consumption in one transfer transaction, without duplicate Kill.
Ref/share capture positions move handles, preserving their actual targets and
generations; Preserve is neither Copy nor an implicit clone.

Contract implication retains the displayed direction, over the exposed
observations and mapped entry assumptions. It is an obligation on the actual
ordinary programs and supported family, not a decidable algorithm for arbitrary
program equivalence. Missing evidence cannot count as success. In particular
non-NoDelta effects cannot be hidden in an omitted-Post Function Pattern, even
with compile-known Val1 of the projected callable.

#### 2.1.7 Region Color

Color remains a future constraint dimension, but its inheritance law is
closed:

```text
Color is a future-continuation constraint
Color(child) includes inherited Color(ancestor)
observation may slice the inherited region but may not remove the color

FiniteColorPrinciple:
  GlobalColorVocabulary is extensible

  for each committed compilation semantic universe Sigma:
    ColorSet(Sigma) is a finite snapshot

  Color is not an arbitrary proposition carrier

  compatibility / exclusion / exchange
    are finite, mechanically decidable relations over
    ColorSet(Sigma) and bounded observed origin topology

ColorExtensionInterface(new_color):
  declare compatibility(new_color, existing_color)
  declare exclusion(new_color, existing_color)
  declare exchange(new_color, existing_color)
  freeze those finite relation rows when Sigma is committed
```

Color is not automatically added to business Object core or Norm. Its values
and region fields are ordinary lifecycle Object content. Its concrete carrier and
source syntax remain implementation/surface questions. The global vocabulary
is not a closed language-wide enumeration: later definitions may contribute
colors and their finite relation rows. Once a compilation semantic universe is
committed, however, its ColorSet and relation tables are finite and fixed for
that compilation. Extension may express broader memory-access models and
computation-reordering constraints. Finite Color relations remain a bounded
mechanical foundation; this does not restrict full contract programs to
conjunctions of Color atoms or make Color an arbitrary proposition black box.

### 2.2 No implicit borrow formation

The overload boundary stays:

```text
NoImplicitBorrowFormation
```

Candidate adaptation, policy migration, and automatic argument passing never
form a borrow merely because a formal requires `T ref` or `T share`. The actual
must already contain the explicit `ref` or `share` result; a `@` result is a
lifetime value, not a borrow. Fixed points and legal weakening of an existing
borrow do not form a new target.

Borrow-constructor composition remains a `ref` / `share` fact, owned by
[`../symbol-world/type-values-places-and-borrow-views.md`](../symbol-world/type-values-places-and-borrow-views.md)
§5.3:

```text
Borrow_k(Borrow_j(q)) = Coerce_{j->k}(Borrow_j(q))
Target(Coerce_{j->k}(v)) = Target(v)
```

Equal-capability `ref ref` / `share share` are fixed points, `ref share` weakens,
and `share ref` has no candidate. Retargeting type values have the symmetric
fixed points:

```text
type ref rebind rebind   = type ref rebind
type share rebind rebind = type share rebind
```

Prospective navigation identity and formed-borrow identity are also distinct:

```text
ProjectionCoordinate(parent_place, selector)
  != ProjectionSlotIdentity(parent_resident, selector)
```

The logical coordinate remains available for later navigation. A projected
borrow binds the parent-resident slot selected at formation, including a slot
whose contents are `None`; `let` may populate that same slot without retargeting.
Wholesale parent replacement may invalidate that borrow but never retargets it
to a replacement slot at the same coordinate. Only `rebind` selects a new
target. Generation/version representation remains deferred.

## 3. Escape checking

A borrow view must not be carried where it outlives its own valid region. The
positive obligation is:

```text
Escapes(view, destination)
  = Region(destination) ⊄ ValidRegion(view)

Escapes(view, destination)  ->  the storing/returning operation is rejected
```

The destinations subject to this check are the ones that can outlive the origin:
storing into a longer-lived place, returning from a callable, capturing into a
materialized callable entity, and installing into global namespace material.

Compile invocation-generated names obey this same check. Stable invocation identity
and cache retention do not extend result residency or borrowed-target regions.
Admitted open input dependencies may bound an output's opening qualification;
that meet is not a replacement for ValidRegion. Ordinary compile directly returns
its instance type; a borrow retained in its Val2 must be valid in the result's
region. A borrow of an untransferred expiring local is rejected.
Result completion transfers only owned material and never promotes targets along
ref/share/rebind edges. See the
[invocation owner](../static-evaluation/compile-instance-invocation-and-result-delivery.md).

### 3.1 `ValidRegion` is a borrow-lifetime judgment

`ValidRegion` is determined by the target lifetime, the holder/destination
relation, and the view's ordinary capability. It is never a construction-Open
region:

```text
ValidRegion( type ref )   =  LifetimeRegion( Target ) ∩ RefCapabilityRegion
ValidRegion( type share ) =  LifetimeRegion( Target )
```

The two views differ in write capability, not in whether their pointee is Open:

| view | carries | requires pointee Open | may leave the target's lifetime |
| --- | --- | --- | --- |
| `type ref` | read and policy-bounded write | no | no |
| `type share` | read/observe only | no | no |

Lifetime validity does not establish write permission after Close. Initialized
type refs preserve their original borrowed generation/opening subject; direct
mut and explicit open-to-mut writes require current OpenHere as well as
Writable. Close defeats both routes and saved-ref writes. Neither view makes
a closed value admissible as the left input to `*`. See type/ref owner §5.2.2.

### 3.2 Borrow validity never discharges construction openness

The positive separation is:

```text
Γ ⊢ r : type ref  does not imply OpenHere_Σ(Read(r))
OpenHere_Σ(v)      does not imply that v has a writable carrier
OpenHere_Σ(v)      does not imply the current computation flow re-enters v
```

The third line separates static openness from live evaluation flow: reentry is
`OpenEvalReentry_κ(v)`, which additionally requires an active evaluation edge
into `v` — a stored `SymbolicReferenceEdge` (including `Self_τ`) alone never
establishes it. The reentry criteria are canonical in
[`../symbol-world/type-values-places-and-borrow-views.md`](../symbol-world/type-values-places-and-borrow-views.md)
§2.1.1.

A consumer that performs `*` must query `OpenHere_Σ(old_value)` even when the
value was read through `type ref`. The place-level `*=` wrapper in
[`../symbol-world/structural-type-formation-and-composition.md`](../symbol-world/structural-type-formation-and-composition.md)
§8 performs two checks independently:

```text
OpenHere_Σ(Read(r))
Writable_Γ(Target(r))
```

Returning or storing `type ref` asks only the ordinary escape question of this
section. A later `*=` may fail because the then-current value's open
window has closed (`WindowLive_Σ(v) = false`) even
though the reference remains lifetime-valid. Weakening to `share` surrenders
write capability, but does not alter the value's anchor or window state and never extends the
target lifetime.

## 4. `@` and lifetime rules never reselect an ordinary call

The boundary between lifetime work and ordinary overload resolution is frozen:

```text
ordinary overload selection must produce one unique candidate
lifetime rules validate that completed result and cannot replace it
```

Ordinary overload resolution runs to a unique candidate on type, pattern, and
policy grounds alone. Lifetime policy then validates that candidate. It may
reject the program; it may not choose a different candidate, reopen
type/policy overload resolution, introduce a second selection stage, or
establish a specificity ordering that competes with ordinary overload order.

This restriction applies to lifetime *rules*, not to `@` itself. `@` is
resolved by ordinary overload resolution like any other operation, while its
selected builtin performs the name/continuation observation of §2.1. Thus "no
lifetime overload stage" and "`@` has overloads" are consistent statements
about two different things.

The apparent circularity dissolves once the three steps are separated. They are
strictly ordered, and each one is finished before the next begins:

```text
1. ordinary selection inside the operand   -> the operand value/name relation
2. ordinary selection of `@` itself        -> one candidate: the lifecycle
                                              observation visible in the
                                              operand's policy stage
3. lifetime validation                     -> accept or reject steps 1 and 2
```

Step 2 uses the ordinary selector, not a lifetime-specific one; the policy stage
only bounds which candidate is visible. Step 3 never reselects steps 1 or
2 — that is the whole content of this section. So "the lifetime stage runs after
ordinary selection has completed" and "`@` is itself resolved by ordinary
selection" describe steps 3 and 2 respectively, and neither feeds back into the
other.

## 5. General dependency handoff

[Dependency observation and realization](../symbol-world/dependency-observation-and-realization.md)
owns dependency requirements and their semantic realization. Explicit closure
captures are one source of those requirements:

```text
DependencyRequirement
  -> SemanticRealization {
       source_life_name,
       source_place?,
       requested_access_view,
       origin_or_region_relation,
       storage_or_link_category
     }
  -> LifetimeValidation
  -> RepresentationSelection
```

Snapshot versus live reference is decided by semantic realization, before
representation/layout. No capture or other dependency enters a completed
closure through an uncheckable side channel. Formation and each later action
pass their actual dependencies and obligations to the ordinary lifetime checks;
a type/call/Pattern projection does not repeat initialization or recapture.

## 6. Open representation questions

The following representation and integration questions remain open without
changing the relations defined above:

- concrete Rust/IR identity for `LifeName`, event positions, generation ids,
  Region slices, lazy origin links, and summary compression;
- concrete Color carrier, the contribution/registration API for the extensible
  global vocabulary, and any future source syntax;
- diagnostics and caching identity for lifetime validation;
- concrete representation of borrow and selected clone realizations, closure
  ABI and environment layout. Terminal Pass=Move and copy-derived
  share/rebind -> clone -> Move are closed, not default-pass selection questions. The entry origin defaults, exact
  move-origin/Region boundary, and selected share/rebind-plus-clone realization
  lifecycle-post boundary above are fixed lifetime semantics.

This revision still defines none of the following: lifetime overloads as a
second selection step, lifetime specificity ordering, multiple-callable handoff
objects, ABI equivalence classes used for selection, or a lifetime-driven
ordinary resolver.

Related canonical contracts:

- [`../patterns-overload/overload-resolution-design.md`](../patterns-overload/overload-resolution-design.md)
- [`../symbol-world/policy-and-static-flow-projection.md`](../symbol-world/policy-and-static-flow-projection.md)
- [`../symbol-world/type-values-places-and-borrow-views.md`](../symbol-world/type-values-places-and-borrow-views.md)


## 7. Continuation transformations

Machine and lifecycle residue are projections of one continuation. After an
optimizer changes that continuation, old lifecycle facts can guide candidate
formation but cannot be copied to new positions. All affected projections
repeat their Pre/commit/Post validation. See
[evaluation and optimization](../static-evaluation/evaluation-residual-and-optimization.md).


## 8. Closure dependency lifetime refinement handoff

### 8.1 Handoff subject

```text
Eval(ClosureExpr_C) = tau_C
D = Dependencies(tau_C)
```

Closure formation fixes D's sources, Pattern/Policy observations, formation
actions and absence of recapture. Lifetime implementation and refinement own:

- the regions/generations in which D may persist;
- Move with Kill/Preserve and ordinary clone-derived transport of tau_C and its members;
- return, storage, escape and promotion;
- persistence of bounded runtime state alongside stable descriptions;
- the concrete regions in which automatically formed dependencies and their
  ordinary closure results may move, persist or escape.

These refinements do not block the defined closure formation semantics.

### 8.2 Preserve checks without inventing conclusions

Formation and later actions retain the existing interfaces:

```text
FormationLegal
LifetimeLegal
Pre / Post
MoveEffect / Movable
EscapeLegal
owned transfer / promotion checks
```

Established rules continue to decide actual actions. Unrefined cases retain
their explicit scope; neither temporary global lifetime nor a blanket ban on
all future local dependencies substitutes for the missing refinement.

None of the following implications is established:

```text
ClosureTau => GlobalLifetime
LocalDependency => not Returnable
OrdinaryClosure => Escapable
InPlace => not Bindable / Returnable / Movable
```

In-place results are already ordinary first-class results. Explicit versus
automatic dependency formation supplies no independent operation prohibition.
TypeRole, ZST layout, Core equality, cache reuse and the absence of machine
storage prove none of these capabilities.

### 8.3 Established ordinary type survival

Existing global-survival rules for ordinary types remain in their established
domain. Universal closure-to-tau formation must not mechanically extend those
rules into a new theorem globalizing closure dependencies.

The lifetime topic must integrate persistence and transport of complete
closure-generated results with the general dependency model. Existing facts
remain valid; syntax categories alone establish neither new capabilities nor
new prohibitions.

### 8.4 Deferral is not an implementation shortcut

The handoff does not permit omitted checks, undefined behavior or a claim that
bounded closures are implemented. Acceptance verifies preservation and delivery
of dependencies and action obligations; it does not require uniformly accepting
or rejecting every closure with local references.

Path, ADL, Policy and closure formation retain defined semantics through these
interfaces without deciding every future lifetime refinement.

### 8.5 Per-name SCC collapse at continuation boundaries

A business type does not determine a lifecycle value. Structural
interpretation involving `$` provides the *formed* type/Pattern's paired
lifecycle structural material; actual formation establishes the **original
regular companion descriptor for each name item** (§2.1.2/LC1).
`$` itself adds no lifetime action. A name-item SCC is collapsed at its
continuation boundary **directly into that original regular representation**
under the name's current access, region, Color and source/effect constraints.
It must not be unfolded indefinitely and reconstructed as a new SCC grammar,
nor inferred solely from the business type.

#### Participation relation

For this judgment I denotes an actual closure activation/boundary in K, not
just a source closure expression, lexical owner, callable tag or cached
CompileInstance. Define the **name-item tracking graph** G_K from the established
semantic material needed by this lifecycle computation:

- A vertex is an already formed name item, retaining its LifeName, generation,
  actual structural item/selector coordinate and original paired regular
  descriptor. A lawfully transported summary can retain that same provenance
  and the ordinary mapping to a new carrier/generation. This records no new
  LifeName and does not assert that a recorded old generation is still alive.
  Any genuinely newly formed item/generation has its own original companion
  from that real producer; transport never replaces it with another item's
  basis. Only actual ordinary transport/delivery supplies such mappings, never
  a result name invented at an intermediate return boundary.
  Potential coordinates, type nodes and regular automaton states are not
  vertices merely because they can be written or visited.
- A directed edge is exactly a fine-tracking dependency supplied by actual
  formed structural/source material or an established selected operation/
  interface relation: its source item's material requires the destination
  item's material under that witnessed relation. Retain the legal direction,
  registered name-level structural/access witness, actual target/mapping and
  scope/control conditions. Mere Val2 membership, a type signature or a
  dependency requirement without its selected realization adds no edge.
  A Move contributes only dependencies actually supplied by its selected
  counterpart/contract. It does not automatically connect the old/new name
  generations, add an old-name origin link or reverse an edge.
- Each vertex records the activation of its actual formation action; each
  edge records the activation whose formation/selected action or instantiated
  interface first establishes that dependency. These are contribution
  provenance, not reparenting of names or Objects. Reading/copying an existing
  row or propagating an unchanged target completion retains its provenance;
  it does not make every transit frame the edge's establishing activation.
- Ordinary scope/access/capture and return-segment judgments must admit those
  edges. In particular no control-propagation edge crosses the nearest
  Ordinary return barrier or an unknown boundary by assumption. Only legally
  exposed material counts; hidden implementation provenance cannot restore
  erased external structural-source authority.

Preserve guards when constructing this graph. A component C carries one
compatible guarded case; mutually exclusive arm edges cannot be unioned into
a synthetic cycle. At an actual exit use its reached mapping. At formation,
the same rule transforms already formed bound descriptions conditionally; it
does not create future concrete vertices, activations or cuts, inspect a future
guard, or enumerate future paths.

For each such case, C is a maximal strongly connected component of G_K with
a witnessed positive-length directed cycle. A singleton without an admitted
self-edge supplies no additional InPlace collapse obligation. Let Edges(C)
contain only edges whose endpoints are in C. The participation rule is:

```text
LocalContribution_K(I,C) =
    {v in C        | formation_activation(v) = I}
  + {e in Edges(C) | establishing_activation(e) = I}

Participates(I,C,K) iff
    I is an InPlace boundary with established applicable scope/control evidence
    and C is the cyclic SCC in the compatible case just described
    and LocalContribution_K(I,C) is nonempty
```

The two sets are tagged vertex/edge contributions; `+` is metanotation here,
not a new source operator. In a cyclic SCC every contributing vertex/internal
edge lies on a cycle. A noncyclic feeder edge, merely containing/reaching C,
forwarding C's material or lying on a target-return route does not suffice.
Neither recursive business-type syntax nor repeated regular-state visits nor
reuse of the same closure tag in another activation establishes participation.
Graph storage, SCC algorithms and caches are open; this relation is not.
Missing required graph, contribution, guard or scope evidence is Unavailable,
not an assumption of nonparticipation or a type-based collapse.

**The collapse schedule is fixed by closure semantics:**

- At an **Ordinary (non-InPlace) closure** boundary, collapse outgoing SCC
  material per name item. At formed-body summary time, transform *bound
  descriptions*, not future concrete actions; at an actual exit, transform
  reached names/history.
- An **InPlace** closure preserves fine-grained tracking by default. Only
  when it **participates in the particular SCC** do its participating
  name items collapse at that InPlace closure's real exit (including a
  legitimately crossing targeted return). Other InPlace closures do not
  collapse unrelated SCCs. Every real exit still performs its usual
  local-name compression.

A targeted return crosses only InPlace boundaries in the reachable return
segment and cannot cross its nearest Ordinary closure barrier. SCCs induced
by these synchronous return interactions are thus segment-local. This does
not prohibit ordinary captured/returned name dependencies across a callable
interface; their outward contract and escape checks remain mandatory.

#### One component at successive exits

Participation is not permission to replace all of C at its first crossed cut.
At I's exit collapse **I's boundary-local cyclic contributions** per name item:
the items formed there and the local relation material established there that
must be exported. If such a row concerns an outer name, transform that local
projection, not every fine representation of the name in surviving contexts.
The fine material of an outer participating activation remains available,
updated with the actual committed effects, until that activation's own required
cut. Nonparticipating InPlace exits still compress their dying locals.

The outward transform must retain the original descriptor association, actual
name/generation/delivery mapping, guards, current restrictions and sufficient
component incidence/contribution provenance to process later exits. An earlier
local collapse cannot erase an outer contribution and make the outer activation
falsely nonparticipating. An implementation may contract graph representations;
it must preserve this semantic correspondence, without reading dead locals or
reconstructing their fine state from a regular type/state. A component identity
need not remain fixed across genuine semantic graph changes; a representation
change alone cannot discard its required lineage.

If mandatory local collapse cannot retain the information needed at the next
boundary, the transform is unavailable at that cut. It cannot delay collapse,
collapse the entire component early, or guess at the next boundary. The
[multi-InPlace example](lifetime-algebra-walkthroughs.md#scc-collapse-at-participating-closure-boundaries)
derives the participation and successive-cut behavior from concrete graph
provenance, rather than assuming a participating/nonparticipating label.

Collapse does not merge actual names/generations/targets, turn unknown into
None, remove earlier Color/effects, lengthen region, or restore cropped
operation endpoints. The original descriptor is a *structural basis* with
current restrictions, not renewed permission. Missing original companions
or unsupported regular operations yield Unavailable, not type-based guesses.
This rule does not prove termination of arbitrary recursive lifecycle
programs or machine tail-call/slot reuse; those remain separate obligations.



Three subjects have separate obligations: regular access/source SCCs (§11),
recursive ordinary lifecycle programs, and physical tail-call/slot reuse.
Regular descriptors preserve finite structure from formation. Their state
reuse does not merge actual names, generations or targets and proves no
arbitrary program termination. Ordinary recursion uses E and exposed contracts;
each supported family must justify its domain, operations, closure and
termination or explicit rejection/unavailability (§14). No external general
solver, universal fixed point or total convergence theorem is assumed.
Machine consumers use already established evidence; they cannot choose an
answer, retry an overload or switch ABI. Cleanup remains fixed before
observation with no cleanup/lifecycle repair loop.

## Current name position and delayed E

```text
Region_K(n)=R(NamePosition_K(n))
```

Region follows current semantic name position, independently of future branch
availability and physical runtime timing. E uses current Pre/commit/Post and fixed
cleanup observations, without future-path enumeration. A closed callable reify P2 boundary resumes its reverse-dependent work in the same E
at args-in. Origins remain relations inside compile name-position algebra.
Runtime strings create no authority to redefine fixed structured-name coordinates.
O may analyze future paths and specialize with affected E revalidation.

The lifecycle projection is compile name/Color algebra for both compile and
runtime callables. Future value facts such as a constant-false branch are not
visible to name/Color reasoning by themselves; an optimizing path removal uses
E O E with the affected projections revalidated.

## 9. Ordinary contract formation and bootstrap responsibilities

The ordinary expression in §2.1.6 runs with the existing compile invocation
rules. Its selected partner and canonical inputs establish a CompileInstance
I before body entry. It uses the ordinary compile self-name/projection and
actual OpenHere window to form complete structural material and install
associated check/admit behavior. A legal instance-open result is the single
direct self-root complete type tau_I, with ordinary NameExpr exposure. An open
type hidden inside a Product/reference/capture cannot escape. No explicit
user-created contract wrapper or two manually duplicated functions is required.

The behavioral interfaces are schematic full Function Patterns:

```text
check: world::lifetime:: -> bool::                 at compile horizon
admit: (world::lifetime::) ref -> unit::            at compile horizon
```

Each associated callable has its own contextual self. Any contract subject or
mapping is ordinary explicit input/dependency material, not an extra implicit
self slot. Source definitions may use ordinary open qualified member formation
and initialized associated entries; check/admit need neither a new registry
nor a special contract interpreter. Their concrete arity, packaging and
member spellings are engineering choices. The boundary requires the stated
observation and authorized update behaviors, with every ordinary Type, Pattern,
Policy, capability and lifecycle premise.

There are two independent authorities. Contract formation may modify associated
members only in its real construction window and at actual retained Places;
updating an outer value copy does not update instance storage. Calling admit
may update only the corresponding mapped lifecycle world at a checked
entry/success boundary. Open + OpenHere completes an omitted mode to mut but
supplies neither Writable nor that world-update authority. Ordinary admit
derives boundary consequences from established assumptions/results; underivable
external facts additionally use the unsafe owner's post-directed admission.
Neither can retract history or make a failed caller Pre legal retroactively.

The minimum bootstrap responsibilities are below. They are inputs/observations
of existing actions, not a requirement for ten VM instructions or new Objects.
Every seed needs its action's Pre, actual cut, common commit, joint Post and
failure-without-publication boundary before it can support library composition.

| Responsibility | Established input / output and consumer |
|---|---|
| Name and generation | Actual formation/observation supplies LifeName, generation and current K position, including temporaries without a Place |
| Origin | Selected formation/transfer supplies links, explicit termination or established unknown boundaries, plus legal field observations |
| Region and Color | Fixed cuts and interval relations, finite directed compatibility/exclusion/exchange rows and inherited historical constraints |
| Move and borrowing | Predetermined Kill/Preserve, frontier Movable, actual ref/share/weakening/rebind targets; no implicit borrow or copy fallback |
| Complete identity | Construction, consuming extraction and non-consuming Pattern observation produce their distinct companion material atomically where applicable |
| Invocation and exit | Selected Param/captures, explicit input, actual activation, delivery, target return and fixed cleanup correspondence |
| Pattern formation | Real structural formation establishes paired field/operation material and registrations for the commutation law |
| Regular access | Finite choice/advance/repetition/crop material retaining targets and operation permissions |
| External success | Compatible trusted facts tied to the successful external action and its mapped outputs |
| Boundary transform | Formed guarded relations/bound local positions for advance summary; actual exit/identity/origin/region/path/effect/world material for reached compression; fidelity between them |

Ordinary lang programs compose these materials into internal APIs, contracts,
access refinements, compression, automatic Post and library protocols. Source
consumers establish the correspondence and enter common transactions. The
materializer implements promised external behavior on its target. It cannot
decide lifecycle meaning from representation or layout. Missing seeds/consumers
are explicit unavailable boundaries, not successful empty summaries.

### 9.1 Bootstrap dependencies must terminate

The dependency here is justification of an action, not merely evaluation order.
Its leaves are the already fixed language semantics A0: ordinary Object/Pattern/
Policy and name/Place operations, selected call/activation/delivery, complete
formation, fixed cleanup, the defined Move/origin/Region/Color/access relations,
and their common Pre/commit/Post. Those laws do not obtain their validity from
a contract that a derived program is currently trying to form.

| Dependency | Required predecessor | Cannot serve as its own predecessor |
|---|---|---|
| Bootstrap observation/action | A0's actual name, target, cut, operation and ordinary authority | A newly constructed contract saying that the observation/action is valid |
| Current-instance type initialization | Actual fresh formation window and one-shot initialization authority at InvokeName(I) | Reading an uninitialized self resident, or returning an unrelated type and declaring it rooted at I |
| Derived ordinary lifecycle program | Available inputs and selected operations whose premises already have a foundation | Its own still-unformed check/admit, automatic Post or generated companion |
| Boundary adoption | Actual mapped entry/committed-success facts and permitted ordinary update operations | The Boolean returned by a user check, or a copy of a proposed world |
| Future source consumer | The same established relation and corresponding ordinary program | A fixture import, missing wiring, a speculative execution or successful empty summary |

A bootstrap implementation may initially realize a justified observation or
formation operation in the host; a later source definition may replace it when
its dependencies already terminate in this basis and it preserves the same
relation. This does not freeze an intrinsic count or representation. It also
does not permit an unrestricted host fact setter under a source function name.
The [host boundary](../static-evaluation/host-capabilities-and-machine-objects.md)
still requires justification for a permanent non-source primitive.

Derived programs' own ordinary computation has lifecycle premises too.
Projection-complete provenance stops another transcription/companion layer; it
does not waive those premises. Finite structural operations and the basis above
anchor that computation. An adopted recursive family must separately establish
its domain and termination/composition condition (§14). Recursive **execution**
under an established contract is different from circularly granting that
contract correctness. A strongly connected justification with no such foundation
is unavailable, even if every node promises to admit the next one's assumption.

In particular, forming C cannot use C.check, C.admit, AutoPost(C) or a companion
whose sole justification is C to establish that very formation's Pre. Omission
does not supply true, NoDelta or a default root at this gap. Ordinary scoped
recursion and E remain the only program evaluation; no proof Object, external
prover, privileged success flag or extra evaluator is introduced.

#### Operation provenance map

The following map classifies the walkthrough names by their **semantic source**,
not by which Rust function might initially implement them. B/L/X are justification
sources; W records a missing consumer and is orthogonal to them. One spelling may
therefore be ordinary composition over a basis observation with source wiring
still pending. None of these rows freezes an API or an intrinsic count.

| Class | Meaning / endpoint of justification |
|---|---|
| B: basis responsibility | The already fixed A0 observation/action laws and actual authority: names, source-established origin, actual cuts/regions, directed Color/access facts, complete formation, borrowing/Move and common publication. A basis fact is not produced merely by copying its carrier |
| L: ordinary lang composition | Pattern extraction/construction, calls, arithmetic/Boolean computation and supported relation transforms over those available facts. Its dependencies must terminate in B or actual compatible X facts |
| W: unconnected consumer | The source-to-semantic handoff for an existing relation/program is missing. W grants no fact and cannot be implemented by substituting another semantics |
| X: trusted external fact | A real facility supplies otherwise underivable resource/validation/exclusion facts by compatible unsafe Post after its ordinary Pre and actual success; failure supplies no success fact |

Namespaces below are the schematic ones used in the walkthroughs. Grouping
operations means they share a source, not that their bodies may be one opaque
host operation. The final column points to the existing pending handoff.

| Example operation(s) | Source / ordinary expansion and final dependency | Pending handoff |
|---|---|---|
| @, observe_origin | B supplies actual name/generation/origin observations; ordinary field/Pattern reads expose their existing link/unknown/termination material. observe_origin is not permission to invent that material | W: LC2/LC3 and ordinary name/reification/field consumers |
| split_link, split_row, split_outward_row, split_exit, select_one_parts, contract_rows | L: registered ordinary extraction of complete descriptive values. B is their actual formation/role/identity and extraction law, not source spelling or fixture import | W: LC1/LC4/LC5 |
| origin_accum, prepend_row, exit_material, guarded_exit_pattern | L: ordinary complete Products/registered constructors retaining bound roles, guards and all effect rows; B is complete construction and the input material's origin | W: LC1/LC5 |
| compose_link, retain_unknown, retain_termination | L: compose one available relation edge or retain its actual endpoint, using B's region order, inherited Color, access and source facts. Each adopted relation/algorithm needs its stated domain; no general solver is assumed | W: LC5/LC8 |
| map_outward_row, map_delivery | L: eliminate local reads and apply the established actual/bound delivery mapping. B is ordinary scope/return/result identity; no new target, validity or authority is synthesized | W: LC5 and ordinary delivery |
| compress_origin, compress_rows, compress_exit | L: the explicit recursive W1 programs over the preceding operations and justified finite/regular families. They are not irreducible lifecycle primitives | W: LC5/LC8 |
| source_frame, selected entries, function_boundary, true_boundary, false_boundary, middle_boundary, inner_boundary, available_guard | W exposes existing semantic body/control/call correspondence. B is the actual formation/activation/cut/selection and legally available control observation; Frame-shaped user data or unknown future control supplies none | W: LC2/LC3 and control/completion |
| share_fuel, is_zero, predecessor, move_leaf_result, recursive_input, return_exit, returned_call_exit in selected | The original selected ordinary operation and its established counterpart/contract determine these projections. L composes them; B supplies actual borrow/Move/formation/return laws and single transfer. Numerical helper meaning stays with its original ordinary callee, not a new lifetime builtin | W: LC2/LC3 and ordinary callee execution |
| transform_formed_arm and the advance summary | L: compose established operation descriptions, eliminate bound locals and retain guarded effects. B is actual semantic material formation and counterpart fidelity; an exposed modular contract can replace unavailable private structure | W: LC1/LC2/LC5/LC6, including adopted recursive-family evidence |
| initialize_empty_root::construction | B is general complete type formation at the already authorized current target and one-shot initialization, not a special lifecycle type factory. The example source facade exposes that existing action; its NameValue argument creates no permission | W: LC4 and current-instance initial formation |
| qualified let, explicit child ref, first write, captures, self return; form_post | L: the expanded W2 construction/call program. B is ordinary OpenHere/name/Place/initialization, closure dependency, self-root and return rules; it cannot justify its own initial Pre | W: LC4 and ordinary source construction/delivery |
| clone::lifetime | An actually selected ordinary share/clone/Move path over immutable descriptor data. B is its real producer Post and complete result formation; cloning data copies no semantic-world authority | W: LC3 and ordinary clone producers |
| source_matches, region_fits, color_compatible, access_is_subset, effects_match, all_true/both | L: mapped observation, supported finite relation queries and Boolean folding. B is the actual source/history/region/Color/access evidence; a true answer creates none of it | W: LC4/LC8 |
| adopt_origin, adopt_region, adopt_color, adopt_access, adopt_effects, end_boundary_view | L: map/extract the relevant rows, check their independent operation premises and thread/end the same scoped view. The B leaf is the narrow existing authority-checked projection update and joint-publication boundary, not an unrestricted fact setter or the whole adopt function. New external facts additionally require X | W: LC3/LC4; X enters only through LC7 |
| try_trust, partition_refs | L: ordinary observation, control, range planning and ownership/result assembly; B supplies actual input/source authority and ref formation. X is only the otherwise underivable successful validation/disjointness fact and its maintained interval, not the entire function | W: LC3/LC7/LC8 and materialization |
| with_ref_window, trusted alias-to-value conversion | L organizes callbacks, ordinary operations and cleanup; B supplies real normal ref/value formation under Pre. X maintains actual validity/exclusion/ownership that alias names or fresh anonymous origins cannot prove | W: LC7 and normal escape/borrow/delivery |
| owned_parts, part_snapshots, pair_fields, plan_repartition, same_resource/same_generation, amount queries/arithmetic, is_full, split_ready, merge_parts and result constructors in W4 | L: ordinary contract/field observations, arithmetic, consuming extraction, resource accounting and complete result formation. B supplies actual operand capabilities and ordinary lifecycle/construction laws. Quantity or an equal id alone mints no part | W: LC3/LC7 and ordinary library/type consumers |
| validate_for_plan and into_box in W4 | L checks available library state and organizes conversion; X is needed only for otherwise underivable actual resource-generation/validity/exclusion/ownership facts. B still requires every normal consuming/construction Pre; full quantity is insufficient | W: LC7 and actual trusted facility/normal Object formation |
| issue_full_part and initial resource acquisition in W4 | L transfers actual unallocated full-use authority into a complete part under B's normal action/ownership laws. Only a real external acquisition needs X for otherwise underivable ownership/validity facts. The amount 1, a Part constructor or a success tag is not an authority source | W: LC7 and ordinary resource acquisition/consuming formation |

An implementation must expose this decomposition even if a temporary bootstrap
realization uses host code. In particular compress_exit, form_post, adopt_origin
and partbox are not justified as permanent indivisible host black boxes. The
ordinary programs should consume the smallest justified observations/operations;
unconnected consumers stop unavailable. No row authorizes deriving a pending
contract from itself, publishing a supplied callback as proof, or replacing
ordinary bodies with source-string recognition. The [library composition example](lifetime-algebra-walkthroughs.md#ordinary-partbox-composition-program)
shows where L, B/W and X meet on a concrete path.

### 9.2 Safe adoption is narrower than mutable observation data

The world-ref interface of §9 denotes an actual authority-scoped boundary view,
not unrestricted setters for every rank-0 field in semantic history. The actual
invocation supplies its activation, input/result mapping, generation, cut and
entry/success scope. Ordinary safe operations on that view must independently
obey their own Pre, even when the surrounding contract's check returns true.
They can transport assumptions already justified at caller Pre, import an
established callee guarantee after successful completion, or derive consequences
through admitted operations. They cannot turn arbitrary computed data into a
new premise. Acquiring or copying a NameValue or a bool supplies no authority.

For example, an ordinary origin-adoption operation must check the proposed
source against the permitted mapped entry/success relation. Region adoption
must retain actual generation cuts and target validity; Color adoption cannot
retract inherited/history facts; access adoption must preserve the admitted
operation ranges and non-expansive cropping. These are the ordinary operations'
semantic premises, not another call to the possibly dishonest C.check. The
boundary checks the full correspondence and all affected Pre before joint
publication. A stored view reused for a foreign activation, cut, generation or
output has no authority there.

Implementations may express this restricted surface through ordinary access
views, operation families and capabilities. The carrier and function spelling
are open; an API exposing an unrestricted safe semantic-world record write is
incompatible with this rule. A function named admit and a well-formed self-root
type do not supply missing write permissions. A successful check establishes
its bool result only; a legitimate contract's useful guarantees come from its
actual sound computations and mapped operations, not from that name or bool.

The [worked contract and malicious variant](lifetime-algebra-walkthroughs.md#ordinary-self-root-contract-program)
make the non-derivability concrete. A safe always-true check can accompany an
admit with no unauthorized delta. Pairing it with a forged origin/region/Color/
access update fails the corresponding operation Pre or requiring consumer;
missing validation is unavailable. No successful permission is published.
Unsafe Post remains the separate route for compatible **new external success
facts**, after ordinary Pre; its truth is an external facility obligation.

### 9.3 Source construction and common publication handoff

The [ordinary contract program](lifetime-algebra-walkthroughs.md#ordinary-self-root-contract-program)
starts by requesting complete initial formation at its own existing invocation
root, then uses normal qualified member formation and first writes, then returns
compile self directly. The [construction owner](../symbol-world/structural-type-formation-and-composition.md#current-instance-initial-formation-handoff)
owns that initial formation; no local copy is substituted for the target.
OpenHere governs later member formation, each child has its own initialization
authority, and a helper has its own self/activation. Construction authority is
never stored as a general world-writing privilege in the resulting contract.

check/admit evaluation participates in the same E. Safe admission computes the
mapped update on the existing transaction's scoped projection storage; internal
component calls do not publish a partial outer boundary. All required checks
and component premises must succeed before its joint public publication, with
the original action/cut/mapping retained. An invalid later component publishes
no part of that boundary delta. This storage discipline is not rollback of
earlier independent actions or external effects. A common transaction callback
alone proves neither source authority nor correctness of supplied material.

## 10. Internal actions, origin compression and automatic Post

Internal programs distinguish sibling transfer from child formation, real
borrowing from ordinary value formation, handles from targets, consuming from
observing, and Kill from Preserve. Origin alone grants no borrow capability;
ordinary values may carry logical dependencies. Internal Move does not simply
call a precompressed public from. There is no automatic reborrow to arrange a
desired origin tree. Pass=Move and selected clone-derived paths stay unchanged.

Automatic Post has two connected consumers at existing semantic boundaries.
These are ordinary computations in E, not additional evaluators, Policy ranks
or a requirement to execute the whole program before analyzing it:

| Boundary | Available input | Result / effect |
|---|---|---|
| Contract formation from a formed body | Semantic body Pattern/structure, bound input/result/local positions, guarded alternatives, fixed scope/return relations, and established selected-operation descriptions or exposed modular guarantees | A conditional outward Post summary; no future action executes or publishes facts |
| Actual continuation exit | The reached path, actual activation/names/generations/cuts, committed history and available lifecycle observations | Local compression and actual Post material, checked against the instantiated summary before successful outward adoption |

### 10.1 Contract-formation structural transformation

Ordinary semantic function-body formation supplies the first input. It preserves
the body's guarded structure and attaches the lifecycle structural descriptions
of operations already determined by the ordinary call/Pattern rules. Those
descriptions express the operation's admitted transfer relation; obtaining them
does not invoke the described action or its state-changing companion. A future
call can instead use an already exposed sufficient Function Pattern contract.
Neither route restores erased external structural-source authority or chooses
a future candidate. Raw/Normalized AST shape or a matching function name alone
supplies none of this semantic material.

These body/operation descriptions are complete ordinary material values formed
by their own actual actions; they are not readable partial callable Objects.
The final callable contract can still await their transformation. Their
formation premises must terminate in §9.1's basis independently of that pending
summary; a cycle through the summary itself remains unavailable.

Before the automatically summarized callable contract is exposed for use, an
ordinary lang program transforms that **formed guarded material**: compose the
admitted operation relations, remove outward dependence on bound local positions,
retain source/guard/effect correlations, and form the self-root summary type
with its ordinary check/admit behavior. Bound local/return positions and
parameterized region constraints are descriptions, not freshly committed future
LifeNames, generations or cuts. Ordinary call mapping later instantiates them.
The program may transform both alternative **descriptions** without performing
either alternative's moves, borrows, cleanup or external effects. An unknown
business guard remains a guard in the resulting conditional material.

This fixes the source of the summary input and its availability boundary; APIs,
representations and supported transformation algorithms remain open. It does
not promise that every body has a computable summary. If a required selected
description, modular guarantee, legal structural transformation or founded
recursive-family result is missing, summary formation is Unavailable. The
consumer cannot expose a supposedly complete automatic contract by guessing,
using NoDelta, waiting for one future path as its only justification, executing
future branches, or reopening selection. Only an actually established remaining
formation continuation may be transported as such.

### 10.2 Actual-exit transformation

Every actual block/call/other delivery boundary that exports lifecycle material
must remove its local-name dependencies while the internal material is still
available. The ordinary boundary program takes:

```text
actual exit + input/result/activation mapping + local/external partition
+ formed origin/region/Color/access/source-choice material
+ all observable selected actions and fixed cleanup
  -> available origin traversal and relation composition
  -> outward material without local-name reads
  -> self-root Post type and associated behavior, where successful
```

Traverse origin.origin to the first **actual unknown**; explicit None stops
as termination. Missing material is Unavailable. A known regular cycle uses
its finite regular representation rather than infinite unfolding, with the
applicable recurrence obligation of §14. It cannot manufacture unknown to hide
a cycle/failure. Contract users do not manually specify the traversal depth.

Compression eliminates local references from the outward material, not the
already committed history. It preserves required region order, Color, access
paths, external/unknown-root identities and correlations between source choice
and effects. All retained validity constraints still apply. Removing a local
carrier cannot lengthen an expiring referent, restore cropped access, revive
a dead generation or cancel Color. A true borrow of a dying local still fails
escape. Nested boundaries perform this work at their real exits rather than
waiting for the outermost function.
For SCC material, the placement rule of §8.5 applies. Nonparticipating
InPlace exits still compress dying locals, but do not collapse unrelated SCC
name items.

The actual-exit program produces Post_actual for the reached path and checks
its correspondence to the already formed Post_summary. A more precise actual
contract may be formed from that material, but it is not the missing source of
an advance automatic summary. Both transformations use ordinary Pattern/material
operations, not inference of arbitrary weakest preconditions from predicates.
Only defined choice/result combination relations apply; there is no universal
join. For select_one's admitted family, transform the formed descriptions of
each legal arm, then form correlated source choice:

```lang
lifetime pre  o@..live_with(r1@ || r2@)
lifetime post o@..from(r1@ || r2@)
```

`live_with` is a candidate spelling for depends_on. The Pre's o is the signature
result position, not a precreated Object. The Post's source component records
one selected origin, never two simultaneous exclusive targets. Any other branch
effects remain in the full Post. A from check may use an interval-dependency
operation, but its admit must also respect the actual source/transfer result;
the two roles are not undefined synonyms. The [worked example](lifetime-algebra-walkthroughs.md#2-select_one-and-boundary-origin-compression)
states conditions for compress-then-combine. No unconditional exchange law is
asserted between compression and choice.

Normal result delivery, actual unit fallthrough, target completion and failure
are distinct exits. A target return crossing InPlace boundaries transforms
only its real exiting material and history. It creates no local unit or result
binding there; the matching target alone performs ReturnPattern delivery.
An unfinished or failed action cannot produce a successful Post. Existing
InvocationResult transports the actual result, Diagnostic or established
remaining continuation. No missing implementation is called a residual.

### 10.3 Summary and actual-exit fidelity

Every supported summary family must preserve, under the ordinary actual call
mapping theta_p, all exposed observations on every legal successful path p:

```text
Post_actual(p, theta_p) implies Post_summary(theta_p)
```

This is the obligation on the ordinary transformations and exposed behaviors,
not a separate contract language, user-supplied proof Object or universal
implication decider. It includes origin/unknown-root identity, regions, Color,
access, all guarded effects and real target/delivery correspondence. Actual
exit validation uses the same mapping and selected actions; failure admits no
successful Post and never reopens. A summary formed from an unexecuted arm's
description does not license publishing that arm's history.

The [select_one witness](lifetime-algebra-walkthroughs.md#advance-summary-and-actual-exit-witness)
derives this relation for its two admitted ref-return arms without observing
the runtime choose value. The actual W1 program is the second consumer, not an
implementation of the first through available_guard. Recursive summary families
need their own founded structural/modular result (§9.1/§14), never a future-path
enumeration or evidence manufactured by executing recursion in advance.

## 11. Regular access material and capability boundaries

Actual Pattern/structure formation establishes lifecycle field companions
synchronously (§2.1.2). `$` remains only interpretation-polarity flip: an actual
formation/transform consumer has this duty, not every occurrence of `$`.
ADL and registered ordinary forwarding/Pattern extraction remain the only
field routes; Val2 presence alone grants no access edge or structural role.

Access/source structure retains regular form from formation.
Each actual name item retains its formed original paired regular descriptor
as the target of boundary-local SCC collapse; the type value alone cannot
supply this lifecycle value, and earlier cropping is not undone. Ordinary
overloaded `||`, field advance/appropriate composition and `~` can represent
choice, concatenation and repetition. This is not a new regular-expression
DSL. A finite state descriptor denotes regular walks paired with actual
name/generation/target parameters and terminal operation permissions. Sharing
a descriptor state never identifies those parameters or constructs a runtime
owned-object cycle. Runtime range endpoints may themselves be ordinary
lifecycle names; numeric range arithmetic is not silently part of regular
path equivalence.

`access(args)` is cropping: its operable endpoints/permissions are a subset of
the input's. Route prefixes needed to reach a retained leaf remain routes,
not permission to operate on the whole ancestor. Cropping destroys no ordinary
field and retracts no historical Color. It cannot reconstruct forgotten
structure or restore a previously removed permission. Concrete selector
packaging and normal forms remain engineering work.

| Capability | Admitted propagation | Excluded propagation |
|---|---|---|
| ref | Its actual exclusive strict tree of legal operations | No traversal through share or alias nodes |
| share | Shared observations; crossing a ref node continues only as share | No internal mutability, alias writes or strengthening |
| alias | Its explicitly registered root-write/default operations; initially no non-root normal access tree | No normal propagation through ref/share; no capability from type shape |

Lifetime alias here is the library alias-write capability, distinct from
[lexical Path aliases](../symbol-world/entity-alias-design.md). Normal return
from alias requires the trusted consume-to-value or bounded-window protocol
of §13. Distinct paths or anonymous origins alone never prove actual disjointness.

## 12. Dynamic credentials and batch acquisition

A trusted runtime check can establish a logical dependency for an ordinary
value, without changing its business type or making it a borrow. For example:

```lang
let safeid = (vector ref ..try_trust(num))?;
```

The [walkthrough](lifetime-algebra-walkthroughs.md#3-dynamic-safeid-and-batch-share-partitioning)
expands success/failure through ordinary registered Patterns. `?` remains
one-layer extraction, with no implicit success unwrap or error propagation.
The successful external action's unsafe Post binds the delivered id's actual
name/generation to resource origin, protected range/region and shared Color;
Pre limits its valid interval. Numeric equality transfers no credential.
Move, clone, arithmetic and view delivery need their own actual selected Post.
Runtime-unknown values may have established names. T * omega need not preserve
all element structural names or numeric relations in the ordinary type for
the lifecycle material to express these range dependencies.

The library's whole operation protocol must block or first end the credential
before every action invalidating its guarantee: length/capacity, layout,
generation, index or other relevant changes. Protecting a field called capacity
alone proves no completeness. With the protocol valid, range access consumes
the maintained credential without repeating the same runtime numeric check.
Without it access is unavailable/rejected, never an implicit dynamic-check
fallback or runner-up overload. Other Pre and machine costs remain. View
transport preserves actual dependencies and intervals, not a global credential.

A batch operation may consume shares with common provenance/current authority
in original ref R and acquire new exclusive **child refs from R** if their
actual operable ranges are disjoint in R's strict tree and all other Pre hold.
R need not be another explicit argument if its real authority is legally
retained. Shares are not strengthened/recolored into refs. It must preserve:

- actual source/target/generation and source-choice correlation;
- range disjointness, including ancestor-whole/descendant conflicts and route
  prefixes that need not be operable endpoints;
- other surviving shares, aliases, derived dependencies and logical credentials;
- legal end of old share intervals and start of exclusive regions without
  retracting their historical Color;
- single consumption and complete atomic publication at the existing action
  boundary, with no success permissions on failure.

No particular divergence depth or explicit path prefix length is required.
For each pair, sufficient evidence can establish that their **actual operation
ranges** lie in separated branches of the original strict ref tree and never
reconverge. Shared routing ancestors grant no whole-ancestor operation. In that
case disjointness follows even when the numerical depth of separation is
unknown. This is an actual tree/range fact, not an inference from unequal path
strings or reused regular descriptor states; an ancestor-whole range, later
meeting or unknown separation supplies no such evidence. Dynamic facilities
must establish the same relation, not replace it with address arithmetic.

Disjointness may be available statically or supplied by a small trusted dynamic
validator's successful Post. For one input no pairwise partition comparison
exists, so that additional runtime check is unnecessary; remaining Pre do not
disappear. Language semantics does not identify addresses with integers. The
materializer must faithfully and lightly realize the facility's actual abstract
range test; its representation/cost model belongs to machine work. See the
[unsafe owner](unsafe-semantic-admission.md#6-library-and-materializer-handoff).

## 13. Alias windows and partbox library protocols

An alias begins with only its own root alias-write capability; the non-root
normal access tree is empty. Assignment, extraction and fields require actual
registered defaults. It does not traverse normal ref/share access. A trusted
facility can either consume it to an ordinary value under the real ownership
and validity premises, or form a finite-lived normal ref inside a checked
window. Window Pre must already establish exclusion and object validity.

Each admitted window ref has a freshly formed anonymous origin, distinct from
other admitted formations and **not the alias as origin**. Normal ref rules
apply inside its valid region. Freshness is not a proof of physical non-aliasing:
the facility maintains real exclusion, referent validity and restrictions on
conflicting alias operations throughout the window. Escape checks reject
return/store beyond it. Unsafe Post cannot fix a failed window Pre.

Partbox is a possible safe library over these facilities. Its resource-right
fractions are ordinary accounting material, not `T share` borrow instances.
Here a **part** is the library value carrying resource identity and an amount;
conservation and consumption are library protocol laws. Numerical amount alone
confers no ref/share access. Amounts are ordinary compile/runtime values, not
required business-type parameters. Split consumes one part and conserves its
amount across parts of the same actual resource; merge consumes same-resource
parts and preserves total amount. Equal amounts
for distinct resources cannot merge. Ordinary copying cannot multiply rights.
Full amount can be consumed to box only with every other ordinary Pre satisfied,
including outstanding windows, Color and object validity. Non-full amount
provides at most its restricted alias, not normal ref access.

### 13.1 Legitimate initial issuance and source preservation

Conservation starts with a real authority source, not the equation q=1.
Initial formation of a legitimate full part for resource Q/current generation
is permitted only from one of these sources:

1. An actual complete **currently unallocated full-use authority** for that
   resource, such as an admitted owned resource/box. Successful issuance consumes
   or transfers that authority into Part(Q,1) at the ordinary commit boundary;
   it leaves no second independently usable full owner or issuance power through
   an alias or saved view. All normal access/validity/window/formation Pre apply.
2. An explicit trusted resource-acquisition action that really obtains that
   full-use authority. Its compatible successful unsafe Post may establish the
   otherwise underivable external ownership/validity facts and issue the part
   at its declared boundary, after ordinary Pre. A failure or a fabricated
   success value issues nothing. It cannot describe an already allocated right
   as another independent acquisition merely by changing its carrier name.

"Unallocated" describes the **current authority disposition**, not a ban on
ever using Q again. A later legitimate full-part-to-box conversion can return
a complete full-use authority, which may be consumed in a new issuance under
its current generation/Pre. Cache reuse, type/value equality or an old initial
reference cannot reset a spent authority or erase its allocation history.

The library's legitimate-part relation is ordinary contract/state material
over actual actions and capabilities, not a new builtin resource kind or a
user-created proof Object. Every usable part must have a source chain:

| Producer | Required real source and successful result |
|---|---|
| Initial issue | One of the full-use sources above, transferred once to the actual new complete part |
| split | An actual legitimate consumed part; complete outputs retain Q/generation and divide that input's amount/right under the protocol |
| merge | Actual legitimate independently consumable input portions, each consumed once; preserve Q/generation and their summed amount/right |
| Ordinary transport | The selected Move/borrow/clone contract and its real source; copies or multiple views are not independent allocations. Preserve Move still creates no second consumable portion |
| Full-part conversion | An actual legitimate full part consumed under all other Pre, producing the complete normal resource/box with its own selected Post |

Arithmetic conservation and matching resource fields are necessary but cannot
replace these source premises. In particular ordinary Part-shaped construction,
equal-value copying, ValidationOk, a resource identifier and the number 1 cannot
mint a legitimate part. A clone is usable only with its actual permitted producer
Post; there is no ordinary clone that duplicates allocatable rights.

Even if a user can construct an ordinary value with the same apparent fields,
classifier or Core content as (Q,1), into_box must reject its missing actual
use authority, or report an unavailable required observation. Type mismatch
need not be the reason: equality is insufficient even when the classifier
matches. It cannot issue first and validate later, use unsafe Pre, or choose
another consumer after failure. A failed issuance Pre leaves the source
unconsumed; failures after an earlier committed acquisition/transfer retain
that action's explicit ownership/cleanup contract, not synthetic rollback.

The [W4 issuance witness](lifetime-algebra-walkthroughs.md#legal-start-of-the-partbox-protocol)
connects this source to the existing split/merge program. Private interfaces,
accounting/locking representation and API names remain library choices; the
source, single-allocation and no-forgery laws above do not.

Option and a specific topology protocol may move parts into/out of actual
link slots while keeping topology mutation separate from payload use. Consuming
option extraction and complete replacement use existing atomic Object rules;
there is no partial parent or implicit part duplication. Actual collection
must recover all necessary rights with legitimate slot access before box/payload
access. This asserts neither that two neighbors suffice for arbitrary graphs
nor that conservation proves all safety, concurrency or reclamation properties.
The [fourth walkthrough](lifetime-algebra-walkthroughs.md#4-alias-partbox-and-option)
gives explicit transfer and failure boundaries. Container implementation,
accounting representation, locking and reclamation remain library work.

## 14. Proof obligations and engineering freedom

The closed responsibilities above do not assert completed metatheorems.
Before an adopted family supplies evidence, its implementation must establish:

| Obligation | Required scope and failure exit |
|---|---|
| Projection fidelity | Preserve selected actions, Stage-erased name/control/dependency/activation/delivery correspondence and no double projection; missing partner is Unavailable |
| Field symmetry | Actual formed registered operations commute with their paired lifecycle observations; no fresh ADL selection to repair a missing partner |
| Boundary soundness | Compression retains external observations, unknown roots, region/Color/path/source/effect correlations and rejects expiring-target escape |
| Advance-summary fidelity | Formed semantic body/selected descriptions yield a conditional summary without future execution; each legal actual Post implies its mapped summary, including every effect row |
| Nested composition | Inner-then-outer compression preserves outer observations for the specified family; no unconditional fact deletion or commute with arbitrary joins |
| Contract behavior | check/admit represent the exposed guarantees under the four-way mapping, limited update authority and compatibility direction; unknown implication cannot accept |
| Regular composition | For adopted finite descriptors with identity parameters, choice/advance/repetition stay in the defined regular domain and cropping is non-expansive |
| Name-item SCC collapse | At Ordinary closure boundaries and participating InPlace exits, collapse to the original paired regular structure with current constraints; missing partners/operations remain Unavailable |
| Recursive calculation | State the ordinary program's semantic domain, combination operations, closure and termination argument, or explicit Unavailable/rejection. A well-founded finite traversal or a proved finite family can qualify; arbitrary recursion has no default result |
| Trusted facilities | Validator success matches external reality; credentials, partitions, windows and accounting cover all invalidating operations and surviving conflicts |
| Rewrite/materialization | O revalidates affected projections; physical realization preserves promises, fixed cleanup and E's ABIKind |

For regular descriptors, any adopted decision algorithm must terminate on its
finite descriptor/parameter observations and justify the particular relation
it decides. Finite representation alone does not decide arbitrary numeric
range predicates or contract-program termination. A recursive contract family
may use a justified recurrence/fixed-point method only after defining its order,
domain and convergence/rejection condition; none is mandated universally here.
The walkthroughs give conditional derivations and counterexamples, not executed
proofs of these general obligations. Missing wiring is an unavailable diagnostic
at the requiring boundary; only actual unfinished K supplies residual material.

API spelling (including depends_on/live_with), argument order, result wrappers,
namespaces, ordinary function decomposition, Object/regular encodings,
normalization/caching, boundary algorithms, diagnostics and dynamic layouts may
evolve. They cannot change failure publication, origins/choice identity,
consumption counts, Color interval/history rules, cropping, actual exclusion,
no-reopen, no implicit reborrow, or the single E. Bootstrap order and evidence
tracking live in the [roadmap](../../planning/roadmap.md#9-lifetime-value-program-wiring).

## 15. Scoped specification migration

This design revises the baseline ac994bc semantics only at the following
boundaries. This table records migration decisions, not alternative current laws.

| Baseline restriction / incomplete handoff | Current rule and minimal distinguishing case | Affected consumers |
|---|---|---|
| §2.1.6 bounded a clause to an atomic conjunction | Complete ordinary compile expression. A local declaration followed by a branch/call can form the final self-root type | Clause E, associated formation, check/admit |
| One omitted-Post convention could be read across all callable forms | Body omission forms automatic Post; handwritten Function Pattern omission remains NoDelta. select_one cannot be stored under an insufficient NoDelta interface | Body boundary, FnExtract/contract compatibility |
| Finite NameView observations did not specify outward local elimination | Boundary traversal/compression, preserving unknown and explicit termination. Returning a borrow of a dying local still fails | Result/block/target exits |
| Excluding Color/origin from Object could be read as excluding lifecycle content | No new business-Object coordinate; lifecycle fields have ordinary content/identity. Equal integers with different credentials stay distinct names | Value representation, names and trusted Post |
| SCC handoff requested convergence without a computation domain | Per-name SCC collapse has fixed Ordinary/participating-InPlace boundaries and original regular companions; arbitrary program recursion and machine reuse remain separate | Companion formation, return boundaries, recursive/access consumers |

U_default, omitted interface Pre, interface NoDelta, Pre_F => Pre_A and
Post_A => Post_F, atomic complete identities, Pass/MoveEffect, self/Param,
cleanup and same-cut order, return barriers and E-selected delivery remain.
This revision supplies no independent lifecycle checker, SMT engine, source
runner, container implementation or physical ABI. Existing carrier tests are
evidence only for their implemented transaction/value slices.
