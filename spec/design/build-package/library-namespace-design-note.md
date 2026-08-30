# Library and Namespace Design Note

**Status: Canonical library/namespace design note. This is not parser syntax.**

See also `spec/design/build-package/build-system-design.md` for the current build-system
architecture document.

## 1. No source-level import/include/module syntax

The core source language has no source-level import, module, or include syntax.

There is no source-level syntax such as:

```text
import mylib
use math::mylib
include "x.lang"
mod ns { ... }
```

## 2. Source files do not import libraries

A source file does not import libraries. The build system assembles a namespace
graph. Source code refers directly to paths in that graph.

## 3. Library import is a build-layer namespace mount

Library import is conceptually a namespace mount performed by the build/package
layer. The source language sees namespace paths, not packages, libraries, files,
static libraries, dynamic libraries, source packages, or cache entries.

## 4. Package layer versus language namespace layer

The package/build layer and the language namespace layer are distinct.
Package managers, build systems, linkers, and dependency resolvers operate
in the package layer. The source language only queries and traverses the
assembled namespace graph. Source-level namespace paths are resolved against
that graph, not directly against packages or files.

## 5. Library/application/distribution form

How a library or application is distributed (source archive, object archive,
dynamic library, executable, bytecode container) does not affect source-level
namespace paths. Distribution form is a build-layer concern.

## 6. Directory structure and namespace structure

Filesystem directories provide a physical namespace skeleton only.
Implementation file names do not create namespace segments. A directory
layout such as:

```text
mylib/
  lang.pkg
  src/
    math/
      vector/
        impl.lang
        ops.lang
      matrix/
        impl.lang
```

may correspond to the namespace navigations `vector::math::mylib` and
`matrix::math::mylib`, but this mapping is performed by the build layer,
not the source language.

For example, both implementation files:

```text
src/math/vector/impl.lang
src/math/vector/ops.lang
```

contribute to:

```text
vector::math::mylib
```

They may create distinct direct children at that namespace level. Each file is
one closed `SourceConstructionUnit`; neither may reopen a namespace/type/
pattern/value subtree created by the other. Thus implementation filenames are
absent from source navigation, but file boundaries still matter for
construction ownership.

They do not create:

```text
impl::vector::math::mylib
ops::vector::math::mylib
```

Directory paths provide the physical namespace skeleton. Implementation
file names do not create namespace segments.

## 7. Namespace graph node kinds

The current graph/provenance model may record three namespace origin categories:

- **Physical namespace nodes**: contributed by filesystem skeleton, build
  descriptors, or package manifests.
- **Declared namespace objects**: created by `let ns: namespace = ...` at
  the language level.
- **Virtual namespace nodes**: created by canonical meta construction and
  installed by the namespace assembler. They are not tied to a physical source
  file; the resolver observes them but does not create their namespace origin.

In the final model these are navigable-construction origins, not mutually
exclusive symbol kinds. Every pure Object has `NamespaceRole`; `TypeRole(x)`
is an additional imported judgment, so it is a strict refinement.
Current `SymbolCell` facet buckets are implementation substrate only.

A navigable child name is role-aware: object/function symbols and pure
namespace subspaces may share the same textual name when resolver callers
provide an expected role. Same-role duplicates remain hard conflicts. See
`spec/design/build-package/build-system-design.md` §7 and
`spec/design/symbol-world/early-meta-functions-and-namespace-graph.md` §3.

## 8. Physical and virtual namespace layers

The physical filesystem skeleton is a proper subset of the full namespace
graph. The language may reference virtual namespace nodes that have no
corresponding filesystem directory.

## 9. `let ns1: namespace = ...` is a language-level declaration

`let ns1: namespace = ...` is a language-level namespace object declaration
or description, not a package mount or import. The source name `namespace`
is an ordinary `Name` token in the weak lexer.

## 10. Export model

Export is not the dual of import. Export is a namespace assembly, resolver,
or package metadata concern. A namespace object may be accessible through
multiple namespace paths. Visibility and re-export are namespace graph
organization decisions, not source-level syntax.

## 11. Dependency visibility

Dependency visibility (which libraries can see which other libraries)
is determined at the build/package layer. The source language receives
the assembled namespace graph and does not perform dependency visibility
checks.

## 12. Access control

Access control (public, private, restricted visibility) is a namespace
graph and resolver concern, not source-level syntax.

## 13. Namespace contribution and injection rule

Ordinary source fragments may contribute only the direct children of their
current physical directory namespace. One file may fully construct the new
direct-child subtree it creates, but a parallel file may not reopen it. An
ordinary canonical meta invocation may construct a complete virtual subtree
because all actions belong to one `MetaConstructionUnit` transaction.
Compiler-defined privileged AST meta functions use only their bounded
current-unit capability. In all contexts, generated nodes must not inject into
parents, siblings, unrelated globals, or subtrees owned by another construction
unit.

If `ns/ns1/` physically exists, only files inside `ns/ns1/` may create direct
contents of `ns1::ns`; parent files may navigate/read that child but cannot
inject into it or upgrade it to a source-created type. Cross-file type-child,
namespace-child, ordinary value-member, and overload-entry injection are
currently forbidden. See `spec/design/build-package/build-system-design.md` §9,
`spec/design/symbol-world/early-meta-functions-and-namespace-graph.md` §4, and
`spec/design/symbol-world/symbol-construction-units-and-namespace-origin.md`.

This is a meta-function / metaprogramming capability. It is not parser
and must not be assumed as general language semantics.

## 14. Versioning and caching

Versioning and caching must not appear in ordinary source namespace paths.
Version resolution and artifact caching are package-layer operations.

## 15. Relationship to `namespace` in source

The source name `namespace` is an ordinary `Name` token. It carries
no special lexical or parser status. It may appear in declaration annotation
position as a source-level token. Future semantic passes may interpret it.

## 16. Frontend boundary

The frontend does not implement package resolution, namespace resolution, imports,
exports, visibility, versioning, caching, filesystem lookup, namespace
graph assembly, dependency resolution, access control, or metaprogramming
injection.
