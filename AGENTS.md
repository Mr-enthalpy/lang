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
owner named there for the concepts being changed. Static evaluation, compile
instances, name production, structural interpretation/type formation and dot
navigation are jointly owned by
`spec/design/unified-static-name-and-structural-semantics.md`. For implementation
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
- Calls preserve P |> E == P E, never E P; ()f, x f and (x,y) f keep pipeline direction.
- Products participate in the documented expression/call-binding skeleton.
- `{ ... }` in atom position is an in-place closure with no head.
- A headed closure without `=>` is in-place; `=>` forms an ordinary closure.
- `<...>` is a DeduceList only in documented strong binding contexts.
- `let <> P` is binderless Pattern material; `let _ P` contains a wildcard.
- `|> P { ... }` preserves the callable head (self, <> P).
- Value-side expressions and Pattern-side material remain distinct.
- `let binder === PathMaterial` preserves a lexical Path alias: form the RHS once
  in the old Path environment, store material, never cache terminal binding identity.
- `return`, `else`, `match`, `if`, `drop`, `move`, `sync`, `effect`, `fn`,
  `type`, `meta`, `runtime`, `compile`, `namespace`, and `struct` are not lexer
  keywords.
- The parser must not create semantic declarations such as `FnDecl`,
  `StructDecl`, `ImportDecl`, HIR, MIR, or codegen nodes.
- Invalid input should produce AST recovery nodes plus spanned diagnostics.

## Canonical semantic invariants

- `Object = <Val1?, Pattern, Val2>`; ordinary normalization observes all three.
- Pattern applicability and extraction come from `R_Gamma(P,c,rho)`.
- Construction and consuming extraction commit complete Object identities atomically
  after Pre. Incremental material and Uninitialized Places are not partial Objects.
  Destructors continue over complete extracted children with ordinary lifecycle rules.
- `tau = bind alpha.<Core(tau), V_tau[alpha]>`; `V_tau` is immutable.
- NameBinding, named type, OverloadGroup, Place, and TypeValueId are distinct.
- Ordinary Val2 member formation accepts terminal selector (), supplying
  ordinary-value callability without another registry. TypeAdd changes V_T only;
  neither axis implies the other. TypeMember requires an ordinary callable with
  present Val1, its target classifier home and non-generative V_T registration.
  Ordinary let f=C binds/installs tau_C:type. Established same-name closure
  buckets consume ClosureMaterial to form one c_C^T per declaration, against a
  common snapshot, without a first sibling, tau_C insertion or bulk V_tau import.
  Delta_v^value and Delta_C,T^call retain distinct consumer roles.
  Replication witnesses apply to already formed ordinary callable members;
  known-target initial formation directly creates c_C^T.
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
- OpenPolicy = {open, close} is independent from PolicyMode = {const, mut}.
  An omitted mode completes to mut exactly when OpenPolicy=open and current
  OpenHere is available; otherwise it completes to const. Unknown required
  OpenHere is unavailable. OpenHere, Writable, PolicyMode, construction
  authority and lifetime remain independent judgments.
- Type +=/-= changes only V_tau under Writable, OpenHere, complete-type /tau
  classifier home and non-generative registration. Named Val2 residency is neither
  required nor implied; classifier navigation is not callable-value navigation.
  Witnessed anchored replication never reparents an existing value.
- Every compile call forms a stable CompileInstance from parent owner, selected
  CompilePartner and canonical inputs. Its ordinary self-name is readable without
  reentry. Result kind is determined by result structure: an instance-open type
  may escape only as the single self-rooted complete type, which exposes the
  instance NameExpr. In-place closures retain ordinary lexical/Self/navigation
  boundaries and use admitted dependencies to reach enclosing instances.
- Stage = {compile, seal, runtime}. P2 is evaluation horizon; P1/Pout producer
  visibility, OpenPolicy, InputAdmissible, migration and Ready are independent.
  Runtime P2 defaults omitted P1 to runtime; seal defaults to seal.
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
- Policy preference, CapabilityRealization, Writable, and DynamicLegality are
  independent judgments.
- Output demand is total before maxima; selected failure never reopens.
- Policy migration is direct, same-Type, candidate-driven, and existing-first.
- Abstract literals form before concrete construction.
- Structural interpretation forms complete types directly. struct:type->type
  adds standard ordinary helpers while preserving structural registration;
  *:type x type->type composes types and *=:type ref x type->unit performs the
  ordinary read-transform-write update under its normal Pre premises.
- InvocationResult is the single semantic result envelope.
- Lifecycle facts are relative to one SemanticContinuation. Killable is instance-local, MoveEffect is fixed, Movable is frontier legality.
  Move does not imply Kill; Preserve Move is not copy and invokes no clone.
  Pass=Move; copy-derived paths are share/rebind -> selected clone -> fresh complete result -> Move.
  Lowering retains the selected ordinary producer actions and terminal Move, with no opaque copy-producing action.
  Cleanup points are fixed under all constraints, then remaining unordered same-point
  events use reverse declaration order; the full sequence precedes observation.
  Killing move adds no
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
  written const/mut are explicit, and a formal-local hole is ordinary Pattern deduction.
  Default completion is separate. Inner-call selection seals before outer use.
  Omitted ReturnTarget selects the outermost enclosing function layer after an event
  exists. Implicit ReturnEvent is separate: non-tail expressions require unit via
  UnitDiscard; continuation-path tail unit falls through, non-unit synthesizes return.
  Explicit unit return still returns; AST-list position is not semantic tailness.
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
- Requested-name producers are ordinary compile computations over a requested
  NameValue and may realize any ordinary resident. Dot enters the generative
  field::adl overload family: receiver candidates perform ordinary method calls,
  type-valued receiver candidates advance NameExpr paths. x.field lowers through
  x |> .field to field::adl. e..field(args) is exactly e |> .field(args).
  Generated occurrences supply no structural-registration evidence. A T -> F
  accessor is affine value access; it does not imply parent death or leave a
  partial aggregate.
- Close freezes non-generative registered structure, not future ordinary generated
  Val2 realization. Such results reopen no construction view and do not alter old
  snapshots. Current Norm/only_val2 observations remain continuation-relative.

- Product layers with all direct entries named are unordered; any bare entry
  makes that whole layer ordered. Nested layers decide independently. Unordered
  to bare sequence requires named extraction and explicit ordered assembly.
- Read_name completes full NameExpr computation to NameValue. e# projects that
  result and blocks only resident reading. Structural bare names share one
  navigation-completion law across Path and extraction consumers. $ is a stable
  interpretation-polarity flip: Interpret(e$,C)=Interpret(e,Flip(C)), Flip^2=Id,
  with arbitrary source nesting. $ is independent from resident Read and Stage.
  Path support preserves Hole identities; ADL dynamic path segments use
  ((field#)[0])$::t.
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
  Non-MetaDecl DependencyMaterial is ExplicitDeps union AutomaticDeps, classified
  per occurrence; resolved explicit capture binders replace corresponding outer
  observations. Ordinary => closures may have both. InPlace excludes explicit
  clauses, but automatic dependencies do not imply InPlace. Placement and
  explicit/automatic origin supply no applicability, specificity or preference;
  otherwise tied distinct candidates follow ordinary ambiguity.
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
