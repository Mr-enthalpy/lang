# Closure Anchored Replication

Status: canonical closure capability; source consumer pending.

## 1. Membership and immutable identity

For T = <tau, V_tau>, a contributed closure v must ultimately satisfy
TypeOf(v) in tau. A previously formed closure has its own anonymous type and
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
    TypeOf(c') in tau_target

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

    AnchorFor(v, tau) = v
      if TypeOf(v) in tau
    AnchorFor(v, tau) = InstantiateUnder(v, tau)
      if the first case does not apply and ReplicableUnder(v, tau)
    otherwise: failure

Type addition requires the actual target's Writable, OpenHere(T), and final
TypeOf(AnchorFor(v, tau)) in tau. Its write changes only V_tau. Replication does
not authorize structural changes to the target Core; those remain extend/inject
operations. A failed contribution does not retry a sealed overload candidate.

A closure RHS can therefore be evaluated at its ordinary lexical anchor before
a later contribution creates the target-anchored instance. It need not know its
future LHS or receive semantic information backward through normalization.
The same capability is usable by ordinary type contribution, inject, and the
construction logic obtained through A[t]. Assignment retains its own ordinary
semantics; this relation is not an initialization-only exception.

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
