# Meta Construction Boundary

Status: semantic handoff with source-consumer migration pending

Meta construction uses the shared semantic universe and ordinary invocation
boundary. A selected primitive or source body may produce private execution
material. That material is installed under the selected call's stable
`SemanticOwner`, after which the declared semantic entity is returned through:

```text
InvocationResult
  = SemanticResult(DeclaredResultClass)
  | Residual
  | Diagnostic
```

For `struct`, the declared result is a complete type value. Cache entries may
reuse parent-neutral execution material, while semantic instance roots are
identified by:

```text
parent SemanticOwner
  × selected callable identity
  × canonical whole argument Product identity
```

A returned construction value does not implicitly install its outer binding.
An explicit binding action creates the destination name and Place. Construction
bodies can perform authorized ordinary binding/inject actions through actual
mutable type references. Structural `P let name::path` commits fresh-name
creation and returns such a reference; its `= e` suffix is ordinary assignment,
without an additional initialization transaction or rollback rule.

Same-name construction synthesizes a named type's V_tau under membership and
OpenHere checks. An explicit OverloadGroup aggregates type candidates instead.
See [names and type algebra](../design/symbol-world/names-and-overload-groups.md),
[closure replication](../design/symbol-world/closure-anchored-replication.md),
and [associated state A](../design/symbol-world/associated-compile-state.md).
Legacy result-class/cell carriers do not add a semantic result ontology.
Construction effects participate in the enclosing evaluator's existing commit
rules; files do not supply construction authority.
