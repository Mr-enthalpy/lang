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
v0.5   — Normalized Surface Semantics Stabilization and Public Documentation Reset — completed public baseline
v0.6   — Build / Namespace Graph Bootstrap — started / partial vertical slice
v0.7   — Early Meta-Function Bootstrap — future
v0.8   — Type-to-Type Meta Construction Interpreter — future
v0.9+  — Resumed semantic design (canonical forms, pattern spaces, meta with control flow, type/kind, closure materialization, NLL, semantic prototype, HIR, codegen) — future
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

### v0.5 — Normalized Surface Semantics Stabilization and Public Documentation Reset — completed

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

### v0.6+ — Build, namespace graph, meta-functions, then resumed semantic design

v0.5 closes the normalized surface semantics. The next stages build the
infrastructure that the language's symbol graph and metaprogramming depend on,
before resuming the deferred semantic design.

Narrative:

- v0.5 closes normalized surface semantics.
- v0.6 builds package / namespace graph infrastructure.
- v0.7 introduces early meta-function lookup and expansion.
- v0.8 introduces a restricted type-to-type meta construction interpreter.
- Later stages resume canonical forms, pattern spaces, value-to-type and
  value-to-value meta, type/kind checking, closure materialization, ownership/NLL,
  the semantic prototype, HIR, and codegen.

The canonical detailed direction for v0.6–v0.8 is
`spec/future/early-meta-functions-and-namespace-graph.md`, building on
`spec/future/build-system-design.md`, `spec/future/namespace-assembly-v0.md`,
and `spec/future/package-manifest-v0.md`. Future field-projection and
injection-place constraints are recorded in
`spec/future/type-associated-function-objects-and-access-trees.md`.

Before formal meta object invocation can become stable, package/manifest records
must provide package identity, mount identity, export-surface boundaries, and
candidate provenance.

#### v0.6 — Build / Namespace Graph Bootstrap

**Goal**: a minimal working build system and a namespace graph world model.
The namespace graph is a persistent, diagnosable, transactional world object,
not a temporary file index. Every future phase (resolver, early meta, type
checker, policy, seal, IDE, cache, HIR lowering) shares this model. Names such
as `struct`, `assert`, `type`, `namespace`, `uint8`, `ref`, `share` enter as
ordinary `SymbolObject`s resolvable through the graph, not as hardcoded compiler
branches.

Must cover:

- package manifest skeleton
- source root / namespace root
- core package default mount
- namespace mount table
- physical namespace skeleton from directories
- implementation file as source fragment; file name does not contribute a
  namespace segment
- declared symbol harvesting
- SymbolObject model
- physical / declared / virtual `NamespaceNode` kind
- resolver returning a `SymbolObject`, not a string path
- provenance and diagnostic attachment
- role-aware child-name buckets: object/function role and namespace-subspace
  role; same-role conflicts are hard errors, while field functions may share
  names with projection namespace subspaces such as `ref` / `share`
- ordinary contribution restricted to direct children; deeper structure owned by
  the immediate direct child (no ordinary parent-to-descendant injection)
- no source-level import/use/include/module
- policy metadata slots on symbols, contexts, and namespace graph nodes,
  including minimal `PolicyEnv::Meta` resolver visibility filtering; full policy
  checking remains future work (see `spec/future/policy-visibility-symbols.md`)
- namespace graph is a persistent, diagnosable, transactional world model shared
  by all future phases (not a temporary scan or file index)
- conflict is a hard error by default; no merge / overlay / duplicate /
  overload-set semantics or package overlay in v0.6
- engineering invariants: snapshot + transaction delta discipline,
  symbol-identity-as-object, core bootstrap boundary, meta-expansion atomicity,
  phase-freeze vocabulary, no-bypass rule, invariant-targeted test philosophy
  (see `spec/future/early-meta-functions-and-namespace-graph.md` §"Namespace
  Graph World Model Invariants")

Non-goals: full version solving; remote package retrieval; lockfile
completeness; dynamic/static distribution distinction; full access-control
lattice; full policy checking; full type checking; full meta-function execution.

**Implementation status:** started. The `lang_build` crate implements the first
v0.6 vertical slice: API-level `BuildManifest`, `CompilationWorld`,
transactional `NamespaceGraphSnapshot` / `NamespaceDelta`, `NamespaceNode`,
`SymbolObject`, resolver contexts with a default core mount, source-root
collection, physical directory namespace skeletons, direct-child declaration
harvesting, role-aware child buckets, expectation-aware resolver lookup, core
bootstrap symbols, and invariant tests. It also includes a minimal early-meta
closure for `core::struct` / `core::assert` lookup so the world model can prove
generated type-associated namespaces are installed atomically. v0.7-prep has
implemented minimal policy-aware early-meta lookup and callable policy-plane
clarification: `PolicyEnv::Meta` is resolver visibility, not callable execution
permission, and generated field functions are `meta+runtime` visible symbols
with runtime-only bodies. Fields named `ref` / `share` are accepted as
object-role field functions that coexist with projection namespace subspaces.
This does **not** complete v0.7 or v0.8: only the narrow
`(uint8 a, uint8 b) |> struct` family is implemented, no full manifest parser,
package manager, type checker, policy checker, type-value equality, access-tree
construction, or general meta interpreter is present.

#### v0.7 — Early Meta-Function Bootstrap

**Goal**: implement the early meta-function call loop on the v0.6 namespace
graph, so an early meta target is found by the resolver, not by a parser /
normalizer special case.

Must cover:

- early meta-function lookup from the namespace graph
- closed `SyntaxObject` passing
- `assert` as a compile-time hard-check primitive
- `struct` as the first real globally visible meta-function object from the core
  namespace
- meta call replacement model
- `MetaExpansionResult` (replacement object / namespace delta / diagnostics /
  provenance)
- parent-to-child namespace injection rule; parent-to-descendant generation only
  as the closed meta exception
- generated child namespace installation; no arbitrary rewrite of parent /
  sibling / global namespace
- `struct` consumes AST by a private checker; failure is a meta hard error, not
  a parser / normalizer error
- policy fields on callable objects — symbol visibility policy, body-entry
  policy, and return-object policy represented distinctly; full projection and
  execution checking remain future work (see
  `spec/future/policy-visibility-symbols.md`)

Non-goals: general compile-time value execution; value-to-value meta-functions;
arbitrary control flow in meta bodies; full generic system; full pattern-space
semantics; HIR/codegen integration beyond placeholder nodes.

#### v0.8 — Type-to-Type Meta Construction Interpreter

**Goal**: the earliest, most restricted meta-function body execution model:
type → type, single entry / single exit, no intermediate control flow, pure
streaming structure. A meta body is the source file's already-produced Raw AST /
Normalized AST executed under a meta policy by a type-object construction
interpreter; it is not a separate DSL and not a text macro.

Must cover:

- meta function body as normalized AST
- type-object construction interpreter
- declaration-as-assignment / assignment-as-injection
- `let` inside a meta body creates symbols through the namespace graph capability
- `===` as symbol alias / forwarding, not copy
- explicit return object slot, e.g. `meta + runtime let r: type`
- `r = t` returns the generated object; `r === t` forwards an existing globally
  visible symbol
- generative meta identity = function symbol + canonical args + build/config
  fingerprint
- symbol shielding: the externally visible result name is determined by the meta
  function name + arguments, not internal temporary names
- generated declarations installed only under a legal parent / instance node
- first-class generic classes such as `Vec(T)`, `Option(T)`, `Pair(A, B)`
- awareness that meta body execution policy differs from function symbol policy
  and return-object policy; implement only the minimum checks needed to avoid
  misrepresenting meta-functions as runtime functions

Non-goals: value-to-type control flow; value-to-value compile-time world;
unrestricted compile-time IO; runtime execution; full borrow/lifetime checking;
full pattern-space subtraction / exhaustiveness; complete operator overload
semantics (the overload resolution pipeline is specified in
`spec/future/overload-resolution-design.md`; overload resolution is gated on
v0.10+ pattern-space infrastructure).

#### v0.9 — Canonical form specification

Define value/type canonical forms and universal extraction matching. Document the
relationship between deduce lists and canonical forms. Do not implement matching
yet.

#### v0.10+ — Pattern-space and extraction-chain semantics

Design pattern spaces as static objects generated by canonical pattern
constructors: sum patterns, structural pattern-space operations, extraction
chains, residual propagation, the `Done` isolation layer, explicit result
consumption, postfix `?`, and conventional closing consumers such as `match`.

This phase provides the pattern-space infrastructure that overload resolution
depends on: extraction-pattern specificity (§4 of `overload-resolution-design.md`)
requires construction-expression-tree depth scoring, which in turn requires
canonical pattern-space construction and extraction-chain matching. Overload
resolution is not implemented before this phase.

Before formal meta object invocation can select callables, an earlier pattern
normalization and first-order type-value candidate-preparation layer is needed;
see `spec/future/pattern-normalization-and-first-order-overload.md`.

The v0.4 normalizer only preserves the Normalized AST boundaries these phases
need: value-side material remains `NormExpr`, pattern-side material remains
`NormPattern`, annotations remain annotation patterns, branch names in extraction
position remain pattern material, and operator names remain unresolved structural
targets. Detailed design note:
`spec/future/static-pattern-spaces-and-extraction-chains.md`.

#### v0.11+ — Value-to-type meta-functions with control flow

Extend the meta-function model with value-to-type meta-functions that allow
control flow in meta bodies, beyond the v0.8 restricted type-to-type form.

#### Later stages

The following remain deferred and are not numbered precisely here:

- value-to-value compile-time meta execution
- type / kind checking integration
- closure materialization model (ClosureAST → ClosureObject; capture rules)
- ownership / NLL / drop / lifetime design (including any future semantics for
  `with { ... }`)
- full policy inference, projection checking, compile / runtime / seal semantics,
  const / mut policy, effect / error / panic policy, and resource capability
  policy (see `spec/future/policy-visibility-symbols.md`)
- first semantic compiler prototype integrating selected passes
- HIR
- code generation

---

## xtask

`xtask` is optional tooling, not part of v0.1 semantics. It exists as a
placeholder for build automation tasks. The workspace compiles without it
if removed.

## Build-system / namespace-graph track

The build system assembles a namespace graph from package manifests, directory
structure, and source fragments. The source language has no
import/use/include/module syntax; source code refers directly to mounted
namespace paths.

This track was previously documented as a parallel side-track. As of the v0.6+
re-sequencing it is the active implementation stage: v0.6 — Build / Namespace
Graph Bootstrap (see the v0.6 stage above and
`spec/future/early-meta-functions-and-namespace-graph.md`). The current code is
a partial vertical slice in `crates/lang_build`, not a complete build system.

### Scope discipline

Build/package work is still gated out of the completed v0.1–v0.5 frontend /
normalizer. It must not change the lexer, parser, Raw AST, or Normalized AST,
and it must not introduce source-level import/use/include/mod/package/export
syntax. Namespace resolution, dependency solving, and declaration indexing are
v0.6+ work, implemented under the v0.6–v0.8 stage boundaries, not retrofitted
into the frontend.

### Deferred within v0.6–v0.8

- full version solving, remote package retrieval, lockfile completeness
- dynamic/static distribution distinction, full access-control lattice
- full policy checking, full type checking
- full (value-level) meta-function execution
