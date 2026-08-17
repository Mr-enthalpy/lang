# Type Values, Places, and Borrow Views

**Status: canonical target semantics for Object identity, complete type-closure
identity, place identity, and borrow views. Current `lang_build` implements only
the first-order identity core: recursive normalization over the present
type-core/`Val2` substrate with an opaque `Val1` leaf, `TypeValueId`, and
per-carrier places. The complete `tau=<Q,V_τ>` snapshot and
`Norm_type(tau)`, full recursive `Norm_Val1?`, the borrow-view operators (`ref`,
`share`, `rebind`), the place-sensitive lifetime observation (`@`),
construction-lineage Open judgment, and type checker
remain unimplemented target semantics. §10 registers the implementation debt.**

This document specifies the semantic boundary between *object values*, *symbol
identity*, *places*, *borrow views*, and *namespace extension targets*. It
defines what an object is, what a place is, which borrow operators exist, which
overloads they have, and when each overload is callable. It is a semantic
authority, not a parser or normalizer rule.

The document is self-contained. It does not require the reader to assemble its
meaning from `type-associated-function-objects-and-access-trees.md` or
`early-meta-functions-and-namespace-graph.md`. Those documents are background
or adjacent design only; the model here stands on its own and is the canonical
authority for the value / place / symbol / borrow-view distinction.

There is no ordinary symbol-alias or place-forwarding declaration form in this
language. `let a = b;` copies a value into a fresh symbol with a fresh place.
Sharing an observation of another object is expressed by the borrow operators
defined in §5, never by a declaration that makes two symbols name one place.

The broader symbol-first facet, `PatternValue`, `compile` / `meta`, pattern
scope, `struct`, pure `extend`, and place-level `inject` model is canonicalized in
`spec/design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md`.
That document composes with this identity/place model rather than replacing it.

## 1. Purpose

The language must distinguish three things that look similar in source text but
are semantically different: the identity of a name, the identity of a writable
location, and the identity of a type value. Conflating them produces subtle
errors — for example, injecting a declaration into a built-in type because its
type value happens to equal a freshly bound symbol's type value.

The invariants this document protects:

```text
value equality must not collapse symbol-place identity
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
substitution and **not** a second name for a symbol. And value equality is
**not** place equality.

## 2. Semantic identities

Three distinct symbol/type identities participate in this model, alongside
canonical pattern-value identity:

```text
SymbolId
PlaceId
TypeValueId
PatternValue identity
```

- `SymbolId` is the identity of a symbol object in the name graph.
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
                    keying, and type-argument identity, exactly as under the
                    former `type = Q` rules (minimal-change rule, §2.2)

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
SymbolId equality does not imply TypeValueId equality.
TypeValueId equality does not imply PlaceId equality.
PatternValue equality does not imply SymbolId or PlaceId equality.
A borrow view names one place from one origin; it relates values and places without erasing the distinction.
```

A type expression cares about the *value*. A namespace extension target or a
declaration-extension site cares about the *place*. A borrow view is itself a
value that carries a place coordinate. The three concerns must not be folded
into one another.

In the symbol-first model, a path initially resolves to one Symbol and the use
site then projects the complete immutable type closure `tau` (if any) and the
heterogeneous typed value members `V_S`. Type projection returns the `tau` that
was formed at installation; namespace projection returns `Core(tau) = Q` when
`tau` is present. Projection does not collapse these identities and is not a
cast.

### 2.1 Object identity is the recursive three-component normal form

Every object in the language has the same shape:

```text
Object x  = ⟨ Val1?(x), P(x), Val2(x) ⟩
Val1?(x) ∈ 1 + Object
```

`Val2` is a finite map from semantic selectors to ordinary Objects. Most
selectors are names and their entries are same-name Symbols; the built-in bare
Product Pattern additionally supplies intrinsic ordinal selectors `pos_i`.
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
Pure(x)          <=> Val1?(x) = null
Navigable(V)     <=> V is a well-formed finite semantic-selector map
                     on which ProjectionSlot lookup is defined

WellFormedObject(x) => Navigable(Val2(x))
NamespaceRole(x)   <=> Pure(x)
TypeRole(x)        =>  NamespaceRole(x)

TypeRole subset NamespaceRole
NamespaceRole not-subset TypeRole
```

`TypeRole(x)` is an imported relational judgment over the Pattern/Object
relation. Its derivation belongs to the subsequent P–Val1–Val2
relational-semantics design, which will define how `P` simultaneously
constrains, observes, and extracts both `Val1` and `Val2`. This layer only
consumes `TypeRole` as an opaque predicate.

`Navigable(V)` means selector lookup yields the resident-specific
`ProjectionSlot` defined in §7; a missing final selector still yields a slot
whose contents are `None`, while continuation from `None` is invalid. Being a
well-formed Object already supplies such a `Val2`, including the empty map.
Consequently every pure Object has `NamespaceRole`; namespace capability is not
an extra entity or witness. A pure Object for which `TypeRole` does not hold remains
navigable but is not usable as a type. These are
predicates; no `NamespaceFacet`, hidden Q/type-role member, or parallel
`NamespaceObject` ontology is introduced. Bare Product and the empty pure
Pattern therefore have intrinsic structural namespace capability through their
ordinary selectors; whether a tuple-like user-facing namespace API exposes
that capability remains a narrow surface question.

An Object carrying `Val1` may still be used where a type is expected when an
ordinary projection such as Symbol's `Q` projection under `TypeRole(Q)` applies
(§5.6). Conversely, `Pure(x)` alone does not imply `TypeRole(x)`. Keeping these
judgments separate prevents payload presence from becoming an implicit kind
classifier.

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
DecodeSymbolPayload(Σ_Object) = ⟨ τ?, V_S ⟩
Norm_Val1?^P_symbol(Σ_Object) = ⟨ Norm_type(τ)? ,
                                Map_{Norm(T_c)}(
                                  Set{ Norm(v) | v in V_S[T_c] }) ⟩
Norm_Val2(V)               = Map_selector( Norm(V[selector]) )
                               -- named entries are ordinary Symbols;
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

The `P_symbol` clause first decodes the two ordinal positions of the ordinary
`Σ_Object` composition specified in `symbol-first-meta-construction-and-pattern-injection.md`,
then applies a Pattern-specific quotient to the logical view
`Σ = ⟨tau?, V_S⟩`, where `tau`, when present, is the complete type value
carried by the Symbol,
`V_S = ⨄_{T_c} V_S[T_c]`, and every homogeneous bucket
`V_S[T_c] : T_c * omega`. The member objects inside a bucket retain their own
ordinary recursive identity, including stable declaration/candidate identity,
callable body, and every annotation that affects semantic selection. There is no
universal `SemanticMember` wrapper type and no exception to the homogeneity of
`T * omega`.

Multiplicity is not retained by the final Symbol normal form. Replaying the
same contribution of the same stable member is idempotent in value identity;
conflicting declarations and duplicate definitions are rejected by construction
or well-formedness before the quotient is observed. Two distinct members are
not collapsed merely because their bodies normalize alike. In particular,
inserting `a` then `b` and inserting `b` then `a` produce the same Symbol normal
form exactly when the stored `tau` (if any) and every normalized `V_S[T_c]` set are
equal.

There is no case split in which one Object component is ignored. Earlier revisions
normalized `Val1? = null` objects as `⟨P, Val2⟩` and `Val1? ≠ null` objects as
`⟨Val1, P⟩` with `Val2` discarded; that bifurcation is retired. A value-bearing
Object whose `Val2` differs is a different Object, and a pure core `Q` whose
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
SymbolId                              ∉ Norm(ordinary object)
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
cache identity all depend on that distinction. Construction-lineage Open does
not depend on the target coordinate.

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
allocation order is not semantic content, so the walk must resolve each name to
its cluster symbol and normalize that symbol's own members.

This exclusion does not erase `StableTargetIdentity(place)` from a borrow-view
normal form. In the ordinary object case a place is an observation source; in
the borrow-view case the target is what the value denotes.

This is what makes an open construction observable at all. Given

```lang
let fn = (...): meta -> _ :symbol = {
    let t = (() t) |> struct;

    let f::((t ref).type) = X;
    let A = t |> compile_fn;

    let g::((t ref).type) = Y;
    let B = t |> compile_fn;
    t;
};
```

the two observations of `t` are different complete type snapshots:

```text
tau_1 = <Q_1, V_τ>
tau_2 = <Q_2, V_τ>

Val2(Q_1) contains f
Val2(Q_2) contains f and g
```

so `Norm_type(tau_1) ≠ Norm_type(tau_2)`. The `compile_fn` calls may consume both
meta-local observations because compile creates no `MetaInstanceKey`; an
ordinary nested meta call on fresh `t` would instead fail `GlobalKeyable`.
Reading only a shared first-order root instead of each carrier's complete
snapshot would still incorrectly merge the two values.

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
normalizes recursively like every other component, and the opaque leaf is a
placeholder for content normalization that is not yet implemented. It does not
override a defined Pattern-specific quotient such as `P_symbol` above.

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

Re-entering any object still on the active recursion stack through any positive
owned path proves a violation at the stage where the cycle is materialized:

```text
x ∈ ActiveStack
∧ x ∈ Children_owned+(x)
--------------------------------
NoNormalForm_kappa(x)
```

Thus `Val1(x) = x`, `Val1(x) = y ∧ Val1(y) = x`, a cyclic product, and a cyclic
`Val2` such as `let loop::t = t;` all have **no normal form** at the stage where
they are materialized. A finished shared acyclic subtree remains valid DAG
reuse. `Self_τ` is one restricted static back-reference instance, not the one
exceptional cycle, and not a general recursive-data constructor.

### 2.2 Complete type values are closed snapshots over Object cores

The ordinary pure Object `Q` keeps the old type behavior:

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
  iff exists a formation witness w:
      FormationLine(w, Q_0, V_τ)
      and CoreOnFormationLine(w, Q)
      -- closure/formation/backref invariants; the judgment that the
         type-value package tau exists

TypeValueRole(tau)
  iff WellFormedTau(tau)
  and TypeRole(Q)
      -- the type-value role; equivalently CompleteType(tau)

NamespaceOnly(tau)
  iff WellFormedTau(tau)
  and NamespaceRole(Q)
  and not TypeRole(Q)

FormationLine(w, Q_0, V_τ)
  // w is the formation event that created Q_0 and fixed V_τ
  iff w is a formation event for Q_0
  and V_τ is the TypeMember set placed into tau at w
      (each F in V_τ satisfies TypeMember_{Q_0}(F) at formation;
       members created later under the same Q_0 never enter V_τ)

CoreOnFormationLine(w, Q)
  // Q is Q_0 itself, or a core derived from Q_0 by zero or more
  // permitted Associate* operations on the same formation line
  iff Q = Q_0
  or exists Q_prev: CoreOnFormationLine(w, Q_prev)
      and Q = AssociateCore(Q_prev, selector, value)
```

Whether `tau` has the type-value role or is namespace-only is decided by
`Q`'s Pattern relations, never by the sibling count of a Symbol space. A
namespace-only `Q` has the form:

```text
Q = <absent, P, Val2>

no Val2 child is used by P as a sub-pattern describing a Val1
  => NamespaceOnly(tau)
     -- P at most describes its own Pattern structure; no Val2
        sub-pattern participates in describing a Val1
```

The namespace-only judgment is a relational property of `Q`'s Pattern `P`
(imported from the Pattern relational semantics); it is **never**
`count(pure members in V_S)`.

A derived snapshot `tau' = Associate(tau, f, v)` is a well-formed type value
on the same formation line: `WellFormedTau(tau')` holds because
`CoreOnFormationLine(w, Q')` holds — `Q'` is obtained from `Q_0` by permitted
`Associate*` — while `FormationLine(w, Q_0, V_τ)` and `V_τ` are unchanged.
`Associate` is not a new formation event, and the ordinary
associated-installation `Associate` preserves `TypeRole(Q)`, so
`TypeValueRole(tau')` holds. Only structural transformations (`extend`) form a
new `V_τ` and establish a new `FormationLine` with a fresh witness `w'`.

`V_τ = CallSpace(tau)` is the callspace captured when the type value was
formed: the direct TypeMember members placed into `tau` at that event
(`TypeMember_Q`, symbol-first §2.1), not a later partition of a shared Symbol
space and not a global function of the bare core `Q`. The witness `w` pins the
snapshot to one formation event, so `WellFormedTau` / `TypeValueRole` are not
global functions of the bare core `Q`. Members created under the same `Q` after formation never
retroactively enter an existing snapshot, and a copied or extracted `tau`
keeps its captured `V_τ`.

`tau` is not another Object and does not add a fourth Object coordinate. `Q`
and every ordinary member in `V_τ` remain Objects governed by the existing
`<Val1?,P,Val2>` ontology. The closure only preserves their type-specific
pairing so a copied or extracted type carries its own callspace. Because `tau`
is not an Object, any Object-position representation — including the
`BareProduct` element inside `Σ_Object` — stores the canonical encoding
`EncodeTypeClosure(tau) ∈ Object`, not `tau` itself. The representation
boundary `EncodeTypeClosure` is defined in symbol-first §4.7.

Evaluation of a type-valued binding yields the complete closure; it never
degrades into `Core(tau)` on its own. Observation is consumer-specific
projection of that first-class value — each consumer names which part it
needs, and no rule silently re-reads `tau` as `Q`:

```text
Read(type-valued place) = tau

old Q-consuming rule observed type = Q
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

`@` is not part of this classification: it is the privileged place-observation
operation that yields a lifetime value (canonical owner
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

The binder is not a `mu`-type, an equi-recursive type rule, or permission for
cyclic Object content.

Each `tau` is an immutable snapshot; no operation mutates an existing closure.
The non-structural `Associate` operation (defined below; place operation §7.1)
replaces a carrier's stored snapshot with a derived `tau' = <Q', V_τ>` without
changing `V_τ` or adding structural incidence; `extend` remains the structural
transformation. Copying a type-valued binding copies the
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

Ordinary associated-member installation is a **non-structural snapshot
update**, not `extend`:

```text
Associate(
    tau = <Q, V_τ>,
    selector,
    value
)
  = tau' = <Q', V_τ>

P(Q')    = P(Q)
Val2(Q') = Val2(Q)[selector := value]

V_τ unchanged
no DirectPatternChild added
CoreOnFormationLine(w, Q') holds
  (same FormationLine(w, Q_0, V_τ); Q' is on the line)
```

The place-level operation is `old := Read(type_place); new := Associate(old, f,
expr); Write(type_place, new)` (§7.1). The old `tau` copy is never mutated; the
carrier receives a fresh snapshot that shares the same `V_τ` and the same
formation line. Only `extend` establishes a new `FormationLine` with a fresh
witness `w'` and a new `V_τ'`.

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
`Root(tau) -> current mutable Symbol -> current V` indirection participates in
type identity or call lookup.

## 3. Value judgment versus place judgment

The model uses two distinct judgments. One evaluates an expression to a value;
the other resolves a name to a writable place.

Value evaluation:

```text
Γ ⊢ x ⇓ v
```

means an expression / rank / type expression evaluates to value `v`.

Place resolution:

```text
Γ ⊢ x ⇐ p
```

means a declaration extension / namespace injection / assignment-like operation
resolves `x` to a writable place `p`.

These are not interchangeable. Canonical creation beneath a pure type slot
selects the explicit higher-level ref of that slot:

```lang
let f::(t |> (type ref)) = ...;
```

When the source is instead a Symbol `S`, the Symbol must first be borrowed and
its ordinary same-name `type` field projected: `let f::((S ref).type) = ...`.
`AsType(S) = S |> type` is by-value only and never participates in place
recovery.

### 3.1 General value binding resolves symbols first

The ordinary rule:

```lang
let r = expr;
```

is:

```text
value(symbol(r)) := evaluate(expr)
```

When `expr` is a source path, value evaluation is not direct value naming:

```text
source path
  -> resolve Symbol
  -> read the selected value / PatternValue from that Symbol
  -> bind the value to the destination Symbol/Place
```

Thus:

```lang
let a = b;
```

binds the exact value read through `symbol(b)` into the fresh destination
`symbol(a)`. It does not alias the symbols or merge their places.

Formally:

```text
resolve(b) = s_b
read(s_b)  = v
fresh SymbolId s_a
fresh PlaceId p_a
--------------------------------
bind(a, v)
```

The source carrier `s_b` is not stored as part of `v` after evaluation.
Provenance may mention it; semantic value identity does not. Consequently no
ordinary binding path may recover associated operations by mapping
`TypeValueId` back to an “original defining Symbol”. A source Symbol projects
`tau=<Q,V_τ>` directly; an already-held type value uses its intrinsic
`CallSpace(tau)=V_τ`. Ordinary Pattern/namespace observation separately follows
`Core(tau)=Q`.

The same separation applies inside derived semantic material. A struct field,
callable signature, canonical argument key, or extraction view that denotes a
type observes the default `Core(tau)=Q`, exactly as the old `type = Q` rules
did; `Addr(Norm_type(tau))` remains available where the language has
independently frozen whole-snapshot identity (transport, distinguishing
shared-root snapshots):

```text
field source path
  -> carrier Symbol
  -> read TypeValue tau
  -> record Core(tau) = Q as field-type identity
```

An implementation may temporarily retain the carrier Symbol for graph
navigation or provenance, but it is not part of field-type equality,
Pattern-head identity, or generated type-definition identity. Consequently
`(uint8 field) struct` and `(T field) struct` have the same field-type material
after `let T: type = uint8`; a reverse `TypeValueId -> original Symbol` lookup
would incorrectly make ordinary binding observable.

Extraction interfaces follow the same split. Their semantic owner/type
coordinates are the owner `TypeValue` and Pattern identity. A graph carrier may
still be present to reach installed field projection Symbols, but
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
let t1::((t ref).type) = bool;
```

resolves `symbol(bool)`, reads its `PatternValue`, and binds that value to the
destination prospective ProjectionSlot under `(t ref).type`. It does not reroot the pattern, rewrite its
navigation, or make the destination symbol identical to the pattern owner.

Literal syntax is the explicit exception only to source-path resolution. It
still evaluates to a value and uses the same binding rule. In
`let a = 'a';`, the left `a` is a symbol name while the right `'a'` is a
character literal; matching textual content does not make them the same object.
Pattern values have no analogous standalone literal syntax, so same-spelled
symbol paths and pattern diagnostic names must be kept especially distinct.

### 3.2 One navigator, many projections

Symbol-first resolution is a single ordered pipeline:

```text
Path -> SelectedHead -> ⟨HostChain, TerminalSymbol⟩ -> ContextDirectedProjection
```

The stability claim applies to the **tail**, not to the head. Once the head
symbol is selected, the remaining navigation is decided by the path alone: it is
**not** decided by whether the result is subsequently used as a call target, a
type, a value, an injection target, or an extraction subject. Head selection is a
separate, earlier step with its own rule.

#### 3.2.1 Head selection: bare versus explicitly anchored

The two forms do not use the same rule, and the difference is confined to this
step:

```text
ResolveBare_q(name)
  = the nearest enclosing Symbol spelled `name` that carries the required
    coarse facet q

ResolveExplicit(path)
  = the uniquely designated anchor, taken as written
```

A bare head may look outward, and the coarse facet `q` demanded by the use site
participates in that search. An explicitly anchored path may not look outward at
all. The search discipline is:

```text
bare head    : walk outward; stop at the first same-spelled Symbol carrying q
explicit head: no outward walk; the written anchor is the head or resolution fails
```

The outward walk is bounded to exactly one decision. Once a Symbol carrying `q`
is found, that Symbol is the head, permanently:

```text
overload resolution failing inside the selected head
  -> the program is ill-formed
  -> NOT a reason to resume the outward walk
```

This is what keeps a local Symbol with a type-capable `Q` but no callable val
member from silently shadowing an outer callable Symbol of the same spelling:
at a call site the coarse role/member demand is callability, so a local Symbol
that carries no callable val member is simply not a
candidate head. It is equally what stops the search from degenerating into
"retry outward until something type-checks" — the demand is coarse, and it is
consulted once.

`q` is coarse in the strict sense: it distinguishes facet presence, never
signatures, argument types, arity, or specificity. Head selection therefore never
becomes overload resolution in disguise.

#### 3.2.2 Tail navigation is context-independent

After the head is fixed, one navigation algorithm serves every context:

```text
SelectedHead                                     -> Symbol
for each following component:
    select the current Symbol's object facet
    push that object as a host layer onto HostChain
    enter that object's OWN Val2 place
    look up the next associated Symbol
-> ⟨HostChain, terminal Symbol⟩
```

Only the final step is context-directed, and it projects a facet of the already
chosen terminal symbol:

| context | projection |
| --- | --- |
| call target | callable sibling vals |
| type | pure-P member |
| value | sibling vals |
| extension target | extendable host object / place |
| extraction | Pattern facet |

Consequently `f::T` denotes `Val2(T)[f]` in all of

```lang
let A: type = f::T;
let B = (f::T) meta_fn;
let g::((U ref).type) = f::T;
(…) |> f::T;
g::f::T
```

and differs only in the facet each site reads. Resolving the same spelling as
an object-level `Val2` path in one context and as a namespace path in another
would make path meaning depend on its consumer, which is name-category-first
resolution in disguise. Namespace children remain reachable: a step consults
the current symbol's object facet and its associated namespace, so ordinary
namespace paths keep resolving unchanged.

The coarse facet of §3.2.1 is not an exception to this. It participates only in
selecting the head, once, and it distinguishes facet presence rather than
meaning; the tail steps and the final projection remain as above.

The host layers traversed on the way are retained as an ordered `HostChain`,
because per-layer exposure is a conjunction over every layer
(`Expose(g::f::T, φ) = Expose(T_t, φ) ∧ Expose(C_f, φ) ∧ …`) rather than a
property of the terminal symbol alone. Consumers do not re-derive this chain:
ordinary invocation reads the whole navigation and refuses the target unless
**every** host layer is exposed at the current phase, so a hidden outer layer
cannot be bypassed by a visible terminal reached through it. Cross-root
resolution likewise deduplicates on the full `⟨HostChain, TerminalSymbol⟩`; two
roots that reach one terminal through different host chains are a navigation
ambiguity, not a silently-merged result.

## 4. Ordinary type-value binding

Type-value binding is the general value-binding rule under a `type`
expectation, not a separate assignment mechanism. The form:

```text
let T: type = uint8
```

means:

```text
symbol(T) = fresh symbol
place(T) = fresh writable place at current lexical level
value(T) = value(uint8)
type_value(T) = type_value(uint8)
pattern_value(T) = pattern_value(uint8)
```

This must be read precisely:

```text
T is not a fresh nominal type.
T is not a symbol alias.
T has fresh place identity.
T may evaluate to an existing type value.
```

`T` is a new symbol with its own fresh, current-level writable place. Its *type
value* is the value read through `uint8`, while its *place* is its own. Binding to an existing
type value does not generate a new type, and it does not forward to `uint8`'s
symbol or place.

This ordinary declaration rule does not license a meta return Symbol to use an
external pure Object as its distinguished pure member. A canonical meta
instance has an additional self-root invariant: if its return Symbol contains a
distinguished pure member `Q`, `Q`'s outer Pattern root must be the
`MetaInstanceScope`. The condition is `Q`'s presence, independent of
`TypeRole(Q)`. Thus ordinary
`let T: type = uint8` remains legal while direct `r = uint8` as a meta return
type construction is rejected.

Consequently, associated-member creation through `T`:

```text
let f::(T |> (type ref)) = ...
```

executes:

```text
place(T) += { f ↦ ... }
```

and not:

```text
place(uint8) += { f ↦ ... }
```

Member creation is a place operation. Structural extension is different: it is
the pure value transformation `extend`, while `inject` is the explicit
read--extend--write wrapper defined in the symbol-first construction document.

### 4.1 Atomic builtin types, concrete numeric types, and literal typing

The literal spelling family, atomic builtin type, and concrete numeric type are
distinct:

```text
LiteralFamily
  = Integer | Float | String

AtomicBuiltinType T
  = Uint | Int | Float | Buffer | Str

NumericTypeKey Tnum
  = NumericFamily x width
```

A literal family records normalized syntax and is not a type identity. Each
member of `AtomicBuiltinType` denotes an actual atomic builtin type whose
identity, once installed, comes from its Type symbol; it is not merely a
classifier invented by literal materialization. The Rust enum is a lookup key,
not itself a `TypeValueId`.

A concrete numeric key selects a type object such as `uint16` or `float32`.
Current Rust code carries the first-order `TypeValueId` projection derived from
the installed canonical core Type symbol. That projection is transitional
material and does not claim final whole-snapshot type-value identity:

```text
literal spelling
  -> LiteralFamily
  -> contextual/default concrete Tnum selection
  -> resolve canonical Type Symbol
  -> project TypeValueId
  -> materialize semantic value
```

The lexical frontend continues to preserve spelling only; it does not choose
width, signedness, precision, or overflow behavior. The semantic selection
step extends that result without changing lexer meaning. An unsuffixed default
and range/context rules remain separate decisions.

Requiring a concrete `Tnum` for numeric literals does not imply that
`uint`/`int`/`float` cease to be Type values. It means only that the numeric
literal's final type is the selected concrete numeric Type rather than the
atomic numeric family Type.

The design target for a string literal is a compile-stage `str` value, not
`str ref`, implicit storage, or a lifetime extension. That statement requires
a `str` Type symbol and its first-order projection in the semantic world. The
current core bootstrap installs `uint8`, `uint16`, `uint32`, and `float32`, but
not `str`; the current helper can materialize a string only when its
`AtomicBuiltinTypeRegistry` contains a resolved `str` projection. It must not
accept an arbitrary numeric identifier as an implemented core `str` carrier.

## 5. Borrow views

There is no declaration form that makes two symbols share one symbol identity
or one place. Shared observation is expressed by the borrow constructors `ref`
and `share`; the privileged place-observation `@` yields a lifetime value
(`LifetimeVal`) and is not a borrow representation.

### 5.1 `ref` and `share` are privileged actual-place builtins

`ref` and `share` are ordinary meta-function calls on their operand. Each
operator has two overload roles (canonical owner
`../lifetime/lifetime-policy-and-overload-boundary.md` §2): a **type-forming**
overload, selected for a type operand, that forms the borrow **type** value
(`t ref` / `t share` as TypeValues), and a **borrow-forming** overload inside
the formed borrow type's callspace that produces the borrow **instance**. Only
the selected borrow-forming defaults of `ref` / `share`, and the single `@`
operation, may obtain the actual's place
(`PrivilegedActualPlace(ref-family)`, `PrivilegedActualPlace(share-family)`,
`PrivilegedActualPlace(@-family)`; canonical owner
`../lifetime/lifetime-policy-and-overload-boundary.md` §2). An ordinary user
function that spells the same formal head cannot obtain that place.

There is no global `E ref = Ref(Read(E))` law. The result depends on the
selected overload and on whether that overload's default implementation
exercises its place privilege:

```text
prepare actual value
-> select unique ref overload
-> if selected builtin requires place:
       acquire PrivilegedActualPlace(actual)
-> execute default
```

For a type-valued binding `t : type`, `t ref` selects the **type-forming**
overload and yields the TypeValue `tau_(t ref)` (the borrow type of `t`), never
a borrow instance; the borrow instance over the type-level place is produced
only by invoking that borrow type explicitly with `t |> (type ref)` (§5.2).
For an ordinary `Val1`-bearing value, the selected borrow-forming default
obtains the actual's place via its privilege (§5.1.0).

#### 5.1.0 The selected borrow-forming default obtains a privileged actual place

The selected borrow-forming default of `ref` (or `share`) obtains the place of
the actual — not a second place source derived from `Read(E)`. The old
`ObjectPlace(value) ≠ CarrierPlace(E)` binary is retired: it existed only
because `ref` was assumed unable to observe the actual place while `@` could,
and that assumption no longer holds now that `ref` and `share` carry
`PrivilegedActualPlace(ref-family)` / `PrivilegedActualPlace(share-family)`.

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
nowhere and carries no borrowable identity — supplies no place, so `ref` has
**no applicable candidate** on it. `ref` never materializes storage on the
writer's behalf and never silently retargets to a carrier slot; a temporary
must first be bound to a named place before it can be borrowed.

`ref` is an ordinary meta-function call. It does not ask which symbol slot the
value came out of, and does not consult, capture, or export it. Therefore:

```lang
let t = uint8;
let r = t ref;
```

Here `uint8` evaluates to the complete type value `tau_uint8`, so `t` is
type-valued. `t ref` selects the type-forming overload and yields the TypeValue
`tau_(uint8 ref)` — not a borrow instance, and not a borrow of `Core(tau)`.
Reaching the type-level carrier slot of a pure type binding uses
`t |> (type ref)` (§5.2). A Symbol's type-member slot instead uses `(S ref).type`.

`share` differs from `ref` in the capability it grants, not in the judgment it
uses: a `share` view admits reading and passing but is not an assignable place
and cannot be an `inject` target (§5.5).

#### 5.1.1 A `Val1` payload makes `ref` sufficient on its own

When the operand slot has a `Val1` payload, `Read` yields the complete object
that carries it, and `ref` borrows that object. Nothing further is required, and
nothing is elaborated in front of the operator:

```lang
let s: symbol = ...;
let r = s ref;              // Read(s) : symbol, so r : symbol ref
```

A symbol value is value-bearing, so `s ref` is the ordinary "form a borrow of
this value" operation. Because `Read` does not descend into `Val1`, `r` is a
`symbol ref` and **not** a reference to the member array held inside the
symbol. The referent is the value that `s` holds: `Target(s ref) =
PrivilegedActualPlace(s)` (§5.1.0) — there is exactly one place source, and no
separate carrier/binding-slot place exists for the view to miss.

When the intent is to form a borrow of the symbol's **type** rather than the
symbol value itself — i.e. `(s |> type) ref` — an explicit `AsType` in a
type-expected position is required: there is **no global** `symbol` ref/share
forwarding bridge (lifetime §2.0.1) and no implicit `AsType` during candidate
matching. A bridge overload authored in a local `ref` Symbol is local Symbol
algebra, not a language default.

The rule is about the presence of the `Val1` dimension, not about type-rank. An
object that happens to sit at type rank and still carries a payload takes the
same path, and is likewise named by its own host Pattern:

```text
x        = ⟨ v, P_val_has_type_field, Val2 ⟩
Read(x)  = ⟨ v, P_val_has_type_field, Val2 ⟩
x ref    : val_has_type_field ref
```

Reaching `v` itself is an ordinary member/projection operation on the read
result, not something `Read` or `ref` performs implicitly.

No implicit projection or conversion participates in an operand position. `s ref`
is never elaborated into `s |> type` or another role projection, because an
operand or argument position performs no implicit type conversion. An explicit
`AsType(E) = E |> type` has two ordinary cases:

```text
AsType(S : symbol) = TypeProjection(S)
  = tau_S
    when TypeSlot(S) = Some(tau_S)

AsType(tau) = tau
  when tau is already a complete TypeValue
```

The second rule validates an existing complete type value. It does not treat a
bare pure namespace Object as a complete type, wrap a namespace, or search for
a hidden type member. `TypeRole(Core(tau))` holds for every carried `tau`; the
complete result is the whole `tau = <Q, V_τ>`, whose callspace `V_τ` was fixed
at formation (the direct TypeMember members of symbol-first §2.1). A copied
or extracted `tau` keeps its captured `V_τ`; members created under the same `Q`
later never enter an existing snapshot. Payload presence alone
remains irrelevant to type
applicability: a Symbol may carry no `tau` (no type projection), or carry a
complete `tau`. A
language-designated type-expected position may
insert `AsType`; ordinary operand positions may not. See §5.6.

#### 5.1.2 No implicit borrow formation

Borrow formation is never candidate adaptation, structural repair, policy
migration, or automatic argument passing:

```text
Object =/=> Object ref | Object share
Symbol =/=> Symbol ref | Symbol share
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
E |> (type ref) is undefined when E has no carrier place (a freshly computed temporary)
t |> (type ref) is not a general PlaceOf(E) available on every expression
```

The former carrier-borrow `@` group that yielded `type ref` is retired: `@` is
a privileged place-observation operation that yields a lifetime value, never a
borrow view and never a `type ref`
(`../lifetime/lifetime-policy-and-overload-boundary.md` §1–§2.1). Reaching the
carrier slot explicitly uses `t |> (type ref)` or `(S ref).type`.

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
| symbol value with `Val1` | `symbol ref` | no |
| type-rank value with `Val1` | borrow of the complete object, named by its host Pattern | no |
| pure pattern value | `ref` of that pattern value | only to reach the carrier slot |
| pure `type` slot | type formation: the TypeValue `t ref` (the borrow type) | yes — for a borrow instance over the carrier place |

The `Val1` column decides only *which path applies* — the borrow-forming `ref`
on the value versus type formation / explicit `type ref` invocation. It never
means that the result of `ref` descends to the `Val1` sub-object: in the rows
that yield a borrow instance, the view's referent is the complete object that
`Read` produced (§5.1); the pure `type` row yields a TypeValue, not a view.

Consequently the compile stage offers no implicit borrow formation for an
operand that has a `Val1` payload — `s ref` already does that job. The retired
`@` carrier-borrow group is not a fallback for `ref`, and `@` is not a borrow
constructor at all (`NoImplicitBorrowFormation`).

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

The former `@` fixed points (`type ref@ = type ref`, `type share@ = type share`)
and the former value-instance rule `t@ = lifetime(t)` are retired: `@` is a
privileged place-observation operation that yields a lifetime value uniformly
and is never a borrow constructor
(`../lifetime/lifetime-policy-and-overload-boundary.md` §2.1). The old blanket
equation “`@@` is identity on every borrow view” does not return.

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
it is the separate `Open_Γ(value)` judgment over construction lineage (§6 and
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

A frozen type-valued slot may still be observed through `type ref`. If the view
is writable, the complete current value may be replaced by any compatible,
well-formed type value. What is forbidden is using the frozen pointee as the
`old` input of `extend`; holding a reference does not change that value's
construction lineage.

The holdable interval is consequently its ordinary borrow-valid region, not an
Open window. Weakening remains useful when write authority is unnecessary:

```lang
r share    // type share: still observable, no write authority
```

Reachability alone still forms no view, and neither reachability nor a view
decides construction state:

```text
GlobalLifetime(q) does not imply Open_Γ(Value(q))
Γ ⊢ r : type ref does not imply Open_Γ(Read(r))
```

`Open_Γ` is defined from `ConstructionLineage` and the current compile-time call
stack in
`symbol-first-meta-construction-and-pattern-injection.md` §12.

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
extend on a type value      ->  Open_Γ(value)
inject through a type ref   ->  Open_Γ(Read(ref)) and Writable(Target(ref))
returning / storing a ref   ->  ordinary lifetime/capability escape check
```

Returning a `type ref` from a `compile` callable is therefore governed by the
same borrow escape rule as any other reference. It may remain usable after the
pointee freezes; a later `extend`/`inject` attempt rechecks the current value's
lineage and may fail independently of the reference's validity.

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

`AsType` returns the complete type value `tau` carried by a Symbol, or
validates an already-complete TypeValue as
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
TypeOf(symbol) = type
```

The old `typeof(type) = symbol` / `typeof(type |> type) = type_1` path is
retired: `type` is no longer a `symbol` whose callable members must be reached
through the Symbol's shared `V`. With `tau = <Q, V_τ>`, `type` carries its own
callspace, so `TypeOf(type) = type_1` directly.

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

```lang
let x: s = ...;             // type-expected: elaborates to s |> type
let r = s ref;              // operand: no projection; r : symbol ref
```

The distinction is positional and fixed by the language, never inferred from the
operand's shape. An operand position never acquires a projection because a
projection would make the program check.

`@` is always an operand operation, so it never performs this implicit
projection. `AsType` therefore cannot be followed by `@` to recover the source
Symbol's type-member slot. `@` yields a lifetime value, not a borrow
(`../lifetime/lifetime-policy-and-overload-boundary.md` §2.1). Symbol provides
an ordinary same-name field/accessor family instead:

```lang
S.type         : type
(S ref).type   : type ref
(S share).type : type share
```

`S.type` is the by-value projection of the complete type snapshot whose `Q`
satisfies `TypeRole`, and agrees in value with `AsType(S)`. The ref/share cases
preserve their borrow observation through
ordinary field projection; they do not reverse-map a type value to an origin.
When `t` itself is already a pure `type` slot, `t |> (type ref)` is the explicit
construction that reaches the type-level place. No `S@` or `(S |> type)@`
shorthand is defined.

## 6. Writability, member creation, and construction openness

The checker owns three independent judgments:

```text
Writable_Γ(q)
CanCreateMember_Γ(parent_place, selector)
Open_Γ(v)
```

`Writable` is a place/borrow-capability question. `CanCreateMember` combines a
stable parent place with construction-unit, lexical, policy, and freshness
authority. `Open` is a value-lineage question used by structural `extend`.
None is a spelling or proof of another:

```text
Writable_Γ(q)           does not imply Open_Γ(Read(q))
Open_Γ(v)               does not imply Writable_Γ(Carrier(v))
CanCreateMember_Γ(p, n) does not follow from Writable_Γ(p) alone
```

A writable slot may contain a frozen type value that can be replaced wholesale
but cannot be structurally extended from. Conversely an Open value may be
extended purely and bound elsewhere even when its source is immutable or has no
write-back place.

At minimum, ordinary place operations reject a core/external stable place, a
place reached only through `share`, a place outside its borrow lifetime, or a
place whose policy denies the action. Member creation additionally rejects a
parent outside the current construction unit or an already-instantiated child.
Structural `extend` independently rejects a value whose `ConstructionLineage`
is not Open in the current compile-time stack.

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
let f::((T ref).type) = ...;
```

No binding or borrow view can amplify the place authority it observes:

```text
Capability(view of p) ≤ Capability(p)
```

## 7. Projection slots and namespace extension targets

Namespace/member creation is a *place* operation, not a value operation. The
target is not determined by ordinary expression evaluation of the target path.

The intended flow:

```text
parse / normalize the target path
resolve the path as a parent place plus selector
obtain the stable prospective ProjectionSlot
check creation or write applicability
install NamespaceDelta under that place
```

Navigation does not fail merely because the final child has not yet been
instantiated:

```text
ProjectionCoordinate(parent_place, selector)
ProjectionSlot(parent_resident, selector)
ProjectionSlotIdentity
  = <ParentResidentIdentity(parent_resident), selector>
Contents(ProjectionSlot) ∈ Some(Object) | None
```

`ProjectionCoordinate` is the reusable logical navigation coordinate.
`ProjectionSlotIdentity` additionally names the current parent resident. `let`
may change one slot's contents from `None` to `Some(value)` without changing the
slot identity; ordinary `=` requires `Some(existing)` and never creates the
missing child. Continuing navigation *from* `None` is ill-formed because there
is no child Object whose own `Val2` could host the next selector.

Borrowing a projection records its formation-time `ProjectionSlotIdentity`,
including when `Contents = None`:

```text
ProjectionCoordinate(parent_place, selector)
  != ProjectionSlotIdentity(parent_resident, selector)

Target(Borrow(Nav(parent_borrow, selector)))
  = ProjectionSlotIdentity(parent_resident_at_formation, selector)
```

Changing `None` to `Some` within that slot is not retargeting. Wholesale parent
replacement ends the old parent resident, produces a distinct family of
projection slots, and invalidates borrows of the old slots under ordinary
lifetime rules. It never redirects them to the replacement parent's same-named
or same-positioned slot; only `rebind` acquires that new target (§5.4). The
concrete generation/version encoding remains implementation debt, but this
resident-slot distinction is target semantics.

Navigation preserves the observation kind:

```text
Nav(type,       n) -> type | None
Nav(type ref,   n) -> (type | None) ref
Nav(type share, n) -> (type | None) share
```

The optionality describes current slot contents, not the existence of the
ProjectionSlot. Full optional-pattern algebra remains deferred.

Named selectors and bare-Product ordinal selectors use this same mechanism:

```text
ProjectionSlot(parent_resident, name)
ProjectionSlot(parent_resident, pos_i)
```

For `T*N` and `T*omega`, ordinal topology belongs to the current Sequence value,
so `CanCreateMember(sequence, pos_i) = false`. Indexing observes an existing
in-domain slot; it cannot grow or resize a Sequence through `let`.

There is no forwarding-chain step: a path resolves to exactly one place, and a
borrow view interposed on that path either denotes the same place (`ref`) or
removes eligibility entirely (`share`).

The resolver here is asking "which eligible place does this path name?", not
"what value does this path evaluate to?". An extension that resolves to a value
rather than a place is ill-formed.

Writability alone does not grant construction ownership. Under the current
future construction contract, another source file cannot reopen a namespace,
type, pattern, ordinary value-member, or overload subtree created by a parallel
`SourceConstructionUnit`, even to add a previously absent child. Physical
directory authority and construction-unit ownership are specified in
`symbol-construction-units-and-namespace-origin.md`.

An ordinary Symbol's pure role member is installed at most once. Repeating:

```lang
let T = A;
let T = B;
```

as two competing pure-role definitions is a conflict, not implicit `A | B`.
For `struct`, the installed member additionally satisfies `TypeRole`. Child
construction and sum construction require explicit APIs and remain distinct
from repeated ordinary binding.

### 7.1 `let` creates a member; `=` writes an existing target

The two forms are distinct operations on the same resolved prospective target:

```text
let f::((T ref).type) = expr   — instantiate a missing associated member
f::((T ref).type) = expr       — write an already existing member
```

Bare `=` never creates a missing member. There is no declaration shorthand in
which omitting `let` silently recovers creation semantics. Both forms use the
place resolution of §7, but they discharge different obligations:

```text
let: Contents(slot) = None and CanCreateMember(parent, selector)
=  : Contents(slot) = Some(old) and Writable(slot)
```

Freshness never implies creation authority, and writability never implies
freshness. `let` is the only operation that changes `None` to `Some(value)`.

The two forms also differ in what they change about the host:

```text
let f::((T ref).type) = expr -> Associate(tau, f, expr) = <Q',V_τ>; P and V_τ unchanged (§2.2)
extend(TypeValue(T), Δ)      -> new tau' = <Q',V_τ'> snapshot
inject((T ref).type, Δ)      -> read + extend + write through the type ref
```

An ordinary member declaration adds a `Val2` entry under an existing pattern
name; it does not widen `P(T)`. Widening the host pattern with a new child
pattern is exactly what `extend` does; `inject` is its place-level wrapper. Both
are specified in
`symbol-first-meta-construction-and-pattern-injection.md` §8. Both are limited to
extending the current parent pattern with a *direct* child; neither reaches into
a grandchild pattern.

Assignment carries no `extend`-specific construction-lineage check, but that is
not the same as carrying no check. The canonical four-layer assignment model in
`symbol-first-meta-construction-and-pattern-injection.md` §4.5.1 governs, and the
text below only spells out its layer 2 for this document.

Layer 2 — universal write applicability checks exactly:

```text
Writable(lhs)              — the left side names a writable place
Compatible(P(lhs), rhs)    — the right value conforms to the target's Pattern
ValidCapability(lhs)       — lifetime and capability conditions of lhs hold
```

Assignment performs no `extend`-specific construction-lineage check: there is no
requirement that the RHS came from a particular producer. That freedom covers
layer 1 (the RHS operation, including `extend`, discharged its own Open check)
and nothing more. Layers 1, 3,
and 4 of the canonical model — RHS operation legality, result-object invariants
(`WellFounded` / `Canonicalizable` / `NoForbiddenCycle`), and the enclosing
region's semantic-boundary constraints (meta self-root, ref / pattern-value
lifetimes, global type-bearing mutability limits, seal /
global promotion, single-pure-role-member bound) — remain independently applicable to
the write result.

## 8. Type values in overload and pattern matching

Ordinary type matching for overload and pattern compatibility compares
canonical type values, not source symbol names:

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
matching, not a provisional stand-in; whole-snapshot identity
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

The same separation applies to normalized pattern layers. If the layer is the
body of a Pattern and every direct element has a complete top-pattern
navigation name, it is
`Map<CanonicalFullNavigation, CanonicalPatternValue>`. A naked Product remains
positional regardless of whether its elements are named. `SymbolId` and
`PlaceId` identify carriers/locations; they are neither map keys nor resident
values. Extraction resolves a source symbol, reads its `PatternValue`, and
looks up that value by complete navigation and normalized resident. A symbol
path may share the value's navigation spelling or differ from it without
changing this sequence. Source/provenance classification does not participate
in `PatternValue` identity.

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

## 10. Relation to current implementation

The `lang_build` semantic spine implements the identity core of this document
only through its existing type-core/`Val2` substrate: opaque-`Val1` Object
normalization, first-order `TypeValueId`, per-carrier places, and meta return
self-root validation. It does not yet represent the complete immutable
`tau=<Q,V_τ>` closure or use `Norm_type(tau)` for equality/keying/copying. The
current `TypeObject` adapter is implementation transport, never the canonical
complete type model or a binding-level policy authority.

Registered implementation debt — semantics closed here, not yet built:

```text
full three-component Norm(x) including recursive Norm_Val1?
  (current normalizer keeps an opaque Val1 leaf)
ref / share / @ / rebind operations and their overloads
type ref and type share values, and ValidContext for them
the independent writability and construction-lineage Open judgments of §6
the = assignment operator and its four-layer check (§4.5.1 there)
construction-unit ownership enforcement
```

Whole-snapshot comparison is required only at independently specified
snapshot-sensitive positions; ordinary type equality/keying keeps observing
`Core(tau) = Q` by default (minimal-change rule, §2.2). Migrating the remaining
first-order comparison consumers is therefore not outstanding implementation
debt.

The retired alias-forwarding model (`AliasChain`, symbol/place forwarding, and
alias-forwarded extension places) is not
implementation debt. It is removed from the target semantics and must not be
revived as future work.

## 11. Non-goals

```text
No parser syntax change.
No full type checker.
No full lifetime/access-tree checker.
No runtime lookup implementation.
No package re-export semantics.
No permission escalation through borrow views.
No revival of symbol-alias or place-forwarding declaration forms.
```

## 12. Relationship to other documents

The documents below are adjacent or background design. They do not define the
distinctions specified here, and this document does not depend on them for its
meaning.

- `symbol-first-meta-construction-and-pattern-injection.md` — canonical
  symbol-first facet resolution, `PatternValue`, `compile` / `meta`, pattern
  scopes, `struct`, pure `extend`, place-level `inject`, construction-lineage
  `Open_Γ`, and the
  binding/install boundary. It uses this document's `SymbolId` / `PlaceId` /
  `TypeValueId` and place judgments.
- `../lifetime/lifetime-policy-and-overload-boundary.md` — canonical owner of
  `@` (privileged place observation yielding a lifetime value) and of
  `ref` / `share` borrow formation, plus escape checking. This document
  supplies only the `Origin`/`Value` split that `@` consumes.
- `type-associated-function-objects-and-access-trees.md` — field functions,
  same-name receiver overloads and access-tree work. It references
  this document for the canonical value / place / borrow-view distinction rather
  than restating it.
- `early-meta-functions-and-namespace-graph.md` — the build / namespace graph and
  early-meta slice, including the v0.6 placeholder `TypeObject` representation
  this document supersedes as the long-term semantics.
- `symbol-construction-units-and-namespace-origin.md` — canonical
  `NamespaceOrigin`, construction-unit ownership, physical contribution
  authority, pure/type role refinement, and cross-file closure rules.
- `pattern-normalization-and-first-order-overload.md` — the pattern/type
  candidate-preparation layer that consumes ordinary type-value observation
  (`Core(τ) = Q`) for first-order type matching.
