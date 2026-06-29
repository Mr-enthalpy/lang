# Lexical Syntax v0.2

## 1. Scope

This document defines the public lexical syntax of the current v0.2 Raw AST
frontend. It covers:

```text
source text → normalized source text → token stream
```

It does not define parsing, Raw AST construction, Normalized AST, or semantic
interpretation. See `spec/implementation/v0.1/ast-construction-v0.1.md` for the concrete parser
syntax and `spec/public/v0.2/raw-ast-frozen-surface-v0.2.md` for the frozen Raw AST
construct inventory.

## 2. Source text normalization

The lexer normalizes source text before tokenization:

- CRLF (`\r\n`) is replaced with LF (`\n`).
- CR (`\r`) without a following LF is replaced with LF (`\n`).

Spans refer to the normalized source text. Byte offsets (`byte_start`,
`byte_end`), line numbers, and column numbers are based on the normalized
representation.

The lexer does not preserve an original pre-normalization line-ending map. No
semantic source mapping is performed.

## 3. Token model

The lexer produces nine token categories:

| Token category | Rust type | Description |
|---|---|---|
| `Name` | `TokenKind::Name` | An identifier. |
| `IntLiteral` | `TokenKind::IntLiteral` | An integer literal. |
| `FloatLiteral` | `TokenKind::FloatLiteral` | A floating-point literal. |
| `StringLiteral` | `TokenKind::StringLiteral` | A string literal. |
| `Symbol` | `TokenKind::Symbol(Symbol)` | A structural delimiter. |
| `Operator` | `TokenKind::Operator(OperatorSpelling)` | An operator spelling. |
| `Trivia` | `TokenKind::Trivia(TriviaKind)` | Whitespace, line comments, or block comments. |
| `Invalid` | `TokenKind::Invalid` | An unrecognized byte sequence. |
| `Eof` | `TokenKind::Eof` | End-of-input sentinel. |

Each token carries a `Span` (byte offset range, line number, column number)
and its source text where applicable. `Eof` carries an empty source text.

## 4. Weak lexer rule

The lexer does not classify any word as a keyword. Every recognized word
sequence is lexed as a `Name` token.

This is a core language invariant. The following words are ordinary `Name`
tokens at the lexer level:

```text
return  else  match  drop  move  ref
sync  effect  fn  type  meta  runtime  compile
namespace  struct  guard  acquire  delete
```

Any other source word matching the Name lexical shape is also lexed as `Name`.
Characters outside the Name lexical shape are not made into Name tokens merely
because they look word-like to a human reader. Lexical classification as
`Name` does not make the word a language construct.

The parser may later recognize selected names structurally in strong contexts
(see `spec/implementation/v0.1/ast-construction-v0.1.md`). Outside those contexts the same names
remain ordinary expression atoms.

## 5. Names

A `Name` token begins with:

- an ASCII letter (`a`–`z`, `A`–`Z`), or
- an underscore (`_`).

A `Name` token continues with:

- an ASCII letter (`a`–`z`, `A`–`Z`),
- an ASCII digit (`0`–`9`), or
- an underscore (`_`).

The token `_` (a single underscore) is a `Name` token. The lexer attaches no
wildcard, unit, ignored-binding, or pattern semantics to it.

Literal-name adjacency is ordinary expression/call composition material. For
example, `1ms` tokenizes as `IntLiteral("1")` followed by `Name("ms")`. Unit
names, encoding names, and similar suffix-like words are parsed as ordinary
names following a literal.

## 6. Integer literals

An integer literal is a sequence of one or more digits in a supported radix,
with optional single-quote digit separators.

### 6.1 Radix forms

| Radix | Prefix | Accepted digits | Example |
|---|---|---|---|
| Decimal | (none) | `0`–`9` | `123`, `0` |
| Binary | `0b` / `0B` | `0`–`1` | `0b1010` |
| Octal | `0o` / `0O` | `0`–`7` | `0o755` |
| Hexadecimal | `0x` / `0X` | `0`–`9`, `a`–`f`, `A`–`F` | `0xFF` |

The radix prefix and digits are case-insensitive (`0b` and `0B` are equivalent;
`0x` and `0X` are equivalent; `0o` and `0O` are equivalent).

### 6.2 Digit separators

The single quote character (`'`) may appear between two valid digits of the
same numeric component as a visual separator. The separator is preserved in
the token source text.

Valid separators:

```text
1'000'000
0b1010'1100
0xFFFF'0000
0o7'55
```

Invalid separator positions (produce `InvalidNumericLiteral` diagnostic
and the separator is included in the token text):

- at the end of a digit sequence (`100'`)
- two consecutive separators (`1''000`)
- immediately after a radix prefix (`0x'FF`)
- adjacent to a decimal point or exponent marker (`1.'0`, `1e'3`)

Separators that appear before any digits are not part of a numeric literal
scan. At the beginning of source or token position, `'100` starts with an
unrecognized apostrophe and, in the current lexer, produces an
`InvalidToken` diagnostic, not `InvalidNumericLiteral`. `InvalidNumericLiteral` applies only to malformed
separator positions inside a numeric literal scan.

### 6.3 Lexical boundary

The lexer preserves exact source text in the `IntLiteral` token. It does not
interpret the integer value, check for overflow, assign a width, or infer a
type. It does not recognize C/C++ integer type suffixes (`u`, `ul`, `ULL`).

## 7. Floating-point literals

A floating-point literal is a numeric sequence with an integral part, an
optional fractional part, and an optional exponent. The lexer produces a
single `FloatLiteral` token.

### 7.1 Decimal float forms

| Form | Example |
|---|---|
| Integral + fractional | `1.2` |
| Integral + dot (trailing) | `1.` |
| Leading dot + fractional | `.5` |
| Scientific notation (`e`/`E`) | `1e3`, `1E3` |
| Scientific with sign | `1e-3`, `1e+3` |
| Fractional + scientific | `1.2e-3` |
| Trailing dot + scientific | `1.e3` |
| Leading dot + scientific | `.5e-2` |

The trailing-dot float (`1.`) is recognized when the dot is not followed by
another dot and is not followed by an ASCII identifier start. Therefore
`1..x` starts as `IntLiteral("1")` followed by `DotDot`, and `1.x` starts as
`IntLiteral("1")` followed by `Dot`. When the dot is followed by neither
condition, the integer and dot are tokenized as a single `FloatLiteral("1.")`. 

### 7.2 Hexadecimal float forms

Hexadecimal floats use the `p` / `P` exponent marker (the exponent is always
decimal). The exponent may carry an optional sign.

| Form | Example |
|---|---|
| Hex integer + exponent | `0x1p+4` |
| Hex integer + fraction + exponent | `0x1.8p+2` |
| Hex trailing dot + exponent | `0x1.p+2` |
| Hex leading dot + fraction + exponent | `0x.8p+2` |
| Uppercase hex + exponent | `0X1.FP-3`, `0X1.P-2`, `0X.FP-3` |

### 7.3 Lexical boundary

The lexer preserves exact source text in the `FloatLiteral` token. It does not
interpret the float value, normalize precision, check IEEE conformance, or
infer a type. It does not recognize C/C++ float type suffixes (`f`, `L`).

## 8. String literals

String literals use ranked quote-boundary delimiters.

### 8.1 Boundary rule

A string literal begins with a boundary consisting of zero or more backslashes
(`\`) followed by a double-quote character (`"`). Let `k` be the number of
backslashes before the opening quote.

A double-quote character closes the string when preceded by at least `k`
consecutive backslashes. If more than `k` backslashes precede the closing
quote, the extra backslashes are body text; the final `k` backslashes plus the
quote form the closing boundary.

### 8.2 Examples by rank

All examples are literal source text inside fenced code blocks; the displayed
backslashes are source characters.

`k = 0` — ordinary short string. Closes at the next bare `"`.

```text
"abc"
```

`k = 1`. The boundary is one backslash followed by a quote. A bare `"`
inside does not close. A quote preceded by one or more consecutive backslashes
closes; extra backslashes are body.

```text
\"text may contain " without ending\"
```

`k = 2`. The boundary is two backslashes followed by a quote. Bare `"` and
`\"` inside do not close. A quote preceded by two or more consecutive
backslashes closes; extra backslashes are body.

```text
\\"text may contain " and \" without ending\\"
```

`k = 3`. The boundary is three backslashes followed by a quote. Bare `"`,
`\"`, and `\\"` inside do not close. A quote preceded by three or more
consecutive backslashes closes; the extra backslashes beyond three are body.

```text
\\\"text may contain " and \" and \\" without ending\\\"
```

### 8.3 No escape decoding

Backslashes inside a string literal participate only in boundary matching. The
lexer does not decode `\n`, `\t`, `\xNN`, `\uNNNN`, `\"`, or `\\` as character
values. Escape interpretation, if any, belongs to library-level string
construction or later semantic interpretation.

### 8.4 Unclosed strings

An unclosed or mismatched ranked string emits an `UnclosedString` diagnostic.

Examples of mismatched ranks (both unclosed):

```text
\"abc"        (k=1 opening, bare " does not match)
\\"abc\"      (k=2 opening, \" does not match)
```

### 8.5 Lexical boundary

The lexer preserves the full source text of the string literal — opening
boundary, body, and closing boundary — in the `StringLiteral` token. Body
content is not decoded or normalized. The lexer does not perform adjacent
string concatenation. `"a" "b"` tokenizes as two separate `StringLiteral`
tokens with trivia between them.

There is no character-literal token category; a source spelling such as
`"a"` is tokenized as `StringLiteral`, not as a separate `CharLiteral`.

Literal-string adjacency (`"abc"utf8`) is ordinary expression/call composition
material: `StringLiteral("\"abc\"")` followed by `Name("utf8")`.

## 9. Symbols

Structural symbols are single-token delimiters. The lexer recognizes 19
symbol variants, listed below with their source spelling and lexical identity.

| Symbol | Spelling | Lexical identity |
|---|---|---|
| `LParen` | `(` | Left parenthesis |
| `RParen` | `)` | Right parenthesis |
| `LBracket` | `[` | Left bracket |
| `RBracket` | `]` | Right bracket |
| `LBrace` | `{` | Left brace |
| `RBrace` | `}` | Right brace |
| `Comma` | `,` | Comma |
| `Colon` | `:` | Colon |
| `Equal` | `=` | Equal sign |
| `Dot` | `.` | Dot |
| `DotDot` | `..` | Dot-dot (double-dot) |
| `ColonColon` | `::` | Colon-colon (navigation) |
| `PipeGreater` | `\|>` | Pipe-greater (pipe transition) |
| `FatArrow` | `=>` | Fat arrow (closure head delimiter) |
| `ThinArrow` | `->` | Thin arrow (return clause) |
| `Less` | `<` | Less-than (symbol at lexer level) |
| `Greater` | `>` | Greater-than (symbol at lexer level) |
| `Semicolon` | `;` | Semicolon (form boundary) |
| `TripleEqual` | `===` | Triple-equal (alias binding delimiter) |

`TripleEqual` is a structural delimiter for alias binding (`let binder ===
EntityRef`). It is not an operator spelling and does not denote equality or
comparison semantics.

`Less` and `Greater` are symbols at the lexer level. In expression and
operator contexts the parser may reinterpret them as operator spellings. In
strong binding contexts (`let`, closure head, parameter, return) they may
delimit a DeduceList (`<...>`). These are parser-level decisions documented in
`spec/implementation/v0.1/ast-construction-v0.1.md`.

## 10. Operator spellings

The lexer produces 30 `TokenKind::Operator` spellings via maximal-munch
(longest-match) tokenization. Two additional operator-name spellings, `<` and
`>`, are lexed as `Symbol::Less` / `Symbol::Greater` and may be reinterpreted
by parser contexts. `[]` is a contextual paired operator spelling and is never
produced by the lexer as a single token.

Structural symbols `=>`, `->`, `|>`, `..`, `::`, and `===` are not operator
spellings. They are `Symbol` tokens (see §9).

| Category | Spellings |
|---|---|
| 3-char | `<<=`, `>>=` |
| 2-char increment/decrement | `++`, `--` |
| 2-char equals-suffixed | `+=`, `-=`, `*=`, `/=`, `&=`, `\|=` |
| 2-char logic | `&&`, `\|\|` |
| 2-char comparison/equality | `<=`, `>=`, `==`, `!=` |
| 2-char shift | `<<`, `>>` |
| 1-char | `+`, `-`, `*`, `/`, `!`, `&`, `\|`, `@`, `~`, `^`, `$`, `?` |
| Symbol-lexed, parser-reinterpreted | `<`, `>` |
| Contextual paired bracket | `[]` |

### 10.1 Symbol-lexed operator names

The spellings `<` and `>` are lexed as `Symbol::Less` and `Symbol::Greater`,
not as operator tokens. The parser may reinterpret them as operator spellings
in expression and operator contexts. In strong binding contexts they may
delimit a DeduceList (`<...>`).

### 10.2 Contextual bracket-call operator

The spelling `[]` is recognized contextually as a paired operator name. It is
never produced by the lexer as a single `TokenKind::Operator`. The parser
recognizes `[]` in operator-name positions (binder, alias binder, entity-ref
innermost component) and as the operator identity of bracket-call sugar
(`obj[args...]`).

### 10.3 Non-semantic operator boundary

Operator spellings are syntax-level names. They do not imply:

- arithmetic
- comparison
- assignment
- mutation
- overload resolution
- argument-dependent lookup (ADL)
- type-directed lookup
- evaluation

Operator syntax is preserved as Raw AST sugar. Operator lookup and semantic
validation are future semantic work. Operator-sugar desugaring belongs to the
future Normalized AST / normalization stages, not to lexical syntax.

## 11. Trivia

Trivia tokens carry spans but are skipped by the parser.

### 11.1 Whitespace

Source text normalization (§2) converts CR and CRLF to LF before tokenization.
In the normalized source, whitespace tokens are `' '`, `'\t'`, and `'\n'`.
A line break (`\n`) is trivia and is never promoted to a form separator.

### 11.2 Line comments

A line comment starts with `//` and ends before the next line break (`\n`)
or end of file. Line comments do not nest. Inside a line comment, `/*`
and `*/` have no special meaning.

### 11.3 Block comments

A block comment starts with `/*` and ends at the matching `*/`. Block comments
nest: each `/*` inside a block comment increments the nesting depth, and each
`*/` decrements it. The comment ends when the depth returns to zero.

Inside a block comment, `//` has no special meaning.

An unclosed block comment (depth never returns to zero before EOF) produces an
`UnclosedComment` diagnostic and is treated as closed at EOF.

## 12. Invalid lexical material

Unrecognized byte sequences produce `Invalid` tokens. The lexer emits an
`InvalidToken` diagnostic and continues lexing following bytes.

The following lexer diagnostics cover lexical-level error conditions:

| Diagnostic | Trigger |
|---|---|
| `InvalidToken` | Unrecognized byte sequence |
| `UnclosedString` | String literal reaching EOF or newline before closing boundary |
| `UnclosedComment` | Block comment never closed before EOF |
| `InvalidNumericLiteral` | Malformed numeric literal (invalid separator position, missing radix digits, empty hex float exponent) |

Parser-level diagnostics are documented in `spec/implementation/v0.1/diagnostics-v0.1.md` and are
outside the scope of this lexical specification.

## 13. End of file

The lexer emits a final `Eof` token after the last source character. `Eof`
marks the end of the token stream. It is not a source character and carries an
empty source text.

## 14. Non-semantic lexer boundary

The lexer is a mechanical tokenizer. It does not perform:

- keyword classification
- name resolution
- type inference
- numeric evaluation
- overflow checking
- string escape decoding
- operator lookup
- alias resolution
- macro expansion
- layout-sensitive parsing
- semantic validation

Some structural recognition belongs to the parser; normalization handles
non-semantic desugaring; name resolution, type inference, numeric evaluation,
operator lookup, alias resolution, and evaluation behavior belong to later
semantic passes.

## 15. Relationship to other documents

| Document | Relationship |
|---|---|
| `spec/contracts/raw-ast-contract-freeze-v0.2.md` | Defines the v0.2 contract freeze boundary in which this lexical syntax is frozen. |
| `spec/public/v0.2/raw-ast-frozen-surface-v0.2.md` | Enumerates the frozen Raw AST constructs, including token categories and diagnostics. |
| `spec/implementation/v0.1/ast-construction-v0.1.md` | Defines parser behavior — the consumer of the token stream defined here. |
| `spec/history/v0.1/operator-design.md` | Detailed operator spelling design and implementation boundaries. |
| `spec/implementation/v0.1/diagnostics-v0.1.md` | Full diagnostic catalog including parser and alias diagnostics. |
| `spec/implementation/v0.1/implementation-status-v0.1.md` | Authoritative factual inventory of current implementation. |

This document is the primary public lexical syntax reference. It defines
the current v0.2 lexical surface. `ast-construction-v0.1.md` defines parser
behavior. `diagnostics-v0.1.md` defines diagnostic rules.
`implementation-status-v0.1.md` records factual implementation status. Earlier
design notes explain historical decisions but are not the primary public syntax
entry point.
