# Symbol world

This block owns the language’s Object, complete-type, Symbol, Place, Policy,
construction, and namespace relations.

## Canonical coordinates

```text
Object x = <Val1?(x), Pattern(x), Val2(x)>
complete tau = bind alpha.<Core(tau), V_tau[alpha]>
Symbol S = <tau?, V_S?>
Place = horizontal residency coordinate
```

Object, complete type value, Symbol, Place, owner identity, Policy, capability,
and lifetime are distinct semantic coordinates. Complete type values have no
HomeSymbol. Name resolution produces one terminal Symbol before value/type/call
projection.

## Documents

- `type-values-places-and-borrow-views.md` — Object normalization, complete
  type observations, Place/resident generations, borrow views, literals and
  lifetime-name reification.
- `function-object-call-model.md` — value/type/call projection and associated
  `()`.
- `symbol-first-meta-construction-and-pattern-injection.md` — `struct -> tau`,
  result/install boundary, OpenHere, pure `extend`, and place-level `inject`.
- `symbol-construction-units-and-namespace-origin.md` — construction ownership
  and namespace contribution authority.
- `symbol-policy-and-compile-flow-projection.md` — PolicyPair, PolicyMode,
  demand, migration, capability, seal and compile projection.
- `early-meta-functions-and-namespace-graph.md` — bootstrap and namespace
  graph consumer details.
- `entity-ref-design.md` — preserved strong entity reference shape.
- `entity-alias-design.md` — block-local lexical alias mapping.

Read the type/value/place owner first, then construction/ownership, Policy, and
the relevant consumer. Pattern and overload semantics are owned by
`../patterns-overload/`; lifecycle is owned by `../lifetime/`.
