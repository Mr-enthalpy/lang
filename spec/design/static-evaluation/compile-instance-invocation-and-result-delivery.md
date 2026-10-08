# Compile Instance Invocation and Result Delivery

Status: canonical invocation authority. Invocation uses the ordinary Object,
Pattern, type, Place, Policy, dependency and lifecycle relations of one E.
The [unified owner](../unified-static-name-and-structural-semantics.md) fixes
instance identity, result shape and interpretation polarity.

## 1. Selected partner and stable identity

Every compile computation forms its instance before entering the selected body:

```text
CompileInstanceKey = ParentSemanticOwner
                  x SelectedCompilePartner
                  x CanonicalizeInvocationInputs(In)
I = CompileInstance(CompileInstanceKey)
n_I = InvokeName(I)
```

ParentSemanticOwner is the established semantic owner at this invocation.
SelectedCompilePartner retains the actual receiver/call-entry pair and sealed
projection/frame. Equal implementations with different receivers stay distinct.
Source spans, graph renderings, scheduler traces, body material and result
material are provenance or storage; they do not participate in instance identity.
Conflicting body material for the same complete key is a conflict, not another
instance. Parent-neutral material caches cannot determine semantic roots.

Canonical input observations retain ordinary values, complete snapshots where
required, observed Name/subject identities, reference targets and admitted
dependency closure. Equal Core never merges subjects or borrowed generations.
An authorized subject update preserves that subject identity; value-sensitive
keys still observe their actual input snapshots. Dependencies legally embodied
in the selected callable or an input retain their ordinary identity and
lifetime obligations. There is no hidden caller-environment coordinate.

Every CompileInstance is a stable semantic owner with an ordinary self-name.
Instance formation is independent of result kind and P2. In particular, a
selected compile invocation can form an instance while evaluating at seal P2.

## 2. Ordinary self-name and body entry

```text
Read_name(n_I) = NameValue(n_I)
Read_resident(n_I) = current complete resident in Sigma
OpenEvalReentry_K(I)
    iff ActiveEvaluation_K(I) and NextEvaluationEdge_K = EnterBody(I)
```

Read_name, Read_resident, Pattern observation and Policy observation are
ordinary observations. They do not enter or rerun the body. A self-name read
requires an initialized complete resident; incremental material is not an
Object. Each ordinary or in-place closure has its own lexical, Self,
navigation and dependency layer. Access to an enclosing instance uses its
ordinary name and admitted dependencies.

The instance has ordinary current storage. Reacquisition after a legal write,
move or invalidation observes that storage; it does not replay initialization,
restore the first resident or resurrect consumed material. Complete snapshots
copied earlier remain immutable. Cache retention extends no dependency region
and supplies no cached permission to read, write or enter an active body.

## 3. Result classes and accessible result closure

The semantic call entry declares independent result planes:

```text
ReturnDeclaration(F) = DeclaredResultClass(F) x ReturnPattern(F) x ResultPolicy(F)
InvocationResult = SemanticResult | Residual | Diagnostic
```

Ordinary scalar, Product, callable/closure, external type and self-root type
results use the same invocation and result delivery. Name-producing action and
NameExpr result shape are independent.

```text
Reach(v) = ordinary accessible closure of v
SelfRootSet_I(v) = {tau in Reach(v)
                   | Type(tau)=type and Root(tau)=I and OpeningSource(tau)=I}

SelfRootSet_I(v) != {}
    => Type(v)=type and v=tau_I and Root(v)=I and SelfRootSet_I(v)={v}
```

Reach includes actual Product elements, owned members, captures/dependencies,
ref/share targets and other admitted accessible wrappers. Missing reachability
facts are unavailable, not evidence for an empty set. The test precedes outward
publication: hiding an instance-open type in an aggregate or closure fails
without installing a result. Multiple reachable instance-open types also fail.

A legal single direct self-root result supplies:

```text
Read_name(result) = n_I
Read_resident(n_I) = tau_I
```

It is a computed NameExpr. An existing external type keeps its own identity;
returning it does not manufacture a NameExpr at n_I. Invocation ownership
creates no structural child of an input and does not change input Val2 or Norm.
A separate requested-name action may realize the delivered value at its
requested coordinate under ordinary name rules.

## 4. Opening sources and completion

Let AccessClosure_out(In) be the actual semantic dependency closure of result
construction, including identity observations, captures and borrow targets.
It is not merely the list of syntactic arguments or an optimizer trace.

```text
o(out) = meet {o(x) | x in AccessClosure_out(In)}
all contributing sources Closed => inherited output source Closed
```

Comparable active sources use the most restricted one; incomparable sources
retain conjunctive requirements. An empty dependency closure supplies no
inherited authority. Fresh formation uses its independently authorized window.
Unused arguments with no result dependency add no output opening constraint.
An identity-relevant dependency remains even when absent from public fields.

Admitted edges retain the original subject/anchor and current window state.
They grant no authority over unrelated enclosing names. OpenHere is evaluated
at the current continuation; it is not a cached boolean.

```text
OpenPolicy = {open, close}
PolicyMode = {const, mut}
DefaultMode_K(x) = (OpenPolicy_K(x)=open and OpenHere_K(x)) ? mut : const
```

Only omission admits completion. Explicit const/mut and Pattern-deduced modes
remain unchanged. Required unknown OpenHere is unavailable. Open retains an
established opening source; close completes and closes after successful ordinary
result delivery. Neither closes an external input or borrowed target. Close
irreversibly ends the relevant construction window.

The self-return path has its ordinary boundaries:

```text
Read(self) -> ReturnEvent -> ordinary result assignment/replacement
          -> Complete -> Close                    // outward close policy
```

Equal resident values do not remove ReturnEvent or assignment. A transfer to a
separate outer destination follows producer delivery; its failure does not undo
an already committed producer Close except through an established enclosing
transaction. Failure before commit publishes no partial Object or Close.

## 5. Lifetime, references and retained state

OpenHere governs mutation qualification, not lifetime. Stable instance identity
does not imply global lifetime or a body-local region. Owned result transfer
obeys ordinary escape rules at the destination. Global publication requires
globally survivable dependencies and closed registered structure. Non-owned
ref/share/rebind targets are never promoted merely by reachability.

Every borrowed reference retains its actual Place, original borrowed generation
and opening subject. Direct mut borrowing and explicit open-ref-to-mut
confirmation require current OpenHere, target Writable and ordinary capability,
access, type and lifetime evidence. Initial typed-slot borrowing remains a
separate one-shot initialization authority. Close defeats saved writes; carrier
replacement does not retarget an existing reference.

For two ordinary outer bindings that read the same invocation snapshot:

```text
q_a != q_b
q_a != BindingPlace(n_I)
q_b != BindingPlace(n_I)
```

Writes to copies do not mutate retained instance storage. Updating its state
requires selecting and borrowing the actual instance/member Place. Cache hits
recheck current dependencies and all operation Pre facts. Terminal transport is
Move; clone-derived material comes from selected ordinary share/rebind, clone
and fresh complete result formation before terminal Move.

## 6. P2, Ready and evaluation

```text
Stage = {compile, seal, runtime}
Visible != Ready != LegalToExecute
```

CompileInstance formation and OpenPolicy are independent of these atoms.
Seal work follows the ordinary seal horizon, fixed Wpre/Wseal domains and
dependency readiness. A compile helper or cache cannot fabricate Ready or
read unavailable material. Runtime residue preserves selected names, receivers,
call entries, projection and frame; it does not re-resolve or reselect.

Main uses runtime P2. Stable roots arise through independently legal ordinary
formation and do not impose an ambient active evaluation frame around entry.
The [evaluation owner](evaluation-residual-and-optimization.md) owns saturation,
common transactions, projection synchronization and residual provenance.

## 7. Requested names and ordinary associated state

```text
q = RequestedName
I = CompileInstance(parent, selected producer, CanonicalInputs(q, In))
E_I(rhs) => v
Realize(q, v)
```

Realization accepts the ordinary declared result. It imposes neither Type(v)=type
nor NameExpr(v). Generated residency alone supplies no V_tau or structural-role
registration. Frozen generative families may realize later ordinary members
without reopening registered structure or altering retained snapshots.

Associated state A, trait-like queries and automatically generated laws are
ordinary compile programs using the same instance registry, self-name, open/close,
type/reference/operator and name algebra. Customization writes actual retained
Places under ordinary Pre. Consumers read committed current state or their
declared snapshots; subsequent writes do not revise a committed decision.

The closed-type singleton-Val2 helper counts actual finite Val2 at its evaluation
position and reads its sole member through ordinary access/navigation. Zero or
multiple members fail. V_tau, candidate visibility and hypothetical generated
members do not supply the count. A later member realization may change a later
count while preserving earlier complete observations.

## 8. Transaction and consumer handoff

Instance admission, body entry, result delivery and completion retain distinct
semantic actions on one continuation. Each producer uses common Pre, commit
and joint Post; scratch publication includes formation, origin and Color.
Outstanding cleanup blocks every action crossing its fixed point. Storage
staging alone is not a semantic producer witness.

Remaining-continuation facts alone authorize Residual. Unsupported source or
missing facts report the actual unavailable boundary, without reopening the
selected family or fabricating success. Persistent key/storage representation
and checked access summaries remain implementation work.
