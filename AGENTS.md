# Agent instructions for `lang`

## Read first

For every task, read:

```text
README.md
spec/README.md
spec/public/normalized-surface-semantics.md
spec/public/agent-interpretation-guide.md
spec/contracts/raw-ast-contract.md
spec/planning/open-questions.md
```

For semantic work, also read `spec/design/README.md` and every canonical topic
owner named there for the concepts being changed. For implementation
sequencing, read `spec/planning/roadmap.md`.

Documents under `spec/history/**` are non-authoritative and are read only when
the user explicitly asks for historical analysis.

## Subagents

Use subagents as read-only scouts for broad cross-file searches, independent
verification, and large peripheral modules. Read foundational documents and
the exact code you will modify yourself.

- Give each scout a self-contained scope and request `file:line` evidence.
- Spawn with `fork_turns = "none"`; do not reuse or retask a scout.
- After spawning, wait until every relevant scout reaches a terminal state.
- Treat results as compressed leads; verify decisive locations directly.
- The primary agent owns edits, design choices, and final verification.

## Architecture

```text
source text
  -> weak tokens
  -> Raw AST
  -> Normalized AST
  -> typed owner / namespace resolution
  -> canonical semantic evaluation
  -> InvocationResult
```

Raw AST preserves source and recovery. Normalized AST is syntax-directed and
non-semantic; it is not HIR. Semantic meaning never feeds back into lexing,
parsing, or normalization.

The current semantic universe is defined only by the canonical topic owners.
If a closed relation has no connected consumer, leave that operation
unsupported or return the appropriate Diagnostic/Residual. Do not invent an
alternate relation or identity.

## Frontend invariants

- The lexer is weak. Contextual words are `Name` tokens.
- The parser owns syntax shape, not semantic meaning.
- Parse left to right without semantic backtracking.
- Traditional `f(args)` call syntax does not exist.
- Products participate in the documented expression/call-binding skeleton.
- `{ ... }` in atom position is an in-place closure with no head.
- A headed closure without `=>` is in-place; `=>` forms an ordinary closure.
- `<...>` is a DeduceList only in documented strong binding contexts.
- `let <> P` is binderless Pattern material; `let _ P` contains a wildcard.
- `|> P { ... }` uses the binderless headed in-place closure shape.
- Value-side expressions and Pattern-side material remain distinct.
- `let binder === EntityRef` is syntax preservation only until its local
  lexical resolver consumer is connected; it creates no semantic entity.
- `return`, `else`, `match`, `if`, `drop`, `move`, `sync`, `effect`, `fn`,
  `type`, `meta`, `runtime`, `compile`, `namespace`, and `struct` are not lexer
  keywords.
- The parser must not create semantic declarations such as `FnDecl`,
  `StructDecl`, `ImportDecl`, HIR, MIR, or codegen nodes.
- Invalid input should produce AST recovery nodes plus spanned diagnostics.

## Canonical semantic invariants

- `Object = <Val1?, Pattern, Val2>`; ordinary normalization observes all three.
- Pattern applicability and extraction come from `R_Gamma(P,c,rho)`.
- `tau = bind alpha.<Core(tau), V_tau[alpha]>`; `V_tau` is immutable.
- NameBinding, named type, OverloadGroup, Place, and TypeValueId are distinct.
- Same-name construction synthesizes a type's V_tau; ordinary lexical let does
  not aggregate. Structural P let name::path installs complete empty T_0 before
  returning mut type ref; NameBinding is structural, not a wrapper Object.
- Type +=/-= changes only V_tau under Writable, OpenHere and final closure-type
  membership. Witnessed anchored replication never reparents an existing value.
- OverloadGroup aggregation has its own bucket algebra and requires Writable.
  A keys identify existing construction subjects, not Core-equality classes;
  its indexed group places recheck that same subject's OpenHere at write Pre.
- Name resolution happens once before context projection.
- Calls use value -> exact tau -> associated `()` and one candidate space.
- `PolicyMode = {const, plain, mut}`; plain is a primitive point.
- Policy preference, CapabilityRealization, Writable, and DynamicLegality are
  independent judgments.
- Output demand is total before maxima; selected failure never reopens.
- Policy migration is direct, same-Type, candidate-driven, and existing-first.
- Abstract literals form before concrete construction.
- `OpenHere`, Writable, PolicyMode, and construction authority do not imply one
  another. `extend` is pure; `inject` is read + extend + write.
- `InvocationResult` is the single semantic result envelope; `struct` returns
  complete tau.
- Lifecycle facts are relative to one SemanticContinuation. Cleanup is fixed
  before observation; Pre precedes mutation; Post describes committed success.
- Color vocabulary is extensible and relation rows are explicit and directed.
- SafetyPolicy is orthogonal to PolicyMode; unsafe admits compatible external
  semantic axioms, never missing Pre facts or private optimizer assumptions.
- Child-directory names normalize to ordinary fresh-name actions followed by
  body evaluation under the returned type ref; root and filenames add no segment.
- Host capabilities return ordinary Objects. Physical normalization and build
  facilities introduce no semantic facts; E alone owns meaning.
- E is idempotent and saturates ready actions without rewriting continuations.
  Optimizer rewrites require revalidation by affected semantic projections.
- Formal Pv:Pp inherits P2; omitted formal mode is plain. Implicit return targets
  the outermost enclosing function layer.

## Scope and Open questions

Do not close a question listed in `spec/planning/open-questions.md` for
implementation convenience. Use opaque carriers and extension interfaces.

Source wiring may be incomplete. Missing wiring means unavailable behavior,
not permission to substitute another semantic implementation.

## Editing

- Preserve unrelated user changes in a dirty worktree.
- Use `rg` / `rg --files` for searches.
- Use `apply_patch` for edits; bulk mechanical renames may use repository-safe
  file operations.
- Update current contracts and tests with parser or diagnostic behavior.
- Do not rewrite files under `spec/history/**` to describe current behavior.
- Current source, tests, and docs use only positive current terminology.

## Tests

Every syntax rule needs golden coverage. Semantic changes need positive,
negative, identity/equality, no-reopen, non-derivability, and
authority-uniqueness tests as relevant.

After changes run:

```text
cargo fmt --all
cargo test
```

## PR hygiene

For a new unrelated task, run `.git/local/pr-task-gate.ps1` when available. Do
not run it for corrections to the current PR.

When asked to publish changes, inspect status/diff, commit intentionally, push
with upstream tracking, and use `gh` for draft PR creation or updates.
