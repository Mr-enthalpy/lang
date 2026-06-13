# Library and Namespace Design Note

**Status: Non-normative future design note. This is not a v0.1 parser rule.**

See also `spec/build-system-design.md` for the current build-system
architecture document.

## 1. No source-level import/include/module syntax

The core source language has no source-level import, module, or include syntax.

There is no source-level syntax such as:

```text
import mylib
use mylib::math
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

may correspond to the namespace paths `mylib::math::vector` and
`mylib::math::matrix`, but this mapping is performed by the build layer,
not the source language.

For example, both implementation files:

```text
src/math/vector/impl.lang
src/math/vector/ops.lang
```

contribute to:

```text
mylib::math::vector
```

They do not create:

```text
mylib::math::vector::impl
mylib::math::vector::ops
```

Directory paths provide the physical namespace skeleton. Implementation
file names do not create namespace segments.

## 7. Namespace graph node kinds

The full namespace graph may contain three kinds of nodes:

- **Physical namespace nodes**: contributed by filesystem skeleton, build
  descriptors, or package manifests.
- **Declared namespace objects**: created by `let ns: namespace = ...` at
  the language level.
- **Virtual namespace nodes**: synthesized by the namespace assembler,
  metaprogramming, or the resolver. Not tied to any physical source file.

## 8. Physical and virtual namespace layers

The physical filesystem skeleton is a proper subset of the full namespace
graph. The language may reference virtual namespace nodes that have no
corresponding filesystem directory.

## 9. `let ns1: namespace = ...` is a language-level declaration

`let ns1: namespace = ...` is a language-level namespace object declaration
or description, not a package mount or import. The source name `namespace`
is an ordinary `Name` token in v0.1.

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

## 13. Metaprogramming injection rule

Parent-to-child metaprogramming injection means a parent or instantiated node
may create declarations below its own node.

It must not inject into parents, siblings, unrelated globals, or arbitrarily
rewrite existing namespace content unless a later specification explicitly
permits that.

This is a future meta-function / metaprogramming capability. It is not v0.1
and must not be assumed as general language semantics.

## 14. Versioning and caching

Versioning and caching must not appear in ordinary source namespace paths.
Version resolution and artifact caching are package-layer operations.

## 15. Relationship to `namespace` in source

The source name `namespace` is an ordinary `Name` token in v0.1. It carries
no special lexical or parser status. It may appear in declaration annotation
position as a source-level token. Future semantic passes may interpret it.

## 16. Relationship to v0.1

v0.1 must not implement package resolution, namespace resolution, imports,
exports, visibility, versioning, caching, filesystem lookup, namespace
graph assembly, dependency resolution, access control, or metaprogramming
injection.
