# Effect steps and concrete continuation handoff

Status: canonical owner of Step, Wait, transparent propagation, handler and
throw control. Source consumers are pending. This owner uses the existing E/K,
ordinary calls, Pattern relation and lifecycle transaction; no independent
effect checker, Future type or async evaluator is introduced.

## 1. One actual step

```text
Gamma; K |- Step(selected f, CallInputs) => Done(v) || Wait(y,k)
```

Done(v) and Wait(y,k) are ordinary effect-result sum branches, matched through
R_Gamma. Done here is distinct from internal chain/target completion
Done_chi/Done_Return, which is never an Object or Pattern. Step payloads cross
the existing InvocationResult envelope; Step adds no independent result envelope
or evaluator. Normal payload v has its established return delivery. Wait contains the yielded
material and the next **concrete ordinary callable** k. Different cuts may
produce different complete types. Its new self has a different name identity
from the original self; self/captures/resume arguments still use the same call
and lifecycle rules. Resume is an ordinary new call with matching input, selected
at its actual frontier. A previous selected call never reopens.

The canonical proposed surfaces are:

```lang
(yield_val, resume_pattern) wait @{repair}
(yield_val) |> wait (resume_pattern)@{repair}
```

These two wait-specific surfaces have the same Step/MoveRepair contract. The
resume Pattern introduces actual resumed parameter names. The postfix repair
attachment covers the complete preceding expression and binds more weakly than
ordinary full E; grouping delimits its extent. Omission selects id. Raw/Norm
must preserve these roles/spans without evaluating them. Grammar and semantic
execution are separately pending in the roadmap.

## 2. Handoff transaction

Inputs are the actual selected action/activation/cut, yielded value, live
continuation material, resume Pattern, ordinary dependencies and user repair.
The [lifecycle owner](../lifetime/lifetime-policy-and-overload-boundary.md#218-empty-slices-inherited-autopre-and-atomic-move-repair)
owns Valid, the id law Pre(id)=Post(id), empty-slice continuity and Move[rho].

The live captured material delivered across the cut is ordinary continuation
output material. Its call-context projection need not be an owned field subtree
of the new self Object. Physical environment fields can represent it only while
preserving the actual semantic source/ownership relations. Formally:

```text
LiveContext_s -- Move[rho] --> CallContext_k
mu_s : transferred live name/generation occurrences -> successor occurrences
```

LiveContext_s includes the actual held callee/caller suffix material selected
for transfer. CallContext_k observes that material in the complete successor's
ordinary call/dependency context; it is not an additional Wait payload, Object
axis or hidden registry. Unfulfilled RequiredCapture and resume requirements
remain specifications, not values bound early by this projection. Resume's new
argument names are introduced only by its ordinary invocation.

The generated lifecycle interface of this held material uses that same id law
as InPlace automatic-capture AutoPre. Let H_k be the legally mapped observation
of the actual successful Move/repair Post on that material. Its zero-action
delivery interval has Pre(id_H_k)=Post(id_H_k)=H_k, and the held-material part of
AutoPre(k) is H_k. No fresh entry defaults reset these existing occurrences or
their cleanup duties. This is equality in the successor's name/generation
vocabulary, not equality of the original and new self Objects or a guarantee
that the resumed body preserves state. Default repair id likewise preserves
its own input state; the composite Move[id] must still prove every ordinary
Move and destination Valid premise and may change the source state.

Three obligations are checked separately before successful handoff publication:

- **Name correspondence:** mu_s accounts for every transferred occurrence and
  its validated post-repair name/generation, origin, region, Color and access.
  Distinct occurrences cannot merge by spelling/value equality; ref handles
  retain their real target identity, including legal shared targets.
- **Ownership transfer:** each actual owned occurrence has one lawful custody
  disposition under the selected MoveEffect/repair. There is no extra self/capture
  Move, cloned owner or referent transfer inferred from a handle or layout field.
- **Cleanup responsibility:** every transferred outstanding destruction duty is
  represented by the successor's ordinary held material/contract exactly once.
  Dropping it follows actual ownership and fixed cleanup order, without duplicate
  source-generation Drop, stranded caller resources or replayed committed work.

A proved name map alone supplies neither ownership nor cleanup evidence. Failed
or unavailable evidence for any obligation exposes no partial successor or
successful Post. All three consume the existing common transaction/lifecycle
rules; none adds a separate continuation type system or changes Wait(y,k).

```text
caller Pre.check -> callee Pre.admit
  -> selected Move[rho] in common scratch state
  -> commit complete successor k and yielded material
  -> callee Post.check -> caller Post.admit -> Wait(y,k)
```

The complete four-way protocol is required; the diagram's commit is not
permission to publish before joint Post validation. Failure publishes no
success at this boundary, creates no visible illegal successor and selects no
runner-up. Unavailable name/access/repair/contract evidence gives a spanned
unavailable diagnostic, or retains only an actually established remaining K.
No synthetic residual can stand in for unsupported syntax or a failed Pre.

The suspending/forwarding side proves Valid after actual Move/repair and legal
renaming to k. The resume side sees this validated AutoPre. User code may change
construction or use a trusted facility with a genuine successful Post when id
does not suffice. Neither automatic boxing nor an invented repair is a fallback.

## 3. Transparent propagation and ordinary handlers

```lang
let val = expression;
expression@{ repair };
expression? |> (pattern1) { ... }
            |> (pattern2) { ... };
```

An ordinary value consumer receives v on Done(v). An unhandled actual Wait
propagates its effect and the remaining concrete computation. The
[whole-original-K CPS obligation](../static-evaluation/evaluation-residual-and-optimization.md#71-whole-original-continuation-cps-conversion-consumer-pending)
requires that this successor compose the callee remainder with every pending
caller suffix up to the applicable handler/delivery boundary. Returning only a
callee continuation while abandoning its caller's bind, remaining input, return
delivery or cleanup is not propagation. This common-E consumer is pending;
neither Step notation nor local wait splitting implements it. `?` exposes the
effect-result material for explicit ordinary branch handling; handlers may
resume, forward, or drop k. Coverage is over effect and parameter Patterns.
An exhaustive handler has no extra obligation to forward its own continuation.
An unhandled propagating branch checks its actual handoff repair only when that
branch occurs. Unselected branches execute no lifecycle or resource actions.
Earlier committed effects are retained, not rolled back by handling.

Propagation can alter a concrete closure's call shape. It cannot silently
broaden an established Function Pattern interface. To combine different effect
shapes, a user wrapper handles them into the intended interface before ordinary
FnExtract. The [Pattern owner](../patterns-overload/pattern-relations-observation-and-extraction.md#113-regular-effect-function-patterns)
owns the regular shape and source-erasing interface relation. A wrapper retains
its real dependencies and has no implicit storage allocation requirement.

Effect propagation is a normal call-result protocol across ordinary closures.
It is independent of targeted ReturnEvent, whose Ordinary barrier and actual
activation/target restrictions remain unchanged.

## 4. Throw and cleanup

Postfix `yield_val throw` denotes this control expansion (do/recursive call
consumers remain pending):

```lang
(do (self) {
    ((yield_val) |> wait ());
    () |> self
}) return
```

The yielded value occurrence is retained according to ordinary ownership and
Move rules; repeated delivery requires an explicitly legal producer such as
`value copy throw`. The spelling grants no implicit copy or reusable ownership.
The delivered continuation remains present. Calling it produces the relevant
effect again, rather than resuming the original normal-return path.

```lang
let _ = coro;
```

When a handler discards the continuation, its concrete type's ordinary Drop
protocol cleans held computation material, including complete extracted children.
Drop placement/order, access, Color and destructor effects follow the lifetime
owner. Transparent propagation only transfers control and cleanup responsibility
through legal Move. There is no separate exception unwind over the call chain,
and no deletion of resume material from the exception result. Linear consumption
and ordinary destructor rules prevent omission or duplicate Kill/Drop.

## 5. Forced PatternMiss and compile boundaries

Only a **selected forced extraction actually executed** and finally failing its
ordinary relation (including its one authorized root conversion if applicable)
enters a PatternMiss throw handoff. Applicability mismatch, ordinary branch miss,
syntax errors, malformed Function Patterns, missing consumers and unproved Pre
remain their own rejection/unavailable rules. They do not manufacture a catchable
effect. A PatternMiss handler cannot reopen the sealed forced consumer.

```text
SelectedForced(F) and Execute(F,v) = FinalMismatch(proof)
  => Step = Wait(PatternMiss(F,v), k_throw)
     subject to the same legal handoff premises
```

The failure creates no successful extraction binding/Post. Earlier legal effects
remain. An ordinary handler can discard k_throw and map the declared Miss branch
to Result, Option, bool or another sum. This protocol applies at compile horizon
too: successful execution creates the names/values needed by subsequent work in
that same continuation. Validation need not be executed twice in require and in
the body. An unhandled compile effect reaching the external compilation boundary
becomes a spanned diagnostic identifying the effect and boundary. Handling only
claims the effects whose argument Patterns it actually covers.

## 6. Stage and proof obligations

Effects carry no implicit runtime Stage. Heap/IO acquisition and calls are ordinary
capability operations; libraries may optionally express control inversion via wait.
Compile/runtime readiness and InputAdmissible remain independent. The
[unified owner](../unified-static-name-and-structural-semantics.md#20-reify-callable-p2-and-scoped-reverse-migration)
owns yield-versus-return self-root checks and compile-determined public reify
interface shape. Yield is never a formal return classification shortcut.

Every connected consumer must demonstrate single selected origin, correct actual
input mapping, atomic valid handoff, exact ordinary cleanup, conservative regular
summary and unchanged target-return barriers. The conformance matrix's C/L/A/E/S
rows distinguish these normative requirements from executable coverage. O may
choose a different generation grain only under the E owner's observation proof;
the existence of a cut alone creates no physical Move or object-generation duty.
