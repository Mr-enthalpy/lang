# Open Questions

This document tracks open design questions for the `lang` language. Each entry
specifies the stage at which the question becomes active.

Resolved questions have been moved to `spec/history/v0.1/resolved-questions.md`.

---

## Normalized AST questions (v0.3 baseline → v0.5)

These questions arose during v0.3 Normalized AST Specification.

> The current normalized surface behavior is defined by
> `spec/public/v0.5/normalized-surface-semantics-v0.5.md`. The `N-AST` entries
> below are resolved by those public docs and are retained as the
> resolution/audit trail; they no longer define current behavior. Remaining
> Normalized AST items are implementation-shape cleanup (see "v0.5 stabilization
> debt" below). Later pattern-space/extraction-chain questions remain v0.6+.

### v0.3: Normalized AST

#### N-AST-1. Exact Normalized AST node set

**Status:** Resolved by v0.5 public normalized surface semantics (structural roles); exact Rust naming is v0.5 stabilization debt.

**Reference:** `spec/public/v0.5/normalized-surface-semantics-v0.5.md` §3–§11.

**Question:** What are the exact Normalized AST node types? Candidates:
normalized call, normalized pattern, normalized declaration. Should there
be a single unified expression node or distinct per-form nodes?

**Resolution (v0.3, partial):** The call / product / closure / alias structural
boundaries are clarified by the source-product continuation call skeleton in
`spec/public/v0.3/normalized-ast-specification-v0.3.md` §7, and the minimum node
roles for v0.4 start are specified in §8 (unified binding-site structure,
annotation-pattern wrapper, the minimal pattern-role family including
`NormPattern::Skeleton`, no general symbolic builtin family, and family-local
error variants). v0.4 implemented a concrete Normalized AST node set. The final
public wording and stabilization of the concrete Rust node set remain v0.5
stabilization/documentation work.

---

#### N-AST-2. Whether Normalized AST lives in `lang_syntax` or a new crate

**Status:** Resolved

**Question:** Should Normalized AST types and the normalization pass live in
`lang_syntax` (alongside Raw AST), or in a new crate (e.g., `lang_norm`)?

**Resolution (v0.3):** Normalized AST and the normalization pass remain under
`lang_syntax`. No new `lang_norm` crate is introduced at this stage.

---

#### N-AST-3. Whether raw-to-normalized dumps should be golden-tested

**Status:** Resolved by v0.5 public normalized surface semantics.

**Reference:** `spec/public/v0.5/normalized-surface-semantics-v0.5.md` §11.

**Question:** Should the normalization pass produce stable dump output that
can be golden-tested alongside Raw AST dumps?

**Resolution (v0.3):** Yes. `spec/public/v0.3/normalized-ast-specification-v0.3.md`
§8.11 requires v0.4 to expose a stable normalized dump entry point that is
golden-test-stable, does not use raw Rust `Debug` as the public golden format,
and shows enough structure to verify source-product continuation / product
merge, the two legality repairs, product/group lifting, operator lowering and
provenance, member/double-dot/bracket-call lowering, closure body
normalization, alias preservation, annotation-pattern / DeduceList structure,
and error-recovery placement. The exact public dump policy and documentation
wording are stabilized under v0.5; the v0.4 implementation provided the baseline.

---

#### N-AST-4. How to represent symbolic builtins introduced by desugaring

**Status:** Resolved by v0.5 public normalized surface semantics.

**Reference:** `spec/public/v0.5/normalized-surface-semantics-v0.5.md` §11.

**Question:** Desugaring may introduce symbolic names (e.g., `operator::call`,
`member::lookup`, `pattern::bind`). How should these be represented in
Normalized AST — as reserved names, as a separate node type, or as
compiler-generated identifiers?

**Resolution (v0.3):** v0.3 does not introduce a general symbolic builtin node
family (no `BuiltinCall(MemberLookup)` / `BuiltinCall(OperatorCall)` /
`BuiltinCall(PatternBind)`). Generated material from lowering is represented as
a generated unresolved name / nav / operator target carrying origin/provenance,
not as a semantically privileged builtin call. See
`spec/public/v0.3/normalized-ast-specification-v0.3.md` §8.7. Future phases may
introduce semantic builtins if needed; v0.3 Normalized AST does not.

---

#### N-AST-5. How to preserve source origins through desugaring

**Status:** Resolved by v0.5 public normalized surface semantics (origin/visibility model); exact Rust source-map is v0.5 stabilization debt.

**Reference:** `spec/public/v0.5/normalized-surface-semantics-v0.5.md` §11.

**Question:** Desugaring creates new AST nodes that did not appear in source
text. How should source spans and diagnostic attribution be preserved through
normalization?

**Resolution (v0.3, partial):** v0.3 §7.15 requires traceability: normalized
nodes are classified as source, generated, or derived nodes, and generated/
derived nodes must carry enough origin/provenance (the named lowering rule and
contributing inputs) to attribute them to source spans. The exact Rust
source-map representation is deferred to v0.4.

---

#### N-AST-6. Whether right-target subsegments become nested call nodes

**Status:** Resolved by v0.5 public normalized surface semantics.

**Reference:** `spec/public/v0.5/normalized-surface-semantics-v0.5.md` §4–§5.

**Question:** Right-target subsegments (`f (a) g`) are currently flat in Raw
AST. Should normalization recursively nest them into explicit (sub-)call
nodes?

**Resolution (v0.3):** Resolved in favor of the source-product continuation
skeleton, product merge, and the two legality repairs (v0.3 §7.2–§7.7). A
following Product is the first product continuation of an incoming source
Product (`P1 |> e P2 => (P1, P2) |> e`), not an argument list of the target.
`f Product g` is the second legality repair (`f (Product |> g)`), never a
positive local call sugar, and never overrides source-product continuation.

---

#### N-AST-7. How to represent pattern normalization for let, params, returns, and canonical skeletons

**Status:** Resolved by v0.5 public normalized surface semantics; exact Rust enum/struct names are v0.5 stabilization debt.

**Reference:** `spec/public/v0.5/normalized-surface-semantics-v0.5.md` §8–§9.

**Question:** Extraction contexts (let, params, returns) use canonical
skeletons. How should normalization unify these into a single normalized
pattern form? Should deduce lists be merged into the pattern structure
or kept separate?

**Resolution (v0.3):** See
`spec/public/v0.3/normalized-ast-specification-v0.3.md` §8.2–§8.5. The direction
is:

- Binding sites keep a DeduceList hole binder list.
- Annotation is an annotation pattern / classifier pattern.
- DeduceList-declared holes may occur inside annotation patterns.
- DeduceList is not merged into the value/extraction pattern itself.
- Canonical skeletons are preserved as a normalized pattern subform
  (e.g. `NormPattern::Skeleton`), not decomposed into semantic matching.
- No canonical matching, type checking, kind checking, or semantic
  interpretation occurs in v0.3.

A single unified binding-site structure is reused for let slots, closure
parameters, returns, and generated closure heads. Exact Rust enum/struct names
remain v0.5 stabilization/documentation work.

---

#### N-AST-8. How to represent alias declarations before name resolution

**Status:** Resolved by v0.5 public normalized surface semantics.

**Reference:** `spec/public/v0.5/normalized-surface-semantics-v0.5.md` §10.

**Question:** Alias bindings (`let binder === EntityRef`) reference compile-time
entities that are not yet resolved. Should normalization preserve `EntityRefAst`
as-is in normalized alias declarations, or desugar it into a different form?

**Resolution (v0.3):** Alias-let does not participate in call normalization. It
normalizes only into an unresolved alias declaration form; the RHS remains
`EntityRef` (not `PipeExpr`, `Product`, `ClosureAst`, or a runtime expression).
See `spec/public/v0.3/normalized-ast-specification-v0.3.md` §7.13. Alias target
resolution, alias scope semantics, operator-alias identity validation, and
namespace resolution remain deferred to later phases.

---

#### N-AST-9. Member / double-dot sugar lowering proposal — unresolved review concerns

**Status:** Resolved by v0.5 public normalized surface semantics (adopted into v0.3 §7; published in v0.5).

**Reference:** `spec/public/v0.5/normalized-surface-semantics-v0.5.md` §7, §11.

**Context:** A proposed v0.3 lowering for member sugar, double-dot
member-call sugar, and bare branch-name sugar was submitted:

- `expr.field` → `(expr |> <T: type>(val : T) { val |> field::T })`
- `expr..member_fun(args...)` → `(expr |> <T: type>(val : T) { (val, args...) |> member_fun::T })`
- `|> name { body }` → `|> (_ name) { body }`

The following concerns were recorded before adoption, while the proposal was
still under review. They are retained as the resolution audit trail; see
**Resolution (v0.3)** below for how each was settled.

**Concern 1 — lowered closure head omits `=>`.** The lowered member /
double-dot forms write `<T: type>(val : T) { ... }`. A closure head carrying a
deduce list (`<...>`) plus a parameter product is a `FnHeadPrefix`, which the
frozen grammar requires to be followed by `=>` (`concrete-syntax-v0.2.md` §16;
without `=>` the parser emits `InvalidClosureHead`). As written, the targets
are not valid frozen Raw AST source. Decision needed: (a) write the targets as
explicit closures with `=>` (`<T: type>(val : T) => { ... }`), or (b) state
explicitly that the lowered forms are Normalized-AST pseudo-notation, not
re-parseable v0.2 source.

**Concern 2 — bare branch-name rule conflicts with the frozen contract.**
`|> name { ... }` is already expanded to the `(_ name)`-head shape at parse
time and does not survive as a distinct shape into v0.3
(`ast-construction-v0.1.md` §7.1.1; `raw-ast-frozen-surface-v0.2.md` §12:
"v0.3 must receive the explicit-form Raw AST shape. No special-case handling
is needed."; `v0.3-normalization-handoff-checklist.md` §6: "Already desugared
in Raw AST. No further desugar needed."). The proposed "eliminate before
general pipe normalization" rule is therefore vacuous against conforming Raw
AST and risks implying the shorthand reaches v0.3. Decision needed: drop the
rule, or keep it only as a defensive idempotence note that cites the frozen
guarantee.

**Concern 3 — member form silently decides N-AST-1 / N-AST-4.**
`raw-ast-frozen-surface-v0.2.md` §14 states member sugar must desugar to a
"normalized member-access form." The proposal instead reuses pipe + closure +
navigation (`field::T`) with no dedicated member node and no `member::lookup`
builtin. This is a legitimate option, but it answers N-AST-1 (node set) and
N-AST-4 (symbolic builtins) and conflicts with the frozen-surface wording.
Decision needed: accept the navigation-based representation (recording it
against N-AST-1/N-AST-4), and decide whether the frozen v0.2 surface wording
("member-access form") may be adjusted or must stay byte-frozen with the
mapping recorded only in v0.3.

**Also note:** the proposal introduces generated hygienic binders (`T`,
`val`). Their origin/hygiene handling is the first concrete instance of
N-AST-5 ("source origins through desugaring") and should be resolved together
with it.

**Resolution (v0.3):** The navigation-based pipe + closure lowering is adopted
into `spec/public/v0.3/normalized-ast-specification-v0.3.md` §7.11 (member,
double-dot, bracket-call) and §7.14 (defensive branch-name expansion). The three
concerns are settled: (1) the lowered forms are normalized construction
notation — the generated closure is an explicit (headed) closure, so a concrete
v0.2-source rendering would require `=>`, but the normalized notation does not
re-parse as v0.2 source; (2) branch-name expansion is recorded as defensive and
idempotent only, citing the frozen guarantee that Raw AST already expands the
shorthand at parse time; (3) the navigation-based member form is adopted, the
frozen `raw-ast-frozen-surface-v0.2.md` §14 "member-access form" wording is left
byte-unchanged, and the mapping is recorded in v0.3 only. Hygiene of the
generated `T`/`val` binders is covered by the §7.15 provenance requirement
(tracked with N-AST-5).

---

### v0.5 stabilization debt

The public v0.5 normalized surface semantics are published
(`spec/public/v0.5/normalized-surface-semantics-v0.5.md`). The only residual
Normalized-AST items are implementation-shape cleanup, not open
public-semantics questions:

- Final Rust enum/struct names for the normalized node set (N-AST-1) and the
  pattern family (N-AST-7).
- Final Rust origin / source-map representation (N-AST-5).

These are tracked as stabilization/documentation debt; they do not change the
published public behavior.

---

### v0.6+: Canonical form specification

#### How should canonical value/type grammar be designed?

**Status:** Open (active at v0.6)

**Current v0.1 foundation:**
Canonical skeletons use the grammar defined in section 6 of
ast-construction-v0.1.md. This grammar is provisional and may be revised
when value/type canonical forms are designed.

---

### v0.6+: Pattern-space and extraction-chain semantics

Future design note:
`spec/future/static-pattern-spaces-and-extraction-chains.md`.

#### How should pattern spaces be represented as static objects?

**Status:** Open (active at v0.6+)

**Current v0.4 foundation:** Normalized AST preserves the value/pattern
boundary but does not construct semantic pattern spaces.

**Question:** What are the canonical pattern constructors that generate static
pattern spaces? How are product patterns, sum patterns, canonical skeletons,
and meta-function-produced pattern interfaces represented without turning the
compiler into a general set-theoretic solver?

---

#### How should sum patterns and pattern combination be specified?

**Status:** Open (active at v0.6+)

**Current v0.4 foundation:** Pattern-position operator-shaped syntax remains
pattern material. No pattern-space reduction is performed.

**Question:** How should canonical sum pattern syntax (`P1 | P2`) differ from
meta-level pattern combination / reduction (`P + Q`)? Which combinations are
constructible, deleted, or rejected by the relevant meta-level `operator+`?

---

#### How should extraction chains propagate residual pattern space?

**Status:** Open (active at v0.6+)

**Current v0.4 foundation:** Pipe and closure structure are normalized without
pattern-head resolution, extraction applicability checks, or exhaustiveness
checking.

**Question:** Given an extraction atom `A |> S { body }`, how should later
phases mechanically decide whether `S` is an applicable subspace of `A`, how
should `A - S` be represented, and how should extraction failure act as
contextual residual propagation rather than a control-flow primitive?

---

#### How should the `Done` isolation layer work?

**Status:** Open (active at v0.6+)

**Current v0.4 foundation:** Normalization does not insert, eliminate, or
interpret `Done`.

**Question:** Where is `Done` introduced for completed extraction-arm results,
which boundaries eliminate it, and how can users explicitly re-enter or consume
`Done`-wrapped material without allowing same-level extraction arms to
implicitly inspect completed results?

---

#### How should binding and product extraction totality be checked?

**Status:** Open (active at v0.6+)

**Current v0.4 foundation:** Binding and closure parameter extraction preserve
pattern/product structure, including explicit unit positions, but perform no
matching or totality checks.

**Question:** In binding contexts where skip has no residual object to
propagate, how should later phases check exact extraction, product-field
completeness, explicit `_` discard, and hard structural product-pattern errors?

---

#### How should result consumption and void-only silent completion work?

**Status:** Open (active at v0.6+)

**Current v0.4 foundation:** Normalization preserves expression and closure-body
structure but does not check whether expression results are consumed.

**Question:** At expression and body boundaries, which pattern spaces must be
bound, passed, returned, explicitly discarded with `_`, or closed by a consumer?
How is pure `void` distinguished from ordinary unit payload?

---

#### How should postfix `?` and conventional `match` closing be modeled?

**Status:** Open (active at v0.6+)

**Current v0.4 foundation:** `?` remains operator-shaped syntax / operator
lowering material, and `match` remains an ordinary name unless later phases
give a library convention meaning to a specific binding.

**Question:** How should postfix `?` perform explicit value-to-pattern
conversion, for example to a closed `if | else` control-pattern space? How can
a conventional identity consumer such as `match` force explicit total
consumption and one `Done`-elimination boundary without becoming a privileged
parser or normalizer form?

---

#### How should closed control-pattern spaces avoid ambient extension?

**Status:** Open (active at package/name-resolution stages)

**Current v0.4 foundation:** Normalization performs no namespace/package
ownership checks, no operator lookup, and no ADL-like forwarding.

**Question:** How do package ownership, explicit lookup routing, and the absence
of unrestricted ADL prevent downstream code from adding ambient candidates that
make closed control-pattern residuals combine with unrelated pattern spaces?

---

### v0.10+: Ownership and NLL

#### How should the NLL CFG be structured?

**Status:** Open (active at v0.10)

**Current v0.1 foundation:**
No CFG is built. The raw AST contains sufficient structure (form order,
closure bodies, and explicit `with { ... }` syntax) for future passes to
construct a control-flow graph.

---

### v0.11+: Control-flow and effect semantics

#### How should `return`, `else`, `match`, `effect`, `sync` be semanticized?

**Status:** Open (active at v0.11)

**Current v0.1 foundation:**
These are ordinary `Name` tokens at the lexical and parser level. No special
AST nodes exist for them. The v0.1 frontend faithfully preserves these names
in expression AST.

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

---

## Documentation reset debt

Items resolved during the documentation reset pass. Recorded here for audit.

| Item | Implementation status | Spec state (before reset) | Action taken | Blocking |
|---|---|---|---|---|
| Operator syntax added after initial v0.1 boundary | Implemented as raw AST sugar | AGENTS.md said "do not implement operator syntax" | Updated AGENTS.md, SKILL.md | No |
| Alias parser preservation after entity-alias documented as future | Implemented as raw AST preservation | AGENTS.md, SKILL.md, README.md said "future only" | Updated all entry docs + entity-alias-design.md | No |
| `acquire` superseded | Previously reserved | `acquire` direction replaced by active `pre`/`post` head clauses (plus `require`/`lifetime pre`/`lifetime post`) | No |
| EntityRef general design vs alias-RHS subset | AliasRhsEntityRef implemented; GeneralEntityRef future | entity-ref-design.md said "not implemented" | Split into status banner distinguishing AliasRhsEntityRef vs GeneralEntityRef | No |
| `InvalidAliasBinder` diagnostic reserved but not emitted | In DiagnosticCode, never triggered | Undocumented as reserved | Marked "reserved; not currently emitted" in diagnostics spec | No |
| `UnusedClosureAst` diagnostic optional / not guaranteed emitted | In DiagnosticCode, may never trigger | Documented as optional | Clarified "not guaranteed to be emitted" in diagnostics spec | No |
| Right-target subsegment AST shape | Flat representation; future may nest | Already open question §4 | No change needed | No |
| Form boundary promotion rules | Provisional rules implemented | Already open question §2 | Replaced with strong-semicolon rule (§2). Newline promotion removed. | No |
| Prefix-negative normalized form divergence | Not implemented (v0.3 spec opinion only) | `operator-design.md` and `glossary.md` show `()zero::(x \|> type) - x`; v0.3 §7.10 records `(x \|> <T: type>(val: T) { (zero::T, val) \|> - })` | Recorded the v0.3 form; deferred reconciliation of `operator-design.md`/`glossary.md` to a later consistency pass (`raw-ast-frozen-surface-v0.2.md` §13 defers the exact form to v0.3) | No |
