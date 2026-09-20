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
plain let completes and closes it. P2 meta remains the evaluation horizon. Global persistence requires the stronger
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
bodies perform value-side name formation and separately authorized ref/write
or inject actions. Initializer-free P let name:t and P let name::path:t
create typed NameExpr at lexical and structural destinations respectively.
Qualified formation uses resolved structural root identity and the current
resident type's OpenHere, valid selector, non-retention and ordinary access/path/type
checks; it requires neither parent Writable nor a parent mut type ref.
Their Place state is Uninitialized, not an Object. The initializer-free
structural form defaults to :type; P let name = rhs is instead a complete
lexical binding with RHS type inference. Value use requires initialization; explicit
ref borrows the Place using its declared type without reading. Ordinary write
initializes it using authority independent of the name's declaration policy,
including const. Successful first commit consumes that authority; saved initial
references do not grant replacement power. Later writes require ordinary
replacement capability and resident compatibility. The structural let=compound
is not canonical. Close requires retained structural names being published to be
initialized; it does not require all future generated coordinates to be realized.
Ordinary lexical let remains unchanged.

Implementation-layer closure expressions produce tau_C; ordinary let binds the
result. Same-name synthesis requires an established structural contribution
role and membership/OpenHere checks; ordinary legal binding/shadowing/mutation
never becomes contribution by RHS shape or failed execution. An explicit OverloadGroup aggregates type candidates instead.
See [names and type algebra](../design/symbol-world/names-and-overload-groups.md),
[closure replication](../design/symbol-world/closure-anchored-replication.md),
and [associated state A](../design/symbol-world/associated-compile-state.md),
which is derived from the general invocation facilities.
Legacy result-class/cell carriers do not add a semantic result ontology.
Construction effects participate in the enclosing evaluator's existing commit
rules; files do not supply construction authority.

Compilation entry has runtime P2 and omitted ordinary P1 defaults to runtime.
Bootstrap or ordinary legal meta formation supplies stable roots, without an
active meta wrapper over all compilation. Actual MetaDom forbids seal work;
actual SealDom forbids meta invocation, including cache hits and helper calls.
