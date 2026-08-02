# Symbol Policy and Compile-Flow Projection

Status: canonical design contract. The typed model in this document is the
normative policy algebra. Flat `PolicySet`/`PolicyFlag` values are transitional
transport only and must not define source semantics.

This document owns the complete chain:

```text
source policy syntax
  -> contextual elaboration
  -> PolicyPair
  -> symbol resolution
  -> phase-slice exposure
  -> binding/overload selection
  -> OpenStatic evaluation
  -> SealStatic evaluation
  -> Runtime binding and evaluation
```

## 1. Complete symbol flow and policy pair

Language computation remains one object flow. Every object has the same three
components:

```text
Object x  = ⟨ Val1?(x), P(x), Val2(x) ⟩
Val1?(x) ∈ 1 + Object
```

The object ontology is owned by
`type-values-places-and-borrow-views.md`. This document owns what an *observer*
sees of that object.

Policy is not a component of the object. It belongs to the observation edge
between a context and an object:

```text
View_Γ(x) = ⟨ x, Pv:Pp, capability_Γ(x) ⟩
```

The same object observed from two contexts is one object with two views. The
policy of a result is always a pair:

```text
Π = Pv:Pp

Pv  policy of the value component observed at this edge
Pp  policy of the Pattern/anonymous-type component observed at this edge
```

There is no scalar replacement for this pair and no third policy slot. A
result object carries its own `PolicyPair` when it re-enters the flow.

The two axes of the pair are independent of the object's internal shape:

```text
Val1?(x) = null   does not imply  Pv = absent
Pv = absent       does not imply  Val1?(x) = null
```

The first says an object without an internal `Val1` payload may still be
observed through a value-bearing edge. The second says an observer may be denied
the value axis of an object that does carry a payload. Reading either direction
as an implication collapses the object into its observation.

Policy dimensions are typed and orthogonal:

```text
stage                       meta / compile / seal / runtime
value mutability            const / mut
value presence              present / optional / absent
ordinary namespace visibility public / private
export-root attribute       yes / no
```

They are not members of one untyped atom bag. In particular, export-root and
ordinary visibility are independent.

### 1.1 There is no central mutability propagation pass

Mutability is a coordinate of an observation edge, never a quantity pushed
through the object graph by a dedicated pass. The language defines no
`const`/`mut` propagation analysis, no transitive const inference over members,
and no whole-graph mutability closure.

The only two mechanisms that produce a propagation-like effect are:

```text
member overload      — a member's own candidates decide what an observer of
                       that member may do, per member, at lookup time
delete               — removing a candidate removes the corresponding
                       capability from every observer of that member
```

Both are local and per-member. An observer that reaches a nested member composes
the views it actually traverses; nothing recomputes an aggregate mutability for
the host.

## 2. Pattern alternative and policy operators

Single `|` belongs to Pattern alternative:

```lang
let bool = ((if | else) bool) |> struct;

let true = if::bool;
let false = else::bool;
```

Therefore:

```text
Pattern(bool) = if::bool | else::bool
true  holds the value read through if::bool
false holds the value read through else::bool
```

`true` and `false` are ordinary bindings, not aliases. Each is a fresh symbol
with a fresh place holding a copy of the value read through the source path:

```text
SymbolId(true) ≠ SymbolId(if::bool)
PlaceId(true)  ≠ PlaceId(if::bool)
Value(true)    =  Value(if::bool)
```

`true | false` is not a second Pattern space for `bool`.

Policy uses three different operators:

```text
||  choice within one policy dimension
+   conjunction of different orthogonal dimensions
:   value/Pattern pair separator
```

Precedence, from tightest to loosest, is:

```text
||  >  +  >  :
```

Thus:

```text
const + runtime || compile : compile
```

means:

```text
const + (runtime || compile) : compile
```

### 2.1 Policy grammar

```text
PolicySpec
  ::= PolicyConjunction
   |  PolicyConjunction ":" PolicyConjunction

PolicyConjunction
  ::= PolicyChoice
   |  PolicyConjunction "+" PolicyChoice

PolicyChoice
  ::= PolicyAtom
   |  PolicyChoice "||" PolicyAtom

PolicyAtom
  ::= Name
   |  "(" PolicyConjunction ")"
   |  AbsentValuePattern
```

The parser is a strong-context parser. `meta`, `compile`, `seal`, `runtime`,
`public`, `private`, and `export` remain ordinary names to the lexer. The token
spelling for `AbsentValuePattern` is provisional; implementation fixtures may
use `S`, but source spelling is not frozen.

### 2.2 Algebra

`||` selects alternatives within one dimension:

```text
runtime || compile
meta || compile
const || mut
runtime || S
```

It is not arbitrary clause-level Boolean disjunction. These are invalid:

```text
runtime || const
compile || public
mut || export
(const + runtime) || (mut + compile)
```

`+` combines different dimensions:

```text
const + runtime
mut + (runtime || compile)
public + compile
```

It cannot combine mutually exclusive values in one dimension:

```text
const + mut
public + private
```

The syntax and normalized AST retain `PolicyPair`, `PolicyConjunction`,
`PolicyChoice`, `PolicyAtom`, and `AbsentValuePattern`; `||` and `+` are never
lowered to the same set insertion operation.

## 3. Contextual elaboration of P1

Three policy contexts can occupy a binding-shaped surface slot, but use three
different elaborators.

### 3.1 Ordinary binding projection

```lang
[P1] let x = expr;
```

Omitted P1 retains the complete inferred RHS result. A single policy is a
value-dominant projection:

```lang
Q let x = expr;
```

It selects the value slice exposed by `Q`, then preserves the Pattern component
associated with that selected value slice. It does not mean `Q:Q`.

An explicit pair constrains both components:

```lang
Qv:Qp let x = expr;
```

Projection returns an identity-preserving restricted view. Given:

```text
Pv = compile || runtime
Pp = compile
```

the projection `runtime` produces:

```text
Pv = runtime
Pp = compile
```

It must not return the original `compile || runtime` entry. Symbol identity and
Pattern identity do not change; only the visible slice is cropped.

Atomic runtime Policy migration is a conservative extension of this query
rule. For the old projection judgment `ProjectP1` and the extended binding
elaborator `ElabP1`:

```text
ElabP1(None, R) = R

ElabP1(Some Q, R):
  S = ProjectP1(Q, R)
  if S != empty:
    return S
  otherwise, only if Q accepts a runtime value branch:
    Qr = RuntimeBranch(Q)
    prepare one direct atomic runtime migration toward Qr
```

Therefore:

```text
Dom_old = {
  (Q, R)
  |
  ProjectP1(Q, R) != empty
}

ProjectP1(Q, R) succeeds
  => ElabP1(Some Q, R) = ProjectP1(Q, R)

Dom_old is a subset of Dom(ElabP1)
```

The extension may add results only where the old projection was empty; it
cannot change an old successful result or selected identity. In the old
successful domain, migration candidate enumeration and invocation are
semantically unreachable. This applies to the complete
`PolicyResultEntry[]`, including collections that mix value-bearing and
absent-Val1 entries.

For pair query `Qv:Qp`, atomic migration preparation first slices only the
Pattern-policy stage capability:

```text
Pp_selected = SlicePatternPolicyStages(Qp, source.Pp)
```

This is Policy slicing over `Pp`; it is not Pattern extraction, PatternValue
projection, postfix `?`, extractor lookup, PatternHead navigation, or a change
of PatternRoot/PatternScope. It preserves PatternValue identity and structural
Pattern shape.

Unselected alternatives in a written query are never obligations to
manufacture every branch. However, after the complete query projects nothing,
an accepted branch that the language explicitly defines as constructible may
satisfy the choice. Runtime is currently the only such stage branch. The
bounded implementation subset is recorded in
`../../contracts/v0.6-cross-policy-value-transition.md`.

### 3.2 Formal parameter policy pattern

In a formal parameter:

```lang
const let x
mut let x
let x
```

the prefix is a formal policy pattern, not a binding slice query. Opposite
const/mut qualifiers remain in the fully admissible set and are compared only
by the overload product order in section 12.

Every written formal parameter first inherits the callable result pair `P2`
without reinterpretation:

```text
FormalBase(parameter) = P2(callable)
```

Omitting the prefix preserves that pair exactly. Writing `const` or `mut`
restricts only the value-mutability axis of the inherited pair:

```text
let x        -> FormalPattern(P2, mutability = unspecified)
const let x  -> FormalPattern(const + P2)
mut let x    -> FormalPattern(mut + P2)
```

Stages, value presence, and the Pattern component remain byte-for-byte the
inherited P2 dimensions; the qualifier may neither shrink nor widen them.
`public`, `private`, `export`, stage atoms, value absence, and an explicit pair
are therefore invalid formal prefixes. If P2 already explicitly restricts its
mutability domain, a contradictory formal qualifier is invalid rather than an
expansion.

The const/mut singleton above is a formal Pattern and preference input. It is
not an ordinary P1 query applied to the actual argument. Consequently an
oppositely qualified actual is not removed before the product order: for a
const actual the order remains `const > unspecified > mut`, and it reverses
for a mut actual.

The elaborated formal pair is not body-local policy metadata. Candidate
formation exports its written/inherited mutability Pattern into the callable's
external parameter-policy position:

```text
FormalPolicyPattern(parameter)
  -> Candidate.parameter_policy[position]
  -> MaxPolicyProduct
```

Thus the P2-derived restriction affects both the view available inside the
callable body and comparison of this callable against other fully admissible
overloads. “Inherit P2” must not be implemented by updating only the body
environment while leaving the candidate externally `unspecified`.

### 3.3 Namespace declaration attributes

`public`, `private`, and `export` are accepted only by namespace-declaration
elaboration. They are rejected in ordinary P1, formal parameters, return
slots, P2, Pattern interiors, expression policies, and local declarations that
are not namespace declaration positions.

`export` has the narrower placement rule described in section 9. Export
elaboration derives a separate external view; it does not crop the namespace's
complete internal declaration view. If the exported symbol has a value facet,
that external view must have a non-empty `const` projection. A pure
`absent:Pp` type/Pattern symbol has no value-mutability obligation.

Absence removes the complete value subspace of *this observation edge* rather
than merely selecting a presence tag:

```text
Pv = absent
  => value stages = ∅
  && value mutability = ∅
```

This is a statement about the edge, not about the object behind it. Per §1,
`Pv = absent` does not assert `Val1?(x) = null`, and an object with
`Val1?(x) = null` is not thereby forced to `Pv = absent`.

Consequently `const + S : compile`, `mut + S : compile`, and their `export`
forms are invalid. The current flat `ValueComponentPolicy` Rust carrier is
compatibility substrate rather than the final sum type, so P1 elaboration, P2
normalization, and resolved export projection each validate this invariant.

### 3.4 Policy Demand Satisfaction: existing first, constructible second

`PolicyDemand` may be retained as consumer-origin metadata:

```text
PolicyDemand
  = BindingP1Demand
  | ParameterPolicyDemand
  | ResultPolicyDemand
  | MechanicalPolicyDemand
```

This enumeration does **not** give all demand kinds a shared arbitrary
conversion search. Each demand kind owns its established
projection/admissibility rule. The general invariant orders two classes of
accepted view:

```text
SatisfyPolicyDemand(demand, result):
  Q = AcceptedPolicyQuery(demand)
  existing = ProjectExistingViewForDemand(Q, result)

  if existing != empty:
    return existing

  if runtime not in AcceptedValueStages(Q):
    fail

  Qr = RuntimeBranch(Q)
  consider one language-authorized atomic runtime migration toward Qr
```

For every demand kind:

```text
ProjectExistingViewForDemand(demand, R) != empty
  => no migration candidate enumeration
  => no migration invocation
  => no value reconstruction
  => Symbol / TypeValue / PatternValue / Place identity is unchanged
```

This is the **Existing-First, Constructible-Second** principle:

```text
1. existing accepted views
2. language-constructible accepted views
```

The current set of constructible stage branches is exactly `{ runtime }`.
Construction does not mean every alternative in `Q` becomes an obligation.
The original query may be `meta || runtime`; if its complete existing
projection is empty, the derived migration target is only its runtime branch.

`BindingP1Demand` uses the exact conservative `ProjectP1` theorem in §3.1.
Formal parameter and result consumers retain their existing policy-Pattern and
applicability rules. A demand that accepts `compile || runtime` is satisfied by
an available compile slice; the mere spelling of `runtime` as another accepted
alternative creates no materialization obligation.

`MechanicalPolicyDemand` records the origin of a language-selected mechanical
operation. It does not imply that arbitrary Policy failure may search `ref`,
`share`, `@`, or another structure-changing operation. Those operations
occur only when separately required by their own language rule and then use
ordinary function-object invocation.

### 3.5 Slicing and atomic runtime migration

Slicing and migration are sequential, not freely competing alternatives:

```text
source result
  -> Project_in: select an existing source Policy view
  -> Migration: one authorized directed runtime materialization
  -> ordinary result object
  -> Project_out: select the demanded output Policy view
```

Conceptually:

```text
Project_out o Migration o Project_in
```

`Project_in` and `Project_out` belong to existing Policy slicing algebra.
Migration is a directed operation, not a partial order. No transitive closure
or migration-chain search is formed. An operation implementation may call
other ordinary operations explicitly, but the demand satisfier prepares at
most one direct migration layer.

The existing P2 legality rule in §4 is the precondition:

```text
Static(Pv) = Pv - runtime

runtime not in Pp
Static(Pv) is empty or Static(Pv) = Pp
```

Therefore a legal value stage domain has at most one additional runtime branch
beyond its Pattern-policy stage domain, or is the runtime-only special case.

The compiler-mandated skeleton of atomic runtime migration is:

```text
input selected static view:
  Pv.stage = S
  Pp = S
  Type = T

output selected view:
  Pv.stage = runtime
  Pv.presence = present
  Pp = S
  Type = T
```

The compiler mandates only the selected-static-stage to runtime-stage edge.
Pattern-policy capability does not migrate to runtime and may not be
manufactured, and Type is unchanged. Other legal endpoint Policy coordinates
belong to the selected ordinary callable. In particular:

```text
Pv.input.mutability
Pv.output.mutability
```

need not be equal. A callable may declare `const + compile -> mut + runtime`
because it constructs a fresh runtime object; the compiler does not infer or
invent that `mut` capability. The declared input/output coordinates participate
in ordinary Bp' comparison. Opposite const/mut endpoint Patterns are not
removed by a hard Policy-domain intersection. They reuse ordinary
actual-relative preference:

```text
const actual/demand: const > unspecified > mut
mut actual/demand:   mut > unspecified > const
```

Stage, presence, Pp capability, Type, and structural applicability remain hard
endpoint conditions. Mutability is a preference coordinate, not a structural
repair and not a capability intersection.

`Pp` equality is about Policy capability; it is not an implementation license
to copy or reroot a source Pattern object. The eventual result Pattern comes
from ordinary invocation result semantics.

For a runtime demand:

```text
ProjectExistingView(complete query, source) is non-empty
  => consume that existing accepted slice

complete existing projection is empty
and runtime is accepted by the query
and Static(source.Pv) is non-empty
and the demanded Pp slice is available
  => extract RuntimeBranch(query)
  => select a pure-static Project_in endpoint
  => atomic runtime migration may be prepared
```

The complete source may already contain a runtime branch that is incompatible
with another requested coordinate. For example, a const
`compile || runtime` source does not satisfy `mut + runtime`; its const compile
view may still be selected as `Project_in` for a callable-declared
`const compile -> mut runtime` materialization. The invariant checked by the
migration request is that the **selected input endpoint** is static, not that
the complete result contains no runtime branch.

A failed Policy demand cannot repair failed Type/Pattern structural
applicability:

```text
not StructurallyApplicable(candidate, actual)
  => Policy migration alone cannot make candidate admissible
```

In particular, `T` is never changed implicitly to `T ref` merely because a
consumer requires runtime. Explicit/mechanical `ref` remains an independent
ordinary operation.

### 3.6 Existing runtime capability versus runtime value readability

For:

```text
Pv = compile || runtime
Pp = compile
```

a runtime query is an existing slice:

```text
ProjectPolicy(runtime, R) != empty
```

It is not a new compile-to-runtime invocation. The two explanations are:

```text
extensional availability:
  runtime is already a member of Pv

operational provenance:
  the language's atomic migration capability may explain
  how that branch can eventually be provided
```

Migration explains availability; slicing consumes availability. This preserves
the phase-layer separation:

```text
ResolveSymbol
ExposePolicySlice
ReadValue
ReadPattern
```

During a static phase, `ExposePolicySlice(runtime)` may establish that the
runtime branch exists in the semantic object while `ReadValue(runtime)` remains
unavailable until Runtime or is represented by residual computation. Runtime
Policy availability is not present-phase value readability.

### 3.7 Mixed-stage evaluation boundary

The core meaning of a mixed-stage result such as
`(compile || runtime):compile` is fixed:

```text
runtime in Pv
  => the runtime Policy slice already exists
  => ExposePolicySlice(runtime) does not invoke migration

compile-readable slice/dependency
  => expose, read, bind, and evaluate in the current static phase

runtime-dependent slot/computation
  => preserve the already-resolved identity
  => residualize until Runtime supplies the missing value
```

Therefore the frozen evaluation foundation is:

```text
Resolve once
Evaluate progressively
Residualize unavailable runtime dependencies
Continue the same already-resolved invocation at Runtime
```

Symbol/path/callable identity and ordinary overload selection occur in the
static semantic world. A runtime continuation does not reopen namespace
lookup, Symbol identity, callable identity, or the overload candidate set
merely because runtime values become readable. Explicit future dynamic
dispatch, if introduced, must be a different named mechanism.

Evaluation should compute the maximal phase-admissible portion subject to data
dependency and effect/sequencing constraints. Runtime-dependent portions are
residualized and later continue the same resolved computation.

What remains open is the implementation and effect boundary, not the existence
or basic binding meaning of the mixed-stage Policy domain:

- the exact residual object/IR representation;
- the physical representation of a mixed-stage `InvocationFrame`;
- the maximal-static-evaluation algorithm under data dependencies and effects;
- the exact sequencing frontier for effectful expressions;
- the continuation ABI and OpenStatic/SealStatic/Runtime handoff;
- composition with future capability/effect systems.

### 3.8 Static frontier and deferred materialization invariants

Static evaluation continues while the expression and its dependencies are
evaluable in the current static phase. Lexical occurrence inside a runtime
body is not by itself a runtime-computation boundary. The frontier is:

```text
static evaluation frontier
  = first dependency/effect boundary not admissible in the current phase
```

The following positive invariants constrain future integration even though
their storage/lowering algorithms are not implemented:

- Crossing a compile value to runtime constructs a new runtime object. It does
  not extend the lifetime of a compile temporary.
- Every addressable runtime value has an ordinary runtime owner/place. There is
  no third category of ownerless addressable temporary.
- A future static-materialization cache keys an ordinary compile value by its
  canonical static-value identity. A compile reference is keyed by compile
  referent identity, not by pointee value equality.
- Cache keying does not swallow the caller's construction context wholesale. A
  `compile` function that only injects through a `type ref` needs no ambient
  `Open` fact in its key: the parameter's canonical identity already carries the
  referent identity, and its applicability is proved by the parameter type. A
  `compile` function that injects a by-value `type` is the case where the same
  value can be legal or illegal depending on the call site:

  ```text
  Eval(F, t; Γ_open)  ≠  Eval(F, t; Γ_closed)
  ```

  Admissible treatments are: fold the required `Open` capability into the
  applicability judgment; cache the pure computation but not the call's
  legality; or record `requires Open(t)` in the function summary and verify it
  at the call site. Admitting the whole lexical context into the key
  indiscriminately is not required by this asymmetry.
- Storage requested by `[[global]]` materialization does not mutate the
  source-visible `NamespaceGraph`; generated storage and source namespace
  declarations remain distinct semantic facts.
- A language-selected `ref`, `share`, or `@` operation may compose
  ordinary operations and apply its own type/access rules. Such a structural
  operation is not Policy-demand repair.

These are deferred positive constraints, not claims that runtime lowering,
cache identity, `[[global]]` seal scanning, or lifetime checking is currently
implemented.

## 4. P2 normalization

P2 is the result pair of a call or expression:

```text
P2 = P2v:P2p
```

Explicit pairs include:

```text
runtime:compile
runtime:seal
(runtime || compile):compile
(runtime || seal):seal
const + (runtime || compile):compile
```

`runtime` is forbidden in P2p. If P2v contains a static stage, its static stage
set must equal P2p. Consequently these are invalid:

```text
runtime:runtime
compile:seal
meta:compile
```

For a single policy `P`:

```text
Pv = P
Pp = P - runtime
```

If that subtraction is empty, Pp is `compile`:

| Source P2 | Normalized pair |
|---|---|
| `meta` | `meta:meta` |
| `compile` | `compile:compile` |
| `seal` | `seal:seal` |
| `runtime` | `runtime:compile` |
| `runtime || compile` | `(runtime || compile):compile` |
| `runtime || seal` | `(runtime || seal):seal` |

`runtime:seal` remains a valid explicit pair; it means that the value is a
runtime value whose Pattern/type is first exposed during SealStatic.

P2 answers result-type and input-compatibility questions only. Three
authorities around an invocation result must stay separate:

```text
InvocationResultExposure   := canonical P1 of the producing declaration
ClusterMemberViewPolicy    := each member's own policy entry
ResultType / InputCompat   := P2
```

Whether an invocation result is outwardly visible at a phase is decided by
the canonical P1 authority, not by re-reading the callable's P2 pair as an
outward visibility source. There is no `P3` return policy, and P2 must not be
promoted into an ordinary-result outward authority.

## 5. Function-object P1 derivation

For `P2 = P2v:P2p`, lift only stages:

```text
Stage(P1p) = Stage(P2p)
Stage(P1v) = Stage(P2v) || Stage(P2p)
```

Examples:

```text
P2 runtime:compile -> P1stage (runtime || compile):compile
P2 runtime:seal    -> P1stage (runtime || seal):seal
P2 meta:meta       -> P1stage meta:meta
```

The following never propagate from P2 to the function object:

```text
const / mut
public / private
export-root
value presence
```

Those properties come only from the function object's declaration.

For a declaration such as:

```lang
let fn = () => { ... };
```

the declaration supplies an empty value-mutability restriction. In the typed
policy domain, empty here means the complete `const || mut` domain, not “no
value” and not an unknown third qualifier. A written declaration P1 may crop
that domain to `const` or `mut`. P2 mutability never propagates into the
function object during stage lifting. Export is not an exception to this
internal default. The function object's namespace-internal declaration view
remains the written/unwritten full domain; only its separately derived external
value view is const-projected.

## 6. Three execution phases

```text
Phase = OpenStatic | SealStatic | Runtime
```

Stage visibility is defined by domains:

```text
Vis(meta)    = { OpenStatic }
Vis(seal)    = { SealStatic }
Vis(compile) = { OpenStatic, SealStatic }
Vis(runtime) = { Runtime }
```

| Policy stage | OpenStatic | SealStatic | Runtime |
|---|:---:|:---:|:---:|
| `meta` | yes | no | no |
| `compile` | yes | yes | no |
| `seal` | no | yes | no |
| `runtime` value | no | no | yes |

`compile` being visible during SealStatic does not make `compile` equal to
`seal`. Exposure checks ask whether the current phase is in `Vis(stage)`; they
do not intersect atom spellings.

## 7. Resolution, exposure, and facet reads

Every phase distinguishes:

```text
ResolveSymbol(path)
ExposePolicySlice(symbol, phase)
ReadValue(slice)
ReadPattern(slice)
EnumerateValueFacet(slice)
EnterCallableBody(candidate)
```

Failure to expose a value slice is not an unresolved symbol. In particular:

```text
Pv = runtime
Pp = compile
```

has this OpenStatic behavior:

```text
symbol/path resolves
runtime value is unreadable
compile Pattern/type is readable
derived compile companion may join static overload resolution
original runtime computation remains in RuntimeResidualFlow
```

Conversely, exposing or selecting an existing runtime Policy slice in a static
phase is not permission to read its value:

```text
runtime in Pv
  => the runtime capability/view exists

current Phase is OpenStatic or SealStatic
  => ReadValue(runtime slice) is unavailable
  => preserve already-resolved runtime computation/residual
```

The runtime continuation consumes the preserved symbol/callable identities. It
does not repeat path resolution or overload choice merely because the value
becomes readable later.

Seal-only symbols follow the same ordinary-symbol rule. Their paths can be
resolved independently of whether a facet is exposed in the current phase.

## 8. Mechanical compile-flow projection

Before any of the three phases executes, structurally project:

```text
CompleteSymbolFlow
  -> StaticFlow
  -> RuntimeResidualFlow
```

Projection does not execute calls or perform final overload selection.

`StaticFlow` preserves:

```text
Pattern/type flow
meta/compile/seal static call nodes
derived compile companions
symbol relationships required by static bindings
DeferredSealTask nodes
D/Done and control-flow structure
```

`RuntimeResidualFlow` preserves:

```text
runtime value computations and bodies
runtime branch value selection
runtime effects
runtime symbol binding
required D/Done and control-flow structure
```

No phase is inferred ad hoc from the original AST after this projection.

## 9. Namespace visibility and export

### 9.1 Three independent symbol views

Namespace resolution, external exposure, and compilation-world membership are
different questions:

```text
Σ_full(N)    complete namespace-internal symbol/overload set
Σ_export(N)  externally exposed projection of that set
Wfinal       Wpre ∪ Wseal, the symbols materialized or retained this build
```

They are consumed by distinct operations:

```text
InternalResolve(N, path) searches Σ_full(N)
ExternalResolve(N, path) searches Σ_export(N)
WorldMembership(s) asks whether s belongs to Wpre or Wseal
```

For one name:

```text
ExportOverloadSet(name)
  = ExternalProjection(FullOverloadSet(name))
```

This projection retains the original candidate identities; it does not create
a second symbol universe. Consequently:

```text
s in Wpre  does not imply s is exported
s in Wseal does not imply s is exported
s is exported does not imply s was itself an export root
```

Explicit navigation is authority-sensitive. Internal explicit navigation may
reach the complete namespace-internal view. External explicit navigation is
restricted to the export projection. Explicit-path success alone therefore
does not prove export membership.

### 9.2 Export roots and value projection

`export` is allowed only on a direct top-level declaration of one namespace
construction level:

```lang
export let name = expr;
```

Let `InternalView(s) = Pv:Pp`. Export derives, rather than replaces, a second
view:

```text
if Pv = absent:
  require value stages = ∅
  require value mutability = ∅
  ExternalView(s) = absent:Pp

if Pv is present/optional:
  require Project_const(Pv) is non-empty
  ExternalView(s) = Project_const(Pv):Pp
```

Thus an omitted mutability axis and `const || mut` are valid complete internal
domains because both have a const projection. A `mut`-only value export is
invalid. `export requires const` is only shorthand for this value-facet
projection rule; it is not a claim about pure type/Pattern exports.

It is forbidden in function/meta-function bodies, parameters, return slots,
P2, Pattern interiors, expression policies, ordinary local P1, and any nested
local declaration below that namespace level. A top-level function object may
be an export root; its body declarations may not.

For export root `s`:

```text
ExportRetentionClosure(s) = PathAncestors(s) ∪ Subtree(s)
```

All ancestors needed to reach the root and its entire subtree enter the export
graph. A child cannot close export again; an unrelated sibling is unaffected.

The declaration spelling and the resolved candidate view are different
layers:

```text
declaration_projection: P1Projection

RHS/result entries
  -> ApplyDeclarationProjection
  -> ResolvedCandidatePolicy { pair: PolicyPair, provenance }
```

Only the resolved pair can be projected into the external interface.
`P1Projection::Infer` is a valid declaration request, and
`P1Projection::ValueDominant` does not yet carry the associated `Pp`; neither
is an external candidate policy.

The typed substrate therefore represents export as an identity-preserving
candidate transformation:

```text
ExportCandidateView {
  identity,
  internal_candidate,
  external_policy: PolicyPair
}

ExportAdmission {
  in_export_retention_closure,
  publicly_reachable
}

if admission.in_export_retention_closure && admission.publicly_reachable:
  internal_policy := ResolveCandidatePolicy(candidate)
  external_policy := Project_const(internal_policy.Pv):internal_policy.Pp
```

Export-retention-closure membership and public path reachability are separate
symbol/name-level facts; both are required before a symbol contributes to
`Σ_export`. In particular, a private child in an exported subtree and every
descendant reached through that private path remain absent externally even
when those symbols belong to `ExportRetentionClosure`.

The retention name is deliberate: membership means that an export root keeps
the symbol in the graph considered for interface construction. It does not by
itself mean that the symbol is externally exported. `Σ_export` is the external
candidate set.

Admission does not arbitrarily select individual overloads. Within an admitted
symbol's complete overload set, every candidate whose resolved pair has a
const value projection enters `Σ_export`; a mut-only candidate remains in
`Σ_full` but has no external candidate view. A pure `absent:Pp` candidate
enters unchanged.

A direct source declaration that explicitly writes `export + mut` is still
invalid at declaration elaboration. This direct-root error is distinct from
filtering a mut-only member of an otherwise exported full overload set.

Ancestors and descendants admitted by the final external-exposure check need
not be export roots and may have used `P1Projection::Infer`; their resolved
candidate pairs are projected in exactly the same way.
`NamespaceDeclarationPolicy.external_projection` is only an early
direct-root validation/preview; `None` on a non-root declaration does not mean
that the eventual namespace export view lacks that declaration.

The current typed set carrier omits a name when its external candidate subset
is empty. That is sufficient to define `Σ_export`, but not to diagnose why no
external candidate exists. Before end-to-end external resolver integration, a
symbol-level diagnostic carrier must preserve admission facts and distinguish
an unresolved name, a name outside the export-retention closure, a private
path, and an admitted symbol with no const-projectable candidate.

### 9.3 Public/private

`public` and `private` are ordinary hierarchical visibility attributes. A
public parent may contain a private child, and a private parent may contain a
public child. External path access checks every segment, so a private parent
blocks external reachability to a public child.

```text
ExternallyVisible(path)
  = Exported(path) && PubliclyReachable(path)
```

The export-retention closure may retain private dependencies without
installing them in `Σ_export`.

## 10. Wpre and seal world

Immediately before SealStatic, compute the least semantic materialization
closure:

```text
R0 = ExportedSymbols
   ∪ MaterializedResultsOfExportedMetaFunctions
   ∪ ParameterDependenciesOfExportedMetaFunctions

R(n+1) = Rn ∪ SemanticDependencies(Rn)

Wpre = least_fixed_point(R)
```

Materialized results include only results actually generated in this build,
not the infinite set a generic meta function might produce for future inputs.
Wpre can contain non-exported private dependencies solely so the exported
interface remains interpretable. Such membership does not install those
dependencies in `Σ_export`.

SealStatic generates `Wseal` and finishes with:

```text
Wfinal = Wpre ∪ Wseal
```

Only a compiler-known privileged seal function may enumerate the symbol world,
and its fixed scan domain is `Wpre`. Adding `Wseal` never expands that domain.
Ordinary seal policy grants no scanning capability.

Explicit lookup is separate:

```text
ResolveExplicitPath != EnumerateSymbolWorld
```

A committed symbol in Wseal can be explicitly resolved by later seal/compile
code under ordinary construction transaction, name-resolution, dependency,
authority, and policy rules. Internal authority may resolve it through
`Σ_full`; external authority still requires a corresponding `Σ_export` view.
Its absence from the current Wpre scan does not make it unaddressable, and its
presence in Wseal does not make it exported.

## 11. Phase execution

### 11.1 OpenStatic

Exposed stages are `meta` and `compile`; seal and runtime value slices are not
exposed. A call may evaluate when its callable exposes a meta/compile view, all
arguments supply the required static views, and the associated `()` candidate
is fully admissible.

Static views include meta values, compile values, compile Pattern/type
projections of runtime symbols, and derived compile companions. Meta and compile
callables may invoke one another in one evaluator. Their return ontologies differ
in authority, not in value rank:

```text
meta    -> may establish a global symbol root and seal a MetaInstance
compile -> ordinary PatternValue only; establishes no global root
```

Both may return any ordinary `PatternValue`, including a type value, a symbol
value, or a `type ref`. `compile` is restricted by what it may *root*, not by
which value shapes it may produce; the root condition is owned by
`symbol-first-meta-construction-and-pattern-injection.md`.

No OpenStatic task may read a runtime value or depend on a runtime effect. If a
task is blocked only by a seal-only view, preserve its call node, Pattern
arguments, symbol dependencies, and overload inputs as a `DeferredSealTask`.

When otherwise equal and fully admissible, phase specificity uses the narrower
visible domain:

```text
Vis(meta) ⊂ Vis(compile), therefore meta > compile in OpenStatic
```

This is one dimension of the complete partial order, not an unconditional
global priority.

### 11.2 SealStatic

Exposed stages are `seal` and `compile`; meta and runtime value slices are not.
The same static evaluator and symbol-construction machinery consumes deferred
tasks, explicit seal/compile callables, privileged seal calls, fixed Wpre scan
results, and ordinary explicitly resolved symbols.

When otherwise equal and fully admissible:

```text
Vis(seal) ⊂ Vis(compile), therefore seal > compile in SealStatic
```

SealStatic is terminal for static work. Missing symbols/projections/companions,
runtime value/effect dependencies, or non-unique overload maxima are errors;
there is no later static deferral phase.

### 11.3 Runtime

Runtime consumes `RuntimeResidualFlow`, exposes runtime value slices, completes
runtime symbol binding and overload selection, executes runtime bodies/effects,
and performs runtime branch value selection. A derived compile companion never
replaces the real runtime call.

## 12. Unified binding and overload selection

All ordinary bindings and call targets use one selection trunk:

```text
C0 = EnumerateValueEntries(ResolveSymbol(path))
C1 = ExposePhaseViews(C0, Phase)
C2 = ProjectExpectedPolicy(C1, P1_or_expected_facet)
A  = FullyAdmissible(C2, argument_frame, expected_result)
M  = MaxPolicyAndOverloadOrder(A)
```

Success requires exactly one maximal candidate. Failure can mean no exposed
slice, no fully admissible entry, multiple incomparable maxima, a unique delete
maximum, or an unfinished terminal SealStatic task.

For each const/mut comparison position:

```text
const actual: const > unspecified > mut
mut actual:   mut > unspecified > const
```

This order is a *preference* among candidates that are already fully admissible.
Being higher in the order never grants a capability, and being lower never
removes one: the order chooses between existing candidates and does not decide
whether a candidate exists. Nor does it propagate: the selected candidate's
mutability qualifier describes that one observation edge and is not pushed into
the argument's other members (§1.1).

Multiple positions form a product partial order: `f` dominates `g` iff `f` is
not worse at every participating position and is strictly better at at least
one. Crossed advantages remain incomparable. There is no score, exact-match
count, parameter weighting, lexicographic order, input-before-output rule, or
separate conversion rank. A result policy participates only when the call
context supplies a target-result constraint.

For the one compiler-inserted atomic runtime-migration call, its selected input
and required output Policy endpoints add two coordinates to this same Bp
product:

```text
Bp' =
  ordinary Bp coordinates
  x migration input endpoint fit
  x migration output endpoint fit
```

They are not a B6 named strategy. They therefore precede B3 Pattern extraction
specificity. When no atomic-migration endpoint context is present,
`Bp' = Bp` exactly, so every old survivor and all later B1..B6 behavior are
unchanged.

Delete members enter the same fully admissible set and order. A unique maximal
delete produces a diagnostic naming that member.

Current source cannot construct a fallback candidate role, so the ordinary
pipeline above remains exactly `A -> Bp'` and `Af = A`. If a future fallback
strategy is introduced, its already-fixed semantics inserts
`SuppressFallback(A)` before Bp': any admissible non-fallback member, including
`delete`, permanently removes fallback. This future behavior is not B6
named-strategy execution, and later delete/lowering/lifetime failure cannot
reopen fallback.

## 13. Lifetime boundary

`@` is an ordinary place-sensitive operation with its own overload groups, owned
by `../lifetime/lifetime-policy-and-overload-boundary.md`. It is not a policy
atom in the stage dimension of §1, and lifetime policy is not a fifth stage.

The only boundary this document asserts is directional: ordinary overload
selection must already have produced one unique candidate, and lifetime rules
validate that result without replacing it. Lifetime checking may reject a
program; it may not reselect a call, reopen type/policy overload resolution, or
introduce a competing specificity order.

This is a restriction on lifetime *rules*, not a denial that `@` has overloads.
`@` is resolved by the ordinary selection trunk of §12 like any other operation.

## 14. Transitional implementation boundary

The typed implementation model contains dedicated policy AST nodes,
`PolicyPair`, typed dimensions, three distinct P1 elaborators, true slice
restriction, three `Phase` values, phase exposure, mechanical flow projection,
Wpre closure, export-retention closure, and phase-aware partial-order
selection.

The atomic-migration prototype compares input and output endpoint Policy
through one product/Pareto order. These are only the endpoint coordinates of
the future Bp' product. The private endpoint-only maxima helper is not a
sequentially composable Bp filter:

```text
Max(Product(old Bp, input endpoint, output endpoint))
  != MaxEndpoint(MaxOldBp(...))
  != MaxOldBp(MaxEndpoint(...))
```

Final integration must compose ordinary Bp coordinates and both migration
endpoint coordinates in one comparator before taking maxima. The prototype
does not add output-type preference to ordinary type overload selection or
define a B6 strategy.
Candidate Policy adaptation intersects typed Policy domains directly,
including stage, Pp, and present/optional/absent alternatives; it does not
fabricate a concrete `Some(value)` to reuse result-entry projection.
Migration-candidate mutability is deliberately excluded from that hard
intersection and instead reuses ordinary actual-relative Bp preference. The
bounded prototype treats a singleton selected/requested mutability as the
actual comparison point; a non-singleton endpoint remains neutral until the
final ordinary Bp carrier is integrated.
The current binding adapter is reached only after the complete ordinary
projection is empty and the original query accepts runtime. It extracts a
runtime-only target branch, skips absent entries, selects a pure-static source
and Pattern-policy stage capability, and carries fixture result Pattern data.
That fixture carrier does not establish final ordinary-result Type/Pattern/
owner coherence. It shares the maximal-element helper but does not yet perform
initializer integration, Symbol/Val2/associated-`()` lookup, `InvocationFrame`
construction, or ordinary function-object invocation.

`PolicySet` and `PolicyFlag` remain in older resolver/build paths as a lossy
transport. They cannot represent `||` structure, Pattern association of a
cropped slice, or independent export-root and public/private dimensions. New
semantics must be implemented in the typed model first; compatibility flags may
only receive a projection from it. Full namespace-graph storage and end-to-end
evaluator integration remain implementation work and must not be inferred from
flat flags.

## 15. Deliberately unfrozen

This document does not freeze:

- the final source token for `AbsentValuePattern`;
- full lifetime/Horae semantics;
- future policy stages;
- arbitrary clause-level Boolean policy logic;
- a complete runtime reflection API;
- export reopening syntax;
- cross-file open overload union;
- unrelated `?`, `inject`, or new PatternValue mechanisms.
