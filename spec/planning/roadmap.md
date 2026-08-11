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
v0.8   — Compile / Symbol Construction Interpreter Bootstrap — future
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

The current v0.6 slice includes typed semantic-owner identity, callable and
meta-instance owner nesting, Pattern-root alpha boundaries, owner-qualified
hole handoff, Full/External/DefaultExtraction views, explicit package-boundary
metadata, identity-preserving mount redirects, and narrow struct-member
visibility carriers. It does not claim persistent manifest parsing, general
name resolution, custom `?`, closure materialization, or backend integration.

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
- 32 DiagnosticCode variants across lexer, parser, return, operator, and alias categories.

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
alias-target resolution, pattern-head resolution, canonical matching, or closure
materialization. (Alias-target resolution has since been retired as a semantic
direction; `LetAliasAst` remains frozen parser material only.)

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
`spec/design/patterns-overload/static-pattern-spaces-and-extraction-chains.md`) motivate the
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

#### v0.5-A — versioned frontend semantic amendment

Later v0.5 semantic-surface work required parser and Raw AST changes. Those
changes do not rewrite the closed v0.1/v0.2/v0.3 documents. They are versioned
by:

```text
spec/contracts/frontend-semantic-amendment-v0.5-a.md
spec/contracts/raw-ast-contract-v0.5.md
```

The amendment classifies closure placement orthogonalization as a hard
structural correction, first-class DotClosure as a normalization-driven
extension, and let-shaped capture binding, Ellipsis/Pack (including canonical
Sequence children), plus callable-tail alternatives as new syntax amendments.
It also records the intentional contraction from arbitrary delete-message
expressions to string literals. The current syntax crate version is `0.5.0`;
the v0.2 freeze remains a historical 19-Symbol/32-diagnostic snapshot.

---

### v0.6+ — Build, namespace graph, meta-functions, then resumed semantic design

v0.5 closes the normalized surface semantics. The next stages build the
infrastructure that the language's symbol graph and metaprogramming depend on,
before resuming the deferred semantic design.

Narrative:

- v0.5 closes normalized surface semantics.
- v0.6 builds package / namespace graph infrastructure.
- v0.7 introduces early meta-function lookup and expansion.
- v0.8 evolves the restricted type-shaped evaluator toward `compile`
  `PatternValue` computation and `meta` ordinary-Symbol construction; the
  current `SymbolConstruction` is an implementation carrier only.
- Later stages resume canonical forms, pattern spaces, value-directed
  compile/meta control flow, type/kind checking, closure materialization,
  ownership/NLL, the semantic prototype, HIR, and codegen.

The canonical detailed direction for v0.6–v0.8 is
`spec/design/symbol-world/early-meta-functions-and-namespace-graph.md`, building on
`spec/design/build-package/build-system-design.md`, `spec/design/build-package/namespace-assembly-v0.md`,
and `spec/design/build-package/package-manifest-v0.md`. Future field-projection and
extension-place constraints are recorded in
`spec/design/symbol-world/type-associated-function-objects-and-access-trees.md`.

Before formal meta object invocation can become stable, package/manifest records
must provide package identity, mount identity, export-surface boundaries, and
candidate provenance.

The active design route (documented under `spec/design/`) is:

```text
package/manifest identity
  -> namespace graph / SymbolCell (current substrate: SymbolObject)
  -> SymbolId / PlaceId / PatternValue / TypeValueId / borrow views
  -> ProductObject / ArgProductShape
  -> pattern normalization + first-order candidate shapes
  -> compile PatternValue computation
  -> ordinary container PatternValues: T*N / T*omega / product / Symbol
  -> meta Symbol-valued construction (current SymbolConstruction is substrate)
  -> ResolvedPatternScope / struct -> symbol / pure extend / place inject
  -> let-only creation + existing-place writes + NamespaceDelta install
  -> formal invocation demand/policy integration
  -> mechanical lowering family
  -> later runtime lookup
  -> first-order type check
```

Two distinctions matter for sequencing:

- Pattern/overload work is split. The earlier `pattern normalization +
  first-order type candidate adaptation` subset serves the formal meta object
  invocation model and comes before it. The later, fuller runtime overload
  resolution remains further out and is not required for meta invocation.
- Runtime lookup and first-order type checking are deliberately later than the
  pattern / type-value / meta-invocation work; they consume its results rather
  than re-deriving them.

The mechanical lowering family (see `spec/design/mechanical-lowering/`) includes:

```text
automatic argument passing
automatic return normalization / error policy
call mode insertion: normal / tco / loop
```

### Documentation fusion note

`spec/design/` is a temporary staging area introduced to break up the former
flat `spec/future/` pile. It is not intended to become the permanent home of
the symbol / pattern / meta-invocation semantics. As those semantics stabilize:

- user-visible stable behavior moves to `spec/public/`;
- implementation-stage obligations and handoff invariants move to `spec/contracts/`;
- sequencing, deferrals, and open scope remain in `spec/planning/`;
- superseded alternatives and absorbed ADR material move to `spec/history/`.

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
- namespace facet creation has exactly one origin: physical directory, one
  source construction unit, or one canonical meta construction unit
- physical directory namespaces define contribution authority: only files in
  that directory may create their direct contents
- each source file is one closed construction unit; parallel files may create
  distinct direct children but may not reopen one another's namespace/type/
  pattern/value subtrees
- declared symbol harvesting
- SymbolObject model
- physical / declared / virtual `NamespaceNode` kind
- resolver returning a `SymbolObject`, not a string path
- provenance and diagnostic attachment
- role-aware child-name buckets: object/function role and namespace-subspace
  role; same-role conflicts are hard errors. This remains generic graph
  substrate and is not a semantic `ref` / `share` projection-space model
- ordinary source authority begins at direct children; one source unit may fully
  construct a new child subtree in its own delta, while parallel files may not
  reopen that subtree
- no source-level import/use/include/module
- policy metadata slots on symbols, contexts, and namespace graph nodes,
  including the legacy flat resolver adapter now mapped onto OpenStatic,
  SealStatic, and Runtime visibility; full pair storage on every entry and
  end-to-end checking remain future work (see
  `spec/design/policy-capability/policy-visibility-symbols.md`)
- a bounded cross-Policy implementation prototype: T/Tnum literal helper,
  existing-view-first projection over mixed result collections,
  projection-only absent entries, runtime-branch extraction after a complete
  choice projects empty, a prototype ordinary-result-shaped carrier, typed
  Runtime Val1 legality, callable-owned mutability endpoints using the existing
  actual-relative ordinary preference rather than hard domain intersection,
  a fixture for the future pre-Bp fallback strategy, complete ordinary result
  Policy separated from the demanded output view, and transition endpoint input × output Policy
  before Pattern specificity without transitive search; the endpoint-only
  maxima helper is private and typed qualification keeps delete rejection
  distinct from availability
- namespace graph is a persistent, diagnosable, transactional world model shared
  by all future phases (not a temporary scan or file index)
- conflict is a hard error by default; no merge / overlay / duplicate /
  overload-set semantics or package overlay in v0.6
- current cross-file closure forbids type-child, namespace-child, ordinary
  value-member, and overload-entry extension into an existing symbol; value
  overload union may be reconsidered only after explicit merge authority and
  stable candidate identity are designed
- engineering invariants: snapshot + transaction delta discipline,
  symbol-identity-as-object, core bootstrap boundary, meta-expansion atomicity,
  phase-freeze vocabulary, no-bypass rule, invariant-targeted test philosophy
  (see `spec/design/symbol-world/early-meta-functions-and-namespace-graph.md` §"Namespace
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
implemented policy-aware resolver visibility and callable policy-plane
clarification: `PolicyEnv` is resolver visibility, not callable execution
permission, and generated field functions are `meta+runtime` visible symbols
with runtime-only bodies. The current implementation's legacy projection nodes
are transitional; target field access uses one same-name associated Symbol with
`T` / `T ref` / `T share` receiver candidates.
The crate also implements a bounded cross-Policy demand preparer: ordinary omitted P1
continues to preserve the complete RHS, explicit P1 first uses the canonical
non-empty projection rule, absent entries lack transition capability without
invalidating value-bearing siblings. Only after the complete query projects
nothing can an accepted runtime alternative be extracted as the derived
runtime-only target and paired with an eligible static input view after
Pattern-Policy stage/domain slicing. Separate helpers cover T/Tnum literal
selection, prototype ordinary-result-shaped fixtures, typed runtime failures,
callable-owned mutability endpoints, and an endpoint-product fixture before
its Pattern-specificity stand-in. Opposite const/mut endpoint Patterns remain
admissible and reuse ordinary `matching > unspecified > opposite` preference.
The four default transport members in the toolchain-source fixture declare complete
`(compile || runtime):compile` input/output Policies; `Project_in` and
`Project_out` select the compile and runtime views around the ordinary result.
Current source has no fallback role (`D = A`). The prototype fixture verifies
that if/when such a future declaration-side strategy exists, suppression runs
after full admissibility and before Bp', and an admissible non-fallback delete
suppresses fallback without retry. A distinct future call-site annotation acts
before candidate generation; only this pipeline position is closed here.
The retained endpoint-only maxima helper is private. The connected
`PreparedCallCandidate` path now composes the implemented ordinary
formal/phase coordinates and optional migration endpoint coordinates in one
Bp' product before a single maxima pass.
Policy migration cannot repair Type/Pattern structural failure; explicit
`ref`/`share`/`rebind` mechanical operations remain separate ordinary calls.
T and Tnum registries
carry current first-order
TypeValue projections derived from installed Type symbols, not final canonical
type-value identity. The implemented consumer is binding P1; future consumers
must project the complete accepted choice first, then may construct its
runtime branch only when the complete existing projection is empty.
Source callables are now installed as one semantic Symbol with heterogeneous
function-object values. Each value reaches its TypeValue, PatternValue owner,
associated `()`, `InvocationFrame`, ordinary candidate pipeline, and complete
result entries. Pattern-owner-authorized calls enter the same trunk with an
explicit semantic receiver. A typed `ToolchainGlobalSourceRoot` supplies
source-visible root construction authority (`Gsrc`); ordinary packages retain
a non-empty install prefix and cannot install direct root members. `Gsrc` is not
a prelude and cross-package calls still require public visibility.

Atomic Runtime migration can consume a checked request through source-backed
associated `()` members installed under the existing `uint8` Pattern owner.
The legacy `PolicyTransitionCallable` path remains algebra/test fixture
material rather than a second resolver. Binding-P1 source lowering now calls
this connected migration adapter after complete existing-view projection
fails and the demand accepts runtime. Other consumer kinds do not yet share a
consumer-neutral demand satisfier.

This does **not** complete v0.7 or v0.8: only the narrow
`(uint8 a, uint8 b) |> struct` family is implemented, no full manifest parser,
package manager, type checker, policy checker, final cross-build canonical
type-value equality, access-tree
construction, general runtime overload resolution, consumer-neutral
parameter/result migration routing, runtime lowering, or general
meta interpretation is present. Mixed-stage Policy-domain existence,
phase-dependent readability, early binding/evaluation of compile-readable
dependencies, runtime residualization, and continuation of the same resolved
invocation are fixed. Residual IR, effect sequencing, residual-frame physical
representation, continuation ABI, and capability/effect composition remain
open.

#### v0.7 — Early Meta-Function Bootstrap

**Goal**: implement the early meta-function call loop on the v0.6 namespace
graph, so an early meta target is found by the resolver, not by a parser /
normalizer special case.

Must cover:

- early meta-function lookup from the namespace graph
- closed `SyntaxObject` passing
- `assert` as a compile-time hard-check primitive
- `struct` as the first real globally visible
  `BuiltinPrivilegedAstMetaFunction` object from the core namespace, producing
  a Symbol with one type member plus generated partner families
- current meta call replacement adapter
- current `MetaExpansionResult` transport (replacement object / namespace delta /
  diagnostics / provenance); final formal invocation returns an uninstalled
  construction and outer binding performs delta installation
- an ordinary canonical meta invocation owns one closed
  `MetaConstructionUnit` and may build its complete virtual subtree without
  cross-unit reopening; compiler-known privileged AST meta functions use only
  their separately bounded construction capability
- generated child namespace installation; no arbitrary rewrite of parent /
  sibling / global namespace
- `struct` consumes AST by a private checker; failure is a meta hard error, not
  a parser / normalizer error
- policy fields on callable objects retained as transitional symbol,
  body-entry, and result metadata; final source semantics use canonical
  `Pv:Pp`, contextual P1 projection, P2 result normalization, and no independent
  `P3`. Parameters may refine inherited P2 mutability, and returns symmetrically
  may refine inherited P1 mutability; neither may alter other policy dimensions.
  The typed pair substrate now exists, while migration of every legacy
  `PolicySet` consumer and end-to-end execution checking remain future work (see
  `spec/design/policy-capability/policy-visibility-symbols.md`)

Non-goals: general `compile` PatternValue execution; value-directed meta construction;
arbitrary control flow in meta bodies; full generic system; full pattern-space
semantics; HIR/codegen integration beyond placeholder nodes.

#### v0.8 — Compile / Symbol Construction Interpreter Bootstrap

**Goal**: evolve the earliest restricted type-shaped evaluator toward the
canonical value-level `compile` and symbol-level `meta` capabilities. Bodies
consume the source file's already-produced structured AST/Normalized AST under
policy; this is not a separate DSL or text macro.

Must cover:

- implement the symbol-first construction boundary defined in
  `spec/design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md`;
- implement namespace origin and construction ownership in the order defined by
  `spec/design/symbol-world/symbol-construction-units-and-namespace-origin.md`;
- preserve the future layered-policy boundary defined in
  `spec/design/symbol-world/symbol-policy-and-compile-flow-projection.md` while
  deferring full compile-flow projection, companions, and automatic require;
- keep normalized body material, current policy metadata, canonical instance keys, and
  outer atomic installation as explicit stage boundaries;
- first-class generic classes such as `(T Vec)`, `(T Option)`, `(A, B Pair)`
- preserve canonical policy as `Pv:Pp`: `P1` is contextual binding projection,
  `P2` is result-pair normalization, and function-object stage views are
  derived from `P2`; current flat symbol/body/result fields remain transitional;
- preserve legal runtime bindings and keep any non-runtime
  projection-source premise local to its specific compile-determined rule;
- preserve typed policy dimensions (stage, value mutability, value presence,
  ordinary namespace visibility, and export-root) rather than flattening atoms;
- preserve three phases: OpenStatic exposes meta/compile, SealStatic exposes
  seal/compile, Runtime exposes runtime values; privileged seal scans consume
  fixed Wpre and never Wseal

Before implementing ordinary generic type-style meta-functions, the v0.8
construction contract must be absorbed:
`spec/contracts/v0.8-meta-construction-agent-constraints.md`. The following are
preconditions, not optional local conveniences: `ProductObject` /
`ArgProductShape`, `PatternValue` / `TypeValueId` / `PlaceId` / borrow views,
transitional `SymbolConstruction` transport / `ResolvedPatternScope`, contextual P1 projection,
P2 pair normalization and function-object stage derivation while preserving
current metadata transport,
canonical meta instance key, and `NamespaceDelta` atomic install. This does not
make a full generic system, full overload resolution, or full type checker a
v0.8 requirement.

Non-goals: unrestricted/general compile-time execution; unrestricted
compile-time IO; runtime execution; full borrow/lifetime checking; full
pattern-space subtraction / exhaustiveness; complete operator overload semantics
(the overload resolution pipeline is specified in
`spec/design/patterns-overload/overload-resolution-design.md`; overload resolution is gated on
v0.10+ pattern-space infrastructure).

#### v0.9 — Canonical form specification

Define value/type canonical forms and universal extraction matching. Document the
relationship between deduce lists and canonical forms. Do not implement matching
yet.

#### v0.10+ — Pattern-space and extraction-chain semantics

Design pattern spaces as static objects generated by canonical pattern
constructors: sum patterns, structural pattern-space operations, extraction
chains, residual propagation, the `Done` isolation layer, explicit result
consumption, postfix `?` as a one-layer top-Pattern view, and conventional
closing consumers such as `match`.

This phase provides the pattern-space infrastructure that overload resolution
depends on: extraction-pattern specificity (§4 of `overload-resolution-design.md`)
requires construction-expression-tree depth scoring, which in turn requires
canonical pattern-space construction and extraction-chain matching. Overload
resolution is not implemented before this phase.

Before formal meta object invocation can select callables, an earlier pattern
normalization and first-order type-value candidate-preparation layer is needed;
see `spec/design/patterns-overload/pattern-normalization-and-first-order-overload.md`.

Automatic return normalization and `noerror` / `Error`-handler semantics require
first-order type predicates and policy-aware invocation, so they are future work
after the meta invocation model; see
`spec/design/mechanical-lowering/mechanical-return-normalization-and-error-policy.md`.

Future first-order lowering also needs explicit call-mode insertion
(`normal` / `tco` / `loop`) for recursion-based repetition, since the language
has no loop core; see `spec/design/mechanical-lowering/call-modes-recursion-and-tail-lowering.md`.

The v0.4 normalizer only preserves the Normalized AST boundaries these phases
need: value-side material remains `NormExpr`, pattern-side material remains
`NormPattern`, annotations remain annotation patterns, branch names in extraction
position remain pattern material, and operator names remain unresolved structural
targets. Detailed design note:
`spec/design/patterns-overload/static-pattern-spaces-and-extraction-chains.md`.

#### v0.11+ — Value-directed compile/meta control flow

Extend `compile` PatternValue computation and `meta` ordinary-Symbol
construction with value-directed control flow beyond the v0.8 restricted
bootstrap. `SymbolConstruction` remains an implementation substrate until it is
lowered into that ordinary value domain.

This later track owns implementation planning for mechanical compile-flow
projection over ordinary call nodes, complete derived `Val2` compile-companion
objects, fully admissible overload preparation, the
`must_select_if_qualified` overload strategy, intrinsic D/Done match flow,
coarse inferred-require extraction, and shared require/body compile evaluation.
Recursive calls remain ordinary call evaluation and are not a separate summary
system. Internal unresolved-call bookkeeping or finer require atoms may be
introduced as implementation IR when useful, but they are not frozen language
objects.

Overload-strategy source syntax is fixed by the callable implementation tail:
`=> strategy_name { ... }`, or `[[strategy_name]] { ... }` where omitting
`=>` requires explicit disambiguation. `#` has no strategy role. The catalog
and comparison semantics of future named strategies remain open. Explicit
compile-companion association syntax remains open. Whether default companion
suppression is ever allowed, and what equivalent compile Pattern/contract
interface it would require, also remains open rather than an implementation
commitment.

#### Later stages

The following remain deferred and are not numbered precisely here:

- general value-to-value `compile` PatternValue execution
- type / kind checking integration
- closure materialization model (ClosureAST → ClosureObject; capture
  environment layout and capture admissibility)
  - preserve the v0.5-A syntax-directed capture binding: every ordinary
    capture is a `NormCapture { slot, initializer }`, inferred shorthand has
    exactly one distinct free non-call bare name, and initializers are
    simultaneous in the pre-capture environment;
  - preserve `InPlace` as an embedded callable-candidate kind with no capture
    list, independently of whether the Raw/Normalized closure has a head;
  - defer unresolved outer reads to the selected embedding layer, while
    requiring ordinary authority for outer writes;
  - place in-place-over-non-in-place preference after
    first-order-over-instantiated and before named strategy filtering;
- result delivery over callable return binding Patterns: direct output writes
  remain per binder, while bare tails and targeted returns match one value as
  `let ResultPattern = expr`;
- ownership / NLL / drop / lifetime design (including any future semantics for
  `with { ... }`); lifetime-policy checking/refinement is after first-order
  type/compile overload selection and is bounded by
  `spec/design/lifetime/lifetime-policy-and-overload-boundary.md`
- storing canonical `Pv:Pp` on every semantic object and wiring full P1
  projection, P2 result validation, function-object views, and compile/runtime/
  seal namespace lookup; formal parameter elaboration must feed the same
  P2-inherited const/mut Pattern both to body entry and to the candidate's
  external policy product-order position, while return elaboration applies the
  symmetric P1-inherited mutability-only refinement;
- seal dependency ordering, complete reflection objects, and any future policy
  stage beyond the current three-phase model;
- integrating const/mut product order into the complete overload resolver, plus
  effect/error/panic and resource-capability policy
  (see `spec/design/policy-capability/policy-visibility-symbols.md`)
- first semantic compiler prototype integrating selected passes
- the named-strategy catalog/semantics and public syntax for explicit
  compile-companion replacement
- any permitted companion-suppression rule and its required replacement
  interface
- finer-grained require atom identity, if later implementation needs it
- the bounded future member set of `BuiltinPrivilegedAstMetaFunction`
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
`spec/design/symbol-world/early-meta-functions-and-namespace-graph.md`). The current code is
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
