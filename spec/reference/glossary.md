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

An ordered structural value or Pattern form. A source Product participates in
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
capability, and Symbol identity are not Object axes.

### Val1

The optional value component of an Object. Unknown content may use a stable
opaque leaf that under-merges without inventing equality.

### Val2

The Object's owned selector-to-Object snapshot. Navigation-visible or inherited
members are separate observations.

### Complete pattern/type value (`tau`)

`tau = bind alpha.<Core(tau), V_tau[alpha]>`. `Core(tau)` supplies ordinary
type equality; `V_tau` is an immutable TypeMember callspace snapshot; the whole
observation distinguishes snapshots.

### TypeValueId

An opaque implementation lookup key for Core material. It is not whole `tau`,
Symbol identity, Place identity, or a defining-Symbol reference.

### Name binding and named type

Name existence is a structural fact independent of visibility or value content.
Structural names denote complete named types; same-name construction contributes
to their V_tau. Ordinary lexical binding is not implicit overload synthesis.
Name binding identity, pattern-value equality and Place identity are distinct.

### OverloadGroup

An ordinary outer candidate aggregation algebra, with eta(T) = {T}. It can be
empty. Group += aggregates type/group candidates by its bucket relation without
mutating the types; type += instead changes V_tau under OpenHere and final
TypeOf(v) membership in the core. The current coarse bucket key is the core tau.

### SemanticOwner

A node in the typed parent-linked owner graph. Package/namespace, callable,
MetaInstance, and generated identities qualify their local identities through
an owner.

### MetaInstance

A semantic owner identified by parent owner, selected callable identity, and
canonical whole argument Product identity.

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

The orthogonal value/Pattern policy pair `Pv:Pp`, containing stage and presence
facts. It does not contain or determine whole-slot mode.

### PolicyMode

The primitive three-point set `{const, plain, mut}`. Plain is neither omission
nor a union of endpoints.

### PolicyView

One observed view with independent PolicyPair, PolicyMode and SafetyPolicy
coordinates. Formal pair inheritance does not change unwritten plain mode.

### ResultPolicyDemand

The candidate-independent output demand formed before maxima. Pair/stage
coordinates constrain admissibility; mode participates in the three-point
preference relation.

### CapabilityRealization

A candidate-local 3x3 input-mode/output-mode table whose cells are
`absent | default | delete | custom`. It is independent of Policy preference.

### DynamicLegality

The post-selection validation of capability, Place, Writable, authority,
lifetime, and other context facts. Failure is terminal and never reopens
overload resolution.

### CallableProjection

The ordinary call candidates of a named type's V_tau or of an explicit
OverloadGroup through singleton type embedding. Name resolution occurs before this
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

A gapless half-open interval delimited by use/move/drop events. Move ends the
source generation and begins its replacement at one continuation cut.

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

P let name::path requires freshness, creates a named type and returns mut type
ref. Declared policy P is independent of construction-reference mut policy.
A following = expression is ordinary assignment, with no private rollback rule.

### Associated compile state A

A builtin globally indexed family of OverloadGroup Places. Its write Pre checks
OpenHere of the key and ordinary write authority every time, including through
saved references. Generalizing this guarded place capability remains open.

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
