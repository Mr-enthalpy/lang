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

The following is an expanded **design program**, not accepted/executed compiler
output. The callable entries in `selected` are the counterparts of the original
sealed share, observation, predecessor, input Move and return actions. The
namespace name is schematic; it is supplied by that correspondence, not found
by resolving the business expression again. Frame/Boundary/Exit/Rows below are
ordinary complete material types with real registered extraction/field members.
Their encodings and these API names are not frozen.

First, spell out the ordinary recursive local-elimination operation. The
registered origin view has three ordinary disjoint branches: a known link, an
actual unknown root, or explicit termination. A missing observation produces a
diagnostic, not a fourth branch that can silently succeed. This example admits
finite known prefixes; a cyclic regular family needs the separate §14 evidence.

```lang
let compress_origin =
    (self, cursor:LifeName::, acc:OriginAccum::):compile -> _:RootMaterial:: => {
        cursor |> observe_origin::lifetime
            |> (_, let edge:KnownOrigin::) {
                let (parent, order, colors, paths, choices, effects) =
                    edge |> split_link::lifetime;
                let next = (acc, order, colors, paths, choices, effects)
                    |> compose_link::lifetime;
                (parent, next) |> self return;
            }
            |> (_, let u:UnknownOrigin::) {
                (u, acc) |> retain_unknown::lifetime return;
            }
            |> (_, let end:TerminatedOrigin::) {
                (end, acc) |> retain_termination::lifetime return;
            };
    };

let compress_rows =
    (self, rows:Rows::, boundary:Boundary::):compile -> _:Rows:: => {
        rows
            |> (_, let empty:EmptyRows::) {
                empty return;
            }
            |> (_, let cell:NonemptyRows::) {
                let (row, rest) = cell |> split_row::lifetime;
                let (cursor, order, colors, paths, choices, effects) =
                    row |> split_outward_row::lifetime;
                let acc = (order, colors, paths, choices, effects)
                    |> origin_accum::lifetime;
                let root = (cursor, acc) |> compress_origin;
                let head = (root, boundary share) |> map_outward_row::lifetime;
                let tail = (rest, boundary) |> self;
                (head, tail) |> prepend_row::lifetime return;
            };
    };

let compress_exit =
    (self, exit:Exit::, boundary:Boundary::):compile -> _:Exit:: => {
        let (target, delivery, rows, control) = exit |> split_exit::lifetime;
        let mapped = (delivery, boundary share) |> map_delivery::lifetime;
        let outward = (rows, boundary) |> compress_rows;
        (target, mapped, outward, control) |> exit_material::lifetime return;
    };
```

`compose_link` combines exactly one observed edge's region order, inherited
Color, permitted paths and correlated effects; it cannot manufacture a root
or lengthen validity. `map_outward_row` uses the boundary's actual local/external
partition, removes reads of exiting local carriers and retains those composed
constraints. `map_delivery` uses the already selected result consumer; it does
not select or create a result Place. Rows cover **all** outward result,
dependency and effect components, not only the return origin. Each split and
constructor is ordinary Pattern extraction/complete formation. Those finite
operations are smaller inputs to the stated composition obligations, not a
hidden whole-program `boundary_post` operation.

Now transcribe the source function. `frame` is the actual source invocation's
established name/selection/activation/cut/delivery material. Scope queries below
return owned boundary descriptors for the source function, branch and two
nested InPlace scopes, including their formed dependencies and fixed cleanup.
They neither invent activations nor execute that cleanup a second time.

```lang
let descend_life =
    (self, frame:Frame::, r_n:LifeName::, fuel_n:LifeName::):compile
        -> _:Exit:: => {
        let function_exit = frame share |> function_boundary::lifetime;
        let (h_n, share_history) =
            (frame share, fuel_n share) |> share_fuel::selected;
        let (guard, guard_history) =
            (frame share, h_n) |> is_zero::selected;
        let is_leaf = (frame share, guard share) |> available_guard::lifetime;

        is_leaf
            |> if {
                let branch_exit = frame share |> true_boundary::lifetime;
                () |> {
                    let middle_exit = frame share |> middle_boundary::lifetime;
                    () |> {
                        let inner_exit = frame share |> inner_boundary::lifetime;
                        let (out_n, move_history) =
                            (frame share, r_n) |> move_leaf_result::selected;
                        let raw = (frame share, out_n, share_history,
                                   guard_history, move_history)
                            |> return_exit::selected;
                        let e1 = (raw, inner_exit) |> compress_exit;
                        let e2 = (e1, middle_exit) |> compress_exit;
                        let e3 = (e2, branch_exit) |> compress_exit;
                        let e4 = (e3, function_exit) |> compress_exit;
                        e4 return;
                    };
                };
            }
            |> else {
                let branch_exit = frame share |> false_boundary::lifetime;
                let (n_n, predecessor_history) =
                    (frame share, fuel_n) |> predecessor::selected;
                let (child_frame, child_r, child_fuel, input_history) =
                    (frame share, r_n, n_n) |> recursive_input::selected;
                let child_exit = (child_frame, child_r, child_fuel) |> self;
                let raw = (frame share, child_exit, share_history, guard_history,
                           predecessor_history, input_history)
                    |> returned_call_exit::selected;
                let e1 = (raw, branch_exit) |> compress_exit;
                let e2 = (e1, function_exit) |> compress_exit;
                e2 return;
            };
    };

let r_l = r@;
let fuel_l = fuel@;
let outward = (source_frame, r_l, fuel_l) |> descend_life;
let actual_post_type = outward |> form_post::lifetime;
```

The last four lines are the projection entry at the source boundary, with
`source_frame` supplied by that invocation's ordinary E correspondence. `@`
projects the already established names, not their unknown business contents.
Each `selected` call preserves the original action, input/output generations
and MoveEffect. For example recursive_input supplies the **new** child input
names after the actual call's single input transfer; child entry does not Move
the parent's old generation again. return_exit records the existing target
tag/activation and actual ReturnPattern delivery. returned_call_exit records
child completion followed by the parent's real return, with no hidden result
binding. No selected action is replaced or retried.

`available_guard` observes only an actually available control decision already
established by the source call. It does not recover a Nat value from fuel@.
For the complete finite witness, those successive observations and finite fuel
are legally available. If a business guard or recursive child frontier is
unknown, E leaves the actual guarded continuation unfinished, or reports the
unavailable observation when its transport consumer is missing. It does not
run both branches. An ahead-of-execution **summary** of both arms must instead
use their already formed guarded Pattern material and the justified modular
family. Its source and formation-time transformation are spelled out in the
[select_one summary witness](#advance-summary-and-actual-exit-witness). The
program above is the **actual-exit** consumer: actual_post_type is a precise
realized-path result, not the way to obtain the callable's advance summary.
The traversal does not create that evidence by executing future actions.

descend_life is an Ordinary closure with its own self/activation. The `|> self`
call sites use its CallAnchor projection for recursion, independently of the
compile-instance name observation used by the type-forming program in W2.
Its anonymous
branch heads do not bind over the outer self used for recursion. The two `()`
calls invoke actual InPlace lifecycle closures with zero explicit entries.
The innermost `e4 return` crosses them and the branch to that Ordinary frame;
it produces no intermediate normal unit/result. Its **Exit payload** separately
retains the corresponding source F_i return target. The four explicit
compression calls transform the material for the four actual source exits
while local observations are available; they do not move cleanup cuts.
Formation/dependency/cleanup effects in each boundary and every selected action
remain in the rows. `form_post` is expanded as ordinary code in W2 below.

These projection-complete lifecycle programs use ordinary values, calls and
their own required lifecycle premises. They are not transcribed into another
`@` program; their foundational operations obey §9.1 of the lifetime owner.
The following table relates the spelled-out program to its source material:

| Position | Input and computed material | Commit / exit |
|---|---|---|
| Entry F_i | Actual input names, generations, ordered Param, result position o_i, fixed cuts, mapped Pre | Caller checks Pre; body admits it with entry authority |
| Guard | fuel name -> explicit share handle h, target/generation, shared region; selected observation produces guarded true/false material | Pre before share/use; end h at its fixed cut, preserving history |
| Leaf, closure formation | Dependency realization for r; each InPlace tau/c/A/() and activation; no new referent | Ordinary formation/transfer Pre and atomic complete publication |
| Leaf, return | Actual transferred handle name d; origin(d)=origin(r) for Kill Move; region(d) begins at the transfer cut | Crossing each real exit compresses local dependencies; no intermediate normal result |
| More | predecessor consumes current fuel under its own Post; n has fresh complete identity and selected origin, not guessed from numeric equality | No pre-move followed by fallback clone |
| Recursive call | Current r and n transferred once to F_(i+1), selected self/Param and actual returned material | Instantiate recursive exposed contract, not its private history; then deliver to F_i |
| Outer boundary | Reached origins and all mapped use/move/drop/Color effects at actual exits | Compress actual Post and check it against the already formed summary; caller adopts only successful mapped Post |

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

### One finite end-to-end trace

Take the admitted witness with fuel=1, then fuel=0 in the recursive call, with
both guard observations legally available at the respective current frontier.
Let the external handle target be Q and its retained unknown origin root U.
All positions below are the source actions' already fixed cuts, not cuts chosen
by the lifecycle program to satisfy a constraint.

1. Entry supplies r@/fuel@ and frame F0. The selected share/is_zero counterparts
   form the shared handle/observation material and record the false guard.
   The share interval ends at its fixed cut before predecessor consumes fuel.
2. predecessor's actual counterpart forms the complete next fuel name. The
   value 0 comes from that ordinary producer, never from arithmetic on a
   LifeName. recursive_input maps the selected single input transfer to F1's
   new r/fuel generations and records its history once.
3. The ordinary call to descend_life on F1 reaches its true branch. Its nested
   lifecycle InPlace calls mirror the source exits. move_leaf_result retains
   Q/U and forms the delivered handle generation under the selected MoveEffect.
   compress_origin walks available links to the same U; compress_rows does
   this for every outward result/dependency/effect row.
4. e1–e4 eliminate the inner, middle, branch and F1 local-name reads in order.
   They retain the F1 target completion, Q validity, colors, paths and effects.
   `e4 return` is the lifecycle function's actual return across its InPlace
   scopes; the Exit value still describes the original F1 return.
5. In F0, returned_call_exit consumes F1's actual completion/delivery material
   and records F0's real return event. The last two compression calls process
   F0's branch/function exits. No extra source result binding, old-generation
   Drop, reborrow or second input Move appears.
6. The ordinary form_post program below can consume that complete outward Exit
   to form a precise actual contract. The callable's automatic summary is
   already available from formed body structure, through the distinct W2
   transformation below. Body Post checking requires the mapped actual result/
   history to satisfy that summary. Caller Post adoption uses
   the corresponding success-scoped world view and the independent component
   update premises, then participates in public E's joint Post publication.
   It imports the summary; it does not replay steps 1–5.

The useful return-source component is from(r@) only under the stated outward
effect condition. All other rows remain in the complete summary. If any
observation, formation, recursive-family or boundary consumer in this trace is
missing, that point is unavailable; the trace is not executed test coverage.

## 2. select_one and boundary origin compression

### Concrete ref source selection

Keep the original ref case explicit. Let Payload be an ordinary complete
payload type. Its ordinary type-forming ref operation gives the external
classifier below; `$` in the formal/return Pattern uses that ready type value.
It is not a reborrow. This is again a design target, not parser/runtime coverage.

```lang
let PayloadRef = Payload:: |> ref;
let select_one_ref =
    (self, r1:PayloadRef$, r2:PayloadRef$, choose:bool::):runtime -> o:PayloadRef$
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

r1/r2 are actual exclusive ref **handles** to valid external targets Q1/Q2.
The signature result position o has no resident before delivery. Normal input
Move, dependency formation, cleanup of the unselected handle, target validity
and all other Pre must hold. The true arm transports the selected r1 handle
through a and into o; the false arm does the corresponding r2 transport. A
killing handle Move creates sibling handle generations with the same target
and inherited origin, not a borrow of the local handle Place. Ending a handle
does not imply destroying its external referent.

| Reached arm | Internal material | Compressed outward source component |
|---|---|---|
| true | Actual transferred handle a, target Q1, origin prefix p1/U1, its region/path/Color and other effects | o is from the r1 choice and remains constrained by Q1's validity |
| false | Actual transferred handle b, target Q2, origin prefix p2/U2, its region/path/Color and other effects | o is from the r2 choice and remains constrained by Q2's validity |

For this legal source-choice family the return component is
`lifetime post o@..from(r1@ || r2@)`. Other outward effects stay guarded with
their chosen source. It is one exclusive ref with one actual target, not two
exclusive capabilities. If Q1 is a dying owned local instead of an admitted
external target, the true exit still fails; compressing a does not repair it.
The following generic T example is an additional admitted family, not a
replacement for this concrete ref case.

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

Use **transform each formed guarded arm description, then combine** for the
advance summary. Those descriptions come from semantic body formation as
specified below; they are not two actual Exit values obtained by running both
arms. Actual-exit compression separately processes the one reached arm. The
arms have a common mapped boundary vocabulary, legal exit relations, the same
exposed output interface and a defined choice-material constructor. Retain
each arm's source and effects together:

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

The resulting guarded Pattern retains source/effect correlation and every
outward row. It is distinct from W1's actual Exit material. Both are ordinary
inputs to the contract construction below; neither requires a wrapper contract
Object or formula string.

### Advance summary and actual-exit witness

For select_one_ref, semantic formation of the body supplies these available
materials without observing choose or performing either return arm:

| Formed body component | Origin of its meaning / retained description |
|---|---|
| Input/output roles | Ordinary head/call formation supplies bound r1/r2 and o positions; actual names and activation are supplied by later ordinary call mapping |
| Guarded alternatives | The formed ordinary bool/branch structure supplies a guard position g with true/false alternatives; the runtime value of g is not observed |
| Per-arm operations | The established selected handle-transfer and return counterparts supply their structural relations, including target/origin inheritance, permitted region constraints and all effect rows |
| Scopes and locals | Actual formed closure/dependency/return structure supplies the local binder positions a/b and crossed InPlace/function boundaries; it does not create their future generations or cuts |

The selected Move relation in the first arm describes target(a)=target(r1) and
origin(a)=origin(r1) for its killing transfer; the analogous second-arm relation
uses r2/b. These are descriptions of the already determined operations. The
structural transformer substitutes/composes those bound relations to eliminate
a/b from the outward Pattern, retaining target validity, inherited colors,
access and every effect. It does **not** ask observe_origin for a nonexistent
future a@, execute move_leaf_result, or manufacture an actual unknown root.
External unknown-root parameters retain their identities through actual mapping.

The callable form_post is assumed already formed; its definition is expanded
below. These explanatory excerpts do not request forward lexical lookup. One
possible ordinary program arrangement is:

```lang
let (roles, guard, left_body, right_body) =
    formed_select_one_body |> select_one_parts::lifetime;
let left_rows = (left_body, roles share) |> transform_formed_arm::lifetime;
let right_rows = (right_body, roles) |> transform_formed_arm::lifetime;
let conditional = (guard, left_rows, right_rows)
    |> guarded_exit_pattern::lifetime;
let post_summary_type = conditional |> form_post;
```

formed_select_one_body is supplied by the ordinary **semantic** body/Pattern
consumer with those established descriptions. It is not Raw/Norm syntax, source
text, an arbitrary user-shaped record or a list of guessed future actions.
transform_formed_arm operates on the bound relation rows just enumerated:
compose the selected transfer/return relation, eliminate local positions at each
described boundary, and preserve the guard-correlated rows. Unlike W1's
compress_exit, it requires no executed-path history or available runtime guard.
Both transformations have the same outward observation obligations. The final
constructor packages ordinary guarded Pattern material; it commits no described
world effects. Its own ordinary computation/formation still obeys E and the
bootstrap basis.

For the complete supported family, the summary type is available **before a
consumer needs the callable's complete contract**, including before execution
of the selected call. The ordinary invocation instantiates formal roles/guards/
scope positions at the actual mapping theta. It does not decide the guard.
Here the summary's return component is from(r1@ || r2@), with the complete
correlated alternatives:

```text
(g=true,  source=r1, target=Q1, unknown_root=U1, regions=R1, colors=C1, access=A1, effects=H1)
||
(g=false, source=r2, target=Q2, unknown_root=U2, regions=R2, colors=C2, access=A2, effects=H2)
```

Q/U/R/C/A/H are mapped parameters and relational constraints in the summary,
not already committed future Objects/events. H includes unselected-handle
cleanup and every other exposed effect. U is the retained first unknown origin
boundary, not a replacement for the actual direct origin link. W1's actual-exit
compression helpers, applied to select_one_ref, later deliver this witness for
each legal successful path:

| Actual path p | Actual compressed material | Why it satisfies the instantiated summary |
|---|---|---|
| choose=true | The selected sibling handle transfers and real return preserve Q1/U1; actual regions/colors/access/effects satisfy the first arm's composed constraints | theta_p fixes g=true and maps the actual generations/cuts to R1/C1/A1/H1, so the first guarded alternative holds |
| choose=false | The corresponding actual r2/b transfer and return preserve Q2/U2 and the second arm's constraints | theta_p fixes g=false and maps those actual observations to R2/C2/A2/H2, so the second guarded alternative holds |

Thus, on this admitted bool family, `Post_actual(p, theta_p) implies
Post_summary(theta_p)` for every legal successful p. This finite structural
case argument observes no future guard and executes no future path. It uses the
established counterpart/closure/delivery relations, not a theorem about arbitrary
programs. An actual wrong source, invalid region or unrepresented outward
effect does not acquire permission by choosing the other summary arm after
failure. Body Post validation rejects without reopening; failure publishes no
successful Post. Caller Post.admit imports only the successful mapped guarantee.

Missing selected structural descriptions, lawful transforms or a needed founded
recursive summary makes summary formation unavailable. An opaque Function
Pattern can contribute only its already exposed sufficient contract, never its
erased private structure. No actual Exit, fabricated residual, speculative
branch execution, candidate enumeration or source-string recognition fills this
gap. This remains an unconnected design consumer, not executed source coverage.

### Ordinary self-root contract program

Here is a complete design example of the construction path. `form_post` has
an Ordinary compile invocation I. Its actual fresh formation action supplies
I's opening window and one-shot root initialization authority independently
of the contract it will return. The first call is the
[initial formation handoff](../symbol-world/structural-type-formation-and-composition.md#current-instance-initial-formation-handoff),
with a schematic source API name. It initializes **I itself** with a complete
empty structural type. `self#` uses the compile self projection and reads its
NameValue only. Passing that value is not sufficient to obtain the authority;
the callee must check the actual target/window/initialization premises.
The qualified `check::self`/`admit::self` roots and final type-valued self read
use that same current I. Recursive callable invocation would instead use the
CallAnchor consumer. These projections do not add another source self formal.

The material family below has ordinary legal share/clone for its immutable
descriptors. Cloning descriptive names/constraints copies no world permission.
Each associated closure captures one owned descriptor once. It does not borrow
the local relation whose lifetime ends when form_post returns. All library
Paths name registered ordinary operations; structural dependencies retain their
actual identity. An explicit capture list leaves no omitted external local value.

```lang
let form_post = <M>(self, material:M):compile -> _:type:: => {
    self# |> initialize_empty_root::construction;

    let relation = material |> contract_rows::lifetime;
    let for_check = relation share |> clone::lifetime;

    mut let check_ref = (let check::self:type) ref;
    check_ref = [let rel = for_check]
        (check_self, w:world::lifetime::):compile -> _:bool:: => {
            let source_ok = (w share, rel share) |> source_matches::lifetime;
            let region_ok = (w share, rel share) |> region_fits::lifetime;
            let color_ok = (w share, rel share) |> color_compatible::lifetime;
            let access_ok = (w share, rel share) |> access_is_subset::lifetime;
            let effects_ok = (w, rel share) |> effects_match::lifetime;
            (source_ok, region_ok, color_ok, access_ok, effects_ok)
                |> all_true::lifetime return;
        };

    mut let admit_ref = (let admit::self:type) ref;
    admit_ref = [let rel = relation]
        (admit_self, w:(world::lifetime::) ref):compile -> _:unit:: => {
            let w1 = (w, rel share) |> adopt_origin::lifetime;
            let w2 = (w1, rel share) |> adopt_region::lifetime;
            let w3 = (w2, rel share) |> adopt_color::lifetime;
            let w4 = (w3, rel share) |> adopt_access::lifetime;
            let w5 = (w4, rel) |> adopt_effects::lifetime;
            w5 |> end_boundary_view::lifetime;
            () return;
        };

    self return;
};

```

The admitted M families here are the actual Exit and formed GuardedExitPattern,
with their ordinary typed contract_rows extraction providers. They remain
distinct: one carries reached-path observations, the other a conditional
description over bound positions. contract_rows extracts the corresponding
lawfully transformed rows; it does not turn one actual path into an advance
summary, redo the body or infer an arbitrary contract. all_true is ordinary
finite Boolean
folding, not a contract evaluator. Its inputs have these concrete responsibilities:

| Ordinary computation | Material examined / required result |
|---|---|
| source_matches | Actual entry/exit mapping, selected origin/target/generation and guard-correlated source; equality of integers or lifetime descriptor copies is insufficient |
| region_fits | The actual cuts and interval/dependency constraints of every row; no extension of an expiring target |
| color_compatible | Established directed rows, inheritance and intervals, retaining committed history |
| access_is_subset | Actual operable endpoint ranges and modes; routing prefixes add no ancestor permission |
| effects_match | Remaining mapped guarded use/move/drop/cleanup/other effects, not just o's origin |
| adopt_origin/region/color/access/effects | The corresponding derivable mapped boundary component, under that operation's independent ordinary Pre on the supplied authority-scoped view |

The w1–w5 bindings are successive ordinary transfers of the **same target view**,
not separately granted worlds. Component operations return the transferred view
and use the existing boundary transaction's projection storage. All components
retain the original action/cut/activation/mapping. A failed later component
publishes no partial boundary delta; earlier independent commits are not undone.
end_boundary_view consumes the temporary view, not the world or the referent.
The explicit unit return is the admit function's real declared result, not a
unit invented for an intermediate target-return scope.

The summary family is formed over bound signature/body roles and instantiated
by an actual boundary mapping; an actual Exit instead supplies a precise reached
mapping. Each uses its owned member dependency under real receiver/Param premises. It
does not assume unlimited reuse of a consumed callable. A reusable library
version must choose and establish its ordinary permitted receiver/capture
strategy, such as actual selected clone-derived descriptor transport or a
proved preserving action; invocation never recaptures or silently copies it.

At member formation, check::self and admit::self navigate the **outer** compile
self's initialized current type. OpenHere, freshness, ordinary access/type/path
checks and each child Place's one-shot authority apply. Parent Writable is not
a child-name-formation premise. Explicit ref obtains the child's initial view;
ordinary assignment installs the complete closure type and consumes that
initialization authority. It supplies neither a persistent replacement ref nor
TypeAdd/V_tau registration. Invoking either named member later uses its full
ordinary callable type and its own check_self/admit_self; the operated world
is an explicit argument. There is no extra contextual self or registry.

Finally, `self return` reads the complete current tau_I and delivers it directly.
The single-self-root reachability test, ReturnEvent, ordinary result assignment,
Complete and Close are all real. The descriptor dependencies must survive;
I's identity grants them no extra lifetime. Returning `(self, something)` or a
wrapper containing this open type fails, and `let root=self` would only copy
a resident rather than provide the retained instance Place.

For example caller-side successful Post adoption invokes `w |> admit::contract_type`
using its already established mapping and scoped world view. It computes the
ordinary component updates just described, checks their premises and joins the
other projection Posts at public E publication. It does not replay the selected
callee's body. Caller Pre.check/body Pre.admit/body Post.check use the same
associated behavior at their corresponding existing boundaries. These code
paths, root initialization and world views remain **unconnected design targets**.

### Always-true check cannot forge a world

An ordinary user can form the same type with a check body that returns true.
That is a bool computation, not a source of lifecycle facts. Consider replacing
only the associated bodies in the program above with this malicious variant:

```lang
check_ref = (check_self, w:world::lifetime::):compile -> _:bool:: => {
    true::bool return;
};
admit_ref = [let claim = forged_relation]
    (admit_self, w:(world::lifetime::) ref):compile -> _:unit:: => {
        let w1 = (w, claim) |> adopt_origin::lifetime;
        w1 |> end_boundary_view::lifetime;
        () return;
    };
```

These are **alternative first initializers**, not second writes through consumed
check_ref/admit_ref. Let the actual committed selected arm deliver Q2 from r2,
while forged_relation says Q1 from r1. The true check does not satisfy
adopt_origin's independent source/mapping Pre. That operation rejects before
publication. Constructing a root, choosing these function names or returning a
bool has created none of its missing premises. If a needed observation/consumer is
unconnected, the result is unavailable, never a successful import.

| Other attempted replacement | Why the ordinary safe update is unavailable/rejected |
|---|---|
| Change the origin to an unrelated name, fabricate None/unknown, or retarget the delivered ref | Not derivable from the actual mapped entry/success source; mutable observation copies do not write that source |
| Extend the result Region beyond Q2's validity or change its generation's birth/cut | The supplied boundary cannot change actual lifecycle events or grant survival |
| Remove an inherited shared Color or recolor the earlier interval as exclusive | History and fixed Color relations remain premises; legal interval end is not deletion |
| Restore cropped ancestor access or turn a share path into a ref path | The actual operation domain and non-expansive access Pre forbid amplification |
| Reuse the same world view at another activation/cut/output | The boundary correspondence is different; saving a ref or descriptor grants no later authority |

A harmless always-true contract whose admit derives only permitted consequences
may be useful as an empty/weak check, subject to all ordinary contract checks.
Its own name cannot give it a stronger guarantee. A library that must add a
genuinely new external success fact instead uses compatible unsafe Post after
ordinary Pre and actual success; it takes responsibility for that fact's truth.
The safe counterfeit above cannot opt into those rights merely by returning true.

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

### Ordinary parts of a resource right

Let resource identity Q have full accounting amount 1. A partbox value carries
Q's identity, current validity and amount q as ordinary material; q can be
compile or runtime, without being a business-type parameter. Numbers below
are a design example, not a required machine representation. A **part** is this
resource-right accounting value, not a `T share` borrow instance. Its fraction
amount confers no ref/share access by itself. Consumption and conservation are
the library protocol's laws, independently of the normal share-borrow algebra.

```text
split: consume (Q,1) -> complete (Q,1/3), (Q,2/3)
merge: consume (Q,1/3), (Q,2/3) -> complete (Q,1)
box:   consume (Q,1) -> complete owned box of Q, when all other Pre hold
```

Each action checks actual resource identity and conservation before commit;
same numeric amounts for Q and Q' cannot merge. There is no ordinary clone
that multiplies rights. Moves transport an existing part under its selected
Post, and failed Pre leaves inputs unconsumed. A normal failure return has
its own explicit input/result contract. A non-full amount offers only its
restricted alias capability, not a normal ref. Full amount still cannot
consume to box with an outstanding window, conflicting Color, invalid object
or other unmet ordinary Pre.

One possible option-based topology protocol keeps an optional partbox in each
admitted link slot. Forming a link consumes a part from its current owner and
constructs the complete some branch by its registered ordinary constructor.
Removing a link uses actual option extraction: none has no part; some yields a
complete part through consuming extraction, ends that option identity and leaves any
replacement None through its separate authorized complete write. No partly
extracted parent or implicit copying is permitted.

For example a node's local ledger plus two neighbors' admitted link slots may
hold `(Q,1/3)` each. The topology operation must obtain the slots' actual write
authority, consume their some payloads, install complete none replacements and
merge the three parts of Q. Only after full amount and all other Pre does it
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
