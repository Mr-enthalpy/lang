# Entity Alias Binding Design

**Status:**
- **Parser preservation** for `let binder === EntityRef` is implemented in v0.1 as raw AST preservation. The lexer recognizes `===` as a single structural delimiter token (`Symbol::TripleEqual`). The parser produces `LetAliasAst` containing `AliasBinderAst` and `EntityRefAst`.
- **Alias semantics, lookup, scope validation, operator identity validation, and namespace resolution are future work.** The parser does not resolve targets, validate operator identity, perform entity lookup, or execute alias semantics.

This document records the design for lexical alias binding of
compile-time entities (Phase 4.3 design complete). Phase 4.4 implemented raw
parser preservation. The remaining sections describe both the implemented
syntax and the future semantic behavior.

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

## Implemented in v0.1 as raw AST preservation

Phase 4.4 implemented raw parser preservation for `let binder === EntityRef`.

**What is implemented:**

- `===` is lexed as `Symbol::TripleEqual`, one structural token (before `==` and `=`).
- The parser accepts `let Name === EntityRef` and `let OperatorName === EntityRef` in let-form position.
- `AliasBinderAst` preserves the binder as `Name(NameAst)` or `Operator(OperatorNameAst)`.
- `EntityRefAst` preserves the right-hand side as source-order inner-to-outer navigation components.
- Operator names are valid only as innermost entity-reference components; `x::+` and `a::+::b` emit `InvalidEntityRef`.
- Residual expression tokens after the entity reference emit `UnexpectedAliasRhsExpression`.
- Missing targets emit `ExpectedAliasTarget`.
- The alias-let dispatch guards against extract-let, annotation, and `with` paths: none of these parse as alias declarations. `guard` is an ordinary binder name, not an alias modifier.

**What is not implemented:**

- Target entity resolution.
- Operator alias identity validation (`spelling + fixity + arity`).
- Name lookup, operator lookup, namespace resolution, dependency resolution.
- Import/package/build-system semantics.
- Alias scope semantics, shadowing, or semantic validation.

## Surface Grammar

Grammar:

```text
AliasBinding ::= "let" AliasBinder "===" EntityRef

AliasBinder ::= Name | OperatorName

EntityRef ::= EntityNavigation
```

`EntityRef` is defined by `spec/entity-ref-design.md`. The full
`EntityNavigation` grammar is not duplicated here; see that document for the complete definition,
parser boundary, and raw-AST sketch.

For this design, the relevant parts are:

- `EntityRef` is a compile-time entity reference, not a runtime expression.
- The path may contain intermediate text-name segments and a final leaf that
  may be a text name or an operator name.
- Operator names are valid only in binder position or innermost navigation-component position.

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
let local_name === exported_name::module::package
let Vec === Vector::collections::std
let map === map::iter::std
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
let << === <<::xxx_bit
let >> === xxx_bit::>>
let + === +::checked_int
```

Invalid future design examples:

```text
let << === xxx_bit::>>
let + === <<::xxx_bit
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

`===` is a structural delimiter token (`Symbol::TripleEqual`) for alias binding.

It is **not**:

- an equality operator;
- a comparison operator;
- an assignment operator;
- a general expression operator;
- an operator name.

The lexer longest-matches `===` before `==` and `=`. This is already implemented
in Phase 4.4.

`===` should not become a general expression operator unless a future design
explicitly changes this.

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

The parser already preserves raw `LetAliasAst` and `EntityRefAst` (Phase 4.4
implementation). It does **not**:

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

The parser preserves raw syntax and emits narrow syntax diagnostics only.

## Diagnostics

The following diagnostic codes are implemented in `DiagnosticCode` (Phase 4.4):

| Diagnostic                          | Status         | Trigger                                                                 |
| ----------------------------------- | -------------- | ----------------------------------------------------------------------- |
| `ExpectedAliasTarget`               | Implemented    | `let binder ===` is not followed by a valid `EntityRef`.                |
| `InvalidAliasBinder`                | Reserved       | The binder position after `let` is not a valid `Name` or `OperatorName`. Currently not emitted; falls through to ordinary-let `ExpectedName`. |
| `InvalidEntityRef`                  | Implemented    | The `EntityRef` on the RHS is malformed (e.g., operator in segment position). |
| `UnexpectedAliasRhsExpression`      | Implemented    | The RHS of `===` is an expression form (PipeExpr, ArgPack, closure, etc.) instead of `EntityRef`. |

Future diagnostics, not implemented:

| Diagnostic                          | Note                                                                     |
| ----------------------------------- | ------------------------------------------------------------------------ |
| `OperatorAliasIdentityMismatch`     | Spelling + fixity + arity check. Deferred to future semantic validation. |

`OperatorAliasIdentityMismatch` may be a parser diagnostic (spelling-only) or
a later static-semantic diagnostic (including fixity/arity).

## Alias binding AST (current Phase 4.4 shape)

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
    components: Vec<NavComponentAst>,
    span: Span
}
```

The exact `EntityRefAst` shape is defined in `spec/entity-ref-design.md`.

These nodes are implemented in the current Rust `ast` module (Phase 4.4).

## Non-Goals

The following are implemented in the current parser (Phase 4.4 / 4.4.1):

```text
=== lexer token (Symbol::TripleEqual)
EntityRef parser (alias-let RHS only)
alias parser (let_stmt.rs parse_let_form dispatch)
LetAliasAst, AliasBinderAst, EntityRefAst in Rust code
ExpectedAliasTarget, InvalidEntityRef, UnexpectedAliasRhsExpression diagnostics
```

Do not implement in the parser:

```text
operator alias identity validation
operator identity checking
name lookup
operator lookup
namespace resolver
dependency resolver
build manifest parser
package/import/use/include/module syntax
runtime value binding semantics
alias target resolution
alias scope validation
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
