# Symbol Construction Units and Namespace Origin

**Status: Canonical future-design note for namespace-facet origin,
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

The symbol/facet, `compile` / `meta`, meta type self-root, `struct`, and
functional `inject` semantics are canonical in
`symbol-first-meta-construction-and-pattern-injection.md`. The build projection
and assembly phases are described in
`../build-package/build-system-design.md` and
`../build-package/namespace-assembly-v0.md`. This note supplies the shared
construction-origin contract used by both tracks.

Layered symbol policy, callable `P1` / `P2`, compile-flow projection, derived
compile companions, match staging, and automatic require are canonical in
`symbol-policy-and-compile-flow-projection.md`. Those flows may reuse material
owned by a construction unit, but they do not relax namespace-origin uniqueness
or cross-unit reopening rules.

## 1. One Construction Capability Substrate

Physical source assembly and ordinary meta invocation use the same symbol-world
construction capabilities:

```text
declare symbol/facet material
open a namespace facet
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
virtual symbol layer: it participates in navigation, may carry namespace/type/
value facets, anchors its returned type root, and is a candidate cache/
incremental unit.

Formal ordinary meta invocation produces uninstalled construction material.
Compiler-defined privileged AST meta functions such as `struct` and `inject`
also remain graph-installation-free, but use their individually bounded ambient
scope/owner and current-unit capability rather than creating an ordinary meta
instance. Physical assembly or an outer `let` binding/injection performs
`NamespaceDelta` validation and installation.

## 2. Namespace Origin Is Unique

Every namespace facet records exactly one creation origin:

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

The three sources are mutually exclusive creators of that namespace facet.
This does **not** mean that a physical directory may contain only one
implementation file. Multiple files in `ns/` may create different direct
children of `ns`; they may not co-create or reopen the same child subtree.

Origin/provenance may remain attached for caching and diagnostics. It does not
become part of a resulting `PatternValue` identity.

## 3. Type Structurally Includes Namespace

Type and namespace are facets of one symbol, with this structural inclusion:

```text
has TypeFacet => has NamespaceFacet
has NamespaceFacet ⇏ has TypeFacet
```

This is not type-system subtyping. It describes facet containment:

```text
TypeSymbol =
    NamespaceFacet
  + TypeFacet(PatternValue)
  + optional ValueFacet

PureNamespaceSymbol =
    NamespaceFacet
  + optional ValueFacet
  + no TypeFacet
```

A type's `PatternValue` may contain pattern-material leaves. A pure namespace
has no type `PatternValue` and therefore no pattern-material leaves, although it
may still contain ordinary namespace value members.

When one construction creates a type symbol, its namespace and type facets are
created as one owned construction. A namespace facet created by another origin
cannot later be upgraded into that type. In particular, if `ns1::ns` already
comes from physical directory `ns/ns1/`, source in the parent directory may not
install a `TypeFacet` at `ns1::ns`: doing so would make one namespace facet have
both physical and source creation origins.

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

- chain functional `inject` operations for different children through an
  `OpenOwnedConstructionHandle` whose owner is this `MetaConstructionUnit` and
  whose state remains open/uninstalled;
- construct a complete type/pattern subtree;
- establish multiple heterogeneous value entries;
- call `compile` helpers to obtain `PatternValue`s;
- combine other uninstalled construction material;
- return one atomic `SymbolConstructionValue`, from which the outer
  binding/assembly layer may form one `NamespaceDelta` candidate.

These operations are not cross-file or cross-construction-unit reopening.
They do not make arbitrary installed symbols injectable. The canonical
`inject` input and ownership preconditions are defined in
`symbol-first-meta-construction-and-pattern-injection.md`.

A helper ordinary-meta invocation with its own canonical instance has a separate
`MetaConstructionUnit`. The caller may compose the helper's returned,
uninstalled construction value according to explicit composition rules. It may
not directly mutate an already installed subtree owned by the helper instance.

The ordinary-meta return symbol's type self-root invariant follows from this
ownership: the invocation's `MetaInstanceScope` is the type root identity
anchor. An external type value may be a member under that root but cannot
replace it. Compiler-defined privileged AST meta functions such as `struct` and
`inject` use their separately specified scope/owner rule and do not acquire an
ordinary externally navigable instance root merely by being called.

## 8. Pattern Material and Namespace Values Are Orthogonal

A pattern-material leaf belongs to a type facet's `PatternValue`. It
participates in:

```text
PatternValue identity
pattern normalization
ordered sequence or Set<PatternValue> formation
pattern matching and extraction
type semantics
```

An ordinary namespace value member is a normal value-bearing symbol under a
namespace facet, analogous to a static member in some languages. Adding one:

```text
changes namespace graph / value-facet material
does not change PatternValue
does not change pattern normal form
does not enter a pattern set or ordered sequence
does not participate in pattern extraction
```

For example:

```text
val::ns1::ns2::ns3
```

navigates namespace facets to `val` and then reads `val`'s value facet. “Value
is a leaf” means only:

```text
value projection is terminal for the current value lookup
```

It does not mean that a `SymbolCell` with a value facet cannot also have a
namespace facet and namespace children. Namespace, type, and heterogeneous
value facets may coexist on one symbol.

## 9. Ordinary Type Installation Is Not Sum Extension

One symbol place's type facet may be installed once by ordinary definition:

```lang
let T = A;
let T = B;
```

If both forms attempt type-facet installation, the second conflicts. It does not
form `A | B`.

These remain separate operations:

```text
ordinary first type installation
explicit child construction through inject or an equivalent owned API
explicit sum construction / sum extension
```

The sum API's final spelling is undecided. Neither duplicate declaration nor
`inject` implicitly turns an existing type/child into a sum. An explicit
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

## 11. Current Implementation Substrate

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
meta result type self-root checking
complete compile/meta language-level separation
MetaInstanceScopeId
ordinary canonical meta invocation navigation atom
NamespaceOrigin uniqueness checking
SourceConstructionUnit / MetaConstructionUnit ownership
physical directory contribution authority checking
cross-file reopening diagnostics
ordinary namespace value vs pattern-material facet implementation
has TypeFacet => has NamespaceFacet structural enforcement
explicit sum construction/extension API
```

In particular, the current binding/materialization destination must not be
described as determining or rerooting a meta result type's pattern identity.
Final meta type identity is anchored by the meta instance's own symbol scope.

## 12. Non-Goals

This document does not:

- modify the lexer, parser, Raw AST, or Normalized AST;
- define source syntax for partial declarations or reopening;
- implement `compile`, `inject`, or a sum API;
- define final overload-entry identity or future mergeable-value syntax;
- implement namespace-origin or construction-unit enforcement in Rust;
- turn physical files or internal AST carriers into a macro system.
