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

- Fields are unary function objects in type-associated companion spaces.
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
  `(int)Vec::std`.
- Final syntax/API shape for resolver expected-role disambiguation; the current
  `lang_build` API is provisional.
- Exact future implementation of writable-place checking.
- Exact future implementation of alias forwarding resolution.
- How meta-function return values expose or hide injection places.
- Interaction between graph freeze, seal phase, and injection-place mutability.
- Whether and how external objects can intentionally expose extension points.
- Whether escaped field names are still needed for namespace-role conflicts
  outside the object/subspace case handled here.
- Exact form of future `unique trait`.
- Full access-tree construction algorithm.
- Full lifetime relation over region/origin facts.
- Interaction between type-value equality and type-associated namespace
  traversal.

---

## v0.7-prep policy correction record

The following point is resolved for the v0.7-prep policy-aware early-meta
track:

- Minimal policy-aware early meta lookup is implemented: `PolicyFlag` /
  `PolicySet` / `PolicyEnv::Meta`, with per-component `Meta` filtering applied
  in the resolver (`resolve_with_policy`) and in early-meta expansion. Core,
  namespace, source-contributed, struct-generated type, and generated
  field-function symbols carry explicit policy flags.
- `PolicyEnv::Meta` is lookup visibility, not callable body execution
  permission.
- Generated field functions are `meta+runtime` visible symbols but runtime-entry
  callables.

Still open after this correction:

- `PolicyEnv::Runtime` resolver mode. The `Runtime` flag is reserved but no
  runtime lookup pass is implemented.
- Full policy lattice.
- Policy projection checking and conformance checking.
- Ordinary function object policy model.
- Alias forwarding resolution under policy filtering.
- Overload buckets and per-policy-pass overload set construction.
- Call execution checker.
- Type checker.
- Runtime residual call construction.
- IR/HIR lowering.

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

#### How should `return`, `else`, `match`, `effect`, `sync` be semanticized?

**Status:** Open (active at later stages)

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
