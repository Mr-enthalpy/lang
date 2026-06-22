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
structurally. Examples: `let` at form start, `where`/`acquire` in closure heads,
`with` inside let bindings, `<>` in binding contexts.

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

_See also: DeduceList, Hole, ArgPack, CanonicalNameRole._

---

## ArgPack

A parenthesized comma-separated list of expressions: `(a, b, c)`. ArgPacks
participate in expression construction through segment-local role assignment.
They are not traditional function call arguments.

> **Distinction**: `f(args)` is not a traditional function call. The `(args)`
> is an `ArgPack` that receives a role (`SourcePack`, `InsertPack`, or
> `RightTargetSubsegment`) within a `Segment`.

_See also: ArgPackRole, Segment, PipeExpr._

---

## ArgPackRole

The syntactic function assigned to each `ArgPack` within a `Segment`:

- **SourcePack**: an `ArgPack` at segment index 0. Starts the
  segment-local source pack. This role is assigned positionally,
  before considering non-initial insert packs or incoming pipe state.
- **InsertPack**: the first `ArgPack` after position 0 in a segment that
  receives an incoming pipe; accepts the piped value.
- **RightTargetSubsegment**: any `ArgPack` after `InsertPack` has been used;
  starts a recursive subsegment.
- **Unknown**: error-recovery placeholder.

_See also: ArgPack, Segment._

---

## PipeExpr

A top-level expression formed by splitting tokens at `|>` into segments.
`PipeExpr` is the entry point for expression parsing.

```text
PipeExpr ::= Segment ("|>" Segment)*
```

_See also: Segment, ArgPackRole._

---

## Segment

One part of a `PipeExpr`, containing a sequence of `OperatorExpr` and `ArgPack`
elements in the operator-aware design. The current parser phase may still store
plain `Atom` elements until operator parsing is implemented. Each segment has
an `has_incoming` flag indicating whether a prior segment exists.

_See also: PipeExpr, Atom, ArgPack._

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
- `Error`

Atoms are constructed by parsing a base and then folding suffixes (`::`, `.`,
`..`, and postfix operators once implemented). Operator sugar itself is stored
at the `OperatorExpr` layer, not as a general `Atom` variant.

_See also: ClosureAST, ArgPack, OperatorSugar, PostfixOperator, SelectorAst, NavPath._

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

Operator identity is `spelling + fixity + arity`.

_See also: Fixity, Arity, NavComponent, OperatorSugar._

---

## Fixity

The syntactic position of an operator relative to its operands. The operator
design uses `Prefix`, `Postfix`, and `Binary` fixities. Fixity is part of
operator identity.

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

The prefix `-x` operator sugar form. It is not a negative literal; the lexer
produces `-` and the following literal or atom separately.

_See also: OperatorSugar, Fixity._

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
Without parentheses, `::` consumes only one immediate valid component.

_See also: NavPath, SelectorAst, OperatorName._

---

## EntityRef

A compile-time entity reference syntax. Phase 4.2 defines the design; Phase
4.4 implements a raw `EntityRef` parser inside alias-let RHS only. `EntityRef`
is not a runtime expression, not a `PipeExpr`, not an `ArgPack`, not a
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
EntityComponent ::= Name | OperatorName
EntityOuterComponent ::= Name
```

EntityRef navigation is inner-to-outer and preserves source-order components.
An operator name is allowed only as the innermost component unless a future
design explicitly allows operator-named scopes. The parser does not perform
operator lookup, name lookup, namespace resolution, or existence checking.

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
operator leaf of the target `EntityRef` must have the same operator identity
(`spelling + fixity + arity`). An operator alias cannot rename one operator
spelling into another. Operator alias validation is future static validation
or name-resolution work, not current parser behavior.

_See also: Alias binding, AliasBinder, OperatorName, EntityRef._

---

## Non-associative operator

An operator class that cannot be chained without explicit grouping in the
operator-aware parser design. Comparison, equality, and compound-looking
operators are non-associative in this phase, so `a < b < c`, `a == b == c`,
and `a += b += c` require grouping.

Semantic validity of grouped expressions remains outside parser scope.

_See also: OperatorSugar._

---

## ClosureAST

The AST representation of a closure literal before materialization into a
callable object. Two forms:

- **InlineClosureAst**: `FnHeadPrefix { ... }`
- **ExplicitClosureAst**: `FnHeadPrefix => { ... }`

> **Distinction**: `ClosureAST` is **not** `ClosureObject`. Closure literals
> produce AST first. A later semantic pass may materialize closure AST into
> callable objects.

> **Distinction**: Bare `{ ... }` in atom position does not produce a
> `ClosureAST`. There is no block-expression node in v0.1 AST.

_See also: InlineClosureAST, ExplicitClosureAST, ClosureObject, Materialization._

---

## InlineClosureAST

A headed closure literal without `=>`: `FnHeadPrefix BodyBlock`. Bare `{ ... }`
is not an inline closure.

_See also: ClosureAST, ExplicitClosureAST, FnHeadPrefix._

---

## ExplicitClosureAST

A closure literal with `=>`: `FnHeadPrefix => BodyBlock`. Minimal form: `() => {}`.

_See also: ClosureAST, InlineClosureAST, FnHeadPrefix._

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
declaration syntax. Declarations carry a `DeclAnnotation` that is parsed
and preserved but not semantically checked.

_See also: Let binding, DeclAnnotation._

---

## Let binding

A top-level `let` form that introduces a name. A simple let binding requires
a `DeclAnnotation` (`Name ":" DeclAnnotation`); an extract let binding uses
`DeduceList CanonicalSkeleton` instead. Both are followed by `=` and a value.
The grammar is `let LetBinder LetWithClause? "=" PipeExpr`.
Let bindings are the only declaration path in v0.1.

_See also: Declaration, LetBinder, DeclAnnotation._

---

## DeclAnnotation

The annotation following `:` in a `SimpleLetBinder`. It preserves the written
annotation associated with a declared name. It has two explicit forms: a bare
annotation expression, or a type-object annotation followed by a rank
annotation. v0.1 does not determine whether the declared object is a value,
type-object, namespace-like object, or function-like object. The grammar is
`DeclAnnotation ::= BareDeclAnnotation | TypeObjectAnnotation ":" RankAnnotation`.
Parsed into `DeclAnnotationAst::Bare` (single expression)
or `DeclAnnotationAst::TypeObjectWithRank` (type-object annotation + rank).

> **Distinction**: `DeclAnnotation` is a parser-level construct, not a
> semantic type. v0.1 does not check that annotation names resolve to
> anything. A bare declaration annotation is preserved exactly as written.

_See also: TypeObjectAnnotation, RankAnnotation, Type-object._

---

## TypeObjectAnnotation

The first part of an explicit rank `DeclAnnotation`, before the second `:`.
Can be a `PipeExpr` or a `TypeHole` (`_`). In `let f: fn = ...`, there is no
type-object annotation; the whole annotation is `Bare(Name("fn"))`.

_See also: DeclAnnotation, TypeHole, RankAnnotation, Type-object._

---

## TypeHole

The token `_` used as a type-object annotation placeholder. Appears in
forms like `let f: _: fn = ...`, where the type-object is anonymous and only
the rank is specified. Represented as `TypeObjectAnnotationAst::Hole`.

> **Distinction**: `TypeHole` is a type-object level placeholder, distinct
> from a canonical skeleton wildcard `_`.

_See also: TypeObjectAnnotation, CanonicalSkeleton, Type-object._

---

## RankAnnotation

The second part of a `DeclAnnotation` after the second `:`. Appears in
forms like `let f: _: fn = ...` where `fn` is the rank annotation. Stored
as an `ExprAst`. v0.1 does not check rank validity.

_See also: DeclAnnotation, TypeObjectAnnotation._

---

## Type-object

A type-theoretic object: the type of some value, or an object that itself
represents a type. In v0.1 declarations:

- In `let t: type = ...`, `type` is preserved as a bare annotation expression.
- In `let f: _: fn = ...`, `_` is an anonymous type-object (a `TypeHole`)
  whose kind/rank is given by the source name `fn`.

_See also: Kind/rank object, TypeObjectAnnotation, TypeHole._

---

## Kind/rank object

An object that classifies type-objects. In source text, names such as `fn`
and `type` may appear in explicit rank annotation position:

- `let t: _: type = ...` - the source name `type` occupies the kind/rank
  annotation position for the anonymous type-object `_`.
- `let f: _: fn = ...` - the source name `fn` occupies the kind/rank
  annotation position for the anonymous type-object `_`.

v0.1 does not check kind/rank validity. The parser preserves the annotation
structure only.

_See also: Type-object, RankAnnotation, DeclAnnotation._

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

A future desugared AST that unifies calls (ArgPack, pipe, operator sugar),
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
operator sugar (prefix `-`, postfix `!`, binary `+`) lowered to named operator
calls; member/double-dot sugar lowered to lookup forms; ArgPack roles unified
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
member sugar, double-dot sugar, pipes, argpacks, extraction skeletons) are
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

Desugaring ArgPack/pipe/operator-sugar structures into a unified normalized
call form. Call normalization flattens pipe segments, resolves ArgPack roles,
and lowers operator sugar to named operator calls. It does not perform
overload resolution or determine which declaration is being called.

_See also: Normalization, ArgPack, ArgPackRole, OperatorSugar, PipeExpr._

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
