# PR103 semantic migration history

This directory records implementation history for the canonical semantic
reset. Current language meaning is documented under `spec/design/`; current
sequencing is documented by `spec/planning/roadmap.md`.

PR103 replaced several implementation-era parallel models with one positive
semantic vocabulary. Its commit history contains the detailed carrier-by-
carrier audit, cut-over order, adapter removal, and death-test record.

The lasting engineering lesson is the distinction among:

```text
semantic authority
storage or execution material
source-operation wiring
open representation
```

A relation is implemented only when the relevant production consumer uses its
canonical authority. A pending consumer does not license a substitute ontology.

Archived migration-era documents in this directory preserve the detailed
implementation sequence and superseded carrier vocabulary:

- `entity-alias-history.md`;
- `static-pattern-spaces-and-extraction-chains-history.md`;
- `pattern-normalization-and-first-order-overload-history.md`;
- `overload-resolution-design-history.md`;
- `meta-object-invocation-and-policy-reduction-history.md`;
- `v0.6-cross-policy-value-transition.md`;
- `v0.8-symbolic-construction-values-and-extraction-interfaces.md`;
- `v0.8-meta-construction-agent-constraints.md`;
- `v0.9-pattern-head-identity-and-explicit-navigation.md`.
