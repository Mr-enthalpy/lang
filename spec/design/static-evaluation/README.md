# Static Evaluation and Compile Instances

Compile, seal and runtime use the same evaluator, ordinary Object/Pattern/type,
Place, Policy, dependency and lifecycle relations. InvocationResult is the single
result boundary. CompileInstance formation is independent of result kind and P2.

- [Compile instances and result delivery](compile-instance-invocation-and-result-delivery.md)
- [E, residual continuations and optimization](evaluation-residual-and-optimization.md)
- [Host capabilities and machine Objects](host-capabilities-and-machine-objects.md)

Construction consumes [name/type algebra](../symbol-world/names-and-overload-groups.md),
[structural formation and composition](../symbol-world/structural-type-formation-and-composition.md),
[associated state](../symbol-world/associated-compile-state.md) and
[anchored replication](../symbol-world/closure-anchored-replication.md).
Each selected CompilePartner realization retains its receiver/call-entry pair,
projection and frame through ordinary evaluation and residual transport.
Missing consumer evidence remains unavailable; representation supplies no
alternative identity, result or authority.
