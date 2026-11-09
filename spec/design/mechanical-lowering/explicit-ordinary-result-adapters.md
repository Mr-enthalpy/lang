# Explicit ordinary result adapters

Status: optional library composition constrained by the canonical result,
Pattern, call, lifecycle and effect owners. Source execution remains pending.
This note grants no compiler-inserted return normalization.

## 1. Ordinary data delivery

A normally returned Object remains ordinary data, including an Object with an
Error member or Error-shaped sum alternative. Its type/member spelling does not
execute throw, create Wait, acquire a return capability or change a caller's
effect interface. Ordinary functions have no ambient Error-return capability
or default type-triggered propagation. Error, Result and noerror may be ordinary
library names; they introduce no language Policy coordinate or implicit action.

The [result owner](../patterns-overload/return-value-extraction-and-implicit-decomposition.md)
and [return owner](../control-flow/targeted-return-and-d-reduction.md) control
delivery, UnitDiscard, tail completion and target inference. Terminal demand
precedes root-call maxima, with no hidden temporary or extra return.

## 2. Explicit adapter calls

```lang
let result = expression;
result |> adapt_result;
```

adapt_result is a schematic ordinary library callable requested explicitly by
the program. It may match an admitted Result/Error sum through R_Gamma and
construct another ordinary value, or explicitly perform a declared effect.
Pattern/Policy, require, unique selection, Pre/commit/Post, lifecycle and result
delivery remain ordinary. The explicit binding is a real boundary; a directly
written terminal adapter instead receives ordinary terminal demand.

A library may provide a wrapper with a fixed Function Pattern interface. Users
select that wrapper by ordinary calls. No compiler inserts it on every call or
infers it from Error membership. Shadowing affects only actual lookup occurrences
in the selected program, not an ambient error handler.

## 3. Values and actual control effects

A value adapter may map Error to an Err-like ordinary sum while control stays
normal. Another explicitly selected adapter may execute throw under the
[effect owner](../control-flow/effect-steps-and-continuation-handoff.md). That
action delivers Wait and a concrete continuation. Handling/resuming/forwarding
or dropping it uses ordinary Move/repair and cleanup. Returning the same
Error-shaped data performs none of these actions.

An explicitly selected adapter using a targeted return capability must prove
the actual activation, reachable segment and lifecycle Pre. Ordinary closure
barriers remain; possession grants no crossing authority. Only selected forced
actual matching failure uses PatternMiss. Ordinary branch mismatch does not.

## 4. Contracts, stages and ownership

Effect-restricted wrappers prove their interfaces through ordinary handlers and
Function Pattern compatibility, not a noerror flag that changes callees.
Compile/runtime availability and reify's ClosedReturn/ClosedInterface remain
independent. Source-erased Function Pattern values take former captures as
explicit inputs; adapters restore no implicit source lookup.

Owned values and reference handles retain origin, generation, access, MoveEffect
and cleanup obligations. Failed Pre publishes no success, selected failure never
reopens, and prior effects remain. O may inline an explicit adapter only under
ordinary equivalence; it cannot insert previously absent error propagation.

## 5. Paired acceptance and consumer status

| ID | Positive obligation | Negative boundary | Execution |
|---|---|---|---|
| RA01 | Error-shaped data returns normally under its result Pattern | No implicit Wait, adapter lookup or early return | Pending delivery |
| RA02 | Explicit value adapter returns ordinary data | Error membership/spelling alone produces no effect | Pending library/call |
| RA03 | Explicit effectful adapter uses declared throw/handler protocol | No ambient propagation, erased continuation or unwind | Pending effect/Drop |
| RA04 | Wrapper proves a fixed interface | noerror name/shadowing supplies no missing permission | Pending Function Pattern |
| RA05 | Tail analysis observes actual explicit adapter/cleanup work | No fictitious automatic error action changes tailness | Pending common-K CPS/materializer |

These are normative library-boundary targets, not executed adapter tests. One E
and its complete original-K CPS account own actual control and caller suffixes.
There is no default Error branch, automatic source rewrite or separate return
error evaluator scheduled for later implementation.
