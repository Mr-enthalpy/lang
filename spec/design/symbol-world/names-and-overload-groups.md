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
    Name -> type

NameBinding is structural binding identity and its relation to a resident Place;
it is not a first-class Object, constructor value, or borrowable wrapper. It has
no implicit .type field. Resolving a name selects its binding; a value read reads
the ordinary resident, and a borrow addresses that resident's actual Place.
Lexical aliases map to the same binding without becoming values themselves.

A structural name denotes a named type T. Same-name contributions synthesize
that type and its V_T, not an OverloadGroup at the name position. Occupancy is
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
key is Core(T): Bucket(T) = Core(T). The semantic candidate domain and current
aggregation laws are closed; carrier/entry encoding remains open. Any future
bucket-law refinement is a semantic change, not an encoding choice.
Applicable bucket combination may combine equal-bucket candidates.
This does not mutate either input type. Group update therefore requires
Writable(g), not OpenHere of its contained types. Distinct contribution entries
must not be erased by an unrelated value-interning or cache equality shortcut;
only the specified bucket/update relation decides aggregation.

For a mutable type reference t:

    TypeAdd(T, v):
      bind alpha.<Core(T), V_T[alpha]>
        -> bind alpha.<Core(T), (V_T + v')[alpha]>
      v' = AnchorFor(v, Core(T))
      Writable(t) and OpenHere(T) and TypeOf(v') in Core(T)

Only eligible closure-like member values enter this operation. It changes
V_T, never Core(T). Type subtraction likewise changes only V_T and
requires Writable and OpenHere. Structural Core changes remain the work of
extend/inject. Complete values remain immutable snapshots: a successful write
replaces the value at the target, without changing an earlier copy.

AnchorFor returns v when TypeOf(v) already belongs to Core(T); otherwise it requires the
closure's ReinstantiationWitness and creates a new anchored instance. It never
mutates v's owner. See [closure replication](closure-anchored-replication.md).

Group += and type += share operator spelling, not one semantic operation.
Associated A[t] is a particular guarded group place whose own write condition
also depends on its key's OpenHere; ordinary group references have no such
key-derived requirement.

## 5. Fresh-name creation returns a construction reference

Only the final selector may be fresh. Every intermediate parent in a multi-
segment path must already exist and be navigable under its ordinary view and
borrow permissions. Resolving a missing intermediate parent fails before
FreshNamedType; structural let does not create directories of missing parents
or infer their policy. Explicit successive formation can create such a chain.

Given a structural target reached from mut type ref under the existing
Writable, OpenHere, lifetime and construction authority premises:

    P let name::path : mut type ref
    DeclaredPolicy(name) = P
    Policy(result construction reference) = mut

The expression requires freshness and commits a complete initial resident before
returning its mutable construction reference. The formation rule is:

    Fresh_Sigma(r, n)
    Writable(r) and OpenHere(Read(r)) and ConstructionAuthority(r, kappa)
    q_n = ProjectionSlot(Target(r), n)
    Q_0 = EmptyPattern at the ordinary resolved child navigation n::path
    T_0 = bind alpha.<Q_0, empty V_T0[alpha]>
    ---------------------------------------------------------------
    FreshNamedType(r, n, P):
      install NameBinding(n, q_n), DeclaredPolicy(n) = P
      begin the resident generation at q_n with Contents(q_n) = Some(T_0)
      return Ref(q_n) with mut construction policy

Q_0 has no contributed structural children or members; it is the ordinary empty
Pattern at the resolved navigation, not a missing value or a universal Pattern.
Its anchor and GenerationRegime are established by the existing construction
coordinate rules: the meta root with transparent in-place layers in meta,
and the owning ordinary coordinate with opaque in-place layers otherwise.
Its construction window begins live under that authority. These are ordinary
formation/window facts, not a new Object axis or a new window for a copied type.
The complete T_0 inherits those facts through Q_0 under the existing Core bridge.

    absent name: no binding/resident at the prospective child
    existing empty named type: binding exists and Contents(q_n) = Some(T_0)

Standalone structural let therefore returns a reference to an existing complete
empty named type. Fresh-name commit is the binding action that establishes this
resident; bare assignment does not implement an absence-to-presence transition.

DeclaredPolicy is independent of
ConstructionReferencePolicy, so a const name can still be initialized through
the construction reference. Omitted P uses the same policy-demand/default
rules as ordinary bare let.

    P let name::path = e == (P let name::path) = e

The left expression commits name creation with Some(T_0); the subsequent
ordinary assignment now has its required existing old resident. Its ordinary
same-Type, Writable, lifetime and other selected-operation checks still apply. No let-specific initialization, write or rollback protocol exists.
An enclosing meta transaction applies only under its existing rules. The
returned reference addresses an existing name, not a manipulable fresh value.

Value navigation observes existing values. Borrowed structural navigation
retains an actual Place and can identify the prospective creation target.

## 6. Positional synthesis and lexical let

Only a normalized named-contribution position with a structural construction
target gives unqualified let name = expression the implicit same-name synthesis
meaning. On the first contribution the construction uses FreshNamedType from section 5;
on subsequent contributions it uses the existing named type. These contributions
form its V_T under the type-update and anchoring rules. Any required Core
construction uses extend/inject before the final membership check; type += cannot
silently populate an empty Core. Compatible first-contribution formation at the
same structural target is joined under the named-contribution algebra, not by
merging independently allocated names or treating explicit fresh lets as updates. They do not first package each RHS into a separate type
and aggregate an OverloadGroup at the name.

Ordinary lexical let remains ordinary Pattern-directed binding; two same-spelled
lexical declarations do not automatically become overload contributions.
Explicit structural P let name::path retains its fresh-name precondition.

Assignment remains ordinary assignment even where a selected ordinary type
construction operation consumes a closure. This does not define a universal
equation between = and +=. In particular, structural `let f::path = closure`
does not itself request TypeAdd or AnchorFor; it presents the ordinary assignment
problem. The assignment-operation owner determines whether a legal selected
candidate realizes it using type construction/replication. Neither structural
let nor the witness supplies an implicit conversion or forbids such a candidate. The explicit
contribution derivation is in [closure replication](closure-anchored-replication.md).
Anchored replication is a closure capability usable
by type contribution and inject, not a special RHS rule for let.

### 6.1 First contribution from one-shot formation equivalence

The member role is fixed by the named-contribution position. Once e evaluates
to v, its construction material is that same ordinary member material with
the RHS value v supplied; there is no search for an arbitrary larger Pattern
that merely happens to admit TypeOf(v).

Let a be the target's already determined anchor, B_0 its empty initial
construction, and Delta_v that member material. The
[construction owner, section 7.6.1](symbol-first-meta-construction-and-pattern-injection.md)
fixes the complete result by equivalence to struct with the member present
from the beginning:

    v = Eval(e)
    t_f = FreshNamedType(r, f, P)            -- commits Some(T_0)
    (t_f, Delta_v) |> inject                 -- the sole actual Extend/write

    Postcondition on that successful operation:
      T_1 = Read(Target(t_f))
      T_1 equivalent_to S_a(B_0 ; Delta_v)
      Q_1 = Core(T_1)
      v_a = the anchored member formed inside that invocation
      TypeOf(v_a) in Q_1

T_1 denotes the result formed inside the single actual inject invocation.
There is no earlier computation of T_1 to transfer or reuse, no second witness
execution, and no additional formation token. S_a is a specification-side
comparison, not an executed second construction. It denotes the existing
one-shot formation relation with
the same role, policy, dependencies, captures and anchor. It is not a new
callable, a second semantic evaluator, or a replay of the source RHS.

This yields the factorization:

    FreshNamedType
      -> Core preparation from the corresponding struct formation
      -> TypeAdd of its anchored member plus ordinary generated-member closure
      -> inject's ordinary completed-snapshot write

The middle steps describe the formation of Extend's complete result. They do
not add an extra post-inject +=. Core is changed by the existing extend
relation; TypeAdd still changes only V_T. The contribution appears exactly
once, and later same-name contributions reuse the same law against the actual
base snapshot. They do not re-evaluate earlier RHS expressions or reconstruct
their captures.

The result is determined up to the existing canonical normalization and bound
identity-renaming laws whenever that ordinary member formation is defined.
Missing witness, invalid captures, construction conflicts or failed write
premises remain ordinary failures. Equivalence cannot supply authority or
silently broaden the member's construction role. Thus the earlier gap was a
missing reference/formation bridge, not a new v-to-arbitrary-Core inference
mechanism.

## 7. Identity and closure

Name binding, entry identity, value equality, complete type, Place and lookup
index remain distinct. Copying a pattern value preserves its anchor and does
not open a new construction window. OpenHere uses the existing Core, anchor,
WindowLive and evaluation-stack judgments independently of carrier writability.

A complete construction result is externally navigable only after its visible
name set is closed. Anonymous implementation layers remain under /tau.
Neither physical files nor group aggregation grant target construction authority.
