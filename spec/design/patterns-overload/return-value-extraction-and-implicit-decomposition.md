# Return Value, Extraction View, and Pattern-Directed Decomposition

**Status: Future design boundary. Not current implementation behavior.**

The relational meaning of Pattern observation/extraction and the fact that
constructor/extractor inverses are family-specific theorems are canonical in
`pattern-values-relational-semantics-and-extraction.md`. This document applies
that authority to result delivery and one-layer view construction; it does not
define a second base Pattern calculus.

Policy staging of this extraction flow, including runtime Pattern retention and
automatic require, is canonical in
`../symbol-world/symbol-policy-and-compile-flow-projection.md`.

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

`?` does not mean destructure or enable extraction. Its current default meaning
is one top-Pattern peel:

```text
leaf? = leaf

P? = P

TopPattern(P)? = P

x?? = (x?)?
```

`TopPattern(P)? = P` is only display shorthand for the resident body. The
exposed extraction view must retain a name-absent Pattern-layer boundary:

```text
OptionalPeel(
  PatternLayer(name = c, body = B, order = O)
) =
  PatternLayer(name = absent, body = B, order = O)
```

If `TopPattern_c` had a fully named, order-insensitive body before the peel, the
exposed view is therefore
`PatternLayer(absent, Product(a, b), Unordered)`, not a naked Product. `absent`
is a semantic name-absence marker; it is not source wildcard `_`, binder
absence syntax, or an artificial child. The peel erases the top Pattern name
while preserving its layer boundary and ordering.
This does not make a naked Product unordered: `(a, b) != (b, a)`, and the
fixed point `(a, b)? = (a, b)` gains no matching authority. A positional top
Pattern body also remains positional after peeling.

If no top Pattern is peelable:

```text
OptionalPeel(x) = x
```

This is an ordinary fixed point, not matching failure and not a `none` result.
The retained name-absent layer must also make peeling commute with
normalization:

```text
PeelView(Norm(x)) = Norm(PeelView(x))
```

These are future semantic requirements and are not claimed as current
executable behavior.

Bare `?` peels at most one layer. It does not keep peeling until a target pattern
is found, perform Error propagation, search for an extractor, or stand for
arbitrary pattern matching. Leaves and Pattern normal forms are fixed points.

Destructuring is a pattern-matcher operation, not value-level `?` semantics:

```text
bind ProductPattern against P
  -> pattern matcher consumes product elements of P
```

Equality never inserts `?`. Pattern matching and binding may directly read the
symbol's Pattern layer; they do not need to insert `?` as an enabling bridge.

A leaf is any value whose current extraction interface does not permit further
decomposition by that same interface. `1uint8`, `uint8`, and `(int Vec::std)` are
all leaves in ordinary value context. Thus:

```text
(int Vec::std)? == (int Vec::std)
```

This is not a claim that `Vec::std` has no argument. It says the current object
is already a fixed-point leaf for the default one-layer view. Extraction of the
type parameter requires an explicit rank-pattern context.

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
        one-layer top Pattern view
```

Upward, a Pattern family that declares these interfaces may prove a
constructor-specific isomorphism:

```text
construct_C : Pattern_C -> Value_C
extract_C   : Value_C -> Pattern_C

extract_C(construct_C(P)) = P
construct_C(extract_C(v)) = v
```

This `extract_C` is a named constructor/extractor pair, not bare `?`.

Downward, `?` peels one top Pattern layer:

```text
leaf e?       = e
product P?    = P
TopPattern(P)? = P
```

For extraction, the last result is a name-absent
`PatternLayer(absent, P, O)` whose
ordering `O` is inherited from the peeled top Pattern. This is a retained
structural boundary, not merely metadata attached to a naked Product and not a
change to Product equality.

If that view contains product elements, each element may itself be a new waist
point, and `?` may be applied again. The result is not a one-shot AST expansion
but a chain of waist points connected by view transitions.

Examples:

- `()single_return` may evaluate to a leaf `e`; `e? = e`.
- `()two_return` may evaluate directly to product normal form `P`; `P? = P`.
- `val : t` may expose its one top named-field Pattern view through `val?`.
- `(int Vec::std)` is a leaf in ordinary value context; `?` is idempotent.

This is why `?` must not be understood as "inverse constructor." It peels one
top layer. The upward
constructor/extractor isomorphism is a separate named interface.

### `?` Peels One Top Pattern Layer

Bare `?` attempts to peel one top Pattern layer.

It is not recursive by default, and it is not an error-propagation shorthand.

```text
leaf e:
  e? = e

product P:
  P? = P

TopPattern(P):
  TopPattern(P)? = PatternLayer(name = absent, body = P, order = TopPatternOrder)
```

Pattern matching may consume a symbol's Pattern layer directly without `?`;
`?` itself does not recursively search for a matching pattern.

A future extension may allow a type to declare one custom exposed view, but the
extension is not frozen here. It must remain bounded and must not imply
arbitrary multi-layer skipping, recursive search, AST rewriting, or macro
capability. Repeated explicit `?` remains ordinary composition:

```text
x?? = (x?)?
```

### Minimal Sum-Pattern Example: `bool`

```lang
let bool: type = ((if | else) bool) |> struct;
```

The first `bool` is the symbol being bound. The second `bool` is the pattern /
construction name attached to the sum Pattern `if | else`.

Logical operators return ordinary bool symbols:

```text
not  : bool -> bool
and  : (bool, bool) -> bool
or   : (bool, bool) -> bool
```

The bool symbol's Pattern layer carries exactly one alternative space:

```text
if | else
```

`true === if::bool` and `false === else::bool` are aliases of those Pattern
symbols. They do not create a second `true | false` alternative space.

Pattern matching reads that layer directly; conditional control flow does not
require `?`. Explicit `bool_value?` only asks for the one-layer top Pattern view.

The same mechanism generalizes beyond `bool`:

```text
Option-like value:
  opt? -> some | none

Result-like value:
  res? -> ok | err

AST node:
  node? -> literal | call | block | name

User-defined wrapper:
  wrapper? -> one declared exposed top view (future extension)
```

These are design examples. They are not permission for default `?` to skip
multiple layers or search recursively, and they are not implemented as current
runtime types or custom view declarations.

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

A direct binding form also fails:

```lang
let a, b = e;       // error; e has no matching product Pattern
```

because neither the value normal form nor its Pattern layer supplies a
two-element product match.

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

## 3. Binding Reads Pattern Directly

The binding rule is:

```text
let Pattern = Expr
```

The checker resolves/evaluates `Expr`, then matches against its value normal
form and Pattern layer. It does not retry a failed match by inserting `?`.
Postfix `?` is applied only when source writes it explicitly:

```text
let Pattern = Expr?
```

The same rule applies in binding, parameter, and other extraction contexts:

```text
(a, b) == e      // no implicit `?`
f(e)             // parameter matching reads e's Pattern directly
```

Therefore:

```text
binding/pattern matching reads Pattern without `?`
only explicit source `?` requests the one-layer view
value equality never inserts `?`
ordinary expression evaluation never inserts `?`
```

### 3.1 Callable result extraction uses the same binding judgment

A callable return slot is a binding Pattern, including when it is a product
extraction:

```lang
-> (r first, d second)
```

Explicit writes in the body address `r` and `d` separately. A bare terminal
expression instead supplies one result object under the expectation:

```text
let (r first, d second) = expr
```

`expr return` and `expr (Self return)` use that same whole-result Pattern after
the active return frame has been selected. The explicit `Self` spelling changes
only which output frame receives the value, not how that frame decomposes it.
No special multi-return container, implicit `?`, or parallel assignment rule is
introduced.

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

## 5. Named Pattern View Is Not Bare Product Extraction

For a constructor-shaped value, the symbol's Pattern layer carries its named
field view. A struct value does not expose a bare product unless that Pattern
layer declares a bare product. `?` may peel one top named layer, but extraction
can read the Pattern directly.

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

This is valid because binding-pattern matching reads the Pattern layer directly.
Writing an explicit one-layer view may reach the same top Pattern:

```lang
let a a, b b = val copy?;
```

Here the first `a` and `b` are field-pattern names, and the second `a` and `b`
are local binders.

## 6. Summary Rule

```text
1. Evaluation result normalizes to e or P.

2. `?` attempts to peel one top Pattern layer. It does not mean destructure,
   search, recursive decomposition, cross-rank conversion, or Error
   propagation.

3. On leaf values, `?` is idempotent:
   e? = e

4. On product normal form, `?` is idempotent:
   P? = P

5. Named construction symbols carry their named extraction shape in Pattern;
   `?` may expose one top layer but is not required for matching.

6. Equality never inserts `?`.

7. Binding/pattern matching may read the Pattern layer directly; `?` is not an
   enabling repair step.

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

The current `question_view` helper is transitional shape substrate for a pure
one-step view transition:

```text
Product(P) -> Product(P)
leaf e     -> e
non-leaf e -> exposed view
```

Binding and pattern matching consume product normal form directly:

```text
ProductPattern + P -> Direct
ProductPattern + non-leaf e exposing P -> AfterExtraction (current substrate name)
ProductPattern + leaf e -> Mismatch
```

`AfterExtraction` is a transitional implementation category. It must not be
read as a final rule that inserts postfix `?` after a failed binding. The final
matcher reads the symbol's Pattern layer directly; an explicit `?` remains a
separate one-layer view operation.

"Split" is a pattern-matcher consumption operation over product normal form. It
is not the value-level result of `?`. Equality does not call `question_view`.
Final pattern matching does not require this helper as an enabling bridge.

## 7. Relationship to Control-Flow-Local Meta Evaluation

The one-layer extraction view is complemented by a branch-local evaluation
substrate (see `static-pattern-spaces-and-extraction-chains.md`§17). That
substrate uses sum-pattern spaces (e.g. `if | else`) as branch-selection material.
It enforces that only the selected branch may perform lookup, policy check, meta
invocation, or local symbol construction. Unselected branches have no lookup,
policy, invocation, or `NamespaceDelta` obligation.

That invariant applies when a compile-policy scrutinee has been evaluated and a
single branch is selected. For a runtime-policy scrutinee, compile-flow
projection retains the Pattern and inferred require retains all
runtime-reachable alternatives as pattern-guarded contracts.
