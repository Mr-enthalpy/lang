# File: spec/ast-construction-v0.1.md

# AST Construction Rules v0.1

## 0. Scope

This document defines how source tokens are converted into AST in `v0.1`.

It defines syntax recognition and AST construction only.

It does not define:

* type checking
* kind checking
* overload resolution
* canonical matching
* closure object materialization
* match semantics
* effect semantics
* NLL/lifetime analysis
* drop insertion
* code generation

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

* lexical token
* parser context
* AST node
* future semantic interpretation

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

For v0.1, a form ends at:

* `;`
* top-level line break
* `}`
* EOF

A line break is top-level only if the current nesting depth of `()`, `[]`, `{}` is zero.

This is a provisional v0.1 rule.

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
LetStmt ::= "let" LetAttr* LetBinder LetWithClause? "=" PipeExpr
```

The declaration annotation after `:` is required for simple binders. v0.1 does
not make it optional.

AST:

```text
LetAst {
    attrs: Vec<LetAttrAst>,
    binder: LetBinderAst,
    with_deps: Vec<NameAst>,
    value: ExprAst,
    span: Span
}
```

### 4.2 Let attributes

```text
LetAttr ::= "guard"
```

`guard` is interpreted only inside the let parser state.

Outside a let statement it is an ordinary name.

AST:

```text
LetAttrAst ::= Guard
```

### 4.3 With clause

```text
LetWithClause ::= "with" NameList
NameList ::= Name ("," Name)*
```

`with` is interpreted only inside the let parser state.

AST:

```text
with_deps: Vec<NameAst>
```

No lifetime semantics are executed in v0.1.

### 4.4 Let binder

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

The `DeclAnnotationAst::Bare` variant covers the surface
forms `let f: fn = ...` (single name as annotation) and `let t: type = ...`.
These are preserved as bare expressions without semantic desugaring.

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

- `TypeObjectAnnotation` stops at a top-level `:`, `where`, `acquire`,
  `=>`, or `{`.
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

Deferred semantic reading: `t` is declared as a type-object whose kind/rank
is the source name `type`.

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
let f: fn = expr
```

Surface sugar: the annotation is written as bare `fn`. The parser produces

```text
SimpleLetBinder {
    name: f,
    annotation: Bare(Expr(Name("fn")))
}
```

and preserves the raw written form. A future declaration-analysis pass may
treat bare `fn` as equivalent to `_: fn` when `fn` resolves as a
kind/rank annotation. v0.1 must not perform semantic desugaring.

**Alignment:**

```text
let f: _: fn = ...    let t:    type = ...
    │   │  │              │       │
    │   │  └─ rank         │       └─ rank (source name `type`)
    │   └─ type-object     └─ itself a type-object
    └─ declared object
```

The declared object (`f`) aligns with the declared object (`t`).
The anonymous type-object (`_`) aligns with the type-object layer occupied by `t` itself.
The rank annotation (`fn`) aligns with the rank annotation (`type`).

More precisely: `t` and `_` occupy the type-object layer. `fn` and the
source name `type` occupy the kind/rank annotation layer.

**Negative / non-declaration examples:**

```text
fn f(x) { x }
```

`fn` is an ordinary `Name` token. The parser sees `Name("fn")`, then `Name("f")`,
then `ArgPack(x)`, then `InlineClosureAst`. Since the first token is not
`Name("let")`, the form is selected as `ExprStmt`. The parser must not
create a `FnDecl` AST node. Depending on future expression adjacency rules,
tokens such as `(` or `{` after `f(x)` may produce diagnostics, but the
core conclusion is: no function declaration syntax exists in v0.1.

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
`fn` and `type` may appear in kind/rank annotation position.

These terms must be distinguished from source names:

* The declared object may be a type-object. The source name `type`, when used
  in declaration annotation position, may denote the kind/rank of type-objects.
* The source name `fn`, when used in declaration annotation position, may
  denote the kind/rank of function type-objects.
* `fn` is not a lexical keyword.
* `type` is not a lexical keyword.
* `namespace` is not a lexical keyword.
* `fn <: type` is a future semantic/rank relation, not a v0.1 parser rule.

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

Outside strong binding contexts, `<` and `>` are ordinary symbol tokens.

The parser must not globally recognize angle-bracket groups.

**Negative / diagnostic example:**

```text
let x: type = a < b > c
```

Since `SegmentElement ::= Atom | ArgPack` and `Atom` does not include a
`SymbolAtom` or operator-atom variant, there is no syntactic slot for `<`
and `>` in ordinary expression position. The lexer still produces
`Symbol("<")` and `Symbol(">")` tokens, but the parser must emit
`UnexpectedToken` for each such symbol when encountered outside a
deduce-list context.

The parser should produce:

```text
Atom(Name(a))  ErrorAst(<)  Atom(Name(b))  ErrorAst(>)  Atom(Name(c))
```

or skip to the next synchronization point after the first `<`.

In v0.1, `<` and `>` in non-binding contexts are parse errors, not valid
expression symbols.

## 6. Canonical skeleton

### 6.1 Scope

Canonical skeletons appear only in extraction contexts.

v0.1 builds their AST but does not execute matching.

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
  | CanonicalPath

CanonicalPath ::= Name ("::" Name)*
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
  | Path {
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

### 6.3 Examples

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
      Path(Triple)
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
SegmentElement ::= Atom | ArgPack
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

## 8. Atom construction

### 8.1 Atom base

```text
AtomBase ::=
    Name
  | Literal
  | Group
  | ClosureAst
```

AST:

```text
AtomAst ::=
    Name(NameAst)
  | Literal(LiteralAst)
  | Group(Box<ExprAst>)
  | Closure(ClosureAst)
  | Path(...)
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

### 8.3 Atom suffix folding

After parsing `AtomBase`, repeatedly fold atom suffixes.

Suffixes:

```text
:: Name
. Name
.. Name ArgPack
```

Folding order is left-to-right.

### 8.4 Path folding

Input:

```text
base :: a :: b
```

AST:

```text
Path {
    base,
    names: [a, b]
}
```

Example:

```text
a b::c
```

is parsed as:

```text
Segment[
  Atom(Name(a)),
  Atom(Path(base=Name(b), names=[c]))
]
```

not as:

```text
Path(base = Segment[a, b], names=[c])
```

### 8.5 Member sugar

Input:

```text
object.field
```

AST:

```text
MemberSugar {
    object,
    field,
    span
}
```

Parser constraint:

The token after `.` must be `Name`.

Invalid:

```text
obj.(field)
```

**Negative / diagnostic example:**

```text
obj.42
```

The token after `.` is `IntLiteral`, not `Name`. Emit `ExpectedNameAfterDot`
with primary span on `.`. Consume the `.` and stop suffix folding; the atom
is just `Name(obj)` without member sugar. The `42` remains as a following
atom.

```text
obj.(field)
```

The token after `.` is `Symbol("(")`, not `Name`. Same diagnostic and
recovery as above.

### 8.6 Double-dot sugar

Input:

```text
object..method(args)
```

AST:

```text
DoubleDotSugar {
    object,
    method,
    args,
    span
}
```

Parser constraints:

* `..` must be followed by `Name`.
* The `Name` must be followed by `ArgPack`.

Invalid:

```text
obj..method
obj..(method)
```

**Negative / diagnostic examples:**

```text
obj..42
```

The token after `..` is `IntLiteral`, not `Name`. Emit
`ExpectedNameAfterDoubleDot` with primary span on `..`. Consume `..` and
continue without double-dot sugar.

```text
obj..method
```

`..` is followed by `Name("method")` which is valid, but `method` is NOT
followed by `ArgPack`. Emit `ExpectedArgPackAfterDoubleDotName` with
primary span on `method`. Do not construct a `DoubleDotSugar` node.
The atom is left as the base object (before `..`) or wrapped in
`ErrorAtom(DoubleDotMissingArgPack { object, method })`.

```text
obj..(method)
```

`..` is followed by `Symbol("(")` which is not `Name`. Same as
`ExpectedNameAfterDoubleDot` case.

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

where elements are `Atom` or `ArgPack`.

Process left-to-right.

State:

```text
index: usize
insert_used: bool = false
```

Rules:

1. If an `ArgPack` appears at index `0`, mark it `SourcePack`.

2. If an `ArgPack` appears at index `> 0`, and:

    * `has_incoming == true`
    * `insert_used == false`

   then mark it `InsertPack` and set `insert_used = true`.

3. Otherwise, mark the `ArgPack` as `RightTargetSubsegment`.

4. A `RightTargetSubsegment` starts a recursively parsed subsegment extending from that ArgPack to the current segment boundary.

The AST may either:

* store the flat segment with roles, or
* explicitly nest right-target subsegments.

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
InlineClosureAst ::= FnHeadPrefix? BodyBlock
```

Minimal form:

```text
{}
```

In atom position, `{ ... }` is always parsed as `InlineClosureAst`.

It is not a normal block expression.

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

## 11. Closure head

### 11.1 Full order

```text
FnHeadPrefix ::=
    DeduceList?
    CaptureClause?
    ParamClause?
    FnItemTraitClause?
    ReturnClause?
    WhereClause?
    AcquireClause?
```

The order is fixed.

Clauses may be omitted.

### 11.2 Capture clause

```text
CaptureClause ::= "[" CaptureItemList? "]"
CaptureItemList ::= CaptureItem ("," CaptureItem)*
```

v0.1 parses `CaptureClause` as a bracket-delimited clause. Capture items are
stored as token-tree-like `CaptureItemAst` placeholders. The exact internal
structure of capture items is not specified in v0.1; parsing depth is
determined by the concrete `CaptureItemAst` definition chosen at
implementation time.

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
    TypeObjectAnnotation(TypeObjectAnnotationAst)
  | ExtractType {
        deduce: DeduceListAst,
        skeleton: CanonicalSkeletonAst
    }
```

### 11.7 Where clause

```text
WhereClause ::= "where" ConstraintExpr
ConstraintExpr ::= PipeExpr
```

`where` is a contextual word only in closure-head parsing state.

### 11.8 Acquire clause

```text
AcquireClause ::= "acquire" AcquireExpr
AcquireExpr ::= PipeExpr
```

`acquire` is a contextual word only in closure-head parsing state.

### 11.9 Closure recognition algorithm

When the expression parser expects an atom:

1. If the current token is `{`, parse `InlineClosureAst`.
2. Otherwise attempt finite lookahead for `FnHeadPrefix`.
3. If the prefix is followed by `=>` and `{`, parse `ExplicitClosureAst`.
4. If the prefix is followed directly by `{`, parse prefixed `InlineClosureAst`.
5. If these attempts fail, restore cursor and parse ordinary atom.

This is finite lookahead, not semantic backtracking.

**Negative / diagnostic example:**

```text
x => { }
```

The closure recognition algorithm first checks:
- Is `x` a `FnHeadPrefix`? No — `FnHeadPrefix ::= DeduceList? CaptureClause?
  ParamClause? FnItemTraitClause? ReturnClause? WhereClause? AcquireClause?`.
  A bare `Name("x")` does not match any of these clauses.
- Therefore the lookahead fails. The parser backtracks and parses `x` as an
  ordinary `Atom(Name("x"))`.
- Then `=>` is encountered in a non-closure-head context. Since `=>` is only
  valid as a closure-head `=> BodyBlock` separator, emit `UnexpectedToken`
  at `=>`.
- If the parser encounters `{` next, it may parse it as an `InlineClosureAst`
  depending on remaining context.

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
then `InlineClosureAst`. The high-level AST is:

```text
PipeExpr
  Segment
    Atom Name(obj)
    Atom Name(match)
    ArgPack RightTargetSubsegment
    Atom InlineClosure({ })
```

No special match-arm relationship exists at the parser level.

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

* expected `Name` after `.`
* expected `Name` after `..`
* expected `ArgPack` after `.. Name`
* unclosed `(`
* unclosed `[`
* unclosed `{`
* top-level comma outside `ArgPack`
* invalid deduce list
* invalid closure head
* invalid canonical skeleton

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
mod
struct
```

They remain names inside expression AST unless appearing in explicitly defined strong contexts.

## 15. Minimum golden cases

Implement tests for at least:

```text
let t: type = int Option Vec

let val: std::int Vec::elements_type = expr

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

{}

() => {}

<T>(x: T): runtime -> T => {
    x
}

obj (
    <val: _>(val option::Sum) { ... },
    (_ option::None) { ... }
) match
```

## 16. v0.1 success criterion

A conforming v0.1 frontend can:

1. tokenize source text;
2. parse forms;
3. build AST according to this document;
4. produce diagnostics with spans;
5. dump tokens/AST/diagnostics in stable text form;
6. pass golden tests.

A conforming v0.1 frontend does not need to run any program.

## 17. Negative / diagnostic examples index

Every major rule should have at least one positive and one negative example.
This index cross-references the negative/diagnostic examples throughout this
document.

| Section | Negative / diagnostic example | Expected diagnostic |
|---|---|---|
| §3.3 Form boundary | `let x: type = (a` with newline | `UnclosedParen` if `)` never found |
| §4.6 DeclAnnotation | `let t: = x` after `let` | `UnexpectedToken` at `=` |
| §4.6 DeclAnnotation | `let t: 42 = x` (syntactically valid) | No diagnostic (valid syntax) |
| §4.6 DeclAnnotation | `fn f(x) { x }` — not a FnDecl | No diagnostic (ordinary expr; adjacencies may vary) |
| §4.6 DeclAnnotation | `type T = expr` — not a TypeDecl | `=` may produce `UnexpectedToken` |
| §4.6 DeclAnnotation | `namespace ns = expr` — not a NamespaceDecl | `=` may produce `UnexpectedToken` |
| §5.3 Non-context `<>` | `a < b > c` in expr | `UnexpectedToken` at `<` |
| §7.2 Pipe split | `\|> f` at form start | `UnexpectedToken` at `\|>` |
| §7.2 Pipe split | `x \|> \|> g` (empty middle) | Diagnostic on empty segment |
| §8.2 Group | `let x: type = (a, b)` at expr top | `TopLevelComma` or `InvalidArgPack` |
| §8.5 Member `.` | `obj.42` | `ExpectedNameAfterDot` |
| §8.5 Member `.` | `obj.(field)` | `ExpectedNameAfterDot` |
| §8.6 Double-dot `..` | `obj..42` | `ExpectedNameAfterDoubleDot` |
| §8.6 Double-dot `..` | `obj..method` (no `ArgPack`) | `ExpectedArgPackAfterDoubleDotName` |
| §8.6 Double-dot `..` | `obj..(method)` | `ExpectedNameAfterDoubleDot` |
| §11.9 Closure lookahead | `x => { }` rejected as non-closure-head | `UnexpectedToken` at `=>` |
| §12 Match non-special | `match obj` at form start | No diagnostic (name, not syntax) |
| §12 Match non-special | `obj match (a) { }` | No diagnostic (name + argpack + closure) |

