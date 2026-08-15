# Meta Object Invocation and Policy Reduction

**Status: Mixed.** This remains the broader future invocation design. The
current implementation contains the earlier source-verification/core-meta path
plus a restricted v0.8 source-declared meta-overload invocation slice described
in §0.1.

The canonical future result-rank and construction boundary is
`spec/design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md`.
It supersedes the older formal-meta-return interpretation that used `r = ...`
for generation and `r === ...` for forwarding, and also supersedes the interim
single-form `r = ...` reading: the final model distinguishes
ordinary `let` creation, existing-place `=` writes, and return control transfer.
The current `let r = expr;` / `r = expr;` / `r;` construction-carrier mapping is
a compatibility encoding, not special long-term semantics of the return-slot
name. There is no fourth alias-member event — the semantic alias/forwarding
direction is retired.
References to the older splits below are
explicitly transitional implementation notes, not final semantics.
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

Supported/preserved selected implementation forms:

- bare delete or message delete returns
  `MetaInvocationResult::Diagnostic(...)`;
- ordinary and named user bodies share the same evaluator path; the
  transitional simple forwarding body, such as `{ r === t; }` or
  `{ r === unit; }`, returns
  `MetaInvocationResult::Value(MetaInvocationValue::ForwardedValue(...))` when
  the forwarded type-pattern value is available in the graph.
- `Defaulted` is preserved as a distinct compiler-generation request; the
  restricted evaluator diagnoses it until a callable-kind default generator is
  available.

Named strategy metadata is carried on the selected candidate only after
restricted applicability. This slice does not execute arbitrary named strategy
rules and does not grant `default` an implicit priority.

This `r === ...` behavior describes only the restricted v0.8 evaluator that is
currently implemented; it is a transitional spelling carried by the frozen
parser surface, not a semantic forwarding mechanism. The final formal meta model
does not grant the return-slot name special `let` semantics: ordinary `let`
creates a Symbol/member, ordinary `=` writes an existing place, and the return
event alone delivers control. The current `let r = ...` return-cluster behavior
is a transitional compatibility encoding of those separate operations. Where a
member must observe an external object, it holds a borrow view
(`ref` / `share`), which is an ordinary value — and a borrow edge is not owned
material, so it is not promoted at seal. Installing an external pure Object as
the meta return role member still fails the self-root invariant. No declaration form,
inside or outside a meta body, forwards a symbol or a place.

Unsupported selected body forms return hard diagnostics. In particular, a body
that requires guarded branch evaluation, predicate calls, postfix `?`,
short-circuit behavior, D/Done reduction, or a full meta block interpreter is
outside this v0.8 slice. `delete` is not a value and there is no
`CoreMetaFunction::Delete`.

The callable result policy pair is read from the selected closure/function
head. The function-object stage view is then derived from that P2 pair, and an
explicit declaration prefix performs ordinary P1 projection. For example:

```lang
meta || runtime let + =
  (self, t: type, u: type): meta -> let r: type =>
{
  r === t;
};
```

has canonical `P2 = meta:meta`. Its derived function-object stage view is meta,
so the written P1 query `meta || runtime` selects only the available meta slice;
it does not manufacture runtime visibility. Current flat symbol,
`body_entry_policy`, and `return_object_policy` metadata therefore all carry
the selected meta compatibility projection for this example. These fields are
transitional transport, not three final source-level policy positions.

This example contributes a non-empty projected meta slice and therefore does
not enter atomic runtime-migration preparation. The written runtime alternative
is an accepted query alternative, not a missing value obligation. The current
migration resolver is only a candidate-ordering prototype; initializer and
ordinary function-object routing remain future integration.

Conversely, a single P2 `runtime` normalizes canonically to `runtime:compile`.
Current flat transport records the runtime value stage but does not yet install
the compile Pattern stage as a first-class graph facet.

The example's first written formal, spelled `self`, denotes callable-frame slot
0. This is a positional rule rather than a reserved-name rule: any first
written formal has the same self role. Its actual caller object is injected
by invocation after callable resolution; it is not part of the call-site
explicit product, `ProductObject`, `ArgProductShape`, or `RawArgShape`. The
explicit user product for the example contains only the user-supplied positions
after the first written formal.

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
default ordinary initializer strategy = meta || runtime / MetaPartial
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

Final policy-binding semantics uses the pair carried by RHS result entries:

```text
Gamma |- expr : (tau, Pv:Pp)
Gamma |- ProjectP1(P1, result(expr)) = selected
selected is non-empty
------------------------------------------------
Gamma |- P1 let x = expr
```

Omitting P1 requests full inference. A single written P1 queries the value
component and follows each selected value's associated pattern component. A
pair P1 filters both components. No form adds a universal non-runtime
condition:

```lang
let x = expr;                 // retain the inferred complete result
meta || runtime let x = expr; // select available meta/runtime value slices
runtime let x = expr;         // select runtime values and follow their types
```

The current binding substrate applies a non-empty stage-slice projection rather
than exact flat-flag verification. Thus an expression that residualizes only to
runtime may satisfy `meta || runtime let x = expr` by selecting its runtime
slice. Full `Pv:Pp`, const/mut, namespace, and value-presence transport through
the namespace graph remains future work. Ambiguous meta-visible candidates
remain hard diagnostics in both `MetaPartial` and `MetaStrict`; ambiguity is
not residualized.

The current adapter consumes provisional RHS result-stage metadata. Direct
type-name forwarding uses the forwarded symbol's current metadata, while
restricted source-callable invocation uses transitional
`return_object_policy` transport. This is neither P3 nor a whole-result
policy. Final semantics retains each value/pattern entry's `Pv:Pp`, and every
returned Val2 object retains its own pair.

When P1 is omitted and RHS evaluation succeeds, the current adapter infers its
available stage slice for the materialized binding. Namespace export visibility
is not copied from a forwarded dependency; namespace visibility belongs to the
destination's namespace-scoped P1 declaration.

If an initializer residualizes and the binding has a closed annotation such as
`: type`, the ordinary result-as transformation residualizes with the
expression. Because v0.8 has no deferred/runtime result-as implementation, the
current substrate reports the legacy-named diagnostic:

```text
UnsupportedDeferredTypeAssertion
```

That diagnostic name does not redefine the annotation as a Boolean assertion.

This diagnostic means the closed result-as boundary is unsupported for
residual initializers; it does not mean the RHS was already transformed and
found invalid.

The v0.8 success path for `let X: type = int + unit;` is:

```text
source declarations install real `+` overload symbols
ordinary initializer sees normalized `int + unit`
MetaPartial invokes restricted overload selection under MetaAction lookup
selected `(self, t: type, _ unit: type): meta -> ...` body (legacy substrate
shape) forwards `t`
RHS value is `ForwardedValue(int)`
hole-free `: type` applies the ordinary result-as-`type` transformation
binding materialization installs `X` as a fresh symbol/place carrying the
complete immutable `int` type closure `tau = <Q,V_T>`, optionally written
`bind alpha.<Q,V_T[alpha]>`; `Q`
satisfies `TypeRole(Q)`, and the
binding is an ordinary fresh symbol and place,
not a forwarding of `int`'s own symbol or place
```

The current evaluator may still name its narrow check an “assertion”; that is
implementation substrate, not the target annotation semantics.

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
`compile` capability (computing a root-conserving `PatternValue` with no root
authority of the compile coordinate itself) or ordinary `meta` capability
(computing a `PatternValue` and establishing a navigable `M`). A privileged
builtin instead follows only its member-specific owner rule. Policy and
partial/strict demand determine whether
the callable may execute or residualize; they do not merge the two capabilities.

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

## 2. P1 Binding Projection and P2 Result Pair

The final callable form is:

```text
[P1] let F = (...): P2 -> let r => { ... }
```

P1 is the optional general binding projection. P2 is the call/expression result
pair. The causal direction is:

```text
P2 -> derived function-object P1
```

The judgments are:

```text
Gamma |- ResolveSymbol(path) => Symbol
Gamma; Phase |- ExposePolicySlice(typed_val_members(Symbol)) => Val2View*

Gamma |- invoke(selected object, InvocationFrame)
      => result(P2v:P2p)

Gamma |- derive_function_object_P1(P2v:P2p) => P1base
Gamma |- ProjectP1(written_prefix, P1base) => bound object view
```

Omitted P1 keeps the complete result. Single P1 `q` selects values visible
under `q` and follows their associated pattern components. Pair P1 `qv:qp`
filters both. Single P1 is not normalized to `q:q`. These lower-case policy
metavariables are distinct from the Symbol pure-role member `Q`.

P2 is an explicit pair or a context-specific single-policy shorthand:

```text
P2 = Pv:Pp

N2(P) = P:(P-runtime), when P-runtime is non-empty
N2(runtime) = runtime:compile
```

P2 pair validity requires:

```text
runtime not in Stage(Pp)
Static(Pv) is empty or Static(Pv) = Stage(Pp)
```

Function-object stages follow:

```text
Stage(P1p) = Stage(P2p)
Stage(P1v) = Stage(P2v) union Stage(P2p)
```

Only stages are lifted. Returned const/mut and namespace visibility do not
propagate to the function object. Slot 0 remains implicit `self`; slots 1..n
remain explicit source arguments. Declared parameter/receiver pair patterns are
checked across the full invocation frame without inventing an independent self
policy plane.

Visibility does not imply executability. A static view may inspect a
runtime-capable object's pattern component or derived companion, but may not
execute the original runtime value body as compile/meta.

`compile` and `meta` remain different capabilities: compile transports and
computes static values construction-transparently while generating no new root;
meta computes values and establishes a
MetaConstructionUnit root. OpenStatic exposes both meta and compile; SealStatic
exposes seal and compile but not meta. A single P2 runtime defaults to
`runtime:compile`; explicit `runtime:seal` remains available when the Pattern
must wait for SealStatic.

There is no independent P3 and no scalar policy for the whole result symbol.
Every value/pattern result entry retains `Pv:Pp`; every returned Val2 object
retains its own pair. Return positions inherit P1 and may refine mutability
only; parameters symmetrically refine inherited P2 mutability only. Current
`self_policy`, `body_entry_policy`, and
`return_object_policy` fields are transitional compatibility transport.

This separation is load-bearing for candidate qualification, partial versus
strict demand, compile-flow projection, and residualization.

## 3. Candidate pipeline

Invocation is resolved through a symbol-first pipeline of progressively narrower
candidate pools. Each layer adds one kind of constraint and never re-opens an
earlier decision.

```text
resolve name/path:
  -> Symbol

project typed val members:
  -> zero or more heterogeneous values

stage-visible object pool:
  observe the policy-projected view of each enumerated Val2 object

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
  + declared receiver/parameter pair compatibility for self and explicit arguments
  + P2 result compatibility with any target-result expectation
  + expected result rank/facet compatibility

selected result:
  A -> ordinary preference filters -> final survivor set
  -> unique candidate satisfying must-select postconditions
```

Reading the layers from the top:

- **Symbol resolution** produces a first-class symbol, then projects its value
  facet. The facet may contain heterogeneous callable and non-callable values.
- The **stage-visible object pool** observes each enumerated `Val2` object's
  available pair-projected view; this does not rerun base symbol resolution.
- The **call-entry candidate pool** obtains each value's type and resolves the
  type-associated `()` entry. Non-callable values are valid facet material but
  are discarded for this call position.
- The **fully admissible set `A`** keeps only those callables whose parameter
  patterns and rank-directed symbol/type/pattern-value expectations are
  compatible with the actual argument shapes, whose P2 pair admits the
  invocation frame (implicit self plus explicit arguments), and whose hard
  concept/require/result checks pass.
- The **preference filters** apply entry preference, concept ordering,
  extraction specificity, first-order preference, in-place-over-non-in-place
  preference, and named strategy rules only after full
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
Gamma |- ResolveSymbol(callee_path) => Symbol
Gamma; Phase |- typed_val_members(Symbol) => V*
Gamma; Phase |- ExposePolicySlice(V*) => V_phase*
Gamma; Phase |- type(V_phase*) / () => C0
Gamma |- explicit_user_product => ArgShapes
Gamma |- InvocationFrame(self, ArgShapes) => Frame
Gamma; Phase |- FullyAdmissible(C0, Frame, expectation) => A
Gamma |- PreferenceFilters(A) => B_final
Gamma |- MustSelectConsistent(A, B_final) => selected_callable
Gamma; P2pair(selected_callable) |- invoke(...) => InvocationResult
```

This sketch is the target for general invocation. v0.8 proves the path for a
restricted source-declared meta-overload subset and leaves the omitted layers
explicitly deferred.

The invocation frame owns self injection. The callable formal frame has slot 0
for the caller-object self-position and slots 1..n for explicit arguments.
The first source-written formal explicitly declares slot 0's Pattern under any
legal spelling; later formals align with the explicit Product positions. If no
formal is written, slot 0 still exists without a source binder.

The complete declaration path for a function object still requires
synthesizing its anonymous function-object type, injecting `()` into that
type's associated space, and then binding the resulting function-object value
to `name`. This is a future elaboration direction, not implemented by the
current substrate.

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
execution capability: compile / meta / seal / runtime
evaluation demand:     partial | strict
result rank:           PatternValue | runtime value
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
compile callable -> PatternValue, construction-transparent, root-conserving,
                    and with no root authority
ordinary meta callable
                  -> symbol PatternValue, plus authority to establish and seal
                     one navigable MetaInstanceRoot M
privileged builtin
                  -> PatternValue, with only its member-specific owner rule
runtime callable -> runtime value
```

`PatternValue` includes ordinary compile-time values, type values, symbol values,
and structured pattern values. A type value is not thereby an installed type
symbol. The `compile` / `meta` difference is world authority, not result rank:
there is no third rank, and the meta result is an ordinary value of type
`symbol`. What the current implementation calls `SymbolConstructionValue` is
the transitional carrier for that ordinary Symbol's multi-member material; it
remains uninstalled and does not define another ontology.

Two names keep classifier and content shape distinct:

```text
ReturnClassifier(ordinary meta) = symbol
ReturnShapeWithinSymbol         = Σ = ⟨ Q?, V ⟩, with |Q| <= 1 and Pure(Q)
```

`Q` may be present or absent and `V` may contain any ordinary sibling values.
Those are content facts within one returned Symbol Object, not type/val/namespace
return categories. `Q` is the one optional pure role member: namespace
projection selects it directly, while type projection additionally requires
`TypeRole(Q)`. “Meta returns type” is only shorthand for a `symbol` result whose
content is `⟨Q, empty⟩` and whose `Q` satisfies `TypeRole`.

The exact capability split, canonical `MetaInstanceScope`, result-symbol/return-
slot relation, rank-directed identity, pure-role self-root validation, and complete
navigation atom belong to
`spec/design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md`.
This document consumes those result ranks only to define candidate preparation,
policy filtering, and partial/strict reduction; it does not restate their
construction semantics.

An ordinary meta invocation additionally forms a globally reusable instance key:

```text
MetaInstanceKey(F, args)
  = MetaCallableIdentity(F) × Addr(Product(Canonicalize(args)))

well formed only if:
  forall a in Canonicalize(args): GlobalKeyable(a)
```

The canonical owner separates callable kind, call admissibility, and effect:
`OrdinaryMetaFunction(F)` fixes `P2(F)=meta` and result classifier `symbol`;
`WellFormedMetaCall(F,args)` contains admissibility plus `GlobalKeyable`; only a
well-formed call establishes `RootIdentityExists(M)` and construction-local
navigation. Root identity is not external namespace installation.

`GlobalKeyable` is a dependency condition evaluated at key-creation time, not a
source-location condition. A meta-local binder may hold and pass a value whose
dependencies are already globally stable or already promoted. A fresh ephemeral
PatternValue dependency created inside the current meta invocation may not enter
another `MetaInstanceKey`; promotion that may occur only when an enclosing meta
later seals cannot justify an inner key now. Horizontal borrow targets are also
global-key dependencies even though they are excluded from owned recursive
normalization: `GlobalKeyable(Borrow(q))` requires `q` to be already globally
stable. `compile` and transparent construction intrinsics do not impose this
boundary because they form no MetaInstance key and generate no root.

The public future boundary is conceptually:

```text
InvocationResult =
  | PatternValue(...)
  | RuntimeValue(...)
  | Residual(expr, suspension_reason)
  | Diagnostic(error)
```

The current Rust substrate still uses `MetaInvocationResult::Value` with
`MetaInvocationValue::{ForwardedValue, GeneratedConstructionValue,
GeneratedTypeDefinitionValue}`. Those cases describe transitional v0.8/v0.9
implementation transport, not the final public rank model. It also does not
implement `MetaInstanceScopeId`, meta return pure-role self-root checking, complete
`compile`/`meta` separation, or the canonical meta-instance navigation atom.

The canonical note also owns the complete invocation navigation atom; this
document assumes that atom has already been resolved before candidate identity
and caching are finalized.

Namespace graph installation is not part of formal invocation. `let` consumes a
value and creates a destination/member through a `NamespaceDelta`; ordinary `=`
writes an existing place. Place-level `inject` is a bounded read--extend--write
operation on one existing type slot and creates no member or root. Internal
control-state vocabulary must stay below the invocation boundary.

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

Candidate exists but current transitional body-entry metadata rejects the
demanded capability:
  partial => residualize when legal
  strict  => error
```

Ambiguity and conflict are errors under both demands: a residual defers one
well-identified call; it does not defer candidate choice.

### 4.2 Expansion / binding layer

After the invocation layer produces a `PatternValue`, the expansion / binding
layer applies it to a build
or declaration context. This includes:

```text
- installing a NamespaceDelta atomically;
- binding the declared target (e.g. `let T: type = ...`);
- exposing an extraction-facing interface on the constructed value;
- applying context-directed `ProjectP1` to value/pattern result entries without
  assigning one scalar policy to the result symbol.
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
prove final `PatternValue` or meta-construction semantics.

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
  meta root establishment and sealing under meta capability;
  MetaInstanceScopeId or returned type-role member self-root validation;
  rank-directed symbol/type/value parameter identity;
  declaration binding from arbitrary within-Symbol meta return shapes;
  extraction-facing interface exposure;
  ordinary generic type constructor behavior.
```

Any implementation, test, or document that uses `IdentityType` as evidence that
ordinary `PatternValue` / meta-construction semantics have been
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
`IfExpr`. The stage of branch selection follows the selected scrutinee value
component, while its pattern component remains statically available:

```text
Pv(scrutinee) selects a static value stage
  -> normal compile evaluation selects the branch

Pv(scrutinee) selects a runtime value stage
  -> Pp/Pattern remains in CompileFlow
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

The later runtime phase continues already-resolved residual computation and
performs the runtime checks that could not complete statically. Residualization
does not reopen an already resolved Symbol path, callable identity, or ordinary
overload choice merely because runtime values become available:

```text
Resolve once
Evaluate progressively
Residualize runtime dependencies
Continue the same resolved computation at Runtime
```

This is a minimal correction to the older loose phrase “runtime lookup over
residual expressions,” which did not distinguish consuming a resolved
residual from redoing namespace/candidate resolution. Any future explicit
dynamic dispatch or unresolved runtime-name mechanism requires its own design;
it cannot be inferred from mixed-stage residualization.

The exact residual IR, mixed-stage InvocationFrame representation, runtime
continuation ABI, effect sequencing, and all OpenStatic/SealStatic/Runtime
handoff details remain deliberately unfrozen.

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
a pattern value with symbol/facet/pattern construction material
a residual expression
a diagnostic
```

Graph deltas and member creation belong to the expansion/binding layer (§4.2),
not the ordinary returned-value layer. Place-level `inject` writes one already
existing type slot; it creates no member and installs no new root.

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
  forwarding body. The final model replaces that formal return split with the
  construction-effect family (`let r = expr;` fresh member, `r = expr;` write,
  `r;` delivery terminal) producing an ordinary Symbol PatternValue (current
  carrier: `SymbolConstruction`); the legacy `===`
  spelling has no successor, because the semantic alias/forwarding direction is
  retired.
- The compatibility `PolicyEnv` now has exactly OpenStatic, SealStatic, and
  Runtime variants; it projects flat visibility metadata while the restricted
  overload selector also checks the
  selected current body-entry field before meta execution. These environments
  are not canonical pair projection or execution permission.
- The current early-meta, verification, and v0.8 overload behavior are not yet
  the full invocation model; they are bounded vertical slices.

Not yet present are Symbol `Q` role projection / implementation caches,
`PatternValue` as the single static
result model, meta root establishment/sealing,
`ResolvedPatternScope`, final binding-independent `struct` owner resolution,
pure `extend`, or place-level `inject`.

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
  `type-values-places-and-borrow-views.md`

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
- further parser-semantic special cases
- complete policy lattice
- effect checking
- borrow checking
- ABI lowering
- code generation
- package dependency solving
```

## 12. Future implementation milestones

The model is expected to be reached in stages. The ordering matters: runtime
continuation over resolved residual expressions must come last, after the
pattern, type-value, and meta-invocation machinery exists.

```text
1. Keep current `struct` and `verify` behavior as implemented vertical slices.
2. Introduce Symbol `Q` role projection and typed value-member candidate lookup.
3. Introduce ProductObject / ArgProductShape and normalized pattern /
   argument-shape objects, with implicit self kept out of product shape.
4. Introduce PatternValue / TypeValueId identities and callable signature objects.
5. Introduce the meta-construction carrier and rank-directed canonical instance
   keys.
6. Carry canonical `Pv:Pp` through candidate qualification and every
   invocation-frame slot.
7. Introduce ResolvedPatternScope and binding-independent `struct` ownership.
8. Move `struct` and `verify` dispatch behind the common invocation engine.
9. Add pure child-only `extend`, then the read--extend--write `inject` wrapper.
10. Add partial versus strict reduction over the intrinsic D/Done match flow.
11. Add mechanical compile-flow projection, derived companions, must-select,
    and shared inferred-require/body evaluation nodes.
12. Only after this, introduce runtime continuation over resolved residual
    expressions.
```

Runtime continuation is deliberately listed last. It must not be pulled earlier than
the product/argument-shape, pattern, type-value, canonical-key, and
meta-invocation milestones: residuals are only well-formed once the invocation
engine that produces them exists. A separately designed dynamic lookup
mechanism, if any, is not this continuation.
