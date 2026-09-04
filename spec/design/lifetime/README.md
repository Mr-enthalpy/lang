# Lifetime design

The canonical owner is
[`lifetime-policy-and-overload-boundary.md`](lifetime-policy-and-overload-boundary.md).

Its core relation is:

```text
@ = ReifyLife(NameOf(E), Pos(SemanticContinuation))
```

`@` returns an ordinary first-class `LifetimeValue`; it is not a borrow or
place-acquisition operation and does not require the operand to have a Place.
`ref` and `share` are explicit borrow constructors with privileged access to
the actual Place.

The owner also defines LifeName/NameView, cleanup-before-observation, gapless
half-open Region generations, exact move cuts, Pre/commit/Post, monotone Color
inheritance, extensible directed Color relations, and the post-selection
lifetime boundary. Concrete IR, source action wiring, access-tree construction,
summary compression, and diagnostics remain implementation frontiers.
