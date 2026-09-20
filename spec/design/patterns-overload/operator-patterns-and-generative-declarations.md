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

Ordinary P let lhs = rhs has extractive polarity: the known RHS is matched by
R_Gamma(lhs,rhs,rho). Generative P let H => B has the direction:

    H -> requested NameCoord -> B

Inside H, (self,args) is still an ordinary extraction head:

    R_Gamma((self,args), call_material, rho)
    concrete name f adds Selector(requested_coord)=f
    generative name _ adds no concrete selector constraint

The requested coordinate must already be legally formed. Extractive _ remains
a wildcard binding no named value. Concrete f is more specific than _ under
ordinary Pattern specificity, after applicability. There is no generator or
fallback-name priority. Multiple incomparable maxima remain ambiguous; selected
failure never reopens another generator.

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

Direct declarations such as `let + = closure` and
`let + = operator[+]` bind the ordinary operator name subject to the same
rules. Their intended source consumer is pending. They are not a general
left-hand-side-free compound assignment `let += g`; compound assignment
and operator-name binding remain distinct syntax roles.

Same-slot ordinary combination retains OG_s and its spelling. It does not
implicitly combine different spellings, infer a selector from an arbitrary
group, or create String-to-Path conversion. General first-class .field/path
algebra remains open; the dot-operator rule above does not close that topic.

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

Policy + and || deduction is a consumer of these same registered relations,
HoleBinderId and require constraints. The [policy owner](../symbol-world/symbol-policy-and-compile-flow-projection.md)
owns coordinate legality, omission/inheritance and the joint invocation relation.

The ordinary law query's default state follows the meta owner's retained
instance/member Place protocol. Customization requires current OpenHere and
Writable; consumers observe the current committed payload, not a copied
outer binding or an optimizer-private fact. Later writes do not change a
previous committed semantic decision.
