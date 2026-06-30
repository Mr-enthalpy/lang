# Meta Object Invocation and Policy Reduction

**Status: Mixed.** This remains the broader future invocation design. The
current implementation contains the earlier source-verification/core-meta path
plus a restricted v0.8 source-declared meta-overload invocation slice described
in §0.1.

This document specifies a single invocation model for the language. Its claim is
that compile-time, meta-time, and runtime behavior are not separate languages
with separate evaluation rules, but one callable-invocation mechanism observed
under different policy environments. The model described here is intended to
eventually subsume the current ad hoc early-meta paths — `struct` expansion and
source verification — without introducing a privileged compile-time syntax,
a macro expansion layer, or a second expression language.

It is a future design. It is not a general macro system, not a full type
checker, not a runtime evaluator, and not a full policy checker. It defines the
invocation *frame* that later passes will use; it does not define the entire
pattern system, type system, or runtime.

## 0.1 v0.8 restricted source-declared meta invocation

v0.8 implements a bounded formal invocation path for selected source-declared
meta overloads:

```text
namespace graph overload candidates
  -> policy and extraction-pattern selection
  -> unique selected source callable
  -> restricted selected-body evaluator
  -> MetaInvocationResult::Value(...) or MetaInvocationResult::Diagnostic(...)
```

This path remains pure. It does not install namespace graph deltas; binding or
materialization remains the graph-installation boundary.

Supported selected body forms:

- delete body, such as `("message") delete`, returns
  `MetaInvocationResult::Diagnostic(...)`;
- simple forwarding equality, such as `{ r === t; }` or `{ r === unit; }`,
  returns `MetaInvocationResult::Value(MetaInvocationValue::ForwardedValue(...))`
  when the forwarded type-pattern value is available in the graph.

Unsupported selected body forms return hard diagnostics. In particular, a body
that requires guarded branch evaluation, predicate calls, postfix `?`,
short-circuit behavior, D/Done reduction, or a full meta block interpreter is
outside this v0.8 slice. `delete` is not a value and there is no
`CoreMetaFunction::Delete`.

The body-entry policy is derived from the selected closure/function head, not
from symbol visibility. For example:

```lang
meta | runtime let + =
  (self, t: type, u: type): meta -> let r: type =>
{
  r === t;
};
```

has symbol self-policy `{ Meta, Runtime }`, body-entry policy `{ Meta }`, and
return-object policy `{ Meta, Runtime }` by default. Runtime lookup may see the
symbol metadata if that lookup phase is requested, but runtime execution must
not enter this meta-only body.

## 0.2 v0.8 default initializer evaluation

Ordinary initializer evaluation is policy-inferred, not annotation-triggered.

For:

```lang
let X: type = int + unit;
```

the `: type` annotation is checked after RHS evaluation. It is not the reason
the RHS enters meta evaluation. The RHS enters the default inferred evaluation
strategy because the binding policy is omitted:

```text
default ordinary initializer strategy = meta | runtime / MetaPartial
```

`MetaPartial` evaluates the normalized AST as far as meta policy allows. If a
call can be reduced through meta-visible source-declared overloads, it returns
a meta value. If it cannot be meta-reduced but has a legal runtime boundary, it
produces a residual expression and the binding policy inference records that
the result is not a pure meta value. Runtime fallback is residualization; it is
not a second runtime lookup that produces a compile-time value.

`MetaStrict` is used inside selected meta-only bodies. It does not allow
residualization to complete the current meta value:

```text
runtime-only dependency in MetaStrict context => diagnostic
```

Runtime body-entry policy does not ban local meta actions. Semantically, a
runtime body may contain local declarations whose initializers are evaluated
under the local default `MetaPartial` strategy. The runtime policy says the
callable's input-to-output mapping is runtime-entered; it is not a blanket ban
on all meta actions in the body. The restricted v0.8 implementation only proves
that runtime-body declarations may contain such local meta-shaped initializers;
full runtime-body execution and local binding materialization are deferred.

Omitted binding policy introduces an inference variable. Explicit policy turns
that inference into verification:

```lang
let x = expr;                 // infer from Value or Residual
meta | runtime let x = expr;  // verify RHS result can satisfy both flags
runtime let x = expr;         // verify runtime-only visibility is enough
```

If `expr` only residualizes to runtime, `meta | runtime let x = expr` fails
verification because no meta-visible value was produced. Ambiguous
meta-visible candidates remain hard diagnostics in both `MetaPartial` and
`MetaStrict`; ambiguity is not residualized.

Verification consumes the RHS result policy. Direct type-name forwarding uses
the forwarded type symbol's own policy. Restricted source-callable invocation
uses the selected callable's return-object policy. This keeps binding policy
inference and explicit policy verification from collapsing every successful
meta value to `meta | runtime`.

When binding policy is omitted and RHS evaluation succeeds with a value, the
binding policy is inferred from that RHS result policy and written onto the
materialized binding. For example, a `meta let + = ...` source callable whose
return-object policy is meta-only produces a meta-only `let X: type = int +
unit;` binding when no explicit policy is written. Explicit policy annotations
still use the same result policy for shrink-only verification. Inference does
not implicitly copy export visibility from a forwarded dependency or core
object; it uses the phase capability portion of the result policy for the new
binding.

If any initializer residualizes and the binding has an assertion annotation
such as `: type`, the assertion is not considered proven or failed. It is
deferred with the residual expression. Because v0.8 has no deferred/runtime
type assertion model, the implementation reports:

```text
UnsupportedDeferredTypeAssertion
```

This diagnostic means the assertion boundary is unsupported for residual
initializers; it does not mean the RHS was already checked and found not to be
a type-level meta value.

The v0.8 success path for `let X: type = int + unit;` is:

```text
source declarations install real `+` overload symbols
ordinary initializer sees normalized `int + unit`
MetaPartial invokes restricted overload selection under MetaAction lookup
selected `(self, t: type, _ unit: type): meta -> ...` body forwards `t`
RHS value is `ForwardedValue(int)`
`: type` assertion checks that the RHS is a type-level value
binding materialization installs `X` as a type forwarding `int`
```

The identity path does not require full canonical sum-pattern values. A
selected body such as `r === t | u` still requires canonical sum-pattern value
support; until that exists, v0.8 reports an explicit unsupported diagnostic
instead of faking success.

Selected meta body local-let support is intentionally narrow: local let
initializers are checked under `MetaStrict`, but local binding materialization
inside selected bodies is not implemented. The supported forwarding body still
resolves only selected parameter bindings or graph-resolved names.

### Structured v0.8 failure routing

The v0.8 initializer evaluator does not inspect diagnostic message text for
semantic routing. Restricted overload selection returns structured failure
kinds and code-tagged diagnostics. The initializer evaluator maps those kinds
to residualization or hard diagnostics:

```text
AmbiguousCandidate
  => hard diagnostic in MetaPartial and MetaStrict

NoSourceDeclaredCallable
NotVisibleToLookupPhase
NoApplicableCandidate
  => Residual in MetaPartial
  => ResidualNotAllowedInMetaStrict in MetaStrict

BodyEntryPolicyMismatch
  => Residual in legal MetaPartial initializer contexts
  => ResidualNotAllowedInMetaStrict in MetaStrict
```

Unsupported selected-body forms remain diagnostics. Canonical sum-pattern
values such as `r === t | u` report
`UnsupportedCanonicalSumPatternValue`. Selected meta body local-let forms that
would require a parameter/local binding environment report
`UnsupportedSelectedMetaBodyLocalBinding`; v0.8 does not implement that local
environment.

## 1. Purpose

The language wants exactly one invocation model. The same mechanism must serve:

- ordinary functions,
- meta functions,
- verification operations,
- future control predicates,
- operators,
- type constructors,
- source-level meta actions.

There is intentionally no second mechanism reserved for "compile-time code."
Compile-time behavior is just ordinary callable behavior that happens to resolve
and execute under a stricter policy environment.

```text
There is no privileged `if constexpr` split.
There is no separate compile-time-only expression language.
There is no macro expansion layer that rewrites syntax by textual privilege.
Meta behavior is ordinary callable behavior observed under a stricter policy environment.
```

Concretely, `struct`, `verify`, a future `cond`, and the future predicate
operators `&&`, `||`, `==`, and `!=` should all eventually be ordinary callable
symbols selected by the same lookup-and-invocation mechanism. They are not
parser keywords and not normalizer special cases. Whatever specialness they
have lives in:

- their **symbol payload** (what kind of callable object they are),
- their **invocation strategy** (how arguments are evaluated and how branches
  are selected),
- their **policy** (where the symbol is visible and where its body may execute),

and never in parser recognition of the name. The parser and normalizer preserve
normalized structure; the meaning of `struct`, `verify`, `cond`, or `==` is
decided by graph lookup and policy-governed invocation, not by the spelling of
the name.

## 2. Lookup visibility is not execution permission

The model keeps two judgments strictly separate. One judgment decides whether a
symbol is *visible* to a query; the other decides whether a resolved callable
may be *executed*.

```text
Γ; LookupEnv ⊢ path ⇓ symbol

Γ; ExecutionEnv ⊢ call(symbol, args) ⇓ result
```

Three policy planes participate, and they answer three different questions:

- **Symbol policy** controls whether a symbol appears in a candidate set for a
  lookup environment. It is a *visibility* filter on resolution.
- **Body-entry policy** controls whether a callable body may be *entered* in an
  execution environment. It is an *execution* gate.
- **Return-object policy** controls the policy of the object the callable
  *produces*. It describes the result, not the call.

Because these are different planes, visibility never implies executability. The
following inference is invalid:

```text
symbol visible under PolicyEnv::Meta
⇒ callable executable under meta
```

The correct distinction is:

```text
symbol visible under PolicyEnv::Meta
⇒ the compiler/meta pass may resolve, inspect, classify, or residualize it

callable body-entry policy admits Meta
⇒ the compiler/meta pass may enter or evaluate the callable body
```

A meta pass may therefore resolve a symbol whose body is runtime-only: it can
see it, inspect its object, classify it, type-check against it, and build a
residual runtime call that defers the actual execution. What it may not do is
*enter the body* under a meta execution environment when the callable's
body-entry policy does not admit meta. Seeing a symbol and running its body are
distinct permissions governed by distinct policy planes.

This separation is the load-bearing invariant of the whole model. Candidate
construction (Section 3), partial vs strict reduction (Section 4), guarded
invocation (Section 5), and residualization (Section 6) all rely on the fact
that a symbol's presence in a candidate set says nothing, by itself, about
whether its body can run here and now.

## 3. Candidate pipeline

Invocation is resolved through a pipeline of progressively narrower candidate
pools. Each layer adds one kind of constraint and never re-opens an earlier
decision.

```text
symbol candidate pool:
  name/path + role + visibility policy

callable candidate pool:
  symbol candidate pool + callable kind + arity/shape

applicable candidate pool:
  callable candidate pool + argument shape + normalized pattern compatibility
  + first-order type-value compatibility

executable candidate pool:
  applicable candidate pool + body-entry policy compatible with demanded execution environment
```

Reading the layers from the top:

- The **symbol candidate pool** is everything resolvable for the requested
  name/path in the requested role, after visibility-policy filtering.
- The **callable candidate pool** keeps only those symbols that are callable in
  the requested way, matching callable kind and arity/shape.
- The **applicable candidate pool** keeps only those callables whose parameter
  patterns and first-order type-value expectations are compatible with the
  actual argument shapes.
- The **executable candidate pool** keeps only those applicable callables whose
  body-entry policy is compatible with the execution environment the call site
  demands.

The current implementation realizes this pipeline only for two narrow paths:
the earlier core-meta/source-verification path, and the v0.8 restricted
source-declared meta-overload path. The v0.8 path has argument-shape matching,
restricted parameter-pattern applicability, body-entry filtering, and selected
simple-body evaluation, but not full runtime overload resolution, concepts,
lifetime preconditions, guarded branch execution, or arbitrary meta block
interpretation.

A formal sketch of the intended end-to-end frame:

```text
Γ; LookupEnv ⊢ callee_path ⇓ C_symbol
Γ ⊢ args ⇓ ArgShapes
Γ ⊢ C_symbol × ArgShapes ⇓ C_applicable
Γ; ExecutionEnv ⊢ C_applicable ⇓ selected_callable
Γ; ExecutionEnv ⊢ invoke(selected_callable, args) ⇓ InvocationResult
```

This sketch is the target for general invocation. v0.8 proves the path for a
restricted source-declared meta-overload subset and leaves the omitted layers
explicitly deferred.

## 4. Partial meta reduction versus strict meta execution

This is the central section. A call site is reduced in one of two contexts, and
the difference between them is what makes a single mechanism cover both
"compile-time only" and "residual runtime" behavior.

```text
MetaPartialContext
  The evaluator attempts to reduce as much as policy and available meta values allow.
  A runtime-only boundary may suspend and produce a residual expression.

MetaStrictContext
  The evaluator must complete under meta policy.
  If lookup or execution requires a non-meta candidate, this is a hard diagnostic.
```

### 4.1 Invocation layer: meta return values

The invocation layer evaluates or reduces a meta callable under policy. It
returns a **value**, not merely a `TypeValueId`.

For ordinary generic type-to-type meta construction (see the clarification
document), the invocation result must distinguish at least:

```text
GeneratedConstructionValue(callee, canonical_args, computed_material, external_shape)
ForwardedValue(target)
Residual(expr, suspension_reason)
Diagnostic(error)
```

This vocabulary matters because the system needs to know whether the call
produced a new generative construction (`r = t`), forwarded an existing value
(`r === t`), suspended at a runtime boundary, or failed. A bare `TypeValueId`
reduction collapses these categories and is only acceptable for placeholder
proof paths (see §4.3).

The internal evaluation model remains phase-internal. At the public semantic
boundary for formal meta invocation, the result vocabulary is:

```text
MetaInvocationResult =
  | Value(MetaInvocationValue)
  | Diagnostic(error)
```

Namespace graph installation is not part of formal invocation. Binding
materialization consumes `MetaInvocationValue` and installs `NamespaceDelta`.
Future user-defined meta bodies may need an internal control-state vocabulary,
but that vocabulary must stay below this invocation boundary.

When candidate preparation or invocation cannot proceed, diagnostics should name
the current semantic boundary:

```text
NoMetaVisibleCandidate
RuntimeOnlyValue
BodyEntryPolicyMismatch
UnresolvedUnderMeta
ExecutionRequiresRuntimePolicy
```

The recommended behavior at each boundary depends on the context:

```text
No meta candidate:
  MetaPartialContext => residualize
  MetaStrictContext  => error

Meta-visible ambiguity or conflict:
  both contexts => error

Meta candidate exists but body-entry policy does not admit meta:
  MetaPartialContext => residualize if residualization is legal
  MetaStrictContext  => error
```

Ambiguity and conflict are errors in *both* contexts: a residual is the deferral
of a single well-identified call, not a way to paper over an unresolved choice.

### 4.2 Expansion / binding layer

After the invocation layer produces a value, the expansion / binding layer
applies it to a build or declaration context. This includes:

```text
- installing a NamespaceDelta atomically;
- binding the declared target (e.g. `let T: type = ...`);
- exposing an extraction-facing interface on the constructed value;
- applying the return-object policy of the callee.
```

This separation is intentional: the invocation result is a **value**, and the
expansion is a **side-effecting binding operation** that consumes that value.
Conflating the two into a single `MetaExpansionResult` without distinguishing
the value from the binding is acceptable as a current temporary shortcut but
must not harden into the permanent model.

### 4.3 IdentityType is a placeholder proof path only

`IdentityType` proves graph-resolved invocation plumbing: it demonstrates that
a prepared candidate can flow through the candidate preparation, key
computation, cache lookup, and primitive reduction pipeline. It does **not**
prove generative type-to-type construction.

```text
IdentityType proves:
  graph-resolved target lookup;
  normalized call-site extraction;
  argument product shaping and classification;
  candidate preparation and policy checking;
  formal meta invocation dispatch;
  canonical key computation and cache memoization.

IdentityType does NOT prove:
  generative construction value production (r = t);
  forwarding value production (r === t);
  declaration binding from arbitrary meta return values;
  extraction-facing interface exposure;
  ordinary generic type constructor behavior.
```

Any implementation, test, or document that uses `IdentityType` as evidence that
ordinary type-to-type meta construction has been implemented is incorrect.

## 5. No `if constexpr`: guarded invocation instead

The language does not introduce an `if constexpr` versus `if` split. Control-like
constructs are not privileged compile-time syntax. `cond`, `&&`, `||`, equality
predicates, inequality predicates, and similar constructs are ordinary callable
objects, distinguished only by their invocation strategy.

An invocation strategy is metadata and semantics attached to a callable object.
It describes how arguments are evaluated and how branches are selected:

```text
InvocationStrategy =
  | Eager
  | Lazy
  | GuardedBranch
  | ShortCircuit
  | PatternDirected
```

How the strategies are intended to be used:

- Ordinary functions default to **eager** argument evaluation.
- `cond` uses **guarded branch** evaluation: it evaluates a predicate and then
  selects a single branch.
- `&&` and `||` use **short-circuit** evaluation.
- Operators such as `==` and `!=` may have meta-visible overloads, selected and
  invoked through the same pipeline as any other callable.
- If all needed components are meta-visible and meta-executable, the expression
  reduces at meta time to a meta value.
- If any required component is runtime-only, partial meta reduction suspends
  into a residual, or strict meta execution reports an error.

The key example is guarded branch selection:

```text
cond(pred, then_branch, else_branch)
```

If `pred` reduces to a meta boolean, only the selected branch is resolved and
evaluated. The unselected branch does not create lookup obligations. This is the
crucial property of guarded invocation: a branch that is **not entered must not
force policy lookup of any symbol inside it**. The unselected branch is not
required to resolve, not required to type-check at meta time, and not required
to have meta-visible callees.

This property is also the foundation for future `noerror` behavior and error
propagation. Because an unentered branch imposes no lookup or execution
obligation, a guarded construct can route around a failing or runtime-only
branch without that branch contaminating the meta reduction of the taken path.
Guarded invocation, not a privileged `if constexpr`, is what gives the language
compile-time branch selection.

The same rule applies to match-like control forms: a `match`-like consumer is
an ordinary meta-callable over pattern spaces / extraction results, not a
parser-level privileged node.

## 6. Residual runtime expressions

A residual is the expression that remains after all admissible meta reduction
has completed in a context that permits partial reduction. It is **not** a
failed compile-time computation. Reaching a runtime boundary under partial
reduction is the normal, expected outcome for any expression that legitimately
depends on runtime values.

Residualization is tightly scoped:

```text
Residualization is legal only in contexts that explicitly allow partial meta reduction.
A strict meta context must not silently residualize.
```

A strict meta context that reaches a runtime boundary must diagnose, not
quietly emit a residual. Silent residualization in a strict context would erase
the very guarantee that the strict context exists to provide.

The later runtime phase performs runtime lookup and type checking over residual
expressions. This document does not define runtime lookup. Runtime lookup is
intentionally a *later* concern than this model: the meta invocation model
prepares residuals and guarantees they are well-identified deferred calls, but
it does not decide all runtime correctness. The residual is a handoff, and the
runtime phase that consumes it is specified separately and afterward.

## 7. Meta object invocation is not macro expansion

Meta object invocation operates on structured objects, not on text. A meta
object is invoked like any other callable and receives structured inputs:

```text
A meta object receives normalized objects, pattern objects, type values, graph objects,
or argument shapes. It does not receive raw text by default.
```

A meta object may produce:

```text
a meta invocation value (GeneratedConstructionValue, ForwardedValue)
a residual expression
a diagnostic
```

Graph deltas and declaration bindings belong to the expansion/binding layer
(§4.2), not the ordinary returned-value layer.

But the mechanism producing those results is still ordinary invocation through
graph-resolved callable objects. There is no separate expansion phase, no
textual substitution, and no privileged rewriting step. A meta object is a
callable selected by the candidate pipeline, executed under an execution
environment, returning an invocation result.

This is why the front end must stay neutral:

```text
Parser and normalizer should not special-case names like `struct`, `verify`, `cond`,
or future predicate operators. They should preserve normalized structure. Later graph
lookup and policy-governed invocation decide what those names do.
```

Two consequences follow. Closure-like source material remains syntax /
normalized material until a later semantic / meta-invocation step explicitly
materializes it as an object; the candidate pipeline does not assume a
pre-materialized callable. And surface call syntax is not a traditional
`f(args)` grammar: meta invocation consumes normalized expression / product /
call-chain material, not a parser-produced call node.

## 8. Relation to existing early-meta slice

The model above is the destination. The current implementation contains two
bounded steps toward it, useful for grounding the design but not a definition
of full invocation.

Current state:

- `crates/lang_build` implements a narrow early-meta slice over the namespace
  graph.
- `struct` is a core meta-function symbol resolved through the namespace graph,
  not a parser keyword.
- `verify` is a core meta-visible verification namespace/object, with
  verification operations installed below it as core symbols.
- Source-declared callable/meta-function overloads can be harvested into graph
  symbols and selected by the restricted v0.8 overload path.
- `PolicyEnv::Meta` and `PolicyEnv::Runtime` support visibility metadata; the
  restricted overload selector also checks selected body-entry policy before
  meta execution.
- The current early-meta, verification, and v0.8 overload behavior are not yet
  the full invocation model; they are bounded vertical slices.

Intended convergence: the existing `struct` and `verify` paths should eventually
stop being bespoke code and instead become clients of one shared meta invocation
engine:

```text
Current `struct` and `verify` paths should eventually be expressed as clients of the same
meta invocation engine:
  resolve callee
  collect candidates
  match argument shape/pattern/type value
  check execution policy
  invoke primitive or residualize/error
```

This convergence is a design intention, not an implemented fact. Today's slice
short-circuits most of the pipeline; the engine that would generalize it does
not exist yet.

## 9. Relation to pattern normalization and first-order type values

Full candidate selection — the applicable and executable layers of Section 3 —
depends on machinery that this document does not define. Argument shape,
normalized parameter pattern compatibility, and first-order type-value
compatibility are prerequisites for real overload selection. This document
defines the invocation frame only; it does not define the pattern system or the
type-value system.

For v0.8-adjacent type-to-type meta construction, argument shape means the
contract-shaped route through `ProductObject` / `ArgProductShape`, not
callee-specific parsing of raw normalized product material. Canonical meta
instance keys must be computed only after product canonicalization and
first-order `TypeValueId` argument compatibility are established. The detailed
construction guardrails live in
`spec/contracts/v0.8-meta-construction-agent-constraints.md`.

```text
Planned companion documents:
- `pattern-normalization-and-first-order-overload.md`
- `type-values-places-and-alias-forwarding.md`

Until those documents exist, the closest background material is
`static-pattern-spaces-and-extraction-chains.md`,
`overload-resolution-design.md`, and
`type-associated-function-objects-and-access-trees.md`.
```

The background documents above are not load-bearing for this model. They provide
context and prior design exploration, but they do not define the invocation
semantics specified here, and this document does not depend on them for its
meaning. The invocation frame stands on its own; the planned companion documents
will later supply the pattern-normalization and type-value details that the
applicable/executable layers require.

## 10. Relation to package/manifest identity

Package and manifest identity affects where candidates may come from, because it
determines the boundaries of the candidate search. The relevant boundaries are:

- core mount,
- package namespace root,
- dependency mount,
- export surface,
- source root contribution,
- package artifact metadata.

These determine which symbols are reachable and which are exported across a
package boundary, and therefore which candidates can populate the symbol
candidate pool for an external lookup. This document does not define manifest
syntax or the build graph. For those, see the existing build and package design
notes (`build-system-design.md`, `package-manifest-v0.md`, and
`namespace-assembly-v0.md`).

## 11. Non-goals

This document does not define:

```text
- full runtime lookup
- first-order type checking
- full overload resolution
- full pattern-space extraction
- macro expansion
- parser syntax changes
- complete policy lattice
- effect checking
- borrow checking
- ABI lowering
- code generation
- package dependency solving
```

## 12. Future implementation milestones

The model is expected to be reached in stages. The ordering matters: runtime
lookup over residual expressions must come last, after the pattern, type-value,
and meta-invocation machinery exists.

```text
1. Keep current `struct` and `verify` behavior as implemented vertical slices.
2. Introduce ProductObject / ArgProductShape and normalized pattern /
   argument-shape objects.
3. Introduce first-order TypeValueId and callable signature objects.
4. Introduce canonical meta instance keys for type-to-type meta construction.
5. Introduce candidate-set construction for meta invocation.
6. Move `struct` and `verify` dispatch behind the common invocation engine.
7. Add partial versus strict meta reduction modes.
8. Add guarded/short-circuit invocation strategy objects.
9. Only after this, introduce runtime lookup over residual expressions.
```

Runtime lookup is deliberately listed last. It must not be pulled earlier than
the product/argument-shape, pattern, type-value, canonical-key, and
meta-invocation milestones: residuals are only well-formed once the invocation
engine that produces them exists, and runtime lookup is the consumer of those
residuals, not a parallel mechanism.
