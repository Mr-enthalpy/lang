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
- `.name` lowers independently to `(val: T, ...args) { (val, args) |> name::T }`;
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
- DeduceList is a binding-site hole binder list (`HoleDecl`); its holes may appear
  as `HoleRef` inside annotation patterns.
- Alias right-hand sides stay unresolved `EntityRef` (dump label `AliasPreserve`),
  never `NormExpr`.
- Pattern-side names are not ordinary call targets and must not fall back to
  ordinary value/function lookup.
- Construction and extraction may be isomorphic; call and extraction are not.
- `...Q` stays `NormPattern::Pack(Q)`, with one pack per normalized level and
  no RHS unpack counterpart. The grammar is shared by every binding slot
  (`let`, parameter, return, and nested product extraction); it is not a
  parameter-only variadic form.
- A canonical Pattern Sequence accepts Pack as a direct child:
  `a ...x b -> Sequence[a, Pack(x), b]`. Ellipsis consumes one following
  Pattern primary; a compound operand needs an explicit boundary such as
  `...(x, y)`.
- Run the global normalized-Pattern validator before downstream build.
  It is the sole authority for pack cardinality across Product, Sequence,
  annotations, local bodies, parameters, and returns. The parser preserves
  syntactically formed Pack nodes and diagnoses only local malformed syntax.

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
