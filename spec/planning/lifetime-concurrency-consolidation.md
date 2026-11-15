# Lifecycle/concurrency consolidation ledger

## Actual baseline and worktree

- Date: 2026-11-15 (Asia/Shanghai).
- actual_base_sha: `efc0ddb2a216f6f6ee87821da6a7dab92f708195`.
- Working branch: `codex/lifetime-semantics-consolidation`.
- worktree_changes_before_task: untracked `.opensquilla/` and `MEMORY.md`;
  preserved and excluded from task edits/staging.
- The previous local branch belonged to merged PR #102. Main was fast-forwarded
  to origin/main and that merged local branch removed; its remote branch was
  already absent. The initial implementation task authorized no remote publication.
- Input: `lang_handoff_v1_1.zip`. Its design brief directs in-place owner
  migration; it supplies requirements/evidence, not assertions of implementation.
  Actual baseline equals its reference; E host improvements are preserved.
- Scope: current owners/contracts/public/entries/planning plus the discrete
  Region carrier and relevant unit fixtures. History and parser behavior unchanged.
- The subsequent user request authorizes a local commit, branch push and an
  auditable draft PR against main. The publication includes 41 tracked files
  modified and 8 task-owned files added (49 total), including a reproducible
  document/evidence audit and an evidence-only LF checkout rule preserving the
  original SHA-256 hashes across platforms. Exact commit and remote PR identifiers are recorded
  in Git and the draft PR metadata rather than self-referential content here.

## Source-consumer status before task

The baseline `lifecycle.rs` stores start/end scalar Regions, supplied Pre/Post
and directed Color rows. Formation/origin/Color importers are private test
fixtures. `CompilationWorld::sync_lifecycle_names` only discovers stable names,
leaving formation/origin pending; no full source lifecycle producer is wired.
`reify_value` ends at LifecycleContinuationPending after preliminary checks.
Source repair attachment reports UnsupportedEffectRepair. Full positional
call-input P let lhs=rhs is not provided by expression PolicyLet/capture slots.
No standard heap, thread-container or atomic semantic/backend consumer was
found. Existing no-reopen, source-erasure, ClosedReturn/ClosedInterface,
common transaction and host dependency contracts remain valid.

## Evidence notation and completion boundary

D=defined; P=proof discharged for stated domain; S=complete source syntax chain;
C=semantic consumer connected; T=positive/negative execution; B=validated backend.
A dash is incomplete, never a default success. Carrier/model evidence is named
as a narrower slice. This work migrates the normative specification and D1's
finite event carrier; it does **not** complete lifecycle/concurrency execution.

All N01–N45 constraints are distributed to unique topic owners. N43 defines
the boundary of the still-open F15 decision rather than choosing SeqCst/fence.
No F01–F18 general proof or backend support is claimed. The paired source
acceptance rows remain pending; parser/token preservation alone does not complete
their S/C/T chain.

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
| N28 | [heap-stack-and-storage-responsibility.md](../design/lifetime/heap-stack-and-storage-responsibility.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N29 | [heap-stack-and-storage-responsibility.md](../design/lifetime/heap-stack-and-storage-responsibility.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N30 | [heap-stack-and-storage-responsibility.md](../design/lifetime/heap-stack-and-storage-responsibility.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N31 | [heap-stack-and-storage-responsibility.md](../design/lifetime/heap-stack-and-storage-responsibility.md) | D | — | — | — | — | — | source/semantic consumer pending |
| N32 | [heap-stack-and-storage-responsibility.md](../design/lifetime/heap-stack-and-storage-responsibility.md) | D | — | — | — | — | — | source/semantic consumer pending |
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

## M replacement map and actual baseline hits

Locations below refer to actual baseline text, not copied obsolete line numbers.
“No prior consumer/rule” means new admission is specified, not that a keyword
scan proved arbitrary absence. Current owner text and consumer failure gates
were reviewed together; prohibitions/counterexamples are not obsolete positives.

| Clause | Actual baseline location / affected owner | Replacement and source boundary |
|---|---|---|
| M01 | [lifetime-policy-and-overload-boundary.md:536](../design/lifetime/lifetime-policy-and-overload-boundary.md) | Region; lifecycle.rs Region; glossary Region generation — N01–N03：离散事件集合、合法解释和端点偏序 |
| M02 | [lifetime-policy-and-overload-boundary.md:538](../design/lifetime/lifetime-policy-and-overload-boundary.md) | Region slicing; E §8; LC2/LC3; host observed-domain support — N02：实际可达切片；真删点才严格 |
| M03 | [lifetime-policy-and-overload-boundary.md:971](../design/lifetime/lifetime-policy-and-overload-boundary.md) | Escape/ValidRegion equations narrowed to endpoint/access; no SupportClosure implementation found — N03：仅严格首末嵌套，无补点 |
| M04 | [lifetime-algebra-walkthroughs.md:939](../design/lifetime/lifetime-algebra-walkthroughs.md) | W3, lifecycle §2.1.7/§9.4 and LC7 serial access dependencies separated from independent Color — N04–N06：默认入口关系、独立访问时染色 |
| M05 | [heterogeneous-computation-containers.md](../design/control-flow/heterogeneous-computation-containers.md) | New owner; effect and call crosslinks; no prior thread consumer found — N05、N07–N08：三项正交，普通逃逸 |
| M06 | [lifetime-policy-and-overload-boundary.md:875](../design/lifetime/lifetime-policy-and-overload-boundary.md) | Lifecycle §2.1.8; effect omission; all entry/public/contract summaries — N10–N14：origin 分类、伴随生产式自动重演 |
| M07 | [dependency-observation-and-realization.md:149](../design/symbol-world/dependency-observation-and-realization.md) | Named positional call input, live obligations and outer repair handoff added — N09–N11：输入槽、捕获及存活责任，外层递归 |
| M08 | [e-host-architecture-and-incremental-revalidation.md](../contracts/e-host-architecture-and-incremental-revalidation.md) | New recipe occurrence distinguished from old action/initialization reuse — N12–N14：普通初始化一次仍成立；新的已定义重演是新发生 |
| M09 | [lifetime-policy-and-overload-boundary.md:907](../design/lifetime/lifetime-policy-and-overload-boundary.md) | Retained expression/selected-partner plan; no prior purity/Copyable success path found — N12–N13：保留原对应；值保留非 copy；副作用真实 |
| M10 | [pattern-relations-observation-and-extraction.md](../design/patterns-overload/pattern-relations-observation-and-extraction.md) | N15; Split/D and W2: preserve correlated source choice, not recoverable arm authority — N15：模式分层 origin 合流 |
| M11 | [return-value-extraction-and-implicit-decomposition.md:194](../design/patterns-overload/return-value-extraction-and-implicit-decomposition.md) | Default structural view scoped; specific Option registered throw; W3/public fixed — N16：特定 Option `?` 注册，通用提取职责保留 |
| M12 | [lifetime-policy-and-overload-boundary.md:1533](../design/lifetime/lifetime-policy-and-overload-boundary.md) | N17/N18; W3 ordinary query distinction; API name remains illustrative — N17–N18：API 示例与匿名聚合访问的适用域 |
| M13 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | N19 and W3 copy/borrow/derived distinction; no prior index consumer found — N19：copy 刷新、借用传递/专门派生、端点偏序 |
| M14 | [product-repetition-and-unpacking.md:148](../design/patterns-overload/product-repetition-and-unpacking.md) | Aggregate insertion now explicitly requires conservative lifecycle summary and operations — N20–N21：遗忘前保守准入，整体依赖 |
| M15 | [lifetime-policy-and-overload-boundary.md](../design/lifetime/lifetime-policy-and-overload-boundary.md) | N21 and Product/extraction summaries; no prior subtraction consumer found — N21：默认不撤除整体依赖，新名字继承 |
| M16 | [lifetime-policy-and-overload-boundary.md:614](../design/lifetime/lifetime-policy-and-overload-boundary.md) | N22/N23 + Policy migration: universal and instance evidence distinct, retained operation domain — N22–N23：三类证据，Drop/Move 实现保存 |
| M17 | [unified-static-name-and-structural-semantics.md:684](../design/unified-static-name-and-structural-semantics.md) | Partial actual-instance recovery and declared failure narrowed in unified/Policy/contracts — N24–N25：部分恢复，具体拒绝走异常 |
| M18 | [heap-stack-and-storage-responsibility.md:46](../design/lifetime/heap-stack-and-storage-responsibility.md) | Heap §§2/4/5 and H03/H08/F08 remove preference/arbitrary depth — N26、N29：删除偏好，显式注入，精确 up |
| M19 | [heap-stack-and-storage-responsibility.md:49](../design/lifetime/heap-stack-and-storage-responsibility.md) | Current-top free and active upward handling explicitly replace vague credential routing — N27：栈顶接收，主动分流上抛 |
| M20 | [heap-stack-and-storage-responsibility.md](../design/lifetime/heap-stack-and-storage-responsibility.md) | LazyAccept/Pending capabilities and executable deferred duty introduced without fixed list layout — N28：能力必备、表示自由 |
| M21 | [heap-stack-and-storage-responsibility.md:90](../design/lifetime/heap-stack-and-storage-responsibility.md) | Exact direct-parent submit including Pending, feasible universal completion — N29：保证向直接父层原样提交 |
| M22 | [lifetime-policy-and-overload-boundary.md:605](../design/lifetime/lifetime-policy-and-overload-boundary.md) | Logical end/custody transfer distinguished from physical destruction in lifecycle/heap/effect — N30：普通延迟析构续体与单次托管 |
| M23 | [heap-stack-and-storage-responsibility.md](../design/lifetime/heap-stack-and-storage-responsibility.md) | Production/allocation escape Pre, structured windows and no language QS; no QS consumer found — N31–N32：结构化窗口，生产 Pre 拒绝 |
| M24 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | New formed-type consumer owner; structural/Pattern interfaces linked — N34–N35：`$` 后消费者、全部保留状态名字（含 I/Index 与 S）的外部 origin 闭合 |
| M25 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | New atomic domain/associated API rules; no field-to-outer certification — N36–N37：不向外传播；关联 ref/share 普通注册 |
| M26 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | Protocol-subset closure and F18 explicit; no prior atomic consumer — N37、N40、N45：内部不变量保持与特定协议分别证明 |
| M27 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | I/S/version/name/generation/rf identities separated; host dependencies extended — N38–N42：各类身份与来源分离 |
| M28 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | Single-point A/I/S transition and single-attempt result; ordinary scheduling — N39、N44：原始和模式 CAS 联合模式/数值/资源交换与结果；调度属于 effect/handler |
| M29 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | Rejected retains S_b; peek no payload/latest; general complete commit not CAS — N34、N39、N42：模式/提交/授权分离，成功旧资源移出且新资源移入，失败原样返还 |
| M30 | [atomic-pattern-transitions-and-causality.md](../design/lifetime/atomic-pattern-transitions-and-causality.md) | Causal/sync promises fixed, only extra SeqCst/fence undecided; bounded evidence limited — N41–N45：因果闭包与所有权同步已定，额外 SeqCst/fence 仅保留 F15 判定 |
| M31 | [heap-stack-and-storage-responsibility.md](../design/lifetime/heap-stack-and-storage-responsibility.md) | N33 locate Pre universal direct-parent/lazy guarantee; custom providers expressly allowed — N33：标准 locate 所有非根 pool 强制保证式父层提交；自定义 heap 不受此约束 |

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
already required. Finite endpoint tests establish neither full F01 nor F02–F04.

| ID | Required argument / consumer | Boundary while incomplete |
|---|---|---|
| F01 | 离散 Region 在固定事件、分支、循环/递归及 activation 下的解释；min/max 偏序及纯切片保持性；生产者/父子来源和当前 E/K 修订对应的独立准入 | 不用连续区间或 SupportClosure 顶替；缺端点、来源或修订证据不可用；纯端点为真或相同 K 身份不发放借用权限 |
| F02 | 析构先定、生成新增修复片段与生产 Pre 的无循环衔接 | 不通过移动旧 cleanup 迭代到成功 |
| F03 | 自动 repair 的来源切片、外层责任递归、根值保留、共享生产节点和支持的 SCC 重演 | 不重复所有者、不无限展开，不造未知根 |
| F04 | 显式与自动 repair 的确定组合/替代责任；异常与 IO 重演路径的真实效果 | 无精确计划时不发布成功后继；不能默认双执行 |
| F05 | Pattern 分层合流及实际提取与 origin/权限相关性；异 Pattern 精度保持 | 不隐含恢复同 Pattern 分支，不拼接来源权限 |
| F06 | 索引验证生产者、有效性协议、借用传递及派生 copy 的端点规则 | 裸数值/相同数值不授权；无重复验证补救隐藏入口 |
| F07 | 合法名字遗忘与 default-world 抽象的健全性；数组整体摘要与取出 | 无证据拒绝；不以 unknown 洗掉依赖 |
| F08 | Drop/Move 可执行材料保留和三类证据的适用域；实例证据不泛化 | 不强制 Copyable/类型 trait，不隐式 Box |
| F09 | reify 具体实例默认性判定的真实性、成功身份映射、异常及 ClosedInterface | 偏离实例拒绝；缺消费者不能假作恢复异常 |
| F10 | 独立容器的 effect 树变换、调用材料、Region/Color 和逃逸 | 无普通来源/同步关系不能并行发射或逃逸 |
| F11 | Heap 顶层接收、逐层补充/提交、单次责任守恒、Pending 顺序和保证完成资源条件 | 无能力不能登记/提交后丢责任；不偷扫原始 pool |
| F12 | Region 上界回收在生产 Pre 的逃逸证明，包括异常、挂起与 deferred destructor 的依赖 | 不把证明推迟到 drop；不靠 QS 语言事实 |
| F13 | Atomic 对全部 RetainedStateNames 的外部来源闭合准入（含保留 I/Index、S）；CAS 的 A(I_a;S_a)→B(I_b;S_b) 单点变换、成功旧资源取出与新资源写入、失败保留输入；无字段/keep/同态 RMW 变体及 Color 受控域 | I 角色不洗掉来源；新闭合名字须有真实普通生产 Post，未保留比较输入仍按普通 Pre；普通 safe 接口不能绕过竞争或分离资源提交；不宣称所有 Pattern 可用 |
| F14 | 版本/rf/mo、成功与失败结果分支的因果差异、后继提交发布/观察继承闭包、所有权交接推导的同步及 no-thin-air；强/弱失败观察 | 基本因果/同步已定但机器与 OOTA 保持性待证；不得由用户选 relaxed 等替代所需保障 |
| F15 | 是否仍需 SeqCst 级更强保证；若需要，决定是否采用独立 fence 及完整公理，检查双边/单边 SB、IRIW、2+2W、累积性与 lowering | **唯一保留的额外内存序设计问题**；在选定前不得把双边 SB 目标或完整 fence 说成源级已保证 |
| F16 | 后端对所选源内存模型的正确性；x86-TSO/AArch64/RVWMO/LLVM 分别验证 | 有限模型不是 ISA 或编译证明；未接后端不宣称支持 |
| F17 | 标准 locate 在构造 Pre 对所有非根 Pool 验证普遍保证式父层提交、lazy 接收和非根/根边界；保证不依赖无限内存 | **规范已定，消费者与证明待接通**；不可注册仅在部分状态偶然可迁移的标准层 |
| F18 | 库组合的不变量保持、受控入口与目标协议的精确差别 | 不传播整体 Atomic/Color，不由基础 safe 推出任意更窄子集 |

The dependency order is event/source material → correlated join/index/forget
→ founded repair → standard heap/lazy responsibility → atomic/causal backend
→ actual E-host dependency integration. F02–F04 must all hold before a real
repair successor is published. No general purity whitelist, implicit clone/Box,
unknown root, missing Post or hardware-CAS-then-rollback substitutes for a gate.

## Actual verification record

The following initial submission checks apply to commit `6972f865e1f1efbece0e3eb7d5ff0352834b993d`.
The audit follow-up and its separate validation record are described below.

- Targeted Rust command: `cargo test -p lang_build --lib lifecycle::tests`:
  37 passed, 0 failed after the carrier migration and same-cut refinement.
- New carrier cases: discrete non-subset parent/child endpoints; equal/outside/
  empty/foreign endpoints; pure restriction, unchanged slice and no restoration;
  actual {1,4,9} events without hole filling or serial Color introduction;
  distinct same-cut instruction ordinals, exact occurrence restriction and
  preserved unequal generation identities without equating parent/child numbers.
- Updated regression: unscheduled ordered same-cut use/ending retains both
  action events. Fixed scheduled cleanup precedence, Kill/Preserve, stale/foreign
  Pre, no duplicate Drop, no fabricated formation/origin and pending @ remain.
- `cargo fmt --all` and `cargo fmt --all -- --check`: exit 0.
- `cargo check --all-targets`: exit 0 after the final source revision.
- `cargo test --workspace`: exit 0; 53 suite summaries, 973 passed,
  0 failed, 0 ignored (including the 37 lifecycle tests).
- `git diff --check`: exit 0. No changes under history, lang_syntax or frontend
  golden tests. Local Git inspection warned that user-level ignore configuration
  and an existing .opensquilla attachment were inaccessible; no task check failed
  and neither path was read or changed to work around that boundary.
- Markdown file/fragment resolution and clause inventories: 40 changed/new
  normative Markdown files, 960 local links, no broken destinations/anchors;
  45 unique N owners, 12 U, 31 M, 18 F and 105 unique paired cases.
- Publication revalidation: formatting, all-target checking and the full
  workspace tests passed again (973 tests, 53 suite summaries, no failures or
  ignored tests). The committed audit command below checks 41 Markdown files
  including the evidence README, 962 local links, all clause/case inventories,
  the two preserved model hashes and actual stdout. It exits 0 with no broken
  destinations/anchors and requires neither Git state nor the downloaded ZIP:

  ```text
  python spec/planning/evidence/lifetime-concurrency/audit_consolidation.py
  ```

- Archive manifest: all 8 listed SHA-256 hashes match. The preserved model and
  expected output hashes match the package. Model execution exits 0 and actual
  stdout matches expected output after line-ending normalization.
- Independent fresh-reader review: passed all 15 questions using current entry
  documents/owners only, without history, chat, external handoff or MEMORY.
  Separate contradiction and code scouts finished; the primary agent reviewed
  their evidence and performed final edits/verification.
- Code review's same-cut occurrence concern was corrected by retaining cut plus
  action ordinal in each Region event. Its generation/correspondence concern is
  explicit: each operand keeps its own generation, equality is not required
  between parent and child, and actual producer/activation/control evidence
  remains a requiring-consumer premise. A supplied set query grants no authority.
- Final primary review also removed unconditional serial Color requirements in
  the scoped producer table/LC7, consistent with the revised W3 walkthrough.

## Fresh-reader answers and residual limits

| Question | Answer derived from current owners |
|---|---|
| 1 Region | Discrete events; strict parent/child endpoints allow interleaving without subset |
| 2 Color | Actual independent access introduces Color; async/unique Wait alone does not |
| 3 Saved material | Live dependencies and unfinished Drop count; origin-None roots need no clone/Copyable |
| 4 Repair | Outer handoff repairs inputs/captures; local replay retains original selected expression/partner |
| 5 Convergence | Same observable Pattern forgets source-arm distinction; actual Some grants trust, None grants none |
| 6 Index/K-V | Ordinary Index copy loses credential; ordinary lookup needs no validate_key phase |
| 7 Anonymous aggregate | New extraction inherits whole obligations; remaining aggregate does not subtract them |
| 8 Movable | Type-wide sufficient evidence and event/instance legality coexist without generalization |
| 9 Recovery | Hidden non-default dependencies prevent default recovery; missing consumer is unavailable |
| 10 Heap | Current-top malloc/free, immediate-parent relocate, unexecuted Pending; universal non-root locate capability |
| 11 Reclamation | Escape fails at production/allocation Pre; QS is not a language premise |
| 12 Atomic material | Consume admitted $-formed type; no new Pattern/enum and no arbitrary resource admission |
| 13 Composition | Atomic fields do not certify outer Atomic; safe actions do not enforce arbitrary narrower protocols |
| 14 CAS | One successful point exchanges state/S_a/S_b; Rejected retains S_b; scheduling is ordinary composition |
| 15 Memory | Branch causal closure and ownership visibility fixed; F15 extra order undecided; OOTA/backend proofs pending |

No contradictory current positive rule was found in the stated audits. Their
inspection is not a universal absence proof. Specific unimplemented source
domains remain those in the N/F table: full normalized-K/activation interpretation,
producer/partner material and F02–F04 replay, F05–F09 join/index/forget/recovery,
F10 independent containers, F11/F12/F17 standard heap and F13/F14/F16/F18 atomic,
host and backend chains. The 105 source scenarios have not executed, and no
general proof or backend B is claimed. Their missing prerequisites remain
Unavailable before action/success, not temporary alternative success semantics.

## PR 120 semantic audit corrections

Audit snapshot: `6972f865e1f1efbece0e3eb7d5ff0352834b993d`.
The user supplied three P2 findings; the correction preserves the original
specification-migration/finite-carrier scope and F01–F18 pending boundaries.

| Finding | Correction | Executable or normative evidence |
|---|---|---|
| Event/cut ambiguity | Remove From<SemanticPosition> and implicit ordinal zero; from_events requires full RegionEventPosition; separate contains_event and contains_cut | The new membership test covers (3,5), rejects (3,0)/(3,6), and checks empty/missing cuts. Committed-event tests also cover an actual nonzero ordinal |
| Endpoints are not authority | Rust docs, N03, scoped initialize_bounded_region, LC2/LC3, host handoff and current entry/public summaries require independently established selected producer/source, names/generations, activation/guarded branch and current E/K revision correspondence | Strengthened same-identity forked-snapshot regression has a true endpoint query but rejects reuse of the other snapshot's Pre without mutation. REG02/REG05 specify the still-pending source admission; no new Region/K or proof Object is introduced |
| Atomic state-number origins | N35 applies to every RetainedStateNames entry, including retained I/Index and S; actual ordinary producer Post may establish a fresh closed name. Temporary comparison-only unretained inputs retain ordinary Pre | N38/N39 and current summaries/handoffs updated; ATOM03/ATOM04/ATOM16 distinguish retained dependent I_b from a genuinely closed new state name. These are defined targets, not executed Atomic or F13 proofs |

Acceptance IDs remain 105 and all complete-source P/S/C/T/B statuses remain
pending. The unchanged bounded model does not test the new retained-Index rule.
The existing counterexamples/proof gates are strengthened in place; no general
borrow authorization or Atomic consumer is connected by these corrections.

Correction verification: `cargo fmt --all -- --check`,
`cargo check --all-targets`, and `cargo test --workspace` pass. The workspace
reports 974 passed, 0 failed across 53 test-suite summaries; the focused
lifecycle run reports 38 passed. The committed consolidation audit reports
41 Markdown files, 962 local links with no broken destination, 45 unique N
owners, 12 U, 31 M, 18 F and 105 unique paired cases. Both original bounded-model
hashes match, the model exits successfully, and its stdout matches the committed
output. Independent read-only Region and admission reviews were reconciled,
including the explicit endpoint/authority boundary in the public summaries.
These checks validate the finite carrier and documentation/evidence structure;
they do not execute the 105 source targets or discharge F01–F18.

## Input provenance and bounded evidence

The package manifest lists 45 N, 31 M, 12 U, 18 F and 105 paired source targets.
The original 01_SPEC.md SHA-256 is
`f1e0a5d2dca964b1ddbe966643db74113592caf9ce6d96753d988238da835ab9`;
02_LOCAL_AGENT_TASK.md is
`3fd0c0b8cf31023dafa476eb25d22c6f12e9563af4329d451fc7e60803887cff`.
The [acceptance matrix](lifetime-concurrency-acceptance.md) is distributed with
per-case status; the brief's instructions are implemented here rather than
installed as a second high-priority spec.

[Original bounded model](evidence/lifetime-concurrency/expressiveness_checks_v0_1.py)
SHA-256: `ab3749e73d086dc8f069a94acafb1d0bb90c039d1715c05d99105770c427e5ab`.
[Expected output](evidence/lifetime-concurrency/model_output.txt)
SHA-256: `74f49cfac4d398afc2ceef483b50729f21d8f4abfc3c6455f299813ce4ff1aa1`.
It checks ARC (9 states/27 transitions), Mutex (8/8), Slot (8/8), simplified
MP with/without causal import (13/14 states), weak-peek SB (14 states), plus
two deliberately invalid API counterexamples. It proves no full language,
resource-CAS/ISA/OOTA/fence/overflow/Weak/reclamation/compiler theorem.
