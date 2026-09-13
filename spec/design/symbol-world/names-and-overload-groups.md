# Names, Named Types, and OverloadGroups

Status: canonical semantics. Source consumers are tracked in the roadmap.

## 1. Notation and existing observations

T and tau both denote complete pattern/type values throughout the canonical
owners. Q denotes a Core projection, never the complete type:

    T = bind alpha.<Core(T), V_T[alpha]>
    tau = bind alpha.<Core(tau), V_tau[alpha]>
    Q = Core(T)
    type : (1 type)
    OverloadGroup : type

V_T and V_tau name the callspace snapshot of the indicated complete type.
The two spellings use the same bound-closure notation and observations;
tau is never a local abbreviation for Core(T).
Ordinary type equality/keying still observes the canonical Core; designated
whole-snapshot positions observe the whole bound closure.

    Object = <Val1?, P, Val2>

## 2. Name existence and named types

For a valid semantic root r and selector s, the structural coordinate exists
independently of evaluation:

    NameCoord(r,s)
    NameCoord is not Object, Place, NameBinding resident or Val2 entry
    NameCoord(r,s) does not imply BindingPlace(r,s)
    NameCoord(r,s) does not imply s in dom(Val2(r))

Evaluation may retain/realize the coordinate:

    Retained_Sigma(r,s)
    Fresh_Sigma(r,s) iff not Retained_Sigma(r,s)
    HasName_Sigma(r,s) is the existing spelling for Retained_Sigma(r,s)
      -- realization/occupancy, never existence of NameCoord itself

A retained coordinate may have a typed Place; that Place may still be
Uninitialized. Only Initialized(v) contributes the ordinary Val2 entry v.
The coordinate space is total for legal roots/selectors without eagerly
allocating an infinite namespace or granting access. Raw strings do not
construct arbitrary semantic coordinates. Stable coordinate identity must not
be confused with a particular resident generation or saved borrow target.

NameBinding is binding identity and its relation to a resident Place;
it is not a first-class Object, constructor value, or borrowable wrapper. It has
no implicit .type field. Resolving a name selects its binding; a value read reads
the ordinary resident, and a borrow addresses the typed Place without requiring
that a first resident already exists.
Lexical aliases map to the same binding without becoming values themselves.

Meta invocation also constructs ordinary names. Their formation owner is the
invocation identity, and they are not structural children of input values.
The direct instance name denotes its instance type tau_M itself. Arbitrary
values and borrows reside in ordinary Val2, accessed through name::path.
P1 meta retains the instance under OpenHere; plain let closes it on completion.
Explicit borrowing uses the selected actual Place; it is not the direct result
of meta invocation. NameBinding gains no Object wrapper or extra value algebra.
[Meta invocation](../meta-invocation/meta-object-invocation-and-policy-reduction.md)
owns their identity, opening-source propagation, completion and cache laws.

An initialized structural name declared :type denotes a named type T. At named-contribution positions,
same-name contributions synthesize that type and its V_T, not an OverloadGroup at the name position. Occupancy is
a structural fact, separate from the content of the existing value. A hidden,
unexported or policy-filtered name still exists. Freshness uses authoritative
occupancy, not the current lookup view.

A retained name with no callspace contributions exists even when it yields no call
candidates. Optional storage can encode occupancy internally, but there is no
fresh-name value or language-level None-to-Some name operation. Empty Pattern
and empty OverloadGroup values likewise differ from absence.

## 3. Type and group call projection

OverloadGroup is the outer, ordinary first-class candidate aggregation algebra.
It embeds a type as a singleton:

    eta : type -> OverloadGroup
    eta(T) = {T}
    CallCandidates(T) = CallCandidates(V_T)
    CallCandidates(G) = disjoint_union over T in G of CallCandidates(T)

The reverse embedding is not automatic. Empty groups and types without call
candidates are valid. Callability is a use-site projection, not a class of
group. Candidate resolution uses the ordinary Pattern/Policy pipeline, unique
selection and no reopen.

Names resolve once. Bare lookup stops at the nearest same-spelled binding;
explicit navigation uses its written anchor. Empty or inapplicable projection
does not resume lookup. Complete values keep their own captured callspaces;
neither a source binding nor a group supplies a missing extra callspace.

## 4. Different update algebras

For an ordinary mutable group reference g:

    g += T   uses G + eta(T)
    g += G'  uses G + G'

Group combination aggregates by its bucket relation. The current coarse bucket
key is the complete type T, observed as its whole bound closure:

    Bucket(T) = WholeTypeObservation(T)
    BucketEq(T1, T2) iff Norm_type(T1) = Norm_type(T2)

Ordinary type equality still observes Core, but does not define this quotient.
Equal Core with different V_T snapshots gives different buckets and may expose
different candidates. WholeTypeObservation is the existing binder-aware whole
snapshot observation, not a new Object axis. Carrier/entry encoding remains
open; equal-bucket combination still preserves distinct contribution entries
unless the specified aggregation operation combines them.
This does not mutate either input type. Group update therefore requires
Writable(g), not OpenHere of its contained types. Distinct contribution entries
must not be erased by an unrelated value-interning or cache equality shortcut;
only the specified bucket/update relation decides aggregation.

For a mutable type reference t:

    TypeAdd(T, v):
      bind alpha.<Core(T), V_T[alpha]>
        -> bind alpha.<Core(T), (V_T + v')[alpha]>
      v' = AnchorFor(v, T)
      Writable(t) and OpenHere(T) and Home(TypeOf(v')) = TypeMemberScope(T)

Only eligible closure-like member values enter this operation. It changes
V_T, never Core(T). Type subtraction likewise changes only V_T and
requires Writable and OpenHere. Pattern-registered structural extension remains
the work of extend/inject. Ordinary name initialization/replacement can also
change the Val2 component of Core, without adding either role registration. Complete values remain immutable snapshots: a successful write
replaces the value at the target, without changing an earlier copy.

TypeMemberScope(T) denotes /tau(T), the complete bound type's implementation
hierarchy, not MemberScope(Core(T)). AnchorFor returns v when its classifier
already has that home; otherwise it requires the
closure's ReinstantiationWitness and creates a new anchored instance. It never
mutates v's owner. See [closure replication](closure-anchored-replication.md).

Group += and type += share operator spelling, not one semantic operation.
A's group resident is carried by an invocation-generated name whose opening
source follows its input dependency. Its write Pre checks that source under
ordinary name rules; group membership itself creates no opening requirement
on the candidate types.

### 4.1 TypeAdd also preserves the complete result's well-formedness

The displayed TypeAdd premises do not replace WellFormedTau of the resulting
snapshot. They also do not require v' to have a named, navigable Val2 resident.
V_T holds ordinary callable values under the complete type's callspace relation;
registration does not provide a val::path selector for those values. Their
anonymous classifiers have the required /tau(T) home. Classifier navigation and
value navigation are distinct facts.

    Core(T') = Core(T)
    V_T' = V_T + v'
    Home(TypeOf(v')) = TypeMemberScope(T)
    non-generative callability registration and WellFormedTau(T')
    ------------------------------------------------------------
    TypeAdd adds no named Val2 entry and needs no such entry as a premise

An eligible anchored replica can therefore enter V_T without first being bound
as val::path. Captures, identity, lifetime, Writable and OpenHere checks still
apply. A separate ordinary name may expose the same value, but this is an
independent residency/binding action, not a required callspace representation.

Ordinary name writes and generated name occurrences can establish Val2 residents
without either registration. A generated occurrence cannot supply V_T or Pattern
registration evidence. Non-generative one-shot/extend formation may establish
the roles requested by its material, once each. Type subtraction removes the
selected callability contribution only; it neither removes an independently
named resident nor its independent Pattern registration.

## 5. Typed name declarations and complete let bindings

    P let name : t        evaluates to NameExpr(n) at a fresh lexical destination
    P let name::path : t  evaluates to NameExpr(n)
    P let name::path      == P let name::path : type
    NameExpr != ValueExpr

The first two are typed name declarations without an initializer. They share
typed Place formation, explicit borrowing and ordinary initialization. Their
destination and creation authority differ: the unqualified form uses the
ordinary fresh lexical binding context, while the qualified form uses the
explicit structural target. Lexical creation does not require navigating a
parent mut type ref; structural creation retains those additional premises.

The parent is reached through existing authorized structural navigation from a
mut type ref in the qualified case. Intermediate parents must exist and be initialized/navigable;
only the final selector is fresh. Creation checks the existing Writable,
OpenHere, access, lifetime and construction-authority premises of the parent.

    Fresh(parent, n)
    t is the declared Place type
    ----------------------------------------------
    Realize(NameCoord(parent,n), P, t):
      establish Retained(parent,n)
      BindingPlace(n) = q_n
      DeclaredPolicy(n) = P
      PlaceType(q_n) = t
      ResidentState(q_n) = Uninitialized
      establish pending InitialInitializationAuthority(q_n) from authorized formation
      yield NameExpr(n)

Uninitialized is evaluator/Place state, not an Object, None value, or
fresh-name value. Creation installs no readable resident and creates no
callability or Pattern registration. The declared t may be any ordinary type.
Omitting :t chooses PlaceType = type, not an already constructed type value.

    NameExpr(n) in value context -> Read(q_n)
      succeeds only when ResidentState(q_n) = Initialized(v)
    NameExpr(n) ref -> Borrow(q_n)
      uses PlaceType(q_n), without first reading a resident

Borrowing remains explicit. For the uninitialized Place it uses the live pending
initialization authority, selected ordinary initial-borrow operation and actual
Place/access/lifetime checks. DeclaredPolicy controls the initialized name's
views, independently of that authority; const does not prevent first
initialization. Creation returns no reference or general replacement capability.

    unretained coordinate: NameCoord exists, no binding/Place
    existing uninitialized name: binding/typed Place, no readable value
    existing initialized name: binding/typed Place with resident v

The ordinary sequence is:

```lang
mut let name_ref = (P let name::path : t) ref;
name_ref = expr;
```

    RealizeNameCoord -> BorrowPlace -> Initialize

Initialization belongs to ordinary Place/write algebra. With an admitted
write operation and InitWriteLegal established by live initialization authority,
explicit borrow capability and ordinary access/construction/lifetime premises:

    PlaceType(q) = t, v : t
    Uninitialized -- Write(v) --> Initialized(v)

First write neither observes an old resident policy nor performs old-resident
compatibility or cleanup. Successful commit consumes the pending authority for
the Place, including all saved references. Later writes need ordinary replacement
capability and same-Type/resident compatibility; an initial reference grants none
of those merely by being saved. Failed write Pre does not initialize the target; an enclosing
transaction governs rollback, if any. There is no old resident to drop or move
during first initialization. Value reads of an uninitialized target fail.
The complete two-case Write derivation and the const-name example are in
[Place/write algebra](type-values-places-and-borrow-views.md#711-initialization-authority-and-the-two-write-cases).

Structural `P let name::path = e` is not a canonical compound expression,
and there is no equality with `(P let name::path) = e`. Any future convenience
must be pure sugar for the explicit creation/borrow/write sequence and cannot
own a separate initialization rule.

    NamedType(n) iff DeclaredType(n) = type
                    and Resident(n) = T : type

Named type is an initialized case of a general structural name, not a distinct
name-creation primitive.

An initializer changes which complete let form is present. In ordinary lexical
context, P let name = rhs is a whole let binding whose declared type is inferred
from its RHS under the existing inference rules. It is not a typed-name
declaration defaulted to type followed by a separate assignment. An explicit
annotation in P let name:t = rhs supplies the type constraint instead.

```lang
mut let local_ref = (const let local:uint8) ref;
local_ref = 1uint8;

const let inferred = 1uint8;
const let annotated:uint8 = 1uint8;
```

The first pair explicitly creates and initializes a typed local Place. The
last two lines are complete lexical bindings: both create initialized uint8
destinations using the ordinary binding/transfer rules. Their internal first
resident installation obeys the same Place initialization law; it is not an
implicit source-level ref or an assignment suffix appended to NameExpr.

Thus the default :type for the initializer-free structural expression
(P let name::path) does not participate in lexical RHS type inference. Only
the separately designated named-contribution position changes unqualified
let name = rhs into named-type synthesis (§6); ordinary lexical binding never
acquires that meaning merely by spelling.

For example, let r be an authorized construction reference and v:uint8:

```lang
mut let byte_ref = (mut let byte::r:uint8) ref;
byte_ref = v;
```

Between these statements, byte exists and a second creation fails freshness.
A value read of byte::r fails, but its explicit ref can already have type
uint8 ref because the Place's declared type is known. A write with an
incompatible type fails Pre and leaves it uninitialized. After the shown write,
byte::r reads v; later writes use replacement checks. This ordinary Val2
payload contributes neither a callability nor a Pattern registration.

### 5.1 Closure requires initialized structural names

At the Close continuation position, every retained structural name being
published must have an initialized resident. An uninitialized name cannot be
silently discarded, filled with a dummy type or hidden by a consumer filter.
HasName means Retained, not every legal NameCoord or every future generative
request. At any observation, actual named Val2 entries are precisely initialized
retained residents in that corresponding view.

Close ends the construction window and freezes the non-generative registered
structure: V_T callability and Pattern construction/extraction registrations.
It does not assert that all future ordinary Val2 realization is impossible.
The already established finite generative rules may answer later legal requested coordinates with
ordinary Val2 results; those occurrences add neither registration. See §7.1.
The only_val2 helper counts the actual entries at its observation position, not
all coordinates or possible future realizations.

## 6. Positional synthesis and lexical let

Only a named-contribution construction position gives unqualified
`let name = expression` same-name synthesis meaning. Ordinary lexical let
retains its ordinary binding/transfer rule. Typed structural name creation
retains freshness and does not itself register any member role.

### 6.1 First contribution forms the first resident directly

Once e evaluates to v, Delta_v is the same ordinary member material accepted
at that construction position, with the same policy, captures and dependencies.
The target navigation is fixed by the typed name construction. The existing
one-shot struct formation relation determines the complete first resident:

    v = Eval(e)
    n_f = Realize(NameCoord(parent,f), P, type)
    q_f = BindingPlace(n_f), ResidentState(q_f) = Uninitialized
    T_1 = OneShotFormation(Delta_v)
    r_f = explicit Borrow(q_f)
    Initialize(q_f, T_1)               -- one ordinary write

OneShotFormation is the existing struct/member formation at the resolved
construction coordinate, not a new primitive. It forms Core, ordinary Val2,
the complete /tau(T_1) implementation hierarchy and the explicitly requested
registrations together. No empty type resident or initial inject is required.
A failure before successful write leaves no readable first resident, subject
to ordinary transaction rules; Close cannot publish that uninitialized name.

For subsequent contributions, T_i = Read(q_f) exists. The existing
[one-shot/extend equivalence](symbol-first-meta-construction-and-pattern-injection.md)
determines Extend(T_i, Delta_v); inject is read + extend + write. TypeAdd is
its callability contribution step with the complete-type home, residency and
well-formedness checks of §4. Full formation contributes each entry once;
there is no extra post-inject += and no replay of earlier RHS captures.

### 6.2 Unordered siblings share the coordinate before realization

Two sibling contributions to f under r refer to the same NameCoord(r,f).
They do not allocate competing name identities:

    Contribution(NameCoord(r,f), Delta_1)
    Contribution(NameCoord(r,f), Delta_2)
      -> existing contribution/effect join at that coordinate

For a common snapshot without a retained f, the accepted joined material
determines one first complete resident by the same OneShotFormation relation
above. A sequential trace is one presentation of that formation, not a
requirement that one sibling wins a CreateName race. When a resident already
exists, the ordinary extension/contribution relation uses that snapshot.
Explicit mutually exclusive declarations may still conflict under realization
rules. Coordinate agreement supplies no write authority, no duplication of
first-initialization capability and no permission to collapse distinct entries.

## 7. Identity and closure

Name binding, entry identity, value equality, complete type, Place and lookup
index remain distinct. Copying a pattern value preserves its anchor and does
not open a new construction window. OpenHere uses the existing Core, anchor,
WindowLive and evaluation-stack judgments independently of carrier writability.

Publishing a closed construction result requires its externally resolvable
structural names to be initialized. Open construction navigation continues to
obey its ordinary access and authority rules. Anonymous implementation layers remain under /tau.
Neither physical files nor group aggregation grant target construction authority.

### 7.1 Generated Val2 after registered structure is closed

The generative declaration rule may realize an ordinary member on a closed T:

    GeneratedOccurrence(T,s,v) -> Val2_current(T)[s] = v
    no V_T registration from this occurrence
    no Pattern registration from this occurrence

Here current denotes the ordinary evaluator observation, not an extra Object
axis or a private cache. A previously copied complete snapshot stays unchanged;
if realization changes current Val2, its new Core/Norm observation is visible to
E. Close is not a proof of constant Norm(Core) across such effects. E facts and
only_val2 counts remain tied to their snapshot/continuation position.

This is the selected generative rule's result realization, not permission to
obtain a mut construction view of closed T, perform arbitrary structural let,
inject a Pattern extension or update V_T. Its ordinary name/result formation,
access, dependency, Place and lifecycle checks still apply. A saved construction
ref remains closed. Generation cannot replace an existing registered witness
and thereby alter the frozen Pattern structure.

The distinction is semantic occurrence, not implementation provenance: a struct
operation may mechanically produce helpers as part of its non-generative
registered formation. Conversely, a requested-name generative occurrence cannot
register its result even when that result has a correctly anchored classifier.
The same value may be registered by another authorized non-generative occurrence.

For example, let closed T already carry a rule for requested selector s:

    before request: NameCoord(T,s), not Retained(T,s)
    selected rule realizes v at s -> current Val2(T)[s] = v
    after request:  s::T reads v, subject to ordinary access
    Pattern registrations and V_T are unchanged

Even if v is callable and its classifier has the right home, this occurrence
cannot add v to V_T or make s a Pattern child. Conversely, while a target is open,
an authorized non-generative TypeAdd of an eligible f need not first create
f::T. The classifier's /tau path still does not name the f value. These two
examples exercise different independent relations, not two kinds of Object.
