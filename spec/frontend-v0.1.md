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
| `operator-design.md`       | Planned operator syntax design: operator names, fixity, precedence, associativity, AST sugar, path/binder leaves, and lookup boundary.                                  |
| `diagnostics-v0.1.md`      | Diagnostic categories, trigger conditions, span policy, recovery behavior.                                                                                              |
| `glossary.md`              | Terminology definitions and distinctions.                                                                                                                               |
| `roadmap.md`               | Stage model from v0.1 to v1.0, scope boundaries.                                                                                                                        |
| `open-questions.md`        | Unresolved design questions.                                                                                                                                            |

## Spec priority

`ast-construction-v0.1.md` is the normative authority for parser behavior.
If a conflict arises between documents, `ast-construction-v0.1.md` takes
precedence.

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
- Recognize syntax-level operator shape when the operator parser is implemented.
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
3. Refer to `glossary.md` when a term is unclear.
4. Consult `diagnostics-v0.1.md` for error behavior.
5. Check `roadmap.md` to understand what is deferred.
6. Check `open-questions.md` for unresolved design items before making
   decisions.

## Current implementation status

The current implementation includes:

- Lexer with operator-aware tokenization (operator spellings recognized as
  tokens; structural symbols distinguished from operators).
- Stable token/AST/diagnostic dumps.
- Simple let forms (bare annotations, explicit rank annotations, guard
  attributes, with clauses).
- Name, integer literal, string literal, and path expression atoms (including
  numeric path leaves such as `uint8::1`).
- Group atoms `(expr)`.
- Pipe segmentation (`|>`) and ArgPack role assignment.
- Atom suffix folding: `:: Selector`, `. Selector` (MemberSugar),
  `.. Selector ArgPack` (DoubleDotSugar).
- Numeric selectors (`obj.1`, `obj..1(args)`) using NumericNameAst.
- Top-level newline form boundaries implemented.
- Full v0.1 diagnostic taxonomy in DiagnosticCode; most codes reachable;
  some phase-2 diagnostics exist in the enum before their syntax lands.
- Golden test suites: 9 lexer, 38 parser, 27 diagnostics tests.

It does **not** yet cover:

- Extract-let binders, deduce lists, canonical skeletons.
- Closure AST (inline `{}`, explicit `() => {}`, closure heads).
- Operator parser (operator spellings are lexed but expression-level operator
  parsing, precedence, associativity, and operator-sugar AST are not
  implemented).

When operator syntax is enabled, the expression segment design becomes:

```text
SegmentElement := OperatorExpr | ArgPack
```

`OperatorExpr` is a segment-local expression layer built from atoms. Operator
syntax remains AST sugar and does not imply lookup, type checking, evaluation,
or lowering.

See `spec/roadmap.md` for a detailed phase breakdown and current coverage.
