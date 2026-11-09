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
| [Owner/namespace identity](semantic-owner-namespace-graph.md#identity) and [compile construction](compile-construction-boundary.md) | Legal parent/selected-input identity, exact held snapshots and distinct current instance/member storage | §§4–5: lawful revision correspondence and state-sensitive cache reuse |
| [LC2](lifetime-program-consumers.md#lc2-whole-continuation-projection-and-projection-complete-clauses) / [LC13](lifetime-program-consumers.md#lc13-whole-original-k-cps-correspondence-pending) | Common actions/guards, projection-complete provenance, actual delivery and pending caller suffixes | §§2 and 9: synchronized observations and new-position rewrite validation |
| [LC4](lifetime-program-consumers.md#lc4-contract-expressions-and-four-way-boundary-consumption) / [LC6](lifetime-program-consumers.md#lc6-function-pattern-and-recursive-interface-consumption) | Pre/Post relation content, scoped four-way mappings and interface-compatibility assumptions | §§6–7: invalidate real call and interface consumers despite equal signature/ABI shape |
| [LC5](lifetime-program-consumers.md#lc5-exit-compression-and-automatic-post) / [LC8](lifetime-program-consumers.md#lc8-regular-access-and-cropping-programs) | Formed guarded summary inputs, separate actual-exit history, original companions and contribution lineage | §§7.2–8: summary propagation, stable observations and prescribed boundary-local collapse |
| [Shared action/lifecycle publication](semantic-values-and-extraction-interfaces.md#shared-action-and-lifecycle-handoff) and [migration delivery](policy-migration.md) | Actual cleanup/Pre evidence, selected frontier and successful joint producer/projection Post | §10: current-world validation without replay or partial publication |

These links assign preservation responsibilities to existing consumers; they
do not add an evaluator phase or imply that those consumers are connected.
The [conformance index](../planning/canonical-semantic-conformance.md#e-host-preservation-and-incremental-revalidation-targets)
maps HA groups to these handoffs; the [roadmap](../planning/roadmap.md#10-structured-e-host-and-incremental-revalidation-readiness)
records their pending wiring. Missing upstream material uses §10 at its actual
requiring boundary, rather than a host substitute.

## 2. Common K, LE and projection correspondence

Basis: E §§2–3, 7.1 and 11, lifecycle §2.1.1, effect §§2–3 and LC2/LC13.
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

Relational observations MUST retain sufficient subject/target, K position or
interval, name/generation, scope, guard, selected operation, supporting premises
and availability to justify their actual consumer. Same name, same Place, same
reference target and overlapping storage ranges are different judgments.
MustAlias and NoAlias observations can both matter; unknown is not NoAlias.
Region/Color, access conflicts, effects, cleanup and delivery keep their actual
directed relations and source-choice correlations. No universal alias query
API or decision algorithm is introduced. Missing relation/guard evidence grants
no local reorder permission.

The host MUST support queries over the real affected relation/dependency range
without requiring every local query to expand a flat table of all pairwise
relations in the function merely because of its storage model. Shared structure,
on-demand computation and local summaries MAY provide this capability. A query
may still require substantial real dependency work; this is no constant-time
complexity promise. Every accelerated answer remains justified by E, with
unavailable required evidence handled under §10.

## 5. Cacheable material and current-state premises

Basis: invocation §§1–2 and 4–6, dependency §§2–5 and 7, build §5,
type/Place identity and lifecycle §§2.1.3–2.1.8 and 9.4.

The host MUST preserve these distinct reuse obligations, even if one physical
cache stores all three classes:

| Material | Required observation on reuse | Excluded inference / missing-material boundary |
|---|---|---|
| Immutable complete snapshot | The exact held complete tau/callspace and the Core projection of that same snapshot | Current registry/resident cannot reconstruct a missing old snapshot; required missing snapshot is unavailable |
| Current-world fact | Actual K/action/cut, Name/Place/resident generation, Region/Color/access, current OpenHere/Writable, Move and cleanup obligations | Old success grants no present read/write/transfer permission; changed or unknown premises require E revalidation or §10 failure |
| Replayable computation/summary material | Established parent-neutral material, ordinary source/selection provenance, observed inputs, dependencies and validity conditions | Cache hit does not replay one-time initialization, source acquisition, external effects, Kill/Drop or sealed selection; missing correspondence prevents reuse |

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

Fingerprints/versions MAY locate reuse candidates. The host MUST account for
collisions and changes of compiler/semantic environment in cache validation;
a digest or matching signature cannot be the final identity, equivalence or
permission proof. Conservative recomputation is permitted when finer evidence
is absent. It still observes §10's no-replay/publication boundary.

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
| Actual Pre/Post relation content, entry/result mapping and exposed interface assumptions | Caller Pre.check, caller Post.admit, subsequent action Pre and Function Pattern contract compatibility |
| Formed guarded body, bound roles, selected-operation descriptions or sufficient exposed contracts | Formation-time Post_summary and its support |
| Current names/generations, residents, Region/Color/access, cleanup and Policy | Action legality, actual Post, delivery and resource responsibility |
| Original regular companions, guarded item/edge contributions and boundary mappings | Supported SCC collapse, local compression and recursive/modular composition |
| E/machine observations actually consumed by a future materialization decision | That machine plan/artifact, independently of unrelated semantic material |

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

1. Recover the call position and its actual selected candidate/projection/frame,
   implicit Param/captures, bound required inputs, explicit arguments,
   result/activation mapping and current premises.
2. Revalidate caller `Pre.check` under that mapping in current E/K. Failure
   prevents the action; body `Pre.admit` supplies no repair and selection does
   not reopen.
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

For a proposed O rewrite the host MUST retain lawful original/new K mappings
for actions, names/generations, calls, guards, cuts and delivery, and subject
affected conclusions to `E_affected` at their new positions. Old facts can guide
a candidate but cannot authorize it. Inlining, fusion/splitting or moving work
between functions cannot erase dependencies because an old call edge disappears.
Missing correspondence prevents acceptance of that candidate.

Planner organization MAY follow actual functions/SCCs and nested subcontinuations.
No global pass order, per-function planner instance or optimizer-specific alias,
effect, lifecycle or Stage truth system follows. Every semantic premise comes
from E. The host MUST distinguish an accepted legal revision from an unvalidated
candidate, so stopping future search preserves a legal continuation. Candidate
failure leaves the accepted revision intact, with no fabricated successful Post.

The host MUST preserve the semantic material needed for a legal default
materialization path from accepted residue: common-K/CPS correspondence, selected
calls, normalized terminal Move and actual repair, delivery target/E-selected
ABIKind, lifetime/cleanup/resource obligations and machine facts. If that path
requires strategy choices, they belong to a future default O2, not hidden E
rewrites. M executes an upstream-decided target-machine plan; this contract
freezes neither that plan's structure nor O/Planner/M entry points. Required
unavailable material blocks the relevant future handoff, not an invented plan.

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
| HA12 | Compile material/key hits while OpenHere or input dependencies change | Recheck actual current requirements; canonicalize newly observed inputs without rebinding a sealed call | Close/spent authority or invalid dependencies defeat reuse, never restore resident/write power; missing evidence unavailable; §5 |
| HA13 | Future O inlines a call, removing its graph edge while preserving semantic dependencies | Carry lawful original/new K mappings and the inlined contract/source dependencies; run affected E | Accept only after new-position validation; missing mapping blocks the candidate, preserving accepted K; §§6, 9 |
| HA14 | Only machine materialization decisions change, with E observations unchanged | Invalidate machine consumers of those decisions by their actual dependencies | Independent semantic material can remain valid; a semantic effect of the decision still requires E; §9 |
| HA15 | Lifecycle consumer, source/origin, formed description or automatic Post evidence is absent | Report the actual requiring unavailable Diagnostic | No synthetic Residual, None/unknown, NoDelta or successful contract; §§3, 7.2, 10 |
| HA16 | Action/late component fails, earlier cleanup is outstanding, or cached execution contains Move/Drop | Apply cleanup/Pre/joint-publication gate and retain actual consumption/history | No partial success or duplicate effect/cleanup; previous independent commits remain, cache reuse replays none; §§5, 10 |
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

- [Identity/persistence](../planning/open-questions.md#identity-and-canonical-representation): serialization and lawful cross-revision owner/action/name/generation correspondence, collision-safe lookup and parent-homomorphism evidence.
- [Continuation representation](../planning/open-questions.md#residual-and-serial-evaluation): structured K/residual frames, full CPS/delivery/control correspondence and effect/overlay storage, without prescribing physical CPS layout.
- [Lifecycle algorithms](../planning/open-questions.md#lifetime-color-and-access): regular companions and contribution/incidence retention, supported boundary/summary equivalence and recursive-family evidence/rejection, scoped authority and current Open-sensitive cache requirements.
- [Infrastructure persistence](../planning/open-questions.md#owner-namespace-and-infrastructure-persistence): storing dependency observations/validity and maintaining fine-grained reverse-consumer lookup, state-sensitive reuse and separate machine-artifact dependencies.

Chosen representations still need the corresponding owners' fidelity,
composition and termination evidence. A future implementation reports each
connected boundary and its actual tests separately from these design targets.
