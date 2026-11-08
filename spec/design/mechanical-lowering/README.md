# mechanical-lowering

**Status: Mixed. `CanonicalMechanicalPassCore` in
`mechanical-argument-passing-and-move-fixed-point.md` is canonical target
semantics, along with instance move effects and cleanup/with placement.
Consumer algorithms, return/call-mode design, normalizer/checker/IR
integration, physical ABI, optimizer, and runtime behavior remain non-normative and
unimplemented. This block is not a machine-ABI design.**

[E](../static-evaluation/evaluation-residual-and-optimization.md#10-e-selected-return-delivery-convention)
owns the canonical FreshLet/ExistingTarget ABIKind selection. Mechanical work
consumes its fixed delivery and lifecycle obligations; machine realization is deferred.

## Scope

Mechanical passing and checked lowering consume actual program actions:

- terminal Pass=Move and its fixed point; copy-derived share/rebind -> clone -> Move
- explicit optional ordinary result adapters; Error-shaped data alone has no effect
- `normal` / `tco` / `loop` call modes, with no loop core (repetition is
  recursion) and tail-position lowering on the first-order AST

The final IR receives fully decided actions. Argument passing has no unresolved
move/copy choice: every producer path ends in Move. Optional library adapters
and call-mode research do not reopen this closed pass ontology. Cleanup points
are fixed before same-point reverse-declaration linearization and @ observation.

## Bool-protected guard rule

Logical, predicate, and guard expressions do not produce a naked `true | false`
pattern space. They produce a bool-protected control result:

```text
(true | false) bool
```

The bool construction supplies the ordinary condition Pattern. Residual
control material can remain inside its chain; a separate CanEscape consumer
checks forbidden boundary escape using ordinary compile facts. Pattern matching reads the bool
symbol's Pattern layer directly:

```text
cond |> if { ... } |> else { ... }
```

Explicit `cond?` is also valid when one top Pattern peel is desired, but `?` is
not required to begin extraction and is not a special conditional entrance.
Examples that spell `?` use that optional explicit view.

## Compile construction boundary
`spec/contracts/compile-construction-boundary.md` first. Its ordinary instance boundary applies here: type/compile/Pattern material has
ordinary pass and lifecycle obligations. Stage and shape do not exempt it.
The lifetime owner supplies Killable, fixed MoveEffect and frontier Movable.

## Not in scope

Backend / machine ABI, machine stack layout, and the final IR instruction
format.

## Documents

- `mechanical-argument-passing-and-move-fixed-point.md` — canonical pass-action
  core, ordinary producer realizations, terminal Move and fixed cleanup ordering.
- [Explicit result adapters](mechanical-return-normalization-and-error-policy.md) —
  optional library calls, with no compiler-inserted default error propagation.
- `call-modes-recursion-and-tail-lowering.md` — `normal` / `tco` / `loop`.

## Reading order

Read argument passing and E's canonical delivery/CPS owner first, then call-mode
research and optional explicit result adapters.

## Dependencies

Consumes `RawArgShape` / `ParameterShape` from `patterns-overload/`, the layered
symbol Policy from `symbol-world/` and unified invocation/CPS from
`static-evaluation/`. No ambient error-return capability is introduced.
