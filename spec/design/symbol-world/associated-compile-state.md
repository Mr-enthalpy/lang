# Associated Compile-Time State as a Meta Invocation

Status: canonical derived instance; source integration pending. A and the member
spelling state below are notation, not frozen public callable/member spellings.

## 1. Derivation from ordinary meta invocation

[Meta invocation](../meta-invocation/meta-object-invocation-and-policy-reduction.md)
constructs an instance name whose value is its instance type. A uses this general
facility and stores an ordinary OverloadGroup in that type's Val2:

    m_A(t) = InvokeName(A, t)
    Value(m_A(t)) = tau_A(t)
    n_A(t) = state::m_A(t)
    Value(n_A(t)) : OverloadGroup
    initial group value = epsilon_OG

The group can have its ordinary type without being registered in the instance's
V_tau or Pattern. The direct meta result remains tau_A(t); the instance name is
not itself a group-valued wrapper. Ordinary navigation retrieves its payload.

A's input dependency is the existing construction subject of t:

    subject(t) = existing construction-window subject of Core(t)
    CanonicalizeInvocationInputs_A(t) retains identity(subject(t))
    m_A(t1) = m_A(t2) iff subject(t1) = subject(t2)
      for the same resolved A and parent semantic owner

The subject carries Anchor, GenerationRegime and WindowLive; it is not a new
Object. Copies preserve it, authorized updates and Close do not rename it, and
independent equal-Core formation does not merge it. Ordinary Core equality,
group bucket equality, whole-snapshot equality and TypeValueId cannot substitute
for that dependency. Incidental lexical carrier Places do not enter this key.

The general meta invocation cache retains the instance and its ordinary member
Places/current state. A needs no separate GlobalMap primitive. Sparse maps can
implement the general facility; their representation does not define A.

## 2. Instance policy and ordinary member observation

Schematic uses of the existing syntax are:

```lang
meta let instance = t |> A;
let group_value = state::instance;
let group_ref = (state::instance) ref;
```

P1 meta retains the instance under its input-derived source. Its name is its type
value, so OpenHere governs its mutation qualification and must hold before
acquiring an instance mut view. There is no additional independent const/mut gate
on that instance. Classic plain let instead completes and closes the instance;
meta let cannot reopen it later.

The state member is an ordinary group name/value. Its declared ordinary member
policy exposes mutable views while the construction source remains open.
Explicit ref selects the ordinary group borrow and checks actual Place,
capability and lifetime. Bare member observation reads a group value. Borrowing
the instance type and borrowing this group are different operations.

The local `instance` spelling does not imply place forwarding. Ordinary outer
binding may hold a snapshot in its own destination; a reference to a member of
that copied resident targets the copy. Shared persistent-state mutation must
select the actual member Place of the invocation-generated instance name.
See the [acquisition/binding trace](../meta-invocation/meta-object-invocation-and-policy-reduction.md#204-worked-example-instance-acquisition-and-outer-binding)
for repeated acquisition, copied values and saved references. P1 meta preserves
openness, not an implicit alias to a cache entry.

A hypothetical A::t would occupy t's namespace. Here:

    m_A(t) notin StructuralChildren(t)
    n_A(t) is a member of tau_A(t), not a child of t
    acquiring either name leaves Val2(t) and Norm(t) unchanged
    Close(t) => stable non-generative registered structure

No input DirectPatternChild evidence is created. Group updates also leave the
candidate types unchanged. Closing the instance fixes its non-generative
registrations and ends the declared state write window. It does not freeze every
possible ordinary generated Val2 realization; those cannot add registration or
revive this state reference's source. A's separate existence does not reopen
the input construction window.

## 3. Openness follows the input dependency

Before an explicit Close of the instance, the general output meet specializes:

    o(m_A(t)) = o(subject(t))
    OpenHere(m_A(t), Sigma) iff OpenHere(subject(t), Sigma)

The ordinary state member is constructed with this same source. This is the
member's declared dependency, not a general parent-to-child OpenHere implication.
Every group write requires its ordinary policy/capability/Place/lifetime facts
and rechecks that source. Explicit instance closure (including plain completion)
ends this instance's construction window and its associated state write window.
Input closure likewise makes their write Pre fail. Saved mut views freeze neither.

A saved group reference retains BindingPlace(n_A(t)) and its original subject
dependency. Replacing the caller's t_ref resident does not retarget it or transfer
the replacement's window. Only ordinary explicit retargeting selects another
target. Temporary authority masking is distinct from true Close.

The group remains readable after closure when ordinary lifetime and visibility
permit. Reacquisition neither resets it to epsilon_OG nor reopens it. Stable
identity proves neither global lifetime nor global mutability.

## 4. Construction logic is an ordinary callable

The group's candidates expose ordinary complete compile function objects. A
receiver invokes the explicit group value with its own construction reference:

    mut let r_ref = (mut let r::some_path:type) ref;
    r_ref = initial_complete_target_type;
    r_ref |> state::instance

Ordinary call projection selects one candidate. First self is the selected
callable object; r is a later argument. The body may inspect the target, branch,
invoke compile functions or host Objects, create intermediate Objects, or inject
several times. Source group mutation and target injection have separate premises:

    source: updating state requires its ordinary writable view and live source
    target: extending/injecting r requires its own OpenHere/Writable

Calling grants no hidden target access. Ambiguity remains ordinary ambiguity;
there is no implicit fan-out. A dispatcher can be an ordinary entry.

For an actually closed instance with exactly one ordinary Val2 entry, the
[compile extraction helper](../meta-invocation/meta-object-invocation-and-policy-reduction.md#31-ordinary-val2-extraction-and-compile-convenience)
can provide value convenience via `instance |> only_val2` or `instance only_val2`.
The helper is not an implicit conversion or an open-state mutation path. While
constructing or borrowing state, use its ordinary explicit member name.
The count observes the current finite Val2 snapshot, not all future requested
coordinates; Close alone is not a proof that the count can never change.

## 5. Effects and implementation boundary

Member Places participate in ordinary evaluation state. Siblings start from a
common snapshot with independent overlays. Commutative associative contributions
may join; conflicting replacements report ordinary unordered write conflicts.
Scheduling does not expose another sibling's new writes.

A consumes the general meta instance cache: stable identity, construction status,
current type/member observations, dependency validity and current Pre. Cache reuse
preserves effects and entry multiplicity without replaying initialization or
restoring an earlier group value. Previously copied snapshots stay immutable.

Public spelling, source definition, sparse storage, persistence and entry encoding
remain implementation work. The general meta facilities must be implemented
first; they satisfy A's needs. No A-specific capability remains to be generalized.
