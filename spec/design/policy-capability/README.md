# policy-capability

**Status: Non-normative future design with a partial implementation note. The
current substrate provides policy metadata and `PolicyEnv::Meta` /
`PolicyEnv::Runtime` lookup filtering; full policy
checking (lattice, projection, conformance, effect/error/panic policy) is not
implemented.**

## Scope

Policy implementation mapping and orthogonal future dimensions:

- mapping current symbol/body/result metadata to final callable `P1` / `P2`;
- lookup visibility versus entry execution and exact invocation-frame policy,
  including implicit self;
- explicit retirement of an independent return-policy `P3`;
- layer-wise result projection rather than a scalar result-symbol policy;
- current `PolicyEnv::Meta` / `PolicyEnv::Runtime` filtering substrate;
- future access, error/panic, const/mut, and resource policy.

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
