# Glossary

Definitions are specific to this repository. Versioned entries distinguish the
frozen v0.1/v0.2 historical surface from the amended v0.5 contract. Terms may
have different meanings in general PL theory.

---

## v0.1.w Raw AST Stability Window

The maintenance and contract-stabilization window after the completed v0.1 Raw
AST Frontend. During this window richer literal spelling and the pipe
branch-name shorthand were implemented as the final v0.1.w additions. v0.1.w
is now closed; it was followed by the now-closed v0.2 freeze.

_See also: Raw AST, v0.2 Raw AST Contract Freeze._

---

## v0.2 Raw AST Contract Freeze

The closed historical stage after v0.1.w. It froze the then-current Raw AST
frontend input and prepared the exact boundary consumed by v0.3. Its documents
remain historical snapshots and are not rewritten for later parser changes.
`v0.2` was not a parser-expansion phase and did not implement Normalized AST.

_See also: v0.1.w, Frontend Semantic Amendment v0.5-A, Raw AST,
Normalized AST, raw-ast-contract-freeze-v0.2.md._

---

## Frontend Semantic Amendment v0.5-A

The versioned amendment that classifies post-v0.2 parser changes without
rewriting the frozen history. Closure orthogonalization and malformed-tail
error preservation are hard structural corrections; `DotClosure` is a
normalization-driven extension; `Ellipsis`/Pack and callable-tail alternatives
are new syntax amendments.

_See also: v0.2 Raw AST Contract Freeze, Raw AST Contract v0.5._

---

## Raw AST Contract v0.5

The current Raw AST contract obtained by applying Frontend Semantic Amendment
v0.5-A to the frozen v0.2 baseline. It defines the 20-symbol/33-diagnostic
surface, callable tail, first-class dot closure, Pattern pack, orthogonal
closure placement, and validated normalized handoff.

_See also: Frontend Semantic Amendment v0.5-A, PatternValidatedNormProgram._

---

## Token

The output of the lexer. A token is the smallest lexical unit: a `Name`, `Literal`,
`Symbol`, `Trivia`, `Invalid`, or `Eof`. Tokens carry a span and are consumed by
the parser. The lexer does not assign semantic roles to tokens.

_See also: Trivia, Name, Literal, Symbol, Span._

---

## Trivia

A token class representing whitespace, comments, or other non-semantic text.
Trivia tokens are skipped by the parser but their spans must remain available
for diagnostic positioning. The lexer must preserve trivia spans; the parser may
discard trivia after consumption.

Line and block comments are trivia. Nested block comments are a lexer concern
(depth counting on `/*`/`*/`), not parser syntax.

_See also: Token._

---

## Name

A token class representing an identifier. Names include what traditional languages
call keywords. In v0.1, `return` (contextually recognized in return terminal
forms at the form level; remains a `Name` token lexically), `else`, `match`, `drop`, `move`, `sync`,
`effect`, `fn`, `type`, `meta`, `runtime`, `compile`, `seal`, `const`, `mut`,
`public`, `private`, `export`, `namespace`, and `struct` are all ordinary
`Name` tokens at the lexical level.

> **Distinction**: A `Name` token is not a keyword. Semantic strength does not
> imply lexical keyword status.

_See also: Token, Strong context._

---

## Strong context

A parser state in which certain `Name` tokens or symbols are interpreted
structurally. Examples: `let` at form start, the active head clauses
`require`/`pre`/`post`/`lifetime pre`/`lifetime post`, `with` inside let bindings, `<>` in binding
contexts.

Outside a strong context, these tokens retain their ordinary `Name` or `Symbol`
identity.

_See also: Name, Hole, DeduceList._

---

## DeduceList

A sequence of hole declarations enclosed in `<...>`, recognized only in strong
binding contexts such as extract-let binders, closure heads, parameter binders,
and return binders. Outside these contexts, `<` and `>` are ordinary symbols;
in expression/operator contexts they may be operator spellings.

Normalized DeduceLists are left-to-right dependent telescopes. A declaration
annotation sees inherited and earlier declarations, never itself or later
declarations. A hole name cannot be repeated within one `PatternRoot`; a new
independent Pattern root may shadow an inherited spelling. Each declaration
receives an alpha-normalized `HoleBinderId` qualified by callable owner,
Pattern root, and root-local ordinal, and a named `HoleRef` targets that exact
identity rather than merely repeating its spelling. Build integration maps the
frontend owner to a persistent `SemanticOwnerId`. Source spans are provenance,
not identity. Generated
receiver holes use hygienic generated keys rather than source spelling. A
callable-head telescope scopes captures, parameters, policy, return, clauses,
body, and inherited nested callables. Within a BindingSlot, policy precedes
the local DeduceList. Norm exact binding covers Pattern/policy occurrences;
value-side names and navigation remain unresolved.

An explicitly empty DeduceList in `let <> P` is semantically significant: it
selects `BinderPresence = Absent` and leaves `P` as the Pattern. It is distinct
from `let _ P`, which contains a real anonymous / wildcard position. Canonical
semantic owner:
[`pattern-values-relational-semantics-and-extraction.md`](../design/patterns-overload/pattern-values-relational-semantics-and-extraction.md).

_See also: Hole, Strong context, CanonicalSkeleton._

---

## Hole

A binder declared in a `DeduceList` that acts as a wildcard standing for an
unknown type or value in following syntax. Raw canonical parsing may mark its
spelling with `CanonicalNameRole::Hole`; normalized uses carry an exact
`HoleBinderId`. The anonymous `_` annotation placeholder is not a named Hole
and targets no DeduceList declaration.

_See also: DeduceList, CanonicalSkeleton._

---

## HoleRef

A resolved use of one declaration in a `DeduceList`. Its identity targets the
exact owner/root-qualified `HoleBinderId`; equal source spelling alone is never
enough. `HoleRefs(P)` is computed after this scope resolution and separates
closed result-as annotations from deduction-bearing annotations.

_See also: DeduceList, Hole, Closed annotation, Deductive annotation._

---

## SemanticOwner

A parent-linked semantic identity domain for namespace objects, callable
anonymous types, canonical meta-invocation instances, and generated objects.
Semantic identity is `(SemanticOwnerId, local identity)`; source file, span, and
printable path are provenance only. Every callable, including an in-place
closure, has a callable owner. Standalone closure materialization defaults to
an anonymous function-object type derived from that owner, but an associated
call-entry implementation may receive a different named receiver type. Source
navigation prints the current/innermost callable-local `Self` owner first and
enclosing owners to its right.

_See also: CallableReceiverType, PatternRoot, PackageBoundary, Mount._

---

## CallableReceiverType

The type of the caller object injected into invocation-frame slot 0. It is
independent of `CallableOwner`. For a standalone function object it defaults to
the owner-derived anonymous function-object type; for an associated `()` entry
it is the formal receiver type selected from the same-name candidate family,
such as `T ref`.

The first written formal binds this object by position under any legal spelling.
Only later formals consume the explicit call-site Product. A mismatch is an
ordinary invocation type-check failure, not a separate `let ()` declaration
rule.

_See also: SemanticOwner, Callable Implementation Tail._

---

## Associated Val2 Contribution

A let-shaped declaration consumed inside `struct` construction that adds
ordinary value-facet material below the current Pattern owner without adding a
Val1 structural slot or Pattern extraction member. Its initializer may be
value-bearing and callable. The empty target `()` installs the current owner's
special call entry. Contributions remain uninstalled until the outer
construction commits its namespace delta.

_See also: DefaultExtractionView, SemanticOwner._

---

## PatternRoot

One independent Pattern/extraction alpha boundary inside a `SemanticOwner`.
Nested BindingSlots, Products, Sequences, annotations, DeduceLists, and Pack
operands inside an extraction retain the same root. An independent let Pattern
or callable head creates a new root. Hole names are unique within one root;
different roots may use normal lexical shadowing.

_See also: DeduceList, Hole, SemanticOwner._

---

## Pattern

The `P` coordinate of `Object = <Val1?,P,Val2>`. It defines a proof-relevant
relation over the complete content `<Val1?,Val2>`; it is not a pointer to one
`Val1` schema, a type tag, or a copy of namespace shape. Canonical owner:
[`pattern-values-relational-semantics-and-extraction.md`](../design/patterns-overload/pattern-values-relational-semantics-and-extraction.md).

_See also: Pattern derivation, PatternValue, Direct Pattern child._

---

## PatternValue

An ordinary Object observed together with its Pattern coordinate. The term
does not name a separate rank or carrier: its value, Pattern, and associated
members remain the ordinary `<Val1?,P,Val2>` coordinates. Open authority
(`Anchor` / `WindowLive_Σ`), places, and borrow capability are orthogonal.

_See also: Pattern, Object normal form (`Norm`), Open authority._

---

## Relational Pattern semantics

The calculus whose base judgment `R_Gamma(P,c,rho)` relates a Pattern to
complete Object content and retains a successful derivation `rho`. Satisfaction,
observation, entailment, and extraction are derived from the set of such
derivations; the base calculus assumes neither uniqueness nor universal
invertibility. Canonical owner:
[`pattern-values-relational-semantics-and-extraction.md`](../design/patterns-overload/pattern-values-relational-semantics-and-extraction.md).

_See also: Pattern derivation, Extraction, Pattern normalization._

---

## Pattern derivation

One proof-relevant witness `rho` of `R_Gamma(P,c,rho)`. It may justify selected
alternatives, direct structural incidence, extraction roles, or binder
observations. It is semantic evidence, not a fourth Object axis.

Canonical owner:
[`pattern-values-relational-semantics-and-extraction.md`](../design/patterns-overload/pattern-values-relational-semantics-and-extraction.md).

_See also: Relational Pattern semantics, Direct Pattern child, Extraction._

---

## Direct Pattern child

A selector-indexed child registered by privileged structural construction.
Direct incidence implies that the child is present in `Val2`, but ordinary
`Val2` presence or navigability does not imply direct Pattern incidence.
`struct` and `extend` may register it; ordinary navigated `let` may not.

Canonical owner:
[`pattern-values-relational-semantics-and-extraction.md`](../design/patterns-overload/pattern-values-relational-semantics-and-extraction.md).

_See also: Structural Pattern child, Associated Val2 Contribution, FieldView._

---

## Structural Pattern child

Synonym for a direct Pattern child when emphasizing its role in construction
or extraction structure. It is not inferred from callable availability or
arbitrary namespace membership.

_See also: Direct Pattern child, Constructor, Extractor._

---

## Extraction

An observation justified by successful Pattern derivations and an extraction
interface. The base calculus may yield zero, one, or several results; unique
extraction and constructor/extractor inversion are theorems of particular
Pattern families, not universal axioms.

Canonical owner:
[`pattern-values-relational-semantics-and-extraction.md`](../design/patterns-overload/pattern-values-relational-semantics-and-extraction.md).

_See also: Relational Pattern semantics, Extractor, DefaultExtractionView._

---

## Constructor

An ordinary callable Object registered as a construction role by a particular
Pattern family. Merely being callable does not grant that role, and a value
constructor does not automatically acquire borrowed reverse variants.

Canonical owner:
[`pattern-values-relational-semantics-and-extraction.md`](../design/patterns-overload/pattern-values-relational-semantics-and-extraction.md).

_See also: Extractor, FieldView, Direct Pattern child._

---

## Extractor

An ordinary callable Object registered as an extraction role by a particular
Pattern family. A structural extractor backed by a real `ProjectionSlot` may
derive ref/share observations; an arbitrary value isomorphism may not.

Canonical owner:
[`pattern-values-relational-semantics-and-extraction.md`](../design/patterns-overload/pattern-values-relational-semantics-and-extraction.md).

_See also: Extraction, Constructor, ProjectionSlot._

---

## FieldView

A Pattern-interface role connecting a structural selector to an ordinary
callable field observation. Pattern registration owns the role relation, not a
second copy of the callable. An ordinary associated helper does not become a
FieldView merely by being present in `Val2`.

Canonical owner:
[`pattern-values-relational-semantics-and-extraction.md`](../design/patterns-overload/pattern-values-relational-semantics-and-extraction.md).

_See also: Direct Pattern child, Extractor, Associated Val2 Contribution._

---

## Binderless Pattern

A Pattern position with `BinderPresence = Absent`. The explicit binding form is
`let <> P`; the current atomic pipe shorthand `|> P { ... }` has the same
binderless head as `|> (<> P) { ... }`. `_` is instead a real wildcard position,
so `(_ P)` is distinct. Canonical owner: the relational Pattern authority
linked under
Pattern.

_See also: DeduceList, Pattern, InPlaceClosureAST._

---

## FullNameView

The complete package-internal namespace and overload view. Same-package
descendant owners may use an ancestor's non-export entries through lexical
lookup. Unrelated siblings do not acquire that visibility merely by sharing a
package.

_See also: ExternalNameView, DefaultExtractionView._

---

## ExternalNameView

The identity-preserving external namespace projection used after lookup crosses
a package boundary. It requires export-retention admission, public reachability
through every access-path component, and a resolved candidate view. Every
candidate of an admitted symbol preserves its identity, `Pv:Pp`, and
`PolicyMode`; consumer Policy selection and dynamic-legality checks happen only
after this stable lookup.

_See also: FullNameView, PackageBoundary, Mount._

---

## DefaultExtractionView

The structural Pattern view exposed by default extraction. It is distinct from
both name views. Private structural members remain in the full structural model
but are absent from this view. Rich custom `?` construction remains future
design.

_See also: FullNameView, ExternalNameView._

---

## PackageBoundary

Build/namespace metadata assigning a stable `PackageId` to a namespace subtree.
`PackageOf(node)` uses the nearest boundary ancestor. Physical directory names
do not define package or symbol identity.

_See also: Mount, SemanticOwner._

---

## Mount

A namespace-graph redirect edge from an alternative access path to an existing
target node. Mount traversal may cross a package boundary and switch to
`ExternalNameView`, but it never copies the target symbol or changes its
identity.

_See also: PackageBoundary, ExternalNameView._

---

## CanonicalSkeleton

A syntactic pattern used in extraction contexts (extract-let binder, extract
parameter, extract return). The historical skeleton is a sequence of
`CanonicalElement` items. Under v0.5-A, Ellipsis may occur as a direct
canonical Pattern Sequence child and normalizes to `NormPattern::Pack` inside
`NormPattern::Sequence`; it is not hidden as a new skeleton atom. The parser
builds shape only and does not execute matching.

All canonical skeleton golden tests in v0.1 are parser preservation tests.
No semantic meaning (matching, destructuring, equality, constructor
interpretation, or admissibility) is assigned to any skeleton shape.
The `Hole`/`NodeName` distinction is a parse-time role marker, not a
semantic binding commitment.

_See also: DeduceList, Hole, ProductForm, CanonicalNameRole._

---

## ProductForm

A parenthesized form with top-level commas, such as `(a, b, c)`.

In expression context, a product form is product construction and is preserved
as `ProductExprAst`. In binding / extraction context, the same surface form is
product extraction and is preserved as `ProductExtractAst` or a canonical
product extraction skeleton.

Leading, doubled, or trailing commas create explicit unit product elements.
These unit elements are not omitted, not wildcards, and not implicit discards.

The parser does not decide whether a product is constructible, destructible,
layout-compatible, type-compatible, or callable. ArgPack and ArgPackRole are
removed historical terms and are not language-level concepts.

_See also: ProductExtract, Segment, PipeExpr._

---

## PipeExpr

A top-level expression formed by splitting tokens at `|>` into segments.
`PipeExpr` is the entry point for expression parsing.

```text
PipeExpr ::= Segment ("|>" Segment)*
```

_See also: Segment, ProductForm._

---

## Segment

One part of a `PipeExpr`, containing a sequence of `OperatorExpr` and product
elements in the operator-aware design. Each segment has a `has_incoming` flag
indicating whether a prior segment exists.

_See also: PipeExpr, Atom, ProductForm._

---

## Atom

The smallest self-contained expression unit. Atoms include:

- `Name("x")`
- `IntLiteral("42")`
- `StringLiteral("\"text\"")`
- `Group(PipeExpr)`
- `Closure(ClosureAst)`
- `NavPath(components)` (components are `NavComponentAst` in source order)
- `DotClosure(selector)` (leading `.name`; no captured receiver)
- `MemberSugar(object, selector)` (selector is `SelectorAst`)
- `DoubleDotSugar(object, selector, args)` (selector is `SelectorAst`)
- `BracketCallSugar(object, operator, args)` (`obj[args...]`; operator spelling `[]`, `args` is a `ProductExprAst`)
- `Error`

Atoms are constructed by parsing a base and then folding suffixes (`::`, `.`,
`..`, `[...]` bracket call, and postfix operators). Operator sugar itself is
stored at the `OperatorExpr` layer, not as a general `Atom` variant.

Leading `.name` is a base atom, distinct from suffix folding.

`BracketCallSugar` is source-preserving sugar for the operator spelling `[]`; it
is not indexing/slicing/container access. The `[]` operator is a contextual
paired operator name, bindable and referable in operator-name positions. Raw AST
may preserve it inside historical alias-let syntax, but no semantic operator
alias operation exists.

_See also: ClosureAST, ProductForm, OperatorSugar, PostfixOperator, SelectorAst, NavPath._

---

## SelectorAst

A name-like construct appearing after leading `.` or in suffix position after
`.` or `..`.
In the current parser phase:

```text
SelectorAst ::=
    Text(NameAst)     // from TokenKind::Name
```

Numeric selectors have been removed. Only `Name` selectors are accepted.

_See also: NameAst, NavComponent, MemberSugar, DoubleDotSugar._

---

_See also: NavPath, SelectorAst, OperatorName._

## OperatorName

A `Symbol` spelling that can be used as an operator identity component, an
expression operator, a binder name, or an innermost navigation component.
Operator names are not keywords, and their spelling does not imply arithmetic, comparison,
mutation, assignment, lookup, or evaluation semantics.

An overloadable operator identity is `spelling + fixity + arity`, where fixity
is `Binary` or `Postfix`. `Prefix` fixity is a Raw AST marker reserved for
the prefix-negative surface sugar `-x` (normalized away before operator lookup);
it is not an overloadable operator fixity.

_See also: Fixity, Arity, NavComponent, OperatorSugar, PrefixNegative._

---

## Fixity

The syntactic position of an operator relative to its operands. The operator
design distinguishes:

- `Binary` and `Postfix`: overloadable operator fixities (part of operator
  identity for declaration, lexical operator-environment selection, and lookup).
- `Prefix`: a Raw AST surface marker used only for the prefix-negative `-x`
  sugar. Prefix negative is normalized to typed-zero binary subtraction before
  operator lookup. The `Prefix` fixity is not a declarable or overloadable
  operator fixity.

_See also: OperatorName, Arity, PrefixNegative, PostfixOperator._

---

## Arity

The number of operands associated with an operator syntax form. Arity is part
of operator identity.

_See also: OperatorName, Fixity._

---

## OperatorSugar

An AST shape inside `OperatorExprAst` that preserves operator syntax without
lowering it to an ordinary call. Planned shape:

```text
OperatorExprAst ::=
  | OperatorSugarAst {
    operator: OperatorName,
    fixity: Prefix | Postfix | Binary,
    args: Vec<OperatorExprAst>
  }
```

Operator lookup is a future semantic pass. It first resolves the nearest lexical
ordinary value `operator : operator`, then selects
`operator[OperatorIdentity]`, where
`OperatorIdentity = spelling + fixity + arity`. It is not direct lookup of an
operator spelling as a visible binding, ADL, or type-directed parser lookup.

_See also: OperatorName, Fixity, Arity._

---

## Naked operator (`operator[op]`)

A bare operator expression resolves through the nearest lexical
`operator : operator` binding and selects `operator[OperatorIdentity(op)]`,
where `OperatorIdentity = spelling + fixity + arity`. It is not direct
spelling lookup, not ADL, and not type-directed parser lookup.

```text
NakedOperator(op)
    -> operator[OperatorIdentity(op)]
```

A global/operator-identity callable may choose to forward to `.op`
(`op::adl`) in its body, but "global operator" and "ADL" are not
synonymous, and not all unqualified operators automatically perform ADL.

_See also: ADL (.op), OperatorName, OperatorSugar._

---

## ADL (`.op`, `op::adl`)

Argument-dependent lookup through the dot-operator path. It is an
independent entity from the naked operator:

```text
.op ≡ op::adl
```

The three operator-lookup paths are orthogonal:

```text
NakedOperator(op)    -> operator[OperatorIdentity(op)]
DotOperator(.op)     -> op::adl
ExplicitPath(P::op)  -> P::op
```

An explicit path `P::op` is direct path lookup and does not run operator
forwarding or ADL.

_See also: Naked operator, OperatorName, Assignment operator (.=)._

---

## Assignment operator (`.=`, `=::adl`)

The default bare `=` resolves to `operator[=]`, which forwards to `.=`,
which is `=::adl`:

```text
operator[=]   -> .=
.=            ≡ =::adl
```

Required source behavior:

```text
object : T        object ref = value   -- form ref, then .=
object : T ref    object = value        -- direct .= on the ref's target
```

The assignment receiver is never implicitly formed from an ordinary `T`
value's carrier place (`NoImplicitBorrowFormation` remains absolute; no
`AssignmentReceiverFromPlace`). When the receiver itself is `T ref`,
assignment writes `Target(receiver)`, not the place storing the ref handle.

Custom Val2 may define setter candidates through `.=`. Setter participation
does not make anything a P structural field.

_See also: ADL (.op), Naked operator, NoImplicitBorrowFormation._

---

## Overload Candidate

A callable entry prepared for a given call. Final preparation first resolves a
Symbol and forms `CallableProjection(S) = DedupCandidateIdentity(V_S ⊎ V_τ)`,
then applies the current policy view to the projected candidates,
obtains each surviving value's type, and resolves
that type's associated `()` entry. Non-callable values are discarded. A derived
compile companion is itself a complete `Val2` function object with stable
object identity, its own function-object type, and its
own associated compile `()`; its existence is derived from the callable under
the compile transform (`CompilePartner(F) = C(F)`, canonical in
`function-object-call-model.md` §8), not from Symbol injection — the symbol-facet
entry used at lowering is implementation substrate. It enters candidate
preparation through the same
path. Compile projection leaves the projected invocation as an ordinary call
until normal compile lookup and overload resolution. A same-name namespace
bucket is only current transitional substrate, not the final candidate
definition.

_See also: OverloadSpecificity, OverloadResolutionPipeline,
`spec/design/patterns-overload/overload-resolution-design.md`,
`spec/design/symbol-world/symbol-policy-and-compile-flow-projection.md`._

---

## Fully Admissible Candidate

An overload candidate that has passed every hard legality check for the current
call: namespace/policy-view visibility, associated `()`, argument/Pattern shape,
receiver and parameter policy pairs, stage legality, any supplied target-result
pair/type/rank/facet constraint, concept and ordinary require satisfaction, and
other compile/type prerequisites. The set of all such candidates is `A`. The
total `OutputModeDemand(call)` is a later PolicyMode-product preference
coordinate, not an optional hard-admissibility constraint.

Preference survivors are not a second meaning of "qualified." They are the
successive subsets obtained by applying the fixed ordered preference filters to
`A`.

_See also: OverloadCandidate, OverloadResolutionPipeline, MustSelectStrategy._

---

## Derived Compile Companion Object

A complete compile-policy `Val2` function object mechanically derived from an
eligible runtime function object. It has its own object and type identity, an
associated compile `()` entry, stable provenance back to the origin runtime
object, and the `must_select_if_qualified` overload strategy. It is not a hidden
fallback or an identity-less extra call entry.

_See also: OverloadCandidate, MustSelectStrategy._

---

## Must-Select Strategy

An overload strategy carried by a `Val2` function object and propagated to its
prepared call candidate. It activates only when that candidate belongs to the
fully admissible set `A`. One admissible must-select candidate must be the sole
final preference survivor; several admissible must-select candidates conflict.
The strategy is not infinite priority and does not forbid non-overlapping
same-name overloads. Source strategy metadata uses `=> name { ... }`, with
`[[name]] { ... }` as the no-`=>` disambiguation form; `@` remains lifetime
syntax.

_See also: FullyAdmissibleCandidate, DerivedCompileCompanionObject, CompilePartner._

---

## PolicyMode

The real whole-slot coordinate `PolicyMode = {const, plain, mut}`. It is
orthogonal to `Val1` shape and to the `Pv:Pp` Policy pair. Plain Policy is the
`plain` point, not an omitted value, inference variable, or `const || mut`
choice. At one constrained position the context-indexed rows are
`succ_const: const > plain > mut`, `succ_mut: mut > plain > const`, and
`succ_plain: plain > const = mut`. In a plain context, `const` and `mut` remain
co-maximal and ambiguous when no `plain` candidate survives; equality never
means arbitrary choice. Across receiver, parameters, and a target-result
constraint when present, candidates are compared by product partial order. A
candidate dominates only when it is no worse everywhere and strictly better
somewhere. Incomparable maxima remain ambiguous; there is no score,
exact-match count, position weight, or lexicographic fallback. Delete members
participate in the same comparison. Preference never grants capability.

_See also: CapabilityRealization, Policy Pair, FullyAdmissibleCandidate,
OverloadResolutionPipeline._

---

## CapabilityRealization

The independent realization of an operation at an input/output `PolicyMode`
coordinate. A family may leave any of the nine 3×3 coordinates absent or
realize it as `default`, `delete`, or `custom`. This grid is not the
three-point Policy preference order: selecting `mut` may still select a
`delete` member, and `mut` on a non-reference object does not universally imply
writability. Realization is a stable declaration/intrinsic candidate-family
fact and may be retained in a namespace candidate snapshot. It is distinct
from `DynamicLegality_Γ`, which is formed for the selected invocation in a
concrete consumer context and is never stored in `Σ_export`.

_See also: PolicyMode, DynamicLegality_Γ, Borrow view._

---

## DynamicLegality_Γ

The legality judgment formed for one already selected invocation in consumer
context `Γ`. It may depend on required capabilities, current writability,
lifetime, access, escape, `OpenHere`, and construction authority. These are
premises of legality, not a second 3×3 capability-realization layer. The
judgment is not declaration metadata and must never be copied from an internal
observation edge into `Σ_export`. External lookup returns the stable candidate
snapshot; ordinary Policy/realization selection chooses the invocation; the
consumer then forms `DynamicLegality_Γ_consumer`. Failure rejects that selected
invocation without reopening candidate lookup or Policy maxima.

_See also: CapabilityRealization, Namespace Symbol Views._

---

## Seal Visibility

Seal slices are exposed only in SealStatic; meta slices only in OpenStatic;
compile slices in both. Symbol resolution precedes this exposure, so a hidden
slice does not erase the binding. Seal policy grants no global scan capability.
Compiler-known privileged seal operations may inspect exactly Wpre, the least
semantic dependency closure rooted at exported symbols, actually materialized
results of exported meta functions, and their parameter/signature dependencies.
Wseal never expands that scan domain, though committed Wseal symbols remain
explicitly addressable.

_See also: PolicyPair,
`spec/design/symbol-world/symbol-policy-and-compile-flow-projection.md`._

---

## Overload Specificity

The priority rule that determines which overload candidate is selected when
multiple candidates survive initial filtering. In this design, overload
specificity is **extraction-pattern specificity**: candidates are ranked by
how deeply their extraction pattern penetrates the unified construction-
expression tree of the call operand. Structural depth evidence is compared
before node-class evidence. At equal depth, ordinary explicit matches outrank
explicit pack matches, which outrank ordinary discards, which outrank pack
discards. Captured length never changes the count. A simple pack has one
outward evidence node. Raw `...(a, b)` is non-canonical, and a legal headed
structured operand still contributes only one outward Pack node at its
containing level; internal evidence stays below the stable head at the next
structural level.
Specificity does not depend on declaration order or an ad-hoc conversion-rank
table. This extraction-only rank is not a const/mut fitness score and never
resolves candidates that remain incomparable under the PolicyMode product order.

_See also: OverloadCandidate, OverloadResolutionPipeline,
`spec/design/patterns-overload/overload-resolution-design.md` §4._

---

## Overload Resolution Pipeline

The fixed process that selects a unique overload candidate:

```text
Resolve Symbol
  -> call-site candidate-family filter
  -> generate candidates
  -> fully admissible A
  -> declaration-side candidate policy D
  -> ordinary partial orders
  -> unique selection
```

The call-site layer acts before candidate generation; this PR closes only its
pipeline position, while source syntax such as future `|[[annotation]]>` and a
general selector algebra remain deferred. Declaration-side `fallback` /
`must-select` policy acts only after hard admissibility and cannot repair a
rejected candidate or restart name lookup. `Bp` then uses
the Policy product partial order across all constrained positions; no total
score or lexicographic fallback resolves incomparable candidates. For an
authorized atomic Runtime-migration call only, input/output endpoint Policy fit
extends this same product as `Bp'`. With no endpoint coordinates, `Bp'` is
exactly old `Bp`. Remaining
side-effect-free preference filters apply in one fixed normative order:
entry, concept, extraction, first-order-over-instantiated,
in-place-over-non-in-place, then named strategy rules. Each
filter is independent of candidate enumeration order; filters are not assumed
to commute. A named strategy only sees fully admissible candidates and cannot
restart lookup. Delete members participate normally, and ordinary uniqueness is
constrained by `must_select_if_qualified` strategies activated from `A`.

Current source cannot construct a fallback candidate role, so current calls
have `D = A`. If a future fallback strategy is exposed, its fixed semantics
will apply within declaration-side policy before Bp: any admissible non-fallback
candidate, including `delete`, suppresses fallback permanently. This future
suppression is not B6 and later failure cannot restore fallback.

Lifetime policy is not a type/compile candidate filter. This revision defines no
lifetime-driven re-selection, refinement order, ABI class, or second selection
stage. Any future lifetime check receives the already unique ordinary overload
result, under the boundary in
`spec/design/lifetime/lifetime-policy-and-overload-boundary.md`. That is a
restriction on lifetime *rules*; `@` itself is an ordinary overloaded operation
with its own candidate set.

Full overload resolution is deferred to v0.10+ and depends on the pattern-space
and extraction-chain infrastructure. The formal specification is in
`spec/design/patterns-overload/overload-resolution-design.md` §5.

_See also: OverloadCandidate, OverloadSpecificity, Concept,
`spec/design/symbol-world/symbol-policy-and-compile-flow-projection.md`._

---

## Lifetime Policy Boundary

The stage boundary between lifetime rules and ordinary overload selection:
lifetime policy is not an ordinary stage policy atom, and no lifetime rule may
reopen or change the already unique ordinary overload result. `@` is evaluated at
a stage; it does not name one.

This boundary is *not* a claim that `@` lacks semantics. `@` reifies a
continuation-relative `LifeName` as a `LifetimeValue`, never a borrow view and
never a `type ref`; its former two instance groups
(`LifetimeFact` for value instances and `P ref` for borrowable pure pattern
slots), `t@ : type ref`, and the borrow-type fixed points `type ref@ = type
ref` / `type share@ = type share` are retired. `ref` and `share` are the borrow
constructors (`PrivilegedActualPlace(ref-family)` /
`PrivilegedActualPlace(share-family)`); `@` does not share that place-acquisition
privilege. Explicit higher-level selection uses
`t |> (type ref)` / `t |> (type share)`. Borrow-constructor composition
preserves its resident target, but that is not a `@` overlap. The semantic core
now closes `SemanticContinuation`, `LifeName`, first-class ordinary
`LifetimeValue`, `NameView<T>`, `origin`, gapless half-open `Region`,
use/move/drop generations, cleanup-before-lifetime, Pre/Post summaries,
pairwise-distinct exclusive-write and same-root shared-read defaults plus finite
Pre patch, exact move-origin preservation, selected CopyConstruct lifecycle
posts, and an extensible Color vocabulary with finite/monotone relations in
each committed compilation semantic universe.
Concrete IR carriers, summary compression, and the checker remain
unimplemented.

_See also: `@`, Escape check, `spec/design/lifetime/lifetime-policy-and-overload-boundary.md`._

---

## Object normal form (`Norm`)

The structural identity of an object. The governing principle has three parts:

1. Ordinary structural content is governed by `Object = <Val1?, P, Val2>` and
   normalizes recursively through `Val1` and `Val2`.
2. Complete types are rank-indexed closures `tau = <Q, V_τ>` over that Object
   material; `tau` participates in Pattern observation through `Core(tau) = Q`
   and is not itself an ordinary Object.
3. Rank is determined by declaration: the rank of a callable's result is
   solely its declared result rank
   (`Rank(result(F, x)) = DeclaredResultRank(F, x)`); there is no default
   rank-preservation rule and no `RankShift(F, n, m)` as a second mechanism
   (`DeclaredResultRank`, §2.3 of the canonical owner below);
   evaluation stage, description depth, and carrier form cannot apply
   `UniverseSuccessor` implicitly. `RefTy`/`ShareTy` are `RankTransparent`
   (`∀n. F : U_n -> U_n`) as declared result properties; field
   projection follows the declared result type; `TypeOf` is genuinely
   rank-changing.

Bare Product, Sequence, `product`, and Symbol are constructor instances,
not parallel aggregates. Borrow targets are horizontal identity-bearing leaves.
The complete equations and well-foundedness rules belong to
[`type-values-places-and-borrow-views.md`](../design/symbol-world/type-values-places-and-borrow-views.md).

`Val1? = absent` is payload absence only. In the well-formed Object kernel every
`Val2` is navigable, so `Pure(x) <=> NamespaceRole(x)`. `TypeRole(x)` is an
additional imported judgment and is therefore the strict refinement.
Anchor and carrier place remain outside normal form.

_See also: Policy Pair, Borrow view, Open authority (`OpenHere_Σ`, `WindowLive_Σ`)._

---

## Open authority (OpenHere_Σ, WindowLive_Σ, Visible_Σ)

Whether a `PatternValue` may be structurally modified in the current evaluation
context is decided by two independent facts:

```text
Anchor(v) = ⟨PatternRoot(v), Navigation(v)⟩

GenerationRegime(v) ∈ { MetaGenerated, NonMetaGenerated }
                     -- immutable birth classification (value attribute)
WindowLive_Σ(v)     -- construction window still open at the current program
                       point (evaluation/window state)
Visible_Σ(v)        -- current frame can obtain v
OpenHere_Σ(v)
  iff WindowLive_Σ(v)
  ∧ AuthorityMatches(v, Σ)
```

`AuthorityMatches` resolves the frame that still owns the value's static
anchor on the current evaluation stack; authority belongs to that owning
frame, not unconditionally to the stack-top callable (canonical owner
§12.1.1):

```text
EvaluationCoordinate(f)
  = ⟨RootCoordinate(Callable(f)), ActiveInlineClosurePath(f)⟩

RootCoordinate(F)
  = MetaPartnerRoot(F, GenericArgs)   if Generic(F)
    CallableRoot(F)                   otherwise

AuthorityFrame_Σ(v)                      -- non-meta context
  = the nearest still-active frame f owning Anchor(v)
    (EvaluationCoordinate(f) = Anchor(v)), searched outward from the
    current frame, skipping compile/transparent frames, and stopping at
    any meta invocation frame (a meta boundary masks v)

AuthorityMatches_nonmeta(v, Σ)
  iff AuthorityFrame_Σ(v) exists

CurrentEvaluationCoordinate_meta(Σ) = ⟨NearestMetaRoot(Σ), ε⟩
AuthorityMatches_meta(v, Σ)
  iff Anchor(v) = CurrentEvaluationCoordinate_meta(Σ)
```

Passing an open value into a deeper ordinary call frame does not destroy
authority: the caller's frame remains still-active and continues to own the
anchor. The equality is opaque navigation-coordinate equality, not prefix
matching; `AuthorityMatches` is the regime-dispatched name of the same
judgment. `PatternValue` records static anchor; the evaluation stack records
the current dynamic evaluation position; `PatternValue` does not record
dynamic call history.

`WindowLive_Σ` is a property of the current evaluation state: the construction
window has not been permanently closed at the current program point.
`OpenHere_Σ` adds the contextual question: does the current
evaluation stack still contain the frame that owns this value's anchor, and is
the window still live there? `Visible_Σ` captures a third state: the value
exists and its window may still be live, but the current frame cannot obtain it
(e.g. shadowed by a deeper meta invocation). Clone, value copy, and
construction-transparent `compile` frames preserve the anchor and
`GenerationRegime`; they do not preserve or manufacture source-place identity
and do not create a fresh window. `Anchor(v) ∉ Norm(v)`; `GenerationRegime(v) ∉
Norm(v)`.

`OpenHere_Σ` does not propagate along owned field relations: no
`OpenHere_Σ(child) ⇒ OpenHere_Σ(parent)` or
`WindowLive_Σ(parent) ⇒ WindowLive_Σ(child)` implication holds. Each
PatternValue's open authority is determined independently by stack-relative
authority-frame resolution, not by the state of any parent or sibling value.

Openness is a static value property: `OpenHere_Σ(v)` alone does not mean the
current computation flow re-traverses `v`. Live reentry additionally requires
an active evaluation edge — `OpenEvalReentry_κ(v)` — whose criteria are
canonical in
[`../design/symbol-world/type-values-places-and-borrow-views.md`](../design/symbol-world/type-values-places-and-borrow-views.md)
§2.1.1.

The state transition of the open window is one-way: once closed
(`WindowLive_Σ(v) := false`), it is never retracted. An ordinary meta
invocation forms a new boundary. Non-meta construction follows its stable
lexical-owner interval; the open window is a linear evaluation flow segment,
and each action on the value receives a disposition
(`OpenDisposition_κ ∈ { Continue, Terminate, Reject }`, canonical owner
§12.1.2): legal terminal actions (`UseForVal1`, meta-argument use,
residual-runtime/control at the generation level, owner exit) end the window
(`Terminate`); `UseForVal1` and meta-argument use inside an opaque non-meta
inline closure below the value's open coordinate are forbidden (`Reject`).
`ControlFlowSplit`/`ControlFlowMerge` are generation-coordinate events: at the
generation level they terminate the window, while at a deeper ordinary
coordinate they are irrelevant to the outer window.
`EffectiveOpenSegment(p) ⊆ OwningInlineClosureEvaluationSegment(p)`. This
judgment is orthogonal to place writability and borrow lifetime.

_See also: `OpenHere_Σ`, `WindowLive_Σ`, `extend`, `type ref`, `GenerationRegime`._

---

## PatternValue container kernel

The four ordinary ordered-container cases used by the value model:

| element structure | fixed outer shape | erased outer shape |
| --- | --- | --- |
| homogeneous | `T * N` | `T * omega` |
| heterogeneous | bare Product | `product` |

`T * N` and `T * omega` are finite homogeneous ordered Object containers; the
former includes `N` in classifier identity and converts to the latter. Their
mechanical `[]` value/ref/share members are bounds-domain partial projections
over ordinal `ProjectionSlot`s. Ordinal topology is fixed:
`CanCreateMember(sequence,pos_i)=false`. Stage is the ordinary dependency meet
over container, index, and selected observation. This says nothing about layout,
capacity, or growth.

They are formed by the global privileged type constructor `*`:
`*(T,N) -> T*N` and `*(T,omega) -> T*omega`. Both preserve the element type's
universe rank: `rank(T*N) = rank(T*omega) = rank(T)`.

All four cases are ordinary Objects. `BareProduct(a_0,...,a_n)` uses intrinsic
ordinal `Val2(pos_i)` children; a `T*N` or `T*omega` Sequence Object holds that
bare Product Object in `Val1`. A `product` value is the same ordinary wrapper
with the erased outer classifier. Their normalization and owned traversal are
therefore exactly the general `Object` rules.

Sequence indexing therefore selects
`ProjectionSlot(Resident(Val1(sequence)), pos_i)`, not `Nav(sequence,pos_i)` on
the outer Sequence `Val2`. Value/ref/share indexing changes only observation
kind and shares one bounds domain.

A bare Product retains its concrete arity and element-type vector in its fixed
shape. The global built-in `product` type classifies any finite heterogeneous
bare Product without moving that concrete vector into the outer type identity.
No general runtime `product[]` is defined in this stage. In both erased cases,
“erased” affects only outer classification; the actual `Val1` information is not
discarded.

_See also: ProductForm, Object normal form (`Norm`), `Symbol` constructor value._

---

## Symbol terminology: NameBinding, `symbol` type, `Symbol` constructor

Three distinct concepts share the word "symbol" in this specification. The
distinction is frozen to prevent conflation:

```text
NameBinding(a)   = the environment contains a resolvable name a
                   (source/environment naming concept)
`symbol`         = the language's first-class type value; symbol : type
                   (same category as uint8 : type)
x : symbol       = classification judgment TypeOf(x) = symbol
Symbol(...)      = the unique well-defined semantic constructor
                   S = ⟨τ?, V_S?⟩; written SymbolValue(τ?, V_S?) when
                   emphasis on the value is needed
```

**Key rule.** `NameBinding(a)` does not imply `a : symbol`. A bare sentence
"a is a symbol" is prohibited as a formal statement: it is ambiguous between
"a is a binding name" and "a : symbol". Natural language prefers "binding
name", "symbolic name", or "named binding" for the naming concept; English
documents write "the `symbol` type" or "the type value `symbol`" for the
type. Capitalized `Symbol` is reserved for the semantic constructor.

**Counterexample.** In `a : uint8; b : type; c : symbol`, all three names
satisfy `NameBinding`, but only `c` satisfies `TypeOf(c) = symbol`.

**Interpretive note.** Phrases such as "symbol hierarchy", "symbol root",
or "meta-symbol structure" describe the symbolic/name/meta-structural side
of the model; they must not be read as automatically carrying a
`symbol`-typed value.

_See also: `Symbol` constructor value (value of type `symbol`), Complete type
closure (`tau`), Namespace (source name)._

---

## `Symbol` constructor value (value of type `symbol`)

An ordinary value of the ordinary `symbol` type. Its mutable member content is
`Val1(Symbol) = Σ = <tau?, V_S?>`, never a mutable `P x Val2` side
structure. `tau`, when present, is the complete type value
`tau = bind alpha.<Q,V_τ[alpha]>`; `V_S` is the Symbol's own sibling candidate
space, independent of `V_τ = CallSpace(tau)`. Namespace projection selects
`Core(tau) = Q` when `tau` is present; type projection returns the whole `tau`.
All four shapes `<None,None>`, `<tau,None>`, `<None,V_S>`, `<tau,V_S>` are
well-formed. The Symbol never forms a type from its own contents: `tau` is
formed before installation and carried as a whole value; it does not return
bare `Q` or recover a defining Symbol later. `Q` and the members in `V_τ` are
ordinary Objects; `tau` is their type-specific closure, not an Object
embedding or fourth Object coordinate. Consumers project `tau` as needed:
ordinary Pattern/namespace observation and type-rank equality/keying use
`Core(tau)=Q`; type-as-callee uses `CallSpace(tau)=V_τ`; copying, `extend`,
and `inject` preserve or transform the whole closure. `CallSpace(τ)` is an
intrinsic property of the closure, fixed at formation and never recovered
from the host Symbol, source binding, or carrier provenance. `@` is the
continuation-relative name-reification operation that yields a lifetime value and never a
`type ref` (canonical owner
`../design/lifetime/lifetime-policy-and-overload-boundary.md` §1–§2).
Callable val members project across the typed buckets of `V_S` to the formal
`OverloadSet`; when `tau` is present, `CallableProjection(S) = DedupCandidateIdentity(V_S ⊎ V_τ)`
forms one candidate set with no priority or fallback.

Symbol normalization is an extensional optional complete type closure plus map
of typed member sets. Stable member and candidate identities, callable-body
identity, and selection-relevant declaration annotations live in the ordinary
member objects and survive normalization. Only order and repeated contribution
of the same member are quotiented away; conflicting declarations remain
diagnostics.

`Σ` is not a private record carrier. Its optional member is represented by an
empty/singleton bare Product, each typed bucket is an ordinary `T_c*omega`
Sequence, each `(T_c, bucket)` entry is classified by `product`, and those
homogeneous entries form a `product*omega` carrier. The logical
`<tau?, V_S?>` notation projects this ordinary Object composition.

The privileged `struct` operation forms a complete type value directly: it
creates the type core `Q_struct = Core(tau_struct)` satisfying
`TypeRole(Q_struct)` with mechanically generated
field/access/assignment/borrow partner families entering `V_τ` at the formation
event. The Symbol carrying the formed value appears only at a subsequent
binding/install. It is therefore a complete-type-forming structural generator,
not merely a type constructor.

_See also: PatternValue container kernel, Overload Candidate, `extend`._

---

## Complete type closure (`tau`)

The immutable type value `tau = bind alpha.<Q,V_τ[alpha]>`, where `Q` is the
ordinary pure Object core satisfying `TypeRole(Q)` and `V_τ = CallSpace(tau)`
is the callspace fixed at type-value formation: the direct TypeMember members
placed into `tau` at that event, not a post-hoc partition of a shared Symbol
space and not a global function of the bare core `Q`.
`Core(tau)=Q`; `CallSpace(tau)=V_τ`. Members created under the same `Q` after
formation never retroactively enter an existing snapshot, and a copied or
extracted `tau` keeps its captured `V_τ`. It is not an Object embedding,
a fourth Object coordinate, or a second owned copy of `V_τ`. Consumers
project `tau` as needed: ordinary Pattern/namespace observation and
type-rank equality/keying use `Core(tau)=Q`; type-as-callee uses
`CallSpace(tau)=V_τ`; copying, `extend`, and `inject` preserve or transform
the whole closure as required by their own judgments. `@` is a
continuation-relative name-reification operation that yields a lifetime value, not a
borrow and not part of the complete-type observation classification.
Canonical owner:
[`type-values-places-and-borrow-views.md`](../design/symbol-world/type-values-places-and-borrow-views.md).

`tau` and its `⟨P, Val2⟩` description material are one semantic entity with
two equivalent views (`τ ≡ DescriptionClosure(P, Val2)` — the
`SameEntityTypeInvariant`): `DescriptionView` and `TypeClosureView` are
projections of the same entity, and constraints on either view constrain that
one entity jointly. Normative statement:
[`pattern-values-relational-semantics-and-extraction.md`](../design/patterns-overload/pattern-values-relational-semantics-and-extraction.md)
§15, restated in
[`type-values-places-and-borrow-views.md`](../design/symbol-world/type-values-places-and-borrow-views.md)
§2.2.

_See also: Object normal form (`Norm`), TypeMember, `Self_τ`._

---

## LowerTypeClosure

The lowering/representation mechanism that stores a complete type closure in
an Object-position carrier: `LowerTypeClosure : WellFormedTau -> Object` with
`DecodeTypeClosure(LowerTypeClosure(tau)) = tau`. `tau` itself is a semantic
package; whenever an implementation must carry a closure in an Object
position (e.g. the `BareProduct` element inside a Symbol's `Σ_Object`), it
stores `LowerTypeClosure(tau) ∈ Object`, never `tau` directly. The lowering
is representation-only: it is not derived from `¬Object(τ)`, and it is not a
precondition for ordinary semantic operations on `τ` (see
`NoSemanticDispatchByCarrierMembership`). Fidelity:
`Norm(LowerTypeClosure(τ₁)) = Norm(LowerTypeClosure(τ₂))` iff
`Norm_type(τ₁) = Norm_type(τ₂)` — the lowering is injective up to the type
value's own normalization and introduces no extra distinction. The lowering
is representation-opaque: ordinary Pattern, Object navigation, and Val1/Val2
inspection cannot observe a second identity system beyond the `tau` API.
Canonical owner:
[`symbol-first-meta-construction-and-pattern-injection.md`](../design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md)
§4.7.

_See also: Complete type closure (`tau`), Object normal form (`Norm`), `Symbol` constructor value._

---

## TypeMember

Home eligibility and snapshot membership are two layers. An ordinary val
member is included in a complete type snapshot exactly when it is actually
carried by that snapshot's `V_τ` and its anonymous classifier has direct
immutable canonical home `TypeMemberScope(Q)`:

```text
HomeEligible_Q(F)   -- equivalently TypeMember_Q(F)
  iff Anonymous(F)
  and DirectClassifierHome(F) = TypeMemberScope(Q)

TypeMember_τ(F)
  iff F ∈ ClassifierDomain(V_τ) and HomeEligible_{Core(τ)}(F)
```

Descendant ownership, copying, rebinding, and namespace
installation are insufficient. Creating a classifier with that direct home
requires current construction authority for `Q`; ordinary callable creation or
navigated `let` cannot nominate the scope and thereby forge membership.
Concretely, a `V_τ` callable member is realized through an anonymous type
directly under `τ` (symbol-first §2.1.1): the callable's entry is the
associated `()` Val2 leaf of that anonymous type.
`V_τ` is fixed at formation; classifiers created later under the same scope
enter only a new snapshot, never an older `V_τ`.
Canonical owner:
[`symbol-first-meta-construction-and-pattern-injection.md`](../design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md).

_See also: TypeMemberScope, `Self_τ`, `Symbol` constructor value._

---

## TypeMemberScope

The direct classifier-home scope derived from the canonical self-pattern root
of the type core `Q`:

```text
CoreAnchor(Q) = CanonicalSelfPatternRoot(Q)
TypeMemberScope(Q) = MemberScope(CoreAnchor(Q))
```

It is invariant under core changes that preserve that root
(`CoreAnchor(Q') = CoreAnchor(Q) ⇒ TypeMemberScope(Q') = TypeMemberScope(Q)`,
`TypeMemberScopeStability`, canonical §2.1) and is independent of the
remaining `Q` snapshot.
Creation under this scope is the membership proof that places a member into
`V_τ` at type-value formation; arbitrary descendants do not qualify. Selecting
this home at classifier creation requires current construction authority for
`Q`, so ordinary namespace installation cannot manufacture membership.

_See also: TypeMember, SemanticOwner._

---

## `Self_τ`

The canonical self reference inside the binder-aware type closure
`tau = bind alpha.<Q,V_τ[alpha]>`. During `Norm_type` it becomes
`BoundRef(alpha)`, which is not an owned child. It establishes a
`SymbolicReferenceEdge` (symbolic anchoring), never an `EvaluationEdge_κ`, so
the presence of `Self_τ` at the same stage does not mean the current
computation flow re-enters the value: `OpenEvalReentry_κ` requires an active
evaluation edge, not merely a stored reference. The normalizer's active stack
is the **normalization/owned-recursion stack**, a distinct object from the
evaluation-active flow. After
authorized back-references are erased, the owned graph must satisfy
`WellFounded_kappa` (`type-values-places-and-borrow-views.md` §2.1): finite,
acyclic after back-ref erasure, all back-refs bound, and restricted to
authorized static edge kinds under static-eval generation (covering both
compile and meta) and acyclic once materialized at runtime.
`Self_τ` is one restricted static back-reference instance, not an exceptional
cycle and not a general recursive-Object constructor. Meta and nonmeta closures
share the same `bind alpha` / `Self_τ` representation; the difference lives in
the symbolic anchoring relation `SelfResolve` (meta: root-relative/deferred;
nonmeta: finite same-stratum static backreference).
Canonical owner:
[`type-values-places-and-borrow-views.md`](../design/symbol-world/type-values-places-and-borrow-views.md)
§2.1–§2.1.1.

_See also: TypeMember, Object normal form (`Norm`), SymbolicReferenceEdge._

---

## ProjectionSlot

The resident-specific projection location
`<ParentResidentIdentity, Selector>`, whose contents are `None` or `Some(Object)`.
The reusable logical `ProjectionCoordinate(parent_place,selector)` is distinct
from this slot identity. `let` may change one slot from `None` to `Some` without
retargeting; bare `=` never creates. A projected borrow records the slot identity
even when contents are `None`. Wholesale parent replacement creates new slots
and invalidates old borrows; only `rebind` obtains a new target. Named and
ordinal selectors use this same mechanism. See the canonical projection-slot
rules in `type-values-places-and-borrow-views.md` §7.

_See also: Let binding, BindingSlot, `@`._

---

## `@`

A continuation-relative name-reification operation that yields a lifetime
value:

```text
E@ = ReifyLife(NameOf(E), Pos(SemanticContinuation))
```

`@` is not a borrow constructor and never yields a borrow view or a `type ref`.
`ref` and `share` are the borrow constructors; each is a privileged
actual-place builtin (`PrivilegedActualPlace(ref-family)` /
`PrivilegedActualPlace(share-family)`) that may obtain the place of its actual
argument, while an ordinary user function cannot. `@` does not use that
privilege. A freshly computed temporary may receive a generated `LifeName`, so
absence of a carrier place is not an `@` failure.

`@` is **not** a general `PlaceOf(E)` defined on every expression. The former
two instance groups (`Val1?(x) ≠ null -> LifetimeFact`,
`Val1?(x) = null -> P ref`), the carrier-slot form `t@ : type ref`, and the
borrow-type fixed points (`type ref@ = type ref`, `type share@ = type share`)
are retired and do not return. Reaching the type-level place of a type-valued
binding explicitly uses `t |> (type ref)` or `(S ref).type`, not `@`.

`@` never projects a `Symbol` constructor value to its type member. Symbol supplies the ordinary
same-name family `S.type : type`, `(S ref).type : type ref`, and
`(S share).type : type share`. The borrow is formed before field projection, so
no source place is recovered from `AsType(S)`. `@` is not a stage name and not
an ordinary policy atom.

`@` is itself resolved by the ordinary selector. The three steps are strictly
ordered and non-circular: ordinary selection inside the operand, then ordinary
selection of `@` in the operand's policy stage, then lifetime validation, which
may reject the first two but never reselects them.

The lifetime core uses gapless half-open `Region=[i,j)`, generation-resetting
move, cleanup-before-lifetime ordering, Pre/Post call summaries,
pairwise-distinct exclusive-write and same-root shared-read defaults with finite
Pre patch, exact move-origin preservation, selected CopyConstruct lifecycle
posts, and an extensible Color vocabulary with finite/monotone relations in
each committed compilation semantic universe. Its concrete IR representation
and checker are not implemented. See
[`lifetime-policy-and-overload-boundary.md`](../design/lifetime/lifetime-policy-and-overload-boundary.md)
§1–§2.

_See also: Borrow view, Open authority, Lifetime Policy Boundary, `type ref`._

---

## LifeName, LifetimeValue, and NameView

`LifeName` is the stable semantic name of one value generation, including a
generated name for a temporary. `LifetimeValue` is the semantic result/value of
reifying that name at `Pos(SemanticContinuation)`. `NameView<T>` is the typed
observation that carries a `LifeName`, its half-open `Region=[i,j)`, and an
`origin` relation for a value of `T`; it is neither a borrow view nor a place
handle.

`LifetimeValue` is an ordinary first-class semantic value. It may be bound,
stored in an ordinary value slot, passed, returned, and moved/copied wherever
its ordinary Type, Policy, capability, lifecycle, and destination judgments
admit those operations. It is not an ephemeral observation-only expression
species. Transport preserves the value reified at its original continuation
position and does not silently execute `@` again. First-class status does not
imply runtime materializability: a same-Type static-to-runtime transition exists
iff the ordinary `RuntimeMaterializable(T)` callspace rule supplies a
non-deleted candidate. Compile-known and runtime-known observations do not form
distinct `LifetimeValue` species.

Every semantically name-projectable value operation has one unique name-level
companion, and the projections commute: `@(f_V(x)) = f_N(x@)`. Companion
existence is semantic; registry or IR representation remains
implementation-open.

Entry lifetime facts are `U_entry = U_default ⊕ Delta_pre`, where `Delta_pre`
is finite. Origin ancestry continues through the default generator of the
semantic construction that produced the name unless an explicit `origin=None`
terminates it; omission is not `None`. Exclusive-write borrow roots are
anonymous and pairwise distinct. All shared-read borrow roots default to the
same anonymous `G_shared`; a finite Pre patch changes only the relations it
states. The borrow name tree mirrors exactly the subname edges already admitted
by the established access relation. Structural membership alone grants neither
lifetime-name reachability nor access capability, and the lifetime default
synthesizes no alias-write/internal-mutability edge. The exclusive/shared root
rules specialize function-entry borrows; they do not make
ordinary non-borrow values origin-free by default.

Moving a nontrivial value uses one boundary `k`: the old first-level Region is
`[i,k)`, the successor Region is `[k,j)`, and the successor has exactly the
same deeper origin, `new@.origin = old@.origin`. There is no gap between
generations. Copy origin is not a lifetime-calculus theorem: the builtin/default
`CopyConstruct.lifecycle_post` states
`origin(result)=NameOf(source)`, while a custom candidate supplies its own
explicit lifecycle post through the ordinary Pre/Post boundary.
Whether generation identity and `LifeName` identity share one concrete
representation is implementation-open. Drop ends the current generation after
cleanup obligations. Color inheritance is monotone. The global Color vocabulary
is extensible, but every committed compilation semantic universe has one finite
Color-set snapshot with mechanically decidable compatibility, exclusion, and
exchange relations. A color contribution must define those finite relation
rows before the universe is committed; Color is not an arbitrary proposition
carrier.

_See also: `@`, Region, Lifetime Policy Boundary._

---

## Region

The half-open semantic-continuation interval `[i,j)` over one `LifeName`
generation. A move is one shared boundary `k`: the old generation is `[i,k)`
and the successor is `[k,j)`, with neither overlap nor an unowned gap. Uses must
lie inside the live region; move or drop closes the current generation. Cleanup
obligations are solved before lifetime closure.
Call summaries expose Pre obligations and Post facts: a failed Pre check makes
no state change, and lifetime failure never reopens ordinary overload
selection.

_See also: LifeName, LifetimeValue, Escape check._

---

## Borrow view

`ref` and `share` are an overloaded callable/operator family with two member
phases: the **type-forming** member is a meta member producing the borrow
TypeValue (`T : U_n ⊢ T |> ref = RefTy(T)`); the **borrow-forming** member
inside the formed borrow type's callspace is a runtime || compile default
member and is the family member that may obtain the privileged actual place
(`PrivilegedActualPlace(ref-family)` / `PrivilegedActualPlace(share-family)`).
There is no global `E ref = Ref(Read(E))` law: the selected overload determines
the result, and the builtin default may acquire `PrivilegedActualPlace(actual)`
(canonical owner `../design/symbol-world/type-values-places-and-borrow-views.md`
§5.1). For `let t: type = uint8`, the expression `t ref` selects the
type-forming overload and yields the TypeValue `uint8 ref` — a borrow-type
formation, not a borrow instance and not a borrow of the binding slot.
Reaching the type-level place of the type-valued binding uses
`t |> (type ref)`, which is borrow formation and yields a value `r : type ref`
with `Target = place(t)`. The two expressions do not produce the same value:
`t ref` is the TypeValue `uint8 ref`, while `t |> (type ref)` is a `type ref`
borrow instance whose target slot currently holds `uint8`. Their semantic
category differs:

```text
t ref            = borrow-type formation = TypeValue(uint8 ref)
                   // type-forming overload, not a borrow instance

t |> (type ref)  = borrow formation = value r : type ref
                   // Target = place(t); the value is the borrow instance,
                   //   not the TypeValue/string `uint8 ref`
```

Neither uses `@` (`@` yields a lifetime value, not a borrow). Whether `ref` or
`t |> (type ref)` is the right operation is decided by what the surface means,
never by type-rank: for `s : symbol` the payload exists, so `s ref : symbol ref`
borrows the `Symbol` value `s` carries: `Target(s ref) = PrivilegedActualPlace(s)`
— the borrow target has one source: `CarrierPlace(actual)`; there is no second
`ObjectPlace(Read(actual))`. A type-rank object with a payload behaves the same
way.

A borrow view is a value, not a second name for a `Symbol` constructor value: it does not forward
`SymbolId`, and its member set is not silently that of its target. It does carry
the stable semantic identity of its referent as value content:

```text
Norm(Borrow_k(q)) contains ⟨BorrowKind_k, StableTargetIdentity(q)⟩
```

That is why assignment, `rebind`, escape/Open-region checks, and compile
reference caching distinguish views of `q1` and `q2` even when the pointee
values are equal. The target is horizontal and remains a normalization leaf;
its current contents are not recursively owned by the view.

Applying a borrow operator to an existing view is **well-formed**, and that
overlapping overload is precisely what makes borrowing non-stacking:

```text
Borrow_k( Borrow_j(q) )     = Coerce_{j->k}( Borrow_j(q) )
Target( Coerce_{j->k}(v) )  = Target(v)
```

So `ref ref` and `share share` are idempotent (`Coerce` at equal capability is the
identity), and `ref share` is an admitted weakening to the same target — which is what
makes `r share` on a `type ref` legal. Borrow constructor composition never
retargets. Only
`share ref` has no candidate, because capability may be surrendered and never
regained. No composition nests, and no provenance or cycle detection is required.

`r_ref = v` writes the referent. `r_ref rebind = E` retargets the view, and it is
a place operation rather than a value borrow: it takes `Target(E)` when `E` is
already a view, or `CarrierPlace(E)` when `E` supplies one, and has no candidate
otherwise. It is deliberately not `E ref`, because for a pure `type` slot `t`
the expression `t ref` is the type-forming overload and yields the TypeValue
`uint8 ref`, not a borrow instance over the slot.
`OwnedClosure(x)` excludes every `ref` / `share` edge.

Borrow views replace the retired alias-forwarding model. Canonical owner:
`spec/design/symbol-world/type-values-places-and-borrow-views.md`.

_See also: `@`, Escape check, Alias binding (retired semantics), `type ref`._

---

## NoImplicitBorrowFormation

The negative invariant that candidate adaptation, structural repair, Policy
migration, and automatic argument passing cannot turn an ordinary
Object/Symbol/type actual into `ref` or `share` merely to make a candidate
applicable. The source or normalized expression must explicitly form the view
with `ref` or `share`; continuation-relative `@` yields a lifetime
value and never forms a borrow view. Fixed points and legal weakening of an
already formed borrow preserve its target and are not implicit formation;
callable-frame implicit `self` is a separate narrow capability rule.

Canonical owner:
`spec/design/symbol-world/type-values-places-and-borrow-views.md` §5.1.2.

_See also: Borrow view, Overload Candidate, `@`._

---

## Escape check

The check that a borrow view is not carried where it outlives its own valid
region:

```text
Escapes(view, destination)
  = Region(destination) ⊄ ValidRegion(view)
```

`ValidRegion` is determined by ordinary borrow lifetime and capability; it is
not a construction-open interval:

```text
ValidRegion( type ref )   =  BorrowLifetimeRegion( Target, ref )
ValidRegion( type share ) =  LifetimeRegion( Target )
```

A `type ref` may remain valid while its pointee type value's window is closed. A mutable
ref may then replace the whole slot with another legal value, but cannot use the
closed-window value as `extend` input. `OpenHere_Σ(v)` and `Writable(place)` are separate
judgments in both directions.

It applies to the destination classes that can outlive a valid region
(global/normalized structures, returned values, captured closure state, and
longer-lived member slots). It is a property of the destination and the view's
valid region only; it is not an RHS-provenance or construction-history check on
assignment.

_See also: Borrow view, `@`, Lifetime Policy Boundary, `type ref`._

---

## `extend`

The pure PatternValue transformation for structural extension:

```text
extend : type x StructLikeMaterial ⇀ type

old = tau_old = <Q_old,V_old>
Extend(old, Δ) ⇓ new = tau_new = <Q_new,V_new>
Root(tau_new) = Root(tau_old)
```

`extend` accepts the whole complete type closure, never a `type ref` or
`type share`. It checks `OpenHere_Σ(old)` from `Anchor(old)` and the
authority-frame resolution of §12.1.1, resolved through `Core(old)` for a
complete type value
(§12.1.2 of the canonical owner), and the result
independently satisfies `WellFormedTau(τ')` (history-free; never inherited
along a modification chain). It creates no root, modifies no place, and
preserves the input root. Failure is
total: no partial value, write, or rollback. Equal roots do not imply equal
closures or equal callspaces; older copies retain `V_old`.

_See also: `inject`, `OpenHere_Σ`, Open authority, `WellFormedTau`._

---

## `inject`

The place-level convenience operation over an existing `type ref`:

```text
inject : type ref x StructLikeMaterial ⇀ type ref

old = Clone(Read(t_ref))             // complete tau_old snapshot
new = Extend(old, Δ)                 // complete tau_new snapshot
Write(t_ref, new)
return t_ref
```

Legality is the conjunction of two independent checks:

```text
OpenHere_Σ(old)
Writable(Target(t_ref)) ∧ BorrowValid_Γ(t_ref)
```

Neither check proves the other. In particular, a `type ref` does not prove that
its current pointee is open. The final `Write` is ordinary slot replacement,
not a `τ -> τ'` construction transformation. Pure value code calls `extend`;
`inject` is only the read–extend–write wrapper.

_See also: `extend`, `OpenHere_Σ`, Open authority, Meta-function, Borrow view, `type ref`._

---

## `type ref`

`type ref` is the borrow-reference type produced by `type |> ref` through the
type-forming meta member of the `ref` family. The operand is the base universe
object itself (`U_0 = type`, classified by `U_1`), so the formed borrow type is
`RefTy(type) = RefTy(U_0)` — not `RefTy(U_1)`, which would conflate the operand
with its classifier. A value
`r : type ref` is a borrow view of a type-valued place. Reaching the
type-level place of a pure type slot uses `t |> (type ref)`; a `Symbol` constructor value uses
`(S ref).type` when `S` carries `τ` and `TypeValueRole(τ)` (equivalently
`TypeRole(Core(τ))`). Ordinary borrow
lifetime and policy rules determine formation, validity, and writability.

`type share` is the borrow-reference type produced by `type |> share`; a value
`s : type share` is a share view of a type-valued place.

```text
let t: type = uint8
let r : type ref = t |> (type ref)
```

A `type ref` is a type value; the borrow instance it carries holds only
ordinary borrow facts, never a second type-value shape:

```text
⟨ TargetPlace, type, BorrowCapability, LifetimeRelation ⟩
```

It contains no construction-open witness. A closed-window type can therefore be read
through `type ref`; if the ref is writable, the whole slot can be replaced by an
independently legal type value. Using the current closed-window value as input to
`extend` still fails. Returning or storing the view is an ordinary borrow escape
question.

`type share` keeps observability but has no write capability and therefore no
applicable `inject` operation. The weakening `r share` is the admitted
`ref share` composition.

```text
ValidRegion( type ref )   =  BorrowLifetimeRegion( Target, ref )
ValidRegion( type share ) =  LifetimeRegion( Target )
```

Borrow type constructors are universe fixed points:
`rank(type ref/share) = rank(type)`, `type ref ref = type ref`,
`type share share = type share`,
`type ref rebind rebind = type ref rebind`, and
`type share rebind rebind = type share rebind`. The former `@` fixed points
(`type ref@ = type ref`, `type share@ = type share`) are retired; `@` yields a
lifetime value uniformly and is not a borrow constructor.

_See also: `@`, `extend`, `inject`, Borrow view, Escape check, Open authority._

---

## PostfixOperator

A unary operator suffix that composes with other atom suffixes. In the
operator-aware design, postfix operators do not terminate suffix parsing, so
`obj!.field` has the same shape as `(obj!).field`.

_See also: OperatorSugar, Atom, NavPath._

---

## PrefixNegative

Parser-preserved prefix-negative surface syntax. The parser produces
`OperatorSugar { fixity: Prefix, operator: "-" }` for `-x`. It is not a
negative literal; the lexer produces `-` and the following literal or atom
separately.

Normalization rewrites prefix negative to typed-zero binary subtraction:

    -x  ⟶  ()zero::(x |> type) - x

Prefix negative is not an overloadable operator identity. The spelling `-`
as a declarable operator identity refers only to binary minus. Only the
generated binary `-` participates in operator-environment selection after
normalization.

_See also: OperatorSugar, Fixity, OperatorName._

---

## NavPath

A source-order inner-to-outer navigation chain separated by `::`.

```text
NavPath ::= NavComponent "::" NavOuterComponent ("::" NavOuterComponent)*
```

The leftmost component is the innermost selected symbol. The rightmost
component is the outermost scope component. Raw AST preserves navigation
components in source order and performs no lookup.

_See also: NavComponent, OperatorName, Atom._

---

## NavComponent

A component in a `NavPath`:

```text
NavComponent ::= Name | OperatorName | GroupedExpr | Error
```

Operator names are valid only as innermost navigation components unless a
future design explicitly allows operator-named scopes. Parenthesized
right-side scope expressions after `::` are preserved as grouped components.
A grouped expression is valid only as an outer component; used as the innermost
component (`(int Vec::std)::ns`) it emits `InvalidNavComponent`. Without
parentheses, `::` consumes only one immediate valid component.

_See also: NavPath, SelectorAst, OperatorName._

---

## EntityRef

A compile-time entity reference syntax. Phase 4.2 defines the design; Phase
4.4 implements a raw `EntityRef` parser inside alias-let RHS only. `EntityRef`
is not a runtime expression, not a `PipeExpr`, not a product form, not a
closure, and not resolved by the parser. EntityRef parsing is not a general
expression parser mode.

Provisional grammar:

```text
EntityRef ::= EntityComponent ("::" EntityOuterComponent)*
```

In the current implementation, `EntityRef` appears only on the right-hand side
of `let binder === EntityRef`. Other possible strong contexts are future work.

_See also: NavPath, NavComponent._

---

## Compile-time entity reference

The conceptual role of `EntityRef`: a source-level reference to a compile-time
entity that may later be resolved by semantic/name-resolution phases. It does
not denote a runtime value and is not checked for existence by the parser.

_See also: EntityRef, NavPath._

---

## EntityRef navigation

The navigation syntax inside a future `EntityRef`:

```text
EntityComponent ::= Name | OperatorName
EntityOuterComponent ::= Name | Group
```

EntityRef navigation is inner-to-outer and preserves source-order components.
An operator name is allowed only as the innermost component unless a future
design explicitly allows operator-named scopes. A grouped expression is valid
only as an outer navigation component after `::`; a grouped expression used as
the innermost component (`(int Vec::std)::ns`) emits `InvalidEntityRef`. The
parser does not perform operator lookup, name lookup, namespace resolution, or
existence checking.

_See also: NavPath, OperatorName, EntityRef._

---

## Alias binding

The frozen *parser* declaration form `let binder === EntityRef`. Phase 4.4
implements raw parser preservation: the parser produces `LetAliasAst` with
`AliasBinderAst` and `EntityRefAst`. Alias binding is not runtime value binding,
not an expression, not equality, not operator syntax, and not package import
syntax. No target resolution, operator identity validation, or entity lookup is
performed.

> **Semantic direction: retired.** The semantic reading of this form — a
> compile-time lookup alias that forwards symbol identity, place, or writability
> to a target — is retired, not deferred. There is no declaration form that
> forwards a `Symbol` constructor value or a place. `let a = b` creates a fresh binding in a fresh
> place carrying `b`'s value (`SymbolId(a) ≠ SymbolId(b)`,
> `PlaceId(a) ≠ PlaceId(b)`, `Value(a) = Value(b)`). Shared observation of another
> object is expressed only by a borrow view (`ref` / `share`); to reify its
> continuation-relative lifetime name, apply `@` (yields `LifetimeValue`). No
> operator-name exception survives: operator environments are ordinary values
> under the global `operator` type, with lexical copy/shadow and Symbol algebra.
> See
> `spec/design/symbol-world/entity-alias-design.md` (retirement notice) and
> `spec/design/symbol-world/type-values-places-and-borrow-views.md`.

> **Distinction**: Alias binding is implemented as raw parser preservation
> only. It is not an ordinary `let name: annotation = expr`. It has no `=`
> value expression, no declaration annotation, no `guard`, and no `with`.
> EntityRef parsing is implemented only inside alias-let RHS.

_See also: Lexical alias, Entity alias, AliasBinder, Operator alias, EntityRef,
Borrow view._

---

## Lexical alias

**Retired semantic term.** It named a compile-time lookup name introduced by
alias binding into a lexical scope, shadowing previous bindings of the same name
without mutating the original entity. That forwarding-based scope/target model is
retired: no declaration form forwards lookup. Ordinary shadowing by a fresh `let`
binding covers the shadowing behavior; observing another object is a borrow view.
The `LetAliasAst` shape it described remains a frozen parser fact.

_See also: Alias binding, Entity alias, Borrow view._

---

## Entity alias

**Retired semantic term.** It named a lexical alias whose target is a
compile-time entity reference (`EntityRef`), binding a name or operator to a
compile-time entity path. Target resolution for this reading is retired, not
future work. `EntityRefAst` preservation in alias-let RHS remains a frozen parser
fact.

_See also: Alias binding, Lexical alias, EntityRef._

---

## AliasBinder

The binder position in a `let binder === EntityRef` form. It may be a
`Name` or `OperatorName`. The parser preserves the binder as raw AST syntax
without resolving the target entity.

_See also: Alias binding, EntityRef, `spec/history/v0.1/operator-design.md`._

---

## Non-associative operator

An operator class that cannot be chained without explicit grouping in the
operator-aware parser design. Comparison, equality, and equals-suffixed
operators are non-associative in the current Raw AST frontend, so `a < b < c`,
`a == b == c`, and `a += b += c` require grouping.

Semantic validity of grouped expressions remains outside parser scope.

_See also: OperatorSugar._

---

## ClosureAST

The AST representation of a closure literal before materialization into a
callable object:

```text
ClosureAst {
  placement: InPlace | Ordinary,
  head: Option<FnHeadPrefixAst>,
  body: ClosureBodyAst
}
```

Placement and head presence are orthogonal. Bare `{ ... }` is headless
in-place; a headed block without `=>` remains in-place; `=>` selects ordinary
placement.

> **Distinction**: `ClosureAST` is **not** `ClosureObject`. Closure literals
> produce AST first. A later semantic pass may materialize closure AST into
> callable objects.

> **Distinction**: Bare `{ ... }` in atom position is an in-place `ClosureAst`,
> not a normal block expression.

_See also: ClosurePlacement, InPlaceClosureAST, OrdinaryClosureAST,
ClosureObject, Materialization._

---

## ClosurePlacement

The independent closure dimension `InPlace | Ordinary`. A no-`=>` body is
in-place even when it has a head or `[[strategy]]`; `=>` selects ordinary
placement. Placement is not inferred from `head.is_some()`.

_See also: ClosureAST, InPlaceClosureAST, OrdinaryClosureAST._

---

## Capture Clause

The ordinary-closure head component `[CaptureItem, ...]`. Each item is a
let-shaped binding: `[let x = E]` and `[x = E]` are explicit forms, while
`[E]` is shorthand only when normalized `E` has exactly one distinct free bare
name occurrence that is not a direct callable target. Policy-bearing captures
retain `let` to anchor the binding policy.

Capture initializers are simultaneous: every initializer sees the enclosing
environment before the clause. Normalization removes the explicit/inferred
surface distinction and produces `NormCapture { slot, initializer, origin }`.
This does not perform name resolution, environment layout, or closure
materialization.

Every source-written item is an explicit capture requirement. `[x]` is
shorthand for `[let x = x]` with the ordinary unwritten `plain` capture mode,
not an automatic const capture. A future resolved stage may add a separate
`ImplicitEligible` requirement for an otherwise uncaptured free outer value
reference. Capture requirements are abstract dependencies: they carry
requested Policy and required access capability into later ordinary checking,
but do not declare `self` fields, copy/reference representation, layout, ZST
status, or ABI.

External explicit navigation reaches the stable namespace export view; internal
explicit navigation reaches the complete namespace view. A capture's access
capability does not filter namespace membership and is checked later together
with its Policy demand. External access does not rewrite the slot's PolicyMode.
Automatic capture and call resolution share the Symbol-identity/stable-view
problem domain; this does
not imply pass ordering or an implementation dependency.

An explicit capture and an automatic capture may name the same source but
remain distinct dependency declarations. Explicit capture can rename, project
policy, use a complex initializer, request `mut`, and preserve provenance.
Only a later layout pass may coalesce equivalent storage/link requirements.

_See also: BindingSlot, NormClosure, Materialization._

---

## InPlaceClosureAST

A `ClosureAst` whose placement is `InPlace`. It may be the bare, headless
`{ ... }` form or a headed no-`=>` block, optionally with `[[strategy]]`.
In-place closures never have capture lists or independent capture
environments. Having no extraction head is not the same as having a unit
extraction pattern: a headless in-place closure accepts no extracted input,
including no implicit unit input.

In future callable materialization it may contribute an overload candidate
while remaining tied to its embedding control-flow layer. Unresolved outer
reads are resolved lazily at that layer; no capture list is required or
allowed. Direct writes to a place outside the closure-local scope are
forbidden; local mutation and effectful calls remain possible. An
otherwise tied in-place candidate is preferred after the
first-order-over-instantiated filter.

> **Explicit self position for return:** A headless in-place closure still has
> a callable owner, a callable-local `Self` space, and an invocation-frame
> caller/self slot, but it has no written binder for that slot. If it is later
> materialized as a standalone function object, its receiver type defaults to
> the owner-derived anonymous callable type; that default is not part of
> return-target identity. The headless form therefore cannot name its own
> return target through a first-formal binder. Early-return examples that
> target a specific closure should use an in-place closure with an explicit
> product/extraction head carrying the self position, e.g.:
>
> ```lang
> (<Self: type> self: Self) {
>   () |> (Self return);
> }
> ```
>
> `Self` and `self` are replaceable positional binders, not reserved
> names. The same positional structure with different names:
>
> ```lang
> (<R: type> this: R) {
>   () |> (R return);
> }
> ```
>
> The return target is not the spelling `Self`; it is the target
> syntax in the explicit target position, resolved later by
> semantic target binding.
>
> The example fragment above is a headed in-place closure. The same shape is
> accepted as a standalone expression atom or in an incoming pipe/branch form;
> its placement remains in-place in either context.

_See also: ClosureAST, OrdinaryClosureAST._

---

## OrdinaryClosureAST

A closure literal whose placement is `Ordinary`, selected by `=>`. It has an
explicit head and a callable implementation tail.
The head may contain deduce list, capture clause, parameter clause, call-result
policy clause, return clause, and head clauses. The tail preserves ordinary or
named user body, compiler-defaulted implementation, or deleted implementation.
As for every callable placement, the first written formal Pattern denotes the
implicitly passed caller-object self slot; only later written formals consume
the explicit call-site Product.
Plain no-`=>` block tails and `[[name]]` stay in-place; the latter is only the
named-strategy escape that does not steal the established return
extraction-pattern parse.

_See also: ClosureAST, InPlaceClosureAST, FnHeadPrefix._

---

## NormClosure

The normalized closure carrier. It stores
`placement: NormClosurePlacement`, optional normalized head, implementation
body, and `NormOrigin` independently. `NormClosurePlacement` is only
`InPlace | Ordinary`; generated lowering provenance belongs to
`NormOrigin::Generated`, never to placement.

_See also: ClosurePlacement, Origin, Dot Closure._

---

## Callable Implementation Tail

The single syntax slot that describes a callable implementation and optional
overload strategy. It normalizes to `UserBody(Ordinary|Named, body)`,
`Defaulted`, or `Deleted(message?)`. `=> name {}` and `[[name]] {}` carry the
same named strategy. Strategy metadata participates only after full
admissibility and never creates a second overload pass. Product/closure
classification and capture-slot bypass require the complete `[[Name]] {`
shape; Deduce alone leaves capture parsing available, and the weaker `[[`
prefix is recovery-only after a later head component has proved the strong
context.

_See also: OrdinaryClosureAST, Fully Admissible Candidate, Overload Resolution Pipeline._

---

## PatternValidatedNormProgram

The downstream handoff produced by `normalize_and_validate_patterns` after all
currently enforced global normalized Pattern invariants have passed: one Pack
per structural level, no bare Product Pack operand, and no duplicate
DeduceList hole in one `PatternRoot`. Its certificate is intentionally
narrow: it does not prove ordered/unordered Pack applicability, stable
Pattern-head identity, complete matching support, parser-diagnostic absence,
or recovery freedom. `normalize_program` alone remains useful for
diagnostic/recovery dumps but does not authorize build-world harvesting.

_See also: Normalized AST, Pack Pattern, Raw AST Contract v0.5._

---

## Dot Closure

The first-class expression `.name`, normalized to a generated in-place
`NormClosure` carrier shaped as
`(self, val: T, ...args) { (val, args) |> name::T }`. The generated first
formal is the implicitly supplied callable object; `val` is the first explicit
call-site argument. `E.name` is compact
`E |> .name`; `.name` itself captures no receiver. After lowering it is an
ordinary expression. Replacing it with a bound equivalent must preserve the
same pipe/product binding spine, and no normalizer rule may inspect
`DotClosureLowering` provenance to absorb surrounding syntax. Only explicit
binding or call context materializes the carrier as a value; normalization and
other expression contexts do not. `..name` remains direct member-call sugar.

_See also: Atom, Function Object, Call normalization._

---

## Pack Pattern

The Pattern-side remainder form `...Q`. It matches the unmatched portion of
one normalized structural level and then applies `Q`. Each level permits one
pack; nested levels are independent. It is not a value/type/ABI category and
has no RHS unpack counterpart. Every Pack contributes one outward specificity
node, independent of captured length and internal node count.

At an unordered named layer only a whole-remainder binder/discard (including a
transparent let-shaped slot) is admissible. At an ordered layer a structured
operand may be meaningful only if its P-normal form retains a stable top mode,
for example `...((a, b) pair)`. Raw `...(a, b)` is preserved by the parser but
rejected after P normalization: Pack cannot reify the bare Product boundary
that ordinary Product normalization removes. Internal evidence below a stable
operand head belongs to the next preserved level; it is never flattened into
multiple same-level EP nodes.

Pack is valid syntax in every let-shaped binding slot, including ordinary/local
let, parameter, return, and nested product extraction; it is not a
parameter-only variadic form. It may be a direct canonical Pattern Sequence
child: `a ...x b` normalizes as `Sequence[a, Pack(x), b]`. Ellipsis consumes one
following Pattern primary. Only Product and Sequence establish cardinality
levels; Pack and BindingSlot are transparent. The parser preserves all formed
Pack nodes, and the normalized Pattern validator is the sole authority for
cardinality and the bare-Product rejection.

_See also: Pattern normalization, Overload Specificity._

---

## ClosureObject

A materialized, callable object produced from a `ClosureAST` by a future
semantic pass. In v0.1, closure objects do not exist. The parser produces
only closure AST.

> **Distinction**: `ClosureObject` is a semantic concept, not a parser concept.
> Materialization is explicitly out of scope for v0.1.

_See also: ClosureAST, Materialization._

---

## Materialization

The future semantic pass that converts `ClosureAST` into a `ClosureObject`.
Before this pass, resolved capture dependencies must first become
lifetime-checkable source/access/storage-or-link forms. Materialization may
then select static links, constant embedding, zero-layout dependencies, stack
environments, stored checked references, or other future representations. A
capture list is not itself an environment-field declaration. This is not
implemented in v0.1.

_See also: ClosureAST, ClosureObject._

---

## MetaInstance / MetaPartner / GeneratedClosure

The virtual construction scope established by a canonical meta invocation is
`M = MetaInstanceScope(callee_symbol, canonical_arguments)` — the
`MetaInstanceRoot`: a symbolic-navigation and construction-authority anchor,
not a `symbol`-typed result value (the default result is `τ_M` rooted at `M`;
`ShapeOfTypeSymbol` applies only to explicit `symbol` results). Generic
construction anchors through the callee's declared meta partner:

```text
MetaInstance(MetaPartner(F), Norm(GenericArgs))
```

Generic arguments key this scope; ordinary invocation arguments do not
participate in generic meta-instance identity. Evaluating the anchored
instance yields `GeneratedClosure(F, GenericArgs)` (written `C_F,A...`), whose
type is rooted at that scope: `ClassifierRoot(TypeOf(C_F,A...)) = M`. A
generated meta partner returns a callable closure, so
`GeneratedMetaPartner ≠ OrdinaryMetaFunction`: the ordinary-meta
`DefaultMetaResult = τ` rule does not apply to generated closures.
Canonical owner:
[`meta-object-invocation-and-policy-reduction.md`](../design/meta-invocation/meta-object-invocation-and-policy-reduction.md)
§4.4.

_See also: Meta-function, CompilePartner._

---

## Meta-function

A callable whose entry executes under meta construction capability. Callable
kind fixes `P2=meta` and `DefaultMetaResult = τ`; a particular call is
well-formed only after ordinary admissibility and `GlobalKeyable` checks. A
successful call establishes root identity and construction navigation, not
external installation. An explicit `f : … -> symbol` is still legal.
The canonical judgments and seal algorithm belong to
[`symbol-first-meta-construction-and-pattern-injection.md`](../design/symbol-world/symbol-first-meta-construction-and-pattern-injection.md).

At seal only the returned result's `OwnedResultClosure(τ)` may be promoted.
For the default result `τ_M`, `OwnedResultClosure(τ_M)` =
`OwnedClosure(Core(τ_M))` plus `OwnedCallSpaceClosure(CallSpace(τ_M))`, and
`Core(τ_M)` is the first projection of `τ_M` (always present). For an
explicitly `symbol`-typed result, the carried `τ`'s owned result closure is
promoted only when `τ` is present. The core may be namespace-only
(`NamespaceRole(Core(τ))` and not
`HasRegisteredSelfConstruction(Core(τ))`; `../design/patterns-overload/pattern-values-relational-semantics-and-extraction.md` §13); type capability is
the additional `TypeRole(Core(τ))` refinement. The registered-self-construction
witness requires an actual `Val2` member: `Val2(Q)[s] = K` together with
`ConstructEdge_P_Q(C, Q, K)` for the same `K`.
`EscapeDeps(τ)` checks the complete returned result at the τ level
(`Core(τ)` ∪ `CallSpace(τ)`) plus horizontal borrow targets; it is not a
val-sibling-only check.

Compiler-defined `BuiltinPrivilegedAstMetaFunction` objects are a separate
subclass. A member such as `struct`, `extend`, or `inject` may accept one
specifically bounded Normalized-AST/pattern carrier and use a member-specific
scope/owner rule. `struct` forms a complete type value `tau` directly
(`struct: StructLikePattern -> tau`; the core `Q_struct = Core(tau_struct)`
is produced during the formation event) and establishes or selects its
`Q_struct` type-role member's stable lexical root from input navigation plus
ambient scope; the `Symbol` constructor value carrying the formed value appears only at a
subsequent binding/install. `extend` establishes no root and preserves the
input root; `inject` is
its place-level read–extend–write wrapper. Any later privileged member must declare its own owner rule. Users
may call these objects but cannot define new privileged members; the privilege
does not imply text substitution, parser re-entry, or a general macro system.

> **Distinction**: meta execution capability is not AST privilege. Names such
> as `match` and `struct` remain ordinary parser-level names; parser code does
> not special-case them.

_See also: Name, Strong context._

---

## Declaration

A user-visible binding introduced by `let`. In v0.1, all declarations enter
through `let`. There is no separate `fn`, `type`, or `namespace`
declaration syntax. Declarations use a binding slot whose annotation, when
present, is parsed and preserved but not semantically checked.

_See also: Let binding, BindingSlot, BindingAnnotation._

---

## Let binding

A top-level `let` form that introduces a binding slot. A let binding may bind
a simple binder name or a canonical skeleton pattern, may carry a per-slot
deduce list, may carry an optional binding annotation, may carry `with { ... }`,
and is followed by `=` and an initializer expression.
Let bindings are the only declaration path in v0.1.

At the semantic layer, ordinary `let LHS = RHS` has one uniform value-binding
rule for ordinary values, type values, and Pattern values:

```text
evaluate RHS -> v
allocate fresh LHS Symbol/Place
bind that exact v to the LHS Symbol
```

If RHS is a path, the path first resolves a `Symbol` constructor value and reads `v`; the RHS
carrier Symbol is not part of `v` after that read. `let T: type = uint8`
therefore binds the existing type value under a fresh carrier. No declaration
form forwards a `Symbol` constructor value or a place; to observe another object's place, bind a
borrow view (`uint8 ref`, `uint8 share`); to reify its continuation-relative
lifetime name, apply `@` (yields `LifetimeValue`).

At the semantic layer this is also the only missing-member creation operation.
Navigation may yield a prospective ProjectionSlot containing `None`; `let` may
instantiate it. Bare `=` writes only an already existing place, and a return
event performs only control transfer. A return-name `let`/overwrite cluster in
the current evaluator is a transitional compatibility encoding, not this rule.

_See also: Declaration, BindingSlot, BindingAnnotation._

---

## BindingSlot

A parser-level binding-site shape reused by let bindings, closure parameters,
and closure returns. It preserves an optional `PolicySpec`, optional `let`,
optional `DeduceList`, a binding pattern, optional binding annotation, optional
`with { ... }`, and an optional initializer where the surrounding context allows
one.

The optional policy is recognized only in the strong policy position before
`let`. It is either one policy expression or an explicit pair separated by
`:`. Without the trailing `let`, the same tokens stay in the binding pattern /
canonical skeleton. `None` means unwritten and later inferred. The parser
preserves syntax and does not perform semantic pair validation.

_See also: Let binding, BindingAnnotation, CanonicalSkeleton._

---

## Source-Visible Global Implementation Space

The toolchain-owned source construction input installed at namespace root
`::`, abbreviated `Gsrc`. Its files pass through ordinary lexing, parsing,
normalization, declaration harvesting, semantic Symbol/Val2 construction, and
ordinary invocation. The typed build authority may use an empty install
prefix; ordinary project source roots may not.

`Gsrc` is source-visible namespace material, not a prelude. A project lookup
still follows ordinary path and public/private rules, and no member is injected
into lexical scope merely because it is installed at `::`. Physical bundle
paths organize build input but do not determine Symbol or Pattern identity.

_See also: Namespace Symbol Views, Toolchain Global Construction Authority._

---

## Toolchain Global Construction Authority

The typed build fact authorizing a toolchain-owned source bundle to contribute
direct members to `::`. Global visibility and global construction authority
are different: ordinary source may resolve a public global path but cannot
obtain root-construction authority from an empty directory, empty mount
prefix, or missing navigation component.

_See also: Source-Visible Global Implementation Space._

---

## Namespace Symbol Views

Three independent sets govern namespace and build-world reasoning:

```text
Σ_full(N)    complete namespace-internal symbol and overload set
Σ_export(N)  externally exposed projection of Σ_full(N)
Wfinal       Wpre ∪ Wseal, materialized/retained/generated build world
```

Internal explicit resolution searches `Σ_full`; external explicit resolution
searches `Σ_export`; world membership asks whether a binding exists in Wpre or
Wseal. The export overload set preserves the same candidate identities as the
full set, but every external candidate preserves its resolved `PolicyPair` and
whole-slot `PolicyMode` rather than carrying a declaration-side `P1Projection`.
External admission requires both
export-retention-closure membership and public reachability through the full
path.
Within each admitted full overload set, every resolved candidate enters the
stable external view with the same identity, pair, mode, declaration/intrinsic
realization facts, and provenance. It carries no context-indexed dynamic
legality judgment. A later concrete consumer applies ordinary Policy and
capability-realization selection, then forms `DynamicLegality_Γ_consumer` for
the selected invocation without changing namespace membership.
Publicly reachable export-retention-closure ancestors and descendants receive this
projection even when they are not export roots. World membership does not
imply export, and export does not imply that the binding itself was an export
root. Retention-closure membership is graph/interface-construction input, not
synonymous with membership in `Σ_export`.

_See also: Policy Pair, Namespace (source name)._

---

## Policy Pair

The canonical internal policy representation:

```text
Π = Pv:Pp
```

`Pv` describes the `Val1`/value component; `Pp` describes its carried
Pattern/anonymous-type component. `PolicyMode={const,plain,mut}` is a separate
whole-slot coordinate; stage, value presence, ordinary namespace visibility,
export-root, and capability realization are further typed orthogonal
dimensions. A scalar policy is surface shorthand or a derived summary and
cannot reconstruct the pair plus mode.
Surface elaboration factors at most one connected whole-slot `ModePattern`
before elaborating the residual `Pv:Pp`; mode atoms cannot independently occupy
the value or Pattern side of `:`. A typed `ModePattern` is exactly one
`ModeAtom ::= const | plain | mut`; a surface `PolicyChoice` containing multiple
ModeAtoms is rejected during typed Policy elaboration. `plain` is the only
neutral/default point and is not `const || mut`. This restriction leaves
pair/view choices such as `compile || runtime` unchanged. In result-demand
context, absence of a written ModeAtom independently defaults to `plain` and
does not erase that residual pair/view choice.
Whether a colon spelling with an empty residual side is
rejected or contextually completed is still a surface question; the current
rejection examples are provisional rather than a consequence of orthogonality.
Ordinary policy notation does not use `@`, which remains reserved for lifetime
policy syntax.

The positive presence invariant is:

```text
Val1 = absent  =>  Pv = Pp
Pv != Pp       =>  Val1 != absent ∧ runtime ∈ Stage(Pv)
```

This does not say `Pv = absent`: the PatternValue itself still has policy. An
observer may hide an existing `Val1`, so `Pv = absent` does not prove physical
absence.

There is no independent `P3`. Written parameters inherit `P2`, and returns
inherit `P1`; either position may refine only the inherited `PolicyMode`.
Stage, presence, visibility, and every other policy dimension remain invariant.

At namespace direct top level, `export` derives an external view without
cropping the complete internal `Pv:Pp` or rewriting `PolicyMode`.
That stable view depends on export retention and public path reachability, not a
future consumer's capability or Policy demand. A pure `absent:Pp` slot still
has a whole-slot mode; absence removes
value stages and `SemanticValueId`, not the mode coordinate. The former
universal `Project_const` export rule is retired semantic vocabulary; any
remaining const-projected adapter is an explicitly bounded implementation
subset.

_See also: PolicyBinding,
`spec/design/symbol-world/symbol-policy-and-compile-flow-projection.md`._

---

## Policy Binding

The future P1 projection judgment for a binding:

```text
[P1] let x = expr
```

Before resolving/evaluating the RHS, binding spelling forms the output-mode
selection preference: bare `let` gives concrete `plain`, while `const let` and
`mut let` give singleton `const` and `mut`. That producer-selection coordinate participates with input
Policy coordinates in ordinary call overload selection. Only after the unique
RHS result has been selected does omitted P1 keep its fully inferred pair view
or an explicit P1 apply the existing pair-view projection/migration machinery.
The selected producer retains its declared concrete `ResultPolicyMode`; the
destination has its own mode from binding elaboration. Mechanical move/copy
transfers the value between those slots without relabeling the producer result.
Thus a `const` producer may win under plain preference while the destination
remains `plain`. The destination never inherits the RHS slot's mode, and the
RHS mode is never rewritten to the destination mode.
Independently, current-phase evaluation derives each candidate's implicit P1
stage view from its P2. Therefore an omitted pair/stage demand does not make
`compile`/`runtime` unknown and does not require `PolicyLet`; omitted binding
P1 still retains the selected RHS's complete pair view.
A single `Q` selects values visible
under `Q` and retains each value's associated Pattern component. An explicit
`Qv:Qp` filters both components. Therefore single P1 `Q` is not pair `Q:Q`.
The selected slice must be non-empty and admitted by the destination binding.
Projection crops the policy slice while preserving symbol and Pattern identity;
it does not return an unchanged entry after a mere intersection check.

The bounded transition prototype does not change this rule. Any non-empty
projection completes binding elaboration; alternatives written in the query but
absent from the RHS are not obligations to manufacture values. More generally,
an existing compatible Policy view dominates migration: successful projection
preserves the existing Symbol, TypeValue, PatternValue, Place, and value
identity and makes migration semantically unreachable.

There is no general prohibition on runtime bindings:

```text
runtime let x = runtime_value
```

is legal when the runtime value slice exists. A `Psrc != runtime` premise may
belong to one compile-flow projection rule, but never to general let lowering.
In P2 context, unlike P1, a single policy is normalized into a result pair; in
particular current `runtime` means `runtime:compile`; explicit `runtime:seal`
remains valid.

_See also: BindingSlot, PolicyPair,
Policy Transition,
`spec/design/symbol-world/symbol-policy-and-compile-flow-projection.md`._

---

## Policy Demand Satisfaction

The act of satisfying a consumer's requested Policy view through the single
Policy migration algebra shared by binding, `PolicyLet`, and atomic runtime
materialization. Demand kind records candidate admission facts; it does not
grant permission to search arbitrary conversion operations. Every selected
migration candidate owns declared source/target Policy endpoints and jointly
produces `PolicyProjection` plus `ValueRealization`, which must be coherent.
The ordering is called **Existing-First, Constructible-Second**:

```text
existing compatible view
  -> candidate set = {identity Policy migration}
  -> PolicyProjection preserves the slice
  -> ValueRealization preserves the value

complete existing projection is empty
  + runtime is an accepted alternative
  + eligible static Val1 view
  -> extract RuntimeBranch(query)
  -> admit authorized direct Policy migration candidates
  -> ordinary Policy overload / unique selection
  -> selected migration jointly supplies PolicyProjection and ValueRealization

otherwise
  -> inadmissible or governed by another explicit language mechanism
```

A consumer accepting `compile || runtime` is satisfied by an available compile
slice. Merely mentioning runtime in a choice does not force materialization.
However, if a complete choice such as `meta || runtime` has no existing
accepted view, runtime is the currently language-constructible accepted stage
branch. Failure of Type/Pattern structural applicability cannot be repaired by
Policy migration.

For an object already carrying `(compile || runtime):compile`, the runtime
branch is an existing Policy slice rather than a migration request.
`ExposePolicySlice(runtime)` may therefore succeed during a static phase while
`ReadValue(runtime)` remains unavailable. Compile-readable dependencies are
bound/evaluated statically; runtime-dependent computation is residualized and
continues the same already-resolved invocation without reopening Symbol lookup
or overload selection. Residual representation, effect sequencing, and
continuation ABI remain open.

_See also: Policy Binding, Policy Pair, Policy Transition._

---

## Default Evaluation Result Context

The result context present for every call even when no explicit result Policy
is written. It has two orthogonal total inputs and one optional input:

```text
EvaluationStageContext(call) = current phase kappa
OutputModeDemand(call)       = immediate explicit demand | plain
TargetResultConstraint(call) = optional explicit pair/type/rank/facet facts
```

For candidate `f`, the evaluation-stage component derives an implicit P1 stage
view from `P2(f)` using
`Stage(P1p)=Stage(P2p)` and
`Stage(P1v)=Stage(P2v) union Stage(P2p)`, then checks that view in `kappa`.
This is the default **P1-stage-follows-P2** behavior. It makes ordinary
`compile`/`runtime` evaluation phase-directed without requiring
`compile let e` or `runtime let e`.

The derivation is stage-only. It does not copy `PolicyMode`, visibility,
capability, export status, or value presence from P2. Unwritten whole-slot mode
is the concrete point `plain`; selecting `const` or `mut` is a separate manual
result demand. An explicit `PolicyLet` may still delimit, narrow, or request a
stage/migration result, or supply that non-plain Mode demand.

_See also: CallLocalPolicyClosure, PolicyLet, PolicyMode, Policy Pair._

---

## CallLocalPolicyClosure

The rule that closes every call node before an unresolved outer candidate may
influence it. A call's output-mode preference is formed before its own candidate
maxima. It may use an already-formed, candidate-independent immediate-consumer
demand; when none exists, the local default is `plain`. After unique selection,
the call's concrete `ResultPolicyMode` is frozen. An outer call consumes that
result as an ordinary actual and never reopens the inner call. Thus an outer
formal PolicyMode Pattern cannot be assumed to select an inner call and then be
used to decide the outer candidate; no cross-call fixed point exists.

This `OutputModeDemand(call)` exists for every call and always participates in
the PolicyMode product. The current phase separately supplies the
P1-stage-follow-P2 default. Optional expected result pair/type/rank/facet facts
are a separate `TargetResultConstraint(call)` used by hard admissibility only
when the context supplies them.

_See also: Default Evaluation Result Context, Policy Binding, PolicyMode,
OverloadResolutionPipeline._

---

## PolicyLet

The expression-level term former:

```text
PolicySpec let PipeExpression
```

It has one preserved syntax/Normalized-AST boundary and two semantic
projections. The inward projection forms `ResultPolicyDemand(PolicySpec)`
before the operand root call's ordinary overload maxima. The outward
projection applies ordinary Policy-demand satisfaction and exposes one
completed concrete Policy view through the node's ordinary expression-result
slot. The slot is not a NameBinding, Symbol, hidden declaration, or
independently addressable Place. Parentheses close the boundary; an outer call
consumes that view as an ordinary actual and cannot reopen the operand call.

This is an explicit override/boundary, not the source of the ordinary
evaluation phase. Without PolicyLet, `compile`/`runtime` stage exposure already
comes from the current phase and the candidate's P2-derived P1 stage view.
`compile let e` / `runtime let e` remain available for explicit stage
delimitation or migration; `const let e` / `mut let e` manually replace the
otherwise concrete `plain` Mode demand.

`PolicyLet` creates no hidden binding, place, or declaration. It is not an
ordinary Val2 `const`/`mut` call and is not an in-place Policy tag rewrite.
After the operand is frozen, `SourcePolicy(result) -> P` enters the same Policy
migration candidate preparation and ordinary Policy overload used by binding
and runtime materialization. The unique selected candidate jointly supplies
the outward `PolicyProjection` and its concrete `ValueRealization`; their
coherence completes the result. A Type-callspace/Val2 operation or canonical
mechanical action may implement that value realization, but its body neither
declares the migration's Policy endpoints nor creates the preceding inward
demand. The parser and Normalized AST preserve this node; the current build
prototype does not yet execute its result-demand/migration semantics.

For singleton `plain`, a `const` producer may win inward selection under
`succ_plain` while retaining `ProducedMode=const`; this is not already an exact
plain outward view. Canonical terminal move/copy transfers the value into the
PolicyLet's ordinary plain expression-result slot. The source mode remains
const, the result-slot mode is plain, and no global `val plain` or Policy tag
rewrite is introduced. The transfer is the selected migration candidate's
`ValueRealization`, not an independent companion relation. Missing or ambiguous
migration is a typed outward failure after producer selection, never permission
to expose the wrong mode or reopen candidates.

_See also: ResultPolicyDemand, Policy Demand Satisfaction,
Default Evaluation Result Context, CallLocalPolicyClosure, PolicyMode._

---

## Policy Transition

Policy transition uses the one migration algebra shared by ordinary binding,
`PolicyLet`, and runtime materialization:

```text
SourcePolicy -> TargetPolicy
  -> PreparePolicyMigrationCandidates
  -> ordinary Policy overload / unique selection
  -> selected m
  -> PolicyProjection(m) x ValueRealization(m)
  -> coherent completed result
```

The current canonical non-identity stage transition is the
language-authorized atomic Runtime Policy migration considered only after a
complete accepted Policy choice has no existing view and that choice contains
runtime. Define:

```text
S = Static(Pv) = Pv - runtime
```

For a legal selected input endpoint, `S` is non-empty and `S = Pp`. The
compiler-mandated endpoint skeleton is:

```text
input:  Type=T, value stage=S,       Pp=S
output: Type=T, value stage=runtime, Pp=S, presence=present
```

Input/output `PolicyMode` may differ because those coordinates belong to
the selected ordinary callable and its overload Policy. Thus
`const compile -> mut runtime` may construct a fresh runtime object; the
compiler authorizes the stage edge but does not invent `mut`. Pattern-side
Policy capability remains `S`. This does not mean the implementation copies
the source Pattern object. An eventual ordinary function-object invocation
supplies an ordinary result whose Type/Pattern/owner coherence is governed by
existing invocation and Pattern semantics.

Migration endpoint `PolicyMode` uses ordinary actual-relative Bp preference, not
hard Policy-domain intersection or subset specificity:

```text
succ_const: const > plain > mut
succ_mut:   mut > plain > const
succ_plain: plain > const = mut
```

Opposite endpoint Patterns remain fully admissible. Stage, presence, Pp
capability, Type, and structural applicability remain hard constraints. The
plain row preserves ambiguity when only tied `const`/`mut` endpoints survive.
Generic ordinary members may realize any of the nine input/output mode
coordinates as absent/default/delete/custom; the current four-member 2×2
fixture is only an implementation subset. More specific Pattern members may
refine or delete regions of that relation.

The demand-preparation helper implements only the binding-P1 entry point. It projects
the complete original query first, then derives a runtime-only target branch:

```text
PolicyTransitionRequest {
  source_policy,
  target_query,
  source_type,
  source_value,
  provenance
}
```

The connected build slice consumes such a request through the source
`PatternValue`'s resolved owner and associated `()` Val2, then uses the same
`PreparedCallCandidate`, `InvocationFrame`, and ordinary result path as source
calls. Its Bp' dominance relation composes the implemented ordinary
formal/phase coordinates with optional input/output migration endpoints before
one maximal-element selection. Without those optional coordinates it reduces
to the connected ordinary order; a source regression preserves the older
restricted selector's winner identity.

The older caller-supplied candidate-ordering carrier remains algebra-only
fixture material. Its endpoint-only maxima helper is private and not
sequentially composable with ordinary Bp. Crossed advantages are ambiguous and
declaration order is irrelevant. Absent Val1 cannot construct the request.
Candidate output Type must equal source Type, so migration cannot search `ref`
or another structure-changing operation to repair applicability.

Input and output Policy slicing bracket the directed migration:

```text
Project_out o Migration o Project_in
```

No transitive migration graph, candidate backtracking, temporary-lifetime
extension, universal transition Symbol, or new callable ontology is implied.
PolicyLet does not introduce a second Policy selector; its demand kind only
restricts the direct migration candidates admitted to this same ordinary
selection.
Explicit mechanical `ref`, `share`, and `rebind` operations remain ordinary
function-object calls distinct from Policy-demand satisfaction.
Binding P1 is the currently connected demand consumer. Consumer-neutral
parameter/result demand preparation, complete Pattern/result construction,
backend/runtime materialization, and residual execution remain future work;
ordinary Symbol/Val2/associated-`()`/InvocationFrame routing itself is now
connected.

_See also: Policy Binding, Policy Pair,
`spec/contracts/v0.6-cross-policy-value-transition.md`._

---

## BindingAnnotation

The annotation following `:` in a `BindingSlot`. It preserves the written
annotation associated with a binding site. It has two explicit raw forms: a
single annotation expression, or a compound annotation with a preserved `:`
between the left annotation term and right annotation expression. v0.1 does
not determine whether the annotation denotes a value object, type object, rank
object, custom rank, concept, region, or future classifier. Parsed into
`BindingAnnotationAst::Expr` or `BindingAnnotationAst::Compound`.

> **Distinction**: `BindingAnnotation` is a parser-level construct, not a
> semantic type. v0.1 does not check that annotation names resolve to
> anything. A single-expression annotation is preserved exactly as written.

_See also: BindingSlot, AnnotationTerm, Type-object._

---

## Closed annotation

A resolved Pattern annotation `P` with `HoleRefs(P) = empty`. In
`let LHS : P = RHS` it is an ordinary result-as transformation target,
equivalent to `let LHS = RHS |> P`; it is not a Boolean compatibility check.
Hole references are resolved `HoleBinderId` identities, not spelling matches.

_See also: Deductive annotation, Hole, BindingAnnotation, Pattern._

---

## Deductive annotation

A resolved Pattern annotation whose `HoleRefs(P)` is non-empty. It participates
in deduction/extraction constraints for those exact hole identities. It shares
the ordinary Pattern/call calculus rather than introducing an annotation-only
compatibility primitive.

_See also: Closed annotation, Extraction, DeduceList._

---

## AnnotationTerm

The left side of a compound `BindingAnnotation`, before the second `:`. It can
be a preserved expression or a hole (`_`). In `let f: fn = ...`, there is no
compound annotation; the whole annotation is `BindingAnnotationAst::Expr`.

_See also: BindingAnnotation, AnnotationHole, Type-object._

---

## AnnotationHole

The token `_` used as an annotation-term placeholder. Appears in forms like
`let f: _: fn = ...`, where the left annotation term is anonymous and the
right annotation expression is preserved. Represented as
`AnnotationTermAst::Hole`.

> **Distinction**: `AnnotationHole` is an annotation-term placeholder, distinct
> from a canonical skeleton wildcard `_`.

_See also: AnnotationTerm, CanonicalSkeleton, Type-object._

---

## Abstract literal type

One of the source-language denotation types `integer`, `real`, or `character`.
An integer literal denotes an exact integer; a finite real literal denotes an
exact rational value (`0.1 = 1/10`, `0x1.8p1 = 3`). These denotations precede
machine-width, signedness, storage, and rounding choices. Their unique initial
Policy pair is `compile:compile`; parsing never directly creates a runtime
abstract-literal value. Character spelling is
still a separate surface amendment and is not added by this semantic closure.
These are ordinary complete type values satisfying the existing Type role, not
members of a separate `LiteralType` universe parallel to ordinary types.
Their ordinary same-Type compile-to-runtime materialization cells are
intrinsically `delete` in the immutable callspaces of `tau_integer`,
`tau_real`, and `tau_character`, so
`RuntimeMaterializable(integer|real|character)` is false. Canonical literal
values are typed by exactly those snapshots. Later associated contributions
form a new complete type snapshot instead of mutating an existing `V_tau`, and
`NoForeignTypeMemberInjection` prevents importing foreign members. No mechanism
beyond the ordinary snapshot and resolver rules is involved. The values must first be
constructed into another, concrete machine-semantic Type. Ranked string
literals follow a separate existing path directly to
`str@compile`; they are not `character` tokens and do not belong to these three
abstract scalar denotation Types.

_See also: Complete type closure (`tau`), Concrete machine-semantic type,
Convert/Construct._

---

## Atomic builtin type

An actual builtin Type value whose identity does not require applying a
dependent type constructor to another Type value. The current T key space is:

```text
uint | int | float | buffer | str
```

The Rust `AtomicBuiltinType` enum is a lookup key for these intended Type
symbols, not itself a `TypeValueId` and not merely a literal classifier.
Current core bootstrap does not yet install every member.

_See also: Concrete numeric type, Type-object, TypeValueId._

---

## Concrete numeric type

A width-bearing numeric Type (`Tnum`) such as `uint16` or `float32`. Numeric
literal construction may target such a Type; the literal does not initially
have that machine Type. Rounding occurs during ordinary construction to the
concrete target, never while parsing the abstract denotation. In the current
implementation, `NumericTypeKey` maps to a first-order `TypeValueId` projection
derived from an installed core Type symbol; final whole-snapshot type-value
identity is `Addr(Norm_type(tau))`, used where the language has frozen
whole-snapshot semantics, while ordinary type equality/keying keeps observing
`Core(tau)=Q` under the minimal-change rule.

_See also: Atomic builtin type, Literal, TypeValueId._

---

## Concrete machine-semantic type

A concrete Type whose value semantics are fixed across compile and runtime
stages, such as a width-bearing integer or floating Type. Stage controls when a
value is available; it may not redefine that Type's equality, range, rounding,
or representation-level value semantics.

_See also: Abstract literal type, StageInvariantTypeSemantics._

---

## Convert/Construct

Ordinary semantic operations that may change Type. Literal flow uses them to
construct a concrete machine-semantic value from an exact abstract literal.
They own range checks and rounding required by the target Type.

_See also: Migrate/Materialize, Abstract literal type._

---

## Migrate/Materialize

Operations that preserve Type while producing an eligible value at another
stage or in a concrete storage form. They may change value/place identity but
must satisfy `Type(output) = Type(input)`; they are not conversion or
construction to a different Type. The abstract denotation Types `integer`,
`real`, and `character` have deleted compile-to-runtime cells and therefore do
not admit this same-Type runtime operation.

_See also: Convert/Construct, StageInvariantTypeSemantics._

---

## StageInvariantTypeSemantics

The invariant that one concrete machine-semantic Type has the same value
semantics at compile and runtime stages. A stage transition may materialize or
migrate a same-Type value, but it cannot switch the Type's range, rounding,
equality, or interpretation.

_See also: Concrete machine-semantic type, Migrate/Materialize._

---

## CanonicalMechanicalPassCore

The normative target action algebra for ordinary value passing:

```text
move(move(x)) = move(x)

copy(x) =
  tmp := CopyConstruct(x)
  Move(tmp)

CopyConstruct(x : ordinary T)
  ~= shared_view := share(x); clone(shared_view)

CopyConstruct(x : T ref | T share)
  ~= rebound_view := rebind(x); clone(rebound_view)

automatic pass in {move, copy}
automatic pass not in {ref, share, @}

ProducedMode(source) = mu_source
PolicyMode(destination) = mu_destination
TransferToDestination(source, destination, pass)
  preserves mu_source and installs the destination under mu_destination
```

There is no move of `x` before `CopyConstruct(x)`, and an explicit pass action
dominates automatic selection. Transfer never rewrites the selected producer's
result mode into the destination mode. `CopyConstruct` is the compact name for
the selected ordinary share/clone or rebind/clone copy realization, not a new
opaque semantic primitive; both expansions end in the same terminal move.
Their internal borrow operations do not enlarge automatic pass selection beyond
`{move, copy}`. These laws are canonical even though the selection algorithm,
checker/normalizer integration, IR, and ABI are not yet implemented.

_See also: PolicyMode, Migrate/Materialize._

---

## Type-object

A type-theoretic object: the type of some value, or an object that itself
represents a type. In v0.1 declarations:

- In `let t: type = ...`, `type` is preserved as a bare annotation expression.
- In `let f: _: fn = ...`, `_` is an annotation hole. A later semantic pass may
  interpret it as an anonymous type-object whose kind/rank is given by the
  source name `fn`.

_See also: Kind/rank object, BindingAnnotation, AnnotationHole._

---

## AsType and TypeOf

`AsType(E) = E |> type` does not raise universe rank or preserve a source place.
For a `Symbol` constructor value carrying a type value `tau`, it returns the complete immutable type
closure `tau = bind alpha.<Q,V_τ[alpha]>`. For an already complete type value
`tau`, it is the identity. Bare `Q` is the type-capable core, not the complete type.
It never searches a namespace-role Object for a hidden type member. Symbol's
ordinary `type` field supplies `S.type`, `(S ref).type`, and `(S share).type`;
reaching the type-level place of a pure type slot uses `t |> (type ref)` (not the
retired `t@`).

`TypeOf(E)` is classifier extraction and may move to the next universe. Its
explicit source family is `let <typeof> x : typeof = RHS`, not ordinary
type-expected elaboration. The global `type` object is itself a value of
`type_1`:

```text
TypeOf(type)   = type_1
TypeOf(symbol) = type
rank(type ref/share) = rank(type)
rank(T*N) = rank(T*omega) = rank(T)
```

`type ref` = `RefTy(U_0)` and `type share` = `ShareTy(U_0)` — the operand is
the base universe object `type` itself (`U_1` only classifies it). The
family-wide constructors are `RefTy(T)` / `ShareTy(T)` for every operand
`T : U_n`, with `RefTy(U_n)` / `ShareTy(U_n)` (`n ≥ 0`) as the universe-object
cases, defined in
[`../design/lifetime/lifetime-policy-and-overload-boundary.md`](../design/lifetime/lifetime-policy-and-overload-boundary.md)
§2; `n type ref` / `n type share` are pure metavariable notation, not source
syntax.

_See also: Type-object, Kind/rank object, `@`, `Symbol` constructor value._

---

## Kind/rank object

An object that classifies type-objects. In source text, names such as `fn`
and `type` may appear in explicit rank annotation position:

- `let t: _: type = ...` - the source name `type` occupies the kind/rank
  annotation position for the anonymous type-object `_`.
- `let f: _: fn = ...` - the source name `fn` occupies the kind/rank
  annotation position for the anonymous type-object `_`.

v0.1 does not check kind/rank validity. The parser preserves binding annotation
structure only.

_See also: Type-object, BindingAnnotation, AnnotationTerm._

---

## Namespace (source name)

The source-level name `namespace` as written by a user in a program. In
v0.1, `namespace` is an ordinary `Name` token, not a keyword. Users may
write it in let declaration annotations (e.g., `let ns: namespace = ...`),
but the parser does not interpret it semantically.

> **Distinction**: The conceptual notion of "namespace" as a module/scope
> is distinct from the source name `namespace`.

_See also: Name, Declaration._

---

## `fn` source name

The source-level name `fn` as written by a user. In v0.1, `fn` is an
ordinary `Name` token, not a keyword. It may denote the kind/rank of
function type-objects when used in explicit rank annotation position
(e.g., `let f: _: fn = ...`). The parser does not interpret `fn` as
implying function object construction — that is a future semantic pass.

> **Distinction**: The conceptual "function object" that `fn` may denote
> in the language is a kind/rank classification for function type-objects,
> distinct from the source name `fn` itself.

_See also: Name, Declaration, Kind/rank object, Type-object._

---

## Raw AST

The AST produced directly by the parser, before any lowering or normalization.
Raw AST preserves surface syntax faithfully; it does not desugar or canonicalize
forms. The v0.1 Raw AST frontend is completed and is the input to future
normalization passes.

_See also: Normalized AST, Normalization, Raw AST contract._

---

## Normalized AST

A future desugared AST that unifies call/product forms (product, pipe, operator sugar),
extraction forms (canonical skeletons, deduce lists), and declaration forms
(simple let, extract let, alias let) into simple pattern / call / declaration
structures. Normalized AST is desugared but still non-semantic; it is not HIR,
not type-checked, and not name-resolved.

> **Distinction**: Normalized AST is a structural simplification of Raw AST.
> It does not resolve names, infer types, evaluate canonical forms, materialize
> closures, or insert drops. HIR is a later representation that assumes name
> resolution and type checking.

_See also: Raw AST, Desugaring, Normalization, HIR, Raw AST contract._

---

## Desugaring

Removing surface syntax sugar into simpler normalized forms. Examples:
operator sugar (prefix-negative `-x`, postfix `!`, binary `+`) lowered to named operator
calls; member/double-dot sugar lowered to lookup forms; product placement unified
into a single call structure; extraction skeletons desugared into pattern forms.

Desugaring does **not** perform name resolution, operator lookup, type checking,
overload resolution, canonical matching, or closure materialization.

_See also: Normalization, Normalized AST._

---

## Normalization

The non-semantic lowering pass from Raw AST to Normalized AST. Normalization
produces structurally simpler AST without resolving names, inferring types, or
evaluating semantics. It is the first desugaring pass after parsing.

_See also: Desugaring, Normalized AST, Raw AST, Non-semantic lowering._

---

## Surface-preserving

A property of Raw AST: syntactic sugar and surface forms (operator expressions,
member sugar, double-dot sugar, pipes, products, extraction skeletons) are
preserved as-is in the AST tree. No desugaring or canonicalization is performed
by the parser.

_See also: Raw AST, Desugaring._

---

## Non-semantic lowering

An AST-to-AST transformation that changes the tree shape (e.g., desugaring)
but does not resolve names, infer types, evaluate expressions, or perform
semantic analysis. Normalization is a non-semantic lowering pass.

_See also: Normalization, Desugaring, Raw AST, Normalized AST._

---

## HIR

High-level IR (or High IR) — a future intermediate representation that assumes
name resolution, type checking, and potentially other semantic analysis has been
completed. HIR is later than Normalized AST in the compilation pipeline.

> **Distinction**: Normalized AST is a desugared but still non-semantic
> representation. HIR assumes semantic analysis has already run. Do not call
> Normalized AST "HIR".

_See also: Normalized AST, Non-semantic lowering._

---

## Raw AST contract

The documented invariants of v0.1 Raw AST (`spec/contracts/raw-ast-contract-v0.1.md`)
that future normalization passes may rely on. Defines what each AST node
preserves and what normalization must not assume.

_See also: Raw AST, Normalization, Normalized AST._

---

## Pattern normalization

The semantic `Norm_P` coordinate of Object normal form. Its soundness contract
is one-way: equal normal forms imply relational equivalence preserving
derivation interface and direct structural incidence. It may erase navigation
formation provenance but never the real child or completed navigation.

Frontend desugaring of extraction skeletons and DeduceLists prepares input for
this later semantic operation; it does not itself execute matching or resolve
semantic applicability. Canonical owner: the relational Pattern authority
linked under
Pattern.

_See also: Normalization, CanonicalSkeleton, DeduceList._

---

## Call normalization

Desugaring product/pipe/operator-sugar structures into a unified normalized
call form. Call normalization flattens pipe segments, interprets product placement,
and lowers operator sugar to named operator calls. It does not perform
overload resolution or determine which declaration is being called.

_See also: Normalization, ProductForm, OperatorSugar, PipeExpr._

---

## Declaration normalization

Desugaring let/alias-let forms into normalized declaration forms. Declaration
normalization may preserve optional `with { ... }` clauses and unify
simple and extract let forms into a common structure. It does not resolve
aliases, check types, or decide declaration semantics.

_See also: Normalization, Let binding, Alias binding._

---

## Diagnostic

A structured error, warning, or note produced during lexing or parsing. Every
diagnostic must carry a span. The parser is error-tolerant: it produces
`ErrorAst` nodes alongside diagnostics and continues parsing.

_See also: ErrorAst, Span, diagnostics-v0.1.md._

---

## Golden test

A test that compares tool output (token dump, AST dump, or diagnostic dump)
against a checked-in expected file. Golden tests must be used for every syntax
rule. The dump format must be stable and hand-written, not Rust `Debug` output.

_See also: lexer_golden.rs, parser_golden.rs, diagnostics_golden.rs._

---

## ReturnEvent

A block terminal form representing a targeted return event. It is
not an expression. Raw AST: `FormAst::ReturnEvent(ReturnEventAst)`.
Norm AST: `NormForm::ReturnEvent(NormReturnEvent)`.

_See also: TailValue, ReturnTargetSyntax, Control-flow end event._

## TailValue

The last expression form in a body block, normalized as
`NormForm::TailValue(NormExpr)`. A block result / tail value,
not early return. For an extraction-style callable result it supplies one
object expected to match the complete declared return Pattern, as in
`let ResultPattern = expr`; it does not assign one value to every output
binder.

_See also: ReturnEvent, Control-flow end event._

## ReturnTargetSyntax

The unresolved target syntax of a return event:

```text
NormReturnTargetSyntax ::=
    ImplicitNearest
  | Explicit(NormExpr)
```

`ImplicitNearest` is a historical marker name preserved by the frozen
normalized surface; it carries no resolved target. Its confirmed semantic
interpretation is a return to the outermost enclosing function layer (while a
plain tail `expr;` delivers to the directly enclosing layer, and
`Explicit(T)` selects the layer named by the function-object type `T`). The
current restricted build pass binds it to an active `ReturnTargetFrame`; full
lexical self-capability resolution remains future.
`Explicit(NormExpr)` preserves the explicit target syntax
without resolution.

_See also: ReturnEvent._

## ImplicitNearest return target

A return target marker whose name is historical: in the parser and normalizer,
`ImplicitNearest` is an unresolved marker, and the confirmed semantics of the
source form `E return;` is a return to the outermost enclosing function layer,
not the nearest one. Implementations must not extend behavior based on the
older nearest-enclosing reading. A restricted post-normalization binder
resolves the active frame; result Pattern delivery remains deferred.

_See also: ReturnTargetSyntax, Explicit return target._

## Explicit return target

A return target where the explicit target syntax is preserved
in the AST. In the parser, `Explicit(ExprAst)`; in the
normalizer, `Explicit(NormExpr)`. Source forms are
`E |> (T return);` and `E (T return);`.

The explicit target syntax `T` is not resolved by parser or
normalizer. The restricted build binder supports active name targets through a
temporary spelling identity; full lexical self-capability resolution is
deferred.

_See also: ReturnTargetSyntax, ImplicitNearest return target._

## Control-flow end event

A structural category covering tail values and return events:

```text
Control-flow end event :=
    TailValue(E)
  | ReturnEvent(E, target)
```

Reported by the parser and normalizer as explicit control-flow
data. Not an expression category.

_See also: TailValue, ReturnEvent._

## Terminal block form

A form that ends a body block. Once a terminal form appears,
no later form may occur before `}`:

```text
Terminal block form :=
    TailValue(E)
  | ReturnEvent(E, target)
```

The parser emits `StatementAfterTerminalBlockForm` for forms
after a terminal.

_See also: Control-flow end event._
