# Package Manifest v0

**Status: Provisional non-normative future design. Not a v0.1 parser rule. Not an
implemented manifest file format.**

## 1. Scope

This document describes the provisional build-manifest design surface needed to
guide future build-system work. It does not finalize a complete manifest schema.

The manifest is an input to the **package/build layer**. It does not define
source syntax, and it does not introduce any package declaration inside `.lang`
source files. A manifest records the build facts that are projected into the
namespace graph and that stabilize candidate identity for future meta object
invocation. It is not a package-manager configuration tutorial; its purpose is
semantic: to make build facts available as referenceable objects.

The canonical layering (package/build layer → namespace graph layer → source
language layer) and the projection model are described in
`spec/future/build-system-design.md`. This document focuses on what the manifest
records and why each record matters semantically.

## 2. Manifest records

A package manifest records, at least, the following. Each record is a build fact
that later projects into the namespace graph or participates in candidate
identity.

| Record | Description |
|---|---|
| Package identity | Globally unique package identity (`PackageId`). |
| Package kind | `library`, `application`, `test`, `plugin`, or future unit kinds. |
| Namespace root | The top-level namespace segment this package provides (e.g., `mylib`, `myapp`). |
| Source roots | Directories containing implementation source fragments (e.g., `["src"]`). |
| Dependencies | External packages whose namespace roots are mounted into the compilation namespace graph. |
| Mount table | Mapping from dependency names to their exposed namespace roots, with optional aliases. |
| Export surface | Which declarations are externally visible across the package boundary. |
| Feature/configuration set | Build-time feature flags and compiler configuration. |
| Entry point | For applications, a namespace path naming the entry declaration (e.g., `myapp::main`). |
| Lockfile relationship | Resolved dependency versions, hashes, and conflict resolutions for reproducible builds. |
| Distribution form | `static`, `dynamic`, `source`, or `interface+binary`. |
| Cache/fingerprint metadata | Keys for build-artifact caching (package version, compiler version, feature set, instantiation arguments). |
| Trust/access policy placeholder | Reserved slot for future trust/access policy on the package. |

## 3. Semantic role of each record

The records are not just configuration fields. Each has a semantic role in
namespace graph projection and in future meta object invocation.

- **Package identity** — used for diagnostics, cache keys, candidate provenance,
  and cross-package uniqueness. Two candidates that share a symbol path but come
  from different packages are distinguished by package identity.
- **Package kind** — selects whether the package merely provides a namespace
  root (library) or also carries an entry point (application), and which future
  distribution/build treatment applies. It is build metadata, not source syntax.
- **Namespace root** — defines the root under which this package's contents are
  projected into the namespace graph.
- **Source roots** — provide the input directories from which the physical
  namespace skeleton is built. Implementation file names remain source-fragment
  names and do not contribute namespace segments.
- **Dependencies** — define the static build-graph ordering and the dependency
  fingerprint flow: a dependency's fingerprint participates in the dependent's
  cache key.
- **Mount table** — defines the paths at which dependency namespace roots are
  projected into the current compilation graph, including optional aliases.
- **Export surface** — defines which symbols, namespaces, and callables an
  external lookup may see. It is a visibility boundary projected into the
  namespace graph, not a source-level import/export pair.
- **Feature/configuration set** — affects the generated namespace graph and the
  cache fingerprint: the same source under a different configuration may produce
  a different graph and therefore a different cache key.
- **Entry point** — for applications, names the entry declaration as a namespace
  path. It is metadata about how the build is consumed, not a source construct.
- **Lockfile relationship** — pins resolved dependency versions/hashes so a
  build is reproducible from package metadata and lock data, while source paths
  stay version-free.
- **Distribution form** — affects metadata availability, linking, and loading.
  It must not change the namespace path spelling that source code writes.
- **Cache/fingerprint metadata** — supplies the keys that let generated and
  instantiated namespace nodes be cached and correctly invalidated.
- **Trust/access policy placeholder** — a reserved slot for future package-level
  trust/access policy; it does not define any current behavior.

## 4. Manifest is not source import

The manifest is a build-layer input, not a source-language construct.

```text
The manifest may mount packages, but source files do not import packages.
The source language sees namespace paths after projection, not manifest clauses.
```

Mounting a dependency makes its namespace root resolvable through the namespace
graph. Source code then refers to namespace paths such as `Vec::std`; it never
writes manifest clauses, import statements, or mount directives.

## 5. Current implementation boundary

The current `crates/lang_build` slice has **no** manifest file parser. Tests and
the build API construct values such as `BuildWorkspace`, `PackageBuildSpec`, and
`BuildManifest` directly in Rust. This is an implementation slice, not the final
manifest surface, and it must not be read as manifest-file syntax. A manifest
file format, dependency solving, lockfiles, remote retrieval, linking, and
binary metadata are future work and are not implemented.

## 6. Non-goals

- No source-level import syntax.
- No package declarations in source files.
- No namespace resolution in v0.1.
- No type checking.
- No linking.
- No remote package retrieval.
- No lockfile completeness.
- No binary/interface distribution metadata implementation.
- No cache validation implementation.
- No dependency solver implementation.
