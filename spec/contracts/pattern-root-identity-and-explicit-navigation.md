# Pattern-Root Identity and Explicit Navigation

Status: Current implementation contract

Pattern roots and Hole binders use stable resolved identities. A Hole is
qualified by its `ResolvedPatternRootId` and `HoleBinderId`; spelling and source
position are diagnostic provenance only.

Pure structured Path formation may occur before external Read and before any
resolved target exists. A text root resolves at this Read occurrence's lexical/
semantic environment, while explicit value/ref roots retain their actual material
and target identities. At that first resolution, navigation fixes its stable host
chain and terminal binding before consumer projection. Callability, applicability, and extraction failure do
not restart lexical resolution at an outer same-name binding.

Pattern structural incidence is recorded separately from ordinary members.
Generated fields contribute explicit `DirectPatternChild` evidence; ordinary
lookup-visible or virtual members do not acquire structural status by presence.

Owner identity is determined by the typed `SemanticOwner` graph and canonical
meta-instance root key. Destination binding paths, registry allocation order,
and display names do not reroot a Pattern or complete type value.


[Path projection and general splice](../design/symbol-world/structured-path-algebra-and-pattern-splice.md)
do not turn strings into resolved coordinates or access authority. Once resolved,
projection, runtime residue and caches retain that same chain without relookup.
Splicing a Pattern preserves its existing PatternRoot/HoleBinderId references;
illegal scope fails rather than rebinding by spelling or automatic alpha-renaming.
