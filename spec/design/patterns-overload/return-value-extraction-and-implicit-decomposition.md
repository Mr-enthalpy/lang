# Return Value, Extraction View, and Implicit Decomposition

**Status: Future design boundary. Not current implementation behavior.**

A function call always returns one value.

That value may itself be a product value, but the call result is still one value:

```text
e      -> e
(e, e) -> P
```

Therefore a call result has two distinguishable forms:

```text
t      = the returned value
t?     = the extraction-facing view of the returned value
```

`?` does not mean destructure. `?` means enter the next extraction view.
Destructuring happens only if that view is product-shaped.

1. `?` is a value-to-extraction-view operator.

2. On leaf values, `?` is idempotent:
   ```text
   e? -> e
   ```

3. On product-facing values, `?` exposes and splits the product view:
   ```text
   P? -> split(P)
   ```
   `split(P)` denotes the product-shaped extraction view made available to
   binding/pattern contexts. It is not an implicit coercion inserted into
   ordinary value expressions.

4. On named construction values, `?` exposes the declared named extraction
   shape, not necessarily a bare product.

5. Equality never inserts `?`.

6. Binding may insert `?` only as pattern-directed decomposition repair.

7. Meta invocation must be specified at the symbolic/reduction level because
   a type constructor call returns a construction value first; type
   computation is only one projection of that value.

A leaf is any value whose current extraction interface does not permit further
decomposition by that same interface. `1uint8`, `uint8`, and `(int)Vec::std`
are all leaves in ordinary value context. Thus:

```text
(int Vec::std)? == int Vec::std
```

This is not a claim that `Vec::std` has no argument — it is the statement that
`(int)Vec::std` carries no product extraction interface in ordinary value
context. Extraction of the type parameter requires an explicit rank-pattern
context.

## 0. The hourglass model

Every constructed value can be understood as a waist point:

```text
        extraction pattern space
              ↑
              |
        construct / extract
              |
              ↓
          value point
              |
              ?
              ↓
        next extraction view
```

Upward, the value participates in a constructor-specific isomorphism:

```text
construct_C : Pattern_C -> Value_C
extract_C   : Value_C -> Pattern_C

extract_C(construct_C(P)) = P
construct_C(extract_C(v)) = v
```

This `extract_C` is a named constructor/extractor pair — not bare `?`.

Downward, `?` enters the value's currently exposed extraction view:

```text
leaf?     -> leaf       (idempotent)
product?  -> split/product-view
C(P)?     -> declared ordinary extraction view
```

If that view contains product elements, each element may itself be a new
waist point, and `?` may be applied again. The result is not a one-shot AST
expansion but a chain of waist points connected by `?`.

Examples:

- `()single_return` is a waist point; `?` enters a leaf view → `?` is
  idempotent.
- `()two_return` is a waist point; `?` enters a product view → split.
- `val : t` is a waist point; `?` enters a named-field product → split;
  the constructor `t` restores the original value upward.
- `(int)Vec::std` is a waist point; `?` enters a leaf view in ordinary
  value context → idempotent. Extraction of `int` requires the
  constructor-specific `extract_Vec` interface, not bare `?`.

This is why `?` must not be understood as "inverse constructor." It goes
downward into the next extraction view. The upward constructor/extractor
isomorphism is a separate named interface.

In summary: a constructed value is not the endpoint of type computation —
it is a symbolic construction node. Upward, it belongs to a
constructor/extractor isomorphism. Downward, `?` enters the value's
currently exposed extraction view, and that view may itself contain new
waist points.

## 1. Single-return value

For:

```lang
let t = () |> single_return;
```

the call value is:

```text
t == ()single_return
```

If `single_return` returns a single expression value, then the trivial extraction view is the value itself:

```text
t? == t
```

This idempotence also holds for constructor-shaped values that do not expose
a product extraction interface:

```text
(int Vec::std)? == int Vec::std
```

`(int)Vec::std` is a single return value — not a product — so `?` acts as
the identity on its leaf extraction view.

Therefore:

```lang
let a, b = t?;
```

is an error because `t?` reduces to the leaf value `t`, which cannot match
a two-element product pattern. `?` is still valid on `t` — it just returns
`t` itself (idempotent on leaf), so `let a, b = t?` fails at pattern
matching, not at extraction.

So:

```text
let t = () |> single_return;
t == ()single_return;      // true
let a, b = t?;             // ERROR
t? == t;                   // true
```

A corresponding implicit decomposition form also fails:

```lang
let a, b = t;              // ERROR, equivalent attempt would be `let a, b = t?`
```

because the extraction view is still a single value, not a two-element product.

## 2. Product-return value

For:

```lang
let t = () |> two_return;
```

the call value is still the call construction value:

```text
t == ()two_return
```

But if `two_return` returns a product, the extraction-facing view is the returned product:

```text
t? == (a, b)
```

Therefore:

```lang
let a, b = t?;
```

is valid.

In binding context, when the left side is a product/extraction pattern and the right side is a value whose direct shape does not match, the checker may insert one implicit extraction bridge:

```lang
let a, b = t;
```

is interpreted as:

```lang
let a, b = t?;
```

provided that `t?` exposes a compatible product pattern.

Thus:

```text
let t = () |> two_return;
t == ()two_return;         // true

let a, b = t?;             // valid
let a, b = t;              // valid by implicit decomposition, equivalent to `let a, b = t?`

t? != t;                   // true
(a, b) == t?;              // true
(a, b) == t;               // false
```

The last line is essential. Equality expression does not insert `?`. Implicit decomposition is a binding-context operation, not a value-level equality rule.

## 3. Binding-context implicit decomposition

The binding rule is:

```text
let Pattern = Expr
```

first tries to bind `Pattern` against the direct value shape of `Expr`.

If that fails, and `Pattern` is an extraction-demanding pattern, the checker may try:

```text
let Pattern = Expr?
```

This implicit bridge is allowed only in binding / extraction contexts.

It is not allowed in ordinary value expressions:

```text
(a, b) == t      // no implicit `?`
f(t)             // no implicit `?` unless f's parameter pattern demands it
```

Therefore:

```text
binding context may insert `?`
value equality never inserts `?`
ordinary expression evaluation never inserts `?`
```

## 4. Named extraction is not bare product extraction

For a constructor-shaped value, `?` uses the constructor's declared extraction
interface. A struct value does not extract to a bare product unless its
extraction interface declares a bare product.

Given:

```lang
let t = (uint8 a, uint8 b)struct;

let val = () |> (t uninit);
val ref. a = 1uint8;
val ref. b = 1uint8;
let val = val as t;
```

`val` is a value of constructed type `t`.

Its extraction view is not the bare product:

```text
(value_of_a, value_of_b)
```

but a field-labeled product shape:

```text
(a value_of_a::t, b value_of_b::t)
```

Field labels are part of the extraction shape.

Therefore:

```lang
let a, b = val copy?;
```

is an error.

The left side:

```lang
a, b
```

is a bare product pattern with two binders. It does not mention the field-pattern names `a` and `b`. But `val copy?` exposes a named field extraction shape, not a bare two-element product.

The correct binding form is:

```lang
let a a, b b = val copy;
```

This is valid by binding-context implicit decomposition, and is equivalent to:

```lang
let a a, b b = val copy?;
```

Here the first `a` and `b` are field-pattern names, and the second `a` and `b` are local binders.

## 5. Struct extraction and reconstruction

For a value `val : t`, its extraction view is the field-labeled product:

```text
val? == (a a::t, b b::t)
```

But the field-labeled product is not itself equal to the original constructed value:

```text
(a a::t, b b::t) == val      // false
```

The original value is recovered by applying the constructor `t` to the field-labeled product:

```text
(a a::t, b b::t) t == val    // true
```

Thus:

```text
(a a::t, b b::t) == val?     // true
(a a::t, b b::t) == val      // false
(a a::t, b b::t) t == val    // true
```

This is the construction / extraction isomorphism:

```text
extract_t(val) = P
construct_t(P) = val
```

but not:

```text
P = val
```

The product view and the constructed value are distinct values at the expression-equality level.

## 6. Summary rule

```text
1. `?` enters the next extraction view. It does not mean destructure.

2. On leaf values, `?` is idempotent:
   e? -> e

3. On product-facing values, `?` splits the product:
   P? -> split(P)

4. Named construction values expose their declared named extraction shape
   via `?`, not necessarily a bare product.

5. Equality never inserts `?`.

6. Binding may insert `?` as pattern-directed decomposition repair.

7. Meta call returns a symbolic construction value first; type computation
   is a projection of that value, not its definition.
```

Therefore:

```text
t == ()two_return        // call value equality
t? == (a, b)             // extraction view
let a, b = t             // binding-context implicit decomposition
(a, b) == t              // false, no implicit decomposition in equality
```

and for structs:

```text
val? == field-labeled product
field-labeled product != val
field-labeled product |> constructor == val
```
