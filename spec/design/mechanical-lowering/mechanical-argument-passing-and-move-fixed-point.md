# Mechanical Argument Passing and the Move Fixed Point

**Status: Non-normative future design. Not implemented as current parser, normalizer, type checker, borrow checker, ABI, or IR lowering behavior.**

This document specifies the future *mechanical argument passing* layer: how a
call argument is normalized into a concrete pass action (move, borrow, or copy)
before a function or meta-function body receives it. Its central claim is that
pass modes are mechanically inserted, source-expressible actions — not backend
ABI heuristics and not optimizer decisions — and that `move` is the fixed point
of pass normalization.

It is a future design note. It is not current public language behavior, not an
implemented pass, not a parser or normalizer rule, and not an IR/ABI rule. The
document is self-contained: it does not require the reader to assemble its
meaning from other documents.

## 1. Purpose

This document defines the mechanical argument-passing normalization that happens
at a call's argument slots. The problem it solves:

```text
source argument
  -> raw argument shape
  -> explicit pass extraction
  -> automatic pass selection, if no explicit pass
  -> concrete pass action
  -> eventual IR/action layer receives fully decided movement/borrow/copy actions
```

The compiler's only privilege is to fix the *insertion framework* during
normalization / lowering: it mechanically inserts a pass action at each value
argument slot. The actual action that gets inserted is still decided by
in-language facts — types, traits, policy, meta-functions, and symbol lookup —
not by an opaque backend convention.

This document does **not** define a full type checker, a full borrow checker, an
ABI, LLVM lowering, runtime overload resolution, or a full trait solver. It
defines a future semantic direction only.

## 2. Pass modes are mechanical source-level lowering, not ABI heuristics

`move`, `ref`, `share`, `copy`, and `in` are not backend heuristics and not
calling-convention choices. At the language level they are visible, checkable
actions that meta-code can describe and that legality checks can inspect.

The compiler may mechanically insert a default pass action, but once inserted it
must become an ordinary semantic object/action. The IR must not carry a
"default pass undecided" state.

```text
default only exists before lowering
IR must not receive `in`
IR receives concrete actions:
  Move
  CopyConstruct + Move
  MakeRefBorrow + Move
  MakeShareBorrow + Move
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
pass action selected from the value argument's first-order type and static facts.
The action must be explicit after lowering; the IR must not carry a deferred
"default pass" state.

The inserted action can be described schematically in language-shaped form:

```lang
arg: type |>
  if { arg; } |>
  else {
      arg |> <T: type>(arg: T) {
          T: has_pass |>
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

1. `arg: type |> if { arg; }` means: non-value argument material passes through
   unchanged. Non-value material includes type objects, rank objects, namespace
   objects, meta objects, pattern objects, verification objects, and future
   manifest/package objects.

2. The `else` branch handles value arguments only.

3. `arg |> <T: type>(arg: T) { ... }` binds the first-order type `T` of value
   argument `arg`.

4. `T: has_pass` stands for the guarded predicate that the argument already
   carries an explicit pass action. If so, the lowering preserves `arg` and does
   not automatically rewrite it.

5. `arg |> (T |> get_default_pass)` means: when no explicit pass is present,
   obtain the default pass action from `T`'s default pass policy / static facts
   and insert that action into the argument slot.

6. This example describes the mechanical lowering framework. It does not
   implement a full trait solver, target ABI decision, borrow checker, copy
   legality checker, or concrete pass-selection algorithm.

`T |> get_default_pass` is a source-shaped placeholder for the future static
selection procedure. It may depend on `Copyable`, layout/size, target facts, and
policy. Those details are not the inserted action's surface shape. The inserted
argument action is still explicit after lowering; the IR must not receive an
undecided default pass.

Key properties:

- automatic default insertion applies only when no explicit pass action exists;
- the default action is not implicitly `move`;
- the default action must become a concrete pass action before IR/action lower;
- `Copyable` only guarantees copyability; it does not guarantee that the default
  copies;
- a large `Copyable` object may still pass by `share` under default policy;
- a small but non-copyable object is not copied merely because it is small;
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

## 7. All pass modes lead to move

There are four mechanical modes, defined in terms of `move`:

```text
move(x):
  consume x
  transfer x into argument slot

copy(x):
  tmp = copy_construct(x)
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
- Every pass mode ultimately becomes a single terminal `move` action.

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
  choose concrete pass action if no explicit pass
```

As judgments:

```text
Γ ⊢ arg ⇓ RawArgShape

Γ ⊢ ParameterShape × RawArgShape ⇓ AdaptedArgShape

Γ ⊢ AdaptedArgShape ⇓ concrete pass action
```

An explicit argument pass dominates automatic pass. A parameter's pass
expectation participates in candidate compatibility. Automatic `in` is used only
when there is no explicit argument pass and a concrete pass action must still be
formed.

Conflict and adaptation examples:

```text
argument explicitly move, parameter expects share
  => candidate incompatible

argument explicitly copy, parameter expects move
  => candidate incompatible

argument automatic in, parameter expects share
  => adapt to share if legal

argument automatic in, parameter expects copy
  => adapt to copy if legal

argument automatic in, parameter pass unspecified
  => use default in(T)
```

This document does not define candidate ranking; it only states that pass
adaptation is part of candidate adaptation. The `RawArgShape` / `ParameterShape`
objects come from `pattern-normalization-and-first-order-overload.md`.

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

MakeShareBorrow x -> b
Move b -> arg_slot

MakeRefBorrow x -> b
Move b -> arg_slot

Move x -> arg_slot
```

If a source/meta layer produces a nested move, it must be canonicalized:

```text
move(move(x)) => move(x)
```

This is a future design requirement, not a description of current behavior.

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
- `type-values-places-and-alias-forwarding.md` — defines `TypeValueId`; pass mode
  is explicitly not part of it, and `T move == T`.
- `type-associated-function-objects-and-access-trees.md` — field-function and
  access-tree work; automatic `ref` / `share` produces a borrow object whose
  handle is moved while preserving parent/origin.
- `overload-resolution-design.md` — candidate matching must separate type/rank
  compatibility from pass compatibility.
- `meta-object-invocation-and-policy-reduction.md` — the invocation engine that
  ultimately receives fully decided pass actions.
