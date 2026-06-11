# File: AGENTS.md

# Agent Instructions for `lang`

## Scope

This repository is currently in the `v0.1` frontend stage.

The only goal of `v0.1` is:

```text
source text -> tokens -> AST -> diagnostics
```

Do not implement:

* type checking
* kind checking
* overload resolution
* canonical-form evaluation
* universal extraction matching
* closure AST materialization into callable objects
* match/effect/sync semantics
* ownership, lifetime, NLL, drop insertion
* interpretation
* code generation
* IR/HIR/MIR/lowering beyond raw AST construction

If a change requires any of the above, stop at syntax/AST representation and leave the semantic behavior as a documented future pass.

## Required commands

After code changes, run:

```bash
cargo fmt --all
cargo test
```

If the workspace is not initialized yet, create the minimal Rust workspace first, then make these commands valid.

The project should keep a single command path for agents:

```bash
make test
```

which should delegate to `cargo test`.

## Preferred technology

Use Rust stable.

Use:

* a hand-written lexer
* a hand-written parser
* golden/snapshot tests for tokens, AST, and diagnostics

Do not introduce parser generators in `v0.1`.

Do not introduce semantic crates such as:

* `typeck`
* `nll`
* `borrowck`
* `hir`
* `mir`
* `codegen`

The first workspace should contain only syntax/frontend-related crates.

Suggested workspace:

```text
crates/
  lang_syntax/
  lang_cli/
spec/
tests/
```

## Core design constraints

### Lexer

The lexer must remain semantically weak.

It should output tokens such as:

* `Name`
* `Literal`
* `Symbol`
* `Trivia`
* `Invalid`
* `Eof`

The lexer must not classify names such as `return`, `else`, `match`, `drop`, `move`, `sync`, `effect`, `fn`, `type`, `meta`, `runtime`, or `compile` as special keyword tokens.

These are ordinary names at the lexical level.

### Contextual structure words

Some names may be interpreted by the parser in strong contexts.

Examples:

* `let` at form start introduces a let binding.
* `where` and `acquire` may delimit closure-head clauses.
* `guard` and `with` may be interpreted inside a let-binding context.

Outside the relevant parser state, these names remain ordinary names.

### `<>`

`<...>` has exactly one special meaning:

```text
declare holes for following syntax in a strong binding context
```

It is recognized only in specific binding contexts, such as:

* extract-let binder
* closure head
* parameter binder
* return binder

Outside these contexts, `<` and `>` are ordinary tokens.

### Calls

Traditional call syntax does not exist in `v0.1`.

Do not parse:

```text
f(args)
```

as a normal function call.

Parenthesized argument packs participate only in the expression skeleton rules described in `spec/ast-construction-v0.1.md`.

### Blocks and closures

`{ ... }` is not a normal block expression.

In expression/atom position, `{ ... }` produces a closure AST.

Closure literals initially produce AST, not callable objects. Object materialization is a future semantic pass.

### Control-flow names

Do not add syntax nodes such as:

* `ReturnStmt`
* `ElseExpr`
* `MatchExpr`

At `v0.1`, `return`, `else`, and `match` remain ordinary names and ordinary expression atoms unless some later semantic pass interprets them.

### Match

`match` is not syntax in `v0.1`.

A future compiler-provided meta-function named `match` may consume closure AST arms, but parser code must not special-case the name `match`.

## Repository layout

Preferred layout:

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
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── source.rs
│   │       ├── span.rs
│   │       ├── token.rs
│   │       ├── lexer.rs
│   │       ├── ast.rs
│   │       ├── dump.rs
│   │       ├── diagnostic.rs
│   │       └── parser/
│   │           ├── mod.rs
│   │           ├── cursor.rs
│   │           ├── form.rs
│   │           ├── let_stmt.rs
│   │           ├── expr.rs
│   │           ├── atom.rs
│   │           ├── pipe.rs
│   │           ├── argpack.rs
│   │           ├── closure.rs
│   │           ├── canonical.rs
│   │           └── recovery.rs
│   └── lang_cli/
│       ├── Cargo.toml
│       └── src/main.rs
└── tests/
    ├── lexer_golden.rs
    ├── ast_golden.rs
    ├── diagnostics_golden.rs
    └── cases/
        ├── lexer/
        ├── ast/
        └── diagnostics/
```

## AST policy

AST must preserve syntax rather than interpret semantics.

For example:

```text
obj (
    <val: _>(val option::Sum) { ... },
    (_ option::None) { ... }
) match
```

The parser should produce ordinary expression structure containing:

* `Name("obj")`
* an `ArgPack`
* closure AST arms
* `Name("match")`

It should not produce a special `MatchExpr`.

## Diagnostics policy

The parser should be error-tolerant.

Prefer:

```text
AST with ErrorNode + Diagnostic
```

over aborting the parse.

Every diagnostic must carry a span.

## Tests

Every syntax rule must have golden tests.

Minimum case groups:

```text
lexer/
  names
  symbols
  comments
  invalid

ast/
  let_simple
  let_extract
  pipe_basic
  argpack_roles
  dot_sugar
  doubledot_sugar
  closure_inline
  closure_explicit
  closure_head
  match_style_expression

diagnostics/
  invalid_dot
  invalid_doubledot
  unclosed_group
  unclosed_closure
  invalid_argpack
```

## Commit discipline

When changing parser behavior:

1. Update `spec/ast-construction-v0.1.md`.
2. Update or add golden tests.
3. Run `cargo fmt --all`.
4. Run `cargo test`.

Do not change parser behavior without updating the corresponding spec or tests.

