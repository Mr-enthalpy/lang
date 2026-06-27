# meta-invocation

**Status: Non-normative future design. Not implemented as a full invocation
engine.**

## Scope

The policy-governed meta object invocation model:

- the dual judgment of symbol lookup vs callable execution
- partial meta reduction vs strict meta execution
- residualization at runtime-only boundaries
- guarded invocation strategies
- control-like constructs (`cond`, `&&`, `||`, `==`, `!=`) as ordinary
  meta-callables — not an `if constexpr` / `if` syntax split

## Not in scope

This block references, and does not redefine, the symbol world, the
pattern/overload candidate model, and the policy planes. It consumes the
applicable candidate set; it does not specify how that set is built.

## Documents

- `meta-object-invocation-and-policy-reduction.md` — the formal invocation model.

## Reading order

Read `meta-object-invocation-and-policy-reduction.md`.

## Dependencies

References `symbol-world/` (lookup), `patterns-overload/` (candidate
preparation), and `policy-capability/` (visibility / body-entry / return-object
policy). The mechanical-lowering family feeds it fully decided pass/return
actions.
