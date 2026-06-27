# Type-Associated Function Objects and Access Trees

**Status: Future design note. No access-tree construction, field access
evaluation, borrow checking, lifetime checking, full meta execution, or
canonical type-value equality is implemented.**

This note records the v0.6 namespace-graph implications for field-access and
access-tree work. The canonical type-value / place / alias-forwarding /
writable-place semantics are specified in
`spec/future/type-values-places-and-alias-forwarding.md`; this note only keeps a
short summary and the field/access-tree specifics that build on that
distinction.

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

## Type Values, Places, and Injection (summary)

Field functions live in a type-associated companion *place*, which is distinct
from the type *value* the bound symbol stores. The access-tree work in this
document therefore depends on three identities being kept separate:

- a name (`SymbolId`),
- a writable location (`PlaceId`),
- a canonical type value (`TypeValueId`).

The consequences that field/access-tree work must preserve:

- `let t: type = uint8` creates a fresh symbol and a fresh current-level writable
  place whose type value equals `uint8`'s. `value(t) == value(uint8)`, but
  `place(t) != place(uint8)`. It is not a fresh nominal type and not a symbol
  alias.
- `let f::t = ...` injects into `place(t)`, never into `place(uint8)`. Type-value
  equality must not canonicalize injection targets, and a `type`-kind symbol may
  own a companion namespace place distinct from the type value it stores.
- `let t === uint8` is symbol forwarding, not binding. It creates no fresh
  writable place; injection through it follows the forwarded place, and a final
  target that is a built-in / external stable / frozen / inner-escaping place is
  not writable and must be rejected.

This is only a summary. For the canonical `TypeValueId` / `PlaceId` /
alias-forwarding distinction — including the value/place judgments, the
`AliasChain` model, writable-place checking, and the namespace injection
pipeline — see `spec/future/type-values-places-and-alias-forwarding.md`.

## v0.6 Implementation Note

The current `lang_build` slice still represents some `let t: type = uint8` cases
using placeholder `TypeObject` payloads, because `TypeValueId` and writable-place
checking do not exist yet. This is a v0.6 placeholder, not the final semantics;
the intended model is documented in
`spec/future/type-values-places-and-alias-forwarding.md`.

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
