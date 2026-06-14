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

The provisional v0.1 top-level newline boundary rule is implemented.
The broader language-design question of whether form boundaries should remain
line-based or become fully explicit remains open.

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

**Status:** Partially resolved for v0.1 parser preservation

**Current v0.1 decision:**
`CaptureClause` is parsed as a bracket-delimited clause, but capture items
are stored as syntactic `CaptureItemAst` entries containing expression AST.
No capture validation, move/ref/copy interpretation, or capture analysis is
performed in v0.1.

**Why it does not block v0.1:**
The parser can recognize and preserve `[ item, item ]` syntax. Deeper
capture semantics can be added later without changing the v0.1 preservation
policy.

**Future stage:** v0.3 (normalization) or v0.6 (closure materialization).

---

## 6. How much closure-head finite lookahead is allowed

**Status:** Open

**Current v0.1 decision:**
The closure recognition algorithm (section 11.9 of ast-construction-v0.1.md)
uses finite lookahead. The exact lookahead depth is bounded by the maximum
implemented clause prefix length: `<T>[cap](x: T): runtime -> T`. `where`
and `acquire` remain reserved but are not active closure-head clauses in
Phase 3.1.

**Why it does not block v0.1:**
The bounded lookahead is implemented with cursor save/restore and stack-based
diagnostic gates. Phase 3.1 adds regression tests for failed lookahead,
group/ArgPack ambiguity, and `where`/`acquire` non-recognition. A formal upper
bound should still be specified before future closure-head clauses are added.

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

Lexer golden tests (9 cases), parser golden tests (154 cases), and diagnostic
golden tests (27 cases) exist.

Remaining test coverage gaps are tracked as v0.1 implementation work, not as
workspace-readiness uncertainty. This entry is closed.

---

## 11. Operator precedence relative to pipe and whitespace auto-pipe

**Status:** Resolved

**Resolution:**
Ordinary operators bind more tightly than both whitespace auto-pipe and `|>`.
Operator precedence is a segment-local sugar layer inside the existing
pipe/segment architecture, not a traditional C-like expression model.

**Implementation status:**
Implemented in parser phase 4 as raw AST sugar.

---

## 12. Angle brackets outside deduce-list contexts

**Status:** Resolved

**Resolution:**
`<...>` remains a `DeduceList` only in strong binding contexts. In
expression/operator contexts, `<`, `>`, `<=`, and `>=` are operator spellings.
`<>` has no generic-call meaning.

**Implementation status:**
Implemented in parser phase 4. `<` and `>` are expression operators outside
strong binding contexts and do not introduce generic-call syntax.

---

## 13. Postfix operator suffix composition

**Status:** Resolved

**Resolution:**
Postfix unary operators compose in the atom suffix loop with `::`, `.`, and
`..`. They do not terminate suffix parsing; `obj!.field` has the shape
`(obj!).field`.

**Implementation status:**
Implemented in parser phase 4 at the `OperatorExprAst` layer.

---

## 14. Operator names in binders and paths

**Status:** Resolved

**Resolution:**
Operator names may appear as binder names and as path leaves:
`BinderName := Name | OperatorName` and `PathLeaf := Name | OperatorName`.
Operator names may only be leaves, not namespace-like intermediate path nodes.

**Implementation TODO:**
Implemented in parser phase 4.1 as raw AST preservation. Operator lookup,
lowering, overload resolution, and alias binding remain future work.

**Known syntax limitation:**
The `<` spelling is not currently accepted as a simple operator binder after
`let`. In that position, `<` starts the strong-context extract-let deduce list,
so `let <: ...` follows extract-let recovery instead of `BinderName::Operator`.
The `>` spelling does not have this conflict. A future phase needs an escaping
or dedicated disambiguation rule if `<` must be declared as an operator binder.

---

## 15. Comparison, equality, and compound-looking chaining

**Status:** Resolved

**Resolution:**
Comparison, equality, and compound-looking operators are non-associative in
this phase. Ungrouped chains such as `a < b < c`, `a == b == c`, and
`a += b += c` require explicit grouping.

**Implementation status:**
Implemented in parser phase 4 with `ChainedNonAssociativeOperator`.

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

---

## 20. Canonical skeleton admissibility

**Status:** Open

**Current v0.1 decision:**
The parser preserves all canonical skeleton shapes (names, wildcards, literals,
paths, argpacks) as raw AST. The Hole/NodeName distinction is a parse-time role
marker.  No semantic matching, destructuring, equality, constructor, or
admissibility semantics are assigned to any skeleton shape.

**Why it does not block v0.1:**
v0.1 is a syntax frontend only.  The canonical skeleton grammar is broad enough
to capture extraction syntax for later semantic interpretation.  Whether a
particular shape (literal in skeleton, bare node-name, nested argpack) is
admissible, produces a constraint, or is rejected by a future semantic match
is a deferred design decision that does not require changing the AST.

**Future stage:** v0.4 (canonical form specification) or v0.7 (type/kind/checking
design).

---

## 21. Lexical alias binding and entity references

**Status:** Documentation phase complete (Phase 4.2 / 4.3 design); implementation unresolved

**Current v0.1 decision:**
`EntityRef` is documented as future compile-time entity reference syntax in
`spec/entity-ref-design.md`. `let binder === EntityRef` is documented as future
lexical alias binding syntax in `spec/entity-alias-design.md`. Phase 4.3
completes the design documentation for alias binding: surface grammar, lexical
scope rule, distinction from ordinary `let`, ordinary name and operator alias
rules, `===` delimiter semantics, future diagnostics sketch, and full
parser/semantic boundary.

The current parser does not accept `===`, does not parse `EntityRef`, and does
not build `LetAliasAst`. The current lexer may tokenize `===` as `==` followed
by `=`; a later alias-parser phase must update lexer maximal-munch rules.

The intended boundary is syntax preservation only:

```text
EntityRef ::= EntityPath
EntityPath ::= EntityPathSegment ("::" EntityPathSegment)* "::" EntityPathLeaf
              | EntityPathLeaf
EntityPathSegment ::= Name
EntityPathLeaf ::= Name | OperatorName

AliasBinding ::= "let" AliasBinder "===" EntityRef
AliasBinder ::= Name | OperatorName
```

The right-hand side is a compile-time entity reference, not `PipeExpr`,
`ArgPack`, `ClosureAst`, an operator expression, or any runtime expression.

Operator aliases are restricted: the binder operator identity must match the
target leaf operator identity (`spelling + fixity + arity`). This validation is
deferred to a future static validation or name-resolution-adjacent phase.

**Why it does not block v0.1:**
v0.1 does not implement entity references, name lookup, namespace resolution,
dependency resolution, or import semantics. Phase 4.1 supplies the
operator-name syntax that alias binding depends on. Phase 4.2 and 4.3 document
the design boundaries.

**Future stages:** Phase 4.4 may optionally preserve raw alias-binding AST if
explicitly assigned. See also open questions 22–25 below.

---

## 22. Operator alias identity mismatch: diagnostic phase

**Status:** Open

**Current Phase 4.3 design:**
The operator alias rule requires `spelling + fixity + arity` match between
binder and target leaf. The design document recommends deferring the full
identity check to a static validation or name-resolution-adjacent phase.
A first-pass spelling-only comparison is possible as optional future parser
validation.

**Question:** Should operator alias identity mismatch be a parser diagnostic
(spelling-only), a static semantic diagnostic (full identity), or deferred
to name resolution?

**Why it does not block v0.1:**
No alias parsing exists in v0.1. The answer affects future implementation
ordering only.

**Future stage:** Phase 4.4 (alias parser preservation) or later
name-resolution design.

---

## 23. Alias binding position: all forms or top-level only

**Status:** Open

**Question:** Should alias bindings be allowed in all form positions (top-level,
inside closures, inside expressions) or only at top-level / namespace-level
positions?

The current Phase 4.3 design defines lexical scoping but does not constrain
where alias bindings may appear syntactically. This decision affects parser
state management and scope nesting.

**Why it does not block v0.1:**
No alias parsing exists.

**Future stage:** Phase 4.4 (alias parser) or later scope/semantic design.

---

## 24. Alias binding with `guard` or `with`

**Status:** Open

**Current Phase 4.3 recommendation:** Alias binding should not permit `guard`
or `with`. Alias bindings have no runtime value, no drop obligation, and no
lifetime dependency.

**Question:** Could future alias binding semantics justify a `guard` or
`with` clause (e.g., compile-time alias ordering or dependency)?

**Why it does not block v0.1:**
No alias parsing exists. The current recommendation is documented but not
binding on future design.

**Future stage:** Phase 4.4 or later scope/semantic design.

---

## 25. Alias binding visibility and export modifiers

**Status:** Open

**Question:** Should alias binding have a visibility or export modifier (e.g.,
`public`/`private`/`restricted`)?

The current Phase 4.3 design does not include visibility modifiers for alias
bindings. Access control and namespace export are documented as namespace-graph
and resolver concerns in `spec/library-namespace-design-note.md`. Whether alias
bindings need source-level visibility annotations is an open namespace design
question.

**Why it does not block v0.1:**
No alias parsing or namespace resolution exists.

**Future stage:** Namespace assembly phase or later language design.

---

## 26. Alias RHS operator leaf continuation after newline

**Status:** Open (Phase 4.4.1 observation)

**Context:** When an alias binding ends with an operator leaf followed by a
newline, the current form-boundary rules treat the newline as a form separator:

```text
let x === a
+ b
```

Because `+` is an operator token and operator tokens are general continuation
tokens in the form-boundary system, the newline before `+ b` may not be
promoted to a form separator. This means `+ b` could be treated as a
continuation of the alias RHS rather than a separate form. The `is_alias_rhs_boundary`
check in Phase 4.4.1 uses `!is_continuation_token(next)` to guard against this,
so `+ b` on the next line would NOT be treated as a new form starting at `+`.

If a future design wants `+ b` on a new line to be a separate expression form
rather than part of the alias RHS, the form-boundary continuation-token rules
would need a broader change. This remains documented as an open edge case.

**Why it does not block v0.1:**
The existing newline-promotion rules already define this behavior; expanding
them to let operator tokens participate in form-boundary separation is a
general parser question, not alias-specific.

**Future stage:** v0.2 (form boundary robustness) or later expression design.
