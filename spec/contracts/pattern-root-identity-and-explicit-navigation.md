# Pattern-Root Identity and Explicit Navigation

Status: Current implementation contract

Pattern roots and Hole binders use stable resolved identities. A Hole is
qualified by its `ResolvedPatternRootId` and `HoleBinderId`; spelling and source
position are diagnostic provenance only.

Name resolution produces one terminal name binding before any value, type, call, or
Pattern projection. Explicit navigation therefore resolves a stable host chain
and terminal name binding once. Callability, applicability, and extraction failure do
not restart lexical resolution at an outer same-name binding.

Pattern structural incidence is recorded separately from ordinary members.
Generated fields contribute explicit `DirectPatternChild` evidence; ordinary
lookup-visible or virtual members do not acquire structural status by presence.

Owner identity is determined by the typed `SemanticOwner` graph and canonical
meta-instance root key. Destination binding paths, registry allocation order,
and display names do not reroot a Pattern or complete type value.
