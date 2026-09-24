# Roadmap

This document separates implemented storage/consumer slices from canonical
semantics. Existing tests of a carrier do not establish that the migrated
source semantics is connected. The [topic index](../design/README.md) owns the
semantic contracts.

This document records current implementation frontiers. Canonical meaning is
owned by the topic documents under `spec/design/`; unresolved decisions are
owned by `spec/planning/open-questions.md`.

## Frontend

The frontend pipeline is:

```text
source text -> tokens -> Raw AST -> Normalized AST (+ diagnostics)
```

Raw AST preserves source shape. Normalized AST is desugared but remains
non-semantic; it is not HIR and does not perform lookup, type checking,
Pattern interpretation, lifetime validation, or evaluation.

## Current semantic architecture

The semantic pipeline is:

```text
Normalized AST
  -> typed owner / namespace resolution
  -> canonical semantic entities and views
  -> R_vis visibility/input evidence and ordinary C_sigma preparation
  -> hard relational Pattern applicability and fallback suppression
  -> Policy preference
  -> unique sealed invocation
  -> DynamicLegality
  -> execution
  -> InvocationResult
```

The implementation in `crates/lang_build` establishes the following positive
semantic vocabulary:

- `Object = <Val1?, Pattern, Val2>` with complete ordinary normalization;
- proof-relevant `R_Gamma(P,c,rho)` Pattern applicability and Hole valuation;
- complete `tau = bind alpha.<Core(tau), V_tau[alpha]>` with immutable callspace
  snapshots;
- separate name-binding, semantic value, Place, resident generation, and
  ProjectionSlot identities;
- `PolicyPair`, primitive `PolicyMode = {const, plain, mut}`,
  `ResultPolicyDemand`, and independent 3×3 `CapabilityRealization`;
- one name-resolution result followed by context projection;
- value → exact complete type → associated `()` call projection;
- sealed candidate selection, post-selection DynamicLegality, and no reopen;
- candidate-driven same-Type Policy migration and PolicyLet result-demand
  boundaries;
- exact abstract literal values followed by ordinary construction;
- `OpenHere`, ConstructionAuthority, Writable, pure `extend`, and place-level
  `inject`;
- unified `InvocationResult`, complete-type `struct` result, and structural
  MetaInstance root identity;
- shared SemanticContinuation substrate with LifeName, Region, Pre/Post, and
  an extensible directed Color algebra.

Storage, graph rendering, and primitive execution are implementation layers;
identity, selection, result class, and legality remain properties of the
semantic relations above.

## Semantic substrate coverage

| Relation | Carrier | Production consumer | Status |
|---|---|---|---|
| Object Norm | Val1/Pattern/owned-Val2 observation | equality, Core and argument identity | Implemented |
| complete tau | Core + immutable V_tau + whole observation | type binding and ordinary call projection | Implemented |
| base R_Gamma and Hole valuation | relational proof | ordinary parameter A-stage | Implemented |
| DirectPatternChild + StructuralDefault | relation interfaces | protected structural extraction | Consumer pending |
| PolicyMode / demand / preference | explicit PolicyView and ResultPolicyDemand | ordinary selection, migration, PolicyLet | Implemented |
| CapabilityRealization | candidate-local 3×3 table | selected operation premise formation | Consumer pending |
| Place / resident generation | Place and ProjectionSlot | binding, Writable and borrow substrate | Implemented; source operation coverage pending |
| DynamicLegality | sealed post-selection validator | supplied capability/place/lifecycle premises | Implemented; automatic premise formation pending |
| InvocationResult | declared result class + semantic payload/residual/diagnostic | connected ordinary and core/meta invocation | Implemented; residual transport remains Open |
| OpenHere / construction | authority, window, Writable and write algebra | meta construction and inject | Base checks implemented; invocation dependency propagation pending |
| Meta instances | instance name/type + P1 meta/plain + dependency sources | instance/member current-state lookup and derived A | Consumer pending |
| abstract literals | exact abstract values and construction requests | annotated construction and Policy migration | Implemented |
| SemanticContinuation | lifecycle machine and event ledger | world-owned registration | source action/cleanup wiring pending |
| Color/access | extensible directed relations and provider interface | lifecycle Pre validation | access-tree construction Open |

“Consumer pending” means the canonical relation exists and no substitute
relation is used; it does not mean the language rule is undecided.

## Source and evaluator connection frontier

The next implementation frontier connects source occurrences to the canonical
relations under the alignment gates below:

1. protected structural extraction through `StructuralDefault` before C0;
2. operation-driven capability, Writable, authority, and lifecycle premises;
3. source `ref` / `share` / `rebind` and invalidation actions;
4. source use/move/drop/`@` events on the world-owned continuation;
5. cleanup placement before lifecycle observation;
6. Residual and Diagnostic transport through the unified invocation boundary;
7. derived associated forwarding that captures the base complete-type
   snapshot and creates anchored forwarding instances;
8. block-local `let ===` lexical entries that create no semantic entity.

Each wiring step must preserve unique selection and no reopen.

## Serial compile evaluation

After source operations are connected, the evaluator may execute them along a
single semantic continuation:

```text
resolved operation
  -> Pre
  -> committed action
  -> Post
  -> next SemanticContinuation position
```

This frontier owns control-flow sequencing, fixed cleanup placement, residual
continuation transport, and serial compile evaluation. It does not create a
separate meta value ontology or lifetime universe.

## Bootstrap and source authority

A compiler implementation is not evidence that an operation is a permanent
semantic primitive. Every builtin family is classified by its target role:

| Role | Meaning |
|---|---|
| `BootstrapSeed` | establishes the initial source-expressible environment |
| `SourceDefinitionPending` | language semantics can express the operation; source definition is not connected yet |
| `IntrinsicObservation` | exposes implementation facts that source cannot synthesize, without deciding legality |
| `SemanticPrimitive` | permanent authority, requiring an explicit non-bootstrappability proof |

Current families:

| Family | Role | Semantic authority |
|---|---|---|
| exact abstract-literal formation | `BootstrapSeed` | exact spelling/family observation |
| concrete literal constructors | `SourceDefinitionPending` | ordinary candidate selection |
| construction and same-Type migration families | `SourceDefinitionPending` | ordinary selection + DynamicLegality |
| capability realization entries | `SourceDefinitionPending` | candidate declarations |
| StructuralDefault providers | `SourceDefinitionPending` | `R_Gamma` |
| associated state A | `SourceDefinitionPending` | ordinary meta instance type + Val2 group/place algebra |
| singleton-Val2 compile extraction | `SourceDefinitionPending` (builtin bootstrap permitted) | closed type + exactly one ordinary Val2 entry + ordinary value read |
| lifecycle move/copy/drop algebra | `SourceDefinitionPending` | lifecycle Pre/commit/Post relations |
| interning, graph allocation, continuation-position observation | `IntrinsicObservation` | canonical relations consuming those observations |

No current family is classified as `SemanticPrimitive`.

## Open representation boundaries

The implementation must provide extension points without choosing final forms
for:

- `TypeValueId` storage encoding;
- full Pattern canonical-space representation;
- Color syntax and storage;
- access-tree construction;
- persistent owner/root encoding;
- complete later overload filters and named strategies;
- Residual IR and continuation ABI;
- cleanup scheduling IR;
- character surface and machine type catalog;
- closure capture layout, HIR, backend lowering, and code generation.

These questions remain in `spec/planning/open-questions.md`. Missing source
wiring is not an open semantic question.

## Physical input and infrastructure migration

The target is Compile(Level) through main.lang anchoring, neutral physical
normalization and E of EntryContinuation(L,F_main), with main.P2=runtime. Child directories desugar to ordinary
typed name creation, explicit borrow, ordinary directory type initialization,
and body evaluation under that reference;
root levels and filenames add no segment. Each file is serial; sibling
blocks use common-snapshot overlays and unordered join. Actual effects produce
the dependency projection. Host calls return ordinary Objects and target facts.

Current code still accepts configured source roots and a package/workspace graph,
then consumes sorted files with per-declaration shared-world commits. These
paths require migration; they cannot remain semantic input alternatives.
Cache, discovery, decoding, diagnostics and artifact persistence remain useful
engineering facilities after their inputs and effects obey the source model.

## Canonical/source alignment gates

- Replace the optional pure-P/sibling cluster carrier semantics with named-type
  synthesis and explicit OverloadGroup aggregation. Existing Rust cluster/result
  labels are implementation encodings, not the target ontology. In particular,
  DeclaredResultClass::ClusterSymbol must be removed or re-encoded as private
  implementation material; it is not an ordinary semantic result class.
- Connect typed structural NameExpr creation, explicit Place borrowing without
  reading, and ordinary first-write initialization. Uninitialized is non-Object
  state. Qualified formation resolves a structural root and checks its current
  type's OpenHere; do not require parent Writable or parent mut type ref. Test
  formation under an open type with no parent write capability, rejection for
  a closed type, and distinct NameCoords for equal type values at distinct roots.
  Keep generated-after-Close realization outside this explicit formation path.
  Connect the narrow meta type/ref qualification and explicit ConfirmMut
  consumer alongside direct mut borrowing. Test same-target/generation/capability
  coherence, no amplification on non-Writable targets, Close invalidation of
  both routes and saved writes, replacement without reference retargeting,
  rejection of arbitrary meta X ref, and no implicit chaining/reopen. This
  consumer is pending; current Rust capability carriers do not implement it.
  Do not install a dummy type or return a ref from creation. Require
  initialized retained names being published at Close, without enumerating all
  future generative coordinates. Current let parser carriers
  are pending alignment; no canonical structural let=compound is implied.
  Connect initializer-free lexical P let name:t through the same typed-name
  rules at a lexical destination. Preserve P let name=rhs as a complete binding
  with RHS inference, not a default-:type declaration plus assignment.
  Connect initial borrow/write authority independently of DeclaredPolicy;
  test const/plain/mut first initialization, missing/expired authority, failed
  Pre without consumption, same-Place aliases and saved-ref rejection after
  initialization. Replacement alone observes old-resident compatibility.
  Derive TypeRole(Q) from purity, independently of SelfConstructible, and check
  both registered closure homes at complete tau consistency; test equal Core
  with distinct homes, including a Pattern
  closure that has no V_tau registration.
- Connect type +=/-= to V_tau updates with Writable, OpenHere and final closure
  membership; connect ordinary group updates to their distinct bucket algebra.
- Connect witnessed anchored replication, preserving captures, internal
  alpha-renaming and the original closure identity. Do not feed the destination
  anchor backward into parsing or RHS evaluation. Typed name creation does not contribute a closure. First named contribution
  forms its full type through OneShotFormation and initializes the Place once;
  subsequent contributions use extend/inject. Check /tau(T) home independently
  of named Val2 residency and either role registration; V_tau membership needs
  no val::path resident. Group buckets use full bound
  type observations, not Core or TypeValueId. Add positive/negative cases for
  uninitialized reads, borrowing before initialization, failed write Pre, Close
  rejection, equal-Core/different-callspace buckets and distinct type homes.
- Extend ordinary meta invocation before connecting its A instance: preserve
  input value observations and semantic name/subject/borrow dependencies;
  construct the direct instance name/type tau_M and ordinary Val2 payload Places;
  propagate output opening-source meets. Implement P1 meta qualification before
  mut-view acquisition and plain completion/closure. Ordinary payload policy,
  borrowing and lifetime checks remain independent.
  The current `semantic_world::meta_type_roots` cache stores only a type lookup
  id and struct construction material. It does not yet retain general instance
  state with ordinary Val2 payload Places and P1 meta/plain completion rules. `canonical_arguments_product_address` records value observations;
  it does not supply the general identity-sensitive dependency boundary.
- Implement generic meta result-name/cache residency with construction status,
  current reads, ordinary writes, effects and dependency revalidation. Repeated
  acquisition must not rerun initialization, freeze the first resident, revive a
  consumed resident or replay stale write authority. The current source meta body
  path remains unsupported, and base OpenHere has no output dependency meet.
  Carrier tests of content-sensitive argument keys remain valid for value
  observations; they do not establish name-dependent invocation semantics.
- Derive A from those general facilities with its input construction subject and
  ordinary Val2 group member. Equal full keys retain the same subject through Close
  and input-carrier replacement; saved references keep their original result
  Place. Recheck inherited opening and ordinary write authority in every write
  Pre. No A-only global indexed-place primitive is needed.
- Connect closed-type singleton-Val2 compile extraction through ordinary navigation:
  exactly one entry yields its value; zero/multiple entries fail through the
  chosen compile-error semantics. Count actual Val2, not callspace or visibility
  projections; preserve access checks. No implicit projection or borrow is added.
  Cover direct-meta type/root rejection, independent Val2/V_tau/Pattern roles,
  meta retention, plain closure, mut-after-OpenHere, and no-reopen on cache reuse.
- Connect Pin=ElabIn(P2,Delta_in) and Pout=ElabOut(P1,Delta_out), with independent
  P1/P2, explicit Pin stage atoms/holes, and Pout.stage=P1.stage. Existing formal mode inheritance is compatible with bare omission; do
  not replace it with unconditional Plain. Audit binding_result_policy_demand,
  policy_let_target_demand and ordinary-invocation defaults for the distinction
  between omitted constraint, explicit concrete atom and explicit HoleRef.
  Use registered operator Pattern extraction plus require to solve the joint
  relation; the 3×3 table is a derived view. Test formal-local holes, shared and
  independent holes, inherited mode, explicit plain override, contextual/default
  completion, output demand before maxima, sealed inner calls and no reopen.
- Connect NameCoord before Retained/Place realization. The resident-generation
  ProjectionSlotIdentity carrier is not automatically the stable coordinate.
  Test identical sibling contribution coordinates, unordered contribution join,
  explicit declaration conflicts and distinct root/selector identities.
  Ordinary Val2 writes may change Core without acquiring either registration.
- Connect the equal meta declaration surfaces and operator call/extract/generative
  projections. Current NormExpr::OperatorTarget preserves unresolved grammar
  material, not these semantic consumers. Add syntax goldens without parser
  semantic lookup, proof-relevant extraction with zero/one/multiple solutions,
  concrete/wildcard specificity and occurrence-level registration rejection.
  Trait-like E laws and optimizer rewrite proofs are ordinary meta results with
  distinct consumers; parsing cannot depend on source evaluation.
- Connect generated Val2 realization after registered structure closure without
  reopening a construction view. Test a later requested member, stable Pattern/
  V_tau registrations, rejection of generative registration evidence, and an
  anchored V_tau callable with no named Val2 resident. Distinguish mechanical
  struct helper production from generative name occurrences. Check current Norm
  and only_val2 counts after effects while preserving prior copied snapshots;
  cached facts must remain snapshot/continuation-relative. A's state references
  still fail their own opening-source check after Close.
- Align implicit return selection to the outermost enclosing function layer.
  The current return_target binder selects its most recent frame; current
  one-frame tests do not prove nested-frame correctness.
- Connect name-preserving @ and ordinary value/borrowed lifecycle fields,
  independent SafetyPolicy, and compatible post-commit external admissions.
- Connect link's compilation-wide E-owned LinkRegistry using canonical
  source-provider/root identity. Diagnose Active re-entry as cycle and Done
  acquisition as duplicate; detect sibling duplicate claims without cache reuse
  or order-based coalescing. This effect law is closed, its carrier is pending.
- Connect host/target-machine Objects and their ordinary policy/views; metadata
  availability must not force compile-time payload realization.
- Implement E saturation and synchronized continuation projections, residual
  transport, and revalidation after equivalent O1/O2 rewrites. Planner search
  never changes E or supplies private facts.

These are known consumer obligations. They do not reopen the resolved semantic
rules or permit graph, file, cache or registry authority. Relevant implementation
changes need source goldens plus identity, no-reopen, non-derivability and
boundary tests; this documentation migration does not claim those consumers
have been implemented.

## Canonical semantic revision implementation gates

The [178-case conformance matrix](canonical-semantic-conformance.md) records
semantic obligations, not passing source tests. This revision changes owners
and handoffs only. Existing Rust carrier tests still describe connected slices;
they cannot authorize an alternate implementation of these rules.

| Gate | Existing evidence | Required consumer and acceptance coverage |
|---|---|---|
| Single-stage positions | lang_build/src/policy_pair.rs has PolicyStage but also StageSet; policy_pair_semantics tests admit unions | Replace resolved set semantics; preserve unresolved solver alternatives, position omission/atom/hole, heterogeneous Pin and Pout authority. S01–S08, S12 |
| Two rounds and origin | ordinary_invocation.rs routes the ordinary trunk; phase_flow.rs has a single derived_compile_companion helper | Connect R_vis evidence, lazy C_sigma family, ordinary A/D/Policy/Pattern selection and shared selected origin. S09–S11 |
| Runtime entry and Seal | phase_flow.rs has DeferredToSealStatic; CLI currently exposes frontend commands | Connect EntryContinuation, runtime main, active dominance, real readiness dependencies, pending seal formation and scheduler invariance. E01–E09 |
| Instance lifecycle and cleanup | lifetime substrate records events; Raw WithClauseAst and NormWithClause preserve shape | Connect Killable/MoveEffect/Movable, uniform type/meta instances, directed Touch closure, default NLL, lexical empty-with and no duplicate drop. L01–L08, W01–W08 |
| Chain and residual boundary | phase_flow.rs carries Done/ControlFlow; InvocationResult residual is an opaque class/provenance carrier | Connect restricted Split/D proofs, internal chain/target completion, result Pattern delivery and separate residual escape. P01–P08 |
| Ordinary meta query state | current meta root cache lacks the full retained instance/member state protocol | Connect default formation, actual member mutation, current committed reads, snapshot/Close and SealDom checks. P09–P12, E06 |
| Operator dispatch | Norm OperatorTarget retains spelling/fixity/arity; world.rs directly resolves operator spelling | Connect OperatorUse/OperatorNameValue, operator[op]/op::adl, OG_s extraction, explicit Forget_s and current slot. Add source goldens for supported operator-name forms. O01–O07 |
| Expression formation and contribution | source declaration carriers and sorted discovery do not supply positional expression evaluation/common-snapshot join | Connect every legal completed ClosureExpr -> tau_C through struct, file structural installation, true lexical binding, explicit contribution roles and anchored formation. N01–N08 |

Paths in the table are relative to crates/ except frontend carriers in
crates/lang_syntax and the CLI in crates/lang_cli. Carrier names are locating
evidence, not semantic definitions. Tests of the new consumers must cover
positive/negative cases, identity versus equality, no reopen, non-derivability,
authority uniqueness and observable effects where relevant.

A small finite-sum fixture can validate Split without implementing arbitrary
Pattern difference. Likewise a minimal projection-family fixture can validate
origin retention without a complete runtime backend. Do not make old carrier
tests pass by relaxing the canonical acceptance scenarios. Full residual IR,
ABI/layout, effect/error/sync interfaces and arbitrary Pattern algebra remain
outside this revision. Structured Path semantics are defined; their source and
evaluator consumers remain pending. Broader quotation and the exact ordinal API
remain open.


## PR106 consumer gates

The semantic revision is docs-only. The 64 earlier cases are retained (N01/N03
clarified), with 114 new cases and D01–D23 owner links. No new source behavior
is claimed by a successful existing Rust test suite.

| Gate | Current locating evidence | Required consumer / acceptance |
|---|---|---|
| Product layers and named open observations | `lang_syntax/src/norm.rs` preserves Product/Pattern carriers | All-named layer unorderedness; explicit named extraction/ordered assembly; finite named open Product. 106-PD/NM |
| Path, quote and splice | `lang_syntax/src/ast.rs` NavPath is restricted; `token.rs` has Dollar but no Hash consumer | Ordinary extractable linked structure, endpoint direction, late versus anchored roots, # and ready $ preserving holes. 106-PT/SP |
| Generative/intermediate extraction | Current declaration/Pattern carriers do not implement the new relation end to end | Optional callable head, Concrete/Wildcard/HoleRef selector, expression body, compatible intermediate R valuations and layer cardinality. 106-GN |
| Public Policy observations | `lang_syntax/src/parser/policy.rs` retains colon/choice grammar; `lang_build/src/policy_pair.rs` retains internal pairs | Retire public pair syntax while preserving full internal observations, same-edge type projection, concrete/hole/splice/omission. 106-SP/RP |
| Direct result demand | `norm.rs` TailValue and `lang_build/src/control_flow_end.rs` preserve terminal shape | Immediate selected ReturnPattern/Pout before inner maxima; both outer positions constrain inner positions; no implicit temp/no reopen. 106-RP |
| Ordinary ADL | `lang_syntax/src/norm.rs` still creates DotClosureLowering and an in-place helper | Emit ordinary field::adl entrance; generate the ordinary forwarder through Path/name relations. Existing goldens describe migration debt. 106-AD |
| General dependency realization | NormCapture/BindingSlot preserve explicit formation; source callable carriers retain NormClosure | Needs -> semantic realization -> layout; common name environment, ordered effects, once-per-formation, no recapture or hidden semantic storage. 106-DP |
| Meta declaration boundary | Generic closure carriers preserve captures/placement; full generative declaration consumers remain pending | Require ordinary => and absent capture clause at the MetaDecl layer. Mask unpassed locals; admit only input dependency closure and established stable definition/instance relations. No CapturedEnv key axis. 106-MD |
| Universal closure and file installation | `lang_build/src/model.rs` SourceCallableObject and `semantic_world.rs` OrdinaryCallEntry retain closure carriers | struct Material_C -> tau_C/c_C/A_C/() with finite leaf and same-formation callable; retain homes/roles and automatic dependency formation with ordinary operation checks. File package-root installation distinct from lexical binding. 106-CL/NS |
| Lifetime integration | Existing lifecycle substrate supplies continuation/Pre/Post primitives | Preserve actual dependencies and action obligations. Further region/escape/state refinement is handed off, not a PR106 blocker. 106-LF |

Paths in this table are relative to `crates/`. Full ordinary meta body
execution also remains unsupported/deferred in `lang_build/src/meta_body.rs`;
new closures and general expression bodies must not silently use a substitute
evaluator. Consumer tests must cover positive/negative cases, identity/equality,
no-reopen, non-derivability, authority uniqueness and observable effects.


### PR106 review alignment

Keep ordinary and type-callee entrances distinct: ordinary x uses its exact
classifier's associated Val2[()], while type tau projects every c in V_tau
and unions their associated implementation entries before one selection.
Preserve the selected callable/implementation pair and x/c as actual self in
the sealed frame. TypeRole follows purity; complete type identity requires
WellFormedTau, not self-construction. AssociatedNamespace(T) is
MemberScope(Core(T)), distinct from the /tau(T) classifier home.

Replace deferred in-place embedding lookup and placement-based binding,
transfer or outer-write prohibitions with automatic dependency formation.
Formation produces ordinary requirements/realizations; invocation consumes
those without recapture. Concrete operation checks retain actual dependency,
access, capability, region/generation and escape evidence. Existing source
placement carriers do not implement this semantic handoff.
Placement also supplies no applicability, specificity or preference dimension.
Keep Raw/Norm placement for syntax only; do not propagate it into candidate
ordering. CallableOwnerPlacement and retained NormClosure are source carriers,
not preference evidence. Ordinary closures may combine explicit and automatic
dependency occurrences; automatic formation is not exclusive to in-place.
Realize each requirement using its source occurrence's selected ordinary action,
uniquely up to observational equivalence. Test ordinary preference/ambiguity and
no-reopen; lowering cannot choose snapshot versus live-reference behavior.
MetaDecl is excluded from both closure capture channels: reject capture clauses
and no-=> declaration bodies at declaration formation, preserve weak-token and
non-semantic normalization boundaries, and do not reinterpret an already
captured ordinary closure as meta. Keep the invocation key limited to its
existing parent/callee/canonical-input coordinates. Nested ordinary closures
may use only material legally available inside the invocation.

Path consumers must validate the inductive PathShaped domain, including finite
chains, unique terminal explicit roots and endpoint compatibility. Wire the
ordinary name-family string projection before using it in ADL generation.
