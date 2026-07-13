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

The future callable form has two policy positions:

```text
P1 let F = (...): P2 -> let r => { ... }
```

```text
P1 ::= compile
     | runtime
     | (compile | runtime)

P2 ::= compile
     | meta
     | runtime
```

Their meanings are different:

```text
P1:
  external lookup policy of the callable object

P2:
  execution policy of this callable entry
  plus an exact total-policy requirement on every argument symbol
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

for every argument a:
  total_policy(a) = external(P2)
```

`P1` is not an argument policy. `P2` is not the callable symbol's lookup
range. The union `runtime | compile` is currently meaningful only at `P1`;
`P2 = runtime | compile` is reserved for a future two-stage binding and
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
back into one result annotation and contradict the layered model. The result
symbol's external lookup policy instead inherits `P1`:

```text
lookup_policy(result_symbol) = P1
```

This inheritance does not assign `P1` to every value leaf or layer object.
Future policy closure or transformation must use an explicit mechanism; it
must not be hidden in a revived return-policy slot.

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

Gamma; P2 |- call(symbol, arguments) => result
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

These fields establish storage and filtering substrate only. In particular:

```text
current symbol policy metadata
  != complete P1 semantics

current body_entry_policy
  != complete P2 qualification semantics

current return_object_policy
  != a final P3 language position
```

The current return-object field may continue to transport provisional result
metadata until result symbols and their layered facets are represented. It
must not be cited as evidence that a third policy plane remains normative.

Generated field callables illustrate only the current lookup/entry separation:
their symbols may be visible to an early resolver while their bodies remain
runtime-entered. Their current return-object metadata is an implementation
fact, not the final source model.

Not yet implemented:

- `Val1 x Pattern x Val2` policy accounting;
- exact argument `total_policy` checks for `P2`;
- complete `P1` lookup projection;
- compile-flow projection;
- runtime-entry compile companions;
- `must_select_if_qualified`;
- automatic inferred require;
- shared require/body compile-evaluation nodes;
- `seal` and any Pattern-policy extension.

## 8. Overload and Invocation Boundaries

Overload resolution first uses `P1` to determine visibility, then forms a
qualified candidate set using structural applicability and `P2`. The
`must_select_if_qualified` postcondition is evaluated only after the ordinary
linear filters. The canonical rules are in:

- `../symbol-world/symbol-policy-and-compile-flow-projection.md`
- `../patterns-overload/overload-resolution-design.md`
- `../meta-invocation/meta-object-invocation-and-policy-reduction.md`

Compile companions are first-class derived entries in that same pipeline. They
are not hidden fallback calls and cannot be replaced by an unrelated,
higher-priority compile overload.

## 9. Guardrails

- Do not introduce `policy`, `meta`, `compile`, `runtime`, or `seal` as lexer
  keywords merely to implement this future design.
- Do not treat a compile-computed type value as an installed type symbol.
- Do not grant compile code symbol-construction capability.
- Do not infer callable entry permission from symbol visibility.
- Do not reintroduce an independent `P3` return-policy position.
- Do not include `Val2` in the current symbol total-policy calculation.
- Do not describe compile-flow projection as policy-constraint solving or
  eager overload evaluation.
- Do not claim the current Rust metadata implements this final policy model.
