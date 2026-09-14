# Specification index

The active specification is organized by responsibility.

## Current authority

| Area | Location | Responsibility |
|---|---|---|
| Public surface | [`public/`](public/) | source-to-Normalized-AST behavior |
| Contracts | [`contracts/`](contracts/) | implementation handoffs and explicit pending consumers |
| Canonical design | [`design/`](design/) | semantic topic owners |
| Reference | [`reference/`](reference/) | shared terminology |
| Planning | [`planning/`](planning/) | consumer frontiers and open questions |

`history/` stores non-authoritative snapshots. Active documents are
self-contained and do not depend on a historical document to define current
behavior.

## Frontend

Read in this order:

1. [`public/normalized-surface-semantics.md`](public/normalized-surface-semantics.md)
2. [`public/agent-interpretation-guide.md`](public/agent-interpretation-guide.md)
3. [`contracts/raw-ast-contract.md`](contracts/raw-ast-contract.md)

The frontend pipeline is:

```text
source text -> weak tokens -> Raw AST -> Normalized AST (+ diagnostics)
```

Raw AST preserves syntax and recovery. Normalized AST is non-semantic and
keeps value-side expressions distinct from Pattern-side material.

## Semantic evaluator

Start with [`design/README.md`](design/README.md), which maps every closed
semantic concept to one normative topic owner. The shortest dependency order
is:

```text
Object / complete tau / Place
  -> names / named type / OverloadGroup / owner graph
  -> relational Pattern
  -> Policy and overload
  -> invocation/result
  -> construction/migration
  -> lifecycle / unsafe admissions
  -> host Objects / physical normalization
  -> evaluator projections / residual / optimizer boundary
```

Current implementation coverage is recorded in
[`planning/roadmap.md`](planning/roadmap.md). A missing consumer is an explicit
frontier; it does not authorize an alternate semantic relation.

## Contracts

- [`raw-ast-contract.md`](contracts/raw-ast-contract.md)
- [`semantic-owner-namespace-graph.md`](contracts/semantic-owner-namespace-graph.md)
- [`control-flow-end-events.md`](contracts/control-flow-end-events.md)
- [`semantic-values-and-extraction-interfaces.md`](contracts/semantic-values-and-extraction-interfaces.md)
- [`pattern-root-identity-and-explicit-navigation.md`](contracts/pattern-root-identity-and-explicit-navigation.md)
- [`meta-construction-boundary.md`](contracts/meta-construction-boundary.md)
- [`policy-migration.md`](contracts/policy-migration.md)

## Open questions

Only [`planning/open-questions.md`](planning/open-questions.md) may classify a
canonical representation or semantic decision as Open. Implementation wiring
of a closed relation belongs in the roadmap instead.
