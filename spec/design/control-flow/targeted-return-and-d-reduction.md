# Targeted Return and D-Reduction

**Status: Partially implemented.**

This document describes future semantic lowering for targeted return
syntax and D-reduction. The current implementation deliberately stops
at return target binding.

The current implementation provides the structural syntax and normalized
AST (`ReturnEvent`, `TailValue`, and unresolved return target syntax) plus
a minimal semantic return-target binding pass. D-reduction, completion
propagation, and execution/lowering semantics remain deferred.

This document owns targeted-return completion and its D-reduction boundary.
Automatic require does not define a second return/control algebra: it consumes
the D/Done-normalized compile flow defined here and in
`../patterns-overload/static-pattern-spaces-and-extraction-chains.md`. The
compile projection and require slicing rules are canonical in
`../symbol-world/symbol-policy-and-compile-flow-projection.md`.

The current return-target binding substrate adds one semantic pass after
normalization:

```text
Raw AST
  -> Normalized AST
  -> ReturnTargetBinding
  -> later result/completion semantics
```

This pass resolves a normalized return event to an active
`ReturnTargetFrame` when possible. It does not type-check the return value,
assign a return slot, propagate non-local control flow, insert drops, check
lifetime postconditions, lower to HIR/ABI, or create `Done_Return`.

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

The implicit return spelling `E return;` binds to the nearest active
return frame in the current substrate. Future completion semantics may
then treat that frame as the enclosing self capability.

## 2. Return Capability Completion

Future return completion is mediated by the function object's return
capability. That capability is exposed through the function object's
type-associated namespace as an ordinary callable capability value under the
anonymous type of `self`, as described in
`spec/design/symbol-world/function-object-self-and-return-capability.md`.

Return is therefore not a parser keyword escape hatch, not an operator, and
not a compiler-intrinsic control action. The target-binding pass in this PR
does not execute or invoke that return capability. It only identifies the
active frame that future completion semantics will use to reach the
appropriate function-object return capability.

The current target-binding pass records a `BoundReturnEvent` containing the
return value expression, the unresolved target form, the resolved frame id,
and provenance.

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

The current implementation checks only whether the requested target is
active in the current `ReturnTargetStack`. It does not propagate
completions or perform D-reduction.

## 7. Relationship to Current Implementation

| Concept | Current (v0.9) | Future (design) |
|---|---|---|
| Return terminal forms | Parsed, normalized as `ReturnEvent` | Same |
| Target syntax | Preserved unresolved, then bound by `ReturnTargetBinding` | Resolved to full function-object self capability |
| Implicit return | Binds to nearest active `ReturnTargetFrame` | Lowered/completed through enclosing self capability |
| Explicit self target | Attempts active self-frame match; does not silently fall back to nearest | Full self capability object |
| Nested unmaterialized closure return | Preserved as unbound nested closure material | Bound when the closure is materialized/elaborated as its own body |
| `Done_Return` | Not represented | Semantic IR concept |
| D-reduction | Not implemented | Future boundary action |
| `Done(unit)` contribution | Not implemented | Local pattern completeness |
| Target propagation | Not implemented | Future traversal |
| Target validity check | Minimal active-frame diagnostics | Full target reachability diagnostics |

## 8. Current Return Target Binding Substrate

The implemented substrate defines these semantic objects:

```text
ReturnTargetFrame
ReturnTargetStack
UnboundReturnEvent
BoundReturnEvent
ResolvedReturnTarget
```

Entering a body that the pass is explicitly elaborating pushes a
`ReturnTargetFrame`; leaving that body pops it. A nested closure literal is
preserved as value material unless that closure is explicitly elaborated as
its own returnable body. Therefore a return inside an unmaterialized nested
closure is not bound to the outer frame.

The current `ReturnSelfIdentity` is a placeholder derived from normalized
binder spelling because full lexical self-slot / function-object identity is
not wired into this substrate yet. Future explicit self-target resolution
must use lexical slot identity, not text equality on the spelling `self`.

The build-layer source callable hook currently runs this pass as validation.
It rejects malformed return targets but does not store bound events in
`SourceCallableObject`; later body evaluators may re-run target binding when
they need the bound return-event stream for completion/result semantics.

Diagnostics are structured:

```text
ReturnOutsideReturnableContext
ReturnTargetNotActive
AmbiguousReturnTarget
UnsupportedReturnTargetForm
```

Each diagnostic carries provenance. These diagnostics are target-binding
diagnostics only; they do not imply return value type failure, lifetime
failure, or lowering failure.
