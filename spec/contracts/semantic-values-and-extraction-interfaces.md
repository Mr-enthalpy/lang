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
The explicit ordinary callable-member substrate builder is not a source let
consumer. Callable registration and selection tests remain substrate evidence, not proof of complete
source closure formation or general body execution.
