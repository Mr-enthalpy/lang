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
Automatic require does not define a second return/control algebra. Match
structure in `CompleteSymbolFlow` already contains D residual and Done
completion constructors defined here and in
`../patterns-overload/static-pattern-spaces-and-extraction-chains.md`; compile
projection preserves them homomorphically. Projection and require slicing are
canonical in
`../symbol-world/symbol-policy-and-compile-flow-projection.md`.

The return-target binding substrate adds one consumer after
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

where `Self₀` is the current enclosing callable-frame self,
obtained from the active return-target context at the point
where the return event is lowered.

The implicit return spelling `E return;` selects the outermost enclosing
function layer. The current active-frame binder still selects its most recent
frame; alignment to this rule is consumer work, not an alternate semantics.

## 2. Return Capability Completion

Future return completion is mediated by the callable frame's return
capability. That capability is exposed through the callable-local `Self` space
as an ordinary callable capability value, as described in
`spec/design/symbol-world/function-object-self-and-return-capability.md`.

Return is not a parser keyword escape hatch, an operator, or a
compiler-intrinsic control action. The target-binding pass identifies the
active frame; return-capability execution belongs to the completion consumer.

The current target-binding pass records a `BoundReturnEvent` containing the
return value expression, the unresolved target form, the resolved frame id,
and provenance.

The selected frame also retains its complete normalized return binding slot,
including an extraction/product Pattern. It does not collapse
`-> (r first, d second)` to one synthetic result name. Pattern-directed value
delivery remains a later pass.

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

### 5.1 Result delivery is ordinary Pattern binding

For a callable declared with an extraction result:

```lang
-> (r first, d second)
```

there is no extra anonymous aggregate output slot that can be written as a
shortcut. Explicit body writes address the bound outputs `r` and `d`
separately. Alternatively, a bare terminal expression delivers one result
object and is checked exactly as the ordinary binding judgment:

```text
Deliver(expr, frame)
  == expect that `let (r first, d second) = expr` can match
```

The same whole-Pattern delivery applies to early-return terminals after target
selection:

```text
expr return
expr (Self return)
```

The first selects the outermost enclosing function layer; the second selects the explicitly
named active Self frame. Their result matching rule is identical. Only the
return-target layer differs. A nested explicit target therefore does not
introduce a second tuple-assignment, decomposition, or return-value algebra.

Bare tail delivery and `Done_Return` delivery both read the declared result
Pattern directly. They do not insert `?`; they do not broadcast one expression
to each output binder; and they do not synthesize positional outputs outside
the normal Pattern matcher.

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

## 7. Consumer coverage

| Concept | Connected consumer | Canonical relation |
|---|---|---|
| Return terminal forms | Parsed, normalized as `ReturnEvent` | Same |
| Target syntax | Preserved unresolved, then bound by `ReturnTargetBinding` | Resolved to full callable-frame self capability |
| Implicit return | Outermost enclosing function layer; binder alignment pending | Lowered/completed through enclosing self capability |
| Explicit self target | Attempts active self-frame match; does not silently fall back to nearest | Full self capability object |
| Nested unmaterialized closure return | Preserved as unbound nested closure material | Bound when the closure is materialized/elaborated as its own body |
| Return binding slot | Complete normalized slot/Pattern retained on the target frame | Used as `let ResultPattern = expr` expectation |
| Extraction-result delivery | Not executed | Explicit writes target each binder; a terminal expression matches the whole result Pattern |
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

`ReturnSelfIdentity` uses the callable's normalized semantic owner. Written
binder spelling is diagnostic material only and never participates in target
identity. Explicit self-target resolution must supply that stable owner
identity; without it the event remains unbound.

`ReturnSlotRef.binding_slot` deliberately retains the complete
`NormBindingSlot`. `ReturnSlotRef.name` is only a convenience for the current
single-binder diagnostic substrate and is `None` for a product/extraction
return. Future result delivery must consume `binding_slot`; it must not rebuild
the result Pattern from `name`.

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
