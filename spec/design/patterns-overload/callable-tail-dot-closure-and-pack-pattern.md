# Callable Tail, Dot Closure, and Pack Pattern

**Status: canonical semantic design with typed parser/normalizer substrate.**
Named-strategy execution and compiler-provided default-body generation remain
strategy-specific later work; their syntax and semantic boundaries are fixed
here.

This note connects three thin surface forms to the existing function-object,
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

### 1.1 No rollback over the return extraction pattern

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

### 1.2 Implementation form and strategy are orthogonal

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

## 2. `.name` is a first-class field-function closure

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

`T` is inferred from the first formal argument when the generated function
object is invoked. It is not captured from a syntactic expression to the left
of the dot. Consequently `.name` can be stored, passed, or composed like any
other function object.

The compact suffix is defined through that same atom:

```text
E.name
  == E |> .name
```

It is not a second field-access semantic node. Raw AST may retain
`MemberSugar(E, name)` for source fidelity, but normalization must use the same
`DotClosure(name)` core.

Explicit incoming-pipe continuation supplies member-style remainder arguments:

```text
E |> .name
E |> .name (P2)
E |> .name P2_item
```

These normalize as one call whose source product starts with `E` and continues
with the right-side items:

```text
items |> .push value
  == (items, value) |> .push
```

Compact `E.name` closes before later space-bound material. Therefore:

```text
E.name P
  == (E |> .name) P
  != E |> .name P
```

The first form calls/applies `P` to the result of `E |> .name`; it does not
silently reinterpret `P` as another argument of `name::T`. This follows the
ordinary space-binding/call chain and is why compact dot syntax is not a
general member-call notation.

The general call rule remains `P1 |> Callable P2`; `.name` merely supplies the
callable expression.

`..name(product)` remains a distinct direct member-call sugar. It models a
receiver-position call directly and need not first expose a transportable
`.name` function value. Neither form removes the other:

```text
.name    first-class field-function closure
..name   direct member-call sugar
```

## 3. `...` is a Pattern remainder matcher

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

### 3.1 Ordered and unordered levels

At a name-directed, order-insensitive level, explicit named siblings match
first and the pack absorbs all unmatched siblings. At an order-sensitive
sequence level, prefix and suffix fixed patterns match normally and the pack
absorbs the remaining contiguous middle sequence.

### 3.2 One pack per normalized level

For every normalized structural level `L`:

```text
count(child in L where child is Pack) <= 1
```

The check occurs after Pattern normalization, so grouping or nested syntax
cannot conceal two packs at one level. Nested levels are independent:

```lang
(a, (b, ...inner), ...outer)  // valid
(a, ...x, ...y)               // invalid
```

### 3.3 No unpack operator

There is no corresponding right-value spread syntax. The remainder bound to
`args` is already ordinary normalized Pattern/value product material. Existing
product normalization composes it in:

```lang
(val, args) |> name::T
```

Introducing `*args`, `unpack(args)`, or an RHS meaning for `...args` would add
a redundant second algebra and is outside this design.

## 4. Pack specificity evidence

A pack contributes one outer Pattern node regardless of whether it absorbs
zero, two, or two hundred elements. Matching records four node classes:

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
additional pack specificity. This tuple is only the Pattern-specificity
preference dimension. It is not a global score across stage, mutability,
result policy, or named strategies.

## 5. Current implementation boundary

Implemented substrate:

- `Ellipsis`, `DotClosure`, `BindingPatternAst::Pack`, and the callable-tail
  Raw AST variants;
- normalization to `NormPattern::Pack`, generated dot closures, named/default/
  delete implementation variants, compact `E.name`, and explicit incoming
  `E |> .name P` product continuation;
- pack preservation in every parser binding-slot context (`let`, parameter,
  return, and nested product extraction), not a parameter-only grammar;
- post-normalization one-pack-per-level validation;
- restricted variadic applicability, remainder binding, and pack node-class
  specificity evidence;
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
