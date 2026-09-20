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
contain its own semantic mode coordinate. Concrete ModeAtom is const/plain/mut;
an explicit Pattern hole supplies a HoleRef instead. A surface `PolicyChoice` containing more
than one ModeAtom, including `const || mut`, is preserved by
Raw/Normalized syntax but rejected by typed Policy elaboration. Resolved stages likewise contain one atom; multi-stage unions are invalid.
Solver alternatives remain possible until they yield concrete solutions. The
current rejection of `const:compile`, `runtime:const`, and `const:mut` is an
empty-residual-side surface rule, not a consequence of
orthogonality; a future contextual shorthand must still factor mode exactly
once and leave no mode coordinate in `Pv` or `Pp`. This is not a new
Raw/Normalized AST node. No written ModeAtom means no explicit override.
Inherited/contextual constraints and a separately applicable default completion
determine any concrete demand before maxima; omission is not explicit plain.

Policy positions have contextual elaborators:

```text
ordinary binding P1          -> identity-preserving slice restriction
formal parameter policy      -> Pin = ElabIn(P2, Delta_in)
return-position policy       -> Pout = ElabOut(P1, Delta_out)
namespace declaration policy -> visibility plus optional export-root
```

A concrete accepted existing view satisfies a demand without reconstruction.
No unresolved solver alternative manufactures a missing view.

## 2. P2 and function objects

P2 is the evaluation horizon, P1/Pout producer visibility, InputAdmissible the
input relation and Ready the current execution condition. Resolved Stage is
{meta,compile,seal,runtime}; the order contains only identity and the three
static-to-runtime edges. Static atoms are mutually incomparable.

Pv:Pp remains an observation pair: runtime:compile and runtime:seal are valid.
Omitted ordinary P1 stage defaults from runtime P2 to runtime, seal to seal,
compile to compile; contextual meta qualification has its separate owner.
Explicit P1 is never overwritten and bare let is not a late wildcard.

Pin inherits P2 with explicit stage/mode atoms or ordinary holes where written.
Pout inherits P1's stage and permits mode refinement. A runtime callable can
therefore have heterogeneous compile/runtime Pins. Compile can admit a seal
input and defer through InputAdmissible/Ready without a seal-to-compile
migration. Self remains the first written formal, supplied implicitly.

Formal elaboration feeds the same position facts to body entry and ordinary
candidate comparison. Namespace export retains identity and stable declaration
facts; later consumer demand and DynamicLegality create no second view owner.

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

For ordinary call evaluation, the current `Phase` is already known. When no
explicit target pair/stage Policy is written, each candidate's evaluation P1
stage view may use the applicable stage-only default completion in §2 and is then
checked against this table. Therefore `compile`/`runtime` exposure does not
require `PolicyLet`; that syntax remains an optional explicit result boundary.
The phase rule does not choose whole-slot mode: no written constraint, explicit
plain/const/mut and an explicit hole remain distinct. Result demand must be
resolved from the actual context/completion before maxima; inner selection seals.

Resolution and exposure are distinct. A name binding whose resident has a
`runtime:compile` view resolves in OpenStatic. Subsequent resident projection
exposes no readable runtime value, but exposes its compile Pattern and derived
compile companion. Seal-only slices are hidden in OpenStatic but
their explicit paths are not semantically conflated with unresolved paths.

A declared runtime view can exist while its Val1 is unreadable at a static
frontier. Hiding that Val1 preserves the Object, Pattern, Val2 and argument
slot. R_vis prepares admissible C_sigma projections before ordinary hard A,
fallback suppression and Policy/Pattern selection. Once selected, every
projection and runtime residue keeps the same origin and frame.

Explicit-path resolution is authority-sensitive:

```text
InternalResolve(path) -> Σ_full
ExternalResolve(path) -> Σ_export
```

Neither operation is a Wpre/Wseal membership query. A name binding may exist in
the materialized world without being externally exposed. Its exported candidate
views preserve the resident's candidate-entry identities and create no second
binding or wrapper Object.

Source-established namespace and access relations determine whether lookup
uses `FullNameView` or `ExternalNameView`. Lexical internal visibility requires
the permitted source-defined domain and an ancestor-or-self declaration owner.
Physical package boundaries and configured mounts establish neither relation.

This is separate from public/private path reachability and from export.

Ordinary seal code can explicitly resolve committed name bindings. Only a
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
path component. For each admitted name binding—including non-root ancestors or
descendants—every resolved candidate is transformed into an identity-preserving
`ExportCandidateView` whose external policy is another complete `PolicyPair`
plus its unchanged `PolicyMode`. The Pattern component and stable candidate/
family `CapabilityRealization` facts are preserved; no context-indexed dynamic
legality judgment is stored.
No later call/read/capture legality check filters this stable `Σ_export`.
Consumer Policy demand and capability-family realization are handled after
lookup by ordinary selection; the consumer then forms
`DynamicLegality_Γ_consumer` for the selected invocation from its place,
lifetime, access, escape, and authority facts. `absent:Pp` is not
special-cased by mode. The generic policy parser and function-object stage
completion do not perform these operations.

Namespace and Pattern consumers use three projections rather than treating
export as one universal visibility bit:

```text
FullNameView          complete permitted internal name/type view
ExternalNameView      export-retained, publicly reachable external candidates
DefaultExtractionView structural members exposed by default extraction
```

A private structural member remains in the full structural representation but
is absent from `DefaultExtractionView`. This is only the hard default boundary;
a future custom `?` design owns richer extraction-interface construction.
`Wpre/Wseal` membership remains orthogonal to all three views.

## 5. Rust substrate

The following carriers are implementation inventory, not evidence that the
single-stage or R_vis/C_sigma model is connected. StageSet and union-accepting
helpers/tests still require migration; the mapping below is not normative algebra. Existing helpers that insert Plain
for every omitted binding or call demand need alignment; operator-Pattern policy
deduction and the joint Pin/Pout solution relation remain pending consumers.

The typed substrate currently provides:

- dedicated `PolicyConjunctionAst`, `PolicyChoiceAst`, and `PolicyAtomAst`;
- `PolicyPair` with typed dimensions and `Phase` with exactly three variants;
- separate binding/formal/namespace elaborators;
- formal elaboration that receives inherited P2 explicitly and preserves all
  non-mode dimensions;
- P2 normalization and stage-only function-object derivation;
- owned P1 restricted views rather than reference-only filtering;
- explicit resolution followed by phase exposure and facet reads;
- `CompleteSymbolFlow` projection (legacy Rust carrier name, not a canonical
  Symbol Object or a binding facet);
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
- candidate-driven same-Type migration whose endpoint preference is
  `input x output`, uses the ordinary maximal-element rule, preserves delete
  rejection, permits callable-declared endpoint `PolicyMode`, and performs no
  transitive search;
- a parent-linked semantic-owner graph plus an owner-aware namespace forest
  substrate with Full/External view routing and typed lookup failures.
  Its configured package/mount routing remains an implementation migration
  gap; source evaluation must establish the namespace and authority facts.

Namespace entries and call candidates retain typed `PolicyPair`, concrete
`PolicyMode`, visibility/export facts, and capability realization without a
scalar policy projection.

## 6. Guardrails

- Policy words remain contextual names, not lexer keywords.
- Pattern `|` is never policy choice; policy choice is `||`.
- Runtime horizon uses runtime:compile for ordinary value/Pattern observation.
- Explicit `runtime:seal` remains valid.
- P1 projection crops an exposed slice.
- A non-empty ordinary P1 projection never manufactures absent query
  alternatives and makes migration unreachable.
- After existing projection is empty, a concrete runtime demand may admit one
  direct same-Type migration; solver alternatives are not manufactured views.
- Policy slicing of `Pp` does not extract, navigate, reroot, or otherwise
  transform a PatternValue.
- Atomic migration mandates only the static-to-runtime stage edge, unchanged
  Type, present output, and unchanged selected `Pp`; callable-declared
  `PolicyMode` endpoints may differ and participate in Bp'.
- Policy failure cannot repair Type/Pattern structural inapplicability.
- Runtime value invisibility never deletes the name binding or erases its
  resident's independently exposed Pattern view.
- Runtime Policy-slice existence does not imply present-phase value
  readability.
- Meta is not exposed in SealStatic.
- Seal policy grants no enumeration capability.
- `@` reifies name interpretation: `N@ is a name iff N is a name`.
  It cannot alter completed ordinary overload selection. SafetyPolicy is
  orthogonal to PolicyMode; lifecycle writes obey the unsafe admission owner.
