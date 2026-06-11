# File: SKILL.md

# Skill: lang v0.1 Frontend Work

## Purpose

This skill defines how to work on the `lang` repository during the `v0.1` frontend stage.

The only accepted output of the compiler frontend at this stage is:

```text
tokens
AST
diagnostics
```

No semantic compilation work belongs in this stage.

## Before editing

Read these files first:

```text
AGENTS.md
README.md
spec/ast-construction-v0.1.md
```

When behavior is unclear, update the spec before or together with code.

## Core invariant

Do not interpret semantic names in the lexer or parser.

The following are ordinary names in v0.1:

```text
return
else
match
drop
move
ref
sync
effect
fn
type
meta
runtime
compile
```

Parser contexts may interpret selected names structurally, but only when explicitly defined by the spec.

## Lexer task

The lexer must produce:

```text
Name
Literal
Symbol
Trivia
Invalid
Eof
```

The lexer must preserve spans.

The lexer must use longest match for compound symbols:

```text
..
::
|>
=>
->
```

The lexer must not emit keyword tokens.

## Parser task

The parser must produce AST.

The parser must not:

* type-check
* resolve names
* lower to IR
* evaluate canonical forms
* execute universal extraction
* materialize closures
* recognize match as syntax
* insert drops
* interpret return/else as control flow

## Main parser components

Recommended parser modules:

```text
parser/cursor.rs
parser/form.rs
parser/let_stmt.rs
parser/expr.rs
parser/pipe.rs
parser/argpack.rs
parser/atom.rs
parser/closure.rs
parser/canonical.rs
parser/recovery.rs
```

## AST construction order

Expression AST must be built according to:

```text
1. Parse atom bases.
2. Fold atom suffixes:
   - ::
   - .
   - .. name ArgPack
3. Split PipeExpr at top-level |>.
4. Parse each Segment.
5. Assign ArgPack roles.
6. Build final ExprAst.
```

Do not implement this as a traditional precedence parser.

## Closure rule

`{ ... }` in atom position is closure AST.

It is not a block expression.

`FnHead => { ... }` is explicit closure AST.

`FnHead { ... }` is inline closure AST.

A closure literal is AST first. It is not a callable object until a later semantic pass materializes it.

## `<>` rule

`<...>` is a deduce list only in a strong binding context.

It declares holes for following syntax.

Outside binding contexts, `<` and `>` are ordinary tokens.

## Testing rule

Any behavior change requires a golden test.

Use separate cases for:

```text
tokens
AST
diagnostics
```

Do not rely on Rust `Debug` format for AST snapshots. Use a stable dump function.

## Recommended commands

```bash
cargo fmt --all
cargo test
```

or:

```bash
make test
```

## Out of scope

If a requested task requires semantic analysis, stop at AST preservation.

Examples:

* For `match`, parse the expression shape. Do not implement match checking.
* For `return`, parse it as a name. Do not implement return.
* For `drop` or `move`, parse them as names. Do not mark blue nodes.
* For `guard` and `with`, preserve annotations in AST. Do not run lifetime analysis.

