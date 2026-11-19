# Operator Patterns and Requested-Name Producers

Status: canonical semantics; source consumers remain pending.

## 1. One declaration, two surface projections

```lang
P let f = <T>[capture](self, x:T):runtime -> result:R
    lifetime pre E_pre lifetime post E_post => { B }

P let <T>[capture](self, x:T) f:runtime -> result:R
    lifetime pre E_pre lifetime post E_post => { B }

P let <T>[capture](self, x:T) f::path:runtime -> result:R
    lifetime pre E_pre lifetime post E_post => { B }
```

The first two are equal surface projections of the **same general Ordinary
callable declaration material**, not just of a `:compile` body with a bare
parameter Product. The schema covers every admitted DeduceList, capture
specification, contextual self/explicit parameter Pattern (including supported
Pack and defaults), callable-head P2 (`compile`, `runtime`, or `reify`),
result `->` clause, ordinary lifetime pre/post head clauses and implementation
tail (ordinary block, named strategy, default or delete), with their original
Policy, source order, binder identities, lexical scope and dependencies.
A particular form remains subject to the same independent grammatical
restrictions as its ordinary closure counterpart. No omitted component is
filled by reparsing the other form, and neither body nor capture initializer
executes twice. In particular, the head's contextual self is not an extra
explicit call argument.

    CallableDeclaration<Policy, NameDestination, GeneralClosureHead, ImplementationTail>
    ProjectCallableMaterial(Form_call_value)
      = ProjectCallableMaterial(Form_pattern_name)  // for the same destination

This is a semantic material correspondence, not a mandated new AST node.
Source normalization preserves the respective syntax without lookup, Pattern
solving or semantic declarations. Both use ordinary dependency formation and
selected CompilePartner identity. Requested-name observations enter canonical
inputs. The established callable retains its legally formed dependencies;
invocation uses only the actual self-name and admitted dependency material.

The third form extends the **name destination** to `f::path` in the existing
structural direction. Its terminal selector is `f`; `path` is interpreted by
the ordinary qualified-name/Path consumer. When the declaration forms a
qualified name it must satisfy the names owner's resolved structural-root,
current OpenHere, selector, freshness, ordinary Place/initialization and
publication premises. The name's destination and associated scope do not
change the GeneralClosureHead or its P2. The post-destination `:runtime` above
is callable-head P2, not the initializer-free `P let f::path:t` Place-type
annotation. The `=>` callable form supplies that syntactic distinction.

Qualified name formation does not by itself establish V_tau callability,
Pattern registration, a member on a different type snapshot, or a special
receiver. Concrete, wildcard and HoleRef requested heads keep the existing
requested-coordinate and specificity rules; this extension grants no
independent generator or naming authority.

Ordinary P let lhs = rhs has extractive polarity: the known RHS is matched by
R_Gamma(lhs,rhs,rho). Generative P let H => B has the direction:

    H -> requested NameCoord -> B

These declaration projections do not make a Function Pattern RHS a constructor
or a generative head. The [Pattern owner](pattern-relations-observation-and-extraction.md#111-directional-function-pattern-consumers)
defines LHS full-input contract extraction and RHS explicit-argument invocation
requests as distinct consumers, with no general inverse theorem.

Inside H, (self,args) is still an ordinary extraction head:

    R_Gamma((self,args), call_material, rho)
    concrete name f adds Selector(requested_coord)=f
    generative name _ adds no concrete selector constraint
    generative HoleRef(h) extracts Read_name(requested_name) into rho(h)

The requested coordinate must already be legally formed. Extractive _ remains
a wildcard binding no named value. Concrete f is more specific than _ under
ordinary Pattern specificity, after applicability. There is no generator or
fallback-name priority. Multiple incomparable maxima remain ambiguous; selected
failure never reopens another generator.

### 1.1 General heads and expression bodies

```text
NameHead = Concrete(s) | Wildcard | HoleRef(h)
let _ => E
let <a> a => E
```

An explicit callable extraction head may be omitted. Omission adds no wildcard
actual or empty Product and removes no implicit self from a real invocation.
E may be a general expression; a surrounding ordinary body uses the same direct
result delivery without an extra Policy-defaulting temp. Requested-name
extraction supplies the full NameValue, including name::path structure, not
only selector s or the binder spelling a. Selector(requested_coord)=s remains
the separate observation for concrete-head constraints and specificity.
This request material enters the admitted invocation inputs In and their
dependency closure; it is no hidden capture. Concrete heads beat unconstrained
heads only by ordinary specificity.

General expression bodies retain the required `=>`: `P let H { B }` is not a
requested-name producer form. All dependencies obey ordinary formation and
legality; omitted extraction heads create no implicit actual or authority.

```text
let <a> (self, object:t, ...args) a = expression
let <a> (self, object:t, ...args) a => { body }
let a = expression
let _ = expression
```

These share head material, not polarity: = extracts from a known RHS; => forms
a result under a legal request. Intermediate `let <a> (c Pattern) a` applies
the [same R_Gamma again at the reached layer](pattern-relations-observation-and-extraction.md#82-intermediate-layer-extraction):
one whole unpositioned extraction on an unordered layer, multiple aligned
extractions on an ordered layer. Name observation, layer and payload remain
distinct. Pack restrictions remain intact.

A requested-name producer accepts any ordinary declared result: scalar,
Product, callable/closure, existing type or NameExpr. NameProducingAction is
independent of NameExprResult. The instance-open self-root reachability check
belongs to ordinary result delivery and applies equally to all producers.

## 2. Grammar facts and ordinary operator dispatch

Grammar fixes OpTok spelling, fixity, precedence and parse associativity.
Contextual elaboration distinguishes RHS value expressions from LHS/Pattern
interpretation. $ flips interpretation polarity: Interpret(e$,C)=Interpret(e,Flip(C)).
Bare LHS names retain navigation inheritance. Semantic dispatch preserves:

```text
RHS naked OperatorUse(op) -> operator[op]
LHS naked operator      -> (operator[op])$
dot operator .op     -> op::adl
explicit path        -> the written ordinary path
```

Paths such as op::type remain ordinary library paths, not the grammar's
hardwired target. Source cannot create tokens or change parsing by compile
evaluation. OperatorUse and OperatorNameValue are distinct roles: the op
argument of operator[op] reads the ordinary operator name and does not
recursively invoke operator[op]. Ordinary lookup, type, policy and lifetime
checks still apply; this rule is not a raw token bypass.

### Qualified binary operator targets

In an operator expression, a binary-capable operator after its left
operand may start the target with a written Path suffix. In that
infix position, `a op::Path b` parses as a binary operation. The
ordinary Path production consumes its components and the expression
parser then consumes one right operand at the operator's usual
precedence. The initial operator token determines both precedence
and associativity, independent of the qualifier. Required operands
use ordinary operator recovery. No selection or lookup decides this
syntax.

```lang
source ->::E target
(source, target) |> (->::E)

let arrow = ->::E;
source (->::E) target
```

The first two expressions deliver the same ordered two-entry Product
to the same explicit Path target. Raw preserves the two operands and
the written operator-rooted Path. Norm uses ordinary binary operator
lowering with that Path as the call target. At the start of an
operand, `->::E` is an ordinary Path value expression. A grouped Path
within a sequence remains ordinary Product/expression call material,
as in the final example. Its parentheses do not become transparent
to the infix recognizer. A bare arrow in a callable head retains its
existing syntactic priority.

### 2.1 OperatorOverloadGroup and current-slot selection

```text
OG_s = s |> OperatorOverloadGroup
```

This ordinary string-to-type compile family accepts ASCII spellings in the
grammar's valid operator vocabulary. OG_s retains extractable spelling s.
A formal a:b OperatorOverloadGroup uses an explicit HoleBinder for b; it
extracts spelling independently of the ordinary value binder a.

The ordinary explicit Forget_s operation projects OG_s to OverloadGroup.
It establishes no subtype, implicit conversion, overload preference or
automatic adaptation. A plain OverloadGroup carries no recoverable spelling;
there is no inverse inference of s.

operator[a:OG_s] selects the current environment's ordinary slot named s.
It does not simply return the candidate contents carried by a. Thus a carried
selector can choose the current binding after ordinary shadowing.

Successful selection preserves the spelling-indexed result type:

```text
a : OG_s
operator[a] -> g_s : OG_s
```

For example, subject to ordinary lookup, policy and lifetime checks:

```lang
let selected = operator[+];
operator[selected]
```

The second selection still extracts "+" from selected's OG_"+" type and
reads the current environment's "+" slot. Selection does not erase its result
to plain OverloadGroup. Only explicit Forget_s performs that projection;
spelling cannot subsequently be recovered from the resulting plain group.

Direct declarations such as `let + = closure` and
`let + = operator[+]` bind the ordinary operator name subject to the same
rules. Their intended source consumer is pending. They are not a general
left-hand-side-free compound assignment `let += g`; compound assignment
and operator-name binding remain distinct syntax roles.

Same-slot ordinary combination retains OG_s and its spelling. It does not
implicitly combine different spellings, infer a selector from an arbitrary
group, or recover semantic coordinates from String. The [Path owner](../symbol-world/structured-path-algebra-and-interpretation-polarity.md)
separately permits single-name structural construction and defines external Read.

## 3. Three projections of one application structure

Call(...), RelationalExtract(...) and GenerativeInvocation(...) below are
semantic metanotation, not callee-first source syntax.

    ordinary RHS:       Call(op,a,b)
    Pattern Pa op Pb:   RelationalExtract(op,Pa,Pb)
    generative head:    GenerativeInvocation(op,Pa,Pb)

For a Pattern-registered operator candidate F_op, let its ordinary relation be
Rel_F(self,x,y,z). Calling observes (self,x,y)->z. Extraction observes the same
relation in the known-result position:

    Rel_F(self,x,y,z) with its derivation
    R_Gamma(Px,x,rho_x), R_Gamma(Py,y,rho_y)
    rho = compatible_join(rho_x,rho_y)
    -------------------------------------
    R_Gamma(Px op Py,z,rho)

This is proof-relevant relational observation, not a synthesized inverse
function. There may be zero, one or many solutions. Callable(op) does not
imply PatternExtractable(op): appropriate Pattern relational registration is
required for the selected candidate. Ordinary Val2 presence or V_tau
registration alone provides no extraction proof.

```lang
P let x * y => B
```

Here * follows the ordinary operator[*] dispatch relation. The head observes
its generative invocation/name relation, not GenerateToken("*"). No fourth
operator projection or separate parameter system is introduced.

## 4. Generated occurrences and registration

A generated occurrence may realize an ordinary Val2 resident under the name
realization and invocation laws. That occurrence supplies no V_tau registration
and no Pattern structural registration: no DirectPatternChild, ConstructEdge,
ExtractEdge or FieldView evidence follows from it.

The two exclusions have distinct reasons. V_tau callable values do not acquire
val::path navigation through callability registration; their anonymous
classifiers' /tau homes do not change that fact. Pattern registrations determine
the type's structured construction/extraction form and therefore must be
non-generative. Neither role is a history-dependent discovery of requested values.

Close freezes those non-generative registrations and ends their construction
window; it does not prohibit later ordinary generated Val2 realization. The
[name owner](../symbol-world/names-and-overload-groups.md#71-generated-val2-after-registered-structure-is-closed)
defines current observations, retained snapshots and the unchanged no-reopen
boundary. Frozen generative matching directly realizes an ordinary occurrence;
it is not explicit NameExpr formation followed by ref acquisition and write.
It implies no OpenHere, mut type ref or open type ref, and cannot mutate a
registered witness. No separate closed-generative authority is introduced.

This restriction belongs to the occurrence, not permanently to the value. The
same ordinary value may obtain an appropriate witness through another lawful,
non-generative declaration. Neither callspace registration nor Pattern role
registration may depend on which generated names were queried later.

The [name owner](../symbol-world/names-and-overload-groups.md) separates name
coordinates, realization, Places and residents; the [compile owner](../static-evaluation/compile-instance-invocation-and-result-delivery.md)
owns invocation identity and result formation. Generative notation grants no
extra construction, write or lifetime authority.

## 5. Laws are ordinary compile results, with separate consumers

Expressions such as op |> associative and op |> commutative are ordinary compile
invocations. Trait-like and auto-trait-like patterns use ordinary generative
rules and overload specificity; there is no TraitObject, TraitAxis or automatic
trait ontology. A fixed compiler consumer does not change the result's ontology.

    ParseAssociativity(op)       -- grammar/AST grouping
    SemanticLaw_E(op,L)          -- evaluator meaning
    RewriteLaw_O(op,L)           -- proof supporting an equivalent rewrite

These are distinct judgments. If Pattern semantics requires commutativity, it
must already be present in the E interpretation of that operator relation.
Optimizer queries cannot decide later whether a Pattern was unordered.
O may rewrite only after Facts_E proves equivalence, with affected projections
revalidated under the [ordinary E/O boundary](../static-evaluation/evaluation-residual-and-optimization.md).

Policy + and ordinary Pattern deduction consume these same registered relations,
HoleBinderId and require constraints. The [policy owner](../symbol-world/policy-and-static-flow-projection.md)
owns coordinate legality, omission/inheritance and the joint invocation relation.

The ordinary law query's default state follows the compile owner's retained
instance/member Place protocol. Customization requires current OpenHere and
Writable; consumers observe the current committed payload, not a copied
outer binding or an optimizer-private fact. Later writes do not change a
previous committed semantic decision.


## 6. Ordinary dot-name generation and forwarding

### 6.1 Generative overload family

```text
adl/
    let <field> (self, object:t, ...args) field
        => ((object, args) |> ((field#)[0])$::t);
    let <field> (self, t:type::) field
        => ((field#)[0])$::t;
```

The first candidate performs an ordinary receiver call. The second progresses
a type-valued receiver's NameExpr path. Both receive the full requested
NameValue; # projects structure, [0] selects the relative single-name segment,
and $ flips polarity under the Path consumer. Ordinary applicability and
specificity select; dot syntax provides no priority or receiver adaptation.

```text
T.field =_Path field::T        -- selected type-Path candidate
foo.bar.baz =_Path baz::bar::foo -- each step selects that ordinary Path family
```

### 6.2 Canonical lowering

```text
.field  -> field::adl
E.field -> E |> .field -> E |> field::adl
```

The normalizer preserves the dot/name/path source role; it does not own the
semantic authority to generate the actual forwarding implementation.

OperatorUse, OperatorNameValue, dot selectors and explicit paths retain their
distinct entrances. OG_s preserves spelling through selector results; only
explicit Forget removes it. Path support does not erase OG_s to ordinary OG.

### 6.3 Requests do not mutate the forwarded type

field::adl forms a permitted ordinary result/member occurrence. Its body reads
field::t; it does not inject field into t. Frozen generative rules can answer
later legal requests without reopening adl or t, modifying Pattern registration
or V_tau, or pre-enumerating names:

```text
finite generative rule -> potentially unbounded legal Name family
not: Val2(adl) pre-materializes infinitely many names
```

Each request still obeys ordinary name, input, selection and lifetime rules.
A current namespace observation remains finite; frozen generative rules can
answer later requests without revising a retained snapshot.

A generated ordinary resident retains its own result identity. Name production
and computed NameExpr results obey independent ordinary relations. Instance
formation precedes the body, independent of result class or horizon.

### 6.4 Transparent Policy and self

The established P1/P2 of the outer ADL forwarder jointly constrain the inner
field call. The terminal expression receives the return demand directly, with
no additional temporary. Ordinary forwarding therefore need not enumerate
every mode/stage combination or add an explicit P let merely to repair return
demand.

The actual field callable has its own self; object remains an explicit
argument. Forwarding does not substitute object for ActualSelf, implicitly form ref/share,
or add coercions when a candidate is absent.

### 6.5 Ordinary member calls do not redefine structural extraction

Real Pattern structure still requires DirectPatternChild, FieldView and
ExtractEdge registration. Atomic extraction retains its established family
filters, including StructuralDefault.

Custom field::adl behavior can change an ordinary field call without changing
the host Pattern's construction/extraction relation:

```text
OrdinaryADLCall != RegisteredStructuralExtraction
```

A selected type-path candidate produces a NameExpr; a receiver candidate
delivers its ordinary result. Dot syntax alone supplies no structural role
or private Place projection.

## 7. E-relative transforms and snapshot registration

Outside a strong built-in callable arrow position, bare `lhs -> rhs`
is a binary, non-associative operator at the weakest precedence tier.
It lowers to `(lhs, rhs) |> operator[->]` through the ordinary operator
dispatch. In a callable head with an applicable built-in arrow, the return
clause uses that built-in syntax. An explicit `(->::E)` names the operator
through a Path and `.->` uses ordinary dot-name dispatch. Grammar decides
which form is present before resolving any candidate.
The built-in input/output relation also uses the ordinary operator algebra
in its own context. A qualified Path and a dot-name selector retain their
ordinary call binding. The direct `source ->::E target` call uses the qualified binary rule.

For a complete E formed by ordinary Pattern construction, ordinary
registered members can construct descriptions and a callable transform:

```lang
let transform =
    ((val1 n1::A::E, _) A::E)
    ->::E
    ((newSource)(val3 B::E));
let keepSource =
    ((val1 n1::A::E, val2 n2::A::E) A::E)
    ->::E
    ((_)(val3 B::E));
```

The selected ordinary constructors (including `A::E`, `B::E` and
`->::E`) determine the actual represented positions and the meaning of
available material at those positions. RHS `_` is ordinary universerank
material. Where a constructor contract admits it, it can denote an omitted
source comparison or inherited target material. Inheritance requires a
compatible actual slot and evidence for the retained material. A missing
slot cannot supply a value, resource or permission. Ordinary Pattern-side
`_` retains its independent anonymous meaning.

The transform retains correspondence to E, its selected producers and
actual source/target positions. Formation uses ordinary selection,
dependencies and lifecycle, and confers no access to a held subject.

### Ordinary borrowed sum transforms

With an owned sum `S`, ordinary `S − M + O` consumes the actual
`M` and constructs `O` under the selected ownership and delivery
contracts. A borrow of `S` is a capability to access its resident.
`S ref − M ref + O ref` supplies neither ownership of the old payload
nor authority to install the new arm. Even a real child projection
yields only the related borrow. A conditional change through an
exclusive borrow needs one operation that describes the expected
source and complete destination, along with the disposition of
the actual resource. An ordinary transform formed by `->::E`
supplies that description.

Write `t = Transform_E(M,O)` for the callable value formed by the
selected ordinary producer from source description `M` and target
description `O`. In the rule below `M` denotes its formed source
Pattern. Formation does
not observe or modify the borrowed resident or acquire its payload.

The ordinary borrowed consumer uses the selected transform on the actual
exclusive view of a resident of `E`. Its attempt Pre admits the
observation of the actual source through that live view. A match
uses the ordinary Pattern relation. Before a matched change can
commit, the selected operation also checks the target correspondence
and applicable replacement, Writable, ownership, Region, Color, and
cleanup premises. A change of resident requires a compatible complete
replacement of the same declared type. Taking a payload requires
actual transfer of the resident's owned material through the selected
operation. Exclusive access and Writable alone do not supply that
transfer. A shared borrow does not authorize replacement.

The successful action is an ordinary conditional commit. For actual
continuation `K`, occupied Place `q` and view `u`, the premises are:

```text
formed t = Transform_E(M,O) and the ordinary attempt through u is admitted
Exclusive_K(u,q) and Resident_K(q) = v
R_Gamma(M, Content(v), rho)
Pre_t(K,q,u,rho,O) proves complete destination, lawful disposition and result
------------------------------------------------------------------------
<K,q,u,t> --commit--> <K',q,u',Success(r)> with Post_t
```

These are observations and actions of the same E. `u'` stands for any
authority the ordinary Post actually retains or delivers. The rule
replaces or retains the resident of `q` as selected and grants no
implicit revival of a view whose target generation ended.

Once admitted, a decided source mismatch follows the selected normal
failure branch with the resident unchanged by this attempt. The branch
accounts for any formed candidate materials passed to the operation
and their remaining duties. Missing required observation or authority
is unavailable, and a proved-false operation Pre rejects before
commit. A failed selected operation does not reopen or retry.
On success the ordinary action commits the complete destination,
resource disposition and result together through the existing
Pre/commit/Post. Semantically there is no intermediate empty Place
or partial Object. Existing views retain their ordinary generation
and access restrictions.

| Ordinary successful resource disposition | Resource delivered to the caller | Post requirement |
|---|---|---|
| Take the actual old payload, by removing it or replacing it with a new payload | Owned old payload | Transfer its ownership and associated duties once. The destination retains the newly installed material, if any. |
| Keep the actual payload in the resident through the selected correspondence | Shared borrow of that payload | Preserve the actual resource identity and custody. Record the borrow's source and target generation. |
| The source has no resource at the selected position | No old-resource material | Account for any new material under the ordinary target contract. |

Take and Keep follow from the actual material correspondence. The
ordinary consumer checks the [lifecycle rules for Take and Keep](../lifetime/lifetime-policy-and-overload-boundary.md#borrowed-sum-transform-lifecycle).
The Keep result's Pre must establish the borrow window through the
exclusive view and actual storage support for that window. Its Post
records the actual origin dependencies and constrains subsequent
access while the share is live. If the containing generation changes,
the share needs a lawful projection of the supported resulting
resident. A failed share Pre prevents the complete change.

An ordinary Option uses an explicitly grouped payload alternative.
For a live exclusive view `r` of `Opt`, and with the named source and
target descriptions already formed by ordinary producers, a take
can be described as:

```lang
let Opt = (((T Some) | None) Opt)$;
let take = someSource ->::Opt noneTarget;
let outcome = r take ?;
```

| Actual source of an admitted attempt | Selected complete destination or failure | Delivered resource |
|---|---|---|
| Some(x), Take | None | Owned x |
| Some(x), Take by replacement | Some(y) | Owned old x, with y installed |
| Some(x), Keep | Some(x), with any other lawful update | Share of the actual x |
| None | Some(y) | No old resource |
| Decided source mismatch | Failure, no destination committed | Uncommitted candidate material and duties returned |

A realization can exchange complete residents and extract from the
owned old `Some` via its ordinary consuming correspondence. The caller
can therefore express safe take or replace through the ordinary
transform under its actual Move, access and storage Pre. The example
and table specify consumer behavior. They introduce no take syntax.

### Atomic consumes the same transform

The [Atomic owner](../lifetime/atomic-pattern-transitions-and-causality.md)
consumes the same E-relative transform descriptions for the material
it admits. Its consumer adds a single externally atomic conditional
commit and the associated causal evidence. Its registered access
can admit controlled overlap under N36. The read/write resource
protocol determines which admitted updates are legal and which rights
success delivers, including N47's read Exit share and write Keep with
no old-resource access. Ordinary transforms use the ordinary result
table above and carry the usual exclusivity, with no Atomic protocol
or ordering premise. Both consumers use ordinary Pattern matching
and complete delivery. The ordinary commit rule alone supplies no
cross-execution atomicity or memory-order guarantee. In particular,
the ordinary commit does not establish an atomic modification order
or rf/Past publication facts. Atomic comparison failures and any
declared lawful spurious failure follow the Atomic consumer's contract.

### Snapshot member on the Atomic share type

The [Atomic snapshot contract](../lifetime/atomic-pattern-transitions-and-causality.md#n42--共享视图上的原子快照-定义及不变量)
applies when ordinary call selection reaches the `?` member that Atomic
registers on the associated `E share` type for admitted Pattern material
`E`. The receiver must be an actual admitted `E share`. The operation
reads the public state of one committed version as a single atomic action
and returns a resource-free value. Ordinary `?` uses retain their own
contracts. Operator dispatch, transform execution, and this registered
snapshot have separate consumers and evidence.
