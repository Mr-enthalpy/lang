# v0.6 Semantic Correction — Resolved Record

This is the historical record of the v0.6 namespace graph / early-meta
semantic correction. It was moved here from `spec/planning/open-questions.md`
so that the open-questions file contains only genuinely unresolved work.

This directory is historical. It does not define current public language
behavior. For current behavior, read `spec/public/v0.5/`. The v0.6–v0.8 build /
namespace graph / early meta roadmap direction is
`spec/design/symbol-world/early-meta-functions-and-namespace-graph.md`.

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
- `let T: type = uint8` creates a fresh symbol/place whose type value equals
  `uint8`.
- Type/rank use evaluates by type value, not by symbol name.
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
- `struct` is a symbol-producing structural generator. Its result has one
  `Q_struct` satisfying `Pure(Q_struct)` and `TypeRole(Q_struct)`, plus generated
  field/access/assignment/borrow partner families;
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
  the nearest enclosing same-spelled Symbol satisfying the required *coarse* role
  `q` — and the outward walk stops at that Symbol permanently, so a later overload
  failure inside it is an error rather than a reason to resume searching. An
  explicitly anchored path performs no outward walk. This is what keeps a local
  Symbol with a type-capable `Q` but no callable val member from shadowing an
  outer callable Symbol, while `q` stays coarse (role/member presence only,
  never signature, arity, or specificity).
  After the head is fixed, one shared navigator serves every use context. A step
  enters the current symbol's own object `Val2` place and its associated namespace
  and records the traversed object as a host layer; only the final role/member projection
  (callable vals / pure-P member / sibling vals / writable place / Pattern) is
  context-directed. `f::T` therefore denotes the same terminal symbol as a call
  target, a type, an extension RHS, a meta argument, and an extraction prefix,
  and every consumer sees the same host chain. Cross-root resolution
  deduplicates on the full navigation, so one terminal reached through distinct
  host chains is a navigation ambiguity rather than a search-root-order pick.
- Implicit type projection is positional, and the two halves are complements:
  an operand/argument position never elaborates `|> type` (`s ref` stays
  `symbol ref`), while a language-designated type-expected position does
  (`Elab_Type(E) = E |> type`) — annotations, type-role path components, type
  argument positions, `t: type`, `t: type ref`, and type-rank return positions.
- A meta return seal validates *at most* one pure role member `Q`. When none exists the
  promotion step is skipped; navigable `Val2` and sibling values remain ordinary
  Object content rather than namespace/val return categories.

## Resolved symbol-first construction direction

These decisions are no longer open questions and are intentionally not repeated
in `open-questions.md`. Their normative future-design owners are:

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
