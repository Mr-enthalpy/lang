# Open Questions

This document tracks unresolved, deferred, and partially-resolved design
questions for the `lang` language. They do not block v0.1 and should be
revisited in the appropriate future stage.

Resolved questions have been moved to `spec/resolved-questions.md`.

---

## 1. Future canonical value/type grammar

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

## 2. Future semantics of `return`, `else`, `match`, `effect`, `sync`

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

## 3. Future always-NLL CFG requirements

**Status:** Deferred

**Current v0.1 decision:**
No CFG is built. The parser does not construct a control-flow graph.

**Why it does not block v0.1:**
Ownership and lifetime analysis is out of scope for v0.1. The raw AST
contains sufficient structure (form order, closure bodies, and explicit
`with { ... }` syntax) for future passes.

**Future stage:** v0.10 (ownership/NLL/drop design).

---

## 4. Numeric selectors: positional access vs. general sugar

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

## 5. Float, scientific, and unit-adjacent numeric literals

**Status:** Partially resolved — bare `1.2` is decided; scientific/unit forms remain open

**Resolution for `Digit+ "." Digit+` (e.g. `1.2`):**
`1.2` is **member sugar**, not a float literal:

```text
1.2 ↦ MemberSugar { object: IntLiteral("1"), selector: NumericName("2") }
```

Float literals are not lexer/Raw-AST primitives in this design. There is no
`FloatLiteral` token or node. `1.2` lexes as `IntLiteral("1") · Dot ·
IntLiteral("2")` and folds through the ordinary `.`-suffix rule. Chains are
left-associated: `1.2.3 ↦ (1.2).3`. This is locked by golden tests
(`member_int_base`, `member_int_chain`, lexer `int_dot_int`). A "float" value
such as `1.2float32` arises naturally later as ordinary sugar/normalization,
not from a primitive token — so `1.2` never becomes a float token.

**Still open (scientific / unit-adjacent):**
The spellings `1.2ms`, `1e3ms`, `1.2e3`, and `1.2e3ms` are reserved for future
numeric literal design. The current parser must not force an interpretation of
these forms. The natural unit syntax `1ms` and `1 ms` remain equivalent as
`IntLiteral(1)` followed by `Name(ms)` at the non-trivia token/parser structure
level. No `UnitLiteral` AST node exists.

**Why it does not block v0.1:**
The lexer does not produce `FloatLiteral`, `ScientificLiteral`, or
`FloatScientificLiteral` tokens. Numeric tokens in selector position go through
the same token class but produce `NumericNameAst` rather than numeric literal
atoms.

**Future stage:** later numeric literal design (scientific/unit forms only).

---

## 6. Name-polymorphic lookup boundary

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

## 7. Canonical skeleton admissibility

**Status:** Open

**Current v0.1 decision:**
The parser preserves all canonical skeleton shapes (names, wildcards, literals,
paths, product extractions) as raw AST. The Hole/NodeName distinction is a parse-time role
marker.  No semantic matching, destructuring, equality, constructor, or
admissibility semantics are assigned to any skeleton shape.

**Why it does not block v0.1:**
v0.1 is a syntax frontend only.  The canonical skeleton grammar is broad enough
to capture extraction syntax for later semantic interpretation.  Whether a
particular shape (literal in skeleton, bare node-name, nested product extraction) is
admissible, produces a constraint, or is rejected by a future semantic match
is a deferred design decision that does not require changing the AST.

**Future stage:** v0.6 (canonical form specification) or v0.9 (type/kind checking
design).

---

## 8. Operator alias identity mismatch: diagnostic phase

**Status:** Open

**Current Phase 4.3 design:**
The operator alias rule requires `spelling + fixity + arity` match between
binder and target leaf, where fixity is `Binary` or `Postfix` (overloadable
fixities only). Prefix negative `-x` is a normalization-special-cased surface
sugar, not an overloadable operator identity; the `-` spelling in alias binder
or target position refers exclusively to binary minus. The design document
recommends deferring the full
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

## 9. Alias binding with `with`

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

## 10. Alias binding visibility and export modifiers

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

## Documentation reset debt

Items resolved during the documentation reset pass. Recorded here for audit.

| Item | Implementation status | Spec state (before reset) | Action taken | Blocking |
|---|---|---|---|---|
| Operator syntax added after initial v0.1 boundary | Implemented as raw AST sugar | AGENTS.md said "do not implement operator syntax" | Updated AGENTS.md, SKILL.md | No |
| Alias parser preservation after entity-alias documented as future | Implemented as raw AST preservation | AGENTS.md, SKILL.md, README.md said "future only" | Updated all entry docs + entity-alias-design.md | No |
| `where`/`acquire` reserved but not active | `where` reserved-inactive; `acquire` superseded | Previously both reserved | `where` stays reserved-inactive; `acquire` direction replaced by active `pre`/`post` head clauses (plus `require`/`lifetime pre`/`lifetime post`) | No |
| EntityRef general design vs alias-RHS subset | AliasRhsEntityRef implemented; GeneralEntityRef future | entity-ref-design.md said "not implemented" | Split into status banner distinguishing AliasRhsEntityRef vs GeneralEntityRef | No |
| `InvalidAliasBinder` diagnostic reserved but not emitted | In DiagnosticCode, never triggered | Undocumented as reserved | Marked "reserved; not currently emitted" in diagnostics spec | No |
| `UnusedClosureAst` diagnostic optional / not guaranteed emitted | In DiagnosticCode, may never trigger | Documented as optional | Clarified "not guaranteed to be emitted" in diagnostics spec | No |
| Right-target subsegment AST shape | Flat representation; future may nest | Already open question §4 | No change needed | No |
| Form boundary promotion rules | Provisional rules implemented | Already open question §2 | Replaced with strong-semicolon rule (§2). Newline promotion removed. | No |

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
