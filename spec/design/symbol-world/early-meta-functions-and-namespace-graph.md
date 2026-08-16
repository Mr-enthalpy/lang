# Early Meta-Functions and the Namespace Graph

**Status: Non-normative future design. This is the post-v0.5 roadmap track
(v0.6–v0.8). A partial v0.6 vertical slice is implemented in `lang_build`;
the remaining v0.6–v0.8 design is future work and is not a parser/normalizer
rule.**

This document is the canonical direction for the v0.6–v0.8 sequence:

- v0.6 — Build / Namespace Graph Bootstrap
- v0.7 — Early Meta-Function Bootstrap
- v0.8 — Compile / Symbol Construction Interpreter Bootstrap

It builds on, and does not replace, the build/package architecture in
`spec/design/build-package/build-system-design.md`, the assembly pipeline in
`spec/design/build-package/namespace-assembly-v0.md`, and the manifest surface in
`spec/design/build-package/package-manifest-v0.md`. The later pattern-space / extraction-chain
semantics remain a separate track in
`spec/design/patterns-overload/static-pattern-spaces-and-extraction-chains.md`.

This document records the **current** build / namespace graph / early-meta
bootstrap track (v0.6–v0.8) and its narrow implemented slice. The **future**
unified invocation semantics — one policy-governed callable-invocation model
covering ordinary functions, meta functions, verification, control predicates,
operators, and type constructors, together with partial/strict meta reduction
and residualization — are specified in
`spec/design/meta-invocation/meta-object-invocation-and-policy-reduction.md`.

The canonical future symbol-facet, `compile` / `meta`, pattern-owner,
`struct`, pure `extend`, and place-level `inject` boundaries are specified in
`spec/design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md`.
That document supersedes the older formal return-slot split between `r = ...`
and `r === ...`, the interim single-form `r = ...` reading (the final model
distinguishes fresh-member creation, existing-target write, and delivery — there
is no alias-member event), and the transitional idea that a binding destination
determines the final `struct` owner identity.

Namespace-facet origin, source/meta construction-unit ownership, physical
directory contribution authority, and current cross-file closure are canonical
in
`spec/design/symbol-world/symbol-construction-units-and-namespace-origin.md`.

Layered symbol policy, final callable `P1` / `P2`, compile-flow projection,
compile companions, and automatic require are canonical in
`spec/design/symbol-world/symbol-policy-and-compile-flow-projection.md`.

## v0.7 implementation additions

v0.7 introduced early policy-aware resolution; the current branch retains five
flat compatibility flags:

- `PolicyFlag::Export`, `PolicyFlag::Meta`, `PolicyFlag::Compile`,
  `PolicyFlag::Seal`, `PolicyFlag::Runtime`
- `PolicySet` — bit-set of flags carried on `PolicyMetadata.policy_set`
- `PolicyEnv::{OpenStatic, SealStatic, Runtime}` — the compatibility resolver's
  three phase views. These do not grant permission to enter a callable body or
  scan the pre-seal world.

`PolicyFlag::Export` is legacy transport only. Canonical export-root and
public/private visibility are independent typed dimensions.

### Source verification forms

`lang_build` also contains a source-driven fixture verification loop. `verify`
is bootstrapped as a core meta-visible verification namespace/object, with
verification operations installed below it as core meta-function symbols.
Ordinary normalized expressions are treated as verification forms only after the
verification entry and operation resolve through the namespace graph under
`PolicyEnv::OpenStatic`. They do not install symbols, produce runtime objects, or add
parser syntax.

The current fixture spelling is a compact expression-chain form such as:

```text
verify exists T;
verify kind T type;
verify field_names T a b;
verify kind a::T field_function;
verify policy a::T meta;
verify not_policy a::T export;
verify body_entry_policy a::T runtime;
verify not_body_entry_policy a::T meta;
```

Verification failures are hard build diagnostics with the stable prefix:

```text
source verification error:
```

The operation dispatch is based on the resolved core symbol payload/primitive,
not on a Rust-side fixture-only string table. This verifier is a test/fixture
observation layer for namespace graph, resolver, early-meta, field-function, and
policy facts. It is not a general meta interpreter, full policy checker, type
checker, macro system, runtime lowering step, or user-facing import/export
mechanism.

Future automatic return normalization will likewise use the formal meta
invocation and policy-aware `Error` handler lookup, but it is not part of the
current early-meta slice; see
`spec/design/mechanical-lowering/mechanical-return-normalization-and-error-policy.md`.

Future formal meta object invocation should likewise share the `normal` / `tco` /
`loop` call-mode vocabulary; meta functions have no loop core and express
repetition by recursion. The current `struct` / `verify` slice does not implement
call modes; see `spec/design/mechanical-lowering/call-modes-recursion-and-tail-lowering.md`.

### Policy flag assignment

| Symbol source | Policy set |
|---|---|
| Core namespace symbol | `{export, meta, runtime}` |
| Namespace symbols (declared, physical, dependency mount, generated) | `{meta, runtime}` |
| Core meta-functions (`struct`, `assert`) | `{export, meta}` |
| Core verification namespace and operations (`verify`, `verify::exists`, …) | `{export, meta}` |
| Core built-in types/ranks (`uint8`, `type`, `namespace`, `ref`, `share`, …) | `{export, meta, runtime}` |
| Source-contributed ordinary value placeholders | `runtime` |
| Source-contributed type-annotated placeholders (`: type`) | `{meta, runtime}` |
| Struct-generated `TypeObject` | `{meta, runtime}` |
| Transitional projection namespace symbols (`ref`/`share` under a generated type) | `{meta, runtime}`; implementation substrate only |
| Generated field-function Symbol and its value/ref/share candidates | `{meta, runtime}` |
| Alias symbols | `runtime` (not transparent for early meta yet) |

Generated `struct` expansion currently assigns these transitional metadata
fields:

| Generated object | Current metadata |
|---|---|
| Generated `TypeObject` | symbol policy = `{meta, runtime}` |
| Transitional projection node `ref` / `share` | symbol policy = `{meta, runtime}`; not target semantics |
| Generated field function | symbol policy = `{meta, runtime}` |
| Generated field function | body entry policy = `runtime` |
| Generated field function | transitional return object policy = `runtime` |

These fields are implementation substrate. They do not establish final scalar
policy planes. The future model stores `Pv:Pp`, elaborates P1 as a binding/view
projection, normalizes P2 as the call-result pair, derives function-object stage
views from P2, and has no independent P3.
Return positions may nevertheless refine inherited P1 mutability only,
symmetrically with parameter refinement of P2; no other policy dimension may
change.

### Policy-aware resolver

New methods on `NamespaceGraphCapability`:

- `resolve_with_policy(…, PolicyEnv)` — filters per-component and terminal
  results; a symbol that does not satisfy the policy environment is treated as
  not found in that search root.
- `resolve_str_with_policy(…, PolicyEnv)`
- `resolve_type_object_with_policy(…, PolicyEnv)`
- `resolve_meta_function_with_policy(…, PolicyEnv)`

The compatibility APIs still filter before cross-root conflict reporting. This
is a known substrate gap: canonical resolution must first produce a symbol and
only then expose its current-phase slices. A hidden runtime value must not erase
the symbol or its compile Pattern facet.

Policy filtering is **per-component**: every path component (including
namespace intermediaries like `core`) is checked against the policy
environment. Namespace symbols that must be traversed under a policy
environment therefore carry appropriate traversal policy flags. For v0.7,
the compiler-seeded `core` namespace symbol is assigned
`{export, meta, runtime}` so that explicit paths such as `struct::core`
and `uint8::core` resolve correctly under `PolicyEnv::OpenStatic`.

`PolicyEnv::OpenStatic` is lookup visibility, not body-entry permission. The
static evaluator may enter only a fully admissible meta/compile callable and
may not read a runtime value.

### Early meta expansion uses PolicyEnv::OpenStatic

- `try_expand_early_meta_initializer` resolves the call target via
  `resolve_meta_function_with_policy(…, PolicyEnv::OpenStatic)`.
- `parse_field_expr` resolves field type names via
  `resolve_type_object_with_policy(…, PolicyEnv::OpenStatic)`.

## Implemented vertical slice (v0.6 partial)

The current implementation is intentionally small but uses the intended world
model boundary:

- `crates/lang_build` defines `CompilationWorld`, `NamespaceGraphSnapshot`,
  `NamespaceDelta`, `NamespaceNode`, `SymbolObject`, `SymbolId`,
  `NamespaceNodeId`, `SourceCategory`, `PolicyMetadata`,
  `VisibilityMetadata`, `Provenance`, `Diagnostic`, `SyntaxObject`, and
  `MetaExpansionResult`.
- Graph mutation goes through clone-on-write `NamespaceDelta` installation.
  Successful install applies the whole delta; conflicts reject the whole delta
  and return diagnostics.
- The API-level `BuildManifest` supports package name, source roots, namespace
  root, dependency mount placeholders, and a default compiler-seeded core mount.
  There is no manifest file parser yet.
- Source collection builds physical namespace skeletons from directories.
  Implementation file names remain source-fragment names only and do not
  contribute namespace segments.
- Core bootstrap installs `struct`, `assert`, `type`, `namespace`, `uint8`,
  `ref`, and `share` as `SymbolObject`s in the namespace graph. `struct` is the
  current compiler-defined `BuiltinPrivilegedAstMetaFunction` slice; `assert`
  is a compiler-known verification callable. Parser and normalizer do not
  special-case either name, and users do not gain authority to define new
  privileged AST meta functions.
- Declaration harvesting supports the narrow top-level direct-child form needed
  by the slice, especially `let T: type = ...`. Ordinary file contributions
  that attempt parent-to-descendant injection are rejected.
- Early meta lookup resolves the call target through the graph. The only
  accepted source form is currently equivalent to
  `(uint8 a, uint8 b) |> struct`; field types resolve through the same graph and
  field binders are private `struct` checker material.
- Resolver contexts distinguish current namespace lookup, explicit mounted paths
  such as `uint8::core`, and short-name default mounts such as `uint8`.
- Successful `struct` expansion currently produces a placeholder type object and
  legacy per-observation field symbols. The target model replaces those
  `a::ref::T` / `a::share::T` transport nodes with one associated Symbol `a`
  containing candidates whose receiver formals are `T`, `T ref`, and `T share`.
  These field-function candidates are visible under `PolicyEnv::OpenStatic`
  because their compatibility symbol policy is `{meta, runtime}`, but their callable
  body-entry and return-object policies are runtime-only.
- PR #94 adds an explicit pattern-head attachment helper with generated,
  global, namespace, and local categorical contexts. Formal `struct` uses the
  `GeneratedTypeDefinition` fallback; ordinary binding preserves attached
  provisional heads or restores stripped heads under that same anonymous
  fallback. It does not derive a context from the destination. This is
  transitional `PatternHeadId` registry substrate, not final
  `ResolvedPatternScope` owner resolution.
- Namespace child lookup is role-aware. Object/function symbols and pure
  namespace subspaces can share the same textual child name. Terminal lookup
  without an expected role reports ambiguity when both roles are present.
- Failed `struct` expansion returns hard diagnostics and leaves no partial
  generated subtree. Duplicate fields, unknown field types, unit/trailing-unit
  fields, and unsupported nested products are rejected. Fields named `ref` or
  `share` are allowed as ordinary associated Symbols. Current legacy projection
  nodes are implementation substrate, not the reason for their legality.
- v0.8 ordinary initializer evaluation can materialize
  `let T: type = uint8` as an ordinary type-value binding: resolve
  `symbol(uint8)`, read its type value, and bind that value to fresh
  symbol/place `T`. This is not symbol aliasing and does not canonicalize
  namespace injection targets.
- The current slice does not implement `NamespaceOrigin`, source/meta
  construction-unit ownership, physical contribution authority, cross-file
  reopening diagnostics, or meta return pure-role self-root checking.
- v0.8 source-declared callable overload selection reports structured failure
  kinds. Initializer MetaPartial residualization is driven by those kinds, not
  by diagnostic message text. Ambiguity remains a hard diagnostic; no
  meta-visible candidate and body-entry mismatch may residualize only at legal
  MetaPartial initializer boundaries.
- Selected source meta body evaluation remains narrow. Simple forwarding and
  delete diagnostics are supported. Local-let initializers may be checked under
  MetaStrict, but a selected-body parameter/local binding environment is not
  implemented; such cases report `UnsupportedSelectedMetaBodyLocalBinding`.
- Policy and visibility are metadata slots only. No policy checker, type
  checker, resolver overlay, overload merging, package solver, lockfile, or
  general meta interpreter is implemented.

## 1. Why these stages come first

The source language has no `import` / `use` / `include` / `module` syntax. Every
navigable name is a path into a **namespace graph** assembled by the build /
package layer. Therefore the build system is not an optional external tool; it
is the core infrastructure that produces the symbol graph the language reads.

Early meta-functions (`assert`, `struct`, …) are not parser/normalizer special
forms. They are symbol objects resolved through the same resolver path as any
other symbol. So early meta-functions depend on the namespace graph existing
first. The dependency chain is:

```text
build system / package layer
  -> namespace graph
       -> NamespaceGraph Capability Layer
            -> resolver (returns SymbolObject)
                 -> early meta-function lookup (assert, struct, ...)
                      -> meta expansion (MetaExpansionResult)
                           -> compile / symbol construction interpreter bootstrap
```

## 2. NamespaceGraph Capability Layer

The first build-system version must not be a CLI-only package scanner. It must
expose a capability layer that the resolver, the meta interpreter, and (later)
the type checker all share. The capability layer abstracts the namespace graph
behind named operations, at least:

- **resolve** — resolve a navigation path to a `SymbolObject`.
- **declare** — introduce a declared symbol under a node.
- **inject child material** — add a direct child to a contribution or
  construction under a legal owner (see §4). The future source-level `inject`
  built-in is functional and returns a new ordinary `type` pattern value; only
  the outer binding/assembly layer installs the resulting delta.
- **open virtual node** — open a virtual namespace node for generated structure.
- **install namespace delta** — apply a set of physical, declared, or generated
  contributions as one unit under a legal node.
- **canonical meta instance key** — compute the stable identity of a meta
  instantiation (see §6).
- **diagnostic** — attach a diagnostic with provenance to a node / operation.
- **assert / hard error** — raise a compile-time hard error.

Capabilities may be stubbed in early versions, but the **surface** must be the
shared one above, not narrowed to package scanning. There is no name-forwarding
alias capability in this layer: a second name for one place is not expressible,
and shared observation is a borrow view of one target place
([`type-values-places-and-borrow-views.md`](type-values-places-and-borrow-views.md)
§5).

### SymbolObject

The resolver returns a **SymbolObject**, not a string path. A `SymbolObject`
carries the resolved identity, its source category (§3), its node kind, and its
provenance / diagnostics. Source code navigates names; the resolver answers with
objects, so later phases (meta lookup, type checking) operate on objects rather
than re-parsing path strings.

`SymbolObject` is the current implementation substrate. The final conceptual
model is one Symbol with one `SymbolId`, one `PlaceId`, an optional pure role
member `Q`, and heterogeneous typed value-member buckets:

```text
path/name -> Symbol -> context-directed role/member projection
```

Namespace projection selects `Q`. When `TypeRole(Q)`, type projection closes it
with its direct TypeMember partition as the complete immutable snapshot
`tau = <Q,V_T>`, optionally written `bind alpha.<Q,V_T[alpha]>`.
Current namespace/type facet buckets may cache
derived views but do not define independent semantic Objects or another copy of
`V_T`.

This document does not require an immediate Rust refactor. It does require that
new design text avoid treating namespace/type/value/callable as disjoint
first-resolution result categories.

Policy metadata (see `spec/design/policy-capability/policy-visibility-symbols.md`) should be
reserved as a slot on `SymbolObject`, the context, and the capability layer, but
full policy inference / projection / checking is deferred to later stages. v0.6–
v0.8 only need the architectural placeholder, not an implementation.

## Namespace Graph World Model Invariants

v0.6 must model a compilation world, not a temporary file index.

The namespace graph should be treated as a persistent, diagnosable,
eventually serializable world object. The first implementation may keep it in
memory, but the architecture must preserve the possibility of caching, diffing,
provenance queries, IDE integration, and later graph freezing.

Avoid language such as “scan files and build a map” unless explicitly framed as
an implementation detail below the world model.

Preferred terms:

```text
CompilationWorld
NamespaceGraphSnapshot
NamespaceDelta
SymbolObject
Provenance
Diagnostic
GraphPhase
```

### Snapshot + transaction delta discipline

Namespace graph mutation must be transaction-shaped.

Passes should not freely mutate the graph in place. They should produce deltas
that are either installed atomically or rejected atomically.

```text
BaseGraph
  + DeclaredSymbolDelta
  -> DeclaredGraphSnapshot

DeclaredGraphSnapshot
  + MetaExpansionDelta
  -> MetaExpandedGraphSnapshot
```

Deltas should carry: intended parent node; declared/generated symbols; aliases;
provenance; diagnostics; policy metadata slots; cache-key fragments where
applicable.

Failure rule: failed delta installation installs nothing; diagnostics remain
available; no half-generated namespace subtree is left behind. This is
especially important for early meta-functions such as `struct`, because
`assert` failure inside a meta-function must not leave a partial
type-associated namespace.

### Conflict policy: conflict is error

The default v0.6 conflict policy is conservative: conflict is a hard error.

Do not introduce merge semantics, overlay semantics, duplicate acceptance,
overload-set merging, or identical-alias coalescing unless a later specification
explicitly permits it.

Default conflict rules:

```text
same parent + same textual name + same child-name role:   hard error
object with namespace_node + namespace subspace with the
  same name in the same parent:                           hard error for now
physical directory name vs namespace-capable declared
  object with the same name:                              hard error
two non-merge-declared object symbols with the same name
  in the same namespace:                                  hard error
two type object symbols with the same name in the same
  namespace:                                              hard error
two alias object symbols with the same name in the same
  namespace:                                              hard error
generated object symbol colliding with another object
  symbol of the same name:                                hard error
core/prelude alias colliding with user declaration:       hard error unless a
  later explicit shadowing rule is specified
overload-set merging:                                     not a v0.6 default
package overlay:                                          not a v0.6 default
```

The allowed cross-role case is intentional:

```text
object/function child without namespace_node
+ namespace-subspace child with the same textual name
= allowed
```

This is the conservative current v0.6 bucket model. The canonical future
Symbol model permits one Symbol's `V` buckets to contain multiple heterogeneous
value entries. That future same-symbol member rule is not the same
as silently merging two independently declared `SymbolObject`s today; it
requires role/member-aware declaration identity and conflict checking first.

This coexistence facility is generic graph substrate. It is not a target
semantic requirement for fields named `ref` or `share`; those are ordinary
associated Symbols and borrow observation kind belongs to candidate types.

If the implementation needs temporary permissiveness, it must be marked as an
implementation limitation, not as language semantics.

### Symbol identity is not a string path

Resolver input may be a path-like navigation form, but resolver output must be a
`SymbolObject`, not a string.

v0.6 should reserve identity categories such as:

```text
PhysicalSymbolId
DeclaredSymbolId
VirtualSymbolId
MetaInstanceSymbolId
GeneratedChildSymbolId
AliasSymbolId
```

The exact representation is future work, but the architecture must not collapse
symbol identity into a raw namespace string.

A `SymbolObject` should preserve slots for:

```text
id
name
kind
source_category
node_kind
parent
policy_metadata
visibility_metadata
provenance
diagnostics
generation_origin
cache_key_fragment
```

Most of these may be placeholders in v0.6. The point is to avoid later
retrofitting them into an underspecified map.

### Core bootstrap boundary

Core symbols may be compiler-seeded in the first implementation, but
conceptually they must still enter the namespace graph as ordinary
`SymbolObject`s.

Allowed bootstrap magic:

- compiler may ship or seed a built-in `core` package artifact;
- build system may mount `core` by default;
- `struct`, `assert`, `type`, `namespace`, `uint8`, `ref`, `share` may initially
  have built-in payloads;
- those symbols must still be installed into the namespace graph and resolved
  through the resolver.

Disallowed bootstrap shortcuts:

- parser special-cases `struct`;
- normalizer special-cases `struct`;
- type checker searches raw string `"struct"` outside resolver;
- early meta executor bypasses `SymbolObject`;
- core symbols are globally visible through ambient installation state rather
  than explicit graph mount.

### Meta expansion is atomic

`MetaExpansionResult` is the current transaction-like transport between
invocation and binding.

It may contain:

```text
replacement_object
namespace_delta
diagnostics
provenance
cache_key_fragment
```

Atomicity rule: success installs the replacement and namespace delta as one unit;
failure installs no generated symbols; diagnostics are retained; partial
type-associated namespace construction is forbidden. This applies to `struct`
and later compile/meta construction callables.

The final boundary is sharper:

```text
compile -> PatternValue, with root conservation and no root authority of its own
ordinary meta -> symbol PatternValue, plus authority to establish and seal one
                 navigable MetaInstanceRoot M
privileged builtin -> PatternValue under its member-specific owner rule
let binding / namespace contribution -> NamespaceDelta atomic install
```

The `compile` / ordinary-`meta` authority difference remains within the ordinary
PatternValue domain and introduces no construction rank; see
[`symbol-first-meta-construction-and-pattern-injection.md`](symbol-first-meta-construction-and-pattern-injection.md)
§4.1.

Formal `struct`, formal meta invocation, and pure `extend` do not install the
graph. `inject` only writes an already existing type slot and creates no graph
member/root. `MetaExpansionResult` may remain an implementation adapter, but it
must not erase the distinction between an uninstalled construction value and
the outer installation operation.

### Phase names and freeze points

v0.6 does not need to implement all later phases, but it should reserve phase
vocabulary for future seal / policy / cache behavior.

Suggested phase names:

```text
BuildGraph
ParsedFragments
DeclaredGraph
EarlyMetaExpandedGraph
TypeCheckedGraph
FrozenGraph
SealGraph
RuntimeArtifact
```

v0.6 likely reaches only the early graph phases. The purpose of naming later
phases is to prevent future seal / policy designs from inventing a separate
graph model.

### No bypass rule

Every future component that needs symbols must go through the shared namespace
graph world model.

This includes:

```text
resolver
early meta-function lookup
struct
assert
type checker
policy checker
seal stage
IDE index
cache layer
diagnostics
later HIR lowering
```

Do not let any component build its own parallel symbol table except as a derived
cache with a clear invalidation relation to the canonical namespace graph
snapshot.

### v0.6 test philosophy

When implementation begins, v0.6 tests should target invariants rather than
feature demos.

Test targets should include:

- no source-level `import/use/include/module`;
- file names do not contribute namespace segments;
- directories contribute physical namespace skeleton;
- source fragments root contributions at direct children and own any complete
  new subtree included in the same source construction;
- cross-file reopening and parent-file injection into physical child
  directories are rejected;
- all name conflicts are hard errors by default;
- resolver returns symbol objects, not strings;
- core symbols resolve through namespace graph;
- missing mount is a build/resolver error;
- meta expansion delta is atomic;
- failed `struct` expansion leaves no partial generated subtree;
- flat flags projected through OpenStatic/SealStatic/Runtime resolver views are
  implemented, while
  full policy checking and callable execution checking remain deferred.

## 3. Symbol source and child-role model

A node in the full namespace graph may be a **physical** node, a **declared**
node, or a **virtual** node (see `build-system-design.md` §7). On top of that
node-kind model, textual child names are partitioned by role.

### 3.1 Role-aware child buckets

A namespace node's child table is conceptually:

```text
textual child name -> {
  object/function role,
  namespace-subspace role,
}
```

The same textual name may appear once in each role. Same-role duplicates remain
hard conflicts. An object that is itself namespace-capable, for example a type
object with a type-associated namespace, may not currently share a textual name
with a namespace subspace in the same parent because intermediate traversal
would be ambiguous. This conservative rule can be revisited after resolver
expectation APIs stabilize.

Role assignment:

```text
FieldFunction, MetaFunction, Alias, Placeholder -> object/function role
Type                                           -> object/function role,
                                                  namespace-capable through
                                                  its type-associated namespace
pure namespace symbols for physical/declared/
virtual namespace nodes                       -> namespace-subspace role
```

Resolver terminal lookup must therefore be expectation-aware. `AnyUnique`
lookup fails if both roles are present. `FieldFunction` selects the object role
when it is a field function. `NamespaceSubspace` selects the namespace-subspace
role. Intermediate path components are resolved as `NamespaceCapableParent`.

### 3.2 Symbol source categories

Child role is distinct from source category. A namespace subspace may come from
physical directory hierarchy, declared namespace assembly, or a virtual
meta-instantiation layer. An object may be declared, generated, aliased, or
core-bootstrapped. Conflict policy applies to `(parent, textual name, role)`,
then applies the conservative namespace-capable cross-role restriction above.

### 3.3 Type-associated namespace

A **type-associated namespace** is the namespace space associated with a type
object. It holds the type's companion symbols, for example generated field
functions, layout metadata, pattern interfaces, and related companion symbols.
Borrow observation kinds are candidate types, not `ref` / `share` projection
subspaces.

A type-associated namespace is **not** simply a "declared namespace object". Its
members may be **declared**, **generated**, or **virtual** depending on origin.
For a `struct`-generated type, the type-associated namespace is a virtual /
generated child namespace attached to the type node.

What unifies the category is the **role** (companion space of a type object),
not the origin of its members.

An ordinary let-shaped declaration consumed inside `struct` construction is a
pending contribution to this companion space:

```lang
let name = expr
```

It contributes the initializer's ordinary Val2 value entries, including
callable values, without adding a structural Val1 slot or Pattern extraction
member. It is not restricted to `Pv=absent`. `let () = impl` is the special
current-owner call-entry target. These contributions remain graph-uninstalled
until the outer struct construction is bound and its `NamespaceDelta` commits.

For `struct`-generated data fields, the generated associated callable is the
unary special case. The first-class `.name` closure is more general: after its
caller/self (the field-function object) is injected, it dispatches through the first explicit
argument type and may forward a normalized remainder product as additional
arguments.

The unary generated-field shape is one same-name associated Symbol:

```text
field : (object: T)       -> field
field : (object: T ref)   -> field ref
field : (object: T share) -> field share
```

Target exposure follows the general structural predicate:

```text
RuntimeField(f)
  <=> Val1_f != absent
    and Materializable_0(Val1_f)
    and not RequiresStaticPattern(f)

Stage(accessor(f)) = runtime || compile  if RuntimeField(f)
Stage(accessor(f)) = compile             otherwise
```

Type-valued fields are compile-only only because they currently fail this
predicate, not because “type/PatternValue field” is a separate category. The
generated assignment partner exists only when the ordinary field Policy admits
mutation. Current compatibility fields remain transport and do not override
these target semantics. In a plain use context generated candidates obey
`succ_plain: let > const = mut`; tied `const` and `mut` candidates remain
ambiguous when no plain `let` candidate exists.

The value candidate has value semantics (`T == T move`). Borrowed field access must begin
from an explicit borrow form such as `val ref.field1` or
`val share.field1`. Field access evaluation, borrow normalization, and
access-tree construction are future work.

Fields named `ref` or `share` are valid ordinary associated Symbols. There is no
generated projection namespace with which they can conflict; receiver type and
Policy select among the same-name candidates.

### 3.4 Type values, symbol places, and aliasing

Type-value evaluation, symbol/place identity, and namespace injection targets
are distinct. A type/rank use evaluates by value:

```text
let T: type = uint8
```

means `T` is a new symbol/place whose value is the existing type value `uint8`.
`value(T) == value(uint8)` holds, but `place(T) != place(uint8)`.

This mirrors ordinary value bindings:

```text
let a = 1
let b = 1
```

`a` and `b` are distinct symbols, while their values are equal.

Member creation is not pure type-value evaluation.
Because this `T` is already a pure type slot, `let f::(T |> (type ref)) = ...`
explicitly targets `place(T)`, not `place(uint8)`. Type-value equality must not
canonicalize injection targets.

`let` and the frozen `===` surface form are not interchangeable, and only `let`
has target semantics:

| Form | Symbol effect | Type-value effect | Extension-place effect |
| --- | --- | --- | --- |
| `let T: type = uint8` | Creates new symbol/place `T` | `value(T) == value(uint8)` | `let f::(T |> (type ref))` may create under `place(T)` when separately authorized |
| `let T === uint8` | Frozen parser surface only; **no target semantics** — the alias/forwarding direction is retired | — | — |
| `let T = ... \|> struct` | Creates new symbol/place `T` | `value(T)` is a generated Symbol with `Q_struct` satisfying `TypeRole` | `let f::((T ref).type)` may create under that explicit type-member place |

Fresh generated Symbols own/provide their `Q_struct` type-role member's associated
namespace, so `let T = (uint8 a, uint8 b) |> struct` creates one associated
Symbol for `a` and one for `b`; each contains value/ref/share receiver candidates.

By contrast, `let T: type = uint8` does not create a fresh type value, but it
may own a fresh current-level companion namespace place. Future namespace
injection through `T` targets that place; future type/rank evaluation of `T`
returns the existing type value `uint8`.

Future generic/meta symbol constructions such as `(int Vec::std)` expose stable
type-facet values after binding. Therefore:

```text
let A: type = (int Vec::std)
let B: type = (int Vec::std)
```

means `A == B` by ordinary type-value equality (default `Core(tau)=Q`,
first-order `TypeValueId`) while `A` and `B` remain distinct symbols.
No declaration form can make them the same symbol or the same place. The
whole-snapshot identity is `Addr(Norm_type(tau))`, used to tell shared-root
snapshots apart in transport and in positions the language has independently
frozen to whole-snapshot semantics; carrying that settled relation into all
remaining semantic comparison sites is future work.

See `spec/design/symbol-world/type-associated-function-objects-and-access-trees.md` for the
field-function and access-tree implications. The intended final distinction
between type values, symbol places, borrow views, and writable extension
targets is documented in
`spec/design/symbol-world/type-values-places-and-borrow-views.md`.

### 3.5 Final role inclusion and value-member boundary

The role-aware `SymbolObject` buckets above describe the current substrate. The
target model uses Object judgments:

```text
Pure(x) <=> NamespaceRole(x)
TypeRole(x) subset NamespaceRole(x)
NamespaceRole(x) not-subset TypeRole(x)
```

This is not type-system subtyping. Every pure Object is navigable and therefore
has `NamespaceRole`; `TypeRole(x)` is an additional imported
judgment over pure Objects. Namespace-role-only Objects remain navigable
but cannot be used by `AsType`. `TypeFacet` and `NamespaceFacet` remain names for
current implementation buckets only, not target ontology.

Pattern-material leaves belong to `P(x)` and affect normalization, matching, and
extraction. Ordinary navigable `Val2` members do not become Pattern leaves merely
because `NamespaceRole(x)` holds. A value projection is terminal for the current
lookup; it does not prevent the same Object from owning navigable children.

The complete origin and ownership rules for these facets are in
`symbol-construction-units-and-namespace-origin.md`.

## 4. Namespace contribution rules

This bootstrap document applies, but does not redefine, the namespace-origin and
construction-unit contract in
`symbol-construction-units-and-namespace-origin.md`. The v0.6/v0.7 architecture
must leave room for:

```text
physical directory -> contribution authority
source file         -> one closed SourceConstructionUnit
meta invocation     -> one closed MetaConstructionUnit
outer assembler     -> conflict-checked atomic NamespaceDelta install
```

The current direct-child harvesting restriction is a conservative precursor,
not a complete implementation of origin uniqueness, subtree ownership, or
cross-file reopening diagnostics. Detailed physical/source/meta conflict rules,
future value-merge relaxations, and diagnostic ownership belong to the canonical
construction-unit note.

## 5. Early meta-function bootstrap (v0.7)

On the v0.6 namespace graph, the early meta-function call loop is closed so that
an early meta target is found by the **resolver**, not by a parser / normalizer
special case.

- **Early meta-function lookup** from the namespace graph (same resolver path as
  any other symbol).
- **Closed `SyntaxObject` passing** — the meta target receives a closed syntax
  object; the call process is opaque to outside observers.
- **`assert`** as a compile-time hard-check primitive.
- **`struct`** as the first real, globally visible
  `BuiltinPrivilegedAstMetaFunction` object resolved from the core namespace.
  It consumes bounded AST material through a private checker; a failure is a
  meta hard error, not a parser / normalizer error.
- **Current meta call replacement model** — the implemented slice replaces a
  meta call through a `MetaExpansionResult` adapter.
- **Current `MetaExpansionResult`** carries:
  - replacement object,
  - namespace delta,
  - diagnostics,
  - provenance.
- **Parent-to-child namespace injection rule** (per §4) — generated child
  namespaces are installed only under a legal parent / instance node; no
  arbitrary rewrite of parent / sibling / global namespace.

The future public boundary is
`struct: StructLikePattern -> symbol`; AST remains an internal carrier, and
graph installation remains in the outer binding layer.

## 6. Compile / symbol construction interpreter bootstrap (v0.8)

The implemented v0.8 slice is a restricted type-shaped evaluator. The final
capability and ownership model is canonical in
`symbol-first-meta-construction-and-pattern-injection.md`; this bootstrap only
records the migration boundary:

```text
restricted evaluator
  -> shared invocation frame and policy checks
  -> PatternValue result rank
  -> outer binding/NamespaceDelta installation
```

The current `ForwardedValue`, `GeneratedConstructionValue`, and
`GeneratedTypeDefinitionValue` enums remain transitional implementation
transport. They do not implement canonical facets, `MetaInstanceScope`,
construction-lineage `Open`, `extend`/`inject`, pure-role self-root validation, or
construction-unit authority.

Before ordinary generic type-style meta-functions are implemented, the
construction contract in
`spec/contracts/v0.8-meta-construction-agent-constraints.md` must be absorbed.
Do not expand the bespoke `struct` path for `(T Vec)`, `(T Option)`,
`(A, B Pair)`,
or other compile/meta construction. New work must follow the shared route:

```text
resolve callee
  -> ProductObject / ArgProductShape
  -> RawArgShape / ParameterShape
  -> rank-directed Symbol / TypeValueId / PatternValue classification
  -> policy body-entry check
  -> PatternValue
  -> binding-layer installation adapter
  -> NamespaceDelta atomic install
```

## 7. Stage scope (must cover / non-goals)

### v0.6 — Build / Namespace Graph Bootstrap

Must cover: package manifest skeleton; source root / namespace root; core
package default mount; namespace mount table; physical namespace skeleton from
directories; implementation file as source fragment (file name does not
contribute a namespace segment); declared symbol harvesting; SymbolObject model;
physical / declared / virtual `NamespaceNode` kind; resolver returning a
`SymbolObject`, not a string path; provenance and diagnostic attachment; the
role-aware child-name model (§3) and the ordinary direct-child contribution /
local-construction rules (§4); `NamespaceOrigin`, physical contribution
authority, and source construction ownership as future contracts; no
source-level import/use/include/module;
policy metadata slots on symbols, contexts, and namespace graph nodes
with three-phase compatibility resolver visibility filtering;
full pair policy
checking remains future work (see `spec/design/policy-capability/policy-visibility-symbols.md`).

Non-goals: full version solving; remote package retrieval; lockfile
completeness; dynamic/static distribution distinction; full access-control
lattice; full policy checking; full type checking; full meta-function execution.

### v0.7 — Early Meta-Function Bootstrap

Must cover: early meta-function lookup from the namespace graph; closed
`SyntaxObject` passing; `assert`; `struct` as the first real core-namespace
meta-function object; meta call replacement; `MetaExpansionResult`
(replacement / namespace delta / diagnostics / provenance); current symbol,
  body-entry, and return-object metadata retained as transitional substrate for
  future `Pv:Pp`, contextual P1 projection, and P2 result normalization, not a
  normative P3 (no full pair checker — see
`spec/design/policy-capability/policy-visibility-symbols.md`); source/meta
construction-unit ownership and physical contribution authority (§4);
generated child namespace installation; no arbitrary rewrite of parent /
sibling / global namespace; `struct` consumes AST by a private checker, failure
is a meta hard error.

Non-goals: general `compile` PatternValue execution; value-directed meta construction;
arbitrary control flow in meta bodies; full generic system; full pattern-space
semantics; HIR/codegen integration beyond placeholder nodes.

### v0.8 — Compile / Symbol Construction Interpreter Bootstrap

Must cover the transition from the restricted type-shaped evaluator toward:
ordinary normalized structured input; `compile` producing `PatternValue`;
`meta` sealing a `MetaInstance` and returning its symbol value; rank-directed
canonical
argument identity; orthogonal creation/write/return events (current compatibility
encoding:
`let r = expr;` fresh member, `r = expr;`
existing-target write — currently a placeholder overwrite scaffold, `r;`
delivery terminal); binding-layer
installation under a legal writable place; and first-class `(T Vec)` /
`(T Option)` / `(A, B Pair)` construction through the shared invocation frame.

Non-goals: unrestricted compile-time IO; runtime execution; full
borrow/lifetime checking; full pattern-space subtraction / exhaustiveness;
complete operator overload semantics; general macros; graph-member creation
inside formal `struct`/`extend` or through `inject`.

## 8. Conceptual constraints

- No source-level `import` / `use` / `include` / `module` syntax.
- `struct` is not a keyword or parser special form. It is a compiler-defined
  `BuiltinPrivilegedAstMetaFunction` object resolved through the same
  symbol-first/function-object call framework as other callables. Its bounded
  AST capability is compiler-known and bootstrap-backed, not user-definable or
  a general macro facility.
- Namespace is not equal to filesystem path. Directory paths provide only the
  physical skeleton; the full graph includes physical, declared, and virtual
  nodes.
- Metaprogramming may not inject into unrelated global namespaces. One source
  unit may fully construct a direct child subtree that it creates; one meta unit
  may fully construct its virtual subtree. Neither may reopen a subtree owned by
  another construction unit.
- Compile/meta bodies consume ordinary parsed and normalized structured material
  under capability policy — not a separate compile-time DSL or text macro.
- `compile` computes `PatternValue`; `meta` seals a `MetaInstance` and returns
  its symbol value, which is an ordinary `PatternValue`. Target semantics use
  ordinary `let` creation, existing-place `=` writes, and a separate return
  event. The current `let`-only compatibility encoding is: `let r = expr;` adds
  a fresh member, `r = expr;` writes to an existing
  target (today a placeholder overwrite scaffold; the final write
  algebra is not fixed), and `r;` delivers the construction. There is no
  alias-member event; a member that must observe an external object holds a
  borrow view.
- `compile` creates no meta-instance scope. An ordinary canonical meta
  invocation does, and any returned type-role member is rooted in that scope rather
  than in an external `PatternValue` or a later binding destination. Privileged
  AST meta functions use their separately declared scope/owner rule.
- `struct` owner identity comes from input pattern navigation plus ambient
  `ResolvedPatternScope`, never from the later binding destination.
- Pure `extend(type, material)` adds direct children only when the value's
  `ConstructionLineage` is Open in the current stack. `inject(type ref,
  material)` is read--extend--write and additionally requires the target place
  writable. A ref proves neither Open nor promotion, and there is no
  construction-handle rank.
- v0.6–v0.8 do not claim full policy checking, full type checking, full pattern
  checking, or full value-level compile-time evaluation. Those remain later
  stages.

## 9. Relationship to other tracks

- Build / package architecture, node kinds, and injection: `build-system-design.md`.
- Assembly pipeline phases: `namespace-assembly-v0.md`.
- Manifest surface: `package-manifest-v0.md`.
- Library/namespace overview: `library-namespace-design-note.md`.
- Namespace origin and construction-unit ownership:
  `symbol-construction-units-and-namespace-origin.md`.
- Later pattern-space / extraction-chain semantics (v0.10+):
  `static-pattern-spaces-and-extraction-chains.md`.

Future formal meta object invocation depends on the package/manifest layer for
stable package identity, mount provenance, export-surface boundaries, and
cache/fingerprint participation. Those build-layer facts are documented in
`build-system-design.md` and `package-manifest-v0.md`.
