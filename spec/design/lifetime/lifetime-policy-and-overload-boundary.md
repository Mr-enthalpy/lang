# Lifetime Policy and Overload Boundary

**Status: Future-design boundary; no lifetime algorithm is frozen here.**

This note prevents lifetime policy from being folded into type checking,
compile-policy candidate qualification, or extraction specificity. It does not
specify a complete lifetime system.

## 1. Stage Boundary

The normative negative boundary is:

```text
lifetime check != type check

lifetime algebra value does not depend on type-value identity

lifetime predicate policy != compile policy
```

Lifetime policy runs after sealing and before runtime execution:

```text
type/compile lookup, overload selection, and instantiation
  -> seal
  -> lifetime-policy checking and possible refinement
  -> runtime
```

Its input is fully first-order IR. All type/compile overload selection and
instantiation has already completed. A future lifetime-refinement stage may see
multiple first-order callable objects with identical ABI shape, reject objects
whose lifetime precondition is not satisfied, and, if several remain, select a
more refined object. None of those checks belongs to the fully admissible
type/compile candidate set.

Lifetime algebra and predicates may reuse the compile evaluator's general
calculation substrate. Reuse does not make their policy `compile`: lifetime
policy remains isolated from compile, seal, and runtime by the stage boundary
above.

## 2. Lexical Boundary

`@` is reserved for lifetime-policy value and predicate structure. Known
examples are:

```text
val@
val@.region
val@.origin
```

Conceptually, `val@` obtains the lifetime-algebra value associated with `val`;
the projections select region- or origin-related material. This boundary does
not freeze their complete algebra or evaluation rules.

In particular, `@` is not a general annotation prefix. Overload strategies,
compile-companion association, and other semantic metadata must not appropriate
this spelling.

## 3. Deliberately Unspecified

This PR does not define:

- the complete region or origin algebra;
- the algorithm for checking lifetime preconditions;
- lifetime specificity or refinement ordering;
- Horae logic;
- lifetime cache identity or diagnostic representation;
- public syntax beyond the existing `@` lexical boundary;
- a lifetime-driven type/compile overload filter.

These subjects require a later design whose inputs are already first-order and
whose rules cannot change the completed type/compile overload result.

## 4. Related Canonical Documents

- Type/compile overload admissibility and preference:
  [`../patterns-overload/overload-resolution-design.md`](../patterns-overload/overload-resolution-design.md)
- `Pv:Pp` symbol policy, seal visibility, and compile projection:
  [`../symbol-world/symbol-policy-and-compile-flow-projection.md`](../symbol-world/symbol-policy-and-compile-flow-projection.md)
- Current policy metadata mapping:
  [`../policy-capability/policy-visibility-symbols.md`](../policy-capability/policy-visibility-symbols.md)

No Rust lifetime checker, refinement pass, or evaluator is implemented by this
document.
