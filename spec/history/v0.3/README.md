# v0.3 — Historical

v0.3 was the Normalized AST Specification stage. It defined the desugared,
non-semantic Normalized AST shape: the source-product continuation call skeleton
and the minimum Normalized AST shape that v0.4 then implemented.

This directory is historical. It does not define current public language
behavior. For current behavior, read `spec/public/v0.5/`.

## Where the v0.3 route and decisions live

- `spec/public/v0.3/normalized-ast-specification-v0.3.md` — the completed v0.3
  Normalized AST specification baseline (§7 source-product continuation call
  skeleton; §8 minimum Normalized AST shape).
- `spec/contracts/v0.3-normalization-handoff-checklist.md` — the v0.3 handoff
  checklist (may-assume / must-not-assume / required inputs); a handoff-time
  snapshot.
- `spec/planning/open-questions.md` — the `N-AST-1..9` design questions and their
  resolution / audit trail, now resolved by the published v0.5 public docs.

## v0.3 resolved design boundary (summary)

- Call binding centers on source-product continuation
  (`Product1 |> e Product2 => (Product1, Product2) |> e`), with two legality
  repairs; a following Product is continuation, not an argument list.
- Sugar (operator / prefix-negative / member / double-dot / bracket) lowers to
  the product-call skeleton with unresolved targets; no lookup or dispatch.
- Value is not Pattern; annotation is pattern-side; alias RHS stays `EntityRef`.
- No general symbolic builtin node family; generated material carries
  origin / provenance.

The published public form of these decisions is
`spec/public/v0.5/normalized-surface-semantics-v0.5.md`. Physical relocation of
the baseline specification is out of scope for the v0.5 closeout.
