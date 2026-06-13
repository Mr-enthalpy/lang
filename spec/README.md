# Specification Index

This directory contains the specification documents for the `lang` language
frontend. Each document has a defined authority level.

## Document list

| File                               | Authority                                             | Role                                                                                                                                        |
| ---------------------------------- | ----------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `frontend-v0.1.md`                 | Non-normative overview                                | Reader entry point. Describes the v0.1 pipeline, document division, spec priority, and the boundaries between tokens, AST, and diagnostics. |
| `ast-construction-v0.1.md`         | Normative for parser behavior                         | Defines every syntax rule, AST shape, and parser constraint in v0.1.                                                                        |
| `operator-design.md`               | Normative design for future parser work               | Defines operator identity, spellings, fixity, precedence, associativity, AST sugar shape, lookup boundaries, and implementation boundary.   |
| `diagnostics-v0.1.md`              | Normative for error reporting                         | Defines diagnostic categories, span policy, and recovery behavior.                                                                          |
| `roadmap.md`                       | Authoritative for scope; non-normative for scheduling | Defines stage boundaries (v0.1–v1.0) and what must not leak between stages.                                                                 |
| `library-namespace-design-note.md` | Non-normative future design note                      | Describes the intended library/namespace/import model. Not a v0.1 parser rule.                                                              |
| `glossary.md`                      | Normative for terminology                             | Resolves naming ambiguity across all documents.                                                                                             |
| `open-questions.md`                | Non-normative                                         | Tracks unresolved design questions. Not implementation authority.                                                                           |

## Reading order

1. `frontend-v0.1.md` - Understand the pipeline.
2. `ast-construction-v0.1.md` - Implement the parser.
3. `operator-design.md` - Understand planned operator syntax rules.
4. `diagnostics-v0.1.md` - Understand error reporting.
5. `glossary.md` - Resolve terminology ambiguity.
6. `roadmap.md` - Understand scope boundaries.
7. `open-questions.md` - Recognize known gaps.

## Spec priority

If two spec documents conflict, `ast-construction-v0.1.md` takes priority for
current parser behavior. `operator-design.md` defines planned operator behavior
only where the current parser spec explicitly marks operator support as not yet
implemented. The current parser is not required to accept operator syntax until
the operator parser work lands. This `README.md` is not itself a spec document.
