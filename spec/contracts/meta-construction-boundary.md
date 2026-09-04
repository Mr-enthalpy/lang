# Meta Construction Boundary

Status: Current implementation contract

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

Construction never installs a binding by itself. The outer binding boundary
creates the destination Symbol and Place and carries the semantic result
explicitly. Namespace installation is atomic.
