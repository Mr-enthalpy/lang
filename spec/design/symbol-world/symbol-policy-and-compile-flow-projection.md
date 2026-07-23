# Symbol Policy and Compile-Flow Projection

Status: canonical design contract. The typed model in this document is the
normative policy algebra. Flat `PolicySet`/`PolicyFlag` values are transitional
transport only and must not define source semantics.

This document owns the complete chain:

```text
source policy syntax
  -> contextual elaboration
  -> PolicyPair
  -> symbol resolution
  -> phase-slice exposure
  -> binding/overload selection
  -> OpenStatic evaluation
  -> SealStatic evaluation
  -> Runtime binding and evaluation
```

## 1. Complete symbol flow and policy pair

Language computation remains one symbol flow:

```text
Symbol = Val1 × Pattern × Val2
```

The policy of a result is always a pair:

```text
Π = Pv:Pp

Pv  policy of Val1/the value component
Pp  policy of Pattern/the anonymous-type component
```

There is no scalar replacement for this pair and no third policy slot. A
result object carries its own `PolicyPair` when it re-enters the flow.

Policy dimensions are typed and orthogonal:

```text
stage                       meta / compile / seal / runtime
value mutability            const / mut
value presence              present / optional / absent
ordinary namespace visibility public / private
export-root attribute       yes / no
```

They are not members of one untyped atom bag. In particular, export-root and
ordinary visibility are independent.

## 2. Pattern alternative and policy operators

Single `|` belongs to Pattern alternative:

```lang
let bool = ((if | else) bool) |> struct;

let true === if::bool;
let false === else::bool;
```

Therefore:

```text
Pattern(bool) = if::bool | else::bool
true  is an alias of if::bool
false is an alias of else::bool
```

`true | false` is not a second Pattern space for `bool`.

Policy uses three different operators:

```text
||  choice within one policy dimension
+   conjunction of different orthogonal dimensions
:   value/Pattern pair separator
```

Precedence, from tightest to loosest, is:

```text
||  >  +  >  :
```

Thus:

```text
const + runtime || compile : compile
```

means:

```text
const + (runtime || compile) : compile
```

### 2.1 Policy grammar

```text
PolicySpec
  ::= PolicyConjunction
   |  PolicyConjunction ":" PolicyConjunction

PolicyConjunction
  ::= PolicyChoice
   |  PolicyConjunction "+" PolicyChoice

PolicyChoice
  ::= PolicyAtom
   |  PolicyChoice "||" PolicyAtom

PolicyAtom
  ::= Name
   |  "(" PolicyConjunction ")"
   |  AbsentValuePattern
```

The parser is a strong-context parser. `meta`, `compile`, `seal`, `runtime`,
`public`, `private`, and `export` remain ordinary names to the lexer. The token
spelling for `AbsentValuePattern` is provisional; implementation fixtures may
use `S`, but source spelling is not frozen.

### 2.2 Algebra

`||` selects alternatives within one dimension:

```text
runtime || compile
meta || compile
const || mut
runtime || S
```

It is not arbitrary clause-level Boolean disjunction. These are invalid:

```text
runtime || const
compile || public
mut || export
(const + runtime) || (mut + compile)
```

`+` combines different dimensions:

```text
const + runtime
mut + (runtime || compile)
public + compile
```

It cannot combine mutually exclusive values in one dimension:

```text
const + mut
public + private
```

The syntax and normalized AST retain `PolicyPair`, `PolicyConjunction`,
`PolicyChoice`, `PolicyAtom`, and `AbsentValuePattern`; `||` and `+` are never
lowered to the same set insertion operation.

## 3. Contextual elaboration of P1

Three policy contexts can occupy a binding-shaped surface slot, but use three
different elaborators.

### 3.1 Ordinary binding projection

```lang
[P1] let x = expr;
```

Omitted P1 retains the complete inferred RHS result. A single policy is a
value-dominant projection:

```lang
Q let x = expr;
```

It selects the value slice exposed by `Q`, then preserves the Pattern component
associated with that selected value slice. It does not mean `Q:Q`.

An explicit pair constrains both components:

```lang
Qv:Qp let x = expr;
```

Projection returns an identity-preserving restricted view. Given:

```text
Pv = compile || runtime
Pp = compile
```

the projection `runtime` produces:

```text
Pv = runtime
Pp = compile
```

It must not return the original `compile || runtime` entry. Symbol identity and
Pattern identity do not change; only the visible slice is cropped.

### 3.2 Formal parameter policy pattern

In a formal parameter:

```lang
const let x
mut let x
let x
```

the prefix is a formal policy pattern, not a binding slice query. Opposite
const/mut qualifiers remain in the fully admissible set and are compared only
by the overload product order in section 12.

Every written formal parameter first inherits the callable result pair `P2`
without reinterpretation:

```text
FormalBase(parameter) = P2(callable)
```

Omitting the prefix preserves that pair exactly. Writing `const` or `mut`
restricts only the value-mutability axis of the inherited pair:

```text
let x        -> FormalPattern(P2, mutability = unspecified)
const let x  -> FormalPattern(const + P2)
mut let x    -> FormalPattern(mut + P2)
```

Stages, value presence, and the Pattern component remain byte-for-byte the
inherited P2 dimensions; the qualifier may neither shrink nor widen them.
`public`, `private`, `export`, stage atoms, value absence, and an explicit pair
are therefore invalid formal prefixes. If P2 already explicitly restricts its
mutability domain, a contradictory formal qualifier is invalid rather than an
expansion.

The const/mut singleton above is a formal Pattern and preference input. It is
not an ordinary P1 query applied to the actual argument. Consequently an
oppositely qualified actual is not removed before the product order: for a
const actual the order remains `const > unspecified > mut`, and it reverses
for a mut actual.

The elaborated formal pair is not body-local policy metadata. Candidate
formation exports its written/inherited mutability Pattern into the callable's
external parameter-policy position:

```text
FormalPolicyPattern(parameter)
  -> Candidate.parameter_policy[position]
  -> MaxPolicyProduct
```

Thus the P2-derived restriction affects both the view available inside the
callable body and comparison of this callable against other fully admissible
overloads. “Inherit P2” must not be implemented by updating only the body
environment while leaving the candidate externally `unspecified`.

### 3.3 Namespace declaration attributes

`public`, `private`, and `export` are accepted only by namespace-declaration
elaboration. They are rejected in ordinary P1, formal parameters, return
slots, P2, Pattern interiors, expression policies, and local declarations that
are not namespace declaration positions.

`export` has the narrower placement rule described in section 9.

## 4. P2 normalization

P2 is the result pair of a call or expression:

```text
P2 = P2v:P2p
```

Explicit pairs include:

```text
runtime:compile
runtime:seal
(runtime || compile):compile
(runtime || seal):seal
const + (runtime || compile):compile
```

`runtime` is forbidden in P2p. If P2v contains a static stage, its static stage
set must equal P2p. Consequently these are invalid:

```text
runtime:runtime
compile:seal
meta:compile
```

For a single policy `P`:

```text
Pv = P
Pp = P - runtime
```

If that subtraction is empty, Pp is `compile`:

| Source P2 | Normalized pair |
|---|---|
| `meta` | `meta:meta` |
| `compile` | `compile:compile` |
| `seal` | `seal:seal` |
| `runtime` | `runtime:compile` |
| `runtime || compile` | `(runtime || compile):compile` |
| `runtime || seal` | `(runtime || seal):seal` |

`runtime:seal` remains a valid explicit pair; it means that the value is a
runtime value whose Pattern/type is first exposed during SealStatic.

## 5. Function-object P1 derivation

For `P2 = P2v:P2p`, lift only stages:

```text
Stage(P1p) = Stage(P2p)
Stage(P1v) = Stage(P2v) || Stage(P2p)
```

Examples:

```text
P2 runtime:compile -> P1stage (runtime || compile):compile
P2 runtime:seal    -> P1stage (runtime || seal):seal
P2 meta:meta       -> P1stage meta:meta
```

The following never propagate from P2 to the function object:

```text
const / mut
public / private
export-root
value presence
```

Those properties come only from the function object's declaration.

For a declaration such as:

```lang
let fn = () => { ... };
```

the declaration supplies an empty value-mutability restriction. In the typed
policy domain, empty here means the complete `const || mut` domain, not “no
value” and not an unknown third qualifier. A written declaration P1 may crop
that domain to `const` or `mut`. P2 mutability never propagates into the
function object during stage lifting.

## 6. Three execution phases

```text
Phase = OpenStatic | SealStatic | Runtime
```

Stage visibility is defined by domains:

```text
Vis(meta)    = { OpenStatic }
Vis(seal)    = { SealStatic }
Vis(compile) = { OpenStatic, SealStatic }
Vis(runtime) = { Runtime }
```

| Policy stage | OpenStatic | SealStatic | Runtime |
|---|:---:|:---:|:---:|
| `meta` | yes | no | no |
| `compile` | yes | yes | no |
| `seal` | no | yes | no |
| `runtime` value | no | no | yes |

`compile` being visible during SealStatic does not make `compile` equal to
`seal`. Exposure checks ask whether the current phase is in `Vis(stage)`; they
do not intersect atom spellings.

## 7. Resolution, exposure, and facet reads

Every phase distinguishes:

```text
ResolveSymbol(path)
ExposePolicySlice(symbol, phase)
ReadValue(slice)
ReadPattern(slice)
EnumerateValueFacet(slice)
EnterCallableBody(candidate)
```

Failure to expose a value slice is not an unresolved symbol. In particular:

```text
Pv = runtime
Pp = compile
```

has this OpenStatic behavior:

```text
symbol/path resolves
runtime value is unreadable
compile Pattern/type is readable
derived compile companion may join static overload resolution
original runtime computation remains in RuntimeResidualFlow
```

Seal-only symbols follow the same ordinary-symbol rule. Their paths can be
resolved independently of whether a facet is exposed in the current phase.

## 8. Mechanical compile-flow projection

Before any of the three phases executes, structurally project:

```text
CompleteSymbolFlow
  -> StaticFlow
  -> RuntimeResidualFlow
```

Projection does not execute calls or perform final overload selection.

`StaticFlow` preserves:

```text
Pattern/type flow
meta/compile/seal static call nodes
derived compile companions
symbol relationships required by static bindings
DeferredSealTask nodes
D/Done and control-flow structure
```

`RuntimeResidualFlow` preserves:

```text
runtime value computations and bodies
runtime branch value selection
runtime effects
runtime symbol binding
required D/Done and control-flow structure
```

No phase is inferred ad hoc from the original AST after this projection.

## 9. Namespace visibility and export

### 9.1 Export roots

`export` is allowed only on a direct top-level declaration of one namespace
construction level:

```lang
export let name = expr;
```

It is forbidden in function/meta-function bodies, parameters, return slots,
P2, Pattern interiors, expression policies, ordinary local P1, and any nested
local declaration below that namespace level. A top-level function object may
be an export root; its body declarations may not.

For export root `s`:

```text
ExportClosure(s) = PathAncestors(s) ∪ Subtree(s)
```

All ancestors needed to reach the root and its entire subtree enter the export
graph. A child cannot close export again; an unrelated sibling is unaffected.

### 9.2 Public/private

`public` and `private` are ordinary hierarchical visibility attributes. A
public parent may contain a private child, and a private parent may contain a
public child. External path access checks every segment, so a private parent
blocks external reachability to a public child.

```text
ExternallyVisible(path)
  = Exported(path) && PubliclyReachable(path)
```

Export closure may retain private dependencies without making them
name-addressable externally.

## 10. Wpre and seal world

Immediately before SealStatic, compute the least semantic materialization
closure:

```text
R0 = ExportedSymbols
   ∪ MaterializedResultsOfExportedMetaFunctions
   ∪ ParameterDependenciesOfExportedMetaFunctions

R(n+1) = Rn ∪ SemanticDependencies(Rn)

Wpre = least_fixed_point(R)
```

Materialized results include only results actually generated in this build,
not the infinite set a generic meta function might produce for future inputs.
Wpre can contain non-exported private dependencies solely so the exported
interface remains interpretable.

SealStatic generates `Wseal` and finishes with:

```text
Wfinal = Wpre ∪ Wseal
```

Only a compiler-known privileged seal function may enumerate the symbol world,
and its fixed scan domain is `Wpre`. Adding `Wseal` never expands that domain.
Ordinary seal policy grants no scanning capability.

Explicit lookup is separate:

```text
ResolveExplicitPath != EnumerateSymbolWorld
```

A committed symbol in Wseal can be explicitly resolved by later seal/compile
code under ordinary construction transaction, name-resolution, dependency, and
policy rules. Its absence from the current Wpre scan does not make it
unaddressable.

## 11. Phase execution

### 11.1 OpenStatic

Exposed stages are `meta` and `compile`; seal and runtime value slices are not
exposed. A call may evaluate when its callable exposes a meta/compile view, all
arguments supply the required static views, and the associated `()` candidate
is fully admissible.

Static views include meta values, compile values, compile Pattern/type
projections of runtime symbols, and derived compile companions. Meta and compile
callables may invoke one another in one evaluator, but result ranks remain:

```text
meta    -> SymbolConstructionValue
compile -> PatternValue / ordinary static value
```

No OpenStatic task may read a runtime value or depend on a runtime effect. If a
task is blocked only by a seal-only view, preserve its call node, Pattern
arguments, symbol dependencies, and overload inputs as a `DeferredSealTask`.

When otherwise equal and fully admissible, phase specificity uses the narrower
visible domain:

```text
Vis(meta) ⊂ Vis(compile), therefore meta > compile in OpenStatic
```

This is one dimension of the complete partial order, not an unconditional
global priority.

### 11.2 SealStatic

Exposed stages are `seal` and `compile`; meta and runtime value slices are not.
The same static evaluator and symbol-construction machinery consumes deferred
tasks, explicit seal/compile callables, privileged seal calls, fixed Wpre scan
results, and ordinary explicitly resolved symbols.

When otherwise equal and fully admissible:

```text
Vis(seal) ⊂ Vis(compile), therefore seal > compile in SealStatic
```

SealStatic is terminal for static work. Missing symbols/projections/companions,
runtime value/effect dependencies, or non-unique overload maxima are errors;
there is no later static deferral phase.

### 11.3 Runtime

Runtime consumes `RuntimeResidualFlow`, exposes runtime value slices, completes
runtime symbol binding and overload selection, executes runtime bodies/effects,
and performs runtime branch value selection. A derived compile companion never
replaces the real runtime call.

## 12. Unified binding and overload selection

All ordinary bindings and call targets use one selection trunk:

```text
C0 = EnumerateValueEntries(ResolveSymbol(path))
C1 = ExposePhaseViews(C0, Phase)
C2 = ProjectExpectedPolicy(C1, P1_or_expected_facet)
A  = FullyAdmissible(C2, argument_frame, expected_result)
M  = MaxPolicyAndOverloadOrder(A)
```

Success requires exactly one maximal candidate. Failure can mean no exposed
slice, no fully admissible entry, multiple incomparable maxima, a unique delete
maximum, or an unfinished terminal SealStatic task.

For each const/mut comparison position:

```text
const actual: const > unspecified > mut
mut actual:   mut > unspecified > const
```

Multiple positions form a product partial order: `f` dominates `g` iff `f` is
not worse at every participating position and is strictly better at at least
one. Crossed advantages remain incomparable. There is no score, exact-match
count, parameter weighting, lexicographic order, input-before-output rule, or
separate conversion rank. A result policy participates only when the call
context supplies a target-result constraint.

Delete members enter the same fully admissible set and order. A unique maximal
delete produces a diagnostic naming that member.

## 13. Lifetime boundary

`@` is lifetime syntax, not an ordinary policy operator. This design defines no
lifetime checking algorithm, overload, ordering, ABI class, refinement pass, or
handoff object. Ordinary overload selection must already have one unique
candidate, and future lifetime rules may not change that result.

## 14. Transitional implementation boundary

The typed implementation model contains dedicated policy AST nodes,
`PolicyPair`, typed dimensions, three distinct P1 elaborators, true slice
restriction, three `Phase` values, phase exposure, mechanical flow projection,
Wpre closure, export closure, and phase-aware partial-order selection.

`PolicySet` and `PolicyFlag` remain in older resolver/build paths as a lossy
transport. They cannot represent `||` structure, Pattern association of a
cropped slice, or independent export-root and public/private dimensions. New
semantics must be implemented in the typed model first; compatibility flags may
only receive a projection from it. Full namespace-graph storage and end-to-end
evaluator integration remain implementation work and must not be inferred from
flat flags.

## 15. Deliberately unfrozen

This document does not freeze:

- the final source token for `AbsentValuePattern`;
- full lifetime/Horae semantics;
- future policy stages;
- arbitrary clause-level Boolean policy logic;
- a complete runtime reflection API;
- export reopening syntax;
- cross-file open overload union;
- unrelated `?`, `inject`, or new PatternValue mechanisms.
