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

Bootstrap or independently legal ordinary formation establishes stable roots.
Root ownership, active evaluation and P2 remain independent. CompileInstance
formation is compatible with compile horizons. [Build normalization](../build-package/build-system-design.md)
provides inherited navigation; [invocation](compile-instance-invocation-and-result-delivery.md)
supplies stable identity, body entry and ordinary result delivery. Host
capabilities return ordinary Objects and add no build-side semantic facts.

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

OpenStatic and Runtime describe readiness/exposure in this one
evaluation. A runtime continuation retains resolved names, selected members,
and sealed invocations. Runtime value control does not reopen resolution.

## 3. Synchronous semantic projections

    E = tensor_product over pi of E_pi

The product synchronizes the projections of the same continuation action.
Machine, type, policy, and lifetime observations share action identity and
continuation position. They are not independent semantic IRs reconciled later.

The lifecycle projection receives this complete continuation and forgets the
business Policy Stage coordinate. It retains selected operations, MoveEffect,
names, dependencies, guarded control, activations and result delivery; erasure
does not select another operation or execute both arms of unknown control.
Established names and structure can participate before machine execution reaches
them, while unknown business values remain unknown. `lifetime pre/post` programs
are already lifecycle material: execute their ordinary computations and boundary
calls, but do not project them again or create second `@`/partner layers. The
[lifetime owner](../lifetime/lifetime-policy-and-overload-boundary.md#211-one-semantic-continuation)
and [LC2 handoff](../../contracts/lifetime-program-consumers.md#lc2-whole-continuation-projection-and-projection-complete-clauses)
define the projection-complete boundary and its fidelity obligations.

    R_machine = pi_machine(R)
    R_lifetime = pi_lifetime(R)

Every affected projection checks its Pre before the action commits. Failure of
any Pre rejects the action before mutation. Post describes successful commit
only. Cleanup is fixed by the existing control/ownership/end-event semantics
before lifecycle observation; lifetime checking does not move cleanup to make
a constraint succeed.

Every action checks fixed cleanup strictly before its cut, including actions
without a lifecycle projection. Outstanding cleanup blocks commit; unavailable
cleanup observations supply no permission. An established Drop or Kill can
discharge its generation's obligation. Preserve cannot. Same-cut endpoint and
ending-event precedence still belong to ordinary lifecycle Pre. This gate uses
the current continuation and projection facts before any affected Pre or Post.

InvocationResult remains the single result envelope. A residual carrier is a
representation of the remaining common continuation, not a separate result
ontology or an evaluator with private interpretation rules.

    ResidualAt(H) does_not_imply Stage = runtime
    PendingContinuation does_not_imply RuntimeResidual
    NoTargetValues does_not_imply ResidualAt(H)
    NoFullyAdmissibleCandidate does_not_imply ResidualAt(H)

Residual provenance requires an established remaining-continuation fact. An
exact immutable empty V_tau is a completed no-candidate failure; fully observed
non-callability or applicability/Policy mismatch is ordinary failure as well.
A consumer cannot infer hidden work from the shape of either failure. Only the
consumer that observes an actual horizon obstruction can report that frontier;
an outer initializer transports it without guessing its cause or producer stage.

An observation obstruction records an unavailable observation at H. It is
distinct from residual material: without an established remaining continuation,
the obstruction crosses the consumer boundary as an unavailable diagnostic.
An unsupported expression or unconnected producer supplies neither fact.

Failure to complete at an observation horizon establishes no producer stage.
Pending continuation work remains in the common continuation with its existing stage,
identity, inputs, effects and dependencies. A written runtime result demand
does not itself supply a producer or a continuation. If their preservation
consumer is unavailable, a binding consumer reports that frontier before
installing a resident or result Policy; it cannot fabricate a runtime result
observation as a substitute.

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
ordinary compile results. A commutative Pattern must already be unordered in E's
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

## 8. Runtime reification of E

Callable P2 reify has outward runtime Policy and executes through the same E.
The [unified judgment](../unified-static-name-and-structural-semantics.md#20-reify-callable-p2-and-scoped-reverse-migration)
owns its syntax, ActiveBoundary, ClosedReturn and ReverseAllowed premises.

```lang
let f = (self, runtime let args):reify -> _:R$ => {
    compile let v = args;
    ...;
};
```

For every admitted input, TypeContract_B(inputs) is determined without a selected
reverse migration. Ordinary return delivery checks the established Pattern/Pout;
an explicit contract and input-derived contract are equally valid when closed.

```text
Invoke(B,args) ~= Deliver_(ReturnPattern_B,Pout_B)(E_K(Body_B,args))
P2(B)=reify => OutwardHorizonStage(B)=runtime
ReverseAllowed_K(m) => RuntimeMaterialize_K(DependentSuccessors(m))
```

The equivalence also requires ordinary InputAdmissible, dependency validity,
actual frontier availability and common Pre/commit/Post. It does not execute a
runtime value at an earlier physical compile frontier. Missing input retains
only an established remaining continuation, never work guessed from a lookup miss.

An actual runtime-value-to-compile-type edge requires carrying its dependent E
cone. Unrelated compile type actions continue normally. Runtime-value actions
remain ordinary runtime continuation work. A value-only compile detour may be
rewritten by O after proving absence of type dependence; affected E projections
are revalidated. Deferral neither wraps unrelated successors nor recolors an
enclosing caller's fixed return classifier.

Runtime-policy types are values of the existing type algebra. Executing type
operations on runtime-only observations requires scoped reverse migration; no
parallel runtime type interpreter follows from storage availability.

Selected invocations retain identity. A genuinely new runtime invocation first
selects at its actual frontier. E does not enumerate future branches or the
Cartesian product of multi-dispatch. An interpreter carrying restricted E,
AOT, JIT, caches and precomputed tables are equivalent O representations.

The lifecycle projection remains ordinary compile-hosted value computation over
the complete current K, with business Stage forgotten as in §3. It is not split
into compile/runtime lifecycle worlds or elevated to a higher type rank.
Region_K(n)=R(NamePosition_K(n)); physical availability and future branch values
do not redefine those names or Regions. O's path elimination still requires E O E
revalidation, and adds no E admission proof obligation.

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

## 10. E-selected return delivery convention

This owner defines the semantic ABIKind from an established result consumer:

```text
ABIKind(Deliver) = ABI1 if Deliver=FreshLet        // let v = express
ABIKind(Deliver) = ABI2 if Deliver=ExistingTarget // v = express
```

ExistingTarget includes an already designated uninitialized hole/Place.
The distinction is the existing let/assignment consumer, not new syntax or
an opportunity discovered by optimization. Plain expressions, nested return
delivery and other positions first obtain their target through existing E
consumer rules, then map to the defined convention. Missing target wiring
is unavailable; O cannot invent a third selection rule or a hidden lexical temp.

ABI1 transports all actual implicit and explicit input through ordinary
Pass=Move to the normalized call growth frontier. Return overwrites/reinterprets
that input frame as the complete output. The output may differ in size: the
growth frontier is interpreted by the complete output type after completion.
ABI2 uses the same input Move semantics and delivers return to the previously
designated target Place. Initialization/replacement authority, aliasing,
Pre/commit/Post, complete-Object atomicity and cleanup remain independent checks.

E fixes ABIKind, the result-target relation and observable lifecycle obligations.
It does not fabricate physical addresses or slot widths. O and the materializer
may choose equivalent direct construction, local temporaries, transport and
machine layouts while satisfying that convention. RVO/NRVO success or failure
cannot select or switch ABIKind. No layout observation adds an Object coordinate
or changes type equality. Equivalent implementations revalidate affected E
projections as in §5.

ABI1 supplies an input/return-frontier contract for legal tail-call reuse.
Actual tco/loop also requires tail position, Move/borrow/lifetime, fixed cleanup
and compatible frame/slot/result evidence. Recursion alone proves no constant
stack bound.

### 10.1 Separate machine materialization work

A later machine PR consumes at least:

```text
<SelectedCall, NormalizedPassMoves, ReturnDeliveryTarget,
 ABIKind, LifetimeEvidence, MachineTargetFacts>
```

It must realize input-frontier transport, ABI1 overwrite/reinterpretation even
with changed output size, ABI2 designated-target delivery, stack-frontier
adjustment and proved tco/loop slot reuse. Cross-target and FFI constraints
need explicit machine boundaries preserving E's guarantees. Physical frame
allocation, calling encodings and FFI ABI are not implemented by this specification
revision. SCC lifecycle evidence is supplied by the independent lifetime work,
never chosen by machine layout or optimization success.
