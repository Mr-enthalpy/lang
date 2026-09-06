# Meta Invocation and Policy Reduction

Status: Current canonical design

Meta evaluation is ordinary semantic evaluation at a meta stage. It shares the
Object, Pattern, complete type, OverloadGroup, Place, Policy, invocation, and
lifecycle relations, with the same structural name-binding relation with compile and runtime evaluation.

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
it is never itself reported as an additional semantic result class.

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

### 2.1 Compilation entry uses ordinary meta-root formation

`Compile(Level)` enters an ordinary meta invocation, not a `compile`-stage
callable that manufactures a root. Its entry callable is the ordinary meta body
obtained from the normalized source tree. The existing core bootstrap's owner
root supplies its parent placement; physical discovery supplies body material,
not an owner, Place, or authority. `F_entry` below denotes that callable's
semantic identity, not a new primitive or surface name:

```text
WellFormedMetaCall_Gamma(F_entry, args_entry)
  => M_compile = MetaInstanceRoot(
       ParentSemanticOwner_Gamma(F_entry),
       MetaInstanceKey(F_entry, Canonicalize(args_entry)))
```

This is precisely the [ordinary meta formation law](../symbol-world/symbol-first-meta-construction-and-pattern-injection.md#41-orthogonal-dimensions).
Entry and any argument dependencies must satisfy its ordinary admissibility
and global-keyability premises. A filename or raw Level string is not a
substitute for callable/argument identity. The existing bootstrap provision of
owner roots is described in the [bootstrap consumer map](../symbol-world/early-meta-functions-and-namespace-graph.md#core-and-early-semantic-operations);
it is fixed language bootstrap, not configurable build input.

The invocation's default result construction has an already formed complete
resident `tau_M` in its ordinary result Place (§4.3.3 of the construction owner).
`r_root` is the ordinary `mut type ref` to that Place under the invocation's
result-construction write authority. It is not a reference to the semantic
owner identity `M_compile`. Its subject is the result's existing construction
window, with `Anchor(tau_M) = <M_compile, epsilon>`. The active entry frame
therefore supplies `AuthorityMatches`; `WindowLive` and the result Place's
ordinary Writable facts remain separate premises. Neither root stability nor
its `plain` policy supplies Writable.

Normalized root actions execute inside this same meta invocation, using
`r_root`; they do not enter a second meta frame that would mask its authority.
Normal return performs the existing default-result seal and ends that window.
Saved references cannot bypass later write Pre. Thus Level selects the source
tree supplied to normalization, while evaluator invocation/result formation
alone establishes the root and its authorized construction context.

### 2.2 Generic symbolic anchor and compile realization

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
  -> explicit outer binding action under ordinary commit rules
```

Closure construction uses the current stable meta-instance owner. Injecting the
resulting closure into a type under construction is a separate operation.
NameBinding is a structural identity/Place relation, not a returned Object or
a borrowable wrapper. A complete type retains its own Core and callspace;
neither a result Place nor a defining binding supplies an additional callspace.
Fresh structural binding installs the complete empty T_0 before returning its
mut type ref; subsequent assignment remains ordinary assignment.

## 5. Partial evaluation

Partial evaluation may return `Residual` only through the shared result
boundary. It does not define a meta-specific result universe or a second value
ontology. Unsupported forms remain explicit residuals or diagnostics until the
shared evaluator and continuation consumers are connected.


The shared continuation, saturation and projection laws are owned by
[evaluation and optimization](evaluation-residual-and-optimization.md).
Host acquisition returns ordinary Objects under
[host capabilities](host-capabilities-and-machine-objects.md).
