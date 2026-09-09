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

    HasName_Sigma(r, n)
    Fresh_Sigma(r, n) iff not HasName_Sigma(r, n)
    typed structural name -> typed Place, initially uninitialized
    ordinary Val2 member name -> resident of any ordinary type
    ordinary name -> binding with an ordinary resident

NameBinding is binding identity and its relation to a resident Place;
it is not a first-class Object, constructor value, or borrowable wrapper. It has
no implicit .type field. Resolving a name selects its binding; a value read reads
the ordinary resident, and a borrow addresses that resident's actual Place.
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

A name with no callspace contributions exists even when it yields no call
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
requires Writable and OpenHere. Structural Core changes remain the work of
extend/inject. Complete values remain immutable snapshots: a successful write
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
snapshot. V_T registers values from the same ordinary Val2; it is not a second
value store. Core contains that Val2 observation. Consequently, keeping Core
fixed also keeps those residents fixed.

For an existing resident f with Home(TypeOf(f)) = TypeMemberScope(T), adding its callability
registration can produce a well-formed new V_T without changing Core. It does
not automatically add Pattern registration. Conversely, if v' is not a resident
of that Val2, classifier home alone does not make TypeAdd legal: the
proposed result would violate the joint Val2/registration consistency law.

    Core(T') = Core(T)
    V_T' = V_T + v'
    WellFormedTau(T') and joint Val2/role consistency
    ------------------------------------------------
    no new Val2 resident is created by this TypeAdd

Required new resident or anchored-instance formation belongs to one-shot
formation for a first resident, or extend/inject for an existing one (§6.1). That full formation already
includes its contribution once; no extra += is implied. Likewise, -= removes
the selected callability contribution only: it does not delete the ordinary
resident or its independent Pattern registration. These are consequences of
the existing update domain and result invariant, not new operation primitives.

## 5. Structural let creates a typed NameExpr

    P let name::path : t  evaluates to NameExpr(n)
    P let name::path      == P let name::path : type
    NameExpr != ValueExpr

The parent is reached through existing authorized structural navigation from a
mut type ref. Intermediate parents must exist and be initialized/navigable;
only the final selector is fresh. Creation checks the existing Writable,
OpenHere, access, lifetime and construction-authority premises of the parent.

    Fresh(parent, n)
    t is the declared Place type
    ----------------------------------------------
    CreateName(parent, n, P, t):
      establish HasName(parent, n)
      BindingPlace(n) = q_n
      DeclaredPolicy(n) = P
      PlaceType(q_n) = t
      ResidentState(q_n) = Uninitialized
      yield NameExpr(n)

Uninitialized is evaluator/Place state, not an Object, None value, or
fresh-name value. Creation installs no readable resident and creates no
callability or Pattern registration. The declared t may be any ordinary type.
Omitting :t chooses PlaceType = type, not an already constructed type value.

    NameExpr(n) in value context -> Read(q_n)
      succeeds only when ResidentState(q_n) = Initialized(v)
    NameExpr(n) ref -> Borrow(q_n)
      uses PlaceType(q_n), without first reading a resident

Borrowing remains explicit and checks the selected operation, declared policy,
actual Place, access and lifetime. Creation does not itself return a reference
or grant a mut view independent of those checks.

    absent name: no binding/Place
    existing uninitialized name: binding/typed Place, no readable value
    existing initialized name: binding/typed Place with resident v

The ordinary sequence is:

```lang
mut let name_ref = (P let name::path : t) ref;
name_ref = expr;
```

    CreateName -> BorrowPlace -> Initialize

Initialization belongs to ordinary Place/write algebra. With an admitted
write operation, Writable and the ordinary type/access/lifetime premises:

    PlaceType(q) = t, v : t
    Uninitialized -- Write(v) --> Initialized(v)

Later writes replace the existing resident under the ordinary replacement and
same-Type rules. Failed write Pre does not initialize the target; an enclosing
transaction governs rollback, if any. There is no old resident to drop or move
during first initialization. Value reads of an uninitialized target fail.

Structural `P let name::path = e` is not a canonical compound expression,
and there is no equality with `(P let name::path) = e`. Any future convenience
must be pure sugar for the explicit creation/borrow/write sequence and cannot
own a separate initialization rule.

    NamedType(n) iff DeclaredType(n) = type
                    and Resident(n) = T : type

Named type is an initialized case of a general structural name, not a distinct
name-creation primitive.

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

    Close(T) requires
      every n in ExternallyResolvableNames(T) has an initialized resident

    while open: HasName(T,n) may hold while n is absent from dom(Val2(T))
    after successful Close:
      ExternallyResolvableNames(T) = dom(Val2(T))
      for ordinary initialized structural members in the same name view

HasName is structural occupancy; Val2 records actual ordinary values.
An uninitialized name cannot be silently discarded, filled with a dummy type,
or hidden by a consumer filter to pass Close. Visibility rules remain ordinary;
the equality compares the corresponding structural member domain, not intrinsic
ordinal selectors or a visibility-filtered singleton count. The closed-type
only_val2 helper therefore still counts actual initialized Val2 entries.

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
    n_f = CreateName(parent, f, P, type)
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

## 7. Identity and closure

Name binding, entry identity, value equality, complete type, Place and lookup
index remain distinct. Copying a pattern value preserves its anchor and does
not open a new construction window. OpenHere uses the existing Core, anchor,
WindowLive and evaluation-stack judgments independently of carrier writability.

Publishing a closed construction result requires its externally resolvable
structural names to be initialized. Open construction navigation continues to
obey its ordinary access and authority rules. Anonymous implementation layers remain under /tau.
Neither physical files nor group aggregation grant target construction authority.
