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
Stage = {compile, seal, runtime}
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
