# File: spec/ast-construction-v0.1.md

# AST Construction Rules v0.1

## 0. Scope

This document defines how source tokens are converted into AST in `v0.1`.

It defines syntax recognition and AST construction only.

The v0.1 Raw AST Frontend is completed. The current active stage is
`v0.1.w` — the Raw AST Stability Window. The lexer/parser skeleton and Raw AST
categories documented here are stable by default. Do not use this document as
a basis for broad parser expansion during `v0.1.w`; only richer literal
spellings and local mechanical whole-shape sugar recognition are in scope
unless a hard correctness error is identified. Allowed additions must extend
existing lexer/parser entry points and AST preservation categories; they must
not replace the product/pipe/operator/binding/closure/navigation architecture.

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
&  |  &&  ||
!  @  ~  ^  $  ++  --  ?
+=  -=  *=  /=  &=  |=  <<=  >>=
```

Operator-aware tokenization uses maximal munch: when multiple operator
spellings can start at the same source position, choose the longest spelling.
For example, `<<=`, `&=`, `|=`, `&&`, `||`, `<=`, `++`, and `==` are each
single operator spellings, not shorter operator spellings followed by another
symbol.

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

A form ends only at one of:

- `;`
- `}`
- EOF

A line break (`\n`) is lexical trivia. It is never promoted to a form
separator. The parser must not use line breaks to decide whether two adjacent
expressions belong to one expression/segment or to two forms.

If two adjacent forms are intended, the source must use `;`, `}`, or EOF.

Examples:

```text
a
b
```
→ one expression form. Two atoms `a` and `b` in the same segment.

```text
a;
b
```
→ two forms. The `;` is a hard form boundary.

```text
f
(x)
```
→ one expression. `(x)` is a group in the same segment.

```text
obj
.field
```
→ one `MemberSugar`. Newline is skipped as trivia; `.field` is a suffix.

```text
a
+ b
```
→ one binary operator expression. Newline is trivia; `+` continues the
expression.

```text
a |>
b
```
→ one `PipeExpr`. `|>` is a pipe delimiter; newline before `b` is trivia.

**Negative / diagnostic example:**

```text
let x = (a
+ b)
```

Here `let x = ` is a structurally valid unannotated binding slot whose
initializer starts at `(`. The newline inside `(...)` is trivia; the form
continues through line 2. The parser should consume both lines and diagnose
`UnclosedParen` if the `)` is never found.

## 4. Let statements

### 4.1 Let statement shape

```text
LetStmt ::= OptionalPolicy "let" BindingSlotWithInitializer
```

`OptionalPolicy` is an optional `Expr` written before `let` (see §4.3). A let
binding uses the general binding-slot shape. The initializer is required
for ordinary let bindings. The annotation after `:` is optional and is preserved
as syntax rather than classified as a type annotation.

AST:

```text
LetAst {
    slot: BindingSlotAst,
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

`with` is interpreted only in binding-slot parser states that allow a
block-delimited with clause.

AST:

```text
WithClauseAst {
    kind: WithClauseKind,
    span: Span
}

WithClauseKind ::= Empty | Items { items: Vec<NameAst> } | Error(ErrorAst)
```

`with {}` is an explicit empty with clause. It is distinct from having no with
clause. `with { a, b }` preserves a non-empty syntactic payload. The parser
does not resolve the names, decide same-level binding dependencies, or run
lifetime/dependency checks.

The non-empty payload of `with { ... }` is intentionally limited to source-level
`Name` items. It does not accept symbols, operator names, paths, expressions,
EntityRef syntax, canonical skeletons, or token trees. The Raw AST stores these
names syntactically only; it does not check whether any name exists in an outer
or earlier scope.

`with` without `{` is invalid. `with a, b` is invalid. Trailing commas in
`with { ... }` are rejected. Malformed `with` syntax must not produce
`WithClauseKind::Empty`; only valid source text `with {}` may produce the empty
with-clause kind.

### 4.3 Binding slot

```text
BindingSlot ::=
    OptionalPolicy
    OptionalLet
    OptionalDeduceList
    BindingPattern
    OptionalAnnotation
    OptionalWithClause
    OptionalInitializer

OptionalPolicy ::= (Expr "let")?
```

A policy expression is recognized **only** by the syntactic shape `Expr let`:
the expression appears immediately before the `let` keyword in binding-slot
prefix position. Without the following `let`, the same tokens remain part of the
binding pattern / canonical skeleton (see §4.4).

Context restrictions:

```text
let binding:
    initializer is required for ordinary let syntax
    with { ... } is allowed
    policy: optional `Expr let` prefix; `let` is always present in let forms

function parameter slot:
    initializer is absent
    with { ... } is allowed
    let is allowed but redundant
    <> is allowed per slot
    policy: optional `Expr let` prefix; a written policy REQUIRES the explicit
        `let` anchor. `policy x: T` (no `let`) is an ordinary pattern/skeleton,
        not a policy slot, and is not an error.

function return slot:
    initializer is absent
    with { ... } is rejected
    let is allowed but redundant
    <> is allowed per slot
    policy: optional `Expr let` prefix; same `let`-anchor rule as parameters.
```

Binding pattern:

```text
BindingPattern ::= BinderName | CanonicalSkeleton
BinderName ::= Name | OperatorName
```

AST:

```text
BindingSlotAst {
    policy: Option<ExprAst>,
    has_let: bool,
    deduce: Option<DeduceListAst>,
    pattern: BindingPatternAst,
    annotation: Option<BindingAnnotationAst>,
    with_clause: Option<WithClauseAst>,
    initializer: Option<ExprAst>,
    span: Span
}

BindingPatternAst ::=
    Binder(BinderNameAst)
  | Skeleton(CanonicalSkeletonAst)
  | Error(ErrorAst)

BindingAnnotationAst ::=
    Expr(ExprAst)
  | Compound {
        left: AnnotationTermAst,
        right: ExprAst
    }
  | Error(ErrorAst)

AnnotationTermAst ::=
    Expr(ExprAst)
  | Hole
```

**Binding-slot policy expressions are Raw AST expressions. The parser preserves
the expression shape and does not decide whether it is a valid accessibility
policy, capability condition, visibility object, contract, type-level object,
rank-level object, or semantic predicate. Those checks belong to later
normalization, name resolution, type calculation, and checking phases.**

A policy expression is recognized only by the syntactic shape `Expr let`.
Without the following `let`, the same tokens remain part of the binding pattern /
canonical skeleton. The `let` keyword is the parser-level boundary between the
policy expression and the rest of the binding slot.

A `policy` of `None` means the policy was **not written** at this binding site
(implicit / to be supplied or inferred by a later phase). It does not mean the
binding has "no policy". The AST dump omits the `policy:` line when it is
unwritten.

The `Expr let` policy prefix is accepted in every position that accepts `let`:
top-level let forms, let forms inside closure bodies, parameter binding slots,
return binding slots, and alias-let forms (`LetAliasAst` also carries an
optional `policy`). In parameter and return slots, where `let` may otherwise be
omitted, a written policy still requires the explicit `let` anchor.

```text
let val = expr
let val: annotation = expr
let <val, T> val pattern: T: annotation with {a, b} = expr
```

Operator binder names use the same optional annotation rules as ordinary names:

```text
let +: _: operator = expr
let >: _: operator = expr
let + = expr
```

`<` is accepted as an operator binder. The binding-slot deduce-list recognition
is guarded by a binder-name lookahead: `<` only starts DeduceList parsing when
it is followed by a valid deduce-list binder start (a `Name` token or `>`).

```text
let <: _: operator = less_impl
let < = less_impl
```

Both are parsed as operator binder `<` because `:` and `=` are not valid
deduce-list binder starts. No escaping syntax is required for this case.

The `BindingAnnotationAst::Expr` variant covers a single written annotation
expression, such as `let f: fn = ...`, `let t: type = ...`, or
`let r: rank1 + rank2 = ...`. The annotation is preserved exactly as written.
The parser does not decide whether it denotes a type, rank, custom rank,
concept, region, value object, type object, or future classifier.

Compound annotation syntax preserves an explicit colon inside the annotation
slot:

```text
left_annotation : right_annotation
```

The `BindingAnnotationAst::Compound` variant covers source such as
`let f: _: fn = ...` without interpreting `_` as a type object or `fn` as a
rank. That interpretation is deferred.

For a binding skeleton example:

```text
let <val, T> val pattern: T: annotation = expr
```

the Raw AST reading is:

```text
deduce = [val, T]
pattern = CanonicalSkeleton(val pattern)
annotation = Compound(left = T, right = annotation)
initializer = expr
```

`val` is a canonical-skeleton hole because it is declared in the deduce list;
`pattern` remains a node name. This is one binding skeleton inside one
`BindingSlotAst`, not an outer binder named `val` plus a separate pattern.

v0.1 does not check that annotation names resolve to anything. Annotation
validity is a future semantic pass.

### 4.5 Annotation parsing boundaries

Because binding annotations are preserved expression syntax, the parser must
know where annotation sub-expressions stop in each strong context. The
termination tokens are determined by the surrounding syntactic frame.

**In let binding context**:

- A binding annotation stops at a top-level `:` (which starts a compound
  annotation), `with`, or `=`.
- If the first annotation expression stopped at `:`, the following expression
  is the right side of a compound annotation.
- The right side of a compound annotation stops at a top-level `with` or `=`.

Example:

```text
let f: _: fn = expr
     ^^^^^^---- left annotation term stops at the second `:`
            ^^--- right annotation expression, stops at `=`

let f: _: fn with {deps} = expr
     ^^^^---- left annotation term, containing `_`
          ^---- this `:` starts the compound annotation right side
           ^^--------- right annotation expression, stops at `with`
```

**In `DeduceList BinderDecl` context** (inside `<...>`):

- Binder annotations stop at a top-level `,` or `>`.

**In parameter binding-slot context** (inside closure-head parameter list):

- Binding annotations stop at a top-level `with`, `,`, or `)`.
- A head-clause keyword (`require`, `pre`, `post`, `lifetime pre`,
  `lifetime post`) is also a stop, so the binder/annotation does not absorb a
  following head clause.

**In return binding-slot context** (after `->` in a closure head):

- Binding annotations stop at a top-level `with`, `=>`, or `{`.
- A head-clause keyword (`require`, `pre`, `post`, `lifetime pre`,
  `lifetime post`) is also a stop, so `-> T require c => { ... }` parses `T`
  as the return binder and `require c` as a head clause.
- `where` is a future reserved stop token; it is not an active Phase 3 parser
  stop. `acquire` is an ordinary name and is not a stop.
- `with { ... }` is rejected in return binding slots.

### 4.6 Binding annotation examples

**Positive examples:**

```text
let f: _: fn = expr
```

AST-level reading:

```text
BindingSlot {
    pattern: Binder(f),
    annotation: AnnotationCompound {
        left: AnnotationTermHole("_"),
        right: Expr(Name("fn"))
    },
    initializer: Expr(Name("expr"))
}
```

Deferred semantic reading may decide that `f` has an anonymous type-object
whose kind/rank is the source name `fn`, but the parser does not make that
decision.

```text
let t: type = expr
```

AST-level reading:

```text
BindingSlot {
    pattern: Binder(t),
    annotation: AnnotationExpr(Expr(Name("type"))),
    initializer: Expr(Name("expr"))
}
```

This is a bare annotation containing the expression `type`. v0.1 preserves
that syntax exactly and does not decide whether `type` is semantically valid.

```text
let ns1: namespace = expr
```

AST-level reading:

```text
BindingSlot {
    pattern: Binder(ns1),
    annotation: AnnotationExpr(Expr(Name("namespace"))),
    initializer: Expr(Name("expr"))
}
```

`namespace` is a source name in annotation position, not a lexical keyword
and not a separate declaration form.

```text
let f: _: fn = expr
```

This is an explicit rank-annotation form. The parser produces:

```text
BindingSlot {
    pattern: Binder(f),
    annotation: AnnotationCompound {
        left: AnnotationTermHole("_"),
        right: Expr(Name("fn"))
    },
    initializer: Expr(Name("expr"))
}
```

A bare annotation such as `let f: fn = expr` is preserved as raw syntax
(`AnnotationExpr(Expr(Name("fn")))`) but is **not** function declaration sugar.
The language does not define `let f: fn = ...` as a normative function
declaration spelling.  A function-like declaration must use the explicit
compound annotation form `let f: _: fn = ...` if that is the intended source
shape.

**Alignment:**

```text
let f: _: fn = ...
    |  |  |
    |  |  +-- right annotation expression
    |  +----- left annotation term
    +-------- declared object
```

The explicit compound form has two annotation layers:
`left_annotation : right_annotation`. The bare form has one annotation
expression and must not be lowered or reinterpreted by the parser.

**Negative / non-declaration examples:**

```text
fn f(x) { x }
```

`fn` is an ordinary `Name` token. The parser sees `Name("fn")`, then `Name("f")`,
then a grouped/product parenthesized form, then an unexpected bare `{`. Since the first token is not
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

Binding annotations are raw syntax slots. They may later be interpreted as
type objects, rank objects, custom ranks, concepts, regions, value objects,
type objects, or future classifiers. The parser preserves the source shape and
does not choose among those meanings.

Source names in annotations must be distinguished from semantic classifier
objects:

- The source name `type`, when used in annotation position, is just a
  preserved `Name`.
- The source name `fn`, when used in annotation position, is just a preserved
  `Name`.
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

The parser expects a binding annotation after `:`. If `=` follows immediately,
emit `UnexpectedToken` at `=`, insert `ErrorAst` for the annotation, and
continue parsing the value.

```text
let t: 42 = x
```

`42` is a valid `Literal` which is a valid `PipeExpr` which is a valid
`BindingAnnotationAst::Expr`. Annotation validity is deferred,
so this is syntactically valid in v0.1 (even if semantically nonsensical).

## 5. Deduce lists

### 5.1 Meaning

A deduce list declares names that act as holes in following syntax.

The parser only recognizes a deduce list in strong binding contexts.

### 5.2 Syntax

```text
DeduceList ::= "<" BinderDeclList? ">"
BinderDeclList ::= BinderDecl ("," BinderDecl)*
BinderDecl ::= Name [ ":" AnnotationTerm ]
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
    annotation: Option<AnnotationTermAst>,
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

- a hole name in a product extraction or as a standalone element;
- a path (`Name::Name`) as a skeleton element or inside a product extraction;
- a literal (integer, string) in skeleton position;
- the nesting depth of product extractions;
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
    CanonicalProductExtract
  | CanonicalAtom

CanonicalProductExtract ::= "(" CanonicalProductSlotList? ")"
CanonicalProductSlotList ::= CanonicalProductSlot ("," CanonicalProductSlot)* ","?
CanonicalProductSlot ::= CanonicalSkeleton | <empty>

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
  | ProductExtract {
        elements: Vec<CanonicalProductElementAst>,
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

```text
CanonicalProductElementAst ::=
    Skeleton(CanonicalSkeletonAst)
  | Unit { span: Span }
```

Empty slots produced by leading, doubled, or trailing commas in canonical
product extraction are preserved as `Unit { span }`. They are not omitted, not
wildcards, and not implicit discards.

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
let <head, tail> (head, tail): List::Cons = xs
```

Construct:

```text
Let.Extract
  deduce: [head, tail]
  skeleton:
    Segment
      ProductExtract
        Name(head, Hole)
        Name(tail, Hole)
      Path(List, Cons)
  value: ...
```

```text
let <x> (_, x, _): Triple = t
```

Construct:

```text
Let.Extract
  deduce: [x]
  skeleton:
    Segment
      ProductExtract
        Wildcard
        Name(x, Hole)
        Wildcard
      Name(Triple, NodeName)
```

## 7. Expressions

### 7.1 Expression entry

```text
Expr ::= PipeExpr | ProductExpr
```

AST:

```text
ExprAst ::=
    Pipe(PipeExprAst)
  | Product(ProductExprAst)
  | Error(ErrorAst)
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
SegmentElement ::= OperatorExpr | ProductExpr
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

It records element sequence only. Product elements do not receive source,
insert, or right-target roles in Raw AST.

### 7.1.1 Pipe branch-name shorthand

During `v0.1.w`, the exact local incoming pipe-segment prefix:

```text
PipeTransition Name BraceBody
```

is accepted as a local mechanical sugar:

```text
|> name { ... }
```

It is accepted only as a mechanical shorthand for:

```text
|> (_ name) { ... }
```

The parser preserves the same Raw AST shape as the explicit form: an incoming
segment containing a two-element product head (`_`, `name`) followed by an
in-place closure body. No semantic validation, name resolution, matching,
closure materialization, or lookup is performed.

The branch-name token is any bare `Name` token in that exact local shape,
including the text `_`. For example:

```text
x |> _ { y; }
=> x |> (_ _) { y; }
```

At Raw AST level, both `_` occurrences are just preserved `Name` text. The
parser does not give either occurrence wildcard semantics, unit semantics,
ignored-binding semantics, or pattern semantics.

The shorthand is justified as a narrowly bounded repair for one
otherwise-invalid local shape. In the ordinary pipe / call-composition model,
`x |> name` without a right product is the explicit pipe form of the whitespace
right-call composition `x name`. If a brace body immediately follows and no
shorthand applies, `x |> name { y; }` falls toward continuous right-call
composition into a headless in-place closure, roughly:

```text
x |> name { y; }
=> (x name) { y; }
```

That is not a valid language-model reading. A headless in-place closure is not
a closure that implicitly accepts `unit`: no extraction head means no extracted
input, including no implicit unit input. The shorthand repairs only this local
bad shape by inserting the explicit branch head:

```text
x |> name { y; }
=> x |> (_ name) { y; }
```

The first product-head element `_` is the supplied extraction hole /
unit-side placeholder of the branch head. The second element `name` is the
branch name.

A closure body in incoming pipe position requires a product/extraction head.
The product head may be a segment-level product before an in-place closure
body, the parameter product inside an explicit closure head, or the product
mechanically inserted by the exact `|> name { ... }` shorthand. `x |> { ... }`
is rejected because it is the fully headless in-place closure case. It has no
product/extraction head at all.

`x |> () => { ... }`, `x |> (a) => { ... }`, and
`x |> [] () => { ... }` are ordinary explicit closures with product extraction
heads. They preserve ordinary explicit closure Raw AST. `x |> () { ... }` and
`x |> (a) { ... }` are product-head plus in-place-closure branch forms in
incoming pipe position.

This is not a precedent for a family of branch-arm sugars. The shorthand is
accepted only because the local token shape is finite, local, explicit, and
mechanically equivalent to the already supported explicit form.

The shorthand recognizes only the local incoming segment prefix
`|> name { ... }`. After that local rewrite, any following token sequence is
parsed by the ordinary existing pipe / segment / composition rules. For
example:

```text
x |> name { y; } z
=> x |> (_ name) { y; } z
```

At Raw AST level, the trailing `z` remains ordinary segment material after the
locally rewritten prefix. Any later interpretation as right-call composition
or an additional normalized call belongs to a future normalization stage, not
to the Raw AST parser. The trailing `z` is not part of a larger branch-arm
sugar.

The shorthand does not generalize. The parser must not treat any of the
following as this shorthand:

```text
|> name expr
|> name
|> name => ...
|> name (...)
|> name [...]
|> name other { ... }
|> a::b { ... }
|> + { ... }
|> (name) { ... }
|> _ name { ... }
|> name1 name2 { ... }
```

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
        args: ProductExprAst,
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
  | InPlaceClosureAst
  | ExplicitClosureAst
```

`InPlaceClosureAst` is a bare `{ ... }` atom. It is closure AST, not a normal
block expression, and has no capture clause, parameter clause, return clause,
or head clauses.

`ExplicitClosureAst` is a `FnHeadPrefix => BodyBlock` atom.

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
  | BracketCallSugar { object, operator: OperatorName "[]", args: ProductExprAst }
  | Error(...)
```

### 8.2 Group and product form

```text
Group ::= "(" PipeExpr ")"

ProductForm ::=
    "()"
  | "(" ProductSlot "," ProductSlotList? ")"
  | "(" "," ProductSlotList? ")"

ProductSlotList ::= ProductSlot ("," ProductSlot)* ","?
ProductSlot ::= PipeExpr | <empty>
```

A non-empty parenthesized form without a top-level comma is always Group.
The empty parenthesized form `()` is the zero-element Product.
A non-empty parenthesized form with at least one top-level comma is Product.

A group is valid only if its contents do not contain a top-level comma.

A parenthesized form with top-level commas is a product form, not an ArgPack.

In expression context, `(a, b)` is product construction:

```text
ExprKind::Product(ProductExprAst { elements: [a, b], span })
```

In binding / extraction context, `(a, b)` is product extraction:

```text
BindingPatternAst::Product(ProductExtractAst { elements: [a, b], span })
```

The same surface product form does not change syntax shape. Later phases may
interpret expression-context products as construction and extraction-context
products as destructuring / matching, but the parser performs no constructible,
destructible, layout, or type validation.

### 8.3 Suffix folding

After parsing `AtomBase`, repeatedly fold suffixes. Parser phase 4 extends this
folding over `OperatorExprAst` so postfix operator results can continue through
the same suffix loop.

Suffixes:

```text
:: NavComponent
. Selector
.. Selector ProductExpr
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
```

Navigation components for this phase:

```text
NavComponentAst ::=
    Text(NameAst)
  | Operator(OperatorNameAst)
  | Group(Box<ExprAst>)
  | Error(ErrorAst)
```

`Operator(OperatorNameAst)` is valid only as the innermost navigation
component. It is not valid after `.`, after `..`, or as an outer component
after `::`.

A numeric token (`IntLiteral`) in atom-base position produces a numeric literal
atom (`IntLiteral`). A float token (`FloatLiteral`) produces a float literal atom
(`FloatLiteral`).

Numeric selectors have been removed. Member selectors accept only `Name`.
Numeric navigation components have been removed. Use bracket form for
projection (`pack[0]` instead of `pack.0`).

`1.2` is a **float literal** (`FloatLiteral` token), not member sugar:

```text
1.2 ↦ FloatLiteral("1.2")
```

`1.x` remains valid member sugar (`IntLiteral` object, `TextName` selector).

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

NavComponent ::= Name | OperatorName

NavOuterComponent ::= Name | Group
```

Operator names may only be innermost navigation components. Valid shapes
include `+::int::std` and `<<::bit::std`. Invalid shapes include `x::+`,
`x::int::+`, and `+::x::+`.

Parenthesized right-side scope expressions after `::` are allowed:

```text
xxx::(int Vec::std)
```

The grouped expression is stored as a grouped outer navigation component.

The innermost navigation component must be a syntactic symbol component:
`Name` or `OperatorName`. A grouped expression is valid only as
an outer navigation component after `::`; it represents a scope-producing
expression, not a selected symbol. A parenthesized form used as the innermost
component, such as `(int Vec::std)::ns`, is invalid and emits
`InvalidNavComponent`. The grouped expression is replaced by a local error
component and the remaining outer components are preserved.

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
        args: ProductExprAst,
        span: Span
    }
  | BracketCallSugar {
        object: OperatorExprAst,
        operator: OperatorName,   // spelling "[]"
        args: ProductExprAst,
        span: Span
    }
```

Operator syntax is preserved as AST sugar at the `OperatorExprAst` layer. The
parser must not lower it into ordinary calls in v0.1. The operator-level
`NavPath`, `MemberSugar`, `DoubleDotSugar`, and `BracketCallSugar` variants
exist only so postfix operator results can continue through suffix folding, for
example `obj!.field`, `obj.field?`, `obj..map(a)!`, and `obj![a]`.

Examples:

```text
obj!    => postfix OperatorSugarAst
a + b   => binary OperatorSugarAst
-x      => prefix OperatorSugarAst (Raw AST preservation only)
```

Prefix `-x` is not a negative literal; the lexer emits `-` and `x`
separately. The `Prefix` fixity in `OperatorSugar` is a Raw AST surface
marker. In v0.1 it is used only for the prefix-negative shape
(`operator="-"`, `fixity=Prefix`). It is not an overloadable prefix operator
declaration and must not be lowered as an ordinary operator call.
Normalization must special-case this shape and rewrite it to
`()zero::(x |> type) - x`.

Comparison, equality, and equals-suffixed operator chains are
non-associative in this phase. The parser diagnoses:

```text
chained non-associative operator requires explicit grouping
```

for ungrouped syntax such as `a < b < c`, `a == b == c`, `a += b += c`, and
`a &= b &= c`.

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

The selector is `Text(NameAst)` for textual names. Numeric selectors
have been removed.

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
    args: ProductExprAst,
    span
}
```

Parser constraints:

- `..` must be followed by a valid selector token (`Name` or `IntLiteral`).
- The selector must be followed by a product form.

Missing product examples:

```text
obj..member
obj..42
obj..(method)
```

**Negative / diagnostic examples:**

```text
obj..42
```

`..` is followed by `IntLiteral("42")`, which is not a valid selector because
numeric selectors have been removed. Emit `ExpectedNameAfterDoubleDot` with the
primary span on `..` or on the invalid selector boundary, according to the parser
diagnostic convention. Do not construct a `DoubleDotSugar` node.

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

### 8.7 Bracket call sugar

Input:

```text
obj[args...]
obj[]
```

AST:

```text
BracketCallSugar {
    object,
    operator: OperatorNameAst,   // spelling "[]"
    args: ProductExprAst,
    span
}
```

`obj[args...]` is parsed as source-preserving bracket-call operator sugar. It
participates in the suffix chain like member sugar, double-dot method sugar, and
postfix operators, and is left-associative: `obj[a][b]` is a bracket call on the
result of `obj[a]`. It exists at both the atom layer and the operator-expr layer
(the latter for chaining after a postfix operator, e.g. `obj![a]` is
`(obj!)[a]`).

It is conceptually normalized later as:

```text
obj[args...] ↦ obj |> [](args...) ↦ (obj, args...) |> []
obj[]        ↦ obj |> []()        ↦ (obj) |> []
```

using the general call/pipe association rule:

```text
(args1...) |> f(args2...) ↦ (args1..., args2...) |> f
```

The operator spelling `[]` is an ordinary operator spelling with special
surface association. It has no global builtin implementation and does **not**
imply indexing, slicing, container access, bounds checking, address
calculation, mutation, lvalue/reference behavior, or any special runtime
operation. `[]` is bindable / aliasable / referable wherever operator names are
allowed (operator binder, alias binder, entity-ref innermost component); it is
recognized contextually as the paired empty brackets `[]` rather than a single
lexer token.

The parser preserves the bracket-call source shape and does not lower it.
Semantic validity belongs to later normalization, operator resolution, type
calculation, and checking.

`obj[]` is valid and represents a bracket-call with an empty argument list.
`[` in expression-suffix position begins a bracket call; `[` at closure-head
prefix position begins a capture clause. The distinction is purely contextual.

## 9. Product form

### 9.1 Syntax

```text
ProductExpr ::= "(" ProductSlotList? ")"
BracketProductExpr ::= "[" ProductSlotList? "]"
ProductSlotList ::= ProductSlot ("," ProductSlot)* ","?
ProductSlot ::= PipeExpr | <empty>
```

AST:

```text
ProductExprAst {
    elements: Vec<ProductElementAst>,
    span: Span
}

ProductElementAst ::=
    Expr(ExprAst)
  | Unit { span: Span }
```

A parenthesized form with top-level commas is a product form. In expression
context, it is preserved as `ExprKind::Product(ProductExprAst)` and later phases
may interpret it as product construction.

A parenthesized form with no top-level comma remains a group expression:

```text
(a)       -> Group(a)
(a, b)    -> ProductExpr([a, b])
(a, b, c) -> ProductExpr([a, b, c])
()        -> ProductExpr([])
```

Raw AST does not create `ExprKind::Unit` for empty comma slots. Empty elements
from leading, doubled, or trailing commas are preserved as unit product
elements. They are not omitted, not wildcards, and not implicit discards.

```text
(, a)    -> ProductExpr([Unit, a])
(a,, b)  -> ProductExpr([a, Unit, b])
(a,)     -> ProductExpr([a, Unit])
```

### 9.2 Extraction-side product form

In binding / extraction context, the same surface product form is parsed as
product extraction:

```text
ProductExtract ::= "(" ProductExtractSlotList? ")"
ProductExtractSlotList ::= ProductExtractSlot ("," ProductExtractSlot)* ","?
ProductExtractSlot ::= BindingSlot | <empty>
```

AST:

```text
ProductExtractAst {
    elements: Vec<ProductExtractElementAst>,
    span: Span
}

ProductExtractElementAst ::=
    Slot(BindingSlotAst)
  | Unit { span: Span }
```

Empty product positions produced by leading, doubled, or trailing commas are
preserved as unit product extraction elements. `_` is the explicit wildcard /
consuming pattern. A comma-created unit position matches only unit.

Product extraction elements inherit the binding-slot context of the surrounding
position. Closure parameter products parse elements with parameter-slot
restrictions. Return products parse elements with return-slot restrictions, so
`with { ... }` remains rejected inside `-> (...)`. Top-level `let` product
extraction currently parses elements with parameter-like binding-slot
restrictions because the outer `let` supplies the initializer context.

Examples:

```text
let (a2, b2) = (a1, b1)
```

Raw AST shape:

```text
Let {
    pattern: ProductExtract(a2, b2),
    initializer: ProductExpr(a1, b1)
}
```

The parser does not decide whether a product is constructible, destructible,
layout-compatible, type-compatible, or otherwise semantically admissible.

### 9.3 Segment interaction

Product forms may appear in pipe segments as product elements, but no
role enum exists. The parser does not assign source, insert, or right-target
roles.

Examples:

```text
(a, b)
```

produces a standalone product expression.

```text
f (a, b)
```

preserves `Name("f")` followed by a product element. Whether that combination
normalizes to a call/application shape is a later Normalized AST concern.

```text
x |> f (a, b)
```

preserves the pipe segmentation and product form without assigning an insert
role to `(a, b)`.

## 10. Closure AST

### 10.1 Closure categories

```text
ClosureAst ::=
    InPlaceClosureAst
  | ExplicitClosureAst
```

### 10.2 In-place closure

```text
InPlaceClosureAst ::= BodyBlock
```

A bare `{ ... }` in atom position is an in-place closure. It has no capture
clause, no parameter clause, no return clause, and no head clauses. It is the
Raw AST representation of a control-flow-embedding closure block.

Having no extraction head is distinct from having a unit extraction pattern.
A headless in-place closure does not implicitly accept unit input; it has no
extracted input, including no implicit unit input.

`{ ... }` is not a normal block expression; it always produces
`ClosureAst::InPlace`.

### 10.3 Explicit closure

```text
ExplicitClosureAst ::= FnHeadPrefix "=>" BodyBlock
```

A headed closure must use `=>`. Forms such as `[](){}`, `[x]{}`, `(){}`, and
`pre c {}` are invalid headed closures without `=>` and produce
`InvalidClosureHead` diagnostics.

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

Inside `{ ... }`, form boundaries are `;`, `}`, and EOF. Newlines are
trivia everywhere; body blocks use the same hard-only boundary rule as the
top level. `{ x \n y }` parses as a single form containing a segment with
two atoms `x y`, not as two separate forms. Semicolon-separated forms split
normally: `{ x; y; }` contains two body forms.

## 11. Closure head

### 11.1 Full order

```text
FnHeadPrefix ::=
    DeduceList?
    CaptureClause?
    ParamClause?
    FnItemTraitClause?
    ReturnClause?
    HeadClause*

// Future reserved, not implemented:
//   WhereClause?
```

`HeadClause` covers the active head clauses `require`/`pre`/`post`/`lifetime
pre`/`lifetime post` (see §11.8). `acquire` is no longer a reserved head-clause
position; only `where` remains future reserved.

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
ParamClause ::= ProductExtract
```

This is a closure-head extraction context. The whole parenthesized parameter
clause is one product extraction form, not a list of independent parameter
slots and not an ArgPack.

AST:

```text
ParamClauseAst {
    extract: ProductExtractAst,
    span: Span
}
```

### 11.4 Param item

```text
ParamItem ::= BindingSlotWithoutInitializer
```

`ParamItem` entries are the elements of `ParamClauseAst.extract`.

Examples:

```text
<T, U>(let <a, b> (a, b): T, let: U) => {}
<T, U>(<a, b> (a, b): T, let: U) => {}
```

Both forms parse the closure parameters as one `ProductExtractAst`. The closure
head is already a binding / extraction context, so an element without a policy
may omit the local `let` anchor. If a policy expression is written for an
element, the policy still requires the `Expr let` anchor.

Function parameter slots reuse the general binding-slot grammar:

```text
val
val: annotation
let val: annotation with {}
<val, T> val pattern: T: annotation with {}
val: annotation with {a, b}
```

`with { ... }` is preserved syntactically. The parser does not decide whether
names inside the with block are valid, same-level, earlier parameters, or
resolvable dependencies. Those checks belong to name resolution, type
calculation, and later ownership/lifetime checking.

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
ReturnClause ::= "->" BindingSlotWithoutInitializer
```

AST:

```text
ReturnClauseAst {
    slot: BindingSlotAst,
    span: Span
}
```

Return clauses are binding sites, not traditional return-type slots. The form:

```text
-> result
```

binds a named return slot named `result` with no explicit annotation. The form:

```text
-> _: annotation
```

is the anonymous return slot constrained by the written annotation.

Valid return binding-slot examples include:

```text
-> _
-> _: annotation
-> result
-> result: annotation
-> let result: annotation
-> <T> result pattern: annotation
```

Return slots reject `with { ... }` in this phase. That restriction is
contextual parser structure, not a semantic dependency check.

### 11.7 Where clause (future reserved)

> **Not implemented in Phase 3.1.** `where` is a reserved closure-head
> position.
> It remains an ordinary name outside a future where-parser state.
> Concrete `where` syntax is deferred until a dedicated closure-clause parser
> and logical-constraint grammar exist.

### 11.8 Head clauses (`require` / `pre` / `post` / `lifetime pre` / `lifetime post`)

The closure/function head may carry a tail of source-preserving clauses after
the deduce, capture, parameter, fn-item-trait, and return clauses:

```text
FnHeadPrefix ::=
    DeduceList?
    CaptureClause?
    ParamClause?
    FnItemTraitClause?
    ReturnClause?
    HeadClause*

HeadClause ::=
    RequireClause
  | PreClause
  | PostClause
  | LifetimePreClause
  | LifetimePostClause

RequireClause      ::= "require" Expr
PreClause          ::= "pre" Expr
PostClause         ::= "post" Expr
LifetimePreClause  ::= "lifetime" "pre" Expr
LifetimePostClause ::= "lifetime" "post" Expr
```

Each clause holds exactly one `ExprAst`. The clause expression stops at the
next head-clause keyword (`require`, `pre`, `post`, `lifetime pre`,
`lifetime post`), at the closure body boundary (`=>` or `{`), or at a form
boundary / EOF. A clause is never a list: a top-level comma inside a clause
slot is diagnosed (`TopLevelComma`) because the slot then holds more than one
expression-shaped part rather than exactly one expression.

`=> BodyBlock` has priority as the closure head/body delimiter. In closure-head
context it terminates the head and starts the body, in the same way that
`{ ... }` confirms the structure of `with { ... }`. A head clause expression
therefore stops *before* that outer `=> BodyBlock`. For example:

```text
require x => { x }
```

parses as a clause-only head `Require(x)` plus body `{ x }`. It is **not**
reinterpreted as `Require(x => { x })`. A bare `x => { x }` is not a valid
expression in any case, because expression-position closure literals require
parenthesized parameters (e.g. `(x) => { x }`); closure parameters cannot drop
the parentheses. If a future expression-position closure literal appears inside
a head clause expression, it must satisfy the ordinary expression grammar and
closure-head requirements.

`lifetime pre` and `lifetime post` are two-token clause heads recognized only
in the head clause-tail context. A bare `lifetime` not followed by `pre` or
`post` is not a clause head and remains an ordinary name. These two-token heads
keep their space-separated spelling; underscore forms such as `lifetime_pre`
are not used.

Head clauses are active only in the closure/function head clause-tail context.
Outside that context the words `require`, `pre`, `post`, and `lifetime` remain
ordinary names. A clause-headed form is recognized as a closure only when a
closure body boundary (`=>` or `{`) follows; otherwise the speculative head
parse is discarded and the words are reparsed as ordinary names. There is no
special rule rejecting clause-only heads: whether the source reduces to a
closure expression is decided by ordinary closure-head recognition.

These clauses are source-preserving only. **Clause expressions are Raw AST
expressions. The parser preserves the expression shape and does not decide
whether the expression is a valid contract, valid lifetime condition, valid
resource condition, type-level object, rank-level object, or semantic
predicate. Those decisions belong to later type calculation, name resolution,
and checking phases.** The AST node names (`Require`, `Pre`, `Post`,
`LifetimePre`, `LifetimePost`) record only the source clause keyword; they do
not imply semantic validation.

`acquire` is no longer a reserved head-clause position; the earlier
`acquire A` direction is replaced by the explicit `pre Expr` / `post Expr`
clauses above. `acquire` is an ordinary name. `where` remains a reserved,
inactive closure-head position (§11.7).

### 11.9 Closure / with / in-place body disambiguation

The parser uses fixed finite lookahead, not semantic backtracking.

A `{ ... }` body-like form is assigned by its immediate syntactic owner:

1. In a binding slot, `with { ... }` is parsed as `WithClauseAst`.
   The body block is consumed by the binding parser and is not offered to the
   expression atom parser.

2. In expression atom position, a bare `{ ... }` with no preceding committed
   closure head is parsed as `InPlaceClosureAst`.

3. A successfully parsed `FnHeadPrefix` followed by `=> { ... }` is parsed as
   `ExplicitClosureAst`.

4. A successfully parsed `FnHeadPrefix` followed directly by `{ ... }` without
   `=>` is invalid and produces `InvalidClosureHead`; it is not reinterpreted as
   an in-place closure.

Failed speculative `FnHeadPrefix` lookahead restores the cursor and drops gated
diagnostics. Committed malformed closure-head parsing keeps diagnostics. Nested
diagnostic gates must preserve outer gated diagnostics until the outer gate is
explicitly kept or dropped.

The following are not recognized as successful closure heads:

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

**Headed closures without `=>`:**
Forms such as `[](){}`, `[x]{ body }`, `(){ body }`, and `pre c { body }`
produce `InvalidClosureHead` diagnostics.

v0.1 does **not** support bare-name parameter closure sugar. Valid minimal
forms remain `() => {}` and `{ }`, and `(x) => {}` where the `()` is a
`ParamClause`. `(x) => {}` with a single param inside parens is the simplest
parametrized explicit closure form.

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
    Product
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

The parser sees `Name("obj")`, then `Name("match")`, then `Group(a)`,
then an unexpected bare `{`. The high-level expression prefix is:

```text
PipeExpr
  Segment
    Atom Name(obj)
    Atom Name(match)
    Group
```

No special match-arm relationship exists at the parser level. Bare `{}` does
not become a match arm; match-style expressions that use closure arms must use
valid headed closure syntax inside the product form.

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
- expected product after `.. Name`
- unclosed `(`
- unclosed `[`
- unclosed `{`
- top-level comma outside a product form
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
AliasBinding ::= OptionalPolicy "let" AliasBinder "===" EntityRef

AliasBinder ::= Name | OperatorName

EntityRef ::= EntityNavigation

EntityNavigation ::= EntityComponent ("::" EntityOuterComponent)*

EntityComponent ::= Name | OperatorName

EntityOuterComponent ::= Name | Group
```

EntityRef navigation order is inner-to-outer. The leftmost component is the
innermost selected symbol, and the rightmost component is the outermost scope
component. Raw AST preserves source-order navigation components and performs no
lookup. Operator names are valid only as innermost entity-reference components
unless a future design explicitly allows operator-named scopes.

The innermost component must be a syntactic symbol component (`Name` or
`OperatorName`). A grouped expression is valid only as an
outer navigation component after `::`, matching ordinary Raw AST navigation:
`xxx::(int Vec::std)` is valid, while a grouped expression as the innermost
component (`(int Vec::std)::ns`) emits `InvalidEntityRef`.

`===` is a structural delimiter token (`Symbol::TripleEqual`), not an
expression operator. It is not available as `OperatorName`.

`<` is not accepted as an alias binder; it goes to extract-let.

### 16.3 Dispatch

An optional policy expression (`Expr let`, see §4.3) may precede `let`. It is
parsed before `let` and carried onto the alias (`LetAliasAst.policy`); alias
dispatch is unaffected by its presence.

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
    policy: Option<ExprAst>,
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

Outer navigation components after `::` may be `Name` or a
parenthesized grouped scope expression (`NavComponentAst::Group`), shared with
ordinary navigation parsing. A grouped expression used as the innermost
component (such as `(int Vec::std)::ns`, or any parenthesized form like
`(a, b)`) emits `InvalidEntityRef` with "grouped expression cannot be an
innermost navigation component". This is distinct from `ExpectedAliasTarget`,
which is reserved for an absent RHS or a token that cannot begin an entity
reference at all.

After completing the entity reference, the parser checks for residual
expression tokens. If the current position is not a form boundary (EOF,
semicolon, or right brace), the parser emits
`UnexpectedAliasRhsExpression` and recovers to the form boundary.

### 16.6 Diagnostics

| Code                           | Trigger                                                                         |
| ------------------------------ | ------------------------------------------------------------------------------- |
| `ExpectedAliasTarget`          | `===` is followed by an absent RHS or a token that cannot begin an `EntityRef` at all.  |
| `InvalidEntityRef`             | Malformed entity reference (e.g., operator as an outer component, grouped expression as the innermost component, dangling `::`). |
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
| §4.6 BindingAnnotation               | `let t: = x` after `let`                    | `ExpectedBindingAnnotation` at `=`                                                       |
| §4.6 BindingAnnotation               | `let t: 42 = x` (syntactically valid)       | No diagnostic (valid syntax)                                                             |
| §4.6 BindingAnnotation               | `fn f(x) { x }` — not a FnDecl              | No diagnostic (ordinary expr; adjacencies may vary)                                      |
| §4.6 BindingAnnotation               | `type T = expr` — not a TypeDecl            | `=` may produce `UnexpectedToken`                                                        |
| §4.6 BindingAnnotation               | `namespace ns = expr` — not a NamespaceDecl | `=` may produce `UnexpectedToken`                                                        |
| §5.3 Expression-context `<`/`>`      | `a < b > c`                                 | `ChainedNonAssociativeOperator` on the ungrouped non-associative chain                   |
| §7.2 Pipe split                      | `\|> f` at form start                       | `UnexpectedToken` at `\|>`                                                               |
| §7.2 Pipe split                      | `x \|> \|> g` (empty middle)                | Diagnostic on empty segment                                                              |
| §8.2 Product form                    | `let x: type = (a, b)` at expr top          | No diagnostic; product construction                                                      |
| §8.5 Member `.`                      | `obj.42`                                    | `ExpectedNameAfterDot` (numeric selectors removed)                                       |
| §8.5 Member `.`                      | `obj.(field)`                               | `ExpectedNameAfterDot`                                                                   |
| §8.5 Member `.`                      | `obj."field"`                               | `ExpectedNameAfterDot`                                                                   |
| §8.6 Double-dot `..`                 | `obj..42`                                   | `ExpectedNameAfterDoubleDot` (numeric selectors removed)                                 |
| §8.6 Double-dot `..`                 | `obj..1`                                    | `ExpectedNameAfterDoubleDot` (numeric selectors removed)                                 |
| §8.6 Double-dot `..`                 | `obj..(method)`                             | `ExpectedNameAfterDoubleDot`                                                             |
| §8.6 Double-dot `..`                 | `obj..+`                                    | `ExpectedNameAfterDoubleDot` (operator selectors are valid only after `::`)              |
| §11.9 Closure lookahead              | `x => { }` rejected as non-closure-head     | `UnexpectedToken` at `=>`                                                                |
| §12 Match non-special                | `match obj` at form start                   | No diagnostic (name, not syntax)                                                         |
| §12 Match non-special                | `obj match (a) { }`                         | No diagnostic (name + group + closure)                                                   |
