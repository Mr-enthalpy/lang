# Heap, IO and effect migration closure

Baseline: merged PR #117, commit 49d0511d865eb1d19fe1534d9ba5b148122ec162.
This is an implementation/verification index, not a second semantic authority.
The external handoff's N01–N20 and U01–U06 are distributed among current owners;
history is neither needed nor read for current interpretation.

## Replacement closure M01–M13

Original locations refer to that baseline, for review without consulting history.
Each replacement is in place; pending means an unconnected consumer, never text
or semantics scheduled for removal. All acceptance IDs below are HIE-scoped.

| ID | Original current location / affected rule | Current owner / replacement | Positive and negative acceptance | Remaining consumer |
|---|---|---|---|---|
| M01 | dependency §§3–4, README lines 141–147, AGENTS dependency paragraph | dependency §3 Required/Initialized; invocation-bound §4.1 Product and §4.2 canonical inputs | C01–C03,C05,C06 | LC9 fulfillment/ordinary realization |
| M02 | Raw §4.2/§7/§9; public capture elaboration; callable material §2 | Same positions distinguish Required versus Initialized, no synthetic bare-name initializer | C04,C01,C02,C03 | Capture syntax/carriers connected; semantic LC9 pending |
| M03 | lifecycle §2.1.2/§2.1.6 entry defaults; glossary/CSF-T11 | lifecycle §2.1.8 empty slice and new-input-only defaults, call §7.3 | L01,L02 | LC10 inherited AutoPre |
| M04 | call §6.2–6.3 Param summaries; owner graph/Self satellites | Same positions equal call/lifecycle inputs, no field ownership premise, coherent pass | C06,L05 | LC9/LC3 actual transfer |
| M05 | Pattern §11.1, CSF-T17/T21, public/guide/glossary | Pattern §11.2 boundary makes former captures explicit; optional ordinary Env forwarding | C07,C08,A06 | FnExtract/FnRequest/LC6 |
| M06 | AGENTS Done sentence; README/spine/guide completion summaries | Internal Done_chi/Done_Return remains private; effect owner's ordinary Done(v) branch | A07,E03 | Step/result Pattern consumer |
| M07 | New topic; existing E §10 ABI applies | E §7.1 whole-original-K CPS and §11 template/grain; ordinary ABI1/ABI2, explicit optional Box | A01–A04,K01–K08 | Whole-K common-E CPS consumer plus ordinary O/materializer; no dedicated async layout |
| M08 | return owner and control-flow contract barriers retained | Effect §3 propagates ordinary results; return §7 distinguishes targeted return | A05,A06,L05 | Common E/result handler |
| M09 | New topic; return-normalization sketch §12 scoped non-goals | Effect §4 throw/Wait/reentry plus ordinary Drop; sketch links owner | E01,E02 | Throw expansion/Drop source execution |
| M10 | conformance intro; policy-visibility carrier summary; extraction contract | Effect §5 selected forced final Miss only; applicability/ordinary branch miss separate | E03–E05,C05 | Selected forced-execution/handler |
| M11 | unified §3, invocation result and glossary/roadmap summaries | Formal return-only NameExpr; yield cannot expose instance-open root | S01 | Compile reach/yield consumer |
| M12 | unified §20 and every repeated reify entry summary | ClosedReturn plus ClosedInterface compile-determined public regular shape | S04,S05 | Actual interface/dependency proof consumer |
| M13 | New capability requirements, broad existing host boundary preserved | host §6 and heap owner: ordinary contexts, optional Wait protocols | H01–H09,S02,S03,S06 | Global providers/library/LC11 |

## Formal proof obligations F01–F14

These are consumer obligations over admitted domains, not unbounded compiler
proof-search requirements. Unknown evidence is unavailable, not permission.

| ID | Obligation | Canonical judgment and evidence |
|---|---|---|
| F01 | Unique declared capture resolution, no second selector or reopen | dependency §3 Bound/Mismatch/Unavailable, overload §4 seal |
| F02 | Equal self/capture/args, initialized once, no duplicate Move/Kill | call §6.2–6.3 coherent projection/transaction |
| F03 | Source erasure and explicit projected captures, optional Env authority | Pattern §11.1–11.2 FnExtract/FnRequest |
| F04 | Empty Post=Pre, inherited old names/new input defaults | lifecycle §2.1.8 Slice_epsilon and AutoPre |
| F05 | First-layer Move locality and valid atomic repair | lifecycle §2.1.3/§2.1.8 Pre_MoveRepair/Valid |
| F06 | Locate/submit conserve hierarchy and all live storage duties | heap §§1,3,4 Pre/commit/Post |
| F07 | Guaranteed submit over every admitted Pre state | heap §4 SubmitCap, feasible metadata guarantee |
| F08 | Conservative regular sources and complete local-layer elimination | heap §5 StorageSource/L_pi and LocalLayers theorem |
| F09 | Actual one-step trace, unselected branches have no actions | effect §§1–3; Pattern §11.3 regular Function Pattern |
| F10 | Legal grain equivalence preserves all observations | E §11 Legal/Generate/Obs |
| F11 | Dropped concrete continuation cleans exactly once ordinarily | effect §4 plus lifecycle fixed cleanup/destructors |
| F12 | Yield no instance NameExpr; compile-closed reify public shape | unified §§3,20 self-root and ClosedInterface |
| F13 | Host/resource protocols compose without mandatory effect or IO stack | host §6, heap §6 and independent migration |
| F14 | Complete original-K CPS preserves caller suffixes, all contexts/stages, selected origin, delivery/targets and exact cleanup | E §7.1 CPS/Plug/Compose; LC13 and K01–K08. Added whole-continuation obligation, not implemented conversion |

## Dependency order and unavailable boundaries

| Phase | Concrete work / gate | Current executable boundary |
|---|---|---|
| 1 | Capture preservation, then required call binding, Function Pattern flattening, inherited AutoPre | Required/Initialized Raw/Norm syntax and tests connected. Source captured call entries reject before installation or relation preparation with explicit unsupported capture-input diagnostic. LC9/FnExtract/LC10 execution pending. |
| 2 | Ordinary HeapStack/pool library, guaranteed submit and global providers | No connected global capability or pool protocol; ordinary source consumer reports unavailable. Rust storage carriers prove no successful allocation/submit. |
| 3 | Full original/common-K CPS conversion/composition, then Wait/resume, repair, regular Function Pattern and handler | Whole-K CPS consumer/LC13 is pending, including ordinary no-wait work and caller suffixes across all contexts/stages. Exact repair attachment reports UnsupportedEffectRepair; no high-binding @ plus closure fallback. Dedicated Wait/effect parsing and common-E steps remain pending. Ordinary @/value-side ~ remain legal. |
| 4 | Throw/forced PatternMiss, compile yield and ClosedInterface | Source execution unavailable at existing common-E/extraction frontier; no fake catchable static diagnostic or invented residual. |
| 5 | Legal grain generation, stage/heap cross-composition, ordinary ABI materialization | Pending consumers; no automatic Box/state-machine/heap-escape command. Prove Obs equivalence per adopted transform. |

The [conformance matrix](canonical-semantic-conformance.md#heap-io-and-effect-acceptance)
separates defined semantics, pending consumers and actual executable slices.
Complete each phase with source-produced positive/negative, identity, no-reopen,
non-derivability and authority-uniqueness evidence before claiming it connected.
Generic syntax/storage fixtures establish only their explicit limited slice.

The original continuation's full CPS obligation is broader than identifying
local wait cuts: pending caller contexts, inputs, let/assignment/result delivery,
guarded control/recursion, cleanup and compile/reify boundaries must compose in
one E/K. Its equations/Pre/Post are owned by E §7.1, the handoff is LC13 and all
K01–K08 acceptance rows have no executable coverage. Conversion need not be
implemented in this PR; it cannot be omitted from future async execution.

## Fresh-reader acceptance

A fresh read-only agent reads AGENTS, README, semantic spine and current owners,
without history or the external handoff, and answers: when [x] binds; InPlace
AutoPre; heap relocate; Wait result; throw cleanup; Function Pattern source
erasure; when reify needs interface closure. Any contrary inference blocks
acceptance and requires editing its decisive current premise in place.

Independent fresh-reader audit completed on 2026-11-08 without history or the
external handoff. All seven answers matched the current owners: per-new-call
required binding versus once-formed initialized sources; inherited InPlace
Post; guaranteed place-preserving pool submission; concrete Wait successor and
suspending-side atomic repair without mandatory Box; ordinary Drop for throw;
source-erasing Function Pattern with explicit former captures; and compile-closed
reify interface with runtime choice inside fixed sums. No contradictory current
premise was found. The audit identified incomplete ClosedInterface references in
Policy, Pattern adaptation and dependency summaries; these were synchronized.
This is independent documentation acceptance, not pending semantic execution.
The seven-question audit predates the explicit CPS obligation; it is not
evidence of whole-K CPS conversion coverage. The publication review additionally
checks whole-K CPS wording/correspondence and the unexecuted K01–K08 matrix.
An additional fresh read-only publication audit passed on 2026-11-08 after the
CPS clarification and review corrections. It verified E §7.1, LC13, K01–K08,
single coherent CallInputs and required specifications without early source
materialization. It read no history or external handoff and claimed no CPS
execution. Source review also caught and fixed over-broad weak-name let capture
recognition; initialized expression shorthand now has dedicated regression goldens.

Final repository validation: cargo fmt --all, cargo check --all-targets, cargo
test and git diff --check pass. Added local Markdown destinations/anchors were
checked; history is unchanged. Passing Rust coverage establishes capture syntax,
constraint/order/Hole/span preservation, pending-repair recovery and rejection
before unsupported capture preparation/installation. The remaining semantic
consumer rows above continue to be explicitly unavailable.
