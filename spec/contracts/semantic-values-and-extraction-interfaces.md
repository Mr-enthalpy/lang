# Semantic Values and Extraction Interfaces

Status: Current implementation contract

The semantic value universe is the ordinary Object universe:

```text
Object(x) = <Val1?(x), Pattern(x), Val2(x)>
```

Construction bodies may use private replay material, but only the declared
semantic result crosses the invocation boundary. `struct` materializes and
returns a complete type value. `StructConstructionMaterial` remains private to
execution. Struct Pattern syntax material is converted to
`CanonicalPatternValue` before it participates in semantic relations.

`R_Gamma(P,c,rho)` is the sole Pattern applicability and extraction relation.
Its content input is the Object's `Val1?` and owned `Val2`. Structural
extraction additionally requires explicit `DirectPatternChild` evidence and
the `StructuralDefault` family filter.

Complete type, NameBinding, OverloadGroup, Place, Pattern root, and semantic value identities remain
separate throughout construction and extraction.
