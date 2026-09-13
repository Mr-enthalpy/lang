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

## 2. Grammar facts and ordinary operator families

The grammar fixes a finite OpTok vocabulary and its fixity, precedence and parse
associativity. For each recognized token:

    ParseOperator(op) = <op::type, Fixity, Precedence, ParseAssociativity>
    *::type belongs to operator::type

The fixed family path is unresolved language material until semantic lookup;
the parser does not enumerate semantic candidates. Programs may contribute
ordinary callable semantics to existing families under ordinary authority, but
cannot create operator tokens or change grammar precedence during their own
meta evaluation. There is no Parse -> Eval -> Parse feedback and no separate
OperatorObject ontology.

An application a*b carries the structure OpApp(*::type,a,b). Surface omission
of callable-object self does not change Type(callee)=Type(first self). The
explicit value/call form remains available when that self must be named.

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

Here * already denotes the grammar-selected *::type family. The head observes
its generative invocation/name relation, not GenerateToken("*"). No fourth
operator projection or separate parameter system is introduced.

## 4. Generated occurrences and registration

A generated occurrence may realize an ordinary Val2 resident under the name
realization and invocation laws. That occurrence supplies no V_tau registration
and no Pattern structural registration: no DirectPatternChild, ConstructEdge,
ExtractEdge or FieldView evidence follows from it.

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
