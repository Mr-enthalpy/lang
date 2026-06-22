# ADR 0002: Closure AST Not Object

**Status:** Accepted

**Context:** Many languages treat `{ ... }` as a block expression and closures
as runtime objects. A decision was needed for the v0.1 parser.

**Decision:** Closure syntax produces `ClosureAST`, not a callable
`ClosureObject`. Bare `{ ... }` in atom position is not a closure literal.
Braces delimit a closure body only after explicit closure syntax. Materialization
into a callable object is a future semantic pass.

**Consequences:**
- The parser has no block-expression AST node.
- Closure AST can be consumed by future meta-functions (match, effect,
  sync) at the AST level without materialization.
- Dump output shows closure AST rather than a callable value.
