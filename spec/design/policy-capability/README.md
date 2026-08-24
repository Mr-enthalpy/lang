# policy-capability

**Status: Non-normative implementation map. The current substrate provides a
typed policy-pair algebra and three-phase visibility helpers, while some graph
consumers still use a legacy flat `PolicyEnv` adapter. Full end-to-end policy
checking (projection at every binding path, conformance, and
effect/error/panic policy) is not implemented.**

## Scope

Policy implementation mapping and orthogonal semantic dimensions:

- mapping current flat symbol/body/result metadata to canonical `Pv:Pp` plus
  whole-slot `PolicyMode={const,plain,mut}`;
- contextual P1 binding/view projection, P2 result normalization, and
  function-object stage-view derivation;
- a projection-empty Runtime Val1 transition-preparation helper plus a
  transitional input × output Policy candidate-ordering prototype (current contract:
  `../../contracts/v0.6-cross-policy-value-transition.md`);
- meta/compile/seal visibility and the pre-seal snapshot boundary;
- explicit retirement of an independent return-policy `P3`;
- component-preserving results rather than a scalar result-class policy;
- current flat `PolicyEnv` filtering substrate;
- typed namespace, three-point PolicyMode, 3×3 capability realization,
  value-presence, error/panic, and resource policy. The flat adapter and current
  2×2 transport fixture are legacy implementation subsets only.

Layered symbol policy, compile-flow projection, compile companions, match
staging, and automatic require are canonical in
`../symbol-world/symbol-policy-and-compile-flow-projection.md`.

## Not in scope

Mechanical return normalization itself. The return-normalization / `noerror`
design lives in `mechanical-lowering/`; do not move it into this block.
Lifetime policy/refinement is also outside the type/compile policy pipeline; its
negative boundary lives in `../lifetime/`.

## Documents

- `policy-visibility-symbols.md` — the overall policy model.

## Reading order

Read `policy-visibility-symbols.md`.

## Dependencies

Maps current policy metadata used by `symbol-world/` lookup and
`meta-invocation/` execution gating. The final symbol-flow policy owner is
`symbol-world/`; mechanical error-handler policy remains separate.
