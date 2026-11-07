# Control-flow end events contract

This contract records syntax preservation and active return-target binding.
Canonical completion belongs to the
[targeted-return owner](../design/control-flow/targeted-return-and-d-reduction.md).

**Status:** Raw/Norm return syntax and boundary-aware omitted-target binding are
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
Its `outermost_enclosing_function_target()` query selects the outermost function
in the reachable segment bounded inclusively by the nearest Ordinary closure.
InPlace adds no barrier. Frames record `ReturnBoundary::{Ordinary, InPlace,
Unknown}`; `push_frame` without evidence records Unknown, not InPlace. Missing
required boundary evidence preserves omitted and explicit events unbound.
Without an active frame it reports ReturnOutsideReturnableContext.

Explicit target resolution consumes a callable-self identity from the semantic
resolver, matches active frames by identity, and diagnoses inactive or ambiguous
targets. It never resolves by spelling or falls back to another target.
An unresolved target requiring the unconnected semantic resolver is preserved.
An active target outside the segment reports ReturnTargetOutsideReachableSegment.
The shared `target_reachability` guard also constrains future return-capability
calls; possession/capture of an outer capability does not authorize crossing.

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

## 6. Effect handoff is a separate ordinary result protocol

The [effect owner](../design/control-flow/effect-steps-and-continuation-handoff.md)
defines wait as an expression cut, yielding Wait(y,k), and normal effect
completion as matchable Done(v). These are ordinary effect sum branches, not
the internal completion in §5. A repair attachment applies to the complete
expression; the suspending side proves atomic Move[repair], default id, before
exposing k. Resume is an ordinary new call of that concrete callable.

Throw is the specified Wait/reentry/return abbreviation. Propagating its ordinary
effect through call results does not cross the Ordinary targeted-return barrier.
The handler may drop the delivered continuation through ordinary destruction;
no specialized unwind is inserted. Only selected forced actual matching failure
can produce PatternMiss, never candidate rejection or ordinary branch miss.

Input/cut, selected origin, Pre/commit/Post, valid successor AutoPre, cleanup
responsibility and resume Pattern must be preserved by the future consumer.
Wait/throw/handler execution and their source carriers are pending; generic
Name/operator call syntax is not evidence that these protocols execute.

Their common-E prerequisite is
[whole-original-continuation CPS conversion](../design/static-evaluation/evaluation-residual-and-optimization.md#71-whole-original-continuation-cps-conversion-consumer-pending).
Pending callers, serial input/binding/result consumers, handler boundaries,
active target-return identities and destructor/cleanup continuations must all
have the same explicit continuation correspondence. A Wait originating inside
a nested call cannot discard the caller suffix or turn ordinary effect
propagation into target-return barrier crossing. This is a pending semantic
continuation consumer, not Raw/Norm CPS lowering or special async ABI.
