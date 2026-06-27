# policy-capability

**Status: Non-normative future design with a partial implementation note. v0.7-prep
provides policy metadata and `PolicyEnv::Meta` lookup filtering; full policy
checking (lattice, projection, conformance, effect/error/panic policy) is not
implemented.**

## Scope

Policy as visibility symbols and capability strategy:

- symbol visibility policy (which symbols a lookup may find)
- callable body-entry policy (whether a found callable may execute)
- return-object policy (the policy of the produced object)
- context policy and meta/runtime policy filtering
- future error/panic policy

## Not in scope

Mechanical return normalization itself. The return-normalization / `noerror`
design lives in `mechanical-lowering/` and only references the policy planes
defined here; do not move it into this block.

## Documents

- `policy-visibility-symbols.md` — the overall policy model.

## Reading order

Read `policy-visibility-symbols.md`.

## Dependencies

Provides the policy planes used by `symbol-world/` lookup, `meta-invocation/`
execution gating, and the `Error`-handler lookup in `mechanical-lowering/`.
