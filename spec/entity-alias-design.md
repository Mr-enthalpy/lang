# Entity Alias Binding Design

This document records a future language design topic: lexical alias binding for
compile-time entities.

It is not implemented in the current parser. It is not part of v0.1 accepted
syntax.

## Purpose

The language will eventually support a declaration form:

```text
let binder === EntityRef
```

This is similar to `import as` or `using` in traditional languages, but
stronger. It introduces a lexical-scope alias for a compile-time entity.

An alias binding:

- does not bind a runtime value;
- does not evaluate an expression;
- binds a compile-time lookup name in the current lexical scope;
- may shadow ordinary names;
- may shadow operator bindings.

Examples:

```text
let name === some_library::some_entity
let << === xxx_bit::<<
let >> === xxx_bit::>>
```

## Provisional Surface Grammar

Future syntax:

```text
AliasBinding ::= "let" AliasBinder "===" EntityRef

AliasBinder ::= Name | OperatorName

EntityRef ::= EntityPath
EntityPath ::= EntityPathSegment ("::" EntityPathLeaf)*
```

For this provisional design, intermediate `EntityPathSegment` entries are text
names. Operators are valid only in binder position or final path-leaf position.

`===` is a structural delimiter for alias binding. It is not an equality
operator and not a general expression operator unless a later design explicitly
changes that.

The grammar is provisional. It exists to document the intended parser/semantic
boundary before implementation.

## Parser Boundary

If a future parser phase accepts this syntax, the parser may preserve a raw
`EntityRefAst`.

The parser must not:

- resolve the entity reference;
- check whether the referenced entity exists;
- perform name lookup;
- perform operator lookup;
- perform namespace resolution;
- perform dependency resolution;
- interpret package/import/build-system semantics.

The parser only preserves shape.

## Right-Hand Side Restriction

The right-hand side of `===` accepts only a lookupable compile-time entity
reference.

It must not accept:

```text
runtime expression
PipeExpr
ArgPack
ClosureAst
ordinary call-like syntax
operator expression
block/body form
```

This is stronger than a normal `let` value expression. The target is an entity
reference, not a runtime value.

## Ordinary Name Alias

For text names, the alias may rename the imported entity:

```text
let local_name === package::module::exported_name
```

This means `local_name` shadows previous visible bindings in the current
lexical scope. The right-hand side remains a compile-time entity reference.

No resolution or existence check is performed by the parser.

## Operator Alias

Operator aliases are stricter than ordinary name aliases.

An operator can only be rebound to the same operator identity:

```text
spelling + fixity + arity
```

Valid design cases:

```text
let << === xxx_bit::<<
let >> === xxx_bit::>>
let + === checked_int::+
```

Invalid design cases:

```text
let << === xxx_bit::>>
let + === xxx_bit::<<
let - === some_lib::+
```

The intent is that operator aliasing can select a concrete visible operator
implementation from another namespace, but it cannot rename one operator
spelling into another.

This validation is future semantic or syntax-only validation work. It is not
implemented by the current parser.

Parser phase 4.1 supplies the operator-name syntax needed in binder and final
path-leaf positions. Alias binding itself is still not parsed: `===`,
`EntityRef`, `LetAliasAst`, alias validation, and entity lookup remain future
work.

## Lexer Note

The spelling `===` is reserved as a future structural delimiter.

If implemented, the lexer must longest-match it before `==` and `=`.

`===` should not be treated as ordinary equality syntax. It should not become a
general expression operator unless a future design explicitly changes this.

## Future AST Sketch

Possible future raw AST shape:

```text
LetAliasAst {
    binder: AliasBinderAst,
    target: EntityRefAst,
    span: Span
}

AliasBinderAst =
    Name(NameAst)
  | Operator(OperatorNameAst)

EntityRefAst {
    path: Vec<EntityPathSegmentAst>,
    leaf: EntityPathLeafAst,
    span: Span
}

EntityPathSegmentAst =
    Name(NameAst)

EntityPathLeafAst =
    Name(NameAst)
  | Operator(OperatorNameAst)
```

These nodes are not implemented in this documentation-only task.

## Non-Goals

Do not implement:

```text
=== parser
EntityRef parser
LetAliasAst
operator alias validation
name lookup
operator lookup
namespace resolver
dependency resolver
build manifest parser
import/use/include/module syntax
runtime value binding semantics
```

Do not reinterpret existing syntax:

```text
let name: annotation = expr
```

That remains ordinary v0.1 let-binding syntax.
