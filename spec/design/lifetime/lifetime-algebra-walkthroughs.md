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

The [operation provenance map](lifetime-policy-and-overload-boundary.md#operation-provenance-map)
classifies the example names as basis observations/actions, ordinary lang
composition, pending consumers and necessary external facts. It is a dependency
map, not a list of required host intrinsics.

Use the following entry guide before implementing an excerpt. The classes
describe responsibilities, not four execution layers; W is an orthogonal
missing-wiring marker, and a facade can combine L with a B/X input and W handoff.

Treat the lang blocks as target source shapes: if a required B/W operation is
unconnected, implement its canonical consumer first; merely supplying a
same-named host function cannot substitute for that consumer or establish
that the example runs.

| What the agent encounters | Correct next action |
|---|---|
| Ordinary lang composition (L) | Once its required operations are connected, write ordinary lang source with normal Pattern/call/Pre/ownership rules; do not replace the whole function with host semantics |
| Fixed basis semantics (B) | Find its canonical owner and implement the smallest missing observation/action consumer of that existing relation; introduce no new ontology or authority |
| Unconnected entry (W) | Follow its LC input/output/boundary/failure contract, connect the actual source producer/consumer and test that slice; missing wiring remains unavailable |
| Real external fact (X) | Keep unsafe Post narrow, after ordinary Pre and actual success, and verify the materializer/facility's real guarantee and maintained interval |

Thus `::selected`, source_frame and available_guard require the existing
selection/control correspondence to be exposed; an agent cannot manufacture
it by writing matching data. initialize_empty_root is a pending general
construction handoff, and adopt_origin composes ordinary mapped operations
over a limited update interface. None is a current implicit compiler builtin.
The helper names are illustrative, not required standard-library names. Their
semantic obligations and failure/authority boundaries are what must be realized.

Each core excerpt below has a local **Origin / Basis / Frontier** note. Origin
distinguishes user/library source, a generated projection and a schematic
expansion. Basis uses the classes above; Frontier identifies the missing semantic
handoff, not just whether its tokens can be parsed. No excerpt currently runs
end-to-end through source E. Recognizing its ordinary syntax is not execution of
its lifecycle operations; in particular the names in selected, construction and
partbox below are not an installed standard-library API.

## 1. Branches, recursion and nested InPlace exits

### Source and selected actions

Assume ordinary `Nat`, `is_zero`, `predecessor`, and the registered bool branch
family. `is_zero` observes an explicit share; that handle's interval ends before
`predecessor` consumes fuel. Consider the admitted family where `r` is an
already formed movable handle of deduced type T. No arbitrary T movement,
implicit borrow or implicit clone is asserted.

**Origin:** user source, design illustration. **Basis:** L control/calls over B
selected Move, capture and return rules; W ordinary execution and LC1–LC3.
**Frontier:** the source carriers do not yet supply this function's lifecycle
projection or an executable recursive call chain.

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

### A headed InPlace with explicit Pre and Post

Headed InPlace syntax permits ordinary lifetime clauses and uses automatic
dependencies; it prohibits an explicit capture list. In the following admitted
handle-source family, T has the actual legal Move/cleanup/return operations
and no other outward effect beyond the stated source transfer. As elsewhere,
any additional outward effects must appear in the full explicit Post.

**Origin:** user-source design illustration. **Basis:** L ordinary head
contracts/control; B actual input/capture Moves, activation-specific return,
result delivery and local compression; W LC2–LC5 and completion propagation.
**Frontier:** head/return syntax carriers exist, but clause evaluation, complete
source execution and lifecycle exit consumers are pending. The helper's
InPlace placement grants no extra capture, lifetime or return permission.

```lang
let headed_step = <T>(self, r:T):runtime -> o:T => {
    let step =
        (inner_self, x:T):runtime -> q:T
            lifetime pre q@..live_with(x@)
            lifetime post q@..from(x@)
            {
                () |> {
                    let local = x;
                    local |> (inner_self return);
                };
            };
    let delivered = r |> step;
    delivered return;
};
```

The outer Ordinary activation is F. step forms a complete InPlace type and
its real callable; invoking it establishes I. Its contextual inner_self
denotes that call's self, and the explicit return consumes its **Return**
projection with actual activation I. It is not the enclosing F's self or an
extra explicit argument. The nested bare InPlace J has its own lexical/Self
boundary, automatically realizes the needed x/inner_self dependencies once,
and is invoked with zero explicit Product entries by (). There is no capture
list or invocation-site recapture.

| Boundary | Actual contract/control/material flow |
|---|---|
| Before I begins | F checks I's mapped Pre, including proposed q's admissible interval relative to x's real target. Actual input/capture Moves occur once; I admits that established Pre |
| J exits toward I | local is the actual transferred handle. Compress J-local name dependencies in its real exiting payload/effect material, retaining source, target validity, region, Color and fixed cleanup. J delivers no normal local result or unit |
| Matching target I | Consume the targeted completion against I's q ReturnPattern delivery relation. Compress I-local outward material and check I's explicit Post on the actual mapped result/effects before successful outward delivery/admission; F imports the guarantee without replaying I |
| F continues | This completed I call really supplies delivered. Its later return to F performs F's own exit compression, Post validation and result delivery |

For a killing handle Move, local is a new sibling generation with the inherited
target/origin, not a borrow of the local x Place. Boundary compression retains
the first actual unknown source and required constraints; it neither rereads
dead locals nor extends target validity. This example requires no SCC collapse
merely because the return crosses J: participation still follows §8.5.

Writing local return instead would infer F, the outermost function in this
reachable segment. That different path would cross both J and I without
delivering q or executing the delivered binding/continuation. One cannot
validate the displayed q-dependent Post by fabricating a normal I result on
such a path; any adopted contract for that path must account for its actual
exit/effects under the existing owners, or remain unavailable. The explicit
target above makes I's own normal result/Post protocol visible while retaining
the intermediate no-result exit. A false/unavailable Pre stops before I's
action, and failed required Post prevents successful delivery/admission at its
gate. Neither failure retries an operation or repairs invalid escape.

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

**Origin:** library-source expansion of compression, not generated host code.
**Basis:** L recursion/extraction/composition; B actual origin, region, Color,
access and delivery observations; W LC3/LC5/LC8. **Frontier:** the required
source observations and boundary consumers are unconnected; these bodies specify
their intended ordinary composition, not a working compress_exit intrinsic.

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

**Origin:** illustrative generated lifecycle projection, including its entry
fragment below. **Basis:** L composition of B sealed selected actions and actual
scope/control material; W LC1–LC3/LC5/LC6. **Frontier:** selected, source_frame and
available_guard have no connected source producer here. User-created lookalike
values cannot stand in for those producers, and no projected call is executed.

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

### SCC collapse at participating closure boundaries

Use the [participation definition](lifetime-policy-and-overload-boundary.md#participation-relation),
not a preassigned "participating" flag. This is a separate admitted structural
case, not a claim that W1's ordinary r itself creates a cycle. Let the actual
reachable segment be:

```text
F (Ordinary target)
  I_A (InPlace)
    I_T (InPlace transit)
      I_B (InPlace, reached targeted return to F)
```

Actual formation supplies n_A in I_A and n_B in I_B, each with its own original
Reg0. The already established structural/source and selected interface material
supplies directed dependencies e_AB: n_A -> n_B and e_BA: n_B -> n_A in one
compatible guarded case, with the ordinary scope/access/control witnesses.
Their establishing actions are in I_B; e_BA is a genuine fine-material
dependency, not a reverse origin edge invented for a Move or a wire added just
because the return crosses a frame. These are edges of the tracking graph,
not a physical owned-object cycle or a new grammar inferred from a recursive
type. If those actual witnesses are missing this case is unavailable.

C={n_A,n_B} is therefore a cyclic SCC: each reaches the other through admitted
edges. Participates(I_A,C,K) follows from n_A's actual formation contribution;
Participates(I_B,C,K) follows from n_B and its established cyclic relations.
I_T forms none of these items and establishes no internal C edge: its own
unrelated captures/names are outside C, and it only propagates the unchanged
completion. Thus Participates(I_T,C,K) is false despite its location on the
same return route. A one-way feeder t_T -> n_A from an I_T-formed item outside
C, without a path back to t_T, would not change that answer. These conclusions refer to these actual activations,
not every future invocation of their source closure types.

| Actual cut | Required transform | Fine material/correspondence still retained |
|---|---|---|
| Return exits I_B | Collapse I_B's outgoing cyclic item/relation contributions into their own original Reg0 bases under current restrictions; compress dying locals | I_A's n_A remains fine in its surviving context, updated by the committed effects. Export the mapped n_B summary and enough guarded C incidence/contribution provenance for the remaining exits; do not collapse all of C globally |
| Same return exits I_T | Ordinary local-name compression; no unrelated SCC collapse | Keep n_A's current fine tracking and the transported n_B summary/correspondence. Transit does not re-own C or reconstruct closed I_B locals |
| Same return exits I_A | Collapse I_A's remaining local cyclic contributions using Reg0(n_A) with current access/region/Color/source/effect constraints | Reuse the valid n_B summary without unfolding it again. The outgoing C view retains actual identity/generation/delivery mappings and all effects; unrelated still-live outer material stays fine |
| Return reaches F | The Ordinary boundary collapses outward SCC material per item and performs its real target delivery | No intermediate unit/result is invented and no target crosses the Ordinary barrier. Already summarized items remain restricted; any F-local fine contribution is handled at this cut |

These are views of the same component under actual transport, not a claim that
an SCC id is immutable across real graph changes. Representation contraction
must retain the correspondence needed to derive I_A's participation after I_B
has collapsed; it cannot erase that evidence, decide I_A is now unrelated or
read dead I_B locals. A transform unable to preserve it stops unavailable at
the mandatory cut. It cannot move or defer that cut to make the algorithm work.

If e_AB instead has guard g=true and e_BA has guard g=false for the **same**
guard occurrence, neither compatible case has that cycle. Combining them into
C would be false participation. Recursive type syntax, repeated regular states
and a merely crossed InPlace independently supply no graph witness either.
At advance summary time these rules transform formed bound roles conditionally;
they do not assert future actual names/activations. No general program
termination or machine TCO proof follows.

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

**Origin:** user source, including the exposed Post component below.
**Basis:** L contract expression/control; B real ref-handle Move, result mapping
and target validity; W LC2–LC5. **Frontier:** the lifecycle clauses and boundary
transform are design targets, with no connected select_one_ref source execution.

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

**Origin:** additional generic user-source design family. **Basis:** L as above,
conditional on B's selected T actions; W LC2–LC5/LC6. **Frontier:** no generic
movement, recursive summary or lifecycle execution is supplied by this example.

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

**Origin:** exposed source contract component, not the entire generated Post.
**Basis:** L ordinary expression over B selected source-choice relations;
W LC4/LC5. **Frontier:** automatic formation/adoption is pending; the display
does not discard the other guarded effect rows.

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

**Origin:** library-authored expansion over generated semantic body material.
**Basis:** L structural transformation and form_post; B formed selected
relations, roles and guards; W LC1/LC2/LC4–LC6. **Frontier:** the semantic body
producer and advance-summary consumer are unconnected, not available from
Raw/Norm AST inspection or a user-constructed lookalike.

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

**Origin:** library-authored contract constructor, schematic source expansion.
**Basis:** L ordinary extraction/closure construction; B current-instance
initialization, OpenHere, Place first-write, self-root delivery and scoped world
authority; W LC3/LC4 and ordinary formation. **Frontier:** in particular
initialize_empty_root and the component adoption interfaces are pending
handoffs. Parsing qualified let/ref/assignment does not execute this bootstrap.
Their authority inputs, permitted increments and distinct publication gates
are specified in the [minimal authorization interface](lifetime-policy-and-overload-boundary.md#94-minimal-authorization-interface-and-operation-rules).

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

The admitted M families here are actual Exit, formed GuardedExitPattern and
the explicit ContractPattern used in W3, each with an ordinary typed
contract_rows extraction provider. They remain distinct: Exit carries
reached-path observations, the others descriptions over established bound roles
and guards (including an unconditional requirement). contract_rows extracts the
corresponding lawfully transformed rows; it does not turn one actual path into
an advance summary, redo the body or infer an arbitrary contract. all_true is
ordinary finite Boolean folding, not a contract evaluator. Its inputs have
these concrete responsibilities:

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

**Origin:** negative user-source variant of those first initializers.
**Basis:** L Boolean/closure construction, B each adopt operation's independent
Pre; W LC3/LC4. **Frontier:** a design rejection target, not an executable exploit
or an implemented diagnostic test.

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

**Origin:** user-source uses, both fragments below. **Basis:** L ordinary ADL
call and registered result extraction; B actual input/result names; W LC2–LC4/LC7.
**Frontier:** neither try_trust nor its credential-producing source consumer is
connected. The compact spelling does not demonstrate executable safeid support.

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

### Writing the trusted API's contracts and body

Here is a minimal **design expansion** of that API. VectorRef, TrustResult,
ProbeOk and ProbeErr denote ordinary registered library types/Patterns. The
success branch of TrustResult contains one ordinary Index; its type need not
change. The declaration fixes that result role before execution. success_id_role
extracts this bound role from the formed return interface; it neither reads a
future out value nor creates a future runtime name. At actual delivery the
ordinary result mapping instantiates it with s and s's current generation.

The three compile expressions construct descriptive rows, then call W2's
expanded form_post to form self-root types with ordinary associated check/admit.
Despite that helper's name, it also constructs the types consumed as Pre.
The row constructors are ordinary library functions over available material.
from_rows is the data-level expansion of the exposed from relation, and
shared_dependency_rows records its actual protected endpoints, region and
shared Color. Neither writes a world. Their normal check/admit operations have
the independent authority premises described in W2.
required_contract_pattern wraps the concatenated Pre rows in a complete typed
ContractPattern; guarded_rows forms that same family with explicit alternatives.
Their contract_rows providers expose those formed descriptions, not inferred
future observations.

**Origin:** library-authored source with schematic contract/interface expansion.
**Basis:** L contract construction, guards and control; B authorized observations,
source/result mapping and legal dependency coloring; X only the successful
validation fact identified below; W LC2–LC5/LC7/LC8. **Frontier:** all three
contract bodies and the runtime body are design targets. In particular
`unsafe post` is pending source notation
([surface status](../../public/normalized-surface-semantics.md#16-lifetime-programs-and-contract-surface-consumer-pending));
this excerpt does not claim an accepted parser production or a working facility.

```lang
let try_trust =
    (self, v:VectorRef$, num:Index::):runtime -> out:TrustResult$
        lifetime pre {
            let s = out@ |> success_id_role::trust;
            let read = v@ |> authorized_validation_read_rows::trust;
            let input = num@ |> legal_input_move_rows::trust;
            let interval = (s, v@) |> within_source_interval_rows::trust;
            (read, input, interval) |> concatenate_rows::lifetime
                |> required_contract_pattern::trust |> form_post return;
        }
        lifetime post {
            let (ok_role, err_role) = out@ |> outcome_roles::trust;
            let s = ok_role share |> success_id_role::trust;
            let source = (s share, v@) |> from_rows::lifetime;
            let shared = (s, v@) |> shared_dependency_rows::trust;
            let success = (source, shared) |> concatenate_rows::lifetime;
            let failure = err_role share |> failure_delivery_rows::trust;
            ((ok_role, success), (err_role, failure))
                |> guarded_rows::lifetime |> form_post return;
        }
        unsafe post {
            let (ok_role, err_role) = out@ |> outcome_roles::trust;
            let s = ok_role share |> success_id_role::trust;
            let validated = (v@, num@, s) |> validated_index_rows::trust;
            let no_fact = () |> empty_rows::lifetime;
            ((ok_role, validated), (err_role, no_fact))
                |> guarded_rows::lifetime |> form_post return;
        }
        => {
            let probe = (v share, num share) |> probe_index::facility;
            probe?
                |> (_, let why:ProbeErr::) {
                    (num, why) |> err::trust return;
                }
                |> (_, let observed:ProbeOk::) {
                    let s = observed |> finish_probe::facility;
                    num |> drop;
                    s |> ok::trust return;
                };
        };
```

The Pre's rows require the actual legal validation-read route, current resource
and input generations, legal input transfers and the proposed successful
result's interval contained in the admitted source interval. The s role can
occur in an entry requirement without executing a future validation. Missing
source/window evidence stops at caller Pre; body admit cannot repair it.
The validator's temporary shared observations end before num is dropped.
ok/err are real complete constructors, and target returns cross only the shown
InPlace branch scopes. Each branch's result is delivered once.

probe_index's successful producer forms a **fresh complete Index**, with the
same numeric content as num, under its selected ordinary construction. It does
not merely return num by Move and rename its origin. Its selected lifecycle
partner establishes the logical borrow while that new name is being formed.
That admitted producer observes num through the explicit shared input and uses
its selected ordinary Index constructor; it is not an implicit copy or a clone
chosen after a failed Move. Its complete-construction Pre remains required.
ProbeOk owns that id plus the temporary probe material; finish_probe consumes
the complete branch, ends only that temporary material and delivers the id once.
The normal failure branch produces no such id. Thus the outer success Post
transports an already established source/shared dependency through actual
handle/value delivery; it does not create it retrospectively.

For clarity, expand the successful producer's lifecycle fragment too. source
is the actual v projection; fresh is the actual one-shot name-formation view
from this selected Index producer; checked is its real successful validation,
not a user-made ProbeOk or a Boolean. The fresh interval is already bounded by
entry Pre. The leaf operations below expose the generic origin/region/Color
responsibilities of real shared dependency formation; their spelling and
grouping are not additional intrinsic requirements.

**Origin:** illustrative generated partner fragment for the selected producer.
**Basis:** L ordered composition; B actual fresh-name formation and authorized
shared borrowing/interval laws; X checked validation only; W LC2/LC3/LC7.
**Frontier:** this formation view and its source operations are pending.

```lang
let probe_success_life =
    (self, source:LifeName::, fresh:FreshNameView::, checked:ValidationFact::):compile
        -> _:FormedNameMaterial:: => {
        let initialized = (fresh, source share) |> initialize_borrow_origin::lifetime;
        let bounded = (initialized, source share) |> initialize_bounded_region::lifetime;
        let colored = (bounded, source) |> begin_shared_dependency::lifetime;
        (colored, checked) |> finish_validated_name_material::trust return;
    };
```

Each initializer checks its own actual formation/source/interval authority,
and begin_shared_dependency requires the ordinary compatible shared Color and
protected-access premises. These are not setters on arbitrary existing names:
there is no rewrite of num's origin, extension of a cut, deletion of Color or
enlargement of access. finish_validated_name_material packages only those
established components and the real compatible validation; copying its inputs'
data cannot reproduce the scoped authority. This is the same B-level action
responsibility as actual shared dependency formation, including logical borrowing
for a non-ref business value, not a trusted safeid constructor for the whole API.
The selected borrow source is v's authorized resource target/protected ranges,
not the local ref-handle Place. Its formed source material retains that distinction;
boundary compression removes the local carrier without extending Q's validity.
The [formation/adoption rules](lifetime-policy-and-overload-boundary.md#94-minimal-authorization-interface-and-operation-rules)
fix the exact permitted components and failure-without-publication behavior.

The unsafe part is precisely the otherwise underivable fact
`ValidatedIndex(Q, generation_Q, s, generation_s, protected_ranges, I)`:
the selected probe actually validated this index against this live resource
generation, and the facility/protocol maintains that condition in I. It does
not assert origin, access or arbitrary world writes. The ordinary success Post
computes the from relation and shared coloring from the authorized source,
actual successful producer and mapped result under their own operation Pre.
Those relations carry the external validation's dependency; no Boolean or
ProbeOk constructor can invent it. Pure numeric comparisons, when available,
remain ordinary lang computation, not unsafe axioms.

probe_index is the narrow actual external action, not an implementation of the
whole try_trust function. Its observation and fresh construction must already
be legal. finish_probe cannot end the protected credential
interval. That interval's invalidators must remain blocked as specified below.
The unsafe description is justified by the real selected probe and its maintained
guarantee, never by observing that this body returned an ok tag. A source-defined
validation with sufficient internal facts can instead use a separately selected
ordinary realization; an unavailable/failed external probe does not retry it.

On ProbeErr, failure_delivery_rows accounts for the actual returned num/error
ownership and other reached effects. It contains no from/shared credential for a
successful id, and the unsafe failure arm is empty. Earlier committed observation
history is retained. At the successful public E action, ordinary Pre, the selected
producer's real result and the compatible guarded ordinary/unsafe Post meet at
the existing joint publication gate. Caller admit consumes the mapped guarantee;
it neither reruns the probe nor manufactures a success payload.

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

### Consuming safeid at range access

Take a Vector of an admitted Word family with a real selected clone that forms
an independent complete Word. VectorShare is its ordinary shared read view.
safeid is the actual ordinary Index delivered by try_trust's successful branch.
Its numeric value may still be unknown to lifecycle computation; its established
name, generation and validation/dependency material are available.

**Origin:** user/library design source, both fragments below. **Basis:** L typed
requirement construction and ordinary projection/clone/call; B actual name/input
mapping, source/generation/Color/interval/access and complete result laws; the
validation was already established by X in try_trust, and this consumer adds no
external fact; W LC2–LC4/LC7/LC8. **Frontier:** requirement providers, the indexed
projection and source execution are pending registrations/consumers. API names
are illustrative; none denotes a compiler shortcut.

```lang
let range_access =
    (self, v:VectorShare$, index:Index::):runtime -> _:Word::
        lifetime pre {
            let source = (v@, index@) |> require_validation_source::trust;
            let generations = (v@, index@) |> require_validation_generations::trust;
            let shared = (v@, index@) |> require_active_shared_dependency::trust;
            let interval = (v@, index@) |> require_access_interval::trust;
            let operations = (v@, index@) |> require_range_read_and_moves::trust;
            (source, generations, shared, interval, operations)
                |> concatenate_rows::lifetime
                |> required_contract_pattern::trust |> form_post return;
        }
        => {
            let element = (v, index) |> element_share_at::vector;
            element |> clone::word return;
        };

let value = (vector share, safeid) |> range_access;
```

The runtime Index enters the compile contract through **index@**, not by
evaluating its integer in the lifecycle world. These are ordinary typed
requirement rows consumed by the same W2 self-root constructor and its
associated check/admit. Their registered ordinary component checks must test:

| Requirement / W2 check component | Actual material which must match |
|---|---|
| Validation source / source_matches | An already established ValidatedIndex for this index name, mapped source choice, actual Q and protected ranges; v's actual read target must be that Q, not an equal container or copied descriptor |
| Generations / source_matches | Current index and resource generations, including the permitted actual input-Move mapping; an old index/resource generation supplies no current credential |
| Shared dependency / color_compatible | Presence of the required live shared dependency on every protocol-protected endpoint, plus its inherited/directed compatibility facts. Historical Color alone or absence of conflict is insufficient |
| Interval / region_fits | This access and its temporary element observation lie in the credential's maintained interval and actual target validity; no invalidating resource/index action has intervened |
| Range/read/transfers / access_is_subset and effects_match | The selected element endpoint is in both the validated range and v's permitted read structure; actual input Moves, temporary borrowing, clone and cleanup satisfy their own Pre |

The row constructors create descriptions, not validation facts. Missing evidence
makes the required check false or unavailable at its actual consumer; a true
user check cannot substitute for any operation's independent Pre.

| Call position | Name/credential mapping |
|---|---|
| Before input transfer | Caller has safeid s/g_s and its existing credential. The formed formal index@ is a bound role. Caller Pre.check uses the ordinary call mapping and the selected input action's guarantees, without manufacturing a future formal generation |
| Actual input Move | The fixed MoveEffect produces the real formal index/g_index once. Only its genuine Post transports the permitted source/validation/dependency relation from s/g_s; equal integer content or a rename grants none |
| Body Pre.admit and indexed call | Admit uses that established mapping. The inner projection's input transfer also needs its own permitted mapping and Pre. Neither boundary reruns try_trust |

element_share_at is the real registered read/projection producer over the
admitted indexed structure. It uses the runtime integer to identify the actual
element and forms its ordinary temporary share under normal access/Color Pre.
It does not compare the integer with length again. clone::word is the selected
ordinary share-to-clone producer, followed by fresh complete Word formation and
terminal Move; it is not a fallback copy. The temporary share ends under fixed
cleanup, and no view of the local index escapes. The omitted automatic Post
retains all reached input/borrow/clone/drop effects and the ordinary owned Word
delivery; it invents no new validation.

Consuming the credential here means using its established relation at Pre;
the actual argument Moves/cleanup determine the names' disposition. This
single-use example returns only Word, so it promises no reusable index output.
The protocol must block every index/resource mutation that could invalidate
the saved fact; const spelling or equality of numbers is no such protection.
Within that maintained interval the prior successful validation justifies the
selected access, so the body has no second numeric bounds test. Other Pre,
addressing and machine costs remain. Foreign/equal integers, stale generations,
expired intervals, cropped-out endpoints or missing shared dependencies stop
before access, without a hidden numeric check or another overload.

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

The separating depth need not be known numerically. For example, a maintained
tree/range relation can show that A1/A2 are confined to distinct descendant
branches of R and cannot meet again, without exposing the prefix length at
which they diverged. That suffices under the remaining Pre. It does not permit
a whole-ancestor operation merely because both ranges use that route. A later
meeting, overlapping ancestor endpoint or unavailable separation invalidates
the argument. Finite regular-state reuse alone is not actual tree separation.

### A two-share partition contract and ordinary program

Expose the original ref as r in this design example so its authority is visible.
A real facade may retain it through a legal ordinary dependency instead.
RootRef, ShareHandle, RangePlan and the result branches are ordinary library
types/interfaces. Observing/moving the existing root **handle** must be legal
while its shares live; that observation grants no exclusive use of the root's
target. The admitted family has two share ranges inside its strict ref tree,
ordinary input/observation/cleanup legality, prospective output validity and
no conflicting surviving dependencies after exactly these shares end. Their
mutual separation is deliberately **not** an entry assumption.

Use three selected actions: prepare descriptive ranges, actually validate
separation, then form the child refs. The validator's compatible unsafe Post
must already be established before the child producer checks its own Pre.
Putting that fact only in partition_refs' outward unsafe Post could not
authorize an earlier internal formation. A normal SeparationOk carries its
range plan as data; the tag is no authority or proof Object.

**Origin:** library-authored contract/control skeleton, all three functions below.
**Basis:** L ordinary row/type formation, range observation, branches and result
construction; B real original-ref/source/input/formation authority, strict access,
regions/Color and common commits; X only actual maintained range separation;
W LC2–LC5/LC7/LC8. **Frontier:** these facades, registrations and consumers are
pending; unsafe post remains design notation without an implemented source
production. No entire partition function is justified as a host intrinsic.

```lang
let validate_separation =
    (self, plan:RangePlan::):runtime -> out:SeparationResult::
        lifetime pre plan@..legal_range_observation()
        unsafe post {
            let (ok_role, err_role) = out@ |> outcome_roles::partition;
            let separated = (plan@, ok_role share)
                |> separated_strict_range_rows::partition;
            let no_fact = () |> empty_rows::lifetime;
            ((ok_role, separated), (err_role, no_fact))
                |> guarded_rows::lifetime |> form_post return;
        }
        => {
            let separated = plan share |> actual_range_test::facility;
            separated
                |> if { plan |> separation_ok::partition return; }
                |> else { plan |> separation_err::partition return; };
        };

let acquire_children =
    (self, r:RootRef$, h1:ShareHandle$, h2:ShareHandle$, checked:SeparationOk::):runtime
        -> _:ChildPair::
        lifetime pre {
            let source = (r@, h1@, h2@) |> require_common_ref_source::partition;
            let authority = r@ |> require_current_root_authority::partition;
            let separated = (r@, h1@, h2@, checked@)
                |> require_established_separation::partition;
            let transition = (r@, h1@, h2@)
                |> require_legal_share_end_and_child_regions::partition;
            let conflicts = (r@, h1@, h2@)
                |> require_no_surviving_conflict::partition;
            (source, authority, separated, transition, conflicts)
                |> concatenate_rows::lifetime
                |> required_contract_pattern::partition |> form_post return;
        }
        => {
            let ranges = checked |> checked_ranges::partition;
            (r, h1, h2, ranges) |> form_child_pair::partition return;
        };

let partition_refs =
    (self, r:RootRef$, h1:ShareHandle$, h2:ShareHandle$):runtime
        -> out:PartitionResult::
        lifetime pre {
            let source = (r@, h1@, h2@) |> require_common_ref_source::partition;
            let authority = r@ |> require_current_root_authority::partition;
            let inputs = (r@, h1@, h2@)
                |> require_observation_moves_and_strict_ranges::partition;
            let transition = (r@, h1@, h2@, out@)
                |> require_prospective_regions_and_other_dependencies::partition;
            (source, authority, inputs, transition)
                |> concatenate_rows::lifetime
                |> required_contract_pattern::partition |> form_post return;
        }
        lifetime post {
            let (ok_role, err_role) = out@ |> outcome_roles::partition;
            let children = (r@, h1@, h2@, ok_role share)
                |> new_child_delivery_rows::partition;
            let returned = (r@, h1@, h2@, err_role share)
                |> returned_share_rows::partition;
            ((ok_role, children), (err_role, returned))
                |> guarded_rows::lifetime |> form_post return;
        }
        => {
            let (r, plan) = (r, h1 share, h2 share) |> prepare_ranges::partition;
            let tested = plan |> validate_separation;
            tested?
                |> (_, let why:SeparationErr::) {
                    (r, h1, h2, why) |> partition_err::partition return;
                }
                |> (_, let checked:SeparationOk::) {
                    (r, h1, h2, checked) |> acquire_children
                        |> partition_ok::partition return;
                };
        };
```

required_contract_pattern forms the same typed descriptive family as W2/W3.
The registered row providers and ordinary check/admit must establish each
requirement from real mapped facts. prepare_ranges threads r and observes the
two handles through actual registered interfaces; it returns their real source,
generation, endpoint/operation and region correspondence. Temporary share views
end before later input consumption. It acquires no new ref or root permission.

actual_range_test is the narrow selected facility action on legally observed
actual ranges. Its success guarantee is only
`SeparatedStrictRanges(source_ref(R), Q/g_Q, A1, A2, I)`, with the actual
input-name/generation/target mapping retained in A1/A2 and maintained through I.
It proves separation of permitted **operations**, including non-reconvergence
in R's strict tree; strings, addresses-as-integers or descriptor states do not
prove it. Neither that fact nor checked_ranges creates R's current authority,
ends any share, removes another dependency or forms a child ref.

The outer ordinary Post transports the actual producer's guarantee. Its ok
rows describe new children sourced from R over their respective actual ranges,
the exact share consumptions/interval ends, new exclusive regions/Color and all
other effects. Its err rows describe current returned r/h1/h2 ownership and
reached observation history, with no child or separation permission. There is
no outer unsafe clause that retrospectively repairs acquire_children.

form_child_pair is the real selected complete result producer, with ordinary
construction/borrow/Move Pre. Its companion can be expanded as follows. frame is
supplied by that actual producer, including its selected source, exact input/
implicit Param mapping, fixed cut, the two current share-interval scopes and
fresh child formation scopes. A user-shaped frame supplies none of these.
All component operations below check their own scope/authority premises.

**Origin:** illustrative generated partner fragment for that selected producer.
**Basis:** L composition; B actual share-ending and new-ref formation scopes,
source, bounded regions and compatible exclusive Color; X separation is already
available; W LC2/LC3/LC7. **Frontier:** this frame and these scoped source
operations are pending. They are responsibilities of normal formation, not
unrestricted lifetime setters.

```lang
let child_pair_life =
    (self, frame:ChildFormationFrame::):compile -> _:ChildPairMaterial:: => {
        let (step, source, old1, old2, left, right) =
            frame |> child_formation_parts::partition;
        let step = (step, old1, old2) |> end_selected_share_intervals::lifetime;
        let left = (left, source share) |> initialize_borrow_origin::lifetime;
        let left = (left, source share) |> initialize_bounded_region::lifetime;
        let right = (right, source share) |> initialize_borrow_origin::lifetime;
        let right = (right, source share) |> initialize_bounded_region::lifetime;
        let (step, left, right) = (step, source, left, right)
            |> begin_exclusive_children::lifetime;
        (step, left, right) |> finish_child_pair_material::partition return;
    };
```

The source in **both** initializers is R's real authorized referent/range
material, never h1 or h2. The fresh scopes belong to new ref names, not renamed
share handles. begin_exclusive_children checks the already established actual
separation, range subset, fixed regions, all surviving dependencies and normal
ref compatibility under its own Pre. The source/region initializers are scoped
to the selected ref-formation action; W3's shared formation rule does not
itself authorize this exclusive case.

These calls stage the prescribed components in **one actual producer's**
unpublished projection storage. At its existing complete-result/joint Post
gate, the admitted old share intervals end and the complete new child pair is
published. No separate partial child publication or historical Color deletion
occurs. Input handle generations follow the ordinary selected Moves and mapping;
the later terminal consumption ends exactly the currently admitted borrow
intervals once. It does not consume an earlier caller generation again.
Result delivery/compression must preserve external referent validity, not
borrow a dying local root-handle Place or extend any lifetime.

| Outcome | Ownership, facts and stopping point |
|---|---|
| Outer caller Pre false/unavailable | No call input transfer or child action; no success delta |
| Real validator returns SeparationErr | The body already received its inputs. partition_err returns their current mapped handles through actual Moves, without terminally consuming their protected share intervals. It publishes no separation fact or child permission; prior observation history remains |
| Validator succeeds, child producer Pre false/unavailable | The established separation fact grants none of the missing root/region/Color/access authority. Stop at that selected producer with no child delta; no retry, hidden reborrow, alternate source or fabricated err/remaining continuation |
| Child producer commits | Consume the listed shares once and publish the complete new refs from R at the prescribed gate, then deliver the actual mapped ok result |

For one input, the pairwise separation requirement and its dynamic test disappear;
the ordinary source/authority/interval/access/Color/conflict/consumption Pre
remain. A statically established separation can use a separately selected
ordinary realization, never a fallback after this validator fails. Exact API,
representation, normalization and caching choices remain engineering work.

### A regular access program, with endpoint-only cropping

Take an already formed recursive Node family with selected legal owned-child
fields left/right and a payload field. Its actual ref-access structure is a
strict tree. An optional observer field watch contains a share and is outside
that exclusive propagation tree. This is an assumption about real formation
and selected access, not an inference from the spelling or recursive type.

For this example, register ordinary operations on a regular descriptor family:
at_root pairs the actual name/target parameters with its original formed
companion; .left/.right advance one admitted Node edge; binary || selects
between those transitions; postfix ~ forms their finite zero-or-more repetition
descriptor; .payload advances to the payload endpoints at those nodes.
These are ordinary operator/ADL registrations, including field::adl, over
descriptive values. They are not strings naming routes or an access interpreter.
The postfix spelling uses the ordinary grammar; the chosen descriptor encoding,
selector packaging and helper signatures remain schematic.

**Origin:** library-authored design source over an actual projected name.
**Basis:** L choice/repetition/field composition and subset cropping; B the
original formed companion, selected field correspondence, actual identities and
ref/share boundary; W LC1–LC3/LC8. **Frontier:** the descriptor registrations,
projected source values and access consumer are unconnected. This is not an
executed traversal of all runtime Nodes or an existing access library.

```lang
let payload_access =
    (self, r_n:LifeName::):compile -> _:LifeAccessView:: => {
        let root = r_n share |> at_root::regular;
        let down = ((root share).left || root.right);
        let descendants = down~;
        let payload_ends = descendants.payload;
        r_n..access(payload_ends) return;
    };

let payload_view = r@ |> payload_access;
```

The finite descriptor denotes zero or more selected left/right steps, followed
by payload. The zero-step case permits the root's payload, not the whole root.
Each walk remains parameterized by its actual Node target/name/generation;
revisiting a descriptor state merges none of those identities. Ordinary
formation supplies that regular structure once; ~ does not execute future
field actions, create new runtime Nodes or prove an arbitrary recursive
lifecycle program terminating.

access intersects the requested payload endpoints/operations with r_n's
currently admitted view. If an earlier crop removed some payload endpoints,
this program retains only the remaining ones; its use of the original
descriptor grants none back. Routing Node/left/right prefixes remain so that
the retained payloads can be reached. No whole-Node read, write, move, destruction
or exclusive ref to an ancestor follows from keeping such a prefix. Historical
Color and the region/source constraints remain. A later actual payload action
must still satisfy its own ordinary Pre at E's frontier.

**Origin:** negative user-source design attempts, both fragments below.
**Basis:** B existing permissions and ref/share propagation, L non-expansive
intersection; W LC3/LC8. **Frontier:** these are rejection targets for future
connected consumers, not present compiler diagnostics.

```lang
let restored = payload_view..access(r@);
restored |> require_whole_root::example;

let beyond_share = r@.watch.payload;
let bad_ref = r@..access(beyond_share);
bad_ref |> require_exclusive_payload_ref::example;
```

The first crop cannot restore whole-root terminal permission; the subsequent
whole-root request fails its own Pre (or the unsupported crop request is
rejected earlier). The second cannot obtain an exclusive endpoint through
watch's share. A legal shared continuation, if registered and independently
authorized, remains shared; it cannot satisfy the shown exclusive demand.
Likewise alias is not a traversable normal edge. Missing field/descriptor
registrations stop unavailable rather than reconstructing them from Node's type.

## 4. Alias, partbox and option

### Root-only alias and a bounded window

Here alias means a library capability for alias writes, distinct from lexical
`let x === PathMaterial`. Initially its normal access tree outside its own
root is empty. Even root replacement requires a real registered default;
spelling `default` or knowing a payload's type supplies no assignment or field
implementation. Registered ordinary field extraction may obtain an ordinary
value, but cannot cross ref/share nodes or smuggle a normal borrow path.

An illustrative window call uses an explicitly formed callback:

**Origin:** user-source design use. **Basis:** L callback/call; B normal ref
formation/escape; X maintained validity/exclusion, not anonymous-name freshness;
W LC3/LC7. **Frontier:** with_ref_window and its escape/window consumers remain
unconnected; the callback's Ordinary return barrier alone does not implement them.

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

### Legal start of the partbox protocol

Before split/merge, the library must obtain a legitimate first part. The
[issuance law](lifetime-policy-and-overload-boundary.md#131-legitimate-initial-issuance-and-source-preservation)
requires actual complete unallocated full-use authority for Q/current
generation, not just its identifier and a quantity. One ordinary design use is:

**Origin:** user-source use, expanded by the production chain below.
**Basis:** L library conversion over B actual full ownership; X only for a real
external acquisition when needed; W LC3/LC7. **Frontier:** issuance is not a
connected source operation.

```lang
let first_part = owned_resource |> issue_full_part::partbox;
```

owned_resource here is the actual result of a lawful full-owner producer or
earlier legitimate full-part-to-box conversion. issue_full_part is ordinary
library code over that real source: its Pre checks current ownership,
unallocated disposition, validity, access and conflicting dependencies; its
successful consuming/construction action transfers that authority into the
complete part with amount 1. It leaves no independently usable old full owner.
Calling it again through an old alias/view does not issue a second part.

Alternatively an explicit trusted acquisition facility may really acquire the
resource and return its full authority, or perform declared acquisition and
initial issuance together. Only its actual successful compatible unsafe Post
supplies otherwise underivable external ownership/validity facts. A failure or
a constructed success tag supplies none. The safe conversion from established
ownership needs no new external axiom merely because its output is a Part.
The issuer's own contract cannot justify its missing source circularly.

Every later part must retain this actual source chain through legitimate
consumed split/merge inputs and ordinary transport. Arithmetic fields are not
the chain: merge cannot count the same portion twice through equal copies or
Preserve views. The normal Part data type/layout and any private accounting
carrier remain library choices; actual allocation is not a builtin type test.

For a negative design case, suppose ordinary user construction is admitted to
produce the same classifier and apparent fields as a real full part:

**Origin:** negative user-source construction. **Basis:** B actual source/owned
capability, L data construction; W LC3/LC7. **Frontier:** this is an independent
authority-rejection target even if the ordinary type/constructor is admitted.

```lang
let fake:Part:: = (resource_id, full_amount) |> user_part_constructor;
fake |> into_box::partbox;
```

Take full_amount=1 and resource_id=Q. Even if fake has equal Core/content to a
legitimate part, it has no actual issuance/consumed-input authority. into_box
fails that independent Pre, or the requiring observation is unavailable. It
does not become valid because Type(fake)=Part, the numbers add up, or a user
also constructs ValidationOk. A real registered clone's Post must justify its
result; no copy silently creates another independently consumable allocation.
This tests authority separately from a possible earlier type/access rejection.

The two inputs of the composition program below are actual legitimate parts
derived from such an issue, not fake data records. Its early rejection and
later-failure ownership rules continue to apply. All issuance consumers and
these negative cases remain unconnected design targets.

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

### Ordinary partbox composition program

#### A minimal production chain inside the library

Before the higher-level composition, expand its producers. This **one possible
serialized library representation** keeps the actual owned box in an owned
private State, alongside a finite allocation table. A Part owns a unique ordinary
allocation cell and its amount; the table records that cell's actual identity
and current allocation. The cell is an ordinary complete owned Object, not a
proof Object, fractional intrinsic or integer key. Its identity/target and lawful
Move mapping distinguish it from an equal-valued fresh cell. A table row is
descriptive data, not a second owner of the cell or resource.

State, Part, Row and their field/constructor interfaces are illustrative closed
library structures. Their actual registered access rules protect the held box
and table; the word private in a path grants no privilege. The functions below
are admitted internal calls with the actual owned State and Parts. In particular
their independent entry Pre establishes:

- the State's lawful held box/current resource and allocation invariant;
- each supplied Part's actual owned cell, source chain, matching live entry and
  admitted current generation/Move mapping, with no outstanding conflicting use;
- distinct entries for merge; valid split amounts; and every ordinary validity,
  window, Color, cleanup and complete-construction requirement.

These premises are checked from the real inputs and established library state,
not from these functions' not-yet-executed Post. Cells/rows observed through
temporary shares are released before their owners are consumed. A saved share,
equal Part-shaped value or copied row cannot supply the owned cell. A fresh cell
made by a user cannot match the already installed actual allocation. Updating
the table additionally requires the actual private State; metadata access alone
does not expose that authority.

**Origin:** library-authored schematic expansion, all five functions below.
**Basis:** L finite-table operations, exact amount computation and consuming
Pattern/constructor calls; B actual owned inputs, fresh complete cell formation,
ordinary identity/access/Move and publication; W LC1/LC3–LC5/LC7. **Frontier:**
these source library definitions and their consumers are unconnected. Fresh cell
formation is ordinary complete Object formation, not a proposed partbox builtin.
The external-resource validity variation after the code is the only X input here.

```lang
let fresh_allocation_data =
    (self, amount:Amount::):runtime -> _:AllocationData:: => {
        let cell = () |> unit_box::private;
        let row = (cell share, amount share) |> allocation_row::private;
        let part = (cell, amount) |> part::private;
        (part, row) |> allocation_data::private return;
    };

let issue_full_part_core =
    (self, owned:ResourceBox::):runtime -> _:Issued:: => {
        let full = () |> one::amount;
        let (part, row) = full |> fresh_allocation_data;
        let rows = row |> singleton_table::private;
        let state = (owned, rows) |> state::private;
        (state, part) |> issued::private return;
    };

let split_core =
    (self, state:State::, p:Part::, wanted:Amount::):runtime
        -> _:SplitResult:: => {
        let q = p share |> quantity::private;
        let rest = (q, wanted share) |> subtract::amount;
        let (held, rows) = state |> state_fields::private;
        let (old_cell, old_amount) = p |> part_fields::private;
        let remaining = (rows, old_cell share) |> remove_exact_entry::private;
        old_cell |> drop;
        old_amount |> drop;
        let (left, left_row) = wanted |> fresh_allocation_data;
        let (right, right_row) = rest |> fresh_allocation_data;
        let next_rows = (remaining, left_row, right_row) |> insert_two::private;
        let next_state = (held, next_rows) |> state::private;
        (next_state, left, right) |> split_result::private return;
    };

let merge_core =
    (self, state:State::, a:Part::, b:Part::):runtime
        -> _:MergeResult:: => {
        let qa = a share |> quantity::private;
        let qb = b share |> quantity::private;
        let total = (qa, qb) |> add::amount;
        let (held, rows) = state |> state_fields::private;
        let (cell_a, amount_a) = a |> part_fields::private;
        let (cell_b, amount_b) = b |> part_fields::private;
        let rest = (rows, cell_a share) |> remove_exact_entry::private;
        let rest2 = (rest, cell_b share) |> remove_exact_entry::private;
        cell_a |> drop;
        cell_b |> drop;
        amount_a |> drop;
        amount_b |> drop;
        let (merged, row) = total |> fresh_allocation_data;
        let next_rows = (rest2, row) |> insert_one::private;
        let next_state = (held, next_rows) |> state::private;
        (next_state, merged) |> merge_result::private return;
    };

let into_box_core =
    (self, state:State::, full:Part::):runtime -> _:ResourceBox:: => {
        let (held, rows) = state |> state_fields::private;
        let (cell, amount) = full |> part_fields::private;
        let empty = (rows, cell share) |> remove_exact_entry::private;
        empty |> drop;
        cell |> drop;
        amount |> drop;
        held return;
    };
```

Here the registered result extractors expose AllocationData/Issued/SplitResult/
MergeResult as the shown complete Products; state_fields and part_fields are
**consuming** extraction. They end each old complete parent and yield complete
children. There is no read of an already extracted State, partial resident,
copy of held, or later recovery of an old Part. unit_box forms only a fresh
owned unit cell. allocation_row observes its real identity and amount, and
part packages that actual cell. fresh_allocation_data on its own creates no
resource right: only lawful insertion into the actual State assigns a portion
of its held resource. It must not be exported as an issuer bypass.

The private table functions are ordinary finite collection programs: removal
matches and removes one exact admitted live entry; insert_one/insert_two add
the supplied complete rows after checking freshness. They do not assign
fractions, manufacture owner facts or certify a sum. This example restricts
their domain to prechecked finite tables/amounts with total selected operations.
Implementations must expose those ordinary operations and their failure/overflow
contracts, not turn the entire State rewrite into a host allocation oracle.

The invariant and conservation calculation can be followed without an opaque
merge/split operation:

| Actual transition | Rows removed / inserted by the lang body | Resource authority |
|---|---|---|
| issue | Empty allocation disposition to one actual cell with amount 1 | Consumes the real unallocated box into State; no old independent owner remains |
| split | Remove p's one actual entry q; insert two fresh cells with wanted and q-wanted | q = wanted + (q-wanted); the same held box moves once into the next State |
| merge | Remove a/b's two distinct actual entries qa/qb; insert one fresh cell with qa+qb | Total is unchanged; two views of one cell fail entry Pre before this body |
| into_box | Its entry Pre requires exactly the one live full entry, not merely a numeric sum of 1; remove it | Consumes State/full and returns the actual extracted held box, not a box built from Q or an amount |

The stronger singleton condition belongs to this small representation; it also
prevents silently discarding other live zero-amount allocations. Other designs
may handle those explicitly. Every full conversion still checks outstanding
windows/dependencies and all normal resource/Color/access Pre. None of these
functions forms a ref; the caller's later box ref needs its actual bound Place.
The private State is passed explicitly to expose dataflow. A public facade may
instead retain it through an admitted ordinary dependency, but must preserve its
single real ownership and the same entry/source checks. This serialized witness
does not establish arbitrary distributed storage, graph collection or concurrency.

When the lawful held box already establishes the needed validity, this entire
chain needs no unsafe fact. For an external-resource realization, a separately
selected legal validator may establish only that actual Q/current generation g
remains live throughout the finite transition interval I. Its real success Post
is X; source-defined control branches on its ordinary success/failure result
**before** calling the prechecked core. It grants no State/Part ownership,
conservation, distinctness, exclusion or ref access. If exclusion is external
too, it needs its own actual maintained guarantee; a ValidationOk tag proves
neither. No unsafe Post repairs the validator's own observation Pre.

Initial ownership thus comes from the actual owned_resource producer (or the
separately declared real external acquisition), not from fresh_allocation_data
or an issuer's self-justifying contract. Automatic Post uses each function's
formed operations: actual entry removal/consumption, fresh cell allocation,
same held-resource delivery and preserved amount/dependency rows. LC5 maps
those rows across local exits; W2 constructs their associated check/admit; LC4
checks/adopts the mapped summary at public E's joint success gate. No extra
issuer, permission publication or evaluator runs between these steps. A forged
(Q,1), cell, row or ValidationOk has no such actual chain.

Precheck/planning failure retains or explicitly returns the currently owned
inputs. This core is only the admitted success family; a later selected failure
cannot restore already consumed generations or pretend earlier commits did not
occur. A different failure-capable implementation must deliver its actual
intermediate complete ownership and cleanup without publishing a success Post.

#### Composing the production chain

This **design illustration** is a small library composition, not a builtin
resource type or executable coverage. Part, snapshots, plans and result variants
are ordinary library Objects with registered Pattern/constructor interfaces.
The admitted Amount family has exact nonnegative addition and subtraction on
ordered operands; a bounded representation must supply its ordinary overflow/
failure contract. Sharing/cloning an amount or identity snapshot copies no part
or resource authority. Interface names below are schematic.

The first ordinary function calculates data only. part_snapshots supplies actual
resource identity, generation and quantities observed from the two input parts;
it does not create rights from those fields. same_resource compares actual
identity, not equal content or a path string. Both Boolean tests and arithmetic
are library lang code, with no unsafe admission:

**Origin:** library-authored planner and public composition, both functions below.
**Basis:** L planning/control and the expanded production chain; B actual
Part/State access, consuming formation and result delivery; X only the declared
external validity input if needed; W LC3–LC5/LC7. **Frontier:** these APIs and
their source consumers are pending. The public facade's omitted State must be a
real admitted dependency of the actual selected call, threaded as above, not
an implicit authority recovered from Part's type, amount or identifier.

```lang
let plan_repartition =
    (self, view:PartPairView::, wanted:Amount::):runtime -> _:PlanResult:: => {
        let (qa, ga, na, qb, gb, nb) = view |> pair_fields::partbox;
        let same = ((qa share, qb) |> same_resource::partbox,
                    (ga share, gb) |> same_generation::partbox)
            |> both::bool;
        same
            |> if {
                let total = (na, nb) |> add::amount;
                let fits = (wanted share, total share) |> not_greater::amount;
                fits
                    |> if {
                        let rest = (total share, wanted share) |> subtract::amount;
                        (qa, ga, total, wanted, rest) |> ready_plan::partbox return;
                    }
                    |> else { () |> bad_amount::partbox return; };
            }
            |> else { () |> different_resource::partbox return; };
    };

let repartition_or_box =
    (self, a:Part::, b:Part::, wanted:Amount::):runtime -> _:PartResult::
        lifetime pre (a@, b@)..owned_parts()
        => {
            let view = (a share, b share) |> part_snapshots::partbox;
            let plan = (view, wanted) |> plan_repartition;
            plan?
                |> (_, let why:RejectedPlan::) {
                    (a, b, why) |> rejected::partbox return;
                }
                |> (_, let ready:ReadyPlan::) {
                    let status = ready share |> validate_for_plan::partbox;
                    status?
                        |> (_, let why:ValidationFailure::) {
                            (a, b, why) |> rejected::partbox return;
                        }
                        |> (_, let _:ValidationOk::) {
                            let merged = (a, b) |> merge_parts::partbox;
                            let full = merged share |> is_full::partbox;
                            full
                                |> if {
                                    let owned = merged |> into_box::partbox;
                                    owned |> boxed::partbox return;
                                }
                                |> else {
                                    let (left, right) =
                                        (merged, ready) |> split_ready::partbox;
                                    (left, right) |> repartitioned::partbox return;
                                };
                        };
                };
        };
```

The sum branches and `?` are ordinary registered extraction, with normal target
return through InPlace scopes to the corresponding Ordinary function. Neither
`?` nor a rejected branch supplies implicit error propagation. owned_parts is
an ordinary lifecycle contract expression over established input capabilities,
not a type-name-based ownership assertion or an unsafe Pre. Its entry check,
normal Param/input Move and all inner operation Pre precede their mutations.

| Step | Library lang work | Basis/consumer or trusted boundary |
|---|---|---|
| Entry and snapshots | Check the mapped owned-input contract; observe the real Q/generation/amount fields through legal share views; end those temporary views before consuming the parts | Ordinary call, access/borrow and lifecycle observations; no rights from copied metadata |
| Plan | Compare Q/generations; calculate total=na+nb, wanted<=total and rest=total-wanted; construct ReadyPlan or a rejected ordinary value | Ordinary identity/amount/Pattern operations. The plan describes an allocation, not permission to execute it |
| Resource validation | Invoke the selected validate_for_plan realization on Q/generation and its plan; branch on its ordinary result | Already available owned-state facts may suffice in a source-defined realization. The illustrated external-resource realization uses a narrow trusted check and compatible unsafe success Post for otherwise underivable validity/interval facts, never for the arithmetic or whole partbox function |
| Merge | merge_parts checks the two actual capabilities, same-resource/current-generation and conservation; consumes a/b once and forms a complete merged part | Ordinary library consuming extraction/accounting/construction over its privately admitted resource interface; common Pre/commit/Post. These internals must be decomposed under the provenance map, not installed as an opaque partbox intrinsic |
| Partial split | split_ready consumes merged under its own Pre, uses the already computed wanted/rest allocation and forms two complete parts of Q | Ordinary library operations preserve total quantity and actual rights; no duplicate part, partial Object or new ref/share capability |
| Full conversion | is_full observes the amount only. into_box additionally checks actual ownership, validity, no conflicting windows/dependencies and every other normal Pre, then consumes the full part and forms the complete ordinary box | This is the first normal-world owned box in the function. If the chosen representation needs trusted alias-to-value/ownership facts, only that narrow facility supplies them; full amount or ValidationOk alone cannot |
| Return and Post | Construct rejected/boxed/repartitioned, transfer actual results once, compress actual local names and validate the corresponding conditional summary | Ordinary result formation, LC5 advance summary/actual-exit relation, LC4 check/admit and public E's joint Post |

For the external-resource realization, the validator's declared successful Post
binds the fact to the actual Q/generation and finite composition interval. Its
facility must maintain the condition until the consuming operations finish,
including constraints on invalidating concurrent/window actions. The check
itself needs an ordinary legal observation route; unsafe Post cannot authorize
an invalid dereference retroactively. An alternative source-defined validator
with sufficient existing facts is a separate ordinary realization, not a retry
after this one's failure or unavailable observation.

ValidationOk is an ordinary result branch, not a user-supplied proof Object or
a right minted by its constructor. Forging the same data yields no established
Post. merge_parts, split_ready and into_box independently require actual input
and mapped-world authority. The boolean full likewise cannot authorize box
formation. If its selected conversion fails, execution does not fall back to
the partial branch or another overload.

Rejected planning/validation returns the currently owned input parts through
ordinary result Moves; it does not undo or resurrect their caller-side input
generations. No merge/split rights have been consumed on those branches. The
demonstrated success path assumes each selected operation's actual Pre holds.
If a later operation fails after merge committed, any normal failure protocol
must describe the **current** merged/intermediate ownership and cleanup; it
cannot pretend a/b were restored. Missing consumers/facts are unavailable and
no success Post is published.

On the partial branch the caller receives parts and at most their separately
admitted restricted alias interfaces. On the full branch it receives the owned
box as an ordinary result. A subsequent `box ref` requires the caller's actual
bound Place and normal borrow Pre; this function does not return a ref to its
own local owned variable. No normal ref arises from an amount calculation.

The omitted explicit Post uses the already formed structural descriptions and
exposed contracts of these library calls to form a guarded summary before use,
as in W2. Runtime execution validates its reached branch. The failed-validation
summary carries no success fact; successful accounting rows preserve Q, quantity,
single consumption and dependency/Color history; box formation has its own
selected producer Post. The complete sequence remains ordinary lang composition
plus narrow justified facilities. None of the source programs or consumers in
this illustration is connected by this documentation revision.

### Relinking topology without accessing payload

This finite serialized example moves N from L--N--R into an existing C--D edge.
Use five distinct actual nodes and six distinct admitted link-slot roots.
The library convention is that an incoming link to a node carries a legitimate
Part of **that target node's resource**. Initially N's local ledger and the
links L.next/R.prev each own one third of Q_N. N.left/N.right own actual Parts
of Q_L/Q_R; C.next/D.prev own actual Parts of Q_D/Q_C. No quantity or node id
can substitute for one of those actual Parts.

| Actual slot root | Initial complete Option | Final complete Option |
|---|---|---|
| L.next | Some(N, Part of Q_N) | Some(R, the Part of Q_R taken from N.right) |
| R.prev | Some(N, Part of Q_N) | Some(L, the Part of Q_L taken from N.left) |
| N.left | Some(L, Part of Q_L) | Some(C, the Part of Q_C taken from D.prev) |
| N.right | Some(R, Part of Q_R) | Some(D, the Part of Q_D taken from C.next) |
| C.next | Some(D, Part of Q_D) | Some(N, a new third of Q_N) |
| D.prev | Some(C, Part of Q_C) | Some(N, the other new third of Q_N) |

The caller supplies the actual State_N, N's local Part, and a RootSet containing
restricted aliases **directly to those slot roots**. They were obtained through
the library's real registered topology interface. Writing node_alias.next would
require its own registration and authority; the type's fields or a path string
do not obtain these aliases. The links' target ids are ordinary immutable routing
data; observing/cloning one gives no node payload capability.

The Edit session is ordinary facility state for a bounded serialized topology
edit. Entry Pre requires actual root-write capabilities, real distinct slot
identities, valid generations, lawful Part sources and the admitted complete
replacement/cleanup operations. The protocol keeps the observed topology and
resource validity stable and excludes competing root edits and observations
requiring a completed topology throughout the session. Where this relies on
external behavior, only the necessary real maintained facts are X. The session
forms no normal ref.

The selected registered alias-root operation exchange_link_root has an explicit
contract: under its own ordinary Pre it consumes the supplied complete replacement,
replaces the **root Option resident**, and returns the complete old Option plus
the transferred root/session handles. Old-result delivery, same-Type replacement,
resident compatibility, cleanup and lifetime must all be legal. Failed Pre
changes nothing at that action; no generic swap is inferred from alias or Option
shape. This is an illustrative library registration to implement, not an installed
operation. Its prechecked take/fill realizations additionally require the actual
Some/None resident, respectively, before mutation.
In particular, transferring the old Option and its Parts requires the actual
protocol authority over that resident. Root-write capability and registration
alone are insufficient. A source-defined realization can use the established
ownership of Parts previously installed through this same closed protocol.
If an opaque external realization needs a trusted resident-transfer guarantee,
it must establish exactly that lawful transfer under real validity/exclusion;
it cannot manufacture another Part or infer ownership from equal slot contents.

**Origin:** library-authored schematic root helpers and relink program below.
**Basis:** L ordinary Option/Link extraction/construction, planning, control and
the expanded merge_core/split_core; B actual root access, complete replacement,
owned Part/State/resident-transfer sources, fixed cleanup and delivery; X only
necessary maintained session/transfer facts if otherwise underivable;
W LC3–LC5/LC7. **Frontier:** the
root registrations, edit protocol and all source consumers are pending. These
definitions neither install a new alias primitive nor execute a graph algorithm.

```lang
let take_link =
    (self, edit:Edit::, root:LinkRootAlias::):runtime -> _:TakenLink:: => {
        let empty = () |> none_link::topology;
        let (edit, root, old) =
            (edit, root, empty) |> exchange_link_root::topology;
        let branch:SomeLink:: = old?;
        let (target, part) = branch |> some_link_fields::topology;
        (edit, root, target, part) |> taken_link::topology return;
    };

let put_link =
    (self, edit:Edit::, root:LinkRootAlias::, target:NodeId::, part:Part::):runtime
        -> _:FilledLink:: => {
        let replacement = (target, part) |> some_link::topology;
        let (edit, root, old) =
            (edit, root, replacement) |> exchange_link_root::topology;
        let _:NoLink:: = old?;
        (edit, root) |> filled_link::topology return;
    };

let relink =
    (self, state:State::, local:Part::, edit:Edit::, roots:RootSet::,
     finish:bool::):runtime -> out:RelinkResult::
        lifetime pre (state@, local@, edit@, roots@, out@)..relink_inputs()
        => {
            let (edit, roots, plan) =
                (edit, roots, state share, local share) |> inspect_relink::topology;
            plan?
                |> (_, let why:NotRelinkable::) {
                    (state, local, edit, roots, why) |> unchanged_relink::topology return;
                }
                |> (_, let ready:RelinkPlan::) {
                    let (ln, rp, nl, nr, cn, dp) = roots |> root_set_fields::topology;
                    let (edit, ln, _, from_l) = (edit, ln) |> take_link;
                    let (edit, rp, _, from_r) = (edit, rp) |> take_link;
                    let (edit, nl, _, to_l) = (edit, nl) |> take_link;
                    let (edit, nr, _, to_r) = (edit, nr) |> take_link;
                    let (edit, cn, _, to_d) = (edit, cn) |> take_link;
                    let (edit, dp, _, to_c) = (edit, dp) |> take_link;
                    finish
                        |> if {
                            let (left, right, node, c, d, q_local, q_c) =
                                ready |> relink_plan_fields::topology;
                            let (edit, ln) = (edit, ln, right, to_r) |> put_link;
                            let (edit, rp) = (edit, rp, left, to_l) |> put_link;
                            let (state, two_parts) = (state, local, from_l) |> merge_core;
                            let (state, full) = (state, two_parts, from_r) |> merge_core;
                            let (state, new_local, rest) = (state, full, q_local) |> split_core;
                            let (state, for_c, for_d) = (state, rest, q_c) |> split_core;
                            let node2 = node share |> clone::topology_id;
                            let (edit, cn) = (edit, cn, node, for_c) |> put_link;
                            let (edit, dp) = (edit, dp, node2, for_d) |> put_link;
                            let (edit, nl) = (edit, nl, c, to_c) |> put_link;
                            let (edit, nr) = (edit, nr, d, to_d) |> put_link;
                            let roots = (ln, rp, nl, nr, cn, dp) |> root_set::topology;
                            let roots = (edit, roots) |> finish_topology_edit::topology;
                            (state, new_local, roots) |> relinked::topology return;
                        }
                        |> else {
                            let roots = (ln, rp, nl, nr, cn, dp) |> root_set::topology;
                            (state, local, edit, roots, ready,
                             from_l, from_r, to_l, to_r, to_d, to_c)
                                |> paused_relink::topology return;
                        };
                };
        };
```

TakenLink/FilledLink and the core results have registered consuming Product
views as shown. SomeLink/NoLink are the ordinary declared one-layer Option
branches: old? consumes the complete old Option; some_link_fields consumes
the complete Some branch and delivers its real target data and owned Part.
The normal NoLink branch is also consumed. No partial aggregate or empty
resident is left: each take has already installed a complete None. put_link
constructs a complete Some, then separately commits its admitted root replacement.
The helpers' entry Pre guarantees Some/None; a mismatch is a selected failure,
never a retry or implicit error propagation through ?.

inspect_relink threads the actual aliases/session through registered **root**
observations, checks the reciprocal identities in the table and the actual
local allocation assumptions, and computes the two one-third split requests.
Its plan is ordinary data; it grants no access. The session and independent
operation Pre maintain those observations while editing. The admitted family
has total finite amount/constructor/table/root operations after those checks,
with temporary snapshot shares ended before consumption. A wider fallible or
concurrent realization needs its own explicit current-ownership protocol.

The first two takes acquire the actual Q_N Parts from N's two old neighbors.
The other takes recover the independently sourced Q_L/Q_R/Q_C/Q_D Parts.
put_link moves those existing neighbor Parts to their new slots. Only Q_N's
allocations are merged/split through State_N; the other resources' ledgers
undergo no allocation change, and their actual Move/source obligations remain.
The full Q_N Part is immediately repartitioned; no into_box, payload ref,
payload read or alias-to-ref window occurs. Topology-root permission alone
therefore supplies no payload access. finish_topology_edit checks the completed
local protocol, ends its actual edit exclusion and transfers the root handles;
future edits still need their own Pre.

finish is an ordinary bool decision reached after detachment. Its false arm
demonstrates a normal paused result with **current** ownership, rather than
claiming restoration. The returned PausedRelink owns the active edit/session,
all six now-None roots, the local Part, all six taken Parts, current State and
the plan. Caller Pre/result delivery must admit carrying that session within
its existing finite interval. The caller must resume/repair or use the library's
explicit cleanup before that interval ends; the protocol cannot release
exclusion while leaving an externally usable incomplete topology.

| Exit or stopped frontier | Actual slots and ownership |
|---|---|
| Planning rejects before a take | No slot changed; result returns the current State/local Part/session/root handles. The Option residents still own their Parts |
| Normal paused return after the six takes | All six slots contain complete None; PausedRelink owns the seven Parts (six taken plus local) and active edit/session. It asserts no completed relink |
| Completed relink | The six Some residents own exactly the table's final Parts; result owns current State_N/new local Part/root handles. The edit session has ended |
| Selected rejection/unavailable operation after an earlier commit | That action adds no delta; earlier None/Some writes remain. Detached Parts stay with actual current locals or the established remaining continuation, according to fixed cleanup. After a merge/split, the current merged/split Parts replace the old inputs. No fabricated paused value, restored generation or rollback is allowed |

The last row is not an additional normal failure branch in this prechecked
family. Supporting such recoverable mid-edit failures requires a real complete
repair/result carrier for the current roots, Parts and session, with maintained
exclusion and actual cleanup; missing wiring does not invent that carrier or
a residual. At each real stop, the ownership above is what the consumer must
preserve, rather than returning stale caller inputs.

The omitted implemented-body Post is formed before use from the selected
root-operation, extraction, accounting, branch and delivery descriptions.
It retains three correlated outcomes: unchanged planning rejection, paused
None roots with all detached ownership/session obligations, or the completed
table and redistributed allocations. All include reached generation, root-write,
dependency and Color history; none has a payload capability effect. Actual
exits compress local carriers and check their corresponding summary arm; W2's
ordinary constructor forms check/admit, and LC4/public E consumes the mapped
Post at the existing joint gates. A normal paused return describes its real
earlier effects, while a rejected semantic action publishes no delta at its
own gate. The whole relink is several ordinary commits, not a new multi-slot
atomic transaction. This proves only the stated finite protocol obligation,
not arbitrary graph safety, collection or reclamation.
