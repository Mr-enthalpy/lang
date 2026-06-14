# Operator Design

This document defines the language-level operator design. It is normative for
future operator parser work, but the current v0.1 parser may not implement these
rules yet.

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
!  &  @  ~  ^  $  ++  --  ?
+=  -=  *=  /=  <<=  >>=
```

These are operator names only. Conventional mathematical or C-like readings are
not built-in semantics.

- `<<` and `>>` are only shift-looking operator spellings.
- `+=` is only a compound-looking operator spelling and does not imply
  assignment or mutation semantics.
- `*` is only the binary `*` operator spelling. It is not unary dereference.
- There is no symbol-encoded dereference operator in this design.

### Lexical Longest Match

Operator spellings are recognized with maximal munch. When multiple operator
spellings can start at the same source position, the lexer must choose the
longest spelling.

Examples:

- `<<=` is one operator spelling, not `<<` followed by `=`.
- `>>=` is one operator spelling, not `>>` followed by `=`.
- `<=` is one operator spelling, not `<` followed by `=`.
- `>=` is one operator spelling, not `>` followed by `=`.
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
+= -= *= /= <<= >>=
```

Ordinary postfix unary operators:

```text
! & @ ~ ^ $ ++ -- ?
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
    "::" PathLeaf
  | "." Name
  | ".." Name ArgPack
  | PostfixOperator
```

Examples:

```text
obj!.field
obj.field?
obj..map(a)!
t::+
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

compound-looking:
    += -= *= /= <<= >>=

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
```

Non-associative in this phase:

```text
< <= > >=
== !=
+= -= *= /= <<= >>=
```

The following require explicit grouping:

```text
a < b < c
a == b == c
a += b += c
```

A future parser may emit a diagnostic such as:

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

Operator names may appear as binder names and as path leaves:

```text
BinderName := Name | OperatorName
PathLeaf   := Name | OperatorName
```

Valid design cases:

```text
let +: _: operator = expr
t::+
std::int::+
```

Operator names may only be path leaves. They are not namespace-like
intermediate path nodes.

Invalid design cases:

```text
+::x
t::+::x
t::+::-
```

Operator binder names and operator path leaves are future parser work.

## Declaration Annotations

Operator declarations, when supported, use the same explicit rank annotation
form as other declarations:

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

Planned operator-expression AST shape:

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
case until operator binder names and operator path leaves are implemented. The
`<t: type, u: type>` head is included to show where the example's type names
come from.

## Relationship To Entity Alias Binding

`spec/entity-alias-design.md` documents future lexical alias binding:

```text
let binder === EntityRef
```

Operator aliases depend on this operator design because alias binders and
entity path leaves may need to contain `OperatorName` values:

```text
let << === xxx_bit::<<
let + === checked_int::+
```

Operator aliasing may select a concrete visible operator implementation from
another namespace, but it cannot rename one operator spelling into another. The
operator identity remains `spelling + fixity + arity`.

This is future design only. It does not implement operator lookup, entity
lookup, namespace resolution, import/package semantics, or alias validation.

## v0.1 Boundary

This design does not require the current parser to implement operator syntax.

Do not implement in this documentation task:

- operator parser;
- operator binder names;
- `t::+` parsing;
- operator lookup;
- entity alias binding (`let binder === EntityRef`);
- operator lowering;
- operator overload resolution;
- ADL;
- mutation semantics for compound-looking operators.

v0.1 remains a syntax frontend whose output is tokens, raw AST, and
diagnostics.
