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

## 3. Policy positions

Function parameter and return positions are coordinate-wise overlays:

```text
P_in  = Overlay(P2, Delta_in)
P_out = Overlay(P1, Delta_out)
```

Evaluation stage is inherited and cannot be overwritten by a position
annotation. Whole-slot PolicyMode may be explicitly overwritten; otherwise it
inherits from the base. Position policy does not grant capability or
writability and does not propagate caller demand back into declaration policy.

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
