# Glossary

Definitions are specific to this repository's v0.1 usage. Terms may have different
meanings in general PL theory.

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

_See also: Token._

---

## Name

A token class representing an identifier. Names include what traditional languages
call keywords. In v0.1, `return`, `else`, `match`, `drop`, `move`, `sync`,
`effect`, `fn`, `type`, `meta`, `runtime`, `compile`, `namespace`, and
`struct` are all ordinary `Name` tokens at the lexical level.

> **Distinction**: A `Name` token is not a keyword. Semantic strength does not
> imply lexical keyword status.

_See also: Token, Strong context._

---

## Strong context

A parser state in which certain `Name` tokens or symbols are interpreted
structurally. Examples: `let` at form start, the active head clauses
`require`/`pre`/`post`/`lifetime pre`/`lifetime post` and the reserved-inactive
`where` position in closure heads, `with` inside let bindings, `<>` in binding
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
- `MemberSugar(object, selector)` (selector is `SelectorAst`)
- `DoubleDotSugar(object, selector, args)` (selector is `SelectorAst`)
- `BracketCallSugar(object, operator, args)` (`obj[args...]`; operator spelling `[]`, `args` is a `ProductExprAst`)
- `Error`

Atoms are constructed by parsing a base and then folding suffixes (`::`, `.`,
`..`, `[...]` bracket call, and postfix operators). Operator sugar itself is
stored at the `OperatorExpr` layer, not as a general `Atom` variant.

`BracketCallSugar` is source-preserving sugar for the operator spelling `[]`; it
is not indexing/slicing/container access. The `[]` operator is a contextual
paired operator name, bindable/aliasable/referable in operator-name positions.

_See also: ClosureAST, ProductForm, OperatorSugar, PostfixOperator, SelectorAst, NavPath._

---

## SelectorAst

A name-like construct appearing in suffix position after `.` or `..`.
In the current parser phase:

```text
SelectorAst ::=
    Text(NameAst)     // from TokenKind::Name
  | Numeric(NumericNameAst)  // from TokenKind::IntLiteral
```

A numeric token (`IntLiteral`) in selector position becomes `NumericNameAst`,
while the same token class in atom-base position becomes a numeric literal atom.
This distinction is mandatory.

_See also: NumericNameAst, NameAst, NavComponent, MemberSugar, DoubleDotSugar._

---

## NumericNameAst

A numeric selector (`1`, `42`, etc.) appearing after `.`, `..`, or `::`.
Carries `text: String` and `span: Span`. Distinct from `NameAst` (textual
names) and from numeric literal atoms (`IntLiteral`).

_See also: SelectorAst, NameAst._

---

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
NavComponent ::= Name | NumericName | OperatorName | GroupedExpr | Error
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

`EntityRef` may appear only in future explicit strong contexts, such as the
right-hand side of `let binder === EntityRef`. Current v0.1 parser behavior is
unchanged.

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
EntityComponent ::= Name | NumericName | OperatorName
EntityOuterComponent ::= Name | NumericName | Group
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
change namespace state globally. Lexical aliases are future design only and
are not resolved by the parser.

_See also: Alias binding, Entity alias._

---

## Entity alias

A lexical alias whose target is a compile-time entity reference (`EntityRef`).
The alias binds a name or operator to a compile-time entity path without
evaluating or constructing a runtime value. Entity aliases are a future
name-resolution construct, not a v0.1 parser feature.

_See also: Alias binding, Lexical alias, EntityRef._

---

## AliasBinder

The binder position in a future `let binder === EntityRef` form. It may be a
`Name` or `OperatorName`. This is a future parser concept: the binder is
preserved as raw AST syntax without resolving the target entity.

_See also: Alias binding, Operator alias._

---

## Operator alias

A future alias binding whose binder is an `OperatorName`. Operator aliases
are stricter than ordinary name aliases: the operator binder and the final
operator leaf of the target `EntityRef` must have the same overloadable
operator identity (`spelling + fixity + arity`, where fixity is `Binary` or
`Postfix`). Prefix negative is not an overloadable operator identity and
cannot appear as an alias binder or target. An operator alias cannot rename
one operator
spelling into another. Operator alias validation is future static validation
or name-resolution work, not current parser behavior.

_See also: Alias binding, AliasBinder, OperatorName, EntityRef._

---

## Non-associative operator

An operator class that cannot be chained without explicit grouping in the
operator-aware parser design. Comparison, equality, and equals-suffixed
operators are non-associative in this phase, so `a < b < c`, `a == b == c`,
and `a += b += c` require grouping.

Semantic validity of grouped expressions remains outside parser scope.

_See also: OperatorSugar._

---

## ClosureAST

The AST representation of a closure literal before materialization into a
callable object. Two forms:

- **InPlaceClosureAst**: Bare `{ ... }` in atom position. An in-place control-flow
  closure with no capture clause, parameters, or head clauses.
- **ExplicitClosureAst**: `FnHeadPrefix => { ... }`. A headed closure that
  requires `=>` between the head and body.

> **Distinction**: `ClosureAST` is **not** `ClosureObject`. Closure literals
> produce AST first. A later semantic pass may materialize closure AST into
> callable objects.

> **Distinction**: Bare `{ ... }` in atom position is an `InPlaceClosureAst`,
> not a normal block expression.

_See also: InPlaceClosureAST, ExplicitClosureAST, ClosureObject, Materialization._

---

## InPlaceClosureAST

A bare `{ ... }` in atom position that produces an in-place closure. It has no
capture clause, no parameter clause, no return clause, and no head clauses. It
is the Raw AST representation of a control-flow-embedding closure block.

_See also: ClosureAST, ExplicitClosureAST._

---

## ExplicitClosureAST

A closure literal with an explicit head and `=>`: `FnHeadPrefix => BodyBlock`.
The head may contain deduce list, capture clause, parameter clause, trait clause,
return clause, and head clauses. The body is a form block. Headed closures
without `=>` (e.g., `[](){}` or `(x){x}`) are rejected.

_See also: ClosureAST, InPlaceClosureAST, FnHeadPrefix._

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

A compiler-provided function that operates on AST or normalized syntax forms
rather than on runtime values. Examples (future): `match`, `struct`, `effect`,
`sync`. Some future built-in meta-functions may consume raw AST directly;
this is a built-in privilege, not unrestricted user macro power.

> **Distinction**: `match` is a name at the parser level, not syntax. A future
> meta-function named `match` may consume closure AST arms, but parser code
> must not special-case the name `match`. `struct` may be such a future
> built-in meta-function.

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
and closure returns. It preserves an optional policy expression, optional `let`,
optional `DeduceList`, a binding pattern, optional binding annotation, optional
`with { ... }`, and an optional initializer where the surrounding context allows
one.

The optional **policy** is recognized only by the contextual shape `Expr let`:
an expression written immediately before `let`. Without the trailing `let`, the
same tokens stay in the binding pattern / canonical skeleton. A `policy` of
`None` means the policy was unwritten (implicit / inferred later), not that the
binding has no policy. The parser preserves the expression shape only and
performs no policy validation.

_See also: Let binding, BindingAnnotation, CanonicalSkeleton._

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

The documented invariants of v0.1 Raw AST (`spec/raw-ast-contract-v0.1.md`)
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
