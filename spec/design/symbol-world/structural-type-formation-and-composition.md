# Structural Type Formation and Composition

Status: canonical construction authority. All actions use the common evaluator
and continuation. The [static/name/structural owner](../unified-static-name-and-structural-semantics.md)
defines interpretation polarity, compile instances and result shape.

## 1. Complete construction and identity

```text
Object = <Val1?, Pattern, Val2>
tau = bind alpha.<Core(tau), V_tau[alpha]>
```

Construction material may accumulate before Pre. ConstructCommit publishes a
complete Object, its actual members and required registered roles atomically.
An Uninitialized Place has no resident Object. Consuming extraction likewise
publishes complete extracted children and ends the decomposed identity in one
action; their destructors follow the ordinary lifecycle continuation.

Core equality observes Object structure. Whole-type observations additionally
observe the immutable bound callspace. NameCoord, NameBinding, Place, resident
generation, classifier home and construction subject retain distinct identities.
Copying or moving an observation cannot manufacture a new opening window.

## 2. Structural interpretation

```text
Interpret(e$, C) = Interpret(e, Flip(C))
Flip(V) = S
Flip(S) = V
BareName(a; p) = a::p
```

The same navigation completion serves Path and extraction consumers.
Structural type formation consumes legally available material in S; nested $
uses ordinary V computation wherever polarity requires it. Interpretation does
not confer resident-read, stage, reference or lifetime authority.

For example, in a value context:

```text
((bool inner)a)$ => tau_a
```

The formation transaction establishes the minimum actual Val2 members together
with DirectPatternChild, ConstructEdge, ExtractEdge, FieldView, navigation and
incidence roles, and LeafSource where applicable. Each role has an actual
member witness:

```text
StructuralRole_P(x) => x in Val2
x in Val2 !=> StructuralRole_P(x)
```

Pattern is the relational coordinate of this complete structure. Ordinary
helper Val2 members may remain outside structural identity. Callability
registration in V_tau is a separate relation; it neither supplies a structural
role nor a named Val2 resident for a callable.

Product ordering is local to each layer. All direct entries named means
unordered; any bare entry makes that layer ordered. Nested layers decide
independently. Unordered material needs named extraction and explicit ordered
assembly before use as a bare sequence.

For a field with source A, structural child C and host T, registered ordinary
members may witness A -> C, C -> A, C -> T and T -> C. FieldView records the
actual registered route; a signature alone is not structural evidence.
Registered callable classifiers satisfy the complete type's /tau home
independently of whether they also enter V_tau. The [Pattern owner](../patterns-overload/pattern-values-relational-semantics-and-extraction.md)
owns applicability, role consistency and protected extraction.

## 3. Ordinary struct helpers

```text
struct : type -> type
T struct = T |> struct
T' = struct(T)
StructuralRegistration(T') = StructuralRegistration(T)
```

The input is an already complete structural type. The ordinary selected call
adds ergonomic interfaces such as ref/share accessors, policy cells, field
write/setter families and associated TypeMember helpers. Those members obey
ordinary formation, classifier-home, registration and capability rules.
DirectPatternChild, ConstructEdge, ExtractEdge and FieldView keep their existing
witnesses and relation. The complete output captures its own immutable V_tau.

Normal invocation uses the actual semantic call entry and its declared planes.
The graph declaration is a rendering projection. Helper implementation
metadata supplies neither Policy nor construction authority.

### 3.1 Ordinary assignment family

A writable reference interface may supply ordinary assignment:

```text
AssignmentFamily(T):
    = : T ref x T -> unit
```

The candidate consumes the actual target reference and complete ordinary source.
Selection and capability applicability precede operation Pre; commit checks
Writable, access, type, borrowed generation, lifecycle and continuation legality.
The write updates the target resident with no structural registration implied.
A share interface supplies observation capabilities and has no assignment
candidate. Its absence is an applicability fact, not a selected Writable failure.
Clone-derived source material follows ordinary share/rebind, selected clone,
fresh result and terminal Move under the same action relations.

## 4. Ordinary type composition and update

```text
*  : type x type -> type
*= : type ref x type -> unit
T * D = T'

r *= D:
    T  = Read(r)
    T' = T * D
    Write(r, T')
```

These are ordinary operator candidates, selected through applicability,
specificity, Policy preference and one seal. Composition produces a complete
new snapshot; previous snapshots remain immutable. Conflict and child
uniqueness are checked in the resulting registered structure. Equal child
values do not justify replaying a registration or merging distinct entries.

Update checks current OpenHere of the read type, actual target Writable,
capability, access, lifecycle and continuation legality. The selected action
may realize read-transform-write within one transaction. Failed Pre or failed
formation publishes nothing. A selected failure never reopens candidates.
Ordinary Val2 initialization/replacement and TypeAdd remain distinct actions:
TypeAdd changes only V_tau, under its own anchoring and registration premises.

## 5. Opening authority

```text
Anchor(v) = <StructuralRoot(v), Navigation(v)>
OpenHere_K(v) = WindowLive_K(v) and AuthorityMatches_K(v)
```

Each closure retains its lexical, Self, navigation and dependency layer.
Authority is checked at the actual construction subject and admitted dependency
edges. A source-preserving dependency carries its existing coordinate and
window facts at its actual lexical and navigation coordinates. Every write
rechecks the current subject, borrowed generation and
ordinary access/capability/lifetime facts.

```text
Writable(q) !=> OpenHere(Read(q))
OpenHere(v) !=> Writable(Carrier(v))
r : type ref !=> OpenHere(Read(r))
Visible(v) !=> Ready(v)
Ready(v) !=> LegalToExecute(v)
```

Close irreversibly ends its construction window. Visibility loss, masking,
copying, cache reacquisition and reference retention cannot reopen it. An
explicit or deduced value mode remains independent. Only omitted mode
completes to mut under open plus current known OpenHere, const otherwise;
required unknown OpenHere is unavailable.

Construction windows follow their established control-flow coordinates.
Disposition Continue, Terminate or Reject is checked before the action.
Generation-coordinate split/merge and leaving the owning interval constrain
the linear window; unrelated deeper control flow supplies no new authority.
Dependencies and liveness are different observations. Lifetime, owned transfer
and escape are independently checked at the actual destination.

## 6. Compile instances and result delivery

Every selected compile computation establishes its stable instance before
EnterBody, including scalar, Product, closure and existing-type results.
Its self-name is an ordinary NameValue; Read_name and Read_resident do not
enter the body. The [invocation owner](../static-evaluation/compile-instance-invocation-and-result-delivery.md)
owns the key, current storage, opening-source meet and result delivery.

An instance-open self-root type can escape only as the single direct complete
type governed by that instance. Reachability includes captures, wrappers and
borrow targets as well as owned members. Complete delivery retains real
ReturnEvent and result assignment/replacement even for an equal self result.
OpenPolicy=open preserves an established opening source; close completes and
closes after successful ordinary result delivery. External inputs keep their
own subjects, windows, targets and regions.

## 7. Closures and anchored contributions

Every legal completed closure expression forms full tau_C from its head,
implementation and admitted dependency material. Its first ordinary callable
c_C, classifier A_C and terminal () entry have distinct identities; formation
terminates at the implementation leaf. Ordinary let binds tau_C:type.

An established same-name contribution bucket instead jointly consumes sibling
closure materials against one snapshot, forms target-anchored c_C^T members
and initializes once. The [name owner](names-and-overload-groups.md) owns that
role and full-type bucket equality. A known target directly forms its members;
rehosting an already formed callable requires the [replication witness](closure-anchored-replication.md).
It preserves dependencies and internal alpha identities while keeping the
original callable unchanged. Complete-type home is not inferred from Core.

## 8. Source, names and publication

Physical hierarchy provides inherited navigation:

```text
NavigationContext(a) |- inner => inner::a
a/ { P let inner = rhs; } => P let inner::a = rhs;
```

Ordinary name consumers perform formation, inference, initialization and
replacement. File provenance grants no authority. Sibling files use independent
common-snapshot overlays and ordinary unordered effect join; source actions
within one file retain their sequence.

Initializer-free typed names create a NameExpr and an Uninitialized Place;
explicit borrowing and ordinary first write initialize it with one-shot
formation authority independent of declared const/mut. Complete let bindings
infer their RHS type. Name production and NameExpr result shape are independent.
Lexical === saves Path material formed in the old environment, retaining
anchors/dependencies and normal later Path composition.

Every semantic publication consumes one common transaction with all affected
Pre and Post projections. Outstanding cleanup forbids crossing its fixed point.
Formation, origin and Color Post are published in the producer's scratch
transaction. Private storage staging is an implementation facility, not Ready,
Pre or successful semantic action evidence. Namespace indices project only
committed facts.

## 9. Representation and consumer boundaries

Persistent identity encoding, Pattern proof IR, access-summary algorithms and
continuation storage remain representation questions in the [open questions](../../planning/open-questions.md).
Closed relations lacking a consumer report their actual unavailable boundary
or preserve an established remaining continuation. They supply no alternate
identity, authority, result class or evaluator.
