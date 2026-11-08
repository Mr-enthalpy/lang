# Implementation Roadmap

Canonical meaning belongs to the [topic owners](../design/README.md).
This document records consumer coverage and dependency order. Passing storage
or syntax tests establishes their stated slice, not complete source evaluation.
Representation choices belong in [open questions](open-questions.md).

## Heap / IO / effect convergence frontier

Projection consumes capture specifications, not projection-site captured values.
Prove binder substitution for every admitted future explicit input/world and
conserve original resource custody/Drop separately. LC6 and FP01–FP08 remain
unconnected normative targets. Optional Result/Error adapters are explicit
ordinary calls; Error-shaped data performs no default effect or early return.

The [migration closure](heap-io-effect-migration.md) records M01–M13 replacements,
F01–F14 obligations and implementation phases, with
[paired acceptance](canonical-semantic-conformance.md#heap-io-and-effect-acceptance).
Current owners already define one meaning for Required/Initialized capture,
inherited InPlace AutoPre, explicit inputs after Function Pattern source erasure,
guaranteed heap submit, Step/Wait/repair/throw, complete original-K CPS
correspondence and compile-closed reify effects.
There is no pending semantic deletion or alternative compatibility behavior.

The connected slice is required-versus-initialized capture syntax/Norm and its
goldens, plus unavailable captured-call-entry/relation gates and unsupported
repair attachment diagnostic. Semantic capture binding, FnExtract, slice/MoveRepair,
global providers, heap library/submit, regular effects, wait/handler/throw/Miss,
yield/reify checks and generation-grain/ordinary ABI consumers remain pending.
The original common continuation's full CPS conversion/composition is a
foundational pending common-E obligation, including ordinary no-wait work,
nested pending callers, every expression/control/delivery context, fixed cleanup
and compile/reify projections. LC13 specifies its handoff; local wait splitting
or a native caller stack abandoned on suspension is insufficient.
Use the existing spanned unsupported source boundary; only a genuinely remaining
continuation may cross as Residual. Missing lifecycle/resource consumers cannot
silently execute a supplied host approximation or ignored capture list.

Dependency order: capture/Function Pattern/AutoPre -> global capability and heap
protocol plus original-K CPS/control/delivery composition ->
Wait/repair/effect/handler -> throw/PatternMiss/compile/reify ->
cross-stage composition and proved O/materialization. Each connected slice must
replace/gate its old path before execution is claimed.

## 1. One source and evaluation pipeline

```text
source -> weak tokens -> Raw AST -> Normalized AST
       -> typed owner / namespace resolution
       -> canonical E -> InvocationResult

EntryContinuation
  -> common E
  -> ordinary completion / actual remaining continuation / diagnostic
```

Raw AST preserves source and recovery. Norm is syntax-directed and non-semantic.
It performs no lookup, Pattern applicability, closure formation or execution.
The [unified owner](../design/unified-static-name-and-structural-semantics.md)
owns static/name/structural interpretation and instance formation.

## 2. Consumer dependency order

1. Independent PolicyMode={const,mut}, OpenPolicy={open,close} and
   Stage={compile,runtime}; total output demand before maxima.
2. Resolve once; ordinary associated Val2[()] entrance and exact immutable V_tau
   expansion; R_vis evidence and ordinary C_sigma preparation.
3. Stable CompilePartner receiver/call-entry pair and canonical inputs;
   CompileInstance before EnterBody, independent of result kind and P2;
   initialized ordinary self-name/resident reads.
4. Complete accessible-result closure, opening subjects and direct single
   instance-open self-root check before outward delivery.
5. Common action transaction/scheduler: affected Pre, one commit, joint Post,
   ordinary result delivery/completion and independent open/close.
6. V/S polarity and arbitrary $, shared bare-name navigation, structural
   formation with actual Val2 role witnesses and a complete type.
7. Ordinary struct:type->type helpers preserving structural registration;
   ordinary * / *= selection with checked read-transform-write.
8. Inherited file navigation and qualified let/name installation, serial files,
   unordered common-snapshot sibling overlays.
9. Requested-name producers accepting any ordinary result independently of
   NameExpr shape; ordinary ADL receiver/type-path overload families.
10. Joint classifier extraction across ranks, HoleOverride solving, full Path
    suffix consumers, bare-branch formation, dependencies, source lifecycle, fixed cleanup,
    ordinary remaining-continuation transport, callable reify P2 closed-boundary/dependency-cone wiring with
    scoped reverse migration and ordinary result delivery, and E saturation.

Every step preserves one identity/authority relation and selected no-reopen.

## 3. Substrate and source boundaries

| Relation | Representation / connected slice | Consumer boundary |
|---|---|---|
| Object and complete tau | Val1/Pattern/owned-Val2 normalization; immutable Core and V_tau observations; exact observed Core material and callspace transport into fresh bindings and constructor results; constructor candidates and ordinary Type(x).Val2[()] consume held Core resident witnesses | Changed nested resident material and unformed implementation families report unavailable; persistent captured-child representation and rich opaque payload normalization remain open |
| Name and owner identity | Parent-linked owners, root-qualified Holes, name/Place/generation coordinates; ordinary/type arguments share one resolved navigation, complete lexical chain and full host exposure | General NameValue/read, directory navigation through Objects, qualified formation and lexical Path aliases require ordinary Path consumers |
| Applicability | R_Gamma proofs, Hole valuations and parameter preparation; held ordinary classifier Core/whole transport before A; unknown classifier/role/named evidence stops before selection, while proved mismatches reject | Protected StructuralDefault, structural role families and richer extraction require consumers |
| Universe classification | Explicit first-order external classifiers and unavailable higher-universe observation boundary | Adjacent-rank classifier extraction, including Type(type)=(1 type), requires the ordinary rank consumer |
| Forced root extraction | Direct matching first; private actual scalar-root mismatch evidence, actual source Policy/horizon observation and one selected ordinary literal conversion; final relation checked without recursive insertion | Compound/return roots and general conversion source families require their common E/Ready consumers; unknown evidence remains unavailable |
| Policy | Explicit observation/demand/preference, migration and derived 2x2 capability views; argument/host carriers retain complete binding edges and reject non-unique required argument projections | General multi-edge joint projection, source deduction, operation-driven premises and full R_vis/C_sigma are separate obligations |
| Migration delivery | Selected declaration/target Mode contradiction rejects without reopening; supplied completed entries must satisfy Pair and exact delivered/declared/target Mode before outer scratch publication | Source bodies and independent actual delivery observations require common E; supplied-material validator tests do not establish successful source migration or post-production failure execution |
| Policy rank chain | Shape-validated arbitrary-depth prefixes of absent* runtime* compile*, two-rank windows including absent:absent; runtime first-present and full-chain compile demand material | Actual higher-rank classifier evidence, mixed output Hole solving and reverse-dependent E remain unconnected; a prefix/target supplies no producer witness |
| Return target | Active activation identity plus Ordinary/InPlace reachable-segment guard; Unknown preserves unbound events | Ordinary capability execution, completion propagation and delivery remain pending and must reuse the guard |
| Retained structural Paths | Canonical prospective-destination-world hard-A requirement for nonidentity compile-to-runtime same-Type migration | Candidate-specific Path recoverability evidence/consumer pending; do not replace with placement bans or selected failure |
| Selection | Exact receiver/call-entry enumeration, total demand, sealed selection and diagnostics | Unknown applicability cannot become fallback evidence; source completion requires common E |
| Instance identity | Parent/selected-partner/canonical-input interning; held complete tau inputs with Core projected from the same snapshot; independent ordinary self-name coordinate and exact selected frame; connected external-type delivery | Initialized self residency, accessible-result closure, dependency-sensitive current storage and general completion require common consumers |
| Structural relations | Layer-local schema normalization and a DirectPatternChild observer requiring both registration and current actual Val2 membership | Source type formation, struct helpers and ordinary * / *= require common producer Pre/Post and real role/call entries |
| Place | Binding/resident generations, Writable and borrow substrate | Source ref/share/rebind, initialization and invalidation require common transactions |
| Lifecycle | SemanticContinuation, LifeName, Region, supplied Pre, Kill/Preserve, fixed cleanup and Color; common transactions gate every action on earlier cleanup observations | Source producer facts, NLL/with, ReifyLife and destructors need ordinary control-flow facts; source actions must enter the common transaction |
| Physical source | Discovery/decoding/provenance and normalized fragments retaining inherited navigation; empty fragments form no semantic names or owners | Declarations under inherited navigation require the qualified ordinary name/Path consumer; EntryContinuation, sibling overlays and dependency projection need E composition |
| InvocationResult | Semantic result, residual and diagnostic transport | Residual requires a real remaining continuation; serial completion and residual ABI remain separate |
| Closure / self / Function Pattern | Capture syntax/Pattern roots, contextual formal-frame carriers, ordinary sealed selection and transaction substrate | Requested UseKind/DemandStage classification, exclusive value-capture modes, invocation-indexed CaptureValueProduct, required binding, candidate-local implicit relation evidence, actual Param transfer and directional FnExtract/FnRequest are pending; generic closure heads do not implement RHS requests |
| Delivery ABIKind | Canonical E law: FreshLet ABI1, ExistingTarget ABI2 | Delivery target consumer wiring and physical input-frontier/target-Place materialization remain pending; RVO/NRVO supplies no selection evidence |

A missing canonical consumer reports unavailable or preserves an established
remaining continuation. Representation supplies no substitute evaluator,
identity, result or authority. Production invocation consumes the actual
semantic call entry; graph declarations are rendering material.

## 4. Common E acceptance gates

Visible, Ready and LegalToExecute have distinct consumers. Visibility cannot
establish a successful producer. The diagnostic-only selected-source-body
frontier must be absorbed by common E; it must not accumulate initializer,
binding, nested-call and return execution as another evaluator.

Every semantic action obeys the cleanup gate:

```text
OutstandingCleanupBefore(k) => action may not commit across that point
```

This includes actions without a lifecycle projection. The common transaction
checks a fixed cleanup prefix and current obligation observations before Pre;
unknown, foreign or future facts cannot grant permission. Source scheduler
integration must route every action through this transaction. Formation, origin
and Color Post
publish in the producer's same scratch transaction with every other affected
projection. Commit followed by separate fact mutation is not a producer handoff.
Storage staging alone proves neither Ready, Pre, common commit nor joint Post.
Tests reject stale/foreign evidence and failure of any Pre/Post without
publication, and verify uniform cleanup gating and actual producer authority.

## 5. Instance and result obligations

Every selected compile call, including scalar/Product/external-type
results, has a stable instance before body entry. Parent, actual selected
receiver/call-entry pair and canonical inputs determine identity. Body/result
material and provenance do not. Conflicting bodies for equal keys are conflicts.

Read_name and initialized Read_resident of InvokeName(I) do not EnterBody.
Reacquisition reads current storage; old complete snapshots stay immutable.
Cache reuse cannot replay initialization, restore consumed residents or grant
saved authority. Writes, moves and invalidations keep their ordinary effects.

Reach includes accessible members, Products, captures, references and share
targets. Required unknown reachability is unavailable. An instance-open self-root
type must be the single direct complete type formal return result. External types and ordinary
values are legal without acquiring that NameExpr. ReturnEvent and delivery remain
real boundaries even for equal self returns.

Only omission completes: open plus known OpenHere gives mut; otherwise const.
Required unknown OpenHere is unavailable. Explicit and deduced modes survive.
Delivery precedes outward Complete/Close; inputs keep their own subjects/lifetimes.

## 6. Structural, Path and name obligations

$ carries opposite-context syntax in both V and S, with arbitrary source
nesting, Hole identity and parity. It supplies no Read, Stage change, borrow,
mode, OpenHere or lifecycle fact. S type formation atomically publishes actual
Val2 witnesses, registered roles and a complete type.

Field/ADL consumers use ordinary semantic callable entries and actual struct
helper registration, independently of graph rendering. General execution remains
unavailable where those producers are not connected.
struct preserves DirectPatternChild, ConstructEdge, ExtractEdge and FieldView
while adding ordinary helpers. * selects ordinary type composition; *= performs
checked read-transform-write without publishing partial state.

The [Product owner](../design/patterns-overload/product-repetition-and-unpacking.md)
also owns ordinary repetition selection, contextual exposure, Val1 identity and
Policy-indexed unpacking. Current syntax preserves `_`, polarity flips,
ordinary `*`/`?` calls and bare Product association. Repetition operator-Pattern
lowering, complete wrapper formation, singleton semantic expansion and `pack?`
execution need the ordinary consumers; runtime variable-length processing
additionally needs iteration. External Pattern queries must remain unavailable
when embedded in otherwise connected Sequences; partial preparation cannot
erase them. None of these gaps grants a special Product builtin or decoder.

File hierarchy supplies inherited navigation only. let inner=bool:: under a
uses destination inner::a at the same NameExpr level as the RHS. Root and
filenames add no segment, type-composition action or authority.

Requested-name producers accept any declared ordinary result. Concrete heads,
wildcards and requested HoleRef use ordinary applicability/specificity.
ADL receiver calls and type-path progression share that relation:
Each selected type-Path candidate advances the canonical Path; object-method
candidates remain ordinary calls. Double-dot contracts pipeline-dot.

Lexical === forms Path material once in the old environment before installation;
it caches no terminal binding and creates no entity or recursive alias thunk.
With items retain full Paths and forward references. Instantiation follows
complete declarations/Self/control-flow on actual layers; absence adds no
constraint. Cleanup points are globally fixed before same-point precedence and
reverse declaration order, then lifetime observation.

## 7. Continuing evaluator work

Closure completion forms full tau_C with distinct c_C, A_C and () roles at a
finite implementation leaf. Ordinary let binds the type; established same-name
buckets jointly form target-anchored members against one snapshot. Legal
ordinary actions retain their meaning without failed execution retry.

Dependencies separate requirements, selected ordinary realization and layout.
Realization is unique up to observational equivalence for one selected action.
Explicit/automatic occurrences retain identity. Automatic/initialized sources
are retained; required specifications bind at each admitted new call and their
observed sources enter canonical inputs.
Dependency lifetime and escape remain ordinary checked handoffs.

Serial completion consumes path-sensitive UnitDiscard, unit fallthrough,
ReturnEvent and target inference. The nearest Ordinary closure bounds the
reachable return segment; InPlace adds no barrier. Explicit/capability targets
cannot cross it and unknown boundary evidence stays unavailable. AST-list
position is not semantic tailness.
R_vis/C_sigma, pending continuation material, dependency readiness, source lifecycle,
current instance state and E saturation share one continuation.

Bootstrap families use source-expressible relations where possible. Host
capabilities return ordinary Objects. Planner/optimizer/layout/storage introduce
no private facts. Equivalent rewrites revalidate affected projections before
E saturates Ready work.

## 8. Acceptance evidence

The [closure/self/Function Pattern matrix](canonical-semantic-conformance.md#closure-self-and-directional-function-pattern-acceptance)
records T01–T22 as normative targets, not executable success. Current
`lang_syntax/tests/callable_tail_dot_pack.rs` covers capture binding syntax and
contextual first-formal/Pack boundaries. Its regression
`self_only_heads_keep_self_without_explicit_formals` checks existing ordinary,
in-place and direct let () head carriers: self remains present and explicit
formals are empty. It does not execute default expansion or prove runtime unit
matching. `lang_build/tests/ordinary_invocation_spine.rs`
checks selected capability/lifecycle failure with a runner-up, proving no reopen
for that substrate. `policy_rank_chain.rs` checks rank-prefix material only.
Source Function Pattern matching/request and candidate-local self relation
solving are not connected: current applicability solves explicit parameters;
closure atoms in Pattern context remain unsupported. Function Pattern syntax
must retain an explicit `->` boundary on either side; arrowless material is
Bare Product and cannot acquire independent Pout from Pin. This is distinct
from ordinary/in-place closure expressions with optional return clauses.
Dedicated Function Pattern parsing/diagnostics remain pending; normative
arrow-positive/negative cases do not claim generic-head implementation coverage.
No executable closure evaluator is supplied by the original closure revision.

The [call-sugar extension matrix](canonical-semantic-conformance.md#empty-invocation-and-default-formal-sugar-acceptance)
adds DS01–DS28 as normative targets. Its
[surface contract](../public/normalized-surface-semantics.md#15-canonical-call-sugars-consumer-pending)
separates the connected empty-Product slice from pending do/default parsing and
execution. Empty parentheses normalize to zero entries across ordinary Product,
continuation, double-dot and bracket paths; comma-hole Units and source spans
remain intact. Norm goldens, syntax/argument-shape tests, canonical input
identity and ordinary zero-versus-one fixed-arity selection cover this slice of
DS02–DS03. No downstream Unit filter was introduced. Param defaults are still
not parsed and no generated default-member family is formed. Remaining order:

1. Preserve contextual do/default clauses, spans and syntax recovery. Add their
   token/AST/Norm/diagnostic goldens without undoing the connected empty-Product,
   first-product-only continuation, nesting/barrier and comma-hole laws.
2. Produce unresolved fixed-arity expansion material with contextual self
   retained, never defaulted, and (self) matching unit explicit input. Preserve
   arity-growing ordinary forwarding targets without duplicating source body/
   capture initializer occurrences. Connect common-snapshot standalone/contribution
   formation, target homes, ordinary dependencies and atomic family publication.
   Direct let () material installs ordinary associated entries with the original
   receiver unchanged, no anonymous receiver or implied V_T registration.
3. Connect ordinary wrapper invocation/default scopes/Move/return delivery.
   Verify effect counts, collisions, source self, unit return, Policy demands,
   lifetime/authority failures and no reopen. Existing optional initializer
   carriers or generic closure nodes cannot stand in for this consumer.

Lifecycle SCC and machine materialization remain separate; the sugar grants
neither recurrence convergence nor automatic tco/loop reuse.
Direct current-owner `let ()` expansion requires the same ordinary call and
role-preserving installation consumers, not a special evaluator. Its shorter
entry calls the existing receiver with one more explicit position; ordinary
competing entries/ambiguity and no-reopen remain observable at that new call.

The [forwarding Warning](../design/symbol-world/function-object-call-model.md#1221-non-semantic-forwarding-warning)
is a non-blocking diagnostic follow-up, not a change to whole-target forwarding
or DS22's legal different-winner outcome. After ordinary selected-entry evidence
is available, add proof-gated reporting and DS24–DS28 regressions: no false
deviation on unknown evidence, candidate-identity comparison, identical execution/
selection with or without Warning, ordinary explicit-constraint failures and
sealed-selection preservation. Do not add an implicit filter, default-specific
Error or speculative evaluation. No warning implementation is claimed here.

Two independent implementation areas consume the laws:

- Lifecycle programs: actual calls/captures, Region/origin/generation and
  Pre/Post/MoveEffect feed ordinary value computations and boundary transforms.
  Finite regular-access SCC representation and recursion of those programs
  need their own justified domains/evidence or explicit rejection/unavailability.
  No universal fixed-point solver, convergence theorem or retry is assumed.
- Machine materialization: SelectedCall, normalized PassMoves, delivery target,
  E-selected ABIKind, lifetime evidence and machine facts yield checked input
  frontier/ABI1 overwrite or ABI2 target delivery and proved tco/loop slot reuse.
  Physical stack/FFI encoding cannot alter E's convention or choose lifecycle facts.

Each phase runs cargo fmt --all, cargo check --all-targets, targeted tests and
git diff --check. Final integration runs cargo test and an active-tree
terminology/authority scan excluding history. Coverage includes positive/negative
behavior, identity/equality, non-derivability, authority uniqueness, no reopen,
effects and failure without publication. Executed coverage and remaining source
frontiers are reported separately from canonical laws.

## 9. Lifetime value-program wiring

The [lifetime owner](../design/lifetime/lifetime-policy-and-overload-boundary.md)
now specifies ordinary lifecycle computation, bootstrap responsibilities and
library protocols. Its [four walkthroughs](../design/lifetime/lifetime-algebra-walkthroughs.md)
and [LA acceptance scenarios](canonical-semantic-conformance.md#lifetime-value-computation-and-library-design-acceptance)
close design responsibilities; they supply neither source execution nor a
general recursion proof. [LC1–LC8](../contracts/lifetime-program-consumers.md)
give every future consumer the same eight-field handoff template.

Current implementation evidence remains limited: name discovery is not formation
or origin authority; lifecycle ref/share/rebind and structured field producers
are pending; `reify_value` reports `LifecycleContinuationPending` after its
earlier gates. Common transaction tests establish supplied Pre/cleanup checking
and joint Post publication. Generic LifetimePre/Post syntax and Rust lifetime
values do not execute contract expressions, automatic Post or the library.

| Consumer | Missing source responsibility | Dependency / acceptance boundary |
|---|---|---|
| LC1 | Atomic Pattern/structure companions and actual registered field pairing | Ordinary formation, structural roles and selected ADL; field commutation |
| LC2 | Whole-K projection, business Stage forgetting and complete-zone provenance | Common E, selected actions/control/activation/delivery; no future execution or reselect |
| LC3 | Source action facts, name/generation/origin, regions/colors and real internal APIs | Ordinary producers, fixed cleanup and common transaction; no double consumption |
| LC4 | Ordinary compile contract evaluation, self-root check/admit formation and mapped four-way calls | Existing CompileInstance/OpenHere/result delivery and actual world authority; failed Pre publishes no success |
| LC5 | Formed guarded-body structural summary and actual-exit compression/validation | LC1–LC4 supply selected descriptions/formal roles separately from reached history; each legal actual Post implies its mapped advance summary without future execution |
| LC6 | Exposed Function Pattern defaults/implication and justified recursive interfaces | FnExtract erasure and ordinary contract computation; no implementation authority recovery |
| LC7 | Safeid, batch partition, alias windows and partbox ordinary programs plus trusted runtime facilities | True external-success facts, actual source authority, intervals and materializer protocol; no default success |
| LC8 | Finite regular access and cropping, with proven adopted recursive combinations | LC1/LC3 material and defined Pattern relations; no restored terminal permission |

The regular SCC collapse placement is fixed at Ordinary/participating-InPlace
boundaries; it is not an optimizer-selected algorithm. The owner's
[participation relation](../design/lifetime/lifetime-policy-and-overload-boundary.md#participation-relation)
uses actual cyclic name-item/edge contributions and preserves remaining outer
fine material across successive exits. Initial part issuance likewise needs
an [actual unallocated full-use source](../design/lifetime/lifetime-policy-and-overload-boundary.md#131-legitimate-initial-issuance-and-source-preservation),
or a real trusted acquisition, before the accounting programs can consume it.

This is a consumer dependency graph, not a pipeline of independent executors.
Connect existing language consumers first, then actual bootstrap observations
and action publication, ordinary contracts/boundary programs and regular/library
composition as their dependencies become available. API spelling, encoding,
automata/cache algorithms and machine cost models may evolve within these
contracts. Every adopted recursive family must state its domain and evidence;
finite regularity supplies neither termination of arbitrary programs nor TCO.

The expanded [source transcription](../design/lifetime/lifetime-algebra-walkthroughs.md#lifetime-transcription)
and [contract construction program](../design/lifetime/lifetime-algebra-walkthroughs.md#ordinary-self-root-contract-program)
add these concrete LC2/LC4/LC5 obligations without new Rust coverage:

- Retain actual selected counterpart entries, source function/InPlace boundary
  descriptors and ordinary recursive/return structure. Available source control
  may drive the program; unknown control remains in the real continuation.
- Supply the formed semantic guarded-body material and selected structural
  descriptions for advance summary transformation independently of actual Exit.
  Connect the separate actual-exit consumer and its mapped summary validation;
  neither speculative branch execution nor one late path completes a missing
  advance summary. See LC5's two existing boundary contracts.
- Wire initial complete structural formation at InvokeName(I) under its actual
  fresh window and one-shot authority before resident self/member reads. The
  schematic initialize_empty_root source call is not currently implemented.
- Install check/admit through ordinary qualified name/ref/first-write actions,
  retaining owned compacted dependencies and direct self-root result delivery.
- Give safe component adoption its actual activation/cut/generation/output
  authority and derivability Pre independently of any user check bool. An
  unrestricted semantic-world setter is not an admissible implementation.
- Terminate the justification dependencies of each adopted program in the fixed
  bootstrap basis. A cycle through its own not-yet-formed contract or automatic
  Post supplies no evidence; missing consumers remain unavailable.

Future executable acceptance must use real source producers and actions. Do
not substitute a standalone checker, source-string recognition, supplied-fact
fixtures, invented residual or default permission. Until connected, report a
spanned unavailable diagnostic, or transport only an actually established
remaining continuation. Keep executed coverage, design scenarios and unproved
obligations separately visible in each implementation PR.
