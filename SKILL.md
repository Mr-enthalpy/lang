# Skill: lang v0.1 Frontend Work

## Purpose

This skill defines how to work on the `lang` repository during the `v0.1`
frontend stage. The only accepted outputs are tokens, AST, and diagnostics.

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
| 4 | `spec/ast-construction-v0.1.md` | Before any parser change |
| 5 | `spec/diagnostics-v0.1.md` | Before any diagnostic change |
| 6 | `spec/glossary.md` | Terminology reference |
| 7 | `spec/roadmap.md` | Scope boundary check |
| 8 | `spec/open-questions.md` | Before touching uncertain areas |

## 2. Core invariant

Do not interpret semantic names in the lexer or parser.

The following are ordinary names in v0.1:

```text
return  else  match  drop  move  ref
sync  effect  fn  type  meta  runtime  compile
namespace  mod  struct
```

Parser contexts may interpret selected names structurally, but only when
explicitly defined by the spec (e.g., `let` at form start, `where`/`acquire`
in closure heads, `guard`/`with` in let bindings).

## 3. Expected outputs

| Phase | Output | Dump format |
|---|---|---|
| Lexer | `Vec<Token>` | Hand-written, stable, golden-testable |
| Parser | `ProgramAst` | Hand-written, stable, golden-testable |
| Diagnostics | `Vec<Diagnostic>` | Hand-written, stable, golden-testable |

Do **not** use Rust `Debug` format for any dump output.

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

- `{ ... }` in atom position is closure AST, not a block expression.
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
| `guard` / `with` | Preserve annotations in AST; do not run lifetime analysis |
| Type checking | Parse kind annotations; do not check them |
| `fn f(x) { }` | Parse as expression; do not create FnDecl or distinguish from ordinary let |
| Canonical matching | Build canonical skeleton AST; do not execute matching |
| Closure materialization | Build closure AST; do not create callable objects |
| Effect / sync | Parse as names; do not interpret effect system |

## 8. What tests to add

Every syntax rule requires a golden test with:

1. Input source file in `tests/cases/<category>/`.
2. Test function in the corresponding `tests/*_golden.rs` file.
3. Expected output snapshot.

Minimum golden case groups (from `AGENTS.md`):

```text
lexer/        names, symbols, comments, invalid
parser/       let_simple, let_extract, pipe_basic, argpack_roles,
              dot_sugar, doubledot_sugar, closure_inline,
              closure_explicit, closure_head, match_style_expression
diagnostics/ invalid_dot, invalid_doubledot, unclosed_group,
             unclosed_closure, invalid_argpack
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
