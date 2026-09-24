# Closure Anchored Replication

Status: canonical closure capability; source consumer pending.

## 1. Membership and immutable identity

For T = bind alpha.<Core(T), V_T[alpha]>, a contributed closure v must satisfy
Home(TypeOf(v)) = TypeMemberScope(T). A previously formed closure has its own anonymous type and
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
    Home(TypeOf(c')) = TypeMemberScope(T_target)

The original c remains unchanged. Replication constructs a new anchored instance
of the same logic; it is neither mutation nor move nor aliasing.

## 3. Realized dependencies and internal identities

The new instance preserves the already determined semantic dependency material,
using their ordinary copy/share/ref rules. It does not re-execute arbitrary
surrounding code or re-resolve outer names. A captured reference keeps its
referent; copying the closure does not duplicate that referent or extend its
lifetime.

The closure's internal anonymous identity graph is preserved up to consistent
anchor-renaming. Self references, nested closures and other internal /tau
references are renamed together. Changing one rendered path is insufficient.
External references are not reparented. All affected type, policy, capture and
lifecycle checks remain ordinary E projection checks.

## 4. Type contribution targets the complete implementation hierarchy

    TypeMemberScope(T) = /tau(T)
    AnchorFor(v, T) = v
      if Home(TypeOf(v)) = TypeMemberScope(T)
    AnchorFor(v, T) = InstantiateUnder(v, TypeMemberScope(T))
      otherwise, if ReplicableUnder(v, TypeMemberScope(T))
    otherwise: failure

Core equality does not select or equate implementation homes. The target is the
complete bound type, with its existing anonymous implementation identity.
Authorized snapshot updates preserve that home through the existing bound-owner
relation, not by reconstructing it from Core or hashing changing callspace data.

TypeAdd requires Writable, OpenHere(T), and a well-formed resulting snapshot.
For v' = AnchorFor(v,T), check separately:

    Home(TypeOf(v')) = TypeMemberScope(T)
    RegisteredCallability_T'(v')      -- authorized non-generative contribution

Pattern-role registration and named Val2 residency are independent. The callable
value needs no val::path resident; its anonymous classifier's /tau home does not
make the value navigable. A witnessed new instance still obeys ordinary formation,
capture and lifetime checks; TypeAdd neither changes Core nor grants a named
selector. Generated-name occurrences cannot supply either registration. See
[name/type algebra](names-and-overload-groups.md#41-typeadd-also-preserves-the-complete-results-well-formedness).

### 4.1 Creation, initialization, and later contribution

```lang
mut let f_ref = (let f::path : type) ref;
f_ref = complete_first_type;
```

The first expression creates a typed NameExpr then explicitly borrows its
uninitialized Place. The next ordinary write initializes it. The RHS must
satisfy the selected write operation and PlaceType; creation does not supply
a closure conversion or anchored replication. Structural let-with-assignment
is not a canonical compound expression.

In a named-contribution position, the first closure contribution instead uses
the existing one-shot formation to produce that complete first type, then
initializes once. After an initialized T exists, further member material uses
extend/inject and its complete-type anchoring relation. An explicit += can
change callability registration when the home, non-generative registration and result consistency
premises hold. Neither the first formation nor an inject is followed by an
additional implicit +=. Ordinary assignment candidates remain owned by the
assignment-operation owner; their existence is not inferred from a witness.

## 5. Meta and non-meta anchors

Within a meta invocation, the MetaInstance root is the unique stable anchor;
in-place navigation is transparent for authority. Local residents end with the
invocation unless ordinary owned transfer admits their resulting realization
into the result region. Global promotion requires global dependency stability;
a bounded result may retain valid input-derived opening sources. Neither cache
identity nor result completion extends an expired local resident. struct,
inject and closure anonymous construction share the same meta anchor rules.

Established non-meta type constructions retain their survival rules and opaque
in-place navigation levels. Universal closure-to-tau formation does not extend
those rules to all dependency-bearing closure results; their survival, move,
return/store/escape/promotion and bounded state are handed to the lifetime owner. Their stable identity cannot be retroactively
reparented. Replication makes a new instance under the requested anchor while
retaining the old one. Neither case adds a new owner kind or window rule.

## 6. Local open question

Which values beyond closure-expression-produced closures can demonstrate a
valid location-parametric ReinstantiationWitness? The initial domain is fixed;
generalization requires proof rather than assuming arbitrary values are
replicable. Concrete witness/template representation is implementation work.


Initial formation at a known authorized contribution site differs from rehosting
an already formed value. The unified [construction relation](symbol-first-meta-construction-and-pattern-injection.md)
does not remove this witness or rerun [] initializers. General [dependency](dependency-observation-and-realization.md)
checks remain active throughout; deferred lifetime refinement grants no exemption.
