# Name Resolution, Meta Construction, and Pattern Injection

Status: canonical construction semantics. This owner preserves the existing
meta identity, root conservation, Pattern construction, extraction, and open
window rules. Source wiring is tracked in the roadmap.

## 1. Canonical boundaries

[Names and OverloadGroups](names-and-overload-groups.md) owns name occupancy,
ordinary group algebra, member projection, and structural let expressions.
A name resolves once to a binding denoting its named complete type. Consumer
projection never reopens lookup. Complete pattern values are self-contained
immutable tau values; a group contributes no additional type callspace.

Ordinary lexical let binds a value at fresh destination bindings and Places.
At a named-contribution position, unqualified let synthesizes the named type's
V_tau through ordinary type contribution. Explicit P let name::path creates a fresh name and
returns a mutable construction type reference. Following = e is ordinary
assignment to that reference. No special initialization transaction is implied.

compile computes ordinary values with the root-conservation rule below.
Ordinary meta establishes one stable MetaInstance root and seals its owned
result at return. struct returns a complete tau; extend is a pure value
transformation; inject is read + extend + write. Their allocation material is
private execution machinery, not a returned ontology.

Physical files contribute normalized meta blocks under the
[source composition](symbol-construction-units-and-namespace-origin.md) rules.
They do not own construction authority. Ordinary globally associated compile
state is defined in [associated compile state](associated-compile-state.md).

The [pattern-value owner](type-values-places-and-borrow-views.md) defines
Object normalization, Core/whole observations, Places and borrow views.
The [Pattern owner](../patterns-overload/pattern-values-relational-semantics-and-extraction.md)
defines R_Gamma and structural incidence.
The [policy owner](symbol-policy-and-compile-flow-projection.md) defines pair,
mode, migration and selection. [Evaluation](../meta-invocation/evaluation-residual-and-optimization.md)
places all of these on the same continuation; [lifecycle](../lifetime/lifetime-policy-and-overload-boundary.md)
and [safety admission](../lifetime/unsafe-semantic-admission.md) govern its observations.

## 2. Name-first resolution and member projections

### 2.1 Named types and ordinary candidate groups

An existing structural name denotes a complete named type T. Same-name
contributions form its V_tau; they do not build an OverloadGroup at that name.
An ordinary OverloadGroup aggregates type candidates through singleton eta(T)
and its bucket relation. Empty groups and candidates without callable members
remain legal. See the name/type algebra owner for the distinct update rules.

    tau = bind alpha.<Q, V_tau[alpha]>
    Core(tau) = Q
    CallSpace(tau) = V_tau

Each complete tau (T in the name/type algebra owner's notation) is formed
before it is contributed as a value. Its callspace
is intrinsic and immutable; copying, transporting, or adding another group
entry does not amend it. Ordinary type equality/keying observes Core, while
explicit whole-snapshot observations retain the bound closure.

TypeMember_Q(F) requires TypeOf(F) in Q for an ordinary complete closure
included in the constructed snapshot's implementation space. Construction actions must satisfy existing
authority and OpenHere; membership is not inferred from file provenance,
lexical-parent topology, or a special implementation declaration. Functions
retain their own CallableOwner and complete anonymous implementation layer
under /tau. A callable already belonging to Q can be used directly. Otherwise only a
ReplicableUnder witness permits InstantiateUnder to create a new anchored
instance; the old callable, its captures and owner remain unchanged. Internal
identity references are renamed consistently. See
[anchored replication](closure-anchored-replication.md).

    CallProjection(G)
      = ordinary call candidates contributed by each entry of G
    Type(callee) = Type(first self)

For a value member, call projection reads its exact captured tau and associated
(). A complete pattern value uses its documented type-as-callee projection.
No candidate is obtained by consulting the group's other members to supplement
that tau. Distinct contribution entries are not merged because their values
compare equal. Ordinary candidate/path identity handling does not impose
general group idempotence.

A receiver can invoke ordinary compile construction logic from A[t] while
holding its own mutable construction reference. That logic can inject through
the supplied target under its ordinary write and OpenHere checks. The
[associated-state owner](associated-compile-state.md) defines the source-side
write window. Neither this mechanism nor ordinary field forwarding introduces
receiver coercion.

Per-entry Policy views remain independent. A group's availability reflects
the entries exposed by a view; it neither changes member policy nor proves
freshness of an invisible name. Structural child incidence is registered
separately from ordinary group membership.

### 2.1.1 V_τ closure materialization: derived semantics

A newly materialized closure has an anonymous complete function-object type
and its associated () leaf. Its own first self has that exact type. The
anonymous implementation layer remains under /tau. The same structural
formation semantics apply whether the function object is expressed as a
closure or constructed through ordinary anonymous structure.

    MaterializeClosure(C)
      = anonymous complete type A with associated () + callable object of A

An eligible contribution preserves the original function object's owner,
captures and type snapshot. Anchored replication, when needed, constructs a
new instance satisfying TypeOf(F) in the target core; it does not reparent the
original. The destination does not gain ownership of
external dependencies merely by storing the function object. Owned promotion
and escape traversal continue to distinguish owned children, bound references,
and horizontal borrows.

The enclosing-reference and meta identity rules below are unchanged:

**Theorem 3 — Enclosing-reference theorem.**

`V_τ` follows the `P × Val2` model: a `Val2` leaf inhabits a structural
position *below* the `P`/type layer it describes. An upward reference from a
`V_τ` descendant to its enclosing `τ` is therefore the same structural problem
as a `Val2` referring to its enclosing `P` layer — a static descriptive
reference, not an owned edge:

```text
upper P/type layer (τ)
    ↓
anonymous type A_F
    ↓
Val2[()] callable entry
    ↖ static reference to enclosing layer
```

The reference is a `BoundRef(alpha)` / `SymbolicReferenceEdge` (§2.1),
authorized and static; it is not in `Children_owned`, so it does not form an
owned cycle `τ → A_F → () → τ`. The existing invariant
`BackRefsOnlyInStaticPV2Region(τ)` (type-values §2.2, `WellFounded_static`) is
the well-foundedness projection of this theorem — it records the consequence,
not the theorem itself, and needs no separate recursive `V_τ` loop rule.

**Theorem 4 — Meta anchoring theorem.**

In a meta construction context, the upward enclosing reference resolves to the
stable `MetaInstanceRoot` determined at invocation entry, never to a
meta-local `r` or another ephemeral PatternValue:

```text
M = MetaInstanceRoot(MetaInstance(F, args))

HostAnchor(A_F) = M                -- always the stable invocation root

Forbidden:
  HostAnchor(A_F) = r_local        -- even when Value(r_local) = Value(installed result)
```

This is not a new prohibitive rule. It is the inevitable closure of three
existing invariant families:

```text
visibility
+ lifetime / global-survivability
+ capture / classifier-home
----------------------------------------
=> ephemeral return-local PatternValue
cannot become a V_τ enclosing anchor
```

While `int Vec::std` is computing, its body may hold a local construction
result `r`. In ordinary name resolution `r` can be only one of three things: a
capture-list entry, a local definition, or a global symbolic name. It cannot
be context-sensitively remembered as "the `r` from the return position": a
local definition or global lookup finds a *different* `r` (or none), and the
only candidate that could denote the meta-local value is the capture list. But
meta-local PatternValues whose lifetime is governed by the open/construction
window have non-global lifetimes that do not extend by simple copy — whether
interpreted by value or by borrow — so the capture-list path is closed as
well. The returned PatternValue and its **dependency closure** must both
satisfy global survivability; `EscapeDeps` checks this at seal (§4.3.2).

Even when `Value(r) = Value(installed result)`, value equality does not
retroactively imply `Identity(r) = MetaInstanceRoot`. Permitting such
retroactive promotion would reintroduce "future promotion can ratify past
capture" — exactly the model the meta-key / global-stability boundary has
always prohibited.

Closure construction and TypeMember injection are orthogonal operations:

```text
ConstructClosure(f) independent-of Inject(f, r)

Owner(AnonymousTypeOf(f)) = current stable MetaInstanceRoot
Inject(f, r)              = a later, explicit contribution to the open τ
```

An in-place closure therefore acquires its anonymous classifier owner from the
ambient meta environment before any return-local construction handle is
consulted. A source spelling or implementation shortcut may sequence closure
construction immediately before injection, but it must not use `r:type` as the
owner anchor, recover a defining binding from type equality, or merge the two
semantic operations. Nested
an unavailable in-place closure-anchoring consumer cannot recover the eventual
result binding as a substitute owner anchor.

### 2.2 Context projections coexist

A named type and an explicit candidate group support ordinary call projection
through the singleton embedding. A group may be empty or include types with
no callable members. Call/type/navigation/value consumers project members
according to their context. These uses neither classify the group itself nor
collapse binding, entry, value, owner, or Place identity.

### 2.3 Identity separation

The model preserves distinct identities:

```text
NameBindingId
PlaceId
TypeValueId
PatternValue identity
ResolvedPatternScope / PatternScopeId
```

Their roles are:

```text
NameBindingId:
  identity of the resolved binding cell

PlaceId:
  identity of the bindable/openable installation location

TypeValueId:
  stable first-order root projection of Core(tau); implementation/index
  key only, not semantic equality. Ordinary type equality and keying
  observe Core(tau) = Q by default (minimal-change rule, type-values §2).
  Addr(Norm_type(tau)) is the whole-snapshot identity, used to tell
  shared-root snapshots apart in transport and in positions the language
  has independently frozen to whole-snapshot semantics

PatternValue identity:
  canonical identity of an ordinary compile-time value or structured pattern
  value — an ordinary Object / PatternValue. A complete type value `tau` is
  not itself an ordinary PatternValue: Pattern-facing observation goes
  through `Core(tau)`, and whole type identity goes through
  `Addr(Norm_type(tau))` (type-values §2.2)

PatternScopeId:
  identity of a navigable pattern-owner layer
```

No equality implication is automatic between these identities.

### 2.4 Program text names bindings before values

Except for literal syntax and other explicitly specified immediate values,
program text does not directly name a value. A source path first names a
name binding, and value use then reads a facet/value from that name binding:

```text
source path
  -> resolve name binding
  -> read value / PatternValue from that name binding
```

This applies to ordinary values, type values, pattern values, callable values,
and values later used as meta construction material.

Pattern navigation follows the same rule. A normalized pattern navigation name
may happen to render exactly like the source binding path that carries it, but
matching diagnostic text does not merge their identities:

```text
source navigation path names a name binding
PatternValue navigation name is a diagnostic/canonical projection
same spelling does not imply same semantic object
```

For a schematic future character spelling (the frozen lexer does not currently
accept `CharLiteral`):

```lang
let a = 'a';
```

The left `a` is a binding name. The right `'a'` is a character literal. Their
textual content happens to match, but they are not one semantic object.
Pattern values have no comparable standalone literal syntax, which makes a
same-spelled source path and pattern diagnostic projection especially easy to
confuse. The language still resolves the source path as a name binding first.

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

Bare `let` first forms output selection preference `PolicyMode=plain`, before
RHS call selection; that preference participates with input Policy coordinates
in the ordinary product order. After unique RHS selection, omitted P1 retains
and infers the complete RHS pair view, while the selected producer retains its
declared concrete `ResultPolicyMode`. The destination remains independently
plain, and ordinary move/copy transfer between the two slots does not rewrite
the producer mode. See the canonical binding judgment in
`symbol-policy-and-compile-flow-projection.md` §3.1. The destination does not
inherit the RHS mode or make runtime the only way to obtain a runtime binding.

Policy migration does not reinterpret a P1 query as an exact target. Any
non-empty `ProjectP1` result completes the binding and makes
migration unreachable. Only after the complete query projects nothing may an
accepted runtime branch be extracted and paired with an eligible static input
view for one language-authorized atomic migration. The compiler mandates the
static-to-runtime stage edge; candidate-declared endpoint `PolicyMode` belongs to
ordinary overload. Empty queries with no runtime alternative fail, and no
Policy failure searches structure-changing operations. See
`../../contracts/policy-migration.md`.

In an ordinary lexical position, the unannotated form:

```lang
let r = expr;
```

means:

```text
Gamma |- expr ⇓ v
fresh NameBindingId s
fresh PlaceId p
--------------------------------
Gamma |- let r = expr
          where BindingPlace(s) = p and Contents(p) = Some(v)
```

If the right-hand expression is a source path, evaluation expands to:

```text
source path
  -> resolve source name binding
  -> read its value / selected facet
  -> bind that value to the destination name binding/Place
```

For example:

```lang
let a = b;
```

resolves b to one NameBinding, reads its resident value and installs that value
in a's fresh lexical Place.
It does not rename `a` to `b`, make their `NameBindingId`s equal, or merge their
`PlaceId`s.

The bound object is the exact evaluated semantic value, not a new
binding-shaped copy:

```text
resolve(b) = s_b
read(s_b)  = v
bind(a, v)
```

Once `read(s_b)` has produced `v`, `s_b` is no longer part of `v`'s semantic
identity. It may remain in diagnostic provenance only. Ordinary `=` therefore
never requires or creates a `value -> original carrier name binding` inverse map.
No declaration form forwards name binding/place lookup (§2.6); shared observation of
another object is expressed only by a borrow view.

The lexical rule applies to ordinary values, including complete type values.
A nested-path let is instead the structural expression followed by assignment:

```lang
let t1::t = bool;
```

Here and in subsequent abbreviated examples, t denotes an already obtained
authorized mut type ref; a type-valued binding must instead be written
`t |> (type ref)`. Every intermediate parent already exists.

```text
r_t1 := FreshNamedType(t, t1, ordinary declared policy)
  -> commit Contents(Target(r_t1)) = Some(T_0)
  -> return r_t1 : mut type ref
r_t1 = bool
  -> ordinary assignment of the complete resident read through Resolve(bool)
  -> validate the selected assignment's Pre and commit the replacement
```

There is no direct Absent-to-RHS binding transition. If formation fails, no
destination is created; if assignment fails, the formed T_0 remains subject
only to the existing enclosing transaction. Successful assignment does not
reroot the RHS or equate the destination NameBindingId with its Pattern owner.

Likewise:

```lang
let T: type = uint8;
let U: type = T;
```

has:

```text
NameBindingId(T) != NameBindingId(uint8)
NameBindingId(U) != NameBindingId(T)
Place(T)  != Place(uint8)
Place(U)  != Place(T)

TypeValue(uint8) = tau_uint8 = <Q_uint8,V_uint8>
TypeValue(T) = TypeValue(U) = Copy(tau_uint8)
Eval(T) = Eval(U) = tau_uint8
CoreView(tau_uint8) = Q_uint8
PatternView(T) = PatternView(U) = Q_uint8
CallSpace(tau_uint8) = V_uint8
```

The hole-free annotation `type` is the ordinary result-as transformation
applied while evaluating the RHS. It does not select a second “type binding”
judgment or a Boolean compatibility check.

Canonical summary:

```text
Program text normally cannot name values directly. It names a name binding, then
obtains a value through that name binding.

Name navigation is a way to obtain a value, not part of ordinary value
identity.

Pattern navigation paths are likewise name binding navigation first. Even when a
PatternValue's canonical navigation name matches the name binding carrying it, the
matching spelling does not establish identity.

A normalized fully named body of a named Pattern contains
complete-navigation to PatternValue entries, not name-graph bindings. A naked Product
remains positional even when all of its children are named. Extraction
resolves a source name binding, reads its PatternValue, and looks up its canonical
navigation/value entry in the normalized map.

let destination = source
uniformly reads source's value and binds it to destination. It does not reroot
patterns, perform binding aliasing, or merge place identity.
```

Any separate rule that requires a compile-determined projection source to have
non-runtime policy constrains that rule's `Psrc` only. It does not constrain
the P1 binding destination. In particular, an
implementation must not reject a binding merely because
`binding_policy == runtime`.

### 2.6 Lexical aliases and value copying

Ordinary lexical let copies/binds values into fresh destination Places. Value
equality implies neither binding identity nor shared Place authority.
The block-local lexical alias form is defined by [local lexical aliases](entity-alias-design.md):
it maps a spelling to an already resolved binding without creating a value,
Place, group entry, or exported member. Borrow sharing is expressed through
ordinary ref/share; @ reifies name interpretation under the lifecycle rules.

Operator environments remain ordinary values mapping operator identity
(spelling, fixity and arity) to ordinary selection material. Their final
selector algebra is local surface work. No selector result is a manipulable
fresh-name value, and no operator-name exception creates write authority.

## 3. Value Members and Calls

### 3.1 Named contributions and ordinary values

An ordinary lexical `let f = expr` binds its RHS normally. Only an explicit
named-contribution construction position synthesizes the named type's
`V_tau`. Its closure contributions satisfy the target membership judgment
`TypeOf(v') in Core(T)`; [anchored replication](closure-anchored-replication.md)
may produce a new eligible instance without modifying the RHS.

An explicit OverloadGroup aggregates type candidates. Some candidates expose
no callable members in the current context; this is a projection result, not
an invalid group or a new category of name.

### 3.2 Call candidate preparation

A call position performs the following conceptual flow:

```text
resolve name binding
  -> form CallCandidates(NamedType(S))
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
preparing candidates for a call position. Its presence does not make the name binding
invalid and does not turn it into a function overload.

Candidate identity and applicability belong to the candidate/invocation model;
name-first resolution only establishes where the heterogeneous values come
from. Derived compile companions are complete first-class `Val2` function
objects whose existence is derived under the compile transform
(`CompilePartner(F) = C(F)`, function-object-call-model §8), not post-failure
fallback entries; their policy and overload
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

result class:
    ordinary PatternValue
    | complete type value τ
    | type ref / type share borrow instance
    | runtime value
```

This is the current result-class set. Invocation results are driven by each
callable's declared result class — `Result(F)` follows
`DeclaredResultClass(F)` — and consumers must not maintain separate narrow
hand-written enumerations of what `compile` or `meta` can return.

An OverloadGroup is an ordinary algebraic value (§4.7); returning one does
not create a new semantic result universe.

`MetaPartial` / `MetaStrict` describe evaluation demand. They do not define the
meaning of `compile` or `meta`, and they do not determine the successful result
class.

Callable semantics still use ordinary PatternValue result declarations; there
is no private construction result class:

```text
CallableSemantics
    = P1 × P2 × DeclaredResultPattern × Privilege

Privilege   ::= Ordinary | BuiltinPrivileged   -- bounded AST access
```

`compile` may return any declared ordinary semantic value across result
classes (§4.1): an ordinary `PatternValue`, an explicit OverloadGroup, a complete type value `tau`, or a `type ref` / `type share` borrow
instance; a returned `tau` participates in Pattern
observation through `Core(tau)` and is not itself an ordinary
PatternValue/Object.
Ordinary-meta callable kind, call legality, and successful-call effects are
separate judgments inside the ordinary value/policy model:

```text
F in OrdinaryMetaFunction
  => P2(F) = meta
  and DefaultMetaResult(F) = τ

WellFormedMetaCall_Gamma(F, args)
  <=> F in OrdinaryMetaFunction
   and Admissible_Gamma(F, args)
   and forall a in Canonicalize(args): GlobalKeyable_Gamma(a)
   and forall a in Canonicalize(args): MetaArgumentAdmissible(a)

WellFormedMetaCall_Gamma(F, args)
  => K = MetaInstanceKey(F, Canonicalize(args))
   and M = MetaInstanceRoot(ParentSemanticOwner_Gamma(F), K)
   and RootIdentityExists(M)
   and ConstructionNavigationAvailable_Gamma(M)
```

The parent owner is an identity coordinate of the root, not diagnostic
placement metadata:

```text
Identity(M)
  = <ParentSemanticOwner(M),
     SelectedCallableIdentity(M),
     Addr(Product(Canonicalize(args)))>
```

The callable/argument pair may remain a reusable `MetaInstanceKey`, but a root
cache must scope that key by `ParentSemanticOwner`; equal callable and argument
material under distinct stable owners denotes distinct roots.

Root consistency is a positive invariant of meta-root formation:

```text
MetaInstanceRootAlwaysPlain:
  MetaInstanceRoot(M) => PolicyMode(M) = plain

MetaInstanceRoot(M) => StableSemanticOwner(M)
PolicyMode(M) = plain =/> Writable(M)
```

This `plain` coordinate belongs to root identity/formation and is not a
contextual default. Parameter/return position overlays and caller demands may
refine views produced under the root; they cannot change the root itself to
`const` or `mut`.

Equivalently:

    DefaultMetaResult(F) = tau
    MetaInstance(M) -> tau_M

The default does not restrict explicitly declared ordinary result types.
An OverloadGroup result is an ordinary value with its declared result/escape
obligations; its type-valued entries are already complete values. struct forms
its complete tau before an outer binding or structural assignment carries it.
No result is decoded through an optional distinguished type slot.

Callable kind fixes `P2` and `DefaultMetaResult`; `GlobalKeyable` belongs to a
particular call's well-formedness, never to the callable type itself. A
successful call establishes a globally stable root identity and makes it
navigable to the construction, while sealing remains the return-stage effect.
No `compile` callable may establish or seal this root kind.

This exclusivity does not claim that every stable owner/root in the language is
a `MetaInstanceRoot`. Lexical declarations and privileged built-ins may
establish, select, or preserve other root kinds only through their separately
specified owner rules (§4.8). They cannot use those rules to manufacture an
ordinary navigable `M`.

This is not a new result class. The default meta result is the complete type
value `τ` itself, which is not an ordinary `PatternValue`; an explicitly
declared group result returns an ordinary OverloadGroup. Root authority governs the
open-window state and global lifetime of the default result's
`OwnedResultClosure(τ_M)` — `OwnedClosure(Core(τ_M))` plus
`OwnedCallSpaceClosure(CallSpace(τ_M))`, where `Core(τ_M)` is the first
projection of the default result and hence always present. An
implementation may retain a carrier to accumulate those members,
but may not expose that carrier as a callable result ontology.

Private execution material may transport construction effects before the
ordinary semantic result is formed; it is not an additional result category. The following roles remain distinct:

```text
Explicit group value           — ordinary OverloadGroup (§4.7)
Name binding                   — structural identity and resident Place relation
Meta return construction role  — the members a meta body accumulates before seal
Namespace same-name synthesis  — merging same-named contributions in a namespace
World installation role        — what a sealed root becomes in the global graph
```

A rule stated for one role does not transfer to another.

### 4.2 `compile`

`compile` is value-level staging. It performs compile-time computation without
creating a symbol-construction root:

```text
compile:
  input / output  any declared ordinary semantic value across result classes
                  (subject to root conservation, §4.2.1)
```

`compile` may pass and return:

- ordinary compile-time values (ordinary PatternValues);
- complete type values `tau` — they participate in Pattern observation through
  `Core(tau)` and are not ordinary PatternValues/Objects;
- explicit OverloadGroups of complete type candidates;
- `type ref` and `type share` views;
- structured pattern values.

All of these may be passed to and returned from a `compile` callable. A computed
type value is still a value: it is not thereby an installed type binding, a
namespace node, or an extendable place.

#### 4.2.1 Root conservation

The positive restriction on `compile` is that it conserves roots:

```text
Roots(Output)
  ⊆ Roots(Arguments)
  ∪ Roots(GlobalConstants)
  ∪ LexicallyDeclaredStableRoots
```

Every root reachable from a `compile` result must already have been rooted
somewhere the caller can name: in an argument, in a global constant, or in a
lexically declared stable declaration. Consequently `compile`:

```text
registers no global name binding
produces no nominal type that lacks a normal global root
never promotes a local temporary pattern value into a global type
```

This is a conservation law, not a shape restriction. Returning an ordinary type/group value or a `type ref` whose root already
exists is legal; manufacturing a rootless
nominal type is not. `compile` is therefore not a rootless meta-type generator,
and "compile may return a type" and "compile may not invent a type root" are
both true.

Returning a `type ref` from `compile` is **not** prohibited:

```lang
let identity = (self, r: type ref): compile -> out: type ref => {
    r;
};
```

The returned view is subject to the ordinary lifetime/capability condition of
[`type-values-places-and-borrow-views.md`](type-values-places-and-borrow-views.md)
§5.5, evaluated at the receiving position. Its validity is independent of
whether the then-current pointee is Open. A return is rejected only when the
ordinary borrow escape check fails; a valid returned ref may later be unable to
`inject` because `OpenHere_Σ(Read(ref))` is false. Escape checking belongs to
[`../lifetime/lifetime-policy-and-overload-boundary.md`](../lifetime/lifetime-policy-and-overload-boundary.md)
§3. The body may weaken before returning when write capability is unnecessary:

```lang
r share;
```

#### 4.2.2 Compile is construction-transparent and root-non-generative

A `compile` evaluation reads two independent contexts:

```text
EvalCompile(F, args; ConstructionContext_caller)

DefinitionLexicalContext(F)
  — local Self space, anonymous closure type ownership,
    lexically declared identity

CallerConstructionContext
  — the current evaluation stack used with each value's `Anchor` and
    current window state
```

The definition context decides names and lexical owners. The caller context is
used only by operations that query `OpenHere_Σ(v)`: they combine the value's
`Anchor` with the current window state and an authority-frame resolution over the
caller's stack (§12.1.1). Neither context substitutes for the other.

Passing through a `compile` call, cloning, selecting, or composing a value
preserves its canonical value and `Anchor`/`GenerationRegime` while discarding source
place identity. A compile frame is transparent to the Open-authority stack walk, so an
OpenHere value remains OpenHere through any number of compile/transparent-intrinsic
frames unless another semantic boundary closes its construction interval:

```text
Anchor(Clone(Read(q))) = Anchor(Read(q))
OpenHere_{Σ + compile-frame}(v) = OpenHere_Σ(v)
```

The formal-parameter case is ordinary value transport:

```lang
let extend =
    (self, t: type): compile -> out: type => {
        (t, ...) |> extend;
    };
```

The call is applicable only when the transported value is open in the caller's
stack:

```text
Requires(extend) = OpenHere_Σ(t)
  -- OpenHere_Σ combines the live window state with the authority-frame
  -- judgment of §12.1.1 (non-meta: AuthorityFrame_Σ(t) exists;
  --          meta: Anchor = CurrentEvaluationCoordinate_meta)
```

A `type ref` parameter proves no such fact. A body that performs place-level
`inject` must read the pointee and check both independent premises:

```lang
let extend_ref =
    (self, t: type ref): compile -> out: type => {
        (t, ...) |> inject;
        t clone;
    };
```

```text
Requires(extend_ref) = OpenHere_Σ(Read(t)) ∧ Writable_Γ(Target(t))
```

Hence compile context sensitivity is construction-authority sensitivity, never
a hidden capability on `type ref`:

```text
a compile evaluation depends on the caller's Open window
  exactly for operations that query OpenHere_Σ on a transported PatternValue
```

not as a general property of every `compile` call, and not decided by whether a
`type` value happens to be a formal parameter. Caches and `Requires` summaries
track `Anchor` and the open-window state separately from canonical value
identity and recheck applicability in the caller stack.

`compile` does **not** create a `MetaInstanceScope`, does not introduce a
meta-style virtual symbolic-navigation layer for name shadowing, and does not
impose a self-root requirement on a returned type value. It may freely return an
already existing value:

```lang
let identity = (self, t: type): compile -> r: type => {
    let r = t;
    r;
};
```

The opposite boundary: `compile` has no responsibility to establish a new
globally stable `MetaInstance` anchor, so it may transport local or open
PatternValues as ordinary values:

```text
compile computation   may transport open/local PatternValues
meta invocation       requires globally survivable inputs (§4.3.3)

transport of an open PatternValue
  ≠
evaluation reentry of that PatternValue
```

Transporting an open PatternValue through `compile` is subject to
`NoOpenEvaluationReentry` (`OpenEvalReentry_κ`, type-values §2.1.1):
the value may be passed, but no active evaluation edge may be re-entered into
it. This is the complement of §4.3.3's argument boundary.

When a `compile` body uses a local `struct`, ordinary function-object scope
rules apply. Its ambient lexical/Pattern owner is the current
`CallableOwner` and that owner's callable-local `Self` space. This statement
does not determine the invocation receiver type. Standalone function-object
materialization defaults to an anonymous callable type derived from the owner;
an associated `()` implementation may instead bind invocation slot 0 and the
receiver-type projection of its local `Self` to a named receiver such as `T ref`.

Nested paths print in source order, current/innermost callable-local `Self`
first and outermost `Self` last, but identity is the parent-linked owner graph.
No `__inner_space` or `__inner_namespace` node participates in canonical
ownership. This owner is not a meta-instance owner such as
`MetaInstanceOwner(meta_function, canonical_arguments)`.

### 4.3 Ordinary `meta`

`meta` is construction-stage evaluation. An ordinary meta invocation is the only
construction that establishes a new navigable `MetaInstanceRoot`, and by §4.1
every ordinary meta invocation does so:

```text
WellFormedMetaCall_Gamma(F, args)
  => M = MetaInstance(F, Canonicalize(args))
   and RootIdentityExists(M)
   and ConstructionNavigationAvailable_Gamma(M)

RootIdentityExists(M) != ExternallyInstalled(M)
ConstructionNavigationAvailable_Gamma(M) != ExternallyInstalled(M)
```

Entering the invocation creates `M` as a **globally identified but unsealed
root** available to its construction. This does not publish a partially built
namespace delta. `ExternallyInstalled(M)` becomes true only after the returned
result crosses an ordinary outer binding/namespace-installation boundary and
that delta commits atomically (§12.4). The returned value is the default result
`τ_M` of `M`:

```text
meta:
  accepted parameters
  -> the default result τ_M of M
```

A meta callable may accept an OverloadGroup parameter, or constrain a parameter to a
narrower `type` or ordinary PatternValue. That does not introduce another result
class: successful ordinary meta invocation still defaults to `τ`. `M` exists in
the global world from body entry; the return stage runs the default-branch seal
`Seal(DefaultTau(τ_M))` of §4.3.2 —
well-formedness of `τ_M`, promotion of `OwnedResultClosure(τ_M)`, escape check —
and seals the result.

Failure never publishes construction material:

```text
FailedMetaCall(M) => not ExternallyObservablePartialInstallation(M)
```

Whether an implementation retains the failed canonical root identity for cache
or diagnostics is non-semantic. No partial namespace delta becomes externally
visible.

Meta functions are divided into two privilege classes:

```text
MetaFunction
  |- OrdinaryMetaFunction
  `- BuiltinPrivilegedAstMetaFunction
```

#### 4.3.1 The body is fully transparent to construction

Everything an ordinary meta body does to its own construction material is
permitted, and none of it closes the construction. The following are all legal
inside a meta body and none of them ends the open state of the values being
built:

```text
generating local pattern values
generating the same struct shape repeatedly
locally modifying material the body itself produced
using a value for Val1
passing material through static control flow
calling compile callables
entering an in-place closure that the body itself writes
referring recursively to M
```

This is the meta-closure transparency rule. The construction anchor of an
in-place closure written inside `M` is `M` itself:

```text
ConstructionAnchor( in-place closure inside M ) = M
```

so material owned by `M` remains open across that closure boundary. Anchor
transparency is not identity erasure: the closure still has its own anonymous
callable type identity,

```text
ClosureType = M::Site
```

and that identity keeps its own owner and lexical `Self` space. Transparency
concerns *who owns the construction*, not *which type the closure is*.

Construction transparency is not lifetime promotion. A fresh PatternValue
created inside an ordinary meta invocation has the invocation-local lifetime:

```text
Life(LocalPatternValue(M)) = MetaInvocation(M)
```

It may be copied through local binders, static control, `compile` calls, and
transparent construction intrinsics without freezing. Those operations do not
form a new global key. It may not, however, become a dependency of another
ordinary `MetaInstance` unless it has independently become `GlobalKeyable`.
Thus:

```text
No freezing inside M
  !=
arbitrary meta-local PatternValues implicitly become global
```

An anonymous closure type such as `M::Site` is globally stable only when every
PatternValue dependency in its signature is global-keyable. A signature may not
capture the identity of an ephemeral local PatternValue merely because the
closure type itself has a stable site name.

#### 4.3.2 Seal happens only at the return stage

The construction-ending disposition of a meta invocation is its final return
stage. The default complete pattern result satisfies:

    Seal(DefaultTau(tau_M)):
        WellFormedTau(tau_M)
        Q := Core(tau_M)
        Pure(Q)
        Root(Q) = M
        promote OwnedResultClosure(tau_M) into M (call it P)
        EscapeDeps(tau_M) subset AlreadyGlobalStable union P
        seal M

An explicitly declared ordinary result follows its result-type-specific seal
and escape obligations. Group membership neither creates a promotion root nor
grants promotion to an external member. Traversal observes every entry,
including the whole callspace of any complete pattern value carried there.
Empty groups require no special optional-core seal branch. The root and owned
closure rules below remain authoritative for every complete tau result.

For the default branch, `Core` is a total projection on complete type
values, so `τ_M` always has a defined core projection:

```text
τ_M = ⟨Q, V_τ⟩
--------------------------------
Core(τ_M) = Q
```

This is a pair projection (an elimination rule), not a cardinality count:
there is no "core collection" to size, no `τ`-absent case to guard, and no
optional installed-core slot. `Q` is the first projection of `τ_M`. The
self-root rule is unconditional there: `Root(Core(τ_M)) = M` holds for every
well-formed default result. A namespace-only core — `NamespaceRole(Core(τ_M))`
and `not HasRegisteredSelfConstruction(Core(τ_M))` — is
therefore a valid promotion anchor even when `TypeRole(Core(τ_M))` is false;
type-role requirements are refinements, not generic result constraints.

For an ordinary group result, borrowed or external entries remain dependency
edges under the same ownership/escape distinction. A returned reference cannot
promote its target merely by being stored in a group.

`EscapeDeps(τ)` traverses the whole returned result at the τ level:
`Core(τ) union CallSpace(τ)` plus every horizontal `ref` / `share` / `rebind`
dependency target. At the Object level this still runs through
`Children_Val1 union Children_Val2`, including nested products, Sequences,
callables, and navigable `Val2` structures; the τ-level entry is what makes
`V_τ` — its closures, their anonymous types, and their captures — part of the
escape check rather than an implementation guess. Thus no returned branch can
smuggle unrelated meta-local material out of the invocation, and no `V_τ`
member can escape the closure check by being reachable only through the
callspace.

Promotion is likewise defined at the τ level:

```text
OwnedResultClosure(τ)
    = OwnedClosure(Core(τ))
      union OwnedCallSpaceClosure(CallSpace(τ))

OwnedCallSpaceClosure(CallSpace(τ))
    = least closure of the CallSpace(τ) members — including the V_τ closure
      anonymous types A_F and their () leaves, per the §2.1 V_τ member
      closure-ownership theorem — under the owned navigation relation of τ
```

Horizontal borrow edges are not ownership and are never dragged into either
component:

```text
OwnedClosure(x) excludes every ref / share / rebind edge reachable from x
```

Edge classification is explicit:

```text
BoundRef / stable enclosing-root reference
    = dependency / backreference, not an owned promotion edge

ref / share / rebind target
    = escape dependency, not an owned promotion edge

external stable dependency
    = dependency leaf, not recursively promoted
```

For this promotion, “owned closure” is not arbitrary graph reachability. Let
`OwnedNavigation_Q(x, y)` hold only when `y` is a genuine direct child owned by
`x` in Q's construction tree; the callspace component uses the isomorphic
relation over `CallSpace(τ)`. Then `OwnedClosure(Q)` is the least closure under
that relation, subject to all of these invariants, applied component-wise:

```text
direct child only:       every step is parent -> direct child
no jump:                 a parent cannot inherit a deeper descendant directly
bare termination:        Bare(x) stops expansion for the component
external termination:    ExternalTo(component, x) is an opaque dependency leaf
no external re-entry:    expansion never leaves the component, enters an
                         external subtree, and later re-enters owned material
no cycle:                 x not-in OwnedNavigation_component+(x)

OwnedNavigation_Q(x, y) => DirectOwnedChild(x, y)
Bare(x) | ExternalTo(Q, x) => no y: OwnedNavigation_Q(x, y)
ExternalTo(Q, q_i) => no j > i: Owner(q_j) = Owner(Q)
```

Borrow edges remain excluded from both components of `OwnedResultClosure(τ)`
and are never promoted merely because they are referenced.

External leaves may retain their own independently owned trees, but those trees
are not promoted through `τ`; their dependencies must already be globally
stable. The ordinary recursive Object normal form still traverses
`Children_Val1 union Children_Val2`; this construction judgment only determines
which fresh-owned part may acquire M's global lifetime.

A member reachable only through a borrow view is therefore not promoted, and its
presence does not extend `M`'s owned material. Its target must already satisfy
the escape condition. After the seal step, `M` is sealed and nothing may reopen
it.

#### 4.3.3 `M` as a navigable layer

Every ordinary canonical meta-function invocation establishes a virtual
symbolic-navigation and construction-authority scope:

```text
M = MetaInstanceScope(callee_symbol, canonical_arguments)
```

`M` is the `MetaInstanceRoot` of §2.1 — the symbolic-navigation, stable-identity,
and construction-authority anchor of the invocation. It is **not** itself the
result value: the default result is `τ_M` with `Root(τ_M) = M`; an explicitly
explicit group result is an ordinary OverloadGroup.
A `NameBinding` or installation is a separate outer-graph binding/assembly
operation and does not constitute the result ontology.

Formation additionally requires:

```text
for every canonical argument a:
  GlobalKeyable(a) ∧ MetaArgumentAdmissible(a)

OwnedDependency(a) != GlobalKeyDependency(a)

Borrow(q) in a
  => Target(q) in GlobalKeyDependency(a)

GlobalKeyable_Γ(a)
  <=> every d in GlobalKeyDependency(a) is, at key-creation time,
        AlreadyGlobalStable_Γ(d)
      | AlreadyPromoted_Γ(d)
```

A meta invocation is a new stable MetaInstance construction boundary, so its
arguments must carry no PatternValue dependency that cannot survive globally:

```text
MetaArgumentAdmissible(a)
  => GlobalSurvivable(a)

GlobalSurvivable(a)
  <=> every dependency d reachable from a is globally survivable:
       direct PatternValue dependency
     | PatternValue held inside a carried type (τ)
     | dependency reachable through a type ref / type share target
     | nested dependency in Val1 / Val2
     | other escaping semantic dependency

GlobalSurvivable(a) ≠ GloballyVisible(a)
```

A value may survive globally without being name-visible everywhere, and a
PatternValue visible in the current lexical scope whose lifetime ends with the
current meta invocation is **not** admissible as an argument of a deeper meta
invocation.

A binder local to a meta invocation is not rejected merely for being local: if
it holds a canonical value whose dependencies are already global-keyable, that
value may enter the key. What is rejected is a fresh ephemeral PatternValue
dependency or a borrow of a meta-local place entering a new `MetaInstance` key.
A closure that might be promoted only when an enclosing meta invocation later
seals is not `AlreadyPromoted` for an inner key created now. `compile` and transparent
construction intrinsics impose no such boundary because they establish no
`MetaInstance` key and no new root.

For:

```lang
let f = (self, t: type): meta -> r: OverloadGroup => { ... };
```

the diagnostic navigation projection of `M` is:

```text
(t f)
```

This is not merely a folder analogy. `M` is a symbolic-navigation layer that
participates in default pattern navigation and name shadowing; the stored
complete type closure and typed value members belong to `τ_M`'s `Core(τ_M)` and
`V_τ` (not to `M` as a name binding). `M` anchors cache/incremental identity and owns
the return construction transaction.

The default result is `τ_M` rooted at `M`; an explicit group result is
an ordinary OverloadGroup. The declared
return slot is a lexical name for the result value, not a transferable
construction class:

```text
ResultValue = τ_M,  Root(τ_M) = M        (default)
ResultValue = G : OverloadGroup          (explicit group)
return_slot(r) = NameBinding of τ_M / G (lexical name, not a result class)
```

The slot name `r` does not add another component to the final navigation path.
Material written through `r` contributes role/value members or children to
`τ_M` rooted at `M`; it does not
create `r::M` or place an extra binding named `r` beneath `M`. For example, a
structural expression `let t1::(r |> (type ref)) = bool;`, given the already
formed result resident r, first commits T_0 at the M-rooted child t1, then
ordinarily assigns the complete bool type. It creates no extra r path segment
and registers no Pattern-child edge; structural registration remains extend/inject.

Canonical argument identity follows parameter rank:

```text
group parameter  -> ordinary group value identity, preserving entry multiplicity
type parameter   -> default Core(tau) = Q observation; `TypeValueId` is only
                    the implementation/index projection, not semantic equality;
                    whole-snapshot Addr(Norm_type(tau)) identity applies only
                    where the language has independently frozen it
value parameter  -> PatternValue identity
```

Reference arguments retain their ordinary reference identity and dependence.
A group's ordinary value key does not acquire the installation Place of the
name through which it was read.

### 4.4 Ordinary meta return self-root invariant

If the declared complete-pattern result of an ordinary canonical meta invocation
is `τ`, its core `Core(τ)` — the structural
material that anchors the returned role root — must have its outermost
pattern root at the invocation's own `M`:

```text
ResultValue = τ
  => Pure(Core(τ))
   and root_pattern_scope(Core(τ)) = M
```

This is identity equality between a pattern root and the meta-instance binding
scope. It is not equality of rendered strings. The root identity is:

```text
MetaRoleRoot = MetaFunctionIdentity
             + Normalize(Arguments where every argument is GlobalKeyable)
```

Nodes beneath the root compare by normalized value: same root and same
normalized value imply the same pattern node. Source spelling, source binding
names, and provenance do not participate in node equality.

Consequently, both of these meta bodies are invalid:

```lang
let f = (self, t: type): meta -> r: type => {
    let r = t;
    r;
};

let fn = (self, t: type): meta -> r: type => {
    let r = uint8;
    r;
};
```

The right sides are valid external type values, but their `PatternValue` roots
belong to external scopes. Reading the values through the bindings `t` or `uint8` does not make that external root identical to `(t f)` or `(t fn)`.
Neither value may directly replace the returned result's required role root.
The failure is the hard diagnostic `MetaReturnRoleRootMismatch`. An
implementation must not silently repair the mismatch by wrapping the external
value in a synthetic self-rooted node; check failure is failure.

A legal meta construction builds under its own scope:

```lang
let f = (self, t: type): meta -> r: type => {
    let r = (t inner) |> struct;
    r;
};
```

Its complete pattern is:

```text
(t inner::(t f))::(t f)
```

External `PatternValue`s may be members of the self-rooted core; they may not
replace the root. For example:

```lang
// In a meta body with its self-rooted complete result already bound as r:
let t1::(r |> (type ref)) = bool;
r;
```

first forms the fresh child T_0 and then assigns the complete bool type. After
success, it keeps the existing MetaInstance as the result's root and includes the externally owned
`bool::` value as a member beneath that root. It must not be summarized as
`NamespaceCoreProjection(r) = bool::`.

The default complete-type result always has Core(τ) = Q and must satisfy the
self-root check, independently of TypeRole(Q). A namespace-only `Q` — `NamespaceRole(Q)` and
`not HasRegisteredSelfConstruction(Q)` — is self-rooted and may own fresh
invocation-local material. An explicit group result instead follows its ordinary
member/escape rules (§4.3.2); it is not an optional-core type result. When
`TypeRole(Q)` does hold, it is the additional type
refinement (imported judgment); namespace-only `Q` is not required to define Val1.

### 4.5 Formal return material

Canonical semantics do not give the spelling of a return slot a special creation
meaning. A meta body computes its result value (`τ` by default); `let` creates its local
members, `=` writes existing places, and the return event transfers that value.
The explicit return-slot spelling `r` denotes the declared return position; it
does not create a construction-value ontology.

Formal meta return material is a family of distinct construction-effect forms,
not one spelling-insensitive binding. Create, write, and deliver are distinct
events that never collapse:

```text
    let x = expr;     -> creates a fresh name binding/member according to the
                         declaration context
    target = expr;    -> Write(existing target, expr)
    return event      -> control transfer only
```

- `target = expr;` writes to an existing target; a write is not append, and a
  construction model that only supports appending cannot express
  `let x = first; x = second; return x` by treating both operations as
  contributions.
- A return event delivers its value to the selected enclosing layer. It is not
  a member contribution and does not give the return-slot spelling special
  binding semantics.

Source wiring for expression-level write and general construction effects is
pending. An unavailable source operation does not acquire a spelling-directed
substitute.

The terminal family follows the general control-flow end model: `expr;`
delivers to the directly enclosing layer, `expr return;` returns to the
outermost function layer, and `expr (T return);` returns to the layer selected
by the function-object type `T`.

Add-fresh-member and write-to-existing-target are two distinct construction
effects. They must not be collapsed into one injection event, and neither is a
return. Whether contributed material references an existing `PatternValue`,
computes new material, or projects a name binding member is represented inside the
construction value; any resulting type core `Core(τ)` must pass the self-root invariant in
§4.4.

There is no fourth "alias member" event. A member is created by `let`, written by
`=`, and nothing forwards an external binding's `Val2` material into a member.
Where shared observation of an external object is wanted, the member holds a
borrow view (`ref` / `share`), which is an ordinary value and is subject to the
ordinary member rules — including the rule that a borrow edge is not owned
material and is therefore not promoted at seal (§4.3.2).

#### 4.5.1 `let` creates, `=` writes, the return event transfers control

The three rules are orthogonal:

```text
let   — only creates a new name binding/member (never writes existing targets)
=     — only writes to an already existing target
return event — produces control return, independent of whether a return
        value was written
```

Consequently:

- `let r` may shadow the return value, because `let` creates and `=` writes;
- explicit return uses the return event mechanism — return depends on the
  event, not on whether a return value binding was written;
- even if an explicit return value exists and has not been shadowed, return
  still requires a return event to produce control flow;
- writing to an explicit return value after which control does not return is
  analogous to dead code — not erroneous, because intermediate computation may
  have side effects.

Assignment is itself an associated operation. The source spelling `=` selects an
ordinary assignment candidate; only the selected candidate's default
implementation performs the universal write judgment below. There is no
compiler primitive `Write` behind the source spelling, and no assignment
candidate exists merely because a checker could prove the place writable —
write capability is exposed by the selected associated callable, not invented
by `mut` policy.

The default `=` entrance forwards through the operator/ADL path, not through
special compiler logic that inspects the LHS and searches for an assignment
family:

```text
operator[=]   -> .=
.=            ≡ =::adl
```

Required source behavior:

```text
object : T        object ref = value   -- form ref, then .=
object : T ref    object = value        -- direct .= on the ref's target
```

`NoImplicitBorrowFormation` remains absolute: an ordinary `T`-valued LHS
never secretly forms `Ref(CarrierPlace(lhs))` (`AssignmentReceiverFromPlace`
is forbidden). When the receiver is already `T ref`, assignment writes
`Target(receiver)`, not the place storing the ref handle. Custom Val2 may
define setter candidates through `.=`; setter participation does not make
anything a P structural field.

Structural let first supplies its ordinary type-reference assignment problem;
it does not itself select a type-contribution operation. The universal family
below defines same-Type replacement. It neither proves nor forbids an additional
ordinary assignment candidate whose realization uses the existing type/replication
algebra. Such a candidate needs its own ordinary declaration, selection and
legality derivation; no let-specific initialization privilege supplies one.
Whether that realization is admitted is an assignment-family question, not a
rule owned by the closure witness.

The universal `=` family for `T ref` is:

```text
AssignmentFamily(T):

  =
  (self,
   mut let object : T ref,
   other : T)
  -> unit
  => default

  =
  (self,
   const let object : T ref,
   other : T)
  => delete

  =
  (self,
   let object : T ref,
   other : T)
  => delete
```

Only the selected `default` performs the universal write judgment below. The
three layers are thereby fully separated:

```text
policy
    controls which = candidate wins

selected = candidate
    exposes the write operation

Write default
    validates the actual place
```

`T share` provides no `=` family at all: `share-value = other` yields **no
applicable overload** in the candidate domain, never a selected assignment that
then fails `Writable`. `AssignmentFamily` here is the universal `T ref × T`
family. Field-specific write candidates (`FieldWriteFamily(T, name, A) ⊆
Candidates(=::adl)`) are a distinct ordinary associated family for every `A`
— including `A = T`: shape coincidence (both `T ref × T -> unit`) is never
family identity, because the field family's target operation is
`field(receiver, name)` while the universal family's is `Target(receiver)`;
selector entry and family identity are normative in
`type-associated-function-objects-and-access-trees.md`. Assignment carries no `extend`-specific validation, but
that is not the same as carrying no validation. A pure `extend` in the right
side already discharged `Open ∧ ParentToChild ∧ NoPatternConflict`. The
place-level `inject` wrapper performs that check before its own write.
Everything else that applies to any write still applies. After the assignment
candidate is selected, the write `lhs = rhs` is checked in four independent
layers:

```text
1. RHS operation legality
     Evaluate(rhs) ⇓ v
     -- an extend inside rhs checks its own Open here, not at the write

2. universal write applicability
     Writable(lhs)
     Compatible( P(lhs), v )
     ValidCapability(lhs)
     Contents(lhs) = Some(old)
     -- a type share is not a write target; bare = never creates None

3. result-object invariants
     WellFounded_kappa(v)
     Canonicalizable(v)
     NoForbiddenCycle(v)
     -- a write forming a non-normalizable Val2 cycle fails, even when it comes
        from an ordinary assignment

4. semantic-boundary constraints of the enclosing region
     meta return self-root; ref / pattern-value lifetimes;
     mutability limits on global type-bearing values; seal / global-promotion
     rules; ordinary group entry and result-type obligations
     -- these may run at write time, normalization time, return time, or
        install time, but they all remain in force
```

Assignment RHS semantics are explicitly value semantics:

```text
AssignmentRHSIsValueSemantic:

object : T ref
other  : T
```

`other` is a genuine `T` value. There is no implicit dereference
(`T ref -> T`), no implicit clone (`T share -> T`), and no reading of a borrow
handle's referent bytes as if they were the value (`T ref -> T ref` memcpy).
In assignment candidate adaptation:

```text
T ref/share =/=> T
```

must hold. If `T ref |> cloneable == true` or `T share |> cloneable == true`,
the legal path is an explicit/independently selected clone producing a `T`
value, then `=`; assignment never secretly clones or dereferences. This is
orthogonal to `NoImplicitBorrowFormation`:

```text
NoImplicitBorrowFormation     forbids T -> T ref/share
AssignmentRHSIsValueSemantic  forbids T ref/share -> T
```

both directions are closed.

The two validation families are therefore distinct and must not be conflated:

```text
ExtendSpecificValidation             -- discharged during RHS evaluation, once
UniversalObjectAndBoundaryValidation -- always applies to the write result
```

Assignment does not inspect how the right value was produced: it asks for no
provenance, no construction witness, and no transition proof from any particular
producer, and a value that conforms to the target Pattern is acceptable
regardless of which operation built it. That freedom covers layer 1 only. It
does **not** exempt the result from layers 2–4 — the write result must still
satisfy every ordinary type, capability, lifetime, normal-form, and boundary
invariant.

This distinction does not cancel `let f::(t |> (type ref)) = expr` at an
explicit construction target: the structural let expression creates f and
returns its mutable reference before ordinary assignment, and does not change the `r;` terminal semantics.

A successful construction returns the semantic entity declared by the selected
callable's result class. A returned value does not itself create a name binding; an outer binding
establishes its own destination identity and Place. Construction effects and replay provenance are
execution material, not a second value ontology.

Value equality remains independent of source name and navigation path and does
not merge binding or place identity. However, that general identity separation
does not waive the meta return self-root invariant (§4.4): `r = uint8` as a direct meta
return core installation is rejected after binding resolution/value read, rather than
being reinterpreted as forwarding or accepted as an identity meta type.

### 4.7 OverloadGroups are ordinary algebraic values

The [name owner](names-and-overload-groups.md) defines group membership and
normalization requirements. An OverloadGroup may be copied, bound, passed,
returned or updated through ordinary value/reference operations. Name
occupancy is outside that algebra; epsilon_OG is an existing empty value.

There is no distinguished optional type component. Group algebra aggregates
type candidates under the specified bucket relation; its current coarse key is
Core(T). Bucket combination is distinct from arbitrary value interning and
does not mutate a candidate type. Candidates may have heterogeneous structure
and may contribute nothing to the current call projection.

Type selection returns a selected complete tau, including its immutable
callspace. It does not decode a hidden type slot, construct a type by counting
members, or use the other entries as a substitute callspace. Internal indices
and replay buffers remain representation only.

### 4.8 Built-in privileged AST meta functions

A compiler-defined privileged family uses the general function-object and meta
invocation framework without becoming user-definable macro capability:

```text
BuiltinPrivilegedAstMetaFunction {
    compiler_known_identity,
    accepted_normalized_ast_or_pattern_rank,
    required_ambient_construction_capability,
    declared_result_pattern,
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
establish no ordinary MetaInstance root;
return ordinary PatternValues or complete type values rather than a construction class;
declare explicitly whether they are pure or write an existing place.
```

Privilege buys a bounded AST carrier and a special scope/owner rule — it buys no
result ontology. There is no shared "construction handle" return family and no
third result class (§4.1):

```text
extend  : type × StructLikeMaterial -> type
inject  : type ref × StructLikeMaterial -> type ref
struct  : StructLikePattern -> tau
*       : type × (CompileNatural | omega) -> type
```

Unlike an `OrdinaryMetaFunction`, an individual built-in defines a
member-specific scope/owner rule and does not create an independently navigable
`MetaInstanceScope M`. Users may call compiler-provided members but cannot
define new privileged AST meta functions. Privilege is member-specific: one
built-in's accepted carrier and bounded transformation do not imply a general
macro system or arbitrary AST rewriting.

The ordinary-meta root-establishment rule in §4.1 governs only a navigable
`MetaInstanceRoot`; it does not claim authority over every stable owner/root in
the language. Built-in root behavior is therefore split by member rather than
inferred from the privilege class:

```text
ordinary meta:
  require GlobalKeyable(Norm(args))
  establish NavigableMetaInstanceRoot(MetaInstance(F, Norm(args)))

struct:
  establish or select StructLexicalRoot(input_navigation, ambient_scope)
  according to §7.2; establish no navigable M

extend:
  establish no root
  Root(output) = Root(input)

inject:
  establish no root
  read the target, call extend, and write the result to that same target

*:
  establish no navigable MetaInstance root
  derive T*N or T*omega from the normalized element type and shape argument
  preserve rank(T)

other privileged built-in:
  must declare its own special_owner_rule and special_scope_rule
  before it can produce rooted material
```

A special owner rule cannot be used as an alternate route to an ordinary
navigable `M`. Liveness, visibility, borrowability, and installation of a
built-in result follow the particular member rule and ordinary outer binding;
the privilege class supplies no generic conclusion that every result is rooted
under the call-site `Self` chain, has global root identity, or is externally installed.

`struct`, `extend`, and `inject` are the first specified members. Future candidates may
include explicit sum construction/extension, bounded AST injection, or a
facet-construction primitive, but each must receive its own capability boundary.

## 5. Physical source normalization and semantic construction

PhysicalTree(Level) normalizes into a meta program. Each file contains serial
meta actions; sibling file and directory blocks start from the same input
snapshot, produce overlays, and join by ordinary unordered effect composition.
A serial implementation cannot expose an earlier sibling's new writes to a
later sibling merely because of filename order.

Source locations are provenance for discovery, decoding, diagnostics and
caching. Actual source meta actions create names and Objects under their
existing capabilities. Files and directories provide no additional ownership,
reopening permission, or prohibition on same-name entry aggregation.

The [composition owner](symbol-construction-units-and-namespace-origin.md) and
[build normalization](../build-package/build-system-design.md) define this
boundary. Transactional storage may realize a semantic transaction; it cannot
invent one for structural let assignment or impose a file-owned transaction.

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
let f = (self, t: OverloadGroup): meta -> r: OverloadGroup {
    let r = (t first, t second) |> struct;
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
let f = (self, t: OverloadGroup): meta -> r: OverloadGroup {
    let r = (t first) |> struct;
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
one navigable binding atom. If `Vec` is found under `std` and the argument is
`int`, the canonical form is:

```text
(int Vec::std)
```

Resolution proceeds as:

```text
resolve callee path Vec::std
  -> resolve argument int
  -> form canonical meta invocation
  -> treat the complete invocation as one navigable binding atom
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

This semantic/navigation rule is not part of the current lexer, parser, Raw
AST, or Normalized AST surface.

## 7. `struct`

### 7.1 Public boundary

`struct` is a `BuiltinPrivilegedAstMetaFunction`, not an ordinary user-definable
meta function. It uses the general function-object/meta call framework but does
not create its own ordinary externally navigable `MetaInstanceScope M`.

The public semantic boundary is:

```text
struct:
  StructLikePattern
  -> tau
```

An implementation may carry AST or Normalized AST as a private structured
carrier. The public result is the complete type value tau under the ordinary
invocation boundary (§4.1, §4.7–§4.8). NameBinding is not a value result class. The formation event is:

```text
struct(P)
  = tau_P
  = bind alpha.<Q_P[alpha], V_τ[alpha]>
```

where the core `Q_struct = Core(tau_struct)` is produced
during the formation event, satisfying `TypeRole(Q_struct)`, and the
direct TypeMembers generated during that formation event enter `V_τ`
immediately; there is no intermediate name binding from which `Q_struct` or `V_τ`
is later projected. Section 7.5 closes the mechanically generated
field/access/ref/share/assignment partners in that complete type snapshot and
exposes corresponding associated views. Other authorized ordinary members, when
present, are likewise part of that snapshot's `V_τ`; type-as-callee never
recovers a defining name binding. This bounded capability does not expose a general
macro system.

In the complete-type notation this producer-specific guarantee is:

```text
Core(struct(material)) = Q_struct
Pure(Q_struct)
TypeRole(Q_struct)
CallSpace(tau_struct) = V_τ
```

struct guarantees its result's core and TypeRole through its formation rule.
The subsequent ordinary binding, named contribution, or structural-let
reference assignment carries the already complete value. The destination name
does not provide a missing type component or reroot the result.

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

left-side let binding/installation path:
  determines only the Place where the construction is installed
```

**In-place closures are transparent to `struct` navigation in meta context.**
When `struct` resolves the ambient `ResolvedPatternScope`, it sees through any
in-place (inline-called) closures within the meta body until it reaches the
meta function call entry point. These intermediate closures do not contribute
navigation components to `struct`'s owner resolution:

```text
meta body:
  in-place closure invocation  <-- struct sees through this
    in-place closure body
      ... |> struct            <-- resolves owner at the meta entry scope,
                                   NOT at the in-place closure scope
```

Only non-meta contexts observe these in-place closures as affecting navigation
names. The rationale is: in-place closures within a meta body are control-flow
mechanisms (combinators, continuations, local abstractions) that do not
represent semantic ownership boundaries. The meta function call entry is the
canonical ownership boundary; closures called within it are internal structure.

Therefore:

```lang
let t1::t = (...) |> struct;
```

does not reroot the right-hand pattern into the internal pattern scope of
`t1::t`. Its effect is:

```text
FreshNamedType(t, t1, P)
  -> commit Some(T_0) and return the destination mut type ref
ordinary assignment through that reference
  -> evaluate the struct RHS under its own ordinary owner rules
  -> validate assignment and replace the resident with the complete result
  -> preserve the result's resolved owner
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
referentially pure
```

Purity means that `struct` does not install a name binding or mutate an
input place. It may establish the result type's declared `StructLexicalRoot`
under its privileged owner rule, but outer `let` remains the only operation that
creates the destination name binding/member in the surrounding graph.

It does not install a `NamespaceDelta`. Private construction material records
the decoded body needed to form the canonical Pattern and complete type; it is
not observable in `Norm` and does not mutate language-visible input. Graph
installation remains outside formal invocation.

### 7.4 Structural leaves and pure Pattern nodes

`struct` recognizes the shape inside each leaf parentheses. The value-bearing
leaf form is:

```text
Expr name
```

`Expr` supplies the leaf value/type material and `name` supplies that leaf's
Pattern name. This resembles the token order of a C-style field declaration
only as a surface mnemonic; it imports no C type, layout, object, or field
semantics.

A single `name` with no preceding `Expr` is instead a pure Pattern node:

```text
name
  -> null x P(name) x Val2(name)
```

It has no Val1. This is the basis on which no-value alternatives such as
`if | else` remain visible Pattern material rather than being rejected as
missing fields.

A named empty Pattern is valid:

```lang
let t = (()t) |> struct;
```

Here `()` supplies an empty child layer and `t` supplies the Pattern name. The
result is not a value-bearing field. This rule does not by itself assign a
meaning to an anonymous bare `() |> struct`; that is a separate boundary.

### 7.5 Generated field and companion members

For a structural field `f : A` produced during the `struct` formation event, let
`tau_struct = struct(material)` and
`T = tau_struct`. The core `Q_struct = Core(tau_struct)` is produced during that
formation event; there is no intermediate `S_struct` from which it is projected.
`struct` uses one general field rule. It does not introduce a separate semantic
category for “type fields”. All observations are candidates of one same-name associated
name binding `f`; receiver and result observation kinds distinguish the overloads.
The `struct` generator produces the full `GeneratedFieldFamily(T, name, A)` —
the by-value accessor plus the `ref`/`share` policy triples with their exact
`default` / `delete` cells (canonical schema in
`type-associated-function-objects-and-access-trees.md`). Erasing policy detail,
the family presents as:

```text
f : (object: T)       -> A
f : (object: T ref)   -> A ref
f : (object: T share) -> A share
```

`ref` and `share` are not generated navigation subspaces. The same-name family
is stored once as ordinary callable/member Objects. Its direct anonymous
classifier home is `TypeMemberScope(Q_struct)`, so it belongs to `V_τ`; `const
let` / `let` / `mut let` policy and the formal object type determine its
candidates.

The `ref` / `share` type constructions do not copy that family. With respect
to inherited associated names, each derived type value `T ref` / `T share`
generates ordinary forwarding function objects
(`ForwardAssoc`, §2.1 immutable complete-type callspace): `f::(T ref) ->
f::T` and `f::(T share) -> f::T` are fresh derived-type members homed in the
derived type's own `V_τ`, whose bodies perform a new ordinary invocation of the
base family. The model is therefore:

```text
struct
    generates the real field family under T

ref/share type construction
    for inherited associated names:
        generates ordinary forwarding function objects
    derived τ still owns its intrinsic
        ref/share formation, borrow formation,
        fixed-point/weakening, and other native callspace members
```

every contributed callable retains its complete type and owner.
Their selection uses the ordinary context-indexed preference relations. In a
plain context `succ_plain: plain > const = mut`; if no `plain` candidate is
admissible, a surviving `const` and `mut` pair remains ambiguous rather than
being resolved by generation order.

Where the field policy permits mutation, the same generator also contributes
field write candidates shaped `T ref × A`. They form a field-specific setter
family `FieldWriteFamily(T, name, A) ⊆ Candidates(=::adl)` — an ordinary
associated family reachable through the same `.=` entrance, and **never** the
universal `AssignmentFamily(T)` (whose domain is `T ref × T`): for every `A`,
`FieldWriteFamily(T, name, A) ≠ AssignmentFamily(T)`; at `A = T` the two have
coincident formal shape only, never coincident family identity (canonical
field-side rules: `type-associated-function-objects-and-access-trees.md`).
Field write, accessor, and policy cells are all registered under the stable
call-site family identity `StructuralFamily(T, name, A)` =
`StableFamilyId(CoreAnchor(Q_T), name, StructuralDefault)` that P-internal
extraction filters on; the identity key is the stable core anchor
(§2.1), not the whole `Q` snapshot. Family registration and the stability
theorem are normative in
`type-associated-function-objects-and-access-trees.md`. Assignment still uses
the general existing-place write rule and never creates the field. Written
`const let` / unqualified `let` / `mut let` field policy selects the admitted
value, shared, mutable, and assignment cells of this ordinary overload
family. The exact machine body and access-tree representation are implementation
debt, not additional semantics.

Accessor stage follows one structural predicate rather than a coarse “type” or
“PatternValue field” category:

```text
RuntimeField(f)
  <=> Val1_f != absent
    and Materializable_0(Val1_f)
    and not RequiresStaticPattern(f)

Stage(accessor(f))
  = runtime || compile   if RuntimeField(f)
  = compile              otherwise
```

`Materializable_0` means that the current first-order runtime object model can
materialize the complete selected `Val1` without a static-only witness;
`RequiresStaticPattern(f)` means that selecting or constructing the field
observation intrinsically depends on PatternValue material unavailable at
runtime. Both are structural judgments over the field object, not nominal type
lists.

A type-valued field is compile-only only because it fails this predicate in the
current runtime model; it is not a special field category. Ordinary runtime
values remain PatternValues and are not excluded by that fact. The mechanically
generated `[]` observations of `T*N` and `T*omega` use this predicate for the
selected element but retain all ordinary call dependencies:

```text
Dependencies(Index(s, i)) = { container observation s,
                              index observation i,
                              selected element observation }
Stage(Index(s, i)) = meet { Stage(d) | d in Dependencies(Index(s, i)) }
```

`RuntimeField(selected element)` is one local condition inside that meet. No
Sequence-specific stage rule exists.

The generated partner candidates are ordinary members whose classifiers
satisfy `TypeMember_Q_struct`; they enter `V_τ` during the `struct` formation
event, and `Core(tau_struct) = Q_struct` exposes them as its associated members.
Any navigable associated
view is a projection of those same members, not a second owned copy in
`Q_struct` or its `Val2`. The partners are ordinary typed member objects: user
construction may remove them, replace them, or add a more specific declaration
subject to the ordinary duplicate, fallback, and overload rules. They are not
hidden compiler metadata.

The closed structural generator contract stops at the same-name field
value/ref/share observations and the corresponding assignment/write partners
described above:

```text
struct closure = field + access + ref/share observation + assignment/write partners
```

Type-as-callee is now closed without any defining-name binding recovery:

```text
TypeValue(t) = tau = <Q,V_τ>
CallSpace(tau) = V_τ
```

A copied or extracted type value retains the `V_τ` of that immutable `tau`
snapshot. A complete type has no home name binding and no reverse carrier, source-place,
or `AsType` identity route.

Open authority does not propagate along owned field relations. Each
PatternValue's open authority is determined independently by stack-relative
authority-frame resolution:

```text
OpenHere_Σ(v)
  iff WindowLive_Σ(v)
  ∧ AuthorityMatches(v, Σ)
```

No parent-to-child or child-to-parent implication holds; a terminal event that
closes multiple windows in one structural region does so because each value
independently fails `WindowLive_Σ` or `AuthorityMatches`, not because a
neighboring value closed. Borrow edges are horizontal and do not participate.
Mutability is independent:

```text
mut(child) does not imply mut(parent)
mut(parent) does not imply mut(child)
```

This same rule makes a typeclass-like object an ordinary struct; its fields are
compile-only exactly when they fail `RuntimeField`, not because they inhabit a
separate “type/PatternValue field” category.

### 7.6 Internal construction and later extension normalize equally

An element written inside the original `struct` input and an equal element
added later through the owner's navigated structural-extension path differ only
in **how their full navigation is obtained**. They do not differ in the Pattern
**entity identity** of the child. For example:

```lang
let t = ((bool inner)t) |> struct;
```

and the construction sequence using place-level `inject`:

```lang
let s = (()t) |> struct;
let t_ref = s |> (type ref);
(t_ref, bool inner) |> inject;
```

produce type values whose core `Q_struct` members both satisfy `TypeRole`,
provided the read value is Open and the destination slot is writable. Both
paths install exactly one canonical Pattern child under `t`:

```text
exists exactly one C.
  C = inner::t
  and DirectPatternChild(t, inner, C)
  and LeafSource(C, bool)
  and Norm(leaf value of C) = Norm(bool::)
```

The canonical entry is the child entity `C = inner::t` carrying its leaf value;
the same structural theorem is stated in
`../patterns-overload/pattern-values-relational-semantics-and-extraction.md` §12.
The two construction paths differ only in formation/navigation provenance:

```text
SameChildPattern(C₁, C₂) ∧ DifferentNavigationFormation(path₁, path₂)
  ⇒ SameCanonicalEntry(C₁, C₂)

-- the converse does not hold: erasing formation provenance never erases the
   child entity itself, and never equates distinct pattern children
```

The first form inherits/completes `inner` under `t`; the second supplies the
same child material through pure `extend`, then writes it back through the
type-level carrier slot reached by `s |> (type ref)`. After
completion, normalization retains the child entity `inner::t` and its
normalized leaf value. It erases only how the child's navigation was obtained
(inherited versus explicit) and how the child was formed (internal versus
extended) — never the Pattern entity identity of `inner`.

Structural `let inner::(s |> (type ref)) = bool::;` first forms the fresh
complete T_0 at inner, then ordinary assignment replaces it with the complete
type read through bool::. The associated Val2 resident is that complete type,
not a binding identity or a raw initializer entry. It does not
register `inner` in `t`'s Pattern structure. Pattern-member registration is a
privilege of `struct` inline construction and the `extend` primitive (directly
or through `inject`). See §12.1 for the full privilege boundary.

### 7.6.1 Value-supplied members use the same formation relation

The equivalence also applies to ordinary member construction material whose
RHS has already evaluated to v. It is not limited to the field spelling in the
example above. The construction position determines the member role (named
contribution, structural field, or exact () entry); value shape does not choose
a different role.

Use the following mathematical notation for the existing formation relation:

    Delta_v = that ordinary member material, with evaluated RHS v
    S_a(B ; Delta_v) = one-shot struct formation with base material B
                      and that member present from the start at anchor a

Delta_v is the same semantic input accepted at the corresponding struct
position. It is not an inferred arbitrary Pattern admitting TypeOf(v), a new
language value class, or a user-exposed AST. B describes the existing base
construction; it does not authorize rerunning its effects or original source.

For the same base, member role, declared policy, resolved dependencies,
captures and target anchor:

    T_B = formed base snapshot
    T_1 = Extend_Gamma(T_B, Delta_v)
    T_1 equivalent_to S_a(B ; Delta_v)

The equality observes the complete result, including Core, captured V_tau,
ordinary generated partners, and internal identities under consistent bound
alpha-renaming. It is not merely satisfaction equivalence or equality of Core
lookup indices. No unrelated helper, field or larger admissible Core can be
added by the incremental path: its result must be the one-shot formation
result for that exact material.

If v already has the required membership, member formation retains it.
Otherwise, an eligible ReinstantiationWitness supplies the same anchored
instance that the corresponding one-shot member construction forms. The
resolved capture values and the anonymous identity graph are preserved under
the existing replication rules. The target anchor is already fixed independently
of the Core contents; it is not recovered from the LHS by RHS evaluation.

This determines Core preparation by projection of the existing formation:

    Q_1 = Core(S_a(B ; Delta_v))
    v_a = the member instance formed in that result
    TypeOf(v_a) in Q_1

The local contribution step is TypeAdd after that Core preparation, together
with the ordinary generated-member closure. Extend returns the whole completed
snapshot; inject commits it through the ordinary reference. Thus an externally
performed inject of Delta_v already includes v_a exactly once. Appending another
TypeAdd afterward would be a second contribution, not this derivation.

The comparison is a formation law, not replay of source code or equality of
execution traces. Incremental formation retains its actual FreshNamedType,
OpenHere, Writable, lifetime and Pre/commit/Post events. A hypothetical
one-shot expression grants no missing incremental authority. If the ordinary
one-shot member formation is undefined (including missing witness or illegal
captures), this equivalence supplies no alternate successful construction.

## 8. `extend` and `inject`

### 8.1 Privileged built-in

`extend` and `inject` are future bounded privileged operations, parallel to
`struct` in trust boundary. Neither creates an ordinary externally navigable
`MetaInstanceScope M`:

- it accepts normalized pattern syntax or an equivalent internal AST carrier;
- `extend` returns `type`; `inject` returns the input `type ref`;
- it does not re-enter the parser;
- it does not concatenate arbitrary tokens;
- it does not expose unrestricted AST-consuming capability to user functions;
- they perform only bounded pattern-child construction.

The source examples in this section are semantic sketches. They do not change
the frozen parser or introduce traditional `f(args)` call syntax.

### 8.2 `extend` is the primitive pure value transformation

`extend` takes one complete ordinary type snapshot and struct-like child
material, and returns a new complete type snapshot:

```text
extend : type × StructLikeMaterial ⇀ type

old = bind alpha. <Q_old, V_old[alpha]>

Extend_Gamma(old, Delta)
  => new = bind beta. <Q_new, V_new[beta]>
```

`extend` establishes no root and preserves the root already carried by its
input:

```text
Root(new) = Root(old)
```

Root preservation is not snapshot equality and never redirects older copies to
a current mutable name binding:

```text
new != old                 when the extension contributes semantic material
V_new != V_old             when generated/direct TypeMembers change
CallSpace(old) = V_old
CallSpace(new) = V_new
```

The structural contribution first changes `Q_new` under the canonical Pattern
relation. Any generated classifier whose
`membership admitted by the current construction` contributes its ordinary
members to `V_new`. Both components belong to the returned snapshot.

There is no construction-handle rank. The input is an ordinary value of rank
`type`; `type ref` and `type share` are not accepted inputs. A caller may first
clone/read through a view to obtain the ordinary value, but the view contributes
no construction permission.

The function is total in its effects in the following sense:

```text
Extend does not modify old
Extend does not install a namespace delta
Extend does not perform an assignment
```

`old` is an input value and is left exactly as it was, including its `V_old`
callspace. `new` is a distinct resulting value. Discarding `new` produces no
symbol-world side effect, because there was never a side effect to discard.

#### 8.2.1 Failure is total

```text
failure => no partial result, no write, no rollback
```

Because `extend` writes nothing, a failed `extend` has nothing to undo. There is
no half-extended pattern, no compensating action, and no rollback protocol. A
failed call simply produces no value.

#### 8.2.2 `extend` applicability is a construction-authority judgment

The primitive checks the old value in the current evaluation context:

```text
Γ ⊢ old : type
OpenHere_Σ(old)
ParentToChild(old, Δ)
NoPatternConflict(old, Δ)
Canonicalizable(result)
--------------------------------
Γ ⊢ (old, Δ) |> extend : type
  and WellFormedTau(result)     -- independently checked on the result structure
```

`OpenHere_Σ(old)` is derived from `Anchor(old)` and the authority-frame
resolution of §12.1.1 (non-meta: `AuthorityFrame_Σ(Core(old))` exists; meta:
coordinate equality against `CurrentEvaluationCoordinate_meta`), not from a
carrier place. Because `old` is a complete type value `τ` rather than an
ordinary `PatternValue`, the horizontal attributes resolve by Core projection (§12.1.2): `OpenHere_Σ(old) = OpenHere_Σ(Core(old))`. Clone/read
preserves the anchor:

```text
Anchor(Clone(old)) = Anchor(old)
```

Consequently an `OpenHere` value with no writable carrier may be extended and
bound elsewhere, while a closed-window value read through a writable
`type ref` is rejected. There are deliberately no `type ref` or `type share`
overloads for `extend`.

A navigated `let child::target = result;` performs FreshNamedType through the
existing authorized parent reference, then ordinary assignment. It creates an
associated named-type resident, not a registered Pattern-child edge. It cannot
substitute for extend's structural registration or inject's write-back.

#### 8.2.3 `inject` is the read--extend--write wrapper

`inject` accepts exactly a writable type-slot view and struct-like material:

```text
inject : type ref × StructLikeMaterial ⇀ type ref

Inject_Σ(r, Δ):
  require Writable_Γ(Target(r))
  old := Clone(Read(r))
  new := Extend_Σ(old, Δ)       -- independently requires OpenHere_Σ(old)
  Write(Target(r), new)           -- ordinary slot replacement, not construction
  return r
```

The two requirements are deliberately independent:

```text
CanInject_Σ(r, Δ)
  = Writable_Γ(Target(r))
  ∧ CanExtend_Σ(Clone(Read(r)), Δ)
```

`inject` is the composition `clone/read old τ → Extend → ordinary Write back`.
The step that depends on construction authority is `Extend`; the final
`Write` is an ordinary slot replacement (`slot := x'`) that needs only
`Writable_Γ(p)` and the slot's local constraints. Ordinary slot replacement
is **not** a `τ -> τ'` construction transformation: it does not require
formation history, and it does not automatically acquire `extend` semantics
just because the carrier is a type value.

`r : type ref` proves target/lifetime/capability only. It never proves the
current pointee satisfies `OpenHere_Σ`. A closed-window pointee may
therefore be replaced wholesale by ordinary assignment through a writable
ref, while `inject(r, Δ)` fails before the write because its `extend` step is
inadmissible.

Failure before `Write` leaves the target unchanged. `type share` has no
`inject` candidate because it is not writable; by-value `type` has no `inject`
candidate because it supplies no destination place. Both may still participate
in pure value computation where their ordinary value is accepted.

Canonical source supplies an explicit mutable type reference:

```lang
let r = T |> (type ref);
(r, delta) |> inject;
```

The reference preserves its actual target and performs no provenance recovery. The result is the same ref `r`, now observing the
successfully written value.

### 8.3 Navigation direction

The distinction between `struct` and `extend` is navigation direction, not
ownership authority; `inject` delegates its middle step to `extend`:

```text
struct:  resolves OUTWARD
  resolve owner by ordinary input navigation + ambient scope
  (always looks up for the top-pattern navigation name)

extend:  resolves INWARD
  takes the input pattern value as the navigation anchor;
  children inherit that pattern's path
  (never looks outward for a top-level scope)

inject: read target -> extend inward -> write the same target
```

This is the whole reason `extend` needs an existing pattern value as input: it
needs a pattern whose navigation path is already resolved, so that the new
children can be linked beneath that path.

Example. `t1::r` is an ordinary pure-pattern path, so it is not a legal
assignment left side (§8.2.2); the carrier slot has to be taken first:

```lang
let r_ref = (t1::r) |> (type ref);
(r_ref, (t first, u second)) |> inject;
```

Pure value construction is separate and performs no write:

```lang
let old = t1::r |> type;
let next = (old, t first) |> extend;
let final = (next, u second) |> extend;
```

The first form performs one read--extend--write transaction through the ref; the
second produces values only. The resulting type Pattern is:

```text
(
    t first::t1::r,
    u second::t1::r
)::t1::r
```

`extend` determines the child set of the resulting pattern value. It does not
change owner identity or reopen a closed-window value. `inject` additionally requires
the target to be writable; formation of `r_ref` alone proves neither premise.

As with `struct`, the lowest-level leaf reduction has the form:

```text
E name
```

At that leaf:

- `name` is the leaf's pattern name;
- `E` is value-bearing material that must be resolved through its external
  binding binding and then evaluated;
- different leaves do not require the same `E`.

Consequently:

```text
t first
u second
```

means:

```text
first is the pattern name; the leaf value is read through binding t
second is the pattern name; the leaf value is read through binding u
```

Pattern-name identity and leaf-value origin are independent. Using `t` for both
leaves would obscure this distinction.

### 8.4 Child-only restriction

`extend` extends the input pattern by **direct children only**; `inject` inherits
the same restriction:

```text
Extend(old, Δ) may add children directly beneath P(old)
Extend(old, Δ) may not reach into a grandchild layer
```

Extending a deeper layer is expressed by composing the operation at that layer —
read the child value, extend it, and write it back where independently
authorized — not by giving either primitive a deep path.

Within that scope, `extend`:

- adds direct children to the resulting pattern;
- preserves the owner identity carried by the input pattern.

It does not:

- replace the owner;
- overwrite an existing core `Core(τ)`;
- delete an existing child;
- implicitly reroot an arbitrary external pattern value;
- mutate the input value or the installed namespace graph;
- extend a value that is not `OpenHere_Σ` in the calling context;
- grant a general macro or arbitrary AST-rewrite capability.

`inject` adds only the ordinary write to an already existing target; it does not
relax any `extend` restriction. Failing Open or write applicability produces no
partial write.

## 9. Pattern-Layer Ordering

This section applies the canonical named-versus-positional and structural-child
rules from
`../patterns-overload/pattern-values-relational-semantics-and-extraction.md` to
name binding construction. It is not an independent definition of Pattern identity
or relational equivalence.

Let the direct children of one pattern layer be:

```text
p1, p2, ..., pn
```

The ordering rule is decided at the level as a whole. Order-insensitivity
requires both:

```text
the sibling level is wrapped by a Pattern;
every direct child has a top-pattern navigation layer.
```

A naked Product never satisfies the first condition. Therefore:

```text
(a, b)c == (b, a)c
(a, b)  != (b, a)
```

Naming both Product elements does not by itself erase their positions.

The normalizer must therefore preserve two distinct node kinds until this
decision has been made:

```text
ProductNode(children)
PatternLayerNode(name, body)
```

It must not flatten both into one undifferentiated children list and then infer
the node kind from whether every child has a complete navigation. Complete
navigation is necessary for an unordered Pattern body, but it is not
sufficient.

### 9.1 Fully named body of a Pattern

If a sibling layer is the body of a Pattern and every direct child has a
top-pattern navigation layer:

```text
normalize layer
  -> Map<CanonicalFullNavigation, CanonicalPatternValue>
```

For example:

```text
{
    bool::,
    t1::t,
    t2::t
}
```

Every entry contains an already completed Pattern navigation and its normalized
resident value. Neither coordinate is a source `name binding`, source path, or binding
reference. The complete navigation is the canonical map key; the resident is
the canonical value at that navigation.

Consequences:

```text
the whole layer is order-insensitive;
layer equality is canonical map equality;
different-name extensions commute;
same-navigation/different-value conflicts are rejected before map formation.
```

For example:

```lang
t1::r
|> extend(t first)
|> extend(u second)
```

and:

```lang
t1::r
|> extend(u second)
|> extend(t first)
```

produce the same pattern value because both direct children have top-pattern
names.

Once normalized, the map does not classify elements as “internal patterns” or
“external patterns.” Parent-scope inheritance, explicit `::`, ordinary binding
binding, and `extend` explain how a `PatternValue` was resolved or produced
before normalization. After its navigation name is fully qualified, source
category and construction route do not participate in `PatternValue` identity,
map equality, or extraction semantics.

An implementation may retain source binding, inherited/explicit navigation,
binding origin, or injection origin as provenance for diagnostics and replay.
That provenance must not affect `PatternValue` equality.

Insertion of an equal `(complete navigation, normalized resident)` entry is
idempotent. Distinct source bindings may remain distinct extraction entry paths
while contributing only one canonical map entry:

```lang
let a::t = bool;
let b::t = bool;
```

```text
Read(Place(resolve(a::t))) = bool::
Read(Place(resolve(b::t))) = bool::

{
  FullNav(bool::) -> Norm(bool)
}
```

Each statement above first commits its own T_0 and then assigns the read
complete bool type through ordinary assignment. The following equalities and
normalization describe the state after both assignments succeed, not an
Absent-to-RHS transition. Both paths may then be used as source navigation paths. After binding
resolution and value read, both look up the single `bool::` entry. The layer
is neither a multiset nor a relation keyed by the carrier name binding's source name.
It is keyed by canonical complete Pattern navigation.

name binding paths and `PatternValue` navigation names may coincide or differ. For
example, the same spelling may describe:

```text
binding navigation path:                 t1::t
PatternValue navigation carried there:  t1::t
```

The `t1::t` key in a normalized map is still canonical Pattern navigation; its
spelling does not turn it into a name binding reference. Conversely:

```lang
let t3::t = bool;
```

after fresh formation and successful ordinary assignment establishes:

```text
binding navigation path:                 t3::t
PatternValue navigation carried there:  bool::
```

The binding path and value path are then visibly different. Both cases use the
same name-resolution/value-read semantics.

### 9.2 Naked Product or Pattern body containing a bare value

The layer is order-sensitive if either:

```text
it is a naked Product; or
it is a Pattern body with at least one bare direct child.
```

In either case:

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
Fully named body of a Pattern:
  representation =
    Map<CanonicalFullNavigation, CanonicalPatternValue>
  membership/equality use the complete navigation and normalized resident
  order-insensitive

OrderedPatternLayer:
  position-preserving, order-sensitive
  used for every naked Product
  also used for a Pattern body containing any bare direct child
```

A canonical serializer may sort a fully named map by canonical complete
navigation encoding. Sorting is only a stable representation of map semantics;
it must not be presented as preserved source-order meaning. An ordered layer
must preserve positions.

### 9.4 Navigation, ordering, and optional peeling are orthogonal

These mechanisms answer different questions:

```text
navigation completeness:
  determined by OwnNavigation and Pattern-parent anchor traversal

ordering:
  determined by ProductNode versus PatternLayerNode

optional top peel:
  erases one top Pattern identity while retaining an anonymous
  PatternLayerNode boundary and that layer's ordering
```

The future default `?` operation must therefore use:

```text
PatternLayer(c, B, O)
  ?-> PatternLayer(NameAbsent, B, O)
```

not:

```text
PatternLayer(c, B, O)
  ?-> Product(B)
```

If no top Pattern is peelable, `OptionalPeel(x) = x`; this is a fixed point,
not failure and not `none`. The retained layer boundary must guarantee:

```text
PeelView(Norm(x)) = Norm(PeelView(x))
```

This is a recorded future extraction invariant. It does not claim that the
current evaluator implements `?`.

## 10. Child Uniqueness and Replay

“Extend once” applies to a complete child navigation path, not to the owner as a
whole.

For named direct children, the conceptual uniqueness key is:

```text
(owner PatternScopeId, child top-pattern identity)
```

This is a construction-time path-conflict key, not the representation of the
normalized layer. After successful validation/evaluation, the child contributes
its complete-navigation/normalized-value entry to the canonical unordered map.

Therefore:

```lang
|> extend(t first)
|> extend(u second)
```

is valid, while:

```lang
|> extend(t first)
|> extend(u first)
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

An ordered layer still preserves positional identity; a binding-keyed or
name-keyed map must not replace either the ordered layer or the normalized
map keyed by canonical complete Pattern navigation.

## 11. Extraction and Explicit Navigation

This section applies the canonical navigation-formation and child-identity
rules from
`../patterns-overload/pattern-values-relational-semantics-and-extraction.md` to
symbol-first lookup. Formation provenance may be retained for diagnostics but
does not define a competing Pattern normal form.

### 11.1 Navigation always reaches a name binding before a value

Both inherited and explicit pattern navigation use the same final two steps:

```text
binding resolution
  -> value read
```

They differ only in how the binding path is formed.

Each Pattern layer has one own-navigation state:

```text
OwnNavigation(layer) =
    Explicit(path)
  | ImplicitGlobal
  | Absent
```

`Absent` is valid only for a non-root layer and means that completion continues
through the semantic Pattern-parent link. A root Pattern whose navigation is
omitted has `ImplicitGlobal`, never `Absent`. Therefore the anchor is total:

```text
Anchor(x) =
  nearest ancestor a of x
  where OwnNavigation(a) != Absent
```

A bare name is completed by walking its already existing Pattern-parent chain
from nearest to farthest:

```text
name
  -> append the nearest parent's local navigation
  -> continue through every parent whose OwnNavigation is Absent
  -> stop at the nearest Explicit(path) or ImplicitGlobal anchor
  -> resolve that completed name binding path
  -> read the PatternValue carried by that name binding
```

Equivalently:

```text
FullNav(x) =
  LocalSegments(x -> Anchor(x))
  :: Navigation(Anchor(x))
```

This walk does not classify either the subject or any parent as internal or
external. Those source/construction categories are irrelevant to navigation
completion. The only question at each parent is whether that parent explicitly
specified its own navigation level.

The top Pattern is always the final anchor. If its own navigation was omitted,
the omission means an exact global lookup—implicit `::`. It does **not** mean
“treat the top name as an ordinary bare name and search
`near -> outer -> core`.” That ordinary scope chain belongs to value/call
target resolution, not extraction navigation completion.

Navigation completion never infers a missing parent by reversing or guessing
from the resident's spelling. It follows only semantic Pattern-parent links
that already exist.

An explicitly navigated extraction subject does not inherit the Pattern-parent
chain:

```text
::external
  -> begin at the explicitly selected external name binding layer
  -> resolve that name binding path
  -> read the PatternValue carried by that name binding
```

In the current inner-to-outer surface notation, an explicitly terminated
external component is written as `external::` where a grouping boundary is
needed. The conceptual `::external` description above emphasizes that the
external layer is selected rather than parent-completed; it does not reverse the
frozen source navigation order.

Default inheritance is therefore not “indirect value access” while explicit
navigation is “direct value access.” Neither form directly touches a pattern
value. Both first produce one exact binding path, resolve it, and then read its
value.

The pattern expectation permits only a `PatternValue`/pattern interface exposed
by that binding. It does not fall back to invoking arbitrary ordinary values or
callables from the heterogeneous typed `V` members.

### 11.2 Assigning a fully qualified PatternValue to a fresh structural name

Given a globally bound complete type:

```lang
let bool = ((if | else) bool) |> struct;
```

The structural identity NameBindingId(bool) differs from the Pattern head of
the complete type read through that binding. NameBinding is not another Object.

With t an existing authorized mut type ref:

```lang
let t1::t = bool;
```

has exactly this structural trace:

```text
FreshNamedType(t, t1, P)
  -> NameBindingId(t1::t), resident Some(T_0), mut type ref r_t1
ordinary assignment r_t1 = bool
  -> Resolve(bool) = one terminal NameBinding b_bool
  -> Read(BindingPlace(b_bool)) = complete type T_bool
  -> selected assignment Pre, replacement commit, Post
```

Formation and replacement have their own resident-generation, window and
failure events. They are not a direct general lexical binding operation.
After successful assignment, the stored complete type has Core navigation
bool::; its owner/navigation is not changed to t1::t. The empty T_0's creation
anchor does not reparent T_bool or reopen its construction window.

Subsequent normalization/extraction below observes this successfully committed
state; it does not erase or redefine the preceding construction trace.

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

For the shorthand, resolving bare `t1` starts at its nearest Pattern parent
`t`. Here `t` is also the nearest navigation anchor, producing the binding
path:

```text
t1::t
```

The evaluator then resolves the terminal binding for `t1::t` and reads its bound
`PatternValue`. That value reveals its fully qualified pattern navigation:

```text
bool::
```

For the explicit form, `bool::` explicitly terminates the external binding path
(the conceptual `::bool` choice) and blocks completion under the current parent
`t`. The evaluator resolves the terminal binding for `bool` and then reads the `PatternValue`
carried by that binding.

Both paths therefore reach:

```text
P = if::bool | else::bool
```

The distinction is solely:

```text
inherited form:
  follow Pattern parents through Absent layers to the nearest
  Explicit(path) or ImplicitGlobal anchor,
  then resolve exact name binding path -> read PatternValue

explicit form:
  select an external binding path, then resolve name binding -> read PatternValue
```

It is never a distinction between an indirect pattern value and a directly
named pattern value. Source navigation names bindings first. A pattern's
canonical/diagnostic navigation may match a source name binding spelling without
becoming the same identity.

### 11.4 Extraction looks up PatternValue in the canonical map

For a fully named sibling layer that is the body of a Pattern, normalization
produces:

```text
M: Map<CanonicalFullNavigation, CanonicalPatternValue>
```

Extraction is therefore value lookup, not binding lookup. The normative process
is:

```text
1. Complete the source navigation path by walking Pattern parents through
   `OwnNavigation = Absent` to the nearest `Explicit(path)` or
   `ImplicitGlobal` anchor. Honor an explicit subject navigation without
   parent completion.
2. Resolve the completed path to a name binding.
3. Read the PatternValue bound to that name binding.
4. Split that normalized PatternValue into its complete navigation and
   normalized resident, then look up the equal entry in M.
5. If present, continue extraction through the matched PatternValue.
```

Formally:

```text
extract(path, M)
  = lookup(canonical_entry(Read(BindingPlace(Resolve(path)))), M)
```

not:

```text
lookup(Resolve(path), M)
```

because `M` contains evaluated canonical navigation/value entries, not
name-graph nodes or name binding references.

For example:

```lang
let bool = ((if | else) bool) |> struct;
let t3::t = bool;
```

and:

```text
M = {
  FullNav(bool::) -> Norm(bool),
  FullNav(t1::t)  -> Norm(t1),
  FullNav(t2::t)  -> Norm(t2)
}
```

the extraction path:

```text
t3 t
```

first inherits parent navigation and forms binding path:

```text
t3::t
```

Then:

```text
Resolve(t3::t) = b_t3
Core(Read(BindingPlace(b_t3))) has canonical navigation bool::
canonical_entry(bool::) ∈ M
```

Thus `t3 t` matches `bool::`, not `t3::t`.

By contrast, if:

```text
Core(Read(BindingPlace(Resolve(t1::t)))) has canonical navigation t1::t
```

then the source binding path and resulting `PatternValue` navigation happen to
share a spelling. The extraction still performs binding resolution and value
read before set lookup; the shared spelling does not permit either step to be
omitted.

## 12. Facet Conflicts and Installation

### 12.1 Structural incidence and name contribution

Structural Pattern children and ordinary lookup members remain separate.

    PatternChild:
      struct Pattern material or structural extend input
      -> R_Gamma / canonical child normalization / DirectPatternChild evidence

    NamedContribution:
      normalized construction position with a structural target
      -> ordinary anchored closure contribution to that named type's V_tau

An ordinary member is not a real Pattern child merely because it is visible.
The same-spelled structural field's complete generated accessor family is
ordinary group material; StructuralDefault extraction selects the registered
real-field family before ordinary overload enumeration.

Explicit P let name::path requires freshness, creates the name, records P, and
returns a mutable construction type reference. Following assignment is the
ordinary reference operation. Unqualified let name = expression has implicit
named-type synthesis sugar only in a named-contribution position; lexical let
retains ordinary binding semantics.

Declared policy, construction-reference mut policy, per-member visibility,
Writable, and OpenHere are independent. Each navigation layer preserves its
own exposure and policy facts. A hidden existing name is not fresh. Group
membership neither changes a member's Pattern nor invents structural incidence.

These operations use the same existing authority judgment below. A[type]'s
associated place also depends on it; it does not introduce a separate window or
ownership kind.

#### 12.1.1 Open authority is stack-relative

Every constructed PatternValue carries a structural anchor and an immutable
birth regime. Whether it may be structurally modified in the current evaluation
context is a separate, dynamic judgment that combines the value's static anchor
with the evaluation stack and the current open-window state:

```text
Anchor(v) = ⟨PatternRoot(v), Navigation(v)⟩

GenerationRegime(v) ∈ { MetaGenerated, NonMetaGenerated }
                     -- immutable birth classification (value attribute)

WindowLive_Σ(v)       -- construction window still open at current program point
                       -- evaluation/window state, not a value attribute
Visible_Σ(v)          -- current frame can obtain v

OpenHere_Σ(v)
  iff WindowLive_Σ(v)
  ∧ AuthorityMatches(v, Σ)

AuthorityMatches(v, Σ)
  iff AuthorityFrame_Σ(v) exists

Anchor(v) ∉ Norm(v)
CarrierPlace(v) ∉ Anchor(v)
GenerationRegime(v) ∉ Norm(v)
```

`GenerationRegime(v)` is fixed at creation. `WindowLive_Σ(v)` is a property of
the current evaluation state: the construction window has not been permanently
closed at the current program point. `OpenHere_Σ` adds the contextual question:
does the current evaluation stack still contain the frame that owns this value's
anchor, and is the window still live there? `Visible_Σ` adds a third state: the
value exists and the window may still be live, but the current frame cannot
obtain it (for example, it is shadowed by a deeper meta invocation — see below).

Clone, value copy, and compile transport preserve the anchor and regime; they
do not preserve or manufacture source-place identity, and they do not create a
fresh window state:

```text
Anchor(Clone(v))    = Anchor(v)
Anchor(let-copy(v)) = Anchor(v)
```

Construction authority is resolved **per value** against the evaluation
stack. The PatternValue supplies the static anchor; the stack supplies each
level's current evaluation position; authority then belongs to the frame that
still owns that anchor — not unconditionally to the stack-top callable:

```text
Frame = ⟨ CallableRoot, MetaPartnerRoot?, ActiveInlineClosurePath ⟩

EvaluationCoordinate(f)
  = ⟨RootCoordinate(Callable(f)), ActiveInlineClosurePath(f)⟩

RootCoordinate(F)
  = MetaPartnerRoot(F, GenericArgs)   if Generic(F)
    CallableRoot(F)                   otherwise

AuthorityFrame_Σ(v)
  -- the nearest still-active frame owning Anchor(v),
     resolved per regime (below)

CurrentAuthority_Γ     -- typing-context form of the same judgment
```

For a **meta** context, walk the compile-time stack in reverse, skipping
`compile` and transparent construction-intrinsic frames. Let `M` be the first
ordinary meta invocation frame found; `NearestMetaRoot(Σ)` is its MetaInstance
root. In-place closure navigation is transparent for authority purposes
(`VisibleInlinePath_meta(path) = ε`), so the meta authority frame degenerates
to the nearest meta root:

```text
CurrentEvaluationCoordinate_meta(Σ)
  = ⟨NearestMetaRoot(Σ), ε⟩

AuthorityFrame_Σ(v)                  -- meta context
  = the nearest meta invocation frame M such that
      EvaluationCoordinate(M) = Anchor(v)
  -- equivalent to Anchor(v) = CurrentEvaluationCoordinate_meta(Σ)

OpenHere_Σ(v)
  iff WindowLive_Σ(v)
  ∧ AuthorityMatches_meta(v, Σ)
  where AuthorityMatches_meta(v, Σ)
          iff Anchor(v) = CurrentEvaluationCoordinate_meta(Σ)
```

The original spelling `RootOf(Anchor(v)) = NearestMetaRoot(Σ)` is the
simplified form of this unified rule under the meta transparent-navigation
quotient.

Meta invocation is naturally masking. If `M₀ └─ M₁` and the current context
is `M₁`, a value anchored on `M₀` satisfies:

```text
WindowLive_Σ(v) = true   -- window still open
Visible_Σ(v)    = false  -- not obtainable in M₁'s frame
OpenHere_Σ(v)   = false  -- AuthorityFrame_Σ(v) undefined: M₁ is the
                             nearest meta frame and does not own the
                             anchor; the resolution does not look past a
                             masking meta boundary
```

The value may persist in `M₀`'s suspended frame. It cannot be accessed or
passed as an argument in `M₁`. When the stack returns to `M₀`, the value
becomes visible and `OpenHere` again — this is **not** a reopen. True close is
the permanent, irreversible transition:

```text
WindowLive_Σ(v) := false   -- the only real close; never retracted
```

Nothing reopens closed material. `extend`/`inject` do not reopen it (§8.2), a
borrow view does not reopen it, and re-navigating to the same object from a
new context does not reopen it. Cloning/copying/transporting a closed-window
value carries its closed state with it: the clone is not a reopening.

For a **non-meta** context, authority is **not** a fixed function of the
stack-top callable. `AuthorityFrame_Σ(v)` is the nearest still-active frame
whose evaluation coordinate owns `Anchor(v)`, searched outward from the
current frame:

```text
AuthorityFrame_Σ(v)                        -- non-meta context
  = the nearest still-active frame f such that
      EvaluationCoordinate(f) = Anchor(v),
    searched outward from the current frame,
    skipping compile and transparent construction-intrinsic frames,
    and stopping at any meta invocation frame:
    a meta boundary between the current frame and f masks v and
    leaves AuthorityFrame_Σ(v) undefined

AuthorityMatches_nonmeta(v, Σ)
  iff AuthorityFrame_Σ(v) exists
```

The owning frame's coordinate contributes the `CallableRoot`, the
`MetaPartnerRoot` when the callable is generic (providing the stable symbolic
anchoring for the generic arguments), and that frame's own
`ActiveInlineClosurePath` — its navigation level within the in-place closure.
The path entering the comparison is always the owning frame's path, never
unconditionally the stack-top frame's path. Meta and non-meta resolutions are
different authority computations over the same `OpenHere_Σ` judgment, not
different notions of place capability.

Passing an open value into a deeper ordinary call frame therefore does not
destroy authority: the caller's frame remains still-active on the stack and
continues to own the anchor:

```text
F calls G (ordinary), v anchored at F:
  AuthorityFrame_Σ(v) = F's still-active frame while G executes
  OpenHere_Σ(v) holds inside G while the window is live
  -- G operates on v under the §12.1.2 disposition rules: at
     coordinates below the anchor the terminal actions are Reject,
     not Terminate
```

`AuthorityMatches` is therefore not an open ontology decision: it is the
coordinate equality between the value's static anchor and the owning frame's
evaluation coordinate.

The equality is opaque navigation-coordinate equality, **not** arbitrary
prefix matching. A prefix match would let an outer PatternValue automatically
acquire authority over every deeper inline closure, destroying the property
that non-meta in-place closure levels are opaque to authority.

The bare name `AuthorityMatches(v, Σ)` is the regime-dispatched form of the
same judgment: `AuthorityMatches_nonmeta` when the current context is non-meta,
`AuthorityMatches_meta` when the current context is meta.

The meta case is the same coordinate model under the transparent-navigation
quotient (above): `CurrentEvaluationCoordinate_meta(Σ) = ⟨NearestMetaRoot(Σ), ε⟩`.

The canonical principle is:

```text
PatternValue     records static PatternRoot + Navigation
evaluation stack records current dynamic evaluation position
PatternValue does not record dynamic call history
```

The `MetaPartnerRoot` answers where generic symbolic anchoring lives for a
generic callable `F`, and is required exactly when `F` is generic:

```text
Generic(F) => MetaPartnerRoot(F, GenericArgs)
```

It is **not** conditioned on whether `F` also has a `CompilePartner(F)`. The
compile partner `CompilePartner(F) = C(F)` (function-object-call-model §8)
answers how the compile-time realization of `F` is produced; the meta partner
`MetaPartner(F) = M(F)` (meta-object-invocation §4) answers at which level the
callable's generic symbolic identity is anchored. The two partners are
orthogonal: a runtime generic `F` has both `C(F)` and `M(F)`; a compile generic
`F` has no distinct compile partner but still has `M(F)`; a meta `F` has
neither. `CurrentAuthority(Σ)` therefore uses `MetaPartnerRoot(F, GenericArgs)`
for generic symbolic anchoring, independent of any `CompilePartner(F)`
consideration.

The required independence is explicit:

```text
Writable_Γ(q)            does not imply OpenHere_Σ(Read(q))
OpenHere_Σ(v)            does not imply Writable_Γ(Carrier(v))
Γ ⊢ r : type ref         does not imply OpenHere_Σ(Read(r))
WindowLive_Σ(v)          does not imply Visible_Σ(v)
Visible_Σ(v)             does not imply OpenHere_Σ(v)
```

The state transition of the open window is one-way:

```text
WindowLive_Σ(v) := false   -- irreversible
```

Nothing reopens closed material. `extend`/`inject` do not reopen it (§8.2), a
borrow view does not reopen it, and re-navigating to the same object from a
new context does not reopen it.

#### 12.1.2 GenerationRegime and open dispositions

Every `PatternValue` carries a small immutable horizontal attribute:

```text
GenerationRegime(v) ∈ { MetaGenerated, NonMetaGenerated }
```

`GenerationRegime(v)` is **not** part of the Object structure
`Object = ⟨Val1?, P, Val2⟩`, is not part of `Norm(v)`, and does not
participate in canonical Pattern identity or τ normalization. It is an
implementation attribute used only to decide how the open window may be
closed.

Although `GenerationRegime` and `Anchor` are defined on
ordinary `PatternValue`s, and `WindowLive_Σ` is defined on the evaluation
state, `extend` operates on the complete type value
`τ = <Q, V_τ>`, which is not itself an ordinary `PatternValue`. The bridge is
by Core projection, consistent with the minimal-change observation rule (§2.2:
ordinary type-rank equality observes `Core(τ) = Q`):

```text
GenerationRegime(τ) := GenerationRegime(Core(τ))
WindowLive_Σ(τ)   := WindowLive_Σ(Core(τ))
Anchor(τ)           := Anchor(Core(τ))

OpenHere_Σ(τ)
  = WindowLive_Σ(τ) ∧ AuthorityMatches(τ, Σ)
  = OpenHere_Σ(Core(τ))
  -- AuthorityMatches as defined in §12.1.1: per-value authority-frame
  -- resolution (non-meta: AuthorityFrame_Σ exists;
  -- meta: coordinate equality against CurrentEvaluationCoordinate_meta)
```

`GenerationRegime(τ)` does not participate in `WellFormedTau(τ)` or in Pattern
identity; it is consulted only by the contextual capability rules above. The horizontal attributes of a complete type value are those of its core
PatternValue. The notation subject(t) used by [A](associated-compile-state.md)
refers to this existing construction/window state subject, not a new language
Object. Its stable identity is preserved by copy and authorized continuation
of that construction, including in-place updates; independent equal-Core
formation must not collapse it. This designated identity observation leaves
ordinary Core equality unchanged.

- **MetaGenerated.** A value produced inside a meta body has no birthright
  global lifetime. It can be used freely within the same meta computation, and
  it may be promoted into a stable result only when the MetaInstance seals and
  owns/copies the material it owns. The original local value is not magically
  prolonged: persistence happens by promoting the MetaInstance's stable value,
  never by extending the local value's lifetime.

- **NonMetaGenerated.** A value produced in an ordinary (non-meta) construction
  context is born globally survivable with a live open window:
  `GlobalSurvivable(v) ∧ WindowLive_Σ(v)` hold from creation. Its open window
  is a linear evaluation flow, not a flat event list. The disposition of an
  action on `v` is one of three outcomes:

```text
OpenDisposition_κ(p, action, Σ)
  ∈ { Continue, Terminate, Reject }
```

The owning in-place closure's evaluation segment is only the natural upper
bound of the open window; the window may end earlier. In particular:

```text
EffectiveOpenSegment(p)
  ⊆ OwningInlineClosureEvaluationSegment(p)
```

At the value's own outermost open coordinate
(`CurrentCoordinate = OpenRootCoordinate(p)`), the legal terminal actions end
the open window (`Terminate`); they are not forbidden, but they close the
window:

```text
CurrentCoordinate = OpenRootCoordinate(p)      -- outermost open coordinate

UseForVal1(p)        ->  Terminate   -- legal action; ends the open window
UseAsMetaArgument(p) ->  Terminate   -- legal action; ends the open window
ControlFlowSplit(p) / ControlFlowMerge(p)
  at generation level                  ->  Terminate
  -- the window requires a single, non-forking, non-merging linear
     evaluation stream; a static join/loop-carried state or a
     residual-runtime fork at the generation level violates that
     requirement exactly at that point
```

Inside an opaque non-meta inline closure (the evaluation has already moved
below the value's own open coordinate), `UseForVal1` and `UseAsMetaArgument`
are **forbidden** (`Reject`) at any depth, because performing the construction
effect would already have crossed the value's legal linear open flow.
`ControlFlowSplit` / `ControlFlowMerge` are **generation-coordinate** events:
they terminate the window only at the value's own generation level, and at a
deeper ordinary coordinate they are **irrelevant to the outer window** —
neither `Reject` nor `Terminate`:

```text
CurrentCoordinate ≻opaque OpenRootCoordinate(p)
  -- evaluation is inside an opaque non-meta inline closure below the
     PatternValue's own open coordinate

UseForVal1(p)        ->  Reject
UseAsMetaArgument(p) ->  Reject
ControlFlowSplit(p) / ControlFlowMerge(p)
  at the generation coordinate    ->  Terminate
  at a deeper ordinary coordinate ->  Continue (irrelevant to outer window)
```

The judgment reversal therefore applies only to `UseForVal1` and
`UseAsMetaArgument`: at the outermost coordinate the action is a legal
terminal action; in a nested opaque non-meta level the same action is a
forbidden one. It cannot be explained as "first allow `UseForVal1`, then
close": by the time the construction effect happens, the value's legal linear
open flow has already been crossed. Control-flow split/merge are not "close
after the fact" events: they are scoped to the value's own generation
coordinate, so a split or merge inside a deeper ordinary frame does not reach
back and close an open value generated at an outer level.

`UseForVal1` and `UseAsMetaArgument` reject/terminate independent of
call-frame depth at the relevant coordinate: a meta boundary cannot be
escaped by performing the meta call inside a deeper ordinary frame, and
installing the value as `Val1` is likewise unconditional. `ControlFlowMerge`
and `ControlFlowSplit` apply only at the value's own generation level; a merge
or split inside a deeper ordinary call frame does not reach back and close an
open value generated at an outer level. Passing the value into a deeper
ordinary call frame does **not** itself end or forbid the window. Likewise, a
value's visibility (`Visible_Σ`) may be lost because of stack masking without
its open window being touched.

In an ordinary, non-meta construction context the concrete dispositions are:

```text
UseForVal1(x)                                    -> Terminate at OpenRootCoordinate(x)
                                                     Reject inside an opaque non-meta
                                                     inline closure below it
x used as a meta argument                        -> Terminate / Reject (same rule)
x entering a global normalized structure         -> Terminate (at OpenRootCoordinate)
x in Dependencies(c), for NonMetaStaticControl(c) -> Terminate
                                                     (at generation level)
x in LiveAcross(c), for ResidualRuntimeFork(c)    -> Terminate
                                                     (at generation level)
leaving the construction interval of the
  in-place closure that owns x                   -> Terminate
                                                     (owner's interval)
```

Observation is not a terminating action: reading `P` or `Val2`, extending a
child pattern, and contributing an ordinary Val2 member of another type all
leave the material open (`Continue`).

For static control, dependency and liveness are different facts:

```text
Dependencies(c) != LiveAcross(c)

NonMetaStaticControl(c)
  => OpenDisposition_κ(d, UseInControlFlow, Σ) = Terminate
     for each d ∈ Dependencies(c) at the generation level
```

`Dependencies(c)` contains the open Pattern values actually read by the
predicate or structural selection, branch/iteration versions whose identity
must be unified at a join, and loop-carried construction state that feeds a
later static decision. A value that is merely live across an unrelated static
branch, join, or loop is not terminated. In contrast, a residual-runtime fork
loses the single known static construction path, so open values carried across
that fork are terminated even when they did not determine its predicate.
Leaving the ordinary owner interval remains an independent terminating
disposition.

#### 12.1.3 Meta construction is transparent but meta-local lifetime is not global

The open dispositions of §12.1.2 are scoped to `NonMetaGenerated` values.
Inside a meta body, material is `MetaGenerated`, and the same actions do
**not** terminate its open window, because the construction anchor is the
meta instance itself (§4.3.1). Meta navigation is transparent for authority:
`ActiveInlineClosurePath_meta` is quotient/erased (`VisibleInlinePath_meta(path)
= ε`), so meta evaluation never produces the opaque nested state that triggers
`Reject` for non-meta inline closures. The meta space is governed by
`NearestMetaRoot`, `MetaArgumentAdmissible`, `GlobalSurvivable`,
`NoOpenEvaluationReentry`, and seal/promotion rules instead:

```text
inside M (MetaGenerated material):
  UseForVal1(x)                     Continue -- does not end the window
  using x as a meta argument        Continue -- presupposes meta argument
                                      admissibility (§4.3.1–§4.3.3):
                                      MetaArgumentAdmissible(a) =>
                                        GlobalSurvivable(a), and a
                                        non-GlobalSurvivable MetaGenerated
                                        local cannot enter another meta
                                        invocation at all
  entering global-normalization     Continue -- does not end the window
  static control flow               Continue -- does not end the window
  entering an in-place closure of M Continue -- transparent navigation;
                                      ActiveInlineClosurePath_meta is erased
```

The only capability-ending event for material owned by the meta construction
is its return-stage seal (§4.3.2). A fresh meta-local PatternValue nevertheless
has `Life = MetaInvocation(M)`. Attempting to pass it to another ordinary meta
does not close or promote it; candidate formation rejects the call when the
canonical argument is not `GlobalKeyable` (§4.3.1–§4.3.3). The rejection is
total: the argument never enters the deeper invocation, so meta invocations
cannot smuggle meta-local open material into the closed world and re-open it
when the stack unwinds. `compile` and
transparent construction intrinsics may consume it because they create no new
MetaInstance key.

At seal, only `OwnedResultClosure(τ)` is promoted: for the default result `τ_M`
that is `OwnedClosure(Core(τ_M))` plus `OwnedCallSpaceClosure(CallSpace(τ_M))`
(§4.3.2); an explicitly explicit group result promotes the carried `τ`'s owned
result closure only when that `τ` is present. Other local
PatternValues expire with the invocation. Consequently the open-disposition rule for
`UseForVal1` (§12.1.2) must not be read as a universal invariant, while “meta body is
transparent” must not be read as implicit global promotion.

#### 12.1.4 The apparent self-typed intersection

With §12.1.2 in force, the ordinary case that looked like an intersection
resolves without a special rule. Suppose an RHS is an ordinary value-bearing
Object whose Pattern core is the `Q` of the type closure being extended, and the
extension is attempted from an ordinary context:

```text
construct RHS value of target type
  -> UseForVal1(target) at OpenRootCoordinate(target)
  -> Terminate -- legal action, but it ends target's open window
  -> target is no longer OpenHere_Σ
  -> attempt to extend target
  -> no applicable overload
```

So in an ordinary context there is no legal situation in which one operation both
extends the target's Pattern and contributes a complete self-typed val to the same
still-open target. A complete value of some *other* type may still be contributed
as ordinary Val2 while the target is open; its own Pattern and Val2 remain
attached to that value.

In a meta body the same sequence is simply legal: the first step is
`Continue` (the material is `MetaGenerated`), so the open window survives the
`UseForVal1` and the subsequent extension is admissible.

A direct-call entry belongs to the callee's exact complete type:

    Type(callee) = Type(first self)
    callee : T       -> matching () in complete type T
    callee : T ref   -> matching () in complete type T ref
    callee : T share -> matching () in complete type T share

Constructing T's associated () does not install entries into the complete types
T ref or T share. A differing formal object Pattern cannot adapt one of those
callees into another receiver type. Their own authorized construction must
supply their exact callspaces. Each entry body has a CallableOwner; that code
owner does not change the exact type of invocation slot 0.

Ordinary field forwarding is separate: its anonymous function object occupies
first self and the operated T ref/T share value is a later argument. The
forwarding callable may perform its ordinary checked forwarding to T; this
does not add a receiver adaptation rule to direct () invocation.

Under equal owner/construction authority, an inner contribution and a later
inner-to-outer navigated declaration denote the same pending namespace delta:

```text
struct-local contribution under owner name1::T
  ==
later installation at name::name1::T
```

Neither spelling forwards a place or reroots the initializer's Pattern.

The language must select the expectation from semantic context or an explicit
rank/facet annotation. It must not guess `PatternChild` merely because the
right side happens to carry a type or `PatternValue`. Both paths still obey the
general name-resolution-then-facet-projection rule.

### 12.2 Same-name contribution

A structural name denotes a complete type. Named-contribution positions
synthesize its V_tau under OpenHere and anchored-membership rules. Type +=
and -= change only V_tau; Core changes use extend/inject. Explicit groups
aggregate candidate types without mutating those types. Contributions from sibling source blocks are governed
by unordered join, not by physical-file exclusivity.

Explicit structural let still requires a fresh name. Ordinary lexical rebinding
does not aggregate merely because two declarations have the same spelling.
Ordinary group addition does not create an absent name. A complete pattern
member retains its own Core, callspace and well-formedness judgments.

Sum construction, Pattern normalization and structural child uniqueness remain
their existing operations. They are not replaced by group aggregation, and
their equality laws do not imply that equal-valued group entries merge.

### 12.3 Value identity does not multiply with names

Do not infer three type values from:

```lang
let Bool = value;
let bool = value;
let t = bool;
```

If the bindings expose the same pattern/type value, the value identity is the
same. Their `NameBindingId` and `PlaceId` nevertheless remain distinct and separately
observable; provenance is diagnostic material and is not part of the value's
normal form.

### 12.4 Installation is always outer-layer work

Invocation produces its declared semantic value under the ordinary root,
normalization and escape rules. An outer lexical binding or named contribution
carries that value. Structural let first creates its fresh name and returns a
mutable construction reference; subsequent assignment has ordinary write
semantics. inject writes through an existing reference.

Namespace indices reflect committed semantic actions. A storage transaction
may implement an enclosing semantic transaction but cannot invent special
atomic initialization or rollback for structural let. struct and pure extend
do not themselves install destination names.

Future compile-to-runtime materialization preserves the same separation:

```text
materialization_place(result)
pattern_owner(result)
```

The first may be a newly allocated runtime owner/place or compiler-generated
`[[global]]` storage. It does not imply that the result Pattern is rerooted to
that place. Pattern owner/root/scope continue to come from ordinary result
construction semantics. Likewise, generated storage placement is not
source-visible `NamespaceGraph` binding installation.

## 13. Non-Goals and Open Representation Boundaries

This document does not change Raw or Normalized AST syntax, introduce a general
macro system, expose unrestricted AST rewriting, or choose the final storage
representation for Pattern space, owner persistence, access trees, or
continuations.

The following semantic distinctions are fixed even while those representations
remain open:

```text
complete type != name binding != Place
construction material != semantic result
ordinary member != TypeMember != structural field
extend = pure transformation
inject = read + extend + write
OpenHere != Writable != PolicyMode
```
