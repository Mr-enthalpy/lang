# Compile Construction Boundary

Status: semantic handoff with source-consumer migration pending

Compile construction uses the shared semantic universe and ordinary invocation
boundary. Selected builtin leaves produce private material consumed at their
own declared-result boundary under the call's specified `SemanticOwner`.
Source bodies require ordinary semantic completion through common E; they do
not forward builtin private material. The declared semantic entity is returned through:

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

Every selected compile call forms its stable CompileInstance before EnterBody,
independent of result kind and P2. The instance self-name is ordinary NameValue;
resident reading requires initialized complete storage and does not reenter the
body. Declared result class, ReturnPattern and ResultPolicy are independent.
Ordinary scalar, Product, closure and external type results are admitted.

Accessible instance-open self-root types must form exactly one direct complete
type result tau_I. The traversal includes Product elements, captures, references,
share targets and other admitted wrappers. Missing traversal evidence is
unavailable. Valid direct self-root results provide Read_name(result)=n_I;
external type results do not manufacture that NameExpr. Input dependency
normalization preserves value, observed name/subject and borrowed-target identity.
Output opening sources follow the actual dependency meet. OpenPolicy=open
retains the established source; close follows ordinary result delivery with
Complete and Close. Unknown required OpenHere is unavailable.

The invocation registry/cache associates the full key with the same result
binding/Place and construction status. Repeated completed acquisition reads the
current resident, does not rerun initialization, and rechecks dependencies and
current Pre. Value/material reuse is separate. A direct self-root result keeps its root invariant; other ordinary results and
Val2 members keep their own owners or targets. Untransferred locals cannot escape via cache storage.
These laws are owned by
[compile invocation](../design/static-evaluation/compile-instance-invocation-and-result-delivery.md).

A returned construction value does not implicitly install its outer binding.
An explicit binding action creates the destination name and Place. Construction
bodies perform value-side name formation and separately authorized ref/write
or ordinary *= actions. Initializer-free P let name:t and P let name::path:t
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
replacement capability and resident compatibility. A qualified let with RHS forms a complete binding using RHS inference. Close requires retained structural names being published to be
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
Construction effects participate in the enclosing evaluator's existing commit
rules; files do not supply construction authority.

The connected substrate uses ordinary `CallableDeclaration` records and one
selected implementation coordinate. `SourceCallableSyntax` holds normalized
syntax, not an Object. Builtin leaves produce private construction material;
source blocks require the shared completion consumer and remain unavailable.
The source-body frontier returns diagnostics only, with no success carrier for
builtin material. Inner builtin invocations consume `BuiltinBodyMaterial` into
their own complete semantic result; future source execution must deliver ordinary
semantic completion through common E rather than forward that private material.
There is no separate compile body evaluator. Preparation and horizon visibility
prove neither Ready nor execution legality.

`CompileInstanceId` reuses the interned semantic owner independently of
TypeValueId. Admission creates its independent ordinary self-name coordinate,
with initialized residency requiring an ordinary producer. `CompilePartner`
retains the actual receiver/call-entry pair; canonical arguments and parent
complete `CompileInstanceKey`. Body/result material and provenance supply no
identity coordinate. Connected external-type delivery retains the input's exact
complete snapshot and leaves instance self residency uninitialized.

General instance residency, accessible-result closure, opening-source meets,
P1 completion and source body execution require their common E consumers.
Structural source formation and ordinary struct helper formation are unavailable;
schema normalization and graph records establish no structural member or role.
Selected failures publish no semantic world mutation.

Compilation entry has runtime P2 and omitted ordinary P1 defaults to runtime.
Bootstrap or ordinary legal compile formation supplies stable roots, without an
active compile wrapper over all compilation. CompileInstance formation is compatible with seal P2. Helper calls and cache
acquisition obey ordinary Ready and execution legality.
