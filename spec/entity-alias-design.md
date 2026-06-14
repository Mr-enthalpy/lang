# Entity Alias Binding Design

This document records the future design for lexical alias binding of
compile-time entities.

It is not implemented in the current parser. It is not part of v0.1 accepted
syntax.

The right-hand side `EntityRef` syntax is defined separately in
`spec/entity-ref-design.md`. This document describes how future alias binding
will use that syntax.

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
- does not call anything;
- does not construct a runtime value;
- does not import a package by itself;
- does not resolve the target in the parser;
- binds a compile-time lookup name in the current lexical scope;
- may shadow ordinary names;
- may shadow operator bindings.

## Surface Grammar

Future syntax:

```text
AliasBinding ::= "let" AliasBinder "===" EntityRef

AliasBinder ::= Name | OperatorName

EntityRef ::= EntityPath
```

`EntityRef` is defined by `spec/entity-ref-design.md`. The full `EntityPath`
grammar is not duplicated here; see that document for the complete definition,
parser boundary, and raw-AST sketch.

For this design, the relevant parts are:

- `EntityRef` is a compile-time entity reference, not a runtime expression.
- The path may contain intermediate text-name segments and a final leaf that
  may be a text name or an operator name.
- Operator names are valid only in binder position or final path-leaf position.

## Meaning

`let binder === EntityRef` creates a lexical alias binding.

It binds `binder` to a compile-time entity reference for lookup in the current
lexical scope.

It does **not**:

- evaluate the right-hand side;
- construct a runtime value;
- call anything;
- import a package by itself;
- resolve the target in the parser;
- perform name lookup, operator lookup, namespace resolution, or dependency
  resolution.

Name resolution and namespace assembly are future phases. The parser, if this
is later implemented, only preserves syntax.

## Distinction from Ordinary `let`

Alias binding is distinct from ordinary v0.1 let binding.

Ordinary let:

```text
let name: annotation = expr
let <holes> skeleton = expr
```

binds syntax around a runtime or compile-time expression position, depending on
later semantics.

Alias let:

```text
let binder === EntityRef
```

binds a compile-time lookup alias only.

An alias binding has **no**:

- declaration annotation (`: type`, `: _ : fn`);
- `=` value expression;
- `guard` attribute;
- `with` clause;
- deduce list;
- canonical skeleton;
- pipe expression on the right-hand side.

Current ordinary `let` behavior is not changed. Existing `let name: annotation
= expr` is not reinterpreted as alias binding.

The `===` delimiter structurally separates the two forms. The parser selects
the alias-binding path when it sees `===` in `let` form position instead of `=`
or `:`.

## Ordinary Name Alias

For text-name binders, aliasing may rename the target:

```text
let local_name === package::module::exported_name
let Vec === std::collections::Vector
let map === std::iter::map
```

These examples are future syntax only.

The alias shadows previous visible bindings named `local_name`, `Vec`, or
`map` in the current lexical scope.

No target existence check occurs in the parser.

No namespace or package loading occurs in the parser.

## Operator Alias

For operator binders, aliasing is stricter than for ordinary names.

An operator alias may select a concrete visible operator implementation from
another namespace, but it may **not** rename one operator into another.

The operator binder and the final operator leaf of the target `EntityRef` must
have the same operator identity.

Operator identity is:

```text
spelling + fixity + arity
```

Valid future design examples:

```text
let << === xxx_bit::<<
let >> === xxx_bit::>>
let + === checked_int::+
```

Invalid future design examples:

```text
let << === xxx_bit::>>
let + === xxx_bit::<<
let - === some_lib::+
```

These are rejected because the operator spelling differs between binder and
target leaf.

### Where identity checking belongs

The parser may later preserve both sides as raw AST. The identity check belongs
to a later static validation or name-resolution-adjacent phase, because fixity,
arity, and operator declaration lookup may be needed to disambiguate the
target.

A purely syntactic first-pass rule (comparing the operator spelling token text)
is possible as optional future parser validation, but it is not a Phase 4.3
implementation item. Operator identity is `spelling + fixity + arity`, and
fixity/arity may depend on the target's resolved declaration, which is not
available in the parser.

Phase 4.3 does not implement this validation.

## Lexical Scope Rule

Alias bindings are lexical. They affect lookup only after the declaration point
and only inside the current lexical scope and its nested scopes, unless
shadowed by a later inner binding.

`let binder === EntityRef` may shadow:

- ordinary value/type/entity names;
- operator bindings;
- prelude bindings;
- imported namespace members;
- outer lexical aliases.

It must **not**:

- mutate the original entity;
- change a namespace globally;
- rewrite other files;
- affect lookup before the declaration point.

Alias bindings follow the same shadowing discipline as ordinary `let` bindings:
an inner alias shadows an outer alias with the same binder identity.

## Relation to `===`

`===` is a future structural delimiter for alias binding.

It is **not**:

- an equality operator;
- a comparison operator;
- an assignment operator;
- a general expression operator;
- an operator name.

When parser preservation is eventually implemented, the lexer must
longest-match:

```text
===
```

before:

```text
==
=
```

`===` should not become a general expression operator unless a future design
explicitly changes this. Phase 4.3 does not implement this lexer change.

The current lexer may tokenize `===` as `==` followed by `=`. A later
alias-parser phase must update lexer maximal-munch rules before adding alias
syntax preservation.

## Relation to EntityRef

The right-hand side of `===` accepts only `EntityRef`.

It must **not** accept:

```text
PipeExpr
ArgPack
ClosureAst
operator expression
runtime expression
ordinary call-like syntax
block/body form
```

Examples that must remain invalid future alias syntax:

```text
let x === a |> f
let x === f(a)
let x === { body }
let x === (a, b)
let x === a + b
```

Note: `f(a)` is not traditional call syntax in this language anyway; still,
alias RHS must not parse as an expression/ArgPack structure.

## Parser Boundary

If a future parser phase accepts this syntax, the parser may preserve a raw
`LetAliasAst` and `EntityRefAst`.

Even when alias binding is eventually parsed, the parser must **not**:

- resolve the target entity;
- check whether the target exists;
- perform name lookup;
- perform operator lookup;
- perform namespace resolution;
- perform dependency resolution;
- load packages;
- interpret import/use/include/module syntax;
- validate operator alias identity (beyond optional spelling comparison);
- perform type checking;
- perform kind checking;
- perform overload resolution;
- lower aliases into runtime values.

The parser may only preserve raw syntax and emit narrow syntax diagnostics.

Phase 4.3 must not implement even that parser preservation.

## Future Diagnostics Design Note

These diagnostics are future design only. They are not added to
`DiagnosticCode` in Rust and are not implemented in this phase.

Possible future diagnostics:

| Diagnostic                          | Trigger                                                                 |
| ----------------------------------- | ----------------------------------------------------------------------- |
| `ExpectedAliasTarget`               | `let binder ===` is not followed by a valid `EntityRef`.                |
| `InvalidAliasBinder`                | The binder position after `let` is not a valid `Name` or `OperatorName`. |
| `InvalidEntityRef`                  | The `EntityRef` on the RHS is malformed (e.g., operator in segment position). |
| `OperatorAliasIdentityMismatch`     | The operator binder spelling differs from the target leaf spelling.     |
| `UnexpectedAliasRhsExpression`      | The RHS of `===` is an expression form (PipeExpr, ArgPack, closure) instead of `EntityRef`. |

`OperatorAliasIdentityMismatch` may be a parser diagnostic (spelling-only) or
a later static-semantic diagnostic (including fixity/arity). This decision is
deferred to the alias-parser implementation phase.

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
  | Error(ErrorAst)

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
  | Error(ErrorAst)
```

The exact `EntityRefAst` shape is defined in `spec/entity-ref-design.md`.

These nodes are not implemented in this documentation-only task. They are not
added to the Rust `ast` module in Phase 4.3.

## Non-Goals

Do not implement in Phase 4.3:

```text
lexer token for ===
EntityRef parser
alias parser
LetAliasAst
AliasBinderAst in Rust code
EntityRefAst in Rust code
DiagnosticCode additions
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

Do not reinterpret existing syntax:

```text
let name: annotation = expr
```

That remains ordinary v0.1 let-binding syntax.

Do not add accepted syntax tests for:

```text
let binder === EntityRef
```

Do not add lexer golden tests for `===`.

Do not change `let name: annotation = expr`.
