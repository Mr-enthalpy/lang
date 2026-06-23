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
| `with { ... }` clause | `implemented-syntax` | `let_stmt.rs` | `ast-construction-v0.1.md` §4.2 | `with {}` is empty; non-empty names are preserved syntactically; no name resolution or lifetime semantics. |
| Binding annotation (`: type`, `: _ : fn`) | `implemented-syntax` | `let_stmt.rs` | `ast-construction-v0.1.md` §4.4-4.6 | `BindingAnnotationAst::Expr` and `Compound` preserved; no type/rank/classifier checking. |
| Operator binder names (`let +: _: operator = ...`) | `implemented-syntax` | `let_stmt.rs`, `token.rs` | `operator-design.md` | Operator names accepted as binder; `<` not accepted (extract-let strong context). |
| PipeExpr / Segment / ArgPack roles | `implemented-syntax` | `pipe.rs`, `argpack.rs` | `ast-construction-v0.1.md` §7-9 | SourcePack, InsertPack, RightTargetSubsegment role assignment. |
| OperatorExpr (prefix `-`, postfix, binary) | `implemented-syntax` | `operator.rs` | `operator-design.md` + `ast-construction-v0.1.md` §7.3 | Raw AST sugar; precedence/associativity per operator-design.md. No lookup or lowering. |
| `::` navigation suffix | `implemented-syntax` | `atom.rs`, `operator.rs` | `ast-construction-v0.1.md` §8.4 | `NavPath` node in AtomAst and OperatorExprAst; components preserved source-order inner-to-outer. Parenthesized scope expressions after `::` are preserved as grouped outer components; a grouped expression as the innermost component (`(int Vec::std)::ns`) emits `InvalidNavComponent`. |
| `.` member sugar | `implemented-syntax` | `atom.rs`, `operator.rs` | `ast-construction-v0.1.md` §8.5 | `MemberSugar` node. Text or numeric selector. |
| `..` double-dot sugar | `implemented-syntax` | `atom.rs`, `operator.rs` | `ast-construction-v0.1.md` §8.6 | `DoubleDotSugar` node. Requires selector + ArgPack. |
| Numeric selectors (`obj.1`, `uint8::1`) | `implemented-syntax` | `atom.rs` | `ast-construction-v0.1.md` §8.3 | IntLiteral in selector position → NumericNameAst. |
| Operator innermost navigation components (`+::int::std`) | `implemented-syntax` | `atom.rs`, `operator.rs` | `operator-design.md` | Valid only as innermost navigation component. Not valid after `.`, `..`, or as an outer navigation component. |
| Closure AST (headed inline, explicit `() => {}`) | `implemented-syntax` | `closure.rs` | `ast-construction-v0.1.md` §10-11 | Bare `{}` is invalid in atom position. Closure AST only; no materialization into callable objects. |
| Closure head (deduce, capture, param, fn-item-trait, return clauses) | `implemented-syntax` | `closure.rs`, `deduce.rs`, `canonical.rs` | `ast-construction-v0.1.md` §11 | All clauses parsed and preserved. |
| `where` in closure head | `reserved-not-active` | `closure.rs` | `ast-construction-v0.1.md` §11.7 | Recognized as a reserved position but not parsed as a clause. Lookahead rejects it. `acquire` is an ordinary name (the earlier `acquire` direction is replaced by `pre`/`post`). |
| Head clauses (`require`/`pre`/`post`/`lifetime pre`/`lifetime post`) | `implemented-syntax` | `closure.rs`, `let_stmt.rs`, `pipe.rs` | `ast-construction-v0.1.md` §11.8 | Parsed as `HeadClauseAst` tail of `FnHeadPrefixAst`. Exactly one expression slot per clause; no contract/lifetime/resource/type/rank/predicate validation. Active only in the closure-head clause tail; ordinary names elsewhere. |
| Canonical skeleton | `parser-preserved-only` | `canonical.rs` | `ast-construction-v0.1.md` §6 | AST preserved; no matching, destructuring, or admissibility semantics. |
| Match-style expressions | `parser-preserved-only` | (expression parsing) | `ast-construction-v0.1.md` §12 | `match` is ordinary Name. No MatchExpr. Arms parse as closure AST. |
| Alias binding (`let binder === EntityRef`) | `implemented-syntax` | `let_stmt.rs`, `ast.rs`, `token.rs` | `ast-construction-v0.1.md` §16 + `entity-alias-design.md` | Raw AST preservation only. No alias semantics, lookup, target validation, or operator identity validation. EntityRef parsed only in alias-let RHS. |
| EntityRef parser (alias RHS subset) | `implemented-syntax` | `let_stmt.rs` | `entity-ref-design.md` + `ast-construction-v0.1.md` §16 | Only inside `let binder === ...`. Not a general expression parser mode. |
| Alias RHS boundary checking | `implemented-syntax` | `form.rs`, `cursor.rs`, `let_stmt.rs` | `entity-alias-design.md` | Layered checks: newline promotion without consuming newline, hard boundaries, residual expression rejection. |
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
