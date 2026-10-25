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

Stage={compile,runtime}. A direct migration can change either observed Policy
coordinate while preserving Type and endpoint coherence. Runtime-to-compile
requires the active closed reify P2 boundary, never a freely supplied flag.
Runtime:runtime -> runtime:compile migrates only the classifier observation.
The selected dependent E work resumes at the runtime frontier; no past compile
world receives runtime information. InputAdmissible, concrete readiness and
endpoint capability remain independent. Bootstrap implementations provide candidate bodies and
realization data; they do not decide the migration relation.

Until active-boundary/closed-return/dependency-cone consumers are connected,
reverse requests report unavailable before candidate execution or publication.
Ordinary requests cannot authorize reverse migration.

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
