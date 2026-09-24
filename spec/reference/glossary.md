# Glossary

This glossary names current frontend and semantic concepts. Normative meaning
belongs to the public contract or canonical topic owner linked from
`spec/design/README.md`.

## Frontend

### Weak lexer

A lexer that emits structural tokens (`Name`, `Literal`, `Symbol`, trivia,
invalid, EOF) without assigning semantic roles to names.

### Raw AST

The syntax-preserving, recovery-capable tree produced by the parser. It records
source shape and diagnostics but performs no name resolution or semantic
classification.

### Normalized AST

The non-semantic tree produced by syntax-directed lowering. It unifies call,
product, sugar, declaration, and Pattern surface structures while preserving
their value/Pattern boundary. It is not HIR.

### Product

An ordinary structural value or Pattern carrier, distinct from OverloadGroup.
All directly named entries make its layer unordered; any bare entry makes that
layer ordered, independently of its top-level name and nested layers. A source Product participates in
the language's call-composition model; it is not a conventional argument-list
node.

### Pattern

Semantic structural material interpreted by the relational judgment
`R_Gamma(P,c,rho)`. Pattern applicability and extraction are one relation.

### DeduceList

A binding-site list of Pattern holes. Each declaration receives a
`HoleBinderId` within a `PatternRoot`; display spelling is provenance.

### PatternRoot

The identity boundary within which Hole declarations must be unique. Nested
callables allocate their own owner/root and may shadow inherited spelling.

### PolicyLet

An expression boundary `P let e` that forms a complete result demand before
the root call of `e` reaches maxima, resolves the operand once, seals that
selection, and completes the outward view without reopening it.

### ReturnEvent / TailValue

Normalized control-flow end events. `TailValue` delivers the final block value;
`ReturnEvent` preserves an early-return value and unresolved target syntax.

## Semantic entities and identity

### Object

The owned semantic ontology `Object = <Val1?, Pattern, Val2>`. Ordinary
normalization observes all three components. Place, Policy, lifetime,
capability, and name-binding identity are not Object axes.

### Val1

The optional value component of an Object. Unknown content may use a stable
opaque leaf that under-merges without inventing equality.

### Val2

The Object's owned selector-to-Object snapshot. Ordinary entries may have any
type and need no callability or Pattern registration. V_tau registers callable
values without requiring or granting val::path navigation to them; Pattern
extraction/construction registration is independent. Both registered families
are non-generative and require their classifiers under tau.
Navigation-visible or inherited members are separate observations.

### Close and generated realization

Close ends the construction window and freezes non-generative Pattern/V_tau
registrations. It requires the retained names being published to be initialized,
not all possible coordinates to be realized. Ordinary generated Val2 results
may still be realized without registration or a reopened construction view.
Current Core/Val2 observations may change; earlier copied snapshots do not.
only_val2 counts its actual snapshot, not every future generated result.

### Complete pattern/type value (`tau`)

`tau = bind alpha.<Core(tau), V_tau[alpha]>`. `Core(tau)` supplies ordinary
type equality; `V_tau` is an immutable TypeMember callspace snapshot; the whole
observation distinguishes snapshots.

### TypeValueId

An opaque implementation lookup key for Core material. It is not whole `tau`,
name-binding identity, Place identity, or a defining-binding reference.

### Name binding and named type

NameCoord(root,selector) exists independently of realization for legal material;
it is not an Object or Place. Retained records actual realization, and Fresh
means not Retained. A typed realized name has a Place that may remain
Uninitialized until ordinary first write. These facts are independent of visibility.
Initialized structural names declared :type denote complete named types; same-name construction contributes
to their V_tau. Ordinary lexical binding is not implicit overload synthesis.
Name binding identity, pattern-value equality and Place identity are distinct.

### OverloadGroup

An ordinary outer candidate aggregation algebra, with eta(T) = {T}. It can be
empty. Group += aggregates type/group candidates by its bucket relation without
mutating the types; type += instead changes V_tau under OpenHere and final
Home(TypeOf(v)) = TypeMemberScope(T), with independent residency and registration.
Buckets compare complete bound type snapshots, never Core equality.

### SemanticOwner

A node in the typed parent-linked owner graph. Source-established namespace
owners, callable owners, canonical meta-instance owners, and generated owners
qualify their local identities. A package graph supplies no semantic owner.

### MetaInstance

A semantic owner identified by parent owner, selected callable identity, and
canonical invocation input identity, preserving its declared value observations
and semantically observed name/subject/borrow dependencies.

### Invocation-generated result name

The ordinary meta instance name denotes its instance type tau_M, rooted at its
invocation owner. It is not an input structural child or an arbitrary payload
wrapper. Ordinary Val2 contains arbitrary payloads accessed by name::path. Its
opening source is the meet over actual semantic input dependencies. The cache
retains the instance and member Places/current state, not a frozen first value.

### P1 meta policy

Contextual openness qualification, currently limited to type and type ref,
not a fourth PolicyMode or arbitrary meta X ref. Meta refs preserve the actual
Place and original borrowed generation/opening subject. Writable candidates
and explicit ConfirmMut require current OpenHere plus independent target
Writable and ordinary capability/access/lifetime; no authority is amplified.

`meta let f = expression` retains an ordinary meta instance under its derived
OpenHere. The instance name is its type value, so OpenHere governs acquisition
of mut qualification without an independent instance const/mut gate. Plain let
completes/closes it; later meta let cannot reopen it. P2 meta independently names
the callable's evaluation horizon. Ordinary Val2 payload policies remain ordinary.

### Place

A horizontal residency coordinate. A value may reside in multiple Places;
binding creation creates a fresh destination Place.

### Resident generation

One occupant interval of a Place. Whole-resident replacement ends one
generation and starts another.

### ProjectionSlot

A prospective member coordinate identified by parent resident generation and
selector. Parent replacement invalidates its slot family rather than
retargeting it.

## Pattern relation

### `R_Gamma(P,c,rho)`

The proof-relevant relation interpreting Pattern `P` against candidate `c` and
producing Hole valuation `rho`. `Applicable_Gamma(P,c)` holds exactly when at
least one valuation exists.

### DirectPatternChild

Evidence that a member participates in a Pattern's structural incidence.
Ordinary Val2 membership alone is insufficient.

### StructuralDefault

The protected candidate family used for P-internal atomic structural
extraction. Ordinary member access does not receive this family filter.

## Policy, capability, and calls

### PolicyPair

An internal pair of independent value/Pattern observations, containing stage
and presence facts. Public `Pv:Pp` syntax is retired. Direct source Policy and
Policy of its direct type projection observe one evaluation edge; a separately
bound type value has a new edge. Mode and safety remain independent.

### PolicyMode

The primitive three-point set `{const, plain, mut}`. Plain is neither omission
nor a union of endpoints.

### PolicyView

One observed view with independent PolicyPair, PolicyMode and SafetyPolicy
coordinates. Pin overlays P2 and Pout overlays P1; omission is no override,
explicit plain is a constraint, and an explicit hole is Pattern deduction.

### ResultPolicyDemand

The candidate-independent output demand formed before maxima. Pair/stage
coordinates constrain admissibility; mode participates in the three-point
preference relation.

### CapabilityRealization

A candidate/family fact with absent/default/delete/custom realizations,
independent of preference. A 3x3 input/output-mode table is a finite derived
explanatory view; relational declarations with ordinary Pattern holes need not
be written as nine primitive declarations.

### Operator Pattern and policy deduction

Naked OperatorUse selects operator[op], dot .op selects op::adl and explicit
paths stay explicit. OperatorNameValue reads without recursively dispatching.
OG_s is an ordinary spelling-retaining type family with explicit Forget_s to
OverloadGroup; selection reads the current environment slot. Ordinary call,
registered extraction and generative projections retain their distinct roles. Extraction observes the same relation as construction,
not a synthesized inverse. Policy composition/deduction uses those registered
relations, HoleBinderId, require and ordinary overload selection. Omission,
explicit concrete mode and explicit hole remain distinct source constraints.

### Generative occurrence

An occurrence forming a requested name/result relation may establish ordinary
Val2 residency, but supplies no V_tau or Pattern registration evidence. This
restriction does not permanently taint the value. Optional callable heads, Concrete/Wildcard/HoleRef name selectors and general
expression bodies use ordinary specificity, not a separate generation priority.

### DynamicLegality

The post-selection validation of capability, Place, Writable, authority,
lifetime, and other context facts. Failure is terminal and never reopens
overload resolution.

### CallableProjection

A type callee projects its own V_tau to actual callable c, which enters through
Type(c)'s associated Val2[()] with self=c. An ordinary x enters directly through
Type(x)'s associated Val2[()] with self=x. Explicit OverloadGroups aggregate
type projections through singleton embedding. Name resolution occurs before this
projection and is never retried because callability or applicability fails.

### Sealed selected invocation

The unique selected callable plus fixed invocation frame and evidence. It
contains no executable runner-up set.

### InvocationResult

The unified result envelope:

```text
SemanticResult(DeclaredResultClass)
| Residual
| Diagnostic
```

`struct` has declared result class CompleteType and carries an actual complete
type value.

### Policy migration

A direct same-Type operation: existing-view-first, then one authorized
candidate family, ordinary selection, and coherent Policy projection plus
value realization. There is no transitive migration search.

## Construction and literals

### OpenHere

A contextual judgment requiring a live construction window and matching
construction authority. It is independent of Writable and PolicyMode.

### Writable

A context-indexed Place judgment. `mut` does not imply Writable.

### `extend`

A pure transformation that returns a new complete value/snapshot without
writing a Place.

### `inject`

`read + extend + write` at an existing writable target. Member creation,
member write, assignment, inject, and rebind remain distinct operations.

### Abstract literal

An exact `integer`, `real`, or `character` semantic value formed at compile
stage before contextual construction. A concrete expected type does not alter
this initial type.

## Lifecycle

### SemanticContinuation

The ordered evaluation position space shared by lifecycle observation and
committed actions.

### LifeName / NameView / LifetimeValue

`LifeName` identifies a lifecycle subject; `NameView` is its observation at a
continuation position; `LifetimeValue` is its ordinary value observation. N@ is a name iff N is a
name. Value and borrowed lifecycle fields remain distinct.

### Region generation

A gapless half-open interval delimited by use/move/drop events. A killing move ends the source generation and begins its replacement at one
continuation cut. Preserve follows its independent narrow proof.

### Pre / Post

Pre validates an action before mutation. Post records only committed success.
Neither stage participates in overload reselection.

### Color

An extensible identity vocabulary with explicit directed Compatible, Excluded,
and Exchangeable relation rows. No symmetry or reflexivity is implicit.

## Implementation frontier

### Consumer pending

A canonical relation exists but a source/evaluator occurrence is not connected
to it. The occurrence remains unavailable; no alternate relation supplies an
answer.

### Open

A representation or semantic choice explicitly listed in
`spec/planning/open-questions.md`. Open questions use opaque carriers and
extension interfaces until resolved.


### Structural let expression

Qualified formation resolves a structural root identity and observes the current
resident type's OpenHere, selector validity, non-retention and ordinary
access/path/type legality. It requires no parent Writable or parent mut type ref.
Equal type values do not merge structural root/name/Place identities. Borrowing
is a separate Place-side judgment. Initialized type names admit direct mut
borrowing or explicit meta type ref followed by ConfirmMut, subject to the same
current OpenHere, target Writable, capability and lifetime checks. These coherent
routes introduce no implicit chain; saved refs retain their borrowed generation
and cannot write after Close. Initial refs remain initialization-only. See the
[type/ref owner](../design/symbol-world/type-values-places-and-borrow-views.md#522-initialized-type-names-meta-references-and-mut-confirmation).

Initializer-free P let name:t and P let name::path:t create typed NameExpr
using lexical and structural destinations respectively, with non-Object
Uninitialized Place state. In (P let name::path), omitted :t defaults to :type,
not an existing type resident. P let name = rhs is a complete lexical binding
with RHS type inference, so that default does not apply. Value use requires initialization; explicit
ref borrows the Place using its declared type without reading. Ordinary write
initializes it using authority independent of the name's declaration policy,
including const. Successful first commit consumes that authority; saved initial
references do not grant replacement power. Later writes require ordinary
replacement capability and resident compatibility. The structural let=compound
is not canonical. Close requires retained structural names being published to be
initialized, not every future generated coordinate realized. Ordinary lexical
let remains unchanged.

### Associated compile state A

A derived meta invocation returning an instance type with ordinary Val2 group
member n_A(t). Invocation normalization retains t's construction subject, whose
source bounds the retained instance and group write window. Member value/ref and
policy rules remain ordinary. Saved group references preserve their member Place
and dependency across input-carrier replacement; every write Pre rechecks it.
The general meta cache supplies instance/member residency, without an A-specific
global map primitive.

### Anchored replication

A ReinstantiationWitness for an eligible closure permits a new instance under
a target anchor. Captures keep their semantic values and ordinary borrow rules;
internal identities are consistently renamed. The original owner is unchanged.

### SafetyPolicy and unsafe admission

safe/unsafe is orthogonal to const/plain/mut. Unsafe admits compatible external
facts after successful commit; it cannot satisfy a missing Pre or revoke
history. The admitted axioms form the program's trusted semantic base; UB is
external reality failing to satisfy an explicit unsafe admission.

### HostCapability

An otherwise unavailable host capability returning ordinary Object. Its source
meta use determines acquisition and target-machine facts; no build side input
or private optimizer assumptions supply them.

### E, O1, O2 and planner

E saturates ready actions without opportunity-seeking rewrites, E E = E, as
synchronous projections of one continuation. O1 exposes new legal E work; O2
lowers accepted runtime residue costs. Both use E facts and revalidate affected
projections. The planner controls search, never meaning.

### Stage, horizon, producer visibility and readiness

Stage={meta,compile,seal,runtime}, one atom per resolved coordinate. Static
atoms are pairwise incomparable; only identity and static-to-runtime edges
exist. P2 is evaluation horizon; P1/Pout is producer visibility; InputAdmissible
is position-sensitive acceptance and Ready is frontier execution legality.
Neither acceptance nor deferral is Policy migration.

### R_vis and C_sigma

Round-one visibility/input/projection evidence prepares ordinary compile
realizations C_sigma(c). Round two performs hard A, fallback suppression and
ordinary Policy/Pattern selection. Selected=(c*,sigma*,frame) fixes the shared
origin of all projections and runtime residue. No speculative bodies or
runtime reselection occur.

### Active MetaDom / SealDom

Restrictions imposed by actual active frames, propagated through helpers.
MetaDom excludes seal; SealDom excludes meta invocation, including cache hits.
Stable owner history does not establish active dominance. Main has runtime P2.

### Killable / MoveEffect / Movable

Killable_K(n) describes an instance; MoveEffect_K(n,m) is predetermined Kill or
Preserve; Movable_K(n,m) checks current Pre. Nonkillability is neither movement
nor copy permission. Type equality, stage and ZST layout do not collapse them.

### With placement

x with{a} adds x's actual Use/Consume/Destroy touches to a's placement uses.
Existing destructors order x before a; a's uses do not extend x. Empty with
anchors lexical cleanup; omission uses NLL. Killing move adds no old-generation
destructor. Placement precedes lifetime observation and grants no access edge.

### Split / internal completion / residual escape

Split_Gamma(A,S)=<H,R,delta> is a restricted proof-relevant split; D returns R.
Done_chi(v) is internal chain completion, never an Object or user Pattern.
Target return has no fabricated local unit. CanEscape_Sigma(R,B) is a separate
fixed consumer of current ordinary meta payload facts, not a universal
empty-residual rule or a new trait ontology.

### Closure formation

Every legal completed closure expression returns full tau_C through ordinary
struct Material_C. tau_C, its contributed callable c_C, A_C=Type(c_C) and
the () entry are distinct. File implementation-layer let installs at the
established package structural root; true lexical let remains a binding.
In-place syntax uses automatic dependency formation and produces an ordinary
first-class result. Binding, transfer and outer writes depend on actual
dependencies, access, capability and lifetime; invocation does not recapture.

### Structured Path and Pattern splice

Path is ordinary extractable linked material (NameNode, ValueRoot/RefRoot,
Link/End and endpoint shape), before external Read. Text roots resolve at use;
explicit value/reference roots retain anchors and dependencies. Postfix # quotes
Path structure. General $ injects ready ordinary material into a Pattern
consumer, preserves Hole identities and performs no textual substitution.
PathShaped admits finite name chains with a textual/open root endpoint, or
inward names followed by exactly one terminal explicit root. Endpoint shape
and root position are semantic constraints. The name family's explicit string
projection returns its stored name parameter. Open navigation yields a finite
Product of individually named entries.

### General dependency

Needs describes required observations; semantic realization fixes snapshot or
live reference; representation selects layout afterward. Explicit [] is one
dependency source. Initialization occurs once per formation, with common
pre-capture name scope and ordinary effect order; projections do not recapture.
Persistence and escape use the lifetime owner's refinement handoff.
