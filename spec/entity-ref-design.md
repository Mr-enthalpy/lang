# EntityRef Design

**Status:**
- **AliasRhsEntityRef**: implemented in Phase 4.4 as raw AST preservation
  inside `let binder === EntityRef`. The parser produces `EntityRefAst`
  with source-order inner-to-outer navigation components. EntityRef parsing is
  available only in alias-let RHS position; it is not a general expression
  parser mode.
- **GeneralEntityRef**: future design. Standalone `EntityRef` parsing in
  arbitrary strong contexts is not yet implemented.

This document records the future compile-time entity reference syntax used by
later parser/design phases (Phase 4.2 design). Phase 4.4 implements a raw
EntityRef parser inside alias-let RHS only; EntityRef is not a general
expression parser mode.

## Purpose

`EntityRef` names a compile-time entity in a strong syntax context. It is not a
runtime expression and is not evaluated by the parser.

Alias binding (Phase 4.3 design; Phase 4.4 alias-RHS EntityRef implemented,
`spec/entity-alias-design.md`) uses it on the
right-hand side:

```text
let binder === EntityRef
```

Phase 4.2 defines only the surface syntax and raw AST preservation boundary for
that future context. It does not add parser behavior. Phase 4.3 completes the
alias binding design documentation that uses `EntityRef` on the RHS.

## Provisional Grammar

```text
EntityRef ::= EntityNavigation

EntityNavigation ::= EntityComponent ("::" EntityOuterComponent)*

EntityComponent ::= Name | NumericName | OperatorName

EntityOuterComponent ::= Name | NumericName | Group
```

Navigation order is inner-to-outer. The leftmost component is the innermost
selected symbol, and the rightmost component is the outermost scope component.
Raw AST preserves source-order navigation components and performs no lookup.
Operator names may only be innermost entity-reference components unless a
future design explicitly allows operator-named scopes.

The innermost navigation component must be a syntactic symbol component:
`Name`, `NumericName`, or `OperatorName`. A grouped expression is valid only as
an outer navigation component after `::`; it represents a scope-producing
expression, not a selected symbol. This matches ordinary Raw AST navigation.
`xxx::(int Vec::std)` is valid (grouped outer component); `(int Vec::std)::ns`
is invalid as an innermost component and emits `InvalidEntityRef`.

Valid future design examples:

```text
+::int::std
+::checked_int
<<::xxx_bit
some_entity::some_library
some_entity
```

Invalid future design examples:

```text
x::+
x::int::+
+::x::+
impl::<<
```

Operator names may be innermost referred entities. They are not outer scope
components.

## Relationship To Expression Navigation

`EntityRef` is related to ordinary expression navigation syntax, but it is a distinct
future syntax form.

Expression navigation appears inside normal expression parsing and produces
ordinary expression AST. It remains subject to the current `PipeExpr`, segment,
operator-expression, atom, and suffix rules.

`EntityRef` appears only inside future strong contexts that explicitly require a
compile-time entity reference. The known intended context (Phase 4.3 design
complete) is:

```text
let binder === EntityRef
```

The parser must not globally reinterpret ordinary navigation as entity references.
Outside a future `EntityRef` context, existing expression parsing remains
unchanged.

## Relationship To Operator Names

Parser phase 4.1 introduced operator names in binder positions and innermost
navigation-component positions. `EntityRef` reuses only that innermost
component capability:

```text
EntityComponent ::= Name | OperatorName
```

This does not implement operator lookup. It does not check that the operator
exists. It does not validate operator identity. Operator alias identity
validation belongs to a later alias-binding phase or semantic/static validation
phase (see `spec/entity-alias-design.md` Phase 4.3 design).

The current `<` operator-binder ambiguity documented in
`spec/operator-design.md` concerns `let` binder syntax. It does not by itself
add an `EntityRef` escape form.

## Raw AST shape

### Current alias-RHS EntityRef AST (Phase 4.4 implemented)

The alias-let RHS parser produces the following raw AST (implemented in
`crates/lang_syntax/src/ast.rs`):

```text
EntityRefAst {
    components: Vec<NavComponentAst>,
    span: Span
}
```

### Future general EntityRef contexts

For future strong contexts outside alias-let RHS, the same `EntityRefAst` shape
is expected. The parser does not yet accept `EntityRef` as a standalone
expression mode.

## Parser Boundary

Future parser preservation may parse `EntityRef` only inside explicit strong
contexts.

Known intended future context (Phase 4.3 design complete):

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

For future alias binding (Phase 4.3 design complete, see
`spec/entity-alias-design.md`):

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
