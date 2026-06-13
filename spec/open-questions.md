# Open Questions

This document tracks unresolved design questions for the `lang` language.
They do not block v0.1 and should be revisited in the appropriate future
stage.

---

## 1. Nested block-comment policy

**Status:** Open

**Current v0.1 decision:**
Block comments (`/* ... */`) do not nest. The first `*/` closes the comment.
Nested `/*` inside a block comment is an error or ignored.

**Why it does not block v0.1:**
The lexer only needs a simple nesting-unaware comment rule for v0.1. A future
stage can add nesting if the language grammar requires it.

**Future stage:** v0.2 (frontend robustness) or v0.7 (full language design).

---

## 2. Form boundary: line-based vs explicit

**Status:** Open

**Current v0.1 decision:**
Form boundaries are: `;`, top-level line break (nesting depth 0), `}`, EOF.
Line breaks inside `()`, `[]`, `{}` are not form boundaries.

**Why it does not block v0.1:**
The provisional rule works for a line-oriented subset. A future explicit
terminator or indentation-based system can replace it without changing AST
construction.

**Future stage:** v0.2 (robustness) or later language design.

---

## 3. Whether non-ArgPack `(a, b)` is always illegal

**Status:** Open

**Current v0.1 decision:**
A parenthesized form with top-level commas is always an `ArgPack`. A
parenthesized form without commas is a group `(PipeExpr)`. Non-ArgPack
`(a, b)` is illegal.

**Why it does not block v0.1:**
The distinction between ArgPack and group is clear in v0.1. A future pass
may interpret certain ArgPacks as tuples, heterogeneous lists, or other
constructs, but that does not affect parsing.

**Future stage:** v0.7 (type design) — tuple types or value types may
reinterpret ArgPacks.

---

## 4. Exact AST shape for right-target subsegments

**Status:** Open

**Current v0.1 decision:**
Right-target subsegments are stored as flat `SegmentElement` nodes with
`ArgPackRole::RightTargetSubsegment`. The AST may optionally nest them,
but v0.1 prefers a flat representation.

**Why it does not block v0.1:**
A flat representation is sufficient for golden tests. A later lowering pass
may restructure the AST to make right-target subsegments explicit sub-trees.

**Future stage:** v0.3 (syntax normalization) or v1.0.

---

## 5. Whether capture clause stores token trees or expression AST

**Status:** Open

**Current v0.1 decision:**
`CaptureClause` is parsed as a bracket-delimited clause, but capture items
are stored as token-tree-like `CaptureItemAst` placeholders. The exact
internal structure of capture items is not specified in v0.1.

**Why it does not block v0.1:**
The parser can recognize `[ item, item ]` at the token level. Deeper
parsing (e.g., recognizing `a = b` patterns inside capture) can be added
later without breaking AST shape.

**Future stage:** v0.3 (normalization) or v0.6 (closure materialization).

---

## 6. How much closure-head finite lookahead is allowed

**Status:** Open

**Current v0.1 decision:**
The closure recognition algorithm (section 11.9 of ast-construction-v0.1.md)
uses finite lookahead. The exact lookahead depth is bounded by the maximum
clause prefix length: `<T>(x: T): runtime -> T where ... acquire ...`.

**Why it does not block v0.1:**
The bounded lookahead can be implemented with a fixed token buffer. A formal
upper bound should be specified to avoid parser ambiguity.

**Future stage:** v0.2 (robustness) — document the exact maximum lookahead
and add tests for edge cases.

---

## 7. Future canonical value/type grammar

**Status:** Deferred

**Current v0.1 decision:**
Canonical skeletons use the grammar defined in section 6 of
ast-construction-v0.1.md. This grammar is provisional and may be revised
when value/type canonical forms are designed.

**Why it does not block v0.1:**
The current canonical skeleton grammar builds AST only. No matching is
performed. Any future revision will produce a different AST shape, but
v0.1 AST will still be parseable.

**Future stage:** v0.4 (canonical form specification).

---

## 8. Future semantics of `return`, `else`, `match`, `effect`, `sync`

**Status:** Deferred

**Current v0.1 decision:**
These are ordinary `Name` tokens at the lexical and parser level. No special
AST nodes exist for them.

**Why it does not block v0.1:**
The v0.1 frontend faithfully preserves these names in expression AST. A
future semantic pass can interpret them by analyzing the AST structure
without requiring parser changes.

**Future stage:** v1.0 (or earlier semantic design stages v0.5–v0.7).

---

## 9. Future always-NLL CFG requirements

**Status:** Deferred

**Current v0.1 decision:**
No CFG is built. The parser does not construct a control-flow graph.

**Why it does not block v0.1:**
Ownership and lifetime analysis is out of scope for v0.1. The raw AST
contains sufficient structure (form order, closure bodies, guard/with
annotations) for a future CFG construction pass.

**Future stage:** v0.8 (ownership/NLL/drop design).

---

## 10. Workspace readiness: cargo test passes with minimal crates

**Status:** Resolved

**Resolution:**
The workspace is ready. `crates/lang_syntax`, `crates/lang_cli`, and `xtask`
exist as valid workspace members. `cargo check --workspace` passes.

Lexer golden tests (7 cases) and parser golden tests (20 cases under parser
phase 1) exist and pass. Diagnostic golden tests in
`tests/diagnostics_golden.rs` are not yet fully populated; that file is
currently a stub.

Remaining test coverage gaps are tracked as v0.1 implementation work, not as
workspace-readiness uncertainty. This entry is closed.

---

## 11. Operator precedence relative to pipe and whitespace auto-pipe

**Status:** Resolved

**Resolution:**
Ordinary operators bind more tightly than both whitespace auto-pipe and `|>`.
Operator precedence is a segment-local sugar layer inside the existing
pipe/segment architecture, not a traditional C-like expression model.

**Implementation TODO:**
Implement the operator-aware `OperatorExpr` layer inside segments.

---

## 12. Angle brackets outside deduce-list contexts

**Status:** Resolved

**Resolution:**
`<...>` remains a `DeduceList` only in strong binding contexts. In
expression/operator contexts, `<`, `>`, `<=`, and `>=` are operator spellings.
`<>` has no generic-call meaning.

**Implementation TODO:**
Update expression parsing so `<` and `>` are accepted as operators outside
strong binding contexts.

---

## 13. Postfix operator suffix composition

**Status:** Resolved

**Resolution:**
Postfix unary operators compose in the atom suffix loop with `::`, `.`, and
`..`. They do not terminate suffix parsing; `obj!.field` has the shape
`(obj!).field`.

**Implementation TODO:**
Add `PostfixOperator` to the atom suffix loop when operator parsing is added.

---

## 14. Operator names in binders and paths

**Status:** Resolved

**Resolution:**
Operator names may appear as binder names and as path leaves:
`BinderName := Name | OperatorName` and `PathLeaf := Name | OperatorName`.
Operator names may only be leaves, not namespace-like intermediate path nodes.

**Implementation TODO:**
Add operator binder-name parsing and operator path-leaf parsing in the operator
parser PR.

---

## 15. Comparison, equality, and compound-looking chaining

**Status:** Resolved

**Resolution:**
Comparison, equality, and compound-looking operators are non-associative in
this phase. Ungrouped chains such as `a < b < c`, `a == b == c`, and
`a += b += c` require explicit grouping.

**Implementation TODO:**
Add parser diagnostics for chained non-associative operators when operator
parsing is implemented.

---

## 16. Numeric selectors: positional access vs. general sugar

**Status:** Open

**Current v0.1 decision:**
Numeric tokens in selector/name-leaf position produce `NumericNameAst`. The
parser treats `obj.1`, `tuple.1`, and `pack.1` identically as
`MemberSugar { object, selector: NumericName("1") }`. No special AST nodes
such as `TupleIndex`, `TupleField`, or `PackIndex` are created.

**Why it does not block v0.1:**
Any future tuple/pack positional access semantics must be implemented by later
semantic lookup, namespace forwarding, or compiler-provided functions. The
parser must not hard-code positional access semantics.

**Future stage:** v0.7 (type design) or v1.0.

---

## 17. Float, scientific, and unit-adjacent numeric literals

**Status:** Open

**Current v0.1 decision:**
The spellings `1.2`, `1.2ms`, `1e3ms`, `1.2e3`, and `1.2e3ms` are reserved
for future numeric literal design. The current parser must not add golden tests
that force a particular interpretation of these forms.

The natural unit syntax `1ms` and `1 ms` remain equivalent as
`IntLiteral(1)` followed by `Name(ms)` at the non-trivia token/parser
structure level. No `UnitLiteral` AST node exists.

**Why it does not block v0.1:**
The existing lexer does not yet produce `FloatLiteral`, `ScientificLiteral`, or
`FloatScientificLiteral` tokens. Numeric tokens in selector position go through
the same token class but produce `NumericNameAst` rather than numeric literal
atoms. The boundary between `Digit+ "." Digit+` (future float) and
`object "." Name` (member sugar) will be decided with future lexer changes.

**Future stage:** v0.2 (frontend robustness) or later numeric literal design.

---

## 18. Numeric token AST identity depends on syntactic position

**Status:** Resolved

**Resolution:**
`IntLiteral` token in atom-base position → numeric literal atom (`IntLiteral`).
`IntLiteral` token in selector position → `NumericNameAst`.
`IntLiteral` token in path-leaf position → `NumericNameAst`.
`IntLiteral` token in argument expression position → numeric literal atom.

The distinction is mandatory and implemented in the current phase.

---

## 19. Name-polymorphic lookup boundary

**Status:** Open (design note, not implemented)

**Current v0.1 decision:**
`MemberSugar` and `DoubleDotSugar` preserve selector syntax (`TextNameAst` /
`NumericNameAst`) for later lookup. Selectors may participate in future
name-polymorphic lookup.

Name-polymorphic lookup is a compile-time-only extension of name binding:
function-name positions may contain explicitly declared name holes such as
`<f: TextNameAst>` or `<i: NumericNameAst>`.

This does **not** make lookup dynamic. Concrete names shadow abstract name
holes. If concrete candidates are found but fail to apply, the compiler reports
that failure and does **not** fall back to abstract name-polymorphic
candidates.

Only declarations that explicitly bind the function-name position as a name AST
hole participate in name-polymorphic lookup. Ordinary functions do not accept
arbitrary names.

Name constraints must be locally decidable at compile time, and candidate
ordering must be stable. If multiple applicable name-polymorphic candidates
remain unordered, lookup is ambiguous.

**Why it does not block v0.1:**
The selector AST already distinguishes `TextNameAst` and `NumericNameAst`
as distinct selector classes. This distinction is sufficient to support future
name-polymorphic lookup without requiring AST changes. v0.1 does not implement
lookup, binding, or name resolution. The parser only preserves selector shape.

**Future stage:** v0.7 (type/kind/checking design) or later name-resolution
design. A future decision document (e.g.
`docs/decisions/0005-name-polymorphic-lookup-boundary.md`) may formalize the
exact rules. The parser must not be changed to accommodate lookup before that
specification exists.
