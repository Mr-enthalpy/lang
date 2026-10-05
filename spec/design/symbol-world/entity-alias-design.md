# Local Lexical Path Alias

**Status:** canonical contract; Raw/Normalized Path material preservation is implemented.
The lexical Path environment consumer remains unavailable.

`let n === q` forms q once in the **old** Path environment, then installs its material:

    Gamma |- q =>_Path p
    Gamma |- let n === q => Gamma[n |->path p]

`LexicalPathAlias { local_spelling, path_material }` is lexical environment material.
It creates no NameBinding, Object, Place, resident generation, value, borrow,
forwarding member or OverloadGroup entry, and has no lifecycle. Ordinary binding
freshness does not apply. The spelling is block scoped, may be shadowed, and is
not a namespace member or export. Callable use retains ordinary lexical/dependency rules.

Explicit roots, anchors and dependencies remain part of p. Textual/open roots retain
their ordinary late resident-use resolution. Subsequent use composes p through the
[Path algebra](structured-path-algebra-and-pattern-splice.md), in `name::path` direction.
It never caches terminal BindingId or bypasses Path composition or no-reopen after
an actual name identity is fixed. Formation precedes installation; no recursive
AliasRef thunk or independent recursion semantics exists.

## Surface and implementation boundary

    AliasBinding ::= OptionalPolicy "let" AliasBinder "===" PathMaterial
    AliasBinder ::= Name | OperatorName
    PathMaterialAst { expression: Box<ExprAst>, span }
    NormPathMaterial { pattern: Box<NormPattern>, origin }

`===` is a structural delimiter. The frontend preserves Pattern/Path syntax,
including navigation and explicit RHS reinjection through `$`. It performs no
Path formation, value evaluation or lookup. Expression-shaped syntax does not
make arbitrary values admissible to Path; the ordinary Path consumer decides that.

Until connected, lang_build reports UnsupportedLexicalAlias and installs nothing.
No forwarding binding, stable terminal cache or independent alias evaluator may
substitute for that consumer.
