# Targeted Return and D-Reduction

**Status: Canonical semantics; return-target binding substrate is partially
implemented, while completion propagation and whole-Pattern delivery remain
pending consumers.**

This document defines canonical semantic lowering for targeted return
syntax and D-reduction. The current implementation deliberately stops
at return target binding.

The current implementation provides the structural syntax and normalized
AST (`ReturnEvent`, `TailValue`, and unresolved return target syntax) plus
a minimal semantic return-target binding pass. D-reduction, completion
propagation, and execution/lowering consumers remain unconnected.

This document owns targeted-return completion and its D-reduction boundary.
Automatic require does not define a second return/control algebra. Match
structure in the common semantic continuation contains D residual and Done
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

Canonical semantic lowering for the three return terminal forms:

```text
E return;
  => E |> (Self₀ return)

E |> (T return);
  => targeted return to resolved T

E (T return);
  => targeted return to resolved T
```

where Self₀ identifies the outermost enclosing function frame selected by
implicit return, obtained from the active return-target context. It does not
mean the most recently entered callable frame.

The implicit return spelling `E return;` selects the outermost enclosing
function layer. The current active-frame binder still selects its most recent
frame; alignment to this rule is consumer work, not an alternate semantics.

## 2. Return Capability Completion

Canonical return completion is mediated by the callable frame's return
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

## 3. Internal Return Completion

Targeted return produces a `Done_Return` completion:

```text
Done_Return(Self, pattern(E), value(E))
```

where:

- `Self` identifies the enclosing function-object receiving the return.
- `pattern(E)` is the structural pattern of the returned value.
- `value(E)` is the evaluated return value.

Done_Return is notation for internal target-completion state, not an Object,
Pattern or user constructor. It is unavailable to lookup, Norm, @, ref/share,
storage or ordinary Pattern matching. A user name with that spelling has no
completion authority. Representation may use ReturnComplete instead. It is **not** represented
in the current normalized AST. The current `NormReturnEvent` is a
surface-structure node, not a semantic completion.

## 4. No local normal result contribution

```text
LocalNormalContribution(ReturnEvent) = none
TargetCompletion = ReturnComplete(target, ordinary payload)
```

The local path is completed; it does not produce unit, zero or a user-visible
Done value. Target completion propagates internally until its matching frame.
Its eventual payload undergoes ordinary ReturnPattern delivery.
The current build evaluator does not execute this propagation.

## 5. D-Reduction Boundary

At the boundary matching the resolved return target, the internal completion
is consumed and its ordinary payload undergoes whole-Pattern delivery against
the target ReturnPattern.

```text
At boundary matching Selfᵢ:
  the internal target completion is consumed
  its ordinary payload is checked against the target ReturnPattern
```

D-reduction semantics is defined here; its completion-propagation and delivery
consumers are not connected in the build evaluator. The parser and normalizer
preserve source structure rather than execute this semantic boundary.

### 5.1 Result delivery is ordinary Pattern binding

The terminal payload is evaluated directly under the target's established
ReturnPattern/Pout demand, before the payload root call seals its candidate
maxima. Delivery does not first complete an unconstrained temporary and then
rematch it. An explicitly written user binding still creates its own boundary.
Both established outer P1 and P2 constrain the immediate inner call's P1/P2;
the selected inner call cannot be reopened by later use. See
[Policy demand](../symbol-world/symbol-policy-and-compile-flow-projection.md#39-direct-result-delivery-and-two-sided-forwarding).


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
  - propagates the internal target completion upward
  - contributes no normal local result

When Selfᵢ is reached:
  - the internal target completion is consumed
  - its ordinary payload undergoes whole-Pattern delivery against the target ReturnPattern
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
| `Done_Return` | Not represented | Internal target-qualified completion |
| D-reduction | Not implemented | Consume target completion and perform ordinary whole-Pattern delivery |
| Local return contribution | Not implemented | No normal value contribution; no fabricated unit |
| Target propagation | Not implemented | Propagate internal completion to its matching target |
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
