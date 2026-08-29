# Entity-alias design history

This historical note records why the frozen `let n === q` syntax does not
create a semantic entity.

Earlier proposals modeled the form with forwarding Symbols, shared Places,
alias values, member forwarding, and alias-specific writability. Those
proposals coupled lexical spelling to runtime and residency identity and made
ordinary binding freshness ambiguous.

The canonical replacement is the local lexical mapping documented by
`spec/design/symbol-world/entity-alias-design.md`:

```text
Resolve_Gamma(q) = S
Gamma[n |->lex S]
```

The frozen Raw AST and parser diagnostics remain historical surface facts.
Semantic forwarding carriers and their algebra are not part of the current
language model. Git history before and during PR103 contains the detailed
proposal text and its implementation-era diagnostic inventory.
