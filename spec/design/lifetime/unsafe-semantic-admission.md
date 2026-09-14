# Safety Policy and External Semantic Admission

Status: canonical semantics; source/projection integration pending.

## 1. Orthogonal safety policy

    SafetyPolicy = safe | unsafe
    PolicyMode = const | plain | mut

These are independent dimensions. mut does not imply unsafe, and unsafe does
not imply mut, Writable, OpenHere, construction authority, or valid borrowing.
Safety policy governs admission of otherwise unobtainable external semantic
knowledge, not the level of a machine operation.

## 2. Lifecycle names and fields

    N@ is a name iff N is a name

Reification preserves the operand's name character. Ordinary lifecycle value
observations and borrowed field observations remain distinct: value field
projection yields a value; a permitted borrowed field projection yields a
reference. @ itself is not a ref constructor or a conversion of every name
into an unaddressable snapshot.

Safe code may observe lifecycle origin, region, and color values. Acquiring a
mutable reference that can write back into lifecycle semantic knowledge requires
unsafe as well as the ordinary borrowing, Writable, and applicable construction
premises. A mutable copy of observation data does not by itself write semantic
history. The protected boundary is the reference back into that semantic world.

## 3. Post-directed axiom admission

Unsafe is program-specific semantic axiom admission. It adds a description of
external reality that the language cannot derive internally, subject to
compatibility with all existing facts:

    ExternalAction -> Commit -> UnsafeSemanticDescription -> PostFacts
    K' = K join DeltaK
    CompatibleKnowledge(K, DeltaK)

It cannot prove a missing Pre merely by being written. Every action still
passes ordinary Pre before commit, and Post exists only after successful
commit. Admissions do not retract established use/drop/move/cleanup history,
revive a closed construction window, or arbitrarily replace known facts.

Callers may consume established callee Post summaries without expanding the
callee's body. That modular summary rule does not let an external assertion
bypass the admission boundary or validate the preceding action's Pre.

## 4. Facts and optimizer access

    UnsafeDescription -> E_pi -> Facts_E_pi -> optimizer query

The optimizer consumes the same ordinary facts whether their provenance is
language derivation, FFI, a device, host acquisition, or unsafe description.
It has no private semantic assumptions. Different types alone establish no
non-aliasing authority. Type punning can be safe when the explicit
representation/lifetime/machine rules justify it; a high-level external
ownership description can require unsafe.

Rewritten continuations undergo all affected projection checks again. Facts
about the old continuation's names and positions are not transplanted to a new
one. See [evaluation and optimization](../meta-invocation/evaluation-residual-and-optimization.md).

## 5. Trusted semantic base and UB

Let A0 be the fixed language semantics and A(u_i) the actual axioms admitted by
unsafe site u_i in program P:

    A_P = A0 union Union_i A(u_i)
    TSB_program(P) = the unsafe sites and their admitted axiom sets

UB arises when external reality does not satisfy an explicitly admitted unsafe
axiom. Machine-defined behavior is represented in ordinary target facts; a
language omission is not another UB permission. With true premises, an
equivalent optimizer rewrite introduces no new UB. With false premises, the
formal model and reality already disagree before optimization.

Auditing should be able to trace unsafe sites to admitted facts, derived facts,
and optimization/materialization uses. This is semantic provenance, not an
additional safety type system or a count of unsafe source lines.
