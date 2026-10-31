# Product, Repetition, and Contextual Unpacking

**Status: canonical Product and repetition owner.**

This owner defines repetition packaging, its ordinary Object identity, bare
Product expansion and the repetition family's `?` extraction. It consumes
[interpretation polarity](../unified-static-name-and-structural-semantics.md#4-interpretation-polarity-and-arbitrary--nesting),
[R_Gamma](pattern-relations-observation-and-extraction.md),
[Object normalization](../symbol-world/type-values-places-and-borrow-views.md#21-object-identity-is-the-recursive-three-component-normal-form)
and ordinary Policy-driven overload selection. Parser/Norm P/E/G shapes are
syntax material; their preservation does not execute these relations.

## 1. Repetition and the standard wrapper

`t * n` is ordinary repetition algebra: `t` constrains each element and `n`
specifies cardinality. A finite `n` fixes that many positions. `ω` admits a
finite current collection of any length; it does not construct infinitely many
Objects. A declared shared Hole in `t` retains its identity across positions;
anonymous structural `_` supplies independent deduction at each occurrence.

```text
product = (_ * ω)$
classifier-side Product constraint: a : _ * ω
```

Here the RHS `$` interprets its complete operand structurally and forms the
complete repetition type. The classifier constraint uses the repetition
Pattern directly. A bare classifier name in simple `let a:b` remains a new
extraction binder; existing type Paths and opposite-context material keep
their ordinary meanings.

`product` names this standard wrapper. Its heterogeneous elements follow from
the independent anonymous positions in `_ * ω`, rather than a separate
primitive Product classifier. The ordinary `*` family also contains type
composition candidates; actual operands and the ordinary relation distinguish
repetition from `type × type -> type`. Spelling alone grants neither operation.

## 2. Bare Product and context

Let `B_K(xs)` denote the bare Product normal form at an established structural
layer. It is the `P` material in the P/E/G source skeleton. It retains the
actual entries, labels, grouping boundaries and unit positions.

```text
G = (E)
G -> E                       in expression context
G -> P                       when a source Product is required
Expose_K(_ * ω material) -> B_K(xs)
Merge_K(B_K(xs), B_K(ys)) = B_K(xs ++ ys)
```

`Expose_K` and `Merge_K` describe contextual normalization of exposed material,
not globally callable primitives. The established consumer determines which
standard packaging is exposed. In particular, call input is a Product context:

```text
CallInput_K((a,(b,c))) = B_K(a,b,c)
CallInput_K(((a,b),c)) = B_K(a,b,c)
```

These equations require the displayed standard `_ * ω` material to be actually
available. Normalized syntax retains the nested source Products; call-input
observation recursively exposes their standard packaging and merges the exposed
entries. An ordinary expression barrier remains one expression until its result
and applicable exposure are available. A non-standard structural layer needs its
own admitted extraction; an unavailable runtime payload supplies no static shape.
Unit is an entry and is not erased. Source-product continuation still merges
only its first following Product; that source binding rule and subsequent
contextual exposure are separate observations.

The standard `_ * ω` packaging is automatically exposed and merged where the
documented Product context requires it and the actual material is available.
This is the same repetition algebra used by explicit unpacking. It grants no
static shape observation of an unavailable runtime collection.

At every preserved layer:

```text
all direct entries named => unordered
any bare direct entry    => the whole layer ordered
```

Nested layers decide independently. Named extraction followed by explicit
ordered assembly is required to turn an unordered layer into a bare sequence;
neither map order nor `?` supplies that sequence.

## 3. Underscore and the singleton form

```text
Interpret(_, S) = AnonymousPattern
Interpret(_, V) = universerank
Interpret(_$, S) = Interpret(_, V) = universerank
Interpret((_)$, S) = universerank
```

The anonymous structural position is a deduction/matching hole with no named
`HoleBinderId`. `<>` binder absence and an explicitly shared Deduce Hole remain
different materials. In structural interpretation `_$` supplies an external
Pattern for extraction, alongside `bool::` and `(if)$`; it is not an anonymous
discard or unconstrained match.

The repetition singleton and ordinary group interpretation give the contextual
form:

```text
_$ =_context (_ * 1)$
```

The equation is consumed under the same V/S polarity and singleton
expansion rules. It is not an assertion that an arbitrary packed Object and
its exposed contents have identical three-component normal forms. `$` still
only flips polarity; it inserts no resident read, rank lift, Policy migration
or lifecycle action.

## 4. Relocation into Val1 and complete identity

Repetition can place element material that would otherwise contribute directly
to a structural Pattern into the value coordinate. Write `Packed(t,n;xs)` as
metanotation for the resulting ordinary Object:

```text
FormationPre_K(t,n,xs) and R_Gamma(t * n, xs, rho)
and CardinalityOK(n,xs)
  => Packed(t,n;xs)
     = <B_K(xs), P_(t*n), AssociatedVal2(t*n)>

Norm(Packed(t,n;xs))
  = <Norm(B_K(xs)), Norm_P(P_(t*n)),
     Norm_Val2(AssociatedVal2(t*n))>
```

This describes ordinary complete formation after Pre, not an incremental or
partial Object. Structural role registrations require actual Val2 witnesses;
the `Val1` payload supplies no additional lookup authority by itself.

During compile formation the relocated material is observed as compile Val1,
together with its Pattern. It remains in identity through `Norm_Val1`.
Moving it into `Val1` does not hide it from the compiler. If `xs` and `ys`
have different normalized content, the packed Objects differ even when their
Pattern and associated members agree. Legal replacement of element values
can preserve `P_(t*n)` while changing Val1 and complete Object identity.
Writable, capabilities, access, generation and lifecycle Pre remain required;
value mutability does not require a Pattern mutation.

For finite `n`, cardinality remains a constraint of `P_(t*n)`. For `ω`, the
current finite length is part of the payload observation, not a new fixed
cardinality Pattern. Presence of Val1 alone implies no Stage or Ready fact.
Later runtime observations require their ordinary Policy/migration relation;
the relocation itself grants no stage transition.

## 5. Explicit unpacking and Policy

The repetition family's selected ordinary postfix `?` operation exposes one
complete packed layer as bare Product:

```text
Ready_K(pack) and SelectedQuestion_K(pack)=RepeatUnpack
and AvailableShape_K(pack)=xs and LegalPre_K(pack?)
  => E_K(pack?) = B_K(xs)

E_K(B_K(xs)?) = B_K(xs)       -- already exposed normal form
```

`RepeatUnpack` is metanotation for the selected family action, not another
primitive or evaluator. Consuming decomposition obeys the existing atomic
ExtractCommit and ordinary Move/lifecycle rules. Observation alone proves no
parent death. General named one-layer views retain the
[result-extraction owner's](return-value-extraction-and-implicit-decomposition.md)
rules. Matching reads the supplied Pattern directly and does not insert `?`
to repair failure; equality likewise inserts no extraction.

```text
compile observation of t * ω + available finite payload
  => ordinary ? can expose that finite bare Product

runtime-only observation of t * ω
  !=> static bare Product expansion
```

Runtime `t * ω` requires an ordinary iteration-like consumer to process its
elements; `?` cannot obtain a variable static call arity by inspecting runtime
length. A finite fixed-cardinality family may provide an ordinary runtime
overload with its established shape. Applicability and Policy select these
behaviors for the same value; selection failure never reopens a candidate set.

Obtaining a compile observation from runtime material requires the independently
admitted reify/migration consumer. `?`, Product context and the spelling `ω`
grant no reverse-migration authority. An unconnected unpacking or iteration
consumer is explicitly unavailable; it supplies no guessed runtime residual.
