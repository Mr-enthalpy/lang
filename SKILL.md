# Skill: lang Frontend Work

## Purpose

This skill defines how to work on the `lang` repository. The v0.1 Raw AST
frontend is completed. v0.1.w is closed. The current active stage is
`v0.2` — Raw AST Contract Freeze / Normalization Boundary Preparation.

In `v0.2`, the lexer/parser architecture, public frontend interfaces, Raw AST
shape, dump formats, and golden-test expectations are frozen contract material.
Work is documentation reconciliation, contract freezing, consistency repair,
version/stage metadata alignment, and preparation of the boundary that v0.3
Normalized AST Specification will consume. Additions must extend existing
lexer/parser entry points and AST preservation categories; they must not
replace the product/pipe/operator/binding/closure/navigation architecture.

The only accepted outputs for implementation work are tokens, AST, and
diagnostics.

## Workflow

Follow this workflow for every change:

0. **Gate new tasks against existing PR branches**
1. **Read the specs**
2. **Decide what to implement**
3. **Edit code + update specs together**
4. **Add golden tests**
5. **Run verification commands**
6. **Commit**

---

## 1. Which spec files to read first

### Default public frontend context

| Priority | File | When to read |
|---|---|---|
| 1 | `AGENTS.md` | Always, before any code change |
| 2 | `README.md` | Repository orientation |
| 3 | `spec/README.md` | Spec index and authority levels |
| 4 | `spec/public/v0.2/lexical-syntax-v0.2.md` | Public lexical syntax |
| 5 | `spec/public/v0.2/concrete-syntax-v0.2.md` | Public concrete syntax |
| 6 | `spec/public/v0.2/diagnostics-recovery-v0.2.md` | Public diagnostics and recovery |
| 7 | `spec/public/v0.2/raw-ast-frozen-surface-v0.2.md` | Frozen Raw AST construct inventory |
| 8 | `spec/reference/glossary.md` | Terminology reference |

### Task-specific extended context

| Task | Files to read |
|---|---|
| Parser implementation changes | `spec/implementation/v0.1/ast-construction-v0.1.md`, `spec/implementation/v0.1/implementation-status-v0.1.md` |
| Diagnostic implementation changes | `spec/implementation/v0.1/diagnostics-v0.1.md` |
| Operator syntax or operator-name repair | `spec/history/v0.1/operator-design.md`, `spec/public/v0.2/concrete-syntax-v0.2.md` |
| Alias / EntityRef repair | `spec/future/entity-alias-design.md`, `spec/future/entity-ref-design.md`, `spec/public/v0.2/concrete-syntax-v0.2.md` |
| v0.3 handoff / normalization-boundary work | `spec/contracts/raw-ast-contract-v0.1.md`, `spec/contracts/raw-ast-contract-freeze-v0.2.md`, `spec/contracts/v0.3-normalization-handoff-checklist.md`, `spec/public/v0.2/raw-ast-frozen-surface-v0.2.md` |
| Future build/package/namespace work | `spec/future/build-system-design.md`, `spec/future/package-manifest-v0.md`, `spec/future/namespace-assembly-v0.md` |
| Resolving old open questions / design history | `spec/history/v0.1/resolved-questions.md`, `spec/planning/open-questions.md`, `spec/history/v0.1/frontend-v0.1.md` |
| Forward-looking scope changes | `spec/planning/roadmap.md` |

## 1a. New-task PR branch gate

Before starting a new task that is not correction feedback for the current PR,
check whether the current branch belongs to an already-open or already-merged
PR.

Preferred local helper:

```powershell
powershell -ExecutionPolicy Bypass -File .git/local/pr-task-gate.ps1
```

The helper lives under `.git/local/` so it is local-only and must not be
committed or uploaded.

Rules:

- If the current branch is the remote default branch, fast-forward it from
  origin before starting the new task.
- If the current branch's PR is merged, switch to the default branch,
  fast-forward it from origin, delete the local PR branch, and delete the
  remote PR branch if it still exists.
- If the current branch's PR is not merged, refuse the unrelated new task and
  ask the user to either keep working on that PR or merge/close it first.
- If branch or PR state is ambiguous, stop and ask for clarification.

Do not run this gate when the user is explicitly requesting corrections to the
current PR.

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
in let bindings). `require`/`pre`/`post`/`lifetime pre`/`lifetime post` are
active closure-head clauses parsed as raw `HeadClauseAst` shape (one expression
slot each, no semantic validation). `acquire` is an ordinary name.

The weak lexer treats unrecognized words as `Name` tokens. This does not
make those words language constructs.

## 2a. Parser owns shape, semantics owns meaning

The parser constructs and preserves raw AST shape. Semantic or meta-function
passes may later interpret preserved shapes. v0.1 must not add special AST
nodes just because a future built-in meta-function may understand a shape.

Parse left to right. Do not go back to reinterpret meaning.

## 2b. v0.2 contract freeze boundary

Treat the following as frozen contract material:

- lexer/parser skeleton
- `lex` / `parse`
- token dump, AST dump, and diagnostic dump
- Raw AST node categories already documented as v0.1 output
- diagnostics infrastructure
- hard form boundaries, weak lexer, product/product-extract architecture
- pipe/segment/operator-expression architecture, closure AST preservation
- inner-to-outer navigation, alias-let parser preservation
- `with { ... }` narrow payload grammar

Completed v0.1.w additions (preserved in the frozen surface):

- richer literal spellings: radix integers, scientific notation,
  digit separators, hexadecimal floats, ranked quote-boundary strings;
  literal-name adjacency as ordinary call/composition material
- `|> name { ... }` accepted as mechanical shorthand for `|> (_ name) { ... }`

Allowed v0.2 work:

- documentation consistency repair, stale comment cleanup
- version/stage metadata alignment
- contract freeze checklist, diagnostic/golden-test inventory sync
- correction of spec/code mismatches where implementation is settled truth
- narrowly scoped golden-test additions (frozen behavior not yet locked)
- no parser behavior change unless a hard correctness error is identified

Forbidden in v0.2:

- traditional call parsing, statement parsing, import/module/package syntax,
  semantic analysis, name/type/kind/operator/alias resolution, canonical
  matching, closure materialization, ownership/NLL/drop, interpretation,
  code generation, a general macro system, or major parser architecture rewrite
- Raw AST → Normalized AST implementation

If a task implies large parser redesign, refuse or narrow it unless the user
identifies a hard correctness error against the call-composition architecture.
Hard correctness means the current architecture cannot represent the intended
model, makes future normalization logically impossible, forces heuristic
semantic backtracking, contradicts the core pipe/product/operator/call-binding
architecture, or makes a documented invariant impossible to maintain.

## 3. Expected outputs

| Phase | Output | Dump format |
|---|---|---|
| Lexer | `Vec<Token>` | Hand-written, stable, golden-testable |
| Parser | `ProgramAst` | Hand-written, stable, golden-testable |
| Diagnostics | `Vec<Diagnostic>` | Hand-written, stable, golden-testable |

Do **not** use Rust `Debug` format for any dump output.

## 3a. Operator design awareness

Before changing expression parsing, read both `spec/implementation/v0.1/ast-construction-v0.1.md`
and `spec/history/v0.1/operator-design.md`.

Operator expression parsing is implemented as raw AST sugar, and operator names
are preserved in binder and innermost navigation-component positions. If the task is not
explicitly about a later operator phase, do not add operator lookup, lowering,
alias binding, or semantic validation.

Navigation order is inner-to-outer. The leftmost component is the innermost
selected symbol; the rightmost component is the outermost scope component. Raw
AST preserves source-order navigation components and performs no lookup.

## 4. AST construction order

Expression AST must be built according to:

```text
1. Parse atom bases.
2. Fold atom suffixes:
   - ::
   - .
   - .. name Product
3. Split PipeExpr at top-level |>.
4. Parse each Segment.
5. Preserve product forms.
6. Build final ExprAst.
```

Do not implement this as a traditional precedence parser.

## 5. Closure rules

- Bare `{ ... }` in atom position is `ClosureAst::InPlace`, not a block expression.
- `FnHead => { ... }` is explicit closure AST.
- `FnHead { ... }` without `=>` is invalid and is not reinterpreted.
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

For v0.2 Raw AST contract freeze work:
- Prefer documentation and contract alignment.
- Do not restructure the lexer/parser skeleton.
- Do not add syntax except under a hard-correctness-error exception. Richer
  literal spelling and local mechanical sugar are completed v0.1.w additions.
- Run `cargo fmt --all` after code or Rust doc-comment changes.
- Run `cargo test` after code, test, or snapshot changes.

For Raw AST contract work:
- Do not change parser behavior; document invariants in `spec/contracts/raw-ast-contract-v0.1.md`.
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

Minimum golden case groups (from `AGENTS.md`). The v0.1 baseline has golden
coverage for lexer, parser/AST, and diagnostics. During `v0.2`, update or add
golden cases only for narrow documentation-aligned corrections, richer literal
spellings, or local mechanical whole-shape sugar recognition:

```text
lexer/        names, symbols, comments, invalid, operators
parser/       let_simple, let_extract, pipe_basic, product_forms,
              dot_sugar, doubledot_sugar, closure_head_inline,
              closure_explicit, closure_head, match_style_expression,
              operator_expr, operator_binder, alias_let
diagnostics/  invalid_dot, invalid_doubledot, unclosed_group,
              unclosed_closure, invalid_product, invalid_operator,
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

## 10. PR creation

When publishing local changes, prefer `gh` over connector-based PR creation.

Default flow:

1. Run `gh --version` and `gh auth status`.
2. Inspect `git status -sb` and the diff before staging.
3. Create a task branch if currently on the default branch.
4. Commit intentionally, push with upstream tracking, then create a draft PR
   with `gh pr create --draft`.
5. Use connector PR creation only when the user explicitly asks for it or `gh`
   cannot perform the required operation.

## 11. Spec update rules

- When changing **parser behavior**: update `spec/implementation/v0.1/ast-construction-v0.1.md`.
- When changing **diagnostic behavior**: update `spec/implementation/v0.1/diagnostics-v0.1.md`.
- When adding **new terminology**: update `spec/reference/glossary.md`.
- When resolving an **open question**: update `spec/planning/open-questions.md`.
- Spec and code changes must be in the same commit.
