# Policy as Visibility Symbols and Capability Strategy

**Status: Non-normative future design. v0.6 reserved metadata slots and
architectural placeholders. v0.7 adds `PolicyFlag` / `PolicySet` / `PolicyEnv`
types, assigns policy flags to core and user-declared symbols, and implements
`PolicyEnv::Meta` filtering in the resolver and early-meta expansion. Full
policy lattice, projection, conformance checking, and additional policy
environments remain future work.**

## 1. Scope

This is a future design note. It is not a current parser rule, not a current
normalizer rule, and not a v0.6–v0.8 implementation requirement.

The immediate, narrow objective for v0.6–v0.8 is:

- Reserve symbol policy metadata on `SymbolObject`.
- Reserve current-policy context slots on resolver / meta-function object /
  namespace node.
- Reserve policy fields on function signatures, return-object slots, and
  namespace graph injection / meta expansion results.
- Ensure `meta`, `compile`, `runtime`, `seal`, `const`, and `mut` are not
  introduced as lexer keywords, parser special forms, or hardcoded compiler
  branches.
- Prevent agents and implementations from inventing a full policy lattice ad-hoc.

Current implementation note: `lang_build` reserves `PolicyMetadata` and
`VisibilityMetadata` slots on `SymbolObject` and `NamespaceNode`, a
current-policy slot on `ResolverContext`, and function/body/return policy slots
on `MetaFunctionObject`. These slots are preserved through namespace deltas, but
no policy lattice, projection rule, conformance checker, or policy-driven
visibility filtering is implemented.

Non-goals for v0.6–v0.8:

- full policy lattice implementation
- full policy inference
- full projection / conformance checker
- full effect checking
- full const / mut checking
- full error / panic policy checking
- full seal-stage execution
- compile-time IR interpreter
- value-to-value compile-time execution

## 2. Core Principle

Policy is a visibility symbol and capability strategy.

Policy should not be understood first as a subtype in a type system, nor as an
ordinary annotation. It is a set of **traits** with partial-order, composition,
and mutual-orthogonality relations, used to constrain:

- which symbols the current context may import or resolve;
- under which contexts a given symbol is visible;
- in what policy environment a function body executes;
- how policy inference or projection checking applies to parameters, return
  values, and local bindings;
- whether a meta-function has the capability to construct types, inject symbols,
  or interpret AST;
- whether a compile-time computation is restricted to value semantics only;
- whether a seal-stage operation may only scan frozen symbols and register
  runtime metadata.

Where a partial order is noted, the "parent-child" or "⊑" relationship records
a trait partial order, **not** a traditional subtype in the type-system sense.

## 3. Basic Syntax Direction

```text
policy let ...
```

For example:

```text
runtime + meta let Vec: _: fn = ...
```

Here the leftmost `runtime + meta` annotates the symbol `Vec`'s own policy —
not the function body's entry policy, and not the return-value policy.

The function body's entry policy is specified through the function arrow or
function trait annotation:

```text
(T: type): meta -> meta + runtime let r: type => { ... }
```

- `meta` describes the function body execution environment.
- The environment begins at the parameter binding; policy is not computed only
  from the `{ ... }` body.
- The return slot `meta + runtime let r: type` annotates the returned object
  `r`'s own policy.

## 4. Context Policy and Import Rule

Every context carries a **current policy**.

A symbol that is to be imported or resolved in that context must have its own
policy be a child-policy of, or identical to, the current context policy:

```text
symbol.policy ⊑ context.policy
```

(where `⊑` is the policy partial order, not a traditional subtype.)

If the relation fails, a later phase would emit a policy visibility error. v0.6–
v0.8 only reserve the metadata slots; full checking is deferred.

## 5. Binding-Side Policy Rule

The policy written on the **left side of `let`** may only be the **parent** of,
or identical to, the policy of the overall expression on the right:

```text
expr.policy ⊑ binder.policy
```

(The final direction may adjust, but the design intent is preserved.)

A function's parameter position is equivalent to a binding position. Therefore
future function parameters require an explicit `let` to separate the policy
dimension from the parameter itself. This is a **future** syntax constraint, not
a current parser special case.

## 6. Orthogonality of compile and runtime

`compile` and `runtime` must not default to:

```text
compile ⊑ runtime
```

Instead they are orthogonal dimensions:

```text
compile ∧ runtime == ∅
```

A symbol intended to be visible in **both** phases must be written:

```text
compile + runtime
```

Traditional partial-order (parent-child-like) relationships belong primarily to
visibility-access-control policy dimensions such as `public ⊑ package ⊑ private`
or `friend`-like grants. `compile` / `runtime` / `meta` / `seal` are stage and
capability policies, not access-visibility policies, and do not necessarily form
a simple parent-child chain.

## 7. meta as Function Trait, Not let-side Policy

`meta` is a **function trait**. It describes what environment the function body
enters. It is not a general `let`-side policy annotation.

```text
runtime + meta let Vec: _: fn =
  (T: type): meta -> meta + runtime let r: type => { ... }
```

- `runtime + meta` — `Vec`'s own symbol policy.
- `meta` — function body execution policy.
- The parameter `T` is already in the `meta` environment from its binding point.
- The return slot `r` has policy `meta + runtime`.
- The entry policy and the exit policy are not required to be identical.
- When not explicit, inference or explicit projection checking determines legality
  (deferred to a later stage).

Do not interpret `meta` as source-level `import`, as a parser keyword, as a
namespace keyword, or as a runtime effect.

## 8. meta and compile

```text
meta  →  compile       (meta can project to compile)

compile  ↛  meta       (compile cannot project to meta)
```

Meanings:

- `meta` can project to `compile`.
- `compile` cannot project to `meta`.
- `compile` may serve as a function entry policy, but the allowed actions are
  narrower.
- `compile` only permits value-in and value-out.
- `compile` cannot execute `val -> type`, because producing a type object requires
  a meta-function, and meta-functions require the `meta` context.
- `compile` does not promise to create type objects, modify or construct symbol
  hierarchies, or interpret source structure.
- `compile` can ultimately execute on a stable IR.
- `meta` must be able to process AST / Normalized AST and the symbol graph, because
  many meta-actions occur before IR formation.

Why not merge compile and meta into a single trait:

- If everything were absorbed into `meta`, ordinary compile-time value computation
  would be forced to carry the full meta capability: type construction, symbol
  injection, AST interpretation. This would expand the checking surface, enlarge
  the trusted surface, and turn "just computing a value" into a meta-world entry.
- If everything were absorbed into `compile`, type construction, generic-class
  generation, symbol shielding, and returning stable type objects would either be
  inexpressible or would stretch `compile` into a de facto `meta`, distorting the
  name and the separation.

## 9. seal

`seal` represents a restricted stage after the compiler has created all symbols
and before runtime execution begins. It permits limited but targeted compile-time
capabilities to register metadata for the runtime.

Properties of `seal`:

- It may read the frozen version of the symbol graph.
- It may call specific built-in APIs whose policy requires `seal`.
- It must not create new symbols in public, navigable namespace layers.
- Symbols created in `seal` must be limited to the local lexical scope, or be
  symbols that the meta-function naturally returns after leaving that scope.
- Built-in APIs only scan the frozen version; they do not observe seal-stage
  additions.
- `seal → runtime` is a legal projection direction.
- `runtime → seal` is not legal.
- Projections from `compile` / `meta` to `seal` are subject to future constraint;
  they are not disallowed a priori but require explicit semantics.
- `compile + seal` and `meta + seal` as direct combination pairs should default
  to **error**, unless a specific, tightly-constrained function is explicitly
  defined to work across them.
- Functions like `sizeof` that might conceptually span both `meta` and `seal`
  should express this through ordinary policy trait inference and constraints,
  not through hardcoded special rules.

seal does not require a separate processing backend:

- `seal`'s characteristic work is symbol scanning, metadata organization, and
  dispatch — not type computation or general value computation. The task profile
  is sufficiently uniform not to justify a separate DSL.
- `seal` may locally open restricted `meta` / `compile` sub-contexts.
- `seal` may safely call `compile` functions, because `compile` is defined to
  register no new symbols, interpret no source structure, and construct no type
  objects.
- `seal` defaults to reusing the `meta` processing backend, but with policy-level
  restriction on external-namespace symbol creation.
- The additional built-in APIs exposed in `seal` are normal products of policy
  checking, not indicative of an independent processing DSL.

## 10. Type Return with runtime-only Policy

A type-typed return value may carry `runtime`-only policy. This is not a
contradiction. In the `seal` stage, after all compiler-visible symbols have been
created and before runtime execution, a seal-context function may use restricted
compile-time capabilities to register richer metadata for the runtime, and return
type metadata or type-related objects that are visible at runtime.

Distinguish explicitly:

- the function symbol's own policy;
- the function body entry policy;
- the return-value object's policy.

`runtime`-only return does not mean `runtime` can construct type objects. It only
means the returned object is visible at `runtime`.

## 11. Const / Mut Motivation and C++ Boundary

C++ discussions of deep `const` can be re-interpreted through this design. The
difficulty is not primarily a memory-model problem; it arises from two path
dependencies:

1. the implicit assumption that `compile` flows into `runtime`, which creates
   cross-boundary conflicts, and
2. the embedding of `const` inside the type identity, so that `constexpr`
   dynamic-allocation crossing into runtime cannot propagate `const`-ness past
   the shallow level.

This language tends to treat `const` / `mut` as policy dimensions extracted from
standard type identity, so that the "should `constexpr`/`const` enter the type"
path dependency is avoided.

A C++ note for context (not a critique): `const` in C++ does not affect the API
at parameter positions; as a return type it has documented effects on RVO/NRVO
whose behaviour is implementation-dependent, and the recommended practice
discourages return-by-const. This suggests that `const` is a policy-like
attribute weaker than a standard type identity, and that it need not be coerced
into the type system unconditionally.

C++ is chosen as the reference because it is the mainstream engineering language
that has pushed compile-time evaluation and metaprogramming furthest while
carrying the largest historical weight and the broadest genealogical influence.
Analysing it exposes the problem prototype; this design note is not written as a
reaction against C++.

## 12. Relationship to v0.6–v0.8 Roadmap

v0.6 (Build / Namespace Graph Bootstrap):
  Preserve policy metadata on symbols, contexts, and namespace graph nodes, but
  must not implement full policy checking.

v0.7 (Early Meta-Function Bootstrap):
  Expose meta-function policy fields and reserve body-entry-policy /
  return-object-policy, but must not implement full meta / compile / seal
  projection.
  Implemented: `PolicyFlag` / `PolicySet` / `PolicyEnv` types; `PolicyEnv::Meta`
  filtering in resolver and early-meta expansion; policy flags assigned to core
  symbols (`export+meta` for meta-functions, `export+meta+runtime` for built-in
  types and the core namespace symbol itself), source-contributed symbols (`runtime`
  for values, `meta+runtime` for type-annotated declarations), and struct-generated
  type objects (`meta+runtime`). Policy filtering is per-component — namespace
  intermediaries must carry traversal-appropriate flags; for v0.7 this is satisfied
  by the compiler-seeded `core` namespace symbol receiving `export+meta+runtime`.
  Policy assignment to other namespace categories (declared, physical, dependency)
  is deferred.

v0.8 (Type-to-Type Meta Construction Interpreter):
  Understand that meta body execution policy differs from function symbol policy
  and return-object policy. Implement only the minimum checks needed to avoid
  misrepresenting meta-functions as runtime functions.

Later stages will implement policy inference, projection checking, compile /
runtime / seal semantics, const / mut policy, effect policy, error / panic
policy, and resource capability policy.

## 13. Explicit Non-Goals / Guardrails

- Do not turn `policy`, `meta`, `compile`, `runtime`, `seal`, `const`, or `mut`
  into lexer keywords.
- Do not introduce source-level import/use/include/module syntax.
- Do not implement a full policy lattice in v0.6–v0.8.
- Do not make `compile` a subtype or sub-phase of `runtime`.
- Do not merge `compile` and `meta` into a single trait or stage.
- Do not allow `compile` to construct type objects or inject namespace symbols.
- Do not allow `seal` to create new public navigable symbols after the graph
  freeze.
- Do not make `const` / `mut` part of ordinary type identity in this design note.
- Do not implement deep `const` or C++ `constexpr` compatibility.
- Do not introduce a second compile-time DSL.
- Do not conflate function symbol policy, function body policy, and return object
  policy.
- Do not claim policy checking is complete.
