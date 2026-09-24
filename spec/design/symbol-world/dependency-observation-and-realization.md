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
    or an established embedding read

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

## 4. Ordinary and in-place dependencies

An ordinary implementation uses its established dependency sources and semantic
realizations. Later invocation does not recapture external names by spelling.

An in-place implementation uses the observation conditions of its established
embedding position. The existence of a dependency does not give it an ordinary
capture environment. These are different uses of one dependency framework,
not separate invocation ontologies.

Both obey the same no-reopen boundary after resolution and selection.

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
