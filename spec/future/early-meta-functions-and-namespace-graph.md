# Early Meta-Functions and the Namespace Graph

**Status: Non-normative future design. This is the post-v0.5 roadmap track
(v0.6–v0.8). A partial v0.6 vertical slice is implemented in `lang_build`;
the remaining v0.6–v0.8 design is future work and is not a parser/normalizer
rule.**

This document is the canonical direction for the v0.6–v0.8 sequence:

- v0.6 — Build / Namespace Graph Bootstrap
- v0.7 — Early Meta-Function Bootstrap
- v0.8 — Type-to-Type Meta Construction Interpreter

It builds on, and does not replace, the build/package architecture in
`spec/future/build-system-design.md`, the assembly pipeline in
`spec/future/namespace-assembly-v0.md`, and the manifest surface in
`spec/future/package-manifest-v0.md`. The later pattern-space / extraction-chain
semantics remain a separate track in
`spec/future/static-pattern-spaces-and-extraction-chains.md`.

This document records the **current** build / namespace graph / early-meta
bootstrap track (v0.6–v0.8) and its narrow implemented slice. The **future**
unified invocation semantics — one policy-governed callable-invocation model
covering ordinary functions, meta functions, verification, control predicates,
operators, and type constructors, together with partial/strict meta reduction
and residualization — are specified in
`spec/future/meta-object-invocation-and-policy-reduction.md`.

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

### Policy flag assignment

| Symbol source | Policy set |
|---|---|
| Core namespace symbol | `export + meta + runtime` |
| Namespace symbols (declared, physical, dependency mount, generated) | `meta + runtime` |
| Core meta-functions (`struct`, `assert`) | `export + meta` |
| Core verification namespace and operations (`verify`, `verify::exists`, …) | `export + meta` |
| Core built-in types/ranks (`uint8`, `type`, `namespace`, `ref`, `share`, …) | `export + meta + runtime` |
| Source-contributed ordinary value placeholders | `runtime` |
| Source-contributed type-annotated placeholders (`: type`) | `meta + runtime` |
| Struct-generated `TypeObject` | `meta + runtime` |
| Projection namespace symbols (`ref`/`share` under a generated type) | `meta + runtime` |
| Generated field-function symbols (`field::T`, `field::ref::T`, `field::share::T`) | `meta + runtime` |
| Alias symbols | `runtime` (not transparent for early meta yet) |

Generated `struct` expansion currently assigns these policy planes:

| Generated object | Policy plane |
|---|---|
| Generated `TypeObject` | symbol policy = `meta + runtime` |
| Projection namespace `ref` / `share` | symbol policy = `meta + runtime` |
| Generated field function | symbol policy = `meta + runtime` |
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
`export + meta + runtime` so that explicit paths such as `struct::core`
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
  because their symbol policy is `meta + runtime`, but their callable
  body-entry and return-object policies are runtime-only.
- Namespace child lookup is role-aware. Object/function symbols and pure
  namespace subspaces can share the same textual child name. Terminal lookup
  without an expected role reports ambiguity when both roles are present.
- Failed `struct` expansion returns hard diagnostics and leaves no partial
  generated subtree. Duplicate fields, unknown field types, unit/trailing-unit
  fields, and unsupported nested products are rejected. Fields named `ref` or
  `share` are allowed because fields are unary function objects while
  `ref` / `share` are namespace subspaces.
- v0.6 still represents non-meta `let T: type = uint8` as a placeholder type
  payload. That is an implementation placeholder only: the long-term semantics
  are ordinary type-value binding, not fresh type generation and not symbol
  aliasing.
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
                           -> type-to-type meta construction interpreter
```

## 2. NamespaceGraph Capability Layer

The first build-system version must not be a CLI-only package scanner. It must
expose a capability layer that the resolver, the meta interpreter, and (later)
the type checker all share. The capability layer abstracts the namespace graph
behind named operations, at least:

- **resolve** — resolve a navigation path to a `SymbolObject`.
- **declare** — introduce a declared symbol under a node.
- **inject child** — attach a generated child under a legal parent / instance
  node (see §4 for the contribution rules).
- **alias** — forward a name to an existing globally visible symbol.
- **open virtual node** — open a virtual namespace node for generated structure.
- **install namespace delta** — apply a set of generated declarations as one
  unit under a legal node.
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

Policy metadata (see `spec/future/policy-visibility-symbols.md`) should be
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

`MetaExpansionResult` is transaction-like.

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
and later type-to-type meta-functions.

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
- source fragments contribute only direct children;
- ordinary parent-to-descendant injection is rejected;
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

Their symbol policy is `meta + runtime`, so the compiler can resolve and inspect
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

Future generic/meta-generated types such as `(int)Vec::std` return stable type
values. Therefore:

```text
let A: type = (int)Vec::std
let B: type = (int)Vec::std
```

means `A == B` by type-value equality while `A` and `B` remain distinct symbols
unless one is declared via `===`. Canonical `TypeValueId` and full type-value
equality are future work.

See `spec/future/type-associated-function-objects-and-access-trees.md` for the
full value/place/injection distinction and alias writability rule.

## 4. Namespace contribution rules

These rules constrain how declarations enter the namespace graph. They protect
the intuition that the physical directory hierarchy explains the namespace
shape: when you open a directory level, the files there contribute the directly
indexable objects **at that level**, not deep virtual structure.

### 4.1 No ordinary parent-to-descendant injection

> **No ordinary parent-to-descendant injection. Only ordinary
> parent-to-direct-child contribution is allowed.**

1. **Ordinary physical / file contribution context:** a source fragment may
   contribute only the directly indexable children of its current namespace
   node. Under `ns`, files may contribute `f::ns`, `g::ns` (direct children).
   They must **not** inject into grandchildren or deeper descendants such as
   `x::f::ns` or `y::x::f::ns`.

2. **Direct-child local construction:** a direct child object may construct its
   own internal / associated namespace structure. Deeper structure must be
   **owned by the immediate parent object** and built once by that object's
   local construction — not scattered across sibling files. So `x::f::ns` must
   come from `f`'s own local / associated construction, not from an unrelated
   file under `ns` injecting across the level.

3. **Meta-function instantiation context (exception):** a closed instantiation
   may generate a parent-to-descendant virtual subtree, because the
   instantiation is a closed, globally consistent, cacheable generation process
   whose result is exposed as a whole as a virtual layer (not a to-be-merged
   physical directory layer). This exception **does not** apply to ordinary
   physical / file contribution. Even in meta context this is **allowed but not
   encouraged**: the generator bears the implementation, cache-key, and
   diagnostic complexity, so scattered deep generation is discouraged.

### 4.2 Rationale

Filesystem directories provide the physical namespace skeleton, and multiple
implementation files may merge-contribute to the current namespace level. The
readability of this depends on the intuition that the files at a level
contribute that level's direct objects. If ordinary files could inject into
grandchild / great-grandchild levels, directory structure would lose its
explanatory power: a user could see the directories yet be unable to infer the
real namespace shape, because any file might secretly stack up multi-level
virtual structure. In the multi-file "merge-contribute a type object" case this
is especially unintuitive, and same-level files could otherwise stack arbitrary
depth that does not appear in the directory hierarchy, so the namespace graph's
shape would only be recoverable by a whole-project scan. The rule also preserves
the locality of type-associated namespaces: `x::f::ns` belongs to `f`'s
associated space, not to an unrelated file under `ns`.

This is **not** a prohibition on multi-level structure. Deep structure may
exist; construction responsibility is localized — each level is built by its
immediate parent object.

### 4.3 Diagnostic

```text
ordinary descendant injection is not allowed

current contribution namespace:
    ns

attempted target:
    x::f::ns

ordinary source fragments may contribute only direct children of their current
namespace. Declare `f` as a direct child, then define `x` inside `f`'s own local
or associated namespace.
```

### 4.4 Combined rules

```text
Source uniqueness:
    a child name comes from exactly one of physical directory / type-associated
    namespace / meta-instantiation virtual layer.

Direct contribution:
    ordinary source fragments contribute only direct children of the current
    namespace.

Local construction:
    deeper levels must be built by their immediate direct child object.

Meta exception:
    a closed instantiation may generate a multi-level virtual subtree, exposed
    as one instantiation virtual layer.
```

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
- **Meta call replacement model** — a meta call is replaced by its expansion
  result.
- **`MetaExpansionResult`** carries:
  - replacement object,
  - namespace delta,
  - diagnostics,
  - provenance.
- **Parent-to-child namespace injection rule** (per §4) — generated child
  namespaces are installed only under a legal parent / instance node; no
  arbitrary rewrite of parent / sibling / global namespace.

## 6. Type-to-type meta construction interpreter (v0.8)

The earliest, most restricted meta-function body execution model is **type ->
type**: a single entry, single exit, no intermediate control flow, pure
streaming structure. A meta-function body is **not** a separate DSL and **not** a
text macro: it is the ordinary parsed and normalized language AST (Raw AST /
Normalized AST) the source file already produced, executed under a meta policy by
a **type-object construction interpreter**.

- **Meta body as normalized AST** — executed by the type-object construction
  interpreter.
- **Declaration-as-assignment / assignment-as-injection** — a `let` inside the
  meta body creates symbols through the NamespaceGraph Capability Layer.
- **`===`** is symbol alias / forwarding, not a copy.
- **Explicit return object slot**, e.g. `meta + runtime let r: type`:
  - `r = t` returns the generated object,
  - `r === t` forwards an existing globally visible symbol.
- **Generative meta identity** is based on the function symbol + canonical
  arguments + build/config fingerprint.
- **Symbol shielding** — the externally visible result name is determined by the
  meta-function name + arguments, not by internal temporary names; the generated
  result has a globally consistent symbol identity.
- **Installation** — generated declarations are installed only under a legal
  parent / instance node (per §4).
- **First-class generic classes** such as `Vec(T)`, `Option(T)`, `Pair(A, B)` are
  expressible in this type-to-type form.

## 7. Stage scope (must cover / non-goals)

### v0.6 — Build / Namespace Graph Bootstrap

Must cover: package manifest skeleton; source root / namespace root; core
package default mount; namespace mount table; physical namespace skeleton from
directories; implementation file as source fragment (file name does not
contribute a namespace segment); declared symbol harvesting; SymbolObject model;
physical / declared / virtual `NamespaceNode` kind; resolver returning a
`SymbolObject`, not a string path; provenance and diagnostic attachment; the
role-aware child-name model (§3) and the ordinary direct-child contribution /
local-construction rules (§4); no source-level import/use/include/module;
policy metadata slots on symbols, contexts, and namespace graph nodes
with minimal `PolicyEnv::Meta` resolver visibility filtering; full policy
checking remains future work (see `spec/future/policy-visibility-symbols.md`).

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
`spec/future/policy-visibility-symbols.md`); the parent-to-child injection rule,
with parent-to-descendant generation only as the closed meta exception (§4);
generated child namespace installation; no arbitrary rewrite of parent /
sibling / global namespace; `struct` consumes AST by a private checker, failure
is a meta hard error.

Non-goals: general compile-time value execution; value-to-value meta-functions;
arbitrary control flow in meta bodies; full generic system; full pattern-space
semantics; HIR/codegen integration beyond placeholder nodes.

### v0.8 — Type-to-Type Meta Construction Interpreter

Must cover: meta body as normalized AST; type-object construction interpreter;
declaration-as-assignment / assignment-as-injection; `let` inside a meta body
creating symbols through the capability layer; `===` as alias / forwarding;
explicit return object slot; `r = t` (generated) vs `r === t` (forwarded);
generative meta identity (function symbol + canonical args + build/config
fingerprint); symbol shielding; installation under a legal parent / instance
node; first-class `Vec(T)` / `Option(T)` / `Pair(A, B)`.

Non-goals: value-to-type control flow; value-to-value compile-time world;
unrestricted compile-time IO; runtime execution; full borrow/lifetime checking;
full pattern-space subtraction / exhaustiveness; complete operator overload
semantics.

## 8. Conceptual constraints

- No source-level `import` / `use` / `include` / `module` syntax.
- `struct` is not a keyword and not a parser special form; it is not a hardcoded
  compiler branch. It is a core-namespace meta-function object resolved through
  the same resolver path as other symbols, even if the first implementation
  internally bootstraps it.
- Namespace is not equal to filesystem path. Directory paths provide only the
  physical skeleton; the full graph includes physical, declared, and virtual
  nodes.
- Metaprogramming may not inject into unrelated global namespaces. Only
  parent-to-direct-child contribution (ordinary) or parent-to-descendant
  generation inside a closed instantiation (meta) is allowed.
- Meta bodies are ordinary parsed and normalized language AST executed by a
  restricted meta construction interpreter — not a separate compile-time DSL.
- Generative and forwarding meta-functions are distinct: `r = t` returns a
  generated object; `r === t` forwards / aliases an existing globally visible
  symbol.
- v0.6–v0.8 do not claim full policy checking, full type checking, full pattern
  checking, or full value-level compile-time evaluation. Those remain later
  stages.

## 9. Relationship to other tracks

- Build / package architecture, node kinds, and injection: `build-system-design.md`.
- Assembly pipeline phases: `namespace-assembly-v0.md`.
- Manifest surface: `package-manifest-v0.md`.
- Library/namespace overview: `library-namespace-design-note.md`.
- Later pattern-space / extraction-chain semantics (v0.10+):
  `static-pattern-spaces-and-extraction-chains.md`.
