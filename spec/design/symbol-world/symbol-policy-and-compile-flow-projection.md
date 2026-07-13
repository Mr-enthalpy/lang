# Symbol Policy and Compile-Flow Projection

**Status: Canonical future-design direction. Not current parser, normalizer, or
build-evaluator behavior.**

This note is the normative owner for:

- the layered policy model of a flowing symbol;
- callable policy positions `P1` and `P2`;
- removal of an independent `P3` return-policy position;
- mechanical compile-flow projection;
- unresolved call families and derived companion families;
- runtime callable compile companions;
- `must_select_if_qualified` overload consistency;
- policy-directed match staging;
- automatic `require` extraction and synthesis;
- finite local flow, recursive call-summary boundaries, and guarded require atoms;
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
     | runtime
     | (compile | runtime)
```

The equivalent source ordering `runtime | compile` is permitted only in `P1`.
`P1` is not an argument policy and does not grant permission to execute the
callable body.

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

After lookup, an entry is execution-policy-qualified only when every argument
symbol has exactly the external policy required by `P2`:

```text
for every argument a:
  total_policy(a) = external(P2)
```

This is equality, not “no later than” and not a priority hint. `P1` does not
describe the parameters; `P2` does not describe the callable object's lookup
visibility.

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

The result symbol's external lookup policy inherits the policy of the selected
callable object:

```text
lookup_policy(result_symbol_of(call))
  = P1(selected_callable_object)
```

It does not inherit a hypothetical policy of the callee symbol as a whole; a
heterogeneous callee symbol may have no such single policy. Nor does this assign
the selected object's `P1` to every internal leaf, pattern, or later `Val2`
object in the result. Their policies remain independently represented and
contribute according to the rules in §2.

Policy changes must not be hidden at the return position. A future operation
that closes, projects, or changes policy requires an explicit mechanism. It must
not reintroduce a generic `P3` slot.

The current Rust fields named `return_object_policy`, and parser-preserved policy
expressions on return binding slots, are transitional substrate. They are not
evidence that final semantics has a third callable policy plane. No Rust or
parser migration is implemented by this note.

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
compile/meta call-family nodes and their static dataflow
derived compile-companion call nodes
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
to discover a removed `P3`: there is no `P3` to discover. A projected graph may
contain an unresolved call family, a derived-companion call family, pattern
guards, and deferred evaluation nodes. Ordinary compile evaluation later
performs symbol lookup, candidate qualification/filtering, and value
computation.

Projection is not approximate evaluation and not global stage-constraint
solving. It is a structural graph operation over already formed symbol flow.

### 4.1 Unresolved call families

Because projection does not select an overload, every projected call retains a
family object rather than a selected entry:

```text
UnresolvedCallFamily {
    callee_symbol,
    argument_symbol_flows,
    lookup_stage,
    expected_rank_or_facet,
    candidate_family_identity,
}
```

`callee_symbol` records completed symbol resolution. Candidate preparation may
still need to project its heterogeneous value facet, obtain each value's type,
resolve type-associated `()` entries, and perform qualification. The stable
`candidate_family_identity` identifies that unresolved preparation family; it
does not assert that any member is applicable or selected.

For an unresolved origin candidate family `C`, the mechanical runtime-entry
projection is:

```text
CompanionFamily(C)
  = {
      companion(e)
      |
      e in C
      and P2(e) = runtime
      and compile in P1(e)
    }
```

Each `companion(e)` has the stable origin-entry identity defined in §5, but the
family as a whole remains unresolved:

```text
DerivedCompanionCallFamily {
    origin_call_family,
    projected_argument_symbol_flows,
    companion_candidate_family = CompanionFamily(C),
    expected_rank_or_facet,
}
```

Projection may retain direct compile/meta candidates and this derived companion
family under the same projected call site. Normal compile evaluation then runs
candidate preparation, per-object `P1` filtering, `P2` qualification, the fixed
linear-filter pipeline, must-select consistency, and final selection. Thus
projection does not choose a runtime entry, while every derived companion still
has a precise origin when candidates are later enumerated.

### 4.2 Finite local flow and recursive call boundaries

The language has no source loop construct and no inline-for node. Repetition is
expressed through callable recursion. The mechanical-lowering `loop` call mode
is a lowering mode for recursive calls, not a source `LocalSymbolFlow` loop
node.

For one callable body, treat every call as an opaque `UnresolvedCallFamily`
node. Under that treatment:

```text
LocalSymbolFlow(callable_body)
  is finite, bounded, and control-flow-acyclic
```

The cross-call graph may contain cycles because callables may recurse directly
or mutually. `compile_projection` does not unfold those calls. It projects the
call node, argument symbol flows, result flow, and a reference to the callee's
contract/require summary. Consequently there is no projection-time recursion
expansion, source-loop fixed point, inline-for elaboration, or loop-invariant
obligation. Local require slicing operates on a finite graph; recursion remains
represented by call and summary-reference edges.

## 5. Runtime Entries Have Derived Compile Companions

For a runtime entry that remains externally visible to compile lookup:

```lang
(runtime | compile) let f =
    (<T>a: T, <U>b: U): runtime -> _ => {
        ...
    };
```

the semantic model derives a compile companion approximately shaped as:

```lang
compile let f = (T, U): compile -> _ => {
    compile_projection(original_body)
};
```

This sketch does not assert a traditional signature rewrite such as
`f : (T, U) -> T`. The companion body is the mechanical projection of the
original complete symbol flow.

Automatic derivation requires:

```text
P2(origin) = runtime
compile in P1(origin)
```

A pure runtime-only callable:

```text
runtime let f = (...): runtime -> ...
```

is not found by compile lookup and has no automatically discoverable compile
companion.

The derived entry has stable identity and provenance:

```text
DerivedCallableEntryId {
    origin_runtime_entry,
    derivation_kind = CompileCompanion,
}
```

Its effective policies are:

```text
P1(companion) = compile
P2(companion) = compile
origin(companion) = origin_runtime_entry
```

The companion is a first-class derived callable entry. It can appear in
candidate enumeration, diagnostics, reflection, and documentation tooling. It
is not synthesized only after ordinary overload resolution fails.

“Implicit companion” therefore means automatically derived from an eligible
runtime entry. It does not mean hidden, identity-less, or fallback-only.

When no explicitly associated compile/meta companion replaces the default
derived path, that path uses the conceptual argument rewrite:

```text
(args...) f
  -> (args... |> type) f
```

where:

```text
arg |> type
```

means “project the argument symbol's compile pattern view.” It is symbol facet /
pattern projection, not a second traditional type-extraction universe.

The projected argument is compile-policy pattern material. It does not mutate
the original runtime symbol's total policy and does not make that original
symbol directly qualify for a compile entry.

The rewrite forms the `DerivedCompanionCallFamily` of §4.1 and remains unresolved
until normal compile evaluation. The existence of a direct compile/meta entry
can affect ordinary candidate qualification, but default companion generation
is not a post-failure fallback and cannot be silently disabled by overload
priority.

## 6. Companion Overload Consistency

Compile companions participate in the ordinary overload candidate pipeline.
They also carry:

```text
must_select_if_qualified
```

Let the initial candidate set be:

```text
C0
```

After lookup, structural, policy, and argument qualification, let:

```text
Q = Qualified(C0, current_lookup_stage, arguments)
```

Membership in `Q`, not mere same-name presence in `C0`, is the boundary for
“participates in this match.” In particular, qualification includes:

```text
current_lookup_stage in P1(candidate)
external(P2(candidate)) admitted by the lookup/evaluation flow
for every argument a:
  total_policy(a) = external(P2(candidate))
structural pattern/arity applicability
```

The ordinary linear filters then operate in one fixed normative order:

```text
C1 = F1(Q)
C2 = F2(C1)
...
Cn = Fn(Cn-1)
```

Each filter is side-effect-free, and each filter's result is independent of
candidate enumeration and source declaration order. The filters themselves are
not assumed to commute: exchanging `Fi` and `Fj` may change the result and is a
specification error unless a separate proof establishes equivalence.

Ordinary success still requires:

```text
|Cn| = 1
```

Define the qualified must-select set:

```text
E = {
    e in Q
    |
    e has must_select_if_qualified
}
```

The final consistency rule is:

```text
E is empty:
  use the ordinary overload result

E = {e}:
  succeed only when Cn = {e};
  otherwise report overload-set inconsistency

|E| > 1:
  report overload-set inconsistency
```

Therefore a more specific ordinary overload cannot silently displace a
qualified companion. If two runtime overloads erase their runtime leaves into
two simultaneously qualified compile companions, the call is rejected: the
runtime overload family has no unique compile projection.

### 6.1 User-accessible must-select capability

The mechanism is not compiler-private. A future user-facing annotation or trait
may be spelled conceptually as:

```text
@must_select_if_qualified
```

Its meaning is:

> Once this callable entry enters the qualified candidate set for a call, it
> must be the final unique candidate; otherwise overload resolution fails.

This allows user-authored constraint callables to request the same overlap
consistency as a derived companion.

All `@...` spellings in this section are conceptual notation only. They do not
request a lexer, parser, Raw AST, or Normalized AST change. An initial
implementation may use compiler-known metadata, a derived-entry origin tag, a
core symbol identity, an internal semantic attribute, build metadata, or
another built-in marker. Public source syntax, if later justified, need not use
these spellings. The implementation order is semantic object first and optional
surface syntax only after usage experience.

It is not equivalent to closing an overload name:

```text
must_select_if_qualified:
  permits non-overlapping same-name entries;
  rejects applicable overlap that selects another entry

sealed_overload_name / closed_overload_set:
  forbids adding any other same-name entry in the specified overload domain
```

The latter is a separate future mechanism.

### 6.2 Explicit replacement and suppression

Ordinary overload priority cannot replace or disable a default companion. A
future explicit association may provide a user-defined replacement:

```text
@companion_of(runtime_entry)
let f = (...): compile -> ...;
```

That association means:

```text
do not generate the default companion for runtime_entry
the declared entry owns the CompileCompanion relation
the declared entry inherits must_select_if_qualified
```

A runtime entry may explicitly opt out of the exported derived interface:

```text
@no_compile_companion
```

This suppresses discovery of an automatic companion at call sites, the
caller-visible inferred-require interface derived through that companion, and
the default runtime-call compile-projection interface exported by that entry.
It does not suppress compile-flow analysis of the callable definition, skip
definition-time static assertions, or disable ordinary compile checking inside
the body. The body remains valid only if its own static computation and
verification succeed. Suppression is an explicit interface decision, not an
accidental result of another overload winning.

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

## 8. D/Done-Normalized Match Flow

Automatic require extraction consumes the existing pattern-normalized control
flow. It does not inspect an unrelated traditional CFG and does not invent a
second branch algebra.

For current pattern space `A` and extracted subpattern `S1`:

```text
hit(S1):
  branch1 receives S1

miss(S1):
  D(A, S1) continues to the next branch

completed branch result B1:
  (B1)Done is isolated from later same-level extraction
```

The exact residual and completion rules remain those of the pattern-space note:

```text
A |> S { body }
  -> D(A, S) + (body(S))Done
```

where the current concrete residual notation is commonly `A - S`. Branch chains
must undergo this D/Done normalization before require synthesis. A later require
pass consumes guarded residual domains and completed assertion endpoints; it
must not reinterpret arbitrary CFG edges as pattern alternatives.

## 9. Automatic Require Extraction

Automatic require is defined over a callable's finite `LocalSymbolFlow` after
compile projection and D/Done normalization. The old shorthand
“parameter-dominated assertion slice” is insufficient because data dependency,
control dominance, and backward slicing are different relations.

Use the following conceptual graph:

```text
LocalCompileFlow = (
    nodes,
    data_dependency_edges,
    control_edges,
    pattern_guard_edges,
    call_summary_edges,
)
```

For a node `n` and formal projection `p`:

```text
parameter-dependent(n, p):
  a data-dependency path exists from p to n

parameter-guarded(n, p):
  n lies in a D/Done-normalized pattern domain whose guard or scrutinee
  is parameter-dependent on p

parameter-dominated(n, p):
  every local control path from body entry to n passes through a
  parameter-derived control/guard gate associated with p
```

`parameter-dominated` is the ordinary graph-theoretic control property. It may
support diagnostics or optimizations, but it is not a synonym for data
dependency and is not by itself the require-inclusion criterion.

Let `A` be an explicit assertion sink. Define:

```text
FormalDeps(A)
  = { p | parameter-dependent(A, p) or parameter-guarded(A, p) }

EligibleRequireSink(A)
  iff A is an assertion/verification endpoint
      and FormalDeps(A) is non-empty
```

Multiple parameters participate by appearing together in `FormalDeps(A)`; the
slice retains every contributing projection rather than choosing one dominant
parameter. The inferred slice is:

```text
InferredRequireSlice(A)
  = the least backward dependency cone from A that retains:
      all data dependencies back to FormalDeps(A),
      parameter-controlled pattern guards and D/Done residual domains,
      required call-summary references,
      closed compile constants and global pure symbols used as side inputs,
      provenance and semantic ordering edges
```

The full pipeline is therefore:

```text
CompleteSymbolFlow
  -> compile_projection
LocalCompileFlow
  -> D/Done normalization
  -> union of InferredRequireSlice(A) for every EligibleRequireSink(A)
InferredRequireFlow
```

Pattern guards are explicit control-dependency edges whose scrutinees may carry
data dependency on formal projections. They are not erased into ordinary data
edges. A compile helper call contributes an instantiated contract-summary
reference at its call node; helper assertions propagate through that summary
rather than by recursively inlining the helper body. Recursive and mutually
recursive calls likewise retain canonical summary references and are never
expanded by slicing.

A closed compile constant, global pure symbol, or helper result may be a side
input to an eligible cone without becoming a formal dependency itself. An
unrelated global assertion does not enter the function contract merely because
it shares a compilation or construction unit. A function with no formal
arguments therefore has no mechanically inferred caller precondition from a
purely global assertion; that assertion remains definition-time verification,
or it may be exposed deliberately through a manual contract.

Assertion endpoints include:

```text
assert
require
delete/reject branch
another explicitly specified verification endpoint
```

A parameter-related intermediate computation that only derives another symbol
is not printed as a standalone require. It may still be retained as a shared
node needed by an assertion slice and by later body continuation.

Automatic require is not a cheap symbol-existence test and is not an attempt to
avoid real compile computation. It extracts the compile graph that expresses
the callable's actual static preconditions.

## 10. Require Synthesis

### 10.1 Guarded require atoms

The contract layer has an explicit structured atom:

```text
GuardedRequireAtom {
    normalized_pattern_domain,
    guarded_alternatives,
    assertion_flow,
    provenance,
}
```

Its semantic identity is canonical over the normalized pattern domain,
semantically ordered guarded alternatives, and assertion-flow node identities.
Provenance is retained for diagnostics but does not by itself change equality.
A guarded atom is therefore cacheable, comparable, and inlineable by semantic
identity rather than by source spelling.

“Semantically ordered” follows `PatternValue` normalization: ordered layers
preserve position, while fully named `Set<PatternValue>` layers use canonical
set representation. Source branch spelling order is not reintroduced where the
underlying normalized pattern domain is unordered.

The external contract remains:

```text
RequireContract
  = GuardedRequireAtom1 && GuardedRequireAtom2 && ... && GuardedRequireAtomN
```

Top-level disjunction is never introduced. A compile match produces one
`GuardedRequireAtom` whose `guarded_alternatives` contains the normalized branch
alternatives. A runtime match produces multiple pattern-guarded atoms combined
by top-level conjunction.

### 10.2 Serial flow

For serial blocks:

```text
Req(A; B; C)
  = Req(A) && Req(B) && Req(C)
```

This conjunction preserves computation order. It is not an unordered Boolean
set and cannot be freely permuted when nodes carry dependency/provenance order.

### 10.3 Compile match

For a compile-policy scrutinee:

```text
Req(compile match)
  = one GuardedRequireAtom {
      guarded_alternatives = [
        (domain1, Req(branch1)),
        (domain2, Req(branch2)),
        ...
      ]
    }
```

Each branch term carries:

```text
its pattern extraction condition
its D-normalized residual domain
its branch assertion flow
```

The alternatives encode the internal normalized disjunction of one guarded
atom. They do not extend the external contract surface with arbitrary top-level
disjunction.

### 10.4 Runtime match

For a runtime-policy scrutinee, every runtime-reachable branch must have a legal
compile precondition flow:

```text
Req(runtime match)
  = GuardedRequireAtom(pattern1, Req(branch1))
    && GuardedRequireAtom(pattern2, Req(branch2))
    && ...
```

This must not be flattened to an unguarded:

```text
Req1 && Req2
```

The guarded contract is:

```text
(pattern1 => Req1)
&&
(pattern2 => Req2)
```

so the pattern domains remain part of each assertion atom.

### 10.5 External contract conjunction boundary

The external contract form remains a sequence/conjunction of contract atoms:

```text
RequireContract
  = GuardedRequireAtom1 && GuardedRequireAtom2 && ... && GuardedRequireAtomN
```

Complex compile alternatives are represented inside one
`GuardedRequireAtom.guarded_alternatives` value using existing pattern,
overload, and normalized branch structure. This note does not add arbitrary
top-level contract disjunction or a new Boolean theorem language.

### 10.6 Inferred and manual require

The total contract is:

```text
Require_total
  = Require_inferred && Require_manual
```

Manual require is neither an override nor an alternative. It may state stronger
interface promises or design constraints, but it cannot delete, mask, or bypass
the computations and assertions mechanically required by the body.

When body evolution adds compile computation, pattern derivation, or static
verification, the required assertion slice automatically enters
`Require_inferred`. The author does not carry a separate synchronization debt.

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
CompleteSymbolFlow or compile_projection
UnresolvedCallFamily or DerivedCompanionCallFamily
derived CompileCompanion entries
DerivedCallableEntryId
must_select_if_qualified
semantic companion replacement/suppression metadata
sealed_overload_name / closed_overload_set
policy-directed compile/runtime match elaboration
D/Done-normalized formal-dependent/guarded require slicing
GuardedRequireAtom and canonical atom identity
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
5. P1 controls one callable/value object's lookup participation; P2 controls
   callable-entry execution and exact argument policy.
6. external(P2) must be included in the selected object's P1.
7. There is no independent P3; result-symbol lookup policy inherits the
   selected callable object's P1.
8. compile_projection is structural and retains unresolved call families; it
   does not evaluate or select overloads.
9. A local callable flow is finite when call-family nodes are opaque; recursion
   remains on cross-call and summary-reference edges.
10. Runtime symbols retain their compile Pattern projection.
11. Compile companions are first-class derived entries, not hidden fallbacks.
12. Qualified must-select entries must be the final unique candidate.
13. Linear filters run in fixed normative order and are only internally
    independent of candidate enumeration order.
14. Explicit companion replacement/suppression is never inferred from priority.
15. Companion suppression closes an exported derived interface, not body checks.
16. Match staging follows scrutinee total_policy; no constexpr match family exists.
17. Require slicing distinguishes data dependency, control dominance, guarded
    domains, and backward assertion cones.
18. Require slicing consumes D/Done-normalized pattern flow.
19. GuardedRequireAtom gives branch contracts canonical structured identity.
20. Runtime branch requires are a conjunction of pattern-guarded atoms.
21. Require_total is inferred && manual.
22. Require and body continuation share one static evaluation graph.
```
