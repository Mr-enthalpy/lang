# Names, Named Types, and OverloadGroups

Status: canonical semantics. Source consumers are tracked in the roadmap.

## 1. Notation and existing observations

In this owner T denotes the complete pattern/type value and tau its core:

    T = <tau, V_tau>
    Core(T) = tau
    type : (1 type)
    OverloadGroup : type

Older complete-type notation in other owners writes the complete value as
tau = bind alpha.<Core(tau), V_tau[alpha]>. These notations describe the same
two observations; they do not introduce a new coordinate or identity.
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
that type and its V_tau, not an OverloadGroup at the name position. Occupancy is
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
    CallCandidates(T) = CallCandidates(V_tau(T))
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
key is the type's core tau; finer bucket rules remain representation/algebra
detail. Applicable bucket combination may combine equal-bucket candidates.
This does not mutate either input type. Group update therefore requires
Writable(g), not OpenHere of its contained types. Distinct contribution entries
must not be erased by an unrelated value-interning or cache equality shortcut;
only the specified bucket/update relation decides aggregation.

For a mutable type reference t:

    TypeAdd(T, v):
      <tau, V_tau> -> <tau, V_tau + v'>
      v' = AnchorFor(v, tau)
      Writable(t) and OpenHere(T) and TypeOf(v') in tau

Only eligible closure-like member values enter this operation. It changes
V_tau, never tau/Core. Type subtraction likewise changes only V_tau and
requires Writable and OpenHere. Structural Core changes remain the work of
extend/inject. Complete values remain immutable snapshots: a successful write
replaces the value at the target, without changing an earlier copy.

AnchorFor returns v when it already belongs to tau; otherwise it requires the
closure's ReinstantiationWitness and creates a new anchored instance. It never
mutates v's owner. See [closure replication](closure-anchored-replication.md).

Group += and type += share operator spelling, not one semantic operation.
Associated A[t] is a particular guarded group place whose own write condition
also depends on its key's OpenHere; ordinary group references have no such
key-derived requirement.

## 5. Fresh-name creation returns a construction reference

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
    T_0 = bind alpha.<Q_0, empty V_tau[alpha]>
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
form its V_tau under the type-update and anchoring rules. Any required Core
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
equation between = and +=. Anchored replication is a closure capability usable
by type contribution and inject, not a special RHS rule for let.

## 7. Identity and closure

Name binding, entry identity, value equality, complete type, Place and lookup
index remain distinct. Copying a pattern value preserves its anchor and does
not open a new construction window. OpenHere uses the existing Core, anchor,
WindowLive and evaluation-stack judgments independently of carrier writability.

A complete construction result is externally navigable only after its visible
name set is closed. Anonymous implementation layers remain under /tau.
Neither physical files nor group aggregation grant target construction authority.
