# Policy Visibility and Capability Mapping

Status: implementation-mapping companion. Canonical semantics are owned by
[`../symbol-world/symbol-policy-and-compile-flow-projection.md`](../symbol-world/symbol-policy-and-compile-flow-projection.md).

## 1. Typed semantic model

```text
PolicyPair = Pv:Pp
PolicyMode = const | plain | mut
```

`Pv:Pp` owns stage and value-presence shape. `PolicyMode` is a whole-slot
coordinate orthogonal to both `Pv:Pp` and `Val1` shape; it is not stored inside
`Pv`. Ordinary namespace visibility, export-root, and per-operation capability
realization are further independent coordinates. Policy syntax preserves `||`
choice, `+` cross-dimension conjunction, and `:` pair structure.

Semantic elaboration first factors one optional whole-slot `ModePattern` from
the complete surface policy and only then elaborates the residual `PairSpec` as
`Pv:Pp`. At most one connected mode Pattern is allowed; neither colon side may
contain its own semantic mode coordinate. Thus `const || mut` is one whole-slot
Pattern. The current rejection of `const:compile`, `runtime:const`,
`const:mut`, and `const || mut:compile` is a provisional surface rule, not a
consequence of orthogonality; a future contextual shorthand must still factor
mode exactly once and leave no mode coordinate in `Pv` or `Pp`. This is not a
new Raw/Normalized AST node.

P1 has three contextual elaborators:

```text
ordinary binding P1          -> identity-preserving slice restriction
formal parameter policy      -> inherit P2, then optional const/mut-only pattern slice
namespace declaration policy -> visibility plus optional export-root
```

A single ordinary P1 is value-dominant, not `Q:Q`. Its selected value stage set
is intersected with the requested set; the associated Pattern identity is
retained. Any non-empty projection completes ordinary binding elaboration.
Unselected alternatives in the P1 query are not missing-value demands.

## 2. P2 and function objects

Explicit P2 requires no runtime stage in Pp and equal static stage sets between
Pv and Pp whenever Pv has a static stage.

| Single P2 | Pair |
|---|---|
| `meta` | `meta:meta` |
| `compile` | `compile:compile` |
| `seal` | `seal:seal` |
| `runtime` | `runtime:compile` |
| `runtime || compile` | `(runtime || compile):compile` |
| `runtime || seal` | `(runtime || seal):seal` |

Function-object stage derivation is:

```text
Stage(P1p) = Stage(P2p)
Stage(P1v) = Stage(P2v) || Stage(P2p)
```

Only stages lift. `PolicyMode`, visibility, export-root, and value presence
come from the object declaration.

Each written formal parameter inherits P2 first. The first written formal is
the caller-object self Pattern even though its actual is passed implicitly;
later formals consume the explicit call-site Product. An omitted qualifier
keeps the pair unchanged and elaborates the formal mode to concrete `plain`;
`const let` / `mut let` restrict only its `PolicyMode` and do not alter any
other component. The function object's unwritten mode spelling likewise
elaborates directly to the real `plain` point; an explicit
declaration P1 may crop it. Namespace
declaration elaboration does not crop this complete internal view merely
because the declaration is exported. Stable external admission is determined
by export retention plus public path visibility; later consumer capability
checks do not rewrite namespace membership or the internal mode to `const`.

Formal elaboration has two consumers of the same result: the entered callable
body receives the effective pair and mode, and overload candidate formation
copies the mode into that parameter's external three-point product-order position.
Neither consumer may reconstruct a different policy.

## 3. Phase mapping

```text
Phase = OpenStatic | SealStatic | Runtime
```

| Stage | OpenStatic | SealStatic | Runtime |
|---|:---:|:---:|:---:|
| meta | yes | no | no |
| compile | yes | yes | no |
| seal | no | yes | no |
| runtime | no | no | yes |

Resolution and exposure are distinct. A `runtime:compile` symbol resolves in
OpenStatic, exposes no readable runtime value, but exposes its compile Pattern
and derived compile companion. Seal-only slices are hidden in OpenStatic but
their explicit paths are not semantically conflated with unresolved paths.

For `(compile || runtime):compile`, selecting the runtime Policy slice is also
distinct from reading it. The slice already exists extensionally, so demand
satisfaction does not invoke migration. In OpenStatic/SealStatic its runtime
value is still unreadable and remains residual. The later Runtime continuation
uses the already resolved Symbol/callable identity rather than reopening
ordinary namespace or overload selection.

Explicit-path resolution is authority-sensitive:

```text
InternalResolve(path) -> Σ_full
ExternalResolve(path) -> Σ_export
```

Neither operation is a Wpre/Wseal membership query. A symbol may belong to the
materialized world without being externally exposed, and an exported view is a
projection of the same full symbol identity rather than a second symbol.

The final authority is derived from package crossing. Lookup inside the same
package can consume `FullNameView`; after a path or mount enters another
package, lookup consumes `ExternalNameView`. A non-export declaration is
lexically visible from descendant semantic owners in the same package, but not
from an unrelated sibling merely because the sibling shares that package:

```text
LexicalInternalVisible(s, query)
  = SamePackage(DeclOwner(s), query)
    && AncestorOrSelf(DeclOwner(s), Owner(query))
```

This is separate from public/private path reachability and from export.

Ordinary seal code can explicitly resolve committed symbols. Only a
compiler-known privileged seal function can enumerate the fixed Wpre scan
domain; Wseal never enlarges it.

## 4. Export and visibility mapping

Export-root and public/private are independent:

```text
ExportRetentionClosure(s) = PathAncestors(s) ∪ Subtree(s)
ExternallyVisible(path) = Exported(path) && PubliclyReachable(path)
```

`export` is legal only at a namespace construction level's direct top-level.
Public/private may vary at every hierarchy layer and external access checks all
path components. Export retains a complete internal declaration view and
derives a separate external view:

```text
InternalView(value export) = full Pv:Pp
ExternalView(value export) = identity-preserving full Pv:Pp plus PolicyMode
InternalView(type export)  = absent:Pp
ExternalView(type export)  = absent:Pp
```

The absent value form has no hidden value stages, but it does not erase the
orthogonal whole-slot mode:

```text
Pv = absent
  => value stages = ∅
  && SemanticValueId = none

PolicyMode(absent:Pp slot) ∈ {const, plain, mut}
```

`const`, `plain`, and `mut` therefore all remain meaningful for a pure
type/Pattern slot. Stable external membership is decided by export-retention
closure plus public path reachability, not by a universal const projection or a
future consumer demand. Direct-root namespace-declaration elaboration may
preview those declaration-side admission facts; it does not create a resolved
consumer policy.

After declaration projection has been applied to actual RHS/result entries,
each candidate carries a resolved `PolicyPair`. External admission then
requires both export-retention-closure membership and public reachability
through every
path component. For each admitted symbol—including non-root ancestors or
descendants—every resolved candidate is transformed into an identity-preserving
`ExportCandidateView` whose external policy is another complete `PolicyPair`
plus its unchanged `PolicyMode`. The Pattern component and stable candidate/
family `CapabilityRealization` facts are preserved; no context-indexed dynamic
capability is stored.
No later call/read/capture capability filters this stable `Σ_export`.
Consumer-specific capability-family applicability and Policy demand are checked
after lookup, where the consumer forms `DynamicCapability_Γ_consumer` from its
operation, place, lifetime, and authority facts. `absent:Pp` is not
special-cased by mode. The generic policy parser and function-object stage
lifting do not perform these operations.

Namespace and Pattern consumers use three projections rather than treating
export as one universal visibility bit:

```text
FullNameView          complete package-internal name/overload view
ExternalNameView      export-retained, publicly reachable external candidates
DefaultExtractionView structural members exposed by default extraction
```

A private structural member remains in the full structural representation but
is absent from `DefaultExtractionView`. This is only the hard default boundary;
a future custom `?` design owns richer extraction-interface construction.
`Wpre/Wseal` membership remains orthogonal to all three views.

## 5. Rust substrate

The typed substrate currently provides:

- dedicated `PolicyConjunctionAst`, `PolicyChoiceAst`, and `PolicyAtomAst`;
- `PolicyPair` with typed dimensions and `Phase` with exactly three variants;
- separate binding/formal/namespace elaborators;
- formal elaboration that receives inherited P2 explicitly and preserves all
  non-mode dimensions;
- P2 normalization and stage-only function-object derivation;
- owned P1 restricted views rather than reference-only filtering;
- explicit resolution followed by phase exposure and facet reads;
- structural `CompleteSymbolFlow` projection;
- Wpre and export-retention least-closure helpers;
- complete and externally projected namespace overload-set carriers that
  require a typed `ExportAdmission { in_export_retention_closure,
  publicly_reachable }` before projection and
  preserve candidate identity while storing a distinct resolved `PolicyPair`
  on each `ExportCandidateView`;
- phase-aware overload preference combined with the current Policy-mode
  carrier;
- atomic builtin type-key / concrete numeric Tnum separation and current
  first-order TypeValue projections. These registries perform concrete type
  lookup only; they do not implement abstract literal denotations;
- a helper that first projects the complete binding query and, only when that
  is empty, extracts an accepted runtime branch for atomic migration, with a
  projection-only pure-type branch;
- a transitional migration candidate adapter whose endpoint preference is
  `input x output`, uses the shared maximal-element rule, preserves delete
  rejection, permits callable-declared endpoint `PolicyMode`, cannot change
  Type, and performs no transitive search. Its endpoint-only maxima helper is
  private and is not a sequentially composable implementation of full Bp';
- a parent-linked semantic-owner graph plus an owner-aware namespace forest
  substrate with explicit package boundaries, identity-preserving mount
  redirects, Full/External view routing, and typed lookup failures.

The older `PolicyFlag`/`PolicySet` path and const-projected export adapter remain
compatibility transport. They are lossy: they cannot represent the three real
`PolicyMode` points, the 3×3 capability-realization space, choice syntax,
Pattern association after cropping, or independent public/private and
export-root state. The resolver now uses the three canonical phase names, but
not every namespace graph operation stores a full `PolicyPair` plus
`PolicyMode` yet. Compatibility behavior must not redefine the typed contract.

## 6. Guardrails

- Policy words remain contextual names, not lexer keywords.
- Pattern `|` is never policy choice; policy choice is `||`.
- Single P2 `runtime` normalizes to `runtime:compile`.
- Explicit `runtime:seal` remains valid.
- P1 projection crops an exposed slice.
- A non-empty ordinary P1 projection never manufactures absent query
  alternatives and makes migration unreachable.
- After the complete existing projection is empty, an accepted runtime
  alternative may be extracted as the constructible branch; other alternatives
  are not manufactured.
- Policy slicing of `Pp` does not extract, navigate, reroot, or otherwise
  transform a PatternValue.
- Atomic migration mandates only the static-to-runtime stage edge, unchanged
  Type, present output, and unchanged selected `Pp`; callable-declared
  `PolicyMode` endpoints may differ and participate in Bp'.
- Policy failure cannot repair Type/Pattern structural inapplicability.
- Runtime value invisibility never deletes the symbol or its Pattern facet.
- Runtime Policy-slice existence does not imply present-phase value
  readability.
- Meta is not exposed in SealStatic.
- Seal policy grants no enumeration capability.
- `@` remains lifetime syntax and cannot alter completed ordinary overload
  selection.
