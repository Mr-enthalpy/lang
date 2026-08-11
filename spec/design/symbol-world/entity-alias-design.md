# Entity Alias Binding Design

> **Retirement notice — the semantic alias model described in this document is
> retired.**
>
> Two separate things were recorded here, and only the parser fact survives:
>
> 1. **Frozen parser fact (retained).** `===` is lexed as
>    `Symbol::TripleEqual` and the parser preserves `LetAliasAst` /
>    `AliasBinderAst` / `EntityRefAst`. This is v0.2 frozen contract material
>    and is not rewritten. It is a syntactic artifact of the frozen surface.
> 2. **Semantic alias model (retired).** Lexical alias binding, symbol/place
>    forwarding, `AliasChain`, alias-inherited writability, and "alias member"
>    contributions are **not** the target semantics and are no longer a future
>    direction.
>
> The target semantics has **no** ordinary symbol-alias or place-forwarding
> declaration form. Binding a name to an existing value is always an ordinary
> copy into a fresh symbol and a fresh place:
>
> ```text
> let T = uint8;
>
> SymbolId(T) ≠ SymbolId(uint8)
> PlaceId(T)  ≠ PlaceId(uint8)
> Value(T)    =  Value(uint8)
> ```
>
> Shared observation of another object is expressed only by the borrow views
> `ref`, `share`, and `@`, specified in
> `spec/design/symbol-world/type-values-places-and-borrow-views.md`.
>
> Operator-name binding is not an exception. The target direction models
> `operator` as an ordinary global type and operator environments as ordinary
> copyable/shadowable values. No semantic `let ===` form survives.
>
> Everything below is retained as the historical record of the surface form and
> of the retired semantic direction. Do not cite it as a specification of
> intended behavior.

**Status:**
- **Parser preservation** for `let binder === EntityRef` is implemented in v0.1 as raw AST preservation. The lexer recognizes `===` as a single structural delimiter token (`Symbol::TripleEqual`). The parser produces `LetAliasAst` containing `AliasBinderAst` and `EntityRefAst`.
- **The alias semantics, lookup, scope validation, and forwarding behavior described below are retired, not deferred.** The parser does not resolve targets, validate operator identity, perform entity lookup, or execute alias semantics, and no future pass is planned to do so under this model.

`v0.2` status: alias-let parser preservation is frozen contract material. Changes
in this window may clarify documentation or preserve narrowly additive syntax,
but must not implement alias target resolution, namespace lookup, operator
identity validation, or alias semantics.

This document records the design for lexical alias binding of
compile-time entities (Phase 4.3 design complete). Phase 4.4 implemented raw
parser preservation. The remaining sections describe the implemented
syntax and the retired semantic direction.

The right-hand side `EntityRef` syntax is defined separately in
`spec/design/symbol-world/entity-ref-design.md`.

This document owns the surface/parser alias syntax only. The *semantic* alias
forwarding model — value/place forwarding, the `AliasChain`, and its
writable-place effect — has been withdrawn;
`spec/design/symbol-world/type-values-places-and-borrow-views.md` now specifies
borrow views in its place and documents no forwarding mechanism.

## Purpose (retired)

The form recorded by the frozen parser is:

```text
AliasForm ::= OptionalPolicy? "let" AliasBinder "===" EntityRef FormBoundary
FormBoundary ::= ";" | "}" | EOF
```

AliasForm is recognized only in Form position. It is not valid inside
BindingSlot, ProductExtract, ParamClause, ReturnClause, Annotation, HeadClause,
or Expr.

The retired intent was a lexical-scope alias for a compile-time entity, stronger
than `import as` or `using`. Under that retired reading the form would:

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
- Outer navigation components after `::` may be `Name` or a parenthesized grouped scope expression (`NavComponentAst::Group`), matching ordinary Raw AST navigation. `xxx::(int Vec::std)` is valid.
- The innermost component must be a syntactic symbol component (`Name` or `OperatorName`). A grouped expression is valid only as an outer component; `(int Vec::std)::ns` emits `InvalidEntityRef` ("grouped expression cannot be an innermost navigation component").
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
AliasBinding ::= OptionalPolicy "let" AliasBinder "===" EntityRef

AliasBinder ::= Name | OperatorName

EntityRef ::= EntityNavigation
```

`EntityRef` is defined by `spec/design/symbol-world/entity-ref-design.md`. The full
`EntityNavigation` grammar is not duplicated here; see that document for the complete definition,
parser boundary, and raw-AST sketch.

For this design, the relevant parts are:

- `EntityRef` is a compile-time entity reference, not a runtime expression.
- The EntityRef preserves source-order inner-to-outer navigation components.
- The innermost component may be a text name, numeric name, or operator name.
- Outer components may be text names, numeric names, or grouped scope-producing expressions.
- Operator names are not valid as outer navigation components.

## Meaning (retired)

Under the retired model, `let binder === EntityRef` created a lexical alias
binding.

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

## Ordinary Name Alias (retired)

For text-name binders, the retired model allowed renaming the target:

```text
let local_name === exported_name::module::package
let Vec === Vector::collections::std
let map === map::iter::std
```

These examples are frozen parser-surface history only; they have no target
semantic alias meaning.

Under the retired model the alias would have shadowed previous visible bindings
named `local_name`, `Vec`, or `map` in the current lexical scope.

No target existence check occurs in the parser.

No namespace or package loading occurs in the parser.

## Operator Alias (retired)

The operator-name branch of the frozen `LetAliasAst` is parser-preserved history
only. It receives no semantic identity check, lookup rule, or forwarding pass.

The closed design direction is value-based:

```text
operator                    -- ordinary global type
operator : operator         -- current lexical operator-environment value
RHS `a op b`                -- desugars toward `(a, b) |> operator[op]`
```

A local operator environment is produced by ordinary value copy, lexical
shadowing, and Symbol `+=` / `-=` transformations. It is not an alias and does
not make two names share a place. The complete operator-environment layout,
lookup rules, and selector algebra remain future design and do not block this
document's retirement decision.

## Lexical Scope Rule (retired)

Under the retired model, alias bindings were lexical: they affected lookup only
after the declaration point and only inside the current lexical scope and its
nested scopes, unless shadowed by a later inner binding.

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
Product
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
alias RHS must not parse as an expression/product structure.

The alias RHS ends only at `;`, `}`, or EOF. Newlines are trivia inside alias
RHS parsing. If a token remains after the parsed EntityRef before a hard
boundary, it is an alias-RHS residual expression error, not the start of a new
form.

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
- validate operator alias identity (the semantic operation is retired);
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
| `UnexpectedAliasRhsExpression`      | Implemented    | The RHS of `===` is an expression form (PipeExpr, product, closure, etc.) instead of `EntityRef`. |

Retired/reserved diagnostic inventory:

| Diagnostic                          | Note                                                                     |
| ----------------------------------- | ------------------------------------------------------------------------ |
| `OperatorAliasIdentityMismatch`     | Historical reserved code; no target semantic validator is planned. |

`OperatorAliasIdentityMismatch` must not be used to revive operator alias
semantics. The frozen parser may retain the code/inventory for compatibility.

## Alias binding AST (current Phase 4.4 shape)

```text
LetAliasAst {
    policy: Option<ExprAst>,
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

The exact `EntityRefAst` shape is defined in `spec/design/symbol-world/entity-ref-design.md`.

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
