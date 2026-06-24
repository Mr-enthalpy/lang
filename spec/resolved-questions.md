# Resolved Design Questions

This document records design questions for the `lang` language that have been
resolved in v0.1. They serve as a historical record and reference for the
decisions made.

Open and deferred questions remain in `spec/open-questions.md`.

---

## 1. Nested block-comment policy

**Status:** Resolved

**Resolution:**
Block comments nest. A block comment starts at `/*` and ends at the matching
`*/`. Nested `/*` increments block-comment depth, and `*/` decrements it.

Inside a block comment, `//` has no special meaning.
Inside a line comment, `/*` and `*/` have no special meaning.

Line comments start at `//` and end before the next line break or EOF.
Comment delimiters are recognized as contiguous character pairs; whitespace may
surround them but may not split them.

**Implementation status:**
`lex_block_comment` in `lexer.rs` uses depth counting. `lex_line_comment` is
unchanged (already newline-sensitive, does not scan for block delimiters).

---

## 2. Form boundary: line-based vs explicit

**Status:** Resolved

**Resolution:**
Form boundaries are explicit and hard-only: `;`, `}`, EOF.
A line break is trivia and is never promoted to a form separator.

The language follows a strong-semicolon rule. The parser has no mixed
line-based / explicit boundary mode.

**Implementation status:**
Newline promotion, continuation tokens, and soft separator logic are removed
from the parser. `is_form_boundary()`, `is_alias_rhs_boundary()`, and
`is_entity_ref_boundary()` all delegate directly to the hard-boundary check
on the cursor.

---

## 3. Whether non-ArgPack `(a, b)` is always illegal

**Status:** Resolved

**Resolution:**
A parenthesized form with top-level commas is a product form, not an ArgPack.
It is always syntactically legal.

In expression context, `(a, b)` is product construction.
In binding / extraction context, `(a, b)` is product extraction.

ArgPack is not a language-level concept. Parser and Raw AST terminology use
Product / ProductExtract instead.

---

## 4. Exact AST shape for right-target subsegments

**Status:** Resolved

**Resolution:**
`ArgPackRole::RightTargetSubsegment` has been removed with the ArgPack
abstraction. Product forms are ordinary expression / extraction constructs in
Raw AST. Any later call/application nesting is a Normalized AST concern, not a
Raw AST role-assignment rule.

---

## 5. Whether capture clause stores token trees or expression AST

**Status:** Resolved

**Resolution:**
Capture clause items are full `ExprAst` nodes, not token trees and not
name-only items. The parser preserves `[expr1, expr2, ...]` syntactically.
Capture materialization is a later lowering step that assigns synthetic
closure fields in source order. Lifetime, ownership, and capture
admissibility checks are not parser work.

**Implementation status:**
`CaptureItemAst { expr: ExprAst }` is implemented in `ast.rs`.
`parse_capture_clause` uses `parse_expr_until` to parse each capture item.
No semantic capture validation is performed.

---

## 6. How much closure-head finite lookahead is allowed

**Status:** Resolved

**Resolution:**
Closure-head lookahead is finite and structurally committed by the first
enclosing body/with delimiter. When a `{ ... }` body-like form is encountered:

- If immediately owned by `with`, it is parsed as `with { ... }`
  (a binding-slot clause, consumed by the binding parser).
- If a successfully parsed `FnHeadPrefix` is followed by `=> { ... }`,
  it is parsed as `ExplicitClosureAst`.
- With no `with` owner and no committed `=>` closure head, a bare `{ ... }`
  in atom position is parsed as `InPlaceClosureAst`.

A `FnHeadPrefix` followed directly by `{ ... }` without `=>` is invalid
(`InvalidClosureHead`) and is not reinterpreted as an in-place closure.

These are fixed longest-match cases. The parser does not perform unbounded
semantic backtracking.

**Implementation status:**
The parser already implements this disambiguation in `try_parse_closure`,
`parse_with_clause`, and `parse_binding_slot`. Golden tests lock the four
committed cases: `let_with_empty_not_inplace`, `closure_inplace_empty`,
`closure_explicit_empty_params`, and `invalid_closure_headed_no_arrow_1/2/3`.

---

## 7. Workspace readiness: cargo test passes with minimal crates

**Status:** Resolved

**Resolution:**
The workspace is ready. `crates/lang_syntax`, `crates/lang_cli`, and `xtask`
exist as valid workspace members. `cargo check --workspace` passes.

Golden test counts are tracked in `spec/implementation-status-v0.1.md`.
Remaining test coverage gaps are tracked as v0.1 implementation work, not as
workspace-readiness uncertainty. This entry is closed.

---

## 8. Operator precedence relative to pipe and whitespace auto-pipe

**Status:** Resolved

**Resolution:**
Ordinary operators bind more tightly than both whitespace auto-pipe and `|>`.
Operator precedence is a segment-local sugar layer inside the existing
pipe/segment architecture, not a traditional C-like expression model.

**Implementation status:**
Implemented in parser phase 4 as raw AST sugar.

---

## 9. Angle brackets outside deduce-list contexts

**Status:** Resolved

**Resolution:**
`<...>` remains a `DeduceList` only in strong binding contexts. In
expression/operator contexts, `<`, `>`, `<=`, and `>=` are operator spellings.
`<>` has no generic-call meaning.

**Implementation status:**
Implemented in parser phase 4. `<` and `>` are expression operators outside
strong binding contexts and do not introduce generic-call syntax.

---

## 10. Postfix operator suffix composition

**Status:** Resolved

**Resolution:**
Postfix unary operators compose in the atom suffix loop with `::`, `.`, and
`..`. They do not terminate suffix parsing; `obj!.field` has the shape
`(obj!).field`.

**Implementation status:**
Implemented in parser phase 4 at the `OperatorExprAst` layer.

---

## 11. Operator names in binders and paths

**Status:** Resolved

**Resolution:**
Operator names may appear as binder names and as innermost navigation
components: `BinderName := Name | OperatorName` and
`NavComponent := Name | NumericName | OperatorName | GroupedExpr`.
Operator names may only be innermost navigation components, not outer scope
components.

**Implementation status:**
Implemented in parser phase 4.1 as raw AST preservation. Operator lookup,
lowering, overload resolution, and alias binding remain future work.

**Resolution update (v0.1 boundary recission):**
The `<` spelling is accepted as an operator binder when it is not followed by a
valid binding deduce-list start. A binding deduce list must contain a binder /
hole name after `<`; therefore `let <: _: operator = expr` and `let < = expr`
are parsed as operator binder declarations, not as extract-let deduce lists.

No escaping syntax is required for this case. The parser only enters
DeduceList parsing when `<` is followed by a valid deduce-list binder start
(`Name` token or `>` for the empty list).

---

## 12. Comparison, equality, and equals-suffixed chaining

**Status:** Resolved

**Resolution:**
Comparison, equality, and equals-suffixed operators are non-associative in
this phase. Ungrouped chains such as `a < b < c`, `a == b == c`, and
`a += b += c` require explicit grouping.

**Implementation status:**
Implemented in parser phase 4 with `ChainedNonAssociativeOperator`.

---

## 13. Numeric token AST identity depends on syntactic position

**Status:** Resolved

**Resolution:**
`IntLiteral` token in atom-base position -> numeric literal atom (`IntLiteral`).
`IntLiteral` token in selector position -> `NumericNameAst`.
`IntLiteral` token in path-leaf position -> `NumericNameAst`.
`IntLiteral` token in argument expression position -> numeric literal atom.

The distinction is mandatory and implemented in the current phase.

---

## 14. Lexical alias binding and entity references

**Status:** Resolved for raw parser preservation; semantic alias resolution remains open

**Current v0.1 implementation:**
Raw parser preservation for `let binder === EntityRef` is implemented in
Phase 4.4. The lexer recognizes `===` as `Symbol::TripleEqual`. The parser
produces `LetAliasAst` containing `AliasBinderAst` and `EntityRefAst`.
EntityRef parsing is available inside alias-let RHS only. Alias-let dispatch
correctly rejects extract-let, annotation, and `with` paths. `guard` is parsed
as an ordinary simple-let binder name, not as an alias modifier. See
`spec/implementation-status-v0.1.md` and `spec/entity-alias-design.md`.

**What is not implemented:**
- Target entity resolution (semantic lookup).
- Operator alias identity validation (spelling + fixity + arity).
- Name lookup, operator lookup, namespace resolution, dependency resolution.
- Alias scope semantics, shadowing, or semantic validation.

---

## 15. Alias binding position: all forms or top-level only

**Status:** Resolved

**Resolution:**
Alias binding is a form-level construct, not a top-level-only construct and not a
binding-slot construct.

It may appear wherever a `Form` may appear: at source-file form level and inside
closure body form lists. It may not appear inside expressions, product extraction
elements, parameter clauses, return clauses, annotations, head-clause expressions,
or ordinary binding slots.

The canonical shape is:

    OptionalPolicy? let AliasBinder === EntityRef

and it must be bounded by hard form boundaries: `;`, `}`, or EOF. In normal
source style this means an alias binding is written as a standalone form:

    let A === B;
    policy let A === B;

It must not be mixed with preceding or following expression material in the same
form.

**Implementation status:**
`parse_let_form` dispatches to alias only at form level. `parse_binding_slot`
and `parse_atom_base` emit `InvalidAliasPosition` when alias-shaped tokens
appear in non-form positions (Param, Return, product extraction element,
or expression atom).

---

## 16. With-clause payload grammar and Raw AST boundary

**Status:** Resolved

**Resolution:**
`with {}` is the only empty with clause. A non-empty `with { ... }` accepts only
comma-separated source-level `Name` items. It does not accept expressions,
paths, operator names, EntityRef syntax, canonical skeletons, or token trees.

The Raw AST preserves these names as `Vec<NameAst>` only. It does not check
whether a name exists above, earlier in the same binding list, in the same scope,
or in any dependency/lifetime environment. Existence, dependency validity, and
lifetime meaning belong to later name-resolution / ownership / lifetime phases.

Trailing commas in `with { ... }` are rejected by the parser.

**Implementation status:**
Already implemented in `let_stmt.rs` (`parse_with_clause`); no parser change
required.

**Why it does not block v0.1:**
The parser already enforces this grammar. The boundary is a documentation
clarification, not an implementation change.
