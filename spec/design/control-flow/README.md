# Control-Flow Design Block

Forward-looking design material for targeted return, D-reduction,
Done_Return, and control-flow lowering.

The current implemented slice is intentionally narrow: normalized
`ReturnEvent` material can be bound to an active return target frame by the
build-layer return-target binding substrate. Completion propagation,
D/Done, lifetime postconditions, HIR lowering, and runtime execution remain
future work.

## Documents

| Document | Purpose |
|---|---|
| `targeted-return-and-d-reduction.md` | Current return-target binding substrate plus future targeted return completion, D-reduction, Done_Return, and local unit contribution |

## Status

Return target binding is partially implemented after normalized AST.
Everything beyond binding the target frame remains design-only in this
directory.
