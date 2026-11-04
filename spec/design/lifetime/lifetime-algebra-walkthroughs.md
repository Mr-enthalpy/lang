# Lifetime algebra walkthroughs

Status: design illustrations applying the [lifecycle owner](lifetime-policy-and-overload-boundary.md).
These examples are acceptance targets, not executable source coverage. The
[handoff](../../contracts/lifetime-program-consumers.md) lists the missing
consumers. This file defines no independent world, evaluator or contract logic.

`lang` blocks below use pipeline calls. Library names and their signatures are
schematic; each is conditional on real ordinary definitions/registrations and
the stated Pre. Tables and `text` blocks describe material, not extra syntax.
The suffix `*` in equations marks an already selected operation. A companion
uses that selection's correspondence; erasing Stage never selects it again.

## 1. Branches, recursion and nested InPlace exits

### Source and selected actions

Assume ordinary `Nat`, `is_zero`, `predecessor`, and the registered bool branch
family. `is_zero` observes an explicit share; that handle's interval ends before
`predecessor` consumes fuel. Consider the admitted family where `r` is an
already formed movable handle of deduced type T. No arbitrary T movement,
implicit borrow or implicit clone is asserted.

```lang
let descend = <T>(self, r:T, fuel:Nat::):runtime -> o:T => {
    (fuel share |> is_zero)
        |> if {
            () |> {
                () |> { r return; };
            };
        }
        |> else {
            let n = fuel |> predecessor;
            (r, n) |> self return;
        };
};
```

The Ordinary outer expression forms tau_descend, its callable c, classifier
A_c and implementation leaf. Calling the type selects (c, Impl); source self
is c, CallAnchor is retained for `(r,n) |> self`, and Return is the callable
tag plus actual activation F_i. Each recursive call creates F_(i+1), even when
its tag equals the parent's. Its explicit Product has two positions; implicit
Param is the formed capture Product, not a third explicit self argument.

Every `{...}` here forms a distinct InPlace tau with automatic dependencies.
The two nested bodies are invoked by `() |>`, with zero explicit Product
entries. A bare block alone would only form a closure. Their dependencies
realize the outer r once through the selected ordinary action, without
recapture on invocation. Branch heads retain `(_, let _: (if)$)` and its
else counterpart. These anonymous heads do not shadow the source self binder.
The enclosing self used for recursion is an admitted dependency, not the
nested closure's receiver or its private capture layout.

On the leaf path, `r return` targets F_i across the InPlace frames. None of
those frames produces a local unit or a normal result binding. The recursive
path explicitly returns the new call's result to F_i; its terminal demand is
the established ReturnPattern/Pout before inner maxima. It inserts no hidden
temporary and does not change ABIKind. Guards retain ordinary restricted
Split/D completion; a branch mismatch is not a failed selected extractor.

### Lifetime transcription

Here is the ordinary-program skeleton of the transcription. `companions`
denotes the actual paired entries retained from selected actions, and `m`
is ordinary structured action/control material. These are illustrative API
roles, not a runnable library or a second interpreter.

```lang
let r_l = r@;
let fuel_l = fuel@;
let shared_l = fuel_l |> share_partner::companions;
let guard_l = shared_l |> is_zero_partner::companions;
let m = (guard_l, r_l, fuel_l) |> guarded_material::companions;
let post_type = m |> boundary_post::lifetime;
```

`is_zero_partner` records the observation/use and its ordinary delivery. It
cannot compute an unknown Nat value from fuel_l. `guarded_material` assembles
the retained true/false Pattern arms with the actions below. It executes no
unchosen branch effects. For a known choice only the reached actions commit;
otherwise the actual guarded continuation remains. The rows describe what
ordinary pattern extraction, Product assembly and calls operate on:

| Position | Input and computed material | Commit / exit |
|---|---|---|
| Entry F_i | Actual input names, generations, ordered Param, result position o_i, fixed cuts, mapped Pre | Caller checks Pre; body admits it with entry authority |
| Guard | fuel name -> explicit share handle h, target/generation, shared region; selected observation produces guarded true/false material | Pre before share/use; end h at its fixed cut, preserving history |
| Leaf, closure formation | Dependency realization for r; each InPlace tau/c/A/() and activation; no new referent | Ordinary formation/transfer Pre and atomic complete publication |
| Leaf, return | Actual transferred handle name d; origin(d)=origin(r) for Kill Move; region(d) begins at the transfer cut | Crossing each real exit compresses local dependencies; no intermediate normal result |
| More | predecessor consumes current fuel under its own Post; n has fresh complete identity and selected origin, not guessed from numeric equality | No pre-move followed by fallback clone |
| Recursive call | Current r and n transferred once to F_(i+1), selected self/Param and actual returned material | Instantiate recursive exposed contract, not its private history; then deliver to F_i |
| Outer boundary | Guarded origins plus every branch's use/move/drop/Color effects and actual exits | Compress, form self-root Post type; body checks Post; caller admits it |

For the restricted family whose only outward delta is handle-source delivery,
both arms retain the same external unknown root U from r and the result has
the corresponding `from(r@)` summary. This conclusion additionally requires
that local fuel/guard/closure cleanup has no other externally observable delta.
If it does, that delta remains in the Post material; returning the same handle
does not justify dropping it.

### Recursion obligation

This example uses two different finite arguments. An available finite Nat
decreases under the selected predecessor, giving a well-founded source
recursion argument on that family. A runtime-unknown fuel does not let compile
evaluation run all recursive activations. Modular boundary formation then
requires a validated recursive contract family; absent that consumer/evidence,
the result is Unavailable. A regular descriptor of the same origin path is
finite but supplies no proof that an arbitrary recursive contract program
terminates. Neither argument proves TCO, bounded stack or reusable machine
slots. These are separate [proof obligations](lifetime-policy-and-overload-boundary.md#14-proof-obligations-and-engineering-freedom).

## 2. select_one and boundary origin compression

### Source, entry and internal material

```lang
let select_one = <T>(self, r1:T, r2:T, choose:bool::):runtime -> o:T
    lifetime pre o@..live_with(r1@ || r2@)
    => {
        choose
            |> if {
                () |> { let a = r1; a return; };
            }
            |> else {
                () |> { let b = r2; b return; };
            };
    };
```

This applies only where the actual moves, unused-input cleanup, captures and
return are legal. In particular it is not a generic permission to discard an
exclusive referent or to copy its handle. `o@` in Pre is the established result
position's name material, not an already constructed result Object.
`live_with` is a candidate name for `depends_on`; the Pre constrains admissible
output intervals for this source-choice family.

Take an entry material with these finite known prefixes (arrows are origin
fields, not borrow operations):

```text
r1 -> p1 -> U1 = unknown
r2 -> p2 -> U2 = unknown
true exit:  local observation a_l -> p1 -> U1
false exit: local observation b_l -> p2 -> U2
```

After a Kill Move of r1 into a, `origin(a)=origin(r1)=p1`; it is a sibling
handle generation, not a child borrowed from r1. An ordinary observation
carrier can refer to a; that carrier's link is not another borrow. By contrast,
a genuinely selected child borrow has its actual child/target relation and
Pre. No automatic reborrow is inserted to make either picture fit.

The ordinary boundary program extracts each actual origin field, matches its
available form, composes its region/path/Color constraints, and retains the
first unknown root. With an explicitly terminated branch `p1 -> None`, it
retains that actual termination instead. Missing observation is Unavailable;
it cannot be relabeled None or made into an arbitrary new unknown root.

At the inner InPlace exit this removes a_l or b_l from the outward material
while their actual histories and any validity constraints remain. At F's
exit it removes remaining function-local carriers. The external caller can
read the resulting material after those locals die, without rereading them.
Unknown roots retain identity and dependency obligations.

### Choice, Post and its ordinary construction

Use **compress each guarded arm, then combine** for this example. The input
arms have a common mapped boundary vocabulary, legal independent exits, the
same exposed output interface and a defined choice-material constructor.
Retain each arm's source and effects together:

```text
(true,  source=r1/p1/U1, region R_true, paths A_true, effects H_true)
||
(false, source=r2/p2/U2, region R_false, paths A_false, effects H_false)
```

This is ordinary correlated sum/Pattern material, not two simultaneous
exclusive targets. It is not an arbitrary join of unrelated results. Subject
to the family's stated outward-effect condition, its return-source component
forms:

```lang
lifetime post o@..from(r1@ || r2@)
```

Any additional observable H_true/H_false remains guarded in the full Post.
`from`'s check may call the appropriate `live_with` computation for interval
admissibility; that alone does not establish the selected source and transfer
delta needed by admit. The operations have different consumer roles.

A possible ordinary implementation organization is:

```lang
let arms = (true_exit_material, false_exit_material);
let compact = arms |> compress_guarded_origins::lifetime;
let relation = compact |> source_choice::lifetime;
open let contract_type = relation |> form_post::lifetime;
```

Here arms/compact/relation are Products or other ordinary intermediate values,
not necessarily NameExpr. The functions must actually extract the origin,
cut, path, Color and effect data just described. `form_post` uses its existing
CompileInstance self projection, actual OpenHere and ordinary structural
formation/associated member installation to deliver a single self-root type
with check/admit. A Product containing an instance-open type cannot escape.
Writing a copy of contract_type does not inject into the retained instance.

The function bodies above owe the following ordinary data flow, rather than
merely recognizing those function names:

1. Extract the exit sum/Pattern and its local/external name partition. For each
   origin observation, distinguish a known parent link, an actual Unknown(root)
   boundary and explicit termination using ordinary registered Patterns. These
   displayed constructor names are material descriptions, not fixed syntax.
2. On a known link, retain the source identity and compose its region order,
   permitted path and inherited Color with the accumulated descriptor. Continue
   on the parent using an ordinary recursive call. Delete a local carrier only
   from outward reads; keep its required validity relation in the descriptor.
3. At Unknown(root) retain that same root and accumulated constraints; at
   explicit termination retain termination. Missing field/combination evidence
   stops unavailable. Finite acyclic prefixes have decreasing remaining length;
   regular cycles require an independently justified regular-family operation,
   not a new recursion cutoff or guessed unknown.
4. Assemble the two complete descriptors as a correlated sum retaining each
   guard and H. Extract the required source-choice view only after preserving
   every other observable component. This is the material captured by the
   formed associated behavior, not a text string encoding a formula.
5. In form_post's already established instance I, ordinary structural formation
   first supplies the complete construction subject needed for member access.
   Under the actual OpenHere/Writable/initialization premises, qualified ordinary
   formation installs check and admit at the corresponding members of that I.
   Their ordinary closure dependencies retain the compact relation once; they
   do not retain local a_l/b_l or recapture on a later boundary invocation.
6. check reads the mapped world, extracts the appropriate guarded relation and
   tests its region/path/Color/source requirements through the admitted ordinary
   operations, returning bool. admit consumes the authorized boundary world
   view and applies only the mapped successful relation/effect delta. Neither
   recomputes business branch values, executes private H again, or invents an
   external fact. Both retain their own real self/Param and ordinary Pre/Post.
7. Deliver the complete I-root type directly, not nested in an intermediate
   Product. Ordinary completion/Close follows delivery. The returned members'
   actual dependencies must survive that delivery; instance identity alone
   grants them no lifetime.

This gives a well-founded ordinary recursion for finite known prefixes and a
specified input/output for the separate regular case. It does not prescribe
the record encoding, cache or final function decomposition.

The boundary supplies the mapping of r1/r2/o and activation plus the authorized
world update view. Constructing the type never gives its code arbitrary world
writes. The body checks the formed Post; the caller admits only its mapped
guarantees, without replaying H_true or H_false.

### NoDelta and negative exits

For an implementation with no observable boundary delta, an LHS Function
Pattern omitting Post accepts when the existing Pre implication and other
interface checks hold. select_one whose exposed source effect changes an
observed output relation has a non-NoDelta Post; a handwritten interface
omitting Post rejects it at extraction. An interface exposing sufficient
`from(r1@ || r2@)` information may accept by ordinary contract compatibility.
Known Val1 of a projected function value cannot recover its erased external
structural-source authority or infer a stronger hidden Post.

For a concrete compatibility comparison, let A0's **established exposed**
contract be `(Pre=U_default, Post=NoDelta)` and let Aselect's established contract
be the select_one source-choice contract just derived. Their implementation
evidence is required; a helper name or absence of a clause does not supply it.

| Expected handwritten interface | Actual exposed contract | Extraction |
|---|---|---|
| F0 omits Pre/Post: U_default / NoDelta | A0 | Accept when the other input/output/Policy checks hold; both implications are identity |
| F0 omits Pre/Post | Aselect with an observed non-NoDelta source effect | Reject: its Post cannot imply NoDelta; body structure is unavailable to this interface |
| Fselect explicitly exposes the sufficient mapped Pre and from/source-effect Post | Aselect | Accept when the two ordinary implications and all other interface checks are established |

Replacing r1 by a real borrow of an expiring owned local fails escape Pre.
Deleting that local's name from the public origin material cannot enlarge the
referent's valid region. InPlace target return compresses real exit material;
failure publishes no result or success Post. Unit fallthrough has its actual
history and no synthetic non-unit result; unfinished work has no successful
Post either. Nested-then-outer compression needs observable equivalence, not
an assumed unconditional commute with choice combination.

## 3. Dynamic safeid and batch share partitioning

### try_trust and ordinary success/failure Patterns

The compact design seed is:

```lang
let safeid = (vector ref ..try_trust(num))?;
```

It means `(vector ref, num) |> try_trust::adl`, followed by the selected
one-layer extraction. It establishes no universal error propagation or success
unwrap. For an ordinary registered `ok | err` result with a declared exposed
view, a full use instead separates the paths:

```lang
let outcome = vector ref ..try_trust(num);
outcome?
    |> ok { /* ordinary named extraction obtains the successful id */ }
    |> err { /* ordinary extraction obtains the failure payload */ };
```

The head names stand for the actual registered success/failure Patterns;
payload extraction follows R_Gamma, not a special meaning of `try` or `?`.
If that one-layer result view is unavailable, the use is unavailable. On
failure the bare binding in the compact seed does not magically denote a
trusted integer. Branch code must explicitly bind the id from the ok payload.

| Step | Material consumed / computed | Result and stop boundary |
|---|---|---|
| Ordinary Pre | Actual vector ref target/generation, index name n/generation, valid region, authorized read of the resource and legal input moves | A failed Pre prevents the selected external action; unsafe cannot repair it |
| Dynamic validation | Concrete runtime index and resource state; library checks bounds plus the identity/layout/version conditions of its protocol | Failed validation produces err with no trusted id or success credential |
| Successful action | Actual successful check tied to resource identity Q and the delivered id name s/current generation | Success Post adds compatible external knowledge at common commit; no separate fact import |
| Lifecycle Post | `origin(s)` source relation, interval constraint, path/range material and shared Color on the resource dependency | s may have the same ordinary integer type as num and need not be ref/share |
| Subsequent access | s's credential and current protected resource/window; actual selected range accessor | Accept only that maintained evidence; missing evidence is unavailable/rejected, without fallback check or reselection |

For example the ordinary lifecycle record may carry `(s, generation_s,
Q, generation_Q, protected_ranges, valid_interval, selected_source)`; these
are named fields/material, not a mandated credential Object encoding. Numeric
equality between another integer t and s supplies none of this mapping.
An actual Move of s preserves only the relations granted by its selected
Post and MoveEffect. A selected clone or arithmetic transformation needs its
own Post; it cannot inherit trust merely by producing an equal integer.

The library may protect `capacity` and a range, but must cover **every** action
that can falsify validation: shrinking length, reallocating/reordering storage,
replacing the resource generation, changing the index and any other relevant
layout/content condition. Such actions either conflict with the credential's
Color/access or require its interval to have ended. A field name alone proves
no protection. A view can transfer the credential only with its actual names,
dependencies and valid interval; it cannot globalize it.

With this protocol established, accesses in the valid interval need not repeat
the same runtime numeric validation. They retain their other ordinary Pre and
machine costs. After legitimate interval end the required exclusive operation
may proceed in a new region; the old validation and shared Color history remain.

### Batch consumption of share, acquisition from original ref

Suppose share handles h1/h2 retain paths/ranges A1/A2 from the same original
ref R, with actual current authorization in R's strict tree. An illustrative
ordinary call is `(h1,h2) |> partition_refs`; its final parameter list need not
include R again if a legal dependency really retains R and its authority.

1. Read the actual source identities/generations and permitted endpoint
   operations. Establish the common original R and legal regions. Ref cannot
   cross share or alias nodes to obtain these ranges.
2. Compare **actual operation ranges**, statically when available or by the
   facility's selected runtime validator. Distinct path strings do not prove
   disjointness. Ancestor-whole and descendant access may overlap; a shared
   route prefix used only to reach disjoint leaves need not overlap.
3. Under all ordinary Pre, consume the input share intervals once. Check other
   surviving shares, descendants and logical credentials separately; consuming
   h1/h2 does not consume those dependents.
4. At the successful existing atomic boundary end the admitted old intervals
   and form complete **new** child ref handles from R's authority, with actual
   child targets, new generations/regions and exclusive Color. Post preserves
   prior shared history and the validated source/range correlation.

With equal or intersecting ranges, foreign origins, unavailable authorization,
or a remaining conflict, publish no child refs. If the operation reports a
normal failure value, its explicit failure contract determines retained input
ownership; a failed precheck consumes nothing and no success Post is admitted.
This is not a `share -> ref` strengthening candidate or an excuse to retract
Color. For one input, the pairwise comparison set is empty, so no extra dynamic
partition check is needed; all source/region/access/Color Pre remain.

The trusted Post describes the validator's actual success. The materializer
must implement a sound, lightweight test for this facility's abstract ranges.
Address-as-integer comparison is no language axiom. Representation and exact
cost belong to machine/library work, not the access algebra.

## 4. Alias, partbox and option

### Root-only alias and a bounded window

Here alias means a library capability for alias writes, distinct from lexical
`let x === PathMaterial`. Initially its normal access tree outside its own
root is empty. Even root replacement requires a real registered default;
spelling `default` or knowing a payload's type supplies no assignment or field
implementation. Registered ordinary field extraction may obtain an ordinary
value, but cannot cross ref/share nodes or smuggle a normal borrow path.

An illustrative window call uses an explicitly formed callback:

```lang
let body = (window_self, r:PayloadRef::) => {
    r |> registered_payload_operation
};
(alias_handle, body) |> with_ref_window;
```

PayloadRef denotes the facility's admitted normal ref type. The callback is
an Ordinary closure with its own return barrier, so it cannot target an outer
return capability. This alone is insufficient to prevent storing/returning r;
the window Pre and escape checks must do that work.

| Phase | Ordinary material and obligation |
|---|---|
| Enter | Actual alias resource Q, current validity and facility exclusion protocol. Missing exclusivity fails before action; anonymous names supply no proof |
| Form | Establish fresh anonymous origin u_i for this actual formation, distinct from other admitted u_j, and finite region W_i. Form ref r with origin u_i, never alias_handle as origin |
| Invoke | Move r and callback's already formed Param once. All normal ref operations within W_i follow ordinary access/Color/Pre; no invocation-site recapture |
| Protect | Facility blocks conflicting alias operations and overlapping active windows throughout W_i, and maintains referent validity |
| Exit | Check all returned/stored material against W_i; cleanup is already fixed. End window and ref intervals under ordinary actions; admit only the actual successful Post |

A second route consumes the alias through a trusted operation into an ordinary
value when its real validity/ownership premises hold. It is separate from the
temporary ref route. Neither introduces unsafe pre, extends a window through
unknown evidence, or identifies fresh anonymous roots with disjoint physical
targets. A facility failing to maintain its asserted exclusion violates its
trusted external description.

### Ordinary shares of a resource right

Let resource identity Q have full accounting amount 1. A partbox value carries
Q's identity, current validity and amount q as ordinary material; q can be
compile or runtime, without being a business-type parameter. Numbers below
are a design example, not a required machine representation.

```text
split: consume (Q,1) -> complete (Q,1/3), (Q,2/3)
merge: consume (Q,1/3), (Q,2/3) -> complete (Q,1)
box:   consume (Q,1) -> complete owned box of Q, when all other Pre hold
```

Each action checks actual resource identity and conservation before commit;
same numeric amounts for Q and Q' cannot merge. There is no ordinary clone
that multiplies rights. Moves transport an existing share under its selected
Post, and failed Pre leaves inputs unconsumed. A normal failure return has
its own explicit input/result contract. A non-full amount offers only its
restricted alias capability, not a normal ref. Full amount still cannot
consume to box with an outstanding window, conflicting Color, invalid object
or other unmet ordinary Pre.

One possible option-based topology protocol keeps an optional partbox in each
admitted link slot. Forming a link consumes a share from its current owner and
constructs the complete some branch by its registered ordinary constructor.
Removing a link uses actual option extraction: none has no share; some yields a complete
share through consuming extraction, ends that option identity and leaves any
replacement None through its separate authorized complete write. No partly
extracted parent or implicit copying is permitted.

For example a node's local ledger plus two neighbors' admitted link slots may
hold `(Q,1/3)` each. The topology operation must obtain the slots' actual write
authority, consume their some payloads, install complete none replacements and
merge the three shares of Q. Only after full amount and all other Pre does it
obtain box, then explicit ref/share for payload use. A facility may instead
use several ordinary commits; it must describe intermediate ownership and
failure cleanup, and cannot promise rollback of earlier external effects.

Modifying adjacency is distinct from payload access. A topology alias/default
can rearrange links according to that protocol while payload normal refs are
unavailable. Conversely a payload window grants no right to change links.
Actual distribution may involve more than two neighbors, inaccessible rights
or cycles. Conservation alone proves neither collection of all rights, memory
safety, concurrency correctness nor automatic reclamation. Those remain
specific library proof obligations; this example asserts only a possible
protocol with explicit evidence and failure boundaries.
