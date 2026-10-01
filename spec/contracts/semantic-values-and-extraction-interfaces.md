# Semantic Values and Extraction Interfaces

Status: Current implementation contract

The semantic value universe is the ordinary Object universe:

```text
Object(x) = <Val1?(x), Pattern(x), Val2(x)>
```

Construction bodies may use private replay material, but only the declared
semantic result crosses the invocation boundary. `struct` materializes and
returns a complete type value. `StructConstructionMaterial` remains private to
execution. Struct Pattern syntax material is converted to
`CanonicalPatternValue` before it participates in semantic relations.

`R_Gamma(P,c,rho)` is the sole Pattern applicability and extraction relation.
Its content input is the Object's `Val1?` and owned `Val2`. Structural
extraction additionally requires explicit `DirectPatternChild` evidence and
the `StructuralDefault` family filter.

Complete type, NameBinding, OverloadGroup, Place, Pattern root, and semantic value identities remain
separate throughout construction and extraction.

One installed name binding has at most one resident: an absent-Val1 type or an
ordinary value with an actual destination Place. The carrier uses `resident=None`
when no resident is established; a present ordinary resident cannot lack its Place.
Binding Policy projections do not introduce additional residents.
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
Builtin body handling returns private material to the ordinary declared-result
consumer. Source completion remains unavailable; no separate meta block evaluator
supplies a substitute result. Preparation supplies no Ready or common commit proof.
`BuiltinBodyMaterial` belongs only to selected builtin leaves. The source-body
frontier has a diagnostic-only interface and cannot produce that private material.
It provides no body-local initializer evaluation hook. Standalone source expression
completion reports the common E frontier. The `verify` namespace and its builtin
entries are ordinary graph entries; no source post-pass interprets verification
calls. The ordinary selected Verify consumer remains unavailable.
An inner builtin call must form its semantic result at its own invocation boundary
before future common E source-body completion can consume it.

## Shared action and lifecycle handoff

The common continuation owns action identity and position. `LifecycleState`
is a projection, with no private continuation or action-commit API.

```text
selected action + fixed continuation cut
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

Move carries its source, destination and already fixed `MoveEffect`.
Supplied Movable, instance Killable and the narrow Preserve proof are distinct
facts, never inferred from Alive, Type or Policy. Failure cannot change the
effect or clone instead. Kill closes the source and establishes the destination
generation at one cut, retaining deeper origin and Color. Preserve retains the
surviving subject and does not construct a copy.

Cleanup points arrive already fixed by all ordinary constraints. The schedule
retains declaration order and precedence. `freeze_cleanup_through(k)` fixes
the complete prefix through k, including empty cuts, without shifting any point.
It linearizes same-point events while preserving all previously fixed order.
New generations may receive placements strictly beyond that prefix and the
committed frontier. Future material remains a separate, unfinalized suffix;
`cleanup()` exposes only the fixed sequence. Every lifecycle Pre requires its
requested cut to be covered by the fixed prefix.
At the scheduled cut cleanup submits Drop,
not a separate lifecycle event. A committed Kill/Drop discharges the generation's
obligation. Every lifecycle action's Pre rejects crossing any earlier fixed
cleanup point whose obligation remains outstanding, even for another subject.
Failure leaves the fixed Drop executable at its original cut; neither the cut
nor the ordinal advances. At its own fixed endpoint, Use/Preserve fail; Drop
and a Kill replacing that Drop must follow the fixed sequence. Kill before
its future scheduled Drop still discharges that obligation without borrowing
the future cut's ordering. An ending action cannot close a half-open Region
at a cut already containing Use/Preserve of that subject, even without a
scheduled Drop. Action ordinals do not enlarge Region membership.
Preserve discharges no cleanup obligation.

Full E integration must enforce the outstanding-cleanup boundary for every
semantic action, including actions without a lifecycle projection. Currently
that check resides in `LifecycleState::check_pre`; `commit_action` does not
perform it automatically. Before any action commits at k, every fixed cleanup
strictly before k must have been committed or its obligation discharged.
The shared scheduler or common transaction must enforce this gate; inclusion
of lifecycle Pre by an individual caller is not sufficient integration evidence.

Roster discovery establishes only the stable value-to-LifeName map. It supplies
neither Alive nor a Region nor an origin proposition. Formation consumes an
explicit formation cut; missing/default origin producer material remains
`LifecycleOrigin::Pending`, distinct from `ExplicitNone` and `Name(n)`.
Finite Color queries cannot truncate pending ancestry as if it terminated.
The `admit_committed_*_fact` interfaces import producer-established facts;
they are not lifecycle action commits. CompilationWorld exposes lifecycle
state read-only; source fact producers remain unconnected.
When those producers are connected, formation/origin/Color facts must enter
their projection Post in the same scratch publication as the producer action:
`ProducerPre -> one common commit -> joint producer/lifecycle Post`.
The trusted-import interfaces do not prove that handoff. They must either be
restricted to internal helpers within joint Post or consume the common
committed producer witness within that transaction. A separate lifecycle
mutation after common publication does not satisfy the contract; any affected
Pre or Post failure must publish neither producer facts nor a continuation cut.
Color admission additionally checks the supplied continuation identity and
that its frontier covers this projection's committed events and subject birth.
It accepts only formed, active generations. Unknown, formation-pending, ended
or foreign subjects fail before mutation. This API does not backfill historical
Color: after Kill/Drop, the old generation's direct Color facts cannot change,
preserving Kill's inherited direct/deeper Color facts. Observations through
retained origins still include permitted additions to active ancestors.
Bare LifeName numbers
are interpreted within the supplied K, never as independent authority.

ReifyLife must describe the fully fixed generation continuation. A frozen
cleanup table alone proves neither its full endpoint nor the absence of other
generation-ending actions. The operational active Region is not a completed
LifetimeValue. Until that continuation projection is connected, `reify_value`
reports `LifecycleContinuationPending` (or the earlier missing formation/origin
frontier), never an incomplete value with an invented unbounded end. Thus a
value formed at 0 with cleanup fixed at 9 cannot be exposed as `[0,?)`.
Reification does not solve placement or move cleanup points.
NLL/with point derivation, source events and atomic decomposition/destructor
continuations are not implemented by these substrate interfaces.
