# Policy Visibility and Capability Mapping

Status: implementation-mapping companion.

Canonical policy semantics are defined in
`../symbol-world/symbol-policy-and-compile-flow-projection.md`. This document
does not independently redefine P1, P2, seal, compile projection, companions,
overload preference, or automatic require.

## 1. Canonical Shape

Internal policy is:

```text
Π = Pv:Pp
```

The dimensions are orthogonal:

```text
stage
value mutability
namespace visibility
value presence
```

Do not model them as one untyped flag bag.

Ordinary judgments use:

```text
Γ ⊢ e : (τ, Pv:Pp)
```

`@` remains reserved for lifetime values and predicates.

## 2. P1 Mapping

P1 is the optional prefix on any binding:

```text
[P1] let x = expr
```

Omission infers the complete RHS result.

A single P1 `Q` is a value-dominant projection. It selects values visible under
`Q` and retains their associated pattern/type components. It is not elaborated
as `Q:Q`.

A pair P1 `Qv:Qp` filters both components.

The projection fails only when no requested result slice remains. In
particular:

```lang
runtime let x = runtime_expr;
```

is a valid general binding. A compile-projection rule may reject a runtime
source in that one source position; it must not reject runtime destination
bindings globally.

## 3. P2 Mapping

P2 describes the call/expression result pair and drives function-object stage
availability:

```text
P2 -> function-object P1
```

Explicit P2 `Pv:Pp` obeys:

```text
runtime not in Stage(Pp)
Static(Pv) is empty or Static(Pv) = Stage(Pp)
```

Single P2 normalization is:

```text
N2(P) = P:(P-runtime), when P-runtime is non-empty
N2(runtime) = runtime:lastStatic
```

With seal:

```text
N2(runtime) = runtime:seal
```

There is no P3 and no scalar result-symbol policy.

## 4. Function-Object Mapping

For `P2 = P2v:P2p`:

```text
Stage(P1p) = Stage(P2p)
Stage(P1v) = Stage(P2v) union Stage(P2p)
```

Only stages are lifted. Function-object const/mut and namespace visibility
come from its declaration position, not from returned values.

An explicit P1 prefix projects this derived object view; it cannot create a
stage absent from P2.

## 5. Visibility Domains

```text
Vis(meta)    = { open }
Vis(seal)    = { seal, postSealCompile }
Vis(compile) = { open, seal, postSealCompile }
```

`meta` and `compile` remain different capabilities even if an implementation
shares evaluator machinery.

Seal is visibility exclusion, not symbol deletion and not reflection
permission. Compiler-known privileged seal meta-functions may scan a frozen
pre-seal world `Wpre`; ordinary seal symbols cannot.

## 6. Namespace and Mutability

Namespace visibility is legal in namespace-scoped P1 positions, normalizes to
one shared attribute across `Pv:Pp`, and is not inherited from P2.

```text
public:compile == compile:public
public:private => error
mut+export => error
```

`const` and `mut` occur only in `Pv`. Overload comparison uses per-position
preference and product partial order; no global conversion score exists.

## 7. Current Rust Mapping

Current structured substrate:

- `PolicySpecAst` / `NormPolicySpec` preserve pair syntax;
- `PolicyPair` separates stages, mutability, namespace visibility, and value
  presence;
- P1/P2 elaboration helpers implement contextual single-policy behavior;
- function-object stage derivation, bounded member views, a pre-seal snapshot,
  and const/mut product-order helpers have direct tests.

Current compatibility substrate:

- `PolicyFlag` / `PolicySet` still flatten stage and legacy export information;
- `PolicyEnv` exposes flat meta/compile/seal/post-seal/runtime resolver views;
  these are visibility filters, not pair projection or execution permission;
- `body_entry_policy` and `return_object_policy` remain transitional transport;
- initializer binding projects a non-empty stage slice but does not yet carry
  complete `Pv:Pp` entries through every namespace graph operation.

Therefore:

```text
current PolicySet != canonical PolicyPair
current return_object_policy != P3
current resolver filtering != complete Pv:Pp facet projection
```

## 8. Guardrails

- Do not add policy words as lexer keywords; they remain ordinary names parsed
  only in strong policy positions.
- Do not normalize single P1 `Q` to `Q:Q`.
- Do not normalize single P2 runtime to `runtime:compile`; use
  `runtime:seal` while seal is `lastStatic`.
- Do not derive P2 from P1.
- Do not copy const/mut or namespace visibility from P2 to the function object.
- Do not make runtime illegal on a general `let` binder.
- Do not grant every seal object global scan capability.
- Do not reuse `@` for ordinary policy notation.
- Do not describe flat implementation metadata as final policy semantics.
