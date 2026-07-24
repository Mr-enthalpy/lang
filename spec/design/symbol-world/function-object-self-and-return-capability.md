# Function Object Self and Return Capability

**Status: Canonical semantic boundary with typed frontend/build substrate.
End-to-end invocation and lifetime behavior remain future work.**

**Canonical definition.** This document is the authoritative definition of
`self`, the return capability, and its lifetime contract. Other documents
that mention `self..return(d)` reference this document rather than redefining
the semantics:

- Pattern-space consequences: `spec/design/patterns-overload/static-pattern-spaces-and-extraction-chains.md` §6.3.1, §7.5
- Mechanical lowering consequences: `spec/design/mechanical-lowering/mechanical-return-normalization-and-error-policy.md` §6.1

## 1. Purpose

This document defines the design boundary for `self`, the built-in return
capability, and the lifetime contract associated with early function return.
The normalized formal-frame projection, generated-helper shape, restricted
callable arity, return-target substrate, and mutability product-order carrier
implement this positional boundary. Full name resolution, callable-object
materialization, return-capability execution, and lifetime checking remain
future work.

The content here is a constraint target — later implementation phases that
introduce lifetime checking, borrow states, or return-capability calls must
respect these design invariants.

## 2. `self` as implicit function-object parameter

Every callable, ordinary or in-place, has an invocation-frame slot 0 containing
the callable object itself.

```text
self-position
```

The semantic role is positional, not tied to a reserved name. If a parameter
position is written, the first written formal is the explicit Pattern/binder
for this self-position. The source spelling may be `self`, `this`, `callable`,
or any other legal Pattern; `self` is only the conventional spelling.

The corresponding actual is never written in the call-site argument Product.
It is injected by the invocation mechanism after the call entry `()` has been
resolved.

`self` is **not** part of `ProductObject`, `ArgProductShape`, or
`RawArgShape`. These represent only the explicit user-supplied argument
product.

### 2.1 Implicit Passing, Explicit Formal Position

`self` is implicitly passed but occupies an explicit formal position.

```text
call-site explicit product:
  contains only user-supplied explicit arguments
  does not contain self

callable formal frame:
  slot 0 = function-object self-position
  slot 1..n = user parameter positions

invocation frame:
  resolves the callable / call entry
  injects the function object itself into slot 0
  passes the explicit user product into slots 1..n
```

The first written formal position denotes the function-object self-position,
not an ordinary user parameter. Only written positions after the first consume
the explicit call-site Product. A callable that writes no formal position still
has slot 0 as an unbound self-position and has no user argument slots.
Declaration-context `()` call-entry definitions follow the same invocation
model: `()` is the call entry, the explicit user product is empty, and the
invocation frame injects the function object into slot 0.

For declaration-context call-entry injection, the self-position may have a
non-anonymous type such as `T ref` in:

```lang
let ()::ref::T = (self: T ref) => { ... }
```

The same rule still holds: self is slot 0 and is not part of the explicit
argument product.

`self` is not an invisible ambient environment and not an ordinary
user-supplied argument. It belongs to the invocation / callable frame boundary,
not product arity, product flattening, canonical argument products, or meta
instance keys.

## 3. Implicit self borrow

Normal continuation within a function body implicitly requires the current
block/function `self` capability to remain borrowable. This is the ordinary
borrow that allows the function body to:
- access the function object's own fields or captured state;
- call further methods or capabilities on `self`;
- pass `self` as a receiver to other functions.

The implicit borrow is not written by the user. It is an automatic consequence
of being inside the function body.

## 4. `return` as a built-in capability under the anonymous type of `self`

The current function object has a built-in return capability:

```text
return
```

This capability is lookupable under the anonymous type of `self`. It is not an
operator, not a keyword, and not a compiler intrinsic escape hatch. It is an
ordinary callable value exposed by the function object's type-associated
namespace.

The self-position is not a path segment and is not identified by the spelling
`self`. A first-formal binder makes the injected object visible under that
source binder. Independently, the corresponding anonymous function-object type
anchor is described as `Self` when discussing type-associated lookup or
diagnostic rendering.

The return capability is associated with the function-object type / self-frame.
A targeted return form such as:

```text
t return
```

selects the return capability associated with the callable frame whose
self-position / type anchor is `t`. This is the semantic basis for
specified-level return; it is not syntax magic, an operator, or a compiler
escape hatch.

## 5. `self..return(d)` — semantics

A call to `self..return(d)` has three semantic effects:

### 5.1 Local branch completion with `Done(unit)`

In the local pattern/type-check continuation, the branch completes with
`Done(unit)`. No further same-level pattern material is contributed by this
branch. `unit` is later absorbed as the zero element of `+`.

### 5.2 Final return accumulator contribution

Simultaneously, `Done(D)` is contributed to the enclosing function's return
accumulator. This is independent of the local branch pattern space — the
accumulator does not need to know which branch produced the value, and the
local extraction/type-check path does not need to know the final accumulator
value.

### 5.3 Lifetime postcondition

`self..return(d)` **declares** that the return-relevant mutable capability of
`self` is consumed / closed after the call. The lifetime checker operates on
the principle of trust: it checks the call precondition before the call, then
trusts the declared postcondition after the call. It does **not** inspect
implementation bodies to rediscover control-flow facts. This is a
name-and-contract lifecycle system — the capability's availability is stated,
not inferred from body analysis.

## 6. Consequence: no more code after `self..return(d)`

Because the return capability consumes `self`'s mutable borrow, any subsequent
same-block code that implicitly borrows `self` is ill-formed.

The canonical repair is:

```text
self..return(d);
()
```

where `()` is the branch's explicit unit return — permissible because `()`
does not require a mutable borrow of `self`.

## 7. Relation to `Error.handle`

`Error.handle(e, self)` may call `self..return(error)` as its default
behavior. This is not an exception mechanism. It is an ordinary call through
the function object's return capability, subject to the same lifetime
postcondition: after the error handler invokes `self..return(error)`, the
current branch is complete, `Done(unit)` is contributed to the local pattern
space, and `Done(error)` is contributed to the final return accumulator.

## 8. Not implemented

The following are future work and must not be implemented in the current
construction phase:

- `self..return(d)` as a runtime capability object;
- the anonymous function-object type that carries `return`;
- the lifetime checker that enforces the return-capability postcondition;
- the final return accumulator;
- the concrete representation of `Done` in later semantic IR;
- `self` implicit borrow tracking.
