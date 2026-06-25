# Implementation Status v0.1

Authoritative factual inventory of current implementation status.

This file records implementation facts. It does not override normative syntax
rules in `ast-construction-v0.1.md`, `operator-design.md`, or
`diagnostics-v0.1.md`.

**Current stage:** v0.1 Raw AST Frontend completed; active stage is `v0.1.w`
Raw AST Stability Window.

The implementation listed here is the stable frontend baseline: lexer/parser
skeleton, `lex` / `parse`, Raw AST categories, token/AST/diagnostic dumps,
diagnostics infrastructure, and golden-test expectations are stable by default.
Future work in `v0.1.w` is documentation alignment, contract stabilization,
richer literal spelling, and local mechanical whole-shape sugar recognition
only, unless a hard correctness error is identified against the
call-composition architecture. Additions must extend existing lexer/parser
entry points and AST preservation categories; they must not replace the
product/pipe/operator/binding/closure/navigation architecture.

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
| Pipe branch-name shorthand (`|> name { ... }`) | `implemented-syntax` | `pipe.rs` | `ast-construction-v0.1.md` §7.1.1 | `v0.1.w` additive local sugar. Accepted only as a mechanical shorthand for `|> (_ name) { ... }`; not a precedent for a family of branch-arm sugars. No semantic validation, matching, lookup, or closure materialization. |
| OperatorExpr (prefix-negative `-x`, postfix, binary) | `implemented-syntax` | `operator.rs` | `operator-design.md` + `ast-construction-v0.1.md` §7.3 | Prefix `-x` is Raw AST preservation only; future normalization rewrites it to typed-zero binary `-`. Postfix and binary are Raw AST sugar only. No lookup or lowering in v0.1. |
| `::` navigation suffix | `implemented-syntax` | `atom.rs`, `operator.rs` | `ast-construction-v0.1.md` §8.4 | `NavPath` node in AtomAst and OperatorExprAst; components preserved source-order inner-to-outer. Parenthesized scope expressions after `::` are preserved as grouped outer components; a grouped expression as the innermost component (`(int Vec::std)::ns`) emits `InvalidNavComponent`. |
| `.` member sugar | `implemented-syntax` | `atom.rs`, `operator.rs` | `ast-construction-v0.1.md` §8.5 | `MemberSugar` node. Text selector only. Numeric selectors removed. |
| `..` double-dot sugar | `implemented-syntax` | `atom.rs`, `operator.rs` | `ast-construction-v0.1.md` §8.6 | `DoubleDotSugar` node. Requires selector + product form. |
| `obj[args...]` bracket-call sugar | `implemented-syntax` | `atom.rs`, `operator.rs`, `product.rs` | `ast-construction-v0.1.md` §8.7 | `BracketCallSugar` node (atom + operator layer); operator spelling `[]`; bracket payload is a product form. Source-preserving; no indexing/slicing/container semantics, no lowering. `obj[]` valid. |
| `[]` operator spelling | `implemented-syntax` | `token.rs`, `let_stmt.rs`, `atom.rs`, `operator.rs` | `ast-construction-v0.1.md` §8.7 | Contextual paired operator name (not a single lexer token). Bindable/aliasable/referable in operator-name positions (binder, alias binder, entity-ref innermost component). No semantics. |
| Product form construction/extraction | `implemented-syntax` | `product.rs`, `let_stmt.rs`, `closure.rs`, `ast.rs` | `ast-construction-v0.1.md` §9, §11 | `(a, b)` is product construction in expression context and product extraction in binding / extraction context. Leading/doubled/trailing comma positions are explicit unit product elements. ArgPack and standalone `ExprKind::Unit` have been removed. |
| `FloatLiteral` token and atom | `implemented-syntax` | `token.rs`, `lexer.rs`, `ast.rs`, `dump.rs` | `ast-construction-v0.1.md` §8.1 | Classic decimal `1.2` lexed as `FloatLiteral`. `1.2ms` splits as `FloatLiteral` + `Name`. |
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
| Diagnostic taxonomy | `implemented-syntax` | `diagnostic.rs` | `diagnostics-v0.1.md` | 28 DiagnosticCode variants. 3 lexer, 17 parser (including 1 optional/not-guaranteed-emitted), 3 operator, 5 alias. |
| `InvalidAliasBinder` diagnostic | `diagnostic-only` | `diagnostic.rs` | `diagnostics-v0.1.md` | Reserved; not currently emitted by parser. |
| `UnusedClosureAst` diagnostic | `diagnostic-only` | `diagnostic.rs` | `diagnostics-v0.1.md` | Optional; not guaranteed to be emitted in current parser. |
| Golden tests | `implemented-syntax` | `tests/lexer_golden.rs`, `tests/parser_golden.rs`, `tests/diagnostics_golden.rs` | `ast-construction-v0.1.md` §15 | Covers lexer, parser/AST, and diagnostics. Stable hand-written dump format. |

## Current golden test snapshot

Golden case counts below are generated from the test case files. The full
`cargo test` count may differ (it includes non-golden unit tests and
workspace smoke tests).

| Category | Count |
|---|---|
| Lexer golden cases | 17 |
| Parser golden cases | 289 |
| Diagnostic golden cases | 42 |
