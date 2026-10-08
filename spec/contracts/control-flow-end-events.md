# Control-flow end events contract

This contract records syntax preservation and active return-target binding.
Canonical completion belongs to the
[targeted-return owner](../design/control-flow/targeted-return-and-d-reduction.md).

**Status:** Raw/Norm return syntax and outermost omitted-target binding are
implemented. Serial UnitDiscard, path-tail completion, full lexical capability
resolution, whole-Pattern result delivery and completion propagation remain
unconnected.

## 1. Syntax carriers

`return` is a contextual Name token, not a lexer keyword. The parser
recognizes these explicit event forms:

```text
E return;
E |> (return);
E (return);
E |> (T return);
E (T return);
```

The first three omit the target; the last two retain explicit target syntax.
Calls and return forms preserve pipeline direction.

```text
FormAst::ReturnEvent(ReturnEventAst {
  value: ExprAst,
  target: Omitted { span } | Explicit { target: ExprAst, span },
  span
})

NormForm::ReturnEvent(NormReturnEvent {
  value: NormExpr,
  target: Omitted | Explicit(NormExpr),
  origin
})
```

These are structural forms, not ordinary expression calls. ReturnEvent cannot
occur as a Product element, Pattern, call argument, annotation or initializer.
Such embedded uses produce spanned diagnostics. The whole form
`E (T return);` remains valid; it is not a call taking a return expression.

Every ordinary expression remains `FormAst::Expr` / `NormForm::Expr`,
including the last list entry. Normalization neither synthesizes a return
event nor labels a semantic tail.

## 2. Parser and structural validation

Only an explicit ReturnEvent marks a block terminal for source recovery.
Following forms before the closing brace produce
`StatementAfterTerminalBlockForm`; extra semicolons remain separators.
The build structural report likewise records only explicit ReturnEvent
terminals and diagnoses subsequent forms.

Plain serial expressions may precede other expressions or declarations.
The parser does not know their result types or continuation-path positions.
Allowing their syntax does not establish their semantic legality.

## 3. Target binding

`ReturnTargetStack` records active function frames and their stable identities.
Its `outermost_enclosing_function_target()` query selects the first active
function frame. An Omitted target on an existing event uses this query.
Without an active frame it reports ReturnOutsideReturnableContext.

Explicit target resolution consumes a callable-self identity from the semantic
resolver, matches active frames by identity, and diagnoses inactive or ambiguous
targets. It never resolves by spelling or falls back to another target.
An unresolved target requiring the unconnected semantic resolver is preserved.

Nested unmaterialized closure literals preserve their returns for later
elaboration rather than binding them to the surrounding frame.
Each frame retains the complete normalized return binding slot and Pattern
in ReturnSlotRef. Binding identifies a recipient; it does not execute delivery.

## 4. Two separate implicit operations

The canonical continuation consumer is:

```text
plain expression completion
  -> non-tail: UnitDiscard (evaluate once; require unit; discard; continue)
  -> path-tail unit: ordinary fallthrough
  -> path-tail non-unit: synthesize ReturnEvent
                         -> infer omitted ReturnTarget
```

An explicit `unit_value return;` already produces ReturnEvent and omits only
its target. Unit fallthrough and return of unit are distinct.
Tailness follows each SemanticContinuation path, including branch paths,
rather than AST-list position. UnitDiscard is an ordinary E consumer, not a
source rewrite, hidden binding or second evaluator.

Until this consumer is connected, selected source bodies requiring serial
completion report an explicit unsupported diagnostic. A final name cannot
stand in for execution or forward an argument as a fabricated result.

## 5. Delivery and completion frontier

Return payload evaluation receives the established target ReturnPattern/Pout
demand before its root call seals maxima. Both outer P1 and P2 constrain the
immediate inner positions. No unconstrained temporary is completed first;
later use cannot reopen the selected call.

Completion is internal and target/chain-qualified. It is not an Object or
Pattern, has no public constructor, and cannot re-enter lookup, storage, @,
ref/share or ordinary value observation. Return contributes no synthetic local
unit value. Whole-Pattern delivery checks the ordinary payload against the
target Pattern as one ordinary result delivery.

The following consumers remain unconnected:

- full lexical self-capability target resolution;
- UnitDiscard and path-sensitive implicit ReturnEvent;
- whole-Pattern result delivery under terminal demand;
- early return/control-flow propagation;
- D-reduction and Done_Return.

The implemented syntax and binding substrate do not substitute for these
relations.
