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

Child-directory basenames normalize to ordinary typed name creation, explicit
borrow, and one-shot directory type initialization before body evaluation.
The selected root and implementation filenames add no segment. This generated
action obeys the same creation and authority rules as written source; block
nesting does not install owners itself. [Physical normalization](../build-package/build-system-design.md)
defines the exact serial wrapper and unordered body law.
Physical provenance, cache/source mapping and scheduling grant no construction
permission and impose no same-name contribution prohibition.

## 2. Construction authority

Semantic construction uses the existing pattern value, anchor, evaluation
coordinate, WindowLive and authority-frame judgments. Copying a value preserves
its anchor and does not create a new open window. Writable belongs to actual
ordinary Places/references and remains independent of the value's OpenHere
judgment. The meta instance name/type uses P1 meta, where OpenHere governs its
mut qualification; this does not collapse policy for ordinary payload Places.

A source action creates or modifies structural names through ordinary structural
target and capability rules. Meta invocation constructs its own ordinary result
name without adding an input structural child. Physical parenthood does not imply semantic
authority. A contribution from a different file is neither automatically
authorized nor automatically prohibited by that fact.

## 3. Names and group composition

    P let name::path:t -> NameExpr(n)
    CreateName -> explicit Borrow -> Initialize

Initializer-free P let name:t and P let name::path:t create typed NameExpr
using lexical and structural destinations respectively, with non-Object
Uninitialized Place state. In (P let name::path), omitted :t defaults to :type,
not an existing type resident. P let name = rhs is a complete lexical binding
with RHS type inference, so that default does not apply. Value use requires initialization; explicit
ref borrows the Place using its declared type without reading. Ordinary write
initializes it using authority independent of the name's declaration policy,
including const. Successful first commit consumes that authority; saved initial
references do not grant replacement power. Later writes require ordinary
replacement capability and resident compatibility. The structural let=compound
is not canonical. Close requires externally resolvable structural names to be
initialized. Ordinary lexical let remains unchanged.

At a normalized named-contribution position, unqualified let name = e
contributes to the same named type's V_tau. Different sibling files
can contribute to that named type. Distinct entry identity survives equal values.
Ordinary lexical let and Pattern structural-child registration remain separate.

Pure extend produces a new complete pattern value. inject reads, extends and
writes through an actual mutable type reference. No file-level delta, owner
wrapper or cache replay grants the required premises.

## 4. Associated construction logic

Meta invocation constructs ordinary result names whose opening sources follow
actual input dependencies. A returns an instance type with ordinary Val2 group
member n_A(t); t supplies its source. A receiver supplies its own mutable
construction reference r when invoking a selected compile callable from that
group. Group-source writes and target r writes satisfy their independent
OpenHere/Writable checks. The general invocation registry/cache supports this
state. [Associated state](associated-compile-state.md) describes the instance.

## 5. Closure and external observation

    receiver construction
      -> ordinary construction calls and writes
      -> name-set closure
      -> external resolution

For foo::(t meta_call), the meta call completes before external foo resolution.
The externally visible names cannot grow after closure. Anonymous implementation
objects remain in their /tau layer without reopening the parent namespace.

True Close is irreversible under the existing open-window rules. Losing
visibility across a masking meta frame is not Close. Meta result completion
transfers only owned material under the actual result region and checks external
and borrow dependencies. Global publication additionally requires global
stability and structural name-set closure. An inherited outer opening source is
not closed merely by retained P1 meta completion; classic plain let closes
the instance. Source composition replaces none of these laws.

## 6. Transactions and implementation

A semantic transaction may stage ordinary state effects and commit them
atomically. File boundaries and explicit name-creation/initialization sequences do not invent a
special rollback protocol. Indices and NamespaceDelta carriers realize the
enclosing semantic actions and expose no independent authority.

The current implementation uses sorted discovery and per-declaration commits.
Common-snapshot overlays, unordered join and the new structural expression
consumer remain pending. Source files never become semantic construction owners
as an interim implementation shortcut.
