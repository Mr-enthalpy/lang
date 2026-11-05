# Lifetime program consumer handoff

Status: design interface for future common-E wiring. This contract introduces
no new executable Rust API or result envelope. [Lifecycle semantics](../design/lifetime/lifetime-policy-and-overload-boundary.md)
owns the computation; [unsafe admission](../design/lifetime/unsafe-semantic-admission.md)
owns external fact sources. [Walkthroughs](../design/lifetime/lifetime-algebra-walkthroughs.md)
and [acceptance rows](../planning/canonical-semantic-conformance.md#lifetime-value-computation-and-library-design-acceptance)
exercise this handoff on paper. [Roadmap](../planning/roadmap.md#9-lifetime-value-program-wiring)
records missing consumers, independently of whether a carrier exists.

## Shared template and result boundary

Each consumer must fill this same template before implementation is claimed:

```text
Input material:
Existing action that determines it:
Kind of ordinary lang program consumed:
Output lifecycle material:
Commit/delivery boundary:
Failure or unavailable result:
Unconnected consumer dependencies:
Fidelity/composition obligations:
```

Inputs retain K/action/cut, actual names and generations, selected
candidate/projection/frame, dependency identities and semantic result consumer
where required. Display paths, equal values, bare identity numbers, generic
transaction storage and fixture imports grant none of that evidence.

An available check returning false rejects its requiring action before commit.
Unavailable required material returns the appropriate spanned Diagnostic;
`Unavailable` below names that semantic frontier, not a new InvocationResult
variant. Only an established remaining continuation can cross as Residual.
No failure retries a candidate, changes ABIKind, silently clones, manufactures
an unknown root, or publishes a success permission. All affected Pre precede
common commit; joint producer/lifecycle Post uses the same scratch publication.
Already committed history/Color and earlier external effects are not erased.

## LC1. Pattern and structure companion formation

- **Input material:** Actual complete formation material, registered
  ConstructEdge/ExtractEdge/FieldView witnesses, complete tau, selected member
  operations, identities and finite recursive source/access structure.
- **Existing action that determines it:** Ordinary Pattern/structural formation,
  registered composition or the actual helper augmentation, after ordinary
  selection. `$` alone supplies only polarity, not another action.
- **Kind of ordinary lang program consumed:** Formation/struct helper programs
  that extract real structure and install corresponding ordinary lifecycle
  members or registered forwarders through normal authority checks.
- **Output lifecycle material:** Paired name/field/operation material, origin
  and regular access descriptors with actual target parameters; enough for
  `(x.field*)@ = x@.field_lifetime*` for the selected admitted operation.
- **Commit/delivery boundary:** Formation's complete atomic publication includes
  companion facts and actual registrations. No partial Object or private map
  substitutes for a witness. Ordinary helper additions preserve structural roles.
- **Failure or unavailable result:** Failed formation publishes nothing;
  missing structural/partner consumers are Unavailable. No same-name ADL retry.
- **Unconnected consumer dependencies:** General S interpretation/formation,
  struct forwarding, Pattern registrations and lifecycle source producers.
- **Fidelity/composition obligations:** Commutation, identity preservation,
  actual registration, finite regular representation without merging names,
  and independence from layout or incidental Val2 presence.

## LC2. Whole-continuation projection and projection-complete clauses

- **Input material:** Established K, selected actions/MoveEffect and origins,
  guarded control, dependencies, actual activations, result delivery and fixed
  cleanup; projection-completion provenance for lifecycle-side programs. The
  walkthrough's source_frame and selected entries must come from this actual
  correspondence; a user-created Frame-shaped value supplies no action authority.
- **Existing action that determines it:** Ordinary E and the name/call/Policy/
  control consumers, not a textual scan or later machine schedule.
- **Kind of ordinary lang program consumed:** Selected lifecycle partners and
  ordinary structured-material assembly; directly written pre/post programs
  remain ordinary evaluation without being projected again.
- **Output lifecycle material:** Stage-forgotten action/name/region/Color/
  access/control material with the same selected correspondence. Unknown
  business values remain unknown while their established names can be used.
- **Commit/delivery boundary:** Synchronized projections of the same action
  and K. Material formation can precede machine execution when ready; guarded
  alternatives are descriptions, not committed alternative effects.
- **Failure or unavailable result:** Missing partner/control/name evidence is
  Unavailable. Actual unfinished work retains its K; unsupported syntax alone
  provides no Residual and cannot trigger @@ or recursive companion generation.
- **Unconnected consumer dependencies:** Common E, LC1, source dependency and
  control/completion consumers, full generation-continuation reification.
- **Fidelity/composition obligations:** Stage erasure retains selection,
  activation, control and effects; no re-selection, future-value path pruning,
  double projection, or global-lifetime inference from compile knowledge.
  The [expanded transcription](../design/lifetime/lifetime-algebra-walkthroughs.md#lifetime-transcription)
  must retain real Ordinary/InPlace scopes, recursive calls and exits. Its own
  ordinary lifecycle premises still terminate in the bootstrap basis.

## LC3. Internal actions and public transaction

- **Input material:** Selected ordinary action, exact implicit Param plus
  explicit input, generations/targets, origin/region/access, fixed MoveEffect,
  supplied operation Pre and fixed cleanup sequence.
- **Existing action that determines it:** Ordinary Move/ref/share/rebind/clone,
  complete construction or consuming extraction, non-consuming observation,
  call input and destructor continuation. Mechanical Pass=Move is already fixed.
- **Kind of ordinary lang program consumed:** Actual action partners and
  internal composite lifecycle programs, with explicit ordinary clone paths
  where selected; not one from call for every action.
- **Output lifecycle material:** Real sibling/child relationships, complete
  new generations and region boundaries, inherited Color and structured action
  history. Ordinary observation is not destructive extraction.
- **Commit/delivery boundary:** Common cleanup gate, all Pre, one existing
  action commit and joint Post. Whole-self/Param are one coherent transfer;
  Kill contributes no duplicate old-generation Drop.
- **Failure or unavailable result:** Before commit no state change; after a
  selected failure no fallback, reborrow, changed MoveEffect or implicit copy.
- **Unconnected consumer dependencies:** Source pass normalization, producer
  formation/origin/Color, ref/share/clone, NLL/with and destructor execution.
- **Fidelity/composition obligations:** Single consumption, target/generation
  retention, Preserve != clone, atomic complete identity and fixed same-cut
  order. Existing transaction tests cover supplied facts, not these producers.

## LC4. Contract expressions and four-way boundary consumption

- **Input material:** Clause expression, actual call mapping for inputs,
  captures, result positions and activation, entry assumptions and real world
  observation/update authority. Result positions are not premature Objects.
- **Existing action that determines it:** Ordinary selected invocation and
  compile instance/self-root result formation, associated state/name/Place
  operations and actual successful body/result boundary. Initial self residency
  requires the [current-instance formation handoff](../design/symbol-world/structural-type-formation-and-composition.md#current-instance-initial-formation-handoff),
  not a read/copy of uninitialized self or a reparented helper result.
- **Kind of ordinary lang program consumed:** Complete compile expressions
  with local declarations, branches, recursion and calls; ordinary associated
  check/admit formed through actual OpenHere and member authority.
- **Output lifecycle material:** A single direct self-root complete type and
  associated behavior. check observes a world and returns bool; admit performs
  the limited mapped update and returns unit. Packaging/arity is not frozen.
- **Commit/delivery boundary:** Caller Pre check -> body Pre admit; body Post
  check -> caller Post admit. Entry assumptions and summary import do not
  replay the internal history. Type construction authority and admit's world
  authority are separate. Joint successful publication remains mandatory.
  Component adoption uses the original boundary's scoped projection storage
  and publishes no partial delta if a later component fails.
- **Failure or unavailable result:** Failed Pre prevents action, failed required
  Post prevents successful outward delivery, and unavailable contract behavior
  stops there. A failure cannot retroactively undo earlier unrelated commits.
- **Unconnected consumer dependencies:** General clause E, compile instances'
  initialized self/state, associated injection, LC2/LC3 and ordinary result
  delivery. Current lifetime head-clause carriers supply syntax only.
- **Fidelity/composition obligations:** Four-way mapping, self-root reachability,
  world authority uniqueness, history preservation, and no duplicate execution.
  A true user check grants no additional update authority: each safe operation
  independently requires its derivable mapped source/region/Color/access/effect
  premises. External new facts require LC7 rather than ordinary derivation alone.
  Formation and updates cannot be justified by the still-unformed contract's
  own check/admit/automatic Post. See the owner's [terminating basis](../design/lifetime/lifetime-policy-and-overload-boundary.md#91-bootstrap-dependencies-must-terminate)
  and [ordinary contract/counterexample](../design/lifetime/lifetime-algebra-walkthroughs.md#ordinary-self-root-contract-program).

## LC5. Exit compression and automatic Post

- **Input material:** At contract formation, the semantically formed body
  Pattern/structure as complete ordinary material, bound roles/scopes/guarded
  alternatives and established selected-operation structural descriptions or exposed sufficient
  modular contracts. At actual exit, the reached path/activation, actual names/
  generations/cuts, committed history, available origin/None/unknown observations
  and full region/Color/path/source/effect/delivery mapping. These inputs differ;
  a future Exit is not a formation-time observation.
- **Existing action that determines it:** Ordinary semantic body/closure/Pattern
  formation supplies the guarded descriptions through LC1/LC2, independently of
  future execution. Actual block/call delivery, fallthrough or target completion
  supplies reached material, with cleanup fixed beforehand. Raw/Norm shape and
  source spelling alone supply neither semantic input.
- **Kind of ordinary lang program consumed:** At formation, supported structural
  relation composition, bound-local elimination and guarded Pattern construction
  without invoking the described actions. At exit, ordinary observation/traversal
  and actual local-name compression. Both retain every outward row and use LC4's
  ordinary self-root contract construction on their distinct typed material.
- **Output lifecycle material:** A conditional Post_summary available before
  exposing the automatically summarized callable contract for use; separately,
  Post_actual for each actual legal successful exit. The summary retains guard/
  source/effect correlation without future world updates; actual outward material
  eliminates local-name reads and is validated against the instantiated summary.
- **Commit/delivery boundary:** Summary formation is a real ordinary computation
  at the formed-body/contract boundary; it publishes no described branch effects.
  Actual compression occurs at each real exit while its observations are
  available, and successful mapped Post adopts through common joint publication.
  Target return through InPlace creates no local unit or fabricated result.
- **Failure or unavailable result:** Missing selected descriptions, sufficient
  modular guarantees, lawful structural transforms or founded recursive-summary
  results makes summary formation Unavailable; no future-path speculation or
  delayed single-path substitute completes it. Missing actual traversal/mapping
  is Unavailable; expiring-target escape or summary mismatch rejects without
  reopening. Failure/unfinished exits publish no success Post; missing origin
  establishes neither None nor a new unknown root.
- **Unconnected consumer dependencies:** LC1–LC4, semantic guarded-body material
  production, structural summary and recursive/modular family programs, actual
  return/completion/result/escape and corresponding exit programs. Both consumer
  boundaries remain pending, not independent executors or Policy stages.
- **Fidelity/composition obligations:** For each supported family and actual
  ordinary mapping theta_p, Post_actual(p,theta_p) implies Post_summary(theta_p)
  on every legal successful path. Preserve external/unknown-root identity, all
  region/Color/path/guarded-effect rows and real delivery; no validity expansion
  or history erasure. Prove adopted nested/choice/recursive combinations, not a
  universal join/implication theorem. See the [select_one witness](../design/lifetime/lifetime-algebra-walkthroughs.md#advance-summary-and-actual-exit-witness).

## LC6. Function Pattern and recursive interface consumption

- **Input material:** Determined callable's exposed full input/capture contract,
  expected Function Pattern, mapped U_default/entry assumptions, Pre/Post and
  ordinary Type/Policy evidence. RHS requests supply only explicit args/output/
  Policy; no capture/lifetime request slots are added.
- **Existing action that determines it:** Pattern owner's FnExtract/FnRequest
  and ordinary selection; projected Function Pattern erases external
  structural-source authority independently of Val1 knowledge.
- **Kind of ordinary lang program consumed:** Exposed contract behavior and
  supported compatibility computations; recursive programs use the same E
  and their established modular contracts, not hidden body restoration.
- **Output lifecycle material:** Interface compatibility evidence, then the
  actual selected call's mapped lifecycle obligations/results. Handwritten
  omitted Post means NoDelta; body omission uses LC5, not that interface default.
- **Commit/delivery boundary:** At interface extraction require Pre_F => Pre_A
  and Post_A => Post_F. Subsequent calls still check actual Pre and commit once.
- **Failure or unavailable result:** Insufficient Post rejects extraction;
  unknown implication/recurrence evidence is Unavailable. No implicit stronger
  Post, recursive default permission, runner-up or ABI change.
- **Unconnected consumer dependencies:** Directional Function Pattern source
  consumers, LC4/LC5, recursive family definitions and their actual evidence.
- **Fidelity/composition obligations:** Contract preservation after source
  authority erasure; recursive domain/operations/closure/termination or explicit
  rejection. Regular SCC finiteness proves no arbitrary recursion termination.

## LC7. Unsafe Post and trusted dynamic facilities

- **Input material:** Selected external operation, ordinary Pre, real resource/
  name/generation/range/window identities, actual validation success or failure,
  existing compatible history and mapped result names.
- **Existing action that determines it:** Real external validation, batch
  partition, alias window or ownership/accounting operation under its ordinary
  call; the materializer provides the promised external behavior.
- **Kind of ordinary lang program consumed:** Trusted library encapsulation
  using ordinary lifecycle programs and post-directed external descriptions.
  No unsafe pre and no admission before the missing Pre check.
- **Output lifecycle material:** Successful compatible external facts:
  credential source/interval/Color, actual disjoint ranges and new refs from
  original R, or valid excluded windows with fresh anonymous non-alias origins.
- **Commit/delivery boundary:** Successful common action and joint Post, then
  ordinary caller summary admit. Validation failure returns its ordinary
  declared failure Pattern without success permissions; input disposition
  follows the explicit failure contract.
- **Failure or unavailable result:** False ordinary Pre prevents action.
  Failure/unknown validation cannot publish trust, erase conflicts or cause a
  hidden check/retry. A lying external success violates its trusted description.
- **Unconnected consumer dependencies:** LC2–LC6, trusted library definitions,
  actual dynamic validation and machine realization; partbox/option protocols.
- **Fidelity/composition obligations:** Full invalidation coverage, real range
  disjointness (not path inequality), surviving dependency checks, history and
  interval monotonicity, alias exclusion/escape, same-resource conservation and
  exact consumption. One-element partition omits only the pairwise test.

## LC8. Regular access and cropping programs

- **Input material:** LC1's finite regular structure, actual identity parameters,
  operable endpoints/permissions, route prefixes, ranges and current Color.
- **Existing action that determines it:** Actual formed/selected field and
  ref/share operations, not every navigable Val2 member or `$` occurrence.
- **Kind of ordinary lang program consumed:** Choice, field advance/composition,
  repetition and access cropping over ordinary values; trusted range facts
  enter through LC7 where necessary.
- **Output lifecycle material:** Composed or cropped regular descriptors with
  actual targets and subset operation permission, preserving historical Color.
- **Commit/delivery boundary:** The selected access/refinement's ordinary
  action/observation boundary and later LC5 delivery. Cropping does not mutate
  ordinary fields or confer permission to an ancestor used only for routing.
- **Failure or unavailable result:** Unsupported operation or missing facts is
  Unavailable. No ref through share/alias, no share strengthening, no implicit
  structure recovery and no numerical theorem from finite state reuse.
- **Unconnected consumer dependencies:** LC1–LC3, LC7 for dynamic ranges,
  regular value/program definitions and actual access/escape consumers.
- **Fidelity/composition obligations:** Closed supported regular domain,
  terminating adopted finite-descriptor operations, non-expansive cropping,
  preserved name/generation/target distinction and ref/share/alias boundaries.

## Current implementation evidence

The [existing shared handoff](semantic-values-and-extraction-interfaces.md#shared-action-and-lifecycle-handoff)
and public transaction checks cover cleanup gates, supplied Pre, atomic joint
publication, Kill/Preserve and explicit finite Color observations. Production
roster discovery establishes no formation/origin fact. Test-only imports are
not semantic producers. `reify_value` still reports
`LifecycleContinuationPending` after preliminary observations; a fixed cleanup
table is insufficient to produce a complete LifetimeValue.

No LC1–LC8 source chain, automatic Post/compression, trusted API, access
algorithm or recursive contract consumer is connected by this documentation
revision. These gaps require ordinary language consumers and a small justified
bootstrap basis, not a parallel Rust interpreter, string-matching semantic
tests, default success or a general external theorem prover.

In particular the initial current-instance root source facility, its complete
resident publication, component-scoped safe world operations and the expanded
walkthrough programs are pending. Existing host callbacks/snapshots prove only
their supplied-material slice; they cannot validate a source function merely
because it is named check/admit or returns true.
