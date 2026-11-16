# HeapStack and storage responsibility

Status: canonical standard HeapStack protocol owner. Source library, common-E,
capability proofs and physical providers remain pending. The following notation
is target syntax/metanotation, not a claim of accepted or executable source.
All actions use ordinary Objects, actual captures/inputs and common Pre/commit/Post.
Historical formation, current storage support and release custody are distinct;
none substitutes for ordinary origin, generation, Region or Color.

## 1. Ordinary single-linked context

Standard globalheap supplies a compatible terminal root and a terminating
outward protocol, including after its legal runtime migration. Termination
promises neither infinite storage nor successful allocation. User Heap/IO
providers have their own ordinary contracts.

### N26 — 无分配偏好的单链路由 [不变量]

标准 HeapStack 使用普通结构及调用：

```text
HeapStack = (pool, next_pool: HeapStack ref | None)
mut let heap_locate = heap .. locate(pool);
let heap = heap_locate ref;
```

`locate` 使用对旧 heap 的真实借用，先形成局部 `heap_locate`，然后通过 `heap_locate ref` 显式创建当前层借用并以 `heap` 名字遮蔽旧入口。不得按值捕获或 Move 外层 heap 充当父层来源，否则失去外层 Region 偏序。没有 `prev` 反向链。当前 pool 需要补充资源时，合法取得/子借用 `next_pool`，局部重新绑定 heap，再普通调用 malloc。

标准 `malloc` 不再提供偏好参数；请求严格从当前栈顶发起，补充请求逐层转发。定制分配策略通过显式注入普通 heap 对象表达。

请求入口/管辖来源不等于最终供给物理块的 pool。根节点必须使转发终止；它不意味着容量无限或 malloc 必成功。

“当前 heap”是普通调用实际借用捕获或显式提供的环境材料，不是隐含线程局部变量。父层 heap 的 origin 若来自真实外部已知名字就直接代入，若仍是树外 unknown 就递归保留向外的 Pre；按值捕获不能替代这个来源证明。标准路由只在已准入环境内作保证。

### Allocation and Box dependency contracts

Standard malloc starts at the actual current heap-stack top. A successful
allocation's ordinary Pre/Post (or compatible unsafe Post after a real
external allocation success when necessary) establishes a fresh independent
origin and a strictly shorter Region than the allocated heap support:

```text
Region(allocated) prec Region(heap_support)
Origin(allocated) = fresh independent allocation source
```

The selected source and exact support/ownership transfer remain real
provider facts. A failed allocation forms no successful allocation origin or
storage authority; neither globalheap nor an exterior unknown implies
infinite capacity. A library Box's internally named `storge` field may
express its dependence as ordinary lifecycle relation material:

```lang
boxobject@.storge .. live_with(heap@)
```

`storge` follows the illustrative field spelling of the Box example.
`storge` and `live_with` here are library spellings, not new language
primitives. The current-top route identifies either the actual externally
known heap source or the scoped outside-tree unknown whose Pre must
be forwarded outward. The Box and allocation retain real Region, origin,
release and Drop obligations. Submission transfers management/support,
not historical origin.

## 2. Allocation and release routing

### N27 — 释放先交栈顶，分流主动上抛 [不变量]

标准释放不 eager 扫描 heap 链寻找历史可托付者，而是将合法释放责任交给当前栈顶。是否分离、本地处理或携带后端释放凭证向外层提交，属于当前 pool 的主动管理协议。

历史分配来源、当前存储支持者、释放责任持有者可以不同；实际凭证必须随操作保留，不能猜测物理地址所属者。释放能力不要求调用方手写原始 pool 路径。

本规则约束标准注册 HeapStack 的合法路由，不为用户任意注入的无关 provider 自动证明接受能力。越过该协议的显式使用仍需自己的普通契约。

### N28 — lazy free 是必备能力，不是固定列表字段 [不变量]

标准 heap 必须能够接受合法的 lazy free/drop 责任。语义上可记 `PendingDrop(H)`，但它是普通责任材料的说明，不是新的 Object 坐标或强制数据结构。

通用实现可维护 drop continuation list；特殊 pool 可以让整池/chunk 的管理本身承担这项待回收责任，甚至把 eager free 也转为 lazy free。

每份待处理责任包含合法执行所需的资源、捕获及顺序义务。标记指针、独立 API、描述符等识别方式不由抽象语义固定。不能在对象已消费后才因登记失败而丢失责任；接收动作的成功条件和失败资源归属必须在其正常 Pre/结果契约中闭合。

若队列匿名承载不同具体类型的析构续体，须按 N20–N24 保留其 Drop/Move 实现与全部必要依赖；不能只保存一个裸入口而遗失捕获或名字资格。

**An Atomic `S share` exit may use this ordinary lazy reception protocol,
but is not defined by it.** A successful CAS that actually removes S from a
`share` source slot must deliver unique owned S custody to some lawful Forget
receiver while simultaneously returning a valid `S share`. If the selected
ordinary receiver is a standard heap manager, its accepted pending duty
must preserve real Drop/Move, storage support and every outstanding legal
share Region, without a fallible *post-commit* attempt to discover ownership
or allocate missing registration resources. The same atomic/source law can
be realized by other ordinary protocols; no particular lazy list, ARC or
reader-guard representation is required by Atomic.

HeapStack's ability to accept Pending does **not** prove that concurrent
share acquisition, source-to-own mode recovery or eventual invalidating
Drop is safe. Those are independent real Pre/Post, Region/Color and
protocol-maintenance obligations; the selected library must prove them.
The [atomic owner](atomic-pattern-transitions-and-causality.md) N47–N50
defines only the resource handoff and access-right proof boundary.

## 3. Historical origin and current duties

`FormedFrom(x,P)` is immutable provenance; `StorageSupport(x,P)` is current
validity/management support; `ReleaseDuty(x,P)` is current release custody.
These are ordinary relation observations, not extra Object coordinates. Real
operations transfer duties once and retain their actual resource credentials.

## 4. Two relocation contracts

Object relocation obtains actual destination storage by an explicit lawful
parent-context request, then moves the complete value under selected ordinary
Move/repair, allocation/failure and alias/lifetime Pre. Standard requests advance
one parent link at a time. It neither rewrites origin nor repairs independent
aliases by a no-action name change.

### N29 — relocate 精确向直接父层提交且不执行 Pending [不变量]

对非根 H：

```text
relocate(heap H) -> up(H)
```

提交已有存储支持、管理描述、释放责任及待处理 drop continuation。对象的存活存储位置不因 pool 提交移动；Pending 原样交接，不在 relocate 时立即执行。

合法 Submit 的保证仍然是：满足其声明 Pre 的每个状态都能完成，不能隐藏临时元数据分配失败，也不能默认全局堆无限。原样提交、元数据重组、逐描述重建是表示/成本差别，不是不同成功语义。

消费当前层后，被遮蔽的父 heap 绑定恢复为当前名字。恢复可见性不复活已毁对象，也不删除历史。跨函数的 `up(H)` 可以是由调用契约代入的外部 unknown，但其“一步父链”关系确定，不是任意偏好选择的未知层。

Object relocate 与 pool relocate 仍不同：前者实际取得目标存储并搬运；后者只交接管理责任。前者不得因后者成功而假设独立借用/移动限制已经解除。

`SubmitCap(P,Q,Pre_submit)` means every legal state satisfying the declared Pre
admits guaranteed completion of Submit(P,Q). Pre includes actual credentials,
complete support/duty/descriptor correspondence, ordinary alias/access/cleanup
legality and feasible worst-case commit resources. Common commit transfers all
covered support/duties/descriptors; Post retains live Places and provenance,
with no orphaned or duplicate duty. Missing evidence is Unavailable; false Pre
rejects before mutation. There is no hidden allocation failure on legal Submit.

## 5. Relative regular storage paths

The only standard relocate target from H is up(H). Repeating k legal submissions
follows up^k(H); an external unknown parent still denotes the immediate parent,
not a choice of depth. Completing all local-layer submissions removes only their
covered storage-support barrier; uncovered or independent borrow obligations
remain. Regular path descriptions prove no missing submission or arbitrary escape.

### N30 — Heap 结束、逻辑死亡与实际析构责任分离 [不变量]

对象名字的 Region 终结，不强制其物理字节在该点消失；物理字节仍存在也不恢复已死名字的普通访问权。

显式采用 lazy drop 协议时，固定的逻辑结束动作交出值材料和析构责任，形成普通 drop continuation；接收 heap 的最终 drop 执行它。如果 heap relocate，责任继续由父层承接。

这不让生命周期计算重新选择析构点。原始续体已规定逻辑结束、责任转交以及之后的析构续体。延迟用户可观察的析构效果必须是该协议的真实语义，不能由 O 无证明地改期或删除。

每份释放/析构责任恰有一个合法持有者；继承既定清理顺序与相关外部依赖。允许延迟/提前物化回收的自由，以所有可观察效果和存储有效性保持为条件。

### N31 — 结构化挂起保持 heap 管辖关系 [不变量]

通过外层借用捕获的 heap，其来源有效期大于当前调用的合法借用窗口。挂起并不使该已成立的来源重新默认生成；合法交接后续体继承实际 Post。失败位于其本来的动作边界：未来使用/固定析构 Region 与继承 Color 不满足时，在请求发起 Pre 拒绝；出口仍依赖不可规约到合法外部 unknown 的本地 heap_locate 时，在 Post 名字压缩拒绝。既不提前汇聚所有约束求全局解，也不以 Wait 延长 Region。

heap 是普通环境材料，可以随具体续体保存和恢复。栈上较内层依赖其父层，正常退出或保证的逐层提交维持这项关系；挂起不暗中制造可以独立逃逸的 heap。

从较外层供给而在内层环境释放的对象，其责任沿合法当前链的 drop/relocate 被处理。保留的共同 K、父层借用、真实所有权和释放凭证共同构成证明，不以机器调用栈地址或全局扫描为依据。

取消偏好后，语言精确追踪的是一层一层的责任迁移路径；物理资源补充仍可递归。无法证明当前/父层路由和生命周期相容时，不允许“先接收，未来再猜”。

## 6. Composition, stages and acceptance

### N32 — Region 管辖回收，不把 QS 暴露为事实 [不变量]

RCU 风格、GC 风格等库可以利用较宽的 Region 管辖窗口承接动态复杂所有权关系的释放责任。无需精确找出最后一个引用或运行期可达集，便可在已证明的上界结束时执行待回收续体。

语言侧不把 QS、epoch 扫描轮次、调度器内部状态提升为可观察生命周期事实。`relocate` 延续责任所在的管辖窗口，`drop` 结束它并履行待处理责任；旧局部名字的 Region 本身不因此被延长或改写。

**逃逸义务位于生产/分配 Pre。** 请求一个将超出已准入管辖范围的对象时，其由固定续体给出的更大 Region 使该申请规约失败。不得改成 drop(heap) 时才检查有没有读者，或最后再隐式 join、延长 heap、寻找回收人。

前述 Pre 可消费已形成的控制/契约描述与真实迁移能力，但不能预支尚未成立的任意未来 Post。后续新操作保持已批准上界，不能扩大既有对象的逃逸能力。涉及独立执行时，访问兼容性及必要内存序仍另有证明。

这是支持保守回收的库模式，不自动证明完整 RCU、任意 GC 或回收算法的进展性质。有证明的实现可减少无可观察影响的物理存储滞留；这不授权提前执行协议已经固定在 heap drop 的待析构续体或改变其可观察效果。

### N33 — 标准 locate 的普遍保证式迁移与自定义 Heap 边界 [不变量]

**标准 `locate` 的每个非根 Pool 必须普遍具备向其直接父层的保证式提交能力**，这是标准 HeapStack 的构造/注册 Pre，不再逐次选择或留作接口政策问题。设 `up(H)` 为合法的直接父层：

```text
StandardLocated(H) and up(H) != None
  => GuaranteedSubmitCap(H, up(H), DeclaredAdmittedPre(H))

StandardLocatePre(pool, parent)
  => LazyAcceptCap(pool)
     and GuaranteedSubmitCap(pool, parent, DeclaredAdmittedPre(pool))
     and ordinary parent-borrow / Region / Color / access legality
```

上述能力必须覆盖**该标准 Pool 经其公开标准操作能合法到达的每个可提交存储管理状态**，而不只是当前测试对象或一次特定 `relocate` 恰巧能成功。不得通过把非空 Pending、合法存活分配或实际容量状态排除在能力前提之外，使用真空的 `SubmitPre` 冒充普遍能力。实际续体的普通借用、Region、Color 等 Pre 仍需分别满足；这些对象使用合法性前提不等于 Pool 可以对本应支持的管理状态任意放弃提交保证。合法提交不能因未声明的临时元数据不足而失败。`heap relocate;` 的唯一目标是 `up(H)`，并且必须转交包括 Pending 在内的完整管理责任，符合 N29。根 Heap 无父节点，依标准终止协议处理，不能要求不存在的父层 Submit。

无法保证交接的纯机器栈存储 Pool 不能注册进语言标准自动支持的 HeapStack；这**不禁止普通栈存储技术，也不禁止用户直接捕获、调用或显式注入其他 Heap**。用户自定义 Heap 不必符合标准形状、lazy 能力或 relocate 接口，但不能假借标准保证逃逸/回收证明。

普遍保证是类型/库协议上的全称承诺，由实际 `locate` 的注册 Pre 接纳；不是自动在每个使用点重新向外层搜索可托付者，也不是让语言为任意自定义 Heap 强制迁移能力。

Storage submission and Policy migration remain independent. Required/Initialized
heap captures use the dependency owner's ordinary context and name-shadowing
rules. IO need not have a HeapStack shape. The [acceptance matrix](../../planning/lifetime-concurrency-acceptance.md)
and [F11/F12/F17 ledger](../../planning/lifetime-concurrency-consolidation.md)
record the pending proof and consumer boundaries.
