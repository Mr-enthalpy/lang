# Meta Invocation and Policy Reduction

Status: Current canonical design

Meta evaluation is ordinary semantic evaluation at a meta stage. It shares the
Object, Pattern, complete type, OverloadGroup, Place, Policy, invocation, and
lifecycle relations with compile and runtime evaluation. This document owns
invocation-generated result names, their identity and dependency propagation,
and the semantic requirements on meta invocation caching. Construction consumes
these laws; associated state A is an instance of them.

## 1. Invocation boundary

The declarations P let f = (self,args):meta => B and
P let (self,args) f => B are equal surface projections of the same declaration.
The latter keeps ordinary extraction in its head and constrains the requested
NameCoord through f (or leaves the selector unconstrained through _). The
[operator/declaration owner](../patterns-overload/operator-patterns-and-generative-declarations.md)
owns this shared material; neither surface changes invocation identity, the
direct CompleteType result boundary or construction authority.

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
MetaInvoke(Name_callee, In) -> Name_out

MetaInstanceRootKey
  = parent SemanticOwner
    × selected callable identity
    × CanonicalizeInvocationInputs(In)

M = MetaInstanceRoot(MetaInstanceRootKey)
n = InvokeName(M)
BindingPlace(n) = q_M
```

For ordinary meta, the constructed instance name denotes the instance type
itself. It cannot be assigned an arbitrary direct result type:

```text
DeclaredResultClass(F) = CompleteType
Value(n) = tau_M
Root(Core(tau_M)) = M
```

The instance name and its type value are two observations of the same entity;
there is no independently replaceable payload at this root. NameBinding remains
an identity/Place relation, not a wrapper Object. InvocationResult carries this
complete type. Arbitrarily typed values, including borrows and external types,
can instead inhabit its ordinary Val2 and be read by `member::(In |> F)`.
That navigation does not change the meta invocation's direct result class.

The parent is the callee's resolved semantic placement, not the caller's current
file or stack-top frame. Equal full invocation keys denote the same result
name and Place. Different parent owners or selected callable identities remain
distinct. Parent-neutral execution material may use a smaller cache key, but
that key cannot determine or merge semantic names.

Input normalization preserves the observations required by the selected
invocation. Ordinary value inputs use their rank-appropriate canonical value
observation; reference inputs retain target identity and validity dependencies.
When construction depends on an input name or construction subject, that
dependency's stable semantic identity is retained as well. It is not recovered
from equal values, source spelling, a carrier address, or TypeValueId. A value
copy preserves an existing construction subject; independent equal-Core
formation does not merge subjects. Authorized updates and Close preserve the
subject's identity while changing its state.

Consequently a name-dependent invocation is not keyed solely by the changing
resident snapshot, and a value-dependent invocation is not silently made
insensitive to its value inputs. These are observations in one invocation
normalization relation, not separate call mechanisms. Input identity and the
current legality of using that input remain independent.

Every meta-instance root is a stable semantic owner. Its retained instance
uses the P1 `meta` policy described in §3: OpenHere governs acquisition of its
mutable view. There is no independent root const/mut gate. A carrier's internal
plain marker is not the semantic policy of the instance. Stable identity does
not imply global lifetime, current visibility, or an open construction window.

### 2.0.1 Invocation ownership and structural ownership

Construction and extraction may be isomorphic; a call forwards to construction
and does not make its call history an extraction Pattern. The constructed
result name supplies the ordinary extraction/value interface.

```text
FormationOwner(n) = invocation M
n notin StructuralChildren(input)
forming or reacquiring n leaves Val2(input) and Norm(input) unchanged
Close(input) => stable non-generative registered structure
```

A structural name such as `child::t` instead occupies t's structural namespace
and its resident participates in t's owned Val2 observation. Structural
namespace membership and registered DirectPatternChild evidence remain
distinct. Invocation ownership establishes neither relation to an input.
The name's own resident can have ordinary structure and members under their
ordinary construction rules. This statement concerns acquiring the invocation
result itself. A separate generative name realization may establish an ordinary
Val2 member at its requested coordinate, including on a closed type; it supplies
neither V_tau nor Pattern registration and does not reopen construction authority.
See [generated Val2](../symbol-world/names-and-overload-groups.md#71-generated-val2-after-registered-structure-is-closed).

### 2.0.2 Open inputs and output dependency closure

An ordinary meta invocation may consume open input names and produce an open
result name. Its input dependencies must have stable identities and be valid
for the actual invocation and result uses. GlobalKeyable/GlobalSurvivable are
the stronger conditions required for global persistence, not blanket input
admission conditions. A local dependency cannot be promoted merely by entering
an invocation key.

Let `AccessClosure_out(In)` be the actual semantic input dependency closure
of result construction: names retained, accessed, or depended on by the
result, including identity dependencies and transitive borrow/capture edges.
It is not the list of syntactic arguments. An unused syntactic input imposes
no output openness constraint when it contributes no such dependency; an
identity-relevant input cannot be omitted just because it is absent from the
resident's fields. Dependencies are semantic derivations or checked summaries,
not optimizer guesses from a particular execution trace.

Write `o(x)` for x's openness source, and `o1 <= o2` when o1 permits opening
no farther than o2. `Closed` is the least source. This is mathematical notation
for existing authority/window facts, not another language type or Object axis.

```text
o(out) = meet { o(x) | x in AccessClosure_out(In) }

for a nonempty chain of comparable active-stack sources:
o(out) = min { o(x) | x in AccessClosure_out(In) }

all contributing sources Closed => o(out) = Closed
```

The order describes admissible opening extent, not numeric stack depth.
Incomparable sources retain their conjunctive requirements; an implementation
must not choose one source arbitrarily. An empty input-dependency closure
supplies no inherited opening authority: fresh local construction uses its own
ordinary window and its outward result is closed unless an independently
authorized source is established by the ordinary construction rules.

`OpenHere_Sigma(out)` interprets this source at the current continuation and
authority context. It is not a cached boolean. True Close irreversibly ends
the relevant window; temporary masking does not. Result completion preserves
inherited sources without extending them. A fresh body-local window cannot
escape its owning invocation merely because a result name is stable.

Entering meta transports only the admitted input dependencies and their existing
access/authority relations. Those edges permit the callee to consult the
original source frames; they do not reparent inputs, create windows, or expose
unpassed outer names. Unadmitted outer material remains masked. An operation
still rechecks the transported source's current authority, window, policy,
capability, and lifetime; an input edge is not a write grant.

For the instance's P1 `meta` policy, openness is the governing qualification:

```text
MetaMutationQualification(n, Sigma) iff OpenHere(n, Sigma)
AcquireMutView(n, Sigma) requires OpenHere(n, Sigma)
```

There is no additional independent const/mut permission on that instance. The
selected operation must still exist and satisfy its ordinary typing, actual
Place, access and lifetime premises. Every write Pre rechecks OpenHere; a saved
mut view is not enduring authority. An ordinary Val2 payload remains an ordinary
value/name with its own policy. A borrow stored there retains its original
target and that target's rules; containing it never transfers write authority.

### 2.0.3 Result completion, lifetime, and cache reuse

Ordinary meta constructs tau_M under M. P1 `meta` preserves its admitted
input-derived opening source through completion; classic `plain let` completes
and closes this instance (§3). Completion and Close are distinct events in the
retained form. Neither form closes external inputs or borrow targets merely
because they occur in Val2.

For fresh result-owned construction material, the derived source also governs
its existing construction subject, including Core(tau_M). Its identity remains rooted at M while authority is checked through
the retained sources; M's body frame need not remain active. This qualification
is established as part of result construction/completion, before any local
window ends. It never revives a closed subject or transfers an expired local.
External Val2 payloads retain their own construction subjects; storing an
external tau in a member does not replace its opening facts with those of M.

Owned result material may acquire the result's admitted region through ordinary
ownership transfer. Global promotion requires globally survivable dependencies;
ref/share/rebind targets remain non-owned dependencies and are never promoted
by reachability. Body locals not validly transferred expire normally. Retaining
a reference to such an expired local in the result fails even if invocation
identity can be retained. Self-root applies to the direct instance type, not
to the value owner of each payload or borrowed target.

Meta invocation facilities maintain the stable association:

```text
MetaInstanceRootKey -> instance type/name / member Places / construction status
Read(InvokeName(M), Sigma) = current complete tau_M snapshot in Sigma
```

Repeated acquisition of a completed invocation returns the same name; it does
not initialize it again, replay its construction effects, or restore the first
type or member snapshot. Ordinary member writes, moves, and invalidations
retain their meaning. An unavailable/consumed payload is not resurrected by
cache lookup. Previously copied complete types keep their immutable snapshots;
cache lookup is current name observation, not mutation of those copied values.
An active construction cannot be read as a completed result; symbolic references
to its root remain distinct from forbidden active evaluation reentry. Failure
does not publish a partially initialized result.

Value/material caches may coexist, but their reuse must preserve current reads,
effects, dependency validity and Pre checks. Cache replay cannot supply a stale
OpenHere/Writable proof. Common-snapshot sibling overlays and unordered conflict
rules apply to these Places like other ordinary state. Persistent identity
storage cannot extend a dependency's lifetime. This general facility supports
indexed mutable state; no separate GlobalMap primitive is required.

### 2.0.4 Worked example: instance acquisition and outer binding

```lang
meta let a = t |> A;
meta let b = t |> A;
```

With the same full invocation key, both RHS evaluations acquire the same
InvokeName(M) and its actual instance Place q_M. Value observation reads its
current complete tau_M snapshot. Outer binding then uses ordinary destination
formation and mechanical transfer. P1 meta retains the admitted opening
qualification; it does not turn `=` into lexical `===`, invent a borrow, or
merge destination Places.

For ordinary snapshot copies into the two destinations:

    q_a != q_b, and neither destination is q_M
    at each read: tau_read = Read(InvokeName(M), Sigma_at_read)
    copy retains type/root and construction-subject facts
    copy does not turn the snapshot into a current-state lookup

Equal snapshots therefore do not imply shared destination writes. A write to
a's ordinary copied resident need not appear in b or q_M. To update the retained
instance state, select its actual member Place through the invocation-generated
name and form the explicit permitted reference to that target. Copying an
already formed reference preserves its target under ordinary reference rules;
it is different from copying the containing type snapshot. Cache reacquisition
observes writes to the instance/member Places, not arbitrary writes to copies.

This derivation does not choose automatic copy versus move: the existing
mechanical pass relation owns that choice. Observing the type result is not
an implicit Move of the stored instance resident out of q_M. Explicit moves
and payload invalidations keep their ordinary checked effects.

If a later `plain let c = t |> A` successfully completes the producer with
Close, the shared instance construction subject closes before the outer
destination transfer. References still targeting its state then fail the
opening-source check on write; external t itself is not closed by that event.
A subsequent destination-transfer failure does not by itself undo an already
committed producer Close. Only an existing enclosing transaction can provide
rollback. Failure before Close commits leaves that action uncommitted. This
uses ordinary producer-before-transfer and Pre/commit rules, not a let-specific
transaction or a cached permission.

### 2.1 Compilation entry and root formation

Entry has runtime horizon: main.P2=runtime, and an omitted ordinary main.P1
stage completes to runtime. The normalized physical tree supplies ordinary
source actions to EntryContinuation(L,F_main), as specified by the
[evaluation owner](evaluation-residual-and-optimization.md#1-compilation-entry).

Stable roots come from the existing bootstrap or an independently legal
ordinary meta formation. Root identity, root construction authority, active
MetaDom and main's P2 are separate facts. There is no global active meta frame
wrapping all compilation. Each root/name/ref/write action must have its own
ordinary authority, OpenHere, Writable, access and lifetime evidence.

An actual meta invocation establishes its ordinary instance tau_M and active
frame, and imposes MetaDom while that frame is active. Returning restores the
enclosing context; a stable root or completed meta payload retains no active
dominance. Seal therefore cannot be globally forbidden merely because a root
was formed by meta. Conversely, invoking meta through a compile helper or cache
inside SealDom remains forbidden.

### 2.2 Generic symbolic anchor and compile realization

MetaPartner(F)=M(F) is the generic symbolic anchor. Compile realization is the
independent family C(F)={C_sigma(F)} admitted by the Policy projection relation.
A runtime generic can have both this family and M(F); a compile generic uses
its ordinary compile evaluation and may have M(F); an ordinary meta callable
does not acquire a second meta partner.

Projection members retain the source invocation and sealed frame, while M(F)
has its separately anchored identity. Lazy representation is permitted, but a
single cached C(F) record is not the entire semantic family. Non-generic
callables retain their CallableRoot. Neither partner construction relabels a
runtime binding's Pout.

## 3. Policy positions

P1 and P2 are independent; Pin and Pout are derived:

    Pin = ElabIn(P2, Delta_in)
    Pout = ElabOut(P1, Delta_out)
    bare let -> empty overlay
    written plain/const/mut -> explicit mode override
    formal-local <p> p let -> ordinary Pattern solution for Mode=p

Input and output constraints belong to one invocation relation; neither policy
side computes the other. Pin admits explicit stage constraints and stage holes; Pout.stage remains
P1.stage. A missing dimension is not an explicit hole or wildcard. Neither position policy grants Writable or changes the
caller's independent result demand.

Call-site `ResultPolicyDemand` is formed independently before maxima and
participates in the ordinary invocation pipeline.

For ordinary names, binding policy and the resident value's OpenHere are
independent because name and value are distinct. The meta instance identifies
its name with its own type value. Here OpenHere dominates the weaker const/mut
qualification; retaining two independent mutation gates would misdescribe the
entity. P1 therefore admits a `meta` marker:

```lang
meta let f = expression;
plain let g = expression;
```

When expression constructs an ordinary meta instance, `meta let` retains that
instance with its dependency-bounded openness. OpenHere must hold before a mut
view can be acquired, and again at write Pre. `plain let` retains the classic
complete-and-close invocation: after completion its instance cannot acquire a
mut view. Bare let supplies no override; plain behavior requires a separate applicable
DefaultModeCompletion when no inherited/contextual constraint supplies one. Neither form extends the
input window, and reacquiring a closed instance with `meta let` cannot reopen it.

P1 `meta` describes the instance policy; P2 `meta` describes the callable's
evaluation horizon. Their positions are distinct. P1 `meta` is not an alias for
ordinary `mut`, a fourth point in its 3×3 capability table, or permission to
execute arbitrary runtime expressions at meta stage. The
[policy owner](../symbol-world/symbol-policy-and-compile-flow-projection.md#3-contextual-elaboration-of-p1)
owns its contextual elaboration, whose narrow openness domain also includes
ordinary `type` and `type ref`. That qualification preserves an existing
opening subject; it cannot create an invocation identity. Ordinary meta still
directly constructs its own CompleteType instance, not an arbitrary payload
or an external type resident.

Explicit instance borrowing follows the same two coherent routes as other
initialized type names: direct mut type ref, or meta type ref followed by
explicit ConfirmMut. Both require independent Place Writable/capability,
access and lifetime as well as current OpenHere. Meta refs retain their
original borrowed generation, not a proof or a lookup of any later resident.
Close disables both mutable routes and saved-ref writes. The initial ref of
an uninitialized name remains a separate one-shot initialization capability.

Value observation remains the default. Explicit type-Place borrowing is distinct
from `t ref` forming a borrow type. For ordinary Val2 members, explicit borrowing,
const/plain/mut demand, same-Type migration and mechanical passing continue to
use the ordinary rules. These observations neither change the instance root nor
turn the meta call into a direct borrow producer. Selected failure never reopens
selection or a closed construction window.

### 3.1 Ordinary Val2 extraction and compile convenience

A type can hold arbitrary ordinary values in Val2. V_tau registers ordinary
callable values for the type's own callability without requiring or granting
val::path navigation to those values. Pattern construction/extraction
registration is separate again. Their precise separation is owned by
[type values](../symbol-world/type-values-places-and-borrow-views.md#22-complete-type-values-are-closed-snapshots-over-object-cores).
No callability or Pattern registration is needed merely to store a payload.

```text
instance = In |> F                 -- direct result: tau_M
payload::instance                  -- ordinary member name
Value(payload::instance)           -- arbitrary ordinary value
```

A compile callable can provide a convenience on a closed type t: inspect its
ordinary Val2; if exactly one entry exists, obtain that entry's value through
ordinary navigation and value observation. Zero or multiple entries take the
compile-error path; the concrete error representation remains owned by compile
error semantics. The count is over actual Val2, not V_tau, Pattern-registered
roles, candidates, or a visibility-filtered subset. Ordinary access checks still
apply; the helper does not disclose private members.

Close fixes non-generative registrations, not the number of every future ordinary
generated member. This helper counts the actual finite Val2 snapshot at its
evaluation position, without invoking every possible name rule. A later ordinary
realization may give a different current count; it neither changes an earlier
value snapshot nor makes its count a timeless optimization premise.

The input must actually be closed, not merely temporarily non-OpenHere. The
helper neither closes it implicitly nor selects an arbitrary entry. It performs
no implicit borrowing, registration, conversion, or projection of every meta
call. Using schematic `only_val2` (public spelling not fixed):

```lang
instance |> only_val2
instance only_val2
```

Both use the existing call skeleton. The helper is an ordinary compile callable,
whose result can have the member's type; it is not another meta result class.
A builtin may bootstrap this callable until source-level enumeration and compile
error expression suffice to implement the same definition. This introduces no
new navigation or invocation algebra.

## 4. Construction and installation

Meta construction separates:

```text
body execution material
  -> invocation instance name/type tau_M with ordinary Val2 payloads
  -> InvocationResult
  -> ordinary value / explicit borrow / policy observation
  -> any explicit outer binding under ordinary commit rules
```

Closure construction uses the current stable meta-instance owner. Injecting the
resulting closure into a type under construction is a separate operation.
NameBinding is a binding identity/Place relation, not a returned Object or
a borrowable wrapper. A complete type retains its own Core and callspace;
neither a result Place nor a defining binding supplies an additional callspace.
Structural creation yields a typed NameExpr with an uninitialized Place.
Explicit ref borrows that Place without reading; ordinary write initializes it.
No type resident or mutable reference is produced by name creation itself.

No outer lexical name or structural input child is installed by merely acquiring
the invocation result name. The direct type is rooted at M; an external type stored as a Val2 payload
keeps its own Core and captured callspace.

## 5. Partial evaluation

Partial evaluation may return `Residual` only through the shared result
boundary. It does not define a meta-specific result universe or a second value
ontology. Unsupported forms remain explicit residuals or diagnostics until the
shared evaluator and continuation consumers are connected.


The shared continuation, saturation and projection laws are owned by
[evaluation and optimization](evaluation-residual-and-optimization.md).
Host acquisition returns ordinary Objects under
[host capabilities](host-capabilities-and-machine-objects.md).

## 6. Conformance cases and consumer handoff

These are normative semantic cases for future consumers, not claims of current
source execution support. Surface borrowing still follows the ordinary
type-forming versus borrow-forming distinction.

| Case | Required result |
| --- | --- |
| Repeat one completed full invocation key | Same result binding and Place; read the current resident |
| Same callable/input material under distinct parent owners | Distinct invocation identities |
| Equal-Core inputs with distinct observed construction subjects | Distinct name-dependent keys; no shared window |
| Copy an input preserving its observed subject | Same subject-dependent result name |
| Update a subject without changing its identity | Preserve the name-dependent association; ordinary value-snapshot keys remain content-sensitive |
| Merely acquire an invocation result after input closure | No new input child, Val2 entry or Norm change from that acquisition |
| Realize a generated member on a closed type | Ordinary Val2 result only; registered Pattern/V_tau structure stays fixed and no construction window reopens |
| Read a group-valued Val2 member without ref | Ordinary group value observation; the meta call itself returns tau_M |
| Explicitly borrow a result name | Ordinary actual-Place/capability/lifetime checks; no implicit borrowing |
| P1 meta while OpenHere holds | Retain the instance; derive mut qualification from OpenHere |
| Classic plain let of a meta invocation | Complete and close its instance; later meta let cannot reopen it |
| Ordinary member demand or same-Type migration | Ordinary selected result and destination rules |
| Selected migration, borrow or write fails | Diagnostic without producer reselection or runner-up execution |
| Fresh result-owned type with a valid open input source | Result construction subject retains the derived source after body exit; no dead frame or local revival is required |
| Two contributing comparable opening sources | Result receives their meet, the more restricted source |
| A contributing source is Closed | Closed inherited output source |
| An unused syntactic input has no output dependency | It adds no output opening constraint |
| A dependency affects result identity but no payload field | Keep it in the dependency closure |
| Return an external type or non-type directly from ordinary meta | Reject; the direct result must be tau_M rooted at M |
| Store an external type or valid outer borrow in Val2 | Preserve its Core/root or target/region; grant no new authority |
| Store a borrow of an untransferred expiring local in result Val2 | Reject escape; cache identity cannot prolong the target |
| Closed type has exactly one Val2 entry | Compile helper reads that value, subject to ordinary access |
| Zero or multiple Val2 entries, or open input | Singleton extraction fails; no filtered count or implicit closing |
| A Val2 member has no callability or Pattern registration | Storage/navigation remain legal; neither registration is inferred |
| Save a mut reference, then close its opening source | Later write Pre fails even after cache/name reacquisition |
| Replace the original input carrier | Saved result references retain the original dependency and Place |
| Reacquire after a legal member write, move or invalidation | Observe current ordinary state; never restore initialization |
| Re-enter an active result construction as a completed value | Reject active evaluation reentry; symbolic root references stay distinct |
| Conflicting sibling result-place replacements | Ordinary unordered conflict; cache scheduling cannot pick a winner |

Implement the general input identity, result-name residency and opening-source
relations first. A then uses these facilities as an ordinary instance. Concrete
IR, persistence and access-summary algorithms are representation work; missing
consumers remain explicit Diagnostic/Residual boundaries.

## 7. Ordinary meta defaults and current-state consumers

A trait-like query such as F_residual is an ordinary meta callable. Its direct
result is its invocation instance type tau_M. Allow/deny or other helper
material resides in ordinary Val2, accessed by ordinary navigation/extraction;
there is no trait-result ontology.

The default instance is formed by its first legal invocation. Customization
writes the retained instance's actual member Place with current OpenHere,
Writable and ordinary access/lifetime evidence. Copying the outer result and
writing that copy does not mutate the retained default instance. Consumers
read the currently committed member observation at their own continuation
frontier; subsequent writes do not retroactively alter an already committed
decision. A consumer requiring stability must use its declared snapshot/Close
discipline.

Meta cache reuse preserves instance and member Places/current state and
rechecks dependencies. It never grants permission to invoke meta under
SealDom. A previously completed payload may be read there if ordinary rules
admit the read. Fixed language consumers such as residual escape are defined
by their topic owners, not by an unconstrained user trait dispatcher.


## 8. Closure and dependency boundary

Every legal completed closure expression produces its complete tau_C through
ordinary struct formation; invoking an ordinary meta callable still directly
produces its own tau_M. These are distinct operations. Optional generative
heads, HoleRef selectors and general expression bodies use the ordinary
[declaration relation](../patterns-overload/operator-patterns-and-generative-declarations.md).
Input identity and cache reuse preserve the observations specified by the
[general dependency owner](../symbol-world/dependency-observation-and-realization.md).
Reuse never reinitializes or recaptures dependencies, and stable identity
supplies no extra lifetime/escape capability.
