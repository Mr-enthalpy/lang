# Canonical semantic conformance matrix

Status: canonical acceptance scenarios; evaluator/source consumers pending.
These are semantic counterexamples and equalities, not claims that the current
parser supports every displayed spelling or that executable tests already
cover them. The topic owners define meaning; this matrix indexes their checks.
The cases distinguish ordinary/type call entrances, immutable snapshots,
instance identity and result structure, interpretation polarity, role
registration, name realization and ordinary dependency/lifecycle checks.
Each row is an acceptance obligation, independently of executable coverage.


| Case group | Canonical owners | Implementation gate |
|---|---|---|
| S01–S12 | [Policy](../design/symbol-world/policy-and-static-flow-projection.md), [calls](../design/symbol-world/function-object-call-model.md), [selection](../design/patterns-overload/overload-resolution-design.md) | Single-stage positions; two rounds and sealed origin |
| E01–E09 | [Evaluation](../design/static-evaluation/evaluation-residual-and-optimization.md), [compile invocation](../design/static-evaluation/compile-instance-invocation-and-result-delivery.md), [build](../design/build-package/build-system-design.md) | Runtime entry, dominance and readiness |
| L01–L08, W01–W08 | [Lifecycle](../design/lifetime/lifetime-policy-and-overload-boundary.md), [mechanical placement](../design/mechanical-lowering/mechanical-argument-passing-and-move-fixed-point.md) | Instance effects and cleanup |
| LA01–LA30 | [Lifecycle value algebra](../design/lifetime/lifetime-policy-and-overload-boundary.md), [unsafe admission](../design/lifetime/unsafe-semantic-admission.md), [walkthroughs](../design/lifetime/lifetime-algebra-walkthroughs.md) | [LC1–LC8](../contracts/lifetime-program-consumers.md): ordinary programs, projection, boundary transforms, regular access and trusted facilities; source wiring pending |
| P01–P12 | [Residual/completion](../design/patterns-overload/static-pattern-spaces-and-extraction-chains.md), [target return](../design/control-flow/targeted-return-and-d-reduction.md), [compile state](../design/static-evaluation/compile-instance-invocation-and-result-delivery.md) | Restricted Split, escape and current compile facts |
| O01–O07 | [Operator families](../design/patterns-overload/operator-patterns-and-generative-declarations.md) | Operator roles and OG_s |
| N01–N08 | [Names](../design/symbol-world/names-and-overload-groups.md), [construction](../design/symbol-world/structural-type-formation-and-composition.md), [source composition](../design/symbol-world/symbol-construction-units-and-namespace-origin.md) | Expression formation and conservative contributions |

Every row currently requires consumer alignment; see the
[implementation evidence and gates](roadmap.md#8-acceptance-evidence).
Single-atom Policy carriers, omitted-P1 completion and horizon facet visibility
have substrate coverage after Stage cleanup. Name resolution fixes identity
before exposure; hidden value and Pattern facets retain their resolved material
while read/enumeration consumers remain unavailable at that horizon. Different
hidden identities remain distinct, and hiding never becomes absence. Explicit
Pin stage atoms are canonical but currently report the unconnected
InputAdmissible consumer before selection; they are not rejected as illegal
overrides or silently replaced by inherited P2. Name resolution proceeds
without a horizon. Subsequent callee observation, candidate preparation and
body-entry observation use one invocation horizon, retaining distinct facts;
the horizon alone proves neither readiness nor execution legality. Ordinary
initializer bindings require the common continuation preservation consumer
when evaluation cannot complete; until it is connected they fail before
installation, without deriving runtime from residual or pending continuation work.
Exact empty V_tau, visible non-callable values and fully observed result-Policy
mismatches have terminal no-candidate coverage at both initializer boundaries.
Actual hidden observations are reported by invocation itself, never inferred
from generic failure or an absent diagnostic. Mixed body observations retain
ordinary visible-candidate ambiguity and terminal selected failure.
Resolved argument bindings whose Val1 is hidden stop before applicability,
preference and selection at the common-continuation frontier. They never fall
back to unknown-expression default mode, even with a readable Pattern or a supplied
argument mode. Readable binding observations retain their mode; genuinely
unknown expressions remain a separate case. Stage and horizon carriers expose
no enum-order preference, and facet observation defines no alternate resolver.
Candidate errors and selected-body failures remain diagnostics, with no second
compile evaluator mode or function/world stage partition. Ordinary
deduction, R_vis/C_sigma and E readiness/saturation still require consumer
coverage. Carrier test success is not general source coverage of these semantics.
Construction-window checks consume explicit WindowLive and authority facts;
generation-coordinate split/merge and owner-interval termination remain
unconnected consumers. Neither a global runtime epoch nor a compile-branch
no-op supplies those facts. Allocation without established construction facts
fails OpenHere, and saved authority cannot revive an explicitly closed window.

## Stage, Policy and two-round calls

| ID | Scenario | Required result |
|---|---|---|
| S01 | P2=runtime with omitted P1 stage | Complete to runtime, without a runtime/compile union. |
| S02 | P2=compile with omitted P1 stage | Complete to compile; OpenPolicy and mode remain independent. |
| S03 | Bare let in a compile binding context | Complete the stage from context; leave no hole for a later runtime consumer. |
| S04 | A runtime callable has an explicitly compile Pin | Apply ordinary input admission; the whole call need not be pure. |
| S05 | An explicit formal-stage hole | Extract from actual/position rules; distinguish it from omission. |
| S06 | A runtime-policy value happens to be known | E does not promote it to compile. |
| S07 | Closed reify body needs unavailable runtime Val1 | Only its actual reverse-dependent continuation resumes with runtime inputs. |
| S08 | A static type has no runtime realization | Target demand does not manufacture a runtime slice. |
| S09 | Round one retains two stage derivations | Do not run both observable bodies; round two selects or reports ambiguity. |
| S10 | A selected compile realization has runtime residue | Preserve the selected origin; runtime does not reselect. |
| S11 | Pre or coherence fails after migration selection | Terminal failure, without a runner-up or chained repair. |
| S12 | An external demand constrains Pout stage | The consumer cannot overwrite established P1/Pout producer facts. |

## Main, reification and current evaluation

| ID | Scenario | Required result |
|---|---|---|
| E01 | Program entry at main | P2 is runtime; stable root identity creates no ambient active body evaluation. |
| E02 | Selected compile invocation | Form a stable instance before entry; Ready and legality remain separate. |
| E03 | Compile helper/cache reuse | Retain dependency validity and current storage; cache grants no execution authority. |
| E04 | Invocation returns to main | Restore enclosing continuation; self-name resident reads do not enter an active body. |
| E05 | Read a completed compile payload | Ordinary resident observation does not EnterBody. |
| E06 | callable P2 reify with closed contract | Callable P2 annotation materializes only actual reverse-dependent E work; unsupported producer is unavailable. |
| E07 | Pending action depends on a write | Preserve actual dependency/effect ordering on the current continuation. |
| E08 | compile:compile demand for runtime:compile | Reverse migration requires a selected same-Type candidate and active closed reify P2 boundary; no backflow to a past world. |
| E09 | Compilation cache reuse | Neither reopen a closed subject nor retain permanent authority. |

## Instance lifecycle and with

Substrate evidence for these obligations is in `lifecycle.rs` and
`semantic_continuation.rs` unit tests: fixed Kill/Preserve, independent supplied
legality facts, joint Pre/commit/Post, proof identity and stale rejection,
precedence/linearization of fixed cleanup points, and no duplicate Drop after Kill.
Additional substrate regressions reject crossing outstanding cleanup boundaries
without advancing K, distinguish discovery from formation/origin facts, and
keep full ReifyLife unavailable instead of returning an incomplete Region.
Prefix regressions cover future cleanup of Kill destinations and late admitted
formation, immutable fixed prefixes, stale evidence after prefix/suffix changes,
same-cut Use/Preserve rejection, and fixed-order boundary Kill. Scalar half-open
Regions also reject later endings at an existing Use/Preserve cut.
Color admission regressions reject foreign K (including colliding numeric
LifeNames), unallocated/pending/closed subjects and stale frontiers without
mutation; ended-generation Color cannot invalidate Kill inheritance or pollute
a later allocation. Active admission remains monotone and invalidates old Pre.
The production world retains one K and lifecycle name map; source lifecycle
actions, complete generation-continuation observation, NLL/with cleanup-point
derivation and complete E remain consumer gates.
These tests do not claim end-to-end conformance of the source scenarios.

Future E integration must additionally pass these regressions; current
substrate tests do not establish either gate:

- With K at 4 and an outstanding fixed Drop at 9, an action at 10 without a
  lifecycle projection cannot commit or publish state. After the Drop is
  committed or its obligation discharged, the same boundary no longer blocks
  advancement. Enforcement must come from the shared scheduler or common
  transaction, not optional inclusion of lifecycle Pre.
- A formation/origin/Color producer and its lifecycle Post share one committed
  action identity/cut and one scratch publication. Failure of any affected Pre
  or Post leaves producer facts, projection state and K unchanged. Successful
  publication includes the producer-established facts; calling a trusted import
  separately after common publication is not conformance evidence.

| ID | Scenario | Required result |
|---|---|---|
| L01 | A killing move transfers a compile-local type instance | End the source generation at that cut. |
| L02 | A global type resident and an equal local copy | Value equality does not share Killable or lifetime. |
| L03 | A legal preserving move observes a stable compile instance | Do not kill the compile root; this does not make all global resources copyable. |
| L04 | Conflicting active borrow at a move frontier | Movable/Pre may fail; do not change the predetermined MoveEffect. |
| L05 | Copy uses a clone with observable postconditions | Do not force a preserving move to use that clone. |
| W01 | x with {a}, followed by more uses of x | Account for those points when placing a's destruction. |
| W02 | x with {a}, followed by more uses of a | Do not extend x in the reverse direction. |
| W03 | y with {x} and x with {a} | Existing destruction obligations order y before x before a. |
| W04 | A killing move consumes x at k | k is a touch of the listed dependencies; insert no later Destroy(x). |
| W05 | A move preserves its source x | Continue accounting for later actual uses of x. |
| W06 | An explicit empty with clause | Anchor lexical cleanup; do not assert absence of dependencies. |
| W07 | Exit lexical scope after explicit consumption | Do not drop the consumed generation again. |
| W08 | With constraints form an unsatisfiable strict cycle | Report failure rather than arbitrarily choosing an order. |
| L06 | @ observes a compile value or a temporary without a Place | Use the same LifeName/K relation, not another compile lifetime. |
| L07 | No source @ appears | Lifecycle Pre/Post still applies. |
| L08 | Lifecycle Pre fails | Do not mutate facts or move cleanup backward. |

## Patterns, completion and compile queries

| ID | Scenario | Required result |
|---|---|---|
| P01 | An else residual remains after an if arm | It is legal inside the chain and may reach the following else arm. |
| P02 | Else tries to escape a forbidden boundary | The fixed consumer rejects using ordinary compile facts. |
| P03 | An ordinary Pattern residual may escape | Retain its input material without implicit discard. |
| P04 | An arm has completed | Later sibling arms do not match its completed result again. |
| P05 | A user declares a type named Done | It remains an ordinary type, distinct from internal Done. |
| P06 | Nested chains complete | Consume each marker without observable Done nesting. |
| P07 | A selected extractor body fails | Do not treat failure as a sum miss or try another arm. |
| P08 | An expected result Pattern exists | Deliver through ordinary R_Gamma, without a private ControlResult. |
| P09 | A default compile trait instance is mutated while OpenHere holds | Write the actual instance; later queries read current committed state. |
| P10 | Only an outer snapshot copy of the compile result is changed | Do not mutate the retained compile instance. |
| P11 | Trait state changes later | Do not rewrite an already formed Pattern or selected invocation. |
| P12 | A Pattern requires unordered interpretation | E uses that relation from the start; O cannot decide it later. |

## Operators, bindings and contributions

| ID | Scenario | Required result |
|---|---|---|
| O01 | Bare a+b | Use the current operator environment without hardwired bypass. |
| O02 | The + argument in operator[+] | Read the ordinary operator-name value without recursive dispatch. |
| O03 | g:OG_"+" is bound under arbitrary name a | The ordinary operator-family Pattern can still extract "+". |
| O04 | a carries another library's OG_"+" | Select the current operator environment's "+" slot, not necessarily a's candidates. |
| O05 | Convert OG_s to ordinary OverloadGroup | Use ordinary explicit conversion; no subtype or implicit reverse recovery. |
| O06 | An invalid new token string attempts operator-family construction | Do not extend lexer/parser grammar retroactively. |
| O07 | Dot .op or an explicit path | Retain its own entrance without unrelated forwarding. |
| N01 | A closure expression at any level | Legal completion produces full tau_C; distinguish file installation and lexical binding. See 106-CL01 and 106-NS01–04. |
| N02 | A legal same-name closure contribution is added later | Do not change the earlier RHS evaluation category retroactively. |
| N03 | Only let a=uint8 | The RHS is exactly tau_uint8; file installation and local binding neither wrap nor arbitrarily merge it. See 106-NS03–04. |
| N04 | Legal inner lexical shadowing | Do not reinterpret it as contribution to the outer same-name object. |
| N05 | OpenHere/Pre fails for a named contribution | Ergonomic repair cannot bypass the failure. |
| N06 | Binding and contribution effects address the same coordinate | Use established role and conflict rules; spelling alone cannot mix them. |
| N07 | Independent legal closure contributions in sibling files | Join through ordinary unordered contribution at one NameCoord; no first-file winner. |
| N08 | A sibling reads a value newly written by another sibling | File sorting cannot create otherwise absent dependency visibility. |


## Consumer acceptance cases

### 106.1 Product layer ordering


| ID | Scenario | Required result | Canonical owner | Consumer status |
|---|---|---|---|---|
| 106-PD01 | All direct entries are named, without a top Pattern name | The layer is unordered; no added top name or wildcard is needed. | [PD owner](../design/patterns-overload/pattern-relations-observation-and-extraction.md) | Defined semantics; source/evaluator consumer pending |
| 106-PD02 | One bare entry is added to that layer | The whole layer becomes ordered; named entries do not form a local unordered region. | [PD owner](../design/patterns-overload/pattern-relations-observation-and-extraction.md) | Defined semantics; source/evaluator consumer pending |
| 106-PD03 | An all-named child layer inside an ordered parent | The child decides independently and is not forced into the parent's order. | [PD owner](../design/patterns-overload/pattern-relations-observation-and-extraction.md) | Defined semantics; source/evaluator consumer pending |
| 106-PD04 | Local variables a and b appear as bare Product values | Their variable spellings do not become structural names. | [PD owner](../design/patterns-overload/pattern-relations-observation-and-extraction.md) | Defined semantics; source/evaluator consumer pending |
| 106-PD05 | An unordered structure must become a bare sequence | Extract by name, then explicitly assemble in the requested order; no default sorting. | [PD owner](../design/patterns-overload/pattern-relations-observation-and-extraction.md) | Defined semantics; source/evaluator consumer pending |
| 106-PD06 | Explicit ? removes the top name | An all-named layer stays unordered; preserve real nesting. | [PD owner](../design/patterns-overload/pattern-relations-observation-and-extraction.md) | Defined semantics; source/evaluator consumer pending |
| 106-PD07 | Unordered result structure formed through observable effects | Result unorderedness alone does not authorize effect reordering. | [PD owner](../design/patterns-overload/pattern-relations-observation-and-extraction.md) | Defined semantics; source/evaluator consumer pending |
| 106-PD08 | A future ordinal operation observes bare entries | Ordinals do not create names; the complete public ordinal API remains open. | [PD owner](../design/patterns-overload/pattern-relations-observation-and-extraction.md) | Defined semantics; source/evaluator consumer pending |

### Product repetition and unpacking acceptance targets

The [Product owner](../design/patterns-overload/product-repetition-and-unpacking.md)
defines these semantic targets. Syntax preservation, existing material
normalization and explicit unavailable boundaries are separate evidence from
complete source formation/unpacking. No unconnected target is reported as an
executed success.

| ID | Scenario | Required result | Consumer status |
|---|---|---|---|
| PR01 | Standard product packaging and its classifier | `product = (_ * ω)$`; classifier Pattern `_ * ω` uses ordinary repetition. | Norm operator material preserved/unsupported; wrapper formation pending |
| PR02 | P/E/G source continuation with a bare Product | Grammar merges only the first following Product; established call input recursively exposes/merges available standard packaging. | Existing source association and product_shape tests connected |
| PR03 | (a,(b,c)) and ((a,b),c) at call input, expression barriers, unit slots | Norm preserves source nesting; call material exposes a,b,c. Keep units and expression barriers; unavailable runtime shape and non-standard layers need their own evidence. | Norm goldens and product_shape connected; arbitrary packed-result exposure pending |
| PR04 | S `_`, S `_$` and S `(_)$` | Anonymous deduction/matching differs from external universerank extraction; the latter never becomes a wildcard. | Polarity carrier and unavailable relation boundary connected; external extraction pending |
| PR05 | S `bool::`, `(if)$` or `_$` in a partial Sequence | Require the actual external Pattern; other matching terms cannot erase it or establish applicability. | Explicit unavailable boundary regression connected |
| PR06 | Singleton contextual form `_$ =_context (_ * 1)$` | Preserve polarity and group interpretation; derive contextual expansion without equating arbitrary wrapper Object identities. | Syntax coverage connected; singleton semantic consumer pending |
| PR07 | Different compile element material, same `t * n` Pattern | Val1 participates in complete identity, so different normalized payloads remain distinct. | General Object identity substrate connected; repetition formation pending |
| PR08 | Legal replacement inside a `t * n` wrapper | Val1 and complete normal form may change while Pattern stays fixed; require ordinary write/Pre checks. | Repetition mutation consumer pending |
| PR09 | Compile `t * ω` with an available finite payload, then `pack?` | Selected extraction exposes exactly that finite bare Product. | Ordinary unpack consumer pending |
| PR10 | Runtime-only `t * ω`, then `pack?` | No static variable-arity expansion; require an ordinary iteration-like consumer. | No runtime expansion success path; iteration/unpack consumers pending |
| PR11 | Unordered packed layer exposed through `?` | One-layer named peel preserves its body and ordering; subsequent Product-context exposure uses its own law. Positional sequence needs named extraction and ordered assembly. | General ordering substrate connected; packed extraction pending |
| PR12 | Runtime data requests a compile view for expansion | Require the admitted reify/reverse-migration boundary; neither `?` nor Product context grants authority. | Missing-authority rejection connected; reverse execution pending |
| PR13 | `t * n` versus `T * D` | Ordinary operator applicability distinguishes repetition cardinality and type composition; spelling grants no private bypass. | Unresolved syntax connected; both ordinary semantic families pending |

### 106.2 Path structure and reading


| ID | Scenario | Required result | Canonical owner | Consumer status |
|---|---|---|---|---|
| 106-PT01 | bool::a::path and bool:: have equal second-level resident observations | Eval_value may agree while Read_name and # differ; no bare Path equality is implied. | [PT owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT02 | Project a pure NameValue with no existing external target | Obtain legal structure without requiring final-target lookup. | [PT owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT03 | Equivalent internal Path constructions | Path projection observes normalized structure, not whitespace, parentheses or source positions. | [PT owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT04 | Three legal internally composed Path segments | Preserve associativity and the language's name::path direction. | [PT owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT05 | The endpoints of ::a and a:: | Retain the difference rather than erasing both to an undirected sequence. | [PT owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT06 | Ordinary extraction and validation of path_pattern | Expose nodes, links and endpoints. PathShaped rejects empty standalone/cyclic chains, multiple or nonterminal explicit roots, non-string names and incompatible endpoints; representation alone is insufficient. | [PT owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT07 | Construct Path material from a string | Create Names([s]) with RelativeSingleName=(Select,OpenRoot), without lookup or access authority. | [PT owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT08 | A string contains :: or parentheses | Do not automatically re-lex or parse it as more paths or arbitrary source. | [PT owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT09 | Project and reinject into a Path consumer | Recover equivalent internal structure within the legal domain. | [PT owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT10 | An inner binding shadows the textual root of a pure Path | Resolve at that external read and use the applicable inner root. | [PT owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT11 | A Path explicitly retains an ordinary value root | Preserve that material under ordinary value rules; a same-spelled name does not replace it. | [PT owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT12 | A Path explicitly retains a reference root | Preserve target/generation without lifetime extension or automatic retargeting. | [PT owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT13 | A Path is sealed into a selected invocation's runtime residue | Runtime does not resolve by spelling or select again. | [PT owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT14 | The target is invisible, unrealized or illegal | Structure formation grants no read authority; ordinary Read fails without reopening. | [PT owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |

### 106.3 Interpretation polarity and strong structural contexts


| ID | Scenario | Required result | Canonical owner | Consumer status |
|---|---|---|---|---|
| 106-FL01 | Reuse an obtained Policy p with <> p$ let | Use p's value, not a new hole or spelling-only interpretation. | [polarity owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-FL02 | runtime let | A concrete constraint with no new deduction binder, not an implicit <runtime>. | [polarity owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-FL03 | Explicit <p> p versus omitted Policy | Keep HoleBinderId, omission and concrete material distinct. | [polarity owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-FL04 | Opposite-context material containing existing HoleRefs | Preserve actual PatternRoot/HoleBinderId rather than rebinding by spelling. | [polarity owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-FL05 | Opposite-context material with invalid scope or structure | Ordinary inapplicability/error; no alpha-renaming or alternate-interpretation fallback. | [polarity owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-FL06 | A polarity-flip operand has observable evaluation | Evaluate the reached occurrence ordinarily, without repetition for projections. | [polarity owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-FL07 | Required opposite-context material is runtime-only or not ready | Do not fabricate static Pattern material; retain continuation and legality boundaries. | [polarity owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-FL08 | General interpretation polarity is defined | The polarity flip establishes neither arbitrary source quotation nor # outside the domain of an applicable path_pattern projection. | [polarity owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |

### 106.4 Open Products and name labels


| ID | Scenario | Required result | Canonical owner | Consumer status |
|---|---|---|---|---|
| 106-NM01 | ::host exposes several named actual entries | Retain each entry's name Pattern rather than returning a bare value list. | [NM owner](../design/symbol-world/names-and-overload-groups.md) | Defined semantics; source/evaluator consumer pending |
| 106-NM02 | Surface notation for a labelled entry | Do not rewrite (val (s name)) into a comma tuple of two bare values. | [NM owner](../design/symbol-world/names-and-overload-groups.md) | Defined semantics; source/evaluator consumer pending |
| 106-NM03 | Local let a=bool:: followed by ::a | Observe bool's internal layer without wrapping the whole value in an a layer. | [NM owner](../design/symbol-world/names-and-overload-groups.md) | Defined semantics; source/evaluator consumer pending |
| 106-NM04 | Rebind the local value with b=a | Changing the holder's binder does not rename internal members. | [NM owner](../design/symbol-world/names-and-overload-groups.md) | Defined semantics; source/evaluator consumer pending |
| 106-NM05 | Observe an outer layer actually containing a and b | Label those entries a and b, separately from their internal layers. | [NM owner](../design/symbol-world/names-and-overload-groups.md) | Defined semantics; source/evaluator consumer pending |
| 106-NM06 | Extract using a concrete or hole name Pattern | Obtain the corresponding entry and name parameter; the value binder may have any spelling. | [NM owner](../design/symbol-world/names-and-overload-groups.md) | Defined semantics; source/evaluator consumer pending |
| 106-NM07 | Same-spelled names under different roots | Equal label types do not merge NameCoord or Place. | [NM owner](../design/symbol-world/names-and-overload-groups.md) | Defined semantics; source/evaluator consumer pending |
| 106-NM08 | Same-name closure contributions | Share a bucket; actual merging still requires the contribution relation. | [NM owner](../design/symbol-world/names-and-overload-groups.md) | Defined semantics; source/evaluator consumer pending |
| 106-NM09 | Conflicting ordinary same-name values | Labels grant neither automatic TypeAdd nor overwrite permission. | [NM owner](../design/symbol-world/names-and-overload-groups.md) | Defined semantics; source/evaluator consumer pending |
| 106-NM10 | A generator can answer infinitely many potential names | ::host returns its finite current observation, not an enumeration of all requests. | [NM owner](../design/symbol-world/names-and-overload-groups.md) | Defined semantics; source/evaluator consumer pending |

### 106.5 File installation and local binding


| ID | Scenario | Required result | Canonical owner | Consumer status |
|---|---|---|---|---|
| 106-NS01 | Sibling files declare package members | Normalize into ordinary installation under the package root rather than leaving an empty package. | [NS owner](../design/symbol-world/symbol-construction-units-and-namespace-origin.md) | Defined semantics; source/evaluator consumer pending |
| 106-NS02 | A true local block inside a file contains let | Retain lexical binding without automatic package-member promotion. | [NS owner](../design/symbol-world/symbol-construction-units-and-namespace-origin.md) | Defined semantics; source/evaluator consumer pending |
| 106-NS03 | File-level let a=uint8 | Install the RHS value without wrapping it in a new type. | [NS owner](../design/symbol-world/symbol-construction-units-and-namespace-origin.md) | Defined semantics; source/evaluator consumer pending |
| 106-NS04 | Local let a=uint8 | Create a new binding/Place without changing the RHS internal structure. | [NS owner](../design/symbol-world/symbol-construction-units-and-namespace-origin.md) | Defined semantics; source/evaluator consumer pending |
| 106-NS05 | Install an ordinary Val2 value | Do not infer Pattern or V_tau registration. | [NS owner](../design/symbol-world/symbol-construction-units-and-namespace-origin.md) | Defined semantics; source/evaluator consumer pending |
| 106-NS06 | Direct structural formation and ordinary composition have equal final structure | Observe equality under all required premises; retain no irrelevant formation history. | [NS owner](../design/symbol-world/symbol-construction-units-and-namespace-origin.md) | Defined semantics; source/evaluator consumer pending |
| 106-NS07 | Reads or effects already occurred during construction | Final structural equality cannot erase them. | [NS owner](../design/symbol-world/symbol-construction-units-and-namespace-origin.md) | Defined semantics; source/evaluator consumer pending |
| 106-NS08 | Sibling files join from a common snapshot | Merge legal contributions and reject conflicts; no first-file winner. | [NS owner](../design/symbol-world/symbol-construction-units-and-namespace-origin.md) | Defined semantics; source/evaluator consumer pending |

### 106.6 Generated names and intermediate extraction


| ID | Scenario | Required result | Canonical owner | Consumer status |
|---|---|---|---|---|
| 106-GN01 | let _ => E without an explicit callable head | A legal generative head; invent neither a wildcard actual nor an implicit empty Product. | [GN owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md) | Defined semantics; source/evaluator consumer pending |
| 106-GN02 | let <a> a => E receives a field request | a retains the full requested NameValue (field::adl); selector constraints remain separate and binder spelling a adds no segment. | [GN owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md) | Defined semantics; source/evaluator consumer pending |
| 106-GN03 | Concrete and general generators both apply | Use ordinary specificity, without a separate generator priority. | [GN owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md) | Defined semantics; source/evaluator consumer pending |
| 106-GN04 | Incomparable maxima or failure after selection | Ordinary ambiguity/terminal failure; do not reopen generation. | [GN owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md) | Defined semantics; source/evaluator consumer pending |
| 106-GN05 | Expression body versus an ordinary enclosing body | Use the same result delivery without inserting a semantic temporary for surface differences. | [GN owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md) | Defined semantics; source/evaluator consumer pending |
| 106-GN06 | Value/call and Pattern/name forms of one compile declaration | Preserve the established equivalence domain without extending it to arbitrary lexical let. | [GN owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md) | Defined semantics; source/evaluator consumer pending |
| 106-GN07 | let a=e and let _=e | Remain extraction/binding of an existing RHS, not generation. | [GN owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md) | Defined semantics; source/evaluator consumer pending |
| 106-GN08 | let <a> (c Pattern) a | Reach a's layer, then apply the same R_Gamma with compatible valuations. | [GN owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md) | Defined semantics; source/evaluator consumer pending |
| 106-GN09 | Intermediate extraction in an unordered layer | At most one whole intermediate extraction, which may contain several named fields. | [GN owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md) | Defined semantics; source/evaluator consumer pending |
| 106-GN10 | Multiple intermediate extractions in an ordered layer | Align in that layer's order without relaxing Pack/remainder multiplicity. | [GN owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md) | Defined semantics; source/evaluator consumer pending |

### 106.7 Policy observations and direct return


| ID | Scenario | Required result | Canonical owner | Consumer status |
|---|---|---|---|---|
| 106-RP01 | Terminal G(...) supplies F's result | F's ReturnPattern/Pout is G's immediate demand before maxima. | [RP owner](../design/symbol-world/policy-and-static-flow-projection.md) | Defined semantics; source/evaluator consumer pending |
| 106-RP02 | Implementation uses SSA temporaries or registers | Add no language binding, Policy default or observable lifecycle. | [RP owner](../design/symbol-world/policy-and-static-flow-projection.md) | Defined semantics; source/evaluator consumer pending |
| 106-RP03 | User explicitly writes let temp=G(...); temp; | Retain the real binding boundary; equivalence with direct forwarding is not required. | [RP owner](../design/symbol-world/policy-and-static-flow-projection.md) | Defined semantics; source/evaluator consumer pending |
| 106-RP04 | Transparent F forwards to G | P1_F and P2_F jointly constrain G, without linear one-way propagation. | [RP owner](../design/symbol-world/policy-and-static-flow-projection.md) | Defined semantics; source/evaluator consumer pending |
| 106-RP05 | Interpret the body after selecting F | F's established signature and valuation are known immediate context for G. | [RP owner](../design/symbol-world/policy-and-static-flow-projection.md) | Defined semantics; source/evaluator consumer pending |
| 106-RP06 | Terminal H(G(...)) | Result demand first constrains H; do not inject unselected H formals into G. | [RP owner](../design/symbol-world/policy-and-static-flow-projection.md) | Defined semantics; source/evaluator consumer pending |
| 106-RP07 | Pout stage and explicit Pin holes | P1 still determines Pout stage; generic Pin does not change producer stage. | [RP owner](../design/symbol-world/policy-and-static-flow-projection.md) | Defined semantics; source/evaluator consumer pending |
| 106-RP08 | Independent value/type Policy observations | Preserve both internal observations through their explicit ordinary projections. | [RP owner](../design/symbol-world/policy-and-static-flow-projection.md) | Defined semantics; source/evaluator consumer pending |
| 106-RP09 | Direct policy(x) and Policy after direct get_type | Observe value/type facts on the same source evaluation edge. | [RP owner](../design/symbol-world/policy-and-static-flow-projection.md) | Defined semantics; source/evaluator consumer pending |
| 106-RP10 | Bind t=get_type(x), then observe t's Policy | Do not unconditionally recover x's Pp; t may have a new destination view. | [RP owner](../design/symbol-world/policy-and-static-flow-projection.md) | Defined semantics; source/evaluator consumer pending |
| 106-RP11 | Constrain both projections | Use one invocation applicability relation, without a public pair or global propagation pass. | [RP owner](../design/symbol-world/policy-and-static-flow-projection.md) | Defined semantics; source/evaluator consumer pending |
| 106-RP12 | A migration target exposes only one Policy atom | Still check full endpoints, presence and capabilities; preserve existing-first and no-reopen. | [RP owner](../design/symbol-world/policy-and-static-flow-projection.md) | Defined semantics; source/evaluator consumer pending |

### 106.8 Ordinary ADL forwarding


| ID | Scenario | Required result | Canonical owner | Consumer status |
|---|---|---|---|---|
| 106-AD01 | Default .field entrance | Lower to field::adl; the normalizer does not privately create the forwarding body. | [AD owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md) | Defined semantics; source/evaluator consumer pending |
| 106-AD02 | Project a full requested NameValue and index its first segment | a# preserves field::adl; (a#)[0] is relative field:: path_pattern, so ((a#)[0])$::t forms field::t without string truncation. | [AD owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md) | Defined semantics; source/evaluator consumer pending |
| 106-AD03 | Request a new field after closing adl rules | Realize an ordinary Val2 occurrence from frozen rules, without new structural registration. | [AD owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md) | Defined semantics; source/evaluator consumer pending |
| 106-AD04 | The forwarded type t | An ADL request neither injects field into t nor grants new OpenHere. | [AD owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md) | Defined semantics; source/evaluator consumer pending |
| 106-AD05 | The forwarder body's object parameter | Remain explicit; do not replace the callee's own self. | [AD owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md) | Defined semantics; source/evaluator consumer pending |
| 106-AD06 | Ordinary mode/stage forwarding combinations | Use legal holes/inheritance and direct return, without enumerating the Cartesian product. | [AD owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md) | Defined semantics; source/evaluator consumer pending |
| 106-AD07 | User-defined ordinary field::adl | Do not redefine real structural extraction or bypass StructuralDefault. | [AD owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md) | Defined semantics; source/evaluator consumer pending |
| 106-AD08 | OG_s and ordinary Path coexist | Preserve spelling and explicit Forget; group, name label and Path remain distinct. | [AD owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md) | Defined semantics; source/evaluator consumer pending |

### 106.9 General dependencies and explicit clauses


| ID | Scenario | Required result | Canonical owner | Consumer status |
|---|---|---|---|---|
| 106-DP01 | An ordinary external function/type/value is required | Describe general Needs without presupposing a capture field. | [DP owner](../design/symbol-world/dependency-observation-and-realization.md) | Defined semantics; source/evaluator consumer pending |
| 106-DP02 | Explicit [x] shorthand | Use ordinary let formation; infer no const, borrow or write authority. | [DP owner](../design/symbol-world/dependency-observation-and-realization.md) | Defined semantics; source/evaluator consumer pending |
| 106-DP03 | Several dependency initializers | Common pre-capture name scope does not make effects simultaneous or unordered. | [DP owner](../design/symbol-world/dependency-observation-and-realization.md) | Defined semantics; source/evaluator consumer pending |
| 106-DP04 | Formation expressions have observable effects | Execute at each reached formation occurrence, not again at body invocation. | [DP owner](../design/symbol-world/dependency-observation-and-realization.md) | Defined semantics; source/evaluator consumer pending |
| 106-DP05 | Snapshot retention versus live-position retention for the same source occurrence | The selected ordinary action fixes realization uniquely up to observational equivalence. A plain value read cannot become a live reference by lowering. Distinct candidates use ordinary preference or ambiguity; selected failure does not reopen. Equivalent layouts preserve observations. | [DP owner](../design/symbol-world/dependency-observation-and-realization.md) | Defined semantics; source/evaluator consumer pending |
| 106-DP06 | A runtime value's type information is compile-observable | Preserve information and source identity without recapture or fabricated runtime Val1. | [DP owner](../design/symbol-world/dependency-observation-and-realization.md) | Defined semantics; source/evaluator consumer pending |
| 106-DP07 | Formation contains runtime work that is not ready | Retain the formation residue; do not manufacture a complete known dependency value. | [DP owner](../design/symbol-world/dependency-observation-and-realization.md) | Defined semantics; source/evaluator consumer pending |
| 106-DP08 | A pure Path name node or ordinary string | Do not treat it as a resolved capture before external Read. | [DP owner](../design/symbol-world/dependency-observation-and-realization.md) | Defined semantics; source/evaluator consumer pending |
| 106-DP09 | A realization retains owned values or references | Validate ordinary structure/referent identity; do not hide state in an extra-semantic side table. | [DP owner](../design/symbol-world/dependency-observation-and-realization.md) | Defined semantics; source/evaluator consumer pending |
| 106-DP10 | Copy or reanchor a formed callable | Preserve dependencies without rerunning surrounding code, lookup or lifetime extension. | [DP owner](../design/symbol-world/dependency-observation-and-realization.md) | Defined semantics; source/evaluator consumer pending |
| 106-DP11 | Ordinary => closure in Automatic value mode, without an explicit capture clause, legally observes external value x | Form the automatic value occurrence using the selected ordinary action; Automatic does not imply InPlace. | [DP owner](../design/symbol-world/dependency-observation-and-realization.md#4-explicit-and-automatic-dependency-formation) | Defined semantics; source/evaluator consumer pending |
| 106-DP12 | Ordinary closure has explicit capture x and another free external value y | Diagnose the omitted value dependency; do not append automatic y. A read resolved to capture x uses that binder. Independent automatic compile structural access remains legal; occurrence identities and ordinary post-formation rules remain. | [DP owner](../design/symbol-world/dependency-observation-and-realization.md#4-explicit-and-automatic-dependency-formation) | Defined semantics; source/evaluator consumer pending |

### 106.10 Uniform closure formation and automatic dependencies


| ID | Scenario | Required result | Canonical owner | Consumer status |
|---|---|---|---|---|
| 106-CL01 | Legal completion of a closure expression at any level | Return full tau, without a local-object/file-type split. | [CL owner](../design/symbol-world/function-object-call-model.md) | Defined semantics; source/evaluator consumer pending |
| 106-CL02 | Atomic complete closure-type formation | Head, body and dependencies form one complete structural type under the common transaction. | [CL owner](../design/symbol-world/function-object-call-model.md) | Defined semantics; source/evaluator consumer pending |
| 106-CL03 | The callable implementation endpoint | Stop at an established leaf, without recursively expanding the same closure expression. | [CL owner](../design/symbol-world/function-object-call-model.md) | Defined semantics; source/evaluator consumer pending |
| 106-CL04 | tau_C, c_C, A_C and Impl_C | c_C belongs to V_tau_C; A_C=Type(c_C); AssociatedNamespace(A_C)=MemberScope(Core(A_C)), whose Val2[()] supplies Impl_C. AssociatedName is NameCoord at that root, not /tau(A_C); Core equality does not merge actual Places. | [CL owner](../design/symbol-world/function-object-call-model.md), [type owner](../design/symbol-world/type-values-places-and-borrow-views.md#associated-namespace-is-the-core-member-scope) | Defined semantics; source/evaluator consumer pending |
| 106-CL05 | Pure Q with no self-construction; well-formed tau with nonempty V_tau | Q has TypeRole and tau is a complete type. Applicable V_tau implementation entries can make it callable without making it SelfConstructible. Payload-bearing x remains an ordinary value. Empty V_tau likewise does not remove type identity. | [type owner](../design/symbol-world/type-values-places-and-borrow-views.md), [Pattern owner](../design/patterns-overload/pattern-relations-observation-and-extraction.md#13-structural-role-registration-and-ordinary-callables) | Defined semantics; source/evaluator consumer pending |
| 106-CL06 | Type tau has several c, each with several associated implementations | Project every (c,Impl) entry into one family. Compute applicability and maxima once across the union, report ties as ordinary ambiguity, and seal the winning pair/projection/frame. Actual self is c, not tau; selected failure and runtime residue cannot reselect either component. | [CL owner](../design/symbol-world/function-object-call-model.md) | Defined semantics; source/evaluator consumer pending |
| 106-CL07 | An ordinary x:T has applicable associated Val2[()] but empty V_T | Call x through the associated entry with self=x; V_T membership is unnecessary. | [CL owner](../design/symbol-world/function-object-call-model.md) | Defined semantics; source/evaluator consumer pending |
| 106-CL08 | The first callable material | Form it in the same structural operation; require no prior arbitrary x:tau_C instance. | [CL owner](../design/symbol-world/function-object-call-model.md) | Defined semantics; source/evaluator consumer pending |
| 106-CL09 | Replace the sole Val2 self-construction witness without registering a new ConstructEdge | Purity and TypeRole persist; SelfConstructible may fail. Independently recheck WellFormedTau, including stale registration and both closure-home constraints. Equal pure Core gives the same TypeRole answer but not compatibility with distinct /tau homes. Type identity requires no dummy/deleted constructor. | [type owner](../design/symbol-world/type-values-places-and-borrow-views.md), [Pattern owner](../design/patterns-overload/pattern-relations-observation-and-extraction.md#13-structural-role-registration-and-ordinary-callables) | Defined semantics; source/evaluator consumer pending |
| 106-CL10 | Contribute an already formed value to another target | Keep the original owner; use a legal replication witness where required. | [CL owner](../design/symbol-world/function-object-call-model.md) | Defined semantics; source/evaluator consumer pending |
| 106-CL11 | In-place closure with free external observations | Form Needs and ordinary realizations at formation; bind the resulting tau ordinarily. Invocation does not recapture. A legal write-capable realization permits outer writes; absent capability still fails. | [CL owner](../design/symbol-world/function-object-call-model.md) | Defined semantics; source/evaluator consumer pending |
| 106-CL12 | Wrap an in-place result in Product, group or tau and transfer it | Preserve actual dependencies and lifecycle obligations. No placement-based ban applies; wrapping cannot extend ValidRegion or erase escape checks. | [CL owner](../design/symbol-world/function-object-call-model.md) | Defined semantics; source/evaluator consumer pending |
| 106-CL13 | Distinct ordinary and in-place declarations form equivalent material and have equal ordinary selection evidence | Neither placement nor dependency formation origin affects applicability, specificity or preference. Distinct candidate identities remain distinct; tied maxima report ordinary ambiguity, without placement priority or fallback after selected failure. | [CL owner](../design/symbol-world/function-object-call-model.md#73-in-place-syntax-uses-automatic-dependency-formation) | Defined semantics; source/evaluator consumer pending |

### 106.11 Lifetime refinement boundary


| ID | Scenario | Required result | Canonical owner | Consumer status |
|---|---|---|---|---|
| 106-LF01 | Universal closure-to-tau formation | Do not mechanically infer ClosureTau => GlobalLifetime. | [LF owner](../design/lifetime/lifetime-policy-and-overload-boundary.md) | Checks retained; refinement handed off |
| 106-LF02 | Return a closure with empty or reference-bearing dependencies | Neither source form is categorically rejected. Check ordinary Pre/LifetimeLegal/EscapeLegal; a retained reference outside ValidRegion fails, and an empty dependency set supplies no source-based veto. | [LF owner](../design/lifetime/lifetime-policy-and-overload-boundary.md) | Checks retained; refinement handed off |
| 106-LF03 | Ordinary versus in-place classification | Neither classification replaces concrete MoveEffect/Movable or escape judgments. | [LF owner](../design/lifetime/lifetime-policy-and-overload-boundary.md) | Checks retained; refinement handed off |
| 106-LF04 | Construction and anchored Paths need validity evidence | Preserve dependencies and check interfaces; the handoff does not waive checks. | [LF owner](../design/lifetime/lifetime-policy-and-overload-boundary.md) | Checks retained; refinement handed off |
| 106-LF05 | Bounded runtime state coexists with stable descriptions | Record lifetime implementation/refinement work, not a canonical consumer blocker. | [LF owner](../design/lifetime/lifetime-policy-and-overload-boundary.md) | Checks retained; refinement handed off |
| 106-LF06 | Conformance versus implementation | Report semantic scenarios, pending consumers and executed tests separately; claim no new end-to-end support. | [LF owner](../design/lifetime/lifetime-policy-and-overload-boundary.md) | Checks retained; refinement handed off |

### 106.12 Compile partner dependencies

| ID | Scenario | Required result | Canonical owner | Consumer status |
|---|---|---|---|---|
| C-DP01 | [cap] (...) :compile => B | Ordinary capture formation retains actual dependencies; selected partner identity is stable. | [dependency owner](../design/symbol-world/dependency-observation-and-realization.md) | Source consumer pending |
| C-DP02 | P let H => E | Requested-name producer may deliver any ordinary declared result; no implicit actual or authority follows from absent head. | [declaration owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md) | Source consumer pending |
| C-DP03 | Callable observes an enclosing local | Form an ordinary admitted dependency at callable formation; invocation never recaptures by caller spelling. | [dependency owner](../design/symbol-world/dependency-observation-and-realization.md) | Source consumer pending |
| C-DP04 | Equal parent/selected partner/canonical In with different incidental caller environments | Same key and instance; no hidden caller coordinate, replay or stale authority. | [invocation owner](../design/static-evaluation/compile-instance-invocation-and-result-delivery.md) | Source consumer pending |
| C-DP05 | Nested ordinary or in-place closure reaches enclosing instance | Preserve lexical/Self/navigation layer and use ordinary self-name plus admitted dependency. | [dependency owner](../design/symbol-world/dependency-observation-and-realization.md) | Source consumer pending |

### 106.13 Callable construction and contribution consumers

| ID | Scenario | Required result | Canonical owner | Consumer status |
|---|---|---|---|---|
| 106-CA01 | P let ()::path:t, then legal initialization | Establish ordinary Val2[()] without modifying V_T. | [Owner](../design/symbol-world/names-and-overload-groups.md) | Defined semantics; source/evaluator consumer pending |
| 106-CA02 | TypeAdd(T,v) with all premises | Only V_T gains AnchorFor(v,T); no associated Val2[()] on T is installed. | [Owner](../design/symbol-world/names-and-overload-groups.md) | Defined semantics; source/evaluator consumer pending |
| 106-CA03 | Ordinary singleton/local let f=C | Resident is tau_C:type, with no function-object wrapper. | [Owner](../design/symbol-world/names-and-overload-groups.md) | Defined semantics; source/evaluator consumer pending |
| 106-CA04 | Two declarations in an established same-name closure bucket | Jointly form c_1^f and c_2^f at T_f's home and register both in its V_T_f. | [Owner](../design/symbol-world/names-and-overload-groups.md) | Defined semantics; source/evaluator consumer pending |
| 106-CA05 | Attempt TypeAdd(T_f,tau_C_i) | Reject the type result with absent Val1 as a TypeMember. | [Owner](../design/symbol-world/names-and-overload-groups.md) | Defined semantics; source/evaluator consumer pending |
| 106-CA06 | Attempt automatic import of all V_tau_C_i | No bulk import: the declaration contributes only its own formation projection. | [Owner](../design/symbol-world/names-and-overload-groups.md) | Defined semantics; source/evaluator consumer pending |
| 106-CA07 | Permute discovery of unordered sibling contributions | Preserve the joined result and entry identities; no first sibling/resident, reordered effects or duplicated initialization authority. | [Owner](../design/symbol-world/names-and-overload-groups.md) | Defined semantics; source/evaluator consumer pending |
| 106-CA08 | Same-spelled ordinary lexical lets with closure RHS | Retain binding/shadowing/duplicate rules; spelling and RHS confer no ContributionRole or TypeAdd. | [Owner](../design/symbol-world/names-and-overload-groups.md) | Defined semantics; source/evaluator consumer pending |
| 106-CA09 | v in V_T | Require present Val1, ordinary callability, target classifier home and registration. Enter Type(v)'s associated Val2[()] in the single candidate union. | [Owner](../design/symbol-world/names-and-overload-groups.md) | Defined semantics; source/evaluator consumer pending |
| 106-CA10 | Known-target formation versus later contribution of existing c | Initially form c_C^T directly; later rehosting uses AnchorFor(c,T) and required witness on c, preserving old identity/dependencies. Never rehost tau_C or replay initializers. | [Owner](../design/symbol-world/names-and-overload-groups.md) | Defined semantics; source/evaluator consumer pending |
| 106-CA11 | An associated operation consumes a held tau after another same-P Core changes | Enumerate only actual Val2 resident witnesses captured by Core(tau); later candidates or declaration-cell replacements remain outside that snapshot. | [Owner](../design/symbol-world/function-object-call-model.md) | Connected literal-construction slice covered |
| 106-CA12 | P is allocated without a Core Object, or an implementation ledger has no resident | No Object/member authority follows; required observation is unavailable and publishes no result. | [Owner](../design/symbol-world/function-object-call-model.md) | Registered Core and ordinary-call entry boundaries covered |

### 106.14 Two-level NameValue and Path observations

| ID | Scenario | Required result | Canonical owner | Consumer status |
|---|---|---|---|---|
| 106-PT15 | NameExpr in Path and value-expected contexts | Read_name retains full NameValue; only value use performs Read_resident. Equal residents do not equate structures. | [Owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT16 | Compare e# and path_pattern projection | One defined projection: NameExpr stops at its first level; general values require applicable projection, with no source quotation fallback. | [Owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT17 | Round-trip a NameExpr n and an already bound NameValue a in the Path consumer | For n, first obtain Read_name(n), then project/reinject. For a=NameValue(field::adl), Interpret_Path(PathPatternProjection(a)) =_Path a and reprojection yields a#. Preserve the bound structure without repeating NameExpr lookup or making a node from binder spelling a. The polarity flip supplies no Path decoder and performs no resident read. | [Owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT18 | Compare direct name_express$ with projection followed by $ | Each operand is interpreted in the opposite context; explicit path_pattern also performs its ordinary projection. No implicit conversion or repeated evaluation. | [Owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT19 | Index ((field::adl)#)[0] | Return relative field:: path_pattern, not string or a path retaining the adl endpoint. | [Owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT20 | Equal segment strings with different endpoints or explicit roots | Omega retains endpoint distinctions and actual root material, not an extra dependency coordinate. Ordinary dependency/lifetime obligations remain in force; strings cannot recover identity or authority. | [Owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT21 | Reconstruct a Path whose target became unavailable | Surrounding Read_resident checks current access/validity without reopening or deriving authority from projection. | [Owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT22 | Ordinary slice construction versus indexing | Indexing yields a relative single-name path_pattern. RHS colon forms ordinary Product |> slice; empty colon/comma slots are unit. Path/container families define ordinary Pattern extraction and boundary interpretation, with no Slice_Omega primitive. | [Owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | Defined semantics; source/evaluator consumer pending |

The consumer scenarios are independently testable. Each acceptance or rejection depends
on its owner's premises; schematic source is not an unconditional theorem.

## Pre-implementation alignment acceptance cases

These 30 cases fix documentation-level obligations; source consumers remain pending.
They add no executable tests or new semantic ontology.

| ID | Scenario | Required result | Canonical owner |
|---|---|---|---|
| ALIGN-01 | P \|> E, P E, ()f, x f, (x,y) f | Delete only the pipe. Never exchange operands or accept traditional callee-first calls as this source grammar. | [Owner](../design/symbol-world/function-object-call-model.md) |
| ALIGN-02 | x.field or movement of its ordinary result | Use x.field == x \|> .field == x \|> field::adl through ordinary invocation. No NameExpr/Place shortcut or partial-parent field move. | [Owner](../design/symbol-world/function-object-call-model.md) |
| ALIGN-03 | A new legal name::adl request after Close | Frozen finite generative rule answers it without reopening adl/t, enumerating infinite Val2 or changing Pattern/V_tau registrations. | [Owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md) |
| ALIGN-04 | Generative field::adl receiver and type-path candidates | Requested-name projection ((field#)[0])$::t supplies the ordinary target Path. Select method or type-path candidates by general applicability and specificity, preserving actual self. | [Owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md) |
| ALIGN-05 | The same name::path material in construction and extraction positions | Keep direction and structural isomorphism; no path::name reversal or second namespace algebra. | [Owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) |
| ALIGN-06 | name::a and name::(a$) in Path interpretation | Interpret bare a in S with inherited navigation and a$ in V. Check actual material and ordinary Path equality; syntax alone guarantees no equality. | [Owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) |
| ALIGN-07 | a, a$, a$$ and a$$$ in extraction interpretation | Use S, V, S and V respectively, preserving Hole identity. Each actual result is checked by the extraction consumer through R_Gamma; neither equality nor inequality follows merely from spelling. | [Owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) |
| ALIGN-08 | A complex name_express with ordinary compile calls and express$::express$ operands, followed by # | Complete ordinary computation once to NameValue, then project without resident read. No AST capture, skipped effect or repeated evaluation. | [Owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) |
| ALIGN-09 | name_express$ versus (name_express \|> path_pattern)$ | Each $ flips interpretation polarity; the latter operand also performs ordinary Path projection. Missing projection applicability cannot be repaired by source quotation. | [Owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) |
| ALIGN-10 | NameExpr and already bound NameValue Path round-trips | Path consumer preserves structural equality and anchors, not arbitrary Object identity or binder spelling. Resident authority is rechecked only by its own consumer. | [Owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) |
| ALIGN-11 | Name-headed in-place callable in ordinary and type calls | Formal head retains (_, let _: (name)$). Contextual self observes selected x or c; full Param input follows the call projection, and the explicit actual never supplies self. | [Owner](../design/patterns-overload/pattern-relations-observation-and-extraction.md) |
| ALIGN-12 | E return versus plain E at a path tail | The former already has ReturnEvent and infers only target; the latter first uses the serial consumer. Do not merge these operations in Omitted. | [Owner](../design/control-flow/targeted-return-and-d-reduction.md) |
| ALIGN-13 | A non-tail serial expression yields non-unit | UnitDiscard rejects; it does not silently discard the result, synthesize a lexical temp or turn it into return. | [Owner](../design/control-flow/targeted-return-and-d-reduction.md) |
| ALIGN-14 | Tail unit versus explicit unit_value return | Tail unit falls through; explicit unit produces ReturnEvent. Neither fabricates a local return contribution. | [Owner](../design/control-flow/targeted-return-and-d-reduction.md) |
| ALIGN-15 | Each branch has a true non-unit tail, or a following continuation | Determine tailness per control-flow path, not AST list position. Synthesize return only at the true tail, then infer the outermost function target in the reachable segment bounded by the nearest Ordinary closure. | [Owner](../design/control-flow/targeted-return-and-d-reduction.md) |
| ALIGN-16 | A non-unit tail's root overload and established ReturnPattern/Pout | Apply immediate demand before sealed maxima, retain both outer P1/P2 constraints; no speculative body, hidden temp, second evaluation or reopen. | [Owner](../design/control-flow/targeted-return-and-d-reduction.md) |
| ALIGN-17 | Construction material is incomplete, or construction Pre fails | No partial Object identity exists. Successful atomic ConstructCommit establishes one complete result; failure publishes none. | [Owner](../design/patterns-overload/pattern-relations-observation-and-extraction.md) |
| ALIGN-18 | A selected consuming parent decomposition | Atomic ExtractCommit ends parent identity and establishes complete children. No observable partly extracted parent or field-death bitmap. | [Owner](../design/patterns-overload/pattern-relations-observation-and-extraction.md) |
| ALIGN-19 | Applicability proof or non-consuming registered borrowed observation | R_Gamma/FieldView evidence alone performs no destructive commit and grants no missing authority. | [Owner](../design/patterns-overload/pattern-relations-observation-and-extraction.md) |
| ALIGN-20 | A destructor begins | It receives complete extracted children and runs ordinary NLL/with/use/move/drop, never a second destructor evaluator or partially destroyed parent. | [Owner](../design/lifetime/lifetime-policy-and-overload-boundary.md) |
| ALIGN-21 | Selected affine field : T -> F candidate | Complete F follows ordinary movement/lifecycle. Affine use alone proves neither Kill nor a partial aggregate; custom ADL cannot redefine structural extraction. | [Owner](../design/symbol-world/type-associated-function-objects-and-access-trees.md) |
| ALIGN-22 | A legal Preserve Move and a separately available observable clone | Do not invoke clone for Preserve. Movable, Killable and fixed MoveEffect remain distinct; failed Pre does not switch effects. | [Owner](../design/lifetime/lifetime-policy-and-overload-boundary.md) |
| ALIGN-23 | Stable ordinary type, compile type outside OpenHere, or accessible closed/global resident | Apply existing instance survival and frontier checks. Local observation transport cannot kill the stable subject; equality does not merge lifecycle instances. | [Owner](../design/lifetime/lifetime-policy-and-overload-boundary.md) |
| ALIGN-24 | A Kill would invalidate an established admitted observation | Reject at Pre before mutation. Alive/dead status alone adds no new observable branch when all admitted observations are unaffected. | [Owner](../design/lifetime/lifetime-policy-and-overload-boundary.md) |
| ALIGN-25 | Ordinary value copy-derived route, or ref/share copy-derived route | Select share+clone or rebind+clone respectively, form fresh complete result and terminal Move; never pre-move source or add Copy as terminal pass. | [Owner](../design/mechanical-lowering/mechanical-argument-passing-and-move-fixed-point.md) |
| ALIGN-26 | Selected clone supplies a lawful non-source origin post | Use that post; no universal origin(result)=source. Copyable implies neither default copy nor movement permission; failure never reopens selection. | [Owner](../design/mechanical-lowering/mechanical-argument-passing-and-move-fixed-point.md) |
| ALIGN-27 | An explicit ref/share handle is passed | Move the already formed handle with its exact type and origin. No implicit borrow pass or increasing borrow depth. | [Owner](../design/mechanical-lowering/mechanical-argument-passing-and-move-fixed-point.md) |
| ALIGN-28 | Three otherwise unordered cleanups share point p with a declared before b before c | At fixed p execute c,b,a. No alternative-order equivalence question remains. | [Owner](../design/mechanical-lowering/mechanical-argument-passing-and-move-fixed-point.md) |
| ALIGN-29 | NLL/with/last-use/move/explicit-drop/return facts order events or place them at different points | Apply them first. Declaration-order tie-break cannot override precedence, shift a point or add old-generation cleanup after killing Move. | [Owner](../design/mechanical-lowering/mechanical-argument-passing-and-move-fixed-point.md) |
| ALIGN-30 | @ observes a destructor/cleanup continuation | Observe the fully fixed sequence and committed lifecycle facts. Do not solve cleanup again or discover failed Pre after mutation. | [Owner](../design/lifetime/lifetime-policy-and-overload-boundary.md) |

## Decision-to-owner map

| Decision | Canonical owner / checks |
|---|---|
| D01 | [PD owner](../design/patterns-overload/pattern-relations-observation-and-extraction.md), 106-PD cases above |
| D02 | [PD owner](../design/patterns-overload/pattern-relations-observation-and-extraction.md), 106-PD cases above |
| D03 | [PD owner](../design/patterns-overload/pattern-relations-observation-and-extraction.md), 106-PD cases above |
| D04 | [NS owner](../design/symbol-world/symbol-construction-units-and-namespace-origin.md), 106-NS cases above |
| D05 | [PT owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md), 106-PT cases above |
| D06 | [polarity owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md), 106-FL cases above |
| D07 | [PT owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md), 106-PT cases above |
| D08 | [PT owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md), 106-PT cases above |
| D09 | [NM owner](../design/symbol-world/names-and-overload-groups.md), 106-NM cases above |
| D10 | [GN owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md), 106-GN cases above |
| D11 | [GN owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md), 106-GN cases above |
| D12 | [AD owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md), 106-AD cases above |
| D13 | [RP owner](../design/symbol-world/policy-and-static-flow-projection.md), 106-RP cases above |
| D14 | [RP owner](../design/symbol-world/policy-and-static-flow-projection.md), 106-RP cases above |
| D15 | [polarity owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md), 106-FL cases above |
| D16 | [polarity owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md), 106-FL cases above |
| D17 | [RP owner](../design/symbol-world/policy-and-static-flow-projection.md), 106-RP cases above |
| D18 | [PT owner](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md), 106-PT cases above |
| D19 | [DP owner](../design/symbol-world/dependency-observation-and-realization.md), 106-DP and C-DP boundary cases above |
| D20 | [CL owner](../design/symbol-world/function-object-call-model.md), 106-CL cases above |
| D21 | [CL owner](../design/symbol-world/function-object-call-model.md), 106-CL cases above |
| D22 | [CL owner](../design/symbol-world/function-object-call-model.md), 106-CL cases above |
| D23 | [LF owner](../design/lifetime/lifetime-policy-and-overload-boundary.md), 106-LF cases above |

## Joint extraction and runtime reification acceptance targets

These are canonical semantic acceptance scenarios. Pending consumer rows are
specification targets, not executable success claims. Connected evidence is
recorded only after its actual consumer has positive and negative tests.

| ID | Scenario | Required judgment / counterexample | Owner | Consumer status |
|---|---|---|---|---|
| J01 | let a:b over value, type and higher ranks | rho(a)=v, rho(b)=Type(v), Rank(b)=Rank(a)+1 at every adjacent rank | [Pattern](../design/patterns-overload/pattern-relations-observation-and-extraction.md#10-joint-extraction-across-adjacent-universe-ranks) | consumer pending |
| J02 | bare b versus R$ and uint8:: | b extracts locally; external Pattern explicitly constrains via R_Gamma; annotated Packs require the whole remaining Product, including an empty one | [Pattern](../design/patterns-overload/pattern-relations-observation-and-extraction.md) | consumer pending |
| J03 | P let name::path:t | separate structural name/Place judgment; no resident on initializer-free formation | [Names](../design/symbol-world/names-and-overload-groups.md) | consumer pending |
| J04 | underscore in V and S | V supplies universerank, S anonymous Pattern; neither is a named Hole | [Unified](../design/unified-static-name-and-structural-semantics.md#13-adjacent-universe-joint-extraction) | frontend carrier; semantic formation pending |
| J05 | dollar 0/1/2/3/deep | even restores polarity, odd flips; no resident read or stage action | [Polarity](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | frontend connected; evaluation pending |
| J06 | initializer-free simple let a:b, including an existing same-spelled b | bare b remains classifier extraction material; no subject means no extracted classifier or typed Place | [Names](../design/symbol-world/names-and-overload-groups.md#5-typed-name-declarations-and-complete-let-bindings) | formation pending; Norm handoff rejection/no-publication tested in initializer_free_heads_preserve_extraction_and_name_formation_boundaries |
| J07 | initializer-free let a:T:: and let a:R$ | explicit external Pattern must determine a complete classifier; fresh lexical formation yields typed NameExpr and Uninitialized Place, no resident | [Names](../design/symbol-world/names-and-overload-groups.md#5-typed-name-declarations-and-complete-let-bindings) | formation pending; same test covers unavailable external-classifier formation, not syntax or successful realization |
| J08 | initializer-free P let name::path:t, with :t omitted or written | qualified structural formation checks root/OpenHere/selector/freshness and declared type; only its omitted :t defaults to type | [Names](../design/symbol-world/names-and-overload-groups.md#5-typed-name-declarations-and-complete-let-bindings) | formation pending; same test covers qualified-head rejection/no-publication with written :t |
| SRC01 | empty .lang under a/b at package or toolchain root | preserve inherited navigation; semantic world, names, owners and namespace projection are unchanged | [Physical](../design/build-package/build-system-design.md#2-neutral-physical-normalization) | connected: empty_inherited_sources_preserve_semantic_world_at_package_and_global_roots |
| SRC02 | actual declaration under inherited navigation | requires ordinary qualified destination consumer; missing consumer publishes no directory, owner or member | [Physical](../design/build-package/build-system-design.md#2-neutral-physical-normalization) | rejection connected: inherited_action_without_qualified_consumer_publishes_no_destination_or_owner; successful consumer pending |
| SRC03 | empty nested file beneath a same-spelled initialized type binding | file material creates no namespace-subspace beside the Object | [Physical](../design/build-package/build-system-design.md#2-neutral-physical-normalization) | connected: empty_nested_source_does_not_create_a_namespace_beside_a_type_binding |
| SRC04 | source install prefix names a missing namespace | discovery cannot realize that namespace; unavailable Path boundary leaves semantic world unchanged | [Namespace](../design/build-package/namespace-assembly.md) | connected rejection: source_install_prefix_cannot_allocate_a_missing_semantic_namespace |
| LIT01 | let a:integer::=42, or the same already formed abstract value | extraction preserves actual integer classifier and literal payload; forms no constructor CompileInstance | [Literal](../design/symbol-world/type-values-places-and-borrow-views.md#41-abstract-literal-denotations-and-concrete-machine-types) | connected: external_classifier_checks_the_actual_literal_without_construction and abstract_literal_pipeline fixture |
| LIT02 | let a:uint16::=42, or the same already formed abstract value | direct relation proves root mismatch; selected forced binding requests one ordinary root conversion and checks its result; original abstract value remains intact | [Result](../design/patterns-overload/return-value-extraction-and-implicit-decomposition.md#forced-extraction-and-one-root-conversion) | scalar literal slice connected: forced_root_mismatch_converts_once_and_preserves_the_initial_value and forced_root_conversion fixture |
| LIT03 | explicit ordinary numeric constructor request | selected call produces a concrete result, retains abstract source, and obeys exact held Core/no-reopen | [Literal](../design/symbol-world/type-values-places-and-borrow-views.md#41-abstract-literal-denotations-and-concrete-machine-types) | connected ordinary constructor slice: literal_construction_tests |
| LIT04 | explicit scalar constructor with equal/different canonical inputs and distinct explicit parent owners | equal keys reuse instance; changed inputs or parent remain distinct; scalar result creates no self-name residency | [Invocation](../design/static-evaluation/compile-instance-invocation-and-result-delivery.md) | connected: explicit_scalar_construction_keys_include_canonical_inputs_and_parent_owner |
| FC01 | selected forced extraction with a directly matching root | deliver original value with no converter call | [Result](../design/patterns-overload/return-value-extraction-and-implicit-decomposition.md#forced-extraction-and-one-root-conversion) | scalar binding connected: external_classifier_checks_the_actual_literal_without_construction |
| FC02 | missing root, unavailable actual classifier or partial multi-entry material | supplies no proved total-root mismatch; unavailable without conversion/publication | [Result](../design/patterns-overload/return-value-extraction-and-implicit-decomposition.md#forced-extraction-and-one-root-conversion) | connected: unknown_or_partial_root_material_cannot_authorize_conversion |
| FC03 | selected root converter fails | terminal failure; no converter retry, producer reopen or partial publication | [Result](../design/patterns-overload/return-value-extraction-and-implicit-decomposition.md#forced-extraction-and-one-root-conversion) | scalar slice connected: forced_root_selected_conversion_failure_never_reopens_or_publishes |
| FC04 | ordinary overload applicability has classifier mismatch although constructor exists | inapplicable relation remains inapplicable; no forced-consumer conversion | [Overload](../design/patterns-overload/overload-resolution-design.md#4-pattern-applicability) | connected: applicability_rejects_classifier_mismatch_even_when_a_constructor_exists |
| FC05 | structured Product/return-delivery root mismatch | one conversion at the total outward root; none at individual fields/Holes; final relation checked once | [Result](../design/patterns-overload/return-value-extraction-and-implicit-decomposition.md#forced-extraction-and-one-root-conversion) | general root material, conversion and source return consumers pending |
| FC06 | missing conversion family or unconnected general source family | explicit terminal failure/unavailable with no publication | [Result](../design/patterns-overload/return-value-extraction-and-implicit-decomposition.md#forced-extraction-and-one-root-conversion) | connected: forced_root_conversion_failure_or_missing_consumer_publishes_nothing |
| FA01 | graph Callable and same-spelled namespace | ordinary Callable expectation selects the object role; graph record establishes no implementation or field authority | [Fields](../design/symbol-world/type-associated-function-objects-and-access-trees.md#struct-owned-derived-forwarding-formation) | connected graph slice: expectation_callable_resolves_object_role |
| FA02 | ordinary callable residents named field/ref/share | names and residency supply no DirectPatternChild or struct helper registration | [Fields](../design/symbol-world/type-associated-function-objects-and-access-trees.md#struct-owned-derived-forwarding-formation) | connected: ordinary_val2_members_never_create_direct_pattern_children |
| FA03 | dot/ADL and struct helpers | dot syntax enters ordinary field::adl; helper formation needs actual struct registration; missing producer publishes nothing | [Fields](../design/symbol-world/type-associated-function-objects-and-access-trees.md#struct-owned-derived-forwarding-formation) | syntax goldens and structural_helper_boundary connected; general ADL/helper execution pending |
| P01 | Relative/Relative and Relative/Absolute | preserve existing endpoint-compatible results | [Path](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md#13-composition-and-endpoints) | consumer pending |
| P02 | Absolute/Relative | undefined for all actual values | [Path](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | consumer pending |
| P03 | Absolute/Absolute | require actual rooted structural suffix overlap | [Path](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | consumer pending |
| P04 | ((a::b::)#)$::(b::) | equals a::b::, including computed equal Path values | [Path](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | consumer pending |
| P05 | same text, different roots/anchors/endpoints | reject overlap; preserve root identity and dependencies | [Path](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | consumer pending |
| P06 | let a p1 p2 p3 | complete suffixes p1::p2::p3, p2::p3, p3:: under common root | [Path](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | consumer pending |
| P07 | explicit full-navigation module | reset inherited chain; normalized identity retains actual structure/extraction | [Path](../design/symbol-world/structured-path-algebra-and-interpretation-polarity.md) | consumer pending |
| P08 | T.a.b | equals b::a::T only when each selected candidate advances type Path | [Unified](../design/unified-static-name-and-structural-semantics.md#16-fully-specified-path-and-structural-navigation) | consumer pending |
| H01 | omitted Hole Policy | inherit outer coordinates | [Policy](../design/symbol-world/policy-and-static-flow-projection.md#61-hole-local-dimension-replacement) | syntax carrier; consumer pending |
| H02 | written Hole dimensions | active replacement; omitted dimensions inherit; no outer-strength gate | [Policy](../design/symbol-world/policy-and-static-flow-projection.md) | consumer pending |
| H03 | distinct Pin/Pout/HoleOverride | Hole override does not inherit ElabIn/ElabOut restrictions | [Policy](../design/symbol-world/policy-and-static-flow-projection.md) | consumer pending |
| H04 | each Hole Pattern and Policy | solve independently through ordinary relation; compatible valuations required | [Pattern](../design/patterns-overload/pattern-relations-observation-and-extraction.md) | consumer pending |
| H05 | <p> let <p let h> | inner p references preceding exact PatternRoot/HoleBinderId | [Pattern](../design/patterns-overload/pattern-relations-observation-and-extraction.md) | frontend scope; semantic consumer pending |
| B01 | bool formation | true|false unique structural branches; if/else ordinary value bindings | [Pattern](../design/patterns-overload/pattern-relations-observation-and-extraction.md#9-bare-nodes-and-pipe-branch-shorthand) | consumer pending |
| B02 | bare branch | unique actual opaque value; no observable machine encoding or automatic field | [Unified](../design/unified-static-name-and-structural-semantics.md#15-bare-branches-and-ordinary-bool-values) | consumer pending |
| B03 | compile Pattern/runtime value | ordinary branch matching; no extra runtime-policy type | [Policy](../design/symbol-world/policy-and-static-flow-projection.md) | consumer pending |
| B04 | pipe name versus explicit head | Raw/Norm head equals (_, let _: (name)$) | [Pattern](../design/patterns-overload/pattern-relations-observation-and-extraction.md) | frontend connected: joint_extraction_and_polarity / parser and Norm goldens |
| O01 | struct(T) | preserve structural registration; add actual ordinary helpers only | [Construction](../design/symbol-world/structural-type-formation-and-composition.md) | producer unavailable |
| O02 | derived T ref forwarding | generated by ordinary struct open computation and actual registration | [Associated](../design/symbol-world/type-associated-function-objects-and-access-trees.md) | producer unavailable |
| O03 | ref without helper registration | borrow/lifecycle only; no automatic structural forwarding | [Associated](../design/symbol-world/type-associated-function-objects-and-access-trees.md) | no forwarding producer |
| O04 | derived state(T0), then Write(place,T1) | state stays on concrete T0; new call required for T1 members | [Construction](../design/symbol-world/structural-type-formation-and-composition.md) | general state consumer pending |
| R01 | type call | CallTarget=tau, ActualSelf=c*, contextual first formal retained; Param/Return projections independent | [Call](../design/symbol-world/function-object-call-model.md#11-callable-tag-and-activation-observations) | selection connected; full projection/tag consumer pending |
| R02 | ordinary value call | actual implicit self=x; tag=x | [Call](../design/symbol-world/function-object-call-model.md) | ordinary selection connected |
| R03 | recursive and nested tags | tag plus actual activation selects distinct frames | [Return](../design/control-flow/targeted-return-and-d-reduction.md#callable-tag-to-active-frame) | routing consumer pending |
| R04 | compile implicit dependencies | ordinary formation allowed; unavailable concrete runtime read waits | [Dependency](../design/symbol-world/dependency-observation-and-realization.md) | general realization pending |
| R05 | NameExpr with opaque self | Norm_Name independent of capture-layout unfolding; result dependencies retained | [Unified](../design/unified-static-name-and-structural-semantics.md#18-dependencies-and-opaque-self) | consumer pending |
| E-R01 | callable P2 reify | outward runtime callable; result classifier independent of reverse migration; ordinary ReturnPattern/Pout delivery | [E](../design/static-evaluation/evaluation-residual-and-optimization.md#8-runtime-reification-of-e) | active closed-boundary execution unavailable |
| E-R02 | (self,args):reify -> _:R$ => body | explicit external contract; bare R is extraction binder | [E](../design/static-evaluation/evaluation-residual-and-optimization.md) | consumer pending |
| E-R03 | compile let val=runtime_express inside closed reify P2 | actual dependent continuation resumed by same E with runtime input | [E](../design/static-evaluation/evaluation-residual-and-optimization.md) | continuation materialization pending |
| E-R04 | already selected versus newly formed runtime call | no reopen versus first ordinary selection at actual frontier | [E](../design/static-evaluation/evaluation-residual-and-optimization.md) | no-reopen connected; reified execution pending |
| E-R05 | Region and dynamic branch | Region_K(n)=R(NamePosition_K(n)); no future-path enumeration by E | [Lifecycle](../design/lifetime/lifetime-policy-and-overload-boundary.md#current-name-position-and-delayed-e) | lifecycle substrate connected; source position wiring pending |
| E-R06 | AOT/JIT/cache/precomputed tables | same E semantics; no Cartesian enumeration admission axiom | [E](../design/static-evaluation/evaluation-residual-and-optimization.md) | optimization consumer pending |

## Classifier observation and first-round selection

The [relation owner](../design/patterns-overload/pattern-relations-observation-and-extraction.md#classifier-observation-and-decided-relation-outcomes)
and [selection owner](../design/patterns-overload/overload-resolution-design.md)
define these scenarios. Connected evidence covers their observation/rejection
boundary, not general source body completion or richer Pattern evaluation.

| ID | Scenario | Required result | Connected evidence |
|---|---|---|---|
| OBS01 | Unknown expression against Hole, named alternative, sequence and type-role constraints, with a real wildcard competitor in either order | Unavailable before maxima, selection or CompileInstance formation; no publication | classifier_observation_boundary |
| OBS02 | Same wildcard candidate without the constrained competitor | First-round selection can finish; later input/body consumer may be unavailable | classifier_observation_boundary |
| OBS03 | Known actual classifier, successful named alternative and another unavailable named observation | Preserve unavailable obligation; no candidate selection in either order | classifier_observation_boundary |
| OBS04 | Known unequal actual and expected classifier, with wildcard competitor | Reject only the proved mismatch; competitor may be selected; later selected failure is terminal | classifier_observation_boundary, relational_pattern_authority |
| OBS05 | Hole/direct named/sequence query with no actual observation; named resolver absent or returning no observation | Explicit unavailable, distinct from known detached Pattern-coordinate equality | relational_pattern_authority |
| OBS06 | Unknown role versus observed value/unit versus observed type in type-role query | Unknown role is unavailable; known non-type is mismatch; observed type still requires classifier evidence | relational_pattern_authority |
| OBS07 | Plain binder, anonymous wildcard or unconstrained whole-remainder Pack with unobserved classifier | Bind/discard without requesting irrelevant classifier facts; shared Hole conflict remains a decided mismatch | relational_pattern_authority |
| OBS08 | Distinct lookup keys with equal canonical Core; constrained candidate versus wildcard | Both applicable; ordinary specificity selects the constrained candidate; selected failure cannot reopen | semantic_world equal_core_distinct_lookup_arguments_select_the_constrained_candidate |
| OBS09 | Old/new ordinary values sharing one Pattern root and lookup key but holding unequal classifier Core | First-round external classifiers distinguish their frozen snapshots; no fresh registered-Core read | classifier_snapshot_boundary |
| OBS10 | Missing carried complete type or frozen ordinary classifier | Unavailable before applicability maxima or instance admission; no publication | semantic_world unavailable_type_input_never_enters_the_selected_body_or_publishes_an_instance and equal_core_distinct_lookup_arguments_select_the_constrained_candidate |
| OBS11 | Ordinary self has an explicit external classifier | Compare expected Core with actual frozen classifier Core; required unknown observation is unavailable | self annotation consumer uses ordinary classifier relation; equivalent-core selection regression covers explicit arguments |

## Forced input observation and complete delivery publication

The [forced extraction owner](../design/patterns-overload/return-value-extraction-and-implicit-decomposition.md#forced-extraction-and-one-root-conversion)
uses Query_Gamma outcomes; TotalRootMismatch refines Mismatch evidence and
Unavailable grants no conversion. The [migration contract](../contracts/policy-migration.md)
requires final demanded projection within the publication boundary.

| ID | Scenario | Required result | Evidence / frontier |
|---|---|---|---|
| FC07 | Stored compile literal observed through a runtime source binding, forced scalar root mismatch | Compile converter cannot read the hidden Val1; unavailable, no instance or destination published | world runtime_source_view_cannot_supply_a_compile_root_converter |
| FC08 | Forced conversion with an actual source observation edge | Preserve both Policy ranks and mode through input exposure; stored value mode cannot replace edge mode | ConstructionRequest source_view and existing literal conversion regressions; general Ready/source E pending |
| MTX01 | Actual ordinary migration body/result followed by complete final demand satisfaction | Publish scratch only after valid result class, nonempty Pair projection and delivered=declared=target Mode | scratch placement and supplied-result validator tested; successful source producer, independent delivery observations and post-production projection-failure execution coverage pending |
| MTX02 | Same source lookup key, held old Core and successor current Core, with distinct migration return classifiers | Migration A compares held source Core with declaration's resolved result Core; selected failure never publishes or retries | migration_snapshot_boundary |
| MTX03 | Only one fully admissible candidate, with const result declaration and mut target | Normal selection retains the candidate; selected declaration contradiction terminates before body/instance entry, without fallback or publication | migration_snapshot_boundary sole_admissible_migration_with_wrong_output_mode_fails_after_selection_without_publication |
| MTX04 | Supplied actual delivery, selected declared Mode and target Mode, all const/mut combinations | Only exact three-way agreement succeeds; preserve complete delivered entries unchanged; every disagreement rejects | ordinary_invocation supplied_migration_delivery unit tests; supplied completion seam, not executed source migration |
| MTX05 | Empty/wrong-Pair projection or mixed delivered Modes under one complete demand | Reject whole delivery; no Mode retagging or partial filtering to manufacture success | ordinary_invocation supplied_migration_delivery unit tests; actual source completion pending |
| FC09 | Unique complete outward T, equal outer classifier Core, and full Pattern unsatisfiable through an internal constraint | Derive TotalRootMismatch from whole-query negative proof; attempt one ordinary conversion of the entire subject to T, then recheck full P | canonical rule closed; compound relation/conversion consumer pending |
| FC10 | Failed alternative/valuation with a remaining solution or unavailable required evidence | No whole-query Mismatch proof; Match or Unavailable as appropriate, no root conversion | canonical acceptance target; general alternative consumer pending |
| FC11 | Whole-query negative proof but unknown/non-unique complete outward root type | Unavailable; do not guess a target or insert internal conversions | scalar missing-root boundary connected; general compound consumer pending |
| PR14 | Named nested structural registration inside an ordered outer layer | Preserve the named layer and its layer-local order; it is not exposed bare call syntax | canonical_value nested_named_product_registration_is_preserved_inside_ordered_outer_layer |
| PR15 | Saved ordinary object used as call input | One argument; source call-input normalization does not inspect or expand the object | classifier_snapshot_boundary; registered nested schema preservation is separately PR14 |
| PR16 | Runtime-only collection input whose length is unavailable | No static length expansion or speculative arity selection; iteration consumer required | runtime saved-value observation gate connected; actual t * ω formation/pack? consumer pending |

## Shared argument navigation and contextual edge observation

The [names owner](../design/symbol-world/names-and-overload-groups.md#3-type-and-group-call-projection)
fixes shared navigation before observation; the [Policy owner](../design/symbol-world/policy-and-static-flow-projection.md#observation-edges-and-unique-contextual-projection)
keeps distinct observation edges. These rows have actual invocation evidence in
argument_navigation_observation; they establish the connected identity/exposure
and non-unique-projection boundaries, not general source E or joint Policy solving.

| ID | Scenario | Required result | Connected evidence |
|---|---|---|---|
| NAV01 | Current inner namespace, outer ordinary argument, same-spelled default-mount argument | Full lexical lookup fixes outer identity first; its actual mode drives ordinary candidate preference | outer mut selects mut candidate over mount const control |
| NAV02 | Outer binding exists but its Val1 is hidden; default same-spelled binding is visible | Stop unavailable before A/maxima; retain fixed outer identity and publish nothing | hidden outer plus readable default regression |
| NAV03 | Same resident/Pattern with const and mut views, in either insertion order, ordinary and type arguments | Retain both complete edges; non-unique required projection is unavailable; explicit mode hints do not choose a view | both edge permutations and both hints; no selection/instance/publication |
| NAV04 | Repeated identical observation edges | One identical required observation remains, with its actual mode | mut observation selects mut candidate |
| NAV05 | Runtime-Pattern host but compile-visible terminal uint8 passed to IdentityType | Shared navigation retains host chain; hidden host blocks before first-round selection; direct uint8 control succeeds | hidden host / visible terminal integration |
| NAV06 | Host has compile and runtime Pattern edges in either order; terminal type has a unique view | Host Pattern exposure can be established by the compile edge without choosing terminal mode or erasing hidden edge | both host permutations retain all edges and permit real IdentityType result |
| NAV07 | Same complete classifier with const/mut edges used only by x:T:: | Classifier Pattern/Core observation may coincide without choosing argument mode; typed candidate wins over generic | both permutations succeed for classifier-only observation, while argument observation remains non-unique |

## Implementation handoff checklist

Statuses describe the stated boundary only. A connected syntax or rejection
boundary does not establish successful execution of the corresponding language
feature. Canonical owner definitions above are authoritative for both statuses.

| Item | Status | Evidence / next consumer |
|---|---|---|
| Stage={compile,runtime}, independent open/close and mode | 规范已闭合、实现已连接 | policy_pair_semantics and stage_observation_boundary |
| Adjacent-rank joint extraction, ordinary values through higher universes | 规范已闭合、消费者待连接 | J01–J03; classifier-side binders and Type(type) observation report unavailable before publication |
| Explicit external classifier constraint in the connected literal/type subset | 规范已闭合、实现已连接 | relational_pattern_authority, world initializer boundary tests |
| Classifier availability, frozen Core transport and decided first-round rejection | 规范已闭合、实现已连接 | OBS01–OBS10; classifier_observation_boundary, classifier_snapshot_boundary and relational_pattern_authority; general relation queries retain their unavailable frontier |
| Shared ordinary/type argument navigation, whole host exposure and facet-specific non-unique Policy observation rejection | 规范已闭合、实现已连接 | NAV01–NAV07; general multi-edge joint projection remains unavailable |
| Forced-conversion input horizon | 规范已闭合、实现已连接 | FC07–FC08; general Ready/source E remains pending |
| Selected migration declaration-Mode rejection and supplied-result final demand validation | 规范已闭合、实现已连接 | MTX03–MTX05; negative real selection plus supplied-material seam, without a positive source migration claim |
| General same-Type migration producer and actual completed delivery | 规范已闭合、消费者待连接 | MTX01; independent actual producer observations, common E completion and post-production failure coverage required |
| Empty physical fragments, neutral inherited navigation and unavailable qualified-action publication boundary | 规范已闭合、实现已连接 | SRC01–SRC04; ordinary successful qualified source consumer remains pending |
| Literal extraction without construction and explicit constructor identity/snapshot isolation | 规范已闭合、实现已连接 | LIT01–LIT04; constructor tests use explicit ordinary invocation |
| RHS underscore and arbitrary polarity parity carriers | 规范已闭合、实现已连接 | joint_extraction_and_polarity, parser/Norm golden fixture |
| Universe-rank material semantic formation | 规范已闭合、消费者待连接 | J04; semantic evaluation reports unavailable |
| Full rooted Path suffix composition and module reset | 规范已闭合、消费者待连接 | P01–P08; shared Path consumers |
| Hole-local Policy syntax and alpha identities | 规范已闭合、实现已连接 | local Policy preserved before own declaration; exact HoleBinderId test |
| HoleOverride and independent Pattern/Policy solving | 规范已闭合、消费者待连接 | H01–H05; source/relation entries reject unavailable Policy solving; annotated Packs reject unavailable remainder extraction |
| Opaque bare-branch members and bool ordinary bindings | 规范已闭合、消费者待连接 | B01–B03; common structural producer |
| Anonymous pipe-branch extraction expansion | 规范已闭合、实现已连接 | B04; Raw/Norm equality with explicit head |
| General open calls, struct helper registration, * / *= | 规范已闭合、消费者待连接 | O01–O04; ordinary producer Pre/commit/Post |
| Ref/share field-forwarding registration | 规范已闭合、消费者待连接 | actual struct helper registration required; reference primitives supply no forwarder |
| Held complete type snapshots and late-member isolation | 规范已闭合、实现已连接 | world snapshot-binding and recursive_object_identity tests |
| Compile dependencies, delayed inputs and opaque self observation | 规范已闭合、消费者待连接 | R04–R05; formation/readiness/materialization consumers |
| Callable-tag to actual activation materialization | 规范已闭合、消费者待连接 | R01–R03; CallTarget and ActualSelf remain independent |
| Active-frame return binding, recursion identity and inactive-target rejection | 规范已闭合、实现已连接 | return_target_binding; no self-type or spelling resolution fallback |
| Callable P2 reify elaboration and unavailable execution boundary | 规范已闭合、实现已连接 | P2 policy tests; closed-boundary execution unavailable before publication |
| Callable reify boundary, dependency-cone E resumption and closed delivery | 规范已闭合、消费者待连接 | E-R01–E-R04; common continuation and return contract consumers |
| Current-position lifecycle and mandatory transaction cleanup gate | 规范已闭合、实现已连接 | lifecycle substrate tests over supplied continuation facts |
| Source NamePosition, cleanup placement and reified continuation wiring | 规范已闭合、消费者待连接 | E-R05; ordinary source/control-flow facts |
| AOT/JIT/cache/table optimization representations | 规范已闭合、消费者待连接 | E-R06; equivalent O rewrites with E revalidation |

## Reify P2 and bidirectional migration acceptance

All rows define semantic acceptance targets. Pending consumers provide explicit
unavailable diagnostics, not executed success claims.

| ID | Scenario | Required result | Consumer |
|---|---|---|---|
| RF01 | callable-head :reify | outward Runtime, distinct evaluation qualification, two Stage atoms | P2 elaboration |
| RF02 | reify in P1/Pin/Pout/Hole/expression Policy | reject that Policy atom position; ordinary spelling stays a Name | Policy elaboration |
| RF03 | explicit R$ or bool:: return classifier | close from input facts before any reverse migration | closed-boundary consumer pending |
| RF04 | deduced return classifier | accept iff its dependency closure excludes reverse migration | closed-boundary consumer pending |
| RF05 | return classifier itself requires reverse migration | reject before migration/publication | closed-boundary consumer pending |
| RF06 | runtime->compile migration outside reify or at ordinary Runtime P2 | reject; matching endpoint/candidate grants no boundary | request boundary |
| RF07 | reverse migration under active closed reify | select an ordinary same-Type candidate, materialize only actual dependent successors | E consumer pending |
| RF08 | runtime:runtime->runtime:compile | runtime Object retained with its actual classifier compile-available; unrelated compile type is no witness | source/continuation consumer pending |
| RF09 | runtime-policy type value | coherent adjacent-rank observations, including a compile classifier; Val1 shape forces no Policy equality or type-algebra permission | observation substrate; type consumer pending |
| RF10 | actual dependent type work plus independent compile type work | defer dependent cone only; independent work remains ready | dependency-cone consumer pending |
| RF11 | runtime value-only compile detour | O may eliminate only with E-proved no type dependency and affected E revalidation | O consumer pending |
| RF12 | Runtime/reify P2 with explicit compile P1 or mixed output Hole Policies | Legal coherent declared constraints; reify is required only by actual reverse-dependent continuation work | P1 acceptance connected; mixed output solver pending |
| RF13 | Runtime/reify Pin with compile input | latest availability may accept it; Ready and applicability remain independent | InputAdmissible consumer pending |
| RF14 | deleted migration, host asymmetry, restricted atomic library type | no generated capability may override missing/deleted realization | ordinary candidate/capability substrate |
| RF15 | existing accepted view / selected migration failure | identity only / terminal failure, no chain or reopen | shared selection substrate |
| RF16 | runtime newly formed multi-dispatch call | first ordinary selection at actual frontier; selected predecessors retain origin | reified E consumer pending |
| RF17 | future false branch and lifecycle facts | current compile name/Color Region only; path elimination uses E O E | lifecycle substrate; source/O consumers pending |

### Adjacent-rank Policy and joint selection targets

The [Policy owner](../design/symbol-world/policy-and-static-flow-projection.md)
defines the complete absent* runtime* compile* chain, runtime first-present
and compile whole-chain demands independently of ordinary
whole-slot Mode. Output Hole solving and actual reverse E remain pending.

| ID | Scenario | Required result | Consumer status |
|---|---|---|---|
| RK01 | P2 runtime, explicit P1 compile, no reverse dependency | Accept declaration with distinct body/output views; no reify requirement from the stage combination. | Callable registration connected; source E pending |
| RK02 | Mixed compile/runtime output Hole Policies | Keep each scoped Hole constraint; no all-output recoloring. | Syntax/identity connected; output solver pending |
| RK03 | Ordinary runtime let on compile:compile | Target runtime:compile, preserve classifier; actual migration requires ordinary selection. | Rank demand/projection connected; general migration realization pending |
| RK04 | Ordinary runtime let on runtime:runtime | Keep runtime:runtime; do not silently recover classifier. | Rank demand/projection connected |
| RK05 | Ordinary runtime let on absent:compile | Target absent:runtime, preserving absent value and the higher tail; do not fabricate Present. | Demand/projection connected; higher-rank migration pending |
| RK06 | compile:runtime injected through installation, binding, export or migration | Reject before publication or successful projection. | Rank gate regressions connected |
| RK07 | Equal input Policies, competing output Policies/demands | Input/output jointly participate before maxima; no second-round body execution supplies applicability. | Ordinary selection connected; general Hole solver pending |
| RK08 | P1/P2 mixed stage pair with actual reverse-dependent successors | Require the active closed reify boundary for those successors; preserve the selected source/runtime Object and its classifier relation. | Missing-boundary rejection connected; dependent E pending |
| RK09 | Unrefined compile let on runtime:runtime | Whole-chain target has all non-absent ranks compile; pair projection is compile:compile, not compile:runtime. | Target formation connected; reverse E pending |
| RK10 | absent:runtime:runtime:compile under compile let | Target absent:compile:compile:compile; preserve absent prefix, do not create a value at that rank. | Arbitrary-depth prefix material connected; actual higher-rank migration pending |
| RK11 | Runtime or compile rank followed by absent; compile followed by runtime at any depth | Reject the prefix; pair legality cannot legalize a deeper inversion. | Prefix validation and publication guards connected |
| RK12 | absent:absent with unseen higher ranks | Legal local prefix, neither first present rank nor AllCompile proven; required higher observation is unavailable, not inapplicable fallback evidence. | Observation/selection unavailable boundary connected |
| RK13 | Equal first pair, different higher-rank observations | Prefixes remain distinct; no whole-chain equality or hidden-tail success from the pair. | Prefix material equality connected; source full-rank transport pending |
| RK14 | An actually observed first compile rank and legal chain | Compile tail follows the chain law without infinite enumeration; target material itself is no observation witness. | Shape relation connected; source evidence consumer pending |
| RK15 | Compile Policy on one scoped Hole versus unrefined compile let | The Hole constrains its rank; ordinary compile let constrains every non-absent rank. | Syntax scopes connected; general Hole solver pending |

## Closure, self and directional Function Pattern acceptance

T01–T22 retain the topic design's obligations; T23–T35 add canonical audit
boundaries. Complete closure/Function Pattern consumers remain pending. Return
segment identity/reachability has a minimal executable binding slice (T23–T26),
not capability execution or completion propagation. Existing syntax, rank-prefix, transaction and
no-reopen tests prove only their slices; they do not execute capture-mode
analysis, FnExtract/FnRequest, full self projection or machine ABI realization.
B_K, F_L/F_R and ABIKind are metanotation, not additional source syntax.

Owners: [dependency](../design/symbol-world/dependency-observation-and-realization.md),
[call](../design/symbol-world/function-object-call-model.md#62-consumer-directed-self-projections),
[Function Pattern](../design/patterns-overload/pattern-relations-observation-and-extraction.md#111-directional-function-pattern-consumers),
[lifetime](../design/lifetime/lifetime-policy-and-overload-boundary.md#216-call-boundary-summaries),
[return](../design/control-flow/targeted-return-and-d-reduction.md#callable-tag-to-active-frame),
[E delivery](../design/static-evaluation/evaluation-residual-and-optimization.md#10-e-selected-return-delivery-convention).

| ID | Scenario | Required judgment and negative boundary | Owner |
|---|---|---|---|
| CSF-T01 | Explicit [a,b] versus [b,a] | Keep source sequence as bare positional material. Different ordered contents normalize differently; equal values alone cannot merge distinct occurrence/binder identities. No name sorting or unordered conversion. | dependency |
| CSF-T02 | Explicit compile,runtime,compile captures | Keep B_K(c0,c1,c2) and each coherent rank Policy, even without physical static slots. Initializers share pre-capture scope, cannot see prior new capture binders, and execute once in ordinary order. | dependency |
| CSF-T03 | Explicit x, additional free external value y | Diagnose omitted y; no automatic addition, even for a distinct binder. Reading captured x uses its resolved binder. | dependency |
| CSF-T04 | Explicit mode plus compile structural access; runtime Object's compile classifier; runtime-only structural Path | Admit the first two as independent compile StructuralAccess. Reject runtime StructuralAccess; source runtime Val1 cannot misclassify the requested observation or disguise runtime name selection as value capture. | dependency |
| CSF-T05 | Form InPlace, then invoke/copy/return/reify | Fix resolved automatic sequence/realizations per completed closure, with no rediscovery/reordering or public portable ordinal guarantee. Admitted clause-free ordinary closures may also use Automatic mode. | dependency |
| CSF-T06 | User-defined x:T associated () call | Param=B_K(x), a singleton case. Closure Param may have 0/1/n positions; contextual first formal adds no universal environment Object. | call |
| CSF-T07 | Candidates capture 0/1/3 values, ordinary type/Policy/result demand distinguishes one | Ordinary selection may use candidate-local read-only implicit requirements; no common capture Pattern or shape preference. Tied ordinary maxima stay ambiguous. Transfer only selected input. | call / Function Pattern |
| CSF-T08 | RHS requests self, captures, ...captures or lifetime pre/post | Reject those request positions. RHS has explicit input/output/type/Policy only, with no alternate capture/lifecycle selector, constructor or spread. | Function Pattern |
| CSF-T09 | RHS root immediate rank+1 type/Policy holes versus LHS roots or deep holes | RHS is not a complete request. LHS may deduce ordinarily; deep legal holes retain identity/availability. Preserve arbitrary-rank coherence and structural _ versus external _$. | Function Pattern |
| CSF-T10 | LHS full input with one ...rest; two same-layer Packs; RHS ...args | Match all meaningful capture/argument positions, allowing one contiguous ordered remainder. Reject uncovered captures, two same-layer Packs and RHS spread. Nested layers keep independent rules. | Function Pattern |
| CSF-T11 | LHS omits pre/post, actual A changes observed state | Keep U_default, exclusive distinct/shared common roots and default origins. Expected Post=NoDelta. At extraction require Pre_F implies Pre_A and Post_A implies Post_F; insufficient actual Post rejects immediately. A with no stronger Pre and a sufficient Post is compatible under mapped entry assumptions. | lifetime |
| CSF-T12 | RHS has no lifetime clause; uniquely selected A fails actual Pre/commit/Post | Fail without publication or runner-up selection. Clause omission skips no actual check and supplies no new lifetime demand. | lifetime / call |
| CSF-T13 | CallType(tau) versus CallValue(x) | Keep CallTarget=tau, ActualSelf=c* versus both x. CallableOwner, ReceiverType, Param and tag/activation ReturnFrame remain independent with one selected identity. | call |
| CSF-T14 | Pass self observed through capture Param | One coherent transaction, all affected Pre before existing atomic commit, joint Post. No duplicate whole-self/capture Kill, implicit Copy or reuse of killed generation. Preserve invokes no clone. | call / lifetime |
| CSF-T15 | Param contains ref/share handles | Move handles with existing target/generation/origin. No direct referent ownership transfer, promotion or region extension. | call / lifetime |
| CSF-T16 | Recursive (args) |> self, same tag with nested activations | Enter fresh activation through CallAnchor with actual Movable/borrow facts. Complete resolves tag plus active activation, respecting barriers/cleanup; no old sealed call reopens. Continue/break-like paths need no new primitive. | call / return |
| CSF-T17 | Mandatory source-authority-erasing Function Pattern projection, later compile-known Val1 | Keep formed implementation/dependencies/contract and opaque internal provenance; necessarily remove external structural navigation/reselection authority, with no opt-out or recovery from known Val1. O specialization stays within interface. Before/after full Type equality is not required; ordinary complete-tau transport still retains V_tau. | Function Pattern |
| CSF-T18 | Library reify forwarder changes lowest present external compile demand to runtime | Require ordinary same-Type migration, active ClosedReturn, readiness/dependency/lifetime evidence, and independent output migration if needed. No built-in Function Pattern conversion or restored source. | Function Pattern / Policy |
| CSF-T19 | Same callable/result in let v=expr versus v=expr, including uninitialized designated target | E selects ABI1 versus ABI2. Alias/initialization/replacement authority remains checked. RVO/NRVO, result-type equality and direct versus temporary construction cannot switch ABIKind. | E delivery |
| CSF-T20 | Tail call under ABI1 versus recursion lacking cleanup/lifetime/slot proof | Consume proved frontier reuse, preserving delivery even for different input/output sizes. Missing evidence grants no unconditional tco/loop or constant stack; physical realization remains deferred. | E delivery / mechanical |
| CSF-T21 | Copy/further pure extraction of formed Function Pattern value | Keep actual captured realizations without initialization/relookup. Without conversion, normalized full contents determine equality; changed owned content differs. NameBinding/Place identity remains separate; no fourth coordinate or hidden state. | dependency / Function Pattern |
| CSF-T22 | Needed classifier, implicit material, rank or contract implication unknown | Keep Unavailable and pending obligations, not Mismatch, fallback suppression, unique maximum or permission. Independently proved hard rejection may still terminate its candidate. | Pattern / call |
| CSF-T23 | Active Ordinary F, then active Ordinary G formed within F | Omitted return in G selects G, not F. F remains active but lies outside G's reachable return segment. Binding slice covered by return_target unit tests; execution pending. | return |
| CSF-T24 | Active Ordinary F, nested InPlace I/J | InPlace adds no barrier: omitted return reaches F; explicit target I/J or F is allowed only with actual activation and ordinary legality evidence. Binding slice covered; completion pending. | return |
| CSF-T25 | G captures F's target/capability, then G executes while F remains active | Explicit syntax and ordinary capability invocation both reject return to F across G's Ordinary barrier. Active identity/possession is insufficient; helper entry cannot reset the requesting segment. Binder resolver/shared guard covered; actual capability execution pending. | return |
| CSF-T26 | Unknown boundary before the prospective outer target, or nearer proved Ordinary barrier | Missing required evidence preserves omitted/explicit targets unbound, not a transparent boundary. A nearer proved Ordinary barrier closes its own segment regardless of unknown outer boundaries. Popping/reentering uses distinct activation identities. Binding guard covered. | return |
| CSF-T27 | Formed closure with no captures, or two captures of types unrelated to ActualSelf | Param is respectively B_K() or B_K(a,b); neither needs a first capture of callee type. ActualSelf still meets the associated entry receiver contract. A real T receiver cannot use a T ref entry by coercion. | call / type |
| CSF-T28 | InternalOnlyPath(tau), nonidentity compile-to-runtime candidate m, prospective world proves nonrecoverability | Reject m at hard A, before preference; do not defer to DynamicLegality/backend/reify. Equivalent Ordinary/InPlace material has the same outcome; no blanket value-operation ban. | Policy / dependency |
| CSF-T29 | Prospective runtime world proves all retained structural Paths recoverable | Continue ordinary hard A and selection; proof alone grants no readiness, lifetime or final delivery success. Compile classifier observations of runtime Objects may be legal. Candidate-specific evidence requires no speculative destination construction or body execution. | Policy / dependency |
| CSF-T30 | Prospective Path evidence unknown while another migration candidate is apparently viable | Applicability Unavailable, not false; no wildcard/runner-up selection by treating missing proof as rejection. Source-world Path success or equal Core is insufficient. | Policy / dependency |
| CSF-T31 | Existing accepted view versus nonidentity migration; projected C:tau_C -> f:F versus complete tau transport | Reuse the existing view first with unchanged identity and established obligations; enumerate no migration. Function Pattern projection necessarily erases external source authority and is not same-Type migration; complete tau transport retains immutable V_tau. No erasure bypass of retained-Path A. | Policy / Function Pattern |
| CSF-T32 | Projected Function Pattern value at compile/runtime frontier with internally retained source IDs | External consumers cannot re-navigate or reselect original structural candidates even if Val1 is compile-known. O specialization stays within exposed contract; opaque implementation provenance grants no authority. | Function Pattern |
| CSF-T33 | Arrowless () or (A,B), even under callable expectation or Pin annotations | Remain Bare Product material, not Function Pattern syntax. A consumer explicitly requiring Function Pattern syntax must diagnose a missing ->, not infer the boundary by semantic lookup. An arrow nested in a child does not supply its parent's boundary. | Function Pattern / surface |
| CSF-T34 | LHS and RHS Function Pattern syntax with () -> R or (A,B) -> R | Retain a written -> delimiting the independent output/Pout position; Pin cannot supply Pout. Ordinary output deduction does not remove the delimiter. Positive and missing-arrow grammar consumers remain pending. | Function Pattern / surface |
| CSF-T35 | Ordinary/in-place closure expression with no written return clause | Existing legal closure syntax remains legal; do not apply FunctionPatternSyntax => HasArrowBoundary to every callable expression or treat its generic head as Function Pattern recognition. | Function Pattern / surface |

T27–T35 are normative targets, not executed capture/Path/Function Pattern tests.

Additional authority subcases keep ordinary determination and delivery ahead of
their consumers:

- **CSF-T11 concrete-callable boundary:** Present an undetermined overload
  collection with two members, only one of whose lifetime contracts satisfies
  F_L. LHS extraction cannot select that member by contract compatibility;
  ordinary concrete-value determination or its ambiguity/unavailable result
  must stand first. Once ordinary determination fixes a member, check only that
  member's contract. Failure cannot reopen the collection for the compatible one.
- **CSF-T19 delivery-target boundary:** For a plain expression or nested return
  whose ordinary E delivery target is unavailable, keep delivery unavailable
  before publishing a result through that consumer. O cannot supply the missing
  target through a hidden lexical temporary, RVO/NRVO or a third ABI selection
  rule. Once E establishes a delivery target, apply only its defined convention.

Cross-owner proof obligations are source/realization preservation (T01–05,T21),
selection-to-projection direction and uniqueness (T06–09,T13,T22), lifecycle
conservation in one transaction (T12,T14–15), self/return independence (T13,T16),
directional contract preservation (T08–11,T17–18,T21), and ABI uniqueness
(T19–20). They use one R_Gamma/Policy/overload/Move/lifetime/E system; these
specification judgments imply no executed source success.

## Empty invocation and default-formal sugar acceptance

Status: DS01–DS28 are normative acceptance targets. The empty-Product syntax,
shape, canonical identity and ordinary arity-selection slices of DS02–DS03 have
executable regressions; do/default parsing and family execution remain pending.
Canonical ownership: [call family/forwarding](../design/symbol-world/function-object-call-model.md#12-trailing-default-formals-form-ordinary-overload-members),
[empty Product](../design/patterns-overload/product-repetition-and-unpacking.md#2-bare-product-and-context),
[contribution formation](../design/symbol-world/names-and-overload-groups.md#61-common-snapshot-closure-contribution-formation),
and [surface/Raw handoff](../public/normalized-surface-semantics.md#15-canonical-call-sugars-consumer-pending).
The `31_empty_products` Norm golden, syntax `empty_products` tests and build
`product_shape`/`canonical_integration` regressions cover empty delimiters versus
comma Units, neutral continuation, nested exposure/barriers, zero-versus-one
canonical input identity and ordinary fixed-arity selection. Existing sealed-call
tests cover substrate no-reopen, not execution of do/default sugars. The [roadmap](roadmap.md#8-acceptance-evidence) records the open consumers.
DS24–DS28 specify the [non-semantic forwarding warning](../design/symbol-world/function-object-call-model.md#1221-non-semantic-forwarding-warning)
and its invariance tests, not implemented diagnostic coverage. Warning emission
is a non-blocking follow-up; DS22 remains an intentional legal outcome.

| Case | Setup / competing derivation | Required result / rejection boundary |
|---|---|---|
| DS01 | do E, where forming E records an effect and yields a callable | Form E once; use ordinary lookup/selection with zero explicit arguments. No second evaluation to obtain self or inspect candidates. Compare with exactly the ordinary () pipe skeleton. |
| DS02 | Empty () versus a present Unit comma slot; zero- and one-argument candidates | Empty has arity 0 and no ordinal child; present Unit counts. Canonical input keys and observations follow actual empty versus present material. No downstream Unit-stripping or fabricated self. |
| DS03 | x pipe f (), () pipe f (x), obj..method(), obj pipe .method(), obj[], obj[()] | Empty material merges neutrally; both method spellings and both exposed index inputs have receiver only. Keep first-product-only binding and source nesting. Actual comma-hole Units remain arguments. |
| DS04 | do f (x) versus do (f (x)); nested Product/barrier or unavailable runtime material | Preserve the existing pipe skeleton and explicit grouping, not a new conventional-call grammar or target barrier. Only ordinary available exposure may change arity. |
| DS05 | do around a compile/runtime/in-place target and inside a return/serial context | No extra stage, lexical owner, dependency scope, return target or control event. Retain ordinary unit discard/fallthrough, implicit event and explicit targeted return rules. |
| DS06 | n explicit formals and k trailing defaults; k=0, k=n, and a self-only head | Form exactly k+1 distinct fixed-arity candidates n through n-k. Retain contextual self Pattern/binder/Policy in every head; self/captures do not count in n. (self, let i=d) produces (self, let i) and (self); separating self from the latter yields unit/empty explicit input, not one Unit argument. Complete internal capture/self input is a separate observation. |
| DS07 | Required-after-default; default on first contextual formal named self, receiver or _; nested/Pack/return default; missing E | Spanned syntax/validation recovery. A contextual-self default is always rejected, never evaluated or used to synthesize/drop self. Apply this by position, not spelling: an ordinary later explicit binder named self is not the contextual position. No parser lookup/backtracking or capture-initializer repair. A whole direct destructuring formal may have a default. |
| DS08 | Generated-family-only call with two omitted defaults and effects; full-arity call with explicit values | Selected shorter entry executes each omitted default once, left to right along fresh arity-growing ordinary calls. Full-arity entry executes none. Formation and preselection execute no default expression; each intermediate call still needs ordinary selection and legality. |
| DS09 | Later default observes prior supplied/defaulted arg or contextual self; default reads current/later arg | Earlier readable bindings use ordinary views. Closure-entry default self is c_m and original body self is c_n; direct associated entries retain the same receiver x in defaults and body. Current/later uninitialized value reads fail without outer-name fallback; classifier access needs its independent evidence. |
| DS10 | Default result mismatches Pattern/Policy or needs unavailable compile observation | Ordinary constrained production/extraction, horizon/readiness and migration rules apply. No default-time type coercion in applicability, runtime-to-compile permission, or execution to prove viability. |
| DS11 | Public name shadowed after formation; target space includes user overloads; direct associated receiver forwarding | Closure/contribution entries call their held complete-type target, not a re-resolved public name or their own c_m. Direct associated entries call the original receiver x. All ordinary target candidates compete: the next generated entry is not privately filtered or preselected; user winners and ambiguity stand. This fresh call cannot reopen its selected outer entry. |
| DS12 | User and generated fixed-arity candidates both applicable and otherwise tied | Ordinary ambiguity. No generated/original/default-count priority, fallback suppression from origin, or per-family preliminary winner. Distinct members do not collapse merely because outputs agree. |
| DS13 | Selected shorter entry default fails, ordinary forwarding call fails, or return delivery fails; outer runner-up exists | No outer or already sealed inner selection reopens. Preserve reached effects and ordinary cleanup/failure transaction boundaries; publish no failed result. |
| DS14 | Source captures have effectful initializers; defaults and body use those captures | Source initializers execute once per formation, not per member/call; defaults execute per selected invocation. Defaults participate in exclusive Explicit/Automatic dependency completeness. No invocation-time relookup or recapture. |
| DS15 | Owned noncopy capture or local ref needed by several wrappers/escaped family | Prove ordinary realization, retention, capability, generation/lifetime and escape legality for every required use. No implicit shared environment, clone, Copy or global lifetime; no blanket ban based only on local provenance. Missing evidence remains unavailable. |
| DS16 | Default consumes an earlier arg still needed for forwarding; ref/share or Preserve forwarding | Require actual frontier legality and one terminal Move per edge/input. Consumed prior generation is not restored; no duplicate same-frontier Kill. Ref/share retains targets; Preserve is not clone. Original associated receiver still needs its own legal call/transfer evidence. |
| DS17 | Forwarded result is unit/non-unit; outer source frame active; direct associated entries share receiver tag x | Terminal return explicitly targets that entry's actual activation, including unit and nested activations with the same tag x. Propagate ReturnPattern/Pout before root maxima and preserve source-written return targets. No unit fallthrough, omitted-target substitution, hidden lexical temp or merging by receiver/tag identity. |
| DS18 | Defaulted declarations in a common bucket, a local defaulted closure, and direct let () implementation material | Bucket/local formation retains complete-type homes, one snapshot and no first sibling/partial Object/repeated lexical aggregation/bulk import. Direct let () instead installs real associated entries Impl_n through Impl_(n-k), each with ActualSelf=x for invoked x:T. Its self-only entry matches unit explicit input and forwards by ordinary call to x. No anonymous receiver substitution, generated V_T registration or new authority. |
| DS19 | Example do (self, let i = 1uint8) => { cond ? if { i } else { (i + 1) pipe self } } | The generated (self) head matches unit explicit input. Empty call may select c_0; default once, then ordinary family call with one explicit value. With only these generated entries, c_1 wins. Body self=c_1 recurses at full arity without rerunning the default. Unknown cond/type/Move/lifetime facts grant no success or tco guarantee. |
| DS20 | Forwarders under compile/reify, output-demand competition and copying a completed family | Ordinary compile-instance keys, ClosedReturn, E-selected ABI, actual dependency identity, no recapture and immutable complete observations apply. Expansion grants no stage conversion, ABI choice, renewed construction window or SCC/slot-reuse proof. |
| DS21 | G={c_2,c_1,c_0}, defaults d_1,d_2; selected c_0 forwards one argument, applicable h strictly loses to c_1 | Normal inner selection chooses c_1, then d_2 executes once. No generated-member preference; the next-member outcome follows only from ordinary preference. |
| DS22 | Same selected c_0 and input, but h strictly wins over c_1 | c_0 forwards to h. Do not force c_1/c_2 or evaluate d_2. Outer selection stays sealed; whole-target competition is intentional. |
| DS23 | Same selected c_0 and input, h and c_1 tied distinct inner maxima | Ordinary inner ambiguity before either body/d_2 executes. Reached d_1 follows ordinary effect/failure/cleanup rules; no retry of outer candidates or private next-member filter. |
| DS24 | DS22 with proof of the selected outer c_0, inner winner h and h != c_1 | May emit Warning: default-argument forwarding selects a different overload member. Preserve the legal h winner, skipped d_2 and sealed outer selection; no default-specific Error. |
| DS25 | Inner winner or its comparison with the nominal next entry is not provable at the diagnostic frontier | No deviation warning from unknown evidence. Do not execute defaults/bodies or force selection for diagnosis. A permitted later ordinary selection remains later; missing evidence required by the ordinary call still yields its ordinary unavailable/failure result, never assumed success. |
| DS26 | Same ordinary forwarding with the warning enabled/omitted; compare a next-entry winner with a proved distinct h whose output is equal | Same candidate set, applicability/preference, winner, result, effect counts, Move/lifecycle and delivery. No deviation warning for the nominal next identity; equal output does not erase a proved distinct candidate identity. Diagnostic emission grants no execution or selection authority. |
| DS27 | Forwarding ambiguity, missing required evidence or failure after an inner seal; outer runner-up exists | Preserve ordinary failure and reached effects/cleanup, never reopen an outer/inner seal. No deviation warning claims a winner where none is proved; a proved selected h may be diagnosed without recovering from its later failure. |
| DS28 | Explicit user candidate controls or call-site constraints on a forwarding call | Apply only the written constraints under ordinary rules. Expansion injects no filter; constraint failure is ordinary failure, not a default-specific Error. Neither warning emission nor its absence bypasses the constraint or retries selection. |

Two explicit boundary subcases refine the matrix:

- **DS06 self-only versus Unit argument:** Compare `(self, let i=1uint8)`'s
  generated `(self)` head with `(receiver)` and a legal supplied Unit slot.
  Both self-only heads have unit explicit input even if their full internal
  inputs contain captures/receiver material. A present Unit argument is still
  one explicit position and must not be stripped to make it match.
- **DS18 direct call-entry forwarding:** For `let () = (receiver, let i=d) => body`
  under owner T, calling an existing x:T with empty explicit input may select
  Impl_0. It evaluates d, then ordinary `(result(d)) |> x` may select Impl_1
  with self=x and a distinct activation. Do not default/create x, substitute
  a generated c_0/c_1 receiver, privately force Impl_1, or register V_T members.
  A written default on receiver is rejected, even if it could produce a T.

For every executable connection, include positive and negative syntax goldens,
empty-versus-present identity/arity, effect counters, generated/user ambiguity,
selected failure with a runner-up, dependency/authority non-derivability and
explicit wrapper-unit-return regressions. A passing existing test suite alone
does not satisfy these unconnected normative rows.

## Lifetime value computation and library design acceptance

Status: normative design scenarios, **not executed source tests or completed
proofs**. All rows use the [lifetime owner](../design/lifetime/lifetime-policy-and-overload-boundary.md)
and [consumer interfaces](../contracts/lifetime-program-consumers.md).
W1–W4 below refer to the four [walkthroughs](../design/lifetime/lifetime-algebra-walkthroughs.md),
not test fixtures. Each row gives an admitted case and the forbidden inference
or negative case. Ordinary missing premises remain unavailable; a known false
premise rejects. There is no general program-termination/contract-implication
decider assumed by this matrix.

| Case | Input / action | Required result and counterexample | Handoff / witness |
|---|---|---|---|
| LA01 | Project the same selected action/candidate/frame with only business Stage labels varied | Same lifecycle definition/correspondence. Actually different selected overloads may have different Post; erasure never selects again | LC2; W1 |
| LA02 | A directly written pre/post program uses an established runtime name whose business value is unknown | Evaluate ordinary lifecycle computation once, without @@ or a second partner. Do not read the unknown value, prune future paths or commit both branches | LC2/LC4; W1 |
| LA03 | Form structure with actual registered selected field operations | Paired lifecycle fields satisfy `(x.field*)@ = x@.field_lifetime*`. Missing partner/registration is unavailable, not a special resolver; `$` itself adds no action | LC1/LC8; W1 |
| LA04 | Observe equal literals/temporaries and two bindings/Places with equal residents | Temporaries can have LifeNames without Places; NameBinding, Place, target and generation remain distinct. Equal content or type does not merge identities/credentials | LC1/LC3; W1/W3 |
| LA05 | Implement select_one with two legitimate guarded source returns and omitted explicit Post | Compress real material into the source-choice Post plus all other outward effects. `r1 || r2` is one correlated source choice, never simultaneous exclusive permission | LC4/LC5; W2 |
| LA06 | Extract into a handwritten Function Pattern with omitted Post | A genuinely NoDelta implementation can match; select_one's non-NoDelta implementation cannot. A sufficient explicit compatible contract can match. Known Val1/provenance cannot restore erased structure | LC6; W2 |
| LA07 | Evaluate a contract helper containing local Products, branching, calls and justified recursion | Intermediates need not be NameExprs; final direct self-root complete type and associated behavior obey real instance/OpenHere/result rules. OpenHere alone grants no world-write authority | LC4/LC6; W2 and owner §9 |
| LA08 | Caller Pre fails, while the body could admit an assumption; compare successful four-way use | Fail before the action/body. Successful path uses caller check/body admit/body check/caller admit with actual Param/output/activation mapping, without body replay or double transfer | LC3/LC4; W1/W3 |
| LA09 | Nested block exits and target return cross two actual InPlace boundaries | Compress at each real exit and preserve target/cleanup. No local unit, phantom result binding, extra self argument or hidden result Policy completion | LC3/LC5; W1/W2 |
| LA10 | Traverse origin links through locals to U=unknown; compare explicit None and missing evidence | Retain U as unknown, stop at genuine None, and report unavailable missing material. Caller can continue without reading dead locals; no manufactured endpoint | LC5; W2 |
| LA11 | Export a result truly borrowing an object that dies at the crossed boundary | Reject under ordinary escape/validity rules. Compression of its local carrier cannot lengthen referent lifetime or make the return safe | LC5; W2 |
| LA12 | Compare sibling handle transfer, real child borrow and ordinary logical dependency | Preserve their actual distinct origin/target/region material; no automatic reborrow or uniform internal from call arranges a preferred tree | LC3/LC5; W2/W3 |
| LA13 | Kill Move, narrow proved Preserve and whole-self/Param transfer | Kill ends its old generation with no duplicate destructor; Preserve does not clone. One coherent implicit-input transfer, no whole-self plus captures double Kill or failed-Move copy retry | LC3; W1 |
| LA14 | Represent a recursive field-access family with finite regular choice/advance/repetition states | Finite descriptors retain concrete target/name/generation parameters. State reuse cannot merge identities; finiteness proves neither arbitrary program termination nor TCO | LC8/LC6; W1 and owner §11/§14 |
| LA15 | Crop access to leaf q using a shared routing prefix p | Output operation ranges are a subset. Keep p as route without granting whole-p terminal access; cropping grants no new permission and removes no historical color | LC8; W3 |
| LA16 | Compare ref-only tree path, a path reaching share, and an alias path | Registered ref-only access can proceed under Pre; ref cannot cross share to recover exclusivity. Share through ref remains share without alias write/internal mutability. Alias has no normal propagation | LC1/LC8; W3/W4 |
| LA17 | Successful try_trust returns ordinary integer safeid; compare equal integer at another name | Post establishes source/generation/interval credential without changing type or requiring ref/share. Equal numeric content alone has no credential; `?` performs only ordinary extraction/control | LC7; W3 |
| LA18 | Attempt an access after length/layout/capacity/resource generation or index-name change invalidates validation | Library protocol covers every invalidator. Old credential is unusable, regardless of field spelling. No failed permission followed by a hidden numeric check or runner-up | LC7; W3 |
| LA19 | Validation succeeds versus fails; ordinary action has a missing/false Pre | Only actual success admits compatible unsafe Post facts. Failure has no success token; Post cannot repair prior Pre, and there is no unsafe pre escape | LC4/LC7; W3 |
| LA20 | Maintain credential interval, transfer an admitted view, then end the interval | Conflicting operations are blocked while it is valid; an actually authorized view preserves mapped credential obligations. After interval end and all remaining Pre, required exclusivity may resume; history is retained | LC3/LC7; W3 |
| LA21 | Batch consume shares with genuine pairwise disjoint operation ranges and a common authorized original ref R | End the consumed share intervals and form complete exclusive children from R once. R may be recovered through real source authority without a mandated explicit argument; share itself is not strengthened | LC3/LC7; W3 |
| LA22 | Batch inputs have equal ranges, ancestor-whole/descendant overlap, or unequal path strings | Reject actual overlap even when strings differ. A shared routing prefix alone need not overlap terminal operation ranges. Dynamic validator targets access semantics, not pointer integers | LC7/LC8; W3 |
| LA23 | Batch consumes listed shares while another conflicting dependency remains alive | The other dependency still blocks exclusivity; consumption cannot erase it or old coloring. Failed Pre/validation publishes no refs and cannot double-consume inputs | LC3/LC7; W3 |
| LA24 | Batch has one input, with ordinary source/validity/exclusion Pre satisfied or failed | No pairwise partition runtime validation is needed. All other Pre still apply; degenerate arity is no blanket success | LC7; W3 |
| LA25 | Alias root replacement/extraction has actual defaults; compare missing defaults and traversal | Registered root operation may proceed under its Pre. Shape alone supplies no default; alias cannot cross into ref/share or inherit their tree. Lexical Path aliases are unrelated | LC7/LC8; W4 |
| LA26 | Open a valid alias-to-ref callback window versus an overlapping/invalid or escaping one | Facility maintains actual validity/exclusion and blocks conflicting alias actions for the entire window. Fresh pairwise distinct anonymous roots are not alias origins and prove no window legality/disjointness | LC7; W4 |
| LA27 | Consume partbox shares of the same resource with quantities compile-known or runtime-known | split/merge consume inputs and conserve quantity on actual successful transfers. Equal quantities of different resources cannot merge; copying a number does not duplicate a right | LC3/LC7; W4 |
| LA28 | Convert nonfull/full partbox and update option topology links while using payload | Nonfull yields only restricted alias; full still requires every other Pre before consuming to box. Option extraction/replacement is ordinary complete-object action; topology/payload authority are distinct, not a general graph/reclamation theorem | LC3/LC7; W4 |
| LA29 | Crop/compress/admit external facts or rewrite continuation with O | No operation retracts committed history/color. Old intervals may end legally; new exclusive intervals do not recolor the past. O revalidates affected projections at rewritten names/cuts | LC2/LC3/LC5/LC7/LC8; all |
| LA30 | Any needed source producer, recursive evidence or materializer facility is unconnected | Report its unavailable diagnostic, or transport only an actual established remaining continuation. No default permission, invented residual, runner-up, ABI switch or independent checker/string test | LC1–LC8; all |

For LA07, the contract helper's recursion needs its own justified semantic
domain; a finite regular path is insufficient. For LA05/LA09, compressing arms
before combining is valid only under the guarded family/interface and retained
effect conditions in W2, not a universal join/exchange law. LA19/LA21/LA26 need
true trusted-facility facts and actual runtime behavior. These remain explicit
[proof obligations](../design/lifetime/lifetime-policy-and-overload-boundary.md#14-proof-obligations-and-engineering-freedom).

Current tests cover syntax carriers, supplied-fact lifecycle relations and the
common transaction's cleanup/Pre/joint-publication slices. They do not close any
row's full source chain. Future connections must add meaningful action-based
positive/negative, identity/equality, non-derivability, authority-uniqueness and
no-reopen tests at their connected boundary, with syntax goldens only for actual
grammar changes. Passing repository tests or matching document strings is not
execution coverage of these design scenarios.
