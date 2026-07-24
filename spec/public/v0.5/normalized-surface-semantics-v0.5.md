# Normalized Surface Semantics v0.5

> **Status:** Published. The v0.5 public normalized surface semantics are
> complete. §1–§7 define call / product / pipe binding; §8–§10 define
> value-side / pattern-side / annotation / alias boundaries; §11 defines
> origin / generated / derived / unsupported visibility; §12–§13 define the
> non-goals and the v0.6+ future-boundary separation. Every example below is
> cross-checked against the v0.4 normalized dumps in `tests/cases/norm/`.

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

## 7. Operator / Dot Closure / Member / Double-Dot / Bracket Sugar

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

### First-class dot closure and compact member sugar

```text
Conceptual rule: dot-closure lowering
Dump label:      DotClosureLowering
```

```text
.field
=> generated closure:
   <T: type>(val: T, ...args) { (val, args) |> field::T }

obj.field
=> obj |> .field

let d = .field
BindingShape(P1 |> .field P2)
== BindingShape(P1 |> d P2)
```

`.field` is independently usable and does not capture a left-hand receiver.
Raw `MemberSugar(obj, field)` may preserve the compact source shape, but its
normalized target is the same generated in-place `NormClosure` carrier.
`MemberLowering` records that compact wrapper; it does not define a second
member semantic system.
After atom lowering, `.field` is an ordinary `NormExpr`. Its generated origin
cannot change pipe/product association, absorb following items, bypass
first-product-only, or replace legality repair. Compact `obj.field`
mechanically produces `obj |> .field`, then returns that ordinary expression
to the existing suffix and space-binding environment.

“First-class expression” does not mean “eagerly materialized value.”
Normalization creates only the carrier above. A later explicit binding or call
context may materialize it; another expression context preserves/composes the
carrier without allocating a function object or capture environment.

### Narrow structural member-view annotation

The v0.6 owner/view amendment adds one exact postfix Raw shape:

```lang
E name [[public]]
E name [[private]]
```

The parser recognizes only those two complete annotations, so ordinary bracket
calls such as `obj[[cap] => { cap }]` and `obj[[strategy]]` retain their prior
shape. Normalization encodes the suffix through ordinary call structure with
`MemberViewAnnotationLowering`; it does not assign general policy or member
semantics. Only the later `struct` consumer interprets it as structural member
metadata. The slot is not available for `runtime`, `seal`, `const`, `export`,
or arbitrary names.

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

In the generated closures, `T`, `val`, and dot-closure `args` are local
generated binders, and the receiver becomes the call's source product (a
`ProductLift`). `...args` is a Pattern remainder binding, not a pack type.

```text
`field::T` and `method::T` are unresolved navigation targets.
No field lookup, method lookup, method dispatch, type checking, or overload resolution occurs.
```

## 8. Value-Side vs Pattern-Side Material

Value and pattern are different kinds of material and do not implicitly convert.

```text
A value does not implicitly become a pattern.
A pattern does not implicitly become a value.
```

A value enters pattern space only through an explicit bridge in a later phase
(for example postfix `?` or another explicit value-to-pattern operation; that
operation's semantics are v0.6+ and are not defined here). A pattern exposes
values only through explicit extraction, binding, passing, or returning.

What each side is:

```text
A Value is the object being processed.
A Pattern is the structural / extraction-side material used to decompose,
classify, or bind material.
```

At the normalized layer, whether a value can be expanded, whether a field
exists, or whether an extractor applies is not decided.

Separation in the normalized tree:

```text
Value-side material remains NormExpr.
Pattern-side material remains NormPattern.
```

Expression-shaped syntax may appear in a binding, annotation, or extraction
context. When it does, it normalizes into pattern-side material:

```text
Raw syntax may look expression-shaped, but if it occurs in a pattern,
annotation, or extraction context, it normalizes into NormPattern-side material,
not NormExpr.
```

The dump labels make this visible. The same source name is a different node on
each side:

```text
value position:       Name "P"          (NormExpr)
annotation position:  PatternName "P"   (NormPattern)
```

Pattern-side names are bounded pattern material, not ordinary call targets:

```text
Pattern-side names are unresolved pattern material.
They are not ordinary call targets.
They must not fall back to ordinary value/function lookup.
```

This applies to annotation patterns, binding patterns, extraction skeletons, and
future pattern-head positions.

### Construction/extraction vs call/extraction

```text
Construction and extraction may be isomorphic.
Call and extraction are not isomorphic.
```

A call returns a value. Extraction operates on the returned value's structure,
not on the call target or call history. For example, in expression position:

```text
((a P1), b P2)
```

`P1` and `P2` are value-side expression-chain / target material; they are not
pattern names. Even if `a P1` returns a value that is later extractable,
extraction sees the returned value's structure, not `P1` as a pattern head.

Contrast with annotation / pattern position:

```text
T Option::std
```

Here `T` and `Option::std` are pattern-side material (see §9), not value-side
call material.

The normalizer does not perform pattern-head resolution, extraction
applicability checks, exhaustiveness checking, or residual propagation. Those
are v0.6+ (see §13).

## 9. Annotation Patterns and DeduceList Holes

### DeduceList is a binding-site hole binder list

```text
DeduceList is not the annotation pattern itself.
DeduceList is not the value/extraction pattern itself.
DeduceList is a binding-site hole binder list.
```

A DeduceList (`<...>`) may occur on let binding slots, closure heads, parameter
slots, and other binding-site structures. A hole it declares may appear inside
the annotation pattern of the same binding site.

```text
Conceptual rule: binding-site / pattern normalization
Dump label:      PatternNormalize
```

Each declared hole is a `HoleDecl` in the binding site's `deduce` list; a use of
that hole inside an annotation is a `HoleRef`:

```text
HoleDecl   { id: HoleBinderId, spelling, annotation, origin }
HoleRef    { target: HoleBinderId, spelling, origin }
```

A `HoleDecl` may itself carry an annotation pattern (e.g. `<T: type>` declares
`T` with annotation `PatternName "type"`).

DeduceLists elaborate as left-to-right dependent telescopes. If
`<A1: T1, ..., An: Tn>` occurs with inherited hole environment `Γ0`, then
`Ti` is interpreted in `Γ(i-1)` and only afterwards is `Ai` added. Therefore
`<A, B: A>` refers to the preceding `A`, while `<A: B, B>` does not resolve the
first annotation to the later `B`. A declaration is not visible in its own
annotation. Hole spellings must be unique within one `PatternRoot`; an
independent `let` Pattern or nested callable head starts a new root and may
lexically shadow an inherited spelling. `HoleBinderId`, rather than the
display spelling, records the exact declaration targeted by a `HoleRef`.

A callable head DeduceList scopes the callable's capture slots and
initializers, parameters, call policy, return slot, head clauses, and complete
body. Nested callables inherit that active hole environment, allocate a new
callable semantic owner and `PatternRoot`, and then extend the environment with
their own DeduceList. Ordinary value binders occupy a separate lexical
environment and do not change Pattern-context hole identity.

Within one BindingSlot, source order remains:

```text
Policy -> let -> DeduceList -> Pattern -> Annotation -> Initializer
```

The policy sees inherited holes only. The slot-local DeduceList then extends
the environment for the following Pattern, annotation, and initializer; it
does not retroactively bind a name in the leading policy.

Return clauses keep ordinary let-shaped BindingSlot order:

```lang
let f = <A>(x: A) -> r: A => {
    let y: A = x;
    y
};
```

`r` is the explicit symbol bound to the returned object. `A` is its postfix
annotation Pattern. Thus `r` denotes a value for a value-returning callable
and a type/Pattern object for a callable whose result has that rank; parser
syntax does not preclassify the binder as a type. Use `-> _: A` when no result
symbol is bound. A prefix-shaped `-> A r` remains an extraction Pattern; it is
not a type annotation on `r`.

Raw AST preserves spelling, lexical scope shape, and provisional canonical
roles. A distinct alpha-normalization step after structural normalization
allocates callable owners, `PatternRoot` boundaries, and root-local binder
ordinals, then rewrites scoped Pattern/policy occurrences to exact
`HoleBinderId` targets. Source spans are provenance, never semantic hole
identity; alpha-equivalent binder/ref structures are independent of source
spelling and byte offset. The frontend identity is collision-safe across root
normalizations and carries:

```text
AlphaOwnerId × NormSemanticOwnerId × PatternRootLocalId × HoleLocalId
```

When material enters the build graph, the frontend callable owner is mapped to
a persistent `SemanticOwnerId`; the cross-pass identity becomes
`SemanticOwnerId × PatternRootLocalId × HoleLocalId`. Nested callable bodies
inherit outer references but allocate a distinct callable owner and Pattern
root for their own head.

Compiler-generated receiver holes carry a hygienic generated key before alpha
conversion. They are not entered in the source-spelling redeclaration table,
so a generated display name `T` cannot collide with a user-written `<T>`.
Generated Pattern/policy references follow the hygienic key to that binder.

Ordinary value-side `NormExpr::Name` and ungrouped navigation-name components
remain unresolved. Callable-wide hole scope is propagated through the whole
callable, but this Norm pass assigns exact identities only to Pattern/policy
occurrences. A later resolved-symbol pass owns value-side name/navigation
identity, including generated navigation such as `field::T`.

The anonymous annotation placeholder `_` is `AnonymousHole`; it is not a named
`HoleRef` and has no `HoleBinderId`.

### Annotation is pattern-side / classifier-pattern material

```text
Annotation is not an ordinary runtime expression.
Annotation is not call syntax.
Annotation is normalized through a pattern-side path (AnnotationPattern).
```

Inside an `AnnotationPattern`, names and navigation normalize to pattern-side
nodes by these rules:

```text
A name declared by the binding-site DeduceList   -> HoleRef
A name not declared by the DeduceList            -> PatternName (not NormExpr::Name)
Navigation (e.g. Option::std)                    -> PatternNav (not value-side Nav)
A sequence of annotation terms                   -> PatternSequence of the above
```

Examples (verified against `tests/cases/norm/`):

```text
let <T> x: T = y
  deduce:      HoleDecl "T"
  value:       Binder "x"
  annotation:  AnnotationPattern( HoleRef "T" )
  initializer: Name "y"            // value-side NormExpr

let <T> z: T Option::std = y
  annotation:  AnnotationPattern( PatternSequence[ HoleRef "T", PatternNav["Option","std"] ] )

let <T> x: U = y
  annotation:  AnnotationPattern( PatternName "U" )      // U undeclared -> PatternName

let <T> x: U Option::std = y
  annotation:  AnnotationPattern( PatternSequence[ PatternName "U", PatternNav["Option","std"] ] )
```

Closure head example (head dump label `ClosureNormalize`):

```text
<T: type>(val: T) => { val }

Closure placement=Ordinary
  head: ClosureHead
    deduce:
      HoleDecl "T" with annotation AnnotationPattern( PatternName "type" )
    params:
      BindingSlot "val" with annotation AnnotationPattern( HoleRef "T" )
  body: NormBody          // recursively normalized as forms/expressions
```

`type` and `T` here are not runtime expressions.

### Extraction skeletons and product extraction

Binding patterns may be product extraction or canonical skeletons; both remain
pattern-side structures preserved for later checking:

```text
(x, y)        -> PatternProduct[ Binder "x", Binder "y" ]
(x,)          -> PatternProduct[ Binder "x", Unit ]
(,x)          -> PatternProduct[ Unit, Binder "x" ]
(x,,y)        -> PatternProduct[ Binder "x", Unit, Binder "y" ]
(x, ...rest)  -> PatternProduct[ Binder "x", Pack(Binder "rest") ]
_ Pair::std   -> PatternSkeleton( SkeletonWildcard, SkeletonNav["Pair","std"] )
T Pair::std   -> PatternSkeleton( SkeletonName "T" role=Hole, SkeletonNav["Pair","std"] )
```

```text
Product extraction shape and explicit Unit positions are preserved.
No totality check, pattern matching, extraction applicability check,
exhaustiveness check, or residual propagation occurs at normalization.
```

At most one `Pack` may occur at each normalized structural level; nested
levels are independent. The pack remains Pattern-side only. Normalization does
not create a pack value, variadic ABI class, runtime container, or RHS unpack
operator.

The common post-normalization `validate_normalized_patterns` pass enforces that
limit over all `NormBindingSlot` consumers: top-level and local `let`,
parameters, returns, annotations, and nested Pattern levels. Both `Product`
and `Sequence` count their direct pack children before recursive validation.
This pass is the sole authority for normalized-level pack cardinality. The
parser preserves all syntactically formed `BindingPatternAst::Pack` nodes and
reports only local syntax failures such as a missing inner Pattern; it neither
counts packs nor predicts normalized structural levels.

`normalize_program` remains available to dump recovered or invalid normalized
structure. Downstream build installation instead uses:

```text
normalize_and_validate_patterns
  -> PatternValidatedNormProgram
  |  PatternInvalidNormProgram
```

Only the Pattern-validated wrapper enters world harvesting. Its current proof
scope is deliberately exact:

```text
one Pack per normalized structural level
no bare Product as a Pack operand
no duplicate DeduceList hole in one PatternRoot
```

It does not prove order-sensitive Pack applicability, stable Pattern-head
identity, full matching support, or absence of recovered `NormExpr::Error`.

`Pack` is part of the general binding-pattern grammar. It is preserved in
every binding-slot context—ordinary/local `let`, product extraction, callable
parameters, callable return slots, and nested binding Patterns—not only in
parameter lists. Normalization does not assign those contexts their later
matching semantics.

Ellipsis may also occur as a direct child of a canonical Pattern Sequence:

```text
PatternSequence ::= PatternTerm*
PatternTerm     ::= "..." PatternPrimary | PatternPrimary

a ...x b       -> NormPattern::Sequence[a, Pack(x), b]
...(x, y)      -> Raw Pack(Product[x, y]), rejected after P normalization
...((x, y) pair)
                -> Pack(Sequence[Product[x, y], PatternName "pair"])
```

The prefix constructor consumes one immediate Pattern primary, not the rest of
the Sequence. A bare Product supplies no stable top mode after P normalization,
so `Pack` cannot make that flattened boundary semantic again. The parser keeps
`...(x, y)` for recovery and auditing, but the normalized Pattern validator
rejects it. An ordered layer may later admit a structured operand only when its
P-normal form retains a stable top mode, as in `...((x, y) pair)`. At an
unordered named layer, only a whole-remainder binder/discard (possibly under a
transparent let-shaped slot) is admissible. Order and stable-top-mode checks
belong to later resolved Pattern semantics.

`Pack` and `BindingSlot` are transparent to the normalized-level cardinality
rule; only Product and Sequence establish structural levels.

Every Pack contributes exactly one outward specificity node at its containing
level:

```text
...rest -> one explicit-Pack node
..._    -> one Pack-discard node
...Q    -> one outward Pack position
```

Captured width and the number of nodes inside `Q` never create more same-level
Pack evidence. For `...((a, b) pair)`, evidence for `pair` and its inner
structure belongs to the preserved next level, not to two flattened EP nodes.

There is no type checking, kind checking, Pattern-head resolution, or general
matching at normalization. `Option::std` / `Pair::std` are not resolved, and
whether `T Option::std` is a legal type pattern is not decided.

### Policy pair preservation

Binding prefixes and callable-head P2 positions normalize to:

```text
NormPolicySpec {
  value_policy: NormValuePolicyPattern,
  pattern_policy: Option<NormPolicyConjunction>
}

NormPolicyConjunction { choices: Vec<NormPolicyChoice> }
NormPolicyChoice { atoms: Vec<NormPolicyAtom> }
```

The single-component and `value:Pattern` pair shapes are preserved. `||`
choice and `+` conjunction remain different normalized nodes; Pattern `|` is
not lowered into either. The explicit absent-value atom reserves
pure-type/value-optional elaboration but has no frozen source token.
Normalization does not decide whether a single
component is P1 value-dominant projection or P2 shorthand, validate pair stage
rules, or interpret const/mut/namespace atoms. Those are semantic policy
elaboration in `design/symbol-world/symbol-policy-and-compile-flow-projection.md`.

### Capture binding elaboration

An ordinary closure capture clause is a list of let-shaped bindings:

```text
CaptureClause ::= "[" CaptureItem ("," CaptureItem)* "]"

CaptureItem
  ::= PolicySpec "let" BindingCore "=" Expr
   |  "let" BindingCore "=" Expr
   |  BindingCore "=" Expr
   |  Expr
```

The first three forms are explicit captures. `let` may be omitted when no
policy prefix needs to be anchored. They reuse the ordinary `BindingSlot`
surface and normalization path; form-level alias `===` is not imported into a
capture item.

The final form is shorthand. Normalization first forms the ordinary call
structure of `Expr`, then collects its distinct free bare names whose concrete
occurrences are not direct callable targets. Exactly one distinct name `n`
elaborates the item as `let n = Expr`; zero or multiple candidates produce a
retained normalization error. Call-target role is local to each call node and
duplicate occurrences of the same text count once:

```text
[x]                 -> [let x = x]
[x x]               -> [let x = x |> x]
[x y z]             -> [let x = x y z]
[(x, x) |> x]       -> [let x = (x, x) |> x]
[(x, y) |> z]       -> inference error
[(1, 2) |> make]    -> inference error
```

Nested closure parameters, local lets, Patterns, and capture binders do not
pollute the outer inference set. All initializers in one capture clause are
interpreted in the enclosing pre-capture environment, so capture bindings are
simultaneous rather than a sequential let block.

After normalization every capture has one shape:

```text
NormCapture {
  slot: NormBindingSlot,
  initializer: NormExpr,
  origin: NormOrigin
}
```

This is syntax-directed capture binding elaboration, not closure environment
layout, name resolution, or materialization.

These source-written captures are explicit requirements. In particular,
`[x]` means explicit `[let x = x]` with the ordinary unwritten capture policy
domain (`const || mut`); it is not an automatic const capture. A later resolved
stage may add an `ImplicitConst` requirement for an otherwise uncaptured free
outer value reference. That later operation requires symbol resolution and
const-slice projection and therefore is not normalization.

In later resolved semantics, external explicit navigation searches the export
view, while internal explicit navigation searches the complete namespace view.
Declaration-side `P1Projection` first applies to actual RHS/result entries and
produces a resolved internal `PolicyPair`. Only that complete pair may become
an external candidate policy: its value component is const-projected and its
associated Pattern component is preserved. Export-retention-closure membership
admits only the graph-retention dimension; external exposure additionally
requires every path component to be publicly reachable. A private child and
public descendants behind it therefore remain internal even inside the
retention closure. Among admitted symbols, mut-only overload candidates remain
in the complete internal set and are omitted from the external overload set.
Pure `absent:Pp` candidates enter unchanged, subject to the structural
invariant that absent Pv has neither value stages nor value mutability. Thus
`const + S : compile`, `mut + S : compile`, and their `export` forms are
invalid. A direct source `export + mut` root remains an invalid declaration.

Value-bearing export views are therefore const-projected, so dependencies reached with
external authority—including ordinary external call targets—normally satisfy
automatic const-capture requirements. Automatic capture and call resolution
therefore touch the same problem domain—symbol identity and readable const
view—but this does not prescribe pass ordering, data flow, or an implementation
dependency.

Explicit and automatic capture remain distinct even when they resolve to the
same source symbol. Explicit capture may rename, project policy, use a complex
initializer, request `mut`, and preserve distinct provenance. Parsing,
normalization, and capture discovery neither reject nor erase it as redundant.
A future layout pass may coalesce equivalent storage/link requirements only
while preserving binder identity, policy, and provenance.

### Semantic-owner and namespace-view handoff

The v0.6 build handoff derives long-lived identity from a parent-linked
`SemanticOwner`, not a file, span, or printable path. Every callable, including
an in-place closure, owns an anonymous `Self` type. Source navigation remains
inner-to-outer; a complete generated meta-call scope used as an outer
component is grouped, as in `child::(int Vec::std)`.

Each independent let Pattern and callable head establishes a `PatternRoot`.
Nested Pattern structure remains in that root. Same-root hole duplicates fail;
a different root may lexically shadow an inherited spelling.

Namespace consumers keep `FullNameView`, `ExternalNameView`, and
`DefaultExtractionView` separate. Package-boundary crossing selects the
external view; mount metadata redirects to an existing namespace without
copying symbol identity. Private structural members remain in the full
structural model but are omitted from default extraction.

Resolved capture requirements are abstract dependencies, not a declaration of
`self` fields, capture-by-value/reference representation, field order, ZST
status, or ABI layout. An ordinary closure that writes an outer place must have
an explicit capture able to project a `mut` view; automatic capture never grants
mutability. An in-place closure has no capture list or capture set, resolves
outer reads at its embedding layer, and may not directly write an outer place.

For example, an exported ordinary closure's source dependency is explicit:

```lang
mut let internal_state = ...;

export let exported_fn =
    [internal_state]() => {
        use internal_state;
    };
```

The dependency does not export `internal_state` and does not by itself require
an environment field. Before a callable is materialized, every resolved
capture requirement must lower to a lifetime-checkable source/access/storage
form. Concrete lifetime, borrow/move/copy, escape, layout, and ABI rules remain
future work.

### Callable implementation tail

Closure normalization preserves one of:

```text
Block(body)                         -> ordinary user body
NamedBlock(strategy, body)          -> named strategy + user body
Defaulted                           -> compiler-default implementation request
Delete(message: optional String)    -> selected-candidate rejection
```

`=> strategy { ... }` and the no-`=>` escape `[[strategy]] { ... }` normalize
to the same `NamedBlock`. The legacy-looking `() -> r name { ... }` is not
reinterpreted: `name` remains return extraction material. This layer preserves
strategy metadata but does not execute a strategy, synthesize a default body,
or perform overload selection.

Closure placement is orthogonal to head presence and implementation:

```text
{ ... }                              -> InPlace, head=None
() -> r name { ... }                 -> InPlace, head=Some
() -> r [[strategy]] { ... }         -> InPlace, head=Some
() -> r => { ... }                   -> Ordinary, head=Some
() -> r => strategy { ... }          -> Ordinary, head=Some
() -> r => default/delete            -> Ordinary, head=Some
```

`[[strategy]]` does not change placement. In-place closures cannot spell a
capture list; `[x] { ... }` remains an error. A malformed callable tail
normalizes as `NormExpr::Error`, never as a legal empty `Block`.

Product-versus-closure classification, and the decision to bypass an available
capture slot, recognize only the complete local tail shape `[[Name]] {`.
A DeduceList alone does not close the capture slot. Thus
`<T> [[cap] => { cap }] () => { value }` parses the bracketed closure as a
capture item, while `<T> [[strategy]] { value }` has the complete strategy
tail. A leading `[[` may be treated as a malformed strategy candidate only
after parameters, call policy, return syntax, or a head clause has independently
closed the capture slot and established the strong context.

Ordinary atom/operator postfix parsing does not exclude `[[`, so
`obj[[cap] => { cap }]`, `()[[cap] => { cap }]`, and
`(a + b)[[cap] => { cap }]` remain bracket calls with capture-closure
arguments.

After `=>`, `Name Block` is selected before the bare contextual forms.
Therefore `=> default { ... }` and `=> delete { ... }` are named strategy
bodies, while bare `=> default` and `=> delete` remain `Defaulted` and
`Delete`.

The message-bearing form is intentionally limited to a source string literal:

```text
=> ("message") delete
```

The historical `(message_expr) delete` surface is not retained in v0.5 because
delete messages are static compiler diagnostic text rather than evaluated
expressions.

Normalized closure placement and origin are separate fields:

```text
NormClosure.placement = InPlace | Ordinary
NormClosure.origin    = Source | Generated(rule) | Derived(rule)
```

Generated provenance never replaces placement. In particular the closure
generated for `.name` has `placement=InPlace` and
`origin=Generated(DotClosureLowering)`.

## 10. Alias Preservation

```text
Alias-let is declaration-side material, not expression-side call material.
```

The shape:

```text
let binder === EntityRef
```

normalizes as an unresolved alias declaration.

```text
Conceptual rule: alias preservation
Dump label:      AliasPreserve
```

The right-hand side remains an `EntityRef`. It is **not**:

```text
NormExpr
Product
PipeExpr
runtime equality
runtime assignment
import
operator call
```

Examples (verified against `tests/cases/norm/`):

```text
let A === B::C
  Decl Alias
    binder: Name "A"
    target: EntityRef[ "B", "C" ]

let + === Add::std
  Decl Alias
    binder: Operator "+"
    target: EntityRef[ "Add", "std" ]
```

The binder may be a `Name` or an `Operator`, and an optional `NormPolicySpec`
prefix is preserved. No alias target resolution, scope semantics, namespace resolution,
operator-alias identity validation, or runtime behavior occurs at the normalized
layer. (A hypothetical target such as `operators::plus` would be preserved the
same way; only forms covered by the parser / normalizer / golden tests are used
as primary examples here.)

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

Rule labels used by the call-binding, sugar-lowering, pattern, closure, and
alias examples in this document:

```text
Generated:
  ProductLift
  OperatorLowering
  PrefixNegativeLowering
  DotClosureLowering
  MemberLowering
  DoubleDotLowering
  BracketCallLowering
  PatternNormalize        (binding-site / annotation / extraction-pattern normalization; §9)
  ClosureNormalize        (closure head normalization; §9)
  AliasPreserve           (alias declaration + EntityRef preservation; §10)
  Unsupported             (node surfaced explicitly; origin Generated(Unsupported))

Derived:
  ProductMerge            (source-product continuation)
  PipeFallback            (first legality repair / ordinary expression-chain growth)
  SecondLegalityRepair    (second legality repair)
```

Pattern-side material surfaces failures explicitly rather than crossing the
value/pattern boundary: a `PatternUnsupported` node in an annotation or pattern
context (origin `Generated(Unsupported)`) records a boundary-preserving failure
to lower expression-like sugar as pattern material. For example,
`let x: obj.field = y` normalizes the annotation to
`PatternUnsupported "member sugar in annotation pattern"` rather than a
value-side member call (see §9).

This list is not guaranteed to be the complete Normalized AST rule-label
inventory, but it now covers the call-binding, sugar-lowering, and
value / pattern / alias material documented in this version.

These labels appear verbatim in the normalized dump, so any example in this
document can be cross-checked against `normalize_program` output and the golden
fixtures in `tests/cases/norm/`.

## 12. Non-Goals

The normalized surface does not perform name resolution, type/kind checking,
operator lookup, operator overload resolution, alias target resolution,
namespace resolution, pattern-head resolution, canonical matching, closure
materialization, capture-environment analysis, ownership/NLL/drop, effect
interpretation, runtime evaluation, or code generation. The syntax-directed
capture-name inference described above is not semantic environment analysis.
The normalized surface does not turn Normalized AST into HIR.

A source Product is never a conventional argument list. There is no callee-first
call, method dispatch, field lookup, resolved function call, operator overload
resolution, or ADL at the normalized layer.

Backing: `spec/contracts/v0.4-normalization-prototype-notes.md`.

## 13. Relation to v0.6+ Future Semantics

Later pattern-space and extraction-chain semantics
(`spec/design/patterns-overload/static-pattern-spaces-and-extraction-chains.md`) motivate the
value-side / pattern-side boundaries, but they are **not** current normalized
call-binding behavior. `Done`, residual propagation, pattern-space subtraction,
`operator+` meta-reduction, `match` closing, exhaustiveness, and pattern-head
resolution are future semantics, not current behavior, and must not be read as
implemented.

## 14. Control-Flow End Events

The normalized surface reports control-flow end events structurally.
These are not ordinary expressions or calls.

### TailValue

The last expression form in each body block is normalized as:

```text
NormForm::TailValue(NormExpr)
```

This is a block result / tail value. It is not early return. It
represents the implicit control-flow end of a body block when no
explicit return event is present.

### ReturnEvent

Explicit return terminal forms are normalized as:

```text
NormForm::ReturnEvent(NormReturnEvent)

NormReturnEvent {
  value: NormExpr,
  target: NormReturnTargetSyntax,
  origin: NormOrigin
}

NormReturnTargetSyntax ::=
    ImplicitNearest
  | Explicit(NormExpr)
```

| Source | Normalized Form |
|---|---|
| `E return;` | `ReturnEvent(value = E, target = ImplicitNearest)` |
| `E \|> (return);` | `ReturnEvent(value = E, target = ImplicitNearest)` |
| `E (return);` | `ReturnEvent(value = E, target = ImplicitNearest)` |
| `E \|> (T return);` | `ReturnEvent(value = E, target = Explicit(T))` |
| `E (T return);` | `ReturnEvent(value = E, target = Explicit(T))` |

`Explicit(NormExpr)` preserves unresolved target syntax. The
normalizer does **not** resolve `Self` or any other target
expression. Target resolution is deferred to a later elaboration
phase.

### Non-Call Representation

Return events are **not** represented as:

```text
✗ NormExpr::Call { target: Name("return"), ... }
✗ NormExpr::Call { target: OperatorTarget("|>"), ... }
✗ NormExpr::Pipe { lhs: E, rhs: Group(...) }
```

They are structurally distinct `NormForm` variants.

### Deferred Semantics

The following are **not** implemented in the current normalized
surface and must not be assumed:

- Target resolution (implicit or explicit)
- Return outside returnable context checking
- D-reduction / Done_Return
- Control-flow propagation
- Result-slot injection
