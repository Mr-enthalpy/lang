# Normalized AST Specification v0.3

## 1. Purpose

This document is the v0.3 Normalized AST Specification scaffold. It defines
the problem space, the non-semantic boundary, the active design questions,
and the specification work items that v0.3 must resolve.

It does not define the final Normalized AST node set.

## 2. Stage boundary

```text
v0.1   — Raw AST Frontend — completed
v0.1.w — Raw AST Stability Window — closed
v0.2   — Raw AST Contract Freeze / Public Frontend Syntax Specification — closed
v0.3   — Normalized AST Specification — current
v0.4   — Raw AST → Normalized AST Prototype — future
```

## 3. Input authority

v0.3 Normalized AST Specification consumes the frozen v0.2 Raw AST frontend
surface as its input contract.

Raw AST is source-preserving and non-desugared. It preserves:

- operator sugar
- member, double-dot, and bracket-call sugar
- pipe / segment / product architecture
- closure literal structure
- canonical skeletons and deduce lists
- alias-let and EntityRef preservation

Normalized AST will be desugared, structurally regular, and suitable for
later semantic passes (name resolution, type checking, etc.).

## 4. Non-semantic boundary

Normalized AST is:

- desugared but still non-semantic
- not HIR (HIR assumes name resolution and type checking)
- not type-checked
- not name-resolved
- not an execution model

Normalized AST does not:

- resolve names
- resolve operators
- resolve alias targets
- check types or kinds
- validate canonical skeletons
- validate deduce lists
- materialize closures
- perform ownership / NLL / drop analysis

## 5. Design questions active in v0.3

The following questions from `spec/planning/open-questions.md` become active
during v0.3. They are listed here without being resolved.

- **N-AST-1.** Exact Normalized AST node set. What are the exact node types?
  Candidates: normalized call, normalized pattern, normalized declaration.
  Should there be a single unified expression node or distinct per-form nodes?

- **N-AST-2.** Whether Normalized AST lives in `lang_syntax` or a new crate
  (e.g., `lang_norm`).

- **N-AST-3.** Whether raw-to-normalized dumps should be golden-tested.

- **N-AST-4.** How to represent symbolic builtins introduced by desugaring
  (e.g., `operator::call`, `member::lookup`, `pattern::bind`).

- **N-AST-5.** How to preserve source origins through desugaring. Desugaring
  creates new AST nodes that did not appear in source text. How should source
  spans and diagnostic attribution be preserved?

- **N-AST-6.** Whether right-target subsegments become nested call nodes.
  Right-target subsegments (`f (a) g`) are currently flat in Raw AST.

- **N-AST-7.** How to represent pattern normalization for let, params, returns,
  and canonical skeletons.

- **N-AST-8.** How to represent alias declarations before name resolution.
  Alias bindings reference compile-time entities that are not yet resolved.

- **N-AST-9.** Member / double-dot sugar lowering — **adopted** into §7. The
  navigation-based pipe + closure lowering for member, double-dot, and
  bracket-call sugar (and the defensive branch-name expansion) is recorded in
  §7.11 / §7.14. The earlier review concerns are settled: the lowered forms are
  normalized construction notation (a concrete v0.2-source rendering of the
  generated closure would require `=>`); branch-name expansion is defensive-only
  per the frozen guarantee; and the navigation-based member form is adopted with
  the frozen `raw-ast-frozen-surface-v0.2.md` §14 wording left unchanged. See
  `spec/planning/open-questions.md` N-AST-9 for the resolution record.

## 6. Specification work items

The following items must be decided during v0.3 specification. They are
listed as obligation checklists, not as resolved decisions.

- normalized form / declaration / expression boundary
- normalized call / composition structure
- product expression normalization
- product extraction and binding pattern normalization
- operator sugar normalization (prefix-negative, postfix, binary)
- bracket-call normalization (`obj[args...]`)
- member sugar normalization (`obj.field`)
- double-dot sugar normalization (`obj..method(args)`)
- navigation normalization (inner-to-outer `::`)
- closure literal and closure-head normalization (InPlace, Explicit)
- alias-let representation before name resolution
- `ErrorAst` and diagnostics origin preservation through normalization
- how generated / desugared nodes carry source origin
- normalized dump policy (format, golden-test policy)
- crate placement (whether Normalized AST types live in `lang_syntax` or a new crate)

## 7. Source-product continuation call skeleton

This section records the v0.3 design decision for core call normalization. It is
a structural (non-semantic) decision. It does not perform name resolution,
operator lookup, overload resolution, type/kind checking, alias target
resolution, namespace resolution, canonical matching, closure materialization,
ownership/NLL/drop, effect interpretation, runtime evaluation, or code
generation.

### 7.0 Wording discipline

Use: normalize, desugar, preserve, carry provenance, defer, unresolved target
expression, source product, target expression, source-product continuation,
structural call skeleton, product merge, legality repair.

Avoid (or explicitly mark as future semantic interpretation): call function,
arguments, method dispatch, field lookup, bind semantics, monad semantics,
sum-type dispatch semantics, resolve, infer, evaluate, materialize.

### 7.1 Core skeleton

The center of v0.3 call normalization is the source-product continuation
skeleton, not traditional callee-first function application.

Notation:

```text
P = Product / source Product
e = ordinary / target-capable expression
G = group expression
```

The core writing skeleton is:

```text
P1 |> e P2
```

A source Product may be written discontinuously around a target expression.
Normalization closes the split source Product by product merge:

```text
P1 |> e P2
=> (P1, P2) |> e
```

This is the core rule. Surrounding association and repair rules exist only to
let expression material grow while preserving this skeleton. They must not
reinterpret `P2` as a traditional argument list of `e`.

The canonical normalized call form is `Product |> Expr`. It is a structural call
skeleton, not a resolved function call. `Expr` is an unresolved target
expression; v0.3 does not decide whether the target is callable.

### 7.2 Shape classes

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
()         -> P

(e)        -> G
G          -> e   (in ordinary expression position)
G          -> P   (when lifted in source-product position)
```

Notes:

1. `P` can be a source.
2. `e` can be a target.
3. `G` is a group. It is not originally a Product, but it may be promoted to a
   one-element Product when a source Product is required.
4. `P` must not be treated as a target.
5. The invalid shape to avoid is `e |> P`. A Product is not a valid target
   expression in the normalized call skeleton.

### 7.3 Growth model

```text
P |> e P e e e e e |> e P e e e ...
```

is understood structurally as:

```text
(((P |> e P) e e e e e) |> e P) e e e ...
```

Each closed skeleton is then normalized by product merge:

```text
P1 |> e P2
=> (P1, P2) |> e
```

### 7.4 First-product-only rule

When an incoming source Product searches to the right inside the same segment,
it absorbs only the first following Product before the next explicit `|>`. The
search stops after that first Product. Later Products are handled by subsequent
normalization of the remaining material.

```text
P1 |> X P2 Y P3
=> ((P1, P2) |> X) Y P3
```

It must not be documented as `(P1, P2, P3) |> X`. `Y P3` is then normalized as
residual segment material.

### 7.5 Priority order

**Priority 0 — high-priority local unit formation.** These local units form
before source-product continuation recognition:

```text
grouping
navigation
member sugar
double-dot sugar
bracket-call sugar
postfix operator suffix
operator sugar
```

They are local structural units, not ordinary low-priority association paths.

**Priority 1 — core skeleton growth.**

```text
P1 |> e P2
=> (P1, P2) |> e
```

**Priority 2 — residual growth after a closed skeleton.**

```text
P |> e P2 e3 e4
=> (((P, P2) |> e) |> e3) |> e4
```

**Priority 3 — first legality repair.** If `P |> e P2` cannot be formed because
no following Product exists before the next boundary, the structure falls back
to ordinary `P |> e`. This is the fallback when the source-product continuation
skeleton cannot close; it is not the main skeleton.

**Priority 4 — second legality repair.** In an ordinary expression chain, a
naked Product may appear in a position that would otherwise produce the illegal
form `e |> P`. If the Product is followed by another expression, normalization
repairs before error by grouping the Product with the following expression:

```text
e ... P e ...
=> e ... (P |> e) ...
```

compactly `e ... (P e) ...`, where `P e -> e`. This is the lowest-priority
legality repair before reporting an error.

Invariant:

```text
P |> e P2 is the core skeleton.
P |> e is the first repair when P |> e P2 cannot be formed.
e ... P e ... => e ... (P |> e) ... is the second repair when ordinary
  e-chain growth would otherwise create expr |> Product.
expr |> Product is never the intended normalized result.
```

### 7.6 `f Product g` is a repair, not call sugar

`f Product g` is not a positive local call sugar. It is only an instance of the
second legality repair. Ordinary left growth would attempt `f Product => f |>
Product`, which is illegal because a Product cannot be a target. Since another
expression `g` follows, the repair is:

```text
f Product g
=> f (Product |> g)
```

compactly `f (Product g)`.

This repair must never override source-product continuation:

```text
P |> f Product g
=> ((P, Product) |> f) |> g
```

It must not be rewritten as `P |> (f |> (Product |> g))`.

### 7.7 Skeleton examples

```text
P |> f (args)
=> (P, args) |> f

P |> f (args) g
=> ((P, args) |> f) |> g

P |> f h (args)
=> (P, args) |> (f h)

P |> (f h) (args)
=> (P, args) |> (f h)

Product1 |> expr1 Product2 expr2 |> expr3
=> (((Product1, Product2) |> expr1) |> expr2) |> expr3
```

In these examples, `Product2` or `(args)` is not an argument list of `expr1`,
`h`, or `(f h)`. It is the first product continuation of the incoming source
Product.

### 7.8 Product normalization

1. The left side of a normalized call is always a Product.
2. Ordinary expressions and group expressions can be product-lifted when used as
   call sources.
3. `()` corresponds to a unit Product / unit element, not an empty Product.
4. `obj[]` lowers to `(obj) |> []`, not `(obj, ()) |> []`.
5. Raw product unit positions are preserved.
6. Product normalization must not recursively flatten nested Products.
7. Flattening that depends on mode specifiers, type derivation, or checking
   belongs to later phases, not v0.3.

```text
x |> f
=> (x) |> f
```

Here `(x)` is normalized product notation, not a claim that Raw source `(x)` is
a Product. Raw source `(x)` is a group `G`, but `G` can be lifted to `P` in
source-product position.

```text
obj[]
=> (obj) |> []

(a,)
=> Product(a, Unit)

(,a)
=> Product(Unit, a)

(a,,b)
=> Product(a, Unit, b)
```

`(a, (b, c))` must not be silently flattened into `(a, b, c)`.

### 7.9 Motivating examples (no semantics)

These show the intended structural shape only. v0.3 does not define monad
semantics, bind semantics, sum-type dispatch, generic dispatch, dynamic
dispatch, or overload behavior.

```text
monad |> f bind (args...)
=> (monad, args...) |> (f bind)

sumtype_val |> Generic_fun sum_type_dispatch(args...)
=> (sumtype_val, args...) |> (Generic_fun sum_type_dispatch)
```

### 7.10 Operator lowering

Operators are high-priority local sugar and later lower into the same
product-call skeleton:

```text
a + b
=> (a, b) |> +

a!
=> (a) |> !
```

v0.3 preserves operator provenance — spelling, source fixity, source arity,
source span / origin. This is syntax provenance for later
symbol-binding/operator-lookup phases, not operator lookup, alias resolution, or
overload resolution.

Prefix negative is special:

```text
-x
=> (x |> <T: type>(val: T) { (zero::T, val) |> - })
```

The operand `x` is piped once into the generated closure and bound to `val`, so
the operand has a single source and is evaluated once — the same
single-evaluation rationale as member and double-dot lowering.

Prefix negative is not an overloadable prefix operator. `zero::T` and `-` remain
unresolved. Only the generated binary minus participates in later operator
lookup.

> Documentation debt: this v0.3 form supersedes the provisional
> `()zero::(x |> type) - x` sketch still written in
> `spec/history/v0.1/operator-design.md` and `spec/reference/glossary.md`
> (PrefixNegative). `spec/public/v0.2/raw-ast-frozen-surface-v0.2.md` §13 defers
> the exact normalized form to v0.3. Reconciling those two documents is deferred
> to a later consistency pass and is tracked in `spec/planning/open-questions.md`.

### 7.11 Member, double-dot, and bracket-call lowering

These are high-priority local sugar forms. They lower before ordinary call
skeleton composition because they must preserve single evaluation of the
receiver.

```text
expr.field
=> (expr |> <T: type>(val : T) { (val) |> field::T })

expr..member_fun(args...)
=> (expr |> <T: type>(val : T) { (val, args...) |> member_fun::T })

obj[args...]
=> (obj, args...) |> []

obj[]
=> (obj) |> []
```

The empty bracket payload contributes no unit argument.

These rules do not perform field lookup, method lookup, dispatch, overload
resolution, or type checking. They only construct unresolved navigation targets
using the locally bound receiver type name `T`.

`T` and `val` are local generated binders inside the generated closure. They do
not conflict with outer names because they are local to the generated closure.
`field::T` and `member_fun::T` refer to that local generated type binder in the
lowering notation.

The lowered forms are normalized construction notation describing generated
closure structure. The generated closure is an explicit (headed) closure; a
concrete `v0.2`-source rendering would require `=>` between head and body. The
frozen `raw-ast-frozen-surface-v0.2.md` §14 member-access wording is left
unchanged; this mapping is recorded in v0.3 only.

### 7.12 Closure boundary

Closure bodies are recursively normalized in v0.3, but closure materialization
does not occur. Closure AST remains closure AST / normalized closure expression.
v0.3 preserves the distinction between in-place closure and explicit closure. A
headless in-place closure has no implicit unit input.

### 7.13 Alias boundary

Alias-let does not participate in call normalization. `let binder === EntityRef`
normalizes, if at all in v0.3, only into an unresolved alias declaration form. It
must not be lowered into ordinary expression or call structure. Alias RHS remains
`EntityRef`, not `PipeExpr`, not `Product`, not `ClosureAst`, not a runtime
expression. Alias target resolution, alias scope semantics, operator alias
identity validation, and namespace resolution are later phases.

### 7.14 Bare branch-name shorthand

If the Raw AST input still contains an unexpanded incoming branch-name
shorthand, v0.3 normalization may mechanically expand:

```text
|> name { body }
=> |> (_ name) { body }
```

This is purely syntactic. It does not assign wildcard, match, branch,
control-flow, or pattern semantics to `_` or `name`. By the frozen guarantee
(`raw-ast-frozen-surface-v0.2.md` §12; `ast-construction-v0.1.md` §7.1.1) the
Raw AST already expands this shape at parse time, so this rule is defensive and
idempotent only.

### 7.15 Origin and provenance

Normalized AST nodes must be traceable. v0.3 does not require a final Rust
source-map implementation, but the specification requires that generated or
derived nodes carry enough origin/provenance to explain where they came from.

Normalized nodes may be:

```text
source nodes
generated nodes
derived nodes
```

- Source nodes correspond directly to source spans.
- Generated nodes are introduced by a named lowering rule, such as product
  lifting, prefix-negative lowering, member sugar lowering, double-dot lowering,
  bracket-call lowering, or branch-name shorthand expansion.
- Derived nodes combine multiple source/generated inputs, such as
  `(P1, P2) |> e` produced by source-product continuation closure.

The exact Rust representation is a v0.4 implementation question, but v0.3
requires traceability so that future normalized dumps and diagnostics can
attribute generated structures to source spans and lowering rules.

## 8. Explicit non-goals

v0.3 must not:

- create Normalized AST Rust types in the codebase (v0.4)
- implement Raw AST → Normalized AST lowering (v0.4)
- create normalized dumps or golden snapshots (v0.4)
- modify the lexer, parser, Raw AST shape, or DiagnosticCode
- change the v0.2 frozen frontend syntax surface
- perform semantic analysis
- perform name resolution
- perform type checking
- perform operator lookup
- perform alias target resolution

## 9. Status

This document is no longer a bare stage-opening scaffold. §3 and §4 boundaries
are defined and stable. §7 records the v0.3 source-product continuation
call-skeleton design decision, including product merge, the priority order and
legality repairs, product normalization, operator lowering, member/double-dot/
bracket-call lowering, the closure and alias boundaries, branch-name expansion,
and the origin/provenance requirement.

The corresponding §6 work items are now recorded as design decisions (normalized
call/composition structure, product expression normalization, operator sugar
normalization, bracket-call/member/double-dot normalization, alias-let
representation boundary, closure-head normalization boundary, and how generated/
desugared nodes carry source origin). The §5/open-questions statuses are updated
accordingly (N-AST-2/6/8 resolved; N-AST-1/5 partially resolved; N-AST-9
adopted).

This does not mean v0.3 is complete. The remaining §6 work items — product
extraction and binding-pattern normalization, navigation normalization detail,
canonical-skeleton/deduce-list pattern normalization (N-AST-7), `ErrorAst` and
diagnostics origin preservation, normalized dump policy (N-AST-3), and the exact
Normalized AST node set (N-AST-1) — are still open.
