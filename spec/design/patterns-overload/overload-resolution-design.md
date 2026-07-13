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

The canonical definitions of symbol total policy, callable `P1` / `P2`,
compile-flow projection, derived compile companions, and
`must_select_if_qualified` are in
`../symbol-world/symbol-policy-and-compile-flow-projection.md`. This document
owns how those entries participate in the overload pipeline; it does not
redefine their staging semantics.

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
- `meta | runtime` declaration-policy elaboration to the current transitional
  symbol-policy metadata `{ Meta, Runtime }`;
- `: meta ->` elaboration to the current `body_entry_policy` `{ Meta }`;
- transitional return-object policy transport defaulting to the current symbol
  policy;
- C0 from selected namespace graph children by name, role, arity, and
  source-callable shape;
- transitional self-policy filtering for `MetaAction` and metadata for
  `RuntimeBinding`;
- restricted parameter extraction-pattern applicability;
- current body-entry eligibility for demanded meta execution;
- C7' extraction-pattern specificity with the lexicographic tuple in §4;
- unique selection or hard ambiguity diagnostics;
- selected delete-body diagnostics and the current legacy `r === x`
  forwarding-body substrate.

This implemented C0 bucket is transitional. Final call preparation resolves one
symbol, projects and enumerates its heterogeneous value facet, filters each
object by its own `P1`, obtains each surviving value's type, resolves each
type-associated `()` entry, discards non-callable entries, and then performs
applicability/`P2`/result filtering to a unique maximal candidate. Same-name
value entries are not assumed to be same-type function overloads. See
`spec/design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md`.

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

The implementation currently stores three metadata fields:

```text
symbol policy metadata
body-entry policy metadata
return-object policy metadata
```

They are not three final source-level policy positions. In the final model,
base path resolution produces `Symbol` before `P1`; each enumerated `Val2`
object's lookup participation is governed by its own `P1`, entry execution and
exact argument total policy by `P2`, and there is no independent `P3`. The
current return-object field is provisional transport until layered
result-symbol facets exist.

For:

```lang
meta | runtime let + =
  (self, t: type, u: type): meta -> let r: type =>
{
  r === t;
};
```

v0.8 currently elaborates:

```text
symbol self-policy = { Meta, Runtime }
body-entry policy = { Meta }
return-object policy = { Meta, Runtime }
```

This block records implementation behavior only; it must not be used to infer
a final `P3` return policy.

The `r === t` body above is an implemented-v0.8 fixture shape, not final formal
meta-return semantics. Final meta construction uses `r = ...` to produce
`SymbolConstructionValue`; ordinary declaration `let ===` aliasing remains
separate. See
`spec/design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md`.

The current `+` overload support is not compiler-intrinsic set union. `+`
remains a source-declared locatable operator/callable symbol, and candidate
sets come from the selected namespace graph view.

---

## 1. Scope

This document is the formal specification of overload resolution for the lang
language. It defines:

- how overload candidate sets are constructed from namespace graph children
- how visibility and export rules gate internal vs external lookup
- how callable-object `P1` filters lookup-stage visibility
- how entry `P2` and argument total policy form the qualified candidate set
- the extraction-pattern specificity rule as a stable lexicographic rank
- the full overload resolution pipeline from raw symbol lookup to uniqueness
- how `must_select_if_qualified` constrains the final result without closing an
  entire overload name
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

### 2.1 Lookup-stage separation

After a callee path has resolved to `Symbol`, callable-object lookup is **not**
a single pass that merges both external stages. The final external object
lookup stage is:

```text
LookupStage ::= compile | runtime
```

A `Val2` callable object from the resolved symbol's heterogeneous value facet
enters the stage-visible candidate set only when:

```text
current_lookup_stage in P1(candidate)
```

`compile` and `meta` remain distinct `P2` execution capabilities, but both have
external stage `compile`:

```text
external(compile) = compile
external(meta)    = compile
external(runtime) = runtime
```

The current Rust implementation instead exposes `CompileEval`, `MetaAction`,
and `RuntimeBinding`-shaped query paths over `PolicyFlag` sets. Those are
transitional resolver mechanics, not a third external flow. Compile-flow
projection may preserve unresolved compile and meta call families; normal
compile evaluation later distinguishes their capabilities.

`P1` does not control the preceding path-to-`Symbol` resolution. One symbol may
hold heterogeneous objects with different `P1` sets, each filtered separately.

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

### 2.3 Candidate preparation and qualification

The final candidate source is symbol-first:

```text
resolve callee path
  -> Symbol
  -> project heterogeneous value facet
  -> enumerate heterogeneous Val2 objects
  -> filter each object by P1 for the current lookup stage
  -> obtain each value's type
  -> resolve its type-associated `()` entry
  -> discard non-callable entries
```

The current same-name namespace bucket is only a restricted precursor to this
flow. Derived compile companions are inserted as first-class entries during
candidate preparation, not after ordinary overload resolution fails.

During compile projection, the call remains an `UnresolvedCallFamily`. Runtime
origin candidates are projected as a `DerivedCompanionCallFamily`; no concrete
origin entry is selected at that point. Normal compile evaluation enumerates
the family and gives each derived entry its stable
`DerivedCallableEntryId(origin_runtime_entry, CompileCompanion)` before forming
`Q`. The family objects are defined canonically in
`../symbol-world/symbol-policy-and-compile-flow-projection.md` §4.1.

```text
C0 = EnumerateValueObjects(Symbol)
C1 = VisibleObjects(C0, V)
C2 = UsableByP1(C1, current_lookup_stage)
C3 = AssociatedCallEntryAndShapeMatch(C2, E)
Q  = P2Qualified(C3, current_lookup_stage, arguments)
```

- `C0`: heterogeneous value/`Val2` objects enumerated from the already-resolved
  callee symbol.
- `C1`: filtered by object-level visibility view (internal or external).
- `C2`: filtered independently by each object's `P1` lookup-stage policy.
- `C3`: objects whose type-associated `()` entry exists and is structurally
  applicable to the call.
- `Q`: entries whose `P2` has the demanded external stage and whose arguments
  all have exactly that total policy.

---

## 3. P1 Visibility and P2 Qualification

The callable shape is:

```text
P1 let F = (...): P2 -> let r => { ... }
```

### 3.1 P1 lookup visibility

A callable object is usable at C2 iff:

```text
usable(c, lookup_stage) := lookup_stage in P1(c)
```

`P1` currently admits `compile`, `runtime`, or `compile | runtime`. It says
whether this already-enumerated object participates in the current external
lookup stage; it does not govern base symbol resolution, argument policy, or
body execution.

### 3.2 P2 qualified-candidate boundary

After structural applicability, an entry belongs to `Q` exactly when:

```text
external(P2(c)) = current_lookup_stage

and

for every argument a:
  total_policy(a) = external(P2(c))
```

`P2` currently admits `compile`, `meta`, or `runtime`; it does not admit
`runtime | compile`. `compile` computes static values and `PatternValue`, while
`meta` constructs `SymbolConstructionValue` in a `MetaConstructionUnit`.
Their shared external stage does not erase that capability distinction.

If the overload design applies an entry preference such as:

```text
compile > meta > runtime
```

that preference is an ordinary linear filter over `Q`; it is not the meaning of
`P2`, and it cannot rescue an unqualified candidate.

### 3.3 Derived entries and must-select

A mechanically derived compile companion has a stable identity and origin:

```text
DerivedCallableEntryId {
  origin_runtime_entry,
  derivation_kind = CompileCompanion
}
```

It participates in `C0`, qualification, and every ordinary linear filter. It
also carries `must_select_if_qualified`. This is not a hidden fallback and is
not a normal overload that a more specific candidate may silently replace.

The same postcondition is available to future user declarations through a
conceptual `@must_select_if_qualified` notation. It permits non-overlapping
same-name overloads. A future `sealed_overload_name` or `closed_overload_set`
would instead forbid any additional same-name entry and is a separate feature.
The `@...` spelling is not a lexer/parser/AST commitment; an initial
implementation may use compiler-known metadata or another internal semantic
marker, as required by the canonical policy note.

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
C0  = EnumerateValueObjects(Symbol)
C1  = VisibleObjects(C0, V)                 -- V ∈ {Internal, External}
C2  = UsableByP1(C1, lookup_stage)
C3  = AssociatedCallEntryAndShapeMatch(C2, E)
Q   = P2Qualified(C3, lookup_stage, args)    -- exact argument total policy
C4  = MaxEntryPreference(Q)                  -- ordinary linear filter, if configured
C5  = ConceptLegal(C4, E)                   -- remove concept-violating candidates
C6  = MaxConceptOrder(C5, E)                -- keep maximal under concept poset
C7  = MaxExtractionSpecificity(C6, E)       -- lexicographic specificity (§4)
C8  = PreferFirstOrderOverInstantiated(C7)  -- first-order before instantiated
C9  = LifetimePreSatisfied(C8)              -- remove candidates failing lifetime pre
C10 = MaxLifetimeSpecificity(C9)            -- origin-path extraction specificity

E_must = { e in Q | e has must_select_if_qualified }

if E_must = empty:
  select the unique element of C10

if E_must = {e}:
  succeed only when C10 = {e}

if |E_must| > 1:
  error: inconsistent qualified must-select entries
```

### 5.2 Pipeline invariants

Each step `Ci+1 = f(Ci, ...)` satisfies:

```text
Ci+1 ⊆ Ci                              -- monotonic filtering
f is side-effect-free                  -- no observable effects
f is independent of candidate order    -- same result regardless of iteration order
```

The named filters execute in exactly the normative `C4` through `C10` order
shown in §5.1. Candidate iteration and source declaration order do not affect
the result of an individual filter, but the filters are not assumed to commute
with one another.

### 5.3 C0: heterogeneous value objects

After path resolution has produced the callee `Symbol`, `C0` enumerates its
heterogeneous value/`Val2` objects. These objects may have unrelated types and
different `P1` sets. The final model does not treat same-name namespace
children as already-formed callable overloads.

The current implementation's restricted same-name child bucket may still
pre-filter by:

- **name**: same textual name (or operator-identity equivalence)
- **role**: object-role symbols for callable targets; namespace-subspace for
  namespace-qualified lookup
- **arity**: compatible argument count (exact, variadic, or defaulted)
- **syntactic callable shape**: operator identity (`spelling + fixity + arity`)
  for operator calls, ordinary name for ordinary calls

### 5.4 C1–C2: Visibility and P1

Defined in §2 and §3 respectively.

**Ordering constraint**: export visibility (C1) precedes `P1` (C2). If a
symbol is not visible in the lookup view, its policy is not checked.

### 5.5 C3–Q–C4: Matching, qualification, then preference

**Associated-call and structural matching (C3)** precedes policy qualification.
For every object surviving `P1`, obtain its type, resolve its type-associated
`()` entry, discard non-callable objects, and match that entry against `E`. A candidate cannot
win on entry preference if its pattern or type signature does not match the
call operand.

`AssociatedCallEntryAndShapeMatch(C2, E)` removes candidates whose:

- extraction pattern is structurally inapplicable to `E`
- type signature is incompatible with the argument types

**P2 qualification (Q)** then requires the entry's external stage to equal the
current lookup stage and every argument symbol's total policy to equal that
same stage. Merely having compile-time type metadata for a runtime symbol does
not make the original runtime argument a compile-policy argument; projection
uses a distinct projected symbol value.

**Entry preference (C4)** is a normal linear filter over `Q`. Where the existing
ordering `compile > meta > runtime` is retained, it compares only already
qualified entries. It does not define `P2` and does not run before exact
argument-policy qualification.

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

First compute the ordinary final survivor set `C10`. Then compute the
must-select subset from the qualified set, not from the same-name or merely
visible set:

```text
E_must = {
  e in Q
  |
  e has must_select_if_qualified
}

E_must = empty:
  |C10| = 1 -> select the unique candidate
  |C10| = 0 -> error: no matching overload
  |C10| > 1 -> error: ambiguous overload

E_must = {e}:
  C10 = {e} -> select e
  otherwise -> error: qualified must-select entry was not uniquely selected

|E_must| > 1:
  error: multiple qualified must-select entries
```

Qualification is the boundary for "participates in this call." A same-name
entry in `C0` that fails visibility, structure, `P2`, or exact argument policy
does not enter `E_must`.

This rule makes a qualified derived compile companion non-overridable by
ordinary specificity. If another candidate wins the linear filters, the call
fails rather than silently choosing that candidate. If two runtime entries
project to two simultaneously qualified companions, the call also fails: the
runtime overload family has no unique compile projection.

Neither zero nor multiple final candidates are otherwise acceptable. There is
no declaration-order fallback; written order is diagnostic presentation only.

---

## 6. Judgment Form

```text
Γ; V; lookup_stage ⊢ name(args) ⇓ f

where:
  Γ  = namespace graph + type / concept / lifetime environment
  V  = lookup visibility view (Internal | External)
  lookup_stage = compile | runtime
  E  = normalized extraction tree of the call operand name(args)
  f  = the selected unique overload candidate
```

The judgment reads: in environment `Γ`, under visibility view `V` and lookup
stage `lookup_stage`, the call `name(args)` with extraction tree `E` resolves
to overload candidate `f`.

Derivation:

```text
CalleeSymbol = ResolveSymbol(Γ, name)
C0  = EnumerateValueObjects(CalleeSymbol)
C1  = VisibleObjects(C0, V)
C2  = UsableByP1(C1, lookup_stage)
C3  = AssociatedCallEntryAndShapeMatch(C2, E)
Q   = P2Qualified(C3, lookup_stage, args)
C4  = MaxEntryPreference(Q)
C5  = ConceptLegal(C4, E)
C6  = MaxConceptOrder(C5, E)
C7  = MaxExtractionSpecificity(C6, E)
C8  = PreferFirstOrderOverInstantiated(C7)
C9  = LifetimePreSatisfied(C8)
C10 = MaxLifetimeSpecificity(C9)
E_must = MustSelectMembers(Q)

OrdinaryUnique(C10, f)
MustSelectConsistent(E_must, C10, f)
────────────────────────────────────
  Γ; V; lookup_stage ⊢ name(args) ⇓ f
```

---

## 7. Relationship to v0.7-prep Implementation

The v0.7-prep work (PR #56) provides a transitional symbol-policy filtering
layer that approximates part of future C2:

- `PolicyFlag::Export`, `PolicyFlag::Meta`, `PolicyFlag::Runtime` — symbol policy
  flags carried on `PolicyMetadata.policy_set`
- `PolicySet` — bit-set of flags
- `PolicyEnv::Meta` — the current early/meta query environment
- `ResolverCode` — miss vs ambiguity discriminator

These are used in early-meta expansion (`try_expand_early_meta_initializer`)
which performs a per-policy-pass lookup of meta-function targets. This is
current substrate for, but not a complete implementation of, final C2
`UsableByP1(C1, compile)`.

The later restricted v0.8 path implements bounded structural applicability,
meta body-entry checking, and extraction specificity for selected source
callables. It does not implement exact `P2` argument-total-policy
qualification, compile companion derivation, `must_select_if_qualified`, or the
complete C5–C10 pipeline.

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
compile companion derivation
must_select_if_qualified enforcement
explicit @companion_of replacement
explicit @no_compile_companion suppression
closed overload-name declarations
```

The `@...` spellings above are conceptual semantic notation only. They are not
parser or AST implementation prerequisites.

---

## 9. Relationship to Other Documents

| Document | Relationship |
|---|---|
| `static-pattern-spaces-and-extraction-chains.md` §12 | Summary overview of overload resolution; this document is the formal specification |
| `pattern-normalization-and-first-order-overload.md` | Earlier, narrower candidate-preparation subset (pattern normalization + first-order type-value candidate model) feeding meta object invocation; not full runtime overload resolution |
| `mechanical-argument-passing-and-move-fixed-point.md` | Pass-mode adaptation (move/ref/share/copy/in) is separate from type/rank compatibility; `move` does not create a new type value |
| `call-modes-recursion-and-tail-lowering.md` | Candidate selection feeds invocation lowering, which may eventually produce explicit call modes (`normal` / `tco` / `loop`) |
| `../policy-capability/policy-visibility-symbols.md` | Implementation mapping for current policy metadata |
| `../symbol-world/symbol-policy-and-compile-flow-projection.md` | Canonical `P1` / `P2`, companion, and must-select semantics |
| `early-meta-functions-and-namespace-graph.md` | Namespace graph resolves the callee `Symbol`; the current same-name child bucket is only transitional candidate substrate |
| `entity-ref-design.md` | Entity references may resolve through overload candidate sets in later phases |
| `glossary.md` | Defines OverloadCandidate, OverloadSpecificity, OverloadResolutionPipeline |
| `roadmap.md` | v0.8 non-goal, v0.10+ gating phase |
