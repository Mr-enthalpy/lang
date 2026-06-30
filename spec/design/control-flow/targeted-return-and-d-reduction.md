# Targeted Return and D-Reduction

**Status: Design only. Not implemented current behavior.**

This document describes future semantic lowering for targeted return
syntax and D-reduction. None of the semantics described here are
implemented in the current parser, normalizer, or build stages.

The current implementation (v0.9, PR86) provides only the structural
syntax and normalized AST: `ReturnEvent`, `TailValue`, and unresolved
return target syntax. Target resolution, D-reduction, and execution
semantics are deferred.

## 1. Targeted Return Core Idea

Future semantic lowering for the three return terminal forms:

```text
E return;
  => E |> (Self₀ return)

E |> (T return);
  => targeted return to resolved T

E (T return);
  => targeted return to resolved T
```

where `Self₀` is the current enclosing function-object self,
obtained from the active return-target context at the point
where the return event is lowered.

The implicit return spelling `E return;` is sugar for targeted
return to the nearest enclosing self.

## 2. Intrinsic Return Action

Future semantic core: `E |> (Self return)` is a built-in /
intrinsic control meta-action, not ordinary overload lookup.

It is **not** dispatched through the call-entry mechanism
described in `spec/design/symbol-world/function-object-call-model.md`.
It is a direct control-flow action implemented by the meta-function
evaluation layer.

## 3. Future Return Completion

Targeted return produces a `Done_Return` completion:

```text
Done_Return(Self, pattern(E), value(E))
```

where:

- `Self` identifies the enclosing function-object receiving the return.
- `pattern(E)` is the structural pattern of the returned value.
- `value(E)` is the evaluated return value.

`Done_Return` is a semantic IR concept. It is **not** represented
in the current normalized AST. The current `NormReturnEvent` is a
surface-structure node, not a semantic completion.

## 4. Local Unit Contribution

At the local (intra-block) level, a `ReturnEvent` contributes unit
to the local pattern space so that local pattern reasoning can
continue:

```text
Local pattern space: A - S + Done(unit)
Return accumulator:  ReturnAccumulator + Done(D)
```

`Done(unit)` is absorbed as the zero element during local pattern
combination. This allows the enclosing context to continue
processing remaining pattern material while the return completion
propagates to the target boundary.

This behavior is **not** implemented in the current build evaluator.

## 5. D-Reduction Boundary

At the matching control-flow / binding / extraction boundary,
the targeted return completion injects the returned pattern into
the target result slot.

```text
At boundary matching Selfᵢ:
  Done(D) is consumed from the return accumulator
  D is injected into the matched result slot
```

D-reduction is a future semantic concept. It is not implemented
in the current parser, normalizer, or build evaluator.

## 6. Non-Local Target Propagation

A return targeted at `Selfᵢ` propagates through intermediate
boundaries until `Selfᵢ` is reached:

```text
Each intermediate boundary:
  - passes Done(D) upward (return accumulator propagation)
  - contributes Done(unit) locally (local pattern completeness)

When Selfᵢ is reached:
  - D-reduction occurs
  - result is injected into the matched slot
```

If no matching active target exists at any reachable boundary,
a semantic diagnostic is emitted.

This behavior is **not** implemented. No current code
checks target validity, propagates completions, or performs
D-reduction.

## 7. Relationship to Current Implementation

| Concept | Current (v0.9) | Future (design) |
|---|---|---|
| Return terminal forms | Parsed, normalized as `ReturnEvent` | Same |
| Target syntax | Preserved unresolved (`Explicit(NormExpr)`) | Resolved to active self |
| Implicit return | Normalized as `ReturnEvent(ImplicitNearest)` | Lowered to `Explicit(Self₀)` |
| `Done_Return` | Not represented | Semantic IR concept |
| D-reduction | Not implemented | Future boundary action |
| `Done(unit)` contribution | Not implemented | Local pattern completeness |
| Target propagation | Not implemented | Future traversal |
| Target validity check | Not implemented | Future semantic diagnostic |
