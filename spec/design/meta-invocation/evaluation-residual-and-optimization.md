# Evaluation, Residual Continuations, and Optimization

Status: canonical semantic boundary. The complete evaluator and optimizer
consumers are pending; implementation phases do not define additional semantics.

## 1. Compilation entry

```text
L = Normalize(PhysicalTree(Level))
main.P2 = runtime
omitted ordinary main.P1.stage = runtime
K_entry = EntryContinuation(L, F_main)
Compile(Level) = Materialize(Pi_machine(Residual(E(K_entry))))
```

Level selects the compilation root anchored by main.lang. File statements
retain source order; sibling files are unordered contributions/actions and
main.lang receives no execution priority. Physical normalization installs no
semantic owner or authority.

Existing bootstrap or independently legal ordinary meta formation establishes
stable roots. Root ownership does not imply an active meta frame around
K_entry. Actual meta/seal frames impose their stack-relative dominance only
during their own activity, including through compile helpers.
[Build normalization](../build-package/build-system-design.md) and
[meta entry](meta-object-invocation-and-policy-reduction.md#21-compilation-entry-and-root-formation)
supply the ordinary source/owner handoffs; host capabilities return ordinary
Objects and add no build-side semantic facts.

## 2. Semantic saturation

At the current continuation structure, stage, policy, and known facts, E
executes every semantically ready action:

    E E = E
    Ready_E(M E) = empty

This is saturation of the current legal frontier, not a promise that all future
runtime computation disappears. E does not change continuation structure to
search for more compile-time opportunities. In particular,

    runtime let r = 1uint8

does not become a compile binding merely because its value is known. Existing
language-defined derivations, including compile companions and policy migration,
retain their own rules; their existence grants no discretionary stage rewrite.

OpenStatic, SealStatic, and Runtime describe readiness/exposure in this one
evaluation. A runtime continuation retains resolved names, selected members,
and sealed invocations. Runtime value control does not reopen resolution.

## 3. Synchronous semantic projections

    E = tensor_product over pi of E_pi

The product synchronizes the projections of the same continuation action.
Machine, type, policy, and lifetime observations share action identity and
continuation position. They are not independent semantic IRs reconciled later.

    R_machine = pi_machine(R)
    R_lifetime = pi_lifetime(R)

Every affected projection checks its Pre before the action commits. Failure of
any Pre rejects the action before mutation. Post describes successful commit
only. Cleanup is fixed by the existing control/ownership/end-event semantics
before lifecycle observation; lifetime checking does not move cleanup to make
a constraint succeed.

InvocationResult remains the single result envelope. A residual carrier is a
representation of the remaining common continuation, not a separate result
ontology or an evaluator with private interpretation rules.

## 4. Two optimization objectives

O1 transforms an equivalent continuation to expose a new legal E frontier:

    L E (O1 E)*

Each E must complete its semantic saturation. After any such E, the planner may
stop; there is no requirement to reach a global optimization fixed point.

O2 lowers the cost of residue already accepted for runtime execution: work,
storage, fusion, scheduling, memory reuse, latency, and code size. The idealized
objective separation is:

    L E (O1 E)* (O2 ; E_affected)*

An actual transform can contain both components. Any newly exposed E work
returns to E; the objective classification does not exempt O2 from semantic
validation. E_affected synchronizes every affected projection on the rewritten
continuation through its Pre/commit/Post checks; it is not an optimizer-owned
validator. Newly ready work still returns to a complete E saturation.

## 5. Facts belong to E

    E owns meaning
    O owns equivalent rewrites
    Planner owns search

Every semantic premise used by a transform is a fact of the ordinary language
world. Associativity, commutativity, non-aliasing, representation, machine,
lifetime, and policy facts are queried from E's projections. There are no
optimizer-private assumptions or semantic gaps interpreted as permissions.

Operator laws have three distinct consumers: grammar ParseAssociativity fixes
AST grouping; SemanticLaw_E fixes the operator's relation; RewriteLaw_O supplies
an E-proved equivalence for a continuation rewrite. Trait-like law queries are
ordinary meta results. A commutative Pattern must already be unordered in E's
interpretation; O cannot make it unordered by consulting a later trait query.
See [operator relations](../patterns-overload/operator-patterns-and-generative-declarations.md).

    Facts_E proves R equivalent_to R'

Old continuation facts can guide generation of a rewrite candidate. They cannot
be transplanted to the rewritten continuation. Reordering, duplication,
elimination, fusion, replacement, storage reuse, and lifetime shortening must
all undergo the affected E_pi Pre/commit/Post checks at the new positions.

The materializer likewise consumes established machine and representation facts.
An ABI choice that changes program meaning must be represented in those facts;
implementation layout/search choices cannot become another semantic authority.

## 6. Planner parameters and external descriptions

Future optimization parameters may change search budget, cost model, strategy,
and scheduling of O1/O2. They cannot change E, language legality, stage rules,
dependencies, target facts, or the semantic premises available to evaluation.

Precise external descriptions enter E through ordinary host results or
[unsafe admissions](../lifetime/unsafe-semantic-admission.md). Once admitted,
their provenance does not impose an extra optimizer opacity boundary. Rewrites
remain ordinary consequences of the established facts.

## 7. Consumer handoff

A Simple Serial Semantic Evaluation consumer executes normalized actions, checks
the common projections, commits, delivers results/completions, and transports
residue. It references the established name, construction, policy, lifecycle,
host, and physical normalization laws. It does not repair or redefine them.
Concrete residual frames, effect summaries, storage, and planner algorithms
remain representation/implementation work.

## 8. Seal readiness and scheduling

P1 seal formation can remain a pending continuation/Place obligation with no
resident yet. Compile work may admit that input and defer until it is ready;
this is not a stage conversion. Ready includes actual read, write, borrow,
lifetime and effect dependencies as well as formation order. Moving the work
to a convenient scheduler queue cannot change those dependencies.

MetaDom forbids seal entry and seal let; SealDom forbids meta invocation and
meta let, including cache hits. Completed meta payloads are ordinary readable
inputs when independently legal. Wpre/Wseal are snapshots of this one
continuation; privileged scans keep their fixed Wpre domain.

All legal ready schedules preserve observable results, instance/call origin,
cleanup and effects. Scheduler traces do not become identity. Saturation is a
semantic fixed point for ready work, not a guarantee that infinite static work
terminates. Representations and scheduling algorithms remain implementation
frontiers.


## 9. Dependency realization at formation

[General dependencies](../symbol-world/dependency-observation-and-realization.md)
separate required observations, semantic realization and representation.
Snapshot versus live reference, readiness and one-time initializer effects are
semantic facts, never scheduler/layout choices. Ready initializers run once
per formation in ordinary effect order, with the specified common pre-capture
name environment. Projecting type/call/Pattern or residualizing completed
material does not recapture. Missing readiness cannot be justified by speculative
execution of the body that consumes it. Closure dependency persistence passes
through the [lifetime handoff](../lifetime/lifetime-policy-and-overload-boundary.md#8-closure-dependency-lifetime-refinement-handoff).
