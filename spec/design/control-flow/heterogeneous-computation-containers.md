# Heterogeneous computation containers

Status: canonical ordinary interface/effect-tree transformation; source,
F10 proof and execution consumers remain pending. Equations are metanotation;
all source sketches denote target syntax, not accepted executable programs.

### N07 — 容器是普通接口与 effect 树的 compile 变换 [定义]

采用简写函数箭头作为元记号（不是新增源码函数模式语法）：

\[
F=I\to E\to O,\qquad
Thread_W(F)=I\to(E\ |> bind((()\to W)^\sim))\to O.
\]

变换遍及 E 的表达式树：在相应节点引入与 `(()→W)~` 的分支组合，保留原树的选择、重复和后续关系。它不是只在函数前后加一个等待，也不预先枚举或执行所有未来路径。

实际值、type、函数模式及 compile 变换仍使用已有 lang 机制；容器的运行设施必须兑现它的实际执行/交接契约。不得把这项元变换当成一个已经完成的线程实现或强制对象布局。

### N08 — 发射、捕获、逃逸分别判断 [不变量]

异构容器入口的全部实际调用材料为已有 `SelfProjection(Param)` 与显式入参的组合；捕获不再作为另一个需重复消费的 self 字段树。

其向外部名字形成访问的输入，在实际传入的 Pre 上证明 Region 偏序和 Color 准入。缺少该能力则拒绝发射；允许并行不意味着自动允许逃逸。

返回 Thread/异构任务材料仍是普通返回。未完成任务的外部依赖必须压缩到合法入参/捕获来源的外部 unknown 并满足实际存活约束；否则先在内部推进到 Done 或合法解除依赖。没有“默认任意逃逸，随后隐式 join 补救”的规则。

已经独立执行的任务与一个唯一被持有、尚未恢复的续体不同。丢弃句柄是否结束执行责任由真实协议决定，不能凭普通句柄 Drop 杀死仍有执行权的主体。

Input is a formed ordinary callable interface/effect tree and actual Param/args,
selected container operations and common-K handoff/control evidence. Compile
computation preserves the original tree while introducing the W branches. At
actual dispatch Pre checks dependency lifetime, access and independent-access
Color; common commit transfers the actual execution/material custody once.
Post records only the dispatched branch and actual outstanding responsibility.
False Pre rejects dispatch; missing proof/consumer is Unavailable. Ordinary
failure/effects remain declared branches, not a fabricated successful Thread.

The [lifecycle owner](../lifetime/lifetime-policy-and-overload-boundary.md)
defines Region/Color and repair, the [effect owner](effect-steps-and-continuation-handoff.md)
defines Wait/Drop, and the [call owner](../symbol-world/function-object-call-model.md)
defines Param/capture correspondence. A unique unresumed continuation is not
an independently executing task. See [F10 and HET evidence](../../planning/lifetime-concurrency-consolidation.md).
