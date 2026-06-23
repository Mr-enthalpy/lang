# Implementation Status v0.1

Authoritative factual inventory of current implementation status.

This file records implementation facts. It does not override normative syntax
rules in `ast-construction-v0.1.md`, `operator-design.md`, or
`diagnostics-v0.1.md`.

This document records what the current codebase implements. It is not
normative for parser behavior — `spec/ast-construction-v0.1.md` and
`spec/operator-design.md` define what the parser must do. This document
only records what it currently does.

## Feature inventory

| Feature | Status | Implemented files | Spec authority | Notes |
|---|---|---|---|---|
| Weak lexer (Name / IntLiteral / StringLiteral / Symbol / Trivia / Invalid / Eof) | `implemented-syntax` | `token.rs`, `lexer.rs` | `ast-construction-v0.1.md` §2 | No keyword classification. Contextual names are ordinary `Name` tokens. |
| Operator-aware lexer (31 operator spellings) | `implemented-syntax` | `token.rs` (OperatorSpelling), `lexer.rs` | `operator-design.md` | Maximal-munch. `+`, `-`, `*`, `/`, `<`, `>`, `<=`, `>=`, `==`, `!=`, `<<`, `>>`, `&`, `|`, `&&`, `||`, `!`, `@`, `~`, `^`, `$`, `++`, `--`, `?`, `+=`, `-=`, `*=`, `/=`, `&=`, `|=`, `<<=`, `>>=` |
| `===` / TripleEqual token | `implemented-syntax` | `token.rs` (Symbol::TripleEqual), `lexer.rs` | `entity-alias-design.md` | Lexed before `==` and `=`. Structural delimiter, NOT an operator spelling. |
| Binding slots (let, parameters, returns) | `implemented-syntax` | `let_stmt.rs`, `closure.rs`, `deduce.rs`, `canonical.rs` | `ast-construction-v0.1.md` §4, §11 | Optional `let`, per-slot deduce list, binding pattern, optional annotation, optional `with` where allowed, optional initializer by context. |
| `guard` attrs | `removed-syntax` | `let_stmt.rs` | `ast-construction-v0.1.md` §4 | `guard` is ordinary `Name` unless future syntax reintroduces it. |
| `with { ... }` clause | `implemented-syntax` | `let_stmt.rs` | `ast-construction-v0.1.md` §4.2 | `with {}` is empty; non-empty payload is `Name*` only (no expressions, paths, operator names, or other syntax). Names preserved syntactically; existence, dependency, and lifetime validity are deferred. |
| Binding annotation (`: type`, `: _ : fn`) | `implemented-syntax` | `let_stmt.rs` | `ast-construction-v0.1.md` §4.4-4.6 | `BindingAnnotationAst::Expr` and `Compound` preserved; no type/rank/classifier checking. |
| Operator binder names (`let +: _: operator = ...`) | `implemented-syntax` | `let_stmt.rs`, `token.rs` | `operator-design.md` | Operator names accepted as binder, including `<` and `>`. |
| PipeExpr / Segment / Product forms | `implemented-syntax` | `pipe.rs`, `product.rs` | `ast-construction-v0.1.md` §7-9 | Product forms replace ArgPack roles. No SourcePack, InsertPack, or RightTargetSubsegment assignment. |
| OperatorExpr (prefix-negative `-x`, postfix, binary) | `implemented-syntax` | `operator.rs` | `operator-design.md` + `ast-construction-v0.1.md` §7.3 | Prefix `-x` is Raw AST preservation only; future normalization rewrites it to typed-zero binary `-`. Postfix and binary are Raw AST sugar only. No lookup or lowering in v0.1. |
| `::` navigation suffix | `implemented-syntax` | `atom.rs`, `operator.rs` | `ast-construction-v0.1.md` §8.4 | `NavPath` node in AtomAst and OperatorExprAst; components preserved source-order inner-to-outer. Parenthesized scope expressions after `::` are preserved as grouped outer components; a grouped expression as the innermost component (`(int Vec::std)::ns`) emits `InvalidNavComponent`. |
| `.` member sugar | `implemented-syntax` | `atom.rs`, `operator.rs` | `ast-construction-v0.1.md` §8.5 | `MemberSugar` node. Text or numeric selector. |
| `..` double-dot sugar | `implemented-syntax` | `atom.rs`, `operator.rs` | `ast-construction-v0.1.md` §8.6 | `DoubleDotSugar` node. Requires selector + product form. |
| `obj[args...]` bracket-call sugar | `implemented-syntax` | `atom.rs`, `operator.rs`, `product.rs` | `ast-construction-v0.1.md` §8.7 | `BracketCallSugar` node (atom + operator layer); operator spelling `[]`; bracket payload is a product form. Source-preserving; no indexing/slicing/container semantics, no lowering. `obj[]` valid. |
| `[]` operator spelling | `implemented-syntax` | `token.rs`, `let_stmt.rs`, `atom.rs`, `operator.rs` | `ast-construction-v0.1.md` §8.7 | Contextual paired operator name (not a single lexer token). Bindable/aliasable/referable in operator-name positions (binder, alias binder, entity-ref innermost component). No semantics. |
| Product form construction/extraction | `implemented-syntax` | `product.rs`, `let_stmt.rs`, `closure.rs`, `ast.rs` | `ast-construction-v0.1.md` §9, §11 | `(a, b)` is product construction in expression context and product extraction in binding / extraction context. Leading/doubled/trailing comma positions are explicit unit product elements. ArgPack and standalone `ExprKind::Unit` have been removed. |
| Numeric selectors (`obj.1`, `uint8::1`, `1.2`) | `implemented-syntax` | `atom.rs`, `lexer.rs` | `ast-construction-v0.1.md` §8.3 | IntLiteral in selector position → NumericNameAst. `1.2` is member sugar `(IntLiteral 1).(NumericName 2)`; no `FloatLiteral` token/node. `1.2.3` is left-assoc `(1.2).3`. Locked by `member_int_base`, `member_int_chain`, lexer `int_dot_int`. |
| Operator innermost navigation components (`+::int::std`) | `implemented-syntax` | `atom.rs`, `operator.rs` | `operator-design.md` | Valid only as innermost navigation component. Not valid after `.`, `..`, or as an outer navigation component. |
| In-place closure (bare `{}`) | `implemented-syntax` | `closure.rs`, `ast.rs` | `ast-construction-v0.1.md` §10 | Bare `{ ... }` in atom position is an in-place closure (`Closure InPlace`). No capture clause, no parameters, no head clauses. |
| Explicit headed closure (`FnHeadPrefix => { ... }`) | `implemented-syntax` | `closure.rs`, `ast.rs` | `ast-construction-v0.1.md` §10-11 | Headed closure must use `=>`; `FnHeadPrefix { ... }` without `=>` is rejected (`InvalidClosureHead`). Closure AST only; no materialization into callable objects. |
| Capture clause (capture item = ExprAst) | `implemented-syntax` | `closure.rs`, `ast.rs` | `ast-construction-v0.1.md` §11 | Capture items stored as `CaptureItemAst { expr: ExprAst }`. No capture validation, move/ref/copy interpretation, or capture analysis. |
| Closure head (deduce, capture, param, fn-item-trait, return clauses) | `implemented-syntax` | `closure.rs`, `deduce.rs`, `canonical.rs` | `ast-construction-v0.1.md` §11 | All clauses parsed and preserved. |
| `where` in closure head | `reserved-not-active` | `closure.rs` | `ast-construction-v0.1.md` §11.7 | Recognized as a reserved position but not parsed as a clause. Lookahead rejects it. `acquire` is an ordinary name (the earlier `acquire` direction is replaced by `pre`/`post`). |
| Head clauses (`require`/`pre`/`post`/`lifetime pre`/`lifetime post`) | `implemented-syntax` | `closure.rs`, `let_stmt.rs`, `pipe.rs` | `ast-construction-v0.1.md` §11.8 | Parsed as `HeadClauseAst` tail of `FnHeadPrefixAst`. Exactly one expression slot per clause; no contract/lifetime/resource/type/rank/predicate validation. Active only in the closure-head clause tail; ordinary names elsewhere. |
| Canonical skeleton | `parser-preserved-only` | `canonical.rs` | `ast-construction-v0.1.md` §6 | AST preserved; no matching, destructuring, or admissibility semantics. |
| Match-style expressions | `parser-preserved-only` | (expression parsing) | `ast-construction-v0.1.md` §12 | `match` is ordinary Name. No MatchExpr. Arms parse as closure AST. |
| Binding-slot policy expression (`Expr let …`) | `implemented-syntax` | `form.rs`, `let_stmt.rs`, `ast.rs` | `ast-construction-v0.1.md` §4.3 | Optional `policy` expression preserved on `BindingSlotAst` and `LetAliasAst`. Recognized only by shape `Expr let` in any `let` position (top-level, body, param, return, alias). `policy = None` = unwritten/implicit, not "no policy". No semantic validation. |
| Alias binding (`let binder === EntityRef`) | `implemented-syntax` | `let_stmt.rs`, `ast.rs`, `token.rs` | `ast-construction-v0.1.md` §16 + `entity-alias-design.md` | Raw AST preservation only. No alias semantics, lookup, target validation, or operator identity validation. EntityRef parsed only in alias-let RHS. Optional `policy` prefix preserved. |
| EntityRef parser (alias RHS subset) | `implemented-syntax` | `let_stmt.rs` | `entity-ref-design.md` + `ast-construction-v0.1.md` §16 | Only inside `let binder === ...`. Not a general expression parser mode. |
| Alias RHS boundary checking | `implemented-syntax` | `form.rs`, `let_stmt.rs` | `entity-alias-design.md` | Hard-only boundary: `;`, `}`, EOF. Newline promotion removed. Residual tokens before a hard boundary produce `UnexpectedAliasRhsExpression`. |
| Diagnostic taxonomy | `implemented-syntax` | `diagnostic.rs` | `diagnostics-v0.1.md` | 31 DiagnosticCode variants. 3 lexer, 18 parser, 3 operator, 4 alias, 2 optional/unreachable. |
| `InvalidAliasBinder` diagnostic | `diagnostic-only` | `diagnostic.rs` | `diagnostics-v0.1.md` | Reserved; not currently emitted by parser. |
| `UnusedClosureAst` diagnostic | `diagnostic-only` | `diagnostic.rs` | `diagnostics-v0.1.md` | Optional; not guaranteed to be emitted in current parser. |
| Golden tests | `implemented-syntax` | `tests/lexer_golden.rs`, `tests/parser_golden.rs`, `tests/diagnostics_golden.rs` | `ast-construction-v0.1.md` §15 | Covers lexer, parser/AST, and diagnostics. Stable hand-written dump format. |

## Current golden test snapshot

Golden case counts below are generated from the test case files. The full
`cargo test` count may differ (it includes non-golden unit tests and
workspace smoke tests).

| Category | Count |
|---|---|
| Lexer golden cases | 11 |
| Parser golden cases | 213 |
| Diagnostic golden cases | 32 |
