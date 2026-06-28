# Mechanical Return Normalization and Error Policy

**Status: Non-normative future design. Not implemented as current parser, normalizer, policy checker, type checker, effect checker, or runtime behavior.**

This document specifies the future *mechanical return normalization* layer: how a
function call's return value is normalized at the return slot, and how default
error handling is a mechanically inserted, source-expressible action rather than
a compiler-intrinsic exception mechanism. Its central claim is that default error
propagation is expressed through ordinary symbols, policy, alias/binding, and
meta invocation — not through a built-in exception channel.

It is a future design note. It is not current public language behavior, not an
implemented pass, not a parser or normalizer rule, and not a current type or
effect checker. The document is self-contained: it does not require the reader to
assemble its meaning from other documents.

## 1. Purpose

A function's default capability is not pure. An ordinary function may, by
default, `panic` or `error`. But `error` must not be understood as a
compiler-intrinsic exception mechanism. Default error propagation is a piece of
mechanically inserted, source-expressible return-value normalization.

The compiler's only privilege is to insert a fixed-form normalization action at
the return slot of a call. Once inserted, the concrete behavior is still decided
by ordinary symbol lookup, policy filtering, trait/type predicates,
alias/binding, and callable execution.

The core structure of return normalization is:

```text
r is_val?
  true  => inspect first-order type T of r
  false => return r unchanged
```

If `r` is a value, the first-order type is inspected:

```text
<T: type>(r: T) {
  if T has Error:
      unwrap or propagate through the visible Error handler
  else:
      r
}
```

Here `T has Error` is a guarded compile-time predicate. Only when the predicate
is true does the error-handling branch get entered. The branch that is not
entered creates no symbol-lookup obligation.

This is important: in a `noerror` function, if the return value's type has no
`Error`, the fact that the default error handler is not visible does **not** cause
an error. A compile-time error arises only when the `T has Error` branch is
actually entered and needs to look up / execute the default `Error` handler,
because the `noerror` environment lacks the error policy.

## 2. Return normalization as mechanical lowering

Return normalization is a kind of mechanical source-level lowering. It is
analogous to automatic argument-pass insertion, but it acts on the return slot:

```text
argument slot:
  insert pass action

return slot:
  insert value/error normalization action
```

It is not type checking itself, not exception syntax, not a runtime catch, and
not macro expansion. It is an ordinary meta-action framework inserted during the
normalization / lowering stage.

Conceptually:

```text
normalize_return(r):
  if !is_val(r):
      r
  else:
      <T: type>(r: T) {
          if T has Error:
              r? through visible Error handler
          else:
              r
      }
```

This is design pseudocode. It does not require any current syntax.

## 3. Value returns vs non-value returns

Automatic error normalization applies only to value returns. Non-value objects
return unchanged.

```text
Only value returns enter automatic error normalization.
Non-value material is passed through unchanged.
```

Non-value material includes type objects, namespace objects, meta objects,
pattern objects, and rank/type material. These must not be subjected to default
error unwrapping. This avoids mistaking type/meta/pattern material for a runtime
result-like value.

## 4. Error carrier detection: `T has Error`

`T has Error` is a guarded compile-time predicate over the first-order type value
of the return. It asks whether the returned value's type carries an `Error`
carrier shape. It is not a runtime test, and it does not, by itself, resolve any
`Error` handler — it only decides whether the error branch is entered.

The predicate is evaluated over the first-order `TypeValueId` of `r`, not over its
source symbol name.

## 5. Guarded branch evaluation

The `T has Error` branch must be guarded / lazy. The two branches must not be
eagerly resolved for all their symbols.

```text
In a noerror function, eagerly resolving Error in the unselected branch
would reject ordinary non-error returns.
```

The correct rule:

```text
If T has Error is false:
  do not enter the error branch
  do not resolve Error
  do not require error policy

If T has Error is true:
  enter the error branch
  resolve Error under current policy
  fail if no usable Error handler exists
```

A non-error return must never incur an `Error` lookup obligation just because the
mechanical normalization framework contains an error branch.

## 6. Error handler lookup

`Error` is not a compiler-intrinsic exception channel. It is a lookupable
symbol / handler / object. Default error propagation is conceptually:

```text
handler = lookup(Error)
handler.handle(error, self)
```

The default `Error` handler's implementation may call the current function's
error-return capability, for example:

```text
self..return(error)
```

But that step is the behavior of the default handler, not a hardcoded compiler
intrinsic.

### 6.1 `self..return(d)` — the function object's built-in return capability

`self..return(d)` is the current function object's built-in return capability.
It is lookupable as a function associated with the anonymous type of `self`, but
its semantic effect is special:

1. **Local pattern/type-check channel**: it completes the current branch with
   `Done(unit)`. The branch contributes no further pattern material to the
   same-level continuation. `unit` is absorbed as the zero element of `+`.

2. **Enclosing function return accumulator**: it contributes `Done(D)` to the
   final return accumulator, independently of the local branch pattern space.

3. **Lifetime postcondition**: it consumes / closes the return-relevant mutable
   capability of `self`. Later same-block code cannot borrow that capability
   again. The lifetime checker trusts the declared postcondition — it does not
   inspect implementation bodies to rediscover this fact.

Thus when `Error.handle(e, self)` calls `self..return(error)`, the result is not
an exception jump or a compiler intrinsic. It is an early return through the
function object's exposed return capability. The error handler remains an
ordinary lookupable symbol subject to policy constraints; only the return
capability itself carries the special semantic effect.

Thus `r?` expands conceptually as:

```text
r? expansion:
  match r {
    error e => Error.handle(e, self)
    val v   => v
  }
```

where `Error` comes from ordinary symbol lookup and is constrained by the current
policy environment.

## 7. Ordinary function behavior

An ordinary function has, by default, the error-return capability. Therefore the
default `Error` handler is visible and executable, and automatic propagation is
inserted and valid when the return type has an `Error` carrier.

## 8. `noerror` behavior

`noerror` does not mean "values containing `Error` may not appear in the body".
It means the current function's default error-return capability is unavailable,
and the default error-propagation handler should not be visible / executable
under the current policy.

```text
noerror removes the ambient default error-return capability.
It does not ban Error-shaped data.
It bans implicit error propagation through the default Error handler.
```

An annotation such as:

```text
(): noerror
```

makes the lookup / execution of the default `Error` handler be blocked by policy.
If automatic return normalization actually enters the `T has Error` branch and
can only rely on the default `Error` handler, a compile-time error is produced.

## 9. Local Error binding and active propagation

Because `Error` is an ordinary lookupable symbol, a local alias/binding may
intercept an unqualified `Error` lookup. This lets a `noerror` function turn an
error into an ordinary return value explicitly, instead of using the default
error-return capability.

Conceptually:

```text
let Error === MyPureResultHandler
```

or an equivalent local binding, so that `Error.handle(e, self)` no longer calls
`self..return(error)` but instead constructs an ordinary value, for example:

```text
Result::Err(e)
```

A `noerror` function can therefore still handle values that contain `Error`, as
long as the handling constructs an ordinary value rather than invoking default
error propagation.

The two paths must be distinguished:

```text
default propagation:
  error case uses current function's error-return capability
  requires error policy
  unavailable in noerror

active propagation / value conversion:
  error case constructs an ordinary return value
  does not require error-return capability
  can be valid in noerror
```

A local alias/binding may only intercept the `Error` symbol lookup. It cannot
forge the ambient error-return capability. That is, this symbol can be affected
by a local binding:

```text
Error
```

but this still requires a real error policy / capability:

```text
self..return(error)
```

A custom handler that wants to be legal in a `noerror` environment must therefore
avoid calling the error-return capability.

This resembles Rust-style `Result` propagation, but it is not a copy of Rust:

```text
By rebinding Error to a pure handler, a noerror function can convert an
Error-shaped carrier into an ordinary sum/value return. This resembles explicit
Result-style propagation, but in this language it arises from symbol binding
and policy-controlled handler lookup rather than from a built-in Result type.
```

Three stable cases summarize the model:

```text
1. Ordinary function, return value type has Error:
   default Error handler is visible
   automatic propagation is inserted and valid

2. noerror function, return value type has Error, no custom Error handler:
   default Error handler is not visible/executable
   entering the error branch is a compile-time error

3. noerror function, return value type has Error, local Error binding points to pure handler:
   Error is handled as ordinary data
   function still satisfies noerror
```

## 10. Policy planes involved

This design depends on future policy checking; it does not assume a complete
error-policy checker exists. The relevant policy planes:

- **Symbol visibility policy** controls whether the `Error` handler can be found.
- **Body-entry policy** controls whether the found handler may execute.
- **Return-object policy** describes the produced object.
- `noerror` changes the current capability / policy environment so that the
  default `Error` handler is excluded or not executable.

These are future design statements, not a description of an implemented policy
checker.

## 11. Relation to meta object invocation

Automatic return normalization should ultimately reuse the formal meta object
invocation model rather than becoming a return-specific compiler oracle. The
steps:

```text
T has Error
Error handler lookup
handler invocation
branch guarding
```

should all be expressed through the unified policy-aware lookup and invocation
mechanism.

This document does not define the full meta object invocation model; it records
the dependency:

```text
return normalization depends on:
  is_val
  first-order TypeValueId
  T has Error predicate
  guarded branch evaluation
  policy-aware Error lookup
  callable body-entry checking
```

## 12. Non-goals

```text
No parser syntax change.
No current normalizer behavior change.
No Rust implementation change in this PR.
No full effect checker.
No full noerror checker.
No full panic/error policy lattice implementation.
No runtime exception mechanism.
No built-in Result type.
No catch/throw semantics.
No current type checker.
No macro system.
No eager branch lookup.
```

## 13. Relationship to other documents

The documents below are adjacent design. They do not define the return-normalization
model specified here, and this document does not depend on them for its meaning.

- `meta-object-invocation-and-policy-reduction.md` — the unified policy-aware
  lookup and invocation engine that `Error` handler lookup and branch guarding
  should reuse.
- `pattern-normalization-and-first-order-overload.md` — provides `is_val` and the
  first-order type information that `T has Error` is evaluated over.
- `type-values-places-and-alias-forwarding.md` — defines `TypeValueId`, over which
  `T has Error` is computed.
- `mechanical-argument-passing-and-move-fixed-point.md` — the argument-slot
  counterpart of this return-slot normalization; both are mechanical source-level
  lowering actions.
- `policy-visibility-symbols.md` — the overall policy model whose visibility /
  body-entry / return-object planes gate `Error` handler lookup and execution.
