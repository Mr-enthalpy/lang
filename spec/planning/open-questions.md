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
- `ref` and `share` are namespace subspaces, not reserved field names.
- Function-object names and namespace-subspace names may be identical under the
  same parent when they occupy different child-name roles.
- Fields named `ref` or `share` are allowed.
- Terminal `ref::T` or `share::T` may be ambiguous without a resolver expected
  role.
- `a::T`, `a::ref::T`, and `a::share::T` are intended type-associated namespace
  paths for field function objects.
- `let T: type = uint8` is ordinary type-value binding: it creates a new symbol
  `T` whose type value equals `uint8`.
- Type/rank use evaluates by type value, not by symbol name.
- `let T: type = uint8` creates a fresh symbol/place whose type value equals
  `uint8`.
- `let f::T = ...` injects into `T` as a place, not into `uint8` as a value.
- Injection is place update, analogous to `a = a + 1`, not value rewriting.
- `let T === uint8` is symbol alias / forwarding, not ordinary type-value
  binding.
- `let T === uint8` does not create a fresh writable place.
- Injection through an alias is allowed only if the final forwarded target is a
  current-level open writable object.
- External stable values are readable / aliasable but not writable injection
  targets.
- Inner lexical symbols cannot be exposed as longer-lived injection targets.
- Type values can be equal even when their binding symbols differ.
- `struct` meta generation creates a fresh type value; ordinary `let` binding
  to an existing type value does not.

Still open after this correction:

- Exact representation of `TypeValueId` and canonical type-value equality.
- Exact representation of symbol/place identity.
- Exact future lowering of generic/meta-generated type expressions such as
  `(int Vec::std)`.
- Final syntax/API shape for resolver expected-role disambiguation; the current
  `lang_build` API is provisional.
- Exact future implementation of writable-place checking.
- Exact future implementation of alias forwarding resolution.
- Exact Rust/IR representation of `SymbolConstructionValue` facet exposure;
  semantically, formal invocation remains uninstalled and outer binding resolves
  the installation place.
- Interaction between graph freeze, seal phase, and injection-place mutability.
- Whether and how external objects can intentionally expose extension points.
- Whether escaped field names are still needed for namespace-role conflicts
  outside the object/subspace case handled here.
- Exact form of future `unique trait`.
- Full access-tree construction algorithm.
- Full lifetime relation over region/origin facts.
- Interaction between type-value equality and type-associated namespace
  traversal.

### Resolved symbol-first construction direction

These decisions are no longer open questions and are intentionally not repeated
here. Their normative future-design owners are:

- `spec/design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md`
  for symbol-first facets, `compile` / `meta`, pattern scopes, owned-open
  `inject`, ordering, extraction handoff, and binding/install boundaries;
- `spec/design/symbol-world/symbol-construction-units-and-namespace-origin.md`
  for namespace origin, construction ownership, physical authority, and
  cross-file closure;
- `spec/design/symbol-world/symbol-policy-and-compile-flow-projection.md` for
  `Val1 x Pattern x Val2`, canonical `Pv:Pp`, contextual P1/P2 elaboration,
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
- A function-object binding has the unrestricted empty mutability domain by
  default (`const || mut`); only its declaration may crop that axis. An export
  root is the contextual exception: bare `export let` elaborates as
  `export + const`, and any written export domain containing `mut` is invalid.
- `...Q` is available in every let-shaped binding slot, not only parameters.
  It remains one Pattern remainder constructor, never a pack type or RHS
  unpack. Raw `...(a, b)` is preserved but rejected after P normalization
  because the bare Product has no stable top mode. At an ordered level,
  explicitly headed structure such as `...((a, b) pair)` may be admissible;
  unordered levels accept only a whole-remainder binder/discard. Every Pack
  contributes one outward specificity node, and internal structure never
  becomes multiple same-level EP nodes.
- DeduceLists elaborate as left-to-right telescopes with exact
  `HoleBinderId`-targeted references. Declarations see inherited and preceding
  holes, not themselves or later declarations; active names cannot be
  redeclared or shadowed. Parser/Norm recursive preservation and the identity
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
- Inferred require retains coarse complete blocks and guarded branch groups,
  conjoins with manual require, and shares one compile-evaluation graph with
  body continuation.

Implemented substrate after this correction:

- Raw and Normalized AST preserve dedicated conjunction, choice, atom, pair,
  and absent-value policy nodes. Pattern `|` and policy `||` are distinct.
- `lang_build` provides typed pair normalization and true slice restriction,
  three contextual P1 elaborators, three-phase exposure, structural compile
  flow projection, Wpre/export closure, and phase/const-mut product-order test
  substrate.
- Flat policy flags remain compatibility transport, while lookup and execution
  environments use the same three canonical phases.

Not implemented after this correction:

- Storing and checking canonical `Pv:Pp` on every symbol/value object.
- Storing policy-pair views on every namespace entry and routing every build
  operation through the typed P1 projection.
- Integrating structural compile-flow projection with the complete evaluator.
- Materialized derived companion objects and must-select enforcement.
- Automatic inferred require, a complete overload resolver, and a call
  execution checker.
- Closure materialization, lazy embedding-layer lookup for in-place closures,
  and the in-place-over-non-in-place overload filter.
- Result Pattern delivery/D-reduction; the current return-target substrate only
  retains the complete return binding slot and selects a restricted active
  frame.
- Any positive lifetime/Horae design.
- Alias forwarding under policy projection, type checking, and runtime IR.

Still open for later design:

- the final source token for the absent-value policy pattern (`S`, `null`, or
  another spelling);
- the complete runtime reflection object model;
- the semantics of any additional future policy stage;
- which named overload strategies beyond compiler-known
  `must_select_if_qualified` are provided, and each rule's monotone comparison
  semantics (the source spelling `=> name {}` / no-`=>` `[[name]] {}` is now
  fixed);
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
function object's built-in return capability. The effect uses a dual-channel
model: local branch produces `Done(unit)`, and the final return accumulator
receives `Done(D)`. `unit` is absorbed as the zero element of `+` — this is
pattern-space reduction, not silent discard.

#### Resolved: formal meta return has one construction form

Status: **Resolved at future-design level** (see the canonical symbol-first
construction note).

`r = ...` assigns pattern/facet material to the return layer of a
`SymbolConstructionValue`. The former formal `r === ...` forwarding category is
superseded. Ordinary declaration aliasing remains `let a === b` and continues to
forward symbol/place lookup according to the alias model.

#### Resolved: TypeValueId is projection material only

Status: **Resolved at future-design level**.

`TypeValueId` is a derived first-order type-value projection material. It is not
a symbol, binding source, installation place, construction identity, or
type-definition identity. Final invocation plumbing returns `PatternValue` for
`compile` and `SymbolConstructionValue` for `meta`. Current
`MetaInvocationValue` variants are transitional implementation transport.

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

### Name resolution and alias validation

#### Operator alias identity mismatch: diagnostic phase

**Status:** Open (active at name resolution)

**Current Phase 4.3 design:**
The operator alias rule requires `spelling + fixity + arity` match between
binder and target leaf, where fixity is `Binary` or `Postfix` (overloadable
fixities only). Prefix negative `-x` is a normalization-special-cased surface
sugar, not an overloadable operator identity; the `-` spelling in alias binder
or target position refers exclusively to binary minus. The design document
recommends deferring the full identity check to a static validation or
name-resolution-adjacent phase. A first-pass spelling-only comparison is
possible as optional future parser validation.

**Question:** Should operator alias identity mismatch be a parser diagnostic
(spelling-only), a static semantic diagnostic (full identity), or deferred
to name resolution?
