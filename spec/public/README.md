# Normalized surface

The public frontend documents define current source-to-Normalized-AST
behavior. This layer is syntax-directed and non-semantic: it does not resolve
names, interpret Patterns, select overloads, validate lifetimes, materialize
closures, or execute code.

- [`normalized-surface-semantics.md`](normalized-surface-semantics.md) defines
  call/product binding, PolicyLet preservation, closure and capture shape,
  Pattern-side normalization, origin tracking, and control-flow end events.
- [`agent-interpretation-guide.md`](agent-interpretation-guide.md) gives
  operational guidance for reading source without importing conventional
  call-syntax assumptions.
- [`../contracts/raw-ast-contract.md`](../contracts/raw-ast-contract.md) is the
  enforced Raw-AST and validated-normalization handoff.

Later semantic relations may consume these structures but never feed meaning
back into lexing, parsing, or normalization.
