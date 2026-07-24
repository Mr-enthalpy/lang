# Symbol-First Meta Construction and Pattern Injection

**Status: Canonical future-design direction. Not current public language
behavior and not fully implemented.** This document is the canonical design
note for symbol-first resolution, symbol facets, `compile` / `meta` result
boundaries, meta return type self-root identity, resolved pattern scopes,
`struct`, functional `inject`, and the binding/install boundary.

The current implementation is a transitional substrate described in §13. In
particular, the current `PatternHeadId` attachment path must not be read as the
final owner-resolution rule.

This document builds on, without replacing:

- `spec/design/symbol-world/type-values-places-and-alias-forwarding.md` for
  `SymbolId` / `PlaceId` / `TypeValueId`, alias forwarding, and writable-place
  judgments;
- `spec/contracts/v0.9-pattern-head-identity-and-explicit-navigation.md` for
  the preserved bare-name versus explicit-`::` distinction and the current
  registry-backed substrate;
- `spec/design/patterns-overload/static-pattern-spaces-and-extraction-chains.md`
  for static pattern spaces, bounded extraction, and extraction-chain
  semantics;
- `spec/design/meta-invocation/meta-object-invocation-and-policy-reduction.md`
  for candidate selection, evaluation demand, policy, and residualization;
- `spec/design/build-package/build-system-design.md` for transactional
  namespace-graph assembly and physical source contributions;
- `spec/design/symbol-world/symbol-construction-units-and-namespace-origin.md`
  for namespace-facet origin, source/meta construction ownership, physical
  authority, and cross-file closure;
- `spec/design/symbol-world/symbol-policy-and-compile-flow-projection.md` for
  `Val1 x Pattern x Val2`, `Pv:Pp`, binding `P1`, result `P2`, compile-flow
  projection, derived compile companions, match staging, and automatic require.

## 1. Canonical Boundaries

The design has five load-bearing boundaries:

```text
name/path resolution:
  path/name -> Symbol -> context-directed facet projection

ordinary value binding:
  let destination = source
    -> resolve source Symbol -> read value -> bind destination Symbol/Place

compile-time value computation:
  compile -> PatternValue

symbol construction:
  meta -> SymbolConstructionValue : symbol

graph mutation:
  let binding/injection -> NamespaceDelta installation
```

Consequences:

1. A name does not initially resolve as a type, value, namespace, function, or
   alias category. It resolves as a first-class symbol.
2. Ordinary `=` reads a value through the source symbol and binds it to a
   distinct destination symbol/place. It does not alias, reroot, or merge
   identities.
3. `compile` computes pattern values. It does not install symbols.
4. `meta` creates or transforms symbol constructions. Its public successful
   result rank is always `symbol`; a return `TypeFacet` is rooted in the
   canonical meta-instance scope.
5. `struct` and `inject` return uninstalled symbol constructions. Neither
   operation installs a graph delta during formal invocation.
6. A `let` binding or injection path chooses the installation place. It does
   not retroactively choose or reroot the pattern owner carried by the value.

## 2. Symbol-First Resolution and Facets

### 2.1 Conceptual SymbolCell

The specification model is:

```text
SymbolCell {
    SymbolId
    PlaceId

    namespace_facet: optional
    type_facet: optional
    value_facet: zero or more heterogeneous value entries
}
```

This is a semantic model, not a requirement that this PR refactor the current
Rust `SymbolObject` into a structure with these exact fields.

Resolution is always:

```text
path/name
  -> Symbol
  -> context-directed facet projection
```

The following are facet projections:

```lang
symbol |> type
symbol |> val
symbol |> namespace
```

They are not traditional casts or conversions. Projection selects a facet of
the same symbol under the expectation of the use site.

### 2.2 Facets may coexist

One symbol may simultaneously provide:

- a namespace facet;
- a type facet;
- an ordinary value;
- a callable value;
- multiple heterogeneous value entries forming an overload candidate set.

The symbol remains one symbol. Facet coexistence does not imply that namespace,
type, and value identity collapse into one identity.

### 2.3 Identity separation

The model preserves distinct identities:

```text
SymbolId
PlaceId
TypeValueId
PatternValue identity
ResolvedPatternScope / PatternScopeId
```

Their roles are:

```text
SymbolId:
  identity of the resolved symbol cell

PlaceId:
  identity of the bindable/openable installation location

TypeValueId:
  canonical identity of a type value

PatternValue identity:
  canonical identity of an ordinary compile-time value, type value, or
  structured pattern value

PatternScopeId:
  identity of a navigable pattern-owner layer
```

No equality implication is automatic between these identities.

### 2.4 Program text names symbols before values

Except for literal syntax and other explicitly specified immediate values,
program text does not directly name a value. A source path first names a
symbol, and value use then reads a facet/value from that symbol:

```text
source path
  -> resolve Symbol
  -> read value / PatternValue from that Symbol
```

This applies to ordinary values, type values, pattern values, callable values,
and values later used as meta construction material.

Pattern navigation follows the same rule. A normalized pattern navigation name
may happen to render exactly like the source symbol path that carries it, but
matching diagnostic text does not merge their identities:

```text
source navigation path names a Symbol
PatternValue navigation name is a diagnostic/canonical projection
same spelling does not imply same semantic object
```

For example:

```lang
let a = 'a';
```

The left `a` is a symbol name. The right `'a'` is a character literal. Their
textual content happens to match, but they are not one semantic object.
Pattern values have no comparable standalone literal syntax, which makes a
same-spelled source path and pattern diagnostic projection especially easy to
confuse. The language still resolves the source path as a symbol first.

### 2.5 General `let` value binding

The ordinary binding rule is uniform. Its optional policy prefix is P1:

```lang
P1 let r = expr;
```

Evaluation first produces policy-indexed value/pattern entries:

```text
Gamma |- expr : (tau, Pv:Pp)
Gamma |- ProjectP1(P1, result(expr)) = selected
selected is non-empty
------------------------------------------------
Gamma |- P1 let r = expr
```

A single P1 `Q` selects RHS value entries visible under Q and follows each
selected value's associated pattern/type component. A pair P1 `Qv:Qp` filters
both components. Single P1 is not `Q:Q`. There is no general
`binding_policy != runtime` condition, so a normal runtime binding is legal:

```lang
runtime let x = runtime_value;
```

An omitted P1 retains and infers the complete RHS result; it does not make
runtime the only way to obtain a runtime binding.

The unannotated form:

```lang
let r = expr;
```

means:

```text
value(symbol(r)) := evaluate(expr)
```

If the right-hand expression is a source path, evaluation expands to:

```text
source path
  -> resolve source Symbol
  -> read its value / selected facet
  -> bind that value to the destination Symbol/Place
```

For example:

```lang
let a = b;
```

reads the value carried by `symbol(b)` and binds that value to `symbol(a)`.
It does not rename `a` to `b`, make their `SymbolId`s equal, or merge their
`PlaceId`s.

The rule does not change merely because the value is a type value, structured
pattern value, or symbol-construction result. In particular:

```lang
let t1::t = bool;
```

means:

```text
resolve symbol(bool)
  -> read the PatternValue carried by symbol(bool)
  -> bind that PatternValue to destination symbol/place t1::t
```

It does not reroot the `PatternValue`, rewrite its internal navigation, rename
its top pattern to `t1`, or identify `symbol(t1::t)` with the pattern owner.

Canonical summary:

```text
Program text normally cannot name values directly. It names a symbol, then
obtains a value through that symbol.

Pattern navigation paths are likewise symbol navigation first. Even when a
PatternValue's canonical navigation name matches the symbol carrying it, the
matching spelling does not establish identity.

A normalized fully named pattern layer contains PatternValue elements, not
Symbols. Extraction resolves a source Symbol, reads its PatternValue, and looks
up that value in the normalized set.

let destination = source
uniformly reads source's value and binds it to destination. It does not reroot
patterns, perform symbol aliasing, or merge place identity.
```

Any separate rule that requires a compile-determined projection source to have
non-runtime policy constrains that rule's `Psrc` only. It does not constrain
the P1 binding destination. In particular, an
implementation must not reject a binding merely because
`binding_policy == runtime`.

### 2.6 Ordinary aliases remain aliases

The ordinary declaration form remains:

```lang
let T === uint8;
```

This is symbol/place forwarding through the alias model. The symbol-first
correction does not remove it.

The canonical conclusion is:

```text
alias does not affect type-value equality;
alias still affects symbol forwarding, place forwarding,
namespace injection target, writability, and provenance.
```

Therefore several symbols may expose the same `TypeValueId` or pattern value
while preserving distinct symbol/place/alias relationships.

## 3. Value Facets and Calls

### 3.1 A value entry is not necessarily a function

The value facet may contain any value:

```lang
let f = expr;
```

If `expr` produces a value, the declaration may contribute a value entry to
the symbol `f`. The entry need not originate from closure syntax and need not
be callable.

Multiple entries under the same symbol may have heterogeneous types. A same-name
value facet is therefore not equivalent to a traditional same-signature
function-overload bucket.

### 3.2 Call candidate preparation

A call position performs the following conceptual flow:

```text
resolve symbol
  -> project value facet
  -> enumerate heterogeneous values
  -> observe each Val2 object's Pv:Pp view for the current lookup stage
  -> obtain each value's type
  -> resolve the type-associated `()` call entry
  -> discard non-callable or non-applicable entries
  -> form fully admissible set A using structure, Pattern/type/result checks,
     receiver/parameter policy-pair compatibility, P2 target-result
     compatibility when constrained, stage legality, and concept/require legality
  -> retain phase-specificity/const-mut product-maximal candidates
  -> apply the remaining fixed-order preference filters
  -> enforce must-select consistency and require one final candidate
```

An uncallable value is valid value-facet material. It is discarded only while
preparing candidates for a call position. Its presence does not make the symbol
invalid and does not turn it into a function overload.

Candidate identity and applicability belong to the candidate/invocation model;
symbol-first resolution only establishes where the heterogeneous values come
from. Derived compile companions are complete first-class `Val2` function
objects, not post-failure fallback entries; their policy and overload
obligations are defined in
`symbol-policy-and-compile-flow-projection.md`.

## 4. `compile`, `meta`, and Evaluation Demand

### 4.1 Orthogonal dimensions

The model has three independent dimensions:

```text
execution capability:
    meta / compile / seal / runtime

evaluation demand:
    partial / strict

result rank:
    PatternValue / SymbolConstructionValue / runtime value
```

`MetaPartial` / `MetaStrict` describe evaluation demand. They do not define the
meaning of `compile` or `meta`, and they do not determine the successful result
rank.

### 4.2 `compile`

`compile` is value-level staging. It performs compile-time computation without
creating a symbol-construction scope:

```text
compile:
  input PatternValue / compile-time value
  -> output PatternValue / compile-time value
```

`PatternValue` includes:

- ordinary compile-time values;
- type values;
- structured pattern values.

A computed type value is still a value. It is not thereby an installed type
symbol, a namespace node, or a writable place.

`compile` does **not** create a `MetaInstanceScope`, does not introduce a
meta-style virtual symbol layer for name shadowing, and does not impose a
self-root requirement on a returned type value. It may freely return an
already existing value:

```lang
let identity = (self, t: type): compile -> r: type => {
    r = t;
    r;
};
```

When a `compile` body uses a local `struct`, ordinary function-object scope
rules apply. Its ambient owner is the current callable owner and its anonymous
`Self` type. Nested paths print in source order, current/innermost `Self` first
and outermost `Self` last, but identity is the parent-linked owner graph. No
`__inner_space` or `__inner_namespace` node participates in canonical
ownership. This owner is not a meta-instance owner such as
`MetaInstanceOwner(meta_function, canonical_arguments)`.

### 4.3 Ordinary `meta`

`meta` is symbol-level staging. It creates or transforms a symbol construction:

```text
meta:
  accepted parameters
  -> SymbolConstructionValue : symbol
```

The public successful return rank is always `symbol`. A meta callable may accept
a `symbol` parameter, or constrain a parameter to a narrower `type` or ordinary
pattern-value rank. That does not change its public construction result rank.

Meta functions are divided into two semantic classes:

```text
MetaFunction
  |- OrdinaryMetaFunction
  `- BuiltinPrivilegedAstMetaFunction
```

Every ordinary canonical meta-function invocation establishes a virtual
symbol-construction scope:

```text
M = MetaInstanceScope(callee_symbol, canonical_arguments)
```

For:

```lang
let f = (self, t: type): meta -> r: symbol => { ... };
```

the diagnostic navigation projection of `M` is:

```text
(t f)
```

This is not merely a folder analogy. `M` is a symbol/namespace layer that
participates in default pattern navigation and name shadowing, may carry
namespace, type, and value facets, anchors cache/incremental identity, and owns
the return construction transaction. An ordinary meta invocation must therefore
establish its own symbol layer rather than act as a value-level forwarding
function.

The externally navigable result symbol is `M` itself. The declared return slot
is only a lexical construction handle to that symbol:

```text
symbol_of_result(invoke_meta(callee, canonical_arguments)) = M
return_slot(r) = lexical_handle(M)
```

The slot name `r` does not add another component to the final navigation path.
Material written through `r` contributes facets or children to `M`; it does not
create `r::M` or place an extra symbol named `r` beneath `M`. For example, a
pattern-child contribution written as `let t1::r = bool;` inside the invocation
targets `t1::M` under the applicable pattern-construction expectation, not
`t1::r::M`.

Canonical argument identity follows parameter rank:

```text
symbol parameter -> SymbolId / symbol-place identity
type parameter   -> TypeValueId
value parameter  -> PatternValue identity
```

The exact inclusion of `PlaceId` in a symbol-parameter key depends on whether
the callable observes the symbol's installation place. A key must not silently
replace symbol identity with type-value equality.

### 4.4 Ordinary meta return type self-root invariant

If the return symbol of an ordinary canonical meta invocation has a type facet,
the outermost pattern root of that facet must be the invocation's own `M`:

```text
type_facet(r) = tau
  => root_pattern_scope(tau) = M
```

This is identity equality between a pattern root and the meta-instance symbol
scope. It is not equality of rendered strings.

Consequently, both of these meta bodies are invalid:

```lang
let f = (self, t: type): meta -> r: symbol => {
    r = t;
    r;
};

let fn = (self, t: type): meta -> r: symbol => {
    r = uint8;
    r;
};
```

The right sides are valid external type values, but their `PatternValue` roots
belong to external scopes. Resolving `symbol(t)` or `symbol(uint8)` and reading
its value does not make that external root identical to `(t f)` or `(t fn)`.
Neither value may directly replace the return symbol's required type root.

A legal meta construction builds under its own scope:

```lang
let f = (self, t: type): meta -> r: symbol => {
    r = (t inner) |> struct;
    r;
};
```

Its complete pattern is:

```text
(t inner::(t f))::(t f)
```

External `PatternValue`s may be members of the self-rooted type; they may not
replace the root. For example:

```lang
let fn = (self, t: type): meta -> r: symbol => {
    let t1::r = bool;
    r;
};
```

keeps `(t fn)` as the return symbol's root and includes the externally owned
`bool::` value as a member beneath that root. It must not be summarized as
`type_facet(r) = bool::`.

The self-root check is conditional on a type facet. A return symbol with only a
namespace facet, ordinary value facet, or both does not acquire a synthetic
type facet merely to satisfy this rule.

### 4.5 Formal return material

Formal meta returns no longer encode generative versus forwarding behavior by
choosing between `=` and `===` on the return slot.

The canonical form is:

```lang
r = ...;
```

It applies the general value-binding rule to the return symbol under
construction. Whether the assigned material references an existing
`PatternValue`, computes new material, or projects a symbol facet is represented
inside the construction value. It is not selected by changing the formal return
operator, but any resulting type facet must pass the self-root invariant in
§4.4.

A `SymbolConstructionValue` is not restricted to newly generated structure
definitions. It may describe a fresh return symbol with its own `SymbolId` and,
once bound, a potentially independent `PlaceId`; it may also reuse existing
values as ordinary value-facet material or as members of a newly self-rooted
type construction:

```text
SymbolConstructionValue {
    return_symbol_identity,
    assigned_facets_or_values,
    optional_child_contributions,
    provenance,
}

assigned non-root value/member may equal an already existing PatternValue
```

Value equality remains independent of source name and navigation path and does
not merge symbol or place identity. However, that general identity separation
does not waive the type self-root invariant: `r = uint8` as a direct meta return
type installation is rejected after symbol resolution/value read, rather than
being reinterpreted as alias forwarding or accepted as an identity meta type.

This correction does not affect ordinary declaration alias syntax:

```lang
let a === b;
```

Formal return construction and ordinary symbol aliasing are different semantic
layers.

### 4.6 Built-in privileged AST meta functions

A compiler-defined privileged family uses the general function-object and meta
invocation framework without becoming user-definable macro capability:

```text
BuiltinPrivilegedAstMetaFunction {
    compiler_known_identity,
    accepted_normalized_ast_or_pattern_rank,
    required_ambient_construction_capability,
    result_rank,
    special_scope_rule,
    special_owner_rule,
    bounded_privileged_behavior,
}
```

These objects:

```text
participate in ordinary symbol-first lookup;
have function-object, type, and associated () identity;
use the ordinary invocation frame, including implicit self;
may accept a bounded Normalized-AST or pattern carrier;
remain graph-installation-free and binding-free;
return SymbolConstructionValue or an owned construction handle;
leave graph installation to an outer binding.
```

Unlike an `OrdinaryMetaFunction`, an individual built-in may define a special
scope/owner rule and need not create an independently navigable
`MetaInstanceScope M`. Users may call compiler-provided members but cannot
define new privileged AST meta functions. Privilege is member-specific: one
built-in's accepted carrier and bounded transformation do not imply a general
macro system or arbitrary AST rewriting.

`struct` and `inject` are the first specified members. Future candidates may
include explicit sum construction/extension, bounded AST injection, or a
facet-construction primitive, but each must receive its own capability boundary.

## 5. Physical Namespace Contributions and Meta Construction

Physical source contributions and meta-produced construction values use the
same symbol-world capability substrate.

For example:

```text
ns/
  impl.lang
  export.lang
```

Both implementation files may create distinct same-level children in namespace
`ns`.
The corresponding meta-shaped construction can be sketched as:

```lang
let ns = (): meta => {
    let r = ...;
    let r = r |> impl;
    let r = r |> export;
    r;
};
```

The example is semantic design notation. It does not introduce a new parser
special form or promise that these exact bodies execute in the current
implementation.

Both origins share capabilities for:

```text
declare symbol/facet material
inject a direct child into a construction
open a namespace facet
form a replayable contribution/delta
install a delta transactionally at the outer assembly/binding layer
```

Sharing a capability substrate does not give physical files an implicit meta
pipeline execution order:

```text
physical source fragments
  -> independently derived contribution/delta values
  -> transactional assembly of distinct direct-child deltas
```

The contribution set is not evaluated as `impl.lang |> export.lang` according
to filename, discovery, or source order. Each file is nevertheless a distinct,
closed `SourceConstructionUnit`: it may create and fully construct its own new
child subtree, but it may not reopen a child subtree created by the other file.
Distinct direct-child contributions can be installed transactionally;
same-child reopening, duplicate names, or incompatible facets are conflicts. No
partial merge is installed after failure.

The canonical namespace-origin, construction-unit ownership, physical-directory
authority, and cross-file merge rules are specified in
`symbol-construction-units-and-namespace-origin.md`.

## 6. Resolved Pattern Scopes

### 6.1 One uniform scope model

The canonical object is:

```text
ResolvedPatternScope
```

or, when emphasizing ownership:

```text
ResolvedOwnerPatternScope
```

A meta-function instance is itself a navigable pattern scope. The design does
not split construction into separate special cases based on whether source
syntax contains a distinguished outer pattern name.

Example:

```lang
let f = (self, t: symbol): meta -> r: symbol {
    r = (t first, t second) |> struct;
};
```

The current meta instance may have this diagnostic projection:

```text
(t f)
```

The fully resolved pattern is:

```text
(
    t first::(t f),
    t second::(t f)
)::(t f)
```

The single-field form uses the same rule:

```lang
let f = (self, t: symbol): meta -> r: symbol {
    r = (t first) |> struct;
};
```

Its fully resolved pattern is:

```text
(t first::(t f))::(t f)
```

The two examples do not represent “no top pattern” versus “a top pattern.” They
are both:

```text
explicit relative pattern components
  + ambient navigable pattern scope
  -> fully resolved pattern path
```

The explicit relative component may be empty. The ambient scope still exists
and still owns the resolved pattern layer.

### 6.2 Scope identity is not rendering

Forms such as `(t f)`, `first::(t f)`, or `first::t1::t` are diagnostic
projections. `ResolvedPatternScope` identity is not raw string concatenation.
Implementations may eventually represent it with a `PatternScopeId` plus
structured owner/child relations.

### 6.3 An ordinary meta invocation is one navigation atom

When an ordinary meta callee has an outer namespace path, the complete
invocation remains
one navigable symbol atom. If `Vec` is found under `std` and the argument is
`int`, the canonical form is:

```text
(int Vec::std)
```

Resolution proceeds as:

```text
resolve callee path Vec::std
  -> resolve argument int
  -> form canonical meta invocation
  -> treat the complete invocation as one navigable symbol atom
```

A child of the resulting instance is written:

```text
child::(int Vec::std)
```

These are not equivalent forms:

```text
(int Vec)::std   // invalid: invocation boundary cuts off the callee path
int Vec::std     // invalid: missing invocation-atom parentheses
```

The future semantic grammar may name this unit:

```text
MetaInstanceNavigationAtom :=
    '(' ArgumentProduct MetaCalleePath ')'
```

This is a future semantic/navigation rule. It does not change the current lexer,
parser, Raw AST, or Normalized AST in this PR.

## 7. `struct`

### 7.1 Public boundary

`struct` is a `BuiltinPrivilegedAstMetaFunction`, not an ordinary user-definable
meta function. It uses the general function-object/meta call framework but does
not create its own ordinary externally navigable `MetaInstanceScope M`.

The public semantic boundary is:

```text
struct:
  PatternSyntax / normalized pattern material
  -> SymbolConstructionValue : symbol
```

An implementation may carry AST or Normalized AST as a private structured
carrier. The public result rank is not AST and this capability does not expose a
general macro system.

### 7.2 Owner resolution

`struct` resolves its pattern owner from:

```text
the input pattern's explicit navigation
+ the ambient ResolvedPatternScope
```

It does not inspect the eventual left-side binding target.

The invariant is:

```text
struct pattern owner:
  determined by input pattern material and ambient pattern scope

left-side let binding/injection path:
  determines only the Place where the construction is installed
```

Therefore:

```lang
let t1::t = (...) |> struct;
```

does not reroot the right-hand pattern into the internal pattern scope of
`t1::t`. Its effect is:

```text
evaluate the right-hand struct invocation
  -> obtain an uninstalled SymbolConstructionValue with an already resolved owner
resolve the destination symbol/place t1::t
  -> bind/install the construction result there without changing that owner
```

Every construction value must therefore distinguish:

```text
install_place(V)
pattern_owner(V)
```

The two identities may differ.

### 7.3 Formal invocation boundary

Formal `struct` invocation is:

```text
graph-installation-free
binding-free
```

It does not install a `NamespaceDelta`. The current implementation may allocate
or attach registry-backed pattern material while forming the invocation value;
that allocation means the invocation must not be described unconditionally as
pure. Graph installation remains outside formal invocation.

## 8. `inject`

### 8.1 Privileged built-in

`inject` is a future `BuiltinPrivilegedAstMetaFunction`, parallel to `struct` in
trust boundary. It does not create an ordinary externally navigable
`MetaInstanceScope M`:

- it accepts normalized pattern syntax or an equivalent internal AST carrier;
- its public successful return rank is `symbol`;
- it does not re-enter the parser;
- it does not concatenate arbitrary tokens;
- it does not expose unrestricted AST-consuming capability to user functions;
- it performs only bounded pattern-child construction.

The source examples in this section are semantic sketches. They do not change
the frozen parser or introduce traditional `f(args)` call syntax.

### 8.2 Functional result

The operation is functional:

```text
inject:
  OpenOwnedConstructionHandle
  x ChildPatternMaterial
  -> OpenOwnedConstructionHandle

preconditions:
  construction_owner(input) = current construction unit
  construction_state(input) = open / uninstalled
```

`OpenOwnedConstructionHandle` is a capability-bearing view of an uninstalled
`SymbolConstructionValue : symbol`; it is not an arbitrary resolved `Symbol`.
Each successful call returns the next functional version of the same owned,
open construction. It does not mutate an already installed graph object and
does not install a namespace delta.

Only an outer binding/injection installs the result:

```lang
let target = result;
let child::target = result;
```

Discarding the returned construction produces no symbol-world side effect.

### 8.3 Owner rule

The owner distinction is:

```text
struct:
  resolve owner by ordinary input navigation + ambient scope

inject:
  explicitly select the owned construction handle's symbol scope as owner
```

Example:

```lang
let t1::r =
    t1::r
    |> inject(t first)
    |> inject(u second);
```

Here `r` and `t1::r` are handles into the construction currently owned by the
same `MetaConstructionUnit`; the example is not permission to resolve and
reopen an arbitrary installed symbol. A handle returned by another construction
unit may be composed only while an explicit composition rule preserves
ownership and open state. An installed result, or a subtree owned by another
unit, cannot be passed to `inject` for reopening.

The resulting pattern is:

```text
(
    t first::t1::r,
    u second::t1::r
)::t1::r
```

`inject` changes the child set of the selected owner construction. It does not
change owner identity.

As with `struct`, the lowest-level leaf reduction has the form:

```text
E name
```

At that leaf:

- `name` is the leaf's pattern name;
- `E` is value-bearing material that must be resolved through its external
  symbol binding and then evaluated;
- different leaves do not require the same `E`.

Consequently:

```text
t first
u second
```

means:

```text
first is the pattern name; the leaf value is read through symbol t
second is the pattern name; the leaf value is read through symbol u
```

Pattern-name identity and leaf-value origin are independent. Using `t` for both
leaves would obscure this distinction.

### 8.4 Child-only restriction

`inject` may only:

- continue the current unit's owned, still-open pattern construction;
- add direct children;
- preserve the selected owner.

It may not:

- replace the owner;
- overwrite an existing type facet;
- delete an existing child;
- implicitly reroot an arbitrary external pattern value;
- directly mutate the installed namespace graph;
- accept an arbitrary installed `Symbol` as reopening authority;
- cross a `SourceConstructionUnit` or `MetaConstructionUnit` ownership boundary;
- bypass place writability or delta installation;
- grant a general macro or arbitrary AST-rewrite capability.

## 9. Pattern-Layer Ordering

Let the direct children of one pattern layer be:

```text
p1, p2, ..., pn
```

The ordering rule is decided at the level as a whole.

### 9.1 Fully named layer

If every direct child has a top-pattern navigation layer:

```text
normalize layer -> Set<PatternValue>
```

For example:

```text
{
    bool::,
    t1::t,
    t2::t
}
```

Every set element is an already evaluated `PatternValue` with a fully qualified
top-pattern navigation name. It is not a `Symbol`, a symbol path, or a symbol
reference. The navigation name participates as part of the `PatternValue`
itself, not as a separate name-map key.

Consequences:

```text
the whole layer is order-insensitive;
set equality is PatternValue equality;
different-name injections commute;
construction-time duplicate-path conflicts are validated before set formation.
```

For example:

```lang
t1::r
|> inject(t first)
|> inject(u second)
```

and:

```lang
t1::r
|> inject(u second)
|> inject(t first)
```

produce the same pattern value because both direct children have top-pattern
names.

Once normalized, the set does not classify elements as “internal patterns” or
“external patterns.” Parent-scope inheritance, explicit `::`, ordinary symbol
binding, and `inject` explain how a `PatternValue` was resolved or produced
before normalization. After its navigation name is fully qualified, source
category and construction route do not participate in `PatternValue` identity,
set equality, or extraction semantics.

An implementation may retain source symbol, inherited/explicit navigation,
binding origin, or injection origin as provenance for diagnostics and replay.
That provenance must not affect `PatternValue` equality.

Because the normalized layer is a mathematical set, insertion is idempotent by
`PatternValue` equality. Distinct source symbols may remain distinct extraction
entry paths while contributing only one set element:

```lang
let a::t = bool;
let b::t = bool;
```

```text
value(symbol(a::t)) = bool::
value(symbol(b::t)) = bool::

{ value(symbol(a::t)), value(symbol(b::t)) }
  = { bool:: }
```

Both `a::t` and `b::t` may be used as source navigation paths. After symbol
resolution and value read, both look up the single `bool::` member. The layer
is therefore neither a multiset nor a name-keyed relation.

Symbol paths and `PatternValue` navigation names may coincide or differ. For
example, the same spelling may describe:

```text
symbol navigation path:                 t1::t
PatternValue navigation carried there:  t1::t
```

The element `t1::t` in a normalized set is still a `PatternValue`; its spelling
does not turn it into a symbol reference. Conversely:

```lang
let t3::t = bool;
```

may establish:

```text
symbol navigation path:                 t3::t
PatternValue navigation carried there:  bool::
```

The symbol path and value path are then visibly different. Both cases use the
same symbol-resolution/value-read semantics.

### 9.2 Layer containing a bare value

If at least one direct child is a bare value:

```text
the entire current layer is order-sensitive;
positions participate in identity;
the layer cannot be replaced by a name map.
```

The rule is not “only the bare child is ordered.” The presence of one bare
value makes the complete sibling layer positional.

### 9.3 Representation guidance

An implementation may distinguish:

```text
No-bare normalized layer:
  representation = Set<PatternValue>
  membership/equality use canonical PatternValue identity
  order-insensitive

OrderedPatternLayer:
  position-preserving, order-sensitive
```

A canonical serializer may sort a fully named set by canonical `PatternValue`
encoding. Sorting is only a stable representation of set semantics; it must not
be presented as preserved source-order meaning. An ordered layer must preserve
positions.

## 10. Child Uniqueness and Replay

“Inject once” applies to a complete child navigation path, not to the owner as a
whole.

For named direct children, the conceptual uniqueness key is:

```text
(owner PatternScopeId, child top-pattern identity)
```

This is a construction-time path-conflict key, not the representation of the
normalized layer. After successful validation/evaluation, the child contributes
its fully qualified `PatternValue` to `Set<PatternValue>`.

Therefore:

```lang
|> inject(t first)
|> inject(u second)
```

is valid, while:

```lang
|> inject(t first)
|> inject(u first)
```

is a conflict because both attempt to create:

```text
first::owner
```

Cache replay remains idempotent only for the same origin and material:

```text
same owner + same child + same construction origin/material
  -> reuse / idempotent replay

same owner + same child + different material
  -> hard conflict
```

Replay origin controls whether a construction action may be reused; it does not
become part of the resulting `PatternValue` identity.

An ordered layer still preserves positional identity; a symbol-keyed or
name-keyed map must not replace either the ordered layer or the normalized
`Set<PatternValue>`.

## 11. Extraction and Explicit Navigation

### 11.1 Navigation always reaches a symbol before a value

Both inherited and explicit pattern navigation use the same final two steps:

```text
symbol resolution
  -> value read
```

They differ only in how the symbol path is formed.

A bare name first attempts bounded completion under the current parent pattern
scope:

```text
name
  -> complete as name::current_scope
  -> resolve that completed Symbol path
  -> read the PatternValue carried by that Symbol
```

An explicit external navigation does not inherit the current parent scope:

```text
::external
  -> begin at the explicitly selected external Symbol layer
  -> resolve that Symbol path
  -> read the PatternValue carried by that Symbol
```

In the current inner-to-outer surface notation, an explicitly terminated
external component is written as `external::` where a grouping boundary is
needed. The conceptual `::external` description above emphasizes that the
external layer is selected rather than parent-completed; it does not reverse the
frozen source navigation order.

Default inheritance is therefore not “indirect value access” while explicit
navigation is “direct value access.” Neither form directly touches a pattern
value. Both resolve a symbol path and then read its value.

The pattern expectation permits only a `PatternValue`/pattern interface exposed
by that symbol. It does not fall back to invoking arbitrary ordinary values or
callables from the heterogeneous value facet.

### 11.2 Binding a fully qualified PatternValue through another symbol

Consider a globally defined symbol construction:

```lang
let bool = ((if | else) bool) |> struct;
```

Two semantic objects may share the diagnostic spelling `bool`:

```text
symbol(bool)
pattern head bool
```

They are not one identity. The first is the source-resolved symbol. The second
is the owner/head projection inside the `PatternValue` carried by that symbol.

Now:

```lang
let t1::t = bool;
```

uses the general value-binding rule:

```text
resolve symbol(bool)
  -> read its PatternValue, whose fully qualified navigation is bool::
resolve destination symbol/place t1::t
  -> bind that same PatternValue to t1::t
```

For normalized-pattern explanation, the relation may be written:

```text
(bool::)t
```

This does not reroot the value, rewrite its navigation, change its top name to
`t1`, identify `symbol(t1::t)` with the `bool` pattern head, or create an
internal `bool` pattern under `t1::t`.

The accurate normalized statement is:

```text
symbol t1::t is bound to a PatternValue whose fully qualified navigation is bool::
```

The source binding route may be retained as provenance, but “external” versus
“internal” is not a category in normalized `PatternValue` identity.

### 11.3 Inherited and explicit extraction are equivalent here

With the binding above, the extraction shorthand:

```lang
let P t1 t = t;
```

and the explicit form:

```lang
let <P> ((P)bool::)t = t;
```

denote the same extraction.

For the shorthand, resolving bare `t1` inherits the current outer navigation
layer `t`, producing the symbol path:

```text
t1::t
```

The evaluator then resolves `symbol(t1::t)` and reads its bound
`PatternValue`. That value reveals its fully qualified pattern navigation:

```text
bool::
```

For the explicit form, `bool::` explicitly terminates the external symbol path
(the conceptual `::bool` choice) and blocks completion under the current parent
`t`. The evaluator resolves `symbol(bool)` and then reads the `PatternValue`
carried by that symbol.

Both paths therefore reach:

```text
P = if::bool | else::bool
```

The distinction is solely:

```text
inherited form:
  parent-complete a symbol path, then resolve Symbol -> read PatternValue

explicit form:
  select an external symbol path, then resolve Symbol -> read PatternValue
```

It is never a distinction between an indirect pattern value and a directly
named pattern value. Source navigation names symbols first. A pattern's
canonical/diagnostic navigation may match a source symbol spelling without
becoming the same identity.

### 11.4 Extraction looks up PatternValue in a set

For a layer with no bare values, normalization produces:

```text
S: Set<PatternValue>
```

Extraction is therefore value lookup, not symbol lookup. The normative process
is:

```text
1. Complete the source navigation path from the parent pattern scope, or honor
   explicit `::` without parent completion.
2. Resolve the completed path to a Symbol.
3. Read the PatternValue bound to that Symbol.
4. Look up that PatternValue in Set<PatternValue> S.
5. If present, continue extraction through the matched PatternValue.
```

Formally:

```text
extract(path, S)
  = lookup(value(resolve_symbol(path)), S)
```

not:

```text
lookup(resolve_symbol(path), S)
```

because `S` contains evaluated, fully qualified `PatternValue`s, not symbols or
symbol references.

For example:

```lang
let bool = ((if | else) bool) |> struct;
let t3::t = bool;
```

and:

```text
S = {bool::, t1::t, t2::t}
```

the extraction path:

```text
t3 t
```

first inherits parent navigation and forms symbol path:

```text
t3::t
```

Then:

```text
resolve_symbol(t3::t) = symbol(t3::t)
value(symbol(t3::t)) = bool::
bool:: ∈ S
```

Thus `t3 t` matches `bool::`, not `t3::t`.

By contrast, if:

```text
value(symbol(t1::t)) = t1::t
```

then the source symbol path and resulting `PatternValue` navigation happen to
share a spelling. The extraction still performs symbol resolution and value
read before set lookup; the shared spelling does not permit either step to be
omitted.

## 12. Facet Conflicts and Installation

### 12.1 Contribution expectation selects the facet

A navigated child binder does not determine its contribution facet from the
runtime shape of the right side. The enclosing semantic position supplies a
construction expectation, optionally made explicit by a rank/facet annotation:

```text
ContributionExpectation =
    PatternChild
  | NamespaceValueMember
```

Under `PatternChild`, the source path is resolved to a symbol and projected to
its type/pattern value. The resulting `PatternValue` is installed as a child of
the owner's type construction and participates in normalization and extraction:

```text
resolve source Symbol
  -> project/read PatternValue
  -> contribute to owner TypeFacet(PatternValue)
```

The earlier `let t1::r = bool;` meta example is interpreted under this
expectation, so `bool::` becomes a member below the self-rooted meta result.

Under `NamespaceValueMember`, the source is projected through its ordinary
value facet and a namespace value symbol is constructed. This changes only the
namespace graph/value facet; it does not enter or change the owner's
`PatternValue`:

```text
resolve source Symbol
  -> project/read ordinary value
  -> construct namespace value member
```

This is also the expectation of an ordinary let-shaped declaration consumed
inside `struct` construction:

```lang
let name = expr
```

It contributes only to the current Pattern owner's `Val2`/namespace value
facet:

```text
Val1 contribution    = none
Pattern contribution = none
Val2 contribution    = value entries produced by expr
```

The initializer is not restricted to type/Pattern material or to `Pv=absent`.
It may contribute any ordinary heterogeneous value entry, including a callable
function object. The construction stores this as an uninstalled child
contribution; it does not mutate the namespace graph during `struct`
evaluation.

The empty destination `()` is the special call-entry leaf rather than a normal
value-member name. Inside construction of `T`, `let () = impl` contributes
`()` below `T` only. A separate `()::ref::T` or `()::share::T` requires a
separate authorized contribution. The body of an associated `()` entry still
has its own `CallableOwner`, while invocation-frame slot 0 receives the object
whose type supplied that entry.

Under equal owner/construction authority, an inner contribution and a later
inner-to-outer navigated declaration denote the same pending namespace delta:

```text
struct-local contribution under owner name1::T
  ==
later installation at name::name1::T
```

Neither spelling creates an alias or reroots the initializer's Pattern.

The language must select the expectation from semantic context or an explicit
rank/facet annotation. It must not guess `PatternChild` merely because the
right side happens to carry a type or `PatternValue`. Both paths still obey the
general symbol-resolution-then-facet-projection rule.

### 12.2 Same-symbol facet rules

The future symbol-facet direction is:

```text
namespace facet:
  establish the facet from exactly one NamespaceOrigin;
  add children only under the owning construction/authority rules

type facet:
  install once by ordinary definition

value facet:
  admit multiple heterogeneous value entries;
  form candidates only in a call position;
  do not infer cross-construction-unit merge authority
```

When `struct` establishes a type/pattern facet inside an already resolved owner
pattern scope, an existing incompatible facet is a hard conflict. Same-origin,
same-material cache replay may reuse the existing facet.

In particular, an ordinary symbol place receives its type facet at most once:

```lang
let T = A;
let T = B;
```

If both declarations attempt to install `T`'s type facet, the second is a hard
conflict. It is never interpreted as:

```text
A | B
```

Three operations must remain distinct:

```text
first type-facet installation
  -> ordinary type installation

add a direct child under an owned, still-open construction
  -> inject or another explicit child-construction API

construct or extend a sum
  -> explicit sum-construction / sum-extension API
```

The final spelling of the sum API remains open. Duplicate ordinary definitions
do not provide that API, and `inject` must not convert an existing type or an
existing child into an implicit sum.

An explicit read-transform-bind form such as:

```lang
let T = T |> some_explicit_transform(...);
```

conceptually reads the existing value, applies a named structural
transformation, and asks the outer binding/update judgment to install the new
value. Whether that writeback spelling is permitted is reserved for later
place/update rules. It does not make two unrelated ordinary definitions
mergeable.

### 12.3 Value identity does not multiply with names

Do not infer three type values from:

```lang
let Bool = value;
let bool = value;
let t === bool;
```

If the bindings expose the same pattern/type value, the value identity is the
same. Their `SymbolId`, `PlaceId`, and alias/provenance relations remain
separately observable according to their declaration forms.

### 12.4 Installation is always outer-layer work

The installation flow is:

```text
compile/meta invocation
  -> PatternValue or SymbolConstructionValue
  -> for a source path: resolve Symbol -> read its value/facets
  -> let binding/injection judgment binds that value/construction
  -> resolve writable install PlaceId
  -> form NamespaceDelta
  -> validate facet/child conflicts
  -> install atomically or install nothing
```

Neither `struct` nor functional `inject` directly mutates the namespace graph.
Graph installation always occurs in the outer declaration/binding layer.

## 13. Current Implementation Substrate

The PR #94 implementation remains a neutral transitional
identity/materialization substrate. It currently provides:

- a doc-hidden explicit context attachment helper for generated type-definition
  pattern heads, retained publicly only for integration-test support;
- categorical `Generated`, `GeneratedTypeDefinition`, `Global`, `Namespace`,
  and `Local` materialization contexts as low-level registry test/materialization
  categories, not final language owner scopes;
- `GeneratedTypeDefinition` as the formal-invocation and binding-time fallback
  for cache-safe anonymous reattachment;
- binding that preserves already attached provisional material and does not
  derive owner identity from the destination global/namespace path;
- registry-backed owner/field `PatternHeadId` allocation and bounded child
  lookup.

This substrate does **not** implement:

- `PatternScopeId` or `ResolvedPatternScope`;
- `MetaInstanceScopeId` or a meta-instance pattern scope such as `(t f)`;
- meta return type self-root validation;
- the canonical meta-invocation navigation atom;
- `SymbolCell` facets;
- the `compile` / `meta` capability split specified here;
- `SymbolConstructionValue` as the public meta result model;
- functional `inject`;
- `OpenOwnedConstructionHandle` ownership/open-state enforcement;
- contribution-expectation-driven pattern-child versus namespace-value facet
  selection;
- an explicit sum construction/extension API;
- the final owner-resolution rule for `struct`;
- fully named `Set<PatternValue>` versus ordered pattern-layer representation;
- namespace-origin uniqueness or source/meta construction-unit ownership;
- physical-directory contribution authority or cross-file reopening checks;
- the structural `TypeFacet`-implies-`NamespaceFacet` model;
- the distinction between ordinary namespace value members and
  pattern-material leaves as implemented facets;
- full alias/place/writability checking;
- graph installation from the construction model in this document.

The categorical global/namespace/local contexts remain available only to the
doc-hidden low-level attachment helper and registry tests. They are not a
stable external owner-construction capability. The ordinary binding path does
not select among them: it preserves attached provisional owner
material, or restores stripped material under the anonymous
`GeneratedTypeDefinition(type_definition_id)` fallback. It must not be
described as determining or rerooting `struct` pattern-owner identity or a meta
return type's root. In final semantics, the meta instance's own symbol scope
anchors that root.

Formal `struct` invocation currently may allocate or attach registry material
under `GeneratedTypeDefinition`. It remains graph-installation-free and
binding-free, but it is not unconditionally pure.

## 14. Non-Goals of This Note

This document does not:

- change the parser, Raw AST, or Normalized AST;
- introduce traditional call syntax;
- implement `inject`;
- define a general macro system;
- allow users to define new `BuiltinPrivilegedAstMetaFunction` members;
- expose arbitrary AST or token rewriting;
- implement type checking, name resolution, overload resolution, pattern-space
  execution, extraction execution, D/Done, ownership, runtime evaluation, or
  code generation;
- require the current Rust `SymbolObject`, `PatternHeadId`, or meta invocation
  enums to implement the future objects defined here. PR #94 only neutralizes
  destination-derived owner attachment in the existing substrate.

## 15. Required Direction for Later Implementation

Future implementation should converge in this order:

```text
SymbolCell / facet-aware resolution
  -> PatternValue identity and rank-directed canonical arguments
  -> SymbolConstructionValue
  -> ResolvedPatternScope / PatternScopeId / MetaInstanceScopeId
  -> namespace origin and construction-unit ownership
  -> meta return type self-root validation
  -> struct owner resolution independent of binding place
  -> functional child-only inject
  -> explicit sum construction/extension
  -> fully named Set<PatternValue> / ordered-layer representation
  -> writable let binding/injection
  -> NamespaceDelta atomic installation
```

Until those objects exist, the current attachment registry is useful substrate,
but documentation must keep the substrate/final-semantics gap explicit.
