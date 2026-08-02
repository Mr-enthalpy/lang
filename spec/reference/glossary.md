# Glossary

Definitions are specific to this repository. Versioned entries distinguish the
frozen v0.1/v0.2 historical surface from the amended v0.5 contract. Terms may
have different meanings in general PL theory.

---

## v0.1.w Raw AST Stability Window

The maintenance and contract-stabilization window after the completed v0.1 Raw
AST Frontend. During this window richer literal spelling and the pipe
branch-name shorthand were implemented as the final v0.1.w additions. v0.1.w
is now closed; it was followed by the now-closed v0.2 freeze.

_See also: Raw AST, v0.2 Raw AST Contract Freeze._

---

## v0.2 Raw AST Contract Freeze

The closed historical stage after v0.1.w. It froze the then-current Raw AST
frontend input and prepared the exact boundary consumed by v0.3. Its documents
remain historical snapshots and are not rewritten for later parser changes.
`v0.2` was not a parser-expansion phase and did not implement Normalized AST.

_See also: v0.1.w, Frontend Semantic Amendment v0.5-A, Raw AST,
Normalized AST, raw-ast-contract-freeze-v0.2.md._

---

## Frontend Semantic Amendment v0.5-A

The versioned amendment that classifies post-v0.2 parser changes without
rewriting the frozen history. Closure orthogonalization and malformed-tail
error preservation are hard structural corrections; `DotClosure` is a
normalization-driven extension; `Ellipsis`/Pack and callable-tail alternatives
are new syntax amendments.

_See also: v0.2 Raw AST Contract Freeze, Raw AST Contract v0.5._

---

## Raw AST Contract v0.5

The current Raw AST contract obtained by applying Frontend Semantic Amendment
v0.5-A to the frozen v0.2 baseline. It defines the 20-symbol/33-diagnostic
surface, callable tail, first-class dot closure, Pattern pack, orthogonal
closure placement, and validated normalized handoff.

_See also: Frontend Semantic Amendment v0.5-A, PatternValidatedNormProgram._

---

## Token

The output of the lexer. A token is the smallest lexical unit: a `Name`, `Literal`,
`Symbol`, `Trivia`, `Invalid`, or `Eof`. Tokens carry a span and are consumed by
the parser. The lexer does not assign semantic roles to tokens.

_See also: Trivia, Name, Literal, Symbol, Span._

---

## Trivia

A token class representing whitespace, comments, or other non-semantic text.
Trivia tokens are skipped by the parser but their spans must remain available
for diagnostic positioning. The lexer must preserve trivia spans; the parser may
discard trivia after consumption.

Line and block comments are trivia. Nested block comments are a lexer concern
(depth counting on `/*`/`*/`), not parser syntax.

_See also: Token._

---

## Name

A token class representing an identifier. Names include what traditional languages
call keywords. In v0.1, `return` (contextually recognized in return terminal
forms at the form level; remains a `Name` token lexically), `else`, `match`, `drop`, `move`, `sync`,
`effect`, `fn`, `type`, `meta`, `runtime`, `compile`, `seal`, `const`, `mut`,
`public`, `private`, `export`, `namespace`, and `struct` are all ordinary
`Name` tokens at the lexical level.

> **Distinction**: A `Name` token is not a keyword. Semantic strength does not
> imply lexical keyword status.

_See also: Token, Strong context._

---

## Strong context

A parser state in which certain `Name` tokens or symbols are interpreted
structurally. Examples: `let` at form start, the active head clauses
`require`/`pre`/`post`/`lifetime pre`/`lifetime post`, `with` inside let bindings, `<>` in binding
contexts.

Outside a strong context, these tokens retain their ordinary `Name` or `Symbol`
identity.

_See also: Name, Hole, DeduceList._

---

## DeduceList

A sequence of hole declarations enclosed in `<...>`, recognized only in strong
binding contexts such as extract-let binders, closure heads, parameter binders,
and return binders. Outside these contexts, `<` and `>` are ordinary symbols;
in expression/operator contexts they may be operator spellings.

Normalized DeduceLists are left-to-right dependent telescopes. A declaration
annotation sees inherited and earlier declarations, never itself or later
declarations. A hole name cannot be repeated within one `PatternRoot`; a new
independent Pattern root may shadow an inherited spelling. Each declaration
receives an alpha-normalized `HoleBinderId` qualified by callable owner,
Pattern root, and root-local ordinal, and a named `HoleRef` targets that exact
identity rather than merely repeating its spelling. Build integration maps the
frontend owner to a persistent `SemanticOwnerId`. Source spans are provenance,
not identity. Generated
receiver holes use hygienic generated keys rather than source spelling. A
callable-head telescope scopes captures, parameters, policy, return, clauses,
body, and inherited nested callables. Within a BindingSlot, policy precedes
the local DeduceList. Norm exact binding covers Pattern/policy occurrences;
value-side names and navigation remain unresolved.

_See also: Hole, Strong context, CanonicalSkeleton._

---

## Hole

A binder declared in a `DeduceList` that acts as a wildcard standing for an
unknown type or value in following syntax. Raw canonical parsing may mark its
spelling with `CanonicalNameRole::Hole`; normalized uses carry an exact
`HoleBinderId`. The anonymous `_` annotation placeholder is not a named Hole
and targets no DeduceList declaration.

_See also: DeduceList, CanonicalSkeleton._

---

## SemanticOwner

A parent-linked semantic identity domain for namespace objects, callable
anonymous types, canonical meta-invocation instances, and generated objects.
Semantic identity is `(SemanticOwnerId, local identity)`; source file, span, and
printable path are provenance only. Every callable, including an in-place
closure, has a callable owner. Standalone closure materialization defaults to
an anonymous function-object type derived from that owner, but an associated
call-entry implementation may receive a different named receiver type. Source
navigation prints the current/innermost callable-local `Self` owner first and
enclosing owners to its right.

_See also: CallableReceiverType, PatternRoot, PackageBoundary, Mount._

---

## CallableReceiverType

The type of the caller object injected into invocation-frame slot 0. It is
independent of `CallableOwner`. For a standalone function object it defaults to
the owner-derived anonymous function-object type; for an associated `()` entry
it is the type whose namespace supplied that entry, such as `ref::T`.

The first written formal binds this object by position under any legal spelling.
Only later formals consume the explicit call-site Product. A mismatch is an
ordinary invocation type-check failure, not a separate `let ()` declaration
rule.

_See also: SemanticOwner, Callable Implementation Tail._

---

## Associated Val2 Contribution

A let-shaped declaration consumed inside `struct` construction that adds
ordinary value-facet material below the current Pattern owner without adding a
Val1 structural slot or Pattern extraction member. Its initializer may be
value-bearing and callable. The empty target `()` installs the current owner's
special call entry. Contributions remain uninstalled until the outer
construction commits its namespace delta.

_See also: DefaultExtractionView, SemanticOwner._

---

## PatternRoot

One independent Pattern/extraction alpha boundary inside a `SemanticOwner`.
Nested BindingSlots, Products, Sequences, annotations, DeduceLists, and Pack
operands inside an extraction retain the same root. An independent let Pattern
or callable head creates a new root. Hole names are unique within one root;
different roots may use normal lexical shadowing.

_See also: DeduceList, Hole, SemanticOwner._

---

## FullNameView

The complete package-internal namespace and overload view. Same-package
descendant owners may use an ancestor's non-export entries through lexical
lookup. Unrelated siblings do not acquire that visibility merely by sharing a
package.

_See also: ExternalNameView, DefaultExtractionView._

---

## ExternalNameView

The identity-preserving external namespace projection used after lookup crosses
a package boundary. It requires export-retention admission, public reachability
through every access-path component, and an externally eligible candidate
policy view.

_See also: FullNameView, PackageBoundary, Mount._

---

## DefaultExtractionView

The structural Pattern view exposed by default extraction. It is distinct from
both name views. Private structural members remain in the full structural model
but are absent from this view. Rich custom `?` construction remains future
design.

_See also: FullNameView, ExternalNameView._

---

## PackageBoundary

Build/namespace metadata assigning a stable `PackageId` to a namespace subtree.
`PackageOf(node)` uses the nearest boundary ancestor. Physical directory names
do not define package or symbol identity.

_See also: Mount, SemanticOwner._

---

## Mount

A namespace-graph redirect edge from an alternative access path to an existing
target node. Mount traversal may cross a package boundary and switch to
`ExternalNameView`, but it never copies the target symbol or changes its
identity.

_See also: PackageBoundary, ExternalNameView._

---

## CanonicalSkeleton

A syntactic pattern used in extraction contexts (extract-let binder, extract
parameter, extract return). The historical skeleton is a sequence of
`CanonicalElement` items. Under v0.5-A, Ellipsis may occur as a direct
canonical Pattern Sequence child and normalizes to `NormPattern::Pack` inside
`NormPattern::Sequence`; it is not hidden as a new skeleton atom. The parser
builds shape only and does not execute matching.

All canonical skeleton golden tests in v0.1 are parser preservation tests.
No semantic meaning (matching, destructuring, equality, constructor
interpretation, or admissibility) is assigned to any skeleton shape.
The `Hole`/`NodeName` distinction is a parse-time role marker, not a
semantic binding commitment.

_See also: DeduceList, Hole, ProductForm, CanonicalNameRole._

---

## ProductForm

A parenthesized form with top-level commas, such as `(a, b, c)`.

In expression context, a product form is product construction and is preserved
as `ProductExprAst`. In binding / extraction context, the same surface form is
product extraction and is preserved as `ProductExtractAst` or a canonical
product extraction skeleton.

Leading, doubled, or trailing commas create explicit unit product elements.
These unit elements are not omitted, not wildcards, and not implicit discards.

The parser does not decide whether a product is constructible, destructible,
layout-compatible, type-compatible, or callable. ArgPack and ArgPackRole are
removed historical terms and are not language-level concepts.

_See also: ProductExtract, Segment, PipeExpr._

---

## PipeExpr

A top-level expression formed by splitting tokens at `|>` into segments.
`PipeExpr` is the entry point for expression parsing.

```text
PipeExpr ::= Segment ("|>" Segment)*
```

_See also: Segment, ProductForm._

---

## Segment

One part of a `PipeExpr`, containing a sequence of `OperatorExpr` and product
elements in the operator-aware design. Each segment has a `has_incoming` flag
indicating whether a prior segment exists.

_See also: PipeExpr, Atom, ProductForm._

---

## Atom

The smallest self-contained expression unit. Atoms include:

- `Name("x")`
- `IntLiteral("42")`
- `StringLiteral("\"text\"")`
- `Group(PipeExpr)`
- `Closure(ClosureAst)`
- `NavPath(components)` (components are `NavComponentAst` in source order)
- `DotClosure(selector)` (leading `.name`; no captured receiver)
- `MemberSugar(object, selector)` (selector is `SelectorAst`)
- `DoubleDotSugar(object, selector, args)` (selector is `SelectorAst`)
- `BracketCallSugar(object, operator, args)` (`obj[args...]`; operator spelling `[]`, `args` is a `ProductExprAst`)
- `Error`

Atoms are constructed by parsing a base and then folding suffixes (`::`, `.`,
`..`, `[...]` bracket call, and postfix operators). Operator sugar itself is
stored at the `OperatorExpr` layer, not as a general `Atom` variant.

Leading `.name` is a base atom, distinct from suffix folding.

`BracketCallSugar` is source-preserving sugar for the operator spelling `[]`; it
is not indexing/slicing/container access. The `[]` operator is a contextual
paired operator name, bindable/aliasable/referable in operator-name positions.

_See also: ClosureAST, ProductForm, OperatorSugar, PostfixOperator, SelectorAst, NavPath._

---

## SelectorAst

A name-like construct appearing after leading `.` or in suffix position after
`.` or `..`.
In the current parser phase:

```text
SelectorAst ::=
    Text(NameAst)     // from TokenKind::Name
```

Numeric selectors have been removed. Only `Name` selectors are accepted.

_See also: NameAst, NavComponent, MemberSugar, DoubleDotSugar._

---

_See also: NavPath, SelectorAst, OperatorName._

## OperatorName

A symbol spelling that can be used as an operator identity component, an
expression operator, a binder name, or an innermost navigation component.
Operator names are not keywords, and their spelling does not imply arithmetic, comparison,
mutation, assignment, lookup, or evaluation semantics.

An overloadable operator identity is `spelling + fixity + arity`, where fixity
is `Binary` or `Postfix`. `Prefix` fixity is a Raw AST marker reserved for
the prefix-negative surface sugar `-x` (normalized away before operator lookup);
it is not an overloadable operator fixity.

_See also: Fixity, Arity, NavComponent, OperatorSugar, PrefixNegative._

---

## Fixity

The syntactic position of an operator relative to its operands. The operator
design distinguishes:

- `Binary` and `Postfix`: overloadable operator fixities (part of operator
  identity for declaration, alias, and lookup).
- `Prefix`: a Raw AST surface marker used only for the prefix-negative `-x`
  sugar. Prefix negative is normalized to typed-zero binary subtraction before
  operator lookup. The `Prefix` fixity is not a declarable or overloadable
  operator fixity.

_See also: OperatorName, Arity, PrefixNegative, PostfixOperator._

---

## Arity

The number of operands associated with an operator syntax form. Arity is part
of operator identity.

_See also: OperatorName, Fixity._

---

## OperatorSugar

An AST shape inside `OperatorExprAst` that preserves operator syntax without
lowering it to an ordinary call. Planned shape:

```text
OperatorExprAst ::=
  | OperatorSugarAst {
    operator: OperatorName,
    fixity: Prefix | Postfix | Binary,
    args: Vec<OperatorExprAst>
  }
```

Operator lookup is a future semantic pass and follows ordinary visible binding
lookup, not ADL or type-directed parser lookup.

_See also: OperatorName, Fixity, Arity._

---

## Overload Candidate

A callable entry prepared for a given call. Final preparation first resolves a
symbol, projects its heterogeneous value facet for the current policy view,
enumerates `Val2` objects, obtains each surviving value's type, and resolves
that type's associated `()` entry. Non-callable values are discarded. A derived
compile companion is itself a complete `Val2` function object with stable
object identity, origin runtime object, its own function-object type, and its
own associated compile `()`; it enters candidate preparation through the same
path. Compile projection leaves the projected invocation as an ordinary call
until normal compile lookup and overload resolution. A same-name namespace
bucket is only current transitional substrate, not the final candidate
definition.

_See also: OverloadSpecificity, OverloadResolutionPipeline,
`spec/design/patterns-overload/overload-resolution-design.md`,
`spec/design/symbol-world/symbol-policy-and-compile-flow-projection.md`._

---

## Fully Admissible Candidate

An overload candidate that has passed every hard legality check for the current
call: namespace/policy-view visibility, associated `()`, argument/Pattern shape,
receiver and parameter policy pairs, stage legality, any target-result policy
constraint, expected result rank/facet, concept and ordinary require
satisfaction, and other compile/type prerequisites. The set of all such
candidates is `A`.

Preference survivors are not a second meaning of "qualified." They are the
successive subsets obtained by applying the fixed ordered preference filters to
`A`.

_See also: OverloadCandidate, OverloadResolutionPipeline, MustSelectStrategy._

---

## Derived Compile Companion Object

A complete compile-policy `Val2` function object mechanically derived from an
eligible runtime function object. It has its own object and type identity, an
associated compile `()` entry, stable provenance back to the origin runtime
object, and the `must_select_if_qualified` overload strategy. It is not a hidden
fallback or an identity-less extra call entry.

_See also: OverloadCandidate, MustSelectStrategy._

---

## Must-Select Strategy

An overload strategy carried by a `Val2` function object and propagated to its
prepared call candidate. It activates only when that candidate belongs to the
fully admissible set `A`. One admissible must-select candidate must be the sole
final preference survivor; several admissible must-select candidates conflict.
The strategy is not infinite priority and does not forbid non-overlapping
same-name overloads. Source strategy metadata uses `=> name { ... }`, with
`[[name]] { ... }` as the no-`=>` disambiguation form; `@` remains lifetime
syntax.

_See also: FullyAdmissibleCandidate, DerivedCompileCompanionObject._

---

## Const/Mut Product Order

The overload preference relation for value mutability. At one constrained
position, a const actual prefers `const`, then unspecified, then `mut`; a mut
actual reverses the endpoints. Across receiver, parameters, and a target-result
constraint when present, candidates are compared by product partial order. A
candidate dominates only when it is no worse everywhere and strictly better
somewhere. Incomparable maxima remain ambiguous; there is no score,
exact-match count, position weight, or lexicographic fallback. Delete members
participate in the same comparison.

_See also: FullyAdmissibleCandidate, OverloadResolutionPipeline._

---

## Seal Visibility

Seal slices are exposed only in SealStatic; meta slices only in OpenStatic;
compile slices in both. Symbol resolution precedes this exposure, so a hidden
slice does not erase the symbol. Seal policy grants no global scan capability.
Compiler-known privileged seal operations may inspect exactly Wpre, the least
semantic dependency closure rooted at exported symbols, actually materialized
results of exported meta functions, and their parameter/signature dependencies.
Wseal never expands that scan domain, though committed Wseal symbols remain
explicitly addressable.

_See also: PolicyPair,
`spec/design/symbol-world/symbol-policy-and-compile-flow-projection.md`._

---

## Overload Specificity

The priority rule that determines which overload candidate is selected when
multiple candidates survive initial filtering. In this design, overload
specificity is **extraction-pattern specificity**: candidates are ranked by
how deeply their extraction pattern penetrates the unified construction-
expression tree of the call operand. Structural depth evidence is compared
before node-class evidence. At equal depth, ordinary explicit matches outrank
explicit pack matches, which outrank ordinary discards, which outrank pack
discards. Captured length never changes the count. A simple pack has one
outward evidence node. Raw `...(a, b)` is non-canonical, and a legal headed
structured operand still contributes only one outward Pack node at its
containing level; internal evidence stays below the stable head at the next
structural level.
Specificity does not depend on declaration order or an ad-hoc conversion-rank
table. This extraction-only rank is not a const/mut fitness score and never
resolves candidates that remain incomparable under the const/mut product order.

_See also: OverloadCandidate, OverloadResolutionPipeline,
`spec/design/patterns-overload/overload-resolution-design.md` §4._

---

## Overload Resolution Pipeline

The fixed process that selects a unique overload candidate. Path resolution and
the current policy view enumerate `Val2` objects. Associated-call preparation
and every hard structural, Pattern, policy-pair, stage, target-result, concept,
and ordinary-require check first form fully admissible set `A`. `Bp` then uses
the Policy product partial order across all constrained positions; no total
score or lexicographic fallback resolves incomparable candidates. For an
authorized atomic Runtime-migration call only, input/output endpoint Policy fit
extends this same product as `Bp'`. With no endpoint coordinates, `Bp'` is
exactly old `Bp`. Remaining
side-effect-free preference filters apply in one fixed normative order:
entry, concept, extraction, first-order-over-instantiated,
in-place-over-non-in-place, then named strategy rules. Each
filter is independent of candidate enumeration order; filters are not assumed
to commute. A named strategy only sees fully admissible candidates and cannot
restart lookup. Delete members participate normally, and ordinary uniqueness is
constrained by `must_select_if_qualified` strategies activated from `A`.

Current source cannot construct a fallback candidate role, so current calls
have `Af = A`. If a future fallback strategy is exposed, its fixed semantics
will insert `SuppressFallback(A)` before Bp: any admissible non-fallback
candidate, including `delete`, suppresses fallback permanently. This future
suppression is not B6 and later failure cannot restore fallback.

Lifetime policy is not a type/compile candidate filter. This revision defines no
lifetime-driven re-selection, refinement order, ABI class, or second selection
stage. Any future lifetime check receives the already unique ordinary overload
result, under the boundary in
`spec/design/lifetime/lifetime-policy-and-overload-boundary.md`. That is a
restriction on lifetime *rules*; `@` itself is an ordinary overloaded operation
with its own candidate set.

Full overload resolution is deferred to v0.10+ and depends on the pattern-space
and extraction-chain infrastructure. The formal specification is in
`spec/design/patterns-overload/overload-resolution-design.md` §5.

_See also: OverloadCandidate, OverloadSpecificity, Concept,
`spec/design/symbol-world/symbol-policy-and-compile-flow-projection.md`._

---

## Lifetime Policy Boundary

The stage boundary between lifetime rules and ordinary overload selection:
lifetime policy is not an ordinary stage policy atom, and no lifetime rule may
reopen or change the already unique ordinary overload result. `@` is evaluated at
a stage; it does not name one.

This boundary is *not* a claim that `@` lacks semantics or overloads. `@` is the
carrier-slot observation `E@ = ObservePlace_policy(CarrierPlace(E), Value(E))`
with two positively defined overload groups (`LifetimeFact` for objects carrying
an internal `Val1` payload; `P ref` for effectively open pure pattern slots).
What remains undefined is the region/origin algebra, checking order, refinement
phase, and handoff object.

_See also: `@`, Escape check, `spec/design/lifetime/lifetime-policy-and-overload-boundary.md`._

---

## Object normal form (`Norm`)

The structural identity of an object. Every object has three components:

```text
Object x    = ⟨ Val1?(x), P(x), Val2(x) ⟩
Val1?(x)   ∈ 1 + Object
```

`Val1?(x) = null` means only that the object carries no internal `Val1` payload;
it is not a separate ontology. `Norm(x)` is the recursive normal form over all
three components; there is no `Val1`-presence fork that drops a component.
`ObjectPlaceId`, `SymbolId`, allocation order, and construction provenance are
not part of `Norm(x)`. Where a complex `Val1` cannot yet be normalized
structurally, an opaque summary is a safe under-merge, never a definition of
identity.

_See also: Policy Pair, Borrow view, EffectiveOpen._

---

## `@`

The carrier-slot observation operation:

```text
E@ = ObservePlace_policy( CarrierPlace(E), Value(E) )
```

On a pure pattern slot the selected candidate is
`E@ = RefCarrierSlot( CarrierPlace(E) )`.

`CarrierPlace(E)` is the carrier slot through which `E` was read; `ref` and
`share` consume only `Read(E)` and never ask for it. A freshly computed temporary
supplies no carrier place, so no `@` candidate applies to it.

`@` is **not** a general `PlaceOf(E)`. It has exactly two positively defined
overload groups with disjoint premises. For `Val1?(x) ≠ null` — a complete
`⟨Val1, P, Val2⟩` object — `@` takes that object's lifetime, yielding a
`LifetimeFact` at the lifetime policy stage; that is a fact, not a borrow, and it
is unaffected by the narrowing of the other group. For `Val1?(x) = null` with
`EffectiveOpen(x, context)` it yields `P ref`. The second group is why `@`
exists: an ordinary read of a pure pattern slot selects the pattern value and so
hides the carrier slot, and `ref` has no basis for guessing otherwise. There is
deliberately no compile-stage borrow-producing `@` candidate for a value-bearing
operand — `s ref` already does that job. When the target is not effectively open,
the failure is "no applicable overload for `@`", not a post-hoc rejection. `@` is
not a stage name and not an ordinary policy atom.

_See also: Borrow view, EffectiveOpen, Lifetime Policy Boundary, `type ref`._

---

## Borrow view

An ordinary value that observes another object without owning it. What `ref` and
`share` observe is the value the expression read:

```text
Read(Σ) = Val1(Σ)                 when Val1(Σ) ≠ ⊥
Read(Σ) = ⟨ ⊥, P(Σ), Val2(Σ) ⟩    when Val1(Σ) = ⊥

E ref    = Ref( Read(E) )
E share  = Share( Read(E) )
E@       = RefCarrierSlot( CarrierPlace(E) )
```

Whether `ref` or `@` is the right operation is decided by the presence of a
`Val1` payload, never by type-rank: for `s : symbol` the payload exists, so
`s ref : symbol ref` borrows the symbol value `s` carries — not the binding slot
that carries `s` — and a type-rank object with a payload behaves the same way.
For `let t: type = uint8`, `t ref` is `uint8 ref` (a correct borrow of what was
read) and only `t@` yields `type ref`.

A borrow view is a value, not a second name for a symbol: it does not forward
`SymbolId` or `PlaceId`, and its member set is not silently that of its target.
Borrowing is idempotent (`Borrow(Borrow(q)) = Borrow(q)`), so no provenance
chain, forwarding chain, or cycle detection is required. `r_ref = v` writes the
referent; `r_ref rebind = E` changes which object the view observes.
`OwnedClosure(x)` excludes every `ref` / `share` edge.

Borrow views replace the retired alias-forwarding model. Canonical owner:
`spec/design/symbol-world/type-values-places-and-borrow-views.md`.

_See also: `@`, Escape check, Alias binding (retired semantics), `type ref`._

---

## Escape check

The check that a borrow view is not stored where it can outlive what it
observes:

```text
Escapes(view, destination)
  = Region(destination) ⊄ ObservationRegion( Origin(view) )
```

It applies to the destination classes that can outlive an observation region
(global/normalized structures, returned values, captured closure state, and
longer-lived member slots). It is a property of the destination and the
observation region only; it is not an RHS-provenance or construction-history
check on assignment.

_See also: Borrow view, `@`, Lifetime Policy Boundary._

---

## EffectiveOpen

The premise that a construction target is still extensible at a given context:

```text
EffectiveOpen(x, c) = StateOpen(x)
                    ∧ ConstructionAnchorCompatible( owner(x), c )
```

The state transition is one-way: `Open -> Frozen`, never `Frozen -> Open`.
Global lifetime does not imply `EffectiveOpen`. Inside a meta instance body the
ordinary freezing events do not fire; sealing happens only at the meta return
stage. `EffectiveOpen` is a premise of the `P ref` group of `@` and of
`inject` input validity, so violating it produces "no applicable overload",
not a late rejection. `Open_Γ(x)` is the same fact written in judgment form, with
`Γ` supplying the context argument.

Canonical owner:
`spec/design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md`.

_See also: `@`, `inject`, Borrow view._

---

## `inject`

A pure function from a type (or a `type ref` view of one) plus child pattern
material to a new type:

```text
inject : ( type | type ref ) x ChildPatternMaterial ⇀ type

Inject(old, Δ) ⇓ new
```

`Inject` does not modify `old`, does not install a namespace delta, and does not
perform an assignment. Failure is total: no partial result, no write, no
rollback. Observing the result in a place is an ordinary write, spelled as three
steps (`old = Read(t_ref); new = Inject(old, Δ); Write(t_ref, new)`). `inject`
extends only the direct child patterns of its input; anything else is "no
applicable overload".

Input validity has two overloads, and the `Open` fact comes from a different
place in each:

```text
Injectable_Γ(x : type)        = Open_Γ( ConstructionRoot(x) )
Injectable_Γ(x : type ref)    = true
Injectable_Γ(x : type share)  = false
```

A by-value `type` carries no `Open` capability, so the evaluation context must be
asked. A `type ref` needs no second query because `x : type ref` already implies
`Open_Γ(Target(x))`.

_See also: EffectiveOpen, Meta-function, Borrow view, `type ref`._

---

## `type ref`

A borrow view of a carrier slot whose object is a pure pattern value. It is
formed only by `@`, only inside the target's open window:

```text
Carrier(t) = q      Open_Γ(q)
---------------------------------
Γ ⊢ t@ : type ref
```

A `type ref` is not merely `⟨Place, type⟩`; it is capability-equivalent to

```text
⟨ Place, type, OpenWitness ⟩
```

so that the invariant

```text
Γ ⊢ r : type ref  =>  Open_Γ( Target(r) )
```

holds. The `OpenWitness` need not exist as a runtime field, but it must be an
unforgeable fact of the static judgment. Consequently the holdable interval of a
`type ref` is the `Open` window itself, not "`Lifetime(Target)` has not ended
yet": once the window closes the view cannot continue to exist as a usable value,
and the only way past the boundary is to weaken to `type share` beforehand.

This is what lets a `type ref` satisfy `inject` with no ambient query, and what
makes returning or storing one an escape-check question rather than a blanket
prohibition.

`type share` keeps observability but positively renounces structural extension:
it has no applicable `inject` overload.

_See also: `@`, `inject`, Borrow view, Escape check, EffectiveOpen._

---

## PostfixOperator

A unary operator suffix that composes with other atom suffixes. In the
operator-aware design, postfix operators do not terminate suffix parsing, so
`obj!.field` has the same shape as `(obj!).field`.

_See also: OperatorSugar, Atom, NavPath._

---

## PrefixNegative

Parser-preserved prefix-negative surface syntax. The parser produces
`OperatorSugar { fixity: Prefix, operator: "-" }` for `-x`. It is not a
negative literal; the lexer produces `-` and the following literal or atom
separately.

Normalization rewrites prefix negative to typed-zero binary subtraction:

    -x  ⟶  ()zero::(x |> type) - x

Prefix negative is not an overloadable operator identity. The spelling `-`
as a declarable or aliasable operator identity refers only to binary minus.
Only the generated binary `-` participates in operator lookup after
normalization.

_See also: OperatorSugar, Fixity, OperatorName._

---

## NavPath

A source-order inner-to-outer navigation chain separated by `::`.

```text
NavPath ::= NavComponent "::" NavOuterComponent ("::" NavOuterComponent)*
```

The leftmost component is the innermost selected symbol. The rightmost
component is the outermost scope component. Raw AST preserves navigation
components in source order and performs no lookup.

_See also: NavComponent, OperatorName, Atom._

---

## NavComponent

A component in a `NavPath`:

```text
NavComponent ::= Name | OperatorName | GroupedExpr | Error
```

Operator names are valid only as innermost navigation components unless a
future design explicitly allows operator-named scopes. Parenthesized
right-side scope expressions after `::` are preserved as grouped components.
A grouped expression is valid only as an outer component; used as the innermost
component (`(int Vec::std)::ns`) it emits `InvalidNavComponent`. Without
parentheses, `::` consumes only one immediate valid component.

_See also: NavPath, SelectorAst, OperatorName._

---

## EntityRef

A compile-time entity reference syntax. Phase 4.2 defines the design; Phase
4.4 implements a raw `EntityRef` parser inside alias-let RHS only. `EntityRef`
is not a runtime expression, not a `PipeExpr`, not a product form, not a
closure, and not resolved by the parser. EntityRef parsing is not a general
expression parser mode.

Provisional grammar:

```text
EntityRef ::= EntityComponent ("::" EntityOuterComponent)*
```

In the current implementation, `EntityRef` appears only on the right-hand side
of `let binder === EntityRef`. Other possible strong contexts are future work.

_See also: NavPath, NavComponent._

---

## Compile-time entity reference

The conceptual role of `EntityRef`: a source-level reference to a compile-time
entity that may later be resolved by semantic/name-resolution phases. It does
not denote a runtime value and is not checked for existence by the parser.

_See also: EntityRef, NavPath._

---

## EntityRef navigation

The navigation syntax inside a future `EntityRef`:

```text
EntityComponent ::= Name | OperatorName
EntityOuterComponent ::= Name | Group
```

EntityRef navigation is inner-to-outer and preserves source-order components.
An operator name is allowed only as the innermost component unless a future
design explicitly allows operator-named scopes. A grouped expression is valid
only as an outer navigation component after `::`; a grouped expression used as
the innermost component (`(int Vec::std)::ns`) emits `InvalidEntityRef`. The
parser does not perform operator lookup, name lookup, namespace resolution, or
existence checking.

_See also: NavPath, OperatorName, EntityRef._

---

## Alias binding

The frozen *parser* declaration form `let binder === EntityRef`. Phase 4.4
implements raw parser preservation: the parser produces `LetAliasAst` with
`AliasBinderAst` and `EntityRefAst`. Alias binding is not runtime value binding,
not an expression, not equality, not operator syntax, and not package import
syntax. No target resolution, operator identity validation, or entity lookup is
performed.

> **Semantic direction: retired.** The semantic reading of this form — a
> compile-time lookup alias that forwards symbol identity, place, or writability
> to a target — is retired, not deferred. There is no declaration form that
> forwards a Symbol or a place. `let a = b` creates a fresh symbol in a fresh
> place carrying `b`'s value (`SymbolId(a) ≠ SymbolId(b)`,
> `PlaceId(a) ≠ PlaceId(b)`, `Value(a) = Value(b)`). Shared observation of another
> object is expressed only by a borrow view (`ref` / `share` / `@`). Operator-name
> binding is the one surviving use of a dedicated binding form; its final surface
> spelling is open. See
> `spec/design/symbol-world/entity-alias-design.md` (retirement notice) and
> `spec/design/symbol-world/type-values-places-and-borrow-views.md`.

> **Distinction**: Alias binding is implemented as raw parser preservation
> only. It is not an ordinary `let name: annotation = expr`. It has no `=`
> value expression, no declaration annotation, no `guard`, and no `with`.
> EntityRef parsing is implemented only inside alias-let RHS.

_See also: Lexical alias, Entity alias, AliasBinder, Operator alias, EntityRef,
Borrow view._

---

## Lexical alias

**Retired semantic term.** It named a compile-time lookup name introduced by
alias binding into a lexical scope, shadowing previous bindings of the same name
without mutating the original entity. That forwarding-based scope/target model is
retired: no declaration form forwards lookup. Ordinary shadowing by a fresh `let`
binding covers the shadowing behavior; observing another object is a borrow view.
The `LetAliasAst` shape it described remains a frozen parser fact.

_See also: Alias binding, Entity alias, Borrow view._

---

## Entity alias

**Retired semantic term.** It named a lexical alias whose target is a
compile-time entity reference (`EntityRef`), binding a name or operator to a
compile-time entity path. Target resolution for this reading is retired, not
future work. `EntityRefAst` preservation in alias-let RHS remains a frozen parser
fact.

_See also: Alias binding, Lexical alias, EntityRef._

---

## AliasBinder

The binder position in a `let binder === EntityRef` form. It may be a
`Name` or `OperatorName`. The parser preserves the binder as raw AST syntax
without resolving the target entity.

_See also: Alias binding, Operator alias._

---

## Operator alias

An alias binding whose binder is an `OperatorName`. This is the one surviving
direction of the retired alias family: binding a name to an *operator identity*
is not symbol/place forwarding, and it must not be generalized back into a
general aliasing feature. Operator aliases are parser-preserved as Raw AST.
Later validation may require the operator binder and the innermost operator
component of the target `EntityRef` to have the same overloadable operator
identity (`spelling + fixity + arity`, where fixity is `Binary` or `Postfix`).
Prefix negative is not an overloadable operator identity and cannot appear as an
operator-alias binder or target. An operator alias cannot rename one operator
spelling into another. Its final surface spelling is an open question. Operator
alias validation is future static validation or name-resolution work, not current
parser behavior.

_See also: Alias binding, AliasBinder, OperatorName, EntityRef._

---

## Non-associative operator

An operator class that cannot be chained without explicit grouping in the
operator-aware parser design. Comparison, equality, and equals-suffixed
operators are non-associative in the current Raw AST frontend, so `a < b < c`,
`a == b == c`, and `a += b += c` require grouping.

Semantic validity of grouped expressions remains outside parser scope.

_See also: OperatorSugar._

---

## ClosureAST

The AST representation of a closure literal before materialization into a
callable object:

```text
ClosureAst {
  placement: InPlace | Ordinary,
  head: Option<FnHeadPrefixAst>,
  body: ClosureBodyAst
}
```

Placement and head presence are orthogonal. Bare `{ ... }` is headless
in-place; a headed block without `=>` remains in-place; `=>` selects ordinary
placement.

> **Distinction**: `ClosureAST` is **not** `ClosureObject`. Closure literals
> produce AST first. A later semantic pass may materialize closure AST into
> callable objects.

> **Distinction**: Bare `{ ... }` in atom position is an in-place `ClosureAst`,
> not a normal block expression.

_See also: ClosurePlacement, InPlaceClosureAST, OrdinaryClosureAST,
ClosureObject, Materialization._

---

## ClosurePlacement

The independent closure dimension `InPlace | Ordinary`. A no-`=>` body is
in-place even when it has a head or `[[strategy]]`; `=>` selects ordinary
placement. Placement is not inferred from `head.is_some()`.

_See also: ClosureAST, InPlaceClosureAST, OrdinaryClosureAST._

---

## Capture Clause

The ordinary-closure head component `[CaptureItem, ...]`. Each item is a
let-shaped binding: `[let x = E]` and `[x = E]` are explicit forms, while
`[E]` is shorthand only when normalized `E` has exactly one distinct free bare
name occurrence that is not a direct callable target. Policy-bearing captures
retain `let` to anchor the binding policy.

Capture initializers are simultaneous: every initializer sees the enclosing
environment before the clause. Normalization removes the explicit/inferred
surface distinction and produces `NormCapture { slot, initializer, origin }`.
This does not perform name resolution, environment layout, or closure
materialization.

Every source-written item is an explicit capture requirement. `[x]` is
shorthand for `[let x = x]` with the ordinary unwritten capture-policy domain
(`const || mut`), not an automatic const capture. A future resolved stage may
add a separate `ImplicitConst` requirement for an otherwise uncaptured free
outer value reference. Capture requirements are abstract dependencies: they
do not declare `self` fields, copy/reference representation, layout, ZST
status, or ABI.

External explicit navigation reaches the namespace export view; internal
explicit navigation reaches the complete namespace view. Because exported
value views are const-projected, externally navigated values and callable
targets normally satisfy `ImplicitConst` dependencies. Automatic capture and
call resolution share the symbol-identity/const-view problem domain; this does
not imply pass ordering or an implementation dependency.

An explicit capture and an automatic capture may name the same source but
remain distinct dependency declarations. Explicit capture can rename, project
policy, use a complex initializer, request `mut`, and preserve provenance.
Only a later layout pass may coalesce equivalent storage/link requirements.

_See also: BindingSlot, NormClosure, Materialization._

---

## InPlaceClosureAST

A `ClosureAst` whose placement is `InPlace`. It may be the bare, headless
`{ ... }` form or a headed no-`=>` block, optionally with `[[strategy]]`.
In-place closures never have capture lists or independent capture
environments. Having no extraction head is not the same as having a unit
extraction pattern: a headless in-place closure accepts no extracted input,
including no implicit unit input.

In future callable materialization it may contribute an overload candidate
while remaining tied to its embedding control-flow layer. Unresolved outer
reads are resolved lazily at that layer; no capture list is required or
allowed. Direct writes to a place outside the closure-local scope are
forbidden; local mutation and effectful calls remain possible. An
otherwise tied in-place candidate is preferred after the
first-order-over-instantiated filter.

> **Explicit self position for return:** A headless in-place closure still has
> a callable owner, a callable-local `Self` space, and an invocation-frame
> caller/self slot, but it has no written binder for that slot. If it is later
> materialized as a standalone function object, its receiver type defaults to
> the owner-derived anonymous callable type; that default is not part of
> return-target identity. The headless form therefore cannot name its own
> return target through a first-formal binder. Early-return examples that
> target a specific closure should use an in-place closure with an explicit
> product/extraction head carrying the self position, e.g.:
>
> ```lang
> (<Self: type> self: Self) {
>   () |> (Self return);
> }
> ```
>
> `Self` and `self` are replaceable positional binders, not reserved
> names. The same positional structure with different names:
>
> ```lang
> (<R: type> this: R) {
>   () |> (R return);
> }
> ```
>
> The return target is not the spelling `Self`; it is the target
> syntax in the explicit target position, resolved later by
> semantic target binding.
>
> The example fragment above is a headed in-place closure. The same shape is
> accepted as a standalone expression atom or in an incoming pipe/branch form;
> its placement remains in-place in either context.

_See also: ClosureAST, OrdinaryClosureAST._

---

## OrdinaryClosureAST

A closure literal whose placement is `Ordinary`, selected by `=>`. It has an
explicit head and a callable implementation tail.
The head may contain deduce list, capture clause, parameter clause, call-result
policy clause, return clause, and head clauses. The tail preserves ordinary or
named user body, compiler-defaulted implementation, or deleted implementation.
As for every callable placement, the first written formal Pattern denotes the
implicitly passed caller-object self slot; only later written formals consume
the explicit call-site Product.
Plain no-`=>` block tails and `[[name]]` stay in-place; the latter is only the
named-strategy escape that does not steal the established return
extraction-pattern parse.

_See also: ClosureAST, InPlaceClosureAST, FnHeadPrefix._

---

## NormClosure

The normalized closure carrier. It stores
`placement: NormClosurePlacement`, optional normalized head, implementation
body, and `NormOrigin` independently. `NormClosurePlacement` is only
`InPlace | Ordinary`; generated lowering provenance belongs to
`NormOrigin::Generated`, never to placement.

_See also: ClosurePlacement, Origin, Dot Closure._

---

## Callable Implementation Tail

The single syntax slot that describes a callable implementation and optional
overload strategy. It normalizes to `UserBody(Ordinary|Named, body)`,
`Defaulted`, or `Deleted(message?)`. `=> name {}` and `[[name]] {}` carry the
same named strategy. Strategy metadata participates only after full
admissibility and never creates a second overload pass. Product/closure
classification and capture-slot bypass require the complete `[[Name]] {`
shape; Deduce alone leaves capture parsing available, and the weaker `[[`
prefix is recovery-only after a later head component has proved the strong
context.

_See also: OrdinaryClosureAST, Fully Admissible Candidate, Overload Resolution Pipeline._

---

## PatternValidatedNormProgram

The downstream handoff produced by `normalize_and_validate_patterns` after all
currently enforced global normalized Pattern invariants have passed: one Pack
per structural level, no bare Product Pack operand, and no duplicate
DeduceList hole in one `PatternRoot`. Its certificate is intentionally
narrow: it does not prove ordered/unordered Pack applicability, stable
Pattern-head identity, complete matching support, parser-diagnostic absence,
or recovery freedom. `normalize_program` alone remains useful for
diagnostic/recovery dumps but does not authorize build-world harvesting.

_See also: Normalized AST, Pack Pattern, Raw AST Contract v0.5._

---

## Dot Closure

The first-class expression `.name`, normalized to a generated in-place
`NormClosure` carrier shaped as
`(self, val: T, ...args) { (val, args) |> name::T }`. The generated first
formal is the implicitly supplied callable object; `val` is the first explicit
call-site argument. `E.name` is compact
`E |> .name`; `.name` itself captures no receiver. After lowering it is an
ordinary expression. Replacing it with a bound equivalent must preserve the
same pipe/product binding spine, and no normalizer rule may inspect
`DotClosureLowering` provenance to absorb surrounding syntax. Only explicit
binding or call context materializes the carrier as a value; normalization and
other expression contexts do not. `..name` remains direct member-call sugar.

_See also: Atom, Function Object, Call normalization._

---

## Pack Pattern

The Pattern-side remainder form `...Q`. It matches the unmatched portion of
one normalized structural level and then applies `Q`. Each level permits one
pack; nested levels are independent. It is not a value/type/ABI category and
has no RHS unpack counterpart. Every Pack contributes one outward specificity
node, independent of captured length and internal node count.

At an unordered named layer only a whole-remainder binder/discard (including a
transparent let-shaped slot) is admissible. At an ordered layer a structured
operand may be meaningful only if its P-normal form retains a stable top mode,
for example `...((a, b) pair)`. Raw `...(a, b)` is preserved by the parser but
rejected after P normalization: Pack cannot reify the bare Product boundary
that ordinary Product normalization removes. Internal evidence below a stable
operand head belongs to the next preserved level; it is never flattened into
multiple same-level EP nodes.

Pack is valid syntax in every let-shaped binding slot, including ordinary/local
let, parameter, return, and nested product extraction; it is not a
parameter-only variadic form. It may be a direct canonical Pattern Sequence
child: `a ...x b` normalizes as `Sequence[a, Pack(x), b]`. Ellipsis consumes one
following Pattern primary. Only Product and Sequence establish cardinality
levels; Pack and BindingSlot are transparent. The parser preserves all formed
Pack nodes, and the normalized Pattern validator is the sole authority for
cardinality and the bare-Product rejection.

_See also: Pattern normalization, Overload Specificity._

---

## ClosureObject

A materialized, callable object produced from a `ClosureAST` by a future
semantic pass. In v0.1, closure objects do not exist. The parser produces
only closure AST.

> **Distinction**: `ClosureObject` is a semantic concept, not a parser concept.
> Materialization is explicitly out of scope for v0.1.

_See also: ClosureAST, Materialization._

---

## Materialization

The future semantic pass that converts `ClosureAST` into a `ClosureObject`.
Before this pass, resolved capture dependencies must first become
lifetime-checkable source/access/storage-or-link forms. Materialization may
then select static links, constant embedding, zero-layout dependencies, stack
environments, stored checked references, or other future representations. A
capture list is not itself an environment-field declaration. This is not
implemented in v0.1.

_See also: ClosureAST, ClosureObject._

---

## Meta-function

A callable whose entry executes with `P2 = meta` and constructs a
`SymbolConstruction` under symbol-world construction capability. An
ordinary user meta-function receives rank-constrained semantic values, creates
an ordinary canonical `MetaInstanceScope`, and has no unrestricted AST access.

Compiler-defined `BuiltinPrivilegedAstMetaFunction` objects are a separate
subclass. A member such as `struct` or `inject` may accept one specifically
bounded Normalized-AST/pattern carrier and use a member-specific scope/owner
rule. Users may call these objects but cannot define new privileged members;
the privilege does not imply text substitution, parser re-entry, or a general
macro system.

> **Distinction**: meta execution capability is not AST privilege. Names such
> as `match` and `struct` remain ordinary parser-level names; parser code does
> not special-case them.

_See also: Name, Strong context._

---

## Declaration

A user-visible binding introduced by `let`. In v0.1, all declarations enter
through `let`. There is no separate `fn`, `type`, or `namespace`
declaration syntax. Declarations use a binding slot whose annotation, when
present, is parsed and preserved but not semantically checked.

_See also: Let binding, BindingSlot, BindingAnnotation._

---

## Let binding

A top-level `let` form that introduces a binding slot. A let binding may bind
a simple binder name or a canonical skeleton pattern, may carry a per-slot
deduce list, may carry an optional binding annotation, may carry `with { ... }`,
and is followed by `=` and an initializer expression.
Let bindings are the only declaration path in v0.1.

At the semantic layer, ordinary `let LHS = RHS` has one uniform value-binding
rule for ordinary values, type values, and Pattern values:

```text
evaluate RHS -> v
allocate fresh LHS Symbol/Place
bind that exact v to the LHS Symbol
```

If RHS is a path, the path first resolves a Symbol and reads `v`; the RHS
carrier Symbol is not part of `v` after that read. `let T: type = uint8`
therefore binds the existing type value under a fresh carrier. No declaration
form forwards a Symbol or a place; to observe another object's place, bind a
borrow view (`uint8 ref`, `uint8 share`, `p@`).

_See also: Declaration, BindingSlot, BindingAnnotation._

---

## BindingSlot

A parser-level binding-site shape reused by let bindings, closure parameters,
and closure returns. It preserves an optional `PolicySpec`, optional `let`,
optional `DeduceList`, a binding pattern, optional binding annotation, optional
`with { ... }`, and an optional initializer where the surrounding context allows
one.

The optional policy is recognized only in the strong policy position before
`let`. It is either one policy expression or an explicit pair separated by
`:`. Without the trailing `let`, the same tokens stay in the binding pattern /
canonical skeleton. `None` means unwritten and later inferred. The parser
preserves syntax and does not perform semantic pair validation.

_See also: Let binding, BindingAnnotation, CanonicalSkeleton._

---

## Source-Visible Global Implementation Space

The toolchain-owned source construction input installed at namespace root
`::`, abbreviated `Gsrc`. Its files pass through ordinary lexing, parsing,
normalization, declaration harvesting, semantic Symbol/Val2 construction, and
ordinary invocation. The typed build authority may use an empty install
prefix; ordinary project source roots may not.

`Gsrc` is source-visible namespace material, not a prelude. A project lookup
still follows ordinary path and public/private rules, and no member is injected
into lexical scope merely because it is installed at `::`. Physical bundle
paths organize build input but do not determine Symbol or Pattern identity.

_See also: Namespace Symbol Views, Toolchain Global Construction Authority._

---

## Toolchain Global Construction Authority

The typed build fact authorizing a toolchain-owned source bundle to contribute
direct members to `::`. Global visibility and global construction authority
are different: ordinary source may resolve a public global path but cannot
obtain root-construction authority from an empty directory, empty mount
prefix, or missing navigation component.

_See also: Source-Visible Global Implementation Space._

---

## Namespace Symbol Views

Three independent sets govern namespace and build-world reasoning:

```text
Σ_full(N)    complete namespace-internal symbol and overload set
Σ_export(N)  externally exposed projection of Σ_full(N)
Wfinal       Wpre ∪ Wseal, materialized/retained/generated build world
```

Internal explicit resolution searches `Σ_full`; external explicit resolution
searches `Σ_export`; world membership asks whether a symbol exists in Wpre or
Wseal. The export overload set preserves the same candidate identities as the
full set, but every external candidate carries a separately const-projected
resolved `PolicyPair` rather than a declaration-side `P1Projection` or a clone
of its complete internal policy. External admission requires both
export-retention-closure membership and public reachability through the full
path.
Within each admitted full overload set, mut-only candidates remain internal
and candidates with const (or pure `Pp`) views enter the external set.
Publicly reachable export-retention-closure ancestors and descendants receive this
projection even when they are not export roots. World membership does not
imply export, and export does not imply that the symbol itself was an export
root. Retention-closure membership is graph/interface-construction input, not
synonymous with membership in `Σ_export`.

_See also: Policy Pair, Namespace (source name)._

---

## Policy Pair

The canonical internal policy representation:

```text
Π = Pv:Pp
```

`Pv` describes the `Val1`/value component; `Pp` describes its carried
Pattern/anonymous-type component. Stage, value mutability, value presence,
ordinary namespace visibility, and export-root are typed orthogonal dimensions. A scalar policy
is surface shorthand or a derived summary and cannot reconstruct the pair.
Ordinary policy notation does not use `@`, which remains reserved for lifetime
policy syntax.

At namespace direct top level, `export` derives an external view without
cropping the complete internal `Pv:Pp`. A value-bearing external view is
`Project_const(Pv):Pp` and therefore requires a non-empty const projection; a
`mut`-only value export is invalid. A pure `absent:Pp` export has no
value-mutability obligation. More strongly, absent Pv has no value stages and
no value-mutability domain at all; `const + S : compile` and
`mut + S : compile` are invalid before namespace export is considered. This
rule is checked by P1/P2 elaboration and resolved export projection rather than
being inferred from export alone.

_See also: PolicyBinding,
`spec/design/symbol-world/symbol-policy-and-compile-flow-projection.md`._

---

## Policy Binding

The future P1 projection judgment for a binding:

```text
[P1] let x = expr
```

Omitted P1 keeps the fully inferred result. A single `Q` selects values visible
under `Q` and retains each value's associated Pattern component. An explicit
`Qv:Qp` filters both components. Therefore single P1 `Q` is not pair `Q:Q`.
The selected slice must be non-empty and admitted by the destination binding.
Projection crops the policy slice while preserving symbol and Pattern identity;
it does not return an unchanged entry after a mere intersection check.

The bounded transition prototype does not change this rule. Any non-empty
projection completes binding elaboration; alternatives written in the query but
absent from the RHS are not obligations to manufacture values. More generally,
an existing compatible Policy view dominates migration: successful projection
preserves the existing Symbol, TypeValue, PatternValue, Place, and value
identity and makes migration semantically unreachable.

There is no general prohibition on runtime bindings:

```text
runtime let x = runtime_value
```

is legal when the runtime value slice exists. A `Psrc != runtime` premise may
belong to one compile-flow projection rule, but never to general let lowering.
In P2 context, unlike P1, a single policy is normalized into a result pair; in
particular current `runtime` means `runtime:compile`; explicit `runtime:seal`
remains valid.

_See also: BindingSlot, PolicyPair,
Policy Transition,
`spec/design/symbol-world/symbol-policy-and-compile-flow-projection.md`._

---

## Policy Demand Satisfaction

The act of satisfying a consumer's requested Policy view. Demand kind records
consumer origin; it does not grant permission to search arbitrary conversion
operations. The ordering is called **Existing-First,
Constructible-Second**:

```text
existing compatible view
  -> use Policy slicing

complete existing projection is empty
  + runtime is an accepted alternative
  + eligible static Val1 view
  -> extract RuntimeBranch(query)
  -> consider one authorized atomic Runtime Policy migration

otherwise
  -> inadmissible or governed by another explicit language mechanism
```

A consumer accepting `compile || runtime` is satisfied by an available compile
slice. Merely mentioning runtime in a choice does not force materialization.
However, if a complete choice such as `meta || runtime` has no existing
accepted view, runtime is the currently language-constructible accepted stage
branch. Failure of Type/Pattern structural applicability cannot be repaired by
Policy migration.

For an object already carrying `(compile || runtime):compile`, the runtime
branch is an existing Policy slice rather than a migration request.
`ExposePolicySlice(runtime)` may therefore succeed during a static phase while
`ReadValue(runtime)` remains unavailable. Compile-readable dependencies are
bound/evaluated statically; runtime-dependent computation is residualized and
continues the same already-resolved invocation without reopening Symbol lookup
or overload selection. Residual representation, effect sequencing, and
continuation ABI remain open.

_See also: Policy Binding, Policy Pair, Policy Transition._

---

## Policy Transition

The current canonical transition case is the language-authorized atomic
Runtime Policy migration considered only after a complete accepted Policy
choice has no existing view and that choice contains runtime. Define:

```text
S = Static(Pv) = Pv - runtime
```

For a legal selected input endpoint, `S` is non-empty and `S = Pp`. The
compiler-mandated endpoint skeleton is:

```text
input:  Type=T, value stage=S,       Pp=S
output: Type=T, value stage=runtime, Pp=S, presence=present
```

Input/output value mutability may differ because those coordinates belong to
the selected ordinary callable and its overload Policy. Thus
`const compile -> mut runtime` may construct a fresh runtime object; the
compiler authorizes the stage edge but does not invent `mut`. Pattern-side
Policy capability remains `S`. This does not mean the implementation copies
the source Pattern object. An eventual ordinary function-object invocation
supplies an ordinary result whose Type/Pattern/owner coherence is governed by
existing invocation and Pattern semantics.

Migration endpoint mutability uses ordinary actual-relative Bp preference, not
hard Policy-domain intersection or subset specificity:

```text
const actual/demand: const > unspecified > mut
mut actual/demand:   mut > unspecified > const
```

Opposite endpoint Patterns remain fully admissible. Stage, presence, Pp
capability, Type, and structural applicability remain hard constraints.
Generic ordinary members may realize the four default transports
`const <- const`, `const <- mut`, `mut <- const`, and `mut <- mut`; more
specific Pattern members may refine or delete regions of that relation.

The demand-preparation helper implements only the binding-P1 entry point. It projects
the complete original query first, then derives a runtime-only target branch:

```text
PolicyTransitionRequest {
  source_policy,
  target_query,
  source_type,
  source_value,
  provenance
}
```

The connected build slice consumes such a request through the source
`PatternValue`'s resolved owner and associated `()` Val2, then uses the same
`PreparedCallCandidate`, `InvocationFrame`, and ordinary result path as source
calls. Its Bp' dominance relation composes the implemented ordinary
formal/phase coordinates with optional input/output migration endpoints before
one maximal-element selection. Without those optional coordinates it reduces
to the connected ordinary order; a source regression preserves the older
restricted selector's winner identity.

The older caller-supplied candidate-ordering carrier remains algebra-only
fixture material. Its endpoint-only maxima helper is private and not
sequentially composable with ordinary Bp. Crossed advantages are ambiguous and
declaration order is irrelevant. Absent Val1 cannot construct the request.
Candidate output Type must equal source Type, so migration cannot search `ref`
or another structure-changing operation to repair applicability.

Input and output Policy slicing bracket the directed migration:

```text
Project_out o Migration o Project_in
```

No transitive migration graph, candidate backtracking, temporary-lifetime
extension, universal transition Symbol, or new callable ontology is implied.
Explicit mechanical `ref`, `share`, and `rebind` operations remain ordinary
function-object calls distinct from Policy-demand satisfaction.
Binding P1 is the currently connected demand consumer. Consumer-neutral
parameter/result demand preparation, complete Pattern/result construction,
backend/runtime materialization, and residual execution remain future work;
ordinary Symbol/Val2/associated-`()`/InvocationFrame routing itself is now
connected.

_See also: Policy Binding, Policy Pair,
`spec/contracts/v0.6-cross-policy-value-transition.md`._

---

## BindingAnnotation

The annotation following `:` in a `BindingSlot`. It preserves the written
annotation associated with a binding site. It has two explicit raw forms: a
single annotation expression, or a compound annotation with a preserved `:`
between the left annotation term and right annotation expression. v0.1 does
not determine whether the annotation denotes a value object, type object, rank
object, custom rank, concept, region, or future classifier. Parsed into
`BindingAnnotationAst::Expr` or `BindingAnnotationAst::Compound`.

> **Distinction**: `BindingAnnotation` is a parser-level construct, not a
> semantic type. v0.1 does not check that annotation names resolve to
> anything. A single-expression annotation is preserved exactly as written.

_See also: BindingSlot, AnnotationTerm, Type-object._

---

## AnnotationTerm

The left side of a compound `BindingAnnotation`, before the second `:`. It can
be a preserved expression or a hole (`_`). In `let f: fn = ...`, there is no
compound annotation; the whole annotation is `BindingAnnotationAst::Expr`.

_See also: BindingAnnotation, AnnotationHole, Type-object._

---

## AnnotationHole

The token `_` used as an annotation-term placeholder. Appears in forms like
`let f: _: fn = ...`, where the left annotation term is anonymous and the
right annotation expression is preserved. Represented as
`AnnotationTermAst::Hole`.

> **Distinction**: `AnnotationHole` is an annotation-term placeholder, distinct
> from a canonical skeleton wildcard `_`.

_See also: AnnotationTerm, CanonicalSkeleton, Type-object._

---

## Atomic builtin type

An actual builtin Type value whose identity does not require applying a
dependent type constructor to another Type value. The current T key space is:

```text
uint | int | float | buffer | str
```

The Rust `AtomicBuiltinType` enum is a lookup key for these intended Type
symbols, not itself a `TypeValueId` and not merely a literal classifier.
Current core bootstrap does not yet install every member.

_See also: Concrete numeric type, Type-object, TypeValueId._

---

## Concrete numeric type

A width-bearing numeric Type (`Tnum`) such as `uint16` or `float32`. Numeric
literal materialization selects a concrete numeric Type rather than using the
atomic `uint`/`int`/`float` Type as the literal's final type. In the current
implementation, `NumericTypeKey` maps to a first-order `TypeValueId` projection
derived from an installed core Type symbol; final canonical type-value equality
is not implemented.

_See also: Atomic builtin type, Literal, TypeValueId._

---

## Type-object

A type-theoretic object: the type of some value, or an object that itself
represents a type. In v0.1 declarations:

- In `let t: type = ...`, `type` is preserved as a bare annotation expression.
- In `let f: _: fn = ...`, `_` is an annotation hole. A later semantic pass may
  interpret it as an anonymous type-object whose kind/rank is given by the
  source name `fn`.

_See also: Kind/rank object, BindingAnnotation, AnnotationHole._

---

## Kind/rank object

An object that classifies type-objects. In source text, names such as `fn`
and `type` may appear in explicit rank annotation position:

- `let t: _: type = ...` - the source name `type` occupies the kind/rank
  annotation position for the anonymous type-object `_`.
- `let f: _: fn = ...` - the source name `fn` occupies the kind/rank
  annotation position for the anonymous type-object `_`.

v0.1 does not check kind/rank validity. The parser preserves binding annotation
structure only.

_See also: Type-object, BindingAnnotation, AnnotationTerm._

---

## Namespace (source name)

The source-level name `namespace` as written by a user in a program. In
v0.1, `namespace` is an ordinary `Name` token, not a keyword. Users may
write it in let declaration annotations (e.g., `let ns: namespace = ...`),
but the parser does not interpret it semantically.

> **Distinction**: The conceptual notion of "namespace" as a module/scope
> is distinct from the source name `namespace`.

_See also: Name, Declaration._

---

## `fn` source name

The source-level name `fn` as written by a user. In v0.1, `fn` is an
ordinary `Name` token, not a keyword. It may denote the kind/rank of
function type-objects when used in explicit rank annotation position
(e.g., `let f: _: fn = ...`). The parser does not interpret `fn` as
implying function object construction — that is a future semantic pass.

> **Distinction**: The conceptual "function object" that `fn` may denote
> in the language is a kind/rank classification for function type-objects,
> distinct from the source name `fn` itself.

_See also: Name, Declaration, Kind/rank object, Type-object._

---

## Raw AST

The AST produced directly by the parser, before any lowering or normalization.
Raw AST preserves surface syntax faithfully; it does not desugar or canonicalize
forms. The v0.1 Raw AST frontend is completed and is the input to future
normalization passes.

_See also: Normalized AST, Normalization, Raw AST contract._

---

## Normalized AST

A future desugared AST that unifies call/product forms (product, pipe, operator sugar),
extraction forms (canonical skeletons, deduce lists), and declaration forms
(simple let, extract let, alias let) into simple pattern / call / declaration
structures. Normalized AST is desugared but still non-semantic; it is not HIR,
not type-checked, and not name-resolved.

> **Distinction**: Normalized AST is a structural simplification of Raw AST.
> It does not resolve names, infer types, evaluate canonical forms, materialize
> closures, or insert drops. HIR is a later representation that assumes name
> resolution and type checking.

_See also: Raw AST, Desugaring, Normalization, HIR, Raw AST contract._

---

## Desugaring

Removing surface syntax sugar into simpler normalized forms. Examples:
operator sugar (prefix-negative `-x`, postfix `!`, binary `+`) lowered to named operator
calls; member/double-dot sugar lowered to lookup forms; product placement unified
into a single call structure; extraction skeletons desugared into pattern forms.

Desugaring does **not** perform name resolution, operator lookup, type checking,
overload resolution, canonical matching, or closure materialization.

_See also: Normalization, Normalized AST._

---

## Normalization

The non-semantic lowering pass from Raw AST to Normalized AST. Normalization
produces structurally simpler AST without resolving names, inferring types, or
evaluating semantics. It is the first desugaring pass after parsing.

_See also: Desugaring, Normalized AST, Raw AST, Non-semantic lowering._

---

## Surface-preserving

A property of Raw AST: syntactic sugar and surface forms (operator expressions,
member sugar, double-dot sugar, pipes, products, extraction skeletons) are
preserved as-is in the AST tree. No desugaring or canonicalization is performed
by the parser.

_See also: Raw AST, Desugaring._

---

## Non-semantic lowering

An AST-to-AST transformation that changes the tree shape (e.g., desugaring)
but does not resolve names, infer types, evaluate expressions, or perform
semantic analysis. Normalization is a non-semantic lowering pass.

_See also: Normalization, Desugaring, Raw AST, Normalized AST._

---

## HIR

High-level IR (or High IR) — a future intermediate representation that assumes
name resolution, type checking, and potentially other semantic analysis has been
completed. HIR is later than Normalized AST in the compilation pipeline.

> **Distinction**: Normalized AST is a desugared but still non-semantic
> representation. HIR assumes semantic analysis has already run. Do not call
> Normalized AST "HIR".

_See also: Normalized AST, Non-semantic lowering._

---

## Raw AST contract

The documented invariants of v0.1 Raw AST (`spec/contracts/raw-ast-contract-v0.1.md`)
that future normalization passes may rely on. Defines what each AST node
preserves and what normalization must not assume.

_See also: Raw AST, Normalization, Normalized AST._

---

## Pattern normalization

Desugaring extraction skeletons (canonical skeletons, deduce lists) into
normalized pattern forms. Pattern normalization is structural simplification
only; it does not execute universal extraction matching, resolve deduce holes,
or validate skeleton admissibility.

_See also: Normalization, CanonicalSkeleton, DeduceList._

---

## Call normalization

Desugaring product/pipe/operator-sugar structures into a unified normalized
call form. Call normalization flattens pipe segments, interprets product placement,
and lowers operator sugar to named operator calls. It does not perform
overload resolution or determine which declaration is being called.

_See also: Normalization, ProductForm, OperatorSugar, PipeExpr._

---

## Declaration normalization

Desugaring let/alias-let forms into normalized declaration forms. Declaration
normalization may preserve optional `with { ... }` clauses and unify
simple and extract let forms into a common structure. It does not resolve
aliases, check types, or decide declaration semantics.

_See also: Normalization, Let binding, Alias binding._

---

## Diagnostic

A structured error, warning, or note produced during lexing or parsing. Every
diagnostic must carry a span. The parser is error-tolerant: it produces
`ErrorAst` nodes alongside diagnostics and continues parsing.

_See also: ErrorAst, Span, diagnostics-v0.1.md._

---

## Golden test

A test that compares tool output (token dump, AST dump, or diagnostic dump)
against a checked-in expected file. Golden tests must be used for every syntax
rule. The dump format must be stable and hand-written, not Rust `Debug` output.

_See also: lexer_golden.rs, parser_golden.rs, diagnostics_golden.rs._

---

## ReturnEvent

A block terminal form representing a targeted return event. It is
not an expression. Raw AST: `FormAst::ReturnEvent(ReturnEventAst)`.
Norm AST: `NormForm::ReturnEvent(NormReturnEvent)`.

_See also: TailValue, ReturnTargetSyntax, Control-flow end event._

## TailValue

The last expression form in a body block, normalized as
`NormForm::TailValue(NormExpr)`. A block result / tail value,
not early return. For an extraction-style callable result it supplies one
object expected to match the complete declared return Pattern, as in
`let ResultPattern = expr`; it does not assign one value to every output
binder.

_See also: ReturnEvent, Control-flow end event._

## ReturnTargetSyntax

The unresolved target syntax of a return event:

```text
NormReturnTargetSyntax ::=
    ImplicitNearest
  | Explicit(NormExpr)
```

`ImplicitNearest` is a historical marker name preserved by the frozen
normalized surface; it carries no resolved target. Its confirmed semantic
interpretation is a return to the outermost enclosing function layer (while a
plain tail `expr;` delivers to the directly enclosing layer, and
`Explicit(T)` selects the layer named by the function-object type `T`). The
current restricted build pass binds it to an active `ReturnTargetFrame`; full
lexical self-capability resolution remains future.
`Explicit(NormExpr)` preserves the explicit target syntax
without resolution.

_See also: ReturnEvent._

## ImplicitNearest return target

A return target marker whose name is historical: in the parser and normalizer,
`ImplicitNearest` is an unresolved marker, and the confirmed semantics of the
source form `E return;` is a return to the outermost enclosing function layer,
not the nearest one. Implementations must not extend behavior based on the
older nearest-enclosing reading. A restricted post-normalization binder
resolves the active frame; result Pattern delivery remains deferred.

_See also: ReturnTargetSyntax, Explicit return target._

## Explicit return target

A return target where the explicit target syntax is preserved
in the AST. In the parser, `Explicit(ExprAst)`; in the
normalizer, `Explicit(NormExpr)`. Source forms are
`E |> (T return);` and `E (T return);`.

The explicit target syntax `T` is not resolved by parser or
normalizer. The restricted build binder supports active name targets through a
temporary spelling identity; full lexical self-capability resolution is
deferred.

_See also: ReturnTargetSyntax, ImplicitNearest return target._

## Control-flow end event

A structural category covering tail values and return events:

```text
Control-flow end event :=
    TailValue(E)
  | ReturnEvent(E, target)
```

Reported by the parser and normalizer as explicit control-flow
data. Not an expression category.

_See also: TailValue, ReturnEvent._

## Terminal block form

A form that ends a body block. Once a terminal form appears,
no later form may occur before `}`:

```text
Terminal block form :=
    TailValue(E)
  | ReturnEvent(E, target)
```

The parser emits `StatementAfterTerminalBlockForm` for forms
after a terminal.

_See also: Control-flow end event._
