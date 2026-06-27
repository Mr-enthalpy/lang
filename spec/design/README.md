# Design Blocks

**Status: Non-normative design / future design. Nothing in `spec/design/` defines
current user-facing language behavior. Current behavior lives in `spec/public/`.**

This directory is the entry point for the language's forward-looking design,
reorganized from the former flat `spec/future/` bucket into design blocks. Some
blocks carry partial-implementation notes (a narrow v0.6/v0.7 slice exists in
`crates/lang_build`), but design material here must not be read as implemented
behavior.

## Authority

- `spec/public/` defines current behavior. If a design block and a public
  document appear to conflict, the public document wins for current behavior.
- These design documents constrain the intended direction. They are not a
  promise that the described semantics are implemented.
- Accepted ADR constraints have been absorbed into the relevant design blocks;
  they constrain direction but do not override `spec/public/`.

## Active semantic route

The blocks are ordered along the intended build-out route:

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
