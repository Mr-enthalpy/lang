# Library and Namespace Design Note

**Status: Non-normative future design note. This is not a v0.1 parser rule.**

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

## 4. Physical namespace skeleton

Filesystem directories provide only a physical namespace skeleton.
Implementation file names do not create namespace segments. The physical
filesystem skeleton is a proper subset of the full namespace graph. The full
graph may contain physical namespace nodes, declared namespace objects, and
virtual namespace nodes.

## 5. `let ns1: namespace = ...` is a language-level declaration

`let ns1: namespace = ...` is a language-level namespace object declaration
or description, not a package mount or import. The source name `namespace`
is an ordinary `Name` token in v0.1.

## 6. Export and visibility

Export and visibility are namespace assembly, resolver, or package metadata
concerns. They are not source-level syntax in v0.1.

## 7. Versioning and caching

Versioning and caching must not appear in ordinary source namespace paths.

## 8. v0.1 prohibition

v0.1 must not implement package resolution, namespace resolution, imports,
exports, visibility, versioning, caching, filesystem lookup, or
metaprogramming injection.
