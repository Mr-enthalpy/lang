# Associated Compile-Time State

Status: canonical semantics; source integration pending. A is notation, not a
frozen callable spelling. The names associated and meta_slot remain candidates.

## 1. Ordinary language-visible state

The meta-evaluation state includes a type-associated map of ordinary groups:

    A_Sigma : type -> Place(OverloadGroup)
    Read(A[t]) = epsilon_OG              in the absence of an explicit update

The indexed place family is conceptually total. Sparse storage introduces no new slot
freshness/existence semantics. This is language-visible compile-time state,
not compiler metadata, a package registry, or another Object axis. A is
currently builtin; that does not claim its guarded-place algebra is incapable
of a more general language formulation.

Ordinary type keying uses the existing canonical Core observation, as specified
in [pattern values](type-values-places-and-borrow-views.md#2-semantic-identities).
An implementation TypeValueId is only an index. A does not redefine equality,
introduce nominal identity, or require whole-snapshot keying by default.

A is exposed as an ordinary callable object with reference projections:

    A(type) -> OverloadGroup ref
    A(mut type ref) -> mut OverloadGroup ref

These signatures show the explicit argument and result; the callable's own
first self follows the ordinary invocation convention. Ordinary reference
assignment and the applicable group update algebra implement =, +=, and -=.

## 2. A bounded writable window

Let a_t be the associated reference obtained from a construction reference t_ref.

    Writable(a_t, kappa)
      iff OpenHere(Read(t_ref), kappa) and WriteAuthority(a_t, kappa)

The existing authority, borrow validity, and policy checks still apply. This
rule composes existing judgments; it adds no independent contribution capability.
OpenHere is determined by the pattern value, through Core, anchor, live window,
and the authority-frame rules in
[construction](symbol-first-meta-construction-and-pattern-injection.md#1211-open-authority-is-stack-relative).
Different carrier Places do not create independent windows for copies of that
value. Value equality itself grants no write authority.

True Close irreversibly clears the existing WindowLive fact. A saved reference
can retain static mut policy while its write Pre fails after Close. Temporary
stack masking is not Close and does not create a new window on return.

    GlobalVisibility does not imply GlobalMutability
    PolicyMode(mut) does not imply Writable

Static closure freezes the externally visible members and the associated state
against later extension. A remains globally addressable while its write window
is bounded by the existing construction semantics. This does not prevent an
ordinary writable carrier from being assigned another complete value under its
own rules, and does not reopen the closed value that it previously carried.

## 3. Construction logic is an ordinary callable

For construction use, the group's candidates expose complete compile function
objects. A selected function
accepts a target mutable type reference as an ordinary explicit argument. Its
body may inspect the target, branch through Pattern relations, call other
compile functions or host IO, create intermediate Objects, inject several
times, or do nothing. The group aggregates ordinary type candidates and their callable members, not a special delta or
an incomplete implementation descriptor.

The consuming construction invokes it normally:

    (mut let r::some_path) |> (t |> A)

The selected callable's first self is that callable object. The construction
reference r is an ordinary subsequent argument:

    Type(callee) = Type(first self)

Call projection and ordinary overload selection choose one candidate.
Indistinguishable matches produce ordinary ambiguity. There is no implicit
fan-out over all entries. A user-defined collection or dispatcher can itself be
an ordinary entry when sequential execution of several actions is desired.

The source and target obey independent existing checks:

    source side: A[t] may change only while OpenHere(t)
    target side: the chosen body may extend r only while OpenHere(r)

The receiver supplies its own target construction reference by calling the
group. The construction uses ordinary state and calling, with no additional
implementation authority. Actual injection is still read + extend +
write through that reference. Calling through A gives no hidden target access.

## 4. Ordinary effects in unordered blocks

A participates in the same evaluation state and effects as other mutable
compile-time Objects. Sibling blocks start from a common input snapshot and
produce independent overlays. Commutative, associative entry contributions can
join. Different conflicting replacements cannot join and report an ordinary
unordered-block write conflict. Whether subtraction commutes with another
update is determined by the ordinary update algebra, not by A or file order.

Reading another sibling's newly written state is not repaired by scheduling
that sibling first. A introduces no ordering exception. See
[source normalization](../build-package/build-system-design.md).

## 5. Representation boundary

The public spelling, sparse storage, bucket/entry identities, reference
encoding, and incremental indexing remain local implementation questions.
The existing pattern-value equality and OpenHere rules are not reopened by
those choices. Cache replay must preserve effects, entry multiplicity, and
current write Pre checks; it cannot make a saved mutable reference writable.


The guarded global indexed place family suggested by A is a local open question:
should key -> Place(value), with write permission dynamically derived from
facts about the key, become a general user-accessible algebraic capability?
For now the capability is restricted to builtin A; no IndexedPlace constructor
or additional capability type is introduced.

Ordinary OverloadGroup mutation needs its own Writable only and does not mutate
its candidate types. A's extra key-dependent guard belongs to this particular
place family. Type += instead changes V_tau under OpenHere and the anchored
closure membership rules of [name/type algebra](names-and-overload-groups.md).
