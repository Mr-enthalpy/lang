# Lifetime Policy and Overload Boundary

Status: negative boundary only. No positive lifetime design is established in
this revision.

The only frozen statements are:

```text
@ belongs only to lifetime syntax
lifetime policy is not ordinary meta/compile/seal/runtime policy
ordinary overload selection must produce one unique candidate
lifetime rules cannot change that completed ordinary overload result
```

Examples such as `val@`, `val@.region`, and `val@.origin` reserve lexical
territory but do not define an algebra or evaluator.

This revision deliberately defines none of the following:

- a lifetime checking algorithm;
- lifetime overloads or a second selection step;
- lifetime ordering or specificity;
- multiple-callable handoff objects;
- ABI equivalence classes used for selection;
- refinement ordering or a refinement phase;
- lifetime cache identity or diagnostics;
- Horae semantics.

Future lifetime work must take the already unique ordinary overload result as
input and may validate it only within rules introduced by that future design.
It may not reopen type/policy overload resolution.

Related canonical contracts:

- [`../patterns-overload/overload-resolution-design.md`](../patterns-overload/overload-resolution-design.md)
- [`../symbol-world/symbol-policy-and-compile-flow-projection.md`](../symbol-world/symbol-policy-and-compile-flow-projection.md)
