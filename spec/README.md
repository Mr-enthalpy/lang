# Specification Index

This directory contains the specification documents for the `lang` language
frontend, normalizer, and build/namespace bootstrap. Documents are organized by
role rather than in a flat list. The current active stage is v0.6 — Build /
Namespace Graph Bootstrap, with a partial vertical slice in `crates/lang_build`.

## Documentation authority hierarchy

Documentation areas have distinct roles and authority levels:

- **`spec/public/`** — Current user-facing and agent-facing language behavior.
  This is the first place to read current rules.
- **`spec/contracts/`** — Stage and implementation contracts. These are
  engineering constraints, not the main user-facing explanation.
- **`spec/implementation/`** — Implementation inventory and status reports.
- **`spec/history/`** — Historical route, design discussion, alternatives,
  resolved disputes, and audit trail. History preserves why decisions were made,
  but does not define current behavior unless linked from public docs.
- **`spec/design/`** — Transitional design-fusion staging area. It temporarily
  groups forward-looking design material while it is being fused into
  `spec/public/`, `spec/contracts/`, `spec/planning/`, and `spec/history/`.
  It is not a long-term authority tier and must not be read as implemented
  behavior. Entry point: `spec/design/README.md`.
- **`spec/planning/`** — Roadmap and open questions. Planning documents must not
  substitute for public language behavior.

The main rule:

```text
If public docs and historical discussion appear to conflict, public docs define
current behavior.
If future docs describe later semantics, they must not be read as implemented
behavior.
```

If `spec/design/` conflicts with public docs, contracts, or stage planning,
the stable tier wins for its own role. `spec/design/` records material still
awaiting absorption.

## Public normalized-surface specification: v0.5

**`spec/public/v0.5/`** — The completed public normalized-surface baseline.
v0.5 stabilizes the normalized surface semantics produced by the v0.4
normalizer and resets the public documentation structure. Specification and
documentation only; it adds no semantic passes.

| File | Authority | Role |
|---|---|---|
| `README.md` | Stage workspace index | Entry point for v0.5 public documentation. |
| `normalized-surface-semantics-v0.5.md` | Published; authoritative for current normalized surface behavior | Public explanation of the normalized surface: source-product continuation and call binding, product/group/target boundaries, sugar lowering, value/pattern separation, annotation patterns, origin/`Unsupported` visibility, and non-goals. |
| `agent-interpretation-guide-v0.5.md` | Published; normative for agents | Normative guidance for coding/documentation agents: how to interpret source without importing conventional call assumptions. |

## Frozen v0.2 frontend input authority

**`spec/public/v0.2/`** — Frozen frontend input contract. v0.2 is closed but
remains authoritative for the Raw AST input surface that the normalizer
consumes.

| File | Authority | Role |
|---|---|---|
| `lexical-syntax-v0.2.md` | Normative for public lexical syntax | Defines source normalization, lexical categories, token spellings, comments, literals, invalid lexical material, and non-semantic lexer boundaries for v0.2. |
| `concrete-syntax-v0.2.md` | Normative for public concrete syntax | Defines the accepted non-semantic source-level grammar, parser shape, Raw AST preservation boundaries, and parser-level non-semantic constraints for v0.2. |
| `diagnostics-recovery-v0.2.md` | Normative for public frontend diagnostics and recovery | Defines v0.2 lexical/parser diagnostic codes, trigger conditions, span policy, recovery behavior, ErrorAst relation, diagnostic stability, and non-semantic diagnostic boundaries. |
| `raw-ast-frozen-surface-v0.2.md` | Normative frozen surface inventory | Enumerates frozen Raw AST constructs with guarantees, non-semantic boundaries, v0.3 obligations, and forbidden assumptions. |

## Global references

**`spec/reference/`** — Cross-cutting references used across all tiers.

| File | Authority | Role |
|---|---|---|
| `glossary.md` | Normative for terminology | Resolves naming ambiguity across all documents. |

## Implementation backing

**`spec/implementation/v0.1/`** — Implementation backing documents. Read
these only for parser implementation repair, diagnostic implementation
repair, factual inventory checks, or archaeology.

| File | Authority | Role |
|---|---|---|
| `ast-construction-v0.1.md` | Normative for parser implementation behavior | Defines every syntax rule, AST shape, and parser constraint. Implementation-level backing reference. |
| `diagnostics-v0.1.md` | Normative for diagnostic implementation behavior | Defines diagnostic categories, span policy, and recovery behavior. Implementation-level reference. |
| `implementation-status-v0.1.md` | Authoritative factual inventory | Records the current implementation status of every feature. Does not define parser rules. |

## Contract and handoff documents

**`spec/contracts/`** — Raw AST contracts, handoff documents, and normalization
prototype boundary notes. Read these for implementation-boundary work, not for
ordinary syntax understanding.

| File | Authority | Role |
|---|---|---|
| `raw-ast-contract-v0.1.md` | Normative contract for future normalization | Defines Raw AST invariants that future normalization passes may rely on. |
| `raw-ast-contract-freeze-v0.2.md` | Normative for v0.2 contract freeze | Defines v0.2 freeze boundary, allowed work, forbidden work, and handoff requirements for v0.3. |
| `v0.3-normalization-handoff-checklist.md` | Normative for v0.3 handoff readiness; non-normative for final Normalized AST design | Checklist of may-assume, must-not-assume, required input families, diagnostic/recovery inputs, normalization obligations, and open v0.3 questions. |
| `v0.4-normalization-prototype-notes.md` | Normative for the v0.4 normalization boundary | Records what the v0.4 Raw AST → Normalized AST prototype/hardening delivered and the boundary it must not cross (value/pattern separation, annotation patterns, unresolved operator/alias targets, `Unsupported` visibility, no pattern-space/semantic behavior). |
| `v0.8-meta-construction-agent-constraints.md` | Draft construction contract for v0.8-adjacent work; not current public behavior | Cross-block agent guardrails for type-to-type meta construction, requiring no-bypass namespace graph use, `ProductObject` / `ArgProductShape`, `SymbolObject`, `TypeValueId` / `PlaceId` / `AliasChain`, policy planes, canonical meta instance keys, and `NamespaceDelta` atomicity. |
| `v0.9-control-flow-end-events.md` | Handoff contract for `TailValue`/`ReturnEvent` terminal forms and deferred target resolution | v0.9 control-flow end events contract (implemented syntax/normalized structure, deferred semantic resolution). Covers the three return terminal form spellings, non-expression guarantees, terminal block enforcement, and consumer handoff expectations. Target resolution and D-reduction are explicitly deferred. |

## Historical design notes

**`spec/history/v0.1/`** — Historical design and resolved-decision documents.
These remain available but are not the normal public entry point.

| File | Authority | Role |
|---|---|---|
| `frontend-v0.1.md` | Non-normative overview | Historical reader entry point. Describes the v0.1 pipeline, document division, and the boundaries between tokens, AST, and diagnostics. |
| `frontend-design-summary.md` | Non-normative overview | Early Raw AST frontend design decisions (weak lexer, contextual parser, `|>` skeleton, `<>` holes, `let`-only declarations, parser-owns-shape). |
| `operator-design.md` | Normative for operator syntax design | Defines operator identity, spellings, fixity, precedence, associativity, AST sugar shape, lookup boundaries, and implementation boundary. Historical reference. |
| `resolved-questions.md` | Authoritative for resolved decisions | Records design questions resolved in v0.1. |

**`spec/history/v0.3/`** — The v0.3 Normalized AST specification design history.
The v0.3 specification baseline was relocated here; the current public surface is
v0.5.

| File | Authority | Role |
|---|---|---|
| `README.md` | Non-normative historical index | v0.3 design-history entry point. |
| `normalized-ast-specification-v0.3.md` | Historical specification baseline | The v0.3 Normalized AST specification (§7 call skeleton, §8 minimum shape). Relocated from `spec/public/`. |
| `normalized-ast-design-history-v0.3.md` | Non-normative historical record | The `N-AST-1..9` design questions, resolutions, the N-AST-9 review audit trail, and the documentation-reset debt log. |

**`spec/history/v0.4/`** — The v0.4 Raw AST → Normalized AST prototype/hardening
route and decisions.

| File | Authority | Role |
|---|---|---|
| `README.md` | Non-normative historical summary | v0.4 prototype/hardening route, `Unsupported`-audit and value/pattern hardening decisions; points to the v0.4 prototype notes and golden tests. |

## Transitional design-fusion staging

**`spec/design/`** is a transitional staging area, not a long-term authority
tier. These blocks are temporary staging buckets. They exist to avoid a flat
`future/` pile while the symbol / pattern / meta-invocation world is still being
fused. They should shrink as material is promoted into public specs, converted
into contracts, moved into planning, or archived into history. Start at
`spec/design/README.md`.

| Block | Role |
|---|---|
| `spec/design/build-package/` | Package/build layer: manifest records, namespace-graph projection, mount paths, export surface, package identity, dependency edges, source roots, cache/fingerprint/provenance. |
| `spec/design/symbol-world/` | Namespace graph world model: `SymbolId` / `PlaceId` / `TypeValueId`, alias forwarding, writable-place, field functions, `ref`/`share` projection namespaces, type-associated function objects, injection targets, and the early-meta / namespace-graph bootstrap. |
| `spec/design/patterns-overload/` | Pattern normalization, occurrence roles, argument/parameter shapes, first-order type-value candidate adaptation, applicability, specificity, the full overload-resolution vision, static pattern spaces, and extraction chains. |
| `spec/design/meta-invocation/` | Policy-governed meta object invocation: dual symbol-lookup vs callable execution, partial vs strict meta reduction, residualization, guarded invocation, and control-like callables instead of `if constexpr` syntax. |
| `spec/design/policy-capability/` | Symbol-visibility / body-entry / return-object policy, context policy, meta/runtime policy filtering, and future error/panic policy. |
| `spec/design/control-flow/` | Targeted return, D-reduction, Done_Return, control-flow lowering — design only |
| `spec/design/mechanical-lowering/` | Compiler-inserted mechanical action frameworks: automatic argument passing and the `move` fixed point, return normalization and error policy, and `normal`/`tco`/`loop` call modes with no loop core. |

## Planning and debt

**`spec/planning/`** — Roadmap and unresolved debt. Planning references,
not syntax specifications.

| File | Authority | Role |
|---|---|---|
| `roadmap.md` | Authoritative for scope and planning; non-normative for parser behavior | Defines stage boundaries (v0.1–v0.11) and what must not leak between stages. |
| `open-questions.md` | Non-normative | Tracks unresolved, forward-looking design questions (v0.5 stabilization debt and v0.6+). |

## Reading order

Current reading order (summary):

1. `spec/public/v0.5/README.md`
2. `spec/public/v0.5/normalized-surface-semantics-v0.5.md`
3. `spec/public/v0.5/agent-interpretation-guide-v0.5.md`
4. `spec/public/v0.2/*` for the frozen Raw AST input syntax
5. `spec/contracts/*` only when doing implementation-boundary work
6. `spec/history/*` for route / decisions / archaeology
7. `spec/design/README.md` only when working on unstable design-fusion material
   that has not yet been absorbed into public/contracts/planning/history

`spec/history/v0.3/` holds the v0.3 Normalized AST design baseline (historical),
not a current reading step. The detailed per-tier lists below expand this order.

### Current v0.5 public documentation

Start here for the completed v0.5 public normalized-surface baseline:

1. `public/v0.5/README.md` - v0.5 public documentation index.
2. `public/v0.5/normalized-surface-semantics-v0.5.md` - normalized surface semantics (published).
3. `public/v0.5/agent-interpretation-guide-v0.5.md` - how agents should interpret source.
4. `contracts/v0.4-normalization-prototype-notes.md` - the v0.4 normalization boundary.

### v0.3 Normalized AST design history

Read these for the v0.3 Normalized AST design baseline (historical):

1. `history/v0.3/README.md` - v0.3 design-history index.
2. `history/v0.3/normalized-ast-specification-v0.3.md` - v0.3 Normalized AST specification (incl. §7 call skeleton, §8 minimum shape).
3. `history/v0.3/normalized-ast-design-history-v0.3.md` - N-AST design questions, resolutions, audit trail.
4. `contracts/v0.3-normalization-handoff-checklist.md` - v0.3 handoff snapshot.

### Frozen v0.2 frontend input

Read these for the frozen Raw AST input surface:

1. `spec/public/v0.2/lexical-syntax-v0.2.md` - Understand the public lexical syntax.
2. `spec/public/v0.2/concrete-syntax-v0.2.md` - Understand the public concrete syntax.
3. `spec/public/v0.2/diagnostics-recovery-v0.2.md` - Understand public diagnostics and recovery.
4. `spec/public/v0.2/raw-ast-frozen-surface-v0.2.md` - Inspect the frozen Raw AST construct inventory.
5. `spec/reference/glossary.md` - Resolve terminology ambiguity.

### Extended implementer reading order

Read these only when implementing, auditing, or repairing the frontend.

1. `spec/implementation/v0.1/ast-construction-v0.1.md` - Implement the parser.
2. `spec/implementation/v0.1/diagnostics-v0.1.md` - Diagnostic catalog (implementation-level reference).
3. `spec/implementation/v0.1/implementation-status-v0.1.md` - Know current implementation facts.
4. `spec/contracts/raw-ast-contract-v0.1.md` - Know Raw AST invariants for normalization.
5. `spec/contracts/raw-ast-contract-freeze-v0.2.md` - Know v0.2 freeze boundary and v0.3 handoff.
6. `spec/history/v0.1/operator-design.md` - Understand operator syntax rules.
7. `spec/history/v0.1/resolved-questions.md` - Understand resolved design decisions.

### Design-block reading order

Read these only when working on forward-looking design topics. Start at
`spec/design/README.md`, which gives the active semantic route across blocks:

```text
build-package -> symbol-world -> patterns-overload -> meta-invocation
  -> mechanical-lowering -> later runtime lookup / type check
```

Then read within each block as needed. Scope boundaries are in
`spec/planning/roadmap.md`, and known gaps in `spec/planning/open-questions.md`.

## Spec priority

For current normalized surface behavior, `spec/public/v0.5/` is the reader-facing
authority. For frozen Raw AST input syntax, `spec/public/v0.2/` remains
authoritative.

The implementation and golden snapshots remain the factual behavior source.

Documents under `spec/implementation/`, `spec/contracts/`, `spec/history/`,
`spec/design/`, and `spec/planning/` remain available for backing reference,
archaeology, future design, and scope management. They are not the normal
public entry point.

If public docs conflict with history/design/planning documents, treat that as
documentation debt; do not use older or future documents to reinterpret current
behavior.
