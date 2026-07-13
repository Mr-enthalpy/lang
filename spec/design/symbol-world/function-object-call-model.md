# Function Object Call Model

Design consolidation note. Not a user-visible syntax document.

The canonical symbol-first/facet boundary is
`symbol-first-meta-construction-and-pattern-injection.md`. This document owns
the type-associated `()` call mechanism; it does not redefine how a name first
resolves to a symbol and heterogeneous value facet.
Layered policy, `P1` / `P2`, compile companions, and must-select consistency
are canonical in `symbol-policy-and-compile-flow-projection.md`.

## 1. Basic thesis

A function is a value.

```text
let f = (self) => {};
```

`f` is a value of an anonymous function-object type `F`. `F` is usually not nameable in source syntax (obtainable as `f |> type`).

The converse is not true: a value entry need not be a function. A symbol's
value facet may contain ordinary uncallable values and multiple heterogeneous
values of unrelated types. Callability is tested only in a call position by
resolving each value's type-associated `()` entry.

Under the associated namespace of `F` there is a call entry `() :: F`. This `()` is the call method of the function object.

Function call is not primitive textual application. It is resolved through the callable object's type-associated namespace.

## 2. Pipeline call form

```text
Product |> Expr
```

`Expr` is the would-be callable object. The call process:

1. Evaluate/resolve `Expr` as a value object
2. Obtain `type(Expr)`
3. Inspect the associated namespace of `type(Expr)`
4. Find the call entry `()`
5. Invoke that entry with implicit self + explicit Product

The target expression is not itself the call method. The target is a value whose type-associated namespace contains the call method.

When `Expr` is a name/path, resolution first produces a symbol and projects its
heterogeneous value facet. Steps 2–5 run independently for each value entry;
entries without an applicable `()` call entry are discarded.

## 3. `()` is not an operator

`()` is not an operator. An operator is a callable value with special binding and parsing behavior. Since values are not namespace/type parents, an operator cannot serve as an intermediate navigation node.

`()` is a special type/namespace call entry. It is not itself a callable operator value. It cannot become the parent of another call lookup. It can only appear as a navigation leaf.

## 4. Direct function object call method

For `let f = (self) => {};`, the generated anonymous function-object type `F` has the call method under `F` itself: `() :: F`. Not under `ref::F`, `share::F`, or `move::F`.

A directly defined function object call owns its function object internally. Ownership is not written by the user — it is part of the generated function-object call method.

## 5. User-defined callable objects

```text
Product |> object
  → object value
  → type T = object |> type
  → associated namespace search for `()`
```

User-defined call entries are commonly installed under borrowed associated namespaces (e.g. `() :: ref::T`). The user writes `ref` explicitly; the expression's type becomes `ref::T`, and lookup follows from there. The language does not automatically jump from `T` to `ref::T`.

Direct function objects are not merely sugar for user-defined `ref::T` callables. They have their call method directly under their anonymous function-object type.

## 6. Implicit `self`

Every function has an implicit first parameter position: `self`. This is a positional position, not a user-visible name. Applies to all functions, including meta functions.

The call entry `()` always receives the callable object as implicit `self`. The user cannot manually pass this `self`.

The source product contains only the explicit user arguments. `ProductObject`, `ArgProductShape`, and `RawArgShape` represent only the explicit product supplied by the user. They do not contain the implicit `self`.

The implicit `self` belongs to the callable-entry invocation frame, not to the source product.

### 6.1 Internal Self frame and local pattern construction

An ordinary function object owns an internal symbol/pattern space for local
construction. Its diagnostic path may be sketched as:

```text
topname::__inner_space::Self
```

The exact eventual Self-frame identity is still future design. The invariant is
that a local `struct` evaluated by an ordinary or `compile` callable uses this
function-object internal scope as its ambient pattern owner. A `compile`
invocation does not manufacture a meta-style `(canonical_arguments name)`
symbol layer.

A canonical `meta` invocation is different: symbol construction is anchored by
its own `MetaInstanceScope(callee_symbol, canonical_arguments)`. Meta callables
still use the ordinary implicit-self invocation mechanics described above, but
their returned type construction is rooted in the meta-instance symbol scope,
not in the ordinary function-object Self frame.

## 7. ZST function objects

A function object with no captures is normally zero-sized. ZST values are not move-killed, so a zero-sized function object can naturally be called multiple times. Reusability follows from the general ZST movement rule.

If a function object captures state, it may be non-ZST and follows ordinary value-passing and ownership rules.

## 8. Call lookup pipeline

```text
Product |> Expr

1. Shape explicit Product: ProductObject → ArgProductShape → RawArgShape*
2. Resolve a name/path to Symbol and project its heterogeneous value facet
3. For each value entry, obtain its type / TypeValueId
4. Find call entry: type(value).associated_namespace → lookup `()`
5. Discard non-callable/non-applicable entries
   and include any visible first-class derived entries
6. Determine self mode: () :: F / () :: ref::T / () :: share::T
7. Build invocation frame: implicit self + explicit shaped product args
8. Filter callable objects by P1; form qualified set Q by structure, P2,
   and exact argument-symbol total policy
9. Apply ordinary linear filters and the must-select final consistency check
10. Enter the unique selected invocation or defer according to demand
```

A derived compile companion is prepared as an ordinary identified call entry
with an origin; it is not a lookup-failure fallback. If it enters `Q`, its
must-select property requires it to be the final unique candidate.

## 9. Relation to v0.8 substrate

For v0.8 construction substrate, `NormalizedCallSite.target` is not itself the `()` method — it is the callable object expression. The full pipeline is:

```text
target expression → target value → target type →
  type-associated namespace → `()` call entry
```

The current implementation uses a documented shortcut (v0.8): the resolved target `SymbolObject` is treated as the callable entry directly, via `ResolvedCallTarget { temporary_direct_callable_shortcut: true }`. This shortcut will be replaced when function-object types and associated call-entry insertion are implemented.

## 10. Invariants

- Function object is a value.
- Every function object has a type.
- A directly defined function object has an anonymous function-object type.
- The call entry `()` for a directly defined function object lives under that anonymous type.
- User-defined callable objects may define `()` under `ref::T` / `share::T` / other associated namespaces.
- Implicit `self` is always passed by the call mechanism.
- Implicit `self` is positional, not a user-visible name.
- Implicit `self` is not part of `ProductObject` / `ArgProductShape`.
- The user cannot manually pass implicit `self`.
- `()` is not an operator.
- Operator values cannot be namespace/navigation parents.
- `()` is a special type/namespace call entry and can only be a navigation leaf.
- ZST function objects are reusable because ZST values are not move-killed.
- Non-ZST function objects obey ordinary ownership and passing rules.
- Meta functions follow the same function-object and implicit-self model.
- Ordinary/compile local pattern construction uses the function-object internal
  Self frame; compile does not create a MetaInstanceScope.
- Meta symbol construction is anchored by the canonical MetaInstanceScope even
  though invocation still passes implicit self.
