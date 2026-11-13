# E host architecture and incremental revalidation

Status: host implementation handoff; no new language semantics or
connected-consumer claims. HA01–HA18 below are normative architecture acceptance
targets, not executed tests. The audited `main` is
`e235798a28ecfe1c632243c7a43d4382d1d8edd0` (2026-11-10), identical to the task
brief's baseline. Implementation evidence is separated in §11.

## 1. Scope and authority

MUST, MUST NOT and MAY constrain the host's preservation, validation and handoff
of existing semantics. They define no additional language judgment. Each
requirement below names its basis and required observations; §10 supplies the
common rejection/unavailable/publication rule whenever those observations are
false or missing.

| Subject | Existing authority consumed by this contract |
|---|---|
| One E, synchronized projections, rewrite validation, original-K CPS and delivery ABIKind | [E owner](../design/static-evaluation/evaluation-residual-and-optimization.md), especially §§2–5, 7.1 and 10–11 |
| Static/name/structural identity, reify and current name position | [Unified owner](../design/unified-static-name-and-structural-semantics.md), especially §§1–3 and 18–21 |
| Lifecycle projection, four-way contracts, bootstrap, summaries and supported recursive families | [Lifecycle owner](../design/lifetime/lifetime-policy-and-overload-boundary.md), especially §§2.1, 8.5, 9–11 and 14; [LC1–LC13 handoff](lifetime-program-consumers.md) |
| Sealed invocation, self/Param/activation and one input transfer | [Call owner](../design/symbol-world/function-object-call-model.md#62-consumer-directed-self-projections), §§6.2–6.3 and 8; [ordinary selection](../design/patterns-overload/overload-resolution-design.md) |
| Dependency requirements, selected realization and once-only formation | [Dependency owner](../design/symbol-world/dependency-observation-and-realization.md), §§2–5 |
| Compile identity, complete snapshots, current storage and reuse checks | [Invocation owner](../design/static-evaluation/compile-instance-invocation-and-result-delivery.md), §§1–6; [owner contract](semantic-owner-namespace-graph.md) |
| Complete values, resident/borrow generations and construction authority | [Type/Place owner](../design/symbol-world/type-values-places-and-borrow-views.md); [construction owner](../design/symbol-world/structural-type-formation-and-composition.md) |
| Fixed cleanup and selected producer actions ending in Move | [Mechanical passing owner](../design/mechanical-lowering/mechanical-argument-passing-and-move-fixed-point.md), §§0, 9 and 15 |
| Wait/repair, pending caller context, ownership and cleanup handoff | [Effect owner](../design/control-flow/effect-steps-and-continuation-handoff.md), §§2–3; [heap owner](../design/lifetime/heap-stack-and-storage-responsibility.md) for storage support/duty |
| Host bootstrap and compatible external-success facts | [Host capability owner](../design/static-evaluation/host-capabilities-and-machine-objects.md); [unsafe admission owner](../design/lifetime/unsafe-semantic-admission.md), §§3–6 |
| External acquisition dependency projection and infrastructure caches | [Build owner](../design/build-package/build-system-design.md), §§4–5 |

The host MUST preserve the single E boundary: lifecycle contracts and their
derived calculations remain ordinary lang programs evaluated by E. It MUST NOT
introduce a separate lifecycle evaluator/checker, global constraint solver,
optimizer-private semantic facts or a second source resolver. The basis is E
§§3–5 and lifecycle §§2.1.5, 9 and 14; observations are the actual ordinary
actions, their premises and results. Missing consumers stop under §10.

This handoff changes no Stage, Policy, Pattern, lifecycle, equivalence,
selection or ABIKind law. Raw/Normalized AST stays syntax-directed. O/Planner/M
algorithms and interfaces, a Frozen Machine Plan format, machine encodings,
register allocation, cost models and planner training are outside scope.
Persistent graphs, arenas, sharing, hashes, Rust types, scheduling and cache
granularity MAY vary within these obligations. No host type name in the current
code is a required future API.

### 1.1 Connected handoff responsibilities

The existing contracts supply the material whose preservation this document
requires. Their producers and requiring consumers form one handoff chain:

| Existing handoff | Material passed to host revalidation | Requiring host boundary |
|---|---|---|
| [Owner/namespace identity](semantic-owner-namespace-graph.md#identity) and [compile construction](compile-construction-boundary.md) | Legal parent/selected-input identity, observed selection domains, exact held snapshots and distinct current storage/effect context | §§4–6: lawful revision correspondence, selection and state-sensitive cache validity |
| [LC2](lifetime-program-consumers.md#lc2-whole-continuation-projection-and-projection-complete-clauses) / [LC13](lifetime-program-consumers.md#lc13-whole-original-k-cps-correspondence-pending) | Common actions/guards, projection-complete provenance, actual delivery and pending caller suffixes | §§2 and 9: synchronized observations, E-grounded equivalence and independent new-position validation |
| [LC4](lifetime-program-consumers.md#lc4-contract-expressions-and-four-way-boundary-consumption) / [LC6](lifetime-program-consumers.md#lc6-function-pattern-and-recursive-interface-consumption) | Pre/Post relation content, scoped four-way mappings and interface-compatibility assumptions | §§6–7: invalidate real call and interface consumers despite equal signature/ABI shape |
| [LC5](lifetime-program-consumers.md#lc5-exit-compression-and-automatic-post) / [LC8](lifetime-program-consumers.md#lc8-regular-access-and-cropping-programs) | Formed guarded summary inputs, separate actual-exit history, original companions and contribution lineage | §§7.2–8: summary propagation, stable observations and prescribed boundary-local collapse |
| [Shared action/lifecycle publication](semantic-values-and-extraction-interfaces.md#shared-action-and-lifecycle-handoff) and [migration delivery](policy-migration.md) | Actual cleanup/Pre evidence, selected frontier and successful joint producer/projection Post | §10: current-world validation without replay or partial publication |

These links assign preservation responsibilities to existing consumers; they
do not add an evaluator phase or imply that those consumers are connected.
Detailed host architecture requirements live here; linked contracts identify
their supplied/consumed evidence, while planning documents track wiring/status.
The [conformance index](../planning/canonical-semantic-conformance.md#e-host-preservation-and-incremental-revalidation-targets)
maps HA groups to these handoffs; the [roadmap](../planning/roadmap.md#10-structured-e-host-and-incremental-revalidation-readiness)
records their pending wiring. Missing upstream material uses §10 at its actual
requiring boundary, rather than a host substitute.

## 2. Common K, LE and projection correspondence

Basis: E §§2–3, 7.1 and 11, lifecycle §2.1.1, effect §§2–3, LC2/LC13,
physical-source §§2–3 and unified §20.
Here LE abbreviates the legally remaining common continuation together with
its corresponding E observations. It is not another result envelope or a list
of machine instructions awaiting a reconstructed lifecycle interpretation.

The host MUST retain or verifiably recover structured K and its residual
correspondence: action identities and positions/cuts, guarded control and order,
actual dependencies, sealed candidate/projection/frame, lexical scopes and
activations, pending caller suffixes, return targets and delivery, fixed cleanup,
handlers, Move/repair and compile/runtime/reify observations. Future actions
remain conditional descriptions; they MUST NOT be recorded as committed
history or supplied with invented concrete names, generations or activations.
Missing required control, origin, delivery or cleanup correspondence is
unavailable at its requiring consumer, including unsupported CPS contexts.

Structured K MUST preserve ordered Seq versus Unordered composition, common
entry snapshots, scopes, sibling overlays and ordinary effect-join/conflict
provenance. Files retain source order; unordered siblings observe their common
snapshot, not another sibling's newly written binding. Host scheduling, diagnostic
sorting and cache-realization order supply no semantic precedence. Lawful
contributions still combine under their ordinary algebra, including once-only
first residency; shared acquisition-registry checks remain required (§5.1) and
grant neither sibling namespace visibility nor a scheduling winner.

At reify boundaries the host MUST retain established ClosedReturn/ClosedInterface
observations, the actual boundary/activation and their dependency support.
ClosedInterface_K(B) requires compile-determined outward input/output, regular
effect shape, necessary captures and lifecycle requirements independently of
RevDeps(B). Changed support invalidates its actual consumers; cached interface
shape alone supplies no current E proof or active-boundary authority. Runtime
choice within a fixed regular sum remains legal; runtime-dependent public shape
requires the existing ordinary handler/wrapper before exposure. Missing closure
evidence stops at its requiring E boundary, never deferred to M to repair.

Machine, type, Policy, lifecycle and effect/control observations MUST identify
the same action at the same K position. Lifecycle projection forgets business
Stage while retaining selection, names/generations, guards, dependencies,
MoveEffect, activation and delivery. It neither repeats candidate selection nor
executes unknown future arms. Projection-complete `lifetime pre/post` material
retains that provenance through composition, without a second `@`/companion
layer. Missing counterpart material cannot be repaired by fresh ADL lookup.

The host MUST preserve E's existing saturation and correspondence laws:
`E(E(K)) = E(K)`, synchronized affected projections after rewrites, and the
whole-original-K CPS account covering ordinary no-wait work as well as Wait.
This includes expression/Product/call inputs, binding/assignment, nested calls
and caller suffixes, serial/control/return/delivery, destructors, handlers and
compile/reify boundaries. The E owner's Plug/Compose correspondence is the
basis, not a requirement to evaluate twice or choose a physical CPS layout.
Shared views or persistent revisions MAY carry it. Local wait splitting cannot
stand in for the missing common-K consumer.

The host MUST support branching continuation/revision descriptions from an
accepted checkpoint, exploring multiple proposed K revisions independently,
and retaining or verifiably recovering earlier accepted checkpoints with their
supporting observations (§9.1). A destructive single chain that loses every
alternative is insufficient. This concerns semantic description preservation;
branching does not copy ordinary program Objects, authority or committed
effects. No persistent IR, simultaneous execution or Planner API is prescribed.

## 3. Hosted ordinary lifecycle programs and bootstrap

Basis: lifecycle §§2.1.6, 9–9.4 and 14, invocation §§1–4, and LC1–LC4/LC7.
Required inputs are actual selected operations, source/target identities,
formation authority, cuts, regions/Color/access, scoped boundary mappings and
the ordinary compile expression and delivery consumer.

The host MUST make established E material available to ordinary lifecycle
programs through the existing observation, Pattern, call and formation laws.
Local values, branches, supported recursion and ordinary calls remain legal
programming tools; the final single self-root complete type and associated
check/admit behavior use ordinary CompileInstance/OpenHere/member/result
formation. Intermediate values need not be NameExprs. Missing source producers,
associated construction or result consumers remain unavailable; a host callback
or a recognized function spelling supplies none of them.

Bootstrap justification MUST terminate in the lifecycle owner's independent
basis: actual ordinary formation/name/Place, selected call/activation/delivery,
fixed cleanup, Move/origin/Region/Color/access and common publication laws, or
actual compatible external-success facts through the unsafe owner. A still
unformed contract, its check/admit, automatic Post or a companion justified only
by that contract MUST NOT establish its own formation/update Pre. An unfounded
justification cycle is unavailable. Established-contract recursive execution is
a separate subject (§8).

Safe boundary adoption MUST retain its actual action/activation/cut,
name/generation/result mapping and limited entry/success authority. Each
component's own premises justify its permitted increment; check=true,
OpenHere, a copied world record or transaction storage grants no world-writing
authority. Type construction and lifecycle-world adoption remain distinct.
Components stage in the same boundary's unpublished projection material and
publish jointly (§10). Unsupported adoption cannot become an unrestricted
safe setter. Temporary host bootstrap implementations MAY realize justified
basis operations while preserving the same premises; whole compression,
automatic-summary or library programs do not thereby become permanent opaque
intrinsics.

## 4. Identity, provenance and guarded relational observations

Basis: owner contract Identity/Pattern roots, invocation §§1–2 and 5,
dependency §§2–4, lifecycle §§2.1.2–2.1.3, 8.5 and 11–14.

The host MUST distinguish source location/display Path, semantic owner,
selected receiver/call-entry and frame, CompileInstance, complete TypeValue/Core,
NameBinding/NameCoord, Place/resident slot, LifeName/generation, action and its
K position. Equal values, Core, machine layout, text, numeric IDs or hashes do
not merge these identities or grant their authority. Cross-revision reuse
requires a lawful correspondence retaining actual owner parents, selected
inputs, dependency sources and action/name/generation/delivery mappings.
Missing correspondence excludes reuse; a requiring consumer stops under §10.
The owner contract's pending parent-homomorphism proof is not supplied by an
exact frontend-owner lookup.

Cross-K structural correspondence MAY be partial, many-to-one, one-to-many or
another structured relation where the existing E rewrite/fidelity laws justify
it (§9). The host MUST NOT require a global bijection between old and new action
IDs. Eliminated, fused, expanded and retained material still needs its actual
observation/obligation correspondence; a partial relation cannot simply omit
required effects or responsibilities. This flexibility concerns continuation
structure, not merging semantic identities or weakening owner-parent,
CompileInstance, handoff-occurrence or ordinary transfer laws. Within each K,
synchronized projections still identify the same action at the same position.
Missing required correspondence blocks the requiring reuse/rewrite under §10.
An arbitrary source edit need not preserve the previous program's behavior;
reuse claims still require their own observations and premises to remain valid
in the new revision. The equivalence gate in §9 applies to proposed O rewrites.

Shared observations MUST retain the revision, K position, history and premises
to which they apply. Updating one candidate branch cannot silently mutate
another retained accepted checkpoint's facts or authority. Cross-branch reuse
requires the same lawful correspondence and current-premise validation as
other reuse. Host checkpoint handles are storage/search coordinates, not new
SemanticOwner, CompileInstance or language identity axes.

E observations combined for one judgment MUST form a coherent semantic
revision/context. Individually valid facts from different K revisions or
checkpoints cannot be conjoined without lawful correspondence and current
validity sufficient for that combination. This permits §9's explicit old/new-K
comparison; it does not transplant an old-position premise into the new K.
No query transaction API or version-storage format is prescribed.

Relational observations MUST retain sufficient subject/target, K position or
interval, name/generation, scope, guard, selected operation, supporting premises
and availability to justify their actual consumer. Same name, same Place, same
reference target and overlapping storage ranges are different judgments.
MustAlias and NoAlias observations can both matter; unknown is not NoAlias.
Region/Color, access conflicts, effects, cleanup and delivery keep their actual
directed relations and source-choice correlations. No universal alias query
API or decision algorithm is introduced. Missing relation/guard evidence grants
no local reorder permission.

Negative/absence conclusions from any E relation query MUST retain sufficient
support for the completeness of their observed domain/range, K interval, guards
and relevant inputs. This includes no conflicting access, no additional effect
and no outstanding cleanup, not only overload exclusions (§6.1). A newly relevant
action, expanded range or changed guard/completeness premise invalidates dependent
conclusions even if all previously observed positive rows remain unchanged.
Unknown support is not absence. Local summaries or conservative dependency
ranges MAY supply this support; neither whole-world enumeration nor execution
of unknown future arms is required or authorized.

The host MUST support queries over the real affected relation/dependency range
without requiring every local query to expand a flat table of all pairwise
relations in the function merely because of its storage model. Shared structure,
on-demand computation and local summaries MAY provide this capability. A query
may still require substantial real dependency work; this is no constant-time
complexity promise. Every accelerated answer remains justified by E, with
unavailable required evidence handled under §10.

The host MUST permit lawful local K revisions to retain unaffected structure,
semantic observations and dependency evidence under valid revision/position
correspondence. Its storage representation MUST NOT make whole-K expansion,
regeneration of every action/projection correspondence or rebuilding all
dependency indexes the only route for a supported local change. Retained
evidence still needs its current validity; textual locality proves no semantic
independence. Actual dependencies crossing the changed region require all
affected propagation/validation, potentially throughout K. This capability
prescribes no persistent IR, update algorithm or worst-case complexity bound.

## 5. Cacheable material and current-state premises

Basis: invocation §§1–2 and 4–6, dependency §§2–5 and 7, build §5,
type/Place identity and lifecycle §§2.1.3–2.1.8 and 9.4.

The host MUST preserve these distinct reuse obligations, even if one physical
cache stores all three classes:

| Material | Required observation on reuse | Excluded inference / missing-material boundary |
|---|---|---|
| Immutable complete snapshot | The exact held complete tau/callspace and the Core projection of that same snapshot | Current registry/resident cannot reconstruct a missing old snapshot; required missing snapshot is unavailable |
| Current-world fact | Actual K/action/cut, Name/Place/resident generation, Region/Color/access, current OpenHere/Writable, Move and cleanup obligations | Old success grants no present read/write/transfer permission; changed or unknown premises require E revalidation or §10 failure |
| Replayable computation/summary material | Established parent-neutral material, ordinary source/selection provenance, observed inputs, dependencies and validity conditions | Same-history reuse does not replay committed initialization, acquisition, external effects, Kill/Drop or selection; fresh-compilation reuse retains its required actions/effects under §5.1; missing correspondence prevents reuse |

CompileInstance identity MUST remain the owner's parent × selected partner ×
canonical inputs relation. Contract/body revision information is validity
information, not a new instance-identity axis. Body/result material cannot mint
another instance for an equal complete key; conflicting bodies obey the
existing conflict boundary. Bound RequiredCapture sources enter canonical
inputs when semantically observed; caching does not bind specifications early
or re-resolve sealed sources. A legitimately new invocation binds/selects at
its own frontier under the existing rules.

A cache hit MUST re-establish every required current-state premise through E,
or reuse evidence whose dependency validity and lawful current correspondence
have themselves been established. Key equality alone proves neither result
validity nor authority. Reacquisition observes current instance/member storage;
it cannot restore an initial resident, reverse Close, revive a generation,
retarget a saved reference or extend a dependency lifetime. Cached Post is not
a license to consume input again. Required missing inputs or OpenHere are
unavailable, not successful empty observations.

Cached Diagnostic results, failed judgments and unavailable observations MUST
retain their actual inputs and absence/availability support with valid consumer
dependencies. Changed Pre, relation content or newly available required evidence
invalidates dependent failures even when source/signature/lookup keys stay equal.
The new or revalidated revision can then use its existing requiring E consumer;
an unchanged failure is reusable only with current support and invalidation
lookup. This neither retries/reopens the old sealed invocation nor replays its
represented actions, and it fabricates no success or Residual (§§6 and 10).

Fingerprints/versions MAY locate reuse candidates. The host MUST account for
collisions and changes of compiler/semantic environment in cache validation;
a digest or matching signature cannot be the final identity, equivalence or
permission proof. Conservative recomputation is permitted when finer evidence
is absent. It still observes §10's no-replay/publication boundary.

Under [host target/external facts §§4–5](../design/static-evaluation/host-capabilities-and-machine-objects.md#4-target-machine-facts)
and [unsafe admission §§3/5](../design/lifetime/unsafe-semantic-admission.md#3-post-directed-axiom-admission),
reuse MUST retain consumed target arithmetic/trap, representation, alignment
and semantic ABI observations and the validity support of admitted external
observations. Their sources, world/identity mapping, required intervals and
truth/maintenance premises are dependencies even when source text, signatures
and ordinary lookup keys stay equal. They enter through ordinary E facts, not
target flags or O/M-private constants. Missing/expired support blocks current
reuse; this neither mutates old immutable snapshots nor retracts committed
history. A false admitted unsafe axiom remains the unsafe owner's truth/UB
boundary, not something cache invalidation retrospectively repairs.

### 5.1 Same-history revalidation and fresh-compilation reuse

Basis: [host acquisition §1.1](../design/static-evaluation/host-capabilities-and-machine-objects.md#11-link-specific-effect-law)
and build §§4–5. Revalidating an already committed action in the represented
history MUST NOT commit it again. Reusing material in a fresh compilation MUST
instead preserve the actions and effect records required in that compilation's
E state, with actual identity, dependencies, current premises and ordinary
Pre/commit/Post. Avoiding physical work is not permission to delete semantic
events, skip conflicts or import the old compilation's registry/history.

For an already represented action a, revalidation MUST obtain the required
observations of its lawful before-action state in the corresponding revision,
with valid input/name/generation/guard mappings and dependencies. A changed Pre
is checked against that revision's before(a), not the cached after(a), the latest
resident state or an old before-state whose correspondence is unproved. For
example, consumption ending x's generation after a cannot itself falsify x's
availability before a. The host MAY retain event material, shared state, local
summaries or recompute observations from an earlier lawful frontier; no full
snapshot per action is required. If this support cannot be established, abandon
that local reuse and perform supported wider E revalidation or report the actual
unavailable boundary. Recovering observations neither rolls back committed
history nor grants present transfer/update authority or replays an action.

Cached material MAY avoid repeated physical computation only where the actual
ordinary operation's semantics and current validity permit that realization.
It cannot supply otherwise missing external/resource truth, ownership or
acquisition authority, or authorize repeating an external physical effect.
For link, a fresh compilation has its own single LinkRegistry. The first
authorized acquisition still reserves Active and reaches Done only on success,
retaining its acquisition effect and normal checks even if cached material
realizes the work. Active reentry remains a cycle; Done remains a duplicate,
not a second cached source-level link. Nested/sibling acquisitions share that
registry and cannot coalesce duplicate effects. The build DependencyGraph
continues to project those actual evaluation effects. Missing required reuse
or acquisition evidence stops at the existing boundary; it creates no Done
entry or successful Post. These source consumers remain pending.

## 6. Fine-grained dependencies and change propagation

Basis: E §§3–5, lifecycle §§2.1.6, 8.5 and 10, dependency §§2–5,
invocation §§1 and 6, and build §§4–5.

Four relations have different purposes and MUST retain those distinctions:

| Relation | Nodes/edges and purpose |
|---|---|
| Function call graph | Actual callable/call relationships; a possible future Planner organization boundary |
| Lifecycle name-item graph | Formed items/generations and witnessed directed tracking edges with guarded contribution provenance; the owner's SCC/collapse law |
| Host incremental check dependencies | An observation/calculation and the established inputs it actually consumed; invalidation and revalidation bookkeeping |
| Build DependencyGraph | `Projection(EvaluationEffects)` of external acquisition; post-hoc build observations, not a semantic input graph |

The host MUST retain enough dependency information to find affected consumers
of changed or invalidated observations, including these associations:

| Observed input | Dependent calculation/consumer |
|---|---|
| Receiver/call-entry, projection/frame, canonical arguments and bound RequiredCapture | Call validity, CompileInstance and result material |
| Resolved binding/shadowing/exposure, exact observed callspace, candidate domain and consumed rejection/fallback/preference evidence | Reuse of resolution, applicability, exclusions and the unique selection, not only the selected tuple (§6.1) |
| Actual Pre/Post relation content, entry/result mapping and exposed interface assumptions | Caller Pre.check, caller Post.admit, subsequent action Pre and Function Pattern contract compatibility |
| Formed guarded body, bound roles, selected-operation descriptions or sufficient exposed contracts | Formation-time Post_summary and its support |
| Current names/generations, residents, Region/Color/access, cleanup and Policy | Action legality, actual Post, delivery and resource responsibility |
| Original regular companions, guarded item/edge contributions and boundary mappings | Supported SCC collapse, local compression and recursive/modular composition |
| E-known target arithmetic/trap, representation, alignment and semantic ABI facts; admitted external observations and their validity/truth support | Dependent E computations and O equivalence evidence, and separately the machine plans/artifacts that consumed them; unchanged source hashes/ordinary keys prove no independence |
| Observed relation domain/range, guards and completeness support, including absent access/effect/cleanup observations | Dependent negative conclusions and their consumers; newly relevant material invalidates absence even when old positive rows stay equal; §4 |
| Target-definition identity and E/machine observations actually consumed by a future materialization decision | That frozen machine plan's validity and dependent artifacts, independently of unrelated semantic material; §9.3 |
| Seq/Unordered composition, common snapshot/scope, sibling contributions and join/conflict support | Dependent overlay observations, legal merged results and conflicts; scheduling/cache order cannot become source visibility or precedence |
| Reify ClosedReturn/ClosedInterface observations, actual boundary/activation and reverse-dependency support | Consuming reverse-work admission, exposed interface and affected materialization; cached shape alone proves no closed/current boundary |
| Actual frozen plan/output contract, target definition, M implementation/capability/options and downstream toolchain inputs | Exact M-validation results and emitted/final artifacts according to their own consumed environment; §9.3 |

Dependencies MAY be tracked by observation, call position, local subcontinuation,
function or a supported SCC; functions are valid coarse boundaries, not a
mandatory smallest unit. The host MUST permit stability to be established below
a function boundary where available structure supports it. A coarse retained
dependency range MAY conservatively cause more reevaluation, but cannot justify
incorrect reuse or prevent supported finer correspondence from being carried.
Missing dependency evidence cannot certify independence.

The host MUST invalidate dependent conclusions when their consumed relation
content or required premise changes, even if signature text, parameter/result
types, ABIKind, layout and machine calling shape are identical. It MUST continue
through actual consumers until the relevant outward observations are proved
stable, or a requiring check fails/is unavailable. Equality is the applicable
E observation relation under valid identity/guard mappings, not textual or
hash equality and not an arbitrary program-equivalence oracle. If unchanged
observations cannot yet be proved, continue required revalidation; if a needed
consumer/evidence is unavailable, stop at that boundary under §10.

A stable-observation stop still requires valid dependency support and invalidation
lookup for the new revision (§7.3). Result equivalence alone does not establish
that the prior revision's dependency sources or lookup entries remain sufficient.

These dependency rules cover cached failure/Diagnostic/unavailable observations
as well as successful calculations (§5).

### 6.1 Selection domains and negative observations

Basis: [ordinary overload §§1–5](../design/patterns-overload/overload-resolution-design.md)
and the call/invocation owners' exact immutable snapshot rules. For conclusions
depending on set completeness, absence, exclusion, uniqueness or preference,
the host MUST retain the observed domain and sufficient validity evidence.
Call reuse therefore accounts for resolved binding/shadowing and exposure,
the exact held complete-type/callspace observations, candidate/projection
enumeration, R_vis/C_sigma and hard-A inputs, proved Pattern mismatches,
fallback suppression and the comparisons establishing a unique winner. Merely
retaining Selected=(candidate,projection,frame) does not establish that no
other relevant candidate can change the result. Unknown/unavailable evidence
cannot be cached as absence, rejection, NoAlias or an established maximum.

This requires sufficient evidence, not an eager full-family ledger, fixed
query API or future-path/candidate-body execution. A conservative dependency
range MAY invalidate more work when finer support is absent. A changed current
lookup domain or newly read snapshot invalidates conclusions that consumed it;
it cannot modify an earlier held immutable tau/callspace. Old K's sealed call
retains its origin and never reselects. If reuse for a new revision cannot
establish the required selection-environment correspondence, its call MUST be
formed through ordinary E selection in the new K; the old seal is not patched
or reopened. Missing required observations remain unavailable at their normal
resolution/Pattern/selection boundary. Lifecycle/Pre failure remains terminal
after selection, not another exclusion or preference rule.

## 7. Call-site revalidation and automatic Post propagation

Basis: lifecycle §2.1.6 and §§10.1–10.3, call §6.3, E §§3–5 and LC4–LC6.
Lifecycle Pre/Post content is externally consumable call semantics (the
semantic contract aspect of ABI); this introduces no additional ABIKind or
machine encoding. Machine shape equality is not contract equality.

### 7.1 Required response to a changed callee contract

For a changed `Pre_B` or `Post_B`, the host MUST identify every actual consumer
of the changed observation. At affected call positions, including equivalent
inlined local consumers, it MUST perform the following. Valid evidence that a
consumer's required observations are unchanged may discharge an individual
check; old cached success alone cannot.

An actual consumer includes already formed semantic descriptions under unknown
guards, not only calls that have executed. If such material consumed B's Pre,
changed content MUST invalidate its dependent formation/validation support
before that support is exposed or reused. Revalidate available conditional
descriptions through their ordinary consumers; the concrete call checks below
use actual mappings when that action occurs. This neither executes/prunes a
future arm nor invents its activation, names or a successful Pre. Missing
required description/contract evidence is unavailable at its formation consumer;
unchanged outward observations can still stop propagation under §7.3.

1. Recover the call position and its actual selected candidate/projection/frame,
   implicit Param/captures, bound required inputs, explicit arguments,
   result/activation mapping and corresponding before-action observations (§5.1).
2. Revalidate caller `Pre.check` under that mapping in the lawful before-action
   state of the current semantic revision, not a cached post-call state. Failure
   prevents the action; body `Pre.admit` supplies no repair and selection does
   not reopen. For represented history this blocks the new revision's reuse,
   without undoing the old commit or its external history.
3. Recompute the mapped guarantee of legal success and its caller `Post.admit`;
   implementation `Pre.admit`/`Post.check` remain their ordinary counterparts.
   Preserve actual history and one transfer, with no cached replay of the body.
4. Revalidate the successor subcontinuation's checks/contract consumption that
   actually depended on the changed Post. Old-position proof is insufficient
   at a changed position or generation.
5. Notify consumers of every changed outward observation. Stop only where
   still-valid evidence establishes the required outward observations stable,
   or reject/report unavailable at the actual requiring boundary (§10).

Other consumers MUST revalidate through their own existing checks, not invent a call frame.
In particular LC6 interface extraction/assignment revalidates
`Pre_F implies Pre_B` and `Post_B implies Post_F` under its ordinary binder,
entry and resource mapping, even if no subsequent call has occurred. A changed
contract can invalidate that extraction while the signature stays equal.
Missing implication evidence is unavailable there; pure matching performs no
Move/Drop, and known Val1 cannot restore erased implementation authority.
This is lifecycle §2.1.6 / LC6 compatibility, not a new implication solver.

These obligations concern actual semantic consumers, not unconditional
whole-program execution. Revalidation recomputes observations of represented actions and
history in a lawfully corresponding revision; it does not physically Move,
Kill, acquire or Drop already committed material again. A genuinely new action
still uses the common gate. Neither an incremental engine nor a cache can
rewrite old committed history to fit the new contract.

### 7.2 Separate formation summary and actual exit

The host MUST carry separate provenance, inputs, dependencies and validity
conditions for these existing lifecycle consumers:

| Consumer | Input / required result |
|---|---|
| Formation-time Post_summary | Complete formed body Pattern/structure, bound parameter/result/local roles, guards, scope/return and selected-operation descriptions or sufficient exposed modular contracts; ordinary lang transformation forms the conditional outward contract before use |
| Actual-exit Post_actual | Reached activation/path, real names/generations/cuts, committed history, fixed cleanup and actual delivery; ordinary local compression yields outward material and checks `Post_actual(p,theta_p) implies Post_summary(theta_p)` |

Both use the same E. A missing formed description, supported transform or
founded recursive-family result makes the summary unavailable. The host MUST
NOT replace it with NoDelta, one later actual path, execution of future arms,
fresh future names or restored hidden implementation authority. Failed or
unfinished exits publish no successful Post. Unknown business guards remain
correlated conditional source/effect relations, including effects beyond origin.

### 7.3 Three distinct stability boundaries

The host MUST propagate automatic-summary changes through the actual dependency
chain, preserving the distinction between these stopping conditions:

| Situation in `C -> A -> B` (call-graph metanotation) | Necessary revalidation and lawful stop |
|---|---|
| B's Post changes; an affected subcontinuation inside A has equivalent required entry/exit observations after remapping | Revalidate its actual affected checks and all exposed relations; supported stability evidence can stop propagation inside A (HA06) |
| A requires updated formed descriptions and recomputed automatic Post, but its outward summary/other consumed observations are equivalent | Re-form through ordinary lang programs; after the internal checks and summary fidelity obligations hold, stop at A's boundary for those stable observations (HA03) |
| A's automatic outward Post or another consumed observation changes | Notify A's consumers, including C's relevant call positions; continue until proven stable or a real failed/unavailable check (HA04) |

Stability covers each consumer's actual observations, including identity,
guards, effects, access, regions/Color, delivery and obligations as required;
an equal origin row alone cannot hide another changed effect. A function's
stable summary does not mask a separately consumed internal observation.
An explicit Post MAY bound propagation when all its guarantees are still
established and other consumed outward observations are unchanged. If new
behavior cannot satisfy it, its ordinary check rejects; the host MUST NOT
silently weaken or replace it with an automatic contract (HA05). Missing proof
is unavailable, not equivalence or successful delivery.

Before exposing/reusing a result at a stable stop, the host MUST establish or
validate the new revision's actual dependency provenance, guards, scope and
domain/range completeness support, including sufficient lookup to invalidate
its consumers on subsequent changes. Equal outward observations do not prove
equal dependencies or validate the old invalidation index. If A now consumes Y
instead of X while its outward Post stays equivalent, record Y's support and
consumer association even when C needs no recomputation now. Conservative extra
dependencies MAY remain, but cannot substitute for Y; retained old checkpoints
keep their own revision's support. Missing new support prevents reuse there
under §10. Stopping downstream recalculation does not skip this bookkeeping.

## 8. Recursive/SCC and termination boundaries

Basis: lifecycle §§8.5, 10 and 14; LC5/LC6/LC8; E's no-future-execution rule.
Incremental scheduling MAY coordinate actual call SCCs or use demand-driven
tracking, versioned sharing or work queues. The host MUST NOT treat such a
notification loop, E saturation, a call SCC or finite regular representation
as a general lifecycle fixed-point solver or a convergence proof.

Each adopted ordinary recursive contract/summary family MUST retain its
supported domain, composition operations, closure and termination evidence or
explicit rejection/unavailable condition. Finite regular descriptions can
denote infinitely many walks; their finiteness proves neither arbitrary program
termination nor numeric-range decidability. Missing family evidence is
unavailable at the requiring summary/contract consumer; unbounded speculative
iteration supplies no result. This does not add a universal termination checker.

For lifecycle name-item SCCs the host MUST preserve the owner's original paired
regular companion for each actually formed item, actual names/generations and
directed edge contributions, compatible guards, formation/establishing
activations and lawful scope/control/delivery mappings. At Ordinary boundaries
and participating InPlace exits it carries exactly the owner's prescribed
boundary-local collapse, not a scheduler-chosen whole-component collapse.
Nonparticipating InPlace exits still compress dying locals. Mutually exclusive
arms cannot be unioned into a synthetic cycle; missing participation evidence
is unavailable, not false.

Successive InPlace exits MUST retain remaining outer fine material, original
descriptor associations and enough contribution/incidence correspondence for
later cuts. An inner reduction cannot erase outer participation, read dead
locals, reconstruct authority from type/state, restore cropped access or defer
a mandatory collapse. If required lineage cannot be retained, the transform
is unavailable at that cut. Ordinary recursive programs, name-item SCC collapse
and physical tail-call/slot reuse have separate proof obligations.

## 9. Future O/Planner/M readiness

Basis: E §§4–6, 7.1 and 10–11, lifecycle §7, mechanical passing and effect §2.
These are preservation requirements for future consumers, not their APIs.

For a proposed O rewrite the host MUST retain two independent obligations and
their supporting E observations:

1. The E owner's existing `Facts_E proves R equivalent_to R'` obligation:
   original/proposed K have equivalent admitted observations, including values,
   effect traces and lifecycle/cleanup/resource observations where applicable.
   Both continuations being legal does not prove this equivalence.
2. `E_affected` validation on the proposed K: every affected synchronized
   projection rechecks its actual new-position Pre/commit/Post, call/Policy,
   dependency, cleanup and delivery premises, with newly ready work returning
   to E saturation. An old-position equivalence argument does not supply these
   current premises.

Both MUST hold before a candidate becomes an accepted rewrite. A legal candidate
with different observable results/effects still fails the first obligation.
This retains existing E facts and transform justification; it introduces no
proof-Object type, universal equivalence decider or optimizer-private premise.
False or unavailable evidence for either obligation prevents acceptance and
leaves the previous accepted revision intact.

The host MUST retain lawful original/new K correspondence for actions,
names/generations, calls, guards, cuts and delivery. Under §4 it need not be a
global one-to-one ID map: admitted elimination, fusion, duplication, inlining
or reordering can use partial or structured many-to-one/one-to-many relations.
The transform's justification accounts for changed/removed occurrences and
observable repair, ownership and cleanup duties; correspondence alone grants
no duplication of effects or consumable authority. No mandatory physical
Object per semantic cut follows. Same-K projection synchronization and specific
canonical identity/transfer relations retain their own laws. Inlining or moving
work between functions cannot erase dependencies when an old call edge
disappears. Missing correspondence blocks acceptance just like either failed
obligation above. CPS representation fidelity is not E performing an O rewrite.
The local-update capability in §4 preserves unaffected material under lawful
correspondence; it waives neither acceptance obligation nor required E saturation.

### 9.1 Branchable accepted checkpoints

Here a checkpoint denotes a retained accepted K revision and its supporting
observations after the required E validation/saturation, not a new semantic
result kind. The host MUST distinguish these accepted checkpoints from pending
candidates and support multiple independently explored descendants of the same
checkpoint. Accepting, rejecting or abandoning one branch MUST NOT require
destructively overwriting every other retained accepted version. Earlier
checkpoints retained as exploration bases and retained accepted siblings must
remain available or verifiably recoverable, with their actual dependency/
correspondence evidence. Retention policy MAY deliberately release unneeded
historical versions; this does not require unbounded retention of every
candidate. Exploring one branch cannot force loss of retained bases/alternatives.

Recovery MAY use sharing, snapshots, journals, retained descriptions or another
representation. It cannot rely on replaying committed source actions, effects
or consumptions to reconstruct lost support. Recovered material still checks
its current premises before reuse; an old checkpoint supplies no stale authority.
Forking, selecting or discarding descriptions itself commits no business action,
clones no owner/resource and rolls back no external history. Any real action
needed during exploration retains its ordinary E/transaction premises; missing
legal observation or isolation/reuse evidence cannot be silently speculated.
In particular, a disposable O1 candidate may expose a new compile-host action
whose irreversible external effect would be needed for E saturation. Without
lawful isolation or reuse evidence for that effect and actual history, the host
MUST stop that exploration before performing it, rather than claim saturation
from a description snapshot. It cannot fake a legal sibling via shared mutable
execution history/authority, cloned rights or later rollback. If an independently
legal effect has actually committed, it persists; sibling reuse checks the real
resulting history. Declining such exploration does not waive E saturation for
any candidate the host does accept.
Candidate failure leaves retained accepted versions intact, with no successful
Post fabricated for the failed branch. This requires neither parallel search
nor a particular checkpoint, versioned IR or Planner interface.

### 9.2 Objective switching without global phase barriers

The host representation and update machinery MUST permit future search to use
and switch O1/O2 objectives at actual function, SCC or subcontinuation dependency
boundaries without requiring global completion of one objective first or an
irreversible O1-to-O2 phase transition. The E owner's idealized objective
separation is not a mandatory global pass schedule; a transform may contain
both components. Each transform still meets both acceptance obligations above,
and newly ready work returns to complete E saturation.

Local search can require wider validation through real dependencies, fixed
cleanup or effects; objective labels cannot override those constraints or
defer mandatory ready E work. Scheduling MAY be serial or coordinated for those
dependencies or an implementation policy, but its representation cannot require
all regions to share a global objective phase. No per-function Planner instance,
parallelism or independent optimizer alias/effect/lifecycle/Stage truth system
is required. O1/O2 are search objectives, not new Policy or language stages.

Function and call-SCC planning partitions are derived organization views of the
current accepted K, not immutable semantic identities. When an accepted rewrite
changes actual call dependencies, the host MUST permit lawful rederivation of
those views, including splitting/merging partitions and reassigning future
planning work. Inlining B into A may remove A's call edge while retaining B's
semantic dependencies inside A. Repartitioning MUST preserve those dependencies,
affected E validation/saturation and retained accepted-checkpoint correspondence;
it cannot keep stale validation merely because a task name survives. Reorganizing
planning work proves neither recursive convergence nor a lifecycle name-item SCC
collapse, and changes no semantic owner or sealed invocation (§§4, 6 and 8).

### 9.3 Complete default route within declared materialization support

A future materialization support claim MUST identify its continuation family,
target, required semantic/machine inputs and unsupported boundaries. For every
accepted checkpoint covered by that claim and its established input conditions,
the host MUST preserve a complete terminating default route through the needed
O2 decisions, affected E validation/saturation and M to a complete executable
target artifact. Completion cannot depend on further optional budgeted,
heuristic or AI search succeeding. The default strategy may make choices and
run its guaranteed algorithms/checks; this does not require a unique layout,
zero lowering work or an optimal artifact.

Accepted rewrites within that claimed support domain MUST preserve this
default-route property or establish it before acceptance. A semantically legal
candidate that loses the route cannot replace an accepted supported checkpoint
under the unchanged claim. Stopping optional search at any covered accepted
checkpoint can therefore take the default route, including after local O2 work;
it need not resume heuristic exploration to discover how to finish. A partial
plan or promise that AI may complete it is insufficient. Failure of the route
for an already admitted covered checkpoint is a failure of the support claim,
not an optional search miss or permission to narrow the claim retroactively.

Required material includes common-K/CPS correspondence, selected calls,
normalized terminal Move and actual repair, delivery target/E-selected ABIKind,
lifetime/cleanup/resource obligations and target facts. Strategy choices belong
to future default O2, not hidden E rewrites. M executes an upstream-decided
target-machine plan.

Upstream plan construction MUST retain E-grounded justification that the plan
implements the accepted LE's admitted observations under the consumed target
facts, including values, effects, delivery/ABI and lifecycle/cleanup/resource
obligations as applicable. Plan completeness, encoding legality and this semantic
fidelity are distinct obligations: a valid subtract instruction does not implement
an E-required addition merely because it encodes. Final M checks precise
materializability of the plan, not whether it implements the original LE. This
retains E §§5/10/11's fidelity responsibility without another evaluator, mandatory
proof-Object format or universal equivalence algorithm.

The host MUST preserve a boundary at which all machine-policy choices needed
for deterministic materialization are settled upstream: instruction selection,
operand legalization, encoding-form selection, layout and fixup policy. Final M
MUST validate the frozen plan before executing it, including its prescribed
deterministic encoding/fixup calculations, and MUST NOT fill undecided choices,
silently repair the plan or search for a different form/layout. Incomplete,
invalid or unsupported plans stop at their actual handoff under §10. Such
rejection does not satisfy the complete default-route promise for an already
covered accepted checkpoint.

Plan validity MUST retain the consumed target-definition identity and supporting
E/machine observations with their dependency/current-validity premises (§§5–6).
Freezing choices for one handoff does not make that support permanently valid;
changed definition/support invalidates actual plan/artifact consumers before
reuse. This boundary divides future O/materialization planning from final M
execution, without relocating E's ABIKind decision or forbidding equivalent
layout planning upstream. It fixes no plan structure, target-definition encoding,
lowering algorithm, FMP serialization or O/Planner/M entry point.

The complete default route MAY extend beyond M's output: prescribed bytes,
sections, symbols or fixup records need not themselves be an executable container.
The declared route MUST include any required terminating packaging/linking steps
and their inputs through the complete executable artifact. Those steps execute
upstream-settled policies; they cannot conceal instruction/form selection,
relaxation, layout/fixup repair or semantic ABI choices. A required new planning
choice returns upstream with its E/fidelity obligations, rather than becoming
implicit M or linker discretion. M success alone proves neither route completion
nor semantic fidelity. No particular container or linker interface is fixed.

The host MUST distinguish these validity claims and retain the full environment
actually consumed by each result, rather than treating one cache hit as all three:

| Claim | Required consumed support |
|---|---|
| Upstream machine-plan semantic validity | Accepted LE correspondence, E/machine facts, target-definition identity and semantic-fidelity justification |
| Final M validation validity | Frozen plan, output contract, target definition, actual M implementation/version or sufficient established capability conditions, and relevant validation options |
| M output and final artifact validity | Actual plan/output contract/definition, M implementation/version or sufficient established capability conditions, output-affecting options and required packaging/link inputs, tool versions/options and dependencies |

Changed consumed environment invalidates dependent validation/output/artifact
reuse unless sufficient current validity is established. An unchanged target
definition alone is insufficient. Independent plan-semantic or E evidence MAY
remain valid under its own premises; materializer/tool options supply no second
channel of meaning. Meaning-affecting facts still enter through ordinary E.
No cache-key encoding, toolchain interface or MonAsm format is prescribed.

This guarantee is conditional on the declared supported family/target and
inputs; semantic legality alone does not establish materialization coverage.
It concerns termination/completeness of that compilation route, not termination
of arbitrary source evaluation or the represented runtime program, universal
equivalence checking, or support for all legal LE. Today's unconnected
materialization consumers remain unavailable at their actual boundaries;
this PR declares no such family/target implemented. Required unavailable inputs
outside a claim supply neither a successful handoff nor an invented plan, and
cannot be used to advertise a covered checkpoint as completely supported.

Semantic and machine-artifact invalidation MUST follow their own actual
dependencies. Pure machine materialization choices with no changed E observation
need not invalidate independent semantic material (HA14); a changed semantic
premise is still revalidated even when machine bytes/shape remain reusable
(HA01). A machine choice affecting meaning requires affected E validation.
Preservation of the default path is a future implementation obligation, not a
claim that a materializer exists today. No mandatory Box/state machine or one
physical Object per semantic cut is introduced.

## 10. Failure, unavailable and transaction discipline

Basis: E §3, lifecycle §§2.1.5, 9.3–9.4 and 10.2, invocation §8, effect §2,
and the LC shared result boundary. This rule applies to every MUST above.

The host MUST use the existing common transaction: fixed earlier-cleanup gate,
all affected action/operation Pre on the actual prior world, one authorized
action and joint producer/projection Post before public success. Outstanding or
unavailable cleanup blocks even an action without a lifecycle projection.
Scratch projection material is not a partial Object or a source-authority proof.
Failed/unavailable components publish no part of that gate's delta, successful
result or new permission. Earlier independent commits/external effects retain
their actual history; there is no general rollback promise.

The host MUST retain InvocationResult as the sole envelope. Known false
premises reject at their actual action/contract boundary. Missing required
consumers, source/identity/guard/summary/recursive evidence report the appropriate
spanned unavailable Diagnostic there. `Unavailable` here names that frontier,
not a new result variant. Only an established unfinished common K can be
transported as Residual. A cache miss, unsupported expression, absent summary
or no-candidate failure supplies no such continuation.

Revalidation MUST preserve once-only initializer/acquisition effects, actual
Move/Kill/Drop counts, fixed cleanup and selected no-reopen. It cannot restore
permissions, invent NoDelta/unknown/None, silently clone/reborrow or try another
candidate after failure. For a retained action with unchanged action/delivery
premises, it MUST NOT switch the fixed MoveEffect/ABIKind as a repair or retry.
A source revision or admitted rewrite genuinely changing those premises instead
invalidates the old evidence and obtains the new action/delivery facts through
the existing E/lifecycle laws; it cannot patch the old sealed invocation.
For example, a changed FreshLet versus ExistingTarget consumer receives the
E owner's corresponding ABIKind, not the previous revision's cached convention.
An ordinary normal error/paused result keeps its own real guarded effects; it
is distinct from semantic rejection. A required observation with no lawful
reuse proof is recomputed by E, or reports its missing boundary, never filled
by host guesses.

The no-replay rule is relative to that represented history. A fresh compilation
using cached material still accounts for its own required actions/effects and
registry checks under §5.1; it neither replays the old history nor omits the
new history's semantic events merely because physical computation was reused.

## 11. Acceptance and current connection status

### 11.1 Distinguishing architecture targets

Every row below is **specified; executable consumer coverage pending**. None is
a passed execution test. References to sections identify host requirements;
their canonical basis and missing-material rules are given there and in §10.
Future tests need real ordinary source producers/actions and meaningful
positive/negative, identity/equality, non-derivability, authority-uniqueness,
effect-count and no-reopen evidence as relevant, not mock proof imports.

| ID | Input/change | Required trigger | Lawful stability or failure boundary / requirements |
|---|---|---|---|
| HA01 | B's Pre relation changes; ordinary signature and machine ABI shape stay equal | Find each actual contract consumer and revalidate mapped caller Pre.check despite cached success | Valid unchanged-observation evidence may discharge an individual check; otherwise false/missing Pre rejects/is unavailable there; §§5–7, 9 |
| HA02 | B's Post changes and A uses that relation | Remap successful Post/admit and revalidate A's dependent successor Pre/contract observations | Continue through changed observations; fail/unavailable at the actual dependent action; §7.1 |
| HA03 | B's changed Post requires work in A, whose automatic outward Post remains equivalent | Update affected formed descriptions/checks and re-form Post_summary(A) | Proven stable outward observations stop this propagation at A, without unconditionally invalidating C; §7.3 |
| HA04 | A's automatic outward Post changes observably | Notify A's actual contract consumers and repeat mapped call-site/successor revalidation | Continue beyond A through C and further real consumers until stable, failed or unavailable; §§6–7 |
| HA05 | A has explicit Post; changed B behavior cannot satisfy it | Revalidate the affected explicit guarantee against supported body/actual-exit evidence | Reject at its actual check; missing evidence is unavailable. No implicit weakening or automatic replacement; §§7.2–7.3 |
| HA06 | A's internal subcontinuation has provably unchanged required entry/exit observations | Revalidate affected local checks and all exposed relations with valid mappings | Stop inside A when stability proves no outward dependency changed; no mandatory whole-function recomputation; §§6, 7.3 |
| HA07 | Two formed branches have an unknown future business Guard | Retain guarded operation/source/effect descriptions and projection correspondence | Form a supported conditional summary without running/pruning arms; missing descriptions/guards unavailable; §§2, 7.2 |
| HA08 | Contract change involves a recursive function call SCC | Coordinate actual dependencies with supported modular/recursive-family evidence | Proven domain/termination or explicit rejection/unavailability; no assumed iterative convergence or name-item collapse shortcut; §8 |
| HA09 | One name-item SCC crosses inner participating InPlace, transit-only InPlace, outer participant and Ordinary boundary | Use original companions, compatible guards and actual item/edge contribution provenance at each real cut | Collapse only required local contributions; preserve outer fine material/incidence for later cuts; missing lineage unavailable at that cut; §8 |
| HA10 | Cached lifecycle success meets a changed Name/generation | Revalidate actual identity, Region/access/Color and action/delivery mapping | Old name spelling/value cannot inherit old authority; stale/unknown evidence rejects/is unavailable; §§4–5 |
| HA11 | A held complete type snapshot differs from the current mutable resident | Keep the exact immutable snapshot and current storage observations distinct | Never reconstruct the old tau from current registry/resident; missing snapshot unavailable; §5 |
| HA12 | Compile material/key hits while OpenHere, input dependencies or the consumed selection domain change | Recheck current requirements and the selection domain's completeness/negative/preference evidence; form new-K calls ordinarily when old-seal correspondence is not established | No restored authority or patched old seal; old immutable snapshots stay unchanged; required unknown observations remain unavailable; §§5–6.1 |
| HA13 | Future O inlines a call, removing its graph edge while preserving semantic dependencies | Retain E-grounded observational equivalence justification, lawful structural correspondence and inlined dependencies; independently run affected E on new K | Accept only when equivalence and new-position validation both hold; missing/false evidence for either or required correspondence blocks the candidate, preserving accepted K; §§4, 6, 9 |
| HA14 | Only machine materialization decisions change, with E observations unchanged | Invalidate machine consumers of those decisions by their actual dependencies | Independent semantic material can remain valid; a semantic effect of the decision still requires E; §9 |
| HA15 | Lifecycle consumer, source/origin, formed description or automatic Post evidence is absent | Report the actual requiring unavailable Diagnostic | No synthetic Residual, None/unknown, NoDelta or successful contract; §§3, 7.2, 10 |
| HA16 | Action/late component fails, cleanup is outstanding, or cache reuse crosses a history/compilation boundary | Apply cleanup/Pre/joint publication; distinguish same-history no replay from a fresh compilation's required action/effect records | No partial success or duplicate consumption; retain earlier commits and new compilation registry/conflict checks, without gratuitous physical replay; §§5.1, 10 |
| HA17 | Local alias/reordering query needs several guarded relations | Query the actual relevant subjects, ranges, guards and dependencies, with positive MustAlias/NoAlias evidence where required | Unknown does not prove NoAlias; storage cannot force all-function pairwise expansion for every local query; §4 |
| HA18 | Contract program tries to use its own unformed check/admit/AutoPost/companion as its basis | Trace ordinary action justification to independently established semantics/observations | Unfounded cycle unavailable; founded ordinary programs still use E with their own premises; §3 |

HA01/HA02 also cover a contract already consumed by LC6 interface extraction:
changing B's Pre/Post can invalidate the mapped implications before any call.
Revalidate at extraction/assignment, without waiting for a future invocation,
replaying resources or recovering erased source authority (§7.1).
For HA14/HA16's boundary contrast, changing actual delivery from FreshLet to
ExistingTarget changes an E premise and requires its new ABI classification;
it is not a pure machine-choice change. Keeping that delivery unchanged while
switching ABIKind/MoveEffect to escape a failed check is forbidden (§10).
These are design subcases of the existing rows, not executed tests.

Further design subcases refine the same IDs:

- **HA12:** A new current read supplies a candidate domain containing c2 that
  outranks or ties the unchanged c1. The new K cannot reuse the old c1 seal
  without valid selection support: ordinary selection chooses its actual
  winner or reports ambiguity/unavailable evidence. A call still consuming the
  original immutable tau keeps that snapshot's family. Changed shadowing,
  fallback suppression, or a formerly proved mismatch now becoming unavailable
  likewise cannot inherit the old exclusion/uniqueness conclusion (§6.1).
- **HA13:** Two K candidates can both satisfy E legality while delivering
  different observed results/effects; this supplies no equivalence. Conversely,
  an E-justified nonbijective structural transform may be accepted after both
  obligations hold. Unaccounted owned occurrences, effects or cleanup, or a
  dropped required correspondence, prevent acceptance (§9).
- **HA16:** Revalidating a committed acquisition in one history submits no
  second action. In a fresh compilation, lawful cached realization of the first
  link still records its authorized acquisition in the new shared LinkRegistry.
  A second source-level/sibling acquisition remains a duplicate and Active
  reentry remains a cycle, even if all physical material is cached (§5.1).

- **HA13 checkpoint branching:** From accepted K0, explore K1 and K2 and retain
  every accepted sibling with its support. Rejecting/abandoning another candidate
  cannot destroy K0 or those accepted versions. Recover them without replaying
  actions or copying consumable ownership, and recheck current premises on use
  (§§2, 4 and 9.1).
- **HA13/HA14 objective scheduling:** Different function/SCC/subcontinuation regions
  can use or switch O1/O2 objectives without a representation-imposed global
  phase barrier. Real cross-region dependencies still force their E work and
  complete saturation; no per-function Planner or parallel search is presumed
  (§9.2).
- **HA15/HA16 default completion:** For a future declared family/target support
  domain, stop optional search at any accepted checkpoint and complete its
  guaranteed default O2/E/M route to an executable artifact. Reject in-domain
  acceptance that loses this route. Contrast a semantically legal LE whose
  materializer/input evidence is unconnected or outside the claim: report its
  unavailable boundary, with no support claim or invented artifact (§9.3).

- **HA08/HA13 planning repartition:** An accepted inlining or other call rewrite
  changes call edges and splits/merges a call SCC. Rederive the planning views
  and reassign future work while preserving inlined semantic dependencies,
  actual affected validation and retained-checkpoint correspondence. Retaining
  the initial task graph as immutable identity, dropping checks with a removed
  edge or applying lifecycle name-item collapse to planning partitions fails
  this case; no particular task graph algorithm is required (§§6, 8 and 9.2).
- **HA12/HA14/HA15/HA16 frozen-plan handoff:** A plan leaves encoding form,
  legalization, layout or fixup policy undecided. Final M rejects it instead of
  selecting, relaxing or silently repairing it; upstream must supply the settled
  plan through the declared complete default route. A complete plan can execute
  its prescribed calculations without planning discretion. Changing a consumed
  target definition or supporting observation invalidates its dependent validity
  and artifacts before reuse even with equal source/key; independent E material
  remains reusable only under its own unchanged observations. No implemented M
  or passed materialization test is asserted (§§5–6 and 9.3).

- **HA13/HA16 irreversible search effect (A):** O1 exploration of K1 needs a
  new compile-host external action to complete saturation, but K1 is then
  rejected and sibling K2 continues. With lawful isolation/reuse support, retain
  each description's real effect dependencies. Without it, refuse exploration
  before the irreversible action; never claim the candidate saturated or that
  discarding K1 undid an effect. K2 cannot inherit copied authority or a fictitious
  unchanged execution history (§9.1).
- **HA01/HA07 unexecuted guarded Pre consumer (B):** A's already formed unknown
  runtime-guarded arm describes a B call through an established operation
  description or sufficient exposed contract. B's Pre changes before the arm
  executes. Invalidate/revalidate the dependent formed material, summary support
  and other actual consumers before reuse, despite equal signature/key. Retain
  the guard; execute/prune neither future arm, invent no activation and select
  no future candidate. Proven stable outward observations may stop further
  propagation; source AST mentioning B alone supplies no such semantic input
  (§§6–7).
- **HA12/HA14 target/external premise change (C):** The source, signature and
  ordinary cache key stay equal while a consumed E-known arithmetic/trap,
  representation, alignment or semantic ABI fact changes in a new observation
  context, or external evidence expires/loses required validity support.
  Invalidate dependent E conclusions and O equivalence evidence, and separately
  affected machine plans/artifacts by their own dependencies. This is not HA14's
  pure machine-choice case with unchanged E facts. No source-hash shortcut or
  private target constant permits reuse; missing current evidence is unavailable.
  Historical snapshots/commits remain recorded; a false unsafe admission is not
  repaired by replacing its cached fact (§§5–6, 9–10).

- **HA13/HA14 plan fidelity:** Accepted LE requires runtime addition; a complete,
  encodable plan instead subtracts on inputs where results differ. M's exact
  encoding success supplies no LE-to-plan fidelity. Upstream must justify the
  E-observation correspondence, including effects/delivery/resource obligations,
  or reject the plan; no new M semantic checker is introduced (§9.3).
- **HA01/HA10/HA16 before-action revalidation:** A legal call consumes x and
  ends its generation. B's Pre then changes. Check the new Pre on that revision's
  corresponding before-call observations and input mapping, not the cached dead-x
  after-state. Missing before-state support prevents local reuse; a real failed
  new Pre still rejects. Observing the historical frontier revives nothing,
  reapplies no Post and performs no second Move/Kill (§§5.1, 7.1 and 10).
- **HA15/HA16 output completion:** M succeeds with the prescribed bytes/records,
  but the declared target still requires packaging/linking. Complete those
  guaranteed steps under settled policies before claiming an executable artifact;
  M success alone is insufficient. Missing completion in the supported domain
  fails the support claim, and a downstream choice/relaxation cannot silently
  repair the plan (§9.3).
- **HA12/HA17 negative-query scope:** A supported local E query proves no
  conflicting write in an observed interval. A rewrite inserts a relevant write
  while old action rows stay unchanged. Invalidate the absence/reorder conclusion
  through changed domain/range completeness support. Apply the same duty to
  additional effects or pending cleanup; missing completeness is unavailable,
  not an empty relation. This requires neither a whole-function pairwise table
  nor execution of unknown future branches (§§4 and 6).

- **HA03/HA06 stable result with changed support:** A's summary or local result
  formerly consumed X and now consumes Y with equivalent outward observations.
  Establish the new revision's Y dependency, guard/scope/completeness support
  and invalidation lookup before reuse, while permitting the appropriate stable
  stop without recalculating C. A later Y change must find and invalidate A's
  affected material and propagate any newly changed outward observation. Keeping
  only X because the earlier outputs were equal fails this case (§§6–7.3).
- **HA13/HA17 local revision preservation:** A justified swap of independent
  local subcontinuations retains valid unaffected K structure, projection
  correspondence and dependency support without representation-forced full-K
  expansion or all-index reconstruction. Contrast a swap whose actual effect,
  cleanup or other dependency crosses the region: validate every affected part,
  however far it reaches. Neither case transplants invalid old-position facts,
  mutates retained sibling checkpoints or promises a complexity bound (§§4, 9).

- **HA07/HA12/HA16 unordered siblings:** Siblings A and B start from one snapshot;
  A installs a new ordinary binding. B cannot read that binding just because
  the host evaluates or restores A first. File-local Seq still observes its own
  preceding actions. Established compatible contributions from siblings can join
  by ordinary algebra with first residency once; replacements and duplicate
  acquisitions retain their ordinary conflicts. Changing a contribution or join
  premise invalidates actual merged-result/conflict consumers, never creating
  filename/worker/cache ordering authority (§§2, 5.1 and 6).
- **HA07/HA12/HA15 reify closure support:** A cached ClosedInterface observation
  meets changed support that makes public capture/effect/lifecycle shape depend
  on reverse runtime flow. Invalidate admission/interface consumers and require
  current E closure evidence or the ordinary handler/wrapper before exposure;
  old cached shape and M cannot repair it. Contrast runtime choice within an
  already compile-determined regular sum, which remains legal (§§2 and 6).
- **HA12/HA14/HA16 materialization environment:** Keep source, frozen plan and
  target definition equal while M capabilities/version, output options or a
  consumed downstream toolchain input changes. Revalidate/rebuild only results
  whose consumed support no longer establishes validity: M validation, emitted
  output and final packaging/link artifacts have separate premises. Prior M
  success or matching target identity alone cannot authorize reuse; unchanged
  independent E/plan-semantic evidence need not be discarded (§9.3).

- **HA01/HA12/HA15 cached failure invalidation:** A's unchanged source previously
  failed B's Pre, or lacked required consumer/evidence. Relaxed Pre or newly
  available evidence invalidates that dependent cached failure. Check the new
  revision through the existing requiring E consumer; other false/missing
  premises can still reject/be unavailable. A key hit cannot make the old
  Diagnostic permanent; revalidation reopens no old selected invocation and
  replays no represented effect (§§5–6 and 10).
- **HA13/HA17 coherent combined queries:** A rewrite judgment obtains a
  no-conflict observation for K1, then Move/cleanup observations for changed K2.
  Individual validity does not authorize their combination: require a coherent
  judgment context or established correspondence/current support for the facts
  used. A new intervening conflict defeats the old premise. Lawful mapped reuse
  and explicit old/new equivalence comparison remain possible; affected new-K
  validation is still independent (§§4 and 9).

These additions specify preservation/acceptance boundaries only; they execute
no source program or optimization test.

### 11.2 Audited implementation evidence

These links describe the baseline code, not implementations added by this PR.

| Existing slice | Evidence and practical limit |
|---|---|
| Common transaction and fixed cleanup ordering | [semantic_continuation.rs](../../crates/lang_build/src/semantic_continuation.rs) `commit_action` checks cleanup/Pre and stages joint Post before publishing state/frontier. Its header explicitly excludes E saturation/source evaluation; its current fields do not carry complete expression/control/caller/CPS structure |
| Supplied lifecycle relations and name discovery | [lifecycle.rs](../../crates/lang_build/src/lifecycle.rs) has Pre/Post and stale/foreign proof checks, directed Color material and tests. `discover_value` supplies no formation/origin/Alive; formation/origin/Color fixture importers are test-only. `reify_value` ends at `LifecycleContinuationPending` after its earlier gates |
| Owner and compile identity | [semantic_owner.rs](../../crates/lang_build/src/semantic_owner.rs), [compile_key.rs](../../crates/lang_build/src/compile_key.rs) and [semantic_world.rs](../../crates/lang_build/src/semantic_world.rs) intern graph-local owners/instances and retain selected frames. Interning is identity admission, not result evaluation/cache validity, initialized self residency or persistent parent-homomorphism proof |
| Sealed selection and invocation storage | [ordinary_invocation.rs](../../crates/lang_build/src/ordinary_invocation.rs) retains candidate/frame and diagnoses selected source-body completion. Scratch-world publication alone is not integration with every common transaction/lifecycle producer. [ordinary_invocation_spine.rs](../../crates/lang_build/tests/ordinary_invocation_spine.rs) covers selected failure/no-reopen and production roster limits, not four-way source contract execution |
| Package artifact caching | [build.rs](../../crates/lang_build/src/build.rs) explicitly implements only in-memory exact-key hit/miss. Source/global implementation content and transitive dependency fingerprints affect the key; [static_build_graph.rs](../../crates/lang_build/tests/static_build_graph.rs) covers those rebuild/hit slices. This is not a semantic Pre/Post dependency index, selective invalidator or observation-stability engine |

The [roadmap](../planning/roadmap.md#10-structured-e-host-and-incremental-revalidation-readiness)
tracks future wiring. LC1–LC13, complete common-K/CPS, ordinary contract/source
producers, formed-body summaries and actual-exit validation remain pending.
This PR adds no execution consumer, contract-content invalidator, recursive
algorithm, optimizer or default machine materializer. Passing repository tests
does not establish HA01–HA18 execution or prove general summary fidelity.

### 11.3 Remaining representation and algorithm choices

The existing [open questions](../planning/open-questions.md) already cover the
concrete choices needed here; no closed semantic question is reopened:

- [Identity/persistence](../planning/open-questions.md#identity-and-canonical-representation): serialization and lawful cross-revision owner/action/name/generation correspondence, including supported nonbijective structural relations without changing identity laws, collision-safe lookup and parent-homomorphism evidence.
- [Continuation representation](../planning/open-questions.md#residual-and-serial-evaluation): structured K/residual frames, branchable/recoverable checkpoints, full CPS/delivery/control correspondence, before-action observation recovery and effect/overlay storage, without prescribing physical CPS layout; supported default-route domains, LE-to-plan fidelity support and complete terminating materialization/packaging algorithms remain future representation/implementation work.
- [Lifecycle algorithms](../planning/open-questions.md#lifetime-color-and-access): regular companions and contribution/incidence retention, supported boundary/summary equivalence and recursive-family evidence/rejection, scoped authority and current Open-sensitive cache requirements.
- [Infrastructure persistence](../planning/open-questions.md#owner-namespace-and-infrastructure-persistence): storing selection and general relation-domain/range completeness support and other dependency validity, fine-grained reverse-consumer lookup, history-relative effect accounting, E-grounded rewrite support and separate machine-artifact dependencies.

Chosen representations still need the corresponding owners' fidelity,
composition and termination evidence. A future implementation reports each
connected boundary and its actual tests separately from these design targets.
