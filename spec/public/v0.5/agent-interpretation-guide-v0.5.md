# Agent Interpretation Guide v0.5

> **Status:** Published — v0.5 public guide. This guide is normative for
> coding/documentation agents working on `lang`. The call-binding semantics are
> published in `normalized-surface-semantics-v0.5.md` §3–§7; the value/pattern,
> annotation, and alias boundaries in §8–§10; node origins in §11.

## 1. Read This Before Editing Language Semantics

`lang` does not use conventional call syntax. Before editing any language
behavior, normalization, or documentation, read:

- `spec/public/v0.5/normalized-surface-semantics-v0.5.md` — the normalized
  surface;
- `spec/history/v0.3/normalized-ast-specification-v0.3.md` — the v0.3 design
  baseline (historical route);
- `spec/contracts/v0.4-normalization-prototype-notes.md` — the v0.4 boundary.

If a change requires semantics (resolution, checking, lookup), stop at the
normalized structural boundary and leave the semantics as a documented future
pass.

## 2. Do Not Import Conventional Call Syntax Assumptions

Do **not** read source as C / Rust / Python. The "do not misread" list:

```text
Do not interpret `a b` as traditional function application.
Do not interpret `(a, b)` as an argument list.
Do not interpret `obj.field` as field lookup.
Do not interpret `obj..f(args)` as method dispatch.
Do interpret `.field` as an independently generated field-function closure.
Do not interpret `...args` as a value spread or pack type.
Do not interpret annotation patterns as runtime expressions.
Do not resolve pattern-side names through ordinary function lookup.
Do not treat semantic-looking names (`return`, `else`, `match`, `drop`, `move`) as keywords; they are ordinary `Name` tokens until a later semantic/meta pass interprets them.
Do not turn Normalized AST into HIR.
Do not add name resolution, type checking, operator lookup, or pattern-head resolution to normalization.
```

## 3. Call Binding Rules to Preserve

See `normalized-surface-semantics-v0.5.md` §3–§7 for the full rules. Preserve:

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
  distinct free non-call bare name. Capture initializers are simultaneous and
  see the pre-capture environment.
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

See `normalized-surface-semantics-v0.5.md` §8–§10 for the full rules. Preserve:

- Value-side material stays `NormExpr`; pattern-side material stays `NormPattern`.
  The same source name dumps as `Name` in value position but `PatternName` in
  annotation position.
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
  `Self` space. Standalone closure materialization defaults to an owner-derived
  anonymous receiver type; an associated `()` entry may use a named receiver
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

In particular, normalizing a closure literal or `.name` produces a Raw/Norm
closure carrier, not a callable value. Only a later explicit binding or call
consumer may materialize that carrier.

Source-written captures are explicit binding requirements. `[x]` is
`[let x = x]` with an unwritten (`const || mut`) capture policy, not automatic
const capture. Automatic const requirements need later resolved free-reference
analysis. Capture requirements do not define `self` fields, layout, or ABI.
In-place closures have no capture set: they may read through embedding-layer
lookup but may not directly write an outer place.

## 6. Common Misreadings

- "`a b` must be a call" — no; it is composition into the product-call skeleton.
- "`(args)` after a name is the argument list" — no; it is the source-product
  continuation when an incoming source product exists.
- "`obj.field` looks up a field" — no; it calls the same first-class `.field`
  closure whose body contains unresolved `field::T` navigation; lookup is future.
- "annotation `T Option::std` is an expression" — no; it is annotation-pattern
  material.
- "Normalized AST is basically HIR" — no; HIR assumes resolution and checking.
- "`if` / `else` / `match` are keywords" — no; they are ordinary names; `match`
  is a future library closer, not built-in control flow.

## 7. Where to Put New Material

- Current public language behavior → `spec/public/` (current stage `v0.5`).
- Stage/implementation constraints → `spec/contracts/`.
- Implementation inventory/status → `spec/implementation/`.
- Route, discussion, alternatives, audit trail → `spec/history/`.
- Later semantic design (v0.6+) → `spec/design/`.
- Roadmap and open questions → `spec/planning/`.

If public docs and history conflict, public docs define current behavior. Future
docs must not be read as implemented behavior.
