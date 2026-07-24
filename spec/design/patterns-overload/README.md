# patterns-overload

**Status: Non-normative future design. Not implemented as a current pattern
matcher, type checker, or overload resolver.**

## Scope

Pattern normalization and the candidate model that feeds invocation:

- `PatternObject` and occurrence roles (binder / type / path / literal / discard)
- `ProductObject` / `ArgProductShape` as the bridge from normalized products to
  argument-shape formation
- `RawArgShape` and `ParameterShape`
- first-order type-value candidate adaptation, applicability, specificity
- the full overload-resolution vision
- static pattern spaces and extraction chains
- callable implementation tails, first-class dot closures, and Pattern
  remainder packs
- in-place closure candidate metadata and its fixed preference position; lazy
  embedding lookup/capture boundaries are owned by the function-object model

Resolved pattern owners, `struct` / functional `inject`, and the rule that fully
named direct-child layers normalize to `Set<PatternValue>` while any bare child
makes the whole layer positional are owned by
`../symbol-world/symbol-first-meta-construction-and-pattern-injection.md`.
Canonical policy pairs, seal visibility, const/mut product order, compile-flow
projection, derived compile companions, must-select semantics, match staging,
and automatic require are owned by
`../symbol-world/symbol-policy-and-compile-flow-projection.md`.

This block distinguishes two layers explicitly:

- the **earlier, narrower** candidate-preparation subset that serves formal meta
  object invocation (pattern normalization + first-order type candidate shapes);
- the **later, fuller** runtime overload resolution and pattern-space /
  extraction-chain semantics, which remain further out.

## Not in scope

Runtime overload resolution implementation, full pattern-space algebra, and
match/exhaustiveness checking. Lifetime checking/refinement is later than this
type/compile pipeline and is bounded separately in `../lifetime/`.

## Product semantic normalization bridge

Product semantic normalization is not surface normalization. Before
`RawArgShape` formation, a normalized product must pass through:

```text
NormProduct -> ProductObject -> FlattenedProductObject -> ArgProductShape -> RawArgShape
```

The bridge flattens exposed Product nodes, does not cross Expression nodes,
preserves order, preserves `Unit`, and preserves provenance. This is an input to
candidate preparation, not runtime overload resolution.

## Documents

- `pattern-normalization-and-first-order-overload.md` — the earlier
  candidate-preparation subset.
- `overload-resolution-design.md` — the broader, later full overload-resolution
  vision, including fully admissible set `A`, ordered preference filters
  (including in-place over non-in-place), and must-select consistency.
- `static-pattern-spaces-and-extraction-chains.md` — the later pattern-space /
  extraction-chain semantics.
- `callable-tail-dot-closure-and-pack-pattern.md` — the canonical connection
  between callable implementation/strategy tails, `.name`, and `...args`.

## Reading order

Read `pattern-normalization-and-first-order-overload.md` first (the earlier
candidate-preparation subset), then the broader overload and pattern-space
documents.

## Dependencies

Uses `TypeValueId` from `symbol-world/`. Produces the applicable candidate set
consumed by `meta-invocation/`. Pass-mode adaptation is in
`mechanical-lowering/` and is separate from type/rank compatibility.
