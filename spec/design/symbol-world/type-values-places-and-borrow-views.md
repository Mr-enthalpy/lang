# Type Values, Places, and Borrow Views

**Status: canonical target semantics for Object identity, complete type-closure
identity, place identity, and borrow views. Current `lang_build` implements only
the first-order identity core: recursive normalization over the present
type-core/`Val2` substrate with an opaque `Val1` leaf, `TypeValueId`, and
per-carrier places. The complete `tau=<Q,V_τ>` snapshot and
`Norm_type(tau)`, full recursive `Norm_Val1?`, the borrow-view operators (`ref`,
`share`, `rebind`), the place-sensitive lifetime observation (`@`),
construction-authority (`OpenHere_Σ` / `WindowLive_Σ`) judgment, and type checker
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
substitution and **not** a second name for a Symbol. And value equality is
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

- `SymbolId` is the identity of a `Symbol` constructor value (name-graph node).
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
relation. Its formal criterion is defined normatively in
`../design/patterns-overload/pattern-values-relational-semantics-and-extraction.md`
§13 (`TypeRole(Q) iff NamespaceRole(Q) and HasRegisteredSelfConstruction(Q)`,
witnessed by an actual `Val2(Q)[s] = K` member), consumed here via §2.2. This
layer only consumes `TypeRole` as an opaque predicate.

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
allocation order is not semantic content, so the walk must resolve each name to
its cluster symbol and normalize that symbol's own members.

This exclusion does not erase `StableTargetIdentity(place)` from a borrow-view
normal form. In the ordinary object case a place is an observation source; in
the borrow-view case the target is what the value denotes.

This is what makes an open construction observable at all. Given

```lang
let fn = (...): meta -> _ :symbol = {
    let t = (() t) |> struct;

    let f::(t |> (type ref)) = X;
    let A = t |> compile_fn;

    let g::(t |> (type ref)) = Y;
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
`Val2` such as `let loop::t = t;` all have **no normal form** at the stage where
they are materialized. A finished shared acyclic subtree remains valid DAG
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
  iff tau = <Q, V_τ>
  and Q is a well-formed pure Object
  and PatternClosureConsistent(Q, V_τ)

PatternClosureConsistent(Q, V_τ) iff
  WellFormedCore(Q)
  and ∀F ∈ ClassifierDomain(V_τ):
      Anonymous(F) ∧ DirectClassifierHome(F) = TypeMemberScope(Q)
      (home eligibility; equivalent to HomeEligible_Q(F), canonical §2.1)
  and AllBoundRefsBoundAndRestricted(bind α.⟨Norm(Q), Norm_V^α(V_τ)⟩)
      (every BoundRef reachable during Norm_type^α(Q, V_τ) is bound by α
       and belongs to an authorized static edge kind;
       BoundRef(alpha) notin Children_owned)
  and all structural/interface registrations referenced by Q and V_τ
      are internally well-formed
  and no CurrentAuthority / OpenHere_Σ / GenerationRegime /
      WindowLive_Σ / stack / provenance premise is used
  -- a structural judgment over the current closure value, with no
     dependence on how tau was produced; the member condition compares
     DirectClassifierHome(F) with the current TypeMemberScope(Q) — it
     never asks for which old core F was created

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
one entity; neither view may be extended independently of the other. This is
not the identity `V_τ = Val2`: it is the statement that both sides constrain
the same semantic entity.

```text
TypeValueRole(tau)
  iff WellFormedTau(tau)
  and TypeRole(Q)
      -- the type-value role; equivalently CompleteType(tau)

NamespaceClosure(tau)
  iff WellFormedTau(tau)
  and NamespaceRole(Core(tau))
      -- any well-formed closure over a namespace-role core

TypeClosure(tau)
  iff WellFormedTau(tau)
  and TypeRole(Core(tau))
      -- a closure whose core has registered self-construction

TypeClosure(tau) => NamespaceClosure(tau)
      -- TypeClosure(tau) ⊂ NamespaceClosure(tau): the type closure is a
         proper sub-judgment of the namespace closure

NamespaceOnly(tau)  iff NamespaceClosure(tau) and not TypeClosure(tau)
      -- equivalently: NamespaceRole(Q) and not TypeRole(Q)

CallSpace(tau) = V_τ
  // Intrinsic property of the closure: the TypeMember set captured in this
  // closure value. It does not depend on the current host Symbol, source
  // binding, carrier Symbol, HomeSymbol, or any other provenance recovery.
  // Members created later under the same Q never retroactively enter an
  // existing snapshot; a copied or extracted tau keeps the same V_τ.
```

The closure value is first-class: `copy(τ)`, `extract(τ)`, and ordinary
parameter-passing of `τ` preserve `CallSpace(τ)` unchanged. Ordinary Symbol
sibling operations only modify `V_S`:

```text
RemoveSibling(<tau, V_S>, F) = <tau, V_S \ {F}>
```

They do not modify the `V_τ` already encapsulated in `τ`; `-=`, sibling
removal, or any ordinary mutation of `V_S` cannot reach into the closure.
Changing `V_τ` requires a semantic operation that produces a new type value
(principally `extend`; the legality of that step is the contextual judgment
`AdmissibleExtend_Γ`, and the result satisfies `WellFormedTau(τ')`
independently — never by inheritance along a formation history).

When a `Symbol` constructor value carries both `V_S` and `τ`,

```text
CallableProjection(S) = DedupCandidateIdentity(V_S ⊎ V_τ)
```

is the Symbol's call-interface exposing, in one step, the candidate-identity
quotient of the Symbol's own sibling candidates and the callspace carried by
its embedded closure (normative form defined in
`symbol-first-meta-construction-and-pattern-injection.md` §2.1; written
`V_S ∪ V_τ` only as shorthand after deduplication). This does not make `V_τ` a
function of `S`; `V_τ` remains an intrinsic snapshot of `τ`, and the formula is
only the exposure of that embedded closure callspace through the Symbol call
interface.

Whether `tau` has the type-value role or is namespace-only is decided by
`Q`'s Pattern relations, never by the sibling count of a Symbol space. The
formal judgments are defined via registered self-construction in
`pattern-values-relational-semantics-and-extraction.md` §13; the witness is a
member actually registered in `Q`'s `Val2`:

```text
TypeRole(Q)
  iff NamespaceRole(Q)
  and HasRegisteredSelfConstruction(Q)
      -- iff exists Pattern P of Q, exists s, exists C, exists K:
            Val2(Q)[s] = K and ConstructEdge_P_Q(C, Q, K)

NamespaceOnly(Q)
  iff NamespaceRole(Q)
  and not HasRegisteredSelfConstruction(Q)
```

The distinction is a judgment over `Q`'s Pattern `P` (imported from the
Pattern relational semantics), **never** `count(pure members in V_S)`. The
`NamespaceClosure`/`TypeClosure` split above follows the same core judgment.

A snapshot `tau' = <Q', V_τ>` written by an ordinary slot update is checked
the same way as any closure: `WellFormedTau(tau')` is an independent structural
judgment over `tau' = <Q', V_τ>` — `Q'` is a well-formed pure Object, and `V_τ`
is unchanged. `TypeRole(Q')` / `TypeValueRole(tau')` must be independently
re-derived from the result structure; ordinary write does not register
`ConstructEdge`, so it neither automatically preserves nor automatically
breaks `TypeRole`.

**Counter-example.** Suppose the sole type-role witness is
`Val2(Q)[s] = K ∧ ConstructEdge_P(C, Q, K)`. An ordinary write
`Write(ProjectionSlot(Q, s), K')` updates `Val2(Q')[s] = K'` but does not
register `ConstructEdge_P(C, Q, K')`. The original joint witness disappears,
so `TypeRole(Q)` may become `¬TypeRole(Q')`. There is no global theorem
`ordinary write ⇒ TypeRole preserved`.

Only structural transformations (`extend`) produce a new `V_τ'`; the resulting
`τ'` satisfies `WellFormedTau(τ')` by its own structure, never by inheriting
any formation history.

`V_τ = CallSpace(tau)` is the callspace captured into the closure value: the
direct TypeMember members placed into `tau` when it was produced
(`TypeMember_Q`, symbol-first §2.1), not a later partition of a shared Symbol
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
    -- the only τ -> τ' construction transformation

place wrapper:
    Inject_Σ(r, Δ)
      = Read -> Extend -> Write
    -- `inject` is the place-level wrapper of `extend`
```

Each `tau` is an immutable snapshot; no operation mutates an existing closure.
A carrier's stored snapshot is replaced only by ordinary slot replacement
(§7.1) — a fresh `tau' = <Q', V_τ>` sharing `V_τ`, with no structural incidence
added. `WellFormedTau(<Q', V_τ>)` is then judged independently and may fail;
slot replacement is **not** a well-formedness-preservation theorem. The
scope-preserving case is the one that keeps the member homes valid:

```text
WF(⟨Q, V⟩) ∧ CoreAnchor(Q') = CoreAnchor(Q) ∧ WellFormedCore(Q')
  ⇒ HomeCompatibility(Q', V)
      -- every F ∈ ClassifierDomain(V) keeps
         DirectClassifierHome(F) = TypeMemberScope(Q')
         (TypeMemberScopeStability, canonical §2.1)
but WF(⟨Q', V⟩) still requires all other conditions and is not
inherited from WF(⟨Q, V⟩)
```

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
not a source-semantic transformation. Only `extend` produces a new `V_τ'`
(`τ -> τ'`), and the legality of that modification step is a contextual
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
`Root(tau) -> current mutable Symbol -> current V` indirection participates in
type identity or call lookup.

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

This replaces the retired “everything is an Object” formulation as the
long-term structural invariant. It does not introduce a fourth Object
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

When the source is instead a `Symbol` constructor value `S`, it must first be borrowed and
its ordinary same-name `type` field projected: `let f::((S ref).type) = ...`.
`AsType(S) = S |> type` is by-value only and never participates in place
recovery.

### 3.1 General value binding resolves bindings first

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
`symbol(a)`. It does not alias the bindings or merge their places.

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
let t1::(t |> (type ref)) = bool;
```

resolves `symbol(bool)`, reads its `PatternValue`, and binds that value to the
destination prospective ProjectionSlot under `(t |> (type ref))`. It does not reroot the pattern, rewrite its
navigation, or make the destination symbol identical to the pattern owner.

Literal syntax is the explicit exception only to source-path resolution. It
still evaluates to a value and uses the same binding rule. In
`let a = 'a';`, the left `a` is a binding name while the right `'a'` is a
character literal; matching textual content does not make them the same object.
Pattern values have no analogous standalone literal syntax, so same-spelled
Symbol paths and pattern diagnostic names must be kept especially distinct.

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

Callability is not determined by the presence of a value-facet member. The
coarse demand `q` at a call site is callability, defined by the full
callable projection:

```text
CallablyPresent(S)
  iff CallableProjection(S) != ∅

CallableProjection(S)
  = DedupCandidateIdentity(V_S ⊎ V_τ)

ResolveBare_call(name)
  = nearest same-spelled Symbol S with CallablyPresent(S)

final call projection
  = CallableProjection(S)
```

A Symbol `S = <τ, None>` with a non-empty `V_τ` is therefore
`CallablyPresent` even though it carries no value-facet members; the `V_τ`
candidates are not lost by a value-facet-only head selection. This is what
keeps a local Symbol with a type-capable `Q` but no callable projection from
silently shadowing an outer callable Symbol of the same spelling, and it is
equally what stops the search from degenerating into
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
| call target | `CallableProjection(S)` |
| type | pure-P member |
| value | sibling vals |
| extension target | extendable host object / place |
| extraction | Pattern facet |

Consequently `f::T` denotes `Val2(T)[f]` in all of

```lang
let A: type = f::T;
let B = (f::T) meta_fn;
let g::(U |> (type ref)) = f::T;
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
T is not a Symbol alias.
T has fresh place identity.
T may evaluate to an existing type value.
```

`T` is a new binding with its own fresh, current-level writable place. Its *type
value* is the value read through `uint8`, while its *place* is its own. Binding to an existing
type value does not generate a new type, and it does not forward to `uint8`'s
Symbol or place.

This ordinary declaration rule does not license a meta returned result to use an
external pure Object as its installed type core. A canonical meta
instance has an additional self-root invariant, stated per result shape:
for the default result `τ_M`, `Root(Core(τ_M)) = MetaInstanceScope` holds
unconditionally (`Core(τ_M)` is the first projection of `τ_M`); an explicitly
declared result that carries an installed type core `Q` requires `Q`'s outer
Pattern root to be the `MetaInstanceScope` when `Q` is present. The condition is
`Q`'s presence, independent of
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

There is no declaration form that makes two bindings share one Symbol identity
or one place. Shared observation is expressed by the borrow constructors `ref`
and `share`; the privileged place-observation `@` yields a lifetime value
(`LifetimeVal`) and is not a borrow representation.

### 5.1 `ref` and `share` are privileged actual-place builtins

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

borrow-forming member:  runtime || compile
  E |> RefTy(T)
      -- forms the actual borrow instance; the runtime || compile
         builtin/default member, and the only family member that may
         obtain PrivilegedActualPlace
```

Only
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
symbol slot the value came out of, and does not consult, capture, or export
it. Therefore:

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

A `Symbol` constructor value is value-bearing, so `s ref` is the ordinary "form a borrow of
this value" operation. Because `Read` does not descend into `Val1`, `r` is a
`symbol ref` and **not** a reference to the member array held inside the
symbol. The referent is the value that `s` holds: `Target(s ref) =
PrivilegedActualPlace(s)` (§5.1.0) — the borrow target has one source:
`CarrierPlace(actual)`; there is no second `ObjectPlace(Read(actual))`.

When the intent is to form a borrow of the `Symbol`'s **type** rather than the
`Symbol` value itself — i.e. `(s |> type) ref` — an explicit `AsType` in a
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
    when TypeSlot(S) = Some(tau_S) and TypeValueRole(tau_S)

AsType(τ) = τ   iff TypeValueRole(τ)

WellFormedTau(τ) ∧ NamespaceOnly(τ)
  => AsType(τ) undefined
```

The second rule validates an existing **type-role** τ, not any well-formed
τ. It does not treat a bare pure namespace Object as a complete type, wrap a
namespace, or search for a hidden type member. A Symbol may carry any
`WellFormedTau` (§2.2): no τ (no type projection), a namespace-only τ
(`WellFormedTau(τ) ∧ NamespaceOnly(τ)`, for which `AsType` is undefined), or
a type-role τ (`TypeValueRole(τ)`). When a type-role τ is present, the
complete result is the whole `tau = <Q, V_τ>`, whose callspace `V_τ` was fixed
at formation (the direct TypeMember members of symbol-first §2.1). A copied
or extracted `tau` keeps its captured `V_τ`; members created under the same `Q`
later never enter an existing snapshot. Payload presence alone remains
irrelevant to type applicability. A language-designated type-expected position
may insert `AsType`; ordinary operand positions may not. See §5.6.

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

#### 5.1.3 The generated `ref` / `share` instance families

The borrow-forming defaults inside the formed borrow type's callspace are not
an ad hoc pair of builtins; they are generated instance families with a fixed
policy matrix. The `ref` family has two input shapes (`T`, `T ref`), two
member result-policies (`mut`, `const`), and three formal mutability patterns
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
| `Symbol` constructor value with `Val1` | `symbol ref` | no |
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

A closed-window type-valued slot may still be observed through `type ref`. If the view
is writable, the complete current value may be replaced by any compatible,
well-formed type value. What is forbidden is using the closed-window pointee as the
`old` input of `extend`; holding a reference does not change that value's
anchor or open window.

The holdable interval is consequently its ordinary borrow-valid region, not an
Open window. Weakening remains useful when write authority is unnecessary:

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
inject through a type ref   ->  OpenHere_Σ(Read(ref)) and Writable(Target(ref))
returning / storing a ref   ->  ordinary lifetime/capability escape check
```

Returning a `type ref` from a `compile` callable is therefore governed by the
same borrow escape rule as any other reference. It may remain usable after the
pointee's open window closes; a later `extend`/`inject` attempt rechecks the current value's
window state and may fail independently of the reference's validity.

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
OpenHere_Σ(v)
```

`Writable` is a place/borrow-capability question. `CanCreateMember` combines a
stable parent place with construction-unit, lexical, policy, and freshness
authority. `OpenHere_Σ` is an open-authority question used by structural
`extend`.
None is a spelling or proof of another:

```text
Writable_Γ(q)           does not imply OpenHere_Σ(Read(q))
OpenHere_Σ(v)           does not imply Writable_Γ(Carrier(v))
CanCreateMember_Γ(p, n) does not follow from Writable_Γ(p) alone
```

A writable slot may contain a closed-window type value that can be replaced wholesale
but cannot be structurally extended from. Conversely an open-window value may be
extended purely and bound elsewhere even when its source is immutable or has no
write-back place.

At minimum, ordinary place operations reject a core/external stable place, a
place reached only through `share`, a place outside its borrow lifetime, or a
place whose policy denies the action. Member creation additionally rejects a
parent outside the current construction unit or an already-instantiated child.
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
let f::(T |> (type ref)) = ...;
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

An ordinary Symbol's type core is installed at most once. Repeating:

```lang
let T = A;
let T = B;
```

as two competing core-installation definitions is a conflict, not implicit `A | B`.
For `struct`, the installed member additionally satisfies `TypeRole`. Child
construction and sum construction require explicit APIs and remain distinct
from repeated ordinary binding.

### 7.1 `let` creates a member; `=` writes an existing target

The two forms are distinct operations on the same resolved prospective target:

```text
let f::(T |> (type ref)) = expr   — instantiate a missing associated member
f::(T |> (type ref)) = expr       — write an already existing member
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
let f::(T |> (type ref)) = expr -> Write(slot, <Q',V_τ>); P and V_τ unchanged (§2.2)
extend(TypeValue(T), Δ)      -> new tau' = <Q',V_τ'> snapshot
inject(T |> (type ref), Δ)   -> read + extend + write through the type ref
```

An ordinary member declaration adds a `Val2` entry under an existing pattern
name; it does not widen `P(T)`. Widening the host pattern with a new child
pattern is exactly what `extend` does; `inject` is its place-level wrapper. Both
are specified in
`symbol-first-meta-construction-and-pattern-injection.md` §8. Both are limited to
extending the current parent pattern with a *direct* child; neither reaches into
a grandchild pattern.

Assignment carries no `extend`-specific construction-authority check, but that is
not the same as carrying no check. The canonical four-layer assignment model in
`symbol-first-meta-construction-and-pattern-injection.md` §4.5.1 governs, and the
text below only spells out its layer 2 for this document.

Layer 2 — universal write applicability checks exactly:

```text
Writable(lhs)              — the left side names a writable place
Compatible(P(lhs), rhs)    — the right value conforms to the target's Pattern
ValidCapability(lhs)       — lifetime and capability conditions of lhs hold
```

Assignment performs no `extend`-specific construction-authority check: there is no
requirement that the RHS came from a particular producer. That freedom covers
layer 1 (the RHS operation, including `extend`, discharged its own Open check)
and nothing more. Layers 1, 3,
and 4 of the canonical model — RHS operation legality, result-object invariants
(`WellFounded` / `Canonicalizable` / `NoForbiddenCycle`), and the enclosing
region's semantic-boundary constraints (meta self-root, ref / pattern-value
lifetimes, global type-bearing mutability limits, seal /
global promotion, single-τ-installation bound) — remain independently applicable to
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
values. Extraction resolves a source Symbol, reads its `PatternValue`, and
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
the independent writability and construction-authority (`OpenHere_Σ` / `WindowLive_Σ`) judgments of §6
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
  scopes, `struct`, pure `extend`, place-level `inject`, open-authority
  `OpenHere_Σ`, and the
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
