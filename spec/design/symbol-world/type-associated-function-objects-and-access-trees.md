# Type-Associated Function Objects and Access Trees

**Status: Future design note. No access-tree construction, field access
evaluation, borrow checking, lifetime checking, full meta execution, or
whole-snapshot type-value identity is implemented.**

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
arguments.

The `struct` registration `Field(T, name, A)` generates the field's complete
associated candidate family under `T`: one by-value accessor, plus for each
borrow observation `ρ ∈ {ref, share}` a triple of policy cells:

```text
GeneratedFieldFamily(T, name, A):

  let
  name(self, object:T)
      -> A
      => default

  const let
  name(self, object:T ref)
      -> A ref
      => default

  mut let
  name(self, const let object:T ref)
      -> A ref
      => delete

  mut let
  name(self, mut let object:T ref)
      -> A ref
      => default

  const let
  name(self, object:T share)
      -> A share
      => default

  mut let
  name(self, const let object:T share)
      -> A share
      => delete

  mut let
  name(self, mut let object:T share)
      -> A share
      => default
```

`default` cells expose their operation; `delete` cells exist so the policy
lattice can reject the mutation-shaped call at selection time instead of
silently degrading the selected operation. Erasing policy detail, this is the
familiar three-signature summary:

```text
AssociatedSymbol(T, field):
  field : (object: T)       -> field
  field : (object: T ref)   -> field ref
  field : (object: T share) -> field share

AssociatedSymbol(T, push):
  push : (object: T, value) -> result
```

The full candidate family is normative; the summary is its policy-erased
projection and is not the sole specification. For a `struct`-generated field
these are ordinary typed candidate objects in one associated Symbol.

#### Family registration: producer side of `StructuralDefault`

Every candidate of the generated schema is registered in one stable
call-site candidate family. The identity key is the stable self-observable
anchor `CoreAnchor(Q_T)`, not the whole `Q` snapshot
(`CoreAnchor(Q) = CanonicalSelfPatternRoot(Q)`, canonical
`symbol-first-meta-construction-and-pattern-injection.md` §2.1):

```text
StructuralFamily(T, name, A)
  = StableFamilyId(CoreAnchor(Q_T), name, StructuralDefault)

c ∈ GeneratedFieldFamily(T, name, A)
---------------------------------------
CandidateFamily(c) = StructuralFamily(T, name, A)
```

where `Q_T = Core(τ_T)`. This gives the family-stability theorem:

```text
StructuralFamilyStability:
  CoreAnchor(Q') = CoreAnchor(Q) ∧ same registered structural field name
  ⇒ StructuralFamily(Q', name, A) = StructuralFamily(Q, name, A)
```

An `extend` that adds unrelated virtual helpers (so `Q ≠ Q'` but
`CoreAnchor(Q') = CoreAnchor(Q)`) therefore keeps every generated structural
candidate's identity: P-internal extraction over the new snapshot still
filters exactly the inherited generated cells. P-internal extraction
(`AtomicExtract_P`, canonical
`pattern-values-relational-semantics-and-extraction.md` §3.1) applies
`CallSiteFamilyFilter = StructuralDefault` before C0; that filter preserves
exactly the candidates registered under this family identity. This is the
registration that closes the consumer/producer loop: the extraction rule
names the family, and the generator assigns every generated cell to it.

Three attributes of a generated cell are distinct and must never be conflated
under the single word `default`:

```text
CandidateFamily(c)
    call-site family identity (what the StructuralDefault filter preserves)

per-cell policy disposition
    => default   exposes the operation's mechanical implementation body
    => delete    rejects the mutation-shaped call at selection time

declaration-side fallback
    =>[[fallback]]   DeclarationCandidatePolicy after FullyAdmissible A
                     (overload-resolution-design.md §2.3)
```

Family identity decides which candidates a call site sees; the per-cell
disposition decides what the generated body does once selected; the fallback
annotation decides how an already-admissible candidate participates after
`A`. None of the three implies another.

#### Field write candidates are not `AssignmentFamily(T)`

Where field policy permits mutation, the generator also contributes field
write candidates shaped `T ref × A`. They are a field-specific setter family
under the ordinary `.=` / `=::adl` candidate domain:

```text
FieldWriteFamily(T, name, A)
  ⊆ Candidates(=::adl)

  =
  (self,
   mut let object : T ref,
   name : A)
  -> unit
  => default
  (+ policy-deleted const/let cells, same schema style as above)
```

The universal assignment family remains exactly:

```text
AssignmentFamily(U):
    U ref × U -> unit        (canonical §4.5.1)
```

The two families never coincide, for any field type:

```text
∀A. FieldWriteFamily(T, name, A) ≠ AssignmentFamily(T)
```

Their target operations differ: `AssignmentFamily(T)` writes
`Target(receiver)` itself, while `FieldWriteFamily(T, name, A)`
writes/customizes `field(receiver, name)`. Even when `A = T` — e.g.
`struct Node { next : Node }`, where both signatures may be
`Node ref × Node -> unit` — `node = rhs` and `node.next = rhs` remain
distinct operations. The field family's identity is

```text
FieldWriteFamily(T, name, A)
  = SetterFamily(CoreAnchor(Q_T), name, StructuralDefault, A)
    -- ⟨structural-field identity, selector, value type, setter-family kind⟩
    -- never the parameter shape alone
```

The parameter-shape statement is separate and weaker:

```text
A = T
⇒
FormalShape(FieldWriteFamily(T, name, A))
  =
FormalShape(AssignmentFamily(T))
    -- coincident formal shape (T ref × T -> unit)
    -- is never family identity
```

Selection uses the selector through the LHS navigation: `lhs.name = rhs`
first lowers the dot expression, and the `=::adl` candidate set over that
navigated receiver contains only the field-scoped family; `lhs = rhs`
contains only `AssignmentFamily(T)`. The two candidate sets are therefore
disjoint in every call, and shape coincidence creates no ambiguity.
They are two ordinary associated candidate families reachable through the same
`.=` entrance; only the selected `default` body performs the universal write
judgment. Field write candidates are admitted only where field policy permits
mutation. `const let`, unqualified `let`, and `mut let` select the admitted
cells of the field's ordinary overload family; there is no special semantic
field category.

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
Open authority does not propagate along owned field relations; each
PatternValue's `OpenHere_Σ` is determined independently by stack-relative
coordinate equality (canonical §12.1.1). Mutability does not propagate:

```text
mut(child) does not imply mut(parent)
mut(parent) does not imply mut(child)
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
For a borrowed receiver the ordinary family reached is the derived type's own
forwarding member (`Derived-Type Associated Forwarding` below), which performs a
fresh ordinary invocation of the base family; the dot lowering itself never
inspects the external receiver's type context.

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

This document does not separately define evaluation or lowering for those
forms; their semantics is given by the canonical `ProjectionSlot` borrow-lifting
law (`type-values-places-and-borrow-views.md` §2.3).

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

## Derived-Type Associated Forwarding

A derived type construction `D(T)` — `T ref`, `T share`, and any future derived
construction — does not gain associated capabilities by copying the original
type's members. Every member of `V_(D(T))` must truly belong to the derived
type's own structural level:

```text
τ = ⟨Q, V_τ⟩

F ∈ V_τ
=>
Anonymous(F)
∧ DirectClassifierHome(F) = TypeMemberScope(Q)
```

In particular, `F ∈ V_T` never implies `F ∈ V_(T ref)` or `F ∈ V_(T share)`:
those are three complete type values with three independently home-checked
callspaces. Foreign-member injection into a derived type's callspace is
forbidden (`NoForeignTypeMemberInjection`,
`symbol-first-meta-construction-and-pattern-injection.md` §2.1). The correct
mechanism is derived associated forwarding: the derived type generates its own
forwarding member

```text
ForwardAssoc(D(T), name)
```

satisfying:

```text
ForwardAssoc(D(T), name) ∈ V_(D(T))

DirectClassifierHome(
  ForwardAssoc(D(T), name)
)
=
TypeMemberScope(Core(τ_(D(T))))
```

so the forwarder is a real ordinary member of the derived type, homed in the
derived type's own level. Its behavior is an ordinary call:

```text
(args) |> name::D(T)

    ↓ selected forwarding callable

(args) |> name::T
```

For fields this gives the two instances:

```text
inner::(T ref)   -> forward -> inner::T
inner::(T share) -> forward -> inner::T
```

with no `T ref -> T` value conversion: the original receiver passes through
unchanged (`object : T ref |> inner::T`), and `inner::T`'s own overload family
already contains `inner : T ref -> A ref`, so the inner ordinary overload
resolution selects the correct candidate.

The defining equation is:

```text
ForwardAssocCall:
  Invoke(name::D(T), args)
    =
  Invoke(name::T, args)
```

not:

```text
CopyAssociatedMembers(T, D(T))
```

Forwarding is a new ordinary invocation, not a reopened candidate set:

```text
resolve name::D(T)
-> select forwarder uniquely
-> execute forwarder body
-> body performs a new ordinary invocation of name::T
```

It is not fallback, candidate reopening, or late adaptation.

### Candidate-domain-preserving registration

The family-level equation above is the summary; the normative specification
is candidate-domain preserving. Each admissible base candidate produces one
forwarder, so the derived candidate domain mirrors the base candidate domain
instead of collapsing to a single universal trampoline:

```text
c ∈ Candidates(name::τ_base)

ForwardRegistration(D, τ_base, name, c)
    -> f

Applicable(f, args)
  iff
  Applicable(c, args)

ForwardBaseSnapshot(f)
  = the complete τ_base snapshot captured when D(τ_base) was formed
```

The forwarder `f` is a real ordinary callable homed in `V_(D(T))`
(`DirectClassifierHome(f) = TypeMemberScope(Core(τ_(D(T))))`). Its body
performs a new ordinary invocation of `c` against `ForwardBaseSnapshot(f)`.
The applicability equivalence lets a derived caller discover at selection
time whether the base family has an applicable candidate — not after
committing to a universal trampoline that then discovers no base candidate
fits.

Coherence requirements:

```text
D(τ) = τ
  => no self-forwarder / no duplicate

Norm_type(D₁(τ)) = Norm_type(D₂(τ))
  => same normalized forwarding set
     -- D(τ) is a complete type value; coherence compares Norm_type,
        never the ordinary Norm(Object) (type-values §2.2)

ForwardedNames(D)
  is explicit
```

`ForwardedNames(D)` restricts which associated names a derived type
forwards. This prevents write capability from leaking from `T ref` to
`T share` through generic forwarding: `T share` forwards only the
share-admissible subset of inherited associated names, never the
ref-only write family.

The `.field` lowering (above) resolves the generated hole `T` and, for a
borrowed receiver `r : X ref`, lands on `r |> inner::(X ref)`. The connection

```text
inner::(X ref) -> inner::X
```

is what the specification must provide: nothing in the independent dot lowering
can by itself reach the `inner : X ref -> A ref` candidate inside `inner::X`.
The full chain therefore has four layers that must not be merged:

```text
surface / dot lowering
    r.inner
      ↓
    r |> inner::(X ref)

derived associated forwarding
      ↓
    r |> inner::X

ordinary overload resolution
      ↓
    inner : X ref -> A ref

projection semantics
      ↓
    Ref(ProjectionSlot(Target(r), inner))
```

In particular:

```text
derived associated forwarding
≠
borrowed projection
```

Forwarding only locates the correct associated callable family; borrowed
projection decides the `Ref(ProjectionSlot(...))` form (canonical
`type-values-places-and-borrow-views.md` §2.3) rather than materializing the
field value and then forming `A ref`.

## Type Values, Places, and Injection (summary)

Field functions live in a type-associated companion *place*, which is distinct
from the type *value* the bound symbol stores. The access-tree work in this
document therefore depends on three identities being kept separate:

- a name (`SymbolId`),
- a writable location (`PlaceId`),
- a canonical type value (the implementation index root currently called
  `TypeValueId`; the canonical semantic type value is the rank-indexed closure
  `tau = <Q, V_τ>` in `type-values-places-and-borrow-views.md`).

The consequences that field/access-tree work must preserve:

- `let t: type = uint8` creates a fresh symbol and a fresh current-level writable
  place whose type value equals `uint8`'s. `value(t) == value(uint8)`, but
  `place(t) != place(uint8)`. It is not a fresh nominal type and not a symbol
  alias.
- `let f::(t |> (type ref)) = ...` explicitly creates the prospective child under
  `place(t)`, never `place(uint8)`, because `t` is already a pure type slot. For
  a Symbol `S`, the corresponding place form is `let f::((S ref).type) = ...`.
  `AsType(S)` never recovers a place. Type-value
  equality must not canonicalize extension targets, and a `type`-kind symbol may
  own a companion namespace place distinct from the type value it stores.
- There is no place-forwarding declaration form. Every binding allocates its own
  place, so no second name reaches `place(uint8)`. Where shared observation is
  wanted, the value held is a borrow view (`ref` / `share`), and its
  capability never exceeds the underlying place's own. A missing final child
  still has stable `ProjectionSlot(parent, selector)` identity: `let` may instantiate
  it, while bare `=` may only write `Some(existing)`.
- That prospective coordinate is not the target identity of a borrow already
  formed from a resident child. Parent wholesale replacement may invalidate the
  old borrow, but never redirects it to a new child at the same coordinate;
  only `rebind` selects a new target.
- `Writable(place)` and `OpenHere_Σ(Value(place))` are independent. A closed-window type slot
  may remain writable for wholesale replacement, and an open-window value may be
  extended purely without a writable carrier.

This is only a summary. For the canonical `TypeValueId` implementation index
root / `PlaceId` / `SymbolId` distinction — including the object normal form,
the borrow views, writability, construction-authority (`OpenHere_Σ` / `WindowLive_Σ`), and the namespace
member-creation/write pipeline — see
`spec/design/symbol-world/type-values-places-and-borrow-views.md`.

## v0.6 Implementation Note

The `lang_build` semantic spine currently implements only the first-order
substrate: `TypeValueId` exists as the stable core root and current observations
still travel through per-carrier `Val2` places. The target complete snapshot identity is
`Addr(Norm_type(tau))` over `tau=<Q,V_τ>`; under the minimal-change rule
(`type-values-places-and-borrow-views.md` §2.2) ordinary type equality/keying
keeps observing `Core(tau)=Q` by default, while `Addr(Norm_type(tau))` is used
to tell shared-root snapshots apart in transport and in positions the language
has independently frozen to whole-snapshot semantics. Preserving `V_τ` in
copied/extended
snapshots remains implementation migration. Writability checking and borrow-view
evaluation remain future work; the field-function / access-tree semantics are
borrowed from `spec/design/symbol-world/type-values-places-and-borrow-views.md` §2.3
and §5, and the access-tree machinery of this note remains future work.

## Non-Goals

This note does not implement or specify:

- type-value identity (the first-order `TypeValueId` root and the snapshot
  identity `Addr(Norm_type(tau))` are owned by
  `type-values-places-and-borrow-views.md`);
- whole-snapshot comparison is required only at independently specified
  snapshot-sensitive positions;
- full borrow-view evaluation (canonical semantics are in
  `type-values-places-and-borrow-views.md` §5);
- writability or extension-place lifetime checking;
- field access evaluation (canonical projection-slot semantics are in
  `type-values-places-and-borrow-views.md` §2.3 and §5);
- access-tree scanning;
- implementation of complete type closures `tau = <Q,V_τ>`, their optional
  binder-aware form `bind alpha.<Q,V_τ[alpha]>`, and direct-home TypeMember
  classification; target type-as-callee lookup is already
  `CallSpace(tau)=V_τ` and never performs
  defining-Symbol or carrier-provenance recovery;
- borrow/lifetime checking;
- `ref` / `share` type normalization;
- generic meta execution;
- HIR or codegen.
