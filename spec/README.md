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
- **`spec/design/`** — Target-semantic design-fusion staging area. It
  temporarily groups forward-looking design material, including explicitly
  named canonical target-semantic owners, while it is being fused into
  `spec/public/`, `spec/contracts/`, `spec/planning/`, and `spec/history/`.
  It is not a long-term public authority tier and must not be read as
  implemented behavior. Within target semantics, the per-topic canonical owner
  named by `spec/design/README.md` controls over satellite design summaries.
  Entry point: `spec/design/README.md`.
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
| `normalized-surface-semantics-v0.5.md` | Published; authoritative for current normalized surface behavior | Public explanation of the normalized surface: source-product continuation and call binding, PolicyLet preservation, product/group/target boundaries, sugar lowering, value/pattern separation, annotation patterns, origin/`Unsupported` visibility, and non-goals. |
| `agent-interpretation-guide-v0.5.md` | Published; normative for agents | Normative guidance for coding/documentation agents: how to interpret source without importing conventional call assumptions. |

## Current Raw AST amendment and contract

The current frontend no longer claims to be the unchanged v0.2 parser. Read:

| File | Authority | Role |
|---|---|---|
| `contracts/frontend-semantic-amendment-v0.5-a.md` | Normative versioned amendment | Classifies hard structural corrections and new syntax, including the expression-level PolicyLet boundary, while preserving the v0.1/v0.2/v0.3 historical snapshots. |
| `contracts/raw-ast-contract-v0.5.md` | Normative current Raw AST contract | Defines the amended lexer/parser/Raw-AST surface, independent closure placement/provenance, and `PatternValidatedNormProgram` handoff. |
| `contracts/v0.6-semantic-owner-namespace-graph.md` | Normative current semantic/build amendment | Defines callable/meta semantic owners, Pattern-root alpha identity, namespace views, structural member visibility, package boundaries, and mount redirects; supersedes the v0.5 active-ancestor no-shadow claim. |
| `contracts/v0.6-cross-policy-value-transition.md` | Normative for the bounded connected implementation slice | Records T/Tnum helpers, complete-choice projection followed by runtime-branch extraction, semantic Symbol/Val2/TypeValue/Pattern-owner/associated-`()` routing, one connected Bp' carrier, source-backed atomic migration, fixture coverage for a future pre-Bp fallback strategy that current source cannot construct, and the retained algebra-only transition prototype. |

## Frozen v0.2 frontend input history

**`spec/public/v0.2/`** — Frozen frontend input contract. v0.2 is closed but
remains authoritative for the v0.2 historical surface. It must not be edited
to make later amendments appear original.

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

## Historical implementation backing

**`spec/implementation/v0.1/`** — Closed v0.1 implementation snapshots. Read
these for parser archaeology and the pre-amendment baseline, then apply v0.5-A
and the current v0.5 contract.

| File | Authority | Role |
|---|---|---|
| `ast-construction-v0.1.md` | Historical v0.1 parser snapshot | Defines the closed v0.1 syntax/AST baseline. |
| `diagnostics-v0.1.md` | Historical v0.1 diagnostic snapshot | Defines the closed v0.1 diagnostic and recovery baseline. |
| `implementation-status-v0.1.md` | Historical v0.1 inventory | Records implementation status at the v0.1 snapshot, not current amended facts. |

## Contract and handoff documents

**`spec/contracts/`** — Raw AST contracts, handoff documents, and normalization
prototype boundary notes. Read these for implementation-boundary work, not for
ordinary syntax understanding.

| File | Authority | Role |
|---|---|---|
| `raw-ast-contract-v0.1.md` | Normative contract for future normalization | Defines Raw AST invariants that future normalization passes may rely on. |
| `raw-ast-contract-freeze-v0.2.md` | Normative for v0.2 contract freeze | Defines v0.2 freeze boundary, allowed work, forbidden work, and handoff requirements for v0.3. |
| `frontend-semantic-amendment-v0.5-a.md` | Normative versioned amendment | Records each post-freeze correction/extension without rewriting the frozen documents. |
| `raw-ast-contract-v0.5.md` | Normative current contract | Defines the current amended Raw AST and validated-normalization boundary. |
| `v0.3-normalization-handoff-checklist.md` | Normative for v0.3 handoff readiness; non-normative for final Normalized AST design | Checklist of may-assume, must-not-assume, required input families, diagnostic/recovery inputs, normalization obligations, and open v0.3 questions. |
| `v0.4-normalization-prototype-notes.md` | Normative for the v0.4 normalization boundary | Records what the v0.4 Raw AST → Normalized AST prototype/hardening delivered and the boundary it must not cross (value/pattern separation, annotation patterns, unresolved operator/alias targets, `Unsupported` visibility, no pattern-space/semantic behavior). |
| `v0.6-semantic-owner-namespace-graph.md` | Normative for current v0.6 identity and namespace substrate | Defines semantic-owner identity, PatternRoot alpha boundaries, Full/External/DefaultExtraction views, struct member visibility, package boundaries, mount redirects, typed failures, and deferred integration gates. |
| `v0.6-cross-policy-value-transition.md` | Normative for the bounded connected implementation slice | Records helper algebra, the connected ordinary invocation/migration substrate, Gsrc-backed transport evidence, and explicit remaining integration boundaries; canonical policy, invocation, overload, Pattern, and type-value owners still define language semantics. |
| `v0.8-symbolic-construction-values-and-extraction-interfaces.md` | Transitional construction/extraction contract; not current public behavior | Preserves extraction and current v0.8/v0.9 construction-substrate boundaries; its old formal `r =`/`r ===` return split is superseded by the canonical symbol-first design note. |
| `v0.8-meta-construction-agent-constraints.md` | Draft construction contract for v0.8-adjacent work; not current public behavior | Cross-block guardrails requiring no-bypass namespace graph use, `ProductObject` / `ArgProductShape`, symbol/place/pattern-value separation, transitional policy metadata aligned toward `P1` / `P2`, rank-directed keys, resolved pattern owners, and `NamespaceDelta` atomicity. |
| `v0.9-pattern-head-identity-and-explicit-navigation.md` | Mixed implemented-substrate/future handoff contract | Preserves bare-name vs explicit-`::` navigation and documents the current registry-backed `PatternHeadId` attachment substrate; final `ResolvedPatternScope`, binding-independent `struct` ownership, and `inject` are future. |
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
| `spec/design/build-package/` | Package/build layer: manifest records, namespace-graph projection, mount paths, physical-directory contribution authority, export surface, package identity, dependency edges, source roots, cache/fingerprint/provenance. |
| `spec/design/symbol-world/` | Namespace graph world model: recursive `Object = <Val1?,P,Val2>`, Symbol `<tau?,V_S?>` role/member projections, canonical `Pv:Pp` plus whole-slot `PolicyMode`, contextual P1/P2 elaboration, call-local nested Policy closure, seal visibility/snapshot, three-point preference and capability realization, stable candidate facts plus selected-invocation dynamic legality, compile-flow projection, abstract scalar literal denotations/concrete construction and ranked string `str@compile`, companions, automatic require, identities, explicit borrow views (`ref` / `share` / `rebind`), `compile` / `meta`, meta return self-root, construction-authority (`OpenHere_Σ` / `WindowLive_Σ`) state, resolved pattern scopes, `struct` forming complete type values, pure `extend`, place-level `inject`, namespace origin/construction ownership, binding/install, and the early-meta bootstrap. |
| `spec/design/patterns-overload/` | Canonical relational Pattern semantics, observation/extraction, structural incidence, binderless Patterns and annotation interaction; candidate adaptation and overload vision; later residual/`Done`/control-pattern consumers. The canonical entry is `pattern-values-relational-semantics-and-extraction.md`. |
| `spec/design/meta-invocation/` | Policy-governed invocation: heterogeneous value candidates, canonical pair handoff, partial vs strict demand, residualization, and policy-staged pattern matching. |
| `spec/design/policy-capability/` | Mapping from current flat policy metadata to canonical `Pv:Pp` plus whole-slot `PolicyMode`, contextual P1/P2 elaboration, seal boundaries, and 3×3 capability realization; no final `P3`. Flat/2×2 carriers are implementation subsets. |
| `spec/design/lifetime/` | Canonical owner of continuation-relative `@ = ReifyLife(NameOf(actual), Pos(SemanticContinuation))`, LifeName/NameView, `LifetimeValue` as an ordinary first-class semantic value whose runtime materialization still uses the ordinary callspace rule, pairwise-distinct exclusive-write and same-root shared-read defaults plus finite Pre patch, exact move-origin and gapless Region boundaries, selected CopyConstruct lifecycle posts, cleanup, Pre/Post summaries, extensible global Color vocabulary with finite/monotone committed-universe relations, escape checking, and the post-overload boundary. `ref` / `share` alone retain `PrivilegedActualPlace`; `@` is neither a borrow nor a place-acquisition operation. Concrete IR/checker, summary compression, access-tree integration, diagnostics, and extended Horae logic remain future work. |
| `spec/design/control-flow/` | Targeted return, D-reduction, Done_Return, control-flow lowering — design only |
| `spec/design/mechanical-lowering/` | Canonical `CanonicalMechanicalPassCore` for move/copy action meaning and move fixed point; non-normative future selection/lowering frameworks for automatic argument passing, return normalization/error policy, and `normal`/`tco`/`loop` call modes. |

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

Read these as the frozen Raw AST historical surface:

1. `spec/public/v0.2/lexical-syntax-v0.2.md` - Understand the public lexical syntax.
2. `spec/public/v0.2/concrete-syntax-v0.2.md` - Understand the public concrete syntax.
3. `spec/public/v0.2/diagnostics-recovery-v0.2.md` - Understand public diagnostics and recovery.
4. `spec/public/v0.2/raw-ast-frozen-surface-v0.2.md` - Inspect the frozen Raw AST construct inventory.
5. `spec/reference/glossary.md` - Resolve terminology ambiguity.

Then read:

1. `spec/contracts/frontend-semantic-amendment-v0.5-a.md`
2. `spec/contracts/raw-ast-contract-v0.5.md`

### Extended implementer reading order

Read these only when implementing, auditing, or repairing the frontend.

1. `spec/implementation/v0.1/ast-construction-v0.1.md` - Implement the parser.
2. `spec/implementation/v0.1/diagnostics-v0.1.md` - Diagnostic catalog (implementation-level reference).
3. `spec/implementation/v0.1/implementation-status-v0.1.md` - Know the historical v0.1 implementation snapshot.
4. `spec/contracts/frontend-semantic-amendment-v0.5-a.md` - Apply the versioned delta.
5. `spec/contracts/raw-ast-contract-v0.5.md` - Know current amended facts.
6. `spec/history/v0.1/operator-design.md` - Understand operator syntax rules.
7. `spec/history/v0.1/resolved-questions.md` - Understand resolved design decisions.

### Design-block reading order

Read these only when working on forward-looking design topics. Start at
`spec/design/README.md`, which gives the active semantic route across blocks:

```text
build-package -> symbol-world -> patterns-overload -> meta-invocation
  -> mechanical-lowering -> later runtime lookup / type check
```

For symbol construction work, begin the symbol-world block with
`spec/design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md`.
Then read
`spec/design/symbol-world/symbol-construction-units-and-namespace-origin.md`
for namespace creation origin, physical authority, and source/meta construction
ownership.
Then read
`spec/design/symbol-world/symbol-policy-and-compile-flow-projection.md` for
layered policy, contextual P1 binding projection, P2 result normalization,
mechanical projection of ordinary calls, complete derived compile-companion
objects, finite local flow, and coarse inferred-require slicing.

Then read within each block as needed. Scope boundaries are in
`spec/planning/roadmap.md`, and known gaps in `spec/planning/open-questions.md`.

## Spec priority

For current normalized surface behavior, `spec/public/v0.5/` is the
reader-facing authority. `spec/public/v0.2/` remains authoritative only for
the frozen historical Raw AST snapshot; v0.5-A and `raw-ast-contract-v0.5.md`
define the current amended parser contract.

The implementation and golden snapshots remain the factual behavior source.

Documents under `spec/implementation/`, `spec/contracts/`, `spec/history/`,
`spec/design/`, and `spec/planning/` remain available for backing reference,
archaeology, future design, and scope management. They are not the normal
public entry point.

If public docs conflict with history/design/planning documents, treat that as
documentation debt; do not use older or future documents to reinterpret current
behavior.
