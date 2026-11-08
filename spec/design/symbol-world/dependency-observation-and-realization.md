# Dependency observation and realization

**Status: canonical dependency authority.**

This owner defines dependency requirements, their semantic realization and
projection. [Invocation](function-object-call-model.md) and
[lifetime](../lifetime/lifetime-policy-and-overload-boundary.md) remain separate
consumers. No dependency-specific evaluator or Object kind is introduced.
Source formation and propagation consumers remain pending.

## 1. General requirements

```text
Needs(action, source, observation, Gamma, Sigma)
```

This relation describes ordinary function references, external value reads,
type information, host resources, explicit reference targets, instance
construction material and actual uses of late-resolved names. It is a
cross-cutting judgment, not a new Object kind.

Automatic capture is not a prerequisite definition for all these cases.
Capture syntax and existing carriers may remain, but general dependencies own
the semantic account.

## 2. Three distinct layers

```text
DependencyRequirement
    the required observation, its source, and required Policy/access

DependencyRealization
    the selected way to obtain or retain semantic material:
    ordinary value transfer, explicit reference, stable link,
    or another existing legal dependency relation

PhysicalRepresentation
    fields, addresses, embedded constants, static links,
    stack environments or zero-storage layouts
```

Representation may optimize among equivalent layouts. It cannot arbitrarily
choose between retaining a value snapshot and reading a live position on each
use. Those different behaviors are fixed by requirements, semantic realization
and ordinary actions.

A requirement grants no access, write, borrow or lifetime permission. Each
operation still needs its ordinary candidate and legality evidence.

### 2.0 Requested observation kind and horizon

Classify the requested observation and its actual demand horizon, rather than
the surface stage of the source Object:

```text
DepKind(d) = (UseKind(d), DemandStage(d))
UseKind = StructuralAccess | Value
DemandStage = compile | runtime
AdmittedDependencyKind(d) iff DepKind(d) != (StructuralAccess, runtime)

D(C) = D_struct_compile(C) union D_value_compile(C) union D_value_runtime(C)
```

StructuralAccess uses the ordinary compile structure, Path, name and type
relations and forms automatic structural dependencies. It is independent of
closure value-capture mode and ordering. A compile classifier/Pattern observation
of a runtime Object is still a compile observation. A structural name or Path
whose identity requires runtime work cannot be admitted as a value capture.
Value observations at either horizon belong to the value-capture class below.

A formed type's retained internal structural Paths also constrain nonidentity
compile-to-runtime same-Type migration. The
[Policy owner](policy-and-static-flow-projection.md#34-policy-migration-satisfaction-existing-first-unique-migration-second)
checks their recoverability in each candidate's prospective destination world
at hard A: false excludes, unknown is unavailable. Source-world success alone
is insufficient. This reads actual retained requirements, not InPlace origin,
and never authorizes re-resolution of formed sources or a placement-based Bind/Move/
Store/Pass ban. An already accepted view retains its established obligations.

### 2.1 Realization is fixed by the selected ordinary action

The source occurrence selects its ordinary semantic operation through the
existing resolution and candidate pipeline. Dependency realization consumes
that selection; closure lowering does not choose an observation strategy:

```text
N = Needs(action, source, observation, Gamma, Sigma)
Realize(N, SelectedOrdinaryAction) = D

Realize(N, A) = D1 and Realize(N, A) = D2
    => D1 observationally_equivalent D2
```

This is at most one semantic realization for the same requirement, selected
action and semantic state, modulo observational equivalence; it does not
promise that an illegal or unready action succeeds. If distinct ordinary
candidates provide different behaviors, ordinary applicability/preference and
unique selection decide, or ordinary ambiguity is reported. A selected failure
does not reopen that choice. A retained continuation preserves the selected
action and its unresolved obligations.

For `let x = ...; let f = { x };`, whether the external occurrence retains a
value snapshot or a live target follows its selected ordinary read/transfer or
reference operation. Snapshot(x) and LiveLink(x) are not interchangeable merely
because both fit an environment layout. A plain value read does not acquire a
live reference through closure lowering; a reference read retains its actual
target and generation through the ordinary reference semantics. The backend
may change representation only while preserving the chosen observations.

## 3. Required and initialized captures

Capture source timing is independent of Explicit/Automatic value-capture mode:

```text
[name], [name: rank], [p let name] => RequiredCapture(slot, requested_name)
[p let name = E], [name = E]      => InitializedCapture(slot, E)
```

A RequiredCapture declares name-sensitive necessary call material. Formation
retains its binder, requested name, rank, Pattern and Policy; it executes no
implicit `name = name`, reads no source resident and establishes no source edge.
InitializedCapture evaluates E once at reached formation under ordinary
selection/Pre/commit/Post and retains the resulting source realization.
Non-bare expression shorthand `[E]` may infer an InitializedCapture binder under
the syntax owner's unique free-name rule; the reserved required forms take
precedence. `[x]` is required, while `[let x = x]` is initialized.

```text
Gamma_call; K; candidate_entry |- ResolveRequired(slot, requested_name)
    => Bound(source, evidence) | Mismatch(proof) | Unavailable(d)
Captured(d,I,K) = ResolveRequired(d, Gamma_call, K, Entry(I)) if Required(d)
Captured(d,I,K) = FormedSource(d)                           if Initialized(d) or Automatic(d)
```

Resolution uses the actual admitted call context and the declared requirement
of this entry. Nearest-name resolution is unique; a missing/incompatible local
name cannot fall back to a formation-time global. Ordinary rank, Pattern,
Policy, access and lifecycle obligations still apply. Required unknown evidence
is unavailable. This is fulfillment of a declared input, not dynamic lookup by
arbitrary body strings. The body resolves its capture binder lexically.

Candidate-local availability is read-only in the existing R_vis/hard-A pipeline;
it performs no initializer, Move or body execution and adds no preference.
After the ordinary unique seal, selected binding/Pre failure is terminal, with
no contextual overload search or runner-up. Only selected input is moved once.

All initialized entries share the enclosing pre-capture name environment.
Their effects follow ordinary formation order, once per reached occurrence.
Projection, copying and invocation do not rerun those expressions or change
their source. An initialized global IO/heap binding therefore survives local
same-name shadowing. Required specifications survive copying/replication and
are fulfilled afresh for each genuinely new invocation; a sealed invocation
retains its bound source and cannot resolve it again at runtime.
These laws preserve the original structured callable and its materials.
Source-erasing projection does not mutate/rebind them: it exposes a separately
parameterized interface under the spec/substitution/resource rules of §4.1
and the Pattern owner.

## 4. Explicit and automatic dependency formation

Each closure has one value-capture mode. Automatic compile structural access
remains independent of that mode:

```text
ValueCaptureMode(C) in {Explicit, Automatic}
Mode(C)=Explicit  => D_value_automatic(C)=empty
Mode(C)=Automatic => D_value_explicit(C)=empty
DependencyMaterial(C) = D_struct_compile(C) union D_value_Mode(C)(C)
DependencyRequirement -> DependencyRealization
ClosureFormation(C) = FormCompleteType(Head_C, Body_C, DependencyMaterial(C))

WrittenCaptureClause(C) => Mode(C)=Explicit
InPlace(C) => Mode(C)=Automatic
Mode(C)=Automatic does not imply InPlace(C)
```

An explicit clause determines all value dependencies of that closure. A body
observation resolved to its capture binder uses that binder, not the outer
source. Any additional uncaptured external value requirement is diagnosed;
it cannot be appended automatically, even for a different binder. Ordinary
automatic compile structural dependencies remain legal in Explicit mode.
Other legal closure forms without a clause use their established formation
rule to determine Automatic mode; it is not exclusive to InPlace syntax.
Capture initializers retain their pre-capture environment and ordinary
once-per-formation effects. Occurrence and binder identities are preserved,
never deduplicated by spelling or value equality.

Automatic observations and initialized occurrences establish realizations while
forming the closure; explicit required entries retain specifications until call
fulfillment (§3). The source occurrence's selected ordinary action fixes
whether its realization uses owned material, ref/share, a stable link or
another existing legal relation, with all of that relation's premises and the
uniqueness law in §2.1. These are not implementation alternatives for one action.
No automatic borrow, copy or write permission follows
merely from needing a source.

Both return complete tau. Invocation uses formed realizations and fulfills
declared RequiredCapture entries in its actual call context. There is no
separate embedding environment or arbitrary dynamic body lookup. Formation
that is not ready retains its real obligations; an unresolved required source
is a legal call requirement, not an unfinished eager initializer.

Bind, Move, Copy, Return, Store and Pass are ordinary operations on either
result. Their legality depends on actual dependencies and ordinary access,
capability, lifecycle and destination checks, not the source placement tag.
The same applies to outer writes: automatic formation alone grants no authority,
but a legal write-capable realization is not vetoed by in-place provenance.
After formation, explicit/automatic origin supplies no additional call,
overload, move, return or other operation dimension. Actual dependency material,
binder identities and ordinary evidence remain observable under their own rules.

### 4.1 Ordered value-capture material

Let B_K denote the existing bare positional Product at continuation K:

```text
ExplicitCaptureSeq(C) = <d1,...,dn>  // specifications in source order, both stages
AutomaticCaptureSeq_Sigma(C) = <d1,...,dn>
CaptureSeq(C) = the applicable fixed sequence above
CaptureSpecProduct(C) = B(Spec(d1),...,Spec(dn))
CaptureValueProduct(C,I,K) = B_K(Captured(d1,I,K),...,Captured(dn,I,K))
```

Spec(d) is the capture binder/occurrence identity and its input rank, Pattern,
Policy, access and lifecycle requirements. This is ordinary interface description
material, not a supplied input value, resident read or source edge. It exists for
required, initialized and automatic positions. An initializer's one formed value
is not proof that arbitrary replacement inputs satisfy its implementation.

CaptureSpecProduct has no invocation context argument and performs no
ResolveRequired. CaptureValueProduct belongs only to an actual structured call:
required entries then resolve in that call's admitted context, while initialized
and automatic entries use their formed sources. Missing call context can make
that value product unavailable, but cannot by itself prevent source-erasing
interface projection from its available specifications. Such projection uses
the Pattern owner's universal input/substitution obligation, not a sample value
from the projection site. The requested spelling is not a name constraint on
the projected value's future explicit argument.

Every capture occupies a bare positional entry, even if it has a binder name.
The layer is ordered by the ordinary Product rule. Thus interleaved compile,
runtime, compile entries keep all three positions in that order; stage sorting
and layout sorting cannot change semantic indices. A static entry may need no
runtime storage slot without losing its semantic position.

Automatic sequencing is encoded from resolved source-use relations and is
determinate and fixed for each completed closure. The definition of a use and
the cross-implementation ordinal mapping are not a stable public guarantee.
Invocation, copy, return and reify never rerun initialized sources or reorder
positions. Required entries keep their position while their declared source is
bound at each new invocation; copying does not bind them early.
Distinct occurrence/binder identities and selected realizations cannot be
collapsed because their values are equal. These are semantic Product positions,
not public fields, an environment Object or a fixed machine layout.

### 4.2 Compile partner dependencies

A selected CompilePartner retains formed dependencies and the actual sources
bound for its required inputs. Its CompileInstance key observes the parent owner, selected
receiver/call-entry pair and canonical invocation inputs. Dependencies legally
retained by that selected callable preserve their actual identity and lifetime;
required bound material is canonical invocation input, not a hidden environment
coordinate. Different observed required sources cannot alias the same key by
omitting them. Requested-name material enters In.

Every closure keeps its lexical, Self, navigation and dependency layer. A nested
closure may use only legally available material, including an enclosing instance
self-name through ordinary admitted dependencies. Invocation consumes those
realizations at their established coordinates; required binding is the declared
call-input operation of §3, not a change to body lexical/navigation scope.

## 5. Projections of one dependency

```text
D -> Pi_sigma(D)
```

Compile evaluation may consume permitted type/Pattern information while
unreadable runtime Val1 remains in the residual continuation. Runtime does
not resolve an already bound source again. A newly formed invocation fulfills
its required specifications at its own actual frontier.

If initialization itself contains runtime actions, their formation residue
must remain. Compile evaluation cannot fabricate an already initialized
capture value. A head, compile key or complete normalization requiring that value
must satisfy readiness and legality; unknown material is not proof of equal
Core.

Continuation deferral preserves these dependencies and effects. A closure dependency
remains subject to its actual horizon, readiness and execution legality.

## 6. Semantic state cannot be hidden in a side table

A dependency judgment may be cross-cutting. A realization retaining an owned
value must expose that material to ordinary structural identity, copy/move and
lifecycle observations. A retained reference preserves its semantic target and
generation.

Two values with equal complete observations in the same applicable context
cannot have different meanings solely because of an unaccounted capture side
table.

This does not require every dependency to become a public self.Val2 field.
Semantic state and layout are distinct; ordinary linked/reference realizations
retain their own rules. Default Core type equality is not complete callable
state equality and cannot merge distinct snapshots or selected instances.

## 7. Identity and copying

Copying or anchored replication of a formed callable preserves its established
dependency material through the existing legal value/ref/share/copy relations.
It does not rerun surrounding expressions, re-resolve formed sources or extend
a referenced object's lifetime. Required specifications remain requirements;
their later fulfillment follows §3 rather than replication-time lookup.

The [lifetime handoff](../lifetime/lifetime-policy-and-overload-boundary.md#8-closure-dependency-lifetime-refinement-handoff)
owns persistence and move/escape refinement. This dependency relation adds no
universal lifetime theorem.

## Delayed concrete input and opaque self

Compile callables admit ordinary implicit dependencies. Requirements, selected
realizations, and current demand for available concrete values are separate.
Runtime materialization can complete ordinary closure capture. Result-affecting
observations enter dependency validity and instance reuse. Norm_Name(n) does not
unfold self's capture layout; self remains observable through normalized Pattern
algebra without public transparent structure. A callable reify P2 boundary uses the same E and ordinary input, dependency
and current-name-position lifecycle facts. Actual reverse dependencies determine
the materialized successor cone; unrelated compile type work remains ready.
Its outward result classifier and input/output/regular-effect/capture/lifecycle
interface shape satisfy ClosedReturn and ClosedInterface in
[unified §20](../unified-static-name-and-structural-semantics.md#20-reify-callable-p2-and-scoped-reverse-migration),
independently of runtime reverse flow. A fixed sum still admits runtime branch choice.
