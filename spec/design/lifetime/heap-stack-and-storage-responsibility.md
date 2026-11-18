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

标准 `malloc` 没有偏好参数；请求严格从当前栈顶发起，补充请求逐层转发。定制分配策略通过显式注入普通 heap 对象表达。

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

普通即时释放不 eager 扫描 heap 链寻找历史可托付者，而是将合法释放责任交给当前栈顶。是否分离、本地处理或携带后端释放凭证向外层提交，属于当前 pool 的主动管理协议。

历史分配来源、当前存储支持者、释放责任持有者可以不同；实际凭证必须随操作保留，不能猜测物理地址所属者。释放能力不要求调用方手写原始 pool 路径。

lazy 句柄遵循 N28 已登记责任；按正常释放入口检查时只验证实际模式、登记及相关资格，不再提交 Drop 续体。

本规则约束标准注册 HeapStack 的合法路由，不为用户任意注入的无关 provider 自动证明接受能力。越过该协议的显式使用仍需自己的普通契约。

### N28 — 分配成功时注册 lazy Drop [不变量]

带 lazy 请求的 malloc 需要选定普通 Drop/释放续体及其所有必要
材料。成功 Post 为实际分配 `a` 的世代 `g` 注册唯一未执行续体
`d`，并建立实际 storage support 和持有资格。分配失败不产生登记。

```text
MallocLazyPre(K, request, H, d) and CommitMalloc(K, request) = (K', a, g)
    => Support_{K'}(a,g,H) and RegisteredLazyDrop_{K'}(H,a,g,d)
       and one FinalReleaseDuty_{K'}(a,g,d)
```

`d` 记录实际析构实现、类型、存储、捕获或其合法参数化绑定、
固定清理顺序和依赖。若请求允许尚未初始化的存储，已登记的续体
按普通初始化/占有事实决定是否执行对象析构；每个合法完成分支
仍只释放一次分配。成功分配已经建立完成该责任所需的全部保证。
逻辑结束按现存续体结束名字/资格；物理字节存续不恢复访问权。
盒上的 lazy Drop 或 read Forget 只执行声明的 NoOp 回收路径，
并按实际动作结束相应持有资格，沿用既有登记。最终执行 `d`
需要所有约束其执行的持有/借用窗口、清理和同步证据；执行后消费
唯一责任。符合 N29 的 relocate 转交同一登记与责任。
非 lazy 存储沿其正常释放契约；其他普通延期清理由各自已选协议负责。

`PendingDrop(H)` 观察该 heap 已登记的未执行责任。登记可由池、
类型化 Box 句柄或标签等方式承载。普通检查须验证标签/句柄所属的
实际分配与世代及匹配的登记和模式；一位标记不能凭空证明曾注册
(d)。例如 read 包装的选定普通构造器可以用局部 lazy 分配请求，
也可以检查已有 Box 的登记再依法包装。heap 接受 Drop 请求时仍须
验证当前责任和模式。该查验也适用于一般 Box 的释放分流。

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

令 `r` 为已选 Submit 的**实际成功交付**，`K^-`、`K^+` 为该
事件前后的续体。记 `h` 为被消费的当前 heap 绑定的实际名字/世代，
`q` 为此次解除遮蔽后恢复的直接父绑定及其实际来源对应。跨边界
的 `q` 可指具有普通参照依据的外部 unknown。记 `A_r` 为实际
交接的分配/世代、管理材料及 Drop 责任，并消费普通借用与清理 Pre。
令 `SupportUse_K(p,a,g,h)` 表示在实际位置 `p` 的存储支持证明，
由同一续体的名字/来源材料确认其使用 `h` 管辖该分配/世代。

```text
Dom(rho_r) = { occurrence u=(p,a,g,h) in LifeProject(K^-)
               | SupportUse_K^-(p,a,g,h)
               and (a,g) covered by A_r
               and p uses the actual removed shadow binding h
                   with the admitted support correspondence }

rho_r(u) = support use for (a,g) through q in K^+       if u in Dom(rho_r)
rho_r(u) = u                                           otherwise

CommitSubmit(K^-, r) = K^+
    => restores q and transfers A_r with the same live Places and pending duties
       and LifeProject(K^+) = LifeNameReinterpret(LifeProject(K^-), r, rho_r)
```

`p` 的判定使用实际绑定、遮蔽、activation 和结构证据，包括仍有效的
支持对应。相同拼写、同层级或物理地址都不足以入域。
`LifeNameReinterpret` 执行该动作已有的生命周期 Post（消费局部绑定、
恢复父层、转交既有责任），并按 `rho_r` 重解释支持证明。映射只替换提交所
覆盖的**支持使用**及其对应的责任终端，限定在这次成功 Post。原名字
位置和 `Region_K(n)=R(NamePosition_K(n))` 的规则继续成立；历史
origin、原 Place/世代、外部名字、独立借用、Color 和已选 Path 的
证据无凭据不得改写。将旧名字替换为任意更长 Region 不能证明本规则。
失败或尚未完成的提交不发布 `rho_r`。

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

### N31 — 结构化挂起保持 heap 管辖关系 [不变量]

通过外层借用捕获的 heap，其来源有效期大于当前调用的合法借用窗口。挂起并不使该已成立的来源重新默认生成；合法交接后续体继承实际 Post。失败位于其本来的动作边界：未来使用/固定析构 Region 与继承 Color 不满足时，在请求发起 Pre 拒绝；出口仍依赖不可规约到合法外部 unknown 的本地 heap_locate 时，在 Post 名字压缩拒绝。既不提前汇聚所有约束求全局解，也不以 Wait 延长 Region。

heap 是普通环境材料，可以随具体续体保存和恢复。栈上较内层依赖其父层，正常退出或保证的逐层提交维持这项关系；挂起不暗中制造可以独立逃逸的 heap。

从较外层供给而在内层环境释放的对象，未执行的既有责任沿合法当前链处理，非 lazy 释放走其正常路由。保留的共同 K、父层借用、真实所有权和释放凭证共同构成证明，不以机器调用栈地址或全局扫描为依据。

语言精确追踪一层一层的责任迁移路径；物理资源补充仍可递归。无法证明当前/父层路由和生命周期相容时，不允许“先接收，未来再猜”。

## 6. Composition, stages and acceptance

### N32 — Region 管辖回收，不把 QS 暴露为事实 [不变量]

RCU 风格、GC 风格等库可以利用较宽的 Region 管辖窗口承接动态复杂所有权关系的释放责任。无需精确找出最后一个引用或运行期可达集，便可在已证明的上界结束时执行待回收续体。

语言侧不把 QS、epoch 扫描轮次、调度器内部状态提升为可观察生命周期事实。`relocate` 延续责任所在的管辖窗口，`drop` 结束它并履行待处理责任；旧局部名字的 Region 本身不因此被延长或改写。

前向检查以每个实际动作的当前状态为准：

```text
{ current H1 and R(Box) prec R(H1), legal allocation request }
  malloc
{ fresh Box, actual H1 support and actual release policy }

{ live H1, actual up(H1), valid SubmitCap and covered duties }
  HeapRelocate
{ up(H1) support for submitted material, scoped life-name reinterpretation }

{ actual successful continuation, legal boundary compression and delivery }
  return Box
{ delivered Box with supported outward dependencies }

ReturnPre(K_exit, Box, mapping)
    requires Compress(LifeProject(K_exit), actual exit and mapping)
        = outward dependency material satisfying ordinary delivery Pre
```

malloc 只检查当前本地 Region 和当下动作义务。未来返回路径不参与
当前 Pre。后来 relocate 独立检查自己的 Pre。返回边界使用真实到达的
续体及其最新 `LifeProject`，只消费已经提交且在此仍有效的 `rho_r`
结果，再用普通 actual-exit compression 保留真实入参/捕获及外部
unknown 的义务。它不重放 Submit 或 malloc，也不倒推其 Pre。仍有无法压缩的局部支持或独立局部借用时，在实际
返回边界拒绝，先前 malloc 的结果保持有效。若分配本身立即请求更长
Region 或交付，则仍须满足当下动作的普通 Pre。Drop 使用预先建立的责任。

### N33 — 标准 locate 的普遍保证式迁移与自定义 Heap 边界 [不变量]

**标准 `locate` 的每个非根 Pool 必须普遍具备向其直接父层的保证式提交能力**，这是标准 HeapStack 的构造/注册 Pre。设 `up(H)` 为合法的直接父层：

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
