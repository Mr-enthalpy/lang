# Pattern Normalization and First-Order Invocation

Status: Current canonical design

Normalized AST preserves Pattern roots, Deduce lists, Hole binders, products,
packs, and PolicyLet boundaries without assigning semantic meaning. Semantic
Pattern interpretation begins only after stable owner and root qualification.

## 1. Identity

```text
HoleIdentity = ResolvedPatternRootId × HoleBinderId
```

Binder spelling is diagnostic provenance. Binderless `<>`, wildcard `_`, and a
named Hole are distinct Pattern forms.

## 2. Normalization boundary

Pattern normalization observes Pattern content without converting it into a
schema or type AST. Object normalization records `Norm_P(P(x))` alongside
`Norm_Val1?` and `Norm_Val2`; complete type observation uses its separate Core
and whole-snapshot normalization.

The final canonical in-memory representation of the full Pattern space remains
open. Implementations therefore expose opaque Pattern handles and relational
derivations rather than a public shape algebra.

## 3. First-order applicability

First-order invocation asks the relation engine for every applicable valuation:

```text
R_Gamma(P, actual, rho)
```

Hard A consumes the derivation. Generic deduction consumes `rho`. Pattern
specificity consumes proof structure at its later-B stage. These consumers do
not re-run lexical name resolution and do not fall back to arity/name matching.

## 4. Structural content

Product and sequence observations are ordinary semantic content. Real
structural fields require registered incidence; ordinary Val2 presence and
virtual members do not create `DirectPatternChild` evidence. Atomic extraction
uses the protected `StructuralDefault` candidate family.

## 5. Result boundary

Pattern extraction projects ordinary semantic values and complete types through
the shared `InvocationResult`. Its observations and derivations are
proof/extraction material rather than semantic value classes.
