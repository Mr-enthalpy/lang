# Closure Anchored Replication

Status: canonical closure capability; source consumer pending.

## 1. Membership and immutable identity

For T = bind alpha.<Core(T), V_T[alpha]>, a contributed closure v must satisfy
TypeOf(v) in Core(T). A previously formed closure has its own anonymous type and
owner. Writing into another target cannot change that existing owner, rewrite
its identity, move it under another parent, or reinterpret it as an alias.

The canonical initial domain is closure-expression-produced closures with a
location-parametric construction witness. This is not a capability of arbitrary
values.

## 2. Reinstantiation witness

    ReinstantiationWitness(c): exists F_c
    c = F_c(a0)
    InstantiateUnder(c, a1) = F_c(a1) = c'

The witness is semantic evidence, not a source AST value. An implementation can
retain a template or IR to realize it without exposing that carrier as a new
language object class.

For a genuinely different anchor:

    Logic(c') equivalent_to Logic(c)
    c' != c
    TypeOf(c') != TypeOf(c)
    TypeOf(c') in Core(T_target)

The original c remains unchanged. Replication constructs a new anchored instance
of the same logic; it is neither mutation nor move nor aliasing.

## 3. Captures and internal identities

The new instance preserves the already determined capture semantic values,
using their ordinary copy/share/ref rules. It does not re-execute arbitrary
surrounding code or re-resolve outer names. A captured reference keeps its
referent; copying the closure does not duplicate that referent or extend its
lifetime.

The closure's internal anonymous identity graph is preserved up to consistent
anchor-renaming. Self references, nested closures and other internal /tau
references are renamed together. Changing one rendered path is insufficient.
External references are not reparented. All affected type, policy, capture and
lifecycle checks remain ordinary E projection checks.

## 4. Type contribution

    Q = Core(T)
    AnchorFor(v, Q) = v
      if TypeOf(v) in Q
    AnchorFor(v, Q) = InstantiateUnder(v, Q)
      if the first case does not apply and ReplicableUnder(v, Q)
    otherwise: failure

Type addition requires the actual target's Writable, OpenHere(T), and final
TypeOf(AnchorFor(v, Core(T))) in Core(T). Its write changes only V_T. Replication does
not authorize structural changes to the target Core; those remain extend/inject
operations. A failed contribution does not retry a sealed overload candidate.

A closure RHS can therefore be evaluated at its ordinary lexical anchor before
a later contribution creates the target-anchored instance. It need not know its
future LHS or receive semantic information backward through normalization.
The same capability is usable by ordinary type contribution, inject, and the
construction logic obtained through A[t]. Assignment retains its own ordinary
semantics; this relation is not an initialization-only exception.

### 4.1 Source forms and the operation that triggers replication

The following structural form does not by itself imply this capability:

```lang
let f::path = (self) => {};
```

Its expansion is ordinary `(let f::path) = closure`: FreshNamedType commits
T_0 and returns mut type ref, then presents the ordinary assignment problem.
Structural let adds no hidden TypeAdd/AnchorFor sugar. Whether an ordinary
assignment candidate can realize this operation using existing type construction
or anchored replication belongs to the [assignment-operation owner](symbol-first-meta-construction-and-pattern-injection.md),
section 4.5.1. This closure capability neither supplies such a candidate nor
forbids it. The universal same-Type replacement family alone does not establish
its existence; only a legal, uniquely selected ordinary candidate can do so.
If none applies, ordinary assignment fails under the existing transaction rules.
If one applies, its realization must satisfy all ordinary membership, OpenHere,
Writable, lifetime and no-reopen obligations.

An explicit contribution has the following schematic source/semantic derivation:

```lang
let c = (self) => {};
let t_f = let f::path;
// Prepare the required target Core through ordinary extend/inject.
t_f += c;
```

The creation line denotes binding the result of the structural expression;
its parser consumer remains pending. The preparation line is an explicit
premise, not work silently performed by +=. With T = Read(Target(t_f)):

```text
Writable(t_f) and OpenHere(T)
Core(T) already admits the required target-anchored closure type
c_f = AnchorFor(c, Core(T))
TypeOf(c_f) in Core(T)
-------------------------------------------------------------
ordinary TypeAdd commit:
  bind alpha.<Core(T), V_T[alpha]>
    -> bind alpha.<Core(T), (V_T + c_f)[alpha]>
```

If the original type already belongs, c_f = c; otherwise the witness constructs
a new anchored instance, then final membership and the ordinary write Pre are
checked. With a fresh empty Core and no preparation satisfying membership,
the addition fails; it never expands Core by itself.

An unqualified `let f = c` in a declared named-contribution position uses that
position's formation/Core-construction/contribution rules and reaches the same
TypeAdd relation as a formation step in Extend's complete result, by the
[name owner's one-shot equivalence derivation](names-and-overload-groups.md).
An inject of that full member material already installs the contribution once;
it is not followed by an additional implicit +=. An ordinary lexical let or structural
let's assignment suffix does not acquire named-contribution sugar. This does
not preclude the separately selected ordinary assignment realization from using
replication; candidate selection belongs to the assignment owner.

## 5. Meta and non-meta anchors

Within a meta invocation, the MetaInstance root is the unique stable anchor;
in-place navigation is transparent for authority. Local residents end with the
invocation, even when their resulting realization is returned. Seal promotes
the owned result realization, not the local resident's lifetime. struct,
inject and closure anonymous construction share the existing meta anchor rules.

Non-meta pattern values have the existing global-survival semantics and opaque
in-place navigation levels. Their stable identity cannot be retroactively
reparented. Replication makes a new instance under the requested anchor while
retaining the old one. Neither case adds a new owner kind or window rule.

## 6. Local open question

Which values beyond closure-expression-produced closures can demonstrate a
valid location-parametric ReinstantiationWitness? The initial domain is fixed;
generalization requires proof rather than assuming arbitrary values are
replicable. Concrete witness/template representation is implementation work.
