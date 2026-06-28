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

`?` does not inspect call history. It exposes the returned value's declared extraction interface.

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

Therefore:

```lang
let a, b = t?;
```

is an error unless `t` exposes a product extraction interface. In the ordinary single-return case, it does not.

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

A struct value does not extract to a bare product unless its extraction interface declares a bare product.

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

There are three different relations:

```text
value equality:
  compares values directly
  never inserts `?`
  never calls constructors

extraction view:
  v? exposes v's extraction-facing pattern space
  may preserve labels / field names / constructor-specific pattern structure

binding decomposition:
  `let Pattern = v` may insert one implicit `?`
  only when Pattern demands extraction and v's direct value shape does not match
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
