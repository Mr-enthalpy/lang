# Glossary

Definitions are specific to this repository's v0.1 usage. Terms may have different
meanings in general PL theory.

---

## Token

The output of the lexer. A token is the smallest lexical unit: a `Name`, `Literal`,
`Symbol`, `Trivia`, `Invalid`, or `Eof`. Tokens carry a span and are consumed by
the parser. The lexer does not assign semantic roles to tokens.

*See also: Trivia, Name, Literal, Symbol, Span.*

---

## Trivia

A token class representing whitespace, comments, or other non-semantic text.
Trivia tokens are skipped by the parser but their spans must remain available
for diagnostic positioning. The lexer must preserve trivia spans; the parser may
discard trivia after consumption.

*See also: Token.*

---

## Name

A token class representing an identifier. Names include what traditional languages
call keywords. In v0.1, `return`, `else`, `match`, `drop`, `move`, `sync`,
`effect`, `fn`, `type`, `meta`, `runtime`, `compile`, `namespace`, `mod`, and
`struct` are all ordinary `Name` tokens at the lexical level.

> **Distinction**: A `Name` token is not a keyword. Semantic strength does not
> imply lexical keyword status.

*See also: Token, Strong context.*

---

## Strong context

A parser state in which certain `Name` tokens or symbols are interpreted
structurally. Examples: `let` at form start, `where`/`acquire` in closure heads,
`guard`/`with` inside let bindings, `<>` in binding contexts.

Outside a strong context, these tokens retain their ordinary `Name` or `Symbol`
identity.

*See also: Name, Hole, DeduceList.*

---

## DeduceList

A sequence of hole declarations enclosed in `<...>`, recognized only in strong
binding contexts such as extract-let binders, closure heads, parameter binders,
and return binders. Outside these contexts, `<` and `>` are ordinary symbols.

*See also: Hole, Strong context, CanonicalSkeleton.*

---

## Hole

A name declared in a `DeduceList` that acts as a wildcard standing for an
unknown type or value in following syntax. Holes appear inside a
`CanonicalSkeleton` with the `CanonicalNameRole::Hole` annotation.

*See also: DeduceList, CanonicalSkeleton.*

---

## CanonicalSkeleton

A syntactic pattern used in extraction contexts (extract-let binder, extract
parameter, extract return). The skeleton is a sequence of `CanonicalElement`
items representing the shape of a value to be matched against. In v0.1, the
parser builds canonical skeleton AST but does not execute matching.

*See also: DeduceList, Hole, ArgPack, CanonicalNameRole.*

---

## ArgPack

A parenthesized comma-separated list of expressions: `(a, b, c)`. ArgPacks
participate in expression construction through segment-local role assignment.
They are not traditional function call arguments.

> **Distinction**: `f(args)` is not a traditional function call. The `(args)`
> is an `ArgPack` that receives a role (`SourcePack`, `InsertPack`, or
> `RightTargetSubsegment`) within a `Segment`.

*See also: ArgPackRole, Segment, PipeExpr.*

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

*See also: ArgPack, Segment.*

---

## PipeExpr

A top-level expression formed by splitting tokens at `|>` into segments.
`PipeExpr` is the entry point for expression parsing.

```text
PipeExpr ::= Segment ("|>" Segment)*
```

*See also: Segment, ArgPackRole.*

---

## Segment

One part of a `PipeExpr`, containing a sequence of `Atom` and `ArgPack`
elements. Each segment has an `has_incoming` flag indicating whether a prior
segment exists.

*See also: PipeExpr, Atom, ArgPack.*

---

## Atom

The smallest self-contained expression unit. Atoms include:

- `Name("x")`
- `Literal`
- `Group(PipeExpr)`
- `Closure(ClosureAst)`
- `Path(base, names)`
- `MemberSugar(object, field)`
- `DoubleDotSugar(object, method, args)`
- `Error`

Atoms are constructed by parsing a base and then folding suffixes (`::`, `.`,
`..`).

*See also: ClosureAST, ArgPack.*

---

## ClosureAST

The AST representation of a closure literal before materialization into a
callable object. Two forms:

- **InlineClosureAst**: `{ ... }` or `FnHeadPrefix { ... }`
- **ExplicitClosureAst**: `FnHeadPrefix => { ... }`

> **Distinction**: `ClosureAST` is **not** `ClosureObject`. Closure literals
> produce AST first. A later semantic pass may materialize closure AST into
> callable objects.

> **Distinction**: `{ ... }` in atom position always produces a `ClosureAST`,
> not a normal block expression. There is no block-expression node in v0.1 AST.

*See also: InlineClosureAST, ExplicitClosureAST, ClosureObject, Materialization.*

---

## InlineClosureAST

A closure literal without `=>`. Minimal form: `{}`. May be prefixed with a
`FnHeadPrefix`. Parsed when the parser sees `{` in atom position.

*See also: ClosureAST, ExplicitClosureAST, FnHeadPrefix.*

---

## ExplicitClosureAST

A closure literal with `=>`: `FnHeadPrefix => BodyBlock`. Minimal form: `() => {}`.

*See also: ClosureAST, InlineClosureAST, FnHeadPrefix.*

---

## ClosureObject

A materialized, callable object produced from a `ClosureAST` by a future
semantic pass. In v0.1, closure objects do not exist. The parser produces
only closure AST.

> **Distinction**: `ClosureObject` is a semantic concept, not a parser concept.
> Materialization is explicitly out of scope for v0.1.

*See also: ClosureAST, Materialization.*

---

## Materialization

The future semantic pass that converts `ClosureAST` into a `ClosureObject`.
Materialization involves capture analysis, environment layout, and callable
object construction. This is not implemented in v0.1.

*See also: ClosureAST, ClosureObject.*

---

## Meta-function

A compiler-provided function that operates on AST or normalized syntax forms
rather than on runtime values. Examples (future): `match`, `effect`, `sync`.

> **Distinction**: `match` is a name at the parser level, not syntax. A future
> meta-function named `match` may consume closure AST arms, but parser code
> must not special-case the name `match`.

*See also: Name, Strong context.*

---

## Declaration

A user-visible binding introduced by `let`. In v0.1, all declarations enter
through `let`. There is no separate `fn`, `type`, or `namespace`
declaration syntax. Declarations carry a `DeclAnnotation` that is parsed
and preserved but not semantically checked.

*See also: Let binding, DeclAnnotation.*

---

## Let binding

A top-level `let` form that introduces a name. A simple let binding requires
a `DeclAnnotation` (`Name ":" DeclAnnotation`); an extract let binding uses
`DeduceList CanonicalSkeleton` instead. Both are followed by `=` and a value.
The grammar is `let LetAttr* LetBinder LetWithClause? "=" PipeExpr`.
Let bindings are the only declaration path in v0.1.

*See also: Declaration, LetBinder, DeclAnnotation.*

---

## DeclAnnotation

The annotation following `:` in a `SimpleLetBinder`. It preserves the written
annotation associated with a declared name. It may contain a type-object
annotation and, optionally, a rank annotation. v0.1 does not determine whether
the declared object is a value, type-object, namespace-like object, or
function-like object. The grammar is
`TypeObjectAnnotation [ ":" RankAnnotation ]`.
Parsed into `DeclAnnotationAst::Bare` (single expression)
or `DeclAnnotationAst::TypeObjectWithRank` (type-object annotation + rank).

> **Distinction**: `DeclAnnotation` is a parser-level construct, not a
> semantic type. v0.1 does not check that annotation names resolve to
> anything.

*See also: TypeObjectAnnotation, RankAnnotation, Type-object.*

---

## TypeObjectAnnotation

The first part of a `DeclAnnotation` before an optional `:` rank annotation.
Can be a `PipeExpr` or a `TypeHole` (`_`). In the sugar form `let f: fn = ...`,
there is no separate type-object annotation — the whole annotation is
`Bare`.

*See also: DeclAnnotation, TypeHole, RankAnnotation, Type-object.*

---

## TypeHole

The token `_` used as a type-object annotation placeholder. Appears in
forms like `let f: _: fn = ...`, where the type-object is anonymous and only
the rank is specified. Represented as `TypeObjectAnnotationAst::Hole`.

> **Distinction**: `TypeHole` is a type-object level placeholder, distinct
> from a canonical skeleton wildcard `_`.

*See also: TypeObjectAnnotation, CanonicalSkeleton, Type-object.*

---

## RankAnnotation

The second part of a `DeclAnnotation` after the second `:`. Appears in
forms like `let f: _: fn = ...` where `fn` is the rank annotation. Stored
as an `ExprAst`. v0.1 does not check rank validity.

*See also: DeclAnnotation, TypeObjectAnnotation.*

---

## DeclAnnotationSugar

A parser flag or variant indicating that the declaration annotation was
written in a surface-sugar form. For example, `let f: fn = ...` writes the
annotation as bare `fn`, which the parser preserves as
`Bare(Name("fn"))` without desugaring to `_: fn`. The
sugar status is tracked by the `DeclAnnotationAst` variant
(`Bare` vs `TypeObjectWithRank`).

*See also: DeclAnnotation.*

---

## Type-object

A type-theoretic object: the type of some value, or an object that itself
represents a type. In v0.1 declarations:

- In `let t: type = ...`, the declared object `t` is a type-object.
- In `let f: _: fn = ...`, `_` is an anonymous type-object (a `TypeHole`)
  whose kind/rank is given by the source name `fn`.

*See also: Kind/rank object, TypeObjectAnnotation, TypeHole.*

---

## Kind/rank object

An object that classifies type-objects. In source text, names such as `fn`
and `type` may appear in kind/rank annotation position:

- `let t: type = ...` — the source name `type` occupies the kind/rank
  annotation position for the type-object `t`.
- `let f: _: fn = ...` — the source name `fn` occupies the kind/rank
  annotation position for the anonymous type-object `_`.

v0.1 does not check kind/rank validity. The parser preserves the annotation
structure only.

*See also: Type-object, RankAnnotation, DeclAnnotation.*

---

## Namespace (source name)

The source-level name `namespace` as written by a user in a program. In
v0.1, `namespace` is an ordinary `Name` token, not a keyword. Users may
write it in let declaration annotations (e.g., `let ns: namespace = ...`),
but the parser does not interpret it semantically.

> **Distinction**: The conceptual notion of "namespace" as a module/scope
> is distinct from the source name `namespace`.

*See also: Name, Declaration.*

---

## `fn` source name

The source-level name `fn` as written by a user. In v0.1, `fn` is an
ordinary `Name` token, not a keyword. It may denote the kind/rank of
function type-objects when used in declaration annotation position
(e.g., `let f: _: fn = ...`). The parser does not interpret `fn` as
implying function object construction — that is a future semantic pass.

> **Distinction**: The conceptual "function object" that `fn` may denote
> in the language is a kind/rank classification for function type-objects,
> distinct from the source name `fn` itself.

*See also: Name, Declaration, Kind/rank object, Type-object.*

---

## Raw AST

The AST produced directly by the parser, before any lowering or normalization.
Raw AST preserves surface syntax faithfully; it does not desugar or canonicalize
forms. In v0.1, only raw AST exists.

*See also: AST (as defined in ast-construction-v0.1.md).*

---

## Diagnostic

A structured error, warning, or note produced during lexing or parsing. Every
diagnostic must carry a span. The parser is error-tolerant: it produces
`ErrorAst` nodes alongside diagnostics and continues parsing.

*See also: ErrorAst, Span, diagnostics-v0.1.md.*

---

## Golden test

A test that compares tool output (token dump, AST dump, or diagnostic dump)
against a checked-in expected file. Golden tests must be used for every syntax
rule. The dump format must be stable and hand-written, not Rust `Debug` output.

*See also: lexer_golden.rs, parser_golden.rs, diagnostics_golden.rs.*
