# Lifetime Design

This directory is the design boundary for lifetime-policy work.

Start with:

- [`lifetime-policy-and-overload-boundary.md`](lifetime-policy-and-overload-boundary.md)

That note is the canonical owner of the `@` operation: it defines the two
positively defined `@` overload groups (`LifetimeFact` for value-bearing objects,
`P ref` for effectively open pattern-value slots), the escape check on borrow
views, and the separation between lifetime rules and the type/compile overload
pipeline. Region/origin algebra, checking, specificity, Horae logic, caching,
diagnostics, and public syntax remain future design work.
