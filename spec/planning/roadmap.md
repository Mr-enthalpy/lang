# Roadmap

This document defines the stage model for the `lang` compiler. It
distinguishes implementation stages from semantic research stages.

Stages before v1.0 may overlap in time. The boundaries are scope boundaries,
not strict chronological gates.

## Stage model

```
v0.1   — Raw AST Frontend — completed
v0.1.w — Raw AST Stability Window — closed
v0.2   — Raw AST Contract Freeze / Public Frontend Syntax Specification — closed
v0.3   — Normalized AST Specification — completed specification baseline
v0.4   — Raw AST → Normalized AST Prototype / Hardening — completed
v0.5   — Normalized Surface Semantics Stabilization and Public Documentation Reset — active
v0.6+  — Later semantic design stages — future
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
- 29 DiagnosticCode variants across lexer, parser, operator, and alias categories.

For the authoritative factual inventory of v0.1 delivered features,
see `spec/implementation/v0.1/implementation-status-v0.1.md`. For the Raw AST contract
that future normalization passes may rely on, see
`spec/contracts/raw-ast-contract-v0.1.md`.

---

### v0.1.w — Raw AST Stability Window — closed

`v0.1.w` was a maintenance and contract-stabilization window that repaired and
completed the remaining Raw AST stability-window questions. During this window:

- Richer literal spelling was implemented (radix integers, digit separators,
  scientific notation, hexadecimal floats, ranked quote-boundary strings).
- The pipe branch-name shorthand (`|> name { ... } ⇝ |> (_ name) { ... }`)
  was accepted as the only local mechanical whole-shape sugar.
- The final current-stage open question was closed.

`v0.1.w` is now complete. The project then entered v0.2; v0.2 is now closed.

---

### v0.2 — Raw AST Contract Freeze / Public Frontend Syntax Specification — closed

v0.2 froze the Raw AST contract and prepared the v0.3 handoff boundary.
The following deliverables were completed during v0.2:

---

### v0.3 — Normalized AST Specification — completed specification baseline

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

v0.3 completed the Normalized AST specification baseline. Implementation of the
Raw AST → Normalized AST lowering followed in v0.4.

Normalized AST is **not** HIR. It is desugared but still non-semantic.

---

### v0.4 — Raw AST → Normalized AST Prototype / Hardening — completed

**Goal**: Implement and harden a Raw AST → Normalized AST lowering pass.

v0.4 delivered:

- Raw AST → Normalized AST lowering loop.
- A stable normalized dump and a CLI normalized dump path.
- Golden tests and structural invariant tests.
- Boundary hardening and error recovery through normalization.
- Explicit `Unsupported` visibility (unsupported Raw AST subshapes remain
  visible in the dump instead of being silently erased).
- Value-side `NormExpr` / pattern-side `NormPattern` boundary preservation.

The output is a Normalized AST, not a type-checked or name-resolved tree. The
v0.4 normalization boundary is recorded in
`spec/contracts/v0.4-normalization-prototype-notes.md`.

v0.4 did **not** implement name resolution, type checking, operator lookup,
alias resolution, pattern-head resolution, canonical matching, or closure
materialization.

---

### v0.5 — Normalized Surface Semantics Stabilization and Public Documentation Reset — active

**Goal**: Turn the v0.4 prototype/hardening result into a stable public
documentation structure and stabilize the normalized surface semantics that are
already implemented.

v0.5 turns the v0.4 result into a stable public documentation structure:

- history absorbs route / design / discussion material;
- public docs explain current language behavior;
- agent docs explain how to interpret source without importing C / Rust / Python
  call assumptions;
- future docs retain v0.6+ semantic designs.

v0.5 is still **non-semantic** in the later-compiler sense. It stabilizes the
normalized surface semantics and the public documentation. It does **not**
implement type checking, name resolution, operator lookup, pattern-head
resolution, HIR, closure materialization, runtime evaluation, or code
generation.

Future pattern-space and extraction-chain semantics (see
`spec/future/static-pattern-spaces-and-extraction-chains.md`) motivate the
current normalized boundaries, but they are **not** implemented by the v0.5
normalizer. `Done`, residual propagation, pattern-space subtraction, `operator+`
meta-reduction, `match` closing, and pattern-head resolution are not current
behavior.

v0.5 proceeded in incremental PRs. v0.5-1 established the documentation
authority structure and the stage reset; v0.5-2 published the normalized
call / product / pipe binding semantics; v0.5-3 published the value-side /
pattern-side / annotation / alias boundary semantics; v0.5-4 closes the public
documentation reset by moving route/design material toward history and
finalizing the public documentation status. The public normalized surface
semantics are published.

The current public v0.5 documentation entry point is `spec/public/v0.5/`.

---

### v0.6+ — Later semantic design stages

The following stages are deferred beyond Normalized AST. They are listed
here for scope boundary reference only. None are implemented.

#### v0.6 — Canonical form specification

Define value/type canonical forms and universal extraction matching.
Document the relationship between deduce lists and canonical forms.
Do not implement matching yet.

#### v0.6+ — Pattern-space and extraction-chain semantics

Design pattern spaces as static objects generated by canonical pattern
constructors. This track covers sum patterns, structural pattern-space
operations, extraction chains, residual propagation, the `Done` isolation
layer, explicit result consumption, postfix `?`, and conventional closing
consumers such as `match`.

This is later semantic design. The v0.4 normalizer only preserves the
Normalized AST boundaries needed by these phases: value-side material remains
`NormExpr`, pattern-side material remains `NormPattern`, annotations remain
annotation patterns, branch names in extraction position remain pattern
material, and operator names remain unresolved structural targets.

Do not implement pattern-space construction, structural subtraction,
`Done` insertion or elimination, `operator+` meta-reduction, result
consumption checking, exhaustiveness checking, pattern-head resolution, or
closed-control-pattern non-additivity during normalization.

Detailed design note:
`spec/future/static-pattern-spaces-and-extraction-chains.md`.

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
