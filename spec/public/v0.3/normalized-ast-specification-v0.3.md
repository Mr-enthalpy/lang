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

The following questions from `spec/planning/open-questions.md` are tracked for
v0.3. Several are now resolved or partially resolved; each entry is annotated
with its status and the section that records the decision. Full resolution
detail lives in `spec/planning/open-questions.md`.

- **N-AST-1.** Exact Normalized AST node set — **partially resolved for v0.4
  start** (§7, §8): minimum node roles specified, exact Rust node set deferred.
- **N-AST-2.** Crate placement — **resolved**: Normalized AST stays under
  `lang_syntax`.
- **N-AST-3.** Normalized dump / golden policy — **resolved for v0.4 start**
  (§8.11).
- **N-AST-4.** Symbolic builtins — **resolved for the v0.3 boundary** (§8.7): no
  general symbolic builtin node family.
- **N-AST-5.** Source origins through desugaring — **partially resolved**
  (§7.15): exact Rust source-map deferred to v0.4.
- **N-AST-6.** Right-target subsegments — **resolved** (§7) via the
  source-product continuation skeleton.
- **N-AST-7.** Pattern / binding-site normalization — **resolved for v0.4
  start** (§8.2–§8.5).
- **N-AST-8.** Alias declarations before name resolution — **resolved for the
  v0.3 boundary** (§7.13).
- **N-AST-9.** Member / double-dot sugar lowering — **adopted** (§7.11, §7.14).

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
3. `()` corresponds to a Product containing one Unit element, not an empty Product.
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

As in §7.11, this lowered form is normalized construction notation describing
generated closure structure, not a re-parseable v0.2 source rendering. The
generated closure is an explicit (headed) closure; a concrete `v0.2`-source
rendering would require `=>` between head and body.

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

## 8. Minimum Normalized AST shape for v0.4

This section closes the minimum v0.3 direction questions needed to begin v0.4
Raw AST → Normalized AST implementation. It builds on the §7 source-product
continuation call skeleton (which is not reopened here) and defines the intended
Normalized AST shape boundary — the required structural roles, not final Rust
types.

This section remains specification-only and non-semantic. v0.3 closeout must not
perform name resolution, type checking, kind checking, operator lookup, operator
overload resolution, alias target resolution, namespace resolution, canonical
matching, closure materialization, capture analysis, ownership/NLL/drop
insertion, effect interpretation, runtime evaluation, or code generation. The
exact Rust enum/struct names below are illustrative; v0.4 may choose different
spellings as long as the structural roles are preserved.

### 8.1 Group does not survive as a normalized expression node

Raw source grouping `(e)` is a group `G`. Normalized AST does not preserve a
dedicated `NormExpr::Group` node as ordinary expression structure. Per the §7.2
shape classes:

```text
(e) -> G
G   -> e     in ordinary expression position
G   -> P     when lifted in source-product position
```

The fact that an expression originated from a group is carried by
origin/provenance (§7.15), not by a persistent normalized group node. This
supports the already recorded rule (§7.8):

```text
x |> f
=> (x) |> f
```

where `(x)` is normalized product notation, not a claim that the Raw source
`(x)` was a Product.

### 8.2 Unified normalized binding-site structure

v0.4 keeps a single normalized binding-site / binding-slot concept. The exact
Rust type name is an implementation detail; the structural shape is:

```text
- optional policy / modifiers
- a DeduceList of local hole declarations
- a value / extraction pattern
- an optional annotation pattern
- an optional with-name list
- an optional initializer / body, depending on context
```

This mirrors the frozen `BindingSlotAst` shape (`concrete-syntax-v0.2.md` §7)
and is reused for the user-visible binding contexts where appropriate:

```text
let binding slots
closure parameters
closure returns / result slots
generated closure heads introduced by lowering
```

Do not split let / param / return into unrelated normalized structures unless
v0.4 implementation later proves it necessary.

### 8.3 DeduceList is a binding-site hole binder list; annotation is a pattern

This is the most important remaining direction decision.

```text
DeduceList is a binding-site hole binder list.
Annotation is an annotation pattern / classifier pattern.
Hole names declared by DeduceList may occur inside annotation patterns.
DeduceList is not merged into the value / extraction pattern itself.
Annotation is not normalized as an ordinary runtime expression.
```

Example:

```text
let <T> x: T = ...
```

normalizes structurally as:

```text
BindingSlot {
  deduce: [HoleDecl(T)],
  value_pattern: Binder(x),
  annotation: AnnotationPattern(HoleRef(T)),
  ...
}
```

A more complex annotation pattern:

```text
let <T> x: T Option::std = ...
```

normalizes as:

```text
BindingSlot {
  deduce: [HoleDecl(T)],
  value_pattern: Binder(x),
  annotation: AnnotationPattern(
    pattern material containing HoleRef(T) and unresolved Option::std
  ),
  ...
}
```

`T Option::std` is annotation-pattern material, not a runtime expression or
ordinary call. v0.3 does not look up `Option::std`, does not check kind/type
validity, and does not decide whether the annotation pattern is admissible.

Closure head:

```text
<T: type>(val: T) => { ... }
```

normalizes structurally as:

```text
ClosureHead {
  deduce: [
    HoleDecl {
      name: T,
      annotation: AnnotationPattern(type)
    }
  ],
  params: [
    BindingSlot {
      value_pattern: Binder(val),
      annotation: AnnotationPattern(HoleRef(T))
    }
  ],
  body: ...
}
```

A more complex parameter annotation:

```text
<T: type>(val: T Option::std) => { ... }
```

normalizes as:

```text
ClosureHead {
  deduce: [
    HoleDecl {
      name: T,
      annotation: AnnotationPattern(type)
    }
  ],
  params: [
    BindingSlot {
      value_pattern: Binder(val),
      annotation: AnnotationPattern(
        pattern material containing HoleRef(T) and unresolved Option::std
      )
    }
  ],
  body: ...
}
```

These shapes are structural only; no type semantics are assigned.

### 8.4 Annotation keeps a dedicated normalized annotation wrapper

Annotations are not normalized into ordinary expressions. They retain a
dedicated normalized annotation wrapper, e.g.
`NormAnnotation::Pattern(NormPattern)` or a semantically neutral equivalent such
as `AnnotationPattern(...)` / `ClassifierPattern(...)`. The exact Rust name is an
implementation detail; the spec-level decision is:

```text
annotation position is pattern position, not ordinary expression position.
```

The internal pattern material is recursively normalized as pattern structure,
but no name resolution, kind checking, type checking, canonical matching, or
classifier interpretation is performed in v0.3.

### 8.5 Canonical skeletons are preserved as a normalized pattern subform

v0.4 is not required to fully decompose canonical skeletons into primitive
pattern nodes. Canonical skeletons normalize into a preserved normalized pattern
subform, e.g. `NormPattern::Skeleton(...)`, not into a semantic matching
structure. v0.3 does not perform canonical matching, admissibility checking,
type-directed decomposition, or semantic pattern interpretation.

A reasonable minimal pattern-role family for v0.4 (roles, not final Rust names):

```text
NormPattern::Binder
NormPattern::Product
NormPattern::Unit
NormPattern::HoleRef
NormPattern::AnnotationMaterial / Nav / Name / Literal as needed
NormPattern::Skeleton
NormPattern::Error
```

Record the required roles, not the final enum spelling.

### 8.6 with-clause remains an unresolved name list

For v0.3, `with { ... }` remains structural and is preserved as an unresolved
list of names on the normalized binding site:

```text
with { names... } -> unresolved with-name list
```

It is not dependency injection, lifetime relation, capability import, ownership
permission, effect requirement, or namespace import.

### 8.7 No general symbolic builtin node family in v0.3

v0.3 does not introduce a general symbolic builtin node family. Do not introduce
nodes such as:

```text
BuiltinCall(MemberLookup)
BuiltinCall(OperatorCall)
BuiltinCall(PatternBind)
```

Generated material from lowering is represented as a generated unresolved name /
nav / operator target carrying origin/provenance (§7.15), not as a semantically
privileged builtin call. Future phases may introduce semantic builtins if
needed; v0.3 Normalized AST does not. (This is the v0.3 boundary answer to
N-AST-4.)

### 8.8 Alias declaration remains a declaration, not an expression

```text
let binder === EntityRef
=> unresolved normalized alias declaration
```

The alias RHS remains `EntityRef`. It is not normalized as ordinary expression,
product, call, closure, or runtime value (confirms §7.13). Alias target
resolution, alias scope semantics, operator alias identity validation, and
namespace resolution are later phases.

### 8.9 Closure normalization boundary

```text
Closure body forms are recursively normalized.
Closure head is structurally normalized into binding-site / slot / clause shapes.
Closure materialization does not occur.
Capture analysis does not occur.
require / pre / post / lifetime clauses are preserved structurally, not interpreted.
```

A headless in-place closure still has no implicit unit input. Generated closures
introduced by member / double-dot / prefix-negative lowering follow the same
structural boundary (confirms §7.12).

### 8.10 Error representation is family-local

v0.4 prefers family-local error variants rather than a single universal error
node:

```text
NormExpr::Error
NormPattern::Error
NormDecl::Error / NormForm::Error
```

The exact Rust variant names are implementation details. The spec-level decision
is that error recovery stays in the syntactic family where the error occurred,
so surrounding normalization can continue without forcing every error into one
universal node category.

### 8.11 Minimum normalized dump policy for v0.4

This closes N-AST-3 enough for v0.4 to begin. It does not design a final dump
format. The minimum requirement:

```text
v0.4 must expose a stable normalized dump entry point.
The dump must be stable enough for golden tests.
The dump must not use raw Rust Debug output as the public golden format.
The dump must show enough structure to verify:
  - source-product continuation and product merge;
  - first and second legality repairs;
  - product / group lifting boundary;
  - operator lowering and provenance summary;
  - member / double-dot / bracket-call lowering;
  - closure body recursive normalization;
  - alias declaration preservation;
  - annotation pattern and DeduceList structure;
  - error recovery placement.
```

The CLI spelling is an implementation choice. Acceptable options include
`lang norm <file>` or `lang ast --normalized <file>`, consistent with the
existing `lang tokens | ast | diag` structure. The final command name and dump
format are v0.4 implementation details.

## 9. Explicit non-goals

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

## 10. Status

§3 and §4 boundaries are defined and stable. §7 records the v0.3 source-product
continuation call-skeleton design decision (product merge, the priority order
and legality repairs, product normalization, operator lowering,
member/double-dot/bracket-call lowering, the closure and alias boundaries,
branch-name expansion, and the origin/provenance requirement). §8 records the
minimum Normalized AST shape needed to start v0.4: group non-survival, the
unified binding-site structure, the DeduceList / annotation-pattern split, the
annotation wrapper, canonical-skeleton preservation, the with-name list, the
no-general-builtin boundary, the alias and closure boundaries, family-local
errors, and the minimum normalized dump policy.

The §6 work items are now recorded as design decisions, and the §5 /
open-questions statuses are updated accordingly: N-AST-2/6 resolved; N-AST-4/8
resolved for the v0.3 boundary; N-AST-3/7 resolved for v0.4 start; N-AST-1/5
partially resolved; N-AST-9 adopted.

After this closeout, v0.3 has enough structural direction to start v0.4 Raw AST →
Normalized AST implementation. This does not mean the full language semantics are
specified. v0.4 may refine the exact Rust node shapes, origin representation, and
dump formatting as implementation feedback arrives, as long as it preserves the
v0.3 structural decisions recorded in §7 and §8.
