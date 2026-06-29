# Return Value, Extraction View, and Implicit Decomposition

**Status: Future design boundary. Not current implementation behavior.**

An evaluation returns one result object. The result object's value normal form is
one of two shapes:

```text
e:
  ordinary non-product value point

P:
  product value / product normal form
```

The distinction is semantic, not an implementation-language distinction between
"single return" and "multiple return":

```text
return e
  -> value normal form is e

return P
  -> value normal form is P
```

`P` is not an outer call wrapper. If a function returns a product, the result is
product normal form.

`?` does not mean destructure. `?` means enter one exposed extraction view:

```text
leaf e:
  e? = e

non-leaf e:
  e? = peel_one_exposed_extraction_view(e)
  // result may be e' or P

product P:
  P? = P
```

Bare `?` peels one declared top extraction pattern layer. It is not recursive by
default. It does not keep peeling until a target pattern is found. It does not
perform Error propagation. It does not stand for arbitrary pattern matching.

A type/value may define its exposed `?` view:
- as one-layer peel;
- as a direct jump to a chosen exposed view;
- as an identity view, making the value opaque under bare `?`.

The language default only performs one declared top-view transition. Multi-layer
or recursive peeling must be explicitly defined by the type / extraction
interface.

For `P`, there is no top-level pattern shell to peel. `P` is already the product
value, so `?` is idempotent on product normal form.

Destructuring is a pattern-matcher operation, not value-level `?` semantics:

```text
bind ProductPattern against P
  -> pattern matcher consumes product elements of P
```

Equality never inserts `?`. Binding may request `?` only as pattern-directed
repair.

A leaf is any value whose current extraction interface does not permit further
decomposition by that same interface. `1uint8`, `uint8`, and `(int)Vec::std` are
all leaves in ordinary value context. Thus:

```text
((int)Vec::std)? == (int)Vec::std
```

This is not a claim that `Vec::std` has no argument. It says `(int)Vec::std`
carries no product extraction interface in ordinary value context. Extraction of
the type parameter requires an explicit rank-pattern context.

## 0. The Hourglass Model

Every constructed value can be understood as a waist point:

```text
        extraction pattern space
              ↑
              |
        construct / extract
              |
              ↓
          value point e
              |
              ?
              ↓
        exposed extraction view e' or P
```

Upward, the value participates in a constructor-specific isomorphism:

```text
construct_C : Pattern_C -> Value_C
extract_C   : Value_C -> Pattern_C

extract_C(construct_C(P)) = P
construct_C(extract_C(v)) = v
```

This `extract_C` is a named constructor/extractor pair, not bare `?`.

Downward, `?` enters the value's currently exposed extraction view:

```text
leaf e?       = e
product P?    = P
C(P)?         = declared ordinary extraction view
```

If that view contains product elements, each element may itself be a new waist
point, and `?` may be applied again. The result is not a one-shot AST expansion
but a chain of waist points connected by view transitions.

Examples:

- `()single_return` may evaluate to a leaf `e`; `e? = e`.
- `()two_return` may evaluate directly to product normal form `P`; `P? = P`.
- `val : t` is a non-leaf `e`; `val?` enters the named-field product view.
- `(int)Vec::std` is a leaf in ordinary value context; `?` is idempotent.

This is why `?` must not be understood as "inverse constructor." It goes
downward into the next exposed extraction view. The upward
constructor/extractor isomorphism is a separate named interface.

### `?` Peels One Top Pattern Layer

Bare `?` is a one-step extraction-view transition. It attempts to peel one
declared top pattern layer from the value.

It is not recursive by default, and it is not an error-propagation shorthand.

```text
leaf e:
  e? = e

product P:
  P? = P

non-leaf e:
  e? = the declared exposed view of e
```

The exposed view may be a product, a named product, a sum pattern, or another
value-shaped view. Pattern matching consumes that exposed view after `?`; `?`
itself does not recursively search for a matching pattern.

A type may define its exposed `?` view as an identity view, making the value
opaque under bare extraction:

```text
opaque_value? = opaque_value
```

A cryptographic key, capability token, opaque handle, or abstract module value
may intentionally define `?` as identity. Internal structure does not
automatically imply extractability through bare `?`.

Conversely, a wrapper may define `?` to expose a chosen inner branch pattern
directly, if that is the abstraction's intended interface:

```text
wrapped_bool? -> if | else
```

This is a custom exposed view, not recursive default peeling. The language
default peels only one declared top layer. If a user wants additional peeling,
they must either write `?` again or explicitly define a multi-step extraction
interface on the type.

### Minimal Sum-Pattern Example: `bool`

```lang
let bool: type = ((if | else) bool) |> struct;
```

The first `bool` is the symbol being bound. The second `bool` is the pattern /
construction name attached to the sum pattern `if + else`.

In cond-oriented contexts, logical operators naturally return the bare branch
pattern:

```text
bool -> if | else
(bool, bool) -> if | else
```

For example:

```text
not  : bool -> if | else
and  : (bool, bool) -> if | else
or   : (bool, bool) -> if | else
```

Because these operators already return `if | else`, the cond consumer does not
need to apply `?` — cond branch selection consumes the returned branch pattern
directly.

If a user wraps the bare `if | else` pattern into a custom `bool` value/type but
still wants `?` to expose `if | else` in one step, the wrapper's extraction
interface should define `?` to expose that branch pattern directly:

```text
wrapped_bool? -> if | else
```

This is a user-defined extraction interface, not recursive default peeling.

If the wrapper's default top pattern exposes only an intermediate layer:

```text
wrapped_bool? -> inner_bool_layer
```

The language default stops there. It does not continue with:

```text
inner_bool_layer? -> if | else
```

unless the user explicitly defines recursive / multi-step extraction policy or
writes `?` a second time.

The same mechanism generalizes beyond `bool`:

```text
Option-like value:
  opt? -> some | none

Result-like value:
  res? -> ok | err

AST node:
  node? -> literal | call | block | name

User-defined wrapper:
  wrapper? -> chosen exposed view
```

These are design examples. They are not implemented as current runtime types,
pattern matchers, or constructor-specific extractors.

## 1. Single-Return Non-Product Value

For:

```lang
let e = () |> single_return;
```

if `single_return` returns a leaf non-product value, then:

```text
e? == e
let a, b = e?       // error: e? is still leaf
(a, b) == e         // false: e is not product normal form
```

A corresponding implicit decomposition form also fails:

```lang
let a, b = e;       // error; pattern-directed repair cannot expose a product
```

because the exposed view is still a single value, not a two-element product.

## 2. Product-Return Normal Form

For:

```lang
let P = () |> two_return;
```

if `two_return` returns product normal form `(a, b)`, then:

```text
P == (a, b)
P? == P
let a, b = P        // direct product binding
(a, b) == P         // true
```

There is no value-level call wrapper around the product. If an implementation
needs call-site provenance, invocation records, or debug origin, that is metadata
or origin material, not a value-level wrapper.

## 3. Binding-Context Implicit Decomposition

The binding rule is:

```text
let Pattern = Expr
```

first tries to bind `Pattern` against the direct value normal form of `Expr`.

If that fails, and `Pattern` is an extraction-demanding pattern, the checker may
try one view transition:

```text
let Pattern = Expr?
```

This implicit bridge is allowed only in binding / extraction contexts.

It is not allowed in ordinary value expressions:

```text
(a, b) == e      // no implicit `?`
f(e)             // no implicit `?` unless f's parameter pattern demands it
```

Therefore:

```text
binding context may request `?`
value equality never inserts `?`
ordinary expression evaluation never inserts `?`
```

## 4. Equality Examples

Equality never inserts `?`, but product normal form participates directly in
product equality:

```text
(a, b) == P
  -> true, when P is exactly product normal form (a, b)

(a, b) == e
  -> false, when e is a non-product value point

(a, b) == e?
  -> true, if e? exposes exactly product normal form (a, b)
```

The correct contrast is:

```text
P:
  let a, b = P
  (a, b) == P
  P? == P

e:
  let a, b = e?      // if e? exposes compatible P
  (a, b) != e
  (a, b) == e?       // if e? exposes compatible P
```

For a non-leaf construction value `e` with an exposed product view:

```text
e? == P
P == e?             // true
P == e              // false
P A== e             // true under named constructor / pattern A, if A reconstructs e
```

`A==` is provisional notation for constructor/pattern mediated equality. It is
not ordinary value equality and does not imply that equality inserts `?`.

## 5. Named Extraction Is Not Bare Product Extraction

For a constructor-shaped value, `?` uses the value's declared exposed extraction
interface. A struct value does not expose a bare product unless its extraction
interface declares a bare product.

Given:

```lang
let t = (uint8 a, uint8 b)struct;

let val = () |> (t uninit);
val ref. a = 1uint8;
val ref. b = 1uint8;
let val = val as t;
```

`val` is a non-product value point `e` of constructed type `t`.

Its exposed extraction view is the field-labeled product:

```text
P_field = (a a::t, b b::t)
val? == P_field
P_field == val?       // true
P_field == val        // false
P_field t == val      // true, constructor-mediated reconstruction
```

Field labels are part of the extraction shape.

Therefore:

```lang
let a, b = val copy?;
```

is an error if `val copy?` exposes the named field product rather than a bare
two-element product.

The correct binding form is:

```lang
let a a, b b = val copy;
```

This is valid by binding-context implicit decomposition, and is equivalent to:

```lang
let a a, b b = val copy?;
```

Here the first `a` and `b` are field-pattern names, and the second `a` and `b`
are local binders.

## 6. Summary Rule

```text
1. Evaluation result normalizes to e or P.

2. `?` peels one declared top extraction pattern layer. It does not mean
   destructure. It is not recursive by default. It does not perform Error
   propagation.

3. On leaf values, `?` is idempotent:
   e? = e

4. On product normal form, `?` is idempotent:
   P? = P

5. Named construction values expose their declared named extraction shape via
   `?`, not necessarily a bare product.

6. Equality never inserts `?`.

7. Binding may request `?` as pattern-directed decomposition repair.

8. Product matching consumes P in the pattern matcher; value-level `?` does not
   produce a separate split result.
```

For product return:

```text
let P = () |> two_return
P == (a, b)
P? == P
let a, b = P
(a, b) == P
```

For structs:

```text
val? == field-labeled product
field-labeled product != val
field-labeled product |> constructor == val
```

## Implementation Substrate Note

The build implementation records this model as a static shape substrate:

```text
EvalResultNormalForm = ValuePoint(e) | Product(P)
```

`question_view` is the pure shape-level view transition:

```text
Product(P) -> Product(P)
leaf e     -> e
non-leaf e -> exposed view
```

Binding and pattern matching consume product normal form directly:

```text
ProductPattern + P -> Direct
ProductPattern + non-leaf e exposing P -> AfterExtraction
ProductPattern + leaf e -> Mismatch
```

"Split" is a pattern-matcher consumption operation over product normal form. It
is not the value-level result of `?`. Equality does not call `question_view` and
never requests extraction repair.

## 7. Relationship to Control-Flow-Local Meta Evaluation

The one-layer extraction view is complemented by a branch-local evaluation
substrate (see `static-pattern-spaces-and-extraction-chains.md`§17). That
substrate uses sum-pattern spaces (e.g. `if | else`) as branch-selection material.
It enforces that only the selected branch may perform lookup, policy check, meta
invocation, or local symbol construction. Unselected branches have no lookup,
policy, invocation, or `NamespaceDelta` obligation.
