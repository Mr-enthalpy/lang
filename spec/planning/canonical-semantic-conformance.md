# Canonical semantic conformance matrix

Status: canonical acceptance scenarios; evaluator/source consumers pending.
These are semantic counterexamples and equalities, not claims that the current
parser supports every displayed spelling or that executable tests already
cover them. The topic owners define meaning; this matrix indexes their checks.
The scenarios derive from the PR105 revision brief and preserve its case IDs.

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

## Stage、Policy 与 two-round call

| 编号 | 场景 | 必须成立 |
|---|---|---|
| S01 | P2=runtime，P1 stage 省略 | 补 runtime，不形成 runtime\|\|compile |
| S02 | P2=seal，P1 stage 省略 | 补 seal，不形成另一复合 stage |
| S03 | 裸 let 位于 compile binding context | 阶段由上下文补全，不留下 `_` 供未来 runtime consumer 反推 |
| S04 | runtime callable 的某 Pin 显式 compile | 可按合法输入规则接受，不要求整调用全纯 |
| S05 | formal stage 为显式 hole | 由 actual/位置规则萃取；与 omission 不等价 |
| S06 | runtime-policy 内容的底层值恰好已知 | E 不自行提升为 compile |
| S07 | compile callable 接受 seal actual | 调用可 defer，无 seal→compile migration |
| S08 | 某 static 类型无 runtime realization | 目标 stage demand 不制造 runtime slice |
| S09 | 第一轮留下两个不同 stage derivations | 不先执行两个候选的可观察 body；第二轮照常选择/报歧义 |
| S10 | 已选 compile realization 对应 runtime residue | 同一 selected origin；runtime 不重选 |
| S11 | 迁移已选后 Pre 或 coherence 失败 | terminal failure，无 runner-up 或链式补救 |
| S12 | Pout stage 有外部 demand | 消费者不能覆写 P1/Pout 的已定生产事实 |

## main、Seal 与静态支配

| 编号 | 场景 | 必须成立 |
|---|---|---|
| E01 | 主程序 main 入口 | P2 runtime；不因根历史由 meta 形成就永久 MetaDom |
| E02 | meta -> compile helper -> seal candidate | seal 不可见 |
| E03 | seal -> compile helper -> meta invocation | meta 不可见 |
| E04 | meta invocation 退出后回到 main | 不把 MetaDom 留在不相干后续调用上 |
| E05 | seal 读取已完成 meta payload | 按普通访问规则；不等于新 meta call |
| E06 | SealStatic 首次需要尚不存在的 meta default instance | 不以 compiler trait 查询为由开后门 |
| E07 | seal-dependent action 与某写存在顺序关系 | defer 保留依赖；不能任意越过写 |
| E08 | seal:seal 要提供 runtime:seal | 只有实际合法 migration 才成功 |
| E09 | 编译 cache 命中 | 不重开已 Close 的 subject，不缓存永久 authority |

## 实例生命周期与 with

| 编号 | 场景 | 必须成立 |
|---|---|---|
| L01 | meta-local type instance 被 killing move | 源 generation 在该 cut 结束 |
| L02 | 全局 type resident 与相等局部 copy | 不因值相等共享 Killable/lifetime |
| L03 | stable meta instance 经合法 preserving move | 不杀死元根；不等于所有全局资源可复制 |
| L04 | 同一 move 前沿有活跃冲突 borrow | Movable/Pre 可失败；MoveEffect 不在此时换一种意义 |
| L05 | copy 的 clone 实现有可观察 post | 不把 preserving move 强行替换成该 clone |
| W01 | `x with {a}`，x 后面仍有 use | a 的析构插入考虑这些点 |
| W02 | `x with {a}`，a 后面仍有 use | 不反向延长 x |
| W03 | `y with {x}` 且 `x with {a}` | 相应析构义务形成 y≺x≺a |
| W04 | x 在 k 处被 killing move | k 是列表项的使用点；不再插入 Destroy(x) |
| W05 | x 的 move preserve 源 | 继续考虑源后续实际使用 |
| W06 | `with {}` | 词法清理边界，不是无依赖断言 |
| W07 | 明确消费后离开 lexical scope | 不重复 drop |
| W08 | with 约束形成不可满足严格循环 | 报约束错误，不任意选择顺序 |
| L06 | @ 观察 compile value 或无 Place temporary | 同一 LifeName/K 关系，不另建 compile lifetime |
| L07 | @ 不出现 | lifetime pre/post 仍发生 |
| L08 | lifecycle Pre 失败 | 不修改事实，不反向移动 cleanup |

## Pattern、完成与 meta 查询

| 编号 | 场景 | 必须成立 |
|---|---|---|
| P01 | if arm 后剩余 else | chain 内合法，可由后续 else arm 接管 |
| P02 | else 试图逃离 forbidden boundary | 固定 consumer 根据普通 meta 事实拒绝 |
| P03 | 普通 Pattern 的 residual 被允许外传 | 保留该输入材料，不隐式丢弃 |
| P04 | 一个 arm 已完成 | 后继同级 arm 不重新匹配其结果 |
| P05 | 用户声明名叫 Done 的类型 | 纯普通类型；不接触 internal Done |
| P06 | 嵌套 chain 完成 | 消去各自标记，不产生可观察 Done nesting |
| P07 | selected extractor body 失败 | 不当成 sum miss 或改走另一个 arm |
| P08 | expected result Pattern 存在 | 普通 RΓ 交付，不另造私有 ControlResult |
| P09 | meta trait 查询默认形成后在 OpenHere 内修改 | 修改实际实例值；以后查询读当前 committed 状态 |
| P10 | 只修改该 meta result 的普通外侧 snapshot copy | 不自动修改 retained meta instance |
| P11 | trait 状态后来改变 | 不倒流重写已形成 Pattern 或已选 invocation |
| P12 | Pattern 要求无序 | E 从开始解释时即采用该关系；O 不能晚些决定 |

## Operator 与绑定/贡献

| 编号 | 场景 | 必须成立 |
|---|---|---|
| O01 | 裸 `a+b` | 经当前 operator 环境，不硬连绕过替换 |
| O02 | `operator[+]` 中的 `+` | 普通 operator-name 值读取，不再次降低为 operator dispatch |
| O03 | `g:OG_"+"` 被绑定成任意变量名 a | 参数 `a:b OperatorOverloadGroup` 仍可萃取 "+" |
| O04 | a 来自另一个库的 OG_"+" | selector 读取当前 operator 的 "+" 槽位，不被迫采用 a 的候选内容 |
| O05 | OG_s 转为普通 OG | 普通转换；无 subtype 与隐式反向恢复 |
| O06 | 非法新 token string 构造实际 operator family | 不反向扩充 lexer/parser |
| O07 | `.op` 或显式 path | 保持其各自入口，不触发不属于它的 forwarding |
| N01 | 实现层级单个 closure expression | 本身产生 type；let 普通绑定 |
| N02 | 之后增加合法同名 closure contribution | 不追溯改变第一次 RHS 的求值类别 |
| N03 | 只有 `let a=uint8` | 精确绑定 τ_uint8，不包装、不合并 |
| N04 | 普通合法 inner shadow | 不变成对 outer 同名对象的贡献 |
| N05 | named contribution 的 OpenHere/Pre 失败 | 不借“人体工程学修复”绕过 |
| N06 | ordinary binding 与 contribution effects 撞同坐标 | 依原有角色/写冲突检查，不凭同名自动混合 |
| N07 | 合法独立 closure contributions 分散在 sibling files | 同一 NameCoord 的普通 unordered contribution join；无 first-file winner |
| N08 | 一个 sibling 读取另一个 sibling 新写值 | 文件排序不能制造本无的依赖可见性 |
