# Call Modes, Recursion, and Tail Lowering

**Status: Non-normative future design. Not implemented as current parser, normalizer, IR, ABI, optimizer, or runtime behavior.**

This document explores internal mechanical call realizations labelled `normal`,
`tco` and `loop`. They are not language-level call modes or proposed source
syntax. They may materialize only already proved semantic delivery, transport,
lifetime and cleanup facts before machine lowering.

It is a future design note. It is not current public language behavior, not an
implemented pass, not a parser or normalizer rule, and not an LLVM / machine ABI
design. The document is self-contained: it does not require the reader to
assemble its meaning from other documents.

## 1. Purpose

This document defines a future call-mode lowering design. The problem it solves:

```text
source recursion
  -> normalized first-order call site
  -> tail-position analysis
  -> call-mode selection
  -> explicit call action in later IR
```

This document does **not** define a full ABI, an LLVM backend, current parser
syntax, a full borrow/lifetime checker, or machine stack layout.

## 2. No loop syntax as semantic core

The language does not have a `while` / `for` loop as a core semantic mechanism.
Repetition is expressed by recursion.

```text
repetition is recursion
memory reuse is call semantics
```

The user writes recursion; a later mechanical consumer determines from semantic evidence whether
the call is an ordinary call, a new-frame call, a tail call with argument
transport, or an in-place reuse of parameter slots.

This is not "rely on the optimizer to perform tail-call optimization". It is not
an optimization wish. Internal call realizations must preserve the semantic guarantees fixed by E.

## 3. The three call modes

There are three call modes.

### normal

An ordinary call. It establishes a new call relationship. The current call frame,
parameter slots, and return continuation are not promised to be reused.

```text
CallNormal(callee, args)
```

### tco

`tco` is a tail call that actively transports arguments. It requires the call
site to be in tail position, but the target function's parameter slots may not
naturally correspond to the current slots. Lowering must therefore first complete
argument transport / rearrangement, then replace the current continuation with
the target call.

```text
CallTco(callee, adapted_args)
```

`tco` is not "might be optimized into a tail call". It is already-determined
tail-call semantics.

### loop

`loop` is a stronger tail call. It corresponds to a tail call that needs no
rearrangement of any existing input parameter slots. It can be understood as a
recursion-shaped back edge to a loop entry, or a frame/slot reuse when the
parameter slot layout already matches.

```text
CallLoop(entry, args)
```

`loop` is not an ordinary `goto`. It is a checked call mode that depends on
future checks: parameter-slot isomorphism, tail position, lifetime/borrow state,
and drop/cleanup boundaries.

## 4. Automatic call-mode insertion

Call-mode insertion is the same kind of mechanical insertion as automatic
argument passing and automatic return normalization.

Automatic insertion must not happen on the surface AST. It must happen on a
sufficiently normalized first-order AST, because the surface AST's last
expression may still be changed by:

```text
argument passing normalization
return normalization
error propagation insertion
compile reduction / residualization
cond / guarded expression lowering
```

The input to tail-position analysis is therefore a future first-order normalized
AST, not raw surface syntax:

```text
surface AST
  -> syntactic normalization
  -> pattern / argument / return normalization
  -> compile reduction / residualization
  -> first-order AST
  -> tail-position analysis
  -> automatic call-mode insertion
  -> later IR with explicit call actions
```

The automatic strategy is conceptually:

```text
if call site is not in tail position:
    normal

if call site is in tail position and loop conditions hold:
    loop

if call site is in tail position and tco conditions hold:
    tco

otherwise:
    normal
```

Any later tail-transfer realization consumes tail-position and legality evidence
on the semantic first-order representation, never a blind surface rewrite or
an AST-list-position heuristic.

## 5. Internal lowering evidence, not source call modes

`normal`, `tco` and `loop` are internal mechanical research labels only. This
note introduces no source annotation, reserved word or promise of future manual
mode syntax. Source recursion retains ordinary callable selection, parameter
transport, return-segment barriers and lifetime/cleanup obligations.

E fixes the applicable FreshLet/ExistingTarget delivery law and its ABI guarantee;
O materializes it. Neither a source mode nor an optimizer chooses ABI1/ABI2.
A possible later tail/back-edge realization must prove tail position, legal
callee/entry, compatible slots, already selected producer/Move actions, legal
borrow/frontier state and completion/cleanup order. Missing evidence leaves
that realization unavailable; it cannot change semantic call selection or
weaken return barriers. No physical stack-reuse guarantee follows merely from
expressible recursion.

## 6. Relation to mechanical argument passing

A call mode depends on already-normalized argument passing.

`tco` does not transport raw source arguments. It transports the parameter
objects that result from argument-passing normalization (semantic metanotation):

```text
copy(x) when x : ordinary T:
  share(x) -> selected clone -> fresh complete result -> Move(result)
copy(x) when x : T ref | T share:
  rebind(x) -> selected clone -> fresh complete result -> Move(result)
share(x) -> b = share_borrow(x); move(b)
ref(x)   -> b = ref_borrow(x); move(b)
move(x)  -> move(x)
```

These selected ordinary share/rebind and clone actions do not authorize
automatic pass adaptation to choose `share` or `ref`. Pass=Move; Preserve Move
uses no clone and is not the copy-derived path shown above. Cleanup points and
same-point reverse-declaration order are fixed before lifecycle observation.

`tco` transports the final objects that enter the parameter slots. `loop`
additionally requires that those objects can be interpreted in the existing slots
without rearrangement.

This document does not restate the full automatic argument-passing design; see
the future mechanical argument passing document,
`spec/design/mechanical-lowering/mechanical-argument-passing-and-move-fixed-point.md`.

## 7. Relation to compile functions

Compile functions also have no loop core. Repetition in compile computation is likewise
expressed by recursive function calls. A future compile execution engine should
recognize `normal` / `tco` / `loop` rather than inventing a separate loop for
compile functions.

```text
Compile functions and runtime functions share the same call-mode vocabulary.
```

This is internal future research; the current compile evaluator is not claimed
to implement it. Any compile tail/back-edge realization requires the same
semantic legality evidence; source code selects no manual mode. Missing proof
grants no tail-transfer or stack-reuse guarantee.

## 8. Relation to IR and backend

This document does not design a machine ABI, and it does not require implementing
a custom LLVM ABI now.

The direction for the language IR is that call mode is made explicit before the
backend. The backend should not have to guess call semantics from an ordinary
call.

```text
CallNormal(callee, args)
CallTco(callee, adapted_args)
CallLoop(entry, args)
```

`loop` and `tco` are not "optional optimizer passes". They are products of
language lowering.

Non-goal note: although future work may study caller/callee stack-slot layout,
the caller writing arguments into the callee frame, or the callee rewriting its
return into argument slots, this document does not define those machine-ABI
details.

The canonical [E delivery convention](../static-evaluation/evaluation-residual-and-optimization.md#10-e-selected-return-delivery-convention)
already selects ABIKind from FreshLet/ExistingTarget. This future mechanical
consumer must preserve it, including ABI1's input-frontier return contract.
Tail position, normalized Move, borrow/lifetime, cleanup and slot/result evidence
determine reuse legality. RVO/NRVO success, optimizer opportunity and recursion
alone are not substitutes for those facts. This note adds no new source
annotation or promise of a future manual call-mode surface.

## 9. Tail position

Tail position must be judged on the first-order AST.

```text
A call is tail-position eligible only after normalization has removed or made explicit:
  implicit return normalization
  implicit error propagation
  guarded branch residualization
  argument adaptation
  local cleanup/drop boundaries
```

If a call looks syntactically last in the surface syntax but still has work that
must execute after it once lowering is applied, then it is not a legal tail call —
unless that work can be proven to complete before the tail transfer.

An early return capability call (`self..return(d)`) is **not** a tail call
candidate. It has no normal local contribution and carries an internal
target-qualified completion, as defined by the targeted-return owner. Tail
analysis sees the already-explicit block-local result boundary and return
capability effect — it does not treat `self..return(d)` as an ordinary callee
eligible for tail-call optimization.

## 10. Non-goals

```text
No current parser syntax change.
No current normalizer behavior change.
Implementation scope: this document defines lowering obligations and does not
change the Rust implementation.
No backend or LLVM ABI design.
No machine stack layout design.
No full borrow/lifetime legality checker.
No full drop/cleanup ordering specification.
No runtime lookup implementation.
No type checker implementation.
No guarantee that current recursion is optimized.
No while/for surface syntax design.
```

## 11. Relationship to other documents

The documents below are adjacent design. They do not define the call-mode model
specified here, and this document does not depend on them for its meaning.

- `mechanical-argument-passing-and-move-fixed-point.md` — produces the parameter
  objects that `tco` transports and that `loop` reuses in place.
- `mechanical-return-normalization-and-error-policy.md` — the return-slot
  counterpart whose normalization must be resolved before a call is
  tail-position eligible.
- `compile-instance-invocation-and-result-delivery.md` — the invocation engine that
  should share the same `normal` / `tco` / `loop` call-mode vocabulary.
- `pattern-normalization-and-first-order-overload.md` — part of the normalization
  that produces the first-order AST on which tail-position analysis runs.
- `overload-resolution-design.md` — candidate selection precedes the invocation
  lowering that produces explicit call modes.
