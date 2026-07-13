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
`struct`, and functional `inject` boundaries are specified in
`spec/design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md`.
That document supersedes the older formal return-slot split between `r = ...`
and `r === ...` and the transitional idea that a binding destination determines
the final `struct` owner identity.

Namespace-facet origin, source/meta construction-unit ownership, physical
directory contribution authority, and current cross-file closure are canonical
in
`spec/design/symbol-world/symbol-construction-units-and-namespace-origin.md`.

## v0.7 implementation additions

v0.7 introduces early policy-aware resolution with three policy flags:

- `PolicyFlag::Export`, `PolicyFlag::Meta`, `PolicyFlag::Runtime`
- `PolicySet` — bit-set of flags carried on `PolicyMetadata.policy_set`
- `PolicyEnv::Meta` — resolver lookup visibility environment; only symbols
  carrying the `Meta` flag are visible to this lookup query. This does not grant
  permission to enter or evaluate a callable body.

### Source verification forms

`lang_build` also contains a source-driven fixture verification loop. `verify`
is bootstrapped as a core meta-visible verification namespace/object, with
verification operations installed below it as core meta-function symbols.
Ordinary normalized expressions are treated as verification forms only after the
verification entry and operation resolve through the namespace graph under
`PolicyEnv::Meta`. They do not install symbols, produce runtime objects, or add
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
| Core namespace symbol | `export | meta | runtime` |
| Namespace symbols (declared, physical, dependency mount, generated) | `meta | runtime` |
| Core meta-functions (`struct`, `assert`) | `export | meta` |
| Core verification namespace and operations (`verify`, `verify::exists`, …) | `export | meta` |
| Core built-in types/ranks (`uint8`, `type`, `namespace`, `ref`, `share`, …) | `export | meta | runtime` |
| Source-contributed ordinary value placeholders | `runtime` |
| Source-contributed type-annotated placeholders (`: type`) | `meta | runtime` |
| Struct-generated `TypeObject` | `meta | runtime` |
| Projection namespace symbols (`ref`/`share` under a generated type) | `meta | runtime` |
| Generated field-function symbols (`field::T`, `field::ref::T`, `field::share::T`) | `meta | runtime` |
| Alias symbols | `runtime` (not transparent for early meta yet) |

Generated `struct` expansion currently assigns these policy planes:

| Generated object | Policy plane |
|---|---|
| Generated `TypeObject` | symbol policy = `meta | runtime` |
| Projection namespace `ref` / `share` | symbol policy = `meta | runtime` |
| Generated field function | symbol policy = `meta | runtime` |
| Generated field function | body entry policy = `runtime` |
| Generated field function | return object policy = `runtime` |

### Policy-aware resolver

New methods on `NamespaceGraphCapability`:

- `resolve_with_policy(…, PolicyEnv)` — filters per-component and terminal
  results; a symbol that does not satisfy the policy environment is treated as
  not found in that search root.
- `resolve_str_with_policy(…, PolicyEnv)`
- `resolve_type_object_with_policy(…, PolicyEnv)`
- `resolve_meta_function_with_policy(…, PolicyEnv)`

Policy filtering happens **before** cross-root conflict reporting. A
runtime-only local `uint8` does not block discovery of
`export+meta+runtime` `core::uint8`.

Policy filtering is **per-component**: every path component (including
namespace intermediaries like `core`) is checked against the policy
environment. Namespace symbols that must be traversed under a policy
environment therefore carry appropriate traversal policy flags. For v0.7,
the compiler-seeded `core` namespace symbol is assigned
`export | meta | runtime` so that explicit paths such as `struct::core`
and `uint8::core` resolve correctly under `PolicyEnv::Meta`.

`PolicyEnv::Meta` is lookup visibility, not meta execution permission. Meta
lookup may resolve runtime-callable symbols whose symbol policy includes `Meta`.
A meta evaluator may only execute a callable if the callable body-entry policy
admits `Meta`.

### Early meta expansion uses PolicyEnv::Meta

- `try_expand_early_meta_initializer` resolves the call target via
  `resolve_meta_function_with_policy(…, PolicyEnv::Meta)`.
- `parse_field_expr` resolves field type names via
  `resolve_type_object_with_policy(…, PolicyEnv::Meta)`.

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
  `ref`, and `share` as `SymbolObject`s in the namespace graph. `struct` and
  `assert` are meta-function symbols; parser and normalizer do not special-case
  either name.
- Declaration harvesting supports the narrow top-level direct-child form needed
  by the slice, especially `let T: type = ...`. Ordinary file contributions
  that attempt parent-to-descendant injection are rejected.
- Early meta lookup resolves the call target through the graph. The only
  accepted source form is currently equivalent to
  `(uint8 a, uint8 b) |> struct`; field types resolve through the same graph and
  field binders are private `struct` checker material.
- Resolver contexts distinguish current namespace lookup, explicit mounted paths
  such as `uint8::core`, and short-name default mounts such as `uint8`.
- Successful `struct` expansion produces a placeholder type object and a
  generated type-associated namespace containing `a::T`, `a::ref::T`,
  `a::share::T`, `b::T`, `b::ref::T`, and `b::share::T`-style field-function
  symbols. These field-function symbols are visible under `PolicyEnv::Meta`
  because their symbol policy is `meta | runtime`, but their callable
  body-entry and return-object policies are runtime-only.
- PR #94 adds an explicit pattern-head attachment helper with generated,
  global, namespace, and local categorical contexts. Formal `struct` uses the
  `GeneratedTypeDefinition` fallback; binding/materialization may reattach
  owner/field heads under a destination global or namespace context. This is
  transitional `PatternHeadId` registry substrate, not final
  `ResolvedPatternScope` or binding-independent owner resolution.
- Namespace child lookup is role-aware. Object/function symbols and pure
  namespace subspaces can share the same textual child name. Terminal lookup
  without an expected role reports ambiguity when both roles are present.
- Failed `struct` expansion returns hard diagnostics and leaves no partial
  generated subtree. Duplicate fields, unknown field types, unit/trailing-unit
  fields, and unsupported nested products are rejected. Fields named `ref` or
  `share` are allowed because fields are unary function objects while
  `ref` / `share` are namespace subspaces.
- v0.8 ordinary initializer evaluation can materialize
  `let T: type = uint8` as an ordinary type-value binding: resolve
  `symbol(uint8)`, read its type value, and bind that value to fresh
  symbol/place `T`. This is not symbol aliasing and does not canonicalize
  namespace injection targets.
- The current slice does not implement `NamespaceOrigin`, source/meta
  construction-unit ownership, physical contribution authority, cross-file
  reopening diagnostics, or meta return type self-root checking.
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
  built-in is functional and returns a new `SymbolConstructionValue`; only the
  outer binding/assembly layer installs the resulting delta.
- **alias** — forward a name to an existing globally visible symbol.
- **open virtual node** — open a virtual namespace node for generated structure.
- **install namespace delta** — apply a set of physical, declared, or generated
  contributions as one unit under a legal node.
- **canonical meta instance key** — compute the stable identity of a meta
  instantiation (see §6).
- **diagnostic** — attach a diagnostic with provenance to a node / operation.
- **assert / hard error** — raise a compile-time hard error.

Capabilities may be stubbed in early versions, but the **surface** must be the
shared one above, not narrowed to package scanning.

### SymbolObject

The resolver returns a **SymbolObject**, not a string path. A `SymbolObject`
carries the resolved identity, its source category (§3), its node kind, and its
provenance / diagnostics. Source code navigates names; the resolver answers with
objects, so later phases (meta lookup, type checking) operate on objects rather
than re-parsing path strings.

`SymbolObject` is the current implementation substrate. The final conceptual
model is a symbol-first `SymbolCell` with one `SymbolId`, one `PlaceId`, and
optional namespace/type plus heterogeneous value facets:

```text
path/name -> Symbol -> context-directed facet projection
```

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
`SymbolCell` model permits one symbol's value facet to contain multiple
heterogeneous value entries. That future same-symbol facet rule is not the same
as silently merging two independently declared `SymbolObject`s today; it
requires facet-aware declaration identity and conflict checking first.

This is required for fields named `ref` or `share`: the field is an object
symbol, while `ref` / `share` projection spaces are namespace subspaces.

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
compile -> PatternValue
meta -> SymbolConstructionValue
let binding/injection -> NamespaceDelta atomic install
```

Formal `struct`, formal meta invocation, and functional `inject` do not install
the graph. `MetaExpansionResult` may remain an implementation adapter, but it
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
- minimal `PolicyEnv::Meta` resolver visibility filtering is implemented, while
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
functions, `ref` / `share` projections, layout metadata, pattern interfaces, and
related companion symbols.

A type-associated namespace is **not** simply a "declared namespace object". Its
members may be **declared**, **generated**, or **virtual** depending on origin.
For a `struct`-generated type, the type-associated namespace is a virtual /
generated child namespace attached to the type node.

What unifies the category is the **role** (companion space of a type object),
not the origin of its members.

For `struct`-generated fields, fields are unary function objects:

```text
field::T        : T       -> field
field::ref::T   : T ref   -> field ref
field::share::T : T share -> field share
```

Their symbol policy is `meta | runtime`, so the compiler can resolve and inspect
them during meta/type-checking phases and can construct residual runtime calls
that reference them. Their callable body-entry policy is `runtime`, and their
return-object policy is `runtime`; meta lookup visibility does not permit a meta
evaluator to enter their bodies.

`field::T` is value semantics (`T == T move`). Borrowed field access must begin
from an explicit borrow form such as `val ref.field1` or
`val share.field1`. Field access evaluation, borrow normalization, and
access-tree construction are future work.

Because fields are object-role function symbols and `ref` / `share` are
namespace-subspace-role projection spaces, fields named `ref` or `share` are
valid. Terminal `ref::T` or `share::T` may be ambiguous unless resolver callers
provide an expected role.

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

Namespace injection is not pure type-value evaluation. `let f::T = ...`
targets `place(T)`, not `place(uint8)`. Type-value equality must not
canonicalize injection targets.

`=` and `===` are not interchangeable:

| Form | Symbol effect | Type-value effect | Injection-place effect |
| --- | --- | --- | --- |
| `let T: type = uint8` | Creates new symbol/place `T` | `value(T) == value(uint8)` | `f::T` injects into `place(T)` if current-level and open |
| `let T === uint8` | `T` forwards to symbol `uint8` | `value(T) == value(uint8)` | `f::T` attempts `place(uint8)` and is rejected because `uint8` is external stable |
| `let T: type = ... |> struct` | Creates new symbol/place `T` | `value(T)` is a fresh generated type value | `f::T` injects into `place(T)` if open |

Fresh generated type values own/provide their own type-associated namespace, so
`let T: type = (uint8 a, uint8 b) |> struct` creates the fresh type value whose
field functions are visible as `a::T`, `a::ref::T`, and `a::share::T`.

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

means `A == B` by type-value equality while `A` and `B` remain distinct symbols
unless one is declared via `===`. Canonical `TypeValueId` and full type-value
equality are future work.

See `spec/design/symbol-world/type-associated-function-objects-and-access-trees.md` for the
field-function and access-tree implications. The intended final distinction
between type values, symbol places, alias forwarding, and writable injection
targets is documented in
`spec/design/symbol-world/type-values-places-and-alias-forwarding.md`.

### 3.5 Final facet inclusion and value-member boundary

The role-aware `SymbolObject` buckets above describe the current substrate. The
final `SymbolCell` model uses structural facet inclusion:

```text
has TypeFacet => has NamespaceFacet
has NamespaceFacet ⇏ has TypeFacet
```

This is not type-system subtyping. A type symbol has a namespace facet, a
`TypeFacet(PatternValue)`, and optionally a heterogeneous value facet. A pure
namespace has no type `PatternValue` but may still contain ordinary value
members.

Pattern-material leaves belong to the `PatternValue` in a type facet and affect
normalization, matching, and extraction. Ordinary namespace value members alter
only the namespace/value graph. They do not enter `Set<PatternValue>`, do not
change ordered pattern material, and do not participate in extraction. A value
projection is terminal for the current lookup; it does not prevent the same
symbol from also owning namespace children.

The complete origin and ownership rules for these facets are in
`symbol-construction-units-and-namespace-origin.md`.

## 4. Namespace contribution rules

These rules constrain how declarations enter the namespace graph. They protect
the intuition that the physical directory hierarchy explains the namespace
shape: when you open a directory level, the files there contribute the directly
indexable objects **at that level**, not deep virtual structure.

### 4.0 Shared physical/meta capability substrate

Physical source fragments and meta-produced symbol constructions use the same
declare / inject-child / open-namespace / delta-install capability base.

For example:

```text
ns/
  impl.lang
  export.lang
```

Both files may create distinct same-level children of `ns`. Each file is one
closed `SourceConstructionUnit`; it may fully construct a child subtree that it
creates, but it may not reopen a subtree created by the other file. A canonical
meta invocation similarly owns one closed `MetaConstructionUnit`. In both cases
the outer assembler or `let` binding forms a conflict-checked `NamespaceDelta`
and installs it atomically.

The common capability base does not give physical source fragments meta-body
pipeline order. Physical contributions are independently derived, replayable
contribution/delta values. Distinct direct-child contributions may be combined
transactionally; filename, filesystem traversal, and source discovery order
have no semantic effect. Same-child reopening, duplicate names, or facet
conflicts remain hard errors.

The future source-level `inject` operation is also functional: it transforms a
symbol/construction value and returns a new uninstalled construction. It does
not directly mutate the namespace graph.

### 4.1 Direct-child roots; no cross-unit descendant reopening

> **An ordinary source contribution begins by creating a direct child. The same
> source unit may construct that new child's subtree, but it may not target an
> already existing descendant owned by another unit.**

1. **Direct-child authority:** a source file may create direct children of its
   current physical directory namespace. A direct child it creates may include
   a complete descendant subtree inside that same source delta.
2. **No reopening:** the complete subtree remains owned by that source file.
   Another file may not target a grandchild of the already-created child, even
   if the requested grandchild name is absent.
3. **Meta transaction:** one canonical meta invocation may build a complete
   virtual subtree because all actions belong to one `MetaConstructionUnit` and
   one transaction. The reason is construction-unit identity, not merely that a
   meta callable has one body.
4. **Helper isolation:** a helper meta invocation has its own construction unit.
   A caller may compose its uninstalled result but may not mutate the helper's
   already installed subtree.

### 4.2 Rationale

Filesystem directories provide the physical namespace skeleton and the
authority boundary for direct source contributions. Multiple implementation
files at one level may create different direct objects; they do not co-own those
objects. A single owner keeps partial declaration, reopening, visibility,
diagnostic ownership, and merge authority out of the current model.

This is **not** a prohibition on multi-level structure. Deep structure may
exist; one source construction or one meta construction may build the complete
new subtree it owns.

The restriction is not derived solely from unordered file contributions.
Cross-file overload-entry union could be relaxed later if the language defines
stable entry identity and explicit merge authority. It is forbidden now along
with cross-file type-child, namespace-child, and ordinary value-member
injection.

### 4.3 Diagnostic

```text
ordinary descendant injection is not allowed

current contribution namespace:
    ns

attempted target:
    x::f::ns

ordinary source fragments may contribute only direct children of their current
namespace and may fully construct only children they create. Define `x` in the
same construction unit that owns `f`, or use an explicitly designed future
reopening facility.
```

### 4.4 Namespace origin and physical authority

Every namespace facet has exactly one creation origin:

```text
NamespaceOrigin =
    PhysicalDirectory(path)
  | SourceConstruction(source_construction_unit, construction_id)
  | MetaConstruction(meta_construction_unit, construction_id)
```

Under one parent, one child namespace path may be created by only one of those
origins. If `ns/ns1/` exists physically, source in `ns/` cannot create or
upgrade `ns1::ns`, and cannot contribute `x::ns1::ns`. Direct content of the
physical child must come from implementation files in `ns/ns1/`. Parent files
may navigate/read the child but cannot reopen it.

Because `has TypeFacet => has NamespaceFacet`, a physical namespace also cannot
be upgraded into a source-created type at the same path.

### 4.5 Combined rules

```text
Origin uniqueness:
    one child NamespaceFacet has one physical/source/meta creation origin.

Physical authority:
    direct content of a physical directory namespace comes only from files in
    that directory.

Construction ownership:
    one source/meta unit may fully construct its new subtree; parallel units may
    not reopen it.

Current cross-file closure:
    no type child, namespace child, ordinary value member, or overload-entry
    injection into a symbol owned by another file.

Meta transaction:
    one canonical invocation may build a multi-level virtual subtree inside its
    own MetaConstructionUnit.
```

The canonical details are in
`symbol-construction-units-and-namespace-origin.md`.

## 5. Early meta-function bootstrap (v0.7)

On the v0.6 namespace graph, the early meta-function call loop is closed so that
an early meta target is found by the **resolver**, not by a parser / normalizer
special case.

- **Early meta-function lookup** from the namespace graph (same resolver path as
  any other symbol).
- **Closed `SyntaxObject` passing** — the meta target receives a closed syntax
  object; the call process is opaque to outside observers.
- **`assert`** as a compile-time hard-check primitive.
- **`struct`** as the first real, globally visible meta-function object resolved
  from the core namespace. `struct` consumes AST through a private checker; a
  failure is a meta hard error, not a parser / normalizer error.
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

The future public boundary is `struct: normalized pattern material ->
SymbolConstructionValue : symbol`; AST remains an internal carrier, and graph
installation remains in the outer binding layer.

## 6. Compile / symbol construction interpreter bootstrap (v0.8)

The implemented v0.8 slice is a restricted type-shaped evaluator. The final
model separates two capabilities that the older type-to-type narrative mixed:

```text
compile:
  compute PatternValue
  (ordinary compile-time value, type value, or structured pattern value)

meta:
  create or transform SymbolConstructionValue
  public successful result rank = symbol
```

Both execute ordinary parsed/normalized structured material under policy. They
are not separate syntax languages or text-macro systems.

- **Compile result** — a `PatternValue`; a type value is not an installed type
  symbol. Compile creates no `MetaInstanceScope`, may return an existing type
  value, and uses the ordinary function-object Self frame for local `struct`.
- **Meta result** — an uninstalled `SymbolConstructionValue` carrying return
  pattern/facet material under a canonical `MetaInstanceScope`.
- **Meta type self-root** — if the return symbol has a `TypeFacet`, its outer
  pattern root is the canonical meta-instance scope. Direct `r = t` or
  `r = uint8` meta type returns are invalid when they would install an external
  root; external values may be members beneath the self-rooted type.
- **Formal meta return slot** — `r = ...` populates the construction's return
  layer. The old `r === ...` forwarding interpretation is superseded.
- **Ordinary alias declaration** — `let a === b` remains symbol/place forwarding
  and is not a formal meta return operation.
- **Rank-directed canonical arguments** — symbol parameters use symbol/place
  identity, type parameters use `TypeValueId`, and ordinary value parameters use
  `PatternValue` identity.
- **Complete navigation atom** — a namespaced instance is `(int Vec::std)` and
  a child is `child::(int Vec::std)`; `(int Vec)::std` and unparenthesized
  `int Vec::std` do not denote that instance.
- **Installation** — only an outer `let` binding/injection resolves a writable
  place and installs a `NamespaceDelta`.
- **Pattern ownership** — `struct` resolves its owner from input navigation plus
  ambient pattern scope; the binding target never reroots it.
- **Functional extension** — future `inject` selects the input symbol's internal
  pattern scope and returns a new uninstalled construction with direct children.
- **Single type installation** — an ordinary type facet is installed once;
  duplicate definitions do not form an implicit sum.
- **Construction ownership** — each source file or canonical meta invocation
  owns the complete subtree it creates; parallel files do not reopen it.

The current `ForwardedValue`, `GeneratedConstructionValue`, and
`GeneratedTypeDefinitionValue` enums remain transitional implementation
transport until these final objects exist.

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
  -> PatternValue or SymbolConstructionValue
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
with minimal `PolicyEnv::Meta` resolver visibility filtering; full policy
checking remains future work (see `spec/design/policy-capability/policy-visibility-symbols.md`).

Non-goals: full version solving; remote package retrieval; lockfile
completeness; dynamic/static distribution distinction; full access-control
lattice; full policy checking; full type checking; full meta-function execution.

### v0.7 — Early Meta-Function Bootstrap

Must cover: early meta-function lookup from the namespace graph; closed
`SyntaxObject` passing; `assert`; `struct` as the first real core-namespace
meta-function object; meta call replacement; `MetaExpansionResult`
(replacement / namespace delta / diagnostics / provenance); policy fields on
callable objects — distinct symbol visibility, body-entry, and return-object
policy planes (no full projection or execution checker — see
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
`meta` producing `SymbolConstructionValue : symbol`; rank-directed canonical
argument identity; `r = ...` assigning return-layer pattern/facet material;
ordinary `let ===` alias forwarding remaining separate; binding-layer
installation under a legal writable place; and first-class `(T Vec)` /
`(T Option)` / `(A, B Pair)` construction through the shared invocation frame.

Non-goals: unrestricted compile-time IO; runtime execution; full
borrow/lifetime checking; full pattern-space subtraction / exhaustiveness;
complete operator overload semantics; general macros; direct graph mutation
inside formal `struct` or `inject` invocation.

## 8. Conceptual constraints

- No source-level `import` / `use` / `include` / `module` syntax.
- `struct` is not a keyword and not a parser special form; it is not a hardcoded
  compiler branch. It is a core-namespace meta-function object resolved through
  the same resolver path as other symbols, even if the first implementation
  internally bootstraps it.
- Namespace is not equal to filesystem path. Directory paths provide only the
  physical skeleton; the full graph includes physical, declared, and virtual
  nodes.
- Metaprogramming may not inject into unrelated global namespaces. One source
  unit may fully construct a direct child subtree that it creates; one meta unit
  may fully construct its virtual subtree. Neither may reopen a subtree owned by
  another construction unit.
- Compile/meta bodies consume ordinary parsed and normalized structured material
  under capability policy — not a separate compile-time DSL or text macro.
- `compile` computes `PatternValue`; `meta` creates or transforms
  `SymbolConstructionValue : symbol`. Formal meta return material uses
  `r = ...`; ordinary `let a === b` remains the separate alias/place-forwarding
  operation.
- `compile` creates no meta-instance scope. A canonical meta invocation does,
  and any return `TypeFacet` is rooted in that scope rather than in an external
  `PatternValue` or a later binding destination.
- `struct` owner identity comes from input pattern navigation plus ambient
  `ResolvedPatternScope`, never from the later binding destination.
- Functional `inject` selects an input symbol's internal pattern scope and adds
  direct children without installing the graph.
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
