# Function Object Self and Return Capability

**Status: Future design boundary. Not current implementation behavior.**

## 1. Purpose

This document defines the design boundary for `self`, the built-in return
capability, and the lifetime contract associated with early function return.
It does **not** claim that any of this is currently implemented.

The content here is a constraint target — later implementation phases that
introduce lifetime checking, borrow states, or return-capability calls must
respect these design invariants.

## 2. `self` as implicit function-object parameter

Every function receives an implicit first parameter: the function object
itself.

```text
self
```

This is a positional slot, not a user-visible name. The user does not write
`self` in the argument product — it is injected by the invocation mechanism
after the call entry `()` has been resolved.

`self` is **not** part of `ProductObject`, `ArgProductShape`, or
`RawArgShape`. These represent only the explicit user-supplied argument
product.

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
