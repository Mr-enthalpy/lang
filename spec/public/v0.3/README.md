# v0.3 Normalized AST Specification — Workspace

## Stage status

v0.3 — Normalized AST Specification — is the current active stage.

v0.3 specifies the Normalized AST. It does not implement Raw AST → Normalized
AST lowering (that is v0.4).

Normalized AST is desugared, structurally regular, and suitable for later
semantic passes. Normalized AST is not HIR, not type-checked, and not
name-resolved.

## Current v0.3 specification workspace

- [`normalized-ast-specification-v0.3.md`](normalized-ast-specification-v0.3.md) — Normalized AST specification scaffold and work items.

## Frozen v0.2 frontend input authority

The v0.2 public frontend specification set remains authoritative for the Raw AST
input surface that v0.3 normalization consumes. These are not current-stage work
targets but frozen input contracts:

- [`../v0.2/lexical-syntax-v0.2.md`](../v0.2/lexical-syntax-v0.2.md) — frozen lexical syntax
- [`../v0.2/concrete-syntax-v0.2.md`](../v0.2/concrete-syntax-v0.2.md) — frozen concrete syntax
- [`../v0.2/diagnostics-recovery-v0.2.md`](../v0.2/diagnostics-recovery-v0.2.md) — frozen diagnostics and recovery
- [`../v0.2/raw-ast-frozen-surface-v0.2.md`](../v0.2/raw-ast-frozen-surface-v0.2.md) — frozen Raw AST surface inventory

## v0.3 handoff

- [`../../contracts/v0.3-normalization-handoff-checklist.md`](../../contracts/v0.3-normalization-handoff-checklist.md) — may-assume, must-not-assume, required inputs, normalization obligations.

## Open v0.3 design questions

- N-AST-1 through N-AST-8 documented in [`../../planning/open-questions.md`](../../planning/open-questions.md).

## Non-goals for v0.3

v0.3 is specification-only unless explicitly assigned otherwise.

v0.3 does not:
- implement Raw AST → Normalized AST lowering (v0.4)
- implement Normalized AST Rust types in the codebase (v0.4)
- create normalization dumps (v0.4)
- perform name resolution
- perform type checking
- perform operator lookup
- perform alias target resolution
- perform canonical matching
- materialize closures
- perform ownership / NLL / drop analysis
- interpret or execute programs
- generate code
