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
  -> non-semantic normalization
  -> normalized Pattern validation
  -> ValidatedNormProgram
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

The unique closure-head continuation lookahead is:

```text
starts_closure_head_continuation
  ::= ":"
   |  "->"
   |  "=>"
   |  "{"
   |  HeadClause
   |  "[[" Name "]]"
```

Segment, operator-expression, binding-slot, call-policy, return, and
head-clause parsing reuse the same `[[strategy]]` recognizer. They must not
maintain independent approximate bracket-boundary sets.

After a deduce list, capture parsing is entered only for:

```text
"[" and not starts_overload_strategy_annotation
```

Thus all of these are closure heads:

```lang
() [[s]] { value }
() : compile [[s]] { value }
() require C [[s]] { value }
<T> [[s]] { value }
<T>() [[s]] { value }
<T>() -> r [[s]] { value }
```

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

Each normalized structural level contains at most one direct pack. Product and
Sequence levels apply the same rule, and nested levels validate independently.

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

Generated provenance is stored only in:

```text
NormOrigin::Generated { rule, span }
```

It never replaces placement. Generated dot/member/prefix helper closures retain
their in-place placement while separately reporting their lowering rule.

## 8. Validated normalized handoff

`normalize_program` produces inspectable normalized structure, including
invalid recovered Patterns. It is not the semantic/build handoff.

The handoff is:

```text
normalize_and_validate(ProgramAst)
  -> Result<ValidatedNormProgram, InvalidNormProgram>
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

Only `ValidatedNormProgram` may be harvested into the build world.

## 9. Diagnostics

The amended implementation has 33 `DiagnosticCode` variants. The v0.2 frozen
diagnostic inventory remains 32 because it is a historical snapshot.

The additional code is:

```text
MultiplePackPatternsAtSameLevel
```

Every diagnostic retains a span. Recovery remains error tolerant, but no
recovery path may replace invalid callable syntax with a valid executable body.

## 10. Non-goals

This contract does not define:

```text
name resolution
Pattern-head resolution
type or kind checking
closure materialization
capture analysis
named strategy execution
default implementation generation
pack matching execution
overload execution
runtime evaluation
code generation
```

