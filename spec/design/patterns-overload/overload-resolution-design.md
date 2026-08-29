# Canonical Overload Resolution

Status: Current canonical design

## 1. Resolve once, then project

Lexical resolution produces one terminal Symbol before callability or
applicability is considered:

```text
S = Resolve_Gamma(path)
Invoke(CallableProjection(S))
```

Shadowing therefore precedes applicability. A non-callable nearest Symbol, an
empty candidate family, or an A-stage rejection never restarts name resolution
at an outer same-name Symbol.

## 2. Callable projection

For Symbol `S` carrying complete type `tau`:

```text
CallableProjection(S)
  = DedupCandidateIdentity(V_S(S) union CallSpace(tau))
```

Symbol-local and TypeMember candidates enter one candidate space. The complete
type snapshot is the snapshot captured by the value or binding, not a live
lookup through a Core index.

## 3. Pipeline

The canonical order is:

```text
1. callee resolution
2. pre-C0 family filter
3. candidate enumeration
4. identity dedup, visibility, phase, and frame formation
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
