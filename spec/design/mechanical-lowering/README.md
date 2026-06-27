# mechanical-lowering

**Status: Non-normative future design. Not implemented as current normalizer,
IR, ABI, optimizer, or runtime behavior. This block is not a machine-ABI
design.**

## Scope

The compiler-inserted mechanical action frameworks at call sites:

- automatic argument passing and the `move` fixed point (`T move == T`)
- automatic return normalization and `Error` / `noerror` policy
- `normal` / `tco` / `loop` call modes, with no loop core (repetition is
  recursion) and tail-position lowering on the first-order AST

A recurring invariant across this block: **default strategies (`in`, default
error propagation, automatic call-mode selection) exist only before lowering /
meta invocation. The final IR receives only fully decided actions.**

## Not in scope

Backend / machine ABI, machine stack layout, and the final IR instruction
format.

## Documents

- `mechanical-argument-passing-and-move-fixed-point.md` — pass insertion and the
  `move` fixed point.
- `mechanical-return-normalization-and-error-policy.md` — return normalization,
  `Error` handler lookup, and `noerror`.
- `call-modes-recursion-and-tail-lowering.md` — `normal` / `tco` / `loop`.

## Reading order

Read in order: argument passing, then return normalization, then call modes.

## Dependencies

Consumes `RawArgShape` / `ParameterShape` from `patterns-overload/`, the `Error`
policy planes from `policy-capability/`, and reuses the unified invocation from
`meta-invocation/`.
