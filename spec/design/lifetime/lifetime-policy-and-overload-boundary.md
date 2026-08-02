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
E@ = ObservePlace_policy( Origin(E), Value(E) )
```

`Origin(E)` is the place coordinate through which `E` was read. This place
sensitivity is what distinguishes `@` from `ref` and `share`, which consume only
`Value(E)`. An expression with no place origin — a freshly computed temporary —
supplies no `Origin`, so no `@` candidate applies to it.

`@` is not an ordinary meta/compile/seal/runtime policy atom, and lifetime
policy is not a fifth stage in that dimension. `@` is evaluated at a stage; it
does not name one.

## 2. The two overload groups of `@`

`@` has two positively defined overload groups, selected by whether the observed
object carries an internal `Val1` payload.

### 2.1 Value-bearing objects: lifetime facts

```text
Val1?(x) ≠ null
policy   = lifetime policy stage
------------------------------------------
Origin(E) × Value(E)  ->  LifetimeFact
```

A `LifetimeFact` is the observation of the origin's region/provenance relation.
It is produced at the lifetime policy stage, which runs after ordinary overload
selection has already completed (§4). Spellings such as `val@`, `val@.region`,
and `val@.origin` project components of that fact.

### 2.2 Pattern-value slots: `P ref`

```text
Val1?(x) = null
policy   ∈ { compile, lifetime policy }
EffectiveOpen(x, current_context)
------------------------------------------
Origin(E) × Value(E)  ->  P ref
```

Observing the place of a pattern-value slot yields a reference to that slot's
pattern component. This group is available at compile stage as well, because a
compile-stage computation legitimately needs to observe a pattern slot it was
given.

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

`@` participates in the general borrow-view fixed point:

```text
Borrow(Borrow(q)) = Borrow(q)
```

so `@@` has no applicable candidate, in the same way as `ref ref` and
`share ref`.

## 3. Escape checking

A borrow view obtained from a place must not outlive the observation region of
that place. The positive obligation is:

```text
Escapes(view, destination)
  = Region(destination) ⊄ ObservationRegion(Origin(view))

Escapes(view, destination)  ->  the storing/returning operation is rejected
```

The destinations subject to this check are the ones that can outlive the origin:
storing into a longer-lived place, returning from a callable, capturing into a
materialized callable entity, and installing into global namespace material.

For pattern-value observations the region is the target's open capability
region, so the escape check and the `EffectiveOpen` premise of §2.2 are the same
condition applied at two moments: at production and at every destination.

`share` is the view that is allowed to leave an open capability region, and it
pays for that with a strictly smaller capability set: it is not an assignment
left side and not an `inject` target.

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
