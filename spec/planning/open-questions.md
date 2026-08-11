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

The v0.6–v0.8 build / namespace graph / early meta direction is
`spec/design/symbol-world/early-meta-functions-and-namespace-graph.md`. It is not an open
design question; it is the next post-v0.5 roadmap track.

---

## v0.6 semantic correction record

The following points are resolved for the v0.6 namespace graph / early-meta
track:

- Generated data fields are unary function objects; the first-class `.name`
  constructor generalizes the same first-argument dispatch to ordinary
  remainder arguments without creating a separate member system.
- Once `.name` is lowered, it is an ordinary expression. Its provenance does
  not alter how `P1 |> E P2` associates; replacing `.name` with a bound
  equivalent must preserve the general pipe/product spine. Compact `E.name`
  mechanically uses the same core. Direct `..name(product)` remains the
  distinct direct member-call simulation and is not redundant.
- The post-v0.2 parser changes are versioned by frontend amendment v0.5-A and
  Raw AST contract v0.5; the closed v0.1/v0.2/v0.3 documents remain historical
  snapshots.
- Closure placement and generated provenance are independent in both Raw and
  Normalized AST. Generated dot closures retain `InPlace` placement.
- Product-versus-closure classification and capture-slot bypass recognize only
  the complete `[[Name]] {` tail. Deduce alone leaves capture available; the
  weaker `[[` recovery candidate is confined to independently proven
  post-capture closure heads and never disables ordinary bracket-call suffixes.
- Ordinary capture items are let-shaped bindings. A naked capture expression
  is retained as shorthand only when normalization finds exactly one distinct
  free non-call bare name; all initializers use the pre-capture environment.
- Pack is a direct canonical Pattern Sequence child as well as a Product child.
  The parser preserves Pack shape, while the normalized Pattern validator alone
  owns the one-pack-per-Product-or-Sequence-level invariant.
- Build-world harvesting consumes `PatternValidatedNormProgram` from
  `normalize_and_validate_patterns`. This proves only global normalized
  Pattern invariants; it does not claim recovery-free syntax.
- Fields named `ref` or `share` are ordinary associated Symbols. Generated field
  access uses one same-name overload family whose receiver formals are `T`,
  `T ref`, and `T share`; no `ref` / `share` projection subspaces are generated.
- `let T: type = uint8` is ordinary type-value binding: it creates a new symbol
  `T` whose type value equals `uint8`.
- Type/rank use evaluates by type value, not by symbol name.
- `let T: type = uint8` creates a fresh symbol/place whose type value equals
  `uint8`.
- Creating an associated member of a pure type slot names the carrier place
  explicitly: `let f::(T@) = ...`. For a Symbol `S`, the type-member place is
  `(S ref).type`; bare `=` never creates a missing member.
- `extend(old, Δ)` is the pure PatternValue transformation. It accepts a type
  value, preserves its construction root, and writes no place. `inject(r, Δ)`
  is the separate convenience wrapper `Read(r) -> extend -> Write(r)` over a
  writable `type ref`, and returns that ref.
- No declaration form forwards a Symbol or a place. Shared observation of another
  object is a borrow view (`ref` / `share` / `@`), which is an ordinary value.
  `ref` and `share` consume `Read(E)`; only `@` consumes `CarrierPlace(E)`, and
  `@` is not a general `PlaceOf(E)`.
- `Read(Σ)` yields a complete `⟨Val1?, P, Val2⟩` object and never projects down to
  the `Val1` payload, so `s ref` on `s : symbol` is a `symbol ref` and not a
  reference to the member array inside the symbol value.
- Which of `ref` and `@` applies is decided by the presence of a `Val1` payload,
  not by type-rank. For `let t: type = uint8`, `t ref` is `uint8 ref` and is not
  an error; explicit `t@` is what yields `type ref`.
- Borrow-operator overlap is a real overload family, not a gap:
  `Borrow_k(Borrow_j(q)) = Coerce_{j->k}(Borrow_j(q))` with the target preserved.
  Equal capability is idempotent, `ref share` is an admitted weakening (which is
  what makes `r share` legal), and only `share ref` has no candidate. Borrow
  constructor composition never retargets. `@` on a value instance remains a
  lifetime observation. Retargeting is available only through `rebind`, which consumes a
  place (`Target(E)` or `CarrierPlace(E)`) rather than `Ref(Read(E))`.
- A `type ref` proves only target, lifetime, and borrow capability. It does not
  prove that the current pointee value is open. `extend(Read(r), Δ)` separately
  requires `Open_Γ(Read(r))`; writing the result separately requires
  `Writable(Target(r))`. A frozen type may still be read or wholly replaced
  through a suitable ref.
- The valid region of both `type ref` and `type share` follows ordinary borrow
  lifetime/capability rules. Construction `Open` is a judgment over the value's
  `ConstructionLineage` and the current compile-time stack, not the lifetime of
  either view.
- External stable values are readable and observable through a borrow view but
  are not thereby structurally open: `GlobalLifetime(v)` does not imply
  `Open_Γ(v)`.
- Inner lexical symbols cannot be exposed as longer-lived extension targets;
  storing a view of one outside its observation region is an escape.
- Type values can be equal even when their binding symbols differ.
- `struct` is a symbol-producing structural generator. Its result has a unique
  type member plus generated field/access/assignment/borrow partner families;
  ordinary `let` copies that Symbol value into a fresh binding.
- A let-shaped declaration consumed inside `struct` contributes ordinary Val2
  material to the current Pattern owner. It is neither a structural member nor
  restricted to `Pv=absent`; callable values are admitted.
- `let ()` is the special current-owner call-entry contribution. It creates
  only that owner's `()` entry and does not synthesize entries for `ref` or
  `share` child owners.
- A pure-P member is a real object: each ordinary type binding owns its own
  Val2 place, so `Pattern(T) = Pattern(U) = Pattern(uint8)` coexists with
  `Place(T) != Place(U) != Place(uint8)`. Reads fall back to the Pattern's
  canonical type object for inherited members; writes stay local to the
  carrier's object.
- `Val2(T_t)[f] = C_f` is a recursive cluster carrier for the source-visible name
  `f`, and `C_f` is the single authority for that name. This is the transitional
  same-name synthesis role only; it is not a distinct value ontology. A raw value
  list under the same name is transport material for compiler-installed anonymous
  entries only.
- Exposure of a navigated target is the per-layer phase conjunction over the
  whole host chain (`Expose(g::f::T, φ) = Expose(T_t, φ) ∧ Expose(C_f, φ) ∧ …`),
  decided from each host carrier's own binding-level view rather than from the
  shared TypeObject adapter. It is not a stage-set intersection: a `meta` host
  may carry `compile` members. Ordinary invocation enforces the full
  conjunction — a hidden outer host makes the target unreachable even when the
  terminal is visible.
- A type object's canonical identity is the recursive object normal form
  `Norm_type(x) = ⟨Norm_P(P_x), Norm_Val2(Val2_x)⟩`, where `Norm_Val2` resolves
  each name to its cluster symbol and normalizes that symbol's own members down
  to empty-`Val2` leaves such as `()`. `PlaceId` is the observation coordinate
  `place(x) ⟼ Val2_x` and never identity material: equal `P` with equal
  recursive `Val2` is one identity across different places, and one open type
  observed before and after an extension is two identities (and two meta
  instance keys) through one place.
- Path resolution is
  `Path -> SelectedHead -> ⟨HostChain, TerminalSymbol⟩ -> ContextDirectedProjection`.
  Head selection is its own step: a bare head resolves as `ResolveBare_q(name)` —
  the nearest enclosing same-spelled Symbol carrying the required *coarse* facet
  `q` — and the outward walk stops at that Symbol permanently, so a later overload
  failure inside it is an error rather than a reason to resume searching. An
  explicitly anchored path performs no outward walk. This is what keeps a local
  type-only Symbol from shadowing an outer callable Symbol, while `q` stays coarse
  (facet presence only, never signature, arity, or specificity).
  After the head is fixed, one shared navigator serves every use context. A step
  enters the current symbol's own object `Val2` place and its associated namespace
  and records the traversed object as a host layer; only the final facet projection
  (callable vals / pure-P member / sibling vals / writable place / Pattern) is
  context-directed. `f::T` therefore denotes the same terminal symbol as a call
  target, a type, an extension RHS, a meta argument, and an extraction prefix,
  and every consumer sees the same host chain. Cross-root resolution
  deduplicates on the full navigation, so one terminal reached through distinct
  host chains is a navigation ambiguity rather than a search-root-order pick.
- Implicit type projection is positional, and the two halves are complements:
  an operand/argument position never elaborates `|> type` (`s ref` stays
  `symbol ref`), while a language-designated type-expected position does
  (`Elab_Type(E) = E |> type`) — annotations, type-facet path components, type
  argument positions, `t: type`, `t: type ref`, and type-rank return positions.
- A meta return seal validates *at most* one type member, matching the Symbol
  ontology; the type-facet promotion step is skipped when no type member exists, so
  namespace-only, val-only, and type-less mixed meta returns are well-formed.

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
- `HomeSymbol(TypeValue)` or equivalent canonical-root recovery for a copied or
  extracted type used as a callee, including lookup of defining-Symbol sibling
  constructors/policy transforms. This cannot be derived from the most recent
  binding carrier or `AsType` provenance.
- Final surface mechanism, if any, for requesting coordinated value/ref/share
  receiver candidates in one associated `()` Symbol; the current rule requires
  separate authorized contributions.
- End-to-end syntax/integration for an externally navigated call-entry
  extension such as `let ()::((T ref).type) = ...`; the semantic
  destination and ordinary type-check behavior are fixed, but the current
  frontend does not claim this complete declaration path.

### Resolved symbol-first construction direction

These decisions are no longer open questions and are intentionally not repeated
here. Their normative future-design owners are:

- `spec/design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md`
  for symbol-first facets, `compile` / `meta`, pattern scopes, construction-lineage `Open`
  and freezing, pure `extend` plus place-level `inject`, ordering, extraction handoff, and
  binding/install boundaries;
- `spec/design/symbol-world/symbol-construction-units-and-namespace-origin.md`
  for namespace origin, construction ownership, physical authority, and
  cross-file closure;
- `spec/design/symbol-world/symbol-policy-and-compile-flow-projection.md` for
  `Val1? x Pattern x Val2`, canonical `Pv:Pp`, contextual P1/P2 elaboration,
  three-phase visibility, seal/const-mut boundaries, mechanical
  compile-flow projection over ordinary call nodes, complete derived `Val2`
  compile-companion objects, must-select consistency, policy-staged match,
  intrinsic D/Done flow, coarse automatic require, and shared compile
  evaluation;
- `spec/design/lifetime/lifetime-policy-and-overload-boundary.md` for the
  negative rule that lifetime syntax cannot reopen the unique ordinary
  overload result.

Unimplemented portions remain roadmap work, not reopened design questions.

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
- Completing canonical result constructor/extractor Pattern coherence,
  materialization place/owner allocation, and all structural slot-0 Pattern
  applicability. Ordinary Symbol/Val2/associated-`()`/InvocationFrame routing
  is connected; the caller-supplied migration fixture remains isolated and no
  universal transition Symbol or new callable ontology is implied.
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
  adds or replaces type facets / val siblings by RHS shape is future
  work. Facet resolution currently distinguishes only the pure-P
  type-member facet of the executable slice, and the placeholder
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
- The bare-value pattern family is not designed or implemented:
  `(Expr _) name` / `(Expr _ | others) name` admission, the prohibition
  of `(Expr _, others) name`, merging of equal bare values within one
  `|` layer, `let t _ : Expr = ...`, and `let _ if::bool = ...` are all
  unclosed. They are excluded from the current Pattern-semantics closure
  claim.
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
   to a missing child yields a prospective SubPlace whose contents are `None`;
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

The following questions are **resolved at the future-design level**. They are
not open semantic decisions — only the implementation mechanics and IR-level
representation remain future work.

#### Resolved: no silent discard including void/unit

Status: **Resolved at future-design level** (see §7 of the pattern-spaces document).

The rejected rule was `final pattern = void => silent completion allowed`.
The correct rule is `every expression result must be consumed`. There is no
void exception. If an implementation would otherwise silently discard an
expression result, that position must be interpreted as an error or as the
current block's return boundary.

#### Resolved: block-final unconsumed result is current-block return

Status: **Resolved at future-design level**.

A block-final expression whose result is not otherwise consumed is the return
value of the current block. This applies to `unit` and `void` as well — there
is no silent completion with no result.

#### Resolved: non-final unconsumed result is an error

Status: **Resolved at future-design level**.

If an expression result is not consumed and later same-block material exists,
the program is ill-formed. The repar is either consume/discard the result, or
remove the later material and let the expression become the block return.

#### Resolved: Done isolates completed branch results

Status: **Resolved at future-design level** (see §6 of the pattern-spaces document).

`Done` separates completed branch results from unprocessed continuation
material. It is not eliminated while same-level extraction continuation is
still processing input residuals. Return/result boundaries perform one local
`Done` reduction and re-wrap the result. `Done` is isolated by default but
explicitly re-enterable.

#### Resolved: early function return via self..return(d)

Status: **Resolved at future-design level** (see §6.3.1, §7.5 of the
pattern-spaces document, and the function-object-self-and-return-capability
design note).

Early function return is modeled by calling `self..return(d)` — the current
callable frame's built-in return capability. The effect uses a dual-channel
model: local branch produces `Done(unit)`, and the final return accumulator
receives `Done(D)`. `unit` is absorbed as the zero element of `+` — this is
pattern-space reduction, not silent discard.

#### Resolved: meta return does not collapse creation, write, and control transfer

Status: **Resolved at future-design level** (see the canonical symbol-first
construction note, §4.5).

Target semantics assign no special `let` meaning to the return-slot name: a meta
body computes an ordinary Symbol, ordinary `let` creates members, ordinary `=`
writes existing places, and the return event transfers control. In the current
open-cluster `let`-only encoding (compatibility spellings pending expression-level `=`;
the settled orthogonal target remains `let` creates, `=` writes, return
events deliver): `let r = expr;` adds a
fresh member binding; `r = expr;` writes to an existing target (currently a
placeholder overwrite scaffold; the final cluster write algebra is open); bare
`r;` is the delivery terminal and not a member event. There are three events, not
four: the former interpretations — `r === ...` as a distinct formal forwarding
category, and the interim single-form `r = ...` reading — are both superseded,
and the alias/forwarding member event is retired rather than deferred. There is
likewise no declaration-layer aliasing: `let a = b` binds a fresh symbol in a
fresh place carrying `b`'s value, and a member that must observe an external
object holds a borrow view (`ref` / `share`).

#### Resolved: TypeValueId is value-side material, not carrier identity

Status: **Resolved at future-design level**.

The current bootstrap may first derive a `TypeValueId` projection from an
installed defining core Type Symbol. After that projection has been read as a
value, ordinary binding carries it directly. It is not a carrier Symbol,
installation Place, construction identity, or type-definition identity, and
there is no semantic `TypeValueId -> original carrier Symbol` inverse map.
Thus `let T: type = uint8; let U: type = T` allocates distinct carrier
Symbols/Places while preserving one evaluated TypeValue/PatternValue.
On the value side, `TypeValueId` is only the stable first-order root: the
full type-object semantic identity at an observation moment is the canonical
observation `Addr(Norm_type)`, which identity-critical positions consume
instead of the bare root.

Final invocation plumbing returns an ordinary `PatternValue` for `compile` and
an ordinary `symbol` PatternValue for `meta`. Current `SymbolConstruction` and
`MetaInvocationValue` variants remain transitional implementation transport,
not result ranks or canonical ontology.

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

### Closed by the value/Policy/Open/borrow/`extend`/`inject` semantic closure

These are no longer open questions. They are recorded here only so that they are
not reopened from older wording elsewhere; their canonical owners are the
documents named in each line.

- Object ontology and normal form: `Object x = ⟨Val1?(x), P(x), Val2(x)⟩` with
  `Val1?(x) ∈ 1 + Object`, and `Norm(x)` recursive over all three components.
  There is no `Val1`-presence ontology fork. Well-foundedness covers
  `Children_Val1 ∪ Children_Val2`; bare-Product ordinal elements are ordinary
  `Val2(pos_i)` children, while `T*N`, `T*omega`, and `product` wrappers carry
  that bare Product Object in `Val1`. Product and Sequence are therefore closed
  under Object recursion rather than traversed as compiler aggregates. Direct/
  mutual `Val1` cycles and all other owned cycles have no normal form. An ordinary object's
  carrier/ObjectPlace, `SymbolId`, allocation order, and provenance are not
  identity material; a borrow view's `StableTargetIdentity(Target(view))` is
  value content and is present in its leaf normal form.
  (`type-values-places-and-borrow-views.md`)
- Policy is an observation edge, not an intrinsic field:
  `View_Γ(x) = ⟨x, Pv:Pp, capability_Γ(x)⟩`.
  `Val1?(x) = null => Pv = Pp`, not `Pv = absent`; conversely an observer may
  hide an existing `Val1`. A distinct runtime projection requires an existing
  value component and runtime visibility. There is no central const/mut
  propagation pass — only member overloads and `delete`. The preference rows are
  `succ_const: const > let > mut`, `succ_mut: mut > let > const`, and
  `succ_plain: let > const = mut`; the last row leaves tied `const`/`mut`
  maxima ambiguous when no plain `let` candidate exists.
  (`symbol-policy-and-compile-flow-projection.md`)
- A Symbol is an ordinary PatternValue with `Σ = ⟨T?, V⟩`,
  `V = ⨄_{T_c} V[T_c]`, and homogeneous buckets `V[T_c] : T_c * ω`.
  `Σ` is an ordinary Object composition: the optional type member is an
  empty/singleton bare Product, `(T_c, V[T_c])` bucket entries are `product`
  values, and those entries form a homogeneous `product*omega` Sequence.
  Normalization maps each normalized `T_c` to a set of ordinary recursively
  normalized member objects. Stable member/candidate identity, callable body
  identity, and selection-relevant declaration annotations remain in those
  objects; only insertion order and repeated contribution of the same member
  are quotiented away. Duplicate/conflicting declarations are
  diagnosed during construction. The meta return seal validates the same *at
  most one* type-member bound and skips promotion when none exists.
  (`symbol-first-meta-construction-and-pattern-injection.md`)
- The global privileged type-forming builtin `*` constructs `T*N` and
  `T*omega`; both preserve `rank(T)`. These homogeneous containers, bare
  Product, and `product` form the closed ordered-container kernel and are all
  ordinary Object instances. Generated `[]` stage follows the same
  `RuntimeField`/materialization predicate as structural fields. Layout,
  capacity, growth APIs, and general `product[]` remain outside this PR.
- `struct` closes one generated field mechanism: same-name value/ref/share
  access candidates plus assignment/write partners, all derived from ordinary
  receiver Policy and `RuntimeField`. It does not close defining-Symbol recovery
  for copied/extracted type-as-callee sibling overloads.
- `compile` may return any ordinary PatternValue (including `type`, `symbol`,
  `type ref`, `type share`) under root conservation
  `Roots(Output) ⊆ Roots(Arguments) ∪ Roots(GlobalConstants) ∪
  LexicallyDeclaredStableRoots`. (`symbol-policy-and-compile-flow-projection.md`)
- For an ordinary meta callable, `P2(F) = meta` is biconditional with
  `EstablishNavigableMetaInstanceRoot(MetaInstance(F, Norm(args)))`. Canonical
  arguments must be `GlobalKeyable` at key-creation time; a binder local to the
  caller is allowed, but every owned dependency and horizontal borrow target
  must be already globally stable or already promoted. Future seal promotion
  cannot justify a key created now.
  The exclusivity covers that root kind only. `struct` establishes/selects its
  lexical root from input navigation and ambient scope; `extend` establishes no
  root and preserves `Root(output) = Root(input)`; every other privileged
  builtin must declare its own owner rule.
  (`symbol-first-meta-construction-and-pattern-injection.md` §4.1, §4.8)
- Meta body transparency: a `MetaInstance` is globally live and unsealed on
  entry, and its body does not fire ordinary freezing events. Meta-local
  PatternValues nevertheless have `MetaInvocation` lifetime: compile and
  transparent construction intrinsics may consume them, but another ordinary
  meta invocation may not implicitly globalize them. At seal, only the owned
  PatternValue closure of the returned Symbol's unique type member, if present,
  is promoted.
  Returned val siblings may depend only on already-global material plus that
  promoted closure (only already-global material when no type member exists),
  and borrow targets participate in the check.
  (`symbol-first-meta-construction-and-pattern-injection.md`)
- `Open_Γ(v)` relates `ConstructionLineage(v)` to the current compile-time stack
  and has a one-way `Open -> Frozen` transition. For
  non-meta static control, `Freeze*(Dependencies(c))`; mere `LiveAcross(c)` is
  not a dependency. Values carried across a residual-runtime fork and values
  leaving their ordinary owner interval still freeze.
  (`symbol-first-meta-construction-and-pattern-injection.md` §12.1)
- Borrow views: `ref` / `share` / `@` / `rebind`, and `OwnedClosure` excluding view
  edges. `ref` / `share` consume `Read(E)`, which yields a complete
  `⟨Val1?, P, Val2⟩` object rather than the bare `Val1` payload; `@` consumes
  `CarrierPlace(E)`. Borrowing is non-stacking *because* the overlapping overloads
  exist: `Borrow_k(Borrow_j(q)) = Coerce_{j->k}(Borrow_j(q))` preserves the target,
  `ref share` is an admitted weakening, and only `share ref` has no candidate.
  `rebind` retargets from a place and is not `Ref(Read(E))`. The target is
  horizontal rather than owned recursion, but
  `Norm(Borrow_k(q))` includes `StableTargetIdentity(q)`.
  A prospective `SubPlace(parent, selector)` coordinate is not that resident
  target identity: wholesale parent replacement may invalidate an existing
  child borrow but cannot retarget it to the replacement child. Only `rebind`
  acquires a new target. Both `type ref rebind` and `type share rebind` are
  fixed under a second `rebind` constructor.
  (`type-values-places-and-borrow-views.md`)
- `@` has two positively defined base groups plus borrow-type-value fixed points and
  is an ordinary overloaded operation, not a general `PlaceOf(E)`; the lifetime boundary restricts
  lifetime *rules*, not `@`. Whether `ref` or `@` applies is decided by the
  presence of a `Val1` payload, not by type-rank. On a complete
  `⟨Val1, P, Val2⟩` object `@` takes that object's lifetime; that group is
  unchanged by the narrowing of the borrow-producing group. Borrow type values
  are fixed points (`type ref@ = type ref`, `type share@ = type share`), while a
  value instance `t : type ref` has `t@ = lifetime(t)`. In the pure-slot group
  `Val1?(Value(E)) = null` selects the group and ordinary borrow formation checks
  the explicit carrier place; no Open fact is implied. `@` never performs
  implicit Symbol-to-type projection. A pure type slot uses `t@`; a Symbol uses
  `S.type` by value, `(S ref).type` for `type ref`, and `(S share).type` for
  shared observation.
  (`lifetime/lifetime-policy-and-overload-boundary.md`)
- A `type ref` is an ordinary borrow view of a type-valued slot. Its validity and
  write capability follow ordinary borrow/policy rules; it neither contains nor
  proves an Open witness. Consequently a frozen type ref is coherent: it can be
  read and, if writable, can receive an independently legal replacement value.
  (`type-values-places-and-borrow-views.md` §5.5)
- `extend : type × StructLikeMaterial ⇀ type` is the pure value primitive. It
  separately checks `Open_Γ(old)`, writes no place, creates no root, and preserves
  `Root(output) = Root(old)`. `inject : type ref × StructLikeMaterial ⇀ type ref`
  is `Read -> extend -> Write`; legality requires both Open of the read value and
  writability/lifetime validity of the place, with neither implying the other.
  (`symbol-first-meta-construction-and-pattern-injection.md` §8)
- A `compile` evaluation is construction-transparent and root-non-generative.
  It transports existing PatternValues and lineages, while each Open-sensitive
  operation is checked in the caller's current stack. Returning a `type ref` is
  permitted subject to its ordinary borrow lifetime.
  (`symbol-first-meta-construction-and-pattern-injection.md` §4.2)
- The semantic alias/forwarding family is retired, not deferred. No declaration
  form forwards a Symbol, place, or operator identity; the frozen alias-let Raw
  AST is historical parser preservation only. Operator environments use ordinary
  values, lexical shadowing, and Symbol algebra.
  (`entity-alias-design.md` retirement notice)
