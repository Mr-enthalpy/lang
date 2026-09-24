# Canonical Overload Resolution

Status: Current canonical design

## 1. Resolve once, then project

Lexical resolution produces one terminal name binding before callability or
applicability is considered:

```text
S = Resolve_Gamma(path)
-- named-type case of the subsequent consumer projection:
Invoke(CallCandidates(NamedType(S)))
```

Shadowing therefore precedes applicability. A non-callable nearest name binding, an
empty candidate family, or an A-stage rejection never restarts name resolution
at an outer same-name name binding.

## 2. Callable projection

An initialized structural name declared :type denotes its complete named type T. This case does not
restrict ordinary Val2 residents to types. Explicit group values use the
singleton type embedding:

    CallCandidates_type(tau) = disjoint_union over c in V_tau of CallCandidates_ordinary(c)
    CallCandidates(G) = disjoint_union over tau in G of CallCandidates_type(tau)
    CallCandidates_ordinary(x) = Entries(AssociatedNamespace(Type(x)).Val2[()], actual_self=x)

Group bucket aggregation does not mutate candidate types. Type projection
expands every c from V_tau into its ordinary Val2[()] entries with self=c.
The entire family participates in one applicability/preference/unique-selection;
there is no preliminary c winner or per-c implementation winner. The selected
candidate retains (c*, Impl*) with its projection and frame, including in residue.
An ordinary x enters its type's associated Val2[()] directly with self=x,
without projecting Type(x).V_tau. Both obey exact callee/first-self type equality. A source binding or Core registry index does
not supply a later callspace snapshot. See
[name/type algebra](../symbol-world/names-and-overload-groups.md).

### 2.1 Value navigation is broader than candidate projection

Suppose an instance has ordinary Val2 members `data` (a non-callable value),
`state` (an OverloadGroup G), and a named type T. Each member can be obtained
through `name::instance` under ordinary access and value rules. Calling the
read group uses CallCandidates(G); calling T uses its captured V_T. Reading
data is legal even though its call projection has no candidates. Neither
successful navigation nor classifier eligibility registers a value in the
instance's own V_tau.

Thus ReadNamedType describes the named-type case, not an implicit conversion
applied to every Val2 resident. Ordinary function values use their exact
complete type's associated Val2[()]. All these entrances share the pipeline below;
none retries name resolution or constructs a wrapper to make a value callable.

## 3. Pipeline

The canonical order is:

```text
1. callee resolution
2. pre-C0 family filter
3. candidate enumeration
4. R_vis(c,Omega,sigma): visibility, InputAdmissible and projection evidence
   -> ordinary C_sigma(c) preparation where required; frame formation
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

ResultPolicyDemand is total before maxima, recording absence when unconstrained. Omitted mode
preserves NoWrittenModeConstraint; inherited/contextual constraints or an
applicable DefaultModeCompletion may resolve a mode demand. Omission alone is
not explicit plain. Pair/stage result demand is a hard candidate constraint;
a resolved whole-slot mode supplies the three-point preference coordinate.
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

Policy holes participate in this same joint relation:

    Pin_i(rho) = ElabIn(P2, Delta_in_i(rho))
    Pout(rho)  = ElabOut(P1, Delta_out(rho))

Pin allows explicit stage atoms/holes; Pout.stage=P1.stage. R_vis executes no
speculative candidate bodies or effects. Unresolved projection evidence stays
in the continuation, not an arbitrary candidate choice.

Actuals and optional output demand constrain compatible solutions together;
neither policy side semantically computes the other. Registered operator
Patterns and require constraints establish applicability before preference.
A concrete generative name head f outranks _ through ordinary specificity only
after matching. Nested producers seal locally; outer candidates cannot reopen
their chosen result policy or overload.

## 5. Selection seal

Unique selection yields a sealed invocation token containing the selected
candidate identity, projection configuration and completed frame:
Selected=(c*,sigma*,frame). Compile projections and runtime residue retain
that same selected origin; runtime does not reselect. Execution receives that token, not the
candidate list. Any later failure—capability, place, lifetime, authority,
projection, body, result class, or migration realization—is terminal for that
invocation and cannot select a runner-up.

## 6. Extension boundary

The order and no-reopen rule are closed. The complete set of later-B filters and
their future source controls remain open. A new filter must register at the
appropriate stage and may not bypass resolve-once, hard A, unique selection, or
DynamicLegality.


## 7. Immediate call-boundary demand

Established outer P1 and P2 jointly constrain the immediate inner call's P1
and P2. A terminal root call receives the selected ReturnPattern/Pout demand
before maxima; no semantic temporary is inserted. When H calls G, H cannot
deduce its demand by running G's unresolved formal or its body. Ready dependency
initializers execute once in the ordinary formation order; speculative body
execution cannot justify a dependency or Policy hole. Explicit user temporaries
remain real boundaries, and selection failure never reopens a sealed inner call.
See the [Policy owner](../symbol-world/symbol-policy-and-compile-flow-projection.md).
