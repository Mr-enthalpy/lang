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

**Future stage:** v0.5 (Normalized AST Stabilization) or v0.9 (type/kind checking design).

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

**Future stage:** v0.5 (Normalized AST Stabilization) or later language design.

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

**Future stage:** v0.9 (type/kind checking design) — tuple types or value types may
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

**Future stage:** v0.3 (Normalized AST Specification) or v0.4 (Raw AST → Normalized AST Prototype).

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

**Future stage:** v0.3 (Normalized AST Specification) or v0.8 (closure materialization).

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

**Future stage:** v0.5 (Normalized AST Stabilization or frontend robustness) — document the exact maximum lookahead
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

**Future stage:** v0.6 (canonical form specification).

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

**Future stage:** v0.11 (or earlier semantic design stages v0.7–v0.9).

---

## 9. Future always-NLL CFG requirements

**Status:** Deferred

**Current v0.1 decision:**
No CFG is built. The parser does not construct a control-flow graph.

**Why it does not block v0.1:**
Ownership and lifetime analysis is out of scope for v0.1. The raw AST
contains sufficient structure (form order, closure bodies, and explicit
`with { ... }` syntax) for future passes.

**Future stage:** v0.10 (ownership/NLL/drop design).

---

## 10. Workspace readiness: cargo test passes with minimal crates

**Status:** Resolved

**Resolution:**
The workspace is ready. `crates/lang_syntax`, `crates/lang_cli`, and `xtask`
exist as valid workspace members. `cargo check --workspace` passes.

Golden test counts are tracked in `spec/implementation-status-v0.1.md`.
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

**Implementation status:**
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

**Future stage:** v0.9 (type/kind checking design) or v0.11.

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

**Future stage:** v0.5 (Normalized AST Stabilization or frontend robustness) or later numeric literal design.

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

**Future stage:** v0.9 (type/kind checking design) or later name-resolution
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

**Future stage:** v0.6 (canonical form specification) or v0.9 (type/kind checking
design).

---

## 21. Lexical alias binding and entity references

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
Raw alias parsing exists; the answer affects future implementation ordering only.

**Future stage:** Later name-resolution design or alias-validation stage.

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
Raw alias parsing exists; this decision affects future scope/semantic design
only.

**Future stage:** Later scope/semantic design or alias-validation stage.

---

## 24. Alias binding with `with`

**Status:** Open

**Current decision:** Alias binding does not permit `with`. `guard` is not a
let attribute and has no alias-specific syntax. Alias bindings have no runtime
value, no drop obligation, and no lifetime dependency.

**Question:** Could future alias binding semantics justify a `with { ... }`
clause (e.g., compile-time alias ordering or dependency)?

**Why it does not block v0.1:**
Raw alias parsing exists; the current recommendation is documented but not
binding on future design.

**Future stage:** Later scope/semantic design or alias-validation stage.

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
Raw alias parsing exists; namespace resolution does not.

**Future stage:** Namespace assembly phase or later language design.

---

## 26. Alias RHS continuation-token after newline

**Status:** Open (Phase 4.4.1 observation)

**Context:** When an alias RHS is followed by a newline and the next line begins
with an operator token, the current form-boundary rules may treat the operator
line as a continuation rather than a new form:

```text
let x === a
+ b
```

Because `+` is an operator token and operator tokens are general continuation
tokens in the form-boundary system, the newline before `+ b` may not be
promoted to a form separator. The `is_alias_rhs_boundary` check in Phase 4.4.1
uses `!is_continuation_token(next)` to guard against this, so `+ b` on the next
line would NOT be treated as a new form starting at `+`.

If a future design wants `+ b` on a new line to be a separate expression form
rather than part of the alias RHS, the form-boundary continuation-token rules
would need a broader change. This remains documented as an open edge case.

**Why it does not block v0.1:**
The existing newline-promotion rules already define this behavior; expanding
them to let operator tokens participate in form-boundary separation is a
general parser question, not alias-specific.

**Future stage:** v0.5 (Normalized AST Stabilization or frontend robustness) or later expression design.

---

## Documentation reset debt

Items resolved during the documentation reset pass. Recorded here for audit.

| Item | Implementation status | Spec state (before reset) | Action taken | Blocking |
|---|---|---|---|---|
| Operator syntax added after initial v0.1 boundary | Implemented as raw AST sugar | AGENTS.md said "do not implement operator syntax" | Updated AGENTS.md, SKILL.md | No |
| Alias parser preservation after entity-alias documented as future | Implemented as raw AST preservation | AGENTS.md, SKILL.md, README.md said "future only" | Updated all entry docs + entity-alias-design.md | No |
| `where`/`acquire` reserved but not active | Reserved, not parsed in closure head | Already documented correctly | Confirmed; marked `reserved-not-active` in implementation-status | No |
| EntityRef general design vs alias-RHS subset | AliasRhsEntityRef implemented; GeneralEntityRef future | entity-ref-design.md said "not implemented" | Split into status banner distinguishing AliasRhsEntityRef vs GeneralEntityRef | No |
| `InvalidAliasBinder` diagnostic reserved but not emitted | In DiagnosticCode, never triggered | Undocumented as reserved | Marked "reserved; not currently emitted" in diagnostics spec | No |
| `UnusedClosureAst` diagnostic optional / not guaranteed emitted | In DiagnosticCode, may never trigger | Documented as optional | Clarified "not guaranteed to be emitted" in diagnostics spec | No |
| Right-target subsegment AST shape | Flat representation; future may nest | Already open question §4 | No change needed | No |
| Form boundary promotion rules | Provisional rules implemented | Already open question §2 | No change needed | No |

---

## Normalized AST design questions

These questions are deferred to v0.3–v0.5. They do not block the current
N0–N1 documentation pass (Raw AST contract freeze).

### N-AST-1. Exact Normalized AST node set

**Status:** Open

**Question:** What are the exact Normalized AST node types? Candidates:
normalized call, normalized pattern, normalized declaration. Should there
be a single unified expression node or distinct per-form nodes?

**Why it does not block N0–N1:** The Raw AST contract only documents invariants;
Normalized AST node types are a v0.3 specification detail.

**Future stage:** v0.3 (Normalized AST Specification).

---

### N-AST-2. Whether Normalized AST lives in `lang_syntax` or a new crate

**Status:** Open

**Question:** Should Normalized AST types and the normalization pass live in
`lang_syntax` (alongside Raw AST), or in a new crate (e.g., `lang_norm`)?

**Why it does not block N0–N1:** This is an implementation organization
question for v0.4.

**Future stage:** v0.4 (Raw AST → Normalized AST Prototype).

---

### N-AST-3. Whether raw-to-normalized dumps should be golden-tested

**Status:** Open

**Question:** Should the normalization pass produce stable dump output that
can be golden-tested alongside Raw AST dumps?

**Why it does not block N0–N1:** Golden testing strategy is a v0.4
implementation question.

**Future stage:** v0.4 (Raw AST → Normalized AST Prototype).

---

### N-AST-4. How to represent symbolic builtins introduced by desugaring

**Status:** Open

**Question:** Desugaring may introduce symbolic names (e.g., `operator::call`,
`member::lookup`, `pattern::bind`). How should these be represented in
Normalized AST — as reserved names, as a separate node type, or as
compiler-generated identifiers?

**Why it does not block N0–N1:** This is a v0.3 specification detail.

**Future stage:** v0.3 (Normalized AST Specification).

---

### N-AST-5. How to preserve source origins through desugaring

**Status:** Open

**Question:** Desugaring creates new AST nodes that did not appear in source
text. How should source spans and diagnostic attribution be preserved through
normalization?

**Why it does not block N0–N1:** Source origin preservation is a v0.3–v0.4
design question.

**Future stage:** v0.3 (Normalized AST Specification), v0.4 (prototype).

---

### N-AST-6. Whether right-target subsegments become nested call nodes

**Status:** Open

**Question:** Right-target subsegments (`f (a) g`) are currently flat in Raw
AST. Should normalization recursively nest them into explicit (sub-)call
nodes?

**Why it does not block N0–N1:** This is a v0.3 desugaring rule.

**Future stage:** v0.3 (Normalized AST Specification).

---

### N-AST-7. How to represent pattern normalization for let, params, returns, and canonical skeletons

**Status:** Open

**Question:** Extraction contexts (let, params, returns) use canonical
skeletons. How should normalization unify these into a single normalized
pattern form? Should deduce lists be merged into the pattern structure
or kept separate?

**Why it does not block N0–N1:** Pattern normalization is a v0.3 specification
detail.

**Future stage:** v0.3 (Normalized AST Specification).

---

### N-AST-8. How to represent alias declarations before name resolution

**Status:** Open

**Question:** Alias bindings (`let binder === EntityRef`) reference compile-time
entities that are not yet resolved. Should normalization preserve `EntityRefAst`
as-is in normalized alias declarations, or desugar it into a different form?

**Why it does not block N0–N1:** Alias normalization is a v0.3 specification
detail.

**Future stage:** v0.3 (Normalized AST Specification).
