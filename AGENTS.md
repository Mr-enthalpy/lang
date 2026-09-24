# Agent instructions for `lang`

## Read first

For every task, read:

```text
README.md
spec/README.md
spec/public/normalized-surface-semantics.md
spec/public/agent-interpretation-guide.md
spec/contracts/raw-ast-contract.md
spec/planning/open-questions.md
```

For semantic work, also read `spec/design/README.md` and every canonical topic
owner named there for the concepts being changed. For implementation
sequencing, read `spec/planning/roadmap.md`.

Documents under `spec/history/**` are non-authoritative and are read only when
the user explicitly asks for historical analysis.

## Subagents

Use subagents as read-only scouts for broad cross-file searches, independent
verification, and large peripheral modules. Read foundational documents and
the exact code you will modify yourself.

- Give each scout a self-contained scope and request `file:line` evidence.
- Spawn with `fork_turns = "none"`; do not reuse or retask a scout.
- After spawning, wait until every relevant scout reaches a terminal state.
- Treat results as compressed leads; verify decisive locations directly.
- The primary agent owns edits, design choices, and final verification.

## Architecture

```text
source text
  -> weak tokens
  -> Raw AST
  -> Normalized AST
  -> typed owner / namespace resolution
  -> canonical semantic evaluation
  -> InvocationResult
```

Raw AST preserves source and recovery. Normalized AST is syntax-directed and
non-semantic; it is not HIR. Semantic meaning never feeds back into lexing,
parsing, or normalization.

The current semantic universe is defined only by the canonical topic owners.
If a closed relation has no connected consumer, leave that operation
unsupported or return the appropriate Diagnostic/Residual. Do not invent an
alternate relation or identity.

## Frontend invariants

- The lexer is weak. Contextual words are `Name` tokens.
- The parser owns syntax shape, not semantic meaning.
- Parse left to right without semantic backtracking.
- Traditional `f(args)` call syntax does not exist.
- Products participate in the documented expression/call-binding skeleton.
- `{ ... }` in atom position is an in-place closure with no head.
- A headed closure without `=>` is in-place; `=>` forms an ordinary closure.
- `<...>` is a DeduceList only in documented strong binding contexts.
- `let <> P` is binderless Pattern material; `let _ P` contains a wildcard.
- `|> P { ... }` uses the binderless headed in-place closure shape.
- Value-side expressions and Pattern-side material remain distinct.
- `let binder === EntityRef` is syntax preservation only until its local
  lexical resolver consumer is connected; it creates no semantic entity.
- `return`, `else`, `match`, `if`, `drop`, `move`, `sync`, `effect`, `fn`,
  `type`, `meta`, `runtime`, `compile`, `namespace`, and `struct` are not lexer
  keywords.
- The parser must not create semantic declarations such as `FnDecl`,
  `StructDecl`, `ImportDecl`, HIR, MIR, or codegen nodes.
- Invalid input should produce AST recovery nodes plus spanned diagnostics.

## Canonical semantic invariants

- `Object = <Val1?, Pattern, Val2>`; ordinary normalization observes all three.
- Pattern applicability and extraction come from `R_Gamma(P,c,rho)`.
- `tau = bind alpha.<Core(tau), V_tau[alpha]>`; `V_tau` is immutable.
- NameBinding, named type, OverloadGroup, Place, and TypeValueId are distinct.
- Same-name construction synthesizes a type's V_tau; ordinary lexical let does
  not aggregate. Structural P let name::path:t creates typed NameExpr and an
  Uninitialized Place, not a resident or ref; omitted :t means :type.
  Qualified formation uses resolved structural root identity and the current
  resident type's OpenHere, selector/non-retention/access/path/type checks;
  parent Writable and parent mut type ref are not premises. Equal type values
  do not merge NameCoords or Places. Ref acquisition is a separate judgment.
  Initializer-free P let name:t shares typed-name formation at a lexical
  destination. P let name=rhs is a complete lexical binding with RHS inference;
  the initializer-free structural default does not apply to it.
  Explicit ref then ordinary write initializes; Close requires retained members
  being published initialized, not every future coordinate realized. NameBinding is not a wrapper Object.
  First initialization uses Place authority independently of DeclaredPolicy,
  including const; commit consumes it. Saved initial refs do not authorize
  replacement, and first write never reads nonexistent resident policy.
  TypeRole(Q) iff Pure(Q) iff Val1?(Q)=absent; complete TypeValueRole(tau)
  iff WellFormedTau(tau). HasRegisteredSelfConstruction defines only
  SelfConstructible, not type identity. Complete tau consistency checks both registered
  closure roles' /tau homes separately.
- Contextual meta qualification is limited to type/type ref, not a fourth
  PolicyMode. Initialized type names retain direct mut borrowing and explicit
  meta-ref-to-mut confirmation; both require current OpenHere, target Writable
  and ordinary capability/access/lifetime. Same-position routes are coherent,
  not implicit chains. Meta refs retain their actual Place and original borrowed
  generation/opening subject. Close defeats both mutable routes and saved-ref
  writes. InitialTypeSlotRef remains separate one-shot initialization authority.
  Frozen generated Val2 realization after Close invokes none of these ref paths.
- Type +=/-= changes only V_tau under Writable, OpenHere, complete-type /tau
  classifier home and non-generative registration. Named Val2 residency is neither
  required nor implied; classifier navigation is not callable-value navigation.
  Witnessed anchored replication never reparents an existing value.
- OverloadGroup aggregation buckets whole bound type snapshots, never Core classes,
  has its own entry algebra and requires Writable.
  Ordinary meta constructs an instance name/type tau_M outside input structure;
  arbitrary payloads occupy ordinary Val2. V_tau callability registration and
  Pattern-role registration are independent; neither follows from Val2 presence.
  Input identity retains observed name/subject dependencies; output openness follows
  their meet. P1 meta let retains the instance under OpenHere, which governs mut
  acquisition; plain let completes/closes it. P2 meta remains evaluation stage.
  Invocation caches retain instances/member Places and current state. A consumes
  these facilities; saved references recheck the original source at write Pre.
- Stage is one atom; static stages are pairwise incomparable. P2 is horizon,
  P1/Pout producer visibility; InputAdmissible, migration and Ready are separate.
  Runtime P2 defaults omitted P1 to runtime; seal defaults to seal.
- Main has runtime horizon; stable roots imply no active MetaDom. Actual
  meta/seal frames exclude seal/meta work through helpers and cache hits.
- Name resolution happens once before R_vis evidence and ordinary C_sigma
  preparation. Hard A, fallback suppression and Policy/Pattern order seal
  (candidate, projection, frame); runtime preserves that origin.
- Ordinary calls use x -> Type(x) -> associated Val2[()], with self=x.
  Type calls union the ordinary implementation entries for every c in V_tau before
  one selection of (c, Impl, projection, frame), with self=c.
  AssociatedNamespace(T)=MemberScope(Core(T)); AssociatedName(T,s) is its
  NameCoord. This is distinct from TypeMemberScope(T)=/tau(T).
  V_tau registration, Val2 residency, Pattern registration and ConstructEdge
  remain independent; no self-construction witness proves type callability.
- `PolicyMode = {const, plain, mut}`; plain is a primitive point.
- Policy preference, CapabilityRealization, Writable, and DynamicLegality are
  independent judgments.
- Output demand is total before maxima; selected failure never reopens.
- Policy migration is direct, same-Type, candidate-driven, and existing-first.
- Abstract literals form before concrete construction.
- `OpenHere`, Writable, PolicyMode, and construction authority do not imply one
  another. `extend` is pure; `inject` is read + extend + write.
- `InvocationResult` is the single semantic result envelope; `struct` returns
  complete tau.
- Lifecycle facts are relative to one SemanticContinuation. Killable is instance-local, MoveEffect is fixed, Movable is frontier legality.
  Cleanup/with placement is fixed before observation; killing move adds no
  old-generation destructor; Pre precedes mutation; Post describes committed success.
- Color vocabulary is extensible and relation rows are explicit and directed.
- SafetyPolicy is orthogonal to PolicyMode; unsafe admits compatible external
  semantic axioms, never missing Pre facts or private optimizer assumptions.
- Child-directory names normalize to ordinary fresh-name actions followed by
  explicit ref and one-shot directory type initialization before body evaluation; root and filenames add no segment.
- Host capabilities return ordinary Objects. Physical normalization and build
  facilities introduce no semantic facts; E alone owns meaning.
- E is idempotent and saturates ready actions without rewriting continuations.
  Optimizer rewrites require revalidation by affected semantic projections.
- P1/P2 are independent; Pin permits explicit stage/mode constraints and holes
  over P2; Pout.stage=P1.stage. Bare let writes no override;
  written plain is explicit, and a formal-local hole is ordinary Pattern deduction.
  Default completion is separate. Inner-call selection seals before outer use.
  Implicit return targets the outermost enclosing function layer.
- Done is internal chain/target completion, never an Object or Pattern.
  Split/D is restricted; residual escape separately reads ordinary meta facts.
  Return has no synthetic local unit contribution.
- Every legal completed closure expression returns full tau_C through ordinary
  struct, with tau_C/c_C/A_C/() distinct and a finite implementation leaf.
  File implementation-layer let installs at the established package root;
  true lexical local let remains binding. Contribution repair preserves all legal binding/shadow/write/group
  actions and never retries failed execution as contribution.
- NameCoord precedes Retained/typed Place realization; Fresh means not Retained.
  Ordinary name writes may change Val2(Core) without either registration.
  Pattern-registered extension uses extend/inject; TypeAdd changes V_tau only.
- Meta call/value and Pattern/name declarations are equal surface projections.
  Grammar-fixed operator use selects operator[op]; dot .op selects op::adl;
  explicit paths remain explicit. OperatorNameValue reads without recursive
  dispatch. OG_s retains spelling and selects the current slot. Ordinary
  call/extract/generative relations apply;
  generated occurrences supply no Pattern or V_tau registration evidence.
- Close freezes non-generative registered structure, not future ordinary generated
  Val2 realization. Such results reopen no construction view and do not alter old
  snapshots. Current Norm/only_val2 observations remain continuation-relative.

- Product layers with all direct entries named are unordered; any bare entry
  makes that whole layer ordered. Nested layers decide independently. Unordered
  to bare sequence requires named extraction and explicit ordered assembly.
- Structured Path precedes external Read. Textual roots resolve at actual use;
  explicit value/reference roots retain anchors and dependencies. # quotes Path;
  general $ consumes ready Pattern material while preserving Hole identities.
- Public Policy pair syntax is retired; internal value/type observations remain
  independent and share the source evaluation edge only for direct projections.
  Concrete atoms, omission, holes and splice remain distinct.
- Terminal ReturnPattern/Pout demand precedes immediate root maxima; established
  outer P1/P2 jointly constrain inner positions, without an implicit semantic temp.
- Dependencies separate requirements, semantic realization and layout. [] is one
  source. Realization follows the source occurrence's selected ordinary action,
  uniquely up to observational equivalence; distinct candidates use ordinary
  preference or ambiguity, never backend choice. Initialization is once per formation.
  Projections never recapture.
  In-place syntax forms dependencies automatically and produces an ordinary
  first-class result. Invocation does not recapture; binding, transfer and outer
  writes use actual access/capability/lifetime, with no placement-based veto.
  Lifetime persistence/escape refinement retains all ordinary checks; tau status
  grants no global lifetime and local dependency grants no universal prohibition.

## Scope and Open questions

Do not close a question listed in `spec/planning/open-questions.md` for
implementation convenience. Use opaque carriers and extension interfaces.

Source wiring may be incomplete. Missing wiring means unavailable behavior,
not permission to substitute another semantic implementation.

## Editing

- Preserve unrelated user changes in a dirty worktree.
- Use `rg` / `rg --files` for searches.
- Use `apply_patch` for edits; bulk mechanical renames may use repository-safe
  file operations.
- Update current contracts and tests with parser or diagnostic behavior.
- Do not rewrite files under `spec/history/**` to describe current behavior.
- Current source, tests, and docs use only positive current terminology.

## Tests

Every syntax rule needs golden coverage. Semantic changes need positive,
negative, identity/equality, no-reopen, non-derivability, and
authority-uniqueness tests as relevant.

After changes run:

```text
cargo fmt --all
cargo test
```

## PR hygiene

For a new unrelated task, run `.git/local/pr-task-gate.ps1` when available. Do
not run it for corrections to the current PR.

When asked to publish changes, inspect status/diff, commit intentionally, push
with upstream tracking, and use `gh` for draft PR creation or updates.
