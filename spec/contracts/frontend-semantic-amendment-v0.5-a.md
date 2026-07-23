# Frontend Semantic Amendment v0.5-A

Status: accepted amendment for the current typed frontend substrate.

This document records the versioned correction that introduces the callable
tail, first-class dot closure, and Pattern remainder syntax implemented during
the v0.5 semantic-surface work. It does not rewrite the completed v0.1 frontend
or the closed v0.2 Raw AST freeze.

The historical records remain, verbatim:

```text
spec/contracts/raw-ast-contract-v0.1.md
spec/contracts/raw-ast-contract-freeze-v0.2.md
spec/contracts/v0.3-normalization-handoff-checklist.md
spec/implementation/v0.1/*
spec/public/v0.2/*
```

The corrected current contract is:

```text
spec/contracts/raw-ast-contract-v0.5.md
```

## 1. Why an amendment is required

The v0.2 freeze prohibited replacing or extending the frozen Raw AST except
for a documented hard structural correction. PR #95 contains both structural
corrections and deliberate new syntax. Treating all of them as if they had
always been part of v0.1/v0.2 would erase the freeze boundary.

This amendment therefore classifies each delta explicitly:

| Delta | Classification | Reason |
|---|---|---|
| Closure `placement`, optional `head`, and implementation stored independently | Hard structural correction | Head presence and in-place placement are independent semantic facts. The old sum type could not preserve a headed in-place closure without lying about placement. |
| Leading `.name` / `DotClosure` atom | Normalization-driven structural extension | `.name` must exist independently before the mechanical `E.name -> E |> .name` lowering can preserve first-class substitution. |
| `...` / `BindingPatternAst::Pack` | New Pattern syntax amendment | This is a new general binding-pattern form, not a repair to a previously accepted v0.2 spelling. |
| Capture `Expr` items -> let-shaped explicit/inferred items | Closure-head grammar amendment | Every capture now elaborates to a binding; the old expression list is retained only where its binding name is uniquely inferable. |
| Callable implementation tail (`default`, bare/message `delete`, named strategy) | New closure-tail grammar amendment | These are new strong-context alternatives in the callable tail. |
| Delete message `(message_expr) delete` -> `(StringLiteral) delete` | Deliberate source-language contraction | Delete diagnostics are static source messages, not general evaluated expressions. |
| Malformed-tail `ErrorAst` recovery | Hard recovery correction | Invalid source must not become a legal empty user body. |
| Global one-pack validation | New post-normalization invariant | Parser-local counting cannot enforce a normalized-level invariant across every binding slot. |

## 2. Version boundary

The frozen v0.2 documents continue to describe the v0.2 surface:

```text
19 structural Symbol variants
32 DiagnosticCode variants
ClosureAst::InPlace | ClosureAst::Explicit
CaptureItemAst { expr }
no general Pattern pack syntax
no v0.5 callable-tail grammar
```

The amended v0.5 contract describes the current implementation:

```text
20 structural Symbol variants, including Ellipsis
33 DiagnosticCode variants
ClosureAst { placement, head, body, span }
CaptureItemAst::Explicit | CaptureItemAst::Inferred
DotClosure
BindingPatternAst::Pack
CallableImplementationTail
```

The `lang_syntax` and `lang_cli` package versions, and
`lang_syntax::VERSION`, advance from `0.2.0` to `0.5.0`. The old v0.4 note that
the crate version remained tied to v0.2 remains true for that historical
checkpoint; it is not a statement about the amended implementation.

## 3. Lexer amendment

`Symbol::Ellipsis` is added as a structural token. Maximal munch is:

```text
.       -> Dot
..      -> DotDot
...     -> Ellipsis
....    -> Ellipsis Dot
.....   -> Ellipsis DotDot
```

`Ellipsis` is not an operator spelling. The lexer assigns no pack, spread,
variadic, ABI, or runtime meaning.

`default`, `delete`, and strategy names remain `Name` tokens. `[[strategy]]`
remains four ordinary bracket tokens plus one `Name`; its meaning exists only
in the closure-tail strong context.

## 4. Parser amendment

The amended callable tail is:

```text
CallableImplementationTail
  ::= "=>" Block
   |  "=>" Name Block
   |  "=>" "default"
   |  "=>" "delete"
   |  "=>" "(" StringLiteral ")" "delete"
   |  "[[" Name "]]" Block
   |  Block
```

The amended capture clause is:

```text
CaptureClause ::= "[" CaptureItem ("," CaptureItem)* "]"

CaptureItem
  ::= PolicySpec "let" BindingCore "=" Expr
   |  "let" BindingCore "=" Expr
   |  BindingCore "=" Expr
   |  Expr
```

The first three alternatives are explicit let-shaped captures. `let` may be
omitted only when no policy needs it as a P1 anchor. Alias `===` remains a
form-level declaration and is not imported into capture items. The final
`Expr` alternative is source-preserving shorthand and becomes valid only if
normalization can infer exactly one binding name.

Placement is determined by the delimiter:

```text
no "=>" -> InPlace
"=>"    -> Ordinary
```

Head presence does not determine placement. `[[strategy]]` adds named strategy
metadata without changing in-place placement. Capture lists remain unavailable
to in-place closures.

Parenthesized Product-versus-head classification uses one closure-head
continuation predicate. Its strategy alternative requires the complete local
tail shape `[[Name]] {`:

```text
:
->
=>
{
head-clause
[[Name]] {
```

The capture slot remains open after a DeduceList. Therefore
`<T> [[cap] => { cap }] () => { ... }` parses `[[cap] => { cap }]` as the
capture clause, while `<T> [[strategy]] { ... }` has the complete strategy-tail
shape and bypasses capture parsing. A leading `[[` becomes a recovery-only
strategy candidate only after a parameter clause, call policy, return clause,
or head clause has independently closed the capture slot. Deduce alone is not
sufficient.

The recovery candidate cannot classify an ordinary Product or disable a
bracket-call suffix. Ordinary atom and operator postfix parsing therefore
continues to accept:

```lang
obj[[cap] => { cap }]
()[[cap] => { cap }]
```

The complete annotation recognizer, plus the recovery-only candidate after a
proven post-capture head component, prevents strategy recovery from stealing
capture expressions.

After `=>`, the parser selects by complete local shape. `Name Block` is tested
before the bare contextual names, so `=> default { ... }` and
`=> delete { ... }` are named strategy bodies; only a `default` or `delete`
not followed by a block selects `Defaulted` or `Deleted`.

The delete-message alternative is intentionally narrower than the historical
surface:

```text
v0.2/v0.8 historical surface   => (message_expr) delete
v0.5 amended surface           => (StringLiteral) delete
```

This is a deliberate source-language contraction, not merely an AST storage
change. A delete message is compiler diagnostic text fixed in source; it is
not an expression to evaluate. Consequently `=> (reason) delete` is invalid,
while `=> ("reason") delete` remains valid.

## 5. Raw AST amendment

The amended nodes are:

```text
ClosureAst {
  placement: ClosurePlacementAst,
  head: Option<FnHeadPrefixAst>,
  body: ClosureBodyAst,
  span: Span
}

ClosurePlacementAst = InPlace | Ordinary

ClosureBodyAst
  = Block
  | NamedBlock
  | Defaulted
  | Delete

AtomKind::DotClosure { selector }

BindingPatternAst::Pack { inner, span }
```

Canonical Pattern sequences may also contain a Pack as a direct child:

```text
PatternSequence ::= PatternTerm*
PatternTerm     ::= "..." PatternPrimary | PatternPrimary
```

The prefix binds exactly one following primary, so `a ...x b` preserves
`Sequence[a, Pack(x), b]`. A grouped/product primary may be used for a
compound operand. Raw `Pack(Pack(x))` is preserved; the parser does not decide
whether the two nodes share a normalized structural level.

Invalid callable tails produce an error atom. They never recover as an empty
`ClosureBodyAst::Block`.

## 6. Normalized handoff amendment

Normalized closure placement and provenance are independent:

```text
NormClosure {
  placement: NormClosurePlacement,
  head,
  body,
  origin
}

NormClosurePlacement = InPlace | Ordinary

NormOrigin::Generated { rule, span }
```

A `.name`-generated closure therefore has:

```text
placement = InPlace
origin.rule = DotClosureLowering
```

`Generated` is not a placement variant.

This is still an AST/Normalized-AST carrier. Normalization does not materialize
it as a callable value or allocate a capture environment. Only a later
explicit binding or call consumer may perform closure materialization.

Capture normalization removes the raw explicit/inferred split:

```text
NormCapture {
  slot: NormBindingSlot,
  initializer: NormExpr,
  origin: NormOrigin
}
```

For shorthand `[E]`, normalization computes the set of distinct free bare
names in `N(E)` whose occurrence is not the callable target of its direct
`Call`. Exactly one name `n` elaborates to `[let n = E]`; zero or multiple
names produce a retained normalized inference error. Locally bound names in
nested closures, parameters, and local lets do not participate. Capture
initializers are simultaneous: each is interpreted in the environment before
the capture clause.

Canonical sequences containing Pack normalize into `NormPattern::Sequence`
with `NormPattern::Pack` children. Pack never enters `NormSkeleton`.

A compound Pack operand remains structural:

```text
...(a, b)
  -> NormPattern::Pack(
       NormPattern::Product[Binder(a), Binder(b)]
     )
```

Pack-operand binding context propagates through the Product. For later overload
specificity, each written inner node is projected into the pack evidence class,
so the example contributes evidence as `...a` and `...b`. This does not create
two `NormPattern::Pack` nodes, and captured input length never adds specificity.

`normalize_program` remains available for diagnostic dumps and recovery
inspection. The downstream build handoff is:

```text
normalize_and_validate_patterns
  -> PatternValidatedNormProgram
  |  PatternInvalidNormProgram
```

Only `PatternValidatedNormProgram` may enter declaration harvesting. This
makes the one-pack-per-normalized-level rule an enforced handoff rather than
an optional caller convention. It is the sole authority for the normalized
level invariant: the parser constructs every syntactically formed
`BindingPatternAst::Pack` and diagnoses only local syntax such as a missing
inner Pattern. It does not count packs or claim knowledge of normalized
structural levels. The certificate proves only Pattern-layer invariants;
recovered `NormExpr::Error` nodes require a separate future recovery-free
certificate and are not ruled out by this type.

## 7. Non-semantic boundary

This amendment does not implement:

```text
name or type resolution
closure materialization
capture-environment layout or admissibility
named strategy execution
default body generation
pack matching execution
runtime spread/unpack
ABI pack classes
overload reopening
```

The weak lexer, parser-owns-shape rule, value/Pattern separation, and
non-semantic Normalized AST boundary remain intact.
