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
  × CanonicalizeInvocationInputs(In)
```

Ordinary meta constructs its instance name/type tau_M rooted at M, with direct
result class CompleteType. Arbitrary payloads belong in ordinary Val2;
name::path navigation and explicit compile extraction expose their values. Input normalization preserves value observations
and every semantically observed name/subject/borrow dependency identity. The
result name is not an input structural child and changes neither input Val2 nor
Norm. Its openness source is the meet over AccessClosure_out(In); open inputs
are admitted with ordinary identity, access and lifetime checks. P1 meta retains the instance under OpenHere, which governs mut acquisition;
plain let completes and closes it. P2 meta remains the evaluation stage. Global persistence requires the stronger
global stability/escape judgments.

The invocation registry/cache associates the full key with the same result
binding/Place and construction status. Repeated completed acquisition reads the
current resident, does not rerun initialization, and rechecks dependencies and
current Pre. Value/material reuse is separate. The direct instance type keeps
its root invariant; ordinary Val2 payloads keep their own value owners or targets. Untransferred locals cannot escape via cache storage.
These laws are owned by
[meta invocation](../design/meta-invocation/meta-object-invocation-and-policy-reduction.md).

A returned construction value does not implicitly install its outer binding.
An explicit binding action creates the destination name and Place. Construction
bodies can perform authorized ordinary binding/inject actions through actual
mutable type references. Structural P let name::path:t creates NameExpr for a fresh typed Place with
ResidentState = Uninitialized (non-Object evaluator state). Omitted :t means
:type, not a resident type value. Value use requires initialization; explicit
ref borrows the Place using its declared type without reading. Ordinary write
initializes it, and later writes replace its resident. The structural let=compound
is not canonical. Close requires externally resolvable structural names to be
initialized. Ordinary lexical let remains unchanged.

Same-name construction synthesizes a named type's V_tau under membership and
OpenHere checks. An explicit OverloadGroup aggregates type candidates instead.
See [names and type algebra](../design/symbol-world/names-and-overload-groups.md),
[closure replication](../design/symbol-world/closure-anchored-replication.md),
and [associated state A](../design/symbol-world/associated-compile-state.md),
which is derived from the general invocation facilities.
Legacy result-class/cell carriers do not add a semantic result ontology.
Construction effects participate in the enclosing evaluator's existing commit
rules; files do not supply construction authority.
