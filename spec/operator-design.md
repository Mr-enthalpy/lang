# Operator Design

This document defines the language-level operator design. Parser phase 4
implements expression-level operator syntax as raw AST sugar. Parser phase 4.1
implements operator names in binder position and innermost navigation-component
position.

Operators are surface syntax for specially shaped function invocation. They are
not built-in arithmetic, comparison, mutation, assignment syntax, parser-level
evaluation, overload-resolution syntax, ADL, or type-directed lookup syntax.

The parser may recognize operator spelling, fixity, arity, precedence, and
associativity because these determine AST shape. It must preserve operator
expressions as operator sugar in AST and must not lower them into ordinary calls
in v0.1.

Later semantic phases may interpret operator sugar as calls to operator-named
functions.

## Operator Identity

Operator identity is:

```text
spelling + fixity + arity
```

The same spelling may have multiple operator identities. For example, `-` may
be a binary operator spelling or prefix negative operator sugar.

## Initial Spellings

Initial operator spellings are:

```text
+  -  *  /
<  <=  >=  >  ==  !=
<<  >>
&  |  &&  ||
!  @  ~  ^  $  ++  --  ?
+=  -=  *=  /=  &=  |=  <<=  >>=
```

These are operator names only. Conventional mathematical or C-like readings are
not built-in semantics.

- `<<` and `>>` are only shift-looking operator spellings.
- `&` and `|` are only ordinary binary operator spellings and do not imply
  built-in integer operations.
- `&=` and `|=` are only equals-suffixed ordinary binary operator spellings
  and do not imply assignment or mutation semantics.
- `&&` and `||` are only ordinary binary operator spellings. They do not
  introduce parser-level lazy evaluation or control-flow constructs.
- `+=`, `-=`, `*=`, `/=`, `<<=`, and `>>=` are only equals-suffixed ordinary
  binary operator spellings and do not imply assignment or mutation semantics.
- `*` is only the binary `*` operator spelling. It is not unary dereference.
- There is no symbol-encoded dereference operator in this design.

Operator spelling is syntax. Operator meaning comes from later operator
resolution. The global built-in operator implementation set remains empty.
Only type companion namespaces may provide built-in implementations. The
current intended built-in companion implementations are:

- `int`, `uint`, and `float` families: basic arithmetic, ordering or partial
  ordering where applicable, and equality.
- `bool`: `!`, `&&`, and `||` as ordinary eager bool operator implementations,
  not parser-level control flow.

This spelling-table adjustment does not add built-in `&`, `|`, `&=`, or `|=`
implementations for integer, unsigned integer, float, or bool families.

### Lexical Longest Match

Operator spellings are recognized with maximal munch. When multiple operator
spellings can start at the same source position, the lexer must choose the
longest spelling.

Examples:

- `<<=` is one operator spelling, not `<<` followed by `=`.
- `>>=` is one operator spelling, not `>>` followed by `=`.
- `<=` is one operator spelling, not `<` followed by `=`.
- `>=` is one operator spelling, not `>` followed by `=`.
- `&=` is one operator spelling, not `&` followed by `=`.
- `|=` is one operator spelling, not `|` followed by `=`.
- `&&` and `||` are single operator spellings.
- `==` and `!=` are single operator spellings.
- `++` and `--` are single operator spellings.

If two spellings have the same length, normal spelling equality determines the
token; no semantic interpretation is involved.

## Fixity And Arity

Ordinary binary operators:

```text
+ - * /
< <= >= > == !=
<< >>
& | && ||
+= -= *= /= &= |= <<= >>=
```

Ordinary postfix unary operators:

```text
! @ ~ ^ $ ++ -- ?
```

Prefix operators:

```text
-
```

Prefix `-x` is prefix negative operator sugar. It is not a negative literal.
The lexer still produces `-` and the following literal or atom separately.

No other C-like prefix operators are part of this design. In particular,
`!x`, `&x`, `*x`, `~x`, `++x`, and `--x` are not prefix forms.

## Postfix Operators

Postfix unary operators are treated like single-argument function-style surface
sugar. They compose with other atom suffixes and do not terminate atom suffix
parsing.

Conceptual suffix grammar:

```text
Atom := Primary AtomSuffix*

AtomSuffix :=
    "::" NavComponent
  | "." Name
  | ".." Name ArgPack
  | PostfixOperator
```

Examples:

```text
obj!.field
obj.field?
obj..map(a)!
```

`obj!.field` has the same AST grouping as:

```text
(obj!).field
```

## Precedence

Operators are local to expression parsing inside pipe segments. Ordinary
operators bind more tightly than both whitespace auto-pipe and `|>` pipe.

```text
a + b |> f
```

parses as:

```text
(a + b) |> f
```

not:

```text
a + (b |> f)
```

Likewise:

```text
a b + c
```

parses as:

```text
a (b + c)
```

not:

```text
(a b) + c
```

Operator-aware segment grammar:

```text
SegmentElement := OperatorExpr | ArgPack
```

This is not a traditional C-like precedence language. Operator precedence is a
segment-local sugar layer inside the existing pipe/segment architecture.

Precedence order from tightest to loosest:

```text
atom suffix:
    :: . .. postfix-operator

prefix negative:
    -x

multiplicative:
    * /

additive:
    + -

shift-looking:
    << >>

comparison:
    < <= > >=

equality:
    == !=

ampersand spelling:
    &

pipe spelling:
    |

double-ampersand spelling:
    &&

double-pipe spelling:
    ||

equals-suffixed:
    += -= *= /= &= |= <<= >>=

pipe:
    |>
```

The pipe operator remains the outer expression skeleton.

## Associativity

Left-associative:

```text
* /
+ -
<< >>
&
|
&&
||
```

Non-associative in this phase:

```text
< <= > >=
== !=
+= -= *= /= &= |= <<= >>=
```

The following require explicit grouping:

```text
a < b < c
a == b == c
a += b += c
a &= b &= c
```

The parser emits a diagnostic for ungrouped chains:

```text
chained non-associative operator requires explicit grouping
```

Explicit grouping still produces accepted AST-level syntax:

```text
(a < b) < c
a < (b < c)
```

Whether such expressions are semantically valid is not a parser question.

## Angle Brackets

The lexer continues to produce symbols by spelling. Parser context decides
interpretation.

In deduce-list strong contexts:

```text
<...>
```

is a `DeduceList`.

In expression/operator contexts:

```text
<
>
<=
>=
```

are operator spellings.

`<>` has no general generic-call meaning. It remains a deduce list only in
strong binding contexts.

## Binder And Path Names

Operator names may appear in:

- expression operator sugar, implemented in Phase 4;
- binder-name position, implemented in Phase 4.1;
- innermost navigation-component position before `::`, implemented in Phase 4.1.

```text
BinderName := Name | OperatorName
NavComponent := Name | NumericName | OperatorName | GroupedExpr
```

Valid design cases:

```text
let +: _: operator = expr
let >: _: operator = expr
+::int::std
<<::bit::std
```

Phase 4.1 does not accept `<` as a simple operator binder spelling:

```text
let <: _: operator = expr
```

In `let` binder position, `<` is already the strong-context entry for an
extract-let deduce list (`let <x> ... = ...`). The parser therefore commits to
extract-let parsing when it sees `<` after `let`, rather than treating `<` as a
simple operator binder. This is a syntax disambiguation limitation, not an
operator semantic rule. A future phase may add escaping or another explicit
disambiguation rule if `<` needs to be declared as an operator binder.

Navigation order is inner-to-outer. The leftmost component is the innermost
selected symbol, and the rightmost component is the outermost scope component.
Raw AST preserves source-order navigation components and performs no lookup.
Operator names may only be innermost navigation components unless a future
design explicitly allows operator-named scopes.

Invalid design cases:

```text
x::+
x::int::+
+::x::+
t::+::-
```

Operator names are syntax only here. They are not looked up, resolved,
overloaded, or lowered by the parser.

## Declaration Annotations

Operator declarations use the same explicit rank annotation form as other
declarations:

```text
let +: _: operator = expr
```

The declaration annotation rule remains:

```text
let name: annotation = expr
let name: type_object_annotation: rank_annotation = expr
```

A bare declaration annotation is preserved exactly as written. Rank annotation
requires the explicit `type_object_annotation : rank_annotation` form. The same
rule applies to operator declarations.

## AST Status

Operator expressions are preserved as operator sugar. The parser must not lower
operator syntax to ordinary calls in v0.1.

Implemented operator-expression AST shape:

```text
OperatorExprAst ::=
    AtomExpr(AtomAst)
  | OperatorSugarAst {
        operator: OperatorName,
        fixity: Prefix | Postfix | Binary,
        args: Vec<OperatorExprAst>
    }
```

Examples:

- `obj!` produces postfix operator sugar, not a call.
- `a + b` produces binary operator sugar.
- `-x` produces prefix operator sugar, not a negative literal.

## Lookup Boundary

Future operator lookup follows ordinary visible binding lookup.

1. Operator lookup follows ordinary visible binding lookup.
2. There is no ADL-style lookup.
3. `1 + 2` does not automatically search operand type namespaces.
4. A visible global or prelude operator binding may forward to a type-local
   implementation.
5. Type-local implementations such as `uint8::+` are not global built-in
   operators.
6. Any forwarding behavior belongs to the operator binding implementation, not
   to parser syntax.

Future design example:

```text
let +: _: operator = <t: type, u: type>(a: t, b: u) => {
    a t::+ b
}
```

This example documents future lookup design only. It is not a parser golden
case for lookup behavior. The `<t: type, u: type>` head is included to show
where the example's type names come from.

## Relationship To Entity Alias Binding

`spec/entity-alias-design.md` documents future lexical alias binding (Phase 4.3
design complete):

```text
let binder === EntityRef
```

Parser phase 4.1 supplies the operator-name syntax that future aliases need in
binder and entity path-leaf positions:

```text
let << === <<::xxx_bit
let + === +::checked_int
```

Operator aliasing may select a concrete visible operator implementation from
another namespace, but it cannot rename one operator spelling into another. The
operator identity remains `spelling + fixity + arity`. The identity check is
deferred to a future static validation or name-resolution-adjacent phase.

Alias binding remains future design only. Phase 4.1 does not implement
operator lookup, entity lookup, namespace resolution, import/package semantics,
alias parsing, or alias validation.

## v0.1 Boundary

The current parser implements expression-level operator syntax preservation and
operator-name preservation in binder/final-path-leaf positions. Do not
implement in this phase:

- operator lookup;
- entity alias binding (`let binder === EntityRef`);
- operator lowering;
- operator overload resolution;
- ADL;
- mutation semantics for equals-suffixed operator spellings.

v0.1 remains a syntax frontend whose output is tokens, raw AST, and
diagnostics.
