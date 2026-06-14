# lang

`lang` is an experimental programming language frontend.

**Current status:** The v0.1 Raw AST Frontend is completed. It lexes,
parses, builds raw AST, emits diagnostics, and has golden tests. It does not
perform semantic analysis.

The next work is Raw AST contract freeze and Normalized AST design:

- **Raw AST**: surface-preserving, non-desugared, parser output.
- **Normalized AST**: future desugared AST that unifies calls, extraction,
  and declarations into simple pattern/call/declaration structures.
  Not HIR, not type-checked, not name-resolved.

## Documentation map

### Current implemented specs

| Document | Purpose |
|---|---|
| `spec/implementation-status-v0.1.md` | Authoritative factual inventory of current implementation status |
| `spec/raw-ast-contract-v0.1.md` | Normative contract: Raw AST invariants for future normalization |
| `spec/ast-construction-v0.1.md` | Normative AST construction rules — implement parser from this |
| `spec/operator-design.md` | Normative operator syntax design and implementation boundaries |
| `spec/diagnostics-v0.1.md` | Normative diagnostic categories, span policy, recovery |
| `spec/glossary.md` | Terminology definitions and critical distinctions |
| `spec/frontend-v0.1.md` | Reader entry point — explains the pipeline and spec organization |

### Future design notes

| Document | Purpose |
|---|---|
| `spec/entity-ref-design.md` | General `EntityRef` design (future); alias-RHS subset implemented in Phase 4.4 |
| `spec/entity-alias-design.md` | Lexical alias binding design (Phase 4.3); raw parser preservation implemented in Phase 4.4; future semantic meaning remains future work |
| `spec/roadmap.md` | Stage model v0.1–v1.0 and scope boundaries |
| `spec/open-questions.md` | Unresolved design questions and documentation debt |

### Build / package / namespace (future notes)

| Document | Purpose |
|---|---|
| `spec/library-namespace-design-note.md` | Non-normative future design note |
| `spec/build-system-design.md` | Build/package/namespace assembly architecture (future) |
| `spec/package-manifest-v0.md` | Provisional build-manifest design surface (future) |
| `spec/namespace-assembly-v0.md` | Namespace assembly pipeline and phase split (future) |

### Operational

| Document | Purpose |
|---|---|
| `AGENTS.md` | Agent instructions — read before making code changes |
| `SKILL.md` | Operational workflow for frontend work |
| `spec/README.md` | Spec index with authority levels |

## Two repository tracks

1. **Frontend syntax track** (active): lexer, parser, AST, diagnostics.
   Implements `source text -> tokens -> AST -> diagnostics` for v0.1.

2. **Build/package/namespace assembly track** (documentation only for now):
   future build system, package manifest, and namespace assembly design.

Start with `spec/frontend-v0.1.md` to understand the pipeline, then
`spec/ast-construction-v0.1.md` for parser behavior. Read
`spec/operator-design.md` before changing operator syntax. Read
`spec/entity-ref-design.md` before changing future compile-time entity
reference or alias-binding syntax.

## Design summary

The language frontend is built around several early decisions.

### 1. Weak lexer

The lexer does not assign semantic roles to ordinary names.

Names such as:

```text
return else match drop move sync effect fn type meta runtime compile
```

are ordinary `Name` tokens.

Semantic strength does not imply lexical keyword status.

### 2. Contextual parser

Some names can act as structure delimiters only in strong parser contexts.

Examples:

- `let` at form start
- `where`/`acquire` as reserved future closure-head positions, not active
  Phase 3.1 parser clauses
- `guard` inside let bindings
- `with` inside let bindings

Outside their context, they remain ordinary names.

### 3. No traditional call syntax

The language does not use traditional:

```text
f(args)
```

as a general call form.

Parenthesized argument packs participate in the expression skeleton through
pipe and segment rules.

### 4. `|>` as expression skeleton

Expression construction is not based on a traditional C-like
operator-precedence table.

The expression frontend is organized around `|>` as the outer skeleton:

```text
top-level |> segmentation
  -> per-segment atom folding
  -> per-segment operator sugar
  -> per-segment automatic pipe
  -> argpack role assignment
```

The current parser preserves a segment-local `OperatorExpr` layer:

```text
SegmentElement := OperatorExpr | ArgPack
```

Ordinary operators bind tighter than whitespace auto-pipe and `|>`, but they
remain AST sugar.

Operator syntax is AST sugar only: no lookup, type checking, evaluation,
mutation semantics, or lowering is performed by the parser. Operator parsing is
local to one pipe segment and does not cross `|>` boundaries.

### 5. Closure literals produce AST first

A closure literal initially produces closure AST, not a callable object.

Examples:

```text
{}
() => {}
```

A later semantic pass may materialize closure AST into callable objects in
binding or call contexts.

Compiler meta-functions may directly consume closure AST.

### 6. `<>` declares holes

`<...>` has exactly one special use:

```text
declare names that act as holes in following syntax
```

It is only recognized in binding contexts.

It is not generic-call syntax, template syntax, or meta-function syntax.
Individual `<`, `>`, `<=`, and `>=` spellings are documented as planned
operator names in expression/operator contexts.

### 7. Declarations enter through `let`

All user-visible declarations use `let`. There is no dedicated parser syntax for
function, type, or namespace declarations.

`fn`, `type`, and `namespace` are ordinary `Name` tokens, not lexer keywords.
v0.1 parses and preserves declaration annotations but does not check their
semantic validity.

Bare declaration annotations are preserved exactly as written. Rank annotations
require the explicit `type_object_annotation : rank_annotation` form.

### 8. Parser owns shape, semantics owns meaning

The parser constructs and preserves raw AST shape. It does not decide what an
AST fragment semantically represents. Future semantic or meta-function passes
may interpret preserved shapes.

Parse left to right. Do not go back to reinterpret meaning. v0.1 must not add
special AST nodes just because a future built-in meta-function may understand
a shape.

## Workspace layout

```text
.
├── AGENTS.md
├── README.md
├── SKILL.md
├── Cargo.toml
├── docs/
│   └── decisions/
├── spec/
│   ├── README.md
│   ├── frontend-v0.1.md
│   ├── implementation-status-v0.1.md
│   ├── ast-construction-v0.1.md
│   ├── operator-design.md
│   ├── entity-ref-design.md
│   ├── entity-alias-design.md
│   ├── diagnostics-v0.1.md
│   ├── roadmap.md
│   ├── library-namespace-design-note.md
│   ├── build-system-design.md
│   ├── package-manifest-v0.md
│   ├── namespace-assembly-v0.md
│   ├── glossary.md
│   └── open-questions.md
├── crates/
│   ├── lang_syntax/
│   └── lang_cli/
└── tests/
    ├── lexer_golden.rs
    ├── parser_golden.rs
    ├── diagnostics_golden.rs
    └── cases/
        ├── lexer/
        ├── parser/
        └── diagnostics/
```

## Build

```bash
cargo check --workspace
cargo test
```

With `make` available:

```bash
make check
make test
make fmt
```

## CLI target

The `lang_cli` crate exposes:

```bash
lang tokens path/to/file.lang
lang ast path/to/file.lang
lang diag path/to/file.lang
```

The repository has golden coverage for lexer, parser/AST, and diagnostics.
See `spec/implementation-status-v0.1.md` for the current test count.

## Non-goals for v0.1

v0.1 does not implement type checking, kind checking, name resolution,
operator lookup, alias resolution, closure materialization, NLL/drop
insertion, interpretation, code generation, or IR/HIR/MIR lowering.

The parser preserves raw AST shape for these future passes but performs
none of them.

## How to read the spec

1. `spec/frontend-v0.1.md` — Understand the pipeline.
2. `spec/implementation-status-v0.1.md` — Know what is currently implemented.
3. `spec/raw-ast-contract-v0.1.md` — Know Raw AST invariants for normalization.
4. `spec/ast-construction-v0.1.md` — Implement the parser.
5. `spec/operator-design.md` — Understand operator syntax and lookup boundaries.
6. `spec/entity-ref-design.md` — Future general EntityRef design.
7. `spec/entity-alias-design.md` — Alias binding design (parser preservation implemented, semantics future).
8. `spec/diagnostics-v0.1.md` — Understand error reporting.
9. `spec/glossary.md` — Resolve terminology.
10. `spec/roadmap.md` — Understand scope boundaries.
11. `spec/open-questions.md` — Recognize known gaps.

## Expected future workspace shape

Future stages may add crates under `crates/` such as:

```text
crates/
  lang_syntax/
  lang_cli/
  lang_typeck/       (v0.7+)
  lang_nll/          (v0.8+)
  lang_codegen/      (v1.0+)
  lang_build/        (future: build/package track)
  lang_manifest/     (future: manifest parser)
```

No semantic crate should be added before its corresponding design stage.
