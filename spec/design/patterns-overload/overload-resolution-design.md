# Overload Resolution Design

**Status: Mixed.** Full overload resolution remains non-normative future
design. v0.8 implements only the restricted source-declared meta-overload
selection slice described in §0.1.

Pattern satisfaction, proof-relevant observation/extraction, direct structural
incidence, binderless semantics, and `Norm_P` soundness are canonical in
`pattern-values-relational-semantics-and-extraction.md`. This document consumes
their applicability evidence; it does not define another Pattern calculus.

This document remains the broader overload-resolution design. The earlier
pattern/type candidate-preparation subset used by future meta object invocation
is documented in `pattern-normalization-and-first-order-overload.md`. That
subset is the narrower candidate model — argument/parameter shapes, applicability,
and a constrained specificity ordering — that the meta invocation engine needs;
it is **not** equivalent to full runtime overload resolution, which this document
continues to specify.

The canonical definitions of `Pv:Pp`, binding P1, result P2,
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
- `PolicySpecAst` / `NormPolicySpec` preservation of single and pair policy;
- P1 declaration-prefix projection over the function-object stage view derived
  from P2;
- `: meta ->` normalization to canonical `meta:meta` and current flat meta
  compatibility transport;
- transitional return-object stage transport derived from the P2 result pair;
- C0 from selected namespace graph children by name, role, arity, and
  source-callable shape;
- transitional self-policy filtering for `MetaAction` and metadata for
  `RuntimeBinding`;
- restricted parameter extraction-pattern applicability;
- current body-entry eligibility for demanded meta execution;
- a restricted extraction-pattern specificity prototype using the tuple in §4;
- callable-tail strategy metadata, default/delete implementation variants, and
  restricted remainder-pack applicability;
- unique selection or hard ambiguity diagnostics;
- selected delete-body diagnostics and the current legacy `r === x`
  forwarding-body substrate.
- a connected `PreparedCallCandidate` path shared by source-Symbol calls and
  Pattern-owner-associated calls;
- one Bp' product comparison over the implemented ordinary formal/phase
  coordinates plus optional atomic-migration input/output endpoint fit;
- the retained caller-supplied transition fixture as algebra-only evidence,
  never as a second resolver or source-language callable family.

This implemented C0 bucket is transitional. Final call preparation resolves one
Symbol, forms `CallableProjection(S) = DedupCandidateIdentity(V_S ⊎ V_τ)`
(symbol-first §2.1), observes each projected candidate's policy-projected view,
obtains each surviving value's type, resolves each
type-associated `()` entry, discards non-callable entries, and then performs
parameter-pair, stage, P2-result, and applicability filtering to a unique
maximal candidate. Same-name
value entries are not assumed to be same-type function overloads. See
`spec/design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md`.

Explicitly not implemented in v0.8:

- full runtime overload resolution;
- concept legality or concept ordering;
- first-order instantiation preference;
- in-place closure materialization, lazy embedding lookup, or the B5
  in-place-over-non-in-place preference;
- ADL, unrestricted lookup, or global search for all symbols of a name
  (only the discovery algorithm; the operator dispatch identity boundary is
  closed in §1);
- D/Done reduction or control-flow pattern transformation;
- guarded branch invocation, short-circuit invocation, or full meta block
  interpretation.

Declaration order is not a semantic tiebreaker. When implemented stages leave
two equal maximal candidates, selection reports ambiguity.

`meta || runtime` is a policy expression in declaration-policy context. Its
`||` is same-dimension stage choice. It is not Pattern alternative, not
expression-level operator lookup, and not evidence that the body may execute
under runtime policy. Legacy-substrate pattern-side forms such as
`_ if | else: type` are parsed and interpreted in parameter-pattern context
only.

The implementation currently stores three metadata fields:

```text
symbol policy metadata
body-entry policy metadata
return-object policy metadata
```

They are not three final source-level policy positions. In the final model,
base path resolution produces `Symbol` before policy-view filtering; each
projected candidate carries its own `Pv:Pp`, P2 describes the call result
pair and derives the function-object stage view, and there is no independent
`P3`. The
current return-object field is provisional transport until canonical component
policy storage exists.

For:

```lang
meta || runtime let + =
  (self, t: type, u: type): meta -> let r: type =>
{
  r === t;
};
```

the current pair-aware adapter elaborates:

```text
P2 = meta:meta
derived function-object stage view = meta:meta
written P1 query meta||runtime selects the available meta slice
flat compatibility metadata = { Meta }
```

This block records implementation behavior only; it must not be used to infer
a final `P3` return policy.

The `r === t` body above is an implemented-v0.8 fixture shape, not final formal
meta-return semantics. Target semantics use the default `τ` result (`DefaultMetaResult = τ`): `let`
creates members, `=` writes existing places, and the return event transfers
control. The current `let r`/`r =`/`r;` cluster behavior is a transitional
compatibility encoding, and `SymbolConstruction` is its carrier rather than a
result class. There is no alias-member event. See
`spec/design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md`.

The current `+` overload support is not compiler-intrinsic set union. `+`
remains a source-declared locatable operator/callable symbol, and candidate
sets come from the selected namespace graph view.

---

## 1. Scope

This document is the formal specification of overload resolution for the lang
language. It defines:

- how overload candidate sets are constructed from
  `CallableProjection(S) = DedupCandidateIdentity(V_S ⊎ V_τ)`
- how visibility and export rules gate internal vs external lookup
- how each Val2 object's policy-projected view enters candidate preparation
- how hard legality, including declared receiver/parameter pair compatibility
  for the complete invocation frame and P2 compatibility with a target-result
  expectation, forms the fully admissible candidate set `A`
- the extraction-pattern specificity rule as a stable lexicographic rank
- the three-point PolicyMode preference relation as a separate product partial order
- the full overload resolution pipeline from raw symbol lookup to uniqueness
- how `must_select_if_qualified` activates from `A` and constrains the final
  result without closing an
  entire overload name
- how source-named overload strategies attach to a callable implementation and
  operate only after the fully admissible set exists
- a compact judgment form

**Operator dispatch identity boundary (closed).** The identity layer that maps
an operator spelling/syntax to a dispatch entrance is frozen by this design
(canonical phrasing: `spec/reference/glossary.md`, "Naked operator" / "ADL" /
"Assignment operator"; assignment special case:
`symbol-first-meta-construction-and-pattern-injection.md` §4.5.1):

```text
OperatorDispatchIdentity:
  NakedOperator(op)    -> operator[OperatorIdentity(op)]
  DotOperator(.op)     -> op::adl
  ExplicitPath(P::op)  -> P::op

operator[=]   -> .=
.=            ≡ =::adl
```

This boundary is normative here: the three entrances are orthogonal, an
explicit path never runs ADL or operator forwarding, and "global operator"
and "ADL" are not synonymous. What remains deferred is not the identity
boundary but only the *general ADL associated-scope discovery / candidate
enumeration algorithm* behind `op::adl`.

This document does **not** define:

- concept semantics (only the interface: `concept_projection` must produce a
  stable poset element)
- lifetime checking or any lifetime-driven second selection; see
  `../lifetime/lifetime-policy-and-overload-boundary.md`
- general ADL associated-scope discovery / candidate enumeration, or
  unrestricted symbol search (the operator dispatch *identity* boundary is
  closed above; only the ADL discovery algorithm behind `op::adl` is deferred)
- implicit type conversion or coercion
- implicit borrow formation (`T` is never repaired to `T ref` / `T share`)
- partial-application overloads
- package-internal symbol aliases as overload candidates

---

## 2. Overload Set Construction

### 2.1 Phase-slice separation

After a callee path has resolved to `Symbol`, candidate preparation observes a
specific execution phase:

```text
Phase ::= OpenStatic | SealStatic | Runtime
```

A `Val2` object enters C2 only through the view exposed in that domain by its
`Pv:Pp` pair. The canonical domains include:

```text
Vis(meta)    = { OpenStatic }
Vis(seal)    = { SealStatic }
Vis(compile) = { OpenStatic, SealStatic }
Vis(runtime) = { Runtime }
```

The compatibility resolver projects those phases onto flat `PolicyFlag` sets;
that lossy transport cannot define the canonical pair algebra. Compile-flow
projection preserves ordinary calls; the static evaluator distinguishes direct
compile/meta/seal objects and derived companion objects.

Policy-view selection does not control the preceding path-to-`Symbol`
resolution. One symbol may hold heterogeneous objects with different pairs,
each viewed separately.

### 2.2 Visibility and export

Overload candidate construction begins from namespace graph children, filtered
by a visibility view `V`:

```text
V ::= Internal | External
```

```text
Visible(C, Internal) = C
Visible(C, External)
  = { c ∈ C | Exported(c.path) && PubliclyReachable(c.path) }
```

- **Internal** lookup: all children of the current namespace are candidates.
  `export` is irrelevant.
- **External** lookup: the path must be in the export-retention closure and every path
  segment must pass ordinary public/private reachability. Export-root and
  visibility are independent dimensions.

Path traversal is always built from the visibility-filtered graph view, never
from raw graph children directly. Once it resolves the callee `Symbol`, object
candidate construction proceeds from that Symbol's callable projection
`CallableProjection(Σ) = DedupCandidateIdentity(V_S ⊎ V_τ)`, where `V_τ = CallSpace(τ)` is the intrinsic
callspace of the embedded closure and `V_S` is the Symbol's own sibling
candidate space.

Candidate construction is closed over the namespace-graph view selected for the
query: it performs no ADL-like expansion and no external scope search, external
users cannot retroactively add candidates to an already-visible namespace node,
and meta-generated injection occurs only through explicit parent-to-child
namespace-delta boundaries.

### 2.3 Candidate preparation and qualification

The final candidate source is symbol-first:

```text
ResolveSymbol(callee path)
  -> Symbol Σ
  -> form CallableProjection(Σ) = DedupCandidateIdentity(V_S ⊎ V_τ)   (unified entrance)
  -> apply call-site candidate-family filter to the projection
  -> enumerate candidates from CallableProjection(Σ)
  -> observe each object's policy-projected view for the lookup stage
  -> obtain each value's type
  -> resolve its type-associated `()` entry
  -> discard non-callable entries
```

The current same-name namespace bucket is only a restricted precursor to this
flow. A derived compile companion is a complete `Val2` function object with its
own type and associated compile `()`. It is derived from the callable under
the compile transform (`CompilePartner(F) = C(F)`, canonical in
`function-object-call-model.md` §8); the symbol-facet entry used at lowering is
an implementation cache, not the semantic cause, and the companion never
appears by that entry alone.

```text
C0 = CallSiteFamilyView(CallableProjection(Σ), call_site_annotation)
C1 = VisibleObjects(C0, V)
C2 = ExposePhaseViews(C1, Phase)
C3 = AssociatedCallEntryAndShapeMatch(C2, E)
A  = FullyAdmissible(C3, Phase, invocation_frame, expectation)
D  = DeclarationCandidatePolicy(A)
```

The call-site annotation acts **before** candidate generation. Its only closed
semantics in this PR is its pipeline position: it limits which declared or
generated candidate families may contribute to `C0` for this call. A future
surface may resemble `args |[[annotation]]> callee` (for example default-only or
nongeneric-only), but spelling and general selector algebra remain deferred.
With no annotation, the family view is the complete typed `V` member family.

The two annotation phases are distinct semantic stages:

```text
|[[α]]>
    CallSiteFamilyFilter
    before C0

=>[[δ]]
    DeclarationCandidatePolicy
    after FullyAdmissible A
```

`|[[α]]>` only decides which candidate families may enter generation for this
call. It is not a priority: it cannot make an invisible, ill-typed,
policy-illegal, or lifetime-illegal candidate legal, and it does not select
among candidates inside a family. `=>[[δ]]` is not a candidate generator: it
cannot resurrect a candidate already excluded at C0..A.

The generated structural default family has a stable identity used by the
language itself (canonical owner
`pattern-values-relational-semantics-and-extraction.md` §3.1,
`AtomicExtract_P`). P-internal extraction applies it implicitly before C0;
ordinary user `x.field` does not. The identity is part of the language
interpretation of P and is never stored as Pattern data.

Custom/default recursion example:

```text
custom getter + generated default =>[[fallback]]

external ordinary call x.field:
    A = {custom, default}
    default retreats (fallback suppressed while custom survives)

inside the custom body:
    ... |[[default]]> ...
    removes the custom family from C0
    at the fallback stage only default remains
    => no recursion
```

Declaration-side annotations act only after `A` has been formed. They control
how already-fully-admissible candidates participate — for example fallback
suppression or must-select consistency. They never generate a candidate or make
an inadmissible candidate viable.

- `C0`: `CallableProjection(Symbol) = DedupCandidateIdentity(V_S ⊎ V_τ)` — heterogeneous value/`Val2`
  objects (symbol-first §2.1; never a `V_S`-only projection); `V_τ` is the
  intrinsic callspace of the carried closure, not recovered from the Symbol.
- `C1`: filtered by object-level visibility view (internal or external).
- `C2`: filtered independently by each object's available policy-pair view.
- `C3`: objects whose type-associated `()` entry exists and is structurally
  applicable to the call.
- `A`: candidates that satisfy every hard precondition, including P2 pair
  compatibility for implicit self and explicit arguments, expected result
  pair/rank/facet, concept/ordinary-require legality, and other compile/type
  checks.

---

## 3. Policy Views and P2 Result Admissibility

The callable shape is:

```text
[P1] let F = (...): P2 -> let r => { ... }
```

### 3.1 P1 is binding projection

P1 is not an intrinsic scalar lookup set attached only to callable objects. It
is the optional general binding projection used when the function object is
bound.

The function-object base stage view is derived from P2:

```text
Stage(P1p) = Stage(P2p)
Stage(P1v) = Stage(P2v) union Stage(P2p)
```

Bare ordinary binding forms output demand `PolicyMode=plain` before RHS call
selection, so that coordinate participates in the same Policy product order as
the inputs. After unique selection, omitted P1 retains the complete object pair
view. Single P1 `Q` selects value
slices visible under Q and follows their associated pattern components. Pair P1
`Qv:Qp` filters both. A written P1 cannot manufacture a stage absent from the
derived object.

After path resolution has formed `CallableProjection(S)`, C2 observes the
object view available at the current lookup stage. Base path-to-Symbol
resolution is not conditioned by P1.

P1 elaboration first applies `project_p1` across the complete
`PolicyResultEntry[]`. Any non-empty result is the identity-preserving binding
projection; alternatives named by the query but absent from that result are not
manufactured. The bounded transition prototype can be reached only when the
complete projection is empty, the source has a static value view, and the
query accepts runtime. The original demand may be a choice such as
`meta || runtime`; only after its complete projection fails is the runtime
branch extracted as the runtime-only migration target. A failed query with no
runtime alternative does not authorize arbitrary operation search.

The connected build slice routes an authorized atomic runtime migration from
an existing source `PatternValue` to that owner's associated `()` Val2, then
through the same `PreparedCallCandidate`, `InvocationFrame`, and ordinary
selection path as source callables. The historical
`PolicyTransitionCallable` carrier remains an isolated endpoint-order fixture
and does not participate in this routing.

The connected restricted path still lacks full Pattern applicability,
requires/concepts, and the non-identity B1/B2/B4/B5/B6 implementations. These
are missing dimensions of the one ordinary resolver, not permission to search
`ref`, `share`, `@`, `alias`, or another structure-changing operation after a
Policy failure. `NoImplicitBorrowFormation` applies before and throughout
candidate generation: a candidate that requires a borrow is structurally
applicable only when the actual already contains the explicit borrow
observation (apart from fixed-point/weakening rules on an existing borrow).

### 3.2 P2 pair at the fully admissible boundary

P2 describes the call/expression result pair:

```text
P2 = Pv:Pp
```

Explicit pair validity requires:

```text
runtime not in Stage(Pp)
Static(Pv) is empty or Static(Pv) = Stage(Pp)
```

Single P2 normalizes contextually:

```text
N2(P) = P:(P-runtime), when non-empty
N2(runtime) = runtime:compile
```

After structural applicability, a candidate enters A only when receiver and
parameter pair patterns match the complete invocation frame, its P2 result pair
matches any target-result expectation, and stage, rank/facet, concept, and
ordinary-require hard conditions hold. The frame is:

```text
slot 0 = implicit caller-object actual
         matched by the first written formal, if present
slot 1..n = explicit source arguments
            matched by written formals 1..n
```

No independent self policy plane is introduced. Pair-policy matching of frame
positions is a hard admissibility check, not a preference score; P2 remains the
result pair rather than a substitute parameter policy.

Explicit `runtime:seal` remains valid. `compile`, `meta`, and `seal` remain
distinct static capabilities/domains.
Compile computes any declared ordinary semantic value across result classes
(an ordinary PatternValue, a complete type value `τ`, or a borrow instance).
Ordinary meta computes the
default complete type value `τ_M` of its MetaInstance and additionally
carries the authority to establish and seal that instance. Only an explicitly
declared `symbol` result returns a `symbol`-typed value. Privileged builtins
have member-declared results. Seal
excludes ordinary OpenStatic meta visibility and provides no global scan
privilege by itself.

### 3.3 Derived objects and must-select

A mechanically derived compile companion is a complete Val2 function object:

```text
DerivedCompileCompanionObject {
  object_id,
  origin_runtime_object_id,   -- implementation-transport field only
  function_object_type,
  associated_namespace,
  associated_call_entry,
  overload_strategy = must_select_if_qualified,
  provenance,                 -- implementation-transport field only
}
```

`object_id`/`origin_runtime_object_id`/`provenance` are
implementation-transport fields: they describe the lowering record, not the
semantic source. The companion's existence is `CompilePartner(F) = C(F)`
(function-object-call-model §8); type-as-callee never recovers a defining
Symbol from these fields, and the carrier-independence rule
(function-object-call-model §8) keeps copied/extracted type-as-callee lookup
on the immutable `V_τ` snapshot.

For origin P2 `runtime:Qstatic`, its static result pair is
`Qstatic:Qstatic`. It participates
in C0, hard admissibility, and every ordinary preference filter. Preparing its
associated `()` propagates the object strategy into the candidate.

Must-select is not a hidden fallback, infinite priority, or a rule closing the
entire overload name. User-written strategy metadata uses
`=> strategy_name { ... }`; `[[strategy_name]] { ... }` is the no-`=>`
disambiguation form. `@` remains reserved for lifetime policy operations. See
`callable-tail-dot-closure-and-pack-pattern.md`.

---

## 4. Extraction-Pattern Specificity — Lexicographic Rank

This section's lexicographic tuple applies only to extraction-pattern
specificity. It is not a general candidate fitness score and is not used for
const/mut matching.

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
            pack matcher ...Q
```

```text
D(P, E) = { n ∈ C(P, E) | the corresponding pattern node is
           explicit ordinary discard _ }

M(P, E)  = ordinary non-discard explicit nodes
EP(P, E) = explicit pack-binding nodes
DP(P, E) = pack-discard nodes (..._)
```

The number of remainder elements absorbed never contributes specificity.
Every Pack supplies exactly one outward pack-class evidence node at its
containing structural level:

```text
...a -> one EP
..._ -> one DP
...Q -> one outward Pack position
```

Raw `...(a, b)` cannot supply structured evidence: the bare Product has no
stable top mode after P normalization and is rejected by the normalized
Pattern handoff. If an ordered layer later admits an explicitly headed operand
such as `...((a, b) pair)`, evidence for `pair` and its internal `a`/`b`
structure belongs below the stable head at the next preserved structural
level. It is not flattened into two same-level EP nodes. Unordered named
levels admit only a whole-remainder binder/discard.

### 4.2 Specificity tuple

```text
specificity(P, E) =
  (
    max depth(n)   for n ∈ C(P, E),    -- deepest explicit penetration
    Σ depth(n)     for n ∈ C(P, E),    -- total explicit depth contribution
    |M(P, E)|,                          -- ordinary explicit match count
    |EP(P, E)|,                         -- explicit pack match count
    |D(P, E)|,                          -- ordinary discard count
    |DP(P, E)|                          -- pack discard count
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
3. If total depth is tied, node-class evidence is compared in the order
   `ordinary explicit > explicit pack > ordinary discard > pack discard`.

Discard `_` contributes depth because it asserts the user knows and requires
that structure. At equal depth totals, a Pack never gains specificity from the
number of elements absorbed or from internal operand width at the same level.

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

### 4.5 Policy product partial order

`PolicyMode = {const, plain, mut}` is a whole-slot coordinate orthogonal to
`Pv:Pp`; it is not a subfield of `Pv`. Plain `let` denotes the real `plain`
point, never an unspecified or missing mode. The preference relation is indexed
by the actual/demand context:

```text
succ_const: const > plain > mut
succ_mut:   mut > plain > const
succ_plain: plain > const = mut
```

The equality in `succ_plain` does not authorize an arbitrary tie break. If a
plain context has one fully admissible `const` candidate and one fully
admissible `mut` candidate but no `plain` candidate, both are co-maximal and
the call is ambiguous.

Across the first written self formal, later explicit parameters, and a target
result policy when one is actually supplied, compare candidates by product
order. The self actual is injected rather than taken from the call-site
Product, but its formal PolicyMode restriction participates in the same order.
Candidate `f`
dominates `g` iff `f` is no worse at every compared position and strictly
better at at least one.

Crossed advantages remain incomparable. Phase-local stage specificity joins
this product only after full admissibility: OpenStatic has `meta > compile` and
SealStatic has `seal > compile` when candidates differ only by the narrower
visible domain. It is not a global override of argument dimensions.

There is no total score, exact-match count, parameter weighting,
left-to-right lexicographic fallback,
input-before-output preference, or independent conversion rank.

Delete candidates participate in this same relation. If the unique maximal
candidate is delete, selection reports the matched specific rejection rather
than removing it before comparison.

For a compiler-inserted atomic runtime-migration call, the selected static
source view and requested runtime output view are known at the call site. Their
Policy coordinates conservatively extend Bp:

```text
Bp' = MaxPolicyProduct(
        ordinary Bp coordinates
        x InputEndpointPolicyFit(candidate, SourcePolicy)
        x OutputEndpointPolicyFit(candidate, TargetPolicyQuery)
      )
```

This is one Pareto order, not an input-first/output-second sequence. Better
input with worse output is incomparable with worse input and better output.
Because this is Bp, it runs before B1, B2, and B3 Pattern extraction
specificity. A candidate with better endpoint Policy fit is removed/retained
before a competing extraction-specificity advantage is considered. On an
ordinary call without migration endpoint coordinates:

```text
Bp' = old Bp exactly
```

so the old Bp survivor identities and every later B1..B6 result are unchanged.
No transition-specific B6 named strategy exists.

The compiler mandates the static-to-runtime stage edge, not equality of every
endpoint coordinate. Candidate-declared input/output `PolicyMode` belongs
to this product. Thus a callable may expose:

```text
const + compile -> mut + runtime
```

for a fresh runtime object. The compiler does not synthesize `mut`; the
callable declares it and must win ordinary overload selection. Type remains
unchanged, so this does not reopen structural applicability repair.

Endpoint `PolicyMode` reuses the ordinary actual-relative order above. It is not
a Policy-domain hard intersection:

```text
HardEndpointApplicability
  = Type
  x stage legality
  x presence legality
  x Pp capability
  x structural applicability

InputPolicyModePreference
  = Compare(candidate input Pattern, selected source actual)

OutputPolicyModePreference
  = Compare(candidate output Pattern, requested target)
```

Consequently an opposite const/mut endpoint remains in fully admissible set
`A`; it is merely worse than `plain`, which is worse than the matching
endpoint. For a const source/target the ordering is `const > plain > mut`, and it
reverses for a mut source/target. A plain target uses
`plain > const = mut`, preserving ambiguity when only the tied endpoints survive.
This is the same ordinary Bp relation, not a migration-specific subset order.
If a unique ordinary winner later produces a result that `Project_out` cannot
expose to the consumer, that failure does not reopen overload selection.

One explanatory semantic normal form is a type Symbol with one pure Pattern
facet and ordinary value members implementing a capability realization over
the full 3×3 input/output mode space:

```text
Symbol t
  Pattern:
    :t

  transport coordinate: output PolicyMode <- input PolicyMode
  every one of the nine coordinates is expressible
  each coordinate may be absent or realized by default/delete/custom
```

This is not frozen surface syntax and does not require a conversion table or a
new callable ontology. It illustrates that ordinary members realize
capabilities independently of Policy preference. A `mut` endpoint may be the
unique selected mode while the selected member is `delete`, and `mut` on a
non-reference object does not by itself grant write capability. More specific
structural Pattern members may locally refine or `delete` regions of the 3×3
relation. Each installed member's ordinary formal
and complete result Policy is `(compile || runtime):compile`; the migration
context compares the compile `Project_in` and runtime `Project_out` views. It
does not replace the member's complete P2 with a direct
`compile -> runtime` signature.

Structural safety remains in later ordinary filters. For example a generic
materialization candidate and a more specific `T ref` delete candidate can
declare identical Policy endpoints:

```text
Bp'  -> Policy tie; both survive
B3   -> T ref Pattern is more specific
final unique candidate = delete
```

The specific delete must not advertise a worse endpoint Policy merely to
encode structural danger; doing so would incorrectly remove it in Bp' before
B3. Policy ordering does not know that `ref` is dangerous. Pattern
specialization and delete express that structural case.

Final Bp' must compare all ordinary and migration coordinates together:

```text
Max(Product(old Bp coordinates, input endpoint, output endpoint))
```

Endpoint-only maxima cannot be applied sequentially before or after old Bp:
crossed ordinary/endpoint advantages must remain incomparable.

Output Policy participation here does not make ordinary return type an overload
preference dimension. An optional output-type expectation remains a hard
admissibility check only. Stage, presence, and Pp endpoint capability remain
hard constraints, while output `PolicyMode` remains an actual-relative Bp
preference coordinate. Existing-slice P1 projection is checked first; any
non-empty binding projection creates no transition request.

Only canonical `Pv:Pp` call dimensions participate. Namespace declaration
visibility and export-root metadata are illegal in ordinary formal/result/P1
Policy and are rejected before Bp comparison; they are not specificity
coordinates. Any value-presence specificity used by the current prototype is
local to its endpoint Bp coordinate and does not define a general Policy
overload order.

When an outer ordinary candidate requires the authorized migration, typed
migration qualification is part of forming that outer candidate's fully
admissible state:

```text
Available(unique non-delete migration) -> outer candidate may remain admissible
RejectedByDelete                      -> outer candidate is not admissible
Missing                               -> outer candidate is not admissible
Ambiguous                             -> outer candidate is not admissible
```

These outcomes remain typed so a final no-candidate diagnostic can report the
relevant cause. A selected delete migration is an explicit rejection, never
“availability.” Once the outer ordinary winner is selected, later validation
cannot reopen its discarded candidate set.

---

## 5. Overload Resolution Pipeline

### 5.1 Notation

```text
Σ   = ResolveSymbol(callee_path)
C0  = CallSiteFamilyView(CallableProjection(Σ), call_site_annotation) -- candidate generation begins
C1  = VisibleObjects(C0, V)                 -- V ∈ {Internal, External}
C2  = ExposePhaseViews(C1, Phase)
C3  = AssociatedCallEntryAndShapeMatch(C2, E)
A   = FullyAdmissible(
        C3,
        Phase,
        invocation_frame,
        expected_result,
        compile_type_requirements
      )

D   = DeclarationCandidatePolicy(A)

Bp' = MaxPolicyProduct(
        D,
        Phase,
        invocation_frame,
        target_result_policy,
        optional_atomic_migration_endpoints
      )
B1  = MaxEntryPreference(Bp')
B2  = MaxConceptOrder(B1, E)
B3  = MaxExtractionSpecificity(B2, E)
B4  = PreferFirstOrderOverInstantiated(B3)
B5  = PreferInPlaceOverNonInPlace(B4)
B6  = ApplyNamedStrategyRules(B5, D)

M = {
  c in D
  |
  overload_strategy(c) = must_select_if_qualified
}
```

The canonical stage order is therefore:

```text
Resolve Symbol
-> call-site candidate-family filter
-> generate candidates (C0--C3)
-> FullyAdmissible (A)
-> declaration-side candidate policy (D)
-> ordinary partial orders (Bp', B1--B6)
-> unique selection
```

Only the two annotation layers' positions and separation are closed here.
Call-site annotation syntax and general selector algebra remain deferred.

### 5.2 Pipeline invariants

Every `Bi+1 = f(Bi, ...)` preference step satisfies:

```text
Bp' ⊆ D ⊆ A, B1 ⊆ Bp', and Bi+1 ⊆ Bi -- monotonic filtering
f is side-effect-free                  -- no observable effects
f is independent of candidate order    -- same result regardless of iteration order
```

The current-language filters execute in exactly the normative `Bp'`, then `B1`
through `B6` order.
Candidate iteration and source declaration order do not affect an individual
filter, but filters are not assumed to commute.

### 5.3 C0: heterogeneous value objects

After path resolution has produced the callee `Symbol` `S`, `C0` is formed in
one step as `CallableProjection(S) = DedupCandidateIdentity(V_S ⊎ V_τ)` (symbol-first §2.1) and
enumerates the projected candidates. These candidates may have
unrelated types and different available `Pv:Pp` views. The final model does not
treat same-name namespace children as already-formed callable overloads.

The current implementation's restricted same-name child bucket may still
pre-filter by:

- **name**: same textual name (or operator-identity equivalence)
- **role**: object-role symbols for callable targets; namespace-subspace for
  namespace-qualified lookup
- **arity**: compatible argument count (exact, variadic, or defaulted)
- **syntactic callable shape**: operator identity (`spelling + fixity + arity`)
  for operator calls, ordinary name for ordinary calls

### 5.4 C1–C2: Visibility and policy views

Defined in §2 and §3 respectively.

**Ordering constraint**: export visibility (C1) precedes object policy-view
selection (C2). If an object is not visible in the lookup view, its policy is
not checked.

### 5.5 C3: Associated call entry and basic applicability

Associated-call and structural matching precedes full admissibility.
For every object surviving C2, obtain its type, resolve its type-associated
`()` entry, discard non-callable objects, and match that entry against `E`. A
candidate cannot win on preference if its pattern or type signature does not
match the call operand.

`AssociatedCallEntryAndShapeMatch(C2, E)` removes candidates whose:

- extraction pattern is structurally inapplicable to `E`
- type signature is incompatible with the argument types

It performs no borrow-producing repair:

```text
E : T       =/=> E : T ref | T share
E : symbol  =/=> E : symbol ref | symbol share
E : type    =/=> E : type ref | type share
```

The explicit `ref`, `share`, or `@` operation must already have formed the
actual observation before this stage. Policy preference, candidate filtering,
and automatic pass lowering cannot introduce it merely to rescue a candidate.

### 5.6 A: Fully admissible candidates

`A` removes every hard-illegal candidate before any preference filter runs.
Hard admissibility includes, at minimum:

```text
path and object visibility
object policy-view admission for the current Phase
existence of associated ()
parameter count and structural shape
Pattern/extraction applicability
receiver and explicit-parameter policy-pair compatibility
P2 result pair compatibility with any target-result constraint
expected result class/facet compatibility
concept and ordinary require legality
other compile/type-stage hard preconditions
```

Merely having compile-time type metadata for a runtime symbol does not make the
original runtime value a compile-policy argument; compile projection supplies
the Pattern projection as a distinct argument view.

Concept legality belongs here. A concept-violating candidate never reaches a
preference filter. Full concept semantics remain deferred, but the legality
boundary is fixed.

Lifetime policy does not belong to `A`. This revision defines no lifetime
overload, ordering, refinement, or second selection; ordinary overload must
already be unique, as bounded by
`../lifetime/lifetime-policy-and-overload-boundary.md`.

### 5.7 Declaration-side candidate policy

Declaration-side annotations are evaluated over the complete fully-admissible
set `A`, before any ordinary preference order:

```text
ApplyDeclarationCandidatePolicy(A):
  ordinary = { c in A | not fallback(c) }
  candidates = ordinary, when ordinary is non-empty
               A,        otherwise
  must_select = { c in candidates | must_select(c) }

D = candidates
```

This is not B6 named-strategy execution. It observes the complete fully
admissible set before Policy or Pattern preference. Any
admissible non-fallback candidate counts, including an admissible `delete`.
Once such a candidate exists, fallback candidates leave the future extended
survivor set permanently. A later unique delete rejection, ambiguity,
body/lowering failure, or lifetime failure cannot reopen fallback candidates.

Must-select is likewise a declaration policy over candidates already in `D`;
it contributes a final consistency obligation but no preference score. Syntax
for fallback/must-select and final ordinary candidate storage remain under
surface consolidation. The current Rust fallback marker is only a prototype
fixture; the semantic pipeline position above is closed.

### 5.8 Bp and B1–B6: Preference filters

Only candidates surviving declaration-side policy enter ordinary preference
filtering:

- **Bp' Policy product order**: retain maximal candidates under §4.5, including
  phase-local stage specificity and const/mut positions; include target-result
  policy only when the context supplies one. For an authorized atomic
  runtime-migration call, add input/output endpoint Policy fit as two product
  coordinates. Without those coordinates, this is exactly old Bp.
  Each parameter position is taken from its elaborated formal policy Pattern:
  the callable P2 is inherited first, then an optional `const let` / `mut let`
  slice supplies `Const` / `Mut`; omission supplies `Unspecified`. This carrier
  is part of the externally compared candidate, not merely the body-entry
  environment.
- **B1 Entry preference**: apply any configured entry preference.
- **B2 Concept ordering**: keep maximal legal candidates under the future
  concept-order poset.
- **B3 Extraction specificity**: apply the lexicographic rank from §4.
- **B4 First-order preference**: if otherwise tied, prefer a first-order object
  over an instantiated object.
- **B5 In-place preference**: if otherwise tied after B4, prefer an embedded
  in-place closure candidate over a non-in-place closure candidate. Closure
  kind is candidate metadata, not hard admissibility, and this filter cannot
  rescue an inapplicable in-place closure. An in-place closure remains bound
  to its embedding control-flow layer, has no capture list, and resolves lazy
  outer reads when used at that layer. Headed no-`=>` and `[[strategy]]`
  closures retain this same in-place candidate metadata; head presence does
  not imply ordinary placement.
- **B6 Named strategy rules**: apply strategy metadata carried by
  `UserBody(Named(strategy), ...)` or by compiler-generated function objects.
  A strategy rule is monotone, side-effect-free, independent of iteration
  order, and restricted to candidates already in `D`; it cannot restart
  lookup or make an inadmissible candidate viable. It receives `B5` as its
  input and may only remove members of `B5`; access to `D` is read-only
  metadata for consistency checks such as must-select. Atomic-migration
  endpoint Policy coordinates are already consumed by `Bp'` in §4.5 and are
  not a named strategy. Fallback suppression has already occurred before Bp'
  and cannot be emulated or
  reversed here.

Each stage only removes candidates. First-order preference does not override
extraction specificity; a deeper applicable generic pattern may outrank a
shallower monomorphic pattern before B4 is reached.

### 5.9 Must-select consistency and uniqueness

Compute must-select membership from `D`, after full admissibility and fallback
suppression, not from `C0`, `C2`, `C3`, or an earlier set that has not passed
concept/require legality:

```text
M = {
  c in D
  |
  overload_strategy(c) = must_select_if_qualified
}

M is empty:
  |B6| = 1 -> select the unique candidate
  |B6| = 0 -> error: no matching overload
  |B6| > 1 -> error: ambiguous overload

M = {m}:
  B6 = {m} -> select m
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
Γ; V; Phase ⊢ name(args) ⇓ f

where:
  Γ  = namespace graph + type / concept environment
  V  = lookup visibility view (Internal | External)
  Phase = OpenStatic | SealStatic | Runtime
  E  = normalized extraction tree of the call operand name(args)
  f  = the selected unique overload candidate
```

The judgment reads: in environment `Γ`, under visibility view `V` and lookup
phase `Phase`, the call `name(args)` with extraction tree `E` resolves
to overload candidate `f`.

Derivation:

```text
CalleeSymbol = ResolveSymbol(Γ, name)
C0  = CallSiteFamilyView(CallableProjection(CalleeSymbol), call_site_annotation)
C1  = VisibleObjects(C0, V)
C2  = ExposePhaseViews(C1, Phase)
C3  = AssociatedCallEntryAndShapeMatch(C2, E)
A   = FullyAdmissible(C3, Phase, invocation_frame, expected_result, Γ)
D   = DeclarationCandidatePolicy(A)
Bp' = MaxPolicyProduct(
        D,
        Phase,
        invocation_frame,
        target_result_policy,
        optional_atomic_migration_endpoints
      )
B1  = MaxEntryPreference(Bp')
B2  = MaxConceptOrder(B1, E)
B3  = MaxExtractionSpecificity(B2, E)
B4  = PreferFirstOrderOverInstantiated(B3)
B5  = PreferInPlaceOverNonInPlace(B4)
B6  = ApplyNamedStrategyRules(B5, D)
M   = MustSelectMembers(D)

OrdinaryUnique(B6, f)
MustSelectConsistent(M, B6, f)
────────────────────────────────────
  Γ; V; Phase ⊢ name(args) ⇓ f
```

---

## 7. Relationship to the connected `lang_build` implementation

The older resolver retains a transitional flat symbol-policy filtering layer
that approximates part of C2:

- `PolicyFlag::{Export, Meta, Compile, Seal, Runtime}` — flat compatibility
  flags carried on `PolicyMetadata.policy_set`
- `PolicySet` — bit-set of flags
- `PolicyEnv::OpenStatic` — the current compatibility OpenStatic query view
- `ResolverCode` — miss vs ambiguity discriminator

These remain in the legacy early-meta expansion
(`try_expand_early_meta_initializer`)
which performs a per-policy-pass lookup of meta-function targets. This is
current substrate for, but not a complete implementation of,
`VisiblePolicyViews`.

The restricted v0.8 path implements bounded structural applicability,
meta body-entry checking, extraction specificity for selected source
callables, one remainder pack at each normalized parameter level, and
propagation of source-named strategy metadata after applicability. It does not
execute arbitrary named strategy rules. Separate pair-model tests cover P1/P2
elaboration and PolicyMode product ordering, but the restricted resolver does
not yet carry full pairs through candidate preparation, derive compile
companions, enforce `must_select_if_qualified`, or replace its existing
specificity selector.

The connected ordinary path now implements:

```text
source Symbol or held PatternValue
  -> form CallableProjection(S) = DedupCandidateIdentity(V_S ⊎ V_τ)
  -> value TypeValue
  -> Pattern owner / associated ()
  -> InvocationFrame
  -> C0/C1/C2/C3/A
  -> one Bp' product over ordinary PolicyMode/phase coordinates
       x optional migration input endpoint
       x optional migration output endpoint
  -> bounded B3 Pattern specificity
  -> unique ordinary invocation and complete result entries
```

There is no maxima pass between ordinary and endpoint coordinates. Without a
migration context, the optional endpoint coordinates are absent and the
comparison is exactly the connected old-Bp PolicyMode/phase order. B1, B2, B4,
B5, B6, full Pattern applicability, concepts/requires, and lifetime remain
incomplete/identity boundaries in this bounded slice.

The separate `PolicyTransitionCallable` fixture still proves isolated endpoint
algebra, typed failures, delete, and no-chain laws. Its private
`prototype_endpoint_policy_maxima` remains deliberately non-composable and is
not source-integration evidence or a second resolver.

---

## 8. Deferred / Non-Goals

The following are explicitly **not** part of this design and are deferred to
later phases or separate documents:

```text
general ADL associated-scope discovery / candidate enumeration
  (only the discovery algorithm behind `op::adl`; the three-entrance
   operator dispatch identity boundary itself is closed in §1)
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

Lifetime checking is separately deferred by
`../lifetime/lifetime-policy-and-overload-boundary.md`. This revision defines
no refinement, ABI class, or second selection stage, and no lifetime-driven
re-selection: a lifetime rule never reopens the unique ordinary overload result.
That is a restriction on lifetime *rules*, not a claim that `@` lacks overloads
— `@` is an ordinary continuation-relative name-reification operation with its own candidate set,
specified in that document.

---

## 9. Relationship to Other Documents

| Document | Relationship |
|---|---|
| `static-pattern-spaces-and-extraction-chains.md` §12 | Summary overview of overload resolution; this document is the formal specification |
| `callable-tail-dot-closure-and-pack-pattern.md` | Callable implementation/strategy tail, first-class `.name`, and remainder-pack matching |
| `pattern-normalization-and-first-order-overload.md` | Earlier, narrower candidate-preparation subset (pattern normalization + first-order type-value candidate model) feeding meta object invocation; not full runtime overload resolution |
| `mechanical-argument-passing-and-move-fixed-point.md` | Pass-mode adaptation (move/ref/share/copy/in) is separate from type/rank compatibility; `move` does not create a new type value |
| `call-modes-recursion-and-tail-lowering.md` | Candidate selection feeds invocation lowering, which may eventually produce explicit call modes (`normal` / `tco` / `loop`) |
| `../policy-capability/policy-visibility-symbols.md` | Implementation mapping for current policy metadata |
| `../symbol-world/symbol-policy-and-compile-flow-projection.md` | Canonical `P1` / `P2`, companion, and must-select semantics |
| `../lifetime/lifetime-policy-and-overload-boundary.md` | Canonical owner of `@` and escape checking; also the negative boundary that a lifetime rule cannot reopen the unique ordinary overload result |
| `early-meta-functions-and-namespace-graph.md` | Namespace graph resolves the callee `Symbol`; the current same-name child bucket is only transitional candidate substrate |
| `entity-ref-design.md` | Entity references may resolve through overload candidate sets in later phases |
| `glossary.md` | Defines OverloadCandidate, OverloadSpecificity, OverloadResolutionPipeline |
| `roadmap.md` | v0.8 non-goal, v0.10+ gating phase |
