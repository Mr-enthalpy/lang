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
  -> ordinary E evaluation of the runtime-horizon entry
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
- Every legally completed closure expression returns tau_C; file installation
  and true lexical binding retain their distinct destinations.
- Explicit structural contribution roles synthesize a named type's V_tau.
- Sibling blocks use common-snapshot overlays and ordinary unordered join.
- Name occupancy is independent of value content and visibility.
- Internal/external views retain semantic identity; overload selection is later.
- Storage transactions realize the enclosing semantic action, not file authority.

## Semantic owner graph

Owner qualification maps frontend owner/root identities into a parent-linked
persistent graph. It preserves callable owner, PatternRoot alpha boundary,
HoleBinder identity and CompileInstance parent placement.

```text
CompileInstanceKey
  = ParentSemanticOwner
  x selected callable identity
  x CanonicalizeInvocationInputs(In)
```

Every CompileInstance is a stable semantic owner, formed before selected body
entry and independent of result kind or P2. Its ordinary self-name is readable
without reentry when initialized. A single direct instance-open self-root type
result supplies a computed NameExpr; ordinary/external results preserve their
own identity. OpenPolicy=open retains established opening sources; close
completes and closes after ordinary result delivery. Current storage and prior
immutable snapshots remain distinct.

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
- structural interpretation/type formation and ordinary struct helpers;
- verification operations;
- registered construction/migration implementations;
- namespace and owner roots.

Bootstrap implementation does not create a separate language ontology.
Main has runtime P2. Stable roots and active body evaluation are independent.
CompileInstance identity is compatible with seal P2. Visibility, Ready and
LegalToExecute are independent consumers. struct:type->type augments an
already complete structural type without changing its structural registrations.

## Call path

```text
ResolveName(path) = S
  -> read the callee
  -> ordinary x: Type(x).associated Val2[()], self=x
     type tau: union Type(c).associated Val2[()] entries for every c in V_tau, self=c
     explicit group: union of its type-callee projections
  -> R_vis visibility/input evidence and ordinary C_sigma preparation
  -> InvocationFrame
  -> hard Pattern applicability and total output demand
  -> fallback suppression
  -> Policy preference
  -> unique sealed invocation
  -> DynamicLegality
  -> execution
  -> InvocationResult
```

A type callee's immutable tau supplies V_tau. An ordinary value does not
project its classifier's V_tau; its associated Val2[()] supplies the entrance.
Callability, applicability failure, selected failure, or result failure never
causes name resolution to search an outer same-name name binding.

## Construction boundary

Structural construction and ordinary source actions use the same facts:

```text
WellFormed
OpenHere_Sigma
Writable
ConstructionAuthority
ActiveConstructionWindow
```

* is ordinary type composition; *= performs read-transform-write on an actual
writable type reference with all ordinary Pre premises. Type contribution requires final classifier home Home(TypeOf(v)) = TypeMemberScope(T).
Eligible closure expressions can be instantiated under another anchor while
preserving the original value. Derived forwarders capture the base complete
snapshot. A is an ordinary compile instance type with an ordinary Val2 group
member; its state uses the general invocation cache facilities.

## Pending consumers

The following are source/evaluator wiring work, not alternative semantics:

- block-local lexical alias entries;
- protected StructuralDefault extraction;
- operation-driven DynamicLegality premises;
- source ref/share/rebind and lifecycle actions;
- cleanup schedule production;
- Residual/Diagnostic continuation transport;
- derived associated forwarder formation;
- instance/member residency, P1 open qualification, input dependency normalization
  and opening-source propagation, including the derived A instance;
- serial compile evaluation.

See `spec/planning/roadmap.md` for sequencing and
`spec/planning/open-questions.md` for representation choices.
