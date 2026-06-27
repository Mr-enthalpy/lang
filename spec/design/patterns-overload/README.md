# patterns-overload

**Status: Non-normative future design. Not implemented as a current pattern
matcher, type checker, or overload resolver.**

## Scope

Pattern normalization and the candidate model that feeds invocation:

- `PatternObject` and occurrence roles (binder / type / path / literal / discard)
- `RawArgShape` and `ParameterShape`
- first-order type-value candidate adaptation, applicability, specificity
- the full overload-resolution vision
- static pattern spaces and extraction chains

This block distinguishes two layers explicitly:

- the **earlier, narrower** candidate-preparation subset that serves formal meta
  object invocation (pattern normalization + first-order type candidate shapes);
- the **later, fuller** runtime overload resolution and pattern-space /
  extraction-chain semantics, which remain further out.

## Not in scope

Runtime overload resolution implementation, full pattern-space algebra, and
match/exhaustiveness checking.

## Documents

- `pattern-normalization-and-first-order-overload.md` — the earlier
  candidate-preparation subset.
- `overload-resolution-design.md` — the broader, later full overload-resolution
  vision.
- `static-pattern-spaces-and-extraction-chains.md` — the later pattern-space /
  extraction-chain semantics.

## Dependencies

Uses `TypeValueId` from `symbol-world/`. Produces the applicable candidate set
consumed by `meta-invocation/`. Pass-mode adaptation is in
`mechanical-lowering/` and is separate from type/rank compatibility.
