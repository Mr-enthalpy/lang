# Lifetime Design

This directory is the design boundary for lifetime-policy work.

Start with:

- [`lifetime-policy-and-overload-boundary.md`](lifetime-policy-and-overload-boundary.md)

That note is the canonical owner of the `@` operation: `@` reifies
`ReifyLife(NameOf(actual), Pos(SemanticContinuation))` as a `LifetimeValue`,
never a borrow view and never a `type ref`. The former two instance groups
(`Val1?(x) ≠ null -> LifetimeFact`, `Val1?(x) = null -> P ref`), the
carrier-slot form `t@ : type ref`, and the borrow-type fixed points
(`type ref@ = type ref`, `type share@ = type share`) are retired. `ref` and
`share` are the borrow constructors; each is a privileged actual-place builtin
(`PrivilegedActualPlace(ref-family)` / `PrivilegedActualPlace(share-family)`)
that may obtain the actual's place, while `@` and ordinary user functions do
not. Explicit higher-level selection uses `t |> (type ref)` /
`t |> (type share)`. The note also keeps target-preserving `ref`/`share`
constructor composition, the escape check on borrow views, the
`NoImplicitBorrowFormation` overload boundary, and the separation between
lifetime rules and the type/compile overload pipeline. Borrow formation does
not require or manufacture construction Open. The semantic core now closes
LifeName/LifetimeValue/NameView, half-open Region, generations, cleanup,
Pre/Post summaries, lazy/coinductive origin, and monotone Color inheritance.
Concrete IR, checker implementation, summary compression, access-tree
integration, diagnostics, and any extended Horae logic remain future work.
