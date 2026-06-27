# Build System Design

**Status: Non-normative future design. Not a v0.1 parser rule.**

> This design is now the active v0.6 track: Build / Namespace Graph Bootstrap.
> The first vertical slice is implemented in `crates/lang_build`. The broader
> v0.6–v0.8 direction (NamespaceGraph Capability Layer, early meta-functions,
> type-to-type meta construction) is detailed in
> `spec/future/early-meta-functions-and-namespace-graph.md`.

The build system produces a **namespace graph world model** — a persistent,
diagnosable, eventually transactional object shared by all future phases
(resolver, early meta, type checker, policy, seal, IDE, cache). It is not a
temporary file index or a one-shot scan. For the engineering invariants that
govern this model (transaction discipline, conflict policy, symbol identity,
no-bypass, phase vocabulary, test philosophy), see
`spec/future/early-meta-functions-and-namespace-graph.md`
§"Namespace Graph World Model Invariants".

## Implementation note (v0.6 partial)

`crates/lang_build` now contains the first implementation slice of this design.
The public Rust API names currently match the design vocabulary:
`BuildManifest`, `CompilationWorld`, `NamespaceGraphSnapshot`,
`NamespaceDelta`, `NamespaceNode`, `SymbolObject`, `SourceCategory`,
`PolicyMetadata`, `VisibilityMetadata`, `Provenance`, `Diagnostic`,
`SyntaxObject`, and `MetaExpansionResult`.

The implemented manifest surface is API-level only: tests construct
`BuildManifest` values directly. The source collector mounts a package source
root under a namespace root, installs the compiler-seeded core package as a
default mount, supports synthetic dependency mount placeholders for explicit
mounted-path resolver tests, scans directories as physical namespace skeleton,
treats `.lang` files as source fragments under their directory namespace,
parses and normalizes those fragments with `lang_syntax`, and harvests only
direct top-level declarations needed by the vertical slice.

The implementation remains narrower than this design. It does not implement a
manifest file parser, dependency solving, remote package retrieval, lockfile
validation, overlays, access-control checking, source-level import syntax, or a
complete build CLI.

## 1. Scope

This document records the intended future direction for libraries, imports,
exports, filesystem layout, and namespaces.

It is not a v0.1 parser rule.

In v0.1, the parser only preserves syntax such as `ns1::ns2::symbol`. It does
not resolve namespaces, load libraries, process imports, check visibility,
assemble packages, or evaluate metaprogramming-generated namespace nodes.

## 2. Core principle

The language does not have source-level import/module/include syntax.

There is no source-level syntax such as `import mylib`, `use math::mylib`,
`include "x.lang"`, or `mod ns { ... }`.

A source file does not import libraries.

Instead, the build system assembles a namespace graph. Source code refers
directly to paths in that graph.

If the build system has mounted the required library namespace, source code can
access it naturally through its namespace path. If the build system has not
mounted it, path resolution fails later as a build/semantic resolution error.

The source language sees namespace paths, not packages, libraries, files,
static libraries, dynamic libraries, source packages, or cache entries.

## 3. Library import as namespace mount

Importing a third-party library is conceptually close to creating a symbolic
link into the namespace graph.

This is only an analogy. The implementation may use source packages, static
libraries, dynamic libraries, interface metadata, cached build artifacts,
remote packages, or other distribution forms.

The essential operation is: mount a library's namespace root into the
compilation namespace graph.

For example, a build manifest may say:

```
mount std from package std
mount mylib from package mylib
```

After this, source code may refer to `Vec::std` and `Vec3::vector::math::mylib`
without writing any source-level import.

The build system controls: which libraries are mounted, which namespace roots
they provide, which versions are selected, which artifacts are used, which
symbols are visible, which packages are trusted, which distribution forms are
allowed, and whether cached metadata is valid.

The source language does not control these through import statements.

## 4. Package layer versus language namespace layer

There are three layers, and they must not be collapsed into one another:

```text
package/build layer
namespace graph layer
source language layer
```

The **package/build layer** contains: package identity, library/application
distinction, source roots, dependency graph, version selection, namespace
mounts, access policy, distribution form, build cache, entry point.

The **namespace graph layer** contains: namespace nodes, symbol objects, mount
projections, core/default mounts, dependency mount markers, physical namespace
skeletons, generated virtual namespaces, symbol provenance, and policy /
visibility metadata.

The **source language layer** contains only namespace paths such as
`ns1::ns2::symbol`.

The package/build layer projects physical and virtual library contents into the
namespace graph. The source language layer only sees the resulting namespace
graph; it never sees packages, files, dependency resolution, caches, or
distribution forms directly.

### 4.1 Three semantic layers

The three layers carry distinct objects and distinct responsibilities.

The package/build layer owns build facts:

```text
PackageId
PackageKind
PackageRoot
PackageManifestObject
SourceRootObject
DependencyEdge
MountPath
ExportSurface
Feature/Config set
Build/cache fingerprint
Distribution form
Entry point metadata
```

These are not defined by ordinary `.lang` source files. Today they may be
constructed through the Rust build API; in the future they are provided by a
manifest.

The namespace graph layer owns resolvable, diagnosable, cacheable objects:

```text
namespace nodes
symbol objects
mount projections
core/default mounts
dependency mount markers
physical namespace skeletons
generated virtual namespaces
symbol provenance
policy/visibility metadata
```

The source language layer writes only paths:

```text
Vec::std
main::app
foo::bar::mylib
```

It does not write, and there is no plan for, source-level import machinery:

```text
import std
use std::Vec
mod foo
include "x.lang"
package mylib
```

The package/build layer projects build facts into the namespace graph; the
source language then resolves names only through that graph.

### 4.2 Build system as a semantic projection layer

The build system is not a temporary file scanner and not an external package
manager bolted onto the language. It is the producer of the namespace graph
world model. Its output is the semantic structure every later phase (resolver,
early meta, type checker, policy, cache, IDE) shares.

```text
Package/build facts are not directly visible to source code, but they determine
which namespace roots exist, which candidates are visible, which exports cross a
package boundary, and which provenance/fingerprint belongs to generated symbols.
```

Package/manifest facts therefore must be normalized into semantically
referenceable build objects, not parsed at the CLI layer and discarded.

### 4.3 Package identity and candidate identity

Formal meta object invocation needs stable candidate identity. A candidate
callable is not identified by name and argument shape alone. Its stable identity
needs, at least:

```text
symbol path inside namespace graph
package identity
mount path
source/generation provenance
policy/export metadata
manifest/config fingerprint
```

This identity participates in:

```text
diagnostics
cache invalidation
candidate comparison
cross-package visibility
incremental rebuild
future interface/binary distribution
```

This is a future design direction. None of these identity structures is claimed
to be implemented; the point is that `PackageId` and mount/provenance facts must
be available as semantic objects, because a path string alone cannot distinguish
two candidates that differ by package, mount, configuration, or provenance.

### 4.4 Internal versus external lookup boundary

A package boundary changes what a lookup may see:

```text
internal lookup  -> may see package-local symbols, including non-exported ones
external lookup  -> may only see symbols admitted by the export surface
```

Internal lookup within a package may resolve symbols that are not exported.
External lookup across a package boundary is restricted to the export surface.
The export surface is package/build metadata projected into the namespace graph
as a visibility boundary.

This is not a source-level import/use mechanism. Source does not import
packages. The manifest/mount table decides which namespace roots exist; the
export surface decides which symbols are visible across a package boundary. Both
are projections, not source clauses.

### 4.5 Core and dependency mounts

The package/build layer injects namespace roots into the graph in several forms:

```text
core/default mount        -> the compiler-seeded core package root
explicit dependency mount -> a declared dependency's namespace root
synthetic mount marker    -> an API/test placeholder root
future real package mount -> a manifest-provided dependency root
```

These are all ways the package/build layer contributes a root to the namespace
graph. The current implementation uses API-level / placeholder mounts; a future
manifest provides a formal mount table. The source language sees the resulting
roots, not the mount mechanism.

### 4.6 Generated namespace provenance

Meta-generated namespace nodes and symbols must also carry provenance back to
their originating package, root, manifest, and fingerprint. Without that
provenance, later meta object invocation, diagnostics, and caching cannot
explain where a candidate came from or when it must be invalidated. A generated
candidate is a build product, and its build origin is part of its identity.

## 5. Library, application, and distribution form

A package may be a library, application, plugin, test package, or another
future distribution unit. This distinction is build/package metadata, not
source syntax.

A library provides a namespace root. An application also provides a namespace
root, but additionally has an entry point.

Static distribution, dynamic distribution, source distribution, and
interface+binary distribution must not change the namespace path exposed to
source code.

Distribution form affects linking, loading, metadata availability, caching,
and verification. It does not affect how source names are written.

## 6. Directory structure and namespace structure

Filesystem directory paths provide a physical namespace skeleton.

However, the physical namespace skeleton is only a proper subset of the full
namespace graph: `filesystem directory path ⊂ namespace graph`, not
`filesystem directory path == namespace graph`.

Ordinary source layout should largely follow directory structure. The directory
path contributes namespace segments.

Implementation file names do **not** contribute namespace segments.

Implementation files are source fragments. They may be split, merged, renamed,
or generated without changing the external namespace API, as long as the
declarations contributed to the namespace remain compatible.

## 7. Namespace graph node kinds

The full namespace graph may contain several kinds of nodes:

- **Physical namespace nodes**: provided by package roots, source roots, and
  directory structure.
- **Declared namespace objects**: produced by language declarations such as
  `let ns1: namespace = ...`.
- **Virtual namespace nodes**: produced by instantiation, metaprogramming, or
  other future semantic mechanisms.

`namespace` remains meaningful as a source-level kind/rank name. Namespace is
not only a build-system concept.

The build system mounts physical namespace skeletons. The language and
metaprogramming system may extend the namespace graph with declared and virtual
namespace nodes.

### 7.1 Role-aware child names

The namespace graph no longer treats a textual child name as exactly one
symbol. A child name is role-aware:

```text
textual child name -> object/function role + namespace-subspace role
```

Same parent + same textual child name + same role is a hard conflict. An
object/function symbol without a namespace node may coexist with a pure
namespace-subspace symbol of the same textual name. This is required for
`struct` field functions named `ref` or `share`: the field is a unary function
object, while `ref` / `share` are projection namespace subspaces.

The conservative v0.6 restriction is that an object with a namespace node
(notably a type object with a type-associated namespace) may not coexist with a
namespace subspace of the same textual name in the same parent. That case would
make intermediate path traversal ambiguous before the resolver expectation API
is fully designed.

### 7.2 Type-associated namespace

A **type-associated namespace** is the namespace space associated with a type
object: generated field functions, `ref` / `share` projections, layout metadata,
pattern interfaces, and related companion symbols. It is a category by **role**,
not by origin: its members may be declared, generated, or virtual. For a
`struct`-generated type, it is a virtual / generated child namespace attached to
the type node. It is therefore not equivalent to the "declared namespace
objects" node kind alone.

See `spec/future/early-meta-functions-and-namespace-graph.md` §3 for the full
model.

## 8. Example: physical and virtual namespace layers

Consider `ns1::(int Vec::std)`:

- `std` may be the last filesystem-backed physical namespace layer.
- `Vec::std` is a declaration found inside that physical namespace.
- `(int Vec::std)` is an instantiated node. It does not correspond to a folder.
- `ns1::(int Vec::std)` is a virtual child namespace under the instantiated
  node. It also does not correspond to a folder.

Therefore, namespace resolution cannot be only filesystem path lookup.
Filesystem lookup provides the physical skeleton; semantic resolution may
continue into virtual namespace nodes.

## 9. Namespace contribution and injection rule

Declarations enter the namespace graph under a depth and context restriction.
The detailed model (rationale, diagnostic, examples) is in
`spec/future/early-meta-functions-and-namespace-graph.md` §4.

**No ordinary parent-to-descendant injection. Only ordinary
parent-to-direct-child contribution is allowed.**

1. **Ordinary physical/file contribution context**: a source fragment may
   contribute only the directly indexable children of its current namespace node
   (e.g. `f::ns`, `g::ns`). It must not inject into grandchildren or deeper
   descendants (e.g. `x::f::ns`, `y::x::f::ns`).
2. **Direct-child local construction**: a direct child object constructs its own
   internal / associated namespace; deeper structure is owned by the immediate
   parent object and built once by that object's local construction, not
   scattered across sibling files.
3. **Meta-function instantiation (exception)**: a closed instantiation may
   generate a parent-to-descendant virtual subtree, exposed as one virtual
   layer, because the instantiation is closed, globally consistent, and
   cacheable. Allowed but not encouraged; the generator bears the
   implementation / cache-key / diagnostic complexity. This exception does not
   apply to ordinary physical/file contribution.

In all contexts, a generated node may not inject into a parent, sibling, or
unrelated global namespace. The namespace graph grows downward from known
parents; metaprogramming cannot arbitrarily rewrite unrelated namespaces.

## 10. Export model

Export is not the source-level counterpart of import.

Since source code has no import mechanism, export should also not be designed
as a file-local import/export pair.

A source declaration introduces a symbol into the namespace contributed by its
source fragment. Whether those symbols are externally visible is a namespace
assembly / package metadata / visibility policy question.

Future designs may use package metadata, directory metadata, or source-level
visibility annotations. This note does not decide the final visibility syntax.

## 11. Dependency visibility

No source-level import does not mean all installed libraries are globally
visible.

The build system must provide an explicit dependency graph or namespace mount
table. Only mounted namespace roots are visible to a compilation.

If two packages attempt to provide the same namespace root, the build system
must resolve the conflict by policy. Possible future policies include: reject
duplicate namespace roots, allow explicit mount aliases, allow overlays only
when metadata permits, pin one version through lockfile.

The source language should not rely on ambient global installation state. A
build must be reproducible from package metadata and lock data.

## 12. Versioning and caching

Versions should not appear in normal source namespace paths.

Source code should normally write `Vec3::math::mylib`, not
`Vec3::math::mylib_1_2_0`. Version selection belongs to the build/package
layer.

A lockfile or equivalent build metadata may map `mylib -> mylib version 1.2.0`
but source paths remain stable.

For generated or instantiated namespace nodes, cache keys must include: source
package version, compiler version, feature/configuration set, instantiation
arguments, metaprogram inputs, visibility/export metadata, distribution form.

The cache may accelerate namespace graph construction, but it must not change
language-visible paths.

## 13. Access control

Access control is a resolver/package concern.

Possible future visibility categories include: public, package, private,
friend, platform-specific, feature-gated. These may be described by package
metadata, namespace metadata, or future source annotations.

The parser does not enforce access control. The namespace resolver decides
whether a resolved symbol is accessible.

## 14. Relationship to `namespace` in source

The source name `namespace` remains an ordinary `Name` token at the lexer
level. It may appear in declaration annotation position:
`let ns1: namespace = ...`.

This is not an import, a package mount, or a file inclusion rule. It declares
or describes a language-level namespace object, whose semantics are future work.

The reason to preserve `namespace` as a language-level kind/rank name is that
not all namespace nodes are filesystem-backed. Some may be declared,
instantiated, or generated by metaprogramming.

## 15. Relationship to v0.1

v0.1 must not implement package resolution, namespace resolution, import,
export, visibility, versioning, caching, or metaprogramming injection.

v0.1 should only preserve raw syntax such as `Vec::std`,
`Vec3::vector::math::mylib`, `ns1::(int Vec::std)` to the extent such syntax
is expressible by the raw AST rules.

The v0.1 parser must not introduce: `ImportDecl`, `UseDecl`, `IncludeDecl`,
`ModDecl`, `LibraryDecl`, `PackageDecl`, `ExportDecl`. No source-level
import/module syntax is specified.

## 16. Design tradeoff

This model sacrifices source-local explicit import lists. A source file alone
does not fully describe where all referenced libraries come from. Correct
interpretation requires package/build context.

It increases the importance and complexity of the build system, namespace
resolver, lockfile, metadata format, and IDE integration.

The benefits are: simpler source language, stable namespace paths across
distribution forms, free splitting/merging of implementation files,
centralized dependency/version/access control, clear separation between package
mechanics and language syntax, room for virtual namespace nodes, controlled
parent-to-child metaprogramming injection.

This tradeoff is acceptable if the language treats the build system and
namespace resolver as core infrastructure rather than optional external
tooling.

## 17. Summary

Libraries provide namespace roots. The build system mounts those roots into a
namespace graph. Directory paths provide a physical namespace skeleton.
Implementation file names do not create namespace segments. The physical
skeleton is a proper subset of the full namespace graph. The full graph may
contain declared and virtual namespace nodes. Source code has no
import/use/include/module syntax. Source code refers directly to mounted
namespace paths. Export and visibility are namespace assembly / resolver
concerns. v0.1 only preserves raw `::` path syntax.
