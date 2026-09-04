# Skill: `lang` repository work

## Purpose

Use this workflow for frontend, semantic-substrate, documentation, and test
changes in `lang`.

## Required context

Read:

```text
AGENTS.md
README.md
spec/README.md
spec/public/normalized-surface-semantics.md
spec/public/agent-interpretation-guide.md
spec/contracts/raw-ast-contract.md
spec/planning/open-questions.md
```

For semantic work, follow the topic-owner map in `spec/design/README.md`. Read
`spec/planning/roadmap.md` for implementation coverage. History is not an
input to current semantic design.

## Workflow

1. Gate unrelated work against the current PR branch.
2. Read the current public contract and relevant topic owners.
3. State the invariant and implementation boundary.
4. Edit code, current docs, and tests together.
5. Run formatting and the focused tests.
6. Run the full workspace tests.
7. Inspect the diff, commit intentionally, and update the active PR.

## Boundaries

- The lexer remains weak.
- Raw AST preserves syntax and recovery.
- Normalized AST performs syntax-directed lowering only; it is not HIR.
- Name resolution, Pattern interpretation, type observation, overload
  selection, DynamicLegality, execution, and lifecycle are semantic stages.
- A missing canonical consumer stays unavailable; it does not use another
  semantic implementation.
- Open representation questions remain in
  `spec/planning/open-questions.md`.

## Frontend construction

Expression structure is built in this order:

```text
atom base
-> atom suffixes (::, ., ..name Product, bracket forms)
-> top-level pipe split
-> segment structure
-> Product preservation
-> ExprAst
```

Do not turn this into conventional callee-first call parsing. Preserve
value/Pattern separation, closure placement, DeduceList binding, PolicyLet,
control-flow end events, and origin information exactly as specified.

## Verification

```bash
cargo fmt --all
cargo check --workspace
cargo test
```

Golden dumps are stable authored formats; do not use Rust `Debug` output as a
public dump format.
