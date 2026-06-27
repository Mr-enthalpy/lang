# Pattern Normalization and First-Order Overload

**Status: Non-normative future design. Not implemented as current parser, normalizer, type checker, runtime lookup, or full overload resolution behavior.**

This document specifies the *pattern/type candidate-preparation layer* that must
exist before a formal meta object invocation model can select callables. It is a
future design note. It is not current public language behavior, not an
implemented pass, and not a parser or normalizer rule.

The document is self-contained. It does not require the reader to assemble its
meaning from `overload-resolution-design.md`,
`static-pattern-spaces-and-extraction-chains.md`, or
`early-meta-functions-and-namespace-graph.md`. Those documents are background
context only; the design here stands on its own.

## 1. Purpose

The next semantic step for the language is **not** to build runtime lookup
first, and **not** to build a full type checker first. It is to let *patterns*
and *first-order type values* participate in how meta-invocation candidates are
formed and filtered. A callable's applicability cannot be decided by name alone;
it must also be decided by the normalized pattern of each parameter slot, the
first-order shape of each argument expression, and first-order type-value
compatibility.

This document defines the preparation layer that turns a call into an applicable
candidate set:

```text
source / normalized call
  -> raw argument shape
  -> normalized parameter pattern
  -> first-order type-value compatibility
  -> overload/meta-invocation candidate applicability
```

This document does **not** define:

- full runtime overload resolution,
- a full type checker,
- full pattern-space / extraction-chain semantics,
- lifetime / access-tree construction,
- implicit conversion or coercion.

"Pattern normalization" here is deliberately narrow. It is not full pattern
matching, not a runtime `match`, not exhaustiveness checking, and not arbitrary
set algebra. It is a static object-ization step that runs *before* a normalized
expression enters the semantic candidate model. It takes the parameter slots,
binding positions, type positions, path positions, discards, and product
patterns found in source/normalized material and turns them into comparable,
provenance-carrying objects (`PatternObject`, `ArgShape`, `ParameterShape`) that
can participate in candidate matching.

Throughout this document, one rule is load-bearing: **a pattern is a pattern and
a value is a value, and the two do not implicitly convert.** A value does not
silently become a pattern, and a pattern does not silently become a value.
Pattern-side name resolution has its own bounded rules and must not fall back to
ordinary value/function lookup.

## 2. Why pattern normalization comes before formal invocation

Meta object invocation cannot be decided by name lookup alone. A callable must
be able to carry a parameter pattern; a call's arguments must be able to form an
argument shape; only then can a candidate be judged applicable or not.

A future call-selection step must therefore not be merely:

```text
lookup callee name
```

It must be at least:

```text
lookup callee name
collect candidates
normalize argument shapes
match candidate parameter patterns
check first-order type-value compatibility
filter by policy/body-entry
select callable
```

The "overload" introduced at this layer serves meta object invocation first. It
is **not** equivalent to full runtime overload resolution. It is the narrower
candidate-preparation subset that the formal meta invocation engine needs in
order to choose a callable at meta time. Full runtime overload resolution
remains a later, broader design.

### 2.1 Product semantic normalization bridge

`RawArgShape` formation consumes `ArgProductShape`; it must not read raw
Normalized AST product structure directly. The bridge from normalized product
material into argument-shape formation is:

```text
NormProduct
  -> ProductObject
  -> FlattenedProductObject
  -> ArgProductShape
  -> RawArgShape
```

Product semantic normalization is not surface normalization. It produces a
semantic product object used by candidate preparation and future meta
invocation:

```text
flatten crosses Product nodes.
flatten does not cross Expression nodes.
order is preserved.
Unit is preserved.
provenance is preserved.
```

Formal skeleton:

```text
NF_P(P) = ordered sequence of product atoms

NF_P((x1, x2, ..., xn))
  = concat(NF_item(x1), NF_item(x2), ..., NF_item(xn))

NF_item(Product p) = NF_P(p)
NF_item(Expression e) = [e]
NF_item(Unit) = [Unit]
```

Examples:

```text
((P, e), e)   -> (P, e, e)
((e, e), e)   -> (e, e, e)
((e, P), e)   -> (e, P, e)
(e, (P, e))   -> (e, P, e)

((a, b), c)         -> (a, b, c)
(a, (b, c))         -> (a, b, c)
((a |> f), (b, c))  -> (a |> f, b, c)
((a, b) |> f, c)    -> ((a, b) |> f, c)
```

Forbidden:

```text
((a, b) |> f, c) -> (a, b, f, c)
```

The reason is that `(a, b) |> f` is an Expression barrier. Candidate
preparation may normalize a call's own source product, but it must not flatten
from an outer product context through an expression/call node.

`Unit` positions remain in the product object:

```text
((a,), b) -> (a, Unit, b)
((), a)   -> (Unit, a)
```

Only a future explicit parameter expectation / extraction rule may consume
`Unit`; product semantic normalization itself does not delete it.

This layer prohibits callee-specific AST argument parsing, special handling of
`(T)Vec` / `(T)Option` / `(A, B)Pair`, flattening through Expression nodes, and
dropping `Unit` during product normalization.

## 3. PatternObject

A `PatternObject` is the normalized, object-ized form of a pattern-position
fragment. The set below is intentionally small — enough to express candidate
signatures, not a full pattern language.

```text
PatternObject ::=
  Binder(name)
  TypedBinder(name, TypeRef)
  Product([PatternObject])
  Discard
  Path(path)
  Literal(literal)
  Unsupported(...)
```

This is a design sketch, not an implementation requirement. The exact Rust
shapes are deferred.

Every `PatternObject` must carry enough metadata to be compared and diagnosed:

```text
origin / provenance
source role
normalized role
```

- `origin / provenance` ties the object back to its source/normalized location
  for diagnostics and future caching.
- `source role` records how the fragment appeared in source/normalized material.
- `normalized role` records the occurrence role this layer assigns (see §4).

`Unsupported(...)` exists so that fragments this layer cannot yet normalize stay
visible as explicit objects rather than being silently dropped, mirroring the
normalizer's existing `Unsupported` discipline.

## 4. Occurrence roles

The same source-looking name means different things in different positions. This
layer must classify each occurrence into a role rather than treating every name
as an ordinary lookup. At minimum:

```text
binder occurrence
type occurrence
path occurrence
literal occurrence
discard occurrence
```

- **binder occurrence** introduces a name. It does **not** perform ordinary
  lookup. A binder is a fresh pattern-side name, not a reference to an existing
  value or function.
- **type occurrence** enters first-order type-value resolution (see §5). It is
  compared by type value, not by source name.
- **path occurrence** is ordinary path material, but in pattern-side position it
  is subject to bounded pattern lookup. It must not silently fall back to
  ordinary value/function lookup.
- **literal occurrence** is a literal used as pattern material.
- **discard occurrence** explicitly consumes structure at that position without
  binding a value.

Because these roles are distinct, the same spelling can be a binder in one slot
and a type in another. The classification is structural, decided by position,
and never by re-interpreting a pattern as a value. This is the concrete form of
the §1 rule that patterns and values do not implicitly convert: a binder
occurrence never resolves as a value, and a value never re-enters as a binder.

## 5. First-order type values

When a first-order type participates in candidate matching, the thing compared
is **not** a symbol name. It is a canonical type value.

The conceptual identity used here is:

```text
TypeValueId
```

For example:

```text
let T: type = uint8
```

In a type-value position this should mean:

```text
value(T) == value(uint8)
```

That is, overload and type compatibility compare *type values*, not source name
identity. Two bindings can share a type value even though their binding symbols
differ.

This document does **not** fully specify symbol identity, places, or injection
targets; the canonical `TypeValueId` / `PlaceId` / `SymbolId` distinction is
defined in `spec/design/symbol-world/type-values-places-and-alias-forwarding.md`. Here it is
enough to state the comparison rule: candidate matching uses type value, not
source name.

Pass mode is explicitly **not** part of `TypeValueId`. A construct such as
`T move` does not introduce a new type, and `move` / `copy` / `ref` / `share`
must not change type-value comparison. Type-value equality is invariant under
pass mode. Pass mode is a separate dimension handled elsewhere and must never be
folded into the type value used for candidate matching.

## 6. RawArgShape and ParameterShape

This layer describes both sides of a candidate fit with early *shape* objects.
Neither is a full type-check result; both are inputs to candidate adaptation.

`RawArgShape` describes a call argument's shape:

```text
is value?
has explicit pass mode?
known first-order type value?
is meta object?
is type/rank object?
is residual/runtime expression?
origin/provenance
```

`ParameterShape` comes from a callable's parameter pattern:

```text
parameter pattern
expected first-order type value, if known
expected rank/classifier, if known
pass expectation, if present
policy requirements, if present
```

`RawArgShape` and `ParameterShape` are candidate-adaptation inputs, not final
IR. They carry only as much information as the candidate-preparation layer can
establish statically; they do not assert a completed type check, and they are
not a lowering target.

The `pass expectation` on a `ParameterShape` and the `explicit pass mode` /
`is value?` facts on a `RawArgShape` are consumed by the mechanical
argument-passing layer, which inserts a concrete pass action (move/ref/share/copy)
after or within candidate adaptation. Pass matching is separate from type
matching: pass mode is not part of `TypeValueId`. See
`spec/design/mechanical-lowering/mechanical-argument-passing-and-move-fixed-point.md`.

## 7. Applicability judgment

Candidate applicability is expressed as a small judgment over shapes:

```text
Γ ⊢ arg ⇓ RawArgShape

Γ ⊢ parameter_pattern ⇓ ParameterShape

Γ ⊢ RawArgShape ≤ ParameterShape ⇓ applicable / not_applicable / deferred
```

The three outcomes:

- `applicable` — current static information is sufficient to confirm the
  candidate can be used.
- `not_applicable` — current static information is sufficient to reject the
  candidate.
- `deferred` — the judgment depends on a runtime value/type check and cannot be
  completed in the meta candidate-preparation stage.

`deferred` only records a boundary. Whether a `deferred` result may enter a
residual runtime path is decided by the meta object invocation / runtime lookup
documents, not here. This layer's responsibility is to mark the boundary
precisely, not to resolve what happens past it.

## 8. Specificity

Candidate preparation needs a stable notion of specificity to choose among
applicable candidates. This document does not restate the full overload
specificity design — that lives in `overload-resolution-design.md`. The early
meta-invocation candidate subset only requires the following ordering
properties:

```text
more specific pattern wins over less specific pattern
deeper explicit structure is more specific than shallow discard
type-specific pattern is more specific than unconstrained binder
ambiguous equally-specific candidates are hard ambiguity
```

This document does **not** claim a fully implemented lexicographic specificity
rank. It constrains the *shape* of specificity that the candidate-preparation
layer relies on, and defers the complete rank to the broader overload-resolution
design. The early subset may reuse or constrain that future rank, but it does
not promise it as implemented behavior.

## 9. Relation to meta object invocation

The output of this layer is not an execution result. It is the *input* to the
formal meta object invocation engine. The end-to-end pipeline is:

```text
normalized call
  -> callee lookup under policy
  -> candidate collection
  -> argument shape formation
  -> parameter pattern normalization
  -> first-order type-value compatibility
  -> applicable candidate set
  -> meta object invocation
```

This document covers only the preparation portion — from argument shape to the
applicable candidate set. It stops at the boundary where the meta object
invocation engine takes over. The invocation engine itself (policy-governed
execution, partial vs strict reduction, residualization) is specified in
`meta-object-invocation-and-policy-reduction.md`.

## 10. Relation to runtime lookup

Runtime lookup is intentionally placed **after** this layer.

The reason is structural. If runtime lookup is pulled in early, the compiler is
forced to improvise an ad hoc set of decisions before pattern normalization,
type-value equality, the alias/place distinction, pass insertion, and the meta
invocation model exist. That improvisation tends to congeal into a second,
parallel resolver and type checker — exactly the duplication this sequencing is
meant to avoid.

The purpose of this document is to establish the pattern/type candidate model
that meta invocation needs. Runtime lookup and full first-order type checking
should be connected only after that model is stable, so that they consume a
well-defined candidate-preparation layer instead of re-deriving one.

## 11. Non-goals

```text
No parser syntax change.
No current normalizer behavior change.
No Rust implementation change in this PR.
No full runtime overload resolution.
No full type checker.
No full pattern-space algebra.
No match/exhaustiveness checker.
No implicit value-pattern conversion.
No runtime lookup implementation.
No ABI or IR lowering rule.
```

## 12. Relationship to other documents

The documents below provide background and adjacent design. They are not
load-bearing for the meaning of this layer, and this document does not depend on
them for its definitions.

- `meta-object-invocation-and-policy-reduction.md` — the formal meta object
  invocation engine that consumes the applicable candidate set produced here.
- `overload-resolution-design.md` — the broader, later full overload-resolution
  design. This document defines only the earlier, narrower candidate-preparation
  subset, which is not equivalent to full runtime overload resolution.
- `static-pattern-spaces-and-extraction-chains.md` — the fuller, later
  pattern-space / extraction-chain semantics. The pattern normalization layer
  here is an earlier candidate-shape layer and is a different layer from that
  pattern-space design.
- `type-values-places-and-alias-forwarding.md` — the canonical `TypeValueId` /
  `PlaceId` / `SymbolId` distinction that first-order type matching here relies
  on. First-order type matching uses `TypeValueId`; that document defines what
  type-value, place, and symbol identity mean.
- `type-associated-function-objects-and-access-trees.md` — background for
  type-associated function objects, field functions, and access-tree work.
- `mechanical-argument-passing-and-move-fixed-point.md` — the mechanical
  argument-passing layer that consumes `RawArgShape` / `ParameterShape` pass
  expectations and inserts concrete pass actions (`move` as the fixed point).
