# Lifetime Policy, `@`, and the Overload Boundary

Status: canonical target semantics for the `@` operation and the lifetime/
ordinary-overload boundary. The `@` overload groups and the escape check defined
here are unimplemented target semantics; §6 registers the implementation debt.

This document is the canonical owner of `@`. The object model, the value/place
split, and the `ref` / `share` / `rebind` operations are owned by
[`../symbol-world/type-values-places-and-borrow-views.md`](../symbol-world/type-values-places-and-borrow-views.md);
`EffectiveOpen` is owned by
[`../symbol-world/symbol-first-meta-construction-and-pattern-injection.md`](../symbol-world/symbol-first-meta-construction-and-pattern-injection.md).

## 1. `@` is an ordinary place-sensitive operation

`@` is a normal overloaded operation. It is not a syntax-only marker, not a
reserved lexical territory without semantics, and not exempt from overload
resolution:

```text
E@ = ObservePlace_policy( CarrierPlace(E), Value(E) )
```

`CarrierPlace(E)` is the carrier slot through which `E` was read. This place
sensitivity is what distinguishes `@` from `ref` and `share`, which consume only
`Read(E)` — the complete object read out of the slot, never the slot itself. An
expression with no carrier place — a freshly computed temporary — supplies none,
so no `@` candidate applies to it.

`@` is **not** a general `PlaceOf(E)` defined on every expression. Its candidate
set is the two groups of §2 and nothing else; there is no third, generic
"address of this expression" meaning to fall back on.

`@` is not an ordinary meta/compile/seal/runtime policy atom, and lifetime
policy is not a fifth stage in that dimension. `@` is evaluated at a stage; it
does not name one.

## 2. The two overload groups of `@`

`@` has two positively defined overload groups, selected by whether the observed
object carries an internal `Val1` payload. Neither group is a general
"take a borrow" facility: for a value-bearing operand that job already belongs to
`ref`.

### 2.1 Value-bearing objects: lifetime facts

```text
Val1?(x) ≠ null
policy   = lifetime policy stage
--------------------------------------------
CarrierPlace(E) × Value(E)  ->  LifetimeFact
```

A `LifetimeFact` is the observation of the origin's region/provenance relation.
It is produced at the lifetime policy stage, which runs after ordinary overload
selection has already completed (§4). Spellings such as `val@`, `val@.region`,
and `val@.origin` project components of that fact.

This is the established meaning of `@` on a complete object shape
`⟨ Val1, P, Val2 ⟩`: **taking its lifetime**. Narrowing the borrow-producing
group to pure pattern slots (§2.2) does not disturb it. The two groups have
disjoint premises (`Val1?(x) ≠ null` versus `Val1?(x) = null`) and different
results, so neither competes with the other and neither is a fallback for the
other.

This group yields a fact, not a borrow. There is deliberately **no** compile-stage
borrow-producing candidate for a value-bearing operand:

```lang
s ref   // borrows Read(s), the complete symbol value — the ordinary way to borrow
s@      // lifetime observation of s; not a way to obtain a borrow
```

Removing a borrow meaning from `s@` therefore removes nothing from `@`: the
lifetime observation was always what `@` meant on a value-bearing operand.

### 2.2 Pure pattern slots: `P ref`

```text
Val1?(x) = null
policy   ∈ { compile, lifetime policy }
EffectiveOpen(x, current_context)
--------------------------------------------
CarrierPlace(E) × Value(E)  ->  P ref
```

Observing the carrier slot of a pure pattern value yields a reference to that
slot's pattern component. This group is available at compile stage as well,
because a compile-stage computation legitimately needs to reach a pattern slot it
was given.

This group is the whole reason `@` exists. Reading a `Val1? = null` name through
the ordinary value judgment preferentially produces the entity's `P x Val2`
pattern value, and `ref` then borrows *that*:

```lang
let t: type = uint8;

t ref   // uint8 ref — a correct borrow of the value that was read
t@      // type ref  — the carrier slot t itself
```

`t ref` is not an error to be repaired. A value-directed operation has no basis
for guessing that the writer meant the slot underneath, and nothing is inserted
to bridge the gap: `s ref` is never elaborated into `s |> type ref`, because an
operand position performs no implicit type conversion. `@` bridges it explicitly
by taking `CarrierPlace(E)` as input.

The selector is the `Val1` dimension of what was read, never type-rank. A
value-bearing operand needs no place observation to be borrowed: for `s : symbol`
the payload exists (`Val1(Symbol) = Member * ω`), so `s ref` is the ordinary
"form a borrow of this value" operation, and a type-rank object that carries a
payload behaves the same way. An explicit `symbol |> type` remains well-formed
whenever the operand really carries a `Val1` dimension — it is simply never
supplied implicitly. The value-side rules and the full classification are owned by
[`../symbol-world/type-values-places-and-borrow-views.md`](../symbol-world/type-values-places-and-borrow-views.md)
§5.1.1–§5.2.1.

The `EffectiveOpen` premise is a real premise, not a post-hoc permission check.
When the target is no longer effectively open at the observation context, this
group has **no applicable candidate**; the failure is "no matching overload for
`@`", not "`@` succeeded and then the result was rejected".

Consequently:

```text
GlobalLifetime(x) does not imply EffectiveOpen(x)
```

A pattern value that is reachable for the whole program is still not observable
as `P ref` outside its open capability region.

### 2.3 Idempotence

`@` participates in the general borrow-view overlap rule:

```text
Borrow_k( Borrow_j(q) )  =  Coerce_{j->k}( Borrow_j(q) )
Target( Coerce_{j->k}(v) )  =  Target(v)
```

So `@@` is not a missing operation. It is admitted, it preserves the target, and
it builds no second layer — which is what "idempotent" means here. Retargeting is
available only through `rebind`. Of the overlapping compositions only `share ref`
has no candidate, because capability may be surrendered and never regained. The
full table is owned by
[`../symbol-world/type-values-places-and-borrow-views.md`](../symbol-world/type-values-places-and-borrow-views.md)
§5.3.

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

### 3.1 `ValidRegion` is indexed by the view's capability

`ValidRegion` is **not** uniformly the target's open capability region. `Open` is
the *extension* capability region, not a single observation lifetime shared by
every view of the target:

```text
ValidRegion( type ref )   =  OpenRegion( Target )
ValidRegion( type share ) =  LifetimeRegion( Target )

OpenRegion( Target )  ⊆  LifetimeRegion( Target )
```

So the two views are checked against different regions, and the difference is
exactly the capability each one carries:

| view | carries | may leave the `Open` window | may leave the target's lifetime |
| --- | --- | --- | --- |
| `type ref` | write + `inject` + `OpenWitness` | no | no |
| `type share` | read/observe only | yes | no |

A `type ref` cannot leave the window because its own formation condition is the
window. A `type share` may, precisely because it surrendered the extension
capability that made the window relevant; it still may not outlive the target
itself.

For a `type ref`, then, the escape check and the `EffectiveOpen` premise of §2.2
are the same condition applied at two moments: at production and at every
destination.

### 3.2 A well-formed `type ref` is its own witness

Because the premise holds at production and the view cannot be held past the
window:

```text
Γ ⊢ r : type ref   =>   Open_Γ( Target(r) )

holdable interval of a type ref  =  the Open window
```

A consumer that already holds such a view therefore does not re-ask for openness;
see `inject` input validity in
[`../symbol-world/symbol-first-meta-construction-and-pattern-injection.md`](../symbol-world/symbol-first-meta-construction-and-pattern-injection.md)
§8.2.3. The obligation this places on this section is the converse direction: the
escape check must reject exactly the destinations that would carry the view past
the closing boundary.

This is what makes returning a `type ref` a question with an answer rather than a
blanket prohibition. A return inside the same open window is well-formed; a
return that crosses the boundary is rejected here, because the receiving context
cannot derive `out : type ref` at all — not later, as a failed `inject`. The
author's option is to weaken before the boundary:

```lang
r share
```

That weakening is the admitted `ref share` composition, not a missing overload
(§2.3). `share` is the view that is allowed to leave an open capability region,
and it pays for that with a strictly smaller capability set: it is not an
assignment left side and not an `inject` target. It does **not** buy escape from
`LifetimeRegion(Target)`.

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
2. ordinary selection of `@` itself        -> one candidate, from the groups visible
                                              in the operand's policy stage (§2)
3. lifetime validation                     -> accept or reject steps 1 and 2
```

Step 2 uses the ordinary selector, not a lifetime-specific one; the policy stage
only bounds which candidate groups are visible. Step 3 never reselects steps 1 or
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
the `@` operation and both overload groups of §2
LifetimeFact and its region/origin projections
the escape check of §3 at all four destination classes
the lifetime policy stage as an evaluation stage
CheckableCaptureForm construction at closure materialization
```

Still genuinely open engineering questions, not closed by this document:

- the concrete representation and granularity of a region (lexical block,
  construction anchor, or a finer relation);
- whether `LifetimeFact` is a first-class value that user code may store, or an
  observation consumed only by checking;
- diagnostics and caching identity for lifetime validation;
- borrow/move/copy defaults, closure ABI, and environment layout, which remain
  the mechanical-lowering design's territory.

This revision still defines none of the following: lifetime overloads as a
second selection step, lifetime specificity ordering, multiple-callable handoff
objects, ABI equivalence classes used for selection, refinement ordering or a
refinement phase, or Horae semantics.

Related canonical contracts:

- [`../patterns-overload/overload-resolution-design.md`](../patterns-overload/overload-resolution-design.md)
- [`../symbol-world/symbol-policy-and-compile-flow-projection.md`](../symbol-world/symbol-policy-and-compile-flow-projection.md)
- [`../symbol-world/type-values-places-and-borrow-views.md`](../symbol-world/type-values-places-and-borrow-views.md)
