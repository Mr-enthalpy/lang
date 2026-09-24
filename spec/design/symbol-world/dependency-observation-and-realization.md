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

## 3. Explicit dependency clauses

```text
[let x = E]
[x = E]
[x]
```

These forms provide explicit binding/dependency material for closure formation
under the existing shorthand rules. [x] is not automatically const and does
not create write authority.

Initializers share the pre-capture name environment. A later item cannot see
a newly created earlier capture binder merely because it occurs later in the
list. This common name scope does not make effects simultaneous: evaluation
order, reads, writes and one-time effects follow ordinary formation semantics.

Each reached formation occurrence executes its initializers as prescribed.
Projection does not repeat execution, and invoking the body does not rerun
the formation expressions.

Resolved external dependencies without an explicit clause use established
ordinary reads and permitted dependency formation. Renaming this relationship
does not expand automatic borrowing, copying or writing.

## 4. Explicit and automatic dependency formation

For a non-MetaDecl closure, classify dependency occurrences, not the closure's
placement. A single ordinary closure may contain both kinds:

```text
DependencyMaterial(C) = ExplicitDeps(C) union AutomaticDeps(C)
ExplicitCaptureOccurrence(C, d)
    => d in ExplicitDeps(C)
FreeExternalObservation(C, d) and not ReplacedByExplicitCapture(C, d)
    => d in AutomaticDeps(C)
    => Needs(Form(C), Source(d), observation, Gamma, Sigma)
DependencyRequirement -> DependencyRealization
ClosureFormation(C) = Struct(Head_C, Body_C, DependencyMaterial(C))

InPlace(C) => ExplicitDeps(C) = empty
AutomaticDeps(C) != empty does not imply InPlace(C)
```

The union preserves occurrence and binder identities; it is not value-based
deduplication. ReplacedByExplicitCapture follows resolved binding, not a match
of source spellings: a body occurrence resolved to an explicit capture binder
does not also capture its outer source automatically. Other eligible free
observations form automatic dependencies even in an ordinary `=>` closure
with an explicit clause for different occurrences. Capture initializers retain
their pre-capture environment and ordinary once-per-formation effects.

Each occurrence establishes ordinary requirements and realizations while
forming the closure. The source occurrence's selected ordinary action fixes
whether its realization uses owned material, ref/share, a stable link or
another existing legal relation, with all of that relation's premises and the
uniqueness law in §2.1. These are not implementation alternatives for one action.
No automatic borrow, copy or write permission follows
merely from needing a source.

Both return complete tau and invoke using established dependencies. There is
no separate embedding environment, delayed lookup by spelling at candidate use,
or recapture during invocation. Formation that is not ready retains its
formation obligations rather than claiming a completed dependency.

Bind, Move, Copy, Return, Store and Pass are ordinary operations on either
result. Their legality depends on actual dependencies and ordinary access,
capability, lifecycle and destination checks, not the source placement tag.
The same applies to outer writes: automatic formation alone grants no authority,
but a legal write-capable realization is not vetoed by in-place provenance.
After formation, explicit/automatic origin supplies no additional call,
overload, move, return or other operation dimension. Actual dependency material,
binder identities and ordinary evidence remain observable under their own rules.

### 4.1 Meta declarations have no closure capture channel

The callable that establishes MetaDecl/MetaInvoke identity is excluded from
the ordinary closure capture rules above:

```text
MetaDecl(C) => Placement(C) = Ordinary
MetaDecl(C) => CaptureClause(C) = absent
MetaDecl(C) => no ExplicitClosureCapture(C)
MetaDecl(C) => no AutomaticClosureDependencyFromUnpassedOuterLocal(C)
```

Its external material must come from admitted invocation inputs and their
dependency closure, the stable definition environment already fixed by the
selected callable/parent owner, lawful instance state/members, or another
mechanism explicitly established by the meta owner with no hidden capture
coordinate. Unpassed caller/enclosing locals remain masked; material that must
affect the invocation must enter through In. CapturedEnv is not an extra input
to MetaInstanceRootKey. See the
[meta owner](../meta-invocation/meta-object-invocation-and-policy-reduction.md#2-meta-instance-identity).

This restriction applies only to the declaration layer establishing that
identity. Its body may form ordinary closures using material legally available
inside the invocation; a nested closure cannot capture a masked outer local.
An ordinary dependency-bearing closure explicitly passed through In retains its
admitted transitive dependencies and ordinary identity/lifetime checks. That
input is not a capture channel of the MetaDecl itself.

## 5. Projections of one dependency

```text
D -> Pi_sigma(D)
```

Compile evaluation may consume permitted type/Pattern information while
unreadable runtime Val1 remains in the residual continuation. Runtime does
not resolve the concrete source again.

If initialization itself contains runtime actions, their formation residue
must remain. Compile evaluation cannot fabricate an already initialized
capture value. A head, meta key or complete normalization requiring that value
must satisfy readiness and legality; unknown material is not proof of equal
Core.

Seal deferral preserves these dependencies and effects. A closure dependency
does not bypass MetaDom or SealDom.

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
It does not rerun surrounding expressions, resolve external names again or
extend a referenced object's lifetime.

The [lifetime handoff](../lifetime/lifetime-policy-and-overload-boundary.md#8-closure-dependency-lifetime-refinement-handoff)
owns persistence and move/escape refinement. This dependency relation adds no
universal lifetime theorem.
