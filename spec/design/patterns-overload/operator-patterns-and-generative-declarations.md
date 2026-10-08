# Operator Patterns and Requested-Name Producers

Status: canonical semantics; source consumers remain pending.

## 1. One declaration, two surface projections

```lang
P let f = (self,args):compile => { B }
P let (self,args) f => { B }
```

These are equal surface projections of ordinary compile callable material:

    CallableDeclaration<Policy, NameHead, SelfPattern, ArgPattern, Body>
    Norm(Form_call_value) = Norm(Form_pattern_name)

The callable-value form preserves ordinary self; the name form receives a
requested NameValue. Source normalization preserves syntax without lookup or
semantic declarations. Both use ordinary dependency formation and selected
CompilePartner identity. Requested-name observations enter canonical inputs.
The established callable may retain legally formed dependencies; invocation
uses only the actual self-name and admitted dependency material.

Ordinary P let lhs = rhs has extractive polarity: the known RHS is matched by
R_Gamma(lhs,rhs,rho). Generative P let H => B has the direction:

    H -> requested NameCoord -> B

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
the [same R_Gamma again at the reached layer](pattern-values-relational-semantics-and-extraction.md#82-intermediate-layer-extraction):
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
    let <field> (self, t:type) field
        => ((field#)[0])$::t;
```

The first candidate performs an ordinary receiver call. The second progresses
a type-valued receiver's NameExpr path. Both receive the full requested
NameValue; # projects structure, [0] selects the relative single-name segment,
and $ flips polarity under the Path consumer. Ordinary applicability and
specificity select; dot syntax provides no priority or receiver adaptation.

```text
T.field =_Path field::T
foo.bar.baz =_Path baz::bar::foo
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
argument. Forwarding does not place object in slot 0, implicitly form ref/share,
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
