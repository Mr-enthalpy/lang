# Atomic Pattern transitions and causal memory

Status: canonical owner of atomic registration, one-attempt CAS/resource
handoff and causal memory. D is established for N34–N42/N44–N45; F13–F18
proofs, source consumers and machine lowering remain pending. N43 is a bounded
open semantic decision. No executable atomic API or cross-ISA support is claimed.
All equations and conventional call notation below are metanotation; source
sketches are target syntax with unconnected consumers.

### N34 — atomic 消费 `$` 已形成的模式类型 [不变量]

`atomic` 与 `struct` 处于相同的类型材料消费者层次：输入是已经通过 `$` 及既有结构形成规则生成的完整类型，输出注册所需原子操作的完整类型。**其基本语义是受信任的和模式变换**：在合法原子状态实例上，对既有和 Pattern 的某个当前分支执行不可分割的条件状态转换，相关的状态数、受控资源与结果交付一并提交。它不是新的 enum 声明语法、Pattern 类别、和模式构造器或 Object 坐标。

源码位置：

```text
formed_type |> atomic::
```

Cell 的结构意图用元记号表达为：

```text
P_Cell(T) = Empty | Live(uint32, Stable(T)) | Retired
Cell(T)   = atomic(Form$(P_Cell(T)))
```

这不是传统 `f(args)` 源码。实际源拼写必须沿既有生成式声明、`self` 名字及 `$` 极性语法形成。状态身份与 Stable 资源可使用现有 bare Product；若引入更语义化的 `I op S`，它只能是一般合法模式材料运算，不能作为 atomic 私有的第二套模式语法。该 operator 名称不是冻结项。

输入模式可以结构化复用既有 atomic 类型和普通语言材料，形成合法递归组合。一次一般模式构造/提取的“原子完整提交”不自动等于跨执行主体的机器 CAS。

### N35 — Atomic 资源准入要求没有外部 origin 依赖 [不变量]

atomic 不无差别接受所有类型材料。其持有资源角色必须满足：

```text
ExternalOriginDeps(held resource, atomic resource boundary) = empty
```

这对应资源所有权的实际语义条件，而不是看到 Rust 风格 `Owned<T>` 名称就接受。条件必须覆盖后续允许放入状态的资源，包括原始 CAS 的目标分支输入资源 `S_b`，并由相应形成/转换 Pre 维持。资源的内部 origin 可以合法递归，但未知/已遗忘的非默认外部依赖不得冒充为闭合。

它不等于所有内部 origin 都为 None；内部合法结构关系仍可存在。但隐藏外部借用的 unknown 不等于没有外部依赖。不能用名字遗忘洗掉该准入要求。

该限制使 atomic 的状态资源不需要依靠外部挂起点的自动 repair 来恢复借用关系；atomic 事务不引入“重演旧 CAS 以修复内部资源”的特殊机制。外部持有的 Cell ref/share 及凭证仍是普通依赖名字，遵守自己的交接规则。

### N36 — 受信任的重叠独占染色只覆盖注册协议 [不变量]

atomic 产生的类型提供其原子访问域的受信任兼容关系，允许多个独立执行主体对同一实例具有重叠独占染色。

异构容器仍在入参/捕获传入 Pre 检查存活 Region；只是在 Color 兼容判断上消费这个类型已经提供的保证。容器无需认识“Atomic 是特殊线程类型”，atomic 也不内建 Thread 规则。

该保证不自动授予内部一般资源的重叠独占写，不替代稳定资源借用的独立 Pre，不让任何普通字段访问绕过原子协议。

### N37 — 组合封闭性不向外传播 Atomic 性质 [不变量]

```text
every field of T is atomic  =/=>  T is atomic
each internal action safe =/=>  composition is one transaction
```

更强表达力来自两处：atomic 接受的普通结构材料可以递归组合；在 OpenHere 下，可以向作为关联类型存在的 Cell ref、Cell share 等注册普通 API，调用已提供材料完成组合。

不添加“遍历所有公开方法然后自动授予外层并发 Color”的通用认证机制。组合能保持内部已强制的安全不变量，不自动保证一个任意更窄的协议、安全子集、全局线性化或公平性。

若某个协议只允许基础转换的一个子集，必须由实际材料、可用入口与 Pre 完整约束这一子集，或单独给出可导出的证明。仅将 ticket 构造器私有化，却保留能任意修改计数的 safe CAS，并不能证明 ARC 不变量。

OpenHere 提供注册能力，不提供伪造生命周期/原子权限的能力。一般结构的真实成员、Pattern 观察及可执行访问仍分别按既有规则判断。

### N38 — 状态、资源、事件版本及名字身份分离 [定义]

对一个 atomic 实例 a，区别：

```text
P : current ordinary sum-Pattern alternative
I : comparable state identity/count/index value
S : held resource admitted by N35
v : identity of a committed atomic event
n/g : ordinary lifetime name and generation
```

`I` 不隐式拥有、解引用或转换成 S。更新 I 不要求搬动 S。相同 P、相同 I、相同业务值都不意味着同一个提交事件或 generation。

数值 ABA 不被默认禁止；需要“自观察以后绝无改变”的协议必须另外提供版本/世代判定。无需把语义事件身份实现为无限增长机器整数，也不从名字等同性推出物理地址相等。

### N39 — 原始 CAS：受信任和模式变换、资源交接及结果和模式 [不变量]

最基本的原始 CAS 不是仅比较一个整数，而是对已由 `$` 形成的和 Pattern 的**单个实例**实施一次不可分割的条件转换。采用语义元记号（不是新增源码语法）：

```text
CAS_a:  (I_a) A  ->  (I_b, S_b) B

old actual state:  A(I_a; S_a)
provided input:   I_b, S_b
on success:       atomic state becomes B(I_b; S_b)
                  old S_a is delivered to caller (if A holds S_a)
                  result = Committed(S_a, receipt?, version facts)
on failure:       atomic state is NOT changed by this attempt
                  input S_b remains with/is returned to caller
                  no S_a or success credential is produced
                  result = Rejected(S_b, actual observation if any)
```

`A/B` 是原有和 Pattern 分支；`I_a/I_b` 是相应分支中实际可比较或更新的状态数；`S_a/S_b` 是分支实际持有的资源位置。成功条件要求**当前模式确实为 A**且满足本次提交所声明的 `I_a` 匹配条件，同时满足资源准入、访问、Generation、Color 和生命周期 Pre。仅与某个旧观察数值相等不能证明提交版本身份不变（允许数值 ABA）。

一次成功提交把 `A` 的资源 `S_a` 与其合法持有责任移交给调用方，把调用方提供的 `S_b` 与其责任移交给新状态 `B`，并形成新提交事件版本；这些都是**同一个原子提交点的契约效果**，不允许先成功机器 CAS 再试着用可能失败的普通 Post 补全所有权。失败不取得 `S_a`，不消耗/遗失 `S_b`，不发布成功凭证。结果是一般语言已经支持的 `Committed(...) | Rejected(...)` 和模式材料，生命期 Post 分别由实际分支建立；回执是普通受控仿射对象而非新资源本体。`Committed` 分支还须承担对其新事件的真实因果描述。

原始 CAS 的角色可以按原有 Pattern 材料及普通关联 API 做以下**不需要新原语**的变体：

| 受控转换 | 实际语义 |
|---|---|
| 没有 `I_a`/`I_b` | 仅用当前分支模式及其他合法条件判定转换；不虚构状态数字段 |
| 没有 `S_a` 或 `S_b` | 成功时只交付实际存在的资源；另一侧可以为空；不能伪造空缺资源 |
| `A/B` 资源类型相同且选择保持旧资源 | 通过受控 `cas_keep` 仅改变模式/状态数，使**同一** `S` 保持原位；类型相同本身不推出对象身份相同 |
| `A=B`，更新 `I` | `update_id` 等 API 在同一模式下进行原子 RMW/条件更新；不取出 `S`、不创建第二份所有权 |
| 仅观察 `observe/peek` | 取得内存模型允许的当前状态/版本观察；既非模式转换也不交付受保护资源 |

便捷 API 可以组合这些基础能力，但不得绕开同一实例上的提交竞争。`cas_keep` 与 `cas_take` 必须竞争同一个原子状态；绝不因为保留 `S` 就有另一条不受 `take` 约束的计数修改路径。原子状态资源不应通过普通结构提取直接被夺取，见 N35–N37。

**结果的和模式转换还有真实的因果效果。** 成功/失败结果经已有和 Pattern 提取进入实际不同分支，其后只执行被选分支的续体和业务/生命周期动作。分支中已经建立的原子版本观察、提交事实和由此取得的因果过去进入该分支的 `C_K`；随后的原子转换将该执行路径已经建立的因果事实发布到它的新版本中，后来的真实版本观察再继承这些事实（N41）。没有发生的分支、失败时未发生的提交、仅相同的状态数都不能提供这条因果来源。

强 CAS 因实际状态不匹配而失败，若确定实际观察了哪个已提交版本，可按普通失败分支导入其 `rf/Past`；弱 CAS 的虚假失败标记不推出一个具体 `rf`。即使同一种结果 Pattern 的多个来源合流，也仍遵守 N15 的相关来源选择，不恢复已失的分支精度。已封印调用的实际竞争失败不重新选择另一个候选。

### N40 — 一般安全组合与外部事实的信任边界 [不变量]

ARC、Mutex、Mailbox、队列和事务算法尽可能由普通调用、和模式分支、契约、资源交接及 atomic 材料组合表达，不逐个增加与 atomic 同级的受信任内建。

安全组合可递归分解实际 Pre/Post 义务，但不得据此宣称任意目标协议可自动证明。支持族之外的证明缺失不得用一条便捷语言规则掩盖。

原子提交、特定宿主原语及设备/FFI/寄存器等无法从语言内部推出的事实由明确受信任基础负责；额外真实成功事实可用兼容的 unsafe post。没有 unsafe pre，不允许从 `check=true`、OpenHere 或一个普通模式标签创造授权。

### N41 — 分支因果闭包、版本发布与所有权同步 [定义及不变量]

不复用 `K` 表示“已知事件集合”，以免与唯一的共同语义续体混淆。记：

```text
mo_a       : atomic instance a's committed-version modification order
rf(r) = v  : observation r actually read committed version v
Past(v)    : causal history carried by that committed version
C_K(s)     : causal knowledge actually established at K position s
```

真实语句、已选择的和 Pattern 分支、循环迭代、调用、返回和合法续体交接建立该执行路径的因果历史。未执行分支、未来预测、相等数值、源代码布局、编译器调度和缓存命中不建立已发生的业务/内存事件。

最小组合规则是：

\[
\begin{aligned}
C_K(s^+) &\supseteq C_K(s^-),\\
rf(r)=v &\Rightarrow C_K(s^+)\supseteq Past(v)\cup\{v\},\\
\operatorname{Commit}(a,v_b) &\Rightarrow Past(v_b)\supseteq \operatorname{Publishable}(C_K(s^-)).
\end{aligned}
\]

其中 `Publishable` 仅是从该执行路径已实际建立的因果事件事实取得可发布部分的数学说明，不是新的业务 Object 或用户可操作的任意证明导入接口。

CAS 成功若在其真实原子事件中观察旧版本 `v_a`，新版本 `v_b` 的因果过去还必须包含对应的旧版本及其 `Past(v_a)`。强 CAS 的实际不匹配失败在能够确认所观察版本时增加失败分支的因果知识；虚假失败不自动获得这样的知识。若在 `Committed`/`Rejected` 和模式转换后，路径所知事实不同，则各自的下一个原子提交发布各自的实际因果历史；另一线程**只在真正观察到该新版本时**继承它。这里保证的是 `rf` 与实际续体因果的**传递闭包**，不是所有独立原子对象上事件的全局总序。

已知对象 `a` 的已提交版本 `v` 后，后续对 `a` 的合法观察不能在 `mo_a` 中退回严格早于 `v` 的版本。普通 `peek` 不承诺真实时间最新版本。生命周期 `origin`、原子 `rf`、历史存储来源、Region 和 Color 互不替代；因果知识不自己发放资源权限。

**所有权系统能推出 CAS 资源交接所必须满足的同步义务**：在同一次 CAS 提交上移交 `S_a/S_b` 时，合法继任持有者对转交资源的访问必须看见契约要求的、在转交前已建立的资源状态。这个从生命周期 Pre/Post 推出的可见性要求是**源语言承诺及后端实现义务**，不是声称所有权类型检查会自动产生机器屏障。目标 ISA/编译器必须实际实现这些顺序要求；绝不能用 relaxed 机器指令产生不满足资源交接保证的代码。

因此语言核心**不保留**要求用户逐次选择 `relaxed/acquire/release/acq_rel` 的操作数或另设相应的源级内存序原语。所有已确认的基本顺序由分支因果闭包与所有权交接义务导出。是否还需某种 SeqCst 级额外约束，只留在 N43/F15，不能静默加入默认全局顺序。

### N42 — 提交线性化不扩大为 peek 的实时最新性 [不变量]

一次成功 CAS/RMW 有该实例的不可分割提交点。普通 peek 只承诺读取模型允许的已提交版本并导入其因果，不默认真实时间最新值，也不因此取得资源访问权。

消息传递最低要求：T0 成功写出 x 的 vx，再写 y 的 vy；若 T1 实际观察 vy 且 `vx∈Past(vy)`，T1 此后不能对 x 观察早于 vx 的版本。`I=1` 数值相等不代替“观察了 vy”。

必须排除自我支撑的凭空值；“因果图无环”或少量模型测试不能单独充当完整 no-thin-air 形式化。完整操作/公理模型及编译保持性见第四部分证明门槛。

### N43 — SeqCst 是否必需及 `fence;` 的保留边界 [未封闭规范义务]

在已确认的基础内存语义中，原子结果分支决定实际控制、资源归属及已建立因果事实；后续原子提交发布本路径的因果历史，实际版本观察继承这些事实。CAS 交接资源所需的可见性同步由所有权/生命周期契约推出、由后端兑现。**除这些保证外，不要求保留独立的 relaxed/acquire/release/acq_rel 语言级排序选择。**

现在唯一明确保留的排序设计问题是：**是否仍需要 SeqCst 意义的更强跨对象顺序，以及如果需要应由什么最小机制表达。** `fence;` 是无操作数、独立于单次 CAS 的候选表面形式，而非已经证明完整公理的必备源级原语。若最终采用，它不能创建资源凭证、无中生有地引入其他线程 `rf`，也不能用一个抽象全局顺序代替真实因果与来源证明。是否需要全局 SC 域、何种局部/累积约束，必须先完成 F15 再固定。

原方案的双边 SB（两个执行流分别 `write X; fence; read Y` 与 `write Y; fence; read X`，两读均为初始版本）可作为**考察额外 SeqCst/fence 价值的候选验收实验**，但在这项语义决策之前，**不是语言已承诺排除的结果**。单边 SB、IRIW、2+2W、R/R、R/W、W/R、W/W 与 fence 的累积性均须根据最终选定的规则判定；不能借助某 ISA 偶然提供的更强指令倒推源级保证。

初始域只包括普通缓存一致共享内存。DMA/MMIO、持久化、设备内存及非一致共享内存需要独立的外部世界对象/域协议。任何选定的额外排序设施都需要真实机器/IR lowering 证明，而不是仅有编译器优化屏障。

### N44 — 内存核心不包含等待与调度策略 [不变量]

语言核心不加入乐观锁、自旋锁、悲观锁、阻塞等待、退避、自动重试或线程调度协议。核心原子操作止于一次实际尝试及其普通结果分支。

分支可由普通代码发射不同 effect；外部 handler 决定如何调用、延后、交给异构主体或丢弃具体续体。策略可以比固定的“锁种类”更细，也可以动态组合。

如果提供 `update_id` 等会重试的便利 API，它是有显式普通续体/分支/effect 行为的库程序，不是隐式重试内建。每次再次尝试是新事件，不重开已完成 CAS，不复制一次性凭证，不把失败前副作用撤销。

async 与异构执行正交，异构执行与自动逃逸正交。基础内存模型不依赖某个调度器，实际调度器也不得违反它的事件/访问承诺。无死锁、无饥饿、lock-free、wait-free 和回收进展不由资源安全自动推出。

### N45 — 协议样例与有限检查的证明范围 [证明义务]

ARC 的计数 n 必须等于实际尚未撤销的 ticket 数，包括仍在成功回执内的票据；有效 ticket 关联具体 cell。clone、release、take_last 分别在一次真实成功提交内改变状态和权利；失败不丢票据，上溢不产生票据。

Mutex 的安全样例是通过状态转换唯一交接一般资源，不能由 peek Free 发放 guard。Mailbox 的安全样例是 Empty/Full 的单次资源交接，不是对任意大小 T 宣称一条机器指令搬运。

这些是组合表达力与局部保持性的验收实例，不使 ARC/Mutex/Mailbox 成为语言核心。特定协议公开的转换必须闭合；不能把“内部操作 memory-safe”直接当成任意人为限制子集已经强制。

附件有限模型重跑结果与原输出一致，但只覆盖有限抽象状态。它没有检验完整 Region/Color、源语言消费者、自动 repair、fence ISA 行为、IRIW、OOTA、overflow、Weak、回收或编译正确性。不得把这些空白写成“全部通过”。

## Ordinary transaction and proof boundary

Input is the actual admitted cell/version, selected transition, candidate/frame,
source names, incoming resource custody, Region/Color and established branch
causality. Ordinary registered calls form the declared result Pattern. Pre
checks all those premises and closed resource dependencies before the actual
hardware commit. Committed transfers S_a/S_b and publishes one version/Post;
Rejected retains S_b and supplies only its actual observation. Contract checking
and scratch computation are not a post-CAS rollback scheme. Missing registration,
version/authority, causal or backend consumers are Unavailable, not Rejected.

The [lifecycle owner](lifetime-policy-and-overload-boundary.md) alone defines
Region/Color/origin and repair. The [effect owner](../control-flow/effect-steps-and-continuation-handoff.md)
owns ordinary result branching and scheduling composition. See the
[implementation gates](../../planning/lifetime-concurrency-consolidation.md)
and [paired atomic/memory acceptance](../../planning/lifetime-concurrency-acceptance.md).
