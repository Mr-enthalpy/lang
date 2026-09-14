# Namespace Projection of Meta Evaluation

Status: canonical semantic handoff; consumer alignment pending.

## 1. Source actions create the world

Physical normalization supplies serial file blocks and unordered sibling
blocks. Child-directory basenames become ordinary typed-name creation actions,
explicit Place borrows, and ordinary one-shot directory type initialization,
followed serially by the directory body under that reference. The selected root and filenames add no
segments; [physical normalization](build-system-design.md) owns the desugaring.
Their ordinary meta evaluation creates Objects, names, Places, and
semantic owners. Namespace indices are projections of those committed actions.

    normalized meta action
      -> ordinary resolution and invocation
      -> Pre
      -> semantic commit
      -> Post
      -> namespace/index projection of the committed state

Graph allocation, parent edges and cache restoration cannot manufacture
construction authority, semantic equality or name occupancy.

## 2. Names and contributions

A structural name may hold any declared resident type; an initialized :type name
denotes a named type. An existing name with no callspace
contributions still shadows an outer same-spelled binding. Freshness is
authoritative occupancy, independent of view filtering.

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
initialized, not every future generated coordinate realized. Ordinary lexical
let remains unchanged. See [name semantics](../symbol-world/names-and-overload-groups.md).

Sibling contributions join according to ordinary type-contribution/effect algebra.
Conflicting writes are not repaired by file order. File ownership, one-type-slot
restrictions and package mounts provide no additional admission rules.

## 3. Resolution and visibility

Resolve a name once, read its resident (a named type in the :type case), then apply
the consumer projection.
An explicitly held OverloadGroup uses its own candidate projection.
Every explicit navigation host retains its own ordinary view/visibility facts.
Export and public/private policy remain source-defined language relations;
their filtering does not establish freshness or change member mode.

Access paths may be represented by redirect/index edges when they encode
ordinary source-established navigation. Such an edge changes neither the
target's identity nor its authority. A configured mount is not a source action.

## 4. Construction closure

Existing OpenHere, Writable, reference validity and construction authority govern
ordinary construction mutation. External navigation follows completion of the
selected meta call. Close freezes non-generative registered structure, while
ordinary generated Val2 results may still be realized without either registration.
Anonymous classifiers stay under /tau; this grants no navigation to their values.
Associated compile state closes under its source pattern value's existing
window; target injection independently requires the receiver's open reference.

Graph state reflects these facts; a graph seal is not another authority that
can close or reopen semantic construction. Representation transactions preserve
the enclosing evaluator transaction, including its actual failure behavior.

## 5. Consumer obligations

Persisted indices preserve semantic owner/root identity, member entry identity,
host chains, policy views and provenance separately. They must not turn file
identity, registry allocation order or a package graph into semantic identity.
Concrete persistent encodings remain open.
