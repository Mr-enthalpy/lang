# Raw AST Contract v0.1

This document defines what future normalization passes may rely on from
the v0.1 Raw AST output. It is a contract between the completed v0.1 parser
and future Raw AST → Normalized AST lowering.

## Scope

The v0.1 Raw AST is surface-preserving and non-desugared. Every syntactic
form the parser accepts has a corresponding Raw AST node. Future
normalization will desugar this Raw AST into a Normalized AST that unifies:

- call/product forms (product, pipe, operator sugar)
- extraction forms (canonical skeletons, deduce lists)
- declaration forms (simple let, extract let, alias let)

into simple pattern / call / declaration structures.

Normalized AST is:
- desugared, but not name-resolved
- structurally simpler, but not type-checked
- a non-semantic intermediate AST between Raw AST and later semantic phases

Normalized AST is **not**:
- HIR (High-level IR assumes name resolution and type checking)
- MIR
- type checking
- name resolution
- canonical matching
- closure materialization
- interpretation

## Non-goals

Normalization must **not** assume:

- names are resolved
- operators are resolved
- aliases are resolved
- types are checked
- kinds are checked
- canonical skeletons are semantically valid
- deduce holes are semantically valid
- closures are materialized
- match / effect / sync have semantics
- drop / move / ref are semantic events
- `with { ... }` has lifetime or dependency semantics
- ErrorAst means parsing has failed globally (it means a local recovery marker)

## Program / Form invariants

- `ProgramAst.forms` preserves source order.
- `FormAst` distinguishes `Let`, `AliasLet`, `Expr`, and `Error`.
- Forms are not reordered by the parser.

## Let and alias-let invariants

- `LetAst` preserves a `BindingSlotAst` and `span`.
- The absence of a with clause is distinct from explicit `with {}`.
- `WithClauseKind::Empty` preserves `with {}` as an explicit empty modifier.
- `WithClauseKind::Items` preserves non-empty `with { name, ... }` payloads as
  `Vec<NameAst>` only. Each element is a source-level `Name`; symbols, operator
  names, paths, expressions, EntityRef syntax, canonical skeletons, and token
  trees are not accepted. Existence, dependency validity, and lifetime meaning
  are not checked by the parser or normalization; they belong to later name-
  resolution / ownership / lifetime phases.
- `WithClauseKind::Error` preserves malformed `with` syntax without making it AST-equivalent to valid `with {}`.
- `BindingSlotAst` preserves an optional `policy` expression, optional `let`, optional `DeduceListAst`, `BindingPatternAst`, optional `BindingAnnotationAst`, optional `WithClauseAst`, optional initializer, and `span`. A policy is recognized only by the shape `Expr let`; `policy = None` means the policy was unwritten (implicit / inferred later), not "no policy". The parser performs no policy validation.
- `BindingPatternAst` distinguishes `Binder`, `Product`, `Skeleton`, and `Error`.
- `LetAliasAst` preserves an optional `policy` expression (same `Expr let` rule as `BindingSlotAst`), `binder` (`AliasBinderAst`), `target` (`EntityRefAst`), and `span`.
- `AliasBinderAst` distinguishes `Name`, `Operator`, and `Error`.
- `BindingAnnotationAst` distinguishes a single preserved expression, an explicit compound annotation, and `Error`.
- `AnnotationTermAst` distinguishes `Expr` and `Hole`.
- Raw AST preserves binding-site shape. It does not determine whether an annotation denotes a type, rank, custom rank, type object, value object, concept, region, or future classifier. It also does not resolve `with` names or decide same-level binding dependencies.

## DeduceList and CanonicalSkeleton invariants

- `DeduceListAst` preserves declared binder names (`BinderDeclAst`) and their optional annotations.
- `CanonicalSkeletonAst` preserves `Segment`, `ProductExtract`, `Wildcard`, `Name` (with `CanonicalNameRole`), `Path`, `Literal`, and `Error`.
- `CanonicalProductElementAst` is either `Skeleton(CanonicalSkeletonAst)` or `Unit { span }`. Empty positions produced by leading, doubled, or trailing commas in canonical product extraction are preserved as unit elements.
- `CanonicalNameRole` distinguishes `Hole`, `NodeName`, and `Unknown`.
- The `Hole` / `NodeName` distinction is a parse-time role marker. No semantic matching is performed.
- `where` is a reserved closure-head position but is not parsed as an active clause. `acquire` is an ordinary name.
- `FnHeadPrefixAst` carries `clauses: Vec<HeadClauseAst>` for the head clause tail (`require`/`pre`/`post`/`lifetime pre`/`lifetime post`).
- `HeadClauseAst` preserves the clause keyword (`Require`, `Pre`, `Post`, `LifetimePre`, `LifetimePost`, or `Error`) and exactly one `ExprAst` slot. The parser performs no contract, lifetime, resource, type-level, rank-level, or predicate validation on the clause expression.

## Expression invariants

- `ExprAst` is a `Pipe(PipeExprAst)`, `Product(ProductExprAst)`, or `Error(ErrorAst)`.
- `ProductExprAst` preserves ordered `ProductElementAst` elements and span. A parenthesized top-level-comma form in expression context is product construction.
- `ProductElementAst` is either `Expr(ExprAst)` or `Unit { span }`. Empty positions produced by leading, doubled, or trailing commas are preserved as unit product elements. They are not skipped, not wildcards, and not implicit discards.
- There is no standalone `ExprKind::Unit`; unit from commas is scoped to product elements only.
- There is no `BlockExpr`, `ReturnStmt`, `ElseExpr`, `MatchExpr`, `IfExpr`, `IfStmt`, `ElseClause`, `ElseIf`, `MatchStmt`, `CallExpr`, or `ArgPack` as a semantic construct in Raw AST.

## PipeExpr / Segment / Product invariants

- `PipeExprAst` preserves ordered `SegmentAst` values.
- `SegmentAst` preserves ordered `SegmentElementAst` values and `has_incoming` (whether a prior segment exists via `|>`).
- `SegmentElementAst` distinguishes `OperatorExpr` and `Product`.
- Product forms do not receive source/insert/right-target roles. Later normalization may interpret product placement, but Raw AST does not assign call/application roles.

## OperatorExpr invariants

- `OperatorExprAst` preserves `Atom`, `OperatorSugar`, `NavPath`, `MemberSugar`, `DoubleDotSugar`, `BracketCallSugar`, and `Error`.
- `OperatorSugar` preserves the operator name (`OperatorNameAst`), `fixity` (`Prefix`, `Postfix`, `Binary`), `args`, and `span`.
- `OperatorSugar` with `fixity = Prefix` and `operator = "-"` is the sole
  prefix-negative Raw AST shape. Future normalization must lower it by the
  prefix-negative rule (`()zero::(x |> type) - x`) and must not resolve it
  as a prefix operator declaration.
- `NavPath`, `MemberSugar`, `DoubleDotSugar`, and `BracketCallSugar` at the `OperatorExpr` layer exist to support postfix operator suffix continuation (e.g., `obj!.field`, `obj![a]`).
- `BracketCallSugar` preserves an object, the operator name (`OperatorNameAst` with spelling `[]`), and a `ProductExprAst`. It is source-preserving bracket-call sugar (`obj[args...]`); the parser does not lower it or attach indexing/container semantics. `[]` is a contextual paired operator name, also bindable/aliasable/referable in operator-name positions.
- Operator expressions are segment-local: they do not cross `|>` pipe boundaries.

## Atom and suffix-sugar invariants

- `AtomAst` preserves `Name`, `IntLiteral`, `FloatLiteral`, `StringLiteral`, `Group`, `NavPath`, `MemberSugar`, `DoubleDotSugar`, `BracketCallSugar`, `Closure`, and `Error`.
- `NavPath` atoms preserve source-order inner-to-outer navigation components.
- `MemberSugar` preserves an object and a selector.
- `DoubleDotSugar` preserves an object, a selector, and a `ProductExprAst`.
- `BracketCallSugar` preserves an object, the `[]` operator name, and a `ProductExprAst` (the bracket arguments). Left-associative; `obj[a][b]` nests.
- The parser's suffix pipeline includes `:: NavComponent`, `. Selector`, `.. Selector Product`, and postfix operators. In Raw AST, postfix operators are represented at the `OperatorExpr` layer (`OperatorSugar` with `Postfix` fixity), while `AtomAst` preserves navigation/member/double-dot/closure/name/literal/group shapes. Postfix operators do not terminate suffix parsing; e.g., `obj!.field` has the shape `(obj!).field`.

## Closure AST invariants

- `ClosureAst` distinguishes `InPlace` (bare `BodyBlock`, no head) and `Explicit` (`FnHeadPrefix => BodyBlock`).
- A bare `{ ... }` in atom position is an `InPlaceClosureAst`, not a normal block expression. It has no capture clause, no parameter clause, no return clause, and no head clauses.
- `ExplicitClosureAst` requires a non-optional `FnHeadPrefixAst` and a body. Headed closures without `=>` (e.g., `[](){}`) are syntax errors, not valid closure AST.
- `FnHeadPrefixAst` preserves `deduce`, `captures`, `params`, `fn_item_trait`, `returns`, `clauses`, and `span`. The optional clauses may be omitted; `clauses` is the (possibly empty) head clause tail.
- `CaptureClauseAst` preserves ordered `CaptureItemAst` entries. Each `CaptureItemAst` holds a full `ExprAst`, not a name or token tree. The parser does not validate whether a capture expression is movable, borrowable, copyable, lifetime-safe, or admissible as a capture.
- `ParamClauseAst` preserves one `ProductExtractAst`, not a parameter-slot list.
- `ProductExtractAst` preserves ordered `ProductExtractElementAst` elements and span. A parenthesized top-level-comma form in binding / extraction context is product extraction.
- `ProductExtractElementAst` is either `Slot(BindingSlotAst)` or `Unit { span }`. Empty positions produced by leading, doubled, or trailing commas are preserved as unit extraction elements. They are not skipped, not wildcards, and not implicit discards.
- `ReturnClauseAst` preserves a `BindingSlotAst`.
- Parameter and return binding slots reuse the same raw binding-site shape as let, with context-specific restrictions on initializer and `with`.
- `BodyBlockAst` preserves ordered `FormAst` entries and `span`.

## Selector and navigation invariants

- `SelectorAst` distinguishes `Text(NameAst)` for `.` and `..` suffixes. Numeric selectors have been removed.
- Numeric navigation components have been removed. Navigation components accept only `Name` and `OperatorName`. The same token class (`IntLiteral`) produces `IntLiteral` atoms in expression position and `FloatLiteral` atoms for float literals.
- `NavPathAst` preserves source-order `NavComponentAst` entries. Navigation order is inner-to-outer: the leftmost component is the innermost selected symbol, and the rightmost component is the outermost scope component.
- Raw AST performs no lookup for navigation paths.
- Operator names are valid only as innermost navigation components unless a future design explicitly allows operator-named scopes. They are not valid after `.`, `..`, or as outer navigation components after `::`.
- Parenthesized right-side scope expressions after `::` are preserved as grouped navigation components. Without parentheses, `::` consumes only the immediate valid navigation component.
- The innermost navigation component must be a syntactic symbol component (`Name` or `OperatorName`). A grouped expression is valid only as an outer component; used as the innermost component (`(int Vec::std)::ns`) it emits `InvalidNavComponent`.

## EntityRef invariants

- `EntityRefAst` preserves source-order inner-to-outer `NavComponentAst` entries.
- `EntityRef` is parsed only inside alias-let RHS (`let binder === EntityRef`). It is not a general expression parser mode.
- Operator names are valid only as innermost entity-reference navigation components unless a future design explicitly allows operator-named scopes. Outer components must not be operator names.
- Outer entity-reference components after `::` may be `Name` or a parenthesized grouped scope expression (`NavComponentAst::Group`), matching ordinary navigation. The innermost component must be a syntactic symbol component; a grouped expression as the innermost component (`(int Vec::std)::ns`) emits `InvalidEntityRef`.

## Diagnostic / ErrorAst invariants

- `ErrorAst` carries a `message` string and a `span`. It is a local recovery marker, not evidence of global parse failure.
- Every `Diagnostic` carries a `DiagnosticCode`, `message`, and `span`.
- The parser is error-tolerant: it inserts `ErrorAst` nodes and continues parsing.

## Span / source-origin requirements

- All Raw AST nodes that carry spans must preserve source-origin information sufficient for later diagnostics.
- Spans record byte positions (`byte_start` / `byte_end`), line numbers, and column numbers in the CRLF/LF-normalized source text.

## What normalization may assume

- `ProgramAst.forms` preserves source order.
- All node variants enumerated above exist and carry the documented fields.
- Spans are valid and refer to the normalized source text.
- Artifact nodes (`ErrorAst`, `Diagnostic`) carry enough information for diagnostic rewiring.
- Product forms have no role enum; normalization must not assume source/insert/right-target role assignment exists in Raw AST.
- `OperatorSugarAst` with `fixity = Prefix` and `operator = "-"` is the only
  prefix-negative shape. Normalization must lower it to typed-zero binary
  subtraction and must not attempt prefix operator lookup for this node.

## What normalization must not assume

- Names are resolved to declarations.
- Operators are associated with operator declarations.
- Alias targets (`EntityRefAst`) are resolved.
- Types or kinds have been checked or inferred.
- Canonical skeletons are admitted or well-formed.
- `Hole` / `NodeName` roles have been validated.
- Closures have been materialized into callable objects.
- `match` / `effect` / `sync` have been recognized as anything beyond ordinary names.
- `guard` is anything beyond an ordinary name unless future syntax reintroduces it explicitly.
- `with { ... }` carries lifetime or dependency semantics.
- `drop` / `move` / `ref` carry ownership semantics.
- `ErrorAst` nodes indicate that the entire form failed.
- The parser preserved any information not explicitly documented above.
