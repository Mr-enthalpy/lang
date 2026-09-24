# Capture Binding, Callable Tail, Dot Closure, and Pack Pattern

**Status: canonical semantic design with typed parser/normalizer substrate.**
Named-strategy execution and compiler-provided default-body generation remain
strategy-specific later work; their syntax and semantic boundaries are fixed
here.

This note connects four thin surface forms to the existing function-object,
Pattern-normalization, and unique-overload-selection model. It retains the internal value/type Policy observations, the three execution phases, or the requirement that ordinary overload
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
      strategy: Ordinary | Named(UnresolvedName),
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
change in-place placement into ordinary placement. In-place
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
ResolveNameBinding                     -- named-type projection case
  -> ReadNamedType
  -> CallCandidates
  -> ExposePhaseViews
  -> ProjectExpectedPolicy
  -> FullyAdmissible
  -> ApplyStrategyAndPartialOrders
  -> UniqueMaximum
```

For a path, resolution returns one NameBinding. In the named-type case above,
ReadNamedType reads its complete type before CallCandidates projects it. This
does not restrict ordinary Val2 reads to types. A held OverloadGroup G, including
a member value obtained by navigation, enters at CallCandidates(G); ordinary
function values use their exact type and associated (). See the
[value-navigation example](overload-resolution-design.md#21-value-navigation-is-broader-than-candidate-projection).
Only repeated exposure of the
same stable candidate-entry identity may collapse; distinct contribution entries
never deduplicate merely because their values or types normalize equally.

A strategy cannot make an inapplicable candidate admissible, read a runtime
value to decide a static candidate relation, erase the ordinary status of a
delete candidate, or reopen candidate enumeration after unique ordinary
selection. The strategy named by source must denote a separately specified
monotone comparison/organization rule; unknown or inapplicable strategies are
diagnostic-bearing, not silently Ordinary.

## 2. Explicit dependency clauses elaborate to let-shaped bindings

The [dependency owner](../symbol-world/dependency-observation-and-realization.md)
defines Needs and requirement/realization/layout. Capture is one surface consumer;
external values, types, host resources and actual late Path reads use the same
framework. This section owns the existing [] syntax and binder scope.

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
This analysis requires no name resolution: it consumes only the normalized
call spine, local binders, and bare name text.

### 2.2 Initializers share a pre-capture name environment

In `[let x = E]`, `E` is interpreted in the environment before the capture
binding. For:

```lang
[let x = E1, let y = E2]
```

both `E1` and `E2` see the same enclosing environment. The second initializer
does not automatically see the first capture. This does not mean simultaneous
or unordered effects. Ordinary formation fixes evaluation/effect order, with one
execution per reached formation occurrence; projections and body calls do not
rerun initializers.

The nested case is therefore recursive but unambiguous:

```lang
let f = [[cap] => { cap }] () => { value };
```

The outer shorthand elaborates to:

```lang
[let cap = [let cap = cap] => { cap }]
```

Each initializer's `cap` denotes its own pre-capture enclosing binding.

### 2.3 Explicit capture is not automatic capture

Every source capture item is explicit, including shorthand whose binder is
inferred:

```text
[let x = E]  -> Explicit
[x = E]      -> Explicit
[E]          -> ExplicitInferredBinder, if shorthand inference succeeds
```

In particular `[x]` elaborates to the explicit empty-policy binding
`[let x = x]`. Its omitted capture mode supplies no override; ordinary contextual
inheritance/completion applies, with no automatic const conversion.

For an ordinary closure, a resolved free reference can impose a Needs
requirement under its stable full/export namespace view. Eligible implicit
realization and explicit [] remain distinct declarations even for the same
source. Neither lookup nor requested Policy grants borrow/write authority.
Outer writes still require a write-capable explicit dependency.

### 2.4 Dependency realization precedes representation

Requirement, selected semantic realization and physical layout are three
different judgments. A snapshot and a live-place reference are not arbitrary
alternative layouts. Actual owned values enter ordinary structural identity;
references retain target/generation; no unobserved side table changes behavior.
Not every requirement needs a public self.Val2 field. Copying retains established
sources; type projections do not recapture runtime values. Full rules and the
lifetime refinement interface are in the dependency owner.

### 2.5 DeduceList scope construction and alpha normalization

The let-shaped slots reused by captures, parameters, returns, and nested
extraction may each carry a DeduceList. Recursive preservation alone is not a
scope rule. Normalized DeduceLists therefore elaborate as left-to-right
telescopes:

```text
rho0 = inherited visible holes
Ti is normalized under rho(i-1)
allocate fresh alpha identity hi for Ai
if Ai is not already declared in this PatternRoot:
    rho_i = rho(i-1)[Ai -> hi]
otherwise:
    retain an invalid binder for diagnostics and do not extend rho
```

Consequently:

```lang
<A, B: A>   // B.annotation targets A_id
<A: B, B>   // A.annotation does not target later B_id
<A: A>      // no self-reference unless an ancestor A was already active
```

Same-list duplicate names and redeclarations in nested BindingSlots of the
same `PatternRoot` are errors. An inherited spelling from a different
`PatternRoot` may be shadowed. A same-root duplicate declaration is retained
for diagnostics but does not shadow or extend the environment.

A BindingSlot preserves source order:

```text
Policy -> let -> DeduceList -> Pattern -> Annotation -> Initializer
```

Therefore its leading policy is alpha-normalized under the inherited
environment before the local DeduceList extends that environment. The local
holes are visible to the following Pattern, annotation, and initializer, but
not retroactively to the policy:

```text
NormalizePolicy(slot.policy, rho0)
rho1 = NormalizeDeduce(slot.deduce, rho0)
Normalize(slot.Pattern, slot.annotation, slot.initializer, rho1)
```

A callable head DeduceList scopes the entire callable:

```text
remaining Deduce annotations
capture slots and capture initializers
parameters and call policy
return slot and head clauses
callable body
```

The return clause remains a let-shaped result binding slot:

```lang
let f = <A>(self, x: A) -> r: A => {
    let y: A = x;
    y
};
```

Here `r` is the explicit binder bound to the returned object, while `A` is the
postfix annotation Pattern on that binder. If the callable returns an ordinary
value, `r` denotes that value; if it returns a type/Pattern object, `r` denotes
that type/Pattern object. The parser does not classify `r` itself as a type.

The anonymous constrained form is `-> _: A`. `-> A r` is an extraction
Pattern and must not be reinterpreted as the annotated-result shorthand
`-> r: A`.

Nested callables inherit the visible hole environment, allocate a new callable
semantic owner and a new callable-head `PatternRoot`, and may shadow inherited
hole spellings. A body-local let Pattern likewise creates an independent root.
Nested BindingSlots, Products, Sequences, annotations, and Pack operands inside
one extraction retain the containing root. Hole scope is separate from ordinary
value-binder scope; a value binder with the same spelling does not retarget a
Pattern-context hole occurrence.

Raw AST preserves lexical scope shape, spelling, and pre-semantic name role.
After structural normalization, an alpha-normalization pass allocates callable
owners, `PatternRoot` identities, and root-local binder ordinals, then rewrites
every scoped Pattern/policy occurrence to an exact `HoleBinderId`. Source spans
remain provenance only. Alpha-equivalent
sources such as `<A, B: A>` and `<X, Y: X>` therefore have the same binder/ref
graph structure regardless of spelling or byte offset. Frontend identity
carries `AlphaOwnerId × NormSemanticOwnerId × PatternRootLocalId ×
HoleLocalId`. Before multi-root build comparison, the Norm owner is mapped to
a persistent `SemanticOwnerId`, yielding `SemanticOwnerId ×
PatternRootLocalId × HoleLocalId`. `SourceUnitId × LocalHoleId` is not the
semantic identity; a source unit remains provenance or an owner-construction
input.

Compiler-generated receiver holes do not enter the source-name redeclaration
table. Before alpha conversion, a generated declaration and its Pattern/policy
references share a generated-syntax-local hygienic key; display spelling such
as `T` is diagnostic provenance only. Thus a generated `.name`, `..name`, or
prefix-negative helper inside a user `<T>` scope receives a fresh binder and
cannot capture or redeclare the user's `T`.

This Norm pass does not alpha-bind ordinary value-side `NormExpr::Name` or
ungrouped `NormNavComponent::Name`. Callable-wide scope means that the hole
environment reaches the whole callable, while exact binding is currently
performed only for Pattern/policy occurrences. Value-side names, including
the generated `T` component in `field::T`, remain unresolved input to the
future name-resolution pass.

The anonymous `_` placeholder has no named binder identity.

## 3. Dot names use ordinary ADL

```text
.name -> name::adl
E.name -> the same entry through the ordinary Product/call spine
```

The [operator/declaration owner](operator-patterns-and-generative-declarations.md)
defines requested-name extraction, ordinary forwarding construction and direct
result delivery. There is no canonical normalizer-generated forwarding body.
The existing DotClosureLowering/NormClosure remains a documented implementation
carrier pending alignment, not semantic authority.

Dot origin never changes pipe/Product association, suffix binding, first-product
continuation or legality repair. Substituting a legally bound ordinary dot result
retains the call-binding shape. The receiver remains an explicit object argument,
not implicit self. `..name` retains its distinct direct-call surface.

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
let f = (self, ...args) -> ...result => { ... };
```

These positions share syntax/normalization only. Their later semantic consumer
(ordinary binding, argument matching, or return-result matching) determines
what the remainder is relative to; no parameter-only pack object is created.

`...args` binds that remainder to the ordinary name `args`. It does not
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

It does not become `Pack(Sequence[x, b])`. The parser preserves any following
Pattern primary, including a parenthesized Product, but parser acceptance does
not prove that the operand survives P normalization. Canonical sequence Pack
nodes live in `NormPattern`; they are never hidden inside `NormSkeleton`.

### 4.2 Ordered and unordered levels

Order is determined at every layer by its direct entries: all named is unordered,
any bare entry orders the whole layer. A top name is unnecessary; nested layers
are independent. Intermediate extraction uses the Pattern owner's same R_Gamma:
one whole extraction at an unordered layer, multiple aligned extractions at an
ordered layer. This does not relax any Pack rule below.

At an order-insensitive named level, ordinary siblings match their distinct
top modes first and the Pack receives the unmatched siblings. Those remaining
siblings do not share a new common top mode. Consequently only a whole
remainder binder/discard, including a transparent let-shaped wrapper, is
admissible:

```text
Order(L) = Unordered
  => AdmissiblePackOperand(L, Q)
     iff Q is WholeRemainderBinder
```

At an order-sensitive level the Pack receives a contiguous interval. A
structured operand is meaningful only when its P-normal form retains a stable
top mode:

```text
Order(L) = Ordered
  => AdmissiblePackOperand(L, Q)
     iff Q is WholeRemainderBinder
      or StableTopMode(N_P(Q))
```

Thus a legal structured spelling supplies a non-flattened top mode:

```lang
...((a, b) pair)
```

Here `pair` is the stable top mode and `(a, b)` is its next-level internal
structure.

The bare spelling:

```lang
...(a, b)
```

has no stable top mode. The parser preserves its Raw shape, but P normalization
cannot let `Pack` reify the Product boundary that ordinary Product
normalization removes. The normalized Pattern-validation handoff therefore
rejects it as non-canonical. This is not a restricted-evaluator implementation
gap.

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

Every Pack contributes exactly one outward node at its containing structural
level:

```text
...rest -> one EP
..._    -> one DP
...Q    -> one outward Pack position
```

For legal `...((a, b) pair)`, evidence for `pair` and its internal `a`/`b`
structure belongs to the operand's preserved next structural level. It is not
flattened sideways into two EP nodes at the containing level. Captured width,
runtime remainder length, and internal node count never manufacture
same-level Pack specificity.

This tuple is only the Pattern-specificity preference dimension. It is not a
global score across stage, PolicyMode, result policy, or named strategies.
