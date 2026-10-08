# Canonical Semantic Design

The topic owners below define one semantic system. Surface preservation lives
under public, implementation handoffs under contracts, and consumer gaps and
local representation questions under planning. A Rust carrier is not evidence
that the corresponding source semantics is implemented.

## Topic owners

| Topic | Owner |
| --- | --- |
| Static evaluation, compile instances, open/close, NameExpr result shape, interpretation polarity, structural type formation, file/name production, dot/ADL | [static, name, and structural semantics](unified-static-name-and-structural-semantics.md) |
| NameValue, two-level Read, Path projection/indexing and opposite-context interpretation | [Path algebra](symbol-world/structured-path-algebra-and-interpretation-polarity.md) |
| General dependencies, semantic realization and one-time formation | [dependency realization](symbol-world/dependency-observation-and-realization.md) |
| Product/result extraction and direct delivery | [result extraction](patterns-overload/return-value-extraction-and-implicit-decomposition.md) |
| Callable tails and Pack boundaries | [callable material](patterns-overload/callable-tail-dot-name-and-pack-pattern.md) |
| Names, named-type synthesis, structural let, type/group algebra | [names and groups](symbol-world/names-and-overload-groups.md) |
| Complete pattern values, Core/whole equality, Places, borrows, literals | [pattern values and Places](symbol-world/type-values-places-and-borrow-views.md) |
| Structural formation, ordinary struct and * / *=, OpenHere | [construction](symbol-world/structural-type-formation-and-composition.md) |
| Source composition and construction closure | [composition](symbol-world/symbol-construction-units-and-namespace-origin.md) |
| Associated state A as a derived compile invocation | [associated state](symbol-world/associated-compile-state.md) |
| Closure anchored replication | [replication](symbol-world/closure-anchored-replication.md) |
| Single Stage, P1/P2/Pin/Pout, R_vis, demand and migration | [policy](symbol-world/policy-and-static-flow-projection.md) |
| Exact callee/self, ordinary C_sigma family and forwarding | [calling](symbol-world/function-object-call-model.md) |
| Proof-relevant Pattern relation and extraction | [Pattern relation](patterns-overload/pattern-values-relational-semantics-and-extraction.md) |
| Restricted Split/D, internal chain completion and residual escape | [extraction chains](patterns-overload/static-pattern-spaces-and-extraction-chains.md) |
| Pass action, move fixed point and directed with cleanup placement | [mechanical passing](mechanical-lowering/mechanical-argument-passing-and-move-fixed-point.md) |
| Requested-name producer surfaces, operator dispatch and OG_s | [Operator Patterns](patterns-overload/operator-patterns-and-generative-declarations.md) |
| Candidate pipeline and no reopen | [overload](patterns-overload/overload-resolution-design.md) |
| Compile instance identity, result delivery, opening sources and current storage | [invocation](static-evaluation/compile-instance-invocation-and-result-delivery.md) |
| Runtime main, active dominance, readiness, E projections and O1/O2 | [evaluation](static-evaluation/evaluation-residual-and-optimization.md) |
| Host IO and target-machine Objects | [host capabilities](static-evaluation/host-capabilities-and-machine-objects.md) |
| Instance Killable/MoveEffect/Movable, same-K @, Region and Color | [lifecycle](lifetime/lifetime-policy-and-overload-boundary.md) |
| SafetyPolicy, external admission and trusted semantic base | [unsafe admission](lifetime/unsafe-semantic-admission.md) |
| Serial unit/fallthrough, separate return-event/target inference and internal completion | [control flow](control-flow/targeted-return-and-d-reduction.md) |
| Level/main.lang anchor and PhysicalTree normalization | [physical source](build-package/build-system-design.md) |

Satellite documents consume these relations rather than redefine them. Existing
Core equality, Pattern normalization, identity, capture, policy migration and
lifecycle rules remain in force alongside the named-type and associated-state
algebras. If an unresolved contradiction is found, identify its exact premises;
do not create another ontology to reconcile it.

## Owner boundaries

The topic-owner index identifies semantic authority. Namespace graph records,
lexer Symbol tokens and SymbolicReferenceEdge have their separate carrier,
grammar and binder-reference roles. Rendered paths grant no semantic identity
or authority.

## Reading order

    static / name / structural semantics
      -> Object / complete pattern value / Place
      -> name existence and named-type / group algebra
      -> construction / OpenHere / anchored replication
      -> Pattern relation / policy / exact-self call / invocation-generated names
      -> dependency-derived openness / invocation residency and caches / A instance
      -> lifecycle / safety admission / host Objects
      -> physical normalization / shared E / residual and optimization

[Semantic spine](semantic-spine.md) supplies the compact dependency map.
[Conformance scenarios](../planning/canonical-semantic-conformance.md) record
canonical acceptance and source-consumer cases. [Roadmap](../planning/roadmap.md) describes actual consumer coverage;
[open questions](../planning/open-questions.md) contains only remaining choices.
Historical files are non-authoritative and are not rewritten for current rules.
