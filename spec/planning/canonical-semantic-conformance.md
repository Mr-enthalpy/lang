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
| 106-DP11 | Ordinary => closure legally observes external x without an explicit clause for x | Form an AutomaticDeps occurrence for x using the selected ordinary action; automatic dependency does not imply InPlace. | [DP owner](../design/symbol-world/dependency-observation-and-realization.md#4-explicit-and-automatic-dependency-formation) | Defined semantics; source/evaluator consumer pending |
| 106-DP12 | Ordinary closure has explicit capture x and another free external y | Combine explicit x and automatic y occurrences. A body read resolved to capture x does not also automatically capture outer x; other distinct occurrences/binders are not deduplicated by spelling or value. Formation origin adds no post-formation operation dimension. | [DP owner](../design/symbol-world/dependency-observation-and-realization.md#4-explicit-and-automatic-dependency-formation) | Defined semantics; source/evaluator consumer pending |

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
| ALIGN-11 | Name-headed in-place callable in ordinary and type calls | Formal head retains (_, let _: (name)$). Selected ordinary x or projected c reaches slot zero; the explicit actual never supplies self. | [Owner](../design/patterns-overload/pattern-relations-observation-and-extraction.md) |
| ALIGN-12 | E return versus plain E at a path tail | The former already has ReturnEvent and infers only target; the latter first uses the serial consumer. Do not merge these operations in Omitted. | [Owner](../design/control-flow/targeted-return-and-d-reduction.md) |
| ALIGN-13 | A non-tail serial expression yields non-unit | UnitDiscard rejects; it does not silently discard the result, synthesize a lexical temp or turn it into return. | [Owner](../design/control-flow/targeted-return-and-d-reduction.md) |
| ALIGN-14 | Tail unit versus explicit unit_value return | Tail unit falls through; explicit unit produces ReturnEvent. Neither fabricates a local return contribution. | [Owner](../design/control-flow/targeted-return-and-d-reduction.md) |
| ALIGN-15 | Each branch has a true non-unit tail, or a following continuation | Determine tailness per control-flow path, not AST list position. Synthesize return only at the true tail, then infer the outermost function target. | [Owner](../design/control-flow/targeted-return-and-d-reduction.md) |
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
| R01 | type call | CallTarget=tau, ActualSelf=c*, actual slot zero unchanged | [Call](../design/symbol-world/function-object-call-model.md#callable-tag-and-activation-observations) | selection connected; tag consumer pending |
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

## Implementation handoff checklist

Statuses describe the stated boundary only. A connected syntax or rejection
boundary does not establish successful execution of the corresponding language
feature. Canonical owner definitions above are authoritative for both statuses.

| Item | Status | Evidence / next consumer |
|---|---|---|
| Stage={compile,runtime}, independent open/close and mode | 规范已闭合、实现已连接 | policy_pair_semantics and stage_observation_boundary |
| Adjacent-rank joint extraction, ordinary values through higher universes | 规范已闭合、消费者待连接 | J01–J03; classifier-side binders and Type(type) observation report unavailable before publication |
| Explicit external classifier constraint in the connected literal/type subset | 规范已闭合、实现已连接 | relational_pattern_authority, world initializer boundary tests |
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
| RF08 | runtime:runtime->runtime:compile | classifier-only migration, value Policy preserved | source/continuation consumer pending |
| RF09 | pure runtime-policy type | equal runtime observations; no runtime type-algebra permission | observation substrate; type consumer pending |
| RF10 | actual dependent type work plus independent compile type work | defer dependent cone only; independent work remains ready | dependency-cone consumer pending |
| RF11 | runtime value-only compile detour | O may eliminate only with E-proved no type dependency and affected E revalidation | O consumer pending |
| RF12 | Runtime/reify P2 with explicit compile P1 or one Runtime output among many | incompatible producer constraint fails; all outputs Runtime | P1 check; joint output consumer pending |
| RF13 | Runtime/reify Pin with compile input | latest availability may accept it; Ready and applicability remain independent | InputAdmissible consumer pending |
| RF14 | deleted migration, host asymmetry, restricted atomic library type | no generated capability may override missing/deleted realization | ordinary candidate/capability substrate |
| RF15 | existing accepted view / selected migration failure | identity only / terminal failure, no chain or reopen | shared selection substrate |
| RF16 | runtime newly formed multi-dispatch call | first ordinary selection at actual frontier; selected predecessors retain origin | reified E consumer pending |
| RF17 | future false branch and lifecycle facts | current compile name/Color Region only; path elimination uses E O E | lifecycle substrate; source/O consumers pending |
