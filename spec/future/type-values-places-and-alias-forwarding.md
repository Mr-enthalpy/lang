# Type Values, Places, and Alias Forwarding

**Status: Non-normative future design. Not implemented as current type-value equality, alias forwarding, writable-place checking, injection-place checking, or type checker behavior.**

This document specifies the future semantic boundary between *type values*,
*symbol identity*, *writable places*, *alias forwarding*, and *namespace
injection targets*. It is a future design note. It is not current public
language behavior, not an implemented pass, and not a parser or normalizer rule.

The document is self-contained. It does not require the reader to assemble its
meaning from `type-associated-function-objects-and-access-trees.md`,
`early-meta-functions-and-namespace-graph.md`, or `entity-alias-design.md`. Those
documents are background or adjacent design only; the model here stands on its
own and is the canonical authority for the type-value / place / symbol / alias
distinction.

## 1. Purpose

The language must distinguish three things that look similar in source text but
are semantically different: the identity of a name, the identity of a writable
location, and the identity of a type value. Conflating them produces subtle
errors — for example, injecting a declaration into a built-in type because its
type value happens to equal a freshly bound symbol's type value.

The invariants this document protects:

```text
type-value equality must not collapse symbol-place identity
alias forwarding must not silently create writable places
namespace injection must target writable places, not values
```

This document does **not** define:

- a full type checker,
- a full alias implementation,
- access-tree construction,
- a full lifetime checker,
- package import / export,
- runtime lookup.

Two phrasings are explicitly rejected throughout. `let T: type = uint8` is
**not** fresh nominal type generation. Alias forwarding is **not** textual
substitution. And type-value equality is **not** writable-place equality.

## 2. Three semantic identities

Three distinct identities participate in this model:

```text
SymbolId
PlaceId
TypeValueId
```

- `SymbolId` is the identity of a symbol object in the name graph.
- `PlaceId` is the identity of a location that can be bound, updated, injected
  into, or opened for a namespace delta.
- `TypeValueId` is the identity of a canonical type value, used for type-value
  equality, first-order type comparison in pattern/overload matching, and
  rank/type expression evaluation.

These identities are independent. None implies another:

```text
SymbolId equality does not imply TypeValueId equality.
TypeValueId equality does not imply PlaceId equality.
Alias forwarding may relate symbols, values, and places, but does not erase the distinction.
```

A type expression cares about the *value*. A namespace injection target or a
declaration-extension site cares about the *place*. Alias forwarding relates a
*symbol* to a forwarding chain. The three concerns must not be folded into one
another.

## 3. Value judgment versus place judgment

The model uses two distinct judgments. One evaluates an expression to a value;
the other resolves a name to a writable place.

Value evaluation:

```text
Γ ⊢ x ⇓ v
```

means an expression / rank / type expression evaluates to value `v`.

Place resolution:

```text
Γ ⊢ x ⇐ p
```

means a declaration extension / namespace injection / assignment-like operation
resolves `x` to a writable place `p`.

These are not interchangeable. `let f::T = ...` uses the **place** judgment on
`T`, not the value judgment: it targets the place that `T` owns, not the value
that `T` evaluates to.

## 4. Ordinary type-value binding

The ordinary binding form:

```text
let T: type = uint8
```

means:

```text
symbol(T) = fresh symbol
place(T) = fresh writable place at current lexical level
type_value(T) = type_value(uint8)
```

This must be read precisely:

```text
T is not a fresh nominal type.
T is not a symbol alias.
T has fresh place identity.
T may evaluate to an existing type value.
```

`T` is a new symbol with its own fresh, current-level writable place. Its *type
value* equals that of `uint8`, but its *place* is its own. Binding to an existing
type value does not generate a new type, and it does not forward to `uint8`'s
symbol or place.

Consequently, injection through `T`:

```text
let f::T = ...
```

executes:

```text
place(T) += { f ↦ ... }
```

and not:

```text
place(uint8) += { f ↦ ... }
```

Injection is closer to `+=` on a place than to pure expression evaluation. The
right-hand use of a name is a value; the injection target is a place being
extended.

## 5. Alias forwarding

The alias form is different from ordinary type-value binding:

```text
let T === uint8
```

means symbol forwarding, not a fresh binding:

```text
alias(T) = uint8
value(T) = value(final_target(uint8))
place(T) = place(final_target(uint8))
```

Crucially, the *writability* of the forwarded place depends on the final target,
not on the alias. An alias does not create a fresh writable place; it points at
whatever place the final target owns, with that target's writability.

To reason about forwarding, the model introduces an `AliasChain` concept. It is a
semantic design object, not an implemented structure:

```text
source symbol
forwarded target
final symbol
final value
final place
provenance chain
writable boundary
cycle detection
```

The `AliasChain` records the path from the source alias symbol through any
intermediate forwarding to the final symbol, the final value and place, the
provenance of each hop, where the writable boundary lies, and whether the chain
contains a cycle. Cycle detection is part of the design because forwarding chains
must terminate.

## 6. Writable-place checking

A future writable-place checker decides whether a place may be written or
injected into from the current context. A place is writable only when it
satisfies the current stage and the current lexical/context boundary.

```text
Γ ⊢ place p writable_at current_context
```

At minimum, the following are **not** writable from an ordinary current-level
injection:

```text
core built-in stable object
external package stable object
closed generated object
alias whose final target is not writable
place from an inner lexical level escaping into a longer-lived injection target
place whose namespace delta is sealed/frozen
place whose policy does not admit the current injection action
```

Type-value equality grants no write permission. Even when:

```text
value(T) == value(uint8)
```

it does not follow that:

```text
place(T) == place(uint8)
```

and it certainly does not follow that:

```text
place(uint8) is writable
```

This is the concrete reason `let T === uint8; let f::T = ...` must be rejected:
the chain forwards to `uint8`, whose place is an external stable object and is
not writable from the current level. Alias forwarding cannot turn a non-writable
place into a writable one.

## 7. Namespace injection target

Namespace injection is a *place* operation, not a type-value operation. The
injection target is not determined by ordinary expression evaluation of the
target path.

The intended flow:

```text
parse / normalize injection target path
resolve path as injection-place target
follow alias chain only if alias semantics requires it
check final place writability
install NamespaceDelta under that place
```

The resolver here is asking "which writable place does this path name?", not
"what value does this path evaluate to?". An injection that resolves to a value
rather than a writable place is ill-formed.

## 8. Type values in overload and pattern matching

First-order type matching for overload and pattern compatibility uses
`TypeValueId`, not source symbol names. (The candidate-preparation layer that
consumes type values is specified in
`pattern-normalization-and-first-order-overload.md`; this document defines what
a type value identity is.)

For example:

```text
let T: type = uint8
```

In first-order type matching, `T` and `uint8` may carry the same `TypeValueId`.
But this says nothing about their places:

```text
T and uint8 may have the same TypeValueId but different PlaceId.
```

Pass mode is **not** part of `TypeValueId`. A construct such as `T move` does not
change the type value, and type-value comparison is invariant under
`move` / `copy` / `ref` / `share`. The detailed treatment of `T move == T` as a
move fixed point belongs to a future mechanical argument-passing / move design
and is only referenced here, not expanded.

## 9. Alias forwarding and policy

Alias forwarding redirects lookup; it does not grant capabilities. It must
operate within the existing policy, visibility, and writable-place restrictions.

```text
Alias may redirect lookup.
Alias may expose a forwarded value.
Alias must not manufacture permission.
Alias must not make non-writable places writable.
Alias must not bypass policy filtering.
```

If an alias target is not visible or not executable under the current
`PolicyEnv`, the alias does not make it visible or executable. A re-export or
wrapper semantics that intentionally re-exposes a forwarded target under
different policy is a separate, later design and is **not** defined here.

## 10. Relation to current implementation

The current implementation contains a placeholder, not the final model:

```text
The current v0.6/v0.7 lang_build slice still represents some `let T: type = uint8` cases using placeholder TypeObject payloads. That is an implementation placeholder, not the intended final semantics.
```

The intended final semantics defined by this document are: canonical
`TypeValueId` for type-value identity, a fresh `PlaceId` for ordinary
type-value binding, explicit alias forwarding through an `AliasChain`, and
writable-place checking at injection sites. None of these is implemented yet.

## 11. Non-goals

```text
No parser syntax change.
No Rust implementation change in this PR.
No test fixture change.
No full type checker.
No full alias resolver implementation.
No full lifetime/access-tree checker.
No runtime lookup implementation.
No package re-export semantics.
No permission escalation through aliasing.
No current public behavior change.
```

## 12. Relationship to other documents

The documents below are adjacent or background design. They do not define the
distinctions specified here, and this document does not depend on them for its
meaning.

- `type-associated-function-objects-and-access-trees.md` — field functions,
  projection namespaces, role-aware lookup, and access-tree work. It references
  this document for the canonical type-value / place / alias-forwarding
  distinction rather than restating it.
- `early-meta-functions-and-namespace-graph.md` — the build / namespace graph and
  early-meta slice, including the v0.6 placeholder `TypeObject` representation
  this document supersedes as the long-term semantics.
- `entity-alias-design.md` — the surface/parser alias syntax (`let binder ===
  EntityRef`) and frozen parser preservation. This document defines the
  *semantic* alias forwarding model (value/place forwarding, `AliasChain`,
  writable-place effect) that surface design will later target.
- `pattern-normalization-and-first-order-overload.md` — the pattern/type
  candidate-preparation layer that uses `TypeValueId` for first-order type
  matching.
