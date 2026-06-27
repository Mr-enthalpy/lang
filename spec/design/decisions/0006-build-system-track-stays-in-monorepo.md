# ADR 0006: Build-System Track Stays in Monorepo For Now

**Status:** Accepted

**Context:** The build system is semantically separate from the parser
implementation, but depends on language design constraints that are still
stabilizing (no source-level import/export syntax, namespace paths separated
from filesystem layout, package layer vs. language namespace layer). A
separate repository would require stable APIs and manifest/namespace models
before these design questions are settled.

**Decision:** Keep the build-system track in this repository for now.

**Consequences:**

- Parser and build-system documentation and tests can evolve together.
- Shared design constraints are visible in one place.
- A future split into a separate repository remains possible.
- No independent build-system repository exists yet.
- Build-system code must not leak semantic resolver responsibilities into v0.1
  parser work.

**Split condition:** Consider a separate repository only after manifest schema,
namespace graph model, declaration-index API, and crate public API are stable.
