# EntityRef Design

This document records the future compile-time entity reference syntax used by
later parser/design phases.

It is a documentation/design note only. It is not implemented in the current
parser and is not part of v0.1 accepted syntax.

## Purpose

`EntityRef` names a compile-time entity in a strong syntax context. It is not a
runtime expression and is not evaluated by the parser.

Future alias binding will use it on the right-hand side:

```text
let binder === EntityRef
```

Phase 4.2 defines only the surface syntax and raw AST preservation boundary for
that future context. It does not add parser behavior.

## Provisional Grammar

```text
EntityRef ::= EntityPath

EntityPath ::= EntityPathSegment ("::" EntityPathSegment)* "::" EntityPathLeaf
             | EntityPathLeaf

EntityPathSegment ::= Name

EntityPathLeaf ::= Name | OperatorName
```

The distinction between `EntityPathSegment` and `EntityPathLeaf` is
intentional:

- intermediate `EntityPathSegment` entries must be text names;
- the final `EntityPathLeaf` may be a text name or an operator name.

Valid future design examples:

```text
std::int::+
checked_int::+
xxx_bit::<<
some_library::some_entity
some_entity
```

Invalid future design examples:

```text
+::x
std::+::x
std::+::-
<<::impl
```

Operator names may be final referred entities. They are not namespace-like
intermediate path segments.

## Relationship To Expression Paths

`EntityRef` is related to ordinary expression path syntax, but it is a distinct
future syntax form.

Expression paths appear inside normal expression parsing and produce ordinary
expression AST. They remain subject to the current `PipeExpr`, segment,
operator-expression, atom, and suffix rules.

`EntityRef` appears only inside future strong contexts that explicitly require a
compile-time entity reference. The known intended context is:

```text
let binder === EntityRef
```

The parser must not globally reinterpret ordinary paths as entity references.
Outside a future `EntityRef` context, existing expression parsing remains
unchanged.

## Relationship To Operator Names

Parser phase 4.1 introduced operator names in binder positions and final path
leaf positions. `EntityRef` reuses only the final-leaf part of that surface
capability:

```text
EntityPathLeaf ::= Name | OperatorName
```

This does not implement operator lookup. It does not check that the operator
exists. It does not validate operator identity. Operator alias identity
validation belongs to a later alias-binding phase or semantic/static validation
phase.

The current `<` operator-binder ambiguity documented in
`spec/operator-design.md` concerns `let` binder syntax. It does not by itself
add an `EntityRef` escape form.

## Future Raw AST Sketch

A future parser-preservation phase may use a raw AST shape like:

```text
EntityRefAst {
    path: Vec<EntityPathSegmentAst>,
    leaf: EntityPathLeafAst,
    span: Span
}

EntityPathSegmentAst {
    name: NameAst,
    span: Span
}

EntityPathLeafAst =
    Name(NameAst)
  | Operator(OperatorNameAst)
  | Error(ErrorAst)
```

This is a design sketch only. These Rust AST nodes are not implemented in Phase
4.2.

## Parser Boundary

Future parser preservation may parse `EntityRef` only inside explicit strong
contexts.

Known intended future context:

```text
let binder === EntityRef
```

Possible later contexts may exist, but this document does not define them.

Even when `EntityRef` parsing is eventually implemented, the parser must not:

- resolve the entity;
- check whether the entity exists;
- perform name lookup;
- perform operator lookup;
- perform namespace resolution;
- perform dependency resolution;
- interpret package/import/build-system semantics;
- perform type checking;
- perform kind checking;
- perform overload resolution;
- lower `EntityRef` into a call or runtime value.

The parser boundary is syntax preservation only.

## Alias-Binding RHS Restriction

For future alias binding:

```text
let binder === EntityRef
```

The right-hand side accepts only `EntityRef`.

It must not accept:

```text
PipeExpr
ArgPack
ClosureAst
operator expression
runtime expression
ordinary call-like syntax
block/body form
```

This restriction is stronger than ordinary `let name: annotation = expr`.
Alias binding does not bind a runtime value.

## Lexer Note For `===`

Phase 4.2 does not implement lexer changes.

If alias parsing is implemented later, `===` must become a structural delimiter
for alias binding. The lexer must longest-match it before:

```text
==
=
```

`===` is not an equality operator and is not a general expression operator.
Phase 4.2 does not add `===` to accepted parser syntax.

## Non-Goals

Do not implement in Phase 4.2:

```text
lexer token for ===
EntityRef parser
LetAliasAst
AliasBinderAst
EntityRefAst in Rust code
operator alias validation
operator identity checking
name lookup
operator lookup
namespace resolver
dependency resolver
build manifest parser
package/import/use/include/module syntax
runtime value binding semantics
```

Do not reinterpret existing expression paths as `EntityRef`.
