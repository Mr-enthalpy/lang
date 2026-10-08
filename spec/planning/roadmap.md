# Implementation Roadmap

Canonical meaning belongs to the [topic owners](../design/README.md).
This document records consumer coverage and dependency order. Passing storage
or syntax tests establishes their stated slice, not complete source evaluation.
Representation choices belong in [open questions](open-questions.md).

## 1. One source and evaluation pipeline

```text
source -> weak tokens -> Raw AST -> Normalized AST
       -> typed owner / namespace resolution
       -> canonical E -> InvocationResult

EntryContinuation
  -> common E
  -> ordinary completion / actual remaining continuation / diagnostic
```

Raw AST preserves source and recovery. Norm is syntax-directed and non-semantic.
It performs no lookup, Pattern applicability, closure formation or execution.
The [unified owner](../design/unified-static-name-and-structural-semantics.md)
owns static/name/structural interpretation and instance formation.

## 2. Consumer dependency order

1. Independent PolicyMode={const,mut}, OpenPolicy={open,close} and
   Stage={compile,seal,runtime}; total output demand before maxima.
2. Resolve once; ordinary associated Val2[()] entrance and exact immutable V_tau
   expansion; R_vis evidence and ordinary C_sigma preparation.
3. Stable CompilePartner receiver/call-entry pair and canonical inputs;
   CompileInstance before EnterBody, independent of result kind and P2;
   initialized ordinary self-name/resident reads.
4. Complete accessible-result closure, opening subjects and direct single
   instance-open self-root check before outward delivery.
5. Common action transaction/scheduler: affected Pre, one commit, joint Post,
   ordinary result delivery/completion and independent open/close.
6. V/S polarity and arbitrary $, shared bare-name navigation, structural
   formation with actual Val2 role witnesses and a complete type.
7. Ordinary struct:type->type helpers preserving structural registration;
   ordinary * / *= selection with checked read-transform-write.
8. Inherited file navigation and qualified let/name installation, serial files,
   unordered common-snapshot sibling overlays.
9. Requested-name producers accepting any ordinary result independently of
   NameExpr shape; ordinary ADL receiver/type-path overload families.
10. Path consumers, dependencies, source lifecycle, fixed cleanup,
    ordinary remaining-continuation transport and E saturation.

Every step preserves one identity/authority relation and selected no-reopen.

## 3. Substrate and source boundaries

| Relation | Representation / connected slice | Consumer boundary |
|---|---|---|
| Object and complete tau | Val1/Pattern/owned-Val2 normalization; immutable Core and V_tau observations; exact observed Core material and callspace transport into fresh bindings and constructor results | Changed nested resident material reports unavailable; persistent captured-child representation and rich opaque payload normalization remain open |
| Name and owner identity | Parent-linked owners, root-qualified Holes, name/Place/generation coordinates | General NameValue/read, qualified formation and lexical Path aliases require ordinary consumers |
| Applicability | R_Gamma proofs, Hole valuations and parameter preparation | Protected StructuralDefault, structural role families and richer extraction require consumers |
| Policy | Explicit observation/demand/preference, migration and derived 2x2 capability views | Source deduction, operation-driven premises and full R_vis/C_sigma are separate obligations |
| Selection | Exact receiver/call-entry enumeration, total demand, sealed selection and diagnostics | Unknown applicability cannot become fallback evidence; source completion requires common E |
| Instance identity | Parent/selected-partner/canonical-input interning; independent ordinary self-name coordinate and exact selected frame; connected external-type delivery | Initialized self residency, accessible-result closure, dependency-sensitive current storage and general completion require common consumers |
| Structural relations | Layer-local schema normalization and a DirectPatternChild observer requiring both registration and current actual Val2 membership | Source type formation, struct helpers and ordinary * / *= require common producer Pre/Post and real role/call entries |
| Place | Binding/resident generations, Writable and borrow substrate | Source ref/share/rebind, initialization and invalidation require common transactions |
| Lifecycle | SemanticContinuation, LifeName, Region, supplied Pre, Kill/Preserve, fixed cleanup and Color | Source producer facts, NLL/with, ReifyLife and destructors need ordinary control-flow facts |
| Physical source | Discovery/decoding/provenance and normalized fragments | EntryContinuation, inherited navigation, sibling overlays and dependency projection need E composition |
| InvocationResult | Semantic result, residual and diagnostic transport | Residual requires a real remaining continuation; serial completion and residual ABI remain separate |

A missing canonical consumer reports unavailable or preserves an established
remaining continuation. Representation supplies no substitute evaluator,
identity, result or authority. Production invocation consumes the actual
semantic call entry; graph declarations are rendering material.

## 4. Common E acceptance gates

Visible, Ready and LegalToExecute have distinct consumers. Visibility cannot
establish a successful producer. The diagnostic-only selected-source-body
frontier must be absorbed by common E; it must not accumulate initializer,
binding, nested-call and return execution as another evaluator.

Every semantic action obeys the cleanup gate:

```text
OutstandingCleanupBefore(k) => action may not commit across that point
```

This includes actions without a lifecycle projection. The common scheduler or
transaction checks it before publication. Formation, origin and Color Post
publish in the producer's same scratch transaction with every other affected
projection. Commit followed by separate fact mutation is not a producer handoff.
Storage staging alone proves neither Ready, Pre, common commit nor joint Post.
Tests reject stale/foreign evidence and failure of any Pre/Post without
publication, and verify uniform cleanup gating and actual producer authority.

## 5. Instance and result obligations

Every selected compile call, including seal P2 and scalar/Product/external-type
results, has a stable instance before body entry. Parent, actual selected
receiver/call-entry pair and canonical inputs determine identity. Body/result
material and provenance do not. Conflicting bodies for equal keys are conflicts.

Read_name and initialized Read_resident of InvokeName(I) do not EnterBody.
Reacquisition reads current storage; old complete snapshots stay immutable.
Cache reuse cannot replay initialization, restore consumed residents or grant
saved authority. Writes, moves and invalidations keep their ordinary effects.

Reach includes accessible members, Products, captures, references and share
targets. Required unknown reachability is unavailable. An instance-open self-root
type must be the single direct complete type result. External types and ordinary
values are legal without acquiring that NameExpr. ReturnEvent and delivery remain
real boundaries even for equal self returns.

Only omission completes: open plus known OpenHere gives mut; otherwise const.
Required unknown OpenHere is unavailable. Explicit and deduced modes survive.
Delivery precedes outward Complete/Close; inputs keep their own subjects/lifetimes.

## 6. Structural, Path and name obligations

$ carries opposite-context syntax in both V and S, with arbitrary source
nesting, Hole identity and parity. It supplies no Read, Stage change, borrow,
mode, OpenHere or lifecycle fact. S type formation atomically publishes actual
Val2 witnesses, registered roles and a complete type.

struct preserves DirectPatternChild, ConstructEdge, ExtractEdge and FieldView
while adding ordinary helpers. * selects ordinary type composition; *= performs
checked read-transform-write without publishing partial state.

File hierarchy supplies inherited navigation only. let inner=bool:: under a
uses destination inner::a at the same NameExpr level as the RHS. Root and
filenames add no segment, type-composition action or authority.

Requested-name producers accept any declared ordinary result. Concrete heads,
wildcards and requested HoleRef use ordinary applicability/specificity.
ADL receiver calls and type-path progression share that relation:
foo.bar.baz has canonical Path baz::bar::foo. Double-dot contracts pipeline-dot.

Lexical === forms Path material once in the old environment before installation;
it caches no terminal binding and creates no entity or recursive alias thunk.
With items retain full Paths and forward references. Instantiation follows
complete declarations/Self/control-flow on actual layers; absence adds no
constraint. Cleanup points are globally fixed before same-point precedence and
reverse declaration order, then lifetime observation.

## 7. Continuing evaluator work

Closure completion forms full tau_C with distinct c_C, A_C and () roles at a
finite implementation leaf. Ordinary let binds the type; established same-name
buckets jointly form target-anchored members against one snapshot. Legal
ordinary actions retain their meaning without failed execution retry.

Dependencies separate requirements, selected ordinary realization and layout.
Realization is unique up to observational equivalence for one selected action.
Explicit/automatic occurrences retain identity; invocation does not recapture.
Dependency lifetime and escape remain ordinary checked handoffs.

Serial completion consumes path-sensitive UnitDiscard, unit fallthrough,
ReturnEvent and target inference. AST-list position is not semantic tailness.
R_vis/C_sigma, pending seal material, dependency readiness, source lifecycle,
current instance state and E saturation share one continuation.

Bootstrap families use source-expressible relations where possible. Host
capabilities return ordinary Objects. Planner/optimizer/layout/storage introduce
no private facts. Equivalent rewrites revalidate affected projections before
E saturates Ready work.

## 8. Acceptance evidence

Each phase runs cargo fmt --all, cargo check --all-targets, targeted tests and
git diff --check. Final integration runs cargo test and an active-tree
terminology/authority scan excluding history. Coverage includes positive/negative
behavior, identity/equality, non-derivability, authority uniqueness, no reopen,
effects and failure without publication. Executed coverage and remaining source
frontiers are reported separately from canonical laws.
