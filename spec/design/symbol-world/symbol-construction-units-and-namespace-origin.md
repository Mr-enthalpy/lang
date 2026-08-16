# Symbol Construction Units and Namespace Origin

**Status: Canonical future-design note for pure-role namespace origin,
construction-unit ownership, physical contribution authority, and cross-file
reopening. These rules are not implemented by the current v0.6–v0.9
substrate.**

This document owns the future rules that answer:

```text
who created a namespace/type/value subtree?
which construction transaction may continue building it?
which physical files have authority to contribute at a directory namespace?
when may independently produced deltas be combined?
```

The Symbol role/member, `compile` / `meta`, meta pure-role self-root, `struct`, pure
`extend`, and place-level `inject` semantics are canonical in
`symbol-first-meta-construction-and-pattern-injection.md`. The build projection
and assembly phases are described in
`../build-package/build-system-design.md` and
`../build-package/namespace-assembly-v0.md`. This note supplies the shared
construction-origin contract used by both tracks.

Policy pairs, binding `P1`, result `P2`, seal visibility, compile-flow projection, derived
compile companions, match staging, and automatic require are canonical in
`symbol-policy-and-compile-flow-projection.md`. Those flows may reuse material
owned by a construction unit, but they do not relax namespace-origin uniqueness
or cross-unit reopening rules.

## 1. One Construction Capability Substrate

Physical source assembly and ordinary meta invocation use the same symbol-world
construction capabilities:

```text
declare Symbol role/value material
extend a pure role member's navigable structure
construct a direct child
construct descendants owned by that new child
form a replayable contribution/delta
validate conflicts and authority
install a delta atomically at an outer binding/assembly boundary
```

They differ in the origin and owner of the construction, not in the identity
system of the resulting symbol graph:

```text
physical directory
  -> physical namespace construction scope

ordinary canonical meta invocation
  -> virtual symbol construction scope
```

An ordinary meta instance is therefore not merely “like a folder.” It is a real
virtual Symbol layer: it participates in navigation, may carry one optional
pure role member plus typed value members, anchors its returned role root, and is a candidate cache/
incremental unit.

Formal ordinary meta invocation produces uninstalled construction material.
Compiler-defined privileged AST operations such as `struct` and `extend` remain
graph-installation-free, but use their individually bounded ambient scope/owner
and current-unit capability rather than creating an ordinary meta instance.
`inject` may write one existing type slot but creates no graph member/root.
Physical assembly or an outer `let` performs `NamespaceDelta` creation,
validation, and installation.

## 2. Namespace Origin Is Unique

Every created pure role member / derived namespace projection records exactly
one creation origin:

```text
NamespaceOrigin =
    PhysicalDirectory(path)
  | SourceConstruction(source_construction_unit, construction_id)
  | MetaConstruction(meta_construction_unit, construction_id)
```

The hard invariant is:

```text
under one parent namespace/symbol scope,
one child namespace path has exactly one NamespaceOrigin
```

For a physical directory `ns/`, the child namespace `ns1::ns` may be created
by exactly one of:

1. the physical directory `ns/ns1/`;
2. one implementation file physically in `ns/`, as a source-owned direct-child
   construction that may include its complete new subtree (for example, a
   `struct` whose resolved top owner is `ns1::ns`);
3. one ordinary canonical meta invocation in `ns`, as a meta-owned virtual
   node.

The three sources are mutually exclusive creators of that role member/path.
This does **not** mean that a physical directory may contain only one
implementation file. Multiple files in `ns/` may create different direct
children of `ns`; they may not co-create or reopen the same child subtree.

Origin/provenance may remain attached for caching and diagnostics. It does not
become part of a resulting `PatternValue` identity.

## 3. Pure Objects Carry Namespace Role; Type Role Refines It

Type and namespace are judgments over the common Object domain, not facets or
parallel nominal Object kinds. The canonical definitions are owned by
`type-values-places-and-borrow-views.md`:

```text
Pure(x)          <=> Val1(x) = absent
WellFormedObject(x) => Navigable(Val2(x))
NamespaceRole(x) <=> Pure(x)
TypeRole(x)      =>  NamespaceRole(x)

TypeRole subset NamespaceRole
NamespaceRole not-subset TypeRole
```

`TypeRole` is an imported judgment; see
`type-values-places-and-borrow-views.md` §2.1. Its derivation belongs to the
subsequent P–Val1–Val2 relational-semantics design. This layer only consumes
it as an opaque predicate.

This is not type-system subtyping. It describes role implication:

```text
Pure(x)          => x is navigable and has NamespaceRole
TypeRole(Q)      => Q is a type-capable pure role core
NamespaceRole(x) and not TypeRole(x)
                 => x is navigable but unavailable to AsType

TypeRole(Q) and V_T = SelectTypeMembers(Q, V)
  => tau = <Q, V_T> is the complete type value,
     where V_T is the callspace captured at type-value formation:
     the direct TypeMember partition of the forming Symbol's V
     (TypeMember_Q at formation), not a global function of bare Q
```

A type-role Object's Pattern may contain pattern-material leaves. A
namespace-role-only Object may still contain ordinary navigable `Val2` members,
but it cannot form a complete type closure because `TypeRole(x)` does not hold.
`Q` is the Object core, while `AsType` returns the complete formation snapshot
`tau = <Q, V_T>` (`V_T = SelectTypeMembers(Q, V)`); it does not return bare `Q`
as complete type identity.

When one construction establishes `TypeRole(x)`, the Object and its navigable
`Val2` share one owned construction origin. An Object whose namespace role was
created by another origin cannot later acquire a new type-role definition. In
particular, if `ns1::ns` already comes from physical directory `ns/ns1/`, source
in the parent directory may not install a new type-role definition at
that Object: doing so would give one navigable construction two creation origins.

## 4. Construction Units

There are two closed semantic construction units:

```text
SourceConstructionUnit =
    one physical implementation file's closed source contribution

MetaConstructionUnit =
    one ordinary canonical meta invocation transaction
```

Stable implementations will assign identities such as
`SourceConstructionUnitId` and `MetaConstructionUnitId`; exact Rust storage is
not fixed here. An implementation filename does not enter the external
namespace path, but this does not yet imply that renaming a file preserves its
construction-unit identity, cache identity, or provenance. Such preservation
requires a future stable logical-file identity independent of the physical
path. At this stage only the namespace API is guaranteed to remain unchanged
by a filename-only rename.

When a unit creates a namespace/type/pattern child subtree, structural
construction ownership belongs to that unit:

```text
NamespaceConstructionOwner(N) = construction unit that created N
TypeConstructionOwner(T)      = construction unit that created T
PatternConstructionOwner(P)   = construction unit that created P's owner tree
```

One source file may create a new direct child and fully construct that child's
descendants inside its own delta. This is one closed construction, not
cross-file parent-to-descendant reopening. Once the delta is committed, a
parallel source file may not reopen the created subtree, even when the desired
new child name does not yet exist.

For example:

```text
a.lang creates T and T's type/pattern subtree
TypeConstructionOwner(T) = SourceConstructionUnit(a.lang)

b.lang attempts to add new child x under T
  -> conflict: b.lang does not own T's construction
```

This is stronger than duplicate-name detection. The absence of `x` does not
grant `b.lang` authority over `T`.

Same-origin cache replay may reuse identical material. A replay with different
material remains a hard conflict.

## 5. Physical Directory Contribution Authority

A physical directory creates both a namespace identity and a contribution
authority boundary:

```text
direct contents of a physical directory namespace
  may be contributed only by implementation files in that directory
```

If the filesystem contains:

```text
ns/
  ns1/
```

then an implementation file in `ns/` may navigate to and read `ns1::ns`, but it
may not contribute:

```text
x::ns1::ns
```

to reopen that physical child. Source that creates direct contents of
`ns1::ns` must be physically located in `ns/ns1/`.

The parent directory also may not create a source type at `ns1::ns`, because the
physical child already owns that namespace origin. The physical directory tree
therefore determines both namespace identity and contribution authority; it is
not merely a lookup hint.

Implementation filenames still do not create namespace path components. Two
files in one directory may create distinct direct children at that directory's
namespace level, but neither file receives authority to reopen a child created
by the other.

## 6. Current Cross-File Closure Rule

At the current specification stage, independently owned source constructions do
not merge into an existing symbol subtree:

```text
cross-file type child injection:          forbidden
cross-file namespace child injection:     forbidden
cross-file ordinary value member injection: forbidden
cross-file overload-entry merging:        forbidden
```

This preserves one construction owner and avoids introducing undefined partial
declarations, reopening syntax, cross-file visibility, diagnostic ownership,
or merge authority. The restriction is **not** a logical consequence of source
file contributions being unordered. Unordered value-entry union could be
well-defined later if stable candidate identity and conflict laws are supplied.

Possible future relaxations include:

- distinguishing declaration files from value/implementation contribution
  files;
- assigning special filenames explicit contribution roles;
- declaring a value/overload symbol `open` or `mergeable`;
- defining a commutative overload-entry union with stable entry identity.

Until such a design is adopted, ordinary value members and overload entries are
closed under the same cross-file ownership rule as namespace/type children.
Distinct direct child symbols created by distinct files remain legal; reopening
one symbol is not.

## 7. Ordinary Meta Construction Is One Transaction

An ordinary canonical meta invocation is exempt from the *cross-unit*
restriction for a precise reason:

```text
all symbol construction performed by that invocation
  belongs to one MetaConstructionUnit
  and one transaction
```

Within that transaction, the meta body may:

- chain pure `extend` operations for different children over PatternValues whose
  construction lineage is still Open in this `MetaConstructionUnit`, and use
  `inject` only when writing an existing local type slot is also required;
- construct a complete type/pattern subtree;
- establish multiple heterogeneous value entries;
- call `compile` helpers to obtain `PatternValue`s;
- combine other uninstalled construction material;
- return one ordinary Symbol PatternValue, for which the outer
  binding/assembly layer may form one `NamespaceDelta` candidate.

These operations are not cross-file or cross-construction-unit reopening.
They do not make arbitrary installed Symbol values structurally extendable. The
canonical `extend`/`inject` input and ownership preconditions are defined in
`symbol-first-meta-construction-and-pattern-injection.md`.

A helper ordinary-meta invocation with its own canonical instance has a separate
`MetaConstructionUnit`. The caller may compose the helper's returned,
uninstalled construction value according to explicit composition rules. It may
not directly mutate an already installed subtree owned by the helper instance.

The ordinary-meta return Symbol's pure-role self-root invariant follows from
this ownership: its optional `Q` is rooted at the invocation's
`MetaInstanceScope`, whether or not `TypeRole(Q)` holds. An external Object may
be a member under that root but cannot replace it. Compiler-defined privileged
AST operations `struct`, `extend`, and
`inject` use their separately specified scope/owner rule and do not acquire an
ordinary externally navigable instance root merely by being called.

## 8. Pattern Material and Namespace Values Are Orthogonal

A pattern-material leaf belongs to a pure role member's `PatternValue`. It
participates in:

```text
PatternValue identity
pattern normalization
ordered Product/layer or fully named Pattern-body navigation-map formation
pattern matching and extraction
type semantics when the owner additionally satisfies `TypeRole`
```

An ordinary namespace value member is a normal value-bearing Symbol under a
pure role member's namespace projection, analogous to a static member in some
languages. Adding one:

```text
changes namespace graph / value-member material
does not change PatternValue
does not change pattern normal form
does not enter a pattern set or ordered sequence
does not participate in pattern extraction
```

For example:

```text
val::ns1::ns2::ns3
```

navigates the pure role member's namespace projection to `val` and then reads
`val`'s value member. “Value
is a leaf” means only:

```text
value projection is terminal for the current value lookup
```

It does not mean that a Symbol with `V` members cannot also have a pure role
member `Q` and namespace children. Type projection derives the direct
TypeMember partition under `TypeRole(Q)` and forms `tau = <Q,V_T>`; it neither
stores another copy in the Symbol nor turns the closure into another Object.

## 9. Ordinary Pure-Role Installation Is Not Sum Extension

One Symbol place's pure role member may be installed once by ordinary definition:

```lang
let T = A;
let T = B;
```

If both forms attempt pure-role installation, the second conflicts. It does not
form `A | B`.

These remain separate operations:

```text
ordinary first pure-role installation
explicit child construction through extend (directly or through inject)
explicit sum construction / sum extension
```

The sum API's final spelling is undecided. Neither duplicate declaration nor
`extend`/`inject` implicitly turns an existing type/child into a sum. An explicit
read-transform-bind operation may be considered by later place/update rules,
but unrelated repeated definitions remain conflicts.

## 10. Ordinary Meta Invocation Navigation Atom

An ordinary canonical meta invocation is one navigation atom. If `Vec` is in
`std` and the argument is `int`, the correct form is:

```text
(int Vec::std)
```

Its child is:

```text
child::(int Vec::std)
```

Resolution first resolves `Vec::std`, then `int`, forms the canonical
invocation, and treats the whole parenthesized invocation as one navigable
symbol atom. These forms are invalid or semantically wrong:

```text
(int Vec)::std
int Vec::std
```

The future semantic grammar may record:

```text
MetaInstanceNavigationAtom :=
    '(' ArgumentProduct MetaCalleePath ')'
```

This note does not request a lexer/parser change.

## 11. Export Roots on Construction Levels

Each namespace construction level accepts `export` only on its direct
top-level declarations. The construction transaction records an export root;
it does not stamp a freely mutable flag on every descendant.

```text
ExportRetentionClosure(root) = PathAncestors(root) ∪ Subtree(root)
```

A child transaction cannot turn export off inside that subtree, while sibling
subtrees remain unaffected. `public`/`private` are independent ordinary
visibility attributes and may vary at every parent/child boundary. The
construction unit must preserve both facts so external traversal can require
both export-retention-closure membership and public reachability. Retention
closure membership is not itself external export status.

Private semantic dependencies may enter Wpre to keep an exported interface
interpretable without becoming externally name-visible. Construction therefore
maintains three independent views:

```text
Σ_full(N)    complete internal symbols and overloads
Σ_export(N)  identity-preserving external projection
Wfinal       Wpre ∪ Wseal world membership
```

Internal explicit navigation searches `Σ_full`; external explicit navigation
searches `Σ_export`. Wpre/Wseal membership neither grants nor denies export
visibility. An export descendant or ancestor may be externally exposed without
being the original export root, while a private dependency may belong to Wpre
without entering `Σ_export`.

The current typed helper now carries:

```text
ResolvedCandidatePolicy {
  pair: PolicyPair,
  provenance
}

ExportAdmission {
  in_export_retention_closure,
  publicly_reachable
}

ExportCandidateView {
  identity,
  internal_candidate,
  external_policy: PolicyPair
}
```

Declaration-side `P1Projection` is first applied to actual RHS/result entries.
Namespace external admission then requires both export-retention-closure
membership and public reachability through every path component. The
retention closure alone is not sufficient: a private child and public
descendants behind it remain internal. This admission is symbol-level and does
not act as an arbitrary per-candidate eligibility callback.

For each admitted symbol, the helper derives an external `PolicyPair` from
every resolved candidate pair that has a const value slice (or has
`Pv = absent`). Mut-only candidates remain in the full overload set and are
absent from the external overload set. A direct source `export + mut` root is
rejected earlier as an invalid declaration.

An absent value component is structurally empty:

```text
Pv = absent
  => value stages = ∅
  && value mutability = ∅
```

The projection helper reports an error when a flat compatibility carrier
violates this invariant; it does not silently pass the malformed pair through
the absent branch.

The helper no longer returns cloned internal policies as external views.
Full namespace-graph installation and external resolver routing remain later
integration work.

## 12. Current Implementation Substrate

The existing build slice already has physical directory skeleton collection,
`SymbolObject`, role-aware namespace nodes, transactional `NamespaceDelta`
installation, provenance slots, and a conservative direct-child harvesting
restriction. Those are useful prerequisites, not an implementation of this
document.

PR #94 remains a neutral `PatternHeadId` registry/materialization substrate.
Its generated/global/namespace/local contexts are transitional categorical
registry inputs for explicit low-level attachment and tests. Ordinary binding
preserves attached provisional material or restores stripped material through
the `GeneratedTypeDefinition` fallback; it does not derive a context from the
destination path. None of this establishes final namespace origin,
construction ownership, or meta return root identity.

Not implemented:

```text
meta result pure-role self-root checking
complete compile/meta language-level separation
MetaInstanceScopeId
ordinary canonical meta invocation navigation atom
NamespaceOrigin uniqueness checking
SourceConstructionUnit / MetaConstructionUnit ownership
physical directory contribution authority checking
cross-file reopening diagnostics
ordinary namespace value vs pattern-material role/cache implementation
Pure(x) => NamespaceRole(x) and TypeRole refinement enforcement
explicit sum construction/extension API
```

In particular, the current binding/materialization destination must not be
described as determining or rerooting a meta result role member's pattern
identity. Final meta role-root identity is anchored by the meta instance's own
symbol scope.

## 13. Non-Goals

This document does not:

- modify the lexer, parser, Raw AST, or Normalized AST;
- define source syntax for partial declarations or reopening;
- implement `compile`, `extend`, `inject`, or a sum API;
- define final overload-entry identity or future mergeable-value syntax;
- implement namespace-origin or construction-unit enforcement in Rust;
- turn physical files or internal AST carriers into a macro system.
