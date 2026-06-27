# Meta Object Invocation and Policy Reduction

**Status: Non-normative future design. Not current public language behavior. The current implementation only contains a narrow early-meta/source-verification slice. This document specifies the intended model that later phases should converge toward.**

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

The current implementation realizes only a narrow part of this pipeline:
namespace graph lookup, the `PolicyEnv::Meta` visibility environment, the core
meta-functions, and the source verification operations. Argument-shape and
pattern/type candidate matching — the applicable and executable layers — are
future work and are not implemented.

A formal sketch of the intended end-to-end frame:

```text
Γ; LookupEnv ⊢ callee_path ⇓ C_symbol
Γ ⊢ args ⇓ ArgShapes
Γ ⊢ C_symbol × ArgShapes ⇓ C_applicable
Γ; ExecutionEnv ⊢ C_applicable ⇓ selected_callable
Γ; ExecutionEnv ⊢ invoke(selected_callable, args) ⇓ InvocationResult
```

This sketch is a target, not a description of present behavior. Only the first
line (policy-filtered lookup) and a hardcoded primitive dispatch for `struct`
and `verify` exist today.

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

Ordinary source normalization may use **partial** reduction. The evaluator runs
the meta machinery as far as the available meta values and policy allow, then
stops cleanly at a runtime boundary:

```text
meta can run as far as it can;
when it reaches a runtime-only value or callable, it stops;
the residual expression is preserved for the later runtime phase.
```

Other contexts demand **strict** reduction. Verification forms, manifest meta,
compile-time contracts, and the bodies of meta-functions must complete under
meta policy. There is no residual fallback for them:

```text
if a strict meta context cannot resolve or execute using meta-admissible candidates,
the program is ill-formed in that context.
```

A reduction step produces one of a fixed set of results:

```text
MetaReductionResult =
  | MetaValue(value)
  | GraphDelta(delta)
  | Residual(expr, suspension_reason)
  | TailCall(mode, callee, args)
  | Diagnostic(error)
```

`TailCall` is part of the result shape because the language has no loop
construct: iteration and continuation are expressed through call modes, and
future meta functions reuse the same call-mode model. The detailed semantics of
call modes are intentionally left to a future call-mode document; here `TailCall`
only marks that reduction may continue as another call rather than returning a
value, a delta, or a residual.

When reduction does not complete to a value or delta, the suspension is tagged
with a reason:

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
The only difference between the two contexts is what happens when reduction
reaches a legitimate runtime boundary — partial reduction suspends into a
residual, while strict reduction reports that the program is ill-formed in that
context.

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

A meta object may produce any of the following:

```text
graph delta
type value
normalized residual expression
diagnostic
verification result
```

But the mechanism producing those results is still ordinary invocation through
graph-resolved callable objects. There is no separate expansion phase, no
textual substitution, and no privileged rewriting step. A meta object is a
callable selected by the candidate pipeline, executed under an execution
environment, returning one of the `MetaReductionResult` shapes.

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

The model above is the destination. The current implementation is a small step
toward it, useful for grounding the design but not a definition of it.

Current state:

- `crates/lang_build` currently implements a narrow early-meta slice over the
  namespace graph.
- `struct` is a core meta-function symbol resolved through the namespace graph,
  not a parser keyword.
- `verify` is a core meta-visible verification namespace/object, with
  verification operations installed below it as core symbols.
- `PolicyEnv::Meta` is implemented as lookup-visibility filtering only.
- The current early-meta and verification behavior is not yet the full
  invocation model; it is a hardcoded vertical slice.

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
2. Introduce normalized pattern / argument-shape objects.
3. Introduce first-order TypeValueId and callable signature objects.
4. Introduce candidate-set construction for meta invocation.
5. Move `struct` and `verify` dispatch behind the common invocation engine.
6. Add partial versus strict meta reduction modes.
7. Add guarded/short-circuit invocation strategy objects.
8. Only after this, introduce runtime lookup over residual expressions.
```

Runtime lookup is deliberately listed last. It must not be pulled earlier than
the pattern, type-value, and meta-invocation milestones: residuals are only
well-formed once the invocation engine that produces them exists, and runtime
lookup is the consumer of those residuals, not a parallel mechanism.
