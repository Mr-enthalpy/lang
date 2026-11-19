# Static, Name, and Structural Semantics

**Status: canonical ontology owner.**

This document defines the canonical relations used by static evaluation, name
production, structural interpretation, type formation, and dot navigation.
Topic owners specialize these relations while preserving their identities and
composition laws.

## 1. One evaluator and three orthogonal coordinates

All source computation uses the same evaluator `E`. A static call is identified
by its evaluation horizon, instance identity, and independent completion policy:

```text
Stage = {compile, runtime}
OpenPolicy = {open, close}
PolicyMode = {const, mut}
```

`P2` owns evaluation horizon and the callable reify qualification of section 20. `OpenPolicy` is a P1 completion coordinate.
`PolicyMode` is the value-mode coordinate. None implies either of the others.

For an omitted mode:

```text
DefaultMode_K(x) =
    mut    if OpenPolicy_K(x) = open and OpenHere_K(x)
    const  otherwise
```

If an `open` completion requires an `OpenHere` fact that is not yet available,
completion is unavailable; unknown is not a proof of the `const` branch.

`open != mut` and `close != const`.

## 2. Compile partners and compile instances

Every compile call uses one `CompilePartner` and forms one stable
`CompileInstance` before evaluating its selected body:

```text
CompileInstanceKey
  = ParentSemanticOwner
  x SelectedCompilePartner
  x CanonicalizeInvocationInputs(In)

I = CompileInstance(CompileInstanceKey)
n_I = InvokeName(I)
```

[Compile invocation](static-evaluation/compile-instance-invocation-and-result-delivery.md#runtime-inputs-to-compile-producers)
fixes key admission for mixed-stage inputs. It uses canonical compile-side
observations and admitted runtime source descriptions. The realized runtime
input supplies neither an extra key coordinate nor an internal Pattern choice.

The instance identity is independent of its result value. A compile call may
return an integer, Product, closure, existing type, fresh self-rooted type, or
another ordinary Object without changing the instance-formation rule.

The instance self-name is an ordinary legal NameValue. Observing it is not
invocation reentry:

```text
Read_name(n_I) != EnterBody(I)
Read_resident(n_I) != EnterBody(I)
```

Actual reentry exists only when the next evaluation edge enters the selected
body again.

In-place closures retain their ordinary lexical, Self, navigation, dependency,
and lifetime boundaries. No closure becomes transparent to instance ownership.
Code that needs an enclosing compile instance uses the instance's ordinary
self-name and admitted dependencies.

## 3. Compile result structure determines expression kind

At the formal normal-return result boundary, whether a completed expression is
a NameExpr is determined by its result structure, not merely by compile horizon.

Let:

```text
Reach(v) = ordinary accessible closure of v
SelfRootSet_I(v)
  = { tau in Reach(v)
      | Type(tau) = type
        and Root(tau) = I
        and OpeningSource(tau) = I }
```

A result that exposes a type still governed by the current instance must expose
that root directly as one complete type:

```text
SelfRootSet_I(v) != {}
=>
Type(v) = type
and v = tau_I
and Root(v) = I
and SelfRootSet_I(v) = {v}
```

Such a result is a computed NameExpr:

```text
Read_name(result) = n_I
Read_resident(n_I) = tau_I
```

A compile expression returning an existing type whose root is not `I` is not
made into a new NameExpr merely because its result has type `type`.

This single-self-root law prevents an instance-open type from escaping hidden
inside a Product, closure capture, reference wrapper, or other accessible
result shape.

Wait's yield_val is not return_val. It never invokes the return-specific
instance NameExpr classification. An instance-open root cannot escape via yield,
even directly, or hidden in an effect sum/Product/capture/reference. Such an
exposure fails the root-escape obligation; it cannot acquire n_I. Already legal
ordinary type/name values retain their established identity when yielded. Vec
and Option-like compile names are still classified at their formal return.

## 4. Interpretation polarity and arbitrary `$` nesting

There are two interpretation polarities:

```text
V = ordinary RHS/value interpretation
S = structural interpretation
```

Structural consumers include Path/navigation completion, extraction structure,
and structural type formation. Define:

```text
Flip(V) = S
Flip(S) = V
Flip(Flip(C)) = C
```

The only general law of postfix `$` is:

```text
Interpret(e$, C) = Interpret(e, Flip(C))
```

Therefore `$` may nest to any source depth:

```text
Interpret(e $$...$, C) = Interpret(e, Flip^n(C))
Flip^(2k)(C)   = C
Flip^(2k+1)(C) = Flip(C)
```

`$` does not read a resident, change Stage, grant OpenHere, change PolicyMode,
create a borrow, or create a lifetime fact.

Resident reading remains independent:

```text
ReadVal(n) = Read_resident(Read_name(n))
ReadVal(e) != e$
```

Bare-name structural completion is likewise independent:

```text
BareName(a; p) = a::p
```

The same navigation-completion law is consumed by Path and extraction-side
structural interpretation. `a$` instead interprets `a` under the opposite
polarity before the surrounding structural consumer uses its result.

A canonical nested example is:

```text
((bool a)(args Mytypefun)$)$
```

When the whole expression occurs in ordinary RHS context, the outer `$`
enters structural interpretation and the inner `$` returns
`args Mytypefun` to ordinary value interpretation. The enclosing evaluation horizon is preserved.

## 5. Pattern is structural registration over Val2

Structural identity is represented by registrations over actual members in
`Val2`.

```text
StructuralRole_P(x) => x in Val2
x in Val2 !=> StructuralRole_P(x)
```

The minimum registered structure includes the members needed to witness the
type's construction/extraction identity and the applicable role relations, such
as:

```text
DirectPatternChild
ConstructEdge
ExtractEdge
FieldView
navigation/incidence roles
LeafSource where applicable
```

Ordinary helper members may exist in `Val2` without participating in those
roles.

Construction and consuming extraction remain atomic complete-object actions
after Pre.

## 6. Structural interpretation forms complete types directly

A structural expression observed through `$` in ordinary RHS context yields a
complete type value directly:

```text
((bool inner)a)$ => tau_a
```

The formation transaction establishes the minimum Val2 witnesses and their
structural registrations together, producing the complete type as its result.

`struct` is an ordinary type-to-type function:

```text
struct : type -> type
T struct = T |> struct
```

It adds the standard ergonomic/helper interface that is not required to define
the input type's structural identity. For `T' = struct(T)`:

```text
StructuralRegistration(T') = StructuralRegistration(T)

Val2(T') may contain additional ordinary helpers
V_(T') may contain additional associated helpers
```

Typical additions include ref/share observation families, their ordinary policy
cells, field-write/setter families, and other mechanically derived associated
helpers.

Type composition and place update are ordinary operators:

```text
*  : type x type -> type
*= : type ref x type -> unit

T * D
r *= D
```

For `r *= D`:

```text
T  = Read(r)
T' = T * D
Write(r, T')
```

The selected action still performs the ordinary Pre checks for OpenHere,
Writable, capability, access, lifecycle, and continuation legality.

## 7. File hierarchy lowers to name formation

A source hierarchy changes the inherited navigation context. It does not imply
structural type composition.

```text
a/
    P let inner = rhs;
```

lowers at the semantic handoff to:

```text
P let inner::a = rhs;
```

For example:

```text
a/
    let inner = bool::;
```

is:

```text
let inner::a = bool::;
```

`inner::a` and `bool::` are NameExpr structures at the same semantic level.
The hierarchy therefore reuses ordinary navigation completion and name
realization rather than type formation.

## 8. Requested-name producers are general compile computations

A generative declaration receives a requested NameValue and evaluates an
ordinary compile body:

```text
q = RequestedName
I = CompileInstance(producer, q, CanonicalInputs)
E_I(rhs) => v
Realize(q, v)
```

Neither of these is required:

```text
Type(v) = type
v is a NameExpr
```

Name production and NameExpr-valued results are orthogonal:

```text
ProducesName(q, v) !=> NameExpr(v)
NameExpr(v) !=> ProducesName(q, v)
```

This is the common basis for generated values, callables, members, types, and
other ordinary residents.

## 9. Dot is ordinary ADL over a generative overload family

Dot syntax enters one ordinary `adl` name family. Its canonical source
definition is:

```text
adl/
    let <field> (self, object:t, ...args) field
        => ((object, args) |> ((field#)[0])$::t);

    let <field> (self, t:type) field
        => ((field#)[0])$::t;
```

The first candidate implements ordinary receiver calls. The second implements
type-path progression. Existing overload applicability and specificity choose
between them.

Thus:

```text
object.field(args)
=> object |> .field(args)
=> (object, args) |> field::adl
```

and for a type-valued receiver:

```text
T.field => field::T
```

Repeated type-path progression provides left-to-right surface navigation while
the canonical Path representation remains `name::path`:

```text
foo.bar.baz =_Path baz::bar::foo
```

Dot syntax supplies the ordinary `field::adl` requested name and receiver
material; subsequent authority and behavior come from the selected candidate.

## 10. Double-dot is only pipeline-dot contraction

Double-dot has one surface law:

```text
express..field(args)
=
express |> .field(args)
```

After this rewrite, the ordinary dot/ADL rules apply.

## 11. Static self-modification and generated design patterns

A compile expression may retain its instance opening with:

```text
OpenPolicy(I) = open
OpenHere_K(I)
```

For an omitted mode this yields the ordinary `mut` observation described in
section 1. The expression can then use the normal type/reference/operator
algebra to update its current value. No extraction of a hidden auxiliary
payload is required.

Trait-like and automatically generated design patterns are ordinary compile
programs assembled from requested-name production, overload applicability,
self-name observation, `open/close`, OpenHere, and ordinary type operations.

## 12. Canonical consistency

The active tree uses one canonical name and relation for each semantic concept.
Source, normalized interfaces, semantic carriers, diagnostics, tests, fixtures,
contracts, planning documents, public guides, glossary, and agent instructions
use the vocabulary and equations defined by their topic owners. Source
conveniences lower to the lowest existing relation that explains them.

## 13. Adjacent-universe joint extraction

For every adjacent universe rank, a simple binding consumes one relational
extraction judgment. Let `Gamma;K |- E(expr) => v` and `Type(v)=T`:

```text
R_Gamma(a:b, v, rho)
  => rho(a)=v and rho(b)=T
     Rank(rho(b))=Rank(rho(a))+1
```

Bare `b` at this extraction site introduces the classifier-side binder. An
existing external Pattern enters explicitly through a full Path, such as
`uint8::`, or opposite-polarity `R$`. Deduction, an external Pattern constraint,
and a classifier Hole all use `R_Gamma`; Hole presence does not change the
binding judgment into a result-transforming call. An already selected forced
extraction/delivery may consume the result owner's [proved-root-mismatch conversion rule](patterns-overload/return-value-extraction-and-implicit-decomposition.md#forced-extraction-and-one-root-conversion).
It is separate from ordinary applicability and preserves selected no-reopen.
Structural qualified
`P let name::path:t` is the name-formation judgment of the names owner.
Initializer-free `let a:uint8::` can form a typed name and uninitialized Place
when that judgment's external-type premises hold. Likewise `let a:R$` requires
the external Pattern to determine a complete classifier. Bare `b` in `a:b`
remains classifier-side extraction material without an initializer; without a
delivered subject it supplies no classifier or typed Place. Successful typed-name
formation creates no resident. The [names owner](symbol-world/names-and-overload-groups.md#5-typed-name-declarations-and-complete-let-bindings)
defines ExternalClassifier and the separate lexical/qualified formation premises.

```text
Interpret(_, S)=AnonymousPattern
Interpret(_, V)=universerank
```

`universerank` is ordinary universe-rank material. Anonymous Pattern, explicit
binder absence `<>`, and root-qualified Deduce Holes have distinct identities.
Polarity changes neither the universe rank nor resident availability by itself.

In S, `_$` and `(_)$` supply the external universerank Pattern, rather than
an anonymous matching hole. The [Product owner](patterns-overload/product-repetition-and-unpacking.md)
defines the singleton contextual form `_$ =_context (_ * 1)$` and the
standard wrapper `product = (_ * ω)$`, with classifier Pattern `_ * ω`.
Bare Product normalization exposes and merges available standard repetition
material at the required context, including call input. Syntax nesting is
preserved separately from that observation; expression barriers, non-standard
structural layers and unavailable runtime shape retain their own premises. Repetition
relocates element material into Val1; compile material still participates in
complete identity, and legal value replacement need not mutate its Pattern.
`pack?` exposes an admitted repetition layer. Compile `t * ω` can expose an
available finite payload; runtime-only `t * ω` needs an ordinary iteration-like
consumer and grants no static variable-arity expansion.

## 14. Hole-local Policy

```text
P let <P_h let h> LHS = RHS
Policy(h)=HoleOverride(P,Delta_h)
HoleOverride(P,Delta)[d] = Delta[d] if d in WrittenDims(Delta)
                         P[d]     otherwise
```

An omitted `P_h` inherits all outer dimensions. An explicit dimension replaces
that dimension without an outer-strength restriction. Pattern constraints and
Policy constraints of each Hole are solved by the ordinary relation under its
`PatternRoot/HoleBinderId`; the local Policy operation is independent of
`ElabIn` and `ElabOut`. In `<p> let <p let h> ...`, the inner Policy occurrence
references the preceding exact Hole identity of `p`.

## 15. Bare branches and ordinary bool values

```text
Bare(s)=<nu_s,P(s),Val2(s)>
UniqueValueMember(Bare(s),nu_s)
```

The member exists. Its machine representation is unobservable; `nu_s` fixes
branch identity through ordinary Pattern relations, without a public integer
encoding or synthesized extraction field. A compile Pattern with a runtime
value observation is a legal view of this Object.

```lang
let bool = ((true | false)bool)$;
let true = true::bool;
let false = false::bool;
let if = true::bool;
let else = false::bool;

|> name { B }
// syntax-directed expansion:
|> (_, let _: (name)$) { B }
```

The two branch-head `_` positions are structural anonymous Patterns. The
annotation's complete `(name)$` operand enters V interpretation and supplies
the existing value Pattern to extraction. `if` and `else` are ordinary value
bindings of the true/false branches.

## 16. Fully specified Path and structural navigation

The [Path owner](symbol-world/structured-path-algebra-and-interpretation-polarity.md)
defines full navigation and structural suffix overlap over actual Path values.
`Relative::Relative` and `Relative::Absolute` retain their composition;
`Absolute::Relative` is undefined; `Absolute::Absolute` requires a matching
complete rooted suffix. Pattern navigation consumes the same completion law,
adds extraction and explicitly full-navigation module resets:

```text
Path algebra strictly-contained-in Pattern algebra
let a p1 p2 p3
  => let a (p1::p2::p3) (p2::p3) (p3::)
```

Normalized navigation retains actual members, full navigation and extraction
relations. Syntactic inner/outer provenance supplies no additional identity.
`foo.bar.baz =_Path baz::bar::foo` requires a type-Path candidate selected at
each step. An object method can return an unrelated ordinary Object.

## 17. General open computation and struct helpers

For a legal compile open call under ordinary by-value passing:

```text
OpenHere_K(type_val) iff OpenHere_K(type_val compile_open_call)
```

This is subject-preserving observation of the same concrete type value;
Writable, access, dependency and lifetime premises remain independent. Open
computation supports name/member generation and ordinary Policy migration.
`(T ref) |> (type ref)` uses ordinary type/value invocation and reference Pre.

`struct` receives a complete structural type and returns a complete type with
ordinary additional helpers. Its legal ordinary open computation can use
`OpenHere`, `|[[attribution]]>`, `type ref`, and `*`/`*=` or member registration
to form forwarding members on derived reference types. Reference/borrow and
lifecycle primitives belong to ref/share; field forwarding requires these
explicit helper registrations. StructuralRegistration is preserved.

```text
DerivedState(T0) depends on the concrete T0 snapshot
Write(place,T1) does not retarget DerivedState(T0)
```

Observing final members of `T1` requires a new ordinary formation/call on `T1`.

## 18. Dependencies and opaque self

Compile callables may have ordinary implicit dependencies. Formation records
automatic/initialized realizations and required capture specifications. New
invocation fulfills required sources in its admitted call context, and includes
observed material in canonical inputs; sealed binding never re-resolves.
Readiness separately asks whether the
current action needs a concrete value that is available. Runtime materialization
can supply such a value to a delayed continuation. Dependency validity and
instance reuse retain all result-affecting identities and observations.

```text
Norm_Name(n) is independent of structural unfolding of self
```

`self` is observable through normalized Pattern algebra without a public capture
layout. Its compile-flow observation uses the current CompileInstance, and its
runtime-flow observation uses the current callable object. The same call,
Policy, dependency and lifecycle relations govern both observations.

## 19. Callable tag and activation

```text
ReturnTarget = current callable object tag
CallType(tau): CallTarget(I)=tau, ActualSelf(I)=c* in V_tau
Tag(closure expression)=tau_C
Type(tau_C)=type; Type(type)=(1 type)
```

Ordinary value invocation retains its normal implicit self. Return routing
observes the callable tag and the actual current control-flow activation;
recursive activations of the same tag have different frame identities. Tag
observation supplies no candidate preference and unfolds no private self layout.
ReturnEvent, ReturnPattern/Pout and internal completion retain their existing
delivery and no-reopen boundaries.

## 20. Reify callable P2 and scoped reverse migration

```text
Stage = {compile,runtime}
P2 = compile | runtime | reify
EvaluationStage(reify) = runtime
OutwardHorizonPolicy(reify) = OutwardHorizonPolicy(runtime)
```

Reify is a callable-head P2 annotation. Its domain is exactly the policy
position between a callable's parameter head and return clause:

```lang
let compare = (self, object1, object2):reify -> _:bool:: => {
    let <runtime let o1, compile let t1> o1:t1 = object1;
    let <runtime let o2, compile let t2> o2:t2 = object2;
    o1 < o2;
};
```

Ordinary closure formation still returns tau_C. The annotation supplies no
additional Stage, Object or callable-construction operation. P1, Pin, Pout,
Hole Policy and expression PolicyLet do not admit reify as a Policy atom.

For a callable boundary B, let RevDeps(B) be its actual selected reverse
migration dependencies. ClosedReturn(B) holds when ordinary input/contract
elaboration determines a unique complete result classifier T_out and:

```text
Deps(TypeContract_B(inputs)) intersect RevDeps(B) = empty
every normal return is delivered under ReturnPattern_B / Pout_B
Type(delivered result) = T_out(inputs)
```

The classifier can be explicit, such as R$ or bool::, or deduced from input
facts independently of reverse migration. A contract requiring a reverse
migration to discover its own classifier fails before that migration. This
closes the boundary rather than propagating delayed type demands to callers.
ClosedReturn admission checks the classifier's uniqueness and dependency
independence. The delivery clauses are enforced on each reached normal return;
they require no advance enumeration or static proof of future branch paths.
Reached normal-return delivery uses the result owner's forced-root extraction
rule. Failure of that completion, including its selected conversion or final
root relation, is terminal failure of the selected call.

```text
ClosedInterface_K(B) iff
  compile elaboration determines outward input/output, regular effect shape,
  necessary capture and lifecycle requirements independently of RevDeps(B)

ReverseAllowed_K(m) iff
  ActiveBoundary_K(m)=B and P2(B)=reify and ClosedReturn_K(B)
  and ClosedInterface_K(B)
  and m is the selected ordinary same-Type migration candidate
```

ActiveBoundary_K(m)=B is established at migration formation by the actual
owning callable activation: B is active on K, its declaration has P2=reify,
and this request belongs to B's body/input-consumer continuation. A nested
ordinary callable retains its own P2; an enclosing reify activation does not
authorize that nested callable to form a new reverse request. Already admitted
dependent work can include ordinary helper calls, retaining the request's B
identity and validity. Leaving B or changing to a foreign continuation supplies
no reusable boundary authority.

Runtime may choose any actual branch inside a fixed regular shape such as
A || B. If transparent propagation would make the public interface shape depend
on runtime information, an ordinary handler/wrapper must close it to a
compile-determined Function Pattern before exposure. Reify allows effects under
this closed interface; it cannot infer an unsettled compile interface from a
runtime value. ClosedReturn and all ordinary dependency/lifetime premises remain.

Compile-to-runtime and scoped runtime-to-compile migration share existing-first,
direct candidate selection, endpoints, capability, DynamicLegality and no-reopen.
Machine types have generated bidirectional candidate declarations. A deleted
candidate, missing host realization or library restriction can remove either
direction. Equal Type and Policy endpoints alone establish no capability.

Reverse migration carries the affected computation to the actual runtime
frontier; no runtime information is written into a past compile world. Let
D_type be the dependency closure of actual runtime-value-to-compile-type edges
within B. Only D_type requires runtime materialization of the corresponding
restricted E work. Independent compile type work remains executable, and
runtime-value work keeps its ordinary continuation. O may remove a value-only
compile detour after E proves that no type dependency is involved and revalidates
the affected projections. E needs no future-path or dispatch-table enumeration.

Runtime-policy type values and runtime:runtime views are ordinary observations.
Their type algebra is the same compile algebra; its use on runtime-only material
requires a selected reverse migration under the reify boundary. In particular:

```text
runtime:runtime -> runtime:compile
result o has Policy(o)=runtime and Policy(Type(o))=compile
```

The endpoint observations belong to adjacent ranks of the same Object's
classification chain. When its actual default-world recovery condition succeeds, reverse migration
recovers the classifier of that runtime Object, not an unrelated compile type.
An instance hiding non-default dependencies follows the declared ordinary throw
branch; missing consumers remain Unavailable, not a catchable recovery failure. Its same-Type and actual
source/classifier evidence survive in the resumed continuation.

```text
Legal(Pi_e) iff Pi_e in (absent)*(runtime)*(compile)*
PolicyPair = first two ranks of the observed chain/window
FirstPresent(Pi_e)=min { i | Pi_e(i) != absent }
Target_runtime(Pi_e)[i] = runtime if i=FirstPresent(Pi_e)
                         Pi_e(i) otherwise
Target_compile(Pi_e)[i] = absent if Pi_e(i)=absent
                         compile otherwise
```

Ordinary `runtime let` requests this first-present-rank change only, preserving
rank+1 and the rest of the chain. It does not request runtime:runtime.
Unrefined `compile let` requests all non-absent ranks compile, preserving the
absent prefix. Pair is only the commonly used two-rank projection; additional
ranks retain actual classifier links. An observed compile rank and the chain
law establish its entire compile tail; an all-absent prefix does not establish
the unseen chain. Target formation supplies no migration witness. Explicit
Hole-local Policies can target different extraction ranks under the same
coherence relation; compile:runtime is not a valid view.

P2=runtime with P1=compile and mixed-Policy output positions are legal. P1/P2
spelling does not by itself require reify; actual reverse-dependent continuation
work does. First-round Policy overload jointly consumes input/output positions
on equal terms without executing second-round realizations to establish its
own applicability facts. Source joint extraction with explicit Hole Policies
expresses the rank observations without a public pair literal.
Selected calls retain their candidate/projection/frame; a call newly formed by
runtime E performs its first ordinary selection at that frontier. AOT, JIT,
caching and tables are equivalent O representations of the same E.

## 21. Continuation-local Region

```text
Region_K(n)=R(NamePosition_K(n))
```

The current semantic name position determines the fixed-K discrete event
interpretation. Actual reachable-continuation restriction may remove events;
unknown future values and physical execution time confer no pruning authority. E uses current continuation Pre/commit/Post
and fixed cleanup observations. It does not require future-path enumeration.
Inside a closed reify P2 boundary, `compile let val = runtime_express` may
remain an actual delayed continuation when its
required value is not yet available. A selected reverse migration inside a closed reify P2 boundary resumes
its actual dependent E work with inputs at args-in; ordinary origin relations stay within semantic name-position
algebra. Runtime strings supply no authority to redefine fixed structured names.
Future-path proof, specialization and layout remain O work with E revalidation.

## Partial recovery and outward closure

### N25 — reify 的接口闭合不因恢复异常而放松 [不变量]

保留 ClosedReturn、ClosedInterface 和 active reify boundary。恢复失败的异常/effect 必须落入 compile 可确定的对外接口，或由内部普通 handler/wrapper 收束。

runtime 可以选择固定和模式中的不同结果，不能倒灌出新的 compile 接口形状、输入要求或生命周期契约。两级端点只是常用观察窗口，不能把所有实例、函数模式或生命周期材料固定在 rank 0。

The lifecycle owner's N20–N24 define legal forgetting, retained executable
Move/Drop material and the actual default-instance recovery condition. A
same-Type migration candidate and active boundary are necessary, not evidence
that every instance is recoverable. Failure keeps actual ownership/effects and
never reopens selection.

## Cross-owner correspondence

U01–U12 below compose the named topic owners; Region/origin/Color/repair remain
owned by lifetime, standard routing by heap, and atomic memory by its own topic.
Mathematical notation and source sketches are target material, with consumers
and proof gates recorded separately in planning.

## U01 — 同一动作、多种伴随观察

对同一个已选动作 `a*`，普通求值、结构/Pattern 变化、生命周期伙伴、原子内存事件必须使用同一实际来源和交接对应：

```text
selected ordinary action a*
  -> actual value/result delivery
  -> corresponding Pattern transition
  -> lifecycle companion over ordinary rank-0 values
  -> memory event when this action really performs an admitted atomic access
```

这些不是另加四个执行器。结构形成时同步产生需要的生命周期伴随材料；`(x.field*)@ = x@.field_lifetime*` 的已有要求继续有效。`$` 本身仍是解释极性机制，不因每个 `$` 额外执行生命周期动作。

atomic 注册的底层事实是共享内存域真实提交能力；对原始 CAS，其同一个提交点同时使 A→B、I_a→I_b、S_a→调用者及 S_b→B 生效，并发布成功版本与相应生命周期 Post。返回的 Committed/Rejected 是一般和 Pattern，其分支续体的内存因果观察属于同一个 K（N39、N41）。不把所有一般构造/提取的完整性提交都改成 CAS。

## U02 — Pre/动作/Post 与正常失败

静态不可用、Pre 不成立、原子竞争拒绝、Option None、reify 实例恢复拒绝是不同情况：

| 结果 | 普通语义处理 |
|---|---|
| 语法/必要消费者缺失或必需证据不可得 | 正常诊断/Unavailable；不造成功或可捕获业务异常 |
| 已选动作 Pre 不成立 | 动作提交前拒绝；不回滚已提交历史、不重选 |
| CAS 实际竞争未提交 | 声明的 Rejected 和模式；不改变本次原子状态、不取出 S_a、返还/保留 S_b；仅在实际观察有证据时发布对应因果事实 |
| 验证产生 None | 普通 None；被该 Option `?` 使用时由其实现 throw |
| 合法已选 reify 恢复遇非默认实例 | 声明的普通异常/effect；无成功恢复 Post |

调用方 Pre.check 与实现体 Pre.admit 不得互相循环自证；Post 的普通 check/admit 仍只传递实际授权事实。Origin 合流、typed helper、bool 与 OpenHere 均不创造权限。

## U03 — 固定事件、repair 与重新验证

Region 从固定 K 的已形成事件描述取得；新 repair 是依保留生产对应生成的普通续体片段，不是生命周期检查器把原有 drop 前后挪动来救程序。

生成片段必须有普通名字形成、Move、cleanup 与交付关系，且在真实交接发布前全部验证。改变 K 的合法生成/优化必须重新验证受影响投影。不得把这个过程实现为“调整 cleanup → 生命周期失败 → 再移动 cleanup”的求解循环。

生成算法如何先形成足够的修复描述、固定新增片段并避免无根据循环，是 F02/F03 的明确证明义务。缺少这项消费者不能被一个修复成功标记替代。

## U04 — 空切片连续性保留，Move 不变成恒等

```text
same world, same vocabulary, no action:
    Pre(id) = Post(id)

real Move[repair]:
    source world -> validated successor world

held successor material:
    AutoPre = mapped successful handoff Post
```

换了 self/activation/名字 generation 时使用真实映射，不把旧 self 和新 self 当成同一对象。新 resume 入参仍在真正调用时形成；未绑定 RequiredCapture 仍是声明，不因手握一个后继续体提前获取来源。

## U05 — Pattern 默认关系不替代实际名字判断

Pattern 节点内的 `lifetime default` 是普通表达式材料，按子到父递归合成完整局部关系声明；`lifetime pre` 在默认表达式之后串行规约，只增量修改它明确表达的关系。普通 Pre/check/admit 消费最终形成的关系与真实调用映射，不存在另一个全局约束求解器。ref/share 默认外部来源是当前名字访问树之外的有根 unknown，已知来源代入，仍未知则沿调用边界递归传递带严格 Region 的义务；ref 默认来源互异，share 默认 may-alias。Pattern 默认不证明任何具体实例的 Movable，真实事件仍验证 Region、Access、Color、来源与目的地。

同理，无外部 origin 是 atomic 全部保留状态名字（含保留的 I/Index 与 S）的准入约束，不给外部 Cell 借用永久存活；只有合法普通生产 Post 能建立真正闭合的新名字，状态数角色不清除旧依赖。保存 Move/Drop 操作给遗忘对象可执行性，不给 reify 可逆性。

## U06 — 合流、压缩、SCC 与名字遗忘各有职责

- Pattern 合流决定来源选择是否仍由结果模式区分。
- 边界 origin 压缩移除已经退出的局部中间名，保留实际外部依赖。
- SCC 在既定 closure 边界坍缩到原始正则材料，不凭类型重新重建名字身份。
- 匿名名字遗忘主动降低精度，只在默认可恢复或保守摘要充分时准入。`[]` 路径表示已知 `[i]`、运行期区间 `[0:n]` 和重复 `[0:ω]`；元素外部来源以 `array@.origin ||= out_lines` 的普通关系表达式汇总并保留 Region 义务，取出新名继承、剩余数组不扣减。
- runtime:compile 到 runtime:runtime 保留非默认 origin 与 Move/Drop 可执行材料；闭合 reify 的反向恢复必须让真实实例满足完整 Pattern 默认，偏离进入已声明失败。

四者均不能提高权限或伪造 None/unknown。它们不是同一个“把看不懂的东西变 unknown”的操作。生产式重演也不得穿过已合法遗忘的边界恢复隐藏历史。

同 Pattern 合流后的外露接口不再分辨原来源；内部为 E 等价验证保留的来源记录不授予调用者恢复这些事实的权力。

## U07 — 函数模式、捕获与输入槽不回退

普通可调用值保留结构化来源，捕获不必由调用者写成显式实参；LHS 函数模式萃取完整入参/捕获及契约，来源遗忘后原捕获作为显式参数。RHS 仍是普通调用请求，不有自定义 capture 或新 Pre/Post 位置。

`P let lhs=rhs` 是调用位置材料命名，不是修改函数模式方向性的理由。RequiredCapture 每次新调用在合法上下文绑定；Initialized/Automatic 已形成来源不重新按同名查找。

自动 repair 新执行的生产片段不把一般调用/投影改成“重跑初始化器”。手写函数模式省略 Post 仍是 NoDelta，不因新增自动 repair 就从被遗忘的函数体中重推合同。

## U08 — 物化与生成粒度不获得第二套解释权

原子切割粒度与实际生成续体粒度继续分开；从保留来源的母版可按需直接生成较大片段，不要求先生成对象再融合。

自动 repair、延迟析构和内存观察有真实可见效果时，融合不得删除这些效果。只有有证明的等价变换才可消除动作。默认自动栈语义、由 E 决定的 ABI1/ABI2 与显式 Box 保持原职责；async、返回、匿名化或默认恢复失败都不授权 O 隐式选堆分配补救。

## U09 — Heap 回收不取代内存排序

malloc 登记实际 lazy Drop、各边界验证 Region 及单次责任交接解决存活/回收合法性；它们不自动建立发布/订阅的 rf、对象修改序或 fence 保证。

malloc 的 Pre 只检查当下分配；HeapRelocate 的成功 Post 将已交接支持
证据中的实际遮蔽绑定重解释到已恢复的直接父层，保留历史 origin、
Places 和独立借用。
实际返回独立检查最新 Post 下的名字压缩。分配期建立的 lazy 责任
覆盖 Atomic read 槽位退出及合法旧 share；Forget_read 为运行时 NoOp。
回收寿命、实际读权和 rf/Past 发布事实各有证据。

## U10 — Atomic 的复用边界是输入结构和关联 API

合法的复用路径为：

```text
general $-formed Pattern/type material
    -> admitted atomic registration
    -> actual associated Cell ref/share surfaces under OpenHere
    -> ordinary safe operations / Pattern branches / effects
```

[普通递归命名 Pattern](patterns-overload/pattern-relations-observation-and-extraction.md#92-recursive-named-pattern-material)
按普通中间层规则观察实际内层材料，再递归调用同一个 `R_Gamma`。
共享 HoleBinder 的估值必须相容。构造使用同一已准入结构对应及独立的
Pre/commit。普通限定中序 `source ->::E target` 向显式路径交付双输入
Product，已选构造器决定可调用变换是否成立。
[普通借用和模式变换](patterns-overload/operator-patterns-and-generative-declarations.md#ordinary-borrowed-sum-transforms)
在独占借用下匹配实际源并提交完整目标。take 交付实际旧资源及其责任。
保留资源的成功分支按普通 Region Pre 构造 share，并在 Post 记录真实
origin 依赖。Atomic 消费同一变换描述，提供条件原子提交。Atomic 的
准入另查完整 E、实际外部类型叶子的包装形成及实例对应（N46）与有限表示。
全源匹配约束真实 arm/协议变化；同 arm read 可在同一提交确认旧 Box
后原位替换，同 arm write 不可替换。N39/N47–N50 定义提交、Keep/Exit
权利、失败候选交还及读协议转写协议时读权限的结束。N41 规定版本因果。
[共享视图原子快照](lifetime/atomic-pattern-transitions-and-causality.md#n42--共享视图上的原子快照-定义及不变量)只由 `Atomic` 在实际关联类型 `E share`
上登记的 `?` 提供。选中的成员以一次原子读取交付同一已提交版本的
公开状态，结果不含资源。其他 `?` 调用遵守各自的普通契约。

没有 `atomic enum` 新本体；没有自动 `struct-of-atomic -> atomic-struct` 规则；没有“用户声明安全即可”的任意世界写入口。

更高层算法的可证明行为必须位于既有内部安全不变量的组合范围内。想强制一个更小的协议转换集合，必须让它成为真实的接口/材料/Pre 限制，不能只给一个包装名字。

## U11 — reify 保留阶段代数与失败效果

[Compile producer 的 runtime 入参](static-evaluation/compile-instance-invocation-and-result-delivery.md#runtime-inputs-to-compile-producers)
在实例形成前记录 compile 观察和已准入的 runtime 来源描述。内部 Pattern
由 compile 观察确定；runtime 值只参与具体萃取及名字形成/比较。
`$` 在各层执行普通极性翻转，选定的生产调用仍遵守同一阶段规则。

默认世界恢复只是现有选定迁移中关于具体实例的准入条件。它不另立 Stage，不给 runtime 值写回过去 compile 世界，也不根据值的运行结果扩展 compile 已闭合接口。

恢复需要的 Drop/Move 可执行材料和默认性判断必须真实保留或实现；类型注册表/标签只是一种实现，不能成为生命周期本体。对外异常行为与所有其他 effect 一样由普通接口匹配/handler 处理。

## U12 — 主干规范与 E host 同步

现行 host 契约为 `spec/contracts/e-host-architecture-and-incremental-revalidation.md`。本轮须保持其单 E、合法来源对应、选择域验证、完整 K、缓存状态依赖与提交前联合验证要求。

新增 Region 形态、生产式 recipe、合流摘要、遗忘资格、恢复资格、Move/Drop 实现、heap 父层及 Pending、原子权限与因果前提，都要进入真正依赖它们的缓存/复用检查。签名或机器布局没变，不等于这些判断仍有效。

缓存命中不重放旧 IO/CAS；语义规定的 repair 重演是新动作，不是 host 重用旧提交。不得以并发/原子为由给优化器或实现层私下新增可观察事实。
