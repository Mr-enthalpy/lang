# Canonical semantic design

`spec/design/` contains the normative topic owners for semantic relations that
extend beyond the normalized surface. Current frontend behavior remains under
`spec/public/`; implementation handoffs belong under `spec/contracts/`; open
representation and sequencing questions belong under `spec/planning/`.

## Topic owners

| Topic | Normative owner |
|---|---|
| Object, complete type values, Place, borrow views, abstract literals, `@` | `symbol-world/type-values-places-and-borrow-views.md` |
| Symbol-first construction, `struct -> tau`, `extend`, `inject`, OpenHere | `symbol-world/symbol-first-meta-construction-and-pattern-injection.md` |
| namespace origin and construction ownership | `symbol-world/symbol-construction-units-and-namespace-origin.md` |
| PolicyPair, PolicyMode, demand, migration, capability and compile projection | `symbol-world/symbol-policy-and-compile-flow-projection.md` |
| value/type/associated-call projection | `symbol-world/function-object-call-model.md` |
| relational Pattern semantics and extraction | `patterns-overload/pattern-values-relational-semantics-and-extraction.md` |
| overload pipeline and no reopen | `patterns-overload/overload-resolution-design.md` |
| invocation frame, result classes and residual boundary | `meta-invocation/meta-object-invocation-and-policy-reduction.md` |
| SemanticContinuation, lifetime, Region, Color and access boundary | `lifetime/lifetime-policy-and-overload-boundary.md` |
| package/build projection into the namespace graph | `build-package/` |

Satellite documents may describe consumers and interfaces but do not redefine
the owner’s relation. If two current owners conflict, record the conflict; do
not infer a resolution from implementation carriers.

## Reading order

For semantic evaluator work:

```text
Object / complete tau / Place
  -> Symbol and owner graph
  -> relational Pattern
  -> Policy and overload
  -> invocation/result
  -> construction/migration
  -> lifecycle
```

For current implementation sequencing, read `spec/planning/roadmap.md` after
the relevant topic owners. Historical alternatives are stored only under
`spec/history/` and are not prerequisites for the current model.

## Blocks

| Block | Responsibility |
|---|---|
| `build-package/` | package identity, roots, mounts, namespace contributions and cache provenance |
| `symbol-world/` | Object/type/Symbol/Place/Policy/construction relations |
| `patterns-overload/` | Pattern relation, structural extraction and candidate selection |
| `meta-invocation/` | invocation frames, declared results and residualization |
| `policy-capability/` | focused Policy and capability reference |
| `lifetime/` | continuation-relative lifecycle and access validation |
| `mechanical-lowering/` | source-expressible pass algebra and lowering obligations |

Implementation status is recorded as `Implemented`, `Consumer pending`,
`Open`, or `Future` in the roadmap. A Rust carrier alone is never evidence that
its relation is implemented.
