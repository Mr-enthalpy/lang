# Lifetime Values, Bootstrap and Library Facilities

Status: canonical semantics for `@`, the lifetime value algebra, and the
lifetime/ordinary-overload boundary, ordinary contract programs, boundary
transforms and library protocol effects. [Walkthroughs](lifetime-algebra-walkthroughs.md)
apply these rules; [consumer contracts](../../contracts/lifetime-program-consumers.md)
record wiring, and [acceptance targets](../../planning/canonical-semantic-conformance.md#lifetime-value-computation-and-library-design-acceptance)
are specifications rather than executed coverage. `LifecycleState` stores LifeName, Region,
Color and lifecycle observations; it checks Pre and consumes the shared
`SemanticContinuation` transaction's committed-action witness for Post.
Linearization and Pre boundaries of already fixed cleanup points are implemented.
The common transaction checks earlier cleanup for every action through a
continuation-relative observation relation. Outstanding or unavailable facts
block commit even when the action has no lifecycle projection.
The substrate fixes continuation-relative cleanup prefixes, leaving future
generations' suffix placements available. Use/Preserve cannot preempt their scheduled cleanup; Drop and boundary Kill
follow fixed order. Regions retain actual discrete events, including ending events.
Roster discovery supplies stable names, not formation/origin facts. Full
generation-continuation reification, source operations, cleanup point derivation,
access-tree construction and the complete E consumer remain pending.

This document is the canonical owner of `@`. The object model, the value/place
split, and the `ref` / `share` / `rebind` operations are owned by
[`../symbol-world/type-values-places-and-borrow-views.md`](../symbol-world/type-values-places-and-borrow-views.md);
construction authority (`OpenHere_Σ(value)`) is owned by
[`../symbol-world/structural-type-formation-and-composition.md`](../symbol-world/structural-type-formation-and-composition.md).

## 1. `@` is continuation-relative name observation

N@ is a name iff N is a name. N@ is rank-0 ordinary lifecycle material with
its own type, fields and associated operations, independently of N's rank.
Origin and region are ordinary fields of lifecycle Objects; Color belongs to
region. They use Object = <Val1?, Pattern, Val2>, not a fourth business-Object
coordinate or a higher-rank constraint layer. Lifecycle computation knows
names and these relations, not special integer/container/business-type meaning.
Type punning's legality remains with the actual conversion's other consumers.

Reification preserves name interpretation; value observations and borrowed
lifecycle fields remain distinct. Safe code may observe origin/region/Color.
Ordinary boundary admit uses the checked boundary's limited world-update
authority (§9); constructing a contract grants no such authority. Raw writeback
of otherwise underivable external knowledge additionally requires
[unsafe admission](unsafe-semantic-admission.md), ordinary borrowing, Writable
and applicable construction premises. Neither route permits arbitrary history
replacement or a missing Pre. This does not make @ a borrow constructor.

`@` is a normal overloaded operation. It is not a syntax-only marker, not a
reserved lexical territory without semantics, and not exempt from overload
resolution:

```text
Apply(@, E):
  n := NameOf(E)
  k := Pos(SemanticContinuation)
  return ReifyLife(n, k)
```

`@` still applies to an expression syntactically, but its builtin action switches
from value interpretation to the semantic name observed at the current
continuation position. It neither exposes a user-callable `PlaceOf(E)` nor reads
hidden residency metadata from the value. Lifetime facts already follow from
the continuation; applying `@` reifies them and does not launch a second
analysis.

`@` is **not** a borrow constructor. It yields a lifetime value, never a borrow
view. Borrow formation belongs to `ref` and `share`, which may be privileged
actual-place builtins (`PrivilegedActualPlace(ref-family)`,
`PrivilegedActualPlace(share-family)`) when the overload needs the actual's
place (§2). A freshly computed temporary may receive a generated semantic
LifeName even when it has no writable or borrowable Place, so
`NoCarrierPlace(actual)` is not a general failure condition for `@`.

`@` is not a Stage atom. `@` is evaluated at a stage; it
does not name one.

## 2. Privileged place acquisition belongs to `ref` and `share`

`ref` and `share` are builtin applications that may, for some overloads, be
granted privileged access to the place of their actual argument:

```text
PrivilegedActualPlace(ref-family)
PrivilegedActualPlace(share-family)
```

This does not make them one callable family. `@` remains a normal overloaded
operator whose selected builtin performs name/continuation reification; `ref`
and `share` each have a type-forming overload (producing `type ref` /
`type share` as type values) and a borrow-forming overload (producing `t ref` /
`t share` as borrow instances). Place permission attaches to each borrow
family's builtin identity, not to ordinary function parameters. An ordinary
user function that spells the same formal head still cannot obtain the
actual's place.

The operators produce different results through different semantic premises:

```text
@      -> ReifyLife(NameOf(actual), Pos(K))
ref    -> borrow instance | TypeValue (type formation)
share  -> borrow instance | TypeValue (type formation)
```

Phase order for a place-sensitive `ref` / `share` member:

```text
ordinary candidate preparation
  (Pattern / type / Policy matching on the actual value)
-> ordinary overload selection
-> unique selected builtin/default
-> if SelectedBuiltinRequiresActualPlace:
       p := PrivilegedActualPlace(actual)
       if p absent:
           InvocationFailure(NoCarrierPlace(actual))
           -- post-selection precondition failure, not candidate
              removal / overload reopening / fallback
-> execute default
```

`ref` and `share` are privileged actual-place builtins
(`PrivilegedActualPlace(ref-family)`, `PrivilegedActualPlace(share-family)`).
There is no global `E ref = Ref(Read(E))` law: the selected overload determines
the result, and the builtin default may acquire `PrivilegedActualPlace(actual)`
(canonical owner [`../symbol-world/type-values-places-and-borrow-views.md`](../symbol-world/type-values-places-and-borrow-views.md)
§5.1). For `let t:type:: = uint8`, `t ref` is the **type-forming** overload: it
yields the TypeValue `uint8 ref` (the borrow type of `t`), not a borrow
instance. The borrow instance over the type-level place is produced by invoking
that borrow type:

```lang
t |> (type ref)    // invocation: borrow instance r : type ref, Target(r) = place(t)
t |> (type share)  // invocation: share instance over the type-level place
```

`type ref` is the ordinary type construction `type |> ref`, and `type share` is
`type |> share`; they are not special tokens and not produced by `@`. A
`type ref` is a type value; an instance `r : type ref` carries the borrow
content (target place, capability, lifetime relation) as a borrow instance, not
as the type value itself.

The builtin `ref` / `share` callables have two overload roles. The
type-forming member makes `type |> ref` / `type |> share` a well-formed
ordinary type construction through the ordinary type-as-callee / overload
machinery; no `RefType` primitive is introduced:

The `ref`/`share` family is not a single compile stage: the **type-forming**
member is a **compile** member (`T : U_n ⊢ T |> ref = RefTy(T)`, producing the
borrow TypeValue), while the **borrow-forming** member inside the formed
borrow type's callspace has ordinary concrete-stage builtin/default realizations and
is the only family member that may obtain `PrivilegedActualPlace`. The
declarations below are the type-forming members; the borrow-forming members
live inside each formed borrow type's callspace (canonical owner
`../symbol-world/type-values-places-and-borrow-views.md` §5.1).

```lang
let ref =
    <n>(self, t: n type):
    compile
    => default;

let share =
    <n>(self, t: n type):
    compile
    => default;
```

The general type-forming rule takes the **operand type** `T` itself as the
parameter to `RefTy` / `ShareTy` — not the universe `U_n` in which `T` resides:

```text
T : U_n
----------------------------
T |> ref   = RefTy(T)   : U_n
T |> share = ShareTy(T) : U_n
```

When the operand **is itself a universe object** `U_n`, the rule
mechanically specializes:

```text
U_0          = type
U_1          = type_1
U_n |> ref   = RefTy(U_n)        (n ≥ 0)
U_n |> share = ShareTy(U_n)      (n ≥ 0)
```

In particular:

```text
type |> ref   = RefTy(U_0) = type ref
type |> share = ShareTy(U_0) = type share
```

This is **not** `RefTy(U_1)`: `type` is `U_0`, and `U_1` is merely the
universe that classifies `type`. Confusing the operand with its classifier
would collapse distinct types (`uint8 ref`, `uint16 ref`) into one.

(`n type ref` / `n type share` are pure mathematical metavariable notation
for `RefTy(U_n)` / `ShareTy(U_n)`; they are **not** a legal source LHS and
**not** a source-parser `n type ref` / `n type share` expression.) The
universe-uniform routing — `type : type_1` being the instance that makes
`U_n |> ref/share` well-formed at every level `n ≥ 0` — is frozen, not
future work:

```text
rank(t ref)   = rank(t)
rank(t share) = rank(t)

CallSpace(RefTy(T))
  contains
    T -> RefTy(T)

CallSpace(ShareTy(T))
  contains
    T -> ShareTy(T)
```

Specialized to universe objects:

```text
CallSpace(RefTy(U_n))
  contains
    U_n -> RefTy(U_n)              (n ≥ 0)

CallSpace(ShareTy(U_n))
  contains
    U_n -> ShareTy(U_n)            (n ≥ 0)
```

The base case is immediate — no off-by-one recursion is needed:

```text
R_1 = RefTy(U_0) = type ref

CallSpace(type ref)
  contains
    type -> type ref
```

This is the instance needed for `t |> (type ref)` / `t |> (type share)`.

The type-forming members need no privileged actual-place access. Only the
selected borrow-forming defaults inside these borrow-type callspaces possess
actual-place privilege (`PrivilegedActualPlace(ref-family)`,
`PrivilegedActualPlace(share-family)`). Policy details remain schematic.

The borrow-forming member's source formal head is not ordinary move-in
parameter semantics:

```lang
mut let ref =
    (self, mut let object: t):
    s -> _: t ref
    => default;
```

Here s is schematic for one admitted concrete stage, not a resolved union;
projection and readiness obey the ordinary Policy owner.

The head's `object : t` displays the candidate head, Pattern, and policy so the
member participates in ordinary overload resolution. It must not be read as
`Read(actual) -> move/copy -> fresh ordinary parameter object`: at application
the builtin member may obtain the corresponding place from the actual. This is
the explicit formalization of the privileged actual-place semantics that the
generated/builtin `ref` / `share` callables already had, not a new arbitrary
place-reflection facility.

#### 2.0.1 Group and named-type observations

An ordinary OverloadGroup reference observes the actual group Place. It does
not implicitly borrow a candidate type or manufacture an OpenHere target.
A structural name denotes its named type, whose type reference and member
projections use ordinary Place, policy and capability rules.

Call projection uses the singleton type embedding into a candidate group.
Neither group callability nor an absent call candidate creates name freshness.
Type selection, borrowing and lifecycle observation retain their independent
relations; they do not recover a source Place from equal type values.

### 2.1 `@` yields a lifetime value, uniformly

```text
E@ : LifetimeValue
E@ = ReifyLife(NameOf(E), Pos(SemanticContinuation))
```

`@` is never a bridge from a hidden carrier slot to a `type ref`. A type-valued
binding evaluates to the complete closure `tau`; an ordinary/namespace consumer observes the projection `Core(tau)=Q`. Reaching the type-level
place is done explicitly with `t |> (type ref)`, including for a structural
named type, never by `@`. NameBinding is a binding relation and has no borrowable
wrapper or `.type` field. A by-value type projection followed by `@` does not
recover a Place.

`LifeName` is not a source spelling, NameBinding, Place, or ordinary
value identity. It is the semantic name whose lifecycle is observed. A bound
value normally has a stable LifeName; a temporary may receive a generated one.
This keeps place identity and lifecycle identity distinct.

#### First-class `LifetimeValue`

A value observation of `ReifyLife` produces an ordinary first-class semantic value, not an ephemeral
observation-only expression species:

```text
FirstClassLifetimeValuePrinciple:

ReifyLife(n, k) : LifetimeValue

LifetimeValue participates in the ordinary value formation machinery:
  binding
  storage in an ordinary value slot
  argument passing
  return
  Move or a clone-derived route when admitted by its ordinary Type/callspace
```

Each use remains subject to the same Type, Policy, capability, lifecycle, and
destination-legality judgments that apply to another value at that point.
First-class therefore does not mean unrestricted storage, implicit copying, or
automatic runtime availability. Binding or transporting `E@` carries the value
already reified at `Pos(SemanticContinuation)`; it does not silently re-run `@`
at the later destination and does not turn the destination into a hidden
LifeName, Place, or borrow view.

```text
StageDoesNotChangeLifetimeValueOntology:

TypeOf(ReifyLife(n, k)) = LifetimeValue
  independent of the stage at which the value is known

compile-known LifetimeValue
  =/= a distinct CompileLifetimeValue species
```

Whether a `LifetimeValue` can make a same-Type static-to-runtime transition is
therefore decided by the ordinary materialization theorem, not by first-class
status and not by a lifetime special case:

```text
RuntimeMaterializable(LifetimeValue)
iff exists an ordinary non-deleted callable realizing
    a legal same-Type static -> runtime transition of LifetimeValue
```

This is the general `RuntimeMaterializable(T)` rule owned by
[`../symbol-world/type-values-places-and-borrow-views.md`](../symbol-world/type-values-places-and-borrow-views.md)
§4.1. Applying `@` is still not an automatic mechanical pass. Once `@` has
produced a `LifetimeValue`, ordinary transport of that result uses the same
admitted Move and ordinary clone realization as another value.

#### 2.1.1 One semantic continuation

All compile, runtime and lifetime activity is projected from one
semantic continuation:

```text
SemanticContinuation K

pi_runtime(K)
pi_compile(K)
pi_life(K)

Life : LifeName × Pos(K) -> LifetimeValue
```

Compile-known and runtime-known values do not inhabit different lifetime
species. Stage says when a fact is known; it does not change the lifecycle
calculus or numeric/value identity.

K as a whole projects to lifecycle computation, forgetting the **business
Policy's Stage dimension**. It does not create compile and runtime lifecycle
timelines. The projection retains already selected operations and MoveEffect,
names, dependencies, activation, control, cleanup and delivery correspondence.
It never selects again after forgetting Stage. Compile evaluation is the
ordinary hosting of this value program, not another stage axis inside its world.
Reify resumes the same E/K and creates no separate lifecycle world.

Current established names and structural material can be computed before their
machine action runs; unknown business values remain unknown. E uses the actual
legal frontier and guarded continuation, without future-path enumeration.
Constructing guarded branch material does not commit both branches' effects
or permit lifecycle code to prune paths using unknown business values. A legal
O path rewrite revalidates affected projections at their new positions.

`lifetime pre/post` programs are projection-complete regions. Their ordinary
evaluation and check/admit boundaries remain, but transcription adds neither
another @ nor another companion layer to their lifecycle-side operations.
Established projected material must retain that completion provenance through
composition; it is not inferred by matching names or traversing source text.

#### 2.1.2 NameView, origin, and finite observation

`NameView<T>` below is mathematical notation for the structured companion of
T carried by **ordinary rank-0 lifecycle Objects**. It does not prescribe a
source type constructor, introduce another Object category or type axis, or
define a lifecycle-specific object ontology. Origin, region and operation
material use the same unified Object/Pattern/Policy mechanism as §2; an
implementation may choose ordinary representations and API names.

For every semantic value operation `f_V : T -> U` that is name-projectable,
there exists a unique semantic name-level companion:

```text
NameView<T> {
  origin,
  region,
  name-observable field/operation companions
}

NameProjectable_T(f)
  => exists unique f_N : NameView<T> -> NameView<U>
```

Value and name projection must commute:

```text
@(f_V*(x))     = f_N*(x@)
@(field_V*(x)) = field_N*(x@)
(x.field*)@   = x@.field_lifetime*
```

The stars mark the original selected operation and its corresponding partner;
they are metanotation, not a source operator or second overload selection.
The name-level companion does not re-execute the runtime field operation.
Companion existence, uniqueness, and commutation are closed semantic
requirements. How companions are generated, stored, or represented in an IR or
registry remains implementation-open. Actual Pattern/structure formation must
synchronously establish sufficient companion material for this law. A missing
consumer is Unavailable, never permission to fabricate the partner or silently
waive the law. Dot still enters field::adl, and any forwarding is real ordinary
registration or Pattern extraction, not a private field resolver.

`origin` observes a LifeName link, explicit None, or an actual unknown boundary.
It records semantic provenance rather than
source text or an address. Origin ancestry is generated coinductively by the
default generator of the semantic construction that produced the name;
omission is not `None`. `None` is an explicit termination proposition. Each
`ReifyLife(n, k)` is nevertheless a finite structural observation. A later
`.origin` projection issues a new request; `@` never eagerly unfolds the whole
chain.

Unknown marks the information boundary of established material. A missing
producer/observation is instead Unavailable; it establishes neither a fresh
unknown root nor None. Boundary programs perform the necessary available
origin.origin traversal (§10); callers need not manually spell an arbitrary
number of links in from/live_with.

```text
OriginDefaultPrinciple:
  if no explicit origin(n) = None fact applies,
  origin(n) continues through the default origin generator
  of the semantic construction that produced n

  origin(n) = None is a terminating fact,
  not the interpretation of missing origin information
```

Construction, field projection, copy, move, and call-result formation may each
provide the applicable name companion or lifecycle post. The general default
therefore does not classify an ordinary non-borrow value as origin-free merely
because no borrow-root rule applies to it.

Genuinely new function-entry parameter names start with the established
base origin/borrow world, then the recursively composed Pattern-local lifetime
default material, then the function's ordinary explicit lifetime pre program.
Previously held names (including InPlace captures and already suspended
continuation material) inherit actual preceding Post and are not regenerated.

```text
EntryDefault(F,theta) = U_default_base(theta)
                     ; Instantiate(PatternDefault(P_formal),theta)
EntryPre(F,theta)    = EntryDefault(F,theta); E_pre(F,theta)
omitted pre         => E_pre = identity expression material
```

The semicolon above has the same serial lifecycle-expression meaning as
`lifetime expression1; lifetime expression2` in ordinary lang programs. It
does not abbreviate an unordered bag of constraints, a universal Hoare-logic
solver, or a second world interpreter. Pattern-local `lifetime default E` is
stored as a name-parametric ordinary compile expression description inside
RHS structural Pattern material `(...)$`, then instantiated into actual
name expressions at a genuinely new formal's Pre. Merely forming the type
does not execute an action on an absent instance. Selected Pattern/structural
evidence must identify the mapping; equally spelled or equal-valued residents
cannot replace it.

**Two distinct override granularities.** A `lifetime default` annotation is
a **complete local declaration** of the default relation items it covers at
that Pattern node. A later declaration on that node replaces its earlier
local relation material as a whole; an ancestor declaration, applied after
descendants, replaces each overlapping *whole relation item* rather than
incrementally editing its fields. Independent child relation items not
addressed by the ancestor are still composed. In contrast, `lifetime pre`
consumes the already formed default **expression material** and serially
executes only the modifications expressed by its ordinary lifecycle program.
Unmentioned default relations remain in effect. These are one ordinary
expression/continuation semantics, not distinct core patch solvers. This
reuses the same serial material interpretation required for formed-body
action descriptions and automatic Post (section 10).

For a Pattern tree node n with children n_i:

```text
D(n) = OverrideWholeRelationItems(
         Compose(D(n_1), ..., D(n_k)),
         LocalDefault(n))
PatternDefault(P) = D(root(P))
PreMaterial_K(F,actual) = E_life(
   U_default_base; Instantiate(PatternDefault(P), theta_K(actual)); E_pre)
```

The name scope of LocalDefault(n) is the named n and its registered structural
descendants only, not sibling/ancestor names. Root `srt@` instantiated for
object is `object@`; nested `srt@` inside `wrapper` becomes
`object@.srt`. The same map applies to `.ref` and `.val`. Structural
registration/field companion evidence is necessary. If several successful
Pattern derivations give different required mappings, the consumer needs
proof of invariant observation; otherwise the required default is Unavailable.

**Base ref/share origin assumptions without an extra declaration.** For a
fresh formal name's traceable owned access tree A (such as `object@.path`),
every ref/share terminal has a parent origin at a *specific exterior
frontier* `ExternalUnknown(A)`: outside the owning object's tree, not an
arbitrary fixed-depth unknown nor an invented `None`. The actual origin
chain may be substituted from the caller and retains its proof frontier:

```text
DefaultOrigin(object@.path.ref)   -> ExternalUnknown(Outside(object@))
DefaultOrigin(object@.path.share) -> ExternalUnknown(Outside(object@))
Region(handle) strictly-within Region(origin(handle))
```

Here strictly-within means the established strict min/max endpoint relation,
not event-set inclusion. Different independently introduced `ref` defaults
have distinct origin roots by default, subject to explicit lawful local
Pattern defaults or explicit Pre. `share` defaults are *may-alias*: the entry Pre deliberately **forgets**
any distinction/equality that would not be needed by that interface, by
ordinary lawful lifecycle relation abstraction. This forget step neither
asserts that two share origins are identical nor destroys established
caller provenance; it simply requires neither equality nor inequality for
a fresh formal share pair. The real caller-to-formal map must still justify
the weaker relation, and the body cannot regain a forgotten distinction
without actual additional producer evidence. A share source cannot obtain
exclusive access from this loss of alias precision.
Known source/caller name/region evidence substitutes for an exterior unknown
where available; otherwise the already formed, boundary-scoped unknown and
its region/access/check obligations are forwarded outward, never silently
discharged. When a ref/share reaches the current function boundary, a
successfully admitted strict region requirement establishes that its source
outlives that boundary. An unresolved unknown does not itself prove the
inequality, access rights, or infinite lifetime.

```text
DefaultOriginUniverse:
  independent ref roots -> distinct unless a lawful relation overrides
  share roots           -> may-alias; no default equality or inequality
  every source          -> exterior unknown at its traceable owner boundary
  known external name   -> instantiate actual origin and region
  external unknown      -> preserve the scoped boundary and pass obligation
  explicit origin=None  -> terminates only when actually established

  access tree -> only already registered and permitted borrow/field routes
  structural membership alone !=> access authority
  missing relation !=> origin=None
```

Actual caller Pre.check and body Pre.admit consume the final serial expression
material with the original mapped source, generation, Region, Color and
access/ownership observations. A false required relation rejects before
commit; needed unavailable evidence is Unavailable. No default or Pre
expression can create a name, replace historical origin, extend a region, or
manufacture access/Move/Drop authority.

The serial explicit Pre expression has finite completed description material
when its actual computation terminates; any notation `Delta_pre` below is
shorthand for the resulting change material, not a primitive unordered
constraint patch. `|Delta_pre|` measures the finite **description material** delivered by one
completed formation/observation: its ordinary bound names, relation rows and
finite regular descriptor graph. It is not the cardinality of the descriptor's
path denotation. A finite descriptor using repetition may describe infinitely
many potential finite walks. It does not enumerate or grant simultaneous
operations on all of them. Each actual requested operation still requires its
real access-tree route, target/generation, Region and Color premises; descriptor
state reuse merges none of those identities.

Only the admitted regular relations and their justified algorithms may be used.
Finite representation proves neither decidability of an arbitrary infinite fact
set nor termination of an arbitrary Pre program or numeric-range query. A
nonterminating/unavailable computation supplies no completed Delta_pre. This
description-versus-denotation distinction also applies to §11's regular access
material and leaves the implementation/proof obligations of §14 intact.

Neither origin nor NameView is automatically added to a business Object's
Norm. Lifecycle Objects themselves have ordinary content normalization of
their fields and retained identities; equal business values do not merge names,
Places, lifecycle generations or credentials.

#### 2.1.3 Region and lifecycle events

### N01 — Region 的离散事件基础 [定义]

把已经按普通控制、所有权与结束事件规则规范化的语义续体记为 `N(K)`，每个语义指令位置有确定行号。这里的行号不是源文件行号，也不是机器地址。

对某个实际名字及其 generation，在其适用的续体/分支解释下：

```text
L(K) = normalized semantic instruction positions
Region_K(n) ⊆ L(K)
event kinds relevant to Region = use | consume/move | drop
```

Region 是这些事件位置的离散集合；分支、合流、循环和递归改变位置的解释及可达性，不改变这个基础。重复出现的静态行号仍由既有 activation、generation 与正则续体材料区分，不得合并不同次的名字实例。

不得把 Region 定义为实数时间、机器栈区间、任意 CFG 本体，或为了满足借用关系而扩张的 `SupportClosure`。对空集合或不可得端点，不得擅自填入无穷大、零或新 unknown；相应消费者必须有既有事件材料或报告不可用。

### N02 — 析构先定与纯切片 [不变量]

普通控制流、MoveEffect、析构、使用和消费事件先确定；生命周期观察随后取得它们的投影。同位置的优先关系也先确定。生命周期程序不能移动析构点以满足自己生成的约束。

对同一名字/generation，若一步只缩小未来可达续体，没有新的名字生产、Move 或其他状态变更，则当前 Region 是此前 Region 的限制：

```text
R_after = RestrictReachable(R_before, actual continuation advance)
R_after ⊆ R_before
```

只有确实删去原 Region 中位置时才为严格子集。分支材料可以预先形成受守卫的描述，但不能据此执行未选分支、提交未来事实或利用未知业务值裁剪未来。

切片上的既有颜色随限制继承。不可达部分不再参加当前观察，不等于撤销历史染色。优化器改变 K 后必须重算/验证对应位置关系，而不是原样搬用旧行号证明。

### N03 — 父子借用只要求首末偏序，不要求集合包含 [定义]

在产生子借用的同一合法事件解释下，对父 F、子 C：

\[
R_C\prec R_F
\quad\Longleftrightarrow\quad
\min R_F<\min R_C\ \land\ \max R_C<\max R_F.
\]

这是生命周期端点的严格嵌套关系，不是 `R_C ⊆ R_F`。例如：

```text
R_F = {1, 4, 9}
R_C = {3, 6}
```

满足父子关系，且允许串行的父、子、父、子交错使用。父 Region 无须补入子事件。

子借用生产时，其 Region 已由该次续体材料决定；相应偏序在生产 Pre 检查。后续纯切片保留已建立的生产事实及观察对应关系，不把每个剩余集合重新当成新借用生产来测试或生成 origin。

端点偏序只是纯关系计算，不是借用生产的充分性证明。`initialize_bounded_region` 及任何访问/借用 Pre 必须独立取得：真实已选生产动作与父子来源、实际名字/generation、该次 activation/守卫分支解释，以及输入 Region 对当前 K 的合法来源和修订对应。外部提供的事件集合、同一 continuation 身份、同样的数值端点或 `strictly_within = true` 都不建立这些事实。共用原始 K 身份的分叉快照不能据此互相发放权限；K 改写后，消费者须验证现有 E/K 对应及其依赖仍适用于当前动作，不能原样采用旧位置证明。所需事实为假则在提交前拒绝，缺失则 Unavailable；端点查询不产生 `LifecyclePreProof`、origin 或访问权限。

Rust 载体的 `contains_event` 只查询完整 `(cut, action ordinal)`，`contains_cut` 只查询该 cut 是否有任意记录事件。原始 cut 不隐式补 ordinal=0；`from_events` 只接受完整事件位置。两种成员查询及集合构造都不建立上段的消费者准入证据，也不是第二种 Region/K 本体。

该条件不免除具体操作的其他 Pre，例如资源版本、有效索引凭证或实际访问权限。不得以“独占借用”名义另加一个无 Color 的普遍父冻结规则。

### N04 — 默认入口世界与已有名字的继承 [不变量]

一般函数的新入参由基础外部来源默认与完整结构 Pattern 的递归 lifetime default 表达式材料生成，随后串行执行实际显式 lifetime pre；显式 Pre 只修改它执行时明确涉及的关系，未提及的默认关系保留。ref 默认来源互异，share 默认 may-alias；二者都以各自真实访问树之外的 unknown 为外向根而保留待外层履行的 Region/来源义务。默认来源不自动建立任意内层父子关系，也不证明 Movable。

已有名字不重新默认生成。InPlace 自动捕获继承上一刻的 Post；空动作切片满足同一观察词汇下的 `Pre(id)=Post(id)`。实际交接之后，后续续体持有材料的 AutoPre 由成功交接 Post 经合法名字对应形成。

父子借用同时传入一个未声明该关系的一般接口而失败，不能统一解释为 Color 冲突。捕获与入参的语义平权也不等于对已存在捕获清空历史。

A killing Move ends old-generation custody and starts destination custody at
one common commit. Its source consume event belongs to the source Region;
formation alone is not an invented destination use. Deeper origin is preserved,
not replaced with an old-name origin edge. A selected clone-derived path still
performs share/rebind -> clone -> fresh complete result -> terminal Move;
its actual producer Post determines origin. Preserve retains the subject and
adds only the actual event, without a second owner or implicit clone.

#### 2.1.3.1 Instance killability and move legality

```text
Killable_K(n)                  -- property of this lifecycle instance
MoveEffect_K(n,m) ∈ {Kill, Preserve}
Movable_K(n,m)                 -- legality at this action's frontier
```

These are independent judgments. Killable is not a Type trait; equal type
values, ZST layout, compile knowledge and compile provenance do not decide it.
Movable requires the ordinary selected action's access, borrow, capability,
origin and lifetime Pre. Movable_K(n,m) != Killable_K(n); Movable does not
imply Kill, and Move does not definitionally mean death. Nonkillability proves
neither Movable nor Copyable.

The effect is fixed before lifecycle observation. Kill ends the old generation
and begins the transferred generation at the same continuation cut, preserving
its deeper origin (§2.1.3). No separate destructor/drop is inserted for the
consumed old generation. Preserve requires the narrow proof that the subject
cannot legally die or that preserving it is observationally equivalent under
all admitted observations, including @, borrow/access capabilities, origin and
destructor effects. It is a legal Move effect, not clone, copy fallback, failed movement or
special non-owning syntax. Preserve does not invoke clone; a copy-derived
share/rebind -> clone -> Move(fresh result) path creates a fresh complete
object and is a different ordinary operation.

Alive/dead status alone introduces no observable branch. Death with no effect
under any admitted observation adds no observable semantics. If killing would
invalidate an established legal language fact, that Kill continuation is
illegal and must be rejected at Pre, before any mutation.

The ordinary realization is killing; Preserve is confined to that proved
exception. A stable compile root or global Val2 resident cannot be killed merely
by local transport of its observation. Moving a local instance into a legal
destination starts the destination generation; it never extends the old local
name's lifetime into a global one. Failed Pre cannot switch the fixed effect.

Compile, runtime, compile, type, Pattern and borrow instances all participate.
An ordinary ordinary type instance in the established domain follows its existing global-survival rule; this is not generalized to every dependency-bearing closure-generated tau (§8).
a compile-local type temporary can end. A stable compile result root, a local copy,
and a globally retained equal resident remain different lifecycle subjects.
Construction OpenHere neither extends a lifetime nor makes a subject killable.
The existing stable-instance rules include ordinary stable type values, compile
type values outside the current OpenHere domain, still-accessible Val2
residents of closed type structures and other stable/global residents.
Their local transport cannot kill those stable subjects; legal movement uses
Preserve while still checking Movable. These are instance/frontier facts,
not a type-wide Copyable or Killable trait.

Every expression occurrence, including a temporary without a Place, has its
continuation-relative LifeName. Lifecycle Pre/Post applies even when no source
@ is written. Value/name companions observe the same action positions; @
does not expose a compiler execution or scheduling trace.

#### 2.1.4 Cleanup placement precedes lifetime observation

```text
CleanupPlacementBeforeLifetimeObservation
```

Ordinary control-flow, ownership, and end-event semantics place cleanup/drop
events first, fix their points, and linearize remaining unordered events
at each same point in reverse declaration order. This last tie-break never
moves a point or overrides NLL/with/other precedence. Lifetime observation
then describes that fully fixed continuation. The NLL/lexical defaults and directed with constraints are defined in
[mechanical cleanup](../mechanical-lowering/mechanical-argument-passing-and-move-fixed-point.md#15-cleanup-placement-and-with).
It does not move cleanup to satisfy a constraint and does not participate in a
cleanup/lifetime fixed point.

#### 2.1.4.1 Atomic decomposition and destructor continuation

Construction and consuming extraction use the Pattern owner's atomic identity
boundary. A destructor receives complete children after parent decomposition:

```text
Pre(drop/extract)
  -> atomic ExtractCommit of the complete parent
  -> parent generation ends where the selected action so requires
  -> complete child identities become established
  -> ordinary destructor continuation
  -> children's ordinary NLL / with / use / move / drop
  -> Post / subsequent lifecycle observation
```

There is no partially destroyed parent, live-field bitmap or separate
destructor evaluator. Matching proofs and non-consuming views are not
ExtractCommit. An affine accessor likewise does not prove parent death.
Cleanup placement and same-point ordering are settled before @ observes this
continuation, and neither destructors nor @ re-solve placement.

#### 2.1.5 Pre-check and post-commit

Every fact-changing lifecycle action has the order:

```text
Pre(action, state)
  -> failure rejects before mutation
  -> perform action
  -> Post(action, committed_state)
```

The corresponding ordinary lifecycle program must reduce legally and deliver
the required result. This is the meaning of checking, rather than another
checker ontology. It does not tentatively publish facts and discover a failure
at function end. A failed precondition leaves prior state unchanged. Scratch
computation is not committed history, and earlier committed external effects
are not generally rolled back by a later failure.

#### 2.1.6 Call-boundary summaries

A contract clause hosts a complete ordinary compile expression:

```text
lifetime post a@ == b@ ref::lifetime::
```

@, == and ref::lifetime:: use ordinary interpretation and selected calls. Local
declarations, branches, recursion, calls and associated injection are permitted.
Intermediate values need not be NameExpr. The final delivery is a complete
single self-root type with the boundary's associated check/admit behavior (§9).
No special formula AST, separate logic evaluator, arbitrary implication solver
or user-written proof Object is required. The caller consumes that behavior
without expanding private internal history.

| Clause | Caller | Inside implementation |
|---|---|---|
| Pre | check | admit |
| Post | admit | check |

Ordinary calls map actual parameters, formed captures, call-bound RequiredCapture
material, result positions and
activation. Caller Pre check precedes the action; internal Pre admit cannot
repair its failure. Internal Post check establishes the delivered guarantee;
caller Post admit neither replays internal actions nor duplicates consumption.
Explicit Post constrains actual results/history; it cannot replace history.

An implementation body omitting Post enters the formation-time structural
summary transformation of §10; reached actual exits separately validate that
summary. It need not execute future branches to expose the contract. A handwritten Function
Pattern/function-pointer interface has already forgotten external structural
source authority; it cannot recover hidden implementation structure to fill in
a contract. The following defaults apply to that interface, not to body-based
automatic Post. An empty clause placeholder proves neither true nor NoDelta.

For a Function Pattern LHS interface F and actual callable contract A,
omission and compatibility have these meanings:

```text
Entry_new_parameters(F) =
  E_life(U_default_base; Instantiate(PatternDefault(P_formal),theta); E_pre)
omitted pre  => E_pre = identity  // new formal names only
omitted post => Post_F = NoDelta(S_before,S_after)
NoDelta = unchanged state on every lifecycle observation of the contract

LifetimeContractCompatibility(F,A):
  Pre_F implies Pre_A
  Post_A implies Post_F
```

These implications use the ordinary symbols, Region/Color/name mapping and
entry assumptions. Omitted Post is a strong guarantee: an actual postcondition
that cannot imply NoDelta rejects the interface assignment/extraction immediately,
before any invocation. Missing implication evidence remains unavailable.
Interface input defaults do not reset actual existing InPlace captures, whose
AutoPre inherits preceding Post under §2.1.8. Projected Function Pattern inputs
are explicit new input material, not restored hidden closure sources.
This does not inspect require/body or enumerate overloads by lifetime.
The [Pattern owner](../patterns-overload/pattern-relations-observation-and-extraction.md#111-directional-function-pattern-consumers)
defines LHS extraction and RHS requests. RHS cannot add pre/post, but after
unique selection it still checks the callable's actual Pre, commits once under
the common transaction and publishes its Post. Dynamic failure never reopens.

The [call owner's self projection](../symbol-world/function-object-call-model.md#63-one-coherent-passing-transaction)
uses this same boundary: whole-self and projected capture input have compatible
observable consumption in one transfer transaction, without duplicate Kill.
Ref/share capture positions move handles, preserving their actual targets and
generations; Preserve is neither Copy nor an implicit clone.

Contract implication retains the displayed direction, over the exposed
observations and mapped entry assumptions. It is an obligation on the actual
ordinary programs and supported family, not a decidable algorithm for arbitrary
program equivalence. Missing evidence cannot count as success. In particular
non-NoDelta effects cannot be hidden in an omitted-Post Function Pattern, even
with compile-known Val1 of the projected callable.

#### 2.1.7 Region Color

Color remains a future constraint dimension, but its inheritance law is
closed:

```text
Color is a future-continuation constraint
Color(child) includes inherited Color(ancestor)
observation may slice the inherited region but may not remove the color

FiniteColorPrinciple:
  GlobalColorVocabulary is extensible

  for each committed compilation semantic universe Sigma:
    ColorSet(Sigma) is a finite snapshot

  Color is not an arbitrary proposition carrier

  compatibility / exclusion / exchange
    are finite, mechanically decidable relations over
    ColorSet(Sigma) and bounded observed origin topology

ColorExtensionInterface(new_color):
  declare compatibility(new_color, existing_color)
  declare exclusion(new_color, existing_color)
  declare exchange(new_color, existing_color)
  freeze those finite relation rows when Sigma is committed
```

Color is not automatically added to business Object core or Norm. Its values
and region fields are ordinary lifecycle Object content. Its concrete carrier and
source syntax remain implementation/surface questions. The global vocabulary
is not a closed language-wide enumeration: later definitions may contribute
colors and their finite relation rows. Once a compilation semantic universe is
committed, however, its ColorSet and relation tables are finite and fixed for
that compilation. Extension may express broader memory-access models and
computation-reordering constraints. Finite Color relations remain a bounded
mechanical foundation; this does not restrict full contract programs to
conjunctions of Color atoms or make Color an arbitrary proposition black box.

### N05 — Color 的引入边界是独立访问 [不变量]

普通串行父子/兄弟借用不自动新增排斥性 Color。访问一旦能够脱离当前续体的串行支配、在另一执行上下文独立发生，交接/发射 Pre 必须检查所需 Region 关系并建立相应 Color。

对实际访问域，具有冲突颜色的父子、兄弟及其他相关访问必须区间排斥；兼容颜色可以重叠。此处的区间是既有 Region/访问协议的时序解释，不新增一类可独立任意移动的“授权时间轴”。

不同执行主体的局部行号不能未经共同 K 的控制、交接或同步关系直接比较。Color 不证明目标仍存活；Region 不证明独立访问彼此兼容。

`async`、挂起、恢复、换设备本身均不自动意味着并发。只交出唯一执行权的续体交接，不因 `wait` 这个拼写就染上并发色。

### N06 — 兼容颜色不等于可交换操作 [不变量]

允许重叠独占染色，只说明规定访问协议的独立使用可以合法并存；不说明两次操作可交换、没有竞争、都能成功，或形成跨对象全局原子性。

Color 词汇及兼容/排斥关系继续使用有限、定向、机械可验证的基础。更强的库行为不变量不能靠增加一个能容纳任意命题的 Color 绕过证明。

#### 2.1.8 Empty slices, inherited AutoPre and atomic Move repair

The identity action id on a lifecycle continuation preserves its complete
established lifecycle observations. On the same material and name/generation
vocabulary, its Pre and Post are equal, not merely compatible or related by a
one-way implication:

```text
Sigma_s -- id --> Sigma_s
Pre(id) = Post(id) = Sigma_s
```

This equality retains origin, generations, regions, Color, access, history and
outstanding cleanup duties. It creates no fresh default state or missing fact.
Unknown required observations remain unavailable. Empty slicing is this identity
law applied between adjacent actions in the same lifecycle world:

```text
K = K_minus ; epsilon ; K_plus
Slice_epsilon(Sigma_s) = Sigma_s
Post(K_minus) = Pre(id) = Post(id) = Pre(K_plus) = Sigma_s
```

InPlace automatic captures already exist at this slice. With omitted explicit
Pre their AutoPre is the preceding Post with legal name projection, retaining
origin, region, Color, access and history. Thus the inherited capture material's
identity interval has equal Pre/Post; this is not a fresh invocation-default
calculation or a claim that the closure body is id. New input names formed at
entry use the default rules of §2.1.2 only for those names. An explicit Pre constrains
the actual inherited world; it cannot erase history. An Ordinary closure has a
real formation/call boundary and cannot claim epsilon continuity by syntax.

An actual handoff additionally proves the composite selected transaction:

```text
Pre_MoveRepair(Sigma_s, names, rho, destination)
  = ordinary Move/cleanup/access/capability/resource premises
    and legal execution of the source-derived automatic/explicit repair plan rho
    and Valid(projected repaired successor)
Sigma_s -- Move[rho] --> Sigma_prime
commit publishes only Sigma_prime and its complete successor callable
AutoPre(k) = RenameAndProject_(s->k)(Sigma_prime)
rho omitted = source-derived automatic repair (id only if no replay is required)
```

The same identity law generates the lifecycle interface of the already
established live handoff material, after the actual Move/repair and legal
correspondence have been validated:

```text
H_k = RenameAndProject_(s->k)(Sigma_prime) on held handoff material
Pre(id_H_k) = Post(id_H_k) = H_k
AutoPre(k) restricted to held handoff material = H_k
```

id_H_k denotes the zero-action delivery interval for that material; recognizing
it inserts no extra Move, binding, clone or repair. Handoff material keeps the
actual successful Post rather than receiving new default origins or obligations.
The equality compares observations in the successor vocabulary, not the original
self and new k as identical Objects. It does not describe execution of k's body.
New resume parameters still receive only their ordinary entry rules when formed;
unfulfilled RequiredCapture slots remain specifications until their admitted call.

For an actually selected identity repair, Pre(rho=id) = Post(rho=id)
in its actual repair-entry world. Move[id] still includes the selected Move and
may change generations/regions or ownership under the ordinary rules. It is not
id on the entire source world and is not evidence that destination Valid holds.
The equality of these identity intervals does not extend to a nonidentity
Move/repair, which retains its own validated state transformation.

Trivial killing Move resets the first-level region (region_0, called Region_1
in §2.1.3), retaining origin and deeper regions. In particular the destination
must satisfy `region_0(n) < origin(n).region` wherever required. A repair such
as `r rebind = v ref;` must supply the ordinary exclusive/valid/access premises;
the suspending side forms and validates the source-derived plan, including
any explicit repair's exact responsibilities. Failed or unavailable repair exposes neither an illegal
intermediate callable nor successful Post. Earlier unrelated committed effects
are not rolled back. Resume receives an already legal concrete callable.

This is the existing common transaction and four-way contract protocol, not a
second lifecycle solver. Only actual retained handoff boundaries incur Move
obligations. [Effect steps](../control-flow/effect-steps-and-continuation-handoff.md)
own Wait/handler control; [heap protocols](heap-stack-and-storage-responsibility.md)
own storage-support/release-duty transfer. Neither can rewrite old origin or
Color to discharge a missing premise.

### N10 — 跨点保存取决于存活，不仅是后续使用 [不变量]

跨挂起点需要交接的材料由已确定的生命周期切片给出，包括存活依赖和未完析构责任，而不是仅扫描挂起后出现哪些变量名。

子借用跨点存活时，其名字依赖也必须满足相应跨点存活关系。该结论来自 N03 的偏序，不通过扩大父 Region 得到。

保存范围是重演当前生命周期切片所需的最小合法材料；“最小”相对于保留的实际生产依赖和契约，不要求编译器求解任意程序的全局最小化问题。

### N11 — repair 按第一层 origin 递归分解 [不变量]

对本层跨点名字，执行以下确定的责任分类：

| 材料 | 本层责任 |
|---|---|
| 入参、捕获 | 外部挂起/调用交接位置负责修复；本层消费修复后的实际映射 |
| 局部名字，第一层 origin 明确为 None | 保存已产生的值材料，停止倒追其生产过程 |
| 局部名字，第一层 origin 非空且有本层可追踪生产来源 | 沿原已选生产表达式形成必要的重演依赖闭包 |
| 已建立的外部 unknown 边界 | 停止跨边界猜测；使用其已有外部契约和交接材料 |
| 缺少必要来源/伙伴/可用材料 | 不可用或拒绝，不能伪造 None/unknown |

本层挂起点也负责其被挂起表达式的输入槽修复，义务沿嵌套调用递归分解。不得反向展开外部函数的全部私有历史或把所有层的修复责任抛给最外层用户。

`unknown` 不等于 `None`；未记录 origin 也不等于 `None`。

### N12 — 重演保留表达式—已选动作—伙伴对应 [不变量]

每个参与重演的节点保留原语义表达式、已选定实现、实际操作数/名字对应及其生命周期伙伴。自动 repair 通过该对应重演表达式本身，不先降成一堆内建 ref/share 原子动作再搜索如何拼回。

共同子表达式保持实际共享与单次生产责任；实际分支使用对应生产关系，不执行所有候选来源。循环/SCC 必须使用已存在的正则描述与组合契约，不能凭有限表示宣称任意反向展开必终止。

重演是新的实际执行发生，不是重新提交旧动作、撤销旧历史、重新选择旧重载或再次使用旧成功回执。真正的新普通调用仍按其真实调用位置的既有规则处理。

### N13 — 保存旧值不要求 Copyable；重演不要求额外纯度 [不变量]

局部 origin-None 根的保存接近保留 SSA 旧版本的语义材料，不调用 `copy/clone`，不创造两个可独立消费的所有者。Move、实际名字/generation 对应和 cleanup 义务仍只交接一次。

重演对象生产式不等于复制原资源所有权。普通 Pre 仍然必须成立，但不得另设“含资源/IO 所以不可自动重演”的纯度白名单。

IO 获得的独立值或指向实际外部 unknown 的材料通常是停止倒追的边界；若某个实际生产式处于重演闭包且有 IO 副作用，新增 IO 是允许的真实可观察效果。不得缓存旧 IO 成功、声称额外效果不存在，或隐藏其 effect 形状。

### N14 — repair 是 Move 内部的交接计算 [不变量]

```text
prior valid slice
  -> ordinary selected Move with automatic/explicit repair
  -> complete valid successor and mapped Post
  -> successor AutoPre
```

不得先向外发布非法续体再修复。没有需重演的部分时自动 repair 可退化为 id；省略 `@{}` 不再一律等于仅尝试 `Move[id]`。

显式修复继续是普通语言程序。其与自动片段必须具有明确、单次的生产/消费责任；不得默认“全部自动一次，再全部手写一次”，也不得默认手写代码具有绕过独立 Pre 的权力。

自动 repair 重演的可信验证可能再次失败或发出 effect，按一般和模式及 effect 协议处理，不虚构成功后继。修复义务仍由形成/交付续体的一侧完成，resume 侧接收已合法形成的材料。

### 2.2 No implicit borrow formation

The overload boundary stays:

```text
NoImplicitBorrowFormation
```

Candidate adaptation, policy migration, and automatic argument passing never
form a borrow merely because a formal requires `T ref` or `T share`. The actual
must already contain the explicit `ref` or `share` result; a `@` result is a
lifetime value, not a borrow. Fixed points and legal weakening of an existing
borrow do not form a new target.

Borrow-constructor composition remains a `ref` / `share` fact, owned by
[`../symbol-world/type-values-places-and-borrow-views.md`](../symbol-world/type-values-places-and-borrow-views.md)
§5.3:

```text
Borrow_k(Borrow_j(q)) = Coerce_{j->k}(Borrow_j(q))
Target(Coerce_{j->k}(v)) = Target(v)
```

Equal-capability `ref ref` / `share share` are fixed points, `ref share` weakens,
and `share ref` has no candidate. Retargeting type values have the symmetric
fixed points:

```text
type ref rebind rebind   = type ref rebind
type share rebind rebind = type share rebind
```

Prospective navigation identity and formed-borrow identity are also distinct:

```text
ProjectionCoordinate(parent_place, selector)
  != ProjectionSlotIdentity(parent_resident, selector)
```

The logical coordinate remains available for later navigation. A projected
borrow binds the parent-resident slot selected at formation, including a slot
whose contents are `None`; `let` may populate that same slot without retargeting.
Wholesale parent replacement may invalidate that borrow but never retargets it
to a replacement slot at the same coordinate. Only `rebind` selects a new
target. Generation/version representation remains deferred.

## 3. Escape checking

A borrow view must not be carried where it outlives its own valid region. The
positive obligation is:

```text
Escapes(view, destination)
  = not LifetimeOrderAndAccessLegal_K(view, destination)

Escapes(view, destination)  ->  the storing/returning operation is rejected
```

The destinations subject to this check are the ones that can outlive the origin:
storing into a longer-lived place, returning from a callable, capturing into a
materialized callable entity, and installing into global namespace material.

Compile invocation-generated names obey this same check. Stable invocation identity
and cache retention do not extend result residency or borrowed-target regions.
Admitted open input dependencies may bound an output's opening qualification;
that meet is not a replacement for ValidRegion. Ordinary compile directly returns
its instance type; a borrow retained in its Val2 must be valid in the result's
region. A borrow of an untransferred expiring local is rejected.
Result completion transfers only owned material and never promotes targets along
ref/share/rebind edges. See the
[invocation owner](../static-evaluation/compile-instance-invocation-and-result-delivery.md).

### 3.1 `ValidRegion` is a borrow-lifetime judgment

`ValidRegion` is determined by the target lifetime, the holder/destination
relation, and the view's ordinary capability. It is never a construction-Open
region:

```text
ValidRegion(type ref)   = target endpoint order + actual ref access premises
ValidRegion(type share) = target endpoint order + actual share access premises
```

The two views differ in write capability, not in whether their pointee is Open:

| view | carries | requires pointee Open | may leave the target's lifetime |
| --- | --- | --- | --- |
| `type ref` | read and policy-bounded write | no | no |
| `type share` | read/observe only | no | no |

Lifetime validity does not establish write permission after Close. Initialized
type refs preserve their original borrowed generation/opening subject; direct
mut and explicit open-to-mut writes require current OpenHere as well as
Writable. Close defeats both routes and saved-ref writes. Neither view makes
a closed value admissible as the left input to `*`. See type/ref owner §5.2.2.

### 3.2 Borrow validity never discharges construction openness

The positive separation is:

```text
Γ ⊢ r : type ref  does not imply OpenHere_Σ(Read(r))
OpenHere_Σ(v)      does not imply that v has a writable carrier
OpenHere_Σ(v)      does not imply the current computation flow re-enters v
```

The third line separates static openness from live evaluation flow: reentry is
`OpenEvalReentry_κ(v)`, which additionally requires an active evaluation edge
into `v` — a stored `SymbolicReferenceEdge` (including `Self_τ`) alone never
establishes it. The reentry criteria are canonical in
[`../symbol-world/type-values-places-and-borrow-views.md`](../symbol-world/type-values-places-and-borrow-views.md)
§2.1.1.

A consumer that performs `*` must query `OpenHere_Σ(old_value)` even when the
value was read through `type ref`. The place-level `*=` wrapper in
[`../symbol-world/structural-type-formation-and-composition.md`](../symbol-world/structural-type-formation-and-composition.md)
§8 performs two checks independently:

```text
OpenHere_Σ(Read(r))
Writable_Γ(Target(r))
```

Returning or storing `type ref` asks only the ordinary escape question of this
section. A later `*=` may fail because the then-current value's open
window has closed (`WindowLive_Σ(v) = false`) even
though the reference remains lifetime-valid. Weakening to `share` surrenders
write capability, but does not alter the value's anchor or window state and never extends the
target lifetime.

## 4. `@` and lifetime rules never reselect an ordinary call

The boundary between lifetime work and ordinary overload resolution is frozen:

```text
ordinary overload selection must produce one unique candidate
lifetime rules validate that completed result and cannot replace it
```

Ordinary overload resolution runs to a unique candidate on type, pattern, and
policy grounds alone. Lifetime policy then validates that candidate. It may
reject the program; it may not choose a different candidate, reopen
type/policy overload resolution, introduce a second selection stage, or
establish a specificity ordering that competes with ordinary overload order.

This restriction applies to lifetime *rules*, not to `@` itself. `@` is
resolved by ordinary overload resolution like any other operation, while its
selected builtin performs the name/continuation observation of §2.1. Thus "no
lifetime overload stage" and "`@` has overloads" are consistent statements
about two different things.

The apparent circularity dissolves once the three steps are separated. They are
strictly ordered, and each one is finished before the next begins:

```text
1. ordinary selection inside the operand   -> the operand value/name relation
2. ordinary selection of `@` itself        -> one candidate: the lifecycle
                                              observation visible in the
                                              operand's policy stage
3. lifetime validation                     -> accept or reject steps 1 and 2
```

Step 2 uses the ordinary selector, not a lifetime-specific one; the policy stage
only bounds which candidate is visible. Step 3 never reselects steps 1 or
2 — that is the whole content of this section. So "the lifetime stage runs after
ordinary selection has completed" and "`@` is itself resolved by ordinary
selection" describe steps 3 and 2 respectively, and neither feeds back into the
other.

## 5. General dependency handoff

[Dependency observation and realization](../symbol-world/dependency-observation-and-realization.md)
owns dependency requirements and their semantic realization. Explicit closure
captures are one source of those requirements:

```text
DependencyRequirement
  -> SemanticRealization {
       source_life_name,
       source_place?,
       requested_access_view,
       origin_or_region_relation,
       storage_or_link_category
     }
  -> LifetimeValidation
  -> RepresentationSelection
```

Snapshot versus live reference is decided by semantic realization, before
representation/layout. No capture or other dependency enters a completed
closure through an uncheckable side channel. Formation and each later action
pass their actual dependencies and obligations to the ordinary lifetime checks;
a type/call/Pattern projection does not repeat initialization or recapture.

## 6. Open representation questions

The following representation and integration questions remain open without
changing the relations defined above:

- concrete Rust/IR identity for `LifeName`, event positions, generation ids,
  Region slices, lazy origin links, and summary compression;
- concrete Color carrier, the contribution/registration API for the extensible
  global vocabulary, and any future source syntax;
- diagnostics and caching identity for lifetime validation;
- concrete representation of borrow and selected clone realizations, closure
  ABI and environment layout. Terminal Pass=Move and copy-derived
  share/rebind -> clone -> Move are closed, not default-pass selection questions. The entry origin defaults, exact
  move-origin/Region boundary, and selected share/rebind-plus-clone realization
  lifecycle-post boundary above are fixed lifetime semantics.

This revision defines none of the following: lifetime overloads as a
second selection step, lifetime specificity ordering, ABI equivalence classes
used for selection, or a lifetime-driven
ordinary resolver.

Related canonical contracts:

- [`../patterns-overload/overload-resolution-design.md`](../patterns-overload/overload-resolution-design.md)
- [`../symbol-world/policy-and-static-flow-projection.md`](../symbol-world/policy-and-static-flow-projection.md)
- [`../symbol-world/type-values-places-and-borrow-views.md`](../symbol-world/type-values-places-and-borrow-views.md)


## 7. Continuation transformations

Machine and lifecycle residue are projections of one continuation. After an
optimizer changes that continuation, old lifecycle facts can guide candidate
formation but cannot be copied to new positions. All affected projections
repeat their Pre/commit/Post validation. See
[evaluation and optimization](../static-evaluation/evaluation-residual-and-optimization.md).


## 8. Closure dependency lifetime refinement handoff

### 8.1 Handoff subject

```text
Eval(ClosureExpr_C) = tau_C
D = Dependencies(tau_C)
```

Closure formation fixes automatic/initialized sources and required capture
specifications. New invocation fulfills the latter in its actual context;
sealed calls retain their bound source. Lifetime implementation and refinement own:

- the regions/generations in which D may persist;
- Move with Kill/Preserve and ordinary clone-derived transport of tau_C and its members;
- return, storage, escape and promotion;
- persistence of bounded runtime state alongside stable descriptions;
- the concrete regions in which automatically formed dependencies and their
  ordinary closure results may move, persist or escape.

These refinements do not block the defined closure formation semantics.

### 8.2 Preserve checks without inventing conclusions

Formation and later actions retain the existing interfaces:

```text
FormationLegal
LifetimeLegal
Pre / Post
MoveEffect / Movable
EscapeLegal
owned transfer / promotion checks
```

Established rules continue to decide actual actions. Unrefined cases retain
their explicit scope; neither temporary global lifetime nor a blanket ban on
all future local dependencies substitutes for the missing refinement.

None of the following implications is established:

```text
ClosureTau => GlobalLifetime
LocalDependency => not Returnable
OrdinaryClosure => Escapable
InPlace => not Bindable / Returnable / Movable
```

In-place results are already ordinary first-class results. Explicit versus
automatic dependency formation supplies no independent operation prohibition.
TypeRole, ZST layout, Core equality, cache reuse and the absence of machine
storage prove none of these capabilities.

### 8.3 Established ordinary type survival

Existing global-survival rules for ordinary types remain in their established
domain. Universal closure-to-tau formation must not mechanically extend those
rules into a new theorem globalizing closure dependencies.

The lifetime topic must integrate persistence and transport of complete
closure-generated results with the general dependency model. Existing facts
remain valid; syntax categories alone establish neither new capabilities nor
new prohibitions.

### 8.4 Deferral is not an implementation shortcut

The handoff does not permit omitted checks, undefined behavior or a claim that
bounded closures are implemented. Acceptance verifies preservation and delivery
of dependencies and action obligations; it does not require uniformly accepting
or rejecting every closure with local references.

Path, ADL, Policy and closure formation retain defined semantics through these
interfaces without deciding every future lifetime refinement.

### 8.5 Per-name SCC collapse at continuation boundaries

A business type does not determine a lifecycle value. Structural
interpretation involving `$` provides the *formed* type/Pattern's paired
lifecycle structural material; actual formation establishes the **original
regular companion descriptor for each name item** (§2.1.2/LC1).
`$` itself adds no lifetime action. A name-item SCC is collapsed at its
continuation boundary **directly into that original regular representation**
under the name's current access, region, Color and source/effect constraints.
It must not be unfolded indefinitely and reconstructed as a new SCC grammar,
nor inferred solely from the business type.

#### Participation relation

For this judgment I denotes an actual closure activation/boundary in K, not
just a source closure expression, lexical owner, callable tag or cached
CompileInstance. Define the **name-item tracking graph** G_K from the established
semantic material needed by this lifecycle computation:

- A vertex is an already formed name item, retaining its LifeName, generation,
  actual structural item/selector coordinate and original paired regular
  descriptor. A lawfully transported summary can retain that same provenance
  and the ordinary mapping to a new carrier/generation. This records no new
  LifeName and does not assert that a recorded old generation is still alive.
  Any genuinely newly formed item/generation has its own original companion
  from that real producer; transport never replaces it with another item's
  basis. Only actual ordinary transport/delivery supplies such mappings, never
  a result name invented at an intermediate return boundary.
  Potential coordinates, type nodes and regular automaton states are not
  vertices merely because they can be written or visited.
- A directed edge is exactly a fine-tracking dependency supplied by actual
  formed structural/source material or an established selected operation/
  interface relation: its source item's material requires the destination
  item's material under that witnessed relation. Retain the legal direction,
  registered name-level structural/access witness, actual target/mapping and
  scope/control conditions. Mere Val2 membership, a type signature or a
  dependency requirement without its selected realization adds no edge.
  A Move contributes only dependencies actually supplied by its selected
  counterpart/contract. It does not automatically connect the old/new name
  generations, add an old-name origin link or reverse an edge.
- Each vertex records the activation of its actual formation action; each
  edge records the activation whose formation/selected action or instantiated
  interface first establishes that dependency. These are contribution
  provenance, not reparenting of names or Objects. Reading/copying an existing
  row or propagating an unchanged target completion retains its provenance;
  it does not make every transit frame the edge's establishing activation.
- Ordinary scope/access/capture and return-segment judgments must admit those
  edges. In particular no control-propagation edge crosses the nearest
  Ordinary return barrier or an unknown boundary by assumption. Only legally
  exposed material counts; hidden implementation provenance cannot restore
  erased external structural-source authority.

Preserve guards when constructing this graph. A component C carries one
compatible guarded case; mutually exclusive arm edges cannot be unioned into
a synthetic cycle. At an actual exit use its reached mapping. At formation,
the same rule transforms already formed bound descriptions conditionally; it
does not create future concrete vertices, activations or cuts, inspect a future
guard, or enumerate future paths.

For each such case, C is a maximal strongly connected component of G_K with
a witnessed positive-length directed cycle. A singleton without an admitted
self-edge supplies no additional InPlace collapse obligation. Let Edges(C)
contain only edges whose endpoints are in C. The participation rule is:

```text
LocalContribution_K(I,C) =
    {v in C        | formation_activation(v) = I}
  + {e in Edges(C) | establishing_activation(e) = I}

Participates(I,C,K) iff
    I is an InPlace boundary with established applicable scope/control evidence
    and C is the cyclic SCC in the compatible case just described
    and LocalContribution_K(I,C) is nonempty
```

The two sets are tagged vertex/edge contributions; `+` is metanotation here,
not a new source operator. In a cyclic SCC every contributing vertex/internal
edge lies on a cycle. A noncyclic feeder edge, merely containing/reaching C,
forwarding C's material or lying on a target-return route does not suffice.
Neither recursive business-type syntax nor repeated regular-state visits nor
reuse of the same closure tag in another activation establishes participation.
Graph storage, SCC algorithms and caches are open; this relation is not.
Missing required graph, contribution, guard or scope evidence is Unavailable,
not an assumption of nonparticipation or a type-based collapse.

**The collapse schedule is fixed by closure semantics:**

- At an **Ordinary (non-InPlace) closure** boundary, collapse outgoing SCC
  material per name item. At formed-body summary time, transform *bound
  descriptions*, not future concrete actions; at an actual exit, transform
  reached names/history.
- An **InPlace** closure preserves fine-grained tracking by default. Only
  when it **participates in the particular SCC** do its participating
  name items collapse at that InPlace closure's real exit (including a
  legitimately crossing targeted return). Other InPlace closures do not
  collapse unrelated SCCs. Every real exit still performs its usual
  local-name compression.

A targeted return crosses only InPlace boundaries in the reachable return
segment and cannot cross its nearest Ordinary closure barrier. SCCs induced
by these synchronous return interactions are thus segment-local. This does
not prohibit ordinary captured/returned name dependencies across a callable
interface; their outward contract and escape checks remain mandatory.

#### One component at successive exits

Participation is not permission to replace all of C at its first crossed cut.
At I's exit collapse **I's boundary-local cyclic contributions** per name item:
the items formed there and the local relation material established there that
must be exported. If such a row concerns an outer name, transform that local
projection, not every fine representation of the name in surviving contexts.
The fine material of an outer participating activation remains available,
updated with the actual committed effects, until that activation's own required
cut. Nonparticipating InPlace exits still compress their dying locals.

The outward transform must retain the original descriptor association, actual
name/generation/delivery mapping, guards, current restrictions and sufficient
component incidence/contribution provenance to process later exits. An earlier
local collapse cannot erase an outer contribution and make the outer activation
falsely nonparticipating. An implementation may contract graph representations;
it must preserve this semantic correspondence, without reading dead locals or
reconstructing their fine state from a regular type/state. A component identity
need not remain fixed across genuine semantic graph changes; a representation
change alone cannot discard its required lineage.

If mandatory local collapse cannot retain the information needed at the next
boundary, the transform is unavailable at that cut. It cannot delay collapse,
collapse the entire component early, or guess at the next boundary. The
[multi-InPlace example](lifetime-algebra-walkthroughs.md#scc-collapse-at-participating-closure-boundaries)
derives the participation and successive-cut behavior from concrete graph
provenance, rather than assuming a participating/nonparticipating label.

Collapse does not merge actual names/generations/targets, turn unknown into
None, remove earlier Color/effects, lengthen region, or restore cropped
operation endpoints. The original descriptor is a *structural basis* with
current restrictions, not renewed permission. Missing original companions
or unsupported regular operations yield Unavailable, not type-based guesses.
This rule does not prove termination of arbitrary recursive lifecycle
programs or machine tail-call/slot reuse; those remain separate obligations.



Three subjects have separate obligations: regular access/source SCCs (§11),
recursive ordinary lifecycle programs, and physical tail-call/slot reuse.
Regular descriptors preserve finite structure from formation. Their state
reuse does not merge actual names, generations or targets and proves no
arbitrary program termination. Ordinary recursion uses E and exposed contracts;
each supported family must justify its domain, operations, closure and
termination or explicit rejection/unavailability (§14). No external general
solver, universal fixed point or total convergence theorem is assumed.
Machine consumers use already established evidence; they cannot choose an
answer, retry an overload or switch ABI. Cleanup remains fixed before
observation with no cleanup/lifecycle repair loop.

## Current name position and delayed E

```text
Region_K(n)=R(NamePosition_K(n))
```

Region is the fixed normalized-K event-set interpretation at the current
semantic name position. Actual reachability restricts it; unknown future values
and physical runtime timing grant no pruning permission. E uses current Pre/commit/Post and fixed
cleanup observations, without future-path enumeration. A closed callable reify P2 boundary resumes its reverse-dependent work in the same E
at args-in. Origins remain relations inside compile name-position algebra.
Runtime strings create no authority to redefine fixed structured-name coordinates.
O may analyze future paths and specialize with affected E revalidation.

The lifecycle projection is compile name/Color algebra for both compile and
runtime callables. Future value facts such as a constant-false branch are not
visible to name/Color reasoning by themselves; an optimizing path removal uses
E O E with the affected projections revalidated.

## 9. Ordinary contract formation and bootstrap responsibilities

The ordinary expression in §2.1.6 runs with the existing compile invocation
rules. Its selected partner and canonical inputs establish a CompileInstance
I before body entry. It uses the ordinary compile self-name/projection and
actual OpenHere window to form complete structural material and install
associated check/admit behavior. A legal instance-open result is the single
direct self-root complete type tau_I, with ordinary NameExpr exposure. An open
type hidden inside a Product/reference/capture cannot escape. No explicit
user-created contract wrapper or two manually duplicated functions is required.

The behavioral interfaces are schematic full Function Patterns:

```text
check: world::lifetime:: -> bool::                 at compile horizon
admit: (world::lifetime::) ref -> unit::            at compile horizon
```

Each associated callable has its own contextual self. Any contract subject or
mapping is ordinary explicit input/dependency material, not an extra implicit
self slot. Source definitions may use ordinary open qualified member formation
and initialized associated entries; check/admit need neither a new registry
nor a special contract interpreter. Their concrete arity, packaging and
member spellings are engineering choices. The boundary requires the stated
observation and authorized update behaviors, with every ordinary Type, Pattern,
Policy, capability and lifecycle premise.

There are two independent authorities. Contract formation may modify associated
members only in its real construction window and at actual retained Places;
updating an outer value copy does not update instance storage. Calling admit
may update only the corresponding mapped lifecycle world at a checked
entry/success boundary. Open + OpenHere completes an omitted mode to mut but
supplies neither Writable nor that world-update authority. Ordinary admit
derives boundary consequences from established assumptions/results; underivable
external facts additionally use the unsafe owner's post-directed admission.
Neither can retract history or make a failed caller Pre legal retroactively.

The minimum bootstrap responsibilities are below. They are inputs/observations
of existing actions, not a requirement for ten VM instructions or new Objects.
Every seed needs its action's Pre, actual cut, common commit, joint Post and
failure-without-publication boundary before it can support library composition.

| Responsibility | Established input / output and consumer |
|---|---|
| Name and generation | Actual formation/observation supplies LifeName, generation and current K position, including temporaries without a Place |
| Origin | Selected formation/transfer supplies links, explicit termination or established unknown boundaries, plus legal field observations |
| Region and Color | Fixed event sets, endpoint order and pure slices, finite directed compatibility/exclusion/exchange rows and inherited historical constraints |
| Move and borrowing | Predetermined Kill/Preserve, frontier Movable, actual ref/share/weakening/rebind targets; no implicit borrow or copy fallback |
| Complete identity | Construction, consuming extraction and non-consuming Pattern observation produce their distinct companion material atomically where applicable |
| Invocation and exit | Selected Param/captures, explicit input, actual activation, delivery, target return and fixed cleanup correspondence |
| Pattern formation | Real structural formation establishes paired field/operation material and registrations for the commutation law |
| Regular access | Finite choice/advance/repetition/crop material retaining targets and operation permissions |
| External success | Compatible trusted facts tied to the successful external action and its mapped outputs |
| Boundary transform | Formed guarded relations/bound local positions for advance summary; actual exit/identity/origin/region/path/effect/world material for reached compression; fidelity between them |

Ordinary lang programs compose these materials into internal APIs, contracts,
access refinements, compression, automatic Post and library protocols. Source
consumers establish the correspondence and enter common transactions. The
materializer implements promised external behavior on its target. It cannot
decide lifecycle meaning from representation or layout. Missing seeds/consumers
are explicit unavailable boundaries, not successful empty summaries.

### 9.1 Bootstrap dependencies must terminate

The dependency here is justification of an action, not merely evaluation order.
Its leaves are the already fixed language semantics A0: ordinary Object/Pattern/
Policy and name/Place operations, selected call/activation/delivery, complete
formation, fixed cleanup, the defined Move/origin/Region/Color/access relations,
and their common Pre/commit/Post. Those laws do not obtain their validity from
a contract that a derived program is currently trying to form.

| Dependency | Required predecessor | Cannot serve as its own predecessor |
|---|---|---|
| Bootstrap observation/action | A0's actual name, target, cut, operation and ordinary authority | A newly constructed contract saying that the observation/action is valid |
| Current-instance type initialization | Actual fresh formation window and one-shot initialization authority at InvokeName(I) | Reading an uninitialized self resident, or returning an unrelated type and declaring it rooted at I |
| Derived ordinary lifecycle program | Available inputs and selected operations whose premises already have a foundation | Its own still-unformed check/admit, automatic Post or generated companion |
| Boundary adoption | Actual mapped entry/committed-success facts and permitted ordinary update operations | The Boolean returned by a user check, or a copy of a proposed world |
| Future source consumer | The same established relation and corresponding ordinary program | A fixture import, missing wiring, a speculative execution or successful empty summary |

A bootstrap implementation may initially realize a justified observation or
formation operation in the host; a later source definition may replace it when
its dependencies already terminate in this basis and it preserves the same
relation. This does not freeze an intrinsic count or representation. It also
does not permit an unrestricted host fact setter under a source function name.
The [host boundary](../static-evaluation/host-capabilities-and-machine-objects.md)
still requires justification for a permanent non-source primitive.

Derived programs' own ordinary computation has lifecycle premises too.
Projection-complete provenance stops another transcription/companion layer; it
does not waive those premises. Finite structural operations and the basis above
anchor that computation. An adopted recursive family must separately establish
its domain and termination/composition condition (§14). Recursive **execution**
under an established contract is different from circularly granting that
contract correctness. A strongly connected justification with no such foundation
is unavailable, even if every node promises to admit the next one's assumption.

In particular, forming C cannot use C.check, C.admit, AutoPost(C) or a companion
whose sole justification is C to establish that very formation's Pre. Omission
does not supply true, NoDelta or a default root at this gap. Ordinary scoped
recursion and E remain the only program evaluation; no proof Object, external
prover, privileged success flag or extra evaluator is introduced.

#### Operation provenance map

The following map classifies the walkthrough names by their **semantic source**,
not by which Rust function might initially implement them. B/L/X are justification
sources; W records a missing consumer and is orthogonal to them. One spelling may
therefore be ordinary composition over a basis observation with source wiring
still pending. None of these rows freezes an API or an intrinsic count.

| Class | Meaning / endpoint of justification |
|---|---|
| B: basis responsibility | The already fixed A0 observation/action laws and actual authority: names, source-established origin, actual cuts/regions, directed Color/access facts, complete formation, borrowing/Move and common publication. A basis fact is not produced merely by copying its carrier |
| L: ordinary lang composition | Pattern extraction/construction, calls, arithmetic/Boolean computation and supported relation transforms over those available facts. Its dependencies must terminate in B or actual compatible X facts |
| W: unconnected consumer | The source-to-semantic handoff for an existing relation/program is missing. W grants no fact and cannot be implemented by substituting another semantics |
| X: trusted external fact | A real facility supplies otherwise underivable resource/validation/exclusion facts by compatible unsafe Post after its ordinary Pre and actual success; failure supplies no success fact |

Namespaces below are the schematic ones used in the walkthroughs. Grouping
operations means they share a source, not that their bodies may be one opaque
host operation. The final column points to the existing pending handoff.

| Example operation(s) | Source / ordinary expansion and final dependency | Pending handoff |
|---|---|---|
| @, observe_origin | B supplies actual name/generation/origin observations; ordinary field/Pattern reads expose their existing link/unknown/termination material. observe_origin is not permission to invent that material | W: LC2/LC3 and ordinary name/reification/field consumers |
| split_link, split_row, split_outward_row, split_exit, select_one_parts, contract_rows | L: registered ordinary extraction of complete descriptive values. B is their actual formation/role/identity and extraction law, not source spelling or fixture import | W: LC1/LC4/LC5 |
| origin_accum, prepend_row, exit_material, guarded_exit_pattern | L: ordinary complete Products/registered constructors retaining bound roles, guards and all effect rows; B is complete construction and the input material's origin | W: LC1/LC5 |
| compose_link, retain_unknown, retain_termination | L: compose one available relation edge or retain its actual endpoint, using B's region order, inherited Color, access and source facts. Each adopted relation/algorithm needs its stated domain; no general solver is assumed | W: LC5/LC8 |
| map_outward_row, map_delivery | L: eliminate local reads and apply the established actual/bound delivery mapping. B is ordinary scope/return/result identity; no new target, validity or authority is synthesized | W: LC5 and ordinary delivery |
| compress_origin, compress_rows, compress_exit | L: the explicit recursive W1 programs over the preceding operations and justified finite/regular families. They are not irreducible lifecycle primitives | W: LC5/LC8 |
| source_frame, selected entries, function_boundary, true_boundary, false_boundary, middle_boundary, inner_boundary, available_guard | W exposes existing semantic body/control/call correspondence. B is the actual formation/activation/cut/selection and legally available control observation; Frame-shaped user data or unknown future control supplies none | W: LC2/LC3 and control/completion |
| share_fuel, is_zero, predecessor, move_leaf_result, recursive_input, return_exit, returned_call_exit in selected | The original selected ordinary operation and its established counterpart/contract determine these projections. L composes them; B supplies actual borrow/Move/formation/return laws and single transfer. Numerical helper meaning stays with its original ordinary callee, not a new lifetime builtin | W: LC2/LC3 and ordinary callee execution |
| transform_formed_arm and the advance summary | L: compose established operation descriptions, eliminate bound locals and retain guarded effects. B is actual semantic material formation and counterpart fidelity; an exposed modular contract can replace unavailable private structure | W: LC1/LC2/LC5/LC6, including adopted recursive-family evidence |
| initialize_empty_root::construction | B is general complete type formation at the already authorized current target and one-shot initialization, not a special lifecycle type factory. The example source facade exposes that existing action; its NameValue argument creates no permission | W: LC4 and current-instance initial formation |
| qualified let, explicit child ref, first write, captures, self return; form_post | L: the expanded W2 construction/call program. B is ordinary OpenHere/name/Place/initialization, closure dependency, self-root and return rules; it cannot justify its own initial Pre | W: LC4 and ordinary source construction/delivery |
| clone::lifetime | An actually selected ordinary share/clone/Move path over immutable descriptor data. B is its real producer Post and complete result formation; cloning data copies no semantic-world authority | W: LC3 and ordinary clone producers |
| source_matches, region_fits, color_compatible, access_is_subset, effects_match, all_true/both | L: mapped observation, supported finite relation queries and Boolean folding. B is the actual source/history/region/Color/access evidence; a true answer creates none of it | W: LC4/LC8 |
| adopt_origin, adopt_region, adopt_color, adopt_access, adopt_effects, end_boundary_view | L: map/extract the relevant rows, check their independent operation premises and thread/end the same scoped view. The B leaf is the narrow existing authority-checked projection update and joint-publication boundary, not an unrestricted fact setter or the whole adopt function. New external facts additionally require X | W: LC3/LC4; X enters only through LC7 |
| try_trust, partition_refs | L: ordinary observation, control, range planning and ownership/result assembly; B supplies actual input/source authority and ref formation. X is only the otherwise underivable successful validation/disjointness fact and its maintained interval, not the entire function | W: LC3/LC7/LC8 and materialization |
| range_access, validation/source/generation/shared/interval/read requirement rows, element_share_at and clone::word in W3 | L forms typed ordinary requirements, queries established evidence and composes a real indexed shared projection with an already selected independent Word clone. B supplies actual call/Move mapping, current names/generations, read/access/Color/interval and complete formation; the consumer creates no new validation fact. A runtime integer supplies targeting, not lifecycle proof by numeric equality | W: LC2–LC4/LC7/LC8 and actual indexed projection/clone producers |
| prepare_ranges, validate_separation, acquire_children, partition result/requirement rows and checked_ranges in W3 | L observes actual handles, preserves ranges/source mapping, forms contracts and branches on the real validator result. B supplies current original-ref authority and normal consumption/formation legality. X is only the validator's earlier compatible strict-range-separation success; neither a plan nor SeparationOk supplies root authority | W: LC2–LC5/LC7/LC8 and actual range validation |
| form_child_pair, child_pair_life, child_formation_parts, end_selected_share_intervals, begin_exclusive_children and finish_child_pair_material in W3 | L composes the actual selected complete producer's scoped companion. B supplies exact input/share-ending/fresh-child scopes, source/Region/actual access laws and one complete publication gate. Independent access additionally checks its required Color. X separation is already available before this producer's Pre. Components may stage only the prescribed ends/new formations; they cannot erase other dependencies, rewrite history or strengthen the share handles | W: LC2/LC3/LC7 and ordinary complete ref/result formation |
| success_id_role/outcome_roles, validation input/interval/from/shared dependency rows, required_contract_pattern, guarded_rows and failure_delivery_rows in W3 | L extracts formed interface roles and builds typed guarded descriptions for the ordinary contract constructor. B supplies the legal input read/Move, source, interval and actual delivery mapping; a result role is not an observed future payload | W: LC2–LC5/LC7 |
| probe_index, finish_probe and validated_index_rows in W3 | L organizes the ordinary probe result and ends its temporary view. X is the real successful validation tied to actual Q/generation/index/result and maintained interval; the row describes that fact rather than producing it. A tag or the outer contract's own check cannot justify the external guarantee | W: LC7 and the selected actual facility; unsafe post source notation is pending |
| probe_success_life, initialize_borrow_origin, initialize_bounded_region, begin_shared_dependency and finish_validated_name_material in W3 | L composes the selected fresh producer's companion. B supplies its actual one-shot name/source authority, bounded region and compatible shared dependency formation, including logical borrowing by ordinary non-ref values. It cannot rewrite an earlier name or grant access. X supplies only the actual validation packaged with those components | W: LC2/LC3/LC7; these are scoped formation responsibilities, not unrestricted world-field setters or a safeid builtin |
| at_root, registered .left/.right/.payload, descriptor choice/repetition operators, payload_access and access in W3 | L composes/crops the supported regular descriptor family. B supplies the original formed companion, selected legal field correspondence, actual target/name parameters and existing capability limits. A finite descriptor or retained route grants no new terminal permission | W: LC1–LC3/LC8 and ordinary operator/ADL registrations |
| with_ref_window, trusted alias-to-value conversion | L organizes callbacks, ordinary operations and cleanup; B supplies real normal ref/value formation under Pre. X maintains actual validity/exclusion/ownership that alias names or fresh anonymous origins cannot prove | W: LC7 and normal escape/borrow/delivery |
| owned_parts, part_snapshots, pair_fields, plan_repartition, same_resource/same_generation, amount queries/arithmetic, is_full, split_ready, merge_parts and result constructors in W4 | L: ordinary contract/field observations, arithmetic, consuming extraction, resource accounting and complete result formation. B supplies actual operand capabilities and ordinary lifecycle/construction laws. Quantity or an equal id alone mints no part | W: LC3/LC7 and ordinary library/type consumers |
| validate_for_plan and into_box in W4 | L checks available library state and organizes conversion; X is needed only for otherwise underivable actual resource-generation/validity/exclusion/ownership facts. B still requires every normal consuming/construction Pre; full quantity is insufficient | W: LC7 and actual trusted facility/normal Object formation |
| issue_full_part and initial resource acquisition in W4 | L transfers actual unallocated full-use authority into a complete part under B's normal action/ownership laws. Only a real external acquisition needs X for otherwise underivable ownership/validity facts. The amount 1, a Part constructor or a success tag is not an authority source | W: LC7 and ordinary resource acquisition/consuming formation |
| fresh_allocation_data, issue_full_part_core, split_core, merge_core, into_box_core and their private State/table/cell operations in W4 | L constructs fresh ordinary owned cells/data, computes amounts, consumes actual inputs, removes/inserts finite rows and rebuilds complete results around the same held box. B supplies actual ownership, cell/entry identity, legal access, normal construction/extraction/Move and common publication. Only lawful insertion in actual State assigns a portion; fresh data alone grants none. No alias-to-box reconstruction is needed when returning the extracted held box | W: LC3–LC5/LC7 and ordinary library/collection consumers; X only for genuinely missing resource validity in an external realization |
| inspect_relink, take_link, put_link, relink and Option/Link/root/session operations in W4 | L observes through registered root interfaces, computes the finite plan, consumes complete Options/Parts, constructs replacements, moves actual neighbor portions and composes the expanded accounting bodies. B requires actual distinct slot roots, resident-transfer and Part/State authority, ordinary access/cleanup/complete commits and mapped delivery. Neither root-write capability nor a plan supplies resident ownership or payload access. X is limited to genuinely external maintained session/transfer facts | W: LC3–LC5/LC7 and actual alias-root registrations/edit protocol; no whole-relink intrinsic |

An implementation must expose this decomposition even if a temporary bootstrap
realization uses host code. In particular compress_exit, form_post, adopt_origin
and partbox are not justified as permanent indivisible host black boxes. The
ordinary programs should consume the smallest justified observations/operations;
unconnected consumers stop unavailable. No row authorizes deriving a pending
contract from itself, publishing a supplied callback as proof, or replacing
ordinary bodies with source-string recognition. The [library composition example](lifetime-algebra-walkthroughs.md#ordinary-partbox-composition-program)
shows where L, B/W and X meet on a concrete path.

### 9.2 Safe adoption is narrower than mutable observation data

The world-ref interface of §9 denotes an actual authority-scoped boundary view,
not unrestricted setters for every rank-0 field in semantic history. The actual
invocation supplies its activation, input/result mapping, generation, cut and
entry/success scope. Ordinary safe operations on that view must independently
obey their own Pre, even when the surrounding contract's check returns true.
They can transport assumptions already justified at caller Pre, import an
established callee guarantee after successful completion, or derive consequences
through admitted operations. They cannot turn arbitrary computed data into a
new premise. Acquiring or copying a NameValue or a bool supplies no authority.

For example, an ordinary origin-adoption operation must check the proposed
source against the permitted mapped entry/success relation. Region adoption
must retain actual generation cuts and target validity; Color adoption cannot
retract inherited/history facts; access adoption must preserve the admitted
operation ranges and non-expansive cropping. These are the ordinary operations'
semantic premises, not another call to the possibly dishonest C.check. The
boundary checks the full correspondence and all affected Pre before joint
publication. A stored view reused for a foreign activation, cut, generation or
output has no authority there.

Implementations may express this restricted surface through ordinary access
views, operation families and capabilities. The carrier and function spelling
are open; an API exposing an unrestricted safe semantic-world record write is
incompatible with this rule. A function named admit and a well-formed self-root
type do not supply missing write permissions. A successful check establishes
its bool result only; a legitimate contract's useful guarantees come from its
actual sound computations and mapped operations, not from that name or bool.

The [worked contract and malicious variant](lifetime-algebra-walkthroughs.md#ordinary-self-root-contract-program)
make the non-derivability concrete. A safe always-true check can accompany an
admit with no unauthorized delta. Pairing it with a forged origin/region/Color/
access update fails the corresponding operation Pre or requiring consumer;
missing validation is unavailable. No successful permission is published.
Unsafe Post remains the separate route for compatible **new external success
facts**, after ordinary Pre; its truth is an external facility obligation.

### 9.3 Source construction and common publication handoff

The [ordinary contract program](lifetime-algebra-walkthroughs.md#ordinary-self-root-contract-program)
starts by requesting complete initial formation at its own existing invocation
root, then uses normal qualified member formation and first writes, then returns
compile self directly. The [construction owner](../symbol-world/structural-type-formation-and-composition.md#current-instance-initial-formation-handoff)
owns that initial formation; no local copy is substituted for the target.
OpenHere governs later member formation, each child has its own initialization
authority, and a helper has its own self/activation. Construction authority is
never stored as a general world-writing privilege in the resulting contract.

check/admit evaluation participates in the same E. Safe admission computes the
mapped update on the existing transaction's scoped projection storage; internal
component calls do not publish a partial outer boundary. All required checks
and component premises must succeed before its joint public publication, with
the original action/cut/mapping retained. An invalid later component publishes
no part of that boundary delta. This storage discipline is not rollback of
earlier independent actions or external effects. A common transaction callback
alone proves neither source authority nor correctness of supplied material.

### 9.4 Minimal authorization interface and operation rules

The provenance map identifies where justification terminates. The following
interface also fixes **what each authorized use may change**. Its rows are
semantic responsibilities of existing ordinary actions, not a permanent
intrinsic inventory. A source-defined facade may compose smaller operations;
host bootstrap code and later source code must obey the same premises.

| Illustrative facade | Actual authority and independent Pre | Only permitted result / lifecycle increment | Publication boundary |
|---|---|---|---|
| initialize_empty_root | Current InvokeName(I), independently authorized fresh formation window, declared complete-type target, live one-shot initialization authority and ordinary formation/lifecycle Pre | A complete empty structural type at that actual I, with its real roles/callspace; consume that target's initialization authority. No foreign root, read of an absent resident or world-writing privilege | Its own ordinary complete-formation commit, under the construction owner |
| initialize_borrow_origin | Selected fresh producer/name/generation, its still-pending source-formation responsibility, actual authorized source target/generation and ranges, and legal dependency-formation Pre | Exactly the origin relation prescribed for that new name by the selected source action. No rewriting an older name or inventing termination/unknown material | Stage in that producer's scoped companion storage; publish only with its complete action and joint Post |
| initialize_bounded_region | Same actual fresh producer/name/generation and authorized parent/child source, actual activation/guarded branch, fixed events and lawful current-K revision correspondence, plus strict source/child endpoint order and ordinary access Pre | Only that producer's actual bounded Region; a supplied event set or true endpoint query grants no proof, origin or authority and changes no cut | Same producer boundary; missing correspondence is Unavailable before publication |
| begin_shared_dependency | Same producer/name/source, actual bounded Region and protected endpoints, existing access/validation/validity premises; independently executing access additionally proves its required directed Color relation | Exactly the shared dependency on those endpoints with its actual validity obligations. Serial production adds no exclusionary Color; any independent-access Color retains inheritance/history. No new access and no recoloring the past | Same producer boundary |
| adopt_origin | Live entry/success-scoped boundary view with actual K/action/activation/cut/generation/result mapping; established permitted source relation and the origin operation's own Pre | Only its corresponding mapped origin consequence; transfer of the same scoped view, not another view with broader authority | Stage in that boundary's storage; all components publish jointly |

These authorities are actual semantic relations established by their ordinary
producers, not Objects which a user must construct as proofs. A NameValue,
FreshNameView-shaped value, declared mode, OpenHere, bool or transaction callback
does not establish them. The first row is owned by
[current-instance construction](../symbol-world/structural-type-formation-and-composition.md#current-instance-initial-formation-handoff);
the remaining rows refine the lifecycle responsibilities above. Type/member
construction authority never becomes lifecycle-world update authority.

The rules below are mathematical descriptions of ordinary E actions, not a
contract language or another evaluator. Fix K and an already selected ordinary
action A. n/g_n and s/g_s denote actual names with their generations; P denotes
actual protected endpoints, and I their fixed admitted interval. F abbreviates
the actual fresh producer's live, one-shot formation responsibility for n/g_n.
scratch_A is that action's **unpublished projection material**, not a partial
ordinary Object. The source origin o is fixed by the selected legal dependency
realization, up to observational equivalence; it is not an arbitrary parameter
chosen by a descriptor constructor.

```text
ActualFreshProducer_K(A, n/g_n), FormationScope(F) = (K,A,n/g_n)
F covers this pending origin component and has not been used for it
SelectedSource_K(A) = (s/g_s, P, o)
ActualSourceAccess_K(A,s/g_s,P), OrdinaryFormationPre_K(A)
I records the actual result event set and strict endpoint order to the source
ActualSharedAccessLegal_K(s/g_s, P, I)
IndependentAccess_K(A) => required directed Color premises
-----------------------------------------------------------------------
initialize_borrow_origin(F,s/g_s)
  stages exactly delta_origin = { origin(n/g_n) = o } in scratch_A
```

The selected borrow source is the actual referent/range, not a local handle
Place merely carrying it. The bounded-region and shared-dependency components
can stage only their table rows for the **same** A/n/source/I; together they
must satisfy the producer's ordinary joint gate. Origin initialization grants
neither those components nor a second use of F. Successful complete publication
discharges this formation responsibility. A previously published origin cannot
be reset through an old view; a descriptor-state match or equal numeric value
does not provide a fresh formation.

For adoption, b abbreviates the actual entry or successful-result boundary,
including K, selected action, activation, cut, names/generations, mapping theta
and the permitted component domain. w must denote its live scoped view. r is an
already justified entry relation or committed-success guarantee available at b.
The ordinary mapped computation may use supported extraction/composition and
lawful boundary compression, retaining actual unknown/termination and guarded
source correlation; no general implication oracle is assumed.

```text
ActualView_K(w) = b, b authorizes the origin component at this phase
r is established entry/success material at b, independently of C.check
OrdinaryMap_K(r,theta) = permitted_origin_row
proposed_origin_row = permitted_origin_row
all independent origin/source/generation/scope and joint-boundary Pre hold
-----------------------------------------------------------------------
adopt_origin(w,proposed_origin_row)
  stages exactly delta_origin = { permitted_origin_row } in scratch_b
  transfers the same scoped view w with the same boundary and domain
```

OrdinaryMap names the result of the actual supported ordinary computation,
not a new callable primitive. The operation cannot retarget a result, change
its generation/cut, delete Color or restore access. It can transport an external
fact already lawfully established by a previous trusted success, but cannot
introduce a new external axiom. Such a fact still needs LC7's actual unsafe Post
producer; ValidatedIndex or check=true supplies no formation/adoption authority.
For fixed actual inputs, selected action, mapping and requested row, the
permitted increment is unique up to observable equivalence. This does not fix
storage normalization or require different legitimate weak contracts to expose
identical summaries.

```text
any required premise false        => reject; published delta at this gate = empty
any required evidence unavailable => unavailable; published delta at this gate = empty
all required components/gates hold => one existing action/boundary publication,
                                     with its ordinary joint Post
```

If origin was staged but a later region/Color/access component fails, none of
that boundary delta is published. The failure rule is local to that gate:
W2's earlier complete root initialization and child first writes, or earlier
independent actions/external effects, are not rolled back. A normal returned
error/paused variant is also distinct from semantic action rejection: its
earlier real effects and current ownership need their own guarded Post, without
an invented successful credential or completion claim. Neither case permits a
selected failure to reopen overloads or retry with another authority source.
LC3/LC4 source consumers for these interfaces remain pending.

## 10. Internal actions, origin compression and automatic Post

Internal programs distinguish sibling transfer from child formation, real
borrowing from ordinary value formation, handles from targets, consuming from
observing, and Kill from Preserve. Origin alone grants no borrow capability;
ordinary values may carry logical dependencies. Internal Move does not simply
call a precompressed public from. There is no automatic reborrow to arrange a
desired origin tree. Pass=Move and selected clone-derived paths stay unchanged.

Automatic Post has two connected consumers at existing semantic boundaries.
These are ordinary computations in E, not additional evaluators, Policy ranks
or a requirement to execute the whole program before analyzing it:

| Boundary | Available input | Result / effect |
|---|---|---|
| Contract formation from a formed body | Semantic body Pattern/structure, bound input/result/local positions, guarded alternatives, fixed scope/return relations, and established selected-operation descriptions or exposed modular guarantees | A conditional outward Post summary; no future action executes or publishes facts |
| Actual continuation exit | The reached path, actual activation/names/generations/cuts, committed history and available lifecycle observations | Local compression and actual Post material, checked against the instantiated summary before successful outward adoption |

### 10.1 Contract-formation structural transformation

Ordinary semantic function-body formation supplies the first input. It preserves
the body's guarded structure and attaches the lifecycle structural descriptions
of operations already determined by the ordinary call/Pattern rules. Those
descriptions express the operation's admitted transfer relation; obtaining them
does not invoke the described action or its state-changing companion. A future
call can instead use an already exposed sufficient Function Pattern contract.
Neither route restores erased external structural-source authority or chooses
a future candidate. Raw/Normalized AST shape or a matching function name alone
supplies none of this semantic material.

These body/operation descriptions are complete ordinary material values formed
by their own actual actions; they are not readable partial callable Objects.
The final callable contract can still await their transformation. Their
formation premises must terminate in §9.1's basis independently of that pending
summary; a cycle through the summary itself remains unavailable.

Before the automatically summarized callable contract is exposed for use, an
ordinary lang program transforms that **formed guarded material**: compose the
admitted operation relations, remove outward dependence on bound local positions,
retain source/guard/effect correlations, and form the self-root summary type
with its ordinary check/admit behavior. Bound local/return positions and
parameterized region constraints are descriptions, not freshly committed future
LifeNames, generations or cuts. Ordinary call mapping later instantiates them.
The program may transform both alternative **descriptions** without performing
either alternative's moves, borrows, cleanup or external effects. An unknown
business guard remains a guard in the resulting conditional material.

This fixes the source of the summary input and its availability boundary; APIs,
representations and supported transformation algorithms remain open. It does
not promise that every body has a computable summary. If a required selected
description, modular guarantee, legal structural transformation or founded
recursive-family result is missing, summary formation is Unavailable. The
consumer cannot expose a supposedly complete automatic contract by guessing,
using NoDelta, waiting for one future path as its only justification, executing
future branches, or reopening selection. Only an actually established remaining
formation continuation may be transported as such.

### 10.2 Actual-exit transformation

Every actual block/call/other delivery boundary that exports lifecycle material
must remove its local-name dependencies while the internal material is still
available. The ordinary boundary program takes:

```text
actual exit + input/result/activation mapping + local/external partition
+ formed origin/region/Color/access/source-choice material
+ all observable selected actions and fixed cleanup
  -> available origin traversal and relation composition
  -> outward material without local-name reads
  -> self-root Post type and associated behavior, where successful
```

Traverse origin.origin to the first **actual unknown**; explicit None stops
as termination. Missing material is Unavailable. A known regular cycle uses
its finite regular representation rather than infinite unfolding, with the
applicable recurrence obligation of §14. It cannot manufacture unknown to hide
a cycle/failure. Contract users do not manually specify the traversal depth.

Compression eliminates local references from the outward material, not the
already committed history. It preserves required region order, Color, access
paths, external/unknown-root identities and correlations between source choice
and effects. All retained validity constraints still apply. Removing a local
carrier cannot lengthen an expiring referent, restore cropped access, revive
a dead generation or cancel Color. A true borrow of a dying local still fails
escape. Nested boundaries perform this work at their real exits rather than
waiting for the outermost function.
For SCC material, the placement rule of §8.5 applies. Nonparticipating
InPlace exits still compress dying locals, but do not collapse unrelated SCC
name items.

The actual-exit program produces Post_actual for the reached path and checks
its correspondence to the already formed Post_summary. A more precise actual
contract may be formed from that material, but it is not the missing source of
an advance automatic summary. Both transformations use ordinary Pattern/material
operations, not inference of arbitrary weakest preconditions from predicates.
Only defined choice/result combination relations apply; there is no universal
join. For select_one's admitted family, transform the formed descriptions of
each legal arm, then form correlated source choice:

```lang
lifetime pre  o@..live_with(r1@ || r2@)
lifetime post o@..from(r1@ || r2@)
```

`live_with` is a candidate spelling for depends_on. The Pre's o is the signature
result position, not a precreated Object. The Post's source component records
one selected origin, never two simultaneous exclusive targets. Any other branch
effects remain in the full Post. A from check may use an interval-dependency
operation, but its admit must also respect the actual source/transfer result;
the two roles are not undefined synonyms. The [worked example](lifetime-algebra-walkthroughs.md#2-select_one-and-boundary-origin-compression)
states conditions for compress-then-combine. No unconditional exchange law is
asserted between compression and choice.

Normal result delivery, actual unit fallthrough, target completion and failure
are distinct exits. A target return crossing InPlace boundaries transforms
only its real exiting material and history. It creates no local unit or result
binding there; the matching target alone performs ReturnPattern delivery.
An unfinished or failed action cannot produce a successful Post. Existing
InvocationResult transports the actual result, Diagnostic or established
remaining continuation. No missing implementation is called a residual.

### 10.3 Summary and actual-exit fidelity

Every supported summary family must preserve, under the ordinary actual call
mapping theta_p, all exposed observations on every legal successful path p:

```text
Post_actual(p, theta_p) implies Post_summary(theta_p)
```

This is the obligation on the ordinary transformations and exposed behaviors,
not a separate contract language, user-supplied proof Object or universal
implication decider. It includes origin/unknown-root identity, regions, Color,
access, all guarded effects and real target/delivery correspondence. Actual
exit validation uses the same mapping and selected actions; failure admits no
successful Post and never reopens. A summary formed from an unexecuted arm's
description does not license publishing that arm's history.

The [select_one witness](lifetime-algebra-walkthroughs.md#advance-summary-and-actual-exit-witness)
derives this relation for its two admitted ref-return arms without observing
the runtime choose value. The actual W1 program is the second consumer, not an
implementation of the first through available_guard. Recursive summary families
need their own founded structural/modular result (§9.1/§14), never a future-path
enumeration or evidence manufactured by executing recursion in advance.

## 11. Regular access material and capability boundaries

Actual Pattern/structure formation establishes lifecycle field companions
synchronously (§2.1.2). `$` remains only interpretation-polarity flip: an actual
formation/transform consumer has this duty, not every occurrence of `$`.
ADL and registered ordinary forwarding/Pattern extraction remain the only
field routes; Val2 presence alone grants no access edge or structural role.

Access/source structure retains regular form from formation.
Ordinary lifecycle path expressions additionally admit `[]` as an index/element
step alongside registered field advance `.`, choice `||` and repetition `~`.
This is a registered ordinary Pattern/operation expression, **not** a new
business-object axis or a read of an array resident by the compile lifecycle
world. For a compile-known index i, the precise path is `[i]`. For an index
unknown until runtime in an admitted actual array of length n, the conservative
access-path description is `[0:n]`. For an unbounded repetition family `T * ω`,
the description is `[0:ω]`, not an attempt to enumerate infinitely many
objects or to manufacture any given bound. Policy overloads may select the
known-index and unknown-index ordinary operations without special syntax
dispatch or a second execution model.

The lifecycle element path step `[i]` is distinct from the existing ordinary
Path-value operation `p[i]`, whose [Path owner](../symbol-world/structured-path-algebra-and-interpretation-polarity.md#26-segment-observation-and-indexing)
returns a rootless relative one-segment `path_pattern`. The common `[]`
spelling is contextual/selected ordinary operation material, not a coercion
between those two representations. It never recovers resident values,
authorizes an element borrow, or equates unlike name identities.

`[0:n]` and `[0:ω]` denote sets of *possible element paths*, not
permissions to operate on all of them or equal identities for their members.
Actual indexing needs a separately admitted index name/value, protected
resource/length/generation, valid access route, and its own Region/Color Pre.
The precise `[i]` route can be retained when the selected input/Policy
provides the needed compile observation. Source/path unknown does not
erase value-dependent bounds checks.

Anonymous insertion compresses external origin-line dependencies into the
aggregate's own lifecycle summary, schematically `array@.origin ||= out_lines`,
and preserves the corresponding strict Region comparisons for each live
dependent source. This is an ordinary lifecycle relation-expression
composition at the actual boundary, not unrestricted world mutation: only
real source-derived lineage and legal conservative unknowns qualify.
An extracted name inherits the aggregate's conservative origin summary,
while its actual selected extraction/Moves retain their own specific producer,
generation and responsibility. The remaining aggregate does not subtract
that summary when an element is extracted. Path-index material and origin
summary therefore cooperate without identifying different element names.

Each actual name item retains its formed original paired regular descriptor
as the target of boundary-local SCC collapse; the type value alone cannot
supply this lifecycle value, and earlier cropping is not undone. Ordinary
overloaded `||`, field advance/appropriate composition and `~` can represent
choice, concatenation and repetition. This is not a new regular-expression
DSL. A finite state descriptor denotes regular walks paired with actual
name/generation/target parameters and terminal operation permissions. Sharing
a descriptor state never identifies those parameters or constructs a runtime
owned-object cycle. Runtime range endpoints may themselves be ordinary
lifecycle names; numeric range arithmetic is not silently part of regular
path equivalence.

`access(args)` is cropping: its operable endpoints/permissions are a subset of
the input's. Route prefixes needed to reach a retained leaf remain routes,
not permission to operate on the whole ancestor. Cropping destroys no ordinary
field and retracts no historical Color. It cannot reconstruct forgotten
structure or restore a previously removed permission. Concrete selector
packaging and normal forms remain engineering work.

The [regular access program](lifetime-algebra-walkthroughs.md#a-regular-access-program-with-endpoint-only-cropping)
combines ordinary binary choice, selected field advance, postfix repetition and
access in a recursive family, then attempts ancestor restoration and exclusive
access through share. Its finite descriptor computation retains actual identity
parameters; it performs no future runtime traversal. The source registrations
and LC8 consumer remain pending.

| Capability | Admitted propagation | Excluded propagation |
|---|---|---|
| ref | Its actual exclusive strict tree of legal operations | No traversal through share or alias nodes |
| share | Shared observations; crossing a ref node continues only as share | No internal mutability, alias writes or strengthening |
| alias | Its explicitly registered root-write/default operations; initially no non-root normal access tree | No normal propagation through ref/share; no capability from type shape |

Lifetime alias here is the library alias-write capability, distinct from
[lexical Path aliases](../symbol-world/entity-alias-design.md). Normal return
from alias requires the trusted consume-to-value or bounded-window protocol
of §13. Distinct paths or anonymous origins alone never prove actual disjointness.

## 12. Dynamic credentials and batch acquisition

### N17 — 可信索引是名字事实，不是新业务类型 [不变量]

`try_trust` 是示例 API 名称，不是关键词、内建识别入口或不可更改命名。库可以使用 `validate_index`、`validate_range` 等名称。真正成功验证的可信外部事实可以在实际成功分支的兼容 `unsafe post` 中以 `index@ .. in(array@)` 这一**普通 compile 生命周期表达式材料**声明（具体 `in` API 的普通注册与前提仍由库决定）。访问函数的普通 Pre 可以消费同样的 `in` 表达式；语言核心不内建一套通用 Hoare 逻辑、全局偏序推理器或名为 `in` 的特权规则。成立性必须来自同一资源、索引名字/generation、实际区间及所有可使事实失效的操作得到的协议保证。

验证对象的业务值和类型可以保持为原 Index 类型；实际成功必须把凭证绑定到交付名字、真实聚合资源及其有效条件。数值相等、同类型、复制描述数据或返回 `true` 均不构成该绑定。

内建匿名聚合的安全区间/元素访问拒绝未经证明的索引。成功验证必须维护所有实际失效条件，例如资源/元素布局、长度、generation、索引内容等，而不只保护一个叫 capacity 的字段。已建立且仍有效的凭证可以支持后续访问不重复同一数值检查。

纯串行情况下，不把凭证维护重新说成普遍并发染色。改变被凭证保护的事实必须受该协议独立 Pre 约束；真正独立访问另按 N05/N06 建立 Color。

### N18 — 普通 K–V 查询不要求先验证 key [不变量]

一般 `lookup(map,key) -> Option(value)` 可以直接做普通查询并按成功 Post 建立结果来源。不得把内建匿名聚合的索引准入推广成所有容器、字典或查询接口必须两阶段验证。

同一库可封装验证及访问而对外提供一个普通查询；内部仍须兑现它使用的访问基础的 Pre。`make_view` 这类安全元素访问不能把裸整数直接当成可信名字。

### N19 — 凭证传递有借用与派生名字两条路径 [不变量]

对平凡 Index 的普通 copy，得到相同数值的新名字，其 origin 按该普通生产规则刷新为 None，验证凭证不随数值相等复制。

合法复用可以选择：

1. 借用可信索引，保留真实 origin 链并向访问 API 传递它；
2. 提供额外的普通派生 API，形成同值同业务类型的新子名字，并按借用式生产规则建立 child origin 与受限 Region。

第二种路径必须满足：

\[
origin(j)=i,\qquad
\min R_i<\min R_j,\qquad
\max R_j<\max R_i.
\]

**不得写成 `R_j ⊆ R_i`。** 它不是普通 Index copy 的隐含特殊化，也不能复制不允许重复发放的独占授权。凭证通过实际生产 Post 传递；选择、接口名和是否提供该便利 API 属于库设计自由。

通过可信索引借用形成视图，必须保留实际证明依赖，使 repair 能沿正确伴随表达式追踪；不能只保留整数值后再猜其来源。

The library's whole operation protocol must block or first end the credential
before every action invalidating its guarantee: length/capacity, layout,
generation, index or other relevant changes. Protecting a field called capacity
alone proves no completeness. With the protocol valid, range access consumes
the maintained credential without repeating the same runtime numeric check.
Without it access is unavailable/rejected, never an implicit dynamic-check
fallback or runner-up overload. Other Pre and machine costs remain. View
transport preserves actual dependencies and intervals, not a global credential.

The [safeid consumer](lifetime-algebra-walkthroughs.md#consuming-safeid-at-range-access)
exposes runtime index@ in range_access's ordinary compile Pre. Its typed rows
check the existing validation/source, actual input-generation mapping, required
actual access/validation dependencies, maintained validity and endpoint permissions. Runtime
targeting then uses the numeric index without another bounds comparison; the
illustrated selected Word clone returns an independent complete result.
There is no new external admission or permission from an equal integer.

A batch operation may consume shares with common provenance/current authority
in original ref R and acquire new exclusive **child refs from R** if their
actual operable ranges are disjoint in R's strict tree and all other Pre hold.
R need not be another explicit argument if its real authority is legally
retained. Shares are not strengthened/recolored into refs. It must preserve:

- actual source/target/generation and source-choice correlation;
- range disjointness, including ancestor-whole/descendant conflicts and route
  prefixes that need not be operable endpoints;
- other surviving shares, aliases, derived dependencies and logical credentials;
- legal end of old share intervals and start of exclusive regions without
  retracting their historical Color;
- single consumption and complete atomic publication at the existing action
  boundary, with no success permissions on failure.

No particular divergence depth or explicit path prefix length is required.
For each pair, sufficient evidence can establish that their **actual operation
ranges** lie in separated branches of the original strict ref tree and never
reconverge. Shared routing ancestors grant no whole-ancestor operation. In that
case disjointness follows even when the numerical depth of separation is
unknown. This is an actual tree/range fact, not an inference from unequal path
strings or reused regular descriptor states; an ancestor-whole range, later
meeting or unknown separation supplies no such evidence. Dynamic facilities
must establish the same relation, not replace it with address arithmetic.

Disjointness may be available statically or supplied by a small trusted dynamic
validator's successful Post. For one input no pairwise partition comparison
exists, so that additional runtime check is unnecessary; remaining Pre do not
disappear. Language semantics does not identify addresses with integers. The
materializer must faithfully and lightly realize the facility's actual abstract
range test; its representation/cost model belongs to machine work. See the
[unsafe owner](unsafe-semantic-admission.md#6-library-and-materializer-handoff).

The [two-share contract skeleton](lifetime-algebra-walkthroughs.md#a-two-share-partition-contract-and-ordinary-program)
separates preparation, trusted validation and actual child formation. The
validator's unsafe success fact is available before the child producer's
ordinary Pre; an outward Post cannot authorize an earlier missing Pre.
Current original-ref authority, legal share-ending and fresh formation scopes,
bounded Regions and actual access premises (Color only for independent access) and absence of surviving conflicts remain
independent premises. Its scoped companion stages the prescribed interval ends
and new children for one existing complete result gate. Dynamic failure returns
current input ownership without child permissions; a selected semantic failure
does not invent that normal error branch or reopen the call.

## 13. Alias windows and partbox library protocols

An alias begins with only its own root alias-write capability; the non-root
normal access tree is empty. Assignment, extraction and fields require actual
registered defaults. It does not traverse normal ref/share access. A trusted
facility can either consume it to an ordinary value under the real ownership
and validity premises, or form a finite-lived normal ref inside a checked
window. Window Pre must already establish exclusion and object validity.

Each admitted window ref has a freshly formed anonymous origin, distinct from
other admitted formations and **not the alias as origin**. Normal ref rules
apply inside its valid region. Freshness is not a proof of physical non-aliasing:
the facility maintains real exclusion, referent validity and restrictions on
conflicting alias operations throughout the window. Escape checks reject
return/store beyond it. Unsafe Post cannot fix a failed window Pre.

Partbox is a possible safe library over these facilities. Its resource-right
fractions are ordinary accounting material, not `T share` borrow instances.
Here a **part** is the library value carrying resource identity and an amount;
conservation and consumption are library protocol laws. Numerical amount alone
confers no ref/share access. Amounts are ordinary compile/runtime values, not
required business-type parameters. Split consumes one part and conserves its
amount across parts of the same actual resource; merge consumes same-resource
parts and preserves total amount. Equal amounts
for distinct resources cannot merge. Ordinary copying cannot multiply rights.
Full amount can be consumed to box only with every other ordinary Pre satisfied,
including outstanding windows, Color and object validity. Non-full amount
provides at most its restricted alias, not normal ref access.

### 13.1 Legitimate initial issuance and source preservation

Conservation starts with a real authority source, not the equation q=1.
Initial formation of a legitimate full part for resource Q/current generation
is permitted only from one of these sources:

1. An actual complete **currently unallocated full-use authority** for that
   resource, such as an admitted owned resource/box. Successful issuance consumes
   or transfers that authority into Part(Q,1) at the ordinary commit boundary;
   it leaves no second independently usable full owner or issuance power through
   an alias or saved view. All normal access/validity/window/formation Pre apply.
2. An explicit trusted resource-acquisition action that really obtains that
   full-use authority. Its compatible successful unsafe Post may establish the
   otherwise underivable external ownership/validity facts and issue the part
   at its declared boundary, after ordinary Pre. A failure or a fabricated
   success value issues nothing. It cannot describe an already allocated right
   as another independent acquisition merely by changing its carrier name.

"Unallocated" describes the **current authority disposition**, not a ban on
ever using Q again. A later legitimate full-part-to-box conversion can return
a complete full-use authority, which may be consumed in a new issuance under
its current generation/Pre. Cache reuse, type/value equality or an old initial
reference cannot reset a spent authority or erase its allocation history.

The library's legitimate-part relation is ordinary contract/state material
over actual actions and capabilities, not a new builtin resource kind or a
user-created proof Object. Every usable part must have a source chain:

| Producer | Required real source and successful result |
|---|---|
| Initial issue | One of the full-use sources above, transferred once to the actual new complete part |
| split | An actual legitimate consumed part; complete outputs retain Q/generation and divide that input's amount/right under the protocol |
| merge | Actual legitimate independently consumable input portions, each consumed once; preserve Q/generation and their summed amount/right |
| Ordinary transport | The selected Move/borrow/clone contract and its real source; copies or multiple views are not independent allocations. Preserve Move still creates no second consumable portion |
| Full-part conversion | An actual legitimate full part consumed under all other Pre, producing the complete normal resource/box with its own selected Post |

Arithmetic conservation and matching resource fields are necessary but cannot
replace these source premises. In particular ordinary Part-shaped construction,
equal-value copying, ValidationOk, a resource identifier and the number 1 cannot
mint a legitimate part. A clone is usable only with its actual permitted producer
Post; there is no ordinary clone that duplicates allocatable rights.

Even if a user can construct an ordinary value with the same apparent fields,
classifier or Core content as (Q,1), into_box must reject its missing actual
use authority, or report an unavailable required observation. Type mismatch
need not be the reason: equality is insufficient even when the classifier
matches. It cannot issue first and validate later, use unsafe Pre, or choose
another consumer after failure. A failed issuance Pre leaves the source
unconsumed; failures after an earlier committed acquisition/transfer retain
that action's explicit ownership/cleanup contract, not synthetic rollback.

The [W4 issuance witness](lifetime-algebra-walkthroughs.md#legal-start-of-the-partbox-protocol)
connects this source to the existing split/merge program. Its
[minimal production chain](lifetime-algebra-walkthroughs.md#a-minimal-production-chain-inside-the-library)
shows one ordinary serialized representation: consume the real box into private
State, track actual owned allocation cells, calculate and replace consumed
allocations, then extract that same box at lawful full conversion. Neither
fresh cells nor metadata alone assign a portion; lawful State updates and actual
input rights remain necessary. This layout is illustrative, not a builtin
resource type or a proof of arbitrary distributed/concurrent protocols.
Private interfaces, accounting/locking representation and API names remain library choices; the
source, single-allocation and no-forgery laws above do not.

Option and a specific topology protocol may move parts into/out of actual
link slots while keeping topology mutation separate from payload use. Consuming
option extraction and complete replacement use existing atomic Object rules;
there is no partial parent or implicit part duplication. Actual collection
must recover all necessary rights with legitimate slot access before box/payload
access. This asserts neither that two neighbors suffice for arbitrary graphs
nor that conservation proves all safety, concurrency or reclamation properties.
The [fourth walkthrough](lifetime-algebra-walkthroughs.md#4-alias-partbox-and-option)
gives explicit transfer and failure boundaries. Container implementation,
accounting representation, locking and reclamation remain library work.
Its [finite relink program](lifetime-algebra-walkthroughs.md#relinking-topology-without-accessing-payload)
makes the distinction concrete: incoming links carry Parts of their actual
target resources; moving N between two neighbor pairs transfers those existing
neighbor Parts and repartitions only Q_N. Registered root operations replace
complete Options without acquiring payload refs. Its normal paused result
owns the actual detached Parts/current roots and retains the edit protocol
within its valid interval; it promises no rollback or completed topology.
Every semantic action still has its own gate, including the
[minimal authorization rules](#94-minimal-authorization-interface-and-operation-rules).

## 14. Proof obligations and engineering freedom

The closed responsibilities above do not assert completed metatheorems.
Before an adopted family supplies evidence, its implementation must establish:

| Obligation | Required scope and failure exit |
|---|---|
| Projection fidelity | Preserve selected actions, Stage-erased name/control/dependency/activation/delivery correspondence and no double projection; missing partner is Unavailable |
| Field symmetry | Actual formed registered operations commute with their paired lifecycle observations; no fresh ADL selection to repair a missing partner |
| Boundary soundness | Compression retains external observations, unknown roots, region/Color/path/source/effect correlations and rejects expiring-target escape |
| Advance-summary fidelity | Formed semantic body/selected descriptions yield a conditional summary without future execution; each legal actual Post implies its mapped summary, including every effect row |
| Nested composition | Inner-then-outer compression preserves outer observations for the specified family; no unconditional fact deletion or commute with arbitrary joins |
| Contract behavior | check/admit represent the exposed guarantees under the four-way mapping, limited update authority and compatibility direction; unknown implication cannot accept |
| Regular composition | For adopted finite descriptors with identity parameters, choice/advance/repetition stay in the defined regular domain and cropping is non-expansive |
| Name-item SCC collapse | At Ordinary closure boundaries and participating InPlace exits, collapse to the original paired regular structure with current constraints; missing partners/operations remain Unavailable |
| Recursive calculation | State the ordinary program's semantic domain, combination operations, closure and termination argument, or explicit Unavailable/rejection. A well-founded finite traversal or a proved finite family can qualify; arbitrary recursion has no default result |
| Trusted facilities | Validator success matches external reality; credentials, partitions, windows and accounting cover all invalidating operations and surviving conflicts |
| Rewrite/materialization | O revalidates affected projections; physical realization preserves promises, fixed cleanup and E's ABIKind |

For regular descriptors, any adopted decision algorithm must terminate on its
finite descriptor/parameter observations and justify the particular relation
it decides. Finite representation alone does not decide arbitrary numeric
range predicates or contract-program termination. A recursive contract family
may use a justified recurrence/fixed-point method only after defining its order,
domain and convergence/rejection condition; none is mandated universally here.
The walkthroughs give conditional derivations and counterexamples, not executed
proofs of these general obligations. Missing wiring is an unavailable diagnostic
at the requiring boundary; only actual unfinished K supplies residual material.

API spelling (including depends_on/live_with), argument order, result wrappers,
namespaces, ordinary function decomposition, Object/regular encodings,
normalization/caching, boundary algorithms, diagnostics and dynamic layouts may
evolve. They cannot change failure publication, origins/choice identity,
consumption counts, Color interval/history rules, cropping, actual exclusion,
no-reopen, no implicit reborrow, or the single E. Bootstrap order and evidence
tracking live in the [roadmap](../../planning/roadmap.md#9-lifetime-value-program-wiring).

## 15. Current integration boundaries

These rules share the ordinary language and one lifecycle world:

| Relation | Current rule and minimal distinguishing case | Affected consumers |
|---|---|---|
| Contract computation | Complete ordinary compile expression; local declarations/branch/call can form the final self-root type | Clause E, associated formation, check/admit |
| Post defaults | Body omission forms automatic Post; handwritten Function Pattern omission remains NoDelta. select_one needs a sufficient interface | Body boundary, FnExtract/contract compatibility |
| Outward local elimination | Boundary traversal/compression preserves unknown and explicit termination; a borrow of a dying local still fails | Result/block/target exits |
| Lifecycle content | Lifecycle fields have ordinary content/identity without another business-Object coordinate; equal integers with different credentials remain distinct names | Value representation, names and trusted Post |
| Regular access | Per-name SCC collapse has fixed Ordinary/participating-InPlace boundaries and original companions; arbitrary program recursion and machine reuse remain separate | Companion formation, return boundaries, recursive/access consumers |
| Slice/handoff | Existing InPlace captures inherit preceding Post; actual Move[repair] exposes only a valid complete successor | LC9/LC10 and effect steps |
| Storage responsibility | Guaranteed pool submit preserves live Places/history and transfers support/duty/descriptors; stage migration is independent | LC11, host/library facilities |

U_default for new input names, omitted interface Pre, interface NoDelta, Pre_F => Pre_A and
Post_A => Post_F, atomic complete identities, Pass/MoveEffect, self/Param,
cleanup and same-cut order, return barriers and E-selected delivery remain.
This revision supplies no independent lifecycle checker, SMT engine, source
runner, container implementation or physical ABI. Existing carrier tests are
evidence only for their implemented transaction/value slices.


## 16. Anonymous names, executable operations and partial recovery

This section uses the recursively composed Pattern-local `lifetime default`
and the ordinary serial `lifetime pre` expression from `2.1.2. Name
forgetting is a reduction of actual retained lifecycle description material,
not an independent solver that collects constraints for later satisfaction.
An exterior unknown remains tagged with the precise traceable access-tree
frontier and carries its still-open source/Region/permission obligation outward.
It is not a fabricated `origin=None` or a fixed-depth universal root.

### N20 — 名字遗忘的保守准入 [不变量]

不可追踪的名字生成/遗忘是本代数的明确精度边界。对即将遗忘的名字：

- 已有依赖符合默认世界的，允许按默认关系隐藏；
- 非默认部分只有在能够保守化为更深 origin 链中的真实 unknown 边界时才允许隐藏；
- 无法这样表达而又不扩大权限的，拒绝该遗忘。

合法 unknown 化是一个有实际来源、由遗忘操作建立的观察边界；不是把缺失证据、失效借用、任意环或已死亡目标伪装成 unknown。不能借默认的独占根独立性掩盖实际共享/父子依赖。特别是 shareable atomic 的受保护视图在旧版本退休后仍可能继续存活，故退休资源的真实读者、Drop 责任不能被普通 Type 摘要或外部 unknown 的精度遗忘消除。

遗忘丢失观察精度，不消除已有历史，不增加 Region，不删除既有 Color，不恢复被裁剪的访问权限。

按默认关系形成新的取出名字时，深层保守依赖仍然存在；这与 N24 中恢复被遗忘具体实例的 compile 名字状态不同，不能从本条推出 reify 的可逆性。

### N21 — 匿名聚合按整体粒度承接依赖 [不变量]

对于以匿名元素聚合为语义的数组/重复结构，投放前必须能把元素相对完整 Pattern-local 默认表达式的全部特殊名字依赖，沿 `[i]`、`[0:n]` 或 `[0:ω]` 的实际有根据的路径汇总到聚合整体，示意为 `array@.origin ||= out_lines`，并同时保留对外 origin 的 Region 严格偏序；否则拒绝投放。这是生命周期表达式/实际边界规约，不是事后收集全局约束求解。

这些约束是义务的保守合并，不是权限的并集。一个数组可能需要同时受 A/B 资源有效性限制，但取出一个索引凭证不因此同时拥有针对 A 和 B 的具体授权。

普通取出生成的新名字沿其合法 `[]` 来源继承聚合 origin/Region 的全部保守依赖；剩余聚合不能仅凭“取出了一个元素”撤去整体依赖。取出实际所有权仍按选定动作交接，不重复复制资源。没有精确来源证据，就不能按数值、位置或相同类型恢复原 LifeName。

整体销毁或有独立充分证明的专门操作可以结束当前责任；“历史不可擦除”不等于一个已经合法结束的对象永远仍有活跃义务。

无外部名字依赖的普通值不因匿名化凭空取得外部约束。值身份、实际资源所有权与析构责任仍保留；不可 Copy 不自动等于不可进入数组。

Shareable atomic 的 `forgetable` 资源退休消费这套保守遗忘及
可执行析构义务：旧状态的唯一资源必须交给实际 lazy-free 管理者，
不得丢失值、Drop continuation、Region/Color、storage support 或读者
保护义务。真实的受保护 `T share` 仍是普通借用名字，持有自己的
origin/Region/不可变访问；遗忘旧状态的精确名字不意味着把这些
尚存读者变为死亡，也不能把旧来源洗成 None。对应的真正退休
CAS/并发读取义务由 [atomic owner](atomic-pattern-transitions-and-causality.md)
N46–N50 管理，单凭保守遗忘并不能证明合法读取。


### N22 — 遗忘必须保留可执行的 Drop 和 Move [不变量]

名字/类型观察精度下降时，不仅要保留保守依赖，还须保留后续操作的实现与合法性材料。`runtime:compile -> runtime:runtime` 的抽象可按 Pattern 默认结构保留可恢复的关系，并把剩余来源逐步规约到有根的外部 unknown；失败于本地外逃依赖的边界压缩不能伪造 unknown。`runtime:runtime -> runtime:compile` 的合法反向迁移只有在闭合 reify 及具体默认性验证成立时恢复相应模板关系，不恢复已遗忘的原名/generation。典型端点为：

```text
runtime:compile -> runtime:runtime
```

这是既有 Policy 链上的相应 rank 迁移，不是新增类型擦除本体。实际 Type 及其合法迁移关系仍遵守既有 owner。

至少必须保留与实际实例对应的析构子及移动子，或足以执行等价普通操作的材料。可以使用代码、闭包、运行时描述符等表示，不强制 vtable、固定字段、全局注册表或隐藏分配。

Move/Drop 的可执行性不等于已遗忘的精确名字关系可以恢复；保存了析构函数也不等于可消除它对外部名字的真实依赖。

### N23 — Pattern 默认关系、统一 repair 与实例 Move 证明 [定义]

基础始终是具体事件判断 `Movable_K(n,m)`。递归 Pattern 默认关系、
普通统一 repair 和事件级证明承担不同职责，不能当成三种可互换
的类型级 Movable 公理：

| 材料 | 含义 |
|---|---|
| `PatternDefault(P_tau)` | 结构 Pattern 节点的局部 lifetime default 完整声明，递归先子后父完整关系项覆盖；新形式名字按模板映射并在显式 Pre 串行表达式后检验，不证明 Movable |
| 普通统一 repair | 类型或具体续体可以提供适用于其已证明输入域的统一修复程序，但每次仍要检验实际源、伙伴、Pre、Region、Color、目的地与资源责任 |
| 当前名字的实例证明 | `Movable_K(n,m)` 对当前真实名字/动作的合法证明，以及该承载协议所需的实际 Move/Drop 可执行材料 |

即使某个受约束实例族可以**另外证明**其默认世界及普通
移动前提蕴涵某类无 repair 移动，这也只是带有明确假设与
动作域的定理；不能由 Pattern 默认声明本身推出。
不符合默认关系的实例并非类型错误：合法显式 Pre 可描述其
额外关系，只要实际调用映射和动作均满足该 Pre。缺少证据
不能当作独立性，不能凭声明覆盖真实 origin、历史或权限。

一次具体 m 的证明不承诺其他目的地、Region 或未来动作。
对目标承载协议的所有必需 Move/Drop 均须保留可执行操作与
充分准入证据；否则拒绝，不隐式 Box、clone、改 ABI 或要求
Copyable。

### N24 — 默认世界反向恢复是部分操作 [不变量]

在 reify 内，已依法选择的 `runtime:runtime -> runtime:compile` 恢复，只接受能够按其完整结构 Pattern 的递归 lifetime default 加基础外部来源默认，真实重建实例依赖的对象。已经遗忘并隐藏了非默认依赖的实例，不因持有正确 Type、移动子或析构子而取得该恢复能力。

```text
RecoverDefault(x):
  actual instance satisfies default-world recovery condition -> success
  actual instance deviates from that condition              -> ordinary throw
```

语言不承诺 compile/runtime 迁移互为逆函数，不承诺双射，也不承诺 `restore(forget(x))=x`。不得由 unknown 推出默认独立性，不重造旧 generation，不以 unsafe pre 追认。

tag、表、描述符或其他运行时判定方式均属实现。抽象侧只规定判定的真实性、成功 Post、拒绝分支及资源归属。具体实例拒绝是已声明的普通异常；未实现消费者、缺失语义定义或不合法选择本身不是伪造出来的 catchable 异常。

These judgments consume selected actions and mapped ordinary Pre/commit/Post.
The [reify owner](../unified-static-name-and-structural-semantics.md#20-reify-callable-p2-and-scoped-reverse-migration)
retains closed interfaces; the [implementation ledger](../../planning/lifetime-concurrency-consolidation.md)
separates definitions from proofs, source consumers and backend support.
