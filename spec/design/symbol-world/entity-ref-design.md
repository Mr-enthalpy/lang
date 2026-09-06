# Entity Reference

**Status:** alias-RHS Raw AST is implemented; general strong-context use is
Open.

`EntityRef` is a source-preserved reference used where syntax requires a
compile-time entity name. It is not a runtime expression and the parser does
not resolve it.

```text
EntityRef ::= EntityNavigation

EntityNavigation ::= EntityComponent ("::" EntityOuterComponent)*

EntityComponent      ::= Name | OperatorName
EntityOuterComponent ::= Name | Group
```

Navigation order is inner-to-outer. The leftmost component is the selected
symbol and the rightmost component is the outermost scope component. An
operator name may occur only in the innermost position unless a future
canonical rule permits operator-named scopes.

Examples:

```text
+::int::std
some_entity::some_library
some_entity
xxx::(int Vec::std)
```

The innermost component cannot be a group, and an operator cannot be an outer
scope component.

## Alias-RHS boundary

The implemented strong context is:

```text
OptionalPolicySpec "let" AliasBinder "===" EntityRef
```

The RHS accepts only an `EntityRef`; it does not accept a `PipeExpr`, product,
closure, operator expression, runtime expression, or block. Its boundary is
`;`, `}`, or EOF. The parser produces:

```text
EntityRefAst {
    components: Vec<NavComponentAst>,
    span: Span,
}
```

Outside this strong context, ordinary navigation remains expression syntax and
is not reclassified as an `EntityRef`.

## Semantic boundary

The parser preserves components and spans only. It does not perform name,
operator, namespace, dependency, package, type, kind, or overload resolution.
The lexical-alias consumer resolves the complete RHS once and stores the
terminal resolved name-binding identity in the block-local lexical environment, as
specified by [`entity-alias-design.md`](entity-alias-design.md).

General `EntityRef` use in other strong contexts remains Open. Such contexts
may reuse `EntityRefAst`; they must not change ordinary expression parsing or
give `EntityRef` runtime-value semantics.
