# symbol-world

**Status: Non-normative future design with a partial implementation note. A
narrow namespace-graph / resolver / early-meta slice exists in
`crates/lang_build`; `TypeValueId`, borrow views, writability and
extension-eligibility checking,
`SymbolCell` facets, `PatternValue`, `SymbolConstruction`,
`ResolvedPatternScope`, `MetaInstanceScopeId`, meta type self-root checking,
functional `inject`, `NamespaceOrigin`, construction-unit ownership,
physical/cross-file contribution authority, field-access evaluation, and
access-tree construction are not implemented. Layered symbol policy,
compile-flow projection, complete derived compile-companion objects,
must-select overload consistency, coarse inferred require, and shared compile
evaluation are also not implemented.**

## Scope

The namespace graph world model and symbol-level identity:

- `SymbolObject` and the namespace graph world model
- symbol-first `SymbolCell` facets and context-directed projection
- the `SymbolId` / `PlaceId` / `TypeValueId` distinction
- `PatternValue`, `compile` / `meta`, and rank-directed canonical identity
- `Val1 x Pattern x Val2`, canonical `Pv:Pp`, contextual P1 projection, P2
  result normalization, derived function-object stage views, seal visibility,
  const/mut product order, compile-flow projection over ordinary call nodes,
  complete derived compile-companion objects, coarse inferred require, and
  shared compile evaluation
- resolved pattern scopes, `struct` ownership, functional child-only `inject`,
  and binding/install separation
- meta-return type self-root identity and complete meta-instance navigation atoms
- namespace-facet origin, source/meta construction-unit ownership, physical
  contribution authority, and cross-file closure
- the borrow views `ref` / `share` / `@`, writability, and extension eligibility
- field functions, `ref` / `share` projection namespaces
- type-associated function objects and namespace extension targets
- the early-meta / namespace-graph bootstrap (broad bootstrap document)

## Not in scope

Pattern/overload candidate adaptation, meta invocation execution, and the full
policy checker (referenced from the other blocks).

## Documents

- `symbol-first-meta-construction-and-pattern-injection.md` — canonical future
  direction for SymbolCell facets, heterogeneous value/call candidates,
  `compile` / `meta`, meta type self-root, resolved pattern scopes, `struct`,
  functional `inject`, pattern-layer ordering, uniqueness/replay, and outer
  graph installation.
- `symbol-construction-units-and-namespace-origin.md` — canonical future
  namespace-origin, source/meta construction-unit ownership, physical-directory
  authority, type/namespace facet inclusion, value-member/pattern-material
  separation, and current cross-file closure.
- `symbol-policy-and-compile-flow-projection.md` — canonical future `Pv:Pp`,
  P1/P2 contextual elaboration, function-object stage derivation, seal
  three-phase visibility, Wpre scanning versus explicit seal lookup,
  independent export-root/public-private dimensions, const/mut product order,
  mechanical compile projection, derived companion objects,
  must-select consistency, match/D/Done, coarse require, and shared evaluation.
- `early-meta-functions-and-namespace-graph.md` — the build / namespace graph
  bootstrap and early-meta `struct` / `verify` slice. This document is broad;
  once the symbol world stabilizes it may be split further.
- `type-values-places-and-borrow-views.md` — canonical `TypeValueId` /
  `PlaceId` / `SymbolId` distinction, object normal form, and the borrow views
  `ref` / `share` / `@`.
- `type-associated-function-objects-and-access-trees.md` — field functions,
  projection namespaces, access-tree implications.
- `entity-ref-design.md` — general `EntityRef` design (alias-RHS subset
  implemented as raw AST preservation).
- `entity-alias-design.md` — historical record of the frozen surface
  `let binder === EntityRef` syntax. **Its semantic alias-forwarding model is
  retired**; borrow views replace it.

## Reading order

Read `symbol-first-meta-construction-and-pattern-injection.md` first for the
canonical future semantic boundary. Then read
`symbol-construction-units-and-namespace-origin.md` for creation/ownership and
cross-file rules. Then read `symbol-policy-and-compile-flow-projection.md` for
policy, staging, companion, and require semantics. Read
`early-meta-functions-and-namespace-graph.md` next for
the current bootstrap route, followed by the type-value/place/borrow-view and
field-function documents.

For v0.8-adjacent work that touches `TypeValueId`, `PlaceId`, borrow views,
generated meta instances, extension-place checking, or the current `struct`
vertical slice, read
`spec/contracts/v0.8-meta-construction-agent-constraints.md` first. That
contract makes these objects implementation preconditions for generic
type-style meta construction, not optional future commentary.

## Dependencies

Builds on `build-package/` (roots, mounts, provenance). Provides `TypeValueId`
to `patterns-overload/` and the symbol world to `meta-invocation/`. Final
symbol-flow policy is defined here; `policy-capability/` maps current metadata
and owns future orthogonal policy dimensions.
