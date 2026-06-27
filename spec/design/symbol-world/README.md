# symbol-world

**Status: Non-normative future design with a partial implementation note. A
narrow namespace-graph / resolver / early-meta slice exists in
`crates/lang_build`; `TypeValueId`, alias forwarding, writable-place checking,
field-access evaluation, and access-tree construction are not implemented.**

## Scope

The namespace graph world model and symbol-level identity:

- `SymbolObject` and the namespace graph world model
- the `SymbolId` / `PlaceId` / `TypeValueId` distinction
- alias forwarding (`AliasChain`) and writable-place checking
- field functions, `ref` / `share` projection namespaces
- type-associated function objects and namespace injection targets
- the early-meta / namespace-graph bootstrap (broad bootstrap document)

## Not in scope

Pattern/overload candidate adaptation, meta invocation execution, and the full
policy checker (referenced from the other blocks).

## Documents

- `early-meta-functions-and-namespace-graph.md` — the build / namespace graph
  bootstrap and early-meta `struct` / `verify` slice. This document is broad;
  once the symbol world stabilizes it may be split further.
- `type-values-places-and-alias-forwarding.md` — canonical `TypeValueId` /
  `PlaceId` / `SymbolId` distinction and alias forwarding.
- `type-associated-function-objects-and-access-trees.md` — field functions,
  projection namespaces, access-tree implications.
- `entity-ref-design.md` — general `EntityRef` design (alias-RHS subset
  implemented as raw AST preservation).
- `entity-alias-design.md` — surface `let binder === EntityRef` syntax and its
  future semantic forwarding meaning.

## Dependencies

Builds on `build-package/` (roots, mounts, provenance). Provides `TypeValueId`
to `patterns-overload/` and the symbol world to `meta-invocation/`. Policy
planes are defined in `policy-capability/`.
