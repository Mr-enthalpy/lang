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
- a restricted extraction-pattern specificity prototype using the tuple in §4;
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
- concept legality or concept ordering;
- first-order instantiation preference;
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
exact total policy for implicit self plus explicit arguments by `P2`, and there
is no independent `P3`. The
current return-object field is provisional transport until layer-directed
result projection exists.

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

- how overload candidate sets are constructed from a resolved Symbol's `Val2`
  objects
- how visibility and export rules gate internal vs external lookup
- how callable-object `P1` filters lookup-stage visibility
- how hard legality, including `P2` and complete invocation-frame policy, forms
  the fully admissible candidate set `A`
- the extraction-pattern specificity rule as a stable lexicographic rank
- the full overload resolution pipeline from raw symbol lookup to uniqueness
- how `must_select_if_qualified` activates from `A` and constrains the final
  result without closing an
  entire overload name
- a compact judgment form

This document does **not** define:

- concept semantics (only the interface: `concept_projection` must produce a
  stable poset element)
- lifetime checking or lifetime-driven refinement; see
  `../lifetime/lifetime-policy-and-overload-boundary.md`
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
projection preserves ordinary calls; normal compile evaluation later
distinguishes direct compile/meta objects and derived companion objects.

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

Path traversal is always built from the visibility-filtered graph view, never
from raw graph children directly. Once it resolves the callee `Symbol`, object
candidate construction proceeds from that symbol's value facet.

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
flow. A derived compile companion is a complete `Val2` function object with its
own type and associated compile `()`. It is inserted into the symbol value
facet before ordinary candidate preparation, not after overload failure.

```text
C0 = EnumerateValueObjects(Symbol)
C1 = VisibleObjects(C0, V)
C2 = UsableByP1(C1, current_lookup_stage)
C3 = AssociatedCallEntryAndShapeMatch(C2, E)
A  = FullyAdmissible(C3, current_lookup_stage, invocation_frame, expectation)
```

- `C0`: heterogeneous value/`Val2` objects enumerated from the already-resolved
  callee symbol.
- `C1`: filtered by object-level visibility view (internal or external).
- `C2`: filtered independently by each object's `P1` lookup-stage policy.
- `C3`: objects whose type-associated `()` entry exists and is structurally
  applicable to the call.
- `A`: candidates that satisfy every hard precondition, including `P2`, exact
  total policy for implicit self and explicit arguments, expected result
  rank/facet, concept/ordinary-require legality, and other compile/type checks.

---

## 3. P1 Visibility and P2 Admissibility

The callable shape is:

```text
P1 let F = (...): P2 -> let r => { ... }
```

### 3.1 P1 lookup visibility

A callable object is usable at C2 iff:

```text
usable(c, lookup_stage) := lookup_stage in P1(c)
```

`P1` currently admits `compile` or `compile | runtime`. A callable function
object is always compile-visible; runtime-only `P1` is invalid. `P1` says
whether this already-enumerated object participates in the current external
lookup stage; it does not govern base symbol resolution, argument policy, or
body execution.

### 3.2 P2 fully admissible boundary

After structural applicability, a candidate can enter `A` only when:

```text
external(P2(c)) = current_lookup_stage

and

InvocationFrame(c):
  slot 0 = implicit self
  slot 1..n = explicit source arguments

for every frame slot a:
  total_policy(a) = external(P2(c))
```

`P2` currently admits `compile`, `meta`, or `runtime`; it does not admit
`runtime | compile`. `compile` computes static values and `PatternValue`, while
`meta` constructs `SymbolConstructionValue` in a `MetaConstructionUnit`.
Their shared external stage does not erase that capability distinction. The
slot-0 self view uses the same `P2` stage requirement; no separate self-policy
plane is introduced.

If the overload design applies an entry preference such as:

```text
compile > meta > runtime
```

that preference is an ordinary linear filter over `A`; it is not the meaning of
`P2`, and it cannot rescue a hard-inadmissible candidate.

### 3.3 Derived objects and must-select

A mechanically derived compile companion is a complete function object:

```text
DerivedCompileCompanionObject {
  object_id,
  origin_runtime_object_id,
  function_object_type,
  associated_namespace,
  associated_call_entry,
  overload_strategy = must_select_if_qualified,
  provenance,
}
```

It participates in `C0`, hard admissibility, and every ordinary preference
filter. Preparing its associated `()` propagates the object strategy into the
candidate. Must-select is not a hidden fallback, infinite priority, or a rule
that closes the entire overload name. Candidate source spelling for the
strategy is deliberately unresolved; `@` is not available as a generic
annotation prefix because it belongs to lifetime-policy operations.

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
A   = FullyAdmissible(
        C3,
        lookup_stage,
        invocation_frame,
        expected_result,
        compile_type_requirements
      )

B1  = MaxEntryPreference(A)
B2  = MaxConceptOrder(B1, E)
B3  = MaxExtractionSpecificity(B2, E)
B4  = PreferFirstOrderOverInstantiated(B3)

M = {
  c in A
  |
  overload_strategy(c) = must_select_if_qualified
}
```

### 5.2 Pipeline invariants

Every `Bi+1 = f(Bi, ...)` preference step satisfies:

```text
B1 ⊆ A and Bi+1 ⊆ Bi                  -- monotonic filtering
f is side-effect-free                  -- no observable effects
f is independent of candidate order    -- same result regardless of iteration order
```

The named filters execute in exactly the normative `B1` through `B4` order.
Candidate iteration and source declaration order do not affect an individual
filter, but filters are not assumed to commute.

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

### 5.5 C3: Associated call entry and basic applicability

Associated-call and structural matching precedes full admissibility.
For every object surviving `P1`, obtain its type, resolve its type-associated
`()` entry, discard non-callable objects, and match that entry against `E`. A
candidate cannot win on preference if its pattern or type signature does not
match the call operand.

`AssociatedCallEntryAndShapeMatch(C2, E)` removes candidates whose:

- extraction pattern is structurally inapplicable to `E`
- type signature is incompatible with the argument types

### 5.6 A: Fully admissible candidates

`A` removes every hard-illegal candidate before any preference filter runs.
Hard admissibility includes, at minimum:

```text
path and object visibility
P1 admission for the current lookup stage
existence of associated ()
parameter count and structural shape
Pattern/extraction applicability
P2 equality with the call stage
total_policy(slot) = external(P2) for implicit self and every explicit argument
expected result rank/facet compatibility
concept and ordinary require legality
other compile/type-stage hard preconditions
```

Merely having compile-time type metadata for a runtime symbol does not make the
original runtime value a compile-policy argument; compile projection supplies
the Pattern projection as a distinct argument view.

Concept legality belongs here. A concept-violating candidate never reaches a
preference filter. Full concept semantics remain deferred, but the legality
boundary is fixed.

Lifetime policy does not belong to `A`. Lifetime checking/refinement occurs
after type/compile overload selection and first-order instantiation, as bounded
by `../lifetime/lifetime-policy-and-overload-boundary.md`.

### 5.7 B1–B4: Preference filters

Only fully admissible candidates enter preference filtering:

- **B1 Entry preference**: apply any configured entry-policy preference.
- **B2 Concept ordering**: keep maximal legal candidates under the future
  concept-order poset.
- **B3 Extraction specificity**: apply the lexicographic rank from §4.
- **B4 First-order preference**: if otherwise tied, prefer a first-order object
  over an instantiated object.

Each stage only removes candidates. First-order preference does not override
extraction specificity; a deeper applicable generic pattern may outrank a
shallower monomorphic pattern before B4 is reached.

### 5.8 Must-select consistency and uniqueness

Compute must-select membership from `A`, not from `C0`, `C2`, `C3`, or an
earlier set that has not passed concept/require legality:

```text
M = {
  c in A
  |
  overload_strategy(c) = must_select_if_qualified
}

M is empty:
  |B4| = 1 -> select the unique candidate
  |B4| = 0 -> error: no matching overload
  |B4| > 1 -> error: ambiguous overload

M = {m}:
  B4 = {m} -> select m
  otherwise -> error: admissible must-select object was not uniquely selected

|M| > 1:
  error: multiple admissible must-select objects
```

Full admissibility is the boundary for “participates in this call.” A same-name
object that fails any hard check does not activate must-select.

This rule makes an admissible derived compile companion non-overridable by
ordinary specificity. If another candidate wins the linear filters, the call
fails rather than silently choosing that candidate. If two runtime entries
produce two simultaneously admissible companions, the call also fails: the
runtime overload family has no unique compile projection.

Neither zero nor multiple final candidates are otherwise acceptable. There is
no declaration-order fallback; written order is diagnostic presentation only.

---

## 6. Judgment Form

```text
Γ; V; lookup_stage ⊢ name(args) ⇓ f

where:
  Γ  = namespace graph + type / concept environment
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
A   = FullyAdmissible(C3, lookup_stage, invocation_frame, expected_result, Γ)
B1  = MaxEntryPreference(A)
B2  = MaxConceptOrder(B1, E)
B3  = MaxExtractionSpecificity(B2, E)
B4  = PreferFirstOrderOverInstantiated(B3)
M   = MustSelectMembers(A)

OrdinaryUnique(B4, f)
MustSelectConsistent(M, B4, f)
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
callables. It does not implement exact `P2` invocation-frame policy
admissibility, compile companion derivation, `must_select_if_qualified`, or the
complete fully-admissible/preference pipeline.

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
operator identity disambiguation (spelling + fixity + arity is presumed)
implicit discard as a candidate-selection mechanism
declaration-order fallback
compile companion derivation
must_select_if_qualified enforcement
explicit companion_of replacement mechanism
whether default companion suppression is permitted
closed overload-name declarations
```

Lifetime checking and lifetime-driven refinement are separately deferred by
`../lifetime/lifetime-policy-and-overload-boundary.md`; they are not stages in
this type/compile overload pipeline.

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
| `../lifetime/lifetime-policy-and-overload-boundary.md` | Negative boundary separating lifetime policy/refinement from this type/compile pipeline |
| `early-meta-functions-and-namespace-graph.md` | Namespace graph resolves the callee `Symbol`; the current same-name child bucket is only transitional candidate substrate |
| `entity-ref-design.md` | Entity references may resolve through overload candidate sets in later phases |
| `glossary.md` | Defines OverloadCandidate, OverloadSpecificity, OverloadResolutionPipeline |
| `roadmap.md` | v0.8 non-goal, v0.10+ gating phase |
