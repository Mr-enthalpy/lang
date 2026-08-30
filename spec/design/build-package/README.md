# build-package

**Status: Canonical build/package design. The typed owner/namespace graph,
transactional deltas, mounts, and cache validation are implemented; manifest files,
dependency solving, lockfiles, remote retrieval, linking, and binary metadata
are not implemented.**

## Scope

The package/build layer as the **semantic projection layer** for the namespace
graph. It owns build facts and projects them into resolvable, diagnosable,
cacheable namespace-graph objects:

- package identity (`PackageId`), package kind, source roots
- dependency edges, mount paths, mount table
- physical directory namespace origin and contribution-authority projection
- export surface (cross-package visibility boundary)
- feature/configuration set, distribution form, entry point
- cache / fingerprint / provenance

## Not in scope

Language expression semantics, type values, pattern/overload, and meta
invocation. This block produces the graph that later blocks resolve names in;
it does not define what those names mean.

## Repository placement

Repository placement is not semantic identity. The build-package track remains
in this repository while the manifest schema, namespace graph model,
declaration-index, and crate public API are unstable; this does not collapse
parser/frontend and build/package responsibilities, and build-package code must
not leak semantic-resolver responsibilities into the frontend. A future split
into a separate repository remains possible only after those APIs stabilize.

## Documents

- `build-system-design.md` — the package/build layer and namespace-graph projection.
- `package-manifest.md` — manifest records and their semantic role.
- `namespace-assembly.md` — assembly pipeline and phase split.
- `library-namespace-design-note.md` — library/namespace/no-import model.

Cross-block canonical construction contract:

- `../symbol-world/symbol-construction-units-and-namespace-origin.md` —
  namespace-facet origin uniqueness, source/meta construction ownership,
  physical directory authority, and cross-file closure.

## Reading order

Read in the listed order, starting with `build-system-design.md`, then read the
cross-block construction contract before changing source-file or meta-produced
namespace assembly.

## Dependencies

Feeds `symbol-world/` (namespace roots, mounts, provenance) and ultimately
`patterns-overload/` and `meta-invocation/` (candidate identity / provenance).
The source language sees only the projected namespace graph, never packages,
files, or manifests directly.
