# Specification Index

This directory contains the specification documents for the `lang` language
frontend. Documents are organized by role rather than in a flat list. The
current active stage is v0.3 — Normalized AST Specification.

## Public current specification: v0.3

**`spec/public/v0.3/`** — The current active specification stage. v0.3
specifies the Normalized AST. Specification-only; implementation is v0.4.

| File | Authority | Role |
|---|---|---|
| `README.md` | Stage workspace index | Entry point for v0.3 Normalized AST Specification work. |
| `normalized-ast-specification-v0.3.md` | v0.3 specification scaffold | Defines the problem space, non-semantic boundary, active design questions, and specification work items. Does not define the final node set. |

## Frozen v0.2 frontend input authority

**`spec/public/v0.2/`** — Frozen frontend input contract. v0.2 is closed but
remains authoritative for the Raw AST input surface that v0.3 normalization
consumes.

| File | Authority | Role |
|---|---|---|
| `lexical-syntax-v0.2.md` | Normative for public lexical syntax | Defines source normalization, lexical categories, token spellings, comments, literals, invalid lexical material, and non-semantic lexer boundaries for v0.2. |
| `concrete-syntax-v0.2.md` | Normative for public concrete syntax | Defines the accepted non-semantic source-level grammar, parser shape, Raw AST preservation boundaries, and parser-level non-semantic constraints for v0.2. |
| `diagnostics-recovery-v0.2.md` | Normative for public frontend diagnostics and recovery | Defines v0.2 lexical/parser diagnostic codes, trigger conditions, span policy, recovery behavior, ErrorAst relation, diagnostic stability, and non-semantic diagnostic boundaries. |
| `raw-ast-frozen-surface-v0.2.md` | Normative frozen surface inventory | Enumerates frozen Raw AST constructs with guarantees, non-semantic boundaries, v0.3 obligations, and forbidden assumptions. |

## Global references

**`spec/reference/`** — Cross-cutting references used across all tiers.

| File | Authority | Role |
|---|---|---|
| `glossary.md` | Normative for terminology | Resolves naming ambiguity across all documents. |

## Implementation backing

**`spec/implementation/v0.1/`** — Implementation backing documents. Read
these only for parser implementation repair, diagnostic implementation
repair, factual inventory checks, or archaeology.

| File | Authority | Role |
|---|---|---|
| `ast-construction-v0.1.md` | Normative for parser implementation behavior | Defines every syntax rule, AST shape, and parser constraint. Implementation-level backing reference. |
| `diagnostics-v0.1.md` | Normative for diagnostic implementation behavior | Defines diagnostic categories, span policy, and recovery behavior. Implementation-level reference. |
| `implementation-status-v0.1.md` | Authoritative factual inventory | Records the current implementation status of every feature. Does not define parser rules. |

## Contract and handoff documents

**`spec/contracts/`** — Raw AST contract and v0.3 handoff boundary documents.
Read these for normalization-boundary work and v0.3 preparation, not for
ordinary syntax understanding.

| File | Authority | Role |
|---|---|---|
| `raw-ast-contract-v0.1.md` | Normative contract for future normalization | Defines Raw AST invariants that future normalization passes may rely on. |
| `raw-ast-contract-freeze-v0.2.md` | Normative for v0.2 contract freeze | Defines v0.2 freeze boundary, allowed work, forbidden work, and handoff requirements for v0.3. |
| `v0.3-normalization-handoff-checklist.md` | Normative for v0.3 handoff readiness; non-normative for final Normalized AST design | Checklist of may-assume, must-not-assume, required input families, diagnostic/recovery inputs, normalization obligations, and open v0.3 questions. |

## Historical design notes

**`spec/history/v0.1/`** — Historical design and resolved-decision documents.
These remain available but are not the normal public entry point.

| File | Authority | Role |
|---|---|---|
| `frontend-v0.1.md` | Non-normative overview | Historical reader entry point. Describes the v0.1 pipeline, document division, and the boundaries between tokens, AST, and diagnostics. |
| `operator-design.md` | Normative for operator syntax design | Defines operator identity, spellings, fixity, precedence, associativity, AST sugar shape, lookup boundaries, and implementation boundary. Historical reference. |
| `resolved-questions.md` | Authoritative for resolved decisions | Records design questions resolved in v0.1. |

## Future design notes

**`spec/future/`** — Forward-looking design notes. These are not current
syntax specifications.

| File | Authority | Role |
|---|---|---|
| `entity-ref-design.md` | Non-normative future design note | General `EntityRef` design (future). Alias-RHS `EntityRef` subset is implemented in Phase 4.4. |
| `entity-alias-design.md` | Implemented-design explanation | Documents lexical alias binding syntax (`let binder === EntityRef`). Phase 4.3 design; Phase 4.4 raw parser preservation implemented. Future semantic meaning remains future work. |
| `library-namespace-design-note.md` | Non-normative future design note | Describes the intended library/namespace/import model. |
| `build-system-design.md` | Non-normative, future design | Formal design note for the build/package/namespace assembly architecture. |
| `package-manifest-v0.md` | Non-normative, future design | Provisional build-manifest design surface. |
| `namespace-assembly-v0.md` | Non-normative, future design | High-level namespace assembly pipeline and phase split. |

## Planning and debt

**`spec/planning/`** — Roadmap and unresolved debt. Planning references,
not syntax specifications.

| File | Authority | Role |
|---|---|---|
| `roadmap.md` | Authoritative for scope and planning; non-normative for parser behavior | Defines stage boundaries (v0.1–v0.11) and what must not leak between stages. |
| `open-questions.md` | Non-normative | Tracks unresolved design questions and documentation debt. |

## Reading order

### Current v0.3 specification work

Start here for current-stage v0.3 Normalized AST Specification:

1. `public/v0.3/README.md` - v0.3 workspace index.
2. `public/v0.3/normalized-ast-specification-v0.3.md` - Normalized AST specification scaffold.
3. `contracts/v0.3-normalization-handoff-checklist.md` - v0.3 may-assume, must-not-assume, required inputs.
4. `planning/open-questions.md` - Open v0.3 design questions (N-AST-1 through N-AST-8).

### Frozen v0.2 frontend input

Read these for the frozen Raw AST input surface:

1. `spec/public/v0.2/lexical-syntax-v0.2.md` - Understand the public lexical syntax.
2. `spec/public/v0.2/concrete-syntax-v0.2.md` - Understand the public concrete syntax.
3. `spec/public/v0.2/diagnostics-recovery-v0.2.md` - Understand public diagnostics and recovery.
4. `spec/public/v0.2/raw-ast-frozen-surface-v0.2.md` - Inspect the frozen Raw AST construct inventory.
5. `spec/reference/glossary.md` - Resolve terminology ambiguity.

### Extended implementer reading order

Read these only when implementing, auditing, or repairing the frontend.

1. `spec/implementation/v0.1/ast-construction-v0.1.md` - Implement the parser.
2. `spec/implementation/v0.1/diagnostics-v0.1.md` - Diagnostic catalog (implementation-level reference).
3. `spec/implementation/v0.1/implementation-status-v0.1.md` - Know current implementation facts.
4. `spec/contracts/raw-ast-contract-v0.1.md` - Know Raw AST invariants for normalization.
5. `spec/contracts/raw-ast-contract-freeze-v0.2.md` - Know v0.2 freeze boundary and v0.3 handoff.
6. `spec/history/v0.1/operator-design.md` - Understand operator syntax rules.
7. `spec/history/v0.1/resolved-questions.md` - Understand resolved design decisions.

### Future-design reading order

Read these only when working on future design topics.

1. `spec/future/entity-alias-design.md` - Understand alias binding syntax (implemented) and future semantics.
2. `spec/future/entity-ref-design.md` - Understand future general EntityRef design.
3. `spec/future/library-namespace-design-note.md` - Understand library/namespace model.
4. `spec/future/build-system-design.md` - Understand build/package architecture.
5. `spec/future/package-manifest-v0.md` - Understand build-manifest surface.
6. `spec/future/namespace-assembly-v0.md` - Understand namespace assembly pipeline.
7. `spec/planning/roadmap.md` - Understand scope boundaries.
8. `spec/planning/open-questions.md` - Recognize known gaps.

## Spec priority

For public v0.2 lexical syntax, concrete syntax, diagnostics, and recovery,
the documents under `spec/public/v0.2/` are the reader-facing authority.

The implementation and golden snapshots remain the factual behavior source.

Documents under `spec/implementation/`, `spec/contracts/`, `spec/history/`,
`spec/future/`, and `spec/planning/` remain available for backing reference,
archaeology, future design, and scope management. They are not the normal
public syntax entry point.

If a public v0.2 spec conflicts with an older moved document, treat the
conflict as documentation debt to repair. Do not use the older document to
silently reinterpret the public v0.2 syntax surface.
