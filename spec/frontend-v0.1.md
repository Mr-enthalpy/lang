# v0.1 Frontend Overview

## Purpose

This document is a reader entry point for the `lang` v0.1 frontend. It explains
the pipeline, how the specification documents are organized, and what each
component is responsible for.

It does **not** define syntax rules, AST shapes, or parser algorithms. Those
are in `ast-construction-v0.1.md`.

## Pipeline

```text
source text
    ↓  lexer
tokens
    ↓  parser
raw AST + diagnostics
    ↓  dumper
stable text dump (tokens | AST | diagnostics)
```

v0.1 produces **three** stable, dumpable outputs:

1. **Token dump**: sequence of tokens with spans.
2. **AST dump**: structured AST tree in a hand-written format.
3. **Diagnostic dump**: list of errors/warnings with spans.

These three outputs are the only acceptance criterion for v0.1. No program is
executed, type-checked, or lowered.

## Document division

| Document                   | What it covers                                                                                                                                                          |
| -------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ast-construction-v0.1.md` | Token classes consumed by parser, form/let/expr grammar, ArgPack roles, closure AST, deduce lists, canonical skeletons, atom suffix folding, error nodes, golden cases. |
| `operator-design.md`       | Operator syntax design: operator names, fixity, precedence, associativity, AST sugar, binder/navigation-component positions, and lookup boundary.                       |
| `entity-ref-design.md`     | Future general compile-time entity reference syntax design. Alias-RHS EntityRef subset implemented in Phase 4.4.                                                        |
| `entity-alias-design.md`   | Lexical alias binding syntax using `let binder === EntityRef`. Phase 4.3 design; Phase 4.4 raw parser preservation implemented; future semantic meaning remains future work. |
| `diagnostics-v0.1.md`      | Diagnostic categories, trigger conditions, span policy, recovery behavior.                                                                                              |
| `implementation-status-v0.1.md` | Authoritative factual inventory of current implementation status.                                                                                                  |
| `glossary.md`              | Terminology definitions and distinctions.                                                                                                                               |
| `roadmap.md`               | Stage model from v0.1 to v1.0, scope boundaries.                                                                                                                        |
| `open-questions.md`        | Unresolved design questions.                                                                                                                                            |

## Spec priority

`ast-construction-v0.1.md` is the normative authority for parser behavior.
If a conflict arises between documents, `ast-construction-v0.1.md` takes
precedence.

`entity-ref-design.md` and `entity-alias-design.md` document future strong
syntax contexts and alias binding design. Phase 4.4 implements raw parser
preservation for alias binding and alias-RHS EntityRef.

## Boundaries

### Lexer boundary

The lexer handles character-level concerns only:

- Recognize identifiers, literals, symbols, trivia, invalid sequences.
- Produce tokens with spans.
- Normalize CRLF and CR line endings to LF before tokenization. Token byte
  spans, lines, and columns refer to this normalized LF source text.
- **Do not** classify names as keywords.
- **Do not** interpret `<...>` as deduce lists.
- **Do not** balance groups or track parser state.

### Parser boundary

The parser handles token-level concerns only:

- Consume tokens, build AST.
- Recognize strong contexts (let, closure head, etc.).
- Recognize syntax-level operator shape where implemented.
- Produce diagnostics for malformed input.
- **Do not** type-check, kind-check, resolve names, or perform semantic analysis.
- **Do not** materialize closures into callable objects.
- **Do not** lower operator syntax into ordinary calls.
- **Do not** perform operator lookup, overload resolution, ADL, or evaluation.
- **Do not** special-case `match`, `return`, `else`, `drop`, `move`, `sync`,
  `effect`, `fn`, `type`, `meta`, `runtime`, or `compile`.

### Diagnostic boundary

Diagnostics in v0.1 cover lexer and parser errors only:

- Invalid tokens, unclosed groups, unexpected tokens, malformed syntax.
- **Do not** produce type errors, kind errors, lifetime errors, or semantic
  warnings.

## How to read this spec

1. Start here.
2. Read `ast-construction-v0.1.md` to understand syntax rules and AST shapes.
3. Read `implementation-status-v0.1.md` to know the current implementation facts.
4. Refer to `glossary.md` when a term is unclear.
5. Read `entity-alias-design.md` for alias binding design (implemented as raw parser preservation).
6. Read `entity-ref-design.md` for future general EntityRef design.
7. Consult `diagnostics-v0.1.md` for error behavior.
8. Check `roadmap.md` to understand what is deferred.
9. Check `open-questions.md` for unresolved design items before making
   decisions.

## Current implementation status

The v0.1 frontend covers:

- Operator-aware lexer with CRLF/LF normalization, operator spellings, and `===` token
- Full parser: simple let, extract let, let alias, pipe/segment/argpack,
  operator expressions, path/member/double-dot sugar, closures with heads,
  canonical skeletons, deduce lists
- Stable token, AST, and diagnostic dumps
- Diagnostic taxonomy covering lexer, parser, operator, and alias parsing
- Golden test coverage for lexer, parser/AST, and diagnostics

For the detailed, factual inventory see `spec/implementation-status-v0.1.md`.

For the current phase breakdown see `spec/roadmap.md`.

It does **not** yet cover:

- Operator lookup, lowering, overload resolution, dispatch, ADL, or
  type-directed lookup.
- Compile-time `EntityRef` resolution.
- Operator alias identity validation.
- The reserved-inactive `where` closure clause, closure object materialization,
  type/kind checking, name resolution, semantic analysis, lowering,
  interpretation, or code generation. (The `require`/`pre`/`post`/`lifetime
  pre`/`lifetime post` head clauses are parsed as raw AST shape only; their
  meaning is deferred to later phases.)

The expression segment design is:

```text
SegmentElement := OperatorExpr | ArgPack
```

`OperatorExpr` is a segment-local expression layer built from atoms. Operator
syntax remains AST sugar and does not imply lookup, type checking, evaluation,
or lowering.

See `spec/roadmap.md` for a detailed phase breakdown and current coverage.
