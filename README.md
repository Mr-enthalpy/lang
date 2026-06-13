# lang

`lang` is an experimental programming language frontend.

The current repository target is **v0.1**.

## Current stage

```text
source text -> tokens -> AST -> diagnostics
```

v0.1 is a syntax frontend only. It lexes, parses, builds AST, and produces
diagnostics. It does not type-check, interpret, lower, or execute programs.

## Documentation map

| Document                        | Purpose                                                          |
| ------------------------------- | ---------------------------------------------------------------- |
| `spec/frontend-v0.1.md`         | Reader entry point — explains the pipeline and spec organization |
| `spec/ast-construction-v0.1.md` | Normative AST construction rules — implement parser from this    |
| `spec/operator-design.md`       | Normative operator design for future parser work                 |
| `spec/diagnostics-v0.1.md`      | Normative diagnostic categories, span policy, recovery           |
| `spec/roadmap.md`               | Stage model v0.1–v1.0 and scope boundaries                       |
| `spec/glossary.md`              | Terminology definitions and critical distinctions                |
| `spec/open-questions.md`        | Unresolved design questions                                      |
| `spec/README.md`                | Spec index with authority levels                                 |
| `AGENTS.md`                     | Agent instructions — read before making code changes             |
| `SKILL.md`                      | Operational workflow for frontend work                           |

Start with `spec/frontend-v0.1.md` to understand the pipeline, then
`spec/ast-construction-v0.1.md` for parser behavior. Read
`spec/operator-design.md` before changing planned operator syntax.

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
- `where` in closure heads
- `acquire` in closure heads
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
  -> per-segment operator sugar (planned)
  -> per-segment automatic pipe
  -> argpack role assignment
```

Operator syntax is documented as a planned segment-local expression layer. It
is AST sugar only: no lookup, type checking, evaluation, mutation semantics, or
lowering is performed by the parser. The ordering above describes binding
structure, not a required parser implementation order. Operator parsing is local
to one pipe segment and does not cross `|>` boundaries.

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
│   ├── ast-construction-v0.1.md
│   ├── diagnostics-v0.1.md
│   ├── roadmap.md
│   ├── library-namespace-design-note.md
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

The `lang_cli` crate currently exposes:

```bash
lang tokens path/to/file.lang
lang ast path/to/file.lang
lang diag path/to/file.lang
```

These commands are implemented as entry points for the current frontend
skeleton.

Current coverage:

- `tokens`: backed by the current lexer implementation and lexer golden tests.
- `ast`: backed by parser phase 1; it does not yet cover the full v0.1 AST
  construction spec (see `spec/roadmap.md` for phase boundaries).
- `diag`: emits lexer/parser diagnostics available in the current
  implementation; the full v0.1 diagnostic catalog and diagnostics golden
  suite are still incomplete.

The output format is stable and suitable for golden tests. All dump output
uses hand-written formatting, not Rust `Debug`.

## Non-goals for v0.1

Do not implement:

- type checking
- kind checking
- overload resolution
- canonical-form evaluation
- universal extraction matching
- closure AST materialization into callable objects
- match / effect / sync semantics
- ownership, lifetime, NLL, drop insertion
- interpretation
- code generation
- IR / HIR / MIR lowering
- parser generators (hand-written parser only)
- library / import / export / module / package syntax

The parser should preserve syntax sufficient for these future passes, but
must not perform them.

v0.1 only preserves raw namespace path shapes such as `std::Vec`,
`mylib::math::vector::Vec3`.

## How to read the spec

1. `spec/frontend-v0.1.md` — Understand the pipeline.
2. `spec/ast-construction-v0.1.md` — Implement the parser.
3. `spec/operator-design.md` - Understand planned operator syntax.
4. `spec/diagnostics-v0.1.md` - Understand error reporting.
5. `spec/glossary.md` - Resolve terminology.
6. `spec/roadmap.md` - Understand scope boundaries.
7. `spec/library-namespace-design-note.md` - Non-normative future design note.
8. `spec/open-questions.md` - Recognize known gaps.

## Expected future workspace shape

Future stages may add crates under `crates/` such as:

```text
crates/
  lang_syntax/
  lang_cli/
  lang_typeck/       (v0.7+)
  lang_nll/          (v0.8+)
  lang_codegen/      (v1.0+)
```

No semantic crate should be added before its corresponding design stage.
