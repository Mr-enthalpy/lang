# Policy Visibility and Capability Mapping

Status: implementation-mapping companion. Canonical semantics are owned by
[`../symbol-world/policy-and-static-flow-projection.md`](../symbol-world/policy-and-static-flow-projection.md).

## 1. Typed semantic model

```text
PolicyPair = Pv:Pp
PolicyMode = const | mut
```

Pv and Pp are internal observations of one source edge, not a public colon
constructor/extractor. Direct Policy observation obtains Pv; Policy observation
composed with the same source's type projection obtains Pp. Binding that type
to a new name introduces a destination view, so later observation of that name
does not automatically recover the source Pp.

Whole-slot Mode, Safety, visibility/export and capability remain orthogonal.
Concrete atoms introduce no holes: `runtime let` corresponds to
`<> runtime let`, not `<runtime> runtime let`. An explicit hole and omitted
constraint remain distinct. `<> p$ let` reuses an evaluated Policy value via
the general polarity interpretation interface; it does not rebind its HoleIds or reparse strings.

Public Policy constrains linked adjacent-rank observations through ordinary extraction.
Legal orthogonal +, ordinary extraction/holes/polarity and require
constrain both observations within one candidate-local relation. Raw/Norm
constraint carriers preserve ordinary atoms and orthogonal conjunction (§5).
Default completion applies separately after inherited/contextual constraints.

Policy positions have contextual elaborators:

```text
ordinary binding P1          -> identity-preserving slice restriction
formal parameter policy      -> Pin = ElabIn(HorizonPolicy(P2), Delta_in)
return-position policy       -> Pout = ElabOut(P1, Delta_out)
namespace declaration policy -> visibility plus optional export-root
```

A concrete accepted existing view satisfies a demand without reconstruction.
No unresolved solver alternative manufactures a missing view.

## 2. P2 and function objects

Stage={compile,runtime}; the ordinary stage order has identity and compile-to-runtime.
A selected runtime-to-compile migration requires a closed active reify P2 boundary.
P2 is horizon, P1/Pout producer visibility. InputAdmissible, Ready and execution
legality remain independent. The internal runtime:compile view is valid.
P2 runtime with explicit compile P1 and mixed output Hole Policies is legal.
The need for reify follows actual reverse-dependent work. Compile P2 completes
omitted ordinary P1 to compile. Pin admits explicit
constraints and Holes; Pout retains P1.stage. HoleOverride independently replaces
written Hole Policy dimensions and inherits the others without strength checks.
Pv/Pp are linked adjacent ranks, not independent stage dimensions. Ordinary
runtime let constrains only the first non-absent observation, preserving its
classifier. Compile:runtime is incoherent. RR -> RC restores the compile
classifier of the same runtime Object under the admitted reify continuation.

## 3. Observation horizon mapping

```text
ObservationHorizon=OpenStatic | Runtime
```

| Stage | OpenStatic | Runtime |
|---|:---:|:---:|
| compile | yes | no |
| runtime | no | yes |

Resolution fixes identity before exposure. Hidden runtime Val1 retains its
Pattern and Object coordinates. R_vis/C_sigma require ordinary evidence;
visibility grants neither Ready nor execution. Selected origins never reopen.
Ordinary internal/external lookup consumes source access and export facts.
Callable-head reify materializes only the selected reverse-dependent E cone
under an independently closed return classifier;
it supplies no namespace scanning privilege.

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

The absent value form has no value-stage coordinate, but it does not erase the
orthogonal whole-slot mode:

```text
Pv = absent
  => no value-stage coordinate in this observation
  && SemanticValueId = none

PolicyMode(absent:Pp slot) ∈ {const, mut}
```

`const` and `mut` therefore all remain meaningful for a pure
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
Committed binding existence remains independent of these views.

## 5. Rust substrate

The following carriers are implementation inventory, not evidence that the
full R_vis/C_sigma model is connected. Resolved observations carry one Stage
atom, while omitted query coordinates remain separate. The mapping below is
not normative algebra. Omission remains separate until contextual completion:
OpenPolicy=open with known current OpenHere yields mut, otherwise const. Missing required
OpenHere evidence is unavailable. Operator-Pattern policy deduction and the joint
Pin/Pout solution relation remain pending consumers.

The typed substrate currently provides:

- `PolicySpecAst { constraint }`, `PolicyConjunctionAst { atoms }`, and `PolicyAtomAst`;
- `PolicyPair` with single `Stage` coordinates and projection-only `ObservationHorizon`;
- horizon-free path/name resolution establishes a fixed binding before exposure;
  subsequent callee observation, candidate preparation, body-entry observation
  and invocation frames share one horizon coordinate, each retaining its own
  observed Policy and judgment;
- separate binding/formal/namespace elaborators;
- formal elaboration that receives inherited P2 explicitly and preserves all
  non-mode dimensions;
- P2 normalization and single-atom omitted-P1 completion;
- owned matching P1 observations that preserve identity without stage cropping;
- explicit resolution followed by horizon exposure and facet reads; hidden
  facets retain their underlying resolved observations; no static/runtime node
  partition or visibility-based readiness classifier exists;
- export-retention least-closure helpers;
- complete and externally projected namespace overload-set carriers that
  require a typed `ExportAdmission { in_export_retention_closure,
  publicly_reachable }` before projection and
  preserve candidate identity while storing a distinct resolved `PolicyPair`
  on each `ExportCandidateView`;
- horizon visibility filtering and ordinary Policy-mode preference, with
  no ranking between distinct static stage atoms;
- callable visibility reads the candidate's resolved Policy, without a
  stage-specific function-member taxonomy or a separately staged world;
- incomplete initializer evaluation establishes no result stage. Until the
  common continuation preservation consumer is connected, binding reports an
  explicit unavailable diagnostic and installs no name or result Policy, even
  with an explicit runtime demand. Exact empty call spaces and fully observed
  non-callability or applicability/Policy mismatch remain ordinary failures.
  Invocation records actual hidden host/callee/body observations directly;
  an initializer never infers horizon obstruction from a generic failure or
  an absent diagnostic. Exposed candidates still undergo ordinary selection.
  Reached candidate errors
  and selected-body failures remain diagnostics. Horizon visibility supplies
  no delete-body execution-legality proof;
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
- Pattern | retains its own relation; no dedicated public Policy union is added.
- Runtime observation retains the actual runtime:compile or runtime:runtime
  coordinates; the horizon supplies neither missing observation.
- Internal (Pv=runtime,Pp=compile) observations remain valid; public colon pairs do not.
- P1 projection accepts existing matching observations without cropping stage atoms.
- A non-empty ordinary P1 projection never manufactures absent query
  alternatives and makes migration unreachable.
- After existing projection is empty, a concrete runtime demand may admit one
  direct same-Type migration; solver alternatives are not manufactured views.
- Policy slicing of `Pp` does not extract, navigate, reroot, or otherwise
  transform the Object's P coordinate.
- Direct migration preserves Type and declared endpoint coherence. A reverse
  value or classifier coordinate requires the active closed reify P2 boundary;
  selected PolicyMode endpoints may differ and participate in Bp'.
- Policy failure cannot repair Type/Pattern structural inapplicability.
- Runtime value invisibility never deletes the name binding or erases its
  resident's independently exposed Pattern view.
- Runtime Policy-slice existence does not imply present-phase value
  readability.
- CompileInstance formation retains the selected ordinary invocation identity.
- Namespace enumeration requires its ordinary callable capability.
- `@` reifies name interpretation: `N@ is a name iff N is a name`.
  It cannot alter completed ordinary overload selection. SafetyPolicy is
  orthogonal to PolicyMode; lifecycle writes obey the unsafe admission owner.
