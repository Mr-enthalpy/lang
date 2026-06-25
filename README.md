# lang

`lang` is an experimental programming language frontend.

**Current status:** v0.1 Raw AST Frontend completed; v0.1.w Raw AST Stability
Window closed; v0.2 Raw AST Contract Freeze / Public Frontend Syntax
Specification closed. Current active stage is v0.3 — Normalized AST
Specification.

The Raw AST frontend surface (lexer, parser, AST, diagnostics, golden tests)
is frozen contract material. Current work is the v0.3 Normalized AST
Specification — specifying a desugared but non-semantic intermediate
AST that unifies the Raw AST input surface into a regular structure
suitable for later semantic passes. v0.3 is specification-only;
implementation of Normalized AST lowering is v0.4. The frontend lexes,
parses, builds raw AST, emits diagnostics, and has golden tests.

- **Raw AST**: surface-preserving, non-desugared, parser output.
- **Normalized AST**: future desugared AST that unifies calls, extraction,
  and declarations into simple pattern/call/declaration structures.
  Not HIR, not type-checked, not name-resolved.

## Documentation map

### Current v0.3 specification workspace

The current active stage is v0.3 — Normalized AST Specification. Start here
for current-stage work:

| Document | Purpose |
|---|---|
| `spec/public/v0.3/README.md` | v0.3 stage workspace index |
| `spec/public/v0.3/normalized-ast-specification-v0.3.md` | Normalized AST specification scaffold and work items |
| `spec/contracts/v0.3-normalization-handoff-checklist.md` | v0.3 handoff: may-assume, must-not-assume, required inputs, open questions |
| `spec/planning/open-questions.md` | Open design questions (N-AST-1 through N-AST-8) |

### Frozen v0.2 frontend input authority

The v0.2 public frontend specification set remains authoritative for the frozen
Raw AST input surface. Read these for the input contract that v0.3 consumes:

| Document | Purpose |
|---|---|
| `spec/public/v0.2/lexical-syntax-v0.2.md` | Public lexical syntax specification: source normalization, token categories, weak lexer, names, literals, symbols, operators, trivia |
| `spec/public/v0.2/concrete-syntax-v0.2.md` | Public concrete syntax specification: form boundaries, let/alias-let, binding slots, products, pipes, operators, closures, skeletons, deduce lists |
| `spec/public/v0.2/diagnostics-recovery-v0.2.md` | Public diagnostics and recovery specification: lexical/parser diagnostic codes, trigger conditions, span policy, ErrorAst recovery, non-semantic boundaries |
| `spec/public/v0.2/raw-ast-frozen-surface-v0.2.md` | Frozen Raw AST surface inventory: construct-by-construct guarantees, v0.3 obligations |
| `spec/reference/glossary.md` | Terminology definitions and critical distinctions |

Older v0.1 implementation, contract, historical, planning, and future-design
documents remain present, but they are not part of the normal public reading path.

### Backing and historical references

| Category | Directory | Document | Purpose |
|---|---|---|---|
| Implementation | `spec/implementation/v0.1/` | `ast-construction-v0.1.md` | AST construction rules — implementation-level backing reference |
| Implementation | `spec/implementation/v0.1/` | `diagnostics-v0.1.md` | Diagnostic categories, span policy, recovery — implementation-level backing reference |
| Implementation | `spec/implementation/v0.1/` | `implementation-status-v0.1.md` | Authoritative factual inventory of current implementation status |
| Contract / handoff | `spec/contracts/` | `raw-ast-contract-v0.1.md` | Raw AST invariants for future normalization |
| Contract / handoff | `spec/contracts/` | `raw-ast-contract-freeze-v0.2.md` | v0.2 freeze boundary, allowed/forbidden work, v0.3 handoff |
| Contract / handoff | `spec/contracts/` | `v0.3-normalization-handoff-checklist.md` | v0.3 normalization handoff: may-assume, must-not-assume, required inputs, open v0.3 questions |
| Design / history | `spec/history/v0.1/` | `operator-design.md` | Operator syntax design and implementation boundaries — historical reference |
| Design / history | `spec/history/v0.1/` | `resolved-questions.md` | Design decisions — resolved for v0.1 |
| Design / history | `spec/history/v0.1/` | `frontend-v0.1.md` | Pipeline overview — historical reader entry point |

### Future design notes

| Directory | Document | Purpose |
|---|---|---|
| `spec/future/` | `entity-ref-design.md` | General `EntityRef` design (future); alias-RHS subset implemented in Phase 4.4 |
| `spec/future/` | `entity-alias-design.md` | Lexical alias binding design (Phase 4.3); raw parser preservation implemented in Phase 4.4; future semantic meaning remains future work |
| `spec/planning/` | `roadmap.md` | Stage model v0.1–v1.0 and scope boundaries |
| `spec/planning/` | `open-questions.md` | Unresolved design questions and documentation debt |

### Build / package / namespace (future notes)

| Directory | Document | Purpose |
|---|---|---|
| `spec/future/` | `library-namespace-design-note.md` | Non-normative future design note |
| `spec/future/` | `build-system-design.md` | Build/package/namespace assembly architecture (future) |
| `spec/future/` | `package-manifest-v0.md` | Provisional build-manifest design surface (future) |
| `spec/future/` | `namespace-assembly-v0.md` | Namespace assembly pipeline and phase split (future) |

### Operational

| Document | Purpose |
|---|---|
| `AGENTS.md` | Agent instructions — read before making code changes |
| `SKILL.md` | Operational workflow for frontend work |
| `spec/README.md` | Spec directory index with authority levels |

## Two repository tracks

1. **Frontend track** (active): v0.3 Normalized AST Specification current. v0.1/v0.1.w/v0.2 completed.
   v0.2 delivers `source text -> tokens -> Raw AST -> diagnostics` (frozen).
   v0.3 specifies Normalized AST (specification-only).

2. **Build/package/namespace assembly track** (documentation only for now):
   future build system, package manifest, and namespace assembly design.

Start with `spec/public/v0.3/README.md` for current-stage v0.3 work.
Read `spec/public/v0.2/lexical-syntax-v0.2.md` when you need the frozen Raw AST
input contract. Read `spec/public/v0.2/concrete-syntax-v0.2.md` for parsed
syntax and `spec/public/v0.2/diagnostics-recovery-v0.2.md` for error behavior.

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
  `acquire` an ordinary name
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
│   ├── public/
│   │   ├── v0.3/
│   │   │   ├── README.md
│   │   │   └── normalized-ast-specification-v0.3.md
│   │   └── v0.2/
│   │       ├── lexical-syntax-v0.2.md
│   │       ├── concrete-syntax-v0.2.md
│   │       ├── diagnostics-recovery-v0.2.md
│   │       └── raw-ast-frozen-surface-v0.2.md
│   ├── reference/
│   │   └── glossary.md
│   ├── implementation/
│   │   └── v0.1/
│   │       ├── ast-construction-v0.1.md
│   │       ├── diagnostics-v0.1.md
│   │       └── implementation-status-v0.1.md
│   ├── contracts/
│   │   ├── raw-ast-contract-v0.1.md
│   │   └── raw-ast-contract-freeze-v0.2.md
│   ├── history/
│   │   └── v0.1/
│   │       ├── frontend-v0.1.md
│   │       ├── operator-design.md
│   │       └── resolved-questions.md
│   ├── future/
│   │   ├── entity-ref-design.md
│   │   ├── entity-alias-design.md
│   │   ├── library-namespace-design-note.md
│   │   ├── build-system-design.md
│   │   ├── package-manifest-v0.md
│   │   └── namespace-assembly-v0.md
│   └── planning/
│       ├── roadmap.md
│       └── open-questions.md
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
See `spec/implementation/v0.1/implementation-status-v0.1.md` for the current test count.

## Non-goals for v0.1/v0.2

v0.1 does not implement type checking, kind checking, name resolution,
operator lookup, alias resolution, closure materialization, NLL/drop
insertion, interpretation, code generation, or IR/HIR/MIR lowering.

The parser preserves raw AST shape for these future passes but performs
none of them.

## How to read the spec

### Current v0.3 specification work

Start here for current-stage v0.3 Normalized AST Specification:

1. `spec/public/v0.3/README.md` — v0.3 workspace index.
2. `spec/public/v0.3/normalized-ast-specification-v0.3.md` — Normalized AST specification scaffold.
3. `spec/contracts/v0.3-normalization-handoff-checklist.md` — v0.3 may-assume, must-not-assume, required inputs.
4. `spec/planning/open-questions.md` — Open v0.3 design questions (N-AST-1 through N-AST-8).

### Frozen v0.2 frontend input

Read these for the frozen Raw AST input surface:

1. `spec/public/v0.2/lexical-syntax-v0.2.md` — Understand the frozen lexical syntax.
2. `spec/public/v0.2/concrete-syntax-v0.2.md` — Understand the frozen concrete syntax.
3. `spec/public/v0.2/diagnostics-recovery-v0.2.md` — Understand frozen diagnostics and recovery.
4. `spec/public/v0.2/raw-ast-frozen-surface-v0.2.md` — Inspect the frozen Raw AST construct inventory.
5. `spec/reference/glossary.md` — Resolve terminology.

### Extended implementer reading

Read these only when implementing, auditing, or repairing the frontend.

1. `spec/implementation/v0.1/ast-construction-v0.1.md` — Implement the parser.
2. `spec/implementation/v0.1/diagnostics-v0.1.md` — Diagnostic catalog (implementation-level reference).
3. `spec/implementation/v0.1/implementation-status-v0.1.md` — Know current implementation facts.
4. `spec/contracts/raw-ast-contract-v0.1.md` — Know Raw AST invariants for normalization.
5. `spec/contracts/raw-ast-contract-freeze-v0.2.md` — Know v0.2 freeze boundary and v0.3 handoff.
6. `spec/history/v0.1/operator-design.md` — Understand operator syntax and lookup boundaries.
7. `spec/history/v0.1/resolved-questions.md` — Understand resolved design decisions.
8. `spec/history/v0.1/frontend-v0.1.md` — Understand the pipeline (v0.1 overview).

### Future design and planning

Read these only when working on future design topics.

1. `spec/future/entity-ref-design.md` — Future general EntityRef design.
2. `spec/future/entity-alias-design.md` — Alias binding design (parser preservation implemented, semantics future).
3. `spec/planning/roadmap.md` — Understand scope boundaries.
4. `spec/planning/open-questions.md` — Recognize known gaps.

Other future design documents (build, package, namespace assembly, library namespace)
are listed in the Documentation map above under Build / package / namespace (future notes).

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
