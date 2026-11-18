# Lifecycle/concurrency consolidation ledger

## Scope and evidence baseline

- Semantic baseline for this specification audit: `453623d13c2eee3e0f505464e2e852eceed355f3`.
- Current inventories: 49 active N clauses, M01–M31, U01–U12, F01–F19 and 131 paired source targets; AS01–AS14 independently route recursive Pattern, transform, Atomic and heap cases.
- The preserved finite Region-carrier and bounded-model evidence has its own recorded commit/hash scope below. It is not evidence of a connected Atomic, transform, parser or backend consumer.
- Current authority is the topic-owner map. This ledger records dependencies and D/P/S/C/T/B status, not another semantic definition.

Clause identifiers are stable keys. The owner headings and ledger enumerate
the active ranges `N01–N29` and `N31–N50`. Gaps carry no obligations and
existing identifiers retain their meaning.

## Source-consumer status

The connected `lifecycle.rs` slice carries discrete events, supplied Pre/Post
and directed Color rows. Formation/origin/Color importers are private fixtures;
name discovery establishes no formation/origin. `reify_value` remains at
LifecycleContinuationPending after preliminary gates. Source repair attachment
reports UnsupportedEffectRepair; general call-input BindingSlot, common-K/CPS,
ordinary lifecycle programs, standard heap and Atomic source/backends remain
unconnected. Existing no-reopen, source-erasure, closed-reify, common transaction
and host dependency contracts retain their own requiring-consumer boundaries.

## Evidence notation and completion boundary

D=defined; P=proof discharged for a stated domain; S=complete source syntax;
C=semantic consumer; T=paired execution; B=validated backend. A dash is
incomplete. Carrier/model evidence is explicitly narrower than the source row.

Ordinary recursive Pattern and selected E-relative producers supply Atomic
inputs. Raw/Norm fixture coverage is available for recursive layers, sum
choice, contextual arrow and runtime holes. It does not complete S for
N34–N50. Atomic consumes actual wrapped Boxes and target representation
proofs. Heap evidence requires allocation-time lazy duty and commit-scoped
support reinterpretation. F13/F14/F16/F18/F19 remain pending; F15 retains
the extra ordering decision.

### Definition and consumer status

| Rule | Owner | Definition | Execution evidence |
|---|---|---|---|
| Typed wrapper | [Typed leaf](../design/patterns-overload/pattern-relations-observation-and-extraction.md#external-typed-leaves) and [N46](../design/lifetime/atomic-pattern-transitions-and-causality.md#n46--普通结构真实类型包装与表示预算-定义) | Defined | Source consumer pending |
| Mixed-stage inputs | [Invocation](../design/static-evaluation/compile-instance-invocation-and-result-delivery.md#runtime-inputs-to-compile-producers) | Defined | Source consumer pending |
| Lazy registration | [N28](../design/lifetime/heap-stack-and-storage-responsibility.md#n28--分配成功时注册-lazy-drop-不变量) | Defined | Source consumer pending |
| Heap submission | [N29](../design/lifetime/heap-stack-and-storage-responsibility.md#n29--relocate-精确向直接父层提交且不执行-pending-不变量) | Defined | Source consumer pending |
| Qualified infix | [Operator Patterns](../design/patterns-overload/operator-patterns-and-generative-declarations.md#qualified-binary-operator-targets) | Defined | Raw/Norm covered |
| Named layers | [Pattern relation](../design/patterns-overload/pattern-relations-observation-and-extraction.md#formation-and-relational-derivation) | Defined | Source consumer pending |
| Extra memory order | [F15](#f-admission-gates) | Open | Semantic choice pending |

## N clauses to owners and executable slices

| Clause | Normative owner | D | P | S | C | T | B | Exact scope |
|---|---|---|---|---|---|---|---|---|
| N01 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | D | — | — | carrier slice only | carrier tests only | — | Region carrier / lifecycle unit tests; source pending |
| N02 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | D | — | — | carrier slice only | carrier tests only | — | Region carrier / lifecycle unit tests; source pending |
| N03 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | D | — | — | carrier slice only | carrier tests only | — | Region carrier / lifecycle unit tests; source pending |
| N04 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N05 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N06 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N07 | [heterogeneous-computation-containers.md](../design/control-flow/heterogeneous-computation-containers.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N08 | [heterogeneous-computation-containers.md](../design/control-flow/heterogeneous-computation-containers.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N09 | [function-object-call-model.md](../design/symbol-world/function-object-call-model.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N10 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N11 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N12 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N13 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N14 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N15 | [pattern-relations-observation-and-extraction.md](../design/patterns-overload/pattern-relations-observation-and-extraction.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N16 | [return-value-extraction-and-implicit-decomposition.md](../design/patterns-overload/return-value-extraction-and-implicit-decomposition.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N17 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N18 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N19 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N20 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N21 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N22 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N23 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N24 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N25 | [unified-static-name-and-structural-semantics.md](../design/unified-static-name-and-structural-semantics.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N26 | [heap-stack-and-storage-responsibility.md](../design/lifetime/heap-stack-and-storage-responsibility.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N27 | [heap-stack-and-storage-responsibility.md](../design/lifetime/heap-stack-and-storage-responsibility.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N28 | [heap-stack-and-storage-responsibility.md](../design/lifetime/heap-stack-and-storage-responsibility.md) | D | — | — | — | — | — | Actual Drop continuation registered at successful lazy malloc; exactly one final release |
| N29 | [heap-stack-and-storage-responsibility.md](../design/lifetime/heap-stack-and-storage-responsibility.md) | D | — | — | — | — | — | Direct-parent handoff maps actual covered support uses in success Post |
| N31 | [heap-stack-and-storage-responsibility.md](../design/lifetime/heap-stack-and-storage-responsibility.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N32 | [heap-stack-and-storage-responsibility.md](../design/lifetime/heap-stack-and-storage-responsibility.md) | D | — | — | — | — | — | Current local allocation Pre, later actual boundary compression |
| N33 | [heap-stack-and-storage-responsibility.md](../design/lifetime/heap-stack-and-storage-responsibility.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N34 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N35 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N36 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N37 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N38 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N39 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N40 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N41 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N42 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N43 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N44 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N45 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N46 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | D | — | — | — | — | — | Typed child and actual Box correspondence from selected ordinary formation; total two-word budget |
| N47 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | D | — | — | — | — | — | Fixed Keep/Exit rights and preexisting read lazy duty |
| N48 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | D | — | — | — | — | — | Full comparison on arm/protocol change; same-arm read replacement and safe read-to-write |
| N49 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | D | — | — | — | — | — | Selected ordinary E-relative transform and readable snapshot consumed at actual CAS; rejected formed I/S retained |
| N50 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | D | — | — | — | — | — | Fixed lifetime/permission/causal proof duties; implementation methods left to libraries |

## M consumer obligation map

These IDs route current obligations to their canonical owners and consumers.
They introduce no alternate semantics or implementation/proof completion.

| Clause | Current authority | Consumer obligation |
|---|---|---|
| M01 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | N01–N03: discrete Region events, endpoint order and exact occurrence/generation evidence |
| M02 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | N02: pure reachable-event restriction; E/LC2/LC3 retain actual domain and source support |
| M03 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | N03: strict endpoint relation and independent actual producer/access Pre; no fabricated events |
| M04 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | N04–N06: new-entry defaults, inherited captures and independent-access Color; W3/LC7 |
| M05 | [heterogeneous-computation-containers.md](../design/control-flow/heterogeneous-computation-containers.md) | N05/N07/N08: ordinary effect-tree transformation, dispatch and escape are independent |
| M06 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | N10–N14: source-derived automatic repair, actual origins and selected producers |
| M07 | [dependency-observation-and-realization.md](../design/symbol-world/dependency-observation-and-realization.md) | N09–N11: positional named input, live obligations, capture/source timing and outer repair |
| M08 | [e-host-architecture-and-incremental-revalidation.md](../contracts/e-host-architecture-and-incremental-revalidation.md) | N12–N14: represented-history reuse does not replay; a lawful repair replay is a new occurrence |
| M09 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | N12/N13: selected expression/partner plan, root retention, shared producers and real effects |
| M10 | [pattern-relations-observation-and-extraction.md](../design/patterns-overload/pattern-relations-observation-and-extraction.md) | N15: correlated Pattern-layer origin convergence, without restored arm authority |
| M11 | [return-value-extraction-and-implicit-decomposition.md](../design/patterns-overload/return-value-extraction-and-implicit-decomposition.md) | N16: registered Option ? contract and independent generic extraction/view protocols |
| M12 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | N17/N18: ordinary query contracts versus anonymous-container indexing |
| M13 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | N19: name-bound index credential, copy/borrow/derived distinctions and strict endpoints |
| M14 | [product-repetition-and-unpacking.md](../design/patterns-overload/product-repetition-and-unpacking.md) | N20/N21: conservative whole-aggregate obligations plus executable Move/Drop before forgetting |
| M15 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | N21: extracted names inherit aggregate support; remainder does not subtract obligations |
| M16 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | N22/N23: Pattern-local defaults and instance/frontier evidence retain actual operation material |
| M17 | [unified-static-name-and-structural-semantics.md](../design/unified-static-name-and-structural-semantics.md) | N24/N25: partial actual-instance recovery, closed reify and ordinary declared failure |
| M18 | [heap-stack-and-storage-responsibility.md](../design/lifetime/heap-stack-and-storage-responsibility.md) | N26/N29: current-top routing, explicit heap context and exact direct-parent submission |
| M19 | [heap-stack-and-storage-responsibility.md](../design/lifetime/heap-stack-and-storage-responsibility.md) | N27: free enters current top; the active provider handles or forwards real duty |
| M20 | [heap-stack-and-storage-responsibility.md](../design/lifetime/heap-stack-and-storage-responsibility.md) | N28: establish lazy release at successful malloc; retain executable Pending custody |
| M21 | [heap-stack-and-storage-responsibility.md](../design/lifetime/heap-stack-and-storage-responsibility.md) | N29: direct-parent transfer preserves live Places and projects scoped heap-name reinterpretation |
| M22 | [heap-stack-and-storage-responsibility.md](../design/lifetime/heap-stack-and-storage-responsibility.md) | N28: logical end closes actual access; final execution consumes the previously registered unique Drop duty |
| M23 | [heap-stack-and-storage-responsibility.md](../design/lifetime/heap-stack-and-storage-responsibility.md) | N31/N32: check actual event windows, local malloc Pre and return compression; no QS authority |
| M24 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | N34/N35: consume formed associated E; every retained I/Index/S name is externally source-closed |
| M25 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | N36/N37: controlled Color scope and ordinary associated calls do not certify outer types |
| M26 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | N37/N40/N45: base safety and narrower library protocol/progress proofs remain separate |
| M27 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | N38–N42: state/resource/event/name/rf/Past identities and actual host validity support |
| M28 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | N39/N44: selected transform attempt jointly commits actual state/resource/rights; ordinary scheduling |
| M29 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | N34/N39/N42: Rejected retains all formed I_b/S_b; observation grants no resource/latest authority |
| M30 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | N41–N45: causal closure and ownership visibility; F14/F16 proofs and F15 extra-order decision |
| M31 | [heap-stack-and-storage-responsibility.md](../design/lifetime/heap-stack-and-storage-responsibility.md) | N33: universal direct-parent/lazy capability at every standard non-root locate; custom heaps separate |

## U correspondence map

The [unified owner's correspondence](../design/unified-static-name-and-structural-semantics.md#cross-owner-correspondence)
contains U01–U12 and consumes, rather than duplicates, the individual owners.

| Clause | Owner and integration | D | P/S/C/T/B |
|---|---|---|---|
| U01 | Unified + selected ordinary/Pattern/lifecycle/atomic action correspondence | D | — |
| U02 | Unified + ordinary four-way contracts and distinct failure domains | D | — |
| U03 | Unified/lifecycle/E: fixed events and founded generated repair | D | — |
| U04 | Unified/lifecycle/effect: identity slices and mapped successful handoff | D | — |
| U05 | Unified/lifecycle: type-wide evidence versus instance/frontier | D | — |
| U06 | Unified/Pattern/lifecycle: join, compression, SCC and forgetting distinct | D | — |
| U07 | Unified/call/dependency: directional Function Pattern and actual input slots | D | — |
| U08 | Unified/E: generation grain, observable effects and existing ABI | D | — |
| U09 | Unified/heap/atomic: reclamation versus visibility | D | — |
| U10 | Unified/construction/atomic: formed type and associated ordinary API | D | — |
| U11 | Unified/Policy: partial recovery and closed outward failure effects | D | — |
| U12 | Unified/E host: consumed recipe/qualification/custody/causal dependencies | D | — |

## F admission gates

These are not proof-complete claims. Every gate is pending proof/source wiring;
F15 additionally needs a semantic decision, while F17's universal guarantee is
already required; F19 records proof/consumer obligations for recursive Pattern formation,
selected transforms and Atomic access-right handoff without deciding memory-order F15. Finite endpoint tests establish neither full F01 nor F02–F04.

| ID | Required argument / consumer | Boundary while incomplete |
|---|---|---|
| F01 | 离散 Region 在固定事件、分支、循环/递归及 activation 下的解释；min/max 偏序及纯切片保持性；生产者/父子来源和当前 E/K 修订对应的独立准入 | 不用连续区间或 SupportClosure 顶替；缺端点、来源或修订证据不可用；纯端点为真或相同 K 身份不发放借用权限 |
| F02 | 析构先定、生成新增修复片段与生产 Pre 的无循环衔接 | 不通过移动旧 cleanup 迭代到成功 |
| F03 | 自动 repair 的来源切片、外层责任递归、根值保留、共享生产节点和支持的 SCC 重演 | 不重复所有者、不无限展开，不造未知根 |
| F04 | 显式与自动 repair 的确定组合/替代责任；异常与 IO 重演路径的真实效果 | 无精确计划时不发布成功后继；不能默认双执行 |
| F05 | Pattern 分层合流及实际提取与 origin/权限相关性；异 Pattern 精度保持 | 不隐含恢复同 Pattern 分支，不拼接来源权限 |
| F06 | 索引验证生产者、有效性协议、借用传递及派生 copy 的端点规则 | 裸数值/相同数值不授权；无重复验证补救隐藏入口 |
| F07 | 合法名字遗忘与 default-world 抽象的健全性；数组整体摘要与取出 | 无证据拒绝；不以 unknown 洗掉依赖 |
| F08 | Drop/Move 可执行材料保留、Pattern-local 默认表达式与实例移动证据的适用域；默认不证明全称 Movable | 不强制 Copyable/类型 trait，不隐式 Box |
| F09 | reify 具体实例默认性判定的真实性、成功身份映射、异常及 ClosedInterface | 偏离实例拒绝；缺消费者不能假作恢复异常 |
| F10 | 独立容器的 effect 树变换、调用材料、Region/Color 和逃逸 | 无普通来源/同步关系不能并行发射或逃逸 |
| F11 | Current-top malloc/free, standard direct-parent guaranteed Submit with storage/duty transfer, allocation-time lazy duty and scoped LifeProject correspondence | Preserve history and live Places; no execution of Pending at relocation |
| F12 | Local allocation Pre, real Region and support at each actual handoff/return, with forward name compression and fixed cleanup | No retroactive malloc constraints or late proof at Drop; no QS premise |
| F13 | Complete E, actual Box wrappers and finite representation with single CAS, N35 closure, fixed Keep/Exit rights and Rejected material return | No inferred roles, duplicate release duty, dynamic read retirement registration or lost candidate material |
| F14 | Actual version/rf/mo; branch-specific causal inheritance/publication; visibility of returned owned S or valid S share; no-thin-air and weak/strong failure distinction. | Numeric Status equality, hidden-field comparisons or raw peek do not imply a committed version or any resource authority; backend/OOTA proof pending. |
| F15 | 是否仍需 SeqCst 级更强保证；若需要，决定是否采用独立 fence 及完整公理，检查双边/单边 SB、IRIW、2+2W、累积性与 lowering | **唯一保留的额外内存序设计问题**；在选定前不得把双边 SB 目标或完整 fence 说成源级已保证 |
| F16 | Backend correctness of single-point Pattern/Status/resource/right handoff including valid published share visibility; x86-TSO/AArch64/RVWMO/LLVM validated separately. | No finite ARC/RCU model or relaxed CAS instruction proves selected source ownership/synchronization; do not claim unconnected backend support. |
| F17 | 标准 locate 在构造 Pre 对所有非根 Pool 验证普遍保证式父层提交、lazy 接收和非根/根边界；保证不依赖无限内存 | **规范已定，消费者与证明待接通**；不可注册仅在部分状态偶然可迁移的标准层 |
| F18 | Safe library composition of all Atomic source-slot operations, ordinary counted-reader/hidden Status/version/clone/retirement/wait implementations under actual Pre/Post. | No mandatory ARC/RCU/GC/Mutex or lazy scheme; no share interior mutation, protocol-subset theorem or automatic progress from individual safe actions. |
| F19 | Recursive Pattern formation, pre-body mixed-stage identity, typed wrapper and actual Box evidence, selected transform, comparison and one-version projection, read registration and two-word target proof | No purely syntactic or numeric authorization; missing backend proof is unavailable |

The dependency order is event/source material → correlated join/index/forget
→ founded repair → standard heap/lazy responsibility → atomic/causal backend
and Atomic slot/mode proof F19 → actual E-host dependency integration. F02–F04 must all hold before a real
repair successor is published. No general purity whitelist, implicit clone/Box,
unknown root, missing Post or hardware-CAS-then-rollback substitutes for a gate.

## Atomic source and proof boundary

Recursive layer/binder incidence, selected ordinary transform construction,
compatible omission, full/masked matching and single-version readable projection
are the producer inputs to N34–N50. Each consumer must establish its actual
Pre/commit/Post, branch custody and causal obligations. Neither numeric counts
nor a protocol name supply rights. Concrete reader/reclamation methods remain
ordinary library choices; F19 does not choose an ARC/RCU/GC/Mutex scheme.

## Actual verification record

Recorded finite-carrier regression evidence is scoped to its source commits:

| Recorded source snapshot | Observed checks | Practical scope |
|---|---|---|
| `6972f865e1f1efbece0e3eb7d5ff0352834b993d` | 37 lifecycle tests; 973 workspace tests in 53 suite summaries; fmt/all-target check passed | Discrete event carrier and common transaction fixtures, not source lifecycle/Atomic execution |
| `89cf17bd06b9d893d99e7a57eb479a78d99b0d04` | 38 lifecycle tests; 974 workspace tests in 53 suite summaries; fmt/all-target check passed | Exact cut/ordinal membership, current-snapshot rejection and retained-state-origin regression boundaries |

These recorded runs do not fill P/S/C/T/B for the 131 source targets or
AS01–AS14. Current checkout structural/evidence verification is reproducible:

```text
python spec/planning/evidence/lifetime-concurrency/audit_consolidation.py
```

The checker discovers every tracked Markdown file outside history. It validates
owner references, definition and consumer statuses, and clause/case inventories,
local links and Markdown structure, and the preserved model hashes and stdout.
It is not a semantic consistency proof. Repository fmt/check/test runs cover
their implemented Rust slices, not pending Pattern formation, transform and CAS semantics.


## Current rules and pending consumers

| Question | Answer derived from current owners |
|---|---|
| 1 Region | Discrete events; strict parent/child endpoints allow interleaving without subset |
| 2 Color | Actual independent access introduces Color; async/unique Wait alone does not |
| 3 Saved material | Live dependencies and unfinished Drop count; origin-None roots need no clone/Copyable |
| 4 Repair | Outer handoff repairs inputs/captures; local replay retains original selected expression/partner |
| 5 Convergence | Same observable Pattern forgets source-arm distinction; actual Some grants trust, None grants none |
| 6 Index/K-V | Ordinary Index copy loses credential; ordinary lookup needs no validate_key phase |
| 7 Anonymous aggregate | New extraction inherits whole obligations; remaining aggregate does not subtract them |
| 8 Movable | Pattern-local entry defaults do not imply Movable; actual event and repair proofs still apply |
| 9 Recovery | Hidden non-default dependencies prevent default recovery; missing consumer is unavailable |
| 10 Heap | Current-top malloc/free, immediate-parent relocate, unexecuted Pending; universal non-root locate capability |
| 11 Reclamation | Local malloc Pre, then actual boundary compression; no QS premise |
| 12 Atomic material | Consume admitted $-formed type; no new Pattern/enum and no arbitrary resource admission |
| 13 Composition | Atomic fields do not certify outer Atomic; safe actions do not enforce arbitrary narrower protocols |
| 14 CAS | Ordinary recursive layers plus selected `->::E` transform; Atomic single commit uses source read/write Exit/Keep, exact source matching and Rejected input return; scheduling is ordinary composition |
| 15 Memory | Branch causal closure and ownership/share-publication visibility fixed; F15 extra order undecided; OOTA/backend proofs pending |
| 16 Atomic contract | Ordinary recursive Pattern + selected `->::E` transform, Atomic one-point CAS/Color; same-arm partial Status and read replacement; source read/write result and failed candidate return; F19 pending |


Specific pending domains are full common-K/activation interpretation and
source production, F02–F04 repair, F05–F09 join/index/forget/recovery, F10
independent containers, F11/F12/F17 standard heap and F13/F14/F16/F18/F19
Atomic/source/host/backend chains. Missing prerequisites stay unavailable
before action/success. All 131 source cases and AS01–AS14 remain unexecuted.

## CI audit integration

The [CI workflow](../../.github/workflows/ci.yml) runs the committed audit in
the independent `Specification and evidence audit` job on pull requests, main
pushes and manual dispatch, using Python 3.13 and the standard library with
read-only repository permissions. Link/inventory, preserved-hash or bounded
stdout failures fail the job. Passing it proves neither textual semantic
consistency nor source/permission/backend execution.

## Bounded evidence

[Preserved bounded model](evidence/lifetime-concurrency/expressiveness_checks_v0_1.py)
SHA-256: `ab3749e73d086dc8f069a94acafb1d0bb90c039d1715c05d99105770c427e5ab`.
[Expected output](evidence/lifetime-concurrency/model_output.txt)
SHA-256: `74f49cfac4d398afc2ceef483b50729f21d8f4abfc3c6455f299813ce4ff1aa1`.
It checks ARC (9 states/27 transitions), Mutex (8/8), Slot (8/8), simplified
MP with/without causal import (13/14 states), weak-peek SB (14 states) and two
invalid API examples. It proves no full resource-CAS/Region/Color/repair/heap,
ISA/OOTA/fence/overflow/Weak/reclamation or compiler-preservation theorem.
