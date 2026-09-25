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
effect checker. Canonical stage, instance lifetime and internal completion laws constrain this
future sketch; it supplies no competing rule for those topics.

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
ordinary returned instance
  -> retain ordinary lifecycle/pass obligations
  -> obtain its ordinary type observation T
  -> guard on T |> has(Error)
  -> branch over the visible Error carrier shape only when the guard is true
```

The guard is lazy. The branch that is not entered creates no symbol-lookup or
policy obligation.

This is important: in a `noerror` function, if the return value's type has no
`Error`, the fact that the default error-return capability is unavailable does
**not** cause an error. A compile-time error arises only when the `T |> has(Error)`
branch is actually entered and the current policy/capability environment does
not permit the return capability used by the default Error behavior.

## 2. Return Normalization as Mechanical Lowering

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

A possible future consumer obtains the returned instance's ordinary type
observation and guards on an explicitly defined Error-carrier predicate.
It then invokes an admitted handler or delivers the ordinary result. This
sketch does not freeze a complete error/effect policy.

## 3. Uniform returned instances

Type, meta, namespace and Pattern results remain ordinary instances. They
are not categorically exempt from movement or lifecycle checks. Any future
Error normalization must define its predicate's applicable domain using
ordinary relations and preserve the canonical result/completion boundary.
A known false guard enters no Error branch; runtime or seal-dependent guards
retain/defer their lawful continuation instead of speculatively running both.

## 4. Error Carrier Detection: `T |> has(Error)`

`T |> has(Error)` is a guarded compile-time predicate over the first-order type
value of the return. It asks whether the returned value's type carries an
`Error` carrier shape. It is not a runtime test, and it does not, by itself,
resolve any `Error` branch material — it only decides whether the error branch is
entered.

The predicate is evaluated over the first-order `TypeValueId` of `r`, not over
its source symbol name. `TypeValueId` remains projection material; it is not an
invocation result or construction identity.

## 5. Guarded Branch Evaluation

The `T |> has(Error)` branch must be guarded / lazy. The two branches must not be
eagerly resolved for all their symbols.

```text
In a noerror function, eagerly resolving Error in the unselected branch would
reject ordinary non-error returns.
```

The correct rule:

```text
If T |> has(Error) is false:
  the error branch is not entered
  Error / default return capability is not required

If T |> has(Error) is true:
  the branch containing return_owner..return(e Error) is entered
  the current policy/capability environment must permit that return capability
```

A non-error return must never incur an `Error` lookup or return-capability
obligation just because the mechanical normalization framework contains an error
branch.

## 6. Error Carrier Branch and Return Capability

The default Error behavior may be represented by the visible Error carrier
branch calling the current callable frame's return capability:

```lang
r |> (branch_self, e Error) {
    return_owner..return(e Error);
} |> (branch_self, val: _) {
    val;
};
```

This is the error-carrier branch inside the guarded `T |> has(Error)` branch. It
is not ordinary value-level `?`, and it must not be confused with extraction-view
`?` from the `e / P` return-normal-form model.

The visible `Error` symbol / predicate decides whether the branch is available.
The return capability itself is not forged by binding `Error`; it belongs to the
current function object's invocation frame.

A library-level Error handler may be factored as an ordinary callable, but the
mechanical example in this document uses the direct source-shaped branch form so
that the capability boundary is explicit.

### 6.1 Callable self and the enclosing return owner

The `return_owner` used in `return_owner..return(e Error)` is not a new
source-level keyword. It is schematic notation for the already resolved
enclosing callable-object identity whose return capability is being targeted.
Every generated helper and branch is itself a callable: if it writes formals,
its first formal binds its own implicitly passed self object. Consequently the
branch-local first formal must not be confused with the enclosing return owner.

The enclosing return owner is not resolved merely as an ordinary user
name. It denotes the current outer function object's invocation-frame position.
The symbol lookup / invocation preparation phase determines this position.

The inserted normalization action may refer to that resolved position without
depending on the source spelling chosen for the outer callable's first formal.

```text
the self role is a position;
`self` is only a conventional binder spelling.
```

### 6.2 `return_owner..return(d)` — the Function Object's Built-In Return Capability

`return_owner..return(d)` denotes the enclosing function object's built-in
return capability. It is lookupable through the resolved function-object
capability position, but its semantic effect is special:

1. The branch has no normal local result contribution; it manufactures no unit.

2. An internal target-qualified completion carries the ordinary payload to
   ReturnPattern delivery. It is not a user-visible Done Object.

3. **Lifetime postcondition**: the return capability declares that the
   return-relevant mutable capability of the enclosing callable object is
   consumed / closed after the call. The lifetime checker trusts the declared
   contract: it checks the call precondition before the call, then trusts the
   declared postcondition after the call. It does **not** inspect implementation
   bodies to rediscover control-flow facts. This is a name-and-contract system —
   checking happens at the call boundary, not through body-level control-flow
   analysis.

Thus when the Error branch calls `return_owner..return(e Error)`, the result is
not an exception jump or a compiler intrinsic. It is an early return through
the enclosing function object's exposed return capability.

## 7. Ordinary Function Behavior

An ordinary function has, by default, the error-return capability. Therefore the
default Error carrier branch is visible and executable, and automatic propagation
is inserted and valid when the return type has an `Error` carrier.

## 8. `noerror` Behavior

`noerror` does not mean "values containing `Error` may not appear in the body".
It means the current function's default error-return capability is unavailable,
and the default error-propagation branch should not be visible / executable under
the current policy.

```text
noerror removes the ambient default error-return capability.
It does not ban Error-shaped data.
It bans implicit default propagation through the default return capability.
```

An annotation such as:

```text
(): noerror
```

makes the use of the default error-return capability be blocked by policy. If
automatic return normalization actually enters the `T |> has(Error)` branch and
requires `return_owner..return(e Error)` through the default capability, a compile-time
error is produced.

If `T |> has(Error)` is false:

```text
the error branch is not entered;
Error / default return capability is not required.
```

If `T |> has(Error)` is true:

```text
the branch containing return_owner..return(e Error) is entered;
this requires the current policy/capability environment to permit that return capability.
```

## 9. Local Error Binding and Active Propagation

Because `Error` is an ordinary lookupable symbol in the carrier predicate and
branch shape, an ordinary local binding may shadow an unqualified `Error`
lookup. This lets a `noerror` function turn an error into an ordinary return
value explicitly, instead of using the default return capability.

Conceptually:

```text
let Error = MyPureResultHandler
```

This is ordinary shadowing by a fresh symbol, not symbol forwarding: `Error`
becomes a distinct symbol in a distinct place that happens to carry the same
value.

An equivalent local binding makes the Error carrier branch construct an
ordinary value, for example:

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
forge the ambient error-return capability. A custom handler that wants to be
legal in a `noerror` environment must therefore avoid calling the return
capability.

This resembles Rust-style `Result` propagation, but it is not a copy of Rust:

```text
By rebinding Error to a pure handler, a noerror function can convert an
Error-shaped carrier into an ordinary sum/value return. This resembles explicit
Result-style propagation, but in this language it arises from symbol binding and
policy-controlled branch availability rather than from a built-in Result type.
```

Three stable cases summarize the model:

```text
1. Ordinary function, return value type has Error:
   default Error carrier branch is visible
   automatic propagation is inserted and valid

2. noerror function, return value type has Error, no custom Error handler:
   default return capability is not visible/executable
   entering the error branch is a compile-time error

3. noerror function, return value type has Error, local Error binding points to pure handler:
   Error is handled as ordinary data
   function still satisfies noerror
```

## 10. Policy Dimensions Involved

This design depends on future policy checking; it does not assume a complete
error-policy checker exists. The relevant dimensions are:

- the current P1-projected value view and namespace visibility determine which
  `Error` branch predicate or handler objects are available;
- receiver/parameter policy pairs and stage constraints determine whether a
  handler call is admissible; P2 is its evaluation horizon;
- the produced result remains layered `Object = ⟨Val1?, P, Val2⟩` material;
  there is no independent arbitrary complete return-policy `P3` or scalar
  replacement for its independent internal value/type Policy observations.
  Pout.stage=P1.stage; omitted mode inherits and an explicit mode overlay
  remains a constraint. Public Policy pair syntax is retired; direct source/type
  projections retain the same edge only before a new binding boundary;
- `noerror` changes the current capability / policy environment so that the
  default return capability is excluded or not executable.

These are future design statements, not a description of an implemented policy
checker. Canonical symbol-flow policy is defined in
`../symbol-world/symbol-policy-and-compile-flow-projection.md`.

## 11. Relation to Meta Object Invocation

Automatic return normalization should ultimately reuse the formal meta object
invocation model rather than becoming a return-specific compiler oracle. The
steps:

```text
T |> has(Error)
Error carrier branch availability
optional handler invocation
branch guarding
```

should all be expressed through the unified policy-aware lookup and invocation
mechanism.

This document does not define the full meta object invocation model; it records
the dependency:

```text
return normalization depends on:
  ordinary returned-instance observation
  first-order TypeValueId projection material
  T |> has(Error) predicate
  guarded branch evaluation
  policy-aware Error lookup
  callable body-entry checking for any factored handler
```

## 12. Non-Goals

```text
No parser syntax change.
No current normalizer behavior change.
Implementation scope: this document defines lowering obligations and does not
change the Rust implementation.
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

## 13. Relationship to Other Documents

The canonical owners below constrain this future return-normalization sketch.

- `meta-object-invocation-and-policy-reduction.md` — the unified policy-aware
  lookup and invocation engine that `Error` branch lookup and branch guarding
  should reuse.
- `pattern-normalization-and-first-order-overload.md` — provides the
  first-order type information that `T |> has(Error)` is evaluated over.
- `type-values-places-and-borrow-views.md` — defines `TypeValueId`, over
  which `T |> has(Error)` is computed as projection material.
- `mechanical-argument-passing-and-move-fixed-point.md` — the argument-slot
  counterpart of this return-slot normalization; both are mechanical source-level
  lowering actions.
- `../symbol-world/symbol-policy-and-compile-flow-projection.md` — canonical
  `Pv:Pp`, contextual P1/P2 elaboration, and stage views that gate Error branch
  lookup and execution.
- `../policy-capability/policy-visibility-symbols.md` — mapping from current
  policy metadata to that final boundary and future orthogonal error policy.
- `return-value-extraction-and-implicit-decomposition.md` — defines the
  extraction-view `?` operator as a one-step declared top-pattern-layer
  transition. Although Error carrier handling also uses pattern-shaped branches,
  it is not written as `r?`. The `?` operator is reserved for the declared
  extraction-view transition described there. Return normalization uses the
  explicit guarded branch form shown in this document.
- `control-flow-local meta evaluation substrate` (see
  [semantic evaluation](../meta-invocation/evaluation-residual-and-optimization.md)) — the guarded
  `T |> has(Error)` branch relies on the control-flow-local meta evaluation
  substrate: the false branch has no Error lookup or return-capability
  obligation, and the true branch alone checks the Error carrier branch and
  `return_owner..return(e Error)` capability.


Terminal payload delivery still receives the selected ReturnPattern/Pout demand
before the immediate root call's maxima. This future error-handling sketch
cannot insert an unconstrained semantic temporary before that delivery.
