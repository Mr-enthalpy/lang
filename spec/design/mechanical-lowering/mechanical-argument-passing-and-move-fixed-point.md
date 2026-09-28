# Mechanical Argument Passing and the Move Fixed Point

**Status: Canonical semantics; source consumers, checker and physical lowering
remain unimplemented.** The [lifetime owner](../lifetime/lifetime-policy-and-overload-boundary.md)
owns Killable, MoveEffect, Movable and Pre/Post on the same continuation.

## 0. Canonical pass-action core

Move(x), clone(x), share(x), rebind(x) and CopyConstruct(x) below are semantic
metanotation, not callee-first source syntax. Source calls retain P |> E or P E.

```text
CanonicalMechanicalPassCore:
  Pass = Move
  move(move(x)) = move(x)

CopyConstructExpansion (only for a selected copy-derived route):
ordinary source : T:
  source -> share(source) -> selected clone -> fresh complete result -> Move(result)
source : T ref | T share:
  source -> rebind(source) -> selected clone -> fresh complete result -> Move(result)

copy-derived route = selected share/rebind + clone realization + terminal Move
CopyConstruct = compact name for that ordinary realization before Move
```

CopyConstruct is not an opaque primitive or a second terminal pass. Copyable
states that the appropriate clone realization can succeed; it selects no
default copy pass. There is no DefaultPass(x) in {move, copy} judgment to
complete. Failed movement permission does not silently try cloning.

The selected clone owns its lifecycle post, including origin; there is no
universal origin(copy_result)=source law. The source is not pre-moved.

```text
ProducedMode(source) = mu_source
PolicyMode(destination) = mu_destination
Transfer(source, destination, Move)
  preserves ProducedMode(source) = mu_source
  installs the destination under mu_destination
```

TransferToDestination is the ordinary-binding specialization, not another
transfer ontology.

## 1. Purpose

```text
source argument -> ordinary evaluation and explicit operations
  -> ordinary selected realization, if required by the consumer
  -> complete result or already formed borrow handle
  -> terminal Move with its fixed effect and frontier Pre
```

Raw AST and syntax-directed normalization preserve source shape. Resolution,
ordinary candidate selection and E determine semantic actions before physical
lowering. An ABI or optimizer cannot choose cloning or borrowing for the language.

## 2. Source operations and terminal transport

Explicit ref/share constructs ordinary borrow Objects whose handles travel
through Move. Copy-derived syntax requests share/rebind plus ordinary clone;
its fresh complete result also travels through Move. These are producer paths,
not different terminal pass kinds. An implementation carrier called in or
default-pass adds no semantic choice or implicit borrow pass.

## 3. Explicit operations are preserved

```lang
arg |> move
arg |> ref
arg |> share
arg |> copy
```

Each written operation retains its ordinary meaning and Pre. Adaptation cannot
rewrite an explicit move to clone, ref or share. Copyable authorizes none of
these rewrites. Copy-derived use requires its selected clone realization;
ref/share requires borrow authority; Move requires frontier Movable.

## 4. Terminal movement is not default move/copy selection

Every legal argument path terminates in Move. Any preceding realization is
selected under ordinary lookup, Pattern, Policy, access and capability rules,
using ordinary preference or ambiguity. It is not a backend choice or fallback
after sealed failure. Consumer wiring remains pending; this ontology is closed.

## 5. Uniform instance participation

Type, rank, namespace, meta, Pattern and verification material do not form a
non-value pass-through category. Every transported Object instance follows
the same pass-action and lifecycle relations. Stage affects observation and
readiness, not whether a lifecycle subject exists.

```text
Killable_K(n) != Movable_K(n,m)
MoveEffect_K(n,m) ∈ {Kill, Preserve}
```

The lifetime owner defines the narrow Preserve proof and the frontier Pre.
A globally surviving type instance and a meta-local type temporary may have
different effects despite equal values. Nonkillability does not grant copy,
borrow or movement capability. Movable does not imply Kill. Preserve is a
legal Move effect, not clone, copy fallback or failed movement:

```text
MoveEffect = Preserve does not invoke clone
Preserve Move != copy-derived route
copy-derived route creates a fresh complete object before terminal Move
```

## 6. Move is the fixed point

This is the core of the document. The central axiom of pass normalization:

```text
T move == T
rank move == rank
move(move(x)) == move(x)
```

`move` is not a type constructor. It does not produce a new type value such as
`T move`, it does not change rank, and it does not change a classifier. It is the selected transfer into the destination with its predetermined
Kill or Preserve effect.

Therefore pass normalization must not recursively produce:

```text
T move move
rank move move
borrow-of-borrow-of-borrow
```

Once an action lands on `move`, normalization terminates.

The same fixed point applies to callable lookup. Moving a caller of type `T`
does not create `move::T`; its call entry is still resolved under `T`.
`ref(x)` and `share(x)` are different because they first construct borrow
objects of exact types T ref and T share. Each uses the matching () entry of
its own complete type/callspace, never a candidate adapted from T's entry.

## 7. Ordinary producer paths lead to move

These ordinary producer paths all terminate in Move (semantic metanotation):

```text
move(x):
  require Movable_K(x,m)
  perform the predetermined MoveEffect_K(x,m)
  transfer into the argument slot

copy(x):
  tmp = CopyConstruct(x)
      ~= shared_view := share(x); clone(shared_view)     when x : ordinary T
      ~= rebound_view := rebind(x); clone(rebound_view)  when x : T ref | T share
  move(tmp)

ref(x):
  b = make_ref_borrow(x)
  move(b)

share(x):
  b = make_share_borrow(x)
  move(b)
```

Here `copy`, `ref`, and `share` are not endpoints. Each constructs some object
that can then be `move`d. The only passing endpoint is `move`.

- copy(x) applies the terminal move to tmp, without a pre-move of x.
- ref(x)/share(x) apply the terminal move to the formed handle b.
- move(x) ends x's generation exactly when its predetermined effect is Kill.
- Every materialized pass handle ultimately reaches a single terminal `move`
  action.

Two additional invariants close the Policy-mode boundary:

```text
PlainMaterializationPrinciple:
  destination PolicyMode ∈ {const, plain, mut}
  copy-to-destination = CopyConstruct(x) + terminal Move

NoPreMoveBeforeCopy:
  copy(x) ≠ move(x); CopyConstruct(x)
  copy(x) = CopyConstruct(x); terminal Move(result)
```

The const, plain, and mut destination cases use this same ordinary realization. The
destination mode may affect candidate preference or capability realization,
but it does not introduce three different kinds of copy and never consumes
`x` before `CopyConstruct(x)` has completed. Nor does transfer relabel the
producer result: source and destination modes are independent slot facts.
`CopyConstruct` here is the ordinary selected copy-family candidate summarized
by `CopyConstructExpansion`, not an additional builtin whose behavior is left
uninterpreted.

## 8. Borrow movement preserves parent/origin

Moving a borrow handle does not create a deeper borrow chain.

```text
move(borrow_node(parent = p, kind = k))
  = borrow_node(parent = p, kind = k)
```

The equality here is a fixed point on type / rank / access shape. It does not
claim that the same runtime handle has no linear state change: when MoveEffect=Kill the old handle dies and a new handle inherits the same
parent/origin/kind. Preserve follows its independent proof.

If `b1` is a borrow produced from `x`, then `move(b1)` produces a *sibling*
borrow handle with the same origin as `b1`, not a *child* borrow of `b1`.

```text
The moved borrow handle keeps the same parent/origin. It does not make the
previous handle the parent of the new handle.
```

This is what keeps access-tree depth from growing without bound during argument
passing: a moved borrow does not increase access-tree depth, does not increase
rank, and does not change the type value.

## 9. Relation to overload and argument adaptation

Argument adaptation consumes ordinary source observations and selected
parameter requirements, not a blind pass-insertion stage.

```text
Gamma |- arg => RawArgShape
Gamma |- ParameterShape x RawArgShape => AdaptedArgShape
Gamma |- AdaptedArgShape => selected ordinary actions followed by Move
```

An explicit borrow is checked at its exact type. A copy-derived result can
satisfy a value parameter through terminal Move: its source operation does
not create an incompatible pass-kind flag.

```text
NoImplicitBorrowFormation:
  adaptation cannot rewrite T to T ref or T share
  structural repair cannot insert ref, share or @
  a clone-derived path requires ordinary selected realization evidence
```

Internal share/rebind inside that realization does not authorize adaptation
to invent a borrowed actual. Ordinary applicability, preference, unique
selection and no-reopen still apply. No second pass-kind ranking exists.
Callable-frame self remains separate from explicit argument adaptation.

## 10. Relation to type values and rank

Pass mode is not part of `TypeValueId`. Type matching and pass matching are
separate concerns:

```text
type/value/rank compatibility:
  arg_type == parameter_type
  arg_rank == parameter_rank

operation legality:
  selected producer path, exact result type, terminal Move and frontier Pre
```

`T move` is not a new type. `T move == T` is a core principle, and
`rank move == rank` is a core principle. Move preserves the transported type
and rank. Explicit ref/share forms its own borrow type before transport;
clone returns its declared type.

## 11. Relation to IR

The IR must not retain `in`, and it must not retain an undecided default pass.
The final IR / lower-action layer sees only fully decided actions, for example:

```text
CopyConstruct x -> tmp
Move tmp -> arg_slot

Move explicit_share_handle -> arg_slot

Move explicit_ref_handle -> arg_slot

Move x -> arg_slot
```

If a source/meta layer produces a nested move, it must be canonicalized:

```text
move(move(x)) => move(x)
```

This fixed-point equation is canonical target semantics, not a description of
current implemented behavior. An IR may retain `CopyConstruct` as a compact
instruction only after recording which ordinary share/clone or rebind/clone
realization was selected; the instruction name does not erase the semantic
equivalence in `CopyConstructExpansion`.

## 12. Relation to later call modes

This layer is also a prerequisite for the future `normal` / `tco` / `loop` call
modes, but this document does not define call modes.

When `tco` actively moves arguments, what it moves are argument objects that have
*already* completed pass normalization. `loop` requires stronger slot
compatibility and may depend on whether an argument object is already reusable in
place. These are cross-references only; this document does not expand
ABI or tail-call checking.

## 13. Non-goals

```text
No parser syntax change.
No current normalizer behavior change.
Implementation scope: this document defines lowering obligations and does not
change the Rust implementation.
No full trait solver.
No full type checker.
No borrow checker implementation.
No access-tree construction implementation.
No ABI design.
No LLVM lowering.
No runtime overload implementation.
No final IR instruction format.
```

## 14. Relationship to other documents

The documents below own the adjacent relations consumed by this model.

- `pattern-normalization-and-first-order-overload.md` — produces the
  `RawArgShape` / `ParameterShape` objects that argument adaptation consumes.
  Pass adaptation is the mechanical-argument-passing step within or after
  candidate adaptation.
- `type-values-places-and-borrow-views.md` — defines `TypeValueId`; pass mode
  is explicitly not part of it, and `T move == T`.
- `type-associated-function-objects-and-access-trees.md` — field-function and
  access-tree work; an explicit `ref` / `share` produces a borrow object whose
  handle is moved while preserving parent/origin.
- `overload-resolution-design.md` — candidate matching must separate type/rank
  compatibility from pass compatibility.
- `meta-object-invocation-and-policy-reduction.md` — the invocation engine that
  ultimately receives fully decided pass actions.
- [semantic evaluation](../meta-invocation/evaluation-residual-and-optimization.md)
  preserves selected ordinary realization and terminal Move. No get_default_pass
  lookup is needed to choose a second terminal pass.

## 15. Cleanup placement and with

Cleanup is placed before @ or other lifetime observation. Omitted with always
uses the ordinary NLL default; explicit empty with{} anchors cleanup at the
lexical boundary. Neither supplies missing access, borrow, capture, type or
construction authority.

```text
x with{a,b}  =>  x -> a and x -> b
Touch(x) = Use(x) ∪ Consume(x) ∪ Destroy(x)
UseForPlacement(a)
  = OrdinaryRequiredUses(a) ∪ ⋃{Touch(x) | x -> a}
```

Only actual continuation events belong to Touch. Each outgoing edge makes the
dependency a survive the dependent x's required touches. If both destruction
events exist, Destroy(x) < Destroy(a). Ordinary uses of a do not extend x.
Transitive y -> x -> a orders their existing destructors y before x before a.
Strict cycles have no legal cleanup placement and diagnose.

A killing move discharges the old generation's cleanup obligation at its
consumption cut. It adds no second Destroy/drop, even with lexical with{}.
Required uses of the transferred generation follow that generation's ordinary
relations. Observable destructors and effects cannot be removed to satisfy
the graph. NLL and lexical anchors are defaults/constraints within this same
placement relation, not an iterative lifetime repair algorithm.

After all NLL, last-use, with, move, explicit-drop, return/control-completion
and other established constraints, each cleanup point is fixed. Only remaining
unordered events at the **same** point use reverse declaration order:

```text
SameCleanupPoint(x,y) and DeclBefore(x,y) and otherwise unordered
  => DropBefore(y,x)

DeclOrder(a) < DeclOrder(b) < DeclOrder(c)
CleanupPoint(a) = CleanupPoint(b) = CleanupPoint(c) = p
no other ordering constraints => drop c; drop b; drop a at p
```

Respect established precedence first; reverse declaration order is the final
priority among still-unordered available events at that cut. It is not a
placement constraint and never advances or delays a cleanup point.

```text
ordinary control/liveness facts -> cleanup constraints -> points fixed
  -> same-point reverse-declaration linearization -> complete sequence fixed
  -> lifecycle observation / @
```

No alternative-cleanup observational-equivalence question remains. There is no
cleanup/lifetime solver loop.

The current Raw/Norm with carrier preserves items, explicit emptiness and
errors. It does not implement Touch, cleanup scheduling or lifetime checking.


## 16. Dependency-bearing closure results

A legal completed closure expression returns full tau_C through ordinary
struct formation. Its dependency requirements and semantic realizations
precede layout and ABI selection. Mechanical transport preserves these
realizations without recapture, and checks FormationLegal, LifetimeLegal,
Pre/Post, MoveEffect/Movable, EscapeLegal and owned transfer/promotion where
applicable. Neither complete-type status nor ZST layout grants global survival.
The [lifetime refinement handoff](../lifetime/lifetime-policy-and-overload-boundary.md#8-closure-dependency-lifetime-refinement-handoff)
retains the unresolved persistence/escape scope; this pass does not infer
universal escape permission or a universal ban on local dependencies.
