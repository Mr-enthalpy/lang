# lang

`lang` is an experimental programming language frontend.

**Current status:** v0.1 Raw AST Frontend completed; v0.1.w Raw AST Stability
Window closed; v0.2 Raw AST Contract Freeze / Public Frontend Syntax
Specification closed; v0.3/v0.4 normalized AST milestones completed; v0.5
public normalized-surface documentation baseline completed.

Current active stage: v0.6 — Build / Namespace Graph Bootstrap, started as a
partial vertical slice in `crates/lang_build`.

v0.3 is the completed Normalized AST specification baseline.
v0.4 is the completed Raw AST -> Normalized AST prototype / hardening checkpoint.
For current public documentation, see `spec/public/v0.5/`.

The Raw AST frontend surface (lexer, parser, AST, diagnostics, golden tests)
is frozen contract material. The v0.4 normalizer lowers Raw AST into a
desugared, non-semantic Normalized AST with a stable dump and golden tests.
v0.5 stabilizes the normalized surface semantics and resets the public
documentation structure; it adds no semantic passes. v0.6 begins the
post-normalization world model: source roots, core bootstrap, namespace graph
snapshots/deltas, resolver objects, and the first narrow early-`struct` meta
slice.

Documentation pointers:

- Current public documentation: `spec/public/v0.5/`
- Frozen Raw AST input surface: `spec/public/v0.2/`
- Normalized AST specification baseline (historical): `spec/history/v0.3/`
- Completed Raw AST -> Normalized AST prototype/hardening notes:
  `spec/contracts/v0.4-normalization-prototype-notes.md`

- **Raw AST**: surface-preserving, non-desugared, parser output.
- **Normalized AST**: desugared, non-semantic AST that unifies calls, extraction,
  and declarations into simple pattern/call/declaration structures.
  Not HIR, not type-checked, not name-resolved.

## Documentation map

### Current v0.5 public documentation

Read these for the completed public normalized surface behavior:

| Document | Purpose |
|---|---|
| `spec/public/v0.5/README.md` | v0.5 public documentation index |
| `spec/public/v0.5/normalized-surface-semantics-v0.5.md` | Published normalized surface semantics (call/product/pipe binding, value/pattern boundaries, origin visibility, non-goals) |
| `spec/public/v0.5/agent-interpretation-guide-v0.5.md` | How agents should interpret source without conventional call assumptions |
| `spec/contracts/v0.4-normalization-prototype-notes.md` | The v0.4 normalization boundary |

### v0.3 Normalized AST design history

v0.3 was the Normalized AST specification stage; v0.4 implemented it and v0.5
publishes the public surface. The v0.3 specification is now historical:

| Document | Purpose |
|---|---|
| `spec/history/v0.3/README.md` | v0.3 design-history index |
| `spec/history/v0.3/normalized-ast-specification-v0.3.md` | v0.3 Normalized AST specification baseline (§7 call skeleton, §8 minimum shape) |
| `spec/history/v0.3/normalized-ast-design-history-v0.3.md` | N-AST design questions, resolutions, audit trail, reset-debt log |
| `spec/contracts/v0.3-normalization-handoff-checklist.md` | v0.3 handoff snapshot (may-assume, must-not-assume, required inputs) |

### Frozen v0.2 frontend input authority

The v0.2 public frontend specification set remains authoritative for the frozen
Raw AST input surface. Read these for the input contract the normalizer consumes:

| Document | Purpose |
|---|---|
| `spec/public/v0.2/lexical-syntax-v0.2.md` | Public lexical syntax specification: source normalization, token categories, weak lexer, names, literals, symbols, operators, trivia |
| `spec/public/v0.2/concrete-syntax-v0.2.md` | Public concrete syntax specification: form boundaries, let/alias-let, binding slots, products, pipes, operators, closures, skeletons, deduce lists |
| `spec/public/v0.2/diagnostics-recovery-v0.2.md` | Public diagnostics and recovery specification: lexical/parser diagnostic codes, trigger conditions, span policy, ErrorAst recovery, non-semantic boundaries |
| `spec/public/v0.2/raw-ast-frozen-surface-v0.2.md` | Frozen Raw AST surface inventory: construct-by-construct guarantees, v0.3 obligations |
| `spec/reference/glossary.md` | Terminology definitions and critical distinctions |

Older v0.1 implementation, contract, historical, planning, and future-design
documents remain present, but they are not part of the normal public reading path.

### Backing and historical references

| Category | Directory | Document | Purpose |
|---|---|---|---|
| Implementation | `spec/implementation/v0.1/` | `ast-construction-v0.1.md` | AST construction rules — implementation-level backing reference |
| Implementation | `spec/implementation/v0.1/` | `diagnostics-v0.1.md` | Diagnostic categories, span policy, recovery — implementation-level backing reference |
| Implementation | `spec/implementation/v0.1/` | `implementation-status-v0.1.md` | Authoritative factual inventory of current implementation status |
| Contract / handoff | `spec/contracts/` | `raw-ast-contract-v0.1.md` | Raw AST invariants for future normalization |
| Contract / handoff | `spec/contracts/` | `raw-ast-contract-freeze-v0.2.md` | v0.2 freeze boundary, allowed/forbidden work, v0.3 handoff |
| Contract / handoff | `spec/contracts/` | `v0.3-normalization-handoff-checklist.md` | v0.3 normalization handoff: may-assume, must-not-assume, required inputs, open v0.3 questions |
| Design / history | `spec/history/v0.1/` | `operator-design.md` | Operator syntax design and implementation boundaries — historical reference |
| Design / history | `spec/history/v0.1/` | `resolved-questions.md` | Design decisions — resolved for v0.1 |
| Design / history | `spec/history/v0.1/` | `frontend-v0.1.md` | Pipeline overview — historical reader entry point |
| Design / history | `spec/history/v0.3/` | `README.md` | v0.3 Normalized AST specification route and resolved design boundary — historical summary pointing to the v0.3 baseline |
| Design / history | `spec/history/v0.4/` | `README.md` | v0.4 Raw AST → Normalized AST prototype/hardening route and decisions — historical summary pointing to the v0.4 prototype notes |

### Design blocks

Forward-looking design material is staged under `spec/design/` (a transitional
design-fusion staging area; non-normative, not current behavior). Start at
`spec/design/README.md`.

| Block | Purpose |
|---|---|
| `spec/design/build-package/` | Package/build layer, manifest, namespace-graph projection, mounts, export surface, provenance |
| `spec/design/symbol-world/` | Namespace graph world model, SymbolId/PlaceId/TypeValueId, alias forwarding, field functions, early-meta bootstrap |
| `spec/design/patterns-overload/` | Pattern normalization, candidate shapes, specificity, overload vision, static pattern spaces |
| `spec/design/meta-invocation/` | Policy-governed meta object invocation, partial/strict reduction, guarded invocation |
| `spec/design/policy-capability/` | Visibility / body-entry / return-object policy, meta/runtime filtering, future error policy |
| `spec/design/mechanical-lowering/` | Automatic argument passing, return normalization, normal/tco/loop call modes |

| Directory | Document | Purpose |
|---|---|---|
| `spec/planning/` | `roadmap.md` | Stage model v0.1–v1.0 and scope boundaries |
| `spec/planning/` | `open-questions.md` | Unresolved design questions and documentation debt |

### Operational

| Document | Purpose |
|---|---|
| `AGENTS.md` | Agent instructions — read before making code changes |
| `SKILL.md` | Operational workflow for frontend work |
| `spec/README.md` | Spec directory index with authority levels |

## Two repository tracks

1. **Frontend track** (completed baseline): v0.5 Normalized Surface Semantics
   Stabilization and Public Documentation Reset. v0.1/v0.1.w/v0.2 completed;
   v0.3 completed the Normalized AST specification baseline; v0.4 completed the
   Raw AST -> Normalized AST prototype/hardening checkpoint.

2. **Build/package/namespace assembly track** (active v0.6 partial
   implementation): `lang_build` implements the first namespace graph world
   model and early `struct` meta vertical slice. Full package management,
   manifest parsing, type checking, policy checking, and general meta execution
   remain future work.

Start with `spec/public/v0.5/README.md` for current v0.5 public documentation.
Read `spec/history/v0.3/` for the v0.3 Normalized AST design baseline
(historical).
Read `spec/public/v0.2/lexical-syntax-v0.2.md` when you need the frozen Raw AST
input contract. Read `spec/public/v0.2/concrete-syntax-v0.2.md` for parsed
syntax and `spec/public/v0.2/diagnostics-recovery-v0.2.md` for error behavior.

## Language surface

`lang` is currently specified by its **normalized surface** (v0.5):

- **Current public surface** — `spec/public/v0.5/`. The published normalized
  surface semantics: how source is read and lowered into Normalized AST
  (call / product / pipe binding, value/pattern boundaries, sugar lowering,
  origin / `Unsupported` visibility, and non-goals).
- **Frozen input layer** — `spec/public/v0.2/`. The Raw AST frontend
  (lexer / parser / diagnostics) is the frozen input syntax the normalizer
  consumes.
- **Implemented lowering layer** — the v0.4 normalizer lowers Raw AST into a
  desugared, non-semantic Normalized AST; its boundary is recorded in
  `spec/contracts/v0.4-normalization-prototype-notes.md`.

The pipeline is `source text -> tokens -> Raw AST -> Normalized AST`, plus
diagnostics. Nothing in the current surface resolves names, checks types, looks
up operators, materializes closures, evaluates, or generates code; it is
structural only, and it is not HIR.

The early Raw AST frontend design decisions (weak lexer, contextual parser,
`|>` skeleton, `<>` holes, `let`-only declarations, parser-owns-shape) are
historical context: see `spec/history/v0.1/frontend-design-summary.md` and the
frozen `spec/public/v0.2/` syntax specs.

## Workspace layout

```text
.
├── AGENTS.md
├── README.md
├── SKILL.md
├── Cargo.toml
├── spec/
│   ├── README.md
│   ├── public/
│   │   ├── v0.2/
│   │   │   ├── lexical-syntax-v0.2.md
│   │   │   ├── concrete-syntax-v0.2.md
│   │   │   ├── diagnostics-recovery-v0.2.md
│   │   │   └── raw-ast-frozen-surface-v0.2.md
│   │   └── v0.5/
│   │       ├── README.md
│   │       ├── normalized-surface-semantics-v0.5.md
│   │       └── agent-interpretation-guide-v0.5.md
│   ├── reference/
│   │   └── glossary.md
│   ├── implementation/
│   │   └── v0.1/
│   │       ├── ast-construction-v0.1.md
│   │       ├── diagnostics-v0.1.md
│   │       └── implementation-status-v0.1.md
│   ├── contracts/
│   │   ├── raw-ast-contract-v0.1.md
│   │   ├── raw-ast-contract-freeze-v0.2.md
│   │   ├── v0.3-normalization-handoff-checklist.md
│   │   └── v0.4-normalization-prototype-notes.md
│   ├── history/
│   │   ├── v0.1/
│   │   │   ├── frontend-v0.1.md
│   │   │   ├── frontend-design-summary.md
│   │   │   ├── operator-design.md
│   │   │   └── resolved-questions.md
│   │   ├── v0.3/
│   │   │   ├── README.md
│   │   │   ├── normalized-ast-specification-v0.3.md
│   │   │   └── normalized-ast-design-history-v0.3.md
│   │   └── v0.4/
│   │       └── README.md
│   ├── design/
│   │   ├── README.md
│   │   ├── build-package/
│   │   ├── symbol-world/
│   │   ├── patterns-overload/
│   │   ├── meta-invocation/
│   │   ├── policy-capability/
│   │   └── mechanical-lowering/
│   └── planning/
│       ├── roadmap.md
│       └── open-questions.md
├── crates/
│   ├── lang_syntax/
│   ├── lang_build/
│   └── lang_cli/
└── tests/
    ├── lexer_golden.rs
    ├── parser_golden.rs
    ├── diagnostics_golden.rs
    ├── normalized_golden.rs
    └── cases/
        ├── lexer/
        ├── parser/
        ├── diagnostics/
        └── norm/
```

## Build

```bash
cargo check --workspace
cargo test
```

With `make` available:

```bash
make check
make test
make fmt
```

## CLI target

The `lang_cli` crate exposes:

```bash
lang tokens path/to/file.lang
lang ast path/to/file.lang
lang norm path/to/file.lang
lang diag path/to/file.lang
```

The repository has golden coverage for lexer, parser/AST, diagnostics, and
normalized AST (`tests/normalized_golden.rs`, `tests/cases/norm/`).
See `spec/implementation/v0.1/implementation-status-v0.1.md` for the current test count.

## Non-goals (current)

The current frontend and normalizer do not implement type checking, kind
checking, name resolution, operator lookup, alias resolution, closure
materialization, NLL/drop insertion, interpretation, code generation, or
IR/HIR/MIR lowering.

The frontend preserves Raw AST shape and the normalizer preserves a desugared,
non-semantic Normalized AST for these future passes, but performs none of them.

## How to read the spec

### Current v0.5 public documentation

Start here for the completed v0.5 public normalized surface baseline:

1. `spec/public/v0.5/README.md` — v0.5 public documentation index.
2. `spec/public/v0.5/normalized-surface-semantics-v0.5.md` — normalized surface semantics (published).
3. `spec/public/v0.5/agent-interpretation-guide-v0.5.md` — how agents should interpret source.
4. `spec/contracts/v0.4-normalization-prototype-notes.md` — the v0.4 normalization boundary.

### v0.3 Normalized AST design history

Read these for the v0.3 Normalized AST design baseline (historical):

1. `spec/history/v0.3/README.md` — v0.3 design-history index.
2. `spec/history/v0.3/normalized-ast-specification-v0.3.md` — v0.3 Normalized AST specification (incl. §7 call skeleton, §8 minimum shape).
3. `spec/history/v0.3/normalized-ast-design-history-v0.3.md` — N-AST design questions, resolutions, and audit trail.
4. `spec/contracts/v0.3-normalization-handoff-checklist.md` — v0.3 handoff snapshot.

### Frozen v0.2 frontend input

Read these for the frozen Raw AST input surface:

1. `spec/public/v0.2/lexical-syntax-v0.2.md` — Understand the frozen lexical syntax.
2. `spec/public/v0.2/concrete-syntax-v0.2.md` — Understand the frozen concrete syntax.
3. `spec/public/v0.2/diagnostics-recovery-v0.2.md` — Understand frozen diagnostics and recovery.
4. `spec/public/v0.2/raw-ast-frozen-surface-v0.2.md` — Inspect the frozen Raw AST construct inventory.
5. `spec/reference/glossary.md` — Resolve terminology.

### Extended implementer reading

Read these only when implementing, auditing, or repairing the frontend.

1. `spec/implementation/v0.1/ast-construction-v0.1.md` — Implement the parser.
2. `spec/implementation/v0.1/diagnostics-v0.1.md` — Diagnostic catalog (implementation-level reference).
3. `spec/implementation/v0.1/implementation-status-v0.1.md` — Know current implementation facts.
4. `spec/contracts/raw-ast-contract-v0.1.md` — Know Raw AST invariants for normalization.
5. `spec/contracts/raw-ast-contract-freeze-v0.2.md` — Know v0.2 freeze boundary and v0.3 handoff.
6. `spec/history/v0.1/operator-design.md` — Understand operator syntax and lookup boundaries.
7. `spec/history/v0.1/resolved-questions.md` — Understand resolved design decisions.
8. `spec/history/v0.1/frontend-v0.1.md` — Understand the pipeline (v0.1 overview).

### Future design and planning

Read these only when working on future design topics.

1. `spec/design/symbol-world/entity-ref-design.md` — Future general EntityRef design.
2. `spec/design/symbol-world/entity-alias-design.md` — Alias binding design (parser preservation implemented, semantics future).
3. `spec/planning/roadmap.md` — Understand scope boundaries.
4. `spec/planning/open-questions.md` — Recognize known gaps.

Other future design documents (build, package, namespace assembly, library namespace)
are listed in the Documentation map above under Build / package / namespace (future notes).

## Expected future workspace shape

Future stages may add crates under `crates/` such as:

```text
crates/
  lang_syntax/
  lang_build/        (v0.6 partial)
  lang_cli/
  lang_manifest/     (v0.6+)
  lang_typeck/       (later)
  lang_nll/          (later)
  lang_codegen/      (v1.0+)
```

No semantic crate should be added before its corresponding design stage.
