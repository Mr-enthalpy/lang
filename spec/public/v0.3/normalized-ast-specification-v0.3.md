# Normalized AST Specification v0.3

## 1. Purpose

This document is the v0.3 Normalized AST Specification scaffold. It defines
the problem space, the non-semantic boundary, the active design questions,
and the specification work items that v0.3 must resolve.

It does not define the final Normalized AST node set.

## 2. Stage boundary

```text
v0.1   — Raw AST Frontend — completed
v0.1.w — Raw AST Stability Window — closed
v0.2   — Raw AST Contract Freeze / Public Frontend Syntax Specification — closed
v0.3   — Normalized AST Specification — current
v0.4   — Raw AST → Normalized AST Prototype — future
```

## 3. Input authority

v0.3 Normalized AST Specification consumes the frozen v0.2 Raw AST frontend
surface as its input contract.

Raw AST is source-preserving and non-desugared. It preserves:

- operator sugar
- member, double-dot, and bracket-call sugar
- pipe / segment / product architecture
- closure literal structure
- canonical skeletons and deduce lists
- alias-let and EntityRef preservation

Normalized AST will be desugared, structurally regular, and suitable for
later semantic passes (name resolution, type checking, etc.).

## 4. Non-semantic boundary

Normalized AST is:

- desugared but still non-semantic
- not HIR (HIR assumes name resolution and type checking)
- not type-checked
- not name-resolved
- not an execution model

Normalized AST does not:

- resolve names
- resolve operators
- resolve alias targets
- check types or kinds
- validate canonical skeletons
- validate deduce lists
- materialize closures
- perform ownership / NLL / drop analysis

## 5. Design questions active in v0.3

The following questions from `spec/planning/open-questions.md` become active
during v0.3. They are listed here without being resolved.

- **N-AST-1.** Exact Normalized AST node set. What are the exact node types?
  Candidates: normalized call, normalized pattern, normalized declaration.
  Should there be a single unified expression node or distinct per-form nodes?

- **N-AST-2.** Whether Normalized AST lives in `lang_syntax` or a new crate
  (e.g., `lang_norm`).

- **N-AST-3.** Whether raw-to-normalized dumps should be golden-tested.

- **N-AST-4.** How to represent symbolic builtins introduced by desugaring
  (e.g., `operator::call`, `member::lookup`, `pattern::bind`).

- **N-AST-5.** How to preserve source origins through desugaring. Desugaring
  creates new AST nodes that did not appear in source text. How should source
  spans and diagnostic attribution be preserved?

- **N-AST-6.** Whether right-target subsegments become nested call nodes.
  Right-target subsegments (`f (a) g`) are currently flat in Raw AST.

- **N-AST-7.** How to represent pattern normalization for let, params, returns,
  and canonical skeletons.

- **N-AST-8.** How to represent alias declarations before name resolution.
  Alias bindings reference compile-time entities that are not yet resolved.

## 6. Specification work items

The following items must be decided during v0.3 specification. They are
listed as obligation checklists, not as resolved decisions.

- normalized form / declaration / expression boundary
- normalized call / composition structure
- product expression normalization
- product extraction and binding pattern normalization
- operator sugar normalization (prefix-negative, postfix, binary)
- bracket-call normalization (`obj[args...]`)
- member sugar normalization (`obj.field`)
- double-dot sugar normalization (`obj..method(args)`)
- navigation normalization (inner-to-outer `::`)
- closure literal and closure-head normalization (InPlace, Explicit)
- alias-let representation before name resolution
- `ErrorAst` and diagnostics origin preservation through normalization
- how generated / desugared nodes carry source origin
- normalized dump policy (format, golden-test policy)
- crate placement (whether Normalized AST types live in `lang_syntax` or a new crate)

## 7. Explicit non-goals

v0.3 must not:

- create Normalized AST Rust types in the codebase (v0.4)
- implement Raw AST → Normalized AST lowering (v0.4)
- create normalized dumps or golden snapshots (v0.4)
- modify the lexer, parser, Raw AST shape, or DiagnosticCode
- change the v0.2 frozen frontend syntax surface
- perform semantic analysis
- perform name resolution
- perform type checking
- perform operator lookup
- perform alias target resolution

## 8. Status

This document is a stage-opening scaffold. All §5 questions are unresolved.
All §6 work items are open. §3 and §4 boundaries are defined and stable.
