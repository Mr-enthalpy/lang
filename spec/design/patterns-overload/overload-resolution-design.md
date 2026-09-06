# Canonical Overload Resolution

Status: Current canonical design

## 1. Resolve once, then project

Lexical resolution produces one terminal name binding before callability or
applicability is considered:

```text
S = Resolve_Gamma(path)
Invoke(CallCandidates(NamedType(S)))
```

Shadowing therefore precedes applicability. A non-callable nearest name binding, an
empty candidate family, or an A-stage rejection never restarts name resolution
at an outer same-name name binding.

## 2. Callable projection

A resolved structural name denotes its complete named type T. Explicit group
values use the singleton type embedding:

    CallCandidates(T) = CallCandidates(V_tau(T))
    CallCandidates(G) = disjoint_union over T in G of CallCandidates(T)

Group bucket aggregation does not mutate the candidate types. Each value
callee uses its exact captured complete type and associated (), with
Type(callee) = Type(first self). A source binding or Core registry index does
not supply a later callspace snapshot. See
[name/type algebra](../symbol-world/names-and-overload-groups.md).

## 3. Pipeline

The canonical order is:

```text
1. callee resolution
2. pre-C0 family filter
3. candidate enumeration
4. repeated candidate-entry exposure collapse, visibility, phase, and frame formation
5. hard applicability A, including Pattern relation and declared result Type
6. declaration fallback/suppression where the language defines it
7. Policy product preference Bp
8. Pattern specificity and registered later-B filters
9. unique selection and seal
10. DynamicLegality
11. execution
12. InvocationResult
13. optional result-view satisfaction or same-Type migration
```

Only repeated exposure of the same stable candidate-entry identity may collapse.
Distinct contribution entries never deduplicate merely because their values or
types normalize equally; equality and interning cannot quotient those entries.

`OutputModeDemand` is total before Bp maxima. Pair/stage result demand is a hard
candidate constraint; whole-slot mode is the three-point preference coordinate.
Capability realization and dynamic legality do not grant preference.

## 4. Pattern applicability

Candidate applicability consumes a proof of:

```text
R_Gamma(formal_pattern, actual, rho)
```

The valuation `rho` supplies generic Hole bindings. Structural extraction uses
explicit `DirectPatternChild` evidence and applies `StructuralDefault` before
candidate enumeration. No product shape or observed-content carrier defines
Pattern meaning.

## 5. Selection seal

Unique selection yields a sealed invocation token containing the selected
candidate identity and completed frame. Execution receives that token, not the
candidate list. Any later failure—capability, place, lifetime, authority,
projection, body, result class, or migration realization—is terminal for that
invocation and cannot select a runner-up.

## 6. Extension boundary

The order and no-reopen rule are closed. The complete set of later-B filters and
their future source controls remain open. A new filter must register at the
appropriate stage and may not bypass resolve-once, hard A, unique selection, or
DynamicLegality.
