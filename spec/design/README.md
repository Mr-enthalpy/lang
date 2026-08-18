# Design Fusion Staging Area

**Status: Transitional design-fusion staging area. Nothing in this directory
defines current user-facing behavior, and this directory is not intended to be
the long-term home for these documents. Current behavior lives in
`spec/public/`. Stage contracts live in `spec/contracts/`. Route and open-scope
decisions live in `spec/planning/`. Superseded design history and ADR material
live in `spec/history/`.**

`spec/design/` is a temporary sorting/staging area, not a long-term authority
tier. The design blocks exist to regroup, fuse, de-duplicate, and split the old
flat `spec/future/` pile; they are not the final documentation layer. As the
symbol / pattern / meta-invocation world stabilizes, each block's material
should migrate into `spec/public/`, `spec/contracts/`, `spec/planning/`, and
`spec/history/`, after which `spec/design/` is shrunk or removed.

## Authority

- `spec/public/` defines current behavior. If a design block and a public
  document appear to conflict, the public document wins for current behavior.
- These design documents constrain the intended direction. They are not a
  promise that the described semantics are implemented.
- Accepted ADR constraints have been absorbed into the relevant design blocks;
  they constrain direction but do not override `spec/public/`.

### Topic owners

To prevent parallel future-design texts from drifting, each cross-cutting topic
has one normative owner inside the staging area:

| Topic | Normative owner | Other documents retain only |
|---|---|---|
| Pattern relational calculus, proof-relevant observation/extraction, direct structural incidence, binderless Pattern, annotation split, constructor/extractor family contracts | `patterns-overload/pattern-values-relational-semantics-and-extraction.md` | Candidate/result/control consumers, implementation gaps, and links |
| Symbol-first resolution, complete type projection, `compile` / `meta`, `struct` forming complete type values, pure `extend`, place-level `inject`, binding/install boundary | `symbol-world/symbol-first-meta-construction-and-pattern-injection.md` | A consistency summary, implementation gap, and link |
| Namespace origin, construction-unit ownership, physical contribution authority, cross-file closure | `symbol-world/symbol-construction-units-and-namespace-origin.md` | Build-phase application, implementation gap, and link |
| Semantic owner identity, PatternRoot alpha boundary, namespace views, package boundary, and mount redirects | `../contracts/v0.6-semantic-owner-namespace-graph.md` | Historical context and implementation mapping only |
| Canonical `Pv:Pp`, contextual binding `P1`, result `P2`, function-object stage derivation, seal visibility/snapshot, const/mut product order, no scalar result policy, mechanical compile projection, companions, match staging, and coarse automatic require | `symbol-world/symbol-policy-and-compile-flow-projection.md` | Implementation mapping, invocation handoff, pattern algebra, and links |
| Existing-First/Constructible-Second demand satisfaction, binding-P1 conservative-extension corollary, atomic Runtime Val1 migration, and mixed-stage boundary | `symbol-world/symbol-policy-and-compile-flow-projection.md` | Current binding-P1 helper coverage stays in the v0.6 transition contract; runtime is the current constructible stage branch; mixed-stage Policy existence/readability/progressive-evaluation semantics are fixed while residual representation and effect/ABI mechanics stay in planning |
| Atomic migration as compiler-inserted ordinary function-object invocation, distinct from structure-changing mechanical operations | `symbol-world/function-object-call-model.md` | The bounded SemanticWorld/PreparedCallCandidate route is connected; caller-supplied transition carriers remain isolated algebra fixtures, not a new callable ontology |
| Transition endpoint Policy preference placement in the overload pipeline | `patterns-overload/overload-resolution-design.md` | Connected Bp' composes implemented ordinary and optional endpoint coordinates once; the older endpoint-only maxima remains a private non-composable fixture |
| Literal family, atomic builtin type T, and concrete numeric Tnum identity | `symbol-world/type-values-places-and-borrow-views.md` | Current helper/core coverage stays in the v0.6 transition contract |
| Current cross-Policy, ordinary-invocation spine, Gsrc transport, and literal-helper implementation subset | `../contracts/v0.6-cross-policy-value-transition.md` | Records implemented carriers and deferred integration only; does not own language semantics |
| Current `PatternHeadId` registry/materialization substrate | `../contracts/v0.9-pattern-head-identity-and-explicit-navigation.md` | No claim of final owner resolution |
| Pattern/argument shape adaptation before overload qualification | `patterns-overload/pattern-normalization-and-first-order-overload.md` | Structural handoff only; consumes the relational Pattern authority and defines no competing relation, policy, or final selection rules |
| Overload candidate preparation, linear filters, qualification boundary, must-select final check | `patterns-overload/overload-resolution-design.md` | Invocation consumes the selected entry; policy definition stays in symbol-world |
| Invocation frames, partial/strict demand, residualization | `meta-invocation/meta-object-invocation-and-policy-reduction.md` | References to symbol-world policy/result ranks and overload selection |
| Pattern-space residual, `Done`, and control-pattern algebra | `patterns-overload/static-pattern-spaces-and-extraction-chains.md` | Consumes the canonical Pattern relation and symbol-to-value lookup |
| Lifetime-policy separation from type/compile overload | `lifetime/lifetime-policy-and-overload-boundary.md` | No positive lifetime algorithm in this stage |
| Stage ordering and implementation dependencies | `../planning/roadmap.md` | Links to semantic owners rather than duplicated rules |

If a satellite summary conflicts with its listed owner, the owner controls the
future-design direction. Current implemented behavior remains governed by
`spec/public/` and the applicable stage contract.

## Current staging route

The current staging route (not a permanent design reading order) is:

```text
symbol-first-meta-construction-and-pattern-injection (canonical construction boundary)
  -> symbol-construction-units-and-namespace-origin (origin/ownership boundary)
  -> symbol-policy-and-compile-flow-projection (policy/flow/require boundary)
  -> v0.8-semantic-spine (value/extraction narrative)
  -> symbolic construction values and extraction interfaces
  -> meta-invocation
  -> mechanical lowering family
  -> later runtime lookup
  -> first-order type check
```

For v0.8-adjacent compile/meta construction work, the recommended
reading order is:

```text
1. symbol-first-meta-construction-and-pattern-injection.md — canonical symbol/construction boundary
2. symbol-construction-units-and-namespace-origin.md — namespace origin and construction ownership
3. symbol-policy-and-compile-flow-projection.md — policy pairs, seal, compile projection, companions, require
4. v0.8-semantic-spine.md — value/extraction narrative
5. return-value-extraction-and-implicit-decomposition.md — extraction view
6. v0.8-symbolic-construction-values-and-extraction-interfaces.md — transitional construction/extraction contract
7. meta-object-invocation-and-policy-reduction.md — invocation demand and residualization
8. v0.8-meta-construction-agent-constraints.md — implementation guardrails
```

In block terms:

```text
symbol-first facets / compile / meta / pattern scopes
  -> namespace origin / construction-unit ownership
  -> semantic-spine value/extraction narrative
  -> patterns-overload / extraction-view
  -> transitional symbolic-construction-values
  -> meta-invocation
  -> mechanical-lowering
  -> later runtime lookup / type check
```

Runtime lookup and first-order type checking are deliberately later than the
pattern/type-value/meta-invocation work.

For implementation patches that touch v0.8-adjacent compile/meta
construction, read `spec/contracts/v0.8-meta-construction-agent-constraints.md`
after the semantic spine and construction-value documents. It is the
implementation guardrail, not the semantic entry point.

## Blocks

| Block | Responsibility | Not responsible for |
|---|---|---|
| `build-package/` | Package/build layer projected into the namespace graph: package identity, manifest records, source roots, dependency edges, mount paths, physical-directory contribution authority, export surface, cache/fingerprint/provenance. | Language expression semantics. |
| `symbol-world/` | Namespace graph world model: Symbol `<tau?,V_S?>` role/member projection and identities, `compile` / `meta`, canonical `Pv:Pp`, contextual P1/P2 elaboration, seal visibility, const/mut member overloads, compile-flow projection and companions, automatic require, meta return self-root, construction-authority (`OpenHere_Σ` / `WindowLive_Σ`) state, explicit borrow views (`ref` / `share` / `rebind`), pattern scopes, `struct` forming complete type values, pure `extend`, place-level `inject`, namespace origin/construction ownership, binding/install, and early-meta bootstrap. | Full type checking, borrow-view/writability checking implementation, access-tree construction, lifetime checking implementation. |
| `patterns-overload/` | Canonical relational Pattern semantics, proof-relevant observation/extraction, structural incidence, binderless Patterns and annotation interaction; plus `PatternObject`, argument/parameter shapes, candidate adaptation, overload vision, and later residual/control-pattern consumers. | Runtime overload resolution implementation and concrete pattern-space/derivation IR. |
| `meta-invocation/` | Symbol-first invocation frames, candidate-selection handoff, partial vs strict demand, residualization, and policy-staged pattern matching. | Defining symbol construction, layered policy, overload ordering, or pattern algebra (it references their canonical owners). |
| `policy-capability/` | Current flat-metadata mapping to canonical `Pv:Pp`, P1/P2 contextual elaboration, and orthogonal policy dimensions. | Compile-flow/require semantics and mechanical return normalization. |
| `lifetime/` | Canonical owner of `@`: `@` is a privileged place-observation operation that yields a lifetime value (`LifetimeVal`), never a borrow view and never a `type ref`; also owns escape checking, the `NoImplicitBorrowFormation` overload boundary, and the boundary placing lifetime rules after completed first-order type/compile overload selection. The former `LifetimeFact` / `P ref` instance groups, `t@ : type ref`, and the `type ref@` / `type share@` fixed points are retired. `ref` / `share` are the borrow constructors (`PrivilegedActualPlace(ref-family)` / `PrivilegedActualPlace(share-family)`); explicit higher-level selection uses `t |> (type ref)` / `t |> (type share)`. The full `@` lifetime algebra is deliberately left unfrozen. | Region/origin algebra, lifetime checking, specificity, Horae logic, or implementation. |
| `control-flow/` | Targeted return, D-reduction, Done_Return, control-flow lowering — design only | Implemented parser / normalizer return syntax (lives in `spec/public/` and `spec/contracts/`); runtime return execution semantics. |
| `mechanical-lowering/` | Compiler-inserted mechanical action frameworks: automatic argument passing and the `move` fixed point, return normalization and error policy, and `normal`/`tco`/`loop` call modes (no loop core). | Backend/machine ABI, final IR instruction format. |

## Eventual absorption targets

| Staging block | Eventual destination |
|---|---|
| build-package/ | `spec/contracts/` for build/namespace invariants; `spec/planning/` for package/manifest roadmap; `spec/public/` only after a manifest/package surface is user-facing. |
| symbol-world/ | `spec/contracts/` for namespace graph / delta / resolver invariants; `spec/public/` for stable symbol/type/place behavior once implemented; `spec/history/` for superseded bootstrap notes. |
| patterns-overload/ | `spec/public/` for stable pattern/overload semantics; `spec/contracts/` for normalized-pattern handoff obligations; `spec/planning/` for runtime overload/type-check staging; `spec/history/` for obsolete extraction-chain alternatives. |
| meta-invocation/ | `spec/public/` once meta invocation is a user-facing semantic model; `spec/contracts/` for evaluator/residualization obligations; `spec/planning/` for runtime lookup/type-check sequencing. |
| policy-capability/ | `spec/public/` for stable policy semantics; `spec/contracts/` for policy metadata/checker boundaries; `spec/planning/` for deferred lattice/effect/error work. |
| lifetime/ | `spec/planning/` until region/origin/refinement semantics are designed; later contract/public placement depends on implementation. |
| mechanical-lowering/ | `spec/contracts/` for lowering obligations and IR handoff invariants; `spec/public/` only for stable source-visible effects such as move/noerror/call-mode semantics; `spec/history/` for rejected loop/if-constexpr alternatives. |

## Implementation status

- Partial implementation (a narrow vertical slice in `crates/lang_build`):
  `build-package/` (API-level build/namespace graph) and parts of
  `symbol-world/` (namespace graph, resolver, early-meta `struct`/`verify`
  slice, flat policy lookup environments) and `policy-capability/` (policy
  metadata and pair helpers).
- Future design only (not implemented): `patterns-overload/`,
  `meta-invocation/`, `mechanical-lowering/`, `lifetime/`, and the remaining
  `symbol-world/` and `policy-capability/` semantics (TypeValueId, borrow views
  and `rebind`, construction-authority (`OpenHere_Σ` / `WindowLive_Σ`) checking, independent
  writability/member-creation checking, pure `extend`, place-level `inject`,
  full policy checking).

## Reading order

1. Read this file for the route and block responsibilities.
2. Read the block `README.md` for any block you are working in.
3. Follow the active route when a topic spans blocks.
4. Scope boundaries: `spec/planning/roadmap.md`. Known gaps:
   `spec/planning/open-questions.md`.
