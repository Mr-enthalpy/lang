# Glossary

Definitions are specific to this repository's v0.1/v0.2 usage. Terms may have different
meanings in general PL theory.

---

## v0.1.w Raw AST Stability Window

The maintenance and contract-stabilization window after the completed v0.1 Raw
AST Frontend. During this window richer literal spelling and the pipe
branch-name shorthand were implemented as the final v0.1.w additions. v0.1.w
is now closed; the project has entered v0.2.

_See also: Raw AST, v0.2 Raw AST Contract Freeze._

---

## v0.2 Raw AST Contract Freeze

The current active stage after v0.1.w closure. The Raw AST frontend input
surface is frozen by default. Work in this stage is documentation
reconciliation, consistency repair, contract freeze checklist, and preparation
of the exact boundary that v0.3 Normalized AST Specification will consume.
`v0.2` is not a parser-expansion phase and does not implement Normalized AST.

_See also: v0.1.w, Raw AST, Normalized AST, raw-ast-contract-freeze-v0.2.md._

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

_See also: Hole, Strong context, CanonicalSkeleton._

---

## Hole

A name declared in a `DeduceList` that acts as a wildcard standing for an
unknown type or value in following syntax. Holes appear inside a
`CanonicalSkeleton` with the `CanonicalNameRole::Hole` annotation.

_See also: DeduceList, CanonicalSkeleton._

---

## CanonicalSkeleton

A syntactic pattern used in extraction contexts (extract-let binder, extract
parameter, extract return). The skeleton is a sequence of `CanonicalElement`
items. In v0.1, the parser builds canonical skeleton AST but does not execute
matching.

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
discards. One pack counts as one node regardless of captured length.
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
and ordinary-require check first form fully admissible set `A`. Const/mut then
uses product partial order across all constrained positions; no total score or
lexicographic fallback resolves incomparable candidates. Remaining
side-effect-free preference filters apply in one fixed normative order:
entry, concept, extraction, first-order-over-instantiated,
in-place-over-non-in-place, then named strategy rules. Each
filter is independent of candidate enumeration order; filters are not assumed
to commute. A named strategy only sees fully admissible candidates and cannot
restart lookup. Delete members participate normally, and ordinary uniqueness is
constrained by `must_select_if_qualified` strategies activated from `A`.

Lifetime policy is not a type/compile candidate filter. This revision defines
no lifetime overload, refinement order, ABI class, or second selection. Any
future lifetime check receives the already unique ordinary overload result,
under the boundary in
`spec/design/lifetime/lifetime-policy-and-overload-boundary.md`.

Full overload resolution is deferred to v0.10+ and depends on the pattern-space
and extraction-chain infrastructure. The formal specification is in
`spec/design/patterns-overload/overload-resolution-design.md` §5.

_See also: OverloadCandidate, OverloadSpecificity, Concept,
`spec/design/symbol-world/symbol-policy-and-compile-flow-projection.md`._

---

## Lifetime Policy Boundary

A negative boundary only: `@` belongs to lifetime syntax, lifetime policy is
not ordinary stage policy, and it cannot reopen or change the already unique
ordinary overload result. No lifetime algorithm, ordering, overload, ABI
equivalence class, refinement phase, or handoff object is defined.

_See also: `spec/design/lifetime/lifetime-policy-and-overload-boundary.md`._

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

A declaration form `let binder === EntityRef` that creates a compile-time
lookup alias in the current lexical scope. Phase 4.4 implements raw parser
preservation: the parser produces `LetAliasAst` with `AliasBinderAst` and
`EntityRefAst`. Alias binding is not runtime value binding, not an expression,
not equality, not operator syntax, and not package import syntax. No target
resolution, operator identity validation, or entity lookup is performed.

> **Distinction**: Alias binding is implemented as raw parser preservation
> only. It is not an ordinary `let name: annotation = expr`. It has no `=`
> value expression, no declaration annotation, no `guard`, and no `with`.
> EntityRef parsing is implemented only inside alias-let RHS.

_See also: Lexical alias, Entity alias, AliasBinder, Operator alias, EntityRef._

---

## Lexical alias

A compile-time lookup name introduced by alias binding into a lexical
scope. A lexical alias shadows previous bindings of the same name in the
current scope and nested scopes but does not mutate the original entity or
change namespace state globally. Lexical aliases are parser-preserved in Raw
AST, but their scope and target semantics are future work and are not resolved
by the parser.

_See also: Alias binding, Entity alias._

---

## Entity alias

A lexical alias whose target is a compile-time entity reference (`EntityRef`).
The alias binds a name or operator to a compile-time entity path without
evaluating or constructing a runtime value. Entity aliases are preserved by the
v0.1 parser, but target resolution is a future name-resolution construct.

_See also: Alias binding, Lexical alias, EntityRef._

---

## AliasBinder

The binder position in a `let binder === EntityRef` form. It may be a
`Name` or `OperatorName`. The parser preserves the binder as raw AST syntax
without resolving the target entity.

_See also: Alias binding, Operator alias._

---

## Operator alias

An alias binding whose binder is an `OperatorName`. Operator aliases are
parser-preserved as Raw AST. Later validation may require the operator binder
and the innermost operator component of the target `EntityRef` to have the same
overloadable operator identity (`spelling + fixity + arity`, where fixity is
`Binary` or `Postfix`). Prefix negative is not an overloadable operator
identity and cannot appear as an alias binder or target. An operator alias
cannot rename one operator spelling into another. Operator alias validation is
future static validation or name-resolution work, not current parser behavior.

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
allowed, and this read rule grants no implicit outer-write authority. An
otherwise tied in-place candidate is preferred after the
first-order-over-instantiated filter.

> **Explicit self position for return:** A headless in-place closure
> has no self target and cannot express early return. Early return
> examples that target a specific closure should use an in-place
> closure with an explicit product/extraction head carrying the
> self position, e.g.:
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
Plain no-`=>` block tails and `[[name]]` stay in-place; the latter is only the
named-strategy escape that does not steal the established return
extraction-pattern parse.

_See also: ClosureAST, InPlaceClosureAST, FnHeadPrefix._

---

## Callable Implementation Tail

The single syntax slot that describes a callable implementation and optional
overload strategy. It normalizes to `UserBody(Ordinary|Named, body)`,
`Defaulted`, or `Deleted(message?)`. `=> name {}` and `[[name]] {}` carry the
same named strategy. Strategy metadata participates only after full
admissibility and never creates a second overload pass.

_See also: ExplicitClosureAST, Fully Admissible Candidate, Overload Resolution Pipeline._

---

## Dot Closure

The first-class expression `.name`, normalized to a generated function object
`(val: T, ...args) { (val, args) |> name::T }`. `E.name` is compact
`E |> .name`; `.name` itself captures no receiver. After lowering it is an
ordinary expression. Replacing it with a bound equivalent must preserve the
same pipe/product binding spine, and no normalizer rule may inspect
`DotClosureLowering` provenance to absorb surrounding syntax. `..name` remains
direct member-call sugar.

_See also: Atom, Function Object, Call normalization._

---

## Pack Pattern

The Pattern-side remainder form `...Q`. It matches the unmatched portion of
one normalized structural level and then applies `Q`. Each level permits one
pack; nested levels are independent. It is not a value/type/ABI category and
has no RHS unpack counterpart. Its specificity contribution is one node,
independent of captured length. It is valid in every let-shaped binding slot,
including ordinary/local let, parameter, return, and nested product extraction;
it is not a parameter-only variadic syntax.

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
Materialization involves capture analysis, environment layout, and callable
object construction. This is not implemented in v0.1.

_See also: ClosureAST, ClosureObject._

---

## Meta-function

A callable whose entry executes with `P2 = meta` and constructs
`SymbolConstructionValue` under symbol-world construction capability. An
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
`spec/design/symbol-world/symbol-policy-and-compile-flow-projection.md`._

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

`ImplicitNearest` represents a return targeted at the nearest
enclosing self. The current restricted build pass binds it to an active
`ReturnTargetFrame`; full lexical self-capability resolution remains future.
`Explicit(NormExpr)` preserves the explicit target syntax
without resolution.

_See also: ReturnEvent._

## ImplicitNearest return target

A return target indicating the return should target the nearest
enclosing function-object self. In the parser and normalizer,
`ImplicitNearest` is an unresolved marker. The source form is
`E return;`. A restricted post-normalization binder resolves the active frame;
result Pattern delivery remains deferred.

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
