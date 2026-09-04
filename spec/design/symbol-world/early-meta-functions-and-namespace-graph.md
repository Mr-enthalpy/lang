# Namespace graph and early semantic bootstrap

**Status:** implementation-facing consumer map. Canonical Object, type, Symbol,
Pattern, Policy, invocation, and construction meaning is owned by the focused
topic documents in this directory and `../patterns-overload/`.

## Goal

Build one persistent semantic world in which core and source declarations are
ordinary graph contributions:

```text
package/source roots
  -> physical namespace skeleton
  -> typed SemanticOwner qualification
  -> transactional declaration contribution
  -> one terminal Symbol per resolved path
  -> context projection and ordinary invocation
```

Names such as `struct`, `verify`, `type`, `uint8`, `ref`, and `share` are
ordinary graph entries, not parser keywords.

## Namespace graph invariants

- A source filename is not a namespace segment.
- Each namespace-role Object has one construction origin.
- A contribution is admitted atomically through a namespace delta.
- Physical directories authorize direct contributions to their namespace.
- A source or meta construction unit may build its newly created subtree but
  cannot reopen another unit’s closed subtree without explicit authority.
- Same-role duplicate children are hard conflicts.
- Object/function and namespace-subspace roles may share spelling while
  retaining distinct typed roles.
- Mount redirects preserve terminal Symbol identity.
- Internal and external views project the same admitted Symbol identities.
- Namespace lookup enumerates identities; overload selection happens later.

## Semantic owner graph

Owner qualification maps frontend owner/root identities into a parent-linked
persistent graph. It preserves callable owner, PatternRoot alpha boundary,
HoleBinder identity, package boundary, and MetaInstance parent placement.

```text
MetaInstanceRootKey
  = ParentSemanticOwner
  x selected callable identity
  x canonical whole argument Product identity
```

Root stability is independent of constness. Every MetaInstance root has
`PolicyMode=plain`, is a stable semantic owner, and does not thereby become
Writable.

## Policy and visibility

Every graph entry carries typed facts as applicable:

```text
PolicyPair
PolicyMode
NamespaceVisibility
export-root membership
CapabilityRealization
```

OpenStatic, SealStatic, and Runtime are visibility/evaluation phases. They do
not grant callable execution, capability, Writable, or construction authority.
External admission preserves candidate identity and stable Policy/capability
facts; consumer demand and DynamicLegality are applied after lookup.

## Core and early semantic operations

Core bootstrap supplies:

- rank and abstract literal complete types;
- ordinary callable/type-member entries;
- privileged AST-consuming `struct` construction;
- verification operations;
- registered construction/migration implementations;
- namespace and owner roots.

Bootstrap implementation does not create a separate language ontology.
`struct` follows the ordinary call pipeline and returns an exact complete type
value. Primitive execution material is installed before the CompleteType
semantic result is formed. Outer binding creates the destination Symbol/Place
and graph rendering.

## Call path

```text
ResolveName(path) = S
  -> CallableProjection(S) = Dedup(V_S union V_tau)
  -> InvocationFrame
  -> Pattern applicability
  -> Policy preference
  -> unique sealed invocation
  -> DynamicLegality
  -> execution
  -> InvocationResult
```

The exact immutable `tau` captured at value formation supplies `V_tau`.
Callability, applicability failure, selected failure, or result failure never
causes name resolution to search an outer same-name Symbol.

## Construction boundary

Meta and source construction use the same facts:

```text
WellFormed
OpenHere_Sigma
Writable
ConstructionAuthority
ActiveConstructionWindow
```

`extend` is a pure transform. `inject` is read+extend+write on an existing
writable Place. TypeMember contributions require direct-home evidence;
derived forwarding creates a fresh direct-home member that captures the base
complete-type snapshot.

## Pending consumers

The following are source/evaluator wiring work, not alternative semantics:

- block-local lexical alias entries;
- protected StructuralDefault extraction;
- operation-driven DynamicLegality premises;
- source ref/share/rebind and lifecycle actions;
- cleanup schedule production;
- Residual/Diagnostic continuation transport;
- derived associated forwarder formation;
- serial compile evaluation.

See `spec/planning/roadmap.md` for sequencing and
`spec/planning/open-questions.md` for representation choices.
