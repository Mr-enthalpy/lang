# Atomic Pattern transitions and causal memory

Status: canonical owner of Atomic admission of **ordinary**
`$`-formed associated sum Patterns, one-attempt CAS and resource/rights
handoff, and causal memory. The direct sum-arm `~~` structural association
belongs to the [Pattern owner](../patterns-overload/pattern-relations-observation-and-extraction.md#92-direct-sum-arm-structural-association-);
Atomic is its stronger trusted execution **consumer**, not its creator.
N34–N42/N44–N50 define the target semantics, F13–F19/source/backends
remain pending, and N43/F15 remains the bounded extra SeqCst decision.
`write/read` here are illustrative ordinary resource member names;
`~~` is the selected association spelling, not an Atomic-only operator.
The selected `->::E` ordinary transform interface is target syntax,
not a connected implementation or a frozen CAS API family.
All equations and conventional call notation below are metanotation; source
sketches are target syntax with unconnected consumers.

### N34 — atomic 消费 `$` 已形成的模式类型 [不变量]

`atomic` 与 `struct` 处于相同的类型材料消费者层次：输入是已经通过 `$` 及既有结构形成规则生成的完整类型，输出注册所需原子操作的完整类型。**其基本语义是受信任的和模式变换**：在合法原子状态实例上，对既有和 Pattern 的某个当前分支执行不可分割的条件状态转换，相关的状态数、受控资源与结果交付一并提交。它不是新的 enum 声明语法、Pattern 类别、和模式构造器或 Object 坐标。

源码位置：

```text
formed_type |> atomic::
```

和模式的结构材料可以直接采用普通 `~~` 关联语法：

```lang
let E = ((
    (Ia1 n1, Ia2 n2) A ~~ (Sa write) |
    (Ib1 m1) B         ~~ (Sb read)  |
    C                  ~~ (Sc write) |
    D
) E)$;

// 目标使用：已形成完整 E 材料再交给 Atomic 消费者
E |> atomic::
```

每个 `~~` 在和模式**直接分支**（含单分支）上将状态
Pattern 与资源 Pattern 结构关联；并未执行提取、移动、
构造或转换。这里的 `write/read` 是各自资源 Pattern 的
普通命名成员，作为独占交付/共享交付的候选协议角色，
仍可再调整命名。没有资源关联的 `D` 是合法分支；
状态数材料也可以在该分支真实缺省。对 `A~~R|B` 的解释
是 `(A~~R)|B`，不允许将 `~~` 下沉到积模式内部，
例如 `(C~~(Sc write),D)` 不合法。两侧是普通 Pattern，
关联证据由 `$` 形成并由 `R_Gamma` 及 Norm 保留；
Atomic 根据所选结构证据判定角色，绝不能根据 bare
Product 第几位、同名 Val2 或一个新 enum 本体猜测。

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
I : optional state-number material inside the selected state arm
    (may have nonpublic components)
S : optional resource from that arm's ordinary ~~ association
    (e.g. named write/read), with real custody
v : identity of a committed atomic event
n/g : ordinary lifetime name and generation
```

`I` 不隐式拥有、解引用或转换成 S。更新 I 不要求搬动 S。相同 P、相同 I、相同业务值都不意味着同一个提交事件或 generation。

`I` 仍是有真实名字/generation 和生产 Post 的普通语言值；可比较、数值相等或状态数角色都不证明 origin=None。它若被状态保留，就和 `S` 一样受 N35 的全部保留状态名字闭合条件约束。

数值 ABA 不被默认禁止；需要“自观察以后绝无改变”的协议必须另外提供版本/世代判定。无需把语义事件身份实现为无限增长机器整数，也不从名字等同性推出物理地址相等。

### N39 — 单次 CAS 原子性、Keep/Exit 与失败材料 [不变量]

对实例 `a`，已依法构造的 `E` 相对变换值 `t`
通过 Atomic 受信任消费者发起一次 CAS；
构造 `t` 时不执行本次 CAS：

```text
attempt(a:E, t:Transform(E))
old actual arm A(I_a?, S_a?)
desired arm B(I_b?, S_b?) after ordinary formation/normalization

successful single commit:
  match actual A with the source evidence required by t
  publish B and its actual State/Resource/permission Post atomically
  if old S_a exits its source slot:
    source resource role write => Committed receives owned S_a
    source resource role read  => sole owned S_a enters lawful Forget;
                                  Committed receives valid S_a share
  if old S_a remains (Keep):
    source write => Committed has no old-resource access
    source read, target read => Committed receives valid S_a share
    source read, target write => Committed receives no S_a share
  deliver actual committed version and causal facts

failure:
  atomic instance unchanged by this attempt
  Rejected returns all *actually formed* incoming new I_b/S_b
  and their unconsumed ownership; only real failure observations
  neither returns old S_a nor mints an unproved share/success receipt
```

If a state/target has no respective I/S role, there is no fictitious
candidate, return or inheritance. `I_b` computed from old material
and never formed cannot be fabricated as a returned name. If a target
omits resource while it actually requires a compatible one, omission
means **identity-preserving Keep** of old S, not a fresh resource or
return of old ownership; incompatible omission is illegal.

For any committed transition e that **actually exits** old S:

```text
ExitSlot(e,S) => ReturnOwned(e,S) XOR ForgetOwned(e,S)
Keep(e,S)     => not ExitSlot(e,S)
ForgetOwned(e,S) => Committed receives a lawful S share
```

The XOR has an ExitSlot premise. `not ReturnOwned` does **not**
imply Forget: a transition may Keep the original S. The
resource delivery role is fixed by its **source arm** and
cannot be selected per API invocation. No two safe
transform implementations on the same source arm may
offer contradictory exit custody. A resource change must
be bound to a true state-Pattern-changing CAS, not only
a Status update or a post-commit ordinary action.

All source/target Pattern, retained-name N35 closure,
Region, Color, origin, generation, ownership and access
Pre are checked at the actual request/commit boundary.
Full source arm+state-number matching is required for
a transition to a **different state arm**, even when
target material is inherited. Only an in-place
same-arm Keep update may use a masked/partial
state-number match (`N48–N49`).
Failed comparisons never consume new target materials
or silently Drop them. Successful/failed result arms
remain ordinary sum values carrying different Post
and N41 causal histories. A value-equal Status does
not establish a committed version or eliminate ABA.

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

### N46 — 普通和模式分支关联与 Atomic 的消费边界 [定义]

一般 `$` Pattern 先产生完整 E，后由普通关联
`~~` 提供每个和模式分支的 State/Resource 结构：

```lang
let E = ((
    (Ia1 n1, Ia2 n2) A ~~ (Sa write) |
    (Ib1 m1) B         ~~ (Sb read)  |
    C                  ~~ (Sc write) |
    D
) E)$;
```

`~~` 是**分支级关联**，不是 `=>` 的生成/提取箭头、
不是一般 Product 运算、不是 CAS 转换或资源移动。
单一关联分支不必为合法性额外构造另一个 `|` 分支；
`(C ~~ (Sc write),D)` 之类嵌入 Product 的式子非法。
即使相同 `Sat`，关联材料也属于完整 Pattern 的
关系/规范化身份，不得被 `Norm_P` 或类型缓存抹除。
完整注册还须具有真正的关联/角色结构证据，
而不是通过普通同名 Val2 字段提升权限。

资源侧 `(S write)`、`(S read)` 的名字
可按语言一般命名语义形成；这里用 `write` 表示
源资源离槽时合法交付 owned S，用 `read` 表示
源资源离槽时合法 Forget 其 owned custody 并
向成功结果交付 S share。两者均**只是暂用命名
约定**，不产生额外 Object 类型、自动回收、
读写锁或 Atomic 私有 Pattern。S 可缺省；
State 分支的 I 也可缺省。数值 Status 可以复用
普通 `Product |> slice` 和 `R_Gamma`：

```lang
let NotZero = ((1u32:) NotZero)$;
let NotOne  = ((0u32 | (2u32:)) NotOne)$;
```

`1u32:` 在 u32 域匹配 1 至最大合法值，
`NotOne` 通过 `0u32 | (2u32:)` 排除 1。
这属于普通 Pattern owner 的数值切片，
不是 Atomic 专用计数器原语、私有 AST、
新类型轴或额外求解器。

普通 `E` 的合法形成与关联不蕴含任何原子执行。
Atomic 仅约束其准入并对实际选中
`Transform(E)` 赋予一次 CAS 提交、受控
不相斥独占 Color、资源与结果 Post 的可信
交接。所有保留 I/S 名字仍须遵守 N35。

### N47 — 源资源角色、离槽与 Keep 的成功权利 [不变量]

每个已关联资源的源 arm 通过自己的
`write/read` 角色确定离槽交付；同一
arm 不允许同时注册两种互斥离槽去向。
所有成功分支依**实际资源行为**判定：

```text
ExitSlot(source write) => Committed receives owned S
ExitSlot(source read)  => Forget receives owned S
                          AND Committed receives S share
Keep(source write)     => Committed does not obtain old S
Keep(source read -> target read)
                       => Committed receives S share
Keep(source read -> target write)
                       => Committed does not obtain S share
```

`Forget` 保持唯一清理责任；它不是立即
删除或某个强制 lazy/ARC 算法。成功返回的
`S share` 必须由真实 Region、Color、origin
及存储有效性支撑；普通 share 不允许隐藏
一般内部可变性。即使新旧两个状态的
资源身份不变，状态分支模式、资源角色
仍须作为真实 CAS 条件/结果解释。

结果权利由**源角色加实际 Exit/Keep 和
目标角色**共同确定，不是 `~~` 自行生成的
提取器。没有 S 的 arm 不虚构资源结果。
从 `not ReturnOwned` 不可推出 `Forget`：
`Keep` 是其余合法情况。

### N48 — 模式变化的完整匹配与共享权恢复 [不变量]

改变和模式 arm（包括 `read/write` 资源
角色变化）必须由一次状态-Pattern-changing
CAS 完成；单纯更改状态数不能改变资源
交付模式。若 Keep 同一 S 但从 read 切到
write，该成功分支**不**产生新 S share。
之后真正取出旧 S 必须另行执行一次 CAS，
不能把第一次成功当成第二次成功。

从 read 状态恢复到 write 时必须在真实
Pre/交接中证明先前全部冲突共享权已经
合法结束且不会继续发放；`n=0`/`n=1`、
数值切片或隐藏状态位本身均不证明这个
性质。具体许可、计数、Region 或外部
受信任设施如何履行该义务属于普通协议。

这里有一个重要的源状态匹配边界：

```text
ChangingArm(A,B) => FullMatch(A, all actual I_a material)
PartialStatusMatch(I_a) => SameArm(A,B) AND Keep(S_a)
```

在原位保持 arm 和资源的 Status 更新中，
可使用局部/蒙版式匹配及增量修改，
未指定的分量由同一实际状态原样保留。
但不能把此操作提升为从 A 到不同 B
状态的部分匹配转换；即使新资源/状态数
目标侧可以合法继承，也必须拥有真实
**完整源 arm+Status 匹配证据**。
不可公开读取的 Status 分量不等于不存在；
受控比较可在未对外泄漏其值的情况下
满足完整匹配，但不能跳过它。

### N49 — E 关联变换值、结构省略与 Atomic 执行 [定义]

已形成和 Pattern E 可通过普通关联的
`->::E` 成员形成**可调用变换值**。下列是
拟议的语法目标（尚未接入源码消费者），
并非立即执行 CAS，也不是预先固定的一批
`cas_keep`/`cas_take` API：

```lang
let transform =
    ((val1 n1::A::E, val2 n2::A::E) A::E)
    (->::E)
    ((val3 B::E) ~~ newSource);

let same_arm_delta =
    (val1 n1::A::E) (->::E) (val2 n1::A::E);
let same_arm_unchanged =
    (val1 n1::A::E) (->::E) ();
```

`->::E` 应是**实际注册、可选择的普通
关联可调用对象**；调用者先构造变换值，
后由使用者选择何时对实例尝试。普通 E
只负责合法的关系/变换材料；Atomic 的
受信任消费者才赋予 CAS 原子性和相应
Color/权限交接。表面式 `B::E ~~ newSource`
构成 E-relative **关联目标臂材料**，
不是 `~~` 自行创建资源或自动运行变换。
目标省略和左侧蒙版匹配遵守 `N48`，
所有合法性由已选择的普通操作检查。

省略规则以目标**实际需要槽位**为前提：

- 目标 B 及新资源都缺省时，沿用源 arm，
  不允许同时显式替换资源。
- 目标 arm 存在而新资源省略，仅在目标
  需要该资源且与真实源资源兼容时 Keep
  同一 S；不兼容拒绝构造变换。
- 目标需要 Status 而新 Status 省略，则
  继承真实源 Status，要求其满足目标
  Pattern；不兼容拒绝。目标根本无
  Status/资源时不存在所谓“缺省继承”。
- 左侧状态数部分省略表示蒙版匹配，
  **仅**同一 arm、Keep 原资源的原位
  状态数更新允许；改变 arm 必须全匹配。
  Status 可由普通合法计算增量更新，
  并可保留不可公开读取分量。
- 若提供新的资源 S_b，替换必须与
  真实状态 arm 转换同一原子动作绑定，
  不能在状态未变化时仅替换旧资源。

普通 E 的 `?` 可作为已注册普通观察
接口的结构模板：复制**合法可读**的
分支/Status 投影，不会因一条隐藏字段
而让整个观察报错；不可读取的分量
没有在投影中解包。对 Atomic 实例，
完成的一次快照必须对应**一个实际一致**
状态观察，而不是从不同提交拼接字段。
这一观察不自动给出 S share、完整
隐藏 Status 的匹配证据、版本永不改变
事实或资源所有权。`?` 仍服从原有
提取/调用 Policy 和来源规则。

对真实实例的后续使用可写成目标形状：

```lang
e transform ?
  |> (self, data success) { ... }
  |> (self, data failure) { ... }
```

这只是一次变换**尝试**和普通结果
和 Pattern 分支交付；成功/失败及
其真实资源输入/输出由 N39/N47 定义，
未宣称当前解析器已有此执行支持。
具体签名、隐藏 Status 操作和机器
实现仍由消费者与后端验证，而不
新增第二种原子变换语言或静态求解器。

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
