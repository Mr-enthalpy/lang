# Static, Name, and Structural Semantics

**Status: canonical ontology owner.**

This document defines the canonical relations used by static evaluation, name
production, structural interpretation, type formation, and dot navigation.
Topic owners specialize these relations while preserving their identities and
composition laws.

## 1. One evaluator and three orthogonal coordinates

All source computation uses the same evaluator `E`. A static call is identified
by its evaluation horizon, instance identity, and independent completion policy:

```text
Stage = {compile, runtime}
OpenPolicy = {open, close}
PolicyMode = {const, mut}
```

`P2` owns evaluation horizon. `OpenPolicy` is a P1 completion coordinate.
`PolicyMode` is the value-mode coordinate. None implies either of the others.

For an omitted mode:

```text
DefaultMode_K(x) =
    mut    if OpenPolicy_K(x) = open and OpenHere_K(x)
    const  otherwise
```

If an `open` completion requires an `OpenHere` fact that is not yet available,
completion is unavailable; unknown is not a proof of the `const` branch.

`open != mut` and `close != const`.

## 2. Compile partners and compile instances

Every compile call uses one `CompilePartner` and forms one stable
`CompileInstance` before evaluating its selected body:

```text
CompileInstanceKey
  = ParentSemanticOwner
  x SelectedCompilePartner
  x CanonicalizeInvocationInputs(In)

I = CompileInstance(CompileInstanceKey)
n_I = InvokeName(I)
```

The instance identity is independent of its result value. A compile call may
return an integer, Product, closure, existing type, fresh self-rooted type, or
another ordinary Object without changing the instance-formation rule.

The instance self-name is an ordinary legal NameValue. Observing it is not
invocation reentry:

```text
Read_name(n_I) != EnterBody(I)
Read_resident(n_I) != EnterBody(I)
```

Actual reentry exists only when the next evaluation edge enters the selected
body again.

In-place closures retain their ordinary lexical, Self, navigation, dependency,
and lifetime boundaries. No closure becomes transparent to instance ownership.
Code that needs an enclosing compile instance uses the instance's ordinary
self-name and admitted dependencies.

## 3. Compile result structure determines expression kind

Whether a completed expression is a NameExpr is determined by the structure of
its result, not merely by the fact that it was evaluated at compile horizon.

Let:

```text
Reach(v) = ordinary accessible closure of v
SelfRootSet_I(v)
  = { tau in Reach(v)
      | Type(tau) = type
        and Root(tau) = I
        and OpeningSource(tau) = I }
```

A result that exposes a type still governed by the current instance must expose
that root directly as one complete type:

```text
SelfRootSet_I(v) != {}
=>
Type(v) = type
and v = tau_I
and Root(v) = I
and SelfRootSet_I(v) = {v}
```

Such a result is a computed NameExpr:

```text
Read_name(result) = n_I
Read_resident(n_I) = tau_I
```

A compile expression returning an existing type whose root is not `I` is not
made into a new NameExpr merely because its result has type `type`.

This single-self-root law prevents an instance-open type from escaping hidden
inside a Product, closure capture, reference wrapper, or other accessible
result shape.

## 4. Interpretation polarity and arbitrary `$` nesting

There are two interpretation polarities:

```text
V = ordinary RHS/value interpretation
S = structural interpretation
```

Structural consumers include Path/navigation completion, extraction structure,
and structural type formation. Define:

```text
Flip(V) = S
Flip(S) = V
Flip(Flip(C)) = C
```

The only general law of postfix `$` is:

```text
Interpret(e$, C) = Interpret(e, Flip(C))
```

Therefore `$` may nest to any source depth:

```text
Interpret(e $$...$, C) = Interpret(e, Flip^n(C))
Flip^(2k)(C)   = C
Flip^(2k+1)(C) = Flip(C)
```

`$` does not read a resident, change Stage, grant OpenHere, change PolicyMode,
create a borrow, or create a lifetime fact.

Resident reading remains independent:

```text
ReadVal(n) = Read_resident(Read_name(n))
ReadVal(e) != e$
```

Bare-name structural completion is likewise independent:

```text
BareName(a; p) = a::p
```

The same navigation-completion law is consumed by Path and extraction-side
structural interpretation. `a$` instead interprets `a` under the opposite
polarity before the surrounding structural consumer uses its result.

A canonical nested example is:

```text
((bool a)(args Mytypefun)$)$
```

When the whole expression occurs in ordinary RHS context, the outer `$`
enters structural interpretation and the inner `$` returns
`args Mytypefun` to ordinary value interpretation. The enclosing evaluation horizon is preserved.

## 5. Pattern is structural registration over Val2

Structural identity is represented by registrations over actual members in
`Val2`.

```text
StructuralRole_P(x) => x in Val2
x in Val2 !=> StructuralRole_P(x)
```

The minimum registered structure includes the members needed to witness the
type's construction/extraction identity and the applicable role relations, such
as:

```text
DirectPatternChild
ConstructEdge
ExtractEdge
FieldView
navigation/incidence roles
LeafSource where applicable
```

Ordinary helper members may exist in `Val2` without participating in those
roles.

Construction and consuming extraction remain atomic complete-object actions
after Pre.

## 6. Structural interpretation forms complete types directly

A structural expression observed through `$` in ordinary RHS context yields a
complete type value directly:

```text
((bool inner)a)$ => tau_a
```

The formation transaction establishes the minimum Val2 witnesses and their
structural registrations together, producing the complete type as its result.

`struct` is an ordinary type-to-type function:

```text
struct : type -> type
T struct = T |> struct
```

It adds the standard ergonomic/helper interface that is not required to define
the input type's structural identity. For `T' = struct(T)`:

```text
StructuralRegistration(T') = StructuralRegistration(T)

Val2(T') may contain additional ordinary helpers
V_(T') may contain additional associated helpers
```

Typical additions include ref/share observation families, their ordinary policy
cells, field-write/setter families, and other mechanically derived associated
helpers.

Type composition and place update are ordinary operators:

```text
*  : type x type -> type
*= : type ref x type -> unit

T * D
r *= D
```

For `r *= D`:

```text
T  = Read(r)
T' = T * D
Write(r, T')
```

The selected action still performs the ordinary Pre checks for OpenHere,
Writable, capability, access, lifecycle, and continuation legality.

## 7. File hierarchy lowers to name formation

A source hierarchy changes the inherited navigation context. It does not imply
structural type composition.

```text
a/
    P let inner = rhs;
```

lowers at the semantic handoff to:

```text
P let inner::a = rhs;
```

For example:

```text
a/
    let inner = bool::;
```

is:

```text
let inner::a = bool::;
```

`inner::a` and `bool::` are NameExpr structures at the same semantic level.
The hierarchy therefore reuses ordinary navigation completion and name
realization rather than type formation.

## 8. Requested-name producers are general compile computations

A generative declaration receives a requested NameValue and evaluates an
ordinary compile body:

```text
q = RequestedName
I = CompileInstance(producer, q, CanonicalInputs)
E_I(rhs) => v
Realize(q, v)
```

Neither of these is required:

```text
Type(v) = type
v is a NameExpr
```

Name production and NameExpr-valued results are orthogonal:

```text
ProducesName(q, v) !=> NameExpr(v)
NameExpr(v) !=> ProducesName(q, v)
```

This is the common basis for generated values, callables, members, types, and
other ordinary residents.

## 9. Dot is ordinary ADL over a generative overload family

Dot syntax enters one ordinary `adl` name family. Its canonical source
definition is:

```text
adl/
    let <field> (self, object:t, ...args) field
        => ((object, args) |> ((field#)[0])$::t);

    let <field> (self, t:type) field
        => ((field#)[0])$::t;
```

The first candidate implements ordinary receiver calls. The second implements
type-path progression. Existing overload applicability and specificity choose
between them.

Thus:

```text
object.field(args)
=> object |> .field(args)
=> (object, args) |> field::adl
```

and for a type-valued receiver:

```text
T.field => field::T
```

Repeated type-path progression provides left-to-right surface navigation while
the canonical Path representation remains `name::path`:

```text
foo.bar.baz =_Path baz::bar::foo
```

Dot syntax supplies the ordinary `field::adl` requested name and receiver
material; subsequent authority and behavior come from the selected candidate.

## 10. Double-dot is only pipeline-dot contraction

Double-dot has one surface law:

```text
express..field(args)
=
express |> .field(args)
```

After this rewrite, the ordinary dot/ADL rules apply.

## 11. Static self-modification and generated design patterns

A compile expression may retain its instance opening with:

```text
OpenPolicy(I) = open
OpenHere_K(I)
```

For an omitted mode this yields the ordinary `mut` observation described in
section 1. The expression can then use the normal type/reference/operator
algebra to update its current value. No extraction of a hidden auxiliary
payload is required.

Trait-like and automatically generated design patterns are ordinary compile
programs assembled from requested-name production, overload applicability,
self-name observation, `open/close`, OpenHere, and ordinary type operations.

## 12. Canonical consistency

The active tree uses one canonical name and relation for each semantic concept.
Source, normalized interfaces, semantic carriers, diagnostics, tests, fixtures,
contracts, planning documents, public guides, glossary, and agent instructions
use the vocabulary and equations defined by their topic owners. Source
conveniences lower to the lowest existing relation that explains them.

## 13. Adjacent-universe joint extraction

For every adjacent universe rank, a simple binding consumes one relational
extraction judgment. Let `Gamma;K |- E(expr) => v` and `Type(v)=T`:

```text
R_Gamma(a:b, v, rho)
  => rho(a)=v and rho(b)=T
     Rank(rho(b))=Rank(rho(a))+1
```

Bare `b` at this extraction site introduces the classifier-side binder. An
existing external Pattern enters explicitly through a full Path, such as
`uint8::`, or opposite-polarity `R$`. Deduction, an external Pattern constraint,
and a classifier Hole all use `R_Gamma`; Hole presence does not change the
binding judgment into a result-transforming call. Structural qualified
`P let name::path:t` is the name-formation judgment of the names owner.
Initializer-free `let a:uint8::` can form a typed name and uninitialized Place
when that judgment's external-type premises hold. It creates no resident.

```text
Interpret(_, S)=AnonymousPattern
Interpret(_, V)=universerank
```

`universerank` is ordinary universe-rank material. Anonymous Pattern, explicit
binder absence `<>`, and root-qualified Deduce Holes have distinct identities.
Polarity changes neither the universe rank nor resident availability by itself.

## 14. Hole-local Policy

```text
P let <P_h let h> LHS = RHS
Policy(h)=HoleOverride(P,Delta_h)
HoleOverride(P,Delta)[d] = Delta[d] if d in WrittenDims(Delta)
                         P[d]     otherwise
```

An omitted `P_h` inherits all outer dimensions. An explicit dimension replaces
that dimension without an outer-strength restriction. Pattern constraints and
Policy constraints of each Hole are solved by the ordinary relation under its
`PatternRoot/HoleBinderId`; the local Policy operation is independent of
`ElabIn` and `ElabOut`. In `<p> let <p let h> ...`, the inner Policy occurrence
references the preceding exact Hole identity of `p`.

## 15. Bare branches and ordinary bool values

```text
Bare(s)=<nu_s,P(s),Val2(s)>
UniqueValueMember(Bare(s),nu_s)
```

The member exists. Its machine representation is unobservable; `nu_s` fixes
branch identity through ordinary Pattern relations, without a public integer
encoding or synthesized extraction field. A compile Pattern with a runtime
value observation is a legal view of this Object.

```lang
let bool = ((true | false)bool)$;
let true = true::bool;
let false = false::bool;
let if = true::bool;
let else = false::bool;

|> name { B }
// syntax-directed expansion:
|> (_, let _: (name)$) { B }
```

The two branch-head `_` positions are structural anonymous Patterns. The
annotation's complete `(name)$` operand enters V interpretation and supplies
the existing value Pattern to extraction. `if` and `else` are ordinary value
bindings of the true/false branches.

## 16. Fully specified Path and structural navigation

The [Path owner](symbol-world/structured-path-algebra-and-interpretation-polarity.md)
defines full navigation and structural suffix overlap over actual Path values.
`Relative::Relative` and `Relative::Absolute` retain their composition;
`Absolute::Relative` is undefined; `Absolute::Absolute` requires a matching
complete rooted suffix. Pattern navigation consumes the same completion law,
adds extraction and explicitly full-navigation module resets:

```text
Path algebra strictly-contained-in Pattern algebra
let a p1 p2 p3
  => let a (p1::p2::p3) (p2::p3) (p3::)
```

Normalized navigation retains actual members, full navigation and extraction
relations. Syntactic inner/outer provenance supplies no additional identity.
`foo.bar.baz =_Path baz::bar::foo` requires a type-Path candidate selected at
each step. An object method can return an unrelated ordinary Object.

## 17. General open computation and struct helpers

For a legal compile open call under ordinary by-value passing:

```text
OpenHere_K(type_val) iff OpenHere_K(type_val compile_open_call)
```

This is subject-preserving observation of the same concrete type value;
Writable, access, dependency and lifetime premises remain independent. Open
computation supports name/member generation and ordinary Policy migration.
`(T ref) |> (type ref)` uses ordinary type/value invocation and reference Pre.

`struct` receives a complete structural type and returns a complete type with
ordinary additional helpers. Its legal ordinary open computation can use
`OpenHere`, `|[[attribution]]>`, `type ref`, and `*`/`*=` or member registration
to form forwarding members on derived reference types. Reference/borrow and
lifecycle primitives belong to ref/share; field forwarding requires these
explicit helper registrations. StructuralRegistration is preserved.

```text
DerivedState(T0) depends on the concrete T0 snapshot
Write(place,T1) does not retarget DerivedState(T0)
```

Observing final members of `T1` requires a new ordinary formation/call on `T1`.

## 18. Dependencies and opaque self

Compile callables may have ordinary implicit dependencies. Formation records
requirements and selected realizations; readiness separately asks whether the
current action needs a concrete value that is available. Runtime materialization
can supply such a value to a delayed continuation. Dependency validity and
instance reuse retain all result-affecting identities and observations.

```text
Norm_Name(n) is independent of structural unfolding of self
```

`self` is observable through normalized Pattern algebra without a public capture
layout. Its compile-flow observation uses the current CompileInstance, and its
runtime-flow observation uses the current callable object. The same call,
Policy, dependency and lifecycle relations govern both observations.

## 19. Callable tag and activation

```text
ReturnTarget = current callable object tag
CallType(tau): CallTarget(I)=tau, ActualSelf(I)=c* in V_tau
Tag(closure expression)=tau_C
Type(tau_C)=type; Type(type)=(1 type)
```

Ordinary value invocation retains its normal implicit self. Return routing
observes the callable tag and the actual current control-flow activation;
recursive activations of the same tag have different frame identities. Tag
observation supplies no candidate preference and unfolds no private self layout.
ReturnEvent, ReturnPattern/Pout and internal completion retain their existing
delivery and no-reopen boundaries.

## 20. Runtime reification of the same E

```text
(C,R) |> reify => f
Callable(C) and ClosedReturnContract(R)
Invoke(f,args) ~= Deliver_R(E(Invoke(C,args)))
Deliver_R(v) = ordinary return-boundary delivery of (v |> R)
```

The equivalence requires InputAdmissible, actual evaluation-frontier facts,
dependency validity, ordinary Pre/commit/Post and a complete result-type
contract determined by R on every normal return. `f` is an ordinary callable
value admitting legal runtime-policy arguments. It materializes the relevant
compile computation and its actual remaining continuation; E resumes when
concrete runtime inputs become available.

```lang
let r_vals:R$ = runtime_vals |> ((closure,R) |> reify);
```

The external contract is `R$`. Bare R is classifier-side extraction material.
An already selected invocation retains its candidate/projection/frame. A call
newly formed by runtime E first selects at its actual frontier by ordinary
overload rules. AOT, JIT, caching and precomputed dispatch tables are equivalent
O choices; exhaustive Cartesian-product enumeration is not an E premise.

## 21. Continuation-local Region

```text
Region_K(n)=R(NamePosition_K(n))
```

The current semantic name position, not future branch availability or physical
execution time, determines Region. E uses current continuation Pre/commit/Post
and fixed cleanup observations. It does not require future-path enumeration.
`compile let val = runtime_express` may remain a delayed continuation when its
required value is not yet available. Reification resumes the same E with actual
inputs at args-in; ordinary origin relations stay within semantic name-position
algebra. Runtime strings supply no authority to redefine fixed structured names.
Future-path proof, specialization and layout remain O work with E revalidation.
