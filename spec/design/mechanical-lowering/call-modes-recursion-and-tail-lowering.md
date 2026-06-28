# Call Modes, Recursion, and Tail Lowering

**Status: Non-normative future design. Not implemented as current parser, normalizer, IR, ABI, optimizer, or runtime behavior.**

This document specifies the future *call-mode lowering* design: the three
language-level call modes `normal`, `tco`, and `loop`, how they replace
language-level loop syntax, and how they are mechanically inserted as explicit
call actions before IR lowering.

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

The user writes recursion; the language decides — through a call mode — whether
the call is an ordinary call, a new-frame call, a tail call with argument
transport, or an in-place reuse of parameter slots.

This is not "rely on the optimizer to perform tail-call optimization". It is not
an optimization wish. Call modes are language-level lowering semantics.

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
meta reduction / residualization
cond / guarded expression lowering
```

The input to tail-position analysis is therefore a future first-order normalized
AST, not raw surface syntax:

```text
surface AST
  -> syntactic normalization
  -> pattern / argument / return normalization
  -> meta reduction / residualization
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

If the design wants the "last call defaults to `tco`", that rewrite happens on
the tail-position-confirmed first-order AST, never as a blind rewrite of surface
syntax.

## 5. Manual call-mode annotation

A user may manually specify a call mode, but a manual mode is not a free hint.

Manual specification dominates the automatic strategy:

```text
explicit call mode > automatic call-mode insertion
```

But manual specification introduces compile-time checks. This document does not
design every check; it lists check categories so that later work does not assume
manual `loop` / `tco` is accepted unconditionally.

Manual `loop` requires, at least, the following future checks:

```text
call site is tail position
callee/entry is a legal loop target
existing parameter slots need no rearrangement
parameter slot layout is compatible
argument pass actions are already determined
borrow/lifetime state permits the back edge
required cleanup/drop actions do not block the loop edge
```

Manual `tco` requires, at least, the following future checks:

```text
call site is tail position
current continuation can be replaced
argument transport order is well-defined
move/copy/ref/share actions are already determined
cleanup/drop actions can run before the tail transfer
callee entry permits tco
```

If a manual check fails, the compiler should report a compile-time error. It must
not silently fall back to `normal`.

## 6. Relation to mechanical argument passing

A call mode depends on already-normalized argument passing.

`tco` does not transport raw source arguments. It transports the parameter
objects that result from argument-passing normalization:

```text
copy(x)  -> tmp = copy_construct(x); move(tmp)
share(x) -> b = share_borrow(x); move(b)
ref(x)   -> b = ref_borrow(x); move(b)
move(x)  -> move(x)
```

`tco` transports the final objects that enter the parameter slots. `loop`
additionally requires that those objects can be interpreted in the existing slots
without rearrangement.

This document does not restate the full automatic argument-passing design; see
the future mechanical argument passing document,
`spec/design/mechanical-lowering/mechanical-argument-passing-and-move-fixed-point.md`.

## 7. Relation to meta functions

Meta functions also have no loop core. Repetition in meta computation is likewise
expressed by recursive function calls. A future meta execution engine should
recognize `normal` / `tco` / `loop` rather than inventing a separate loop for
meta functions.

```text
Meta functions and runtime functions share the same call-mode vocabulary.
```

This is a future design statement; the current meta evaluator is not claimed to
implement it. For strict meta execution, if a recursive meta function later uses
manual `loop` or `tco`, it is still subject to the same category of legality
checks. If a check fails, that is a compile-time / meta-execution error.

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
candidate. It goes through the dual-channel model (branch-local `Done(unit)`
plus final-accumulator `Done(D)`) rather than through ordinary callee TCO. Tail
analysis sees the already-explicit block-local result boundary and return
capability effect — it does not treat `self..return(d)` as an ordinary callee
eligible for tail-call optimization.

## 10. Non-goals

```text
No current parser syntax change.
No current normalizer behavior change.
No Rust implementation change in this PR.
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
- `meta-object-invocation-and-policy-reduction.md` — the invocation engine that
  should share the same `normal` / `tco` / `loop` call-mode vocabulary.
- `pattern-normalization-and-first-order-overload.md` — part of the normalization
  that produces the first-order AST on which tail-position analysis runs.
- `overload-resolution-design.md` — candidate selection precedes the invocation
  lowering that produces explicit call modes.
