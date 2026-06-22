# Skill: lang Frontend Work

## Purpose

This skill defines how to work on the `lang` repository. The v0.1 Raw AST
frontend is completed. Current work is Normalized AST specification (v0.3); v0.2 Raw AST Contract Freeze is completed and
Normalized AST design.

The only accepted outputs for implementation work are tokens, AST, and
diagnostics.

## Workflow

Follow this workflow for every change:

1. **Read the specs**
2. **Decide what to implement**
3. **Edit code + update specs together**
4. **Add golden tests**
5. **Run verification commands**
6. **Commit**

---

## 1. Which spec files to read first

| Priority | File | When to read |
|---|---|---|
| 1 | `AGENTS.md` | Always, before any code change |
| 2 | `README.md` | Repository orientation |
| 3 | `spec/frontend-v0.1.md` | Pipeline understanding |
| 4 | `spec/implementation-status-v0.1.md` | Current implementation inventory |
| 5 | `spec/raw-ast-contract-v0.1.md` | Raw AST invariants for normalization |
| 6 | `spec/ast-construction-v0.1.md` | Before any parser change |
| 7 | `spec/operator-design.md` | Before any operator syntax change |
| 8 | `spec/entity-ref-design.md` | Before any EntityRef or alias RHS change |
| 9 | `spec/entity-alias-design.md` | Before any alias-binding change (parser preservation implemented, semantics future) |
| 10 | `spec/diagnostics-v0.1.md` | Before any diagnostic change |
| 11 | `spec/glossary.md` | Terminology reference |
| 12 | `spec/roadmap.md` | Scope boundary check |
| 13 | `spec/open-questions.md` | Before touching uncertain areas |

## 2. Core invariant

Do not interpret semantic names in the lexer or parser.

The following are ordinary names in v0.1:

```text
return  else  match  drop  move  ref
sync  effect  fn  type  meta  runtime  compile
namespace  struct
```

Parser contexts may interpret selected names structurally, but only when
explicitly defined by the spec (e.g., `let` at form start and `with { ... }`
in let bindings). `where` and `acquire` are reserved future closure-head
positions, not active parser clauses in Phase 3.1.

The weak lexer treats unrecognized words as `Name` tokens. This does not
make those words language constructs.

## 2a. Parser owns shape, semantics owns meaning

The parser constructs and preserves raw AST shape. Semantic or meta-function
passes may later interpret preserved shapes. v0.1 must not add special AST
nodes just because a future built-in meta-function may understand a shape.

Parse left to right. Do not go back to reinterpret meaning.

## 3. Expected outputs

| Phase | Output | Dump format |
|---|---|---|
| Lexer | `Vec<Token>` | Hand-written, stable, golden-testable |
| Parser | `ProgramAst` | Hand-written, stable, golden-testable |
| Diagnostics | `Vec<Diagnostic>` | Hand-written, stable, golden-testable |

Do **not** use Rust `Debug` format for any dump output.

## 3a. Operator design awareness

Before changing expression parsing, read both `spec/ast-construction-v0.1.md`
and `spec/operator-design.md`.

Operator expression parsing is implemented as raw AST sugar, and operator names
are preserved in binder/final-path-leaf positions. If the task is not
explicitly about a later operator phase, do not add operator lookup, lowering,
alias binding, or semantic validation.

## 4. AST construction order

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

## 5. Closure rules

- Bare `{ ... }` in atom position is not closure AST and not a block expression.
- `FnHead => { ... }` is explicit closure AST.
- `FnHead { ... }` is inline closure AST.
- A closure literal is AST first. It is not a callable object.

## 6. `<>` rules

- `<...>` is a deduce list only in strong binding contexts.
- Outside binding contexts, `<` and `>` are ordinary symbols.

## 7. What is out of scope

If a requested task requires any of the following, stop at AST preservation:

| Feature | How to handle in v0.1 |
|---|---|
| `match` | Parse the expression shape; do not implement match checking |
| `return` | Parse as a name; do not implement return semantics |
| `else` | Parse as a name; do not implement else branching |
| `drop` / `move` | Parse as names; do not mark blue nodes |
| `with { ... }` | Preserve syntax in AST; do not run lifetime analysis |
| Type/rank checking | Preserve bare annotations exactly; parse explicit `type_object_annotation : rank_annotation`; do not check semantic validity |
| `fn f(x) { }` | Parse as an expression form or emit syntax diagnostics as required; never create FnDecl |
| Canonical matching | Build canonical skeleton AST; do not execute matching |
| Closure materialization | Build closure AST; do not create callable objects |
| Effect / sync | Parse as names; do not interpret effect system |
| Library / import / export | v0.1 has no such syntax; preserve raw :: paths only |
| Entity alias binding (`let binder === EntityRef`) | Implemented as raw AST parser preservation; do not add target resolution, operator identity validation, or alias semantics |
| Meta-function AST consumption | Built-in privilege; do not generalize to user macros |
| `struct` / field syntax | Not parser syntax; future built-in meta-function may consume raw AST |

## 7a. Phase boundaries

For Raw AST work:
- Edit parser, lexer, tests, and specs as usual.
- Run `cargo fmt --all` and `cargo test` after changes.

For Raw AST contract work:
- Do not change parser behavior; document invariants in `spec/raw-ast-contract-v0.1.md`.
- Update the contract when parser changes affect AST shape.

For Normalized AST work:
- First update or create Normalized AST specs before implementing code.
- Normalized AST may desugar syntax but must not resolve names, infer types,
  evaluate canonical forms, materialize closures, or insert drops.
- Do not call Normalized AST "HIR".

## 8. What tests to add

Every syntax rule requires a golden test with:

1. Input source file in `tests/cases/<category>/`.
2. Test function in the corresponding `tests/*_golden.rs` file.
3. Expected output snapshot.

Minimum golden case groups (from `AGENTS.md`). Golden tests are added
incrementally as parser phases are implemented; the case groups below are the
full v0.1 target. The current test suite covers parser phase 1, phase 2
binding-context syntax, phase 3 closure AST, and phase 3.1 closure/parser
stabilization (see `spec/roadmap.md` for current coverage):

```text
lexer/        names, symbols, comments, invalid, operators
parser/       let_simple, let_extract, pipe_basic, argpack_roles,
              dot_sugar, doubledot_sugar, closure_head_inline,
              closure_explicit, closure_head, match_style_expression,
              operator_expr, operator_binder, alias_let
diagnostics/  invalid_dot, invalid_doubledot, unclosed_group,
              unclosed_closure, invalid_argpack, invalid_operator,
              invalid_alias
```

## 9. Commands to run

```bash
# Format all code
cargo fmt --all

# Check compilation
cargo check --workspace

# Run all tests
cargo test
```

## 10. Spec update rules

- When changing **parser behavior**: update `spec/ast-construction-v0.1.md`.
- When changing **diagnostic behavior**: update `spec/diagnostics-v0.1.md`.
- When adding **new terminology**: update `spec/glossary.md`.
- When resolving an **open question**: update `spec/open-questions.md`.
- Spec and code changes must be in the same commit.
