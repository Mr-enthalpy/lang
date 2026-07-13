# meta-invocation

**Status: Non-normative future design. Not implemented as a full invocation
engine.**

## Scope

The policy-governed meta object invocation model:

- symbol-first heterogeneous value-facet candidate preparation
- orthogonal execution capability / evaluation demand / result rank
- canonical `MetaInstanceScope`, return type self-root validation, and complete
  invocation navigation atom
- the dual judgment of symbol lookup vs callable execution
- callable `P1` visibility and `P2` qualification handoff
- partial meta reduction vs strict meta execution
- residualization at runtime-only boundaries
- unified pattern-match staging rather than `if constexpr` / `if` families

## Not in scope

This block references, and does not redefine, the symbol world, the
pattern/overload candidate model, and layered policy. It consumes the
applicable candidate set; it does not specify how that set is built.

## Documents

- `meta-object-invocation-and-policy-reduction.md` — the formal invocation model.

## Reading order

Read
`../symbol-world/symbol-first-meta-construction-and-pattern-injection.md` for
the canonical SymbolCell, `compile` / `meta`, result-rank, pattern-owner,
`struct`, `inject`, and meta type self-root boundary. Read
`../symbol-world/symbol-construction-units-and-namespace-origin.md` for the
`MetaConstructionUnit` transaction and namespace ownership boundary. Then read
`../symbol-world/symbol-policy-and-compile-flow-projection.md` for `P1` / `P2`,
compile companions, match staging, and automatic require. Finally read
`meta-object-invocation-and-policy-reduction.md` for invocation demand and
policy reduction.

## Dependencies

References `symbol-world/` (symbol facets, construction, lookup),
`patterns-overload/` (candidate preparation and selection), and
`policy-capability/` (current metadata mapping and orthogonal policy). The
mechanical-lowering family feeds it fully decided pass/return actions.
