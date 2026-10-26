# Policy Migration Contract

Status: canonical migration contract; full stage/projection consumers pending

Policy migration is an ordinary same-Type invocation family:

```text
source PolicyView
  -> target ResultPolicyDemand
  -> existing-view-first
  -> authorized migration candidates
  -> ordinary applicability and Policy preference
  -> one sealed candidate
  -> DynamicLegality
  -> PolicyProjection × ValueRealization
```

The target demand exists before candidate enumeration. Migration never searches
a graph, chains intermediate views, changes the source Core type, or reopens
selection after a selected failure.

Stage={compile,runtime}. Endpoints observe linked adjacent ranks. A direct
migration preserves Type and rank-chain coherence; compile:runtime is invalid.
Ordinary PolicyLet requests only the first non-absent rank, preserving its
classifier and further ranks. Explicit Hole Policies can constrain multiple
rank positions. Runtime-to-compile
requires the active closed reify P2 boundary, never a freely supplied flag.
Runtime:runtime -> runtime:compile yields a runtime Object whose own classifier
is compile-available. The classifier must be that Object's actual Type; a
separate compile type with equal Policy is no witness for this migration.
The selected dependent E work resumes at the runtime frontier; no past compile
world receives runtime information. InputAdmissible, concrete readiness and
endpoint capability remain independent. Bootstrap implementations provide candidate bodies and
realization data; they do not decide the migration relation.

Until active-boundary/closed-return/dependency-cone consumers are connected,
reverse requests report unavailable before candidate execution or publication.
Ordinary requests cannot authorize reverse migration.
P2 runtime / P1 compile is legal by itself; reify is required by the actual
reverse-dependent continuation, not an in/out stage matrix. First-round Policy
selection consumes declared input/output constraints equally and independently
of second-round execution.

Every request carries explicit `PolicyView` and `ResultPolicyDemand` values.
`PolicyPair`, whole-slot `PolicyMode`, capability realization, and dynamic
legality remain independent coordinates.


The endpoint pairs above are internal notation, not source Policy literals.
Public demand uses ordinary constraints on the value and direct type projections
of the same observation edge. Removing colon syntax removes no endpoint checks:
presence, both Policy observations, same Type and capability coherence remain
mandatory. Already available views win before the one admitted direct family;
no-reopen and selected-failure rules are unchanged. opposite-context Policy interpretation obeys readiness
and never runs a candidate body to obtain its own applicability Pattern.
