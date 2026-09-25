# Canonical semantic conformance matrix

Status: canonical acceptance scenarios; evaluator/source consumers pending.
These are semantic counterexamples and equalities, not claims that the current
parser supports every displayed spelling or that executable tests already
cover them. The topic owners define meaning; this matrix indexes their checks.
The 64 PR105 IDs are retained, with N01/N03 clarified by PR106. The review
revision distinguishes ordinary/type calls and replaces in-place permission
restrictions with automatic dependency formation; the existing CL/PT/AD/LF
case IDs record the corrected relations. Type identity is independent of
self-construction; type calls select once over all callable/implementation pairs,
and dependency realization is fixed by the selected ordinary source action.
The 132 PR106 scenarios bring this index to 196 cases, including independent
callable construction axes, closure contribution consumers and two-level Path
observations. These are semantic acceptance obligations, not 196 executed tests.

| Case group | Canonical owners | Implementation gate |
|---|---|---|
| S01–S12 | [Policy](../design/symbol-world/symbol-policy-and-compile-flow-projection.md), [calls](../design/symbol-world/function-object-call-model.md), [selection](../design/patterns-overload/overload-resolution-design.md) | Single-stage positions; two rounds and sealed origin |
| E01–E09 | [Evaluation](../design/meta-invocation/evaluation-residual-and-optimization.md), [meta invocation](../design/meta-invocation/meta-object-invocation-and-policy-reduction.md), [build](../design/build-package/build-system-design.md) | Runtime entry, dominance and readiness |
| L01–L08, W01–W08 | [Lifecycle](../design/lifetime/lifetime-policy-and-overload-boundary.md), [mechanical placement](../design/mechanical-lowering/mechanical-argument-passing-and-move-fixed-point.md) | Instance effects and cleanup |
| P01–P12 | [Residual/completion](../design/patterns-overload/static-pattern-spaces-and-extraction-chains.md), [target return](../design/control-flow/targeted-return-and-d-reduction.md), [meta state](../design/meta-invocation/meta-object-invocation-and-policy-reduction.md) | Restricted Split, escape and current meta facts |
| O01–O07 | [Operator families](../design/patterns-overload/operator-patterns-and-generative-declarations.md) | Operator roles and OG_s |
| N01–N08 | [Names](../design/symbol-world/names-and-overload-groups.md), [construction](../design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md), [source composition](../design/symbol-world/symbol-construction-units-and-namespace-origin.md) | Expression formation and conservative contributions |

Every row currently requires consumer alignment; see the
[implementation evidence and gates](roadmap.md#canonical-semantic-revision-implementation-gates).
Existing carrier test success is not coverage of these new semantics.

## Stage, Policy and two-round calls

| ID | Scenario | Required result |
|---|---|---|
| S01 | P2=runtime with omitted P1 stage | Complete to runtime, without a runtime/compile union. |
| S02 | P2=seal with omitted P1 stage | Complete to seal, without a composite stage. |
| S03 | Bare let in a compile binding context | Complete the stage from context; leave no hole for a later runtime consumer. |
| S04 | A runtime callable has an explicitly compile Pin | Apply ordinary input admission; the whole call need not be pure. |
| S05 | An explicit formal-stage hole | Extract from actual/position rules; distinguish it from omission. |
| S06 | A runtime-policy value happens to be known | E does not promote it to compile. |
| S07 | A compile callable accepts a seal actual | The call may defer; there is no seal-to-compile migration. |
| S08 | A static type has no runtime realization | Target demand does not manufacture a runtime slice. |
| S09 | Round one retains two stage derivations | Do not run both observable bodies; round two selects or reports ambiguity. |
| S10 | A selected compile realization has runtime residue | Preserve the selected origin; runtime does not reselect. |
| S11 | Pre or coherence fails after migration selection | Terminal failure, without a runner-up or chained repair. |
| S12 | An external demand constrains Pout stage | The consumer cannot overwrite established P1/Pout producer facts. |

## Main, Seal and static dominance

| ID | Scenario | Required result |
|---|---|---|
| E01 | Program entry at main | P2 is runtime; meta root history creates no permanent MetaDom. |
| E02 | Meta invokes a compile helper reaching a seal candidate | The seal candidate is invisible. |
| E03 | Seal invokes a compile helper reaching meta invocation | Meta invocation is invisible. |
| E04 | A meta invocation returns to main | Do not retain MetaDom over unrelated later calls. |
| E05 | Seal reads a completed meta payload | Use ordinary access rules; this is not a new meta call. |
| E06 | SealStatic first needs an absent default meta instance | A compiler trait query cannot bypass the invocation restriction. |
| E07 | A seal-dependent action is ordered relative to a write | Deferral preserves the dependency and cannot cross the write arbitrarily. |
| E08 | An internal seal:seal view must provide runtime:seal | Succeed only through an actual legal migration. |
| E09 | Compilation cache reuse | Neither reopen a closed subject nor retain permanent authority. |

## Instance lifecycle and with

| ID | Scenario | Required result |
|---|---|---|
| L01 | A killing move transfers a meta-local type instance | End the source generation at that cut. |
| L02 | A global type resident and an equal local copy | Value equality does not share Killable or lifetime. |
| L03 | A legal preserving move observes a stable meta instance | Do not kill the meta root; this does not make all global resources copyable. |
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

## Patterns, completion and meta queries

| ID | Scenario | Required result |
|---|---|---|
| P01 | An else residual remains after an if arm | It is legal inside the chain and may reach the following else arm. |
| P02 | Else tries to escape a forbidden boundary | The fixed consumer rejects using ordinary meta facts. |
| P03 | An ordinary Pattern residual may escape | Retain its input material without implicit discard. |
| P04 | An arm has completed | Later sibling arms do not match its completed result again. |
| P05 | A user declares a type named Done | It remains an ordinary type, distinct from internal Done. |
| P06 | Nested chains complete | Consume each marker without observable Done nesting. |
| P07 | A selected extractor body fails | Do not treat failure as a sum miss or try another arm. |
| P08 | An expected result Pattern exists | Deliver through ordinary R_Gamma, without a private ControlResult. |
| P09 | A default meta trait instance is mutated while OpenHere holds | Write the actual instance; later queries read current committed state. |
| P10 | Only an outer snapshot copy of the meta result is changed | Do not mutate the retained meta instance. |
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
| N01 | A closure expression at any level; revised by PR106 | Legal completion produces full tau_C; distinguish file installation and lexical binding. See 106-CL01 and 106-NS01–04. |
| N02 | A legal same-name closure contribution is added later | Do not change the earlier RHS evaluation category retroactively. |
| N03 | Only let a=uint8; clarified by PR106 | The RHS is exactly tau_uint8; file installation and local binding neither wrap nor arbitrarily merge it. See 106-NS03–04. |
| N04 | Legal inner lexical shadowing | Do not reinterpret it as contribution to the outer same-name object. |
| N05 | OpenHere/Pre fails for a named contribution | Ergonomic repair cannot bypass the failure. |
| N06 | Binding and contribution effects address the same coordinate | Use established role and conflict rules; spelling alone cannot mix them. |
| N07 | Independent legal closure contributions in sibling files | Join through ordinary unordered contribution at one NameCoord; no first-file winner. |
| N08 | A sibling reads a value newly written by another sibling | File sorting cannot create otherwise absent dependency visibility. |


## PR106 acceptance cases

### 106.1 Product layer ordering


| ID | Scenario | Required result | Canonical owner | Consumer status |
|---|---|---|---|---|
| 106-PD01 | All direct entries are named, without a top Pattern name | The layer is unordered; no added top name or wildcard is needed. | [PD owner](../design/patterns-overload/pattern-values-relational-semantics-and-extraction.md) | Defined semantics; source/evaluator consumer pending |
| 106-PD02 | One bare entry is added to that layer | The whole layer becomes ordered; named entries do not form a local unordered region. | [PD owner](../design/patterns-overload/pattern-values-relational-semantics-and-extraction.md) | Defined semantics; source/evaluator consumer pending |
| 106-PD03 | An all-named child layer inside an ordered parent | The child decides independently and is not forced into the parent's order. | [PD owner](../design/patterns-overload/pattern-values-relational-semantics-and-extraction.md) | Defined semantics; source/evaluator consumer pending |
| 106-PD04 | Local variables a and b appear as bare Product values | Their variable spellings do not become structural names. | [PD owner](../design/patterns-overload/pattern-values-relational-semantics-and-extraction.md) | Defined semantics; source/evaluator consumer pending |
| 106-PD05 | An unordered structure must become a bare sequence | Extract by name, then explicitly assemble in the requested order; no default sorting. | [PD owner](../design/patterns-overload/pattern-values-relational-semantics-and-extraction.md) | Defined semantics; source/evaluator consumer pending |
| 106-PD06 | Explicit ? removes the top name | An all-named layer stays unordered; preserve real nesting. | [PD owner](../design/patterns-overload/pattern-values-relational-semantics-and-extraction.md) | Defined semantics; source/evaluator consumer pending |
| 106-PD07 | Unordered result structure formed through observable effects | Result unorderedness alone does not authorize effect reordering. | [PD owner](../design/patterns-overload/pattern-values-relational-semantics-and-extraction.md) | Defined semantics; source/evaluator consumer pending |
| 106-PD08 | A future ordinal operation observes bare entries | Ordinals do not create names; the complete public ordinal API remains open. | [PD owner](../design/patterns-overload/pattern-values-relational-semantics-and-extraction.md) | Defined semantics; source/evaluator consumer pending |

### 106.2 Path structure and reading


| ID | Scenario | Required result | Canonical owner | Consumer status |
|---|---|---|---|---|
| 106-PT01 | bool::a::path and bool:: have equal second-level resident observations | Eval_value may agree while Read_name and # differ; no bare Path equality is implied. | [PT owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT02 | Project a pure NameValue with no existing external target | Obtain legal structure without requiring final-target lookup. | [PT owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT03 | Equivalent internal Path constructions | Path projection observes normalized structure, not whitespace, parentheses or source positions. | [PT owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT04 | Three legal internally composed Path segments | Preserve associativity and the language's name::path direction. | [PT owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT05 | The endpoints of ::a and a:: | Retain the difference rather than erasing both to an undirected sequence. | [PT owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT06 | Ordinary extraction and validation of path_pattern | Expose nodes, links and endpoints. PathShaped rejects empty standalone/cyclic chains, multiple or nonterminal explicit roots, non-string names and incompatible endpoints; representation alone is insufficient. | [PT owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT07 | Construct Path material from a string | Create Names([s]) with RelativeSingleName=(Select,OpenRoot), without lookup or access authority. | [PT owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT08 | A string contains :: or parentheses | Do not automatically re-lex or parse it as more paths or arbitrary source. | [PT owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT09 | Project and reinject into a Path consumer | Recover equivalent internal structure within the legal domain. | [PT owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT10 | An inner binding shadows the textual root of a pure Path | Resolve at that external read and use the applicable inner root. | [PT owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT11 | A Path explicitly retains an ordinary value root | Preserve that material under ordinary value rules; a same-spelled name does not replace it. | [PT owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT12 | A Path explicitly retains a reference root | Preserve target/generation without lifetime extension or automatic retargeting. | [PT owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT13 | A Path is sealed into a selected invocation's runtime residue | Runtime does not resolve by spelling or select again. | [PT owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT14 | The target is invisible, unrealized or illegal | Structure formation grants no read authority; ordinary Read fails without reopening. | [PT owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |

### 106.3 General splice and strong Pattern contexts


| ID | Scenario | Required result | Canonical owner | Consumer status |
|---|---|---|---|---|
| 106-SP01 | Reuse an obtained Policy p with <> p$ let | Use p's value, not a new hole or spelling-only interpretation. | [SP owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-SP02 | runtime let | A concrete constraint with no new deduction binder, not an implicit <runtime>. | [SP owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-SP03 | Explicit <p> p versus omitted Policy | Keep HoleBinderId, omission and concrete material distinct. | [SP owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-SP04 | Splice material containing existing HoleRefs | Preserve actual PatternRoot/HoleBinderId rather than rebinding by spelling. | [SP owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-SP05 | Splice material with invalid scope or structure | Ordinary inapplicability/error; no alpha-renaming or alternate-interpretation fallback. | [SP owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-SP06 | A splice operand has observable evaluation | Evaluate the reached occurrence ordinarily, without repetition for projections. | [SP owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-SP07 | Required splice material is runtime-only or not ready | Do not fabricate static Pattern material; retain continuation and legality boundaries. | [SP owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-SP08 | General splice is defined | This does not establish arbitrary source quotation or universal #. | [SP owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |

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
| 106-NS06 | Direct struct and incremental extend have equal final structure | Observe equality under all required premises; retain no irrelevant formation history. | [NS owner](../design/symbol-world/symbol-construction-units-and-namespace-origin.md) | Defined semantics; source/evaluator consumer pending |
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
| 106-GN06 | Value/call and Pattern/name forms of one meta declaration | Preserve the established equivalence domain without extending it to arbitrary lexical let. | [GN owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md) | Defined semantics; source/evaluator consumer pending |
| 106-GN07 | let a=e and let _=e | Remain extraction/binding of an existing RHS, not generation. | [GN owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md) | Defined semantics; source/evaluator consumer pending |
| 106-GN08 | let <a> (c Pattern) a | Reach a's layer, then apply the same R_Gamma with compatible valuations. | [GN owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md) | Defined semantics; source/evaluator consumer pending |
| 106-GN09 | Intermediate extraction in an unordered layer | At most one whole intermediate extraction, which may contain several named fields. | [GN owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md) | Defined semantics; source/evaluator consumer pending |
| 106-GN10 | Multiple intermediate extractions in an ordered layer | Align in that layer's order without relaxing Pack/remainder multiplicity. | [GN owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md) | Defined semantics; source/evaluator consumer pending |

### 106.7 Policy observations and direct return


| ID | Scenario | Required result | Canonical owner | Consumer status |
|---|---|---|---|---|
| 106-RP01 | Terminal G(...) supplies F's result | F's ReturnPattern/Pout is G's immediate demand before maxima. | [RP owner](../design/symbol-world/symbol-policy-and-compile-flow-projection.md) | Defined semantics; source/evaluator consumer pending |
| 106-RP02 | Implementation uses SSA temporaries or registers | Add no language binding, Policy default or observable lifecycle. | [RP owner](../design/symbol-world/symbol-policy-and-compile-flow-projection.md) | Defined semantics; source/evaluator consumer pending |
| 106-RP03 | User explicitly writes let temp=G(...); temp; | Retain the real binding boundary; equivalence with direct forwarding is not required. | [RP owner](../design/symbol-world/symbol-policy-and-compile-flow-projection.md) | Defined semantics; source/evaluator consumer pending |
| 106-RP04 | Transparent F forwards to G | P1_F and P2_F jointly constrain G, without linear one-way propagation. | [RP owner](../design/symbol-world/symbol-policy-and-compile-flow-projection.md) | Defined semantics; source/evaluator consumer pending |
| 106-RP05 | Interpret the body after selecting F | F's established signature and valuation are known immediate context for G. | [RP owner](../design/symbol-world/symbol-policy-and-compile-flow-projection.md) | Defined semantics; source/evaluator consumer pending |
| 106-RP06 | Terminal H(G(...)) | Result demand first constrains H; do not inject unselected H formals into G. | [RP owner](../design/symbol-world/symbol-policy-and-compile-flow-projection.md) | Defined semantics; source/evaluator consumer pending |
| 106-RP07 | Pout stage and explicit Pin holes | P1 still determines Pout stage; generic Pin does not change producer stage. | [RP owner](../design/symbol-world/symbol-policy-and-compile-flow-projection.md) | Defined semantics; source/evaluator consumer pending |
| 106-RP08 | Public Policy pair literal/extraction through colon | Retired from the public algebra; retain both internal observations. | [RP owner](../design/symbol-world/symbol-policy-and-compile-flow-projection.md) | Defined semantics; source/evaluator consumer pending |
| 106-RP09 | Direct policy(x) and Policy after direct get_type | Observe value/type facts on the same source evaluation edge. | [RP owner](../design/symbol-world/symbol-policy-and-compile-flow-projection.md) | Defined semantics; source/evaluator consumer pending |
| 106-RP10 | Bind t=get_type(x), then observe t's Policy | Do not unconditionally recover x's Pp; t may have a new destination view. | [RP owner](../design/symbol-world/symbol-policy-and-compile-flow-projection.md) | Defined semantics; source/evaluator consumer pending |
| 106-RP11 | Constrain both projections | Use one invocation applicability relation, without a public pair or global propagation pass. | [RP owner](../design/symbol-world/symbol-policy-and-compile-flow-projection.md) | Defined semantics; source/evaluator consumer pending |
| 106-RP12 | A migration target exposes only one Policy atom | Still check full endpoints, presence and capabilities; preserve existing-first and no-reopen. | [RP owner](../design/symbol-world/symbol-policy-and-compile-flow-projection.md) | Defined semantics; source/evaluator consumer pending |

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
| 106-CL02 | Closure formation through struct | Head, body and dependencies use the existing formation relation; no new ClosureObject ontology. | [CL owner](../design/symbol-world/function-object-call-model.md) | Defined semantics; source/evaluator consumer pending |
| 106-CL03 | The callable implementation endpoint | Stop at an established leaf, without recursively expanding the same closure expression. | [CL owner](../design/symbol-world/function-object-call-model.md) | Defined semantics; source/evaluator consumer pending |
| 106-CL04 | tau_C, c_C, A_C and Impl_C | c_C belongs to V_tau_C; A_C=Type(c_C); AssociatedNamespace(A_C)=MemberScope(Core(A_C)), whose Val2[()] supplies Impl_C. AssociatedName is NameCoord at that root, not /tau(A_C); Core equality does not merge actual Places. | [CL owner](../design/symbol-world/function-object-call-model.md), [type owner](../design/symbol-world/type-values-places-and-borrow-views.md#associated-namespace-is-the-core-member-scope) | Defined semantics; source/evaluator consumer pending |
| 106-CL05 | Pure Q with no self-construction; well-formed tau with nonempty V_tau | Q has TypeRole and tau is a complete type. Applicable V_tau implementation entries can make it callable without making it SelfConstructible. Payload-bearing x remains an ordinary value. Empty V_tau likewise does not remove type identity. | [type owner](../design/symbol-world/type-values-places-and-borrow-views.md), [Pattern owner](../design/patterns-overload/pattern-values-relational-semantics-and-extraction.md#13-structural-role-registration-and-ordinary-callables) | Defined semantics; source/evaluator consumer pending |
| 106-CL06 | Type tau has several c, each with several associated implementations | Project every (c,Impl) entry into one family. Compute applicability and maxima once across the union, report ties as ordinary ambiguity, and seal the winning pair/projection/frame. Actual self is c, not tau; selected failure and runtime residue cannot reselect either component. | [CL owner](../design/symbol-world/function-object-call-model.md) | Defined semantics; source/evaluator consumer pending |
| 106-CL07 | An ordinary x:T has applicable associated Val2[()] but empty V_T | Call x through the associated entry with self=x; V_T membership is unnecessary. | [CL owner](../design/symbol-world/function-object-call-model.md) | Defined semantics; source/evaluator consumer pending |
| 106-CL08 | The first callable material | Form it in the same structural operation; require no prior arbitrary x:tau_C instance. | [CL owner](../design/symbol-world/function-object-call-model.md) | Defined semantics; source/evaluator consumer pending |
| 106-CL09 | Replace the sole Val2 self-construction witness without registering a new ConstructEdge | Purity and TypeRole persist; SelfConstructible may fail. Independently recheck WellFormedTau, including stale registration and both closure-home constraints. Equal pure Core gives the same TypeRole answer but not compatibility with distinct /tau homes. Type identity requires no dummy/deleted constructor. | [type owner](../design/symbol-world/type-values-places-and-borrow-views.md), [Pattern owner](../design/patterns-overload/pattern-values-relational-semantics-and-extraction.md#13-structural-role-registration-and-ordinary-callables) | Defined semantics; source/evaluator consumer pending |
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
| 106-LF05 | Bounded runtime state coexists with stable descriptions | Record lifetime implementation/refinement work, not a PR106 blocker. | [LF owner](../design/lifetime/lifetime-policy-and-overload-boundary.md) | Checks retained; refinement handed off |
| 106-LF06 | Conformance versus implementation | Report semantic scenarios, pending consumers and executed tests separately; claim no new end-to-end support. | [LF owner](../design/lifetime/lifetime-policy-and-overload-boundary.md) | Checks retained; refinement handed off |

### 106.12 Meta declaration capture boundary

| ID | Scenario | Required result | Canonical owner | Consumer status |
|---|---|---|---|---|
| 106-MD01 | [cap] (...) :meta => B | Invalid MetaDecl: the declaration layer has no capture slot. Do not form a captured ordinary closure and reinterpret it as meta. | [declaration owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md#1-one-declaration-two-surface-projections) | Defined semantics; declaration consumer pending |
| 106-MD02 | P let H { B } | Not an in-place spelling of generative MetaDecl; the implementation requires =>. Ordinary non-meta block syntax retains its own meaning. | [declaration owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md#11-general-heads-and-expression-bodies) | Defined semantics; declaration consumer pending |
| 106-MD03 | MetaDecl body reads an unpassed enclosing local x | x is unavailable/masked. No automatic closure dependency may bypass input admission; explicitly passing x through In is the lawful route subject to ordinary checks. | [meta owner](../design/meta-invocation/meta-object-invocation-and-policy-reduction.md#2-meta-instance-identity) | Defined semantics; source/evaluator consumer pending |
| 106-MD04 | Equal parent, selected callable and canonical In under different caller-local environments | Same MetaInstanceRootKey and instance; no CapturedEnv coordinate or hidden capture in callee identity. Hidden caller locals cannot affect results; repeated acquisition preserves current lawful instance state without reinitialization. | [meta owner](../design/meta-invocation/meta-object-invocation-and-policy-reduction.md#2-meta-instance-identity) | Defined semantics; source/evaluator consumer pending |
| 106-MD05 | Meta body forms an ordinary closure from admitted inputs, or In carries a dependency-bearing ordinary closure | Permit ordinary explicit/automatic dependencies on legally available material, retaining input normalization, transitive dependency and lifetime checks. The nested closure cannot recover a masked enclosing local or add a MetaDecl capture axis. | [dependency owner](../design/symbol-world/dependency-observation-and-realization.md#41-meta-declarations-have-no-closure-capture-channel), [meta owner](../design/meta-invocation/meta-object-invocation-and-policy-reduction.md) | Defined semantics; source/evaluator consumer pending |

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

### 106.14 Two-level NameValue and Path observations

| ID | Scenario | Required result | Canonical owner | Consumer status |
|---|---|---|---|---|
| 106-PT15 | NameExpr in Path and value-expected contexts | Read_name retains full NameValue; only value use performs Read_resident. Equal residents do not equate structures. | [Owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT16 | Compare e# and path_pattern projection | One defined projection: NameExpr stops at its first level; general values require applicable projection, with no source quotation fallback. | [Owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT17 | Round-trip a NameExpr n and an already bound NameValue a in the Path consumer | For n, first obtain Read_name(n), then project/reinject. For a=NameValue(field::adl), Interpret_Path(PathPatternProjection(a)) =_Path a and reprojection yields a#. Preserve the bound structure without repeating NameExpr lookup or making a node from binder spelling a. General splice supplies no Path decoder and performs no resident read. | [Owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT18 | Compare direct a$ with projection followed by splice | Direct splice uses the current Pattern; projected splice first converts it. No implicit conversion or repeated evaluation. | [Owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT19 | Index ((field::adl)#)[0] | Return relative field:: path_pattern, not string or a path retaining the adl endpoint. | [Owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT20 | Equal segment strings with different endpoints or explicit roots | Retain Omega distinctions and actual root dependencies; strings cannot recover identity or authority. | [Owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT21 | Reconstruct a Path whose target became unavailable | Surrounding Read_resident checks current access/validity without reopening or deriving authority from projection. | [Owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |
| 106-PT22 | General slice beyond the singleton case | p[i:i+1]=p[i] is fixed; general Slice_Omega remains open. Do not copy stale roots or infer an empty public Path. | [Owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md) | Defined semantics; source/evaluator consumer pending |

The PR106 extension contains **132 cases**. Each acceptance or rejection depends
on its owner's premises; schematic source is not an unconditional theorem.

## Decision-to-owner map

| Decision | Canonical owner / checks |
|---|---|
| D01 | [PD owner](../design/patterns-overload/pattern-values-relational-semantics-and-extraction.md), 106-PD cases above |
| D02 | [PD owner](../design/patterns-overload/pattern-values-relational-semantics-and-extraction.md), 106-PD cases above |
| D03 | [PD owner](../design/patterns-overload/pattern-values-relational-semantics-and-extraction.md), 106-PD cases above |
| D04 | [NS owner](../design/symbol-world/symbol-construction-units-and-namespace-origin.md), 106-NS cases above |
| D05 | [PT owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md), 106-PT cases above |
| D06 | [SP owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md), 106-SP cases above |
| D07 | [PT owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md), 106-PT cases above |
| D08 | [PT owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md), 106-PT cases above |
| D09 | [NM owner](../design/symbol-world/names-and-overload-groups.md), 106-NM cases above |
| D10 | [GN owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md), 106-GN cases above |
| D11 | [GN owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md), 106-GN cases above |
| D12 | [AD owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md), 106-AD cases above |
| D13 | [RP owner](../design/symbol-world/symbol-policy-and-compile-flow-projection.md), 106-RP cases above |
| D14 | [RP owner](../design/symbol-world/symbol-policy-and-compile-flow-projection.md), 106-RP cases above |
| D15 | [SP owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md), 106-SP cases above |
| D16 | [SP owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md), 106-SP cases above |
| D17 | [RP owner](../design/symbol-world/symbol-policy-and-compile-flow-projection.md), 106-RP cases above |
| D18 | [PT owner](../design/symbol-world/structured-path-algebra-and-pattern-splice.md), 106-PT cases above |
| D19 | [DP owner](../design/symbol-world/dependency-observation-and-realization.md), 106-DP and 106-MD boundary cases above |
| D20 | [CL owner](../design/symbol-world/function-object-call-model.md), 106-CL cases above |
| D21 | [CL owner](../design/symbol-world/function-object-call-model.md), 106-CL cases above |
| D22 | [CL owner](../design/symbol-world/function-object-call-model.md), 106-CL cases above |
| D23 | [LF owner](../design/lifetime/lifetime-policy-and-overload-boundary.md), 106-LF cases above |
