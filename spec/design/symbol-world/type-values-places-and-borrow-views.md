# Complete Pattern Values, Places, and Borrow Views

Complete type values are called complete pattern values (tau). Core observation,
whole-snapshot observation, and ordinary Pattern algebra retain the distinctions
specified below. [Names and OverloadGroups](names-and-overload-groups.md) owns
name occupancy and group algebra; they do not add an Object axis.

**Status: canonical semantic authority for Object identity, complete
type-closure identity, Place identity, and borrow views.**

This document specifies the semantic boundary between *object values*, *binding
identity*, *places*, *borrow views*, and *namespace extension targets*. It
defines what an object is, what a place is, which borrow operators exist, which
overloads they have, and when each overload is callable. It is a semantic
authority, not a parser or normalizer rule.

The document is self-contained. It does not require the reader to assemble its
meaning from `type-associated-function-objects-and-access-trees.md` or
`early-meta-functions-and-namespace-graph.md`. Those documents are background
or adjacent design only; the model here stands on its own and is the canonical
authority for the value / place / binding / borrow-view distinction.

There is no ordinary symbol-alias or place-forwarding declaration form in this
language. `let a = b;` copies a value into a fresh binding with a fresh place.
Sharing an observation of another object is expressed by the borrow operators
defined in §5, never by a declaration that makes two bindings name one place.

The broader symbol-first facet, `PatternValue`, `compile` / `meta`, pattern
scope, `struct`, pure `extend`, and place-level `inject` model is canonicalized in
`spec/design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md`.
That document composes with this identity/place model rather than replacing it.

## 1. Purpose

The language must distinguish three things that look similar in source text but
are semantically different: the identity of a name, the identity of a writable
location, and the identity of a type value. Conflating them produces subtle
errors — for example, injecting a declaration into a built-in type because its
type value happens to equal the type value stored in a fresh binding.

The invariants this document protects:

```text
value equality must not collapse binding/place identity
a borrow view must not manufacture a place that its source does not own
member creation must target a place or stable prospective ProjectionSlot
structural extension must transform an Open value, not borrow authority
writability and Open must be checked independently
```

This document does **not** define:

- a full type checker,
- access-tree construction,
- the lifetime checking algorithm (see `../lifetime/lifetime-policy-and-overload-boundary.md`),
- package import / export,
- runtime lookup.

Three phrasings are explicitly rejected throughout. `let T: type = uint8` is
**not** fresh nominal type generation. A borrow view is **not** textual
substitution and **not** a second name for a name binding. And value equality is
**not** place equality.

## 2. Semantic identities

Three distinct binding/type identities participate in this model, alongside
canonical pattern-value identity:

```text
NameBindingId
NameCoord(root, selector) -- independent of realization
PlaceId
TypeValueId
PatternValue identity
```

- NameCoord is a structural coordinate, not an Object, Place or resident. A
  coordinate does not establish Retained or BindingPlace. See the name owner
  for realization and Fresh = not Retained.
- `NameBindingId` identifies an ordinary realized binding and its resident
  Place relation. It is not a language value and cannot be borrowed or carry a
  `.type` field. A borrow addresses its typed Place, including before the first
  resident exists; its type comes from PlaceType, not an implicit value read.
- `PlaceId` is the identity of a location that can be bound, updated, injected
  into, or opened for a namespace delta.
- `TypeValueId` is the stable first-order root of `Core(tau)` — a registry
  projection used by the current substrate (cache, lookup key, or first-order
  root), not the full type-closure identity and not the default semantic
  equality of types. The complete identity of a type value is:

  ```text
  TypeObservation(tau) = Addr(Norm_type(tau))
  ```

  Three identity layers apply to a type value:

  ```text
  TypeValueId = stable first-order root projection of Core(tau)
                -- implementation/index key, not semantic equality

  Core(tau) = Q -- default observation for ordinary type-rank equality,
                    keying, and type-argument identity

  Addr(Norm_type(tau)) = bind alpha.<Norm(Q), Norm_V^alpha(V_τ)>
                -- whole-snapshot identity; used to tell shared-root snapshots
                   apart in transport and in positions the language has
                   independently frozen to whole-snapshot semantics
  ```

  Two closures may have the same `TypeValueId`/core root while carrying
  different immutable `V_τ` snapshots. Bare `TypeValueId` comparison is therefore
  not upgraded into the default observation of ordinary type equality: the
  ordinary default remains `Core(tau)=Q` (the canonical Object/Pattern equality
  on the core), while whole-snapshot positions use `Addr(Norm_type(tau))`.
- `PatternValue identity` is ordinary Object identity. A type value participates
  in Pattern/value/namespace observation through `Core(tau) = Q`. Per the
  minimal-change rule, ordinary type-rank equality, keying, and type-argument
  identity keep observing that core by default; `CallSpace(tau)=V_τ` supplies
  type-as-callee candidates; and copying, `extend`, and `inject` transport or
  transform the whole snapshot including `V_τ`.

These identities are independent. None implies another:

```text
NameBindingId does not determine an immutable TypeValueId across resident replacement.
TypeValueId equality does not imply PlaceId equality.
PatternValue equality does not imply NameBindingId or PlaceId equality.
A borrow view names one place from one origin; it relates values and places without erasing the distinction.
```

Meta invocation normalization retains the identities of semantically observed
name/construction-subject dependencies in addition to its ordinary value
observations. This does not insert NameBindingId, PlaceId or Anchor into
Norm(Object). Equal Core values may have different subjects; copies preserve
an existing subject. A is a derived instance: a saved result reference retains
its captured invocation Place and subject across input-carrier replacement.
See [invocation](../meta-invocation/meta-object-invocation-and-policy-reduction.md)
and [associated state](associated-compile-state.md).

A type expression cares about the *value*. A namespace extension target or a
declaration-extension site cares about the *place*. A borrow view is itself a
value that carries a place coordinate. The three concerns must not be folded
into one another.

Read_name supplies Path structure before Read_resident; # / path_pattern
projection can retain it without resident lookup.
At external Read a textual root resolves in the use environment, while an explicit
value/ref root preserves its identity. A resolved named type uses V_tau for calls. Explicit
OverloadGroups aggregate type candidates through eta(T), without adding a
second name container. Member projection follows the ordinary consumer rules. A selected tau
is already self-contained; its Core, CallSpace and whole observations do not
come from a privileged group slot or the other entries.

### 2.1 Object identity is the recursive three-component normal form

Every object in the language has the same shape:

```text
Object x  = ⟨ Val1?(x), P(x), Val2(x) ⟩
Val1?(x) ∈ 1 + Object
```

`Val2` is a finite map from semantic selectors to ordinary Objects. Named
entries can hold arbitrary ordinary values of any type. A complete named type
is one possible resident, not the universal type of a Val2 entry. The built-in
bare Product Pattern additionally supplies intrinsic ordinal selectors `pos_i`.

The structural lookup and the value snapshot are distinct:

```text
named selector n -> NameCoord(parent,n), independent of realization
Retained(parent,n) -> realized structural NameBinding / typed Place, when established
initialized q   -> ResidentState(q) = Initialized(v)
Val2(parent)[n]  = v only in that initialized case
```

| HasName(parent,n) | ResidentState(BindingPlace(n)) | Val2 entry |
| --- | --- | --- |
| false | no binding/Place | absent |
| true | Uninitialized | absent |
| true | Initialized(v) | ordinary resident v |

An internal Contents(q) = Some(v) encoding describes only the last row.
Structural occupancy is not resident presence, and Uninitialized is not an
Object or a None value. Close checks the initialized-name condition of §7.1.

HasName in this table means Retained, never the existence of NameCoord. The
first row still has a coordinate for a legal root/selector, but no borrowable
Place follows from it. Prospective ProjectionSlot lookup is structural addressing
material; realization, resident generation and write authority remain separate.

Ordinary name initialization/replacement may update Core's ordinary Val2:

    Q = <Val1?, P, V>
    OrdinaryNameWrite(q_s,v): V' = V[s -> v]
    Q' = <Val1?, P, V'>

Thus Q' may differ from Q even though P and its registrations are unchanged.
The write uses the ordinary Place first-write/replacement rules and their Pre
checks; it implies no DirectPatternChild, ConstructEdge, ExtractEdge, FieldView
or V_tau registration. Pattern-registered structural extension is owned by
extend/inject; TypeAdd separately updates V_tau registration. These are distinct
relations, not a restriction that every Core change must use extend/inject.

NameBinding and ProjectionSlot are not Val2 value entries. Normalization consumes
the resident v with its existing complete-type/Object observation; it never
normalizes a binding wrapper or a cluster carrier.
Those ordinal entries are not a compiler aggregate outside `Object`.

The least Object domain is closed by the following constructors:

```text
a_i ∈ Object
-----------------------------------------------------------------
BareProduct(a_0, ..., a_{n-1})
  = ⟨ null,
      P_bare_product(P(a_0), ..., P(a_{n-1})),
      { pos_i ↦ a_i | 0 <= i < n } ⟩
  ∈ Object

forall i < N: v_i ∈ Object and v_i : T
-----------------------------------------------------------------
Seq_N(T; v_0, ..., v_{N-1})
  = ⟨ BareProduct(v_0, ..., v_{N-1}), P_(T*N), GeneratedVal2(T*N) ⟩
  ∈ Object

forall i < n: v_i ∈ Object and v_i : T
-----------------------------------------------------------------
Seq_omega(T; v_0, ..., v_{n-1})
  = ⟨ BareProduct(v_0, ..., v_{n-1}), P_(T*omega),
      GeneratedVal2(T*omega) ⟩
  ∈ Object
```

Thus a bare Product's fixed heterogeneous shape is carried by its Pattern and
its owned elements are ordinary ordinal `Val2` children. A homogeneous Sequence
is an ordinary Object whose `Val1` is that bare Product Object and whose `Val2`
contains the mechanically generated associated operations. The erased
classifier case is likewise an ordinary wrapper:

`Val1(BareProduct) = absent` does not erase its elements: the bare Product is
the concrete structural carrier, and each `pos_i` child remains a complete
Object with its own Policy projections. An outer `product` or Sequence value has
an independent runtime value projection precisely because its `Val1` contains
that bare Product Object.

```text
ProductValue(a_0, ..., a_{n-1})
  = ⟨ BareProduct(a_0, ..., a_{n-1}), P_product, GeneratedVal2(product) ⟩
  ∈ Object
```

This is the precise meaning of `Val1(p) = (a_0, ..., a_{n-1})` for
`p : product`: the right-hand side denotes the `BareProduct(...)` Object above,
not a compiler-private tuple carrier. No constructor adds a fourth Object
component or a semantic collection outside the recursive Object domain.
For the empty case, `BareProduct() = ()` and
`P_bare_product() = P_FunctionItem`, agreeing with the standard leaf below.
Mentions of the host classifier in a mechanically generated accessor signature
are Pattern/type references, not owned vertical Object edges back to the host;
the generated member object otherwise follows the same ordinary recursion rule.

`Val1?(x) = null` states exactly one fact: this object carries no internal
`Val1` payload. It does not mean the object is untyped, unobservable,
value-less at the observation edge, or a different kind of entity. Type and
namespace are judgments over this one Object domain, not facets or nominal
Object subclasses:

```text
Pure(x)          <=> Val1?(x) = absent   -- null denotes this absence
OrdinaryValue(x) <=> Val1?(x) != absent
Navigable(V)     <=> V is a well-formed finite semantic-selector map
                     on which ProjectionSlot lookup is defined

WellFormedObject(x) => Navigable(Val2(x))
NamespaceRole(x)   <=> Pure(x)
TypeRole(x)        <=> Pure(x)

TypeRole = Pure = NamespaceRole
```

Type identity is determined by absent Val1, not by an available constructor.
The separate `SelfConstructible(Q)` judgment consumes the registered joint
Val2/ConstructEdge witness defined in the
[Pattern owner](../patterns-overload/pattern-values-relational-semantics-and-extraction.md#13-structural-role-registration-and-ordinary-callables).
It establishes construction capability, never membership in the type domain.

`Navigable(V)` means selector lookup yields the resident-specific
`ProjectionSlot` defined in §7; a missing final selector still yields a slot
whose contents are `None`, while continuation from `None` is invalid. Being a
well-formed Object already supplies such a `Val2`, including the empty map.
Consequently every pure Object has `NamespaceRole`; namespace capability is not
an extra entity or witness. Every pure Object has the type role, including one
with no self-construction witness. These are
predicates; no `NamespaceFacet`, hidden Q/type-role member, or parallel
`NamespaceObject` ontology is introduced. Bare Product and the empty pure
Pattern therefore have intrinsic structural namespace capability through their
ordinary selectors; whether a tuple-like user-facing namespace API exposes
that capability remains a narrow surface question.

An Object carrying `Val1` is an ordinary value. An independently authorized
ordinary type projection (§5.6) may produce a type from it; that operation does
not make the original payload-bearing value a type. Core type role and complete
type well-formedness remain distinct: a complete tau must satisfy §2.2.

The canonical identity of an object is the recursive normal form over **all
three** components. `Val1` normalization is indexed by the host Pattern because
some built-in carriers have a representation order that their Pattern does not
admit as semantic identity:

```text
Norm(x)                    = ⟨ Norm_Val1?^(P(x))(Val1?(x)),
                                Norm_P(P(x)),
                                Norm_Val2(Val2(x)) ⟩
Norm_Val1?^P(null)         = null
Norm_Val1?^P(v)            = Norm(v)       -- default owned-object case
Norm_Val2(V)               = Map_selector( Norm(V[selector]) )
                               -- named entries may have any ordinary resident type;
                               -- bare-Product pos_i entries are ordinary Objects
```

The Product and Sequence equations require no parallel normalizer:

```text
Norm(BareProduct(a_0, ..., a_{n-1}))
  = ⟨ null,
      Norm_P(P_bare_product(P(a_0), ..., P(a_{n-1}))),
      { pos_i ↦ Norm(a_i) | 0 <= i < n } ⟩

Norm(Seq_N(T; values))
  = ⟨ Norm(BareProduct(values)), Norm_P(P_(T*N)),
      Norm_Val2(GeneratedVal2(T*N)) ⟩

Norm(Seq_omega(T; values))
  = ⟨ Norm(BareProduct(values)), Norm_P(P_(T*omega)),
      Norm_Val2(GeneratedVal2(T*omega)) ⟩
```

Positions remain ordered because `pos_i` is part of the selector identity. `N`
remains in `P_(T*N)` identity; the current finite length of `T*omega` is visible
only through its `BareProduct` Val1. These are instances of `Norm(Object)`, not
compiler-container identities beside it.

OverloadGroup normalization uses this same Object universe. Carrier/entry
encoding remains open; the semantic candidate domain (complete type candidates,
with eta(type) as singleton embedding) and current aggregation laws are closed
by the [name/group owner](names-and-overload-groups.md). Encoding must preserve
associative/commutative contribution and distinct entries even when their values
are equal. It has no distinguished type coordinate. Pattern child normalization
and replay use their own existing laws; group aggregation does not replace them.

There is no case split in which one Object component is ignored. A value-bearing
Object whose `Val2` differs is a different Object, and a pure Core `Q` whose
`Val1?` is `null` still normalizes its `P` and `Val2` fully. `Norm(Q)` is the
ordinary three-component Object normal form. `Norm_type(tau)`, defined in §2.2,
is instead the normal form of the complete `<Q,V_τ>` closure and must not be
collapsed to `Norm(Q)`.

An ordinary object's incidental carrier/residency coordinates never enter its
normal form. A borrow view is the deliberate exception in kind, not in
principle: its target coordinate is content of the borrow-view value itself and
is covered below.

```text
CarrierPlace(ordinary object)         ∉ Norm(ordinary object)
ResidencyObjectPlaceId(ordinary object) ∉ Norm(ordinary object)
NameBindingId                              ∉ Norm(ordinary object)
allocation order                     ∉ Norm(ordinary object)
provenance                           ∉ Norm(ordinary object)
```

`ResidencyObjectPlaceId` names the ordinary object's `ObjectPlaceId` only in
its role as the coordinate from which content was observed.

The recursion is **well-founded finite recursion** over every owned vertical
edge that a component normalizer traverses, not only over `Val2`:

```text
Children_owned(x)
  = Children_Val1(x)
  ∪ Children_Val2(x)
```

For a bare Product, `Children_Val2` includes every ordinal element selected by
`pos_i`. For a Sequence or a value classified by `product`,
`Children_Val1` contains its `BareProduct` Object, whose ordinal children are
then reached by the same `Val2` rule. These two sets exhaust owned-object descent
in the current normal form; any future component rule that introduces another
owned object edge must express that edge through `Val1` or `Val2` before the rule
is canonicalizable.
The leaf boundary is the general condition

```text
L = { x | Children_owned(x) = ∅ }
```

`()` is the standard leaf:

```text
Val2(()) = ∅
Norm(()) = ⟨ null, Norm_P(P_FunctionItem), ∅ ⟩
```

Other typical leaves are terminal built-in type objects and associated pure-P
objects whose concrete object carries no further owned expansion — an
associated type is not a special recursion rule, it is an ordinary pure P that
happens to have run out of children. Borrow views (`ref` / `share`) are also
owned-recursion leaves, not back references, but the target identity they carry
is part of their leaf value:

```text
Norm( Borrow_k(q) )
  = ⟨ ⟨ BorrowKind_k, StableTargetIdentity(q) ⟩,
      Norm_P(P(Borrow_k(q))),
      ∅ ⟩

Children_owned(t ref)   = ∅
Children_owned(t share) = ∅
PatternOf(t ref)        = t ref        (extraction still matches the form)
```

`StableTargetIdentity(q)` is the stable semantic identity of the target selected
when the borrow forms. For direct places it identifies the resident place; for
a projected target it is the `ProjectionSlotIdentity` defined in §7. Its final
representation remains an implementation choice, but it must distinguish
`q1 != q2` even when the two targets currently contain equal values. It is not
merely a reusable logical navigation coordinate, is not the borrow view's
incidental holder/carrier place, and normalization does not recurse into the
current contents of `q`.

In particular:

```text
ProjectionCoordinate(parent_place, selector)
  != ProjectionSlotIdentity(parent_resident, selector)

Target(Borrow(Nav(parent_borrow, selector)))
  = ProjectionSlotIdentity(parent_resident_at_formation, selector)
```

The prospective coordinate remains reusable for later navigation and creation.
An already formed borrow remains bound to the parent-resident slot generation
that existed at formation time, whether its contents were `None` or `Some`.
Wholesale parent replacement makes the old slot invalid under ordinary
lifetime/validity rules; it never causes the borrow to observe a new slot at the
same logical coordinate. Only an explicit `rebind` can acquire that target. A
generation, resident id, or versioned encoding remains an implementation choice.

The `t` in `(t ref)` is pattern material of the built-in operation that
produced the value, not a vertical object edge inside the produced value.
Borrow-view extraction is **horizontal, not vertical**: pattern decomposition
never creates an owned child edge, so `extractable` does not imply
`recursively traversable`, and `t → (t ref) → t` never exists as an object
cycle.

This separates two rules that must not be merged:

```text
CarrierPlace(ordinary object) ∉ Norm(ordinary object)
Target(Borrow_k(q))           ∈ Norm(Borrow_k(q))
```

The second coordinate is not provenance about where an ordinary value happened
to be stored; it is the observable referent identity of the borrow-view value.
Assignment, `rebind`, ordinary borrow escape checking, and compile-reference
cache identity all depend on that distinction. Construction-authority Open
(`OpenHere_Σ`) does not depend on the target coordinate.

For an ordinary by-value Object, its `PlaceId` is **not** identity material. A
place is only the coordinate from which that Object's current value is read:

```text
place(x) -> Read(place(x))
```

For a type-valued binding that read is the complete immutable `tau` snapshot.
Snapshot identity follows `Norm_type(tau)`, including both its core and
callspace; ordinary type equality/keying defaults to the core observation
`Core(tau)=Q` (minimal-change rule, §2.2):

```text
Norm(Q_x) = Norm(Q_y) and Norm_V(V_x) = Norm_V(V_y)
  => Norm_type(tau_x) = Norm_type(tau_y)

Norm(Q_x) != Norm(Q_y) or Norm_V(V_x) != Norm_V(V_y)
  => Norm_type(tau_x) != Norm_type(tau_y)
```

The first line holds even when the closures are stored in different places; the
second holds even when two snapshots share a first-order root. These equations
state when two snapshots are the same snapshot; they do not by themselves decide
which observation a particular old rule must use (see the classification in
§2.2). A list of
allocated value ids under each name is not a normal form:
allocation order is not semantic content. For a live place observation, first
read each occupied named slot's complete resident T. For a captured Val2
snapshot, those resident values are already present; recursively normalize
them without consulting the live name graph. Neither path includes NameBindingId
or PlaceId in an ordinary by-value normal form.

This exclusion does not erase `StableTargetIdentity(place)` from a borrow-view
normal form. In the ordinary object case a place is an observation source; in
the borrow-view case the target is what the value denotes.

This makes an open construction observable. Within one meta body, with ordinary
Writable/OpenHere premises and type-valued X/Y:

```lang
let t = (() t) |> struct;
mut let f_ref = (let f::t:type) ref;
f_ref = X;
let A = t |> compile_fn;
mut let g_ref = (let g::t:type) ref;
g_ref = Y;
let B = t |> compile_fn;
```

Each sequence creates a typed Place, explicitly borrows it, and initializes
it with X or Y. Only successful writes produce these two complete snapshots:

```text
tau_1 = <Q_1, V_τ>
tau_2 = <Q_2, V_τ>

Val2(Q_1) contains f
Val2(Q_2) contains f and g
```

so `Norm_type(tau_1) ≠ Norm_type(tau_2)`. The `compile_fn` calls may consume both
meta-local observations through ordinary value transport. A nested meta call
may also consume valid open dependencies. Its value-snapshot observation must
still distinguish these values; a name-dependent input additionally preserves
its stable subject under the invocation identity rules. Reading only a shared
first-order root would incorrectly merge ordinary snapshot observations.

Memoizing FINISHED cycle-free subtrees is permitted (a shared acyclic diamond
is DAG reuse, not a cycle), but no `PlaceId` or memo node number may appear in
the resulting normal form, and no `SemanticValueId` may enter the
recursively-normalizable object structure.
A `Val1` payload that has no content normal form yet is the one permitted
exception: it keeps an identity-stable opaque leaf (`OpaqueValue`), so two
references to one value share an address while two content-equal but distinct
values stay distinct. This is a safe under-merge, never a claim of a stronger
equivalence than the implementation actually decides, and never a licence to
treat `Val1` as excluded from the normal form: the target rule is that `Val1?`
normalizes recursively like every other component. The opaque leaf preserves
safe under-merge until a content normalizer is available. It does not
override a defined Pattern-specific quotient of an ordinary Pattern family.

Complete type values contain one tightly scoped normal-form binder
back-reference, defined below. Such a `BoundRef(alpha)` is not an owned child
edge. Well-foundedness is stage/policy-sensitive:

```text
WellFounded_kappa(x)

static-eval (kappa = static-eval):
  terminating finite generation; restricted P*Val2 back-references admitted
  as finite-graph compression (BoundRef(alpha) is the canonical instance)
  -- compile and meta both instantiate this regime; the label does not
     identify compile policy with meta policy semantics

runtime (kappa = runtime):
  the materialized owned graph must remain acyclic; a back-reference cannot
  be reified into a real ownership cycle
```

Re-entering any object still on the normalization/owned-recursion stack through
any positive owned path proves a violation at the stage where the cycle is
materialized:

```text
x ∈ OwnedRecursionStack
∧ x ∈ Children_owned+(x)
--------------------------------
NoNormalForm_kappa(x)
```

Thus `Val1(x) = x`, `Val1(x) = y ∧ Val1(y) = x`, a cyclic product, and a cyclic
owned Val2 graph with an actual back-edge to its own ancestor all have **no
normal form** at the stage where they are materialized. A source spelling such
as creating a typed loop name, explicitly borrowing it and initializing with t
alone does not prove such a cycle: it performs ordinary snapshot initialization, whose actual
owned edges must be checked. A finished shared acyclic subtree remains valid DAG
reuse. `Self_τ` is one restricted static back-reference instance, not the one
exceptional cycle, and not a general recursive-data constructor.

#### 2.1.1 Symbolic reference edges and evaluation edges

The model separates two edge kinds connecting objects, because legality of a
stored reference and legality of a live reentry are different questions:

```text
SymbolicReferenceEdge(x, y)
  -- x refers to y by binder/symbolic means, with no ownership edge.
  -- Self_τ inside V_τ establishes
       SymbolicReferenceEdge(member, tau)
     for a member referring to the enclosing closure.
  -- Norm_type^alpha(Self_τ) = BoundRef(alpha)
  -- BoundRef(alpha) notin Children_owned

EvaluationEdge_kappa(x, y)
  -- the stage-kappa evaluation flow proceeds from x into y.

ActiveEvaluation_kappa(x)
  := x lies on the currently running stage-kappa evaluation flow.

OpenEvalReentry_kappa(x)
  iff OpenHere_Σ(x)
  and ActiveEvaluation_kappa(x)
  and NextEvaluationStep_kappa enters x
```

A symbolic reference never establishes an `ActiveEvaluation`: the existence of
a `SymbolicReferenceEdge` to a value at the same stage does not mean the
current computation flow re-traverses that value. Openness plus a stored
reference is not reentry; only an `EvaluationEdge_κ` on the live flow can
reenter. Therefore `Self_τ` inside a stored `V_τ` is legal under static-eval:
it is a binder back-reference (symbolic anchoring), not an evaluation cycle,
and it does not trigger `OpenEvalReentry_κ`.

The normalizer's active recursion stack is the **normalization/owned-recursion
stack**: it records owned-child traversal during `Norm` / `WellFounded_kappa`
checking and is a distinct object from the evaluation-active flow above.
Re-entering an object on the normalization/owned-recursion stack through a
positive owned path proves `NoNormalForm_kappa`; following a `BoundRef` is a
bounded binder jump, not a stack push and not an evaluation reentry.

Meta and nonmeta type closures share one `bind alpha` / `Self_τ`
representation. Their difference belongs to the symbolic anchoring relation,
not to the graph-shape rule:

```text
SelfResolve(meta)    = root-relative/deferred symbolic resolution
SelfResolve(nonmeta) = finite same-stratum static backreference
```

Both resolve through the same binder; `SelfResolve` records which regime
applies.

### 2.2 Complete type values are closed snapshots over Object cores

The ordinary pure Object `Q` supplies the Core of a complete type value:

```text
Q in Object
Pure(Q)
TypeRole(Q)
```

A complete language-level type value is the closure:

```text
tau = <Q, V_τ>

Core(tau)      = Q
CallSpace(tau) = V_τ

WellFormedTau(tau)
  iff tau = <Q, V_τ>
  and Q is a well-formed pure Object
  and PatternClosureConsistent(tau)

PatternClosureConsistent(tau) iff
  Q = Core(tau) and V_τ = CallSpace(tau)
  and WellFormedCore(Q)
  and ∀F ∈ ClassifierDomain(V_τ):
      F is registered for this type's own callability
      and OrdinaryCallableValue(F) and Val1?(F) != absent
      and F is a complete internally well-formed callable and Home(TypeOf(F)) = TypeMemberScope(tau)
      and its () entry obeys Type(callee) = Type(first self)
  and ∀K registered as a Pattern construction/extraction closure in Q:
      K is the actual ordinary Val2 resident referenced by that registration
      and K is a complete internally well-formed callable
      and Home(TypeOf(K)) = TypeMemberScope(tau)
      -- /tau(tau); checked whether or not K is also registered in V_τ
  and AllBoundRefsBoundAndRestricted(bind α.⟨Norm(Q), Norm_V^α(V_τ)⟩)
      (every BoundRef reachable during Norm_type^α(Q, V_τ) is bound by α
       and belongs to an authorized static edge kind;
       BoundRef(alpha) notin Children_owned)
  and all structural/interface registrations referenced by Q and V_τ
      are internally well-formed
  and no CurrentAuthority / OpenHere_Σ / GenerationRegime /
      WindowLive_Σ / stack / provenance premise is used
  -- a structural judgment over the current closure value, with no
     dependence on how tau was produced; member identity and captured
     callspaces remain intrinsic to the contributed complete values

WellFormedTau is history-free:
  Norm_type(τ₁) = Norm_type(τ₂)
  ⇒ WellFormedTau(τ₁) ↔ WellFormedTau(τ₂)
      -- structurally identical closures never differ in well-formedness
         because of construction history
         (τ is not an Object; complete type normalization is
          Norm_type(tau) = bind alpha.⟨Norm(Q), Norm_V^alpha(V_τ)⟩)
```

**Same-entity boundary (normative).** A complete type value and its
description material are one semantic entity observed through two equivalent
views (canonical owner:
`spec/design/patterns-overload/pattern-values-relational-semantics-and-extraction.md`
§15):

```text
SameEntityTypeInvariant:
  DescriptionView(X) = ⟨P, Val2⟩
  TypeClosureView(X) = τ = ⟨Q, V_τ⟩
  τ ≡ DescriptionClosure(P, Val2)
```

`DescriptionView` and `TypeClosureView` are projections of one semantic
entity; they are not two objects later mapped to each other, and neither
`⟨P,Val2⟩` nor `⟨Q,Vτ⟩` is an entity prior to or beside the other. The
closed/well-formedness constraints on the two views apply jointly to that
one entity; an update observed in one view must satisfy the resulting complete
closure's constraints. This does not identify ordinary Val2 residency with
callspace registration or freeze all generated Val2 realization at Close. This is
not the identity `V_τ = Val2`: it is the statement that both sides constrain
the same semantic entity.

```text
TypeValueRole(tau)
  iff WellFormedTau(tau)
      -- the type-value role; equivalently CompleteType(tau)

ConstructibleType(tau)
  iff TypeValueRole(tau)
  and HasRegisteredSelfConstruction(Core(tau))

NamespaceWithoutSelfConstruction(tau)
  iff TypeValueRole(tau)
  and not HasRegisteredSelfConstruction(Core(tau))
      -- still a complete type, with ordinary namespace observation

CallSpace(tau) = V_τ
  // Intrinsic property of the closure: the TypeMember set captured in this
  // closure value. It does not depend on the current host name binding, source
  // binding or any other provenance recovery.
  // Members created later under the same Q never retroactively enter an
  // existing snapshot; a copied or extracted tau keeps the same V_τ.
```

The closure value is first-class: copying, extraction and argument transport
retain its captured callspace. Adding or removing an entry from an ordinary
OverloadGroup changes that group, not the tau encapsulated in any entry.

Changing V_tau produces a new complete pattern value, under the existing
extend, OpenHere and independent well-formedness rules. Each group's call
consumer projects its entries; any selected pattern member supplies its own
complete immutable callspace. Other entries do not supplement that snapshot.

V_tau registration, ordinary Val2 residency, Pattern registration and
ConstructEdge are independent judgments. In particular:

```text
CallOrdinary(x) -> Type(x) -> AssociatedNamespace(Type(x)).Val2[()] -> Impl
    self = x
CallCandidates_type(tau)
    = disjoint_union over c in V_tau of CallCandidates_ordinary(c)
CallCandidates_ordinary(c)
    = Entries(AssociatedNamespace(Type(c)).Val2[()], actual_self=c)
    -- one ordinary selection seals (c*, Impl*, projection*, frame)
V_tau != Val2
```

These entrances use the same ordinary selection pipeline. Type-callee
projection does not require a ConstructEdge self-construction witness, while
TypeRole follows purity and SelfConstructible records construction capability.
An ordinary callable x does not require membership in V_(Type(x)).

Ordinary Val2, V_tau membership, and Pattern-role registration remain separate
facts. V_tau is the immutable snapshot of ordinary callable values
registered for the type's own callability, with classifiers in /tau(tau) (the canonical
`Home(TypeOf(F)) = TypeMemberScope(tau)` notation). Ordinary Val2 can have any type
and need not satisfy that anchoring. Classifier eligibility alone does not
register a member for callability. V_tau registration gives the callable value
no val::path selector and requires no named Val2 resident. The anonymous
classifier's navigable home does not establish navigation to that value.

Pattern-registered extraction/construction closures likewise have their own
classifier under tau, but Pattern registration does not imply V_tau membership,
and V_tau membership does not imply Pattern registration. These are role
registrations over ordinary values; the same value may have both roles without
being copied for that reason. A type
may therefore hold an unregistered OverloadGroup payload without making that
group part of its own callspace or its Pattern identity. Both registrations are
non-generative. Pattern registration fixes the structured construction/extraction
form, while ordinary generated Val2 results carry neither registration.
Close freezes the registered structure and ends construction authority; it does
not freeze the set of all future ordinary generated Val2 realizations. Those
effects remain ordinary observable Core changes, never hidden cache facts.

Every well-formed tau has the type-value role. Self-construction is a separate
Q-local structural capability, never a condition on type identity or the
sibling count of a name binding space:

```text
TypeRole(Q) iff Pure(Q) iff Val1?(Q) = absent
SelfConstructible(Q) iff HasRegisteredSelfConstruction(Q)
      -- iff exists Pattern P of Q, exists s, exists C, exists K:
            Val2(Q)[s] = K and ConstructEdge_P_Q(C, Q, K)

NamespaceWithoutSelfConstruction(Q)
  iff NamespaceRole(Q)
  and not HasRegisteredSelfConstruction(Q)
```

The construction witness has no implicit tau argument or classifier-home test.
CompleteType(tau) requires
PatternClosureConsistent(tau), which checks /tau(tau) homes for both registered
closure roles independently. Equal Core can establish the same TypeRole answer
without establishing compatibility with two distinct complete-type homes.

For example, if Q has the joint Val2/ConstructEdge witness K, SelfConstructible(Q) can
hold even when K is not in V_tau. If Home(TypeOf(K)) differs from /tau(tau),
PatternClosureConsistent(tau) fails and tau is not a CompleteType. Removing K
from the callability projection cannot conceal the failed Pattern-role home
check. A matching home satisfies that premise but registers no new role.

Conversely, a well-formed tau with nonempty V_tau and no self-construction
witness is a complete type. It can offer applicable call candidates without
supporting construction of ordinary instances of itself. Closure-generated
tau_C requires no self-construction witness to be a type or to supply calls.

A snapshot `tau' = <Q', V_τ>` written by an ordinary slot update is checked
the same way as any closure: `WellFormedTau(tau')` is an independent structural
judgment over `tau' = <Q', V_τ>` — `Q'` is a pure Object, and `V_τ`
is unchanged. Changing only Val2 preserves Pure(Q') and hence TypeRole(Q').
Complete well-formedness and SelfConstructible(Q') are independently checked;
ordinary write registers no ConstructEdge and guarantees neither of them.

**Construction-capability counter-example.** Suppose the sole self-construction witness is
`Val2(Q)[s] = K ∧ ConstructEdge_P(C, Q, K)`. An ordinary write
`Write(ProjectionSlot(Q, s), K')` updates `Val2(Q')[s] = K'` but does not
register `ConstructEdge_P(C, Q, K')`. The original joint witness disappears,
so SelfConstructible(Q') may be false. TypeRole(Q') still holds because its
Val1 remains absent. A stale Pattern registration may separately make the
complete tau' ill-formed; type-role preservation is not a theorem that arbitrary
registered-member replacement preserves WellFormedTau.

Type +=/-= can produce a new V_tau with Core fixed under OpenHere and
anchored closure membership. Structural extend/inject can change the Core.
The resulting
`τ'` satisfies `WellFormedTau(τ')` by its own structure, never by inheriting
any formation history.

`V_τ = CallSpace(tau)` is the callspace captured into the closure value: the
direct TypeMember members placed into `tau` when it was produced
(`TypeMember_tau`, symbol-first §2.1), not a later partition of a shared name binding
space and not a global function of the bare core `Q`. `V_τ` is part of the
closure value itself — snapshot capture is intrinsic to `τ`, not a history
judgment — so `WellFormedTau` / `TypeValueRole` are not global functions of
the bare core `Q`. Members created under the same `Q` later never
retroactively enter an existing snapshot, and a copied or extracted `tau`
keeps its captured `V_τ`.

`tau` is not another Object and does not add a fourth Object coordinate. `Q`
and every ordinary member in `V_τ` remain Objects governed by the existing
`<Val1?,P,Val2>` ontology. The closure only preserves their type-specific
pairing so a copied or extracted type carries its own callspace.

Object-membership is **not** a semantic dispatch axis: whether `tau` is or is
not an Object does not by itself decide whether any operation may consume it.
Only the judgments an operation actually requires participate in admissibility:

```text
NoSemanticDispatchByCarrierMembership
  x ∈ Object / x ∉ Object
    itself implies no operation role
```

Consumers use the projections they need (`Core`, `CallSpace`, `CarrierPlace`,
`ProjectionSlot`, `OpenHere`, `GlobalSurvivable`, `TypeRole`, `WellFormedTau`)
rather than first classifying `tau` as Object / non-Object / PatternValue /
CompleteType and then bridging values that "do not belong" to a class.

If an implementation needs to store `tau` in an Object-position carrier (for
example the `BareProduct` element inside `Σ_Object`), it uses the lowering
mechanism `LowerTypeClosure(tau) ∈ Object` (symbol-first §4.7), never `tau`
itself. `LowerTypeClosure` is representation-only: it is not derived from
`¬Object(τ)`, it is not a precondition for ordinary semantic operations on
`τ`, and its fidelity is a representation theorem:

```text
Fidelity (representation faithfulness):
  Norm(LowerTypeClosure(τ₁)) = Norm(LowerTypeClosure(τ₂))
    iff Norm_type(τ₁) = Norm_type(τ₂)
```

The representation is opaque: ordinary Pattern, Object navigation, and
Val1/Val2 inspection cannot observe any distinction beyond the `tau` API; the
lowering does not form a second observable identity system.

Evaluation of a type-valued binding yields the complete closure; it never
degrades into `Core(tau)` on its own. Observation is consumer-specific
projection of that first-class value — each consumer names which part it
needs, and no rule silently re-reads `tau` as `Q`:

```text
Read(type-valued place) = tau

Core-consuming operation
  -> consume Core(tau) = Q        (equality, keying, type-argument identity,
                                   ordinary compatibility, ordinary Pattern
                                   and namespace observation)
type-as-callee candidate acquisition
  -> CallSpace(tau) = V_τ
snapshot transport / copy / extend / inject
  -> the whole tau snapshot, including V_τ
```

(`ref`/`share` over a type value are governed by §5: borrow constructors are
privileged actual-place builtins and never implicitly degrade `tau` to `Q`.)

`@` is not part of this classification: it is the continuation-relative
name-reification operation that yields a lifetime value (canonical owner
`../lifetime/lifetime-policy-and-overload-boundary.md` §1–§2) and never a
borrow or a `type ref`.

When members refer to the current type, the closure has the binder-aware normal
form:

```text
tau = bind alpha. <Q, V_τ[alpha]>

Norm_type(tau)
  = bind alpha.
      <Norm(Q), Norm_V^alpha(V_τ)>

Norm_type^alpha(Self_τ)
  = BoundRef(alpha)

BoundRef(alpha) notin Children_owned
```

This is the normal form of the type closure, not `Norm(Q)` and not a second
shape for `Norm(Object)`. Alpha-renaming is non-semantic. Static-eval
generation must be terminating over the owned Object/member graph **after**
authorized binder back-references are erased, and the erased graph must be
acyclic with every back-reference bound and restricted to the authorized
static edge kinds; once materialized at runtime the owned graph must be
acyclic (`WellFounded_kappa`, §2.1):

```text
WellFounded_static(tau):
  Finite(GenGraph(tau))
  and Acyclic(GenGraph(tau))
  and AllBackRefsBound(tau)
  and BackRefsOnlyInStaticPV2Region(tau)
  where GenGraph(tau) = OwnedGraph(tau) with authorized BoundRef edges removed

WellFounded_runtime(tau):
  the materialized owned graph is acyclic
```

`BackRefsOnlyInStaticPV2Region(tau)` is the well-foundedness projection of the
enclosing-reference theorem (symbol-first §2.1.1): an upward reference from a
`V_τ` descendant to its enclosing `τ` follows the same `P × Val2`
descriptive-reference rule as a `Val2` referring to its enclosing `P` layer —
a static, non-owned `BoundRef` edge, never an owned edge, so it does not form
a `τ → A_F → () → τ` owned cycle and needs no separate recursive `V_τ` loop
condition.

The binder is not a `mu`-type, an equi-recursive type rule, or permission for
cyclic Object content.

There are exactly three ways a type-valued place moves from one snapshot to
another; they are not one family, and ordinary `let`/`=` does not secretly
perform structural extension:

```text
ordinary slot replacement:
    Write(slot, new_value)
    -- no old τ -> new τ semantic relationship is established
       and no structural incidence is added
    -- does not consume Open/construction authority, and does not
       guarantee preserving WellFormedTau: the result is judged
       independently and may be false

structural transformation:
    Extend_Σ(old, Δ) -> new
    -- the structural Core-changing transformation

place wrapper:
    Inject_Σ(r, Δ)
      = Read -> Extend -> Write
    -- `inject` is the place-level wrapper of `extend`
```

Each `tau` is an immutable snapshot; no operation mutates an existing closure.
A carrier's stored snapshot is replaced only by ordinary slot replacement
(§7.1) — a fresh `tau' = <Q', V_τ>` sharing `V_τ`, with no structural incidence
added. `WellFormedTau(<Q', V_τ>)` is then judged independently and may fail;
slot replacement is **not** a well-formedness-preservation theorem. A retained member keeps its own complete identity and owner. New closure
contributions must satisfy Home(TypeOf(v)) = TypeMemberScope(tau), either directly or through a
ReinstantiationWitness that constructs a new anchored instance. The original
value is not reparented; captures and internal identity edges follow
[closure replication](closure-anchored-replication.md).
The newly written value must satisfy the current closure's structural,
binding-reference and registration checks; no construction history substitutes
for them.

Copying a type-valued binding copies the
whole closure:

```text
TypeValue(T) = tau = <Q, V_τ>
let U: type = T
TypeValue(U) = Copy(tau) = <Q, V_τ>
Eval(T) = Eval(U) = tau
CoreView(tau) = Q
PatternView(tau) = Q
CallSpace(tau) = V_τ
```

Ordinary associated-member installation is ordinary **slot replacement**, not
`extend`:

```text
Write(place_or_projection_slot, new_value)
-- the new_value independently satisfies its own well-formedness judgments;
   the judgment may answer false — replacement is not WF preservation
```

No `old -> new` derivation is implied: the written `new_value` is a fresh
snapshot validated on its own structure. If a persistent implementation
reconstructs the parent snapshot to realise a slot update (old parent
-> reconstructed parent), that is a lowering / storage representation,
not a source-semantic transformation. Type contribution produces a new
callspace snapshot with Core fixed; extend handles structural change. The
legality of either modification is a contextual
operation judgment, separate from well-formedness:

```text
AdmissibleExtend_Γ(τ, Δ, τ')
=>
WellFormedTau(τ')

-- τ' is checked independently on its own structure; well-formedness
   is never inherited along a modification chain
```

Pure extension may preserve a construction root while producing a different
snapshot:

```text
tau_old = <Q_old, V_old>
tau_new = <Q_new, V_new>

Root(tau_old) = Root(tau_new)
  !=> tau_old = tau_new
  !=> V_old = V_new
```

An old copy retains `V_old`. No
`Root(tau) -> current mutable name binding -> current V` indirection participates in
type identity or call lookup.

#### Associated namespace is the Core member scope

The associated namespace used by ordinary invocation and `name::T` is the
existing structural member view of the complete type's Core:

```text
Q = Core(T)
AssociatedNamespace(T) = MemberScope(Q)
Val2(AssociatedNamespace(T)) = Val2(Q)
AssociatedName(T, s) = NameCoord(AssociatedNamespace(T), s)

TypeMemberScope(T) = /tau(T)
TypeMemberScope(T) != AssociatedNamespace(T)
```

MemberScope(Q) denotes Q's structural namespace root and selector view, not a
new Object, separate companion namespace, or a reverse defining-name lookup.
The view observes the held snapshot. When navigation or a write addresses an
actual structural name, it retains that route's resolved root/Place and resident
generation; Core equality does not merge those coordinates or grant a Place.
AssociatedName names a coordinate, not evidence of retention, initialization,
access or writability. Reading its resident requires the ordinary checks.

The `/tau(T)` scope is instead the complete bound implementation hierarchy
used for registered callable classifiers. Home(Type(c)) = /tau(T) locates
that classifier; ordinary invocation of c reads Val2(Core(Type(c)))[()].
Neither coordinate is V_T. A graph carrier named type-associated companion
represents the Core member view; its storage node introduces no independent
semantic namespace. Field-family registration may separately use /tau(T) for
stable family identity without changing the associated name's root.

### 2.3 Rank and typing naturality

The semantic layers compose as one principle:

```text
Object structural core:    ordinary structural content is governed by
                           Object = <Val1?, P, Val2>
Rank-indexed closure:      complete types are rank-indexed closures
                           tau = <Q, V_τ> over that Object material
Declared-result rank:      the rank of a callable's result is solely its
                           declared result rank:

                           Rank(result(F, x)) = DeclaredResultRank(F, x)

                           There is no default rank-preservation rule and
                           no RankShift(F, n, m) as a second mechanism;
                           evaluation stage, description depth, and carrier
                           form cannot apply UniverseSuccessor implicitly;
                           place projection, borrow lifting, and type
                           formation compose by the ordinary typing/
                           naturality laws below
Description-rank stability: P/Val2 formation and transformation do not
                           apply UniverseSuccessor; the description layer
                           stays at rank 0 regardless of what rank the
                           described type inhabits
```

**DeclaredResultRank.** The rank of a callable's result is solely
`DeclaredResultRank(F, x)` — the rank declared by `F`'s signature or
formation rule. No ordinary operation silently changes the semantic
*category* of what it transports: an ordinary Object stays an ordinary
Object, a complete type value stays a complete type value, and a borrow
view stays a borrow view. But rank is not preserved by default; it is
determined by declaration. The evaluation stage, description depth, or
carrier form never injects a rank silently.

The rank invariant is:

```text
Rank(result(F, x)) = DeclaredResultRank(F, x)

evaluation stage / description depth / carrier form
  cannot apply UniverseSuccessor implicitly
```

A family may be rank-parametric:

```text
RankTransparent(F)
  iff ∀n. F : U_n -> U_n
```

`RefTy` and `ShareTy` are `RankTransparent` (`T : U_n ⊢ RefTy(T) : U_n` and
`T : U_n ⊢ ShareTy(T) : U_n`): borrow-type formation preserves the operand's
rank. This is a property of their declared result, not a default rule.
Field projection follows the declared result type:
`inner : T ref -> A ref` has `Rank(result) = rank(A ref) = rank(A)`. When
`rank(A) ≠ rank(T)` this is a *declared* rank shift — the signature itself
says `T ref -> A ref` — never an implicit rank change injected by the
projection step. `TypeOf` is genuinely rank-changing
(`TypeOf(type) = type_1`). Ordinary cross-rank functions simply follow
their declared result rank.

**DescriptionRankStability.** The `P × Val2` description layer is orthogonal
to universe rank. Even when `P/Val2` describes a type `τ : U_n`, the
description formation and transformation themselves do not induce universe
lifting:

```text
DescriptionRank(P, Val2) = 0

P/Val2 formation and transformation
  do not apply UniverseSuccessor
```

This is the structural invariant. It does not introduce a fourth Object
coordinate; `P` and `Val2` are structural material within
`Object = <Val1?, P, Val2>`, and their description activity stays at rank 0.

Borrow lifting and type observation commute (naturality of `TypeOf` with
borrow formation):

```text
TypeOf(Ref(p))   = RefTy(TypeOf(Read(p)))
TypeOf(Share(p)) = ShareTy(TypeOf(Read(p)))

rank(RefTy(T))   = rank(T)
rank(ShareTy(T)) = rank(T)
```

`RefTy(T)` / `ShareTy(T)` (general operand form, with `RefTy(U_n)` /
`ShareTy(U_n)` as the universe-object specialization) are defined in
`../lifetime/lifetime-policy-and-overload-boundary.md` §2; the rank equations
state that borrow-type formation preserves the semantic rank of its pointee.

The borrowed-extraction law of
`../patterns-overload/pattern-values-relational-semantics-and-extraction.md`
§14 is a theorem of this section:

```text
E_ref(Ref(p)) = Ref(ProjectionSlot_E(p))

where ProjectionSlot_E(p) = ProjectionSlot(Target(Ref(p)), pi_E)
```

Field functions follow the same declared result-rank family:

```text
inner : T        -> A
inner : T ref    -> A ref
inner : T share  -> A share
```

When `A` is itself a type position, the same family reads at the type rank.
The field function is the same borrowed-extraction theorem as above — the
argument is a `T ref` borrow handle, and the projection is taken through its
target, never through the handle's own carrier place:

```text
inner_ref : T ref -> A ref

inner_ref(r)
  = Ref(
      ProjectionSlot(
        Target(r),
        inner
      )
    )
```

When `inner : T -> type`, this mechanically yields:

```text
inner_ref : T ref -> type ref

object ref.inner
  = Ref(
      ProjectionSlot(
        Target(object ref),
        inner
      )
    )
```

By the naturality law `TypeOf(Ref(p)) = RefTy(TypeOf(Read(p)))`:

```text
TypeOf(object ref.inner) = type ref
```

Contrast the type formation on the extracted value:

```text
(object.inner) ref : type = RefTy(value(object.inner))
```

`object ref.inner` is a `type ref` borrow **instance** (its target slot
currently holds a type); `(object.inner) ref` is the borrow **type** of the
extracted value. The two are distinct and both follow from the same
declared result-rank family. Neither derivation asks whether `type ∈ Object` or
`type ∉ Object` first: the borrowed field projection
(`inner_ref(r) = Ref(ProjectionSlot(Target(r), inner))`) is admissibility-
judgment-driven and simply yields `T ref -> type ref` when the field type
happens to be `type`
(`NoSemanticDispatchByCarrierMembership`).

Finally, type formation and borrow formation are distinct operations and stay
separate, each following its declared result rank: `t ref` is type formation
(a type-forming
overload yielding the borrow TypeValue), while `t |> (type ref)` is borrow
formation (yielding a value `r : type ref`). The member-phase split and the
no-implicit-borrow rule are canonical in
`../lifetime/lifetime-policy-and-overload-boundary.md` §2 and §5.1 below.

## 3. Value judgment versus place judgment

Value-side formation and Place-side borrowing/writing use distinct judgments.
NameExpr formation observes the current type value through a resolved structural
root identity; it does not acquire a writable parent Place.

Value evaluation:

```text
Γ ⊢ x ⇓ v
```

means an expression / rank / type expression evaluates to value `v`.

Place resolution:

```text
Γ ⊢ x ⇐ p
```

means a selected write/inject operation resolves `x` to its actual writable
Place `p`. This is not the judgment for forming a qualified NameExpr: formation
checks the current root type value and creates the child Place before borrowing.

These are not interchangeable. Canonical creation beneath an open type name
uses the resolved value/path, then separately borrows the new typed Place:

```lang
mut let f_ref = (let f::t:type) ref;
f_ref = ...;
```

A structural name S already reads its complete named type. Its type-level
Place is reached with `S |> (type ref)` under the selected privileged
borrow-forming operation. There is no intervening binding-wrapper value or
`.type` projection. A by-value type observation never recovers a Place.

### 3.1 General value binding resolves bindings first

The ordinary rule:

```lang
let r = expr;
```

is:

```text
v := evaluate(expr)
fresh binding b_r with fresh Place p_r
install ordinary resident v at p_r; bind lexical name r to b_r
```

When `expr` is a source path, value evaluation is not direct value naming:

```text
source path
  -> resolve name binding
  -> read the selected value / PatternValue from that name binding
  -> bind the value to the destination name binding/Place
```

Thus:

```lang
let a = b;
```

binds the exact resident read through b's resolved binding into a's fresh
destination Place. It does not alias the bindings or merge their places.

Formally:

```text
resolve(b) = s_b
read(s_b)  = v
fresh NameBindingId s_a
fresh PlaceId p_a
--------------------------------
bind(a, v)
```

The source carrier `s_b` is not stored as part of `v` after evaluation.
Provenance may mention it; semantic value identity does not. Consequently no
ordinary binding path may recover associated operations by mapping
`TypeValueId` back to an “original defining name binding”. A source name binding projects
`tau=<Q,V_τ>` directly; an already-held type value uses its intrinsic
`CallSpace(tau)=V_τ`. Ordinary Pattern/namespace observation separately follows
`Core(tau)=Q`.

The same separation applies inside derived semantic material. A struct field,
callable signature, canonical argument key, or extraction view that denotes a
type observes `Core(tau)=Q`; `Addr(Norm_type(tau))` remains available where the
language requires whole-snapshot identity (transport and distinguishing
shared-root snapshots):

```text
field source path
  -> carrier name binding
  -> read TypeValue tau
  -> record Core(tau) = Q as field-type identity
```

An implementation may temporarily retain the carrier name binding for graph
navigation or provenance, but it is not part of field-type equality,
Pattern-head identity, or struct construction-material identity. Consequently
`(uint8 field) struct` and `(T field) struct` have the same field-type material
after `let T: type = uint8`; a reverse `TypeValueId -> original name binding` lookup
would incorrectly make ordinary binding observable.

Extraction interfaces follow the same split. Their semantic owner/type
coordinates are the owner `TypeValue` and Pattern identity. A graph carrier may
still be present to reach installed field projection name bindings, but
`semantic_eq` cannot distinguish two extraction shapes merely because the same
type value is carried by different bindings.

Ordinary Pattern applicability follows the same rule. A written Pattern name is
resolved forward to its `PatternValue`; the actual argument contributes the
`PatternValue` reached through its evaluated type/value. Matching compares
those identities, not the carrier spellings. Hence a formal `_ uint8` accepts a
type value read through `T` after `let T: type = uint8`; comparing the strings
`"uint8"` and `"T"` would be name-category-first resolution in disguise.

The same rule applies to an externally owned pattern value:

```lang
mut let t1_ref = (let t1::t:type) ref;
t1_ref = bool;
```

first resolves the existing parent t and checks its current type's OpenHere,
selector legality, freshness and ordinary access/type/path well-formedness.
Typed name creation yields NameExpr and an uninitialized Place. Explicit ref
uses its declared type without reading; ordinary write initializes it with
bool's complete type. The initialized value retains its own Pattern navigation
and owner. Failed initialization leaves the Place uninitialized, subject to the
ordinary enclosing transaction. While it remains uninitialized, Close rejects it.

Literal syntax is the explicit exception only to source-path resolution. It
still evaluates to a value; an ordinary lexical let then uses lexical binding,
while a structural name is initialized by a separate explicit-ref write. In the schematic
future spelling `let a = 'a';`, the left `a` is a binding name while the right
`'a'` denotes a character literal; matching textual content does not make them
the same object. The frozen lexer does not yet accept that character spelling
(§4.1.3).
Pattern values have no analogous standalone literal syntax, so same-spelled
name binding paths and pattern diagnostic names must be kept especially distinct.

### 3.2 Name resolution and navigation

#### 3.2.1 Head selection: bare versus explicitly anchored

    bare head: nearest same-spelled binding, then stop
    explicit head: the written anchor, with no outward retry
    resolved binding -> named type -> context-directed member projection

A failed projection, empty group, hidden view, or non-callable member does not
restart name resolution. The same source binding remains selected for value,
type, call and Pattern uses.

#### 3.2.2 Value navigation and borrowed structural navigation

Ordinary value navigation selects existing members of an Object. A group
consumer uses the ordinary member projection appropriate to the navigation
position; it does not assume one distinguished object/type facet.

Borrowed structural navigation begins from an actual type reference and
retains its target Place, resident generation and ordinary capabilities.
It may identify a prospective final slot, but is not a prerequisite for typed
NameExpr formation. Value-side formation uses the resolved structural identity
and current value's OpenHere, not parent Writable. Neither path can
turn missing-name occupancy into an optional ordinary language value.

Each resolved host layer is retained for exposure and policy checks; the whole
host chain must admit the access. Different host chains are not silently merged
merely because they reach one terminal. Neither navigation route introduces
lookup fallback or a second name authority.

## 4. Ordinary type-value binding

Type-value binding is the general value-binding rule under a `type`
expectation, not a separate assignment mechanism. The form:

```text
let T: type = uint8
```

means:

```text
NameBindingId(T) = fresh structural binding identity
place(T) = fresh writable place at current lexical level
value(T) = value(uint8)
type_value(T) = type_value(uint8)
pattern_value(T) = pattern_value(uint8)
```

This must be read precisely:

```text
T is not a fresh nominal type.
T is not a name binding alias.
T has fresh place identity.
T may evaluate to an existing type value.
```

`T` is a new binding with its own fresh, current-level writable place. Its *type
value* is the value read through `uint8`, while its *place* is its own. Binding to an existing
type value does not generate a new type, and it does not forward to `uint8`'s
name binding or place.

An ordinary meta instance name is its instance type value tau_M, with
Root(Core(tau_M)) = M. It cannot directly carry an arbitrary value, borrow, or
external type instead. Such payloads belong in ordinary Val2 and retain their
own type/root or target/escape obligations. P1 meta retains the instance under
OpenHere, which governs acquisition of its mut view; plain let completes and
closes it. Ordinary names and payload Places retain their independent policy
and value facts. See the construction owner, section 4.4.

Consequently, associated-member creation through `T`:

```text
mut let f_ref = (let f::T:type) ref;
f_ref = ...;
```

creates a typed uninitialized Place under T's resolved structural root after
the value-side formation checks. Explicit ref
and ordinary write then initialize it. It does not perform a parent +=, write
to place(uint8), or bypass the copied value's existing OpenHere rules. Parent
writability is not required; equal values do not identify name coordinates.

NameExpr formation is value-side; borrowing and initializing its resulting
Place are separate operations. Structural extension is different: it is
the pure value transformation `extend`, while `inject` is the explicit
read--extend--write wrapper defined in the symbol-first construction document.

### 4.1 Abstract literal denotations and concrete machine types

Lexical family, semantic denotation type, and concrete machine-semantic type
are three different layers:

```text
lexer/Normalized-AST carrier:
  LiteralFamily = Integer | Float | String

abstract semantic literal types:
  integer | real | character

implementation lookup carriers:
  AtomicBuiltinType T
    = Uint | Int | Float | Buffer | Str

  NumericTypeKey Tnum
    = NumericFamily x width
```

`LiteralFamily`, `AtomicBuiltinType`, and `NumericTypeKey` remain useful Rust
and registry shapes, but none defines the initial source-language semantic
type. The abstract denotation types use the existing type ontology:

```text
TypeRole(integer)
TypeRole(real)
TypeRole(character)

NoSeparateLiteralTypeUniverse
```

`integer`, `real`, and `character` are ordinary complete type values satisfying
the existing Type role. They do not form a parallel `LiteralType` universe or a
fourth semantic type ontology; ordinary construction consumes them through the
same Type/call machinery as other type values.

The canonical semantic path for integer/real literals and any future character
token is:

```text
token spelling
  -> ParseLiteral
  -> AbstractLiteralValue : integer | real | character
  -> ordinary Convert/Construct to a concrete machine-semantic Type
  -> optional same-Type Migrate/Materialize of that concrete Type
     for another stage/policy view
```

The optional final edge never applies while the value still has abstract Type
`integer`, `real`, or `character`; their associated cells are normatively
deleted in §4.1.4.

The initial semantic result is normatively compile-known:

```text
ParseLiteral(tok) = v : Tlit
Tlit ∈ {integer, real, character}

InitialLiteralPolicyPair(v) = compile:compile
```

This is the unique initial pair, not one optional view among compile and
runtime alternatives. `ParseLiteral(tok)` therefore cannot directly produce
`integer@runtime`, `real@runtime`, or `character@runtime`. Whole-slot
`PolicyMode` is supplied by the surrounding binding/call demand formation; it
is not inferred from the token. Runtime availability requires the later
ordinary construction/materialization path described below.

Ranked strings retain their existing, separate canonical path:

```text
ParseRankedString(tok) = v : str
InitialStringPolicyPair(v) = compile:compile
```

Thus a ranked string denotes `str@compile`; it does not first denote
`character`, does not belong to the three abstract scalar denotation Types, and
does not imply `str ref`, hidden storage, or a lifetime extension. The current
`LiteralFamily::String` carrier preserves that source family. Whether the core
bootstrap has installed a concrete `str` Type binding is an implementation
availability question, not the owner of this semantic path.

The frozen lexer continues to preserve spelling only. It does not choose
width, signedness, precision, encoding, overflow behavior, or a machine type.
Expected-type-driven insertion of the ordinary construction is a separate
surface/diagnostic question; an expected `uint32` does not retroactively make
`1` start with type `uint32`.

#### 4.1.1 `integer`

```text
Denote(integer) = Z
```

Radix and digit separators affect parsing, not the semantic type:

```text
42
0x2a
0b101010
  -> the same integer value 42

0xff
  -> integer value 255
```

The leading minus sign remains ordinary prefix-operator material. `-1` applies
unary minus to `1 : integer`; it is not a separate signed literal token/type.

#### 4.1.2 `real`

Every finite decimal or hexadecimal source real literal first receives an
exact denotation:

```text
0.1      : real = 1/10
0x1.8p1  : real = 3
```

`ParseLiteral` performs no binary32/binary64 rounding. The complete
mathematical carrier of `real` remains extensible, but it must contain an exact
representation for every finite source real literal. An exact rational-like
carrier is sufficient for the current finite forms.

#### 4.1.3 `character`

`character` is an abstract source-character domain:

```text
character != char8
character != char16
character != char32
```

The final character source spelling, escape rules, and whether the carrier is
exactly the Unicode scalar-value set remain Open. Ranked strings
remain the independent `str@compile` path above and are not reinterpreted as
character tokens.

#### 4.1.4 Construct versus materialize

The terminology boundary is normative:

```text
Convert / Construct
  may change Type

Migrate / Materialize
  preserves Type
  changes only stage/policy availability
```

For example:

```text
0.1 : real
  -> Construct_float32
  -> rounded float32 value

float32@compile
  -> Materialize_float32
  -> float32@runtime with the same typed value semantics
```

Same-Type runtime materialization remains an ordinary associated callable
family, not a compiler special case. The general mechanism is:

```text
RuntimeMaterializable(T)
iff exists ordinary non-deleted callable realizing
    a legal same-Type static -> runtime transition of T
```

The three canonical abstract denotation type values carry an intrinsic negative
fact in their immutable callspace snapshots:

```text
AbstractLiteralNoRuntimeMaterialization:

CanonicalLiteralType(integer)   = tau_integer
CanonicalLiteralType(real)      = tau_real
CanonicalLiteralType(character) = tau_character

CallSpace(tau_integer) contains:
  integer@compile -> integer@runtime => delete

CallSpace(tau_real) contains:
  real@compile -> real@runtime => delete

CallSpace(tau_character) contains:
  character@compile -> character@runtime => delete

RuntimeMaterializable(integer)   = false
RuntimeMaterializable(real)      = false
RuntimeMaterializable(character) = false
```

Parsing an abstract scalar literal assigns exactly `tau_integer`, `tau_real`,
or `tau_character`; it does not contextually retarget the literal to a later
snapshot. Each intrinsic `delete` member is an ordinary associated callable and
participates in the ordinary resolver/must-select pipeline.

The result follows from the existing complete-type snapshot invariants.
As established in §2.2 and the immutable callspace rule in
[`symbol-first-meta-construction-and-pattern-injection.md`](symbol-first-meta-construction-and-pattern-injection.md),
`V_tau` is fixed when `tau` is formed and later associated contributions cannot
mutate that existing snapshot:

```text
CallSpace(tau) = V_tau                         at tau formation
LaterAssociatedContribution(tau, F)
  -> forms some new complete tau' with V_tau'
  -> tau' != tau
  -> CallSpace(tau) remains V_tau
```

An `extend` may therefore form a different complete type value with a different
`V_tau'`; it does not alter the canonical literal type value or the callspace
used by canonical literal values. Consequently no later declaration can add a
non-deleted same-Type materializer to `tau_integer`, `tau_real`, or
`tau_character`.

These deleted cells therefore belong to each canonical Type's ordinary
associated callspace. Their non-overridability follows mechanically from
immutable complete-type snapshots, not
from an ad hoc abstract-literal checker branch or an unformalized global ban.
Abstract denotations must first enter ordinary construction to another,
concrete machine-semantic Type. A normal integer path is therefore
`integer@compile -> Construct_int32 -> int32@compile -> Materialize_int32 ->
int32@runtime`; `integer@runtime`, `real@runtime`, and `character@runtime` are
not legal same-Type results.

#### 4.1.5 Stage-invariant machine semantics

Once a concrete Type fixes machine semantics, those semantics do not vary by
stage:

```text
StageInvariantTypeSemantics

[[op]]_T : T^n -> Outcome(T)
Outcome = Value(v) | Error(e) | Trap(t) | ...
```

There is no separate `[[op]]_(T, stage)`. Compile evaluation of `float32` must
apply the same per-operation rounding and outcomes as runtime `float32`;
integer wrap/saturate/trap/checked behavior likewise belongs to the concrete
Type or operation, not to compile/runtime stage.

```text
MaterializationPreservesTypedValue

v : T
--------------------------------
[[Materialize_T(v)]]_T = [[v]]_T
```

The existing core bootstrap and helper remain a bounded implementation subset:
`uint8`, `uint16`, `uint32`, and `float32` are installed concrete lookup
targets, while `str` is not. `AtomicBuiltinTypeRegistry` and
`NumericTypeRegistry` resolve those concrete targets only; they neither choose
an abstract literal's identity nor implement contextual construction.

## 5. Borrow views

There is no declaration form that makes two bindings share one name binding identity
or one place. Shared observation is expressed by the borrow constructors `ref`
and `share`; `@` reifies a `LifeName` at the current semantic-continuation
position and is not a borrow representation.

### 5.1 `ref` and `share` are privileged actual-place builtins

The value-operand families below assume a resident value is available. Explicit
ref consuming an uninitialized typed NameExpr uses the ordinary initial-borrow
realization of §7.1.1 instead: it obtains the existing Place and PlaceType without
Read. This is a borrow-forming realization, not another Object or operator role.
In particular, PlaceType(q)=type does not supply a type-value operand for type
formation, and DeclaredPolicy(name)=const is not an actual const T resident
fed into the value family's delete cell. The distinction is made by the operand
and Place state, never by retrying after a selected value operation fails.

`ref` and `share` are ordinary overloaded callable/operator families on their
operand — not a single meta-stage operation. Each
operator has two overload roles (canonical owner
`../lifetime/lifetime-policy-and-overload-boundary.md` §2): a **type-forming**
member, selected for a type operand, that forms the borrow **type** value
(`t ref` / `t share` as TypeValues), and a **borrow-forming** member inside
the formed borrow type's callspace that produces the borrow **instance**. The
member phases are distinct:

```text
type-forming member:    meta
  T : U_n ⊢ T |> ref = RefTy(T) : U_n
      -- produces the borrow TypeValue T ref, indexed by the operand type
         itself (not by the classifying universe); the borrow-type
         constructor RefTy(T) is defined in
         lifetime-policy-and-overload-boundary.md §2

borrow-forming members: concrete stage declarations / admissible projections
  E |> RefTy(T)
      -- forms the actual borrow instance; a selected ordinary
         builtin/default realization, the only family role that may
         obtain PrivilegedActualPlace
```

Only the selected borrow-forming defaults of `ref` / `share` may obtain the
actual's place (`PrivilegedActualPlace(ref-family)` and
`PrivilegedActualPlace(share-family)`; canonical owner
`../lifetime/lifetime-policy-and-overload-boundary.md` §2). `@` does not use
this acquisition path: it reifies a continuation-relative `LifeName`, including
for temporaries. An ordinary user
function that spells the same formal head cannot obtain that place.

There is no global `E ref = Ref(Read(E))` law. The result depends on the
selected overload and on whether that overload's default implementation
exercises its place privilege:

```text
ordinary candidate preparation
    (Pattern / type / Policy matching on the actual value)

ordinary overload selection
    -> unique selected builtin/default

if SelectedBuiltinRequiresActualPlace:
    p := PrivilegedActualPlace(actual)
    if no stable place available:
        InvocationFailure(NoCarrierPlace(actual))
        -- a precondition failure AFTER selection: not candidate-space
           repair, not candidate removal, not overload reopening,
           not fallback
-> execute default
```

The type-forming `ref` / `share` members do not require a carrier place, so a
stable-place-less temporary TypeValue still participates in type-forming
overload selection.

For a type-valued binding `t : type`, `t ref` selects the **type-forming**
overload and yields the TypeValue `tau_(t ref)` (the borrow type of `t`), never
a borrow instance; the borrow instance over the type-level place is produced
only by invoking that borrow type explicitly with `t |> (type ref)` (§5.2).
For an ordinary `Val1`-bearing value, the selected borrow-forming default
obtains the actual's place via its privilege (§5.1.0).

#### 5.1.0 The selected borrow-forming default obtains a privileged actual place

The selected borrow-forming default of `ref` (or `share`) obtains the place of
the actual — not a second place source derived from `Read(E)`. `ref` and `share`
`PrivilegedActualPlace(ref-family)` / `PrivilegedActualPlace(share-family)`;
continuation-relative `@` does not acquire a place.

```text
ordinary candidate preparation:
    evaluate actual for Pattern / Policy / overload matching

selected privileged borrow default:
    p := PrivilegedActualPlace(actual)
    return Ref(p) / Share(p)
```

There is no global `E ref = Ref(Read(E))` law: overload selection runs first
(§5.1), and only the selected builtin default exercises its place privilege.
An ordinary user function that spells the same formal head cannot obtain the
same place. The place `p` is the actual's place under the selected overload;
how `p` is formed for a given expression category is a place-judgment matter,
not a second `Read(E)`-derived place source.

A type-valued operand does **not** reach the borrow-forming path: for
`t : type`, `t ref` is type formation (§5.2) and yields the TypeValue
`tau_(t ref)`, never a borrow of `Core(tau)`. Reaching the type-level carrier
place explicitly invokes `t |> (type ref)`, whose borrow-forming member obtains
`place(t)` and yields the borrow instance; `ref` never falls back to a carrier
slot and never elaborates a higher-level `(type ref)` implicitly (§5.2).

A value with no stable place — a freshly computed temporary that resides
nowhere and carries no borrowable identity — supplies no place. Overload
selection still runs: the temporary's type participates in ordinary
Pattern/type/Policy matching. Only after the unique borrow-forming builtin is
selected does place acquisition run; with no place available the invocation
fails as a precondition failure — `InvocationFailure(NoCarrierPlace(actual))`
— never as candidate removal, overload reopening, or fallback. `ref` never
materializes storage on the writer's behalf and never silently retargets to a
carrier slot; a temporary must first be bound to a named place before it can
be borrowed.

`ref` is an ordinary overloaded callable family member. It does not ask which
binding slot the value came out of, and does not consult, capture, or export
it. Therefore:

```lang
let t = uint8;
let r = t ref;
```

Here `uint8` evaluates to the complete type value `tau_uint8`, so `t` is
type-valued. `t ref` selects the type-forming overload and yields the TypeValue
`tau_(uint8 ref)` — not a borrow instance, and not a borrow of `Core(tau)`.
Reaching the type-level carrier slot of a pure type binding uses
`t |> (type ref)` (§5.2). This also applies to a structural named type;
the binding itself is not an intermediate Object.

`share` differs from `ref` in the capability it grants, not in the judgment it
uses: a `share` view admits reading and passing but is not an assignable place
and cannot be an `inject` target (§5.5).

#### 5.1.1 Borrowing an ordinary group or pattern value

A borrow observes the actual operand's Place. Reading or borrowing an ordinary
OverloadGroup does not implicitly select one of its members or project a type.

    g : OverloadGroup
    g ref : OverloadGroup ref

An explicit type projection selects a complete pattern value under the ordinary
type consumer's rules. TypeRole and NamespaceRole remain judgments over the
selected value's Core, not classifications of the group. A type-expected
position may elaborate AsType where already specified; candidate enumeration
does not insert a conversion merely to make a competing candidate applicable.

Borrowed member projection follows ordinary reference/field rules and keeps
the actual target identity. A complete pattern value's captured callspace is
not replaced by searching a source binding or registry.

#### 5.1.2 No implicit borrow formation

Borrow formation is never candidate adaptation, structural repair, policy
migration, or automatic argument passing:

```text
Object =/=> Object ref | Object share
OverloadGroup =/=> OverloadGroup ref | OverloadGroup share
type   =/=> type ref   | type share
```

An overload requiring `T ref` or `T share` is applicable only when the actual
argument already is the corresponding borrow observation. The compiler may not
invent `ref`, `share`, or `@` merely to make a candidate applicable. Borrow
formation requires the explicit operator in the source/normalized expression;
ordinary value copy such as `let b = a` creates no borrow edge. The established
fixed points and weakening on an **existing** borrow (`ref ref`, `share share`,
and `ref share`) remain valid (§5.3), as does the separately specified implicit
`self` capability of a callable frame; neither is ordinary argument repair.

#### 5.1.3 The generated `ref` / `share` instance families

The borrow-forming defaults inside the formed borrow type's callspace are not
an ad hoc pair of builtins; they are generated instance families with a fixed
policy matrix for resident value operands. The `ref` family has two input shapes (`T`, `T ref`), two
member result-policies (`mut`, `const`), and three formal PolicyMode patterns
(`mut`, `const`, `plain`):

```text
GeneratedRefInstanceFamily(T):

member  formal  actual T       actual T ref
-------------------------------------------------
mut     mut     default        ref fixed-point
mut     const   delete         delete
mut     plain   delete         delete

const   mut     default        ref fixed-point
const   const   default        ref fixed-point
const   plain   default        ref fixed-point
```

`ref fixed-point` is not "borrow again". It is the ordinary candidate
realization of the existing fixed-point theorem
`Borrow_ref(Borrow_ref(p)) = Borrow_ref(p)` (§5.3): the old
`{ object ref; }` declaration is demoted to an ordinary forwarding body of
that theorem, not a new primitive. For an actual `T` shape, the `default`
cells are the selected builtin/default borrow-forming members, and only the
selected builtin/default holds `PrivilegedActualPlace(actual)`:

```text
PrivilegedActualPlace(actual)
    -- held only by the selected borrow-forming default,
       not by the formal pattern, not by ordinary parameter semantics
```

The formal head does not materialize a borrow source:

```text
FormalHeadDoesNotMaterializeBorrowSource:

object : T
    = candidate extraction head + formal policy pattern

    !=  first move actual into a parameter-local T slot,
        then borrow that slot
```

The selected borrow-forming builtin observes the call-site actual place, not
the ordinary post-pass parameter binding place. This is the builtin's place
privilege (§5.1.0), not general parameter semantics.

The `share` family is simpler and carries no write capability:

```text
GeneratedShareInstanceFamily(T):

T        -> T share     default
T share  -> T share     fixed point
T ref    -> T share     legal weakening
T share  -> T ref       no candidate
```

This is the §5.3 `Borrow_k(Borrow_j(q)) = Coerce_{j->k}(Borrow_j(q))` algebra
expressed as a generated family:

```text
ref ref       = ref
share share   = share
ref share     = share
share ref     = no candidate
```

and the type-value layer obeys the same capability direction:
`ShareTy(RefTy(T))` is a legal weakening; the reverse strengthening is
forbidden. The capability conclusion is explicit:

```text
share exposes no write operation.
share does not acquire internal mutability merely by being shared.

SharedObservation
≠
AliasWrite
```

If the language has an alias-write / internal-mutability path, it must come
from that independent capability system, not from `share`. `T share` also
provides no `=` / assignment family (`AssignmentFamily`,
`symbol-first-meta-construction-and-pattern-injection.md` §4.5.1): a
`share`-valued left side yields no applicable assignment overload, never a
selected write that then fails `Writable`.

### 5.2 Reaching the type-level place: `t |> (type ref)`

The type-forming `ref` overload over a type value forms the borrow **type**
value; it never borrows `Core(tau) = Q`. Reaching
the type-level carrier place is an explicit invocation of the formed borrow
type value itself — `t |> (type ref)` — never an implicit fallback of `ref`:

```text
t |> (type ref)     -- explicit higher-level ref formation over the type-level place
t |> (type share)   -- explicit higher-level share formation
```

`type ref` is the ordinary type construction `type |> ref`, and `type share` is
`type |> share`; they are not special tokens and not produced by `@`. Each
operator has a type-forming overload (forming `t ref` / `t share` as
**TypeValues** when applied to a type operand) and a borrow-forming overload
inside the formed borrow type's callspace (producing borrow **instances**
`r : t ref`). The borrow-forming defaults are the privileged actual-place
builtins (`PrivilegedActualPlace(ref-family)`,
`PrivilegedActualPlace(share-family)`) that may obtain the actual's place
(canonical owner `../lifetime/lifetime-policy-and-overload-boundary.md`
§1–§2); the type-forming member needs no privileged actual-place access. An
ordinary user function spelling the same formal head cannot.

The domain restriction remains:

```text
E |> (type ref) is undefined when E has no carrier place
t |> (type ref) is not a general PlaceOf(E) available on every expression
an invocation-generated result name has its own ordinary actual Place
```

`@` is a continuation-relative name-reification operation that yields a lifetime value, never a
borrow view and never a `type ref`
(`../lifetime/lifetime-policy-and-overload-boundary.md` §1–§2.1). Reaching the
carrier slot explicitly uses `t |> (type ref)`.

#### 5.2.1 `t ref` is type formation; `t |> (type ref)` is the borrow instance

For `t : type`, the two spellings land in different semantic categories. `t ref`
(`= t |> ref`) selects the global `ref` **type-forming** overload and yields the
TypeValue `tau_(t ref)` — the borrow type of `t`, never a borrow instance. The
borrow **instance** is produced only by invoking that borrow type on the
binding's place, because the ordinary read of a pure type slot has already
selected the pattern facet and the carrier slot is not reachable from the read
value:

```lang
let t: type = uint8;

t ref               // type formation:  TypeValue tau_(uint8 ref)
t |> (type ref)     // invocation:      borrow instance r : type ref
                    //                  Target(r) = place(t)
```

`t ref` is not a mistake to be corrected; it is type formation over the type
value that was read. A value-directed meta-function has no business guessing
that the writer actually meant the slot underneath. `t |> (type ref)` is the
explicit invocation that reaches the type-level place and yields the borrow
instance.

The operator choice is decided by what the surface means, never by type-rank:

| what the expression reads | `E ref` | `E |> (type ref)` needed |
| --- | --- | --- |
| ordinary value with `Val1` | borrow of the complete value-bearing object | no |
| type-rank value with `Val1` | borrow of the complete object, named by its host Pattern | no |
| pure pattern value | `ref` of that pattern value | only to reach the carrier slot |
| pure `type` slot | type formation: the TypeValue `t ref` (the borrow type) | yes — for a borrow instance over the carrier place |

The `Val1` column decides only *which path applies* — the borrow-forming `ref`
on the value versus type formation / explicit `type ref` invocation. It never
means that the result of `ref` descends to the `Val1` sub-object: in the rows
that yield a borrow instance, the view's referent is the complete object that
`Read` produced (§5.1); the pure `type` row yields a TypeValue, not a view.

Consequently the compile stage offers no implicit borrow formation for an
operand that has a `Val1` payload — `s ref` already does that job. `@` is not a
fallback for `ref` and is not a borrow constructor
(`NoImplicitBorrowFormation`).

#### 5.2.2 Initialized type names, meta references and mut confirmation

Contextual meta qualification currently has the narrow domain type and type ref.
It is not a fourth PolicyMode point and does not generalize to arbitrary meta X
ref. P2 meta remains evaluation stage. Qualifying an ordinary type value does
not turn its name into a MetaInstance or change its root identity.

NameExpr formation and borrowing are separate. The cases are:

| Name's Place state | Initialization/mutable-view consumer (non-mut views remain ordinary) |
| --- | --- |
| Uninitialized(type) | InitialTypeSlotRef: pending one-shot initialization authority only |
| Initialized(T:type) | Ordinary direct mut type ref, or an explicit meta type ref view |

InitialTypeSlotRef names the existing initial-borrow judgment, not a new Object
or policy. It does not read a nonexistent T, cannot use meta qualification to
replace its initialization authority, and grants no replacement after commit.

For an initialized name n, direct mutable borrowing remains available:

    q = BindingPlace(n), Read(q) = T : type
    OpenHere_Sigma(T), Writable_Sigma(q)
    ordinary borrow capability, AccessLegal(q), LifetimeLegal(q)
    ---------------------------------------------------------
    DirectMut(n) : mut type ref
    Target(DirectMut(n)) = q

Explicit MetaRef(n) instead retains the actual target Place, borrowed type
generation/construction subject and the source at which openness is rechecked:

    r_m : meta type ref
    Target(r_m) = q
    OpeningSubject(r_m) = the borrowed T/generation's construction subject
    MetaOpen_Sigma(r_m) iff OpenHere_Sigma(OpeningSubject(r_m))

Its formation uses ordinary actual-Place, borrow, access and lifetime checks.
It stores no enduring writable proof. Replacement of q's resident cannot silently
retarget OpeningSubject; ordinary generation invalidation and explicit rebind
rules apply. A saved identity may remain meaningful after Close while every
writable use of it fails.

The meta-qualified ref family admits ordinary writable candidates, not just a
read marker. Their applicability and write Pre require current facts:

    MetaWriteApplicable(r_m) requires
      OpenHere_Sigma(OpeningSubject(r_m))
      Writable_Sigma(Target(r_m)) and the selected operation's Place capability
      ordinary type/access/lifetime checks

Meta qualification alone implies neither Writable nor an operation's existence.
The body and write still use ordinary Pre/commit/Post and assignment constraints.

An explicit ordinary candidate confirms the mutable view:

    ConfirmMut : meta type ref -> mut type ref
    requires MetaOpen_Sigma(r_m), Writable_Sigma(Target(r_m))
             and ordinary capability/access/lifetime legality
    r_mu = ConfirmMut(r_m)
    Target(r_mu) = Target(r_m)
    BorrowedGeneration(r_mu) = BorrowedGeneration(r_m)
    Capability(r_mu) <= Capability(Target(r_m))

This confirms existing facts; it converts no authority and does not revive an
expired borrow. The operation names above specify candidate judgments, not new
syntax or an implicit conversion path.

At the same continuation position, if both explicit routes are legal:

    Target(DirectMut(n)) = Target(ConfirmMut(MetaRef(n)))
    realizable ordinary mut capability is the same

This coherence does not license resolver chaining Name -> meta -> mut. Each
explicit operation uses ordinary selection; selected failure never reopens.

For initialized-type mutable references the irreversible Close law is:

    Valid_Sigma(r : mut type ref) => OpenHere_Sigma(BorrowedType(r))
    Read(BindingPlace(n)) = T : type and Closed(T)
      => neither DirectMut(n) nor ConfirmMut(MetaRef(n)) succeeds
    not MetaOpen_Sigma(r_m) => not MetaWriteApplicable(r_m)

Previously obtained mutable refs recheck this condition on subsequent validity/
write Pre. Ordinary non-mut observations may survive under their lifetime rules;
InitialTypeSlotRef remains governed by its separate uninitialized-state law.
GeneratedOccurrence(T,s,v) implies none of these refs or opening facts: a frozen
generative rule may realize ordinary Val2 after Close without entering this
formation/borrow/write path.

These cases distinguish the judgments without introducing new syntax:

| Current facts and operation | Required outcome |
| --- | --- |
| path resolves an open T; parent Place is not Writable; selector is valid/unretained and access/path/type checks pass | `const let child::path:U` forms an uninitialized NameExpr; no parent ref is needed |
| Explicit ref of that child, then first write of v:U with live initial authority | Initializes even though the child is declared const; consumes initialization authority |
| Reuse that initial ref to replace the resident | No replacement authority follows from the initial ref |
| Initialized type name, OpenHere and Writable plus ordinary borrow checks | Both direct mut and explicit MetaRef then ConfirmMut yield the same target/generation and realizable mut capability |
| Same type is OpenHere but target lacks Writable | Neither mut route nor meta write becomes legal merely from openness |
| Save both kinds of ref, then Close their borrowed subject | Later mut validity/write and ConfirmMut fail; saving the ref does not save the proof |
| An authorized resident replacement changes generation | A saved ref is checked against its original generation; it does not switch OpeningSubject to the new resident |
| Equal type values in distinct resolved root bindings | Formation addresses distinct NameCoords, even if both current values are OpenHere |
| A frozen generator realizes a new ordinary member after Close | No explicit name-formation, ref acquisition, or write-capability inference occurs |

These are semantic conformance cases. Source consumers for the new contextual
meta ref family remain pending in the implementation; the table does not claim
that current Rust carriers execute them.

### 5.3 Borrow constructors have fixed points

Applying a borrow operator to something that is already a borrow view is
**well-formed**. There is a candidate for it, and that candidate is what makes
borrowing behave idempotently instead of building a second layer:

```text
Borrow_k( Borrow_j(q) )  =  Coerce_{j->k}( Borrow_j(q) )

Target( Coerce_{j->k}(v) )  =  Target(v)
```

The result is never a view of a view. The target is preserved and only the
capability index changes, so the borrow-type family collapses to one layer:

| composition | result | why |
| --- | --- | --- |
| `ref ref` | the same `ref` view | `Coerce` at equal capability is the identity |
| `share share` | the same `share` view | same |
| `ref share` | a `share` view of the same target | legal weakening |
| `share ref` | **no candidate** | illegal strengthening |
| `type ref ref` | `type ref` | borrow type-value fixed point |
| `type share share` | `type share` | borrow type-value fixed point |
| `type ref rebind rebind` | `type ref rebind` | retargeting type-value fixed point |
| `type share rebind rebind` | `type share rebind` | retargeting type-value fixed point |

Borrow-type universe fixed points prevent borrow classifiers from climbing the
type universe:

```text
rank(t ref)                    = rank(t)
rank(t share)                  = rank(t)
rank(t ref/share rebind)       = rank(t)
```

`@` is `ReifyLife(NameOf(actual), Pos(SemanticContinuation))`, yields a lifetime
value uniformly, and is never a borrow constructor
(`../lifetime/lifetime-policy-and-overload-boundary.md` §2.1).

Idempotence is the consequence of providing the equal-capability overload, not a
rule that contradicts it:

```text
Borrow_j( Borrow_j(q) ) = Borrow_j(q)          idempotence, from Coerce_{j->j} = id
ref  -> share  is a capability weakening       admitted
share -> ref   is a capability strengthening   no candidate
```

Capability weakening remains well-formed:

```lang
let r = t |> (type ref);        // r : type ref
let s = r share;            // ref share: s : type share, same target
```

`r share` is exactly the `ref share` composition. It is admitted, it does not
nest, and it does not retarget.

Only `share ref` is rejected, and it is rejected at selection time as "no
applicable overload" rather than being evaluated and then diagnosed: a `share`
view never carries the write/extension capability that `ref` would have to
produce. Capability can be surrendered, never regained.

No borrow-constructor overlap retargets the view:

```text
retargeting is available only through rebind (§5.4)
```

### 5.4 Writing through a reference versus retargeting a reference

A reference value is itself held in a place. The two operations are distinct and
both are ordinary assignments — they differ in **which** place is the target:

```lang
r_ref = value;              // writes value into the referent
r_ref rebind = expression;  // retargets r_ref itself at a new referent
```

```text
r_ref = v          ->  Write( Referent(r_ref), v )
r_ref rebind = E   ->  Target( Value( HolderPlace(r_ref) ) ) := NewTarget(E)
```

`rebind` is a **retargeting** operation, not a value borrow. It does not evaluate
`E ref`, because for a pure `type` slot `t` the expression `t ref` is the
type-forming overload and yields the TypeValue `uint8 ref`, not a borrow
instance over the slot `t` (§5.2). So the new target is taken from a
place-bearing right side:

```text
NewTarget(E) = Target(E)          when E is already a borrow view
NewTarget(E) = CarrierPlace(E)    when E supplies a carrier place
NewTarget(E) is undefined         otherwise
```

An `E` that supplies neither — a freshly computed temporary — gives `rebind` no
applicable candidate. The obligations a `rebind` must discharge are:

```text
E supplies an origin/place
Pattern( NewTarget(E) ) conforms to the Pattern the view is declared over
Capability( result ) ≤ Capability( E )        no strengthening
lifetime / escape check on the new target
```

The last obligation is the escape check of
[`../lifetime/lifetime-policy-and-overload-boundary.md`](../lifetime/lifetime-policy-and-overload-boundary.md)
§3.

Without `rebind`, an assignment whose left side is a reference always reaches
through to the referent. `rebind` is what selects the borrow-holder place as the
assignment target. There is no context in which the same spelling means both.

### 5.5 `type`, `type ref`, and `type share`

A by-value `type` closure carries no carrier-slot place or borrow capability;
consuming one can only produce a new closure value. Ordinary `Read` of a
type-valued place yields the complete closure `tau` (§2.2) — it never projects
`Core(tau)` on its own, and no rule silently degrades the read to `Q`;
consumers select `Core(tau)`, `CallSpace(tau)`, or the whole snapshot
explicitly. `ref` over a type-valued operand forms the borrow type value
`tau_(t ref)` (§5.2), not a borrow of `Core(tau)`. A bound type
expression has a carrier slot that `t |> (type ref)` reaches explicitly as
`type ref`; `@` yields a lifetime value and never forms a `type ref`
(`../lifetime/lifetime-policy-and-overload-boundary.md` §2.1).
Construction openness is not a capability carried by the closure or by a view;
it is the separate `OpenHere_Σ(value)` judgment over open authority (§6 and
the symbol-first construction document).

`type ref` is the borrow-reference type produced by `type |> ref`; a value
`r : type ref` is a borrow view of a slot whose contents conform to `type`.
Such a view is formed whenever ordinary place, policy, lifetime, and
capability rules admit it:

```lang
let t: type = uint8
let r : type ref = t |> (type ref)
```

A `type ref` is a type value; the borrow instance it carries holds only the
ordinary borrow coordinates:

```text
⟨ TargetPlace, type, BorrowCapability, LifetimeRelation ⟩
```

A closed-window type-valued slot may still admit non-mut observations under
ordinary borrow/lifetime rules. An initialized-type mut reference is valid only
while its borrowed type generation is OpenHere (§5.2.2); holding it does not
authorize replacement after Close. Writable(target) alone cannot repair that
failure. Meta-qualified writable candidates and ConfirmMut recheck the same
original opening subject; neither follows whatever resident later occupies q.

Identity retention/non-mut observation follows the ordinary borrow-valid region;
mutable type-reference use additionally requires the live opening condition.
Weakening remains useful when write authority is unnecessary:

```lang
r share    // type share: still observable, no write authority
```

Reachability alone still forms no view, and neither reachability nor a view
decides construction state:

```text
GlobalLifetime(q) does not imply OpenHere_Σ(Value(q))
Γ ⊢ r : type ref does not imply OpenHere_Σ(Read(r))
```

`OpenHere_Σ` is defined from `Anchor`/`WindowLive_Σ` and the
authority-frame resolution of §12.1.1 in
`symbol-first-meta-construction-and-pattern-injection.md` §12.1.1.

`type share` is the deliberately weaker view. It may be stored or passed across
any region admitted by the ordinary lifetime relation, but is not assignable and
is not an `inject` target:

```text
type share is not a valid assignment left side
type share is not a valid inject target
```

The last two lines are domain facts. A `type share` in an assignment-target or
`inject`-target position produces "no applicable overload", never a permission
error discovered after the operation has begun.

#### 5.5.1 Three independent judgments

The following obligations never collapse into one check:

```text
extend on a type value      ->  OpenHere_Σ(value)
inject through a type ref   ->  valid selected ref capability, OpenHere_Σ(Read(ref))
                               and Writable(Target(ref))
returning / storing a ref   ->  ordinary lifetime/capability escape check
```

Returning a `type ref` from a `compile` callable is therefore governed by the
same borrow escape rule as any other reference. Its identity and permitted
non-mut observation may survive closure within that lifetime. Mutable type-ref
validity additionally requires OpenHere of the original borrowed generation;
neither a saved mut ref nor a meta ref supplies a write after Close. A later
resident at the same Place does not retarget that reference (§5.2.2).

### 5.6 Type-expected positions elaborate `|> type`; candidate discovery does not

§5.1.1 excludes implicit projection in *operand* positions. That exclusion is
about candidate discovery / formal applicability, and it must not be read as
"the language performs no type-context projection anywhere". The two rules are
distinct:

```text
candidate discovery / formal applicability
    =/=>  implicit AsType

unique language-designated result/type transformation
    ==>  AsType(E) = E |> type
```

A function formal `t: type` must not, during candidate enumeration, try
`actual |> type` to make an inapplicable candidate suddenly match. An operand
position never acquires a projection because a projection would make the program
check. Implicit `AsType` is admitted only where the language has already
committed to a unique transformation target — declaration annotations,
path components that demand `TypeRole`, and type-rank return positions — never
where multiple candidates compete for applicability.

In a language-designated type-expected position the elaboration is supplied:

```text
AsType(E)  =  E |> type
```

`AsType` validates an already-complete TypeValue read from a binding, or uses
an explicitly defined value-side type projection as
specified in §2.2. It does not compute the type of
the expression, wrap a namespace-like Object, or raise universe rank:

```text
AsType(E) != TypeOf(E)
rank(AsType(E)) = rank(the selected or validated complete type value)
```

Only explicit type-of extraction — for example the future canonical form
`let <typeof> x : typeof = RHS` — may produce the classifier one universe above
`RHS`. The global `type` object is itself a value of `type_1`:

```text
TypeOf(type)   = type_1
TypeOf(OverloadGroup) = type
```

With `tau = <Q, V_τ>`, `type` carries its own callspace, so
`TypeOf(type) = type_1` directly; no name binding recovery participates.

The designated positions are:

| position | example |
| --- | --- |
| declaration annotation | `let x: E` |
| a path component that demands `TypeRole` | the type projection step of §3.2 |
| type argument position | a parameter declared to receive a type |
| `t: type` | a parameter or binder at type rank |
| `t: type ref` | the borrow-view form of the same |
| type-rank return position | a callable whose return is declared at type rank |

So `E` supplying a `Val1` dimension in one of these positions is projected to
its complete type snapshot when its `Q` satisfies `TypeRole`, without the
author writing `|> type`, while the very same `E` under
`ref` is not:

For an explicitly bound ordinary OverloadGroup g whose type projection is
valid in that designated context (g denotes the group resident, not a binding
wrapper):

```lang
let x: g = ...;             // type-expected: elaborates to g |> type
let r = g ref;              // actual group operand: r : OverloadGroup ref
```

For a structural named type t instead, t ref forms the borrow type and
`t |> (type ref)` borrows its resident Place; it never yields a group reference.

The distinction is positional and fixed by the language, never inferred from the
operand's shape. An operand position never acquires a projection because a
projection would make the program check.

@ preserves name interpretation: N@ is a name iff N is a name. It does not
perform an implicit AsType conversion or itself create a borrow. Lifecycle
value fields and borrowed fields follow ordinary projection rules. Safe code
can observe the values; a mutable reference that writes back into lifecycle
knowledge additionally requires unsafe and all ordinary permissions.
The [admission owner](../lifetime/unsafe-semantic-admission.md) defines the
post-commit, compatibility-checked information flow.

## 6. Writability, member creation, and construction openness

The checker distinguishes the following judgments:

```text
Writable_Γ(q)
CanCreateMember_Σ(resolved_root, selector)
OpenHere_Σ(v)
```

`Writable` is a Place/borrow-capability question. `CanCreateMember` is value-side:
read the initialized type at the resolved structural root, check its OpenHere,
selector validity, non-retention and ordinary access/path/type well-formedness.
It neither requires parent Writable nor acquires a parent mut type ref.
The coordinate root remains structural identity, not the type's normalized value.
OpenHere is also required by pure structural `extend`; by itself it grants no
Place capability:

```text
Writable_Γ(q)           does not imply OpenHere_Σ(Read(q))
OpenHere_Σ(v)           does not imply Writable_Γ(Carrier(v))
CanCreateMember_Σ(r, n) does not follow from Writable_Γ(Carrier(r)) alone
```

`PolicyMode` is equally orthogonal to object shape and operation capability:

```text
PolicyModeOrthogonalToObjectShape:
  Val1(x) = absent does not determine PolicyMode(slot(x))
  PolicyMode(slot(x)) does not alter Norm(x)

default ref/write-family realization:
  Write(const ref) = delete
  Write(plain ref) = delete
  Write(mut ref)   = default
```

The table is a theorem of the builtin ref/write family, not a Policy axiom.
Here const/plain/mut describe the selected reference view, not the declaration
policy of a possibly uninitialized target name. The ordinary initial-borrow
realization in §7.1 can supply a mut reference view with initialization-only
capability even for a const-declared name. It does not turn a selected delete
into default or grant replacement through that reference.
Another family may mark any 3×3 coordinate absent or realize it with
`default`, `delete`, or `custom`. In particular `mut` selected for a non-ref
object does not automatically make any place writable.

A Place's Writable fact alone does not permit replacing a closed type through
a type ref: the selected mutable type-reference capability also requires
OpenHere of its borrowed generation (§5.2.2). Conversely an open-window value may be
extended purely and bound elsewhere even when its source is immutable or has no
write-back place.

At minimum, ordinary place operations reject a core/external stable place, a
place reached only through `share`, a place outside its borrow lifetime, or a
place whose policy denies the action. Name formation separately rejects a
non-OpenHere current parent type or an already-retained selector.
Structural `extend` independently rejects a value whose window is
closed (`WindowLive_Σ = false`) or whose `Anchor` lacks authority under the
authority-frame resolution
of §12.1.1.

Value equality grants no write permission. Even when:

```text
value(T) == value(uint8)
```

it does not follow that:

```text
place(T) == place(uint8)
```

and it certainly does not follow that:

```text
place(uint8) is writable
```

This is the concrete reason member creation under the globally stable `uint8`
slot is rejected while creation under a locally constructed type place may be
accepted:

```lang
let T = (() t) |> struct;
mut let f_ref = (let f::T:type) ref;
f_ref = ...;
```

No binding or borrow view can amplify the place authority it observes:

```text
Capability(view of p) ≤ Capability(p)
```

## 7. Projection slots and structural name expressions

A prospective structural target consists of a parent Place/resident and a
selector. Its coordinate may be represented before a child name exists:

    ProjectionCoordinate(parent_place, selector)
    ProjectionSlotIdentity = <ParentResidentIdentity, selector>
    Fresh_Sigma(parent, name) iff not HasName_Sigma(parent, name)

Occupancy is a structural judgment. Implementations may store an optional
payload, but that representation does not define an optional-name value.
Hidden or policy-filtered names remain occupied.

An existing borrow preserves its formation-time target generation. Replacing
the parent resident invalidates the old projection under the ordinary lifetime
rules; it never redirects an old borrow to the replacement's same-spelled slot.
Only ordinary rebind can acquire a new target. The concrete generation encoding
remains open.

Value/path-based name formation uses the resolved structural root identity,
current resident T:type, OpenHere(T), selector/freshness and ordinary access/type/
path checks. Parent Writable and a parent mut type ref are not premises. A
borrowed navigation route preserves its actual target but is not required to
form the typed child NameExpr. Named and ordinal selectors retain their
own topology; T*N and T*omega indexing cannot grow a Sequence through let.

### 7.1 Typed NameExpr, explicit borrowing and first write

    P let name:t -> NameExpr(n) at an ordinary fresh lexical destination
    P let name::path:t -> NameExpr(n)
    PlaceType(BindingPlace(n)) = t
    ResidentState(BindingPlace(n)) = Uninitialized
    P let name::path == P let name::path:type

These initializer-free declarations share the typed Place/NameExpr rules;
lexical and structural destinations retain their own creation authority.
P let name = rhs is instead one complete lexical binding, with ordinary RHS
type inference; P let name:t = rhs is its explicitly constrained form. Neither
first defaults to :type nor decomposes into NameExpr followed by source =.
See [let forms](names-and-overload-groups.md#5-typed-name-declarations-and-complete-let-bindings).

Creation does not install a resident or return a ref. In value context NameExpr
reads only an initialized resident. Explicit ref borrows the existing typed
Place without first reading. Its initial borrow uses the initialization
authority below; ordinary initialized-name views use their declared policy.

    CreateName -> explicit Borrow -> ordinary Write
    Uninitialized -- Write(v:t) --> Initialized(v)
    Initialized(old) -- Write(v:t) --> Initialized(v)

The first write has no old resident to clean up. Replacement retains its ordinary
same-Type and lifecycle checks. Failed Pre leaves the target state unchanged.
Uninitialized is non-Object Place state, never a None value or fresh-name value.

### 7.1.1 Initialization authority and the two Write cases

    DeclaredPolicy(name) independent_of InitialInitializationAuthority(place)

Successful authorized fresh typed-Place formation establishes a pending
initialization authority for that actual Place in its creation context,
regardless of const/plain/mut declaration policy. This is ordinary Place state
and authority, not a token Object or an additional PolicyMode. It is neither
inferred from Uninitialized alone nor transferred by equal type/value identity.
The original authority source, actual Place access, applicable construction constraints
and borrow lifetime must remain valid at use; saving a reference extends none
of them. The concrete capability carrier remains an implementation choice.

For such a Place the ordinary explicit ref operation has an initial-borrow
realization: from the live initialization authority and ordinary access/lifetime
premises it obtains a T ref view capable of the first write. A mut result view
can be requested independently of the future resident's declared name policy.
Its capability is limited to initialization of that Place, with no right to
read an absent resident or replace a future one. Formation itself returns only
NameExpr; the explicit borrow remains necessary.

    InitialInitializationAuthority_Γ(q)
    ResidentState(q) = Uninitialized, PlaceType(q) = t
    selected ordinary initial-borrow realization, access and lifetime legal
    ---------------------------------------------------------------------
    explicit Borrow(q) -> r : t ref
      with capability for Initialize(q), not Replace(q)

The same ordinary Write family distinguishes the current Place state at Pre:

    InitWriteLegal_Γ(r,q) iff
      InitialInitializationAuthority_Γ(q) is still live and unconsumed
      and r targets q with a valid capability for Initialize(q)
      and the original authority/access/applicable construction/lifetime checks hold
      -- these premises establish Writable_Γ(q) for this initialization

    ResidentState(q) = Uninitialized, PlaceType(q) = t, v:t
    InitWriteLegal_Γ(r,q), ordinary RHS/result/boundary checks
    -------------------------------------------------------
    Write(r,v): Uninitialized -> Initialized(v)
      consume the pending initialization authority at successful commit

    ResidentState(q) = Initialized(old), PlaceType(q) = t, v:t
    valid replacement capability through r, Writable_Γ(q) for replacement
    ReplacementCompatible(old,v), ordinary access/lifetime/RHS/result checks
    ---------------------------------------------------------------------
    Write(r,v): Initialized(old) -> Initialized(v)

Writable is checked for the selected operation and supplied capability; a
proof for Initialize(q) is not a proof for Replace(q). Only the second case
observes old and checks resident compatibility or old-resident cleanup.
ReplacementCompatible(old,v) abbreviates the existing resident compatibility
check Compatible(P(old),v), together with the ordinary same-Type constraints;
it is not a new compatibility relation or a policy inferred for an empty Place.
The first case never evaluates P(old) or Compatible(P(uninitialized),v).
DeclaredPolicy determines the initialized name's views, not compatibility with
a nonexistent resident; the supplied v must still satisfy the declared type
and all independently specified destination/operation constraints.

Successful initialization consumes the authority for all aliases of the same
Place. A second write through a saved initial reference must establish ordinary
replacement capability anew; the reference's mut spelling is insufficient.
Failed Pre consumes nothing and installs nothing. Ordinary enclosing transaction
rules alone determine rollback. Move/drop, copying a handle or a cache hit do
not recreate this first-initialization authority. No failed selected operation
reopens overload selection or retries the other state case.

For an authorized parent path and a legal value v:T:

```lang
mut let r = (const let x::path:T) ref;
r = v;
```

The first line creates x and explicitly borrows its Place using the pending
initialization authority, not a const resident. The second initializes it.
Subsequent x views follow its const declaration; a saved r has no replacement
right from its consumed initial capability. The same construction works for
plain or mut declarations; neither mode supplies authority by itself. Without
live initialization authority, the first write fails even if the name is mut.

### 7.1.2 Closure and contribution handoff

The structural let-with-assignment compound is not canonical; any future sugar
must expand into these steps. Ordinary lexical let remains unchanged.

Close requires retained structural names being published to be initialized;
open HasName facts may precede dom(Val2). Legal coordinates not yet retained are
not uninitialized members and need not all be realized before Close. Later
generated ordinary Val2 results do not reopen registered structure. See [name semantics](names-and-overload-groups.md)
for formation, closure and the joined named-contribution trace.
An established sibling contribution bucket joins all role-specific materials
against a common snapshot, forms one complete type, and initializes once.
No first closure RHS is its initial resident. Ordinary singleton let installs
tau_C:type. Extension of an already initialized snapshot uses extend/inject.

Creation/initialization registers neither callability nor Pattern roles.
Inject still reads an existing resident, extends it and writes the result;
it cannot initialize an unreadable target. Physical files grant no authority.
A's instance/member references preserve their original targets and dependencies.

### 7.1.3 Independent callable construction axes

P let s::path:t accepts a terminal selector s in Selector, including the special
leaf (). Its Place formation, borrow, initialization and replacement use the
ordinary rules; () is neither an operator nor a navigation parent.
Initializing AssociatedNamespace(T).Val2[()]=k supplies ordinary x:T's call
entrance. This is ordinary Val2 semantics, with no third callability registry.

    OrdinaryCallableValue(v) implies Val1?(v) != absent
    CallCandidates_ordinary(v)
      = Entries(AssociatedNamespace(Type(v)).Val2[()], actual_self=v)
    TypeMember_T(v) iff OrdinaryCallableValue(v)
      and Home(Type(v)) = TypeMemberScope(T)
      and RegisteredCallability_T(v)

RegisteredCallability_T is the existing non-generative registration in V_T;
the anonymous classifier and complete-snapshot consistency checks still apply.
TypeAdd(T,v) forms v'=AnchorFor(v,T) and changes only V_T to V_T + v':

    TypeAdd(T,v) does not imply AssociatedNamespace(T).Val2[()] = v
    AssociatedNamespace(T).Val2[()] = k does not imply k in V_T
    CallCandidates_type(T)
      = disjoint_union over v in V_T of CallCandidates_ordinary(v)

This union undergoes one selection. Named residency of v in T is not required;
the associated () on Type(v) is a different coordinate. A closure result tau_C
has absent Val1 and cannot itself be a TypeMember. Ordinary binding installs
tau_C:type; established same-name contribution consumes ClosureMaterial(C)
to form one eligible c_C^T at the known target. It imports neither tau_C nor
the whole V_tau_C. See the [formation consumers](symbol-first-meta-construction-and-pattern-injection.md#211-v_τ-closure-materialization-derived-semantics).

## 8. Type values in overload and pattern matching

Ordinary type matching for overload and pattern compatibility compares
canonical type values, not source binding names:

```text
OrdinaryTypeObservation(τ) = Core(τ) = Q

τ₁ ≈type τ₂  iff  Norm(Core(τ₁)) = Norm(Core(τ₂))
```

`TypeValueId` is only the implementation/index root projection of `Core(τ)`
(§2); it is not semantic equality and does not participate in overload or
pattern compatibility. (The candidate-preparation layer that consumes type
values is specified in `pattern-normalization-and-first-order-overload.md`;
this document defines what a type-value identity is.) Under the minimal-change
rule (§2.2), `Core(τ) = Q` observation is the default for ordinary type
matching. Whole-snapshot identity
`Addr(Norm_type(tau))` is used only where the language has independently frozen
whole-snapshot semantics.

For example:

```text
let T: type = uint8
```

In ordinary type matching, `T` and `uint8` observe the same core:
`Norm(Core(τ_T)) = Norm(Core(τ_uint8))`. But this says nothing about their
places:

```text
T and uint8 may observe the same Core(τ) yet have different PlaceId.
```

The same separation applies to Pattern layers. Every all-named direct-entry
layer is unordered, independent of a top Pattern name; any bare entry makes the
whole layer positional. BareProduct's pos_i equations above describe that
positional subdomain, not every Product. See the [Pattern owner](../patterns-overload/pattern-values-relational-semantics-and-extraction.md#7-product-ordering-is-local-to-each-layer).
NameBindingId and PlaceId are neither resident values nor ordinary content keys.
Open navigation retains each observed member's name Pattern; carrier spelling
does not rename that member or merge coordinates.

Pass mode is **not** part of ordinary type-value observation. A construct such
as `T move` does not
change the type value, and type-value comparison is invariant under
`move` / `copy`. Borrow views are different: `T`, `T ref`, and `T share` are
three distinct values with distinct patterns, because a borrow view is a value
produced by an operation (§5), not a passing annotation. The detailed treatment
of `T move == T` as a move fixed point belongs to the mechanical
argument-passing / move design and is only referenced here, not expanded.

## 9. Borrow views and policy

A borrow view neither manufactures nor amplifies capability:

```text
Capability( view )  ⪯  Capability( source )      -- never above the source
```

It may expose a restricted capability that its own formation overload explicitly
grants — that is exactly how `ref` differs from `share` (§5.5) — but it can only
surrender, never regain (§5.3). It must operate within the existing policy,
visibility, and place-eligibility restrictions.

```text
A borrow view may expose an observation of its source value.
A borrow view must not manufacture permission.
A borrow view must not make an ineligible place eligible.
A borrow view must not bypass policy filtering.
```

If the observed object is not visible or not usable under the current
`PolicyEnv`, taking a `ref` or `share` of it does not make it visible or usable.
Re-export or wrapper semantics that intentionally re-expose a target under a
different policy is a separate, later design and is **not** defined here.

## 10. Non-goals

```text
No parser syntax change.
No full type checker.
No full lifetime/access-tree checker.
No runtime lookup implementation.
No package re-export semantics.
No permission escalation through borrow views.
```

## 11. Relationship to other documents

The documents below are adjacent or background design. They do not define the
distinctions specified here, and this document does not depend on them for its
meaning.

- `symbol-first-meta-construction-and-pattern-injection.md` — canonical
  symbol-first facet resolution, `PatternValue`, `compile` / `meta`, pattern
  scopes, `struct`, pure `extend`, place-level `inject`, open-authority
  `OpenHere_Σ`, and the
  binding/install boundary. It uses this document's `NameBindingId` / `PlaceId` /
  `TypeValueId` and place judgments.
- `../lifetime/lifetime-policy-and-overload-boundary.md` — canonical owner of
  `@` (continuation-relative name reification yielding a lifetime value) and of
  `ref` / `share` borrow formation, plus escape checking. This document
  supplies only the `Origin`/`Value` split that `@` consumes.
- `type-associated-function-objects-and-access-trees.md` — field functions,
  same-name receiver overloads and access-tree work. It references
  this document for the canonical value / place / borrow-view distinction rather
  than restating it.
- `early-meta-functions-and-namespace-graph.md` — the build / namespace graph
  and bootstrap consumers of complete type and Place observations.
- `symbol-construction-units-and-namespace-origin.md` — canonical
  `NamespaceOrigin`, construction-unit ownership, physical contribution
  authority, pure/type role refinement, and cross-file closure rules.
- `pattern-normalization-and-first-order-overload.md` — the pattern/type
  candidate-preparation layer that consumes ordinary type-value observation
  (`Core(τ) = Q`) for first-order type matching.

## Instance lifetime and type transport

Type value equality, stable root identity, local binding, Place and lifecycle
instance remain separate. Established non-meta type instances retain the existing global
survival rule; its extension to dependency-bearing closure-generated tau is
explicitly handed to lifetime refinement, not inferred from the new result category; meta-local type temporaries may have finite generations and a
killing move. A stable meta root, an equal local copy and an equal globally
retained resident do not thereby share lifetime or Killable facts.

The [lifetime owner](../lifetime/lifetime-policy-and-overload-boundary.md#2131-instance-killability-and-move-legality)
defines Killable_K, predetermined MoveEffect and frontier Movable uniformly
for type, meta, compile and runtime instances. OpenHere, Writable, Place
residency and lifetime imply none of one another. A narrow Preserve proof is
not an implicit clone and does not exempt the action from borrow/access Pre.


## Direct type projection and closure formation boundary

Policy observation after the same source-edge type projection exposes Pp;
ordinary direct Policy observation exposes Pv. A fresh binding of that projected
type has its own destination view and does not recover the original edge by
Core equality. This preserves all same-entity and whole-snapshot distinctions.

Every legal completed closure expression produces tau_C through ordinary struct
formation. Its c_C, classifier A_C and terminal () leaf are distinct roles.
First callable formation needs no arbitrary instance construction or deleted
constructor; TypeRole follows purity, independently of SelfConstructible. General
dependency state uses ordinary owned structure/reference identity. FormationLegal,
LifetimeLegal, Pre/Post, MoveEffect/Movable, EscapeLegal and transfer/promotion
checks remain required; tau, Core equality and layout establish none of them.
