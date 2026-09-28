# Control-flow end events contract

Contract for the syntax and normalized structure of control-flow end events
(return terminal forms and tail values).

**Status:** Implemented syntax/normalized structure plus build-layer
return-target substrate. Outermost implicit-target selection is pending
alignment; full self-capability resolution, result Pattern
delivery, D-reduction, Done_Return, and return execution are not implemented.

## 1. Scope

This contract covers:

- Source / Raw AST / Norm AST reporting of control-flow end events.
- Parser and normalizer contracts for terminal block forms.
- The `ReturnTargetStack` / active-frame binding substrate.
- Handoff expectations for future semantic consumers.

It does **not** implement or specify:

- D-reduction / `Done_Return` (not implemented).
- Early-return execution (not implemented).
- Control-flow propagation (not implemented).
- Whole-Pattern result delivery (not implemented).
- Full lexical self-capability target resolution. The current binder records
  stable callable-owner identities for active frames; the source resolver that
  supplies an explicit target identity is not connected.

## 2. Implemented Event Categories

The current normalized carrier distinguishes two structural categories.
These are not the full canonical completion algebra:

```
Control-flow end event :=
    TailValue(E)
  | ReturnEvent(E, target)
```

| Event | Meaning | Norm form |
|---|---|---|
| `TailValue(E)` | Current final-expression carrier; semantic path-tail and unit/non-unit consumer still required. | `NormForm::TailValue(NormExpr)` |
| `ReturnEvent(E, ImplicitNearest)` | Early return whose target is unresolved. Semantic binding selects the outermost enclosing function layer. | `NormForm::ReturnEvent(NormReturnEvent { target: ImplicitNearest })` |
| `ReturnEvent(E, Explicit(T))` | Early return to the layer selected by the function-object type target `T`. Target unresolved. | `NormForm::ReturnEvent(NormReturnEvent { target: Explicit(NormExpr) })` |

TailValue is currently assigned from list position. It is implementation
debt, not proof of semantic tail position. The canonical consumer is defined
by the [return owner](../design/control-flow/targeted-return-and-d-reduction.md#11-two-distinct-implicit-operations):
non-tail expressions require unit via UnitDiscard; true path-tail unit falls
through; path-tail non-unit synthesizes ReturnEvent and then infers its target.
An explicit E return already supplies an event, even for unit, and omits only
the target. These two implicit operations must not be merged.

## 3. Parser Contract

### 3.1 Lexical Level

`return` remains a `TokenKind::Name` token. It is **not** a lexer
keyword. The parser recognizes it contextually in return terminal form
positions.

### 3.2 Return Terminal Forms

The parser recognizes these only at form / terminal-form level:

```text
E return;
E |> (T return);
E (T return);
E |> (return);
E (return);
```

`E |> (return);` and `E (return);` are also accepted (the parser
produces `ImplicitNearest` when the group contains only `return`
without an explicit target expression). These are equivalent to
`E return;`.

Their Raw AST meanings:

```text
E return;
  => FormAst::ReturnEvent(ReturnEventAst {
       value: E,
       target: ReturnTargetAst::ImplicitNearest { span }
     })

E |> (return);
  => FormAst::ReturnEvent(ReturnEventAst {
       value: E,
       target: ReturnTargetAst::ImplicitNearest { span }
     })

E (return);
  => FormAst::ReturnEvent(ReturnEventAst {
       value: E,
       target: ReturnTargetAst::ImplicitNearest { span }
     })

E |> (T return);
  => FormAst::ReturnEvent(ReturnEventAst {
       value: E,
       target: ReturnTargetAst::Explicit { target: T, span }
     })

E (T return);
  => FormAst::ReturnEvent(ReturnEventAst {
       value: E,
       target: ReturnTargetAst::Explicit { target: T, span }
     })
```

`T` is target syntax only. It is **not** resolved by parser or
normalizer.

### 3.3 Raw AST Types

```text
FormAst ::= ...
  | ReturnEvent(ReturnEventAst)

ReturnEventAst {
  value: ExprAst,
  target: ReturnTargetAst,
  span: Span
}

ReturnTargetAst ::=
    ImplicitNearest { span }
  | Explicit { target: ExprAst, span }
```

### 3.4 Non-Expression Guarantee

```text
ReturnEvent ∈ BlockTerminal / Form
ReturnEvent ∉ Expr
ReturnEvent ∉ Pattern
ReturnEvent ∉ Group
```

Return events are **not** expressions. They cannot be embedded in
expression, pattern, group, call-argument, product-element, annotation,
or let-initializer contexts.

These are diagnostic-bearing when embedded:

```lang
(x return)
(x |> (T return))
(x (T return))

let y = (x return);
let y = x |> (T return);
let y = x (T return);

(x return) |> g;
(x |> (T return)) |> g;
(x (T return)) |> g;

(x return) + y;
let y: (x return) = z;
```

But this is legal as a whole terminal form:

```lang
f (x return);
```

It means:

```text
ReturnEvent(value = f, target = Explicit(x))
```

It is **not** a call with `x return` as an argument.

### 3.5 Terminal Block Rule

The current parser applies the following restriction (including to plain
expressions); this is migration debt for serial UnitDiscard, not a canonical
ban on non-tail unit expressions. Once it marks a terminal form, no later form may occur before
`}`. Terminal forms are:

```text
TailValue(E)                    (bare expression in final position)
ReturnEvent(E, ImplicitNearest) (E return)
ReturnEvent(E, Explicit(T))     (E |> (T return) or E (T return))
```

Parser diagnostic: `StatementAfterTerminalBlockForm`.

Extra semicolons after a terminal form are tolerated as separators.
Actual following forms generate the diagnostic.

## 4. Normalizer Contract

### 4.1 TailValue

The last expression form in each body block is normalized as:

```text
NormForm::TailValue(NormExpr)
```

The normalizer records list position structurally. It cannot determine
continuation-path tailness, result type, fallthrough or synthesized return.

### 4.2 ReturnEvent

`FormAst::ReturnEvent` normalizes to:

```text
NormForm::ReturnEvent(NormReturnEvent)
```

with:

```text
NormReturnEvent {
  value: NormExpr,
  target: NormReturnTargetSyntax,
  origin: NormOrigin
}

NormReturnTargetSyntax ::=
    ImplicitNearest
  | Explicit(NormExpr)
```

`Explicit(NormExpr)` preserves the unresolved target syntax. The
normalizer does **not** resolve `Self` or any other target expression.

### 4.3 Normalized Form Family

```text
NormForm ::=
    Let(NormDecl)
  | Alias(NormDecl)
  | Expr(NormExpr)
  | TailValue(NormExpr)
  | ReturnEvent(NormReturnEvent)
  | Error(NormError)
```

### 4.4 Non-Call Guarantee

Return events are not represented as ordinary calls:

```text
✗ NormExpr::Call { target: Name("return"), ... }
✗ NormExpr::Call { target: OperatorTarget("|>"), ... }
```

They are structurally distinct `NormForm` variants.

## 5. Non-Expression Contract

Return events and tail values are terminal form structures, not
expressions.

```text
ReturnEvent ∉ Expr
ReturnEvent ∉ Pattern
ReturnEvent ∉ Group
ReturnEvent ∈ BlockTerminal / Form
```

Future semantics must not treat return events as expressions,
call targets, or ordinary value producers without explicit
control-flow lowering.

## 6. Current terminality carrier restriction

```text
No later form may appear after a terminal block form before `}`.
```

The parser enforces this with `StatementAfterTerminalBlockForm`.
Semicolons after a terminal form are tolerated. Actual following
forms are diagnosed.

## 7. Consumer Handoff

### 7.1 Explicit Control-Flow End Reports

Semantic consumers preserve explicit ReturnEvent reports, but must derive
plain-expression completion from the established SemanticContinuation path.
A final AST-list position is insufficient; each branch path has its own tail.
The current TailValue carrier must feed this consumer, not replace it.

```text
plain expression completion -> serial/block consumer
  -> UnitDiscard if non-tail
  -> fallthrough if tail and unit
  -> synthesize ReturnEvent if tail and non-unit
     -> infer omitted ReturnTarget
```

UnitDiscard is an internal consumer of ordinary E, not a lexical rewrite,
new primitive or second evaluator. The current parser restriction and carrier
tags do not implement this relation.

### 7.2 Unresolved Target Syntax

The explicit target syntax in `NormReturnTargetSyntax::Explicit(NormExpr)`
is preserved verbatim. Semantic target resolution is deferred to a
later elaboration phase:

```text
Parser:
  recognizes return terminal shape and creates ReturnEventAst.

Normalizer:
  preserves ReturnEvent as NormForm::ReturnEvent and normalizes
  the value / explicit target syntax.

Build elaboration:
  must resolve the implicit target to the outermost enclosing function layer,
  or a supported Explicit target by its identity, and retain the complete return binding
  slot/Pattern.

Later result/completion semantics:
  resolves the full lexical self capability and delivers the result by
  matching `let ResultPattern = value`.
```

The parser and normalizer do not resolve `Self`. The build binder consumes a
stable callable-owner identity supplied by a semantic resolver and does not
perform whole-Pattern result delivery.

### 7.3 Deferred Semantics

Canonical completion is internal and chain/target-qualified. It is not an
Object, Pattern, source Done constructor or observable value; no lookup, @,
ref/share or storage consumer can re-enter it. Return has no fabricated local
unit contribution and delivers its ordinary payload against the target's
ReturnPattern. These laws are owned by the targeted-return and extraction-chain
documents; the following carriers do not implement them.

The following are **not** implemented and must not be assumed by consumers of
this contract:

```text
- full lexical self-capability target resolution
- D-reduction
- Done_Return
- Early-return execution
- Control-flow propagation
- Whole-Pattern result delivery
```

The existing binder selects its most recent active frame for implicit returns;
this behavior must be aligned to outermost selection. It supports explicit-name
targets and reports returns outside
a returnable frame, preserves nested unmaterialized closure returns for later
elaboration, and stores the complete `NormBindingSlot` in `ReturnSlotRef`.


The Raw/Norm tag ImplicitNearest is a current carrier spelling for an omitted
target on an already explicit event; it does not describe implicit event inference. It does not
authorize nearest-frame semantics. No syntax change or parser name resolution
is implied by the outermost rule.


## 8. Direct terminal demand

The terminal payload is evaluated directly under the target's established
ReturnPattern/Pout demand, before the payload root call seals its candidate
maxima. Delivery does not first complete an unconstrained temporary and then
rematch it. An explicitly written user binding still creates its own boundary.
Both established outer P1 and P2 constrain the immediate inner call's P1/P2;
the selected inner call cannot be reopened by later use. See
[Policy demand](../design/symbol-world/symbol-policy-and-compile-flow-projection.md).
