# Normalized Surface Semantics v0.5

> **Status:** v0.5-2 publishes the call / product / pipe binding semantics
> (§1–§7, §11–§13). The value-side / pattern-side, annotation, and alias
> sections (§8–§10) remain concise placeholders for v0.5-3. Every example below
> is cross-checked against the v0.4 normalized dumps in `tests/cases/norm/`.

This document explains the current normalized surface behavior for both human
readers and coding/documentation agents. Where a rule has an implementation
name, it is given in two layers:

```text
Conceptual rule: <reader-facing name>
Dump label:      <name shown in the normalized dump origin=…>
```

The conceptual name explains the semantic position; the dump label lets agents
and implementers cross-check the documentation against actual
`normalize_program` / golden dump output.

## 1. Purpose and Scope

This document describes current normalized surface semantics: how
already-parsed Raw AST is read and lowered into Normalized AST at the
call / product / sugar level.

```text
Normalized surface semantics are not HIR semantics.
They describe structural binding and lowering before name resolution,
type checking, operator lookup, or runtime evaluation.
```

It explains the structure the normalizer builds. It does not explain what any
name, operator, field, method, or type *means*.

## 2. Stage Boundary

v0.5 stabilizes the public explanation of behavior already produced by the v0.4
normalizer. The structure is fixed; this document makes it readable.

```text
The normalizer does not decide whether a target exists.
The normalizer does not decide whether a call is valid.
The normalizer does not decide whether an operator, field, method, or type exists.
The normalizer only builds the unresolved normalized structure.
```

## 3. Source Product and Target Expression

Notation:

```text
P = Product / source product
e = ordinary expression / target-capable expression
G = group expression
```

Shape discipline:

```text
P can be a source.
e can be a target.
G can become P only when a source Product is required.
P cannot be a target.
```

Local forms:

```text
e e        -> e
P e        -> e
P |> e P   -> e

(e, e)     -> P
(e, P)     -> P
(P, e)     -> P
(P, P)     -> P
P          -> P
e          -> e
()         -> Product(Unit)

(e)        -> G
G          -> e   in ordinary expression position
G          -> P   when lifted in source-product position
```

Every normalized call has the shape `Product |> target`. The source side is
always a Product. When a single expression is used as a call source, it is
lifted into a one-element source Product.

```text
Conceptual rule: product lift
Dump label:      ProductLift
```

`(x)` in the normalized examples below denotes a normalized source Product
containing `x` (a `ProductLift`), not a re-parseable source group.

## 4. Source-Product Continuation

```text
Conceptual rule: source-product continuation
Dump label:      ProductMerge
```

The central call-binding rule:

```text
Product1 |> TargetExpr Product2
=> (Product1, Product2) |> TargetExpr
```

The source product may be written discontinuously around the target
expression. The following Product is merged back into the incoming source
Product. The target expression is **not** receiving an argument list in the
conventional sense; `Product2` is the first source-product continuation of the
incoming source Product, not an argument list of `TargetExpr`.

Examples (verified against `tests/cases/norm/`):

```text
x |> f (a)
=> (x, a) |> f

x |> f (a) g
=> ((x, a) |> f) |> g

x |> f h (a)
=> (x, a) |> (f h)

x |> (f h) (a)
=> (x, a) |> (f h)

Product1 |> expr1 Product2 expr2 |> expr3
=> (((Product1, Product2) |> expr1) |> expr2) |> expr3
```

In `x |> f (a) g`, the inner `(x, a) |> f` is the continuation (`ProductMerge`),
and the outer `|> g` is ordinary expression-chain growth (see §5).

### First-product-only

An incoming source Product absorbs only the **first** following Product:

```text
P1 |> X P2 Y P3
=> ((P1, P2) |> X) Y P3
```

It is **not**:

```text
(P1, P2, P3) |> X
```

`Y P3` is then normalized as residual expression-chain material. For example,
`x |> f (a) (b)` merges only `(a)` into the source product and leaves `(b)` as
residual: `((x, a) |> f) |> b`.

### Why this exists

The language's main written call skeleton is not callee-first. It lets the
source product be written before and after the target expression while
preserving a single normalized source Product. The "argument list" reading is
explicitly rejected: a following Product is source-product continuation, not a
conventional argument list.

## 5. Legality Repairs

When a source-product continuation cannot be formed, normalization falls back to
two legality repairs. Neither is the main call skeleton.

### First legality repair

```text
Conceptual rule: first legality repair
Dump label:      PipeFallback
Condition:       no following source product / no source-product continuation can be formed
Shape:           P |> e
```

Used when an incoming source Product has no following Product to merge. Example:

```text
x |> f
=> (x) |> f          // Derived(PipeFallback; no following source product)
```

The same `PipeFallback` label also marks ordinary expression-chain growth after
a skeleton has closed (dump summary `ordinary expression-chain growth`), as in
the outer `|> g` of `x |> f (a) g`.

### Second legality repair

```text
Conceptual rule: second legality repair
Dump label:      SecondLegalityRepair
Shape:           e ... P e ...  =>  e ... (P |> e) ...
```

A Product cannot be a target. When a naked Product would otherwise land in
target position (which would form the illegal `e |> P`), and another expression
follows it, the Product is grouped with that following expression instead:

```text
f (a) g
=> f |> ((a) |> g)
```

(In the dump this is two nested `SecondLegalityRepair` calls: the outer source
is `(f)`, whose target is the repaired `(a) |> g`.)

### Second repair never overrides source-product continuation

When an incoming source Product exists, continuation wins; the second repair
does not fire:

```text
x |> f (a) g
=> ((x, a) |> f) |> g
```

It is **not**:

```text
x |> (f |> ((a) |> g))
```

```text
Second repair never overrides source-product continuation.
```

## 6. Product, Group, and Unit Boundaries

Unit and comma positions are preserved exactly; they are never silently dropped:

```text
()        => Product(Unit)
(a,)      => Product(a, Unit)
(,a)      => Product(Unit, a)
(a,,b)    => Product(a, Unit, b)
(a,(b,c)) does not flatten to (a,b,c)
```

Group rules:

```text
(x) is a group in expression position.
It does not survive as a persistent NormExpr::Group.
A group may be product-lifted (ProductLift) only when a source Product is required.
((x)) unwraps to x.
```

Examples:

```text
(x) |> f
=> (x) |> f            // grouped expression becomes the normalized source product

x |> f ((a))
=> (x, a) |> f         // single-element group unwraps, then merges

x |> f ((a, b))
=> (x, (a, b)) |> f    // inner product is preserved as one nested element
```

The notation is normalized, not raw source: `(x)` here is a normalized source
Product, and a nested `(a, b)` is a preserved product element, not a flattened
list.

## 7. Operator / Member / Double-Dot / Bracket Sugar

All of these are normalization-level lowering into the same product-call
skeleton. None of them perform lookup, dispatch, or resolution.

### Operator lowering

```text
Conceptual rule: operator lowering
Dump label:      OperatorLowering
```

```text
a + b
=> (a, b) |> +        // OperatorTarget spelling="+" fixity=Binary arity=2

a!
=> (a) |> !           // OperatorTarget spelling="!" fixity=Postfix arity=1
```

The operator becomes an unresolved `OperatorTarget` carrying its spelling,
fixity, and arity. No operator lookup or overload resolution occurs.

### Prefix negative

```text
Conceptual rule: prefix-negative lowering
Dump label:      PrefixNegativeLowering
```

```text
-x
=> x |> generated closure:
   <T: type>(val: T) => { (zero::T, val) |> - }
```

```text
Prefix negative is not an overloadable prefix operator identity.
Only the generated binary `-` participates in later operator lookup.
No operator lookup occurs during normalization.
```

### Member sugar

```text
Conceptual rule: member lowering
Dump label:      MemberLowering
```

```text
obj.field
=> obj |> generated closure:
   <T: type>(val: T) => { (val) |> field::T }
```

### Double-dot sugar

```text
Conceptual rule: double-dot lowering
Dump label:      DoubleDotLowering
```

```text
obj..method(args...)
=> obj |> generated closure:
   <T: type>(val: T) => { (val, args...) |> method::T }
```

### Bracket sugar

```text
Conceptual rule: bracket-call lowering
Dump label:      BracketCallLowering
```

```text
obj[args...]
=> (obj, args...) |> []      // OperatorTarget spelling="[]" fixity=BracketCall

obj[]
=> (obj) |> []               // arity 1; empty bracket payload contributes no implicit Unit
```

```text
Empty bracket payload contributes no implicit Unit.
Explicit `()` inside brackets is a user-written Unit product: obj[()] => (obj, ()) |> [].
```

### Shared boundary

In the generated closures, `T` and `val` are local generated binders, and the
receiver becomes the call's source product (a `ProductLift`).

```text
`field::T` and `method::T` are unresolved navigation targets.
No field lookup, method lookup, method dispatch, type checking, or overload resolution occurs.
```

## 8. Value-Side vs Pattern-Side Material

_(Placeholder for v0.5-3.)_ Value-side material remains `NormExpr` and
pattern-side material remains `NormPattern`. A value enters pattern space only
through an explicit bridge; a pattern exposes values only through explicit
extraction. Pattern-side names are not ordinary call targets. Backing:
`spec/contracts/v0.4-normalization-prototype-notes.md`.

## 9. Annotation Patterns and DeduceList Holes

_(Placeholder for v0.5-3.)_ Annotations are annotation-pattern (classifier)
material, not ordinary runtime expressions; DeduceList is a binding-site hole
binder list whose holes may appear inside annotation patterns. Backing:
`spec/public/v0.3/normalized-ast-specification-v0.3.md` §8.2–§8.5.

## 10. Alias Preservation

_(Placeholder for v0.5-3.)_ Alias declarations are preserved as unresolved
declarations with an `EntityRef` right-hand side; no target resolution, scope
semantics, or operator alias identity validation occurs at the normalized layer.
Backing: `spec/public/v0.3/normalized-ast-specification-v0.3.md` §7.13, §8.8.

## 11. Origin, Generated Nodes, Derived Nodes, and Unsupported

Every normalized node carries an origin in the dump:

```text
origin=Source
origin=Generated(<Rule>)
origin=Derived(<Rule>; <summary>)
```

- **Source nodes** come directly from source.
- **Generated nodes** are introduced by a single named lowering rule.
- **Derived nodes** combine multiple source/generated inputs, such as a product
  merge.
- **Unsupported nodes** are ordinary normalized nodes whose payload records an
  unsupported Raw AST subshape (for example `Unsupported "..."` or
  `PatternUnsupported "..."`). They are surfaced explicitly instead of being
  silently erased. `Unsupported` is a node kind / rule label, not a separate
  origin: such a node's origin usually uses `Generated(Unsupported)` or another
  explicit rule label.

Rule labels used by the call-binding and sugar-lowering examples in this
document:

```text
Generated:
  ProductLift
  OperatorLowering
  PrefixNegativeLowering
  MemberLowering
  DoubleDotLowering
  BracketCallLowering
  Unsupported             (node surfaced explicitly; origin Generated(Unsupported))

Derived:
  ProductMerge            (source-product continuation)
  PipeFallback            (first legality repair / ordinary expression-chain growth)
  SecondLegalityRepair    (second legality repair)
```

This is not the complete Normalized AST rule-label inventory; v0.5-3 expands the
remaining pattern / alias / closure-facing labels (for example
`PatternNormalize`).

These labels appear verbatim in the normalized dump, so any example in this
document can be cross-checked against `normalize_program` output and the golden
fixtures in `tests/cases/norm/`.

## 12. Non-Goals

The normalized surface does not perform name resolution, type/kind checking,
operator lookup, operator overload resolution, alias target resolution,
namespace resolution, pattern-head resolution, canonical matching, closure
materialization, capture analysis, ownership/NLL/drop, effect interpretation,
runtime evaluation, or code generation. It does not turn Normalized AST into
HIR.

A source Product is never a conventional argument list. There is no callee-first
call, method dispatch, field lookup, resolved function call, operator overload
resolution, or ADL at the normalized layer.

Backing: `spec/contracts/v0.4-normalization-prototype-notes.md`.

## 13. Relation to v0.6+ Future Semantics

Later pattern-space and extraction-chain semantics
(`spec/future/static-pattern-spaces-and-extraction-chains.md`) motivate the
value-side / pattern-side boundaries, but they are **not** current normalized
call-binding behavior. `Done`, residual propagation, pattern-space subtraction,
`operator+` meta-reduction, `match` closing, exhaustiveness, and pattern-head
resolution are future semantics, not current behavior, and must not be read as
implemented.
