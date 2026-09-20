# Mechanical Argument Passing and the Move Fixed Point

**Status: Canonical target semantics for the named pass-action algebra below.
The selection algorithm, parser/normalizer integration, checker, IR, ABI, and
runtime lowering remain non-normative and unimplemented.**

This document specifies the future *mechanical argument passing* layer: how a
call argument is normalized into a concrete pass action (move or copy), while
an explicitly formed borrow is moved as an already existing borrow value,
before a function or meta-function body receives it. Its central claim is that
pass modes are mechanically inserted, source-expressible actions — not backend
ABI heuristics and not optimizer decisions — and that `move` is the fixed point
of pass normalization.

It is not current public language behavior or an implemented pass. The pass-action core, instance lifecycle boundary and cleanup/with rules are
normative. Selection algorithms and lowering/IR representations remain pending.
The [lifetime owner](../lifetime/lifetime-policy-and-overload-boundary.md)
defines the shared Killable, MoveEffect, Movable and Pre/Post judgments.

## 0. Canonical pass-action core

The following small algebra is normative and is consumed by Policy and ordinary
binding semantics:

```text
CanonicalMechanicalPassCore:

MoveFixedPoint:
  move(move(x)) = move(x)

CopyAction:
  copy(x) =
    tmp := CopyConstruct(x)
    Move(tmp)

CopyConstructExpansion:
  ordinary x : T
    CopyConstruct(x)
      ~= shared_view := share(x); clone(shared_view)

  x : T ref | T share
    CopyConstruct(x)
      ~= rebound_view := rebind(x); clone(rebound_view)

  therefore:
    copy(ordinary T)
      ~= share -> clone -> terminal Move

    copy(T ref | T share)
      ~= rebind -> clone -> terminal Move

NoPreMoveBeforeCopy:
  copy(x) != Move(x)
  copy(x) = CopyConstruct(x); Move(result)

ExplicitPassDominates:
  explicit pass present => preserve and check that action

AutomaticPassDomain:
  automatic pass in {move, copy}
  automatic pass not in {ref, share, @}

ProducerConsumerModeSeparation:
  ProducedMode(source) = mu_source
  PolicyMode(destination) = mu_destination
  Transfer(source, destination, pass)
    preserves ProducedMode(source) = mu_source
    and installs the destination under mu_destination
```

This core fixes action meaning and normalization only. It does not choose when
an automatic slot moves or copies, prove copy/borrow legality, prescribe an IR
instruction, or define an ABI. `TransferToDestination` in the canonical binding
judgment is the ordinary-binding specialization of `Transfer` above, not a
second transfer algebra.

`CopyConstruct` names the selected ordinary copy-family realization; it is not
a new opaque semantic primitive. The `~=` equations expose its existing value
algebra. Concrete source spelling may differ, but ordinary values obtain a
share/read view and clone it, while borrow values use their existing rebind
path and clone the rebound view; both then perform exactly one terminal move.
`CopyConstruct` contributes no additional lifecycle-origin law: its
`lifecycle_post` is exactly the post of the selected share/rebind-plus-clone
realization. Any `origin(result)=...` relation must be declared by that selected
clone-family candidate.
These share/rebind steps occur inside the selected copy realization. They do
not authorize automatic argument adaptation to invent a borrow and do not
enlarge `AutomaticPassDomain` beyond `{move, copy}`.

## 1. Purpose

This document defines the mechanical argument-passing normalization that happens
at a call's argument slots. The problem it solves:

```text
source argument
  -> raw argument shape
  -> explicit pass extraction
  -> automatic move/copy selection, if no explicit pass
  -> concrete pass action
  -> eventual IR/action layer receives fully decided movement/borrow/copy actions
```

The compiler's only privilege is to fix the *insertion framework* during
normalization / lowering: it mechanically inserts a move/copy pass action at each value
argument slot. The actual action that gets inserted is still decided by
in-language facts — types, traits, policy, meta-functions, and symbol lookup —
not by an opaque backend convention.

This document does **not** define a full type checker, a full borrow checker, an
ABI, LLVM lowering, runtime overload resolution, or a full trait solver. Those
parts remain future design/implementation even though the named action algebra
above is canonical target semantics.

## 2. Pass modes are mechanical source-level lowering, not ABI heuristics

`move`, `ref`, `share`, `copy`, and `in` are not backend heuristics and not
calling-convention choices. At the language level they are visible, checkable
actions that meta-code can describe and that legality checks can inspect.

`ref` and `share` are nevertheless different from automatic move/copy choice:
they form borrow Objects and must be explicit before candidate adaptation. This
layer may transport an already formed borrow handle; it may not invent one.

The compiler may mechanically insert a default pass action, but once inserted it
must become an ordinary semantic object/action. The IR must not carry a
"default pass undecided" state.

```text
default only exists before lowering
IR must not receive `in`
IR receives concrete actions:
  Move
  CopyConstruct + Move
  Move(existing explicit ref/share handle)
```

By the time an action reaches the IR/action layer, the movement/borrow/copy
decision is already fully made.

## 3. Explicit pass mode dominates automatic strategy

An explicit pass mode always has the highest priority:

```text
arg |> move   => move
arg |> ref    => ref
arg |> share  => share
arg |> copy   => copy
```

Once a source or normalized argument slot already carries an explicit pass mode,
automatic strategy must not rewrite it.

A manual move performs its predetermined MoveEffect, Kill or Preserve. `Copyable` only guarantees that a
value *can* be copied; it does not permit optimizing an explicit `move` into a
`copy`, and it does not permit downgrading an explicit `move` into a `share` or
`ref`.

Manual pass modes are not free hints. They are semantic requirements, and future
work will introduce corresponding compile-time legality checks — for example,
explicit `copy` requires copyable, explicit `ref` requires an exclusive borrow,
explicit `share` requires a shared borrow, and explicit move requires Movable at the current continuation frontier. The detailed conditions are out of scope here;
the point is that a manual pass mode is a requirement, not a suggestion.

## 4. Default pass insertion

An argument with an explicit pass retains it. Otherwise the ordinary
instance/context-dependent pass judgment selects a concrete move or copy.
It never inserts ref, share or @. Before action lowering there must be a
selected realization with its ordinary legality evidence; no unresolved
default-pass marker reaches an executing action.

Copyable establishes a possible copy realization, not a mandatory default.
Type, policy, target facts and ordinary instance facts can constrain selection;
layout size alone supplies no capability. The concrete selection algorithm is
pending and cannot substitute a heuristic for the canonical judgment.

## 5. Uniform instance participation

Type, rank, namespace, meta, Pattern and verification material do not form a
non-value pass-through category. Every transported Object instance follows
the same pass-action and lifecycle relations. Stage affects observation and
readiness, not whether a lifecycle subject exists.

```text
Killable_K(n) != Movable_K(n,m)
MoveEffect_K(n,m) ∈ {Kill, Preserve}
```

The lifetime owner defines the narrow Preserve proof and the frontier Pre.
A globally surviving type instance and a meta-local type temporary may have
different effects despite equal values. Nonkillability does not grant copy,
borrow or movement capability.

## 6. Move is the fixed point

This is the core of the document. The central axiom of pass normalization:

```text
T move == T
rank move == rank
move(move(x)) == move(x)
```

`move` is not a type constructor. It does not produce a new type value such as
`T move`, it does not change rank, and it does not change a classifier. It is the selected transfer into the destination with its predetermined
Kill or Preserve effect.

Therefore pass normalization must not recursively produce:

```text
T move move
rank move move
borrow-of-borrow-of-borrow
```

Once an action lands on `move`, normalization terminates.

The same fixed point applies to callable lookup. Moving a caller of type `T`
does not create `move::T`; its call entry is still resolved under `T`.
`ref(x)` and `share(x)` are different because they first construct borrow
objects of exact types T ref and T share. Each uses the matching () entry of
its own complete type/callspace, never a candidate adapted from T's entry.

## 7. All pass modes lead to move

There are four mechanical modes, defined in terms of `move`:

```text
move(x):
  require Movable_K(x,m)
  perform the predetermined MoveEffect_K(x,m)
  transfer into the argument slot

copy(x):
  tmp = CopyConstruct(x)
      ~= shared_view := share(x); clone(shared_view)     when x : ordinary T
      ~= rebound_view := rebind(x); clone(rebound_view)  when x : T ref | T share
  move(tmp)

ref(x):
  b = make_ref_borrow(x)
  move(b)

share(x):
  b = make_share_borrow(x)
  move(b)
```

Here `copy`, `ref`, and `share` are not endpoints. Each constructs some object
that can then be `move`d. The only passing endpoint is `move`.

- copy(x) applies the terminal move to tmp, without a pre-move of x.
- ref(x)/share(x) apply the terminal move to the formed handle b.
- move(x) ends x's generation exactly when its predetermined effect is Kill.
- Every materialized pass handle ultimately reaches a single terminal `move`
  action.

Two additional invariants close the Policy-mode boundary:

```text
PlainMaterializationPrinciple:
  destination PolicyMode ∈ {const, plain, mut}
  copy-to-destination = CopyConstruct(x) + terminal Move

NoPreMoveBeforeCopy:
  copy(x) ≠ move(x); CopyConstruct(x)
  copy(x) = CopyConstruct(x); terminal Move(result)
```

The const, plain, and mut destination cases use this same primitive. The
destination mode may affect candidate preference or capability realization,
but it does not introduce three different kinds of copy and never consumes
`x` before `CopyConstruct(x)` has completed. Nor does transfer relabel the
producer result: source and destination modes are independent slot facts.
`CopyConstruct` here is the ordinary selected copy-family candidate summarized
by `CopyConstructExpansion`, not an additional builtin whose behavior is left
uninterpreted.

## 8. Borrow movement preserves parent/origin

Moving a borrow handle does not create a deeper borrow chain.

```text
move(borrow_node(parent = p, kind = k))
  = borrow_node(parent = p, kind = k)
```

The equality here is a fixed point on type / rank / access shape. It does not
claim that the same runtime handle has no linear state change: when MoveEffect=Kill the old handle dies and a new handle inherits the same
parent/origin/kind. Preserve follows its independent proof.

If `b1` is a borrow produced from `x`, then `move(b1)` produces a *sibling*
borrow handle with the same origin as `b1`, not a *child* borrow of `b1`.

```text
The moved borrow handle keeps the same parent/origin. It does not make the
previous handle the parent of the new handle.
```

This is what keeps access-tree depth from growing without bound during argument
passing: a moved borrow does not increase access-tree depth, does not increase
rank, and does not change the type value.

## 9. Relation to overload and argument adaptation

Automatic pass insertion is not a blind pre-pass. Different overload candidates
may have different pass expectations at the same parameter slot, so the model
separates callee-independent normalization from candidate-dependent adaptation:

```text
callee-independent raw argument normalization:
  identify the argument instance and its ordinary observation
  detect explicit pass
  form RawArgShape

candidate-dependent argument adaptation:
  given ParameterShape
  validate an already explicit borrow, or choose move/copy if no explicit pass
```

As judgments:

```text
Γ ⊢ arg ⇓ RawArgShape

Γ ⊢ ParameterShape × RawArgShape ⇓ AdaptedArgShape

Γ ⊢ AdaptedArgShape ⇓ concrete pass action
```

An explicit argument pass dominates automatic pass. A parameter's pass
expectation participates in candidate compatibility. Automatic `in` is an
unresolved pre-lowering marker, never a canonical automatic action. It appears only
when there is no explicit argument pass and must resolve to one concrete action
in `{move, copy}`.

Conflict and adaptation examples:

```text
argument explicitly move, parameter expects share
  => candidate incompatible

argument explicitly copy, parameter expects move
  => candidate incompatible

argument automatic in, parameter expects share
  => candidate incompatible; adaptation must not form a borrow

argument explicitly share, parameter expects share
  => candidate may be compatible after ordinary borrow legality checks

argument automatic in, parameter expects copy
  => adapt to copy if legal

argument automatic in, parameter pass unspecified
  => use the ordinary instance/context default-pass judgment
```

This document does not define candidate ranking; it only states that pass
adaptation is part of candidate adaptation. The `RawArgShape` / `ParameterShape`
objects come from `pattern-normalization-and-first-order-overload.md`.

The hard boundary is `NoImplicitBorrowFormation`:

```text
candidate adaptation cannot rewrite T to T ref or T share
structural repair cannot insert ref, share, or @
default pass selection returns only move or copy
```

The fixed points and weakening of an already formed borrow remain ordinary
borrow-constructor rules; the callable-frame implicit `self` capability is a
separate, narrow rule and is not argument adaptation.

## 10. Relation to type values and rank

Pass mode is not part of `TypeValueId`. Type matching and pass matching are
separate concerns:

```text
type/value/rank compatibility:
  arg_type == parameter_type
  arg_rank == parameter_rank

pass compatibility:
  move/ref/share/copy/in adaptation
```

`T move` is not a new type. `T move == T` is a core principle, and
`rank move == rank` is a core principle. Two arguments that differ only by pass
mode have the same type value and the same rank.

## 11. Relation to IR

The IR must not retain `in`, and it must not retain an undecided default pass.
The final IR / lower-action layer sees only fully decided actions, for example:

```text
CopyConstruct x -> tmp
Move tmp -> arg_slot

Move explicit_share_handle -> arg_slot

Move explicit_ref_handle -> arg_slot

Move x -> arg_slot
```

If a source/meta layer produces a nested move, it must be canonicalized:

```text
move(move(x)) => move(x)
```

This fixed-point equation is canonical target semantics, not a description of
current implemented behavior. An IR may retain `CopyConstruct` as a compact
instruction only after recording which ordinary share/clone or rebind/clone
realization was selected; the instruction name does not erase the semantic
equivalence in `CopyConstructExpansion`.

## 12. Relation to later call modes

This layer is also a prerequisite for the future `normal` / `tco` / `loop` call
modes, but this document does not define call modes.

When `tco` actively moves arguments, what it moves are argument objects that have
*already* completed pass normalization. `loop` requires stronger slot
compatibility and may depend on whether an argument object is already reusable in
place. These are cross-references only; this document does not expand
ABI or tail-call checking.

## 13. Non-goals

```text
No parser syntax change.
No current normalizer behavior change.
Implementation scope: this document defines lowering obligations and does not
change the Rust implementation.
No full trait solver.
No full type checker.
No borrow checker implementation.
No access-tree construction implementation.
No ABI design.
No LLVM lowering.
No runtime overload implementation.
No final IR instruction format.
```

## 14. Relationship to other documents

The documents below own the adjacent relations consumed by this model.

- `pattern-normalization-and-first-order-overload.md` — produces the
  `RawArgShape` / `ParameterShape` objects that argument adaptation consumes.
  Pass adaptation is the mechanical-argument-passing step within or after
  candidate adaptation.
- `type-values-places-and-borrow-views.md` — defines `TypeValueId`; pass mode
  is explicitly not part of it, and `T move == T`.
- `type-associated-function-objects-and-access-trees.md` — field-function and
  access-tree work; an explicit `ref` / `share` produces a borrow object whose
  handle is moved while preserving parent/origin.
- `overload-resolution-design.md` — candidate matching must separate type/rank
  compatibility from pass compatibility.
- `meta-object-invocation-and-policy-reduction.md` — the invocation engine that
  ultimately receives fully decided pass actions.
- control-flow-local meta evaluation substrate (see
  [semantic evaluation](../meta-invocation/evaluation-residual-and-optimization.md)) — the guarded
  `T: has_pass` branch relies on the same substrate: if explicit pass is
  present, the default-pass branch is not entered and `get_default_pass` has no
  lookup or policy obligation.


## 15. Cleanup placement and with

Cleanup is placed before @ or other lifetime observation. Omitted with always
uses the ordinary NLL default; explicit empty with{} anchors cleanup at the
lexical boundary. Neither supplies missing access, borrow, capture, type or
construction authority.

```text
x with{a,b}  =>  x -> a and x -> b
Touch(x) = Use(x) ∪ Consume(x) ∪ Destroy(x)
UseForPlacement(a)
  = OrdinaryRequiredUses(a) ∪ ⋃{Touch(x) | x -> a}
```

Only actual continuation events belong to Touch. Each outgoing edge makes the
dependency a survive the dependent x's required touches. If both destruction
events exist, Destroy(x) < Destroy(a). Ordinary uses of a do not extend x.
Transitive y -> x -> a orders their existing destructors y before x before a.
Strict cycles have no legal cleanup placement and diagnose.

A killing move discharges the old generation's cleanup obligation at its
consumption cut. It adds no second Destroy/drop, even with lexical with{}.
Required uses of the transferred generation follow that generation's ordinary
relations. Observable destructors and effects cannot be removed to satisfy
the graph. NLL and lexical anchors are defaults/constraints within this same
placement relation, not an iterative lifetime repair algorithm.

The current Raw/Norm with carrier preserves items, explicit emptiness and
errors. It does not implement Touch, cleanup scheduling or lifetime checking.
