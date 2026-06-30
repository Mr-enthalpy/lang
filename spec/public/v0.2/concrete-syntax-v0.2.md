# Concrete Syntax v0.2

## 1. Scope

This document defines the public concrete syntax of the current v0.2 Raw AST
frontend. It covers:

```text
token stream → Raw AST + syntax diagnostics
```

This document assumes the token categories defined in
`spec/public/v0.2/lexical-syntax-v0.2.md`. Token-level spelling rules (names, literals,
symbols, operators, trivia, invalid material) are defined there.

This document does not define:

- Normalized AST or Raw AST → Normalized AST lowering
- semantic interpretation
- name resolution or type checking
- operator lookup or overload resolution
- alias target resolution
- canonical matching or closure materialization
- ownership, lifetime, NLL, or drop insertion
- code generation or interpretation

## 2. Core parser invariants

The following invariants govern the concrete syntax parser:

- **Parser owns shape, semantics owns meaning.** The parser constructs and
  preserves Raw AST shape. It does not decide what any AST fragment
  semantically represents.
- **Raw AST is source-preserving and non-desugared.** Operator sugar, member
  sugar, navigation paths, product forms, and extraction skeletons are
  preserved as-is. No desugaring is performed by the parser.
- **The parser proceeds left-to-right.** It does not go back to reinterpret
  meaning. Contextual parsing is allowed only where explicitly specified.
- **No semantic backtracking.** The parser never revises an earlier AST
  decision based on later tokens or semantic information.
- **No keyword-classified lexer.** The lexer produces `Name` tokens for all
  words. The parser recognizes names structurally only in specified contexts.
- **Syntax acceptance does not imply semantic validity.** A syntactically
  accepted form may be semantically inadmissible in a later pass. The parser
  does not enforce semantic rules.

## 3. Program and form boundaries

A program is a sequence of forms. A form ends only at one of:

- semicolon (`;`)
- right brace (`}`)
- end of file (EOF)

A newline is lexical trivia and is never promoted to a form separator. This
is a public syntax guarantee — the language has a hard-only form boundary.

**Examples:**

```text
a
b
```

One expression form. Two atoms `a` and `b` are in the same segment.

```text
a;
b
```

Two forms. The semicolon is a hard form boundary.

```text
f
(x)
```

One expression form. `(x)` is a group in the same segment.

```text
let x = expr1;
let y = expr2
```

Two let forms separated by a semicolon.

## 4. Form selection

A form is a let path either when it starts with `Name("let")`, or when a
leading expression is followed by a top-level `Name("let")`, which is
recognized as a policy-prefixed let form. Alias-let is recognized inside that
let path by a following `===` delimiter. Expression forms cover all other
non-trivia token sequences.

The parser produces four form categories at the Raw AST level:

| Raw AST shape | Recognition rule |
|---|---|
| `FormAst::Let(LetAst)` | The form enters the let path and alias-let recognition does not match. |
| `FormAst::AliasLet(LetAliasAst)` | The form enters the let path and alias recognition (`===` delimiter) succeeds. |
| `FormAst::Expr(ExprAst)` | The first non-trivia token starts neither a let path nor a policy-prefixed let path. |
| `FormAst::ReturnEvent(ReturnEventAst)` | The form contains a return terminal event with a value expression and an unresolved return target. Recognized contextually by the parser in return terminal form positions. |
| `FormAst::Error(ErrorAst)` | The form cannot be recovered to any valid structure. |

The parser preserves these shapes as Raw AST. It does not classify forms as
declarations, statements, or semantic categories.

## 5. Let forms

An ordinary let form has the shape:

```text
OptionalPolicy "let" BindingSlot "=" Expr
```

Where `OptionalPolicy` is an optional expression written immediately before
`let`. The policy is recognized only by the syntactic shape `Expr let`.
Without the following `let`, the same tokens remain part of the binding
pattern or canonical skeleton. A `policy` of `None` means the policy was
unwritten (implicit), not that the binding has "no policy."

The let body includes:

- an optional deduce list (`<...>`)
- a binding pattern (binder name, product extraction, or canonical skeleton)
- an optional annotation (`: expr` or `: term : expr`)
- an optional `with { ... }` clause
- an initializer expression after `=`

The parser preserves all of these as raw syntax. It does not check:

- whether the policy expression is a valid accessibility or visibility policy
- whether the annotation denotes a valid type, rank, or classifier
- whether the deduce list declares valid holes
- whether the binding pattern is admissible
- whether the initializer expression is well-typed

## 6. Alias-let forms

An alias-let form has the shape:

```text
OptionalPolicy "let" AliasBinder "===" EntityRef
```

Alias binding is a form-level construct. It may appear wherever a form may
appear (source-file form level, inside closure bodies). It may not appear
inside expressions, product extraction elements, parameter clauses, return
clauses, annotations, head-clause expressions, or ordinary binding slots.

The alias binder may be a `Name` or an `OperatorName`. The operator binder
is accepted wherever operator-name positions are syntactically valid.

`EntityRef` is parsed only inside the alias-let right-hand side. It is not a
general expression parser mode. The alias RHS boundary is hard-only: `;`,
`}`, or EOF. Residual tokens before a hard boundary produce a diagnostic.

In alias-let dispatch, `===` is a structural delimiter
(`Symbol::TripleEqual`). In ordinary expression context, the parser may
reinterpret the same spelling as an operator expression spelling for source
preservation. The alias delimiter use does not imply alias semantics,
operator lookup, or equality semantics.

`with { ... }` is not accepted in alias binding.

The parser preserves alias shape but does not perform target resolution,
name lookup, operator identity validation, namespace resolution, or alias
semantics.

## 7. Binding slots

The binding slot is the reusable parser shape for let bindings, parameter
slots, and return slots. It preserves:

| Field | Presence by context |
|---|---|
| `policy: Option<ExprAst>` | Optional in all let/param/return positions; recognized only by `Expr let` shape |
| `has_let: bool` | Required in let forms; optional/redundant in param/return slots |
| `deduce: Option<DeduceListAst>` | Optional per slot in strong binding contexts |
| `pattern: BindingPatternAst` | Required |
| `annotation: Option<BindingAnnotationAst>` | Optional |
| `with_clause: Option<WithClauseAst>` | Optional in let and param slots; rejected in return slots |
| `initializer: Option<ExprAst>` | Required in let contexts; absent in param/return contexts |

Context restrictions:

- **Ordinary let form:** initializer required; `let` is always present.
- **Parameter slot:** initializer absent; `with { ... }` allowed; `let` allowed
  but redundant; `<>` allowed per slot.
- **Return slot:** initializer absent; `with { ... }` rejected; `let` allowed
  but redundant; `<>` allowed per slot.

## 8. Binding patterns

The parser recognizes four binding pattern variants:

```text
BindingPatternAst ::=
    Binder(BinderNameAst)
  | Product(ProductExtractAst)
  | Skeleton(CanonicalSkeletonAst)
  | Error(ErrorAst)
```

`Binder` covers simple text names (`NameAst`) and operator names
(`OperatorNameAst`). `Product` is product extraction in binding/extraction
context — parenthesized forms with top-level commas. `Skeleton` is a
canonical skeleton pattern (see §21). `Error` is a recovery marker.

`_` is a `Name` token at the lexical level. The canonical skeleton grammar
may mark it syntactically as a hole only in extraction contexts where that
grammar applies. The parser attaches no wildcard, unit, ignored-binding, or
pattern semantics to `_` at the lexer level.

## 9. With clauses

A with clause appears as:

```text
with {}
```

or:

```text
with { name1, name2 }
```

`with {}` is an explicit empty with clause. It is distinct from the absence
of a with clause (which produces `None` in `BindingSlotAst.with_clause`).

The non-empty payload accepts only comma-separated `Name` items. It does not
accept symbols, operator names, paths, expressions, EntityRef syntax,
canonical skeletons, or token trees. Trailing commas in `with { ... }` are
rejected. Malformed `with` syntax must not be recognized as `with {}`.

The parser does not resolve with-clause names and does not run dependency,
lifetime, ownership, or ordering analysis.

## 10. Expressions: outer architecture

The expression parser is organized around `|>` as the outer skeleton:

```text
Expr
  → PipeExpr
  → Segment*
  → SegmentElement*
```

Core invariants:

- `|>` separates pipe segments.
- Each pipe segment is parsed locally.
- Operators bind tighter than whitespace auto-pipe and `|>`.
- Operator precedence is segment-local.
- Product forms are preserved as `ProductExprAst`.
- Whitespace auto-pipe/right-call composition is not traditional
  `f(args)` call syntax.
- There is no traditional `f(args)` call syntax at the parser level.
- Parenthesized top-level-comma forms are product forms, not argument lists.

## 11. Pipe expressions and segments

A pipe expression splits at `|>` into segments:

```text
PipeExpr ::= Segment ("|>" Segment)*
```

Each segment has a `has_incoming` flag indicating whether a prior segment
exists. The first segment has `has_incoming = false`.

A segment consists of operator expressions and product elements:

```text
SegmentElement ::= OperatorExpr | Product
```

### 11.1 Pipe branch-name shorthand

The exact local incoming pipe-segment prefix:

```text
|> name { ... }
```

is accepted as a mechanical shorthand for:

```text
|> (_ name) { ... }
```

The parser preserves the same Raw AST shape as the explicit form: an incoming
segment with a two-element product head (`_`, `name`) followed by an in-place
closure body.

The branch-name token may be `_` because `_` is a bare `Name` token in this
shape. No wildcard, unit, ignored-binding, or pattern semantics attach to
either `_` or `name` at the parser level.

This is not a precedent for branch-arm sugar families. The shorthand is
accepted only because the local token shape is finite, explicit, and
mechanically equivalent to the already-supported explicit form. Following
token sequences remain ordinary segment material.

```text
x |> { ... }
```

is parsed as an incoming headless in-place closure segment element. It emits
`InvalidClosureHead` because incoming pipe branch bodies require an explicit
extraction head. The headless in-place closure is still preserved as a closure
AST segment element with a diagnostic.

## 12. Product forms

A parenthesized form with at least one top-level comma is a product form.
In ordinary expression segment position, the presence of a top-level comma
distinguishes product construction from grouping. In argument-product
contexts such as double-dot payloads (`obj..m(args)`) and bracket-call
payloads (`obj[args]`), the delimiter payload is parsed as a `ProductExprAst`
even when no top-level comma appears.

In expression context, it is product construction (`ProductExprAst`):

```text
(a, b)
```

In binding/extraction context, it is product extraction (`ProductExtractAst`):

```text
let (a, b) = ...
```

A parenthesized form with no top-level comma is a grouping expression, not a
product form:

```text
(a)  → group
```

Leading, doubled, or trailing comma positions produce explicit `Unit` product
elements. These are not omitted, not wildcards, and not implicit discards:

```text
(, a)    → ProductExpr([Unit, a])
(a,, b)  → ProductExpr([a, Unit, b])
(a,)     → ProductExpr([a, Unit])
```

There are no `ArgPack`, `SourcePack`, `InsertPack`, or `RightTargetSubsegment`
role enumerations in Raw AST. The parser does not assign call/application
roles to product elements.

## 13. Atom forms

The parser recognizes the following atom-level expression sources:

| Atom source | Raw AST shape |
|---|---|
| Name | `AtomKind::Name(NameAst)` |
| Integer literal | `AtomKind::IntLiteral(String)` |
| Float literal | `AtomKind::FloatLiteral(String)` |
| String literal | `AtomKind::StringLiteral(String)` |
| Grouped expression | `AtomKind::Group(ExprAst)` |
| Closure | `AtomKind::Closure(ClosureAst)` |
| Error | `AtomKind::Error(ErrorAst)` |

See `spec/public/v0.2/lexical-syntax-v0.2.md` for the lexical forms of names and
literals. The parser preserves exact source text from literal tokens and
performs no value interpretation.

## 14. Atom suffixes

After parsing an atom base, the parser repeatedly folds suffix forms
left-to-right:

### 14.1 Navigation (`::`)

```text
x :: T :: std
```

produces a `NavPath` with components in inner-to-outer source order. The
leftmost component is the innermost selected symbol; the rightmost component
is the outermost scope component. The parser preserves source-order
navigation components and performs no lookup.

The innermost component must be a `Name` or `OperatorName`. Outer components
may be `Name` or a parenthesized grouped expression. A grouped expression is
valid only as an outer scope component; used as the innermost component it
produces a diagnostic.

### 14.2 Member (`.`)

```text
object.field
```

produces `MemberSugar`. The selector is a text `Name` only. Numeric selectors
have been removed. The parser performs no field or method lookup.

### 14.3 Double-dot (`..`)

```text
object..method(args)
```

produces `DoubleDotSugar`. The selector is a text `Name` only. A product form
must follow the selector. The parser performs no method resolution or dispatch.

### 14.4 Bracket-call sugar (`[...]`)

```text
obj[args...]
```

produces `BracketCallSugar`. `[]` is a contextual paired operator spelling.
`obj[]` is valid (empty args). Left-associative (`obj[a][b]` nests). The
parser does not interpret this as indexing, slicing, bounds checking, or
container access.

## 15. Operator expressions

Operator expressions are preserved as Raw AST sugar at the `OperatorExprAst`
layer. Ordinary operators bind tighter than whitespace auto-pipe and `|>`.
Operator parsing is segment-local and does not cross `|>` boundaries.

The parser preserves three fixity shapes:

- **Prefix** (`-x`): the sole prefix-negative shape. It is not an
  overloadable operator declaration. Prefix-negative is preserved as Raw AST
  sugar. Future normalization must desugar it non-semantically; the exact
  normalized representation belongs to the v0.3 normalization specification.
- **Postfix** (`obj!`, `obj?`): unary operator suffix. Postfix operators
  compose with the suffix chain (`obj!.field`).
- **Binary** (`a + b`): two-operand operator sugar.

Comparison, equality, and equals-suffixed operator chains are non-associative.
Ungrouped chains (`a < b < c`, `a == b == c`, `a += b += c`) produce a
`ChainedNonAssociativeOperator` diagnostic.

The `===` spelling is accepted as an ordinary non-associative binary operator
expression outside alias-let dispatch. This preserves restricted meta-body
forms such as `r === t`; the parser does not assign equality, aliasing, or
forwarding semantics to the operator.

The parser does not perform operator lookup, overload resolution, ADL,
type-directed lookup, mutation semantics, semantic validation, or semantic
lowering. Future normalization must desugar operator sugar non-semantically
to named operator calls.

## 16. Operator names in syntax positions

Operator names are syntactically accepted in:

- binder positions (`let + = expr`)
- alias binder positions (`let + === ...`)
- innermost navigation components (`+::int::std`)
- contextual `[]` operator-name positions

The `<` and `>` spellings are lexed as `Symbol::Less` / `Symbol::Greater`.
The parser reinterprets them as operator spellings in expression and operator
contexts. In strong binding contexts they may delimit a DeduceList.

This is syntactic preservation only. The parser does not attach operator
declaration identity, fixity validation, arity validation, or overload
semantics.

## 17. Closure literals

The parser recognizes two closure syntax categories:

### 17.1 In-place closure

```text
{ ... }   (in atom position)
```

Bare `{ ... }` in atom position produces `ClosureAst::InPlace`. It is not a
normal block expression. It has no capture clause, no parameter clause, no
return clause, and no head clauses. Having no extraction head means it has
no extracted input, including no implicit unit input.

### 17.2 Explicit headed closure

```text
FnHeadPrefix => { ... }
```

An explicit headed closure requires `=>` between the head prefix and the body.
`FnHeadPrefix { ... }` without `=>` is invalid and produces an
`InvalidClosureHead` diagnostic.

Closure literals produce AST, not callable objects. Closure materialization
into callable objects is a future semantic pass.

## 18. Closure heads

An explicit closure head (`FnHeadPrefix`) has a fixed clause order:

1. optional deduce list (`<...>`)
2. optional capture clause (`[...]`)
3. optional parameter clause (`()`, `(a, b)`)
4. optional fn-item-trait clause (`: trait_expr`)
5. optional return clause (`-> binding_slot`)
6. head clause tail (zero or more)

Active head clauses:

| Clause | Shape | Parser behavior |
|---|---|---|
| `Require` | `require Expr` | One expression slot preserved |
| `Pre` | `pre Expr` | One expression slot preserved |
| `Post` | `post Expr` | One expression slot preserved |
| `LifetimePre` | `lifetime pre Expr` | One expression slot preserved |
| `LifetimePost` | `lifetime post Expr` | One expression slot preserved |

The parser preserves each clause as a `HeadClauseAst` with exactly one
`ExprAst` slot. It performs no contract, lifetime, resource, type-level,
rank-level, or predicate validation.

No other closure-head clause names are active in v0.2. `acquire` is an ordinary
name (the earlier `acquire` direction is
replaced by the active `pre`/`post` head clauses).

## 19. Capture clauses

A capture clause is a bracket-delimited list of expression items:

```text
[expr1, expr2]
```

Each item is stored as `CaptureItemAst { expr: ExprAst }`, not as a name-only
item and not as a token tree. The parser does not interpret move, ref, copy,
or capture mode. No capture analysis is performed.

## 20. Parameter and return clauses

### 20.1 Parameter clause

```text
(a, b)
```

The parameter clause is one `ProductExtractAst`. Each element is a binding
slot in parameter context (no initializer, `with` allowed, `let` optional,
`<>` allowed per slot).

A parenthesized product in expression position is recognized as the parameter
clause of an explicit closure head when it is followed by later closure-head
material such as a fn-item-trait clause (`:`), return clause (`->`), head
clause, or body delimiter. For example, `(self, t: type): meta -> r => { ... }`
is parsed as an explicit headed closure, not as a product expression followed
by an unrelated `:` token.

### 20.2 Return clause

```text
-> binding_slot
```

The return clause is one `BindingSlotAst` in return context. `with { ... }`
is rejected in return slots. The return slot may be a wildcard (`-> _`), a
named slot (`-> result`), or an annotated slot (`-> _: annotation`).

The parser preserves these shapes but performs no type checking, return
checking, or semantic validation.

## 21. Canonical skeletons

Canonical skeletons are syntactic patterns used in extraction contexts
(extract-let binder, extract parameter, extract return). The parser builds
skeleton AST but does not execute matching.

A skeleton is a sequence of elements:

```text
CanonicalElement ::=
    CanonicalProductExtract   // ( ... )
  | "_"                       // wildcard
  | Name                      // node name or hole
  | Literal                   // literal atom
  | CanonicalNavPath          // Name::Name
```

In the restricted parameter-pattern forms needed by the v0.8 overload slice,
a top-level `|` inside a canonical skeleton segment may separate adjacent
pattern alternatives, as in `_ if | else`. This is parser preservation in a
strong pattern context. It is not policy union, expression-level operator
lookup, or pattern-space canonical-sum evaluation.

A `Name` in skeleton position carries a `CanonicalNameRole`:

- `Hole`: declared in the active deduce list
- `NodeName`: not declared; a structural node name
- `Unknown`: ambiguous context

These are parse-time role markers. No semantic matching is performed. All
canonical skeleton golden tests are parser preservation tests only.

Product extraction skeletons (`(...)`) accept product extraction elements
(including unit positions from commas) and nest.

## 22. Deduce lists

A deduce list declares names that act as holes in following syntax:

```text
<Name (, Name)*>
```

An empty deduce list (`<>`) is syntactically valid (the `BinderDeclList` is
optional), but the parser may emit a diagnostic depending on context.

A deduce list is recognized only in strong binding contexts:

- extract-let binder
- closure head
- parameter binder
- return binder

Outside these contexts, `<` and `>` are ordinary symbols. In expression and
operator contexts they may be operator spellings. Deduce lists are not
generic-call syntax, not template syntax, and not meta-function syntax.

Each deduce-list entry is a `BinderDeclAst` with a name and an optional
annotation term.

## 23. Match-style expressions and semantic names

The parser does not have special syntax for `match`, `else`, `if`,
`drop`, `move`, `sync`, `effect`, `fn`, `type`, or similar words. These are
ordinary `Name` tokens at the lexer level.
The parser does contextually recognize `return` in return terminal form
positions (see §Terminal Block Forms and Return Events).

The parser may produce ordinary expression structure — names, product forms,
closure AST arms — that future meta-functions or semantic passes may later
interpret. There is no `MatchExpr`, `IfExpr`, `FnDecl`,
`TypeDecl`, or `NamespaceDecl` node in Raw AST.
There is no semantic `ReturnStmt` construct; the parser
does preserve a structural `ReturnEvent` terminal form
(see §Terminal Block Forms and Return Events).


## 24. Parser diagnostics and recovery

The parser is error-tolerant. When an error is detected:

1. A `Diagnostic` is emitted with a primary span.
2. An `ErrorAst` node is inserted at the recovery point.
3. Parsing continues from a reasonable resynchronization point.

The complete diagnostic catalog (32 `DiagnosticCode` variants across lexer,
parser, operator, and alias categories) is documented in
`spec/implementation/v0.1/diagnostics-v0.1.md`. The concrete syntax document names diagnostics
only when needed to define a syntax boundary.

## 25. Non-semantic concrete-syntax boundary

The concrete syntax parser does not perform:

- semantic declaration classification
- name resolution
- type checking or kind checking
- operator lookup or overload resolution
- alias target resolution
- canonical matching
- closure materialization into callable objects
- ownership, lifetime, NLL, or drop insertion
- control-flow semantics
- effect semantics
- interpretation
- code generation
- HIR/MIR construction

Some structural recognition belongs to the parser; normalization handles
non-semantic desugaring; name resolution, type checking, operator lookup,
alias resolution, and evaluation behavior belong to later semantic passes.

## 26. Relationship to other documents

| Document | Relationship |
|---|---|
| `spec/public/v0.2/lexical-syntax-v0.2.md` | Defines the token categories and lexical rules consumed by this syntax. |
| `spec/contracts/raw-ast-contract-freeze-v0.2.md` | Defines the v0.2 freeze boundary in which this syntax is frozen. |
| `spec/public/v0.2/raw-ast-frozen-surface-v0.2.md` | Enumerates frozen Raw AST constructs and their guarantees. |
| `spec/implementation/v0.1/ast-construction-v0.1.md` | Detailed implementation-level parser construction spec. |
| `spec/history/v0.1/operator-design.md` | Detailed operator spelling design and implementation boundaries. |
| `spec/design/symbol-world/entity-alias-design.md` | Alias binding design (parser preservation implemented). |
| `spec/design/symbol-world/entity-ref-design.md` | Future general EntityRef design (alias-RHS subset implemented). |
| `spec/implementation/v0.1/diagnostics-v0.1.md` | Full diagnostic catalog including parser diagnostics. |
| `spec/implementation/v0.1/implementation-status-v0.1.md` | Authoritative factual inventory of current implementation. |

This document is the primary public concrete syntax reference. It defines the
current v0.2 parser surface. `lexical-syntax-v0.2.md` defines tokenization.
`ast-construction-v0.1.md` remains the detailed parser-construction and
implementation-level normative document. Historical design notes explain
decisions but are not the primary public syntax entry point.

## 27. Terminal Block Forms and Return Events

### Recognition

The parser contextually recognizes `return` only in return terminal
form positions at the form level:

```lang
E return;
E |> (T return);
E (T return);
E |> (return);
E (return);
```

`return` remains a `Name` token at the lexer level. The parser
disambiguates return terminal forms from ordinary name expressions
by detecting the `return` keyword after the value expression and
extracting the target syntax from the pipe (`|>`) or adjacency
(`(...)`) pattern.

### Raw AST

```text
FormAst ::= ...
  | ReturnEvent(ReturnEventAst)

ReturnEventAst {
  value: ExprAst,
  target: ReturnTargetAst,
  span: Span
}

ReturnTargetAst ::=
    ImplicitNearest { span }
  | Explicit { target: ExprAst, span }
```

### Meaning

| Source | Raw AST |
|---|---|
| `E return;` | `ReturnEvent(value = E, target = ImplicitNearest)` |
| `E \|> (return);` | `ReturnEvent(value = E, target = ImplicitNearest)` |
| `E (return);` | `ReturnEvent(value = E, target = ImplicitNearest)` |
| `E \|> (T return);` | `ReturnEvent(value = E, target = Explicit(T))` |
| `E (T return);` | `ReturnEvent(value = E, target = Explicit(T))` |

`E |> (return);` and `E (return);` are equivalent to `E return;`
(all produce `ImplicitNearest`). These forms are recognized by the
parser's return target extraction when the group contains only
`return` without an explicit target expression.
`T` is target syntax only. It is not resolved by the parser or
normalizer.

### Non-Expression Status

Return events are not expressions:

```text
ReturnEvent ∉ Expr
ReturnEvent ∉ Pattern
ReturnEvent ∉ Group
ReturnEvent ∈ BlockTerminal / Form
```

Return-like forms may not be grouped, embedded in expressions,
or used in non-form contexts. These are diagnostic-bearing:

```lang
(x return)
(x |> (T return))
(x (T return))

let y = (x return);
let y = x |> (T return);
let y = x (T return);

g((x return));
g(x |> (T return));
g(x (T return));

(x return) + y;
let y: (x return) = z;
```

But this is legal as a whole terminal form:

```lang
f(x return);
```

It means `ReturnEvent(value = f, target = Explicit(x))`, not a call
with `x return` as an argument.

### Terminal Block Enforcement

Once a terminal block form appears, no later form may occur before
`}`. Terminal forms are:

```text
TailValue(E)                    (bare expression in final position)
ReturnEvent(E, ImplicitNearest) (E return)
ReturnEvent(E, Explicit(T))     (E |> (T return) or E (T return))
```

The parser emits `StatementAfterTerminalBlockForm` for forms after
a terminal. Extra semicolons after a terminal are tolerated as
separators.
