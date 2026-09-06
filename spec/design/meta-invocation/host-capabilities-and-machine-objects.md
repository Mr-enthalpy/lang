# Host Capabilities and Machine Objects

Status: canonical boundary; concrete IO/FFI callable families remain future work.

## 1. The ordinary Object boundary

    HostCapability(args) -> Object

A host capability supplies a resource or observation that the language cannot
obtain by itself. Its result enters the ordinary three-axis Object universe.
File IO, DLL/FFI, system calls, devices, registers, heterogeneous compute and
storage share this boundary. Authority/effect contracts may differ without
introducing separate file, device, or foreign-result ontologies.

link is a future member of this callable family. Acquiring a library is an
ordinary call, followed by ordinary binding or named contribution. The result
becomes a navigation prefix through its ordinary Object members. link has no
mount, namespace-injection, package-graph, or build authority.

The public model is an already available first-class object::path. Source meta
evaluation can implement acquisition, validation, representation construction,
specialization, and export without requiring users to operate a separate
file-to-language-object subsystem.

### 1.1 link-specific effect law

Being an ordinary host callable does not erase link's once-per-compilation
root-acquisition discipline:

    LinkRegistry is part of State_E
    LinkState : HierarchyRootId -> {Active, Done}  (partial map)
    h = CanonicalRootIdentity(source_provider, requested_root)

Absence is registry state, not an optional language result. The provider's
canonical root identity identifies the source hierarchy; raw request strings,
relative/absolute spellings, aliases, or alternate handles for the same
provider/root cannot create distinct h values. Content equality alone does not
identify independently rooted hierarchies either.

    absent h -> reserve h as Active -> evaluate hierarchy -> Done
    Active h -> CycleDiagnostic
    Done h   -> DuplicateLinkDiagnostic

The Active-to-Done transition occurs only after successful evaluation. Failure
propagates through the ordinary Diagnostic/transaction rules, not a fabricated
Done entry or implicit retry. The explicitly selected compilation root is
entered as Active while it evaluates and becomes Done on success, so a link
back into that active root is also a cycle.

This is one compilation-wide registry owned by E, shared by nested link calls
and every sibling block. Nested entry cannot allocate a private registry.
Unordered overlays must retain and validate root-acquisition effects against
this same registry; they cannot each accept the same absent key and silently
coalesce it at join. Overlapping sibling acquisitions of the same root are a
duplicate-root effect conflict (DuplicateLinkDiagnostic); actual re-entry into
an active evaluation is CycleDiagnostic. A serial or parallel implementation
must preserve these distinctions rather than choose a winner by filename or
worker timing. Registry validation does not expose ordinary sibling namespace
writes or give siblings a new execution-order dependency.

Done means the root has already been acquired in this compilation; it is not
permission to return a cached result for a second source-level link. A cache
may realize the first authorized acquisition while preserving its effects,
identity and Pre/commit/Post checks. A fresh compilation has its own registry.

The returned result is still an ordinary Object, and any resulting names are
created by ordinary language actions. This operation-specific effect law
introduces no build graph, mount authority, namespace injection or restriction
on unrelated host IO callables. DependencyGraph remains a projection of actual
evaluation effects, including these root-acquisition effects.

## 2. Primitive admission discipline

For a proposed capability, first ask whether existing IO Objects and ordinary
representation can express it. If so, use them. Otherwise a primitive is
justified only by a genuinely unavailable host capability. Expose that primitive
as an ordinary callable whenever possible. New syntax requires a semantic
boundary that ordinary calling cannot express.

    representation > library > host callable primitive > syntax

These are design priorities, not a new overload order. Bootstrap implementation
does not prove that an operation is a permanent primitive.

## 3. Knowledge and realization

    CompileAcquirable(x) does not imply FullyCompileExported(x)
    CompileKnowledge(x) differs from CompileMaterialization(x)

An acquired Object can expose metadata through compile policy/view while
deferring expensive payload realization to runtime. Shape, layout, dtype,
partition, storage and device metadata need not force full compile-time payload
loading. Ordinary policy, view, construction, and migration rules describe the
available observations and realizations.

## 4. Target-machine facts

The target machine is described by compile-known Objects obtained through source
meta actions and, where necessary, host capabilities. Width, alignment,
representation validity, arithmetic/rounding, trap behavior, and ABI-relevant
semantic facts enter the language world before evaluation, optimization, or
materialization relies on them.

Concrete machine types have the same meaning at compile and runtime stages.
Overflow, invalid representation, alignment failure, and machine traps follow
their explicit target/operation rules: ordinary result, wrap, trap, diagnostic,
or another explicitly defined outcome. An unspecified region is never a premise
that permits an optimizer to assume the event cannot occur.

Target flags, feature flags, dependency flags, manifests, and library paths
cannot supply a second channel of program meaning. The compilation level and
source meta actions determine acquisition and selection. Planner parameters
only govern equivalent-transform search.

## 5. External behavior and optimization

Externality does not imply opacity. The precision of ordinary semantic
descriptions controls what can be proven. Machine types carry no hidden alias
authority: different types alone imply no non-aliasing fact. Representation,
lifetime, and machine projections judge type punning explicitly.

Host acquisition brings external resources into ordinary Objects. Unsafe
admission brings externally established behavior into ordinary projection
facts. The latter follows the separate
[admission boundary](../lifetime/unsafe-semantic-admission.md); neither path
feeds private assumptions directly to an optimizer.
