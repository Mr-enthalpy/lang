# Safety Policy and External Semantic Admission

Status: canonical semantics; source/projection integration pending.

## 1. Orthogonal safety policy

    SafetyPolicy = safe | unsafe
    PolicyMode = const | mut

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

These are ordinary rank-0 lifecycle Objects, with ordinary fields and associated
operations. They add no fourth business Object coordinate or higher-rank
constraint plane. Safe code may observe their origin, region, and color values.
A mutable copy of observation data does not by itself write semantic history.

Ordinary boundary `admit` publishes only the updates derivable from its contract
and actual invocation correspondence, under the world-update authority supplied
by that boundary. Direct admission of otherwise unobtainable facts requires
unsafe in addition to ordinary borrowing, Writable and applicable construction
premises. Neither permission follows merely from constructing an open contract
type, its omitted-mode completion, or acquiring a mutable data copy. See the
[lifetime owner](lifetime-policy-and-overload-boundary.md#9-ordinary-contract-formation-and-bootstrap-responsibilities).

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

`unsafe post` is the language-carried route for these successful-action facts.
There is no `unsafe pre` route that retrospectively legalizes an action. Its
contract is an ordinary compile expression returning the required self-rooted
type and associated behavior, not a separate axiom-expression interpreter.
Failure publishes no successful credential. Ordinary contract derivation and
trusted external admission remain distinct fact sources even when both are
consumed through the same four-way check/admit boundary protocol.

An ordinary user check returning true creates no permission to fabricate origin,
extend Region, erase Color or amplify access. Each safe adoption operation must
independently satisfy the actual mapped boundary's update premises; the world
view is narrower than an unrestricted mutable record. See the
[safe-world rule](lifetime-policy-and-overload-boundary.md#92-safe-adoption-is-narrower-than-mutable-observation-data)
and [counterexamples](lifetime-algebra-walkthroughs.md#always-true-check-cannot-forge-a-world).
Neither installing associated functions under OpenHere nor passing a mutable
copy of observation data opens the external-admission route. Unsafe Post still
requires real compatible external success, with its ordinary Pre already met.

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
one. See [evaluation and optimization](../static-evaluation/evaluation-residual-and-optimization.md).

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

## 6. Library and materializer handoff

The [library protocols](lifetime-policy-and-overload-boundary.md#12-dynamic-credentials-and-batch-acquisition)
and [walkthroughs](lifetime-algebra-walkthroughs.md) specify what a trusted
facility must establish: safeid validation binds a credential to actual names,
generations and an interval; batch share partition establishes actual disjoint
operation ranges and forms refs from the authorized original ref; an alias
window maintains validity and exclusion throughout its finite duration.
Anonymous roots, unequal path strings and equal integer values prove none of
those external facts.

The ordinary library composes those facts with Pre, consumption, boundary
compression, color intervals and option/control behavior. The materializer
must implement the actual validation and runtime protocol soundly and lightly;
it cannot replace access semantics with an address-as-integer axiom. Partbox
adds resource-specific conserved parts (accounting fractions, not `T share`
borrow instances) and ordinary consuming operations, not
a universal graph-safety theorem. The common consumer contract records these
[trusted inputs and failure boundaries](../../contracts/lifetime-program-consumers.md#lc7-unsafe-post-and-trusted-dynamic-facilities).

The [try_trust contract/body expansion](lifetime-algebra-walkthroughs.md#writing-the-trusted-apis-contracts-and-body)
separates the ordinary source/shared-dependency formation from the unsafe fact
that the actual selected probe validated this delivered id against Q/generation
and maintains it for the declared interval. Its failure arm admits no credential.
The ordinary row constructors return descriptive material for the same self-root
contract constructor; they grant no world authority. The displayed unsafe post
clause remains pending source notation, not an implemented parser/API.

Likewise the [partbox production chain](lifetime-algebra-walkthroughs.md#a-minimal-production-chain-inside-the-library)
can be wholly ordinary when the actual held box supplies sufficient validity.
An external variant may require real Q/generation validity throughout its finite
transition interval. Such a fact cannot supply missing initial ownership,
allocation distinctness/conservation, exclusion or normal ref authority. Returning
the extracted owned box needs no trusted reconstruction from resource metadata.
