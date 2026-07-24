# Raw AST Contract v0.5

Status: current amended Raw AST and validated-normalization input contract.

This contract is defined as:

```text
closed Raw AST v0.2 freeze
+ Frontend Semantic Amendment v0.5-A
= current Raw AST contract v0.5
```

The v0.1/v0.2/v0.3 documents are historical snapshots and are not edited to
retroactively contain this surface. The complete change classification and
migration boundary are recorded in
[`frontend-semantic-amendment-v0.5-a.md`](frontend-semantic-amendment-v0.5-a.md).

## 1. Pipeline and boundary

```text
source text
  -> weak lexer
  -> Raw AST + syntax diagnostics
  -> non-semantic normalization + capture binding elaboration
  -> normalized Pattern validation
  -> PatternValidatedNormProgram
```

Raw AST preserves syntax and recovery. It does not resolve names, check types,
select overloads, materialize closures, execute callable bodies, or interpret
Pattern packs.

## 2. Lexical contract

Names remain weak `Name` tokens. In particular:

```text
default
delete
strategy identifiers
meta / compile / seal / runtime
```

are not lexer keywords.

There are 20 structural `Symbol` variants in the amended implementation.
`Ellipsis` is the only v0.5-A addition.

Dot-family maximal munch is:

| Source | Tokens |
|---|---|
| `.` | `Dot` |
| `..` | `DotDot` |
| `...` | `Ellipsis` |
| `....` | `Ellipsis`, `Dot` |
| `.....` | `Ellipsis`, `DotDot` |
| `.name` | `Dot`, `Name` |
| `..name` | `DotDot`, `Name` |
| `...name` | `Ellipsis`, `Name` |

`Ellipsis` is structural and cannot be reinterpreted as an ordinary operator
spelling.

## 3. Strong parser contexts

The parser may recognize ordinary `Name` and bracket tokens specially only in
documented strong contexts.

The callable implementation tail grammar is:

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

The closure-head continuation lookahead uses the complete strategy shape:

```text
starts_closure_head_continuation
  ::= ":"
   |  "->"
   |  "=>"
   |  "{"
   |  HeadClause
   |  CompleteStrategyTail

CompleteStrategyTail
  ::= "[[" Name "]]" "{"
```

Only `CompleteStrategyTail` may classify a Product as a closure
parameter head. A second recognizer for a leading `[[` is recovery-only and
may be used only after another head component has independently established
the callable-tail context.

After a deduce list, the capture slot remains open. Capture parsing is entered
for `[` unless the complete strategy tail is present:

```text
"["
and not CompleteStrategyTail
```

A leading `[[` is a malformed-strategy recovery candidate only after a
parameter clause, call policy, return clause, or head clause has independently
closed the capture slot. Deduce alone does not close it.

Thus all of these are closure heads:

```lang
() [[s]] { value }
() : compile [[s]] { value }
() require C [[s]] { value }
<T> [[s]] { value }
<T>() [[s]] { value }
<T>() -> r [[s]] { value }
```

Ordinary atom/operator suffix parsing never globally excludes a leading
`[[`. These remain bracket calls whose argument is a capture closure:

```lang
obj[[cap] => { cap }]
()[[cap] => { cap }]
(a + b)[[cap] => { cap }]
```

After `=>`, implementation selection examines the full local tail:

```text
Block                         -> Block
"(" StringLiteral ")" delete -> Delete(message)
Name Block                    -> NamedBlock
default without Block         -> Defaulted
delete without Block          -> Delete
other Name without Block      -> Error
```

Consequently `default` and `delete` remain weak names and may be named
strategies when followed by a block.

The parenthesized delete-message form is a deliberate contraction from the
historical `=> (message_expr) delete` surface to
`=> (StringLiteral) delete`. Delete messages are static compiler diagnostic
text, not evaluated expressions. The v0.2/v0.8 documents remain historical
records of the broader accepted shape; v0.5 intentionally rejects
`=> (reason) delete`.

## 4. Closure Raw AST

Closure placement, head presence, and implementation are orthogonal:

```text
ClosureAst {
  placement: ClosurePlacementAst,
  head: Option<FnHeadPrefixAst>,
  body: ClosureBodyAst,
  span: Span
}

ClosurePlacementAst
  = InPlace
  | Ordinary
```

Source mapping:

| Source | Placement | Head | Body |
|---|---|---|---|
| `{ ... }` | InPlace | none | Block |
| `Head { ... }` | InPlace | present | Block |
| `Head [[s]] { ... }` | InPlace | present | NamedBlock |
| `Head => { ... }` | Ordinary | present | Block |
| `Head => s { ... }` | Ordinary | present | NamedBlock |
| `Head => default` | Ordinary | present | Defaulted |
| `Head => delete` | Ordinary | present | Delete |
| `Head => ("message") delete` | Ordinary | present | Delete(message) |

`Head -> r name { ... }` keeps `name` in the return extraction Pattern.
`[[name]]` is the explicit named-strategy escape. The parser does not
backtrack.

In-place closures cannot have capture lists. Invalid capture/tail forms become
`ErrorAst`; an error cannot be represented as a valid empty Block.

### 4.1 Capture items are let-shaped

```text
CaptureClause ::= "[" CaptureItem ("," CaptureItem)* "]"

CaptureItem
  ::= PolicySpec "let" BindingCore "=" Expr
   |  "let" BindingCore "=" Expr
   |  BindingCore "=" Expr
   |  Expr
```

The Raw AST preserves whether a capture was explicit or shorthand:

```text
CaptureItemAst
  = Explicit {
      slot: BindingSlotAst,
      initializer: ExprAst
    }
  | Inferred {
      initializer: ExprAst
    }
```

`BindingSlotAst` is the same let-shaped structure used by declarations and
formal parameters. `[let x = E]` and `[x = E]` are equivalent; a policy prefix
requires the `let` anchor, for example `[runtime let x = E]`. Alias `===`
remains form-level and is rejected after capture `let`.

The old `[E]` surface is retained only as inferable shorthand. It normalizes to
`[let n = E]` exactly when the normalized expression contains one distinct
free bare name `n` in non-call-target position. Occurrences are deduplicated;
a name may also appear as a call target without losing its non-call evidence.
Zero or multiple candidates produce a retained normalized inference error.
Nested binders do not leak into this calculation.

All initializers in one capture clause see the enclosing environment before
the clause; captures are simultaneous, not a sequential let block.

## 5. Dot closure and member forms

The Raw AST contains:

```text
AtomKind::DotClosure { selector }
```

for independent `.name`.

Normalization alone defines:

```text
.name
  -> (val: T, ...args) {
       (val, args) |> name::T
     }
```

After this one lowering, the result is an ordinary expression. No pipe,
product, or legality-repair rule may inspect `DotClosureLowering` provenance to
change binding.

The normalized result is a closure carrier, not an already materialized
callable value. Only a later explicit binding or call consumer may materialize
it; normalization and arbitrary expression composition do not.

Compact `E.name` mechanically lowers through `E |> .name`. Direct
`E..name(product)` remains a separate member-call sugar.

## 6. Pattern remainder

Pattern-side:

```text
BindingPatternAst::Pack { inner, span }
NormPattern::Pack { inner, origin }
```

is permitted in every let-shaped binding slot. It has no RHS spread meaning,
pack type, ABI class, or unpack operator.

Ellipsis is also a direct canonical Pattern Sequence child:

```text
PatternSequence ::= PatternTerm*
PatternTerm     ::= "..." PatternPrimary | PatternPrimary
```

It binds only the immediately following primary:

```text
a ...x b     -> Sequence[a, Pack(x), b]
...(x, y)    -> Raw Pack(Product[x, y])
```

Canonical sequences with Pack normalize to `NormPattern::Sequence`; Pack is
never hidden in `NormSkeleton`.

Raw preservation does not make `...(x, y)` a valid structured match. A bare
Product has no stable top mode after P normalization, so the normalized
Pattern validator rejects it. An explicitly headed candidate such as
`...((x, y) pair)` may survive structurally for later ordered matching. At an
unordered layer, only a whole-remainder binder/discard is admissible. Every
Pack contributes one outward specificity node at the containing level;
captured length and internal nodes do not become additional same-level Pack
evidence.

Each normalized structural level contains at most one direct pack. Product and
Sequence levels apply the same rule, and nested levels validate independently.
The parser constructs every syntactically formed Pack, including multiple
direct packs and directly nested packs. It diagnoses only local syntax such as
a missing inner Pattern. The post-normalization Pattern validator is the sole
authority for the normalized-level cardinality rule.

## 7. Normalized closure contract

```text
NormClosure {
  placement: NormClosurePlacement,
  head: Option<NormClosureHead>,
  body: NormClosureBody,
  origin: NormOrigin
}

NormClosurePlacement
  = InPlace
  | Ordinary
```

Normalized captures are uniformly explicit bindings:

```text
NormCapture {
  slot: NormBindingSlot,
  initializer: NormExpr,
  origin: NormOrigin
}
```

Generated provenance is stored only in:

```text
NormOrigin::Generated { rule, span }
```

It never replaces placement. Generated dot/member/prefix helper closures retain
their in-place placement while separately reporting their lowering rule.

## 8. Pattern-validated normalized handoff

`normalize_program` produces inspectable normalized structure, including
invalid recovered Patterns. It is not the semantic/build handoff.

The handoff is:

```text
normalize_and_validate_patterns(ProgramAst)
  -> Result<PatternValidatedNormProgram, PatternInvalidNormProgram>
```

The validator traverses every Pattern-bearing location:

```text
top-level and local let
formal parameters
return slots
deduce and binding annotations
nested Product and Sequence
expression-carried closure bodies
```

Only `PatternValidatedNormProgram` may be harvested into the build world. Its
current proof scope is exactly:

```text
one Pack per normalized Product/Sequence level
no bare Product Pack operand
no duplicate DeduceList hole in an active telescope
```

It does not prove ordered/unordered Pack applicability, stable Pattern-head
identity, complete matching support, that the parser emitted no diagnostics,
or that the program contains no recovered `NormExpr::Error`; consumers that
require those properties need distinct resolved-stage checks or certificates.

## 9. Deduce telescope and capture dependency boundary

Normalized DeduceLists are left-to-right telescopes:

```text
Ti in <A1:T1, ..., Ai:Ti> sees inherited holes and A1..A(i-1)
Ai is not visible in Ti
later binders are not visible in Ti
active hole names cannot be redeclared or shadowed
```

Raw AST carries lexical structure, surface spelling, and provisional canonical
roles. Normalized alpha conversion, not the parser and not a source span,
allocates each `HoleBinderId`; every named `HoleRef` then targets one exact
owner-local ordinal identity. The display spelling and span are provenance
data; IDs from distinct `NormProgram` owners are not directly comparable.
Generated receiver holes use a hygienic generated key rather than their
display spelling. `_` is an anonymous hole and targets no declaration.

Within a BindingSlot, the leading policy is processed under inherited holes
before the local DeduceList extends the environment for Pattern, annotation,
and initializer. Alpha conversion binds Pattern/policy occurrences only;
ordinary value-side names and navigation components remain unresolved.

A callable head DeduceList remains active through capture clauses and
initializers, parameters, call policy, return slot, head clauses, and the
complete body. Nested callables inherit the active environment and extend it
with their own telescope. Ordinary value binders do not shadow hole identity.

Normalized source capture items are explicit let-shaped bindings. `[x]` is
explicit shorthand for `[let x = x]` with an unwritten policy domain; it is not
automatic const capture. Future resolved free-reference analysis may create
separate `ImplicitConst` capture requirements. Such requirements are abstract
dependencies, not `self` fields or layout decisions. In-place closures create
no capture set, may resolve outer reads at the embedding layer, and may not
directly write an outer place.

Explicit-navigation/export checking and automatic capture remain resolved
semantics, not Raw-to-Norm work. External navigation searches the export view;
internal navigation searches the complete namespace view. A navigable exported
value supplies the const projection for `ImplicitConst`; ordinary external call
references normally inhabit the same external-symbol problem domain. This does
not imply an implementation dependency on call resolution. Explicit and
automatic capture remain distinct dependency declarations even when they
resolve to the same source symbol; only later layout may coalesce equivalent
storage while preserving binder, policy, and provenance.

## 10. Diagnostics

The amended implementation has 33 `DiagnosticCode` variants. The v0.2 frozen
diagnostic inventory remains 32 because it is a historical snapshot.

The additional code is:

```text
MultiplePackPatternsAtSameLevel
```

This code is reserved for a consumer that projects normalized Pattern
validation failures into the syntax diagnostic transport. The parser does not
emit it or independently count packs. Every diagnostic retains a span.
Recovery remains error tolerant, but no recovery path may replace invalid
callable syntax with a valid executable body.

## 11. Non-goals

This contract does not define:

```text
name resolution
Pattern-head resolution
type or kind checking
closure materialization
capture-environment analysis
named strategy execution
default implementation generation
pack matching execution
overload execution
runtime evaluation
code generation
```
