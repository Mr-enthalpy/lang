# Overload Resolution Design

**Status: Mixed.** Full overload resolution remains non-normative future
design. v0.8 implements only the restricted source-declared meta-overload
selection slice described in §0.1.

This document remains the broader overload-resolution design. The earlier
pattern/type candidate-preparation subset used by future meta object invocation
is documented in `pattern-normalization-and-first-order-overload.md`. That
subset is the narrower candidate model — argument/parameter shapes, applicability,
and a constrained specificity ordering — that the meta invocation engine needs;
it is **not** equivalent to full runtime overload resolution, which this document
continues to specify.

---

## 0.1 v0.8 restricted implemented slice

The v0.8 implementation is deliberately narrower than this full design. It
supports source-declared callable/meta-function overloads harvested through:

```text
source text
  -> lexer
  -> parser
  -> normalizer
  -> declaration harvesting
  -> namespace graph symbols
  -> overload candidate construction
```

Implemented for this slice:

- multiple same-name callable object-role children under one namespace node;
- non-call lookup of a same-name overload set as ambiguity, not silent choice;
- `meta | runtime` declaration-policy elaboration to symbol self-policy
  `{ Meta, Runtime }`;
- `: meta ->` body-entry policy as `{ Meta }`;
- return-object policy defaulting to the symbol self-policy;
- C0 from selected namespace graph children by name, role, arity, and
  source-callable shape;
- C2 self-policy filtering for `MetaAction` and metadata for
  `RuntimeBinding`;
- C3 restricted parameter extraction-pattern applicability;
- C4 body-entry eligibility for demanded meta execution;
- C7' extraction-pattern specificity with the lexicographic tuple in §4;
- unique selection or hard ambiguity diagnostics;
- selected delete-body diagnostics and simple `r === x` forwarding bodies.

Explicitly not implemented in v0.8:

- full runtime overload resolution;
- concept legality or concept ordering (`C5`, `C6`);
- first-order instantiation preference (`C8`);
- lifetime precondition matching or lifetime specificity (`C9`, `C10`);
- ADL, unrestricted lookup, or global search for all symbols of a name;
- D/Done reduction or control-flow pattern transformation;
- guarded branch invocation, short-circuit invocation, or full meta block
  interpretation.

Declaration order is not a semantic tiebreaker. When implemented stages leave
two equal maximal candidates, selection reports ambiguity.

`meta | runtime` is a policy expression in declaration-policy context. Its
`|` is policy-set union. It is not pattern-space canonical sum, not
expression-level operator lookup, and not evidence that the body may execute
under runtime policy. Pattern-side forms such as `_ if | else: type` are parsed
and interpreted in parameter-pattern context only.

Three policy planes remain separate:

```text
symbol self-policy  -> lookup visibility
body-entry policy   -> execution permission
return-object policy -> result object capability
```

For:

```lang
meta | runtime let + =
  (self, t: type, u: type): meta -> let r: type =>
{
  r === t;
};
```

v0.8 elaborates:

```text
symbol self-policy = { Meta, Runtime }
body-entry policy = { Meta }
return-object policy = { Meta, Runtime }
```

The current `+` overload support is not compiler-intrinsic set union. `+`
remains a source-declared locatable operator/callable symbol, and candidate
sets come from the selected namespace graph view.

---

## 1. Scope

This document is the formal specification of overload resolution for the lang
language. It defines:

- how overload candidate sets are constructed from namespace graph children
- how visibility and export rules gate internal vs external lookup
- how self-policy filtering removes candidates inapplicable to the current phase
- the distinction between self-policy (visibility) and body-entry policy
  (execution priority)
- the extraction-pattern specificity rule as a stable lexicographic rank
- the full overload resolution pipeline from raw symbol lookup to uniqueness
- a compact judgment form

This document does **not** define:

- concept semantics (only the interface: `concept_projection` must produce a
  stable poset element)
- lifetime / origin-path graph construction
- ADL or unrestricted symbol search
- implicit type conversion or coercion
- partial-application overloads
- package-internal symbol aliases as overload candidates

---

## 2. Overload Set Construction

### 2.1 Per-policy-pass separation

Symbol lookup is **not** a single pass that merges all policy environments.
Different compiler phases operate under different `LookupPhase` values:

```text
LookupPhase ::=
  | CompileEval       -- compile-time value evaluation
  | MetaAction        -- AST-level meta-function expansion
  | RuntimeBinding    -- runtime symbol binding
```

Each phase specifies which self-policy flags are admissible. A symbol whose
`self_policy` does not intersect the phase's admissible policy set is excluded
from the candidate set at step C2.

```text
admissible(CompileEval)    = { Compile, Meta }
admissible(MetaAction)     = { Meta }
admissible(RuntimeBinding) = { Runtime }
```

A runtime-only symbol is never a candidate in `CompileEval` or `MetaAction`
phases. A compile-only symbol is never a candidate in `RuntimeBinding`.

This means the body-entry priority `compile > meta > runtime` (§5.5) operates
**within** a phase, after the candidate set has already been filtered by
self-policy. It does not merge all phases into one pool.

### 2.2 Visibility and export

Overload candidate construction begins from namespace graph children, filtered
by a visibility view `V`:

```text
V ::= Internal | External
```

```text
Visible(C, Internal) = C
Visible(C, External) = { c ∈ C | export(c) }
```

- **Internal** lookup: all children of the current namespace are candidates.
  `export` is irrelevant.
- **External** lookup: only `export`-bearing symbols are visible. External
  path traversal must be export-gated segment-by-segment.

The overload set is always built from the visible child set, never from raw
graph children directly.

Candidate construction is closed over the namespace-graph view selected for the
query: it performs no ADL-like expansion and no external scope search, external
users cannot retroactively add candidates to an already-visible namespace node,
and meta-generated injection occurs only through explicit parent-to-child
namespace-delta boundaries.

### 2.3 Candidate set notation

```text
C0 = RawChildren by name, role, arity, and syntactic callable shape
C1 = Visible(C0, V)
C2 = UsableBySelfPolicy(C1, ρ)
```

- `C0`: all same-named symbols in the namespace graph matching the call's
  syntactic shape and expected arity / role.
- `C1`: filtered by visibility view (internal or external).
- `C2`: filtered by self-policy compatibility with the current `LookupPhase`.

---

## 3. Self-Policy vs Body-Entry Policy

Two distinct policy dimensions operate at different stages:

| Dimension | Field | Controls | Pipeline step |
|---|---|---|---|
| **Self-policy** | `SymbolObject.policy_metadata.policy_set` | Whether the symbol is visible as a candidate in the current phase | C2 |
| **Body-entry policy** | `MetaFunctionObject.body_entry_policy` | Execution priority of the function's body (compile > meta > runtime) | C4 |

### 3.1 Self-policy (C2)

A symbol is usable in lookup phase `ρ` iff:

```text
usable(c, ρ) := self_policy(c) ∩ admissible(ρ) ≠ ∅
```

where `self_policy(c)` is the set of `PolicyFlag` values on the symbol.

### 3.2 Body-entry policy (C4)

After pattern and type matching (C3), candidates are ranked by their body-entry
policy. The rank is:

```text
body_entry_rank ::= compile > meta > runtime
```

Given matched candidates C3:

```text
C4 = { c ∈ C3 | body_entry_rank(c) is maximal in C3 }
```

A `compile`-entry candidate is eligible **only if** all required arguments are
available as compile-time values. If this condition is not satisfied, the
candidate is not merely lower priority — it is not eligible at all.

---

## 4. Formal Specificity — Lexicographic Rank

### 4.1 Definitions

Let `E` be the normalized extraction tree of the call operand (the unified
construction-expression tree). Every node `n ∈ E` has depth:

```text
depth(root) = 1
depth(child) = depth(parent) + 1
```

A candidate overload pattern `P` is matched against `E`. Define:

```text
C(P, E) = nodes of E explicitly visited by pattern P
          where the corresponding pattern node is explicitly
          written as one of:
            constructor match
            binder (including type-rank binders like <t: type>)
            literal match
            type / rank match
            discard _
```

```text
D(P, E) = { n ∈ C(P, E) | the corresponding pattern node is
           explicit discard _ }

M(P, E) = C(P, E) \ D(P, E)       -- matched (non-discard) nodes
```

### 4.2 Specificity tuple

```text
specificity(P, E) =
  (
    max depth(n)   for n ∈ C(P, E),    -- deepest explicit penetration
    Σ depth(n)     for n ∈ C(P, E),    -- total explicit depth contribution
    |M(P, E)|                           -- non-discard explicit node count
  )
```

### 4.3 Comparison rule

```text
P₁ more specific than P₂ wrt E
  iff
specificity(P₁, E) > specificity(P₂, E)   lexicographically
```

The lexicographic order resolves the "many shallow `_` outrank one deep
constructor match" problem:

1. If P₁ reaches a deeper node than P₂, P₁ wins — deeper penetration is
   the primary signal.
2. If maximum depth is tied, the pattern with greater total depth
   contribution wins.
3. If total depth is tied, the pattern with more non-discard explicit
   nodes wins — explicit binders and constructor matches carry more
   semantic weight than `_` discards.

Discard `_` contributes depth because it asserts the user knows and
requires that structure. But at equal depth totals, more binders win.

### 4.4 Examples

**Example 1: Generic built-in vs specific overload.**

```text
ref(self, t: type)
```
Matches only the abstract type parameter — contribution is small.

```text
ref(self, let <t: type> t ref: type) => delete
```
Explicitly matches `t ref`, penetrating to the `ref` constructor layer.
Its `max depth` and `sum depth` are both higher, so it outranks the generic
built-in.

```text
ref(self, let <t: type> t share: type) => delete
```
Similarly outranks `ref(self, t: type)`. So `ref(T share)` hits the delete
overload and does not fall through to the built-in.

**Example 2: Type-rank binder contributes depth.**

```text
<t: type>
```
Not a passive variable hole — it explicitly requires the current position to
be a type-rank object. This node's depth contributes to specificity.

---

## 5. Overload Resolution Pipeline

### 5.1 Notation

```text
C0  = RawChildren by name, role, arity, and syntactic callable shape
C1  = Visible(C0, V)                        -- V ∈ {Internal, External}
C2  = UsableBySelfPolicy(C1, ρ)             -- ρ ∈ LookupPhase
C3  = ShapeAndTypeMatch(C2, E)              -- structural pattern + type matching
C4  = MaxBodyEntryPolicy(C3, ρ)             -- body-entry rank: compile > meta > runtime
C5  = ConceptLegal(C4, E)                   -- remove concept-violating candidates
C6  = MaxConceptOrder(C5, E)                -- keep maximal under concept poset
C7  = MaxExtractionSpecificity(C6, E)       -- lexicographic specificity (§4)
C8  = PreferFirstOrderOverInstantiated(C7)  -- first-order before instantiated
C9  = LifetimePreSatisfied(C8)              -- remove candidates failing lifetime pre
C10 = MaxLifetimeSpecificity(C9)            -- origin-path extraction specificity

select unique element of C10
  if |C10| = 1 → selected
  if |C10| = 0 → error: no matching overload
  if |C10| > 1 → error: ambiguous overload
```

### 5.2 Pipeline invariants

Each step `Ci+1 = f(Ci, ...)` satisfies:

```text
Ci+1 ⊆ Ci                              -- monotonic filtering
f is side-effect-free                  -- no observable effects
f is independent of candidate order    -- same result regardless of iteration order
```

### 5.3 C0: RawChildren

Initial candidates are drawn from the namespace graph children at the lookup
site, matching by:

- **name**: same textual name (or operator-identity equivalence)
- **role**: object-role symbols for callable targets; namespace-subspace for
  namespace-qualified lookup
- **arity**: compatible argument count (exact, variadic, or defaulted)
- **syntactic callable shape**: operator identity (`spelling + fixity + arity`)
  for operator calls, ordinary name for ordinary calls

### 5.4 C1–C2: Visibility and self-policy

Defined in §2 and §3 respectively.

**Ordering constraint**: visibility (C1) precedes self-policy (C2). If a
symbol is not visible in the lookup view, its policy is not checked.

### 5.5 C3–C4: Pattern/type matching, then body-entry policy

**Structural matching (C3)** must precede body-entry policy (C4). A
`compile`-body candidate cannot win on policy alone if its pattern or
type signature does not match the call operand.

`ShapeAndTypeMatch(C2, E)` removes candidates whose:

- extraction pattern is structurally inapplicable to `E`
- type signature is incompatible with the argument types

**Body-entry policy (C4)** applies after matching: among matched candidates,
keep those with maximal `body_entry_rank` (compile > meta > runtime), subject
to the current `LookupPhase` admitting that entry policy.

A `compile`-entry candidate is eligible only if all required arguments are
available as compile-time values. If this condition is not met, the candidate
is not eligible — it does not merely lose to meta.

### 5.6 C5–C6: Concept layer (deferred)

Full concept design is deferred to later phases. This section defines only the
interface that overload resolution depends on.

**Concept legality (C5)**: remove candidates whose concept constraints are
violated by the call site.

**Concept poset ordering (C6)**: given a function `concept_projection(c, E)`
that maps each surviving candidate to an element of a `ConceptOrder` poset,
keep candidates with maximal concept order.

If multiple candidates have incomparable maximal concept orders, they all
survive into C6 and proceed to extraction specificity (C7).

### 5.7 C7: Extraction-pattern specificity

Defined in §4. Among the surviving candidates, compute `specificity(P, E)` for
each and keep those with maximal specificity (lexicographically).

### 5.8 C8: First-order before instantiated

If candidates are otherwise equal under all preceding steps, prefer a
first-order (non-instantiated) candidate over a candidate obtained by first-order
instantiation.

This is a **tie-breaker only**. It does not override extraction specificity
(C7). A deep generic pattern outranks a shallow monomorphic pattern.

### 5.9 C9–C10: Lifetime layer (deferred)

Full lifetime / origin-path design is deferred to later phases. This section
defines the interface.

**Lifetime pre-check (C9)**: remove candidates whose `pre` / `lifetime pre`
conditions cannot be satisfied. Since origin-path matching is structurally
analogous to extraction matching, this check uses the same pattern-matching
primitives.

**Lifetime specificity (C10)**: among viable candidates, compare origin-path
extraction specificity. Define `L(P_life, O)` where `O` is the lifetime /
origin graph normalized into an origin-path tree, and `P_life` is the candidate's
lifetime pre-pattern:

```text
L(P_life, O) =
  (
    max explicit origin-path depth,
    total explicit origin-path depth,
    non_discard_origin_node_count
  )
```

Candidates with maximal `L(P_life, O)` survive. The comparison rule is the
same lexicographic order as §4.3.

**Ordering constraint**: lifetime pre-check (C9) may depend on the selected
candidate's concrete type or instantiation result. C9 therefore follows C8
(first-order instantiation).

### 5.10 Uniqueness

```text
if |C10| = 1 → select the unique candidate
if |C10| = 0 → error: no matching overload
if |C10| > 1 → error: ambiguous overload
```

Neither zero nor multiple candidates are acceptable. There is no fallback to
where a declaration was written. The written listing may be used for diagnostic
message presentation only.

---

## 6. Judgment Form

```text
Γ; V; ρ ⊢ name(args) ⇓ f

where:
  Γ  = namespace graph + type / concept / lifetime environment
  V  = lookup visibility view (Internal | External)
  ρ  = LookupPhase (CompileEval | MetaAction | RuntimeBinding)
  E  = normalized extraction tree of the call operand name(args)
  f  = the selected unique overload candidate
```

The judgment reads: in environment `Γ`, under visibility view `V` and lookup
phase `ρ`, the call `name(args)` with extraction tree `E` resolves to overload
candidate `f`.

Derivation:

```text
C0  = RawChildren(Γ, name, role, arity, shape)
C1  = Visible(C0, V)
C2  = UsableBySelfPolicy(C1, ρ)
C3  = ShapeAndTypeMatch(C2, E)
C4  = MaxBodyEntryPolicy(C3, ρ)
C5  = ConceptLegal(C4, E)
C6  = MaxConceptOrder(C5, E)
C7  = MaxExtractionSpecificity(C6, E)
C8  = PreferFirstOrderOverInstantiated(C7)
C9  = LifetimePreSatisfied(C8)
C10 = MaxLifetimeSpecificity(C9)

|C10| = 1       C10 = { f }
───────────────────────────
  Γ; V; ρ ⊢ name(args) ⇓ f
```

---

## 7. Relationship to v0.7-prep Implementation

The v0.7-prep work (PR #56) provides the self-policy filtering layer (step C2)
that future overload resolution will invoke:

- `PolicyFlag::Export`, `PolicyFlag::Meta`, `PolicyFlag::Runtime` — symbol policy
  flags carried on `PolicyMetadata.policy_set`
- `PolicySet` — bit-set of flags
- `PolicyEnv::Meta` — the `MetaAction` lookup phase with `admissible = { Meta }`
- `ResolverCode` — miss vs ambiguity discriminator

These are used in early-meta expansion (`try_expand_early_meta_initializer`)
which performs a per-policy-pass lookup of meta-function targets. This is
step C2 in the overload pipeline: `UsableBySelfPolicy(C1, MetaAction)`.

No other overload resolution steps are implemented. Steps C3–C10 are deferred
to v0.10+.

---

## 8. Deferred / Non-Goals

The following are explicitly **not** part of this design and are deferred to
later phases or separate documents:

```text
ADL (argument-dependent lookup)
implicit type conversion / coercion ranks
partial-application overloads (curried candidate matching)
package-internal symbol aliases as overload carriers
concept inference and concept lattice construction
full lifetime / origin-path graph construction
operator identity disambiguation (spelling + fixity + arity is presumed)
implicit discard as a candidate-selection mechanism
declaration-order fallback
```

---

## 9. Relationship to Other Documents

| Document | Relationship |
|---|---|
| `static-pattern-spaces-and-extraction-chains.md` §12 | Summary overview of overload resolution; this document is the formal specification |
| `pattern-normalization-and-first-order-overload.md` | Earlier, narrower candidate-preparation subset (pattern normalization + first-order type-value candidate model) feeding meta object invocation; not full runtime overload resolution |
| `mechanical-argument-passing-and-move-fixed-point.md` | Pass-mode adaptation (move/ref/share/copy/in) is separate from type/rank compatibility; `move` does not create a new type value |
| `call-modes-recursion-and-tail-lowering.md` | Candidate selection feeds invocation lowering, which may eventually produce explicit call modes (`normal` / `tco` / `loop`) |
| `policy-visibility-symbols.md` §12.1 | Policy-filtered namespace lookup feeds overload set construction |
| `early-meta-functions-and-namespace-graph.md` | Namespace graph provides the `RawChildren` (C0) layer |
| `entity-ref-design.md` | Entity references may resolve through overload candidate sets in later phases |
| `glossary.md` | Defines OverloadCandidate, OverloadSpecificity, OverloadResolutionPipeline |
| `roadmap.md` | v0.8 non-goal, v0.10+ gating phase |
