# Operator Patterns and Generative Meta Declarations

Status: canonical semantics; source consumers remain pending.

## 1. One declaration, two surface projections

```lang
P let f = (self,args):meta => { B }
P let (self,args) f => { B }
```

These are equal projections of the same meta declaration:

    MetaDecl<Policy, NameHead, SelfPattern, ArgPattern, Body>
    Norm(Form_call_value) = Norm(Form_pattern_name)

Neither form is a higher-level language implemented by the other. MetaDecl is
notation for common declaration material, not a new Object axis or a mandate
to introduce semantic declarations in Raw AST. Source normalization preserves
the syntax-directed material; resolution, Pattern solving and realization
remain evaluator work. The callable-value form still exposes the ordinary
callable-object self. This equality applies to the displayed meta declaration,
not arbitrary lexical bindings of non-meta RHS values.

The callable layer establishing this declaration accepts only an ordinary
`=>` implementation and has no capture slot:

```text
MetaDecl(C) => Placement(C) = Ordinary
MetaDecl(C) => CaptureClause(C) = absent
NoMetaCaptureAxis:
  MetaDecl(C) => no ExplicitClosureCapture(C)
  MetaDecl(C) => no AutomaticClosureDependencyFromUnpassedOuterLocal(C)
```

A capture-bearing meta callable is invalid MetaDecl material; it is not first
formed as a captured ordinary closure and then reinterpreted as meta. A
no-`=>` body is not an in-place spelling of MetaDecl. The same rules apply to
both surface projections, including generative names. They concern the current
declaration's identity layer, not ordinary closures legally defined inside B.

Unpassed caller/enclosing locals remain masked. Material that must affect an
invocation enters its admitted In dependency closure; stable definition
relations already fixed by the selected callable/parent owner and lawful
meta-instance state remain available under the
[meta owner's boundary](../meta-invocation/meta-object-invocation-and-policy-reduction.md#2-meta-instance-identity).
MetaInstanceRootKey has no CapturedEnv coordinate.

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
generative MetaDecl form. Omitted extraction heads do not create a capture slot
or permit automatic acquisition of an unpassed enclosing local.

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

An ordinary meta implementation still directly returns its own instance tau_M.
Expression-body freedom does not permit arbitrary foreign direct result types;
ordinary payloads and closure material enter through legal instance formation.

## 2. Grammar facts and ordinary operator dispatch

Grammar fixes OpTok spelling, fixity, precedence and parse associativity.
Semantic dispatch preserves three distinct forms:

```text
naked OperatorUse(op) -> operator[op]
dot operator .op     -> op::adl
explicit path        -> the written ordinary path
```

Paths such as op::type remain ordinary library paths, not the grammar's
hardwired target. Source cannot create tokens or change parsing by meta
evaluation. OperatorUse and OperatorNameValue are distinct roles: the op
argument of operator[op] reads the ordinary operator name and does not
recursively invoke operator[op]. Ordinary lookup, type, policy and lifetime
checks still apply; this rule is not a raw token bypass.

### 2.1 OperatorOverloadGroup and current-slot selection

```text
OG_s = s |> OperatorOverloadGroup
```

This ordinary string-to-type meta family accepts ASCII spellings in the
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
group, or recover semantic coordinates from String. The [Path owner](../symbol-world/structured-path-algebra-and-pattern-splice.md)
separately permits single-name structural construction and defines external Read.

## 3. Three projections of one application structure

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
It implies no OpenHere, mut type ref or meta type ref, and cannot mutate a
registered witness. No separate closed-generative authority is introduced.

This restriction belongs to the occurrence, not permanently to the value. The
same ordinary value may obtain an appropriate witness through another lawful,
non-generative declaration. Neither callspace registration nor Pattern role
registration may depend on which generated names were queried later.

The [name owner](../symbol-world/names-and-overload-groups.md) separates name
coordinates, realization, Places and residents; the [meta owner](../meta-invocation/meta-object-invocation-and-policy-reduction.md)
owns invocation identity and result formation. Generative notation grants no
extra construction, write or lifetime authority.

## 5. Laws are ordinary meta results, with separate consumers

Expressions such as op |> associative and op |> commutative are ordinary meta
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
revalidated under the [ordinary E/O boundary](../meta-invocation/evaluation-residual-and-optimization.md).

Policy + and ordinary Pattern deduction consume these same registered relations,
HoleBinderId and require constraints. The [policy owner](../symbol-world/symbol-policy-and-compile-flow-projection.md)
owns coordinate legality, omission/inheritance and the joint invocation relation.

The ordinary law query's default state follows the meta owner's retained
instance/member Place protocol. Customization requires current OpenHere and
Writable; consumers observe the current committed payload, not a copied
outer binding or an optimizer-private fact. Later writes do not change a
previous committed semantic decision.


## 6. Ordinary dot-name generation and forwarding

### 6.1 Default generator skeleton

The following name/call skeleton belongs in the adl namespace. Shared Policy
holes must use the existing legal head forms; omission below is not a wildcard
over every coordinate.

```text
adl/
    let <a> a =>
        <t:type>(self, object:t, ...args) => {
            (object, args)
                |> ((a#)[0])$::t
        };
```

Its interpretation uses the established relations:

```text
requested name field::adl
-> name-head extraction binds its full NameValue as a
-> a# projects its complete path_pattern
-> [0] selects relative single-name path_pattern
-> $ injects Path material
-> navigation under the explicit t
-> ordinary selected call
-> direct terminal result delivery
```

For a field request:

```text
a = NameValue(field::adl)
a# = PathPattern(field::adl)
(a#)[0] = PathPattern(field::)
((a#)[0])$::t =_Path field::t
```

The [Path index](../symbol-world/structured-path-algebra-and-pattern-splice.md#26-segment-observation-and-indexing)
owns the relative single-segment result. It discards the original adl endpoint
rather than copying it onto field::. Ordinary name-to-string projection remains
available for text observation, but is not this Path truncation operation.

### 6.2 Canonical lowering

```text
.field  -> field::adl
E.field -> E |> field::adl
```

The normalizer preserves the dot/name/path source role; it does not own the
semantic authority to generate the actual forwarding implementation.

OperatorUse, OperatorNameValue, dot selectors and explicit paths retain their
distinct entrances. OG_s preserves spelling through selector results; only
explicit Forget removes it. Path support does not erase OG_s to ordinary OG.

### 6.3 Requests do not mutate the forwarded type

field::adl forms a permitted ordinary result/member occurrence. Its body reads
field::t; it does not inject field into t. Frozen generative rules can answer
later legal requests without infinite predeclaration or reopening Pattern/V_tau
registration in either adl or t.

When a meta instance supports generation, its direct result and ordinary
payloads obey the existing instance model. ADL does not broaden the direct
meta result class.

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

Replacing compiler-private closure sugar preserves this semantic boundary.
