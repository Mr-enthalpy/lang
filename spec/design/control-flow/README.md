# Control-Flow Design Block

Canonical targeted return, serial completion and D-reduction semantics,
with implementation consumers still pending.

The current implemented slice is intentionally narrow: normalized
`ReturnEvent` material can be bound to an active return target frame by the
build-layer return-target binding substrate. Completion propagation,
D/Done, lifetime postconditions, HIR lowering, and runtime execution remain
future work.

## Documents

| Document | Purpose |
|---|---|
| `targeted-return-and-d-reduction.md` | Separate ReturnEvent/ReturnTarget inference, UnitDiscard, path-tail unit fallthrough, direct result delivery and internal completion with no local return contribution |

## Status

Return target binding is partially implemented after normalized AST.
Completion consumers remain unconnected. The canonical laws are settled:
non-tail expressions require unit, tail unit falls through, tail non-unit
synthesizes return and then infers its target. Raw/Norm preserves plain Expr separately from explicit ReturnEvent. The
binder queries the outermost active function; serial completion remains pending.
