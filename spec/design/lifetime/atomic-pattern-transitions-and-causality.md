# Atomic Pattern transitions and causal memory

Status: canonical owner for admitted Atomic transactions, resource custody
and causal memory. Source and backend consumers remain pending. E evaluates
ordinary Pattern formation, transform selection and lifecycle contracts.

### N34 — atomic 消费 `$` 已形成的模式类型 [不变量]

Atomic 消费通过[普通递归命名 Pattern](../patterns-overload/pattern-relations-observation-and-extraction.md#92-recursive-named-pattern-material)形成的完整 E：

```lang
let ReadStorage = ReadBox read::atomic::;
let WriteStorage = WriteBox write::atomic::;
let E = ((
    ((ReadStorage$ resource) (I1 state1, I2 state2) A) |
    ((WriteStorage$ resource) (I3 state3) B) |
    ((I4 state4) C) |
    D
) E)$;
```

这里 `ReadBox` 和 `WriteBox` 是已形成的完整 `T Box` 分类器。
前两次调用按普通 V 解释形成完整包装类型。`ReadStorage$` 和
`WriteStorage$` 在 S 中读取这些类型值。各分支中的 `resource` 是
结构名，协议由 N46 的形成证据确定。

每层保留实际结构和名字对应。Atomic 由真实形成的类型包装和结构
证据识别状态、Status 和资源。分支 I/S 均可缺省；实际保留名字遵守
N35。完整 Object 的构造/提取提交与跨执行主体的 CAS 是不同契约。

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

源资源不论包装协议都受 N35 约束。成功交付的 S share 具有真实
origin/Region/Color；它保持现有回收责任，不构成第二份所有权。

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
S : optional actual T Box under an admitted read/write type wrapper
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
    source resource role read  => discharge slot custody under existing lazy duty;
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

ExitSlot commits exactly one disposition. A write source delivers owned S
under ordinary exclusion/transfer. A read source discharges the slot's
custody under its allocation-time lazy duty and delivers a valid S share.
Keep preserves actual resource identity and uses the source/target rights
matrix above. A resource-free source supplies no old-resource right.

Check source/target correspondence, N35, Region, Color, origin, generation,
access and ownership at the real commit boundary. Use N48 for full or
permitted partial matching. Rejected retains formed candidate material
and remaining duties. Committed and Rejected publish only their actual
causal histories. Equal Status does not rule out ABA.

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

双边 SB（两个执行流分别 `write X; fence; read Y` 与 `write Y; fence; read X`，两读均为初始版本）可作为**考察额外 SeqCst/fence 价值的候选验收实验**，但在这项语义决策之前，**不是语言已承诺排除的结果**。单边 SB、IRIW、2+2W、R/R、R/W、W/R、W/W 与 fence 的累积性均须根据最终选定的规则判定；不能借助某 ISA 偶然提供的更强指令倒推源级保证。

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

### N46 — 普通结构、真实类型包装与表示预算 [定义]

令 `m` 为 read 或 write，`Family_m` 为标准库 Path
`read::atomic::` 或 `write::atomic::` 解析到的实际类型形成家族。
以下为普通调用、完整类型和结构证据的合取，不是 Atomic 私有注册表：

```text
WrapperType(m, B, W, route)
    iff B is the exact complete classifier formed by the ordinary T Box contract
    and ordinary selection of (B, Family_m) commits the declared wrapper type W
    and the selected formation of W proves a structural route to B with mode m

WrapperLeaf(E, node, m, B, W, route)
    iff the formed E registers the actual child at node with LeafSource(child, W)
    and WrapperType(m, B, W, route)

FormedWrapper(s, m, b)
    iff s is the actual resource at such a child of the selected E arm
    and ordinary construction/extraction along its registered route identifies
        b of exact classifier B and preserves its actual allocation/ownership evidence
```

`WrapperType` 的来源是这一标准家族所选调用的成功交付及实际结构。
别名或绑定在普通求值下交付同一个家族或完整类型时，使用该材料已具有的
证据。`(ReadStorage$ resource)` 根据
[普通外部类型叶子](../patterns-overload/pattern-relations-observation-and-extraction.md#external-typed-leaves)
把实际 `W` 作为叶子来源。仅使用裸 `resource` 或同名 `read`，
仅有同型布局，或返回不具该结构对应的类型，均不建立推导。
`FormedWrapper` 检查实际资源，不能由仅有的类型规格虚构一个 Box。

read 入口另消费 [HeapStack N28](heap-stack-and-storage-responsibility.md)
对同一 Box 分配及世代建立的 lazy 责任。普通带所有权转换可检查实际句柄，
依法构造 read 包装并保持义务；转换失败保持原资源责任。表示标签只是供
所选普通检查验证的材料。无资源状态合法。具体包装/检查消费者仍待接通。

I 可以用普通数值 Slice，含非公开分量。Atomic 要求实际 Status 能
平凡移动、析构及合法原子比较。一次条件提交的逻辑表示至多两个
目标机器字，预算覆盖实际判别位、模式、比较状态和 Box 链接。
可压缩时使用单字；双字是准入上限，不保证目标具有双字 CAS。
没有原生操作时，需要经验证的等价整体提交表示，保持比较、结果
权限及 N41 因果；缺少该证明或平台消费者时不可用。超预算材料不
满足当前 Atomic 准入。规范不规定具体布局、后备算法或进度保证。

### N47 — 源资源角色、离槽与 Keep 的成功权利 [不变量]

| 实际动作 | Committed 的旧资源结果 | 源槽位与清理责任 |
| --- | --- | --- |
| write Exit | owned S | 在真实独占交接中转移所有权与相应义务 |
| read Exit 或同 arm read 替换 | valid S share | 原子解除旧槽位持有资格，分配期 lazy 责任继续有效 |
| write Keep | 无 | 槽位保留 S |
| read Keep → read | valid S share | 槽位保留 S，继续读协议 |
| read Keep → write | 无 | 保留 S，按 N48 关闭旧读资格 |
| 无源资源 | 无 | 无旧资源责任可交付 |

`Forget_read(oldBox)` 的运行时回收动作是 NoOp。解除槽位持有资格
与交付 share 属同一成功 CAS。已登记的 Drop 续体及其最终释放责任继续承担唯一
清理，并覆盖新交付的 share 及其他有效读者的实际 Region、origin、
Color 和存储有效性。这里没有提交后接收人搜索、动态登记或重复
析构；普通 share 仍只授予证明过的读权限。

### N48 — 完整比较、原位替换与共享权恢复 [不变量]

真实 arm 或协议变化需要匹配完整源 arm 与全部实际 Status，包括
非公开部分。源值只能通过受控比较观察时仍须完成该比较。继承
目标材料不减少这个义务。

同 arm 可以部分匹配和更新 Status，并保留未指定的实际分量。
同 arm read 可更换 Box，条件是同一提交比较确认实际旧 Box 身份，
实际旧 share 仍有效，且新 Box 具备真实 read 包装与分配期 lazy
义务。成功交付旧 share 并解除旧槽位资格；合法比较失败归还新 Box。
同 arm write 不得更换 Box。即使新旧 Box 的业务类型相同，write
更换也要求真实跨 arm、完整源比较及相应独占交接。

read → write 的 Keep 必须证明此前全部冲突读权结束并已关闭旧读权
发放。计数值和切片本身不能证明这些条件。该提交不交付 share；
真正取出 owned Box 另需一次符合 Pre 的 CAS。若该 Box 在分配期
建立了最终释放义务，后来交付 owned Box 仍须保留该义务和实际
释放所需的对应，不能重复取得析构责任。write → read 同样需要
该实际 Box 已具备分配期 lazy 资格。

### N49 — E 关联变换值、结构省略与 Atomic 执行 [定义]

普通变换值的形成、省略正规化、源匹配义务及一致可读观察由
[普通 callable owner §7](../patterns-overload/operator-patterns-and-generative-declarations.md#7-ordinary-e-relative-transforms-and-readable-observation)
定义。Atomic 只消费真实形成的完整 E、已选普通生产者形成的
可调用 t 及其实际 E/arm/binder 对应；拼写 `->::E` 本身不证明
注册、选择或可调用性，构造 t 也不取得旧资源权限。

目标省略按实际选定构造器和兼容槽位解释。实际 arm 保持时使用
N48 的同 arm 规则；没有兼容目标资源槽位时不能虚构 Keep。普通
变换须提供所需的匹配、保留与交付对应。独占借用上的 Atomic 变更
只通过一次合法条件提交；以值取得的材料仍可走普通拥有值和模式代数。

对实例的一次尝试可写为目标形状：

```lang
e transform ?
  |> (self, data success) { ... }
  |> (self, data failure) { ... }
```

准入先检查真实实例、资源/Color/Region、N35 闭合及所有普通
前提；完整比较与目标提交使用同一原子事件的实际状态。
准入后比较不符（或所声明弱操作的合法虚假失败）才交付
Rejected 及已形成候选。缺少消费者/证明为 Unavailable，
已知准入 Pre 为假则拒绝；二者不能伪装成正常竞争失败。

选中的普通 E `?` 观察始终遵守其单个实际版本/状态的可读
投影契约，Atomic 不增加隐藏字段可读性、完整匹配凭证或 share。
Committed/Rejected 的资源交付由 N39/N47 决定，版本因果由 N41
决定。普通生产者与 Atomic 源码/后端消费者待接线。

### N50 — 固定安全证明义务，开放具体方法 [不变量及证明义务]

任何依法发放的 `S share` 都必须真实具有
origin、Region、Color、版本可见性和支撑
其整个合法读窗口的资源有效性；share 不得
包含一般内部可变性。`read -> write` 源协议切换要证明
此前发放且冲突的读权均已终止，不能只
凭原子计数的某个数值。槽位交接与权限结果在提交点完成，
read 的唯一回收责任已在分配 Post 确定，且覆盖提交后的真实 share。
同一次 CAS 失败须保持所有拟写入状态数/
资源材料及对应责任。全部保留状态名字
仍满足 N35 来源闭合，现有权限/Region/
Color/因果与 N41 同步要求照常成立。

这些是**固定的证明义务**，不是强制的
RCU/ARC/GC/Mutex/reader guard、Clone/写时复制或 HeapStack
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
The single committed result uses the source associated `write/read`
resource role if S exits. A kept S remains held: Keep from write
delivers no old-resource access, Keep read->read delivers valid S share,
and Keep read->write delivers no S share. A source read ExitSlot
delivers a valid borrow and releases slot custody under the existing
allocation-time lazy duty. A same-arm read replacement follows N48.
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
