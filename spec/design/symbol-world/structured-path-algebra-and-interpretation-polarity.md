# Structured Path Algebra and Interpretation Polarity

**Status: canonical semantic authority.**

This owner defines NameValue/Path composition, two-level reading, # projection, and the
common $ interface. The [Pattern owner](../patterns-overload/pattern-values-relational-semantics-and-extraction.md)
owns R_Gamma; the [Policy owner](policy-and-static-flow-projection.md)
owns Policy admissibility; the [call owner](function-object-call-model.md) owns
invocation. All operations use the same E and ordinary Objects. Source consumers
are pending; examples specify relations, not implemented parser coverage.

## 1. Name structure and two-level reading

### 1.1 Name observation and resident observation

A NameExpr first supplies its own structural NameValue. Only a value-expected
consumer proceeds to the resident:

```text
Read_name(n:NameExpr) = NameValue(n)
Eval_Path(n) = Read_name(n)
Eval_value(n) = Read_resident(Read_name(n))
```

NameValue retains the complete name::path structure, including endpoints and
explicit root material. It is ordinary structural material, not a NameBinding
wrapper, a new Object universe, or a Place capability. Textual nodes need not
already resolve to a binding. Path/Pattern projection deliberately consumes
this first level; ordinary value use automatically performs the second.
Read_resident resolves the structural target and observes its resident under
ordinary lookup, access, Policy, readiness and no-reopen rules.

Read_name includes the **complete computation** of the NameExpr in the ordinary
E relation; it is not a syntax snapshot or a suspension of operand evaluation:

```text
Gamma; Sigma |- name_express => NameValue(p)    -- ordinary E, first-level consumer
Gamma; Sigma |- name_express# => PathPatternProjection(NameValue(p))
value-expected continuation only: NameValue(p) -> Read_resident(NameValue(p))
```

For example, express$::express$ may evaluate both operands. The computation
may contain ordinary compile/value calls and non-name
intermediate values. Its final result must supply the legal NameValue (or the
ordinary projection domain of §2.5). # suppresses only the final resident read;
it neither stops these computations nor repeats them. A simple written
name::path is one entrance, not the limit of this rule.

In this owner Read(p) abbreviates Read_resident(p) on already formed name/path
material; Read(source_path) abbreviates the corresponding two-level composition.
It never means that the first-level structure and its resident are identical.
NameBinding identity remains distinct from both observations.

### 1.2 Two equalities

After the relevant file installation, the following may hold:

```text
Eval_value(bool::a::path) = Eval_value(bool::)
Read_resident(Read_name(bool::a::path))
    = Read_resident(Read_name(bool::))
```

The first-level structures remain different:

```text
Read_name(bool::a::path) != Read_name(bool::)
(bool::a::path)# != (bool::)#
```

Equal external reads do not merge Path structures. One default normalizer
must not conflate structural equality with equality of the values read.

### 1.3 Composition and endpoints

express::express permits evaluation and composition of both operands in their
applicable interpretation positions. It does not give arbitrary values a legal
:: candidate.

For legal internal composition:

```text
(p::q)::r =_Path p::(q::r)
```

The direction remains the language's name::path direction. This is not a
conventional left-to-right member chain, and it does not assert:

```text
Read(p::q)::r = Read(p::(q::r))
```

Once an operand has been read externally, it cannot be treated as the same
internal Path without an applicable conversion. Open endpoints such as ::a
and a:: remain part of the structure; a string array with no endpoint
information is insufficient.

:: first composes Pattern/path material. The same structure has construction
and extraction interpretations at RHS and LHS relation positions, in the same
name::path direction. This construction/extraction isomorphism fixes the
direction; it is not an arbitrary namespace-selector convention. Reversing it
to path::name would require additional reversals in those existing relations.

### 1.4 Composition grants no authority

Forming a structure creates no retained name, Place, borrow, OpenHere evidence
or member registration:

```text
PathFormed(p) does not imply Writable(target)
PathFormed(p) does not imply Retained(target)
PathFormed(p) does not imply Visible(target)
```

External reading and explicit construction actions still check their ordinary
premises. Failure cannot reopen completed name resolution.

## 2. The path_pattern structural contract

### 2.1 Ordinary extractable material

path_pattern is the working name for ordinary structured Pattern values
interpretable by a Path consumer. It is neither an AST token nor a String
wrapper carrying access authority nor a new nominal Path ontology.

PathShaped(v) is a Pattern/role judgment, not a subtype relation. Users must
be able to construct and extract name nodes, links and endpoints ordinarily.
A string-to-opaque-handle operation plus opaque_handle$ does not satisfy this
contract.

### 2.2 An equivalent ordinary representation must be observable

The following is a schematic normal form, not a mandated Rust enum or frozen
public constructor spelling:

```text
Segment:
    NameNode(s)             // one ordinary string naming one node
    ValueRoot(v)            // explicit ordinary root material
    RefRoot(r)              // explicit reference root material

Chain:
    End
    Link(segment, next)

PathPattern:
    chain
    endpoint_shape          // open/closed ends and their direction
```

Ordinary Products, named members and established sum/recursive Patterns can
describe these parts. Construction and extraction use ordinary relations,
not private implementation metadata.

Link's segment and next fields may form an all-named unordered layer. The next
relation and endpoints determine path direction, not field presentation order.
Path direction does not require converting every named layer into BareProduct.

Observable structure and composition laws are fixed. Equivalent constructor
spellings, field names, immutable subtree sharing and layouts add no semantics;
they must preserve distinctions between endpoints, segments and explicit roots.

For example:

```text
(bool::a::path)#
    -> {
         chain:
           Link(NameNode("bool"),
             Link(NameNode("a"),
               Link(NameNode("path"), End))),
         endpoint_shape:
           the established endpoint shape of this fully written chain
       }
```

By contrast, (bool::)# contains the bool segment and its own endpoint shape.
A Link can expose ordinary named fields {segment: ..., next: ...}; extraction
can obtain its string and remaining chain without first asking $ to read an
external target.

End terminates the finite representation. It does not decide whether a public
empty-Path spelling or unit exists. The established meanings of ::a and a::
remain distinct in the representation; storage notation creates no new
endpoint semantics.

### 2.3 Minimal well-formedness domain

PathShaped is a semantic judgment on ordinary observable structure, not a test
for an opaque representation tag. The following inductive presentation fixes
the legal domain; constructor and field spellings remain schematic.

Chains run from the selected member toward the root, in name::path order:

```text
Names([]) = End
Names(s :: rest) = Link(NameNode(s), Names(rest))     where s : string

Anchored([], r) = Link(r, End)
Anchored(s :: rest, r) = Link(NameNode(s), Anchored(rest, r))
    where s : string and r is ValueRoot(v) or RefRoot(b)
```

Names and Anchored are finite inductively generated chains. A back-edge is not
a finite Path chain. Immutable sharing is permitted when each observed chain
has a finite unfolding. Names([]) is auxiliary termination material, not a
public empty Path.

The observable endpoint shape has two coordinates:

```text
member end: Select | Expand
root end:   TextRoot | OpenRoot | AnchoredRoot

PathShaped(Product(chain=Names(ss), endpoint_shape=(m, TextRoot)))
    if ss is nonempty and m in {Select, Expand}

PathShaped(Product(chain=Names(ss), endpoint_shape=(Select, OpenRoot)))
    if ss is nonempty

PathShaped(Product(chain=Anchored(ss, r), endpoint_shape=(m, AnchoredRoot)))
    if m in {Select, Expand} and RootMaterial(r)
```

For TextRoot, the final NameNode is the textual root resolved at external Read;
preceding NameNodes are inward selectors. OpenRoot leaves the root endpoint
available for compatible composition or the established ambient-root reading.
For AnchoredRoot, the final node supplies the explicit root. Expand observes
the finite members of the reached host; Select observes the selected target.

```text
(field::root)#:
    Names(["field","root"]), (Select, TextRoot)
(::a)#:
    Names(["a"]), (Expand, TextRoot)
(a::)#:
    Names(["a"]), (Select, OpenRoot)

RelativeSingleName = (Select, OpenRoot)
"field" |> path_pattern:
    Product(chain=Names(["field"]), endpoint_shape=RelativeSingleName)
```

RootMaterial(ValueRoot(v)) requires legal ordinary root material of the relevant
namespace/type interpretation; RootMaterial(RefRoot(b)) requires an explicitly
formed reference carrying such a root interpretation and retaining its actual
target/generation. This is not proof that a later read, borrow or write is
authorized. Read rechecks current validity and access.

Consequently explicit root material appears at most once and only at the final,
outermost end. A representable Product with two roots, a root followed by a name,
a cyclic chain, a non-string NameNode, mismatched endpoints or an empty standalone
chain does not satisfy PathShaped. It is not repaired by guessing a root or
discarding fields. Selector spelling and target existence are separately checked
at Read; a string node is not itself a resolved selector.

Compatible composition preserves this domain and the established direction:
name chains compose before the terminal root; an explicit root cannot become
an intermediate selector or be silently replaced by another root. The
associativity law applies only when both grouped compositions are legal.
This fixes structural validity without defining a new public empty-Path unit,
universal quotation or access authority.

### 2.4 A string constructs one node

```text
ConstructNameSegment(s:string) => NameNode(s)
"field" |> path_pattern
```

This constructs relative single-name material without lookup. The string
"a::b" does not automatically parse into two Path segments or arbitrary source.
Ordinary selector rules decide whether a string is legal at the eventual use;
some structures can form yet fail external reading.

Construction returns no resolved NameCoord, recovers no Place and grants no
access.

### 2.5 Path projection and round-trip

The round-trip domain is legally Path-projectable values, including already
bound NameValues; it is not restricted to source NameExpr. For such a value v
and its normalized projected Pattern p:

```text
PathPatternProjection(v) = p
    implies Interpret_Path(p) =_Path v

PathPatternProjection(Interpret_Path(p)) = p
    for normalized p in the legal PathPattern domain
```

Here =_Path observes v's Path structure, never resident equality or equality
of arbitrary value representations. For example, a string's defined projection
observes its relative single-name structure (§2.4); it does not parse source.
The two operations are inverse on this legal structural domain, modulo Path
equivalence and Pattern normalization. Endpoints and explicit root material
remain part of that observation. Projection grants no resident-read authority.

NameExpr is an entrance to this value law through Read_name, not its final
domain restriction. The source spellings specialize it as follows:

```text
n# = n |> path_pattern = PathPattern(Read_name(n))       for legal NameExpr n
e# equivalent_to e |> path_pattern                     where projection is defined
e => v; e# = PathPatternProjection(v)                  for ordinary non-name e

Gamma; Sigma |- (n#)$ =>_Path p
p =_Path Read_name(n)
PathPatternProjection(p) = n#

Gamma; Sigma |- (n |> path_pattern)$ =>_Path p
p =_Path Read_name(n)
PathPatternProjection(p) = n |> path_pattern

a = v                                      -- already bound Path-projectable value
a# = a |> path_pattern = PathPatternProjection(v)
Gamma; Sigma |- (a#)$ =>_Path q
Gamma; Sigma |- (a |> path_pattern)$ =>_Path q
q =_Path v
PathPatternProjection(q) = a#
```

These round-trip laws are indexed by the Path consumer: Interpret_Path in
the interpretation-polarity judgment of §4 interprets the projected Pattern material.
Thus (a#)$ =_Path a and (a |> path_pattern)$ =_Path a compare structural
observations. The surface shorthands (n#)$# = n# and
((a |> path_pattern)$)# = a# assume this Path interpretation of the inner
polarity flip. They give neither Policy nor other Pattern consumers an implicit
Path decoder and add no decoding step to the general definition of $.

Both spellings use one projection. A NameExpr operand supplies Read_name(n),
without entering Read_resident; a general expression supplies its ordinarily
evaluated value. A resolved Pattern binder holding a NameValue supplies that
bound material, not a new node made from the binder's spelling; thus a# in the
generative forwarder observes the requested field::adl, not the local label a.
Projection/reinjection of this bound value does not repeat NameExpr lookup or
reconstruct a Path from the binder spelling. Opposite-context interpretation evaluates
its operand once and checks its consumer's ordinary readiness/admissibility.
A value cannot be projected merely because its source once
looked like a Path. Undefined projection fails through ordinary applicability,
without source reparsing or reconstruction from a resident value.

This is value normalization, not source quotation: parentheses, whitespace,
spans and equivalent construction histories add no identity. Equal residents
do not imply equal projections. Actual value/ref anchors remain observable,
and their ordinary dependency obligations remain in force. The Path consumer
reconstructs structure; any subsequent resident read
belongs to its surrounding value-expected consumer, not to $ itself.

### 2.6 Segment observation and indexing

The linked presentation has an equivalent ordinary normalized observation:

```text
Norm_path(p) = <ss, omega>
ss : String*
omega : Omega
PathPattern ~= {<ss,omega> in String* x Omega | PathShaped(<ss,omega>)}
```

Omega retains Select/Expand, TextRoot/OpenRoot/AnchoredRoot, explicit ordinary
root material, and the endpoint information required to reconstruct the Path.
This is a restricted legal domain, not an arbitrary pair or a string-only identity.
Names run in name::path order. No anchor can be recovered from strings alone.

ValueRoot(v) and RefRoot(r) retain actual ordinary material; r retains its
target/generation under ordinary reference semantics. Any dependency/lifetime
obligations induced by this material remain governed by the ordinary
[dependency](dependency-observation-and-realization.md) and lifetime relations.
They are not an extra Omega coordinate or a new owned dependency axis.

For each existing segment position 0 <= i < length(ss):

```text
p[i] : path_pattern
p[i] = PathPattern(Names([ss[i]]), (Select, OpenRoot))
((field::adl)#)[0] = PathPattern(field::)
```

Indexing returns a relative single-name path_pattern, never a bare string. It
does not copy the original root endpoint or anchor into the relative result.
It reads no external resident and grants no authority. Its evaluation retains
ordinary source/dependency checks; projection does not extend anchor lifetimes.

RHS colon is ordinary structural construction, not a new Path or slicing
primitive. Lexing uses maximal munch `:: > :`. Value parsing uses
`FullExpr > : > ,`: each colon slot consumes its full ordinary expression,
one colon layer assembles all slots, then comma assembles the Product. Colon
has no fixity, associativity or operator entry.

    (e1:e2:...:en) = (e1,e2,...,en) |> slice
    (:a) = ((),a) |> slice
    (a:) = (a,()) |> slice
    (:) = ((),()) |> slice
    (a,,b) = (a,(),b)
    (a::b:c) = (a::b,c) |> slice
    (a:b,c) = ((a,b) |> slice,c)
    (a,b:c) = (a,(b,c) |> slice)

Empty colon and comma slots use the same unit completion law. There is no
missing-endpoint ontology. Ordinary `slice` structure is formed through Product
Pattern items and ordinary struct:

    slice = (((product items) slice)$) |> struct

Container, Path, string and user callable
families extract it using R_Gamma. Each family defines admissible item Patterns
and boundary interpretation, including unit. Those ordinary declarations do
not introduce Slice_Omega, SliceIR, protocol or another evaluator. Their missing
consumers are roadmap frontier, not an open slicing algebra.

LHS, BindingSlot, callable head and Pattern `x:T` remain annotations.

## 3. Late textual roots and explicit anchors

### 3.1 First-level projection retains unresolved structure

```text
let root = T1;
let p = (field::root)#;

{
    let root = T2;
    p$
}
```

Assuming both T1 and T2 legally provide field, the inner ordinary value use of
p$ interprets root at its resident read and therefore uses T2. Projection does not
silently retain the outer root's NameBindingId merely because root could have
been found while p was defined.

### 3.2 The use environment follows ordinary callable rules

The read position is the occurrence's established lexical/semantic environment,
not an arbitrary dynamic caller's stack. Both ordinary and in-place callable
bodies use their established dependencies
and lexical rules. In-place syntax forms those dependencies automatically;
invocation does not resolve an outer name anew by spelling.

Once that use resolves its target, projections, delayed execution, runtime
residue and cache hits cannot look it up again by spelling:

```text
late binding determines the first Resolve position
no-reopen constrains uses after that Resolve
```

### 3.3 Explicit root material preserves its own dependencies

ValueRoot(v) retains explicit ordinary root material. RefRoot(r) retains an
actual reference. Their observations, transfers and dependencies use ordinary
value and reference rules respectively; they are not one fixed-address model.

A reference in a Path gains no lifetime extension. Replacement or invalidation
does not automatically retarget it while the Path survives. Lifetime judgments
still decide retention, movement and escape.

### 3.4 Text nodes are not already resolved dependencies

NameNode("x") in pure Path structure is not an external read of x. Dependency
discovery cannot create a capture from a string or uninterpreted name node alone.

A dependency arises when an actual external observation requires it, or when
the structure explicitly retains value/reference material. Projection need not scan
the surrounding namespace, and later same-spelled declarations cannot revise
an already completed read.

## 4. Interpretation polarity

### 4.1 Common flip and structural consumers

```text
V = RHS/value interpretation
S = structural interpretation
Flip(V)=S; Flip(S)=V; Flip^2=Id
Interpret(e$,C)=Interpret(e,Flip(C))
BareName(a;p)=a::p
```

$ may nest to arbitrary source depth. Even parity preserves the interpretation;
odd parity flips it. It performs no resident read, stage change, borrowing,
OpenHere grant, PolicyMode change or lifecycle action. ReadVal remains an
independent Name/resident consumer.

Path, Policy, extraction and complete structural type formation specialize S.
Within S, e$ interprets e in V and the surrounding structural consumer checks
its resulting ordinary material. Within V, e$ interprets e in S; type formation
publishes a complete type with actual Val2 witnesses and registered roles.
Bare names use shared navigation completion. Bare S operators use
(operator[op])$; V operators use operator[op].

Each reached operand executes once under the common E; projection never repeats
its effects or establishes another stage copy.

### 4.2 No textual macros or implicit binders

Interpretation polarity does not re-lex or parse strings, arbitrarily evaluate source AST,
implicitly allocate HoleBinderId, capture another same-spelled hole, or grant
Writable/OpenHere.

Existing HoleRef material retains its actual PatternRoot/HoleBinderId. Invalid
scope or incomplete material is inapplicable or erroneous under existing rules;
renaming or redeclaration does not repair it. New holes require explicitly
formed binding material. The flip performs no alpha-renaming.

### 4.3 Readiness and local selection

The consumer's material must be legally available. A real remaining
continuation may be retained; missing observations or consumers are unavailable.
A body that requires the
Pattern for its own selection cannot be run early to produce that Pattern.

In particular, a candidate cannot execute to obtain evidence for its own
earlier applicability. Failure to obtain opposite-context Policy material does not reinterpret
the spelling as a concrete atom or new hole.

### 4.4 Concrete atoms, holes and opposite-context Policy material

```text
runtime let a = e
    // runtime is a concrete constraint; no new deduction binder

<p> p ...
    // explicitly declares a HoleBinderId, solved by ordinary extraction

<> p$ let a = e
    // interprets p in V, then checks its result in the surrounding S consumer

runtime let ~ <> runtime let
runtime let != <runtime> runtime let
<> p let != <> p$ let
```

The last distinction concerns interpretation roles; it does not reject every
otherwise legal program with the same spelling. It provides no implicit
dereference repair.

### 4.5 Projection and polarity are independent

In structural interpretation, name_express$ evaluates its operand in V and
the surrounding consumer uses its resulting ordinary material.
(name_express |> path_pattern)$ first performs path_pattern projection in V.
The surrounding S consumer then interprets that result. These operations are
not equivalent in general; $ never inserts the projection.

name_express may be a complex computed name expression, not merely a single
name. In general, (name_express |> path_pattern)$ != name_express$ as operations.

The equation e# equivalent_to e |> path_pattern applies to any expression in
the projection's domain. It does not quote arbitrary Policy syntax or callable
source bodies. General $ retains its independent admissibility judgment and
single evaluation, including existing HoleBinderId, scope and readiness rules.

### 4.6 Construction and extraction interpret bare material differently

The following judgments use existing Path composition and R_Gamma, not another
evaluator. In their common legal non-extraction domain, with the same ordinary
operand evaluation and material:

```text
Gamma; Sigma |- name::a =>_Path p
Gamma; Sigma |- name::(a$) =>_Path p
    hence name::a =_Path name::(a$) in non-extraction use
```

This is not an unconditional source rewrite. In extraction position:

```text
R_Gamma(name::a, c, rho)
    interprets bare a as Pattern/path material with navigation-chain inheritance

Gamma; Sigma |- a => v
Admissible_Pattern(v)
R_Gamma(name::Interpret_Pattern(v), c, rho)
    interprets name::(a$) using explicitly evaluated and reinjected material
```

Explicit reinjection cuts the bare Pattern's inherited navigation-chain
interpretation. Thus name::a != name::(a$) as general extraction judgments;
particular equal results do not establish a rewrite theorem. Evaluation,
admissibility, readiness and Hole identities still obey §4.1–§4.3.

## 5. Open navigation, name types and buckets

### 5.1 Preserve the observed members' names

The ordinary compile type family is:

```text
N_s = s |> name
D_Sigma(a) = {s_i -> v_i}
Read_Sigma(::a) = Product[ v_i (s_i |> name) ]
```

s is an ordinary string parameter in extractable Pattern material, not an
implicit provenance field of the read value.

The notation (val ("name" name)) denotes one name-pattern-bearing entry.
It is not the two-bare-value tuple (val, ("name" name)), which would destroy
the all-named layer. The result is Product, not OverloadGroup; constructing
s |> name does not create a resolved Path or NameBinding.

### 5.2 Ordinary name-to-string observation

The name family owns an ordinary explicit string projection:

```text
n = NameObservation(s)
n : N_s
N_s = s |> name

R_Gamma(N_<h>, Content(n), rho) => rho(h) = s
n |> string => s : string
```

The family-provided string projection returns the same parameter obtained by
ordinary registered Pattern extraction. N_<h> is schematic notation for the
name-family Pattern with an explicit hole. The result is the stored string
parameter, not the source binder spelling and not a NameBindingId, NameCoord
or resolved path. This is an ordinary explicit projection, subject to ordinary
applicability and selection, not an implicit conversion or general reflection
over arbitrary values.

This observes a name-family string parameter, not a general Path truncation.
A generative name head receives the full requested NameValue, for example
field::adl. Default ADL forwarding uses (a#)[0] to obtain the relative field::
PathPattern; it does not discard the root through a string conversion.
A legal name-to-string projection remains available when text is requested.

### 5.3 Names belong to the observed layer

In a local block:

```text
let a = bool::;
```

a binds the RHS value at its existing level. Open observation ::a sees the
value's exposed internal structure: if bool has if/else members, it sees that
layer. It does not wrap the whole BoolValue in another ("a" name) layer.

An outer layer actually containing a and b exposes entries labelled "a" and
"b" when that outer layer is observed. Renaming a local holder does not rename
the held value's internal members.

### 5.4 Extraction and bucketing

```text
let x ("a" name) = observed_material;
Bucket_L(v N_s) = s
```

Ordinary Pattern extraction obtains the a bucket. An explicit hole may extract
the name parameter; the arbitrary user binder x does not alter s. L limits
bucketing to the current layer. Equal strings do not merge NameCoords rooted
at different owners.

Same-name closure contributions naturally enter the same bucket. The member
contribution relation still decides TypeAdd, merge or conflict:

```text
SameNameBucket does not imply ArbitraryMerge
SameNameBucket does not imply ConstructionAuthority
```

### 5.5 Current observations are finite

A generator's ability to answer legal future names does not make ::a enumerate
all possible requests. An open Product contains only the finite actual
material/snapshot available to that observation.

Later legal generation may change a later current observation without changing
an earlier immutable Product. Consumers cannot trigger infinitely many
generators to obtain a supposedly complete Product, or use cache contents as
the sole namespace authority. Current observation and retained snapshots
remain distinct.

## 6. Ordinary construction and extraction example

The following is ordinary named-Product material, in schematic notation rather
than frozen constructor spellings:

```text
node = NameNode("field")
link = Product(segment = node, next = End)
p = Product(chain = link, endpoint_shape = RelativeSingleName)
R_Gamma((segment:s, next:n), Content(link), rho)
  -> rho(s) = NameNode("field"), rho(n) = End
R_Gamma(NameNode(text), Content(rho(s)), rho2)
  -> rho2(text) = "field"
```

Named fields may be permuted without reversing the chain. Changing `next` or an
endpoint changes Path structure. These extractions require no external Read;
Any subsequent resident observation checks its own Read environment independently
of the polarity flip.
`End` terminates this representation; it does not establish a public empty-Path
unit or choose new open-end semantics.
