# Symbol Policy and Compile-Flow Projection

**Status: Canonical future-design direction. Not current parser, normalizer, or
build-evaluator behavior.**

This note is the normative owner for:

- the layered policy model of a flowing symbol;
- callable policy positions `P1` and `P2`;
- removal of an independent `P3` return-policy position;
- layer-directed result projection;
- mechanical compile-flow projection;
- derived compile-companion `Val2` function objects;
- `must_select_if_qualified` overload consistency;
- policy-directed match staging;
- coarse automatic `require` extraction and grouped synthesis;
- finite local flow with ordinary recursive calls;
- shared evaluation identity between require and body continuation.

It builds on two sibling canonical notes:

- `symbol-first-meta-construction-and-pattern-injection.md` owns symbol-first
  resolution, facets, `PatternValue`, `compile` / `meta` result ranks, pattern
  scopes, and graph-installation boundaries;
- `symbol-construction-units-and-namespace-origin.md` owns namespace origin,
  construction-unit ownership, physical contribution authority, and cross-unit
  closure.

Pattern residuals and `Done` are not redefined here. Their algebra is owned by
`../patterns-overload/static-pattern-spaces-and-extraction-chains.md`. Overload
specificity and the ordinary linear filters are owned by
`../patterns-overload/overload-resolution-design.md`. This note defines the
policy/companion obligations those documents must preserve.

## 1. Complete Symbol Flow

Language computation is one flow of symbols. It is not split into a traditional
“type world” and a separate “value world.” For policy and flow projection, use
the abstract view:

```text
Symbol = Val₁ × Pattern × Val₂

ASCII notation in implementation-oriented sketches:
  Val1 x Pattern x Val2
```

where:

```text
Val1:
  value leaves inside the pattern tree

Pattern:
  the pattern structure itself

Val2:
  objects located at a pattern layer;
  commonly callable objects
```

This is a semantic flow view, not a replacement Rust layout for `SymbolCell`.
The symbol-first storage model may still expose namespace, type, and
heterogeneous value facets. A source name first resolves a symbol; semantic
context then projects the relevant facet and places its material into the
`Val1 x Pattern x Val2` flow view.

The traditional categories are degeneracies of this one model:

```text
traditional pure type:
  Val1 is empty

traditional ordinary value:
  Val2 is empty
```

Neither case creates a second identity universe. `SymbolId`, `PlaceId`, and
`PatternValue` identity remain distinct as specified by the symbol-first note,
but type-shaped and value-shaped computations still travel through the same
symbol flow.

## 2. Layered Policy

Each layer has its own policy:

```text
policy(Val1 leaf)
policy(Pattern)
policy(Val2 object)
```

The current design has not introduced `seal` into pattern construction.
Therefore:

```text
policy(Pattern) = compile
```

Value leaves may currently be:

```text
policy(Val1 leaf) in { compile, runtime }
```

For a collection of leaves, `policy(Val1)` is their least upper stage bound.
The current two-stage order is:

```text
compile < runtime
```

The total policy of the symbol's pattern-bearing value is:

```text
total_policy(Symbol)
  = policy(Val₁) ⊔ policy(Pattern)
```

Here `⊔` is the least upper stage bound (`join`).

The empty-leaf case contributes no later stage. Consequently:

```text
Val1 is empty
  => total_policy(Symbol) = compile

any runtime leaf in Val1
  => total_policy(Symbol) = runtime
```

This explains why a pure type is naturally compile-policy without introducing a
special type-only rule.

`Val2` is deliberately excluded from `total_policy(Symbol)`. It contains objects
available at that pattern layer for later symbol lookup and invocation. Each
`Val2` object has its own lookup policy (`P1` below) and, when callable, its own
entry execution policy (`P2`). Including those objects in the enclosing
pattern-value policy would make the mere presence of a runtime callable turn a
compile pattern into a runtime pattern, which is incorrect.

Future `seal` work may extend `policy(Pattern)`. Until an explicit seal design is
adopted, documents and implementations must not infer a seal-pattern policy or
change the equations above.

## 3. Callable Policy Positions

The future semantic callable form is:

```text
P1 let F = (...): P2 -> let r => { ... }
```

This is a semantic notation over the parser-preserved binding and arrow shape.
It does not request a parser change in the current PR.

### 3.1 `P1`: callable-object lookup policy

`P1` is the external lookup policy of one callable object or other `Val2`
object. It does not control whether a source path resolves to its containing
`Symbol`. The required order is:

```text
path
  -> resolve Symbol
  -> project heterogeneous value facet
  -> enumerate Val2 objects
  -> filter each object by P1 for the current lookup stage
```

Export, access, or namespace visibility may independently restrict path
resolution, but `P1` is not one of those base symbol-resolution gates. A single
symbol may carry heterogeneous objects with different `P1` sets; there is no
requirement that the whole symbol have one uniform callable policy.

The current permitted forms are:

```text
P1 ::= compile
     | (compile | runtime)
```

The canonical spelling of the two-stage set is `compile | runtime`. A callable
`Val2` function object is always compile-visible because its Pattern, parameter
and result patterns, constraints, associated `()` identity, and compile
projection must participate in compile symbol flow. Pure runtime visibility is
therefore not a valid callable-object `P1`.

`P1` is not an argument policy and does not grant permission to execute the
callable body. Runtime execution belongs to `P2 = runtime`; the containing
function object then has `P1 = compile | runtime` by declaration
well-formedness.

After the symbol and its value facet have been obtained, a particular object is
externally usable only when:

```text
current_lookup_stage in P1
```

### 3.2 `P2`: callable-entry execution policy

`P2` is the execution capability of the callable entry:

```text
P2 ::= compile
     | meta
     | runtime
```

The current design does not permit:

```text
P2 = runtime | compile
```

That composite entry capability is reserved for a future two-stage
symbol-binding/evaluation model. It must not be inferred from a `P1` union.

External flow maps execution capabilities to lookup stages:

```text
external(compile) = compile
external(meta)    = compile
external(runtime) = runtime
```

The callable declaration is well formed only when:

```text
external(P2) ⊆ P1
```

The notation treats `P1` as a set of external lookup stages. The rule means that
the stage needed to execute the entry must be a stage in which the callable
object itself can be found.

After lookup, an entry is execution-policy-qualified only when every slot in
the complete invocation frame has exactly the external policy
required by `P2`:

```text
InvocationFrame:
  slot 0:    implicit self
  slot 1..n: explicit source arguments

for every frame slot a:
  total_policy(a) = external(P2)
```

This is equality, not “no later than” and not a priority hint. `P1` does not
describe the parameters; `P2` does not describe the callable object's lookup
visibility. No independent self-policy mechanism is needed. `self` is the
selected `Val2` function object viewed at the call stage. Because
`external(P2) ⊆ P1`, declaration well-formedness guarantees that object is
visible at the required stage; invocation records the slot-0 view under the
same `P2` requirement used for slots 1..n.

### 3.3 `compile` and `meta` remain internally distinct

Within a callable body:

```text
compile:
  compute ordinary static values and PatternValue

meta:
  construct SymbolConstructionValue inside one MetaConstructionUnit
```

In external compile-flow projection, both map to the early `compile` flow through
`external(P2)`. This unification does not grant `compile` the symbol-construction
capability of `meta`.

Evaluation demand remains orthogonal:

```text
execution capability: compile | meta | runtime
evaluation demand:     partial | strict
result rank:           PatternValue | SymbolConstructionValue | runtime value
```

### 3.4 There is no independent `P3`

An independent return-policy position is removed from the final model. The
returned symbol is still layered:

```text
Val1 x Pattern x Val2
```

and each layer retains its own policy. A single `P3` would flatten those layers
back into one result-policy label and contradict the symbol model.

Removing `P3` also means that the result symbol does not receive one replacement
scalar policy. Instead, apply the selected callable object's `P1` as a
layer-directed result projection:

```text
Result = Result.Val1 x Result.Pattern x Result.Val2

result_projection_by_P1(Result, P1)
```

The projection is:

```text
Result.Pattern:
  retain stages admitted by P1 and supported by Pattern

current rule:
  policy(Pattern) = compile
  therefore Result.Pattern retains only its compile part

Result.Val1 when P1 = compile:
  retain compile-policy leaves only

Result.Val1 when P1 = compile | runtime:
  retain both compile-policy and runtime-policy leaves

Result.Val2:
  retain each returned object with that object's own P1
  do not copy the caller function object's P1 onto returned objects
```

Thus `P1` determines which result layers can flow outward at admitted stages,
but it does not become a policy field on the result symbol. A heterogeneous
result may expose several `Val2` objects with different `P1` sets.

Policy changes must not be hidden at the return position. A future operation
that closes, projects, or changes policy requires an explicit mechanism. It must
not reintroduce a generic `P3` slot.

The current Rust fields named `return_object_policy`, and parser-preserved
policy expressions on return binding slots, are transitional transport
metadata. They are neither a final `P3` nor a whole-result-symbol policy. No
Rust or parser migration is implemented by this note.

## 4. Compile Flow Is a Projection

The complete program first forms one symbol-flow graph:

```text
CompleteSymbolFlow
```

`compile flow`, `meta flow`, and `runtime flow` do not name three independent
semantic graphs. In this document:

```text
compile flow:
  the compile projection of CompleteSymbolFlow

meta flow:
  meta-capability construction edges retained inside that compile projection

runtime flow:
  runtime leaf computation and branch continuation still represented by
  CompleteSymbolFlow after compile projection has extracted its static view
```

The terms classify edges or views of one symbol flow; they do not revive
separate type/value/stage languages.

Compile-stage analysis is a mechanical projection of that complete graph:

```text
CompileFlow = compile_projection(CompleteSymbolFlow)
```

The projection preserves:

```text
Pattern flow
compile-policy Val1 leaf flow
compile/meta calls and their static dataflow
calls that can select a derived compile-companion object
```

It removes or defers:

```text
runtime-policy Val1 leaf flow
value computation executable only by a runtime entry
runtime branch selection
```

A runtime-valued symbol therefore does not vanish. Given:

```text
S_runtime = Val₁_runtime × Pattern_compile × Val₂
```

the projection retains at least:

```text
compile_projection(S_runtime) = Pattern_compile
```

The projection pass extracts and rewrites flow structure. It does not:

```text
execute a call
select a final overload
compute a predicate
compute PatternValue
decide whether an assertion is true
```

In particular, projection does not need a selected overload or a returned value
to discover a removed `P3`: there is no `P3` to discover. Projected calls remain
ordinary unresolved calls. Ordinary compile evaluation later performs symbol
lookup, candidate admissibility/preference filtering, and value computation.

Projection is not approximate evaluation and not global stage-constraint
solving. It is a structural graph operation over already formed symbol flow.

### 4.1 Calls project homomorphically

An ordinary call is already unresolved before normal overload evaluation.
Compile projection therefore preserves ordinary call syntax/flow rather than
requiring a new public call-family semantic object:

```text
C[(args...) f]
  = (C[args]...) f
```

For a runtime-valued argument, its compile projection is its Pattern view:

```text
C[arg_runtime]
  = arg_runtime |> type
```

Therefore the compile projection of a runtime call is represented as the
ordinary call:

```text
(args...) f
  -> (args... |> type) f
```

Here `arg |> type` is symbol facet/Pattern projection, not access to a separate
traditional type system. Normal compile lookup and overload resolution later
decide whether a direct compile/meta object or a derived compile companion is
the admissible callee.

An implementation may use an internal unresolved-call IR node, candidate-family
key, or projection provenance to preserve bookkeeping. Those are implementation
choices, not required public language identities.

### 4.2 Finite local flow and ordinary recursion

The language has no source loop construct and no inline-for node. Repetition is
expressed through callable recursion. The mechanical-lowering `loop` call mode
is a lowering mode for recursive calls, not a source control-flow node.

For one callable body, treat every call as one opaque finite node:

```text
CallNode {
    callee expression or resolved symbol,
    argument symbol flows,
    result flow,
    projection provenance,
}

LocalSymbolFlow(callable_body)
  is finite, bounded, and has no source-loop control node
```

`compile_projection` projects the call node and does not unfold the callee
body. Direct or mutual recursion remains ordinary compile evaluation:

```text
f -> f
f -> g -> f
```

Whether such evaluation terminates is an ordinary compile-program semantic
question, not an additional compile-flow-extraction problem. Projection and
require slicing neither summarize nor unfold recursive callees, and they do not
assume a finite dynamic call graph.

## 5. Runtime Function Objects Have Derived Compile Companions

For a runtime execution entry:

```lang
(compile | runtime) let f =
    (<T>a: T, <U>b: U): runtime -> _ => {
        ...
    };
```

the semantic model derives another complete `Val2` function object. It is not
an identity-less extra `()` entry under the origin object's type:

```text
origin runtime Val2 function object
  |- runtime function-object type
  `- associated runtime ()

derived compile companion Val2 function object
  |- derived function-object type
  `- associated compile ()
```

A conceptual semantic record is:

```text
DerivedCompileCompanionObject {
    object_id,
    origin_runtime_object_id,
    function_object_type,
    associated_namespace,
    associated_call_entry,
    overload_strategy,
    provenance,
}
```

Typical policy is:

```text
origin runtime object:
  P1 = compile | runtime
  P2(origin associated ()) = runtime

derived companion object:
  P1 = compile
  P2(companion associated ()) = compile
  overload_strategy = must_select_if_qualified
```

The derived object enters the named callable symbol's value facet and follows
the ordinary symbol-first pipeline:

```text
resolve Symbol
  -> enumerate Val2 objects
  -> filter each object by P1
  -> obtain each object's type
  -> resolve its type-associated ()
  -> form fully admissible set A after every hard check
  -> run fixed-order overload preference filters
  -> enforce must-select consistency
```

It has stable object identity, origin-runtime-object identity, its own type, and
its own associated `()`. Candidate diagnostics, reflection, and documentation
tools may display it. Exact user selector syntax is open, but the object must
not be implemented as a completely unobservable fallback.

Its body/behavior is the mechanical compile projection of the origin runtime
object's complete symbol flow, not a traditional signature assertion rewrite.
At a projected call site:

```text
(args...) f
  -> (args... |> type) f
```

normal compile lookup later sees the companion object in the value facet and
performs ordinary overload resolution. The companion is not synthesized only
after overload failure, and ordinary priority cannot silently replace it.

## 6. Must-Select Is an Overload Strategy

The companion object's semantic metadata includes:

```text
overload_strategy = must_select_if_qualified
```

When its associated `()` is prepared, that strategy is copied to the prepared
candidate:

```text
PreparedCandidate {
    callable_object_id,
    call_entry_id,
    overload_strategy,
    ...
}
```

Let `A` be the fully admissible candidate set after every hard precondition has
passed, including visibility, `P1`, associated `()`, structure, `P2`, complete
invocation-frame policy (including self), expected result rank/facet, concepts,
ordinary require, and other compile/type legality. Let fixed-order preference
filters produce `Bn` from `A`.

Define:

```text
M = {
    c in A
    |
    overload_strategy(c) = must_select_if_qualified
}
```

The final consistency rule is:

```text
M is empty:
  use the ordinary unique-survivor rule

M = {m}:
  succeed only when Bn = {m}
  otherwise report overload-set inconsistency

|M| > 1:
  report conflicting admissible must-select objects
```

This strategy is not infinite priority. It does not automatically defeat other
candidates; it requires the fully legal overload set to remain consistent with
the must-select object.

The semantic attribute may later be user-accessible. Possible surface sketches
include:

```lang
[[must_select_if_qualified]]
let f = ...;

#must_select_if_qualified
let f = ...;
```

Both `[[...]]` and `#...` are candidate spellings only. This specification
freezes the semantic strategy, not lexer, parser, Raw AST, or Normalized AST
syntax.

### 6.1 Replacement direction and open suppression question

Ordinary overload priority cannot replace a default companion. A future
explicit replacement may associate another complete compile function object by
semantic `companion_of` metadata. Candidate spellings include:

```lang
[[companion_of(runtime_f)]]
let f = ...;

#companion_of(runtime_f)
let f = ...;
```

Again, neither spelling is frozen. The final association mechanism and the way
users name a derived companion object remain open.

This note does not currently grant a general ability to suppress the default
companion. Open questions are:

```text
whether suppression is allowed at all;
whether an equivalent compile Pattern/contract interface is mandatory;
whether suppression requires an explicit replacement companion.
```

## 7. Match, `if`, and Stage Selection

The language has one pattern-matching mechanism. It does not have distinct
semantic constructs for:

```text
match
constexpr match
if
if constexpr
```

`if` / `else` is the two-pattern case:

```text
match cond {
    true  => ...
    false => ...
}
```

A library extraction view may expose conventional arm labels such as `if` and
`else`. Those labels select the same two pattern alternatives; they do not
create a second conditional or constexpr mechanism.

The scrutinee symbol's total policy determines when branch selection occurs:

```text
total_policy(scrutinee) = compile:
  compile match
  normal compile evaluation selects the branch

total_policy(scrutinee) = runtime:
  runtime match
  Pattern remains in CompileFlow
  runtime value-leaf branch selection remains deferred to runtime
```

Compile-flow projection itself does not evaluate the scrutinee. It preserves the
guarded match structure so later compile evaluation can select a compile match
or validate every reachable runtime branch contract.

No separate `if constexpr` or `constexpr match` syntax/semantics is needed.

## 8. D/Done Is Intrinsic to Complete Match Flow

Match is already represented in `CompleteSymbolFlow` by the pattern system's
residual and completion normal form:

```text
A |> S { body }
  -> D(A, S) + Done(body(S))
```

The complete flow already contains D residual domains, Done isolation, and
guarded branch structure; there is no competing pass-order choice. Compile
projection acts homomorphically:

```text
C[D(A, S)]
  = D(C[A], C[S])

C[Done(B)]
  = Done(C[B])

C[X + Y]
  = C[X] + C[Y]

C[guarded flow]
  = guarded C[flow]
```

For a runtime scrutinee, runtime `Val1` branch choice is deferred, while the
Pattern, D residual domain, Done completion structure, and guards remain in the
compile projection. Automatic require consumes that projected structure and
does not invent a parallel traditional CFG branch algebra.

## 9. Automatic Require Extraction

Automatic require initially uses a coarse structural rule:

> From compile-projected symbol flow, retain every complete computation flow
> block that is data-dependent on, guarded by, or control-dominated by one or
> more formal inputs and that ultimately terminates in an explicit
> assertion/verification structure.

Assertion endpoints include:

```text
assert
require
delete/reject branch
another explicitly specified verification endpoint
```

The retained unit is a complete flow block, not a requirement to canonicalize
every internal node into a separate contract atom. Implementations may track
data dependency, guarded domains, and control dominance to find those blocks,
but this note does not freeze one least-backward-cone algorithm or per-node
contract identity.

Closed compile constants, global pure symbols, and helper calls may remain
inside a retained block as necessary side inputs. Unrelated global assertions
do not become a callable's inferred contract merely because they share a
compilation unit.

If a retained block contains a call, retain that ordinary `CallNode` without
unfolding the callee body during projection or slicing. Normal compile
evaluation executes the call and may recurse in the ordinary way.

Automatic require is not a cheap symbol-existence test and is not an attempt to
avoid real compile computation. It selects the compile flow that expresses the
callable's actual static preconditions.

## 10. Coarse Require Synthesis

### 10.1 Serial complete blocks

For retained complete blocks:

```text
Require(BlockA ; BlockB)
  = Require(BlockA) && Require(BlockB)
```

The conjunction preserves source/semantic computation order. It does not imply
that every statement or internal node is independently atomized.

### 10.2 Compile match

A compile-policy match remains one grouped branch structure:

```text
Require(
  compile match {
    S1 => B1
    S2 => B2
  }
)
=
(
  Guard(S1) && Require(B1)
)
||
(
  Guard(S2) && Require(B2)
)
```

Each `Guard(Si)` is the branch's normalized Pattern domain, including the D
residual inherited from earlier alternatives. The grouped OR therefore keeps
the original match structure rather than disjoining bare assertions.

With surrounding blocks:

```text
Require(Before)
&&
(
  branch1 || branch2
)
&&
Require(After)
```

The external contract may wrap that grouped OR in one structured atom so the
top level remains conjunctive. This PR does not freeze that atom's exact fields,
cache identity, comparison rules, or spelling.

### 10.3 Runtime match

Every runtime-reachable branch must retain its Pattern guard:

```text
GuardedRequire(S1, B1)
&&
GuardedRequire(S2, B2)
&& ...
```

This means:

```text
(S1 => Require(B1))
&&
(S2 => Require(B2))
```

It must not be flattened to unguarded `Require(B1) && Require(B2)`.

### 10.4 Inferred and manual require

The total contract remains:

```text
Require_total
  = Require_inferred && Require_manual
```

Manual require is neither an override nor an alternative. It may state stronger
interface promises or design constraints, but it cannot delete, mask, or bypass
the computations and assertions mechanically required by the body.

## 11. One Evaluation Graph, Not Two Executions

Automatic require is close to evaluating the relevant compile flow early. The
design must state this honestly; it must not reuse the traditional rationale
that require is only a cheap `type -> bool` existence check.

Require and body continuation are two views of one graph:

```text
CompleteCompileFlow
  |- RequireView
  `- BodyContinuationView
```

A static node has one evaluation meaning in one canonical instantiation
environment:

```text
one static computation node
  + one canonical instantiation environment
  -> one evaluation identity/result
```

If the require view first demands a node, body continuation reuses the same
semantic result. Shared results include:

```text
overload selection
PatternValue
intermediate type/pattern computation
predicate result
pattern normalization result
reusable static diagnostic material
```

This is not “run require, then hope an incidental cache hits during the body.”
The views share node identity and a consistency cache by definition.
This shared evaluation identity does not require the contract layer to freeze a
canonical identity for every grouped require block or guarded branch atom.

Compile-flow projection still does not perform this evaluation. Projection forms
the graph and views; normal compile evaluation realizes demanded nodes.

## 12. Simple and General Generic Constraints

For common generic functions, inferred require naturally resembles familiar
constraint vocabulary:

```text
comparable(T)
constructible(T)
has_operation(T)
```

This is a simple instance of the general rule, not a separate fast path:

```text
simple flow:
  inferred require resembles a traditional concept/require

complex flow:
  inferred require retains general compile symbol-flow computation
```

Complex require is not a pessimistic fallback. Both cases are projections and
views of the same compile graph.

## 13. Current Implementation Substrate

The current repository implements only narrower prerequisites:

- `PolicyFlag`, `PolicySet`, and limited `PolicyEnv::Meta` /
  `PolicyEnv::Runtime` lookup filtering;
- separate current metadata slots for symbol visibility, body entry, and a
  transitional return-object policy;
- restricted meta candidate preparation and partial/strict failure routing;
- a linear overload-filtering prototype;
- parser/normalizer preservation of `require` head clauses without contract
  evaluation;
- pattern-space and `Done` design notes, with only limited build-evaluator
  substrates;
- no complete symbol-flow semantic IR.

Not implemented:

```text
Val1 x Pattern x Val2 flow representation
total_policy computation
P1/P2 declaration and call well-formedness checking
removal/migration of transitional return-object-policy storage
layer-directed result_projection_by_P1
CompleteSymbolFlow or compile_projection
derived CompileCompanion Val2 function objects
derived function-object types and associated compile () entries
must_select_if_qualified
semantic companion replacement metadata or public selector syntax
sealed_overload_name / closed_overload_set
policy-directed compile/runtime match elaboration
coarse formal-dependent/guarded require slicing and grouped synthesis
automatic inferred require
shared RequireView / BodyContinuationView node identity
future P2 = runtime | compile
seal-aware Pattern policy
```

The current restricted evaluator must not be presented as implementing these
objects. This note establishes a target boundary for later work; it does not
authorize a broad multi-stage evaluator implementation in PR #94.

## 14. Required Invariants

Future implementation must preserve:

```text
1. Programs form CompleteSymbolFlow, not separate type/value flows.
2. total_policy uses Val1 and Pattern, never Val2.
3. Pattern policy is compile until an explicit seal design changes it.
4. Path resolution reaches Symbol before per-Val2-object P1 filtering.
5. Callable-object P1 is compile or compile | runtime, never runtime alone.
6. P2 controls entry execution and exact total policy for the full invocation
   frame, including implicit self.
7. external(P2) must be included in the selected object's P1.
8. There is no independent P3 and no scalar result-symbol policy; result
   material is projected layer by layer.
9. compile_projection is structural and preserves ordinary unresolved calls; it
   does not evaluate or select overloads.
10. A local callable flow is finite when ordinary CallNodes are opaque;
    recursion remains ordinary evaluation and requires no summary semantics.
11. Runtime symbols retain their compile Pattern projection.
12. A compile companion is a complete derived Val2 function object with its own
    type and associated compile ().
13. Must-select activates from the fully admissible candidate set and requires
    the strategy-bearing object to be the final unique survivor.
14. Linear filters run in fixed normative order and are only internally
    independent of candidate enumeration order.
15. Ordinary priority never silently replaces a companion; replacement requires
    an explicit future association mechanism, while suppression remains open.
16. Match staging follows scrutinee total_policy; no constexpr match family exists.
17. D/Done structure is intrinsic to CompleteSymbolFlow and projects
    homomorphically.
18. Initial inferred require preserves complete blocks and grouped branches; it
    does not freeze per-node or guarded-atom identity.
19. Runtime branch requires are a conjunction of pattern-guarded contracts.
20. Require_total is inferred && manual.
21. Require and body continuation share one static evaluation graph.
```
