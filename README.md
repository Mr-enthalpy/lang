# lang

`lang` contains a syntax-preserving frontend and the canonical semantic
substrate used by the build/evaluation layer.

```text
source text
  -> tokens
  -> Raw AST
  -> Normalized AST
  -> typed owner and namespace resolution
  -> canonical semantic evaluation
  -> SemanticEntity / SemanticView
  -> InvocationResult
```

Raw AST preserves source shape and recovery. Normalized AST performs
syntax-directed lowering while keeping value-side and Pattern-side material
separate. It is not HIR and does not resolve names, select overloads, validate
lifetimes, or execute code.

The semantic layer is organized around these independent coordinates:

- `Object = <Val1?, Pattern, Val2>` and complete ordinary normalization;
- relational Pattern interpretation `R_Gamma(P,c,rho)`;
- complete type values `tau = bind alpha.<Core(tau), V_tau[alpha]>`;
- distinct Symbol, semantic value, Place, resident generation, and lookup IDs;
- `PolicyPair`, primitive `PolicyMode = {const, plain, mut}`, capability
  realization, and post-selection DynamicLegality;
- one name-resolution result followed by value/type/call projection;
- unique sealed invocation, no reopen, and unified `InvocationResult`;
- candidate-driven same-Type Policy migration;
- construction authority, `OpenHere`, Writable, `extend`, and `inject`;
- continuation-relative lifecycle facts, Region generations, Pre/Post, and an
  extensible directed Color algebra.

## Workspace

```text
crates/lang_syntax   lexer, parser, Raw AST, normalization, diagnostics
crates/lang_build    namespace graph and canonical semantic substrate
crates/lang_cli      token/AST/normalized/diagnostic inspection
spec/public          current normalized-surface documentation
spec/contracts       current implementation handoffs
spec/design          canonical semantic topic owners
spec/planning        current implementation frontiers and open questions
spec/history         non-authoritative snapshots and design history
tests                frontend golden tests
```

## Documentation

- [Specification index](spec/README.md)
- [Normalized surface](spec/public/normalized-surface-semantics.md)
- [Raw AST contract](spec/contracts/raw-ast-contract.md)
- [Canonical semantic owners](spec/design/README.md)
- [Implementation roadmap](spec/planning/roadmap.md)
- [Open questions](spec/planning/open-questions.md)
- [Glossary](spec/reference/glossary.md)

Documents under `spec/history/` have no current semantic authority and are not
required to understand the active architecture.

## Development

Use Rust stable.

```bash
cargo fmt --all
cargo test
```

The lexer remains weak: contextual language words are ordinary `Name` tokens.
The parser owns syntax shape, not semantic meaning. Closed canonical relations
belong in their topic owners; genuinely unresolved representation questions
belong in `spec/planning/open-questions.md`.
