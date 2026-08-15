# Open Questions

This document tracks unresolved, forward-looking design questions for `lang`.

Current normalized surface behavior is defined by
`spec/public/v0.5/normalized-surface-semantics-v0.5.md`. This file does not
explain current behavior.

Resolved records live in history:

- v0.1 questions: `spec/history/v0.1/resolved-questions.md`.
- v0.3 Normalized AST questions (`N-AST-1..9`), their resolutions, the N-AST-9
  review audit trail, and the documentation-reset debt log:
  `spec/history/v0.3/normalized-ast-design-history-v0.3.md`.
- v0.6 semantic correction record and the resolved symbol-first construction
  direction index:
  `spec/history/v0.6/resolved-questions.md`.

The v0.6–v0.8 build / namespace graph / early meta direction is
`spec/design/symbol-world/early-meta-functions-and-namespace-graph.md`. It is not an open
design question; it is the next post-v0.5 roadmap track.

---

## v0.6 semantic correction record

The resolved v0.6 semantic correction facts and the resolved symbol-first
construction direction index moved to
`spec/history/v0.6/resolved-questions.md`.

Still open after this correction:

- Exact representation of the first-order `TypeValueId` root (a registry
  projection, not canonical type-value equality). Canonical type-value
  equality is the recursive object normal form above, consumed as the
  observation `Addr(Norm_type)`; what remains open is the root representation
  itself and the normal form of value payloads that currently
  keep an identity-stable opaque form.
- Exact representation of symbol/place identity and
  `StableTargetIdentity(q)`. The representation is open; the semantic
  requirement that distinct borrow targets normalize distinctly is closed.
- Exact future lowering of generic/meta-generated type expressions such as
  `(int Vec::std)`.
- Final syntax/API shape for resolver expected-role disambiguation; the current
  `lang_build` API is provisional.
- Exact future implementation of independent place-writability and
  construction-lineage Open checking.
- Implementation of source-level `let f::(U@)` against an already
  installed type carrier/place: the associated-extension entry point currently
  requires a still-open construction and resolves the target object from the
  constructed Pattern. Bare `let f::U` is not the target place form.
- Exact future implementation of borrow-view evaluation (`ref` / `share` / `@` /
  `rebind`), of the capability coercion `Coerce_{j->k}` behind borrow-operator
  overlap, and of the escape check
  `Escapes(view, destination) = Region(destination) ⊄ ValidRegion(view)`.
- Exact Rust/IR representation of the transitional `SymbolConstruction` carrier's
  facet exposure;
  semantically, formal invocation remains uninstalled and outer binding resolves
  the installation place.
- Exact Rust/IR scheduling of `ConstructionLineage` freeze events relative to
  graph seal, without reintroducing a place-level Open capability.
- Whether and how external objects can intentionally expose extension points.
- Whether escaped field names are still needed for namespace-role conflicts
  outside the object/subspace case handled here.
- Exact form of future `unique trait`.
- Full access-tree construction algorithm.
- Full lifetime relation over region/origin facts.
- Interaction between type-value equality and type-associated namespace
  traversal.
- Final surface mechanism, if any, for requesting coordinated value/ref/share
  receiver candidates in one associated `()` Symbol; the current rule requires
  separate authorized contributions.
- End-to-end syntax/integration for an externally navigated call-entry
  extension such as `let ()::((T ref).type) = ...`; the semantic
  destination and ordinary type-check behavior are fixed, but the current
  frontend does not claim this complete declaration path.


---

## v0.7-prep policy correction record

Minimal policy-aware lookup remains implemented through transitional
`PolicyFlag`, `PolicySet`, and `PolicyEnv` metadata. The future semantic model is
now the typed pair:

```text
Π = Pv:Pp
```

Resolved future-design decisions:

- `P1` is the optional projection on any binding, not a function-object-only
  scalar. Omitted `P1` infers the complete result, a single `P1` is a
  value-dominant projection, and a pair `P1` filters value and Pattern
  components independently.
- A general `runtime let` is legal. A source-side non-runtime premise belongs
  only to the particular compile-flow projection rule that declares it.
- `P2` describes a call/expression result pair. Single-policy `P2` uses
  `P:(P-runtime)` and supplies `compile` when only `runtime` remains.
- Function-object stage views are derived from `P2`:
  `Stage(P1p) = Stage(P2p)` and
  `Stage(P1v) = Stage(P2v) union Stage(P2p)`. Mutability, namespace
  visibility, and value presence are not copied by this stage derivation.
- The only execution phases are OpenStatic, SealStatic, and Runtime. `meta` is
  exposed only in OpenStatic, `seal` only in SealStatic, `compile` in both
  static phases, and runtime values only in Runtime.
- Privileged seal scans read the frozen pre-seal world `Wpre`, never symbols
  generated during seal.
- Namespace state distinguishes the complete internal `Σ_full`, the external
  projection `Σ_export`, and build-world membership `Wpre ∪ Wseal`.
  `ExportOverloadSet(name)` is an identity-preserving projection of
  `FullOverloadSet(name)` rather than a separate symbol universe.
- Results retain component policy pairs and returned `Val2` objects retain
  their own policy; no whole-result scalar policy is inferred.
- Compile flow is a mechanical projection of complete symbol flow. Calls stay
  ordinary unresolved call nodes, D/Done structure is intrinsic, and recursive
  evaluation is not replaced by summary/fixed-point machinery.
- Eligible runtime function objects have complete derived `Val2` compile
  companion objects. Overload resolution forms a fully admissible set before
  preference, and must-select is an object strategy rather than a fallback.
- `const`/`mut` is a `Pv` dimension. Multi-position preference uses product
  partial order; delete members remain candidates and may be the unique maximal
  rejection.
- Written formal parameters inherit their callable P2. `const let` / `mut let`
  restrict only the inherited mutability Pattern; all other P2 dimensions are
  invariant. The resulting qualifier is exported into the candidate's external
  policy product order as well as its body-entry pair. Opposite actual
  qualifiers remain preference inputs rather than being removed by ordinary
  P1 projection.
- Every callable has invocation-frame slot 0 for its caller object. Ordinary,
  in-place, meta, and generated closures use the same positional rule: the
  first written formal explicitly declares that self-position under any legal
  spelling, while its actual is supplied implicitly. Only later written
  formals consume the call-site Product. For a standalone function the caller
  is its function object; for an associated `()` entry it is the object whose
  type supplied the entry. `CallableOwner` and receiver type are independent.
  A head with no written formal retains an unbound semantic self-position.
  Generated receiver helpers therefore use `[self, val, ...]`, not
  `[val, ...]`.
- A function-object binding has the unrestricted empty mutability domain by
  default (`const || mut`); only its declaration may crop that internal axis.
  Export derives a separate external view: value-bearing exports expose
  `Project_const(Pv):Pp`, while pure `absent:Pp` exports have no mutability
  requirement. A `mut`-only value export is invalid; `const || mut` remains a
  valid complete internal view. This projection consumes the resolved internal
  `PolicyPair` after declaration-side P1 application. Direct `export + mut`
  roots are rejected; mut-only overload members of an exported symbol remain
  in `Σ_full` and are omitted from `Σ_export`. `Pv = absent` is structurally
  empty on the value side: both value stages and value mutability are empty.
  P1 elaboration, P2 normalization, and resolved export projection reject flat
  compatibility carriers that attach either subdimension to absent Pv.
- `...Q` is available in every let-shaped binding slot, not only parameters.
  It remains one Pattern remainder constructor, never a pack type or RHS
  unpack. Raw `...(a, b)` is preserved but rejected after P normalization
  because the bare Product has no stable top mode. At an ordered level,
  explicitly headed structure such as `...((a, b) pair)` may be admissible;
  unordered levels accept only a whole-remainder binder/discard. Every Pack
  contributes one outward specificity node, and internal structure never
  becomes multiple same-level EP nodes.
- DeduceLists elaborate as left-to-right telescopes with exact
  alpha-normalized `HoleBinderId`-targeted Pattern/policy references.
  Declarations see inherited and preceding holes, not themselves or later
  declarations. Names are unique within one `PatternRoot`; an independent let
  Pattern or nested callable head creates a new root and may shadow inherited
  names. A BindingSlot policy precedes its local DeduceList. Generated receiver
  holes use hygienic generated keys and do not collide with source spelling.
  Callable head holes scope captures, parameters, policy, return, clauses,
  body, and inherited nested callables. Spans are provenance rather than
  semantic identity. Frontend identities carry normalization owner, callable
  owner, Pattern root, and root-local binder; build integration maps the
  callable owner to persistent `SemanticOwnerId`.
  Value-side names/navigation remain unresolved for a later resolved-symbol
  pass. Parser/Norm recursive preservation and the Pattern/policy identity
  substrate are implemented, while general Pattern-directed execution remains
  a later consumer.
- Extraction-style result delivery uses the declared return Pattern. Explicit
  writes address its binders separately; a bare tail or targeted return matches
  one result object as `let ResultPattern = expr`.
- In-place closures may contribute callable overload candidates. They have no
  capture list, defer unresolved outer reads to the embedding layer, gain no
  capture set, and may not directly write an outer place. Ordinary closures
  have explicit source captures plus future resolved automatic-const
  requirements; `[x]` is explicit `[let x = x]`, not automatic capture.
  Capture requirements remain abstract dependencies rather than `self` fields
  or layout declarations. In-place candidates are preferred over otherwise tied
  non-in-place candidates after first-order-over-instantiated preference.
- Internal explicit navigation searches `Σ_full`; external explicit navigation
  searches the const-projected `Σ_export`; neither is a Wpre/Wseal membership
  query. Ordinary external call dependencies normally fall within automatic
  const capture. Automatic capture and call resolution share a problem domain
  around symbol identity and const visibility, without implying pass ordering,
  shared intermediate objects, or an implementation dependency. Explicit and
  automatic capture remain distinct dependency declarations even when they
  resolve to the same source; later layout alone may coalesce equivalent
  storage while retaining binder, policy, and provenance.
- Inferred require retains coarse complete blocks and guarded branch groups,
  conjoins with manual require, and shares one compile-evaluation graph with
  body continuation.

Implemented substrate after this correction:

- Raw and Normalized AST preserve dedicated conjunction, choice, atom, pair,
  and absent-value policy nodes. Pattern `|` and policy `||` are distinct.
- `lang_build` provides typed pair normalization and true slice restriction,
  three contextual P1 elaborators, three-phase exposure, structural compile
  flow projection, Wpre/export-retention closure, candidate-level
  `ResolvedCandidatePolicy { pair: PolicyPair, provenance }` to
  `ExportCandidateView { identity, internal_candidate, external_policy:
  PolicyPair }` transformation, and phase/const-mut product-order test
  substrate. The direct declaration `external_projection` remains a
  root-local `P1Projection` preview. External admission requires both
  symbol-level export-retention-closure membership and public path
  reachability; among an admitted symbol's resolved candidates, mut-only
  entries remain in `Σ_full` and are omitted from `Σ_export`. Namespace-graph
  installation supplies the persistent admission facts. Retention membership
  is not itself export status; `Σ_export` is the external candidate set.
- `lang_build` now also provides a parent-linked `SemanticOwnerGraph`,
  owner-derived standalone anonymous callable types, independent receiver
  bindings, canonical meta-instance interning, owner-qualified Pattern/hole
  carriers, and an owner-aware namespace forest with explicit
  `PackageBoundary`, identity-preserving `Mount`, package-derived Full/External
  view routing, `DefaultExtractionView`, and typed lookup failures.
- `lang_build` now has an implementation-only atomic Runtime-migration helper
  that first preserves canonical ordinary P1 projection: any non-empty
  `project_p1` result completes binding elaboration and makes migration
  unreachable. After a complete query projects nothing, an accepted runtime
  alternative is extracted as the runtime-only migration target and paired
  with an eligible static input view; pure types use a projection-only
  `Infallible` carrier. The compiler mandates the static-to-runtime stage edge,
  while callable-declared mutability endpoints may differ. Migration cannot
  repair Type/Pattern structural failure. Literal helpers separate literal
  family, atomic builtin type `T`, and concrete numeric `Tnum`; registries store
  current first-order TypeValue projections derived from resolved Type symbols,
  not final canonical type-value identities. The caller-supplied candidate
  prototype preserves input × output endpoint Pareto preference before its
  Pattern-specificity stand-in, with direct-only selection. Its endpoint-only
  maxima helper is private and is not a sequentially composable full-Bp
  implementation. Typed qualification distinguishes available,
  delete-rejected, missing, and ambiguous outcomes.
- Flat policy flags remain compatibility transport, while lookup and execution
  environments use the same three canonical phases.

Not implemented after this correction:

- Retiring the legacy `MetaInstanceCache` compatibility digest
  (`MetaInvocationInput::compute_key()`, a seed fingerprint over the bound
  argument `TypeValueId`s). The canonical source-meta instance key is no
  longer transitional: world-connected invocations compute
  `MetaInstanceKey = MetaCallableIdentity × Addr(Product(args))` over the
  normalized canonical argument product. The legacy digest only keys the
  compatibility cache and does not participate in canonical semantic
  identity.
- Storing and checking canonical `Pv:Pp` on every symbol/value object.
- Storing policy-pair views on every namespace entry and routing every build
  operation through the typed P1 projection.
- Connecting the candidate-level export-view projector to the persistent
  namespace graph and authority-sensitive external resolver.
- Integrating structural compile-flow projection with the complete evaluator.
- Routing ordinary source initializers/bindings from canonical `project_p1`
  failure into transition preparation without changing the existing query
  semantics.
- Applying the existing-view-first rule at every future Policy-demanding
  consumer. A consumer accepting `compile || runtime` is already satisfied by
  an available compile slice. If the complete accepted choice has no existing
  view, its runtime branch is the currently authorized constructible branch.
- Completing every old ordinary Bp coordinate and every later B1..B6 filter in
  the connected `PreparedCallCandidate` carrier. The implemented slice already
  composes its ordinary formal/phase coordinates and optional migration
  endpoints in one product before maxima; the retained endpoint-only fixture
  remains non-composable algebra evidence only.
- Implementing the already-specified family-specific constructor/extractor
  role checks, materialization place/owner allocation, and structural slot-0
  Pattern applicability. Ordinary Symbol/Val2/associated-`()`/InvocationFrame
  routing is connected; the caller-supplied migration fixture remains isolated
  and no universal transition Symbol or new callable ontology is implied.
- Full, separately selected mechanical `ref` storage construction,
  `share`/`rebind` composition, `[[global]]` seal scanning, and any future
  non-Runtime Policy-migration legality.
- Runtime owner/place allocation, static-materialization cache identity, and
  the generated-storage boundary that keeps `[[global]]` out of the
  source-visible namespace graph.
- Materialized derived companion objects and must-select enforcement.
- Automatic inferred require, a complete overload resolver, and a call
  execution checker.
- Closure materialization, lazy embedding-layer lookup for in-place closures,
  and the in-place-over-non-in-place overload filter.
- Result Pattern delivery/D-reduction; the current return-target substrate only
  retains the complete return binding slot and selects a restricted active
  frame.
- Any positive lifetime/Horae design beyond the `@` overload groups and the
  escape check.
- Borrow-view evaluation (`ref` / `share` / `@` / `rebind`) under policy
  projection, type checking, and runtime IR.

Deferred materialization and mixed-stage work must preserve these
already-recorded design constraints:

```text
existing Policy view => slice; migration is unreachable
complete choice empty + runtime accepted => construct runtime branch
compiler mandates static -> runtime; callable owns legal mutability endpoints
compile -> runtime = new runtime object, not lifetime extension
addressable runtime value => ordinary owner/place
compile-ref cache identity = StableTargetIdentity(Target(ref)), not pointee equality
generated [[global]] storage != source-visible NamespaceGraph mutation
materialization place != Pattern owner
ref/share/rebind are explicit mechanical operations, not Policy-demand repair
Resolve once; Evaluate progressively; Residualize runtime dependencies
```

### Fixed mixed-stage semantics and open implementation questions

The core binding/evaluation meaning of a mixed-stage Policy domain is fixed
without claiming a complete evaluator:

```text
runtime in Pv means an existing runtime Policy slice
ExposePolicySlice(runtime) does not invoke migration
slice existence does not imply ReadValue availability in the current phase
compile-readable dependencies bind/evaluate in the static phase
runtime-dependent slots/computation residualize
symbol/namespace/callable identity is resolved in the static world
phase-admissible computation is evaluated as early as dependencies and effects permit
Runtime supplies missing values and continues the same already-resolved computation
```

Runtime continuation does not reopen an already-resolved Symbol, namespace
path, callable identity, or overload set merely because a value becomes
available later. Any future explicit dynamic dispatch would be a separate
language mechanism.

The following remain open and must not be inferred from the current migration
prototype:

- the final residual representation of static/runtime arguments in an
  `InvocationFrame`;
- the partial-evaluation IR across OpenStatic, SealStatic, and Runtime;
- the exact sequencing frontier for effectful expressions under the
  evaluate-as-much-as-possible principle;
- the ABI and physical representation of runtime residual continuations;
- the final composition of mixed-stage evaluation with future capability and
  effect systems.

Current source does not expose declaration-side fallback/must-select syntax or
call-site candidate-family annotations. Their pipeline positions are fixed:

```text
ResolveSymbol
  -> CallSiteCandidateFilter
  -> GenerateCandidates
  -> A = FullyAdmissible(...)
  -> D = DeclarationCandidatePolicy(A)
  -> Bp'
  -> ordinary partial orders
  -> Unique
```

Any admissible non-fallback candidate, including `delete`, then suppresses
fallback permanently within `D`. Declaration policy cannot revive an
inadmissible candidate. Call-site annotations filter candidate families before
generation; their final syntax and general selector algebra remain open.

Build-world integration gates (not blockers for the current frontend/build
substrate):

- Owner/root qualification is implemented. Persistent/incremental restoration
  of `SemanticOwnerGraphId`, stable syntax-node local keys, and serialized
  meta-instance construction keys remains unfrozen; byte offsets must not be
  substituted for those keys.

- `SemanticOwnerQualification` currently verifies exact mapping presence and
  rejects a conflicting remap. It does not yet prove that the whole frontend
  owner tree embeds homomorphically into the persistent tree:

  ```text
  Map(Parent_frontend(x)) = Parent_persistent(Map(x))
  ```

  Establishing that proof is a P1 gate before multi-root persistent owner
  harvesting; it is not a reason to reopen the owner/Pattern-root semantics.

- The new owner-aware namespace graph already preserves typed failures
  (`Unresolved`, non-retention, private path, no eligible candidate, missing
  mount target, and missing package boundary). The legacy
  `NamespaceOverloadSets.exported` compatibility map still omits a symbol when
  its projected candidate list is empty. Its eventual migration should retain
  symbol-level admission facts even for an empty candidate subset, for example:

  ```text
  ExternalSymbolView {
    admission,
    candidates
  }
  ```

  It must map onto the typed owner-namespace failure carrier rather than
  collapsing back to `None`.

- The restricted v0.8 overload selector still reports
  `UnsupportedExternalVisibility`. The implemented scope is the export-view
  carrier and persistent namespace resolver substrate, not complete migration
  of legacy namespace consumers or end-to-end external overload routing. Those
  two connections remain a P1 integration gate, not a core semantic blocker.

- Custom `?` construction of a richer extraction interface remains open.
  Private structural members are already excluded from the default extraction
  view and form a hard non-disclosure boundary; this does not define the
  eventual custom-question protocol.

- The build API carries package-boundary and mount metadata, but no manifest
  file format, registry/version solver, dynamic loading, or binary namespace
  serialization is frozen.

Still open for later design:

- the final source token for the absent-value policy pattern (`S`, `null`, or
  another spelling);
- the complete runtime reflection object model;
- the semantics of any additional future policy stage;
- which named overload strategies beyond compiler-known
  `must_select_if_qualified` are provided, and each rule's monotone comparison
  semantics (the source spelling `=> name {}` / no-`=>` `[[name]] {}` is now
  fixed);
- the remaining semantic surface of member bindings inside meta bodies.
  The construction-effect *family* split is fixed — fresh member,
  existing-target write, and the delivery terminal are three distinct
  events that never collapse. There is no fourth alias-member event: the
  alias/forwarding member direction is retired, and a member that must observe
  an external object holds a borrow view (`ref` / `share`), which is an ordinary
  value. The spellings that currently reach these events are the transitional
  `let`-only encoding while the grammar lacks expression-level `=`:
  `let r = expr;` adds a fresh member, `r = expr;` writes to an existing target
  (today a placeholder overwrite scaffold — the final cluster write algebra is
  not fixed), and bare `r;` is the delivery terminal, not a member event.
  The settled orthogonal target remains `let` creates, `=` writes, return
  events deliver; the return-slot spelling reading and its no-shadow
  restriction are compatibility encoding, not final surface rules.
  Still open is the cluster write algebra itself (which facets an existing member
  exposes for later structural writes, and how a write interacts with overwrite);
- how source code references a derived compile companion and associates an
  explicit replacement;
- whether default companion suppression is permitted and which equivalent
  compile Pattern/contract interface would be mandatory;
- finer-grained require atomization and canonical identities for grouped
  require structures;
- future Pattern policy after an explicit sealing mechanism;
- complete lifetime region/origin/Horae algebra;
- the future member set of `BuiltinPrivilegedAstMetaFunction` and each member's
  bounded capability.

### Open: D-reduction / control-flow pattern-transform global function family

**Status:** Open (reopened at the v0.9 spine integration).

The early `let +` global overload family (the retired
`v08_meta_overload*` fixtures and their `int + unit` / `unit + int`
invocations) was a first attempt to model D-reduction and control-flow
pattern transformation (`if`/`else`/`unit` absorption) as an ordinary
global function family. That family is still wanted, but every member was
written against retired semantic invariants:

- forwarding-terminal bodies (the legacy `r === t;` spelling returning an
  external type value), which the self-root invariant rejects for meta return
  members and which has no successor spelling — the forwarding direction itself
  is retired;
- no meaningful-Pv-dimension requirement on cross-policy value transitions
  (`N2(runtime) = runtime:compile` now makes a runtime-only result P2 with a
  pure-P return slot a declaration-time hard error);
- the legacy restricted overload selector rather than the connected
  ordinary-invocation spine.

Rather than rewrite the family in place, the fixtures and their dedicated
tests were deleted. Redesigning the family under the current invariants —
self-rooted construction bodies, meaningful value dimensions, connected
spine selection — is an open design task, not a mechanical port.

Tracked with the same retirement:

- meta-strict local-initializer semantics inside selected meta bodies
  (previously pinned by `v08_initializer_meta_strict_fail`);
- the generic sum pattern value (`r === t | u`), previously pinned as an
  explicit unsupported boundary;
- folding the legacy restricted initializer evaluator (the S8 transitional
  adapter in `initializer_eval.rs` and the restricted overload selector
  behind it) into the connected invocation spine, after which the
  residual/meta-partial routing should be re-expressed in spine terms.

### Semantic spine: explicit incomplete records

These implementation shortcuts are settled as *known incomplete*, recorded
here so they are not mistaken for design decisions:

- Return-event binders are currently matched by spelled name against the
  declared return slot. The final rule must match by binder identity
  (`PatternRoot` + root-local binder), not by string spelling. This is safe
  only under the current no-shadow restriction; see the next-stage entry
  conditions below for the mandatory coupling.
- The legacy `MetaInstanceCache` compatibility digest
  (`MetaInvocationInput::compute_key()`) still exists next to the canonical
  `MetaInstanceKey = MetaCallableIdentity × Addr(Product(args))`; the digest
  keys only the compatibility cache and does not participate in canonical
  semantic identity (see "Not implemented after this correction" above).
- Legacy `let r === path;` member bindings inside meta bodies are
  unconditionally rejected as `MetaReturnTypeRootMismatch`. That rejection is
  now permanently correct for a different reason: the alias/forwarding member
  direction is retired, so there is no future self-rooted alias member to
  distinguish. The diagnostic code is a legacy label; a member that must observe
  another object holds a borrow view (`ref` / `share`) instead.
- Member write (`r = expr;`) is a CURRENT PLACEHOLDER (internally
  `PlaceholderOverwrite`): while the frozen v0.2 grammar has no
  expression-level `=`, the scaffold exists only to validate
  existing-target addressing inside a meta body. Its behavior — select
  the unique existing member carrying the written facet, reject zero or
  several matches, replace the value under the member's own binding P1 —
  is a deliberately conservative internal choice and is NOT the final
  `Write(ClusterSymbol, RHS)` algebra. What this stage freezes is only
  the boundary pair `let` (creation) ≠ `=` (write to an existing target)
  and return event ≠ binding/write; how a real `=` on a cluster target
  adds or replaces the Q-role cache / val siblings by RHS shape is future
  work. Current executable-slice cache resolution distinguishes only the pure-P
  type-role projection, and the placeholder
  selection and harvest-shape behavior are pinned by unit tests only.
- Cluster member contributions carry per-member views: each member's own
  written binding P1 is elaborated and projected over its initializer's
  complete result (an empty projection is a hard error, never a collapse
  onto the callable's function P2). The executable slice is still
  self-rooted generated type members only; val members
  remain future work, and a positive member-specific P1 over a pure-P
  type member is not yet spellable because the `Absent` value-component
  policy has no frozen source spelling.
- Control-flow end execution is asymmetric: only the `expr;` delivery to
  the directly enclosing layer executes in the restricted meta body
  evaluator. `expr return;` (outermost function layer) and
  `expr (T return);` (layer selected by the function-object type) are
  contract-complete but not yet executable; each fails with its own
  per-form execution-gap diagnostic.
- `AmbientTypeBinder` records only whole-symbol `let` binders today. The
  `ExtractionMembers` and `CallableParameter` shapes are defined for the
  ambient-struct collision guidance but their recording paths are not yet
  wired; a collision against an unrecorded binder falls back to the generic
  "use its existing binding" guidance.
- `OwnerStrategy::ExplicitPrivilegedOwnerRule` today covers only the nested
  meta-body path (the outer meta invocation injects its MetaInstance root).
  User-spelled explicit pattern navigation overriding the ambient owner of a
  direct `struct` generation is not implemented.
- Ordinary construction windows carry provisional closing coordinates
  (`OrdinaryOpenWindow { creation_flow_segment, first_use_seen,
  closed_by_fork_or_end }`) and freeze on the explicit events
  `note_first_semantic_use` / `note_residual_runtime_fork_or_end`, which the
  evaluation driver must raise by hand. The canonical freezing event set for an
  ordinary (non-meta) construction is defined in
  `spec/design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md`
  §12.1.2 and covers `UseForVal1`, use as a meta argument, entry into a global
  normalized structure, actual dependencies/identity merges of non-meta static
  control, values carried across a residual-runtime fork, and leaving the
  construction interval of the owning in-place closure. Mere `LiveAcross` over
  an unrelated compile-only branch/join/loop is not a freezing dependency. The
  current API does not derive `Dependencies(control)` or distinguish every
  static edge from a residual-runtime fork through real control-flow analysis;
  its coarse `note_residual_runtime_fork_or_end` event is implementation debt,
  not a broader language rule.
- The privileged `extend` and `inject` built-ins do not exist yet.
  `let member::((target ref).type) = RHS;` is only associated-member installation
  (never a Pattern-structure write); the end-to-end equivalence
  `let t = ((x inner) t) |> struct;`  ≡
  `let t = (() t) |> struct; let t_ref = (t ref).type;`
  `(t_ref, (x inner)) |> inject;`
  is a *future acceptance test*, blocked on Symbol borrowing, same-name `.type`
  place projection, and `extend`/`inject`. `inject` denotes the ordinary
  three-step read–extend–write
  wrapper, and its left side must be an existing writable `type ref`: a pure
  PatternValue is not writable through its own name. It must not be
  abbreviated to `t = t |> extend(x inner)`, and it must not be
  approximated with `let inner::t = x`, which is a different (non-privileged,
  Val2-only) operation.
- The `let`-only meta construction encoding (`let r = expr` → AddMember,
  `r = expr` → PlaceholderOverwrite, `r;` → Delivery) is a transitional
  encoding while the grammar lacks `=`; the settled orthogonal rule
  remains `let` creates, `=` writes, return events deliver. Neither the
  current special reading of `let r` nor the placeholder's
  unique-existing-member overwrite behavior is final surface semantics.
- A non-return-slot `let x:symbol = ...` inside a meta body has no
  defined meaning yet (symbol-rank local construction). The restricted
  evaluator rejects it with an explicit execution-gap diagnostic rather
  than silently accepting a dead local, so the future pass that defines
  symbol-rank locals is the first to give the form positive semantics.
- The future `?` operator's rule — strip exactly one Pattern layer while
  preserving the semantics of the stripped unordered layer — is a
  registered boundary only; no design or implementation exists.

### Semantic spine: explicit entry conditions for the next stage

These three items are recorded as *entry conditions* of the next semantic
stage, not as generic future work. The next stage must open by addressing
them explicitly:

1. **Return-binder identity before shadowing.** The return-event binder is
   matched by spelled name today, which is a safe placeholder only while
   the no-shadow restriction of the `let`-only encoding holds. Once an
   ordinary `let r` may shadow the return parameter, string matching
   becomes an actual bug, so the two must land as one change:

   ```text
   remove(no-shadow) ⇒ implement(binder identity: PatternRoot + root-local binder)
   ```

2. **Full by-value comparison must also audit what an ordinary type
   binding preserves.** Migrating the remaining first-order consumers is
   not just replacing `== TypeValueId` with `== TypeObservation`. Once a
   carrier-locally extended `T` can legally exist, `let U: type = T` reads
   the RHS *value* — the complete type object `P × Val2` observed at
   binding time — and must initialize `U`'s fresh carrier/place with that
   object; `T`/`U` may then diverge through their own places. Giving `U`
   an empty fresh place with fallback to the Pattern's canonical object
   would silently drop `T`'s carrier-local `Val2`. The path is not
   executable yet (installed-carrier extension is not wired), but the
   comparison question ("what is equal") and the binding question ("what
   is captured") must be solved together.

3. **Installed-carrier member creation / `type ref` targets / writability
   need explicit owners.** Source-level `let f::(U@)` against an already
   installed type carrier/place, place-level `inject`, and
   writability / construction-open checking exist today only as substrate
   (also listed in the general future-work pool above). The next stage must
   assign them explicit scope rather than leaving them pooled. Canonical source
   spells the host place explicitly as `let f::(U@) = expr`. Navigation
   to a missing child yields a prospective ProjectionSlot whose contents are `None`;
   `let` may instantiate it, while bare `=` may not.

---

## v0.5 stabilization debt

The public v0.5 normalized surface semantics are published
(`spec/public/v0.5/normalized-surface-semantics-v0.5.md`). The only residual
Normalized-AST items are implementation-shape cleanup, not open
public-semantics questions:

- Final Rust enum/struct names for the normalized node set and the pattern
  family.
- Final Rust origin / source-map representation.

These are tracked as stabilization/documentation debt; they do not change the
published public behavior.

---

### v0.9: Canonical form specification

#### How should canonical value/type grammar be designed?

**Status:** Open (active at v0.9)

**Current v0.1 foundation:**
Canonical skeletons use the grammar defined in section 6 of
ast-construction-v0.1.md. This grammar is provisional and may be revised
when value/type canonical forms are designed.

---

### v0.10+: Pattern-space and extraction-chain semantics

Future design note:
`spec/design/patterns-overload/static-pattern-spaces-and-extraction-chains.md`.

Resolved semantics are indexed rather than restated here:

| Decision | Canonical owner | Remaining implementation work |
| --- | --- | --- |
| every expression result is consumed; block-final is return, non-final unconsumed is error | `design/patterns-overload/static-pattern-spaces-and-extraction-chains.md` §7 | diagnostics and IR plumbing |
| `Done` isolation and early `self..return(d)` | same document §6–§7 plus `design/symbol-world/function-object-self-and-return-capability.md` | concrete `Done` and lifetime-fact representation |
| `let` creates, `=` writes, return transfers control; alias event retired | `design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md` §4.5 | remove compatibility return accumulator |
| `TypeValueId` is only a first-order root projection; canonical equality is `Addr(Norm_type)` | `design/symbol-world/type-values-places-and-borrow-views.md` | root representation and consumer migration |
| compile returns declared PatternValue; ordinary meta returns Symbol | canonical symbol-first construction document §4 | transitional transport removal |

#### Still open

The following remain open for later implementation phases:

- Concrete representation of pattern spaces as static objects and canonical
  pattern constructors (product patterns, sum patterns, canonical skeletons).
- Concrete representation of `Done` in later semantic IR.
- Exact lifetime fact encoding for the self-return capability postcondition.
- Exact implementation phase that builds the final return accumulator.
- Diagnostics and recovery details for unconsumed results.
- Representation of extraction chains and residual propagation in later IR.
- Closed control-pattern non-additivity enforcement via package ownership /
  explicit lookup routing.

---

### Later: Ownership and NLL

#### How should the NLL CFG be structured?

**Status:** Open (active at later stages)

**Current v0.1 foundation:**
No CFG is built. The raw AST contains sufficient structure (form order,
closure bodies, and explicit `with { ... }` syntax) for future passes to
construct a control-flow graph.

---

### Later: Control-flow and effect semantics

#### How should `return`, `effect`, and `sync` be semanticized?

**Status:** Open (active at later stages)

**Current v0.1 foundation:**
These are ordinary `Name` tokens at the lexical and parser level. No special
AST nodes exist for them. The v0.1 frontend faithfully preserves these names
in expression AST.

Future `match` / `if` staging is no longer open: both use the same
pattern-matching mechanism and select static versus runtime branching from the
scrutinee value component `Pv`, while the Pattern component remains in static
flow. That semantic decision does not change the current lexer/parser boundary.

---

### Operator environments

**Status:** Closed direction; detailed selector algebra deferred

No semantic operator-alias exception survives. `operator` is an ordinary
global type, and the nearest lexical `operator : operator` value maps an
`OperatorIdentity = spelling + fixity + arity` to a Symbol or `None`. Operator
expressions select from that ordinary value; local environments use value copy,
shadowing, and Symbol `+=`/`-=`. The complete environment shape and selector
algebra remain later design work, but they must not revive alias/place
forwarding or spelling-only lookup.

---

### Static materialization cache

#### How should a compile cache represent Open-sensitive applicability?

**Status:** Open (engineering, active at v0.10+)

**Closed part of the question:**
The semantics do not place `ConstructionLineage`, `Open`, or a borrow witness in
normalized PatternValue identity. A `compile` frame transports the input value
and its lineage without creating a root, while every `extend` application
rechecks `Open_Γ(value)` in the current compile-time stack. A `type ref`
parameter does not discharge that check. Thus the same normalized value may be
extension-legal at one call site and illegal at another:

```text
Eval(F, t; Γ_open)  ≠  Eval(F, t; Γ_closed)
```

**Question:** Which implementation representation should carry the mandatory
call-site applicability recheck — an uncached legality judgment or an explicit
`requires Open(value)` summary? Caching the pure computation may not cache or
manufacture construction authority, and admitting the whole lexical context
into the value key is excluded.

---

### Pure Product namespace surface

#### Should intrinsic ordinal navigation be user-visible as a tuple namespace API?

**Status:** Open (surface only; does not block #99)

The semantic kernel is closed: every pure Object has `NamespaceRole`, and bare
Product owns intrinsic `pos_i` selectors in ordinary `Val2`. The remaining
question is only whether bare Product / empty pure Pattern should expose those
selectors through a user-visible tuple-style built-in namespace interface, or
keep them as structural navigation used by products and Sequences. No extra
namespace Object or facet may be introduced to answer this question.

---

### Closed-decision index

Resolved semantics do not remain as pseudo-questions here. Canonical owners are:

| Decision | Canonical owner | Remaining implementation question |
| --- | --- | --- |
| Object closure, `Pure => NamespaceRole`, AsType, ProjectionSlot, borrow targets, NoImplicitBorrowFormation | `design/symbol-world/type-values-places-and-borrow-views.md` | concrete root/slot generation representation |
| Policy projections and `succ_const/succ_mut/succ_plain` | `design/symbol-world/symbol-policy-and-compile-flow-projection.md` | evaluator integration only |
| Pattern relation, binderless Pattern, structural child and extraction roles | `design/patterns-overload/pattern-values-relational-semantics-and-extraction.md` | derivation/residual IR only |
| Symbol `<Q?,V>` Set quotient, complete type snapshots, ordinary-meta Q-root seal, `struct`/`extend`/`inject`, containers | `design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md` | implementation substrate only |
| construction lineage and Open/frozen | same canonical construction document plus `design/lifetime/` | region/stack representation |
| retired semantic alias family | `design/symbol-world/entity-alias-design.md` | frozen Raw-AST preservation only |

The concrete unresolved items remain in the earlier “Still open” lists. This
index intentionally does not duplicate their closed judgments.
