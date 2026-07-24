# Type-Associated Function Objects and Access Trees

**Status: Future design note. No access-tree construction, field access
evaluation, borrow checking, lifetime checking, full meta execution, or
canonical type-value equality is implemented.**

This note records the v0.6 namespace-graph implications for field-access and
access-tree work. The canonical type-value / place / alias-forwarding /
writable-place semantics are specified in
`spec/design/symbol-world/type-values-places-and-alias-forwarding.md`; this note only keeps a
short summary and the field/access-tree specifics that build on that
distinction.

## Field Functions and Projection Spaces

Fields and member-like operations are function objects installed in a
type-associated companion space. A field is the unary special case; a
member-like operation may consume a receiver plus ordinary remaining
arguments:

```text
field::T        : T       -> field
field::ref::T   : T ref   -> field ref
field::share::T : T share -> field share
push::T         : (T, value) -> result
```

These signatures display only the explicit call-site Product. Every listed
function object also receives itself in invocation slot 0. Thus an associated
member-like function written directly as a closure has its function object as
the first written formal, the `T` object as the second written formal, and any
remaining arguments afterwards. There is no separate method-receiver calling
convention.

The first-class surface constructor is:

```lang
.field
```

and normalizes to a function object shaped as:

```lang
(self, val: T, ...args) { (val, args) |> field::T }
```

Thus `E.field` mechanically lowers to `E |> .field`; `.field` itself is
independently storable/transportable. After that one lowering, `.field` is an
ordinary expression: `E |> .field P` and `E |> d P` (where `d` is bound to
`.field`) must use exactly the same general pipe/product binding path. No rule
may inspect `DotClosureLowering` provenance to absorb `P`, end a target, or
override the ordinary continuation and legality-repair rules. Compact
`E.field P` likewise lowers `E.field` first and then resumes the general
expression rules. `...args` is a Pattern remainder matcher only. Existing
product normalization forwards the bound remainder; no pack type or unpack
operator is introduced. The generated `self` formal binds the implicitly
injected field-function object; `val` remains the first explicit receiver
argument.
`E..field(product)` remains the direct member-call sugar.

An ordinary let-shaped declaration consumed by `struct` contributes its
initializer as Val2 material under the current Pattern owner:

```lang
let virtual = (self_virtual, object: T) => { ... };
let method = (self_method, object: T, ...args) => { ... };
```

The first is virtual-field-like and the second is member-like only by explicit
parameter shape. Both remain ordinary function objects. By contrast:

```lang
let () = (object: T) => { ... };
```

installs the current owner's call entry, so `object` is the implicitly supplied
slot-0 caller by position. A mismatch between the invoked object type and this
first formal is an ordinary invocation type error, not a separate declaration
rule.

`field::T` is value semantics (`T == T move`). Borrowed field access must begin
from an explicit borrow form, for example:

```text
val ref.field1.field2
val share.field1.field2
```

This document does not specify evaluation or lowering for those forms.

Because source navigation is inner-to-outer, `ref::T` denotes the `ref` child
under owner `T`. A construction authorized to add children of `T` can create
that path. `T::ref` would instead place `T` below an outer `ref` owner and is
not the same injection.

Automatic `ref` / `share` argument passing constructs a borrow object and moves
the borrow handle. Moving a borrow handle keeps the same parent/origin and does
not deepen the access tree, so access-tree depth does not grow through argument
passing. The mechanical pass-insertion semantics are specified in
`spec/design/mechanical-lowering/mechanical-argument-passing-and-move-fixed-point.md`.

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
pipeline — see `spec/design/symbol-world/type-values-places-and-alias-forwarding.md`.

## v0.6 Implementation Note

The current `lang_build` slice still represents some `let t: type = uint8` cases
using placeholder `TypeObject` payloads, because `TypeValueId` and writable-place
checking do not exist yet. This is a v0.6 placeholder, not the final semantics;
the intended model is documented in
`spec/design/symbol-world/type-values-places-and-alias-forwarding.md`.

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
