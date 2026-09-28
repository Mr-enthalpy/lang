# mechanical-lowering

**Status: Mixed. `CanonicalMechanicalPassCore` in
`mechanical-argument-passing-and-move-fixed-point.md` is canonical target
semantics, along with instance move effects and cleanup/with placement.
Consumer algorithms, return/call-mode design, normalizer/checker/IR
integration, ABI, optimizer, and runtime behavior remain non-normative and
unimplemented. This block is not a machine-ABI design.**

## Scope

The compiler-inserted mechanical action frameworks at call sites:

- terminal Pass=Move and its fixed point; copy-derived share/rebind -> clone -> Move
- automatic return normalization and `Error` / `noerror` policy
- `normal` / `tco` / `loop` call modes, with no loop core (repetition is
  recursion) and tail-position lowering on the first-order AST

The final IR receives fully decided actions. Argument passing has no unresolved
move/copy choice: every producer path ends in Move. Separate error-policy and
call-mode questions do not reopen this closed pass ontology. Cleanup points
are fixed before same-point reverse-declaration linearization and @ observation.

## Bool-protected guard rule

Logical, predicate, and guard expressions do not produce a naked `if | else`
pattern space. They produce a bool-protected control result:

```text
(if | else) bool
```

The bool construction supplies the ordinary condition Pattern. Residual
control material can remain inside its chain; a separate CanEscape consumer
checks forbidden boundary escape using ordinary meta facts. Pattern matching reads the bool
symbol's Pattern layer directly:

```text
cond |> if { ... } |> else { ... }
```

Explicit `cond?` is also valid when one top Pattern peel is desired, but `?` is
not required to begin extraction and is not a special conditional entrance.
Older examples in this block that spell `?` use that optional explicit view.

## For compile/meta construction work, read
`spec/contracts/meta-construction-boundary.md` first. Its ordinary instance boundary applies here: type/meta/Pattern material has
ordinary pass and lifecycle obligations. Stage and shape do not exempt it.
The lifetime owner supplies Killable, fixed MoveEffect and frontier Movable.

## Not in scope

Backend / machine ABI, machine stack layout, and the final IR instruction
format.

## Documents

- `mechanical-argument-passing-and-move-fixed-point.md` — canonical pass-action
  core, ordinary producer realizations, terminal Move and fixed cleanup ordering.
- `mechanical-return-normalization-and-error-policy.md` — return normalization,
  `Error` handler lookup, and `noerror`.
- `call-modes-recursion-and-tail-lowering.md` — `normal` / `tco` / `loop`.

## Reading order

Read in order: argument passing, then return normalization, then call modes.

## Dependencies

Consumes `RawArgShape` / `ParameterShape` from `patterns-overload/`, the layered
symbol policy from `symbol-world/`, orthogonal error-policy mapping from
`policy-capability/`, and the unified invocation from `meta-invocation/`.
