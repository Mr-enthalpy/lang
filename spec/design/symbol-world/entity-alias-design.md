# Local Lexical Alias

**Status:** canonical semantic contract; parser preservation is implemented;
semantic resolver wiring is pending.

`let n === q` adds one block-scoped lexical name entry. It does not create a
language entity.

```text
Resolve_Gamma(q) = S

Gamma |- let n === q
  => Gamma[n |->lex S]
```

Name resolution is context-independent. Both `q` and the local spelling `n`
resolve to the same terminal name binding `S`; value, type, and call projections are
performed only after that resolution.

```text
Resolve_Gamma(n) = Resolve_Gamma(q) = S
```

The lexical entry:

- is visible from its declaration point to the end of the current block;
- may be shadowed in a nested lexical scope;
- cannot cross a normal callable boundary;
- cannot be installed as a namespace/module member or exported;
- creates no NameBinding, Object, Place, runtime identity, or `V_tau` member;
- does not participate in ordinary binding freshness.

Consequently, the following laws coexist:

```text
NewOrdinaryBinding => FreshNameBinding + FreshDestinationPlace
LexicalAlias       => no binding entity
```

The semantic resolver needs only a scoped entry such as:

```text
LexAliasEntry {
    local_name,
    resolved_target: ResolvedNameBindingIdentity,
}
```

The entry is an implementation of lexical lookup, not a semantic value or
identity carrier.

## Surface contract

The frozen parser recognizes:

```text
AliasBinding ::= OptionalPolicy "let" AliasBinder "===" EntityRef
AliasBinder  ::= Name | OperatorName
```

`===` is a structural delimiter, not an equality, comparison, assignment, or
general expression operator. `EntityRef` is defined by
[`entity-ref-design.md`](entity-ref-design.md) and preserves source-order
inner-to-outer navigation.

The alias form is valid only in form position. It has no declaration
annotation, `=` value expression, `with` clause, deduce list, canonical
skeleton, or pipe-expression RHS. Its boundary is `;`, `}`, or EOF.

The Raw AST remains:

```text
LetAliasAst {
    policy: Option<ExprAst>,
    binder: AliasBinderAst,
    target: EntityRefAst,
    span: Span,
}
```

The parser preserves this shape and performs no target lookup.

## Diagnostics and implementation boundary

The parser owns only syntax diagnostics:

| Diagnostic | Meaning |
|---|---|
| `ExpectedAliasTarget` | no valid `EntityRef` follows `===` |
| `InvalidEntityRef` | the preserved entity reference is structurally malformed |
| `UnexpectedAliasRhsExpression` | expression material appears where an `EntityRef` is required |

The semantic pass owns target resolution, lexical scope, and shadowing. Until
that pass is connected, `lang_build` reports `UnsupportedLexicalAlias` and
installs nothing.

The current implementation must not use this form to create a forwarding
NameBinding, value, Place, member, or writable relation. Shared place observation is
expressed by the ordinary `ref`, `share`, and `@` relations defined in
[`type-values-places-and-borrow-views.md`](type-values-places-and-borrow-views.md).
