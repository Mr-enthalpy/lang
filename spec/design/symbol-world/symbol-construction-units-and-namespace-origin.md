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
legal compile formation supplies stable roots, not an active compile wrapper around
all source actions.

Child-directory basenames extend inherited navigation. Under context a,
a source let inner=rhs targets inner::a; root and filenames add no segment.
The ordinary let/Path consumer owns formation and installation. Discovery
creates no semantic owner, empty type resident or authority. [Physical normalization](../build-package/build-system-design.md)
fixes serial files and unordered common-snapshot sibling overlays.

## 2. Construction authority

Semantic construction uses the existing pattern value, anchor, evaluation
coordinate, WindowLive and authority-frame judgments. Copying a value preserves
its anchor and does not create a new open window. Writable belongs to actual
ordinary Places/references and remains independent of the value's OpenHere
judgment. The CompileInstance uses P1 open, where OpenHere governs its
mut qualification; this does not collapse policy for ordinary payload Places.

A source action creates or modifies structural names through atomic complete-type formation
target and capability rules. Compile invocation establishes its own instance/self-name before its body.
Result structure determines whether its result offers a NameExpr. Physical parenthood does not imply semantic
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
replacement capability and resident compatibility. A qualified let with RHS forms a complete binding using RHS inference. Close requires retained structural names being published to be
initialized; it does not require all future generated coordinates to be realized.
Ordinary lexical let remains unchanged.

Every legally completed ordinary closure evaluation returns tau_C. Ordinary
singleton file declarations install tau_C:type at the established root; local
blocks bind it lexically. An established same-name contribution bucket instead
consumes ClosureMaterial(C_i) to form one target-anchored ordinary callable
c_i^f per declaration. Its joined materials jointly form T_f and V_T_f against
a common snapshot, without choosing a first RHS or installing standalone
tau_C_i in V_T_f. Different sibling files can contribute to that bucket.
No whole V_tau_C_i import is implied. Distinct entry identity survives equal values.
Ordinary lexical let and Pattern structural-child registration remain separate.

Sibling actions established as contributions to f share NameCoord(root,f)
before either is retained. Coordinate equality alone does not establish
contribution status; ordinary legal bindings and mutations retain their meaning.
Their overlays join contribution effects under the ordinary named-contribution
algebra, including the first one-shot formation. There are no competing name
identities to choose between and no file-order winner. Exclusive explicit
declarations may still conflict under realization rules. See the
[name owner](names-and-overload-groups.md#62-unordered-siblings-share-the-coordinate-before-realization).

Ordinary * composes complete type values; *= updates through an actual
mutable type reference under read-transform-write semantics. No file-level delta, owner
wrapper or cache replay grants the required premises.

## 4. Associated construction logic

Compile invocation establishes stable instances with ordinary self-names.
Opening sources follow actual input dependencies; arbitrary ordinary results
are supported, and only a direct self-rooted type supplies the computed NameExpr. A returns an instance type with ordinary Val2 group
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

For foo::(t compile_partner), the compile call completes before external foo resolution.
The registered structure and callspace cannot grow after closure. Ordinary
generated Val2 names may still be realized by the selected generative rules;
they do not extend Pattern structure or V_tau. Anonymous implementation
classifiers remain in /tau without giving their callable values named navigation.

True Close is irreversible under the existing open-window rules. Losing
visibility across a masking compile frame is not Close. Compile result completion
transfers only owned material under the actual result region and checks external
and borrow dependencies. Global publication additionally requires global
stability and closure of the non-generative registered structure. An inherited outer opening source is
not closed merely by retained P1 open completion; classic close let closes
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

in the ordinary installation case hands off to actions that form and install
Eval(e) at the corresponding name/member position under r. An established
multiple-closure contribution role instead consumes formation material as in
§3; it never evaluates tau_C for insertion into V_T. Merely binding a inside
a file-local lexical block would leave the package without its members.

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
-> ordinary * / *= updates where applicable
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

### 7.3 Navigation inheritance for a complete binding

```text
a/
    let inner = bool::;

=> let inner::a = bool::;
Read_resident(Read_name(inner::a)) = Eval(bool::)
```

The LHS and RHS are at the same NameExpr level. Name installation does not
wrap the RHS in another structural type. A local let binds at its lexical
destination; a file let uses the inherited navigation destination. Both retain
ordinary binding, inference, policy and transfer boundaries.

### 7.4 Direct formation and ordinary composition observations

When final member values, Pattern/V_tau registrations, owners/homes,
dependencies and all relevant identity observations agree:

```text
Norm(IncrementalConstruction) = Norm(DirectConstruction)
```

A later type update is not an extra component of final Pattern identity. This
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
