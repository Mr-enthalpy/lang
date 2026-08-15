# Type-Associated Function Objects and Access Trees

**Status: Future design note. No access-tree construction, field access
evaluation, borrow checking, lifetime checking, full meta execution, or
canonical type-value equality is implemented.**

This note records the v0.6 namespace-graph implications for field-access and
access-tree work. The canonical type-value / place / borrow-view /
writable-place semantics are specified in
`spec/design/symbol-world/type-values-places-and-borrow-views.md`; this note only keeps a
short summary and the field/access-tree specifics that build on that
distinction.

## Field Functions and Same-Name Overload Families

Fields and member-like operations are function objects installed in a
type-associated companion space. A field is the unary special case; a
member-like operation may consume a receiver plus ordinary remaining
arguments:

```text
AssociatedSymbol(T, field):
  field : (object: T)       -> field
  field : (object: T ref)   -> field ref
  field : (object: T share) -> field share

AssociatedSymbol(T, push):
  push : (object: T, value) -> result
```

For a `struct`-generated field these are ordinary typed candidate objects in one
associated Symbol, accompanied by an assignment/write candidate over
`T ref × field` whenever field policy admits mutation. `const let`, unqualified
`let`, and `mut let` select the admitted cells of the same general access/
borrow/write family; there is no special semantic field category.

The stage rule is structural:

```text
RuntimeField(f)
  <=> Val1_f != absent
    and Materializable_0(Val1_f)
    and not RequiresStaticPattern(f)

Stage(accessor(f)) = runtime || compile  if RuntimeField(f)
Stage(accessor(f)) = compile             otherwise
```

A type-valued field is compile-only only because it currently fails this
predicate, not because “type/PatternValue field” is a separate category.
Generated candidates use the ordinary context preference
`succ_plain: let > const = mut`; if no plain `let` candidate exists, tied
`const` and `mut` candidates are ambiguous rather than arbitrarily selected.
Owned construction relations propagate Open/frozen state but never mutability:

```text
Open(child)   => Open(parent)
Frozen(parent) => Frozen(child)
mut(child)    does not imply mut(parent)
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
`E..field(product)` remains the direct member-call sugar. Candidate selection
uses the actual receiver Pattern (`T`, `T ref`, or `T share`) in the ordinary
overload family; it does not navigate through a `ref` or `share` child namespace.

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

The value receiver candidate has value semantics (`T == T move`). Borrowed field access must begin
from an explicit borrow form, for example:

```text
val ref.field1.field2
val share.field1.field2
```

This document does not specify evaluation or lowering for those forms.

Explicit `ref` / `share` constructs a borrow object before candidate adaptation;
argument passing only moves that already formed borrow handle. Moving a borrow
handle keeps the same parent/origin and does
not deepen the access tree, so access-tree depth does not grow through argument
passing. The mechanical pass-insertion semantics are specified in
`spec/design/mechanical-lowering/mechanical-argument-passing-and-move-fixed-point.md`.

## Same-Name Candidate Lookup

One associated field Symbol contains every value/ref/share observation
candidate. `ref` and `share` are types/observation kinds in candidate formals and
results, not generated namespace subspaces. A structural field literally named
`ref` or `share` is therefore just another same-name associated Symbol and does
not collide with a projection namespace. Ordinary overload resolution selects
the candidate from the receiver Pattern and Policy.

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
- `let f::(t@) = ...` explicitly creates the prospective child under
  `place(t)`, never `place(uint8)`, because `t` is already a pure type slot. For
  a Symbol `S`, the corresponding place form is `let f::((S ref).type) = ...`.
  `AsType(S)` never recovers a place. Type-value
  equality must not canonicalize extension targets, and a `type`-kind symbol may
  own a companion namespace place distinct from the type value it stores.
- There is no place-forwarding declaration form. Every binding allocates its own
  place, so no second name reaches `place(uint8)`. Where shared observation is
  wanted, the value held is a borrow view (`ref` / `share` / `@`), and its
  capability never exceeds the underlying place's own. A missing final child
  still has stable `ProjectionSlot(parent, selector)` identity: `let` may instantiate
  it, while bare `=` may only write `Some(existing)`.
- That prospective coordinate is not the target identity of a borrow already
  formed from a resident child. Parent wholesale replacement may invalidate the
  old borrow, but never redirects it to a new child at the same coordinate;
  only `rebind` selects a new target.
- `Writable(place)` and `Open(Value(place))` are independent. A frozen type slot
  may remain writable for wholesale replacement, and an Open value may be
  extended purely without a writable carrier.

This is only a summary. For the canonical `TypeValueId` / `PlaceId` / `SymbolId`
distinction — including the object normal form, the borrow views, writability,
construction-lineage Open, and the namespace member-creation/write pipeline — see
`spec/design/symbol-world/type-values-places-and-borrow-views.md`.

## v0.6 Implementation Note

The `lang_build` semantic spine currently implements only the first-order
substrate: `TypeValueId` exists as the stable core root and current observations
still travel through per-carrier `Val2` places. The target complete identity is
`Addr(Norm_type(tau))` over `tau=<Q,V_T>`; preserving `V_T` in copied/extended
snapshots remains implementation migration. Writability checking, borrow-view evaluation, and
the field-function / access-tree machinery of this note remain future work;
the identity model and its implemented/future split are documented in
`spec/design/symbol-world/type-values-places-and-borrow-views.md`.

## Non-Goals

This note does not implement or specify:

- type-value identity (the first-order `TypeValueId` root and the canonical
  observation `Addr(Norm_type(tau))` are owned by
  `type-values-places-and-borrow-views.md`);
- migration of the remaining first-order type comparisons to full by-value
  comparison;
- full borrow-view evaluation;
- writability or extension-place lifetime checking;
- field access evaluation;
- access-tree scanning;
- implementation of complete type closures `tau = <Q,V_T>`, their optional
  binder-aware form `bind alpha.<Q,V_T[alpha]>`, and direct-home TypeMember
  classification; target type-as-callee lookup is already
  `CallSpace(tau)=V_T` and never performs
  defining-Symbol or carrier-provenance recovery;
- borrow/lifetime checking;
- `ref` / `share` type normalization;
- generic meta execution;
- HIR or codegen.
