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

It is not current public language behavior or an implemented pass. Only
`CanonicalMechanicalPassCore` is normative target semantics; examples,
selection heuristics, and lowering/IR descriptions are implementation guidance.
The document is self-contained: it does not require the reader to assemble its
meaning from other documents.

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

A manual `move` always consumes the object. `Copyable` only guarantees that a
value *can* be copied; it does not permit optimizing an explicit `move` into a
`copy`, and it does not permit downgrading an explicit `move` into a `share` or
`ref`.

Manual pass modes are not free hints. They are semantic requirements, and future
work will introduce corresponding compile-time legality checks — for example,
explicit `copy` requires copyable, explicit `ref` requires an exclusive borrow,
explicit `share` requires a shared borrow, and explicit `move` requires that the
current object can be consumed. The detailed conditions are out of scope here;
the point is that a manual pass mode is a requirement, not a suggestion.

## 4. Default Pass Insertion

When no explicit pass mode is given, the lowering framework inserts a concrete
`move` or `copy` action selected from the value argument's first-order type and
static facts. It never inserts `ref`, `share`, or `@`. The action must be
explicit after lowering; the IR must not carry a deferred "default pass" state.

The inserted action can be described schematically in language-shaped form:

```lang
(arg: type)? |>
  if { arg; } |>
  else {
      arg |> <T: type>(self, arg: T) {
          (T: has_pass)? |>
              if {
                  arg;
              } |>
              else {
                  arg |> (T |> get_default_pass);
              };
      }
}
```

Semantic points:

1. `(arg: type)? |> if { arg; }` uses an optional one-layer top Pattern view.
   The guard `arg: type` produces a bool symbol whose Pattern carries the
   `if` / `else` alternatives; matching does not require `?`. Non-value
   material includes type objects, rank objects, namespace objects, meta
   objects, pattern objects, verification objects, and future manifest/package
   objects.

2. The `else` branch handles value arguments only.

3. `arg |> <T: type>(self, arg: T) { ... }` binds the generated helper's
   implicitly passed caller object (here the generated helper function object)
   to its first written formal `self`, then
   binds the explicit value argument `arg` and its first-order type `T`.

4. `(T: has_pass)? |> if { ... }` means: the guarded predicate produces a bool
   symbol and `?` explicitly peels one top Pattern layer. The branch could read
   the Pattern directly. If explicit
   pass is present, the lowering preserves `arg` and does not automatically
   rewrite it.

5. `arg |> (T |> get_default_pass)` means: when no explicit pass is present,
   obtain the default pass action from `T`'s default pass policy / static facts
   and insert that action into the argument slot.

6. This example describes the mechanical lowering framework. It does not
   implement a full trait solver, target ABI decision, borrow checker, copy
   legality checker, or concrete pass-selection algorithm.

`T |> get_default_pass` is a source-shaped placeholder for the future static
selection procedure. Its result domain is `move | copy`; it may depend on
`Copyable`, layout/size, target facts, and
policy. Those details are not the inserted action's surface shape. The inserted
argument action is still explicit after lowering; the IR must not receive an
undecided default pass.

Key properties:

- automatic default insertion applies only when no explicit pass action exists;
- the default action is not implicitly `move`;
- the default action must become a concrete pass action before IR/action lower;
- `Copyable` only guarantees copyability; it does not guarantee that the default
  copies;
- a large `Copyable` object may still move rather than copy under default policy;
- a small but non-copyable object is not copied merely because it is small;
- no default policy forms a `ref` or `share` view;
- if no selected pass action is viable, a later checking stage should report an
  error; this document does not define the full error conditions.

## 5. Non-Value Arguments Pass Through Unchanged

Automatic pass insertion applies only to value arguments:

```text
non-value argument material
  -> pass unchanged

value argument material
  -> bind first-order type T
  -> preserve explicit pass if present
  -> otherwise insert T |> get_default_pass
```

Non-value arguments include, but are not limited to, type objects, rank objects,
namespace objects, meta objects, pattern objects, verification objects, and
future manifest/package objects. These must not receive an automatically
inserted `copy`, `share`, `ref`, or `move`.

This rule prevents ordinary meta/type/pattern material from being mistaken for a
runtime value at an argument slot.

## 6. Move is the fixed point

This is the core of the document. The central axiom of pass normalization:

```text
T move == T
rank move == rank
move(move(x)) == move(x)
```

`move` is not a type constructor. It does not produce a new type value such as
`T move`, it does not change rank, and it does not change a classifier. It is the
consuming transfer of an object's resource / handle from its location into the
argument slot.

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
objects of types `T ref` and `T share`, whose candidates in the same associated `()` Symbol may be
distinct.

## 7. All pass modes lead to move

There are four mechanical modes, defined in terms of `move`:

```text
move(x):
  consume x
  transfer x into argument slot

copy(x):
  tmp = CopyConstruct(x)
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

- `copy(x)` consumes `tmp`, not `x`.
- `ref(x)` / `share(x)` consume the borrow handle `b`, not `x`.
- `move(x)` consumes `x` itself.
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

## 8. Borrow movement preserves parent/origin

Moving a borrow handle does not create a deeper borrow chain.

```text
move(borrow_node(parent = p, kind = k))
  = borrow_node(parent = p, kind = k)
```

The equality here is a fixed point on type / rank / access shape. It does not
claim that the same runtime handle has no linear state change: the old handle
dies, and a new handle inherits the same parent/origin/kind.

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
  detect is_val
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
expectation participates in candidate compatibility. Automatic `in` is only a
pre-lowering placeholder, never a canonical automatic action. It appears only
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
  => use default in(T)
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
current implemented behavior.

## 12. Relation to later call modes

This layer is also a prerequisite for the future `normal` / `tco` / `loop` call
modes, but this document does not define call modes.

When `tco` actively moves arguments, what it moves are argument objects that have
*already* completed pass normalization. `loop` requires stronger slot
compatibility and may depend on whether an argument object is already reusable in
place. These are cross-reference placeholders only; this document does not expand
ABI or tail-call checking.

## 13. Non-goals

```text
No parser syntax change.
No current normalizer behavior change.
No Rust implementation change in this PR.
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

The documents below are adjacent design. They do not define the mechanical
passing model specified here, and this document does not depend on them for its
meaning.

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
  `static-pattern-spaces-and-extraction-chains.md`§17) — the guarded
  `T: has_pass` branch relies on the same substrate: if explicit pass is
  present, the default-pass branch is not entered and `get_default_pass` has no
  lookup or policy obligation.
