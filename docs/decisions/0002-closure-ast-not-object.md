# ADR 0002: Closure AST Not Object

**Status:** Accepted

**Context:** Many languages treat `{ ... }` as a block expression and closure
as a runtime object. A decision was needed for the v0.1 parser.

**Decision:** `{ ... }` in atom position produces `ClosureAST`, not a block
expression and not a callable `ClosureObject`. Materialization into a callable
object is a future semantic pass.

**Consequences:**
- The parser has no block-expression AST node.
- Closure literals can be consumed by future meta-functions (match, effect,
  sync) at the AST level without materialization.
- Dump output shows `Closure(InlineClosureAst(...))` rather than a callable
  value.
