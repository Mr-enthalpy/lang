# Specification Index

This directory contains the specification documents for the `lang` language
frontend. Each document has a defined authority level.

## Document list

### Public v0.2 specification set

| File                               | Authority                                             | Role                                                                                                                                        |
| ---------------------------------- | ----------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `lexical-syntax-v0.2.md`           | Normative for public lexical syntax                   | Defines source normalization, lexical categories, token spellings, comments, literals, invalid lexical material, and non-semantic lexer boundaries for v0.2. |
| `concrete-syntax-v0.2.md`          | Normative for public concrete syntax                  | Defines the accepted non-semantic source-level grammar, parser shape, Raw AST preservation boundaries, and parser-level non-semantic constraints for v0.2. |
| `diagnostics-recovery-v0.2.md`    | Normative for public frontend diagnostics and recovery | Defines v0.2 lexical/parser diagnostic codes, trigger conditions, span policy, recovery behavior, ErrorAst relation, diagnostic stability, and non-semantic diagnostic boundaries. |
| `raw-ast-frozen-surface-v0.2.md`   | Normative frozen surface inventory                    | Enumerates frozen Raw AST constructs with guarantees, non-semantic boundaries, v0.3 obligations, and forbidden assumptions.                    |
| `glossary.md`                      | Normative for terminology                             | Resolves naming ambiguity across all documents.                                                                                             |

### Backing / historical / future-design references

| File                               | Authority                                             | Role                                                                                                                                        |
| ---------------------------------- | ----------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `ast-construction-v0.1.md`         | Normative for parser implementation behavior          | Defines every syntax rule, AST shape, and parser constraint. Backing reference for implementation repair.                                   |
| `diagnostics-v0.1.md`              | Normative for diagnostic implementation behavior      | Defines diagnostic categories, span policy, and recovery behavior. Implementation-level reference.                                          |
| `implementation-status-v0.1.md`    | Authoritative factual inventory                      | Records the current implementation status of every feature. Does not define parser rules.                                                   |
| `raw-ast-contract-v0.1.md`         | Normative contract for future normalization          | Defines Raw AST invariants that future normalization passes may rely on.                                                                     |
| `raw-ast-contract-freeze-v0.2.md`  | Normative for v0.2 contract freeze                    | Defines v0.2 freeze boundary, allowed work, forbidden work, and handoff requirements for v0.3.                                                |
| `operator-design.md`               | Normative for operator syntax design                   | Defines operator identity, spellings, fixity, precedence, associativity, AST sugar shape, lookup boundaries, and implementation boundary.   |
| `frontend-v0.1.md`                 | Non-normative overview                                | Historical reader entry point. Describes the v0.1 pipeline, document division, and the boundaries between tokens, AST, and diagnostics. |
| `entity-ref-design.md`             | Non-normative future design note                      | General `EntityRef` design (future). Alias-RHS `EntityRef` subset is implemented in Phase 4.4.                                               |
| `entity-alias-design.md`           | Implemented-design explanation                        | Documents lexical alias binding syntax (`let binder === EntityRef`). Phase 4.3 design; Phase 4.4 raw parser preservation implemented. Future semantic meaning remains future work. |
| `library-namespace-design-note.md` | Non-normative future design note                      | Describes the intended library/namespace/import model. Not a v0.1 parser rule.                                                              |
| `open-questions.md`                | Non-normative                                         | Tracks unresolved design questions and documentation debt.                                                                                 |
| `resolved-questions.md`            | Authoritative for resolved decisions                  | Records design questions resolved in v0.1.                                                                                                  |
| `build-system-design.md`           | Non-normative, future design                          | Formal design note for the build/package/namespace assembly architecture.                                                                   |
| `package-manifest-v0.md`           | Non-normative, future design                          | Provisional build-manifest design surface.                                                                                                  |
| `namespace-assembly-v0.md`         | Non-normative, future design                          | High-level namespace assembly pipeline and phase split.                                                                                     |
| `roadmap.md`                       | Authoritative for scope and planning; non-normative for parser behavior | Defines stage boundaries (v0.1–v0.11) and what must not leak between stages.                                                                 |

## Reading order

### Normal public reading order

This path is sufficient to understand the current non-semantic frontend
language: lexical syntax, concrete syntax, diagnostics/recovery, and the
Raw AST preservation surface.

1. `lexical-syntax-v0.2.md` - Understand the public lexical syntax.
2. `concrete-syntax-v0.2.md` - Understand the public concrete syntax.
3. `diagnostics-recovery-v0.2.md` - Understand public diagnostics and recovery.
4. `raw-ast-frozen-surface-v0.2.md` - Inspect the frozen Raw AST construct inventory.
5. `glossary.md` - Resolve terminology ambiguity.

### Extended implementer reading order

Read these only when implementing, auditing, or repairing the frontend.

1. `ast-construction-v0.1.md` - Implement the parser.
2. `diagnostics-v0.1.md` - Diagnostic catalog (implementation-level reference).
3. `implementation-status-v0.1.md` - Know current implementation facts.
4. `raw-ast-contract-v0.1.md` - Know Raw AST invariants for normalization.
5. `raw-ast-contract-freeze-v0.2.md` - Know v0.2 freeze boundary and v0.3 handoff.
6. `operator-design.md` - Understand operator syntax rules.
7. `resolved-questions.md` - Understand resolved design decisions.

### Future-design reading order

Read these only when working on future design topics.

1. `entity-alias-design.md` - Understand alias binding syntax (implemented) and future semantics.
2. `entity-ref-design.md` - Understand future general EntityRef design.
3. `library-namespace-design-note.md` - Understand library/namespace model.
4. `build-system-design.md` - Understand build/package architecture.
5. `package-manifest-v0.md` - Understand build-manifest surface.
6. `namespace-assembly-v0.md` - Understand namespace assembly pipeline.
7. `roadmap.md` - Understand scope boundaries.
8. `open-questions.md` - Recognize known gaps.

## Spec priority

For public v0.2 lexical syntax, concrete syntax, diagnostics, and recovery,
the v0.2 public specification set is the reader-facing authority.

Older v0.1 construction, diagnostic, operator, and implementation documents
remain backing references for implementation archaeology and consistency repair.

The implementation and golden snapshots remain the factual behavior source.
The v0.2 public specs are the public documentation surface for that behavior.
Older v0.1 documents are backing references and historical construction notes.

If a v0.2 public specification conflicts with an older v0.1 document, treat
the conflict as documentation debt to repair. Do not use the older document to
silently reinterpret the public v0.2 syntax surface.
