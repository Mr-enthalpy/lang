# Policy Visibility and Capability Mapping

**Status: Mixed implementation note and future-design satellite.** The final
symbol-flow policy model is canonical in
`../symbol-world/symbol-policy-and-compile-flow-projection.md`. This document
records how that model relates to the policy metadata already present in
`lang_build` and to policy dimensions that remain intentionally orthogonal.

This document does not define a third return-policy position, compile-flow
projection, compile companions, or automatic require. Those rules have one
normative owner: the canonical symbol-policy document above.

## 1. Canonical Policy Shape

### 1.1 General binding policy

The `P` in a general policy-bearing binding is the destination policy, not a
callable-object `P1` and not the source policy of a particular projection rule:

```text
Gamma |- e : tau @ P_e
P_e ⊑ P
----------------------------
Gamma |- P let x = e
```

There is no general `P ≠ runtime` side condition. In particular,
`runtime let x = runtime_value;` is a legal binding when the RHS has runtime
policy. A compile-determined projection may separately require its local source
policy `P_src` (sometimes written `P₁`) to be non-runtime. That premise applies
only to that projection operand; it must not be moved to the enclosing binder
or implemented as `reject if binding_policy == runtime`.

### 1.2 Callable-object and entry policies

The future callable form has two policy positions:

```text
P1 let F = (...): P2 -> let r => { ... }
```

```text
P1 ::= compile
     | (compile | runtime)

P2 ::= compile
     | meta
     | runtime
```

Their meanings are different:

```text
P1:
  external lookup policy of one callable/Val2 object after Symbol resolution

P2:
  execution policy of this callable entry
  plus an exact total-policy requirement on every invocation-frame slot
```

The external stage of an entry is:

```text
external(compile) = compile
external(meta)    = compile
external(runtime) = runtime
```

A declaration is well formed only when:

```text
external(P2) subset-of P1
```

A call is policy-applicable only when:

```text
current_lookup_stage in P1

and

for every invocation-frame slot a:
  total_policy(a) = external(P2)
```

The frame includes slot 0, the implicit `self` view of the selected `Val2`
function object, followed by all explicit source arguments. Declaration
well-formedness establishes that the object is visible at `external(P2)`;
invocation records the same stage requirement uniformly for `self` and the
explicit arguments.

The containing path is resolved to `Symbol` before this check; its
heterogeneous value facet is then projected and each object is filtered by its
own `P1`. One symbol may therefore carry objects with different `P1` sets.
`P1` is not a base symbol-resolution policy or an argument policy. Every
callable `Val2` object is compile-visible so its Pattern, parameter and result
patterns, constraints, and compile projection can participate in symbol flow;
runtime-only execution is represented by `P2 = runtime` on an object whose
`P1 = compile | runtime`. `P2` is not the callable object's lookup range. The
set `compile | runtime` is currently meaningful only at `P1`;
composite-stage `P2` is reserved for a future two-stage binding and
evaluation model.

## 2. There Is No Final P3

There is no independent final return-policy position:

```text
P1 let F = (...): P2 -> let r => { ... }
```

not:

```text
P1 let F = (...): P2 -> P3 let r => { ... }
```

A returned symbol remains layered symbol material:

```text
Val1 x Pattern x Val2
```

Each layer retains its own policy. A single `P3` would flatten those policies
back into one result annotation and contradict the layered model. Apply the
selected callable object's `P1` to result material layer by layer:

```text
result_projection_by_P1(Result, P1)

Result.Pattern:
  retain stages admitted by P1 and supported by Pattern
  currently policy(Pattern) = compile, so only its compile projection exists

Result.Val1 when P1 = compile:
  retain compile-policy leaves

Result.Val1 when P1 = compile | runtime:
  retain compile- and runtime-policy leaves

Result.Val2:
  each exposed object retains its own P1
```

The result symbol does not receive one scalar lookup policy, and returned
`Val2` objects do not inherit the caller object's `P1`. Future policy closure
or transformation must use an explicit mechanism; it must not be hidden in a
revived return-policy slot.

## 3. Symbol Policy Is Layered

The canonical flow model is:

```text
Symbol = Val1 x Pattern x Val2
```

Until `seal` is introduced:

```text
policy(Pattern) = compile
compile < runtime

total_policy(Symbol)
  = policy(Val1) join policy(Pattern)
```

`Val2` is deliberately excluded from this total-policy calculation. It is the
object layer used for later lookup and invocation, commonly containing
callable objects.

A pure type is the degenerate case with no `Val1` leaves and therefore has
compile total policy. An ordinary value is the degenerate case with no `Val2`
object. These are not two disconnected semantic worlds.

The policy expression attached to an object answers a lookup question. The
entry policy answers an execution and argument-qualification question:

```text
Gamma; lookup_stage |- path => symbol

Gamma; P2 |- call(selected_Val2_object, InvocationFrame) => result
```

Visibility never implies entry permission.

## 4. Compile and Meta

`compile` and `meta` have different internal capabilities:

```text
compile:
  compute ordinary static values and PatternValue

meta:
  construct SymbolConstructionValue inside a MetaConstructionUnit
```

They share an external stage:

```text
external(compile) = compile
external(meta)    = compile
```

Consequently compile-flow projection keeps both compile and meta invocation
edges as early flow, while normal compile evaluation still checks which
internal capability the selected entry requires. Grouping them in an external
projection does not grant compile code meta construction capability.

Evaluation demand remains a separate axis:

```text
execution capability:
  compile | meta | runtime

evaluation demand:
  partial | strict

result rank:
  PatternValue | SymbolConstructionValue | runtime value
```

`partial` and `strict` do not replace `P1` or `P2`, and the compile/meta split
does not replace partial/strict reduction.

## 5. Orthogonal Policy Dimensions

The symbol-flow stage policy above must not be overloaded with unrelated
capabilities. Future access and effect dimensions may include:

```text
public / package / private / friend
const / mut
error / panic / noerror
resource capabilities
```

These dimensions may have their own partial orders or composition rules. They
do not change the present two-stage total-policy relation merely because all
of them use the general word "policy".

In particular, `compile | runtime` at `P1` is a lookup-stage set. It is not a
claim that a single callable entry executes simultaneously under both stages.

## 6. Seal Boundary

`seal` is not yet part of the active symbol policy calculation. Therefore the
current invariant remains:

```text
policy(Pattern) = compile
```

A future `seal` design may allow a frozen-pattern or registration stage, but it
must specify explicitly:

- whether and how Pattern policy can change;
- which frozen graph material is readable;
- whether any local construction is permitted;
- how seal projection composes with compile and runtime;
- how the extension affects `total_policy(Symbol)`.

No present document may infer these answers from the transitional
`PolicyFlag::Meta` / `PolicyFlag::Runtime` representation.

## 7. Current Implementation Substrate

The current Rust implementation predates the final `P1` / `P2` model. It
contains useful but transitional metadata:

- `PolicyFlag`, `PolicySet`, and `PolicyEnv`;
- symbol policy metadata used by resolver filtering;
- callable `body_entry_policy` fields;
- `return_object_policy` fields on callable and generated-field payloads;
- `PolicyEnv::Meta` / `PolicyEnv::Runtime` lookup filtering;
- policy transport through namespace deltas and selected early-meta paths.
- an explicit initializer-policy verifier that currently treats every written
  policy flag as independently required rather than implementing `P_e ⊑ P`.

These fields establish storage and filtering substrate only. In particular:

```text
current symbol policy metadata
  != complete P1 semantics

current body_entry_policy
  != complete P2 qualification semantics

current return_object_policy
  != a final P3 language position

current explicit-policy flag verification
  != the final general policy-binding relation
```

The current return-object field may continue to transport provisional result
metadata until result symbols and their layered facets are represented. It
must not be cited as evidence that a third policy plane remains normative.

Generated field callables illustrate only the current lookup/entry separation:
their symbols may be visible to an early resolver while their bodies remain
runtime-entered. Their current return-object metadata is an implementation
fact, not the final source model.

The current explicit initializer verifier can reject a runtime residual under a
written combined policy because it expects every written flag to be evidenced.
That behavior is transitional. It must not be generalized into a language rule
that forbids runtime destination bindings or requires exact policy equality.

Not yet implemented:

- `Val1 x Pattern x Val2` policy accounting;
- exact invocation-frame `total_policy` checks for `P2`, including `self`;
- the final general `P_e ⊑ P` binding check and inference model;
- complete `P1` lookup projection;
- compile-flow projection;
- complete derived `Val2` compile-companion objects;
- `must_select_if_qualified`;
- automatic inferred require;
- shared require/body compile-evaluation nodes;
- `seal` and any Pattern-policy extension.

## 8. Overload and Invocation Boundaries

After path resolution has produced `Symbol` and enumerated its heterogeneous
objects, overload resolution uses each object's `P1`, resolves its type-
associated `()`, and applies every hard shape, Pattern, `P2`, frame-policy,
result-expectation, concept, and require check to form the fully admissible set
`A`. Pure preference filters run only after `A` exists. A
`must_select_if_qualified` strategy activates from `A` and is checked against
the final preference survivor set. The canonical rules are in:

- `../symbol-world/symbol-policy-and-compile-flow-projection.md`
- `../patterns-overload/overload-resolution-design.md`
- `../meta-invocation/meta-object-invocation-and-policy-reduction.md`

Compile companions are complete derived `Val2` function objects in that same
pipeline. Each has its own object identity, function-object type, associated
compile `()`, origin runtime object, and overload strategy. They are not hidden
fallback calls and cannot be replaced by an unrelated, higher-priority compile
overload.

Lifetime policy is outside this type/compile pipeline. Its negative boundary is
canonical in
[`../lifetime/lifetime-policy-and-overload-boundary.md`](../lifetime/lifetime-policy-and-overload-boundary.md).

## 9. Guardrails

- Do not introduce `policy`, `meta`, `compile`, `runtime`, or `seal` as lexer
  keywords merely to implement this future design.
- Do not treat a compile-computed type value as an installed type symbol.
- Do not grant compile code symbol-construction capability.
- Do not infer callable entry permission from symbol visibility.
- Do not turn a projection-local `P_src ≠ runtime` premise into a general
  `P ≠ runtime` binder restriction.
- Do not reject a general binding merely because its destination policy is
  runtime.
- Do not reintroduce an independent `P3` return-policy position.
- Do not include `Val2` in the current symbol total-policy calculation.
- Do not describe compile-flow projection as policy-constraint solving or
  eager overload evaluation.
- Do not claim the current Rust metadata implements this final policy model.
