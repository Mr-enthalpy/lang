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

**Current implementation status (parser phase 1 plus parser phase 2 binding-context syntax):**

The current implementation includes parser phase 1 plus parser phase 2
binding-context syntax. It includes:

- Lexer loop with CRLF/LF normalization and stable token dumps.
- Operator-aware lexer (operator spellings tokenized as `Operator` tokens;
  `+`, `-`, `*`, `/`, `++`, `--`, `<<=`, `>>=`, etc.).
- Cursor, form parser, expression parser, and atom parser.
- Simple `let` forms with bare declaration annotations (`: type`, `: fn`),
  explicit rank annotations (`: _: fn`), `guard` attributes, and `with` clauses.
- Name, integer literal, string literal, and `::`-path expression atoms
  (including numeric path leaves such as `uint8::1`).
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
- Golden test infra for tokens (9 cases), AST (58 cases), and diagnostics (27
  cases).
- Extract-let binders (`let <head, tail> (...) = ...`), deduce-list
  parsing, and canonical skeleton parsing.

It does **not** yet include:

- Closure AST (inline `{}`, explicit `() => {}`, closure heads).
- Operator parser (operator spellings are tokenized but expression-level
  operator parsing, precedence, associativity, and operator-sugar AST are
  not yet implemented).

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
