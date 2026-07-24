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
retained.

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

Each written formal parameter inherits P2 first. An omitted qualifier keeps it
unchanged; `const let` / `mut let` restrict only its mutability Pattern and do
not alter any other component. The function object itself defaults to an empty
mutability restriction, whose typed-domain meaning is the full
`const || mut` choice; an explicit declaration P1 may crop it. Namespace
declaration elaboration has one intentional exception: bare `export let`
defaults the independent mutability axis to `const`. Any written export policy
whose mutability domain contains `mut` is invalid.

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

Explicit-path resolution is authority-sensitive:

```text
InternalResolve(path) -> Σ_full
ExternalResolve(path) -> Σ_export
```

Neither operation is a Wpre/Wseal membership query. A symbol may belong to the
materialized world without being externally exposed, and an exported view is a
projection of the same full symbol identity rather than a second symbol.

Ordinary seal code can explicitly resolve committed symbols. Only a
compiler-known privileged seal function can enumerate the fixed Wpre scan
domain; Wseal never enlarges it.

## 4. Export and visibility mapping

Export-root and public/private are independent:

```text
ExportClosure(s) = PathAncestors(s) ∪ Subtree(s)
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

An omitted mutability domain and a written `const || mut` domain are valid when
their const projection is non-empty. A `mut`-only value export is invalid. A
pure type/Pattern export has no value-mutability requirement. This projection
is previewed/validated for a direct root by namespace-declaration elaboration.
That preview remains declaration-side `P1Projection`; it is not a resolved
external policy.

After declaration projection has been applied to actual RHS/result entries,
each candidate carries a resolved `PolicyPair`. Export closure then admits
symbols—including non-root ancestors or descendants—and each externally
eligible candidate is transformed into an identity-preserving
`ExportCandidateView` whose external policy is another complete `PolicyPair`.
The Pattern component is preserved. Mut-only candidates stay in `Σ_full` and
are filtered from `Σ_export`; `absent:Pp` candidates enter unchanged. The
generic policy parser and function-object stage lifting do not perform these
operations.

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
- Wpre and export least-closure helpers;
- complete and externally projected namespace overload-set carriers that
  preserve candidate identity while storing a distinct resolved `PolicyPair`
  on each `ExportCandidateView`;
- phase-aware overload preference combined with const/mut product order.

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
- Runtime value invisibility never deletes the symbol or its Pattern facet.
- Meta is not exposed in SealStatic.
- Seal policy grants no enumeration capability.
- `@` remains lifetime syntax and cannot alter completed ordinary overload
  selection.
