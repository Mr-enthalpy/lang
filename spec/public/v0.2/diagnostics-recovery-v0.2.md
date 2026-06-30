# Diagnostics and Recovery v0.2

## 1. Scope

This document defines the public diagnostics and recovery behavior of the
current v0.2 Raw AST frontend. It covers:

```text
lex(source text) → tokens + lexical diagnostics
parse(tokens)    → Raw AST + parser diagnostics + ErrorAst recovery nodes
```

This document assumes:

- lexical syntax from `spec/public/v0.2/lexical-syntax-v0.2.md`
- concrete syntax from `spec/public/v0.2/concrete-syntax-v0.2.md`
- Raw AST preservation from `spec/public/v0.2/raw-ast-frozen-surface-v0.2.md`

This document does not define:

- semantic errors (name resolution, type errors, kind errors)
- operator lookup or overload resolution errors
- alias target resolution errors
- canonical matching failures
- closure capture analysis errors
- lifetime, ownership, NLL, or drop errors
- interpretation or code generation errors

## 2. Diagnostic model

Each diagnostic carries:

```text
Diagnostic {
    code: DiagnosticCode,
    message: String,
    span: Span
}
```

- `code` is the stable machine-readable category.
- `message` is human-readable and may be less stable than `code`.
- `span` is the primary source span, referring to the normalized (LF) source text.

Diagnostics are emitted alongside tokens (lexer diagnostics) or alongside Raw
AST nodes (parser diagnostics). They do not by themselves imply semantic
invalidity. A syntactically accepted form with a diagnostic is still preserved
as Raw AST.

## 3. DiagnosticCode inventory

The current `DiagnosticCode` enum contains 32 variants. Each is listed below
with its category, typical trigger, and stability status.

| Code | Category | Typical trigger | Recovery | Status |
|---|---|---|---|---|
| `InvalidToken` | Lexer | Unrecognized byte sequence | Emit `Invalid` token and continue | Guaranteed |
| `UnclosedString` | Lexer | String literal reaches EOF or newline before closing boundary | Emit `StringLiteral` token spanning unterminated text; continue | Guaranteed |
| `UnclosedComment` | Lexer | Block comment reaches EOF before `*/` | Treat comment as closed at EOF; emit trivia token | Guaranteed |
| `InvalidNumericLiteral` | Lexer | Malformed numeric literal (invalid separator position, missing radix digits, empty hex float exponent) | Emit diagnostic; preserve invalid material in token text; continue | Guaranteed |
| `UnexpectedToken` | Parser | Token does not match any expected production | Skip to synchronization point or consume offending token; continue | Guaranteed |
| `ExpectedName` | Parser | Context requires a `Name` token but current token is not a name | Insert `ErrorAst` and continue | Guaranteed |
| `ExpectedColon` | Parser | Legacy/simple-binder colon expectation | Reserved; not currently emitted by parser | Reserved / not-currently-emitted |
| `ExpectedBindingAnnotation` | Parser | Binding slot has `:` but no annotation before context boundary | Continue at boundary; use `BindingAnnotationAst::Error` | Guaranteed |
| `ExpectedEqual` | Parser | Let binder and optional `with` not followed by `=` | Skip to form boundary; use error expression as initializer | Guaranteed |
| `EmptyPipeSegment` | Parser | `\|>` at start of pipe expression or two consecutive `\|>` | Insert `ErrorAst` as missing segment body | Guaranteed |
| `ExpectedNameAfterDot` | Parser | `.` suffix is followed by a non-`Name` token | Consume `.` and stop suffix folding | Guaranteed |
| `ExpectedNameAfterDoubleDot` | Parser | `..` suffix is followed by a non-`Name` token | Consume `..` and stop suffix folding | Guaranteed |
| `ExpectedProductAfterDoubleDotName` | Parser | `..` selector is not followed by a product form | Consume `..` and selector; no partial `DoubleDotSugar` | Guaranteed |
| `UnclosedParen` | Parser | `(` without matching `)` by form boundary or EOF | Insert implicit `)` at boundary; preserve parsed content | Guaranteed |
| `UnclosedBracket` | Parser | `[` without matching `]` by form boundary or EOF | Insert implicit `]` at boundary; preserve parsed content | Guaranteed |
| `UnclosedBrace` | Parser | `{` without matching `}` in a delimiter-owned context | Insert implicit `}` at boundary; preserve parsed content | Guaranteed |
| `InvalidDeduceList` | Parser | Malformed deduce list (missing name, trailing comma, unclosed, missing annotation) | Preserve parsed binders where possible; some malformed annotation positions produce an error expression, other cases recover without adding a dedicated `ErrorAst` | Guaranteed |
| `InvalidCanonicalSkeleton` | Parser | Malformed canonical skeleton in extraction context | Skip to context boundary; insert `ErrorAst` | Guaranteed |
| `InvalidClosureHead` | Parser | `FnHeadPrefix { ... }` without `=>`, headless pipe branch body, malformed head clause | Replace malformed clause with `ErrorAst`; preserve recoverable parts | Guaranteed |
| `TopLevelComma` | Parser | Comma at top level of a form outside any product or group | Consume comma; no additional AST structure | Guaranteed |
| `UnusedClosureAst` | Parser | Exists for optional / non-guaranteed closure-recovery reporting | Closure AST still produced; callers must not rely on this code being emitted | Optional / not-guaranteed-emitted |
| `InvalidOperatorExpression` | Operator | Malformed or unsupported operator syntax (missing operand, unsupported prefix) | Best-effort operator expression node or `ErrorAst` | Guaranteed |
| `ChainedNonAssociativeOperator` | Operator | Ungrouped chain of non-associative operators (`a < b < c`) | Best-effort operator sugar shape; continue | Guaranteed |
| `InvalidNavComponent` | Operator | Invalid navigation component (operator as outer component, grouped expression as innermost) | Local error component; preserve outer components | Guaranteed |
| `ExpectedAliasTarget` | Alias | After `===`, RHS is absent or cannot start EntityRef; also covers missing component after `::` when alias RHS reaches a hard boundary | Recovery depends on detection point: at a hard boundary the parser inserts an error component without consuming the boundary; for a non-boundary invalid token it may consume the token and recover to the form boundary | Guaranteed |
| `InvalidAliasBinder` | Alias | Binder token is neither `Name` nor operator-eligible | Reserved; not currently emitted by parser | Reserved / not-currently-emitted |
| `InvalidAliasPosition` | Alias | Alias-shaped token sequence appears in non-form position | Emit diagnostic; recover to enclosing delimiter or form boundary | Guaranteed |
| `InvalidEntityRef` | Alias | Malformed non-boundary EntityRef components (grouped innermost component, operator as outer component) | Preserve parsed segments; replace malformed continuation with error component | Guaranteed |
| `UnexpectedAliasRhsExpression` | Alias | Valid EntityRef parsed but next token is not a form boundary | Consume tokens until form boundary | Guaranteed |
| `ReturnRequiresValue` | Parser | Bare `return;` without a value expression. | `self.cursor.bump_non_trivia()` + `consume_form_boundary()` + ErrorAst recovery. | guaranteed emitted |
| `StatementAfterTerminalBlockForm` | Parser | A form appears after a terminal block form (tail value or return event) before `}`. | ErrorAst wrapping + recovery to `}`. | guaranteed emitted |
| `ReturnExpressionNotAllowed` | Parser | Return-like syntax embedded in expression, group, pattern, let-initializer, annotation, call-argument, or operator context. | Diagnostic emitted at expression span; the expression is preserved. | guaranteed emitted |

## 4. Category overview

| Category | Count | Codes |
|---|---|---|
| Lexer | 4 | `InvalidToken`, `UnclosedString`, `UnclosedComment`, `InvalidNumericLiteral` |
| Parser | 20 | `UnexpectedToken`, `ExpectedName`, `ExpectedColon`, `ExpectedBindingAnnotation`, `ExpectedEqual`, `EmptyPipeSegment`, `ExpectedNameAfterDot`, `ExpectedNameAfterDoubleDot`, `ExpectedProductAfterDoubleDotName`, `UnclosedParen`, `UnclosedBracket`, `UnclosedBrace`, `InvalidDeduceList`, `InvalidCanonicalSkeleton`, `InvalidClosureHead`, `TopLevelComma`, `UnusedClosureAst`, `ReturnRequiresValue`, `StatementAfterTerminalBlockForm`, `ReturnExpressionNotAllowed` |
| Operator | 3 | `InvalidOperatorExpression`, `ChainedNonAssociativeOperator`, `InvalidNavComponent` |
| Alias | 5 | `ExpectedAliasTarget`, `InvalidAliasBinder`, `InvalidAliasPosition`, `InvalidEntityRef`, `UnexpectedAliasRhsExpression` |

## 5. Lexer diagnostics

### `InvalidToken`

The lexer encounters a byte sequence that does not form a valid token.
The lexer emits an `Invalid` token with an `InvalidToken` diagnostic and
continues lexing following bytes.

### `UnclosedString`

A string literal reaches the end of the file or a newline before a closing
boundary. The lexer emits an `UnclosedString` diagnostic and still produces
a `StringLiteral` token spanning the unterminated text.

### `UnclosedComment`

A block comment (`/* ... */`) reaches EOF without the depth returning to
zero. The lexer emits an `UnclosedComment` diagnostic and treats the comment
as closed at EOF, producing a block-comment trivia token.

### `InvalidNumericLiteral`

A malformed numeric literal is encountered during numeric scanning: invalid
digit separator position (adjacent to radix prefix, dot, or exponent marker;
doubled separator; trailing separator), missing digits after a radix prefix,
or empty hex float exponent. The lexer emits the diagnostic, preserves the
invalid material in the token text where possible, and continues.

## 6. General parser diagnostics and recovery

The parser is error-tolerant. When an error is detected:

1. A `Diagnostic` is emitted with a primary span.
2. An `ErrorAst` node is inserted at the recovery point.
3. Parsing continues from a reasonable resynchronization point (hard
   boundary `;`, `}`, EOF; matching delimiter; next form).

### `UnexpectedToken`

A token at the current parser position does not match any expected production.
The parser skips tokens until a synchronization point or consumes the
offending token and continues with later atoms where the surrounding
expression remains structurally recoverable.

### `ExpectedName`

A parser context requires a `Name` token but the current token is not a name.
The parser inserts an `ErrorAst` where the missing name-dependent node would
appear and continues.

### `ExpectedColon` — reserved

This diagnostic code exists in the `DiagnosticCode` enum and in the dump
surface but the current parser does not normally emit it. The binding
annotation parser handles `:` through optional-colon paths with
`ExpectedBindingAnnotation` / `ExpectedEqual` alternatives.
`ExpectedColon` is reserved / not-currently-emitted for the current
implementation.

### `UnclosedParen`, `UnclosedBracket`, `UnclosedBrace`

An opening delimiter is not matched by its closing counterpart by the end
of the current form or EOF. The parser inserts an implicit closing delimiter
at the form boundary and preserves whatever content was parsed.

### `TopLevelComma`

A comma appears at the top level of a form outside any product form or
parenthesized group. The parser emits the diagnostic and continues; the
comma is consumed but does not create additional AST structure.

## 7. Binding and annotation diagnostics

### `ExpectedBindingAnnotation`

A binding slot has `:` but no valid annotation expression before its
context-specific boundary (`=`, `with`, `,`, `)`, `=>`, `{`). The parser
continues at the boundary and uses `BindingAnnotationAst::Error`.

### `ExpectedEqual`

A let binder and optional `with` clause are not followed by `=`. The parser
skips to the current form boundary and uses an error expression as the
let initializer.

### `ExpectedName` in binding context

Used when a binding pattern, parameter binding, return binding, or navigation
name is missing. The parser inserts an `ErrorAst` in the binding position.

Binding diagnostics do not validate policy meaning, type validity, rank
validity, classifier validity, binding admissibility, or initializer type.

## 8. With-clause diagnostics

There is no dedicated `DiagnosticCode` for malformed `with` clauses.
Malformed `with` syntax (missing `{`, trailing comma, unclosed `{`, name-list
violations) uses general parser diagnostics — typically `UnexpectedToken`,
`UnclosedBrace`, or `InvalidClosureHead` in contexts where `with` is forbidden.

## 9. Product, group, and delimiter recovery

### `UnclosedParen`, `UnclosedBracket`, `UnclosedBrace`

These diagnostics cover group parsing, product expression parsing, product
extraction parsing, bracket product parsing for bracket-call sugar, and
body block parsing. Context-dependent recovery produces the best-effort
Raw AST (group, product, or body block) with whatever content was parsed.

### `TopLevelComma`

A comma at the top level of a form (outside any product or group) produces
`TopLevelComma`. The comma is discarded without creating additional AST
structure.

## 10. Pipe and segment diagnostics

### `EmptyPipeSegment`

A `|>` operator at the start of a pipe expression (no left operand), or
two consecutive `|>` operators with no segment between them. The parser
inserts an `ErrorAst` node as the missing segment body.

### `InvalidClosureHead` for headless pipe branch body

The incoming segment shape `x |> { ... }` is a headless in-place closure
in pipe-branch position. The parser emits `InvalidClosureHead` because
incoming pipe branch bodies require an explicit extraction head. The
headless in-place closure AST is still preserved as a segment element.

## 11. Atom suffix and navigation diagnostics

### `ExpectedNameAfterDot`

A `.` atom suffix is followed by a token that is not a valid selector
token (not a `Name`). The `.` is consumed and suffix folding stops.

### `ExpectedNameAfterDoubleDot`

A `..` atom suffix is followed by a token that is not a valid selector
token (not a `Name`). The `..` is consumed and suffix folding stops.

### `ExpectedProductAfterDoubleDotName`

A `..` selector is not followed by a product form. The `..` and selector
are consumed; no partial `DoubleDotSugar` node is created.

### `InvalidNavComponent`

An operator name appears as an outer navigation component after `::`, or
a grouped expression appears as the innermost navigation component. The
parser preserves the navigation path with a local error component.

### `ExpectedName` after `::`

When `::` is followed by a non-Name, non-operator, non-group token, the
parser emits `ExpectedName`.

These diagnostics are syntactic only. They do not perform field lookup,
method resolution, dispatch, indexing, slicing, or namespace resolution.

## 12. Operator-expression diagnostics

### `InvalidOperatorExpression`

Malformed operator sugar: missing operand (`a +`), unsupported prefix
operator (`!x`, `*x`, `++x`). The parser produces an error operator
expression node or best-effort sugar and continues.

### `ChainedNonAssociativeOperator`

Ungrouped chains of non-associative operators (`a < b < c`, `a == b == c`,
`a += b += c`). The parser emits the diagnostic and preserves a best-effort
operator sugar shape.

The parser does not perform operator lookup, overload resolution, arity
validation, type-directed lookup, ADL, mutation semantics, or semantic
lowering.

## 13. Closure diagnostics

### `InvalidClosureHead`

Emitted for:

- `FnHeadPrefix { ... }` without the required `=>` delimiter
- incoming `|> { ... }` headless pipe branch body
- malformed head clauses
- return slot with forbidden `with { ... }` clause
- missing expression after a head-clause keyword

### Delimiter diagnostics

Capture clauses and body blocks use `UnclosedBracket`, `UnclosedBrace`, or
`UnclosedParen` when delimiters are not matched.

### `UnusedClosureAst` — optional

This diagnostic code exists for optional / non-guaranteed closure-recovery
reporting. Callers must not rely on it being emitted. The closure AST is
still produced regardless.

Closure diagnostics do not perform closure materialization, capture mode
validation, contract validation, or lifetime validation.

## 14. Deduce-list diagnostics

### `InvalidDeduceList`

A deduce list (`<...>`) in a strong binding context is malformed:

- missing binder name (`<,x>`, `<<`)
- trailing comma before `>` (`<x,>`)
- missing annotation after `:` (`<x:>`)
- unclosed list (missing `>`)

The parser preserves whatever binders were already parsed where possible.
Some malformed annotation positions produce an error expression; other
malformed deduce-list cases recover without adding a dedicated `ErrorAst`
node. Deduce-list diagnostics are syntactic only; they do not
validate generic parameters, templates, type variables, or type-level
entities.

## 15. Canonical skeleton diagnostics

### `InvalidCanonicalSkeleton`

In an extraction context, the syntax after a deduce list does not form a
valid canonical skeleton. The parser skips to the next context boundary
and inserts an `ErrorAst`.

### `ExpectedName` in canonical paths

Used where a name is expected in a canonical navigation path.

Canonical skeleton diagnostics are parser preservation diagnostics only.
No canonical matching, admissibility checking, or semantic interpretation
is performed.

## 16. Alias and EntityRef diagnostics

### `ExpectedAliasTarget`

After `===` in an alias binding, the right-hand side is absent or the
current token cannot start an `EntityRef`. Also covers a missing component
after `::` when the alias RHS reaches a hard boundary. Recovery depends
on where the missing alias target is detected: at a hard boundary the
parser inserts an error component without consuming the boundary; for a
non-boundary invalid token it may consume the token and recover to the
form boundary.

### `InvalidAliasBinder` — reserved

This diagnostic code exists but the current parser does not normally emit it.
An invalid binder falls through to the ordinary-let error path which emits
`ExpectedName`. `InvalidAliasBinder` is reserved for future use.

### `InvalidAliasPosition`

An alias-shaped token sequence (`let` followed by a valid alias binder and
`===`) appears in a non-form position: inside a binding slot (parameter,
return, product extraction element) or inside an expression. The parser
emits the diagnostic and recovers to the enclosing delimiter or form boundary.

### `InvalidEntityRef`

The `EntityRef` structure has malformed non-boundary components: a grouped
expression used as the innermost component (`(a, b)::ns`), or an operator
name used as an outer component (`x::+`). The parser preserves parsed
segments and replaces the malformed continuation with an error component.

### `UnexpectedAliasRhsExpression`

A valid `EntityRef` was parsed, but the next token is not a form boundary
(`;`, `}`, EOF). Residual tokens form an expression shape. The parser
consumes tokens until the form boundary.

Alias diagnostics do not perform alias target resolution, namespace lookup,
or operator identity validation. EntityRef parsing in v0.2 is only the
alias-RHS subset.

## 17. Diagnostic span policy

- Spans refer to the normalized (LF) source text.
- Each diagnostic has one primary span.
- Lexer diagnostics span the offending token or the unterminated construct
  from its opening delimiter to the end of the scanned text.
- Delimiter diagnostics (`UnclosedParen`, `UnclosedBracket`, `UnclosedBrace`)
  point to the opening delimiter.
- Parser diagnostics point to the unexpected, missing, or invalid token
  at the point of failure.
- `ErrorAst` span corresponds to the recovery placeholder or the malformed
  region that could not be parsed into a valid construct.

## 18. ErrorAst relation

`ErrorAst` is a Raw AST recovery node. It carries a `message` and a `span`.
It allows parsing to continue beyond structurally invalid input.

`ErrorAst` is not a semantic error object, not an exception, not a runtime
value, not a typed hole, and not a proof obligation.

The current AST carries `Error(ErrorAst)` variants in these positions:

- `FormAst::Error`
- `BindingPatternAst::Error`
- `BindingAnnotationAst::Error`
- `WithClauseKind::Error`
- `ExprKind::Error`
- `OperatorExprKind::Error`
- `AtomKind::Error`
- `NavComponentAst::Error`
- `CanonicalSkeletonAst::Error`
- `HeadClauseAst::Error`
- `AliasBinderAst::Error`

## 19. Diagnostic stability boundary

- The `DiagnosticCode` enum is part of the frozen v0.2 frontend surface.
- v0.2 must not add, remove, or rename Raw AST frontend diagnostic codes
  unless a documented hard-correctness-error exception applies.
- Diagnostic messages are human-facing and may be less stable than codes.
- Diagnostic ordering should not be relied on outside of golden dump contexts.
- v0.3 must not mutate the frozen Raw AST frontend diagnostic code set;
  any normalization-stage diagnostics require their own explicit v0.3
  diagnostic specification.

## 20. Golden diagnostic snapshots

Diagnostic dumps are part of the current conformance surface. Golden tests
lock externally visible diagnostic behavior for covered cases.

Current diagnostic golden test count: 43 cases.

Golden snapshots are not updated by this document.

## 21. Non-semantic diagnostic boundary

The v0.2 frontend diagnostics do not report:

- unresolved names
- undefined aliases
- invalid alias targets
- unknown operators
- ambiguous overloads
- type errors
- kind errors
- invalid contracts
- failed canonical matching
- capture analysis errors
- ownership errors
- lifetime errors
- drop errors
- control-flow errors
- effect errors
- runtime exceptions
- code generation failures

Syntax acceptance plus diagnostics only describes Raw AST frontend shape
and recovery. Semantic errors belong to later semantic passes.

## 22. Relationship to other documents

| Document | Relationship |
|---|---|
| `spec/public/v0.2/lexical-syntax-v0.2.md` | Defines the token categories and lexical rules that produce lexical diagnostics. |
| `spec/public/v0.2/concrete-syntax-v0.2.md` | Defines the parser syntax accepted by the frontend and the grammar that may trigger parser diagnostics. |
| `spec/contracts/raw-ast-contract-freeze-v0.2.md` | Defines the v0.2 freeze boundary for the diagnostic surface. |
| `spec/public/v0.2/raw-ast-frozen-surface-v0.2.md` | Enumerates the frozen Raw AST constructs including diagnostic codes and ErrorAst. |
| `spec/implementation/v0.1/diagnostics-v0.1.md` | Older detailed diagnostic catalog; remains the implementation-level reference. |
| `spec/implementation/v0.1/ast-construction-v0.1.md` | Detailed parser-construction spec. |
| `spec/history/v0.1/operator-design.md` | Operator syntax design and implementation boundaries. |
| `spec/design/symbol-world/entity-alias-design.md` | Alias binding design (parser preservation implemented). |
| `spec/design/symbol-world/entity-ref-design.md` | Future general EntityRef design (alias-RHS subset implemented). |
| `spec/implementation/v0.1/implementation-status-v0.1.md` | Authoritative factual inventory of current implementation. |

This document is the primary public diagnostics and recovery reference for
v0.2. `lexical-syntax-v0.2.md` defines tokenization. `concrete-syntax-v0.2.md`
defines accepted parser syntax. `diagnostics-v0.1.md` remains the older
implementation-level reference. Historical design notes explain decisions
but are not the primary public diagnostics entry point.
