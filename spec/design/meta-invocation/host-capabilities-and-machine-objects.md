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
