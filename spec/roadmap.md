# Roadmap

This document defines the stage model for the `lang` compiler. It
distinguishes implementation stages from semantic research stages.

Stages before v1.0 may overlap in time. The boundaries are scope boundaries,
not strict chronological gates.

## Stage model

```
v0.1 — Raw AST Frontend — completed
v0.2 — Raw AST Contract Freeze — current
v0.3 — Normalized AST Specification
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
- Operator names in binder and final path-leaf positions.
- Alias binding (`let binder === EntityRef`) as raw AST preservation.
- EntityRef parser for alias RHS.
- 31 DiagnosticCode variants across lexer, parser, operator, and alias categories.

For the authoritative factual inventory of v0.1 delivered features,
see `spec/implementation-status-v0.1.md`. For the Raw AST contract
that future normalization passes may rely on, see
`spec/raw-ast-contract-v0.1.md`.

---

### v0.2 — Raw AST Contract Freeze — current

**Goal**: Document the invariants of the completed v0.1 Raw AST so that
future normalization passes can safely desugar it.

**Deliverable**: `spec/raw-ast-contract-v0.1.md`

This document defines what future normalization passes may rely on from
Raw AST. It lists invariants for every AST node category and explicitly
states what normalization must not assume (name resolution, type checking,
operator lookup, alias resolution, etc.).

**No new syntax or parser behavior.** The Raw AST contract is a
documentation-only deliverable.

---

### v0.3 — Normalized AST Specification

**Goal**: Define the Normalized AST node set and document how Raw AST
constructs desugar into Normalized AST.

Normalized AST unifies:

- call forms (ArgPack, pipe, operator sugar) into simple call nodes
- extraction forms (canonical skeletons, deduce lists) into pattern nodes
- declaration forms (simple let, extract let, alias let) into declaration nodes

Define:

- Normalized form for let bindings, preserving guard/with as declaration
  attributes/dependencies without lifetime semantics, and unifying simple/extract.
- Normalized form for pipe expressions (flattened segments, desugared ArgPack roles).
- Normalized form for operator sugar (lowered to named operator calls).
- Normalized form for closure heads (canonicalized clause order).
- Normalized form for canonical skeletons (pattern representation, not matching).
- Normalized form for member/double-dot/numeric selector sugar.
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

Design ownership, always-NLL, and drop semantics. Define guard/with semantics
in let bindings and user-defined drop/move points.

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

- **After parser phase 2**: build-system documentation and architecture track
  may start (manifest design, namespace mount model, assembly pipeline docs).
- **In parallel with parser phase 3**: build manifest and physical namespace
  skeleton design may proceed.
- **After parser phase 3**: parser-backed declaration indexing may begin.
- **Post-v0.1**: semantic namespace resolver, visibility checking, version
  solving, cache validation, and virtual namespace expansion may begin.

### Non-goals for this track in v0.1

- No build manifest parser implementation.
- No dependency solver.
- No namespace resolver.
- No lockfile generator.
- No cache validator.
- No declaration indexer.
- No source-level import/use/include/mod/package/export syntax.
