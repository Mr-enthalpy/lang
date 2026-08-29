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
| DeduceList telescope and exact `HoleBinderId` references | Normalized binding correction | A string-only `HoleRef` cannot identify its declaration or define forward/self/duplicate behavior in nested let-shaped slots. |
| First written formal = callable self-position | Normalized formal-frame correction | Invocation already injects callable self as slot 0. Treating the first written Pattern as an explicit user argument split one position into two incompatible meanings and made generated receiver helpers consume their own callable object as the business receiver. |
| Empty DeduceList selects a binderless Pattern; atomic pipe-branch shorthand uses it | Hard binding-shape correction (PR #100) | `let <> P` must preserve binder absence while `let P` remains the ordinary singleton binder. Lowering `|> P { ... }` through `(_ P)` fabricated a wildcard position and changed Pattern structure. |
| Expression `PolicySpec let PipeExpr` / `PolicyLetAst` | New expression-context syntax amendment (PR #102) | A result-Policy demand must exist before the operand root call is selected. An ordinary value-side `const`/`mut` call occurs too late and cannot preserve that boundary. |

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
34 DiagnosticCode variants
ClosureAst { placement, head, body, span }
CaptureItemAst::Explicit | CaptureItemAst::Inferred
DotClosure
BindingPatternAst::Pack
CallableImplementationTail
ExprKind::PolicyLet(PolicyLetAst)
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

An explicit empty DeduceList is valid in a strong binding context:

```text
let <> P
  -> DeduceListAst { binders: [] }
  -> BinderPresence = Absent
  -> Pattern = P
```

It is distinct from the ordinary singleton binder `let P` and from a real
wildcard position `let _ P`.

The current atomic pipe-branch shorthand reuses that binding shape:

```text
|> P { body }
  == |> (<> P) { body }
```

The resulting closure is headed and `InPlace`. Explicit `(_ P)` remains a
different Product head containing a real wildcard Pattern position. `<` and
`>` remain the existing two structural tokens; no `<>` token is added.

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

### 4.1 Expression-level Policy context

The amended expression grammar adds one low-precedence term former:

```text
Expression          ::= PolicyLetExpression | PipeExpression
PolicyLetExpression ::= PolicySpec "let" PipeExpression
```

Its right operand covers the complete following pipe expression:

```text
P let a |> f       == P let (a |> f)
(P let a |> f) |> g
(P let a) |> f
```

The last two forms are distinct. Parentheses close the local Policy context.
At a form start, a depth-aware non-consuming classification preserves the
existing declaration path when the material after `PolicySpec let` contains a
top-level `=`. A top-level `===` continues to select the frozen alias
Raw-AST path only; no alias semantics are restored. Without either delimiter,
the form is an expression `PolicyLet`. In a pure expression context,
`PolicySpec let` is always `PolicyLet`, and a following top-level `=`/`===` is
invalid expression material rather than a nested declaration.

`const`, `plain`, `mut`, and `let` remain weak-lexer `Name` tokens. A missing
operand emits `ExpectedPolicyLetOperand` and preserves an error operand.
Consequently a former inferred-capture expression ending in the exact strong
shape `P let`, such as `[x let]`, is now a malformed PolicyLet rather than a
two-name capture-inference expression.

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
params = [generated self, val: T, ...args]
```

`Generated` is not a placement variant.

For every ordinary or in-place closure, the first written formal is the
explicit Pattern for invocation-frame slot 0. Its actual caller object is
passed implicitly and never belongs to the explicit call-site Product. Written
formals after it consume that Product in order. The spelling `self` is not
reserved. A closure with no written formal still has an unbound semantic self
slot. Prefix-negative, dot-closure, and double-dot generated helpers therefore
write a generated self formal before their `val` receiver formal.

The Raw/Normalized carrier does not decide the caller's type. Standalone
function-object materialization and associated `()` installation share this
positional rule but may supply different receiver types.

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

A parenthesized Product is still preserved in Raw AST:

```text
...(a, b)
  -> Raw Pack(Product[a, b])
```

It is not, however, a valid semantic structured Pack operand. Ordinary P
normalization flattens a bare Product boundary, and Pack cannot reify that
boundary again. The post-normalization Pattern handoff therefore rejects this
shape. A future ordered matcher may admit an operand whose P-normal form keeps
a stable top mode:

```text
...((a, b) pair)
```

At an unordered named level, only a whole-remainder binder/discard (including
a transparent let-shaped wrapper) is admissible. Every Pack supplies one
outward specificity node at its containing level. Captured width and internal
node count never become multiple same-level EP nodes; structured evidence, when
legal, remains below the stable operand head.

> Superseded identity rule: the following paragraph records the v0.5-A
> telescope substrate. v0.6 retains left-to-right telescope order but replaces
> active-ancestor uniqueness with same-`PatternRoot` uniqueness and permits a
> new independent Pattern root to shadow inherited names. The persistent
> identity is qualified by `SemanticOwnerId`.

DeduceLists normalize as left-to-right telescopes. Raw AST preserves lexical
scope shape, spelling, and provisional roles. A post-structural
alpha-normalization pass allocates fresh lexical ordinals and makes each
`HoleRef` target that exact `HoleBinderId`; source spans remain provenance, not
semantic identity. A declaration annotation sees inherited and preceding
binders, not the declaration itself or following binders. Same-list and
active-ancestor duplicates are retained for diagnostics but do not shadow or
extend the active environment. `_` normalizes as an anonymous hole rather than
a named reference.

A callable head DeduceList scopes capture slots and initializers, parameters,
call policy, return slot, head clauses, and the complete body. Nested callables
inherit that environment before adding their own telescope.

BindingSlot order is normative: policy is normalized in the inherited hole
environment, then the local DeduceList extends the environment for Pattern,
annotation, and initializer. Generated receiver holes use a hygienic
generated-syntax key rather than source spelling, so a generated display name
cannot redeclare or capture an active source hole.

`HoleBinderId` is local to an `AlphaOwner`: the complete normalized tree
produced by one root `normalize_program` invocation. Nested closure-body
`NormProgram` nodes share the same owner and ordinal space. Norm alpha
conversion rewrites Pattern/policy occurrences; value-side `NormExpr::Name`
and ordinary navigation-name components remain unresolved for a later
resolved-symbol pass.

The scope extension does not alter return BindingSlot syntax. `-> r` binds the
returned object to the explicit symbol `r`; `-> r: A` adds the postfix
annotation Pattern `A`, and `-> _: A` leaves the constrained result anonymous.
Whether `r` denotes an ordinary value or a type/Pattern object follows the
callable's result rank rather than parser classification. `-> A r` remains an
extraction Pattern rather than a prefix type annotation.

`normalize_program` remains available for diagnostic dumps and recovery
inspection. The downstream build handoff is:

```text
normalize_and_validate_patterns
  -> PatternValidatedNormProgram
  |  PatternInvalidNormProgram
```

Only `PatternValidatedNormProgram` may enter declaration harvesting. This
makes the normalized Pattern rules an enforced handoff rather than an optional
caller convention. It is the sole authority for pack cardinality,
non-canonical bare-Product Pack operands, and same-`PatternRoot` duplicate holes
under the superseding v0.6 owner amendment:
the parser constructs every syntactically formed
`BindingPatternAst::Pack` and diagnoses only local syntax such as a missing
inner Pattern. It does not count packs or claim knowledge of normalized
structural levels. The certificate proves only Pattern-layer invariants;
recovered `NormExpr::Error` nodes require a separate future recovery-free
certificate and are not ruled out by this type.

### 6.1 PolicyLet preservation

Raw `ExprKind::PolicyLet(PolicyLetAst)` normalizes to a dedicated value-side
node:

```text
NormExpr::PolicyLet {
  policy: NormPolicySpec,
  operand: NormExpr,
  origin: Generated(PolicyLetPreserve)
}
```

The policy is alpha-normalized under the inherited hole environment, then the
operand is normalized under that same environment. The node introduces no
binder, place, declaration, or new hole scope. It is not lowered to
`NormDecl`, an ordinary `const`/`mut` call, or a hidden temporary binding.
Value-side structural visitors recurse through its operand while retaining the
wrapper. Call/target/struct semantic consumers treat the wrapper as an opaque
unsupported boundary until a later Policy elaborator exists. In a Pattern or
annotation position the value-side node becomes explicit `PatternUnsupported`
rather than a value call.

## 7. Non-semantic boundary

This amendment does not implement:

```text
name or type resolution
closure materialization
capture-environment layout or admissibility
resolved automatic capture analysis
named strategy execution
default body generation
stable Pattern-head discovery and general pack matching execution
runtime spread/unpack
ABI pack classes
overload reopening
PolicyLet result-demand or Policy-cast execution
```

The weak lexer, parser-owns-shape rule, value/Pattern separation, and
non-semantic Normalized AST boundary remain intact.
