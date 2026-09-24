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
- distinct NameBinding, named type, OverloadGroup, Place, resident generation,
  and lookup IDs;
- `PolicyPair`, primitive `PolicyMode = {const, plain, mut}`, capability
  realization, and post-selection DynamicLegality;
- one resolved target, R_vis evidence, ordinary C_sigma realizations and sealed
  selection shared by static projections and runtime residue;
- unique sealed invocation, no reopen, and unified `InvocationResult`;
- candidate-driven same-Type Policy migration;
- construction authority, `OpenHere`, Writable, `extend`, and `inject`;
- continuation-relative lifecycle facts, Region generations, Pre/Post, and an
  extensible directed Color algebra;
- meta instance names/types, P1 meta retention versus plain completion/closure,
  dependency-derived openness, ordinary Val2 payloads, generic instance caching,
  derived associated state A, and witnessed closure re-instantiation;
- unsafe semantic axiom admission and ordinary host-capability Objects;
- source-only namespace construction, unordered physical normalization, and
  one evaluator E with synchronized projections and validated optimization.

Stage is a single atom. P2 is horizon, Pin admits explicit stage constraints,
and Pout follows P1. Runtime main enters one E/K; active meta/seal dominance
is independent of stable root history. Instance move effects and directed with
cleanup feed the same lifecycle observations. Done is internal completion;
residual escape separately consumes ordinary meta facts.

Every legal completed closure expression produces full tau_C through ordinary
struct. File implementation-layer let installs at the established package root;
true lexical local let remains binding. Explicit structural contributions synthesize named types;
OverloadGroup is the separate aggregation algebra. Naked operators select
operator[op], with spelling retained by the ordinary OG_s family. Current Rust carriers do not yet implement every
closed relation; the roadmap records those consumer gaps.

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
- [Canonical conformance scenarios](spec/planning/canonical-semantic-conformance.md)
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


Ordinary calls enter through Type(x)'s associated Val2[()] with self=x.
Type calls first select c from their own V_tau, then enter through Type(c)'s
associated Val2[()] with self=c. V_tau, Val2 residency, Pattern registration
and ConstructEdge remain independent. In-place syntax forms dependencies
automatically; its completed result supports ordinary value operations whose
legality depends on actual dependencies, access, capabilities and lifecycle.
Invocation does not recapture or resolve external names again by spelling.
