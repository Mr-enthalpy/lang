# Targeted Return and D-Reduction

**Status: Canonical semantics; return-target binding substrate is partially
implemented, while completion propagation and whole-Pattern delivery remain
pending consumers.**

This document defines canonical semantic lowering for targeted return
syntax and D-reduction. The current implementation deliberately stops
at return target binding.

The current implementation provides the structural syntax and normalized
AST (`Expr`, explicit `ReturnEvent`, and Omitted/Explicit target syntax) plus
a minimal semantic return-target binding pass. D-reduction, completion
propagation, and execution/lowering consumers remain unconnected.

This document owns targeted-return completion and its D-reduction boundary.
Automatic require does not define a second return/control algebra. Match
structure in the common semantic continuation contains D residual and Done
completion constructors defined here and in
`../patterns-overload/static-pattern-spaces-and-extraction-chains.md`; compile
projection preserves them homomorphically. Projection and require slicing are
canonical in
`../symbol-world/policy-and-static-flow-projection.md`.

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

where Self₀ is the callable-object tag plus actual activation of the implicit
target in the current **reachable return segment**. For continuation K, scan
active enclosing callable frames outward from the current position. The nearest
Ordinary closure frame is included and ends this segment; no frame outside it
is reachable. InPlace adds no barrier. If all encountered boundaries are known
InPlace, the segment extends to the outermost enclosing function frame.

```text
ImplicitReturnTarget(K) = outermost function frame in ReachableReturnSegment(K)
ExplicitReturnTarget(K) must belong to ReachableReturnSegment(K)
ExplicitReturnTarget(K) cannot be more outer than ImplicitReturnTarget(K)
```

The implicit return spelling `E return;` uses this segment query, not a global
first-frame or always-nearest-frame rule. The active-frame binder implements
the query without inferring whether a plain expression produces an event.
Missing boundary evidence makes the query unavailable (preserved unbound,
Diagnostic or Residual at the requiring boundary), never implicitly transparent.
A proved nearer Ordinary barrier seals its segment independently of unknown
boundaries farther outward. Frame existence alone proves no reachability.

### 1.1 Two distinct implicit operations

```text
E return;             -- explicit ReturnEvent, omitted ReturnTarget
E (T return);         -- explicit ReturnEvent and explicit target
E;                    -- ordinary expression completion, consumer decides below

ImplicitReturnEvent != ImplicitReturnTarget
```

The first form already requests return. Target inference supplies Self₀; it
does not decide whether the event exists. A plain expression instead enters
the serial/block consumer on its SemanticContinuation path:

```text
non-tail e:
  UnitDiscard(e): evaluate e once; require Type(result)=unit; discard; continue

true path-tail e:
  Type(result)=unit  -> ordinary fallthrough
  Type(result)!=unit -> synthesize ReturnEvent -> infer omitted ReturnTarget

explicit unit_value return; -> ReturnEvent, never ordinary fallthrough
```

UnitDiscard names this existing E consumer judgment, not a new primitive,
lexical binding, hidden temporary or evaluator. The explanatory actions
`let _:t = express; (t == unit) assert;` are not a source rewrite: introducing
bindings would change lexical/contextual observations.

Tail is determined on each continuation/control-flow path, including branch
paths, not by the last node of an AST list. ReturnEvent completes its path;
unit fallthrough continues the ordinary enclosing control relation. It neither
manufactures a return event nor a result-slot write.

For a path that delivers a return payload, the established ReturnPattern/Pout
demand applies before its root call seals maxima (§5.1). These judgments do not
evaluate an unconstrained temporary to discover a type, execute candidate
bodies speculatively, re-evaluate e, or reopen selection. Type/result evidence
and the consumer belong to the same ordinary E relation.

Plain expressions remain Expr in Raw/Norm. The Omitted target tag records only
missing target syntax on an explicit event. Serial completion remains pending.

## 2. Return Capability Completion

Canonical return completion is mediated by the callable frame's return
capability. That capability is exposed through the callable-local `Self` space
as an ordinary callable capability value, as described in
`spec/design/symbol-world/function-object-self-and-return-capability.md`.

Return is not a parser keyword escape hatch, an operator, or a
compiler-intrinsic control action. The target-binding pass identifies the
active frame; return-capability execution belongs to the completion consumer.

The same segment restriction applies to ordinary return-capability invocation,
checked at its requesting continuation K, not reset by a helper call frame.
Capturing an outer target/capability grants no authority to cross an Ordinary
barrier: a transmissible Ordinary G invoked while its enclosing F remains active
still cannot return to F. Target syntax, stored capability and direct capability
invocation cannot bypass this guard. InPlace transparency grants no exemption
from ordinary lifetime, dependency or activation checks.

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

- `Self` denotes the callable-object tag resolved to its actual active frame.
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
[Policy demand](../symbol-world/policy-and-static-flow-projection.md#39-direct-result-delivery-and-two-sided-forwarding).


For a callable declared with an extraction result:

```lang
-> (r first, d second)
```

there is no extra anonymous aggregate output slot that can be written as a
shortcut. Explicit body writes address the bound outputs `r` and `d`
separately. Alternatively, a non-unit path-tail expression synthesizes a return
event whose one result object is checked by the ordinary binding judgment:

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

The first selects the outermost function in the reachable return segment; the
second selects the explicitly named active Self frame within that same segment. Their result matching rule is identical. Only the
return-target layer differs. A nested explicit target therefore does not
introduce a second tuple-assignment, decomposition, or return-value algebra.

Synthesized non-unit tail returns and explicit `Done_Return` delivery both
read the declared result Pattern directly. A unit tail instead falls through.
Return deliveries do not insert `?`; they do not broadcast one expression
to each output binder; and they do not synthesize positional outputs outside
the normal Pattern matcher.

## 6. Non-Local Target Propagation

After target identity and segment reachability are proved, a return targeted at
`Selfᵢ` propagates only through intermediate boundaries of that same segment
until `Selfᵢ` is reached. It never crosses an Ordinary frame to an outer target:

```text
Each reachable intermediate boundary:
  - propagates the internal target completion upward
  - contributes no normal local result

When Selfᵢ is reached:
  - the internal target completion is consumed
  - its ordinary payload undergoes whole-Pattern delivery against the target ReturnPattern
```

If no matching active target exists at any reachable boundary,
a semantic diagnostic is emitted.

The current `ReturnTargetStack` checks active activation identity and segment
reachability from explicit Ordinary/InPlace boundary evidence. An active target
outside the segment is diagnosed; unknown boundary evidence preserves the event
unbound. It does not execute capability calls, propagate completions or perform
D-reduction. Those consumers must reuse the same reachability guard.

Each genuinely crossed delivery/exit boundary also processes its actual outward
lifecycle material: compress local-name dependencies while preserving origin
choices, regions, colors, effects and validity. This includes a target return
crossing nested InPlace boundaries. It introduces neither a local unit nor an
already completed result binding at an intermediate boundary, and it changes
neither target reachability nor fixed cleanup order. A borrow of a dying local
cannot be made valid by compression. See the
[boundary algorithm obligations](../lifetime/lifetime-policy-and-overload-boundary.md#10-internal-actions-origin-compression-and-automatic-post)
and the [nested-return walkthrough](../lifetime/lifetime-algebra-walkthroughs.md#1-branches-recursion-and-nested-inplace-exits).

## 7. Consumer coverage

| Concept | Connected consumer | Canonical relation |
|---|---|---|
| Return terminal forms | Parsed, normalized as `ReturnEvent` | Same |
| Target syntax | Preserved unresolved, then bound by `ReturnTargetBinding` | Resolved to full callable-frame self capability |
| Omitted ReturnTarget | Boundary-aware reachable-segment query; unknown stays unbound | Outermost function in the reachable segment, after a ReturnEvent exists |
| Implicit ReturnEvent / serial consumer | Path-tail and UnitDiscard consumer pending | Non-tail requires unit; tail unit falls through; tail non-unit synthesizes ReturnEvent |
| Explicit self target | Attempts active self-frame match; does not silently fall back to nearest | Full self capability object |
| Nested unmaterialized closure return | Preserved as unbound nested closure material | Bound when the closure is materialized/elaborated as its own body |
| Return binding slot | Complete normalized slot/Pattern retained on the target frame | Used as `let ResultPattern = expr` expectation |
| Extraction-result delivery | Not executed | Explicit writes target each binder; a terminal expression matches the whole result Pattern |
| `Done_Return` | Not represented | Internal target-qualified completion |
| D-reduction | Not implemented | Consume target completion and perform ordinary whole-Pattern delivery |
| Local return contribution | Not implemented | No normal value contribution; no fabricated unit |
| Target propagation | Not implemented | Propagate internal completion to its matching target |
| Target validity check | Active identity plus Ordinary/InPlace segment guard | Same guard for explicit targets and capability calls; execution pending |

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
`ReturnTargetFrame` with known Ordinary/InPlace boundary evidence; leaving that
body pops it. Low-level `push_frame` without evidence records Unknown and never
assumes a transparent boundary. `push_frame_with_boundary` supplies explicit
evidence, and `target_reachability` is the common explicit/capability guard.
A nested closure literal is preserved as value material unless that closure is
explicitly elaborated as
its own returnable body. Therefore a return inside an unmaterialized nested
closure is not bound to the outer frame.

`ReturnSelfIdentity` uses the callable's normalized semantic owner. Written
binder spelling is diagnostic material only and never participates in target
identity. Explicit target resolution supplies the callable tag plus actual
active activation; lexical owner identity or spelling alone is insufficient.
Without that semantic resolution or required boundary evidence the event stays
unbound.

`ReturnSlotRef.binding_slot` deliberately retains the complete
`NormBindingSlot`. `ReturnSlotRef.name` is only a convenience for the current
single-binder diagnostic substrate and is `None` for a product/extraction
return. Future result delivery must consume `binding_slot`; it must not rebuild
the result Pattern from `name`.

The build-layer source callable hook currently runs this pass as validation.
It rejects malformed return targets but does not store bound events in
`SourceCallableSyntax`; later body evaluators may re-run target binding when
they need the bound return-event stream for completion/result semantics.

Diagnostics are structured:

```text
ReturnOutsideReturnableContext
ReturnTargetNotActive
ReturnTargetOutsideReachableSegment
AmbiguousReturnTarget
UnsupportedReturnTargetForm
```

Each diagnostic carries provenance. These diagnostics are target-binding
diagnostics only; they do not imply return value type failure, lifetime
failure, or lowering failure.

## Callable tag to active frame

```text
ReturnTarget=current callable object tag
ResolveReturn(K,tag,activation)=active frame F
CallType(tau): CallTarget=tau, ActualSelf=c* in V_tau
Type(tau_C)=type; Type(type)=(1 type)
Enter(call_anchor,new_args) -> ordinary new call, fresh activation
ReturnEvent(F,v) -> Deliver(ReturnPattern(F),Pout(F),v)
                -> Complete(CallTarget(F),Activation(F),result)
```

Resolution uses actual control-flow activation, independent of self parameter
type or capture layout. Recursive invocations of one tag create distinct
activations. Omitted target selects the outermost function in the reachable
segment; explicit tag selects its associated current activation only inside that
segment. Neither tag nor captured capability supplies reachability or candidate
preference.

The [call owner's](../symbol-world/function-object-call-model.md#62-consumer-directed-self-projections)
CallAnchor supports `(args) |> self`; Param's capture/receiver transport uses
the current Move/borrow/lifetime facts. Reentry creates a new activation even
for the same callable tag. Continue-like paths enter the legal tagged callable;
break-like paths complete the resolved active return frame through this same
target-return/D-reduction relation. Neither creates a for primitive or reduces
target return to an ordinary result. Existing return barriers, cleanup and
tail-position legality remain. Expressible recursion proves no physical stack
reuse; later tco/loop consumes independent evidence. Activation execution and
completion propagation remain pending beyond the target-binding substrate.
