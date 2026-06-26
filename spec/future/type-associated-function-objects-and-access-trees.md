# Type-Associated Function Objects and Access Trees

**Status: Future design note. No access-tree construction, field access
evaluation, borrow checking, lifetime checking, full meta execution, or
canonical type-value equality is implemented.**

This note records the v0.6 namespace-graph implications that future
field-access, access-tree, alias-forwarding, and writable-place checks must
preserve.

## Field Functions and Projection Spaces

Fields are unary function objects installed in a type-associated companion
space:

```text
field::T        : T       -> field
field::ref::T   : T ref   -> field ref
field::share::T : T share -> field share
```

`field::T` is value semantics (`T == T move`). Borrowed field access must begin
from an explicit borrow form, for example:

```text
val ref.field1.field2
val share.field1.field2
```

This document does not specify evaluation or lowering for those forms.

## Role-Aware Namespace Lookup

`ref` and `share` are namespace subspaces. Field functions are object-role
symbols. Therefore a field function and a namespace subspace may have the same
textual name under the same parent:

```text
ref::T          // may refer to a field function or projection namespace
ref::ref::T     // field named ref under the ref projection namespace
ref::share::T   // field named ref under the share projection namespace
```

Terminal lookup of `ref::T` or `share::T` requires a resolver expectation when
both roles exist. `AnyUnique` lookup must report ambiguity. Intermediate path
components resolve as namespace-capable parents.

## Type Value, Symbol Place, and Injection Target

The language must distinguish three uses that look similar in source text:

```text
type value usage
symbol/place usage
namespace injection target
```

When a type is used as a rank / classifier value, what matters is its value, not
its symbol name:

```text
let t: type = uint8
```

In a type/rank expression, `t` evaluates to the same type value as `uint8`.
This is analogous to ordinary value use:

```text
let a = 1
let b = 1

a + b
```

The value of `a + b` depends on the values of `a` and `b`, not on their symbol
names.

Namespace injection is not ordinary type-value evaluation. Injection targets a
writable symbol/place:

```text
let t: type = uint8
let f::t = ...
```

The injection target is `t`, not `uint8`.

The reason mirrors assignment:

```text
let a = 1
a = a + 1
```

This means `a == 2`, not `1 == 2`. The right-hand `a` is evaluated as a value;
the left-hand `a` is a place being updated. Assignment updates the place, not
the value `1`.

Likewise:

```text
let t: type = uint8
let f::t = ...
```

means:

```text
value(t) == value(uint8)
place(t) != place(uint8)
f is injected into place(t)
```

It must not be rewritten as:

```text
f is injected into place(uint8)
```

Type-value equality must not collapse symbol-place identity.

## Formal Judgment Distinction

Use distinct judgments for value evaluation and writable-place resolution:

```text
Γ ⊢ x ⇓ v
```

means expression / rank evaluation: `x` evaluates to value `v`.

```text
Γ ⊢ x ⇐ p
```

means injection-place resolution: `x` resolves to writable place `p`.

Then:

```text
let t: type = uint8
```

establishes:

```text
symbol(t)
value(t) = value(uint8)
place(t) = fresh place at current lexical level
```

while:

```text
let f::t = ...
```

executes a namespace delta operation:

```text
place(t) += { f ↦ ... }
```

not:

```text
place(uint8) += { f ↦ ... }
```

Injection is therefore closer to `+=` than to pure expression evaluation.

## Ordinary Type Binding Versus Alias Forwarding

`let t: type = uint8` and `let t === uint8` are not equivalent.

### Ordinary Type-Value Binding

```text
let t: type = uint8
```

means:

```text
t is a new symbol.
t has its own symbol/place identity.
value(t) == value(uint8).
place(t) is a fresh current-level place.
```

Therefore:

```text
let f::t = ...
```

may inject into `t`, assuming `t` is still open for namespace delta
installation. This does not mutate `uint8`.

### Symbol Alias / Forwarding

```text
let t === uint8
```

means:

```text
t forwards to symbol uint8.
t does not create a fresh writable place.
value(t) == value(uint8).
place(t) forwards to place(uint8).
```

Therefore:

```text
let t === uint8
let f::t = ...
```

attempts to inject into `place(uint8)`, not into a fresh `place(t)`. Since
`uint8` is an external stable object / built-in type, it is not writable by the
current lexical level. This injection must be rejected.

## Injection Through Alias

Injection through an alias / forwarding symbol is allowed only if the final
forwarded target resolves to a writable object place defined at the current
lexical level:

```text
InjectionAllowed(current_level, target_place) iff

1. target_place resolves to a symbol/object place, not merely a value;
2. target_place is defined at the current lexical level;
3. target_place is still open for namespace delta installation;
4. target_place is not an inner lexical object escaping outward;
5. target_place is not an external stable object.
```

This shape may be allowed in the future:

```text
let Local: type = (uint8 a) |> struct
let t === Local
let f::t = ...
```

Here `t` forwards to `Local`, and `Local` is a current-level object place that
may still be open for injection.

These shapes must be rejected:

```text
let t === uint8
let f::t = ...
```

because `uint8` is a built-in / external stable object, and:

```text
let t === (int)Vec::std
let f::t = ...
```

because `(int)Vec::std` denotes an external stable generated type value / type
object. It may be used as a type value, but it is not a current-level writable
place.

## Inner Lexical Places Cannot Escape

Injection targets must not be inner lexical objects that escape to an outer
level:

```text
{
  let Inner: type = (uint8 a) |> struct
  let t === Inner
}

let f::t = ... // invalid / impossible
```

The reason is place lifetime, not type-value equality. The inner symbol `Inner`
is destroyed or becomes unreachable when leaving its lexical level. Letting an
outer declaration inject into it is equivalent to a long-lived borrow pointing
to a short-lived name.

The same problem appears at meta-function return boundaries: a returned type or
returned alias may need to live at a global or externally visible level. It must
not expose a writable injection place that belongs to an inner temporary symbol.

## External Stable Objects

External stable objects are readable and may be alias targets, but they are not
writable injection targets. External stable objects include at least:

```text
built-in types
external visible first-order types
external meta-function generated type values / type objects
locked / frozen graph objects
```

These objects have sufficient lifetime, but they are not modifiable by the
current lexical level.

Therefore:

```text
let t: type = uint8
```

creates a new current-level place whose value equals `uint8`, while:

```text
let t === uint8
```

does not create a new place; it forwards to the external stable place of
`uint8`.

## Corrected Form Table

| Form | Symbol effect | Value effect | Injection-place effect |
| --- | --- | --- | --- |
| `let t: type = uint8` | Creates new symbol/place `t` | `value(t) == value(uint8)` | `f::t` injects into `place(t)` |
| `let t === uint8` | `t` forwards to symbol `uint8` | `value(t) == value(uint8)` | `f::t` attempts `place(uint8)`; rejected because external stable |
| `let t === Local` | `t` forwards to symbol `Local` | `value(t) == value(Local)` | `f::t` may inject into `place(Local)` if `Local` is current-level and open |
| `let T: type = ... |> struct` | Creates new symbol/place `T` | `value(T)` is a fresh generated type | `f::T` injects into `place(T)` |

## Relation to Type-Associated Namespace

A symbol binding of kind `type` may have a companion / injection place distinct
from the canonical type value it stores.

For:

```text
let t: type = uint8
```

`t` does not create a fresh type value, but it may own a fresh current-level
companion namespace place.

For type/rank use:

```text
t
```

`t` evaluates to the type value `uint8`.

For injection:

```text
let f::t = ...
```

the target is `place(t)`.

Thus:

```text
value(t) == value(uint8)
```

does not imply:

```text
companion_place(t) == companion_place(uint8)
```

Do not canonicalize injection targets through type-value equality.

## Relation to `===`

`===` is symbol forwarding, not value equality and not ordinary binding. It does
not create a fresh companion namespace place. Injection through `===` follows
the forwarded target place and then checks whether that final target is writable
from the current lexical level.

If the forwarded target is a current-level open object, injection may be allowed.
If the forwarded target is an inner lexical object, external stable object,
built-in object, frozen object, or external meta-generated object, injection
must be rejected.

## v0.6 Implementation Note

The current `lang_build` implementation still represents:

```text
let t: type = uint8
```

as a placeholder `TypeObject` because type-value evaluation, `TypeValueId`, and
writable-place checking do not exist yet. This is a v0.6 placeholder
representation, not final semantics. Long-term, `let t: type = uint8` is an
ordinary binding of symbol/place `t` to the existing type value `uint8`, not
fresh type generation and not symbol aliasing.

## Non-Goals

This note does not implement or specify:

- canonical `TypeValueId`;
- full type-value equality;
- full alias forwarding evaluation;
- writable-place or injection-place lifetime checking;
- field access evaluation;
- access-tree scanning;
- borrow/lifetime checking;
- `ref` / `share` type normalization;
- generic meta execution;
- HIR or codegen.