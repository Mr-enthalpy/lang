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

Stage is one atom. The directed static-to-runtime cases preserve Pp and Type,
including compile:compile -> runtime:compile and seal:seal -> runtime:seal.
No runtime-to-compile or seal-to-compile edge exists. InputAdmissible and Ready
are separate; deferring a compile callable for a seal input is not migration. Bootstrap implementations provide candidate bodies and
realization data; they do not decide the migration relation.

Every request carries explicit `PolicyView` and `ResultPolicyDemand` values.
`PolicyPair`, whole-slot `PolicyMode`, capability realization, and dynamic
legality remain independent coordinates.


The endpoint pairs above are internal notation, not source Policy literals.
Public demand uses ordinary constraints on the value and direct type projections
of the same observation edge. Removing colon syntax removes no endpoint checks:
presence, both Policy observations, same Type and capability coherence remain
mandatory. Already available views win before the one admitted direct family;
no-reopen and selected-failure rules are unchanged. Policy splice obeys readiness
and never runs a candidate body to obtain its own applicability Pattern.
