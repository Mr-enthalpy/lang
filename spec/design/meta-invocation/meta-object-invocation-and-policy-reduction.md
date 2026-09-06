# Meta Invocation and Policy Reduction

Status: Current canonical design

Meta evaluation is ordinary semantic evaluation at a meta stage. It shares the
Object, Pattern, complete type, Symbol, Place, Policy, invocation, and lifecycle
universes with compile and runtime evaluation.

## 1. Invocation boundary

Every selected callable declares one result class:

```text
InvocationResult(F)
  = SemanticResult(DeclaredResultClass(F))
  | Residual
  | Diagnostic
```

One callable declaration carries independent result coordinates:

```text
ReturnDeclaration(F)
  = DeclaredResultClass(F)
    × ReturnPattern(F)
    × ResultPolicy(F)
```

The return Pattern is interpreted by the Pattern relation. It does not define
or refine `DeclaredResultClass(F)`.

Primitive and source bodies may use private execution material. Execution
material is installed or interpreted before `SemanticResult` is constructed;
it is never itself reported as a complete type, Symbol, or ordinary value.

`struct` declares `CompleteType` and returns an actual complete type value
`tau`. The binding boundary receives `tau` explicitly and may derive a Core
lookup projection only after the semantic binding exists.

## 2. Meta-instance identity

```text
MetaInstanceRootKey
  = parent SemanticOwner
    × selected callable identity
    × canonical whole argument Product identity
```

Parent-neutral execution material may be cached by a smaller material key. Such
cache reuse never reuses or determines the semantic root identity.

Every meta-instance root is a stable semantic owner with whole-slot
`PolicyMode::Plain`. Root consistency is an owner/identity invariant, not a
const projection, and plain does not imply writable.

### 2.1 Generic symbolic anchor and compile realization

MetaPartner(F) = M(F) names the generic symbolic anchor; CompilePartner(F) =
C(F) names the derived compile realization. These are independent roles.

| Callable | Distinct compile partner | Generic meta partner |
| --- | --- | --- |
| runtime generic F | C(F) | M(F) |
| compile generic F | none | M(F) |
| meta F | none | none |

Non-generic callables use their existing CallableRoot for ordinary symbolic
anchoring. Compile companion derivation does not authorize E to change a
runtime binding's stage or give an optimizer its own semantic facts.

## 3. Policy positions

Formal pair inheritance and whole-slot mode are different dimensions:

    Pair(P_in) = Pair(P2)
    Mode(P_in) = explicit formal mode or plain

The return position retains its separately specified P_out = Overlay(P1,
Delta_out) rule, including inherited omitted mode. It is not used to infer a
formal's mode. Evaluation stage is inherited and cannot be overwritten by a
position annotation. Neither position policy grants Writable or changes the
caller's independent result demand.

Call-site `ResultPolicyDemand` is formed independently before maxima and
participates in the ordinary invocation pipeline.

## 4. Construction and installation

Meta construction separates:

```text
body execution material
  -> semantic entity materialization
  -> InvocationResult
  -> outer binding and atomic namespace installation
```

Closure construction uses the current stable meta-instance owner. Injecting the
resulting closure into a type under construction is a separate operation.
Neither a construction return Place nor its binding Symbol becomes a
`HomeSymbol` of the complete type value.

## 5. Partial evaluation

Partial evaluation may return `Residual` only through the shared result
boundary. It does not define a meta-specific result universe or a second value
ontology. Unsupported forms remain explicit residuals or diagnostics until the
shared evaluator and continuation consumers are connected.


The shared continuation, saturation and projection laws are owned by
[evaluation and optimization](evaluation-residual-and-optimization.md).
Host acquisition returns ordinary Objects under
[host capabilities](host-capabilities-and-machine-objects.md).
