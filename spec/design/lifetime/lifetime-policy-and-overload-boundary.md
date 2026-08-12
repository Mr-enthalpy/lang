# Lifetime Policy, `@`, and the Overload Boundary

Status: canonical target semantics for the `@` operation and the lifetime/
ordinary-overload boundary. The `@` overload groups and the escape check defined
here are unimplemented target semantics; §6 registers the implementation debt.

This document is the canonical owner of `@`. The object model, the value/place
split, and the `ref` / `share` / `rebind` operations are owned by
[`../symbol-world/type-values-places-and-borrow-views.md`](../symbol-world/type-values-places-and-borrow-views.md);
construction-lineage `Open_Γ(value)` is owned by
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
so no *place-observing* `@` candidate applies to it. The borrow-type-value fixed
points of §2.3 do not consult `CarrierPlace`; they stabilize the classifier
universe and are not an existing-borrow-value overlap or retargeting rule.

`@` is **not** a general `PlaceOf(E)` defined on every expression. Its candidate
set is nevertheless closed: the two instance groups of §2, dispatched by the
`Val1` dimension, plus the borrow-type-value fixed point of §2.3. There is no
third, generic
"address of this expression" meaning to fall back on.

`@` is not an ordinary meta/compile/seal/runtime policy atom, and lifetime
policy is not a fifth stage in that dimension. `@` is evaluated at a stage; it
does not name one.

## 2. The two base overload groups of `@`

`@` has two positively defined **instance** overload groups, selected by whether
the observed object carries an internal `Val1` payload. A borrow **type value**
is matched first by the universe fixed point of §2.3; an ordinary value instance
whose Pattern happens to be a borrow type still follows the `Val1` dispatch.
Neither instance group is a general
"take a borrow" facility: for a value-bearing operand that job already belongs to
`ref`. The whole dispatch is:

```text
E is a borrow type value       ->  §2.3
else Val1?( Value(E) ) ≠ null  ->  §2.1
else Val1?( Value(E) ) = null  ->  §2.2
```

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
Val1?( Value(E) ) = null
CarrierPlace(E) = q
policy ∈ { compile, lifetime policy }
CanBorrowRef_Γ(q)
--------------------------------------------
E@ : P ref
Target(E@) = q
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
t@      // type ref — the pure type carrier slot
```

`t ref` is not an error to be repaired. A value-directed operation has no basis
for guessing that the writer meant the slot underneath, and nothing is inserted
to bridge the gap: `s ref` is never elaborated into `s |> type ref`, because an
operand position performs no implicit type conversion. `@` bridges it explicitly
by taking `CarrierPlace(E)` as input.

The selector is the `Val1` dimension of what was read, never type-rank. A
value-bearing operand needs no place observation to be borrowed: for `s : symbol`
the payload exists (`Val1(Symbol) = Σ = ⟨T?, V⟩`), so `s ref` is the ordinary
"form a borrow of this value" operation, and a type-rank object that carries a
payload behaves the same way. An explicit `E |> type` is well-formed exactly
when `E` exposes one unambiguous type facet (`|TypeMembers(E)| = 1`), never
merely because it carries a `Val1` dimension — the `Val1` payload is present even
for a Symbol with no type member, and it selects only `ref`-versus-`@`, never
type-projection applicability. The projection is also never supplied implicitly.
The value-side rules and the full classification are owned by
[`../symbol-world/type-values-places-and-borrow-views.md`](../symbol-world/type-values-places-and-borrow-views.md)
§5.1.1–§5.2.1.

`Val1?(Value(E))` selects the overload group; it does not decide construction
openness. Because the result refers to `q = CarrierPlace(E)`, ordinary borrow
formation checks `q`'s addressability, policy, lifetime, and requested
capability. It does **not** ask whether the current contents are Open. For
`let t: type = uint8`, a frozen but live and borrowable `CarrierPlace(t)` may
therefore still yield `type ref`.

Consequently the two judgments remain independent:

```text
Γ ⊢ t@ : type ref  does not imply Open_Γ(Read(t@))
Open_Γ(Value(q))   does not imply CanBorrowRef_Γ(q)
```

For a Symbol `S`, the distinct ordinary field path is `(S ref).type : type ref`;
`AsType(S)` remains by-value and is never followed by `@` to recover a place.

### 2.3 Borrow type values are fixed points; borrow instances are not

Universe overlap is selected only when the operand denotes the borrow **type
value** itself:

```text
type ref@    = type ref
type share@  = type share

rank(type ref)               = rank(type)
rank(type share)             = rank(type)
rank(type ref/share rebind)  = rank(type)
```

These equations prevent a borrow classifier from manufacturing an ever-higher
classifier. They are not an identity overload for every value whose Pattern is
a borrow type.

Once an expression evaluates to a borrow **value instance**, that value carries
`Val1` and §2.1 applies:

```lang
let t: type ref = ...;
t@;                         // LifetimeFact(t)
```

The lifetime observed is the lifetime of the instance `t`, not a second borrow
of its referent and not the borrow type value. The former blanket rule
`E : BorrowView => E@ = E` is retired.

The independent borrow-constructor overlap remains:

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

None of these constructor equations changes the value-instance meaning of `@`.
The full table is owned by
[`../symbol-world/type-values-places-and-borrow-views.md`](../symbol-world/type-values-places-and-borrow-views.md)
§5.3.

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

Both views may remain valid after the current pointee freezes. `type ref` may
then replace the pointee wholesale if `Writable(Target)` holds, but neither view
can make the frozen value admissible as `extend`'s old value.

### 3.2 Borrow validity never discharges construction openness

The positive separation is:

```text
Γ ⊢ r : type ref  does not imply Open_Γ(Read(r))
Open_Γ(v)          does not imply that v has a writable carrier
```

A consumer that performs `extend` must query `Open_Γ(old_value)` even when the
value was read through `type ref`. The place-level `inject` wrapper in
[`../symbol-world/symbol-first-meta-construction-and-pattern-injection.md`](../symbol-world/symbol-first-meta-construction-and-pattern-injection.md)
§8 performs two checks independently:

```text
Open_Γ(Read(r))
Writable_Γ(Target(r))
```

Returning or storing `type ref` asks only the ordinary escape question of this
section. A later `inject` may fail because the then-current value is frozen even
though the reference remains lifetime-valid. Weakening to `share` surrenders
write capability, but does not alter construction lineage and never extends the
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
the `@` operation, both instance groups, and borrow-type fixed point of §2
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
