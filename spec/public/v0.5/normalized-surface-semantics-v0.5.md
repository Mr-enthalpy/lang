# Normalized Surface Semantics v0.5

> **Status:** Authoritative outline (v0.5-1). This document defines the section
> structure and records the central call-binding rule now. The full prose and
> worked examples are filled in by later v0.5 PRs (v0.5-2 / v0.5-3). Where a
> section is a placeholder, the backing decisions already live in
> `spec/public/v0.3/normalized-ast-specification-v0.3.md` (§7, §8) and
> `spec/contracts/v0.4-normalization-prototype-notes.md`.

## 1. Purpose and Scope

This document is the public explanation of the normalized surface: the behavior
that the v0.4 normalizer already produces, described for readers and agents
rather than as a contract or a design discussion. It explains how source text is
read at the normalized layer. _(Placeholder: full prose in v0.5-2.)_

## 2. Stage Boundary

v0.5 stabilizes the normalized surface and the public documentation. It is still
non-semantic: no name resolution, type/kind checking, operator lookup,
pattern-head resolution, HIR, closure materialization, runtime evaluation, or
code generation. _(Placeholder: expanded boundary statement in v0.5-2.)_

## 3. Source Product and Target Expression

Defines the notation `P` (source product), `e` / `TargetExpr` (target-capable
expression), and `G` (group), and the rule that a product is a source, not a
target. _(Placeholder: full definitions in v0.5-2; backing: v0.3 §7.1–§7.2.)_

## 4. Source-Product Continuation

The central rule of normalized call binding is:

```text
Product1 |> TargetExpr Product2
=> (Product1, Product2) |> TargetExpr
```

A source product written discontinuously around a target expression is closed by
product merge into a single normalized call.

```text
This is not traditional callee-first function-call syntax.
A following Product is source-product continuation when an incoming source Product exists.
`f Product g` is a legality repair, not ordinary local call sugar.
```

_(Placeholder: full first-product-only rule, growth model, and worked examples in
v0.5-2; backing: v0.3 §7.2–§7.7.)_

## 5. Legality Repairs

Describes the first legality repair (`P |> e` when no continuation product
exists) and the second legality repair (`e ... P e ... => e ... (P |> e) ...`,
which is what `f Product g` actually is), and the invariant that `expr |> Product`
is never the intended normalized result. _(Placeholder: full prose in v0.5-2;
backing: v0.3 §7.5–§7.6.)_

## 6. Product, Group, and Unit Boundaries

Describes product-lifting, the group `G` that does not survive as a normalized
expression node, `()` as a Product containing one Unit element, and the rule that
nested products are not recursively flattened. _(Placeholder: full prose in
v0.5-2; backing: v0.3 §7.8, §8.1.)_

## 7. Operator / Member / Double-Dot / Bracket Sugar

Describes operator/member/double-dot/bracket sugar as normalization-level
lowering into the product-call skeleton, with preserved operator provenance
(spelling, fixity, arity, span) and unresolved navigation targets. No operator
lookup, field lookup, or method dispatch occurs. _(Placeholder: full prose and
examples in v0.5-2; backing: v0.3 §7.8–§7.11.)_

## 8. Value-Side vs Pattern-Side Material

Describes the boundary that value-side material remains `NormExpr` and
pattern-side material remains `NormPattern`; a value enters pattern space only
through an explicit bridge, and a pattern exposes values only through explicit
extraction. Pattern-side names are not ordinary call targets. _(Placeholder: full
prose in v0.5-3; backing: v0.4 prototype notes.)_

## 9. Annotation Patterns and DeduceList Holes

Describes annotations as annotation-pattern (classifier-pattern) material, not
ordinary runtime expressions, and DeduceList as a binding-site hole binder list
whose holes may appear inside annotation patterns. _(Placeholder: full prose in
v0.5-3; backing: v0.3 §8.2–§8.5.)_

## 10. Alias Preservation

Describes alias declarations preserved as unresolved declarations with an
`EntityRef` right-hand side; no target resolution, scope semantics, or operator
alias identity validation occurs at the normalized layer. _(Placeholder: full
prose in v0.5-3; backing: v0.3 §7.13, §8.8.)_

## 11. Origin, Generated Nodes, Derived Nodes, and Unsupported

Describes node provenance (source / generated / derived), how generated and
derived nodes are traceable, and how unsupported Raw AST subshapes remain visible
as `Unsupported` in the normalized output instead of being silently erased.
_(Placeholder: full prose in v0.5-3; backing: v0.3 §7.15, §8.10; v0.4 prototype
notes.)_

## 12. Non-Goals

The normalized surface does not perform name resolution, type/kind checking,
operator lookup, operator overload resolution, alias target resolution,
namespace resolution, pattern-head resolution, canonical matching, closure
materialization, capture analysis, ownership/NLL/drop, effect interpretation,
runtime evaluation, or code generation. It does not turn Normalized AST into HIR.
_(Backing: `spec/contracts/v0.4-normalization-prototype-notes.md`.)_

## 13. Relation to v0.6+ Future Semantics

Later pattern-space and extraction-chain semantics
(`spec/future/static-pattern-spaces-and-extraction-chains.md`) motivate the
current normalized boundaries but are **not** implemented by the v0.5 normalizer.
`Done`, residual propagation, pattern-space subtraction, `operator+`
meta-reduction, `match` closing, exhaustiveness, and pattern-head resolution are
future semantics, not current behavior, and must not be read as implemented.
