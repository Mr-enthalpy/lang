# Heap stack and storage responsibility

Status: canonical resource protocol owner. All library and common-E consumers
described here are pending. Heap nodes and pool operations are ordinary Objects
and calls; host facilities are limited by the host/unsafe owners. There is no
separate heap evaluator or resource type system.

## 1. Ordinary single-linked context

The [host owner](../static-evaluation/host-capabilities-and-machine-objects.md#6-global-io-and-heap-capabilities)
requires compile globalheap with a legal compile-to-runtime migration capability.
PoolInterface, HeapStack, malloc/free and descriptor organization are library
choices. A representative structural shape is:

```lang
((PoolInterface pool, (HeapStack ref next_pool) | None) HeapStack)
let heap = heap ref .. locate(pool);
```

`locate` is the canonical surface name of an ordinary library call. RHS resolves
the old heap before LHS installs its shadowing local name. Its selected contract
establishes the legal next_pool dependency. No reverse prev link or special push
instruction is implied. A with block may manage lexical scope but is not required
to locate a layer. Pools may legally subborrow/match next_pool and locally bind
heap to it when requesting replenishment. Results return through ordinary calls;
None uses the root pool's declared intercept/termination protocol. A `copy` of a
permitted handle is governed by ordinary subborrowing/clone, not pool duplication.

## 2. Allocation and release routing

`malloc(request, preference=no_preference)` is mathematical signature notation,
not conventional call source syntax. Preference is an ordinary argument. Without
preference the request first enters the current top pool; it need not physically
finish there. A pool may translate it into a different optimal replenishment
size and request that resource through next_pool. Preference can skip layers;
within a preference group propagation follows legally adjacent participating nodes.

`free` has no allocation preference. Pools use actual allocation credentials and
their management protocols to handle or legally forward release. The caller
need not retain a hand-written original pool path. Rebinding local heap cannot
retroactively change existing allocation/release responsibility. Unknown routing
or credentials are unavailable, never permission to guess an owning pool.

## 3. Historical origin and current duties

The following are distinct ordinary name/contract observations, not extra
built-in business Object fields:

```text
FormedFrom(x,P)       // immutable historical formation provenance
StorageSupport(x,P)   // current validity/management guarantee
ReleaseDuty(x,P)      // current release obligation
```

Ordinary Move preserves history under the lifetime algebra. Object relocation
obtains genuinely new storage and establishes replacement support through its
actual action. Pool submission preserves storage addresses/Places and history
while transferring current support and duties. Neither changes old Color,
origin or generations to remove a restriction.

## 4. Two relocation contracts

### Object relocate

Input is a complete owned value, actual owning/independent-storage fields,
current H, preference and an admitted outer slice excluding H's current top.
Pre requires a selected allocation contract, destination capacity/success evidence,
ordinary access/Move/ownership and all other lifecycle obligations. The operation
obtains new storage and legally moves/recursively relocates necessary fields.
Commit publishes the complete replacement with its actual storage support and
release responsibility; Post preserves historical provenance according to the
actual selected operations. It does not automatically repair independent aliases
or self-reference constraints. Allocation and failure reporting are defined by
the selected ordinary library contract, with no fictitious no-action name rewrite.

### Pool submit and heap relocate

Pool submission transfers management of **existing** storage, its validity,
release duty and consistent descriptors to an admitted outer manager. No live
object bytes or storage Places move. Direct transfer, metadata reorganization,
descriptor rebuilding, coalescing, page classification, generation sealing and
compression are possible ordinary implementation costs, subject to the contract.

```text
SubmitCap(P,Q,Pre_submit) iff
  every legal Sigma satisfying Pre_submit admits guaranteed Submit(P,Q)

Pre_submit(Sigma,P,Q):
  selected actual capability, credentials and destination acceptance;
  complete allocation/support/duty/descriptor correspondence;
  ordinary alias/access/cleanup/lifecycle legality;
  guaranteed worst-case metadata and other commit resources
commit Submit(P,Q): atomically transfer support + duty + descriptor responsibility
Post_submit:
  all covered live Places unchanged; FormedFrom unchanged;
  StorageSupport(x,Q) and ReleaseDuty(x,Q) for every covered x;
  no covered duty remains orphaned or multiply owned
```

Capability is binary. A pool either supplies this guarantee within declared Pre
or offers no submit capability. A legal submit has no hidden runtime failure
caused by temporary metadata allocation. Trusted facilities must implement a
feasible worst-case resource guarantee, not assume an infinite global heap or
probe at runtime and call that a strong guarantee. Missing proof is unavailable;
proved absence rejects submit before mutation. Unsafe cannot waive its Pre.

```lang
heap relocate;
```

This consumes the current layer using guaranteed submit. On successful common
commit, its shadowed old heap binding becomes available again, reducing the
tracked local chain by one. The binding transition belongs to this ordinary
consuming protocol, not a general rule reviving arbitrary moved bindings.
External aliases and allocation dependencies still obey their actual contracts.
No resource is resurrected, and merely deleting a name is not a submit.

## 5. Relative regular storage paths

With current H and outward successor up(H):

```text
no-preference object relocate: requested source = up(H)
k such relocates: requested source = up^k(H)
preference pi: StorageSource_pi(x) in {up^k(H) | k in L_pi}
```

The path describes the ordinary outer allocation request/support relation, not
a guarantee that a pool cannot replenish from elsewhere. No-preference relative
path is exact even when H's object belongs to the caller. Preference may leave
the depth unknown in a finite regular descriptor. An outward Move may be rejected
if its contract cannot exclude a remaining local-pool support dependency.

```text
LocalLayers(K)=L and every P in L has completed valid Submit
  => no StorageSupport/ReleaseDuty in the covered submitted range
     obstructs return solely by dependence on a member of L
```

This is local-layer elimination for the committed range. It requires actual
complete submissions, not guessed depth, and grants no exemption from other
origin/reference/access/Color/escape constraints. A concrete runtime path depth
is not required to prove elimination of all local layers. Uncovered allocations
or unsubmitted layers retain their ordinary obligations.

## 6. Composition, stages and acceptance

Storage responsibility submission and compile/runtime migration are independent
chains. Each selected action proves its own recursive contract; globalheap's
migration capability supplies neither local submit nor arbitrary external-object
migration. Heap and IO context use the dependency owner's RequiredCapture or
InitializedCapture mechanism, with ordinary name shadowing and no default effect.
IO need not have a stack shape.

The C/H/L/A/S acceptance rows require positive and negative routing, identity,
guarantee, non-derivability, authority uniqueness and no-reopen cases. Concrete
pool algorithms and helper names remain library choices. Missing source protocols
return the spanned unavailable diagnostic at their ordinary consumer; a supplied
credential fixture or standalone checker cannot count as implemented execution.
