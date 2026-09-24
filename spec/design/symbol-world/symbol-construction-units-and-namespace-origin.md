# Source Composition and Construction Closure

Status: canonical semantics. Physical provenance and semantic construction
authority are independent.

## 1. Physical blocks and ordinary source actions

    PhysicalTree(Level) -> normalized source actions
    directory -> Unordered{sibling file blocks, child-directory blocks}
    file -> Seq(source actions)

Sibling blocks start from a common input snapshot, produce independent overlays
and join under ordinary effect algebra. A sequential implementation must preserve
that result and cannot make a sibling's new writes available by file ordering.
main.lang anchors the explicitly selected root; it has no sibling priority.
Entry has runtime P2, and omitted ordinary P1 defaults to runtime. Bootstrap or
legal meta formation supplies stable roots, not an active meta wrapper around
all source actions.

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
    Realize(NameCoord(parent,name),P,t) -> explicit Borrow -> Initialize

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
is not canonical. Close requires retained structural names being published to be
initialized; it does not require all future generated coordinates to be realized.
Ordinary lexical let remains unchanged.

Every legally completed closure RHS returns tau_C. At the file implementation
layer, declarations install that result at the established structural root;
local blocks retain ordinary lexical let. Further contribution to the named
type's V_tau requires an established structural contribution role. Different sibling files
can contribute to that named type. Distinct entry identity survives equal values.
Ordinary lexical let and Pattern structural-child registration remain separate.

Sibling actions established as contributions to f share NameCoord(root,f)
before either is retained. Coordinate equality alone does not establish
contribution status; ordinary legal bindings and mutations retain their meaning.
Their overlays join contribution effects under the ordinary named-contribution
algebra, including the first one-shot formation. There are no competing name
identities to choose between and no file-order winner. Exclusive explicit
declarations may still conflict under realization rules. See the
[name owner](names-and-overload-groups.md#62-unordered-siblings-share-the-coordinate-before-realization).

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
      -> non-generative registered-structure closure
      -> external resolution

For foo::(t meta_call), the meta call completes before external foo resolution.
The registered structure and callspace cannot grow after closure. Ordinary
generated Val2 names may still be realized by the selected generative rules;
they do not extend Pattern structure or V_tau. Anonymous implementation
classifiers remain in /tau without giving their callable values named navigation.

True Close is irreversible under the existing open-window rules. Losing
visibility across a masking meta frame is not Close. Meta result completion
transfers only owned material under the actual result region and checks external
and borrow dependencies. Global publication additionally requires global
stability and closure of the non-generative registered structure. An inherited outer opening source is
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


## 7. File declaration installation and local binding

### 7.1 File implementation declarations have structural destinations

At implementation root r, a top-level declaration:

```text
let a = e;
```

normalizes to ordinary actions that form and install Eval(e) at the corresponding
name/member position under r. Merely binding a inside a file-local lexical
block would leave the implemented package without its members.

The structural role and destination are established at the source-to-actions
handoff. This is not a retry after failed lexical let evaluation. True local
blocks still use ordinary binding; nested lets are not all promoted to package
members.

### 7.2 Installation retains the ordinary action boundaries

```text
legal name coordinate and typed formation
-> required explicit borrow / initialization
-> ordinary member formation
-> legal Pattern/V_tau registration when the material requires it
-> extend/inject updates where applicable
```

This decomposition does not rewrite a complete lexical let into an illegal
default-type declaration followed by assignment.

```text
ordinary Val2 installation != Pattern registration
ordinary Val2 installation != V_tau registration
structural destination != permission to aggregate every same-name declaration
```

let a=uint8 installs that RHS value, without a new type wrapping tau_uint8.
Universal closure-to-type formation does not authorize automatic TypeAdd merely
because an RHS is a type value.

### 7.3 Two contexts for the bool example

At file implementation root path:

```text
let a = bool::;
```

After the applicable structural formation has completed:

```text
Read(bool::a::path) = Read(bool::)
```

Here a is an actual installation layer. The two quoted Path structures remain
different.

In a true local block the same let instead establishes:

```text
Read(a) = Eval(bool::)
```

It adds no internal layer named a and does not itself authorize extra nesting
such as bool::a. This absence of an implication does not reject a program that
independently has the required same-named structure.

### 7.4 Direct struct and incremental extend observations

When final member values, Pattern/V_tau registrations, owners/homes,
dependencies and all relevant identity observations agree:

```text
Norm(IncrementalConstruction) = Norm(DirectConstruction)
```

A later inject is not an extra component of final Pattern identity. This
equality neither erases intermediate reads, writes, errors, OpenHere checks or
lifecycle effects nor asserts that every construction history produces equal
semantic identities. All equality premises remain necessary.

### 7.5 Sibling files retain ordinary composition

```text
directory = unordered sibling blocks from a common snapshot
file = sequential actions
```

Declarations install under the shared structural root. Ordinary contribution
and conflict relations decide whether same-coordinate effects can join;
filename order decides nothing. Later contributions do not retroactively
change the observations of earlier actions within a file.
