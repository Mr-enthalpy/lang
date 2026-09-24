# Agent interpretation guide

> **Status:** Current public guide. This guide is normative for
> coding/documentation agents working on `lang`. The call-binding semantics are
> published in `normalized-surface-semantics.md` §3–§7; the value/pattern,
> annotation, and alias boundaries in §8–§10; node origins in §11.

## 1. Read This Before Editing Language Semantics

`lang` does not use conventional call syntax. Before editing any language
behavior, normalization, or documentation, read:

- `spec/public/normalized-surface-semantics.md` — the normalized surface;
- `spec/contracts/raw-ast-contract.md` — the enforced syntax handoff;
- `spec/design/README.md` — the canonical semantic topic-owner map.

When editing the frontend, stop at the normalized structural boundary.
Resolution, checking, lookup, and evaluation belong to the canonical semantic
layer and must not be implemented inside parsing or normalization.

## 2. Do Not Import Conventional Call Syntax Assumptions

Do **not** read source as C / Rust / Python. The "do not misread" list:

```text
Do not interpret `a b` as traditional function application.
Do not interpret `(a, b)` as an argument list.
Do not interpret `obj.field` as field lookup.
Do not interpret `obj..f(args)` as method dispatch.
Do interpret `.field` as ordinary `field::adl`; the default generator forms an ordinary closure.
Do not interpret `...args` as a value spread or pack type.
Do not interpret annotation patterns as runtime expressions.
Do not resolve pattern-side names through ordinary function lookup.
Do not treat semantic-looking names (`return`, `else`, `match`, `drop`, `move`) as keywords; they are ordinary `Name` tokens until a later semantic/meta pass interprets them.
Do not turn Normalized AST into HIR.
Do not add name resolution, type checking, operator lookup, or pattern-head resolution to normalization.
```

## 3. Call Binding Rules to Preserve

See `normalized-surface-semantics.md` §3–§7 for the full rules. Preserve:

- The core rule is `Product1 |> TargetExpr Product2 => (Product1, Product2) |> TargetExpr`
  (conceptual: source-product continuation; dump label: `ProductMerge`).
- A following Product is the **first source-product continuation** of an incoming
  source Product, not an argument list of the target. Only the first following
  Product merges; later material is residual.
- `f Product g` is the **second legality repair** (`f |> (Product |> g)`; dump
  label `SecondLegalityRepair`), not a positive local call sugar, and it never
  overrides source-product continuation.
- `P |> e` with no following Product is the **first legality repair** (dump label
  `PipeFallback`), not the main skeleton.
- `expr |> Product` is never the intended normalized result.
- Operator / dot-closure / member / double-dot / bracket sugar lower into the same
  product-call skeleton with preserved provenance; they are not resolved.
- `.name` lowers independently to
  `(self, val: T, ...args) { (val, args) |> name::T }`; the generated `self`
  formal is implicitly supplied, while `val` is the first explicit argument;
  `E.name` mechanically uses that same closure. After lowering, `.name` is an
  ordinary expression: replacing it with `let d = .name` must preserve the
  general pipe/product binding spine. Never inspect `DotClosureLowering`
  provenance to absorb nearby material. `..name` remains direct call sugar.
- Callable tails preserve ordinary/named user bodies, `default`, and optional-
  message `delete`; strategy metadata is not overload selection at normalization.
- Closure placement is independent of head presence. No-`=>` headed bodies,
  including `[[strategy]]`, stay `InPlace`; `=>` bodies are `Ordinary`.
  In-place capture lists are rejected, and malformed callable tails stay Error.
- Ordinary capture clauses are lists of let-shaped bindings. Explicit
  `[let x = E]` and `[x = E]` share `BindingSlot` normalization; shorthand
  `[E]` elaborates to `let n = E` only when normalized `E` has exactly one
  distinct free non-call bare name. Capture initializers share the pre-capture name environment;
  their effects follow ordinary formation order and run once.
- Only the complete `[[Name]] {` shape bypasses an available capture slot.
  Deduce alone leaves that slot open; malformed `[[` strategy recovery is
  reserved for a head independently established by a later component.

Quick continuation checklist:

```text
Incoming source Product (`P |>`) with a following Product?  -> continuation (ProductMerge), not an argument list.
No incoming source Product, naked Product in target position, expr follows?  -> second legality repair (SecondLegalityRepair).
Incoming source Product, no following Product?  -> first legality repair (PipeFallback).
```

## 4. Value/Pattern Boundary Rules to Preserve

See `normalized-surface-semantics.md` §8–§10 for the full rules. Preserve:

- Value-side material stays `NormExpr`; pattern-side material stays `NormPattern`.
  The same source name dumps as `Name` in value position but `PatternName` in
  annotation position.
- `PolicySpec let PipeExpr` in value position is `NormExpr::PolicyLet`, not a
  declaration, hidden binding, or ordinary `const`/`mut` call. Its operand is
  the complete following pipe; parentheses close the Policy context. In a
  Pattern/annotation position it is explicit unsupported Pattern material.
- A value enters pattern space only through an explicit bridge; a pattern exposes
  values only through explicit extraction, binding, passing, or returning.
- Annotations are annotation-pattern (classifier) material, not runtime
  expressions. Inside an `AnnotationPattern`: a DeduceList-declared name →
  `HoleRef`; an undeclared name → `PatternName`; navigation → `PatternNav`; a
  multi-term annotation → `PatternSequence`.
- DeduceList is a left-to-right telescope of `HoleDecl { id, ... }`. Each
  annotation sees inherited and preceding holes, not its own or following
  declarations. Names are unique inside one `PatternRoot`; independent let
  Patterns and nested callable heads create new roots and may shadow inherited
  names. A `HoleRef` targets an exact owner/root-qualified `HoleBinderId`.
  Frontend owners are mapped to persistent `SemanticOwnerId`s before
  multi-root build comparison. Source spans remain provenance.
  BindingSlot policy precedes its local DeduceList.
  Generated receiver holes use hygienic keys, not source spelling. A
  callable-head telescope scopes captures, parameters, call policy, return,
  clauses, body, and inherited nested callables. Exact Norm binding covers
  Pattern/policy occurrences; value-side names/navigation remain unresolved.
  `_` is an anonymous hole, not a named ref.
- Alias right-hand sides stay unresolved `EntityRef` (dump label `AliasPreserve`),
  never `NormExpr`.
- Pattern-side names are not ordinary call targets and must not fall back to
  ordinary value/function lookup.
- `E name [[public/private]]` is a narrow structural member-view annotation
  consumed by `struct`; it is not a general policy slot. Other `[[...]]`
  suffixes remain in the ordinary bracket-call/closure-tail grammar.
- Source navigation is inner-to-outer. A generated call expression used as one
  outer navigation component must be grouped in full:
  `child::(int Vec::std)`.
- Every callable, including in-place, has a semantic owner and callable-local
  `Self` space. Every legal completed closure expression forms full tau_C through struct;
  its c_C, A_C=Type(c_C), and () entry are distinct; an associated `()` entry may use a named receiver
  type instead. Independent let Patterns/callable heads create Pattern roots;
  duplicate holes fail only within one root.
- Construction and extraction may be isomorphic; call and extraction are not.
- `...Q` stays `NormPattern::Pack(Q)`, with one pack per normalized level and
  no RHS unpack counterpart. The grammar is shared by every binding slot
  (`let`, parameter, return, and nested product extraction); it is not a
  parameter-only variadic form.
- A canonical Pattern Sequence accepts Pack as a direct child:
  `a ...x b -> Sequence[a, Pack(x), b]`. Ellipsis consumes one following
  Pattern primary. Raw `...(x, y)` is preserved but rejected after P
  normalization because the bare Product has no stable top mode. A later
  ordered matcher may admit an explicitly headed operand such as
  `...((x, y) pair)`; an unordered layer admits only a whole-remainder
  binder/discard.
- Every Pack contributes one outward specificity node at its containing level.
  Captured width and inner-node count never add same-level EP evidence. Any
  evidence below a stable operand head belongs to the next preserved level.
- Run the global normalized-Pattern validator before downstream build.
  It is the sole authority for pack cardinality, bare-Product Pack rejection,
  and same-PatternRoot hole uniqueness. The parser preserves syntactically
  formed Pack nodes and diagnoses only local malformed syntax.

Quick pattern-context lowering checklist:

```text
Value-side source? Use NormExpr.
Binding / annotation / extraction position? Use NormPattern.
DeduceList-declared name inside annotation? HoleRef.
Undeclared annotation name? PatternName, not NormExpr::Name.
Annotation nav? PatternNav, not value-side Nav.
Alias RHS? EntityRef, not NormExpr.
Expression-like sugar in annotation/pattern context? Keep pattern-side or surface PatternUnsupported; do not lower as value call.
```

## 5. What Normalization Must Not Do

Normalization must not perform name resolution, type/kind checking, operator
lookup or overload resolution, alias target resolution, namespace resolution,
pattern-head resolution, canonical matching, closure materialization, capture
analysis, ownership/NLL/drop, effect interpretation, runtime evaluation, or code
generation. It must not implement pattern-space construction, `Done`
insertion/elimination, `operator+` meta-reduction, exhaustiveness checking, or
`match` closing.

Normalization produces syntax carriers, not semantic values. Every legal
completed closure expression yields full tau_C. The current `.name` generated
helper carrier is pending replacement by ordinary `name::adl`; its provenance
grants no dispatch or binding privilege.

Source-written captures are explicit binding requirements. `[x]` is
`[let x = x]` with no written mode override, not automatic const
capture. Any implicit capture requirement needs later resolved free-reference
and external-eligibility analysis. Capture requirements do not define `self`
fields, layout, or ABI.
In-place closures have no capture set: they may read through embedding-layer
lookup but may not directly write an outer place.

## 6. Common Misreadings

- "`a b` must be a call" — no; it is composition into the product-call skeleton.
- "`(args)` after a name is the argument list" — no; it is the source-product
  continuation when an incoming source product exists.
- "`obj.field` looks up a field" — no; it calls the same first-class `.field`
  selector, canonically `field::adl`; source/evaluator wiring remains pending.
- "annotation `T Option::std` is an expression" — no; it is annotation-pattern
  material.
- "`P let e` is `e |> P` or a hidden `let`" — no; `PolicyLet` preserves a
  result-Policy context that exists before the operand root call is selected.
- "Normalized AST is basically HIR" — no; HIR assumes resolution and checking.
- "`if` / `else` / `match` are keywords" — no; they are ordinary names; `match`
  is a future library closer, not built-in control flow.

## 7. Where to Put New Material

- Current public language behavior → `spec/public/`.
- Enforced implementation handoffs → `spec/contracts/`.
- Canonical semantic topic owners → `spec/design/`.
- Non-authoritative snapshots and design history → `spec/history/`.
- Roadmap and open questions → `spec/planning/`.

Current public docs and canonical topic owners define meaning. The roadmap
states which consumers are connected.


## 8. Semantic construction handoff

NameCoord(root,selector) precedes realization; it is not an Object or Place.
Fresh means not Retained. The typed-name forms below realize that coordinate,
rather than manufacture its identity. Ordinary Val2 writes may change Core
without Pattern or V_tau registration. Generated occurrences supply neither
registration witness, and operator extraction requires an appropriate registered
relation rather than an inferred inverse. The
[operator/declaration owner](../design/patterns-overload/operator-patterns-and-generative-declarations.md)
defines the equal call/value and Pattern/name meta declaration surfaces.

Qualified formation resolves a structural root identity and observes the current
resident type's OpenHere, selector validity, non-retention and ordinary
access/path/type legality. It requires no parent Writable or parent mut type ref.
Equal type values do not merge structural root/name/Place identities. Borrowing
is a separate Place-side judgment. Initialized type names admit direct mut
borrowing or explicit meta type ref followed by ConfirmMut, subject to the same
current OpenHere, target Writable, capability and lifetime checks. These coherent
routes introduce no implicit chain; saved refs retain their borrowed generation
and cannot write after Close. Initial refs remain initialization-only. See the
[type/ref owner](../design/symbol-world/type-values-places-and-borrow-views.md#522-initialized-type-names-meta-references-and-mut-confirmation).

Initializer-free P let name:t and P let name::path:t create typed NameExpr
using lexical and structural destinations respectively, with non-Object
Uninitialized Place state. In (P let name::path), omitted :t defaults to :type,
not an existing type resident. P let name = rhs is a complete lexical binding
with RHS type inference, so that default does not apply. Value use requires initialization; explicit
ref borrows the Place using its declared type without reading. Ordinary write
initializes it using authority independent of the name's declaration policy,
including const. Successful first commit consumes that authority; saved initial
references do not grant replacement power. Later writes require ordinary
replacement capability and resident compatibility. The structural let=compound
is not canonical. Close requires retained structural names being published to be
initialized; it does not require all future generated coordinates to be realized.
Ordinary lexical let remains unchanged.
See [name semantics](../design/symbol-world/names-and-overload-groups.md).
These expression consumers are pending; existing BindingSlot carriers do not
redefine them. Named contributions synthesize types; OverloadGroups aggregate them.

A type contribution requires Writable, OpenHere and final classifier home
Home(TypeOf(v)) = TypeMemberScope(T). [Witnessed anchored replication](../design/symbol-world/closure-anchored-replication.md)
creates a new closure identity and preserves capture obligations; it never
reparents the RHS. Pin elaborates P2 with explicit stage/mode constraints or holes; Pout inherits
P1's stage and may refine mode. Bare let writes no override. Explicit
plain is a concrete constraint, distinct from contextual default completion. Implicit return targets the outermost enclosing
function layer; a current nearest-frame carrier is not semantic authority.

Follow the [semantic spine](../design/semantic-spine.md) for A, lifecycle/unsafe,
host capabilities, source normalization and E. Representation, cache, scheduler
and optimizer structures cannot introduce program facts.

## Canonical evaluation and lifecycle guardrails

- Stage is one atom; meta/compile/seal are mutually incomparable. P2 is horizon,
  P1/Pout producer visibility. InputAdmissible, migration and Ready are separate.
  Runtime P2 defaults omitted P1 to runtime; seal defaults to seal. Omission
  is not a hole. Known runtime inputs do not change the producer stage.
- Resolve once, R_vis evidence, ordinary C_sigma preparation, hard A, fallback
  suppression, Policy/Pattern order, unique Selected=(c*,sigma*,frame).
  No speculative candidate bodies and no runtime reselection.
- Main has runtime horizon. Stable roots do not establish active MetaDom.
  Actual meta/seal frames exclude seal/meta respectively, including through
  compile helpers and caches; deferred work preserves actual dependencies.
- Killable is instance-local; MoveEffect is fixed before observation; Movable
  is frontier legality. Equal types and ZST layout prove no blanket exemption.
  with placement precedes @; killing move adds no old-generation destructor.
- Split/D is restricted residualization, Done is internal boundary completion,
  and residual escape is a separate current-state ordinary meta consumer.
  Return contributes no synthetic local unit.
- Naked operators use operator[op], dot .op uses op::adl, explicit paths stay
  explicit. OG_s retains spelling, and its selector reads the current slot.
- Implementation-layer closure expressions return tau_C; ordinary let binds
  it. Contribution roles are conservative and cannot override legal binding,
  shadowing, write or group behavior.

The [conformance matrix](../planning/canonical-semantic-conformance.md) lists
the acceptance cases. Current carriers and their passing tests are not proof
that these pending consumers are implemented.


## PR106 interpretation boundaries

All directly named entries make a Product layer unordered; one bare entry makes
that layer ordered. Nested layers decide independently of the top name. Named
extraction followed by explicit ordered assembly is required for a bare sequence.

Structured Path precedes external Read; textual roots resolve at actual use,
explicit value/reference roots retain dependencies, and # quotes Path structure.
General $ consumes ready ordinary material in a Pattern context without textual
substitution or rebinding Hole identities. Public Policy pair syntax is retired;
direct source/type Policy projections observe the same edge, whereas a newly
bound type has its own view. Concrete atoms do not implicitly declare holes.

Terminal delivery supplies selected ReturnPattern/Pout demand before immediate
root-call maxima, with both outer P1/P2 constraining both inner positions. No
implicit semantic temp intervenes; explicit user bindings remain boundaries.

General dependencies separate requirements, semantic realization and layout;
snapshot/reference choice is semantic and projections never recapture.
Closure formation uses ordinary struct with a finite implementation leaf and
same-formation first callable. File implementation-layer let installs under the
established package root; true lexical let remains binding. Current in-place
permission permits direct invocation and legal candidate embedding, without
Product/group/type wrappers bypassing it. Lifetime persistence and escape remain
checked through the explicit refinement handoff, not inferred from tau status.
