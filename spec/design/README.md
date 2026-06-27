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

## Current staging route

The current staging route (not a permanent design reading order) is:

```text
package/manifest identity
  -> TypeValueId / PlaceId / AliasChain
  -> pattern normalization + first-order candidate shapes
  -> formal meta object invocation
  -> mechanical lowering family
  -> later runtime lookup
  -> first-order type check
```

In block terms:

```text
build-package -> symbol-world -> patterns-overload -> meta-invocation
  -> mechanical-lowering -> later runtime lookup / type check
```

Runtime lookup and first-order type checking are deliberately later than the
pattern/type-value/meta-invocation work.

## Blocks

| Block | Responsibility | Not responsible for |
|---|---|---|
| `build-package/` | Package/build layer projected into the namespace graph: package identity, manifest records, source roots, dependency edges, mount paths, export surface, cache/fingerprint/provenance. | Language expression semantics. |
| `symbol-world/` | Namespace graph world model: `SymbolObject`, `SymbolId` / `PlaceId` / `TypeValueId`, alias forwarding, writable-place, field functions, `ref`/`share` projection namespaces, type-associated function objects, injection targets; plus the early-meta / namespace-graph bootstrap. | Full type checking, full alias resolver, access-tree construction. |
| `patterns-overload/` | `PatternObject`, occurrence roles, `RawArgShape` / `ParameterShape`, first-order type-value candidate adaptation, applicability, specificity; the full overload-resolution vision; static pattern spaces and extraction chains. | Runtime overload resolution implementation; full pattern-space algebra. |
| `meta-invocation/` | Policy-governed meta object invocation: dual symbol-lookup vs callable execution, partial vs strict meta reduction, residualization, guarded invocation; control-like callables instead of an `if constexpr` / `if` syntax split. | Defining symbol-world, patterns-overload, or policy-capability internals (it references them). |
| `policy-capability/` | Symbol-visibility policy, callable body-entry policy, return-object policy, context policy, meta/runtime policy filtering, and future error/panic policy. | Mechanical return normalization (that lives in `mechanical-lowering/` and only references policy planes here). |
| `mechanical-lowering/` | Compiler-inserted mechanical action frameworks: automatic argument passing and the `move` fixed point, return normalization and error policy, and `normal`/`tco`/`loop` call modes (no loop core). | Backend/machine ABI, final IR instruction format. |

## Eventual absorption targets

| Staging block | Eventual destination |
|---|---|
| build-package/ | `spec/contracts/` for build/namespace invariants; `spec/planning/` for package/manifest roadmap; `spec/public/` only after a manifest/package surface is user-facing. |
| symbol-world/ | `spec/contracts/` for namespace graph / delta / resolver invariants; `spec/public/` for stable symbol/type/place behavior once implemented; `spec/history/` for superseded bootstrap notes. |
| patterns-overload/ | `spec/public/` for stable pattern/overload semantics; `spec/contracts/` for normalized-pattern handoff obligations; `spec/planning/` for runtime overload/type-check staging; `spec/history/` for obsolete extraction-chain alternatives. |
| meta-invocation/ | `spec/public/` once meta invocation is a user-facing semantic model; `spec/contracts/` for evaluator/residualization obligations; `spec/planning/` for runtime lookup/type-check sequencing. |
| policy-capability/ | `spec/public/` for stable policy semantics; `spec/contracts/` for policy metadata/checker boundaries; `spec/planning/` for deferred lattice/effect/error work. |
| mechanical-lowering/ | `spec/contracts/` for lowering obligations and IR handoff invariants; `spec/public/` only for stable source-visible effects such as move/noerror/call-mode semantics; `spec/history/` for rejected loop/if-constexpr alternatives. |

## Implementation status

- Partial implementation (a narrow vertical slice in `crates/lang_build`):
  `build-package/` (API-level build/namespace graph) and parts of
  `symbol-world/` (namespace graph, resolver, early-meta `struct`/`verify`
  slice, `PolicyEnv::Meta` lookup filtering) and `policy-capability/` (policy
  metadata).
- Future design only (not implemented): `patterns-overload/`,
  `meta-invocation/`, `mechanical-lowering/`, and the remaining `symbol-world/`
  and `policy-capability/` semantics (TypeValueId, alias forwarding,
  writable-place checking, full policy checking).

## Reading order

1. Read this file for the route and block responsibilities.
2. Read the block `README.md` for any block you are working in.
3. Follow the active route when a topic spans blocks.
4. Scope boundaries: `spec/planning/roadmap.md`. Known gaps:
   `spec/planning/open-questions.md`.
