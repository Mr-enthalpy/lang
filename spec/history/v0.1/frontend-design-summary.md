# v0.1 Frontend Design Summary (historical)

These are the early design decisions that shaped the `lang` Raw AST frontend.
They are historical context. The current authoritative descriptions are:

- the frozen Raw AST input syntax in `spec/public/v0.2/`, and
- the current normalized surface in `spec/public/v0.5/`.

The pipeline overview lives in `spec/history/v0.1/frontend-v0.1.md`; operator
design lives in `spec/history/v0.1/operator-design.md`.

## 1. Weak lexer

The lexer does not assign semantic roles to ordinary names. Names such as:

```text
return else match drop move sync effect fn type meta runtime compile
```

are ordinary `Name` tokens. Semantic strength does not imply lexical keyword
status.

## 2. Contextual parser

Some names can act as structure delimiters only in strong parser contexts:

- `let` at form start
- `require`/`pre`/`post`/`lifetime pre`/`lifetime post` as active raw-AST
  closure-head clauses (one expression slot each, no semantic validation);
  `acquire` an ordinary name
- `with` inside let bindings, only as `with { ... }`

Outside their context, they remain ordinary names.

## 2a. Inner-to-outer navigation

Navigation order is inner-to-outer. The leftmost component is the innermost
selected symbol, and the rightmost component is the outermost scope component.
Raw AST preserves source-order navigation components and performs no lookup.

```text
x::T::std
+::int::std
xxx::(int Vec::std)
```

Parenthesized right-side scope expressions after `::` are preserved as grouped
navigation components. Without parentheses, `::` consumes only one immediate
valid navigation component.

## 3. No traditional call syntax

The language does not use traditional `f(args)` as a general call form.
Parenthesized top-level-comma forms are product forms. In expression context
they are product construction; in binding / extraction context they are product
extraction.

## 4. `|>` as expression skeleton

Expression construction is not based on a traditional C-like operator-precedence
table. The expression frontend is organized around `|>` as the outer skeleton:

```text
top-level |> segmentation
  -> per-segment atom folding
  -> per-segment operator sugar
  -> per-segment automatic pipe
  -> product form preservation
```

The parser preserves a segment-local `OperatorExpr` layer
(`SegmentElement := OperatorExpr | Product`). Ordinary operators bind tighter
than whitespace auto-pipe and `|>`, but they remain AST sugar: no lookup, type
checking, evaluation, mutation semantics, or lowering is performed by the
parser, and operator parsing is local to one pipe segment.

## 5. Closure literals produce AST first

A closure literal initially produces closure AST, not a callable object. Bare
`{ ... }` in atom position produces `ClosureAst::InPlace`; it is not a normal
block expression and has no closure head. Braces also delimit explicit closure
bodies after `FnHeadPrefix =>`. Compiler meta-functions may directly consume
closure AST.

## 6. `<>` declares holes

`<...>` has exactly one special use: declaring names that act as holes in
following syntax, recognized only in binding contexts. It is not generic-call,
template, or meta-function syntax. Individual `<`, `>`, `<=`, and `>=` spellings
are documented as planned operator names in expression/operator contexts.

## 7. Declarations enter through `let`

All user-visible declarations use `let`. There is no dedicated parser syntax for
function, type, or namespace declarations. `fn`, `type`, and `namespace` are
ordinary `Name` tokens, not lexer keywords. Declaration annotations are parsed
and preserved but not semantically checked. Rank annotations require the
explicit `type_object_annotation : rank_annotation` form.

## 8. Parser owns shape, semantics owns meaning

The parser constructs and preserves raw AST shape. It does not decide what an
AST fragment semantically represents; future semantic or meta-function passes
may interpret preserved shapes. Parse left to right; do not go back to
reinterpret meaning. The frontend must not add special AST nodes just because a
future built-in meta-function may understand a shape.
