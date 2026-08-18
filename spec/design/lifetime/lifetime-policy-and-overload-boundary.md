# Lifetime Policy, `@`, and the Overload Boundary

Status: canonical target semantics for the `@` operation and the lifetime/
ordinary-overload boundary. The `@` place-observation and the escape check
defined here are unimplemented target semantics; §6 registers the implementation
debt. The full lifetime algebra of `@` (region representation, `LifetimeVal`
shape, ordering) is deliberately left unfrozen.

This document is the canonical owner of `@`. The object model, the value/place
split, and the `ref` / `share` / `rebind` operations are owned by
[`../symbol-world/type-values-places-and-borrow-views.md`](../symbol-world/type-values-places-and-borrow-views.md);
construction authority (`OpenHere_Σ(value)`) is owned by
[`../symbol-world/symbol-first-meta-construction-and-pattern-injection.md`](../symbol-world/symbol-first-meta-construction-and-pattern-injection.md).

## 1. `@` is a privileged place-observation operation

`@` is a normal overloaded operation. It is not a syntax-only marker, not a
reserved lexical territory without semantics, and not exempt from overload
resolution:

```text
Apply(@, E):
  p := privileged-place-of-actual(E)
  return LifetimeVal(p)
```

`@` still applies to an expression syntactically, but obtaining the place is an
implicit privilege of the builtin application itself: there is no user-callable
`PlaceOf(E)`, and an ordinary user function cannot obtain the same place merely
by writing a matching parameter Pattern. The produced lifetime value depends on
the abstract place `p`; this is not a Rust-style "borrow a generic lifetime
parameter" operation.

`@` is **not** a borrow constructor. It yields a lifetime value, never a borrow
view. Borrow formation belongs to `ref` and `share`, which may be privileged
actual-place builtins (`PrivilegedActualPlace(ref-family)`,
`PrivilegedActualPlace(share-family)`) when the overload needs the actual's
place (§2). An expression with no abstract place — a freshly computed temporary —
supplies none: `@` selection still runs, and only after the unique `@` builtin
is selected does place acquisition run; absence of a place is a post-selection
invocation precondition failure (`InvocationFailure(NoCarrierPlace(actual))`),
not candidate removal or overload reopening.

`@` is not an ordinary meta/compile/seal/runtime policy atom, and lifetime
policy is not a fifth stage in that dimension. `@` is evaluated at a stage; it
does not name one.

## 2. Privileged place acquisition: `@`, `ref`, and `share`

`@`, `ref`, and `share` are builtin applications that may, for some overloads,
be granted privileged access to the place of their actual argument:

```text
PrivilegedActualPlace(@-family)
PrivilegedActualPlace(ref-family)
PrivilegedActualPlace(share-family)
```

This does not make them one callable family. Each operator keeps its own
family identity and overload set: `@` remains a normal overloaded operator,
and `ref` / `share` each have a type-forming overload (producing `type ref` /
`type share` as type values) and a borrow-forming overload (producing `t ref` /
`t share` as borrow instances). The permission attaches to each operator's
builtin family identity, not to ordinary function parameters. An ordinary
user function that spells the same formal head still cannot obtain the
actual's place.

The three operators produce different results:

```text
@      -> LifetimeVal(p)
ref    -> borrow instance | TypeValue (type formation)
share  -> borrow instance | TypeValue (type formation)
```

Phase order (same for `@`, `ref`, `share`):

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
§5.1). For `let t: type = uint8`, `t ref` is the **type-forming** overload: it
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

The `ref`/`share` family is not a single meta stage: the **type-forming**
member is a **meta** member (`T : U_n ⊢ T |> ref = RefTy(T)`, producing the
borrow TypeValue), while the **borrow-forming** member inside the formed
borrow type's callspace is a **runtime || compile** builtin/default member and
is the only family member that may obtain `PrivilegedActualPlace`. The
declarations below are the type-forming members; the borrow-forming members
live inside each formed borrow type's callspace (canonical owner
`../symbol-world/type-values-places-and-borrow-views.md` §5.1).

```lang
let ref =
    <n>(self, t: n type):
    meta
    => default;

let share =
    <n>(self, t: n type):
    meta
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
`U_n |> ref/share` well-formed at every level `n ≥ 1` — is frozen, not
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
    runtime||compile -> _: t ref
    => default;
```

The head's `object : t` displays the candidate head, Pattern, and policy so the
member participates in ordinary overload resolution. It must not be read as
`Read(actual) -> move/copy -> fresh ordinary parameter object`: at application
the builtin member may obtain the corresponding place from the actual. This is
the explicit formalization of the privileged actual-place semantics that the
generated/builtin `ref` / `share` callables already had, not a new arbitrary
place-reflection facility.

#### 2.0.1 `symbol` ref / share: no global forwarding bridge

A `symbol`-valued operand is an ordinary value with a `Val1` payload, so the
ordinary borrow-forming `ref` / `share` defaults apply directly: `s ref` is
`symbol ref` (a borrow of the symbol value; `Target(s ref) =
PrivilegedActualPlace(s)`). There is **no global** `symbol` ref/share
forwarding bridge to type formation, and no implicit `AsType` during matching
(§4 NoImplicitBorrowFormation,
[`../symbol-world/type-values-places-and-borrow-views.md`](../symbol-world/type-values-places-and-borrow-views.md)
§5.1.2): `symbol =/=> symbol |> type` during matching. The language default is:

```text
s : symbol   ->  s ref : symbol ref      (ordinary borrow of the symbol value)
t : type     ->  t ref  : type formation (TypeValue t ref; the type-forming
                                           overload, reached directly because
                                           `type : type_1`)
```

A user may still author a local `ref` Symbol whose overload matches `symbol`
and whose body performs `AsType` inside the selected candidate:

```lang
let ref =
    (self, s: symbol):
    compile => {
        (s |> type) ref
    };
```

That is local Symbol algebra, not a language default and not part of the
global builtin algebra. No implicit `AsType` is attempted at matching time;
inside the body, `s |> type` is an explicit `AsType` in a type-expected
position (§5.6 of type-values), which is permitted; the resulting `tau_S` then
enters the ordinary type-forming `ref` / `share` overload. This is explicit
user-authored forwarding, not candidate adaptation and not a reopening of the
overload boundary.

If `TypeSlot(S) = None` (the symbol carries no type value), that is **not** an
applicability failure and not a reason for the resolver to revisit other
candidates. Applicability of a user-authored bridge candidate is decided
entirely by the ordinary `s : symbol` formal match and policy admissibility,
before body evaluation:

```text
CandidateApplicable(bridge, S)      // s : symbol formal + ordinary admissibility
    -> unique bridge selected
    -> evaluate body
    -> AsType(S)
    -> may fail here
```

The failure of `AsType(S)` inside the already-selected candidate's body is a
selected-invocation / body / result-transformation failure, not an implicit
conversion, and it does not reopen the candidate space: the overload phase
stays closed exactly as for any other uniquely selected candidate whose body
fails.

### 2.1 `@` yields a lifetime value, uniformly

```text
E@ : LifetimeVal(p)    where p = privileged-place-of-actual(E)
```

The retired forms do not return:

```text
Val1?(x) ≠ null  ->  LifetimeFact        (retired as a separate @ group)
Val1?(x) = null  ->  P ref               (retired: @ is not a borrow constructor)
t@ : type ref                            (retired)
type ref@ = type ref                     (retired)
type share@ = type share                 (retired)
```

`@` is never a bridge from a hidden carrier slot to a `type ref`. A type-valued
binding evaluates to the complete closure `tau`; an ordinary/namespace consumer observes the projection `Core(tau)=Q`. Reaching the type-level
place is done explicitly with `t |> (type ref)` (or `(S ref).type` for a Symbol),
never by `@`. `AsType(S) = S |> type` remains by-value and is never followed by
`@` to recover a place.

The complete lifetime algebra of `@` is deliberately **not frozen** here: the
concrete representation and granularity of a region, the shape of `LifetimeVal`
and its region/origin projections, and whether it is a first-class storable
value are open questions (§6). This section freezes only the responsibility
boundary — privileged place observation producing a lifetime value — and the
fact that `@` is not a borrow constructor.

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

Both views may remain valid after the current pointee's open window closes. `type ref` may
then replace the pointee wholesale if `Writable(Target)` holds, but neither view
can make the closed-window value admissible as `extend`'s old value.

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

A consumer that performs `extend` must query `OpenHere_Σ(old_value)` even when the
value was read through `type ref`. The place-level `inject` wrapper in
[`../symbol-world/symbol-first-meta-construction-and-pattern-injection.md`](../symbol-world/symbol-first-meta-construction-and-pattern-injection.md)
§8 performs two checks independently:

```text
OpenHere_Σ(Read(r))
Writable_Γ(Target(r))
```

Returning or storing `type ref` asks only the ordinary escape question of this
section. A later `inject` may fail because the then-current value's open
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
resolved by ordinary overload resolution like any other operation — §2 defines
its candidate set — so "no lifetime overload stage" and "`@` has overloads" are
consistent statements about two different things.

The apparent circularity dissolves once the three steps are separated. They are
strictly ordered, and each one is finished before the next begins:

```text
1. ordinary selection inside the operand   -> the operand value and its carrier place
2. ordinary selection of `@` itself        -> one candidate: the privileged
                                              place-observation visible in the
                                              operand's policy stage (§2)
3. lifetime validation                     -> accept or reject steps 1 and 2
```

Step 2 uses the ordinary selector, not a lifetime-specific one; the policy stage
only bounds which candidate is visible. Step 3 never reselects steps 1 or
2 — that is the whole content of this section. So "the lifetime stage runs after
ordinary selection has completed" and "`@` is itself resolved by ordinary
selection" describe steps 3 and 2 respectively, and neither feeds back into the
other.

## 5. Closure capture handoff

The positive obligation established for closure capture is:

```text
ResolvedCaptureRequirement
  -> CheckableCaptureForm {
       source_place,
       requested_access_view,
       origin_or_region_relation,
       storage_or_link_category
     }
  -> LifetimeValidation (§3 escape check)
  -> RepresentationSelection
```

No capture may enter a materialized callable entity through an implicit,
uncheckable representation side channel. Every capture presents a checkable form
whose `origin_or_region_relation` is exactly the input the §3 escape check
consumes.

## 6. Registered implementation debt

Semantics closed here, not yet built:

```text
the privileged `@` place-observation (`PrivilegedActualPlace(@-family)`) and
`LifetimeVal`
the escape check of §3 at all four destination classes
the lifetime policy stage as an evaluation stage
CheckableCaptureForm construction at closure materialization
```

Still genuinely open engineering questions, not closed by this document:

- the full `@` lifetime algebra: concrete representation and granularity of a
  region (lexical block, construction anchor, or a finer relation), the shape of
  `LifetimeVal`, and whether it is a first-class value that user code may store
  or an observation consumed only by checking — deliberately not frozen (§2.1);
- diagnostics and caching identity for lifetime validation;
- borrow/move/copy defaults, closure ABI, and environment layout, which remain
  the mechanical-lowering design's territory.

This revision still defines none of the following: lifetime overloads as a
second selection step, lifetime specificity ordering, multiple-callable handoff
objects, ABI equivalence classes used for selection, refinement ordering or a
refinement phase, or Horae semantics. The retired `@` forms — the two instance
groups (`Val1?(x) ≠ null -> LifetimeFact`, `Val1?(x) = null -> P ref`),
`t@ : type ref`, and the borrow-type fixed points `type ref@ = type ref` /
`type share@ = type share` — do not return.

Related canonical contracts:

- [`../patterns-overload/overload-resolution-design.md`](../patterns-overload/overload-resolution-design.md)
- [`../symbol-world/symbol-policy-and-compile-flow-projection.md`](../symbol-world/symbol-policy-and-compile-flow-projection.md)
- [`../symbol-world/type-values-places-and-borrow-views.md`](../symbol-world/type-values-places-and-borrow-views.md)
