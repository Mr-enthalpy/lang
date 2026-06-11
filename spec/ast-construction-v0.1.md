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

## 4. Let statements

### 4.1 Let statement shape

```text
LetStmt ::= "let" LetAttr* LetBinder LetWithClause? "=" PipeExpr
```

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
SimpleLetBinder ::= Name ":" KindExpr
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
        kind: ExprAst
    }
  | Extract {
        deduce: DeduceListAst,
        skeleton: CanonicalSkeletonAst
    }
```

### 4.5 Kind expression

```text
KindExpr ::= PipeExpr
```

Kind validity is not checked in v0.1.

Example:

```text
let t: type = int Option Vec
```

constructs:

```text
Let.Simple(
  name = t,
  kind = PipeExpr(...),
  value = PipeExpr(...)
)
```

## 5. Deduce lists

### 5.1 Meaning

A deduce list declares names that act as holes in following syntax.

The parser only recognizes a deduce list in strong binding contexts.

### 5.2 Syntax

```text
DeduceList ::= "<" BinderDeclList? ">"
BinderDeclList ::= BinderDecl ("," BinderDecl)*
BinderDecl ::= Name [ ":" KindExpr ]
```

AST:

```text
DeduceListAst {
    binders: Vec<BinderDeclAst>,
    span: Span
}

BinderDeclAst {
    name: NameAst,
    kind: Option<ExprAst>,
    span: Span
}
```

### 5.3 Non-context rule

Outside strong binding contexts, `<` and `>` are ordinary symbol tokens.

The parser must not globally recognize angle-bracket groups.

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

v0.1 may store capture items as token trees or expressions.

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
    Name [ ":" KindExpr ]
  | DeduceList? CanonicalSkeleton [ ":" KindExpr ]
```

AST:

```text
ParamItemAst ::=
    NameParam {
        name: NameAst,
        kind: Option<ExprAst>,
        span: Span
    }
  | ExtractParam {
        deduce: Option<DeduceListAst>,
        skeleton: CanonicalSkeletonAst,
        kind: Option<ExprAst>,
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
    KindExpr
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
```

They remain names inside expression AST unless appearing in explicitly defined strong contexts.

## 15. Minimum golden cases

Implement tests for at least:

```text
let t: type = int Option Vec

let val: std::int Vec::elements_type = expr

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
