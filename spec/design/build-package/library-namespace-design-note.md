# Libraries as Ordinary Objects

Status: canonical boundary; concrete acquisition APIs remain future work.

A library is presented to its users as a first-class Object with ordinary
navigation. Source meta evaluation can acquire resources, validate them,
construct their representation, specialize them and expose ordinary members.

    HostCapability(args) -> Object
    source binding/contribution -> ordinary object navigation

link is one possible future host-backed callable. It is not a mount operation
or a separate namespace mechanism. The language uses its ordinary calls,
Patterns, complete pattern values and OverloadGroups at the boundary.

The source decides external resource acquisition and target-machine facts.
Build manifests, feature configuration, dependency lists and package roots are
not additional semantic inputs. Dependency information is a projection of
actual evaluation effects, available to engineering tools afterward.

Compilation-stage acquisition need not eagerly realize an entire external
payload. Policy/views can expose shape, layout, dtype, partition and metadata
while ordinary runtime realization supplies expensive data or handles.
External provenance does not require optimizer opacity; precise ordinary
projection facts can support the same proofs as for internally acquired data.

Existing public/private/export and capability rules govern the resulting
Objects. File or distribution boundaries neither grant nor withhold construction
authority. Mutable target references and OpenHere do that work. The externally
visible name set is closed before external navigation observes a construction
result. Stable externally supplied members do not authorize later reopening.

See [host capabilities](../meta-invocation/host-capabilities-and-machine-objects.md),
[source normalization](build-system-design.md) and
[construction composition](../symbol-world/symbol-construction-units-and-namespace-origin.md).
