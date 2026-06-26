# v0.4 — Historical

v0.4 implemented and hardened the Raw AST → Normalized AST prototype: the
lowering loop, a stable normalized dump, a CLI normalized dump path, golden
tests, structural invariant tests, error recovery through normalization,
explicit `Unsupported` visibility, and value-side / pattern-side boundary
preservation.

This directory is historical. It does not define current public language
behavior. For current behavior, read `spec/public/v0.5/`.

## Where the v0.4 route and decisions live

- `spec/contracts/v0.4-normalization-prototype-notes.md` — the normative v0.4
  normalization boundary (what the prototype/hardening delivered and must not
  cross).
- `spec/public/v0.5/normalized-surface-semantics-v0.5.md` — the published public
  explanation of the behavior v0.4 produces.
- `tests/cases/norm/` and `tests/normalized_golden.rs` — the golden coverage,
  including the unsupported-audit cases that record where expression-like sugar
  in pattern / annotation context is surfaced as `PatternUnsupported` instead of
  crossing the value/pattern boundary.

## v0.4 hardening decisions (summary)

- Unsupported Raw AST subshapes remain visible (`Unsupported` /
  `PatternUnsupported`) rather than being silently erased.
- Value-side `NormExpr` and pattern-side `NormPattern` remain distinct; value to
  pattern requires an explicit bridge, pattern to value requires extraction.
- Operator and alias targets remain unresolved structural targets / `EntityRef`.
- No name resolution, type/kind checking, operator lookup, pattern-head
  resolution, or pattern-space construction occurs.

Route notes, prototype discussion, and superseded v0.4 planning material may be
expanded here in a later archaeology pass; physical document relocation is out of
scope for the v0.5 closeout.
