# Policy Visibility and Capability Mapping

Status: implementation-mapping companion. Canonical semantics are owned by
[`../symbol-world/symbol-policy-and-compile-flow-projection.md`](../symbol-world/symbol-policy-and-compile-flow-projection.md).

## 1. Typed semantic model

```text
PolicyPair = Pv:Pp
```

Its dimensions are stage, value mutability, value presence, ordinary namespace
visibility, and export-root. They are not one flag set. Policy syntax preserves
`||` choice, `+` cross-dimension conjunction, and `:` pair structure.

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

Only stages lift. Mutability, visibility, export-root, and value presence come
from the object declaration.

Each written formal parameter inherits P2 first. The first written formal is
the caller-object self Pattern even though its actual is passed implicitly;
later formals consume the explicit call-site Product. An omitted qualifier
keeps it unchanged; `const let` / `mut let` restrict only its mutability Pattern
and do not alter any other component. The function object itself defaults to an empty
mutability restriction, whose typed-domain meaning is the full
`const || mut` choice; an explicit declaration P1 may crop it. Namespace
declaration elaboration does not crop this complete internal view merely
because the declaration is exported. A value-bearing exported candidate must
admit a const projection. Its external candidate view is const-projected; a
mut-only value candidate is therefore externally ineligible, while
`const || mut` is valid.

Formal elaboration has two consumers of the same result: the entered callable
body receives the effective pair, and overload candidate formation copies the
qualifier into that parameter's external const/mut product-order position.
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
ExternalView(value export) = Project_const(Pv):Pp
InternalView(type export)  = absent:Pp
ExternalView(type export)  = absent:Pp
```

The absent value form has no hidden value subdimensions:

```text
Pv = absent
  => value stages = ∅
  && value mutability = ∅
```

Accordingly `const + S : compile` and `mut + S : compile` are invalid before
export projection; adding `export` does not make either form valid.

An omitted mutability domain and a written `const || mut` domain are valid when
their const projection is non-empty. A `mut`-only value export is invalid. A
pure type/Pattern export has no value-mutability requirement. This projection
is previewed/validated for a direct root by namespace-declaration elaboration.
That preview remains declaration-side `P1Projection`; it is not a resolved
external policy.

After declaration projection has been applied to actual RHS/result entries,
each candidate carries a resolved `PolicyPair`. External admission then
requires both export-retention-closure membership and public reachability
through every
path component. For each admitted symbol—including non-root ancestors or
descendants—every policy-eligible candidate is transformed into an
identity-preserving
`ExportCandidateView` whose external policy is another complete `PolicyPair`.
The Pattern component is preserved. Mut-only candidates stay in `Σ_full` and
are filtered from `Σ_export`; `absent:Pp` candidates enter unchanged. The
generic policy parser and function-object stage lifting do not perform these
operations.

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
  non-mutability dimensions;
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
- phase-aware overload preference combined with const/mut product order.
- atomic builtin type-key / concrete numeric Tnum separation, current
  first-order TypeValue projections, and context-selected literal typing;
- a helper that first projects the complete binding query and, only when that
  is empty, extracts an accepted runtime branch for atomic migration, with a
  projection-only pure-type branch;
- a transitional migration candidate adapter whose endpoint preference is
  `input x output`, uses the shared maximal-element rule, preserves delete
  rejection, permits callable-declared endpoint mutability, cannot change
  Type, and performs no transitive search. Its endpoint-only maxima helper is
  private and is not a sequentially composable implementation of full Bp';
- a parent-linked semantic-owner graph plus an owner-aware namespace forest
  substrate with explicit package boundaries, identity-preserving mount
  redirects, Full/External view routing, and typed lookup failures.

The older `PolicyFlag`/`PolicySet` path remains compatibility transport. It is
lossy: it cannot represent choice syntax, Pattern association after cropping,
or independent public/private and export-root state. The resolver now uses the
three canonical phase names, but not every namespace graph operation stores a
full `PolicyPair` yet. Compatibility behavior must not redefine the typed
contract.

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
  mutability endpoints may differ and participate in Bp'.
- Policy failure cannot repair Type/Pattern structural inapplicability.
- Runtime value invisibility never deletes the symbol or its Pattern facet.
- Runtime Policy-slice existence does not imply present-phase value
  readability.
- Meta is not exposed in SealStatic.
- Seal policy grants no enumeration capability.
- `@` remains lifetime syntax and cannot alter completed ordinary overload
  selection.
