# File: spec/ast-construction-v0.1.md

# AST Construction Rules v0.1

## 0. Scope

This document defines how source tokens are converted into AST in `v0.1`.

It defines syntax recognition and AST construction only.

It does not define:

- type checking
- kind checking
- overload resolution
- canonical matching
- closure object materialization
- match semantics
- effect semantics
- NLL/lifetime analysis
- drop insertion
- code generation

## 1. Notation

This document uses the following notation:

```text
A ::= B C
```

means syntactic production.

```text
tokens ⟶ AST
```

means AST construction.

```text
X ⇝ Y
```

means syntax node `X` is represented as AST node `Y`.

This document distinguishes:

- lexical token
- parser context
- AST node
- future semantic interpretation

Only AST construction is normative here.

## 2. Token classes

The parser consumes token classes produced by the lexer.

```text
Token ::=
    Name(text)
  | IntLiteral(text)
  | StringLiteral(text)
  | Symbol(sym)
  | Trivia
  | Invalid
  | Eof
```

Symbols include at least:

```text
( ) [ ] { } , : = . .. :: |> => -> < > ;
```

Operator-aware tokenization preserves these additional operator spellings as
syntax-level operator names:

```text
+  -  *  /
<  <=  >=  >  ==  !=
<<  >>
!  &  @  ~  ^  $  ++  --  ?
+=  -=  *=  /=  <<=  >>=
```

Operator-aware tokenization uses maximal munch: when multiple operator
spellings can start at the same source position, choose the longest spelling.
For example, `<<=`, `<=`, `++`, and `==` are each single operator spellings,
not shorter operator spellings followed by another symbol.

Trivia is skipped by the parser, but spans must remain available.

## 3. Program and forms

### 3.1 Program

```text
Program ::= Form*
```

AST:

```text
Program {
    forms: Vec<FormAst>
}
```

### 3.2 Form

```text
Form ::= LetStmt | ExprStmt
```

Form selection rule:

If the first non-trivia token of a form is `Name("let")`, parse `LetStmt`.

Otherwise parse `ExprStmt`.

AST:

```text
FormAst ::=
    Let(LetAst)
  | Expr(ExprAst)
  | Error(ErrorAst)
```

### 3.3 Form boundary

A form ends at any of:

- `;`
- `}`
- EOF

A line break (`\n`) is lexical trivia; it is **not** itself a form boundary.

At form-level decision points, a top-level line break **may be promoted** to a
soft form separator (FormSep) only if all of the following hold:

1. Current nesting depth of `()`, `[]`, `{}` is zero.
2. The previous significant token can end a form
   (`Name`, `IntLiteral`, `StringLiteral`, `)`, `]`, `}`).
3. The next significant token can start a form
   (`Name`, `IntLiteral`, `StringLiteral`, `(`, `{`).
4. Neither side is a continuation token.
5. The parser is not inside a syntactic frame that requires more tokens
   (e.g., the right side of a `|>` pipe expression).

Continuation tokens include at least:
`|>`, `=>`, `->`, `.`, `..`, `::`, `,`, `=`, `:`, `<`, `>`, and
`Operator` tokens (the lexer's operator spellings).

Examples:

```text
a
b
```
→ two forms (Name can end form; Name can start form).

```text
a |>
b
```
→ one `PipeExpr` (`|>` is a continuation token; newline is not promoted).

```text
a +
b
```
→ future: one `OperatorExpr`, not two forms (`+` is a continuation token;
newline is not promoted).

```text
obj.
field
```
→ one `MemberSugar` (`.` is a continuation token in the suffix loop;
newline before a suffix-start token is not promoted).

```text
obj::
field
```
→ one `Path` (same reasoning as above).

This is a provisional v0.1 rule. The broader language-design question of
whether form boundaries should remain line-based or become fully explicit
remains open (see `spec/open-questions.md` §2).

**Negative / diagnostic example:**

```text
let x = (a
+ b)
```

Here `let x = ` is missing the required `: DeclAnnotation` before `=`, so the
form is structurally invalid before reaching the `(`. The parser should
diagnose the missing declaration annotation. For this example to be structurally
valid, it must be `let x: type = (a` (see §4.1 for the full let grammar).

Line 1 ends inside `(...)`, nesting depth is 1, so the line break is NOT a
form boundary. The form continues through line 2. The parser should consume
both lines and diagnose `UnclosedParen` if the `)` is never found.

## 4. Let statements

### 4.1 Let statement shape

```text
LetStmt ::= "let" LetBinder LetWithClause? "=" PipeExpr
```

The declaration annotation after `:` is required for simple binders. v0.1 does
not make it optional.

AST:

```text
LetAst {
    binder: LetBinderAst,
    with_clause: Option<WithClauseAst>,
    value: ExprAst,
    span: Span
}
```

There is no let-level attribute list. `guard` is an ordinary `Name` unless a
future syntax reintroduces it explicitly.

### 4.2 With clause

```text
LetWithClause ::= "with" WithBlock
WithBlock ::= "{" WithItems? "}"
WithItems ::= Name ("," Name)*
```

`with` is interpreted only inside the let parser state.

AST:

```text
WithClauseAst {
    kind: WithClauseKind,
    span: Span
}

WithClauseKind ::= Lexical | Semantic { items: Vec<NameAst> } | Error(ErrorAst)
```

`with {}` is an explicit lexical-only with clause. It is distinct from having
no with clause. `with { a, b }` preserves a non-empty syntactic payload. No
lifetime or dependency semantics are executed in Raw AST.

`with` without `{` is invalid. `with a, b` is invalid. Trailing commas in
`with { ... }` are rejected. Malformed `with` syntax must not produce
`WithClauseKind::Lexical`; only valid source text `with {}` may produce the
lexical-only with-clause kind.

### 4.3 Let binder

```text
LetBinder ::= SimpleLetBinder | ExtractLetBinder
```

Simple binder:

```text
SimpleLetBinder ::= Name ":" DeclAnnotation

DeclAnnotation ::= BareDeclAnnotation | TypeObjectAnnotation ":" RankAnnotation

BareDeclAnnotation ::= PipeExpr

TypeObjectAnnotation ::= PipeExpr | TypeHole

TypeHole ::= "_"

RankAnnotation ::= PipeExpr
```

Extract binder:

```text
ExtractLetBinder ::= DeduceList CanonicalSkeleton
```

AST:

```text
LetBinderAst ::=
    Simple {
        name: NameAst,
        annotation: DeclAnnotationAst
    }
  | Extract {
        deduce: DeduceListAst,
        skeleton: CanonicalSkeletonAst
    }

DeclAnnotationAst ::=
    Bare(ExprAst)
  | TypeObjectWithRank {
        type_object_annotation: TypeObjectAnnotationAst,
        rank_annotation: ExprAst
    }

TypeObjectAnnotationAst ::=
    Expr(ExprAst)
  | Hole
```

```text
BinderName ::= Name | OperatorName
SimpleLetBinder ::= BinderName ":" DeclAnnotation
```

Operator binder names use the same declaration annotation rules as ordinary
names. For example, operator-named binders may use the explicit rank form:

```text
let +: _: operator = expr
let >: _: operator = expr
```

Phase 4.1 intentionally does not parse `<` as a simple operator binder:

```text
let <: _: operator = expr
```

After `let`, `<` is the strong-context entry for `ExtractLetBinder` and its
deduce list. The parser therefore treats `let <...` as extract-let syntax and
does not reinterpret the token as an operator binder. This keeps the current
binding grammar streaming-friendly. Declaring `<` as an operator binder requires
a future escaping or disambiguation rule.

The `DeclAnnotationAst::Bare` variant covers a single written annotation
expression, such as `let f: fn = ...` and `let t: type = ...`. A bare
declaration annotation is preserved exactly as written.

Rank annotation syntax requires the explicit form:

```text
type_object_annotation : rank_annotation
```

The `DeclAnnotationAst::TypeObjectWithRank` variant covers the form
`let f: _: fn = ...`, where `_` is a `TypeHole` and `fn` is a rank annotation.

v0.1 does not check that annotation names resolve to anything. Annotation
validity is a future semantic pass.

### 4.5 Annotation parsing boundaries

Because `TypeObjectAnnotation ::= PipeExpr | TypeHole` and
`RankAnnotation ::= PipeExpr`, the parser must know where the annotation
sub-expressions stop in each strong context. The termination tokens are
determined by the surrounding syntactic frame.

**In `SimpleLetBinder` context** (parsing the binder after `let`):

- `TypeObjectAnnotation` stops at a top-level `:` (which starts a rank
  annotation), `with`, or `=`.
- If `TypeObjectAnnotation` stopped at `:`, the following expression is
  `RankAnnotation`.
- `RankAnnotation` stops at a top-level `with` or `=`.

Example:

```text
let f: _: fn = expr
     ^^^^^^---- TypeObjectAnnotation stops at the second `:`
            ^^--- RankAnnotation, stops at `=`

let f: _: fn with deps = expr
     ^^^^---- TypeObjectAnnotation, containing `_`
          ^---- this `:` starts RankAnnotation
           ^^--------- RankAnnotation, stops at `with`
```

**In `DeduceList BinderDecl` context** (inside `<...>`):

- `TypeObjectAnnotation` stops at a top-level `,` or `>`.

**In `ParamItem` context** (inside closure-head parameter list):

- `TypeObjectAnnotation` stops at a top-level `,` or `)`.

**In `ReturnBinder` context** (after `->` in a closure head):

- `TypeObjectAnnotation` stops at a top-level `:`, `=>`, or `{`.
- `where` and `acquire` are future reserved stop tokens; they are not
  active Phase 3 parser stops.
- If stopped by `:`, the following expression is `ReturnConstraint`, not
  `RankAnnotation`.

### 4.6 Declaration annotation examples

**Positive examples:**

```text
let f: _: fn = expr
```

AST-level reading:

```text
SimpleLetBinder {
    name: f,
    annotation: TypeObjectWithRank {
        type_object_annotation: TypeHole("_"),
        rank_annotation: Expr(Name("fn"))
    }
}
```

Deferred semantic reading: `f` has an anonymous type-object whose kind/rank
is the source name `fn`.

```text
let t: type = expr
```

AST-level reading:

```text
SimpleLetBinder {
    name: t,
    annotation: Bare(Expr(Name("type")))
}
```

This is a bare annotation containing the expression `type`. v0.1 preserves
that syntax exactly and does not decide whether `type` is semantically valid.

```text
let ns1: namespace = expr
```

AST-level reading:

```text
SimpleLetBinder {
    name: ns1,
    annotation: Bare(Expr(Name("namespace")))
}
```

`namespace` is a source name in annotation position, not a lexical keyword
and not a separate declaration form.

```text
let f: _: fn = expr
```

This is an explicit rank-annotation form. The parser produces:

```text
SimpleLetBinder {
    name: f,
    annotation: TypeObjectWithRank {
        type_object_annotation: TypeHole("_"),
        rank_annotation: Expr(Name("fn"))
    }
}
```

A bare annotation such as `let f: fn = expr` is preserved as raw syntax
(`Bare(Expr(Name("fn")))`) but is **not** function declaration sugar.
The language does not define `let f: fn = ...` as a normative function
declaration spelling.  A function-like declaration must use the explicit
rank-annotation form `let f: _: fn = ...`.

**Alignment:**

```text
let f: _: fn = ...
    |  |  |
    |  |  +-- rank annotation
    |  +----- type-object annotation
    +-------- declared object
```

The explicit rank form has two annotation layers:
`type_object_annotation : rank_annotation`. The bare form has one annotation
expression and must not be lowered or reinterpreted by the parser.

**Negative / non-declaration examples:**

```text
fn f(x) { x }
```

`fn` is an ordinary `Name` token. The parser sees `Name("fn")`, then `Name("f")`,
then `ArgPack(x)`, then an unexpected bare `{`. Since the first token is not
`Name("let")`, the form is selected as an expression form. The parser must not
create a `FnDecl` AST node. Tokens such as `(` or `{` after `f(x)` may produce
diagnostics, but the core conclusion is: no function declaration syntax exists
in v0.1.

```text
type T = expr
```

`type` is an ordinary `Name` token. Form selected as `ExprStmt` because the
first token is not `Name("let")`. The `=` token is not an ordinary expression
operator and may produce `UnexpectedToken` according to the expression grammar.
The parser must not create a `TypeDecl` AST node.

```text
namespace ns = expr
```

`namespace` is an ordinary `Name` token. Form selected as `ExprStmt`.
The `=` may produce `UnexpectedToken`. No `NamespaceDecl` AST node is created.

### 4.7 Terminology note

A **type-object** is a type-theoretic object: the type of some value, or an
object that itself represents a type.

A **kind/rank object** classifies type-objects. In source text, names such as
`fn` and `type` may appear in explicit rank annotation position.

These terms must be distinguished from source names:

- The declared object may be a type-object. The source name `type`, when used
  in bare declaration annotation position, is just a preserved `Name`.
- The source name `type`, when used after the second `:` in an explicit rank
  annotation, may denote the kind/rank of type-objects.
- The source name `fn`, when used after the second `:` in an explicit rank
  annotation, may denote the kind/rank of function type-objects.
- `fn` is not a lexical keyword.
- `type` is not a lexical keyword.
- `namespace` is not a lexical keyword.
- `fn <: type` is a future semantic/rank relation, not a v0.1 parser rule.

The parser does not check these meanings in v0.1. It only preserves the
annotation structure.

**Negative / diagnostic example:**

```text
let t: = expr
```

The parser expects a `DeclAnnotation` after `:`. If `=` follows immediately,
emit `UnexpectedToken` at `=`, insert `ErrorAst` for the annotation, and
continue parsing the value.

```text
let t: 42 = x
```

`42` is a valid `Literal` which is a valid `PipeExpr` which is a valid
`DeclAnnotationAst::Bare`. Annotation validity is deferred,
so this is syntactically valid in v0.1 (even if semantically nonsensical).

## 5. Deduce lists

### 5.1 Meaning

A deduce list declares names that act as holes in following syntax.

The parser only recognizes a deduce list in strong binding contexts.

### 5.2 Syntax

```text
DeduceList ::= "<" BinderDeclList? ">"
BinderDeclList ::= BinderDecl ("," BinderDecl)*
BinderDecl ::= Name [ ":" TypeObjectAnnotation ]
```

An empty deduce list (`<>`) is syntactically valid (the `BinderDeclList?` is
optional). However, an extract-let binder requires at least one hole declaration;
an empty deduce list in extract-let context produces an `InvalidDeduceList`
diagnostic.

AST:

```text
DeduceListAst {
    binders: Vec<BinderDeclAst>,
    span: Span
}

BinderDeclAst {
    name: NameAst,
    annotation: Option<TypeObjectAnnotationAst>,
    span: Span
}
```

### 5.3 Non-context rule

Outside strong binding contexts, `<` and `>` are ordinary symbol tokens. They
do not introduce generic-call syntax or angle-bracket grouping.

In expression/operator contexts, `<`, `>`, `<=`, and `>=` are operator
spellings. The parser must still not globally recognize angle-bracket groups.

**Expression-context example:**

```text
let x: type = a < b > c
```

The parser produces operator sugar for `<` and `>`:

```text
OperatorSugar(">", OperatorSugar("<", a, b), c)
```

This is expression syntax, not a deduce list or generic application.
Because `<` and `>` are non-associative at this level, the ungrouped chain also
produces `ChainedNonAssociativeOperator`.

## 6. Canonical skeleton

### 6.1 Scope

Canonical skeletons appear only in extraction contexts.

v0.1 builds their AST but does not execute matching.

**All canonical skeleton tests in this phase are parser preservation tests
only.**  No semantic commitment is implied by any of the following:

- a hole name in an argpack or as a standalone element;
- a path (`Name::Name`) as a skeleton element or inside an argpack;
- a literal (integer, string) in skeleton position;
- the nesting depth of argpacks;
- the presence or absence of wildcards;
- the relative positioning of holes, node-names, paths, and literals.

Any golden test that includes these shapes documents the AST shape the parser
produces, not a language decision about matching, destructibility, equality,
constructor interpretation, or admissibility.  Whether a particular skeleton
form is admissible or rejected by a future semantic match is a deferred design
decision.

### 6.2 Syntax

```text
CanonicalSkeleton ::= CanonicalElement+

CanonicalElement ::=
    CanonicalArgPack
  | CanonicalAtom

CanonicalArgPack ::= "(" CanonicalSkeletonList? ")"
CanonicalSkeletonList ::= CanonicalSkeleton ("," CanonicalSkeleton)*

CanonicalAtom ::=
    "_"
  | Name
  | Literal
  | CanonicalNavPath

CanonicalNavPath ::= Name ("::" Name)*
```

AST:

```text
CanonicalSkeletonAst ::=
    Segment {
        elements: Vec<CanonicalSkeletonAst>,
        span: Span
    }
  | ArgPack {
        elements: Vec<CanonicalSkeletonAst>,
        span: Span
    }
  | Wildcard {
        span: Span
    }
  | Name {
        name: NameAst,
        role: CanonicalNameRole,
        span: Span
    }
  | NavPath {
        names: Vec<NameAst>,
        span: Span
    }
  | Literal {
        literal: LiteralAst,
        span: Span
    }
```

`CanonicalNameRole`:

```text
CanonicalNameRole ::=
    Hole
  | NodeName
  | Unknown
```

The parser may mark a name as `Hole` if it appears in the active deduce list.

Otherwise it should mark it as `NodeName`.

### 6.3 Examples (parser preservation cases)

The following examples document the AST shapes the parser produces. They do
not express semantic admissibility decisions.

```text
let <head, tail> (head, tail) List::Cons = xs
```

Construct:

```text
Let.Extract
  deduce: [head, tail]
  skeleton:
    Segment
      ArgPack
        Name(head, Hole)
        Name(tail, Hole)
      Path(List, Cons)
  value: ...
```

```text
let <x> (_, x, _) Triple = t
```

Construct:

```text
Let.Extract
  deduce: [x]
  skeleton:
    Segment
      ArgPack
        Wildcard
        Name(x, Hole)
        Wildcard
      Name(Triple, NodeName)
```

## 7. Expressions

### 7.1 Expression entry

```text
Expr ::= PipeExpr
```

AST:

```text
ExprAst ::= Pipe(PipeExprAst)
```

### 7.2 Pipe expression

```text
PipeExpr ::= Segment ("|>" Segment)*
```

Construction rule:

At top-level expression nesting depth, split the token sequence at `|>`.

Each part becomes a `Segment`.

**Negative / diagnostic example:**

```text
|> f
```

A `|>` at form start with no left operand. The parser should emit a
diagnostic (unexpected `|>` at position 0) and treat the left side as
empty or insert an `ErrorAst`. The right side `f` still forms a valid
segment.

```text
x |> |> g
```

A double `|>` with an empty middle segment. The parser should diagnose the
empty segment between the two `|>` operators. Recovery: skip the second
`|>` or insert an `ErrorAst` for the missing segment body.

AST:

```text
PipeExprAst {
    segments: Vec<SegmentAst>,
    span: Span
}
```

For segment index `i`:

```text
has_incoming = i > 0
```

### 7.3 Segment

```text
Segment ::= SegmentElement+
SegmentElement ::= OperatorExpr | ArgPack
OperatorExpr ::= segment-local operator expression built from Atom
```

AST:

```text
SegmentAst {
    elements: Vec<SegmentElementAst>,
    has_incoming: bool,
    span: Span
}
```

Segment parsing does not directly execute function application.

It records element sequence and ArgPack roles.

`OperatorExpr` is the ordinary-operator expression layer built from atoms.
Ordinary operators bind more tightly than both whitespace auto-pipe and `|>`.

Operator-aware AST shape:

```text
OperatorExprAst ::=
    AtomExpr(AtomAst)
  | OperatorSugarAst {
        operator: OperatorName,
        fixity: Prefix | Postfix | Binary,
        args: Vec<OperatorExprAst>,
        span: Span
    }
  | NavPath {
        components: Vec<NavComponentAst>,
        span: Span
    }
  | MemberSugar {
        object: OperatorExprAst,
        selector: SelectorAst,
        span: Span
    }
  | DoubleDotSugar {
        object: OperatorExprAst,
        selector: SelectorAst,
        args: ArgPackAst,
        span: Span
    }
```

Binary and prefix operator sugar belong to `OperatorExprAst`, not to
`AtomAst`. Postfix operator suffixes compose with atom suffix parsing, but the
resulting sugar is still represented at the operator-expression layer.
Therefore, after a postfix operator, continuing suffixes such as `::`, `.`, and
`..` are preserved by operator-level `NavPath`, `MemberSugar`, and
`DoubleDotSugar` nodes.

These operator-level suffix nodes are raw AST preservation only. They do not
perform lookup, lower to calls, or assign member semantics. Navigation order is
inner-to-outer: the leftmost component is the innermost selected symbol, and the
rightmost component is the outermost scope component. Operator names are valid
only as innermost navigation components, such as `+::int::std`, unless a future
design explicitly allows operator-named scopes.

```text
a + b |> f
```

groups as:

```text
(a + b) |> f
```

and:

```text
a b + c
```

groups as:

```text
a (b + c)
```

Operator precedence remains segment-local inside the pipe/segment architecture.
See `spec/operator-design.md` for the full precedence and associativity table.

## 8. Atom construction

### 8.1 Atom base

```text
AtomBase ::=
    Name
  | Literal
  | Group
  | HeadedOrExplicitClosureAst
```

`HeadedOrExplicitClosureAst` means headed inline closure or explicit closure.
Bare `{ ... }` is not an atom-base closure form.

AST:

```text
AtomAst ::=
    Name(NameAst)
  | Literal(LiteralAst)
  | Group(Box<ExprAst>)
  | Closure(ClosureAst)
  | NavPath(...)
  | MemberSugar(...)
  | DoubleDotSugar(...)
  | Error(...)
```

### 8.2 Group

```text
Group ::= "(" PipeExpr ")"
```

A group is valid only if its contents do not contain a top-level comma.

If a parenthesized form contains top-level commas, it is an `ArgPack`, not a group.

Non-ArgPack `(a, b)` is invalid in v0.1.

**Negative / diagnostic example:**

```text
let x: type = (a, b)
```

At the top level of a form, `(a, b)` is interpreted as an `ArgPack` because
it contains a top-level comma. In expression position, a bare `ArgPack` with
no incoming segment or preceding atom triggers `InvalidArgPack` or
`TopLevelComma` diagnostics. The parser should produce a diagnostic and
still parse the `ArgPack` node.

### 8.3 Suffix folding

After parsing `AtomBase`, repeatedly fold suffixes. Parser phase 4 extends this
folding over `OperatorExprAst` so postfix operator results can continue through
the same suffix loop.

Suffixes:

```text
:: NavComponent
. Selector
.. Selector ArgPack
PostfixOperator
```

Folding order is left-to-right.

Postfix unary operators participate in the same left-folding suffix loop as
`::`, `.`, and `..`. Therefore `obj!.field` has the shape `(obj!).field`; the
postfix operator does not terminate suffix parsing.

Before the first postfix operator, suffix folding may produce atom-level
`NavPath`, `MemberSugar`, or `DoubleDotSugar` nodes. After an operator-level value
exists, such as `obj!`, continued suffix folding is preserved with
operator-level `NavPath`, `MemberSugar`, or `DoubleDotSugar` nodes inside
`OperatorExprAst`. This is an AST-shape preservation rule only; it does not
perform lookup, lower suffixes to calls, or assign member semantics.

SelectorAst for this phase:

```text
SelectorAst ::=
    Text(NameAst)
  | Numeric(NumericNameAst)
```

Navigation components for this phase:

```text
NavComponentAst ::=
    Text(NameAst)
  | Numeric(NumericNameAst)
  | Operator(OperatorNameAst)
  | Group(Box<ExprAst>)
  | Error(ErrorAst)
```

`Operator(OperatorNameAst)` is valid only as the innermost navigation
component. It is not valid after `.`, after `..`, or as an outer component
after `::`.

A numeric token (`IntLiteral`) in atom-base position produces a numeric literal
atom (`IntLiteral`). The same token class in selector/name-leaf position
produces a `NumericNameAst`. This distinction is mandatory.

Examples of valid numeric selectors:

```text
obj.1
obj.42
uint8::1
obj..1(args)
1.x
```

### 8.4 Navigation folding

Input:

```text
x :: T :: std
```

AST:

```text
NavPath {
    components: [x, T, std]
}
```

Navigation order is inner-to-outer. The leftmost component is the innermost
selected symbol. The rightmost component is the outermost scope component. Raw
AST preserves source-order navigation components and performs no lookup.

Example:

```text
a b::c
```

is parsed as:

```text
Segment[
  Atom(Name(a)),
  Atom(NavPath(components=[b, c]))
]
```

not as:

```text
NavPath(components=[Segment[a, b], c])
```

```text
NavPath ::= NavComponent "::" NavOuterComponent ("::" NavOuterComponent)*

NavComponent ::= Name | NumericName | OperatorName

NavOuterComponent ::= Name | NumericName | Group
```

Operator names may only be innermost navigation components. Valid shapes
include `+::int::std` and `<<::bit::std`. Invalid shapes include `x::+`,
`x::int::+`, and `+::x::+`.

Parenthesized right-side scope expressions after `::` are allowed:

```text
xxx::(int Vec::std)
```

The grouped expression is stored as a grouped outer navigation component.
Without parentheses, `::` consumes only one immediate valid navigation
component:

```text
xxx::int Vec::std
```

is parsed as two segment elements:

```text
xxx::int
Vec::std
```

### 8.4a Operator sugar

```text
OperatorExprAst ::=
    AtomExpr(AtomAst)
  | OperatorSugarAst {
        operator: OperatorName,
        fixity: Prefix | Postfix | Binary,
        args: Vec<OperatorExprAst>,
        span: Span
    }
  | NavPath { components: Vec<NavComponentAst>, span: Span }
  | MemberSugar { object: OperatorExprAst, selector: SelectorAst, span: Span }
  | DoubleDotSugar {
        object: OperatorExprAst,
        selector: SelectorAst,
        args: ArgPackAst,
        span: Span
    }
```

Operator syntax is preserved as AST sugar at the `OperatorExprAst` layer. The
parser must not lower it into ordinary calls in v0.1. The operator-level
`NavPath`, `MemberSugar`, and `DoubleDotSugar` variants exist only so postfix
operator results can continue through suffix folding, for example
`obj!.field`, `obj.field?`, and `obj..map(a)!`.

Examples:

```text
obj!    => postfix OperatorSugarAst
a + b   => binary OperatorSugarAst
-x      => prefix OperatorSugarAst
```

Prefix `-x` is not a negative literal; the lexer emits `-` and `x`
separately.

Comparison, equality, and compound-looking operator chains are
non-associative in this phase. The parser diagnoses:

```text
chained non-associative operator requires explicit grouping
```

for ungrouped syntax such as `a < b < c`, `a == b == c`, and `a += b += c`.

### 8.5 Member sugar

Input:

```text
object.field
object.42
1.x
```

AST:

```text
MemberSugar {
    object,
    selector: SelectorAst,
    span
}
```

The selector is `Text(NameAst)` for textual names and `Numeric(NumericNameAst)` for
numeric names such as `1` and `42`.

Parser constraint:

The token after `.` must be a valid selector token (`Name` or `IntLiteral`).

Invalid:

```text
obj.(field)
obj."field"
obj.+
```

**Negative / diagnostic example:**

```text
obj.(field)
```

The token after `.` is `Symbol("(")`, not `Name` or `IntLiteral`. Emit
`ExpectedNameAfterDot` with primary span on `.`. Consume the `.` and stop
suffix folding; the atom is just `Name(obj)` without member sugar.

### 8.6 Double-dot sugar

Input:

```text
object..method(args)
object..1(args)
```

AST:

```text
DoubleDotSugar {
    object,
    selector: SelectorAst,
    args: ArgPackAst,
    span
}
```

Parser constraints:

- `..` must be followed by a valid selector token (`Name` or `IntLiteral`).
- The selector must be followed by `ArgPack`.

Missing ArgPack examples:

```text
obj..member
obj..42
obj..(method)
```

**Negative / diagnostic examples:**

```text
obj..42
```

`..` is followed by `IntLiteral("42")` which is a valid numeric selector,
but `42` is NOT followed by `ArgPack`. Emit
`ExpectedArgPackAfterDoubleDotName` with primary span on `42`. Consume
`..` and selector, stop suffix folding. Do not construct a `DoubleDotSugar` node.

```text
obj..(method)
```

The token after `..` is `Symbol("(")`, not `Name` or `IntLiteral`. Emit
`ExpectedNameAfterDoubleDot` with primary span on `..`. Consume `..` and
continue without double-dot sugar.

```text
obj..+
```

The token after `..` is an operator, not `Name` or `IntLiteral`. Same as
`ExpectedNameAfterDoubleDot` case above. Operator selectors are valid only
after `::`, not after `..`.

## 9. ArgPack

### 9.1 Syntax

```text
ArgPack ::= "(" ArgList? ")"
ArgList ::= PipeExpr ("," PipeExpr)*
```

AST:

```text
ArgPackAst {
    args: Vec<ExprAst>,
    span: Span
}
```

### 9.2 Role assignment

Every `ArgPack` appearing inside a `Segment` must receive a role:

```text
ArgPackRole ::=
    SourcePack
  | InsertPack
  | RightTargetSubsegment
  | Unknown
```

`Unknown` is allowed only for error recovery.

### 9.3 Segment role algorithm

Given:

```text
Segment(elements, has_incoming)
```

where elements are `OperatorExpr` or `ArgPack`.

Process left-to-right.

State:

```text
index: usize
insert_used: bool = false
```

Rules:

1. If an `ArgPack` appears at index `0`, mark it `SourcePack`.

2. If an `ArgPack` appears after an `OperatorExpr`, and:

   - `has_incoming == true`
   - `insert_used == false`

   then mark it `InsertPack` and set `insert_used = true`.

3. Otherwise, mark the `ArgPack` as `RightTargetSubsegment`.

4. A `RightTargetSubsegment` starts a recursively parsed subsegment extending from that ArgPack to the current segment boundary.

The AST may either:

- store the flat segment with roles, or
- explicitly nest right-target subsegments.

v0.1 should prefer a flat representation with roles unless a later pass needs nested structure.

### 9.4 Examples

Input:

```text
(args) f g
```

Segment roles:

```text
ArgPack(args): SourcePack
Name(f)
Name(g)
```

Input:

```text
x |> f (a) g
```

Right segment roles:

```text
Name(f)
ArgPack(a): InsertPack
Name(g)
```

Input:

```text
f (a) g
```

Single segment, no incoming input:

```text
Name(f)
ArgPack(a): RightTargetSubsegment
Name(g)
```

Input:

```text
x |> f (a) g (b) h
```

Right segment:

```text
Name(f)
ArgPack(a): InsertPack
Name(g)
ArgPack(b): RightTargetSubsegment
Name(h)
```

## 10. Closure AST

### 10.1 Closure categories

```text
ClosureAst ::=
    InlineClosureAst
  | ExplicitClosureAst
```

### 10.2 Inline closure

```text
InlineClosureAst ::= FnHeadPrefix BodyBlock
```

Bare `{ ... }` in atom position is not a closure literal and must not produce
`ClosureAst`. Braces delimit a closure body only after explicit closure syntax,
such as `FnHeadPrefix => BodyBlock`, or after a valid closure head where the
inline headed form is accepted.

`{ ... }` is not a normal block expression.

### 10.3 Explicit closure

```text
ExplicitClosureAst ::= FnHeadPrefix "=>" BodyBlock
```

Minimal form:

```text
() => {}
```

### 10.4 Body block

```text
BodyBlock ::= "{" Form* "}"
```

AST:

```text
BodyBlockAst {
    forms: Vec<FormAst>,
    span: Span
}
```

Inside `{ ... }`, form boundaries are `;`, `}`, and EOF.  Newline promotion
to form separator is suppressed because nesting depth is non-zero inside the
body block.  This means `{ x \n y }` parses as a single form containing a
segment with two atoms `x y`, not as two separate forms.  This is the
provisional v0.1 rule; the broader language-design question of body-block
form separation remains open. Semicolon-separated forms still split normally:
`{ x; y; }` contains two body forms.

## 11. Closure head

### 11.1 Full order

```text
FnHeadPrefix ::=
    DeduceList?
    CaptureClause?
    ParamClause?
    FnItemTraitClause?
    ReturnClause?

// Future reserved, not implemented in Phase 3.1:
//   WhereClause?
//   AcquireClause?
```

The order is fixed.

Clauses may be omitted.

### 11.2 Capture clause

```text
CaptureClause ::= "[" CaptureItemList? "]"
CaptureItemList ::= CaptureItem ("," CaptureItem)*
```

v0.1 parses `CaptureClause` as a bracket-delimited clause. Capture items are
stored as syntactic `CaptureItemAst` entries containing preserved expression
structure. No capture validation, move/ref/copy interpretation, or capture
analysis is performed.

Suggested AST:

```text
CaptureClauseAst {
    items: Vec<CaptureItemAst>,
    span: Span
}
```

### 11.3 Param clause

```text
ParamClause ::= "(" ParamItemList? ")"
ParamItemList ::= ParamItem ("," ParamItem)*
```

This is a closure-head parameter context, not a general `ArgPack`.

AST:

```text
ParamClauseAst {
    params: Vec<ParamItemAst>,
    span: Span
}
```

### 11.4 Param item

```text
ParamItem ::=
    Name [ ":" TypeObjectAnnotation ]
  | DeduceList? CanonicalSkeleton [ ":" TypeObjectAnnotation ]
```

AST:

```text
ParamItemAst ::=
    NameParam {
        name: NameAst,
        type_object_annotation: Option<TypeObjectAnnotationAst>,
        span: Span
    }
  | ExtractParam {
        deduce: Option<DeduceListAst>,
        skeleton: CanonicalSkeletonAst,
        type_object_annotation: Option<TypeObjectAnnotationAst>,
        span: Span
    }
```

### 11.5 Function item trait clause

```text
FnItemTraitClause ::= ":" TraitExpr
TraitExpr ::= PipeExpr
```

AST field:

```text
fn_item_trait: Option<ExprAst>
```

### 11.6 Return clause

```text
ReturnClause ::= "->" ReturnBinder [ ":" ReturnConstraint ]
```

```text
ReturnBinder ::=
    TypeObjectAnnotation
  | DeduceList CanonicalSkeleton
```

```text
ReturnConstraint ::= PipeExpr
```

AST:

```text
ReturnClauseAst {
    binder: ReturnBinderAst,
    constraint: Option<ExprAst>,
    span: Span
}
```

```text
ReturnBinderAst ::=
    TypeExpr(ExprAst)
  | ExtractType {
        deduce: DeduceListAst,
        skeleton: CanonicalSkeletonAst
    }
  | Error(ErrorAst)
```

Return clauses preserve syntax only. `TypeExpr`, `ExtractType`, and optional
return constraints are not type-checked or constraint-solved in v0.1.

### 11.7 Where clause (future reserved)

> **Not implemented in Phase 3.1.** `where` is a reserved closure-head
> position.
> It remains an ordinary name outside a future where-parser state.
> Concrete `where` syntax is deferred until a dedicated closure-clause parser
> and logical-constraint grammar exist.

### 11.8 Acquire clause (future reserved)

> **Not implemented in Phase 3.1.** `acquire` is a reserved closure-head
> position.
> It remains an ordinary name outside a future acquire-parser state.
> Concrete `acquire` syntax is deferred until a dedicated closure-clause parser
> and resource-requirement grammar exist.

### 11.9 Closure recognition algorithm

When the expression parser expects an atom:

1. Attempt finite lookahead for `FnHeadPrefix`.
2. If the prefix is followed by `=>` and `{`, parse `ExplicitClosureAst`.
3. If the prefix is followed directly by `{`, parse headed `InlineClosureAst`.
4. If these attempts fail, restore cursor and parse ordinary atom.

A bare `{` in atom position is not a closure-recognition entry point.

This is finite lookahead, not semantic backtracking.
Failed closure-head lookahead must not leak diagnostics or consume tokens.
Committed malformed closure parsing keeps diagnostics. Nested diagnostic gates
must preserve outer gated diagnostics until the outer gate is explicitly kept
or dropped.

The following are not recognized as successful closure heads in Phase 3.1:

```text
x => { x }
x { x }
where C => { x }
acquire A => { x }
(x) where C => { x }
(x) acquire A => { x }
<T>(x: T) where C => { x }
<T>(x: T) acquire A => { x }
```

**Negative / diagnostic example:**

```text
x => { }
```

The closure recognition algorithm first checks:

- Is `x` a `FnHeadPrefix`? No — `FnHeadPrefix ::= DeduceList? CaptureClause?
  ParamClause? FnItemTraitClause? ReturnClause?` (plus future reserved
  WhereClause?/AcquireClause?).
  A bare `Name("x")` does not match any of these clauses.
- Therefore the lookahead fails. The parser backtracks and parses `x` as an
  ordinary `Atom(Name("x"))`.
- Then `=>` is encountered in a non-closure-head context. Since `=>` is only
  valid as a closure-head `=> BodyBlock` separator, emit `UnexpectedToken`
  at `=>`.
- If the parser encounters `{` next in atom position, it is unexpected. It must
  not parse as an `InlineClosureAst`.

v0.1 does **not** support bare-name parameter closure sugar. Valid minimal
forms remain `() => {}` and `(x) => {}` where the `()` is a `ParamClause`.
`(x) => {}` with a single param inside parens is the simplest parametrized
explicit closure form.

## 12. Match-style expression

The parser must not special-case `match`.

Input:

```text
obj (
    <val: _>(val option::Sum) { ... },
    (_ option::None) { ... }
) match
```

Expected high-level AST shape:

```text
PipeExpr
  Segment
    Atom Name(obj)
    ArgPack RightTargetSubsegment
      Explicit/Inline Closure AST arm 1
      Explicit/Inline Closure AST arm 2
    Atom Name(match)
```

Whether `match` consumes the closure AST arms is a future semantic/meta-function pass.

**Negative / diagnostic example:**

```text
match obj
```

At form start, `match` is a `Name` token, not syntax. The parser produces:

```text
Form.Expr(
  PipeExpr(
    Segment(
      Atom(Name("match")),
      Atom(Name("obj"))
    )
  )
)
```

No `MatchExpr` is created. This is correct v0.1 behavior.

```text
obj match (a) { }
```

The parser sees `Name("obj")`, then `Name("match")`, then `ArgPack(a)`,
then an unexpected bare `{`. The high-level expression prefix is:

```text
PipeExpr
  Segment
    Atom Name(obj)
    Atom Name(match)
    ArgPack RightTargetSubsegment
```

No special match-arm relationship exists at the parser level. Bare `{}` does
not become a match arm; match-style expressions that use closure arms must use
valid headed closure syntax inside the argument pack.

## 13. Error nodes

Parser errors should produce diagnostics and continue.

AST may contain:

```text
ErrorAst {
    message: String,
    span: Span
}
```

Recommended recoverable errors:

- expected `Name` after `.`
- expected `Name` after `..`
- expected `ArgPack` after `.. Name`
- unclosed `(`
- unclosed `[`
- unclosed `{`
- top-level comma outside `ArgPack`
- invalid deduce list
- invalid closure head
- invalid canonical skeleton

## 14. Normative non-interpretation list

The parser must not construct special semantic AST nodes for:

```text
return
else
match
drop
move
ref
sync
effect
fn
type
meta
runtime
compile
namespace
struct
```

They remain names inside expression AST unless appearing in explicitly defined strong contexts.

## 15. Representative golden coverage

The golden test suite covers at least the following syntax areas.
For the current full test count, see `spec/implementation-status-v0.1.md`.

```text
let t: type = int Option Vec

let val: int::std elements_type::Vec = expr

let ns1: namespace = expr

let f: _: fn = expr

let f: fn = expr

let <head, tail> (head, tail) List::Cons = xs

let <x> (_, x, _) Triple = t

obj.field

obj..map(a, b)

(args) f g

f (a) g

x |> f (a) g

x |> f (a) g (b) h

() => {}

<T>(x: T): runtime -> T => {
    x
}

obj (
    <val: _>(val option::Sum) { ... },
    (_ option::None) { ... }
) match

let Vec === Vector::collections::std

let + === +::checked_int

let << === <<::xxx_bit

let local === some_entity

let + === +
```

## 16. Alias binding (Phase 4.4)

### 16.1 Scope

Phase 4.4 implements raw parser preservation for alias binding:

```text
let binder === EntityRef
```

Alias binding is distinct from ordinary `let`. It does not have declaration
annotations, `with` clauses, deduce lists, canonical
skeletons, or `=` value expressions.

The parser preserves alias binding as raw AST. It does not resolve entities,
validate operator identity, perform name/operator lookup, or lower aliases.

### 16.2 Grammar

```text
AliasBinding ::= "let" AliasBinder "===" EntityRef

AliasBinder ::= Name | OperatorName

EntityRef ::= EntityNavigation

EntityNavigation ::= EntityComponent ("::" EntityOuterComponent)*

EntityComponent ::= Name | OperatorName

EntityOuterComponent ::= Name
```

EntityRef navigation order is inner-to-outer. The leftmost component is the
innermost selected symbol, and the rightmost component is the outermost scope
component. Raw AST preserves source-order navigation components and performs no
lookup. Operator names are valid only as innermost entity-reference components
unless a future design explicitly allows operator-named scopes.

`===` is a structural delimiter token (`Symbol::TripleEqual`), not an
expression operator. It is not available as `OperatorName`.

`<` is not accepted as an alias binder; it goes to extract-let.

### 16.3 Dispatch

After consuming `let`:

1. If next token is `guard` → ordinary let only.
2. If next token is `<` → extract let only.
3. If next token is a valid `AliasBinder` and the token after is `===` →
   alias let.
4. Otherwise → ordinary let.

### 16.4 AST

```text
FormAst ::=
    Let(LetAst)
  | AliasLet(LetAliasAst)
  | Expr(ExprAst)
  | Error(ErrorAst)

LetAliasAst {
    binder: AliasBinderAst,
    target: EntityRefAst,
    span: Span
}

AliasBinderAst ::=
    Name(NameAst)
  | Operator(OperatorNameAst)
  | Error(ErrorAst)

EntityRefAst {
    components: Vec<NavComponentAst>,
    span: Span
}
```

### 16.5 EntityRef parsing

EntityRef is parsed only inside alias-let RHS. It is not a general expression
parser mode.

Operator names are valid only as the innermost entity-reference navigation
component. If an operator appears as an outer component after `::`, the parser
emits `InvalidEntityRef`.

After completing the entity reference, the parser checks for residual
expression tokens. If the current position is not a form boundary (EOF,
semicolon, right brace, or promoted newline), the parser emits
`UnexpectedAliasRhsExpression` and recovers to the form boundary.

### 16.6 Diagnostics

| Code                           | Trigger                                                                         |
| ------------------------------ | ------------------------------------------------------------------------------- |
| `ExpectedAliasTarget`          | `===` is followed by a token that cannot start an `EntityRef`.                  |
| `InvalidEntityRef`             | Malformed entity reference (e.g., operator in segment position, dangling `::`). |
| `UnexpectedAliasRhsExpression` | Valid `EntityRef` was parsed, but residual expression tokens follow.            |

`InvalidAliasBinder` is reserved for future use but not currently emitted.

### 16.7 Non-interpretation

The parser does not:

- resolve the target entity;
- check whether the target exists;
- validate operator alias identity (`spelling + fixity + arity`);
- perform name lookup, operator lookup, namespace resolution, or dependency
  resolution;
- lower aliases into runtime bindings.

### 16.8 Examples

Valid:

```text
let Vec === Vector::collections::std
let map === map::iter::std
let + === +::checked_int
let << === <<::xxx_bit
let local === some_entity
let + === +
```

Invalid:

```text
let x ===
let x === a |> f
let x === { body }
let x === (a, b)
let x === a + b
let x === a::+::b
let (x) === y
let <x> x === y
let guard x === y
```

## 17. v0.1 success criterion

A conforming v0.1 frontend can:

1. tokenize source text;
2. parse forms;
3. build AST according to this document;
4. produce diagnostics with spans;
5. dump tokens/AST/diagnostics in stable text form;
6. pass golden tests.

A conforming v0.1 frontend does not need to run any program.

## 18. Negative / diagnostic examples index

Every major rule should have at least one positive and one negative example.
This index cross-references the negative/diagnostic examples throughout this
document.

| Section                              | Negative / diagnostic example               | Expected diagnostic                                                                      |
| ------------------------------------ | ------------------------------------------- | ---------------------------------------------------------------------------------------- |
| §3.3 Form boundary                   | `let x: type = (a` with newline             | `UnclosedParen` if `)` never found                                                       |
| §4.6 DeclAnnotation                  | `let t: = x` after `let`                    | `UnexpectedToken` at `=`                                                                 |
| §4.6 DeclAnnotation                  | `let t: 42 = x` (syntactically valid)       | No diagnostic (valid syntax)                                                             |
| §4.6 DeclAnnotation                  | `fn f(x) { x }` — not a FnDecl              | No diagnostic (ordinary expr; adjacencies may vary)                                      |
| §4.6 DeclAnnotation                  | `type T = expr` — not a TypeDecl            | `=` may produce `UnexpectedToken`                                                        |
| §4.6 DeclAnnotation                  | `namespace ns = expr` — not a NamespaceDecl | `=` may produce `UnexpectedToken`                                                        |
| §5.3 Expression-context `<`/`>`      | `a < b > c`                                 | `ChainedNonAssociativeOperator` on the ungrouped non-associative chain                   |
| §7.2 Pipe split                      | `\|> f` at form start                       | `UnexpectedToken` at `\|>`                                                               |
| §7.2 Pipe split                      | `x \|> \|> g` (empty middle)                | Diagnostic on empty segment                                                              |
| §8.2 Group                           | `let x: type = (a, b)` at expr top          | `TopLevelComma` or `InvalidArgPack`                                                      |
| §8.5 Member `.`                      | `obj.42`                                    | No diagnostic (valid numeric selector)                                                   |
| §8.5 Member `.`                      | `obj.(field)`                               | `ExpectedNameAfterDot`                                                                   |
| §8.5 Member `.`                      | `obj."field"`                               | `ExpectedNameAfterDot`                                                                   |
| §8.6 Double-dot `..`                 | `obj..42`                                   | No diagnostic expected for selector; `ExpectedArgPackAfterDoubleDotName` if no ArgPack   |
| §8.6 Double-dot `..`                 | `obj..1` (no `ArgPack`)                     | `ExpectedArgPackAfterDoubleDotName`                                                      |
| §8.6 Double-dot `..`                 | `obj..(method)`                             | `ExpectedNameAfterDoubleDot`                                                             |
| §8.6 Double-dot `..`                 | `obj..+`                                    | `ExpectedNameAfterDoubleDot` (operator selectors are valid only after `::`)              |
| §11.9 Closure lookahead              | `x => { }` rejected as non-closure-head     | `UnexpectedToken` at `=>`                                                                |
| §12 Match non-special                | `match obj` at form start                   | No diagnostic (name, not syntax)                                                         |
| §12 Match non-special                | `obj match (a) { }`                         | No diagnostic (name + argpack + closure)                                                 |
