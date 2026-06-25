# Package Manifest v0

**Status: Provisional non-normative future design. Not a v0.1 parser rule.**

## Scope

This document defines the provisional build-manifest design surface needed to
guide future build-system work. It does not finalize a complete manifest schema.

## Manifest surface

A package manifest describes:

- **Package identity**: globally unique package name.
- **Package kind**: `library`, `application`, `test`, `plugin`, or future unit
  kinds.
- **Namespace root**: the top-level namespace segment this package provides
  (e.g., `mylib`, `myapp`).
- **Source roots**: directories containing implementation source fragments
  (e.g., `["src"]`).
- **Dependencies**: list of external packages whose namespace roots are
  mounted into the compilation namespace graph.
- **Mount table**: mapping from dependency names to their exposed namespace
  roots, with optional aliases.
- **Entry point**: for applications, a namespace path naming the entry
  declaration (e.g., `myapp::main`).
- **Lockfile relationship**: a lockfile records resolved dependency versions,
  hashes, and conflict resolutions for reproducible builds.
- **Distribution form**: static, dynamic, source, or interface+binary.
- **Visibility/export metadata**: controls which declarations are externally
  visible from the package's namespace.
- **Feature/configuration set**: build-time feature flags and compiler
  configuration.
- **Cache metadata**: keys for build-artifact caching (package version,
  compiler version, feature set, instantiation arguments).

## Non-goals

- No source-level import syntax.
- No package declarations in source files.
- No namespace resolution in v0.1.
- No type checking.
- No linking.
- No cache validation implementation.
- No dependency solver implementation.
