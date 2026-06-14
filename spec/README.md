# Specification Index

This directory contains the specification documents for the `lang` language
frontend. Each document has a defined authority level.

## Document list

| File                               | Authority                                             | Role                                                                                                                                        |
| ---------------------------------- | ----------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `frontend-v0.1.md`                 | Non-normative overview                                | Reader entry point. Describes the v0.1 pipeline, document division, spec priority, and the boundaries between tokens, AST, and diagnostics. |
| `ast-construction-v0.1.md`         | Normative for parser behavior                         | Defines every syntax rule, AST shape, and parser constraint in v0.1.                                                                        |
| `operator-design.md`               | Normative for operator syntax design                   | Defines operator identity, spellings, fixity, precedence, associativity, AST sugar shape, lookup boundaries, and implementation boundary.   |
| `entity-alias-design.md`           | Non-normative future design note                      | Documents future lexical alias binding syntax (`let binder === EntityRef`) and the parser/semantic boundary. Not a v0.1 parser rule.        |
| `diagnostics-v0.1.md`              | Normative for error reporting                         | Defines diagnostic categories, span policy, and recovery behavior.                                                                          |
| `roadmap.md`                       | Authoritative for scope; non-normative for scheduling | Defines stage boundaries (v0.1–v1.0) and what must not leak between stages.                                                                 |
| `library-namespace-design-note.md` | Non-normative future design note                      | Describes the intended library/namespace/import model. Not a v0.1 parser rule.                                                              |
| `glossary.md`                      | Normative for terminology                             | Resolves naming ambiguity across all documents.                                                                                             |
| `open-questions.md`                | Non-normative                                         | Tracks unresolved design questions. Not implementation authority.                                                                           |
| `build-system-design.md`           | Non-normative, future design                          | Formal design note for the build/package/namespace assembly architecture. Not a v0.1 parser rule.                                            |
| `package-manifest-v0.md`           | Non-normative, future design                          | Provisional build-manifest design surface. Not a v0.1 parser rule.                                                                          |
| `namespace-assembly-v0.md`         | Non-normative, future design                          | High-level namespace assembly pipeline and phase split. Not a v0.1 parser rule.                                                             |

## Reading order

1. `frontend-v0.1.md` - Understand the pipeline.
2. `ast-construction-v0.1.md` - Implement the parser.
3. `operator-design.md` - Understand operator syntax rules.
4. `entity-alias-design.md` - Understand future lexical alias binding.
5. `diagnostics-v0.1.md` - Understand error reporting.
6. `glossary.md` - Resolve terminology ambiguity.
7. `roadmap.md` - Understand scope boundaries.
8. `open-questions.md` - Recognize known gaps.

## Spec priority

If two spec documents conflict, `ast-construction-v0.1.md` takes priority for
current parser behavior. `operator-design.md` defines operator syntax behavior
and future lookup boundaries. Operator expression parsing is implemented as raw
AST sugar, and operator names are preserved in binder/final-path-leaf
positions. Operator lookup, lowering, overload resolution, and alias binding
remain future work. This `README.md` is not itself a spec document.
