# Specification Index

This directory contains the specification documents for the `lang` language
frontend. Each document has a defined authority level.

## Document list

| File                               | Authority                                             | Role                                                                                                                                        |
| ---------------------------------- | ----------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `frontend-v0.1.md`                 | Non-normative overview                                | Reader entry point. Describes the v0.1 pipeline, document division, and the boundaries between tokens, AST, and diagnostics. |
| `ast-construction-v0.1.md`         | Normative for parser behavior                         | Defines every syntax rule, AST shape, and parser constraint in v0.1.                                                                        |
| `operator-design.md`               | Normative for operator syntax design                   | Defines operator identity, spellings, fixity, precedence, associativity, AST sugar shape, lookup boundaries, and implementation boundary.   |
| `implementation-status-v0.1.md`    | Authoritative factual inventory                      | Records the current implementation status of every feature. Does not define parser rules — `ast-construction-v0.1.md` is normative.          |
| `raw-ast-contract-v0.1.md`         | Normative contract for future normalization          | Defines Raw AST invariants that future normalization passes may rely on.                                                                     |
| `raw-ast-contract-freeze-v0.2.md`  | Normative for v0.2 contract freeze                    | Defines v0.2 freeze boundary, allowed work, forbidden work, and handoff requirements for v0.3.                                                |
| `raw-ast-frozen-surface-v0.2.md`   | Normative frozen surface inventory                    | Enumerates frozen Raw AST constructs with guarantees, non-semantic boundaries, v0.3 obligations, and forbidden assumptions.                    |
| `lexical-syntax-v0.2.md`           | Normative for public lexical syntax                   | Defines source normalization, lexical categories, token spellings, comments, literals, invalid lexical material, and non-semantic lexer boundaries for v0.2. |
| `concrete-syntax-v0.2.md`          | Normative for public concrete syntax                  | Defines the accepted non-semantic source-level grammar, parser shape, Raw AST preservation boundaries, and parser-level non-semantic constraints for v0.2. |
| `diagnostics-v0.1.md`              | Normative for error reporting                         | Defines diagnostic categories, span policy, and recovery behavior.                                                                          |
| `glossary.md`                      | Normative for terminology                             | Resolves naming ambiguity across all documents.                                                                                             |
| `roadmap.md`                       | Authoritative for scope and planning; non-normative for parser behavior | Defines stage boundaries (v0.1–v0.11) and what must not leak between stages.                                                                 |
| `entity-ref-design.md`             | Non-normative future design note                      | General `EntityRef` design (future). Alias-RHS `EntityRef` subset is implemented in Phase 4.4.                                               |
| `entity-alias-design.md`           | Implemented-design explanation                        | Documents lexical alias binding syntax (`let binder === EntityRef`). Phase 4.3 design; Phase 4.4 raw parser preservation implemented. Future semantic meaning remains future work. |
| `library-namespace-design-note.md` | Non-normative future design note                      | Describes the intended library/namespace/import model. Not a v0.1 parser rule.                                                              |
| `open-questions.md`                | Non-normative                                         | Tracks unresolved design questions and documentation debt.                                                                                 |
| `resolved-questions.md`            | Authoritative for resolved decisions                  | Records design questions resolved in v0.1.                                                                                                  |
| `build-system-design.md`           | Non-normative, future design                          | Formal design note for the build/package/namespace assembly architecture.                                                                   |
| `package-manifest-v0.md`           | Non-normative, future design                          | Provisional build-manifest design surface.                                                                                                  |
| `namespace-assembly-v0.md`         | Non-normative, future design                          | High-level namespace assembly pipeline and phase split.                                                                                     |

## Reading order

1. `frontend-v0.1.md` - Understand the pipeline.
2. `implementation-status-v0.1.md` - Know current implementation facts.
3. `raw-ast-contract-v0.1.md` - Know Raw AST invariants for normalization.
4. `raw-ast-contract-freeze-v0.2.md` - Know v0.2 freeze boundary and v0.3 handoff.
5. `raw-ast-frozen-surface-v0.2.md` - Inspect the frozen Raw AST construct inventory.
6. `lexical-syntax-v0.2.md` - Understand the public lexical syntax.
7. `concrete-syntax-v0.2.md` - Understand the public concrete syntax.
8. `ast-construction-v0.1.md` - Implement the parser.
9. `operator-design.md` - Understand operator syntax rules.
10. `entity-alias-design.md` - Understand alias binding syntax (implemented) and future semantics.
11. `entity-ref-design.md` - Understand future general EntityRef design.
12. `diagnostics-v0.1.md` - Understand error reporting.
13. `glossary.md` - Resolve terminology ambiguity.
14. `roadmap.md` - Understand scope boundaries.
15. `open-questions.md` - Recognize known gaps.
16. `resolved-questions.md` - Understand resolved design decisions.

## Spec priority

If two spec documents conflict, `ast-construction-v0.1.md` takes priority for
current parser behavior. `operator-design.md` defines operator syntax behavior
and future lookup boundaries. Operator expression parsing is implemented as raw
AST sugar, and operator names are preserved in binder and innermost
navigation-component positions. Navigation order is inner-to-outer: the
leftmost component is the innermost selected symbol and the rightmost component
is the outermost scope component. Raw AST preserves source-order navigation
components and performs no lookup. Alias binding (`let binder === EntityRef`) is implemented as raw
AST preservation; alias semantics, target resolution, operator identity
validation, and lookup remain future work. `entity-ref-design.md` defines the
general `EntityRef` design; the alias-RHS subset is implemented.
`implementation-status-v0.1.md` records current implementation facts but does
not define parser rules. This `README.md` is not itself a spec document.
