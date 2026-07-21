# Meta Object Invocation and Policy Reduction

**Status: Mixed.** This remains the broader future invocation design. The
current implementation contains the earlier source-verification/core-meta path
plus a restricted v0.8 source-declared meta-overload invocation slice described
in §0.1.

The canonical future result-rank and construction boundary is
`spec/design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md`.
It supersedes the older formal-meta-return interpretation that used `r = ...`
for generation and `r === ...` for forwarding. References to that split below
are explicitly current transitional implementation notes, not final semantics.
Namespace-origin and `MetaConstructionUnit` ownership are canonical in
`spec/design/symbol-world/symbol-construction-units-and-namespace-origin.md`.
The canonical symbol-flow policy model, callable `P1` / `P2` boundary,
compile-flow projection, compile companions, and automatic require are in
`spec/design/symbol-world/symbol-policy-and-compile-flow-projection.md`.

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

This path remains graph-installation-free. It does not install namespace graph
deltas; binding or materialization remains the graph-installation boundary.

Supported selected body forms:

- delete body, such as `("message") delete`, returns
  `MetaInvocationResult::Diagnostic(...)`;
- the transitional simple forwarding body, such as `{ r === t; }` or
  `{ r === unit; }`, returns
  `MetaInvocationResult::Value(MetaInvocationValue::ForwardedValue(...))` when
  the forwarded type-pattern value is available in the graph.

This `r === ...` behavior describes only the restricted v0.8 evaluator that is
currently implemented. The final formal meta model uses `r = ...` to populate a
`SymbolConstructionValue`; ordinary `let a === b` remains the separate
symbol/place alias form.

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

currently has symbol policy metadata `{ Meta, Runtime }`, body-entry metadata
`{ Meta }`, and transitional return-object metadata `{ Meta, Runtime }` by
default. Runtime lookup may see the symbol metadata if that lookup phase is
requested, but runtime execution must not enter this meta-only body. These are
current implementation fields, not three final source-level policy positions;
the final form has `P1`, `P2`, and no independent `P3`.

The written `self` formal denotes the callable frame's explicit slot 0
self-position. It is injected by invocation after callable resolution; it is not
part of the call-site explicit product, `ProductObject`, `ArgProductShape`, or
`RawArgShape`. The explicit user product for the example contains only the
user-supplied positions after slot 0.

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

Final policy-binding semantics distinguishes destination policy `P` from RHS
policy `P_e`:

```text
Gamma |- expr : tau @ P_e
P_e ⊑ P
-------------------------------
Gamma |- P let x = expr
```

Omitting `P` requests inference. Writing `P` constrains the destination through
the same parent/admission relation; it does not add a universal non-runtime
condition:

```lang
let x = expr;                 // infer P from the evaluated RHS
meta | runtime let x = expr;  // legal when P_e ⊑ (meta | runtime)
runtime let x = expr;         // legal when P_e = runtime
```

The current v0.8 verifier does not yet implement that final relation. It treats
each written flag as an independently required result flag, so an expression
that only residualizes to runtime currently makes
`meta | runtime let x = expr` fail because no meta-visible value was produced.
This is a transitional implementation limitation, not final policy-binding
semantics and not evidence for a general `P ≠ runtime` rule. Ambiguous
meta-visible candidates remain hard diagnostics in both `MetaPartial` and
`MetaStrict`; ambiguity is not residualized.

The current verifier consumes provisional RHS result-policy metadata. Direct
type-name forwarding uses the forwarded type symbol's current policy, while
restricted source-callable invocation uses the selected callable's
transitional `return_object_policy` field. This describes v0.8 transport only;
it is not a final `P3` rule or whole-result policy. Final semantics applies
layer-directed `result_projection_by_P1`: Pattern retains its supported compile
part, Val1 retains stages admitted by the selected object's P1, and every
returned Val2 object keeps its own P1.

When binding policy is omitted and RHS evaluation succeeds with a value, the
binding policy is inferred from that RHS result policy and written onto the
materialized binding. For example, a `meta let + = ...` source callable whose
transitional return-object field is meta-only produces a meta-only
`let X: type = int + unit;` binding when no explicit policy is written.
Explicit policy annotations still use that provisional result metadata in the
current exact-flag verifier. This transport behavior must migrate to `P_e ⊑ P`;
it is not the final binding judgment. Inference does not implicitly copy export
visibility from a forwarded dependency or core object; it uses the phase
capability portion of the result policy for the new binding.

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
binding materialization installs `X` as a fresh symbol/place whose type facet
projects the `int` type value; this is not ordinary `let X === int` aliasing
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
- future pattern-match consumers and predicates,
- operators,
- type constructors,
- source-level meta actions.

There is intentionally no second mechanism reserved for "compile-time code."
Compile-time behavior uses the ordinary callable framework under either
`compile` capability (producing `PatternValue`) or `meta` capability (producing
`SymbolConstructionValue`). Policy and partial/strict demand determine whether
the callable may execute or residualize; they do not merge the two result ranks.

```text
There is no privileged `if constexpr` split.
There is no separate compile-time-only expression language.
There is no macro expansion layer that rewrites syntax by textual privilege.
Meta behavior is ordinary callable behavior observed under a stricter policy environment.
```

Concretely, `struct`, `verify`, match-closing consumers, and predicate operators
such as `&&`, `||`, `==`, and `!=` should all eventually be ordinary callable
symbols selected by the same lookup-and-invocation mechanism. None introduces
a second branch semantics beyond policy-staged pattern matching. They are not
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

## 2. P1 Lookup Visibility Is Not P2 Execution Permission

The final callable form is:

```text
P1 let F = (...): P2 -> let r => { ... }
```

It has two distinct judgments:

```text
Gamma; lookup_stage |- path => Symbol

Gamma; lookup_stage |- value_facet(Symbol) => Val2*

Gamma; lookup_stage |- P1_filter(Val2*) => stage-visible objects

Gamma; P2 |- call(selected object, InvocationFrame) => result
```

The first judgment is base symbol resolution. `P1` is not consulted until the
resolved symbol's heterogeneous value objects have been enumerated. Different
objects stored by the same symbol may have different `P1` sets.

`P1` is the callable object's externally visible lookup-stage set:

```text
P1 ::= compile
     | (compile | runtime)
```

A callable function object is always compile-visible. Runtime body execution is
expressed by `P2 = runtime` on an object whose `P1 = compile | runtime`.

`P2` is an individual callable entry's execution capability and exact
invocation-frame total-policy requirement:

```text
P2 ::= compile | meta | runtime

external(compile) = compile
external(meta)    = compile
external(runtime) = runtime
```

A declaration is well formed only when:

```text
external(P2) subset-of P1
```

A call entry is policy-qualified only when:

```text
current_lookup_stage in P1

and

for every invocation-frame slot a:
  total_policy(a) = external(P2)
```

Slot 0 is the implicit `self` view of the selected `Val2` function object;
slots 1..n are the explicit source arguments. The declaration condition
`external(P2) subset-of P1` guarantees that `self` is available at the entry's
stage, and the invocation frame records the same exact stage requirement for
all slots.

Visibility therefore never implies executability. A compile lookup may see a
runtime entry whose object has `compile | runtime` `P1`, inspect it, preserve
its pattern projection, or prepare its derived compile companion. It may not
execute the original runtime body as a compile or meta entry.

`compile` and `meta` remain different internal capabilities: compile computes
static values and `PatternValue`; meta constructs `SymbolConstructionValue` in
a `MetaConstructionUnit`. They are grouped only by their shared external
compile stage.

There is no independent final return-policy `P3` and no scalar lookup policy
for the whole result symbol. The selected object's `P1` projects result material
layer by layer: Pattern currently contributes only its compile projection;
`Val1` contributes the compile leaves, and also runtime leaves when `P1` admits
runtime; every returned `Val2` object keeps its own `P1`. Current `self_policy`,
`body_entry_policy`, and `return_object_policy` fields are transitional
implementation substrate for parts of this model, not three normative source
positions.

This lookup/entry separation is load-bearing for candidate qualification,
partial versus strict demand, compile-flow projection, and residualization.

## 3. Candidate pipeline

Invocation is resolved through a symbol-first pipeline of progressively narrower
candidate pools. Each layer adds one kind of constraint and never re-opens an
earlier decision.

```text
resolve name/path:
  -> Symbol

project value facet:
  -> zero or more heterogeneous values

stage-visible object pool:
  filter each enumerated Val2 object by its own P1

call-entry candidate pool:
  obtain each stage-visible value's type
  -> resolve the type-associated `()` entry
  -> discard non-callable entries
  -> include stable derived callable entries

compile-projected call site:
  retain an ordinary projected call
  -> do not select a concrete object

fully admissible candidate set A:
  call-entry pool + every hard structural/Pattern/type/require check
  + P2 execution-stage compatibility
  + exact total-policy equality for self and explicit arguments
  + expected result rank/facet compatibility

selected result:
  A -> ordinary preference filters -> final survivor set
  -> unique candidate satisfying must-select postconditions
```

Reading the layers from the top:

- **Symbol resolution** produces a first-class symbol, then projects its value
  facet. The facet may contain heterogeneous callable and non-callable values.
- The **stage-visible object pool** filters each enumerated `Val2` object by its
  own `P1`; this does not rerun or condition base symbol resolution.
- The **call-entry candidate pool** obtains each value's type and resolves the
  type-associated `()` entry. Non-callable values are valid facet material but
  are discarded for this call position.
- The **fully admissible set `A`** keeps only those callables whose parameter
  patterns and rank-directed symbol/type/pattern-value expectations are
  compatible with the actual argument shapes, whose `P2` has the demanded
  external stage, whose implicit self and explicit arguments have exactly that
  total policy, and whose hard concept/require/result checks pass.
- The **preference filters** apply entry preference, concept ordering,
  extraction specificity, and first-order preference only after full
  admissibility and in one fixed normative order. Each filter is independent
  of candidate enumeration/source declaration order; filters are not assumed
  to commute.
- The **must-select postcondition** is computed from `A`. An admissible derived
  compile companion must be the final unique survivor; a more specific normal
  overload cannot silently replace it.

Same-name value entries are not assumed to be same-type function overloads.
They may have unrelated types and become comparable only after their own
type-associated call entries have been prepared.

The current implementation realizes this pipeline only for two narrow paths:
the earlier core-meta/source-verification path, and the v0.8 restricted
source-declared meta-overload path. The v0.8 path has argument-shape matching,
restricted parameter-pattern applicability, body-entry filtering, and selected
simple-body evaluation, but not full runtime overload resolution, concepts,
exact `P2` invocation-frame policy checks, compile
companions, must-select enforcement, guarded branch execution, or arbitrary
meta block interpretation.

A formal sketch of the intended end-to-end frame:

```text
Gamma; lookup_stage |- callee_path => Symbol
Gamma; lookup_stage |- value_facet(Symbol) => V*
Gamma; lookup_stage |- P1_filter(V*) => V_stage*
Gamma; lookup_stage |- type(V_stage*) / () => C0
Gamma |- explicit_user_product => ArgShapes
Gamma |- InvocationFrame(self, ArgShapes) => Frame
Gamma; lookup_stage |- FullyAdmissible(C0, Frame, expectation) => A
Gamma |- PreferenceFilters(A) => B_final
Gamma |- MustSelectConsistent(A, B_final) => selected_callable
Gamma; P2(selected_callable) |- invoke(...) => InvocationResult
```

This sketch is the target for general invocation. v0.8 proves the path for a
restricted source-declared meta-overload subset and leaves the omitted layers
explicitly deferred.

The invocation frame owns self injection. The callable formal frame has slot 0
for the function-object self-position and slots 1..n for user parameters.
`ArgShapes` describe only the explicit user product; adding self to product
arity, product flattening, canonical argument products, or meta instance keys is
a boundary violation.

### 3.1 InvocationFrame and CallableFrameShape substrate

The first implementation substrate for this boundary is:

```text
ProductObject / ArgProductShape
  -> InvocationFrame
  -> CallableFrameShape
  -> later callable body entry
```

The explicit argument product is shaped first as `ProductObject` /
`ArgProductShape`. The invocation frame then injects `self` into formal slot 0.
`self` is not part of `ArgProductShape`, `RawArgShape`, product arity, product
flattening, or canonical meta instance keys.

Zero-user-argument callables still have self slot 0 and an empty explicit
argument product. Declaration-context `()` call-entry definitions, such as:

```lang
let ()::ref::T = (self: T ref) => { ... }
```

use the same frame model as ordinary function values. They are symbol/overload
injections into an associated space, not a separate call mechanism. In that
shape, `self: T ref` still occupies formal slot 0, and the explicit user
argument product starts after self.

Closure/function body syntax is not immediately forced to materialize an object.
In value/call context it may materialize as a lambda / function-object value. In
declaration / symbol-injection context it may elaborate as a call-entry
definition or overload candidate.

A function-value binding such as:

```lang
let name = (self) => { ... }
```

can be modeled as synthesizing an anonymous function-object type, injecting `()`
into that type's associated space, and then binding the resulting
function-object value to `name`. This is a future elaboration direction, not
implemented by the current substrate.

The current v0.8 direct-callable shortcut may use a placeholder
`InvocationFrame` until full target value → target type → `()` call-entry
resolution exists. The current `InvocationFrame` substrate also does not
elaborate declaration-context `()` call-entry forms. It only records the frame
invariant: self is slot 0 and the explicit user argument product remains
separate. It does not implement runtime invocation, full overload resolution,
return execution, D/Done, lifetime checking, or implicit `?`.

## 4. Partial meta reduction versus strict meta execution

Evaluation demand is orthogonal to execution capability and result rank:

```text
execution capability: compile | meta | runtime
evaluation demand:     partial | strict
result rank:           PatternValue | SymbolConstructionValue | runtime value
```

`MetaPartialContext` and `MetaStrictContext` retain their existing purpose: they
say whether a runtime boundary may residualize. They do not define `meta`, do
not turn `compile` into symbol construction, and do not change the rank of a
successful result.

A call site is reduced in one of two demand contexts, and the difference between
them is what makes one invocation framework cover both compile-time reduction
and residual runtime behavior.

```text
MetaPartialContext
  The evaluator attempts to reduce as much as policy and available meta values allow.
  A runtime-only boundary may suspend and produce a residual expression.

MetaStrictContext
  The evaluator must complete under meta policy.
  If lookup or execution requires a non-meta candidate, this is a hard diagnostic.
```

### 4.1 Invocation layer: capability-directed results

The invocation layer evaluates or reduces a callable under policy and a demanded
execution capability. A successful compile-time result is not merely a
`TypeValueId`:

```text
compile callable -> PatternValue
meta callable    -> SymbolConstructionValue : symbol
runtime callable -> runtime value
```

`PatternValue` includes ordinary compile-time values, type values, and
structured pattern values. A type value is not thereby an installed type
symbol. `SymbolConstructionValue` carries symbol/facet/pattern construction
material but remains uninstalled.

The exact capability split, canonical `MetaInstanceScope`, result-symbol/return-
slot relation, rank-directed identity, type self-root validation, and complete
navigation atom belong to
`spec/design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md`.
This document consumes those result ranks only to define candidate preparation,
policy filtering, and partial/strict reduction; it does not restate their
construction semantics.

The public future boundary is conceptually:

```text
InvocationResult =
  | PatternValue(...)
  | SymbolConstructionValue(...)
  | RuntimeValue(...)
  | Residual(expr, suspension_reason)
  | Diagnostic(error)
```

The current Rust substrate still uses `MetaInvocationResult::Value` with
`MetaInvocationValue::{ForwardedValue, GeneratedConstructionValue,
GeneratedTypeDefinitionValue}`. Those cases describe transitional v0.8/v0.9
implementation transport, not the final public rank model. It also does not
implement `MetaInstanceScopeId`, meta return type self-root checking, complete
`compile`/`meta` separation, or the canonical meta-instance navigation atom.

The canonical note also owns the complete invocation navigation atom; this
document assumes that atom has already been resolved before candidate identity
and caching are finalized.

Namespace graph installation is not part of formal invocation. Binding or
injection consumes a construction value, resolves a writable `PlaceId`, forms a
`NamespaceDelta`, and installs it atomically. Internal control-state vocabulary
must stay below the invocation boundary.

When candidate preparation or invocation cannot proceed, diagnostics should name
the current semantic boundary:

```text
NoMetaVisibleCandidate
RuntimeOnlyValue
BodyEntryPolicyMismatch
UnresolvedUnderMeta
ExecutionRequiresRuntimePolicy
```

Demand controls failure versus residualization:

```text
No admissible compile/meta candidate:
  partial => residualize when legal
  strict  => error

Visible ambiguity or construction conflict:
  both demands => error

Candidate exists but final P2 (current substrate: body-entry policy) rejects
the demanded capability:
  partial => residualize when legal
  strict  => error
```

Ambiguity and conflict are errors under both demands: a residual defers one
well-identified call; it does not defer candidate choice.

### 4.2 Expansion / binding layer

After the invocation layer produces a `PatternValue` or
`SymbolConstructionValue`, the expansion / binding layer applies it to a build
or declaration context. This includes:

```text
- installing a NamespaceDelta atomically;
- binding the declared target (e.g. `let T: type = ...`);
- exposing an extraction-facing interface on the constructed value;
- applying layer-directed `result_projection_by_P1` without assigning one
  scalar policy to the result symbol.
```

This separation is intentional: invocation produces an uninstalled value, and
expansion is a side-effecting binding operation that consumes that value. For a
symbol construction, the layer must preserve both `pattern_owner(V)` and the
independently resolved `install_place(V)`; binding must not reroot the pattern
owner. Conflating invocation and binding into one `MetaExpansionResult` is
acceptable as a current temporary transport but must not harden into the
permanent model.

### 4.3 IdentityType is a placeholder proof path only

`IdentityType` proves graph-resolved invocation plumbing: it demonstrates that
a prepared candidate can flow through the candidate preparation, key
computation, cache lookup, and primitive reduction pipeline. It does **not**
prove final `PatternValue` or `SymbolConstructionValue` semantics.

```text
IdentityType proves:
  graph-resolved target lookup;
  normalized call-site extraction;
  argument product shaping and classification;
  candidate preparation and policy checking;
  formal meta invocation dispatch;
  canonical key computation and cache memoization.

IdentityType does NOT prove:
  PatternValue computation under compile capability;
  SymbolConstructionValue production under meta capability;
  MetaInstanceScopeId or return TypeFacet self-root validation;
  rank-directed symbol/type/value parameter identity;
  declaration binding from arbitrary meta return values;
  extraction-facing interface exposure;
  ordinary generic type constructor behavior.
```

Any implementation, test, or document that uses `IdentityType` as evidence that
ordinary `PatternValue` / `SymbolConstructionValue` semantics have been
implemented is incorrect.

## 5. Match and If Share One Pattern Mechanism

The language has no semantic split among:

```text
match
constexpr match
if
if constexpr
```

There is one pattern-matching mechanism. `if` / `else` is the two-pattern case:

```text
match cond {
  true  => ...
  false => ...
}
```

Surface spellings remain ordinary syntax/call material until later semantic
interpretation; this does not require parser keywords or a privileged
`IfExpr`. The stage of branch selection follows the matched symbol's total
policy:

```text
total_policy(scrutinee) = compile
  -> normal compile evaluation selects the branch

total_policy(scrutinee) = runtime
  -> the Pattern projection remains in CompileFlow
  -> actual branch selection remains runtime
```

Pattern-space subtraction and completed-result isolation are not invocation
strategies invented here. Complete symbol flow already represents match through
the canonical extraction residual `D(A, S)` and `Done` semantics in
`../patterns-overload/static-pattern-spaces-and-extraction-chains.md`. Compile
projection preserves those constructors homomorphically, and automatic require
consumes the retained structure as specified in
`../symbol-world/symbol-policy-and-compile-flow-projection.md`.

For a compile-policy scrutinee, only the selected guarded branch is evaluated
in normal compile evaluation. For a runtime-policy scrutinee, each possible
runtime branch retains a guarded complete block; automatic require conjoins
those guarded contracts rather than evaluating an unguarded Boolean conjunction
of branch bodies. Exact guarded-atom fields and identity are not frozen.

`partial` versus `strict` still controls whether an unresolved runtime boundary
may residualize. It does not create a separate constexpr control language.

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

## 7. Ordinary meta and privileged AST meta are not macro expansion

Meta callable objects operate on structured objects, not on text. The semantic
classification is:

```text
MetaFunction
  |- OrdinaryMetaFunction
  `- BuiltinPrivilegedAstMetaFunction
```

An ordinary user meta function is invoked like any other callable and receives
ordinary symbol, type, PatternValue, or other rank-constrained inputs. It does
not acquire unrestricted AST-consuming capability merely because `P2 = meta`.

A compiler-defined `BuiltinPrivilegedAstMetaFunction` may additionally accept
a specifically bounded normalized-AST or pattern-material rank under an
explicit ambient construction capability. Each built-in defines its own scope,
owner, input, and result rules. Users may call such an object but cannot define
new privileged members or infer a general rewrite facility from one built-in.

```text
Neither class receives raw text by default.
Neither class grants arbitrary token splicing or parser re-entry.
```

A meta object may produce:

```text
a SymbolConstructionValue with symbol/facet/pattern construction material
a residual expression
a diagnostic
```

Graph deltas and declaration bindings belong to the expansion/binding layer
(§4.2), not the ordinary returned-value layer.

A `compile` callable uses the same invocation framework but produces a
`PatternValue`, not symbol construction. The distinction is an execution
capability/result-rank boundary, not a second parser or expression language.

Both classes still participate in symbol-first lookup, function-object/type/
associated-`()` preparation, and the general invocation framework. There is no
textual substitution or general macro expansion. The bounded behavior of a
privileged built-in is a compiler-known semantic capability, not permission for
arbitrary AST rewriting.

This is why the front end must stay neutral:

```text
Parser and normalizer should not special-case names like `struct`, `verify`, `cond`,
`self`, `Self`, `return`, or future predicate operators. They should preserve
normalized structure, including `()` call-entry material. Later graph lookup,
invocation-frame construction, and policy-governed invocation decide what those
names or call entries do.
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
- `struct` is a core `BuiltinPrivilegedAstMetaFunction` symbol resolved through
  the namespace graph, not a parser keyword or an ordinary user-definable meta
  function.
- `verify` is a core meta-visible verification namespace/object, with
  verification operations installed below it as core symbols.
- Source-declared callable/meta-function overloads can be harvested into graph
  symbols and selected by the restricted v0.8 overload path.
- Formal `struct` invocation still produces anonymous
  `GeneratedTypeDefinitionValue` pattern heads. Ordinary binding preserves those
  provisional heads or restores stripped material under the anonymous
  `GeneratedTypeDefinition` fallback; it does not derive owner context from the
  destination. The doc-hidden explicit helper still exposes
  generated/global/namespace/local categories only as transitional registry and
  integration-test substrate, not as a stable owner-construction API.
- The current restricted evaluator still recognizes the legacy `r === ...`
  forwarding body. The final model replaces that formal return split with
  `r = ...` producing a `SymbolConstructionValue`; ordinary `let ===` aliasing
  remains separate.
- `PolicyEnv::Meta` and `PolicyEnv::Runtime` support visibility metadata; the
  restricted overload selector also checks the selected current body-entry field before
  meta execution.
- The current early-meta, verification, and v0.8 overload behavior are not yet
  the full invocation model; they are bounded vertical slices.

Not yet present are `SymbolCell` facets, `PatternValue` as the compile result
model, `SymbolConstructionValue` as the meta result model,
`ResolvedPatternScope`, final binding-independent `struct` owner resolution, or
functional `inject`.

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

Full candidate selection — fully admissible set `A` and preference filters of
Section 3 —
depends on machinery that this document does not define. Argument shape,
normalized parameter pattern compatibility, and first-order type-value
compatibility are prerequisites for real overload selection. This document
defines the invocation frame only; it does not define the pattern system or the
type-value system.

For v0.8-adjacent compile/meta construction, argument shape means the
contract-shaped route through `ProductObject` / `ArgProductShape`, not
callee-specific parsing of raw normalized product material. Canonical meta
instance keys must be computed only after product canonicalization and
first-order `TypeValueId` argument compatibility are established. The detailed
construction guardrails live in
`spec/contracts/v0.8-meta-construction-agent-constraints.md`.

This argument shape is only the explicit user-supplied product. Function-object
self belongs to `InvocationFrame` / callable-frame slot 0 and is not a product
atom.

The companion boundaries are now explicit:

```text
symbol/facet + compile/meta + pattern-owner construction:
  `spec/design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md`

candidate pattern normalization:
  `pattern-normalization-and-first-order-overload.md`

layered policy / compile projection / companions / automatic require:
  `../symbol-world/symbol-policy-and-compile-flow-projection.md`

type/place/alias identity:
  `type-values-places-and-alias-forwarding.md`

later extraction/static pattern semantics:
  `static-pattern-spaces-and-extraction-chains.md`
```

This document owns invocation demand and policy framing. It does not redefine
the symbol-facet model, pattern-layer ordering, pattern-owner resolution, or
type/place/alias identities owned by those companion documents.

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
2. Introduce SymbolCell facets and symbol-first value-facet candidate lookup.
3. Introduce ProductObject / ArgProductShape and normalized pattern /
   argument-shape objects, with implicit self kept out of product shape.
4. Introduce PatternValue / TypeValueId identities and callable signature objects.
5. Introduce SymbolConstructionValue and rank-directed canonical instance keys.
6. Introduce final P1/P2 candidate qualification for compile/meta invocation.
7. Introduce ResolvedPatternScope and binding-independent `struct` ownership.
8. Move `struct` and `verify` dispatch behind the common invocation engine.
9. Add functional child-only `inject` without graph installation.
10. Add partial versus strict reduction over the intrinsic D/Done match flow.
11. Add mechanical compile-flow projection, derived companions, must-select,
    and shared inferred-require/body evaluation nodes.
12. Only after this, introduce runtime lookup over residual expressions.
```

Runtime lookup is deliberately listed last. It must not be pulled earlier than
the product/argument-shape, pattern, type-value, canonical-key, and
meta-invocation milestones: residuals are only well-formed once the invocation
engine that produces them exists, and runtime lookup is the consumer of those
residuals, not a parallel mechanism.
