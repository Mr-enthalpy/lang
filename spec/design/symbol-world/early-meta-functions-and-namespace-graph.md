# Namespace graph and early semantic bootstrap

**Status:** implementation-facing consumer map. Canonical Object, type, name binding,
Pattern, Policy, invocation, and construction meaning is owned by the focused
topic documents in this directory and `../patterns-overload/`.

## Goal

Build one persistent semantic world in which core and source declarations are
ordinary graph contributions:

```text
compilation Level with main.lang anchor
  -> neutral physical block normalization
  -> ordinary source meta evaluation
  -> typed SemanticOwner qualification
  -> transactional declaration contribution
  -> one terminal name binding per resolved path
  -> context projection and ordinary invocation
```

Names such as `struct`, `verify`, `type`, `uint8`, `ref`, and `share` are
ordinary graph entries, not parser keywords.

## Namespace graph invariants

- Physical files supply provenance, not identity or construction authority.
- Source actions create names and Objects under ordinary capability rules.
- Same-name named-contribution positions synthesize a named type's V_tau.
- Sibling blocks use common-snapshot overlays and ordinary unordered join.
- Name occupancy is independent of value content and visibility.
- Internal/external views retain semantic identity; overload selection is later.
- Storage transactions realize the enclosing semantic action, not file authority.

## Semantic owner graph

Owner qualification maps frontend owner/root identities into a parent-linked
persistent graph. It preserves callable owner, PatternRoot alpha boundary,
HoleBinder identity and MetaInstance parent placement.

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
semantic result is formed. Outer binding creates the destination name binding/Place
and graph rendering.

## Call path

```text
ResolveName(path) = S
  -> CallCandidates(NamedType(S)), or the explicit group's candidate projection
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
causes name resolution to search an outer same-name name binding.

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
writable Place. Type contribution requires final TypeOf(v) membership in the target core.
Eligible closure expressions can be instantiated under another anchor while
preserving the original value. Derived forwarders capture the base complete
snapshot. A[t] is ordinary guarded compile state, not graph metadata.

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
