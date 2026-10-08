# Policy and capability

**Status:** focused reference for the canonical Policy and capability
relations. The normative owner is
`../symbol-world/policy-and-static-flow-projection.md`.

The semantic coordinates are:

```text
PolicyView = <PolicyPair, PolicyMode>
PolicyMode = const | mut
OpenPolicy = compile | close
ResultPolicyDemand = <P1Projection, PolicyMode, ...future dimensions>
```

`PolicyPair`, whole-slot `PolicyMode`, derived two-mode finite `CapabilityRealization` view,
DynamicLegality, namespace visibility, and export-root identity are orthogonal.
No coordinate is inferred from another.

Position policies inherit evaluation stage from their declaration base and may
override only dimensions explicitly declared overridable, including
PolicyMode. Caller result demand does not propagate into declaration-local
parameter/return policies.

Read `policy-visibility-symbols.md` for the focused model and the symbol-world
owner for compile projection, PolicyLet, migration, seal, and invocation
integration. Mechanical return normalization and lifetime validation remain in
their own design blocks.
