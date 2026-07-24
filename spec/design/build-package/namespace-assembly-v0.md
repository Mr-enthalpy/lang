# Namespace Assembly v0

**Status: Provisional non-normative future design. Not a v0.1 parser rule.**

The canonical namespace-origin and construction-unit ownership contract is
`spec/design/symbol-world/symbol-construction-units-and-namespace-origin.md`.
The current `lang_build` slice does not implement that complete contract.

## Scope

This document explains the future namespace assembly pipeline at a high level.

## Assembly pipeline

```
manifest -> package graph
  -> namespace root mount table
  -> physical namespace skeleton from source roots
  -> source fragment declaration index
  -> later: semantic namespace graph
```

## Phase split

### Build Phase A: manifest and package graph

Parse package manifests, resolve dependency identities, construct the package
dependency graph.

### Build Phase B: namespace mount table

From the package graph, produce a mount table mapping each dependency's
namespace root to its resolved origin. Resolve mount conflicts by policy.

The v0.6 typed substrate models namespace assembly as an ownership/containment
forest plus redirect edges:

```text
PackageBoundary { package_id }
Mount { target: existing NamespaceNodeId }
```

`PackageOf(node)` is the nearest package-boundary ancestor. A mount is an
alternative access path, not copied ownership:

```text
Identity(resolve(x::mount_path)) = Identity(resolve(x::target_path))
```

Crossing from the query package into the mounted target package switches
subsequent lookup from `FullNameView` to `ExternalNameView` and retains typed
failure causes such as private path, non-retention, no externally eligible
candidate, missing target, and missing package boundary.

The notation above follows source order: the selected inner symbol is leftmost
and the outer mount/namespace components follow to the right. A graph resolver
may reverse the component list mechanically for outer-to-inner containment
traversal.

### Build Phase C: physical namespace skeleton

Walk source roots to build the physical namespace skeleton from directory
structure. Each physical directory creates a namespace facet with
`NamespaceOrigin::PhysicalDirectory(path)` and establishes contribution
authority for files physically in that directory. Implementation filenames do
not contribute namespace segments.

### Build Phase D: parser-backed declaration index

Parse source fragments and index top-level declarations by namespace path.
This phase requires a stable enough AST (at minimum: let binders and closure
AST shape). It does not resolve types, values, or references. Each physical file
is indexed as one closed `SourceConstructionUnit`; files in one directory may
declare distinct direct children but do not acquire authority to reopen one
another's child subtrees.

### Build Phase E: semantic namespace graph

Resolve declarations across namespaces, apply visibility rules, evaluate
virtual namespaces, populate cache metadata, and integrate closure object
materialization. This phase validates:

```text
one NamespaceOrigin per child namespace facet
source/meta construction-unit ownership
physical directory contribution authority
type-facet single installation
cross-file reopening prohibition
NamespaceDelta atomicity
semantic-owner identity
package-derived FullNameView / ExternalNameView routing
default extraction projection
```

One source unit may fully construct a new direct-child subtree in its own delta.
One ordinary canonical meta invocation is one `MetaConstructionUnit` and may
fully construct its virtual subtree. Compiler-defined privileged AST meta
functions operate only through their bounded current-unit capability. Parallel
units may not reopen either subtree. This is post-v0.1 semantic work.

At the current specification stage, cross-file type-child, namespace-child,
ordinary value-member, and overload-entry injection are all forbidden. A future
explicitly mergeable overload/value design may relax the last two cases; file
unorderedness alone is not the reason for the current restriction.

## Phase gates

- **Build Phase A, B, C** may start after parser phase 2 (deduce/canonical/extract-let).
- **Build Phase D** should wait until closure AST is stable enough for ordinary
  source fragment indexing (i.e., after parser phase 3).
- **Build Phase E** is post-v0.1 semantic work.

## Non-goals

- No namespace resolution in v0.1.
- No declaration indexing implementation in v0.1.
- No semantic resolver, visibility checker, version solver, or cache validator
  until their respective phases.
