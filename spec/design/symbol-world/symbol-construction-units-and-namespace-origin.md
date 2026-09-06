# Source Composition and Construction Closure

Status: canonical semantics. Physical provenance and semantic construction
authority are independent.

## 1. Physical blocks and ordinary meta actions

    PhysicalTree(Level) -> MetaProgram
    directory -> Unordered{sibling file blocks, child-directory blocks}
    file -> Seq(meta actions)

Sibling blocks start from a common input snapshot, produce independent overlays
and join under ordinary effect algebra. A sequential implementation must preserve
that result and cannot make a sibling's new writes available by file ordering.
main.lang anchors the explicitly selected root; it has no sibling priority.

Files and directories supply provenance, diagnostics, cache/source mapping and
scheduling information. They do not decide which name a construction owns,
which subtree can be extended, or which same-name entries may coexist.

## 2. Construction authority

Semantic construction uses the existing pattern value, anchor, evaluation
coordinate, WindowLive and authority-frame judgments. Copying a value preserves
its anchor and does not create a new open window. Writable belongs to actual
Places/references and remains independent of the value's OpenHere judgment.

A source action can create or modify a name only through the ordinary structural
target and capability rules. Physical parenthood does not imply semantic
authority. A contribution from a different file is neither automatically
authorized nor automatically prohibited by that fact.

## 3. Names and group composition

    P let name::path : mut type ref
    P let name::path = e == (P let name::path) = e

Structural let requires freshness, creates the fresh named type, records its
declaration policy and returns its mutable construction reference. Following
assignment is ordinary assignment. Declared const policy does not remove the
mutable reference needed during construction.

At a normalized named-contribution position, unqualified let name = e
contributes to the same named type's V_tau. Different sibling files
can contribute to that named type. Distinct entry identity survives equal values.
Ordinary lexical let and Pattern structural-child registration remain separate.

Pure extend produces a new complete pattern value. inject reads, extends and
writes through an actual mutable type reference. No file-level delta, owner
wrapper or cache replay grants the required premises.

## 4. Associated construction logic

The source pattern value t controls the write window of ordinary compile-global
A[t]. A receiver supplies its own mutable construction reference r by invoking
the selected ordinary compile callable in that group. Source-side A[t] mutation
and target-side r mutation satisfy their separate existing OpenHere/Writable
checks. The [associated-state owner](associated-compile-state.md) defines this
composition; no implementation-contribution protocol is necessary.

## 5. Closure and external observation

    receiver construction
      -> ordinary construction calls and writes
      -> name-set closure
      -> external resolution

For foo::(t meta_call), the meta call completes before external foo resolution.
The externally visible names cannot grow after closure. Anonymous implementation
objects remain in their /tau layer without reopening the parent namespace.

True Close is irreversible under the existing open-window rules. Losing
visibility across a masking meta frame is not Close. The ordinary meta return
seal promotes only the owned result closure and checks external/borrow
dependencies; source composition does not replace those identity/lifetime laws.

## 6. Transactions and implementation

A semantic transaction may stage ordinary state effects and commit them
atomically. File boundaries and structural let assignment do not invent a
special rollback protocol. Indices and NamespaceDelta carriers realize the
enclosing semantic actions and expose no independent authority.

The current implementation uses sorted discovery and per-declaration commits.
Common-snapshot overlays, unordered join and the new structural expression
consumer remain pending. Source files never become semantic construction owners
as an interim implementation shortcut.
