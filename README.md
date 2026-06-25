# lang

`lang` is an experimental programming language frontend.

**Current status:** v0.1 Raw AST Frontend completed; v0.1.w Raw AST Stability
Window closed. Current active stage is v0.2 — Raw AST Contract Freeze /
Normalization Boundary Preparation.

The lexer/parser architecture, public frontend interfaces, Raw AST shape, dump
formats, and golden-test expectations are frozen contract material. Work in
this stage is documentation reconciliation, contract freezing, consistency
repair, version/stage metadata alignment, and preparation of the exact boundary
that v0.3 Normalized AST Specification will consume. It does not implement
Normalized AST, semantic analysis, or new syntax features. The frontend lexes,
parses, builds raw AST, emits diagnostics, and has golden tests.

- **Raw AST**: surface-preserving, non-desugared, parser output.
- **Normalized AST**: future desugared AST that unifies calls, extraction,
  and declarations into simple pattern/call/declaration structures.
  Not HIR, not type-checked, not name-resolved.

## Documentation map

### Public v0.2 frontend specification

The normal public reading path for current lexical and syntactic behavior:

| Document | Purpose |
|---|---|
| `spec/lexical-syntax-v0.2.md` | Public lexical syntax specification: source normalization, token categories, weak lexer, names, literals, symbols, operators, trivia |
| `spec/concrete-syntax-v0.2.md` | Public concrete syntax specification: form boundaries, let/alias-let, binding slots, products, pipes, operators, closures, skeletons, deduce lists |
| `spec/diagnostics-recovery-v0.2.md` | Public diagnostics and recovery specification: lexical/parser diagnostic codes, trigger conditions, span policy, ErrorAst recovery, non-semantic boundaries |
| `spec/raw-ast-frozen-surface-v0.2.md` | Frozen Raw AST surface inventory: construct-by-construct guarantees, v0.3 obligations |
| `spec/glossary.md` | Terminology definitions and critical distinctions |

Older v0.1 design and implementation documents remain available as backing
references, but they are not the normal public entry point.

### Backing and historical references

| Category | Document | Purpose |
|---|---|---|
| Implementation | `spec/ast-construction-v0.1.md` | Normative AST construction rules — implement parser from this |
| Implementation | `spec/diagnostics-v0.1.md` | Normative diagnostic categories, span policy, recovery (implementation-level) |
| Implementation | `spec/implementation-status-v0.1.md` | Authoritative factual inventory of current implementation status |
| Contract / handoff | `spec/raw-ast-contract-v0.1.md` | Raw AST invariants for future normalization |
| Contract / handoff | `spec/raw-ast-contract-freeze-v0.2.md` | v0.2 freeze boundary, allowed/forbidden work, v0.3 handoff |
| Design / history | `spec/operator-design.md` | Normative operator syntax design and implementation boundaries |
| Design / history | `spec/resolved-questions.md` | Design decisions — resolved for v0.1 |
| Design / history | `spec/frontend-v0.1.md` | Reader entry point — explains the pipeline and spec organization |

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

1. **Frontend track** (active): Raw AST frontend (v0.1) completed; v0.1.w closed; v0.2 Contract Freeze current.
   Delivers `source text -> tokens -> Raw AST -> diagnostics`.

2. **Build/package/namespace assembly track** (documentation only for now):
   future build system, package manifest, and namespace assembly design.

Start with `spec/lexical-syntax-v0.2.md` to understand the current lexical
syntax, then `spec/concrete-syntax-v0.2.md` for parsed syntax and
`spec/diagnostics-recovery-v0.2.md` for error behavior. Read
`spec/raw-ast-frozen-surface-v0.2.md` for the frozen Raw AST inventory.

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
- `require`/`pre`/`post`/`lifetime pre`/`lifetime post` as active raw-AST
  closure-head clauses (one expression slot each, no semantic validation);
  `where` reserved-inactive; `acquire` an ordinary name
- `with` inside let bindings, only as `with { ... }`

Outside their context, they remain ordinary names.

### 2a. Inner-to-outer navigation

Navigation order is inner-to-outer. The leftmost component is the innermost
selected symbol, and the rightmost component is the outermost scope component.
Raw AST preserves source-order navigation components and performs no lookup.

Examples:

```text
x::T::std
+::int::std
xxx::(int Vec::std)
```

Parenthesized right-side scope expressions after `::` are preserved as grouped
navigation components. Without parentheses, `::` consumes only one immediate
valid navigation component.

### 3. No traditional call syntax

The language does not use traditional:

```text
f(args)
```

as a general call form.

Parenthesized top-level-comma forms are product forms. In expression context
they are product construction; in binding / extraction context they are product
extraction.

### 4. `|>` as expression skeleton

Expression construction is not based on a traditional C-like
operator-precedence table.

The expression frontend is organized around `|>` as the outer skeleton:

```text
top-level |> segmentation
  -> per-segment atom folding
  -> per-segment operator sugar
  -> per-segment automatic pipe
  -> product form preservation
```

The current parser preserves a segment-local `OperatorExpr` layer:

```text
SegmentElement := OperatorExpr | Product
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
() => {}
```

Bare `{ ... }` in atom position produces `ClosureAst::InPlace`. It is not a
normal block expression and has no closure head. Braces also delimit explicit
closure bodies after `FnHeadPrefix =>`.

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
│   ├── raw-ast-contract-v0.1.md
│   ├── raw-ast-contract-freeze-v0.2.md
│   ├── raw-ast-frozen-surface-v0.2.md
│   ├── lexical-syntax-v0.2.md
│   ├── concrete-syntax-v0.2.md
│   ├── diagnostics-recovery-v0.2.md
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
│   ├── open-questions.md
│   └── resolved-questions.md
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

## Non-goals for v0.1/v0.2

v0.1 does not implement type checking, kind checking, name resolution,
operator lookup, alias resolution, closure materialization, NLL/drop
insertion, interpretation, code generation, or IR/HIR/MIR lowering.

The parser preserves raw AST shape for these future passes but performs
none of them.

## How to read the spec

### Normal reading path

This path is sufficient to understand the current non-semantic frontend
language: lexical syntax, concrete syntax, Raw AST preservation surface,
diagnostics, and recovery.

1. `spec/lexical-syntax-v0.2.md` — Understand the public lexical syntax.
2. `spec/concrete-syntax-v0.2.md` — Understand the public concrete syntax.
3. `spec/diagnostics-recovery-v0.2.md` — Understand public diagnostics and recovery.
4. `spec/raw-ast-frozen-surface-v0.2.md` — Inspect the frozen Raw AST construct inventory.
5. `spec/glossary.md` — Resolve terminology.

### Extended implementer reading

Read these only when implementing, auditing, or repairing the frontend.

1. `spec/ast-construction-v0.1.md` — Implement the parser.
2. `spec/diagnostics-v0.1.md` — Diagnostic catalog (implementation-level reference).
3. `spec/implementation-status-v0.1.md` — Know current implementation facts.
4. `spec/raw-ast-contract-v0.1.md` — Know Raw AST invariants for normalization.
5. `spec/raw-ast-contract-freeze-v0.2.md` — Know v0.2 freeze boundary and v0.3 handoff.
6. `spec/operator-design.md` — Understand operator syntax and lookup boundaries.
7. `spec/frontend-v0.1.md` — Understand the pipeline (v0.1 overview).

### Future design and planning

Read these only when working on future design topics.

1. `spec/entity-ref-design.md` — Future general EntityRef design.
2. `spec/entity-alias-design.md` — Alias binding design (parser preservation implemented, semantics future).
3. `spec/roadmap.md` — Understand scope boundaries.
4. `spec/open-questions.md` — Recognize known gaps.
5. `spec/resolved-questions.md` — Understand resolved design decisions.

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
