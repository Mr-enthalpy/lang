# Semantic Values and Extraction Interfaces

Status: Current implementation contract

The semantic value universe is the ordinary Object universe:

```text
Object(x) = <Val1?(x), Pattern(x), Val2(x)>
```

Construction bodies may use private replay material, but only the declared
semantic result crosses the invocation boundary. `struct` materializes and
returns a complete type value. No construction record, generated-definition
record, or observed shape forms an additional semantic value class.

Pattern applicability and extraction use `R_Gamma(P,c,rho)`. Observed argument
or product content is transport into that relation, not a Pattern IR and not an
independent equality authority. Structural extraction additionally requires
explicit `DirectPatternChild` evidence and the `StructuralDefault` family
filter.

Complete type, Symbol, Place, Pattern root, and semantic value identities remain
separate throughout construction and extraction.
