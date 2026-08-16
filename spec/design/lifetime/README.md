# Lifetime Design

This directory is the design boundary for lifetime-policy work.

Start with:

- [`lifetime-policy-and-overload-boundary.md`](lifetime-policy-and-overload-boundary.md)

That note is the canonical owner of the `@` operation: `@` is a privileged
place-observation builtin that yields a lifetime value (`LifetimeVal`), never a
borrow view and never a `type ref`. The former two instance groups
(`Val1?(x) ≠ null -> LifetimeFact`, `Val1?(x) = null -> P ref`), the
carrier-slot form `t@ : type ref`, and the borrow-type fixed points
(`type ref@ = type ref`, `type share@ = type share`) are retired. `ref` and
`share` are the borrow constructors; each is a privileged actual-place builtin
(`PrivilegedActualPlace(ref-family)` / `PrivilegedActualPlace(share-family)`)
that may obtain the actual's place, while an ordinary user function cannot. Explicit higher-level selection uses `t |> type ref` /
`t |> type share`. The note also keeps target-preserving `ref`/`share`
constructor composition, the escape check on borrow views, the
`NoImplicitBorrowFormation` overload boundary, and the separation between
lifetime rules and the type/compile overload pipeline. Borrow formation does
not require or manufacture construction Open. The full `@` lifetime algebra —
region representation, `LifetimeVal` shape, and ordering — is deliberately left
unfrozen; region/origin algebra, checking, specificity, Horae logic, caching,
diagnostics, and public syntax remain future design work.
