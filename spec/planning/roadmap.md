# Roadmap

This document records current implementation frontiers. Canonical meaning is
owned by the topic documents under `spec/design/`; unresolved decisions are
owned by `spec/planning/open-questions.md`.

## Frontend

The frontend pipeline is:

```text
source text -> tokens -> Raw AST -> Normalized AST (+ diagnostics)
```

Raw AST preserves source shape. Normalized AST is desugared but remains
non-semantic; it is not HIR and does not perform lookup, type checking,
Pattern interpretation, lifetime validation, or evaluation.

## Current semantic architecture

The semantic pipeline is:

```text
Normalized AST
  -> typed owner / namespace resolution
  -> canonical semantic entities and views
  -> candidate enumeration and relational Pattern applicability
  -> Policy preference
  -> unique sealed invocation
  -> DynamicLegality
  -> execution
  -> InvocationResult
```

The implementation in `crates/lang_build` establishes the following positive
semantic vocabulary:

- `Object = <Val1?, Pattern, Val2>` with complete ordinary normalization;
- proof-relevant `R_Gamma(P,c,rho)` Pattern applicability and Hole valuation;
- complete `tau = bind alpha.<Core(tau), V_tau[alpha]>` with immutable callspace
  snapshots;
- separate Symbol, semantic value, Place, resident generation, and
  ProjectionSlot identities;
- `PolicyPair`, primitive `PolicyMode = {const, plain, mut}`,
  `ResultPolicyDemand`, and independent 3×3 `CapabilityRealization`;
- one name-resolution result followed by context projection;
- value → exact complete type → associated `()` call projection;
- sealed candidate selection, post-selection DynamicLegality, and no reopen;
- candidate-driven same-Type Policy migration and PolicyLet result-demand
  boundaries;
- exact abstract literal values followed by ordinary construction;
- `OpenHere`, ConstructionAuthority, Writable, pure `extend`, and place-level
  `inject`;
- unified `InvocationResult`, complete-type `struct` result, and structural
  MetaInstance root identity;
- shared SemanticContinuation substrate with LifeName, Region, Pre/Post, and
  an extensible directed Color algebra.

Storage, graph rendering, and primitive execution are implementation layers;
identity, selection, result class, and legality remain properties of the
semantic relations above.

## Semantic substrate coverage

| Relation | Carrier | Production consumer | Status |
|---|---|---|---|
| Object Norm | Val1/Pattern/owned-Val2 observation | equality, Core and argument identity | Implemented |
| complete tau | Core + immutable V_tau + whole observation | type binding and ordinary call projection | Implemented |
| base R_Gamma and Hole valuation | relational proof | ordinary parameter A-stage | Implemented |
| DirectPatternChild + StructuralDefault | relation interfaces | protected structural extraction | Consumer pending |
| PolicyMode / demand / preference | explicit PolicyView and ResultPolicyDemand | ordinary selection, migration, PolicyLet | Implemented |
| CapabilityRealization | candidate-local 3×3 table | selected operation premise formation | Consumer pending |
| Place / resident generation | Place and ProjectionSlot | binding, Writable and borrow substrate | Implemented; source operation coverage pending |
| DynamicLegality | sealed post-selection validator | supplied capability/place/lifecycle premises | Implemented; automatic premise formation pending |
| InvocationResult | declared result class + semantic payload/residual/diagnostic | connected ordinary and core/meta invocation | Implemented; residual transport remains Open |
| OpenHere / construction | authority, window, Writable and write algebra | meta construction and inject | Implemented |
| abstract literals | exact abstract values and construction requests | annotated construction and Policy migration | Implemented |
| SemanticContinuation | lifecycle machine and event ledger | world-owned registration | source action/cleanup wiring pending |
| Color/access | extensible directed relations and provider interface | lifecycle Pre validation | access-tree construction Open |

“Consumer pending” means the canonical relation exists and no substitute
relation is used; it does not mean the language rule is undecided.

## Source and evaluator connection frontier

The next implementation frontier connects source occurrences to the existing
relations without changing their meaning:

1. protected structural extraction through `StructuralDefault` before C0;
2. operation-driven capability, Writable, authority, and lifecycle premises;
3. source `ref` / `share` / `rebind` and invalidation actions;
4. source use/move/drop/`@` events on the world-owned continuation;
5. cleanup placement before lifecycle observation;
6. Residual and Diagnostic transport through the unified invocation boundary;
7. derived associated forwarding that captures the base complete-type
   snapshot and creates fresh direct-home members;
8. block-local `let ===` lexical entries that create no semantic entity.

Each wiring step must preserve unique selection and no reopen.

## Serial compile evaluation

After source operations are connected, the evaluator may execute them along a
single semantic continuation:

```text
resolved operation
  -> Pre
  -> committed action
  -> Post
  -> next SemanticContinuation position
```

This frontier owns control-flow sequencing, fixed cleanup placement, residual
continuation transport, and serial compile evaluation. It does not create a
separate meta value ontology or lifetime universe.

## Bootstrap and source authority

A compiler implementation is not evidence that an operation is a permanent
semantic primitive. Every builtin family is classified by its target role:

| Role | Meaning |
|---|---|
| `BootstrapSeed` | establishes the initial source-expressible environment |
| `SourceDefinitionPending` | language semantics can express the operation; source definition is not connected yet |
| `IntrinsicObservation` | exposes implementation facts that source cannot synthesize, without deciding legality |
| `SemanticPrimitive` | permanent authority, requiring an explicit non-bootstrappability proof |

Current families:

| Family | Role | Semantic authority |
|---|---|---|
| exact abstract-literal formation | `BootstrapSeed` | exact spelling/family observation |
| concrete literal constructors | `SourceDefinitionPending` | ordinary candidate selection |
| construction and same-Type migration families | `SourceDefinitionPending` | ordinary selection + DynamicLegality |
| capability realization entries | `SourceDefinitionPending` | candidate declarations |
| StructuralDefault providers | `SourceDefinitionPending` | `R_Gamma` |
| lifecycle move/copy/drop algebra | `SourceDefinitionPending` | lifecycle Pre/commit/Post relations |
| interning, graph allocation, continuation-position observation | `IntrinsicObservation` | canonical relations consuming those observations |

No current family is classified as `SemanticPrimitive`.

## Open representation boundaries

The implementation must provide extension points without choosing final forms
for:

- `TypeValueId` storage encoding;
- full Pattern canonical-space representation;
- Color syntax and storage;
- access-tree construction;
- persistent owner/root encoding;
- complete later overload filters and named strategies;
- Residual IR and continuation ABI;
- cleanup scheduling IR;
- character surface and machine type catalog;
- closure capture layout, HIR, backend lowering, and code generation.

These questions remain in `spec/planning/open-questions.md`. Missing source
wiring is not an open semantic question.

## Build/package track

The build system assembles a namespace graph from package manifests, source
roots, physical namespaces, explicit mounts, and source contributions. Source
filenames are not namespace segments. The language has no source-level
import/use/include/module/package syntax.

The build layer provides package identity, transactional namespace
deltas, provenance, typed owner qualification, role-aware name admission,
default core mounting, and cache validation. Remote retrieval, full version
solving, lockfiles, package distribution policy, and persistent root encoding
remain future work.
