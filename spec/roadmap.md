# Roadmap

This document defines the long-term stage model for the `lang` compiler. It
distinguishes implementation stages from semantic research stages.

Stages before v1.0 may overlap in time. The boundaries are scope boundaries,
not strict chronological gates.

## Stage model

### v0.1 — Raw frontend

**Goal**: `source text → tokens → AST → diagnostics`

v0.1 produces stable token dumps, AST dumps, and diagnostic dumps. It is
backed by golden tests.

**What v0.1 is NOT:**

- Not a toy interpreter. It does not execute anything.
- Not a partial compiler. It does not lower, optimize, or emit code.
- Not a semantic analysis prototype. It does not type-check, kind-check,
  resolve names, or analyze lifetimes.

**What v0.1 IS:**

A syntax frontend that:

- Lexes source text into tokens (Name, Literal, Symbol, Trivia, Invalid, Eof).
- Parses tokens into raw AST (forms, lets, expressions, closures, canonical
  skeletons, deduce lists).
- Handles errors gracefully (produces ErrorAst + diagnostic, continues).
- Dumps all three outputs (tokens, AST, diagnostics) in stable, hand-written
  formats suitable for golden testing.

**Deliverables:**

- Crate `lang_syntax` with lexer, parser, AST, dumper, diagnostics.
- Crate `lang_cli` with CLI subcommands: `tokens`, `ast`, `diag`.
- Golden test suite covering all syntax rules.
- Specification documents for AST construction and diagnostics.

**Non-goals (deferred to later stages):**

- Type checking / kind checking / overload resolution.
- Canonical-form evaluation / universal extraction matching.
- Closure AST materialization into callable objects.
- Match, effect, or sync semantics.
- Ownership, lifetime, NLL, drop insertion.
- Interpretation or code generation.
- IR / HIR / MIR lowering.

#### Current implementation snapshot

For the authoritative factual inventory of current implementation status,
see `spec/implementation-status-v0.1.md`. The summary below tracks
phase-level completion, not per-feature status.

**Current implementation status (parser phase 1 + phase 2 binding-context syntax + phase 3.1 closure/parser stabilization + phase 4/4.1 operator syntax + phase 4.2 EntityRef design + phase 4.3 alias binding design + phase 4.4 alias binding parser preservation + phase 4.4.1 alias-parser stabilization):**

The current implementation includes parser phase 1, parser phase 2
binding-context syntax, parser phase 3 closure surface AST, and parser phase
3.1 closure/parser stabilization, parser phase 4 operator syntax as raw AST
sugar, and parser phase 4.1 operator names in binder/path-leaf positions. It
also includes parser phase 4.2 compile-time entity reference syntax design,
phase 4.3 lexical alias binding design, and phase 4.4 raw AST parser
preservation for alias binding (`let binder === EntityRef`). It includes:

- Lexer loop with CRLF/LF normalization and stable token dumps.
- Operator-aware lexer (operator spellings tokenized as `Operator` tokens;
  `+`, `-`, `*`, `/`, `++`, `--`, `<<=`, `>>=`, etc.).
- Cursor, form parser, expression parser, and atom parser.
- Simple `let` forms with text or operator binder names, bare declaration
  annotations (`: type`, `: fn`), explicit rank annotations (`: _: fn`),
  `guard` attributes, and `with` clauses.
- Name, integer literal, string literal, and `::`-path expression atoms
  (including numeric path leaves such as `uint8::1` and final operator path
  leaves such as `std::int::+`).
- Group atoms (`(expr)` without top-level commas).
- Pipe segmentation (`|>`) and ArgPack role assignment (SourcePack,
  InsertPack, RightTargetSubsegment).
- Atom suffix folding for `:: Selector`, `. Selector`, and `.. Selector ArgPack`
  (MemberSugar and DoubleDotSugar).
- Numeric selectors (`obj.1`, `obj..1(args)`, `uint8::1`) using
  `NumericNameAst` in selector position, distinct from numeric literal atoms.
- Expression-local error recovery and permissive atom collection.
- Full v0.1 diagnostic taxonomy (DiagnosticCode covers all phase 2 categories);
  most diagnostics are reachable; unreachable diagnostics exist in the enum
  until corresponding syntax lands.
- Golden test infra for tokens (9 cases), AST (154 cases), and diagnostics (27
  cases).
- Extract-let binders (`let <head, tail> (...) = ...`), deduce-list
  parsing, and canonical skeleton parsing.
- Closure AST (inline `{}`, explicit `() => {}`, closure heads with
  deduce/capture/parameter/fn-item-trait/return clauses).  `where` and
  `acquire` are reserved closure-head positions, not implemented.
- Parser phase 3.1 closure/parser stabilization: stack-based diagnostic gates
  for finite lookahead, regression coverage for failed closure-head lookahead,
  `where`/`acquire` non-recognition, group/ArgPack ambiguity, body-block form
  boundaries, malformed closure recovery, match-style closure arms, capture
  syntax preservation, and return syntax preservation.
- Parser phase 4 operator expression parsing as raw AST sugar: segment-local
  precedence/associativity, prefix `-`, postfix operators, binary operators,
  angle comparison operators in expression context, and diagnostics for
  malformed or chained non-associative operator expressions.
- Parser phase 4.1 operator binder names and final operator path leaves as raw
  AST preservation.
- Parser phase 4.2 compile-time `EntityRef` syntax design documentation in
  `spec/entity-ref-design.md`. This defines the future strong-context syntax
  and parser/semantic boundary, but does not implement an `EntityRef` parser.
- Parser phase 4.3 lexical alias binding design documentation in
  `spec/entity-alias-design.md`. This defines the future `let binder ===
  EntityRef` declaration form, lexical scope rule, operator-alias restriction,
  and parser/semantic boundary, but does not implement alias parsing or
  entity resolution.
- Parser phase 4.4 raw AST parser preservation for alias binding. Lexer
  recognizes `===` as a single structural delimiter token (`Symbol::TripleEqual`).
  Parser produces `LetAliasAst` containing `AliasBinderAst` (Name or Operator)
  and `EntityRefAst` (path segments + leaf). EntityRef parsing is implemented
  only inside alias-let RHS; it is not a general expression parser mode. New
  diagnostic codes: `ExpectedAliasTarget`, `InvalidEntityRef`, and
  `UnexpectedAliasRhsExpression`.
- Parser phase 4.4.1 alias-parser stabilization. Adds boundary-aware entity-ref
  start checking to prevent missing-target recovery from swallowing following
  forms across newline or semicolon boundaries. Refactors `is_alias_rhs_boundary`
  into layered helpers (`is_alias_rhs_newline_boundary`, `is_alias_rhs_hard_boundary`).
  Adds 11 golden tests covering expression-shaped RHS forms, guard/extract/
  annotation/with non-alias invariants, and `===` token non-regression.

It does **not** yet include:

- Operator lookup, operator lowering, operator overload resolution, operator
  dispatch, ADL, or type-directed lookup.
- Operator alias identity validation (spelling + fixity + arity check).
- Compile-time entity lookup, operator lookup, namespace resolution, and
  dependency resolution.
- `where`/`acquire` clause parsing.
- Closure object materialization, capture analysis, type checking, kind
  checking, name resolution, match/effect/sync semantics, HIR/MIR/IR lowering,
  interpretation, and code generation.

The provisional v0.1 top-level newline boundary rule is now implemented.
The broader language-design question of whether form boundaries should remain
line-based or become fully explicit remains open (see
`spec/open-questions.md` §2).

The next implementation phases must fill the gap between this skeleton and
`spec/ast-construction-v0.1.md`.

The selector AST already distinguishes `TextNameAst` and `NumericNameAst` for
future name-polymorphic lookup (see `spec/open-questions.md` §19). No lookup,
binding, or name resolution is implemented in v0.1.

Canonical skeleton AST preservation is implemented (names, wildcards,
literals, paths, argpacks).  All canonical skeleton golden tests are parser
coverage; no matching semantics are assigned.  The Hole/NodeName distinction
is a parse-time role marker, not a semantic binding commitment.

#### Parser/documentation phase track

The syntax-facing work after Phase 4.1 is ordered as:

```text
Phase 4: operator syntax as raw AST sugar (implemented)
Phase 4.1: operator binder names and operator path leaves (implemented)
Phase 4.2: compile-time entity reference syntax design (documentation — complete)
Phase 4.3: lexical alias binding design (`let binder === EntityRef`) (documentation — complete)
Phase 4.4: raw AST parser preservation for alias binding (implemented)
Phase 4.4.1: alias-parser stabilization (implemented)
```

Alias binding is after Phase 4.1 because operator aliases require operator
names in binder and path-leaf positions:

```text
let << === xxx_bit::<<
let >> === xxx_bit::>>
```

These phases are before semantic name resolution. They do not implement lookup,
namespace resolution, dependency resolution, import/package/build-system
semantics, or operator alias validation. Phase 4.4 implements only raw AST
preservation: the parser preserves alias-binding syntax but does not resolve
targets, validate operator identity, or perform entity lookup. The `===`
lexer token is a structural symbol, not an expression operator. EntityRef
parsing is implemented only inside alias-let RHS. See
`spec/entity-ref-design.md` and `spec/entity-alias-design.md`.

---

### v0.2 — Frontend robustness

**Goal**: Hardening the v0.1 frontend.

- Error recovery improvements: fewer panics, better resynchronization.
- Diagnostic quality: clearer messages, secondary spans where helpful.
- Syntax corpus: test against a larger corpus of source files.
- Parser invariants: property-based testing for parser crashes.
- AST dump stability: ensure dump format does not change between minor edits.
- Documentation/test synchronization: every spec change must update golden
  cases and vice versa.

**No new syntax or semantic features.**

---

### v0.3 — Syntax normalization documents

**Goal**: Specify how raw AST may later lower into normalized syntax forms.

This is a documentation stage. Do not implement HIR yet.

Define:

- Normalized form for let bindings (desugared guard/with).
- Normalized form for pipe expressions (flattened segments).
- Normalized form for closure heads (canonicalized clause order).
- Normalized form for canonical skeletons (resolved holes).

The output of this stage is a document specifying normalization rules. Actual
lowering to HIR is deferred.

---

### v0.4 — Canonical form specification

**Goal**: Document value/type canonical forms and universal extraction matching.

Define:

- Value canonical form and type canonical form.
- How universal extraction matching relates to canonical skeletons.
- The relationship between deduce lists and canonical forms.

Do not implement type checking or matching yet. This is a specification stage.

---

### v0.5 — Meta-function boundary specification

**Goal**: Document compiler-provided meta-functions.

Define the interface for compiler-provided meta-functions such as `match`,
`effect`, and `sync`. These are privileged consumers of AST or normalized
syntax. They are not ordinary library functions and not special syntax.

Specify:

- How meta-functions receive AST/normalized-syntax arguments.
- How meta-functions produce results.
- The relationship between meta-functions and closure AST.

Do not implement any meta-function.

---

### v0.6 — Closure materialization model

**Goal**: Document ClosureAST → ClosureObject model.

Define:

- Contexts in which closure AST is materialized into callable objects.
- Capture rules (what is captured, how).
- Inline closure vs explicit closure materialization defaults.
- Object transferability (move, copy, borrow semantics for closures).

This stage produces a specification. Implementation may begin if the frontend
is stable.

---

### v0.7 — Type/kind/checking design

**Goal**: Design and document kind/type checking.

Define:

- Kind system (types of types).
- Type checking rules for each AST node.
- Bidirectional or deductive checking algorithm.
- Error messages for type mismatches.

Implementation may be partial. The primary output is a design document.

---

### v0.8 — Ownership/NLL/drop design

**Goal**: Design and document ownership, always-NLL, and drop semantics.

Define:

- Always-NLL borrow-checking model.
- `guard` and `with` semantics in let bindings.
- User-defined drop and move points.
- Future CFG requirements for NLL.

Do not implement drop insertion in any v0.x frontend stage.

---

### v1.0 — First semantic compiler prototype

**Goal**: Begin integrating selected semantic passes.

Prerequisite: Frontend AST and specification documents are stable.

v1.0 may include:

- Type checking.
- Kind checking (if designed in v0.7).
- Canonical-form evaluation and extraction matching (if specified in v0.4).
- Closure materialization (if specified in v0.6).
- Meta-function invocation for `match`, `effect`, `sync` (if specified in v0.5).

But each semantic pass must be individually gated and must not destabilize the
frontend.

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
