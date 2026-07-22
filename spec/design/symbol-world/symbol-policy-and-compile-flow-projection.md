# Symbol Policy and Compile-Flow Projection

Status: canonical future-design note, with an explicitly identified transitional
implementation substrate.

This document owns:

- the `Val1 × Pattern × Val2` symbol-flow model;
- the canonical policy pair `Π = Pv:Pp`;
- general binding-position `P1` projection;
- callable-result `P2` normalization and function-object `P1` derivation;
- `meta`, `compile`, `seal`, and `runtime` visibility boundaries;
- mechanical compile-flow projection and derived compile companions;
- the policy portion of overload admissibility and const/mut preference;
- match staging and coarse automatic-`require` extraction.

Related canonical owners:

- `symbol-first-meta-construction-and-pattern-injection.md` owns symbol-first
  resolution, facets, `compile`/`meta` construction ranks, pattern scopes,
  `struct`, `inject`, and graph-installation boundaries.
- `symbol-construction-units-and-namespace-origin.md` owns namespace origin,
  construction-unit ownership, physical contribution authority, and cross-unit
  closure.
- `../patterns-overload/overload-resolution-design.md` owns the complete
  admissibility and preference pipeline.
- `../lifetime/lifetime-policy-and-overload-boundary.md` owns the negative
  boundary between lifetime policy and type/compile overload resolution.

## 1. Complete Symbol Flow

Language computation is one flow of symbols, not separate type and value
worlds. The abstract semantic shape is:

```text
Symbol = Val1 × Pattern × Val2
```

where:

```text
Val1:
  value leaves in the pattern tree

Pattern:
  the anonymous type/pattern structure carried by those values

Val2:
  objects installed at a pattern level, commonly function objects
```

Traditional categories are degenerate cases:

```text
traditional pure type:
  Val1 is absent

traditional ordinary value:
  Val2 is absent
```

`Val2` does not disappear from symbol-first semantics. Each `Val2` object has
its own policy pair and, when callable, its own type-associated `()` entry.
Different objects in one symbol value facet may therefore expose different
policy slices.

## 2. Canonical Policy Representation

The internal policy representation is a pair:

```text
Π = Pv:Pp
```

```text
Pv:
  policy of the Val1/value component

Pp:
  policy of the Pattern/anonymous-type component
```

A scalar policy is only surface notation or a derived summary. Implementations
must not store one scalar and later attempt to reconstruct `Pv` and `Pp` from
it.

Policy dimensions are typed and orthogonal:

```text
stage:
  meta | compile | seal | runtime

value mutability:
  const | mut

namespace visibility:
  public | private | export | ...

value presence:
  present | optional | absent
```

These dimensions must not be flattened into one untyped atom set and then
unioned indiscriminately.

Ordinary policy judgments reserve `@` for lifetime syntax and use notation such
as:

```text
Γ ⊢ e : (τ, Pv:Pp)
```

or:

```text
Γ ⊢ e : τ ; policy = Pv:Pp
```

## 3. P1 Is a General Binding Projection

`P1` is the optional policy prefix of any binding:

```text
[P1] let x = expr
```

It is not a function-object-only policy position.

### 3.1 Omitted P1

```lang
let x = expr;
```

Omission requests full inference. The binding retains every result pair
produced by `expr`; it does not proactively crop the result.

### 3.2 Single-policy P1

```lang
Q let x = expr;
```

The single-policy form is a value-dominant projection query:

```text
ProjectP1(Q, R)
  = { (v, p) in R | v is visible under Q }
```

The pattern/type component associated with each selected value follows that
value. It is not independently filtered by `Q`.

Therefore:

```lang
runtime let x = expr;
```

means:

1. evaluate `expr` to its policy-indexed symbol result;
2. select entries whose value component has a runtime-visible slice;
3. retain each selected value's associated compile/seal pattern component;
4. reject only if the selected slice is empty.

It does **not** mean that the whole RHS must equal one scalar `runtime` policy.
It also does not normalize to `runtime:runtime`.

### 3.3 Pair P1

```lang
Qv:Qp let x = expr;
```

The pair form filters both components:

```text
ProjectP1(Qv:Qp, R)
  = {
      (v, p) in R
      | v satisfies Qv
      and p satisfies Qp
    }
```

The single form `Q` and pair form `Q:Q` are not equivalent. Surface syntax may
share `PolicySpec`, but elaboration is context-directed.

### 3.4 Absent value component

Pure types and queries that admit either a value or no value require an
explicit value-presence pattern. Design prose uses abstract `S` or `∅`:

```text
runtime|S:compile
```

This admits either:

- a runtime value carrying a compile pattern; or
- no value and a compile pattern.

The normalized AST reserves an absent-value variant. This document does not
freeze `S`, `null`, `val`, or any other source token.

### 3.5 General binding remains runtime-legal

There is no general side condition `binding_policy != runtime`. A binding may
legally be:

```lang
runtime let x = runtime_value;
compile let y = compile_value;
```

After projection, the selected RHS slice must be admitted by the destination
binding policy. In lattice notation:

```text
Πselected <= Πbinding
```

A rule used by compile-flow projection may separately require a source policy
`Psrc != runtime`. That premise applies only to that source position. It must
not be implemented as:

```text
reject if binding_policy == runtime
```

## 4. P2 Produces a Result Policy Pair

Function form is conceptually:

```text
[P1] let F = (...): P2 -> let r => { ... }
```

`P2` describes the call/expression result pair. The function object's available
`P1` stage slices are then derived from `P2`; the causal direction is:

```text
P2 -> function-object P1
```

There is no premise that `P2` is a subset of a pre-existing scalar `P1`, and
`P1` does not determine `P2`.

### 4.1 Explicit P2 pair

P2 may be written directly as:

```text
Pv:Pp
```

Examples:

```text
runtime:compile
runtime:seal
(runtime|compile):compile
(runtime|seal):seal
const+(runtime|compile):compile
```

Stage constraints are:

```text
runtime not in Stage(Pp)

Static(Pv) is empty
  or Static(Pv) = Stage(Pp)
```

Consequences:

- mixed runtime/static stage sets occur only in `Pv`;
- `compile:seal` and `meta:compile` are invalid because their static stages
  disagree;
- `runtime:runtime` is invalid because `Pp` cannot be runtime;
- explicit `runtime:compile` remains valid and requests earlier type
  availability than the default runtime shorthand.

### 4.2 Single-policy P2 normalization

In P2 position only, a single policy `P` expands uniformly:

```text
Pv = P
Pp = P - runtime
```

If `P - runtime` is non-empty:

```text
N2(P) = P:(P - runtime)
```

Examples:

```text
meta
  => meta:meta

compile
  => compile:compile

seal
  => seal:seal

runtime|compile
  => (runtime|compile):compile

runtime|seal
  => (runtime|seal):seal
```

If `P - runtime` is empty, P2 supplies the latest legal static type stage:

```text
N2(runtime) = runtime:lastStatic
```

With `seal` present:

```text
lastStatic = seal
N2(runtime) = runtime:seal
```

This replaces the obsolete fixed expansion `runtime => runtime:compile`.

### 4.3 No P3 and no scalar result-symbol policy

There is no independent P3 position:

```text
P1 let F = (...): P2 -> P3 let r => ...  // rejected model
```

The result remains layered:

```text
Result = Val1 × Pattern × Val2
```

Each returned value/pattern pair retains its `Pv:Pp`. Each returned `Val2`
object retains its own policy pair. The selected caller object does not stamp
one scalar policy onto the entire result symbol.

Current Rust fields named `return_object_policy` are transitional transport
metadata. They are not a language-level P3 and cannot define whole-result
identity.

## 5. Function-Object P1 Is Derived From P2

Let a callable result have:

```text
P2 = P2v:P2p
```

The function object's stage components are:

```text
Stage(P1p) = Stage(P2p)

Stage(P1v) = Stage(P2v) union Stage(P2p)
```

Examples:

```text
P2 = runtime:seal
  => P1stage = (runtime|seal):seal

P2 = (runtime|compile):compile
  => P1stage = (runtime|compile):compile
```

This derivation explains why a runtime body still has a static function-object
view. It is not an independent ban on runtime bindings.

Only stage dimensions are lifted. The following do not propagate from P2:

- returned `mut` or `const` does not make the function object mut/const;
- returned namespace policy does not make the function object public/exported;
- value presence does not rewrite the function object's declaration shape.

Function-object mutability and namespace visibility come from its actual P1
declaration position.

An explicit P1 prefix then projects the derived function-object view like any
other binding. For example, a source prefix `meta|runtime` cannot manufacture a
runtime slice from `P2 = meta`; it selects only the available meta slice.

## 6. Function-Object Runtime and Seal Views

Policy projection creates a restricted view, not a new nominal object:

- original symbol identity is retained;
- original anonymous function-object type relation is retained;
- only the observable member set changes.

Runtime view:

```text
Members(runtimeSlice(F)) = ConcreteMembers(F)
```

It cannot enumerate uninstantiated generic members.

Seal view:

```text
Members(sealSlice(F))
  = ConcreteMembers(F)
    union MaterializedInstances(F)
```

`MaterializedInstances` contains only instances generated and committed before
the seal snapshot. It is not the infinite mathematical set of all possible
generic instantiations.

## 7. Meta, Compile, Seal, and Runtime Visibility

`meta` and `compile` may share evaluator machinery, but remain semantically
different:

```text
meta:
  open-world symbol construction capability

compile:
  static value and PatternValue computation across open, seal, and post-seal
  compile views

seal:
  static visibility domain excluded from open meta lookup

runtime:
  runtime value execution
```

Canonical visibility domains are:

```text
Vis(meta)    = { open }
Vis(seal)    = { seal, postSealCompile }
Vis(compile) = { open, seal, postSealCompile }
```

Therefore:

```text
Vis(compile) is a superset of Vis(seal)
```

This is a visibility-domain relation. It does not require source or AST to
rewrite every `compile` spelling into a literal `compile|seal` union.

Rules:

- meta-policy objects are not visible through ordinary seal lookup;
- seal objects are not ordinary open-meta arguments or members;
- compile lookup can observe seal symbols in seal/post-seal compile contexts;
- seal does not mean a type or symbol ceased to exist;
- seal policy alone grants no reflection or global-scan capability.

## 8. Seal Privilege and the Pre-Seal Snapshot

Global symbol scanning is a capability of compiler-known privileged seal
meta-functions, not a property of every seal-policy object.

On entering seal, freeze:

```text
Wpre = every symbol committed before seal
```

Privileged scan domain:

```text
ScanDomain = Wpre
```

Symbols generated during seal form:

```text
Wseal
Wfinal = Wpre union Wseal
```

But:

```text
ScanDomain != Wfinal
```

The frozen domain prevents source-order-dependent scans, self-observation,
mutually expanding seal generators, and non-finite reflection closure.

Ordinary seal lookup and privileged `Wpre` scanning are distinct operations.
The scan may inspect pre-seal symbol descriptions without making meta-policy
objects ordinary seal-visible arguments.

This document does not freeze all explicit-name dependency ordering inside
seal.

## 9. Namespace Policy Is Shared Across the Pair

Namespace visibility (`public`, `private`, `export`, and future equivalents):

- is valid only in a namespace-scoped P1 declaration position;
- is not a general P2 result policy;
- may be written syntactically on either side of a P1 pair;
- normalizes to one shared namespace attribute;
- conflicts if both sides state different values.

Thus:

```text
public:compile
compile:public
```

normalize to the same namespace visibility plus stage pair.

This is invalid:

```text
public:private
```

Exported global mutable objects are also invalid:

```text
mut in Pv and export in NamespacePolicy => error
```

Namespace policy does not propagate from P2 to a function object's P1.

## 10. Const/Mut Is a Pv Dimension

`const` and `mut` belong only to `Pv`. They do not modify `Pp`.

An unspecified parameter/result mutability is a broad match:

```text
let x
```

It need not be spelled `const|mut`.

Per-position preference for a const actual value is:

```text
const > unspecified > mut
```

For a mut actual value:

```text
mut > unspecified > const
```

This order is local to one compared policy position. It is not a global
conversion ban; an abstraction controls permitted behavior by the callable
members it provides.

### 10.1 Product partial order

Across self, parameters, and an applicable target-result constraint, compare
candidates by product order. Candidate `f` dominates `g` iff:

```text
for every compared position i:
  fi >=i gi

and for at least one position j:
  fj >j gj
```

No total score, exact-match count, position weighting, left-to-right
lexicographic fallback, input-over-output preference, or separate conversion
rank may break incomparability.

The ordinary selection set is:

```text
Max(A)
```

where `A` is the fully admissible candidate set. Success requires:

```text
|Max(A)| = 1
```

If one candidate is better on parameter one and another is better on parameter
two, they remain incomparable and the call is ambiguous.

Return policy participates only when the call context supplies a target policy
constraint.

### 10.2 Delete members

`delete` members remain in the admissible set and product-order comparison. A
delete candidate must not be removed before preference.

If the unique maximal candidate is delete, report that the call matched a
specific rejection member. This permits an abstraction such as:

```text
mut object   -> ordinary member
const object -> more-specific delete member
```

## 11. Compile Flow Is a Mechanical Projection

The complete program first forms:

```text
CompleteSymbolFlow
```

Compile flow is a mechanical normalization:

```text
CompileFlow = compile_projection(CompleteSymbolFlow)
```

It retains:

- Pattern/type-component flow visible to compile;
- compile-policy value leaves;
- compile/meta early calls at their respective capabilities;
- static calls through derived compile companions;
- D residual and Done completion structure.

It removes or defers:

- runtime value-leaf computation;
- runtime body execution;
- runtime branch value selection.

A runtime symbol does not disappear:

```text
Sruntime = Val1runtime × Patternstatic × Val2

compile_projection(Sruntime)
  includes at least Patternstatic
```

Projection does not execute calls, select final overloads, compute predicates,
calculate PatternValue, prove assertions, or decide recursive termination.

### 11.1 Local source restrictions

A specific projection rule may state:

```text
source_policy != runtime
```

This checks the source item of that projection. It says nothing about whether a
general `runtime let` binding is legal.

### 11.2 Calls project homomorphically

Ordinary calls remain ordinary unresolved calls:

```text
C[(args...) f] = (C[args]...) f
```

For a runtime value argument:

```text
C[arg_runtime] = arg_runtime |> type
```

`|> type` is the symbol's static Pattern projection, not a separate traditional
type world.

Objects such as `UnresolvedCallFamily` may be useful implementation IR, but are
not required public language-semantic objects.

### 11.3 Recursion

With calls treated as opaque finite `CallNode`s, one callable body has finite,
bounded, loop-free local flow. There are no source loop or inline-for nodes.

Projection does not expand callees. Recursive evaluation remains ordinary call
evaluation and may form `f -> f` or `f -> g -> f`. Termination is the compile
program's normal semantic obligation; projection introduces no recursive
summary or fixed-point contract mechanism.

## 12. Derived Compile Companion Objects

A runtime-capable `Val2` function object has a default mechanically derived
static companion object. A merely same-named or more-preferred static overload
does not suppress that derivation. The companion is a complete `Val2` function
object:

```text
DerivedCompileCompanionObject {
  object_id,
  origin_runtime_object_id,
  derived_function_object_type,
  associated_static_call_entry,
  overload_strategy = must_select_if_qualified,
  provenance,
}
```

If the origin result pair is `runtime:Qstatic`, the companion result pair is
`Qstatic:Qstatic` (`compile:compile` for explicit `runtime:compile`, or
`seal:seal` for default `runtime => runtime:seal`).

The derived object:

- enters the carrying symbol's heterogeneous value facet;
- has stable identity and origin;
- has its own type and associated `()`;
- participates in normal symbol-first candidate preparation;
- is visible in diagnostics, reflection, and documentation tools;
- is not a hidden fallback after overload failure.

Projected calls remain ordinary calls. Normal static lookup later enumerates
the derived object and performs complete overload resolution.

Explicit replacement may be supported later through semantic `companion_of`
metadata. Ordinary overload priority cannot silently replace a default
companion. Whether default companions may be suppressed, and what equivalent
static interface would then be required, remains open.

Any future source spelling for companion metadata is separate syntax design;
this document freezes no annotation prefix.

## 13. Fully Admissible Candidates and Must-Select

Candidate processing is:

```text
resolve Symbol
  -> enumerate heterogeneous Val2 objects
  -> apply visibility and each bound object's available pair/stage view
  -> resolve each object's type-associated ()
  -> perform all hard shape, pair-policy, concept, and require checks
  -> form fully admissible set A
  -> apply fixed-order preference filters
  -> obtain ordinary survivor set Bn
```

`must_select_if_qualified` is an overload strategy carried by a function object
and propagated to its prepared candidate. Let:

```text
M = { c in A | strategy(c) = must_select_if_qualified }
```

Then:

```text
M is empty:
  use ordinary unique-maximal selection

M = {m}:
  succeed only when Bn = {m}

|M| > 1:
  overload-set inconsistency
```

Must-select is not infinite priority. It requires a fully admissible protected
candidate to remain the unique final choice.

The precise overload pipeline is canonical in
`../patterns-overload/overload-resolution-design.md`.

## 14. Match Staging and D/Done

The language has one pattern-match mechanism. `if/else` is a two-alternative
match, not an independent `if constexpr` system.

Stage follows the scrutinee pair:

```text
static value scrutinee:
  branch selected during static evaluation

runtime value scrutinee:
  Pattern remains in compile projection;
  value branch selection remains runtime
```

Match already has D/Done normal form inside `CompleteSymbolFlow`:

```text
A |> S { body }
  -> D(A, S) + Done(body(S))
```

Compile projection is homomorphic:

```text
C[D(A, S)] = D(C[A], C[S])
C[Done(B)] = Done(C[B])
C[X + Y] = C[X] + C[Y]
```

D/Done is not a separately ordered pass and automatic require does not invent
a parallel CFG branch algebra.

## 15. Coarse Automatic Require

Automatic require extracts complete compile-projected flows that:

1. depend on formal-argument projections or their guarded pattern domains; and
2. terminate in an assertion/verification endpoint.

Endpoints include `assert`, `require`, delete/reject branches, and other
explicit verification structures.

The initial design keeps coarse complete blocks rather than freezing a
node-by-node canonical contract identity.

Serial blocks:

```text
Require(BlockA; BlockB)
  = Require(BlockA) && Require(BlockB)
```

Compile match is one grouped guarded alternative structure:

```text
(Guard(P1) && Require(B1))
||
(Guard(P2) && Require(B2))
```

The outer contract may wrap that grouped OR as one structured atom; this does
not introduce unrestricted top-level Boolean theorem syntax.

Runtime match contributes guarded conjuncts:

```text
(P1 => Require(B1))
&&
(P2 => Require(B2))
```

Guards must not be erased.

Total contract:

```text
Require_total = Require_inferred && Require_manual
```

Manual require cannot remove inferred requirements.

Calls in the slice remain ordinary `CallNode`s. The design does not require
recursive contract summaries or a require fixed point.

## 16. Shared Evaluation Graph

Require and body continuation are views of one compile graph:

```text
CompleteCompileFlow
  |- RequireView
  `- BodyContinuationView
```

For one static node and one canonical instantiation environment, there is one
evaluation identity and result. A result first demanded by require is reused by
body continuation, including overload choice, PatternValue, intermediate
static values, predicates, normalization, and diagnostics.

This is semantic reuse, not a second execution that merely hopes to hit a
cache.

## 17. Surface Grammar

Policy pair syntax is:

```text
PolicySpec
  ::= PolicyExpr
   |  PolicyExpr ":" PolicyExpr

PolicyExpr
  ::= PolicyTerm
   |  PolicyExpr "|" PolicyTerm

PolicyTerm
  ::= PolicyAtom
   |  PolicyTerm "+" PolicyAtom

PolicyAtom
  ::= Name
   |  '(' PolicyExpr ')'
```

Precedence is:

```text
+ higher than |
| higher than :
```

Thus:

```text
const+(compile|runtime):compile
```

parses as:

```text
(const + (compile | runtime)) : compile
```

`PolicySpec` is recognized only in strong policy positions. `meta`, `compile`,
`seal`, `runtime`, `const`, `mut`, namespace words, and future absent-value
spelling remain ordinary lexer names.

The same surface `PolicySpec` elaborates differently by context:

- P1 single policy is value-dominant projection;
- P2 single policy uses `N2(P)` normalization.

## 18. Current Implementation Substrate

Implemented in this PR:

- Raw AST `PolicySpecAst` with value/type components;
- Normalized AST `NormPolicySpec` and an explicit absent-value variant;
- parser preservation of `PolicyExpr:PolicyExpr` in binding and callable P2
  positions;
- structured stage/mutability/namespace/value-presence policy data;
- P2 normalization and validation for the rules above;
- P1 value-dominant and pair projection helpers;
- function-object P1 stage derivation;
- bounded runtime/seal member-view and pre-seal snapshot models;
- const/mut product-partial-order selection substrate, including delete result;
- current initializer binding changed from exact flat-set verification to a
  non-empty stage-slice projection.

Still transitional:

- `PolicyFlag`, `PolicySet`, `PolicyEnv`, `body_entry_policy`, and
  `return_object_policy` transport only a flat compatibility projection;
- the resolver does not yet store full `Pv:Pp` on every symbol entry;
- flat `PolicyEnv::Compile` / `Seal` / `PostSealCompile` visibility filtering is
  wired into the current resolver, but namespace entries do not yet carry or
  project complete `Pv:Pp` views;
- current restricted overload selection is not replaced wholesale by the new
  const/mut product-order substrate;
- no full compile-flow evaluator, derived companion materializer, seal-world
  builder, reflection model, or automatic-require pass exists.

The implementation must not be described as complete policy-pair semantics.

## 19. Required Invariants

1. Internal policy is `Pv:Pp`; scalar policy is surface shorthand or summary.
2. Omitted P1 infers; single P1 projects values and follows their patterns;
   pair P1 filters both components.
3. Runtime is a legal binding policy.
4. P2 single-policy normalization uses `P:(P-runtime)` and
   `runtime:lastStatic`; currently `lastStatic = seal`.
5. `Pp` never contains runtime, and P2 static stages agree across components.
6. Function-object stage P1 is derived from P2; non-stage dimensions are not
   copied from the result.
7. Runtime and seal member views preserve object identity and contain only
   already concrete/materialized members.
8. Meta is open-world-only; compile visibility includes seal/post-seal compile;
   seal alone grants no scan privilege.
9. Privileged seal scans read exactly `Wpre`, never seal-generated `Wseal`.
10. Namespace visibility is one shared P1 attribute; `mut+export` is invalid.
11. Const/mut preference uses product partial order; no score or lexicographic
    fallback exists; delete candidates participate normally.
12. Compile projection is mechanical and does not perform final overload
    selection or recursive expansion.
13. A compile companion is a derived `Val2` object, not a fallback entry.
14. Require and body continuation share one evaluation graph.
15. Ordinary policy notation never reuses lifetime operator `@`.
