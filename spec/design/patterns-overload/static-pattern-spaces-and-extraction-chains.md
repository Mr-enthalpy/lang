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

## 5. Residual splitting and extraction chains

```text
Split_Gamma(A,S) -> <H,R,delta>
D(A,S) = R
```

H is the matched part, R the residual and delta the proof-relevant extraction
evidence. This is a restricted consumer of R_Gamma; it does not assert a
general Boolean difference, arbitrary Pattern complement or synthesized
inverse. A finite sum with registered direct cases admits its corresponding
case split. Product integrity obligations are separate from a sum branch
miss. Once a unique extractor is sealed, its execution/projection/capability
failure is terminal; it does not turn into a miss or try the next extractor.

An extraction chain has an internal boundary identity chi:

```text
Chain_chi = <R, Q_completed>
match H -> evaluate selected branch -> Done_chi(v)
miss R -> retain ordinary residual for the next branch
```

Done_chi(v) is evaluator completion state, never an Object, Pattern, user
value, name, Val2 member, stored value or source constructor. It participates
in no ordinary lookup, Norm, @, ref/share, migration or Pattern matching.
A user declaration named Done is ordinary and grants no completion privilege.
The boundary consumes its own completed channel and exposes only ordinary
payload results. A nested chain unwraps at its boundary before the outer
chain makes its independent completion; no user-visible Done(Done(v)) arises.

Compile-known guards execute only the chosen branch; unchosen bodies produce
no effects or require/lookup obligations. A runtime guard retains both
possible branch continuations, sealed identities and their lawful effects
until the choice is ready. A seal dependency defers the same guarded action.

Targeted return uses a distinct internal target completion with ordinary
ReturnPattern delivery, as owned by
[targeted return](../control-flow/targeted-return-and-d-reduction.md).
It contributes no fabricated local unit result.

## 5.1 Residual escape is a separate boundary decision

```text
CanEscape_Sigma(R,B)
```

This fixed consumer asks whether residual R may leave boundary B. It does not
change D, branch matching, Done or require an empty residual universally.
An if|else Pattern can legally flow through its matching chain and be rejected
only where a forbidden residual would escape; other ordinary residuals may
escape when admitted.

F_residual is an ordinary meta query returning its instance type tau_M. Its
allow/deny payload lives in ordinary Val2. Default instance formation,
OpenHere/Writable member customization, current committed reads, snapshot/Close
discipline and cache/dominance checks follow
[ordinary meta defaults](../meta-invocation/meta-object-invocation-and-policy-reduction.md#7-ordinary-meta-defaults-and-current-state-consumers).
Writing an outer result copy does not customize the retained instance. The
consumer reads at its own frontier; later changes have no retroactive effect.
SealDom cannot invoke meta indirectly to answer the query; an independently
available completed observation must suffice or the action is unavailable.

## 5.2 Result Pattern boundary

With an expected result Pattern R, every delivered ordinary payload is checked
against R through ordinary extraction. Without an expected R, use the
ordinary partial result-combination relation where defined. There is no
universal least upper bound and no theorem that unit absorbs arbitrary results.
Control completion is not a value that participates in result joining.
InvocationResult remains the sole semantic result/Residual/Diagnostic envelope.

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
