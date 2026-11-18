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
  -> single-stage positions / R_vis / C_sigma / ordinary overload
  -> invocation/result
  -> construction/migration
  -> instance move effects / with cleanup / lifecycle / unsafe admissions
  -> host Objects / physical normalization
  -> runtime entry / active dominance / evaluator projections
  -> restricted residual / internal completion / escape / optimizer boundary
  -> concrete Step/Wait / regular effect / ordinary handler and Drop
  -> heap support/duty submission independently of Stage migration
```

Current implementation coverage is recorded in
[`planning/roadmap.md`](planning/roadmap.md). A missing consumer is an explicit
frontier; it does not authorize an alternate semantic relation. The
[canonical conformance matrix](planning/canonical-semantic-conformance.md)
links canonical acceptance scenarios to owners and pending consumers.

For host implementation, read the [owner/namespace handoff](contracts/semantic-owner-namespace-graph.md),
[compile construction/cache boundary](contracts/compile-construction-boundary.md),
[LC1–LC13 consumers](contracts/lifetime-program-consumers.md) and
[shared action publication](contracts/semantic-values-and-extraction-interfaces.md#shared-action-and-lifecycle-handoff)
together with the [E host preservation chain](contracts/e-host-architecture-and-incremental-revalidation.md#11-connected-handoff-responsibilities).
The latter connects their evidence and invalidation duties; it supplies no new
semantic authority. This includes complete selection observations, fresh-compilation
effect accounting and separate rewrite-equivalence/new-position validation.
The host also preserves recoverable search branches, local O1/O2 objective
switching and a complete default route for each accepted checkpoint within a
future declared materialization family/target support domain.
[HA01–HA18](planning/canonical-semantic-conformance.md#e-host-preservation-and-incremental-revalidation-targets)
remain architecture acceptance targets, separate from executed substrate tests.

## Contracts

For Atomic work, follow the [ordinary recursive Pattern](design/patterns-overload/pattern-relations-observation-and-extraction.md#92-recursive-named-pattern-material),
[selected ordinary transform/observation](design/patterns-overload/operator-patterns-and-generative-declarations.md#7-ordinary-e-relative-transforms-and-readable-observation)
and [Atomic execution](design/lifetime/atomic-pattern-transitions-and-causality.md)
owners in that order. The [status ledger](planning/lifetime-concurrency-consolidation.md)
separates their definitions from pending source, proof and backend gates.

- [`raw-ast-contract.md`](contracts/raw-ast-contract.md)
- [`semantic-owner-namespace-graph.md`](contracts/semantic-owner-namespace-graph.md)
- [`control-flow-end-events.md`](contracts/control-flow-end-events.md)
- [`semantic-values-and-extraction-interfaces.md`](contracts/semantic-values-and-extraction-interfaces.md)
- [`lifetime-program-consumers.md`](contracts/lifetime-program-consumers.md)
- [`e-host-architecture-and-incremental-revalidation.md`](contracts/e-host-architecture-and-incremental-revalidation.md)
- [`pattern-root-identity-and-explicit-navigation.md`](contracts/pattern-root-identity-and-explicit-navigation.md)
- [`compile-construction-boundary.md`](contracts/compile-construction-boundary.md)
- [`policy-migration.md`](contracts/policy-migration.md)
- [Heap/IO/effect migration closure and acceptance](planning/heap-io-effect-migration.md)

The handoffs for [typed Pattern leaves](design/patterns-overload/pattern-relations-observation-and-extraction.md#external-typed-leaves), [mixed-stage invocation](design/static-evaluation/compile-instance-invocation-and-result-delivery.md#runtime-inputs-to-compile-producers),
[Atomic wrapper admission](design/lifetime/atomic-pattern-transitions-and-causality.md#n46--普通结构真实类型包装与表示预算-定义), [allocation-time Drop registration](design/lifetime/heap-stack-and-storage-responsibility.md#n28--分配成功时注册-lazy-drop-不变量)
and [committed Heap submission](design/lifetime/heap-stack-and-storage-responsibility.md#n29--relocate-精确向直接父层提交且不执行-pending-不变量) identify the canonical relations.
The [definition and consumer status](planning/lifetime-concurrency-consolidation.md#definition-and-consumer-status)
tracks their source wiring and the open memory-order choice.

The [qualified infix rule](design/patterns-overload/operator-patterns-and-generative-declarations.md#qualified-binary-operator-targets)
defines the explicit Path call target. The [recursive layer rule](design/patterns-overload/pattern-relations-observation-and-extraction.md#formation-and-relational-derivation)
uses ordinary structural formation and intermediate-layer extraction.

## Open questions

Only [`planning/open-questions.md`](planning/open-questions.md) may classify a
canonical representation or semantic decision as Open. Implementation wiring
of a closed relation belongs in the roadmap instead.

## Lifecycle/concurrency consolidation

Start with the [topic owner map](design/README.md), then the updated
[lifetime](design/lifetime/lifetime-policy-and-overload-boundary.md),
[heap](design/lifetime/heap-stack-and-storage-responsibility.md),
[atomic](design/lifetime/atomic-pattern-transitions-and-causality.md) and
[heterogeneous container](design/control-flow/heterogeneous-computation-containers.md) owners.
The [migration/proof/status ledger](planning/lifetime-concurrency-consolidation.md)
and [paired acceptance](planning/lifetime-concurrency-acceptance.md) separate
D/P/S/C/T/B and retain unconnected behavior as unavailable.
