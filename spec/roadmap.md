# Roadmap

This document defines the stage model for the `lang` compiler. It
distinguishes implementation stages from semantic research stages.

Stages before v1.0 may overlap in time. The boundaries are scope boundaries,
not strict chronological gates.

## Stage model

```
v0.1 — Raw AST Frontend — completed
v0.1.w — Raw AST Stability Window — current
v0.2+ — only after an explicit future decision
v0.3 — Normalized AST Specification — not active until v0.1.w is closed
v0.4 — Raw AST → Normalized AST Prototype
v0.5 — Normalized AST Stabilization
v0.6+ — Later semantic design stages
```

Raw AST is surface-preserving and non-desugared.
Normalized AST is desugared but still non-semantic.
HIR is later than Normalized AST.
Type checking is later than Normalized AST.
Name resolution is later than Normalized AST.
Canonical matching is later than Normalized AST.
Closure materialization is later than Normalized AST.
NLL/drop insertion is later than Normalized AST.

---

### v0.1 — Raw AST Frontend — completed

**Goal**: `source text → tokens → Raw AST → diagnostics`

v0.1 delivered a complete Raw AST frontend with lexer, parser, AST,
dumper, diagnostics, and golden tests.

**What v0.1 delivered:**

A syntax frontend that:

- Lexes source text into tokens (Name, Literal, Symbol, Trivia, Invalid, Eof).
- Parses tokens into raw AST (forms, lets, expressions, closures, canonical
  skeletons, deduce lists).
- Handles errors gracefully (produces ErrorAst + diagnostic, continues).
- Dumps all three outputs (tokens, AST, diagnostics) in stable, hand-written
  formats suitable for golden testing.

**v0.1 completed deliverables:**

- Crate `lang_syntax` with lexer, parser, AST, dumper, diagnostics.
- Crate `lang_cli` with CLI subcommands: `tokens`, `ast`, `diag`.
- Golden test suite covering all syntax rules.
- Specification documents for AST construction and diagnostics.
- Operator expression parsing as raw AST sugar.
- Operator names in binder and innermost navigation-component positions.
- Alias binding (`let binder === EntityRef`) as raw AST preservation.
- EntityRef parser for alias RHS.
- 31 DiagnosticCode variants across lexer, parser, operator, and alias categories.

For the authoritative factual inventory of v0.1 delivered features,
see `spec/implementation-status-v0.1.md`. For the Raw AST contract
that future normalization passes may rely on, see
`spec/raw-ast-contract-v0.1.md`.

---

### v0.1.w — Raw AST Stability Window — current

**Goal**: Stabilize the completed v0.1 Raw AST frontend contract without
turning the stage into broad parser expansion.

`v0.1.w` is a maintenance and contract-stabilization window. The lexer/parser
architecture, public frontend interfaces, Raw AST shape, dump formats, and
golden-test expectations are stable by default.

Stable by default:

- lexer/parser skeleton
- Raw AST categories
- `lex` / `parse`
- token dump, AST dump, and diagnostic dump
- diagnostics infrastructure
- hard form boundaries
- weak lexer
- product/product-extract architecture
- pipe/segment/operator-expression architecture
- closure AST preservation
- inner-to-outer navigation
- alias-let parser preservation
- `with { ... }` narrow payload grammar

Allowed additive work:

- richer literal spellings, including scientific numeric notation, radix
  notation, numeric separators, richer string literal spellings, escape syntax,
  literal-adjacent unit spelling when defined as syntax only, and similar
  lexical / Raw-AST-preserving additions
- local, mechanical, whole-form sugar recognition, only when triggered by a
  finite explicit token shape, preserved as Raw AST, requiring no lookup or
  semantic validation, changing no existing source meaning, and avoiding
  lexer/parser skeleton restructuring

Forbidden in `v0.1.w`:

- semantic analysis
- name resolution
- type/kind checking
- operator lookup
- alias target resolution
- closure materialization
- canonical matching
- ownership/NLL/drop
- import/package/module syntax
- traditional call syntax
- general macro system
- major parser architecture rewrite

Large parser refactors are forbidden unless there is a hard correctness error:
the current architecture cannot represent the intended call-composition model,
future normalization is logically impossible, the grammar forces heuristic
semantic backtracking, current accepted syntax contradicts the core
pipe/product/operator/call-binding architecture, or a documented invariant is
impossible to maintain without structural correction.

Historical note: the earlier "v0.2 Raw AST Contract Freeze — reopened for hard
corrections" wording described a transition after the initial v0.1 baseline.
Those corrections have been folded into the completed Raw AST frontend and the
current active stage is `v0.1.w`, not an open-ended v0.2 parser revision phase.

---

### v0.2+ — only after an explicit future decision

No active `v0.2` implementation or parser-revision phase exists. Any `v0.2+`
work requires an explicit future decision that defines scope, deliverables, and
phase gates.

---

### v0.3 — Normalized AST Specification — not active until v0.1.w is closed

**Goal**: Define the Normalized AST node set and document how Raw AST
constructs desugar into Normalized AST.

Normalized AST unifies:

- call/product forms (product, pipe, operator sugar) into simple normalized nodes
- extraction forms (canonical skeletons, deduce lists) into pattern nodes
- declaration forms (simple let, extract let, alias let) into declaration nodes

Define:

- Normalized form for let bindings, preserving optional `with { ... }` syntax
  without lifetime semantics, and unifying simple/extract.
- Normalized form for pipe expressions (flattened segments, preserved product placement).
- Normalized form for operator sugar (lowered to named operator calls).
- Normalized form for closure heads (canonicalized clause order).
- Normalized form for canonical skeletons (pattern representation, not matching).
- Normalized form for member/double-dot selector sugar.
- Normalized form for alias bindings (preserved as unresolved entity references).

This is a design/specification stage. Do not implement Normalized AST yet.

Normalized AST is **not** HIR. It is desugared but still non-semantic.

---

### v0.4 — Raw AST → Normalized AST Prototype

**Goal**: Implement a Raw AST → Normalized AST lowering pass.

- Implement `normalize.rs` in `lang_syntax` or a new crate.
- Produce golden-tested Normalized AST dumps.
- Each desugaring rule from v0.3 should have at least one golden test.

The output is a Normalized AST, not a type-checked or name-resolved tree.

**Do not implement** name resolution, type checking, operator lookup,
alias resolution, canonical matching, or closure materialization in this
lowering pass.

---

### v0.5 — Normalized AST Stabilization

**Goal**: Harden the Normalized AST representation and the normalization pass.

- Error recovery through normalization (do not crash on malformed input).
- Diagnostic rewiring (map Raw AST error spans into Normalized AST context).
- Property-based testing for normalization invariants.
- AST dump stability for Normalized AST.

**No new semantic features.**

---

### v0.6+ — Later semantic design stages

The following stages are deferred beyond Normalized AST. They are listed
here for scope boundary reference only. None are implemented.

#### v0.6 — Canonical form specification

Define value/type canonical forms and universal extraction matching.
Document the relationship between deduce lists and canonical forms.
Do not implement matching yet.

#### v0.7 — Meta-function boundary specification

Document compiler-provided meta-functions (`match`, `effect`, `sync`) that
consume AST or normalized syntax. Do not implement any meta-function.

#### v0.8 — Closure materialization model

Document ClosureAST → ClosureObject conversion. Define capture rules,
materialization defaults, and object transferability.

#### v0.9 — Type/kind checking design

Design and document kind/type checking. Define kind system, checking rules,
and bidirectional/deductive checking algorithm.

#### v0.10 — Ownership/NLL/drop design

Design ownership, always-NLL, and drop semantics. Define any future lexical or
dependency semantics for `with { ... }` and user-defined drop/move points.

#### v0.11 — First semantic compiler prototype

Begin integrating selected semantic passes: type checking, kind checking,
canonical-form evaluation, closure materialization, meta-function invocation.
Each pass must be individually gated.

---

## xtask

`xtask` is optional tooling, not part of v0.1 semantics. It exists as a
placeholder for build automation tasks. The workspace compiles without it
if removed.

## Build-system track (parallel)

The build-system track is a parallel documentation and architecture track
inside this repository. It is not part of v0.1 frontend implementation.

The build system assembles a namespace graph from package manifests, directory
structure, and source fragments. The source language has no
import/use/include/module syntax; source code refers directly to mounted
namespace paths.

### Phase gates

Now that v0.1 Raw AST Frontend is completed, build-system documentation may
continue as a parallel design track. Build/package implementation remains
gated and must not introduce namespace resolution, dependency solving,
declaration indexing, or semantic imports unless explicitly assigned in a
later stage.

### Non-goals for this track in v0.1

- No build manifest parser implementation.
- No dependency solver.
- No namespace resolver.
- No lockfile generator.
- No cache validator.
- No declaration indexer.
- No source-level import/use/include/mod/package/export syntax.
