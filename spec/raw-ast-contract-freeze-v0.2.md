# Raw AST Contract Freeze v0.2

## Purpose

This document defines the v0.2 contract freeze boundary. It records what the
completed Raw AST frontend delivers, what work is allowed during `v0.2`, what
is forbidden, and what handoff requirements `v0.3 Normalized AST Specification`
may rely on.

`v0.2` is not a parser-expansion phase. It is a documentation-reconciliation
and contract-freeze stage preparing the exact boundary that v0.3 normalization
design will consume.

## v0.2 stage position

```text
v0.1   — Raw AST Frontend — completed
v0.1.w — Raw AST Stability Window — closed
v0.2   — Raw AST Contract Freeze / Normalization Boundary Preparation — current
v0.3   — Normalized AST Specification — future, begins only after v0.2 freeze
v0.4   — Raw AST → Normalized AST Prototype — future
v0.5   — Normalized AST Stabilization — future
v0.6+  — Later semantic design stages
```

## What v0.1 and v0.1.w delivered

- A complete Raw AST frontend: lexer, parser, token dump, AST dump, diagnostic dump.
- 29 `DiagnosticCode` variants across lexer, parser, operator, and alias categories.
- Golden test coverage (25 lexer, 298 parser, 43 diagnostics).
- `crates/lang_syntax` and `crates/lang_cli`.
- Richer literal spelling: radix integers, digit separators, scientific notation,
  hexadecimal floats, ranked quote-boundary strings.
- Pipe branch-name shorthand (`|> name { ... } ⇝ |> (_ name) { ... }`).

## Frozen v0.2 surface

The following are contract material in v0.2. They may not be broadened,
replaced, or restructured without going through a hard-correctness-error
exception:

- Lexer token categories: `Name`, `IntLiteral`, `FloatLiteral`, `StringLiteral`,
  `Symbol`, `Operator(OperatorSpelling)`, `Trivia`, `Invalid`, `Eof`
- 31 operator spellings plus `BracketCall` contextual operator
- 17 `Symbol` variants including `TripleEqual`
- Raw AST node categories documented in `ast-construction-v0.1.md`
- `lex` / `parse` public API
- Stable hand-written dump format for tokens, AST, and diagnostics
- Golden-test expectations
- Hard form boundaries (`;`, `}`, EOF)
- Weak lexer (no keyword classification)
- Product / product-extract architecture
- Pipe / segment / operator-expression architecture
- Closure AST preservation (InPlace, Explicit)
- Inner-to-outer navigation
- Alias-let parser preservation (`let binder === EntityRef`)
- `with { ... }` narrow payload grammar (names only)
- All 29 diagnostic codes with documented trigger conditions and span policy
- Binding-slot shape (policy, deduce, pattern, annotation, with, initializer)
- Canonical skeleton / deduce-list AST preservation (no matching semantics)
- Operator sugar as Raw AST (no lookup, no lowering)
- Full set of golden test snapshots

## Allowed v0.2 work

- Documentation consistency repair
- Stale comment cleanup
- Version / stage metadata alignment
- Raw AST contract freeze checklist
- Explicit inventory of frozen parser outputs
- Diagnostic inventory synchronization
- Golden-test inventory synchronization
- Correction of spec / code mismatches where implementation is already the
  settled Raw AST truth
- Narrowly scoped golden-test additions only if a frozen behavior is
  implemented but not locked by a test
- No parser behavior change unless a hard correctness error is identified

## Forbidden v0.2 work

- Broad lexer / parser restructuring
- New general syntax families
- Traditional call syntax (`f(args)`)
- Source-level import / use / include / module / package / export syntax
- General macro system
- Semantic analysis
- Name resolution
- Type checking
- Kind checking
- Operator lookup
- Overload resolution
- Alias target resolution
- Canonical matching
- Closure materialization
- Ownership / NLL / drop insertion
- HIR / MIR / codegen
- Interpreter behavior
- Raw AST → Normalized AST implementation

## Hard correctness error

A parser or AST change in v0.2 is allowed only if all of the following are true:

- The current Raw AST cannot represent the intended call-composition model, OR
  future normalization is logically impossible with the current shape, OR the
  current grammar forces heuristic parsing or semantic backtracking, OR an
  accepted syntax contradicts the product / pipe / operator / binding / closure /
  navigation architecture, OR a documented invariant is impossible to maintain
  without structural correction.

Aesthetic cleanup, naming preference, local convenience, and speculative future
semantic work are not hard correctness errors.

## Handoff requirements for v0.3

v0.3 Normalized AST Specification work may assume:

- The frozen Raw AST surface documented above is the complete v0.2 input.
- All Raw AST node variants exist and carry the fields documented in
  `spec/raw-ast-contract-v0.1.md`.
- Spans are valid and refer to the normalized (LF) source text.
- `ErrorAst` nodes and `Diagnostic` entries carry sufficient information for
  diagnostic rewiring through normalization.
- Product forms have no source / insert / right-target role enum in Raw AST.
- `OperatorSugar` with `fixity=Prefix` and `operator="-"` is the sole
  prefix-negative shape and must be normalized to typed-zero binary subtraction.
- `Normalized AST` is desugared but non-semantic. It is not HIR, not
  type-checked, not name-resolved.

v0.3 must not assume:

- Names are resolved.
- Operators are associated with operator declarations.
- Alias targets are resolved.
- Types or kinds have been checked or inferred.
- Canonical skeletons are admitted or well-formed.
- `Hole` / `NodeName` roles have been validated.
- Closures have been materialized.
- `match` / `effect` / `sync` have been recognized as anything beyond names.
- `with { ... }` carries lifetime or dependency semantics.
