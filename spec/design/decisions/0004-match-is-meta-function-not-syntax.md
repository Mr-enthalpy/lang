# ADR 0004: Match Is Meta-Function, Not Syntax

**Status:** Accepted

**Context:** Pattern matching is often syntactic (e.g., Rust `match`, Haskell
`case`). The language design calls for `match` to be a compiler-provided
meta-function that consumes closure AST arms, but early contributors might
implement it as parser syntax.

**Decision:** `match` is a `Name` token at the parser level. It is not
syntax. A future compiler-provided meta-function named `match` may consume
closure AST arms, but the parser must not special-case `match`.

**Consequences:**
- No `MatchExpr` AST node exists in v0.1.
- `match` appears in AST as `Name("match")`.
- Expression shapes that look like match (e.g., `obj (... { ... }) match`)
  are parsed as ordinary PipeExpr → Segment → Atom + ArgPack + Atom(Name).
- A future semantic pass will interpret these shapes as match expressions.
