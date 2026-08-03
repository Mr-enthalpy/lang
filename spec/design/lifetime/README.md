# Lifetime Design

This directory is the design boundary for lifetime-policy work.

Start with:

- [`lifetime-policy-and-overload-boundary.md`](lifetime-policy-and-overload-boundary.md)

That note is the canonical owner of the `@` operation: it defines the two
positively defined base groups (`LifetimeFact` for value-bearing objects and
`P ref` for carrier slots that are effectively open) plus the
target-preserving existing-view overlap, the escape check on borrow views, and
the separation between lifetime rules and the type/compile overload pipeline.
`Val1?` selects the base group; the `P ref` group's Open premise is checked on
the carrier that becomes the referent. Region/origin algebra, checking,
specificity, Horae logic, caching, diagnostics, and public syntax remain future
design work.
