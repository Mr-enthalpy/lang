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

Projection-complete programs still have their own ordinary action premises.
Their [bootstrap justification](../lifetime/lifetime-policy-and-overload-boundary.md#91-bootstrap-dependencies-must-terminate)
must reach fixed language laws and available observations, not their own
still-unformed contract or companion. Boolean check success creates no world
authority. Safe admit components use the existing boundary's mapped projection
storage and all required premises before joint publication; transaction storage
alone cannot establish source correctness or authorize a supplied fact.

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

### 7.1 Whole-original-continuation CPS conversion (consumer pending)

Async's continuation requirement applies to the **entire original common
semantic continuation K**, including ordinary computation without a written
wait. Its semantic consumer must provide a complete continuation-passing (CPS)
account, rather than converting only wait-containing closure bodies. Here
original means the resolved semantic continuation, not Raw AST. CPS conversion
belongs after the relevant ordinary semantic resolution/selection and retains
unresolved future work as guarded material; it never executes future paths to
construct that account. A genuinely future invocation still selects at its own
actual frontier. Already sealed calls retain their origin.

```text
CPS(K, outward_delivery) => (K_cps, correspondence)
Obs(E(K)) = Obs(E(K_cps))
```

This is fidelity of one E/K under an admitted representation, not evaluation
twice or a second CPS evaluator. It is not an opportunity-seeking continuation
rewrite performed by E saturation. The correspondence covers action identities,
actual branch guards/order, selected frames, dependencies, name/cut projections,
activations/return targets, delivery consumers and fixed cleanup obligations.
Outward_delivery is the existing semantic delivery role; it is not another
user-written argument, self slot, Policy plane or new source ABI.

Let Q denote the pending ordinary context up to the applicable explicit handler
or outward delivery boundary. For an unhandled effect, the composition law is:

```text
Plug_Q(Done(v)) = Continue_Q(v)
Plug_Q(Wait(y,k)) = Wait(y, Compose_Q(k))
Step(Compose_Q(k), resume_args) ~= Plug_Q(Step(k, resume_args))
```

Continue_Q performs Q's existing normal delivery/completion checks, not an
unconstrained temporary result. Compose_Q retains both the callee remainder and
the pending caller suffix in the next concrete callable. Actual formation and
handoff require all ordinary dependencies and atomic Move[repair] premises;
this equation grants no move/copy permission. A handler that consumes an effect
delimits Q according to its ordinary selected behavior. Forwarding repeats this
composition only for the actual unhandled branch. Dropping the resulting
successor cleans all held callee/caller material exactly once through ordinary
Drop; already committed work is never replayed.

Coverage is comprehensive: expression/call input and Product evaluation,
let/assignment and result delivery, nested calls and their pending suffixes,
non-tail/tail serial work, guarded branches, admitted recursion/iteration,
handlers/resume/forwarding, target-return routing, and destructor/cleanup
continuations. Compile/runtime projections and reify's actual dependent E cone
use this same account, preserving ClosedReturn/ClosedInterface and the Ordinary
target-return barrier. It cannot be confined to an async-marked sublanguage,
runtime-only path, explicit await sites, or a native stack abandoned on Wait.

Conversion Pre requires actual common-continuation/control/delivery and selected
origin correspondence, with ordinary dependency/lifecycle/cleanup evidence for
every real transfer. Description conversion creates no business action or fact
by itself. Actual producer commits still use the common transaction and joint
Post; Post exposes only the complete valid mapped continuation and its real
responsibilities. Missing correspondence or an unsupported context reports a
spanned unavailable diagnostic at its requiring boundary. It cannot report
successful propagation with the caller suffix missing, invent a Residual, or
silently fall back to an effect-blind serial executor.

This semantic CPS obligation is closed; its common-E consumer and concrete IR
are pending. Ordinary source carriers and capture-gate tests do not implement it.
It neither prescribes a uniform Future/state-machine representation nor forces
Box, stack frames or one physical object per semantic edge. O may directly
generate larger legal slices under §11, including direct-style machine paths
where equivalent, without erasing observable repair/handler actions or inserting
unnecessary Move boundaries. Whole-K CPS and generation grain are separate
obligations.

## 8. Runtime reification of E

Callable P2 reify has outward runtime Policy and executes through the same E.
The [unified judgment](../unified-static-name-and-structural-semantics.md#20-reify-callable-p2-and-scoped-reverse-migration)
owns its syntax, ActiveBoundary, ClosedReturn, ClosedInterface and ReverseAllowed premises.

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

RequiredCapture is a specification, not an initialized source: a new invocation
fulfills it in its declared actual call context. Once bound/sealed it is retained
by every projection. The dependency owner defines this operation; no deferred
eager initialization or arbitrary body lookup follows from it.

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
<CommonContinuationMaterial with CPS correspondence,
 SelectedCall, NormalizedPassMoves, ReturnDeliveryTarget,
 ABIKind, LifetimeEvidence, MachineTargetFacts>
```

It must realize input-frontier transport, ABI1 overwrite/reinterpretation even
with changed output size, ABI2 designated-target delivery, stack-frontier
adjustment and proved tco/loop slot reuse. Cross-target and FFI constraints
need explicit machine boundaries preserving E's guarantees. Physical frame
allocation, calling encodings and FFI ABI are not implemented by this specification
revision. SCC lifecycle evidence is supplied by the independent lifetime work,
never chosen by machine layout or optimization success.

## 11. Atomic cuts and requested generation grain

The [effect owner](../control-flow/effect-steps-and-continuation-handoff.md)
defines Step/Wait. The template is the whole original K with the CPS account of
§7.1, including pending caller contexts, not an isolated callee body.
Serial n waits identify n+1 semantic atomic slices A_0..A_n
in the common continuation template. Those are not n+1 mandatory generated
Objects. A materialization request selects retained boundaries B from this
template and can generate a continuous G_(i,j) directly when ordinary handler/
resume and projection evidence justifies the internal handoffs. It need not
split objects then fuse them, or generate up to 2^n possible partitions.

```text
Legal(K,B1) and Legal(K,B2) and preserved observable internal actions
  => Obs(Generate(K,B1)) = Obs(Generate(K,B2))
```

Obs includes values, effect traces and admitted lifecycle/cleanup observations.
Generate is O/materialization of the same E/K, not a new evaluator. Actual Move/
repair obligations occur at actual retained boundaries. An inlined boundary
cannot invent an unnecessary Move. Observable repair/handler actions must still
execute with equivalent effects or force retention of the cut. Every affected
projection revalidates; finite regular shape supplies no arbitrary fusion proof.

`let coro = expression ?; coro return;` delivers concrete movable bare Product
continuation material under ordinary ABI1 and stack-growth-frontier semantics.
If ordinary Move/escape is legal, no implicit Box is needed. `? Box` explicitly
introduces storage/pool dependencies. Legal pool submission can discharge those
local storage duties before return; it does not change object storage positions.
`val =` uses ordinary ExistingTarget ABI2. These syntax examples are pending
source acceptance targets. Async, boxing and Function Pattern projection are
independent choices. Neither return nor async instructs O to infer heap escape,
allocate a box, generate a uniform Future/state-machine type, or choose a special
async ABI. The existing ordinary materialization inputs in §10.1 suffice.
