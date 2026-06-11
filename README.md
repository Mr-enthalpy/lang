# File: README.md

# lang

`lang` is an experimental programming language frontend.

The current repository target is `v0.1`.

`v0.1` is deliberately narrow:

```text
source text -> tokens -> AST -> diagnostics
```

It does not type-check, interpret, lower, compile, or execute programs.

## Current status

The repository is in the syntax-front-end stage.

The first implementation should produce:

* token dumps
* AST dumps
* diagnostic dumps
* golden tests for the above

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

* `let` at form start
* `where` in closure heads
* `acquire` in closure heads
* `guard` inside let bindings
* `with` inside let bindings

Outside their context, they remain ordinary names.

### 3. No traditional call syntax

The language does not use traditional:

```text
f(args)
```

as a general call form.

Parenthesized argument packs participate in the expression skeleton through pipe and segment rules.

### 4. `|>` as expression skeleton

Expression construction is not based on a traditional operator-precedence table.

The expression frontend is organized as:

```text
atom folding
  -> top-level |> segmentation
  -> segment-local automatic pipe
  -> argpack role assignment
```

### 5. Closure literals produce AST first

A closure literal initially produces closure AST, not a callable object.

Examples:

```text
{}
() => {}
```

A later semantic pass may materialize closure AST into callable objects in binding or call contexts.

Compiler meta-functions may directly consume closure AST.

### 6. `<>` declares holes

`<...>` has exactly one special use:

```text
declare names that act as holes in following syntax
```

It is only recognized in binding contexts.

It is not generic-call syntax, template syntax, or meta-function syntax.

## Suggested workspace

```text
.
├── AGENTS.md
├── README.md
├── SKILL.md
├── Cargo.toml
├── Makefile
├── spec/
│   ├── frontend-v0.1.md
│   ├── ast-construction-v0.1.md
│   └── diagnostics-v0.1.md
├── crates/
│   ├── lang_syntax/
│   └── lang_cli/
└── tests/
    ├── lexer_golden.rs
    ├── ast_golden.rs
    ├── diagnostics_golden.rs
    └── cases/
```

## Build

After workspace initialization:

```bash
cargo check --workspace
cargo test
```

Convenience command:

```bash
make test
```

## CLI target

The `lang_cli` crate should eventually support:

```bash
lang tokens path/to/file.lang
lang ast path/to/file.lang
lang diag path/to/file.lang
```

The output format should be stable and suitable for golden tests.

Use a hand-written dump format rather than Rust `Debug` output.

## Specification files

Primary specification files:

```text
spec/ast-construction-v0.1.md
spec/diagnostics-v0.1.md
```

The implementation must follow the spec rather than undocumented parser behavior.

## Non-goals for v0.1

Do not implement:

* type checking
* overload resolution
* borrow checking
* lifetime/NLL analysis
* drop insertion
* canonical-form matching
* closure materialization
* match semantics
* effect system
* code generation

The parser should preserve syntax sufficient for these future passes, but must not perform them.

