# Capture Binding, Callable Tail, Dot Closure, and Pack Pattern

**Status: canonical semantic design with typed parser/normalizer substrate.**
Named-strategy execution and compiler-provided default-body generation remain
strategy-specific later work; their syntax and semantic boundaries are fixed
here.

This note connects four thin surface forms to the existing function-object,
Pattern-normalization, and unique-overload-selection model. It does not change
`Pv:Pp`, the three execution phases, or the requirement that ordinary overload
resolution produce one result.

## 1. Callable implementation tail

The suffix after a headed callable is one **callable implementation tail**:

```text
CallableImplementationTail
  ::= "=>" Block
   |  "=>" Name Block
   |  "=>" "default"
   |  "=>" "delete"
   |  "=>" "(" StringLiteral ")" "delete"
   |  "[[" Name "]]" Block
   |  Block
```

`default` and `delete` are strong-context names, not lexer keywords. The last
two alternatives are the no-`=>` headed-closure forms.

The normalized semantic carrier is:

```text
ClosurePlacement
  = InPlace
  | Ordinary

CallableImplementation
  = UserBody {
      strategy: Ordinary | Named(Symbol),
      body: Block
    }
  | Defaulted
  | Deleted {
      message: OptionalString
    }
```

The source forms map as follows:

| Source tail | Normalized form |
|---|---|
| `=> { ... }` or `{ ... }` | `UserBody(Ordinary, body)` |
| `=> strategy { ... }` | `UserBody(Named(strategy), body)` |
| `[[strategy]] { ... }` | `UserBody(Named(strategy), body)` |
| `=> default` | `Defaulted` |
| `=> delete` | `Deleted(None)` |
| `=> ("message") delete` | `Deleted(Some(message))` |

`=> strategy { ... }` is the preferred spelling. `[[strategy]]` is the
explicit escape only where a no-`=>` tail would otherwise be ambiguous. `#`
has no overload-strategy role.

Placement, head presence, and implementation are independent dimensions:

| Source | Placement | Head | Implementation |
|---|---|---|---|
| `{ ... }` | `InPlace` | none | `UserBody(Ordinary)` |
| `() -> r name { ... }` | `InPlace` | present | `UserBody(Ordinary)` |
| `() -> r [[strategy]] { ... }` | `InPlace` | present | `UserBody(Named(strategy))` |
| `() -> r => { ... }` | `Ordinary` | present | `UserBody(Ordinary)` |
| `() -> r => strategy { ... }` | `Ordinary` | present | `UserBody(Named(strategy))` |
| `() -> r => default` | `Ordinary` | present | `Defaulted` |
| `() -> r => delete` | `Ordinary` | present | `Deleted` |

In particular, `[[strategy]]` disambiguates strategy metadata; it does not
change an in-place closure into an ordinary materializable closure. In-place
closures never have a capture list or an independent capture environment.
`[x] { ... }` is rejected. Their external reads instead use the lazy
embedding-layer lookup defined by the function-object model.

### 1.1 Strong-context boundary

The lexer continues to emit ordinary bracket tokens. Product-versus-closure
classification recognizes only a complete strategy tail:

```text
starts_closure_head_continuation
  = :
  | ->
  | =>
  | {
  | head-clause
  | [[Name]] {
```

The weaker prefix `[[` is a malformed-strategy candidate only after some other
post-capture head syntax has already proved the closure-tail context. A
DeduceList alone leaves the capture slot open and therefore does not suffice.
The weak prefix may improve error recovery after parameters, call policy,
return, or a head clause, but it cannot classify an ambiguous Product or
disable ordinary bracket-call suffix parsing.

Therefore all of the following remain ordinary bracket calls whose argument is
a capture closure:

```lang
obj[[cap] => { cap }]
()[[cap] => { cap }]
(a + b)[[cap] => { cap }]
```

A complete `[[Name]] {` tail is excluded from capture-clause parsing. This
keeps `() [[s]] { ... }` and `<T> [[s]] { ... }` on the strategy path, while:

```lang
<T> [[cap] => { cap }] () => { value }
```

continues through the capture slot because the tokens after the inner capture
closure do not form `[[Name]] {`.

### 1.2 Tail selection uses complete local shape

After `=>`, implementation selection is:

```text
Block                         -> UserBody(Ordinary)
"(" StringLiteral ")" delete -> Deleted(message)
Name Block                    -> UserBody(Named(Name))
default without Block         -> Defaulted
delete without Block          -> Deleted(None)
other Name without Block      -> Error
```

`Name Block` precedes the two bare contextual names. Consequently:

```lang
=> default { ... }
=> delete { ... }
```

are named strategy bodies. `default` and `delete` remain weak `Name` tokens;
their special implementation meaning exists only when they are not followed
by a block.

### 1.3 No rollback over the return extraction pattern

The established form remains unchanged:

```lang
() -> r name {
    ...
}
```

`name` is part of the extraction pattern on `r`. The parser must not see the
following block and backtrack to reinterpret `name` as strategy metadata. A
named strategy in this no-`=>` form is written explicitly:

```lang
() -> r [[strategy_name]] {
    ...
}
```

### 1.4 Implementation form and strategy are orthogonal

`UserBody`, `Defaulted`, and `Deleted` occupy the same tail slot but do not
mean the same operation:

- `UserBody` supplies source implementation.
- `Defaulted` asks the compiler-known rule for that callable kind to synthesize
  an implementation. The spelling alone grants no overload priority.
- `Deleted` remains a real candidate. If it is uniquely selected, evaluation
  produces its specific rejection diagnostic.

A named strategy is static candidate metadata:

```text
Candidate
  = head
  x parameter Patterns
  x result policy
  x overload strategy
  x implementation
```

It is applied only after the fully admissible set `A` exists:

```text
ResolveSymbol
  -> EnumerateValueObjects
  -> ExposePhaseViews
  -> ProjectExpectedPolicy
  -> FullyAdmissible
  -> ApplyStrategyAndPartialOrders
  -> UniqueMaximum
```

A strategy cannot make an inapplicable candidate admissible, read a runtime
value to decide a static candidate relation, erase the ordinary status of a
delete candidate, or reopen candidate enumeration after unique ordinary
selection. The strategy named by source must denote a separately specified
monotone comparison/organization rule; unknown or inapplicable strategies are
diagnostic-bearing, not silently Ordinary.

## 2. Capture clauses elaborate to let-shaped bindings

The capture surface is:

```text
CaptureClause ::= "[" CaptureItem ("," CaptureItem)* "]"

CaptureItem
  ::= PolicySpec "let" BindingCore "=" Expr
   |  "let" BindingCore "=" Expr
   |  BindingCore "=" Expr
   |  Expr
```

The first three alternatives reuse the complete ordinary binding-slot shape.
`let` may be omitted when no policy prefix needs it as an anchor:

```lang
[let x = E]
[x = E]
[runtime let x = E]
```

`===` alias binding remains a form-level declaration and is not added to the
capture grammar.

All successful forms normalize to:

```text
NormCapture {
  slot: NormBindingSlot,
  initializer: NormExpr,
  origin: NormOrigin
}
```

There is no naked normalized capture expression.

### 2.1 Strict shorthand inference

For shorthand `[E]`, let `N(E)` be the ordinary non-semantic normalized
expression and define:

```text
C(E) = {
  text(n)
  | n is a free bare Name occurrence in N(E)
  | n is not the callable target of its direct Call node
}
```

`C(E)` is a set of distinct name texts, not occurrences. Shorthand succeeds
exactly when `|C(E)| = 1`; if `C(E) = {n}`, `[E]` elaborates to
`[let n = E]`. Examples:

```text
[x]                -> [let x = x]
[x x]              -> [let x = x |> x]
[x y z]            -> [let x = x y z]
[(x, x) |> x]      -> [let x = (x, x) |> x]
[(x, y) |> z]      -> inference error
[(x, y) |> x]      -> inference error
[(1, 2) |> make]   -> inference error
```

Call-target role is local to each direct Call. A target does not become a
non-call occurrence merely because its call result later becomes another
call's source. A name that occurs in both roles remains a candidate because
of its non-call occurrence.

Only free names participate. Parameters, local let binders, capture binders,
and other nested binding Patterns do not pollute an outer shorthand.
This analysis requires no symbol resolution: it consumes only the normalized
call spine, local binders, and bare name text.

### 2.2 Initializer scope is simultaneous

In `[let x = E]`, `E` is interpreted in the environment before the capture
binding. For:

```lang
[let x = E1, let y = E2]
```

both `E1` and `E2` see the same enclosing environment. The second initializer
does not automatically see the first capture.

The nested case is therefore recursive but unambiguous:

```lang
let f = [[cap] => { cap }] () => { value };
```

The outer shorthand elaborates to:

```lang
[let cap = [let cap = cap] => { cap }]
```

Each initializer's `cap` denotes its own pre-capture enclosing binding.

## 3. `.name` is a first-class field-function closure

The semantic atom is the leading-dot expression itself:

```lang
.name
```

Raw AST preserves it as `DotClosure(name)`. It normalizes independently of any
receiver to:

```lang
(val: T, ...args) {
    (val, args) |> name::T
}
```

Normalization produces an in-place `NormClosure` carrier. `T` is inferred from
the first formal argument only when an explicit call context consumes and
materializes that carrier. It is not captured from a syntactic expression to
the left of the dot. A binding context may also materialize the carrier; other
expression contexts merely preserve or compose the closure expression.

The compact suffix is defined through that same atom:

```text
E.name
  == E |> .name
```

It is not a second field-access semantic node. Raw AST may retain
`MemberSugar(E, name)` for source fidelity, but normalization must use the same
`DotClosure(name)` core.

After that one lowering, the generated closure is an ordinary `NormExpr`.
No pipe/product rule may inspect `DotClosureLowering` provenance to decide how
nearby material binds:

```text
let d = .name

BindingShape(P1 |> .name P2)
  == BindingShape(P1 |> d P2)
```

The equality is about the general pipe/product/call spine; the leaf retains
its own symbol identity and provenance. `.name` does not decide whether a
following item becomes an argument, how many following items are absorbed,
where a target expression ends, or whether first-product-only and legality
repair apply. Those decisions belong exclusively to the existing expression,
pipe, and product normalizer.

Compact `MemberSugar(E, name)` mechanically lowers its compact core to
`E |> .name` and then returns that result to the same ordinary suffix and
space-binding environment. Thus `E.name P` is interpreted exactly as placing
the ordinary result of `E |> .name` back before `P`; there is no second compact
dot call algebra and no explicit-pipe DotClosure privilege.

`..name(product)` remains a distinct direct member-call sugar. It models a
receiver-position call directly and need not first expose and then materialize
a `.name` closure carrier. Neither form removes the other:

```text
.name    first-class field-function closure
..name   direct member-call sugar
```

## 4. `...` is a Pattern remainder matcher

Ellipsis is structural only on the left/pattern side:

```text
Pattern ::= ExistingPattern | "..." Pattern
```

The normalized node is:

```text
Pattern::Pack(inner_pattern)
```

Its meaning is: take the part of the current normalized structural level that
ordinary sibling nodes have not matched, normalize that remainder as an
ordinary product, then match `inner_pattern` against it.

Because this is a general binding Pattern constructor, it is accepted anywhere
the language admits a `let`-shaped binding slot, not only in callable
parameters. This includes ordinary and local `let`, product extraction,
callable parameters, return slots, and nested binding Patterns:

```lang
let ...rest = value;
let (head, ...rest) = value;
let f = (...args) -> ...result => { ... };
```

These positions share syntax/normalization only. Their later semantic consumer
(ordinary binding, argument matching, or return-result matching) determines
what the remainder is relative to; no parameter-only pack object is created.

`...args` binds that remainder to the ordinary symbol `args`. It does not
construct a new pack value kind, type kind, ABI class, or runtime container.

### 4.1 Canonical Sequence children

Ellipsis is a prefix Pattern constructor over one primary:

```text
PatternSequence ::= PatternTerm*
PatternTerm     ::= "..." PatternPrimary | PatternPrimary
```

Therefore:

```text
a ...x b   -> NormPattern::Sequence[a, Pack(x), b]
```

It does not become `Pack(Sequence[x, b])`. A compound operand requires an
explicit primary boundary such as `...(x, y)`. Canonical sequence Pack nodes
live in `NormPattern`; they are never hidden inside `NormSkeleton`.

### 4.2 Ordered and unordered levels

At an order-insensitive named level, ordinary siblings match their names first
and the Pack receives the unmatched siblings. At an order-sensitive level, the
Pack receives the ordinary prefix/suffix remainder. Its inner Pattern then
matches that remainder as normal structure.

In particular:

```text
...(a, b)
  -> Pack(Product[Binder(a), Binder(b)])
```

This is one Pack constructor with a structured operand. The product constrains
the captured remainder to the structure expected by `a` and `b`. Pack-operand
context is propagated through this Product so these names retain the same
binding meaning as direct `...a` and `...b`; it is not an opaque binding of the
entire remainder to one symbol.

### 4.3 One pack per normalized level

For every normalized structural level `L`:

```text
count(child in L where child is Pack) <= 1
```

Only Product and Sequence create structural levels. Pack and BindingSlot are
transparent for this rule, so `Pack(Pack(x))` is rejected at one level.

The parser does not enforce this invariant. It preserves `(...x, ...y)` and
`......x` as complete Raw Pattern shapes and diagnoses only a missing operand.
The post-normalization Pattern validator is the single authority.

Nested Product/Sequence levels remain independent:

```lang
(a, (b, ...inner), ...outer)  // valid
(a, ...x, ...y)               // validator error
```

### 4.4 No unpack operator

There is no corresponding right-value spread syntax. The remainder bound to
`args` is already ordinary normalized Pattern/value product material. Existing
product normalization composes it in:

```lang
(val, args) |> name::T
```

Introducing `*args`, `unpack(args)`, or an RHS meaning for `...args` would add
a redundant second algebra and is outside this design.

## 5. Pack specificity evidence

Matching records four node classes:

```text
E   ordinary explicit match
EP  explicit pack match
D   ordinary fixed-node discard
DP  pack discard
```

At equal pre-existing structural-depth evidence, compare these counts in the
following lexicographic order:

```text
E > EP > D > DP
```

Thus an ordinary explicit node is more specific than an explicit pack;
`...args` is more specific than `..._`; and input length never manufactures
additional pack specificity.

An unstructured `...args` or `..._` has one inner evidence node. A structured
operand projects each of its explicit/discard nodes into the corresponding
pack class:

```text
...(a, b) -> specificity evidence equivalent to (...a, ...b)
...(a, _) -> one EP plus one DP
```

The first line means two explicit pack-match evidence nodes for the partial
order. It does not mean the syntax or AST contains two Pack constructors:
the AST remains `Pack(Product[a, b])`. Nor does either `a` or `b` gain evidence
from how many runtime elements the outer remainder happened to contain.
Nested/other structured operands extend by the same node-wise projection.

This tuple is only the Pattern-specificity preference dimension. It is not a
global score across stage, mutability, result policy, or named strategies.

## 6. Current implementation boundary

Implemented substrate:

- `Ellipsis`, `DotClosure`, `BindingPatternAst::Pack`, and the callable-tail
  Raw AST variants;
- let-shaped explicit/inferred `CaptureItemAst` variants and uniform
  `NormCapture { slot, initializer }`;
- normalized capture shorthand inference from one distinct free non-call bare
  name, including local-binder exclusion and simultaneous initializer scope;
- normalization to `NormPattern::Pack`, generated dot closures, named/default/
  delete implementation variants, and compact `E.name`;
- orthogonal `ClosurePlacementAst` plus optional head, preserving headed and
  headless in-place closures without granting them capture lists;
- orthogonal `NormClosurePlacement`; generated provenance remains exclusively
  in `NormOrigin::Generated`, so a generated dot closure is still in-place;
- one complete `[[Name]] {` closure-head continuation recognizer, plus a
  recovery-only `[[` candidate confined to independently proven closure heads;
- ordinary atom/operator bracket-call suffixes remain closed under capture
  closure payloads and are not disabled by strategy lookahead;
- full-shape callable-tail selection, with `Name Block` preceding bare
  `default`/`delete`;
- DotClosure substitution invariance: after atom lowering, ordinary
  pipe/product normalization cannot observe `DotClosureLowering` provenance;
- pack preservation in every parser binding-slot context (`let`, parameter,
  return, nested product extraction, and canonical Sequence), not a
  parameter-only grammar;
- global post-normalization one-pack-per-level validation over declarations,
  local bodies, parameters, returns, annotations, and nested binding slots;
- `normalize_and_validate_patterns -> PatternValidatedNormProgram` as the
  Pattern-layer build-world harvesting handoff; raw `normalize_program`
  remains available for diagnostic/recovery inspection, and the certificate
  does not claim recovery-free syntax;
- `AtomKind::Error` recovery for malformed callable tails, never an executable
  empty user body;
- restricted variadic applicability, remainder binding, direct structured
  product Pack matching, and pack node-class specificity evidence;
- named strategy metadata carried by selected restricted candidates only after
  applicability.

Not yet general implementation:

- executing arbitrary source-named overload strategies;
- resolving the compatibility `String` strategy carrier to stable `Symbol`
  identity (the normative carrier above is `Named(Symbol)`);
- compiler generation rules for every `Defaulted` callable kind;
- complete ordered/unordered Pattern matching over all future Pattern forms;
- full runtime overload resolution using these nodes.

The restricted evaluator diagnoses selected `Defaulted` bodies it cannot yet
materialize. It never treats that implementation gap as extra priority or as a
second overload pass.
