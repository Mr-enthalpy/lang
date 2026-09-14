# name binding Policy and Compile-Flow Projection

Status: canonical design contract. The typed model in this document is the
normative policy algebra.

This document owns the complete chain:

```text
source policy syntax
  -> contextual elaboration
  -> PolicyPair
  -> binding resolution
  -> phase-slice exposure
  -> binding/overload selection
  -> OpenStatic evaluation
  -> SealStatic evaluation
  -> Runtime binding and evaluation
```

## 1. Complete binding flow and policy pair

The ordinary slot/view laws below apply where a name and its resident value are
distinct. P1 also admits `meta` for the invocation instance whose name is its
own type value (§3.0). For that instance OpenHere governs mut qualification;
ordinary const/mut modes do not supply a second independent gate. This marker
is contextual P1 policy, not another atom of ordinary PolicyMode's 3×3 tables.
P2 meta continues to specify the callable's evaluation stage.

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
remain their existing three-point relations.

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
3×3 cell is realized and may therefore be retained with a candidate snapshot.
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
policy of a result is always a pair:

```text
Π = Pv:Pp

Pv  policy of the value component observed at this edge
Pp  policy of the Pattern/anonymous-type component observed at this edge
```

There is no scalar replacement for this pair, no third `Pv`/`Pp` component, and
no independent complete P3 Policy product. Parameter and return positions do
nevertheless have position Policies: `P_in` overlays P2 and `P_out` overlays
P1. Their inherited pair/stage coordinates remain fixed while their orthogonal
whole-slot `PolicyMode` may be explicitly refined. A result object carries its
own `PolicyPair` when it re-enters the flow.

The pair is an observation edge, but its two axes are constrained by whether the
object actually has an independent value projection. This constraint does not
constrain the whole-slot PolicyMode coordinate:

```text
Val1?(x) = null  =>  Pv = Pp

Pv != Pp  =>  Val1?(x) != null
          and runtime ∈ Stage(Pv)

Pv = absent  does not imply  Val1?(x) = null

PolicyMode_Γ(slot) ∈ { const, plain, mut }
PolicyMode_Γ(slot) is independent of Val1?(x), Pv, and Pp
```

The first rule does **not** say `Pv = absent`: a pure PatternValue still has one
Policy, observed identically as `Pv` and `Pp`. Divergence becomes meaningful only
when an independent runtime value projection exists. The final rule preserves
observer hiding: an edge may suppress the value projection of an object that
does carry `Val1`. Object shape is therefore not inferred back from an
observation, while an impossible two-policy split is not invented for a pure
PatternValue.

The whole-slot separation is a semantic invariant:

```text
PolicyModeOrthogonalToObjectShape:

Val1?(x) = null
  =/> PolicyMode_Γ(slot) = const
  =/> PolicyMode_Γ(slot) = plain
  =/> PolicyMode_Γ(slot) = mut
```

`Pv = Pp` says only that the value-side and Pattern-side stage/exposure facts
cannot split for a pure Object. It does not erase the PolicyMode of the binding,
formal, argument, or result slot that carries that Object. The same pure value
may therefore occupy const, plain, and mut slots without changing its Object
identity or introducing a fourth Object component.

Policy dimensions are typed and orthogonal:

```text
pair/view stage              meta / compile / seal / runtime
pair/view presence           present / optional / absent
whole-slot PolicyMode        const / plain / mut
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

`const let` / `plain let` / `mut let` on a formal parameter are first an overload
preference coordinate (the `succ_const` / `succ_mut` / `succ_plain` partial
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
`symbol-first` §4.5.1 and `type-values` §5.1.3 reference exactly this rule
rather than redefining a second PolicyMode system).

### 1.1.1 Initializing a typed Place before a resident exists

    DeclaredPolicy(name) independent_of InitialInitializationAuthority(place)

A typed Uninitialized Place has a declared type and pending initialization
authority from its authorized formation, but no resident Object to observe.
PolicyView(slot,x), P(x), and resident compatibility are not evaluated for a
missing x. The declaration's const/plain/mut mode determines the initialized
name's views; it neither creates nor removes the separate first-write authority.

The ordinary explicit initial-borrow realization uses that live authority and
actual Place/access/lifetime checks to supply a reference capability for the
first write. It can supply a mut T ref view for a const-declared name; the
reference view's mode and the target name's declared mode are distinct. The
existing const/ plain reference delete cells remain delete. Selecting a mut
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
Pin=Overlay(P2,Mode=rho(p)); inherited-only coordinates retain P2.
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

The three symmetric Policy preference points do not require three symmetric
global reconstruction operators:

```text
NoPlainReconstructorRequirement:

PolicyMode = {const, plain, mut}
  !=>
source language exposes one global reconstructor per mode

global const / mut
  = explicit ordinary reconstruction operations

plain materialization
  = normally a plain destination plus ordinary move/copy mechanical passing
```

There is no language requirement for a global `val plain` dispatcher. This is
compatible with a full 3x3 capability space: selection coordinates and the
surface inventory of explicit reconstruction operations are different facts.
The normative action meaning is owned by
[`CanonicalMechanicalPassCore`](../mechanical-lowering/mechanical-argument-passing-and-move-fixed-point.md#0-canonical-pass-action-core):
copy performs `CopyConstruct` followed by one terminal Move, with no pre-move
of the source. `CopyConstruct` is the compact name for the selected ordinary
copy algebra: ordinary `T` expands through share/clone, while `T ref` and
`T share` expand through rebind/clone. It is not a new opaque primitive.

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

`true` and `false` are ordinary bindings, not aliases. Each is a fresh binding
with a fresh place holding a copy of the value read through the source path:

```text
NameBindingId(true) ≠ NameBindingId(if::bool)
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
spelling for `AbsentValuePattern` remains Open. Implementation fixtures use
`S`; that fixture spelling does not freeze the public surface.

Elaboration assigns atoms to typed coordinates. `const`, `plain`, and `mut`
are the three atoms of the whole-slot PolicyMode pattern; none is stored inside
`Pv` or `Pp`. Bare let contributes no explicit override. Written plain constrains the plain
point; an explicit HoleBinderId is a third, distinct case solved by ordinary
Pattern extraction. Default completion is separate from written constraints. A written choice such as
`const || plain`, `plain || mut`, or `const || mut` is not a legal whole-slot
mode demand. In particular, `const || mut` is not a neutral whole-slot mode and
does not elaborate through the general PolicyChoice syntax.

For ordinary slot policies, surface elaboration factors that whole-slot
coordinate before building the pair. P1 meta-instance policy is recognized by
its position under §3.0 before applying this ordinary-slot factorization:

```text
PolicySurfaceElaboration(surface)
  -> FactorWholeSlotMode(surface)
  -> <ModePattern?, PairSurface?>
  -> <ModePattern?, PairSpec = Pv:Pp | InferPair>

ModeAtom
  ::= const
   |  plain
   |  mut

ModePattern
  ::= ModeAtom
   |  HoleRef(HoleBinderId)
```

Each solved ModePattern denotes one whole-slot point; holes may have several
solutions before applicability and unique selection. There is no
set-lifted PolicyMode demand and no second neutral element beside `plain`. It
may not mix a stage, visibility, presence, or pair atom into that coordinate.
`plain` therefore always factors as `ModePattern(plain)` when written; it can
never remain as a residual `PolicyAtom` for `Pv` or `Pp`.

In a result-demand context, omission records NoWrittenModeConstraint. Existing
inherited/contextual constraints apply first; only a context requiring default
completion with no such constraint may complete the mode to plain. Thus
compile || runtime let e retains its stage choice without spelling a mode.

`FactorWholeSlotMode` walks the complete `PolicySpec`, extracts one connected
Mode Pattern once, and removes those atoms before either colon side is
elaborated. The residual `PairSurface`, if present, contains only pair/view
coordinates. The closed semantic well-formedness rules are:

```text
AtMostOneWholeSlotModePattern
NoPolicyModeCoordinateInPv
NoPolicyModeCoordinateInPp
NoIndependentModePatternsAcrossColon
NoMultiPointPolicyModeChoice

no residual pair/view atom
  => PairSpec = InferPair
```

Thus `const`, `plain`, and `mut` alone are singleton whole-slot Mode Patterns
with inferred pair. `const + runtime : compile` parses as
`(const + runtime):compile` and factors to mode `const` plus pair
`runtime:compile`. A surface PolicyChoice containing more than one ModeAtom is
preserved by Raw/Normalized syntax but rejected by typed Policy elaboration.

How a written colon whose factorization leaves an empty residual side is handled
is a separate surface decision, not a theorem of PolicyMode orthogonality. The
current parser rule rejects `const:compile`, `runtime:const`, and `const:mut`;
the Open surface question may instead
define an unambiguous contextual shorthand while still satisfying the closed
coordinate rules above. No semantic elaborator may place a mode atom in `Pv` or
`Pp`, regardless of which surface completion is selected. Independently,
`const || mut:compile` is invalid because its mode choice violates
`NoMultiPointPolicyModeChoice`, not because of the current empty-side rule. This
factorization does not require the frozen Raw/Normalized Policy AST carrier to
change.

### 2.2 Algebra

`||` selects alternatives within one dimension:

```text
runtime || compile
meta || compile
runtime || S
```

It is not arbitrary clause-level Boolean disjunction. These are invalid:

```text
runtime || const
runtime || plain
const || plain
plain || mut
const || mut
const || plain || mut
compile || public
mut || export
(const + runtime) || (mut + compile)
```

`+` combines different dimensions:

```text
const + runtime
plain + runtime
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

### 2.3 Deduction is ordinary operator Pattern extraction

Policy + and || consume the corresponding registered operator relations under
type. Their typed coordinate restrictions remain hard applicability rules.

    R_+(h1,h2,p,rho)
    R_||(h1,h2,p,rho)
    independent holes -> distinct HoleBinderId
    repeated hole -> shared identity/equality constraint
    require C -> {rho in Solutions | C(rho)}

There is no independent policy deduction calculus:

    P1, P2 independent
    Pin_i(rho) = Overlay(P2, Delta_in_i(rho))
    Pout(rho) = Overlay(P1, Delta_out(rho))
    Omega = <Actual_1,...,Actual_n,Demand?>

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
position-specific elaborators. P1 meta-instance policy has the following
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
not separate inference ontologies. Plain is an atom, not a hole or omission.
Declaration and call contexts supply observations of the same joint relation.

### 3.0 Meta-instance P1 policy

The contextual openness qualification has the narrow domain `type` and
`type ref`: `meta type` and `meta type ref` are admitted, not arbitrary
`meta X ref`. This is not a fourth PolicyMode point. Qualifying an ordinary
type preserves its existing opening subject; it does not manufacture a meta
invocation identity, a construction window, or a writable Place. P2 `meta`
continues to denote evaluation stage.

For initialized type names, both direct `NameExpr -> mut type ref` and explicit
`NameExpr -> meta type ref -> mut type ref` remain available ordinary routes.
A meta ref records its actual Place, borrowed generation and OpeningSubject;
its writable candidates require current OpenHere of that subject, independent
Writable of the target, and the selected capability/access/type/lifetime checks.
It contains no permanent writable proof. `ConfirmMut` is an explicit ordinary
candidate that confirms these facts, preserving target/generation and never
amplifying capability. Where both routes are legal at the same continuation
position they have the same target and realizable mut capability. Coherence
does not authorize implicit chaining, candidate retry, or reopen.

Close defeats direct mut acquisition, meta-to-mut confirmation and later
writes through saved refs. A replacement resident does not retarget a saved
opening subject. An uninitialized typed Place instead uses the separate
one-shot initial-borrow/write rules; `InitialTypeSlotRef` is not `meta type ref`.
The [type/ref owner](type-values-places-and-borrow-views.md#522-initialized-type-names-meta-references-and-mut-confirmation)
gives the precise judgments. The instance-specific retention behavior is:

```lang
meta let f = expression;
plain let g = expression;
```

When expression denotes an ordinary meta instance, P1 meta retains it. Its
instance name is its own type value tau_M; the narrow policy domain does not
broaden the direct result class to arbitrary payloads or external types.
In this policy, OpenHere is the governing mutation qualification:

```text
MetaInstanceMutationQualification(f, Sigma) iff OpenHere(f, Sigma)
AcquireMutView(f, Sigma) requires OpenHere(f, Sigma)
```

Unlike an ordinary slot containing a distinct value, there are not independent
instance const/mut and value openness coordinates to combine. OpenHere is the
stronger fact: first establish it, then acquire the mut view. Such a view is
still explicit and its write Pre rechecks the current window. The selected
operation must exist and satisfy independent target Writable and ordinary type,
access, capability and lifetime
rules; meta does not synthesize missing operations or make expired targets live.

The marker retains the invocation's dependency-bounded source through completion.
Explicit plain let, or a context separately completed to plain, selects the classic
complete-and-close meta invocation. This closes the instance, not external input
subjects or external borrow targets. A later meta let of the same cached instance
cannot reopen it. A temporary non-OpenHere context is not itself Close.

This is a P1 policy selection, distinct from P2 meta evaluation stage. It is not
ordinary mut, a choice between const/mut, a new ordinary mode-preference point,
or an implicit request to change evaluation stage. Incompatible ordinary-mode
and meta-instance demands cannot be combined to create a stronger authority.
No new parser shape or Object coordinate is introduced.

Ordinary Val2 payload names still carry distinct values. Their const/plain/mut
policies, result demand, migration, and borrowing remain ordinary. Meta invocation
itself directly returns CompleteType; payload navigation and ordinary compile
extraction, not another meta result class, expose other values. The
[invocation owner](../meta-invocation/meta-object-invocation-and-policy-reduction.md)
defines instance identity, openness meet, completion and cache behavior.

### 3.1 Ordinary binding projection

P1 let x = expr is a complete lexical binding. Omission contributes no override,
not a request to rewrite the producer's mode to plain. Destination type inference
and policy completion are distinct ordinary relations.

```text
OrdinaryBindingElaboration(prefix, expr, destination):
  kappa := CurrentEvaluationPhase
  written := WrittenPolicyConstraints(prefix)
  // no mode atom -> NoWrittenModeConstraint, not WrittenModeDemand(plain)
  demand := CallSitePolicyDemandFormation(binding_context, written)
  // inherited/contextual constraints first; required defaults second
  R := ResolveAndEvaluate(expr, kappa, demand)  // demand precedes root maxima
  mu_produced := ResultPolicyMode(SelectedCandidate(R))  // frozen
  PairView(destination) := ElabP1(demand.pair_query, R)
  mu_destination := CompleteDestinationMode(written, inherited_context, R)
  mechanical_pass := SelectMechanicalPass(PairView(destination), destination)
  TransferToDestination(
    source = PairView(destination), produced_mode = mu_produced,
    destination, destination_mode = mu_destination, mechanical_pass)
```

A completed destination has one concrete mode. Completion uses ordinary
inheritance/context and, where needed and otherwise unconstrained, the default
plain point. Written plain is an explicit source constraint; omitted mode is
not. Producer and destination remain separate: transfer never rewrites the
selected producer's mode. Pair-view projection and mechanical move/copy retain
their existing rules, with no implicit ref or policy cast.

SelectMechanicalPass preserves an explicit pass or uses the separately specified
automatic move/copy consumer of CanonicalMechanicalPassCore. A const producer
may win under an explicit plain output preference without becoming plain itself;
ordinary transfer installs its result in the independently completed destination.
For an existing source that must be preserved, the explicit copy trace remains:

    plain let y = x copy
      -> tmp := CopyConstruct(x)
           ~= share -> clone   for ordinary T
           ~= rebind -> clone  for T ref / T share
      -> Move(tmp) -> y with PolicyMode=plain

There is no move of x before CopyConstruct and no implicit policy conversion.
Changing the omission rule changes neither that mechanical core nor the sealed
producer/destination distinction.

    WrittenModeDemand(mu)
    NoWrittenModeConstraint
    ExplicitModeHole(h)

These are distinct elaboration inputs, not new PolicyMode points. A hole is
solved through the ordinary candidate relation; a missing atom is no hole.

    DefaultModeCompletion applies only when the context requires completion
      and no written/inherited/contextual constraint already supplies the mode.
    DefaultStageCompletion obeys existing one-way stage admissibility.
    Neither is a general P1-from-P2 or P2-from-P1 semantic deduction.

The current phase always constrains evaluation. Candidate-local stage
completion may use the established lift when the relevant stage is omitted;
explicit P1 is never overwritten. ImplicitEvaluationP1StageView names exposure
of the resulting candidate view in kappa, not another outward authority.

```text
CallLocalPolicyClosure:
  1. form ResultPolicyDemand(c), including omission/completion state, before maxima(c)
  2. use only already available candidate-independent immediate-consumer constraints
  3. never use an unresolved outer candidate's formal policy as inner demand
  4. use plain only through an applicable DefaultModeCompletion;
     omission itself imposes no plain constraint
  5. after unique selection, ResultPolicyMode(c)=mu_c is frozen
  6. the outer call consumes that result and never reopens c

EvaluationStageContext(c) = kappa
OutputModeDemand(c) = written/inherited/contextual mode demand
                   | separately completed mode demand
                   | NoWrittenModeConstraint
TargetResultConstraint(c) = optional expected pair/type/rank constraints
```

Demand formation is total: absence is recorded rather than replaced by a
fabricated written atom. A resolved concrete mode contributes its preference
coordinate; no constraint contributes no invented preference. Supplied hard
result constraints participate in applicability. Completion cannot defer a
known consumer constraint until after root-call selection.

In schematic g(f()), f closes under its own immediate context. If that context
requires otherwise-unconstrained completion, it uses plain through
DefaultModeCompletion. Then g consumes the frozen mu_f and its independently
formed demand; it cannot change f's solution.

### 3.1.1 Explicit expression result-Policy context

An expression may override or delimit the default evaluation result context
with an explicit candidate-independent result demand:

```text
PolicyLetExpression ::= PolicySpec "let" PipeExpression
```

`PolicySpec` is the existing typed Policy grammar, not an ordinary value
expression. The operand covers the complete following pipe; parentheses close
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
plain constraint; typed coordinate legality still applies to written Patterns. Any
residual `Pv:Pp` choice, including `compile || runtime`, remains part of `pi`.

Producer preference and outward acceptance are different relations:

```text
ProducerPreferredUnder(mu_demand, mu_candidate)
  = preference by succ_mu_demand

ExistingOutwardModeAccepted(mu_demand, mu_result)
  iff mu_result = mu_demand
```

Thus a `const` producer may uniquely win under a `plain` output preference, but
its `const` result is not already an outward singleton-`plain` view. The
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

Singleton `plain` has a closed ordinary realization without a global
reconstructor:

```text
PlainPolicyLetResultTransfer:

ProducedMode(r) = mu_r
sigma_plain = ExpressionResultSlot(plain let e)
PolicyMode(sigma_plain) = plain

S := SourcePolicy(r)
T := PolicyOf(sigma_plain)
C := PreparePolicyMigrationCandidates(S, T, ResultPolicyDemand)
m_plain := Unique(PolicyOverload(C, PolicyMigrationDemand(S, T)))

PolicyProjection(m_plain, r, sigma_plain) = rho_plain
ValueRealization(m_plain, r, sigma_plain)
  = TransferToExpressionResult(r, sigma_plain, move | copy)

CoherentPolicyMigrationResult(m_plain, rho_plain, move | copy)
  uses CanonicalMechanicalPassCore
  preserves ProducedMode(r) = mu_r
  exposes the transferred result through sigma_plain
```

A fresh consumable producer result may use terminal `Move`. A source that must
be preserved requires the ordinary `CopyConstruct` plus terminal `Move`
realization. The selected migration candidate gives both the plain Policy
projection and this value realization. If no such candidate is uniquely
selected, outward completion fails after the producer is frozen; that failure
does not erase `sigma`, expose the producer's wrong mode, or reopen producer
selection. This is why no global `val plain` dispatcher is required.

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
of §1.2. `plain` satisfaction does not imply a global `val plain` dispatcher.

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

It must not return the original `compile || runtime` entry. name binding identity and
Pattern identity do not change; only the visible slice is cropped.

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

For pair query `Qv:Qp`, result-view satisfaction slices the Pattern-policy stage
capability before migration candidate enumeration:

```text
Pp_selected = SlicePatternPolicyStages(Qp, source.Pp)
```

This is Policy slicing over `Pp`; it is not Pattern extraction, PatternValue
projection, postfix `?`, extractor lookup, Pattern-root navigation, or a change
of PatternRoot/PatternScope. It preserves PatternValue identity and structural
Pattern shape.

Unselected alternatives in a written query are never obligations to
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

Pin and Pout are symmetric derived positions with independent parents:

    Pin = Overlay(P2, Delta_in)
    Pout = Overlay(P1, Delta_out)
    bare let -> Delta = empty
    written atom -> explicit override
    <p> p let -> mode overlay containing the formal-local HoleBinderId p

    let x          -> Pin=P2
    plain let x    -> Overlay(P2, Mode=plain)
    const let x    -> Overlay(P2, Mode=const)
    mut let x      -> Overlay(P2, Mode=mut)
    <p> p let x    -> Overlay(P2, Mode=rho(p))

Mode may be overridden; stage and other inherited-only coordinates remain
P2's. Omission, explicit concrete mode and explicit hole are not interchangeable.
Visibility/export do not acquire invented formal/output counterparts.
SafetyPolicy retains its own independent consumer meaning.

The selected PolicyMode is a formal preference input. It is not an ordinary P1
query applied to the actual argument. Consequently an oppositely qualified
actual is not removed before the product order. The three context-indexed
relations are:

```text
succ_const: const > plain > mut
succ_mut:   mut > plain > const
succ_plain: plain > const = mut
```

The equality in `succ_plain` is semantic: if the fully admissible set contains
one `const` and one `mut` candidate but no `plain` candidate, both are
co-maximal and selection is ambiguous. An implementation may not choose either
one arbitrarily or use declaration order to break the tie.

The elaborated formal view is not body-local policy metadata. Candidate
formation exports its whole-slot PolicyMode into the callable's parameter
Policy product position:

```text
FormalPolicyMode(parameter)
  -> Candidate.parameter_policy[position]
  -> MaxPolicyProduct
```

Thus P2 still governs the pair visible inside the body, while the whole-slot
mode participates in comparison against other fully admissible overloads.
Implementations must not collapse `plain` back into an unspecified carrier.

#### 3.2.1 Return policy refinement inherits P1

There is no independent complete `P3` Policy product. A return position has a
position Policy `P_out` formed from the callable declaration P1 plus an
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
return plain let x  -> inherited P1, PolicyMode = plain
return const let x  -> inherited P1, PolicyMode = const
return mut let x    -> inherited P1, PolicyMode = mut
```

The mode may not alter stage, value presence, Pattern policy, ordinary
visibility, or export-root status. Policy dimensions are not replace-all: each
dimension must be classified as `InheritedOnly` or `Overridable`; evaluation
stage is `InheritedOnly` here and whole-slot mode is `Overridable`. “No P3”
therefore means that the return site has no third arbitrary complete Policy
vector; it does not mean that the return position has no Policy.

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
  => value stages = ∅
```

The review matrix is therefore complete rather than shape-dependent:

| Observed Val1 | const | plain | mut |
|---|---:|---:|---:|
| present | valid mode coordinate | valid mode coordinate | valid mode coordinate |
| absent | valid mode coordinate | valid mode coordinate | valid mode coordinate |

The cells assert only that the mode coordinate exists. They do not manufacture
a value stage or any operation capability.

This is a statement about the edge, not about the object behind it. Per §1,
`Pv = absent` does not assert `Val1?(x) = null`; when `Val1?(x) = null`, the
canonical unhidden observation instead has `Pv = Pp`.

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
`succ_const` / `succ_plain` / `succ_mut` relations rank producer candidates;
they do not widen the set of concrete modes accepted by outward satisfaction.
In particular, a `const` producer that wins under `plain` preference is not an
existing singleton-`plain` outward view.

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
  => name binding / TypeValue / PatternValue / Place identity is unchanged
```

This is the **Existing-First, Constructible-Second** principle:

```text
1. existing accepted views
2. language-constructible accepted views
```

The current set of stage branches admitted for non-identity construction is
exactly `{ runtime }`. Construction does not mean every alternative in `Q`
becomes an obligation.
The original query may be `meta || runtime`; if its complete existing
projection is empty, the derived migration target is only its runtime branch.

```text
OnePolicyMigrationAlgebra:

ordinary binding Policy completion
PolicyLet outward completion
compile:compile -> runtime:compile materialization
  -> PreparePolicyMigrationCandidates
  -> ordinary Policy overload / unique selection
  -> selected m
  -> PolicyProjection(m) × ValueRealization(m)
  -> CoherentPolicyMigrationResult
```

Demand kinds restrict which direct candidates are admitted; they do not own
different selectors. The runtime-stage case admits the one authorized atomic
runtime-migration family described in §3.5. A PolicyLet result-slot mode
transfer admits the corresponding identity or canonical mechanical
move/copy-backed migration. Ordinary binding consumes the same candidate
algebra. No demand kind may reinterpret an arbitrary ordinary value call as a
Policy transition merely because its return value has a useful shape.

`BindingP1Demand` uses the exact conservative `ProjectP1` theorem in §3.1.
Formal parameter and result consumers retain their existing policy-Pattern and
applicability rules. A demand that accepts `compile || runtime` is satisfied by
an available compile slice; the mere spelling of `runtime` as another accepted
alternative creates no materialization obligation.

`MechanicalPolicyDemand` records the origin of a language-selected mechanical
realization within a selected migration. It does not imply that arbitrary
Policy failure may search `ref`, `share`, `@`, or another structure-changing
operation. Those operations occur only when separately required by their own
language rule and then use ordinary function-object invocation.

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
manufactured, and Type is unchanged. The whole-slot input/output PolicyModes
belong to the selected ordinary callable. In particular:

```text
PolicyMode(input_slot)
PolicyMode(output_slot)
```

need not be equal. A callable may declare `const + compile -> mut + runtime`
because it constructs a fresh runtime object; the compiler does not infer or
invent that `mut` capability. The declared input/output coordinates participate
in ordinary Bp' comparison. Opposite const/mut endpoint Patterns are not
removed by a hard Policy-domain intersection. They reuse ordinary
actual-relative preference:

```text
const actual/demand: const > plain > mut
mut actual/demand:   mut > plain > const
plain demand:        plain > const = mut
```

Stage, presence, Pp capability, Type, and structural applicability remain hard
endpoint conditions. PolicyMode is a preference coordinate, not a structural
repair and not a capability intersection. In the plain-demand row, equal
maximal `const` and `mut` endpoints remain ambiguous when no `plain` endpoint
survives.

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
consumer requires runtime. This is `NoImplicitBorrowFormation`: explicit `ref`,
`share`, or `@` remains an independent ordinary operation, never candidate or
Policy repair.

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
Resolve
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

name binding/path/callable identity and ordinary overload selection occur in the
static semantic world. A runtime continuation does not reopen namespace
lookup, name binding identity, callable identity, or the overload candidate set
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
  applicability. A `compile` function that calls pure `extend` on a transported
  type, or place-level `inject` through a ref, may be legal or illegal for the
  same normalized contents in different stacks:

  ```text
  Eval(F, t; Γ_open)  ≠  Eval(F, t; Γ_closed)
  ```

  `extend` requires `OpenHere_Σ(value)`; `inject` independently also requires
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

## 5. Function-object default stage completion

Only when P1 stage material is omitted and the context requires completion,
the existing permitted default from P2 = P2v:P2p lifts stages:

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

The evaluator uses this stage lift candidate-locally as its default result
stage context. Because `CurrentEvaluationPhase` is already fixed, ordinary
`compile`/`runtime` exposure is known from `P2` without an explicit
`PolicyLet`. This evaluation default does not mutate the declaration's
canonical P1, add an outward authority, or manufacture an absent pair slice;
it selects/exposes the P1 stage view admitted by the current phase. An explicit
result Policy remains available when the programmer wants a narrower stage
boundary or a migration target.

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

the source supplies no written mode constraint. An otherwise unconstrained
ordinary materialization context may separately complete it to plain. That
completed atom is neither an inference hole nor source omission. An explicitly
written `const let` or `mut let` selects the corresponding concrete mode. P2
stage/exposure facts never manufacture or propagate a PolicyMode during stage
lifting, and export does not silently replace the internal mode with const.

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
Resolve(path)
ExposePolicySlice(binding, phase)
ReadValue(slice)
ReadPattern(slice)
EnumerateValueFacet(slice)
EnterCallableBody(candidate)
```

Failure to expose a value slice is not an unresolved binding. In particular:

```text
Pv = runtime
Pp = compile
```

has this OpenStatic behavior:

```text
binding/path resolves
runtime value is unreadable
compile Pattern/type is readable
derived compile companion (CompilePartner(F) = C(F)) may join static overload resolution
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

The runtime continuation consumes the preserved binding/callable identities. It
does not repeat path resolution or overload choice merely because the value
becomes readable later.

Seal-only bindings follow the same ordinary-binding rule. Their paths can be
resolved independently of whether a facet is exposed in the current phase.

## 8. Mechanical compile-flow projection

StaticFlow and RuntimeResidualFlow are observations of one semantic
continuation, not independent semantic passes or separate IR worlds.

    complete continuation -> static observation
                          -> runtime residual observation

Pattern/type work, ready calls, derived companions and deferred seal work retain
their ordinary stage rules. Runtime residue retains value computations, runtime
bodies/effects and branch value selection. D/Done and other control structures
are projections of the same actions and positions, not separately interpreted
copies.

Projection alone executes no call and selects no overload. E exhausts ready
work under the current continuation, stage, policy and facts; E E = E.
It does not rewrite runtime bindings into compile bindings to obtain more work.
[E and optimizer boundaries](../meta-invocation/evaluation-residual-and-optimization.md)
govern transformations and revalidation by all affected projections.

## 9. Namespace visibility and export

### 9.1 Three independent binding views

Namespace resolution, external exposure, and compilation-world membership are
different questions:

```text
Σ_full(N)    complete namespace-internal binding/overload set
Σ_export(N)  externally exposed projection of that set
Wfinal       Wpre ∪ Wseal, the bindings materialized or retained this build
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
a second binding universe. Consequently:

```text
s in Wpre  does not imply s is exported
s in Wseal does not imply s is exported
s is exported does not imply s was itself an export root
```

Explicit navigation is authority-sensitive. Internal explicit navigation may
reach the complete namespace-internal view. External explicit navigation is
restricted to the export projection. Explicit-path success alone therefore
does not prove export membership.

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
`P1Projection::ValueDominant` does not yet carry the associated `Pp`; neither
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

Const, plain, and mut coordinates may be independently defaulted, deleted, or
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

## 10. Wpre and seal world

Immediately before SealStatic, compute the least semantic materialization
closure:

```text
R0 = ExportedBindings
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

Only a compiler-known privileged seal function may enumerate the binding world,
and its fixed scan domain is `Wpre`. Adding `Wseal` never expands that domain.
Ordinary seal policy grants no scanning capability.

Explicit lookup is separate:

```text
ResolveExplicitPath != EnumerateBindingWorld
```

A committed binding in Wseal can be explicitly resolved by later seal/compile
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
projections of runtime bindings, and derived compile companions. Meta and compile
callables may invoke one another in one evaluator. Their return ontologies differ
in both root authority and the admissible direct result class:

```text
ordinary meta
        -> constructs its instance name/type tau_M (CompleteType)
           P1 meta retains dependency-derived openness; plain completes/closes
compile -> any declared ordinary semantic value across result classes
           (PatternValue, complete type value tau, type ref/share borrow
           instance); root-conserving, with no root authority
privileged builtin
        -> follows its member-declared result and owner rules
```

An ordinary meta callable directly returns its instance type tau_M. Arbitrary
values, external types, groups and valid borrows can be Val2 payloads, with
ordinary member observation and policy. They cannot replace tau_M as the direct
meta result. P1 meta qualification follows OpenHere (§3.0). See the
[invocation owner](../meta-invocation/meta-object-invocation-and-policy-reduction.md).
`compile` may return a complete type
value `tau` (participating in Pattern observation through `Core(tau)`, not
itself an ordinary PatternValue/Object), an OverloadGroup
value, `type ref`, or any other declared ordinary PatternValue. Privileged
builtins are member-specific: in this closure `struct -> tau`,
`extend -> type`, and `inject -> type ref`. The root conditions are owned by
`symbol-first-meta-construction-and-pattern-injection.md`.

No OpenStatic task may read a runtime value or depend on a runtime effect. If a
task is blocked only by a seal-only view, preserve its call node, Pattern
arguments, binding dependencies, and overload inputs as a `DeferredSealTask`.

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
results, and ordinary explicitly resolved bindings.

When otherwise equal and fully admissible:

```text
Vis(seal) ⊂ Vis(compile), therefore seal > compile in SealStatic
```

SealStatic is terminal for static work. Missing bindings/projections/companions,
runtime value/effect dependencies, or non-unique overload maxima are errors;
there is no later static deferral phase.

### 11.3 Runtime

Runtime consumes `RuntimeResidualFlow`, exposes runtime value slices, continues
the already resolved and sealed runtime invocations, executes runtime bodies/effects,
and performs runtime branch value selection. A derived compile companion never
replaces the real runtime call.

## 12. Unified binding and overload selection

All call-candidate entrances use one selection trunk. The prefix below shows
its named-type case:

```text
b = Resolve(path)
T = ReadNamedType(b)
C0 = CallCandidates(T)
C1 = ExposePhaseViews(
       C0,
       EvaluationStageContext(call),
       candidate_stage_view = StageLiftP2(P2(candidate)))
C2 = ProjectExpectedPolicy(C1, P1_or_expected_facet)
T? = TargetResultConstraint(call)
A  = FullyAdmissible(C2, argument_frame, T?)
M  = MaxPolicyAndOverloadOrder(
       A,
       argument PolicyMode coordinates,
       OutputModeDemand(call))
```

Resolve returns a structural NameBinding, not a callable carrier. ReadNamedType
reads its resident complete named type in this case. Ordinary Val2 navigation
may read a value of any type; it does not automatically run ReadNamedType or
register the resident for callability. A held OverloadGroup G, including one
read through state::instance, supplies CallCandidates(G) at C0. Ordinary function
values use their exact complete type's associated () entrance. See the
[navigation and projection example](../patterns-overload/overload-resolution-design.md#21-value-navigation-is-broader-than-candidate-projection).
Only repeated exposure of the same stable candidate-entry
identity may collapse; distinct contribution entries never deduplicate merely
because their values or types normalize equally.

Success requires exactly one maximal candidate. Failure can mean no exposed
slice, no fully admissible entry, multiple incomparable maxima, a unique delete
maximum, or an unfinished terminal SealStatic task.

C1 exposes candidate P1 under the current phase, using the permitted stage
completion only where omitted material requires it. `C2` applies an explicit expected projection when one exists;
its absence does not make stage policy unknown and does not require
`PolicyLet`.

For each whole-slot PolicyMode comparison position:

```text
succ_const: const > plain > mut
succ_mut:   mut > plain > const
succ_plain: plain > const = mut
```

This order is a *preference* among candidates that are already fully admissible.
Being higher in the order never grants a capability, and being lower never
removes one: the order chooses between existing candidates and does not decide
whether a candidate exists. Nor does it propagate: the selected candidate's
PolicyMode describes that one slot edge and is not pushed into the argument's
other members (§1.1). `const = mut` in `succ_plain` leaves two co-maximal
candidates and therefore an ambiguity if no `plain` candidate is available; it
never means “pick either”.

Multiple positions form a product partial order: `f` dominates `g` iff `f` is
not worse at every participating position and is strictly better at at least
one. Crossed advantages remain incomparable. There is no score, exact-match
count, parameter weighting, lexicographic order, input-before-output rule, or
separate conversion rank. Every call accounts for its formed OutputModeDemand;
only a resolved concrete mode demand contributes a mode preference coordinate. Optional
target-result pair/type/rank/facet constraints participate only when supplied,
as hard admissibility in `A`; they are not the output-mode coordinate. The
separately total EvaluationStageContext drives phase exposure in C1. Stage
completion creates no hidden mode constraint.

Preference and capability are separate relations. Relational declarations with
mode holes have the following finite 3×3 explanatory expansion:

```text
                  input
              const   plain   mut
output const    C<-C    C<-P    C<-M
output plain    P<-C    P<-P    P<-M
output mut      M<-C    M<-P    M<-M
```

Each solved coordinate may be default, delete, custom or absent. The table is
a derived view, not required source declarations or a primitive. One formal-local
<p> p let covers three inputs; shared/independent output holes and require
express correlations. A family need not admit all nine solutions. In
particular, a Policy preference may select a mut candidate whose requested
operation is deleted or whose target is not writable; capability facts never
flow backward into the Policy order.

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
declaration-policy stage is currently `D = A`. If a future fallback strategy is
introduced, its already-fixed semantics applies inside `D` after full
admissibility and before `Bp'`: any admissible non-fallback member, including
`delete`, permanently removes fallback. A distinct call-site candidate-family
annotation acts before candidate generation; only that position is closed, not
its syntax or selector algebra. This future behavior is not B6
named-strategy execution, and later delete/lowering/lifetime failure cannot
reopen fallback.

## 13. Lifetime boundary

`@` is an ordinary continuation-relative name-reification operation with its own overload groups, owned
by `../lifetime/lifetime-policy-and-overload-boundary.md`. It is not a policy
atom in the stage dimension of §1, and lifetime policy is not a fifth stage.

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
- future policy stages;
- arbitrary clause-level Boolean policy logic;
- a complete runtime reflection API;
- export reopening syntax;
- unrelated `?`, `inject`, or new PatternValue mechanisms.
