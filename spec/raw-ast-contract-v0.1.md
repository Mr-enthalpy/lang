# Raw AST Contract v0.1

This document defines what future normalization passes may rely on from
the v0.1 Raw AST output. It is a contract between the completed v0.1 parser
and future Raw AST → Normalized AST lowering.

## Scope

The v0.1 Raw AST is surface-preserving and non-desugared. Every syntactic
form the parser accepts has a corresponding Raw AST node. Future
normalization will desugar this Raw AST into a Normalized AST that unifies:

- call forms (ArgPack, pipe, operator sugar)
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
- guard / with have lifetime semantics
- ErrorAst means parsing has failed globally (it means a local recovery marker)

## Program / Form invariants

- `ProgramAst.forms` preserves source order.
- `FormAst` distinguishes `Let`, `AliasLet`, `Expr`, and `Error`.
- Forms are not reordered by the parser.

## Let and alias-let invariants

- `LetAst` preserves `attrs` (guard attributes), `binder`, `with_deps`, `value`, and `span`.
- `LetBinderAst` distinguishes `Simple` (name + annotation), `Extract` (deduce + skeleton), and `Error`.
- `LetAliasAst` preserves `binder` (`AliasBinderAst`), `target` (`EntityRefAst`), and `span`.
- `AliasBinderAst` distinguishes `Name`, `Operator`, and `Error`.
- `DeclAnnotationAst` distinguishes `Bare` (single expression), `TypeObjectWithRank` (type-object + rank), and `Error`.
- `TypeObjectAnnotationAst` distinguishes `Expr` and `Hole`.

## DeduceList and CanonicalSkeleton invariants

- `DeduceListAst` preserves declared binder names (`BinderDeclAst`) and their optional annotations.
- `CanonicalSkeletonAst` preserves `Segment`, `ArgPack`, `Wildcard`, `Name` (with `CanonicalNameRole`), `Path`, `Literal`, and `Error`.
- `CanonicalNameRole` distinguishes `Hole`, `NodeName`, and `Unknown`.
- The `Hole` / `NodeName` distinction is a parse-time role marker. No semantic matching is performed.
- `where` and `acquire` are reserved closure-head positions but are not parsed as active clauses.

## Expression invariants

- `ExprAst` is either a `Pipe(PipeExprAst)` or `Error(ErrorAst)`.
- There is no `BlockExpr`, `ReturnStmt`, `ElseExpr`, or `MatchExpr` node in Raw AST.

## PipeExpr / Segment / ArgPack invariants

- `PipeExprAst` preserves ordered `SegmentAst` values.
- `SegmentAst` preserves ordered `SegmentElementAst` values and `has_incoming` (whether a prior segment exists via `|>`).
- `SegmentElementAst` distinguishes `OperatorExpr` and `ArgPack`.
- `ArgPackAst` preserves `args`, `role`, and `span`.
- `ArgPackRole` preserves `SourcePack`, `InsertPack`, `RightTargetSubsegment`, and `Unknown`.

## OperatorExpr invariants

- `OperatorExprAst` preserves `Atom`, `OperatorSugar`, `Path`, `MemberSugar`, `DoubleDotSugar`, and `Error`.
- `OperatorSugar` preserves the operator name (`OperatorNameAst`), `fixity` (`Prefix`, `Postfix`, `Binary`), `args`, and `span`.
- `Path`, `MemberSugar`, and `DoubleDotSugar` at the `OperatorExpr` layer exist to support postfix operator suffix continuation (e.g., `obj!.field`).
- Operator expressions are segment-local: they do not cross `|>` pipe boundaries.

## Atom and suffix-sugar invariants

- `AtomAst` preserves `Name`, `IntLiteral`, `StringLiteral`, `Group`, `Path`, `MemberSugar`, `DoubleDotSugar`, `Closure`, and `Error`.
- `Path` atoms preserve a base and ordered selector leaves.
- `MemberSugar` preserves an object and a selector.
- `DoubleDotSugar` preserves an object, a selector, and an `ArgPackAst`.
- The parser's suffix pipeline includes `:: Selector`, `. Selector`, `.. Selector ArgPack`, and postfix operators. In Raw AST, postfix operators are represented at the `OperatorExpr` layer (`OperatorSugar` with `Postfix` fixity), while `AtomAst` preserves path/member/double-dot/closure/name/literal/group shapes. Postfix operators do not terminate suffix parsing; e.g., `obj!.field` has the shape `(obj!).field`.

## Closure AST invariants

- `ClosureAst` distinguishes `Inline` (`FnHeadPrefix? BodyBlock`) and `Explicit` (`FnHeadPrefix => BodyBlock`).
- `FnHeadPrefixAst` preserves `deduce`, `captures`, `params`, `fn_item_trait`, `returns`, and `span`. All clauses are optional.
- `CaptureClauseAst` preserves ordered `CaptureItemAst` entries containing expression AST.
- `ParamClauseAst` preserves ordered `ParamItemAst` entries.
- `ParamItemAst` distinguishes `NameParam` (name + optional annotation), `ExtractParam` (deduce + skeleton + optional annotation), and `Error`.
- `ReturnClauseAst` preserves a `ReturnBinderAst` and optional constraint expression.
- `ReturnBinderAst` distinguishes `TypeExpr`, `ExtractType` (deduce + skeleton), and `Error`.
- `BodyBlockAst` preserves ordered `FormAst` entries and `span`.

## Selector and operator-name invariants

- `SelectorAst` distinguishes `Text(NameAst)`, `Numeric(NumericNameAst)`, and `Operator(OperatorNameAst)`.
- Numeric selectors (`obj.1`, `uint8::1`) use `NumericNameAst` in selector position. The same token class (`IntLiteral`) produces `IntLiteral` atoms in expression position.
- Operator selectors are valid only as final path leaves after `::`. They are not valid after `.` or `..`.

## EntityRef invariants

- `EntityRefAst` preserves ordered `EntityPathSegmentAst` entries (text names) and a final `EntityPathLeafAst` (Name or OperatorName).
- `EntityRef` is parsed only inside alias-let RHS (`let binder === EntityRef`). It is not a general expression parser mode.
- Operator names are valid only as final `EntityPathLeaf`. Intermediate segments must be text names.

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
- `ArgPackRole` assignment is position-based and deterministic.

## What normalization must not assume

- Names are resolved to declarations.
- Operators are associated with operator declarations.
- Alias targets (`EntityRefAst`) are resolved.
- Types or kinds have been checked or inferred.
- Canonical skeletons are admitted or well-formed.
- `Hole` / `NodeName` roles have been validated.
- Closures have been materialized into callable objects.
- `match` / `effect` / `sync` have been recognized as anything beyond ordinary names.
- `guard` / `with` carry lifetime semantics.
- `drop` / `move` / `ref` carry ownership semantics.
- `ErrorAst` nodes indicate that the entire form failed.
- The parser preserved any information not explicitly documented above.
