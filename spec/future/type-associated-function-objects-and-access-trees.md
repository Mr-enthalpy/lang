# Access Trees and Field Projection

**Status: Future design note. No access-tree construction, field access
evaluation, borrow checking, or lifetime checking is implemented.**

This note records the v0.6 namespace-graph implications that future access-tree
work must preserve.

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

## Binding Versus Alias

`let T: type = uint8` is ordinary type-value binding:

```text
T is a new symbol
value(T) == value(uint8)
T is not the same symbol as uint8
```

`let T === uint8` is symbol alias / forwarding. It is not ordinary type-value
binding.

For `let T: type = ... |> struct`, `struct` returns a fresh generated type
value. That fresh type value owns/provides the type-associated namespace where
field functions are installed.

Future generic/meta-generated type expressions such as `(int)Vec::std` should
return stable type values. Separate bindings of the same generated type value
remain distinct symbols unless declared with `===`.

## Non-Goals

This note does not implement or specify:

- canonical `TypeValueId`;
- full type-value equality;
- field access evaluation;
- access-tree scanning;
- borrow/lifetime checking;
- `ref` / `share` type normalization;
- generic meta execution;
- HIR or codegen.
