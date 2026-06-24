# Open Questions

This document tracks open design questions for the `lang` language. Each entry
specifies the stage at which the question becomes active.

Resolved questions have been moved to `spec/resolved-questions.md`.

---

## Current-stage questions (v0.1–v0.2)

### 1. Float, scientific, and unit-adjacent numeric literals

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

### 2. Operator alias identity mismatch: diagnostic phase

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

## Later-stage questions

These questions become active when their stage is reached. They do not
block the current stage.

### v0.3–v0.5: Normalized AST

#### N-AST-1. Exact Normalized AST node set

**Status:** Open

**Question:** What are the exact Normalized AST node types? Candidates:
normalized call, normalized pattern, normalized declaration. Should there
be a single unified expression node or distinct per-form nodes?

---

#### N-AST-2. Whether Normalized AST lives in `lang_syntax` or a new crate

**Status:** Open

**Question:** Should Normalized AST types and the normalization pass live in
`lang_syntax` (alongside Raw AST), or in a new crate (e.g., `lang_norm`)?

---

#### N-AST-3. Whether raw-to-normalized dumps should be golden-tested

**Status:** Open

**Question:** Should the normalization pass produce stable dump output that
can be golden-tested alongside Raw AST dumps?

---

#### N-AST-4. How to represent symbolic builtins introduced by desugaring

**Status:** Open

**Question:** Desugaring may introduce symbolic names (e.g., `operator::call`,
`member::lookup`, `pattern::bind`). How should these be represented in
Normalized AST — as reserved names, as a separate node type, or as
compiler-generated identifiers?

---

#### N-AST-5. How to preserve source origins through desugaring

**Status:** Open

**Question:** Desugaring creates new AST nodes that did not appear in source
text. How should source spans and diagnostic attribution be preserved through
normalization?

---

#### N-AST-6. Whether right-target subsegments become nested call nodes

**Status:** Open

**Question:** Right-target subsegments (`f (a) g`) are currently flat in Raw
AST. Should normalization recursively nest them into explicit (sub-)call
nodes?

---

#### N-AST-7. How to represent pattern normalization for let, params, returns, and canonical skeletons

**Status:** Open

**Question:** Extraction contexts (let, params, returns) use canonical
skeletons. How should normalization unify these into a single normalized
pattern form? Should deduce lists be merged into the pattern structure
or kept separate?

---

#### N-AST-8. How to represent alias declarations before name resolution

**Status:** Open

**Question:** Alias bindings (`let binder === EntityRef`) reference compile-time
entities that are not yet resolved. Should normalization preserve `EntityRefAst`
as-is in normalized alias declarations, or desugar it into a different form?

---

### v0.6+: Canonical form specification

#### How should canonical value/type grammar be designed?

**Status:** Open (active at v0.6)

**Current v0.1 foundation:**
Canonical skeletons use the grammar defined in section 6 of
ast-construction-v0.1.md. This grammar is provisional and may be revised
when value/type canonical forms are designed.

---

### v0.10+: Ownership and NLL

#### How should the NLL CFG be structured?

**Status:** Open (active at v0.10)

**Current v0.1 foundation:**
No CFG is built. The raw AST contains sufficient structure (form order,
closure bodies, and explicit `with { ... }` syntax) for future passes to
construct a control-flow graph.

---

### v0.11+: Control-flow and effect semantics

#### How should `return`, `else`, `match`, `effect`, `sync` be semanticized?

**Status:** Open (active at v0.11)

**Current v0.1 foundation:**
These are ordinary `Name` tokens at the lexical and parser level. No special
AST nodes exist for them. The v0.1 frontend faithfully preserves these names
in expression AST.

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
