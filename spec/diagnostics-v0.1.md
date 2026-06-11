# Diagnostics Specification v0.1

## 1. Scope

This document defines error and warning diagnostics produced by the v0.1 lexer
and parser. It covers only syntax-level diagnostics.

It does **not** define:

- type errors
- kind errors
- lifetime / ownership / NLL errors
- match exhaustiveness errors
- semantic warnings

Every diagnostic must carry a primary span pointing to the location of the
error. Additional spans (secondary, help) are permitted but not required in
v0.1.

## 2. Recovery policy

The parser is error-tolerant. When an error is detected:

1. Emit a diagnostic with a primary span.
2. Insert an `ErrorAst` node into the AST at the recovery point.
3. Continue parsing from a reasonable resynchronization point.

The goal is to produce as much valid AST as possible, even from partially
invalid input. A form that cannot be recovered becomes a single `ErrorAst`
spanning the failed region.

## 3. Diagnostic categories

### 3.1 Lexer diagnostics

#### `InvalidToken`

- **Trigger**: A byte sequence that does not form a valid token (e.g., stray
  null byte, unrecognized symbol).
- **Primary span**: The offending bytes.
- **Recovery**: Skip the invalid byte(s) and continue.
- **AST effect**: No AST node. The invalid token is discarded.

#### `UnclosedString`

- **Trigger**: A string literal begun by `"` that reaches the end of the file
  or a newline before the closing `"`.
- **Primary span**: From the opening `"` to EOF or newline.
- **Recovery**: Emit the broken string as a `Literal(StringLiteral)` with an
  error flag, then continue.
- **AST effect**: The broken literal appears in AST as a `LiteralAst`; the
  parser may optionally attach an error annotation.

#### `UnclosedComment`

- **Trigger**: A block comment (`/* ...`) that reaches EOF before `*/`.
- **Primary span**: From `/*` to EOF.
- **Recovery**: Treat the comment as closed at EOF. Emit as `Trivia`.
- **AST effect**: No AST effect. Comment is trivia.

### 3.2 Parser diagnostics

#### `UnexpectedToken`

- **Trigger**: A token that does not match any expected production at the
  current parser position.
- **Primary span**: The unexpected token's span.
- **Recovery**: Skip tokens until a synchronization point (`;`, `}`, `)`, `]`,
  or form boundary). Insert `ErrorAst` at the failure site.
- **AST effect**: An `ErrorAst` node replaces the expected construct.

#### `ExpectedNameAfterDot`

- **Trigger**: A `.` atom suffix is followed by a token that is not `Name`.
- **Primary span**: The `.` token.
- **Recovery**: Consume the `.` and stop suffix folding. The atom stands
  without the `.` suffix.
- **AST effect**: No additional AST node. The `.` is dropped.

#### `ExpectedNameAfterDoubleDot`

- **Trigger**: A `..` atom suffix is followed by a token that is not `Name`.
- **Primary span**: The `..` token.
- **Recovery**: Consume the `..` and stop suffix folding. The atom stands
  without the `..` suffix.
- **AST effect**: No additional AST node. The `..` is dropped.

#### `ExpectedArgPackAfterDoubleDotName`

- **Trigger**: A `.. Name` suffix is not followed by an `ArgPack` (parenthesized
  list).
- **Primary span**: The `Name` after `..`.
- **Recovery**: Consume the `Name` and stop. The atom includes the `.. Name`
  prefix but no args.
- **AST effect**: The atom becomes a partial `DoubleDotSugar` node with an
  empty or missing args field.

#### `UnclosedParen`

- **Trigger**: An opening `(` without a matching `)` by the end of the current
  form or input.
- **Primary span**: The opening `(`.
- **Recovery**: Insert an implicit `)` at the form boundary. Parse contents as
  `ArgPack`.
- **AST effect**: The `ArgPack` (or group) is created with whatever contents
  were parsed before recovery.

#### `UnclosedBracket`

- **Trigger**: An opening `[` without a matching `]` by end of form or EOF.
- **Primary span**: The opening `[`.
- **Recovery**: Insert implicit `]` at the form boundary.
- **AST effect**: The `CaptureClause` (or other bracket-delimited construct)
  contains whatever items were parsed. The node is flagged as incomplete.

#### `UnclosedBrace`

- **Trigger**: An opening `{` without a matching `}` by end of input.
- **Primary span**: The opening `{`.
- **Recovery**: Insert implicit `}` at EOF. Parse body contents as `Form*`.
- **AST effect**: The `BodyBlock` is created with forms parsed before recovery.

#### `InvalidDeduceList`

- **Trigger**: A `<` that starts what appears to be a `DeduceList` in a strong
  binding context, but the contents do not form a valid binder declaration list
  (e.g., empty `<>` is valid, but `<,>` or `<a b>` without separator is not).
- **Primary span**: From `<` to the point of failure.
- **Recovery**: Skip to the matching `>` if identifiable, or to the next
  synchronization point. Insert `ErrorAst`.
- **AST effect**: An `ErrorAst` replaces the expected `DeduceListAst`.

#### `InvalidCanonicalSkeleton`

- **Trigger**: In an extraction context, the syntax after the deduce list does
  not form a valid `CanonicalSkeleton` (e.g., `(,)` with empty slots, or `_ _`
  where a single skeleton element is expected).
- **Primary span**: The failing token or region.
- **Recovery**: Skip to the next context boundary (`=`, `=>`, `:`, `->`, `{`,
  `,`, `)`). Insert `ErrorAst`.
- **AST effect**: An `ErrorAst` replaces the expected `CanonicalSkeletonAst`.

#### `InvalidClosureHead`

- **Trigger**: A sequence that starts like a `FnHeadPrefix` but contains a
  malformed clause (e.g., duplicate deduce list, misplaced `=>`, unrecognizable
  param list contents).
- **Primary span**: The failing clause or token.
- **Recovery**: Depending on severity, either skip the malformed clause or
  fall back to parsing as a non-closure atom.
- **AST effect**: An `ErrorAst` inside the closure head, or the entire closure
  head is replaced with `ErrorAst`.

#### `TopLevelComma`

- **Trigger**: A comma at the top level of a form (outside any `ArgPack` or
  parenthesized group).
- **Primary span**: The comma token.
- **Recovery**: Produce diagnostic and continue; the comma is consumed but
  does not create additional AST structure.
- **AST effect**: No change to AST. The comma is discarded.

#### `UnusedClosureAst`

- **Trigger** (optional / stretch goal): A closure literal appears in a
  position where it cannot be consumed by an operator, pipe, or binding
  (e.g., a lone `{}` form with no let binding, no `|>` in scope, and no
  enclosing construct that accepts closure AST).
- **Primary span**: The closure body or `{}` token.
- **Recovery**: The closure AST is still produced. The diagnostic warns that
  the closure is unused.
- **AST effect**: The closure AST node is preserved. The diagnostic is
  emitted as a warning or note.

This diagnostic is de-emphasized in v0.1. The parser should always produce
the closure AST node regardless of context.

## 4. Diagnostic format

The dump format for diagnostics in v0.1 should be stable and suitable for
golden testing. A suggested format:

```text
<level>: <message>
  --> <file>:<line>:<column>
```

Example:

```text
error: Expected name after `.`
  --> test.lang:3:10
```

The level may be `error`, `warning`, or `note`. In v0.1, all diagnostics are
`error` level unless the spec says otherwise.
