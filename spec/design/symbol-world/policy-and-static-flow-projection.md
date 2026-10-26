# Policy and Static Flow Projection

Status: canonical design contract. The typed model in this document is the
normative policy algebra.

This document owns the complete chain:

```text
source policy syntax
  -> contextual elaboration
  -> internal value/type observation facts
  -> binding resolution
  -> visibility/input evidence R_vis
  -> ordinary compile realization C_sigma where required
  -> hard admissibility / ordinary overload selection
  -> OpenStatic evaluation
  -> Runtime binding and evaluation
```

## 1. Complete binding flow and policy pair

P1 supplies OpenPolicy={open,close}, independently of value-mode
PolicyMode={const,mut} and Stage={compile,runtime}. Only finally omitted
mode completes contextually: open plus current known OpenHere gives mut;
otherwise const. Required unknown OpenHere is unavailable. P2 supplies the
evaluation horizon; stable CompileInstance identity is independent of P2.

Language computation remains one object flow. Every object has the same three
components:

```text
Object x  = ⟨ Val1?(x), P(x), Val2(x) ⟩
Val1?(x) ∈ 1 + Object
```

The object ontology is owned by
`type-values-places-and-borrow-views.md`. This document owns what an *observer*
sees of that object.

Policy is not a component of the object. It belongs to a complete slot/view
edge between a context and an object:

```text
PolicyView_Γ(slot, x)
  = ⟨ x, Pv:Pp, PolicyMode_Γ(slot), SafetyPolicy_Γ(slot) ⟩
```

SafetyPolicy = safe | unsafe is independent of PolicyMode and the pair.
Its semantic-admission boundary is defined in
[unsafe admission](../lifetime/unsafe-semantic-admission.md); it grants neither
write authority nor a substitute for action Pre. The mode comparisons below
remain their existing two-point relations.

Policy preference, capability realization, and dynamic legality must not be
collapsed:

```text
CapabilityRealization(candidate, family, input_mode, output_mode)
  ∈ { absent, default, delete, custom }
  // stable declaration/intrinsic fact of the candidate and associated family

DynamicLegality_Γ(
  selected_invocation,
  place_state,
  lifetime_state,
  authority_state,
  ...)
  // consumer-context legality of the already selected invocation
```

`CapabilityRealization` is stable candidate/family metadata. It records how a
derived two-mode cell is realized and may therefore be retained with a candidate snapshot.
`DynamicLegality_Γ` is formed only for the selected invocation in the current
consumer context. Place writability, lifetime validity, construction authority,
access, escape, and `OpenHere` are premises of that legality judgment rather
than a second capability-realization layer. It is never frozen into a namespace
export snapshot:

```text
DynamicLegality_Γ(inv)
  iff RequiredCapabilityExists(inv)
  and (RequiresWrite(inv) => Writable_Γ(Target(inv)))
  and LifetimeLegal_Γ(inv)
  and AuthorityLegal_Γ(inv)
  and (OpenSensitive(inv) => OpenHere_Σ(OldTarget(inv)))
  and EscapeLegal_Γ(inv)
```

Failure of `DynamicLegality_Γ` rejects that selected invocation and never
reopens ordinary candidate lookup or Policy maxima.

The same object observed from two contexts is one object with two views. The
Policy of a result is a rank-indexed observation chain. Its common two-rank
window is:

```text
Pi_e : rank -> {absent,runtime,compile}
Legal(Pi_e) iff Pi_e belongs to (absent)*(runtime)*(compile)*
Pair_r(Pi_e) = <Pi_e(r),Pi_e(r+1)> = Pv:Pp

Pv  policy of the value component observed at this edge
Pp  policy of the Pattern/anonymous-type component observed at this edge
```

The two-rank window is convenient in common encoding, not a bound on semantic
rank depth. Every finite observed prefix must obey the displayed ordered
blocks: absence cannot follow a present observation, and runtime cannot follow
compile. A window may be absent:absent; such a window alone does not determine
the first present rank or the unobserved tail. Parameter and return positions do
nevertheless have position Policies: `P_in` overlays P2 and `P_out` overlays
P1. Pin admits explicit stage constraints and ordinary stage holes; Pout inherits
P1's stage. Both positions may refine their orthogonal whole-slot PolicyMode.
A result retains its actual rank observations when it re-enters the flow;
its pair window does not discharge unseen higher-rank obligations.

The pair observes two adjacent ranks of one classification chain. Let
Pi_e(i) be the Policy observation at rank i on the same source edge e:

```text
Pv = Pi_e(r)
Pp = Pi_e(r+1) = Policy_e(Type(x))
Pi_e(i)=runtime => Pi_e(i+1) in {runtime,compile}
Pi_e(i)=compile => Pi_e(i+1)=compile
Pi_e(i)=absent => Pi_e(i+1) in {absent,runtime,compile}
legal two-rank prefixes = { absent:absent, absent:runtime, absent:compile,
                           runtime:runtime, runtime:compile, compile:compile }
illegal = { runtime:absent, compile:absent, compile:runtime }

Pv = absent  does not imply  Val1?(x) = null

PolicyMode_Γ(slot) ∈ { const, mut }
PolicyMode_Γ(slot) is independent of Val1?(x), Pv, and Pp
```

These are linked rank observations, not independent Policy dimensions.
Val1 absence is an Object-shape fact; it neither hides an observation nor
equates the Policies of the Object and its classifier. Type values use this
same adjacent-rank rule. `absent:Pp` skips the unavailable/hidden value
observation without manufacturing a value or erasing the classifier rank.

The whole-slot separation is a semantic invariant:

```text
PolicyModeOrthogonalToObjectShape:

Val1?(x) = null
  =/> PolicyMode_Γ(slot) = const
  =/> PolicyMode_Γ(slot) = mut
```

Whole-slot PolicyMode is independent of this rank-chain coherence relation.
The same complete type value may occupy const and mut slots without changing
its Object identity or adding an Object component.

Policy dimensions are typed and orthogonal:

```text
pair/view stage              compile / runtime
pair/view presence           present / optional / absent
whole-slot PolicyMode        const / mut
ordinary namespace visibility public / private
export-root attribute        yes / no
```

They are not members of one untyped atom bag. In particular, export-root and
ordinary visibility are independent.

### 1.1 There is no central PolicyMode propagation pass

`PolicyMode` is a whole-slot coordinate of an observation edge, never a quantity pushed
through the object graph by a dedicated pass. The language defines no
`const`/`mut` propagation analysis, no transitive const inference over members,
and no whole-graph PolicyMode closure.

The only two mechanisms that produce a propagation-like effect are:

```text
member overload      — a member's own candidates decide what an observer of
                       that member may do, per member, at lookup time
delete               — removing a candidate removes the corresponding
                       capability from every observer of that member
```

Both are local and per-member. An observer that reaches a nested member composes
the views it actually traverses; nothing recomputes an aggregate PolicyMode for
the host.

Policy is a selection relation, never a capability grant. The accompanying
theorem is:

```text
PolicyDoesNotGrantCapability:

PolicyMode(actual_slot) = mut
  !=>
Writable(actual)

PolicyMode(actual_slot) = const
  !=>
not Writable(underlying place)

Writable(place)
  !=>
PolicyMode(view_slot) = mut
```

`const let` / `mut let` on a formal parameter are first an overload
preference coordinate (the `succ_const` / `succ_mut` partial
orders of §3.2). A `mut` candidate being preferred and the selected operation
actually exposing a write are two different facts. Real write capability comes
from the conjunction:

```text
selected associated operation
+ borrow capability
+ Writable(place)
+ lifetime validity
```

never from the `mut let` spelling itself. This is why the `ref` family, the
`share` family, and the `=` family compose to the language's actual behavior
without policy carrying a writable/nonwritable promise (§1.1's two mechanisms
`member overload` + `delete` remain the only local capability-exposure
mechanisms, and the candidate schemas of `=` / field / `ref` / `share` in
`structural construction owner and `type-values` §5.1.3 reference exactly this rule
rather than redefining a second PolicyMode system).

### 1.1.1 Initializing a typed Place before a resident exists

    DeclaredPolicy(name) independent_of InitialInitializationAuthority(place)

A typed Uninitialized Place has a declared type and pending initialization
authority from its authorized formation, but no resident Object to observe.
PolicyView(slot,x), P(x), and resident compatibility are not evaluated for a
missing x. The declaration's const/mut mode determines the initialized
name's views; it neither creates nor removes the separate first-write authority.

The ordinary explicit initial-borrow realization uses that live authority and
actual Place/access/lifetime checks to supply a reference capability for the
first write. It can supply a mut T ref view for a const-declared name; the
reference view's mode and the target name's declared mode are distinct. The
existing const reference delete cells remain delete. Selecting a mut
reference default still requires the applicable capability and current legality.

First write uses InitWriteLegal; replacement uses the initialized resident and
its ordinary compatibility rules. Initialization authority is consumed at the
successful commit for the actual Place. Saved references and cached capability
metadata do not preserve that authority or turn it into replacement power.
See [Place/write algebra](type-values-places-and-borrow-views.md#711-initialization-authority-and-the-two-write-cases)
for the complete branches and const initialization example.

This uses the existing operation/capability/legality separation. It adds no
PolicyMode, implicit ref, general const-to-mut conversion, or no-reopen exception.

## 1.2 Explicit `const` / `mut` are value reconstruction, not in-place policy casts

Global `const` / `mut` are not a way to change the policy tag on the current
place. They are explicit policy reconstruction:

```text
ExplicitPolicyReconstruction
```

For example `val const` logically:

```text
1. derive TypeOf(val) = T
2. invoke T's own construction/call family with the requested const-result
   policy
3. obtain a new T value
```

and `val mut` likewise produces a fresh `T` result carried by a result slot/view
whose `PolicyMode` is `mut`. This does not assert `Writable(result)`. A relational declaration expresses its input modes with a formal-local hole:

```lang
const let const(self, <p> p let object:T) -> T
{ const let r = object |> T; r; }

mut let mut(self, <p> p let object:T) -> T
{ mut let r = object |> T; r; }
```

Here p belongs to the formal extraction site, not a lifted P1 binder.
Pin=Overlay(HorizonPolicy(P2),Mode=rho(p)); inherited coordinates retain that ordinary view.
With no result override, Pout=P1. Three input rows are a finite explanatory
expansion of each declaration, not three required primitive declarations.

First:

```text
PolicyConversionIsConstruction
```

`val const` and `val mut` both generate a new value. Therefore in general:

```text
CarrierPlace(result) != CarrierPlace(source)
```

when both reside in places. Even

```text
ValueEquality(result, source)
```

does not imply:

```text
PlaceIdentity(result, source)
```

Second:

```text
PolicyConversionIsNotInPlaceCast
```

There is no `mutate-policy-tag(source)` primitive.

Third — the most important one under the current `τ` model — conversion
capability is recoverable from `τ` itself:

```text
T is the complete type value τ

object |> T
    gets candidates from CallSpace(τ) = V_τ
```

The construction/reconstruction capability corresponding to type formation is
part of the complete `τ` snapshot, so `copy τ`, `return τ`, and `store τ` all
keep the knowledge of how to attempt reconstruction. It is never recovered by
going back out of `τ`:

The value directly supplies CallSpace(τ); no defining-binding recovery step
participates.

The global `const` / `mut` dispatcher itself does not guarantee conversion
success. The required order is:

```text
select global const/mut dispatcher
-> execute its body
-> ordinary invocation object |> τ
-> this invocation may succeed/fail according to τ's callspace
```

Formally:

```text
ExplicitConst(v:T)
    -> fresh T value through ordinary T invocation

ExplicitMut(v:T)
    -> fresh T value through ordinary T invocation

CallSpace(T) = V_τ        -- the conversion-capability source
```

If the inner ordinary invocation fails, the failure is final for this
candidate: the resolver does not go back and re-select another global
`const` / `mut` overload.

PolicyMode has two points, const and mut. CompileInstance completion uses the
independent OpenPolicy = {open, close}; it is not a reconstruction
operator inventory. Copying uses ordinary share/rebind, selected clone, a fresh
complete result and terminal Move, as owned by
[mechanical passing](../mechanical-lowering/mechanical-argument-passing-and-move-fixed-point.md).

## 2. Pattern alternative and policy operators

```lang
let bool = ((true | false)bool)$;
let true = true::bool;
let false = false::bool;
let if = true::bool;
let else = false::bool;
```

The two bare branches have distinct unique actual value members with
unobservable machine representation. Pattern stage is compile; value stage may
be compile or runtime. The ordinary alternative relation matches these values.

### 2.1 Policy material and two observations

```text
ValuePolicyObservation(x,epsilon) = Pv(x,epsilon)
PolicyObservation(TypeProjection(x,epsilon)) = Pp(x,epsilon)
```

Both observations retain the same source edge epsilon. Directly composing
`get_type` with Policy observation may expose Pp; first binding
`let t = x |> get_type` creates a destination whose later Policy is not
automatically the original x's Pp. Equal type content does not recover that edge.

Whole-slot Mode, Safety and namespace visibility/export retain their independent
owners; none is moved into Pv/Pp. The adjacent-rank coherence law of §1
applies to the same source/classifier relation. A hidden Val1 is not evidence
of absent content.

Concrete atoms, explicit holes, omission and polarity flip are distinct:

```text
runtime let ...  ~=  <> runtime let ...
runtime let ...  !=  <runtime> runtime let ...
<p> p ...       -- explicit HoleBinderId and ordinary extraction
<> p$ let ...   -- observe an existing value and interpret it as Policy material
<> p let ...    -- not implicitly the preceding polarity flip
```

The [interpretation-polarity judgment](structured-path-algebra-and-interpretation-polarity.md)
evaluates its operand once at a reached occurrence, requires Policy-admissible
ready material, and preserves existing HoleBinderIds/scope. It introduces no
hole, string parser, phase-copy side effect or fallback from unavailable value
to same-spelled atom. Omission supplies no constraint; default completion is
separate. The exact absent-value spelling remains open.

A demand on both Pv and Pp uses ordinary Patterns/require on these two
observations in the same candidate-local joint relation. Internal PolicyPair
carriers and endpoint tuples may retain both facts, but public syntax consists of independent constraint material. Raw/Norm Policy constraints preserve
ordinary atoms and orthogonal conjunction; they provide no pair/choice carrier.

### 2.2 Algebra

A completed stage coordinate is one atom:

```text
Stage = {compile, runtime}
OrdinaryStageOrder = Id ∪ {(compile,runtime)}
ScopedReverse_K(runtime,compile) iff ReverseAllowed_K(selected migration)
```

The compile atom uses the ordinary static
evaluator; neither that fact nor execution readiness establishes a stage edge.
The static-to-runtime edges require an admitted ordinary same-Type migration.
Reverse migration uses the active reify P2 / closed-return judgment of §3.5.
It is a selected scoped operation, not an unconditional stage-order edge.

An unresolved solver may retain several valuations, each with one stage atom.
Genericity uses explicit holes, ordinary extraction, legal orthogonal `+`,
polarity flip and require. It does not use a resolved stage union. Presence alternatives use their
ordinary Pattern relation, not a stage union. Independently registered ordinary
`||` outside this Policy sublanguage is unaffected.

Internal endpoint descriptions such as `runtime:compile`
retain distinct value/type facts; they are not source literals.

### 2.3 Deduction is ordinary operator Pattern extraction

Policy + consumes its registered ordinary operator relation under type.
Ordinary extraction, polarity and require retain typed coordinate restrictions.

    R_+(h1,h2,p,rho)
    independent holes -> distinct HoleBinderId
    repeated hole -> shared identity/equality constraint
    require C -> {rho in Solutions | C(rho)}

There is no independent policy deduction calculus:

    P1, P2 independent
    Pin_i(rho) = Overlay(HorizonPolicy(P2), Delta_in_i(rho))
    Pout(rho) = Overlay(P1, Delta_out(rho))
    Omega = <Actual_1,...,Actual_n,Demand?>

Input and output positions participate equally in this first-round relation.
Its Policy facts come from declarations, actual observations and total demand,
not execution of second-round candidate bodies. The rounds are therefore
decoupled without an input-to-output or output-to-input inference direction.
Applicability solves the one joint relation against Omega. It does not compute
Pin from Pout or Pout from Pin. Shared input/output holes must be in ordinary
scope; independent holes stay distinct until require relates them. Solver order
may exploit known actuals first, without becoming semantic direction.

    PatternSolve -> require/default/hard admissibility
      -> fully applicable solutions -> policy/specificity preference
      -> unique selection -> DynamicLegality

Preference never legalizes an invalid head. CapabilityRealization remains a
candidate/family fact; failed dynamic legality never reopens selection.

## 3. Contextual elaboration of P1

Policy contexts can share a binding-shaped surface slot while using their
position-specific elaborators. P1 open-instance policy has the following
qualification before ordinary slot projection.

Two named inference operations must remain distinct:

```text
DeclarationSidePolicyInference(declaration, initializer)
  -> CandidatePolicySig
  // when a callable declaration omits declared policy material, form the
  // callable's own declared pair/mode; no call-site actual is consumed

CallSitePolicyDemandFormation(context, written demand)
  -> CallPolicyDemand
  // form written/inherited/contextual constraints, then any required completion

PolicyOverload(
  CandidatePolicySig × CallPolicyDemand
)
  -> product partial order over fully admissible candidates
```

These operation names describe consumers of one ordinary Pattern relation,
not separate inference ontologies. Concrete modes, holes and omission remain distinct.
Declaration and call contexts supply observations of the same joint relation.

### 3.0 Instance P1 open policy

    PolicyMode = {const, mut}
    OpenPolicy = {open, close}

P1 open retains an already established opening source. P1 close completes the
instance and performs Close **after ordinary result delivery**. P2 independently supplies an evaluation Stage. Neither coordinate implies the other. The contextual
qualification applies to the current instance and its ordinary type/type ref views; it creates no instance, opening
window, Writable proof or lifetime extension.

Mode deduction is completed only after explicit const/mut constraints and
ordinary Pattern deduction. Explicit Hole identities remain distinct from omission:

    DefaultMode_K(x) =
      mut   if OpenPolicy_K(x)=open and OpenHere_K(x)
      const otherwise

This applies only to a still-omitted mode. Unknown OpenHere facts are unavailable,
not a proof of the otherwise branch. A mode derived as mut is a current
continuation-relative observation, never a permanent property of an instance root.

The body executes under open; successful ordinary result assignment/replacement
precedes outward completion/Close. Later omitted mode completes const when
the current OpenPolicy/OpenHere condition is false. close is not const. Completion retained under
open preserves the established opening source, not a new one.

Direct initialized-name mut borrowing and explicit open-ref-to-mut confirmation
retain their actual Place, borrowed generation and opening subject. Both require
current OpenHere, target Writable and ordinary capability/access/type/lifetime.
Close defeats mutable acquisition and saved-ref writes. One-shot initial type
slot authority remains independent. See the
[type/ref owner](type-values-places-and-borrow-views.md#522-initialized-type-names-open-references-and-mut-confirmation).

OpenHere controls construction/mutation qualification, not the instance lifetime.
Instance survival, movement, escape and dependencies remain ordinary lifecycle
relations. Ordinary Val2 payload policies remain independent. The
[invocation owner](../static-evaluation/compile-instance-invocation-and-result-delivery.md)
owns self-name observation, ordinary delivery, identity and current storage reads.

### 3.1 Ordinary binding projection

P1 let x = expr forms a complete lexical binding. Written const/mut, explicit
Hole, deduced mode and omission are distinct inputs. The independent completion
qualification defaults to close unless the established context retains open.

    written := WrittenPolicyConstraints(prefix)
    pending := FormCandidateIndependentDemand(written, immediate_context)
    mode := CompleteOnlyOmittedMode(pending, OpenPolicy, current OpenHere)
    demand := CompleteResultPolicyDemand(pending, mode)
    result := ordinary E(expr, demand)  // demand precedes root maxima
    producer_mode := frozen selected result mode
    destination_view := ordinary projection and satisfaction
    transfer := selected ordinary realization followed by terminal Move

Missing OpenHere needed by open completion diagnoses unavailable; it cannot
supply a concrete preference. The total output demand must exist before maxima.
An unresolved outer candidate cannot send formal constraints into an inner call.
Selected body, extraction, migration, delete or transfer failure never reopens it.

Mode preference does not change the selected producer mode. Existing exact
outward views satisfy first; otherwise one direct same-Type selected migration
owns coherent PolicyProjection and ValueRealization. Copy-derived delivery
retains share/rebind -> clone -> fresh complete result -> Move. Preserve Move
is not copying and invokes no clone. Destination assignment and CompileInstance
Complete/Close are distinct boundaries; completion policy supplies no value cast.

### 3.1.1 Explicit expression result-Policy context

An expression may override or delimit the default evaluation result context
with an explicit candidate-independent result demand:

```text
PolicyLetExpression ::= PolicySpec "let" PipeExpression
```

`PolicySpec` here denotes canonical typed Policy material (§2), including a
opposite-context interpretation of an ordinary value. The operand covers the
complete following pipe; parentheses close
the boundary:

```lang
P let a |> f          // P let (a |> f)
(P let a |> f) |> g   // f closes under P before g is selected
(P let a) |> f        // only a is inside the boundary
```

The syntax is not required merely to evaluate a call in `compile` or
`runtime`. Without it, current-phase evaluation still checks the candidate's
stage view, using the stage default completion only where applicable.
`compile let e` / `runtime let e`
remain available when the programmer wants an explicit stage boundary or
migration target. `const let e` / `mut let e` are the orthogonal explicit
ModeAtom cases that add a written output-mode demand locally.

The normative judgment is:

```text
PolicyLetFormation:

  pi := ElaboratePolicySpec(P, ResultPolicyContext)
  sigma := ExpressionResultSlot(PolicyLet(P, e))

  Gamma ; ResultPolicyDemand = pi
    |- e ⇓ r

  S := SourcePolicy(r)
  T := TargetPolicy(pi, sigma)
  C := PreparePolicyMigrationCandidates(S, T, ResultPolicyDemand)
  m := Unique(PolicyOverload(C, PolicyMigrationDemand(S, T)))

  rho := PolicyProjection(m, r, sigma)
  v := ValueRealization(m, r, sigma)
  require CoherentPolicyMigrationResult(m, rho, v)
  result := CompletePolicyMigrationResult(m, rho, v)

  --------------------------------------------------
  Gamma |- PolicyLet(P, e) ⇓ result
```

The one syntax node has two projections. Its inward projection supplies `pi`
before the maxima of the operand root call are chosen. Its outward projection
forms a completed accepted Policy view; it does not leave an expected-result
variable for an outer consumer. The selected operand producer keeps its
concrete `ResultPolicyMode`.

`sigma` is the ordinary semantic result position already owned by this
expression node:

```text
PolicyLetResultSlot:

sigma = ExpressionResultSlot(PolicyLet(P, e))

sigma is not:
  a NameBinding
  a name binding
  a hidden declaration
  an independently acquired or source-addressable Place

PolicyMode(sigma) = ConcreteMode(pi)
ConcreteMode(pi) = resolved written/inherited/contextual constraint
                or a separately required DefaultModeCompletion
```

The slot is not an anonymous variable and creates no source entity. It is the
ordinary result carrier through which a completed expression view is exposed
to its parent expression. The outward view exposes exactly the concrete
whole-slot mode completed under the local context. Omission is not an explicit
const constraint; typed coordinate legality still applies to written Patterns. Any
residual `Pv:Pp` constraints remain part of `pi`; each solved stage is one atom.

Producer preference and outward acceptance are different relations:

```text
ProducerPreferredUnder(mu_demand, mu_candidate)
  = preference by succ_mu_demand

ExistingOutwardModeAccepted(mu_demand, mu_result)
  iff mu_result = mu_demand
```

Thus a `mut` producer may uniquely win under a `const` output preference, but
its `mut` result is not already an outward singleton-`const` view. The
producer fact remains frozen while the expression-result slot receives its own
mode.

Outward completion is one selected Policy migration, not an independently
created cast plus an optional second action:

```text
r := frozen operand result
S := SourcePolicy(r)
T := TargetPolicy(pi, sigma)
C := PreparePolicyMigrationCandidates(S, T, ResultPolicyDemand)
m := Unique(PolicyOverload(C, PolicyMigrationDemand(S, T)))

rho := PolicyProjection(m, r, sigma)
v := ValueRealization(m, r, sigma)
require CoherentPolicyMigrationResult(m, rho, v)

ExposeInExpressionResult(sigma, CompletePolicyMigrationResult(m, rho, v))
PolicyMode(sigma) = ConcreteMode(pi)
ResultPolicyMode(r) remains unchanged
```

The selected candidate `m` owns declared source/target Policy endpoints and
produces both projections of the same migration: `PolicyProjection` and
`ValueRealization`. Their coherence is checked before the result is completed.
An exact existing accepted view is represented by the identity migration
candidate; its Policy projection preserves identity and its value realization
is the same value. A non-identity candidate may realize its value side through
an established Type callspace, ordinary Val2 body, or canonical mechanical
action. None of those bodies independently defines the Policy edge.

`ExposeInExpressionResult` is observation, not a copy into a new Place. Failure
to select a unique migration, execute its value realization, or establish
coherence is a typed post-producer failure and never reopens operand selection.

Transport preserves the selected ordinary realization and fixed MoveEffect.
Failure after producer selection is terminal; outward completion cannot expose
an unsatisfied view or reopen the producer.

```text
PolicyMigrationNotDerivedFromValueCall:

MigrationCandidate(m, SourcePolicy(r), TargetPolicy(pi))

PolicyProjection(m, r) ⇓ rho
ValueRealization(m, r) ⇓ v

CoherentPolicyMigrationResult(m, rho, v)

ordinary Val2 action =/> creates the inward ResultPolicyDemand
ordinary Val2 body =/> defines the Policy transition endpoints of m
ordinary Val2 action =/> replaces PolicyLet
PolicyLet =/> is itself or lowers to an ordinary Val2 call
```

The reason is temporal: an ordinary call can be selected only after its input
expression exists, while `PolicyLet` must contribute its demand before the
operand root call is selected. A Val2 operation may be the selected migration
candidate's `ValueRealization`, but it cannot retroactively create that demand
or independently establish the candidate's declared Policy edge. Policy
migration is not the forbidden in-place `mutate-policy-tag(source)` operation
of §1.2. `const` satisfaction does not imply a global `val const` dispatcher.

```text
NoCrossCallPolicyPropagation:

PolicyLet(P, e)
  -> establish ResultPolicyDemand(P)
  -> select/evaluate e once under that demand
  -> uniquely select one Policy migration for SourcePolicy(result(e)) -> P
  -> obtain its coherent PolicyProjection and ValueRealization
  -> close the boundary

outer consumer demand
  =/> modify ResultPolicyDemand(e)
  =/> reopen Candidates(e)
```

Failure of the outward satisfaction step is a typed failure after the operand
selection and never reopens that selection. Thus `(P let a |> f) |> g` gives
`g` one ordinary actual carrying the already-completed concrete view.

The three operations are distinct:

```text
DeclarationSidePolicyInference
  -> forms a callable's declared result policy

CallSiteImplicitDemand
  -> binding/call context supplies its candidate-independent default demand

ExplicitExpressionDemandAndMigration
  -> PolicyLet supplies an explicit local demand and one completed migration
```

Concretely:

```lang
Q let x = expr;
```

Its unrefined Stage demand has two different scopes. Define:

```text
j = FirstPresent(Pi_e) = min { i | Pi_e(i) != absent }
Target_runtime(Pi_e)[i] = runtime if i=j
                         Pi_e(i) otherwise
Target_compile(Pi_e)[i] = absent  if Pi_e(i)=absent
                         compile otherwise
AllCompile(Pi_e) iff forall i. Pi_e(i) in {absent,compile}
```

For runtime let, runtime:compile stays runtime:compile; compile:compile targets
runtime:compile; absent:compile targets absent:runtime without manufacturing
a value. Compile let requires all non-absent ranks compile, preserving the
absent prefix: runtime:runtime targets compile:compile, and
absent:runtime:runtime:compile targets absent:compile:compile:compile.
This is a demand over the entire chain, not merely two writes in a pair.

An observed compile rank plus Legal(Pi_e) proves the compile tail without
enumerating further ranks. A finite all-absent prefix provides no such proof;
its higher-rank observation consumer is required. Building a target or a
prefix is not migration success. Actual migration still requires its selected
ordinary candidate, actual classifier links and legal continuation. Explicit
Hole-local Policies can target individual ranks under the same chain law.

Projection preserves the complete result relation. If an entry's required
higher rank is unknown, it reports unavailable rather than returning a
partially classified result set or treating that entry as a mismatch. Neither
another matching entry nor a fallback supplies the missing observation.

Two ordinary observation constraints may jointly constrain both components,
using the same source edge and candidate-local relation. The public surface
does not construct or extract a `Qv:Qp` pair.

Projection returns an identity-preserving restricted view. Given:

```text
Pv = runtime
Pp = compile
```

the projection `runtime` produces:

```text
Pv = runtime
Pp = compile
```

The query accepts this concrete observation without reconstruction. Binding
and Pattern identities remain unchanged; there is no second stage slice.

Result-view satisfaction is existing-view-first:

```text
SatisfyResultView(source, demand):
  S = ProjectResultPolicyDemand(demand, source)
  if S != empty:
    return ExistingView(S)
  otherwise:
    enumerate one authorized same-Type migration family
    perform ordinary candidate selection exactly once
    return SelectedMigration
```

An existing projection preserves the source semantic identity and does not
enumerate migration candidates. Migration is considered only after the exact
projection fails. A selected migration is sealed; projection, realization, or
DynamicLegality failure never reopens selection. These rules apply to the
complete `PolicyResultEntry[]`, including collections that mix value-bearing
and absent-Val1 entries.

For the internal joint query <Qv,Qp>, result-view satisfaction slices the Pattern-policy stage
capability before migration candidate enumeration:

```text
Pp_selected = SlicePatternPolicyStages(Qp, source.Pp)
```

This is Policy slicing over `Pp`; it is not Pattern extraction, type
projection, postfix `?`, extractor lookup, Pattern-root navigation, or a change
of PatternRoot/PatternScope. It preserves the Object's P coordinate and structural
Pattern shape.

Unselected ordinary Pattern solutions in a query are never obligations to
manufacture every branch. When the complete query projects nothing, only an
authorized direct same-Type migration may satisfy the demand. There is no
transitive search or compiler-owned conversion table; the implementation
contract is `../../contracts/policy-migration.md`.

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

Pin and Pout have independent parents and different stage rules:

    Pin = ElabIn(HorizonPolicy(P2), Delta_in)
    Pout = ElabOut(P1, Delta_out)
    bare let -> Delta = empty
    written atom -> explicit override
    <p> p let -> mode overlay containing the formal-local HoleBinderId p

    let x          -> Pin=HorizonPolicy(P2)
    const let x    -> Overlay(HorizonPolicy(P2), Mode=const)
    mut let x      -> Overlay(HorizonPolicy(P2), Mode=mut)
    <p> p let x    -> Overlay(HorizonPolicy(P2), Mode=rho(p))

Pin may override stage with an explicit atom or an ordinary Pattern hole.
For example a runtime-horizon callable can have one compile Pin and one
runtime Pin. Missing input stage inherits HorizonPolicy(P2); a hole is solved, never defaulted
as if omitted. InputAdmissible checks each actual against its position, rather
than comparing every input stage with P2 in a total order. Omission, explicit concrete mode and explicit hole are not interchangeable.
Visibility/export do not acquire invented formal/output counterparts.
SafetyPolicy retains its own independent consumer meaning.

The selected PolicyMode is a formal preference input. It is not an ordinary P1
query applied to the actual argument. Consequently an oppositely qualified
actual is not removed before the product order. The two context-indexed
relations are:

```text
succ_const: const > mut
succ_mut:   mut > const
```

No neutral preference point exists. Distinct tied candidates remain ambiguous;
declaration order cannot choose one.

The elaborated formal view is not body-local policy metadata. Candidate
formation exports its whole-slot PolicyMode into the callable's parameter
Policy product position:

```text
FormalPolicyMode(parameter)
  -> Candidate.parameter_policy[position]
  -> MaxPolicyProduct
```

Thus the elaborated Pin governs each body's input observation, while its
whole-slot mode participates in comparison against fully admissible overloads.
Implementations must not collapse `const` back into an unspecified carrier.

#### 3.2.1 Return policy refinement inherits P1

There is no independent complete `P3` Policy product. A return position has a
position Policy `P_out` formed by default from the callable declaration P1 plus an
optional mode-only overlay:

```text
P_out = Overlay(P1(callable), Delta_out)

stage(P_out) = stage(P1(callable))
```

An omitted mode preserves P1's mode. An explicit spelling selects the mode
under this return-specific rule while leaving P1's stage/exposure
pair unchanged:

```text
return let x        -> P1 unchanged
return const let x  -> inherited P1, PolicyMode = const
return mut let x    -> inherited P1, PolicyMode = mut
```

The mode may not alter stage, value presence, Pattern policy, ordinary
visibility, or export-root status. Policy dimensions are not replace-all: each
dimension must be classified as `InheritedOnly` or `Overridable`; evaluation
stage is `InheritedOnly` here and whole-slot mode is `Overridable`. “No P3”
therefore means that the return site has no third arbitrary complete Policy
vector; it does not mean that the return position has no Policy.

Hole-local return extraction is a separate HoleOverride consumer: each explicit
Hole Policy addresses its own rank/position. Mixed coherent compile/runtime
output observations do not recolor siblings and are not restricted by this
ordinary-slot mode overlay. The complete output constraints participate in the
same pre-maxima relation as input constraints.

`P_in` and `P_out` are declaration/evaluation-boundary facts. A caller's
`ResultPolicyDemand` is a distinct call-site judgment and never rewrites either
position Policy. It can affect candidate admissibility, preference, and
outward view satisfaction only through the ordinary sealed invocation
pipeline (`NoCrossCallPolicyPropagation`).

### 3.3 Namespace declaration attributes

`public`, `private`, and `export` are accepted only by namespace-declaration
elaboration. They are rejected in ordinary P1, formal parameters, return
slots, P2, Pattern interiors, expression policies, and local declarations that
are not namespace declaration positions.

`export` has the narrower placement rule described in section 9. Export
elaboration derives a separate stable external candidate snapshot; it does not
crop the namespace's complete internal declaration view. Export admission is
determined by retention plus public reachability, not by a universal const
projection and not by a future consumer's Policy demand or
`DynamicLegality_Γ`. Consumer legality is formed only after external lookup and
ordinary invocation selection.

Absence removes the complete value subspace of *this observation edge* rather
than merely selecting a presence tag:

```text
Pv = absent
  => no value-stage coordinate in this observation
```

The review matrix is therefore complete rather than shape-dependent:

| Observed Val1 | const | mut |
|---|---:|---:|
| present | valid mode coordinate | valid mode coordinate |
| absent | valid mode coordinate | valid mode coordinate |

The cells assert only that the mode coordinate exists. They do not manufacture
a value stage or any operation capability.

This is a statement about the edge, not about the object behind it. Per §1,
`Pv = absent` does not assert `Val1?(x) = null`; Val1 shape supplies no equality
between the Policy of x and the Policy of its classifier.

Value-side well-formedness may reject combinations with an absent value
component, but that restriction does not erase or change the independent
whole-slot PolicyMode.

### 3.4 Policy migration satisfaction: existing first, unique migration second

`PolicyDemand` may be retained as consumer-origin metadata:

```text
PolicyDemand
  = BindingP1Demand
  | ParameterPolicyDemand
  | ResultPolicyDemand
  | MechanicalPolicyDemand
```

This enumeration does **not** give all demand kinds an arbitrary conversion
search. It supplies demand-kind admission facts to one Policy migration
algebra. Every admitted candidate has declared source/target Policy endpoints
and produces both a Policy projection and a value realization:

```text
PolicyMigrationCandidate m:
  SourcePolicy(m)
  TargetPolicy(m)
  PolicyProjection(m, result)
  ValueRealization(m, result)

SatisfyPolicyDemand(demand, result):
  Q = AcceptedPolicyQuery(demand)
  existing = ProjectExistingViewForDemand(Q, result)

  if existing != empty:
    C = { IdentityPolicyMigration(existing, Q) }
  else:
    C = DirectPolicyMigrationCandidates(
          SourcePolicy(result),
          TargetPolicy(Q),
          demand.kind)

  D = PolicyMigrationDemand(SourcePolicy(result), TargetPolicy(Q))
  m = Unique(PolicyOverload(FullyAdmissible(C), D))

  rho = PolicyProjection(m, result)
  v = ValueRealization(m, result)
  require CoherentPolicyMigrationResult(m, rho, v)
  return CompletePolicyMigrationResult(m, rho, v)
```

When `Q` contains a whole-slot ModeAtom, an existing outward view is accepted
only when its concrete mode equals that point. The
`succ_const` / `succ_mut` relations rank producer candidates;
they do not widen the set of concrete modes accepted by outward satisfaction.
In particular, a `mut` producer that wins under `const` preference is not an
existing singleton-`const` outward view.

The two result projections are inseparable outputs of the selected migration:

```text
SelectPolicyMigration(SourcePolicy(r), P) ⇓ m

PolicyProjection(m, r) ⇓ rho
ValueRealization(m, r) ⇓ v

SatisfyPolicyDemand(P, r) ⇓ result
iff
  UniqueSelectedPolicyMigration(m)
  and CoherentPolicyMigrationResult(m, rho, v)
  and result = CompletePolicyMigrationResult(m, rho, v)
```

`PolicyProjection` is not independently formed before candidate selection, and
`ValueRealization` is not an optional proof supplied afterward. Both belong to
`m`. An ordinary Type-callspace/Val2 operation or mechanical transfer may
implement `ValueRealization(m, r)`, but the migration candidate's declared
Policy endpoints define the transition. Ordinary Policy overload performs the
unique selection; no PolicyLet-specific selector or second transition algebra
exists.
Once `m` is selected, failure to execute either projection or establish
coherence does not reopen the operand candidate set.

For every demand kind:

```text
ProjectExistingViewForDemand(demand, R) != empty
  => candidate set is exactly {IdentityPolicyMigration}
  => no non-identity migration candidate enumeration
  => no non-identity migration invocation
  => no value reconstruction
  => PolicyProjection(identity, R) preserves the accepted view
  => ValueRealization(identity, R) = R
  => name binding / TypeValue / P coordinate / Place identity is unchanged
```

This is the **Existing-First, Constructible-Second** principle:

```text
1. existing accepted views
2. language-constructible accepted views
```

A stage-changing migration has explicit selected source and target atoms.
A runtime-to-compile endpoint requires the reify boundary judgment. Unresolved solver alternatives are not a completed Policy view.
Migration evidence may constrain R_vis, but speculative candidate enumeration
executes no migration body or effect.

```text
OnePolicyMigrationAlgebra:

ordinary binding Policy completion
PolicyLet outward completion
forward or scoped reverse same-Type migration
  -> PreparePolicyMigrationCandidates
  -> ordinary Policy overload / unique selection
  -> selected m
  -> PolicyProjection(m) × ValueRealization(m)
  -> CoherentPolicyMigrationResult
```

Demand kinds restrict which direct candidates are admitted; they do not own
different selectors. Stage-changing cases admit the direct authorized family of §3.5. A PolicyLet result-slot mode
transfer admits the corresponding identity or canonical mechanical
realization ending in Move, with any selected clone-derived producer path
made explicit. Ordinary binding consumes the same candidate
algebra. No demand kind may reinterpret an arbitrary ordinary value call as a
Policy transition merely because its return value has a useful shape.

`BindingP1Demand` uses the exact conservative `ProjectP1` theorem in §3.1.
Formal parameter and result consumers retain their existing policy-Pattern and
applicability rules. An existing accepted concrete view is consumed without materialization.
An unresolved stage hole is not an instruction to construct all possible views.

`MechanicalPolicyDemand` records the origin of a language-selected mechanical
realization within a selected migration. It does not imply that arbitrary
Policy failure may search `ref`, `share`, `@`, or another structure-changing
operation. Those operations occur only when separately required by their own
language rule and then use ordinary function-object invocation.

### 3.5 Direct bidirectional migration

```text
source -> Project_in -> one selected Migration -> result -> Project_out
ExistingAcceptedView(demand,source) => identity only
Selected(m) => same Type at both endpoints
Reverse(m) iff any observed stage coordinate changes runtime -> compile
Reverse(m) => ActiveBoundary_K(m)=B, P2(B)=reify, ClosedReturn_K(B)
```

Endpoints include both value and classifier observations. Thus runtime:runtime
to runtime:compile changes the classifier observation alone; compile:compile to
runtime:compile is the ordinary value materialization case. An endpoint may also
carry an absent value projection. Actual Object shape, presence and Type remain
independent premises, and the migration candidate preserves their coherence.

Both directions use the same direct family and ordinary selection. Mechanical
generation supplies bidirectional machine-Type declarations and can derive
further families from actual component realizations. Deletion, missing external
realization, or a restricted library type can remove either direction. Generation
is not capability evidence when a required declaration/realization is unavailable.

A reverse request requires the current continuation's active reify boundary and
independently closed return classifier, checked before invocation and publication.
It materializes only actually dependent successor work with the same E; it does
not return runtime information to an earlier world. The current-stage and Ready
consumers remain separate from the endpoint relation. Storage, a freely supplied
flag or matching Type does not supply active-boundary evidence.

Mode, endpoint applicability, capability and DynamicLegality retain their ordinary
roles. One selected failure is terminal; there is no chain search, Type repair,
implicit borrow formation or reopening.

### 3.6 Producer visibility, admissible inputs and readiness

P1/Pout controls producer visibility; P2 controls evaluation horizon;
InputAdmissible controls input position compatibility; Ready controls whether
this action can execute at the current continuation frontier. None replaces
another. A runtime-produced result remains runtime-produced even if all its
inputs are known during static evaluation. Hiding its unreadable Val1 does not
remove its Object, Pattern, Val2, binding or argument slot.

A compile callable can have ordinary implicit dependencies. When a required
concrete runtime value is unavailable, its actual remaining continuation waits
for that dependency. Under a closed reify P2 boundary, selected reverse migration materializes
only its actual dependent E cone; value
availability does not change semantic name coordinates or producer identity.

### 3.7 Progressive evaluation and sealed source identity

A source candidate c may have a family of ordinary compile realizations:

```text
C(c) = { C_sigma(c) | admissible projection configuration sigma }
Selected = (c*, sigma*, InvocationFrame)
```

The family may be represented lazily. Each C_sigma has ordinary call structure,
a correspondence to the same source invocation, and its concrete visibility
and input evidence. Unreadable runtime Val1 is hidden without deleting
arguments, Pattern observations, Val2 or identity. Selected CompilePartner(c)
has separate anchored identity; it is not this projection family.

Resolve once, progressively evaluate ready work, and retain unavailable
dependencies in the same continuation. Every projection and runtime residue
of Selected retains c*, sigma* and the fixed frame. Runtime resumes that
invocation, never namespace lookup or candidate selection.

Residual IR, cache layout, continuation ABI and scheduling algorithms remain
implementation choices. Their observations must preserve source invocation
identity, actual read/write/borrow/lifetime/effect dependencies and cleanup
order. They cannot relabel stages to obtain readiness.

### 3.8 Static frontier and deferred materialization invariants

Static evaluation continues while the expression and its dependencies are
evaluable in the current static phase. Lexical occurrence inside a runtime
body is not by itself a runtime-computation boundary. The frontier is:

```text
static evaluation frontier
  = first dependency/effect boundary not admissible in the current phase
```

The following invariants hold independently of storage and lowering:

- Crossing a compile value to runtime constructs a new runtime object. It does
  not extend the lifetime of a compile temporary.
- Every addressable runtime value has an ordinary runtime owner/place. There is
  no third category of ownerless addressable temporary.
- A future static-materialization cache keys an ordinary compile value by its
  canonical static-value identity. A compile reference is keyed by compile
  referent identity, not by pointee value equality. Concretely, the borrow-view
  leaf normal form contains
  `⟨BorrowKind, StableTargetIdentity(Target(view))⟩`; two targets remain distinct
  even when their current contents normalize equally.
- Cache keying does not swallow the caller's construction context wholesale.
  Canonical value identity and `Anchor`/`WindowLive_Σ` remain separate inputs to
  applicability. A `compile` function that calls ordinary type composition `*` on a transported
  type, or ordinary type update `*=` through a ref, may be legal or illegal for the
  same normalized contents in different stacks:

  ```text
  Eval(F, t; Γ_open)  ≠  Eval(F, t; Γ_closed)
  ```

  `*` requires `OpenHere_Σ(value)`; `*=` independently also requires
  `Writable_Γ(Target(ref))`. A `type ref` key preserves referent identity but
  proves neither current premise. Cache the pure value computation separately
  from applicability, or record/recheck those requirements in a function
  summary. Admitting the whole lexical context into canonical value identity is
  forbidden; the current stack is consulted only by the applicability judgment.
- Storage requested by `[[global]]` materialization does not mutate the
  source-visible `NamespaceGraph`; generated storage and source namespace
  declarations remain distinct semantic facts.
- A language-selected `ref`, `share`, or `@` operation may compose
  ordinary operations and apply its own type/access rules. Such a structural
  operation is not Policy-demand repair.

These are deferred positive constraints, not claims that runtime lowering,
cache identity, ordinary namespace observation, or lifetime checking is currently
implemented.

### 3.9 Direct result delivery and two-sided forwarding

#### 3.9.1 Select the return target before delivery

Existing implicit, explicit and targeted-return rules select the target.
Delivery does not redefine whether an in-place block introduces a return target.
For the selected F:

```text
Pout_F = ElabOut(P1_F, Delta_out)
BindResult(ReturnPattern_F, e)
```

The terminal e is interpreted under this result demand. The schematic let r=e
names the existing result position and whole ReturnPattern; it does not create
a lexical local. Product and borrow results retain their ordinary delivery
rules.

#### 3.9.2 No implicit semantic temporary

Default return must not mean:

```text
let temp = e;       // independently infer/complete temp's Policy
let result = temp;
```

The selected return position supplies immediate demand before e's root call
forms its maxima. Registers, SSA temporaries and argument buffers may exist,
but they introduce no extra language binding, default Policy boundary or
observable LifeName.

An explicit user-written let temp=e; temp; does create a binding boundary and
need not be equivalent to directly returning e.

#### 3.9.3 Both forwarding sides constrain the inner call

For the terminal inner call G of transparent wrapper F:

```text
C_in(P2_F, P2_G) and C_out(P1_F, P1_G) and C_positions
(P1_F, P2_F) => (P1_G, P2_G)
```

This is not the linear propagation P2_F -> P2_G -> P1_G -> P1_F.
The constraints use ordinary admissibility, preference, satisfaction and
migration; they do not require literal equality of every coordinate.

While the selected F's body is interpreted, its established signature and
valuation are G's known immediate context. An unselected outer candidate's
formal demand is not such a context.

#### 3.9.4 Selection remains sealed

For schematic terminal H(G(...)), return demand first constrains H. It does
not inject the parameter Policy of an unselected H into G. G closes under its
own immediate context; H and F cannot reopen it afterward. Direct return
removes an extra default temporary, not the local selection boundary.

#### 3.9.5 Generic declarations avoid enumerating a Cartesian product

Transparent coordinates inherit. Coordinates requiring extraction or
correlation use explicit holes; one HoleBinderId constrains the corresponding
positions to agree, and require constrains legal combinations. Omission is
not an implicit generic variable over every dimension. Declarations need not
enumerate const/mut by compile/runtime cells.

PolicyMode preference, capability realization, stage admissibility, Ready and
DynamicLegality remain independent. Forwarding a Policy fact creates no write
capability, reference, OpenHere evidence or additional migration candidate.

## 4. P2 evaluation horizon and result observations

```text
Stage={compile,runtime}
OpenPolicy={open,close}
PolicyMode={const,mut}
```

Visibility, InputAdmissible, Ready and LegalToExecute are independent. Bare nodes
have actual unique opaque values, permitting runtime:compile observations.
All Objects retain the adjacent-rank coherence constraints. R_vis preserves
hidden resident identity; concrete value availability is a separate fact.

## 5. Callable P2, producer completion and return closure

```text
HorizonPolicy(compile) = ordinary compile evaluation view
HorizonPolicy(runtime) = ordinary runtime evaluation view
HorizonPolicy(reify) = HorizonPolicy(runtime)
omitted ordinary P1 => declaration-side default from HorizonPolicy(P2)
explicit P1 / Hole-local output Policies => ordinary joint constraints
P2=runtime and P1=compile is legal
Pin_i.stage <= latest input availability admitted by P2
```

P2 is evaluation horizon; P1 is the declared producer observation. No stage
matrix forces explicit producer/output constraints from P2 or from another
output position. Pin is a latest-arrival constraint and actual availability
remains required. An ordinary Pout slot inherits P1; explicitly scoped output
Holes use HoleOverride and may express mixed Policies. Their linked rank
observations must remain coherent, including runtime:compile/runtime:runtime.
Runtime-policy type values supply no parallel runtime type-calculation algebra.

Only callable-head P2 admits reify. Its outward horizon view equals
runtime, while its active closed boundary admits selected reverse migration.
Whether that boundary is required follows actual runtime-to-compile dependent
continuation work, not the spelling of a P1/P2 stage combination. A legal
runtime-horizon callable with compile output need not perform reverse migration.
The complete output classifier must be explicit or input-inferable without
depending on reverse migration. Unknown closure/continuation facts are unavailable.
Finally omitted mode still completes under open + known OpenHere, independently
of these stage judgments. HoleOverride retains its independent replacement law.

## 6. Observation horizons

```text
ObservationHorizon=OpenStatic | Runtime
Vis(compile)={OpenStatic}
Vis(runtime)={Runtime}
```

Exposure proves no readiness or execution legality. A hidden runtime value can
retain available Pattern facts. Concrete value use waits for ordinary required
facts; reify materializes the same E and actual remaining continuation.

### 6.1 Hole-local dimension replacement

```text
Policy(h)=HoleOverride(P_outer,Delta_h)
HoleOverride(P,Delta)[d] = Delta[d] if d in WrittenDims(Delta)
                         P[d]     otherwise
```

For P let <P_h let h>, omitted dimensions inherit; written coordinates replace
without outer-strength restriction. Pattern and Policy share HoleBinderId scope.
This third operation is independent of ElabIn and ElabOut restrictions.

## 7. Resolution, exposure, and facet reads

Every horizon distinguishes:

```text
Resolve(path, context, expectation)
ExposePolicySlice(binding, horizon)
ReadValue(slice)
ReadPattern(slice)
EnumerateValueFacet(slice)
EnterCallableBody(candidate)
```

Resolution consumes no horizon. Exposure observes the already-fixed binding;
hidden resident facets cannot suppress a binding, restart search, or remove a
name conflict. Failure to expose a value slice is not an unresolved binding. In particular:

```text
Pv = runtime
Pp = compile
```

has this OpenStatic behavior:

```text
binding/path resolves
runtime value is unreadable
compile Pattern/type is readable
the admissible compile-realization family C_sigma(F) may enter round two
original runtime computation remains in the common residual continuation
```

Conversely, exposing or selecting an existing runtime Policy slice in a static
phase is not permission to read its value:

```text
Stage(Pv) = runtime
  => the declared runtime view exists

current observation horizon is OpenStatic
  => ReadValue(runtime slice) is unavailable
  => preserve already-resolved runtime computation/residual
```

The runtime continuation consumes the preserved binding/callable identities. It
does not repeat path resolution or overload choice merely because the value
becomes readable later.

All bindings follow the ordinary identity/exposure rule. Their paths can be
resolved independently of whether a facet is exposed in the current phase.

## 8. Mechanical compile-flow projection

Static visibility and runtime residual observations belong to one semantic
continuation. They do not partition node kinds into separate execution flows.

    complete continuation -> static observation
                          -> runtime residual observation

Pattern/type work, ready calls, derived companions and actual delayed work retain
their ordinary stage rules. Runtime residue retains value computations, runtime
bodies/effects and branch value selection. D/Done and other control structures
are projections of the same actions and positions, not separately interpreted
copies.

Projection alone executes no call and selects no overload. E exhausts ready
work under the current continuation, stage, policy and facts; E E = E.
It does not rewrite runtime bindings into compile bindings to obtain more work.
[E and optimizer boundaries](../static-evaluation/evaluation-residual-and-optimization.md)
govern transformations and revalidation by all affected projections.

## 9. Namespace visibility and export

### 9.1 Three independent binding views

```text
Sigma_full    actual bindings for permitted internal lookup
Sigma_export  export-retained stable views with public path reachability
CurrentView   horizon/access-qualified observation of resolved identity
```

Existence, namespace access and exposure are independent. Private/hidden
residents retain identity. DynamicLegality follows ordinary selection.

### 9.2 Export roots and stable external projection

`export` is allowed only on a direct top-level declaration of one namespace
construction level:

```lang
export let name = expr;
```

Let `InternalView(s) = ⟨Pv:Pp, μ⟩`, where `μ` is the resolved whole-slot
PolicyMode. Export derives, rather than replaces, a second view:

```text
ExportAdmission(binding, path)
  = InExportRetentionClosure(binding)
    && PubliclyReachable(path)

ExportAdmission(binding, path)
  => for each candidate in FullOverloadSet(binding):
       ExternalView(candidate)
         = ExportSnapshotOf(ResolveCandidateSnapshot(candidate))
```

`Σ_export` is therefore stable for one committed namespace snapshot. It depends
on export retention and path visibility, never on a future consumer's
`policy_demand` or requested read/call/capture capability. It preserves
candidate identity, `Pv:Pp`, and `PolicyMode` without selecting an overload.

No PolicyMode is universally safe for a later operation. Stable
default/delete/custom `CapabilityRealization` facts may accompany a candidate,
but a concrete consumer forms `DynamicLegality_Γ_consumer` only after lookup
from `Σ_export` and ordinary selection. There is no `const <= mut` ordering and
external views do not perform a universal const projection.

If a future language design introduces publication itself as a capability, it
must be an explicit, demand-independent family:

```text
ExportCapability(candidate)
```

It must not consume a later caller's Policy demand and must not be disguised as
ordinary namespace visibility. No such additional publication filter is
defined by this document.

It is forbidden in function/compile-function bodies, parameters, return slots,
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
  -> ResolvedCandidateSnapshot {
       identity,
       pair: PolicyPair,
       mode: PolicyMode,
       realization_facts: CapabilityRealization[],
       provenance
     }
```

Only the resolved complete view can enter the stable external projection.
`P1Projection::Infer` is a valid declaration request, and
`P1Projection::Leading` carries a local first-present query, and AllCompile
carries the full-chain compile demand; neither
is an external candidate view.

The typed substrate therefore represents export as an identity-preserving
candidate transformation:

```text
ExportCandidateView {
  identity,
  internal_candidate,
  external_snapshot: ResolvedCandidateSnapshot
}

ExportAdmission {
  in_export_retention_closure,
  publicly_reachable
}

if admission.in_export_retention_closure && admission.publicly_reachable:
  for candidate in FullOverloadSet(binding):
    internal_snapshot := ResolveCandidateSnapshot(candidate)
    external_snapshot := ExportSnapshotOf(internal_snapshot)
    insert identity-preserving external_snapshot into Σ_export
```

`ExportSnapshotOf` preserves candidate identity, pair, mode, declaration/
intrinsic realization facts, and provenance. It carries no
`DynamicLegality_Γ` judgment: no such judgment exists before a consumer has
selected an invocation. Equality here is equality of stable candidate facts,
not equality of internal and consumer-context observation edges.

Export-retention-closure membership and public path reachability are separate
binding/name-level facts; both are required before a binding contributes to
`Σ_export`. In particular, a private child in an exported subtree and every
descendant reached through that private path remain absent externally even
when those bindings belong to `ExportRetentionClosure`.

The retention name is deliberate: membership means that an export root keeps
the binding in the graph considered for interface construction. It does not by
itself mean that the binding is externally exported. `Σ_export` is the external
candidate set.

Admission does not select or filter individual overloads. Within an admitted
binding's complete overload set, every resolved candidate enters `Σ_export`
with the same identity, pair, and mode. A concrete consumer then performs the
ordinary sequence:

```text
candidate from Σ_export
  -> CallSitePolicyDemandFormation
  -> ordinary Policy overload and CapabilityRealization selection
  -> unique executable selected invocation (or typed delete rejection)
  -> form DynamicLegality_Γ_consumer for the selected invocation
  -> accept or reject without reopening the candidate set
```

Const and mut coordinates may be independently defaulted, deleted, or
given a custom realization by that consumer family. No candidate is included
or excluded from the stable namespace view merely because of its mode or a
future caller's demand.

Ancestors and descendants admitted by the final external-exposure check need
not be export roots and may have used `P1Projection::Infer`; their resolved
candidate pairs are projected in exactly the same way.
`NamespaceDeclarationPolicy.external_projection` is only an early
direct-root validation/preview; `None` on a non-root declaration does not mean
that the eventual namespace export view lacks that declaration.

The symbol-level diagnostic carrier preserves admission facts and distinguishes an unresolved
name, a name outside the export-retention closure, and a private path; ordinary
consumer Policy-selection or dynamic-legality failure occurs only after stable
external lookup.

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

## 10. Ordinary world observation

Name resolution uses committed identity and source-established internal or
external views. Export retention and public/private reachability determine
external membership. Existence supplies no separate scan/evaluation privilege.
Operations check ordinary access, Policy and dependency premises.

## 11. Evaluation frontier

E evaluates ready actions in the current continuation. CompileInstance identity,
visibility, Ready and legality remain independent. Unavailable concrete runtime
input may retain an actual unfinished compile continuation, materialized by
a callable reify P2 boundary with independently closed return classifier. Selected calls retain
origin; newly formed runtime calls first select at the actual frontier. Region
uses current NamePosition without future-path/optimization prerequisites.

## 12. Unified binding and overload selection

All call-candidate entrances use one selection trunk. The prefix below shows
its named-type case:

Resolve once to a binding and read its ordinary resident. Values project
through their exact complete type's associated (). An explicit OverloadGroup
supplies its ordinary candidate algebra. Pre-C0 family filtering retains its
separate position.

```text
C0 = Enumerate(ResolvedTarget, PreC0Filter)
Cvis = { (c,sigma,evidence) | c in C0 and R_vis(c,Omega,sigma) }
Cprepared = ordinary C_sigma(c) where required, otherwise c
A = FullyAdmissible(Cprepared, frame, total ResultPolicyDemand)
D = SuppressFallback(A)
Bp = MaxPolicyProduct(D, input modes, output demand, migration endpoints)
B = ordinary Pattern specificity and remaining declared filters(Bp)
Selected = Unique(B)
```

Round one R_vis solves producer visibility, position InputAdmissible,
projection configuration and required migration evidence without executing
speculative bodies or effects. If evidence is not ready, retain the unresolved
continuation. Round two uses the ordinary callable relation; compile
realizations are not a second dispatch language.

The total output demand is formed before maxima from candidate-independent
immediate-consumer facts. An unresolved outer candidate cannot send its
formal policy backward into a sealed inner call. Selection fixes
(c*,sigma*,frame); selected extraction, migration, delete, lifetime or dynamic
legality failure never reopens candidates. Distinct entry identities do not
collapse merely because their values/types normalize equally.

For each whole-slot PolicyMode comparison position:

```text
succ_const: const > mut
succ_mut:   mut > const
```

This order is a *preference* among candidates that are already fully admissible.
Being higher in the order never grants a capability, and being lower never
removes one: the order chooses between existing candidates and does not decide
whether a candidate exists. Nor does it propagate: the selected candidate's
PolicyMode describes that one slot edge and is not pushed into the argument's
other members (§1.1). Tied distinct candidates remain co-maximal and ambiguous.

Multiple positions form a product partial order: `f` dominates `g` iff `f` is
not worse at every participating position and is strictly better at at least
one. Crossed advantages remain incomparable. There is no score, exact-match
count, parameter weighting, lexicographic order, input-before-output rule, or
separate conversion rank. Every call accounts for its formed OutputModeDemand;
only a resolved concrete mode demand contributes a mode preference coordinate. Optional
target-result pair/type/rank/facet constraints participate only when supplied,
as hard admissibility in `A`; they are not the output-mode coordinate. The
separately total EvaluationStageContext constrains R_vis evidence. Stage
completion creates no hidden mode constraint.

Preference and capability are independent. A finite representation is only the
derived expansion of a general relational declaration in the current domain:

    input          const   mut
    output const   C<-C    C<-M
    output mut     M<-C    M<-M

Cells may independently be default, delete, custom or absent. The four-cell
view is not a semantic primitive, required source inventory or execution proof.
Formal-local holes, correlation and require retain ordinary Pattern semantics.
Selection never derives capability or Writable from preference.

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

Source wiring of fallback remains pending. Its canonical semantics applies inside `D` after full
admissibility and before `Bp'`: any admissible non-fallback member, including
`delete`, permanently removes fallback. A distinct call-site candidate-family
annotation acts before candidate generation; only that position is closed, not
its syntax or selector algebra. This future behavior is not B6
named-strategy execution, and later delete/lowering/lifetime failure cannot
reopen fallback.

## 13. Lifetime boundary

`@` is an ordinary continuation-relative name-reification operation with its own overload groups, owned
by `../lifetime/lifetime-policy-and-overload-boundary.md`. It is not a policy
atom in the stage dimension of §1, and lifetime observation is a continuation projection.

The only boundary this document asserts is directional: ordinary overload
selection must already have produced one unique candidate, and lifetime rules
validate that result without replacing it. Lifetime checking may reject a
program; it may not reselect a call, reopen type/policy overload resolution, or
introduce a competing specificity order.

This is a restriction on lifetime *rules*, not a denial that `@` has overloads.
`@` is resolved by the ordinary selection trunk of §12 like any other operation.

## 14. Migration comparator boundary

Policy migration compares ordinary Bp coordinates and input/output endpoint
Policy through one product/Pareto order. Endpoint maxima are not a sequentially
composable Bp filter:

```text
Max(Product(Bp, input endpoint, output endpoint))
  != MaxEndpoint(MaxBp(...))
  != MaxBp(MaxEndpoint(...))
```

Ordinary Bp coordinates and both migration endpoint coordinates are composed
in one comparator before taking maxima. Migration does not add output-type
preference to ordinary type overload selection or define a B6 strategy.
Candidate Policy adaptation intersects typed Policy domains directly,
including stage, Pp, and present/optional/absent alternatives; it does not
fabricate a concrete `Some(value)` to reuse result-entry projection.
Migration-candidate PolicyMode is deliberately excluded from that hard
intersection and instead reuses ordinary actual-relative Bp preference.
Migration is reached only after the complete ordinary projection is empty and
the original demand admits a constructible target. Candidate enumeration,
hard applicability, Policy preference, unique selection, DynamicLegality, and
execution use the same ordinary invocation boundary as source calls.

## 15. Deliberately unfrozen

This document does not freeze:

- the final source token for `AbsentValuePattern`;
- concrete LifeName/Region/Color IR, lifetime-checker integration, summary
  compression, access-tree integration, and extended Horae logic;
- arbitrary clause-level Boolean policy logic;
- a complete runtime reflection API;
- export reopening syntax;
- unrelated `?`, `*=`, or Pattern-coordinate mechanisms.
