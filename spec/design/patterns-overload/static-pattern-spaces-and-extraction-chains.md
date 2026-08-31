# Static Pattern Relations and Extraction Chains

Status: Current canonical design

This document records the current positive boundary between Pattern relations,
structural extraction, and invocation. It does not freeze the final in-memory
representation of the complete Pattern space.

## 1. Pattern meaning

For environment `Gamma`, Pattern `P`, candidate object `c`, and valuation
`rho`, the canonical relation is:

```text
R_Gamma(P, c, rho)

Applicable_Gamma(P, c)
  iff exists rho. R_Gamma(P, c, rho)
```

The relation establishes applicability and extraction together. A successful
derivation carries the valuation of every extracted Hole. Generic deduction is
therefore ordinary Pattern extraction; it does not introduce a separate
language-level universal-quantification ontology.

Hole identity is qualified by its resolved Pattern root and `HoleBinderId`.
Display spelling does not participate in that identity.

## 2. Structural incidence

Object membership and structural incidence are distinct relations:

```text
Val2Member(x, selector)
  does not imply
DirectPatternChild(P, x, selector)
```

Likewise, an overload-visible ordinary member is not automatically a real
structural field. Structural incidence is established explicitly when the
Pattern value is formed and is observed through `DirectPatternChild` evidence.
Virtual or computed members remain available to ordinary member lookup without
becoming structural children.

## 3. Atomic structural extraction

Atomic structural extraction uses the registered real-field family for the
requested selector and applies the `StructuralDefault` family filter before
ordinary candidate enumeration:

```text
AtomicExtract_P(selector, x)
  = Resolve(
      RegisteredRealFieldFamily(P, selector),
      x,
      CallSiteFamilyFilter = StructuralDefault
    )
```

`StructuralDefault` is confined to Pattern interpretation. An ordinary source
member access receives no implicit structural filter and may select a virtual
or custom member according to the ordinary invocation pipeline.

Selected structural extraction failures obey the normal no-reopen rule. Once a
unique extractor candidate is sealed, execution, projection, capability, or
lifecycle failure does not select another extractor.

## 4. Product, sequence, and sum structure

Ordered and unordered structure are properties of the Pattern relation, not of
ordinary Val2 lookup. Product and sequence observations use ordinary Object
normalization for their elements. A sum derivation records the selected branch
and its nested derivation rather than converting the candidate into a different
value ontology.

The final canonical-space representation for the full Pattern algebra remains
open. Implementations expose an opaque relation/proof interface and must not
promote a convenient product or sum shape carrier into the canonical Pattern
IR.

## 5. Extraction chains and result boundaries

Nested extraction composes proof-relevant derivations. Each step observes the
value produced by the preceding step and extends the same qualified valuation
when binder identities agree. A branch miss is a failed derivation; an
execution failure after unique selection is a terminal invocation failure.

Call results use the shared `InvocationResult` boundary. Pattern extraction may
project a successful semantic result, but it does not define a private result
class, residual universe, or diagnostic channel.

## 6. Invariants

```text
Pattern != schema AST
Pattern != Product shape
Val2 member != DirectPatternChild
ordinary member != structural field
applicability and extraction share one R_Gamma derivation
generic deduction consumes Hole valuations
selected failure never reopens extraction overload resolution
```
