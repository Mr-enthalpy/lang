# v0.5 — Normalized Surface Semantics Stabilization and Public Documentation Reset

## Stage status

v0.5 is the current active stage. It stabilizes the normalized surface semantics
that the v0.4 normalizer already produces, and resets the public documentation
structure so that current language behavior is explained in one place.

v0.5 is still **non-semantic** in the later-compiler sense. It does not implement
type checking, name resolution, operator lookup, pattern-head resolution, HIR,
closure materialization, runtime evaluation, or code generation.

## What v0.5 public docs explain

The v0.5 public documents explain the currently stabilized normalized surface
behavior, especially:

- source-product continuation and call binding;
- product / group / target boundaries;
- operator / member / bracket sugar as normalization-level lowering;
- value-side vs pattern-side separation;
- annotation patterns and DeduceList holes;
- origin / provenance and error / `Unsupported` visibility at the normalized
  layer;
- what Normalized AST explicitly does **not** do.

## v0.5 public documents

- [`normalized-surface-semantics-v0.5.md`](normalized-surface-semantics-v0.5.md)
  — the public, authoritative explanation of the normalized surface (published).
  §1–§7 define call / product / pipe binding; §8–§10 define value-side /
  pattern-side / annotation / alias boundaries; §11 defines origin / generated /
  derived / unsupported visibility; §12–§13 define non-goals and the v0.6+
  future boundary.
- [`agent-interpretation-guide-v0.5.md`](agent-interpretation-guide-v0.5.md)
  — operational, normative guidance for coding/documentation agents on how to
  interpret source without importing C / Rust / Python call assumptions.

## Authority

For the documentation authority hierarchy (public vs contracts vs history vs
future vs planning), see [`../../README.md`](../../README.md). Public docs define
current behavior; future design docs do not.

The v0.4 normalization boundary that v0.5 stabilizes is recorded in
[`../../contracts/v0.4-normalization-prototype-notes.md`](../../contracts/v0.4-normalization-prototype-notes.md).

## Future boundary

Later pattern-space and extraction-chain semantics
([`../../future/static-pattern-spaces-and-extraction-chains.md`](../../future/static-pattern-spaces-and-extraction-chains.md))
motivate the current normalized boundaries but are **not** implemented by the
v0.5 normalizer. `Done`, residual propagation, pattern-space subtraction,
`operator+` meta-reduction, `match` closing, and pattern-head resolution are not
current behavior.
