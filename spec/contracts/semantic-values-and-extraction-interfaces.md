# Semantic Values and Extraction Interfaces

Status: Current implementation contract

The semantic value universe is the ordinary Object universe:

```text
Object(x) = <Val1?(x), Pattern(x), Val2(x)>
```

The [Product owner](../design/patterns-overload/product-repetition-and-unpacking.md)
defines the standard `(_ * ω)$` wrapper and `_ * ω` classifier Pattern.
Bare Product material expands/merges contextually; actual repetition Val1
contributes to complete identity. Call input is an established exposure context
for available standard packaging, including nested source Products. Syntax/material flattening supplies no
packed-value extraction or runtime length observation. Repetition formation,
selected `pack?` and iteration consumers remain unavailable until their
ordinary producer/Policy/Pre interfaces are connected. External S `_$` cannot
be erased or interpreted as the anonymous `_` hole by a partial matcher.

Structural interpretation forms complete types atomically with actual Val2
witnesses and registered roles. struct:type->type receives the complete type
and adds helpers without changing its structural registrations. Private
execution/formation material is consumed within its common transaction;
only ordinary declared semantic results cross InvocationResult.

`R_Gamma(P,c,rho)` is the sole Pattern applicability and extraction relation.
Argument transport retains unknown value/type role separately from observed
presence/absence and unavailable classifier observation separately from known
detached Pattern-coordinate evidence. Relation consumers return a proof,
decided mismatch, or explicit unavailable/unsupported diagnostic. Missing
actual or named classifier observations cannot supply negative applicability
evidence or permit selection of another candidate.
World-connected ordinary arguments transport available frozen complete classifier
snapshots and their Core together before first-round applicability; required
unavailable observations stop the query, while binders/discards add no classifier
premise. Selected input identity normalization is an independent consumer. Lookup keys locate
observations; they are not equality proofs. Source binding/member Policy views
remain on forced-conversion inputs through horizon and Ready checks; stored
payload availability supplies no authority to read a runtime view at compile.
Already selected forced extraction/delivery consumes its proved total-root
mismatch through the result owner's [one root conversion rule](../design/patterns-overload/return-value-extraction-and-implicit-decomposition.md#forced-extraction-and-one-root-conversion).
Applicability remains conversion-free; missing mismatch/root evidence is
unavailable. Selected conversion or final extraction failure never reopens.
If that final actual forced relation proves mismatch, its connected execution
consumer uses the [PatternMiss effect protocol](../design/control-flow/effect-steps-and-continuation-handoff.md#5-forced-patternmiss-and-compile-boundaries).
Applicability/ordinary branch mismatch and unavailable consumers do not throw.
Its content input is the Object's `Val1?` and owned `Val2`. Structural
extraction additionally requires explicit `DirectPatternChild` evidence and
the `StructuralDefault` family filter.

Complete type, NameBinding, OverloadGroup, Place, Pattern root, and semantic value identities remain
separate throughout construction and extraction.

One installed name binding has at most one resident: an absent-Val1 type or an
ordinary value with an actual destination Place. The carrier uses `resident=None`
when no resident is established; a present ordinary resident cannot lack its Place.
Binding Policy projections do not introduce additional residents.
Ordinary and type argument names retain the shared resolved navigation and
complete lexical lookup order. Every required host exposure factor is checked
at the actual invocation horizon before terminal observation. Resident identity
does not deduplicate distinct Policy edges. A single-argument view/mode carrier
is formed only after a unique contextual observation is established; unresolved
multi-edge projection is unavailable, independently of registration order.
Host navigability requests Pattern exposure, without choosing terminal mode.
Object Val2 normalization maps each realized selector to one resident address.
An associated implementation ledger whose ordinary resident is not formed yet
has no Object normal form; normalization reports the unconnected consumer.

The invocation substrate reads ordinary `Type(x)` associated `Val2[()]` entries.
For a complete type it projects all ordinary members of that exact `V_tau`,
retaining `(receiver, implementation)` pairs through a single selection.
The complete V_tau normal form contains unique callable-member observations;
placing the same member in multiple storage selector buckets changes no identity.
Registering an associated entry does not register a type-call member. Type-call
members require ordinary callable values, excluding Core transport projections
and terminal implementation leaves. Migration uses the source's stored complete
Type observation without recovering a declaration binding.

Same-spelled source declarations do not establish contribution authority. The
source closure-to-tau formation and common-snapshot contribution consumers are
unavailable. A named source closure declaration fails before installing any
resident or graph callable record; it cannot bind c_C in place of tau_C.
`install_callable_fixture` is crate-private and compiled only for unit tests.
Integration tests assemble ordinary receivers and associated entries in test-local
helpers. Neither path is a source let consumer. Callable registration and selection tests remain substrate evidence, not proof of complete
source closure formation or general body execution.

Callable source syntax and declaration records are distinct from Object identity.
Declaration, call-entry and prepared-selection carriers retain one implementation
variant each, with stage/Policy facts independent of implementation kind.
Builtin preparation consumes the actual call entry's declared planes and fixed
identity; it creates no synthetic graph Object and derives no Policy from the
implementation enum. An incomplete applicability relation stops the candidate
family before maxima. In particular, Assert/Verify currently have unconnected
parameter relations; they cannot be removed to select a known runner-up.
Builtin body handling returns private material to the ordinary declared-result
consumer. Source completion remains unavailable; no separate compile block evaluator
supplies a substitute result. Preparation supplies no Ready or common commit proof.
`BuiltinBodyMaterial` belongs only to selected builtin leaves. The source-body
frontier has a diagnostic-only interface and cannot produce that private material.
It provides no body-local initializer evaluation hook. Standalone source expression
completion reports the common E frontier. The `verify` namespace and its builtin
entries are ordinary graph entries; no source post-pass interprets verification
calls. Verify applicability and selected body consumers remain unavailable.
An inner builtin call must form its semantic result at its own invocation boundary
before future common E source-body completion can consume it.

## Shared action and lifecycle handoff

The [lifetime-program contract](lifetime-program-consumers.md) extends this
handoff with LC1–LC13: real structure companions, whole-continuation Stage
forgetting, ordinary contract expressions, four-way boundary calls, origin
compression/automatic Post, Function Pattern interfaces, trusted facilities and
regular-access programs, required captures, slice/repair, resource/effect handoff
and complete original-K CPS correspondence. It specifies future source
consumers, not extra Rust evaluator phases. Existing supplied-fact and
transaction tests cover only the slice below; they do not execute those ordinary
source programs or certify a complete `@` value producer.

The common continuation owns action identity and position. `LifecycleState`
is a projection, with no private continuation or action-commit API.

```text
selected action + fixed continuation cut
  -> common cleanup gate on the current state and continuation
  -> all affected projection Pre checks on the current state
  -> one common transaction commit
  -> joint Post publication at that same action identity/cut
```

`LifecycleState::check_pre` returns opaque action/state/continuation/cut evidence without
mutation. `apply_post` consumes the common committed-action witness and that
evidence. Stale, foreign or mismatched evidence cannot publish a projection or
advance the continuation. The generic transaction's scratch storage does not
constitute another evaluator or a semantic retry. Full E/source integration
and external effect publication remain consumer gates.
The evidence includes the fixed cleanup prefix and pending suffix; extending
either changes the continuation snapshot. A different snapshot cannot reuse
evidence merely because identity and action ordinal agree.

The [host revalidation handoff](e-host-architecture-and-incremental-revalidation.md#10-failure-unavailable-and-transaction-discipline)
consumes this exact current-world evidence, not a cached successful callback.
Changed contract content, generation, cleanup snapshot or action/delivery
mapping invalidates dependent checks; the requiring E consumer re-establishes
them before its existing publication gate. Revalidation of represented history
does not apply Post, Move/Kill/Drop or acquisition again. A genuinely new action
uses its own ordinary premises. Missing correspondence remains unavailable;
transaction tests with supplied material do not execute HA acceptance scenarios.

For represented history, the [before-action observation rule](e-host-architecture-and-incremental-revalidation.md#51-same-history-revalidation-and-fresh-compilation-reuse)
locates changed Pre checks in the corresponding revision's lawful pre-state,
not a cached post-state or an unrelated latest resident. Recovered historical
observations establish no present commit witness or authority; new actions
still require the actual gate above. For negative access/effect/cleanup queries,
the host retains observed range/domain completeness support as well as positive
rows. Newly relevant actions invalidate dependent absence conclusions, even
when old rows remain equal; missing support is unavailable (§4 of the handoff).

For a fresh compilation, [authorized cache realization](e-host-architecture-and-incremental-revalidation.md#51-same-history-revalidation-and-fresh-compilation-reuse)
still accounts for required actions/effects in that compilation's E state;
same-history no replay cannot erase new registry entries or effect conflicts.
For O candidates, this publication/legality gate is independently required
alongside [E-grounded observational equivalence](e-host-architecture-and-incremental-revalidation.md#9-future-oplannerm-readiness).
A structural cross-K relation may be nonbijective, but proves neither that
equivalence nor authority to duplicate owned occurrences, effects or cleanup.

The [checkpoint consumer](e-host-architecture-and-incremental-revalidation.md#91-branchable-accepted-checkpoints)
retains or recovers multiple accepted descriptions and their support without
destructively replacing all siblings. Forking/selecting/discarding those
descriptions performs no common business commit or external rollback. Real
actions still use this gate with current evidence. Local O1/O2 objective
switching cannot bypass E saturation or actual cross-region dependencies;
transaction storage must not require a global objective phase barrier.
Function/call-SCC planning partitions are views derived from the current accepted
K. After accepted call rewrites, the host permits their lawful rederivation and
future-work reassignment, preserving actual inlined dependencies, affected
validation and retained-checkpoint correspondence (§9.2 of that handoff).
Partition identity supplies no semantic ownership or lifecycle SCC collapse law.
These are future host capabilities, not tests of checkpoint execution.

HA13/HA16 also distinguish a new irreversible compile-host effect exposed by
an O1 candidate that is later discarded. Description snapshots alone cannot
isolate real execution: without lawful isolation/reuse support, decline that
exploration before the effect. No shared mutable branch history, cloned authority
or claimed rollback may fabricate sibling legality; an actual prior commit
retains its history. Acceptance still requires complete E saturation.

Consumed E-known target facts and admitted external observations retain their
actual source/context, validity interval and truth/maintenance premises as host
dependencies (HA12/HA14). Their change invalidates dependent E/O evidence and
machine artifacts separately. Source-key equality, machine shape or O/M-private
constants cannot replace current evidence or alter historical admission facts.

Move carries its source, destination and already fixed `MoveEffect`.
Supplied Movable, instance Killable and the narrow Preserve proof are distinct
facts, never inferred from Alive, Type or Policy. Failure cannot change the
effect or clone instead. Kill closes the source and establishes the destination
generation at one cut, retaining deeper origin and Color. Preserve retains the
surviving subject and does not construct a copy.

Pending selected-call consumers also preserve the [call owner's self/Param coherence](../design/symbol-world/function-object-call-model.md#63-one-coherent-passing-transaction):
one terminal-Move transaction, no duplicate capture Kill. [Function Pattern](../design/patterns-overload/pattern-relations-observation-and-extraction.md#111-directional-function-pattern-consumers)
LHS contract extraction checks lifetime implication/defaults at assignment;
RHS requests explicit input and still validates the uniquely selected actual
contract. [E's ABIKind](../design/static-evaluation/evaluation-residual-and-optimization.md#10-e-selected-return-delivery-convention)
is fixed by delivery before physical materialization. Existing transaction
carriers/tests establish none of these complete source consumers.

Cleanup points arrive already fixed by all ordinary constraints. The schedule
retains declaration order and precedence. `freeze_cleanup_through(k)` fixes
the complete prefix through k, including empty cuts, without shifting any point.
It linearizes same-point events while preserving all previously fixed order.
New generations may receive placements strictly beyond that prefix and the
committed frontier. Future material remains a separate, unfinalized suffix;
`cleanup()` exposes only the fixed sequence. Every common transaction requires
its requested cut to be covered by the fixed prefix.
At the scheduled cut cleanup submits Drop,
not a separate lifecycle event. A committed Kill/Drop discharges the generation's
obligation. The common transaction rejects crossing any earlier fixed cleanup
point whose obligation remains outstanding, even for another subject.
Failure leaves the fixed Drop executable at its original cut; neither the cut
nor the ordinal advances. At its own fixed endpoint, Use/Preserve fail; Drop
and a Kill replacing that Drop must follow the fixed sequence. Kill before
its future scheduled Drop still discharges that obligation without borrowing
the future cut's ordering. An ending action cannot close a half-open Region
at a cut already containing Use/Preserve of that subject, even without a
scheduled Drop. Action ordinals do not enlarge Region membership.
Preserve discharges no cleanup obligation.

`commit_action` enforces this boundary before projection Pre for every action,
including actions without a lifecycle projection. Its state implements the
`CleanupObligations` observation relation relative to the supplied continuation.
Every fixed cleanup strictly before k must be discharged. Outstanding blocks
the action; missing, foreign or future facts report unavailable and also block
it. Roster discovery and absence from active storage prove no discharge.
Lifecycle storage validates continuation identity, committed event ordinals and
positions, and generation birth before exposing an observation. The common
transaction depends on this relation, not a concrete lifecycle storage type.
Same-cut endpoint and ending-event precedence remain lifecycle Pre obligations.
Source actions still require the common E consumer to enter this transaction;
passing transaction tests supplies no source scheduling or readiness witness.

Roster discovery establishes only the stable value-to-LifeName map. It supplies
neither Alive nor a Region nor an origin proposition. Formation consumes an
explicit formation cut; missing/default origin producer material remains
`LifecycleOrigin::Pending`, distinct from `ExplicitNone` and `Name(n)`.
Finite Color queries cannot truncate pending ancestry as if it terminated.
The private `#[cfg(test)]` `admit_committed_*_fact` helpers seed lifecycle test
fixtures only; no production fact-import API is exposed. CompilationWorld
exposes lifecycle state read-only; source fact producers remain unconnected.
When those producers are connected, formation/origin/Color facts must enter
their projection Post in the same scratch publication as the producer action:
`ProducerPre -> one common commit -> joint producer/lifecycle Post`.
Any internal helper must run within joint Post or consume the common
committed producer witness within that transaction. Test fixture imports do
not prove that handoff. A separate lifecycle mutation after common publication
does not satisfy the contract; any affected
Pre or Post failure must publish neither producer facts nor a continuation cut.
The Color fixture helper additionally checks the supplied continuation
identity and that its frontier covers this projection's committed events and
subject birth.
It accepts only formed, active generations. Unknown, formation-pending, ended
or foreign subjects fail before mutation. This helper does not backfill historical
Color: after Kill/Drop, the old generation's direct Color facts cannot change,
preserving Kill's inherited direct/deeper Color facts. Observations through
retained origins still include permitted additions to active ancestors.
Bare LifeName numbers
are interpreted within the supplied K, never as independent authority.

ReifyLife describes fixed generation and cleanup facts of the current semantic
continuation K, with Region_K(n)=R(NamePosition_K(n)). Future branch enumeration
is not a premise of E. A frozen
cleanup table alone proves neither its full endpoint nor the absence of other
generation-ending actions. The operational active Region is not a completed
LifetimeValue. Until that continuation projection is connected, `reify_value`
reports `LifecycleContinuationPending` (or the earlier missing formation/origin
frontier), never an incomplete value with an invented unbounded end. Thus a
value formed at 0 with cleanup fixed at 9 cannot be exposed as `[0,?)`.
Reification does not solve placement or move cleanup points.
NLL/with point derivation, source events and atomic decomposition/destructor
continuations are not implemented by these substrate interfaces.
