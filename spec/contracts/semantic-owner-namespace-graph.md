# Semantic owner and namespace graph contract

**Status: current semantic/build contract.**

The language-level Pattern relation, binderless Pattern semantics, and the
distinction between ordinary `Val2` presence and direct structural incidence
are canonical in
`../design/patterns-overload/pattern-values-relational-semantics-and-extraction.md`.
This contract owns only the persistent owner/root/binder identities and the
build handoff consumed by that relation.

## Identity

Long-lived semantic identity is owner-based:

```text
NameBindingId = (DeclOwner, LocalBindingIdentity)
GeneratedIdentity = (GeneratingOwner, LocalGenerationIdentity)
SemanticOwnerId = (SemanticOwnerGraphId, graph-local owner)
```

NameBindingId identifies the structural binding relation and its resident Place;
it is not an Object or a value with a .type field. GeneratedIdentity and
SemanticOwnerId index semantic ownership, not an extra result ontology.

Source files, paths, spans and printable navigation strings are provenance.
Child-directory selectors are first normalized into ordinary fresh-name actions;
only their evaluation establishes named types/Places/owners. Root levels and
implementation filenames add no segment. The physical tree installs no owner
by discovery; see [normalization](../design/build-package/build-system-design.md).

The semantic owner forest contains source-established namespace owners, callable owners,
canonical meta-instance owners, and generated owners. Every callable,
including an in-place closure, has a lexical/code owner. A standalone closure
materialization also has an owner-derived anonymous function-object type:

```text
DefaultStandaloneReceiverType(C)
  = AnonymousType(CallableOwner(C))
```

CallableOwner and the callee's type are distinct facts. Every invocation
requires Type(callee) = Type(first self). A T ref callee uses its own matching
entry; a named T entry does not coerce its callee to repair a receiver mismatch.

Nested callable-local `Self` owner paths follow the language's source-order
navigation: innermost/current `Self` first and outermost `Self` last. The
printable string is not identity and does not determine any receiver type.
`__inner_namespace` and related synthetic path components have no canonical
role.

Allocating this owner is not closure-value materialization. A closure remains
syntax/normalized callable material until an explicit binding or call context
requires a value; the owner exists earlier only so `Self`, return targets,
Pattern roots, and nested declarations have stable semantic containment.

Every invocation has frame slot 0 for its caller object. This is independent of
ordinary/in-place placement. When a closure writes any formal position, its
first written formal is the explicit Pattern for that slot under any legal
spelling; the caller object is passed implicitly. Only later written formals
consume explicit call-site Product positions. A headless or empty-parameter
closure still has the semantic slot but no written binder for it.
Compiler-generated receiver helpers write a generated self formal before their
explicit `val` receiver.

A standalone function object is its own first self. An ordinary field
function supplies its anonymous function object first and consumes the
operated object as a subsequent argument. A directly callable instance is
first self for the associated entry of that instance's exact complete type.
Anchored replication may create an eligible new closure instance; it cannot
modify the original callable's owner or capture lifetime.

A meta instance is interned by:

```text
(parent owner, resolved meta-function identity, canonical argument key)
```

Repeated canonical invocation reuses its owner. A different canonical key
creates a different child owner.

Source navigation remains inner-to-outer. A generated meta-call scope used as
one outer component must group the complete call expression:

```lang
child::(int Vec::std)
```

It is not `child::int Vec::std` and not `child::(int Vec)::std`. Namespace
graph traversal may internally reverse source components to walk containment
outer-to-inner; that implementation order is not source syntax.

## Pattern roots and alpha identity

Each independent extraction establishes:

```text
PatternRootId = (SemanticOwnerId, local root)
HoleBinderId  = (PatternRootId, local binder)
```

Independent top/local let patterns and callable heads create new roots.
Nested BindingSlots, Products, Sequences, annotations, DeduceLists, and Packs
inside one extraction remain in that root.

DeduceLists remain left-to-right telescopes. Duplicate source hole names are
rejected only within the current PatternRoot. A new root may shadow an
inherited spelling:

```lang
let f = <A>() => {
    let g = <A>(self, x: A) => { x }; // valid: new callable owner/root
};
```

The following remains invalid:

```lang
let <A> (let <A> x) = value; // same extraction root
```

The alpha environment therefore carries two facts:

```text
visible inherited/local holes
same-root declaration table
```

Lookup searches the visible lexical environment. Duplicate checking consults
only the same-root table. Generated hole keys remain hygienic and are
root-qualified during alpha conversion.

The frontend allocates collision-safe `AlphaOwnerId × NormOwner × PatternRoot`
identities. When normalized material enters the build graph, the Norm owner is
mapped to a persistent `SemanticOwnerId`, producing a resolved
`SemanticOwnerId × local root × local binder` identity before multi-root
semantic comparison.

## Namespace views

Namespace membership has three distinct projections:

```text
FullNameView
ExternalNameView
DefaultExtractionView
```

They remain orthogonal to:

```text
Σ_full
Σ_export
Wfinal = Wpre ∪ Wseal
```

Non-export does not imply private or same-level-only. Existing lexical
ownership and explicit navigation rules determine access: lexical descendants
can see their enclosing bindings, while sibling ownership alone supplies no
unqualified lookup. External navigation consumes the source-established export
view and public/private reachability. Build configuration defines no additional
visibility domain.

Path/name resolution returns one terminal NameBinding, preserving the resolved
host chain and exposure context. It does not return a candidate set:

    Resolve(path) -> terminal NameBinding
      -> read resident complete named type
      -> consumer projection (including exposed call candidates)
      -> admissibility and unique overload selection

An empty or inapplicable projection never restarts name resolution. Candidate
identity sets belong to the projection consumer, not the resolver.

`ExportRetentionClosure` is a retention/materialization closure. Membership
does not itself mean external visibility. External candidate projection keeps
the internal candidate identity, resolved `PolicyPair`, and whole-slot
`PolicyMode`. Stable external admission requires public path reachability and
never consumes a future caller's Policy or capability demand. Ordinary
capability-family applicability and Policy selection occur only after external
lookup.

External resolution retains typed failure reasons:

```text
Unresolved
NotInExportRetentionDomain
PrivatePath
NoExternallyEligibleCandidate
```

Stable namespace projection separates namespace admission failures from later
consumer Policy-selection and dynamic-legality failures.

## Structural members and associated Val2 contributions

`struct` forms a complete type value `tau` whose core `Q_struct = Core(tau)`
satisfies `Pure(Q_struct)` and `TypeRole(Q_struct)`; structural leaves and
associated lets contribute to that construction. Generated field/access/
assignment/borrow partners are ordinary members entering `V_τ` at the formation
event; a name binding appears only at a subsequent binding/install of the formed
value. Same-name associated `Val2` named types expose those same members and own
no second copy. Members satisfying `TypeOf(v) in Q_struct` are part
of `V_τ`, and the formed closure is `tau = <Q_struct,V_τ>`. Copied/extracted
type-as-callee uses `CallSpace(tau)=V_τ`; there is no defining-name binding or
recent-carrier recovery route.

The generic parser preserves the narrow postfix shape:

```lang
E name [[public]]
E name [[private]]
```

Only the struct decoder interprets it as:

```text
StructuralMember {
  evaluator = E,
  name,
  visibility = Default | Public | Private
}
```

The annotation slot accepts only `public` or `private`; it is not a general
PolicySpec.

Struct/type construction may also consume an ordinary let-shaped declaration:

```lang
let name = expr
public let name = expr
private let name = expr
let () = callable_expr
```

A named declaration here is in an explicit named-contribution position:
it first uses FreshNamedType if the name is fresh, otherwise the existing
named type. Eligible closure contributions enter that named type's V_tau
under Writable, OpenHere and final anchored membership checks. Required Core
construction uses extend/inject, not an implicit side effect of type +=.

```text
named selector -> structural binding / Place
occupied resident -> complete named type T
contribution -> T.V_tau through ordinary type contribution
```

The initializer is not inserted directly as Val2[name]. Neither the binding
identity nor an independently collected initializer bag is a Val2 Object.
This form does not register a structural field or replace the postfix field
form. A named target requires one plain binder; extraction/Product/Sequence/Pack
targets do not acquire named-contribution sugar. The empty Product target ()
is separately the current exact callee type's call-entry construction position.
Closure syntax there supplies complete entry implementation material under its
own rules; it does not authorize general receiver adaptation.

Structural P let name::path instead commits Some(T_0), returns mut type ref,
then performs ordinary assignment for its = suffix. Ordinary lexical let uses
neither named-contribution sugar nor structural fresh-name formation. Final
membership for a type contribution may use witnessed InstantiateUnder without
feeding an LHS anchor backward into RHS parsing or normalization.

Named callable contributions remain ordinary function objects. Their first
written formal is their own caller/self; an object accepted by a member-like
function is therefore the second written formal. The `()` target is different:
it installs an implementation for invocation of the current owner itself, so
its first written formal receives that invoked object.

Writing local `let ()` while constructing `T` contributes one candidate to the
same associated `()` name binding. It does not also synthesize receiver candidates for
`T ref` or `T share`; each distinct callee type requires its own matching first-self entry.

Accessor stage follows one structural rule:

```text
RuntimeField(f)
  iff Val1_f != absent
  and Materializable_0(Val1_f)
  and not RequiresStaticPattern(f)
```

`RuntimeField` generates runtime-or-compile accessors; every other field is
compile-only. Type is not a special field category. Generated Sequence `[]`
uses `RuntimeField` only for its selected observation; its complete stage is the
ordinary dependency meet over container, index, and selection. Its ordinal
slot is under the resident bare Product in `Val1(sequence)`, not under the
outer Sequence Object's generated `Val2`.
`OpenHere_Σ(v)` is per-value and per-window: it depends only on
`WindowLive_Σ(v)`, `AuthorityMatches(v, Σ)`, and the current evaluation
coordinate. No parent-to-child or child-to-parent implication holds; a terminal
event that closes multiple windows in one structural region does so because
each value independently fails `WindowLive_Σ` or `AuthorityMatches`, not
because a neighboring value closed. Borrow edges are horizontal and do not
participate. Mutability remains independent and does not propagate.

Default and Public structural members enter the current default extraction
view. Private structural members remain present in the complete structural
model and FullNameView, but are absent from DefaultExtractionView. A future custom `?`
may construct a richer extraction interface, but may not directly expose a
private member without a future explicit capability rule.

## Source-established navigation and closure

Namespace/index edges project ordinary committed source actions. Physical
normalization has no mount or package authority. Redirect representations can
encode an already established path without copying the target's identity.

External navigation into a meta construction result waits until its visible
name set is closed. Anonymous implementation layers remain under /tau.
SemanticOwnerId and navigation nodes need not be one-to-one: callable owners
and Pattern roots may have no ordinary user navigation entry.

## Current implementation boundary

Implemented substrate:

- parent-linked `SemanticOwnerGraph`, namespace/callable/meta/generated owner
  interning, owner-derived standalone anonymous callable types, an explicit
  callable-owner/receiver-type binding carrier, hygienic meta-instance
  interning, and owner-derived symbol identity;
- frontend callable owners, PatternRoot alpha boundaries, same-root duplicate
  validation, cross-root shadowing, and an explicit
  `SemanticOwnerQualification` handoff that rejects an unmapped or
  inconsistently remapped frontend owner before a hole identity enters the
  build world;
- owner-aware namespace views and public/private path checks. Configured
  package/mount routing is an implementation path pending source-normalization
  migration; it is not canonical semantic authority;
- narrow member-view Raw/Norm shape, struct structural visibility, private
  extraction filtering, and a typed associated-Val2/call-entry contribution
  decoder that preserves value-bearing initializer policy.

Deferred:

- stable serialized owner fingerprints and incremental graph persistence;
- a parent-homomorphism proof for the frontend-to-persistent owner
  qualification:

  ```text
  Map(Parent_frontend(x)) = Parent_persistent(Map(x))
  ```

  `SemanticOwnerQualification` currently proves only that an exact frontend
  owner has a mapping and that a repeated mapping does not conflict. It must
  not be described as validating the complete owner-tree embedding;
- end-to-end connection of every namespace snapshot consumer;
- wiring owner-namespace admission to the canonical
  `ResolvedCandidateSnapshot` / `ExportCandidateView` payload and complete
  external overload routing. That payload retains identity, pair, mode,
  declaration/intrinsic realization facts, and provenance but no
  context-indexed `DynamicLegality_Γ`; the current entry records eligibility
  and preserves symbol identity but does not duplicate the candidate projector
  or form consumer legality;
- recursive materialization of visibility-bearing structured fields beyond
  the current simple-field struct slice (the decoded Pattern retains the
  visibility metadata);
- full custom `?`, general Pattern execution, capture discovery, closure
  materialization, lifetime checking and ABI/layout/materialization;
- end-to-end installation of associated Val2 contributions, external navigated
  call-entry declarations, and ordinary type checking of the slot-0 receiver
  against the first written formal;
- source structural-let expression, named-type synthesis, associated state,
  anchored replication and unordered overlay consumer alignment.

The owner-tree homomorphism proof and the persistent namespace consumer/routing
migration are **P1 integration gates**. They do not reopen the semantic
definitions of `SemanticOwner`, `PatternRoot`, namespace views, or overload
uniqueness established by this contract.
