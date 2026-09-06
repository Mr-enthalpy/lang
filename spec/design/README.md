# Canonical Semantic Design

The topic owners below define one semantic system. Surface preservation lives
under public, implementation handoffs under contracts, and consumer gaps and
local representation questions under planning. A Rust carrier is not evidence
that the corresponding source semantics is implemented.

## Topic owners

| Topic | Owner |
| --- | --- |
| Names, named-type synthesis, structural let, type/group algebra | [names and groups](symbol-world/names-and-overload-groups.md) |
| Complete pattern values, Core/whole equality, Places, borrows, literals | [pattern values and Places](symbol-world/type-values-places-and-borrow-views.md) |
| Meta identity, root/return seal, struct, extend/inject, OpenHere | [construction](symbol-world/symbol-first-meta-construction-and-pattern-injection.md) |
| Source composition and construction closure | [composition](symbol-world/symbol-construction-units-and-namespace-origin.md) |
| Associated guarded compile state A | [associated state](symbol-world/associated-compile-state.md) |
| Closure anchored replication | [replication](symbol-world/closure-anchored-replication.md) |
| PolicyPair, PolicyMode, demand, migration, capability and stages | [policy](symbol-world/symbol-policy-and-compile-flow-projection.md) |
| Exact callee/self, ordinary function objects and forwarding | [calling](symbol-world/function-object-call-model.md) |
| Proof-relevant Pattern relation and extraction | [Pattern relation](patterns-overload/pattern-values-relational-semantics-and-extraction.md) |
| Candidate pipeline and no reopen | [overload](patterns-overload/overload-resolution-design.md) |
| Invocation/result and meta partner identity | [invocation](meta-invocation/meta-object-invocation-and-policy-reduction.md) |
| E saturation, residual, synchronous projections, O1/O2/planner | [evaluation](meta-invocation/evaluation-residual-and-optimization.md) |
| Host IO and target-machine Objects | [host capabilities](meta-invocation/host-capabilities-and-machine-objects.md) |
| Continuation-relative lifecycle, Region, Color and access | [lifecycle](lifetime/lifetime-policy-and-overload-boundary.md) |
| SafetyPolicy, external admission and trusted semantic base | [unsafe admission](lifetime/unsafe-semantic-admission.md) |
| Outermost return, D reduction and completion handoff | [control flow](control-flow/targeted-return-and-d-reduction.md) |
| Level/main.lang anchor and PhysicalTree normalization | [physical source](build-package/build-system-design.md) |

Satellite documents consume these relations rather than redefine them. Existing
Core equality, Pattern normalization, identity, capture, policy migration and
lifecycle rules remain in force alongside the named-type and associated-state
algebras. If an unresolved contradiction is found, identify its exact premises;
do not create another ontology to reconcile it.

## Reading order

    Object / complete pattern value / Place
      -> name existence and named-type / group algebra
      -> construction / OpenHere / anchored replication / A
      -> Pattern relation / policy / exact-self call / invocation result
      -> lifecycle / safety admission / host Objects
      -> physical normalization / shared E / residual and optimization

[Semantic spine](semantic-spine.md) supplies the compact dependency map.
[Roadmap](../planning/roadmap.md) describes actual consumer coverage;
[open questions](../planning/open-questions.md) contains only remaining choices.
Historical files are non-authoritative and are not rewritten for current rules.
