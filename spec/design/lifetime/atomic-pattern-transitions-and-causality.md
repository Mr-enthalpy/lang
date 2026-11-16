# Atomic Pattern transitions and causal memory

Status: canonical owner of atomic registration, ordinary Pattern role
interpretation, one-attempt CAS/resource and access-right handoff, and causal
memory. N34–N42/N44–N50 define the semantic boundary; F13–F19 proof,
source-consumer and backend obligations remain pending. N43/F15 remains the
bounded extra SeqCst decision. `Status`, `own` and `share` below are
provisionally fixed **structural member-name conventions**, not new
keywords or Atomic-only Pattern syntax. CAS API names and variants are not
frozen. No executable Atomic API or cross-ISA support is claimed.
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
P_Cell(T) = Empty | Live(uint32 Status, Stable(T) own)
                  | Readable(uint32 Status, Stable(T) share)
Cell(T)   = atomic(Form$(P_Cell(T)))
```

以上不是传统 `f(args)` 或已实现的完整源码样例，而是从
已有 `$` 形成的和 Pattern 的结构示意。每一个分支里
`(I Status, S own)` 或 `(I Status, S share)` 的两个成员都**有名字**，
故本层 Product 按原有规则为无序结构；`own/share` 表示两种
不同的候选成员名，不是斜杠运算符或第三个裸标记。
`I Status` 和 `S own/share` 各自可以缺省：没有 I 就不虚构状态数，
没有 S 就没有可交付的资源。其他普通结构分支可由 Pattern 自己
细化。Atomic 消费者以实际 Pattern/结构证据确定唯一合法的角色，
而不是靠 bare Product 的位置、普通同名 Val2 成员、Type 相等或
不可验证的字段拼写猜出身份。其受信任语义是对已准入状态进行
一次真实和 Pattern 条件转换；没有新的 enum、Object 轴或第二套 E/K。

输入模式可以结构化复用既有 atomic 类型和普通语言材料，形成合法递归组合。一次一般模式构造/提取的“原子完整提交”不自动等于跨执行主体的机器 CAS。

### N35 — Atomic 所有保留状态名字均须对外来源闭合 [不变量]

atomic 不无差别接受所有类型材料。对实际状态 A 及其 atomic 资源边界，准入条件覆盖全部保留状态名字：

```text
for every n in RetainedStateNames(A):
    ExternalOriginDeps(n, atomic resource boundary) = empty
```

`RetainedStateNames(A)` 是实际状态存储、保留或继续依赖的普通值名字及其真实保留依赖闭包的数学记号，不是新内建集合、Region/K 本体或证明 Object。它包含状态数/Index `I`、资源 `S` 及其实际保留组成，不能按 `I`/`S` 角色或业务类型免检。该条件对应真实来源与所有权，而不是看到 Rust 风格 `Owned<T>` 名称就接受；注册须约束全部准入状态，初始形成及每个状态转换 Pre 必须维持它，包括将被保留的 `I_b`、`S_b`，以及 keep/同态 RMW 后仍保留的全部名字。

例如，可信 Index 可以保持普通 Index 业务类型而带有对外部聚合的 origin。若 CAS 将这个名字作为 `I_b` 保留，其外部依赖就必须接受闭合检查，不能因它是“状态数”而被抹去。调用者可以通过一个已经合法选择并实际完成的普通复制/形成动作取得真正无外部依赖的新状态数名字，再将该名字交付给 CAS；其生产 Post 必须证明闭合。平凡 Index copy 的新名字可以 origin=None，但不带原凭证，这不是所有 copy/clone 的普遍无来源定理，也不是 atomic 的隐式 copy、擦除、回退或重选机制。

只在一次比较/匹配中读取且未被结果状态保留、也不成为其依赖的临时输入，不仅因参与 CAS 就加入 `RetainedStateNames`；它仍须满足真实访问、来源有效性和普通 Pre。比较输入是否被保留取决于实际选定操作及交付对应，不能只靠 API 参数名判定。

它不等于所有内部 origin 都为 None；内部合法结构关系仍可存在。但隐藏外部借用的 unknown 不等于没有外部依赖。不能用名字遗忘洗掉该准入要求。

该限制使 atomic 的全部保留状态材料不需要依靠外部挂起点的自动 repair 来恢复借用关系；atomic 事务不引入“重演旧 CAS 以修复内部资源”的特殊机制。闭合检查在真实硬件提交前完成，失败不能先安装 `I_b`/`S_b` 再修复来源。外部持有的 Cell ref/share 及凭证仍是普通依赖名字，遵守自己的交接规则。

无论资源槽位命名为 `own` 还是 `share`，其实际保留的 I/S
名字都须满足这条闭合条件。成功 CAS 返回的 `S share` 是有真实
origin/Region/Color 的**输出借用**，不是额外的资源所有者，也
不能凭槽位名称抹除任何外部依赖。旧资源的 Forget 接收者
只承接确实移交的唯一责任，不能因“名字被遗忘”取消仍有效
的借用、storage support 或 Drop。

### N36 — 受信任的重叠独占染色只覆盖注册协议 [不变量]

atomic 产生的类型提供其原子访问域的受信任兼容关系，允许多个独立执行主体对同一实例具有重叠独占染色。

异构容器仍在入参/捕获传入 Pre 检查存活 Region；只是在 Color 兼容判断上消费这个类型已经提供的保证。容器无需认识“Atomic 是特殊线程类型”，atomic 也不内建 Thread 规则。

该保证不自动授予内部一般资源的重叠独占写，不替代稳定
资源借用的独立 Pre，不让任何普通字段访问绕过原子协议。
`S share` 一旦依法交付，只有普通共享读 Color，不能塞进
一般内部可变性；需要改变受管辖资源则必须另走真实、合法、
不会与既存 share 冲突的资源权限交接。注册的所有 CAS 操作
共同维持这一安全条件，不能要求使用者自行约定“不去偷”。

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
I : optional ordinary Status role (may have nonpublic components)
S : optional held resource in the own/share role
v : identity of a committed atomic event
n/g : ordinary lifetime name and generation
```

`I` 不隐式拥有、解引用或转换成 S。更新 I 不要求搬动 S。相同 P、相同 I、相同业务值都不意味着同一个提交事件或 generation。

`I` 仍是有真实名字/generation 和生产 Post 的普通语言值；可比较、数值相等或状态数角色都不证明 origin=None。它若被状态保留，就和 `S` 一样受 N35 的全部保留状态名字闭合条件约束。

数值 ABA 不被默认禁止；需要“自观察以后绝无改变”的协议必须另外提供版本/世代判定。无需把语义事件身份实现为无限增长机器整数，也不从名字等同性推出物理地址相等。

### N39 — 单次 CAS 的原子性、资源去向与失败材料 [不变量]

最基本的 CAS 是作用于一个实际 Atomic 实例的、已形成和 Pattern
分支上的单次条件转换。它同时约束实际模式、可选 Status、可选
资源和由结果建立的权限，而不是仅比较整数。采用语义元记号
（不是已经冻结的 CAS API）：

```text
attempt(a, A, proposed I_b?, proposed S_b?, selected ordinary transition)
old actual state:  A(I_a?, S_a?)

on successful one-point commit:
  actual state becomes selected B(I_b?, S_b? or preserved S_a)
  if S_a actually leaves the old resource slot:
    source role own  => transfer owned S_a to Committed result
    source role share => transfer sole owned S_a to lawful Forget receiver
                         AND deliver lawful S_a share to Committed result
  if S_a remains in the slot:
    Keep(S_a); no ExitSlot transfer; share result only if this operation
    actually admits/establishes such a borrow
  publish actual version and branch-dependent Post

on failure:
  state a is unchanged by this attempt
  Rejected(proposed I_b?, proposed S_b?, actual observation if available)
  do not consume, silently Drop or lose either already-formed input;
  no S_a, minted share or success credential is delivered
```

`?` above denotes a genuinely optional role/input, not implicit null storage
or a new syntax. In particular, if a proposed Status is formed by internal
computation from actual retained state, a failed attempt preserves/returns
the ordinary still-owned inputs and any already-formed unused candidate;
it cannot synthesize a Status name that was never actually formed.

For every committed event e involving a real old source S:

```text
ExitSlot(e,S) => ReturnOwned(e,S) XOR ForgetOwned(e,S)
Keep(e,S)     => not ExitSlot(e,S)
ForgetOwned(e,S) => Committed contains a lawful S share
```

The XOR has the **ExitSlot precondition**: `not ReturnOwned` alone does
not imply `ForgetOwned` because `Keep` is another legal resource
behavior. `own/share` is fixed by the **source Pattern's resource slot**,
not selected afresh by each CAS operation. The same source state cannot
expose both owned-return and Forget-on-exit safe CAS variants.
An actual new resource belongs to the target state only on a successful
commit; all resource/Status candidates offered by a failed attempt are
returned in its Rejected branch, not silently destroyed. Both branches
are ordinary sum Pattern results with their own real Post.

Success checks the actual A branch and selected Status/Pattern conditions,
destination N35 closure for every retained I/S name, and the real Region,
Color, origin, access, generation and source/ownership Pre **before**
the commit. Neither a successful machine CAS followed by a fallible
ownership transfer nor an ordinary Post attempting to repair absent
authority is admissible. `Status` is ordinary value/name material:
a number alone cannot establish event version identity or permission,
and equal numeric values do not exclude ABA. Only the actually selected
committed or rejected branch establishes its causal observations (N41);
weak/spurious failure does not invent a committed version.

### N40 — 一般安全组合与外部事实的信任边界 [不变量]

ARC、RCU、GC、Mutex、读者登记、版本共存、读写排斥、按需 clone
与延迟回收都可以是普通 Pattern、CAS、Pre/Post、资源交接和
调度/effect 构成的**工程协议示例**，不应升格为 Atomic
本体或强制受信任库。计数有时协调写入资格，有时管理资源
回收；其存在、用途和维持方法都取决于具体协议。
语言提供安全组合的必要权限边界，而不事先选择工程实现、
回收结构或是否等待/重试。

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

任何实际由 CAS 成功分支交付的 `S share` 必须观察到其
已发布且合法存活的资源状态，满足来源、Region、Color 以及
rf/Past 所要求的初始化可见性；普通裸 peek、相等状态数和
原子指针载入均不能凭空取得借用。若某个库协议允许旧读者
与新版本共存，旧借用可在自己的合法 Region 内继续读取；
若要求写后旧读权失效，写入前必须完成合法的读者协作。
这两种工程选择都不赋予普通 share 内部可变性，
也不让 peek 承诺实时最新状态。

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

### N45 — 工程协议案例与有限检查的证明范围 [证明义务]

已有 ARC 票据/计数、Mutex 和 Mailbox 有限模型是检验
组合表达力的**特定工程案例**，不要求语言提供任何一种
通用 ARC、RCU、GC、读写锁或 HeapStack 退休协议。
若某个库选择用计数协调读写，必须自己证明所计对象与
实际读取凭证的对应；若用于回收，则须证明最后一个合法
引用/借用终止之前不执行失效 Drop。`n=1` 仅是普通
状态数事实，不能直接推出唯一存活读者或已经取得独占权。
已有有限模型不构成对所有 Region、Color、来源、版本、
机器后端、复杂回收或一般库协议的证明。

### N46 — 由普通 $ Pattern 给 Atomic 提供 Status 与资源角色 [定义]

`atomic` 消费 `$` 已形成的完整和 Pattern 类型。约定使用
本来合法的命名成员，而不是让第零个 bare 项自动成为状态数、
第二项自动成为资源：

```lang
// 结构/Pattern 形成示意，具体 atomic 注册消费者尚未接通
(I Status, S own)
(I Status, S share)
(I Status)        // 无资源
(S own)           // 无状态数
(S share)         // 无状态数
```

同一资源槽位的 `own` 和 `share` 是不同的命名结构角色：
它们并非斜杠算子、额外可见资源、第三个裸标记、
新关键字或第二套 Pattern 语法。所有直接成员都命名时
这一 Product 层按原有规则无序；包含 bare 成员则遵守
原有有序规则。S 上的普通裸 `name` 与 `name$` 粘贴值
仍按一般极性/Val1 语义解释，不可因 Atomic 角色而改写。
其他字段及更复杂模式通过真正的普通 Pattern 分支表达。
缺省 I/S 只消去对应的状态数/资源职责。

Atomic 必须从实际结构注册与 `R_Gamma` 推导中确定
`Status`、`own/share` 的唯一角色观察（或证明所有成功
推导所得观察相同），不得由同名虚拟 Val2 成员、类型
相等或位置猜测赋予能力。`Status` 是普通可计算的值材料，
可以复用普通 Pattern 的数值切片，例如：

```lang
let NotZero = ((1u32:) NotZero)$;
let NotOne = ((0u32 | (2u32:)) NotOne)$;
```

上面 `:` 仍经现有 RHS ordinary `Product |> slice` 形成；
在 u32 域内 `1u32:` 覆盖 1 至最大合法 u32，
`2u32:` 覆盖 2 至最大合法 u32，因此 NotZero
排除 0，NotOne 排除 1。既有普通 Slice/Pattern
消费者须提供域边界、端点和匹配证据；不新增
Atomic 专用范围、独立 Slice AST/本体或第二个
Pattern 求解器。数值溢出、空范围和缺证据仍由
实际普通计算/匹配与 Pre 处理。

### N47 — 槽位 own/share 的条件离槽交付契约 [不变量]

每个有 S 的**源状态 Pattern**固定其资源槽位的
`own` 或 `share` 契约。同一源状态不得同时注册
安全的“取出旧 owned S”和“Forget 旧 S”两种离槽
方式；需要不同业务分支时，应在 Atomic Pattern
状态结构里显式区分，不将协调状态机塞进 S 内部。

```text
actual CAS success with S leaving source slot:
  source S own   => result obtains owned S
  source S share => sole owned S goes to lawful Forget responsibility
                    AND success result obtains lawful S share
actual CAS success with S remaining in source slot:
  Keep(S)        => no owned exit; share issue only if operation
                    explicitly forms a lawful share
```

`Forget` 不是关键字、自动删除、固定 lazy-free 队列或
无主资源。它是与同一个单点提交绑定的、实际有合法
所有权承接者的保守遗忘/析构责任交付；由于成功结果
同时得到 `S share`，其真实存活/Region/来源及
合法 Drop 窗口必须继续成立。`Keep` 不是第三个
离槽去向，而是根本没有触发离槽前件。
从 `not ReturnOwned` 不可推导 `Forget`。
没有 S 的分支不得假造 owned S 或 share。

### N48 — 转换访问模式与活共享权的证明 [不变量]

由于 `own/share` 是**源 Pattern 中与槽位绑定**
的角色，改变这一契约必须真实切换 Atomic
状态 Pattern；单纯改变 `Status`（包括增减状态数）
不能使原槽位从 share 偷渡到 own，反之亦然。
模式切换即使保持 S 及 Status 完全不变，也需
单独的一次合法 CAS；之后想实际取出 S，
必须再发起另一次重新检查当前状态的 CAS：

```text
A(I Status, S share)
  -- CAS: change Pattern, Keep(S) --> B(I Status, S own)
  -- a separate CAS: exit old S --> Empty or another legal state
```

`own -> share` 在真实来源与读取准入成立时通常是
放宽可交付的权限；`share -> own` 则需要
证明不存在仍存活且与随后独占取出冲突的
合法 share，以及新读取资格的创建路径已被
同一原子状态转换关闭。`n=1`、`n=0` 或
`NotZero/NotOne` 的匹配**单独**不能建立
实际共享许可的完整性或最后归还事实。
计数协议可以用相应凭证/Region/续体 Pre/Post
证明这一点；也可能需要确有根据的外部受信任
成功事实，但不能由一个 `unsafe` 标签或
`check=true` 直接凭空建立独占权。
一切安全 CAS 变体必须共同保护此不变量，
而非寄希望于协作者不调用危险入口。

### N49 — CAS 变体所需的机制空间，非预先规定的 API [定义]

在先不固定 `cas_keep`、`update_id` 等具体 API
数量、名称、参数顺序与结果形状的条件下，
普通注册接口需要有能力表达下列**独立要求**：

- 模式与普通 Pattern 条件匹配，包括数值切片；
  实际分支和当前版本只由真实 CAS 观察证明。
- 模式可改变而 Status/S 原样保留；Status
  可保持、替换或经合法普通计算增量更新，
  不因计算改变 `own/share`。
- Status 可含不可公开读取的普通分量；
  注册操作可以根据真实权限对其受控匹配、
  增量更新或原样保留。公开观察/调用端义务
  只请求对应可读分量，不能因隐藏而使整个
  状态数不存在；也不能让未经证明的隐藏
  部分免于内部真实条件检验。
- I 或 S 可缺省；资源可原位 Keep、合法
  离槽或由新 S 替换；共享读取资格由
  **声明并实际准入**的成功操作发放，
  而不是所有 Keep 操作自动产生 share。
- 成功资源出槽由**源状态**的 `own/share`
  固定交付方式，失败将本次已形成的
  拟写入 Status 和拟写入资源及其仍有的
  所有权返回 Rejected 分支，不隐式析构、
  遗失或假造未形成的值。
- 针对同一 Atomic 实例的全部修改/Keep/
  取出/Forget 转换在同一实际提交竞争
  关系中；额外代际/版本/凭证、等待和
  重试策略仅由确有证据的库/用户机制定义。

这是**机制需求清单**而非一份预先设计好的
受信任 CAS API 家族。精确受控入口的可用性、
匹配资料、候选输入与失败返还 Pattern，
以及所需硬件实现是后续接口设计/验证工作。

### N50 — 固定安全证明义务，开放具体方法 [不变量及证明义务]

任何依法发放的 `S share` 都必须真实具有
origin、Region、Color、版本可见性和支撑
其整个合法读窗口的资源有效性；share 不得
包含一般内部可变性。`share -> own` 要证明
此前发放且冲突的读权均已终止，不能只
凭原子计数的某个数值。资源离槽/Forget
须在提交点完成唯一所有权与合法 Drop
责任的交接，结果 share 不得立即悬空；
同一次 CAS 失败须保持所有拟写入状态数/
资源材料及对应责任。全部保留状态名字
仍满足 N35 来源闭合，现有权限/Region/
Color/因果与 N41 同步要求照常成立。

这些是**固定的证明义务**，不是强制的
RCU/ARC/GC/Mutex/reader guard、特定
Boxlike、Clone/写时复制或 HeapStack
回收算法。允许旧读者跨新版本存活的协议
必须保证其读取/Drop 的延迟合法性；
要求写前撤销旧读权的协议则必须真实
协作完成写入资格的交接。计数若用于
协作，必须证明它正确追踪全部活读资格；
若用于回收，必须证明回收条件；二者都
不是语言默认含义。安全原语和普通
生命周期表达式/库可以承担这些证明，
需要的外部设施可通过相容的受信任成功
事实提供，但不能形成无条件的权限来源。
缺源/消费者/证明为 Unavailable；已知
前提为假在真实 Pre 拒绝，不能事后修复
已失败提交。F13/F14/F16/F18/F19 仍分别
负责相应单点事务、内存、组合及证明
消费者，F15 只保留额外排序选择。

## Ordinary transaction and proof boundary

Input is the actual admitted cell/version, selected Pattern transition and
ordinary expression material, proposed Status/resource names (when present),
current resource custody, Region/Color/source and causal observations.
The single committed result applies the source state `own/share` exit
disposition if S actually leaves; a kept S remains held and shares are
issued only by operations whose actual Pre/commit/Post justify them.
Any successful source `share` exit delivers a lawful borrow to the
Committed result while giving old S ownership to lawful Forget custody.
A Rejected result returns the proposed newly formed Status/resources
and their unconsumed duties, never silently Drops them or fabricates a
successful old-resource claim. Checks and complete ownership handoffs
occur as one admitted atomic operation, not via fallible post-CAS
rollback. Missing Pattern roles, Region/Color, proof, causal or backend
consumers are Unavailable, not a fabricated Rejected branch.

The [lifecycle owner](lifetime-policy-and-overload-boundary.md) alone defines
Region/Color/origin and repair. The [effect owner](../control-flow/effect-steps-and-continuation-handoff.md)
owns ordinary result branching and scheduling composition. See the
[implementation gates](../../planning/lifetime-concurrency-consolidation.md)
and [paired atomic/memory acceptance](../../planning/lifetime-concurrency-acceptance.md).
