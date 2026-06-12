# Agent Instructions for `lang`

## Read these files first

Before making any code changes, read:

```text
AGENTS.md              (this file)
README.md              (repository overview)
SKILL.md               (operational workflow)
spec/frontend-v0.1.md  (pipeline overview)
spec/ast-construction-v0.1.md  (normative parser rules)
spec/diagnostics-v0.1.md       (normative diagnostic rules)
spec/roadmap.md        (scope boundaries)
spec/glossary.md       (terminology)
spec/open-questions.md (known gaps)
```

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

If a change requires any of the above, stop at syntax/AST representation and
leave the semantic behavior as a documented future pass.

## Required commands

After code changes, run:

```bash
cargo fmt --all
cargo test
```

If the workspace is not initialized yet, create the minimal Rust workspace
first, then make these commands valid.

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

The lexer must not classify names such as `return`, `else`, `match`, `drop`,
`move`, `sync`, `effect`, `fn`, `type`, `meta`, `runtime`, `compile`,
`namespace`, `mod`, or `struct` as special keyword tokens.

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

Parenthesized argument packs participate only in the expression skeleton rules
described in `spec/ast-construction-v0.1.md`.

### Blocks and closures

`{ ... }` is not a normal block expression.

In expression/atom position, `{ ... }` produces a closure AST.

Closure literals initially produce AST, not callable objects. Object
materialization is a future semantic pass.

### Control-flow names

Do not add syntax nodes such as:

* `ReturnStmt`
* `ElseExpr`
* `MatchExpr`

At `v0.1`, `return`, `else`, and `match` remain ordinary names and ordinary
expression atoms unless some later semantic pass interprets them.

### Match

`match` is not syntax in `v0.1`.

A future compiler-provided meta-function named `match` may consume closure AST
arms, but parser code must not special-case the name `match`.

### Declaration model

All user-visible declarations in v0.1 enter through `let`.

There is no dedicated parser syntax for:

```text
fn f(...) { ... }
type T = ...
namespace ns = ...
```

Do not invent semantic AST nodes such as `FnDecl`, `TypeDecl`, or
`NamespaceDecl`.

At parser level, `fn`, `type`, `namespace` remain ordinary `Name` tokens except
in documented strong annotation contexts within `let` binders.

Declaration annotations (`: type`, `: _ : fn`, `: fn`) are parsed and
preserved but not semantically checked.

### `struct` and field declarations

`struct` is not parser syntax. A future built-in meta-function named `struct`
may consume raw AST and return a type-object. This is a semantic/meta-function
behavior, not a parser rule.

The parser must not introduce `StructDecl`, `FieldDecl`, `MemberDecl`,
`BitfieldDecl`, `LayoutDecl`, or similar semantic AST nodes in v0.1.

### Parser owns shape, semantics owns meaning

The parser constructs and preserves raw AST shape. It does not decide whether
an AST fragment is a field, a struct member, a namespace object, a function
declaration, a return statement, a match arm, or an import. Semantic or
meta-function passes may later interpret preserved shapes.

v0.1 must not add special AST nodes just because a future built-in
meta-function may understand a shape.

Parse left to right. Do not go back to reinterpret meaning. The parser should
be streaming-friendly. Contextual parsing is allowed only for explicitly
specified strong syntax contexts. Semantic meaning must not feed back into
v0.1 parsing.

### Privileged AST-consuming meta-functions

Some future built-in meta-functions may consume raw AST directly. Examples may
include future built-ins such as `match`, `struct`, `effect`, and `sync`.

Accepting raw AST as input is a privileged capability of built-in
meta-functions. User-defined functions must not be assumed to have unrestricted
AST-consuming power in v0.1.

AST-consuming meta-functions are built-in privileges until the language is
stable enough to define a controlled user-facing macro/metaprogramming system.
v0.1 only preserves AST shape; it does not decide which functions may consume
AST.

### No library/import/export/package syntax

v0.1 has no library, import, export, module, or package syntax. The parser
only preserves raw namespace path syntax such as `std::Vec`,
`mylib::math::vector::Vec3`, and `((int)std::Vec)::ns1` where expressible by
raw AST rules.

Do not create AST nodes such as `ImportDecl`, `UseDecl`, `IncludeDecl`,
`ModuleDecl`, `LibraryDecl`, `PackageDecl`, or `ExportDecl`.

## Repository layout

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
    ├── parser_golden.rs
    ├── diagnostics_golden.rs
    └── cases/
        ├── lexer/
        ├── parser/
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

Refer to `spec/diagnostics-v0.1.md` for the full diagnostic catalog.

## Tests

Every syntax rule must have golden tests.

Minimum case groups:

```text
lexer/
  names
  symbols
  comments
  invalid

parser/
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

When changing diagnostic behavior:

1. Update `spec/diagnostics-v0.1.md`.
2. Update or add golden tests.
3. Run `cargo fmt --all`.
4. Run `cargo test`.

Do not change parser or diagnostic behavior without updating the corresponding
spec or tests.

## Spec awareness

* `spec/roadmap.md` defines scope boundaries. If a change would cross a stage
  boundary (e.g., implementing semantic analysis), stop and document the
  limitation instead.
* `spec/open-questions.md` records unresolved design issues. Before implementing
  behavior that touches an open question, check the file and update its status
  if a decision is reached.
* `spec/glossary.md` enforces terminology. Use terms consistently.
